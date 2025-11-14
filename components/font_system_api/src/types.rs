//! Common types for font_system_api

/// Cache configuration for font system components
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Glyph cache configuration
    pub glyph_cache: GlyphCacheConfig,
    /// Shaping cache configuration
    pub shaping_cache: ShapingCacheConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            glyph_cache: GlyphCacheConfig::default(),
            shaping_cache: ShapingCacheConfig::default(),
        }
    }
}

/// Configuration for glyph renderer cache
#[derive(Debug, Clone)]
pub struct GlyphCacheConfig {
    /// Maximum number of cached glyphs (default: 10,000)
    pub max_entries: usize,
    /// Maximum memory usage in bytes (default: 100 MB)
    pub max_memory_bytes: usize,
    /// Enable cache statistics tracking
    pub enable_statistics: bool,
}

impl Default for GlyphCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            enable_statistics: true,
        }
    }
}

/// Configuration for text shaping cache
#[derive(Debug, Clone)]
pub struct ShapingCacheConfig {
    /// Maximum number of cached shaping results (default: 1,000)
    pub max_entries: usize,
    /// Enable cache statistics tracking
    pub enable_statistics: bool,
}

impl Default for ShapingCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1_000,
            enable_statistics: true,
        }
    }
}

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
    /// Cache configuration for glyph rendering and text shaping
    pub cache_config: CacheConfig,
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
            cache_config: CacheConfig::default(),
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

    // CacheConfig tests
    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.glyph_cache.max_entries, 10_000);
        assert_eq!(config.glyph_cache.max_memory_bytes, 100 * 1024 * 1024);
        assert!(config.glyph_cache.enable_statistics);
        assert_eq!(config.shaping_cache.max_entries, 1_000);
        assert!(config.shaping_cache.enable_statistics);
    }

    #[test]
    fn test_glyph_cache_config_custom() {
        let config = GlyphCacheConfig {
            max_entries: 20_000,
            max_memory_bytes: 200 * 1024 * 1024,
            enable_statistics: false,
        };
        assert_eq!(config.max_entries, 20_000);
        assert_eq!(config.max_memory_bytes, 200 * 1024 * 1024);
        assert!(!config.enable_statistics);
    }

    #[test]
    fn test_shaping_cache_config_custom() {
        let config = ShapingCacheConfig {
            max_entries: 2_000,
            enable_statistics: false,
        };
        assert_eq!(config.max_entries, 2_000);
        assert!(!config.enable_statistics);
    }

    // FontSystemConfig tests
    #[test]
    fn test_font_system_config_default() {
        let config = FontSystemConfig::default();
        assert_eq!(config.cache_config.glyph_cache.max_entries, 10_000);
        assert_eq!(
            config.cache_config.glyph_cache.max_memory_bytes,
            100 * 1024 * 1024
        );
        assert_eq!(config.cache_config.shaping_cache.max_entries, 1_000);
        assert!(config.enable_subpixel);
        assert!(config.enable_hinting);
        assert!(config.load_system_fonts_on_init);
    }

    #[test]
    fn test_font_system_config_custom() {
        let cache_config = CacheConfig {
            glyph_cache: GlyphCacheConfig {
                max_entries: 15_000,
                max_memory_bytes: 150 * 1024 * 1024,
                enable_statistics: false,
            },
            shaping_cache: ShapingCacheConfig {
                max_entries: 1_500,
                enable_statistics: false,
            },
        };

        let config = FontSystemConfig {
            cache_config,
            enable_subpixel: false,
            enable_hinting: false,
            load_system_fonts_on_init: false,
        };
        assert_eq!(config.cache_config.glyph_cache.max_entries, 15_000);
        assert!(!config.enable_subpixel);
        assert!(!config.enable_hinting);
        assert!(!config.load_system_fonts_on_init);
    }

    #[test]
    fn test_font_system_config_clone() {
        let config = FontSystemConfig::default();
        let cloned = config.clone();
        assert_eq!(
            config.cache_config.glyph_cache.max_entries,
            cloned.cache_config.glyph_cache.max_entries
        );
        assert_eq!(config.enable_subpixel, cloned.enable_subpixel);
    }

    #[test]
    fn test_font_system_config_debug() {
        let config = FontSystemConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("FontSystemConfig"));
        assert!(debug_str.contains("cache_config"));
    }
}
