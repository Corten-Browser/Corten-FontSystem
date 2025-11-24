//! Pure Rust glyph rasterizer implementation
//!
//! This module provides a native Rust glyph rasterizer that can work alongside
//! or replace FreeType. It implements:
//! - Path-based outline rasterization
//! - Anti-aliased grayscale rendering
//! - Basic hinting support
//!
//! This is infrastructure for gradual migration from FreeType.

use crate::types::{BoundingBox, Contour, GlyphBitmap, GlyphOutline, Point, RenderMode};

/// Hinting mode for glyph rasterization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HintingMode {
    /// No hinting (native outlines)
    None,
    /// Light hinting (slight grid-fitting)
    Light,
    /// Full hinting (maximum grid-fitting)
    Full,
    /// Auto-hinting (computed hinting)
    Auto,
}

impl Default for HintingMode {
    fn default() -> Self {
        HintingMode::Light
    }
}

/// Error type for rasterization operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RasterizeError {
    /// Invalid outline data
    InvalidOutline(String),
    /// Outline has no contours
    EmptyOutline,
    /// Rasterization failed
    RasterizationFailed(String),
    /// Size is invalid
    InvalidSize,
}

impl std::fmt::Display for RasterizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RasterizeError::InvalidOutline(msg) => write!(f, "Invalid outline: {}", msg),
            RasterizeError::EmptyOutline => write!(f, "Outline has no contours"),
            RasterizeError::RasterizationFailed(msg) => write!(f, "Rasterization failed: {}", msg),
            RasterizeError::InvalidSize => write!(f, "Invalid size for rasterization"),
        }
    }
}

impl std::error::Error for RasterizeError {}

/// Native Rust glyph rasterizer
///
/// Provides outline-based glyph rasterization using scanline algorithm.
/// Supports anti-aliased grayscale and monochrome rendering.
pub struct NativeRasterizer {
    /// Enable subpixel positioning
    subpixel_positioning: bool,
    /// Hinting mode
    hinting_mode: HintingMode,
    /// Anti-aliasing sample count (1 = no AA, 4 = 4x AA, 16 = 16x AA)
    aa_samples: u8,
}

/// Edge for scanline rasterization
#[derive(Debug, Clone)]
struct Edge {
    /// Y coordinate at the top of the edge
    y_top: f32,
    /// Y coordinate at the bottom of the edge
    y_bottom: f32,
    /// X coordinate at current scanline
    x_current: f32,
    /// Inverse slope (dx/dy)
    dx_dy: f32,
    /// Winding direction (+1 or -1)
    winding: i32,
}

impl Edge {
    /// Create a new edge from two points
    fn from_points(p0: Point, p1: Point) -> Option<Self> {
        // Skip horizontal edges
        if (p1.y - p0.y).abs() < 0.001 {
            return None;
        }

        let (top, bottom) = if p0.y < p1.y { (p0, p1) } else { (p1, p0) };

        let dy = bottom.y - top.y;
        let dx = bottom.x - top.x;
        let dx_dy = dx / dy;

        // Winding: positive if going up (top.y < bottom.y means original was p0->p1)
        let winding = if p0.y < p1.y { 1 } else { -1 };

        Some(Edge {
            y_top: top.y,
            y_bottom: bottom.y,
            x_current: top.x,
            dx_dy,
            winding,
        })
    }

    /// Update x position for a new scanline
    fn update_x(&mut self, y: f32) {
        self.x_current = self.x_at_y(y);
    }

    /// Get x coordinate at a given y
    fn x_at_y(&self, y: f32) -> f32 {
        let dy = y - self.y_top;
        self.x_current + dy * self.dx_dy
    }

    /// Check if this edge is active at the given y coordinate
    fn is_active_at(&self, y: f32) -> bool {
        y >= self.y_top && y < self.y_bottom
    }
}

impl NativeRasterizer {
    /// Create a new native rasterizer with default settings
    pub fn new() -> Self {
        Self {
            subpixel_positioning: false,
            hinting_mode: HintingMode::default(),
            aa_samples: 4,
        }
    }

    /// Create a new rasterizer with custom settings
    #[allow(dead_code)]
    pub fn with_settings(subpixel_positioning: bool, hinting_mode: HintingMode) -> Self {
        Self {
            subpixel_positioning,
            hinting_mode,
            aa_samples: 4,
        }
    }

