//! Common types for the font system

/// Font weight values (100-900)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum FontWeight {
    /// Thin weight (100)
    Thin = 100,
    /// Extra light weight (200)
    ExtraLight = 200,
    /// Light weight (300)
    Light = 300,
    /// Regular/normal weight (400)
    Regular = 400,
    /// Medium weight (500)
    Medium = 500,
    /// Semi-bold weight (600)
    SemiBold = 600,
    /// Bold weight (700)
    Bold = 700,
    /// Extra bold weight (800)
    ExtraBold = 800,
    /// Black/heavy weight (900)
    Black = 900,
}

/// Font style variations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    /// Normal/upright style
    Normal,
    /// Italic style
    Italic,
    /// Oblique style with angle in degrees
    Oblique(f32),
}

/// Font stretch values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum FontStretch {
    /// Ultra condensed (50%)
    UltraCondensed = 50,
    /// Extra condensed (62%)
    ExtraCondensed = 62,
    /// Condensed (75%)
    Condensed = 75,
    /// Semi condensed (87%)
    SemiCondensed = 87,
    /// Normal width (100%)
    Normal = 100,
    /// Semi expanded (112%)
    SemiExpanded = 112,
    /// Expanded (125%)
    Expanded = 125,
    /// Extra expanded (150%)
    ExtraExpanded = 150,
    /// Ultra expanded (200%)
    UltraExpanded = 200,
}

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Left-to-right text direction
    LeftToRight,
    /// Right-to-left text direction
    RightToLeft,
    /// Top-to-bottom text direction
    TopToBottom,
    /// Bottom-to-top text direction
    BottomToTop,
}

/// Glyph rasterization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// 1-bit monochrome rendering
    Mono,
    /// 8-bit grayscale rendering
    Gray,
    /// Subpixel RGB rendering
    SubpixelRgb,
    /// Subpixel BGR rendering
    SubpixelBgr,
    /// Vertical subpixel RGB rendering
    SubpixelVrgb,
    /// Vertical subpixel BGR rendering
    SubpixelVbgr,
}

/// 2D point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

/// 2D vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
}

/// Glyph identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphId {
    /// Glyph ID value
    pub id: u32,
}

/// Font identifier
pub type FontId = usize;

/// Font metrics and measurements
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Units per em
    pub units_per_em: u16,
    /// Ascent in font units
    pub ascent: f32,
    /// Descent in font units (typically negative)
    pub descent: f32,
    /// Line gap in font units
    pub line_gap: f32,
    /// Cap height in font units
    pub cap_height: f32,
    /// X-height in font units
    pub x_height: f32,
    /// Underline position in font units
    pub underline_position: f32,
    /// Underline thickness in font units
    pub underline_thickness: f32,
}

/// Font selection descriptor
#[derive(Debug, Clone, PartialEq)]
pub struct FontDescriptor {
    /// Font family names (fallback chain)
    pub family: Vec<String>,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Font stretch
    pub stretch: FontStretch,
    /// Font size in pixels
    pub size: f32,
}

/// Positioned glyph with layout information
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    /// Glyph identifier
    pub glyph_id: GlyphId,
    /// Font identifier
    pub font_id: FontId,
    /// Baseline position
    pub position: Point,
    /// Advance to next glyph
    pub advance: Vector,
    /// Positioning offset
    pub offset: Vector,
}

/// Shaped text result
#[derive(Debug, Clone)]
pub struct ShapedText {
    /// Positioned glyphs
    pub glyphs: Vec<PositionedGlyph>,
    /// Total width
    pub width: f32,
    /// Total height
    pub height: f32,
    /// Baseline offset
    pub baseline: f32,
}

/// Rendered glyph bitmap
pub struct GlyphBitmap {
    /// Bitmap width
    pub width: u32,
    /// Bitmap height
    pub height: u32,
    /// Bearing X (horizontal offset from cursor)
    pub left: i32,
    /// Bearing Y (vertical offset from baseline)
    pub top: i32,
    /// Bytes per row
    pub pitch: usize,
    /// Pixel data
    pub data: Vec<u8>,
    /// Rendering format
    pub format: RenderMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== FontWeight Tests ==========

