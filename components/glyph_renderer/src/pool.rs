//! Memory pool optimization for glyph allocations (FEAT-047)
//!
//! This module provides generic memory pools for efficient allocation and reuse
//! of frequently created objects. Features include:
//! - Generic `MemoryPool<T>` for any clonable type
//! - Specialized pools for `PositionedGlyph` and bitmap buffers
//! - Thread-safe access with minimal contention
//! - Configurable pool sizes and growth strategies

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Default initial pool capacity
const DEFAULT_INITIAL_CAPACITY: usize = 64;

/// Default maximum pool size
const DEFAULT_MAX_POOL_SIZE: usize = 1024;

/// Configuration for memory pools
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial number of pre-allocated items
    pub initial_capacity: usize,
    /// Maximum number of items to keep in pool
    pub max_pool_size: usize,
    /// Whether to pre-allocate on creation
    pub pre_allocate: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_capacity: DEFAULT_INITIAL_CAPACITY,
            max_pool_size: DEFAULT_MAX_POOL_SIZE,
            pre_allocate: true,
        }
    }
}

/// Statistics for pool usage
#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStats {
    /// Number of items currently in pool
    pub available: usize,
    /// Total items allocated
    pub total_allocations: usize,
    /// Number of items returned to pool
    pub total_returns: usize,
    /// Number of items dropped (pool was full)
    pub total_drops: usize,
    /// Peak pool size
    pub peak_size: usize,
}

/// Generic memory pool for reusable objects
///
/// # Type Parameters
///
/// * `T` - Type of pooled objects, must be Default + Send
///
/// # Example
///
/// ```ignore
/// use glyph_renderer::pool::MemoryPool;
///
/// let pool: MemoryPool<Vec<u8>> = MemoryPool::new();
/// let mut buffer = pool.acquire();
/// buffer.extend_from_slice(&[1, 2, 3]);
/// // When dropped, buffer returns to pool
/// ```
pub struct MemoryPool<T: Default + Send> {
    pool: Mutex<VecDeque<T>>,
    config: PoolConfig,
    stats_allocations: AtomicUsize,
    stats_returns: AtomicUsize,
    stats_drops: AtomicUsize,
    stats_peak: AtomicUsize,
}

impl<T: Default + Send> MemoryPool<T> {
    /// Create a new memory pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create a new memory pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        let mut pool = VecDeque::with_capacity(config.initial_capacity);
        let initial_capacity = config.initial_capacity;

        if config.pre_allocate {
            for _ in 0..config.initial_capacity {
                pool.push_back(T::default());
            }
        }

        Self {
            pool: Mutex::new(pool),
            config,
            stats_allocations: AtomicUsize::new(0),
            stats_returns: AtomicUsize::new(0),
            stats_drops: AtomicUsize::new(0),
            stats_peak: AtomicUsize::new(initial_capacity),
        }
    }

    /// Acquire an item from the pool
    ///
    /// Returns a pooled item if available, otherwise creates a new one
    #[inline]
    pub fn acquire(&self) -> PooledItem<T> {
        let item = {
            let mut pool = self.pool.lock();
            pool.pop_front()
        };

        let value = item.unwrap_or_else(T::default);
        self.stats_allocations.fetch_add(1, Ordering::Relaxed);

        PooledItem {
            value: Some(value),
            pool: self,
        }
    }

    /// Return an item to the pool
    ///
    /// Called automatically when PooledItem is dropped
    fn return_item(&self, item: T) {
        let mut pool = self.pool.lock();
        if pool.len() < self.config.max_pool_size {
            pool.push_back(item);
            self.stats_returns.fetch_add(1, Ordering::Relaxed);

            // Update peak
            let current = pool.len();
            loop {
                let peak = self.stats_peak.load(Ordering::Relaxed);
                if current <= peak {
                    break;
                }
                if self
                    .stats_peak
                    .compare_exchange_weak(peak, current, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    break;
                }
            }
        } else {
            self.stats_drops.fetch_add(1, Ordering::Relaxed);
            // Item is dropped
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let pool = self.pool.lock();
        PoolStats {
            available: pool.len(),
            total_allocations: self.stats_allocations.load(Ordering::Relaxed),
            total_returns: self.stats_returns.load(Ordering::Relaxed),
            total_drops: self.stats_drops.load(Ordering::Relaxed),
            peak_size: self.stats_peak.load(Ordering::Relaxed),
        }
    }

    /// Clear the pool
    pub fn clear(&self) {
        self.pool.lock().clear();
    }

    /// Pre-warm the pool with additional items
    pub fn pre_warm(&self, count: usize) {
        let mut pool = self.pool.lock();
        for _ in 0..count {
            if pool.len() >= self.config.max_pool_size {
                break;
            }
            pool.push_back(T::default());
        }
    }
}

