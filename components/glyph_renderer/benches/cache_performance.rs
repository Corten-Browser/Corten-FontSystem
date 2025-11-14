//! Benchmarks for glyph cache performance
//!
//! Tests cache hit rates, memory usage, and cache eviction performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glyph_renderer::GlyphRenderer;
use glyph_renderer::types::{RenderMode, GlyphId, OpenTypeFont};

/// Create a stub font for benchmarking
fn create_stub_font() -> OpenTypeFont {
    // Use from_data constructor since fields are private
    OpenTypeFont::from_data(vec![0; 1024], 0)
}

/// Benchmark cache performance with varying cache sizes
fn bench_cache_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_scalability");

    for cache_size in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*cache_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(cache_size), cache_size, |b, &size| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();

            // Pre-populate cache with 'size' glyphs
            for i in 0..size {
                let _ = renderer.rasterize_glyph(
                    &font,
                    GlyphId(i as u16 % 256),
                    16.0,
                    RenderMode::Gray,
                );
            }

            b.iter(|| {
                let stats = renderer.cache_stats();
                black_box(stats);
            });
        });
    }
    group.finish();
}

/// Benchmark cache hit rate with working set
fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");

    // Different working set sizes vs cache size
    for working_set in [10, 50, 100, 200].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(working_set), working_set, |b, &size| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();

            b.iter(|| {
                // Access glyphs in working set pattern
                for i in 0..1000 {
                    let glyph_id = GlyphId((i % size) as u16);
                    let _ = renderer.rasterize_glyph(
                        black_box(&font),
                        black_box(glyph_id),
                        black_box(16.0),
                        black_box(RenderMode::Gray),
                    );
                }
            });
        });
    }
    group.finish();
}

/// Benchmark cache with random access pattern
fn bench_cache_random_access(c: &mut Criterion) {
    c.bench_function("cache_random_access", |b| {
        let mut renderer = GlyphRenderer::new();
        let font = create_stub_font();

        // Pseudo-random sequence (deterministic for benchmarking)
        let mut lfsr = 0xACE1u16;

        b.iter(|| {
            for _ in 0..1000 {
                // Linear feedback shift register for pseudo-random numbers
                let bit = ((lfsr >> 0) ^ (lfsr >> 2) ^ (lfsr >> 3) ^ (lfsr >> 5)) & 1;
                lfsr = (lfsr >> 1) | (bit << 15);

                let glyph_id = GlyphId(lfsr % 256);
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(glyph_id),
                    black_box(16.0),
                    black_box(RenderMode::Gray),
                );
            }
        });
    });
}

/// Benchmark cache with sequential access (best case)
fn bench_cache_sequential(c: &mut Criterion) {
    c.bench_function("cache_sequential", |b| {
        let mut renderer = GlyphRenderer::new();
        let font = create_stub_font();

        b.iter(|| {
            // Sequential access through glyphs
            for i in 0..256 {
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(GlyphId(i)),
                    black_box(16.0),
                    black_box(RenderMode::Gray),
                );
            }
        });
    });
}

/// Benchmark cache with size variations (mixed sizes in cache)
fn bench_cache_mixed_sizes(c: &mut Criterion) {
    c.bench_function("cache_mixed_sizes", |b| {
        let mut renderer = GlyphRenderer::new();
        let font = create_stub_font();
        let sizes = [12.0, 16.0, 24.0, 48.0];

        b.iter(|| {
            for i in 0..100 {
                let size = sizes[i % sizes.len()];
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(GlyphId((i % 26 + 65) as u16)), // A-Z
                    black_box(size),
                    black_box(RenderMode::Gray),
                );
            }
        });
    });
}

/// Benchmark cache with different render modes (mode affects caching)
fn bench_cache_mixed_modes(c: &mut Criterion) {
    c.bench_function("cache_mixed_modes", |b| {
        let mut renderer = GlyphRenderer::new();
        let font = create_stub_font();
        let modes = [RenderMode::Mono, RenderMode::Gray, RenderMode::SubpixelRgb];

        b.iter(|| {
            for i in 0..100 {
                let mode = modes[i % modes.len()];
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(GlyphId(65)), // Always 'A'
                    black_box(16.0),
                    black_box(mode),
                );
            }
        });
    });
}

/// Benchmark cache statistics overhead
fn bench_cache_stats_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_stats_overhead");

    for cache_size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(cache_size), cache_size, |b, &size| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();

            // Pre-populate cache
            for i in 0..size {
                let _ = renderer.rasterize_glyph(
                    &font,
                    GlyphId((i % 256) as u16),
                    16.0,
                    RenderMode::Gray,
                );
            }

            b.iter(|| {
                let stats = renderer.cache_stats();
                black_box(stats);
            });
        });
    }
    group.finish();
}

/// Benchmark cache thrashing (working set > cache capacity)
fn bench_cache_thrashing(c: &mut Criterion) {
    c.bench_function("cache_thrashing", |b| {
        let mut renderer = GlyphRenderer::new();
        let font = create_stub_font();

        b.iter(|| {
            // Access more glyphs than typical cache can hold
            for i in 0..1000 {
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(GlyphId(i as u16 % 512)),
                    black_box(16.0),
                    black_box(RenderMode::Gray),
                );
            }
        });
    });
}

/// Benchmark optimal cache usage (repeated small working set)
fn bench_cache_optimal(c: &mut Criterion) {
    c.bench_function("cache_optimal", |b| {
        let mut renderer = GlyphRenderer::new();
        let font = create_stub_font();

        b.iter(|| {
            // Small working set that fits in cache
            for _ in 0..1000 {
                for i in 0..26 {
                    // A-Z repeatedly
                    let _ = renderer.rasterize_glyph(
                        black_box(&font),
                        black_box(GlyphId(65 + i)),
                        black_box(16.0),
                        black_box(RenderMode::Gray),
                    );
                }
            }
        });
    });
}

criterion_group!(
    benches,
    bench_cache_scalability,
    bench_cache_hit_rate,
    bench_cache_random_access,
    bench_cache_sequential,
    bench_cache_mixed_sizes,
    bench_cache_mixed_modes,
    bench_cache_stats_overhead,
    bench_cache_thrashing,
    bench_cache_optimal,
);
criterion_main!(benches);
