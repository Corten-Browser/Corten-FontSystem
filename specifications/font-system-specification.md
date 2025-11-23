# Font System Component Specification
## CortenBrowser Font Stack v1.0

### Executive Summary
The Font System component provides a complete text shaping and rendering pipeline for the CortenBrowser, handling font loading, text shaping, glyph rendering, and font fallback. Starting with FreeType + Harfbuzz bindings, the system will progressively migrate to a pure Rust implementation while maintaining compatibility with the Web Platform's font requirements.

### Component Metadata
- **Component ID**: `font-system`
- **Repository**: `cortenbrowser/font-system`
- **Language**: Rust
- **License**: MIT/Apache-2.0
- **Current Implementation**: FreeType2 + Harfbuzz via rust bindings
- **Target Implementation**: Pure Rust font stack
- **Estimated LOC**: 75,000-100,000
- **Development Time**: 3-4 months
- **Priority**: Phase 2 (Required for rendering)

## Architecture Overview

### System Architecture
```
┌─────────────────────────────────────────────────┐
│              Font System Component               │
├─────────────────────────────────────────────────┤
│                  Public API Layer                │
├─────────────────────────────────────────────────┤
│   Font         │   Text      │   Glyph          │
│   Registry     │   Shaper    │   Renderer       │
├────────────────┼─────────────┼──────────────────┤
│   Font         │   Layout    │   Rasterizer     │
│   Parser       │   Engine    │                  │
├────────────────┴─────────────┴──────────────────┤
│           Platform Integration Layer             │
└─────────────────────────────────────────────────┘
```

### Module Structure
```rust
font-system/
├── src/
│   ├── lib.rs                 // Public API exports
│   ├── registry/              // Font discovery and management
│   │   ├── mod.rs
│   │   ├── system_fonts.rs    // Platform font discovery
│   │   ├── web_fonts.rs       // @font-face handling
│   │   ├── font_cache.rs      // Font caching system
│   │   └── font_matching.rs   // Font selection algorithm
│   ├── parser/                // Font file parsing
│   │   ├── mod.rs
│   │   ├── opentype.rs        // OpenType/TrueType parser
│   │   ├── woff.rs            // WOFF/WOFF2 support
│   │   ├── variable_fonts.rs  // Variable font support
│   │   └── color_fonts.rs     // COLR/CBDT/sbix support
│   ├── shaper/                // Text shaping engine
│   │   ├── mod.rs
│   │   ├── harfbuzz_shaper.rs // Initial Harfbuzz wrapper
│   │   ├── rust_shaper.rs     // Pure Rust shaper
│   │   ├── bidi.rs            // Bidirectional text
│   │   ├── line_breaking.rs   // Line break algorithm
│   │   └── features.rs        // OpenType features
│   ├── renderer/              // Glyph rendering
│   │   ├── mod.rs
│   │   ├── rasterizer.rs      // Glyph rasterization
│   │   ├── hinting.rs         // TrueType hinting
│   │   ├── subpixel.rs        // Subpixel rendering
│   │   └── gpu_cache.rs       // GPU glyph caching
│   ├── layout/                // Text layout engine
│   │   ├── mod.rs
│   │   ├── paragraph.rs       // Paragraph layout
│   │   ├── line_layout.rs     // Line layout
│   │   ├── justification.rs   // Text justification
│   │   └── vertical.rs        // Vertical text support
│   ├── platform/              // Platform-specific code
│   │   ├── mod.rs
│   │   ├── linux.rs           // FontConfig integration
│   │   ├── windows.rs         // DirectWrite integration
│   │   └── macos.rs           // CoreText integration
│   └── types.rs               // Common types and traits
├── tests/
│   ├── unit/
│   ├── integration/
│   └── fixtures/              // Test fonts
├── benches/
│   └── shaping_bench.rs
├── Cargo.toml
└── README.md
```

## Public API Specification

### Core Types

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Font weight values (100-900)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Regular = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(f32), // Oblique angle in degrees
}

/// Font stretch values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FontStretch {
    UltraCondensed = 50,
    ExtraCondensed = 62,
    Condensed = 75,
    SemiCondensed = 87,
    Normal = 100,
    SemiExpanded = 112,
    Expanded = 125,
    ExtraExpanded = 150,
    UltraExpanded = 200,
}

/// Font descriptor for font selection
#[derive(Debug, Clone, PartialEq)]
pub struct FontDescriptor {
    pub family: Vec<String>,      // Font family names (fallback chain)
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub size: f32,                // Font size in pixels
}

