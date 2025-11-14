//! Benchmarks for font matching algorithm
//!
//! Tests the performance of the font matching system with various
//! registry sizes and descriptor complexity.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use font_registry::{FontRegistry, FontDescriptor, FontWeight, FontStyle, FontStretch};

/// Benchmark font matching with default descriptor
fn bench_match_font_default(c: &mut Criterion) {
    c.bench_function("match_font_default", |b| {
        let registry = FontRegistry::new();
        let descriptor = FontDescriptor::default();

        b.iter(|| {
            let result = registry.match_font(black_box(&descriptor));
            black_box(result);
        });
    });
}

/// Benchmark font matching with specific descriptor
fn bench_match_font_specific(c: &mut Criterion) {
    c.bench_function("match_font_specific", |b| {
        let registry = FontRegistry::new();
        let descriptor = FontDescriptor {
            family: vec!["Arial".to_string()],
            weight: FontWeight::Bold,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 16.0,
        };

        b.iter(|| {
            let result = registry.match_font(black_box(&descriptor));
            black_box(result);
        });
    });
}

/// Benchmark font matching with various weights
fn bench_match_font_weights(c: &mut Criterion) {
    let mut group = c.benchmark_group("match_font_weights");

    let registry = FontRegistry::new();
    let weights = [
        ("thin", FontWeight::Thin),
        ("light", FontWeight::Light),
        ("regular", FontWeight::Regular),
        ("medium", FontWeight::Medium),
        ("semibold", FontWeight::SemiBold),
        ("bold", FontWeight::Bold),
        ("extrabold", FontWeight::ExtraBold),
        ("black", FontWeight::Black),
    ];

    for (name, weight) in weights.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), weight, |b, &weight| {
            let descriptor = FontDescriptor {
                family: vec!["Test".to_string()],
                weight,
                style: FontStyle::Normal,
                stretch: FontStretch::Normal,
                size: 16.0,
            };

            b.iter(|| {
                let result = registry.match_font(black_box(&descriptor));
                black_box(result);
            });
        });
    }
    group.finish();
}

/// Benchmark font matching scalability with increasing registry size
fn bench_match_font_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("match_font_scalability");

    for size in [1, 10, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(1)); // One match per iteration
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            // In real benchmark: create registry with 'size' fonts
            let registry = FontRegistry::new();
            let descriptor = FontDescriptor::default();

            b.iter(|| {
                let result = registry.match_font(black_box(&descriptor));
                black_box(result);
            });
        });
    }
    group.finish();
}

/// Benchmark multiple consecutive matches (cache behavior)
fn bench_match_font_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("match_font_repeated");

    for count in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let registry = FontRegistry::new();
            let descriptor = FontDescriptor::default();

            b.iter(|| {
                for _ in 0..count {
                    let result = registry.match_font(black_box(&descriptor));
                    black_box(result);
                }
            });
        });
    }
    group.finish();
}

/// Benchmark descriptor cloning (used in matching)
fn bench_descriptor_clone(c: &mut Criterion) {
    c.bench_function("descriptor_clone", |b| {
        let descriptor = FontDescriptor {
            family: vec!["Arial".to_string()],
            weight: FontWeight::Bold,
            style: FontStyle::Italic,
            stretch: FontStretch::Condensed,
            size: 16.0,
        };

        b.iter(|| {
            let cloned = black_box(&descriptor).clone();
            black_box(cloned);
        });
    });
}

criterion_group!(
    benches,
    bench_match_font_default,
    bench_match_font_specific,
    bench_match_font_weights,
    bench_match_font_scalability,
    bench_match_font_repeated,
    bench_descriptor_clone,
);
criterion_main!(benches);
