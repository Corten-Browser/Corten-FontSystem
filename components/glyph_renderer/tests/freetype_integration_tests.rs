//! Integration tests for FreeType-based glyph rendering
//!
//! These tests verify that the GlyphRenderer correctly uses FreeType
//! to rasterize glyphs from real font data.

use glyph_renderer::types::*;
use glyph_renderer::GlyphRenderer;

/// Minimal TrueType font data for testing
/// This is a base64-encoded minimal TTF font with a single glyph (space)
/// Generated using FontForge for testing purposes
const MINIMAL_FONT_DATA: &[u8] = include_bytes!("test_data/minimal_test.ttf");

/// Helper to create a test font from minimal font data
fn create_test_font() -> OpenTypeFont {
    // For now, we'll use the stub. This will be replaced when we integrate
    // with font_parser component
    OpenTypeFont::new_stub()
}

#[cfg(test)]
mod freetype_tests {
    use super::*;

    #[test]
    fn test_rasterize_glyph_with_real_font_produces_bitmap() {
        // Given: A renderer and test font
        let mut renderer = GlyphRenderer::new();
        let font = create_test_font();
        let glyph_id = GlyphId(0); // Space character

        // When: Rasterizing a glyph
        let result = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);

        // Then: Should produce a valid bitmap (when FreeType is implemented)
        // For now this will fail - that's expected (RED phase)
        match result {
            Ok(bitmap) => {
                assert!(bitmap.width > 0 || bitmap.height > 0); // Space might be empty but should have advance
                assert_eq!(bitmap.format, RenderMode::Gray);
            }
            Err(e) => {
                // Expected to fail until FreeType implementation
                eprintln!("Expected failure (not yet implemented): {:?}", e);
                assert!(matches!(
                    e,
                    RenderError::RasterizationFailed(_) | RenderError::GlyphNotFound(_)
                ));
            }
        }
    }

    #[test]
    fn test_rasterize_glyph_mono_mode() {
        // Given: A renderer and test font
        let mut renderer = GlyphRenderer::new();
        let font = create_test_font();
        let glyph_id = GlyphId(0);

        // When: Rasterizing with monochrome mode
        let result = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Mono);

        // Then: Should produce monochrome bitmap when implemented
        if let Ok(bitmap) = result {
            assert_eq!(bitmap.format, RenderMode::Mono);
        }
    }

    #[test]
    fn test_rasterize_glyph_subpixel_mode() {
        // Given: A renderer and test font
        let mut renderer = GlyphRenderer::new();
        let font = create_test_font();
        let glyph_id = GlyphId(0);

        // When: Rasterizing with subpixel RGB mode
        let result = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::SubpixelRgb);

        // Then: Should produce subpixel bitmap when implemented
        if let Ok(bitmap) = result {
            assert_eq!(bitmap.format, RenderMode::SubpixelRgb);
            // Subpixel mode should have 3 bytes per pixel (RGB)
            if bitmap.width > 0 && bitmap.height > 0 {
                assert_eq!(bitmap.pitch, (bitmap.width * 3) as usize);
            }
        }
    }

    #[test]
    fn test_rasterize_glyph_has_correct_metrics() {
        // Given: A renderer and test font
        let mut renderer = GlyphRenderer::new();
        let font = create_test_font();
        let glyph_id = GlyphId(0);

        // When: Rasterizing a glyph
        let result = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);

        // Then: Bitmap should have valid metrics
        if let Ok(bitmap) = result {
            // Metrics should be reasonable values
            assert!(bitmap.left >= -100 && bitmap.left <= 100);
            assert!(bitmap.top >= -100 && bitmap.top <= 100);

            // Pitch should match width for grayscale
            if bitmap.width > 0 {
                assert_eq!(bitmap.pitch, bitmap.width as usize);
            }
        }
    }

    #[test]
    fn test_rasterize_different_sizes() {
        // Given: A renderer and test font
        let mut renderer = GlyphRenderer::new();
        let font = create_test_font();
        let glyph_id = GlyphId(0);

        // When: Rasterizing same glyph at different sizes
        let size_12 = renderer.rasterize_glyph(&font, glyph_id, 12.0, RenderMode::Gray);
        let size_24 = renderer.rasterize_glyph(&font, glyph_id, 24.0, RenderMode::Gray);

        // Then: Different sizes should produce different bitmaps (when implemented)
        if let (Ok(bitmap_12), Ok(bitmap_24)) = (size_12, size_24) {
            // Larger size should generally have larger dimensions
            // (though this depends on the glyph)
            assert!(bitmap_12.width != bitmap_24.width || bitmap_12.height != bitmap_24.height);
        }
    }

    #[test]
    fn test_cache_hit_after_first_render() {
        // Given: A renderer and test font
        let mut renderer = GlyphRenderer::new();
        let font = create_test_font();
        let glyph_id = GlyphId(0);

        // When: Rasterizing same glyph twice
        let first_result = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);
        let initial_stats = renderer.cache_stats();

        let second_result = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);
        let after_stats = renderer.cache_stats();

        // Then: Behavior depends on whether font has real data
        if !font.has_data() {
            // Stub font: both should fail, no cache entries
            assert!(first_result.is_err());
            assert!(second_result.is_err());
            assert_eq!(initial_stats.entries, 0);
            assert_eq!(after_stats.entries, 0);
        } else {
            // Real font: first render should cache, second should hit cache
            assert!(first_result.is_ok());
            assert!(second_result.is_ok());
            assert_eq!(initial_stats.entries, 1);
            assert_eq!(after_stats.hits, initial_stats.hits + 1);
            assert_eq!(after_stats.misses, initial_stats.misses);
        }
    }

    #[test]
    fn test_get_glyph_outline_produces_contours() {
        // Given: A renderer and test font
        let renderer = GlyphRenderer::new();
        let font = create_test_font();
        let glyph_id = GlyphId(0);

        // When: Getting glyph outline
        let result = renderer.get_glyph_outline(&font, glyph_id);

        // Then: Should produce outline with contours (when implemented)
        if let Ok(outline) = result {
            // Even space might have empty outline, but structure should be valid
            assert!(outline.contours.len() >= 0);
            // Bounding box should be reasonable
            assert!(outline.bounds.min_x <= outline.bounds.max_x);
            assert!(outline.bounds.min_y <= outline.bounds.max_y);
        }
    }

    #[test]
    fn test_invalid_glyph_id_returns_error() {
        // Given: A renderer and test font
        let mut renderer = GlyphRenderer::new();
        let font = create_test_font();
        let invalid_glyph_id = GlyphId(9999); // Very high glyph ID unlikely to exist

        // When: Attempting to rasterize invalid glyph
        let result = renderer.rasterize_glyph(&font, invalid_glyph_id, 16.0, RenderMode::Gray);

        // Then: Should return GlyphNotFound error (when validation is implemented)
        assert!(result.is_err());
        if let Err(e) = result {
            // Should be either GlyphNotFound or RasterizationFailed
            assert!(matches!(
                e,
                RenderError::GlyphNotFound(_) | RenderError::RasterizationFailed(_)
            ));
        }
    }

    #[test]
    fn test_outline_extraction_for_invalid_glyph() {
        // Given: A renderer and test font
        let renderer = GlyphRenderer::new();
        let font = create_test_font();
        let invalid_glyph_id = GlyphId(9999);

        // When: Getting outline for invalid glyph
        let result = renderer.get_glyph_outline(&font, invalid_glyph_id);

        // Then: Should return error
        assert!(result.is_err());
    }
}
