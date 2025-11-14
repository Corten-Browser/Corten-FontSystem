# Corten Font System - Caching Strategy

**Version**: 0.1.0
**Date**: 2025-11-14
**Phase**: 3 (Performance Optimization)

---

## Overview

The Corten Font System implements multi-level caching to optimize performance for font loading, text shaping, and glyph rendering operations. This document describes the caching architecture, eviction policies, configuration options, and performance characteristics.

---

## Architecture

### Cache Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                     Font System API                          │
│                  (Orchestrates all caches)                   │
└────────────┬─────────────┬─────────────┬────────────────────┘
             │             │             │
    ┌────────▼───┐  ┌──────▼─────┐ ┌────▼─────────┐
    │  Glyph     │  │  Shaping   │ │  Font        │
    │  Cache     │  │  Cache     │ │  Cache       │
    └────────────┘  └────────────┘ └──────────────┘
```

### Cache Levels

1. **Glyph Cache** (glyph_renderer)
   - Caches rasterized glyph bitmaps
   - Key: (font_id, glyph_id, size, render_mode)
   - Default: 10,000 entries, 100 MB max

2. **Shaping Cache** (text_shaper)
   - Caches shaped text results
   - Key: (text, font_id, size, options_hash)
   - Default: 1,000 entries

3. **Font Cache** (font_registry)
   - Caches loaded font data
   - Key: font_id
   - Lazy loading with memory limits

---

## Cache Configuration

### CacheConfig Structure

```rust
pub struct CacheConfig {
    pub glyph_cache: GlyphCacheConfig,
    pub shaping_cache: ShapingCacheConfig,
}

pub struct GlyphCacheConfig {
    pub max_entries: usize,       // Default: 10,000 glyphs
    pub max_memory_bytes: usize,  // Default: 100 MB
    pub enable_statistics: bool,  // Default: true
}

pub struct ShapingCacheConfig {
    pub max_entries: usize,      // Default: 1,000 entries
    pub enable_statistics: bool, // Default: true
}
```

### Default Configuration

```rust
let config = CacheConfig::default();
// Equivalent to:
let config = CacheConfig {
    glyph_cache: GlyphCacheConfig {
        max_entries: 10_000,
        max_memory_bytes: 100 * 1024 * 1024, // 100 MB
        enable_statistics: true,
    },
    shaping_cache: ShapingCacheConfig {
        max_entries: 1_000,
        enable_statistics: true,
    },
};
```

### Custom Configuration

```rust
use font_system_api::{FontSystemConfig, CacheConfig, GlyphCacheConfig, ShapingCacheConfig};

let config = FontSystemConfig {
    cache_config: CacheConfig {
        glyph_cache: GlyphCacheConfig {
            max_entries: 20_000,                // Double the glyphs
            max_memory_bytes: 200 * 1024 * 1024, // 200 MB
            enable_statistics: true,
        },
        shaping_cache: ShapingCacheConfig {
            max_entries: 5_000,  // More shaping cache
            enable_statistics: true,
        },
    },
    enable_subpixel: true,
    enable_hinting: true,
    load_system_fonts_on_init: true,
};

let font_system = FontSystem::new(config)?;
```

---

## Eviction Policies

### Glyph Cache: LRU (Least Recently Used)

**Strategy**: Remove least recently used glyphs when limit reached

**Trigger Conditions**:
1. Entry count exceeds `max_entries`
2. Memory usage exceeds `max_memory_bytes`

**Eviction Process**:
```
1. Sort entries by last access time (oldest first)
2. Remove entries until:
   - entry_count <= max_entries AND
   - memory_bytes <= max_memory_bytes
