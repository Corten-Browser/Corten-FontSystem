//! Benchmarks for parallel text shaping (FEAT-046)
//!
//! Tests the performance of batch and parallel shaping operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use font_registry::FontRegistry;
use font_types::types::Direction;
use std::collections::HashMap;
use std::sync::Arc;
use text_shaper::{
    Language, ParallelShaper, ParallelShapingConfig, Script, ShapingOptions, SharedBatchShaper,
    TextRun, TextRunBatch, TextShaper,
};

/// Create default shaping options for benchmarking
fn default_shaping_options() -> ShapingOptions {
    ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: "en-US".to_string(),
        },
        direction: Direction::LeftToRight,
        features: HashMap::new(),
        kerning: true,
        ligatures: false,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    }
}

/// Create test text runs
fn create_test_runs(count: usize) -> Vec<TextRun> {
    let options = default_shaping_options();
    let texts = [
        "Hello, World!",
        "The quick brown fox",
        "Lorem ipsum dolor sit amet",
        "Performance testing text",
        "Another sample string",
    ];

    (0..count)
        .map(|i| TextRun::new(texts[i % texts.len()], 0, 16.0, options.clone()))
        .collect()
}

/// Benchmark sequential vs parallel shaping
fn bench_sequential_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_vs_parallel");

    for run_count in [4, 16, 64, 256].iter() {
        let runs = create_test_runs(*run_count);
        group.throughput(Throughput::Elements(*run_count as u64));

        // Sequential shaping
        group.bench_with_input(
            BenchmarkId::new("sequential", run_count),
            &runs,
            |b, runs| {
                let registry = FontRegistry::new();
                let shaper = TextShaper::new(&registry);
                let options = default_shaping_options();

                b.iter(|| {
                    for run in runs {
                        let _ = shaper.shape_text(
                            black_box(&run.text),
                            black_box(run.font_id),
                            black_box(run.size),
                            black_box(&options),
                        );
                    }
                });
            },
        );

        // Parallel shaping
        group.bench_with_input(BenchmarkId::new("parallel", run_count), &runs, |b, runs| {
            let registry = FontRegistry::new();
            let shaper = ParallelShaper::new(&registry);

            b.iter(|| {
                let result = shaper.shape_batch(black_box(runs));
                black_box(result);
            });
        });
    }
    group.finish();
}

/// Benchmark TextRunBatch building
fn bench_batch_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_building");

    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let options = default_shaping_options();

            b.iter(|| {
                let mut batch = TextRunBatch::with_capacity(count);
                for i in 0..count {
                    batch.add(TextRun::new(
                        format!("Text {}", i),
                        0,
                        16.0,
                        options.clone(),
                    ));
                }
                black_box(batch);
            });
        });
    }
    group.finish();
}

/// Benchmark SharedBatchShaper (thread-safe version)
fn bench_shared_batch_shaper(c: &mut Criterion) {
    let mut group = c.benchmark_group("shared_batch_shaper");

    for run_count in [8, 32, 128].iter() {
        let runs = create_test_runs(*run_count);
        group.throughput(Throughput::Elements(*run_count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(run_count), &runs, |b, runs| {
            let registry = Arc::new(FontRegistry::new());
            let shaper = SharedBatchShaper::new(registry);

            b.iter(|| {
                let result = shaper.shape_batch(black_box(runs));
                black_box(result);
            });
        });
    }
    group.finish();
}

/// Benchmark parallel threshold decision
fn bench_parallel_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_threshold");

    // Below threshold (should use sequential)
    group.bench_function("below_threshold", |b| {
        let runs = create_test_runs(2);
        let registry = FontRegistry::new();
        let config = ParallelShapingConfig {
            min_parallel_runs: 4,
            min_parallel_chars: 100,
            num_threads: None,
            enable_caching: true,
        };
        let shaper = ParallelShaper::with_config(&registry, config);

        b.iter(|| {
            let result = shaper.shape_batch(black_box(&runs));
            black_box(result);
        });
    });

    // Above threshold (should use parallel)
    group.bench_function("above_threshold", |b| {
        let runs = create_test_runs(16);
        let registry = FontRegistry::new();
        let config = ParallelShapingConfig {
            min_parallel_runs: 4,
            min_parallel_chars: 100,
            num_threads: None,
            enable_caching: true,
        };
        let shaper = ParallelShaper::with_config(&registry, config);

        b.iter(|| {
            let result = shaper.shape_batch(black_box(&runs));
            black_box(result);
        });
    });

    group.finish();
}

/// Benchmark with varying text lengths
fn bench_varying_text_lengths(c: &mut Criterion) {
    let mut group = c.benchmark_group("varying_text_lengths");
    let options = default_shaping_options();

    let text_configs = [
        ("short_texts", 10, "Hi!"),
        (
            "medium_texts",
            10,
            "The quick brown fox jumps over the lazy dog.",
        ),
        ("long_texts", 10, &"Lorem ipsum dolor sit amet. ".repeat(10)),
    ];

    for (name, count, text_template) in text_configs.iter() {
        let runs: Vec<TextRun> = (0..*count)
            .map(|_| TextRun::new(*text_template, 0, 16.0, options.clone()))
            .collect();

        let total_chars: usize = runs.iter().map(|r| r.text.len()).sum();
        group.throughput(Throughput::Bytes(total_chars as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &runs, |b, runs| {
            let registry = FontRegistry::new();
            let shaper = ParallelShaper::new(&registry);

            b.iter(|| {
                let result = shaper.shape_batch(black_box(runs));
                black_box(result);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_vs_parallel,
    bench_batch_building,
    bench_shared_batch_shaper,
    bench_parallel_threshold,
    bench_varying_text_lengths,
);
criterion_main!(benches);
