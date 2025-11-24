//! text_shaper - Text shaping, bidirectional text, line breaking, and OpenType features
//!
//! This crate provides text shaping capabilities including:
//! - Unicode Bidirectional Algorithm (UAX #9) for mixed-direction text
//! - RTL (right-to-left) text support for Arabic, Hebrew, etc.
//! - Complex script shaping for Arabic, Indic, and Southeast Asian scripts
//! - OpenType feature support
//! - Native Rust shaper (gradual replacement for HarfBuzz)
//! - Parallel batch shaping for performance (FEAT-046)

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod bidi;
pub mod native_shaper;
pub mod parallel;
pub mod scripts;
pub mod shaper;
pub mod types;

// Re-export main types for convenience
pub use bidi::{BidiClass, BidiInfo, BidiLevel, BidiParagraph, BidiRun, ParagraphDirection};
pub use native_shaper::NativeShaper;
pub use parallel::{
    BatchShapingResult, ParallelShaper, ParallelShapingConfig, SharedBatchShaper, TextRun,
    TextRunBatch,
};
pub use scripts::ScriptShaper;
pub use shaper::TextShaper;
pub use types::{Language, Script, ShapingError, ShapingOptions};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify module re-exports work correctly
        let script = Script::Latin;
        assert!(matches!(script, Script::Latin));
    }
}
