//! Types for font_registry component
//!
//! Note: These types should eventually be imported from font_types component
//! once it is fully implemented. For now, they are defined locally.

use thiserror::Error;

// Re-export types from font_types that are already available
pub use font_types::types::{FontStretch, FontStyle, FontWeight};

/// Font identifier
pub type FontId = usize;

/// Font selection descriptor
#[derive(Debug, Clone, PartialEq)]
pub struct FontDescriptor {
    /// Font family names (fallback chain)
    pub family: Vec<String>,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Font stretch
    pub stretch: FontStretch,
    /// Font size in pixels
    pub size: f32,
}

impl Default for FontDescriptor {
    fn default() -> Self {
        Self {
            family: vec!["sans-serif".to_string()],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            size: 16.0,
        }
    }
}

/// Font metrics and measurements
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Units per EM
    pub units_per_em: u16,
    /// Ascent (above baseline)
    pub ascent: f32,
    /// Descent (below baseline)
    pub descent: f32,
    /// Line gap
    pub line_gap: f32,
    /// Cap height
    pub cap_height: f32,
    /// X-height
    pub x_height: f32,
    /// Underline position
    pub underline_position: f32,
    /// Underline thickness
    pub underline_thickness: f32,
}

/// Loaded font face
#[derive(Debug, Clone, PartialEq)]
pub struct FontFace {
    /// Font identifier
    pub id: FontId,
    /// Family name
    pub family_name: String,
    /// PostScript name
    pub postscript_name: String,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Font stretch
    pub stretch: FontStretch,
    /// Font metrics
    pub metrics: FontMetrics,
    /// Path to font file (for system fonts, lazy loading)
    pub(crate) file_path: Option<std::path::PathBuf>,
    /// Raw font data (loaded eagerly or lazily)
    pub(crate) data: Option<Vec<u8>>,
    /// Whether this is a system font
    pub(crate) is_system_font: bool,
}

impl FontFace {
    /// Get reference to raw font data
    ///
    /// This is needed for text shaping and rendering operations.
    ///
    /// # Returns
    ///
    /// Reference to the raw font file data, or None if data not loaded
    ///
    /// # Note
    ///
    /// For system fonts loaded lazily, data may be None until explicitly loaded.
    /// Use FontRegistry::ensure_font_data_loaded() to load data on-demand.
    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    /// Check if this is a system font
    pub fn is_system_font(&self) -> bool {
        self.is_system_font
    }

    /// Get the file path if this font was loaded from a file
    pub fn file_path(&self) -> Option<&std::path::Path> {
        self.file_path.as_deref()
    }
}

/// Font registry errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RegistryError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid font data
    #[error("Invalid font: {0}")]
    InvalidFont(String),

    /// Duplicate font already loaded
    #[error("Duplicate font")]
    DuplicateFont,

    /// System fonts unavailable
    #[error("System fonts unavailable")]
    SystemFontsUnavailable,
}
