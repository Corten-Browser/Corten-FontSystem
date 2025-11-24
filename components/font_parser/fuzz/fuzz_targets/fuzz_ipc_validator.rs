//! Fuzz target for IPC message validation
//!
//! Tests the IPC validator with arbitrary message data.

#![no_main]

use libfuzzer_sys::fuzz_target;
use font_parser::security::{IpcMessageHeader, IpcMessageType, IpcValidator};

fuzz_target!(|data: &[u8]| {
    // Try parsing as IPC header
    let _ = IpcMessageHeader::from_bytes(data);

    // Try validating as IPC message
    let mut validator = IpcValidator::new();

    // Register some expected response IDs
    for id in 0..10u32 {
        validator.expect_response(id);
    }

    // Validate header only
    let _ = validator.validate_header(data);

    // Validate full message
    let _ = validator.validate_message(data);

    // Test message type conversion
    if !data.is_empty() {
        let _ = IpcMessageType::from_byte(data[0]);
    }

    // Test header construction and roundtrip
    if data.len() >= 4 {
        let message_id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let header = IpcMessageHeader::new(
            IpcMessageType::ParseRequest,
            message_id,
            &data[4..],
        );
        let bytes = header.to_bytes();
        let _ = IpcMessageHeader::from_bytes(&bytes);
    }
});
