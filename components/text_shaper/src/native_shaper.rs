//! Pure Rust text shaper implementation
//!
//! This module provides a native Rust text shaper that can work alongside or
//! replace harfbuzz_rs. It uses GSUB/GPOS tables from font_parser for:
//! - Glyph substitution (ligatures, contextual forms)
//! - Glyph positioning (kerning, mark positioning)
//!
//! Complex scripts (Arabic, Indic) fall back to HarfBuzz for full support.

use crate::types::{Script, ShapingError, ShapingOptions};
use font_parser::{
    GposSubtableData, GposTable, GsubSubtableData, GsubTable, LigatureSubst,
    PairAdjustmentSubtable, SingleSubst, Tag,
};
use font_types::types::{FontId, GlyphId, Point, PositionedGlyph, ShapedText, Vector};

/// Native Rust text shaper
///
/// Provides basic text shaping using GSUB/GPOS tables for Latin script.
/// Falls back to HarfBuzz for complex scripts (Arabic, Indic, etc.).
pub struct NativeShaper {
    /// GSUB table data (glyph substitution)
    gsub_data: Option<GsubTable>,
    /// GPOS table data (glyph positioning)
    gpos_data: Option<GposTable>,
    /// Whether to use fallback for complex scripts
    use_fallback_for_complex: bool,
    /// Units per em for the font
    units_per_em: u16,
}

/// Shaping context holding glyph information during processing
#[derive(Debug, Clone)]
struct ShapingContext {
    /// Glyph IDs after cmap lookup
    glyphs: Vec<u16>,
    /// Character indices (original string positions)
    char_indices: Vec<usize>,
    /// Cluster IDs for ligature tracking
    clusters: Vec<u32>,
    /// X advances for each glyph
    x_advances: Vec<i16>,
    /// Y advances for each glyph
    y_advances: Vec<i16>,
    /// X offsets for each glyph
    x_offsets: Vec<i16>,
    /// Y offsets for each glyph
    y_offsets: Vec<i16>,
}

impl ShapingContext {
    /// Create a new shaping context from glyph IDs
    fn new(glyphs: Vec<u16>, char_indices: Vec<usize>) -> Self {
        let len = glyphs.len();
        Self {
            glyphs,
            char_indices,
            clusters: (0..len as u32).collect(),
            x_advances: vec![0; len],
            y_advances: vec![0; len],
            x_offsets: vec![0; len],
            y_offsets: vec![0; len],
        }
    }
}

impl NativeShaper {
    /// Create a new native shaper with GSUB/GPOS data
    ///
    /// # Arguments
    /// * `gsub_data` - Optional GSUB table for glyph substitution
    /// * `gpos_data` - Optional GPOS table for glyph positioning
    /// * `units_per_em` - Font units per em
    pub fn new(
        gsub_data: Option<GsubTable>,
        gpos_data: Option<GposTable>,
        units_per_em: u16,
    ) -> Self {
        Self {
            gsub_data,
            gpos_data,
            use_fallback_for_complex: true,
            units_per_em,
        }
    }

    /// Create a shaper that always uses fallback
    #[allow(dead_code)]
    pub fn fallback_only() -> Self {
        Self {
            gsub_data: None,
            gpos_data: None,
            use_fallback_for_complex: true,
            units_per_em: 1000,
        }
    }

    /// Set whether to use fallback for complex scripts
    #[allow(dead_code)]
    pub fn set_fallback_for_complex(&mut self, use_fallback: bool) {
        self.use_fallback_for_complex = use_fallback;
    }

    /// Check if a script requires fallback (complex shaping)
    fn requires_fallback(&self, script: Script) -> bool {
        if !self.use_fallback_for_complex {
            return false;
        }

        matches!(
            script,
            Script::Arabic | Script::Hebrew // RTL and complex joining
        )
    }

