//! DirectWrite integration for Windows
//!
//! This module provides Windows-specific font discovery and text rendering
//! using the DirectWrite API.

use crate::types::{FontCategory, FontStyle, FontWeight, PlatformFontInfo};
use std::collections::HashMap;
use std::path::PathBuf;

/// DirectWrite font collection wrapper
#[cfg(target_os = "windows")]
pub struct DirectWriteFontCollection {
    /// System font paths discovered
    system_fonts: Vec<PlatformFontInfo>,
}

#[cfg(target_os = "windows")]
impl DirectWriteFontCollection {
    /// Create a new DirectWrite font collection
    ///
    /// # Returns
    ///
    /// A new `DirectWriteFontCollection` instance
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
        // Discover fonts from Windows Fonts directory
        let fonts = discover_fonts_from_directory()?;
        self.system_fonts = fonts;
        Ok(self.system_fonts.len())
    }

    /// Get all discovered fonts
    pub fn fonts(&self) -> &[PlatformFontInfo] {
        &self.system_fonts
    }

    /// Find fonts by family name
    ///
    /// # Arguments
    ///
    /// * `family` - Font family name to search for
    ///
    /// # Returns
    ///
    /// Vector of matching font info
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
}

#[cfg(target_os = "windows")]
impl Default for DirectWriteFontCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover fonts from the Windows Fonts directory
///
/// This function enumerates fonts from the system Fonts directory
/// and extracts metadata from each font file.
#[cfg(target_os = "windows")]
fn discover_fonts_from_directory() -> Result<Vec<PlatformFontInfo>, String> {
    let mut fonts = Vec::new();

    // Get Windows Fonts directory
    let fonts_dir = get_windows_fonts_directory()
        .ok_or_else(|| "Cannot determine Windows Fonts directory".to_string())?;

    // Enumerate font files
    let entries =
        std::fs::read_dir(&fonts_dir).map_err(|e| format!("Cannot read fonts directory: {}", e))?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if is_font_file(&path) {
            if let Some(font_info) = parse_font_file(&path) {
                fonts.push(font_info);
            }
        }
    }

    Ok(fonts)
}

/// Get the Windows Fonts directory path
#[cfg(target_os = "windows")]
fn get_windows_fonts_directory() -> Option<PathBuf> {
    std::env::var_os("WINDIR").map(|windir| PathBuf::from(windir).join("Fonts"))
}

/// Check if a file is a font file based on extension
#[cfg(target_os = "windows")]
fn is_font_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().unwrap_or("").to_lowercase().as_str(),
            "ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2"
        )
    } else {
        false
    }
}

/// Parse a font file and extract metadata
#[cfg(target_os = "windows")]
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

    // Map weight
    let weight = map_weight(face.weight().to_number());

    // Map style
    let style = if face.is_italic() {
        FontStyle::Italic
    } else {
        FontStyle::Normal
    };

    Some(PlatformFontInfo::new(
        family_name,
        path.to_path_buf(),
        weight,
        style,
        true, // System font
    ))
}

/// Map numeric weight to FontWeight enum
#[cfg(target_os = "windows")]
fn map_weight(weight: u16) -> FontWeight {
    match weight {
        100 => FontWeight::Thin,
        200 => FontWeight::ExtraLight,
        300 => FontWeight::Light,
        350..=450 => FontWeight::Regular,
        500 => FontWeight::Medium,
        600 => FontWeight::SemiBold,
        700 => FontWeight::Bold,
        800 => FontWeight::ExtraBold,
        _ => FontWeight::Black,
    }
}

/// DirectWrite text analyzer for script detection and shaping
#[cfg(target_os = "windows")]
pub struct DirectWriteTextAnalyzer {
    /// Cached script analysis results
    script_cache: HashMap<String, Vec<ScriptRun>>,
}

/// A run of text with the same script
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptRun {
    /// Start position in the text
    pub start: usize,
    /// Length of the run
    pub length: usize,
    /// ISO 15924 script code
    pub script: String,
}

#[cfg(target_os = "windows")]
impl DirectWriteTextAnalyzer {
    /// Create a new text analyzer
    pub fn new() -> Self {
        Self {
            script_cache: HashMap::new(),
        }
    }

    /// Analyze text for script runs
    ///
    /// # Arguments
    ///
    /// * `text` - Text to analyze
    ///
    /// # Returns
    ///
    /// Vector of script runs
    pub fn analyze_script(&mut self, text: &str) -> Vec<ScriptRun> {
        // Check cache first
        if let Some(cached) = self.script_cache.get(text) {
            return cached.clone();
        }

        // Perform basic script detection
        let runs = detect_script_runs(text);

        // Cache result
        self.script_cache.insert(text.to_string(), runs.clone());

        runs
    }

    /// Clear the script cache
    pub fn clear_cache(&mut self) {
        self.script_cache.clear();
    }
}

#[cfg(target_os = "windows")]
impl Default for DirectWriteTextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect script runs in text using Unicode properties
#[cfg(target_os = "windows")]
fn detect_script_runs(text: &str) -> Vec<ScriptRun> {
    let mut runs = Vec::new();
    let mut current_script = "Latn".to_string(); // Default to Latin
    let mut current_start = 0;
    let mut current_length = 0;

    for (i, ch) in text.char_indices() {
        let script = detect_char_script(ch);

        if script != current_script && current_length > 0 {
            runs.push(ScriptRun {
                start: current_start,
                length: current_length,
                script: current_script.clone(),
            });
            current_start = i;
            current_length = 0;
        }

        current_script = script;
        current_length += ch.len_utf8();
    }

    // Push final run
    if current_length > 0 {
        runs.push(ScriptRun {
            start: current_start,
            length: current_length,
            script: current_script,
        });
    }

    runs
}

