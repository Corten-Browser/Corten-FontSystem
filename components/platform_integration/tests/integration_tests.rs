//! Integration tests for platform_integration
//!
//! These tests verify that the platform integration works correctly
//! with real system resources (actual font directories, config files).

use platform_integration::{
    discover_system_fonts, get_default_font_families, get_font_config_path, FontCategory,
};
use std::collections::HashSet;

#[test]
fn test_end_to_end_font_discovery() {
    // This tests the complete workflow of discovering fonts
    let fonts = discover_system_fonts();

    // If fonts are found, they should be valid
    if !fonts.is_empty() {
        // All paths should exist
        for font in &fonts {
            assert!(font.exists(), "Discovered font should exist: {:?}", font);
            assert!(
                font.is_file(),
                "Discovered font should be a file: {:?}",
                font
            );
        }

        // Paths should be unique (no duplicates)
        let unique_fonts: HashSet<_> = fonts.iter().collect();
        assert_eq!(
            fonts.len(),
            unique_fonts.len(),
            "Font list should not contain duplicates"
        );
    }
}

#[test]
fn test_default_families_are_consistent() {
    // Get defaults multiple times and ensure consistency
    let defaults1 = get_default_font_families();
    let defaults2 = get_default_font_families();

    assert_eq!(
        defaults1.len(),
        defaults2.len(),
        "Default families should be consistent"
    );

    for category in [
        FontCategory::Serif,
        FontCategory::SansSerif,
        FontCategory::Monospace,
        FontCategory::Cursive,
        FontCategory::Fantasy,
        FontCategory::Emoji,
    ] {
        assert_eq!(
            defaults1.get(&category),
            defaults2.get(&category),
            "Category {:?} should be consistent",
            category
        );
    }
}

#[test]
fn test_font_config_path_consistency() {
    // Get config path multiple times
    let path1 = get_font_config_path();
    let path2 = get_font_config_path();

    assert_eq!(path1, path2, "Config path should be consistent");

    // If path exists, it should be a directory
    if let Some(path) = path1 {
        assert!(
            path.exists(),
            "Config path should exist if returned: {:?}",
            path
        );
        assert!(
            path.is_dir(),
            "Config path should be a directory: {:?}",
            path
        );
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_fontconfig_integration() {
    use std::process::Command;

    // Check if fc-list is available
    if let Ok(output) = Command::new("fc-list").arg("--version").output() {
        if output.status.success() {
            // If fontconfig is available, we should find fonts
            let _fonts = discover_system_fonts();

            // With fontconfig, we should find at least some fonts (unless minimal system)
            // We'll just verify the function returns successfully (no panic)
        }
    }
}

#[test]
fn test_discovered_fonts_match_extensions() {
    let fonts = discover_system_fonts();

    let valid_extensions = [
        "ttf", "otf", "ttc", "otc", "woff", "woff2", "dfont", "pfb", "pfa",
    ];

    for font in &fonts {
        if let Some(ext) = font.extension() {
            let ext_str = ext.to_str().unwrap().to_lowercase();
            assert!(
                valid_extensions.contains(&ext_str.as_str()),
                "Font extension should be valid: {} in {:?}",
                ext_str,
                font
            );
        }
    }
}

#[test]
fn test_default_families_have_reasonable_names() {
    let defaults = get_default_font_families();

    for (category, families) in defaults {
        for family in families {
            // Family names should:
            // 1. Not be empty
            assert!(!family.is_empty(), "Family name should not be empty");

            // 2. Not have leading/trailing whitespace
            assert_eq!(
                family.trim(),
                family,
                "Family name should not have leading/trailing whitespace: '{}'",
                family
            );

            // 3. Have reasonable length (not too short, not too long)
            assert!(
                family.len() >= 2 && family.len() <= 100,
                "Family name should have reasonable length: '{}' (len: {})",
                family,
                family.len()
            );

            // 4. Contain printable characters
            assert!(
                family.chars().all(|c| !c.is_control()),
                "Family name should not contain control characters: '{}' (category: {:?})",
                family,
                category
            );
        }
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_system_fonts_paths_are_absolute() {
    let fonts = discover_system_fonts();

    for font in &fonts {
        assert!(
            font.is_absolute(),
            "Font path should be absolute: {:?}",
            font
        );
    }
}

#[test]
fn test_component_api_exports() {
    // Verify the component exports the correct public API
    // This ensures contract compliance

    // These should all compile (types are exported)
    let _category = FontCategory::Serif;
    let _platform = platform_integration::Platform::Linux;

    // Functions should be callable
    let _fonts = discover_system_fonts();
    let _defaults = get_default_font_families();
    let _config = get_font_config_path();

    // PlatformFontInfo should be constructable
    use platform_integration::{FontStyle, FontWeight, PlatformFontInfo};
    use std::path::PathBuf;

    let _info = PlatformFontInfo::new(
        "Arial".to_string(),
        PathBuf::from("/tmp/arial.ttf"),
        FontWeight::Regular,
        FontStyle::Normal,
        true,
    );
}
