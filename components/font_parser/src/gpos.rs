//! GPOS (Glyph Positioning) table parsing
//!
//! The GPOS table provides precise control over glyph placement for
//! accurate typography. This includes kerning, mark positioning, and
//! cursive attachment.

pub use crate::gsub::{ClassDef, Coverage};
use crate::types::{GlyphId, Tag};
use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

/// GPOS table
#[derive(Debug, Clone)]
pub struct GposTable {
    /// Major version of the GPOS table
    pub major_version: u16,
    /// Minor version of the GPOS table
    pub minor_version: u16,
    /// Script list
    pub scripts: ScriptList,
    /// Feature list
    pub features: FeatureList,
    /// Lookup list
    pub lookups: LookupList,
}

/// Script list (reuses same structure as GSUB)
pub type ScriptList = crate::gsub::ScriptList;
/// Feature list (reuses same structure as GSUB)
pub type FeatureList = crate::gsub::FeatureList;

/// Lookup list for GPOS
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
    pub flags: crate::gsub::LookupFlags,
    /// Subtables for this lookup
    pub subtables: Vec<SubtableData>,
}

/// Lookup type for GPOS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupType {
    /// Single adjustment
    SingleAdjustment = 1,
    /// Pair adjustment (kerning)
    PairAdjustment = 2,
    /// Cursive attachment
    CursiveAttachment = 3,
    /// Mark to base attachment
    MarkToBase = 4,
    /// Mark to ligature attachment
    MarkToLigature = 5,
    /// Mark to mark attachment
    MarkToMark = 6,
    /// Context positioning
    Context = 7,
    /// Chaining context positioning
    ChainingContext = 8,
    /// Extension positioning
    Extension = 9,
}

impl TryFrom<u16> for LookupType {
    type Error = ParseError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(LookupType::SingleAdjustment),
            2 => Ok(LookupType::PairAdjustment),
            3 => Ok(LookupType::CursiveAttachment),
            4 => Ok(LookupType::MarkToBase),
            5 => Ok(LookupType::MarkToLigature),
            6 => Ok(LookupType::MarkToMark),
            7 => Ok(LookupType::Context),
            8 => Ok(LookupType::ChainingContext),
            9 => Ok(LookupType::Extension),
            _ => Err(ParseError::CorruptedData(format!(
                "Invalid GPOS lookup type: {}",
                value
            ))),
        }
    }
}

/// Subtable data for different lookup types
#[derive(Debug, Clone)]
pub enum SubtableData {
    /// Single adjustment
    SingleAdjustment(SingleAdjustmentSubtable),
    /// Pair adjustment (kerning)
    PairAdjustment(PairAdjustmentSubtable),
    /// Mark to base attachment
    MarkToBase(MarkToBaseSubtable),
    /// Raw data for unimplemented types
    Raw(Vec<u8>),
}

/// Value record for positioning adjustments
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ValueRecord {
    /// Horizontal adjustment for placement
    pub x_placement: i16,
    /// Vertical adjustment for placement
    pub y_placement: i16,
    /// Horizontal adjustment for advance
    pub x_advance: i16,
    /// Vertical adjustment for advance
    pub y_advance: i16,
}

impl ValueRecord {
    /// Check if the value record is empty (no adjustments)
    pub fn is_empty(&self) -> bool {
        self.x_placement == 0
            && self.y_placement == 0
            && self.x_advance == 0
            && self.y_advance == 0
    }

    fn parse(cursor: &mut Cursor<&[u8]>, format: u16) -> Result<Self, ParseError> {
        let x_placement = if format & 0x0001 != 0 {
            cursor.read_i16::<BigEndian>()?
        } else {
            0
        };

        let y_placement = if format & 0x0002 != 0 {
            cursor.read_i16::<BigEndian>()?
        } else {
            0
        };

        let x_advance = if format & 0x0004 != 0 {
            cursor.read_i16::<BigEndian>()?
        } else {
            0
        };

        let y_advance = if format & 0x0008 != 0 {
            cursor.read_i16::<BigEndian>()?
        } else {
            0
        };

        // Skip device table offsets (not implemented)
        if format & 0x0010 != 0 {
            cursor.read_u16::<BigEndian>()?;
        }
        if format & 0x0020 != 0 {
            cursor.read_u16::<BigEndian>()?;
        }
        if format & 0x0040 != 0 {
            cursor.read_u16::<BigEndian>()?;
        }
        if format & 0x0080 != 0 {
            cursor.read_u16::<BigEndian>()?;
        }

        Ok(ValueRecord {
            x_placement,
            y_placement,
            x_advance,
            y_advance,
        })
    }

