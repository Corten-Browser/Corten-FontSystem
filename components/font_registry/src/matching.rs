//! Advanced font matching with weight/style scoring
//!
//! This module implements CSS-compliant font matching algorithms with
//! sophisticated weight, style, and stretch scoring.

use crate::types::{FontDescriptor, FontFace, FontId, FontStretch, FontStyle, FontWeight};
use std::collections::HashMap;

/// Font match result with scoring information
#[derive(Debug, Clone)]
pub struct FontMatch {
    /// Matched font ID
    pub font_id: FontId,
    /// Overall match score (lower is better)
    pub score: i32,
    /// Family match score component
    pub family_score: i32,
    /// Weight match score component
    pub weight_score: i32,
    /// Style match score component
    pub style_score: i32,
    /// Stretch match score component
    pub stretch_score: i32,
}

impl FontMatch {
    /// Create a new font match
    pub fn new(font_id: FontId) -> Self {
        Self {
            font_id,
            score: 0,
            family_score: 0,
            weight_score: 0,
            style_score: 0,
            stretch_score: 0,
        }
    }

    /// Calculate total score from components
    pub fn calculate_total(&mut self) {
        self.score = self.family_score + self.weight_score + self.style_score + self.stretch_score;
    }
}

/// Advanced font matcher with configurable scoring
#[derive(Debug)]
pub struct FontMatcher {
    /// Weight for family match score
    pub family_weight: i32,
    /// Weight for font weight score
    pub weight_weight: i32,
    /// Weight for style score
    pub style_weight: i32,
    /// Weight for stretch score
    pub stretch_weight: i32,
    /// Generic family mappings
    generic_families: HashMap<String, Vec<String>>,
}

impl FontMatcher {
    /// Create a new font matcher with default settings
    pub fn new() -> Self {
        let mut matcher = Self {
            family_weight: 1,
            weight_weight: 100,
            style_weight: 1000,
            stretch_weight: 50,
            generic_families: HashMap::new(),
        };
        matcher.setup_generic_families();
        matcher
    }

    /// Set up default generic family mappings
    fn setup_generic_families(&mut self) {
        self.generic_families.insert(
            "serif".to_string(),
            vec![
                "Times New Roman".to_string(),
                "Georgia".to_string(),
                "DejaVu Serif".to_string(),
                "Liberation Serif".to_string(),
            ],
        );

        self.generic_families.insert(
            "sans-serif".to_string(),
            vec![
                "Arial".to_string(),
                "Helvetica".to_string(),
                "DejaVu Sans".to_string(),
                "Liberation Sans".to_string(),
            ],
        );

        self.generic_families.insert(
            "monospace".to_string(),
            vec![
                "Courier New".to_string(),
                "Consolas".to_string(),
                "DejaVu Sans Mono".to_string(),
                "Liberation Mono".to_string(),
            ],
        );

        self.generic_families.insert(
            "cursive".to_string(),
            vec![
                "Comic Sans MS".to_string(),
                "Apple Chancery".to_string(),
                "URW Chancery L".to_string(),
            ],
        );

        self.generic_families.insert(
            "fantasy".to_string(),
            vec![
                "Impact".to_string(),
                "Papyrus".to_string(),
                "Copperplate".to_string(),
            ],
        );

        self.generic_families.insert(
            "system-ui".to_string(),
            vec![
                "Segoe UI".to_string(),
                "San Francisco".to_string(),
                ".SF NS Text".to_string(),
                "Ubuntu".to_string(),
            ],
        );

        self.generic_families.insert(
            "ui-serif".to_string(),
            vec!["Georgia".to_string(), "Times New Roman".to_string()],
        );

        self.generic_families.insert(
            "ui-sans-serif".to_string(),
            vec![
                "Segoe UI".to_string(),
                "San Francisco".to_string(),
                "Arial".to_string(),
            ],
        );

        self.generic_families.insert(
            "ui-monospace".to_string(),
            vec![
                "SF Mono".to_string(),
                "Consolas".to_string(),
                "Menlo".to_string(),
            ],
        );

        self.generic_families.insert(
            "emoji".to_string(),
            vec![
                "Apple Color Emoji".to_string(),
                "Segoe UI Emoji".to_string(),
                "Noto Color Emoji".to_string(),
            ],
        );
    }

