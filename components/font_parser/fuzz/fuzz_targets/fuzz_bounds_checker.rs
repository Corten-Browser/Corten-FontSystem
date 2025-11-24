//! Fuzz target for bounds-checked reader
//!
//! Tests the BoundsChecker with arbitrary data and operations.

#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use font_parser::security::BoundsChecker;

/// Operations to perform on the bounds checker
#[derive(Arbitrary, Debug)]
enum Operation {
    ReadU8,
    ReadU16,
    ReadI16,
    ReadU32,
    ReadI32,
    ReadBytes(u8),  // Read up to 255 bytes
    PeekU8,
    Seek(u16),      // Seek to position
    GetSlice { offset: u16, length: u8 },
}

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    data: Vec<u8>,
    operations: Vec<Operation>,
}

fuzz_target!(|input: FuzzInput| {
    let mut reader = BoundsChecker::new(&input.data);

    for op in input.operations {
        match op {
            Operation::ReadU8 => {
                let _ = reader.read_u8();
            }
            Operation::ReadU16 => {
                let _ = reader.read_u16();
            }
            Operation::ReadI16 => {
                let _ = reader.read_i16();
            }
            Operation::ReadU32 => {
                let _ = reader.read_u32();
            }
            Operation::ReadI32 => {
                let _ = reader.read_i32();
            }
            Operation::ReadBytes(count) => {
                let _ = reader.read_bytes(count as usize);
            }
            Operation::PeekU8 => {
                let _ = reader.peek_u8();
            }
            Operation::Seek(pos) => {
                let _ = reader.seek(pos as usize);
            }
            Operation::GetSlice { offset, length } => {
                let _ = reader.get_slice(offset as usize, length as usize);
            }
        }
    }

    // Verify invariants
    assert!(reader.position() <= reader.len());
});