    fn size_from_format(format: u16) -> usize {
        let mut size = 0;
        if format & 0x0001 != 0 {
            size += 2;
        }
        if format & 0x0002 != 0 {
            size += 2;
        }
        if format & 0x0004 != 0 {
            size += 2;
        }
        if format & 0x0008 != 0 {
            size += 2;
        }
        if format & 0x0010 != 0 {
            size += 2;
        }
        if format & 0x0020 != 0 {
            size += 2;
        }
        if format & 0x0040 != 0 {
            size += 2;
        }
        if format & 0x0080 != 0 {
            size += 2;
        }
        size
    }
}

/// Single adjustment subtable
#[derive(Debug, Clone)]
pub struct SingleAdjustmentSubtable {
    /// Coverage table
    pub coverage: Coverage,
    /// Value format
    pub value_format: u16,
    /// Single value (format 1) or per-glyph values (format 2)
    pub values: SingleAdjustmentValues,
}

/// Values for single adjustment
#[derive(Debug, Clone)]
pub enum SingleAdjustmentValues {
    /// Format 1: single value for all covered glyphs
    Single(ValueRecord),
    /// Format 2: per-glyph values
    Array(Vec<ValueRecord>),
}

impl SingleAdjustmentSubtable {
    /// Get adjustment for a glyph
    pub fn get_adjustment(&self, glyph: GlyphId) -> Option<ValueRecord> {
        let index = self.coverage.get_index(glyph)?;

        match &self.values {
            SingleAdjustmentValues::Single(value) => Some(*value),
            SingleAdjustmentValues::Array(values) => values.get(index as usize).copied(),
        }
    }
}

/// Pair adjustment subtable (kerning)
#[derive(Debug, Clone)]
pub struct PairAdjustmentSubtable {
    /// Coverage table (first glyph)
    pub coverage: Coverage,
    /// Format (1 = glyph pairs, 2 = class pairs)
    pub format: u16,
    /// Pair sets for format 1
    pub pair_sets: HashMap<GlyphId, Vec<PairValueRecord>>,
    /// Class-based kerning for format 2
    pub class_kerning: Option<ClassKerning>,
}

/// A pair value record
#[derive(Debug, Clone, Copy)]
pub struct PairValueRecord {
    /// Second glyph in the pair
    pub second_glyph: GlyphId,
    /// Adjustment for first glyph
    pub value1: ValueRecord,
    /// Adjustment for second glyph
    pub value2: ValueRecord,
}

/// Class-based kerning
#[derive(Debug, Clone)]
pub struct ClassKerning {
    /// Class definition for first glyphs
    pub class_def1: ClassDef,
    /// Class definition for second glyphs
    pub class_def2: ClassDef,
    /// Class 1 count
    pub class1_count: u16,
    /// Class 2 count
    pub class2_count: u16,
    /// Class pair records [class1][class2]
    pub records: Vec<Vec<(ValueRecord, ValueRecord)>>,
}

