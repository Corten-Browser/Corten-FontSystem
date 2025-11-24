//! GSUB (Glyph Substitution) table parsing
//!
//! The GSUB table contains information for substituting glyphs to render
//! the scripts and language systems supported in a font. This includes
//! ligatures, contextual substitutions, and alternates.

use crate::types::{GlyphId, Tag};
use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

/// GSUB table
#[derive(Debug, Clone)]
pub struct GsubTable {
    /// Major version of the GSUB table
    pub major_version: u16,
    /// Minor version of the GSUB table
    pub minor_version: u16,
    /// Script list
    pub scripts: ScriptList,
    /// Feature list
    pub features: FeatureList,
    /// Lookup list
    pub lookups: LookupList,
}

/// Script list containing all scripts supported by the font
#[derive(Debug, Clone, Default)]
pub struct ScriptList {
    /// Scripts indexed by tag
    pub scripts: HashMap<Tag, ScriptRecord>,
}

/// A single script record
#[derive(Debug, Clone)]
pub struct ScriptRecord {
    /// Script tag (e.g., 'latn', 'arab')
    pub tag: Tag,
    /// Default language system
    pub default_lang_sys: Option<LangSys>,
    /// Language systems for this script
    pub lang_sys: HashMap<Tag, LangSys>,
}

/// Language system record
#[derive(Debug, Clone)]
pub struct LangSys {
    /// Required feature index (0xFFFF if none)
    pub required_feature_index: Option<u16>,
    /// Feature indices
    pub feature_indices: Vec<u16>,
}

/// Feature list
#[derive(Debug, Clone, Default)]
pub struct FeatureList {
    /// Features indexed by their index in the list
    pub features: Vec<FeatureRecord>,
}

/// A single feature record
#[derive(Debug, Clone)]
pub struct FeatureRecord {
    /// Feature tag (e.g., 'liga', 'kern')
    pub tag: Tag,
    /// Lookup list indices for this feature
    pub lookup_indices: Vec<u16>,
}

/// Lookup list containing all lookups
#[derive(Debug, Clone, Default)]
pub struct LookupList {
    /// Lookups
    pub lookups: Vec<Lookup>,
}

/// A single lookup
#[derive(Debug, Clone)]
pub struct Lookup {
    /// Lookup type
    pub lookup_type: LookupType,
    /// Lookup flags
    pub flags: LookupFlags,
    /// Subtables for this lookup
    pub subtables: Vec<SubtableData>,
}

/// Lookup type for GSUB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupType {
    /// Single substitution
    Single = 1,
    /// Multiple substitution
    Multiple = 2,
    /// Alternate substitution
    Alternate = 3,
    /// Ligature substitution
    Ligature = 4,
    /// Contextual substitution
    Context = 5,
    /// Chaining contextual substitution
    ChainingContext = 6,
    /// Extension substitution
    Extension = 7,
    /// Reverse chaining contextual single substitution
    ReverseChaining = 8,
}

impl TryFrom<u16> for LookupType {
    type Error = ParseError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(LookupType::Single),
            2 => Ok(LookupType::Multiple),
            3 => Ok(LookupType::Alternate),
            4 => Ok(LookupType::Ligature),
            5 => Ok(LookupType::Context),
            6 => Ok(LookupType::ChainingContext),
            7 => Ok(LookupType::Extension),
            8 => Ok(LookupType::ReverseChaining),
            _ => Err(ParseError::CorruptedData(format!(
                "Invalid GSUB lookup type: {}",
                value
            ))),
        }
    }
}

/// Lookup flags
#[derive(Debug, Clone, Copy, Default)]
pub struct LookupFlags {
    /// Right to left flag
    pub right_to_left: bool,
    /// Ignore base glyphs
    pub ignore_base_glyphs: bool,
    /// Ignore ligatures
    pub ignore_ligatures: bool,
    /// Ignore marks
    pub ignore_marks: bool,
    /// Mark attachment type filter
    pub mark_attachment_type: u8,
    /// Mark filtering set
    pub mark_filtering_set: Option<u16>,
}

impl LookupFlags {
    /// Parse lookup flags from raw values
    pub fn parse(flags: u16, mark_filtering_set: Option<u16>) -> Self {
        LookupFlags {
            right_to_left: (flags & 0x0001) != 0,
            ignore_base_glyphs: (flags & 0x0002) != 0,
            ignore_ligatures: (flags & 0x0004) != 0,
            ignore_marks: (flags & 0x0008) != 0,
            mark_attachment_type: ((flags >> 8) & 0xFF) as u8,
            mark_filtering_set,
        }
    }
}

/// Subtable data for different lookup types
#[derive(Debug, Clone)]
pub enum SubtableData {
    /// Single substitution (format 1: delta, format 2: array)
    Single(SingleSubst),
    /// Multiple substitution
    Multiple(MultipleSubst),
    /// Alternate substitution
    Alternate(AlternateSubst),
    /// Ligature substitution
    Ligature(LigatureSubst),
    /// Contextual substitution
    Context(ContextSubst),
    /// Chaining contextual substitution
    ChainingContext(ChainingContextSubst),
    /// Raw data for unimplemented types
    Raw(Vec<u8>),
}

