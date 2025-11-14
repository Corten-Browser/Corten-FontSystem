//! glyph_renderer - Glyph rasterization, hinting, subpixel rendering, and glyph caching

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod types;

use std::collections::HashMap;
use types::*;

use freetype as ft;

/// Glyph renderer with caching support
pub struct GlyphRenderer {
    cache: GlyphCache,
}

/// Glyph cache key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    glyph_id: GlyphId,
    size: u32, // Size in fixed-point (size * 64)
    mode: RenderMode,
}

/// Internal glyph cache
struct GlyphCache {
    entries: HashMap<CacheKey, GlyphBitmap>,
    hits: u64,
    misses: u64,
}

impl GlyphCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn get_stats(&self) -> CacheStats {
        let memory_bytes = self.entries.values().map(|bitmap| bitmap.data.len()).sum();

        CacheStats {
            entries: self.entries.len(),
            hits: self.hits,
            misses: self.misses,
            memory_bytes,
        }
    }
}

/// Convert RenderMode to FreeType render mode
fn to_freetype_render_mode(mode: RenderMode) -> ft::RenderMode {
    match mode {
        RenderMode::Mono => ft::RenderMode::Mono,
        RenderMode::Gray => ft::RenderMode::Normal,
        RenderMode::SubpixelRgb => ft::RenderMode::Lcd,
    }
}

/// Convert FreeType load flags from RenderMode
fn get_load_flags(mode: RenderMode) -> ft::face::LoadFlag {
    match mode {
        RenderMode::Mono => ft::face::LoadFlag::MONOCHROME,
        RenderMode::Gray => ft::face::LoadFlag::DEFAULT,
        RenderMode::SubpixelRgb => ft::face::LoadFlag::DEFAULT,
    }
}

impl GlyphRenderer {
    /// Create a new glyph renderer
    pub fn new() -> Self {
        Self {
            cache: GlyphCache::new(),
        }
    }

    /// Rasterize a glyph to bitmap
    pub fn rasterize_glyph(
        &mut self,
        font: &OpenTypeFont,
        glyph_id: GlyphId,
        size: f32,
        mode: RenderMode,
    ) -> Result<GlyphBitmap, RenderError> {
        // Create cache key
        let size_fixed = (size * 64.0) as u32; // Convert to fixed-point (26.6)
        let cache_key = CacheKey {
            glyph_id,
            size: size_fixed,
            mode,
        };

        // Check cache first
        if let Some(bitmap) = self.cache.entries.get(&cache_key) {
            self.cache.hits += 1;
            return Ok(bitmap.clone());
        }

        // Cache miss
        self.cache.misses += 1;

        // Check if font has data
        if font.data.is_empty() {
            return Err(RenderError::RasterizationFailed(
                "Font has no data (stub font)".to_string(),
            ));
        }

        // Rasterize using FreeType
        let bitmap = self.rasterize_with_freetype(font, glyph_id, size, mode)?;

        // Store in cache
        self.cache.entries.insert(cache_key, bitmap.clone());

        Ok(bitmap)
    }

    /// Internal method to rasterize using FreeType
    fn rasterize_with_freetype(
        &self,
        font: &OpenTypeFont,
        glyph_id: GlyphId,
        size: f32,
        mode: RenderMode,
    ) -> Result<GlyphBitmap, RenderError> {
        // Initialize FreeType library
        let library = ft::Library::init().map_err(|e| {
            RenderError::RasterizationFailed(format!("Failed to initialize FreeType: {:?}", e))
        })?;

        // Load font face from memory
        // freetype-rs requires Rc<Vec<u8>>
        let font_data = std::rc::Rc::new(font.data.clone());
        let face = library
            .new_memory_face(font_data, font.face_index)
            .map_err(|e| {
                RenderError::RasterizationFailed(format!("Failed to load font face: {:?}", e))
            })?;

        // Set character size (size in points * 64, DPI = 72)
        let size_26dot6 = (size * 64.0) as isize;
        face.set_char_size(size_26dot6, 0, 72, 72).map_err(|e| {
            RenderError::RasterizationFailed(format!("Failed to set char size: {:?}", e))
        })?;

        // Load glyph
        let load_flags = get_load_flags(mode);
        face.load_glyph(glyph_id.0 as u32, load_flags)
            .map_err(|e| {
                if matches!(e, ft::Error::InvalidGlyphIndex) {
                    RenderError::GlyphNotFound(glyph_id)
                } else {
                    RenderError::RasterizationFailed(format!("Failed to load glyph: {:?}", e))
                }
            })?;

        // Render glyph to bitmap
        let ft_render_mode = to_freetype_render_mode(mode);
        face.glyph().render_glyph(ft_render_mode).map_err(|e| {
            RenderError::RasterizationFailed(format!("Failed to render glyph: {:?}", e))
        })?;

        // Extract bitmap data
        let ft_bitmap = face.glyph().bitmap();
        let width = ft_bitmap.width() as u32;
        let height = ft_bitmap.rows() as u32;
        let pitch = ft_bitmap.pitch().unsigned_abs() as usize;

        // Copy bitmap data
        let buffer = ft_bitmap.buffer();
        let data = buffer.to_vec();

        // Extract metrics
        let metrics = face.glyph().metrics();
        let bearing_x = metrics.horiBearingX / 64; // Convert from 26.6 fixed-point
        let bearing_y = metrics.horiBearingY / 64;

        Ok(GlyphBitmap {
            width,
            height,
            left: bearing_x as i32,
            top: bearing_y as i32,
            pitch,
            data,
            format: mode,
        })
    }