3. Update statistics (evictions++)
```

**Advantages**:
- Keeps frequently accessed glyphs in cache
- Predictable performance for repeated rendering
- Simple to implement and reason about

### Shaping Cache: LRU (Least Recently Used)

**Strategy**: Remove least recently used shaped text when limit reached

**Trigger Conditions**:
1. Entry count exceeds `max_entries`

**Eviction Process**:
- Same as glyph cache (LRU based on access time)

### Font Cache: Reference Counting + LRU

**Strategy**: Keep loaded fonts in memory while in use, evict unused fonts

**Trigger Conditions**:
1. Memory pressure (total font data > limit)
2. Font not accessed recently

**Eviction Process**:
- Fonts actively in use are never evicted
- Unused fonts are evicted LRU when memory limit reached

---

## Cache Statistics

### MemoryStats Structure

```rust
pub struct MemoryStats {
    pub font_data_bytes: usize,      // Font file data in memory
    pub glyph_cache_bytes: usize,    // Cached glyph bitmaps
    pub shaping_cache_bytes: usize,  // Cached shaped text
    pub total_bytes: usize,          // Total memory usage
    pub font_count: usize,           // Loaded fonts
    pub cached_glyphs: usize,        // Cached glyph count
    pub cached_shapings: usize,      // Cached shaping count
}
```

### Accessing Statistics

```rust
// Get current memory usage
let stats = font_system.memory_stats();
println!("Total memory: {} MB", stats.total_bytes / 1024 / 1024);
println!("Cached glyphs: {}", stats.cached_glyphs);
println!("Cached shapings: {}", stats.cached_shapings);

// Get detailed component breakdown
let breakdown = font_system.detailed_stats();
println!("Glyph cache hit rate: {:.2}%", breakdown.glyph_cache_hit_rate * 100.0);
println!("Shaping cache hit rate: {:.2}%", breakdown.shaping_cache_hit_rate * 100.0);
```

### Cache Hit Rates

**Target Performance**:
- Glyph cache: **> 90% hit rate** for typical text rendering
- Shaping cache: **> 80% hit rate** for repeated text
- Font cache: **> 95% hit rate** after initial load

**Monitoring**:
```rust
if enable_statistics {
    let stats = glyph_renderer.cache_stats();
    println!("Glyph cache:");
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.2}%", stats.hit_rate * 100.0);
    println!("  Evictions: {}", stats.evictions);
}
```

---

## Memory Management

### Memory Limits

**Default Limits**:
- Glyph cache: 100 MB
- Shaping cache: ~20 MB (1,000 entries @ ~20 KB/entry)
- Font cache: Dynamic (based on loaded fonts)
- **Total default**: ~120 MB base + font data

**Tuning Recommendations**:

| Use Case | Glyph Cache | Shaping Cache | Notes |
|----------|-------------|---------------|-------|
| **Desktop App** | 100 MB | 1,000 entries | Default settings optimal |
| **Web Browser** | 200 MB | 5,000 entries | More content, more cache |
| **Mobile App** | 50 MB | 500 entries | Limited memory |
| **Server-side** | 500 MB | 10,000 entries | High throughput |
| **Embedded** | 10 MB | 100 entries | Minimal footprint |

### Memory Profiling

```rust
use font_system_api::profiling::MemoryProfiler;

