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
