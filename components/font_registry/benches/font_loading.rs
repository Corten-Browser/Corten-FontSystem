//! Benchmarks for font loading operations
//!
//! Tests the performance of loading fonts from various sources.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use font_registry::{FontRegistry, FontDescriptor};
use std::path::PathBuf;

/// Benchmark loading a single font file
fn bench_load_font_file(c: &mut Criterion) {
    // Note: In a real benchmark, we'd use actual font files
    // For now, this demonstrates the benchmark structure
    c.bench_function("load_font_file", |b| {
        b.iter(|| {
            let mut registry = FontRegistry::new();
            // Would call: registry.load_font_file(path)
            black_box(registry);
        });
    });
}

/// Benchmark loading font data from memory
fn bench_load_font_data(c: &mut Criterion) {
    c.bench_function("load_font_data", |b| {
        b.iter(|| {
            let mut registry = FontRegistry::new();
            // In real benchmark: registry.load_font_data(data.clone())
            black_box(registry);
        });
    });
}

/// Benchmark creating empty registry (baseline)
fn bench_create_registry(c: &mut Criterion) {
    c.bench_function("create_registry", |b| {
        b.iter(|| {
            let registry = FontRegistry::new();
            black_box(registry);
        });
    });
}

/// Benchmark font descriptor creation
fn bench_create_descriptor(c: &mut Criterion) {
    c.bench_function("create_descriptor", |b| {
        b.iter(|| {
            let descriptor = FontDescriptor::default();
            black_box(descriptor);
        });
    });
}

/// Benchmark registry with varying number of fonts (scalability test)
fn bench_registry_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_scalability");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut registry = FontRegistry::new();
                // In real benchmark: load 'size' fonts
                // This would demonstrate how loading time scales with font count
                for _ in 0..size {
                    black_box(&registry);
                }
                black_box(registry);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_create_registry,
    bench_create_descriptor,
    bench_load_font_file,
    bench_load_font_data,
    bench_registry_scalability,
);
criterion_main!(benches);