/// Detect the script of a single character
#[cfg(target_os = "windows")]
fn detect_char_script(ch: char) -> String {
    match ch {
        '\u{0000}'..='\u{007F}' => "Latn".to_string(), // Basic Latin
        '\u{0080}'..='\u{00FF}' => "Latn".to_string(), // Latin-1 Supplement
        '\u{0100}'..='\u{017F}' => "Latn".to_string(), // Latin Extended-A
        '\u{0400}'..='\u{04FF}' => "Cyrl".to_string(), // Cyrillic
        '\u{0370}'..='\u{03FF}' => "Grek".to_string(), // Greek
        '\u{0600}'..='\u{06FF}' => "Arab".to_string(), // Arabic
        '\u{0590}'..='\u{05FF}' => "Hebr".to_string(), // Hebrew
        '\u{4E00}'..='\u{9FFF}' => "Hans".to_string(), // CJK Unified Ideographs
        '\u{3040}'..='\u{309F}' => "Hira".to_string(), // Hiragana
        '\u{30A0}'..='\u{30FF}' => "Kana".to_string(), // Katakana
        '\u{AC00}'..='\u{D7AF}' => "Hang".to_string(), // Hangul Syllables
        '\u{0900}'..='\u{097F}' => "Deva".to_string(), // Devanagari
        '\u{0E00}'..='\u{0E7F}' => "Thai".to_string(), // Thai
        _ => "Zyyy".to_string(),                       // Common/Unknown
    }
}

/// Get default font families for Windows DirectWrite
#[cfg(target_os = "windows")]
pub fn get_directwrite_defaults() -> HashMap<FontCategory, Vec<String>> {
    let mut defaults = HashMap::new();

    defaults.insert(
        FontCategory::Serif,
        vec![
            "Times New Roman".to_string(),
            "Georgia".to_string(),
            "Cambria".to_string(),
            "Palatino Linotype".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::SansSerif,
        vec![
            "Segoe UI".to_string(),
            "Arial".to_string(),
            "Verdana".to_string(),
            "Tahoma".to_string(),
            "Calibri".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::Monospace,
        vec![
            "Consolas".to_string(),
            "Courier New".to_string(),
            "Cascadia Mono".to_string(),
            "Lucida Console".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::Cursive,
        vec![
            "Segoe Script".to_string(),
            "Comic Sans MS".to_string(),
            "Brush Script MT".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::Fantasy,
        vec![
            "Impact".to_string(),
            "Copperplate Gothic".to_string(),
            "Papyrus".to_string(),
        ],
    );

    defaults.insert(
        FontCategory::Emoji,
        vec!["Segoe UI Emoji".to_string(), "Segoe UI Symbol".to_string()],
    );

    defaults
}

// Non-Windows stub implementations
#[cfg(not(target_os = "windows"))]
pub struct DirectWriteFontCollection;

#[cfg(not(target_os = "windows"))]
impl DirectWriteFontCollection {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize(&mut self) -> Result<usize, String> {
        Err("DirectWrite is only available on Windows".to_string())
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
}

#[cfg(not(target_os = "windows"))]
impl Default for DirectWriteFontCollection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "windows"))]
pub struct DirectWriteTextAnalyzer;

#[cfg(not(target_os = "windows"))]
impl DirectWriteTextAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_script(&mut self, _text: &str) -> Vec<ScriptRun> {
        Vec::new()
    }

    pub fn clear_cache(&mut self) {}
}

#[cfg(not(target_os = "windows"))]
impl Default for DirectWriteTextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_directwrite_defaults() -> HashMap<FontCategory, Vec<String>> {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directwrite_font_collection_new() {
        let collection = DirectWriteFontCollection::new();
        assert_eq!(collection.font_count(), 0);
    }

    #[test]
    fn test_directwrite_text_analyzer_new() {
        let _analyzer = DirectWriteTextAnalyzer::new();
        // Should not panic
    }

    #[test]
    fn test_script_run_struct() {
        let run = ScriptRun {
            start: 0,
            length: 5,
            script: "Latn".to_string(),
        };
        assert_eq!(run.start, 0);
        assert_eq!(run.length, 5);
        assert_eq!(run.script, "Latn");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_detect_char_script_latin() {
        assert_eq!(detect_char_script('a'), "Latn");
        assert_eq!(detect_char_script('Z'), "Latn");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_detect_char_script_cyrillic() {
        assert_eq!(detect_char_script('\u{0410}'), "Cyrl"); // Cyrillic A
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_detect_char_script_cjk() {
        assert_eq!(detect_char_script('\u{4E2D}'), "Hans"); // Chinese character
    }

    #[test]
    fn test_get_directwrite_defaults() {
        let defaults = get_directwrite_defaults();
        #[cfg(target_os = "windows")]
        {
            assert!(defaults.contains_key(&FontCategory::Serif));
            assert!(defaults.contains_key(&FontCategory::SansSerif));
            assert!(defaults.contains_key(&FontCategory::Monospace));
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert!(defaults.is_empty());
        }
    }
}
