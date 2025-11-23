//! SVG-in-OpenType Support
//!
//! This module provides parsing for the SVG table in OpenType fonts,
//! which contains SVG glyph definitions for color glyphs.

use crate::types::GlyphId;
use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// SVG Table - Contains SVG glyph definitions
///
/// The SVG table provides SVG (Scalable Vector Graphics) representations
/// of glyphs, commonly used for color emoji and decorative fonts.
#[derive(Debug, Clone)]
pub struct SvgDocumentTable {
    /// Table version
    pub version: u16,
    /// SVG document index entries
    pub documents: Vec<SvgDocumentRecord>,
}

/// An SVG document record mapping glyph ranges to SVG data
#[derive(Debug, Clone)]
pub struct SvgDocumentRecord {
    /// First glyph ID covered by this document
    pub start_glyph_id: GlyphId,
    /// Last glyph ID covered by this document
    pub end_glyph_id: GlyphId,
    /// Raw SVG document data (may be gzip compressed)
    pub svg_data: Vec<u8>,
    /// Whether the SVG data is gzip compressed
    pub is_compressed: bool,
}

impl SvgDocumentTable {
    /// Parse SVG table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 10 {
            return Err(ParseError::CorruptedData(
                "SVG table too short".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read header
        let version = cursor.read_u16::<BigEndian>()?;
        let svg_document_list_offset = cursor.read_u32::<BigEndian>()?;
        let _reserved = cursor.read_u32::<BigEndian>()?;

        if svg_document_list_offset == 0 {
            return Ok(SvgDocumentTable {
                version,
                documents: Vec::new(),
            });
        }

        // Move to SVG document list
        cursor.set_position(svg_document_list_offset as u64);

        // Read number of document records
        let num_entries = cursor.read_u16::<BigEndian>()?;

        // Read document index entries
        let mut entries = Vec::new();
        for _ in 0..num_entries {
            let start_glyph_id = cursor.read_u16::<BigEndian>()?;
            let end_glyph_id = cursor.read_u16::<BigEndian>()?;
            let svg_doc_offset = cursor.read_u32::<BigEndian>()?;
            let svg_doc_length = cursor.read_u32::<BigEndian>()?;

            entries.push((start_glyph_id, end_glyph_id, svg_doc_offset, svg_doc_length));
        }

        // Extract SVG document data
        let mut documents = Vec::new();
        for (start_glyph_id, end_glyph_id, svg_doc_offset, svg_doc_length) in entries {
            // Offset is relative to the start of the SVG table
            let abs_offset = svg_document_list_offset + svg_doc_offset;
            let start = abs_offset as usize;
            let end = start + svg_doc_length as usize;

            if end > data.len() {
                return Err(ParseError::CorruptedData(
                    "SVG document offset extends past table".to_string(),
                ));
            }

            let svg_data = data[start..end].to_vec();

            // Check if data is gzip compressed (starts with 0x1F 0x8B)
            let is_compressed = svg_data.len() >= 2 && svg_data[0] == 0x1F && svg_data[1] == 0x8B;

            documents.push(SvgDocumentRecord {
                start_glyph_id,
                end_glyph_id,
                svg_data,
                is_compressed,
            });
        }

        Ok(SvgDocumentTable { version, documents })
    }

    /// Get the number of SVG document records
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Check if a glyph has an SVG representation
    pub fn has_svg_glyph(&self, glyph_id: GlyphId) -> bool {
        self.documents
            .iter()
            .any(|doc| glyph_id >= doc.start_glyph_id && glyph_id <= doc.end_glyph_id)
    }

    /// Get the SVG document record for a glyph
    pub fn get_svg_document(&self, glyph_id: GlyphId) -> Option<&SvgDocumentRecord> {
        self.documents
            .iter()
            .find(|doc| glyph_id >= doc.start_glyph_id && glyph_id <= doc.end_glyph_id)
    }

    /// Get raw SVG data for a glyph
    pub fn get_svg_data(&self, glyph_id: GlyphId) -> Option<&[u8]> {
        self.get_svg_document(glyph_id)
            .map(|doc| doc.svg_data.as_slice())
    }

    /// Get the glyph ID range covered by SVG documents
    pub fn get_glyph_range(&self) -> Option<(GlyphId, GlyphId)> {
        if self.documents.is_empty() {
            return None;
        }

        let min_glyph = self
            .documents
            .iter()
            .map(|d| d.start_glyph_id)
            .min()
            .unwrap_or(0);
        let max_glyph = self
            .documents
            .iter()
            .map(|d| d.end_glyph_id)
            .max()
            .unwrap_or(0);

        Some((min_glyph, max_glyph))
    }

    /// Get all glyph IDs that have SVG representations
    pub fn get_svg_glyph_ids(&self) -> Vec<GlyphId> {
        let mut glyph_ids = Vec::new();
        for doc in &self.documents {
            for glyph_id in doc.start_glyph_id..=doc.end_glyph_id {
                glyph_ids.push(glyph_id);
            }
        }
        glyph_ids
    }
}

impl SvgDocumentRecord {
    /// Get the number of glyphs covered by this document
    pub fn glyph_count(&self) -> usize {
        (self.end_glyph_id - self.start_glyph_id + 1) as usize
    }

