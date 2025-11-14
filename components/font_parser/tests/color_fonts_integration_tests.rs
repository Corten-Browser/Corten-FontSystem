//! Integration tests for color fonts support

use font_parser::{ColorFormat, OpenTypeFont, ParseError};

/// Create a minimal valid OpenType font with COLR and CPAL tables
fn create_font_with_colr_cpal() -> Vec<u8> {
    let mut data = Vec::new();

    // OpenType font header (sfnt version)
    data.extend_from_slice(&0x00010000u32.to_be_bytes()); // TrueType version
    data.extend_from_slice(&3u16.to_be_bytes()); // num_tables (head, COLR, CPAL)
    data.extend_from_slice(&48u16.to_be_bytes()); // search_range
    data.extend_from_slice(&1u16.to_be_bytes()); // entry_selector
    data.extend_from_slice(&0u16.to_be_bytes()); // range_shift

    // Table directory entries (3 tables)
    // Each entry is 16 bytes: tag(4) + checksum(4) + offset(4) + length(4)

    // 1. head table (required)
    let head_tag = u32::from_be_bytes(*b"head");
    let head_offset = 60u32; // After 3 table entries (12 + 3*16 = 60)
    let head_length = 54u32;
    data.extend_from_slice(&head_tag.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes()); // checksum
    data.extend_from_slice(&head_offset.to_be_bytes());
    data.extend_from_slice(&head_length.to_be_bytes());

    // 2. COLR table
    let colr_tag = u32::from_be_bytes(*b"COLR");
    let colr_offset = head_offset + head_length;
    let colr_length = 28u32;
    data.extend_from_slice(&colr_tag.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes()); // checksum
    data.extend_from_slice(&colr_offset.to_be_bytes());
    data.extend_from_slice(&colr_length.to_be_bytes());

    // 3. CPAL table
    let cpal_tag = u32::from_be_bytes(*b"CPAL");
    let cpal_offset = colr_offset + colr_length;
    let cpal_length = 22u32;
    data.extend_from_slice(&cpal_tag.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes()); // checksum
    data.extend_from_slice(&cpal_offset.to_be_bytes());
    data.extend_from_slice(&cpal_length.to_be_bytes());

    // Pad to head table offset
    while data.len() < head_offset as usize {
        data.push(0);
    }

    // head table (minimal)
    data.extend_from_slice(&0x00010000u32.to_be_bytes()); // version
    data.extend_from_slice(&0x00010000u32.to_be_bytes()); // font revision
    data.extend_from_slice(&0u32.to_be_bytes()); // checksum adjustment
    data.extend_from_slice(&0x5F0F3CF5u32.to_be_bytes()); // magic number
    data.extend_from_slice(&0u16.to_be_bytes()); // flags
    data.extend_from_slice(&1000u16.to_be_bytes()); // units_per_em
    data.extend_from_slice(&[0u8; 16]); // created (8) + modified (8)
    data.extend_from_slice(&0i16.to_be_bytes()); // xMin
    data.extend_from_slice(&0i16.to_be_bytes()); // yMin
    data.extend_from_slice(&1000i16.to_be_bytes()); // xMax
    data.extend_from_slice(&1000i16.to_be_bytes()); // yMax
    data.extend_from_slice(&0u16.to_be_bytes()); // macStyle
    data.extend_from_slice(&8u16.to_be_bytes()); // lowestRecPPEM
    data.extend_from_slice(&2i16.to_be_bytes()); // fontDirectionHint
    data.extend_from_slice(&0i16.to_be_bytes()); // indexToLocFormat
    data.extend_from_slice(&0i16.to_be_bytes()); // glyphDataFormat

    // Pad to COLR offset
    while data.len() < colr_offset as usize {
        data.push(0);
    }

    // COLR table
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&1u16.to_be_bytes()); // num_base_glyph_records
    data.extend_from_slice(&14u32.to_be_bytes()); // base_glyph_records_offset
    data.extend_from_slice(&20u32.to_be_bytes()); // layer_records_offset
    data.extend_from_slice(&2u16.to_be_bytes()); // num_layer_records

    // Base glyph record
    data.extend_from_slice(&42u16.to_be_bytes()); // glyph_id
    data.extend_from_slice(&0u16.to_be_bytes()); // first_layer_index
    data.extend_from_slice(&2u16.to_be_bytes()); // num_layers

    // Layer records
    data.extend_from_slice(&100u16.to_be_bytes()); // layer 0 glyph_id
    data.extend_from_slice(&0u16.to_be_bytes()); // layer 0 palette_index
    data.extend_from_slice(&101u16.to_be_bytes()); // layer 1 glyph_id
    data.extend_from_slice(&1u16.to_be_bytes()); // layer 1 palette_index

    // Pad to CPAL offset
    while data.len() < cpal_offset as usize {
        data.push(0);
    }

    // CPAL table
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&2u16.to_be_bytes()); // num_palette_entries
    data.extend_from_slice(&1u16.to_be_bytes()); // num_palettes
    data.extend_from_slice(&2u16.to_be_bytes()); // num_color_records
    data.extend_from_slice(&14u32.to_be_bytes()); // color_records_array_offset

    // Palette index
    data.extend_from_slice(&0u16.to_be_bytes());

    // Color records
    data.extend_from_slice(&[0, 0, 255, 255]); // Red
    data.extend_from_slice(&[0, 255, 0, 255]); // Green

    data
}

