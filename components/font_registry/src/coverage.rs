//! Coverage-based font fallback
//!
//! This module provides character coverage analysis for fonts and implements
//! coverage-based font fallback selection.

use crate::types::{FontFace, FontId};
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;

/// Unicode block ranges for common scripts
pub mod unicode_blocks {
    use std::ops::RangeInclusive;

    /// Basic Latin (ASCII)
    pub const BASIC_LATIN: RangeInclusive<u32> = 0x0000..=0x007F;
    /// Latin-1 Supplement
    pub const LATIN_1_SUPPLEMENT: RangeInclusive<u32> = 0x0080..=0x00FF;
    /// Latin Extended-A
    pub const LATIN_EXTENDED_A: RangeInclusive<u32> = 0x0100..=0x017F;
    /// Latin Extended-B
    pub const LATIN_EXTENDED_B: RangeInclusive<u32> = 0x0180..=0x024F;
    /// Greek and Coptic
    pub const GREEK: RangeInclusive<u32> = 0x0370..=0x03FF;
    /// Cyrillic
    pub const CYRILLIC: RangeInclusive<u32> = 0x0400..=0x04FF;
    /// Arabic
    pub const ARABIC: RangeInclusive<u32> = 0x0600..=0x06FF;
    /// Hebrew
    pub const HEBREW: RangeInclusive<u32> = 0x0590..=0x05FF;
    /// Devanagari
    pub const DEVANAGARI: RangeInclusive<u32> = 0x0900..=0x097F;
    /// Thai
    pub const THAI: RangeInclusive<u32> = 0x0E00..=0x0E7F;
    /// CJK Unified Ideographs
    pub const CJK_UNIFIED: RangeInclusive<u32> = 0x4E00..=0x9FFF;
    /// Hiragana
    pub const HIRAGANA: RangeInclusive<u32> = 0x3040..=0x309F;
    /// Katakana
    pub const KATAKANA: RangeInclusive<u32> = 0x30A0..=0x30FF;
    /// Hangul Syllables
    pub const HANGUL: RangeInclusive<u32> = 0xAC00..=0xD7AF;
    /// Emoji
    pub const EMOJI: RangeInclusive<u32> = 0x1F300..=0x1F9FF;
    /// Mathematical Symbols
    pub const MATH_SYMBOLS: RangeInclusive<u32> = 0x2200..=0x22FF;
}

/// Character coverage information for a font
#[derive(Debug, Clone)]
pub struct FontCoverage {
    /// Font ID this coverage belongs to
    pub font_id: FontId,
    /// Set of supported codepoints
    supported_codepoints: HashSet<u32>,
    /// Coverage statistics
    pub stats: CoverageStats,
}

/// Statistics about font coverage
#[derive(Debug, Clone, Default)]
pub struct CoverageStats {
    /// Total number of supported glyphs
    pub total_glyphs: usize,
    /// Basic Latin coverage percentage (0-100)
    pub latin_coverage: f32,
    /// Extended Latin coverage percentage
    pub extended_latin_coverage: f32,
    /// Cyrillic coverage percentage
    pub cyrillic_coverage: f32,
    /// Greek coverage percentage
    pub greek_coverage: f32,
    /// CJK coverage percentage
    pub cjk_coverage: f32,
    /// Arabic coverage percentage
    pub arabic_coverage: f32,
    /// Hebrew coverage percentage
    pub hebrew_coverage: f32,
    /// Emoji coverage percentage
    pub emoji_coverage: f32,
}

impl FontCoverage {
    /// Create new font coverage from a font face
    ///
    /// # Arguments
    ///
    /// * `font_id` - ID of the font
    /// * `font_data` - Raw font data bytes
    ///
    /// # Returns
    ///
    /// Font coverage information or None if parsing fails
    pub fn from_font_data(font_id: FontId, font_data: &[u8]) -> Option<Self> {
        let face = ttf_parser::Face::parse(font_data, 0).ok()?;

        let mut supported_codepoints = HashSet::new();

        // Build codepoint set from cmap table
        if let Some(cmap) = face.tables().cmap {
            for subtable in cmap.subtables {
                subtable.codepoints(|cp| {
                    supported_codepoints.insert(cp);
                });
            }
        }

        let stats = calculate_coverage_stats(&supported_codepoints);

        Some(Self {
            font_id,
            supported_codepoints,
            stats,
        })
    }

    /// Check if font supports a specific codepoint
    pub fn supports_codepoint(&self, codepoint: u32) -> bool {
        self.supported_codepoints.contains(&codepoint)
    }

    /// Check if font supports a character
    pub fn supports_char(&self, ch: char) -> bool {
        self.supports_codepoint(ch as u32)
    }

    /// Check if font supports all characters in a string
    pub fn supports_string(&self, text: &str) -> bool {
        text.chars().all(|ch| self.supports_char(ch))
    }

