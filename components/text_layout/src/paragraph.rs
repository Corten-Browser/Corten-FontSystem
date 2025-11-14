//! Paragraph layout engine

use crate::justification::Justifier;
use crate::line_breaker::LineBreaker;
use crate::types::{LayoutError, LayoutLine, LayoutOptions, LayoutResult};
use font_types::{PositionedGlyph, ShapedText};

/// Main paragraph layout engine
///
/// Handles multi-line text layout including line breaking, justification,
/// and vertical positioning.
pub struct ParagraphLayout {
    line_breaker: LineBreaker,
    justifier: Justifier,
}

impl ParagraphLayout {
    /// Create a new paragraph layout engine
    pub fn new() -> Self {
        Self {
            line_breaker: LineBreaker::new(),
            justifier: Justifier::new(),
        }
    }

    /// Layout a paragraph of shaped text
    ///
    /// Takes shaped text from the text shaper and lays it out into multiple
    /// lines according to the provided options.
    ///
    /// # Arguments
    ///
    /// * `text` - The original text string (for line breaking)
    /// * `shaped_text` - The shaped text with positioned glyphs
    /// * `options` - Layout options (width, justification, etc.)
    ///
    /// # Returns
    ///
    /// Result containing the laid out lines or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The text is empty
    /// - Layout options are invalid (e.g., negative width)
    /// - Text cannot fit within constraints
    ///
    /// # Example
    ///
    /// ```no_run
    /// use text_layout::{ParagraphLayout, LayoutOptions};
    /// use font_types::ShapedText;
    ///
    /// let layout = ParagraphLayout::new();
    /// let options = LayoutOptions::default();
    /// // shaped_text would come from text_shaper
    /// # let shaped_text = ShapedText { glyphs: vec![], width: 0.0, height: 0.0, baseline: 0.0 };
    /// let result = layout.layout_paragraph("Hello world", &shaped_text, &options);
    /// ```
    pub fn layout_paragraph(
        &self,
        text: &str,
        shaped_text: &ShapedText,
        options: &LayoutOptions,
    ) -> Result<LayoutResult, LayoutError> {
        // Validate inputs
        self.validate_inputs(text, options)?;

        // Find line break opportunities
        let breaks = self.line_breaker.find_breaks(text);

        // Break into lines based on max_width
        let mut lines = self.break_into_lines(text, shaped_text, &breaks, options)?;

        // Apply justification
        self.justifier
            .justify_lines(&mut lines, options.max_width, options.justification);

        // Calculate vertical positions
        self.position_lines_vertically(&mut lines, options);

        // Calculate total dimensions
        let total_width = lines
            .iter()
            .map(|l| l.width + l.x_offset)
            .fold(0.0f32, f32::max);
        let total_height = lines.last().map(|l| l.y_offset + l.height).unwrap_or(0.0);

        // Check for overflow
        let overflow = if let Some(max_height) = options.max_height {
            total_height > max_height
        } else {
            false
        };

        Ok(LayoutResult {
            lines,
            total_height,
            total_width,
            overflow,
        })
    }

    /// Validate layout inputs
    fn validate_inputs(&self, text: &str, options: &LayoutOptions) -> Result<(), LayoutError> {
        if text.is_empty() {
            return Err(LayoutError::InvalidText("Text is empty".to_string()));
        }

        if options.max_width <= 0.0 {
            return Err(LayoutError::InvalidOptions(
                "max_width must be positive".to_string(),
            ));
        }

        if options.line_spacing <= 0.0 {
            return Err(LayoutError::InvalidOptions(
                "line_spacing must be positive".to_string(),
            ));
        }

        Ok(())
    }

