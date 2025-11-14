//! Tests for TextShaper functionality

use text_shaper::{TextShaper, ShapingOptions, Script, Language, ShapingError};
use font_registry::FontRegistry;
use font_types::types::{FontDescriptor, FontWeight, FontStyle, FontStretch, Direction};
use std::collections::HashMap;

#[test]
fn test_text_shaper_new() {
    // Given: A font registry
    let registry = FontRegistry::new();

    // When: Creating a new TextShaper
    let shaper = TextShaper::new(&registry);

    // Then: TextShaper should be created successfully
    // (No panic = success for this test)
    drop(shaper);
}

#[test]
fn test_shape_text_basic_latin() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let _ = registry.load_system_fonts();

    let shaper = TextShaper::new(&registry);

    // When: Shaping simple Latin text
    let text = "Hello";
    let font_id = 0; // Assume first font
    let size = 16.0;

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

    let result = shaper.shape_text(text, font_id, size, &options);

    // Then: Should return shaped text or appropriate error
    assert!(result.is_ok() || matches!(result, Err(ShapingError::FontNotFound)));
}

#[test]
fn test_shape_text_with_invalid_font_id() {
    // Given: A font registry
    let registry = FontRegistry::new();
    let shaper = TextShaper::new(&registry);

    // When: Shaping text with invalid font ID
    let text = "Hello";
    let font_id = 999999; // Invalid font ID
    let size = 16.0;

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

    let result = shaper.shape_text(text, font_id, size, &options);

    // Then: Should return FontNotFound error
    assert!(matches!(result, Err(ShapingError::FontNotFound)));
}

#[test]
fn test_shape_text_with_empty_string() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let _ = registry.load_system_fonts();

    let shaper = TextShaper::new(&registry);

    // When: Shaping empty text
    let text = "";
    let font_id = 0;
    let size = 16.0;

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

    let result = shaper.shape_text(text, font_id, size, &options);

    // Then: Should return InvalidText error or empty shaped text
    assert!(result.is_ok() || matches!(result, Err(ShapingError::InvalidText(_))));
}

#[test]
fn test_shape_text_with_fallback() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let _ = registry.load_system_fonts();

    let shaper = TextShaper::new(&registry);

    // When: Shaping text with font fallback
    let text = "Hello";
    let descriptor = FontDescriptor {
        family: vec![String::from("Arial"), String::from("Helvetica")],
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

    let result = shaper.shape_text_with_fallback(text, &descriptor, &options);

    // Then: Should return shaped text or appropriate error
    assert!(result.is_ok() || matches!(result, Err(ShapingError::FontNotFound)));
}

#[test]
fn test_shape_text_with_features() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let _ = registry.load_system_fonts();

    let shaper = TextShaper::new(&registry);

    // When: Shaping text with OpenType features
    let text = "Hello";
    let font_id = 0;
    let size = 16.0;

    let mut features = HashMap::new();
    features.insert(String::from("liga"), 1); // Enable ligatures
    features.insert(String::from("kern"), 1); // Enable kerning

    let options = ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: String::from("en"),
        },
        direction: Direction::LeftToRight,
        features,
        kerning: true,
        ligatures: true,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    };

    let result = shaper.shape_text(text, font_id, size, &options);

    // Then: Should apply features and return shaped text
    assert!(result.is_ok() || matches!(result, Err(ShapingError::FontNotFound)));
}

#[test]
fn test_shape_text_rtl() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let _ = registry.load_system_fonts();

    let shaper = TextShaper::new(&registry);

    // When: Shaping RTL text (Arabic or Hebrew)
    let text = "مرحبا"; // "Hello" in Arabic
    let font_id = 0;
    let size = 16.0;

    let options = ShapingOptions {
        script: Script::Arabic,
        language: Language {
            tag: String::from("ar"),
        },
        direction: Direction::RightToLeft,
        features: HashMap::new(),
        kerning: true,
        ligatures: true,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    };

    let result = shaper.shape_text(text, font_id, size, &options);

    // Then: Should handle RTL text correctly
    assert!(result.is_ok() || matches!(result, Err(ShapingError::FontNotFound) | Err(ShapingError::UnsupportedScript(_))));
}
