//! Bitmap Font Support (EBDT/EBLC, EBSC)
//!
//! This module provides parsing for embedded bitmap font tables:
//! - EBLC (Embedded Bitmap Location Table): Index and size information
//! - EBDT (Embedded Bitmap Data Table): Actual bitmap data
//! - EBSC (Embedded Bitmap Scaling Table): Scaling behavior

use crate::types::GlyphId;
use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// EBLC Table - Embedded Bitmap Location Table
///
/// Provides size information and locations for embedded bitmap data.
#[derive(Debug, Clone)]
pub struct EblcTable {
    /// Major version
    pub major_version: u16,
    /// Minor version
    pub minor_version: u16,
    /// Bitmap size records (one per strike)
    pub bitmap_sizes: Vec<BitmapSizeRecord>,
}

/// Bitmap size record - describes a bitmap strike
#[derive(Debug, Clone)]
pub struct BitmapSizeRecord {
    /// Offset to IndexSubTableArray
    pub index_subtable_array_offset: u32,
    /// Size of IndexSubTables
    pub index_subtables_size: u32,
    /// Number of index subtables
    pub number_of_index_subtables: u32,
    /// Color reference (reserved in EBLC)
    pub color_ref: u32,
    /// Horizontal line metrics
    pub hori: SbitLineMetrics,
    /// Vertical line metrics
    pub vert: SbitLineMetrics,
    /// First glyph index
    pub start_glyph_index: u16,
    /// Last glyph index
    pub end_glyph_index: u16,
    /// Horizontal pixels per em
    pub ppem_x: u8,
    /// Vertical pixels per em
    pub ppem_y: u8,
    /// Bit depth (1, 2, 4, 8, or 32)
    pub bit_depth: u8,
    /// Flags
    pub flags: i8,
}

/// Line metrics for bitmap strikes
#[derive(Debug, Clone, Copy, Default)]
pub struct SbitLineMetrics {
    /// Ascender
    pub ascender: i8,
    /// Descender
    pub descender: i8,
    /// Maximum width
    pub width_max: u8,
    /// Caret slope numerator
    pub caret_slope_numerator: i8,
    /// Caret slope denominator
    pub caret_slope_denominator: i8,
    /// Caret offset
    pub caret_offset: i8,
    /// Minimum origin SB
    pub min_origin_sb: i8,
    /// Minimum advance SB
    pub min_advance_sb: i8,
    /// Maximum before BL
    pub max_before_bl: i8,
    /// Minimum after BL
    pub min_after_bl: i8,
    /// Padding
    pub pad1: i8,
    /// Padding
    pub pad2: i8,
}

/// Index subtable header
#[derive(Debug, Clone)]
pub struct IndexSubTableHeader {
    /// First glyph index
    pub first_glyph_index: u16,
    /// Last glyph index
    pub last_glyph_index: u16,
    /// Additional offset to index subtable
    pub additional_offset_to_index_subtable: u32,
}

/// Index subtable format types
#[derive(Debug, Clone)]
pub enum IndexSubTable {
    /// Format 1: Variable metrics with 4-byte offsets
    Format1 {
        /// Image format
        image_format: u16,
        /// Image data offset
        image_data_offset: u32,
        /// Offsets to image data
        sbit_offsets: Vec<u32>,
    },
    /// Format 2: Constant metrics with identical glyph metrics
    Format2 {
        /// Image format
        image_format: u16,
        /// Image data offset
        image_data_offset: u32,
        /// Image size
        image_size: u32,
        /// Big metrics for all glyphs
        big_metrics: BigGlyphMetrics,
    },
    /// Format 3: Variable metrics with 2-byte offsets
    Format3 {
        /// Image format
        image_format: u16,
        /// Image data offset
        image_data_offset: u32,
        /// Offsets to image data
        sbit_offsets: Vec<u16>,
    },
    /// Format 4: Variable metrics with sparse glyph codes
    Format4 {
        /// Image format
        image_format: u16,
        /// Image data offset
        image_data_offset: u32,
        /// Glyph code/offset pairs
        glyph_array: Vec<GlyphIdOffsetPair>,
    },
    /// Format 5: Constant metrics with sparse glyph codes
    Format5 {
        /// Image format
        image_format: u16,
        /// Image data offset
        image_data_offset: u32,
        /// Image size
        image_size: u32,
        /// Big metrics for all glyphs
        big_metrics: BigGlyphMetrics,
        /// Glyph codes
        glyph_codes: Vec<u16>,
    },
}

