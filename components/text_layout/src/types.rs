//! Core types for text layout

use font_types::{Direction, PositionedGlyph};
use thiserror::Error;

/// Errors that can occur during layout operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum LayoutError {
    /// Invalid layout options provided
    #[error("Invalid layout options: {0}")]
    InvalidOptions(String),

    /// Text is too large to fit in constraints
    #[error("Text overflow: {0}")]
    Overflow(String),

    /// Empty or invalid text input
    #[error("Invalid text input: {0}")]
    InvalidText(String),
}

/// Text direction for layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    /// Left-to-right horizontal text
    LeftToRight,
    /// Right-to-left horizontal text
    RightToLeft,
    /// Top-to-bottom vertical text (CJK)
    TopToBottom,
}

impl From<Direction> for TextDirection {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::LeftToRight => TextDirection::LeftToRight,
            Direction::RightToLeft => TextDirection::RightToLeft,
            Direction::TopToBottom => TextDirection::TopToBottom,
            Direction::BottomToTop => TextDirection::TopToBottom, // Map to same as TopToBottom
        }
    }
}

/// Text justification mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustificationMode {
    /// Align text to left edge
    Left,
    /// Align text to right edge
    Right,
    /// Center text
    Center,
    /// Distribute space evenly across line (full justification)
    Justify,
}

/// Options for paragraph layout
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutOptions {
    /// Maximum width for text (in pixels)
    pub max_width: f32,
    /// Optional maximum height (None = no limit)
    pub max_height: Option<f32>,
    /// Justification mode
    pub justification: JustificationMode,
    /// Line spacing multiplier (default: 1.2)
    pub line_spacing: f32,
    /// Text direction
    pub direction: TextDirection,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            max_width: 500.0,
            max_height: None,
            justification: JustificationMode::Left,
            line_spacing: 1.2,
            direction: TextDirection::LeftToRight,
        }
    }
}

/// A single line of laid out text
#[derive(Debug, Clone)]
pub struct LayoutLine {
    /// Positioned glyphs in this line
    pub glyphs: Vec<PositionedGlyph>,
    /// Total width of the line (before justification)
    pub width: f32,
    /// Height of the line
    pub height: f32,
    /// Baseline offset from top of line
    pub baseline: f32,
    /// Horizontal offset (for justification)
    pub x_offset: f32,
    /// Vertical offset from top of paragraph
    pub y_offset: f32,
    /// Text range (start and end character indices)
    pub text_range: (usize, usize),
}

/// Result of a layout operation
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// Lines of text
    pub lines: Vec<LayoutLine>,
    /// Total height of laid out text
    pub total_height: f32,
    /// Total width of laid out text
    pub total_width: f32,
    /// Whether text overflowed max_height constraint
    pub overflow: bool,
}