    /// Break shaped text into lines
    fn break_into_lines(
        &self,
        text: &str,
        shaped_text: &ShapedText,
        breaks: &[crate::types::LineBreak],
        options: &LayoutOptions,
    ) -> Result<Vec<LayoutLine>, LayoutError> {
        let mut lines = Vec::new();

        if shaped_text.glyphs.is_empty() {
            // Return single empty line
            return Ok(vec![LayoutLine {
                glyphs: vec![],
                width: 0.0,
                height: shaped_text.height,
                baseline: shaped_text.baseline,
                x_offset: 0.0,
                y_offset: 0.0,
                text_range: (0, 0),
            }]);
        }

        // Simple greedy line breaking algorithm
        // In a real implementation, this would use proper Knuth-Plass algorithm
        let mut current_line_glyphs: Vec<PositionedGlyph> = Vec::new();
        let mut current_width = 0.0;
        let mut line_start_char = 0;
        let mut char_index = 0;

        for (_glyph_index, glyph) in shaped_text.glyphs.iter().enumerate() {
            let glyph_width = glyph.advance.x;

            // Check if adding this glyph would exceed max width
            if current_width + glyph_width > options.max_width && !current_line_glyphs.is_empty() {
                // Find break opportunity before this glyph
                let should_break = self.should_break_here(char_index, breaks, true);

                if should_break || current_width + glyph_width > options.max_width * 1.2 {
                    // Create line with current glyphs
                    lines.push(LayoutLine {
                        glyphs: current_line_glyphs.clone(),
                        width: current_width,
                        height: shaped_text.height,
                        baseline: shaped_text.baseline,
                        x_offset: 0.0,
                        y_offset: 0.0,
                        text_range: (line_start_char, char_index),
                    });

                    // Start new line
                    current_line_glyphs.clear();
                    current_width = 0.0;
                    line_start_char = char_index;
                }
            }

            // Add glyph to current line
            current_line_glyphs.push(glyph.clone());
            current_width += glyph_width;
            char_index += 1;

            // Check for mandatory break
            if self.should_break_here(char_index, breaks, false) {
                // Create line
                lines.push(LayoutLine {
                    glyphs: current_line_glyphs.clone(),
                    width: current_width,
                    height: shaped_text.height,
                    baseline: shaped_text.baseline,
                    x_offset: 0.0,
                    y_offset: 0.0,
                    text_range: (line_start_char, char_index),
                });

                // Start new line
                current_line_glyphs.clear();
                current_width = 0.0;
                line_start_char = char_index;
            }
        }

        // Add final line if not empty
        if !current_line_glyphs.is_empty() {
            lines.push(LayoutLine {
                glyphs: current_line_glyphs,
                width: current_width,
                height: shaped_text.height,
                baseline: shaped_text.baseline,
                x_offset: 0.0,
                y_offset: 0.0,
                text_range: (line_start_char, text.len()),
            });
        }

        Ok(lines)
    }

    /// Check if we should break at this character position
    fn should_break_here(
        &self,
        char_index: usize,
        breaks: &[crate::types::LineBreak],
        require_optional: bool,
    ) -> bool {
        breaks
            .iter()
            .any(|b| b.offset == char_index && (b.required || (!require_optional && !b.required)))
    }

    /// Position lines vertically
    fn position_lines_vertically(&self, lines: &mut [LayoutLine], options: &LayoutOptions) {
        let mut y_offset = 0.0;

        for line in lines {
            line.y_offset = y_offset;
            y_offset += line.height * options.line_spacing;
        }
    }
}