/// Loaded font face
pub struct FontFace {
    id: FontId,
    family_name: String,
    postscript_name: String,
    weight: FontWeight,
    style: FontStyle,
    stretch: FontStretch,
    metrics: FontMetrics,
    data: Arc<Vec<u8>>,           // Font file data
}

/// Font metrics
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub units_per_em: u16,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub cap_height: f32,
    pub x_height: f32,
    pub underline_position: f32,
    pub underline_thickness: f32,
}

/// Glyph identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphId(pub u32);

/// Positioned glyph
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: GlyphId,
    pub font_id: FontId,
    pub position: Point,          // Baseline position
    pub advance: Vector,          // Advance to next glyph
    pub offset: Vector,           // Positioning offset
}

/// Shaped text run
#[derive(Debug, Clone)]
pub struct ShapedText {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
}

/// Text shaping options
#[derive(Debug, Clone)]
pub struct ShapingOptions {
    pub script: Script,
    pub language: Language,
    pub direction: Direction,
    pub features: HashMap<String, u32>, // OpenType features
    pub kerning: bool,
    pub ligatures: bool,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

/// Rasterization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Mono,                         // 1-bit monochrome
    Gray,                         // 8-bit grayscale
    SubpixelRgb,                  // Subpixel RGB
    SubpixelBgr,                  // Subpixel BGR
    SubpixelVrgb,                 // Vertical RGB
    SubpixelVbgr,                 // Vertical BGR
}

/// Rendered glyph bitmap
pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,
    pub left: i32,                // Bearing X
    pub top: i32,                 // Bearing Y
    pub pitch: usize,             // Bytes per row
    pub data: Vec<u8>,            // Pixel data
    pub format: RenderMode,
}
```

### Main API Interface

```rust
/// Main font system interface
pub trait FontSystem: Send + Sync {
    /// Initialize the font system
    fn new(config: FontSystemConfig) -> Result<Self, FontError>
    where
        Self: Sized;
    
    /// Load system fonts
    fn load_system_fonts(&mut self) -> Result<(), FontError>;
    
    /// Load a font from file
    fn load_font_file(&mut self, path: &Path) -> Result<FontId, FontError>;
    
    /// Load a font from memory
    fn load_font_data(&mut self, data: Vec<u8>) -> Result<FontId, FontError>;
    
    /// Find best matching font for descriptor
    fn match_font(&self, descriptor: &FontDescriptor) -> Option<FontId>;
    
    /// Shape text with a specific font
    fn shape_text(
        &self,
        text: &str,
        font_id: FontId,
        size: f32,
        options: &ShapingOptions,
    ) -> Result<ShapedText, FontError>;
    
    /// Shape text with font fallback
    fn shape_text_with_fallback(
        &self,
        text: &str,
        descriptor: &FontDescriptor,
        options: &ShapingOptions,
    ) -> Result<ShapedText, FontError>;
    
    /// Rasterize a glyph
    fn rasterize_glyph(
        &self,
        font_id: FontId,
        glyph_id: GlyphId,
        size: f32,
        mode: RenderMode,
    ) -> Result<GlyphBitmap, FontError>;
    
    /// Get font metrics
    fn get_font_metrics(&self, font_id: FontId, size: f32) -> Option<FontMetrics>;
    
    /// Get glyph outline (for vector rendering)
    fn get_glyph_outline(
        &self,
        font_id: FontId,
        glyph_id: GlyphId,
    ) -> Result<GlyphOutline, FontError>;
}
```

### Browser Integration Interface

```rust
/// Interface for browser component integration
impl BrowserComponent for FontSystem {
    fn initialize(&mut self, config: ComponentConfig) -> Result<(), ComponentError> {
        self.load_system_fonts()?;
        self.setup_font_cache(config.cache_size)?;
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), ComponentError> {
        self.clear_caches();
        Ok(())
    }
    
    fn handle_message(&mut self, msg: ComponentMessage) -> Result<ComponentResponse, ComponentError> {
        match msg {
            ComponentMessage::LoadWebFont { url, data } => {
                let font_id = self.load_font_data(data)?;
                Ok(ComponentResponse::FontLoaded { font_id })
            }
            ComponentMessage::ShapeText { text, descriptor, options } => {
                let shaped = self.shape_text_with_fallback(&text, &descriptor, &options)?;
                Ok(ComponentResponse::ShapedText { shaped })
            }
            ComponentMessage::RasterizeGlyph { font_id, glyph_id, size, mode } => {
                let bitmap = self.rasterize_glyph(font_id, glyph_id, size, mode)?;
                Ok(ComponentResponse::GlyphBitmap { bitmap })
            }
            _ => Err(ComponentError::UnsupportedMessage),
        }
    }
    
