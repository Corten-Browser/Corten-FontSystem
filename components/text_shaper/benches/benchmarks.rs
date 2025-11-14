//! Benchmarks for text shaping operations
//!
//! Tests the performance of text shaping with various text lengths,
//! scripts, and complexity levels.
//!
//! Note: These benchmarks currently measure overhead with stub implementations.
//! They will measure actual performance once text shaping is fully implemented.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use text_shaper::{TextShaper, ShapingOptions, Language, Script};
use font_registry::FontRegistry;
use std::collections::HashMap;

/// Create default shaping options for benchmarking
fn default_shaping_options() -> ShapingOptions {
    ShapingOptions {
        script: Script::Latin,
        language: Language { tag: "en-US".to_string() },
        direction: font_types::types::Direction::LeftToRight,
        features: HashMap::new(),
        kerning: true,
        ligatures: false,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    }
}

/// Benchmark creating a text shaper
fn bench_create_shaper(c: &mut Criterion) {
    c.bench_function("create_shaper", |b| {
        let registry = FontRegistry::new();
        b.iter(|| {
            let shaper = TextShaper::new(black_box(&registry));
            black_box(shaper);
        });
    });
}

/// Benchmark shaping simple ASCII text
fn bench_shape_ascii(c: &mut Criterion) {
    let mut group = c.benchmark_group("shape_ascii");

    let texts = [
        ("short", "Hello"),
        ("medium", "The quick brown fox jumps over the lazy dog"),
        ("long", "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."),
        ("very_long", &"The quick brown fox jumps over the lazy dog. ".repeat(10)),
    ];

    for (name, text) in texts.iter() {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), text, |b, &text| {
            let registry = FontRegistry::new();
            let shaper = TextShaper::new(&registry);
            let options = default_shaping_options();
            let font_id: font_types::types::FontId = 0; // Stub font ID (type alias)
            let size = 16.0;

            b.iter(|| {
                // Note: Will return error with empty registry, but measures overhead
                let _ = shaper.shape_text(
                    black_box(text),
                    black_box(font_id),
                    black_box(size),
                    black_box(&options),
                );
            });
        });
    }
    group.finish();
}

/// Benchmark shaping with different character counts
fn bench_shape_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("shape_scalability");

    for char_count in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*char_count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(char_count), char_count, |b, &count| {
            let text: String = "a".repeat(count);
            let registry = FontRegistry::new();
            let shaper = TextShaper::new(&registry);
            let options = default_shaping_options();
            let font_id: font_types::types::FontId = 0;
            let size = 16.0;

            b.iter(|| {
                let _ = shaper.shape_text(
                    black_box(&text),
                    black_box(font_id),
                    black_box(size),
                    black_box(&options),
                );
            });
        });
    }
    group.finish();
}

/// Benchmark shaping options creation
fn bench_create_options(c: &mut Criterion) {
    c.bench_function("create_shaping_options", |b| {
        b.iter(|| {
            let options = default_shaping_options();
            black_box(options);
        });
    });
}

/// Benchmark repeated shaping (cache behavior)
fn bench_shape_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("shape_repeated");

    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let text = "The quick brown fox";
            let registry = FontRegistry::new();
            let shaper = TextShaper::new(&registry);
            let options = default_shaping_options();
            let font_id: font_types::types::FontId = 0;
            let size = 16.0;

            b.iter(|| {
                for _ in 0..count {
                    let _ = shaper.shape_text(
                        black_box(text),
                        black_box(font_id),
                        black_box(size),
                        black_box(&options),
                    );
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_create_shaper,
    bench_create_options,
    bench_shape_ascii,
    bench_shape_scalability,
    bench_shape_repeated,
);
criterion_main!(benches);