    /// Shape text using native Rust implementation
    ///
    /// # Arguments
    /// * `text` - Text to shape
    /// * `font_id` - Font identifier
    /// * `options` - Shaping options
    /// * `glyph_lookup` - Function to look up glyph IDs from codepoints
    /// * `advance_lookup` - Function to look up horizontal advances
    ///
    /// # Returns
    /// Result containing shaped text or error
    #[allow(dead_code)]
    pub fn shape<F, G>(
        &self,
        text: &str,
        font_id: FontId,
        options: &ShapingOptions,
        glyph_lookup: F,
        advance_lookup: G,
    ) -> Result<ShapedText, ShapingError>
    where
        F: Fn(char) -> Option<u16>,
        G: Fn(u16) -> i16,
    {
        // Handle empty text
        if text.is_empty() {
            return Ok(ShapedText {
                glyphs: Vec::new(),
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
            });
        }

        // Check if we need fallback for complex scripts
        if self.requires_fallback(options.script) {
            return Err(ShapingError::UnsupportedScript(format!(
                "{:?} requires fallback shaper",
                options.script
            )));
        }

        // Convert text to glyph IDs using cmap
        let (glyphs, char_indices) = self.text_to_glyphs(text, &glyph_lookup)?;

        // Create shaping context
        let mut ctx = ShapingContext::new(glyphs, char_indices);

        // Set initial advances
        for (i, &glyph_id) in ctx.glyphs.iter().enumerate() {
            ctx.x_advances[i] = advance_lookup(glyph_id);
        }

        // Apply GSUB features (substitution)
        self.apply_gsub_features(&mut ctx, options)?;

        // Recalculate advances after substitution (ligatures may have changed glyphs)
        for (i, &glyph_id) in ctx.glyphs.iter().enumerate() {
            if i < ctx.x_advances.len() {
                ctx.x_advances[i] = advance_lookup(glyph_id);
            }
        }

        // Apply GPOS features (positioning)
        self.apply_gpos_features(&mut ctx, options)?;

        // Convert to output format
        self.finalize_shaping(ctx, font_id, options)
    }

    /// Convert text to glyph IDs using cmap lookup
    fn text_to_glyphs<F>(
        &self,
        text: &str,
        glyph_lookup: &F,
    ) -> Result<(Vec<u16>, Vec<usize>), ShapingError>
    where
        F: Fn(char) -> Option<u16>,
    {
        let mut glyphs = Vec::with_capacity(text.len());
        let mut char_indices = Vec::with_capacity(text.len());

        for (idx, ch) in text.chars().enumerate() {
            let glyph_id = glyph_lookup(ch).unwrap_or(0); // 0 is typically .notdef
            glyphs.push(glyph_id);
            char_indices.push(idx);
        }

        Ok((glyphs, char_indices))
    }

