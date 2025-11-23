//! Font data validation utilities
//!
//! This module provides validation functions for font data structures,
//! ensuring data integrity and preventing parsing of malformed fonts.

use crate::limits::{
    LimitExceeded, SecurityLimits, MAX_CMAP_SUBTABLES, MAX_COMPOSITE_COMPONENTS,
    MAX_CONTOURS_PER_GLYPH, MAX_FONT_SIZE, MAX_LOOKUPS, MAX_NAMED_INSTANCES, MAX_NAME_RECORDS,
    MAX_PALETTE_ENTRIES, MAX_POINTS_PER_CONTOUR, MAX_STRING_LENGTH, MAX_VARIATION_AXES,
};
use crate::ParseError;

/// Result of font validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the font is valid
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with an error
    pub fn error(error: ValidationError) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Combine with another validation result
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Invalid font signature
    InvalidSignature {
        /// Expected signatures
        expected: Vec<u32>,
        /// Actual signature found
        found: u32,
    },
    /// Invalid checksum
    InvalidChecksum {
        /// Table tag
        table: String,
        /// Expected checksum
        expected: u32,
        /// Actual checksum
        found: u32,
    },
    /// Offset out of bounds
    OffsetOutOfBounds {
        /// Table or structure name
        context: String,
        /// Offset value
        offset: u64,
        /// Data size
        data_size: usize,
    },
    /// Invalid table structure
    InvalidTableStructure {
        /// Table tag
        table: String,
        /// Description of the issue
        reason: String,
    },
    /// Limit exceeded
    LimitExceeded(LimitExceeded),
    /// Circular reference detected
    CircularReference {
        /// Context where circular reference was found
        context: String,
    },
    /// Invalid data value
    InvalidValue {
        /// Field name
        field: String,
        /// Value description
        value: String,
        /// Why it's invalid
        reason: String,
    },
    /// Missing required field or table
    MissingRequired {
        /// What is missing
        item: String,
    },
    /// Overlapping table data
    OverlappingTables {
        /// First table
        table1: String,
        /// Second table
        table2: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidSignature { expected, found } => {
                write!(
                    f,
                    "Invalid font signature: expected one of {:08X?}, found {:08X}",
                    expected, found
                )
            }
            ValidationError::InvalidChecksum {
                table,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Invalid checksum for table '{}': expected {:08X}, found {:08X}",
                    table, expected, found
                )
            }
            ValidationError::OffsetOutOfBounds {
                context,
                offset,
                data_size,
            } => {
                write!(
                    f,
                    "Offset {} out of bounds in {}: data size is {}",
                    offset, context, data_size
                )
            }
            ValidationError::InvalidTableStructure { table, reason } => {
                write!(f, "Invalid structure in table '{}': {}", table, reason)
            }
            ValidationError::LimitExceeded(limit) => write!(f, "Limit exceeded: {}", limit),
            ValidationError::CircularReference { context } => {
                write!(f, "Circular reference detected in {}", context)
            }
            ValidationError::InvalidValue {
                field,
                value,
                reason,
            } => {
                write!(f, "Invalid value for '{}': {} ({})", field, value, reason)
            }
            ValidationError::MissingRequired { item } => {
                write!(f, "Missing required: {}", item)
            }
            ValidationError::OverlappingTables { table1, table2 } => {
                write!(f, "Overlapping table data: {} and {}", table1, table2)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validation warnings (non-fatal issues)
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationWarning {
    /// Unusual value that may indicate problems
    UnusualValue {
        /// Field name
        field: String,
        /// Value description
        value: String,
    },
    /// Deprecated feature used
    DeprecatedFeature {
        /// Feature name
        feature: String,
    },
    /// Non-standard extension
    NonStandard {
        /// What is non-standard
        item: String,
    },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationWarning::UnusualValue { field, value } => {
                write!(f, "Unusual value for '{}': {}", field, value)
            }
            ValidationWarning::DeprecatedFeature { feature } => {
                write!(f, "Deprecated feature: {}", feature)
            }
            ValidationWarning::NonStandard { item } => {
                write!(f, "Non-standard: {}", item)
            }
        }
    }
}