impl PairAdjustmentSubtable {
    /// Get kerning for a glyph pair
    pub fn get_kerning(&self, first: GlyphId, second: GlyphId) -> Option<(ValueRecord, ValueRecord)> {
        if !self.coverage.contains(first) {
            return None;
        }

        match self.format {
            1 => {
                // Format 1: direct glyph pair lookup
                if let Some(pairs) = self.pair_sets.get(&first) {
                    for pair in pairs {
                        if pair.second_glyph == second {
                            return Some((pair.value1, pair.value2));
                        }
                    }
                }
                None
            }
            2 => {
                // Format 2: class-based lookup
                if let Some(ref class_kerning) = self.class_kerning {
                    let class1 = class_kerning.class_def1.get_class(first) as usize;
                    let class2 = class_kerning.class_def2.get_class(second) as usize;

                    if class1 < class_kerning.records.len() {
                        if class2 < class_kerning.records[class1].len() {
                            return Some(class_kerning.records[class1][class2]);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}

/// Mark to base attachment subtable
#[derive(Debug, Clone)]
pub struct MarkToBaseSubtable {
    /// Mark coverage
    pub mark_coverage: Coverage,
    /// Base coverage
    pub base_coverage: Coverage,
    /// Mark class count
    pub mark_class_count: u16,
    /// Mark array
    pub mark_array: Vec<MarkRecord>,
    /// Base array
    pub base_array: Vec<Vec<AnchorPoint>>,
}

/// A mark record
#[derive(Debug, Clone, Copy)]
pub struct MarkRecord {
    /// Mark class
    pub mark_class: u16,
    /// Anchor point for the mark
    pub anchor: AnchorPoint,
}

/// An anchor point
#[derive(Debug, Clone, Copy, Default)]
pub struct AnchorPoint {
    /// X coordinate
    pub x: i16,
    /// Y coordinate
    pub y: i16,
}

impl GposTable {
    /// Parse GPOS table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 10 {
            return Err(ParseError::CorruptedData(
                "GPOS table too short".to_string(),
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

        // Parse script list (reuse GSUB parser)
        let scripts = if script_list_offset > 0 && script_list_offset < data.len() {
            crate::gsub::GsubTable::parse_script_list_static(&data[script_list_offset..])?
        } else {
            ScriptList::default()
        };

        // Parse feature list (reuse GSUB parser)
        let features = if feature_list_offset > 0 && feature_list_offset < data.len() {
            crate::gsub::GsubTable::parse_feature_list_static(&data[feature_list_offset..])?
        } else {
            FeatureList::default()
        };

        // Parse lookup list
        let lookups = if lookup_list_offset > 0 && lookup_list_offset < data.len() {
            Self::parse_lookup_list(&data[lookup_list_offset..])?
        } else {
            LookupList::default()
        };

        Ok(GposTable {
            major_version,
            minor_version,
            scripts,
            features,
            lookups,
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
                let lookup = Self::parse_lookup(&data[offset..])?;
                lookups.push(lookup);
            }
        }

        Ok(LookupList { lookups })
    }

    fn parse_lookup(data: &[u8]) -> Result<Lookup, ParseError> {
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

        let flags = crate::gsub::LookupFlags::parse(lookup_flags, mark_filtering_set);
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
            LookupType::SingleAdjustment => Ok(SubtableData::SingleAdjustment(
                Self::parse_single_adjustment(data)?,
            )),
            LookupType::PairAdjustment => Ok(SubtableData::PairAdjustment(
                Self::parse_pair_adjustment(data)?,
            )),
            LookupType::MarkToBase => {
                Ok(SubtableData::MarkToBase(Self::parse_mark_to_base(data)?))
            }
            _ => Ok(SubtableData::Raw(data.to_vec())),
        }
    }

    fn parse_single_adjustment(data: &[u8]) -> Result<SingleAdjustmentSubtable, ParseError> {
        if data.len() < 6 {
            return Err(ParseError::CorruptedData(
                "Single adjustment subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let format = cursor.read_u16::<BigEndian>()?;
        let coverage_offset = cursor.read_u16::<BigEndian>()? as usize;
        let value_format = cursor.read_u16::<BigEndian>()?;

        let coverage = if coverage_offset < data.len() {
            Coverage::parse(&data[coverage_offset..])?
        } else {
            Coverage::default()
        };

        let values = match format {
            1 => {
                let value = ValueRecord::parse(&mut cursor, value_format)?;
                SingleAdjustmentValues::Single(value)
            }
            2 => {
                let value_count = cursor.read_u16::<BigEndian>()?;
                let mut values = Vec::with_capacity(value_count as usize);
                for _ in 0..value_count {
                    values.push(ValueRecord::parse(&mut cursor, value_format)?);
                }
                SingleAdjustmentValues::Array(values)
            }
            _ => {
                return Err(ParseError::CorruptedData(format!(
                    "Unknown single adjustment format: {}",
                    format
                )));
            }
        };

        Ok(SingleAdjustmentSubtable {
            coverage,
            value_format,
            values,
        })
    }

    fn parse_pair_adjustment(data: &[u8]) -> Result<PairAdjustmentSubtable, ParseError> {
        if data.len() < 10 {
            return Err(ParseError::CorruptedData(
                "Pair adjustment subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let format = cursor.read_u16::<BigEndian>()?;
        let coverage_offset = cursor.read_u16::<BigEndian>()? as usize;
        let value_format1 = cursor.read_u16::<BigEndian>()?;
        let value_format2 = cursor.read_u16::<BigEndian>()?;

        let coverage = if coverage_offset < data.len() {
            Coverage::parse(&data[coverage_offset..])?
        } else {
            Coverage::default()
        };

        let mut pair_sets = HashMap::new();
        let mut class_kerning = None;

        match format {
            1 => {
                // Format 1: individual glyph pairs
                let pair_set_count = cursor.read_u16::<BigEndian>()?;
                let mut pair_set_offsets = Vec::with_capacity(pair_set_count as usize);
                for _ in 0..pair_set_count {
                    pair_set_offsets.push(cursor.read_u16::<BigEndian>()? as usize);
                }

                for glyph in coverage.glyphs() {
                    if let Some(index) = coverage.get_index(glyph) {
                        if (index as usize) < pair_set_offsets.len() {
                            let offset = pair_set_offsets[index as usize];
                            if offset < data.len() {
                                let pairs = Self::parse_pair_set(
                                    &data[offset..],
                                    value_format1,
                                    value_format2,
                                )?;
                                pair_sets.insert(glyph, pairs);
                            }
                        }
                    }
                }
            }
            2 => {
                // Format 2: class-based kerning
                let class_def1_offset = cursor.read_u16::<BigEndian>()? as usize;
                let class_def2_offset = cursor.read_u16::<BigEndian>()? as usize;
                let class1_count = cursor.read_u16::<BigEndian>()?;
                let class2_count = cursor.read_u16::<BigEndian>()?;

                let class_def1 = if class_def1_offset < data.len() {
                    ClassDef::parse(&data[class_def1_offset..])?
                } else {
                    ClassDef::default()
                };

                let class_def2 = if class_def2_offset < data.len() {
                    ClassDef::parse(&data[class_def2_offset..])?
                } else {
                    ClassDef::default()
                };

                let value1_size = ValueRecord::size_from_format(value_format1);
                let value2_size = ValueRecord::size_from_format(value_format2);
                let record_size = value1_size + value2_size;

                let mut records = Vec::with_capacity(class1_count as usize);
                for _ in 0..class1_count {
                    let mut class2_records = Vec::with_capacity(class2_count as usize);
                    for _ in 0..class2_count {
                        let value1 = ValueRecord::parse(&mut cursor, value_format1)?;
                        let value2 = ValueRecord::parse(&mut cursor, value_format2)?;
                        class2_records.push((value1, value2));
                    }
                    records.push(class2_records);
                }

                class_kerning = Some(ClassKerning {
                    class_def1,
                    class_def2,
                    class1_count,
                    class2_count,
                    records,
                });
            }
            _ => {
                return Err(ParseError::CorruptedData(format!(
                    "Unknown pair adjustment format: {}",
                    format
                )));
            }
        }

        Ok(PairAdjustmentSubtable {
            coverage,
            format,
            pair_sets,
            class_kerning,
        })
    }

    fn parse_pair_set(
        data: &[u8],
        value_format1: u16,
        value_format2: u16,
    ) -> Result<Vec<PairValueRecord>, ParseError> {
        if data.len() < 2 {
            return Err(ParseError::CorruptedData("Pair set too short".to_string()));
        }

        let mut cursor = Cursor::new(data);
        let pair_count = cursor.read_u16::<BigEndian>()?;

        let mut pairs = Vec::with_capacity(pair_count as usize);
        for _ in 0..pair_count {
            let second_glyph = cursor.read_u16::<BigEndian>()?;
            let value1 = ValueRecord::parse(&mut cursor, value_format1)?;
            let value2 = ValueRecord::parse(&mut cursor, value_format2)?;

            pairs.push(PairValueRecord {
                second_glyph,
                value1,
                value2,
            });
        }

        Ok(pairs)
    }

    fn parse_mark_to_base(data: &[u8]) -> Result<MarkToBaseSubtable, ParseError> {
        if data.len() < 12 {
            return Err(ParseError::CorruptedData(
                "Mark to base subtable too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);
        let _format = cursor.read_u16::<BigEndian>()?; // Should be 1
        let mark_coverage_offset = cursor.read_u16::<BigEndian>()? as usize;
        let base_coverage_offset = cursor.read_u16::<BigEndian>()? as usize;
        let mark_class_count = cursor.read_u16::<BigEndian>()?;
        let _mark_array_offset = cursor.read_u16::<BigEndian>()?;
        let _base_array_offset = cursor.read_u16::<BigEndian>()?;

        let mark_coverage = if mark_coverage_offset < data.len() {
            Coverage::parse(&data[mark_coverage_offset..])?
        } else {
            Coverage::default()
        };

        let base_coverage = if base_coverage_offset < data.len() {
            Coverage::parse(&data[base_coverage_offset..])?
        } else {
            Coverage::default()
        };

        Ok(MarkToBaseSubtable {
            mark_coverage,
            base_coverage,
            mark_class_count,
            mark_array: Vec::new(),
            base_array: Vec::new(),
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

    /// Get kerning for a glyph pair from all pair adjustment lookups
    pub fn get_kerning(&self, first: GlyphId, second: GlyphId) -> Option<i16> {
        for lookup in &self.lookups.lookups {
            if lookup.lookup_type == LookupType::PairAdjustment {
                for subtable in &lookup.subtables {
                    if let SubtableData::PairAdjustment(pair_adj) = subtable {
                        if let Some((value1, _)) = pair_adj.get_kerning(first, second) {
                            // Return x_advance from first glyph (most common kerning)
                            if value1.x_advance != 0 {
                                return Some(value1.x_advance);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_type_conversion() {
        assert_eq!(
            LookupType::try_from(1).unwrap(),
            LookupType::SingleAdjustment
        );
        assert_eq!(
            LookupType::try_from(2).unwrap(),
            LookupType::PairAdjustment
        );
        assert_eq!(
            LookupType::try_from(4).unwrap(),
            LookupType::MarkToBase
        );
        assert!(LookupType::try_from(100).is_err());
    }

    #[test]
    fn test_value_record_default() {
        let vr = ValueRecord::default();
        assert!(vr.is_empty());
        assert_eq!(vr.x_placement, 0);
        assert_eq!(vr.y_placement, 0);
        assert_eq!(vr.x_advance, 0);
        assert_eq!(vr.y_advance, 0);
    }

    #[test]
    fn test_value_record_parse() {
        // Value record with x_placement and x_advance
        let data = [
            0x00, 0x10, // x_placement = 16
            0xFF, 0xF0, // x_advance = -16
        ];

        let mut cursor = Cursor::new(&data[..]);
        let format = 0x0005; // x_placement (0x0001) + x_advance (0x0004)
        let vr = ValueRecord::parse(&mut cursor, format).unwrap();

        assert_eq!(vr.x_placement, 16);
        assert_eq!(vr.y_placement, 0);
        assert_eq!(vr.x_advance, -16);
        assert_eq!(vr.y_advance, 0);
    }

    #[test]
    fn test_value_record_size() {
        assert_eq!(ValueRecord::size_from_format(0x0000), 0);
        assert_eq!(ValueRecord::size_from_format(0x0001), 2);
        assert_eq!(ValueRecord::size_from_format(0x0003), 4);
        assert_eq!(ValueRecord::size_from_format(0x000F), 8);
        assert_eq!(ValueRecord::size_from_format(0x00FF), 16);
    }

    #[test]
    fn test_single_adjustment_get() {
        let mut glyphs = HashMap::new();
        glyphs.insert(10_u16, 0_u16);
        let coverage = Coverage { glyphs };

        let value = ValueRecord {
            x_placement: 0,
            y_placement: 0,
            x_advance: -50,
            y_advance: 0,
        };

        let subtable = SingleAdjustmentSubtable {
            coverage,
            value_format: 0x0004,
            values: SingleAdjustmentValues::Single(value),
        };

        let adj = subtable.get_adjustment(10);
        assert!(adj.is_some());
        assert_eq!(adj.unwrap().x_advance, -50);

        let adj = subtable.get_adjustment(20);
        assert!(adj.is_none());
    }

    #[test]
    fn test_pair_adjustment_format1() {
        let mut glyphs = HashMap::new();
        glyphs.insert(10_u16, 0_u16);
        let coverage = Coverage { glyphs };

        let pairs = vec![PairValueRecord {
            second_glyph: 20,
            value1: ValueRecord {
                x_placement: 0,
                y_placement: 0,
                x_advance: -100,
                y_advance: 0,
            },
            value2: ValueRecord::default(),
        }];

        let mut pair_sets = HashMap::new();
        pair_sets.insert(10_u16, pairs);

        let subtable = PairAdjustmentSubtable {
            coverage,
            format: 1,
            pair_sets,
            class_kerning: None,
        };

        let kern = subtable.get_kerning(10, 20);
        assert!(kern.is_some());
        assert_eq!(kern.unwrap().0.x_advance, -100);

        let kern = subtable.get_kerning(10, 30);
        assert!(kern.is_none());
    }

    #[test]
    fn test_gpos_minimal_parse() {
        // Minimal GPOS header
        let data = [
            0x00, 0x01, // major version
            0x00, 0x00, // minor version
            0x00, 0x00, // script list offset (NULL)
            0x00, 0x00, // feature list offset (NULL)
            0x00, 0x00, // lookup list offset (NULL)
        ];

        let gpos = GposTable::parse(&data).unwrap();
        assert_eq!(gpos.major_version, 1);
        assert_eq!(gpos.minor_version, 0);
    }

    #[test]
    fn test_gpos_unsupported_version() {
        let data = [
            0x00, 0x02, // major version 2 (unsupported)
            0x00, 0x00, // minor version
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let result = GposTable::parse(&data);
        assert!(matches!(result, Err(ParseError::UnsupportedVersion)));
    }
}
