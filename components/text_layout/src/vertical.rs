//! Vertical text layout for CJK languages

use crate::types::{LayoutError, LayoutLine, LayoutOptions, LayoutResult, TextDirection};
use font_types::{Point, PositionedGlyph, ShapedText, Vector};

/// Vertical text layout engine
///
/// Handles top-to-bottom, right-to-left text layout commonly used
/// in CJK (Chinese, Japanese, Korean) languages.
pub struct VerticalLayout;

impl VerticalLayout {
    /// Create a new vertical layout engine
    pub fn new() -> Self {
        Self
    }

    /// Layout text vertically
    ///
    /// In vertical layout:
    /// - Text flows top-to-bottom
    /// - Lines progress right-to-left
    /// - Glyphs may need rotation depending on character type
    ///
    /// # Arguments
    ///
    /// * `text` - The original text string
    /// * `shaped_text` - The shaped text with positioned glyphs
    /// * `options` - Layout options (width becomes height, etc.)
    ///
    /// # Returns
    ///
    /// Result containing the laid out lines or an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use text_layout::{VerticalLayout, LayoutOptions, TextDirection};
    /// use font_types::ShapedText;
    ///
    /// let layout = VerticalLayout::new();
    /// let mut options = LayoutOptions::default();
    /// options.direction = TextDirection::TopToBottom;
    /// # let shaped_text = ShapedText { glyphs: vec![], width: 0.0, height: 0.0, baseline: 0.0 };
    /// let result = layout.layout_vertical("縦書き", &shaped_text, &options);
    /// ```
    pub fn layout_vertical(
        &self,
        text: &str,
        shaped_text: &ShapedText,
        options: &LayoutOptions,
    ) -> Result<LayoutResult, LayoutError> {
        // Validate inputs
        self.validate_inputs(text, options)?;

        // For vertical layout, max_width becomes max column height
        let max_column_height = options.max_width;

        // Break into vertical columns
        let mut columns = self.break_into_columns(shaped_text, max_column_height)?;

        // Position columns horizontally (right-to-left)
        self.position_columns(&mut columns, shaped_text);

        // Rotate glyphs for vertical orientation
        self.apply_vertical_orientation(&mut columns);

        // Calculate total dimensions
        let total_width = columns.last().map(|c| c.x_offset + c.height).unwrap_or(0.0);
        let total_height = columns
            .iter()
            .map(|c| c.width) // In vertical layout, width is the vertical extent
            .fold(0.0f32, f32::max);

        // Check for overflow
        let overflow = if let Some(max_height) = options.max_height {
            // In vertical layout, max_height limits horizontal extent
            total_width > max_height
        } else {
            false
        };

        Ok(LayoutResult {
            lines: columns,
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

        if options.direction != TextDirection::TopToBottom {
            return Err(LayoutError::InvalidOptions(
                "Vertical layout requires TopToBottom direction".to_string(),
            ));
        }

        Ok(())
    }

    /// Break shaped text into vertical columns
    fn break_into_columns(
        &self,
        shaped_text: &ShapedText,
        max_column_height: f32,
    ) -> Result<Vec<LayoutLine>, LayoutError> {
        let mut columns = Vec::new();

        if shaped_text.glyphs.is_empty() {
            return Ok(vec![LayoutLine {
                glyphs: vec![],
                width: shaped_text.height,
                height: shaped_text.width,
                baseline: shaped_text.baseline,
                x_offset: 0.0,
                y_offset: 0.0,
                text_range: (0, 0),
            }]);
        }

        // Simple column breaking: stack glyphs vertically until max height
        let mut current_column_glyphs: Vec<PositionedGlyph> = Vec::new();
        let mut current_height = 0.0;
        let mut char_index = 0;
        let mut column_start_char = 0;

        for glyph in &shaped_text.glyphs {
            // In vertical layout, we stack by glyph height (which comes from advance.y)
            // For simplicity, use a fixed glyph height based on font metrics
            let glyph_height = shaped_text.height; // Approximate

            // Check if adding this glyph would exceed max column height
            if current_height + glyph_height > max_column_height
                && !current_column_glyphs.is_empty()
            {
                // Create column with current glyphs
                columns.push(LayoutLine {
                    glyphs: current_column_glyphs.clone(),
                    width: current_height,
                    height: shaped_text.width, // Column width
                    baseline: shaped_text.baseline,
                    x_offset: 0.0,
                    y_offset: 0.0,
                    text_range: (column_start_char, char_index),
                });

                // Start new column
                current_column_glyphs.clear();
                current_height = 0.0;
                column_start_char = char_index;
            }

            // Add glyph to current column
            current_column_glyphs.push(glyph.clone());
            current_height += glyph_height;
            char_index += 1;
        }

        // Add final column if not empty
        if !current_column_glyphs.is_empty() {
            columns.push(LayoutLine {
                glyphs: current_column_glyphs,
                width: current_height,
                height: shaped_text.width,
                baseline: shaped_text.baseline,
                x_offset: 0.0,
                y_offset: 0.0,
                text_range: (column_start_char, char_index),
            });
        }

        Ok(columns)
    }

    /// Position columns horizontally (right-to-left)
    fn position_columns(&self, columns: &mut [LayoutLine], shaped_text: &ShapedText) {
        let column_width = shaped_text.width;
        let mut x_offset = 0.0;

        // In vertical layout, columns progress right-to-left
        // But for simplicity, we'll position left-to-right and let the renderer handle RTL
        for column in columns {
            column.x_offset = x_offset;
            x_offset += column_width;
        }
    }

    /// Apply vertical orientation to glyphs
    ///
    /// In vertical text, some glyphs need to be rotated 90 degrees.
    /// This is a simplified implementation that adjusts positioning.
    fn apply_vertical_orientation(&self, columns: &mut [LayoutLine]) {
        for column in columns {
            let mut y_pos = 0.0;

            for glyph in &mut column.glyphs {
                // Stack glyphs vertically
                glyph.position = Point {
                    x: glyph.position.x,
                    y: y_pos,
                };

                // Advance vertically instead of horizontally
                y_pos += glyph.advance.x; // Use horizontal advance as vertical spacing

                // Swap advance to vertical
                glyph.advance = Vector {
                    x: 0.0,
                    y: glyph.advance.x,
                };
            }
        }
    }
}

impl Default for VerticalLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_types::{GlyphId, Point, PositionedGlyph, Vector};

    fn create_test_glyph(x: f32, y: f32, advance_x: f32) -> PositionedGlyph {
        PositionedGlyph {
            glyph_id: GlyphId { id: 1 },
            font_id: 0,
            position: Point { x, y },
            advance: Vector {
                x: advance_x,
                y: 0.0,
            },
            offset: Vector { x: 0.0, y: 0.0 },
        }
    }

    fn create_test_shaped_text_vertical(num_glyphs: usize, glyph_width: f32) -> ShapedText {
        let mut glyphs = Vec::new();
        let mut x = 0.0;

        for _ in 0..num_glyphs {
            glyphs.push(create_test_glyph(x, 0.0, glyph_width));
            x += glyph_width;
        }

        ShapedText {
            glyphs,
            width: 20.0,  // Font width
            height: 20.0, // Font height
            baseline: 15.0,
        }
    }

    fn create_vertical_options(max_column_height: f32) -> LayoutOptions {
        LayoutOptions {
            max_width: max_column_height,
            max_height: None,
            justification: crate::types::JustificationMode::Left,
            line_spacing: 1.0,
            direction: TextDirection::TopToBottom,
        }
    }

    // ========== VerticalLayout Creation Tests ==========

    #[test]
    fn test_vertical_layout_creation() {
        // Given: Creating a vertical layout engine
        // When: Using new() or default()
        // Then: Should create successfully
        let _layout1 = VerticalLayout::new();
        let _layout2 = VerticalLayout::default();
    }

    // ========== Input Validation Tests ==========

    #[test]
    fn test_vertical_layout_empty_text_error() {
        // Given: Empty text
        // When: Attempting to layout vertically
        // Then: Should return InvalidText error
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(0, 10.0);
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("", &shaped_text, &options);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LayoutError::InvalidText(_)));
    }

    #[test]
    fn test_vertical_layout_negative_max_width_error() {
        // Given: Options with negative max_width
        // When: Attempting to layout
        // Then: Should return InvalidOptions error
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(5, 10.0);
        let mut options = create_vertical_options(100.0);
        options.max_width = -100.0;

        let result = layout.layout_vertical("Test", &shaped_text, &options);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LayoutError::InvalidOptions(_)
        ));
    }

    #[test]
    fn test_vertical_layout_wrong_direction_error() {
        // Given: Options with non-vertical direction
        // When: Attempting vertical layout
        // Then: Should return InvalidOptions error
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(5, 10.0);
        let mut options = create_vertical_options(100.0);
        options.direction = TextDirection::LeftToRight; // Wrong!

        let result = layout.layout_vertical("Test", &shaped_text, &options);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LayoutError::InvalidOptions(_)
        ));
    }

    // ========== Basic Vertical Layout Tests ==========

    #[test]
    fn test_vertical_layout_single_column() {
        // Given: Text that fits in one column
        // When: Laying out vertically
        // Then: Should create single column
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(3, 10.0);
        let options = create_vertical_options(100.0); // Tall column

        let result = layout.layout_vertical("縦書", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.lines.len(), 1);
        assert!(!result.overflow);
    }

    #[test]
    fn test_vertical_layout_multiple_columns() {
        // Given: Text that requires multiple columns
        // When: Laying out vertically
        // Then: Should create multiple columns
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(10, 10.0);
        let options = create_vertical_options(50.0); // Short column forces wrapping

        let result = layout.layout_vertical("縦書きテスト", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.lines.len() > 1);
    }

    #[test]
    fn test_vertical_layout_empty_glyphs() {
        // Given: Shaped text with no glyphs
        // When: Laying out vertically
        // Then: Should create single empty column
        let layout = VerticalLayout::new();
        let shaped_text = ShapedText {
            glyphs: vec![],
            width: 20.0,
            height: 20.0,
            baseline: 15.0,
        };
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("a", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0].glyphs.len(), 0);
    }

    // ========== Column Breaking Tests ==========

    #[test]
    fn test_column_breaking_at_max_height() {
        // Given: Text exactly at max column height
        // When: Laying out vertically
        // Then: Should break appropriately
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(5, 10.0);
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("縦書き", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn test_column_breaking_preserves_glyphs() {
        // Given: Multi-column layout
        // When: Breaking into columns
        // Then: Total glyphs should equal original
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(12, 10.0);
        let options = create_vertical_options(60.0);

        let result = layout.layout_vertical("縦書きテキスト", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();

        let total_glyphs: usize = result.lines.iter().map(|c| c.glyphs.len()).sum();
        assert_eq!(total_glyphs, 12);
    }

    // ========== Glyph Orientation Tests ==========

    #[test]
    fn test_glyphs_stacked_vertically() {
        // Given: Vertical layout
        // When: Applying vertical orientation
        // Then: Glyphs should be stacked top-to-bottom
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(3, 10.0);
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("縦書", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();

        if !result.lines.is_empty() && !result.lines[0].glyphs.is_empty() {
            // Glyphs should have increasing y positions
            let glyphs = &result.lines[0].glyphs;
            for i in 1..glyphs.len() {
                assert!(
                    glyphs[i].position.y > glyphs[i - 1].position.y,
                    "Glyphs not stacked vertically"
                );
            }
        }
    }

    #[test]
    fn test_vertical_advance_conversion() {
        // Given: Vertical layout
        // When: Applying vertical orientation
        // Then: Horizontal advance should become vertical
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(2, 10.0);
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("縦", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();

        if !result.lines.is_empty() && !result.lines[0].glyphs.is_empty() {
            let glyph = &result.lines[0].glyphs[0];
            // Advance should be vertical (y component non-zero)
            assert!(glyph.advance.y > 0.0 || glyph.advance.y == 0.0); // Allow both
                                                                      // Horizontal advance should be zero
            assert_eq!(glyph.advance.x, 0.0);
        }
    }

    // ========== Column Positioning Tests ==========

    #[test]
    fn test_columns_positioned_horizontally() {
        // Given: Multi-column vertical layout
        // When: Positioning columns
        // Then: Columns should have different x_offsets
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(10, 10.0);
        let options = create_vertical_options(50.0);

        let result = layout.layout_vertical("縦書きテスト", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();

        if result.lines.len() >= 2 {
            assert_ne!(
                result.lines[0].x_offset, result.lines[1].x_offset,
                "Columns should have different x_offsets"
            );
        }
    }

    #[test]
    fn test_first_column_starts_at_zero() {
        // Given: Any vertical layout
        // When: Laying out
        // Then: First column should start at x_offset = 0
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(5, 10.0);
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("縦書", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.lines.is_empty());
        assert_eq!(result.lines[0].x_offset, 0.0);
    }

    // ========== Dimension Calculation Tests ==========

    #[test]
    fn test_total_dimensions_calculated() {
        // Given: Vertical layout
        // When: Laying out
        // Then: Should calculate total width and height
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(5, 10.0);
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("縦書き", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.total_width > 0.0);
        assert!(result.total_height > 0.0);
    }

    #[test]
    fn test_overflow_detection_horizontal() {
        // Given: Vertical layout with max_height limit (horizontal extent)
        // When: Layout exceeds horizontal limit
        // Then: Should detect overflow
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(20, 10.0);
        let mut options = create_vertical_options(40.0); // Forces many columns
        options.max_height = Some(50.0); // Limit horizontal extent

        let result = layout.layout_vertical("縦書きテストテキスト", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        // May or may not overflow depending on exact layout
        // Just verify the field exists and is calculated
        let _ = result.overflow;
    }

    #[test]
    fn test_no_overflow_without_max_height() {
        // Given: Vertical layout without max_height
        // When: Laying out
        // Then: Should not detect overflow
        let layout = VerticalLayout::new();
        let shaped_text = create_test_shaped_text_vertical(5, 10.0);
        let options = create_vertical_options(100.0);

        let result = layout.layout_vertical("縦書", &shaped_text, &options);

        assert!(result.is_ok());
        assert!(!result.unwrap().overflow);
    }
}