/// Big glyph metrics (used for format 2 and 5)
#[derive(Debug, Clone, Copy, Default)]
pub struct BigGlyphMetrics {
    /// Height
    pub height: u8,
    /// Width
    pub width: u8,
    /// Horizontal bearing X
    pub hori_bearing_x: i8,
    /// Horizontal bearing Y
    pub hori_bearing_y: i8,
    /// Horizontal advance
    pub hori_advance: u8,
    /// Vertical bearing X
    pub vert_bearing_x: i8,
    /// Vertical bearing Y
    pub vert_bearing_y: i8,
    /// Vertical advance
    pub vert_advance: u8,
}

/// Small glyph metrics (used for format 1 and 3)
#[derive(Debug, Clone, Copy, Default)]
pub struct SmallGlyphMetrics {
    /// Height
    pub height: u8,
    /// Width
    pub width: u8,
    /// Bearing X
    pub bearing_x: i8,
    /// Bearing Y
    pub bearing_y: i8,
    /// Advance
    pub advance: u8,
}

/// Glyph ID and offset pair (for format 4)
#[derive(Debug, Clone, Copy)]
pub struct GlyphIdOffsetPair {
    /// Glyph ID
    pub glyph_id: u16,
    /// Sbit offset
    pub sbit_offset: u16,
}

/// EBDT Table - Embedded Bitmap Data Table
#[derive(Debug, Clone)]
pub struct EbdtTable {
    /// Major version
    pub major_version: u16,
    /// Minor version
    pub minor_version: u16,
    /// Raw bitmap data
    pub data: Vec<u8>,
}

/// Bitmap glyph data with metrics
#[derive(Debug, Clone)]
pub struct BitmapGlyph {
    /// Small metrics (for formats 1, 2, 8)
    pub small_metrics: Option<SmallGlyphMetrics>,
    /// Big metrics (for formats 6, 7, 9)
    pub big_metrics: Option<BigGlyphMetrics>,
    /// Raw bitmap data
    pub bitmap_data: Vec<u8>,
    /// Image format (1-9)
    pub format: u8,
}

/// EBSC Table - Embedded Bitmap Scaling Table
#[derive(Debug, Clone)]
pub struct EbscTable {
    /// Major version
    pub major_version: u16,
    /// Minor version
    pub minor_version: u16,
    /// Scaling records
    pub strikes: Vec<BitmapScaleRecord>,
}

/// Bitmap scaling record
#[derive(Debug, Clone)]
pub struct BitmapScaleRecord {
    /// Horizontal line metrics for substitution
    pub hori: SbitLineMetrics,
    /// Vertical line metrics for substitution
    pub vert: SbitLineMetrics,
    /// Horizontal PPEM to substitute
    pub ppem_x: u8,
    /// Vertical PPEM to substitute
    pub ppem_y: u8,
    /// Horizontal PPEM to use for substitution
    pub substitute_ppem_x: u8,
    /// Vertical PPEM to use for substitution
    pub substitute_ppem_y: u8,
}

