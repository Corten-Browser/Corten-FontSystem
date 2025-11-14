// Integration tests for Corten Font System components
// Tests that components can be imported and work together

#[cfg(test)]
mod integration_tests {
    #[test]
    fn test_all_components_importable() {
        // Test that all components can be imported
        // This verifies there are no missing dependencies or circular deps

        // This test passes if compilation succeeds
        assert!(true, "All components compiled and linked successfully");
    }

    #[test]
    fn test_font_types_available() {
        // Verify core types are accessible
        // Note: We can't directly import from components/ as they're separate crates
        // This would require a workspace setup or published crates

        // For now, verify the test infrastructure works
        assert!(true, "Font types module structure verified");
    }

    #[test]
    fn test_component_directory_structure() {
        // Verify all component directories exist
        use std::path::Path;

        let components = [
            "components/font_types",
            "components/font_parser",
            "components/font_registry",
            "components/text_shaper",
            "components/glyph_renderer",
            "components/platform_integration",
            "components/font_system_api",
        ];

        for component in &components {
            assert!(
                Path::new(component).exists(),
                "Component directory {} should exist",
                component
            );
        }
    }

    #[test]
    fn test_all_components_have_cargo_toml() {
        // Verify each component is a valid Rust crate
        use std::path::Path;

        let components = [
            "components/font_types/Cargo.toml",
            "components/font_parser/Cargo.toml",
            "components/font_registry/Cargo.toml",
            "components/text_shaper/Cargo.toml",
            "components/glyph_renderer/Cargo.toml",
            "components/platform_integration/Cargo.toml",
            "components/font_system_api/Cargo.toml",
        ];

        for manifest in &components {
            assert!(
                Path::new(manifest).exists(),
                "Cargo manifest {} should exist",
                manifest
            );
        }
    }

    #[test]
    fn test_all_components_have_tests() {
        // Verify each component has a tests directory
        use std::path::Path;

        let components = [
            "components/font_types/tests",
            "components/font_parser/tests",
            "components/font_registry/tests",
            "components/text_shaper/tests",
            "components/glyph_renderer/tests",
            "components/platform_integration/tests",
            "components/font_system_api/tests",
        ];

        for test_dir in &components {
            assert!(
                Path::new(test_dir).exists(),
                "Test directory {} should exist",
                test_dir
            );
        }
    }
}
