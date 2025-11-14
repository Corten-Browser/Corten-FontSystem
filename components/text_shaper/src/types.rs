//! Common types for text_shaper

use std::collections::HashMap;
use thiserror::Error;

/// Unicode script identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    /// Latin script
    Latin,
    /// Arabic script
    Arabic,
    /// Hebrew script
    Hebrew,
    /// Cyrillic script
    Cyrillic,
    /// Greek script
    Greek,
    /// Han (Chinese) script
    Han,
    /// Hangul (Korean) script
    Hangul,
    /// Hiragana (Japanese) script
    Hiragana,
    /// Katakana (Japanese) script
    Katakana,
    /// Common script (shared characters)
    Common,
}

/// Language identifier with BCP 47 tag
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Language {
    /// BCP 47 language tag (e.g., "en-US", "fr-FR")
    pub tag: String,
}

/// Text shaping errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ShapingError {
    /// Font not found in registry
    #[error("Font not found")]
    FontNotFound,

    /// Invalid text input
    #[error("Invalid text: {0}")]
    InvalidText(String),

    /// Unsupported script
    #[error("Unsupported script: {0}")]
    UnsupportedScript(String),
}

/// Options for text shaping
#[derive(Debug, Clone)]
pub struct ShapingOptions {
    /// Unicode script
    pub script: Script,

    /// Language tag
    pub language: Language,

    /// Text direction
    pub direction: font_types::types::Direction,

    /// OpenType features to apply (feature tag -> value)
    pub features: HashMap<String, u32>,

    /// Enable kerning
    pub kerning: bool,

    /// Enable ligatures
    pub ligatures: bool,

    /// Additional letter spacing (in pixels)
    pub letter_spacing: f32,

    /// Additional word spacing (in pixels)
    pub word_spacing: f32,
}

// Custom Hash implementation for ShapingOptions
// HashMap doesn't implement Hash, so we convert to sorted vector for hashing
impl std::hash::Hash for ShapingOptions {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.script.hash(state);
        self.language.hash(state);
        // Direction needs to derive Hash in font_types, or we hash its discriminant
        std::mem::discriminant(&self.direction).hash(state);

        // Hash features as sorted vector
        let mut features: Vec<_> = self.features.iter().collect();
        features.sort_by_key(|(k, _)| *k);
        for (key, value) in features {
            key.hash(state);
            value.hash(state);
        }

        self.kerning.hash(state);
        self.ligatures.hash(state);
        // Hash floats as their bit representation
        self.letter_spacing.to_bits().hash(state);
        self.word_spacing.to_bits().hash(state);
    }
}