/// Font validator for comprehensive font file validation
#[derive(Debug)]
pub struct FontValidator {
    limits: SecurityLimits,
}

impl Default for FontValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl FontValidator {
    /// Create a new validator with default limits
    pub fn new() -> Self {
        Self {
            limits: SecurityLimits::default(),
        }
    }

    /// Create a validator with custom limits
    pub fn with_limits(limits: SecurityLimits) -> Self {
        Self { limits }
    }

    /// Validate font file size
    pub fn validate_size(&self, data: &[u8]) -> ValidationResult {
        if data.len() > self.limits.max_font_size {
            return ValidationResult::error(ValidationError::LimitExceeded(
                LimitExceeded::FontTooLarge {
                    size: data.len(),
                    limit: self.limits.max_font_size,
                },
            ));
        }
        ValidationResult::ok()
    }

    /// Validate font signature
    pub fn validate_signature(&self, data: &[u8]) -> ValidationResult {
        if data.len() < 4 {
            return ValidationResult::error(ValidationError::InvalidTableStructure {
                table: "header".to_string(),
                reason: "Data too short for signature".to_string(),
            });
        }

        let signature = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let valid_signatures = [
            0x00010000, // TrueType
            0x4F54544F, // OpenType/CFF ('OTTO')
            0x774F4646, // WOFF ('wOFF')
            0x774F4632, // WOFF2 ('wOF2')
        ];

        if !valid_signatures.contains(&signature) {
            return ValidationResult::error(ValidationError::InvalidSignature {
                expected: valid_signatures.to_vec(),
                found: signature,
            });
        }

        ValidationResult::ok()
    }

    /// Validate table count
    pub fn validate_table_count(&self, count: u16) -> ValidationResult {
        if count > self.limits.max_table_count {
            return ValidationResult::error(ValidationError::LimitExceeded(
                LimitExceeded::TooManyTables {
                    count,
                    limit: self.limits.max_table_count,
                },
            ));
        }
        ValidationResult::ok()
    }

    /// Validate offset is within bounds
    pub fn validate_offset(
        &self,
        offset: u64,
        length: u64,
        data_size: usize,
        context: &str,
    ) -> ValidationResult {
        if offset > data_size as u64 || offset.saturating_add(length) > data_size as u64 {
            return ValidationResult::error(ValidationError::OffsetOutOfBounds {
                context: context.to_string(),
                offset,
                data_size,
            });
        }
        ValidationResult::ok()
    }

    /// Validate table does not overlap with others
    pub fn validate_no_overlap(
        &self,
        tables: &[(String, u64, u64)], // (name, offset, length)
    ) -> ValidationResult {
        let mut result = ValidationResult::ok();

        for i in 0..tables.len() {
            let (name1, start1, len1) = &tables[i];
            let end1 = start1.saturating_add(*len1);

            for (name2, start2, len2) in tables.iter().skip(i + 1) {
                let end2 = start2.saturating_add(*len2);

                // Check for overlap
                if *start1 < end2 && *start2 < end1 {
                    result.add_error(ValidationError::OverlappingTables {
                        table1: name1.clone(),
                        table2: name2.clone(),
                    });
                }
            }
        }

        result
    }