    #[test]
    fn test_font_weight_variants_exist() {
        // Given: Font weight values should be available
        // When: Creating each weight variant
        // Then: All variants should compile and be distinct
        let _thin = FontWeight::Thin;
        let _extra_light = FontWeight::ExtraLight;
        let _light = FontWeight::Light;
        let _regular = FontWeight::Regular;
        let _medium = FontWeight::Medium;
        let _semi_bold = FontWeight::SemiBold;
        let _bold = FontWeight::Bold;
        let _extra_bold = FontWeight::ExtraBold;
        let _black = FontWeight::Black;
    }

    #[test]
    fn test_font_weight_numeric_values() {
        // Given: Font weight should map to numeric values
        // When: Casting to numeric values
        // Then: Values should match OpenType specification
        assert_eq!(FontWeight::Thin as u16, 100);
        assert_eq!(FontWeight::ExtraLight as u16, 200);
        assert_eq!(FontWeight::Light as u16, 300);
        assert_eq!(FontWeight::Regular as u16, 400);
        assert_eq!(FontWeight::Medium as u16, 500);
        assert_eq!(FontWeight::SemiBold as u16, 600);
        assert_eq!(FontWeight::Bold as u16, 700);
        assert_eq!(FontWeight::ExtraBold as u16, 800);
        assert_eq!(FontWeight::Black as u16, 900);
    }

    #[test]
    fn test_font_weight_equality() {
        // Given: Two font weights
        // When: Comparing them
        // Then: Same weights should be equal
        assert_eq!(FontWeight::Bold, FontWeight::Bold);
        assert_ne!(FontWeight::Bold, FontWeight::Regular);
    }

    #[test]
    fn test_font_weight_ordering() {
        // Given: Font weights with different values
        // When: Comparing their order
        // Then: Heavier weights should be greater
        assert!(FontWeight::Bold > FontWeight::Regular);
        assert!(FontWeight::Thin < FontWeight::Black);
        assert!(FontWeight::Regular <= FontWeight::Bold);
    }

    #[test]
    fn test_font_weight_clone_and_copy() {
        // Given: A font weight
        // When: Cloning or copying it
        // Then: Should produce identical values
        let weight = FontWeight::Bold;
        let cloned = weight.clone();
        let copied = weight;
        assert_eq!(weight, cloned);
        assert_eq!(weight, copied);
    }

    // ========== FontStyle Tests ==========

    #[test]
    fn test_font_style_variants() {
        // Given: Font style variations
        // When: Creating each variant
        // Then: All variants should be available
        let _normal = FontStyle::Normal;
        let _italic = FontStyle::Italic;
        let _oblique = FontStyle::Oblique(15.0);
    }

    #[test]
    fn test_font_style_equality() {
        // Given: Font style instances
        // When: Comparing them
        // Then: Same styles should be equal
        assert_eq!(FontStyle::Normal, FontStyle::Normal);
        assert_eq!(FontStyle::Italic, FontStyle::Italic);
        assert_ne!(FontStyle::Normal, FontStyle::Italic);
    }

    #[test]
    fn test_font_style_oblique_angle() {
        // Given: Oblique style with angle
        // When: Creating with different angles
        // Then: Angles should be stored correctly
        let oblique_15 = FontStyle::Oblique(15.0);
        let oblique_20 = FontStyle::Oblique(20.0);

        if let FontStyle::Oblique(angle) = oblique_15 {
            assert_eq!(angle, 15.0);
        } else {
            panic!("Expected Oblique variant");
        }

        assert_ne!(oblique_15, oblique_20);
    }

    #[test]
    fn test_font_style_clone() {
        // Given: A font style
        // When: Cloning it
        // Then: Should produce identical value
        let style = FontStyle::Oblique(12.5);
        let cloned = style.clone();
        assert_eq!(style, cloned);
    }

    // ========== FontStretch Tests ==========

    #[test]
    fn test_font_stretch_variants() {
        // Given: Font stretch values
        // When: Creating each variant
        // Then: All variants should be available
        let _ultra_condensed = FontStretch::UltraCondensed;
        let _extra_condensed = FontStretch::ExtraCondensed;
        let _condensed = FontStretch::Condensed;
        let _semi_condensed = FontStretch::SemiCondensed;
        let _normal = FontStretch::Normal;
        let _semi_expanded = FontStretch::SemiExpanded;
        let _expanded = FontStretch::Expanded;
        let _extra_expanded = FontStretch::ExtraExpanded;
        let _ultra_expanded = FontStretch::UltraExpanded;
    }

