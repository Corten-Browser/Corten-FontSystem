//! Security limits for font parsing
//!
//! This module defines constants that limit resource usage during font parsing
//! to prevent denial-of-service attacks and resource exhaustion.

/// Maximum font file size (100 MB)
///
/// Fonts larger than this are rejected as potentially malicious.
/// Legitimate fonts rarely exceed 20MB, so 100MB provides a generous margin.
pub const MAX_FONT_SIZE: usize = 100 * 1024 * 1024;

/// Maximum number of tables in a font
///
/// OpenType fonts typically have 10-30 tables. A limit of 1000 prevents
/// malicious fonts from causing excessive iteration.
pub const MAX_TABLE_COUNT: u16 = 1000;

/// Maximum recursion depth for nested structures
///
/// Some font structures (like CFF subroutines) can be nested. This limit
/// prevents stack overflow from deeply nested or circular references.
pub const MAX_RECURSION_DEPTH: usize = 100;

/// Maximum number of glyphs in a font
///
/// OpenType supports up to 65535 glyphs. This is the maximum valid value.
pub const MAX_GLYPH_COUNT: u32 = 65535;

/// Operation timeout in milliseconds
///
/// Long-running operations (like decompression) should be bounded.
/// 5 seconds is generous for any legitimate font operation.
pub const OPERATION_TIMEOUT_MS: u64 = 5000;

/// Maximum table size (50 MB)
///
/// No single table should exceed this size.
pub const MAX_TABLE_SIZE: usize = 50 * 1024 * 1024;

/// Maximum string length in font tables
///
/// Name strings and other text data are limited to prevent memory exhaustion.
pub const MAX_STRING_LENGTH: usize = 65535;

/// Maximum number of name records
///
/// Prevents excessive iteration in name table parsing.
pub const MAX_NAME_RECORDS: usize = 10000;

/// Maximum number of cmap subtables
///
/// Character mapping subtables are limited to prevent DoS.
pub const MAX_CMAP_SUBTABLES: usize = 100;

/// Maximum number of GSUB/GPOS lookups
///
/// Feature lookups are limited to prevent excessive processing.
pub const MAX_LOOKUPS: usize = 10000;

/// Maximum number of color palette entries
///
/// Color palettes are limited to prevent memory exhaustion.
pub const MAX_PALETTE_ENTRIES: usize = 65535;

/// Maximum number of variation axes
///
/// Variable fonts typically have 1-5 axes. 100 is extremely generous.
pub const MAX_VARIATION_AXES: usize = 100;

/// Maximum number of named instances
///
/// Named instances in variable fonts are limited.
pub const MAX_NAMED_INSTANCES: usize = 10000;

/// Maximum memory allocation per parsing operation (256 MB)
///
/// Total memory allocated during parsing should not exceed this limit.
pub const MAX_MEMORY_ALLOCATION: usize = 256 * 1024 * 1024;

/// Maximum contour count per glyph
///
/// Complex glyphs can have many contours, but there's a reasonable limit.
pub const MAX_CONTOURS_PER_GLYPH: usize = 1000;

/// Maximum points per contour
///
/// Each contour has a limit on the number of points.
pub const MAX_POINTS_PER_CONTOUR: usize = 10000;

/// Maximum components in a composite glyph
///
/// Composite glyphs reference other glyphs but must be bounded.
pub const MAX_COMPOSITE_COMPONENTS: usize = 100;

/// Maximum IPC message size (16 MB)
///
/// Messages for sandboxed parsing are bounded.
pub const MAX_IPC_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

/// Security limit configuration
///
/// Allows customization of security limits for specific use cases.
#[derive(Debug, Clone)]
pub struct SecurityLimits {
    /// Maximum font file size
    pub max_font_size: usize,
    /// Maximum number of tables
    pub max_table_count: u16,
    /// Maximum recursion depth
    pub max_recursion_depth: usize,
    /// Maximum glyph count
    pub max_glyph_count: u32,
    /// Operation timeout in milliseconds
    pub operation_timeout_ms: u64,
    /// Maximum table size
    pub max_table_size: usize,
    /// Maximum memory allocation
    pub max_memory_allocation: usize,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_font_size: MAX_FONT_SIZE,
            max_table_count: MAX_TABLE_COUNT,
            max_recursion_depth: MAX_RECURSION_DEPTH,
            max_glyph_count: MAX_GLYPH_COUNT,
            operation_timeout_ms: OPERATION_TIMEOUT_MS,
            max_table_size: MAX_TABLE_SIZE,
            max_memory_allocation: MAX_MEMORY_ALLOCATION,
        }
    }
}

