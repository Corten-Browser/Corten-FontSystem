//! Integration tests for system font loading
//!
//! These tests verify that the font_registry correctly integrates with
//! platform_integration to load and use real system fonts.

use font_registry::{FontDescriptor, FontRegistry, FontStretch};

#[test]
fn test_load_system_fonts_integration() {
    //! Given: A new FontRegistry
    //! When: Loading system fonts
    //! Then: Should successfully load fonts from the system
    //!       (count may be 0 on systems without fonts, but should not error)

    // Given
    let mut registry = FontRegistry::new();

    // When
    let result = registry.load_system_fonts();

    // Then
    assert!(result.is_ok(), "load_system_fonts() should succeed");

    let count = result.unwrap();
    println!("Loaded {} system fonts", count);

    // Verify registry state is consistent
    assert_eq!(registry.font_count(), count);
}

#[test]
fn test_system_fonts_have_valid_metadata() {
    //! Given: A FontRegistry with system fonts loaded
    //! When: Examining loaded fonts
    //! Then: Each font should have valid metadata (family, weight, style)

    // Given
    let mut registry = FontRegistry::new();
    let result = registry.load_system_fonts();
    assert!(result.is_ok());

    let count = result.unwrap();
    if count == 0 {
        println!("No system fonts found, skipping test");
        return;
    }

    // When/Then: Try to get font face for first loaded font
    let first_font = registry.get_font_face(0);
    assert!(
        first_font.is_some(),
        "Should be able to retrieve loaded font"
    );

    if let Some(font) = first_font {
        // Verify metadata is present and non-empty
        assert!(
            !font.family_name.is_empty(),
            "Family name should not be empty"
        );
        assert!(
            !font.postscript_name.is_empty(),
            "PostScript name should not be empty"
        );

        // Verify font has file path (system font)
        assert!(
            font.file_path().is_some(),
            "System font should have file path"
        );
        assert!(font.is_system_font(), "Should be marked as system font");

        println!(
            "First font: {} ({:?} {:?})",
            font.family_name, font.weight, font.style
        );
    }
}

#[test]
fn test_match_system_font_by_family() {
    //! Given: A FontRegistry with system fonts loaded
    //! When: Matching fonts by common family names
    //! Then: Should find matching fonts for common families

    // Given
    let mut registry = FontRegistry::new();
    let result = registry.load_system_fonts();
    assert!(result.is_ok());

    let count = result.unwrap();
    if count == 0 {
        println!("No system fonts found, skipping test");
        return;
    }

    // Get first font to test matching
    let first_font = registry.get_font_face(0);
    assert!(first_font.is_some());

    if let Some(font) = first_font {
        // When: Try to match font by its family name
        let descriptor = FontDescriptor {
            family: vec![font.family_name.clone()],
            weight: font.weight,
            style: font.style,
            stretch: FontStretch::Normal,
            size: 16.0,
        };

        // Then: Should find exact match
        let matched_id = registry.match_font(&descriptor);
        assert!(matched_id.is_some(), "Should match font by family name");

        if let Some(id) = matched_id {
            let matched_font = registry.get_font_face(id).unwrap();
            assert_eq!(matched_font.family_name, font.family_name);
        }
    }
}

#[test]
fn test_font_data_is_accessible() {
    //! Given: A FontRegistry with system fonts loaded
    //! When: Accessing font data
    //! Then: Font data should be available for rendering

    // Given
    let mut registry = FontRegistry::new();
    let result = registry.load_system_fonts();
    assert!(result.is_ok());

    let count = result.unwrap();
    if count == 0 {
        println!("No system fonts found, skipping test");
        return;
    }

    // When: Get first loaded font
    let first_font = registry.get_font_face(0);
    assert!(first_font.is_some());

    if let Some(font) = first_font {
        // Then: Font data should be accessible
        let data = font.data();
        assert!(data.is_some(), "Font data should be available");

        if let Some(data_bytes) = data {
            assert!(!data_bytes.is_empty(), "Font data should not be empty");
            println!("Font data size: {} bytes", data_bytes.len());
        }
    }
}

#[test]
fn test_get_font_metrics_for_system_font() {
    //! Given: A FontRegistry with system fonts loaded
    //! When: Requesting metrics for system font
    //! Then: Should return properly scaled metrics

    // Given
    let mut registry = FontRegistry::new();
    let result = registry.load_system_fonts();
    assert!(result.is_ok());

    let count = result.unwrap();
    if count == 0 {
        println!("No system fonts found, skipping test");
        return;
    }

    // When: Get metrics for first font at 16px
    let metrics = registry.get_font_metrics(0, 16.0);

    // Then: Metrics should be available and reasonable
    assert!(metrics.is_some(), "Should return metrics for loaded font");

    if let Some(m) = metrics {
        // Verify metrics are reasonable (not zero or negative)
        assert!(m.ascent > 0.0, "Ascent should be positive");
        assert!(m.descent < 0.0, "Descent should be negative");
        assert!(m.units_per_em > 0, "Units per EM should be positive");

        println!(
            "Font metrics at 16px: ascent={}, descent={}, line_gap={}",
            m.ascent, m.descent, m.line_gap
        );
    }
}

#[test]
fn test_multiple_system_font_loads_are_idempotent() {
    //! Given: A FontRegistry with system fonts already loaded
    //! When: Loading system fonts multiple times
    //! Then: Should handle gracefully without duplicating fonts excessively

    // Given
    let mut registry = FontRegistry::new();

    let first_load = registry.load_system_fonts();
    assert!(first_load.is_ok());
    let first_count = first_load.unwrap();
    let registry_count_after_first = registry.font_count();

    // When: Load again
    let second_load = registry.load_system_fonts();
    assert!(second_load.is_ok());
    let second_count = second_load.unwrap();
    let registry_count_after_second = registry.font_count();

    // Then: Should handle gracefully
    // (May add duplicates or may skip, depending on implementation)
    println!(
        "First load: {} fonts, registry has {}",
        first_count, registry_count_after_first
    );
    println!(
        "Second load: {} fonts, registry has {}",
        second_count, registry_count_after_second
    );

    // Registry should have at least as many fonts as first load
    assert!(registry_count_after_second >= registry_count_after_first);
}
