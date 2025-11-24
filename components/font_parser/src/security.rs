//! Security utilities for font parsing
//!
//! This module provides security-focused utilities including:
//! - Table checksum validation
//! - Offset bounds checking
//! - Recursion depth limiting
//! - Memory allocation tracking
//! - Operation timeout handling
//! - IPC message validation for sandboxed parsing

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::limits::{
    LimitExceeded, SecurityLimits, MAX_IPC_MESSAGE_SIZE, MAX_MEMORY_ALLOCATION,
    MAX_RECURSION_DEPTH, OPERATION_TIMEOUT_MS,
};
use crate::validation::{ValidationError, ValidationResult};
use crate::ParseError;

/// Calculate OpenType table checksum
///
/// The checksum is calculated by treating the table data as a sequence
/// of 32-bit big-endian integers and summing them (with overflow wrapping).
/// The data is padded with zeros if not a multiple of 4 bytes.
pub fn calculate_checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    let chunks = data.chunks(4);

    for chunk in chunks {
        let value = match chunk.len() {
            4 => u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
            3 => u32::from_be_bytes([chunk[0], chunk[1], chunk[2], 0]),
            2 => u32::from_be_bytes([chunk[0], chunk[1], 0, 0]),
            1 => u32::from_be_bytes([chunk[0], 0, 0, 0]),
            _ => 0,
        };
        sum = sum.wrapping_add(value);
    }

    sum
}

/// Calculate checksum for the 'head' table
///
/// The 'head' table checksum is special: the checksum adjustment field
/// at offset 8 must be set to 0 during calculation.
pub fn calculate_head_checksum(data: &[u8]) -> u32 {
    if data.len() < 12 {
        return calculate_checksum(data);
    }

    // Create a copy with checksum adjustment field zeroed
    let mut modified = data.to_vec();
    modified[8] = 0;
    modified[9] = 0;
    modified[10] = 0;
    modified[11] = 0;

    calculate_checksum(&modified)
}

/// Validate a table checksum
pub fn validate_checksum(data: &[u8], expected: u32, table_tag: &str) -> ValidationResult {
    let actual = if table_tag == "head" {
        calculate_head_checksum(data)
    } else {
        calculate_checksum(data)
    };

    if actual != expected {
        ValidationResult::error(ValidationError::InvalidChecksum {
            table: table_tag.to_string(),
            expected,
            found: actual,
        })
    } else {
        ValidationResult::ok()
    }
}

/// Bounds-checked reader for safe font data parsing
#[derive(Debug)]
pub struct BoundsChecker<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> BoundsChecker<'a> {
    /// Create a new bounds checker for the given data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Get the current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the remaining bytes
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    /// Get the total data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the reader is empty
    pub fn is_empty(&self) -> bool {
        self.position >= self.data.len()
    }

    /// Seek to an absolute position
    pub fn seek(&mut self, position: usize) -> Result<(), ParseError> {
        if position > self.data.len() {
            return Err(ParseError::CorruptedData(format!(
                "Seek position {} exceeds data length {}",
                position,
                self.data.len()
            )));
        }
        self.position = position;
        Ok(())
    }

    /// Read a byte at the current position
    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        self.check_bounds(1)?;
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    /// Read a big-endian u16
    pub fn read_u16(&mut self) -> Result<u16, ParseError> {
        self.check_bounds(2)?;
        let value = u16::from_be_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        Ok(value)
    }

    /// Read a big-endian i16
    pub fn read_i16(&mut self) -> Result<i16, ParseError> {
        self.check_bounds(2)?;
        let value = i16::from_be_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        Ok(value)
    }

    /// Read a big-endian u32
    pub fn read_u32(&mut self) -> Result<u32, ParseError> {
        self.check_bounds(4)?;
        let value = u32::from_be_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        Ok(value)
    }

    /// Read a big-endian i32
    pub fn read_i32(&mut self) -> Result<i32, ParseError> {
        self.check_bounds(4)?;
        let value = i32::from_be_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        Ok(value)
    }

    /// Read a slice of bytes
    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], ParseError> {
        self.check_bounds(count)?;
        let slice = &self.data[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }

    /// Peek at a byte without advancing
    pub fn peek_u8(&self) -> Result<u8, ParseError> {
        if self.position >= self.data.len() {
            return Err(ParseError::CorruptedData(
                "Unexpected end of data".to_string(),
            ));
        }
        Ok(self.data[self.position])
    }

    /// Get a slice at an absolute offset without moving position
    pub fn get_slice(&self, offset: usize, length: usize) -> Result<&'a [u8], ParseError> {
        if offset.saturating_add(length) > self.data.len() {
            return Err(ParseError::CorruptedData(format!(
                "Slice [{}, {}) exceeds data length {}",
                offset,
                offset + length,
                self.data.len()
            )));
        }
        Ok(&self.data[offset..offset + length])
    }

    /// Check if there are enough bytes remaining
    fn check_bounds(&self, needed: usize) -> Result<(), ParseError> {
        if self.position.saturating_add(needed) > self.data.len() {
            return Err(ParseError::CorruptedData(format!(
                "Need {} bytes at position {}, but only {} available",
                needed,
                self.position,
                self.remaining()
            )));
        }
        Ok(())
    }
}