    /// Set subpixel positioning
    #[allow(dead_code)]
    pub fn set_subpixel_positioning(&mut self, enabled: bool) {
        self.subpixel_positioning = enabled;
    }

    /// Set hinting mode
    #[allow(dead_code)]
    pub fn set_hinting_mode(&mut self, mode: HintingMode) {
        self.hinting_mode = mode;
    }

    /// Set anti-aliasing sample count
    #[allow(dead_code)]
    pub fn set_aa_samples(&mut self, samples: u8) {
        self.aa_samples = samples.max(1).min(16);
    }

    /// Rasterize a glyph outline to a bitmap
    ///
    /// # Arguments
    /// * `outline` - The glyph outline to rasterize
    /// * `size` - Font size in pixels
    /// * `mode` - Render mode (mono, gray, or subpixel)
    ///
    /// # Returns
    /// Result containing the rasterized bitmap or error
    #[allow(dead_code)]
    pub fn rasterize(
        &self,
        outline: &GlyphOutline,
        size: f32,
        mode: RenderMode,
    ) -> Result<GlyphBitmap, RasterizeError> {
        if size <= 0.0 {
            return Err(RasterizeError::InvalidSize);
        }

        if outline.contours.is_empty() {
            return Err(RasterizeError::EmptyOutline);
        }

        // Scale the outline to the desired size
        let scaled_outline = self.scale_outline(outline, size);

        // Calculate bitmap dimensions
        let (width, height, left, top) = self.calculate_bitmap_dimensions(&scaled_outline);

        if width == 0 || height == 0 {
            return Ok(self.create_empty_bitmap(mode));
        }

        // Rasterize based on mode
        let data = match mode {
            RenderMode::Mono => self.rasterize_mono(&scaled_outline, width, height, left, top)?,
            RenderMode::Gray => self.rasterize_gray(&scaled_outline, width, height, left, top)?,
            RenderMode::SubpixelRgb => {
                self.rasterize_subpixel(&scaled_outline, width, height, left, top)?
            }
        };

        let pitch = match mode {
            RenderMode::Mono => (width + 7) / 8,
            RenderMode::Gray => width,
            RenderMode::SubpixelRgb => width * 3,
        } as usize;

        Ok(GlyphBitmap {
            width: width as u32,
            height: height as u32,
            left,
            top,
            pitch,
            data,
            format: mode,
        })
    }

    /// Scale outline to target size
    fn scale_outline(&self, outline: &GlyphOutline, size: f32) -> GlyphOutline {
        // Assume outline is in font units (typically 1000 or 2048 units per em)
        // Scale to pixel size
        let scale = size / 1000.0; // Assuming 1000 units per em

        let scaled_contours: Vec<Contour> = outline
            .contours
            .iter()
            .map(|contour| Contour {
                points: contour
                    .points
                    .iter()
                    .map(|p| Point {
                        x: p.x * scale,
                        y: p.y * scale,
                    })
                    .collect(),
                closed: contour.closed,
            })
            .collect();

        let scaled_bounds = BoundingBox {
            min_x: outline.bounds.min_x * scale,
            min_y: outline.bounds.min_y * scale,
            max_x: outline.bounds.max_x * scale,
            max_y: outline.bounds.max_y * scale,
        };

        GlyphOutline {
            contours: scaled_contours,
            bounds: scaled_bounds,
        }
    }

    /// Calculate bitmap dimensions from outline bounds
    fn calculate_bitmap_dimensions(&self, outline: &GlyphOutline) -> (u32, u32, i32, i32) {
        let bounds = &outline.bounds;

        // Add padding for anti-aliasing
        let padding = 1.0;

        let left = (bounds.min_x - padding).floor() as i32;
        let top = (bounds.max_y + padding).ceil() as i32;
        let right = (bounds.max_x + padding).ceil() as i32;
        let bottom = (bounds.min_y - padding).floor() as i32;

        let width = (right - left).max(0) as u32;
        let height = (top - bottom).max(0) as u32;

        (width, height, left, top)
    }

    /// Create an empty bitmap
    fn create_empty_bitmap(&self, mode: RenderMode) -> GlyphBitmap {
        GlyphBitmap {
            width: 0,
            height: 0,
            left: 0,
            top: 0,
            pitch: 0,
            data: Vec::new(),
            format: mode,
        }
    }

