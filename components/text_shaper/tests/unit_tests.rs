//! Unit tests for text_shaper
//! This is an integration test file that tests the public API

mod test_types {
    use std::collections::HashMap;
    use text_shaper::{Language, Script, ShapingError, ShapingOptions};

    #[test]
    fn test_script_variants_exist() {
        // Given: Script enum should have all variants from contract
        // When: Creating Script variants
        // Then: All should compile and be constructible
        let _latin = Script::Latin;
        let _arabic = Script::Arabic;
        let _hebrew = Script::Hebrew;
        let _cyrillic = Script::Cyrillic;
        let _greek = Script::Greek;
        let _han = Script::Han;
        let _hangul = Script::Hangul;
        let _hiragana = Script::Hiragana;
        let _katakana = Script::Katakana;
        let _common = Script::Common;
    }

    #[test]
    fn test_language_has_tag_field() {
        // Given: Language struct should have tag field
        // When: Creating Language with BCP 47 tag
        let language = Language {
            tag: String::from("en-US"),
        };

        // Then: tag should be accessible
        assert_eq!(language.tag, "en-US");
    }

    #[test]
    fn test_shaping_error_variants_exist() {
        // Given: ShapingError enum should have all variants from contract
        // When: Creating ShapingError variants
        // Then: All should compile
        let _font_not_found = ShapingError::FontNotFound;
        let _invalid_text = ShapingError::InvalidText(String::from("test"));
        let _unsupported_script = ShapingError::UnsupportedScript(String::from("test"));
    }

    #[test]
    fn test_shaping_error_display() {
        // Given: ShapingError should implement Display
        // When: Converting to string
        let error = ShapingError::InvalidText(String::from("invalid input"));

        // Then: Should have meaningful message
        let message = error.to_string();
        assert!(message.contains("invalid") || message.contains("Invalid"));
    }

    #[test]
    fn test_shaping_options_default_construction() {
        // Given: ShapingOptions struct should be constructible
        // When: Creating ShapingOptions with default values
        let options = ShapingOptions {
            script: Script::Latin,
            language: Language {
                tag: String::from("en"),
            },
            direction: font_types::types::Direction::LeftToRight,
            features: HashMap::new(),
            kerning: true,
            ligatures: true,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        };

        // Then: All fields should be accessible
        assert!(matches!(options.script, Script::Latin));
        assert_eq!(options.language.tag, "en");
        assert!(options.kerning);
        assert!(options.ligatures);
        assert_eq!(options.letter_spacing, 0.0);
        assert_eq!(options.word_spacing, 0.0);
    }

    #[test]
    fn test_shaping_options_with_features() {
        // Given: ShapingOptions should support OpenType features
        // When: Creating options with features
        let mut features = HashMap::new();
        features.insert(String::from("liga"), 1);
        features.insert(String::from("kern"), 1);

        let options = ShapingOptions {
            script: Script::Latin,
            language: Language {
                tag: String::from("en"),
            },
            direction: font_types::types::Direction::LeftToRight,
            features,
            kerning: true,
            ligatures: true,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        };

        // Then: Features should be accessible
        assert_eq!(options.features.get("liga"), Some(&1));
        assert_eq!(options.features.get("kern"), Some(&1));
    }

    #[test]
    fn test_shaping_options_with_spacing() {
        // Given: ShapingOptions should support custom spacing
        // When: Creating options with non-zero spacing
        let options = ShapingOptions {
            script: Script::Latin,
            language: Language {
                tag: String::from("en"),
            },
            direction: font_types::types::Direction::LeftToRight,
            features: HashMap::new(),
            kerning: true,
            ligatures: false,
            letter_spacing: 2.5,
            word_spacing: 5.0,
        };

        // Then: Spacing values should be preserved
        assert_eq!(options.letter_spacing, 2.5);
        assert_eq!(options.word_spacing, 5.0);
        assert!(!options.ligatures);
    }
}
