//! Script-specific text shaping modules
//!
//! This module provides specialized shaping for complex scripts:
//! - Arabic: Contextual forms and ligatures
//! - Indic: Consonant clusters and vowel reordering
//! - Southeast Asian: Thai, Lao, and Khmer

pub mod arabic;
pub mod indic;
pub mod southeast_asian;

/// Trait for script-specific shapers
pub trait ScriptShaper {
    /// Shape the given text according to script rules
    fn shape(&self, text: &str) -> String;

    /// Check if this shaper can handle the given text
    fn can_shape(&self, text: &str) -> bool;
}