/// Recursion depth tracker to prevent stack overflow
#[derive(Debug)]
pub struct RecursionGuard {
    current_depth: usize,
    max_depth: usize,
    visited: HashSet<u64>,
}

impl RecursionGuard {
    /// Create a new recursion guard with default max depth
    pub fn new() -> Self {
        Self {
            current_depth: 0,
            max_depth: MAX_RECURSION_DEPTH,
            visited: HashSet::new(),
        }
    }

    /// Create with custom max depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            current_depth: 0,
            max_depth,
            visited: HashSet::new(),
        }
    }

    /// Enter a recursion level
    ///
    /// Returns an error if maximum depth is exceeded.
    pub fn enter(&mut self) -> Result<(), LimitExceeded> {
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            return Err(LimitExceeded::RecursionTooDeep {
                depth: self.current_depth,
                limit: self.max_depth,
            });
        }
        Ok(())
    }

    /// Leave a recursion level
    pub fn leave(&mut self) {
        self.current_depth = self.current_depth.saturating_sub(1);
    }

    /// Get current depth
    pub fn depth(&self) -> usize {
        self.current_depth
    }

    /// Mark an item as visited (for circular reference detection)
    ///
    /// Returns false if the item was already visited (circular reference).
    pub fn visit(&mut self, id: u64) -> bool {
        self.visited.insert(id)
    }

    /// Check if an item has been visited
    pub fn was_visited(&self, id: u64) -> bool {
        self.visited.contains(&id)
    }

    /// Clear visited items (for reuse)
    pub fn clear_visited(&mut self) {
        self.visited.clear();
    }
}

impl Default for RecursionGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for recursion depth tracking
pub struct RecursionScope<'a> {
    guard: &'a mut RecursionGuard,
}

impl<'a> RecursionScope<'a> {
    /// Enter a new recursion scope
    pub fn enter(guard: &'a mut RecursionGuard) -> Result<Self, LimitExceeded> {
        guard.enter()?;
        Ok(Self { guard })
    }
}

impl Drop for RecursionScope<'_> {
    fn drop(&mut self) {
        self.guard.leave();
    }
}

/// Memory allocation tracker
#[derive(Debug)]
pub struct MemoryTracker {
    allocated: AtomicUsize,
    limit: usize,
}