impl<T: Default + Send> Default for MemoryPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII wrapper for pooled items
///
/// Automatically returns the item to the pool when dropped
pub struct PooledItem<'a, T: Default + Send> {
    value: Option<T>,
    pool: &'a MemoryPool<T>,
}

impl<'a, T: Default + Send> std::ops::Deref for PooledItem<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<'a, T: Default + Send> std::ops::DerefMut for PooledItem<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

impl<'a, T: Default + Send> Drop for PooledItem<'a, T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.pool.return_item(value);
        }
    }
}

impl<'a, T: Default + Send> PooledItem<'a, T> {
    /// Take ownership of the item without returning to pool
    pub fn take(mut self) -> T {
        self.value.take().unwrap()
    }
}

/// Specialized buffer pool for glyph bitmap data
pub struct BufferPool {
    /// Small buffer pool (< 4KB)
    small: MemoryPool<Vec<u8>>,
    /// Medium buffer pool (4KB - 64KB)
    medium: MemoryPool<Vec<u8>>,
    /// Large buffer pool (> 64KB)
    large: MemoryPool<Vec<u8>>,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new() -> Self {
        Self {
            small: MemoryPool::with_config(PoolConfig {
                initial_capacity: 128,
                max_pool_size: 512,
                pre_allocate: true,
            }),
            medium: MemoryPool::with_config(PoolConfig {
                initial_capacity: 32,
                max_pool_size: 128,
                pre_allocate: true,
            }),
            large: MemoryPool::with_config(PoolConfig {
                initial_capacity: 8,
                max_pool_size: 32,
                pre_allocate: false,
            }),
        }
    }

    /// Acquire a buffer of at least the specified size
    #[inline]
    pub fn acquire(&self, min_size: usize) -> PooledBuffer<'_> {
        let (pool, initial_capacity) = if min_size <= 4096 {
            (&self.small, 4096)
        } else if min_size <= 65536 {
            (&self.medium, 65536)
        } else {
            (&self.large, min_size.next_power_of_two())
        };

        let mut item = pool.acquire();
        if item.capacity() < min_size {
            let current_len = item.len();
            item.reserve(initial_capacity.saturating_sub(current_len));
        }
        item.clear();

        PooledBuffer {
            buffer: item,
            min_size,
        }
    }

    /// Get combined statistics
    pub fn stats(&self) -> BufferPoolStats {
        BufferPoolStats {
            small: self.small.stats(),
            medium: self.medium.stats(),
            large: self.large.stats(),
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for buffer pool
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    /// Small buffer stats
    pub small: PoolStats,
    /// Medium buffer stats
    pub medium: PoolStats,
    /// Large buffer stats
    pub large: PoolStats,
}

/// Wrapper for pooled buffers
pub struct PooledBuffer<'a> {
    buffer: PooledItem<'a, Vec<u8>>,
    min_size: usize,
}

impl<'a> std::ops::Deref for PooledBuffer<'a> {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a> std::ops::DerefMut for PooledBuffer<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

/// Arena allocator for batch glyph operations
///
/// Provides fast bump allocation for temporary glyph data
/// that can be freed all at once
pub struct GlyphArena {
    /// Current chunk being allocated from
    current: Mutex<ArenaChunk>,
    /// Completed chunks
    chunks: Mutex<Vec<ArenaChunk>>,
    /// Default chunk size
    chunk_size: usize,
    /// Statistics
    stats_bytes_allocated: AtomicUsize,
    stats_chunks_allocated: AtomicUsize,
}

struct ArenaChunk {
    data: Vec<u8>,
    cursor: usize,
}

impl ArenaChunk {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            cursor: 0,
        }
    }

    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        // Align cursor
        let aligned = (self.cursor + align - 1) & !(align - 1);
        let end = aligned + size;

        if end <= self.data.len() {
            let ptr = unsafe { self.data.as_mut_ptr().add(aligned) };
            self.cursor = end;
            Some(ptr)
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.cursor = 0;
    }

    fn used(&self) -> usize {
        self.cursor
    }
}

impl GlyphArena {
    /// Default chunk size (1MB)
    const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

    /// Create a new glyph arena
    pub fn new() -> Self {
        Self::with_chunk_size(Self::DEFAULT_CHUNK_SIZE)
    }