    #[test]
    fn test_font_stretch_numeric_values() {
        // Given: Font stretch should map to percentage values
        // When: Casting to numeric values
        // Then: Values should match CSS specification
        assert_eq!(FontStretch::UltraCondensed as u16, 50);
        assert_eq!(FontStretch::ExtraCondensed as u16, 62);
        assert_eq!(FontStretch::Condensed as u16, 75);
        assert_eq!(FontStretch::SemiCondensed as u16, 87);
        assert_eq!(FontStretch::Normal as u16, 100);
        assert_eq!(FontStretch::SemiExpanded as u16, 112);
        assert_eq!(FontStretch::Expanded as u16, 125);
        assert_eq!(FontStretch::ExtraExpanded as u16, 150);
        assert_eq!(FontStretch::UltraExpanded as u16, 200);
    }

    #[test]
    fn test_font_stretch_ordering() {
        // Given: Font stretch values
        // When: Comparing their order
        // Then: More expanded should be greater
        assert!(FontStretch::Expanded > FontStretch::Normal);
        assert!(FontStretch::Condensed < FontStretch::Normal);
        assert!(FontStretch::UltraCondensed < FontStretch::UltraExpanded);
    }

    // ========== Direction Tests ==========

    #[test]
    fn test_direction_variants() {
        // Given: Text direction options
        // When: Creating each variant
        // Then: All variants should be available
        let _ltr = Direction::LeftToRight;
        let _rtl = Direction::RightToLeft;
        let _ttb = Direction::TopToBottom;
        let _btt = Direction::BottomToTop;
    }

    #[test]
    fn test_direction_equality() {
        // Given: Direction instances
        // When: Comparing them
        // Then: Same directions should be equal
        assert_eq!(Direction::LeftToRight, Direction::LeftToRight);
        assert_ne!(Direction::LeftToRight, Direction::RightToLeft);
    }

    #[test]
    fn test_direction_clone_and_copy() {
        // Given: A text direction
        // When: Cloning or copying it
        // Then: Should produce identical values
        let dir = Direction::RightToLeft;
        let cloned = dir.clone();
        let copied = dir;
        assert_eq!(dir, cloned);
        assert_eq!(dir, copied);
    }

    // ========== RenderMode Tests ==========

    #[test]
    fn test_render_mode_variants() {
        // Given: Rendering mode options
        // When: Creating each variant
        // Then: All variants should be available
        let _mono = RenderMode::Mono;
        let _gray = RenderMode::Gray;
        let _rgb = RenderMode::SubpixelRgb;
        let _bgr = RenderMode::SubpixelBgr;
        let _vrgb = RenderMode::SubpixelVrgb;
        let _vbgr = RenderMode::SubpixelVbgr;
    }

    #[test]
    fn test_render_mode_equality() {
        // Given: Render mode instances
        // When: Comparing them
        // Then: Same modes should be equal
        assert_eq!(RenderMode::Gray, RenderMode::Gray);
        assert_ne!(RenderMode::Mono, RenderMode::Gray);
    }

    #[test]
    fn test_render_mode_clone_and_copy() {
        // Given: A render mode
        // When: Cloning or copying it
        // Then: Should produce identical values
        let mode = RenderMode::SubpixelRgb;
        let cloned = mode.clone();
        let copied = mode;
        assert_eq!(mode, cloned);
        assert_eq!(mode, copied);
    }

    // ========== Point Tests ==========

    #[test]
    fn test_point_creation() {
        // Given: X and Y coordinates
        // When: Creating a point
        // Then: Fields should be set correctly
        let point = Point { x: 10.5, y: 20.3 };
        assert_eq!(point.x, 10.5);
        assert_eq!(point.y, 20.3);
    }

