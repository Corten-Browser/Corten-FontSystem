//! Unit tests for platform_integration

use platform_integration::{
    detect_platform, discover_system_fonts, get_default_font_families, get_font_config_path,
    FontCategory, Platform,
};

#[test]
fn test_detect_platform_returns_known_platform() {
    let platform = detect_platform();
    // On Linux CI we expect Linux, but generally should not be Unknown
    assert_ne!(platform, Platform::Unknown);
}

#[test]
fn test_discover_system_fonts_returns_valid_paths() {
    let fonts = discover_system_fonts();
    // Verify all returned paths have font extensions
    for font_path in &fonts {
        let ext = font_path.extension().and_then(|s| s.to_str());
        assert!(
            ext.is_some(),
            "Font path should have extension: {:?}",
            font_path
        );
        let ext = ext.unwrap().to_lowercase();
        assert!(
            matches!(
                ext.as_str(),
                "ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2" | "dfont" | "pfb" | "pfa"
            ),
            "Font should have valid extension, got: {}",
            ext
        );
    }
}

#[test]
fn test_discover_system_fonts_no_duplicates() {
    let fonts = discover_system_fonts();
    let mut unique = fonts.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        fonts.len(),
        unique.len(),
        "discover_system_fonts should not return duplicates"
    );
}

#[test]
fn test_get_default_font_families_has_all_categories() {
    let defaults = get_default_font_families();

    // All categories should be present
    assert!(defaults.contains_key(&FontCategory::Serif));
    assert!(defaults.contains_key(&FontCategory::SansSerif));
    assert!(defaults.contains_key(&FontCategory::Monospace));
    assert!(defaults.contains_key(&FontCategory::Cursive));
    assert!(defaults.contains_key(&FontCategory::Fantasy));
    assert!(defaults.contains_key(&FontCategory::Emoji));
}

#[test]
fn test_get_default_font_families_has_entries() {
    let defaults = get_default_font_families();

    // Each category should have at least one font
    for (category, fonts) in defaults.iter() {
        assert!(
            !fonts.is_empty(),
            "Category {:?} should have at least one font",
            category
        );
    }
}

#[test]
fn test_get_default_font_families_monospace() {
    let defaults = get_default_font_families();
    let monospace = defaults.get(&FontCategory::Monospace);

    assert!(monospace.is_some());
    let monospace = monospace.unwrap();
    assert!(!monospace.is_empty(), "Monospace should have fonts");

    // Common monospace fonts should be present on at least one platform
    let has_common = monospace.iter().any(|f| {
        f.contains("Mono")
            || f.contains("Courier")
            || f.contains("Consolas")
            || f.contains("Menlo")
            || f.contains("Monaco")
    });
    assert!(has_common, "Should have common monospace font");
}

#[test]
fn test_font_config_path_is_valid_if_present() {
    if let Some(config_path) = get_font_config_path() {
        // If a path is returned, it should exist
        assert!(
            config_path.exists(),
            "Config path should exist: {:?}",
            config_path
        );
    }
    // If None is returned, that's also valid (some systems may not have config)
}

#[test]
fn test_platform_linux_on_linux() {
    #[cfg(target_os = "linux")]
    {
        assert_eq!(detect_platform(), Platform::Linux);
    }
}

#[test]
fn test_platform_windows_on_windows() {
    #[cfg(target_os = "windows")]
    {
        assert_eq!(detect_platform(), Platform::Windows);
    }
}

#[test]
fn test_platform_macos_on_macos() {
    #[cfg(target_os = "macos")]
    {
        assert_eq!(detect_platform(), Platform::MacOS);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_discovers_fonts_in_common_locations() {
    let fonts = discover_system_fonts();

    // On Linux with fonts installed, we should find fonts
    // This might not always be true in minimal containers, so we check leniently
    if !fonts.is_empty() {
        // If fonts exist, at least one should be in a common location
        let has_common_path = fonts.iter().any(|p| {
            p.to_str()
                .map(|s| {
                    s.contains("/usr/share/fonts")
                        || s.contains("/usr/local/share/fonts")
                        || s.contains("/.fonts")
                        || s.contains("/.local/share/fonts")
                })
                .unwrap_or(false)
        });
        assert!(has_common_path, "Fonts should be in common directories");
    }
}

#[test]
fn test_default_families_returns_strings() {
    let defaults = get_default_font_families();

    for (category, families) in defaults {
        for family in families {
            assert!(
                !family.is_empty(),
                "Family name for {:?} should not be empty",
                category
            );
            assert!(
                family.chars().any(|c| c.is_alphanumeric()),
                "Family name should contain alphanumeric characters: {}",
                family
            );
        }
    }
}
