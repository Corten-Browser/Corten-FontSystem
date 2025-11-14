//! Unit tests for GlyphRenderer

use glyph_renderer::types::*;
use glyph_renderer::GlyphRenderer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_renderer() {
        // Given: No prerequisites

        // When: Creating a new GlyphRenderer
        let renderer = GlyphRenderer::new();

        // Then: Renderer is created successfully
        // (Just verifying it compiles and doesn't panic)
        drop(renderer);
    }

    #[test]
    fn test_new_renderer_has_empty_cache() {
        // Given: No prerequisites

        // When: Creating a new GlyphRenderer
        let renderer = GlyphRenderer::new();

        // Then: Cache should be empty
        let stats = renderer.cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.memory_bytes, 0);
    }

    #[test]
    fn test_clear_cache_on_empty_cache() {
        // Given: A new renderer with empty cache
        let mut renderer = GlyphRenderer::new();

        // When: Clearing the cache
        renderer.clear_cache();

        // Then: Cache stats remain zero
        let stats = renderer.cache_stats();
        assert_eq!(stats.entries, 0);
    }

    #[test]
    fn test_cache_stats_returns_valid_stats() {
        // Given: A new renderer
        let renderer = GlyphRenderer::new();

        // When: Getting cache stats
        let stats = renderer.cache_stats();

        // Then: Stats should be valid CacheStats structure
        assert!(stats.entries == 0);
        assert!(stats.hits == 0);
        assert!(stats.misses == 0);
        assert!(stats.memory_bytes == 0);
    }

    #[test]
    fn test_rasterize_glyph_returns_error_for_stub_font() {
        // Given: A renderer and a stub font
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(0);

        // When: Rasterizing a glyph
        let result = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);

        // Then: Should return an error (stub font has no real data)
        assert!(result.is_err());
    }

    #[test]
    fn test_rasterize_glyph_increments_cache_miss() {
        // Given: A renderer with empty cache
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(0);

        let initial_stats = renderer.cache_stats();
        assert_eq!(initial_stats.misses, 0);

        // When: Attempting to rasterize (will fail with stub font)
        let _ = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);

        // Then: Cache miss should be incremented
        let stats = renderer.cache_stats();
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_get_glyph_outline_returns_error_for_stub_font() {
        // Given: A renderer and a stub font
        let renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(0);

        // When: Getting glyph outline
        let result = renderer.get_glyph_outline(&font, glyph_id);

        // Then: Should return an error (stub font has no real data)
        assert!(result.is_err());
    }

    #[test]
    fn test_different_render_modes_are_distinct_in_cache() {
        // Given: A renderer
        let mut renderer = GlyphRenderer::new();
        let font = OpenTypeFont::new_stub();
        let glyph_id = GlyphId(42);

        // When: Attempting to rasterize same glyph with different modes
        let _ = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);
        let _ = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Mono);
        let _ = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::SubpixelRgb);

        // Then: Should have 3 cache misses (different modes = different cache keys)
        let stats = renderer.cache_stats();
        assert_eq!(stats.misses, 3);
    }
}