    #[test]
    fn test_point_equality() {
        // Given: Two points with same coordinates
        // When: Comparing them
        // Then: They should be equal
        let p1 = Point { x: 5.0, y: 10.0 };
        let p2 = Point { x: 5.0, y: 10.0 };
        let p3 = Point { x: 5.0, y: 11.0 };
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_point_clone_and_copy() {
        // Given: A point
        // When: Cloning or copying it
        // Then: Should produce identical values
        let point = Point { x: 7.5, y: 15.2 };
        let cloned = point.clone();
        let copied = point;
        assert_eq!(point.x, cloned.x);
        assert_eq!(point.y, cloned.y);
        assert_eq!(point.x, copied.x);
        assert_eq!(point.y, copied.y);
    }

    // ========== Vector Tests ==========

    #[test]
    fn test_vector_creation() {
        // Given: X and Y components
        // When: Creating a vector
        // Then: Fields should be set correctly
        let vector = Vector { x: 3.5, y: -2.1 };
        assert_eq!(vector.x, 3.5);
        assert_eq!(vector.y, -2.1);
    }

    #[test]
    fn test_vector_equality() {
        // Given: Two vectors with same components
        // When: Comparing them
        // Then: They should be equal
        let v1 = Vector { x: 1.0, y: 2.0 };
        let v2 = Vector { x: 1.0, y: 2.0 };
        let v3 = Vector { x: 1.0, y: 3.0 };
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_vector_clone_and_copy() {
        // Given: A vector
        // When: Cloning or copying it
        // Then: Should produce identical values
        let vector = Vector { x: -5.5, y: 8.3 };
        let cloned = vector.clone();
        let copied = vector;
        assert_eq!(vector.x, cloned.x);
        assert_eq!(vector.y, cloned.y);
        assert_eq!(vector.x, copied.x);
        assert_eq!(vector.y, copied.y);
    }

    // ========== GlyphId Tests ==========

    #[test]
    fn test_glyph_id_creation() {
        // Given: A glyph identifier value
        // When: Creating a GlyphId
        // Then: ID should be stored correctly
        let glyph_id = GlyphId { id: 42 };
        assert_eq!(glyph_id.id, 42);
    }

    #[test]
    fn test_glyph_id_equality() {
        // Given: Two glyph IDs
        // When: Comparing them
        // Then: Same IDs should be equal
        let g1 = GlyphId { id: 100 };
        let g2 = GlyphId { id: 100 };
        let g3 = GlyphId { id: 200 };
        assert_eq!(g1, g2);
        assert_ne!(g1, g3);
    }

    #[test]
    fn test_glyph_id_hash() {
        // Given: GlyphId should be hashable
        // When: Using it as a hash map key
        // Then: It should work correctly
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let glyph = GlyphId { id: 1 };
        map.insert(glyph, "test");
        assert_eq!(map.get(&GlyphId { id: 1 }), Some(&"test"));
    }

    #[test]
    fn test_glyph_id_clone_and_copy() {
        // Given: A glyph ID
        // When: Cloning or copying it
        // Then: Should produce identical values
        let glyph_id = GlyphId { id: 99 };
        let cloned = glyph_id.clone();
        let copied = glyph_id;
        assert_eq!(glyph_id, cloned);
        assert_eq!(glyph_id, copied);
    }

    // ========== FontId Tests ==========

    #[test]
    fn test_font_id_type_alias() {
        // Given: FontId is a type alias for usize
        // When: Creating and using a FontId
        // Then: Should behave like a usize
        let font_id: FontId = 42;
        assert_eq!(font_id, 42);
        let another: FontId = font_id + 1;
        assert_eq!(another, 43);
    }

    // ========== FontMetrics Tests ==========

    #[test]
    fn test_font_metrics_creation() {
        // Given: Font metric values
        // When: Creating FontMetrics
        // Then: All fields should be set correctly
        let metrics = FontMetrics {
            units_per_em: 1000,
            ascent: 800.0,
            descent: -200.0,
            line_gap: 100.0,
            cap_height: 700.0,
            x_height: 500.0,
            underline_position: -50.0,
            underline_thickness: 25.0,
        };
        assert_eq!(metrics.units_per_em, 1000);
        assert_eq!(metrics.ascent, 800.0);
        assert_eq!(metrics.descent, -200.0);
        assert_eq!(metrics.line_gap, 100.0);
        assert_eq!(metrics.cap_height, 700.0);
        assert_eq!(metrics.x_height, 500.0);
        assert_eq!(metrics.underline_position, -50.0);
        assert_eq!(metrics.underline_thickness, 25.0);
    }

    #[test]
    fn test_font_metrics_clone() {
        // Given: FontMetrics instance
        // When: Cloning it
        // Then: Should produce identical copy
        let metrics = FontMetrics {
            units_per_em: 2048,
            ascent: 1638.0,
            descent: -410.0,
            line_gap: 0.0,
            cap_height: 1467.0,
            x_height: 1062.0,
            underline_position: -204.0,
            underline_thickness: 102.0,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.units_per_em, metrics.units_per_em);
        assert_eq!(cloned.ascent, metrics.ascent);
    }

    // ========== FontDescriptor Tests ==========

    #[test]
    fn test_font_descriptor_creation() {
        // Given: Font selection criteria
        // When: Creating a FontDescriptor
        // Then: All fields should be set correctly
        let descriptor = FontDescriptor {
            family: vec!["Arial".to_string(), "Helvetica".to_string()],
            weight: FontWeight::Bold,
            style: FontStyle::Italic,
            stretch: FontStretch::Normal,
            size: 16.0,
        };
        assert_eq!(descriptor.family.len(), 2);
        assert_eq!(descriptor.family[0], "Arial");
        assert_eq!(descriptor.weight, FontWeight::Bold);
        assert_eq!(descriptor.style, FontStyle::Italic);
        assert_eq!(descriptor.stretch, FontStretch::Normal);
        assert_eq!(descriptor.size, 16.0);
    }

    #[test]
    fn test_font_descriptor_fallback_chain() {
        // Given: Multiple font families for fallback
        // When: Creating descriptor with fallback chain
        // Then: Order should be preserved
        let descriptor = FontDescriptor {
            family: vec![
                "CustomFont".to_string(),
                "Arial".to_string(),
                "sans-serif".to_string(),
            ],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 12.0,
        };
        assert_eq!(descriptor.family[0], "CustomFont");
        assert_eq!(descriptor.family[1], "Arial");
        assert_eq!(descriptor.family[2], "sans-serif");
    }

    #[test]
    fn test_font_descriptor_equality() {
        // Given: Two font descriptors
        // When: Comparing them
        // Then: Should be equal if all fields match
        let d1 = FontDescriptor {
            family: vec!["Arial".to_string()],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 14.0,
        };
        let d2 = FontDescriptor {
            family: vec!["Arial".to_string()],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 14.0,
        };
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_font_descriptor_clone() {
        // Given: A FontDescriptor
        // When: Cloning it
        // Then: Should produce identical copy
        let descriptor = FontDescriptor {
            family: vec!["Times".to_string()],
            weight: FontWeight::SemiBold,
            style: FontStyle::Oblique(10.0),
            stretch: FontStretch::Condensed,
            size: 18.0,
        };
        let cloned = descriptor.clone();
        assert_eq!(descriptor, cloned);
    }

    // ========== PositionedGlyph Tests ==========

    #[test]
    fn test_positioned_glyph_creation() {
        // Given: Glyph positioning information
        // When: Creating a PositionedGlyph
        // Then: All fields should be set correctly
        let glyph = PositionedGlyph {
            glyph_id: GlyphId { id: 42 },
            font_id: 1,
            position: Point { x: 100.0, y: 200.0 },
            advance: Vector { x: 15.0, y: 0.0 },
            offset: Vector { x: 0.5, y: -0.5 },
        };
        assert_eq!(glyph.glyph_id.id, 42);
        assert_eq!(glyph.font_id, 1);
        assert_eq!(glyph.position.x, 100.0);
        assert_eq!(glyph.position.y, 200.0);
        assert_eq!(glyph.advance.x, 15.0);
        assert_eq!(glyph.advance.y, 0.0);
    }

    #[test]
    fn test_positioned_glyph_clone() {
        // Given: A PositionedGlyph
        // When: Cloning it
        // Then: Should produce identical copy
        let glyph = PositionedGlyph {
            glyph_id: GlyphId { id: 99 },
            font_id: 2,
            position: Point { x: 50.0, y: 75.0 },
            advance: Vector { x: 12.5, y: 0.0 },
            offset: Vector { x: 0.0, y: 0.0 },
        };
        let cloned = glyph.clone();
        assert_eq!(glyph.glyph_id, cloned.glyph_id);
        assert_eq!(glyph.font_id, cloned.font_id);
    }

    // ========== ShapedText Tests ==========

    #[test]
    fn test_shaped_text_creation() {
        // Given: Shaped text information
        // When: Creating ShapedText
        // Then: All fields should be set correctly
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
                advance: Vector { x: 12.0, y: 0.0 },
                offset: Vector { x: 0.0, y: 0.0 },
            },
        ];

        let shaped = ShapedText {
            glyphs: glyphs.clone(),
            width: 22.0,
            height: 16.0,
            baseline: 12.0,
        };

        assert_eq!(shaped.glyphs.len(), 2);
        assert_eq!(shaped.width, 22.0);
        assert_eq!(shaped.height, 16.0);
        assert_eq!(shaped.baseline, 12.0);
    }

    #[test]
    fn test_shaped_text_empty() {
        // Given: Empty shaped text
        // When: Creating ShapedText with no glyphs
        // Then: Should have empty glyph vector
        let shaped = ShapedText {
            glyphs: vec![],
            width: 0.0,
            height: 0.0,
            baseline: 0.0,
        };
        assert_eq!(shaped.glyphs.len(), 0);
    }

    #[test]
    fn test_shaped_text_clone() {
        // Given: A ShapedText instance
        // When: Cloning it
        // Then: Should produce identical copy
        let shaped = ShapedText {
            glyphs: vec![],
            width: 100.0,
            height: 20.0,
            baseline: 15.0,
        };
        let cloned = shaped.clone();
        assert_eq!(shaped.width, cloned.width);
        assert_eq!(shaped.height, cloned.height);
        assert_eq!(shaped.baseline, cloned.baseline);
    }

    // ========== GlyphBitmap Tests ==========

    #[test]
    fn test_glyph_bitmap_creation() {
        // Given: Bitmap data for a glyph
        // When: Creating a GlyphBitmap
        // Then: All fields should be set correctly
        let data = vec![0xFF, 0x00, 0xFF, 0x00];
        let bitmap = GlyphBitmap {
            width: 2,
            height: 2,
            left: 5,
            top: 10,
            pitch: 2,
            data: data.clone(),
            format: RenderMode::Gray,
        };
        assert_eq!(bitmap.width, 2);
        assert_eq!(bitmap.height, 2);
        assert_eq!(bitmap.left, 5);
        assert_eq!(bitmap.top, 10);
        assert_eq!(bitmap.pitch, 2);
        assert_eq!(bitmap.data.len(), 4);
        assert_eq!(bitmap.format, RenderMode::Gray);
    }

    #[test]
    fn test_glyph_bitmap_negative_bearings() {
        // Given: Glyph with negative bearings
        // When: Creating bitmap with negative left/top
        // Then: Should accept negative values
        let bitmap = GlyphBitmap {
            width: 10,
            height: 10,
            left: -2,
            top: -3,
            pitch: 10,
            data: vec![0; 100],
            format: RenderMode::Mono,
        };
        assert_eq!(bitmap.left, -2);
        assert_eq!(bitmap.top, -3);
    }

    #[test]
    fn test_glyph_bitmap_different_formats() {
        // Given: Different render modes
        // When: Creating bitmaps with various formats
        // Then: Format should be stored correctly
        let bitmap_mono = GlyphBitmap {
            width: 8,
            height: 8,
            left: 0,
            top: 0,
            pitch: 1,
            data: vec![0; 8],
            format: RenderMode::Mono,
        };
        assert_eq!(bitmap_mono.format, RenderMode::Mono);

        let bitmap_rgb = GlyphBitmap {
            width: 8,
            height: 8,
            left: 0,
            top: 0,
            pitch: 24,
            data: vec![0; 192],
            format: RenderMode::SubpixelRgb,
        };
        assert_eq!(bitmap_rgb.format, RenderMode::SubpixelRgb);
    }
}
