//! Integration tests for OpenType GSUB/GPOS features

use font_parser::{
    default_features, kerning_and_ligatures, kerning_only, tags, Coverage, FeatureApplicator,
    FeatureQuery, FeatureSelection, GposTable, GsubTable, OpenTypeFont,
};

// ============================================================================
// GSUB Table Tests
// ============================================================================

#[test]
fn test_gsub_table_parse_minimal() {
    // Minimal GSUB table with version only
    let data = [
        0x00, 0x01, // major version 1
        0x00, 0x00, // minor version 0
        0x00, 0x00, // script list offset (NULL)
        0x00, 0x00, // feature list offset (NULL)
        0x00, 0x00, // lookup list offset (NULL)
    ];

    let gsub = GsubTable::parse(&data).unwrap();
    assert_eq!(gsub.major_version, 1);
    assert_eq!(gsub.minor_version, 0);
    assert!(gsub.scripts.scripts.is_empty());
    assert!(gsub.features.features.is_empty());
    assert!(gsub.lookups.lookups.is_empty());
}

#[test]
fn test_gsub_table_unsupported_version() {
    let data = [
        0x00, 0x02, // major version 2 (unsupported)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let result = GsubTable::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_gsub_supported_features_empty() {
    let data = [
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let gsub = GsubTable::parse(&data).unwrap();
    assert!(gsub.supported_features().is_empty());
    assert!(!gsub.has_feature(tags::LIGA));
}

// ============================================================================
// GPOS Table Tests
// ============================================================================

#[test]
fn test_gpos_table_parse_minimal() {
    // Minimal GPOS table with version only
    let data = [
        0x00, 0x01, // major version 1
        0x00, 0x00, // minor version 0
        0x00, 0x00, // script list offset (NULL)
        0x00, 0x00, // feature list offset (NULL)
        0x00, 0x00, // lookup list offset (NULL)
    ];

    let gpos = GposTable::parse(&data).unwrap();
    assert_eq!(gpos.major_version, 1);
    assert_eq!(gpos.minor_version, 0);
    assert!(gpos.scripts.scripts.is_empty());
    assert!(gpos.features.features.is_empty());
    assert!(gpos.lookups.lookups.is_empty());
}

#[test]
fn test_gpos_table_unsupported_version() {
    let data = [
        0x00, 0x02, // major version 2 (unsupported)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let result = GposTable::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_gpos_kerning_empty() {
    let data = [
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let gpos = GposTable::parse(&data).unwrap();
    assert!(gpos.get_kerning(10, 20).is_none());
}

// ============================================================================
// Coverage Table Tests
// ============================================================================

#[test]
fn test_coverage_format1() {
    // Format 1: list of glyph IDs
    let data = [
        0x00, 0x01, // format 1
        0x00, 0x03, // glyph count
        0x00, 0x10, // glyph 16
        0x00, 0x20, // glyph 32
        0x00, 0x30, // glyph 48
    ];

    let coverage = Coverage::parse(&data).unwrap();
    assert!(coverage.contains(16));
    assert!(coverage.contains(32));
    assert!(coverage.contains(48));
    assert!(!coverage.contains(24));
    assert_eq!(coverage.get_index(16), Some(0));
    assert_eq!(coverage.get_index(32), Some(1));
    assert_eq!(coverage.get_index(48), Some(2));
    assert_eq!(coverage.get_index(24), None);
}

#[test]
fn test_coverage_format2() {
    // Format 2: ranges
    let data = [
        0x00, 0x02, // format 2
        0x00, 0x01, // range count
        0x00, 0x10, // start glyph 16
        0x00, 0x14, // end glyph 20
        0x00, 0x00, // start coverage index
    ];

    let coverage = Coverage::parse(&data).unwrap();
    assert!(coverage.contains(16));
    assert!(coverage.contains(18));
    assert!(coverage.contains(20));
    assert!(!coverage.contains(15));
    assert!(!coverage.contains(21));
}

// ============================================================================
// Feature Selection Tests
// ============================================================================

#[test]
fn test_feature_selection_default() {
    let selection = default_features();
    assert!(selection.is_enabled(tags::LIGA));
    assert!(selection.is_enabled(tags::KERN));
    assert!(selection.is_enabled(tags::CALT));
    assert!(!selection.is_enabled(tags::SMCP));
    assert!(!selection.is_enabled(tags::DLIG));
}

#[test]
fn test_feature_selection_kerning_only() {
    let selection = kerning_only();
    assert!(selection.is_enabled(tags::KERN));
    assert!(!selection.is_enabled(tags::LIGA));
    assert!(!selection.is_enabled(tags::CALT));
}

#[test]
fn test_feature_selection_kerning_and_ligatures() {
    let selection = kerning_and_ligatures();
    assert!(selection.is_enabled(tags::KERN));
    assert!(selection.is_enabled(tags::LIGA));
    assert!(selection.is_enabled(tags::CLIG));
}

#[test]
fn test_feature_selection_custom() {
    let mut selection = FeatureSelection::new();
    assert!(!selection.is_enabled(tags::LIGA));

    selection.enable(tags::LIGA);
    assert!(selection.is_enabled(tags::LIGA));
    assert_eq!(selection.get_value(tags::LIGA), 1);

    selection.enable_with_value(tags::SALT, 5);
    assert!(selection.is_enabled(tags::SALT));
    assert_eq!(selection.get_value(tags::SALT), 5);

    selection.disable(tags::LIGA);
    assert!(!selection.is_enabled(tags::LIGA));
}

#[test]
fn test_feature_selection_script_language() {
    let mut selection = FeatureSelection::new();
    assert!(selection.script().is_none());
    assert!(selection.language().is_none());

    selection.set_script(tags::LIGA); // Using LIGA as a 4-char tag for test
    assert!(selection.script().is_some());

    selection.set_language(tags::KERN);
    assert!(selection.language().is_some());
}

#[test]
fn test_feature_selection_enabled_features() {
    let selection = kerning_only();
    let features = selection.enabled_features();
    assert_eq!(features.len(), 1);
    assert!(features.contains(&tags::KERN));
}

// ============================================================================
// Feature Query Tests
// ============================================================================

#[test]
fn test_feature_query_empty() {
    let query = FeatureQuery::new(None, None);
    assert!(query.gsub_features().is_empty());
    assert!(query.gpos_features().is_empty());
    assert!(query.all_features().is_empty());
    assert!(!query.has_feature(tags::LIGA));
    assert!(!query.has_ligatures());
    assert!(!query.has_kerning());
}

#[test]
fn test_feature_query_scripts_empty() {
    let query = FeatureQuery::new(None, None);
    assert!(query.available_scripts().is_empty());
}

// ============================================================================
// Feature Applicator Tests
// ============================================================================

#[test]
fn test_feature_applicator_empty() {
    let selection = default_features();
    let applicator = FeatureApplicator::new(None, None, &selection);

    let mut glyphs: Vec<u16> = vec![1, 2, 3, 4, 5];
    applicator.apply_gsub(&mut glyphs);
    assert_eq!(glyphs, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_feature_applicator_kerning_empty() {
    let selection = default_features();
    let applicator = FeatureApplicator::new(None, None, &selection);

    let glyphs: Vec<u16> = vec![1, 2, 3];
    let kerning = applicator.get_kerning(&glyphs);
    assert_eq!(kerning, vec![0, 0, 0]);
}

#[test]
fn test_feature_applicator_kerning_disabled() {
    let mut selection = FeatureSelection::new();
    selection.disable(tags::KERN);

    let data = [
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let gpos = GposTable::parse(&data).unwrap();

    let applicator = FeatureApplicator::new(None, Some(&gpos), &selection);

    let glyphs: Vec<u16> = vec![1, 2, 3];
    let kerning = applicator.get_kerning(&glyphs);
    assert_eq!(kerning, vec![0, 0, 0]);
}

// ============================================================================
// Feature Tag Tests
// ============================================================================

#[test]
fn test_common_feature_tags() {
    // GSUB features
    assert_eq!(tags::LIGA.as_str(), "liga");
    assert_eq!(tags::CLIG.as_str(), "clig");
    assert_eq!(tags::DLIG.as_str(), "dlig");
    assert_eq!(tags::CALT.as_str(), "calt");
    assert_eq!(tags::SMCP.as_str(), "smcp");
    assert_eq!(tags::SALT.as_str(), "salt");

    // GPOS features
    assert_eq!(tags::KERN.as_str(), "kern");
    assert_eq!(tags::MARK.as_str(), "mark");
    assert_eq!(tags::MKMK.as_str(), "mkmk");

    // Numeric features
    assert_eq!(tags::ONUM.as_str(), "onum");
    assert_eq!(tags::LNUM.as_str(), "lnum");
    assert_eq!(tags::TNUM.as_str(), "tnum");
    assert_eq!(tags::PNUM.as_str(), "pnum");
    assert_eq!(tags::FRAC.as_str(), "frac");

    // Script-specific
    assert_eq!(tags::INIT.as_str(), "init");
    assert_eq!(tags::MEDI.as_str(), "medi");
    assert_eq!(tags::FINA.as_str(), "fina");
    assert_eq!(tags::ISOL.as_str(), "isol");
}

// ============================================================================
// Integration with OpenTypeFont
// ============================================================================

#[test]
fn test_opentype_font_gsub_gpos_absent() {
    // Create minimal font without GSUB/GPOS tables
    // This tests the behavior when these tables are not present
    let mut data = vec![
        0x00, 0x01, 0x00, 0x00, // sfnt version (TrueType)
        0x00, 0x01, // numTables = 1
        0x00, 0x10, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
    ];

    // Add a minimal 'head' table entry
    data.extend_from_slice(b"head"); // tag
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // checksum
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x1C]); // offset (28)
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x36]); // length (54)

    // Add minimal head table data (54 bytes)
    data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // version
    data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // fontRevision
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // checksumAdjustment
    data.extend_from_slice(&[0x5F, 0x0F, 0x3C, 0xF5]); // magicNumber
    data.extend_from_slice(&[0x00, 0x00]); // flags
    data.extend_from_slice(&[0x03, 0xE8]); // unitsPerEm = 1000
    // Fill rest with zeros
    data.extend_from_slice(&[0; 36]);

    let font = OpenTypeFont::parse(data).unwrap();
    assert!(!font.has_table("GSUB".parse().unwrap()));
    assert!(!font.has_table("GPOS".parse().unwrap()));
}
