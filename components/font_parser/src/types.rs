//! Common types for font_parser

use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;
use std::str::FromStr;

/// OpenType table tag (4-byte identifier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag([u8; 4]);

/// Table record in the font directory
#[derive(Debug, Clone)]
struct TableRecord {
    #[allow(dead_code)] // Checksum validation not yet implemented
    checksum: u32,
    offset: u32,
    length: u32,
}

/// Parsed OpenType font
#[derive(Debug, Clone)]
pub struct OpenTypeFont {
    data: Vec<u8>,
    tables: HashMap<Tag, TableRecord>,
}

/// Font metrics (from head, hhea tables)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Units per em (from head table)
    pub units_per_em: u16,
    /// Ascender (from hhea table)
    pub ascender: i16,
    /// Descender (from hhea table)
    pub descender: i16,
    /// Line gap (from hhea table)
    pub line_gap: i16,
}

/// Glyph ID
pub type GlyphId = u16;

/// Character mapping table
#[derive(Debug, Clone)]
pub struct CMapTable {
    // Simplified cmap - maps codepoints to glyph IDs
    mappings: HashMap<u32, GlyphId>,
}

impl CMapTable {
    /// Get glyph ID for a character
    pub fn get_glyph(&self, codepoint: char) -> Option<GlyphId> {
        self.mappings.get(&(codepoint as u32)).copied()
    }
}

/// Bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// Minimum X coordinate
    pub x_min: i16,
    /// Minimum Y coordinate
    pub y_min: i16,
    /// Maximum X coordinate
    pub x_max: i16,
    /// Maximum Y coordinate
    pub y_max: i16,
}

/// Contour in a glyph outline
#[derive(Debug, Clone, PartialEq)]
pub struct Contour {
    /// Points in the contour
    pub points: Vec<(i16, i16)>,
    /// On-curve flags for each point
    pub on_curve: Vec<bool>,
}

/// Vector glyph outline
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphOutline {
    /// Contours in the glyph
    pub contours: Vec<Contour>,
    /// Bounding box
    pub bounds: BoundingBox,
}

/// Error when parsing Tag from string
#[derive(Debug, Clone)]
pub struct TagParseError;

impl fmt::Display for TagParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tag must be exactly 4 bytes")
    }
}

impl std::error::Error for TagParseError {}

impl FromStr for Tag {
    type Err = TagParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(TagParseError);
        }
        let bytes = s.as_bytes();
        Ok(Tag([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

impl Tag {
    /// Create a Tag from a string (convenience wrapper around FromStr)
    /// Returns None if string is not exactly 4 bytes
    pub fn new(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Create a Tag from a 32-bit big-endian value
    pub fn from_bytes(value: u32) -> Self {
        Tag([
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])
    }

    /// Convert Tag to a 32-bit big-endian value
    pub fn to_bytes(&self) -> u32 {
        ((self.0[0] as u32) << 24)
            | ((self.0[1] as u32) << 16)
            | ((self.0[2] as u32) << 8)
            | (self.0[3] as u32)
    }

    /// Get the tag as a string slice
    pub fn as_str(&self) -> &str {
        // Safety: We only create Tags from valid UTF-8 strings or bytes
        std::str::from_utf8(&self.0).unwrap_or("????")
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl OpenTypeFont {
    /// Parse an OpenType/TrueType font from bytes
    pub fn parse(data: Vec<u8>) -> Result<Self, ParseError> {
        if data.len() < 12 {
            return Err(ParseError::CorruptedData("Font data too short".to_string()));
        }

        let mut cursor = Cursor::new(&data);

        // Read sfnt version
        let version = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        // Check for valid versions: 0x00010000 (TrueType) or 0x4F54544F ('OTTO' = OpenType/CFF)
        if version != 0x00010000 && version != 0x4F54544F {
            return Err(ParseError::InvalidFormat);
        }

        // Read table directory header
        let num_tables = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _search_range = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _entry_selector = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _range_shift = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        // Parse table directory entries
        let mut tables = HashMap::new();
        for _ in 0..num_tables {
            let tag_bytes = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
            let checksum = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
            let offset = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
            let length = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

            let tag = Tag::from_bytes(tag_bytes);
            tables.insert(
                tag,
                TableRecord {
                    checksum,
                    offset,
                    length,
                },
            );
        }

        Ok(OpenTypeFont { data, tables })
    }

    /// Get the number of tables in this font
    pub fn table_count(&self) -> usize {
        self.tables.len()
    }

    /// Check if a table exists
    pub fn has_table(&self, tag: Tag) -> bool {
        self.tables.contains_key(&tag)
    }

    /// Get raw table data
    pub fn get_table(&self, tag: Tag) -> Option<&[u8]> {
        self.tables.get(&tag).map(|record| {
            let start = record.offset as usize;
            let end = start + record.length as usize;
            &self.data[start..end]
        })
    }

    /// Get font metrics
    pub fn get_metrics(&self) -> FontMetrics {
        // Parse head table for units_per_em
        let units_per_em = self
            .get_table("head".parse().unwrap())
            .and_then(|data| {
                if data.len() >= 18 {
                    let mut cursor = Cursor::new(data);
                    cursor.set_position(18); // units_per_em is at offset 18
                    cursor.read_u16::<BigEndian>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(1000); // Default value if head table missing or invalid

        // Parse hhea table for ascender, descender, line_gap
        let (ascender, descender, line_gap) = self
            .get_table("hhea".parse().unwrap())
            .and_then(|data| {
                if data.len() >= 12 {
                    let mut cursor = Cursor::new(data);
                    cursor.set_position(4); // ascender is at offset 4
                    let asc = cursor.read_i16::<BigEndian>().ok()?;
                    let desc = cursor.read_i16::<BigEndian>().ok()?;
                    let gap = cursor.read_i16::<BigEndian>().ok()?;
                    Some((asc, desc, gap))
                } else {
                    None
                }
            })
            .unwrap_or((800, -200, 0)); // Default values

        FontMetrics {
            units_per_em,
            ascender,
            descender,
            line_gap,
        }
    }

    /// Get character mapping table
    pub fn get_cmap(&self) -> Option<CMapTable> {
        // Stub implementation - returns empty cmap
        // Full implementation would parse the cmap table
        Some(CMapTable {
            mappings: HashMap::new(),
        })
    }

    /// Get glyph outline
    pub fn get_glyph_outline(&self, _glyph_id: GlyphId) -> Option<GlyphOutline> {
        // Stub implementation - returns None
        // Full implementation would parse glyf table
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_from_str_basic() {
        let tag: Result<Tag, _> = "cmap".parse();
        assert!(tag.is_ok());
    }

    #[test]
    fn test_tag_from_str_invalid_length() {
        let r1: Result<Tag, _> = "".parse();
        let r2: Result<Tag, _> = "abc".parse();
        let r3: Result<Tag, _> = "abcde".parse();
        assert!(r1.is_err());
        assert!(r2.is_err());
        assert!(r3.is_err());
    }

    #[test]
    fn test_tag_from_bytes_to_bytes() {
        let tag = Tag::from_bytes(0x636D6170);
        assert_eq!(tag.to_bytes(), 0x636D6170);
    }

    #[test]
    fn test_tag_display() {
        let tag: Tag = "head".parse().unwrap();
        assert_eq!(format!("{}", tag), "head");
    }
}
