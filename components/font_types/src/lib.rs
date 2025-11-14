//! font_types - Common types, traits, enums, and interfaces for the font system
//!
//! This crate provides the fundamental types and enums used throughout the font system,
//! including font descriptors, metrics, glyph identifiers, and rendering structures.
//!
//! # Examples
//!
//! Creating a font descriptor:
//! ```
//! use font_types::{FontDescriptor, FontWeight, FontStyle, FontStretch};
//!
//! let descriptor = FontDescriptor {
//!     family: vec!["Arial".to_string()],
//!     weight: FontWeight::Bold,
//!     style: FontStyle::Normal,
//!     stretch: FontStretch::Normal,
//!     size: 16.0,
//! };
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Common types for the font system
pub mod types;

// Re-export all public types for convenient access
pub use types::{
    Direction, FontDescriptor, FontId, FontMetrics, FontStretch, FontStyle, FontWeight,
    GlyphBitmap, GlyphId, Point, PositionedGlyph, RenderMode, ShapedText, Vector,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_exports_available() {
        // Given: All types should be re-exported at crate root
        // When: Using types without module prefix
        // Then: They should be accessible
        let _weight = FontWeight::Bold;
        let _style = FontStyle::Normal;
        let _stretch = FontStretch::Normal;
        let _direction = Direction::LeftToRight;
        let _mode = RenderMode::Gray;
        let _point = Point { x: 0.0, y: 0.0 };
        let _vector = Vector { x: 1.0, y: 0.0 };
        let _glyph_id = GlyphId { id: 1 };
        let _font_id: FontId = 0;
    }

    #[test]
    fn test_font_descriptor_re_export() {
        // Given: FontDescriptor should be re-exported
        // When: Creating a descriptor at crate root
        // Then: It should work without module prefix
        let descriptor = FontDescriptor {
            family: vec!["Test".to_string()],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 12.0,
        };
        assert_eq!(descriptor.size, 12.0);
    }

    #[test]
    fn test_font_metrics_re_export() {
        // Given: FontMetrics should be re-exported
        // When: Creating metrics at crate root
        // Then: It should work without module prefix
        let metrics = FontMetrics {
            units_per_em: 1000,
            ascent: 800.0,
            descent: -200.0,
            line_gap: 0.0,
            cap_height: 700.0,
            x_height: 500.0,
            underline_position: -50.0,
            underline_thickness: 25.0,
        };
        assert_eq!(metrics.units_per_em, 1000);
    }
}
