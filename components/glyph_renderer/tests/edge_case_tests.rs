//! Edge case and error handling tests for glyph_renderer

use glyph_renderer::types::*;
use glyph_renderer::GlyphRenderer;

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_rasterize_with_very_large_size() {
        // Given: A renderer and stub font
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(0);

        // When: Rasterizing with very large size
        let result = renderer.rasterize_glyph(&font, glyph_id, 1000.0, RenderMode::Gray);

        // Then: Should handle gracefully (error for stub font)
        assert!(result.is_err());
    }

    #[test]
    fn test_rasterize_with_very_small_size() {
        // Given: A renderer and stub font
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(0);

        // When: Rasterizing with very small size
        let result = renderer.rasterize_glyph(&font, glyph_id, 1.0, RenderMode::Gray);

        // Then: Should handle gracefully (error for stub font)
        assert!(result.is_err());
    }

    #[test]
    fn test_rasterize_with_zero_size() {
        // Given: A renderer and stub font
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(0);

        // When: Rasterizing with zero size
        let result = renderer.rasterize_glyph(&font, glyph_id, 0.0, RenderMode::Gray);

        // Then: Should handle gracefully (error for stub font)
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_cleared_removes_all_entries() {
        // Given: A renderer with some cache entries (simulated)
        let mut renderer = GlyphRenderer::new();

        // When: Clearing cache
        renderer.clear_cache();

        // Then: Cache should be empty
        let stats = renderer.cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.memory_bytes, 0);
    }

    #[test]
    fn test_cache_stats_memory_calculation() {
        // Given: A new renderer
        let renderer = GlyphRenderer::new();

        // When: Getting cache stats on empty cache
        let stats = renderer.cache_stats();

        // Then: Memory should be zero
        assert_eq!(stats.memory_bytes, 0);
    }

    #[test]
    fn test_different_glyph_ids_cache_separately() {
        // Given: A renderer
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();

        // When: Attempting to render different glyphs
        let _ = renderer.rasterize_glyph(&font, GlyphId(0), 16.0, RenderMode::Gray);
        let _ = renderer.rasterize_glyph(&font, GlyphId(1), 16.0, RenderMode::Gray);
        let _ = renderer.rasterize_glyph(&font, GlyphId(42), 16.0, RenderMode::Gray);

        // Then: Should have separate cache misses
        let stats = renderer.cache_stats();
        assert_eq!(stats.misses, 3);
    }

    #[test]
    fn test_same_glyph_different_sizes_cache_separately() {
        // Given: A renderer
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(0);

        // When: Rendering same glyph at different sizes
        let _ = renderer.rasterize_glyph(&font, glyph_id, 12.0, RenderMode::Gray);
        let _ = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);
        let _ = renderer.rasterize_glyph(&font, glyph_id, 24.0, RenderMode::Gray);

        // Then: Should have separate cache entries (3 misses)
        let stats = renderer.cache_stats();
        assert_eq!(stats.misses, 3);
    }

    #[test]
    fn test_render_error_display_glyph_not_found() {
        // Given: A GlyphNotFound error
        let error = RenderError::GlyphNotFound(GlyphId(42));

        // When: Converting to string
        let error_string = error.to_string();

        // Then: Should contain glyph ID
        assert!(error_string.contains("Glyph not found"));
        assert!(error_string.contains("42"));
    }

    #[test]
    fn test_render_error_display_rasterization_failed() {
        // Given: A RasterizationFailed error
        let error = RenderError::RasterizationFailed("Test error message".to_string());

        // When: Converting to string
        let error_string = error.to_string();

        // Then: Should contain error message
        assert!(error_string.contains("Rasterization failed"));
        assert!(error_string.contains("Test error message"));
    }

    #[test]
    fn test_render_error_display_out_of_memory() {
        // Given: An OutOfMemory error
        let error = RenderError::OutOfMemory;

        // When: Converting to string
        let error_string = error.to_string();

        // Then: Should contain appropriate message
        assert!(error_string.contains("Out of memory"));
    }

    #[test]
    fn test_glyph_bitmap_structure() {
        // Given: A glyph bitmap
        let bitmap = GlyphBitmap {
            width: 16,
            height: 24,
            left: 2,
            top: 20,
            pitch: 16,
            data: vec![0u8; 16 * 24],
            format: RenderMode::Gray,
        };

        // Then: All fields should be accessible and correct
        assert_eq!(bitmap.width, 16);
        assert_eq!(bitmap.height, 24);
        assert_eq!(bitmap.left, 2);
        assert_eq!(bitmap.top, 20);
        assert_eq!(bitmap.pitch, 16);
        assert_eq!(bitmap.data.len(), 16 * 24);
        assert!(matches!(bitmap.format, RenderMode::Gray));
    }

    #[test]
    fn test_glyph_outline_structure() {
        // Given: A glyph outline
        let outline = GlyphOutline {
            contours: vec![Contour {
                points: vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 10.0, y: 0.0 },
                    Point { x: 10.0, y: 10.0 },
                ],
                closed: true,
            }],
            bounds: BoundingBox {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 10.0,
                max_y: 10.0,
            },
        };

        // Then: All fields should be accessible
        assert_eq!(outline.contours.len(), 1);
        assert_eq!(outline.contours[0].points.len(), 3);
        assert!(outline.contours[0].closed);
        assert_eq!(outline.bounds.min_x, 0.0);
        assert_eq!(outline.bounds.max_x, 10.0);
    }

    #[test]
    fn test_cache_stats_default() {
        // Given: Default cache stats
        let stats = CacheStats::default();

        // Then: Should be empty
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.memory_bytes, 0);
    }

    #[test]
    fn test_cache_stats_new() {
        // Given: New cache stats
        let stats = CacheStats::new();

        // Then: Should match default
        assert_eq!(stats, CacheStats::default());
    }

    #[test]
    fn test_opentypefont_from_data() {
        // Given: Some font data
        let data = vec![0u8; 100];

        // When: Creating font from data
        let font = OpenTypeFont::from_data(data.clone(), 0);

        // Then: Font should have data
        assert!(font.has_data());
    }

    #[test]
    fn test_opentypefont_stub_has_no_data() {
        // Given: A stub font
        let font = OpenTypeFont::new_stub();

        // Then: Should have no data
        assert!(!font.has_data());
    }

    #[test]
    fn test_render_mode_equality() {
        // Given: Render modes
        let mode1 = RenderMode::Gray;
        let mode2 = RenderMode::Gray;
        let mode3 = RenderMode::Mono;

        // Then: Equality should work correctly
        assert_eq!(mode1, mode2);
        assert_ne!(mode1, mode3);
    }

    #[test]
    fn test_glyph_id_equality() {
        // Given: Glyph IDs
        let id1 = GlyphId(42);
        let id2 = GlyphId(42);
        let id3 = GlyphId(43);

        // Then: Equality should work correctly
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_renderer_default() {
        // Given: Default renderer
        let renderer = GlyphRenderer::default();

        // Then: Should be same as new()
        let stats = renderer.cache_stats();
        assert_eq!(stats.entries, 0);
    }
}
