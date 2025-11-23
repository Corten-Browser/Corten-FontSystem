//! CFF/CFF2 - Compact Font Format parsing
//!
//! This module provides parsing for PostScript outline data stored in OpenType fonts.
//! CFF (Compact Font Format) provides an efficient representation of PostScript outlines
//! and is used by OpenType/CFF fonts (with 'OTTO' signature).

use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// CFF Table - Compact Font Format
#[derive(Debug, Clone)]
pub struct CffTable {
    /// CFF version (major)
    pub major_version: u8,
    /// CFF version (minor)
    pub minor_version: u8,
    /// Font names in the Name INDEX
    pub font_names: Vec<String>,
    /// Top Dict for each font
    pub top_dicts: Vec<TopDict>,
    /// Global subroutines
    pub global_subrs: Vec<Vec<u8>>,
    /// CharStrings INDEX for each font
    pub charstrings: Vec<Vec<Vec<u8>>>,
    /// Private Dict for each font
    pub private_dicts: Vec<PrivateDict>,
}

/// Top DICT contains font-wide metadata
#[derive(Debug, Clone, Default)]
pub struct TopDict {
    /// Version string SID
    pub version: Option<u16>,
    /// Notice string SID
    pub notice: Option<u16>,
    /// Full name SID
    pub full_name: Option<u16>,
    /// Family name SID
    pub family_name: Option<u16>,
    /// Weight string SID
    pub weight: Option<u16>,
    /// Is fixed pitch
    pub is_fixed_pitch: bool,
    /// Italic angle
    pub italic_angle: f64,
    /// Underline position
    pub underline_position: f64,
    /// Underline thickness
    pub underline_thickness: f64,
    /// Char string type (1 or 2)
    pub charstring_type: i32,
    /// Font matrix
    pub font_matrix: [f64; 6],
    /// Font bounding box
    pub font_bbox: [f64; 4],
    /// Unique ID
    pub unique_id: Option<i32>,
    /// Charset offset
    pub charset: u32,
    /// Encoding offset
    pub encoding: u32,
    /// CharStrings offset
    pub charstrings_offset: u32,
    /// Private DICT size and offset
    pub private: Option<(u32, u32)>,
    /// CID font specific fields
    pub ros: Option<(u16, u16, u16)>, // Registry, Ordering, Supplement
    /// CIDFont version
    pub cid_font_version: Option<f64>,
    /// CIDCount
    pub cid_count: Option<u32>,
    /// FDArray offset (CID fonts)
    pub fd_array: Option<u32>,
    /// FDSelect offset (CID fonts)
    pub fd_select: Option<u32>,
}

/// Private DICT contains font-wide private data
#[derive(Debug, Clone, Default)]
pub struct PrivateDict {
    /// Blue values for hinting
    pub blue_values: Vec<f64>,
    /// Other blues
    pub other_blues: Vec<f64>,
    /// Family blues
    pub family_blues: Vec<f64>,
    /// Family other blues
    pub family_other_blues: Vec<f64>,
    /// Blue scale
    pub blue_scale: f64,
    /// Blue shift
    pub blue_shift: f64,
    /// Blue fuzz
    pub blue_fuzz: f64,
    /// Standard horizontal width
    pub std_hw: Option<f64>,
    /// Standard vertical width
    pub std_vw: Option<f64>,
    /// Stem snap horizontal widths
    pub stem_snap_h: Vec<f64>,
    /// Stem snap vertical widths
    pub stem_snap_v: Vec<f64>,
    /// Force bold
    pub force_bold: bool,
    /// Language group
    pub language_group: i32,
    /// Expansion factor
    pub expansion_factor: f64,
    /// Initial random seed
    pub initial_random_seed: f64,
    /// Local subroutines offset
    pub subrs_offset: Option<u32>,
    /// Default width X
    pub default_width_x: f64,
    /// Nominal width X
    pub nominal_width_x: f64,
}

