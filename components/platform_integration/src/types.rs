//! Types for platform_integration component

use std::path::PathBuf;

/// Font weight values (100-900)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontWeight {
    /// Thin weight (100)
    Thin = 100,
    /// Extra Light weight (200)
    ExtraLight = 200,
    /// Light weight (300)
    Light = 300,
    /// Regular weight (400)
    Regular = 400,
    /// Medium weight (500)
    Medium = 500,
    /// Semi Bold weight (600)
    SemiBold = 600,
    /// Bold weight (700)
    Bold = 700,
    /// Extra Bold weight (800)
    ExtraBold = 800,
    /// Black weight (900)
    Black = 900,
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    /// Normal (upright) style
    Normal,
    /// Italic style
    Italic,
    /// Oblique style with angle in degrees
    Oblique(f32),
}

/// Font categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontCategory {
    /// Serif fonts
    Serif,
    /// Sans-serif fonts
    SansSerif,
    /// Monospace fonts
    Monospace,
    /// Cursive fonts
    Cursive,
    /// Fantasy fonts
    Fantasy,
    /// Emoji fonts
    Emoji,
}

/// Supported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux
    Linux,
    /// Windows
    Windows,
    /// macOS
    MacOS,
    /// Unknown platform
    Unknown,
}

/// Platform-specific font information
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformFontInfo {
    /// Font family name
    pub family_name: String,
    /// Path to font file
    pub path: PathBuf,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Whether this is a system font
    pub is_system_font: bool,
}

impl PlatformFontInfo {
    /// Create a new PlatformFontInfo
    pub fn new(
        family_name: String,
        path: PathBuf,
        weight: FontWeight,
        style: FontStyle,
        is_system_font: bool,
    ) -> Self {
        Self {
            family_name,
            path,
            weight,
            style,
            is_system_font,
        }
    }
}
