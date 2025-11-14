//! Unit tests for OpenTypeFont

use font_parser::{OpenTypeFont, ParseError, Tag};

#[test]
fn test_opentype_font_parse_empty_data() {
    // Given empty byte data
    // When parsing as OpenType font
    // Then it should return an error
    let data = vec![];
    let result = OpenTypeFont::parse(data);
    assert!(result.is_err());
}

#[test]
fn test_opentype_font_parse_invalid_format() {
    // Given data with invalid sfnt version
    // When parsing as OpenType font
    // Then it should return InvalidFormat error
    let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0];
    let result = OpenTypeFont::parse(data);
    assert!(result.is_err());
    match result {
        Err(ParseError::InvalidFormat) => (),
        _ => panic!("Expected InvalidFormat error"),
    }
}

#[test]
fn test_opentype_font_parse_truetype_version() {
    // Given data with valid TrueType version (0x00010000)
    // When parsing as OpenType font
    // Then it should succeed
    let mut data = vec![
        0x00, 0x01, 0x00, 0x00, // sfnt version (TrueType)
        0x00, 0x01, // numTables
        0x00, 0x10, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
        // Table directory entry for 'head' table
        0x68, 0x65, 0x61, 0x64, // tag: "head"
        0x00, 0x00, 0x00, 0x00, // checksum
        0x00, 0x00, 0x00, 0x20, // offset
        0x00, 0x00, 0x00, 0x10, // length
    ];
    // Add some dummy data at offset 0x20
    data.resize(0x30, 0);

    let result = OpenTypeFont::parse(data);
    assert!(result.is_ok());
}

#[test]
fn test_opentype_font_parse_opentype_version() {
    // Given data with valid OpenType version (0x4F54544F = 'OTTO')
    // When parsing as OpenType font
    // Then it should succeed
    let mut data = vec![
        0x4F, 0x54, 0x54, 0x4F, // sfnt version (OpenType)
        0x00, 0x01, // numTables
        0x00, 0x10, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
        // Table directory entry
        0x43, 0x46, 0x46, 0x20, // tag: "CFF "
        0x00, 0x00, 0x00, 0x00, // checksum
        0x00, 0x00, 0x00, 0x20, // offset
        0x00, 0x00, 0x00, 0x10, // length
    ];
    data.resize(0x30, 0);

    let result = OpenTypeFont::parse(data);
    assert!(result.is_ok());
}

#[test]
fn test_opentype_font_parse_too_short() {
    // Given data shorter than minimum required
    // When parsing as OpenType font
    // Then it should return an error
    let data = vec![0x00, 0x01, 0x00, 0x00];
    let result = OpenTypeFont::parse(data);
    assert!(result.is_err());
}

#[test]
fn test_opentype_font_parse_multiple_tables() {
    // Given font data with multiple tables
    // When parsing as OpenType font
    // Then it should parse all tables
    let mut data = vec![
        0x00, 0x01, 0x00, 0x00, // sfnt version
        0x00, 0x03, // numTables: 3
        0x00, 0x30, // searchRange
        0x00, 0x01, // entrySelector
        0x00, 0x00, // rangeShift
        // Table 1: 'head'
        0x68, 0x65, 0x61, 0x64, // tag
        0x00, 0x00, 0x00, 0x00, // checksum
        0x00, 0x00, 0x00, 0x40, // offset
        0x00, 0x00, 0x00, 0x10, // length
        // Table 2: 'cmap'
        0x63, 0x6D, 0x61, 0x70, // tag
        0x00, 0x00, 0x00, 0x00, // checksum
        0x00, 0x00, 0x00, 0x50, // offset
        0x00, 0x00, 0x00, 0x08, // length
        // Table 3: 'glyf'
        0x67, 0x6C, 0x79, 0x66, // tag
        0x00, 0x00, 0x00, 0x00, // checksum
        0x00, 0x00, 0x00, 0x58, // offset
        0x00, 0x00, 0x00, 0x20, // length
    ];
    data.resize(0x78, 0);

    let result = OpenTypeFont::parse(data);
    assert!(result.is_ok());
    let font = result.unwrap();

    // Verify table count
    assert_eq!(font.table_count(), 3);
}

#[test]
fn test_opentype_font_has_table() {
    // Given a parsed font
    // When checking for existing table
    // Then it should return true
    let mut data = vec![
        0x00, 0x01, 0x00, 0x00, // sfnt version
        0x00, 0x01, // numTables
        0x00, 0x10, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
        0x68, 0x65, 0x61, 0x64, // tag: "head"
        0x00, 0x00, 0x00, 0x00, // checksum
        0x00, 0x00, 0x00, 0x20, // offset
        0x00, 0x00, 0x00, 0x10, // length
    ];
    data.resize(0x30, 0);

    let font = OpenTypeFont::parse(data).unwrap();
    let head_tag: Tag = "head".parse().unwrap();
    assert!(font.has_table(head_tag));

    let missing_tag: Tag = "maxp".parse().unwrap();
    assert!(!font.has_table(missing_tag));
}
