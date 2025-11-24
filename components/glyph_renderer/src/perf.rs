//! Performance instrumentation and benchmarking utilities (FEAT-030)
//!
//! This module provides timing instrumentation for hot paths and
//! performance tracking utilities. Features include:
//! - Zero-overhead timing when disabled
//! - Hierarchical timing scopes
//! - Statistical aggregation (min, max, avg, percentiles)
//! - Memory allocation tracking

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Global flag to enable/disable instrumentation
static INSTRUMENTATION_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable performance instrumentation globally
pub fn enable_instrumentation() {
    INSTRUMENTATION_ENABLED.store(true, Ordering::SeqCst);
}

/// Disable performance instrumentation globally
pub fn disable_instrumentation() {
    INSTRUMENTATION_ENABLED.store(false, Ordering::SeqCst);
}

/// Check if instrumentation is enabled
#[inline]
pub fn is_instrumentation_enabled() -> bool {
    INSTRUMENTATION_ENABLED.load(Ordering::Relaxed)
}

/// Timing sample for a single measurement
#[derive(Debug, Clone, Copy)]
pub struct TimingSample {
    /// Duration of the operation
    pub duration: Duration,
    /// Timestamp when measurement started
    pub timestamp: Instant,
}

/// Aggregated timing statistics
#[derive(Debug, Clone, Default)]
pub struct TimingStats {
    /// Number of samples
    pub count: u64,
    /// Total time
    pub total: Duration,
    /// Minimum time
    pub min: Duration,
    /// Maximum time
    pub max: Duration,
    /// Mean time
    pub mean: Duration,
    /// Variance (for std dev calculation)
    variance_sum: f64,
}

impl TimingStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            count: 0,
            total: Duration::ZERO,
            min: Duration::MAX,
            max: Duration::ZERO,
            mean: Duration::ZERO,
            variance_sum: 0.0,
        }
    }

    /// Add a sample using Welford's online algorithm
    pub fn add_sample(&mut self, duration: Duration) {
        self.count += 1;
        self.total += duration;

        if duration < self.min {
            self.min = duration;
        }
        if duration > self.max {
            self.max = duration;
        }

        // Welford's online algorithm for mean and variance
        let duration_secs = duration.as_secs_f64();
        let delta = duration_secs - self.mean.as_secs_f64();
        let new_mean = self.mean.as_secs_f64() + delta / self.count as f64;
        let delta2 = duration_secs - new_mean;
        self.variance_sum += delta * delta2;
        self.mean = Duration::from_secs_f64(new_mean);
    }

    /// Get standard deviation
    pub fn std_dev(&self) -> Duration {
        if self.count < 2 {
            return Duration::ZERO;
        }
        let variance = self.variance_sum / (self.count - 1) as f64;
        Duration::from_secs_f64(variance.sqrt())
    }

    /// Get operations per second
    pub fn ops_per_second(&self) -> f64 {
        if self.mean.as_secs_f64() > 0.0 {
            1.0 / self.mean.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Merge with another stats instance
    pub fn merge(&mut self, other: &TimingStats) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = other.clone();
            return;
        }

        let total_count = self.count + other.count;
        let delta = other.mean.as_secs_f64() - self.mean.as_secs_f64();

        self.total += other.total;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);

        // Combined mean
        let new_mean = (self.mean.as_secs_f64() * self.count as f64
            + other.mean.as_secs_f64() * other.count as f64)
            / total_count as f64;

        // Combined variance (parallel algorithm)
        self.variance_sum += other.variance_sum
            + delta * delta * self.count as f64 * other.count as f64 / total_count as f64;

        self.mean = Duration::from_secs_f64(new_mean);
        self.count = total_count;
    }
}

/// RAII guard for timing a scope
pub struct TimingScope {
    name: &'static str,
    start: Instant,
    collector: Option<*const TimingCollector>,
}