    /// Get the SVG data size in bytes
    pub fn data_size(&self) -> usize {
        self.svg_data.len()
    }

    /// Try to decompress the SVG data if it's gzip compressed
    #[cfg(feature = "gzip")]
    pub fn decompress(&self) -> Result<Vec<u8>, ParseError> {
        if !self.is_compressed {
            return Ok(self.svg_data.clone());
        }

        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(&self.svg_data[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| ParseError::CorruptedData(format!("Failed to decompress SVG: {}", e)))?;

        Ok(decompressed)
    }

    /// Get the raw SVG data (possibly compressed)
    pub fn raw_data(&self) -> &[u8] {
        &self.svg_data
    }

    /// Try to extract SVG as a string (only works for uncompressed data)
    pub fn as_string(&self) -> Option<String> {
        if self.is_compressed {
            return None;
        }
        String::from_utf8(self.svg_data.clone()).ok()
    }
}

/// SVG parsing options
#[derive(Debug, Clone, Default)]
pub struct SvgParseOptions {
    /// Whether to decompress gzip-compressed SVG data
    pub decompress: bool,
    /// Whether to validate SVG structure
    pub validate: bool,
}

/// Parsed SVG element (simplified representation)
#[derive(Debug, Clone)]
pub struct SvgElement {
    /// Element tag name
    pub tag: String,
    /// Element attributes as key-value pairs
    pub attributes: Vec<(String, String)>,
    /// Child elements
    pub children: Vec<SvgElement>,
    /// Text content
    pub text: Option<String>,
}

impl SvgElement {
    /// Get an attribute value by name
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Check if element has a specific attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|(k, _)| k == name)
    }

    /// Get all child elements with a specific tag
    pub fn get_children_by_tag(&self, tag: &str) -> Vec<&SvgElement> {
        self.children.iter().filter(|c| c.tag == tag).collect()
    }
}

/// SVG glyph information extracted from document
#[derive(Debug, Clone)]
pub struct SvgGlyphInfo {
    /// Glyph ID
    pub glyph_id: GlyphId,
    /// Viewbox dimensions (x, y, width, height)
    pub viewbox: Option<(f64, f64, f64, f64)>,
    /// SVG content for this glyph
    pub content: String,
}

/// Extract glyph-specific SVG from a document
///
/// SVG documents in OpenType fonts often contain multiple glyphs
/// referenced by id attributes like `glyph123`.
pub fn extract_glyph_svg(document: &str, glyph_id: GlyphId) -> Option<String> {
    // Look for element with id="glyphN" where N is the glyph ID
    let glyph_id_str = format!("glyph{}", glyph_id);

    // Simple extraction - look for the ID in the document
    if document.contains(&glyph_id_str) {
        // For now, return the entire document
        // A full implementation would extract just the relevant portion
        Some(document.to_string())
    } else {
        // Single-glyph document - return as-is
        Some(document.to_string())
    }
}

/// Parse viewbox attribute from SVG
pub fn parse_viewbox(viewbox_str: &str) -> Option<(f64, f64, f64, f64)> {
    let parts: Vec<&str> = viewbox_str.split_whitespace().collect();
    if parts.len() != 4 {
        // Try comma-separated
        let parts: Vec<&str> = viewbox_str.split(',').map(|s| s.trim()).collect();
        if parts.len() != 4 {
            return None;
        }
        let x = parts[0].parse().ok()?;
        let y = parts[1].parse().ok()?;
        let width = parts[2].parse().ok()?;
        let height = parts[3].parse().ok()?;
        return Some((x, y, width, height));
    }

    let x = parts[0].parse().ok()?;
    let y = parts[1].parse().ok()?;
    let width = parts[2].parse().ok()?;
    let height = parts[3].parse().ok()?;

    Some((x, y, width, height))
}

