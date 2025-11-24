//! OpenType feature selection and management
//!
//! This module provides APIs for selecting and applying OpenType features
//! such as ligatures ('liga'), kerning ('kern'), stylistic alternates ('salt'), etc.

use crate::gpos::GposTable;
use crate::gsub::{GsubTable, LigatureSubst, SingleSubst, SubtableData as GsubSubtable};
use crate::types::{GlyphId, Tag};
use std::collections::{HashMap, HashSet};

/// Common OpenType feature tags
pub mod tags {
    use crate::types::Tag;

    // Required features
    /// Required ligatures
    pub const RLIG: Tag = Tag(*b"rlig");

    // Common GSUB features
    /// Standard ligatures
    pub const LIGA: Tag = Tag(*b"liga");
    /// Contextual ligatures
    pub const CLIG: Tag = Tag(*b"clig");
    /// Discretionary ligatures
    pub const DLIG: Tag = Tag(*b"dlig");
    /// Historical ligatures
    pub const HLIG: Tag = Tag(*b"hlig");
    /// Contextual alternates
    pub const CALT: Tag = Tag(*b"calt");
    /// Stylistic alternates
    pub const SALT: Tag = Tag(*b"salt");
    /// Swash
    pub const SWSH: Tag = Tag(*b"swsh");
    /// Historical forms
    pub const HIST: Tag = Tag(*b"hist");
    /// Small capitals
    pub const SMCP: Tag = Tag(*b"smcp");
    /// Small caps from capitals
    pub const C2SC: Tag = Tag(*b"c2sc");
    /// Petite capitals
    pub const PCAP: Tag = Tag(*b"pcap");
    /// Titling
    pub const TITL: Tag = Tag(*b"titl");
    /// Subscript
    pub const SUBS: Tag = Tag(*b"subs");
    /// Superscript
    pub const SUPS: Tag = Tag(*b"sups");
    /// Ordinals
    pub const ORDN: Tag = Tag(*b"ordn");
    /// Fractions
    pub const FRAC: Tag = Tag(*b"frac");
    /// Numerator
    pub const NUMR: Tag = Tag(*b"numr");
    /// Denominator
    pub const DNOM: Tag = Tag(*b"dnom");
    /// Oldstyle figures
    pub const ONUM: Tag = Tag(*b"onum");
    /// Lining figures
    pub const LNUM: Tag = Tag(*b"lnum");
    /// Tabular figures
    pub const TNUM: Tag = Tag(*b"tnum");
    /// Proportional figures
    pub const PNUM: Tag = Tag(*b"pnum");
    /// Slashed zero
    pub const ZERO: Tag = Tag(*b"zero");

    // Common GPOS features
    /// Kerning
    pub const KERN: Tag = Tag(*b"kern");
    /// Mark positioning
    pub const MARK: Tag = Tag(*b"mark");
    /// Mark to mark positioning
    pub const MKMK: Tag = Tag(*b"mkmk");

    // Script-specific features
    /// Initial forms (Arabic)
    pub const INIT: Tag = Tag(*b"init");
    /// Medial forms (Arabic)
    pub const MEDI: Tag = Tag(*b"medi");
    /// Final forms (Arabic)
    pub const FINA: Tag = Tag(*b"fina");
    /// Isolated forms (Arabic)
    pub const ISOL: Tag = Tag(*b"isol");
    /// Vertical alternates
    pub const VERT: Tag = Tag(*b"vert");
    /// Vertical rotation
    pub const VRT2: Tag = Tag(*b"vrt2");
}

/// Feature selection configuration
#[derive(Debug, Clone)]
pub struct FeatureSelection {
    /// Enabled features with their settings (0 = off, 1+ = on with variant)
    enabled: HashMap<Tag, u32>,
    /// Script tag for feature lookup
    script: Option<Tag>,
    /// Language tag for feature lookup
    language: Option<Tag>,
}

impl Default for FeatureSelection {
    fn default() -> Self {
        let mut enabled = HashMap::new();
        // Default enabled features
        enabled.insert(tags::LIGA, 1);
        enabled.insert(tags::CLIG, 1);
        enabled.insert(tags::CALT, 1);
        enabled.insert(tags::KERN, 1);
        enabled.insert(tags::MARK, 1);
        enabled.insert(tags::MKMK, 1);
        enabled.insert(tags::RLIG, 1);

        Self {
            enabled,
            script: None,
            language: None,
        }
    }
}

