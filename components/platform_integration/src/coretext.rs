//! CoreText integration for macOS
//!
//! This module provides macOS-specific font discovery and text rendering
//! using the CoreText API.

use crate::types::{FontCategory, FontStyle, FontWeight, PlatformFontInfo};
use std::collections::HashMap;
use std::path::PathBuf;

/// CoreText font collection wrapper
#[cfg(target_os = "macos")]
pub struct CoreTextFontCollection {
    /// System fonts discovered
    system_fonts: Vec<PlatformFontInfo>,
}

#[cfg(target_os = "macos")]
impl CoreTextFontCollection {
    /// Create a new CoreText font collection
    pub fn new() -> Self {
        Self {
            system_fonts: Vec::new(),
        }
    }

    /// Initialize the font collection by discovering system fonts
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of fonts discovered
    /// * `Err(String)` - Error message if initialization fails
    pub fn initialize(&mut self) -> Result<usize, String> {
        let fonts = discover_fonts_from_directories()?;
        self.system_fonts = fonts;
        Ok(self.system_fonts.len())
    }

    /// Get all discovered fonts
    pub fn fonts(&self) -> &[PlatformFontInfo] {
        &self.system_fonts
    }

    /// Find fonts by family name
    pub fn find_by_family(&self, family: &str) -> Vec<&PlatformFontInfo> {
        self.system_fonts
            .iter()
            .filter(|f| f.family_name.eq_ignore_ascii_case(family))
            .collect()
    }

    /// Get font count
    pub fn font_count(&self) -> usize {
        self.system_fonts.len()
    }

    /// Get fonts by trait (weight/style combination)
    pub fn find_by_traits(
        &self,
        weight: Option<FontWeight>,
        style: Option<FontStyle>,
    ) -> Vec<&PlatformFontInfo> {
        self.system_fonts
            .iter()
            .filter(|f| {
                let weight_matches = weight.map_or(true, |w| f.weight == w);
                let style_matches = style.map_or(true, |s| f.style == s);
                weight_matches && style_matches
            })
            .collect()
    }
}

#[cfg(target_os = "macos")]
impl Default for CoreTextFontCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover fonts from macOS font directories
#[cfg(target_os = "macos")]
fn discover_fonts_from_directories() -> Result<Vec<PlatformFontInfo>, String> {
    let mut fonts = Vec::new();

    // Standard macOS font directories
    let font_directories = get_macos_font_directories();

    for dir in font_directories {
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    // Handle .dfont bundles and font folders
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.filter_map(Result::ok) {
                            let sub_path = sub_entry.path();
                            if is_font_file(&sub_path) {
                                if let Some(font_info) = parse_font_file(&sub_path) {
                                    fonts.push(font_info);
                                }
                            }
                        }
                    }
                } else if is_font_file(&path) {
                    if let Some(font_info) = parse_font_file(&path) {
                        fonts.push(font_info);
                    }
                }
            }
        }
    }

    Ok(fonts)
}

/// Get standard macOS font directories
#[cfg(target_os = "macos")]
fn get_macos_font_directories() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/System/Library/Fonts"),
        PathBuf::from("/Library/Fonts"),
        PathBuf::from("/System/Library/Fonts/Supplemental"),
    ];

    // Add user fonts directory
    if let Some(home) = std::env::var_os("HOME") {
        let user_fonts = PathBuf::from(home).join("Library/Fonts");
        dirs.push(user_fonts);
    }

    dirs
}

/// Check if a file is a font file
#[cfg(target_os = "macos")]
fn is_font_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().unwrap_or("").to_lowercase().as_str(),
            "ttf" | "otf" | "ttc" | "otc" | "dfont" | "woff" | "woff2"
        )
    } else {
        false
    }
}

/// Parse a font file and extract metadata
#[cfg(target_os = "macos")]
fn parse_font_file(path: &std::path::Path) -> Option<PlatformFontInfo> {
    let data = std::fs::read(path).ok()?;
    let face = ttf_parser::Face::parse(&data, 0).ok()?;

    // Extract family name
    let family_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::FAMILY)
        .and_then(|name| name.to_string())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

    // Map CoreText weight values
    let weight = map_coretext_weight(face.weight().to_number());

    // Map style
    let style = if face.is_italic() {
        FontStyle::Italic
    } else if face.is_oblique() {
        FontStyle::Oblique(12.0) // Default oblique angle
    } else {
        FontStyle::Normal
    };

    // Determine if system font
    let is_system = path
        .to_str()
        .map(|s| s.starts_with("/System/Library/") || s.starts_with("/Library/Fonts"))
        .unwrap_or(false);

    Some(PlatformFontInfo::new(
        family_name,
        path.to_path_buf(),
        weight,
        style,
        is_system,
    ))
}