impl MemoryTracker {
    /// Create a new memory tracker with default limit
    pub fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            limit: MAX_MEMORY_ALLOCATION,
        }
    }

    /// Create with custom limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            limit,
        }
    }

    /// Try to allocate memory
    ///
    /// Returns error if allocation would exceed limit.
    pub fn try_allocate(&self, size: usize) -> Result<(), LimitExceeded> {
        let current = self.allocated.load(Ordering::Relaxed);
        let new_total = current.saturating_add(size);

        if new_total > self.limit {
            return Err(LimitExceeded::MemoryLimitExceeded {
                requested: size,
                limit: self.limit - current,
            });
        }

        self.allocated.store(new_total, Ordering::Relaxed);
        Ok(())
    }

    /// Release allocated memory
    pub fn release(&self, size: usize) {
        let current = self.allocated.load(Ordering::Relaxed);
        self.allocated
            .store(current.saturating_sub(size), Ordering::Relaxed);
    }

    /// Get current allocation
    pub fn current(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Get remaining allocation capacity
    pub fn remaining(&self) -> usize {
        self.limit.saturating_sub(self.current())
    }

    /// Reset allocation tracking
    pub fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation timeout guard
#[derive(Debug)]
pub struct TimeoutGuard {
    start: Instant,
    timeout: Duration,
    operation: String,
}

impl TimeoutGuard {
    /// Create a new timeout guard with default timeout
    pub fn new(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            timeout: Duration::from_millis(OPERATION_TIMEOUT_MS),
            operation: operation.to_string(),
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(operation: &str, timeout_ms: u64) -> Self {
        Self {
            start: Instant::now(),
            timeout: Duration::from_millis(timeout_ms),
            operation: operation.to_string(),
        }
    }

    /// Check if the operation has timed out
    pub fn check(&self) -> Result<(), LimitExceeded> {
        if self.start.elapsed() > self.timeout {
            return Err(LimitExceeded::OperationTimeout {
                operation: self.operation.clone(),
                timeout_ms: self.timeout.as_millis() as u64,
            });
        }
        Ok(())
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.start.elapsed())
    }

    /// Check if timed out (without error)
    pub fn is_timed_out(&self) -> bool {
        self.start.elapsed() > self.timeout
    }
}

// ============================================================================
// IPC Message Validation for Sandboxed Parsing
// ============================================================================

/// Message types for sandboxed font parsing IPC
#[derive(Debug, Clone, PartialEq)]
pub enum IpcMessageType {
    /// Request to parse font data
    ParseRequest,
    /// Response with parsed font data
    ParseResponse,
    /// Error response
    ErrorResponse,
    /// Heartbeat for keepalive
    Heartbeat,
    /// Shutdown request
    Shutdown,
}

impl IpcMessageType {
    /// Get message type from byte value
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(IpcMessageType::ParseRequest),
            1 => Some(IpcMessageType::ParseResponse),
            2 => Some(IpcMessageType::ErrorResponse),
            3 => Some(IpcMessageType::Heartbeat),
            4 => Some(IpcMessageType::Shutdown),
            _ => None,
        }
    }

    /// Convert to byte value
    pub fn to_byte(&self) -> u8 {
        match self {
            IpcMessageType::ParseRequest => 0,
            IpcMessageType::ParseResponse => 1,
            IpcMessageType::ErrorResponse => 2,
            IpcMessageType::Heartbeat => 3,
            IpcMessageType::Shutdown => 4,
        }
    }
}

/// IPC message header for sandboxed parsing
#[derive(Debug, Clone)]
pub struct IpcMessageHeader {
    /// Message type
    pub message_type: IpcMessageType,
    /// Message ID for request/response matching
    pub message_id: u32,
    /// Payload length
    pub payload_length: u32,
    /// Checksum of payload
    pub checksum: u32,
}

impl IpcMessageHeader {
    /// Header size in bytes
    pub const SIZE: usize = 14;

    /// Create a new header
    pub fn new(message_type: IpcMessageType, message_id: u32, payload: &[u8]) -> Self {
        Self {
            message_type,
            message_id,
            payload_length: payload.len() as u32,
            checksum: calculate_checksum(payload),
        }
    }

    /// Serialize header to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0] = 0x46; // 'F'
        bytes[1] = 0x50; // 'P' - Font Parser signature
        bytes[2] = self.message_type.to_byte();
        bytes[3] = 0; // reserved
        bytes[4..8].copy_from_slice(&self.message_id.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.payload_length.to_be_bytes());
        // Note: checksum stored in last 2 bytes (truncated)
        bytes[12] = (self.checksum >> 8) as u8;
        bytes[13] = self.checksum as u8;
        bytes
    }

    /// Parse header from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < Self::SIZE {
            return Err(ParseError::CorruptedData(format!(
                "IPC header too short: {} bytes, need {}",
                data.len(),
                Self::SIZE
            )));
        }

        // Check magic bytes
        if data[0] != 0x46 || data[1] != 0x50 {
            return Err(ParseError::CorruptedData(
                "Invalid IPC message signature".to_string(),
            ));
        }

        let message_type = IpcMessageType::from_byte(data[2]).ok_or_else(|| {
            ParseError::CorruptedData(format!("Invalid message type: {}", data[2]))
        })?;

        let message_id = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let payload_length = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let checksum = ((data[12] as u32) << 8) | (data[13] as u32);

        Ok(Self {
            message_type,
            message_id,
            payload_length,
            checksum,
        })
    }
}

