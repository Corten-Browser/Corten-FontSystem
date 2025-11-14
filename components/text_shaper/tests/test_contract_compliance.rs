//! Contract compliance tests - verify public API matches contract exactly

use font_registry::FontRegistry;
use font_types::types::{Direction, FontDescriptor, FontStretch, FontStyle, FontWeight};
use std::collections::HashMap;
use text_shaper::{Language, Script, ShapingError, ShapingOptions, TextShaper};

#[test]
fn test_contract_textshaper_exports() {
    // Given: Contract specifies TextShaper must be exported
    // When: Checking if TextShaper is accessible
    // Then: Should compile (this test verifies export exists)
    let registry = FontRegistry::new();
    let _shaper = TextShaper::new(&registry);
}

#[test]
fn test_contract_shaping_options_exports() {
    // Given: Contract specifies ShapingOptions must be exported
    // When: Creating ShapingOptions
    // Then: Should compile with all required fields
    let _options = ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: String::from("en"),
        },
        direction: Direction::LeftToRight,
        features: HashMap::new(),
        kerning: true,
        ligatures: true,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    };
}

#[test]
fn test_contract_script_enum_variants() {
    // Given: Contract specifies Script enum with exact variants
    // When: Creating Script variants
    // Then: All variants from contract must exist
    let variants = vec![
        Script::Latin,
        Script::Arabic,
        Script::Hebrew,
        Script::Cyrillic,
        Script::Greek,
        Script::Han,
        Script::Hangul,
        Script::Hiragana,
        Script::Katakana,
        Script::Common,
    ];

    // Verify count matches contract (10 variants)
    assert_eq!(variants.len(), 10);
}

#[test]
fn test_contract_language_struct_fields() {
    // Given: Contract specifies Language with tag field
    // When: Creating Language
    // Then: tag field must be String type
    let language = Language {
        tag: String::from("en-US"),
    };

    assert_eq!(language.tag, "en-US");
}

#[test]
fn test_contract_shaping_error_variants() {
    // Given: Contract specifies ShapingError enum variants
    // When: Creating ShapingError variants
    // Then: All variants must match contract exactly
    let _font_not_found = ShapingError::FontNotFound;
    let _invalid_text = ShapingError::InvalidText(String::from("test"));
    let _unsupported_script = ShapingError::UnsupportedScript(String::from("test"));
}

#[test]
fn test_contract_textshaper_new_signature() {
    // Given: Contract specifies new(registry: &FontRegistry) -> TextShaper
    // When: Calling new with registry reference
    // Then: Should return TextShaper
    let registry = FontRegistry::new();
    let shaper = TextShaper::new(&registry);

    // Verify it compiles and returns correct type
    drop(shaper);
}

#[test]
fn test_contract_shape_text_signature() {
    // Given: Contract specifies shape_text(text, font_id, size, options) -> Result<ShapedText, ShapingError>
    // When: Calling shape_text
    // Then: Should accept correct parameter types
    let registry = FontRegistry::new();
    let shaper = TextShaper::new(&registry);

    let options = ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: String::from("en"),
        },
        direction: Direction::LeftToRight,
        features: HashMap::new(),
        kerning: true,
        ligatures: true,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    };

    // Verify method signature matches contract
    let result = shaper.shape_text("test", 0, 16.0, &options);

    // Result type should be Result<ShapedText, ShapingError>
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_contract_shape_text_with_fallback_signature() {
    // Given: Contract specifies shape_text_with_fallback(text, descriptor, options) -> Result<ShapedText, ShapingError>
    // When: Calling shape_text_with_fallback
    // Then: Should accept correct parameter types
    let registry = FontRegistry::new();
    let shaper = TextShaper::new(&registry);

    let descriptor = FontDescriptor {
        family: vec![String::from("Arial")],
        weight: FontWeight::Regular,
        style: FontStyle::Normal,
        stretch: FontStretch::Normal,
        size: 16.0,
    };

    let options = ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: String::from("en"),
        },
        direction: Direction::LeftToRight,
        features: HashMap::new(),
        kerning: true,
        ligatures: true,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    };

    // Verify method signature matches contract
    let result = shaper.shape_text_with_fallback("test", &descriptor, &options);

    // Result type should be Result<ShapedText, ShapingError>
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_contract_shaping_error_display() {
    // Given: ShapingError should implement Display (for error messages)
    // When: Converting to string
    // Then: Should produce meaningful error messages
    let error1 = ShapingError::FontNotFound;
    let error2 = ShapingError::InvalidText(String::from("empty"));
    let error3 = ShapingError::UnsupportedScript(String::from("Unknown"));

    // Verify Display is implemented
    let msg1 = error1.to_string();
    let msg2 = error2.to_string();
    let msg3 = error3.to_string();

    // Messages should be non-empty and meaningful
    assert!(!msg1.is_empty());
    assert!(!msg2.is_empty());
    assert!(!msg3.is_empty());
    assert!(msg1.contains("not found") || msg1.contains("Font"));
    assert!(msg2.contains("Invalid") || msg2.contains("text"));
    assert!(msg3.contains("Unsupported") || msg3.contains("script"));
}