    /// Validate string length
    pub fn validate_string_length(&self, length: usize, context: &str) -> ValidationResult {
        if length > MAX_STRING_LENGTH {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: context.to_string(),
                value: format!("{} characters", length),
                reason: format!("Exceeds maximum of {} characters", MAX_STRING_LENGTH),
            });
        }
        ValidationResult::ok()
    }

    /// Validate name record count
    pub fn validate_name_record_count(&self, count: usize) -> ValidationResult {
        if count > MAX_NAME_RECORDS {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "name record count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_NAME_RECORDS),
            });
        }
        ValidationResult::ok()
    }

    /// Validate cmap subtable count
    pub fn validate_cmap_subtable_count(&self, count: usize) -> ValidationResult {
        if count > MAX_CMAP_SUBTABLES {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "cmap subtable count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_CMAP_SUBTABLES),
            });
        }
        ValidationResult::ok()
    }

    /// Validate lookup count
    pub fn validate_lookup_count(&self, count: usize) -> ValidationResult {
        if count > MAX_LOOKUPS {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "lookup count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_LOOKUPS),
            });
        }
        ValidationResult::ok()
    }

    /// Validate variation axis count
    pub fn validate_axis_count(&self, count: usize) -> ValidationResult {
        if count > MAX_VARIATION_AXES {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "variation axis count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_VARIATION_AXES),
            });
        }
        ValidationResult::ok()
    }

    /// Validate named instance count
    pub fn validate_instance_count(&self, count: usize) -> ValidationResult {
        if count > MAX_NAMED_INSTANCES {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "named instance count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_NAMED_INSTANCES),
            });
        }
        ValidationResult::ok()
    }

    /// Validate palette entry count
    pub fn validate_palette_count(&self, count: usize) -> ValidationResult {
        if count > MAX_PALETTE_ENTRIES {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "palette entry count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_PALETTE_ENTRIES),
            });
        }
        ValidationResult::ok()
    }

    /// Validate contour count
    pub fn validate_contour_count(&self, count: usize) -> ValidationResult {
        if count > MAX_CONTOURS_PER_GLYPH {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "contour count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_CONTOURS_PER_GLYPH),
            });
        }
        ValidationResult::ok()
    }

    /// Validate point count
    pub fn validate_point_count(&self, count: usize) -> ValidationResult {
        if count > MAX_POINTS_PER_CONTOUR {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "point count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_POINTS_PER_CONTOUR),
            });
        }
        ValidationResult::ok()
    }

    /// Validate composite component count
    pub fn validate_component_count(&self, count: usize) -> ValidationResult {
        if count > MAX_COMPOSITE_COMPONENTS {
            return ValidationResult::error(ValidationError::InvalidValue {
                field: "composite component count".to_string(),
                value: count.to_string(),
                reason: format!("Exceeds maximum of {}", MAX_COMPOSITE_COMPONENTS),
            });
        }
        ValidationResult::ok()
    }
}

/// Sanitize potentially malformed font data
///
/// This function attempts to fix common issues in malformed font data.
#[derive(Debug, Default)]
pub struct FontSanitizer {
    /// Whether to truncate oversized data
    pub truncate_oversized: bool,
    /// Whether to fix invalid checksums
    pub fix_checksums: bool,
    /// Whether to remove invalid tables
    pub remove_invalid_tables: bool,
}

impl FontSanitizer {
    /// Create a new sanitizer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable all sanitization options
    pub fn strict() -> Self {
        Self {
            truncate_oversized: true,
            fix_checksums: true,
            remove_invalid_tables: true,
        }
    }

    /// Sanitize font data by rejecting oversized fonts
    pub fn sanitize(&self, data: &[u8]) -> Result<Vec<u8>, ParseError> {
        // Check size limit
        if data.len() > MAX_FONT_SIZE {
            if self.truncate_oversized {
                // We don't actually truncate fonts - that would corrupt them
                // Instead, reject them
                return Err(ParseError::CorruptedData(format!(
                    "Font size {} exceeds maximum {}",
                    data.len(),
                    MAX_FONT_SIZE
                )));
            }
            return Err(ParseError::CorruptedData(format!(
                "Font size {} exceeds maximum {}",
                data.len(),
                MAX_FONT_SIZE
            )));
        }

        // Validate signature
        if data.len() < 4 {
            return Err(ParseError::CorruptedData("Font data too short".to_string()));
        }

        let signature = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let valid_signatures = [0x00010000, 0x4F54544F, 0x774F4646, 0x774F4632];

        if !valid_signatures.contains(&signature) {
            return Err(ParseError::InvalidFormat);
        }

        // Return validated data
        Ok(data.to_vec())
    }
}