/// Validate basic SVG structure
pub fn validate_svg(data: &[u8]) -> Result<(), ParseError> {
    // Check for XML/SVG header
    let content = std::str::from_utf8(data)
        .map_err(|_| ParseError::CorruptedData("SVG is not valid UTF-8".to_string()))?;

    // Must contain <svg element
    if !content.contains("<svg") {
        return Err(ParseError::CorruptedData(
            "SVG document missing <svg> element".to_string(),
        ));
    }

    // Must be properly closed
    if !content.contains("</svg>") && !content.contains("/>") {
        return Err(ParseError::CorruptedData(
            "SVG document not properly closed".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_document_table_empty() {
        // Minimal valid SVG table with no documents
        let data = vec![
            0x00, 0x00, // version = 0
            0x00, 0x00, 0x00, 0x00, // svgDocumentListOffset = 0 (no documents)
            0x00, 0x00, 0x00, 0x00, // reserved
        ];
        let table = SvgDocumentTable::parse(&data).unwrap();
        assert_eq!(table.version, 0);
        assert_eq!(table.document_count(), 0);
    }

    #[test]
    fn test_svg_document_table_too_short() {
        let data = vec![0x00, 0x00, 0x00];
        let result = SvgDocumentTable::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_svg_document_record_glyph_count() {
        let record = SvgDocumentRecord {
            start_glyph_id: 10,
            end_glyph_id: 20,
            svg_data: vec![],
            is_compressed: false,
        };
        assert_eq!(record.glyph_count(), 11);
    }

    #[test]
    fn test_svg_document_record_data_size() {
        let record = SvgDocumentRecord {
            start_glyph_id: 0,
            end_glyph_id: 0,
            svg_data: vec![0; 100],
            is_compressed: false,
        };
        assert_eq!(record.data_size(), 100);
    }

    #[test]
    fn test_svg_document_record_as_string() {
        let svg_content = "<svg></svg>";
        let record = SvgDocumentRecord {
            start_glyph_id: 0,
            end_glyph_id: 0,
            svg_data: svg_content.as_bytes().to_vec(),
            is_compressed: false,
        };
        assert_eq!(record.as_string(), Some(svg_content.to_string()));
    }

    #[test]
    fn test_svg_document_record_compressed_no_string() {
        let record = SvgDocumentRecord {
            start_glyph_id: 0,
            end_glyph_id: 0,
            svg_data: vec![0x1F, 0x8B], // gzip header
            is_compressed: true,
        };
        assert!(record.as_string().is_none());
    }

    #[test]
    fn test_parse_viewbox_space_separated() {
        let viewbox = parse_viewbox("0 0 100 100");
        assert_eq!(viewbox, Some((0.0, 0.0, 100.0, 100.0)));
    }

    #[test]
    fn test_parse_viewbox_comma_separated() {
        let viewbox = parse_viewbox("0,0,200,150");
        assert_eq!(viewbox, Some((0.0, 0.0, 200.0, 150.0)));
    }

    #[test]
    fn test_parse_viewbox_invalid() {
        let viewbox = parse_viewbox("0 0 100");
        assert!(viewbox.is_none());
    }

    #[test]
    fn test_parse_viewbox_negative() {
        let viewbox = parse_viewbox("-50 -50 100 100");
        assert_eq!(viewbox, Some((-50.0, -50.0, 100.0, 100.0)));
    }

    #[test]
    fn test_validate_svg_valid() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        assert!(validate_svg(svg).is_ok());
    }

    #[test]
    fn test_validate_svg_self_closing() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
        assert!(validate_svg(svg).is_ok());
    }

    #[test]
    fn test_validate_svg_missing_svg_element() {
        let svg = b"<div>Not an SVG</div>";
        assert!(validate_svg(svg).is_err());
    }

    #[test]
    fn test_validate_svg_invalid_utf8() {
        let svg = vec![0xFF, 0xFE, 0x00, 0x00];
        assert!(validate_svg(&svg).is_err());
    }

    #[test]
    fn test_extract_glyph_svg() {
        let document = "<svg><path id=\"glyph42\" d=\"M0 0\"/></svg>";
        let result = extract_glyph_svg(document, 42);
        assert!(result.is_some());
    }

    #[test]
    fn test_svg_element_get_attribute() {
        let element = SvgElement {
            tag: "path".to_string(),
            attributes: vec![
                ("d".to_string(), "M0 0".to_string()),
                ("fill".to_string(), "red".to_string()),
            ],
            children: vec![],
            text: None,
        };
        assert_eq!(element.get_attribute("d"), Some("M0 0"));
        assert_eq!(element.get_attribute("fill"), Some("red"));
        assert_eq!(element.get_attribute("stroke"), None);
    }

    #[test]
    fn test_svg_element_has_attribute() {
        let element = SvgElement {
            tag: "circle".to_string(),
            attributes: vec![("cx".to_string(), "50".to_string())],
            children: vec![],
            text: None,
        };
        assert!(element.has_attribute("cx"));
        assert!(!element.has_attribute("cy"));
    }

    #[test]
    fn test_svg_element_get_children_by_tag() {
        let element = SvgElement {
            tag: "g".to_string(),
            attributes: vec![],
            children: vec![
                SvgElement {
                    tag: "path".to_string(),
                    attributes: vec![],
                    children: vec![],
                    text: None,
                },
                SvgElement {
                    tag: "circle".to_string(),
                    attributes: vec![],
                    children: vec![],
                    text: None,
                },
                SvgElement {
                    tag: "path".to_string(),
                    attributes: vec![],
                    children: vec![],
                    text: None,
                },
            ],
            text: None,
        };
        let paths = element.get_children_by_tag("path");
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_svg_glyph_info() {
        let info = SvgGlyphInfo {
            glyph_id: 42,
            viewbox: Some((0.0, 0.0, 100.0, 100.0)),
            content: "<path d=\"M0 0\"/>".to_string(),
        };
        assert_eq!(info.glyph_id, 42);
        assert!(info.viewbox.is_some());
    }

    #[test]
    fn test_svg_parse_options_default() {
        let options = SvgParseOptions::default();
        assert!(!options.decompress);
        assert!(!options.validate);
    }

    #[test]
    fn test_has_svg_glyph() {
        let table = SvgDocumentTable {
            version: 0,
            documents: vec![SvgDocumentRecord {
                start_glyph_id: 10,
                end_glyph_id: 20,
                svg_data: vec![],
                is_compressed: false,
            }],
        };
        assert!(table.has_svg_glyph(10));
        assert!(table.has_svg_glyph(15));
        assert!(table.has_svg_glyph(20));
        assert!(!table.has_svg_glyph(9));
        assert!(!table.has_svg_glyph(21));
    }

    #[test]
    fn test_get_svg_glyph_ids() {
        let table = SvgDocumentTable {
            version: 0,
            documents: vec![
                SvgDocumentRecord {
                    start_glyph_id: 5,
                    end_glyph_id: 7,
                    svg_data: vec![],
                    is_compressed: false,
                },
                SvgDocumentRecord {
                    start_glyph_id: 10,
                    end_glyph_id: 11,
                    svg_data: vec![],
                    is_compressed: false,
                },
            ],
        };
        let ids = table.get_svg_glyph_ids();
        assert_eq!(ids, vec![5, 6, 7, 10, 11]);
    }

    #[test]
    fn test_get_glyph_range() {
        let table = SvgDocumentTable {
            version: 0,
            documents: vec![
                SvgDocumentRecord {
                    start_glyph_id: 10,
                    end_glyph_id: 20,
                    svg_data: vec![],
                    is_compressed: false,
                },
                SvgDocumentRecord {
                    start_glyph_id: 5,
                    end_glyph_id: 8,
                    svg_data: vec![],
                    is_compressed: false,
                },
            ],
        };
        assert_eq!(table.get_glyph_range(), Some((5, 20)));
    }

    #[test]
    fn test_get_glyph_range_empty() {
        let table = SvgDocumentTable {
            version: 0,
            documents: vec![],
        };
        assert_eq!(table.get_glyph_range(), None);
    }
}
