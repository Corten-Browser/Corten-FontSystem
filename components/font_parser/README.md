# font_parser

**Type**: library
**Tech Stack**: Rust, ttf-parser, byteorder
**Version**: 0.1.0

## Responsibility

Parse OpenType, TrueType, WOFF, and WOFF2 font files

## Structure

```
├── src/           # Source code
├── tests/         # Tests (unit, integration, contracts)
├── benches/       # Benchmarks
├── Cargo.toml     # Rust package manifest
├── CLAUDE.md      # Component-specific instructions for Claude Code
└── README.md      # This file
```

## Dependencies

- font_types
- byteorder - Binary data reading
- flate2 - WOFF decompression (zlib)
- brotli - WOFF2 decompression (Brotli)

## Features

- **TrueType (TTF)** parsing - Signature: 0x00010000
- **OpenType (OTF/CFF)** parsing - Signature: 'OTTO' (0x4F54544F)
- **WOFF** web font parsing - Signature: 'wOFF' (0x774F4646)
- **WOFF2** web font parsing - Signature: 'wOF2' (0x774F4632)
- **Variable Fonts** (OpenType Font Variations) - fvar and avar table parsing
- **Color Fonts** (Emoji support) - COLR/CPAL, CBDT/CBLC, sbix, and SVG table parsing

All web font formats (WOFF/WOFF2) are automatically decompressed to standard TTF/OTF format before parsing.

## Usage

This component is ready for immediate use via Task tool orchestration.

### Basic Usage

```rust
use font_parser::OpenTypeFont;

// Parse any supported font format (TTF, OTF, WOFF, WOFF2)
let font_data = std::fs::read("path/to/font.woff2")?;
let font = OpenTypeFont::parse(font_data)?;

// Access font metrics
let metrics = font.get_metrics();
println!("Units per em: {}", metrics.units_per_em);
```

### Variable Fonts

```rust
use font_parser::{OpenTypeFont, VariationCoordinates, Tag};

// Parse variable font
let font_data = std::fs::read("path/to/variable-font.ttf")?;
let font = OpenTypeFont::parse(font_data)?;

// Check if variable
if font.is_variable() {
    // Get available axes
    let axes = font.get_variation_axes();
    for axis in &axes {
        println!("Axis: {}, range: {} to {}",
            axis.tag, axis.min_value, axis.max_value);
    }

    // Get named instances (e.g., "Bold", "Light")
    let instances = font.get_named_instances();
    for instance in &instances {
        println!("Instance ID: {}", instance.subfamily_name_id);
    }

    // Create instance at specific coordinates
    let mut coords = VariationCoordinates::new();
    coords.set_axis(Tag::WEIGHT, 700.0);  // Bold
    coords.set_axis(Tag::WIDTH, 100.0);   // Normal width

    // Validate coordinates
    font.validate_coordinates(&coords)?;

    // Apply avar mapping if present
    if let Some(avar) = font.get_avar() {
        let mapped_weight = avar.map_value(0, 700.0);
        println!("Mapped weight: {}", mapped_weight);
    }
}
```

### Standard Variation Axes

The library provides constants for standard OpenType variation axes:

```rust
use font_parser::Tag;

// Standard axis tags
Tag::WEIGHT        // 'wght' - Weight (100-900)
Tag::WIDTH         // 'wdth' - Width (50-200)
Tag::SLANT         // 'slnt' - Slant angle (-90 to 90)
Tag::OPTICAL_SIZE  // 'opsz' - Optical size (6-72)
Tag::ITALIC        // 'ital' - Italic (0 or 1)
```

### Color Fonts & Emoji

```rust
use font_parser::OpenTypeFont;

// Parse color font (emoji font)
let font_data = std::fs::read("path/to/emoji-font.ttf")?;
let font = OpenTypeFont::parse(font_data)?;

// Check if font has color glyphs
if font.is_color_font() {
    // Get supported color formats
    let formats = font.get_color_formats();
    println!("Color formats: {:?}", formats);

    // Get color palettes (CPAL)
    if let Some(cpal) = font.get_cpal() {
        let palette = cpal.default_palette().unwrap();
        println!("Default palette has {} colors", palette.len());

        // Access individual colors
        for (i, color) in palette.iter().enumerate() {
            println!("Color {}: R={}, G={}, B={}, A={}",
                i, color.red, color.green, color.blue, color.alpha);
        }
    }

    // Get color layers for a glyph (COLR)
    let glyph_id = 42;
    if font.has_color_layers(glyph_id) {
        let layers = font.get_color_layers(glyph_id).unwrap();
        for layer in layers {
            println!("Layer: glyph {:?}, palette color {}",
                layer.glyph_id, layer.palette_index);
        }
    }
}
```

#### Supported Color Font Formats

- **COLR/CPAL** - Vector-based layered color glyphs with palette support
- **CBDT/CBLC** - Embedded color bitmap data (commonly used for emoji)
- **sbix** - Apple's standard bitmap graphics format
- **SVG** - SVG-in-OpenType for scalable color glyphs

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

### Build and Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code
cargo clippy

# Format code
cargo fmt
```

## Architecture

Implementation details will be added during development following the specifications in `/home/user/Corten-FontSystem/font-system-specification.md`.