    /// Find best matching font from a collection
    ///
    /// # Arguments
    ///
    /// * `descriptor` - Font descriptor with matching criteria
    /// * `fonts` - Available fonts to match against
    ///
    /// # Returns
    ///
    /// Best matching font ID and match details, if any
    pub fn find_best_match(
        &self,
        descriptor: &FontDescriptor,
        fonts: &HashMap<FontId, FontFace>,
    ) -> Option<FontMatch> {
        let mut best_match: Option<FontMatch> = None;

        // Expand generic families
        let expanded_families = self.expand_families(&descriptor.family);

        for (font_id, font) in fonts {
            // Calculate family match score
            let family_score = self.calculate_family_score(&expanded_families, &font.family_name);

            // Skip fonts that don't match any family in the list
            if family_score == i32::MAX {
                continue;
            }

            let mut font_match = FontMatch::new(*font_id);
            font_match.family_score = family_score * self.family_weight;
            font_match.weight_score =
                self.calculate_weight_score(descriptor.weight, font.weight) * self.weight_weight;
            font_match.style_score =
                self.calculate_style_score(descriptor.style, font.style) * self.style_weight;
            font_match.stretch_score = self
                .calculate_stretch_score(descriptor.stretch, font.stretch)
                * self.stretch_weight;
            font_match.calculate_total();

            if let Some(ref best) = best_match {
                if font_match.score < best.score {
                    best_match = Some(font_match);
                }
            } else {
                best_match = Some(font_match);
            }
        }

        best_match
    }

    /// Expand family list with generic family mappings
    fn expand_families(&self, families: &[String]) -> Vec<String> {
        let mut expanded = Vec::new();

        for family in families {
            let lower = family.to_lowercase();
            if let Some(mapped) = self.generic_families.get(&lower) {
                expanded.extend(mapped.clone());
            } else {
                expanded.push(family.clone());
            }
        }

        expanded
    }

    /// Calculate family match score (lower is better)
    ///
    /// Returns position in family list (0 = first choice) or i32::MAX if no match
    fn calculate_family_score(&self, requested_families: &[String], font_family: &str) -> i32 {
        let font_lower = font_family.to_lowercase();

        for (index, family) in requested_families.iter().enumerate() {
            if family.to_lowercase() == font_lower {
                return index as i32;
            }
        }

        i32::MAX
    }

    /// Calculate weight match score using CSS font matching algorithm
    ///
    /// Reference: https://www.w3.org/TR/css-fonts-4/#font-matching-algorithm
    fn calculate_weight_score(&self, requested: FontWeight, available: FontWeight) -> i32 {
        let req_num = requested as i32;
        let avail_num = available as i32;

        // Direct match
        if req_num == avail_num {
            return 0;
        }

        // CSS weight matching algorithm
        // For weights < 400, prefer lighter weights, then heavier
        // For weights > 500, prefer heavier weights, then lighter
        // For weights 400-500, prefer 500, 400, then lighter, then heavier

        let diff = (req_num - avail_num).abs();

        // Apply CSS preference rules
        if req_num < 400 {
            if avail_num < req_num {
                diff // Lighter is preferred
            } else {
                diff + 100 // Heavier is penalized
            }
        } else if req_num > 500 {
            if avail_num > req_num {
                diff // Heavier is preferred
            } else {
                diff + 100 // Lighter is penalized
            }
        } else {
            // 400-500 range
            if avail_num == 500 || avail_num == 400 {
                diff
            } else if avail_num < req_num {
                diff + 50 // Lighter is slightly penalized
            } else {
                diff + 100 // Heavier is more penalized
            }
        }
    }