impl TimingScope {
    /// Create a new timing scope
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            collector: None,
        }
    }

    /// Create a timing scope with collector
    #[inline]
    pub fn with_collector(name: &'static str, collector: &TimingCollector) -> Self {
        Self {
            name,
            start: Instant::now(),
            collector: Some(collector as *const _),
        }
    }

    /// Get elapsed time without ending scope
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for TimingScope {
    fn drop(&mut self) {
        if let Some(collector_ptr) = self.collector {
            let duration = self.start.elapsed();
            // Safety: collector is valid for the lifetime of the scope
            unsafe {
                (*collector_ptr).record(self.name, duration);
            }
        }
    }
}

/// Thread-local timing collector
pub struct TimingCollector {
    /// Statistics by name
    stats: RefCell<HashMap<&'static str, TimingStats>>,
    /// Whether this collector is active
    active: bool,
}

impl TimingCollector {
    /// Create a new timing collector
    pub fn new() -> Self {
        Self {
            stats: RefCell::new(HashMap::new()),
            active: true,
        }
    }

    /// Create an inactive collector (no-op)
    pub fn inactive() -> Self {
        Self {
            stats: RefCell::new(HashMap::new()),
            active: false,
        }
    }

    /// Record a timing measurement
    #[inline]
    pub fn record(&self, name: &'static str, duration: Duration) {
        if !self.active {
            return;
        }
        self.stats
            .borrow_mut()
            .entry(name)
            .or_insert_with(TimingStats::new)
            .add_sample(duration);
    }

    /// Start a timing scope
    #[inline]
    pub fn scope(&self, name: &'static str) -> TimingScope {
        if self.active {
            TimingScope::with_collector(name, self)
        } else {
            TimingScope::new(name)
        }
    }

    /// Get statistics for a specific name
    pub fn get_stats(&self, name: &str) -> Option<TimingStats> {
        self.stats.borrow().get(name).cloned()
    }

    /// Get all statistics
    pub fn all_stats(&self) -> HashMap<&'static str, TimingStats> {
        self.stats.borrow().clone()
    }

    /// Clear all statistics
    pub fn clear(&self) {
        self.stats.borrow_mut().clear();
    }

    /// Generate a report string
    pub fn report(&self) -> String {
        let stats = self.stats.borrow();
        let mut report = String::new();
        report.push_str("Performance Report\n");
        report.push_str("==================\n\n");

        let mut entries: Vec<_> = stats.iter().collect();
        entries.sort_by_key(|(name, _)| *name);

        for (name, stat) in entries {
            report.push_str(&format!("{}\n", name));
            report.push_str(&format!("  Count: {}\n", stat.count));
            report.push_str(&format!("  Total: {:?}\n", stat.total));
            report.push_str(&format!("  Mean: {:?}\n", stat.mean));
            report.push_str(&format!("  Min: {:?}\n", stat.min));
            report.push_str(&format!("  Max: {:?}\n", stat.max));
            report.push_str(&format!("  Std Dev: {:?}\n", stat.std_dev()));
            report.push_str(&format!("  Ops/sec: {:.2}\n\n", stat.ops_per_second()));
        }

        report
    }
}

impl Default for TimingCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory tracking for allocation analysis
#[derive(Debug, Default)]
pub struct MemoryTracker {
    /// Current allocated bytes
    current_bytes: AtomicU64,
    /// Peak allocated bytes
    peak_bytes: AtomicU64,
    /// Total allocations count
    total_allocations: AtomicU64,
    /// Total deallocations count
    total_deallocations: AtomicU64,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an allocation
    #[inline]
    pub fn record_alloc(&self, bytes: usize) {
        let current = self
            .current_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed)
            + bytes as u64;
        self.total_allocations.fetch_add(1, Ordering::Relaxed);