/// Single substitution subtable
#[derive(Debug, Clone)]
pub struct SingleSubst {
    /// Coverage table
    pub coverage: Coverage,
    /// Substitution mapping (input -> output)
    pub substitutions: HashMap<GlyphId, GlyphId>,
}

impl SingleSubst {
    /// Apply single substitution to a glyph
    pub fn substitute(&self, glyph: GlyphId) -> Option<GlyphId> {
        if self.coverage.contains(glyph) {
            self.substitutions.get(&glyph).copied()
        } else {
            None
        }
    }
}

/// Multiple substitution subtable (one-to-many)
#[derive(Debug, Clone)]
pub struct MultipleSubst {
    /// Coverage table
    pub coverage: Coverage,
    /// Sequences (input glyph -> sequence of output glyphs)
    pub sequences: HashMap<GlyphId, Vec<GlyphId>>,
}

impl MultipleSubst {
    /// Apply multiple substitution to a glyph
    pub fn substitute(&self, glyph: GlyphId) -> Option<&Vec<GlyphId>> {
        if self.coverage.contains(glyph) {
            self.sequences.get(&glyph)
        } else {
            None
        }
    }
}

/// Alternate substitution subtable
#[derive(Debug, Clone)]
pub struct AlternateSubst {
    /// Coverage table
    pub coverage: Coverage,
    /// Alternate sets (input glyph -> list of alternates)
    pub alternate_sets: HashMap<GlyphId, Vec<GlyphId>>,
}

impl AlternateSubst {
    /// Get alternates for a glyph
    pub fn get_alternates(&self, glyph: GlyphId) -> Option<&Vec<GlyphId>> {
        if self.coverage.contains(glyph) {
            self.alternate_sets.get(&glyph)
        } else {
            None
        }
    }
}

/// Ligature substitution subtable
#[derive(Debug, Clone)]
pub struct LigatureSubst {
    /// Coverage table (first glyph of ligature)
    pub coverage: Coverage,
    /// Ligature sets indexed by first glyph
    pub ligature_sets: HashMap<GlyphId, Vec<Ligature>>,
}

/// A single ligature
#[derive(Debug, Clone)]
pub struct Ligature {
    /// Ligature glyph (output)
    pub ligature_glyph: GlyphId,
    /// Component glyphs (excluding first glyph)
    pub components: Vec<GlyphId>,
}

impl LigatureSubst {
    /// Try to match a ligature starting at a glyph sequence
    pub fn match_ligature(&self, glyphs: &[GlyphId]) -> Option<(GlyphId, usize)> {
        if glyphs.is_empty() {
            return None;
        }

        let first = glyphs[0];
        if !self.coverage.contains(first) {
            return None;
        }

        if let Some(ligature_set) = self.ligature_sets.get(&first) {
            // Try to match each ligature (longer ligatures should be tried first)
            for ligature in ligature_set {
                if ligature.components.len() + 1 <= glyphs.len() {
                    let matches = ligature
                        .components
                        .iter()
                        .zip(&glyphs[1..])
                        .all(|(a, b)| a == b);
                    if matches {
                        return Some((ligature.ligature_glyph, ligature.components.len() + 1));
                    }
                }
            }
        }

        None
    }
}

/// Contextual substitution subtable
#[derive(Debug, Clone)]
pub struct ContextSubst {
    /// Format of the subtable
    pub format: u16,
    /// Rules by coverage index (format 1)
    pub rule_sets: HashMap<GlyphId, Vec<ContextRule>>,
    /// Class-based rules (format 2)
    pub class_rules: Option<ClassBasedContext>,
    /// Coverage-based rules (format 3)
    pub coverage_rules: Option<CoverageBasedContext>,
}

/// A single context rule
#[derive(Debug, Clone)]
pub struct ContextRule {
    /// Input glyphs (or classes) after the first
    pub input: Vec<u16>,
    /// Substitution lookups to apply
    pub lookups: Vec<SubstLookupRecord>,
}

/// Class-based context rules
#[derive(Debug, Clone)]
pub struct ClassBasedContext {
    /// Input coverage
    pub coverage: Coverage,
    /// Class definition
    pub class_def: ClassDef,
    /// Rules by class
    pub class_rules: HashMap<u16, Vec<ContextRule>>,
}

/// Coverage-based context rules
#[derive(Debug, Clone)]
pub struct CoverageBasedContext {
    /// Input coverages
    pub coverages: Vec<Coverage>,
    /// Substitution lookups
    pub lookups: Vec<SubstLookupRecord>,
}