/// Map numeric weight to FontWeight enum (CoreText scale)
#[cfg(target_os = "macos")]
fn map_coretext_weight(weight: u16) -> FontWeight {
    // CoreText uses similar values to CSS font-weight
    match weight {
        0..=150 => FontWeight::Thin,
        151..=250 => FontWeight::ExtraLight,
        251..=350 => FontWeight::Light,
        351..=450 => FontWeight::Regular,
        451..=550 => FontWeight::Medium,
        551..=650 => FontWeight::SemiBold,
        651..=750 => FontWeight::Bold,
        751..=850 => FontWeight::ExtraBold,
        _ => FontWeight::Black,
    }
}

/// CoreText typesetter for advanced text layout
#[cfg(target_os = "macos")]
pub struct CoreTextTypesetter {
    /// Cached typesetter results
    cache: HashMap<String, TypesetResult>,
}

/// Result of typesetting operation
#[derive(Debug, Clone)]
pub struct TypesetResult {
    /// Lines of text
    pub lines: Vec<TypesetLine>,
    /// Total width
    pub width: f64,
    /// Total height
    pub height: f64,
}

/// A typeset line of text
#[derive(Debug, Clone)]
pub struct TypesetLine {
    /// Glyph runs in this line
    pub runs: Vec<GlyphRun>,
    /// Line origin
    pub origin: (f64, f64),
    /// Line width
    pub width: f64,
    /// Line ascent
    pub ascent: f64,
    /// Line descent
    pub descent: f64,
}

/// A run of glyphs with the same attributes
#[derive(Debug, Clone)]
pub struct GlyphRun {
    /// Font family for this run
    pub font_family: String,
    /// Glyph indices
    pub glyphs: Vec<u16>,
    /// Glyph positions (x, y pairs)
    pub positions: Vec<(f64, f64)>,
    /// Glyph advances
    pub advances: Vec<f64>,
}

#[cfg(target_os = "macos")]
impl CoreTextTypesetter {
    /// Create a new CoreText typesetter
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Typeset text with given constraints
    ///
    /// # Arguments
    ///
    /// * `text` - Text to typeset
    /// * `font_family` - Font family name
    /// * `font_size` - Font size in points
    /// * `max_width` - Maximum line width (None for unlimited)
    ///
    /// # Returns
    ///
    /// Typeset result with lines and dimensions
    pub fn typeset(
        &mut self,
        text: &str,
        font_family: &str,
        font_size: f64,
        max_width: Option<f64>,
    ) -> TypesetResult {
        let cache_key = format!(
            "{}:{}:{}:{}",
            text,
            font_family,
            font_size,
            max_width.unwrap_or(-1.0)
        );

        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        // Perform basic typesetting
        let result = perform_typesetting(text, font_family, font_size, max_width);

        self.cache.insert(cache_key, result.clone());
        result
    }

