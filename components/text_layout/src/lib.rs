//! Text layout engine for multi-line text
//!
//! This crate provides high-level text layout capabilities including:
//! - Paragraph layout (multi-line text rendering)
//! - Line breaking (Unicode UAX #14 compliant)
//! - Text justification (left, right, center, full)
//! - Vertical text layout (for CJK languages)
//!
//! # Example
//!
//! ```no_run
//! use text_layout::{ParagraphLayout, LayoutOptions, JustificationMode};
//! use font_types::ShapedText;
//!
//! // Create a paragraph layout engine
//! let layout = ParagraphLayout::new();
//!
//! // Configure layout options
//! let options = LayoutOptions {
//!     max_width: 500.0,
//!     max_height: None,
//!     justification: JustificationMode::Justify,
//!     line_spacing: 1.2,
//!     ..Default::default()
//! };
//!
//! // Layout shaped text (from text_shaper)
//! # let text = "Hello world";
//! # let shaped_text = ShapedText { glyphs: vec![], width: 0.0, height: 0.0, baseline: 0.0 };
//! let result = layout.layout_paragraph(text, &shaped_text, &options);
//!
//! // Process layout result
//! match result {
//!     Ok(layout_result) => {
//!         for line in layout_result.lines {
//!             // Render each line at (line.x_offset, line.y_offset)
//!             for glyph in line.glyphs {
//!                 // Render glyph at position
//!             }
//!         }
//!     }
//!     Err(e) => eprintln!("Layout error: {}", e),
//! }
//! ```
//!
//! # Architecture
//!
//! The text layout system consists of several specialized modules:
//!
//! - **types**: Core types and enums for layout configuration and results
//! - **line_breaker**: Unicode UAX #14 compliant line breaking
//! - **justification**: Text alignment and justification algorithms
//! - **paragraph**: Main paragraph layout engine
//! - **vertical**: Vertical text layout for CJK languages
//!
//! # Feature Flags
//!
//! This crate currently has no optional features.
//!
//! # Performance
//!
//! The layout engine is designed for efficiency:
//! - Paragraph layout: < 5ms for 1000 characters
//! - Line breaking: < 1ms for 1000 characters
//! - Memory usage: < 10KB per paragraph
//!
//! # Thread Safety
//!
//! All layout structures are `Send` and `Sync` where applicable.
//! Layout operations are stateless and can be performed concurrently.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)] // LayoutOptions, LayoutResult are clear

// Module declarations
pub mod justification;
pub mod line_breaker;
pub mod paragraph;
pub mod types;
pub mod vertical;

