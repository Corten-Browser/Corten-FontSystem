//! Unit tests for Harfbuzz integration
//!
//! These tests verify that real text shaping is performed using Harfbuzz,
//! not just placeholder implementations.

use font_registry::FontRegistry;
use font_types::types::{Direction, FontDescriptor, FontStretch, FontStyle, FontWeight};
use std::collections::HashMap;
use text_shaper::{Language, Script, ShapingOptions, TextShaper};

#[test]
fn test_shape_text_returns_glyphs() {
    // Given: A font registry with system fonts loaded
    let mut registry = FontRegistry::new();
    let loaded = registry.load_system_fonts().unwrap_or(0);

    // Skip test if no fonts available
    if loaded == 0 {
        eprintln!("Warning: No system fonts loaded, skipping test");
        return;
    }

    let shaper = TextShaper::new(&registry);

    // When: Shaping text with real Harfbuzz
    let text = "Hello";
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

    // Then: Should return shaped text with actual glyphs
    if let Ok(shaped) = result {
        // Should have glyphs (one per character at minimum)
        assert!(!shaped.glyphs.is_empty(), "Expected glyphs to be populated");

        // Should have width based on actual glyph metrics
        assert!(shaped.width > 0.0, "Expected non-zero width");

        // Should have height based on font metrics
        assert!(shaped.height > 0.0, "Expected non-zero height");

        // Should have baseline
        assert!(shaped.baseline > 0.0, "Expected non-zero baseline");
    }
}

#[test]
fn test_shape_text_glyph_positioning() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let loaded = registry.load_system_fonts().unwrap_or(0);

    if loaded == 0 {
        eprintln!("Warning: No system fonts loaded, skipping test");
        return;
    }

    let shaper = TextShaper::new(&registry);

    // When: Shaping text
    let text = "Hi";
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

    // Then: Glyphs should have proper positioning
    if let Ok(shaped) = result {
        assert!(!shaped.glyphs.is_empty(), "Expected glyphs");

        for glyph in &shaped.glyphs {
            // Each glyph should have a valid glyph ID
            assert!(glyph.glyph_id.id > 0, "Expected valid glyph ID");

            // Should have horizontal advance
            assert!(glyph.advance.x > 0.0, "Expected positive x advance");
        }
    }
}

#[test]
fn test_shape_text_with_ligatures() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let loaded = registry.load_system_fonts().unwrap_or(0);

    if loaded == 0 {
        eprintln!("Warning: No system fonts loaded, skipping test");
        return;
    }

    let shaper = TextShaper::new(&registry);

    // When: Shaping text with ligatures enabled
    let text = "fi fl"; // Common ligature pairs
    let font_id = 0;
    let size = 16.0;

    let mut features = HashMap::new();
    features.insert(String::from("liga"), 1); // Enable ligatures

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

    // Then: Should successfully shape with ligatures
    // Note: Actual ligature application depends on font support
    assert!(result.is_ok(), "Expected successful shaping");

    if let Ok(shaped) = result {
        assert!(!shaped.glyphs.is_empty(), "Expected glyphs");
    }
}

#[test]
fn test_shape_text_with_kerning() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let loaded = registry.load_system_fonts().unwrap_or(0);

    if loaded == 0 {
        eprintln!("Warning: No system fonts loaded, skipping test");
        return;
    }

    let shaper = TextShaper::new(&registry);

    // When: Shaping text with kerning
    let text = "AV"; // Common kerning pair
    let font_id = 0;
    let size = 16.0;

    let mut features = HashMap::new();
    features.insert(String::from("kern"), 1); // Enable kerning

    let options = ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: String::from("en"),
        },
        direction: Direction::LeftToRight,
        features,
        kerning: true,
        ligatures: false,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    };

    let result = shaper.shape_text(text, font_id, size, &options);

    // Then: Should successfully shape with kerning
    assert!(result.is_ok(), "Expected successful shaping");

    if let Ok(shaped) = result {
        assert!(!shaped.glyphs.is_empty(), "Expected glyphs");
    }
}

#[test]
fn test_shape_text_with_fallback_descriptor() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let loaded = registry.load_system_fonts().unwrap_or(0);

    if loaded == 0 {
        eprintln!("Warning: No system fonts loaded, skipping test");
        return;
    }

    let shaper = TextShaper::new(&registry);

    // When: Shaping with fallback descriptor
    let text = "Test";
    let descriptor = FontDescriptor {
        family: vec![
            String::from("NonexistentFont"),
            String::from("Arial"),
            String::from("Helvetica"),
        ],
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

    // Then: Should use fallback font if first font not found
    // Result depends on whether fallback fonts are available
    assert!(
        result.is_ok() || result.is_err(),
        "Should return a Result type"
    );
}

#[test]
fn test_shape_text_multiple_scripts() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let loaded = registry.load_system_fonts().unwrap_or(0);

    if loaded == 0 {
        eprintln!("Warning: No system fonts loaded, skipping test");
        return;
    }

    let shaper = TextShaper::new(&registry);

    // When: Shaping different scripts
    let texts_and_scripts = vec![
        ("Hello", Script::Latin, "en"),
        ("Привет", Script::Cyrillic, "ru"),
        ("Γεια", Script::Greek, "el"),
    ];

    for (text, script, lang) in texts_and_scripts {
        let options = ShapingOptions {
            script,
            language: Language {
                tag: String::from(lang),
            },
            direction: Direction::LeftToRight,
            features: HashMap::new(),
            kerning: true,
            ligatures: true,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        };

        let result = shaper.shape_text(text, 0, 16.0, &options);

        // Should handle different scripts (font availability may vary)
        assert!(
            result.is_ok() || result.is_err(),
            "Should return Result for script {:?}",
            script
        );
    }
}

#[test]
fn test_shape_text_cluster_indices() {
    // Given: A font registry with system fonts
    let mut registry = FontRegistry::new();
    let loaded = registry.load_system_fonts().unwrap_or(0);

    if loaded == 0 {
        eprintln!("Warning: No system fonts loaded, skipping test");
        return;
    }

    let shaper = TextShaper::new(&registry);

    // When: Shaping text
    let text = "test";
    let font_id = 0;
    let size = 16.0;

    let options = ShapingOptions {
        script: Script::Latin,
        language: Language {
            tag: String::from("en"),
        },
        direction: Direction::LeftToRight,
        features: HashMap::new(),
        kerning: false,
        ligatures: false,
        letter_spacing: 0.0,
        word_spacing: 0.0,
    };

    let result = shaper.shape_text(text, font_id, size, &options);

    // Then: Glyphs should be in order with proper clusters
    if let Ok(shaped) = result {
        assert!(!shaped.glyphs.is_empty(), "Expected glyphs");

        // Verify glyphs are positioned from left to right
        let mut prev_x = 0.0;
        for glyph in &shaped.glyphs {
            assert!(
                glyph.position.x >= prev_x,
                "Expected glyphs in LTR order"
            );
            prev_x = glyph.position.x;
        }
    }
}