/// CFF2 Table - Compact Font Format version 2
#[derive(Debug, Clone)]
pub struct Cff2Table {
    /// Major version (should be 2)
    pub major_version: u8,
    /// Minor version
    pub minor_version: u8,
    /// Top Dict data
    pub top_dict: Cff2TopDict,
    /// Global subroutines
    pub global_subrs: Vec<Vec<u8>>,
    /// CharStrings INDEX
    pub charstrings: Vec<Vec<u8>>,
    /// Font DICT INDEX for variable fonts
    pub font_dicts: Vec<Cff2FontDict>,
    /// Item Variation Store (for variable fonts)
    pub variation_store: Option<ItemVariationStore>,
}

/// CFF2 Top DICT
#[derive(Debug, Clone, Default)]
pub struct Cff2TopDict {
    /// CharStrings offset
    pub charstrings_offset: u32,
    /// Font DICT INDEX offset
    pub fd_array_offset: Option<u32>,
    /// FDSelect offset
    pub fd_select_offset: Option<u32>,
    /// Variation Store offset
    pub vstore_offset: Option<u32>,
}

/// CFF2 Font DICT
#[derive(Debug, Clone, Default)]
pub struct Cff2FontDict {
    /// Private DICT size and offset
    pub private: Option<(u32, u32)>,
}

/// Item Variation Store for variable fonts
#[derive(Debug, Clone, Default)]
pub struct ItemVariationStore {
    /// Format (should be 1)
    pub format: u16,
    /// Item variation data subtables
    pub item_variation_data: Vec<ItemVariationData>,
}

/// Item Variation Data
#[derive(Debug, Clone, Default)]
pub struct ItemVariationData {
    /// Item count
    pub item_count: u16,
    /// Region indices
    pub region_indices: Vec<u16>,
    /// Delta sets
    pub delta_sets: Vec<Vec<i16>>,
}

/// CharString command for PostScript outlines
#[derive(Debug, Clone, PartialEq)]
pub enum CharStringCommand {
    /// MoveTo command
    MoveTo(f64, f64),
    /// LineTo command
    LineTo(f64, f64),
    /// CurveTo command (cubic bezier)
    CurveTo(f64, f64, f64, f64, f64, f64),
    /// End of charstring
    EndChar,
    /// Hint mask
    HintMask(Vec<u8>),
    /// Stem hints
    HStem(f64, f64),
    /// Vertical stem hints
    VStem(f64, f64),
}