impl SecurityLimits {
    /// Create a new SecurityLimits with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create strict security limits (more restrictive)
    pub fn strict() -> Self {
        Self {
            max_font_size: 50 * 1024 * 1024, // 50 MB
            max_table_count: 500,
            max_recursion_depth: 50,
            max_glyph_count: MAX_GLYPH_COUNT,
            operation_timeout_ms: 2000,
            max_table_size: 25 * 1024 * 1024,         // 25 MB
            max_memory_allocation: 128 * 1024 * 1024, // 128 MB
        }
    }

    /// Create relaxed security limits (for trusted fonts)
    pub fn relaxed() -> Self {
        Self {
            max_font_size: 200 * 1024 * 1024, // 200 MB
            max_table_count: MAX_TABLE_COUNT,
            max_recursion_depth: 200,
            max_glyph_count: MAX_GLYPH_COUNT,
            operation_timeout_ms: 10000,
            max_table_size: MAX_TABLE_SIZE,
            max_memory_allocation: 512 * 1024 * 1024, // 512 MB
        }
    }

    /// Check if a font size is acceptable
    pub fn check_font_size(&self, size: usize) -> Result<(), LimitExceeded> {
        if size > self.max_font_size {
            Err(LimitExceeded::FontTooLarge {
                size,
                limit: self.max_font_size,
            })
        } else {
            Ok(())
        }
    }

    /// Check if table count is acceptable
    pub fn check_table_count(&self, count: u16) -> Result<(), LimitExceeded> {
        if count > self.max_table_count {
            Err(LimitExceeded::TooManyTables {
                count,
                limit: self.max_table_count,
            })
        } else {
            Ok(())
        }
    }

    /// Check if recursion depth is acceptable
    pub fn check_recursion_depth(&self, depth: usize) -> Result<(), LimitExceeded> {
        if depth > self.max_recursion_depth {
            Err(LimitExceeded::RecursionTooDeep {
                depth,
                limit: self.max_recursion_depth,
            })
        } else {
            Ok(())
        }
    }

    /// Check if table size is acceptable
    pub fn check_table_size(&self, size: usize) -> Result<(), LimitExceeded> {
        if size > self.max_table_size {
            Err(LimitExceeded::TableTooLarge {
                size,
                limit: self.max_table_size,
            })
        } else {
            Ok(())
        }
    }

    /// Check if memory allocation is acceptable
    pub fn check_memory_allocation(&self, size: usize) -> Result<(), LimitExceeded> {
        if size > self.max_memory_allocation {
            Err(LimitExceeded::MemoryLimitExceeded {
                requested: size,
                limit: self.max_memory_allocation,
            })
        } else {
            Ok(())
        }
    }
}

/// Error indicating a security limit was exceeded
#[derive(Debug, Clone, PartialEq)]
pub enum LimitExceeded {
    /// Font file is too large
    FontTooLarge {
        /// Actual size
        size: usize,
        /// Maximum allowed
        limit: usize,
    },
    /// Too many tables in font
    TooManyTables {
        /// Actual count
        count: u16,
        /// Maximum allowed
        limit: u16,
    },
    /// Recursion depth exceeded
    RecursionTooDeep {
        /// Current depth
        depth: usize,
        /// Maximum allowed
        limit: usize,
    },
    /// Individual table too large
    TableTooLarge {
        /// Actual size
        size: usize,
        /// Maximum allowed
        limit: usize,
    },
    /// Memory allocation limit exceeded
    MemoryLimitExceeded {
        /// Requested allocation
        requested: usize,
        /// Maximum allowed
        limit: usize,
    },
    /// Operation timed out
    OperationTimeout {
        /// Operation name
        operation: String,
        /// Timeout in milliseconds
        timeout_ms: u64,
    },
    /// Glyph count exceeded
    TooManyGlyphs {
        /// Actual count
        count: u32,
        /// Maximum allowed
        limit: u32,
    },
}

