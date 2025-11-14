use font_parser::{
    AvarTable, AxisSegmentMap, FvarTable, NamedInstance, OpenTypeFont, Tag, VariationAxis,
    VariationCoordinates,
};

/// Helper to create a minimal fvar table for testing
fn create_test_fvar_data() -> Vec<u8> {
    let mut data = Vec::new();

    // fvar header
    data.extend_from_slice(&1u16.to_be_bytes()); // major version
    data.extend_from_slice(&0u16.to_be_bytes()); // minor version
    data.extend_from_slice(&16u16.to_be_bytes()); // axes array offset
    data.extend_from_slice(&0u16.to_be_bytes()); // reserved
    data.extend_from_slice(&2u16.to_be_bytes()); // axis count
    data.extend_from_slice(&20u16.to_be_bytes()); // axis size
    data.extend_from_slice(&1u16.to_be_bytes()); // instance count
    data.extend_from_slice(&8u16.to_be_bytes()); // instance size (4 + 2*4)

    // Axis 1: wght (weight)
    data.extend_from_slice(&0x77676874u32.to_be_bytes()); // 'wght' tag
    data.extend_from_slice(&((100.0 * 65536.0) as i32).to_be_bytes()); // min: 100.0
    data.extend_from_slice(&((400.0 * 65536.0) as i32).to_be_bytes()); // default: 400.0
    data.extend_from_slice(&((900.0 * 65536.0) as i32).to_be_bytes()); // max: 900.0
    data.extend_from_slice(&0u16.to_be_bytes()); // flags
    data.extend_from_slice(&256u16.to_be_bytes()); // name ID

    // Axis 2: wdth (width)
    data.extend_from_slice(&0x77647468u32.to_be_bytes()); // 'wdth' tag
    data.extend_from_slice(&((75.0 * 65536.0) as i32).to_be_bytes()); // min: 75.0
    data.extend_from_slice(&((100.0 * 65536.0) as i32).to_be_bytes()); // default: 100.0
    data.extend_from_slice(&((125.0 * 65536.0) as i32).to_be_bytes()); // max: 125.0
    data.extend_from_slice(&0u16.to_be_bytes()); // flags
    data.extend_from_slice(&257u16.to_be_bytes()); // name ID

    // Named instance 1: Bold
    data.extend_from_slice(&258u16.to_be_bytes()); // subfamily name ID
    data.extend_from_slice(&0u16.to_be_bytes()); // flags
    data.extend_from_slice(&((700.0 * 65536.0) as i32).to_be_bytes()); // wght: 700
    data.extend_from_slice(&((100.0 * 65536.0) as i32).to_be_bytes()); // wdth: 100

    data
}

/// Helper to create a minimal avar table for testing
fn create_test_avar_data() -> Vec<u8> {
    let mut data = Vec::new();

    // avar header
    data.extend_from_slice(&1u16.to_be_bytes()); // major version
    data.extend_from_slice(&0u16.to_be_bytes()); // minor version
    data.extend_from_slice(&0u16.to_be_bytes()); // reserved
    data.extend_from_slice(&2u16.to_be_bytes()); // axis count

    // Axis 1 segment map (3 mappings)
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(&((-1.0 * 65536.0) as i32).to_be_bytes()); // from: -1.0
    data.extend_from_slice(&((-1.0 * 65536.0) as i32).to_be_bytes()); // to: -1.0
    data.extend_from_slice(&((0.0 * 65536.0) as i32).to_be_bytes()); // from: 0.0
    data.extend_from_slice(&((0.0 * 65536.0) as i32).to_be_bytes()); // to: 0.0
    data.extend_from_slice(&((1.0 * 65536.0) as i32).to_be_bytes()); // from: 1.0
    data.extend_from_slice(&((1.0 * 65536.0) as i32).to_be_bytes()); // to: 1.0

    // Axis 2 segment map (3 mappings with non-linear mapping)
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(&((-1.0 * 65536.0) as i32).to_be_bytes()); // from: -1.0
    data.extend_from_slice(&((-1.0 * 65536.0) as i32).to_be_bytes()); // to: -1.0
    data.extend_from_slice(&((0.0 * 65536.0) as i32).to_be_bytes()); // from: 0.0
    data.extend_from_slice(&((-0.5 * 65536.0) as i32).to_be_bytes()); // to: -0.5 (non-linear)
    data.extend_from_slice(&((1.0 * 65536.0) as i32).to_be_bytes()); // from: 1.0
    data.extend_from_slice(&((1.0 * 65536.0) as i32).to_be_bytes()); // to: 1.0

    data
}

