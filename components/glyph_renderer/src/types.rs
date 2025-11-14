//! Common types for glyph_renderer

use std::fmt;

// Temporary stubs for types from dependencies (font_types, font_parser)
// These will be replaced with actual imports once dependencies are implemented

/// Glyph ID type (stub - will come from font_types)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GlyphId(pub u16);

/// Render mode enumeration (stub - will come from font_types)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RenderMode {
    /// Monochrome (1-bit) rendering
    Mono,
    /// Grayscale (8-bit) rendering with anti-aliasing
    Gray,
    /// Subpixel RGB rendering
    SubpixelRgb,
}

/// OpenType font structure (stub - will come from font_parser)
#[derive(Debug, Clone)]
pub struct OpenTypeFont {
    // Font data (actual TrueType/OpenType font bytes)
    pub(crate) data: Vec<u8>,
    // Face index (for TTC collections)
    pub(crate) face_index: isize,
}

impl OpenTypeFont {
    /// Create a font from raw font data
    ///
    /// # Arguments
    /// * `data` - Raw TrueType or OpenType font data
    /// * `face_index` - Face index (0 for single fonts, varies for TTC collections)
    pub fn from_data(data: Vec<u8>, face_index: isize) -> Self {
        Self { data, face_index }
    }

    /// Create a temporary stub font for testing
    /// Note: This is a test helper stub. Will be replaced when font_parser is implemented.
    pub fn new_stub() -> Self {
        Self {
            data: Vec::new(),
            face_index: 0,
        }
    }

    /// Check if this font has any data
    pub fn has_data(&self) -> bool {
        !self.data.is_empty()
    }
}

/// Glyph bitmap data
#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    /// Bitmap width in pixels
    pub width: u32,
    /// Bitmap height in pixels
    pub height: u32,
    /// Left bearing (horizontal offset from origin)
    pub left: i32,
    /// Top bearing (vertical offset from baseline)
    pub top: i32,
    /// Pitch (bytes per row)
    pub pitch: usize,
    /// Pixel data
    pub data: Vec<u8>,
    /// Render mode used
    pub format: RenderMode,
}

/// Glyph outline (vector representation)
#[derive(Debug, Clone)]
pub struct GlyphOutline {
    /// Outline contours
    pub contours: Vec<Contour>,
    /// Bounding box
    pub bounds: BoundingBox,
}

/// Outline contour
#[derive(Debug, Clone)]
pub struct Contour {
    /// Points in the contour
    pub points: Vec<Point>,
    /// Whether the contour is closed
    pub closed: bool,
}

/// 2D point
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

/// Bounding box
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    /// Minimum X
    pub min_x: f32,
    /// Minimum Y
    pub min_y: f32,
    /// Maximum X
    pub max_x: f32,
    /// Maximum Y
    pub max_y: f32,
}

/// Glyph cache statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Total memory used by cache in bytes
    pub memory_bytes: usize,
}

impl CacheStats {
    /// Create new empty cache stats
    pub fn new() -> Self {
        Self {
            entries: 0,
            hits: 0,
            misses: 0,
            memory_bytes: 0,
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Glyph rendering errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// Glyph not found in font
    GlyphNotFound(GlyphId),
    /// Rasterization failed with error message
    RasterizationFailed(String),
    /// Out of memory during rendering
    OutOfMemory,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::GlyphNotFound(id) => write!(f, "Glyph not found: {:?}", id),
            RenderError::RasterizationFailed(msg) => write!(f, "Rasterization failed: {}", msg),
            RenderError::OutOfMemory => write!(f, "Out of memory during rendering"),
        }
    }
}

impl std::error::Error for RenderError {}
