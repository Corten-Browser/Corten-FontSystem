//! Unit tests for color fonts support

use font_parser::{
    BaseGlyph, CbdtTable, Color, ColorFormat, ColrTable, CpalTable, Layer, SvgTable,
};
use std::io::Write;

#[test]
fn test_color_from_rgba() {
    let color = Color::from_rgba(255, 128, 64, 192);
    assert_eq!(color.red, 255);
    assert_eq!(color.green, 128);
    assert_eq!(color.blue, 64);
    assert_eq!(color.alpha, 192);
}

#[test]
fn test_color_to_rgba_u32() {
    let color = Color::from_rgba(255, 128, 64, 192);
    let rgba = color.to_rgba_u32();
    // ARGB format: alpha=192(0xC0), red=255(0xFF), green=128(0x80), blue=64(0x40)
    assert_eq!(rgba, 0xC0FF8040);
}

#[test]
fn test_color_equality() {
    let color1 = Color::from_rgba(255, 128, 64, 192);
    let color2 = Color::from_rgba(255, 128, 64, 192);
    let color3 = Color::from_rgba(255, 128, 64, 191);

    assert_eq!(color1, color2);
    assert_ne!(color1, color3);
}

#[test]
fn test_layer_creation() {
    let layer = Layer {
        glyph_id: 42,
        palette_index: 5,
    };
    assert_eq!(layer.glyph_id, 42);
    assert_eq!(layer.palette_index, 5);
}

#[test]
fn test_layer_equality() {
    let layer1 = Layer {
        glyph_id: 42,
        palette_index: 5,
    };
    let layer2 = Layer {
        glyph_id: 42,
        palette_index: 5,
    };
    let layer3 = Layer {
        glyph_id: 43,
        palette_index: 5,
    };

    assert_eq!(layer1, layer2);
    assert_ne!(layer1, layer3);
}

#[test]
fn test_base_glyph_creation() {
    let layers = vec![
        Layer {
            glyph_id: 100,
            palette_index: 0,
        },
        Layer {
            glyph_id: 101,
            palette_index: 1,
        },
    ];
    let base_glyph = BaseGlyph {
        glyph_id: 42,
        layers: layers.clone(),
    };

    assert_eq!(base_glyph.glyph_id, 42);
    assert_eq!(base_glyph.layers.len(), 2);
    assert_eq!(base_glyph.layers[0].glyph_id, 100);
    assert_eq!(base_glyph.layers[1].glyph_id, 101);
}

#[test]
fn test_color_format_variants() {
    // Test all color format variants exist and are distinct
    assert_ne!(ColorFormat::ColrCpal, ColorFormat::Cbdt);
    assert_ne!(ColorFormat::ColrCpal, ColorFormat::Sbix);
    assert_ne!(ColorFormat::ColrCpal, ColorFormat::Svg);
    assert_ne!(ColorFormat::Cbdt, ColorFormat::Sbix);
    assert_ne!(ColorFormat::Cbdt, ColorFormat::Svg);
    assert_ne!(ColorFormat::Sbix, ColorFormat::Svg);
}

#[test]
fn test_cpal_parse_minimal() {
    // Create a minimal valid CPAL table
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&2u16.to_be_bytes()); // num_palette_entries
    data.extend_from_slice(&1u16.to_be_bytes()); // num_palettes
    data.extend_from_slice(&2u16.to_be_bytes()); // num_color_records
    data.extend_from_slice(&14u32.to_be_bytes()); // color_records_array_offset

    // Palette color record indices
    data.extend_from_slice(&0u16.to_be_bytes()); // palette 0 starts at color 0

    // Color records (at offset 14)
    // Color 0: Red
    data.push(0); // blue
    data.push(0); // green
    data.push(255); // red
    data.push(255); // alpha

    // Color 1: Green
    data.push(0); // blue
    data.push(255); // green
    data.push(0); // red
    data.push(255); // alpha

    let cpal = CpalTable::parse(&data).expect("Failed to parse CPAL");

    assert_eq!(cpal.palette_count(), 1);
    assert_eq!(cpal.palettes[0].len(), 2);

    let palette = cpal.default_palette().unwrap();
    assert_eq!(palette[0].red, 255);
    assert_eq!(palette[0].green, 0);
    assert_eq!(palette[0].blue, 0);
    assert_eq!(palette[1].red, 0);
    assert_eq!(palette[1].green, 255);
    assert_eq!(palette[1].blue, 0);
}