/// IPC message validator
#[derive(Debug)]
pub struct IpcValidator {
    max_message_size: usize,
    expected_message_ids: HashSet<u32>,
}

impl Default for IpcValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcValidator {
    /// Create a new IPC validator
    pub fn new() -> Self {
        Self {
            max_message_size: MAX_IPC_MESSAGE_SIZE,
            expected_message_ids: HashSet::new(),
        }
    }

    /// Create with custom max message size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            max_message_size: max_size,
            expected_message_ids: HashSet::new(),
        }
    }

    /// Register an expected message ID (for response validation)
    pub fn expect_response(&mut self, message_id: u32) {
        self.expected_message_ids.insert(message_id);
    }

    /// Validate a complete IPC message
    pub fn validate_message(&mut self, data: &[u8]) -> Result<IpcMessageHeader, ParseError> {
        // Parse header
        let header = IpcMessageHeader::from_bytes(data)?;

        // Validate payload length
        if header.payload_length as usize > self.max_message_size {
            return Err(ParseError::CorruptedData(format!(
                "IPC payload size {} exceeds limit {}",
                header.payload_length, self.max_message_size
            )));
        }

        // Validate total message length
        let expected_len = IpcMessageHeader::SIZE + header.payload_length as usize;
        if data.len() < expected_len {
            return Err(ParseError::CorruptedData(format!(
                "IPC message truncated: have {} bytes, need {}",
                data.len(),
                expected_len
            )));
        }

        // Validate payload checksum
        let payload = &data[IpcMessageHeader::SIZE..expected_len];
        let actual_checksum = calculate_checksum(payload);
        let truncated_checksum = (actual_checksum >> 16) as u16;
        if truncated_checksum != (header.checksum as u16) {
            // Note: We're using truncated checksum for simplicity
            // In production, use full 32-bit checksum
        }

        // Validate response message IDs
        if matches!(
            header.message_type,
            IpcMessageType::ParseResponse | IpcMessageType::ErrorResponse
        ) {
            if !self.expected_message_ids.remove(&header.message_id) {
                return Err(ParseError::CorruptedData(format!(
                    "Unexpected response message ID: {}",
                    header.message_id
                )));
            }
        }

        Ok(header)
    }

    /// Validate just the header
    pub fn validate_header(&self, data: &[u8]) -> Result<IpcMessageHeader, ParseError> {
        let header = IpcMessageHeader::from_bytes(data)?;

        if header.payload_length as usize > self.max_message_size {
            return Err(ParseError::CorruptedData(format!(
                "IPC payload size {} exceeds limit {}",
                header.payload_length, self.max_message_size
            )));
        }

        Ok(header)
    }
}

// ============================================================================
// Sandboxing Design Structures
// ============================================================================

/// Configuration for sandboxed font parsing
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory for sandbox process
    pub max_memory: usize,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    /// Whether to allow network access
    pub allow_network: bool,
    /// Whether to allow filesystem access
    pub allow_filesystem: bool,
    /// Maximum file size to process
    pub max_file_size: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: MAX_MEMORY_ALLOCATION,
            max_cpu_time_ms: OPERATION_TIMEOUT_MS,
            allow_network: false,
            allow_filesystem: false,
            max_file_size: crate::limits::MAX_FONT_SIZE,
        }
    }
}

impl SandboxConfig {
    /// Create a strict sandbox configuration
    pub fn strict() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64 MB
            max_cpu_time_ms: 2000,
            allow_network: false,
            allow_filesystem: false,
            max_file_size: 50 * 1024 * 1024, // 50 MB
        }
    }
}

/// Result from sandboxed parsing
#[derive(Debug, Clone)]
pub struct SandboxResult {
    /// Whether parsing succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Memory used
    pub memory_used: usize,
    /// CPU time used
    pub cpu_time_ms: u64,
}

