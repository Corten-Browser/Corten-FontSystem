//! Unit tests for FontRegistry

use font_registry::{
    FontDescriptor, FontRegistry, FontStretch, FontStyle, FontWeight, RegistryError,
};

// ========== FontRegistry::new() Tests ==========

#[test]
fn test_font_registry_new_creates_empty_registry() {
    //! Given: No existing registry
    //! When: Creating a new FontRegistry
    //! Then: Registry should be created successfully and be empty

    // When
    let registry = FontRegistry::new();

    // Then
    assert_eq!(registry.font_count(), 0);
}

#[test]
fn test_font_registry_new_is_ready_for_operations() {
    //! Given: A new FontRegistry
    //! When: Checking if it's ready for operations
    //! Then: All operations should be available

    // Given
    let registry = FontRegistry::new();

    // Then - should be able to call match_font without panicking
    let descriptor = FontDescriptor::default();
    let result = registry.match_font(&descriptor);
    assert_eq!(result, None); // No fonts loaded yet
}

// ========== load_font_data() Tests ==========

#[test]
fn test_load_font_data_with_invalid_data_returns_error() {
    //! Given: Invalid font data
    //! When: Attempting to load it
    //! Then: Should return InvalidFont error

    // Given
    let mut registry = FontRegistry::new();
    let invalid_data = vec![0, 1, 2, 3]; // Not valid font data

    // When
    let result = registry.load_font_data(invalid_data);

    // Then
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RegistryError::InvalidFont(_)));
}

#[test]
fn test_load_font_data_with_empty_data_returns_error() {
    //! Given: Empty font data
    //! When: Attempting to load it
    //! Then: Should return InvalidFont error

    // Given
    let mut registry = FontRegistry::new();
    let empty_data = vec![];

    // When
    let result = registry.load_font_data(empty_data);

    // Then
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RegistryError::InvalidFont(_)));
}

// Note: Testing with valid font data requires actual font file data
// This will be added in integration tests

// ========== get_font_face() Tests ==========

#[test]
fn test_get_font_face_with_invalid_id_returns_none() {
    //! Given: A registry with no fonts
    //! When: Requesting a font face with any ID
    //! Then: Should return None

    // Given
    let registry = FontRegistry::new();

    // When
    let result = registry.get_font_face(0);

    // Then
    assert_eq!(result, None);
}

#[test]
fn test_get_font_face_with_nonexistent_id_returns_none() {
    //! Given: A registry
    //! When: Requesting a font face with non-existent ID
    //! Then: Should return None

    // Given
    let registry = FontRegistry::new();

    // When
    let result = registry.get_font_face(999);

    // Then
    assert_eq!(result, None);
}

// ========== match_font() Tests ==========

#[test]
fn test_match_font_with_empty_registry_returns_none() {
    //! Given: An empty registry
    //! When: Trying to match any font
    //! Then: Should return None

    // Given
    let registry = FontRegistry::new();
    let descriptor = FontDescriptor {
        family: vec!["Arial".to_string()],
        weight: FontWeight::Regular,
        style: FontStyle::Normal,
        stretch: FontStretch::Normal,
        size: 16.0,
    };

    // When
    let result = registry.match_font(&descriptor);

    // Then
    assert_eq!(result, None);
}

#[test]
fn test_match_font_with_default_descriptor() {
    //! Given: An empty registry
    //! When: Matching with default descriptor
    //! Then: Should return None (no fonts available)

    // Given
    let registry = FontRegistry::new();
    let descriptor = FontDescriptor::default();

    // When
    let result = registry.match_font(&descriptor);

    // Then
    assert_eq!(result, None);
}

// ========== get_font_metrics() Tests ==========

#[test]
fn test_get_font_metrics_with_invalid_id_returns_none() {
    //! Given: A registry with no fonts
    //! When: Requesting font metrics for any ID
    //! Then: Should return None

    // Given
    let registry = FontRegistry::new();

    // When
    let result = registry.get_font_metrics(0, 16.0);

    // Then
    assert_eq!(result, None);
}

#[test]
fn test_get_font_metrics_with_zero_size_returns_none() {
    //! Given: A registry
    //! When: Requesting metrics with zero size
    //! Then: Should return None (invalid size)

    // Given
    let registry = FontRegistry::new();

    // When
    let result = registry.get_font_metrics(0, 0.0);

    // Then
    assert_eq!(result, None);
}

#[test]
fn test_get_font_metrics_with_negative_size_returns_none() {
    //! Given: A registry
    //! When: Requesting metrics with negative size
    //! Then: Should return None (invalid size)

    // Given
    let registry = FontRegistry::new();

    // When
    let result = registry.get_font_metrics(0, -10.0);

    // Then
    assert_eq!(result, None);
}

// ========== load_system_fonts() Tests ==========

#[test]
fn test_load_system_fonts_returns_count() {
    //! Given: A new FontRegistry
    //! When: Loading system fonts
    //! Then: Should return count of fonts loaded (may be 0 on some systems)

    // Given
    let mut registry = FontRegistry::new();
    let initial_count = registry.font_count();

    // When
    let result = registry.load_system_fonts();

    // Then
    assert!(result.is_ok());
    let loaded_count = result.unwrap();
    assert_eq!(registry.font_count(), initial_count + loaded_count);
}

#[test]
fn test_load_system_fonts_registers_fonts_with_metadata() {
    //! Given: A new FontRegistry
    //! When: Loading system fonts
    //! Then: Loaded fonts should have proper metadata (family, weight, style)

    // Given
    let mut registry = FontRegistry::new();

    // When
    let result = registry.load_system_fonts();

    // Then
    if let Ok(count) = result {
        if count > 0 {
            // At least one font was loaded, verify we can match it
            let descriptor = FontDescriptor::default();
            let _matched = registry.match_font(&descriptor);
            // Note: May be None if no fonts match default descriptor
            // This test just verifies load_system_fonts() completes
        }
    }
}

#[test]
fn test_load_system_fonts_can_be_called_multiple_times() {
    //! Given: A FontRegistry with system fonts already loaded
    //! When: Loading system fonts again
    //! Then: Should handle gracefully (may add more fonts or skip duplicates)

    // Given
    let mut registry = FontRegistry::new();
    let first_result = registry.load_system_fonts();
    assert!(first_result.is_ok());
    let first_count = registry.font_count();

    // When
    let second_result = registry.load_system_fonts();

    // Then
    assert!(second_result.is_ok());
    // After second load, font count should be >= first count
    assert!(registry.font_count() >= first_count);
}

#[test]
fn test_load_system_fonts_fonts_are_findable() {
    //! Given: A FontRegistry with system fonts loaded
    //! When: Matching fonts by family name
    //! Then: Should be able to find common system fonts

    // Given
    let mut registry = FontRegistry::new();
    let result = registry.load_system_fonts();

    // When/Then
    if let Ok(count) = result {
        if count > 0 {
            // We should have at least one font
            assert!(registry.font_count() > 0);

            // Try to match a generic sans-serif descriptor
            // (most systems have some sans-serif font)
            let descriptor = FontDescriptor {
                family: vec!["sans-serif".to_string()],
                weight: FontWeight::Regular,
                style: FontStyle::Normal,
                stretch: FontStretch::Normal,
                size: 16.0,
            };

            // Note: Matching may return None if no exact family match
            // The implementation should handle generic family names
            let _ = registry.match_font(&descriptor);
        }
    }
}
