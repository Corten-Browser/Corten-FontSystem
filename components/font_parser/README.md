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

All web font formats (WOFF/WOFF2) are automatically decompressed to standard TTF/OTF format before parsing.

## Usage

This component is ready for immediate use via Task tool orchestration.

### Example

```rust
use font_parser::OpenTypeFont;

// Parse any supported font format (TTF, OTF, WOFF, WOFF2)
let font_data = std::fs::read("path/to/font.woff2")?;
let font = OpenTypeFont::parse(font_data)?;

// Access font metrics
let metrics = font.get_metrics();
println!("Units per em: {}", metrics.units_per_em);
```

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