impl std::fmt::Display for LimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LimitExceeded::FontTooLarge { size, limit } => {
                write!(
                    f,
                    "Font size {} bytes exceeds limit of {} bytes",
                    size, limit
                )
            }
            LimitExceeded::TooManyTables { count, limit } => {
                write!(f, "Table count {} exceeds limit of {}", count, limit)
            }
            LimitExceeded::RecursionTooDeep { depth, limit } => {
                write!(f, "Recursion depth {} exceeds limit of {}", depth, limit)
            }
            LimitExceeded::TableTooLarge { size, limit } => {
                write!(
                    f,
                    "Table size {} bytes exceeds limit of {} bytes",
                    size, limit
                )
            }
            LimitExceeded::MemoryLimitExceeded { requested, limit } => {
                write!(
                    f,
                    "Memory allocation {} bytes exceeds limit of {} bytes",
                    requested, limit
                )
            }
            LimitExceeded::OperationTimeout {
                operation,
                timeout_ms,
            } => {
                write!(
                    f,
                    "Operation '{}' timed out after {} ms",
                    operation, timeout_ms
                )
            }
            LimitExceeded::TooManyGlyphs { count, limit } => {
                write!(f, "Glyph count {} exceeds limit of {}", count, limit)
            }
        }
    }
}

impl std::error::Error for LimitExceeded {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = SecurityLimits::default();
        assert_eq!(limits.max_font_size, 100 * 1024 * 1024);
        assert_eq!(limits.max_table_count, 1000);
        assert_eq!(limits.max_recursion_depth, 100);
    }

    #[test]
    fn test_strict_limits() {
        let limits = SecurityLimits::strict();
        assert_eq!(limits.max_font_size, 50 * 1024 * 1024);
        assert_eq!(limits.max_table_count, 500);
        assert_eq!(limits.max_recursion_depth, 50);
    }

    #[test]
    fn test_relaxed_limits() {
        let limits = SecurityLimits::relaxed();
        assert_eq!(limits.max_font_size, 200 * 1024 * 1024);
        assert_eq!(limits.max_recursion_depth, 200);
    }

    #[test]
    fn test_check_font_size_ok() {
        let limits = SecurityLimits::default();
        assert!(limits.check_font_size(1024).is_ok());
    }

    #[test]
    fn test_check_font_size_exceeded() {
        let limits = SecurityLimits::default();
        let result = limits.check_font_size(200 * 1024 * 1024);
        assert!(result.is_err());
        match result.unwrap_err() {
            LimitExceeded::FontTooLarge { size, limit } => {
                assert_eq!(size, 200 * 1024 * 1024);
                assert_eq!(limit, 100 * 1024 * 1024);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_check_table_count_ok() {
        let limits = SecurityLimits::default();
        assert!(limits.check_table_count(50).is_ok());
    }

    #[test]
    fn test_check_table_count_exceeded() {
        let limits = SecurityLimits::default();
        let result = limits.check_table_count(1500);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_recursion_depth_ok() {
        let limits = SecurityLimits::default();
        assert!(limits.check_recursion_depth(50).is_ok());
    }

    #[test]
    fn test_check_recursion_depth_exceeded() {
        let limits = SecurityLimits::default();
        let result = limits.check_recursion_depth(150);
        assert!(result.is_err());
    }

    #[test]
    fn test_limit_exceeded_display() {
        let err = LimitExceeded::FontTooLarge {
            size: 200,
            limit: 100,
        };
        assert!(format!("{}", err).contains("200"));
        assert!(format!("{}", err).contains("100"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_FONT_SIZE, 100 * 1024 * 1024);
        assert_eq!(MAX_TABLE_COUNT, 1000);
        assert_eq!(MAX_RECURSION_DEPTH, 100);
        assert_eq!(MAX_GLYPH_COUNT, 65535);
        assert_eq!(OPERATION_TIMEOUT_MS, 5000);
    }
}
