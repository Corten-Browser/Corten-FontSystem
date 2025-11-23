//! GPU glyph caching with texture atlas support (FEAT-045)
//!
//! This module provides a high-performance glyph cache optimized for GPU rendering.
//! Features include:
//! - Texture atlas packing for efficient GPU uploads
//! - LRU eviction policy with configurable limits
//! - Support for different render modes (mono, gray, subpixel)
//! - Thread-safe access with minimal contention

use crate::types::{GlyphBitmap, GlyphId, RenderMode};
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};

/// Default atlas size (2048x2048 texture)
const DEFAULT_ATLAS_SIZE: u32 = 2048;

/// Default maximum number of atlases
const DEFAULT_MAX_ATLASES: usize = 4;

/// Minimum glyph size for atlas packing
const MIN_GLYPH_SIZE: u32 = 1;

/// Configuration for the GPU glyph cache
#[derive(Debug, Clone)]
pub struct GpuCacheConfig {
    /// Atlas texture width in pixels
    pub atlas_width: u32,
    /// Atlas texture height in pixels
    pub atlas_height: u32,
    /// Maximum number of atlas textures
    pub max_atlases: usize,
    /// Maximum cached glyphs per render mode
    pub max_cached_glyphs: usize,
    /// Enable statistics tracking
    pub enable_statistics: bool,
}

impl Default for GpuCacheConfig {
    fn default() -> Self {
        Self {
            atlas_width: DEFAULT_ATLAS_SIZE,
            atlas_height: DEFAULT_ATLAS_SIZE,
            max_atlases: DEFAULT_MAX_ATLASES,
            max_cached_glyphs: 10_000,
            enable_statistics: true,
        }
    }
}

/// Region within a texture atlas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasRegion {
    /// X position in atlas (pixels)
    pub x: u32,
    /// Y position in atlas (pixels)
    pub y: u32,
    /// Width of region (pixels)
    pub width: u32,
    /// Height of region (pixels)
    pub height: u32,
    /// Atlas index (which texture)
    pub atlas_index: usize,
}

impl AtlasRegion {
    /// Get UV coordinates for this region (normalized 0.0-1.0)
    #[inline]
    pub fn uv_coords(&self, atlas_width: u32, atlas_height: u32) -> (f32, f32, f32, f32) {
        let u0 = self.x as f32 / atlas_width as f32;
        let v0 = self.y as f32 / atlas_height as f32;
        let u1 = (self.x + self.width) as f32 / atlas_width as f32;
        let v1 = (self.y + self.height) as f32 / atlas_height as f32;
        (u0, v0, u1, v1)
    }
}

/// Cached glyph entry with atlas location
#[derive(Debug, Clone)]
pub struct CachedGlyph {
    /// Region in atlas where glyph is stored
    pub region: AtlasRegion,
    /// Left bearing (horizontal offset)
    pub left: i32,
    /// Top bearing (vertical offset)
    pub top: i32,
    /// Render mode used
    pub render_mode: RenderMode,
}

/// Cache key for GPU glyph lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GpuCacheKey {
    /// Glyph identifier
    pub glyph_id: GlyphId,
    /// Font size in fixed-point (size * 64)
    pub size_fixed: u32,
    /// Render mode
    pub render_mode: RenderMode,
}

impl GpuCacheKey {
    /// Create a new cache key
    #[inline]
    pub fn new(glyph_id: GlyphId, size: f32, render_mode: RenderMode) -> Self {
        Self {
            glyph_id,
            size_fixed: (size * 64.0) as u32,
            render_mode,
        }
    }
}

/// Statistics for GPU cache performance
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuCacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of glyphs uploaded to GPU
    pub uploads: u64,
    /// Number of evictions
    pub evictions: u64,
    /// Current number of cached glyphs
    pub cached_glyphs: usize,
    /// Number of active atlases
    pub active_atlases: usize,
    /// Total atlas memory in bytes
    pub atlas_memory_bytes: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

/// Row in the atlas for shelf-first-fit packing
struct AtlasRow {
    y: u32,
    height: u32,
    x_cursor: u32,
}

/// Single texture atlas
struct TextureAtlas {
    /// Raw pixel data (RGBA or grayscale depending on mode)
    data: Vec<u8>,
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Rows for shelf-first-fit packing
    rows: Vec<AtlasRow>,
    /// Current Y cursor (next row start)
    y_cursor: u32,
    /// Bytes per pixel (1 for gray/mono, 4 for RGBA)
    bytes_per_pixel: usize,
    /// Whether the atlas has been modified (needs GPU upload)
    dirty: bool,
}