    /// Calculate coverage for a string (percentage of characters supported)
    pub fn string_coverage(&self, text: &str) -> f32 {
        if text.is_empty() {
            return 100.0;
        }

        let total = text.chars().count();
        let supported = text.chars().filter(|ch| self.supports_char(*ch)).count();

        (supported as f32 / total as f32) * 100.0
    }

    /// Get unsupported characters from a string
    pub fn unsupported_chars(&self, text: &str) -> Vec<char> {
        text.chars().filter(|ch| !self.supports_char(*ch)).collect()
    }

    /// Check if font supports a Unicode block
    pub fn supports_block(&self, block: RangeInclusive<u32>) -> f32 {
        calculate_block_coverage(&self.supported_codepoints, block)
    }

    /// Get total glyph count
    pub fn glyph_count(&self) -> usize {
        self.supported_codepoints.len()
    }
}

/// Calculate coverage statistics for a set of codepoints
fn calculate_coverage_stats(codepoints: &HashSet<u32>) -> CoverageStats {
    CoverageStats {
        total_glyphs: codepoints.len(),
        latin_coverage: calculate_block_coverage(codepoints, unicode_blocks::BASIC_LATIN),
        extended_latin_coverage: calculate_block_coverage(
            codepoints,
            unicode_blocks::LATIN_EXTENDED_A,
        ),
        cyrillic_coverage: calculate_block_coverage(codepoints, unicode_blocks::CYRILLIC),
        greek_coverage: calculate_block_coverage(codepoints, unicode_blocks::GREEK),
        cjk_coverage: calculate_block_coverage(codepoints, unicode_blocks::CJK_UNIFIED),
        arabic_coverage: calculate_block_coverage(codepoints, unicode_blocks::ARABIC),
        hebrew_coverage: calculate_block_coverage(codepoints, unicode_blocks::HEBREW),
        emoji_coverage: calculate_block_coverage(codepoints, unicode_blocks::EMOJI),
    }
}

/// Calculate coverage percentage for a Unicode block
fn calculate_block_coverage(codepoints: &HashSet<u32>, block: RangeInclusive<u32>) -> f32 {
    let block_size = (block.end() - block.start() + 1) as f32;
    let covered = block.filter(|cp| codepoints.contains(cp)).count() as f32;
    (covered / block_size) * 100.0
}

/// Font fallback manager using coverage analysis
#[derive(Debug)]
pub struct CoverageFallbackManager {
    /// Font coverage cache
    coverage_cache: HashMap<FontId, FontCoverage>,
    /// Script to font mapping for fast lookup
    script_fonts: HashMap<String, Vec<FontId>>,
}

impl CoverageFallbackManager {
    /// Create a new coverage fallback manager
    pub fn new() -> Self {
        Self {
            coverage_cache: HashMap::new(),
            script_fonts: HashMap::new(),
        }
    }

    /// Add font coverage to the manager
    ///
    /// # Arguments
    ///
    /// * `coverage` - Font coverage to add
    pub fn add_coverage(&mut self, coverage: FontCoverage) {
        let font_id = coverage.font_id;

        // Categorize font by script support
        if coverage.stats.latin_coverage > 80.0 {
            self.script_fonts
                .entry("Latin".to_string())
                .or_default()
                .push(font_id);
        }
        if coverage.stats.cyrillic_coverage > 80.0 {
            self.script_fonts
                .entry("Cyrillic".to_string())
                .or_default()
                .push(font_id);
        }
        if coverage.stats.greek_coverage > 80.0 {
            self.script_fonts
                .entry("Greek".to_string())
                .or_default()
                .push(font_id);
        }
        if coverage.stats.cjk_coverage > 10.0 {
            // CJK fonts don't typically cover 80%
            self.script_fonts
                .entry("CJK".to_string())
                .or_default()
                .push(font_id);
        }
        if coverage.stats.arabic_coverage > 80.0 {
            self.script_fonts
                .entry("Arabic".to_string())
                .or_default()
                .push(font_id);
        }
        if coverage.stats.hebrew_coverage > 80.0 {
            self.script_fonts
                .entry("Hebrew".to_string())
                .or_default()
                .push(font_id);
        }
        if coverage.stats.emoji_coverage > 50.0 {
            self.script_fonts
                .entry("Emoji".to_string())
                .or_default()
                .push(font_id);
        }

        self.coverage_cache.insert(font_id, coverage);
    }

    /// Find fallback font for a specific character
    ///
    /// # Arguments
    ///
    /// * `ch` - Character needing a font
    /// * `exclude` - Font IDs to exclude from search
    ///
    /// # Returns
    ///
    /// Font ID that supports the character, if any
    pub fn find_fallback_for_char(&self, ch: char, exclude: &[FontId]) -> Option<FontId> {
        let codepoint = ch as u32;

        // First, try fonts for the character's script
        let script = detect_script(ch);
        if let Some(script_fonts) = self.script_fonts.get(&script) {
            for &font_id in script_fonts {
                if exclude.contains(&font_id) {
                    continue;
                }
                if let Some(coverage) = self.coverage_cache.get(&font_id) {
                    if coverage.supports_codepoint(codepoint) {
                        return Some(font_id);
                    }
                }
            }
        }

        // Fall back to searching all fonts
        for (font_id, coverage) in &self.coverage_cache {
            if exclude.contains(font_id) {
                continue;
            }
            if coverage.supports_codepoint(codepoint) {
                return Some(*font_id);
            }
        }

        None
    }