    fn health_check(&self) -> ComponentHealth {
        ComponentHealth {
            status: if self.font_count() > 0 { 
                HealthStatus::Healthy 
            } else { 
                HealthStatus::Degraded 
            },
            message: format!("{} fonts loaded", self.font_count()),
        }
    }
    
    fn get_metrics(&self) -> ComponentMetrics {
        ComponentMetrics {
            memory_usage: self.cache_size_bytes(),
            cpu_usage: 0.0,
            custom: hashmap! {
                "fonts_loaded" => self.font_count() as f64,
                "cache_hits" => self.cache_hit_rate(),
                "shaping_time_ms" => self.avg_shaping_time_ms(),
            },
        }
    }
}
```

## Implementation Strategy

### Phase 1: Harfbuzz Integration (Weeks 1-2)
Initial implementation using existing libraries:

```rust
// src/shaper/harfbuzz_shaper.rs
use harfbuzz_rs::{Face, Font, UnicodeBuffer, GlyphBuffer};

pub struct HarfbuzzShaper {
    faces: HashMap<FontId, Face<'static>>,
}

impl HarfbuzzShaper {
    pub fn shape(&self, text: &str, font_id: FontId, options: &ShapingOptions) -> ShapedText {
        let face = &self.faces[&font_id];
        let mut font = Font::new(face);
        
        let mut buffer = UnicodeBuffer::new();
        buffer.add_str(text);
        buffer.set_direction(convert_direction(options.direction));
        buffer.set_script(convert_script(options.script));
        buffer.set_language(convert_language(options.language));
        
        let output = font.shape(&buffer, &convert_features(&options.features));
        
        convert_to_shaped_text(output)
    }
}
```

### Phase 2: Font Parser Implementation (Weeks 3-4)
Pure Rust OpenType/TrueType parser:

```rust
// src/parser/opentype.rs
use byteorder::{BigEndian, ReadBytesExt};

pub struct OpenTypeFont {
    data: Vec<u8>,
    tables: HashMap<Tag, TableRecord>,
}

impl OpenTypeFont {
    pub fn parse(data: Vec<u8>) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(&data);
        
        // Read sfnt version
        let version = cursor.read_u32::<BigEndian>()?;
        if version != 0x00010000 && version != 0x4F54544F {
            return Err(ParseError::InvalidFormat);
        }
        
        let num_tables = cursor.read_u16::<BigEndian>()?;
        let search_range = cursor.read_u16::<BigEndian>()?;
        let entry_selector = cursor.read_u16::<BigEndian>()?;
        let range_shift = cursor.read_u16::<BigEndian>()?;
        
        // Parse table directory
        let mut tables = HashMap::new();
        for _ in 0..num_tables {
            let tag = Tag::from_bytes(cursor.read_u32::<BigEndian>()?);
            let checksum = cursor.read_u32::<BigEndian>()?;
            let offset = cursor.read_u32::<BigEndian>()?;
            let length = cursor.read_u32::<BigEndian>()?;
            
            tables.insert(tag, TableRecord {
                checksum,
                offset,
                length,
            });
        }
        
        Ok(OpenTypeFont { data, tables })
    }
    
    pub fn get_table(&self, tag: Tag) -> Option<&[u8]> {
        self.tables.get(&tag).map(|record| {
            &self.data[record.offset as usize..(record.offset + record.length) as usize]
        })
    }
}
```

### Phase 3: Text Shaping Engine (Weeks 5-8)
Pure Rust text shaping implementation:

```rust
// src/shaper/rust_shaper.rs
pub struct RustShaper {
    fonts: HashMap<FontId, Arc<OpenTypeFont>>,
}

impl RustShaper {
    pub fn shape(&self, text: &str, font_id: FontId, options: &ShapingOptions) -> ShapedText {
        let font = &self.fonts[&font_id];
        
        // 1. Unicode processing
        let unicode_props = analyze_unicode(text);
        
        // 2. Script itemization
        let script_runs = itemize_scripts(text, &unicode_props);
        
        // 3. Bidi processing
        let bidi_runs = process_bidi(text, &script_runs, options.direction);
        
        // 4. Font matching and fallback
        let font_runs = match_fonts(text, &bidi_runs, font_id, &self.fonts);
        
        // 5. Shape each run
        let mut shaped_runs = Vec::new();
        for run in font_runs {
            let shaped = shape_run(&run, font, options);
            shaped_runs.push(shaped);
        }
        
        // 6. Position glyphs
        position_glyphs(&mut shaped_runs, options);
        
        // 7. Apply features
        apply_opentype_features(&mut shaped_runs, options.features);
        
        combine_runs(shaped_runs)
    }
}