    /// Apply GSUB features (glyph substitution)
    fn apply_gsub_features(
        &self,
        ctx: &mut ShapingContext,
        options: &ShapingOptions,
    ) -> Result<(), ShapingError> {
        let gsub = match &self.gsub_data {
            Some(table) => table,
            None => return Ok(()), // No GSUB table, skip
        };

        // Get script tag
        let script_tag = self.script_to_tag(options.script);

        // Get feature indices for this script
        let feature_indices = gsub.get_feature_indices(script_tag, None);

        // Apply enabled features
        for feature_idx in feature_indices {
            if let Some(feature) = gsub.features.features.get(feature_idx as usize) {
                // Check if this feature is enabled
                if self.is_feature_enabled(&feature.tag, options) {
                    for &lookup_idx in &feature.lookup_indices {
                        self.apply_gsub_lookup(ctx, gsub, lookup_idx as usize)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply a single GSUB lookup
    fn apply_gsub_lookup(
        &self,
        ctx: &mut ShapingContext,
        gsub: &GsubTable,
        lookup_idx: usize,
    ) -> Result<(), ShapingError> {
        let lookup = match gsub.lookups.lookups.get(lookup_idx) {
            Some(l) => l,
            None => return Ok(()),
        };

        for subtable in &lookup.subtables {
            match subtable {
                GsubSubtableData::Single(single) => {
                    self.apply_single_substitution(ctx, single);
                }
                GsubSubtableData::Ligature(ligature) => {
                    self.apply_ligature_substitution(ctx, ligature);
                }
                _ => {
                    // Other types not yet implemented
                }
            }
        }

        Ok(())
    }

    /// Apply single substitution
    fn apply_single_substitution(&self, ctx: &mut ShapingContext, single: &SingleSubst) {
        for glyph in ctx.glyphs.iter_mut() {
            if let Some(substituted) = single.substitute(*glyph) {
                *glyph = substituted;
            }
        }
    }

    /// Apply ligature substitution
    fn apply_ligature_substitution(&self, ctx: &mut ShapingContext, ligature: &LigatureSubst) {
        let mut i = 0;
        while i < ctx.glyphs.len() {
            if let Some((lig_glyph, consumed)) = ligature.match_ligature(&ctx.glyphs[i..]) {
                // Replace the sequence with the ligature glyph
                ctx.glyphs[i] = lig_glyph;

                // Remove the consumed glyphs (all but the first)
                for _ in 1..consumed {
                    if i + 1 < ctx.glyphs.len() {
                        ctx.glyphs.remove(i + 1);
                        ctx.char_indices.remove(i + 1);
                        ctx.clusters.remove(i + 1);
                        ctx.x_advances.remove(i + 1);
                        ctx.y_advances.remove(i + 1);
                        ctx.x_offsets.remove(i + 1);
                        ctx.y_offsets.remove(i + 1);
                    }
                }
            }
            i += 1;
        }
    }

    /// Apply GPOS features (glyph positioning)
    fn apply_gpos_features(
        &self,
        ctx: &mut ShapingContext,
        options: &ShapingOptions,
    ) -> Result<(), ShapingError> {
        let gpos = match &self.gpos_data {
            Some(table) => table,
            None => return Ok(()), // No GPOS table, skip
        };

        // Get script tag
        let script_tag = self.script_to_tag(options.script);

        // Get feature indices for this script
        let feature_indices = gpos.get_feature_indices(script_tag, None);

        // Apply enabled features
        for feature_idx in feature_indices {
            if let Some(feature) = gpos.features.features.get(feature_idx as usize) {
                // Check if this feature is enabled
                if self.is_feature_enabled(&feature.tag, options) {
                    for &lookup_idx in &feature.lookup_indices {
                        self.apply_gpos_lookup(ctx, gpos, lookup_idx as usize)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply a single GPOS lookup
    fn apply_gpos_lookup(
        &self,
        ctx: &mut ShapingContext,
        gpos: &GposTable,
        lookup_idx: usize,
    ) -> Result<(), ShapingError> {
        let lookup = match gpos.lookups.lookups.get(lookup_idx) {
            Some(l) => l,
            None => return Ok(()),
        };

        for subtable in &lookup.subtables {
            match subtable {
                GposSubtableData::PairAdjustment(pair) => {
                    self.apply_pair_adjustment(ctx, pair);
                }
                _ => {
                    // Other types not yet implemented
                }
            }
        }

        Ok(())
    }

    /// Apply pair adjustment (kerning)
    fn apply_pair_adjustment(&self, ctx: &mut ShapingContext, pair: &PairAdjustmentSubtable) {
        for i in 0..ctx.glyphs.len().saturating_sub(1) {
            let first = ctx.glyphs[i];
            let second = ctx.glyphs[i + 1];

            if let Some((value1, value2)) = pair.get_kerning(first, second) {
                // Apply adjustments from first glyph
                ctx.x_advances[i] = ctx.x_advances[i].saturating_add(value1.x_advance);
                ctx.y_advances[i] = ctx.y_advances[i].saturating_add(value1.y_advance);
                ctx.x_offsets[i] = ctx.x_offsets[i].saturating_add(value1.x_placement);
                ctx.y_offsets[i] = ctx.y_offsets[i].saturating_add(value1.y_placement);

                // Apply adjustments from second glyph
                ctx.x_offsets[i + 1] = ctx.x_offsets[i + 1].saturating_add(value2.x_placement);
                ctx.y_offsets[i + 1] = ctx.y_offsets[i + 1].saturating_add(value2.y_placement);
            }
        }
    }

    /// Check if a feature is enabled in the options
    fn is_feature_enabled(&self, tag: &Tag, options: &ShapingOptions) -> bool {
        let tag_str = tag.as_str();

        // Check explicit feature settings
        if let Some(&value) = options.features.get(tag_str) {
            return value > 0;
        }

        // Default feature states
        match tag_str {
            "kern" => options.kerning,
            "liga" | "clig" => options.ligatures,
            "calt" | "ccmp" | "locl" => true, // Usually enabled by default
            _ => false,
        }
    }

    /// Convert Script enum to OpenType tag
    fn script_to_tag(&self, script: Script) -> Tag {
        match script {
            Script::Latin => Tag::new("latn").unwrap_or(Tag([b'l', b'a', b't', b'n'])),
            Script::Arabic => Tag::new("arab").unwrap_or(Tag([b'a', b'r', b'a', b'b'])),
            Script::Hebrew => Tag::new("hebr").unwrap_or(Tag([b'h', b'e', b'b', b'r'])),
            Script::Cyrillic => Tag::new("cyrl").unwrap_or(Tag([b'c', b'y', b'r', b'l'])),
            Script::Greek => Tag::new("grek").unwrap_or(Tag([b'g', b'r', b'e', b'k'])),
            Script::Han => Tag::new("hani").unwrap_or(Tag([b'h', b'a', b'n', b'i'])),
            Script::Hangul => Tag::new("hang").unwrap_or(Tag([b'h', b'a', b'n', b'g'])),
            Script::Hiragana => Tag::new("hira").unwrap_or(Tag([b'h', b'i', b'r', b'a'])),
            Script::Katakana => Tag::new("kana").unwrap_or(Tag([b'k', b'a', b'n', b'a'])),
            Script::Common => Tag::new("DFLT").unwrap_or(Tag([b'D', b'F', b'L', b'T'])),
        }
    }

    /// Finalize shaping and produce output
    fn finalize_shaping(
        &self,
        ctx: ShapingContext,
        font_id: FontId,
        options: &ShapingOptions,
    ) -> Result<ShapedText, ShapingError> {
        let mut glyphs = Vec::with_capacity(ctx.glyphs.len());
        let mut cursor_x = 0.0f32;
        let cursor_y = 0.0f32;

        let scale = 1.0; // Scaling will be applied later by the caller

        for i in 0..ctx.glyphs.len() {
            let x_advance = ctx.x_advances.get(i).copied().unwrap_or(0) as f32 * scale;
            let y_advance = ctx.y_advances.get(i).copied().unwrap_or(0) as f32 * scale;
            let x_offset = ctx.x_offsets.get(i).copied().unwrap_or(0) as f32 * scale;
            let y_offset = ctx.y_offsets.get(i).copied().unwrap_or(0) as f32 * scale;

            // Apply additional spacing from options
            let adjusted_x_advance = x_advance + options.letter_spacing;

            glyphs.push(PositionedGlyph {
                glyph_id: GlyphId {
                    id: ctx.glyphs[i] as u32,
                },
                font_id,
                position: Point {
                    x: cursor_x + x_offset,
                    y: cursor_y + y_offset,
                },
                advance: Vector {
                    x: adjusted_x_advance,
                    y: y_advance,
                },
                offset: Vector {
                    x: x_offset,
                    y: y_offset,
                },
            });

            cursor_x += adjusted_x_advance;
        }

        // Calculate dimensions (placeholder - would need font metrics)
        let width = cursor_x;
        let height = self.units_per_em as f32 * scale;
        let baseline = height * 0.8; // Approximate

        Ok(ShapedText {
            glyphs,
            width,
            height,
            baseline,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_types::types::Direction;
    use std::collections::HashMap;

    fn create_test_options() -> ShapingOptions {
        ShapingOptions {
            script: Script::Latin,
            language: crate::types::Language {
                tag: "en".to_string(),
            },
            direction: Direction::LeftToRight,
            features: HashMap::new(),
            kerning: true,
            ligatures: true,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        }
    }

    #[test]
    fn test_native_shaper_creation() {
        let shaper = NativeShaper::new(None, None, 1000);
        assert!(shaper.gsub_data.is_none());
        assert!(shaper.gpos_data.is_none());
    }

    #[test]
    fn test_fallback_only_shaper() {
        let shaper = NativeShaper::fallback_only();
        assert!(shaper.use_fallback_for_complex);
    }

    #[test]
    fn test_requires_fallback_for_arabic() {
        let shaper = NativeShaper::new(None, None, 1000);
        assert!(shaper.requires_fallback(Script::Arabic));
        assert!(shaper.requires_fallback(Script::Hebrew));
        assert!(!shaper.requires_fallback(Script::Latin));
    }

    #[test]
    fn test_script_to_tag() {
        let shaper = NativeShaper::new(None, None, 1000);
        let tag = shaper.script_to_tag(Script::Latin);
        assert_eq!(tag.as_str(), "latn");
    }

    #[test]
    fn test_empty_text_shaping() {
        let shaper = NativeShaper::new(None, None, 1000);
        let options = create_test_options();
        let font_id: FontId = 1;

        let result = shaper.shape("", font_id, &options, |_| Some(0), |_| 500);
        assert!(result.is_ok());
        let shaped = result.unwrap();
        assert!(shaped.glyphs.is_empty());
        assert_eq!(shaped.width, 0.0);
    }

    #[test]
    fn test_basic_text_shaping() {
        let shaper = NativeShaper::new(None, None, 1000);
        let options = create_test_options();
        let font_id: FontId = 1;

        // Simple glyph lookup: 'A' = glyph 65, etc.
        let glyph_lookup = |c: char| Some(c as u16);
        // Simple advance: each glyph advances 500 units
        let advance_lookup = |_: u16| 500i16;

        let result = shaper.shape("ABC", font_id, &options, glyph_lookup, advance_lookup);
        assert!(result.is_ok());

        let shaped = result.unwrap();
        assert_eq!(shaped.glyphs.len(), 3);

        // Check glyph IDs
        assert_eq!(shaped.glyphs[0].glyph_id.id, 'A' as u32);
        assert_eq!(shaped.glyphs[1].glyph_id.id, 'B' as u32);
        assert_eq!(shaped.glyphs[2].glyph_id.id, 'C' as u32);

        // Check positions
        assert_eq!(shaped.glyphs[0].position.x, 0.0);
        assert_eq!(shaped.glyphs[1].position.x, 500.0);
        assert_eq!(shaped.glyphs[2].position.x, 1000.0);
    }

    #[test]
    fn test_arabic_requires_fallback() {
        let shaper = NativeShaper::new(None, None, 1000);
        let mut options = create_test_options();
        options.script = Script::Arabic;
        let font_id: FontId = 1;

        let result = shaper.shape("test", font_id, &options, |_| Some(0), |_| 500);
        assert!(result.is_err());

        if let Err(ShapingError::UnsupportedScript(msg)) = result {
            assert!(msg.contains("fallback"));
        } else {
            panic!("Expected UnsupportedScript error");
        }
    }

    #[test]
    fn test_letter_spacing() {
        let shaper = NativeShaper::new(None, None, 1000);
        let mut options = create_test_options();
        options.letter_spacing = 10.0;
        let font_id: FontId = 1;

        let result = shaper.shape("AB", font_id, &options, |c| Some(c as u16), |_| 500);
        assert!(result.is_ok());

        let shaped = result.unwrap();

        // Each glyph should have advance of 500 + 10 = 510
        assert_eq!(shaped.glyphs[0].advance.x, 510.0);
        assert_eq!(shaped.glyphs[1].advance.x, 510.0);

        // Second glyph should be at position 510
        assert_eq!(shaped.glyphs[1].position.x, 510.0);
    }

    #[test]
    fn test_shaping_context_creation() {
        let glyphs = vec![65, 66, 67];
        let indices = vec![0, 1, 2];
        let ctx = ShapingContext::new(glyphs.clone(), indices.clone());

        assert_eq!(ctx.glyphs, glyphs);
        assert_eq!(ctx.char_indices, indices);
        assert_eq!(ctx.clusters, vec![0, 1, 2]);
        assert_eq!(ctx.x_advances.len(), 3);
    }
}
