//! Benchmarks for memory pool and GPU cache performance (FEAT-047, FEAT-045)
//!
//! Tests the performance of memory pools, GPU glyph cache,
//! and allocation patterns.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glyph_renderer::cache::{GpuCacheConfig, GpuCacheKey, GpuGlyphCache};
use glyph_renderer::pool::{BufferPool, GlyphArena, MemoryPool, PoolConfig};
use glyph_renderer::types::{GlyphBitmap, GlyphId, RenderMode};

/// Create a test bitmap of specified size
fn create_test_bitmap(width: u32, height: u32) -> GlyphBitmap {
    GlyphBitmap {
        width,
        height,
        left: 0,
        top: height as i32,
        pitch: width as usize,
        data: vec![128u8; (width * height) as usize],
        format: RenderMode::Gray,
    }
}

/// Benchmark memory pool acquire/release cycle
fn bench_memory_pool_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pool_cycle");

    for pool_size in [64, 256, 1024].iter() {
        group.throughput(Throughput::Elements(*pool_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(pool_size),
            pool_size,
            |b, &size| {
                let pool: MemoryPool<Vec<u8>> = MemoryPool::with_config(PoolConfig {
                    initial_capacity: size,
                    max_pool_size: size * 2,
                    pre_allocate: true,
                });

                b.iter(|| {
                    for _ in 0..100 {
                        let item = pool.acquire();
                        black_box(&*item);
                        // Item returns to pool on drop
                    }
                });
            },
        );
    }
    group.finish();
}

/// Benchmark memory pool vs direct allocation
fn bench_pool_vs_direct(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_vs_direct");

    // Pooled allocation
    group.bench_function("pooled", |b| {
        let pool: MemoryPool<Vec<u8>> = MemoryPool::new();

        b.iter(|| {
            for _ in 0..100 {
                let mut item = pool.acquire();
                item.resize(4096, 0);
                black_box(&*item);
            }
        });
    });

    // Direct allocation
    group.bench_function("direct", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut vec: Vec<u8> = Vec::with_capacity(4096);
                vec.resize(4096, 0);
                black_box(&vec);
            }
        });
    });

    group.finish();
}

/// Benchmark buffer pool with different sizes
fn bench_buffer_pool_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool_sizes");

    let sizes = [
        ("small_1k", 1024),
        ("medium_16k", 16 * 1024),
        ("large_128k", 128 * 1024),
    ];

    for (name, size) in sizes.iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), size, |b, &sz| {
            let pool = BufferPool::new();

            b.iter(|| {
                let mut buf = pool.acquire(sz);
                buf.extend_from_slice(&vec![0u8; sz]);
                black_box(&*buf);
            });
        });
    }
    group.finish();
}

/// Benchmark glyph arena allocation
fn bench_arena_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_allocation");

    for alloc_size in [64, 256, 1024, 4096].iter() {
        group.throughput(Throughput::Bytes(*alloc_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(alloc_size),
            alloc_size,
            |b, &size| {
                let arena = GlyphArena::new();

                b.iter(|| {
                    for _ in 0..100 {
                        let slice = arena.allocate_bytes(size);
                        black_box(slice);
                    }
                    arena.reset();
                });
            },
        );
    }
    group.finish();
}

/// Benchmark arena vs direct allocation
fn bench_arena_vs_direct(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_vs_direct");

    // Arena allocation
    group.bench_function("arena", |b| {
        let arena = GlyphArena::new();

        b.iter(|| {
            for _ in 0..1000 {
                let slice = arena.allocate_bytes(256);
                black_box(slice);
            }
            arena.reset();
        });
    });

    // Direct allocation
    group.bench_function("direct", |b| {
        b.iter(|| {
            let mut allocations = Vec::with_capacity(1000);
            for _ in 0..1000 {
                let vec = vec![0u8; 256];
                allocations.push(vec);
            }
            black_box(&allocations);
        });
    });

    group.finish();
}

/// Benchmark GPU cache insert and lookup
fn bench_gpu_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_cache_operations");

    // Insert benchmark
    group.bench_function("insert", |b| {
        let cache = GpuGlyphCache::new();
        let bitmap = create_test_bitmap(32, 32);
        let mut i = 0u16;

        b.iter(|| {
            let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
            cache.insert(key, &bitmap);
            i = i.wrapping_add(1);
        });
    });

    // Lookup benchmark (cache hit)
    group.bench_function("lookup_hit", |b| {
        let cache = GpuGlyphCache::new();
        let bitmap = create_test_bitmap(32, 32);

        // Pre-populate
        for i in 0..100 {
            let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
            cache.insert(key, &bitmap);
        }

        b.iter(|| {
            for i in 0..100 {
                let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
                let result = cache.get(&key);
                black_box(result);
            }
        });
    });

    // Lookup benchmark (cache miss)
    group.bench_function("lookup_miss", |b| {
        let cache = GpuGlyphCache::new();

        b.iter(|| {
            for i in 0..100 {
                let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
                let result = cache.get(&key);
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark GPU cache with varying glyph sizes
fn bench_gpu_cache_glyph_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_cache_glyph_sizes");

    let sizes = [
        ("small_8x8", 8, 8),
        ("medium_32x32", 32, 32),
        ("large_64x64", 64, 64),
        ("xlarge_128x128", 128, 128),
    ];

    for (name, width, height) in sizes.iter() {
        group.throughput(Throughput::Bytes((*width * *height) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(*width, *height),
            |b, &(w, h)| {
                let cache = GpuGlyphCache::new();
                let bitmap = create_test_bitmap(w, h);

                b.iter(|| {
                    for i in 0..50 {
                        let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
                        cache.get_or_insert(key, &bitmap);
                    }
                });
            },
        );
    }
    group.finish();
}

/// Benchmark GPU cache atlas utilization
fn bench_gpu_cache_atlas_packing(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_cache_atlas_packing");

    // Small atlas (forces multiple atlases)
    group.bench_function("small_atlas", |b| {
        let config = GpuCacheConfig {
            atlas_width: 256,
            atlas_height: 256,
            max_atlases: 8,
            max_cached_glyphs: 1000,
            enable_statistics: false,
        };
        let cache = GpuGlyphCache::with_config(config);
        let bitmap = create_test_bitmap(24, 24);

        b.iter(|| {
            for i in 0..100 {
                let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
                cache.get_or_insert(key, &bitmap);
            }
        });
    });

    // Large atlas (better packing)
    group.bench_function("large_atlas", |b| {
        let config = GpuCacheConfig {
            atlas_width: 2048,
            atlas_height: 2048,
            max_atlases: 2,
            max_cached_glyphs: 1000,
            enable_statistics: false,
        };
        let cache = GpuGlyphCache::with_config(config);
        let bitmap = create_test_bitmap(24, 24);

        b.iter(|| {
            for i in 0..100 {
                let key = GpuCacheKey::new(GlyphId(i), 16.0, RenderMode::Gray);
                cache.get_or_insert(key, &bitmap);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memory_pool_cycle,
    bench_pool_vs_direct,
    bench_buffer_pool_sizes,
    bench_arena_allocation,
    bench_arena_vs_direct,
    bench_gpu_cache_operations,
    bench_gpu_cache_glyph_sizes,
    bench_gpu_cache_atlas_packing,
);
criterion_main!(benches);