        // Update peak
        loop {
            let peak = self.peak_bytes.load(Ordering::Relaxed);
            if current <= peak {
                break;
            }
            if self
                .peak_bytes
                .compare_exchange_weak(peak, current, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Record a deallocation
    #[inline]
    pub fn record_dealloc(&self, bytes: usize) {
        self.current_bytes
            .fetch_sub(bytes as u64, Ordering::Relaxed);
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current memory usage
    pub fn current_bytes(&self) -> u64 {
        self.current_bytes.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_bytes(&self) -> u64 {
        self.peak_bytes.load(Ordering::Relaxed)
    }

    /// Get total allocations
    pub fn total_allocations(&self) -> u64 {
        self.total_allocations.load(Ordering::Relaxed)
    }

    /// Get total deallocations
    pub fn total_deallocations(&self) -> u64 {
        self.total_deallocations.load(Ordering::Relaxed)
    }

    /// Reset the tracker
    pub fn reset(&self) {
        self.current_bytes.store(0, Ordering::Relaxed);
        self.peak_bytes.store(0, Ordering::Relaxed);
        self.total_allocations.store(0, Ordering::Relaxed);
        self.total_deallocations.store(0, Ordering::Relaxed);
    }
}

/// Macro for conditional timing
#[macro_export]
macro_rules! timed {
    ($name:expr, $block:expr) => {{
        if $crate::perf::is_instrumentation_enabled() {
            let _scope = $crate::perf::TimingScope::new($name);
            $block
        } else {
            $block
        }
    }};
}

/// Macro for timing with collector
#[macro_export]
macro_rules! timed_with {
    ($collector:expr, $name:expr, $block:expr) => {{
        let _scope = $collector.scope($name);
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_timing_stats_basic() {
        let mut stats = TimingStats::new();
        stats.add_sample(Duration::from_millis(10));
        stats.add_sample(Duration::from_millis(20));
        stats.add_sample(Duration::from_millis(30));

        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(30));
        // Mean should be ~20ms
        assert!((stats.mean.as_millis() as i64 - 20).abs() < 2);
    }

    #[test]
    fn test_timing_collector() {
        let collector = TimingCollector::new();

        collector.record("test_op", Duration::from_millis(5));
        collector.record("test_op", Duration::from_millis(10));
        collector.record("test_op", Duration::from_millis(15));

        let stats = collector.get_stats("test_op").unwrap();
        assert_eq!(stats.count, 3);
    }

    #[test]
    fn test_timing_scope() {
        let collector = TimingCollector::new();

        {
            let _scope = collector.scope("scoped_op");
            thread::sleep(Duration::from_millis(1));
        }

        let stats = collector.get_stats("scoped_op").unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.total >= Duration::from_millis(1));
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();

        tracker.record_alloc(1000);
        assert_eq!(tracker.current_bytes(), 1000);
        assert_eq!(tracker.peak_bytes(), 1000);

        tracker.record_alloc(500);
        assert_eq!(tracker.current_bytes(), 1500);
        assert_eq!(tracker.peak_bytes(), 1500);

        tracker.record_dealloc(1000);
        assert_eq!(tracker.current_bytes(), 500);
        assert_eq!(tracker.peak_bytes(), 1500); // Peak unchanged
    }

    #[test]
    fn test_timing_stats_merge() {
        let mut stats1 = TimingStats::new();
        stats1.add_sample(Duration::from_millis(10));
        stats1.add_sample(Duration::from_millis(20));

        let mut stats2 = TimingStats::new();
        stats2.add_sample(Duration::from_millis(30));
        stats2.add_sample(Duration::from_millis(40));

        stats1.merge(&stats2);

        assert_eq!(stats1.count, 4);
        assert_eq!(stats1.min, Duration::from_millis(10));
        assert_eq!(stats1.max, Duration::from_millis(40));
    }

    #[test]
    fn test_instrumentation_toggle() {
        disable_instrumentation();
        assert!(!is_instrumentation_enabled());

        enable_instrumentation();
        assert!(is_instrumentation_enabled());

        disable_instrumentation();
    }
}