fn shape_run(run: &TextRun, font: &OpenTypeFont, options: &ShapingOptions) -> ShapedRun {
    let mut glyphs = Vec::new();
    
    // Map characters to glyphs
    let cmap = font.get_cmap().unwrap();
    for ch in run.text.chars() {
        let glyph_id = cmap.get_glyph(ch).unwrap_or(0);
        glyphs.push(glyph_id);
    }
    
    // Apply GSUB (glyph substitution)
    if let Some(gsub) = font.get_gsub() {
        apply_gsub(&mut glyphs, &gsub, options);
    }
    
    // Apply GPOS (glyph positioning)
    let mut positions = vec![GlyphPosition::default(); glyphs.len()];
    if let Some(gpos) = font.get_gpos() {
        apply_gpos(&glyphs, &mut positions, &gpos, options);
    }
    
    ShapedRun { glyphs, positions }
}
```

### Phase 4: Rasterization Engine (Weeks 9-10)
Glyph rasterization implementation:

```rust
// src/renderer/rasterizer.rs
pub struct Rasterizer {
    cache: GlyphCache,
}

impl Rasterizer {
    pub fn rasterize(
        &mut self,
        font: &OpenTypeFont,
        glyph_id: GlyphId,
        size: f32,
        mode: RenderMode,
    ) -> GlyphBitmap {
        // Check cache first
        if let Some(bitmap) = self.cache.get(font_id, glyph_id, size, mode) {
            return bitmap.clone();
        }
        
        // Get glyph outline
        let glyf = font.get_glyf().unwrap();
        let outline = glyf.get_outline(glyph_id).unwrap();
        
        // Scale to pixel size
        let scale = size / font.units_per_em() as f32;
        let scaled_outline = outline.scale(scale);
        
        // Rasterize outline
        let bitmap = match mode {
            RenderMode::Mono => rasterize_mono(&scaled_outline),
            RenderMode::Gray => rasterize_grayscale(&scaled_outline),
            RenderMode::SubpixelRgb => rasterize_subpixel_rgb(&scaled_outline),
            _ => rasterize_grayscale(&scaled_outline),
        };
        
        // Cache the result
        self.cache.insert(font_id, glyph_id, size, mode, bitmap.clone());
        
        bitmap
    }
}

fn rasterize_grayscale(outline: &ScaledOutline) -> GlyphBitmap {
    let bounds = outline.bounds();
    let width = (bounds.max_x - bounds.min_x).ceil() as u32;
    let height = (bounds.max_y - bounds.min_y).ceil() as u32;
    
    let mut pixels = vec![0u8; (width * height) as usize];
    
    // Scan conversion with anti-aliasing
    for y in 0..height {
        let scan_y = bounds.min_y + y as f32;
        let intersections = outline.scan_line_intersections(scan_y);
        
        for x in 0..width {
            let coverage = calculate_coverage(x as f32 + bounds.min_x, scan_y, &intersections);
            pixels[(y * width + x) as usize] = (coverage * 255.0) as u8;
        }
    }
    
    GlyphBitmap {
        width,
        height,
        left: bounds.min_x as i32,
        top: bounds.max_y as i32,
        pitch: width as usize,
        data: pixels,
        format: RenderMode::Gray,
    }
}
```

### Phase 5: Platform Integration (Weeks 11-12)
Platform-specific font discovery:

```rust
// src/platform/linux.rs
use std::process::Command;