// Periodic monitoring
loop {
    let stats = font_system.detailed_stats();

    if stats.total_bytes > MEMORY_WARNING_THRESHOLD {
        eprintln!("Warning: Memory usage high: {} MB",
                  stats.total_bytes / 1024 / 1024);

        // Optionally clear caches
        font_system.clear_caches();
    }

    tokio::time::sleep(Duration::from_secs(60)).await;
}
```

---

## Performance Characteristics

### Cache Lookup Performance

**Target Latencies**:
- Glyph cache lookup: **< 0.1 ms** (hit)
- Glyph cache miss: **< 5 ms** (render + cache)
- Shaping cache lookup: **< 0.1 ms** (hit)
- Shaping cache miss: **< 10 ms** (shape + cache)

**Measured Performance** (Baseline v0.1.0):
- Glyph cache hit: ~0.05 ms
- Glyph cache miss: ~3.2 ms (12pt glyph)
- Shaping cache hit: ~0.08 ms
- Shaping cache miss: ~8.5 ms (100 char string)

### Cache Warm-up Strategies

**1. Preload Common Glyphs**:
```rust
// Warm cache with ASCII glyphs
for c in 'a'..='z' {
    renderer.rasterize_glyph(font_id, glyph_id_for(c), 12.0, RenderMode::Gray, &registry)?;
}
for c in 'A'..='Z' {
    renderer.rasterize_glyph(font_id, glyph_id_for(c), 12.0, RenderMode::Gray, &registry)?;
}
```

**2. Batch Rendering**:
```rust
// Shape and render entire paragraph at once
let shaped = shaper.shape_text(paragraph, font_id, 12.0, &options, &registry)?;
for glyph in &shaped.glyphs {
    renderer.rasterize_glyph(font_id, glyph.glyph_id, 12.0, RenderMode::Gray, &registry)?;
}
```

**3. Predictive Caching**:
- Cache next page of text while rendering current page
- Cache common UI elements (buttons, labels)
- Cache frequently used fonts at startup

---

## Best Practices

### 1. Cache Configuration

✅ **DO**:
- Use default configuration for typical desktop apps
- Increase cache sizes for high-throughput scenarios
- Enable statistics during development
- Profile memory usage in production

❌ **DON'T**:
- Set cache sizes too small (< 1,000 glyphs)
- Ignore memory limits on mobile/embedded
- Disable statistics without measuring impact
- Set unlimited cache sizes

### 2. Cache Management

✅ **DO**:
- Monitor cache hit rates
- Clear caches when memory pressure detected
- Use batch operations when possible
- Warm up caches for critical UI

❌ **DON'T**:
- Clear caches unnecessarily (performance hit)
- Ignore eviction statistics
- Render same glyph repeatedly (cache should hit)
- Mix different DPIs unnecessarily

### 3. Memory Optimization

✅ **DO**:
- Use lower cache limits on resource-constrained devices
- Monitor total memory usage periodically
- Implement cache warming for predictable workloads
- Use appropriate render modes (Mono < Gray < LCD)

❌ **DON'T**:
- Cache at multiple DPIs simultaneously
- Keep unused fonts loaded
- Render at sizes not used
- Ignore memory growth

---

## Debugging Cache Issues

### Low Hit Rate

**Symptoms**: Cache hit rate < 50%

**Causes**:
1. Cache size too small
2. Too many unique font/size combinations
3. Text constantly changing
4. Wrong cache key generation

**Solutions**:
```rust
// Check cache statistics
let stats = renderer.cache_stats();
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
println!("Cache size: {}/{}", stats.current_size, stats.max_size);

// If cache is full, increase size
config.cache_config.glyph_cache.max_entries = 20_000;

// If hit rate still low, analyze access patterns
// Consider warming cache or optimizing rendering
```

### Memory Growth

**Symptoms**: Memory usage continuously increasing

**Causes**:
1. Cache eviction not working
2. Font data leaking
3. Too many fonts loaded
4. Shaped text accumulating

**Solutions**:
```rust
// Monitor memory growth
let stats = font_system.detailed_stats();
println!("Total memory: {} MB", stats.total_bytes / 1024 / 1024);
println!("Fonts loaded: {}", stats.font_count);

// Manually clear caches if needed
font_system.clear_caches();

// Reduce cache sizes
config.cache_config.glyph_cache.max_memory_bytes = 50 * 1024 * 1024; // 50 MB
```

### Eviction Thrashing

**Symptoms**: High eviction rate, poor performance

**Causes**:
1. Working set larger than cache
2. Random access pattern
3. Cache size too small
4. Too many sizes/modes

**Solutions**:
- Increase cache size to match working set
- Batch similar operations together
- Reduce number of font sizes used
- Use fewer render modes

---

## Future Enhancements

**Planned for v0.2.0+**:
- [ ] Adaptive cache sizing based on memory pressure
- [ ] Cache sharing across multiple FontSystem instances
- [ ] Persistent cache (disk-based)
- [ ] Compressed cache entries (save memory)
- [ ] GPU texture cache integration
- [ ] Cache preloading from hints
- [ ] Machine learning-based eviction predictions

---

## Related Documentation

- [Performance Baseline](PERFORMANCE-BASELINE.md) - Benchmark results
- [Memory Profiling](../components/font_system_api/src/profiling.rs) - Profiling APIs
- [Font System API](../components/font_system_api/README.md) - API documentation

---

**Last Updated**: 2025-11-14
**Author**: Corten Font System Team
**Version**: 0.1.0