    /// Calculate style match score
    fn calculate_style_score(&self, requested: FontStyle, available: FontStyle) -> i32 {
        match (requested, available) {
            // Exact match
            (FontStyle::Normal, FontStyle::Normal) => 0,
            (FontStyle::Italic, FontStyle::Italic) => 0,
            (FontStyle::Oblique(r), FontStyle::Oblique(a)) => {
                ((r - a).abs() * 10.0) as i32 // Scale angle difference
            }

            // Italic and Oblique can substitute for each other
            (FontStyle::Italic, FontStyle::Oblique(_)) => 1,
            (FontStyle::Oblique(_), FontStyle::Italic) => 1,

            // Normal vs Italic/Oblique is a major mismatch
            (FontStyle::Normal, FontStyle::Italic) | (FontStyle::Normal, FontStyle::Oblique(_)) => {
                10
            }
            (FontStyle::Italic, FontStyle::Normal) | (FontStyle::Oblique(_), FontStyle::Normal) => {
                10
            }
        }
    }

    /// Calculate stretch match score
    fn calculate_stretch_score(&self, requested: FontStretch, available: FontStretch) -> i32 {
        let req_num = requested as i32;
        let avail_num = available as i32;

        // Calculate difference
        let diff = (req_num - avail_num).abs();

        // CSS stretch matching: prefer adjacent values
        // UltraCondensed(50) -> UltraExpanded(200)
        // Normal is 100

        if diff == 0 {
            0
        } else if diff <= 25 {
            diff // Adjacent values
        } else {
            diff * 2 // Non-adjacent penalized
        }
    }

    /// Add a custom generic family mapping
    pub fn add_generic_family(&mut self, generic: &str, families: Vec<String>) {
        self.generic_families
            .insert(generic.to_lowercase(), families);
    }

    /// Get fonts for a generic family
    pub fn get_generic_fonts(&self, generic: &str) -> Option<&Vec<String>> {
        self.generic_families.get(&generic.to_lowercase())
    }
}

impl Default for FontMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// System font substitution configuration
#[derive(Debug, Clone)]
pub struct SubstitutionConfig {
    /// Enable automatic substitution
    pub enabled: bool,
    /// Preferred substitution families by category
    pub preferences: HashMap<String, Vec<String>>,
}

impl Default for SubstitutionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preferences: HashMap::new(),
        }
    }
}

/// System font substitution manager
#[derive(Debug)]
pub struct FontSubstitution {
    /// Substitution configuration
    config: SubstitutionConfig,
    /// Direct substitution mappings
    direct_mappings: HashMap<String, String>,
}

impl FontSubstitution {
    /// Create a new font substitution manager
    pub fn new() -> Self {
        let mut manager = Self {
            config: SubstitutionConfig::default(),
            direct_mappings: HashMap::new(),
        };
        manager.setup_default_mappings();
        manager
    }

    /// Set up default font substitution mappings
    fn setup_default_mappings(&mut self) {
        // Common Windows fonts to cross-platform alternatives
        self.direct_mappings
            .insert("Arial".to_lowercase(), "Liberation Sans".to_string());
        self.direct_mappings.insert(
            "Times New Roman".to_lowercase(),
            "Liberation Serif".to_string(),
        );
        self.direct_mappings
            .insert("Courier New".to_lowercase(), "Liberation Mono".to_string());
        self.direct_mappings
            .insert("Verdana".to_lowercase(), "DejaVu Sans".to_string());
        self.direct_mappings
            .insert("Georgia".to_lowercase(), "DejaVu Serif".to_string());

        // macOS fonts to alternatives
        self.direct_mappings
            .insert("Helvetica".to_lowercase(), "Arial".to_string());
        self.direct_mappings
            .insert("Helvetica Neue".to_lowercase(), "Arial".to_string());

        // Metric-compatible fonts
        self.direct_mappings
            .insert("Calibri".to_lowercase(), "Carlito".to_string());
        self.direct_mappings
            .insert("Cambria".to_lowercase(), "Caladea".to_string());
    }