impl TextureAtlas {
    fn new(width: u32, height: u32, render_mode: RenderMode) -> Self {
        let bytes_per_pixel = match render_mode {
            RenderMode::Mono | RenderMode::Gray => 1,
            RenderMode::SubpixelRgb => 4,
        };

        Self {
            data: vec![0u8; (width * height) as usize * bytes_per_pixel],
            width,
            height,
            rows: Vec::new(),
            y_cursor: 0,
            bytes_per_pixel,
            dirty: false,
        }
    }

    /// Try to allocate space for a glyph using shelf-first-fit algorithm
    fn allocate(&mut self, glyph_width: u32, glyph_height: u32) -> Option<(u32, u32)> {
        let width = glyph_width.max(MIN_GLYPH_SIZE);
        let height = glyph_height.max(MIN_GLYPH_SIZE);

        // Add 1 pixel padding to avoid bleeding
        let padded_width = width + 1;
        let padded_height = height + 1;

        // Try to find an existing row that fits
        for row in &mut self.rows {
            if row.height >= padded_height && row.x_cursor + padded_width <= self.width {
                let x = row.x_cursor;
                let y = row.y;
                row.x_cursor += padded_width;
                return Some((x, y));
            }
        }

        // Create a new row if there's space
        if self.y_cursor + padded_height <= self.height {
            let y = self.y_cursor;
            self.rows.push(AtlasRow {
                y,
                height: padded_height,
                x_cursor: padded_width,
            });
            self.y_cursor += padded_height;
            return Some((0, y));
        }

        None
    }

    /// Copy glyph bitmap data into the atlas
    fn copy_glyph(&mut self, x: u32, y: u32, bitmap: &GlyphBitmap) {
        let src_bpp = match bitmap.format {
            RenderMode::Mono | RenderMode::Gray => 1,
            RenderMode::SubpixelRgb => 3,
        };

        for row in 0..bitmap.height {
            let src_start = row as usize * bitmap.pitch;
            let src_end = src_start + (bitmap.width as usize * src_bpp);
            let src_row = if src_end <= bitmap.data.len() {
                &bitmap.data[src_start..src_end]
            } else {
                continue;
            };

            let dst_y = y + row;
            for col in 0..bitmap.width {
                let dst_x = x + col;
                let dst_idx = (dst_y * self.width + dst_x) as usize * self.bytes_per_pixel;

                if dst_idx + self.bytes_per_pixel > self.data.len() {
                    continue;
                }

                match (bitmap.format, self.bytes_per_pixel) {
                    (RenderMode::Gray, 1) | (RenderMode::Mono, 1) => {
                        if let Some(&pixel) = src_row.get(col as usize) {
                            self.data[dst_idx] = pixel;
                        }
                    }
                    (RenderMode::SubpixelRgb, 4) => {
                        let src_idx = col as usize * 3;
                        if src_idx + 2 < src_row.len() {
                            self.data[dst_idx] = src_row[src_idx]; // R
                            self.data[dst_idx + 1] = src_row[src_idx + 1]; // G
                            self.data[dst_idx + 2] = src_row[src_idx + 2]; // B
                            self.data[dst_idx + 3] = 255; // A
                        }
                    }
                    _ => {
                        // Format conversion: expand gray to RGBA
                        if let Some(&pixel) = src_row.get(col as usize) {
                            self.data[dst_idx] = pixel;
                            if self.bytes_per_pixel > 1 {
                                self.data[dst_idx + 1] = pixel;
                                self.data[dst_idx + 2] = pixel;
                                self.data[dst_idx + 3] = pixel;
                            }
                        }
                    }
                }
            }
        }
        self.dirty = true;
    }

    /// Get raw atlas data for GPU upload
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Check if atlas needs GPU upload
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark as uploaded to GPU
    #[inline]
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get memory usage in bytes
    #[inline]
    pub fn memory_bytes(&self) -> usize {
        self.data.len()
    }
}

/// Thread-safe GPU glyph cache with texture atlas
pub struct GpuGlyphCache {
    /// Configuration
    config: GpuCacheConfig,
    /// Atlases per render mode (Gray/Mono share, SubpixelRgb separate)
    gray_atlases: RwLock<Vec<TextureAtlas>>,
    subpixel_atlases: RwLock<Vec<TextureAtlas>>,
    /// LRU cache for glyph lookups
    cache: RwLock<LruCache<GpuCacheKey, CachedGlyph>>,
    /// Statistics (atomic for lock-free updates)
    stats_hits: AtomicU64,
    stats_misses: AtomicU64,
    stats_uploads: AtomicU64,
    stats_evictions: AtomicU64,
}

