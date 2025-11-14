//! Benchmarks for glyph rasterization operations
//!
//! Tests the performance of glyph rendering at various sizes,
//! modes, and with different fonts.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glyph_renderer::GlyphRenderer;
use glyph_renderer::types::{RenderMode, GlyphId, OpenTypeFont};

/// Create a stub font for benchmarking
fn create_stub_font() -> OpenTypeFont {
    // Use from_data constructor since fields are private
    OpenTypeFont::from_data(vec![0; 1024], 0)
}

/// Benchmark creating a glyph renderer
fn bench_create_renderer(c: &mut Criterion) {
    c.bench_function("create_renderer", |b| {
        b.iter(|| {
            let renderer = GlyphRenderer::new();
            black_box(renderer);
        });
    });
}

/// Benchmark glyph rasterization at different sizes
fn bench_rasterize_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("rasterize_sizes");

    let sizes = [12.0, 16.0, 24.0, 48.0, 72.0, 96.0, 144.0];

    for size in sizes.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();
            let glyph_id = GlyphId(65); // 'A'

            b.iter(|| {
                // Note: This will fail with stub font, but measures overhead
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(glyph_id),
                    black_box(size),
                    black_box(RenderMode::Gray),
                );
            });
        });
    }
    group.finish();
}

/// Benchmark different render modes
fn bench_render_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_modes");

    let modes = [
        ("mono", RenderMode::Mono),
        ("gray", RenderMode::Gray),
        ("subpixel", RenderMode::SubpixelRgb),
    ];

    for (name, mode) in modes.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), mode, |b, &mode| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();
            let glyph_id = GlyphId(65);

            b.iter(|| {
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(glyph_id),
                    black_box(16.0),
                    black_box(mode),
                );
            });
        });
    }
    group.finish();
}

/// Benchmark batch glyph rendering
fn bench_batch_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_rendering");

    for count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();

            b.iter(|| {
                for i in 0..count {
                    let glyph_id = GlyphId((65 + i % 26) as u16); // A-Z
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

/// Benchmark cache hit performance (same glyph repeatedly)
fn bench_cache_hits(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hits");

    for count in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();
            let glyph_id = GlyphId(65);

            // First render to populate cache (will fail but setup is measured)
            let _ = renderer.rasterize_glyph(&font, glyph_id, 16.0, RenderMode::Gray);

            b.iter(|| {
                for _ in 0..count {
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

/// Benchmark outline extraction
fn bench_get_outline(c: &mut Criterion) {
    c.bench_function("get_glyph_outline", |b| {
        let renderer = GlyphRenderer::new();
        let font = create_stub_font();
        let glyph_id = GlyphId(65);

        b.iter(|| {
            let _ = renderer.get_glyph_outline(
                black_box(&font),
                black_box(glyph_id),
            );
        });
    });
}

/// Benchmark cache statistics retrieval
fn bench_cache_stats(c: &mut Criterion) {
    c.bench_function("cache_stats", |b| {
        let renderer = GlyphRenderer::new();

        b.iter(|| {
            let stats = renderer.cache_stats();
            black_box(stats);
        });
    });
}

/// Benchmark cache clearing
fn bench_cache_clear(c: &mut Criterion) {
    c.bench_function("cache_clear", |b| {
        let mut renderer = GlyphRenderer::new();
        let font = create_stub_font();

        // Pre-populate cache
        for i in 0..100 {
            let _ = renderer.rasterize_glyph(&font, GlyphId(i), 16.0, RenderMode::Gray);
        }

        b.iter(|| {
            renderer.clear_cache();
        });
    });
}

/// Benchmark rendering at various DPIs
fn bench_render_dpi(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_dpi");

    // DPI affects size: size_at_dpi = (size_points * dpi) / 72
    let dpis = [72, 96, 144, 192, 300];

    for dpi in dpis.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(dpi), dpi, |b, &dpi| {
            let mut renderer = GlyphRenderer::new();
            let font = create_stub_font();
            let glyph_id = GlyphId(65);

            // Adjust size for DPI
            let size = (12.0 * dpi as f32) / 72.0;

            b.iter(|| {
                let _ = renderer.rasterize_glyph(
                    black_box(&font),
                    black_box(glyph_id),
                    black_box(size),
                    black_box(RenderMode::Gray),
                );
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_create_renderer,
    bench_rasterize_sizes,
    bench_render_modes,
    bench_batch_rendering,
    bench_cache_hits,
    bench_get_outline,
    bench_cache_stats,
    bench_cache_clear,
    bench_render_dpi,
);
criterion_main!(benches);