/// Chaining contextual substitution subtable
#[derive(Debug, Clone)]
pub struct ChainingContextSubst {
    /// Format of the subtable
    pub format: u16,
    /// Rules by coverage index (format 1)
    pub rule_sets: HashMap<GlyphId, Vec<ChainContextRule>>,
    /// Class-based rules (format 2)
    pub class_rules: Option<ClassBasedChainContext>,
    /// Coverage-based rules (format 3)
    pub coverage_rules: Option<CoverageBasedChainContext>,
}

/// A single chaining context rule
#[derive(Debug, Clone)]
pub struct ChainContextRule {
    /// Backtrack glyphs (or classes)
    pub backtrack: Vec<u16>,
    /// Input glyphs (or classes)
    pub input: Vec<u16>,
    /// Lookahead glyphs (or classes)
    pub lookahead: Vec<u16>,
    /// Substitution lookups to apply
    pub lookups: Vec<SubstLookupRecord>,
}

/// Class-based chaining context
#[derive(Debug, Clone)]
pub struct ClassBasedChainContext {
    /// Backtrack coverage
    pub backtrack_coverage: Coverage,
    /// Input coverage
    pub input_coverage: Coverage,
    /// Lookahead coverage
    pub lookahead_coverage: Coverage,
    /// Backtrack class definition
    pub backtrack_class_def: ClassDef,
    /// Input class definition
    pub input_class_def: ClassDef,
    /// Lookahead class definition
    pub lookahead_class_def: ClassDef,
    /// Rules by class
    pub class_rules: HashMap<u16, Vec<ChainContextRule>>,
}

/// Coverage-based chaining context
#[derive(Debug, Clone)]
pub struct CoverageBasedChainContext {
    /// Backtrack coverages
    pub backtrack: Vec<Coverage>,
    /// Input coverages
    pub input: Vec<Coverage>,
    /// Lookahead coverages
    pub lookahead: Vec<Coverage>,
    /// Substitution lookups
    pub lookups: Vec<SubstLookupRecord>,
}

/// Substitution lookup record (used in contextual lookups)
#[derive(Debug, Clone, Copy)]
pub struct SubstLookupRecord {
    /// Sequence index (position in the input sequence)
    pub sequence_index: u16,
    /// Lookup index to apply
    pub lookup_index: u16,
}

/// Coverage table
#[derive(Debug, Clone, Default)]
pub struct Coverage {
    /// Glyph to coverage index mapping
    pub glyphs: HashMap<GlyphId, u16>,
}

impl Coverage {
    /// Check if a glyph is covered
    pub fn contains(&self, glyph: GlyphId) -> bool {
        self.glyphs.contains_key(&glyph)
    }

    /// Get coverage index for a glyph
    pub fn get_index(&self, glyph: GlyphId) -> Option<u16> {
        self.glyphs.get(&glyph).copied()
    }

    /// Get all covered glyphs
    pub fn glyphs(&self) -> impl Iterator<Item = GlyphId> + '_ {
        self.glyphs.keys().copied()
    }

    /// Parse coverage table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 4 {
            return Err(ParseError::CorruptedData(
                "Coverage table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let format = cursor.read_u16::<BigEndian>()?;

        let mut glyphs = HashMap::new();

        match format {
            1 => {
                // Format 1: list of glyph IDs
                let glyph_count = cursor.read_u16::<BigEndian>()?;
                for i in 0..glyph_count {
                    let glyph_id = cursor.read_u16::<BigEndian>()?;
                    glyphs.insert(glyph_id, i);
                }
            }
            2 => {
                // Format 2: ranges
                let range_count = cursor.read_u16::<BigEndian>()?;
                for _ in 0..range_count {
                    let start = cursor.read_u16::<BigEndian>()?;
                    let end = cursor.read_u16::<BigEndian>()?;
                    let start_index = cursor.read_u16::<BigEndian>()?;
                    for (i, glyph_id) in (start..=end).enumerate() {
                        glyphs.insert(glyph_id, start_index + i as u16);
                    }
                }
            }
            _ => {
                return Err(ParseError::CorruptedData(format!(
                    "Unknown coverage format: {}",
                    format
                )));
            }
        }

        Ok(Coverage { glyphs })
    }
}

/// Class definition table
#[derive(Debug, Clone, Default)]
pub struct ClassDef {
    /// Glyph to class mapping
    classes: HashMap<GlyphId, u16>,
}

impl ClassDef {
    /// Get class for a glyph (returns 0 if not defined)
    pub fn get_class(&self, glyph: GlyphId) -> u16 {
        self.classes.get(&glyph).copied().unwrap_or(0)
    }