impl FeatureSelection {
    /// Create a new feature selection with no features enabled
    pub fn new() -> Self {
        Self {
            enabled: HashMap::new(),
            script: None,
            language: None,
        }
    }

    /// Create feature selection with default features
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// Enable a feature
    pub fn enable(&mut self, tag: Tag) -> &mut Self {
        self.enabled.insert(tag, 1);
        self
    }

    /// Enable a feature with a specific variant
    pub fn enable_with_value(&mut self, tag: Tag, value: u32) -> &mut Self {
        self.enabled.insert(tag, value);
        self
    }

    /// Disable a feature
    pub fn disable(&mut self, tag: Tag) -> &mut Self {
        self.enabled.remove(&tag);
        self
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, tag: Tag) -> bool {
        self.enabled.contains_key(&tag)
    }

    /// Get feature value (0 if disabled)
    pub fn get_value(&self, tag: Tag) -> u32 {
        self.enabled.get(&tag).copied().unwrap_or(0)
    }

    /// Set script for feature lookup
    pub fn set_script(&mut self, script: Tag) -> &mut Self {
        self.script = Some(script);
        self
    }

    /// Set language for feature lookup
    pub fn set_language(&mut self, language: Tag) -> &mut Self {
        self.language = Some(language);
        self
    }

    /// Get all enabled feature tags
    pub fn enabled_features(&self) -> Vec<Tag> {
        self.enabled.keys().copied().collect()
    }

    /// Get script tag
    pub fn script(&self) -> Option<Tag> {
        self.script
    }

    /// Get language tag
    pub fn language(&self) -> Option<Tag> {
        self.language
    }
}

/// Feature applicator for applying OpenType features to glyph sequences
#[derive(Debug)]
pub struct FeatureApplicator<'a> {
    gsub: Option<&'a GsubTable>,
    gpos: Option<&'a GposTable>,
    selection: &'a FeatureSelection,
}

impl<'a> FeatureApplicator<'a> {
    /// Create a new feature applicator
    pub fn new(
        gsub: Option<&'a GsubTable>,
        gpos: Option<&'a GposTable>,
        selection: &'a FeatureSelection,
    ) -> Self {
        Self {
            gsub,
            gpos,
            selection,
        }
    }

    /// Get all lookup indices for enabled features in order
    fn get_gsub_lookup_indices(&self) -> Vec<u16> {
        let gsub = match self.gsub {
            Some(g) => g,
            None => return Vec::new(),
        };

        let mut indices = Vec::new();

        // Get feature indices for script/language
        let script = self
            .selection
            .script
            .unwrap_or_else(|| Tag::new("DFLT").unwrap());
        let feature_indices = gsub.get_feature_indices(script, self.selection.language);

        // For each feature index, if the feature is enabled, add its lookups
        for feature_idx in feature_indices {
            if let Some(feature) = gsub.features.features.get(feature_idx as usize) {
                if self.selection.is_enabled(feature.tag) {
                    indices.extend(&feature.lookup_indices);
                }
            }
        }

        // Also add lookups for explicitly enabled features
        for tag in self.selection.enabled_features() {
            let lookup_indices = gsub.get_lookup_indices(tag);
            for idx in lookup_indices {
                if !indices.contains(&idx) {
                    indices.push(idx);
                }
            }
        }

        indices.sort();
        indices.dedup();
        indices
    }

    /// Apply GSUB substitutions to a glyph sequence
    pub fn apply_gsub(&self, glyphs: &mut Vec<GlyphId>) {
        let gsub = match self.gsub {
            Some(g) => g,
            None => return,
        };

        let lookup_indices = self.get_gsub_lookup_indices();

        for lookup_idx in lookup_indices {
            if let Some(lookup) = gsub.lookups.lookups.get(lookup_idx as usize) {
                self.apply_gsub_lookup(glyphs, lookup);
            }
        }
    }

