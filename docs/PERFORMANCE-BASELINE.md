# Performance Baseline - Corten Font System v0.1.0

This document establishes performance baselines and benchmarking procedures for the Corten Font System.

## Table of Contents

1. [Running Benchmarks](#running-benchmarks)
2. [Performance Targets](#performance-targets)
3. [Benchmark Suites](#benchmark-suites)
4. [Hardware Specifications](#hardware-specifications)
5. [Interpreting Results](#interpreting-results)
6. [Baseline Measurements](#baseline-measurements)

## Running Benchmarks

### Quick Start

```bash
# Run all benchmarks across the workspace
cargo bench --workspace

# Run benchmarks for a specific component
cargo bench --package font_registry
cargo bench --package text_shaper
cargo bench --package glyph_renderer
cargo bench --package font_system_api

# Run a specific benchmark
cargo bench --package font_registry -- font_matching
```

### Generate HTML Reports

Criterion automatically generates HTML reports in `target/criterion/`:

```bash
# Run benchmarks and save as baseline
cargo bench --package font_registry -- --save-baseline v0.1.0

# Compare against baseline
cargo bench --package font_registry -- --baseline v0.1.0

# View reports
open target/criterion/report/index.html
```

### CI/CD Integration

```bash
# Run benchmarks without plots (faster, CI-friendly)
cargo bench --workspace -- --noplot

# Save results for comparison
cargo bench --workspace -- --save-baseline ci-$(date +%Y%m%d)
```

## Performance Targets

### Primary Goals

| Operation | Target | Critical | Notes |
|-----------|--------|----------|-------|
| **Font Loading** | < 100ms | < 200ms | Per 100 fonts |
| **Font Matching** | < 1ms | < 5ms | Single match operation |
| **Text Shaping (100 chars)** | < 10ms | < 25ms | Simple ASCII text |
| **Text Shaping (complex)** | < 50ms | < 100ms | BiDi/ligatures |
| **Glyph Render (12pt)** | < 5ms | < 10ms | First render (cache miss) |
| **Glyph Render (cached)** | < 0.1ms | < 0.5ms | Cache hit |
| **E2E Workflow** | < 50ms | < 100ms | Load + shape + render |

### Secondary Goals

| Operation | Target | Notes |
|-----------|--------|-------|
| Font descriptor creation | < 1μs | Should be negligible |
| Cache statistics retrieval | < 10μs | O(1) operation |
| Cache clearing | < 1ms | Even with large cache |
| Registry creation | < 100μs | Empty registry |

### Performance Categories

- **Excellent**: Exceeds target by >20%
- **Good**: Meets or exceeds target
- **Acceptable**: Within critical threshold
- **Poor**: Exceeds critical threshold (requires optimization)

## Benchmark Suites

### 1. Font Registry Benchmarks

**Location**: `components/font_registry/benches/`

#### font_loading.rs
- `create_registry`: Baseline registry creation overhead
- `create_descriptor`: Font descriptor creation
- `load_font_file`: Single font file loading
- `load_font_data`: Font data from memory
- `registry_scalability`: Loading N fonts (10, 50, 100, 500, 1000)

#### font_matching.rs
- `match_font_default`: Matching with default descriptor
- `match_font_specific`: Matching with full descriptor
- `match_font_weights`: Matching across all weight values
- `match_font_scalability`: Matching with varying registry sizes
- `match_font_repeated`: Multiple consecutive matches (cache test)

### 2. Text Shaper Benchmarks

**Location**: `components/text_shaper/benches/`

#### benchmarks.rs
- `create_shaper`: Shaper creation overhead
- `shape_ascii`: ASCII text (short/medium/long/very_long)
- `shape_scalability`: Shaping N characters (10, 50, 100, 500, 1000)
- `shape_rtl`: Right-to-left text (Arabic)
- `shape_complex_scripts`: Arabic, Devanagari, Thai, Hebrew
- `shape_ligatures`: Text with common ligatures (ff, fi, fl, ffi, ffl)
- `shape_bidi`: Mixed LTR/RTL text
- `shape_repeated`: Same text repeatedly (cache test)

### 3. Glyph Renderer Benchmarks

**Location**: `components/glyph_renderer/benches/`

#### glyph_rendering.rs
- `create_renderer`: Renderer creation
- `rasterize_sizes`: Rendering at 12pt, 16pt, 24pt, 48pt, 72pt, 96pt, 144pt
- `render_modes`: Mono, Gray, SubpixelRGB
- `batch_rendering`: 10, 50, 100, 500 glyphs in batch
- `cache_hits`: Repeated rendering (cache performance)
- `get_outline`: Vector outline extraction
- `render_dpi`: Rendering at 72, 96, 144, 192, 300 DPI

#### cache_performance.rs
- `cache_scalability`: Cache with 10, 50, 100, 500, 1000 entries
- `cache_hit_rate`: Working set size vs cache capacity
- `cache_random_access`: Random glyph access pattern
- `cache_sequential`: Sequential access (best case)
- `cache_mixed_sizes`: Multiple font sizes in cache
- `cache_mixed_modes`: Multiple render modes in cache
- `cache_thrashing`: Working set exceeding cache capacity
- `cache_optimal`: Small working set (ideal case)

### 4. Font System API Benchmarks

**Location**: `components/font_system_api/benches/`

#### end_to_end.rs
- `create_font_system`: System initialization
- `cold_start_workflow`: Full workflow from scratch
- `warm_workflow`: Pre-initialized system
- `full_page_render`: 100, 500, 1000, 5000 characters
- `typical_scenarios`: Short label, button, paragraph, heading
- `font_switching`: Switching between multiple fonts
- `multi_font_text`: Mixed scripts (font fallback)
- `repeated_rendering`: Cache utilization
- `realistic_content`: Document with headings, paragraphs, lists

## Hardware Specifications

### Recording Baseline Measurements

Always include hardware specifications with benchmark results:

```bash
# Generate hardware info
cat > benchmark-env.txt << EOF
System: $(uname -a)
CPU: $(lscpu | grep "Model name" | sed 's/Model name:\s*//')
Cores: $(nproc)
Memory: $(free -h | grep Mem | awk '{print $2}')
Rust: $(rustc --version)
Date: $(date -Iseconds)
Build: Release (--release)
EOF

# Include in benchmark report
cat benchmark-env.txt
cargo bench --workspace
```

### Reference Hardware

Example baseline measurements should specify:

```
System: Linux 5.15.0-x86_64
CPU: Intel Core i7-9700K @ 3.60GHz
Cores: 8
Memory: 32GB DDR4-3200
Rust: rustc 1.75.0
Build: Release
```

## Interpreting Results

### Understanding Criterion Output

```
font_loading/load_font_file
                        time:   [45.234 ms 45.891 ms 46.612 ms]
                        change: [-2.1234% +0.1234% +2.3456%] (p = 0.89)
```

**Reading**:
- **time**: [lower bound, estimate, upper bound] at 95% confidence
- **change**: Percent change from previous run
- **p-value**: Statistical significance (< 0.05 is significant)

### Performance Indicators

**Good Signs**:
- Low variance (tight bounds)
- Consistent measurements across runs
- Linear scaling with input size
- High cache hit rates
- Sub-linear growth with complexity

**Warning Signs**:
- High variance (wide bounds)
- Non-linear scaling where linear expected
- Cache thrashing (performance cliff)
- Memory allocation spikes
- Unexpected performance cliffs

### Comparing Baselines

```bash
# Create baseline for version
cargo bench --workspace -- --save-baseline v0.1.0

# After optimization
cargo bench --workspace -- --baseline v0.1.0

# Look for:
# - Green numbers (improvement)
# - Red numbers (regression)
# - Statistical significance (p < 0.05)
```

## Baseline Measurements

### Initial Baselines (v0.1.0)

**Note**: These are target estimates. Run benchmarks to establish actual baselines for your hardware.

#### Font Registry

```
Operation               Target      Critical    Notes
------------------------------------------------------------
create_registry         < 100μs     < 500μs     Empty registry
load_font_file          < 1ms       < 5ms       Single font
match_font_default      < 100μs     < 1ms       Empty registry
match_font_specific     < 500μs     < 5ms       100 fonts loaded
registry_scalability:
  10 fonts             < 10ms      < 50ms
  100 fonts            < 100ms     < 200ms
  1000 fonts           < 1s        < 2s
```

#### Text Shaper

```
Operation               Target      Critical    Notes
------------------------------------------------------------
create_shaper           < 10μs      < 100μs     Minimal overhead
shape_ascii/short       < 100μs     < 1ms       5 characters
shape_ascii/medium      < 1ms       < 5ms       44 characters
shape_ascii/long        < 5ms       < 10ms      128 characters
shape_rtl               < 5ms       < 15ms      Arabic text
shape_ligatures         < 2ms       < 10ms      English with ligatures
shape_bidi              < 10ms      < 25ms      Mixed LTR/RTL
```

#### Glyph Renderer

```
Operation               Target      Critical    Notes
------------------------------------------------------------
create_renderer         < 10μs      < 100μs     Minimal overhead
rasterize_glyph@12pt    < 5ms       < 10ms      Cache miss
rasterize_glyph@96pt    < 20ms      < 50ms      Large glyph
cache_hit               < 100μs     < 500μs     Cached glyph
batch_rendering/100     < 500ms     < 1s        100 glyphs
cache_stats             < 10μs      < 100μs     O(1) operation
```

#### Font System API

```
Operation               Target      Critical    Notes
------------------------------------------------------------
create_font_system      < 1ms       < 5ms       Full initialization
cold_start_workflow     < 50ms      < 100ms     Load+shape+render
warm_workflow           < 10ms      < 25ms      Pre-initialized
full_page_render/1000   < 100ms     < 250ms     1000 characters
typical_scenarios:
  button_text           < 5ms       < 10ms
  paragraph             < 20ms      < 50ms
```

### Regression Thresholds

Flag performance regressions when:
- Any operation exceeds **critical** threshold
- Any operation regresses by > **10%** from baseline
- Variance increases by > **20%** from baseline
- p-value indicates significant change (p < 0.05)

### Optimization Priorities

1. **Critical Path**: Operations in `font_system_api` end-to-end benchmarks
2. **Hot Loops**: Text shaping and glyph rendering
3. **Scalability**: Registry with large font databases
4. **Cache Efficiency**: Glyph cache hit rates

### Continuous Monitoring

```bash
# Add to CI pipeline
cargo bench --workspace -- --noplot --save-baseline ci-$(git rev-parse --short HEAD)

# Compare against main branch
git checkout main
cargo bench --workspace -- --noplot --save-baseline main
git checkout -
cargo bench --workspace -- --baseline main
```

## Further Reading

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph Profiling](https://github.com/flamegraph-rs/flamegraph)

## Version History

- **v0.1.0** (2025-11-14): Initial baseline targets and benchmark suite