    /// Clear typesetter cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(target_os = "macos")]
impl Default for CoreTextTypesetter {
    fn default() -> Self {
        Self::new()
    }
}

/// Perform basic typesetting (simplified implementation)
#[cfg(target_os = "macos")]
fn perform_typesetting(
    text: &str,
    font_family: &str,
    font_size: f64,
    max_width: Option<f64>,
) -> TypesetResult {
    let mut lines = Vec::new();
    let mut current_y = 0.0;
    let line_height = font_size * 1.2; // Simple line height calculation

    // Simple line breaking at word boundaries
    let words: Vec<&str> = text.split_whitespace().collect();
    let char_width = font_size * 0.5; // Approximate character width
    let space_width = font_size * 0.25;

    let max_w = max_width.unwrap_or(f64::MAX);
    let mut current_line_words = Vec::new();
    let mut current_line_width = 0.0;

    for word in words {
        let word_width = word.len() as f64 * char_width;

        if current_line_width + word_width > max_w && !current_line_words.is_empty() {
            // Create line
            let line_text = current_line_words.join(" ");
            lines.push(create_typeset_line(
                &line_text,
                font_family,
                font_size,
                current_y,
            ));
            current_y += line_height;
            current_line_words.clear();
            current_line_width = 0.0;
        }

        if !current_line_words.is_empty() {
            current_line_width += space_width;
        }
        current_line_words.push(word);
        current_line_width += word_width;
    }

    // Add final line
    if !current_line_words.is_empty() {
        let line_text = current_line_words.join(" ");
        lines.push(create_typeset_line(
            &line_text,
            font_family,
            font_size,
            current_y,
        ));
        current_y += line_height;
    }

    let total_width = lines.iter().map(|l| l.width).fold(0.0, f64::max);

    TypesetResult {
        lines,
        width: total_width,
        height: current_y,
    }
}

/// Create a typeset line
#[cfg(target_os = "macos")]
fn create_typeset_line(text: &str, font_family: &str, font_size: f64, y: f64) -> TypesetLine {
    let char_width = font_size * 0.5;
    let mut positions = Vec::new();
    let mut advances = Vec::new();
    let mut glyphs = Vec::new();

    let mut x = 0.0;
    for ch in text.chars() {
        glyphs.push(ch as u16);
        positions.push((x, y));
        advances.push(char_width);
        x += char_width;
    }

    TypesetLine {
        runs: vec![GlyphRun {
            font_family: font_family.to_string(),
            glyphs,
            positions,
            advances,
        }],
        origin: (0.0, y),
        width: x,
        ascent: font_size * 0.8,
        descent: font_size * 0.2,
    }
}

/// Get default font families for macOS CoreText
#[cfg(target_os = "macos")]
pub fn get_coretext_defaults() -> HashMap<FontCategory, Vec<String>> {
    let mut defaults = HashMap::new();

    defaults.insert(
        FontCategory::Serif,
        vec![
            "Times New Roman".to_string(),
            "Georgia".to_string(),
            "Palatino".to_string(),
            "Baskerville".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::SansSerif,
        vec![
            "SF Pro".to_string(),
            "Helvetica Neue".to_string(),
            "Helvetica".to_string(),
            "Arial".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::Monospace,
        vec![
            "SF Mono".to_string(),
            "Menlo".to_string(),
            "Monaco".to_string(),
            "Courier New".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::Cursive,
        vec![
            "Apple Chancery".to_string(),
            "Brush Script MT".to_string(),
            "Snell Roundhand".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::Fantasy,
        vec![
            "Papyrus".to_string(),
            "Herculanum".to_string(),
            "Party LET".to_string(),
        ],
    );

    defaults.insert(FontCategory::Emoji, vec!["Apple Color Emoji".to_string()]);

    defaults
}

// Non-macOS stub implementations
#[cfg(not(target_os = "macos"))]
pub struct CoreTextFontCollection;

#[cfg(not(target_os = "macos"))]
impl CoreTextFontCollection {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize(&mut self) -> Result<usize, String> {
        Err("CoreText is only available on macOS".to_string())
    }

    pub fn fonts(&self) -> &[PlatformFontInfo] {
        &[]
    }

    pub fn find_by_family(&self, _family: &str) -> Vec<&PlatformFontInfo> {
        Vec::new()
    }

    pub fn font_count(&self) -> usize {
        0
    }

    pub fn find_by_traits(
        &self,
        _weight: Option<FontWeight>,
        _style: Option<FontStyle>,
    ) -> Vec<&PlatformFontInfo> {
        Vec::new()
    }
}

#[cfg(not(target_os = "macos"))]
impl Default for CoreTextFontCollection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "macos"))]
pub struct CoreTextTypesetter;

#[cfg(not(target_os = "macos"))]
impl CoreTextTypesetter {
    pub fn new() -> Self {
        Self
    }

    pub fn typeset(
        &mut self,
        _text: &str,
        _font_family: &str,
        _font_size: f64,
        _max_width: Option<f64>,
    ) -> TypesetResult {
        TypesetResult {
            lines: Vec::new(),
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn clear_cache(&mut self) {}
}

#[cfg(not(target_os = "macos"))]
impl Default for CoreTextTypesetter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_coretext_defaults() -> HashMap<FontCategory, Vec<String>> {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coretext_font_collection_new() {
        let collection = CoreTextFontCollection::new();
        assert_eq!(collection.font_count(), 0);
    }

    #[test]
    fn test_coretext_typesetter_new() {
        let _typesetter = CoreTextTypesetter::new();
        // Should not panic
    }

    #[test]
    fn test_typeset_result_default() {
        let result = TypesetResult {
            lines: Vec::new(),
            width: 0.0,
            height: 0.0,
        };
        assert!(result.lines.is_empty());
        assert_eq!(result.width, 0.0);
    }

    #[test]
    fn test_glyph_run_struct() {
        let run = GlyphRun {
            font_family: "Arial".to_string(),
            glyphs: vec![65, 66, 67],
            positions: vec![(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)],
            advances: vec![10.0, 10.0, 10.0],
        };
        assert_eq!(run.glyphs.len(), 3);
        assert_eq!(run.positions.len(), 3);
    }

    #[test]
    fn test_get_coretext_defaults() {
        let defaults = get_coretext_defaults();
        #[cfg(target_os = "macos")]
        {
            assert!(defaults.contains_key(&FontCategory::Serif));
            assert!(defaults.contains_key(&FontCategory::SansSerif));
            assert!(defaults.contains_key(&FontCategory::Emoji));
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(defaults.is_empty());
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_map_coretext_weight() {
        assert!(matches!(map_coretext_weight(100), FontWeight::Thin));
        assert!(matches!(map_coretext_weight(400), FontWeight::Regular));
        assert!(matches!(map_coretext_weight(700), FontWeight::Bold));
    }
}
