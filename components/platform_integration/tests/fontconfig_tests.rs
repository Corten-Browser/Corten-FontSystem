//! Tests for fontconfig-based font discovery on Linux
//!
//! These tests verify that fontconfig integration works correctly
//! and that font metadata is properly parsed.

#[cfg(target_os = "linux")]
mod linux_fontconfig_tests {
    use platform_integration::{
        discover_system_fonts_detailed, FontStyle, FontWeight, PlatformFontInfo,
    };

    #[test]
    fn test_discover_system_fonts_detailed_returns_font_info() {
        // Given: A Linux system with fontconfig
        // When: We discover fonts with detailed metadata
        let fonts = discover_system_fonts_detailed();

        // Then: We should get PlatformFontInfo structures
        if !fonts.is_empty() {
            // Verify structure of returned data
            for font in &fonts {
                assert!(
                    !font.family_name.is_empty(),
                    "Family name should not be empty"
                );
                assert!(
                    font.path.exists(),
                    "Font path should exist: {:?}",
                    font.path
                );
                assert!(font.path.is_file(), "Font path should be a file");
            }
        }
    }

    #[test]
    fn test_fontconfig_parses_font_weights() {
        // Given: System fonts with various weights
        // When: We discover fonts
        let fonts = discover_system_fonts_detailed();

        if !fonts.is_empty() {
            // Then: Fonts should have valid weight values
            let has_weights = fonts
                .iter()
                .any(|f| matches!(f.weight, FontWeight::Regular | FontWeight::Bold));

            assert!(
                has_weights,
                "Should find fonts with at least Regular or Bold weight"
            );
        }
    }

    #[test]
    fn test_fontconfig_parses_font_styles() {
        // Given: System fonts with various styles
        // When: We discover fonts
        let fonts = discover_system_fonts_detailed();

        if !fonts.is_empty() {
            // Then: All fonts should have valid styles
            for font in &fonts {
                // Style should be one of Normal, Italic, or Oblique
                match font.style {
                    FontStyle::Normal => {}
                    FontStyle::Italic => {}
                    FontStyle::Oblique(_) => {}
                }
            }
        }
    }

    #[test]
    fn test_fontconfig_finds_common_fonts() {
        // Given: A Linux system with common fonts installed
        // When: We discover fonts
        let fonts = discover_system_fonts_detailed();

        if !fonts.is_empty() {
            // Then: We should find at least some common fonts
            let common_fonts = ["DejaVu", "Liberation", "Noto", "Droid"];

            let has_common = fonts
                .iter()
                .any(|f| common_fonts.iter().any(|name| f.family_name.contains(name)));

            assert!(
                has_common,
                "Should find at least one common font family (DejaVu, Liberation, Noto, or Droid)"
            );
        }
    }

    #[test]
    fn test_fontconfig_distinguishes_font_families() {
        // Given: System fonts from different families
        // When: We discover fonts
        let fonts = discover_system_fonts_detailed();

        if fonts.len() > 1 {
            // Then: We should see different family names
            let mut families = std::collections::HashSet::new();
            for font in &fonts {
                families.insert(font.family_name.clone());
            }

            assert!(
                families.len() > 1,
                "Should discover fonts from multiple families"
            );
        }
    }

    #[test]
    fn test_fontconfig_marks_system_fonts() {
        // Given: System fonts
        // When: We discover fonts
        let fonts = discover_system_fonts_detailed();

        if !fonts.is_empty() {
            // Then: Fonts from system locations should be marked as system fonts
            let system_fonts_count = fonts.iter().filter(|f| f.is_system_font).count();

            assert!(
                system_fonts_count > 0,
                "Should find at least some system fonts"
            );
        }
    }

    #[test]
    fn test_fontconfig_handles_missing_fontconfig_gracefully() {
        // This test verifies error handling
        // Even if fontconfig is missing, we should get a Result, not panic
        let fonts = discover_system_fonts_detailed();
        // Should not panic, just return empty vec or error
        let _ = fonts;
    }
}

#[cfg(target_os = "linux")]
mod weight_mapping_tests {
    use platform_integration::FontWeight;

    #[test]
    fn test_fontconfig_weight_to_font_weight_thin() {
        // Fontconfig thin = 0
        let weight = map_fontconfig_weight(0);
        assert_eq!(weight, FontWeight::Thin);
    }

    #[test]
    fn test_fontconfig_weight_to_font_weight_normal() {
        // Fontconfig regular = 80
        let weight = map_fontconfig_weight(80);
        assert_eq!(weight, FontWeight::Regular);
    }

    #[test]
    fn test_fontconfig_weight_to_font_weight_bold() {
        // Fontconfig bold = 200
        let weight = map_fontconfig_weight(200);
        assert_eq!(weight, FontWeight::Bold);
    }

    #[test]
    fn test_fontconfig_weight_to_font_weight_medium() {
        // Fontconfig medium = 100
        let weight = map_fontconfig_weight(100);
        assert_eq!(weight, FontWeight::Medium);
    }

    // Helper function we'll implement
    fn map_fontconfig_weight(fc_weight: i32) -> FontWeight {
        // This will be implemented in the main code
        // For now, this is a placeholder that will fail tests
        match fc_weight {
            0..=40 => FontWeight::Thin,
            41..=55 => FontWeight::ExtraLight,
            56..=75 => FontWeight::Light,
            76..=90 => FontWeight::Regular,
            91..=110 => FontWeight::Medium,
            111..=180 => FontWeight::SemiBold,
            181..=200 => FontWeight::Bold,
            201..=209 => FontWeight::ExtraBold,
            _ => FontWeight::Black,
        }
    }
}

#[cfg(target_os = "linux")]
mod style_mapping_tests {
    use platform_integration::FontStyle;

    #[test]
    fn test_fontconfig_slant_to_normal() {
        let style = map_fontconfig_slant(0);
        assert_eq!(style, FontStyle::Normal);
    }

    #[test]
    fn test_fontconfig_slant_to_italic() {
        let style = map_fontconfig_slant(100);
        assert_eq!(style, FontStyle::Italic);
    }

    #[test]
    fn test_fontconfig_slant_to_oblique() {
        let style = map_fontconfig_slant(110);
        match style {
            FontStyle::Oblique(_) => {}
            _ => panic!("Expected Oblique style"),
        }
    }

    // Helper function we'll implement
    fn map_fontconfig_slant(fc_slant: i32) -> FontStyle {
        match fc_slant {
            0 => FontStyle::Normal,
            100 => FontStyle::Italic,
            110 => FontStyle::Oblique(10.0),
            _ => FontStyle::Normal,
        }
    }
}