#[test]
fn test_parse_font_with_colr_cpal() {
    let font_data = create_font_with_colr_cpal();
    let font = OpenTypeFont::parse(font_data).expect("Failed to parse font");

    // Verify color font detection
    assert!(font.is_color_font());

    // Verify color formats
    let formats = font.get_color_formats();
    assert!(formats.contains(&ColorFormat::ColrCpal));
    assert!(!formats.contains(&ColorFormat::Cbdt));
    assert!(!formats.contains(&ColorFormat::Sbix));
    assert!(!formats.contains(&ColorFormat::Svg));
}

#[test]
fn test_get_cpal_from_font() {
    let font_data = create_font_with_colr_cpal();
    let font = OpenTypeFont::parse(font_data).expect("Failed to parse font");

    let cpal = font.get_cpal().expect("CPAL table should exist");

    assert_eq!(cpal.palette_count(), 1);

    let palette = cpal.default_palette().unwrap();
    assert_eq!(palette.len(), 2);

    // Check colors
    assert_eq!(palette[0].red, 255);
    assert_eq!(palette[0].green, 0);
    assert_eq!(palette[0].blue, 0);

    assert_eq!(palette[1].red, 0);
    assert_eq!(palette[1].green, 255);
    assert_eq!(palette[1].blue, 0);
}

#[test]
fn test_get_colr_from_font() {
    let font_data = create_font_with_colr_cpal();
    let font = OpenTypeFont::parse(font_data).expect("Failed to parse font");

    let colr = font.get_colr().expect("COLR table should exist");

    assert_eq!(colr.color_glyph_count(), 1);
    assert!(colr.is_color_glyph(42));
    assert!(!colr.is_color_glyph(43));
}

#[test]
fn test_has_color_layers() {
    let font_data = create_font_with_colr_cpal();
    let font = OpenTypeFont::parse(font_data).expect("Failed to parse font");

    assert!(font.has_color_layers(42));
    assert!(!font.has_color_layers(43));
}

#[test]
fn test_get_color_layers() {
    let font_data = create_font_with_colr_cpal();
    let font = OpenTypeFont::parse(font_data).expect("Failed to parse font");

    let layers = font
        .get_color_layers(42)
        .expect("Glyph 42 should have layers");

    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].glyph_id, 100);
    assert_eq!(layers[0].palette_index, 0);
    assert_eq!(layers[1].glyph_id, 101);
    assert_eq!(layers[1].palette_index, 1);
}

#[test]
fn test_get_color_layers_for_nonexistent_glyph() {
    let font_data = create_font_with_colr_cpal();
    let font = OpenTypeFont::parse(font_data).expect("Failed to parse font");

    let layers = font.get_color_layers(999);
    assert!(layers.is_none());
}

#[test]
fn test_font_without_color_tables() {
    // Create minimal font without color tables
    let mut data = Vec::new();

    // OpenType font header
    data.extend_from_slice(&0x00010000u32.to_be_bytes());
    data.extend_from_slice(&1u16.to_be_bytes()); // num_tables (only head)
    data.extend_from_slice(&16u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());

    // head table entry
    let head_tag = u32::from_be_bytes(*b"head");
    data.extend_from_slice(&head_tag.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(&28u32.to_be_bytes()); // offset
    data.extend_from_slice(&54u32.to_be_bytes()); // length

    // Pad to head offset
    while data.len() < 28 {
        data.push(0);
    }

    // Minimal head table
    data.extend_from_slice(&0x00010000u32.to_be_bytes());
    data.extend_from_slice(&0x00010000u32.to_be_bytes());
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(&0x5F0F3CF5u32.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&1000u16.to_be_bytes());
    data.extend_from_slice(&[0u8; 16]);
    data.extend_from_slice(&0i16.to_be_bytes());
    data.extend_from_slice(&0i16.to_be_bytes());
    data.extend_from_slice(&1000i16.to_be_bytes());
    data.extend_from_slice(&1000i16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&8u16.to_be_bytes());
    data.extend_from_slice(&2i16.to_be_bytes());
    data.extend_from_slice(&0i16.to_be_bytes());
    data.extend_from_slice(&0i16.to_be_bytes());

    let font = OpenTypeFont::parse(data).expect("Failed to parse font");

    assert!(!font.is_color_font());
    assert!(font.get_color_formats().is_empty());
    assert!(font.get_cpal().is_none());
    assert!(font.get_colr().is_none());
    assert!(!font.has_color_layers(42));
}

#[test]
fn test_color_format_equality() {
    assert_eq!(ColorFormat::ColrCpal, ColorFormat::ColrCpal);
    assert_ne!(ColorFormat::ColrCpal, ColorFormat::Cbdt);
}