// Re-export main types for convenience
pub use justification::Justifier;
pub use line_breaker::LineBreaker;
pub use paragraph::ParagraphLayout;
pub use types::{
    JustificationMode, LayoutError, LayoutLine, LayoutOptions, LayoutResult, LineBreak,
    TextDirection,
};
pub use vertical::VerticalLayout;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_types_exported() {
        // Given: The public API
        // When: Using exported types
        // Then: All should be accessible without module paths

        // Types
        let _options = LayoutOptions::default();
        let _mode = JustificationMode::Left;
        let _direction = TextDirection::LeftToRight;

        // Main APIs
        let _paragraph_layout = ParagraphLayout::new();
        let _line_breaker = LineBreaker::new();
        let _justifier = Justifier::new();
        let _vertical_layout = VerticalLayout::new();
    }

    #[test]
    fn test_error_type_exported() {
        // Given: Layout errors
        // When: Creating error instances
        // Then: Should be accessible from crate root
        let _err1 = LayoutError::InvalidOptions("test".to_string());
        let _err2 = LayoutError::InvalidText("test".to_string());
        let _err3 = LayoutError::Overflow("test".to_string());
    }

    #[test]
    fn test_result_types_exported() {
        // Given: Layout result types
        // When: Creating instances
        // Then: Should be accessible from crate root
        let _result = LayoutResult {
            lines: vec![],
            total_height: 0.0,
            total_width: 0.0,
            overflow: false,
        };

        let _line = LayoutLine {
            glyphs: vec![],
            width: 0.0,
            height: 0.0,
            baseline: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
            text_range: (0, 0),
        };
    }

    #[test]
    fn test_line_break_exported() {
        // Given: LineBreak type
        // When: Creating instances
        // Then: Should be accessible from crate root
        let _break = LineBreak {
            offset: 10,
            required: true,
        };
    }

    #[test]
    fn test_default_implementations() {
        // Given: Types with Default implementations
        // When: Using default()
        // Then: Should create sensible defaults
        let options = LayoutOptions::default();
        assert_eq!(options.max_width, 500.0);
        assert_eq!(options.justification, JustificationMode::Left);
        assert_eq!(options.line_spacing, 1.2);

        let _layout = ParagraphLayout::default();
        let _breaker = LineBreaker::default();
        let _justifier = Justifier::default();
        let _vertical = VerticalLayout::default();
    }

    #[test]
    fn test_integration_basic_layout() {
        // Given: A paragraph layout engine and shaped text
        // When: Performing basic layout
        // Then: Should integrate all components successfully
        use font_types::{GlyphId, Point, PositionedGlyph, ShapedText, Vector};

        let layout = ParagraphLayout::new();

        // Create simple shaped text
        let glyphs = vec![
            PositionedGlyph {
                glyph_id: GlyphId { id: 1 },
                font_id: 0,
                position: Point { x: 0.0, y: 0.0 },
                advance: Vector { x: 10.0, y: 0.0 },
                offset: Vector { x: 0.0, y: 0.0 },
            },
            PositionedGlyph {
                glyph_id: GlyphId { id: 2 },
                font_id: 0,
                position: Point { x: 10.0, y: 0.0 },
                advance: Vector { x: 10.0, y: 0.0 },
                offset: Vector { x: 0.0, y: 0.0 },
            },
        ];

        let shaped_text = ShapedText {
            glyphs,
            width: 20.0,
            height: 20.0,
            baseline: 15.0,
        };

        let options = LayoutOptions {
            max_width: 100.0,
            max_height: None,
            justification: JustificationMode::Left,
            line_spacing: 1.0,
            direction: TextDirection::LeftToRight,
        };

        let result = layout.layout_paragraph("Hi", &shaped_text, &options);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn test_integration_line_breaking() {
        // Given: Line breaker
        // When: Finding breaks in text
        // Then: Should work correctly
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("Hello world");

        // Should find at least one break
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_integration_justification() {
        // Given: Justifier and a line
        // When: Applying justification
        // Then: Should modify line correctly
        use font_types::{GlyphId, Point, PositionedGlyph, Vector};

        let justifier = Justifier::new();
        let mut line = LayoutLine {
            glyphs: vec![PositionedGlyph {
                glyph_id: GlyphId { id: 1 },
                font_id: 0,
                position: Point { x: 0.0, y: 0.0 },
                advance: Vector { x: 10.0, y: 0.0 },
                offset: Vector { x: 0.0, y: 0.0 },
            }],
            width: 50.0,
            height: 20.0,
            baseline: 15.0,
            x_offset: 0.0,
            y_offset: 0.0,
            text_range: (0, 5),
        };

        justifier.justify_line(&mut line, 100.0, JustificationMode::Center);

        // Should center the line
        assert_eq!(line.x_offset, 25.0); // (100 - 50) / 2
    }

    #[test]
    fn test_integration_vertical_layout() {
        // Given: Vertical layout engine
        // When: Laying out vertical text
        // Then: Should create vertical layout
        use font_types::{GlyphId, Point, PositionedGlyph, ShapedText, Vector};

        let layout = VerticalLayout::new();

        let glyphs = vec![PositionedGlyph {
            glyph_id: GlyphId { id: 1 },
            font_id: 0,
            position: Point { x: 0.0, y: 0.0 },
            advance: Vector { x: 10.0, y: 0.0 },
            offset: Vector { x: 0.0, y: 0.0 },
        }];

        let shaped_text = ShapedText {
            glyphs,
            width: 20.0,
            height: 20.0,
            baseline: 15.0,
        };

        let options = LayoutOptions {
            max_width: 100.0,
            max_height: None,
            justification: JustificationMode::Left,
            line_spacing: 1.0,
            direction: TextDirection::TopToBottom,
        };

        let result = layout.layout_vertical("縦", &shaped_text, &options);

        assert!(result.is_ok());
    }
}
