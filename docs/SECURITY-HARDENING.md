# Security Hardening Guide - Corten Font System

This guide provides practical recommendations for hardening the Corten Font System for production deployment.

---

## Table of Contents

1. [Input Validation](#input-validation)
2. [Resource Limits](#resource-limits)
3. [Fuzzing Setup](#fuzzing-setup)
4. [Security Testing](#security-testing)
5. [Production Configuration](#production-configuration)
6. [Incident Response](#incident-response)

---

## 1. Input Validation

### Font File Validation

**Priority**: 🔴 **CRITICAL**

#### File Size Limits

```rust
// Recommended implementation for font_registry
const MAX_FONT_FILE_SIZE: usize = 100 * 1024 * 1024;  // 100 MB

pub fn load_font_file(&mut self, path: &Path) -> Result<FontId, RegistryError> {
    // Validate path exists
    if !path.exists() {
        return Err(RegistryError::FileNotFound(path.to_path_buf()));
    }

    // Check file size BEFORE reading
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_FONT_FILE_SIZE as u64 {
        return Err(RegistryError::FileTooLarge {
            path: path.to_path_buf(),
            size: metadata.len(),
            limit: MAX_FONT_FILE_SIZE as u64,
        });
    }

    // Validate file is readable
    let data = std::fs::read(path)?;

    // Load font data
    self.load_font_data(data)
}
```

**Error Type Addition**:
```rust
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Font file too large: {path:?} ({size} bytes, limit {limit} bytes)")]
    FileTooLarge {
        path: PathBuf,
        size: u64,
        limit: u64,
    },
    // ... existing error types
}
```

#### Font Data Validation

```rust
// Recommended for font_parser
pub fn parse_font_data(data: &[u8]) -> Result<OpenTypeFont, ParseError> {
    // Validate minimum size (header + required tables)
    const MIN_FONT_SIZE: usize = 256;  // Minimum valid font
    if data.len() < MIN_FONT_SIZE {
        return Err(ParseError::CorruptedData(
            format!("Font data too small: {} bytes", data.len())
        ));
    }

    // Validate magic number (OTF/TTF signature)
    if data.len() < 4 {
        return Err(ParseError::CorruptedData("Missing font signature".to_string()));
    }

    let signature = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    match signature {
        0x00010000 | // TrueType 1.0
        0x4F54544F | // "OTTO" (OpenType with CFF)
        0x74727565 | // "true" (Apple TrueType)
        0x74797031   // "typ1" (PostScript)
        => Ok(()),
        _ => Err(ParseError::InvalidFormat),
    }?;

    // Continue with parsing...
}
```

### Text Input Validation

**Priority**: 🔴 **CRITICAL**

#### Text Length Limits

```rust
// Recommended for text_shaper
const MAX_TEXT_LENGTH: usize = 1_000_000;  // 1 million characters
const WARN_TEXT_LENGTH: usize = 100_000;   // Warn at 100K

pub fn shape_text(&self, text: &str, options: &ShapingOptions) -> Result<Vec<ShapedGlyph>, ShapingError> {
    // Empty check (already implemented)
    if text.is_empty() {
        return Ok(Vec::new());
    }

    // Length validation
    let text_len = text.chars().count();  // Unicode character count

    if text_len > MAX_TEXT_LENGTH {
        return Err(ShapingError::TextTooLong {
            length: text_len,
            limit: MAX_TEXT_LENGTH,
        });
    }

    if text_len > WARN_TEXT_LENGTH {
        eprintln!("Warning: Shaping large text ({} chars)", text_len);
    }

    // Continue with shaping...
}
```

**Error Type Addition**:
```rust
#[derive(Error, Debug)]
pub enum ShapingError {
    #[error("Text too long: {length} characters (limit {limit})")]
    TextTooLong {
        length: usize,
        limit: usize,
    },
    // ... existing error types
}
```

### Glyph Index Validation

**Priority**: 🔴 **CRITICAL**

```rust
// Recommended for glyph_renderer
pub fn rasterize_glyph(
    &mut self,
    font: &OpenTypeFont,
    glyph_id: GlyphId,
    size: f32,
    mode: RenderMode,
) -> Result<GlyphBitmap, RenderError> {
    // Validate glyph ID is within bounds
    if glyph_id.0 >= font.num_glyphs() {
        return Err(RenderError::InvalidGlyphId {
            glyph_id,
            max_glyphs: font.num_glyphs(),
        });
    }

    // Validate font size (prevent overflow and unreasonable values)
    const MIN_FONT_SIZE: f32 = 1.0;
    const MAX_FONT_SIZE: f32 = 1000.0;

    if size < MIN_FONT_SIZE || size > MAX_FONT_SIZE {
        return Err(RenderError::InvalidFontSize {
            size,
            min: MIN_FONT_SIZE,
            max: MAX_FONT_SIZE,
        });
    }

    // Continue with rasterization...
}
```

---

## 2. Resource Limits

### Memory Limits

**Priority**: 🟡 **HIGH**

#### Cache Configuration

```rust
// Recommended production configuration
let config = CacheConfig {
    glyph_cache: GlyphCacheConfig {
        max_entries: 10_000,              // 10K glyphs
        max_memory_bytes: 100 * 1024 * 1024,  // 100 MB
        enable_statistics: true,          // Monitor in production
    },
    shaping_cache: ShapingCacheConfig {
        max_entries: 1_000,               // 1K shaped text results
        enable_statistics: true,
    },
};

// For embedded/constrained systems
let embedded_config = CacheConfig {
    glyph_cache: GlyphCacheConfig {
        max_entries: 1_000,               // 1K glyphs
        max_memory_bytes: 10 * 1024 * 1024,  // 10 MB
        enable_statistics: false,         // Save memory
    },
    shaping_cache: ShapingCacheConfig {
        max_entries: 100,
        enable_statistics: false,
    },
};
```

#### Proactive Cache Management

```rust
// Monitor and manage cache proactively
fn monitor_cache_health(font_system: &FontSystem) {
    let stats = font_system.cache_stats();

    // Memory pressure detection
    let memory_usage_percent =
        (stats.memory_bytes as f64 / stats.max_memory_bytes as f64) * 100.0;

    if memory_usage_percent > 90.0 {
        eprintln!("Warning: Cache memory usage at {:.1}%", memory_usage_percent);
        // Consider clearing cache or reducing limits
    }

    // Hit rate monitoring
    if stats.hit_rate < 0.5 {
        eprintln!("Warning: Low cache hit rate ({:.1}%)", stats.hit_rate * 100.0);
        // May need to increase cache size or adjust access patterns
    }

    // Eviction monitoring
    if stats.evictions > 1000 {
        eprintln!("Info: High eviction count ({})", stats.evictions);
        // May need larger cache
    }
}
```

### Recursion Limits

**Priority**: 🟡 **MEDIUM**

```rust
// For font parsing with nested structures
const MAX_RECURSION_DEPTH: usize = 100;

struct Parser {
    depth: usize,
}

impl Parser {
    fn parse_nested(&mut self, data: &[u8]) -> Result<Value, ParseError> {
        self.depth += 1;

        if self.depth > MAX_RECURSION_DEPTH {
            return Err(ParseError::RecursionLimitExceeded {
                depth: self.depth,
                limit: MAX_RECURSION_DEPTH,
            });
        }

        // Parse...
        let result = self.parse_value(data)?;

        self.depth -= 1;
        Ok(result)
    }
}
```

---

## 3. Fuzzing Setup

**Priority**: 🟡 **HIGH**

### Installing cargo-fuzz

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing for font_parser
cd components/font_parser
cargo fuzz init

# This creates fuzz/ directory with fuzzing targets
```

### Fuzzing Target: Font Parser

```rust
// fuzz/fuzz_targets/fuzz_font_parser.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use font_parser::{parse_font_data, OpenTypeFont};

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary data as font
    let _ = parse_font_data(data);

    // If parsing succeeds, try to access font data
    if let Ok(font) = parse_font_data(data) {
        let _ = font.num_glyphs();
        let _ = font.family_name();
        // ... access other fields
    }
});
```

### Running Fuzzing

```bash
# Run fuzzing (runs indefinitely until crash or Ctrl+C)
cargo fuzz run fuzz_font_parser

# Run with timeout
timeout 3600 cargo fuzz run fuzz_font_parser  # 1 hour

# Run with corpus from real fonts
mkdir -p fuzz/corpus/fuzz_font_parser
cp /usr/share/fonts/*/*.ttf fuzz/corpus/fuzz_font_parser/
cargo fuzz run fuzz_font_parser

# Minimize corpus (reduce to unique test cases)
cargo fuzz cmin fuzz_font_parser
```

### Fuzzing Target: Text Shaper

```rust
// fuzz/fuzz_targets/fuzz_text_shaper.rs
#![no_main]

use libfuzzer_sys::fuzz_target;
use text_shaper::{TextShaper, ShapingOptions};
use font_registry::FontRegistry;

fuzz_target!(|data: &[u8]| {
    // Convert arbitrary bytes to valid UTF-8
    if let Ok(text) = std::str::from_utf8(data) {
        let mut registry = FontRegistry::new();
        // Load a known-good font
        let font_id = registry.load_font_file(Path::new("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf")).ok();

        if let Some(font_id) = font_id {
            let shaper = TextShaper::new();
            let options = ShapingOptions::default();
            let _ = shaper.shape_text(text, font_id, &options);
        }
    }
});
```

### CI/CD Integration

```yaml
# .github/workflows/fuzz.yml
name: Fuzzing

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust nightly
        run: rustup default nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Run fuzzing (1 hour)
        run: |
          cd components/font_parser
          timeout 3600 cargo fuzz run fuzz_font_parser || true

      - name: Upload crashes
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-crashes
          path: components/font_parser/fuzz/artifacts/
```

---

## 4. Security Testing

### Unit Tests for Security

```rust
// components/font_parser/tests/security_tests.rs
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_reject_oversized_font() {
        // Create data larger than limit
        let oversized = vec![0u8; MAX_FONT_FILE_SIZE + 1];

        let result = parse_font_data(&oversized);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::CorruptedData(_)));
    }

    #[test]
    fn test_reject_undersized_font() {
        let tiny = vec![0u8; 10];  // Too small to be valid

        let result = parse_font_data(&tiny);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_magic_number() {
        let mut data = vec![0u8; 512];
        // Set invalid magic number
        data[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

        let result = parse_font_data(&data);
        assert!(matches!(result.unwrap_err(), ParseError::InvalidFormat));
    }

    #[test]
    fn test_extreme_glyph_count() {
        // Test with font claiming massive glyph count
        // Should be rejected or handled safely
    }

    #[test]
    fn test_malformed_table_offsets() {
        // Test with invalid table offsets
        // Should detect and reject
    }
}
```

### Integration Tests for Security

```rust
// tests/security_integration_tests.rs
#[test]
fn test_memory_limit_enforcement() {
    let config = FontSystemConfig {
        cache_config: CacheConfig {
            glyph_cache: GlyphCacheConfig {
                max_entries: 100,
                max_memory_bytes: 1024 * 1024,  // 1 MB limit
                enable_statistics: true,
            },
            // ...
        },
        // ...
    };

    let mut font_system = FontSystem::new(config).unwrap();

    // Load font
    let font_id = load_test_font(&mut font_system);

    // Render many glyphs to fill cache
    for glyph_id in 0..1000 {
        let _ = font_system.rasterize_glyph(
            font_id,
            GlyphId(glyph_id),
            12.0,
            RenderMode::Gray
        );
    }

    // Verify memory limit was respected
    let stats = font_system.cache_stats();
    assert!(stats.memory_bytes <= 1024 * 1024);
    assert!(stats.evictions > 0);  // Some evictions occurred
}

#[test]
fn test_concurrent_safety() {
    // Test that FontSystem is safe under concurrent access
    // (or documents it's not thread-safe)
}

#[test]
fn test_resource_cleanup_on_error() {
    // Verify resources are cleaned up when errors occur
}
```

### Property-Based Testing

```rust
// Use proptest for property-based testing
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_any_text_length_handled(text in "\\PC*") {
        // Any valid UTF-8 string should be handled safely
        let shaper = TextShaper::new();
        let options = ShapingOptions::default();

        // Should either succeed or return error, never panic
        let _ = shaper.shape_text(&text, font_id, &options);
    }

    #[test]
    fn test_any_font_size_handled(size in 0.0f32..10000.0f32) {
        // Any font size should be handled safely
        let result = font_system.rasterize_glyph(
            font_id,
            GlyphId(0),
            size,
            RenderMode::Gray
        );

        // Should either succeed or return error, never panic
        assert!(result.is_ok() || result.is_err());
    }
}
```

---

## 5. Production Configuration

### Secure Defaults

```rust
// src/config.rs - Production-ready defaults
impl Default for FontSystemConfig {
    fn default() -> Self {
        Self {
            cache_config: CacheConfig {
                glyph_cache: GlyphCacheConfig {
                    max_entries: 10_000,
                    max_memory_bytes: 100 * 1024 * 1024,
                    enable_statistics: cfg!(debug_assertions),  // Enable in debug only
                },
                shaping_cache: ShapingCacheConfig {
                    max_entries: 1_000,
                    enable_statistics: cfg!(debug_assertions),
                },
            },
            enable_subpixel: true,
            enable_hinting: true,
            load_system_fonts_on_init: false,  // Load on-demand for security

            // New security settings
            max_font_file_size: 100 * 1024 * 1024,  // 100 MB
            max_text_length: 1_000_000,  // 1M characters
            max_recursion_depth: 100,
            validate_font_signatures: true,
        }
    }
}
```

### Environment-Specific Configuration

```rust
// Different configurations for different environments
pub struct FontSystemConfigBuilder;

impl FontSystemConfigBuilder {
    pub fn development() -> FontSystemConfig {
        FontSystemConfig {
            cache_config: CacheConfig {
                glyph_cache: GlyphCacheConfig {
                    max_entries: 1_000,  // Smaller for faster tests
                    max_memory_bytes: 10 * 1024 * 1024,
                    enable_statistics: true,
                },
                // ...
            },
            // Stricter limits for faster failure
            max_font_file_size: 10 * 1024 * 1024,  // 10 MB
            max_text_length: 10_000,
            // ...
        }
    }

    pub fn production() -> FontSystemConfig {
        FontSystemConfig::default()  // Use secure defaults
    }

    pub fn embedded() -> FontSystemConfig {
        FontSystemConfig {
            cache_config: CacheConfig {
                glyph_cache: GlyphCacheConfig {
                    max_entries: 500,
                    max_memory_bytes: 5 * 1024 * 1024,  // 5 MB
                    enable_statistics: false,  // Save memory
                },
                // ...
            },
            enable_subpixel: false,  // Save memory
            // ...
        }
    }
}
```

---

## 6. Incident Response

### Incident Response Plan

1. **Detection**
   - Monitor logs for unusual patterns
   - Track crash reports
   - Monitor memory usage
   - Watch for security advisories

2. **Assessment**
   - Classify severity (Critical/High/Medium/Low)
   - Determine impact scope
   - Identify affected versions

3. **Containment**
   - Disable affected feature if possible
   - Update to safe configuration
   - Apply temporary workarounds

4. **Remediation**
   - Develop fix
   - Test thoroughly
   - Prepare security advisory
   - Coordinate disclosure

5. **Recovery**
   - Release patched version
   - Notify users
   - Monitor for exploitation
   - Post-incident review

### Monitoring and Alerting

```rust
// Implement security-relevant logging
use log::{warn, error};

pub fn load_font_file(&mut self, path: &Path) -> Result<FontId, RegistryError> {
    // Security event logging
    if let Some(extension) = path.extension() {
        if extension != "ttf" && extension != "otf" {
            warn!("Unusual font file extension: {:?}", extension);
        }
    }

    let metadata = std::fs::metadata(path)?;
    if metadata.len() > 50 * 1024 * 1024 {  // > 50 MB
        warn!("Large font file: {:?} ({} bytes)", path, metadata.len());
    }

    // ... continue loading
}

pub fn shape_text(&self, text: &str, ...) -> Result<Vec<ShapedGlyph>, ShapingError> {
    if text.len() > 50_000 {
        warn!("Shaping large text: {} characters", text.len());
    }

    // ... continue shaping
}
```

---

## Summary Checklist

### Before Production Deployment

- [ ] Input validation implemented
  - [ ] Font file size limits
  - [ ] Text length limits
  - [ ] Glyph ID validation
  - [ ] Font signature validation

- [ ] Resource limits configured
  - [ ] Cache memory limits
  - [ ] Cache entry limits
  - [ ] Recursion depth limits

- [ ] Security testing complete
  - [ ] Unit tests for security scenarios
  - [ ] Integration tests for limits
  - [ ] Fuzzing configured and run
  - [ ] Property-based tests added

- [ ] Production configuration
  - [ ] Secure defaults set
  - [ ] Environment-specific configs ready
  - [ ] Monitoring and logging configured

- [ ] Incident response ready
  - [ ] Contact information updated
  - [ ] Monitoring in place
  - [ ] Alert thresholds configured
  - [ ] Response procedures documented

---

## References

- [Security Audit Report](./SECURITY-AUDIT-v0.1.0.md)
- [Security Policy](./SECURITY.md)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [cargo-fuzz Documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)

---

**Last Updated**: 2025-11-14
**Version**: 0.1.0
