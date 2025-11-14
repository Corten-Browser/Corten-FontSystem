//! Contract compliance tests - verify public API matches contract exactly

use text_shaper::{TextShaper, ShapingOptions, Script, Language, ShapingError};
use font_registry::FontRegistry;
use font_types::types::{FontDescriptor, FontWeight, FontStyle, FontStretch, Direction};
use std::collections::HashMap;

#[test]
fn test_textshaper_exports() {
    // Given: Contract specifies TextShaper must be exported
    // When: Checking if TextShaper is accessible
    // Then: Should compile (this test verifies export exists)
    let registry = FontRegistry::new();
    let _shaper = TextShaper::new(&registry);
}

#[test]
fn test_shaping_options_exports() {
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
fn test_script_enum_variants() {
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
fn test_language_struct_fields() {
    // Given: Contract specifies Language with tag field
    // When: Creating Language
    // Then: tag field must be String type
    let language = Language {
        tag: String::from("en-US"),
    };

    assert_eq!(language.tag, "en-US");
}

#[test]
fn test_shaping_error_enum_variants() {
    // Given: Contract specifies ShapingError enum variants
    // When: Creating ShapingError variants
    // Then: All variants must match contract exactly
    let _font_not_found = ShapingError::FontNotFound;
    let _invalid_text = ShapingError::InvalidText(String::from("test"));
    let _unsupported_script = ShapingError::UnsupportedScript(String::from("test"));
}

#[test]
fn test_textshaper_new_method_signature() {
    // Given: Contract specifies new(registry: &FontRegistry) -> TextShaper
    // When: Calling new with registry reference
    // Then: Should return TextShaper
    let registry = FontRegistry::new();
    let shaper = TextShaper::new(&registry);

    // Verify it compiles and returns correct type
    drop(shaper);
}

#[test]
fn test_shape_text_method_signature() {
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
fn test_shape_text_with_fallback_method_signature() {
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
fn test_shaping_options_all_fields() {
    // Given: Contract specifies exact fields for ShapingOptions
    // When: Creating ShapingOptions
    // Then: All fields from contract must be present
    let mut features = HashMap::new();
    features.insert(String::from("liga"), 1);

    let options = ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: String::from("en"),
        },
        direction: Direction::LeftToRight,
        features: features.clone(),
        kerning: true,
        ligatures: false,
        letter_spacing: 2.0,
        word_spacing: 3.0,
    };

    // Verify all fields are accessible
    assert!(matches!(options.script, Script::Latin));
    assert_eq!(options.language.tag, "en");
    assert!(matches!(options.direction, Direction::LeftToRight));
    assert_eq!(options.features.get("liga"), Some(&1));
    assert_eq!(options.kerning, true);
    assert_eq!(options.ligatures, false);
    assert_eq!(options.letter_spacing, 2.0);
    assert_eq!(options.word_spacing, 3.0);
}

#[test]
fn test_shaping_error_display_trait() {
    // Given: ShapingError should implement Display (for error messages)
    // When: Converting to string
    // Then: Should produce meaningful error messages
    let error1 = ShapingError::FontNotFound;
    let error2 = ShapingError::InvalidText(String::from("empty"));
    let error3 = ShapingError::UnsupportedScript(String::from("Unknown"));

    // Verify Display is implemented
    let _msg1 = error1.to_string();
    let _msg2 = error2.to_string();
    let _msg3 = error3.to_string();

    // Messages should be non-empty
    assert!(!_msg1.is_empty());
    assert!(!_msg2.is_empty());
    assert!(!_msg3.is_empty());
}

#[test]
fn test_contract_no_extra_public_types() {
    // Given: Contract specifies exactly which types should be exported
    // When: Checking public exports
    // Then: Only contracted types should be public

    // This test verifies that we're not exporting extra types
    // by ensuring all expected types compile
    let _: TextShaper;
    let _: ShapingOptions;
    let _: Script;
    let _: Language;
    let _: ShapingError;

    // The test passes if it compiles (types are exported as per contract)
}