#[test]
fn test_parse_fvar_table() {
    let data = create_test_fvar_data();
    let fvar = FvarTable::parse(&data).expect("Failed to parse fvar");

    assert_eq!(fvar.axes.len(), 2);
    assert_eq!(fvar.instances.len(), 1);
}

#[test]
fn test_fvar_weight_axis() {
    let data = create_test_fvar_data();
    let fvar = FvarTable::parse(&data).expect("Failed to parse fvar");

    let wght_axis = fvar.get_axis(Tag::WEIGHT).expect("Weight axis not found");
    assert_eq!(wght_axis.tag, Tag::WEIGHT);
    assert!((wght_axis.min_value - 100.0).abs() < 0.1);
    assert!((wght_axis.default_value - 400.0).abs() < 0.1);
    assert!((wght_axis.max_value - 900.0).abs() < 0.1);
    assert_eq!(wght_axis.name_id, 256);
}

#[test]
fn test_fvar_width_axis() {
    let data = create_test_fvar_data();
    let fvar = FvarTable::parse(&data).expect("Failed to parse fvar");

    let wdth_axis = fvar.get_axis(Tag::WIDTH).expect("Width axis not found");
    assert_eq!(wdth_axis.tag, Tag::WIDTH);
    assert!((wdth_axis.min_value - 75.0).abs() < 0.1);
    assert!((wdth_axis.default_value - 100.0).abs() < 0.1);
    assert!((wdth_axis.max_value - 125.0).abs() < 0.1);
    assert_eq!(wdth_axis.name_id, 257);
}

#[test]
fn test_fvar_named_instance() {
    let data = create_test_fvar_data();
    let fvar = FvarTable::parse(&data).expect("Failed to parse fvar");

    assert_eq!(fvar.instances.len(), 1);
    let instance = &fvar.instances[0];
    assert_eq!(instance.subfamily_name_id, 258);
    assert_eq!(instance.coordinates.len(), 2);
    assert!((instance.coordinates[0] - 700.0).abs() < 0.1); // wght: Bold
    assert!((instance.coordinates[1] - 100.0).abs() < 0.1); // wdth: Normal
    assert_eq!(instance.postscript_name_id, None);
}

#[test]
fn test_fvar_is_variable() {
    let data = create_test_fvar_data();
    let fvar = FvarTable::parse(&data).expect("Failed to parse fvar");

    assert!(fvar.is_variable());
}

#[test]
fn test_fvar_get_nonexistent_axis() {
    let data = create_test_fvar_data();
    let fvar = FvarTable::parse(&data).expect("Failed to parse fvar");

    let result = fvar.get_axis(Tag::SLANT);
    assert!(result.is_none());
}

#[test]
fn test_parse_avar_table() {
    let data = create_test_avar_data();
    let avar = AvarTable::parse(&data, 2).expect("Failed to parse avar");

    assert_eq!(avar.axis_segment_maps.len(), 2);
}

#[test]
fn test_avar_linear_mapping() {
    let data = create_test_avar_data();
    let avar = AvarTable::parse(&data, 2).expect("Failed to parse avar");

    // First axis has linear mapping
    assert!((avar.map_value(0, -1.0) - (-1.0)).abs() < 0.01);
    assert!((avar.map_value(0, 0.0) - 0.0).abs() < 0.01);
    assert!((avar.map_value(0, 1.0) - 1.0).abs() < 0.01);
    assert!((avar.map_value(0, 0.5) - 0.5).abs() < 0.01);
}

