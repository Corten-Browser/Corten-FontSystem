//! text_shaper - Text shaping, bidirectional text, line breaking, and OpenType features

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod shaper;
pub mod types;

// Re-export main types for convenience
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
