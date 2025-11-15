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

    // Standard variation axis tags
    /// Weight axis tag ('wght')
    pub const WEIGHT: Tag = Tag(*b"wght");
    /// Width axis tag ('wdth')
    pub const WIDTH: Tag = Tag(*b"wdth");
    /// Slant axis tag ('slnt')
    pub const SLANT: Tag = Tag(*b"slnt");
    /// Optical size axis tag ('opsz')
    pub const OPTICAL_SIZE: Tag = Tag(*b"opsz");
    /// Italic axis tag ('ital')
    pub const ITALIC: Tag = Tag(*b"ital");
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl OpenTypeFont {
    /// Parse an OpenType/TrueType font from bytes
    ///
    /// This method supports multiple font formats:
    /// - TrueType (0x00010000)
    /// - OpenType/CFF (0x4F54544F or 'OTTO')
    /// - WOFF (0x774F4646 or 'wOFF')
    /// - WOFF2 (0x774F4632 or 'wOF2')
    pub fn parse(data: Vec<u8>) -> Result<Self, ParseError> {
        if data.len() < 12 {
            return Err(ParseError::CorruptedData("Font data too short".to_string()));
        }

        // Check signature to determine format
        let signature = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        // Handle WOFF and WOFF2 formats by decompressing first
        let data = match signature {
            0x774F4646 => {
                // WOFF - decompress to TTF/OTF
                let woff = crate::woff::WoffFont::parse(&data)?;
                woff.ttf_data
            }
            0x774F4632 => {
                // WOFF2 - decompress to TTF/OTF
                let woff2 = crate::woff2::Woff2Font::parse(&data)?;
                woff2.ttf_data
            }
            _ => data, // TTF/OTF - use as-is
        };

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

    /// Check if this is a variable font
    ///
    /// Returns true if the font contains an 'fvar' table, which indicates
    /// it supports OpenType Font Variations.
    pub fn is_variable(&self) -> bool {
        self.has_table("fvar".parse().unwrap())
    }

    /// Get font variations table (fvar)
    ///
    /// Returns the parsed fvar table if this is a variable font, or None otherwise.
    /// The fvar table defines the available variation axes and named instances.
    pub fn get_fvar(&self) -> Option<crate::variable_fonts::FvarTable> {
        let data = self.get_table("fvar".parse().unwrap())?;
        crate::variable_fonts::FvarTable::parse(data).ok()
    }

    /// Get axis variations table (avar)
    ///
    /// Returns the parsed avar table if present, or None otherwise.
    /// The avar table defines non-linear axis value mappings.
    pub fn get_avar(&self) -> Option<crate::variable_fonts::AvarTable> {
        let fvar = self.get_fvar()?;
        let data = self.get_table("avar".parse().unwrap())?;
        crate::variable_fonts::AvarTable::parse(data, fvar.axes.len()).ok()
    }

    /// Get available variation axes
    ///
    /// Returns a list of all variation axes defined in the font's fvar table.
    /// Each axis specifies a design dimension (e.g., weight, width) with min/max bounds.
    pub fn get_variation_axes(&self) -> Vec<crate::variable_fonts::VariationAxis> {
        self.get_fvar().map(|fvar| fvar.axes).unwrap_or_default()
    }

    /// Get named instances
    ///
    /// Returns a list of all named instances defined in the font's fvar table.
    /// Named instances are predefined coordinate sets (e.g., "Bold", "Light").
    pub fn get_named_instances(&self) -> Vec<crate::variable_fonts::NamedInstance> {
        self.get_fvar()
            .map(|fvar| fvar.instances)
            .unwrap_or_default()
    }

    /// Validate variation coordinates
    ///
    /// Checks that all coordinates are within the valid range for their respective axes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The font is not a variable font
    /// - Any coordinate value is outside the axis bounds
    pub fn validate_coordinates(
        &self,
        coords: &crate::variable_fonts::VariationCoordinates,
    ) -> Result<(), ParseError> {
        let fvar = self
            .get_fvar()
            .ok_or_else(|| ParseError::CorruptedData("Not a variable font".to_string()))?;

        // Validate each coordinate is within bounds
        for (tag, value) in &coords.values {
            if let Some(axis) = fvar.get_axis(*tag) {
                if *value < axis.min_value || *value > axis.max_value {
                    return Err(ParseError::CorruptedData(format!(
                        "Coordinate {} out of range [{}, {}] for axis {}",
                        value, axis.min_value, axis.max_value, tag
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if this font has color glyphs
    ///
    /// Returns true if the font contains any color font tables (COLR, CBDT, sbix, or SVG).
    pub fn is_color_font(&self) -> bool {
        self.has_table("COLR".parse().unwrap())
            || self.has_table("CBDT".parse().unwrap())
            || self.has_table("sbix".parse().unwrap())
            || self.has_table("SVG ".parse().unwrap())
    }

    /// Get supported color formats
    ///
    /// Returns a list of all color font formats supported by this font.
    pub fn get_color_formats(&self) -> Vec<crate::color_fonts::ColorFormat> {
        let mut formats = Vec::new();

        if self.has_table("COLR".parse().unwrap()) {
            formats.push(crate::color_fonts::ColorFormat::ColrCpal);
        }
        if self.has_table("CBDT".parse().unwrap()) {
            formats.push(crate::color_fonts::ColorFormat::Cbdt);
        }
        if self.has_table("sbix".parse().unwrap()) {
            formats.push(crate::color_fonts::ColorFormat::Sbix);
        }
        if self.has_table("SVG ".parse().unwrap()) {
            formats.push(crate::color_fonts::ColorFormat::Svg);
        }

        formats
    }

    /// Get color palette table (CPAL)
    ///
    /// Returns the parsed CPAL table if present, which defines color palettes
    /// used by the COLR table for layered color glyphs.
    pub fn get_cpal(&self) -> Option<crate::color_fonts::CpalTable> {
        let data = self.get_table("CPAL".parse().unwrap())?;
        crate::color_fonts::CpalTable::parse(data).ok()
    }

    /// Get color layers table (COLR)
    ///
    /// Returns the parsed COLR table if present, which defines layered color glyphs
    /// using palette colors from the CPAL table.
    pub fn get_colr(&self) -> Option<crate::color_fonts::ColrTable> {
        let data = self.get_table("COLR".parse().unwrap())?;
        crate::color_fonts::ColrTable::parse(data).ok()
    }

    /// Get color bitmap table (CBDT)
    ///
    /// Returns the parsed CBDT table if present, which contains embedded color
    /// bitmap data for glyphs (commonly used for emoji).
    pub fn get_cbdt(&self) -> Option<crate::color_fonts::CbdtTable> {
        let data = self.get_table("CBDT".parse().unwrap())?;
        crate::color_fonts::CbdtTable::parse(data).ok()
    }

    /// Get SVG table
    ///
    /// Returns the parsed SVG table if present, which contains SVG glyph definitions.
    pub fn get_svg(&self) -> Option<crate::color_fonts::SvgTable> {
        let data = self.get_table("SVG ".parse().unwrap())?;
        crate::color_fonts::SvgTable::parse(data).ok()
    }

    /// Check if a specific glyph has color layers
    ///
    /// Returns true if the glyph has color layers defined in the COLR table.
    pub fn has_color_layers(&self, glyph_id: GlyphId) -> bool {
        if let Some(colr) = self.get_colr() {
            return colr.is_color_glyph(glyph_id);
        }
        false
    }

    /// Get color layers for a glyph
    ///
    /// Returns the color layers for a specific glyph if defined in the COLR table.
    pub fn get_color_layers(&self, glyph_id: GlyphId) -> Option<Vec<crate::color_fonts::Layer>> {
        let colr = self.get_colr()?;
        colr.get_layers(glyph_id).cloned()
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