/// Quick validation of font data without full parsing
pub fn quick_validate(data: &[u8]) -> Result<(), ParseError> {
    let validator = FontValidator::new();

    // Check size
    let result = validator.validate_size(data);
    if !result.is_valid {
        return Err(ParseError::CorruptedData(result.errors[0].to_string()));
    }

    // Check signature
    let result = validator.validate_signature(data);
    if !result.is_valid {
        return Err(ParseError::InvalidFormat);
    }

    // Check minimum header size
    if data.len() < 12 {
        return Err(ParseError::CorruptedData(
            "Font data too short for header".to_string(),
        ));
    }

    // Check table count
    let num_tables = u16::from_be_bytes([data[4], data[5]]);
    let result = validator.validate_table_count(num_tables);
    if !result.is_valid {
        return Err(ParseError::CorruptedData(result.errors[0].to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_ok() {
        let result = ValidationResult::ok();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_result_error() {
        let result = ValidationResult::error(ValidationError::MissingRequired {
            item: "test".to_string(),
        });
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::ok();
        let result2 = ValidationResult::error(ValidationError::MissingRequired {
            item: "test".to_string(),
        });
        result1.merge(result2);
        assert!(!result1.is_valid);
        assert_eq!(result1.errors.len(), 1);
    }

    #[test]
    fn test_validate_size_ok() {
        let validator = FontValidator::new();
        let data = vec![0u8; 1024];
        let result = validator.validate_size(&data);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_size_exceeded() {
        let validator = FontValidator::with_limits(SecurityLimits {
            max_font_size: 100,
            ..SecurityLimits::default()
        });
        let data = vec![0u8; 200];
        let result = validator.validate_size(&data);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_signature_ttf() {
        let validator = FontValidator::new();
        let data = [0x00, 0x01, 0x00, 0x00]; // TrueType
        let result = validator.validate_signature(&data);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_signature_otf() {
        let validator = FontValidator::new();
        let data = [0x4F, 0x54, 0x54, 0x4F]; // OTTO
        let result = validator.validate_signature(&data);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_signature_woff() {
        let validator = FontValidator::new();
        let data = [0x77, 0x4F, 0x46, 0x46]; // wOFF
        let result = validator.validate_signature(&data);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_signature_invalid() {
        let validator = FontValidator::new();
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        let result = validator.validate_signature(&data);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_offset_ok() {
        let validator = FontValidator::new();
        let result = validator.validate_offset(10, 20, 100, "test");
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_offset_out_of_bounds() {
        let validator = FontValidator::new();
        let result = validator.validate_offset(90, 20, 100, "test");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_no_overlap_ok() {
        let validator = FontValidator::new();
        let tables = vec![
            ("table1".to_string(), 0, 100),
            ("table2".to_string(), 100, 100),
            ("table3".to_string(), 200, 100),
        ];
        let result = validator.validate_no_overlap(&tables);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_no_overlap_detected() {
        let validator = FontValidator::new();
        let tables = vec![
            ("table1".to_string(), 0, 150),
            ("table2".to_string(), 100, 100), // Overlaps with table1
        ];
        let result = validator.validate_no_overlap(&tables);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_sanitizer_size_limit() {
        let sanitizer = FontSanitizer::new();
        let data = vec![0u8; MAX_FONT_SIZE + 1];
        let result = sanitizer.sanitize(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitizer_invalid_signature() {
        let sanitizer = FontSanitizer::new();
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = sanitizer.sanitize(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitizer_valid_ttf() {
        let sanitizer = FontSanitizer::new();
        let mut data = vec![0x00, 0x01, 0x00, 0x00]; // TrueType signature
        data.extend(vec![0u8; 100]); // Padding
        let result = sanitizer.sanitize(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_validate_valid() {
        let mut data = vec![0x00, 0x01, 0x00, 0x00]; // TrueType
        data.extend(vec![0, 10]); // num_tables = 10
        data.extend(vec![0u8; 100]); // Rest of header
        let result = quick_validate(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_validate_too_short() {
        let data = vec![0x00, 0x01, 0x00, 0x00]; // Just signature
        let result = quick_validate(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::InvalidChecksum {
            table: "head".to_string(),
            expected: 0x12345678,
            found: 0x87654321,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("head"));
        assert!(msg.contains("12345678"));
        assert!(msg.contains("87654321"));
    }
}