#[test]
fn test_cpal_parse_multiple_palettes() {
    // Create CPAL with 2 palettes
    let mut data = Vec::new();

    // Header (12 bytes)
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&2u16.to_be_bytes()); // num_palette_entries
    data.extend_from_slice(&2u16.to_be_bytes()); // num_palettes
    data.extend_from_slice(&4u16.to_be_bytes()); // num_color_records
    data.extend_from_slice(&16u32.to_be_bytes()); // color_records_array_offset (after header + indices)

    // Palette color record indices (4 bytes)
    data.extend_from_slice(&0u16.to_be_bytes()); // palette 0 starts at color 0
    data.extend_from_slice(&2u16.to_be_bytes()); // palette 1 starts at color 2

    // Color records (at offset 16)
    // Palette 0
    data.extend_from_slice(&[0, 0, 255, 255]); // Red
    data.extend_from_slice(&[0, 255, 0, 255]); // Green

    // Palette 1
    data.extend_from_slice(&[255, 0, 0, 255]); // Blue
    data.extend_from_slice(&[255, 255, 0, 255]); // Cyan

    let cpal = CpalTable::parse(&data).expect("Failed to parse CPAL");

    assert_eq!(cpal.palette_count(), 2);

    let palette0 = cpal.get_palette(0).unwrap();
    assert_eq!(palette0[0].red, 255);
    assert_eq!(palette0[1].green, 255);

    let palette1 = cpal.get_palette(1).unwrap();
    assert_eq!(palette1[0].blue, 255);
    assert_eq!(palette1[1].blue, 255);
    assert_eq!(palette1[1].green, 255);
}

#[test]
fn test_colr_parse_minimal() {
    // Create a minimal valid COLR table
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&1u16.to_be_bytes()); // num_base_glyph_records
    data.extend_from_slice(&14u32.to_be_bytes()); // base_glyph_records_offset
    data.extend_from_slice(&20u32.to_be_bytes()); // layer_records_offset
    data.extend_from_slice(&2u16.to_be_bytes()); // num_layer_records

    // Base glyph record (at offset 14)
    data.extend_from_slice(&42u16.to_be_bytes()); // glyph_id
    data.extend_from_slice(&0u16.to_be_bytes()); // first_layer_index
    data.extend_from_slice(&2u16.to_be_bytes()); // num_layers

    // Layer records (at offset 20)
    data.extend_from_slice(&100u16.to_be_bytes()); // layer 0 glyph_id
    data.extend_from_slice(&0u16.to_be_bytes()); // layer 0 palette_index
    data.extend_from_slice(&101u16.to_be_bytes()); // layer 1 glyph_id
    data.extend_from_slice(&1u16.to_be_bytes()); // layer 1 palette_index

    let colr = ColrTable::parse(&data).expect("Failed to parse COLR");

    assert_eq!(colr.color_glyph_count(), 1);
    assert!(colr.is_color_glyph(42));
    assert!(!colr.is_color_glyph(43));

    let layers = colr.get_layers(42).unwrap();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].glyph_id, 100);
    assert_eq!(layers[0].palette_index, 0);
    assert_eq!(layers[1].glyph_id, 101);
    assert_eq!(layers[1].palette_index, 1);
}

