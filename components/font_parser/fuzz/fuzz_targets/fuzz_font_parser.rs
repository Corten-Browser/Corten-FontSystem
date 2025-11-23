//! Fuzz target for the main font parser
//!
//! This fuzzer tests the font parser with arbitrary input data,
//! looking for crashes, panics, and security issues.

#![no_main]

use libfuzzer_sys::fuzz_target;
use font_parser::{OpenTypeFont, WoffFont, Woff2Font};
use font_parser::validation::{FontValidator, quick_validate};
use font_parser::security::SecurityContext;

fuzz_target!(|data: &[u8]| {
    // Skip trivially small inputs
    if data.len() < 4 {
        return;
    }

    // Create security context for validation
    let ctx = SecurityContext::new();

    // Quick validation should not panic
    let _ = quick_validate(data);

    // Full validation should not panic
    let validator = FontValidator::new();
    let _ = validator.validate_size(data);
    let _ = validator.validate_signature(data);

    // Try parsing as different formats based on signature
    let signature = if data.len() >= 4 {
        u32::from_be_bytes([data[0], data[1], data[2], data[3]])
    } else {
        return;
    };

    match signature {
        // WOFF
        0x774F4646 => {
            let _ = WoffFont::parse(data);
        }
        // WOFF2
        0x774F4632 => {
            let _ = Woff2Font::parse(data);
        }
        // TTF/OTF
        0x00010000 | 0x4F54544F => {
            let _ = OpenTypeFont::parse(data.to_vec());
        }
        // Unknown - still try parsing
        _ => {
            let _ = OpenTypeFont::parse(data.to_vec());
        }
    }
});
