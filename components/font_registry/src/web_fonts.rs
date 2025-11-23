//! Web fonts support (@font-face CSS declarations)
//!
//! This module provides parsing and management of @font-face CSS declarations
//! for loading web fonts.

use crate::types::{FontStretch, FontStyle, FontWeight, RegistryError};
use std::collections::HashMap;

/// A parsed @font-face declaration
#[derive(Debug, Clone, PartialEq)]
pub struct FontFaceDeclaration {
    /// Font family name
    pub family: String,
    /// Font sources (URLs or local names)
    pub sources: Vec<FontSource>,
    /// Font weight (single value or range)
    pub weight: FontWeightRange,
    /// Font style
    pub style: FontStyle,
    /// Font stretch
    pub stretch: FontStretch,
    /// Unicode range restrictions
    pub unicode_range: Option<Vec<UnicodeRange>>,
    /// Font display strategy
    pub display: FontDisplay,
    /// Font feature settings
    pub feature_settings: Option<String>,
    /// Font variation settings
    pub variation_settings: Option<String>,
}

impl Default for FontFaceDeclaration {
    fn default() -> Self {
        Self {
            family: String::new(),
            sources: Vec::new(),
            weight: FontWeightRange::Single(FontWeight::Regular),
            style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            unicode_range: None,
            display: FontDisplay::Auto,
            feature_settings: None,
            variation_settings: None,
        }
    }
}

/// Font source specification
#[derive(Debug, Clone, PartialEq)]
pub enum FontSource {
    /// Local font reference
    Local(String),
    /// URL with optional format hint
    Url {
        /// Font URL
        url: String,
        /// Format hint (e.g., "woff2", "truetype")
        format: Option<String>,
    },
}

/// Font weight range for variable fonts
#[derive(Debug, Clone, PartialEq)]
pub enum FontWeightRange {
    /// Single weight value
    Single(FontWeight),
    /// Weight range (min, max)
    Range(FontWeight, FontWeight),
}

/// Unicode range for font subsetting
#[derive(Debug, Clone, PartialEq)]
pub struct UnicodeRange {
    /// Start codepoint
    pub start: u32,
    /// End codepoint (inclusive)
    pub end: u32,
}

impl UnicodeRange {
    /// Create a single codepoint range
    pub fn single(codepoint: u32) -> Self {
        Self {
            start: codepoint,
            end: codepoint,
        }
    }

    /// Create a codepoint range
    pub fn range(start: u32, end: u32) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    /// Check if a codepoint is within this range
    pub fn contains(&self, codepoint: u32) -> bool {
        codepoint >= self.start && codepoint <= self.end
    }
}

/// Font display strategy
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontDisplay {
    /// Browser-defined behavior (default)
    #[default]
    Auto,
    /// Short block period, infinite swap period
    Block,
    /// Extremely short block period, short swap period
    Swap,
    /// Extremely short block period, no swap period
    Fallback,
    /// No block period, no swap period
    Optional,
}

/// Font loading state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontLoadState {
    /// Font is being loaded
    Loading,
    /// Font loaded successfully
    Loaded,
    /// Font loading failed
    Failed,
}

/// Web font entry in the registry
#[derive(Debug, Clone)]
pub struct WebFont {
    /// The original declaration
    pub declaration: FontFaceDeclaration,
    /// Current loading state
    pub state: FontLoadState,
    /// Loaded font data (if any)
    pub data: Option<Vec<u8>>,
    /// Error message if loading failed
    pub error: Option<String>,
}

/// Web font manager for handling @font-face declarations
#[derive(Debug)]
pub struct WebFontManager {
    /// Registered font faces by family name
    font_faces: HashMap<String, Vec<WebFont>>,
    /// Pending font loads
    pending_loads: Vec<String>,
    /// Maximum concurrent downloads
    max_concurrent: usize,
}

impl WebFontManager {
    /// Create a new web font manager
    pub fn new() -> Self {
        Self {
            font_faces: HashMap::new(),
            pending_loads: Vec::new(),
            max_concurrent: 4,
        }
    }

    /// Register a @font-face declaration
    ///
    /// # Arguments
    ///
    /// * `declaration` - Parsed font face declaration
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn register(&mut self, declaration: FontFaceDeclaration) -> Result<(), RegistryError> {
        if declaration.family.is_empty() {
            return Err(RegistryError::InvalidFont("Empty font family name".to_string()));
        }

        if declaration.sources.is_empty() {
            return Err(RegistryError::InvalidFont("No font sources specified".to_string()));
        }

        let web_font = WebFont {
            declaration: declaration.clone(),
            state: FontLoadState::Loading,
            data: None,
            error: None,
        };

        self.font_faces
            .entry(declaration.family.clone())
            .or_default()
            .push(web_font);

        Ok(())
    }