#[test]
fn test_colr_parse_multiple_glyphs() {
    // Create COLR with multiple base glyphs
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&2u16.to_be_bytes()); // num_base_glyph_records
    data.extend_from_slice(&14u32.to_be_bytes()); // base_glyph_records_offset
    data.extend_from_slice(&26u32.to_be_bytes()); // layer_records_offset
    data.extend_from_slice(&4u16.to_be_bytes()); // num_layer_records

    // Base glyph records (at offset 14)
    // Glyph 42 with 2 layers
    data.extend_from_slice(&42u16.to_be_bytes()); // glyph_id
    data.extend_from_slice(&0u16.to_be_bytes()); // first_layer_index
    data.extend_from_slice(&2u16.to_be_bytes()); // num_layers

    // Glyph 43 with 2 layers
    data.extend_from_slice(&43u16.to_be_bytes()); // glyph_id
    data.extend_from_slice(&2u16.to_be_bytes()); // first_layer_index
    data.extend_from_slice(&2u16.to_be_bytes()); // num_layers

    // Layer records (at offset 26)
    data.extend_from_slice(&100u16.to_be_bytes()); // layer 0 glyph_id
    data.extend_from_slice(&0u16.to_be_bytes()); // layer 0 palette_index
    data.extend_from_slice(&101u16.to_be_bytes()); // layer 1 glyph_id
    data.extend_from_slice(&1u16.to_be_bytes()); // layer 1 palette_index
    data.extend_from_slice(&102u16.to_be_bytes()); // layer 2 glyph_id
    data.extend_from_slice(&2u16.to_be_bytes()); // layer 2 palette_index
    data.extend_from_slice(&103u16.to_be_bytes()); // layer 3 glyph_id
    data.extend_from_slice(&3u16.to_be_bytes()); // layer 3 palette_index

    let colr = ColrTable::parse(&data).expect("Failed to parse COLR");

    assert_eq!(colr.color_glyph_count(), 2);
    assert!(colr.is_color_glyph(42));
    assert!(colr.is_color_glyph(43));

    let layers42 = colr.get_layers(42).unwrap();
    assert_eq!(layers42.len(), 2);
    assert_eq!(layers42[0].glyph_id, 100);

    let layers43 = colr.get_layers(43).unwrap();
    assert_eq!(layers43.len(), 2);
    assert_eq!(layers43[0].glyph_id, 102);
}

#[test]
fn test_cbdt_parse_minimal() {
    // Create a minimal CBDT table
    let mut data = Vec::new();

    data.extend_from_slice(&2u16.to_be_bytes()); // major_version
    data.extend_from_slice(&0u16.to_be_bytes()); // minor_version

    let cbdt = CbdtTable::parse(&data).expect("Failed to parse CBDT");

    assert_eq!(cbdt.major_version, 2);
    assert_eq!(cbdt.minor_version, 0);
}

#[test]
fn test_svg_parse_minimal() {
    // Create a minimal SVG table
    let mut data = Vec::new();

    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&10u32.to_be_bytes()); // svg_document_list_offset
    data.extend_from_slice(&0u32.to_be_bytes()); // reserved

    let svg = SvgTable::parse(&data).expect("Failed to parse SVG");

    assert_eq!(svg.version, 0);
}

#[test]
fn test_cpal_parse_corrupted_palette_index() {
    // Create CPAL with invalid palette index
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&2u16.to_be_bytes()); // num_palette_entries
    data.extend_from_slice(&1u16.to_be_bytes()); // num_palettes
    data.extend_from_slice(&2u16.to_be_bytes()); // num_color_records (only 2!)
    data.extend_from_slice(&14u32.to_be_bytes()); // color_records_array_offset

    // Palette color record index points beyond color records
    data.extend_from_slice(&5u16.to_be_bytes()); // palette 0 starts at color 5 (out of range!)

    // Color records (at offset 14)
    data.extend_from_slice(&[0, 0, 255, 255]);
    data.extend_from_slice(&[0, 255, 0, 255]);

    let result = CpalTable::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_colr_parse_corrupted_layer_index() {
    // Create COLR with invalid layer index
    let mut data = Vec::new();

    // Header
    data.extend_from_slice(&0u16.to_be_bytes()); // version
    data.extend_from_slice(&1u16.to_be_bytes()); // num_base_glyph_records
    data.extend_from_slice(&14u32.to_be_bytes()); // base_glyph_records_offset
    data.extend_from_slice(&20u32.to_be_bytes()); // layer_records_offset
    data.extend_from_slice(&2u16.to_be_bytes()); // num_layer_records (only 2!)

    // Base glyph record (at offset 14) - requests 3 layers but only 2 exist
    data.extend_from_slice(&42u16.to_be_bytes()); // glyph_id
    data.extend_from_slice(&0u16.to_be_bytes()); // first_layer_index
    data.extend_from_slice(&3u16.to_be_bytes()); // num_layers (out of range!)

    // Layer records (at offset 20)
    data.extend_from_slice(&100u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());
    data.extend_from_slice(&101u16.to_be_bytes());
    data.extend_from_slice(&1u16.to_be_bytes());

    let result = ColrTable::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_cpal_parse_insufficient_data() {
    // Too short to be valid CPAL
    let data = vec![0u8; 5];
    let result = CpalTable::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_colr_parse_insufficient_data() {
    // Too short to be valid COLR
    let data = vec![0u8; 5];
    let result = ColrTable::parse(&data);
    assert!(result.is_err());
}
