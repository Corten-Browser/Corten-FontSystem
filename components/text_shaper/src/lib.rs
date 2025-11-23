//! text_shaper - Text shaping, bidirectional text, line breaking, and OpenType features
//!
//! This crate provides text shaping capabilities including:
//! - Unicode Bidirectional Algorithm (UAX #9) for mixed-direction text
//! - RTL (right-to-left) text support for Arabic, Hebrew, etc.
//! - Complex script shaping for Arabic, Indic, and Southeast Asian scripts
//! - OpenType feature support

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod bidi;
pub mod scripts;
pub mod shaper;
pub mod types;

// Re-export main types for convenience
pub use bidi::{BidiClass, BidiInfo, BidiLevel, BidiParagraph, BidiRun, ParagraphDirection};
pub use scripts::ScriptShaper;
pub use shaper::TextShaper;
pub use types::{Language, Script, ShapingError, ShapingOptions};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        // Tests will be added during TDD
        assert!(true);
    }
}