#[test]
fn test_avar_nonlinear_mapping() {
    let data = create_test_avar_data();
    let avar = AvarTable::parse(&data, 2).expect("Failed to parse avar");

    // Second axis has non-linear mapping
    assert!((avar.map_value(1, -1.0) - (-1.0)).abs() < 0.01);
    assert!((avar.map_value(1, 0.0) - (-0.5)).abs() < 0.01);
    assert!((avar.map_value(1, 1.0) - 1.0).abs() < 0.01);

    // Test interpolation: at input 0.5, should interpolate between -0.5 and 1.0
    // t = 0.5, output = -0.5 + 0.5 * (1.0 - (-0.5)) = 0.25
    assert!((avar.map_value(1, 0.5) - 0.25).abs() < 0.01);
}

#[test]
fn test_avar_axis_count_mismatch() {
    let data = create_test_avar_data();
    // Try to parse with wrong axis count
    let result = AvarTable::parse(&data, 3);
    assert!(result.is_err());
}

#[test]
fn test_variation_coordinates_basic() {
    let mut coords = VariationCoordinates::new();

    coords.set_axis(Tag::WEIGHT, 700.0);
    coords.set_axis(Tag::WIDTH, 100.0);

    assert_eq!(coords.get_axis(Tag::WEIGHT), Some(700.0));
    assert_eq!(coords.get_axis(Tag::WIDTH), Some(100.0));
    assert_eq!(coords.get_axis(Tag::SLANT), None);
}

#[test]
fn test_variation_coordinates_update() {
    let mut coords = VariationCoordinates::new();

    coords.set_axis(Tag::WEIGHT, 700.0);
    assert_eq!(coords.get_axis(Tag::WEIGHT), Some(700.0));

    // Update existing axis
    coords.set_axis(Tag::WEIGHT, 800.0);
    assert_eq!(coords.get_axis(Tag::WEIGHT), Some(800.0));
    assert_eq!(coords.values.len(), 1); // Should not duplicate
}

#[test]
fn test_tag_constants() {
    // Verify standard axis tag constants
    assert_eq!(Tag::WEIGHT.as_str(), "wght");
    assert_eq!(Tag::WIDTH.as_str(), "wdth");
    assert_eq!(Tag::SLANT.as_str(), "slnt");
    assert_eq!(Tag::OPTICAL_SIZE.as_str(), "opsz");
    assert_eq!(Tag::ITALIC.as_str(), "ital");
}

#[test]
fn test_fvar_invalid_version() {
    let mut data = Vec::new();
    // Invalid version 2.0
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());

    let result = FvarTable::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_avar_invalid_version() {
    let mut data = Vec::new();
    // Invalid version 2.0
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(&0u16.to_be_bytes());

    let result = AvarTable::parse(&data, 1);
    assert!(result.is_err());
}

#[test]
fn test_fvar_corrupted_data() {
    // Too short data
    let data = vec![0, 1, 0, 0];
    let result = FvarTable::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_avar_corrupted_data() {
    // Too short data
    let data = vec![0, 1, 0, 0];
    let result = AvarTable::parse(&data, 1);
    assert!(result.is_err());
}

#[test]
fn test_variation_coordinates_equality() {
    let mut coords1 = VariationCoordinates::new();
    coords1.set_axis(Tag::WEIGHT, 700.0);

    let mut coords2 = VariationCoordinates::new();
    coords2.set_axis(Tag::WEIGHT, 700.0);

    assert_eq!(coords1, coords2);
}

#[test]
fn test_variation_axis_equality() {
    let axis1 = VariationAxis {
        tag: Tag::WEIGHT,
        name_id: 256,
        min_value: 100.0,
        default_value: 400.0,
        max_value: 900.0,
    };

    let axis2 = VariationAxis {
        tag: Tag::WEIGHT,
        name_id: 256,
        min_value: 100.0,
        default_value: 400.0,
        max_value: 900.0,
    };

    assert_eq!(axis1, axis2);
}

#[test]
fn test_named_instance_equality() {
    let instance1 = NamedInstance {
        subfamily_name_id: 258,
        coordinates: vec![700.0, 100.0],
        postscript_name_id: None,
    };

    let instance2 = NamedInstance {
        subfamily_name_id: 258,
        coordinates: vec![700.0, 100.0],
        postscript_name_id: None,
    };

    assert_eq!(instance1, instance2);
}
