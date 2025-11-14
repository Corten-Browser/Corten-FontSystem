//! End-to-end benchmarks for the complete font system
//!
//! Tests full workflows from font loading through text shaping to rendering.
//!
//! Note: These benchmarks currently measure overhead with stub implementations.
//! They will measure actual performance once the full system is implemented.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use font_system_api::{FontSystem, FontSystemConfig};

/// Benchmark creating a font system
fn bench_create_font_system(c: &mut Criterion) {
    c.bench_function("create_font_system", |b| {
        b.iter(|| {
            let config = FontSystemConfig::default();
            let system = FontSystem::new(config);
            black_box(system);
        });
    });
}

/// Benchmark cold start: create system + load font + shape + render
fn bench_cold_start_workflow(c: &mut Criterion) {
    c.bench_function("cold_start_workflow", |b| {
        b.iter(|| {
            // Create system
            let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");

            // In real benchmark: load a font
            // system.load_font_file(font_path)

            // In real benchmark: shape text
            // let shaped = system.shape_text("Hello World", &options)

            // In real benchmark: render glyphs
            // for glyph in shaped.glyphs {
            //     system.render_glyph(glyph)
            // }

            black_box(system);
        });
    });
}

/// Benchmark warm workflow: system already initialized
fn bench_warm_workflow(c: &mut Criterion) {
    c.bench_function("warm_workflow", |b| {
        // Pre-create system (amortize initialization)
        let config = FontSystemConfig::default();
        let mut system = FontSystem::new(config).expect("Failed to create system");

        b.iter(|| {
            // In real benchmark: shape and render (font already loaded)
            // let shaped = system.shape_text("Hello World", &options)
            // for glyph in shaped.glyphs {
            //     system.render_glyph(glyph)
            // }

            black_box(&system);
        });
    });
}

/// Benchmark rendering a full page of text
fn bench_full_page_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_page_render");

    for char_count in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*char_count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(char_count), char_count, |b, &count| {
            let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");
            let text: String = "a".repeat(count);

            b.iter(|| {
                // In real benchmark: shape entire text
                // let shaped = system.shape_text(&text, &options)

                // In real benchmark: render all glyphs
                // for glyph in shaped.glyphs {
                //     system.render_glyph(glyph)
                // }

                black_box(&system);
                black_box(&text);
            });
        });
    }
    group.finish();
}

/// Benchmark typical text rendering scenarios
fn bench_typical_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("typical_scenarios");

    let scenarios = [
        ("short_label", "OK"),
        ("button_text", "Submit Form"),
        ("paragraph", "The quick brown fox jumps over the lazy dog. This is a typical paragraph of text."),
        ("heading", "Main Application Title"),
    ];

    for (name, text) in scenarios.iter() {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), text, |b, &text| {
            let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");

            b.iter(|| {
                // In real benchmark: shape and render
                // let shaped = system.shape_text(text, &options)
                // for glyph in shaped.glyphs {
                //     system.render_glyph(glyph)
                // }

                black_box(&system);
                black_box(text);
            });
        });
    }
    group.finish();
}

/// Benchmark font switching overhead
fn bench_font_switching(c: &mut Criterion) {
    c.bench_function("font_switching", |b| {
        let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");

        b.iter(|| {
            // In real benchmark: switch between fonts
            // for font in [font_a, font_b, font_c] {
            //     system.set_current_font(font)
            //     system.shape_text("Test", &options)
            // }

            black_box(&system);
        });
    });
}

/// Benchmark multi-font text (font fallback)
fn bench_multi_font_text(c: &mut Criterion) {
    c.bench_function("multi_font_text", |b| {
        let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");
        let text = "English 中文 العربية"; // Mixed scripts

        b.iter(|| {
            // In real benchmark: shape text with font fallback
            // let shaped = system.shape_text(text, &options)

            black_box(&system);
            black_box(text);
        });
    });
}

/// Benchmark repeated rendering (cache utilization)
fn bench_repeated_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_rendering");

    for iterations in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*iterations as u64));
        group.bench_with_input(BenchmarkId::from_parameter(iterations), iterations, |b, &iters| {
            let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");
            let text = "Cached Text";

            b.iter(|| {
                for _ in 0..iters {
                    // In real benchmark: shape and render same text
                    // let shaped = system.shape_text(text, &options)
                    // for glyph in shaped.glyphs {
                    //     system.render_glyph(glyph)
                    // }

                    black_box(&system);
                    black_box(text);
                }
            });
        });
    }
    group.finish();
}

/// Benchmark memory overhead
fn bench_memory_overhead(c: &mut Criterion) {
    c.bench_function("memory_overhead", |b| {
        b.iter(|| {
            // Create multiple font systems to measure per-instance overhead
            let systems: Vec<_> = (0..10).map(|_| {
                let config = FontSystemConfig::default();
                FontSystem::new(config).expect("Failed to create system")
            }).collect();
            black_box(systems);
        });
    });
}

/// Benchmark system cleanup/teardown
fn bench_system_cleanup(c: &mut Criterion) {
    c.bench_function("system_cleanup", |b| {
        b.iter(|| {
            let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");

            // In real benchmark: load fonts and use system
            // system.load_font_file(path)
            // system.shape_text("test", &options)

            // Drop explicitly to measure cleanup
            drop(system);
        });
    });
}

/// Benchmark concurrent rendering (if supported)
fn bench_concurrent_rendering(c: &mut Criterion) {
    c.bench_function("concurrent_rendering", |b| {
        let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");
        let texts = ["Text A", "Text B", "Text C", "Text D"];

        b.iter(|| {
            // In real benchmark: render multiple texts
            // Could use rayon for parallel rendering if system supports it
            for text in &texts {
                // system.shape_text(text, &options)

                black_box(text);
            }

            black_box(&system);
        });
    });
}

/// Benchmark with realistic mixed content
fn bench_realistic_content(c: &mut Criterion) {
    c.bench_function("realistic_content", |b| {
        let config = FontSystemConfig::default();
            let mut system = FontSystem::new(config).expect("Failed to create system");

        // Realistic document with headings, paragraphs, lists
        let content = vec![
            ("heading1", "Document Title", 24.0),
            ("heading2", "Section 1: Introduction", 18.0),
            ("paragraph", "Lorem ipsum dolor sit amet, consectetur adipiscing elit.", 12.0),
            ("paragraph", "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.", 12.0),
            ("heading2", "Section 2: Details", 18.0),
            ("list_item", "• First item", 12.0),
            ("list_item", "• Second item", 12.0),
        ];

        b.iter(|| {
            for (_label, text, _size) in &content {
                // In real benchmark: shape with appropriate size
                // let shaped = system.shape_text(text, &options_with_size(*size))

                black_box(text);
            }

            black_box(&system);
        });
    });
}

criterion_group!(
    benches,
    bench_create_font_system,
    bench_cold_start_workflow,
    bench_warm_workflow,
    bench_full_page_render,
    bench_typical_scenarios,
    bench_font_switching,
    bench_multi_font_text,
    bench_repeated_rendering,
    bench_memory_overhead,
    bench_system_cleanup,
    bench_concurrent_rendering,
    bench_realistic_content,
);
criterion_main!(benches);