pub fn discover_system_fonts() -> Vec<PathBuf> {
    let mut fonts = Vec::new();
    
    // Use fontconfig
    let output = Command::new("fc-list")
        .arg("--format=%{file}\n")
        .output()
        .expect("Failed to execute fc-list");
    
    let paths = String::from_utf8_lossy(&output.stdout);
    for path in paths.lines() {
        if let Ok(path) = PathBuf::from_str(path) {
            if path.exists() {
                fonts.push(path);
            }
        }
    }
    
    // Also check common directories
    let common_dirs = [
        "/usr/share/fonts",
        "/usr/local/share/fonts",
        "~/.fonts",
        "~/.local/share/fonts",
    ];
    
    for dir in &common_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if is_font_file(&path) {
                    fonts.push(path);
                }
            }
        }
    }
    
    fonts
}
```

## Testing Strategy

### Unit Test Suite

```rust
// tests/unit/parser_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_ttf() {
        let data = include_bytes!("../fixtures/NotoSans-Regular.ttf");
        let font = OpenTypeFont::parse(data.to_vec()).unwrap();
        
        assert!(font.has_table(Tag::from_str("cmap")));
        assert!(font.has_table(Tag::from_str("glyf")));
        assert!(font.has_table(Tag::from_str("head")));
    }
    
    #[test]
    fn test_cmap_lookup() {
        let data = include_bytes!("../fixtures/NotoSans-Regular.ttf");
        let font = OpenTypeFont::parse(data.to_vec()).unwrap();
        let cmap = font.get_cmap().unwrap();
        
        assert_eq!(cmap.get_glyph('A'), Some(GlyphId(36)));
        assert_eq!(cmap.get_glyph('€'), Some(GlyphId(123)));
        assert_eq!(cmap.get_glyph('\u{10000}'), None); // Not in font
    }
    
    #[test]
    fn test_metrics() {
        let data = include_bytes!("../fixtures/NotoSans-Regular.ttf");
        let font = OpenTypeFont::parse(data.to_vec()).unwrap();
        let metrics = font.get_metrics();
        
        assert_eq!(metrics.units_per_em, 2048);
        assert!(metrics.ascent > 0.0);
        assert!(metrics.descent < 0.0);
    }
}

// tests/unit/shaper_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_shaping() {
        let shaper = create_test_shaper();
        let shaped = shaper.shape("Hello", font_id, 16.0, &default_options());
        
        assert_eq!(shaped.glyphs.len(), 5);
        assert!(shaped.width > 0.0);
    }
    
    #[test]
    fn test_ligatures() {
        let shaper = create_test_shaper();
        let mut options = default_options();
        options.ligatures = true;
        
        let shaped = shaper.shape("ff", font_id, 16.0, &options);
        
        // Should produce single ff ligature glyph
        assert_eq!(shaped.glyphs.len(), 1);
    }
    
    #[test]
    fn test_arabic_shaping() {
        let shaper = create_test_shaper();
        let options = ShapingOptions {
            script: Script::Arabic,
            direction: Direction::RightToLeft,
            ..default_options()
        };
        
        let shaped = shaper.shape("السلام", font_id, 16.0, &options);
        
        // Check RTL ordering
        assert!(shaped.glyphs[0].position.x > shaped.glyphs.last().unwrap().position.x);
    }
    
    #[test]
    fn test_vertical_text() {
        let shaper = create_test_shaper();
        let options = ShapingOptions {
            direction: Direction::TopToBottom,
            ..default_options()
        };
        
        let shaped = shaper.shape("日本", font_id, 16.0, &options);
        
        // Check vertical positioning
        assert!(shaped.glyphs[1].position.y > shaped.glyphs[0].position.y);
    }
}

// tests/unit/rasterizer_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glyph_rasterization() {
        let mut rasterizer = Rasterizer::new();
        let bitmap = rasterizer.rasterize(font, GlyphId(36), 16.0, RenderMode::Gray);
        
        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
        assert_eq!(bitmap.data.len(), (bitmap.width * bitmap.height) as usize);
    }
    
    #[test]
    fn test_subpixel_rendering() {
        let mut rasterizer = Rasterizer::new();
        let bitmap = rasterizer.rasterize(font, GlyphId(36), 16.0, RenderMode::SubpixelRgb);
        
        // Subpixel should have 3x the width in data
        assert_eq!(bitmap.data.len(), (bitmap.width * 3 * bitmap.height) as usize);
    }
    
    #[test]
    fn test_cache_hit() {
        let mut rasterizer = Rasterizer::new();
        
        let bitmap1 = rasterizer.rasterize(font, GlyphId(36), 16.0, RenderMode::Gray);
        let bitmap2 = rasterizer.rasterize(font, GlyphId(36), 16.0, RenderMode::Gray);
        
        // Should be cached, same memory
        assert_eq!(bitmap1.data.as_ptr(), bitmap2.data.as_ptr());
    }
}
```

### Integration Tests

```rust
// tests/integration/browser_integration_test.rs
#[test]
fn test_component_message_handling() {
    let mut font_system = FontSystem::new(default_config()).unwrap();
    font_system.initialize(component_config()).unwrap();
    
    // Test web font loading
    let response = font_system.handle_message(ComponentMessage::LoadWebFont {
        url: "https://fonts.gstatic.com/test.woff2".into(),
        data: load_test_font_data(),
    }).unwrap();
    
    match response {
        ComponentResponse::FontLoaded { font_id } => {
            assert!(font_id.0 > 0);
        }
        _ => panic!("Unexpected response"),
    }
    
    // Test text shaping
    let response = font_system.handle_message(ComponentMessage::ShapeText {
        text: "Hello, World!".into(),
        descriptor: FontDescriptor {
            family: vec!["Arial".into()],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 16.0,
        },
        options: default_shaping_options(),
    }).unwrap();
    
    match response {
        ComponentResponse::ShapedText { shaped } => {
            assert!(shaped.glyphs.len() > 0);
            assert!(shaped.width > 0.0);
        }
        _ => panic!("Unexpected response"),
    }
}