    /// Create arena with custom chunk size
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            current: Mutex::new(ArenaChunk::new(chunk_size)),
            chunks: Mutex::new(Vec::new()),
            chunk_size,
            stats_bytes_allocated: AtomicUsize::new(0),
            stats_chunks_allocated: AtomicUsize::new(1),
        }
    }

    /// Allocate a slice of bytes
    ///
    /// # Safety
    /// The returned slice is valid until reset() is called
    pub fn allocate_bytes(&self, size: usize) -> Option<&mut [u8]> {
        self.allocate_aligned(size, 1)
    }

    /// Allocate with alignment
    pub fn allocate_aligned(&self, size: usize, align: usize) -> Option<&mut [u8]> {
        let mut current = self.current.lock();

        // Try current chunk first
        if let Some(ptr) = current.allocate(size, align) {
            self.stats_bytes_allocated
                .fetch_add(size, Ordering::Relaxed);
            return Some(unsafe { std::slice::from_raw_parts_mut(ptr, size) });
        }

        // Need new chunk
        let new_chunk_size = self.chunk_size.max(size);
        let mut new_chunk = ArenaChunk::new(new_chunk_size);

        let ptr = new_chunk.allocate(size, align)?;

        // Move old chunk to completed list
        let old_chunk = std::mem::replace(&mut *current, new_chunk);
        self.chunks.lock().push(old_chunk);
        self.stats_chunks_allocated.fetch_add(1, Ordering::Relaxed);
        self.stats_bytes_allocated
            .fetch_add(size, Ordering::Relaxed);

        Some(unsafe { std::slice::from_raw_parts_mut(ptr, size) })
    }

    /// Reset the arena, freeing all allocations
    pub fn reset(&self) {
        let mut current = self.current.lock();
        current.clear();

        let mut chunks = self.chunks.lock();
        for chunk in chunks.iter_mut() {
            chunk.clear();
        }

        self.stats_bytes_allocated.store(0, Ordering::Relaxed);
    }

    /// Get arena statistics
    pub fn stats(&self) -> ArenaStats {
        let current = self.current.lock();
        let chunks = self.chunks.lock();

        let total_capacity: usize =
            current.data.len() + chunks.iter().map(|c| c.data.len()).sum::<usize>();

        ArenaStats {
            bytes_allocated: self.stats_bytes_allocated.load(Ordering::Relaxed),
            chunks_allocated: self.stats_chunks_allocated.load(Ordering::Relaxed),
            total_capacity,
            current_chunk_used: current.used(),
        }
    }
}

impl Default for GlyphArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Arena statistics
#[derive(Debug, Clone, Copy)]
pub struct ArenaStats {
    /// Total bytes currently allocated
    pub bytes_allocated: usize,
    /// Total chunks allocated
    pub chunks_allocated: usize,
    /// Total capacity across all chunks
    pub total_capacity: usize,
    /// Bytes used in current chunk
    pub current_chunk_used: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_acquire_and_return() {
        let pool: MemoryPool<Vec<u8>> = MemoryPool::new();
        let initial_stats = pool.stats();
        assert_eq!(initial_stats.available, DEFAULT_INITIAL_CAPACITY);

        // Acquire an item
        {
            let mut item = pool.acquire();
            item.push(42);
            assert_eq!(*item, vec![42]);
        }
        // Item returned to pool

        let stats = pool.stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_returns, 1);
    }

    #[test]
    fn test_memory_pool_take() {
        let pool: MemoryPool<Vec<u8>> = MemoryPool::new();

        let item = pool.acquire();
        let taken = item.take();
        // Item NOT returned to pool

        let stats = pool.stats();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_returns, 0);
    }

    #[test]
    fn test_buffer_pool_sizes() {
        let pool = BufferPool::new();

        // Small buffer
        {
            let buf = pool.acquire(100);
            assert!(buf.capacity() >= 100);
        }

        // Medium buffer
        {
            let buf = pool.acquire(10000);
            assert!(buf.capacity() >= 10000);
        }

        // Large buffer
        {
            let buf = pool.acquire(100000);
            assert!(buf.capacity() >= 100000);
        }
    }

    #[test]
    fn test_arena_allocation() {
        let arena = GlyphArena::new();

        let slice1 = arena.allocate_bytes(100).unwrap();
        assert_eq!(slice1.len(), 100);

        let slice2 = arena.allocate_bytes(200).unwrap();
        assert_eq!(slice2.len(), 200);

        let stats = arena.stats();
        assert_eq!(stats.bytes_allocated, 300);
    }

    #[test]
    fn test_arena_reset() {
        let arena = GlyphArena::new();

        arena.allocate_bytes(100);
        arena.allocate_bytes(200);

        arena.reset();

        let stats = arena.stats();
        assert_eq!(stats.bytes_allocated, 0);
    }

    #[test]
    fn test_arena_large_allocation() {
        let arena = GlyphArena::with_chunk_size(1024);

        // Allocate more than chunk size
        let slice = arena.allocate_bytes(2048).unwrap();
        assert_eq!(slice.len(), 2048);

        let stats = arena.stats();
        assert!(stats.chunks_allocated >= 2);
    }
}