    fn apply_gsub_lookup(&self, glyphs: &mut Vec<GlyphId>, lookup: &crate::gsub::Lookup) {
        for subtable in &lookup.subtables {
            match subtable {
                GsubSubtable::Single(single) => {
                    self.apply_single_subst(glyphs, single);
                }
                GsubSubtable::Ligature(lig) => {
                    self.apply_ligature_subst(glyphs, lig);
                }
                _ => {}
            }
        }
    }

    fn apply_single_subst(&self, glyphs: &mut Vec<GlyphId>, single: &SingleSubst) {
        for glyph in glyphs.iter_mut() {
            if let Some(replacement) = single.substitute(*glyph) {
                *glyph = replacement;
            }
        }
    }

    fn apply_ligature_subst(&self, glyphs: &mut Vec<GlyphId>, lig: &LigatureSubst) {
        let mut i = 0;
        while i < glyphs.len() {
            if let Some((lig_glyph, count)) = lig.match_ligature(&glyphs[i..]) {
                // Replace the sequence with the ligature
                glyphs[i] = lig_glyph;
                // Remove the components (all but the first)
                for _ in 1..count {
                    if i + 1 < glyphs.len() {
                        glyphs.remove(i + 1);
                    }
                }
            }
            i += 1;
        }
    }

    /// Get kerning adjustments for a glyph sequence
    pub fn get_kerning(&self, glyphs: &[GlyphId]) -> Vec<i16> {
        let mut adjustments = vec![0i16; glyphs.len()];

        let gpos = match self.gpos {
            Some(g) => g,
            None => return adjustments,
        };

        if !self.selection.is_enabled(tags::KERN) {
            return adjustments;
        }

        for i in 0..glyphs.len().saturating_sub(1) {
            if let Some(kern) = gpos.get_kerning(glyphs[i], glyphs[i + 1]) {
                adjustments[i] = kern;
            }
        }

        adjustments
    }
}

/// Query available features in a font
#[derive(Debug)]
pub struct FeatureQuery<'a> {
    gsub: Option<&'a GsubTable>,
    gpos: Option<&'a GposTable>,
}

impl<'a> FeatureQuery<'a> {
    /// Create a new feature query
    pub fn new(gsub: Option<&'a GsubTable>, gpos: Option<&'a GposTable>) -> Self {
        Self { gsub, gpos }
    }

    /// Get all available GSUB features
    pub fn gsub_features(&self) -> Vec<Tag> {
        self.gsub
            .map(|g| g.supported_features())
            .unwrap_or_default()
    }

    /// Get all available GPOS features
    pub fn gpos_features(&self) -> Vec<Tag> {
        self.gpos
            .map(|g| g.supported_features())
            .unwrap_or_default()
    }

    /// Get all available features (GSUB + GPOS)
    pub fn all_features(&self) -> Vec<Tag> {
        let mut features: HashSet<Tag> = HashSet::new();
        features.extend(self.gsub_features());
        features.extend(self.gpos_features());
        features.into_iter().collect()
    }

    /// Check if a specific feature is available
    pub fn has_feature(&self, tag: Tag) -> bool {
        self.gsub.map(|g| g.has_feature(tag)).unwrap_or(false)
            || self.gpos.map(|g| g.has_feature(tag)).unwrap_or(false)
    }

    /// Check if ligatures are available
    pub fn has_ligatures(&self) -> bool {
        self.has_feature(tags::LIGA) || self.has_feature(tags::CLIG) || self.has_feature(tags::DLIG)
    }

    /// Check if kerning is available
    pub fn has_kerning(&self) -> bool {
        self.has_feature(tags::KERN)
    }

    /// Get available scripts
    pub fn available_scripts(&self) -> Vec<Tag> {
        let mut scripts = HashSet::new();

        if let Some(gsub) = self.gsub {
            scripts.extend(gsub.scripts.scripts.keys().copied());
        }
        if let Some(gpos) = self.gpos {
            scripts.extend(gpos.scripts.scripts.keys().copied());
        }

        scripts.into_iter().collect()
    }

