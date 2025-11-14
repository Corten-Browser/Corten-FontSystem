//! Benchmarks for font_parser
//! Benchmarks will be added as implementation progresses

use criterion::{criterion_group, criterion_main, Criterion};

fn placeholder_benchmark(c: &mut Criterion) {
    c.bench_function("placeholder", |b| {
        b.iter(|| {
            // Placeholder benchmark
            1 + 1
        })
    });
}

criterion_group!(benches, placeholder_benchmark);
criterion_main!(benches);