    /// Rasterize to monochrome bitmap (1-bit)
    fn rasterize_mono(
        &self,
        outline: &GlyphOutline,
        width: u32,
        height: u32,
        left: i32,
        top: i32,
    ) -> Result<Vec<u8>, RasterizeError> {
        // First rasterize as grayscale, then threshold
        let gray = self.rasterize_gray(outline, width, height, left, top)?;

        let pitch = ((width + 7) / 8) as usize;
        let mut mono = vec![0u8; pitch * height as usize];

        for y in 0..height as usize {
            for x in 0..width as usize {
                let gray_value = gray[y * width as usize + x];
                if gray_value >= 128 {
                    let byte_idx = y * pitch + x / 8;
                    let bit_idx = 7 - (x % 8);
                    mono[byte_idx] |= 1 << bit_idx;
                }
            }
        }

        Ok(mono)
    }

    /// Rasterize to grayscale bitmap (8-bit) with anti-aliasing
    fn rasterize_gray(
        &self,
        outline: &GlyphOutline,
        width: u32,
        height: u32,
        left: i32,
        top: i32,
    ) -> Result<Vec<u8>, RasterizeError> {
        let mut bitmap = vec![0u8; (width * height) as usize];

        // Build edge list from outline
        let edges = self.build_edge_list(outline, left as f32, top as f32)?;

        if edges.is_empty() {
            return Ok(bitmap);
        }

        // Scanline rasterization with anti-aliasing
        let aa_scale = self.aa_samples as f32;
        let inv_aa_samples = 1.0 / (aa_scale * aa_scale);

        for y in 0..height {
            let y_pixel = top - y as i32;

            // Sample multiple times for anti-aliasing
            let mut row_coverage = vec![0.0f32; width as usize];

            for sub_y in 0..self.aa_samples {
                let y_sample = y_pixel as f32 - (sub_y as f32 + 0.5) / aa_scale;

                // Get active edges at this y
                let mut active_edges: Vec<&Edge> =
                    edges.iter().filter(|e| e.is_active_at(y_sample)).collect();

                // Sort by x coordinate
                active_edges.sort_by(|a, b| {
                    a.x_at_y(y_sample)
                        .partial_cmp(&b.x_at_y(y_sample))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // Fill spans using non-zero winding rule
                let mut winding = 0;
                let mut prev_x = left as f32;

                for edge in &active_edges {
                    let x = edge.x_at_y(y_sample);

                    if winding != 0 {
                        // Fill from prev_x to x
                        let x_start = (prev_x - left as f32).max(0.0);
                        let x_end = (x - left as f32).min(width as f32);

                        for sub_x in 0..self.aa_samples {
                            let x_sample = x_start + (sub_x as f32 + 0.5) / aa_scale;
                            if x_sample >= x_start && x_sample < x_end {
                                let pixel_x = x_sample.floor() as usize;
                                if pixel_x < width as usize {
                                    row_coverage[pixel_x] += inv_aa_samples;
                                }
                            }
                        }

                        // Fill complete pixels
                        let start_pixel = (x_start.ceil() as usize).min(width as usize);
                        let end_pixel = (x_end.floor() as usize).min(width as usize);
                        for px in start_pixel..end_pixel {
                            row_coverage[px] += 1.0 / self.aa_samples as f32;
                        }
                    }

                    winding += edge.winding;
                    prev_x = x;
                }
            }

            // Convert coverage to pixel values
            for (x, &coverage) in row_coverage.iter().enumerate() {
                let clamped = coverage.min(1.0).max(0.0);
                let value = (clamped * 255.0) as u8;
                bitmap[y as usize * width as usize + x] = value;
            }
        }

        Ok(bitmap)
    }

    /// Rasterize to subpixel RGB bitmap
    fn rasterize_subpixel(
        &self,
        outline: &GlyphOutline,
        width: u32,
        height: u32,
        left: i32,
        top: i32,
    ) -> Result<Vec<u8>, RasterizeError> {
        // Render at 3x horizontal resolution
        let subpixel_width = width * 3;
        let gray = self.rasterize_gray(outline, subpixel_width, height, left * 3, top)?;

        // Convert to RGB
        let mut rgb = vec![0u8; (width * height * 3) as usize];

        for y in 0..height as usize {
            for x in 0..width as usize {
                let r = gray[y * subpixel_width as usize + x * 3];
                let g = gray[y * subpixel_width as usize + x * 3 + 1];
                let b = gray[y * subpixel_width as usize + x * 3 + 2];

                let idx = (y * width as usize + x) * 3;
                rgb[idx] = r;
                rgb[idx + 1] = g;
                rgb[idx + 2] = b;
            }
        }

        Ok(rgb)
    }

    /// Build edge list from outline contours
    fn build_edge_list(
        &self,
        outline: &GlyphOutline,
        _left: f32,
        _top: f32,
    ) -> Result<Vec<Edge>, RasterizeError> {
        let mut edges = Vec::new();

        for contour in &outline.contours {
            if contour.points.len() < 2 {
                continue;
            }

            // Create edges between consecutive points
            for i in 0..contour.points.len() {
                let p0 = contour.points[i];
                let p1 = if i + 1 < contour.points.len() {
                    contour.points[i + 1]
                } else if contour.closed {
                    contour.points[0]
                } else {
                    continue;
                };

                if let Some(edge) = Edge::from_points(p0, p1) {
                    edges.push(edge);
                }
            }
        }

        Ok(edges)
    }

    /// Apply hinting to outline (basic implementation)
    #[allow(dead_code)]
    fn apply_hinting(&self, outline: &mut GlyphOutline) {
        match self.hinting_mode {
            HintingMode::None => {}
            HintingMode::Light => {
                // Slight grid-fitting: round y coordinates to half-pixel boundaries
                for contour in &mut outline.contours {
                    for point in &mut contour.points {
                        point.y = (point.y * 2.0).round() / 2.0;
                    }
                }
            }
            HintingMode::Full | HintingMode::Auto => {
                // Full grid-fitting: round to pixel boundaries
                for contour in &mut outline.contours {
                    for point in &mut contour.points {
                        point.x = point.x.round();
                        point.y = point.y.round();
                    }
                }
            }
        }
    }
}

impl Default for NativeRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_outline() -> GlyphOutline {
        // Simple square outline (100x100 units)
        GlyphOutline {
            contours: vec![Contour {
                points: vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 100.0, y: 0.0 },
                    Point { x: 100.0, y: 100.0 },
                    Point { x: 0.0, y: 100.0 },
                ],
                closed: true,
            }],
            bounds: BoundingBox {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 100.0,
                max_y: 100.0,
            },
        }
    }

    fn create_empty_outline() -> GlyphOutline {
        GlyphOutline {
            contours: vec![],
            bounds: BoundingBox {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 0.0,
                max_y: 0.0,
            },
        }
    }

    #[test]
    fn test_rasterizer_creation() {
        let rasterizer = NativeRasterizer::new();
        assert!(!rasterizer.subpixel_positioning);
        assert_eq!(rasterizer.hinting_mode, HintingMode::Light);
        assert_eq!(rasterizer.aa_samples, 4);
    }

    #[test]
    fn test_rasterizer_with_settings() {
        let rasterizer = NativeRasterizer::with_settings(true, HintingMode::Full);
        assert!(rasterizer.subpixel_positioning);
        assert_eq!(rasterizer.hinting_mode, HintingMode::Full);
    }

    #[test]
    fn test_empty_outline_error() {
        let rasterizer = NativeRasterizer::new();
        let outline = create_empty_outline();

        let result = rasterizer.rasterize(&outline, 12.0, RenderMode::Gray);
        assert!(matches!(result, Err(RasterizeError::EmptyOutline)));
    }

    #[test]
    fn test_invalid_size_error() {
        let rasterizer = NativeRasterizer::new();
        let outline = create_test_outline();

        let result = rasterizer.rasterize(&outline, 0.0, RenderMode::Gray);
        assert!(matches!(result, Err(RasterizeError::InvalidSize)));

        let result = rasterizer.rasterize(&outline, -10.0, RenderMode::Gray);
        assert!(matches!(result, Err(RasterizeError::InvalidSize)));
    }

    #[test]
    fn test_basic_rasterization() {
        let rasterizer = NativeRasterizer::new();
        let outline = create_test_outline();

        let result = rasterizer.rasterize(&outline, 10.0, RenderMode::Gray);
        assert!(result.is_ok());

        let bitmap = result.unwrap();
        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
        assert_eq!(bitmap.format, RenderMode::Gray);
    }

    #[test]
    fn test_mono_rasterization() {
        let rasterizer = NativeRasterizer::new();
        let outline = create_test_outline();

        let result = rasterizer.rasterize(&outline, 10.0, RenderMode::Mono);
        assert!(result.is_ok());

        let bitmap = result.unwrap();
        assert_eq!(bitmap.format, RenderMode::Mono);
        // Mono pitch should be (width + 7) / 8 bytes per row
        assert_eq!(bitmap.pitch, ((bitmap.width + 7) / 8) as usize);
    }

    #[test]
    fn test_edge_creation() {
        // Vertical edge
        let p0 = Point { x: 10.0, y: 0.0 };
        let p1 = Point { x: 10.0, y: 100.0 };
        let edge = Edge::from_points(p0, p1);
        assert!(edge.is_some());

        let edge = edge.unwrap();
        assert_eq!(edge.y_top, 0.0);
        assert_eq!(edge.y_bottom, 100.0);
        assert_eq!(edge.dx_dy, 0.0);
    }

    #[test]
    fn test_horizontal_edge_skipped() {
        // Horizontal edges should be skipped
        let p0 = Point { x: 0.0, y: 10.0 };
        let p1 = Point { x: 100.0, y: 10.0 };
        let edge = Edge::from_points(p0, p1);
        assert!(edge.is_none());
    }

    #[test]
    fn test_edge_is_active_at() {
        let p0 = Point { x: 0.0, y: 10.0 };
        let p1 = Point { x: 50.0, y: 60.0 };
        let edge = Edge::from_points(p0, p1).unwrap();

        assert!(edge.is_active_at(20.0));
        assert!(edge.is_active_at(10.0)); // At top
        assert!(!edge.is_active_at(60.0)); // At bottom (exclusive)
        assert!(!edge.is_active_at(5.0)); // Below top
        assert!(!edge.is_active_at(70.0)); // Above bottom
    }

    #[test]
    fn test_hinting_mode_default() {
        assert_eq!(HintingMode::default(), HintingMode::Light);
    }

    #[test]
    fn test_set_aa_samples_bounds() {
        let mut rasterizer = NativeRasterizer::new();

        rasterizer.set_aa_samples(0);
        assert_eq!(rasterizer.aa_samples, 1); // Clamped to minimum

        rasterizer.set_aa_samples(100);
        assert_eq!(rasterizer.aa_samples, 16); // Clamped to maximum

        rasterizer.set_aa_samples(8);
        assert_eq!(rasterizer.aa_samples, 8);
    }

    #[test]
    fn test_scale_outline() {
        let rasterizer = NativeRasterizer::new();
        let outline = create_test_outline();

        // Scale by 10% (size 100 / 1000 em units = 0.1)
        let scaled = rasterizer.scale_outline(&outline, 100.0);

        // Original bounds were 0-100, scaled should be 0-10
        assert!((scaled.bounds.max_x - 10.0).abs() < 0.001);
        assert!((scaled.bounds.max_y - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_apply_light_hinting() {
        let rasterizer = NativeRasterizer::new();
        let mut outline = GlyphOutline {
            contours: vec![Contour {
                points: vec![Point { x: 10.3, y: 10.7 }],
                closed: false,
            }],
            bounds: BoundingBox {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 20.0,
                max_y: 20.0,
            },
        };

        rasterizer.apply_hinting(&mut outline);

        // Light hinting rounds Y to half-pixel
        let point = &outline.contours[0].points[0];
        assert!((point.y - 10.5).abs() < 0.001 || (point.y - 11.0).abs() < 0.001);
    }

    #[test]
    fn test_rasterize_error_display() {
        let error = RasterizeError::EmptyOutline;
        assert_eq!(format!("{}", error), "Outline has no contours");

        let error = RasterizeError::InvalidSize;
        assert_eq!(format!("{}", error), "Invalid size for rasterization");

        let error = RasterizeError::InvalidOutline("test".to_string());
        assert_eq!(format!("{}", error), "Invalid outline: test");
    }
}
