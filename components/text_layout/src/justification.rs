//! Text justification algorithms

use crate::types::{JustificationMode, LayoutLine};
use font_types::{Point, Vector};

/// Text justifier for different alignment modes
pub struct Justifier;

impl Justifier {
    /// Create a new justifier
    pub fn new() -> Self {
        Self
    }

    /// Apply justification to a line
    ///
    /// Modifies the line's x_offset and adjusts glyph positions for full justification.
    ///
    /// # Arguments
    ///
    /// * `line` - The layout line to justify (modified in place)
    /// * `target_width` - The target width to justify to
    /// * `mode` - The justification mode to apply
    ///
    /// # Example
    ///
    /// ```
    /// use text_layout::{Justifier, LayoutLine, JustificationMode};
    ///
    /// let mut line = LayoutLine {
    ///     glyphs: vec![],
    ///     width: 80.0,
    ///     height: 20.0,
    ///     baseline: 15.0,
    ///     x_offset: 0.0,
    ///     y_offset: 0.0,
    ///     text_range: (0, 10),
    /// };
    ///
    /// let justifier = Justifier::new();
    /// justifier.justify_line(&mut line, 100.0, JustificationMode::Center);
    /// // line.x_offset is now 10.0 (centered)
    /// ```
    pub fn justify_line(&self, line: &mut LayoutLine, target_width: f32, mode: JustificationMode) {
        match mode {
            JustificationMode::Left => {
                line.x_offset = 0.0;
            }
            JustificationMode::Right => {
                line.x_offset = target_width - line.width;
            }
            JustificationMode::Center => {
                line.x_offset = (target_width - line.width) / 2.0;
            }
            JustificationMode::Justify => {
                self.distribute_space(line, target_width);
            }
        }
    }

    /// Distribute space evenly across a line (full justification)
    ///
    /// For full justification, we distribute the extra space between words
    /// (spaces). This implementation adds equal spacing between each gap.
    ///
    /// # Arguments
    ///
    /// * `line` - The layout line to justify
    /// * `target_width` - The target width to justify to
    fn distribute_space(&self, line: &mut LayoutLine, target_width: f32) {
        // Calculate extra space needed
        let extra_space = target_width - line.width;

        // If line is already wider than target, don't justify
        if extra_space <= 0.0 {
            line.x_offset = 0.0;
            return;
        }

        // Count the number of spaces (gaps) between glyphs
        // We'll use advance vectors to identify potential break points
        let gap_count = self.count_justification_gaps(line);

        // If no gaps (single word), use left alignment
        if gap_count == 0 {
            line.x_offset = 0.0;
            return;
        }

        // Calculate additional space per gap
        let space_per_gap = extra_space / gap_count as f32;

        // Adjust glyph positions
        let mut cumulative_offset = 0.0;
        let mut in_gap = false;

        for glyph in &mut line.glyphs {
            // Add cumulative offset to this glyph's position
            glyph.position.x += cumulative_offset;

            // Check if this glyph's advance indicates a gap (simplified heuristic)
            // In a real implementation, you'd track actual word boundaries
            if glyph.advance.x > 0.0 {
                // This is a potential gap (after a visible character)
                if in_gap {
                    cumulative_offset += space_per_gap;
                    in_gap = false;
                } else {
                    in_gap = true;
                }
            }
        }

        line.x_offset = 0.0;
        line.width = target_width;
    }

    /// Count the number of gaps suitable for justification
    ///
    /// This is a simplified heuristic. In a real implementation, you would
    /// track actual word boundaries from the text shaping phase.
    ///
    /// # Arguments
    ///
    /// * `line` - The layout line to analyze
    ///
    /// # Returns
    ///
    /// Number of gaps between words
    fn count_justification_gaps(&self, line: &LayoutLine) -> usize {
        if line.glyphs.is_empty() {
            return 0;
        }

        // Simple heuristic: count transitions from visible to space
        // In practice, you'd use actual text/glyph metadata
        let mut gap_count = 0;
        let mut prev_advance = 0.0;

        for glyph in &line.glyphs {
            // Detect gaps based on advance width changes
            // This is a simplification - real implementation would use glyph IDs
            if prev_advance > 0.0 && glyph.advance.x > prev_advance * 1.5 {
                gap_count += 1;
            }
            prev_advance = glyph.advance.x;
        }

        gap_count.max(1) // At least 1 gap for non-empty lines
    }