#[test]
fn test_font_fallback_chain() {
    let mut font_system = FontSystem::new(default_config()).unwrap();
    font_system.load_system_fonts().unwrap();
    
    // Text with multiple scripts
    let text = "Hello 世界 مرحبا";
    let shaped = font_system.shape_text_with_fallback(
        text,
        &FontDescriptor {
            family: vec!["Arial".into()],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 16.0,
        },
        &default_shaping_options(),
    ).unwrap();
    
    // Should use different fonts for different scripts
    let font_ids: HashSet<_> = shaped.glyphs.iter().map(|g| g.font_id).collect();
    assert!(font_ids.len() >= 2); // At least 2 different fonts
}
```

### Performance Benchmarks

```rust
// benches/shaping_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_shaping(c: &mut Criterion) {
    let font_system = setup_font_system();
    let text = "The quick brown fox jumps over the lazy dog";
    
    c.bench_function("shape_latin_text", |b| {
        b.iter(|| {
            font_system.shape_text(
                black_box(text),
                font_id,
                16.0,
                &default_options(),
            )
        })
    });
    
    let arabic_text = "نص عربي طويل للاختبار";
    c.bench_function("shape_arabic_text", |b| {
        b.iter(|| {
            font_system.shape_text(
                black_box(arabic_text),
                font_id,
                16.0,
                &arabic_options(),
            )
        })
    });
    
    let cjk_text = "日本語のテキストと中文混合";
    c.bench_function("shape_cjk_text", |b| {
        b.iter(|| {
            font_system.shape_text(
                black_box(cjk_text),
                font_id,
                16.0,
                &cjk_options(),
            )
        })
    });
}

fn benchmark_rasterization(c: &mut Criterion) {
    let mut rasterizer = Rasterizer::new();
    
    c.bench_function("rasterize_glyph_gray", |b| {
        b.iter(|| {
            rasterizer.rasterize(
                font,
                GlyphId(36),
                black_box(16.0),
                RenderMode::Gray,
            )
        })
    });
    
    c.bench_function("rasterize_glyph_subpixel", |b| {
        b.iter(|| {
            rasterizer.rasterize(
                font,
                GlyphId(36),
                black_box(16.0),
                RenderMode::SubpixelRgb,
            )
        })
    });
}

criterion_group!(benches, benchmark_shaping, benchmark_rasterization);
criterion_main!(benches);
```

### Web Platform Tests Integration

```rust
// tests/wpt/font_loading_tests.rs
use wpt_runner::WptTest;

#[wpt_test("css/css-fonts/font-face-loading.html")]
fn test_font_face_loading(test: WptTest) {
    let font_system = FontSystem::new(default_config()).unwrap();
    
    // Execute WPT test case
    test.run(|step| {
        match step {
            WptStep::LoadFont { url, data } => {
                font_system.load_font_data(data).unwrap();
            }
            WptStep::CheckMetrics { expected } => {
                let metrics = font_system.get_font_metrics(font_id, 16.0).unwrap();
                assert_eq!(metrics, expected);
            }
            _ => {}
        }
    });
}
```

## Build Configuration

### Cargo.toml

```toml
[package]
name = "font-system"
version = "0.1.0"
edition = "2021"
authors = ["CortenBrowser Team"]
license = "MIT OR Apache-2.0"
description = "Font loading, shaping, and rendering system for CortenBrowser"

[dependencies]
# Core dependencies
browser-interfaces = { path = "../shared/interfaces" }
browser-messages = { path = "../shared/messages" }
browser-types = { path = "../shared/types" }

# Font handling
harfbuzz-sys = "0.5"        # Initial implementation
harfbuzz_rs = "2.0"          # Rust bindings
freetype-rs = "0.32"         # Initial rasterization
ttf-parser = "0.19"          # Pure Rust TTF parser

# Unicode and text processing
unicode-bidi = "0.3"
unicode-normalization = "0.1"
unicode-script = "0.5"
unicode-segmentation = "1.10"
unicode-linebreak = "0.1"

# Data structures
rustc-hash = "1.1"           # Fast hash maps
smallvec = "1.11"            # Small vector optimization
arc-swap = "1.6"             # Atomic Arc swapping

