# Font Parser Fuzz Testing

This directory contains fuzz testing infrastructure for the font_parser component.

## Prerequisites

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

Note: cargo-fuzz requires a nightly Rust toolchain.

## Running Fuzz Tests

### Main Font Parser Fuzzer

Tests the complete font parsing pipeline:

```bash
cd components/font_parser
cargo +nightly fuzz run fuzz_font_parser
```

### Checksum Fuzzer

Tests the OpenType checksum calculation:

```bash
cargo +nightly fuzz run fuzz_checksum
```

### Bounds Checker Fuzzer

Tests the bounds-checked data reader:

```bash
cargo +nightly fuzz run fuzz_bounds_checker
```

### IPC Validator Fuzzer

Tests the IPC message validation for sandboxed parsing:

```bash
cargo +nightly fuzz run fuzz_ipc_validator
```

## Common Options

### Run for a specific duration

```bash
cargo +nightly fuzz run fuzz_font_parser -- -max_total_time=300
```

### Run with specific number of jobs

```bash
cargo +nightly fuzz run fuzz_font_parser -- -jobs=4
```

### Use corpus from previous runs

Corpus is automatically saved in `fuzz/corpus/<target_name>/`.

### Minimize a crashing input

```bash
cargo +nightly fuzz tmin fuzz_font_parser crash-input.bin
```

### List all fuzz targets

```bash
cargo +nightly fuzz list
```

## Coverage

Generate coverage report:

```bash
cargo +nightly fuzz coverage fuzz_font_parser
```

## Adding New Fuzz Targets

1. Create a new file in `fuzz_targets/` directory
2. Add the target to `Cargo.toml` under `[[bin]]`
3. Use the `fuzz_target!` macro from `libfuzzer-sys`

Example:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use font_parser::YourModule;

fuzz_target!(|data: &[u8]| {
    let _ = YourModule::parse(data);
});
```

## Structured Fuzzing with Arbitrary

For more targeted fuzzing, use the `arbitrary` crate:

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct MyInput {
    field1: u32,
    field2: Vec<u8>,
}

fuzz_target!(|input: MyInput| {
    // Test with structured input
});
```

## Security Considerations

These fuzz targets are designed to find:

- Memory safety issues (buffer overflows, use-after-free)
- Denial of service (infinite loops, resource exhaustion)
- Panic conditions
- Logic errors

All parsing code should handle malformed input gracefully without panicking.