    /// Get substitution for a font family
    ///
    /// # Arguments
    ///
    /// * `family` - Font family to substitute
    ///
    /// # Returns
    ///
    /// Substitution family name, if available
    pub fn get_substitution(&self, family: &str) -> Option<&String> {
        if !self.config.enabled {
            return None;
        }
        self.direct_mappings.get(&family.to_lowercase())
    }

    /// Add a direct substitution mapping
    pub fn add_mapping(&mut self, from: &str, to: &str) {
        self.direct_mappings
            .insert(from.to_lowercase(), to.to_string());
    }

    /// Remove a substitution mapping
    pub fn remove_mapping(&mut self, from: &str) -> Option<String> {
        self.direct_mappings.remove(&from.to_lowercase())
    }

    /// Enable or disable substitution
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Check if substitution is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get all direct mappings
    pub fn get_all_mappings(&self) -> &HashMap<String, String> {
        &self.direct_mappings
    }

    /// Expand a family list with substitutions
    pub fn expand_with_substitutions(&self, families: &[String]) -> Vec<String> {
        let mut expanded = Vec::new();

        for family in families {
            expanded.push(family.clone());
            if let Some(sub) = self.get_substitution(family) {
                expanded.push(sub.clone());
            }
        }

        expanded
    }
}

impl Default for FontSubstitution {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_match_new() {
        let match_result = FontMatch::new(1);
        assert_eq!(match_result.font_id, 1);
        assert_eq!(match_result.score, 0);
    }

    #[test]
    fn test_font_match_calculate_total() {
        let mut match_result = FontMatch::new(1);
        match_result.family_score = 10;
        match_result.weight_score = 20;
        match_result.style_score = 30;
        match_result.stretch_score = 40;
        match_result.calculate_total();
        assert_eq!(match_result.score, 100);
    }

    #[test]
    fn test_font_matcher_new() {
        let matcher = FontMatcher::new();
        assert!(matcher.get_generic_fonts("serif").is_some());
        assert!(matcher.get_generic_fonts("sans-serif").is_some());
    }

    #[test]
    fn test_weight_score_exact_match() {
        let matcher = FontMatcher::new();
        let score = matcher.calculate_weight_score(FontWeight::Regular, FontWeight::Regular);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_weight_score_difference() {
        let matcher = FontMatcher::new();
        let score = matcher.calculate_weight_score(FontWeight::Bold, FontWeight::Regular);
        assert!(score > 0);
    }

    #[test]
    fn test_style_score_exact_match() {
        let matcher = FontMatcher::new();
        let score = matcher.calculate_style_score(FontStyle::Normal, FontStyle::Normal);
        assert_eq!(score, 0);

        let score = matcher.calculate_style_score(FontStyle::Italic, FontStyle::Italic);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_style_score_italic_oblique() {
        let matcher = FontMatcher::new();
        let score = matcher.calculate_style_score(FontStyle::Italic, FontStyle::Oblique(12.0));
        assert_eq!(score, 1); // They can substitute for each other
    }

    #[test]
    fn test_font_substitution_new() {
        let sub = FontSubstitution::new();
        assert!(sub.is_enabled());
        assert!(sub.get_substitution("Arial").is_some());
    }

    #[test]
    fn test_font_substitution_get() {
        let sub = FontSubstitution::new();
        assert_eq!(
            sub.get_substitution("Arial"),
            Some(&"Liberation Sans".to_string())
        );
    }

    #[test]
    fn test_font_substitution_disabled() {
        let mut sub = FontSubstitution::new();
        sub.set_enabled(false);
        assert!(sub.get_substitution("Arial").is_none());
    }

    #[test]
    fn test_expand_with_substitutions() {
        let sub = FontSubstitution::new();
        let families = vec!["Arial".to_string()];
        let expanded = sub.expand_with_substitutions(&families);
        assert!(expanded.len() >= 2);
        assert!(expanded.contains(&"Liberation Sans".to_string()));
    }
}
