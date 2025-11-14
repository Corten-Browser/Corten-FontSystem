//! Memory profiling utilities for the font system
//!
//! This module provides tools to monitor and analyze memory usage across
//! all components of the font system.

/// Memory statistics for the entire font system
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryStats {
    /// Memory used by loaded font data (bytes)
    pub font_data_bytes: usize,
    /// Memory used by glyph cache (bytes)
    pub glyph_cache_bytes: usize,
    /// Memory used by shaping cache (bytes)
    pub shaping_cache_bytes: usize,
    /// Total memory usage (bytes)
    pub total_bytes: usize,
    /// Number of loaded fonts
    pub font_count: usize,
    /// Number of cached glyphs
    pub cached_glyphs: usize,
    /// Number of cached shaping results
    pub cached_shapings: usize,
}

impl MemoryStats {
    /// Create new empty memory stats
    pub fn new() -> Self {
        Self {
            font_data_bytes: 0,
            glyph_cache_bytes: 0,
            shaping_cache_bytes: 0,
            total_bytes: 0,
            font_count: 0,
            cached_glyphs: 0,
            cached_shapings: 0,
        }
    }

    /// Get memory usage as megabytes
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get glyph cache usage as megabytes
    pub fn glyph_cache_mb(&self) -> f64 {
        self.glyph_cache_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get font data usage as megabytes
    pub fn font_data_mb(&self) -> f64 {
        self.font_data_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get shaping cache usage as megabytes
    pub fn shaping_cache_mb(&self) -> f64 {
        self.shaping_cache_bytes as f64 / (1024.0 * 1024.0)
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for components that can report memory usage
pub trait MemoryProfiler {
    /// Get current memory usage in bytes
    fn memory_usage(&self) -> usize;

    /// Get detailed memory statistics
    fn detailed_stats(&self) -> MemoryStats;
}

/// Memory usage breakdown by component
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentMemoryBreakdown {
    /// Font registry memory usage
    pub font_registry_bytes: usize,
    /// Glyph renderer memory usage
    pub glyph_renderer_bytes: usize,
    /// Text shaper memory usage
    pub text_shaper_bytes: usize,
    /// Platform integration memory usage
    pub platform_integration_bytes: usize,
}

impl ComponentMemoryBreakdown {
    /// Create new empty breakdown
    pub fn new() -> Self {
        Self {
            font_registry_bytes: 0,
            glyph_renderer_bytes: 0,
            text_shaper_bytes: 0,
            platform_integration_bytes: 0,
        }
    }

    /// Get total memory usage across all components
    pub fn total(&self) -> usize {
        self.font_registry_bytes
            + self.glyph_renderer_bytes
            + self.text_shaper_bytes
            + self.platform_integration_bytes
    }

    /// Get breakdown as percentages
    pub fn percentages(&self) -> ComponentMemoryPercentages {
        let total = self.total() as f64;
        if total == 0.0 {
            return ComponentMemoryPercentages::default();
        }

        ComponentMemoryPercentages {
            font_registry: (self.font_registry_bytes as f64 / total) * 100.0,
            glyph_renderer: (self.glyph_renderer_bytes as f64 / total) * 100.0,
            text_shaper: (self.text_shaper_bytes as f64 / total) * 100.0,
            platform_integration: (self.platform_integration_bytes as f64 / total) * 100.0,
        }
    }
}

impl Default for ComponentMemoryBreakdown {
    fn default() -> Self {
        Self::new()
    }
}

/// Component memory usage as percentages
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentMemoryPercentages {
    /// Font registry percentage
    pub font_registry: f64,
    /// Glyph renderer percentage
    pub glyph_renderer: f64,
    /// Text shaper percentage
    pub text_shaper: f64,
    /// Platform integration percentage
    pub platform_integration: f64,
}

impl Default for ComponentMemoryPercentages {
    fn default() -> Self {
        Self {
            font_registry: 0.0,
            glyph_renderer: 0.0,
            text_shaper: 0.0,
            platform_integration: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_new() {
        let stats = MemoryStats::new();
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.font_count, 0);
    }

    #[test]
    fn test_memory_stats_mb_conversion() {
        let stats = MemoryStats {
            font_data_bytes: 0,
            glyph_cache_bytes: 10 * 1024 * 1024, // 10 MB
            shaping_cache_bytes: 0,
            total_bytes: 10 * 1024 * 1024,
            font_count: 0,
            cached_glyphs: 0,
            cached_shapings: 0,
        };

        assert!((stats.glyph_cache_mb() - 10.0).abs() < 0.01);
        assert!((stats.total_mb() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_component_memory_breakdown() {
        let breakdown = ComponentMemoryBreakdown {
            font_registry_bytes: 100,
            glyph_renderer_bytes: 200,
            text_shaper_bytes: 300,
            platform_integration_bytes: 400,
        };

        assert_eq!(breakdown.total(), 1000);

        let percentages = breakdown.percentages();
        assert!((percentages.font_registry - 10.0).abs() < 0.01);
        assert!((percentages.glyph_renderer - 20.0).abs() < 0.01);
        assert!((percentages.text_shaper - 30.0).abs() < 0.01);
        assert!((percentages.platform_integration - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_zero_total_percentages() {
        let breakdown = ComponentMemoryBreakdown::new();
        let percentages = breakdown.percentages();

        assert_eq!(percentages.font_registry, 0.0);
        assert_eq!(percentages.glyph_renderer, 0.0);
        assert_eq!(percentages.text_shaper, 0.0);
        assert_eq!(percentages.platform_integration, 0.0);
    }
}