    /// Parse and register a @font-face CSS rule
    ///
    /// # Arguments
    ///
    /// * `css` - CSS @font-face rule string
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    pub fn register_from_css(&mut self, css: &str) -> Result<(), RegistryError> {
        let declaration = parse_font_face_css(css)?;
        self.register(declaration)
    }

    /// Get all registered fonts for a family
    pub fn get_fonts(&self, family: &str) -> Option<&Vec<WebFont>> {
        self.font_faces.get(family)
    }

    /// Get mutable reference to fonts for a family
    pub fn get_fonts_mut(&mut self, family: &str) -> Option<&mut Vec<WebFont>> {
        self.font_faces.get_mut(family)
    }

    /// Check if a family has any registered fonts
    pub fn has_family(&self, family: &str) -> bool {
        self.font_faces.contains_key(family)
    }

    /// Get all registered family names
    pub fn families(&self) -> Vec<&String> {
        self.font_faces.keys().collect()
    }

    /// Find best matching font for criteria
    pub fn find_best_match(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&WebFont> {
        let fonts = self.font_faces.get(family)?;

        fonts.iter().min_by_key(|font| {
            let mut score = 0;

            // Weight score
            let font_weight = match &font.declaration.weight {
                FontWeightRange::Single(w) => *w,
                FontWeightRange::Range(min, max) => {
                    // Use closest edge of range
                    let w = weight as i32;
                    let min_i = *min as i32;
                    let max_i = *max as i32;
                    if w < min_i {
                        *min
                    } else if w > max_i {
                        *max
                    } else {
                        weight // Within range
                    }
                }
            };
            score += (weight as i32 - font_weight as i32).abs();

            // Style score
            if font.declaration.style != style {
                score += 1000;
            }

            score
        })
    }

    /// Remove a family from the registry
    pub fn remove_family(&mut self, family: &str) -> Option<Vec<WebFont>> {
        self.font_faces.remove(family)
    }

    /// Clear all registered fonts
    pub fn clear(&mut self) {
        self.font_faces.clear();
        self.pending_loads.clear();
    }

    /// Get total number of registered fonts
    pub fn font_count(&self) -> usize {
        self.font_faces.values().map(|v| v.len()).sum()
    }

    /// Set font data for a loaded font
    pub fn set_font_data(
        &mut self,
        family: &str,
        index: usize,
        data: Vec<u8>,
    ) -> Result<(), RegistryError> {
        let fonts = self
            .font_faces
            .get_mut(family)
            .ok_or_else(|| RegistryError::InvalidFont("Family not found".to_string()))?;

        let font = fonts
            .get_mut(index)
            .ok_or_else(|| RegistryError::InvalidFont("Font index out of bounds".to_string()))?;

        font.data = Some(data);
        font.state = FontLoadState::Loaded;
        Ok(())
    }

    /// Mark a font as failed
    pub fn set_font_error(&mut self, family: &str, index: usize, error: String) {
        if let Some(fonts) = self.font_faces.get_mut(family) {
            if let Some(font) = fonts.get_mut(index) {
                font.state = FontLoadState::Failed;
                font.error = Some(error);
            }
        }
    }
}

impl Default for WebFontManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a @font-face CSS rule
///
/// # Arguments
///
/// * `css` - CSS @font-face rule string
///
/// # Returns
///
/// Parsed font face declaration or error
pub fn parse_font_face_css(css: &str) -> Result<FontFaceDeclaration, RegistryError> {
    let mut declaration = FontFaceDeclaration::default();

    // Remove @font-face wrapper if present
    let content = css
        .trim()
        .strip_prefix("@font-face")
        .unwrap_or(css)
        .trim()
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(css)
        .trim();

    // Parse properties
    for property in content.split(';') {
        let property = property.trim();
        if property.is_empty() {
            continue;
        }

        let (key, value) = match property.split_once(':') {
            Some((k, v)) => (k.trim().to_lowercase(), v.trim()),
            None => continue,
        };

        match key.as_str() {
            "font-family" => {
                declaration.family = parse_font_family_value(value);
            }
            "src" => {
                declaration.sources = parse_src_value(value);
            }
            "font-weight" => {
                declaration.weight = parse_font_weight_value(value);
            }
            "font-style" => {
                declaration.style = parse_font_style_value(value);
            }
            "font-stretch" => {
                declaration.stretch = parse_font_stretch_value(value);
            }
            "unicode-range" => {
                declaration.unicode_range = Some(parse_unicode_range_value(value));
            }
            "font-display" => {
                declaration.display = parse_font_display_value(value);
            }
            "font-feature-settings" => {
                declaration.feature_settings = Some(value.to_string());
            }
            "font-variation-settings" => {
                declaration.variation_settings = Some(value.to_string());
            }
            _ => {} // Ignore unknown properties
        }
    }

    if declaration.family.is_empty() {
        return Err(RegistryError::InvalidFont(
            "Missing font-family in @font-face".to_string(),
        ));
    }

    Ok(declaration)
}

/// Parse font-family value
fn parse_font_family_value(value: &str) -> String {
    // Remove quotes
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

/// Parse src value for font sources
fn parse_src_value(value: &str) -> Vec<FontSource> {
    let mut sources = Vec::new();

    for source in value.split(',') {
        let source = source.trim();

        if source.starts_with("local(") {
            // Local font reference
            if let Some(name) = source
                .strip_prefix("local(")
                .and_then(|s| s.strip_suffix(')'))
            {
                sources.push(FontSource::Local(
                    name.trim().trim_matches('"').trim_matches('\'').to_string(),
                ));
            }
        } else if source.starts_with("url(") {
            // URL source
            let parts: Vec<&str> = source.splitn(2, ')').collect();
            if let Some(url_part) = parts.first() {
                let url = url_part
                    .strip_prefix("url(")
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                // Check for format hint
                let format = if parts.len() > 1 {
                    let rest = parts[1].trim();
                    if rest.starts_with("format(") {
                        rest.strip_prefix("format(")
                            .and_then(|s| s.strip_suffix(')'))
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                sources.push(FontSource::Url { url, format });
            }
        }
    }

    sources
}

/// Parse font-weight value
fn parse_font_weight_value(value: &str) -> FontWeightRange {
    let value = value.trim().to_lowercase();

    // Check for range (e.g., "100 900")
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.len() == 2 {
        let min = parse_single_weight(parts[0]);
        let max = parse_single_weight(parts[1]);
        return FontWeightRange::Range(min, max);
    }

    FontWeightRange::Single(parse_single_weight(&value))
}

/// Parse a single font weight value
fn parse_single_weight(value: &str) -> FontWeight {
    match value {
        "100" | "thin" => FontWeight::Thin,
        "200" | "extra-light" | "extralight" => FontWeight::ExtraLight,
        "300" | "light" => FontWeight::Light,
        "400" | "normal" | "regular" => FontWeight::Regular,
        "500" | "medium" => FontWeight::Medium,
        "600" | "semi-bold" | "semibold" => FontWeight::SemiBold,
        "700" | "bold" => FontWeight::Bold,
        "800" | "extra-bold" | "extrabold" => FontWeight::ExtraBold,
        "900" | "black" => FontWeight::Black,
        _ => FontWeight::Regular,
    }
}

/// Parse font-style value
fn parse_font_style_value(value: &str) -> FontStyle {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "normal" => FontStyle::Normal,
        "italic" => FontStyle::Italic,
        _ if value.starts_with("oblique") => {
            // Parse oblique angle
            let angle = value
                .strip_prefix("oblique")
                .and_then(|s| s.trim().strip_suffix("deg"))
                .and_then(|s| s.trim().parse::<f32>().ok())
                .unwrap_or(14.0);
            FontStyle::Oblique(angle)
        }
        _ => FontStyle::Normal,
    }
}

/// Parse font-stretch value
fn parse_font_stretch_value(value: &str) -> FontStretch {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "ultra-condensed" | "50%" => FontStretch::UltraCondensed,
        "extra-condensed" | "62.5%" => FontStretch::ExtraCondensed,
        "condensed" | "75%" => FontStretch::Condensed,
        "semi-condensed" | "87.5%" => FontStretch::SemiCondensed,
        "normal" | "100%" => FontStretch::Normal,
        "semi-expanded" | "112.5%" => FontStretch::SemiExpanded,
        "expanded" | "125%" => FontStretch::Expanded,
        "extra-expanded" | "150%" => FontStretch::ExtraExpanded,
        "ultra-expanded" | "200%" => FontStretch::UltraExpanded,
        _ => FontStretch::Normal,
    }
}

/// Parse unicode-range value
fn parse_unicode_range_value(value: &str) -> Vec<UnicodeRange> {
    let mut ranges = Vec::new();

    for part in value.split(',') {
        let part = part.trim();
        if part.starts_with("U+") || part.starts_with("u+") {
            let hex_part = &part[2..];
            if let Some((start, end)) = hex_part.split_once('-') {
                // Range
                if let (Ok(s), Ok(e)) = (
                    u32::from_str_radix(start, 16),
                    u32::from_str_radix(end, 16),
                ) {
                    ranges.push(UnicodeRange::range(s, e));
                }
            } else if hex_part.contains('?') {
                // Wildcard range (e.g., U+00??)
                let prefix = hex_part.replace('?', "0");
                let suffix = hex_part.replace('?', "F");
                if let (Ok(s), Ok(e)) = (
                    u32::from_str_radix(&prefix, 16),
                    u32::from_str_radix(&suffix, 16),
                ) {
                    ranges.push(UnicodeRange::range(s, e));
                }
            } else {
                // Single codepoint
                if let Ok(cp) = u32::from_str_radix(hex_part, 16) {
                    ranges.push(UnicodeRange::single(cp));
                }
            }
        }
    }

    ranges
}

/// Parse font-display value
fn parse_font_display_value(value: &str) -> FontDisplay {
    match value.trim().to_lowercase().as_str() {
        "auto" => FontDisplay::Auto,
        "block" => FontDisplay::Block,
        "swap" => FontDisplay::Swap,
        "fallback" => FontDisplay::Fallback,
        "optional" => FontDisplay::Optional,
        _ => FontDisplay::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_face_declaration_default() {
        let decl = FontFaceDeclaration::default();
        assert!(decl.family.is_empty());
        assert!(decl.sources.is_empty());
    }

    #[test]
    fn test_unicode_range_single() {
        let range = UnicodeRange::single(0x0041);
        assert!(range.contains(0x0041));
        assert!(!range.contains(0x0042));
    }

    #[test]
    fn test_unicode_range_range() {
        let range = UnicodeRange::range(0x0041, 0x005A);
        assert!(range.contains(0x0041)); // A
        assert!(range.contains(0x005A)); // Z
        assert!(!range.contains(0x0061)); // a
    }

    #[test]
    fn test_web_font_manager_new() {
        let manager = WebFontManager::new();
        assert_eq!(manager.font_count(), 0);
    }

    #[test]
    fn test_web_font_manager_register() {
        let mut manager = WebFontManager::new();
        let decl = FontFaceDeclaration {
            family: "Test Font".to_string(),
            sources: vec![FontSource::Url {
                url: "test.woff2".to_string(),
                format: Some("woff2".to_string()),
            }],
            ..Default::default()
        };

        assert!(manager.register(decl).is_ok());
        assert_eq!(manager.font_count(), 1);
        assert!(manager.has_family("Test Font"));
    }

    #[test]
    fn test_web_font_manager_register_empty_family() {
        let mut manager = WebFontManager::new();
        let decl = FontFaceDeclaration::default();
        assert!(manager.register(decl).is_err());
    }

    #[test]
    fn test_parse_font_face_css_basic() {
        let css = r#"@font-face {
            font-family: "Open Sans";
            src: url("opensans.woff2") format("woff2");
            font-weight: 400;
            font-style: normal;
        }"#;

        let result = parse_font_face_css(css);
        assert!(result.is_ok());
        let decl = result.unwrap();
        assert_eq!(decl.family, "Open Sans");
        assert!(!decl.sources.is_empty());
    }

    #[test]
    fn test_parse_font_weight_range() {
        let range = parse_font_weight_value("100 900");
        match range {
            FontWeightRange::Range(min, max) => {
                assert_eq!(min, FontWeight::Thin);
                assert_eq!(max, FontWeight::Black);
            }
            _ => panic!("Expected range"),
        }
    }

    #[test]
    fn test_parse_font_display() {
        assert_eq!(parse_font_display_value("swap"), FontDisplay::Swap);
        assert_eq!(parse_font_display_value("optional"), FontDisplay::Optional);
        assert_eq!(parse_font_display_value("invalid"), FontDisplay::Auto);
    }

    #[test]
    fn test_parse_src_local() {
        let sources = parse_src_value("local('Arial')");
        assert_eq!(sources.len(), 1);
        assert!(matches!(&sources[0], FontSource::Local(name) if name == "Arial"));
    }

    #[test]
    fn test_parse_src_url() {
        let sources = parse_src_value("url('test.woff2') format('woff2')");
        assert_eq!(sources.len(), 1);
        if let FontSource::Url { url, format } = &sources[0] {
            assert_eq!(url, "test.woff2");
            assert_eq!(format, &Some("woff2".to_string()));
        } else {
            panic!("Expected URL source");
        }
    }
}