    /// Get glyph vector outline
    pub fn get_glyph_outline(
        &self,
        font: &OpenTypeFont,
        glyph_id: GlyphId,
    ) -> Result<GlyphOutline, RenderError> {
        // Check if font has data
        if font.data.is_empty() {
            return Err(RenderError::RasterizationFailed(
                "Font has no data (stub font)".to_string(),
            ));
        }

        // Initialize FreeType library
        let library = ft::Library::init().map_err(|e| {
            RenderError::RasterizationFailed(format!("Failed to initialize FreeType: {:?}", e))
        })?;

        // Load font face from memory
        let font_data = std::rc::Rc::new(font.data.clone());
        let face = library
            .new_memory_face(font_data, font.face_index)
            .map_err(|e| {
                RenderError::RasterizationFailed(format!("Failed to load font face: {:?}", e))
            })?;

        // Set a default size for outline extraction
        face.set_char_size(16 * 64, 0, 72, 72).map_err(|e| {
            RenderError::RasterizationFailed(format!("Failed to set char size: {:?}", e))
        })?;

        // Load glyph without rendering
        face.load_glyph(glyph_id.0 as u32, ft::face::LoadFlag::NO_BITMAP)
            .map_err(|e| {
                if matches!(e, ft::Error::InvalidGlyphIndex) {
                    RenderError::GlyphNotFound(glyph_id)
                } else {
                    RenderError::RasterizationFailed(format!("Failed to load glyph: {:?}", e))
                }
            })?;

        // Get outline
        let glyph = face.glyph();
        let ft_outline = glyph
            .outline()
            .ok_or_else(|| RenderError::RasterizationFailed("Glyph has no outline".to_string()))?;

        // Extract points and contours
        let points_vec = ft_outline.points();
        let contours_vec = ft_outline.contours();

        // Calculate bounding box manually from points
        let mut min_x = i64::MAX;
        let mut min_y = i64::MAX;
        let mut max_x = i64::MIN;
        let mut max_y = i64::MIN;

        for point in points_vec {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        let bounds = if points_vec.is_empty() {
            BoundingBox {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 0.0,
                max_y: 0.0,
            }
        } else {
            BoundingBox {
                min_x: (min_x as f32) / 64.0,
                min_y: (min_y as f32) / 64.0,
                max_x: (max_x as f32) / 64.0,
                max_y: (max_y as f32) / 64.0,
            }
        };

        // Extract contours
        let mut contours = Vec::new();
        let mut start_idx = 0;

        for &contour_end in contours_vec {
            let end_idx = contour_end as usize + 1;

            // Extract points for this contour
            let points: Vec<Point> = points_vec[start_idx..end_idx.min(points_vec.len())]
                .iter()
                .map(|point| Point {
                    x: (point.x as f32) / 64.0,
                    y: (point.y as f32) / 64.0,
                })
                .collect();

            if !points.is_empty() {
                contours.push(Contour {
                    points,
                    closed: true,
                });
            }

            start_idx = end_idx;
        }

        Ok(GlyphOutline { contours, bounds })
    }

    /// Clear the glyph cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.get_stats()
    }
}

impl Default for GlyphRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_renderer() {
        let renderer = GlyphRenderer::new();
        assert!(renderer.cache.entries.is_empty());
    }

    #[test]
    fn test_cache_stats_initially_empty() {
        let renderer = GlyphRenderer::new();
        let stats = renderer.cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }
}
