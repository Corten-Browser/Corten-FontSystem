# platform_integration

**Type**: library
**Tech Stack**: Rust, fontconfig, dwrote, core-text
**Version**: 0.1.0

## Responsibility

Platform-specific font discovery (Linux, Windows, macOS)

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
- font_registry

## Usage

This component is ready for immediate use via Task tool orchestration.

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

### Platform-Specific Font Discovery

This component provides platform-specific font discovery with detailed metadata parsing.

#### Linux Implementation (Fontconfig)

**Status**: ✅ Fully Implemented

Uses the `fontconfig` library to discover system fonts with full metadata:

```rust
use platform_integration::discover_system_fonts_detailed;

let fonts = discover_system_fonts_detailed();
for font in fonts {
    println!("{}: weight={:?}, style={:?}",
             font.family_name, font.weight, font.style);
}
```

**Features**:
- Discovers all system fonts via fontconfig API
- Parses font metadata (family name, weight, style)
- Maps fontconfig weight values (0-210) to `FontWeight` enum
- Maps fontconfig slant values (0, 100, 110) to `FontStyle` enum
- Marks fonts as system fonts based on installation location
- Deduplicates font paths
- Falls back to basic discovery if fontconfig is unavailable

**Weight Mapping**:
- 0-40 → Thin (100)
- 41-55 → ExtraLight (200)
- 56-75 → Light (300)
- 76-90 → Regular (400)
- 91-110 → Medium (500)
- 111-180 → SemiBold (600)
- 181-200 → Bold (700)
- 201-209 → ExtraBold (800)
- 210+ → Black (900)

**Style Mapping**:
- 0 → Normal
- 100 → Italic
- 110 → Oblique (10°)

#### Windows Implementation

**Status**: 🔄 Planned (v0.2.0)

Will use DirectWrite API for font discovery:
- IDWriteFontCollection for system fonts
- Font property parsing (family, weight, style, stretch)
- DirectWrite enum mapping

See: [DirectWrite Documentation](https://docs.microsoft.com/en-us/windows/win32/directwrite/direct-write-portal)

#### macOS Implementation

**Status**: 🔄 Planned (v0.2.0)

Will use CoreText API for font discovery:
- CTFontCollection for system fonts
- Font descriptor parsing
- CoreText trait mapping

See: [CoreText Documentation](https://developer.apple.com/documentation/coretext)

### API

```rust
// Discover fonts with detailed metadata
pub fn discover_system_fonts_detailed() -> Vec<PlatformFontInfo>;

// Discover fonts (paths only, for backward compatibility)
pub fn discover_system_fonts() -> Vec<PathBuf>;

// Get platform default font families
pub fn get_default_font_families() -> HashMap<FontCategory, Vec<String>>;

// Get font configuration directory
pub fn get_font_config_path() -> Option<PathBuf>;

// Detect current platform
pub fn detect_platform() -> Platform;
```

### Testing

Comprehensive test coverage (43 tests):
- Unit tests for core functionality
- Integration tests with real system fonts
- Platform-specific tests (Linux fontconfig)
- Weight/style mapping tests
- Contract compliance tests

```bash
# Run all tests
cargo test

# Run fontconfig-specific tests
cargo test --test fontconfig_tests

# Run with coverage
cargo tarpaulin --out Html
```