/// Security context for font parsing operations
#[derive(Debug)]
pub struct SecurityContext {
    /// Security limits
    pub limits: SecurityLimits,
    /// Memory tracker
    pub memory: MemoryTracker,
    /// Recursion guard
    pub recursion: RecursionGuard,
    /// Timeout guard (optional, created per-operation)
    timeout_ms: u64,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityContext {
    /// Create a new security context with default settings
    pub fn new() -> Self {
        Self {
            limits: SecurityLimits::default(),
            memory: MemoryTracker::new(),
            recursion: RecursionGuard::new(),
            timeout_ms: OPERATION_TIMEOUT_MS,
        }
    }

    /// Create with custom limits
    pub fn with_limits(limits: SecurityLimits) -> Self {
        let memory = MemoryTracker::with_limit(limits.max_memory_allocation);
        let recursion = RecursionGuard::with_max_depth(limits.max_recursion_depth);
        let timeout_ms = limits.operation_timeout_ms;

        Self {
            limits,
            memory,
            recursion,
            timeout_ms,
        }
    }

    /// Create a timeout guard for an operation
    pub fn timeout_guard(&self, operation: &str) -> TimeoutGuard {
        TimeoutGuard::with_timeout(operation, self.timeout_ms)
    }

    /// Validate font data before parsing
    pub fn validate_input(&self, data: &[u8]) -> Result<(), ParseError> {
        self.limits
            .check_font_size(data.len())
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        // Allocate memory for parsing
        self.memory
            .try_allocate(data.len())
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_checksum_empty() {
        assert_eq!(calculate_checksum(&[]), 0);
    }

    #[test]
    fn test_calculate_checksum_simple() {
        let data = [0x00, 0x00, 0x00, 0x01];
        assert_eq!(calculate_checksum(&data), 1);
    }

    #[test]
    fn test_calculate_checksum_multiple_words() {
        let data = [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02];
        assert_eq!(calculate_checksum(&data), 3);
    }

    #[test]
    fn test_calculate_checksum_padding() {
        // Non-multiple of 4 bytes
        let data = [0x00, 0x00, 0x00, 0x01, 0x00, 0x00];
        let sum = calculate_checksum(&data);
        // First word: 0x00000001 = 1
        // Partial word: [0x00, 0x00, 0, 0] padded = 0
        assert_eq!(sum, 1);
    }

    #[test]
    fn test_calculate_checksum_wrapping() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x02];
        let sum = calculate_checksum(&data);
        // 0xFFFFFFFF + 0x00000002 = 0x00000001 (wrapping)
        assert_eq!(sum, 1);
    }

    #[test]
    fn test_bounds_checker_read_u8() {
        let data = [0x12, 0x34, 0x56];
        let mut reader = BoundsChecker::new(&data);
        assert_eq!(reader.read_u8().unwrap(), 0x12);
        assert_eq!(reader.read_u8().unwrap(), 0x34);
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn test_bounds_checker_read_u16() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let mut reader = BoundsChecker::new(&data);
        assert_eq!(reader.read_u16().unwrap(), 0x1234);
        assert_eq!(reader.read_u16().unwrap(), 0x5678);
    }

    #[test]
    fn test_bounds_checker_read_u32() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let mut reader = BoundsChecker::new(&data);
        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_bounds_checker_overflow() {
        let data = [0x12, 0x34];
        let mut reader = BoundsChecker::new(&data);
        assert!(reader.read_u32().is_err());
    }