# Utilities
byteorder = "1.5"
bitflags = "2.4"
thiserror = "1.0"
anyhow = "1.0"
log = "0.4"

# Platform-specific
[target.'cfg(target_os = "linux")'.dependencies]
fontconfig = "0.8"

[target.'cfg(target_os = "windows")'.dependencies]
dwrote = "0.11"              # DirectWrite bindings

[target.'cfg(target_os = "macos")'.dependencies]
core-text = "20.1"           # CoreText bindings

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
test-case = "3.1"
pretty_assertions = "1.4"

[features]
default = ["harfbuzz", "system-fonts"]
harfbuzz = ["harfbuzz-sys", "harfbuzz_rs"]
pure-rust = []               # Pure Rust implementation
system-fonts = []            # Load system fonts
web-fonts = []               # Web font loading
color-fonts = []             # Color font support
variable-fonts = []          # Variable font support
subpixel = []               # Subpixel rendering
gpu-cache = []              # GPU glyph caching

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.bench]
inherits = "release"
```

### Build Script

```rust
// build.rs
use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    match target_os.as_str() {
        "linux" => {
            // Link fontconfig
            pkg_config::Config::new()
                .atleast_version("2.11.0")
                .probe("fontconfig")
                .unwrap();
        }
        "windows" => {
            // Windows font libraries are usually available
        }
        "macos" => {
            // Link CoreText framework
            println!("cargo:rustc-link-lib=framework=CoreText");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=CoreGraphics");
        }
        _ => {}
    }
    
    // Set up feature flags based on available libraries
    if cfg!(feature = "harfbuzz") {
        pkg_config::Config::new()
            .atleast_version("2.6.0")
            .probe("harfbuzz")
            .unwrap();
    }
}
```

## Development Milestones

### Milestone 1: Basic Font Loading (Week 1)
- [ ] Load system fonts on Linux/Windows/macOS
- [ ] Parse TTF/OTF files
- [ ] Basic font registry
- [ ] Font matching by family name
- **Validation**: Load and list all system fonts

### Milestone 2: Harfbuzz Integration (Week 2)
- [ ] Wrap Harfbuzz for text shaping
- [ ] Basic Latin text shaping
- [ ] Glyph positioning
- [ ] FreeType rasterization
- **Validation**: Shape and render "Hello World"

### Milestone 3: Font Metrics & Fallback (Week 3)
- [ ] Extract font metrics
- [ ] Implement font fallback chain
- [ ] Unicode script detection
- [ ] Coverage-based fallback
- **Validation**: Correctly render mixed-script text

### Milestone 4: Pure Rust Parser (Week 4)
- [ ] Complete OpenType parser
- [ ] WOFF/WOFF2 support
- [ ] CFF/CFF2 parsing
- [ ] Variable font parsing
- **Validation**: Parse Google Fonts collection

### Milestone 5: Bidirectional Text (Week 5)
- [ ] Unicode bidi algorithm
- [ ] RTL text shaping
- [ ] Mixed direction text
- [ ] Bidi paragraph levels
- **Validation**: Pass Unicode bidi test suite

### Milestone 6: Complex Shaping (Week 6)
- [ ] Arabic shaping
- [ ] Indic scripts
- [ ] Thai/Lao/Khmer
- [ ] Vertical text (CJK)
- **Validation**: Pass Harfbuzz test suite

### Milestone 7: OpenType Features (Week 7)
- [ ] GSUB implementation
- [ ] GPOS implementation
- [ ] Feature selection
- [ ] Contextual substitution
- **Validation**: Liga, kern, and other features work

### Milestone 8: Pure Rust Shaper (Week 8)
- [ ] Replace Harfbuzz with Rust impl
- [ ] Performance optimization
- [ ] Caching system
- [ ] Memory optimization
- **Validation**: Performance within 2x of Harfbuzz

### Milestone 9: Rasterization (Week 9)
- [ ] Pure Rust rasterizer
- [ ] Anti-aliasing
- [ ] Subpixel rendering
- [ ] Hinting support
- **Validation**: Visual quality matches FreeType

### Milestone 10: Advanced Features (Week 10)
- [ ] Color fonts (COLR/CPAL)
- [ ] Emoji support
- [ ] SVG-in-OpenType
- [ ] Bitmap fonts
- **Validation**: Render color emoji correctly

### Milestone 11: Platform Integration (Week 11)
- [ ] DirectWrite integration (Windows)
- [ ] CoreText integration (macOS)
- [ ] Advanced font matching
- [ ] System font substitution
- **Validation**: Platform-specific features work

### Milestone 12: Performance & Polish (Week 12)
- [ ] GPU glyph caching
- [ ] Parallel shaping
- [ ] Memory pool optimization
- [ ] Final optimizations
- **Validation**: Meet performance targets

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Font loading | < 100ms for 100 fonts | Time to load system fonts |
| Text shaping | < 1ms for 1000 chars | Latin text shaping time |
| Complex shaping | < 5ms for 1000 chars | Arabic/Indic shaping time |
| Glyph rasterization | < 0.1ms per glyph | 16px grayscale rendering |
| Memory per font | < 1MB | Loaded font memory usage |
| Cache hit rate | > 95% | Glyph cache effectiveness |
| Fallback lookup | < 0.01ms | Time to find fallback font |

## Security Considerations

### Font File Validation
- Validate all table checksums
- Bounds check all offsets
- Limit recursion depth
- Sanitize malformed data
- Reject suspiciously large fonts

### Memory Safety
- Use safe Rust patterns
- No unsafe blocks without justification
- Fuzz test all parsers
- Bounds check all array access
- Validate all user input

### Sandboxing
- Run font parsing in separate process
- Limit memory allocation
- Timeout long operations
- Validate all IPC messages

## External Dependencies

### Required System Libraries
- **Linux**: fontconfig, freetype2 (initial)
- **Windows**: DirectWrite (optional)
- **macOS**: CoreText (optional)

### Rust Crates
- **Essential**: unicode-*, ttf-parser
- **Initial**: harfbuzz_rs, freetype-rs
- **Optional**: fontconfig, dwrote, core-text

### Test Data
- Google Fonts collection
- Noto fonts (comprehensive Unicode)
- Adobe Source fonts
- Test fonts from Harfbuzz

## Integration Points

### CSS Engine
- Receives font requests
- Provides font descriptors
- Updates on @font-face rules

### Rendering Engine
- Receives shaped text
- Gets glyph bitmaps
- Manages GPU cache

### Network Stack
- Downloads web fonts
- Handles font subsets
- CORS validation

### Browser Shell
- System font preferences
- Font rendering settings
- Subpixel configuration

## Commands for Claude Code

### Initial Setup
```bash
# Create project structure
cargo new --lib font-system
cd font-system