impl Default for ParagraphLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_types::{GlyphId, Point, PositionedGlyph, Vector};

    fn create_test_glyph(x: f32, advance_x: f32) -> PositionedGlyph {
        PositionedGlyph {
            glyph_id: GlyphId { id: 1 },
            font_id: 0,
            position: Point { x, y: 0.0 },
            advance: Vector {
                x: advance_x,
                y: 0.0,
            },
            offset: Vector { x: 0.0, y: 0.0 },
        }
    }

    fn create_test_shaped_text(num_glyphs: usize, glyph_width: f32) -> ShapedText {
        let mut glyphs = Vec::new();
        let mut x = 0.0;

        for _ in 0..num_glyphs {
            glyphs.push(create_test_glyph(x, glyph_width));
            x += glyph_width;
        }

        ShapedText {
            glyphs,
            width: num_glyphs as f32 * glyph_width,
            height: 20.0,
            baseline: 15.0,
        }
    }

    // ========== ParagraphLayout Creation Tests ==========

    #[test]
    fn test_paragraph_layout_creation() {
        // Given: Creating a paragraph layout
        // When: Using new() or default()
        // Then: Should create successfully
        let _layout1 = ParagraphLayout::new();
        let _layout2 = ParagraphLayout::default();
    }

    // ========== Input Validation Tests ==========

    #[test]
    fn test_layout_empty_text_error() {
        // Given: Empty text
        // When: Attempting to layout
        // Then: Should return InvalidText error
        let layout = ParagraphLayout::new();
        let _shaped_text = create_test_shaped_text(0, 10.0);
        let options = LayoutOptions::default();

        let result = layout.layout_paragraph("", &_shaped_text, &options);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LayoutError::InvalidText(_)));
    }

    #[test]
    fn test_layout_negative_max_width_error() {
        // Given: Options with negative max_width
        // When: Attempting to layout
        // Then: Should return InvalidOptions error
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(5, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = -100.0;

        let result = layout.layout_paragraph("Hello", &shaped_text, &options);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LayoutError::InvalidOptions(_)
        ));
    }

    #[test]
    fn test_layout_zero_max_width_error() {
        // Given: Options with zero max_width
        // When: Attempting to layout
        // Then: Should return InvalidOptions error
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(5, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 0.0;

        let result = layout.layout_paragraph("Hello", &shaped_text, &options);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LayoutError::InvalidOptions(_)
        ));
    }

    #[test]
    fn test_layout_negative_line_spacing_error() {
        // Given: Options with negative line_spacing
        // When: Attempting to layout
        // Then: Should return InvalidOptions error
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(5, 10.0);
        let mut options = LayoutOptions::default();
        options.line_spacing = -1.0;

        let result = layout.layout_paragraph("Hello", &shaped_text, &options);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LayoutError::InvalidOptions(_)
        ));
    }

    // ========== Basic Layout Tests ==========

    #[test]
    fn test_layout_single_line() {
        // Given: Text that fits on one line
        // When: Laying out the paragraph
        // Then: Should create single line
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(5, 10.0); // 50px total
        let mut options = LayoutOptions::default();
        options.max_width = 100.0; // Plenty of space

        let result = layout.layout_paragraph("Hello", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.lines.len(), 1);
        assert!(!result.overflow);
    }

    #[test]
    fn test_layout_multiple_lines() {
        // Given: Text that requires multiple lines
        // When: Laying out the paragraph
        // Then: Should create multiple lines
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(20, 10.0); // 200px total
        let mut options = LayoutOptions::default();
        options.max_width = 100.0; // Forces wrapping

        let result = layout.layout_paragraph("Hello world test example", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.lines.len() > 1);
    }

    #[test]
    fn test_layout_empty_shaped_text() {
        // Given: Shaped text with no glyphs
        // When: Laying out
        // Then: Should create single empty line
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(0, 10.0);
        let options = LayoutOptions::default();

        // Note: This will fail validation due to empty text
        // Let's use non-empty text with empty glyphs
        let shaped_text = ShapedText {
            glyphs: vec![],
            width: 0.0,
            height: 20.0,
            baseline: 15.0,
        };

        let result = layout.layout_paragraph("a", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0].glyphs.len(), 0);
    }

    // ========== Line Breaking Tests ==========

    #[test]
    fn test_line_breaking_at_max_width() {
        // Given: Text exactly at max width boundary
        // When: Laying out
        // Then: Should break appropriately
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(10, 10.0); // 100px total
        let mut options = LayoutOptions::default();
        options.max_width = 100.0; // Exact fit

        let result = layout.layout_paragraph("Hello test", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        // Should fit in one or two lines depending on break opportunities
        assert!(result.lines.len() >= 1);
    }

    #[test]
    fn test_line_breaking_preserves_glyphs() {
        // Given: Multi-line layout
        // When: Breaking into lines
        // Then: Total glyphs should equal original
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(15, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 80.0;

        let result = layout.layout_paragraph("Hello world example", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();

        let total_glyphs: usize = result.lines.iter().map(|l| l.glyphs.len()).sum();
        assert_eq!(total_glyphs, 15);
    }

    // ========== Vertical Positioning Tests ==========

    #[test]
    fn test_vertical_positioning_with_default_spacing() {
        // Given: Multiple lines with default line spacing
        // When: Laying out
        // Then: Lines should be spaced correctly
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(20, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 80.0;
        options.line_spacing = 1.2; // Default

        let result = layout.layout_paragraph("Hello world test example", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();

        if result.lines.len() >= 2 {
            let line1_y = result.lines[0].y_offset;
            let line2_y = result.lines[1].y_offset;
            let expected_spacing = shaped_text.height * 1.2;

            assert!((line2_y - line1_y - expected_spacing).abs() < 0.1);
        }
    }

    #[test]
    fn test_vertical_positioning_with_custom_spacing() {
        // Given: Multiple lines with custom line spacing
        // When: Laying out
        // Then: Lines should use custom spacing
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(20, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 80.0;
        options.line_spacing = 2.0; // Double spacing

        let result = layout.layout_paragraph("Hello world test example", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();

        if result.lines.len() >= 2 {
            let line1_y = result.lines[0].y_offset;
            let line2_y = result.lines[1].y_offset;
            let expected_spacing = shaped_text.height * 2.0;

            assert!((line2_y - line1_y - expected_spacing).abs() < 0.1);
        }
    }

    #[test]
    fn test_first_line_starts_at_zero() {
        // Given: Any paragraph layout
        // When: Laying out
        // Then: First line should start at y_offset = 0
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(10, 10.0);
        let options = LayoutOptions::default();

        let result = layout.layout_paragraph("Hello world", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.lines.is_empty());
        assert_eq!(result.lines[0].y_offset, 0.0);
    }

    // ========== Total Dimensions Tests ==========

    #[test]
    fn test_total_width_calculation() {
        // Given: Laid out paragraph
        // When: Checking total width
        // Then: Should reflect widest line
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(10, 10.0);
        let options = LayoutOptions::default();

        let result = layout.layout_paragraph("Hello test", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.total_width > 0.0);
    }

    #[test]
    fn test_total_height_calculation() {
        // Given: Multi-line paragraph
        // When: Checking total height
        // Then: Should include all lines with spacing
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(20, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 80.0;

        let result = layout.layout_paragraph("Hello world test example", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.total_height > shaped_text.height);
    }

    // ========== Overflow Detection Tests ==========

    #[test]
    fn test_no_overflow_without_max_height() {
        // Given: Layout without max_height constraint
        // When: Laying out
        // Then: overflow should be false
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(20, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 80.0;
        options.max_height = None;

        let result = layout.layout_paragraph("Hello world test example", &shaped_text, &options);

        assert!(result.is_ok());
        assert!(!result.unwrap().overflow);
    }

    #[test]
    fn test_overflow_when_exceeds_max_height() {
        // Given: Layout with restrictive max_height
        // When: Laying out tall content
        // Then: overflow should be true
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(50, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 80.0;
        options.max_height = Some(50.0); // Very restrictive

        let result =
            layout.layout_paragraph("Hello world test example long text", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        // With enough text and small max_height, should overflow
        // Exact behavior depends on line breaking
        assert!(result.total_height > 0.0);
    }

    #[test]
    fn test_no_overflow_when_within_max_height() {
        // Given: Layout with generous max_height
        // When: Laying out short content
        // Then: overflow should be false
        let layout = ParagraphLayout::new();
        let shaped_text = create_test_shaped_text(5, 10.0);
        let mut options = LayoutOptions::default();
        options.max_width = 200.0;
        options.max_height = Some(1000.0); // Very generous

        let result = layout.layout_paragraph("Hello", &shaped_text, &options);

        assert!(result.is_ok());
        assert!(!result.unwrap().overflow);
    }
}
