//! Fuzz target for checksum calculation
//!
//! Tests the OpenType checksum calculation with arbitrary data.

#![no_main]

use libfuzzer_sys::fuzz_target;
use font_parser::security::{calculate_checksum, calculate_head_checksum, validate_checksum};

fuzz_target!(|data: &[u8]| {
    // Calculate checksum should not panic for any input
    let checksum = calculate_checksum(data);

    // Head checksum calculation should not panic
    let head_checksum = calculate_head_checksum(data);

    // Validate checksum should not panic
    let _ = validate_checksum(data, checksum, "test");
    let _ = validate_checksum(data, head_checksum, "head");

    // Validate with arbitrary expected value should not panic
    let _ = validate_checksum(data, 0xDEADBEEF, "arbitrary");
    let _ = validate_checksum(data, 0, "zero");
    let _ = validate_checksum(data, u32::MAX, "max");
});