impl EblcTable {
    /// Parse EBLC table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 8 {
            return Err(ParseError::CorruptedData(
                "EBLC table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        let major_version = cursor.read_u16::<BigEndian>()?;
        let minor_version = cursor.read_u16::<BigEndian>()?;
        let num_sizes = cursor.read_u32::<BigEndian>()?;

        // Parse bitmap size records
        let mut bitmap_sizes = Vec::with_capacity(num_sizes as usize);
        for _ in 0..num_sizes {
            let bitmap_size = Self::parse_bitmap_size_record(&mut cursor)?;
            bitmap_sizes.push(bitmap_size);
        }

        Ok(EblcTable {
            major_version,
            minor_version,
            bitmap_sizes,
        })
    }

    /// Parse a bitmap size record
    fn parse_bitmap_size_record(
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<BitmapSizeRecord, ParseError> {
        let index_subtable_array_offset = cursor.read_u32::<BigEndian>()?;
        let index_subtables_size = cursor.read_u32::<BigEndian>()?;
        let number_of_index_subtables = cursor.read_u32::<BigEndian>()?;
        let color_ref = cursor.read_u32::<BigEndian>()?;

        // Parse horizontal line metrics
        let hori = Self::parse_sbit_line_metrics(cursor)?;

        // Parse vertical line metrics
        let vert = Self::parse_sbit_line_metrics(cursor)?;

        let start_glyph_index = cursor.read_u16::<BigEndian>()?;
        let end_glyph_index = cursor.read_u16::<BigEndian>()?;
        let ppem_x = cursor.read_u8()?;
        let ppem_y = cursor.read_u8()?;
        let bit_depth = cursor.read_u8()?;
        let flags = cursor.read_i8()?;

        Ok(BitmapSizeRecord {
            index_subtable_array_offset,
            index_subtables_size,
            number_of_index_subtables,
            color_ref,
            hori,
            vert,
            start_glyph_index,
            end_glyph_index,
            ppem_x,
            ppem_y,
            bit_depth,
            flags,
        })
    }

    /// Parse sbit line metrics
    fn parse_sbit_line_metrics(cursor: &mut Cursor<&[u8]>) -> Result<SbitLineMetrics, ParseError> {
        Ok(SbitLineMetrics {
            ascender: cursor.read_i8()?,
            descender: cursor.read_i8()?,
            width_max: cursor.read_u8()?,
            caret_slope_numerator: cursor.read_i8()?,
            caret_slope_denominator: cursor.read_i8()?,
            caret_offset: cursor.read_i8()?,
            min_origin_sb: cursor.read_i8()?,
            min_advance_sb: cursor.read_i8()?,
            max_before_bl: cursor.read_i8()?,
            min_after_bl: cursor.read_i8()?,
            pad1: cursor.read_i8()?,
            pad2: cursor.read_i8()?,
        })
    }

    /// Get the number of bitmap strikes
    pub fn strike_count(&self) -> usize {
        self.bitmap_sizes.len()
    }

    /// Get bitmap size record by index
    pub fn get_strike(&self, index: usize) -> Option<&BitmapSizeRecord> {
        self.bitmap_sizes.get(index)
    }

    /// Find the best strike for a given PPEM
    pub fn find_best_strike(&self, ppem: u8) -> Option<(usize, &BitmapSizeRecord)> {
        // First try exact match
        if let Some((idx, strike)) = self
            .bitmap_sizes
            .iter()
            .enumerate()
            .find(|(_, s)| s.ppem_x == ppem || s.ppem_y == ppem)
        {
            return Some((idx, strike));
        }

        // Find closest match
        self.bitmap_sizes.iter().enumerate().min_by_key(|(_, s)| {
            let diff_x = (s.ppem_x as i16 - ppem as i16).abs();
            let diff_y = (s.ppem_y as i16 - ppem as i16).abs();
            diff_x.min(diff_y)
        })
    }

    /// Check if a glyph has a bitmap at a specific strike
    pub fn has_bitmap(&self, strike_index: usize, glyph_id: GlyphId) -> bool {
        if let Some(strike) = self.get_strike(strike_index) {
            glyph_id >= strike.start_glyph_index && glyph_id <= strike.end_glyph_index
        } else {
            false
        }
    }
}

impl EbdtTable {
    /// Parse EBDT table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 4 {
            return Err(ParseError::CorruptedData(
                "EBDT table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        let major_version = cursor.read_u16::<BigEndian>()?;
        let minor_version = cursor.read_u16::<BigEndian>()?;

        // Store remaining data
        let data = data[4..].to_vec();

        Ok(EbdtTable {
            major_version,
            minor_version,
            data,
        })
    }

    /// Get bitmap data at an offset with a specific format
    pub fn get_bitmap(
        &self,
        offset: usize,
        length: usize,
        format: u8,
    ) -> Result<BitmapGlyph, ParseError> {
        if offset + length > self.data.len() {
            return Err(ParseError::CorruptedData(
                "Bitmap data offset out of range".to_string(),
            ));
        }

        let glyph_data = &self.data[offset..offset + length];
        let mut cursor = Cursor::new(glyph_data);

        let (small_metrics, big_metrics, bitmap_start) = match format {
            1 | 2 | 8 => {
                // Small metrics
                let metrics = Self::parse_small_metrics(&mut cursor)?;
                (Some(metrics), None, 5)
            }
            6 | 7 | 9 => {
                // Big metrics
                let metrics = Self::parse_big_metrics(&mut cursor)?;
                (None, Some(metrics), 8)
            }
            5 => {
                // No metrics in data (use subtable metrics)
                (None, None, 0)
            }
            _ => {
                return Err(ParseError::CorruptedData(format!(
                    "Unknown bitmap format: {}",
                    format
                )));
            }
        };

        let bitmap_data = if bitmap_start < glyph_data.len() {
            glyph_data[bitmap_start..].to_vec()
        } else {
            Vec::new()
        };

        Ok(BitmapGlyph {
            small_metrics,
            big_metrics,
            bitmap_data,
            format,
        })
    }

    /// Parse small glyph metrics
    fn parse_small_metrics(cursor: &mut Cursor<&[u8]>) -> Result<SmallGlyphMetrics, ParseError> {
        Ok(SmallGlyphMetrics {
            height: cursor.read_u8()?,
            width: cursor.read_u8()?,
            bearing_x: cursor.read_i8()?,
            bearing_y: cursor.read_i8()?,
            advance: cursor.read_u8()?,
        })
    }

    /// Parse big glyph metrics
    fn parse_big_metrics(cursor: &mut Cursor<&[u8]>) -> Result<BigGlyphMetrics, ParseError> {
        Ok(BigGlyphMetrics {
            height: cursor.read_u8()?,
            width: cursor.read_u8()?,
            hori_bearing_x: cursor.read_i8()?,
            hori_bearing_y: cursor.read_i8()?,
            hori_advance: cursor.read_u8()?,
            vert_bearing_x: cursor.read_i8()?,
            vert_bearing_y: cursor.read_i8()?,
            vert_advance: cursor.read_u8()?,
        })
    }
}

impl EbscTable {
    /// Parse EBSC table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 8 {
            return Err(ParseError::CorruptedData(
                "EBSC table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        let major_version = cursor.read_u16::<BigEndian>()?;
        let minor_version = cursor.read_u16::<BigEndian>()?;
        let num_sizes = cursor.read_u32::<BigEndian>()?;

        let mut strikes = Vec::with_capacity(num_sizes as usize);
        for _ in 0..num_sizes {
            let strike = Self::parse_bitmap_scale_record(&mut cursor)?;
            strikes.push(strike);
        }

        Ok(EbscTable {
            major_version,
            minor_version,
            strikes,
        })
    }

    /// Parse a bitmap scale record
    fn parse_bitmap_scale_record(
        cursor: &mut Cursor<&[u8]>,
    ) -> Result<BitmapScaleRecord, ParseError> {
        // Parse horizontal line metrics
        let hori = SbitLineMetrics {
            ascender: cursor.read_i8()?,
            descender: cursor.read_i8()?,
            width_max: cursor.read_u8()?,
            caret_slope_numerator: cursor.read_i8()?,
            caret_slope_denominator: cursor.read_i8()?,
            caret_offset: cursor.read_i8()?,
            min_origin_sb: cursor.read_i8()?,
            min_advance_sb: cursor.read_i8()?,
            max_before_bl: cursor.read_i8()?,
            min_after_bl: cursor.read_i8()?,
            pad1: cursor.read_i8()?,
            pad2: cursor.read_i8()?,
        };

        // Parse vertical line metrics
        let vert = SbitLineMetrics {
            ascender: cursor.read_i8()?,
            descender: cursor.read_i8()?,
            width_max: cursor.read_u8()?,
            caret_slope_numerator: cursor.read_i8()?,
            caret_slope_denominator: cursor.read_i8()?,
            caret_offset: cursor.read_i8()?,
            min_origin_sb: cursor.read_i8()?,
            min_advance_sb: cursor.read_i8()?,
            max_before_bl: cursor.read_i8()?,
            min_after_bl: cursor.read_i8()?,
            pad1: cursor.read_i8()?,
            pad2: cursor.read_i8()?,
        };

        let ppem_x = cursor.read_u8()?;
        let ppem_y = cursor.read_u8()?;
        let substitute_ppem_x = cursor.read_u8()?;
        let substitute_ppem_y = cursor.read_u8()?;

        Ok(BitmapScaleRecord {
            hori,
            vert,
            ppem_x,
            ppem_y,
            substitute_ppem_x,
            substitute_ppem_y,
        })
    }

    /// Find a scaling substitution for a given PPEM
    pub fn find_substitution(&self, ppem: u8) -> Option<&BitmapScaleRecord> {
        self.strikes
            .iter()
            .find(|s| s.ppem_x == ppem || s.ppem_y == ppem)
    }
}

impl BitmapGlyph {
    /// Get the width of the bitmap
    pub fn width(&self) -> u8 {
        self.big_metrics
            .map(|m| m.width)
            .or_else(|| self.small_metrics.map(|m| m.width))
            .unwrap_or(0)
    }

    /// Get the height of the bitmap
    pub fn height(&self) -> u8 {
        self.big_metrics
            .map(|m| m.height)
            .or_else(|| self.small_metrics.map(|m| m.height))
            .unwrap_or(0)
    }

    /// Get the advance width
    pub fn advance(&self) -> u8 {
        self.big_metrics
            .map(|m| m.hori_advance)
            .or_else(|| self.small_metrics.map(|m| m.advance))
            .unwrap_or(0)
    }

    /// Get the bitmap data
    pub fn data(&self) -> &[u8] {
        &self.bitmap_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eblc_parse_too_short() {
        let data = vec![0, 0, 0];
        let result = EblcTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_eblc_parse_empty_strikes() {
        let data = vec![
            0x00, 0x02, // major version = 2
            0x00, 0x00, // minor version = 0
            0x00, 0x00, 0x00, 0x00, // num_sizes = 0
        ];
        let table = EblcTable::parse(&data).unwrap();
        assert_eq!(table.major_version, 2);
        assert_eq!(table.minor_version, 0);
        assert_eq!(table.strike_count(), 0);
    }

    #[test]
    fn test_ebdt_parse_too_short() {
        let data = vec![0, 0];
        let result = EbdtTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_ebdt_parse_minimal() {
        let data = vec![
            0x00, 0x02, // major version = 2
            0x00, 0x00, // minor version = 0
        ];
        let table = EbdtTable::parse(&data).unwrap();
        assert_eq!(table.major_version, 2);
        assert_eq!(table.minor_version, 0);
        assert!(table.data.is_empty());
    }

    #[test]
    fn test_ebsc_parse_too_short() {
        let data = vec![0, 0, 0];
        let result = EbscTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_ebsc_parse_empty_strikes() {
        let data = vec![
            0x00, 0x02, // major version = 2
            0x00, 0x00, // minor version = 0
            0x00, 0x00, 0x00, 0x00, // num_sizes = 0
        ];
        let table = EbscTable::parse(&data).unwrap();
        assert_eq!(table.major_version, 2);
        assert_eq!(table.minor_version, 0);
        assert_eq!(table.strikes.len(), 0);
    }

    #[test]
    fn test_sbit_line_metrics_default() {
        let metrics = SbitLineMetrics::default();
        assert_eq!(metrics.ascender, 0);
        assert_eq!(metrics.descender, 0);
        assert_eq!(metrics.width_max, 0);
    }

    #[test]
    fn test_big_glyph_metrics_default() {
        let metrics = BigGlyphMetrics::default();
        assert_eq!(metrics.width, 0);
        assert_eq!(metrics.height, 0);
        assert_eq!(metrics.hori_advance, 0);
    }

    #[test]
    fn test_small_glyph_metrics_default() {
        let metrics = SmallGlyphMetrics::default();
        assert_eq!(metrics.width, 0);
        assert_eq!(metrics.height, 0);
        assert_eq!(metrics.advance, 0);
    }

    #[test]
    fn test_bitmap_glyph_dimensions_from_big_metrics() {
        let glyph = BitmapGlyph {
            small_metrics: None,
            big_metrics: Some(BigGlyphMetrics {
                height: 16,
                width: 12,
                hori_bearing_x: -1,
                hori_bearing_y: 14,
                hori_advance: 14,
                vert_bearing_x: -6,
                vert_bearing_y: -7,
                vert_advance: 16,
            }),
            bitmap_data: vec![],
            format: 6,
        };
        assert_eq!(glyph.width(), 12);
        assert_eq!(glyph.height(), 16);
        assert_eq!(glyph.advance(), 14);
    }

    #[test]
    fn test_bitmap_glyph_dimensions_from_small_metrics() {
        let glyph = BitmapGlyph {
            small_metrics: Some(SmallGlyphMetrics {
                height: 10,
                width: 8,
                bearing_x: 0,
                bearing_y: 9,
                advance: 10,
            }),
            big_metrics: None,
            bitmap_data: vec![],
            format: 1,
        };
        assert_eq!(glyph.width(), 8);
        assert_eq!(glyph.height(), 10);
        assert_eq!(glyph.advance(), 10);
    }

    #[test]
    fn test_bitmap_glyph_no_metrics() {
        let glyph = BitmapGlyph {
            small_metrics: None,
            big_metrics: None,
            bitmap_data: vec![0xFF; 10],
            format: 5,
        };
        assert_eq!(glyph.width(), 0);
        assert_eq!(glyph.height(), 0);
        assert_eq!(glyph.advance(), 0);
        assert_eq!(glyph.data().len(), 10);
    }

    #[test]
    fn test_glyph_id_offset_pair() {
        let pair = GlyphIdOffsetPair {
            glyph_id: 42,
            sbit_offset: 1024,
        };
        assert_eq!(pair.glyph_id, 42);
        assert_eq!(pair.sbit_offset, 1024);
    }

    #[test]
    fn test_bitmap_size_record_fields() {
        let record = BitmapSizeRecord {
            index_subtable_array_offset: 100,
            index_subtables_size: 200,
            number_of_index_subtables: 5,
            color_ref: 0,
            hori: SbitLineMetrics::default(),
            vert: SbitLineMetrics::default(),
            start_glyph_index: 10,
            end_glyph_index: 100,
            ppem_x: 16,
            ppem_y: 16,
            bit_depth: 8,
            flags: 0,
        };
        assert_eq!(record.ppem_x, 16);
        assert_eq!(record.ppem_y, 16);
        assert_eq!(record.bit_depth, 8);
    }

    #[test]
    fn test_eblc_has_bitmap() {
        let table = EblcTable {
            major_version: 2,
            minor_version: 0,
            bitmap_sizes: vec![BitmapSizeRecord {
                index_subtable_array_offset: 0,
                index_subtables_size: 0,
                number_of_index_subtables: 0,
                color_ref: 0,
                hori: SbitLineMetrics::default(),
                vert: SbitLineMetrics::default(),
                start_glyph_index: 10,
                end_glyph_index: 20,
                ppem_x: 16,
                ppem_y: 16,
                bit_depth: 8,
                flags: 0,
            }],
        };
        assert!(table.has_bitmap(0, 10));
        assert!(table.has_bitmap(0, 15));
        assert!(table.has_bitmap(0, 20));
        assert!(!table.has_bitmap(0, 9));
        assert!(!table.has_bitmap(0, 21));
        assert!(!table.has_bitmap(1, 15)); // Invalid strike index
    }

    #[test]
    fn test_eblc_find_best_strike() {
        let table = EblcTable {
            major_version: 2,
            minor_version: 0,
            bitmap_sizes: vec![
                BitmapSizeRecord {
                    index_subtable_array_offset: 0,
                    index_subtables_size: 0,
                    number_of_index_subtables: 0,
                    color_ref: 0,
                    hori: SbitLineMetrics::default(),
                    vert: SbitLineMetrics::default(),
                    start_glyph_index: 0,
                    end_glyph_index: 100,
                    ppem_x: 12,
                    ppem_y: 12,
                    bit_depth: 8,
                    flags: 0,
                },
                BitmapSizeRecord {
                    index_subtable_array_offset: 0,
                    index_subtables_size: 0,
                    number_of_index_subtables: 0,
                    color_ref: 0,
                    hori: SbitLineMetrics::default(),
                    vert: SbitLineMetrics::default(),
                    start_glyph_index: 0,
                    end_glyph_index: 100,
                    ppem_x: 16,
                    ppem_y: 16,
                    bit_depth: 8,
                    flags: 0,
                },
            ],
        };

        // Exact match
        let (idx, strike) = table.find_best_strike(16).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(strike.ppem_x, 16);

        // Closest match (15 is closer to 16 than to 12)
        let (idx, strike) = table.find_best_strike(15).unwrap();
        assert_eq!(idx, 1); // 16 is closer to 15 than 12
        assert_eq!(strike.ppem_x, 16);
    }

    #[test]
    fn test_ebdt_get_bitmap_out_of_range() {
        let table = EbdtTable {
            major_version: 2,
            minor_version: 0,
            data: vec![0; 10],
        };
        let result = table.get_bitmap(5, 10, 1); // 5 + 10 > 10
        assert!(result.is_err());
    }

    #[test]
    fn test_index_subtable_format1() {
        let subtable = IndexSubTable::Format1 {
            image_format: 1,
            image_data_offset: 100,
            sbit_offsets: vec![0, 50, 100],
        };
        match subtable {
            IndexSubTable::Format1 {
                image_format,
                image_data_offset,
                sbit_offsets,
            } => {
                assert_eq!(image_format, 1);
                assert_eq!(image_data_offset, 100);
                assert_eq!(sbit_offsets.len(), 3);
            }
            _ => panic!("Wrong format"),
        }
    }

    #[test]
    fn test_index_subtable_format2() {
        let subtable = IndexSubTable::Format2 {
            image_format: 2,
            image_data_offset: 200,
            image_size: 64,
            big_metrics: BigGlyphMetrics::default(),
        };
        match subtable {
            IndexSubTable::Format2 {
                image_format,
                image_size,
                ..
            } => {
                assert_eq!(image_format, 2);
                assert_eq!(image_size, 64);
            }
            _ => panic!("Wrong format"),
        }
    }

    #[test]
    fn test_ebsc_find_substitution() {
        let hori = SbitLineMetrics::default();
        let vert = SbitLineMetrics::default();
        let table = EbscTable {
            major_version: 2,
            minor_version: 0,
            strikes: vec![BitmapScaleRecord {
                hori,
                vert,
                ppem_x: 12,
                ppem_y: 12,
                substitute_ppem_x: 16,
                substitute_ppem_y: 16,
            }],
        };

        let sub = table.find_substitution(12);
        assert!(sub.is_some());
        assert_eq!(sub.unwrap().substitute_ppem_x, 16);

        let sub = table.find_substitution(14);
        assert!(sub.is_none());
    }
}