    /// Get available languages for a script
    pub fn available_languages(&self, script: Tag) -> Vec<Tag> {
        let mut languages = HashSet::new();

        if let Some(gsub) = self.gsub {
            if let Some(script_record) = gsub.scripts.scripts.get(&script) {
                languages.extend(script_record.lang_sys.keys().copied());
            }
        }
        if let Some(gpos) = self.gpos {
            if let Some(script_record) = gpos.scripts.scripts.get(&script) {
                languages.extend(script_record.lang_sys.keys().copied());
            }
        }

        languages.into_iter().collect()
    }
}

/// Convenience function to create a default feature selection
pub fn default_features() -> FeatureSelection {
    FeatureSelection::default()
}

/// Convenience function to create feature selection with only kerning
pub fn kerning_only() -> FeatureSelection {
    let mut selection = FeatureSelection::new();
    selection.enable(tags::KERN);
    selection
}

/// Convenience function to create feature selection with kerning and ligatures
pub fn kerning_and_ligatures() -> FeatureSelection {
    let mut selection = FeatureSelection::new();
    selection.enable(tags::KERN);
    selection.enable(tags::LIGA);
    selection.enable(tags::CLIG);
    selection
}

/// Convenience function to create feature selection with all common features
pub fn all_common_features() -> FeatureSelection {
    let mut selection = FeatureSelection::default();
    selection.enable(tags::DLIG);
    selection.enable(tags::SMCP);
    selection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_selection_default() {
        let selection = FeatureSelection::default();
        assert!(selection.is_enabled(tags::LIGA));
        assert!(selection.is_enabled(tags::KERN));
        assert!(!selection.is_enabled(tags::SMCP));
    }

    #[test]
    fn test_feature_selection_enable_disable() {
        let mut selection = FeatureSelection::new();
        assert!(!selection.is_enabled(tags::LIGA));

        selection.enable(tags::LIGA);
        assert!(selection.is_enabled(tags::LIGA));
        assert_eq!(selection.get_value(tags::LIGA), 1);

        selection.enable_with_value(tags::SALT, 3);
        assert!(selection.is_enabled(tags::SALT));
        assert_eq!(selection.get_value(tags::SALT), 3);

        selection.disable(tags::LIGA);
        assert!(!selection.is_enabled(tags::LIGA));
        assert_eq!(selection.get_value(tags::LIGA), 0);
    }

    #[test]
    fn test_feature_selection_script_language() {
        let mut selection = FeatureSelection::new();

        selection.set_script(Tag::new("latn").unwrap());
        assert_eq!(selection.script(), Some(Tag::new("latn").unwrap()));

        selection.set_language(Tag::new("DEU ").unwrap());
        assert_eq!(selection.language(), Some(Tag::new("DEU ").unwrap()));
    }

    #[test]
    fn test_convenience_functions() {
        let kern = kerning_only();
        assert!(kern.is_enabled(tags::KERN));
        assert!(!kern.is_enabled(tags::LIGA));

        let kern_lig = kerning_and_ligatures();
        assert!(kern_lig.is_enabled(tags::KERN));
        assert!(kern_lig.is_enabled(tags::LIGA));

        let all = all_common_features();
        assert!(all.is_enabled(tags::KERN));
        assert!(all.is_enabled(tags::LIGA));
        assert!(all.is_enabled(tags::DLIG));
        assert!(all.is_enabled(tags::SMCP));
    }

    #[test]
    fn test_feature_query_empty() {
        let query = FeatureQuery::new(None, None);
        assert!(query.gsub_features().is_empty());
        assert!(query.gpos_features().is_empty());
        assert!(!query.has_feature(tags::LIGA));
    }

    #[test]
    fn test_feature_applicator_empty() {
        let selection = FeatureSelection::default();
        let applicator = FeatureApplicator::new(None, None, &selection);

        let mut glyphs = vec![1, 2, 3];
        applicator.apply_gsub(&mut glyphs);
        assert_eq!(glyphs, vec![1, 2, 3]);

        let kerning = applicator.get_kerning(&glyphs);
        assert_eq!(kerning, vec![0, 0, 0]);
    }

    #[test]
    fn test_tag_constants() {
        assert_eq!(tags::LIGA.as_str(), "liga");
        assert_eq!(tags::KERN.as_str(), "kern");
        assert_eq!(tags::SMCP.as_str(), "smcp");
    }
}