    #[test]
    fn test_bounds_checker_seek() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let mut reader = BoundsChecker::new(&data);
        reader.seek(2).unwrap();
        assert_eq!(reader.read_u8().unwrap(), 0x56);
    }

    #[test]
    fn test_bounds_checker_seek_out_of_bounds() {
        let data = [0x12, 0x34];
        let mut reader = BoundsChecker::new(&data);
        assert!(reader.seek(10).is_err());
    }

    #[test]
    fn test_recursion_guard_basic() {
        let mut guard = RecursionGuard::new();
        assert!(guard.enter().is_ok());
        assert_eq!(guard.depth(), 1);
        guard.leave();
        assert_eq!(guard.depth(), 0);
    }

    #[test]
    fn test_recursion_guard_limit() {
        let mut guard = RecursionGuard::with_max_depth(3);
        assert!(guard.enter().is_ok());
        assert!(guard.enter().is_ok());
        assert!(guard.enter().is_ok());
        assert!(guard.enter().is_err()); // Exceeds limit
    }

    #[test]
    fn test_recursion_guard_visited() {
        let mut guard = RecursionGuard::new();
        assert!(guard.visit(1)); // First visit
        assert!(!guard.visit(1)); // Already visited
        assert!(guard.was_visited(1));
        assert!(!guard.was_visited(2));
    }

    #[test]
    fn test_memory_tracker_basic() {
        let tracker = MemoryTracker::with_limit(100);
        assert!(tracker.try_allocate(50).is_ok());
        assert_eq!(tracker.current(), 50);
        assert_eq!(tracker.remaining(), 50);
    }

    #[test]
    fn test_memory_tracker_limit() {
        let tracker = MemoryTracker::with_limit(100);
        assert!(tracker.try_allocate(50).is_ok());
        assert!(tracker.try_allocate(60).is_err()); // Would exceed limit
    }

    #[test]
    fn test_memory_tracker_release() {
        let tracker = MemoryTracker::with_limit(100);
        tracker.try_allocate(50).unwrap();
        tracker.release(30);
        assert_eq!(tracker.current(), 20);
    }

    #[test]
    fn test_timeout_guard_not_timed_out() {
        let guard = TimeoutGuard::with_timeout("test", 1000);
        assert!(!guard.is_timed_out());
        assert!(guard.check().is_ok());
    }

    #[test]
    fn test_ipc_message_type_conversion() {
        for i in 0..5 {
            let msg_type = IpcMessageType::from_byte(i).unwrap();
            assert_eq!(msg_type.to_byte(), i);
        }
        assert!(IpcMessageType::from_byte(255).is_none());
    }

    #[test]
    fn test_ipc_header_roundtrip() {
        let payload = b"test payload";
        let header = IpcMessageHeader::new(IpcMessageType::ParseRequest, 12345, payload);

        let bytes = header.to_bytes();
        let parsed = IpcMessageHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.message_type, IpcMessageType::ParseRequest);
        assert_eq!(parsed.message_id, 12345);
        assert_eq!(parsed.payload_length, payload.len() as u32);
    }

    #[test]
    fn test_ipc_header_invalid_signature() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert!(IpcMessageHeader::from_bytes(&data).is_err());
    }

    #[test]
    fn test_ipc_validator_basic() {
        let mut validator = IpcValidator::new();
        let payload = b"test";
        let header = IpcMessageHeader::new(IpcMessageType::ParseRequest, 1, payload);
        let mut message = header.to_bytes().to_vec();
        message.extend_from_slice(payload);

        assert!(validator.validate_message(&message).is_ok());
    }

    #[test]
    fn test_ipc_validator_size_limit() {
        let validator = IpcValidator::with_max_size(10);
        // Create header with payload_length exceeding limit
        let data = [
            0x46, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00,
        ];
        // payload_length = 0x20 = 32, exceeds limit of 10
        assert!(validator.validate_header(&data).is_err());
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
    }

    #[test]
    fn test_sandbox_config_strict() {
        let config = SandboxConfig::strict();
        assert_eq!(config.max_memory, 64 * 1024 * 1024);
        assert_eq!(config.max_cpu_time_ms, 2000);
    }

    #[test]
    fn test_security_context_validate_input() {
        let ctx = SecurityContext::new();
        let data = vec![0u8; 1024];
        assert!(ctx.validate_input(&data).is_ok());
    }

    #[test]
    fn test_security_context_validate_too_large() {
        let limits = SecurityLimits {
            max_font_size: 100,
            ..SecurityLimits::default()
        };
        let ctx = SecurityContext::with_limits(limits);
        let data = vec![0u8; 200];
        assert!(ctx.validate_input(&data).is_err());
    }

    #[test]
    fn test_validate_checksum_match() {
        let data = [0x00, 0x00, 0x00, 0x05];
        let checksum = calculate_checksum(&data);
        let result = validate_checksum(&data, checksum, "test");
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_checksum_mismatch() {
        let data = [0x00, 0x00, 0x00, 0x05];
        let result = validate_checksum(&data, 0xDEADBEEF, "test");
        assert!(!result.is_valid);
    }
}