# Add dependencies
cargo add browser-interfaces browser-messages browser-types
cargo add harfbuzz_rs freetype-rs ttf-parser
cargo add unicode-bidi unicode-script unicode-segmentation
cargo add rustc-hash smallvec byteorder
cargo add --dev criterion proptest test-case

# Create module structure
mkdir -p src/{registry,parser,shaper,renderer,layout,platform}
touch src/{registry,parser,shaper,renderer,layout,platform}/mod.rs

# Set up test fixtures
mkdir -p tests/{unit,integration,fixtures}
# Download test fonts to tests/fixtures/
```

### Development Commands
```bash
# Build with default features (Harfbuzz)
cargo build --release

# Build pure Rust version
cargo build --release --no-default-features --features pure-rust

# Run unit tests
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run benchmarks
cargo bench

# Check specific platform
cargo check --target x86_64-pc-windows-msvc
cargo check --target x86_64-apple-darwin

# Generate documentation
cargo doc --no-deps --open
```

### Testing Commands
```bash
# Run with test harness
cargo run --example test_harness -- --font-dir /usr/share/fonts

# Test against WPT
./run-wpt-tests.sh css/css-fonts/

# Fuzz testing
cargo fuzz run parse_font

# Memory profiling
valgrind --leak-check=full target/release/font-system-test

# Performance profiling
perf record -g target/release/font-system-bench
perf report
```

## Success Criteria

### Phase 1 (Harfbuzz-based)
- ✓ Loads all system fonts
- ✓ Shapes Latin text correctly
- ✓ Renders basic text
- ✓ Font fallback works

### Phase 2 (Hybrid)
- ✓ Pure Rust font parser
- ✓ Complex script shaping
- ✓ Bidirectional text
- ✓ 80% of WPT font tests pass

### Phase 3 (Pure Rust)
- ✓ Harfbuzz-free operation
- ✓ Performance within 2x
- ✓ All major scripts supported
- ✓ 95% of WPT font tests pass

## Conclusion

This specification provides a complete blueprint for implementing the Font System component of CortenBrowser. The phased approach allows for rapid initial development using existing libraries, followed by progressive migration to a pure Rust implementation. The comprehensive testing strategy ensures correctness and performance throughout development.

The modular architecture ensures clean integration with other browser components while maintaining independence for parallel development. With clear milestones and success criteria, this component can be developed efficiently by Claude Code instances working within the orchestration system.

---

*End of Font System Component Specification v1.0*