    /// Find best fallback font for a string
    ///
    /// # Arguments
    ///
    /// * `text` - Text needing a font
    /// * `exclude` - Font IDs to exclude from search
    ///
    /// # Returns
    ///
    /// Font ID with best coverage for the text
    pub fn find_best_fallback(&self, text: &str, exclude: &[FontId]) -> Option<FontId> {
        let mut best_font: Option<(FontId, f32)> = None;

        for (font_id, coverage) in &self.coverage_cache {
            if exclude.contains(font_id) {
                continue;
            }

            let score = coverage.string_coverage(text);
            if score > 0.0 {
                if let Some((_, best_score)) = best_font {
                    if score > best_score {
                        best_font = Some((*font_id, score));
                    }
                } else {
                    best_font = Some((*font_id, score));
                }
            }
        }

        best_font.map(|(id, _)| id)
    }

    /// Get coverage for a font
    pub fn get_coverage(&self, font_id: FontId) -> Option<&FontCoverage> {
        self.coverage_cache.get(&font_id)
    }

    /// Get fonts for a specific script
    pub fn fonts_for_script(&self, script: &str) -> &[FontId] {
        self.script_fonts
            .get(script)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Clear the coverage cache
    pub fn clear(&mut self) {
        self.coverage_cache.clear();
        self.script_fonts.clear();
    }
}

impl Default for CoverageFallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect the script of a character
fn detect_script(ch: char) -> String {
    let cp = ch as u32;

    if unicode_blocks::BASIC_LATIN.contains(&cp)
        || unicode_blocks::LATIN_1_SUPPLEMENT.contains(&cp)
        || unicode_blocks::LATIN_EXTENDED_A.contains(&cp)
        || unicode_blocks::LATIN_EXTENDED_B.contains(&cp)
    {
        "Latin".to_string()
    } else if unicode_blocks::CYRILLIC.contains(&cp) {
        "Cyrillic".to_string()
    } else if unicode_blocks::GREEK.contains(&cp) {
        "Greek".to_string()
    } else if unicode_blocks::CJK_UNIFIED.contains(&cp)
        || unicode_blocks::HIRAGANA.contains(&cp)
        || unicode_blocks::KATAKANA.contains(&cp)
    {
        "CJK".to_string()
    } else if unicode_blocks::HANGUL.contains(&cp) {
        "Korean".to_string()
    } else if unicode_blocks::ARABIC.contains(&cp) {
        "Arabic".to_string()
    } else if unicode_blocks::HEBREW.contains(&cp) {
        "Hebrew".to_string()
    } else if unicode_blocks::DEVANAGARI.contains(&cp) {
        "Devanagari".to_string()
    } else if unicode_blocks::THAI.contains(&cp) {
        "Thai".to_string()
    } else if unicode_blocks::EMOJI.contains(&cp) {
        "Emoji".to_string()
    } else {
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_stats_default() {
        let stats = CoverageStats::default();
        assert_eq!(stats.total_glyphs, 0);
        assert_eq!(stats.latin_coverage, 0.0);
    }

    #[test]
    fn test_detect_script_latin() {
        assert_eq!(detect_script('a'), "Latin");
        assert_eq!(detect_script('Z'), "Latin");
        assert_eq!(detect_script('0'), "Latin");
    }

    #[test]
    fn test_detect_script_cyrillic() {
        assert_eq!(detect_script('\u{0410}'), "Cyrillic"); // Cyrillic A
    }

    #[test]
    fn test_detect_script_cjk() {
        assert_eq!(detect_script('\u{4E2D}'), "CJK"); // Chinese character for "middle"
    }

    #[test]
    fn test_fallback_manager_new() {
        let manager = CoverageFallbackManager::new();
        assert!(manager.coverage_cache.is_empty());
        assert!(manager.script_fonts.is_empty());
    }

    #[test]
    fn test_fonts_for_script_empty() {
        let manager = CoverageFallbackManager::new();
        assert!(manager.fonts_for_script("Latin").is_empty());
    }

    #[test]
    fn test_calculate_block_coverage_empty() {
        let codepoints = HashSet::new();
        let coverage = calculate_block_coverage(&codepoints, unicode_blocks::BASIC_LATIN);
        assert_eq!(coverage, 0.0);
    }

    #[test]
    fn test_calculate_block_coverage_full() {
        let mut codepoints = HashSet::new();
        for cp in unicode_blocks::BASIC_LATIN {
            codepoints.insert(cp);
        }
        let coverage = calculate_block_coverage(&codepoints, unicode_blocks::BASIC_LATIN);
        assert_eq!(coverage, 100.0);
    }
}