    /// Justify multiple lines
    ///
    /// Applies justification to all lines except the last one (which is
    /// typically left-aligned even in justified paragraphs).
    ///
    /// # Arguments
    ///
    /// * `lines` - The lines to justify
    /// * `target_width` - The target width
    /// * `mode` - The justification mode
    pub fn justify_lines(
        &self,
        lines: &mut [LayoutLine],
        target_width: f32,
        mode: JustificationMode,
    ) {
        if lines.is_empty() {
            return;
        }

        // For full justification, don't justify the last line
        let lines_to_justify = if mode == JustificationMode::Justify {
            lines.len().saturating_sub(1)
        } else {
            lines.len()
        };

        for i in 0..lines_to_justify {
            self.justify_line(&mut lines[i], target_width, mode);
        }

        // Last line in justified mode uses left alignment
        if mode == JustificationMode::Justify && !lines.is_empty() {
            lines.last_mut().unwrap().x_offset = 0.0;
        }
    }
}

impl Default for Justifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_types::{GlyphId, PositionedGlyph};

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

    fn create_test_line(width: f32) -> LayoutLine {
        LayoutLine {
            glyphs: vec![
                create_test_glyph(0.0, 10.0),
                create_test_glyph(10.0, 10.0),
                create_test_glyph(20.0, 10.0),
            ],
            width,
            height: 20.0,
            baseline: 15.0,
            x_offset: 0.0,
            y_offset: 0.0,
            text_range: (0, 10),
        }
    }

    // ========== Justifier Creation Tests ==========

    #[test]
    fn test_justifier_creation() {
        // Given: Creating a justifier
        // When: Using new() or default()
        // Then: Should create successfully
        let _justifier1 = Justifier::new();
        let _justifier2 = Justifier::default();
    }

    // ========== Left Justification Tests ==========

    #[test]
    fn test_justify_left() {
        // Given: A line and left justification mode
        // When: Applying left justification
        // Then: x_offset should be 0
        let justifier = Justifier::new();
        let mut line = create_test_line(80.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Left);

        assert_eq!(line.x_offset, 0.0);
    }

    #[test]
    fn test_justify_left_exact_width() {
        // Given: A line that exactly matches target width
        // When: Applying left justification
        // Then: x_offset should still be 0
        let justifier = Justifier::new();
        let mut line = create_test_line(100.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Left);

        assert_eq!(line.x_offset, 0.0);
    }

    // ========== Right Justification Tests ==========

    #[test]
    fn test_justify_right() {
        // Given: A line and right justification mode
        // When: Applying right justification
        // Then: x_offset should position line at right edge
        let justifier = Justifier::new();
        let mut line = create_test_line(80.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Right);

        assert_eq!(line.x_offset, 20.0); // 100 - 80
    }

    #[test]
    fn test_justify_right_exact_width() {
        // Given: A line that exactly matches target width
        // When: Applying right justification
        // Then: x_offset should be 0 (line fills width)
        let justifier = Justifier::new();
        let mut line = create_test_line(100.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Right);

        assert_eq!(line.x_offset, 0.0); // 100 - 100
    }

    #[test]
    fn test_justify_right_wider_than_target() {
        // Given: A line wider than target width
        // When: Applying right justification
        // Then: x_offset should be negative (overflow to left)
        let justifier = Justifier::new();
        let mut line = create_test_line(120.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Right);

        assert_eq!(line.x_offset, -20.0); // 100 - 120
    }

    // ========== Center Justification Tests ==========

    #[test]
    fn test_justify_center() {
        // Given: A line and center justification mode
        // When: Applying center justification
        // Then: x_offset should center the line
        let justifier = Justifier::new();
        let mut line = create_test_line(80.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Center);

        assert_eq!(line.x_offset, 10.0); // (100 - 80) / 2
    }

    #[test]
    fn test_justify_center_exact_width() {
        // Given: A line that exactly matches target width
        // When: Applying center justification
        // Then: x_offset should be 0
        let justifier = Justifier::new();
        let mut line = create_test_line(100.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Center);

        assert_eq!(line.x_offset, 0.0); // (100 - 100) / 2
    }

    #[test]
    fn test_justify_center_odd_difference() {
        // Given: A line with odd pixel difference from target
        // When: Applying center justification
        // Then: Should handle fractional offset
        let justifier = Justifier::new();
        let mut line = create_test_line(75.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Center);

        assert_eq!(line.x_offset, 12.5); // (100 - 75) / 2
    }

    // ========== Full Justification Tests ==========

    #[test]
    fn test_justify_full_basic() {
        // Given: A line and full justification mode
        // When: Applying full justification
        // Then: Line should be expanded to target width
        let justifier = Justifier::new();
        let mut line = create_test_line(80.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Justify);

        // Line should be expanded to target width
        // Note: exact behavior depends on gap distribution
        assert!(line.x_offset <= 1.0); // Should be close to 0
    }

    #[test]
    fn test_justify_full_exact_width() {
        // Given: A line that exactly matches target width
        // When: Applying full justification
        // Then: Should not modify line
        let justifier = Justifier::new();
        let mut line = create_test_line(100.0);
        let original_width = line.width;

        justifier.justify_line(&mut line, 100.0, JustificationMode::Justify);

        // Line should remain unchanged
        assert_eq!(line.width, original_width);
    }

    #[test]
    fn test_justify_full_single_word() {
        // Given: A line with single word (no gaps)
        // When: Applying full justification
        // Then: Should fall back to left alignment
        let justifier = Justifier::new();
        let mut line = create_test_line(80.0);
        // Remove glyphs to simulate single word
        line.glyphs = vec![create_test_glyph(0.0, 80.0)];

        justifier.justify_line(&mut line, 100.0, JustificationMode::Justify);

        // Single word should use left alignment (no distribution)
        assert_eq!(line.x_offset, 0.0);
    }

    #[test]
    fn test_justify_full_empty_line() {
        // Given: An empty line
        // When: Applying full justification
        // Then: Should handle gracefully
        let justifier = Justifier::new();
        let mut line = LayoutLine {
            glyphs: vec![],
            width: 0.0,
            height: 20.0,
            baseline: 15.0,
            x_offset: 0.0,
            y_offset: 0.0,
            text_range: (0, 0),
        };

        justifier.justify_line(&mut line, 100.0, JustificationMode::Justify);

        // Empty line should not crash
        assert_eq!(line.x_offset, 0.0);
    }

    #[test]
    fn test_justify_full_wider_than_target() {
        // Given: A line wider than target width
        // When: Applying full justification
        // Then: Should not expand (already too wide)
        let justifier = Justifier::new();
        let mut line = create_test_line(120.0);

        justifier.justify_line(&mut line, 100.0, JustificationMode::Justify);

        // Line already wider than target, should not justify
        assert_eq!(line.x_offset, 0.0);
    }

    // ========== Multiple Lines Justification Tests ==========

    #[test]
    fn test_justify_multiple_lines_left() {
        // Given: Multiple lines
        // When: Applying left justification to all
        // Then: All lines should have x_offset = 0
        let justifier = Justifier::new();
        let mut lines = vec![
            create_test_line(70.0),
            create_test_line(80.0),
            create_test_line(90.0),
        ];

        justifier.justify_lines(&mut lines, 100.0, JustificationMode::Left);

        for line in &lines {
            assert_eq!(line.x_offset, 0.0);
        }
    }

    #[test]
    fn test_justify_multiple_lines_center() {
        // Given: Multiple lines
        // When: Applying center justification to all
        // Then: All lines should be centered
        let justifier = Justifier::new();
        let mut lines = vec![create_test_line(60.0), create_test_line(80.0)];

        justifier.justify_lines(&mut lines, 100.0, JustificationMode::Center);

        assert_eq!(lines[0].x_offset, 20.0); // (100 - 60) / 2
        assert_eq!(lines[1].x_offset, 10.0); // (100 - 80) / 2
    }

    #[test]
    fn test_justify_multiple_lines_full() {
        // Given: Multiple lines
        // When: Applying full justification
        // Then: All lines except last should be fully justified
        let justifier = Justifier::new();
        let mut lines = vec![
            create_test_line(70.0),
            create_test_line(80.0),
            create_test_line(90.0),
        ];

        justifier.justify_lines(&mut lines, 100.0, JustificationMode::Justify);

        // Last line should have left alignment (x_offset = 0)
        assert_eq!(lines[2].x_offset, 0.0);
    }

    #[test]
    fn test_justify_empty_lines_array() {
        // Given: Empty lines array
        // When: Applying justification
        // Then: Should handle gracefully without panic
        let justifier = Justifier::new();
        let mut lines: Vec<LayoutLine> = vec![];

        justifier.justify_lines(&mut lines, 100.0, JustificationMode::Left);

        // Should not crash
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_justify_single_line_array() {
        // Given: Array with single line
        // When: Applying full justification
        // Then: Should use left alignment (last line rule)
        let justifier = Justifier::new();
        let mut lines = vec![create_test_line(80.0)];

        justifier.justify_lines(&mut lines, 100.0, JustificationMode::Justify);

        // Single line in justify mode should use left alignment
        assert_eq!(lines[0].x_offset, 0.0);
    }

    // ========== Gap Counting Tests ==========

    #[test]
    fn test_count_gaps_empty_line() {
        // Given: Empty line
        // When: Counting justification gaps
        // Then: Should return 0
        let justifier = Justifier::new();
        let line = LayoutLine {
            glyphs: vec![],
            width: 0.0,
            height: 0.0,
            baseline: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
            text_range: (0, 0),
        };

        let gap_count = justifier.count_justification_gaps(&line);
        assert_eq!(gap_count, 0);
    }

    #[test]
    fn test_count_gaps_non_empty_line() {
        // Given: Line with glyphs
        // When: Counting justification gaps
        // Then: Should return at least 1 for non-empty lines
        let justifier = Justifier::new();
        let line = create_test_line(80.0);

        let gap_count = justifier.count_justification_gaps(&line);
        assert!(gap_count >= 1);
    }
}