    /// Parse from raw data
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 4 {
            return Err(ParseError::CorruptedData(
                "ClassDef table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let format = cursor.read_u16::<BigEndian>()?;

        let mut classes = HashMap::new();

        match format {
            1 => {
                // Format 1: array
                let start_glyph = cursor.read_u16::<BigEndian>()?;
                let glyph_count = cursor.read_u16::<BigEndian>()?;
                for i in 0..glyph_count {
                    let class = cursor.read_u16::<BigEndian>()?;
                    if class != 0 {
                        classes.insert(start_glyph + i, class);
                    }
                }
            }
            2 => {
                // Format 2: ranges
                let range_count = cursor.read_u16::<BigEndian>()?;
                for _ in 0..range_count {
                    let start = cursor.read_u16::<BigEndian>()?;
                    let end = cursor.read_u16::<BigEndian>()?;
                    let class = cursor.read_u16::<BigEndian>()?;
                    if class != 0 {
                        for glyph_id in start..=end {
                            classes.insert(glyph_id, class);
                        }
                    }
                }
            }
            _ => {
                return Err(ParseError::CorruptedData(format!(
                    "Unknown ClassDef format: {}",
                    format
                )));
            }
        }

        Ok(ClassDef { classes })
    }
}

impl GsubTable {
    /// Parse GSUB table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 10 {
            return Err(ParseError::CorruptedData(
                "GSUB table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        let major_version = cursor.read_u16::<BigEndian>()?;
        let minor_version = cursor.read_u16::<BigEndian>()?;

        if major_version != 1 {
            return Err(ParseError::UnsupportedVersion);
        }

        let script_list_offset = cursor.read_u16::<BigEndian>()? as usize;
        let feature_list_offset = cursor.read_u16::<BigEndian>()? as usize;
        let lookup_list_offset = cursor.read_u16::<BigEndian>()? as usize;

        // Parse script list
        let scripts = if script_list_offset > 0 && script_list_offset < data.len() {
            Self::parse_script_list(&data[script_list_offset..])?
        } else {
            ScriptList::default()
        };

        // Parse feature list
        let features = if feature_list_offset > 0 && feature_list_offset < data.len() {
            Self::parse_feature_list(&data[feature_list_offset..])?
        } else {
            FeatureList::default()
        };

        // Parse lookup list
        let lookups = if lookup_list_offset > 0 && lookup_list_offset < data.len() {
            Self::parse_lookup_list(&data[lookup_list_offset..])?
        } else {
            LookupList::default()
        };

        Ok(GsubTable {
            major_version,
            minor_version,
            scripts,
            features,
            lookups,
        })
    }

    fn parse_script_list(data: &[u8]) -> Result<ScriptList, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData(
                "Script list too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let script_count = cursor.read_u16::<BigEndian>()?;

        let mut scripts = HashMap::new();

        for _ in 0..script_count {
            let tag = Tag::from_bytes(cursor.read_u32::<BigEndian>()?);
            let offset = cursor.read_u16::<BigEndian>()? as usize;

            if offset < data.len() {
                let script = Self::parse_script_record(&data[offset..], tag)?;
                scripts.insert(tag, script);
            }
        }

        Ok(ScriptList { scripts })
    }

    fn parse_script_record(data: &[u8], tag: Tag) -> Result<ScriptRecord, ParseError> {
        if data.len() < 4 {
            return Err(ParseError::CorruptedData(
                "Script record too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let default_lang_sys_offset = cursor.read_u16::<BigEndian>()? as usize;
        let lang_sys_count = cursor.read_u16::<BigEndian>()?;

        let default_lang_sys =
            if default_lang_sys_offset > 0 && default_lang_sys_offset < data.len() {
                Some(Self::parse_lang_sys(&data[default_lang_sys_offset..])?)
            } else {
                None
            };

        let mut lang_sys = HashMap::new();
        for _ in 0..lang_sys_count {
            let lang_tag = Tag::from_bytes(cursor.read_u32::<BigEndian>()?);
            let offset = cursor.read_u16::<BigEndian>()? as usize;

            if offset < data.len() {
                let ls = Self::parse_lang_sys(&data[offset..])?;
                lang_sys.insert(lang_tag, ls);
            }
        }

        Ok(ScriptRecord {
            tag,
            default_lang_sys,
            lang_sys,
        })
    }

    fn parse_lang_sys(data: &[u8]) -> Result<LangSys, ParseError> {
        if data.len() < 6 {
            return Err(ParseError::CorruptedData("LangSys too short".to_string()));
        }

        let mut cursor = Cursor::new(data);
        let _lookup_order = cursor.read_u16::<BigEndian>()?; // Reserved
        let required_feature_index = cursor.read_u16::<BigEndian>()?;
        let feature_count = cursor.read_u16::<BigEndian>()?;

        let required_feature_index = if required_feature_index == 0xFFFF {
            None
        } else {
            Some(required_feature_index)
        };

        let mut feature_indices = Vec::with_capacity(feature_count as usize);
        for _ in 0..feature_count {
            feature_indices.push(cursor.read_u16::<BigEndian>()?);
        }

        Ok(LangSys {
            required_feature_index,
            feature_indices,
        })
    }

    fn parse_feature_list(data: &[u8]) -> Result<FeatureList, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData(
                "Feature list too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let feature_count = cursor.read_u16::<BigEndian>()?;

        let mut features = Vec::with_capacity(feature_count as usize);

        for _ in 0..feature_count {
            let tag = Tag::from_bytes(cursor.read_u32::<BigEndian>()?);
            let offset = cursor.read_u16::<BigEndian>()? as usize;

            if offset < data.len() {
                let feature = Self::parse_feature_record(&data[offset..], tag)?;
                features.push(feature);
            }
        }

        Ok(FeatureList { features })
    }

    fn parse_feature_record(data: &[u8], tag: Tag) -> Result<FeatureRecord, ParseError> {
        if data.len() < 4 {
            return Err(ParseError::CorruptedData(
                "Feature record too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let _feature_params = cursor.read_u16::<BigEndian>()?; // Usually NULL
        let lookup_count = cursor.read_u16::<BigEndian>()?;

        let mut lookup_indices = Vec::with_capacity(lookup_count as usize);
        for _ in 0..lookup_count {
            lookup_indices.push(cursor.read_u16::<BigEndian>()?);
        }

        Ok(FeatureRecord {
            tag,
            lookup_indices,
        })
    }

    fn parse_lookup_list(data: &[u8]) -> Result<LookupList, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData(
                "Lookup list too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let lookup_count = cursor.read_u16::<BigEndian>()?;

        let mut offsets = Vec::with_capacity(lookup_count as usize);
        for _ in 0..lookup_count {
            offsets.push(cursor.read_u16::<BigEndian>()? as usize);
        }

        let mut lookups = Vec::with_capacity(lookup_count as usize);
        for offset in offsets {
            if offset < data.len() {
                let lookup = Self::parse_lookup(&data[offset..], data)?;
                lookups.push(lookup);
            }
        }

        Ok(LookupList { lookups })
    }

    fn parse_lookup(data: &[u8], _full_data: &[u8]) -> Result<Lookup, ParseError> {
        if data.len() < 6 {
            return Err(ParseError::CorruptedData("Lookup too short".to_string()));
        }

        let mut cursor = Cursor::new(data);
        let lookup_type = cursor.read_u16::<BigEndian>()?;
        let lookup_flags = cursor.read_u16::<BigEndian>()?;
        let subtable_count = cursor.read_u16::<BigEndian>()?;

        let mut subtable_offsets = Vec::with_capacity(subtable_count as usize);
        for _ in 0..subtable_count {
            subtable_offsets.push(cursor.read_u16::<BigEndian>()? as usize);
        }

        // Check for mark filtering set
        let use_mark_filtering_set = (lookup_flags & 0x0010) != 0;
        let mark_filtering_set = if use_mark_filtering_set {
            Some(cursor.read_u16::<BigEndian>()?)
        } else {
            None
        };

        let flags = LookupFlags::parse(lookup_flags, mark_filtering_set);
        let lookup_type = LookupType::try_from(lookup_type)?;

        let mut subtables = Vec::with_capacity(subtable_count as usize);
        for offset in subtable_offsets {
            if offset < data.len() {
                let subtable_data = &data[offset..];
                let subtable = Self::parse_subtable(subtable_data, lookup_type)?;
                subtables.push(subtable);
            }
        }

        Ok(Lookup {
            lookup_type,
            flags,
            subtables,
        })
    }

    fn parse_subtable(data: &[u8], lookup_type: LookupType) -> Result<SubtableData, ParseError> {
        match lookup_type {
            LookupType::Single => Ok(SubtableData::Single(Self::parse_single_subst(data)?)),
            LookupType::Multiple => Ok(SubtableData::Multiple(Self::parse_multiple_subst(data)?)),
            LookupType::Alternate => {
                Ok(SubtableData::Alternate(Self::parse_alternate_subst(data)?))
            }
            LookupType::Ligature => Ok(SubtableData::Ligature(Self::parse_ligature_subst(data)?)),
            LookupType::Context => Ok(SubtableData::Context(Self::parse_context_subst(data)?)),
            LookupType::ChainingContext => Ok(SubtableData::ChainingContext(
                Self::parse_chaining_context_subst(data)?,
            )),
            _ => Ok(SubtableData::Raw(data.to_vec())),
        }
    }

    fn parse_single_subst(data: &[u8]) -> Result<SingleSubst, ParseError> {
        if data.len() < 6 {
            return Err(ParseError::CorruptedData(
                "Single substitution subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let format = cursor.read_u16::<BigEndian>()?;
        let coverage_offset = cursor.read_u16::<BigEndian>()? as usize;

        let coverage = if coverage_offset < data.len() {
            Coverage::parse(&data[coverage_offset..])?
        } else {
            Coverage::default()
        };

        let mut substitutions = HashMap::new();

        match format {
            1 => {
                // Format 1: add delta to glyph ID
                let delta = cursor.read_i16::<BigEndian>()?;
                for glyph in coverage.glyphs() {
                    let new_glyph = (glyph as i32 + delta as i32) as u16;
                    substitutions.insert(glyph, new_glyph);
                }
            }
            2 => {
                // Format 2: array of substitutions
                let glyph_count = cursor.read_u16::<BigEndian>()?;
                let mut subst_glyphs = Vec::with_capacity(glyph_count as usize);
                for _ in 0..glyph_count {
                    subst_glyphs.push(cursor.read_u16::<BigEndian>()?);
                }

                for glyph in coverage.glyphs() {
                    if let Some(index) = coverage.get_index(glyph) {
                        if (index as usize) < subst_glyphs.len() {
                            substitutions.insert(glyph, subst_glyphs[index as usize]);
                        }
                    }
                }
            }
            _ => {
                return Err(ParseError::CorruptedData(format!(
                    "Unknown single substitution format: {}",
                    format
                )));
            }
        }

        Ok(SingleSubst {
            coverage,
            substitutions,
        })
    }

    fn parse_multiple_subst(data: &[u8]) -> Result<MultipleSubst, ParseError> {
        if data.len() < 6 {
            return Err(ParseError::CorruptedData(
                "Multiple substitution subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let _format = cursor.read_u16::<BigEndian>()?; // Should be 1
        let coverage_offset = cursor.read_u16::<BigEndian>()? as usize;
        let sequence_count = cursor.read_u16::<BigEndian>()?;

        let coverage = if coverage_offset < data.len() {
            Coverage::parse(&data[coverage_offset..])?
        } else {
            Coverage::default()
        };

        let mut sequence_offsets = Vec::with_capacity(sequence_count as usize);
        for _ in 0..sequence_count {
            sequence_offsets.push(cursor.read_u16::<BigEndian>()? as usize);
        }

        let mut sequences = HashMap::new();
        for glyph in coverage.glyphs() {
            if let Some(index) = coverage.get_index(glyph) {
                if (index as usize) < sequence_offsets.len() {
                    let seq_offset = sequence_offsets[index as usize];
                    if seq_offset < data.len() {
                        let sequence = Self::parse_sequence(&data[seq_offset..])?;
                        sequences.insert(glyph, sequence);
                    }
                }
            }
        }

        Ok(MultipleSubst {
            coverage,
            sequences,
        })
    }

    fn parse_sequence(data: &[u8]) -> Result<Vec<GlyphId>, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData(
                "Sequence table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let glyph_count = cursor.read_u16::<BigEndian>()?;

        let mut glyphs = Vec::with_capacity(glyph_count as usize);
        for _ in 0..glyph_count {
            glyphs.push(cursor.read_u16::<BigEndian>()?);
        }

        Ok(glyphs)
    }

    fn parse_alternate_subst(data: &[u8]) -> Result<AlternateSubst, ParseError> {
        if data.len() < 6 {
            return Err(ParseError::CorruptedData(
                "Alternate substitution subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let _format = cursor.read_u16::<BigEndian>()?; // Should be 1
        let coverage_offset = cursor.read_u16::<BigEndian>()? as usize;
        let alt_set_count = cursor.read_u16::<BigEndian>()?;

        let coverage = if coverage_offset < data.len() {
            Coverage::parse(&data[coverage_offset..])?
        } else {
            Coverage::default()
        };

        let mut alt_set_offsets = Vec::with_capacity(alt_set_count as usize);
        for _ in 0..alt_set_count {
            alt_set_offsets.push(cursor.read_u16::<BigEndian>()? as usize);
        }

        let mut alternate_sets = HashMap::new();
        for glyph in coverage.glyphs() {
            if let Some(index) = coverage.get_index(glyph) {
                if (index as usize) < alt_set_offsets.len() {
                    let set_offset = alt_set_offsets[index as usize];
                    if set_offset < data.len() {
                        let alternates = Self::parse_sequence(&data[set_offset..])?;
                        alternate_sets.insert(glyph, alternates);
                    }
                }
            }
        }

        Ok(AlternateSubst {
            coverage,
            alternate_sets,
        })
    }

    fn parse_ligature_subst(data: &[u8]) -> Result<LigatureSubst, ParseError> {
        if data.len() < 6 {
            return Err(ParseError::CorruptedData(
                "Ligature substitution subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let _format = cursor.read_u16::<BigEndian>()?; // Should be 1
        let coverage_offset = cursor.read_u16::<BigEndian>()? as usize;
        let lig_set_count = cursor.read_u16::<BigEndian>()?;

        let coverage = if coverage_offset < data.len() {
            Coverage::parse(&data[coverage_offset..])?
        } else {
            Coverage::default()
        };

        let mut lig_set_offsets = Vec::with_capacity(lig_set_count as usize);
        for _ in 0..lig_set_count {
            lig_set_offsets.push(cursor.read_u16::<BigEndian>()? as usize);
        }

        let mut ligature_sets = HashMap::new();
        for glyph in coverage.glyphs() {
            if let Some(index) = coverage.get_index(glyph) {
                if (index as usize) < lig_set_offsets.len() {
                    let set_offset = lig_set_offsets[index as usize];
                    if set_offset < data.len() {
                        let ligatures = Self::parse_ligature_set(&data[set_offset..])?;
                        ligature_sets.insert(glyph, ligatures);
                    }
                }
            }
        }

        Ok(LigatureSubst {
            coverage,
            ligature_sets,
        })
    }

    fn parse_ligature_set(data: &[u8]) -> Result<Vec<Ligature>, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData(
                "Ligature set too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let ligature_count = cursor.read_u16::<BigEndian>()?;

        let mut offsets = Vec::with_capacity(ligature_count as usize);
        for _ in 0..ligature_count {
            offsets.push(cursor.read_u16::<BigEndian>()? as usize);
        }

        let mut ligatures = Vec::with_capacity(ligature_count as usize);
        for offset in offsets {
            if offset < data.len() {
                let ligature = Self::parse_ligature(&data[offset..])?;
                ligatures.push(ligature);
            }
        }

        // Sort by component count (longest first) for proper matching
        ligatures.sort_by(|a, b| b.components.len().cmp(&a.components.len()));

        Ok(ligatures)
    }

    fn parse_ligature(data: &[u8]) -> Result<Ligature, ParseError> {
        if data.len() < 4 {
            return Err(ParseError::CorruptedData("Ligature too short".to_string()));
        }

        let mut cursor = Cursor::new(data);
        let ligature_glyph = cursor.read_u16::<BigEndian>()?;
        let component_count = cursor.read_u16::<BigEndian>()?;

        // Component count includes the first glyph (from coverage)
        let mut components = Vec::with_capacity((component_count.saturating_sub(1)) as usize);
        for _ in 1..component_count {
            components.push(cursor.read_u16::<BigEndian>()?);
        }

        Ok(Ligature {
            ligature_glyph,
            components,
        })
    }

    fn parse_context_subst(data: &[u8]) -> Result<ContextSubst, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData(
                "Context substitution subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let format = cursor.read_u16::<BigEndian>()?;

        Ok(ContextSubst {
            format,
            rule_sets: HashMap::new(),
            class_rules: None,
            coverage_rules: None,
        })
    }

    fn parse_chaining_context_subst(data: &[u8]) -> Result<ChainingContextSubst, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData(
                "Chaining context substitution subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let format = cursor.read_u16::<BigEndian>()?;

        Ok(ChainingContextSubst {
            format,
            rule_sets: HashMap::new(),
            class_rules: None,
            coverage_rules: None,
        })
    }

    /// Get feature indices for a script and language
    pub fn get_feature_indices(&self, script: Tag, language: Option<Tag>) -> Vec<u16> {
        if let Some(script_record) = self.scripts.scripts.get(&script) {
            let lang_sys = match language {
                Some(lang_tag) => script_record.lang_sys.get(&lang_tag),
                None => script_record.default_lang_sys.as_ref(),
            };

            if let Some(ls) = lang_sys {
                return ls.feature_indices.clone();
            }
        }
        Vec::new()
    }

    /// Get lookup indices for a feature
    pub fn get_lookup_indices(&self, feature_tag: Tag) -> Vec<u16> {
        for feature in &self.features.features {
            if feature.tag == feature_tag {
                return feature.lookup_indices.clone();
            }
        }
        Vec::new()
    }

    /// Check if a feature is supported
    pub fn has_feature(&self, feature_tag: Tag) -> bool {
        self.features.features.iter().any(|f| f.tag == feature_tag)
    }

    /// Get all supported features
    pub fn supported_features(&self) -> Vec<Tag> {
        self.features.features.iter().map(|f| f.tag).collect()
    }

    /// Static method to parse script list (for reuse by GPOS)
    pub fn parse_script_list_static(data: &[u8]) -> Result<ScriptList, ParseError> {
        Self::parse_script_list(data)
    }

    /// Static method to parse feature list (for reuse by GPOS)
    pub fn parse_feature_list_static(data: &[u8]) -> Result<FeatureList, ParseError> {
        Self::parse_feature_list(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_type_conversion() {
        assert_eq!(LookupType::try_from(1).unwrap(), LookupType::Single);
        assert_eq!(LookupType::try_from(2).unwrap(), LookupType::Multiple);
        assert_eq!(LookupType::try_from(3).unwrap(), LookupType::Alternate);
        assert_eq!(LookupType::try_from(4).unwrap(), LookupType::Ligature);
        assert_eq!(LookupType::try_from(5).unwrap(), LookupType::Context);
        assert_eq!(
            LookupType::try_from(6).unwrap(),
            LookupType::ChainingContext
        );
        assert!(LookupType::try_from(100).is_err());
    }

    #[test]
    fn test_coverage_format1_parse() {
        // Format 1 coverage: format=1, count=3, glyphs=[10, 20, 30]
        let data = [
            0x00, 0x01, // format
            0x00, 0x03, // glyph count
            0x00, 0x0A, // glyph 10
            0x00, 0x14, // glyph 20
            0x00, 0x1E, // glyph 30
        ];

        let coverage = Coverage::parse(&data).unwrap();
        assert!(coverage.contains(10));
        assert!(coverage.contains(20));
        assert!(coverage.contains(30));
        assert!(!coverage.contains(15));
        assert_eq!(coverage.get_index(10), Some(0));
        assert_eq!(coverage.get_index(20), Some(1));
        assert_eq!(coverage.get_index(30), Some(2));
    }

    #[test]
    fn test_coverage_format2_parse() {
        // Format 2 coverage: format=2, count=1, range=[10-15, start_index=0]
        let data = [
            0x00, 0x02, // format
            0x00, 0x01, // range count
            0x00, 0x0A, // start glyph 10
            0x00, 0x0F, // end glyph 15
            0x00, 0x00, // start coverage index
        ];

        let coverage = Coverage::parse(&data).unwrap();
        assert!(coverage.contains(10));
        assert!(coverage.contains(12));
        assert!(coverage.contains(15));
        assert!(!coverage.contains(9));
        assert!(!coverage.contains(16));
    }

    #[test]
    fn test_class_def_format1_parse() {
        // Format 1 class def: format=1, start=10, count=3, classes=[1, 2, 3]
        let data = [
            0x00, 0x01, // format
            0x00, 0x0A, // start glyph
            0x00, 0x03, // glyph count
            0x00, 0x01, // class 1
            0x00, 0x02, // class 2
            0x00, 0x03, // class 3
        ];

        let class_def = ClassDef::parse(&data).unwrap();
        assert_eq!(class_def.get_class(10), 1);
        assert_eq!(class_def.get_class(11), 2);
        assert_eq!(class_def.get_class(12), 3);
        assert_eq!(class_def.get_class(9), 0); // Not defined
    }

    #[test]
    fn test_lookup_flags_parse() {
        let flags = LookupFlags::parse(0x000F, None);
        assert!(flags.right_to_left);
        assert!(flags.ignore_base_glyphs);
        assert!(flags.ignore_ligatures);
        assert!(flags.ignore_marks);

        let flags = LookupFlags::parse(0x0000, Some(5));
        assert!(!flags.right_to_left);
        assert_eq!(flags.mark_filtering_set, Some(5));
    }

    #[test]
    fn test_single_subst_substitute() {
        let mut glyphs = HashMap::new();
        glyphs.insert(10_u16, 0_u16);
        glyphs.insert(20_u16, 1_u16);

        let coverage = Coverage { glyphs };

        let mut substitutions = HashMap::new();
        substitutions.insert(10_u16, 100_u16);
        substitutions.insert(20_u16, 200_u16);

        let single = SingleSubst {
            coverage,
            substitutions,
        };

        assert_eq!(single.substitute(10), Some(100));
        assert_eq!(single.substitute(20), Some(200));
        assert_eq!(single.substitute(30), None);
    }

    #[test]
    fn test_ligature_match() {
        let mut glyphs = HashMap::new();
        glyphs.insert(10_u16, 0_u16); // 'f' at index 0

        let coverage = Coverage { glyphs };

        let ligature_ff = Ligature {
            ligature_glyph: 100,  // 'ff' ligature
            components: vec![10], // second 'f'
        };

        let ligature_ffi = Ligature {
            ligature_glyph: 101,      // 'ffi' ligature
            components: vec![10, 11], // 'f', 'i'
        };

        let mut ligature_sets = HashMap::new();
        // Note: ffi should come first since it's longer
        ligature_sets.insert(10_u16, vec![ligature_ffi.clone(), ligature_ff.clone()]);

        let lig_subst = LigatureSubst {
            coverage,
            ligature_sets,
        };

        // Test 'ff' match
        let result = lig_subst.match_ligature(&[10, 10]);
        assert_eq!(result, Some((100, 2)));

        // Test 'ffi' match
        let result = lig_subst.match_ligature(&[10, 10, 11]);
        assert_eq!(result, Some((101, 3)));

        // Test no match
        let result = lig_subst.match_ligature(&[20, 10]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_gsub_minimal_parse() {
        // Minimal GSUB header
        let data = [
            0x00, 0x01, // major version
            0x00, 0x00, // minor version
            0x00, 0x00, // script list offset (NULL)
            0x00, 0x00, // feature list offset (NULL)
            0x00, 0x00, // lookup list offset (NULL)
        ];

        let gsub = GsubTable::parse(&data).unwrap();
        assert_eq!(gsub.major_version, 1);
        assert_eq!(gsub.minor_version, 0);
    }

    #[test]
    fn test_gsub_unsupported_version() {
        let data = [
            0x00, 0x02, // major version 2 (unsupported)
            0x00, 0x00, // minor version
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let result = GsubTable::parse(&data);
        assert!(matches!(result, Err(ParseError::UnsupportedVersion)));
    }
}