/// A line break opportunity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineBreak {
    /// Byte offset in the text
    pub offset: usize,
    /// Whether this is a mandatory break (hard break)
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== LayoutError Tests ==========

    #[test]
    fn test_layout_error_variants() {
        // Given: Various error conditions
        // When: Creating LayoutError instances
        // Then: Error messages should be descriptive
        let err1 = LayoutError::InvalidOptions("width < 0".to_string());
        assert!(err1.to_string().contains("Invalid layout options"));

        let err2 = LayoutError::Overflow("max height exceeded".to_string());
        assert!(err2.to_string().contains("Text overflow"));

        let err3 = LayoutError::InvalidText("empty".to_string());
        assert!(err3.to_string().contains("Invalid text input"));
    }

    #[test]
    fn test_layout_error_equality() {
        // Given: Two identical errors
        // When: Comparing them
        // Then: They should be equal
        let err1 = LayoutError::InvalidOptions("test".to_string());
        let err2 = LayoutError::InvalidOptions("test".to_string());
        let err3 = LayoutError::InvalidOptions("other".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    // ========== TextDirection Tests ==========

    #[test]
    fn test_text_direction_variants() {
        // Given: Text direction options
        // When: Creating each variant
        // Then: All variants should be available
        let _ltr = TextDirection::LeftToRight;
        let _rtl = TextDirection::RightToLeft;
        let _ttb = TextDirection::TopToBottom;
    }

    #[test]
    fn test_text_direction_from_direction() {
        // Given: font_types::Direction values
        // When: Converting to TextDirection
        // Then: Should map correctly
        assert_eq!(
            TextDirection::from(Direction::LeftToRight),
            TextDirection::LeftToRight
        );
        assert_eq!(
            TextDirection::from(Direction::RightToLeft),
            TextDirection::RightToLeft
        );
        assert_eq!(
            TextDirection::from(Direction::TopToBottom),
            TextDirection::TopToBottom
        );
    }

    // ========== JustificationMode Tests ==========

    #[test]
    fn test_justification_mode_variants() {
        // Given: Justification options
        // When: Creating each variant
        // Then: All variants should be available
        let _left = JustificationMode::Left;
        let _right = JustificationMode::Right;
        let _center = JustificationMode::Center;
        let _justify = JustificationMode::Justify;
    }

    #[test]
    fn test_justification_mode_equality() {
        // Given: Justification modes
        // When: Comparing them
        // Then: Same modes should be equal
        assert_eq!(JustificationMode::Left, JustificationMode::Left);
        assert_ne!(JustificationMode::Left, JustificationMode::Right);
    }

    // ========== LayoutOptions Tests ==========

    #[test]
    fn test_layout_options_default() {
        // Given: Default layout options
        // When: Creating with Default::default()
        // Then: Should have sensible defaults
        let options = LayoutOptions::default();

        assert_eq!(options.max_width, 500.0);
        assert_eq!(options.max_height, None);
        assert_eq!(options.justification, JustificationMode::Left);
        assert_eq!(options.line_spacing, 1.2);
        assert_eq!(options.direction, TextDirection::LeftToRight);
    }

    #[test]
    fn test_layout_options_custom() {
        // Given: Custom layout options
        // When: Creating with specific values
        // Then: All values should be set correctly
        let options = LayoutOptions {
            max_width: 300.0,
            max_height: Some(200.0),
            justification: JustificationMode::Justify,
            line_spacing: 1.5,
            direction: TextDirection::RightToLeft,
        };

        assert_eq!(options.max_width, 300.0);
        assert_eq!(options.max_height, Some(200.0));
        assert_eq!(options.justification, JustificationMode::Justify);
        assert_eq!(options.line_spacing, 1.5);
        assert_eq!(options.direction, TextDirection::RightToLeft);
    }

    #[test]
    fn test_layout_options_clone() {
        // Given: A LayoutOptions instance
        // When: Cloning it
        // Then: Should produce identical copy
        let options = LayoutOptions {
            max_width: 400.0,
            max_height: Some(300.0),
            justification: JustificationMode::Center,
            line_spacing: 1.8,
            direction: TextDirection::TopToBottom,
        };

        let cloned = options.clone();
        assert_eq!(options.max_width, cloned.max_width);
        assert_eq!(options.max_height, cloned.max_height);
        assert_eq!(options.justification, cloned.justification);
        assert_eq!(options.line_spacing, cloned.line_spacing);
        assert_eq!(options.direction, cloned.direction);
    }

    // ========== LayoutLine Tests ==========

    #[test]
    fn test_layout_line_creation() {
        // Given: Line layout information
        // When: Creating a LayoutLine
        // Then: All fields should be set correctly
        let line = LayoutLine {
            glyphs: vec![],
            width: 100.0,
            height: 20.0,
            baseline: 15.0,
            x_offset: 10.0,
            y_offset: 5.0,
            text_range: (0, 10),
        };

        assert_eq!(line.width, 100.0);
        assert_eq!(line.height, 20.0);
        assert_eq!(line.baseline, 15.0);
        assert_eq!(line.x_offset, 10.0);
        assert_eq!(line.y_offset, 5.0);
        assert_eq!(line.text_range, (0, 10));
    }

    #[test]
    fn test_layout_line_clone() {
        // Given: A LayoutLine
        // When: Cloning it
        // Then: Should produce identical copy
        let line = LayoutLine {
            glyphs: vec![],
            width: 150.0,
            height: 25.0,
            baseline: 18.0,
            x_offset: 0.0,
            y_offset: 30.0,
            text_range: (10, 25),
        };

        let cloned = line.clone();
        assert_eq!(line.width, cloned.width);
        assert_eq!(line.height, cloned.height);
        assert_eq!(line.text_range, cloned.text_range);
    }

    // ========== LayoutResult Tests ==========

    #[test]
    fn test_layout_result_creation() {
        // Given: Layout result information
        // When: Creating a LayoutResult
        // Then: All fields should be set correctly
        let result = LayoutResult {
            lines: vec![],
            total_height: 100.0,
            total_width: 200.0,
            overflow: false,
        };

        assert_eq!(result.lines.len(), 0);
        assert_eq!(result.total_height, 100.0);
        assert_eq!(result.total_width, 200.0);
        assert!(!result.overflow);
    }

    #[test]
    fn test_layout_result_with_overflow() {
        // Given: Layout result with overflow
        // When: Creating result with overflow flag
        // Then: Overflow should be true
        let result = LayoutResult {
            lines: vec![],
            total_height: 500.0,
            total_width: 300.0,
            overflow: true,
        };

        assert!(result.overflow);
    }

    #[test]
    fn test_layout_result_clone() {
        // Given: A LayoutResult
        // When: Cloning it
        // Then: Should produce identical copy
        let result = LayoutResult {
            lines: vec![],
            total_height: 250.0,
            total_width: 350.0,
            overflow: false,
        };

        let cloned = result.clone();
        assert_eq!(result.total_height, cloned.total_height);
        assert_eq!(result.total_width, cloned.total_width);
        assert_eq!(result.overflow, cloned.overflow);
    }

    // ========== LineBreak Tests ==========

    #[test]
    fn test_line_break_creation() {
        // Given: Line break information
        // When: Creating a LineBreak
        // Then: Fields should be set correctly
        let break_opt = LineBreak {
            offset: 10,
            required: false,
        };

        assert_eq!(break_opt.offset, 10);
        assert!(!break_opt.required);

        let break_req = LineBreak {
            offset: 20,
            required: true,
        };

        assert_eq!(break_req.offset, 20);
        assert!(break_req.required);
    }

    #[test]
    fn test_line_break_equality() {
        // Given: Line break instances
        // When: Comparing them
        // Then: Same breaks should be equal
        let b1 = LineBreak {
            offset: 5,
            required: true,
        };
        let b2 = LineBreak {
            offset: 5,
            required: true,
        };
        let b3 = LineBreak {
            offset: 5,
            required: false,
        };

        assert_eq!(b1, b2);
        assert_ne!(b1, b3);
    }
}
