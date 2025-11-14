//! Common types for font_system_api

/// FontError represents all possible errors in the font system
#[derive(Debug, Clone, PartialEq)]
pub enum FontError {
    /// Font was not found
    FontNotFound,
    /// Font data is invalid
    InvalidFont(String),
    /// Error loading font
    LoadError(String),
    /// Error during text shaping
    ShapingError(String),
    /// Error during glyph rendering
    RenderError(String),
    /// System-level error
    SystemError(String),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::FontNotFound => write!(f, "Font not found"),
            FontError::InvalidFont(msg) => write!(f, "Invalid font: {}", msg),
            FontError::LoadError(msg) => write!(f, "Load error: {}", msg),
            FontError::ShapingError(msg) => write!(f, "Shaping error: {}", msg),
            FontError::RenderError(msg) => write!(f, "Render error: {}", msg),
            FontError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for FontError {}

/// Configuration for FontSystem initialization
#[derive(Debug, Clone)]
pub struct FontSystemConfig {
    /// Cache size in megabytes
    pub cache_size_mb: usize,
    /// Enable subpixel rendering
    pub enable_subpixel: bool,
    /// Enable font hinting
    pub enable_hinting: bool,
    /// Load system fonts on initialization
    pub load_system_fonts_on_init: bool,
}

impl Default for FontSystemConfig {
    fn default() -> Self {
        Self {
            cache_size_mb: 64,
            enable_subpixel: true,
            enable_hinting: true,
            load_system_fonts_on_init: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FontError tests
    #[test]
    fn test_font_error_font_not_found() {
        let error = FontError::FontNotFound;
        assert_eq!(error.to_string(), "Font not found");
    }

    #[test]
    fn test_font_error_invalid_font() {
        let error = FontError::InvalidFont("corrupt header".to_string());
        assert_eq!(error.to_string(), "Invalid font: corrupt header");
    }

    #[test]
    fn test_font_error_load_error() {
        let error = FontError::LoadError("file not found".to_string());
        assert_eq!(error.to_string(), "Load error: file not found");
    }

    #[test]
    fn test_font_error_shaping_error() {
        let error = FontError::ShapingError("invalid script".to_string());
        assert_eq!(error.to_string(), "Shaping error: invalid script");
    }

    #[test]
    fn test_font_error_render_error() {
        let error = FontError::RenderError("out of memory".to_string());
        assert_eq!(error.to_string(), "Render error: out of memory");
    }

    #[test]
    fn test_font_error_system_error() {
        let error = FontError::SystemError("platform unavailable".to_string());
        assert_eq!(error.to_string(), "System error: platform unavailable");
    }

    #[test]
    fn test_font_error_clone() {
        let error = FontError::InvalidFont("test".to_string());
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_font_error_debug() {
        let error = FontError::FontNotFound;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("FontNotFound"));
    }

    // FontSystemConfig tests
    #[test]
    fn test_font_system_config_default() {
        let config = FontSystemConfig::default();
        assert_eq!(config.cache_size_mb, 64);
        assert!(config.enable_subpixel);
        assert!(config.enable_hinting);
        assert!(config.load_system_fonts_on_init);
    }

    #[test]
    fn test_font_system_config_custom() {
        let config = FontSystemConfig {
            cache_size_mb: 128,
            enable_subpixel: false,
            enable_hinting: false,
            load_system_fonts_on_init: false,
        };
        assert_eq!(config.cache_size_mb, 128);
        assert!(!config.enable_subpixel);
        assert!(!config.enable_hinting);
        assert!(!config.load_system_fonts_on_init);
    }

    #[test]
    fn test_font_system_config_clone() {
        let config = FontSystemConfig::default();
        let cloned = config.clone();
        assert_eq!(config.cache_size_mb, cloned.cache_size_mb);
        assert_eq!(config.enable_subpixel, cloned.enable_subpixel);
    }

    #[test]
    fn test_font_system_config_debug() {
        let config = FontSystemConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("FontSystemConfig"));
        assert!(debug_str.contains("cache_size_mb"));
    }
}