impl CffTable {
    /// Parse CFF table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 4 {
            return Err(ParseError::CorruptedData(
                "CFF data too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read CFF header
        let major_version = cursor.read_u8()?;
        let minor_version = cursor.read_u8()?;
        let _hdr_size = cursor.read_u8()?;
        let _off_size = cursor.read_u8()?;

        if major_version != 1 {
            return Err(ParseError::UnsupportedVersion);
        }

        // Parse Name INDEX
        let font_names = Self::parse_name_index(data, &mut cursor)?;

        // Parse Top DICT INDEX
        let top_dict_data = Self::parse_index(data, &mut cursor)?;
        let top_dicts = Self::parse_top_dicts(&top_dict_data)?;

        // Parse String INDEX
        let _strings = Self::parse_index(data, &mut cursor)?;

        // Parse Global Subr INDEX
        let global_subrs = Self::parse_index(data, &mut cursor)?;

        // Parse CharStrings and Private DICTs for each font
        let mut charstrings = Vec::new();
        let mut private_dicts = Vec::new();

        for top_dict in &top_dicts {
            // Parse CharStrings INDEX
            if top_dict.charstrings_offset > 0 {
                let mut cs_cursor = Cursor::new(data);
                cs_cursor.set_position(top_dict.charstrings_offset as u64);
                let cs = Self::parse_index(data, &mut cs_cursor)?;
                charstrings.push(cs);
            } else {
                charstrings.push(Vec::new());
            }

            // Parse Private DICT
            if let Some((size, offset)) = top_dict.private {
                let private_data = &data[offset as usize..(offset + size) as usize];
                let private_dict = Self::parse_private_dict(private_data)?;
                private_dicts.push(private_dict);
            } else {
                private_dicts.push(PrivateDict::default());
            }
        }

        Ok(CffTable {
            major_version,
            minor_version,
            font_names,
            top_dicts,
            global_subrs,
            charstrings,
            private_dicts,
        })
    }

    /// Parse Name INDEX
    fn parse_name_index(data: &[u8], cursor: &mut Cursor<&[u8]>) -> Result<Vec<String>, ParseError> {
        let index_data = Self::parse_index(data, cursor)?;
        let mut names = Vec::new();
        for name_data in index_data {
            let name = String::from_utf8_lossy(&name_data).to_string();
            names.push(name);
        }
        Ok(names)
    }

    /// Parse INDEX structure (common in CFF)
    fn parse_index(data: &[u8], cursor: &mut Cursor<&[u8]>) -> Result<Vec<Vec<u8>>, ParseError> {
        let count = cursor.read_u16::<BigEndian>()?;
        if count == 0 {
            return Ok(Vec::new());
        }

        let off_size = cursor.read_u8()?;
        if off_size == 0 || off_size > 4 {
            return Err(ParseError::CorruptedData(
                "Invalid CFF INDEX off_size".to_string(),
            ));
        }

        // Read offsets
        let mut offsets = Vec::new();
        for _ in 0..=count {
            let offset = Self::read_offset(cursor, off_size)?;
            offsets.push(offset);
        }

        // Data starts at current position
        let data_start = cursor.position() as usize;

        // Extract data items
        let mut items = Vec::new();
        for i in 0..count as usize {
            let start = data_start + offsets[i] as usize - 1;
            let end = data_start + offsets[i + 1] as usize - 1;
            if end > data.len() {
                return Err(ParseError::CorruptedData(
                    "CFF INDEX data extends past buffer".to_string(),
                ));
            }
            items.push(data[start..end].to_vec());
        }

        // Move cursor past data
        let last_offset = offsets[count as usize] as usize;
        cursor.set_position((data_start + last_offset - 1) as u64);

        Ok(items)
    }

    /// Read an offset value of variable size
    fn read_offset(cursor: &mut Cursor<&[u8]>, off_size: u8) -> Result<u32, ParseError> {
        let mut value: u32 = 0;
        for _ in 0..off_size {
            value = (value << 8) | cursor.read_u8()? as u32;
        }
        Ok(value)
    }

    /// Parse Top DICTs from index data
    fn parse_top_dicts(dict_data: &[Vec<u8>]) -> Result<Vec<TopDict>, ParseError> {
        let mut dicts = Vec::new();
        for data in dict_data {
            let dict = Self::parse_dict(data)?;
            dicts.push(dict);
        }
        Ok(dicts)
    }

    /// Parse a DICT structure
    fn parse_dict(data: &[u8]) -> Result<TopDict, ParseError> {
        let mut dict = TopDict {
            charstring_type: 2, // Default is Type 2
            font_matrix: [0.001, 0.0, 0.0, 0.001, 0.0, 0.0],
            font_bbox: [0.0, 0.0, 0.0, 0.0],
            ..Default::default()
        };

        let mut operands: Vec<f64> = Vec::new();
        let mut cursor = Cursor::new(data);

        while cursor.position() < data.len() as u64 {
            let b0 = cursor.read_u8()?;

            match b0 {
                // Operators (0-21)
                0 => {
                    // version
                    if let Some(&sid) = operands.last() {
                        dict.version = Some(sid as u16);
                    }
                    operands.clear();
                }
                1 => {
                    // Notice
                    if let Some(&sid) = operands.last() {
                        dict.notice = Some(sid as u16);
                    }
                    operands.clear();
                }
                2 => {
                    // FullName
                    if let Some(&sid) = operands.last() {
                        dict.full_name = Some(sid as u16);
                    }
                    operands.clear();
                }
                3 => {
                    // FamilyName
                    if let Some(&sid) = operands.last() {
                        dict.family_name = Some(sid as u16);
                    }
                    operands.clear();
                }
                4 => {
                    // Weight
                    if let Some(&sid) = operands.last() {
                        dict.weight = Some(sid as u16);
                    }
                    operands.clear();
                }
                5 => {
                    // FontBBox
                    if operands.len() >= 4 {
                        dict.font_bbox = [operands[0], operands[1], operands[2], operands[3]];
                    }
                    operands.clear();
                }
                12 => {
                    // Two-byte operator
                    let b1 = cursor.read_u8()?;
                    match b1 {
                        0 => {
                            // Copyright
                            operands.clear();
                        }
                        1 => {
                            // isFixedPitch
                            dict.is_fixed_pitch = operands.last().copied().unwrap_or(0.0) != 0.0;
                            operands.clear();
                        }
                        2 => {
                            // ItalicAngle
                            dict.italic_angle = operands.last().copied().unwrap_or(0.0);
                            operands.clear();
                        }
                        3 => {
                            // UnderlinePosition
                            dict.underline_position = operands.last().copied().unwrap_or(-100.0);
                            operands.clear();
                        }
                        4 => {
                            // UnderlineThickness
                            dict.underline_thickness = operands.last().copied().unwrap_or(50.0);
                            operands.clear();
                        }
                        6 => {
                            // CharstringType
                            dict.charstring_type = operands.last().copied().unwrap_or(2.0) as i32;
                            operands.clear();
                        }
                        7 => {
                            // FontMatrix
                            if operands.len() >= 6 {
                                dict.font_matrix = [
                                    operands[0],
                                    operands[1],
                                    operands[2],
                                    operands[3],
                                    operands[4],
                                    operands[5],
                                ];
                            }
                            operands.clear();
                        }
                        30 => {
                            // ROS (CID fonts)
                            if operands.len() >= 3 {
                                dict.ros = Some((
                                    operands[0] as u16,
                                    operands[1] as u16,
                                    operands[2] as u16,
                                ));
                            }
                            operands.clear();
                        }
                        31 => {
                            // CIDFontVersion
                            dict.cid_font_version = operands.last().copied();
                            operands.clear();
                        }
                        34 => {
                            // CIDCount
                            dict.cid_count = operands.last().map(|v| *v as u32);
                            operands.clear();
                        }
                        36 => {
                            // FDArray
                            dict.fd_array = operands.last().map(|v| *v as u32);
                            operands.clear();
                        }
                        37 => {
                            // FDSelect
                            dict.fd_select = operands.last().map(|v| *v as u32);
                            operands.clear();
                        }
                        _ => {
                            operands.clear();
                        }
                    }
                }
                13 => {
                    // UniqueID
                    dict.unique_id = operands.last().map(|v| *v as i32);
                    operands.clear();
                }
                15 => {
                    // charset
                    dict.charset = operands.last().copied().unwrap_or(0.0) as u32;
                    operands.clear();
                }
                16 => {
                    // Encoding
                    dict.encoding = operands.last().copied().unwrap_or(0.0) as u32;
                    operands.clear();
                }
                17 => {
                    // CharStrings
                    dict.charstrings_offset = operands.last().copied().unwrap_or(0.0) as u32;
                    operands.clear();
                }
                18 => {
                    // Private
                    if operands.len() >= 2 {
                        dict.private = Some((operands[0] as u32, operands[1] as u32));
                    }
                    operands.clear();
                }
                // Number operands
                28 => {
                    // 16-bit signed integer
                    let b1 = cursor.read_u8()?;
                    let b2 = cursor.read_u8()?;
                    let value = ((b1 as i16) << 8) | (b2 as i16);
                    operands.push(value as f64);
                }
                29 => {
                    // 32-bit signed integer
                    let value = cursor.read_i32::<BigEndian>()?;
                    operands.push(value as f64);
                }
                30 => {
                    // Real number
                    let real = Self::parse_real(&mut cursor)?;
                    operands.push(real);
                }
                32..=246 => {
                    // Small integer
                    operands.push((b0 as i32 - 139) as f64);
                }
                247..=250 => {
                    // Positive integer
                    let b1 = cursor.read_u8()?;
                    let value = ((b0 as i32 - 247) * 256) + b1 as i32 + 108;
                    operands.push(value as f64);
                }
                251..=254 => {
                    // Negative integer
                    let b1 = cursor.read_u8()?;
                    let value = -((b0 as i32 - 251) * 256) - b1 as i32 - 108;
                    operands.push(value as f64);
                }
                _ => {
                    // Unknown operator, skip operands
                    operands.clear();
                }
            }
        }

        Ok(dict)
    }

    /// Parse a real number in CFF format
    fn parse_real(cursor: &mut Cursor<&[u8]>) -> Result<f64, ParseError> {
        let mut s = String::new();
        let mut done = false;

        while !done {
            let b = cursor.read_u8()?;
            for nibble in [b >> 4, b & 0x0F] {
                match nibble {
                    0..=9 => s.push((b'0' + nibble) as char),
                    0xa => s.push('.'),
                    0xb => s.push('E'),
                    0xc => {
                        s.push('E');
                        s.push('-');
                    }
                    0xe => s.push('-'),
                    0xf => {
                        done = true;
                        break;
                    }
                    _ => {}
                }
            }
        }

        s.parse::<f64>()
            .map_err(|_| ParseError::CorruptedData("Invalid CFF real number".to_string()))
    }

    /// Parse Private DICT
    fn parse_private_dict(data: &[u8]) -> Result<PrivateDict, ParseError> {
        let mut dict = PrivateDict {
            blue_scale: 0.039625,
            blue_shift: 7.0,
            blue_fuzz: 1.0,
            expansion_factor: 0.06,
            ..Default::default()
        };

        let mut operands: Vec<f64> = Vec::new();
        let mut cursor = Cursor::new(data);

        while cursor.position() < data.len() as u64 {
            let b0 = cursor.read_u8()?;

            match b0 {
                6 => {
                    // BlueValues
                    dict.blue_values = operands.clone();
                    operands.clear();
                }
                7 => {
                    // OtherBlues
                    dict.other_blues = operands.clone();
                    operands.clear();
                }
                8 => {
                    // FamilyBlues
                    dict.family_blues = operands.clone();
                    operands.clear();
                }
                9 => {
                    // FamilyOtherBlues
                    dict.family_other_blues = operands.clone();
                    operands.clear();
                }
                10 => {
                    // StdHW
                    dict.std_hw = operands.last().copied();
                    operands.clear();
                }
                11 => {
                    // StdVW
                    dict.std_vw = operands.last().copied();
                    operands.clear();
                }
                12 => {
                    // Two-byte operator
                    let b1 = cursor.read_u8()?;
                    match b1 {
                        9 => {
                            // BlueScale
                            dict.blue_scale = operands.last().copied().unwrap_or(0.039625);
                            operands.clear();
                        }
                        10 => {
                            // BlueShift
                            dict.blue_shift = operands.last().copied().unwrap_or(7.0);
                            operands.clear();
                        }
                        11 => {
                            // BlueFuzz
                            dict.blue_fuzz = operands.last().copied().unwrap_or(1.0);
                            operands.clear();
                        }
                        12 => {
                            // StemSnapH
                            dict.stem_snap_h = operands.clone();
                            operands.clear();
                        }
                        13 => {
                            // StemSnapV
                            dict.stem_snap_v = operands.clone();
                            operands.clear();
                        }
                        14 => {
                            // ForceBold
                            dict.force_bold = operands.last().copied().unwrap_or(0.0) != 0.0;
                            operands.clear();
                        }
                        17 => {
                            // LanguageGroup
                            dict.language_group = operands.last().copied().unwrap_or(0.0) as i32;
                            operands.clear();
                        }
                        18 => {
                            // ExpansionFactor
                            dict.expansion_factor = operands.last().copied().unwrap_or(0.06);
                            operands.clear();
                        }
                        19 => {
                            // initialRandomSeed
                            dict.initial_random_seed = operands.last().copied().unwrap_or(0.0);
                            operands.clear();
                        }
                        _ => {
                            operands.clear();
                        }
                    }
                }
                19 => {
                    // Subrs
                    dict.subrs_offset = operands.last().map(|v| *v as u32);
                    operands.clear();
                }
                20 => {
                    // defaultWidthX
                    dict.default_width_x = operands.last().copied().unwrap_or(0.0);
                    operands.clear();
                }
                21 => {
                    // nominalWidthX
                    dict.nominal_width_x = operands.last().copied().unwrap_or(0.0);
                    operands.clear();
                }
                // Number operands (same as Top DICT)
                28 => {
                    let b1 = cursor.read_u8()?;
                    let b2 = cursor.read_u8()?;
                    let value = ((b1 as i16) << 8) | (b2 as i16);
                    operands.push(value as f64);
                }
                29 => {
                    let value = cursor.read_i32::<BigEndian>()?;
                    operands.push(value as f64);
                }
                30 => {
                    let real = Self::parse_real(&mut cursor)?;
                    operands.push(real);
                }
                32..=246 => {
                    operands.push((b0 as i32 - 139) as f64);
                }
                247..=250 => {
                    let b1 = cursor.read_u8()?;
                    let value = ((b0 as i32 - 247) * 256) + b1 as i32 + 108;
                    operands.push(value as f64);
                }
                251..=254 => {
                    let b1 = cursor.read_u8()?;
                    let value = -((b0 as i32 - 251) * 256) - b1 as i32 - 108;
                    operands.push(value as f64);
                }
                _ => {
                    operands.clear();
                }
            }
        }

        Ok(dict)
    }

    /// Get the number of fonts in the CFF
    pub fn font_count(&self) -> usize {
        self.font_names.len()
    }

    /// Get font name by index
    pub fn get_font_name(&self, index: usize) -> Option<&str> {
        self.font_names.get(index).map(|s| s.as_str())
    }

    /// Get the number of glyphs in a font
    pub fn glyph_count(&self, font_index: usize) -> usize {
        self.charstrings
            .get(font_index)
            .map(|cs| cs.len())
            .unwrap_or(0)
    }

    /// Get raw charstring data for a glyph
    pub fn get_charstring(&self, font_index: usize, glyph_index: usize) -> Option<&[u8]> {
        self.charstrings
            .get(font_index)?
            .get(glyph_index)
            .map(|v| v.as_slice())
    }

    /// Check if font is a CID-keyed font
    pub fn is_cid_font(&self, font_index: usize) -> bool {
        self.top_dicts
            .get(font_index)
            .map(|d| d.ros.is_some())
            .unwrap_or(false)
    }
}

impl Cff2Table {
    /// Parse CFF2 table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 5 {
            return Err(ParseError::CorruptedData(
                "CFF2 data too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read CFF2 header
        let major_version = cursor.read_u8()?;
        let minor_version = cursor.read_u8()?;
        let header_size = cursor.read_u8()?;
        let top_dict_length = cursor.read_u16::<BigEndian>()?;

        if major_version != 2 {
            return Err(ParseError::UnsupportedVersion);
        }

        // Skip to end of header
        cursor.set_position(header_size as u64);

        // Parse Top DICT
        let top_dict_end = header_size as usize + top_dict_length as usize;
        let top_dict_data = &data[header_size as usize..top_dict_end];
        let top_dict = Self::parse_top_dict(top_dict_data)?;

        // Move cursor past Top DICT
        cursor.set_position(top_dict_end as u64);

        // Parse Global Subr INDEX
        let global_subrs = Self::parse_index(data, &mut cursor)?;

        // Parse CharStrings INDEX
        let charstrings = if top_dict.charstrings_offset > 0 {
            let mut cs_cursor = Cursor::new(data);
            cs_cursor.set_position(top_dict.charstrings_offset as u64);
            Self::parse_index(data, &mut cs_cursor)?
        } else {
            Vec::new()
        };

        // Parse FDArray if present
        let font_dicts = if let Some(fd_array_offset) = top_dict.fd_array_offset {
            let mut fd_cursor = Cursor::new(data);
            fd_cursor.set_position(fd_array_offset as u64);
            let fd_data = Self::parse_index(data, &mut fd_cursor)?;
            fd_data
                .iter()
                .map(|d| Self::parse_font_dict(d))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

        // Parse Variation Store if present
        let variation_store = if let Some(vstore_offset) = top_dict.vstore_offset {
            Some(Self::parse_variation_store(
                &data[vstore_offset as usize..],
            )?)
        } else {
            None
        };

        Ok(Cff2Table {
            major_version,
            minor_version,
            top_dict,
            global_subrs,
            charstrings,
            font_dicts,
            variation_store,
        })
    }

    /// Parse CFF2 INDEX structure
    fn parse_index(data: &[u8], cursor: &mut Cursor<&[u8]>) -> Result<Vec<Vec<u8>>, ParseError> {
        let count = cursor.read_u32::<BigEndian>()?;
        if count == 0 {
            return Ok(Vec::new());
        }

        let off_size = cursor.read_u8()?;
        if off_size == 0 || off_size > 4 {
            return Err(ParseError::CorruptedData(
                "Invalid CFF2 INDEX off_size".to_string(),
            ));
        }

        // Read offsets
        let mut offsets = Vec::new();
        for _ in 0..=count {
            let offset = Self::read_offset(cursor, off_size)?;
            offsets.push(offset);
        }

        // Data starts at current position
        let data_start = cursor.position() as usize;

        // Extract data items
        let mut items = Vec::new();
        for i in 0..count as usize {
            let start = data_start + offsets[i] as usize - 1;
            let end = data_start + offsets[i + 1] as usize - 1;
            if end > data.len() {
                return Err(ParseError::CorruptedData(
                    "CFF2 INDEX data extends past buffer".to_string(),
                ));
            }
            items.push(data[start..end].to_vec());
        }

        // Move cursor past data
        let last_offset = offsets[count as usize] as usize;
        cursor.set_position((data_start + last_offset - 1) as u64);

        Ok(items)
    }

    /// Read an offset value of variable size
    fn read_offset(cursor: &mut Cursor<&[u8]>, off_size: u8) -> Result<u32, ParseError> {
        let mut value: u32 = 0;
        for _ in 0..off_size {
            value = (value << 8) | cursor.read_u8()? as u32;
        }
        Ok(value)
    }

    /// Parse CFF2 Top DICT
    fn parse_top_dict(data: &[u8]) -> Result<Cff2TopDict, ParseError> {
        let mut dict = Cff2TopDict::default();
        let mut operands: Vec<f64> = Vec::new();
        let mut cursor = Cursor::new(data);

        while cursor.position() < data.len() as u64 {
            let b0 = cursor.read_u8()?;

            match b0 {
                12 => {
                    // Two-byte operator
                    let b1 = cursor.read_u8()?;
                    match b1 {
                        36 => {
                            // FDArray
                            dict.fd_array_offset = operands.last().map(|v| *v as u32);
                            operands.clear();
                        }
                        37 => {
                            // FDSelect
                            dict.fd_select_offset = operands.last().map(|v| *v as u32);
                            operands.clear();
                        }
                        _ => {
                            operands.clear();
                        }
                    }
                }
                17 => {
                    // CharStrings
                    dict.charstrings_offset = operands.last().copied().unwrap_or(0.0) as u32;
                    operands.clear();
                }
                24 => {
                    // vstore
                    dict.vstore_offset = operands.last().map(|v| *v as u32);
                    operands.clear();
                }
                // Number operands
                28 => {
                    let b1 = cursor.read_u8()?;
                    let b2 = cursor.read_u8()?;
                    let value = ((b1 as i16) << 8) | (b2 as i16);
                    operands.push(value as f64);
                }
                29 => {
                    let value = cursor.read_i32::<BigEndian>()?;
                    operands.push(value as f64);
                }
                32..=246 => {
                    operands.push((b0 as i32 - 139) as f64);
                }
                247..=250 => {
                    let b1 = cursor.read_u8()?;
                    let value = ((b0 as i32 - 247) * 256) + b1 as i32 + 108;
                    operands.push(value as f64);
                }
                251..=254 => {
                    let b1 = cursor.read_u8()?;
                    let value = -((b0 as i32 - 251) * 256) - b1 as i32 - 108;
                    operands.push(value as f64);
                }
                _ => {
                    operands.clear();
                }
            }
        }

        Ok(dict)
    }

    /// Parse Font DICT
    fn parse_font_dict(data: &[u8]) -> Result<Cff2FontDict, ParseError> {
        let mut dict = Cff2FontDict::default();
        let mut operands: Vec<f64> = Vec::new();
        let mut cursor = Cursor::new(data);

        while cursor.position() < data.len() as u64 {
            let b0 = cursor.read_u8()?;

            match b0 {
                18 => {
                    // Private
                    if operands.len() >= 2 {
                        dict.private = Some((operands[0] as u32, operands[1] as u32));
                    }
                    operands.clear();
                }
                // Number operands
                28 => {
                    let b1 = cursor.read_u8()?;
                    let b2 = cursor.read_u8()?;
                    let value = ((b1 as i16) << 8) | (b2 as i16);
                    operands.push(value as f64);
                }
                29 => {
                    let value = cursor.read_i32::<BigEndian>()?;
                    operands.push(value as f64);
                }
                32..=246 => {
                    operands.push((b0 as i32 - 139) as f64);
                }
                247..=250 => {
                    let b1 = cursor.read_u8()?;
                    let value = ((b0 as i32 - 247) * 256) + b1 as i32 + 108;
                    operands.push(value as f64);
                }
                251..=254 => {
                    let b1 = cursor.read_u8()?;
                    let value = -((b0 as i32 - 251) * 256) - b1 as i32 - 108;
                    operands.push(value as f64);
                }
                _ => {
                    operands.clear();
                }
            }
        }

        Ok(dict)
    }

    /// Parse Item Variation Store
    fn parse_variation_store(data: &[u8]) -> Result<ItemVariationStore, ParseError> {
        let mut cursor = Cursor::new(data);

        let format = cursor.read_u16::<BigEndian>()?;
        if format != 1 {
            return Err(ParseError::UnsupportedVersion);
        }

        let _variation_region_list_offset = cursor.read_u32::<BigEndian>()?;
        let _item_variation_data_count = cursor.read_u16::<BigEndian>()?;

        // For now, just return a basic structure
        // Full implementation would parse all variation data
        let item_variation_data = Vec::new();

        Ok(ItemVariationStore {
            format,
            item_variation_data,
        })
    }

    /// Get the number of glyphs
    pub fn glyph_count(&self) -> usize {
        self.charstrings.len()
    }

    /// Get raw charstring data for a glyph
    pub fn get_charstring(&self, glyph_index: usize) -> Option<&[u8]> {
        self.charstrings.get(glyph_index).map(|v| v.as_slice())
    }

    /// Check if this is a variable font
    pub fn is_variable(&self) -> bool {
        self.variation_store.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_dict_default() {
        let dict = TopDict::default();
        assert_eq!(dict.charstring_type, 0);
        assert!(!dict.is_fixed_pitch);
        assert_eq!(dict.italic_angle, 0.0);
    }

    #[test]
    fn test_private_dict_default() {
        let dict = PrivateDict::default();
        assert!(dict.blue_values.is_empty());
        assert_eq!(dict.default_width_x, 0.0);
        assert!(!dict.force_bold);
    }

    #[test]
    fn test_cff2_top_dict_default() {
        let dict = Cff2TopDict::default();
        assert_eq!(dict.charstrings_offset, 0);
        assert!(dict.fd_array_offset.is_none());
    }

    #[test]
    fn test_charstring_command_variants() {
        let _ = CharStringCommand::MoveTo(0.0, 0.0);
        let _ = CharStringCommand::LineTo(100.0, 100.0);
        let _ = CharStringCommand::CurveTo(0.0, 0.0, 50.0, 50.0, 100.0, 100.0);
        let _ = CharStringCommand::EndChar;
        let _ = CharStringCommand::HintMask(vec![0xFF]);
        let _ = CharStringCommand::HStem(0.0, 100.0);
        let _ = CharStringCommand::VStem(0.0, 100.0);
    }

    #[test]
    fn test_item_variation_store_default() {
        let store = ItemVariationStore::default();
        assert_eq!(store.format, 0);
        assert!(store.item_variation_data.is_empty());
    }

    #[test]
    fn test_cff_parse_short_data() {
        let result = CffTable::parse(&[0, 1, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cff2_parse_short_data() {
        let result = Cff2Table::parse(&[0, 1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cff_invalid_version() {
        // Version 2 should fail for CFF (expects version 1)
        let data = vec![
            2, 0, // Version 2.0
            4,  // Header size
            2,  // Off size
            0, 0, // Empty Name INDEX count
        ];
        let result = CffTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cff2_invalid_version() {
        // Version 1 should fail for CFF2 (expects version 2)
        let data = vec![
            1, 0, // Version 1.0
            5,  // Header size
            0, 0, // Top dict length
        ];
        let result = Cff2Table::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cff2_font_dict_default() {
        let dict = Cff2FontDict::default();
        assert!(dict.private.is_none());
    }

    #[test]
    fn test_item_variation_data_default() {
        let data = ItemVariationData::default();
        assert_eq!(data.item_count, 0);
        assert!(data.region_indices.is_empty());
        assert!(data.delta_sets.is_empty());
    }
}