impl GpuGlyphCache {
    /// Create a new GPU glyph cache with default configuration
    pub fn new() -> Self {
        Self::with_config(GpuCacheConfig::default())
    }

    /// Create a new GPU glyph cache with custom configuration
    pub fn with_config(config: GpuCacheConfig) -> Self {
        Self {
            gray_atlases: RwLock::new(Vec::new()),
            subpixel_atlases: RwLock::new(Vec::new()),
            cache: RwLock::new(LruCache::new(
                NonZeroUsize::new(config.max_cached_glyphs).unwrap(),
            )),
            stats_hits: AtomicU64::new(0),
            stats_misses: AtomicU64::new(0),
            stats_uploads: AtomicU64::new(0),
            stats_evictions: AtomicU64::new(0),
            config,
        }
    }

    /// Get a cached glyph by key
    #[inline]
    pub fn get(&self, key: &GpuCacheKey) -> Option<CachedGlyph> {
        let mut cache = self.cache.write();
        if let Some(glyph) = cache.get(key) {
            self.stats_hits.fetch_add(1, Ordering::Relaxed);
            Some(glyph.clone())
        } else {
            self.stats_misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert a glyph bitmap into the cache
    ///
    /// Returns the cached glyph entry with atlas location
    pub fn insert(&self, key: GpuCacheKey, bitmap: &GlyphBitmap) -> Option<CachedGlyph> {
        let render_mode = key.render_mode;
        let is_subpixel = matches!(render_mode, RenderMode::SubpixelRgb);

        // Get the appropriate atlas list
        let region = if is_subpixel {
            self.allocate_in_atlases(&mut self.subpixel_atlases.write(), bitmap, render_mode)
        } else {
            self.allocate_in_atlases(&mut self.gray_atlases.write(), bitmap, render_mode)
        }?;

        let cached = CachedGlyph {
            region,
            left: bitmap.left,
            top: bitmap.top,
            render_mode: bitmap.format,
        };

        // Insert into LRU cache
        let mut cache = self.cache.write();
        if cache.push(key, cached.clone()).is_some() {
            self.stats_evictions.fetch_add(1, Ordering::Relaxed);
        }
        self.stats_uploads.fetch_add(1, Ordering::Relaxed);

        Some(cached)
    }

    /// Allocate space in atlas list, creating new atlas if needed
    fn allocate_in_atlases(
        &self,
        atlases: &mut Vec<TextureAtlas>,
        bitmap: &GlyphBitmap,
        render_mode: RenderMode,
    ) -> Option<AtlasRegion> {
        // Try existing atlases
        for (idx, atlas) in atlases.iter_mut().enumerate() {
            if let Some((x, y)) = atlas.allocate(bitmap.width, bitmap.height) {
                atlas.copy_glyph(x, y, bitmap);
                return Some(AtlasRegion {
                    x,
                    y,
                    width: bitmap.width,
                    height: bitmap.height,
                    atlas_index: idx,
                });
            }
        }

        // Create new atlas if under limit
        if atlases.len() < self.config.max_atlases {
            let mut new_atlas = TextureAtlas::new(
                self.config.atlas_width,
                self.config.atlas_height,
                render_mode,
            );

            if let Some((x, y)) = new_atlas.allocate(bitmap.width, bitmap.height) {
                new_atlas.copy_glyph(x, y, bitmap);
                let atlas_index = atlases.len();
                atlases.push(new_atlas);
                return Some(AtlasRegion {
                    x,
                    y,
                    width: bitmap.width,
                    height: bitmap.height,
                    atlas_index,
                });
            }
        }

        None
    }

    /// Get or insert a glyph
    ///
    /// Returns existing cached glyph or inserts the provided bitmap
    #[inline]
    pub fn get_or_insert(&self, key: GpuCacheKey, bitmap: &GlyphBitmap) -> Option<CachedGlyph> {
        // Fast path: check if already cached
        if let Some(cached) = self.get(&key) {
            return Some(cached);
        }

        // Slow path: insert into cache
        self.insert(key, bitmap)
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.write().clear();
        self.gray_atlases.write().clear();
        self.subpixel_atlases.write().clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> GpuCacheStats {
        let hits = self.stats_hits.load(Ordering::Relaxed);
        let misses = self.stats_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        let gray_atlases = self.gray_atlases.read();
        let subpixel_atlases = self.subpixel_atlases.read();
        let active_atlases = gray_atlases.len() + subpixel_atlases.len();
        let atlas_memory_bytes: usize =
            gray_atlases.iter().map(|a| a.memory_bytes()).sum::<usize>()
                + subpixel_atlases
                    .iter()
                    .map(|a| a.memory_bytes())
                    .sum::<usize>();

        GpuCacheStats {
            hits,
            misses,
            uploads: self.stats_uploads.load(Ordering::Relaxed),
            evictions: self.stats_evictions.load(Ordering::Relaxed),
            cached_glyphs: self.cache.read().len(),
            active_atlases,
            atlas_memory_bytes,
            hit_rate,
        }
    }

    /// Get dirty atlases that need GPU upload
    ///
    /// Returns list of (atlas_index, is_subpixel, data_slice)
    pub fn get_dirty_atlases(&self) -> Vec<(usize, bool, Vec<u8>)> {
        let mut dirty = Vec::new();

        for (idx, atlas) in self.gray_atlases.read().iter().enumerate() {
            if atlas.is_dirty() {
                dirty.push((idx, false, atlas.data().to_vec()));
            }
        }

        for (idx, atlas) in self.subpixel_atlases.read().iter().enumerate() {
            if atlas.is_dirty() {
                dirty.push((idx, true, atlas.data().to_vec()));
            }
        }

        dirty
    }

    /// Mark atlases as uploaded
    pub fn mark_atlases_clean(&self) {
        for atlas in self.gray_atlases.write().iter_mut() {
            atlas.mark_clean();
        }
        for atlas in self.subpixel_atlases.write().iter_mut() {
            atlas.mark_clean();
        }
    }
}

impl Default for GpuGlyphCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bitmap(width: u32, height: u32) -> GlyphBitmap {
        GlyphBitmap {
            width,
            height,
            left: 0,
            top: height as i32,
            pitch: width as usize,
            data: vec![128u8; (width * height) as usize],
            format: RenderMode::Gray,
        }
    }

    #[test]
    fn test_gpu_cache_creation() {
        let cache = GpuGlyphCache::new();
        let stats = cache.stats();
        assert_eq!(stats.cached_glyphs, 0);
        assert_eq!(stats.active_atlases, 0);
    }

    #[test]
    fn test_gpu_cache_insert_and_get() {
        let cache = GpuGlyphCache::new();
        let bitmap = create_test_bitmap(32, 32);
        let key = GpuCacheKey::new(GlyphId(65), 16.0, RenderMode::Gray);

        // Insert
        let cached = cache.insert(key.clone(), &bitmap);
        assert!(cached.is_some());

        // Get
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().region.width, 32);
    }

    #[test]
    fn test_gpu_cache_statistics() {
        let cache = GpuGlyphCache::new();
        let bitmap = create_test_bitmap(16, 16);
        let key = GpuCacheKey::new(GlyphId(65), 12.0, RenderMode::Gray);

        // Miss
        let _ = cache.get(&key);
        let stats = cache.stats();
        assert_eq!(stats.misses, 1);

        // Insert
        cache.insert(key.clone(), &bitmap);

        // Hit
        let _ = cache.get(&key);
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.uploads, 1);
    }

    #[test]
    fn test_atlas_region_uv_coords() {
        let region = AtlasRegion {
            x: 100,
            y: 200,
            width: 50,
            height: 30,
            atlas_index: 0,
        };

        let (u0, v0, u1, v1) = region.uv_coords(1000, 1000);
        assert!((u0 - 0.1).abs() < 0.001);
        assert!((v0 - 0.2).abs() < 0.001);
        assert!((u1 - 0.15).abs() < 0.001);
        assert!((v1 - 0.23).abs() < 0.001);
    }

    #[test]
    fn test_multiple_atlases_creation() {
        let config = GpuCacheConfig {
            atlas_width: 64,
            atlas_height: 64,
            max_atlases: 4,
            max_cached_glyphs: 100,
            enable_statistics: true,
        };
        let cache = GpuGlyphCache::with_config(config);

        // Insert many glyphs to force multiple atlases
        for i in 0..20 {
            let bitmap = create_test_bitmap(20, 20);
            let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
            cache.insert(key, &bitmap);
        }

        let stats = cache.stats();
        assert!(stats.active_atlases > 1);
    }
}
