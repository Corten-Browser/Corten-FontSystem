mod integration {
    use font_parser::{OpenTypeFont, ParseError, Woff2Font, WoffFont};

    #[test]
    fn test_opentype_font_detects_woff_format() {
        // Given a minimal WOFF font structure
        let mut woff_data = vec![0x77, 0x4F, 0x46, 0x46]; // "wOFF" signature
        woff_data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // flavor (TrueType)
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x2C]); // length (44 bytes)
        woff_data.extend_from_slice(&[0x00, 0x00]); // num_tables (0)
        woff_data.extend_from_slice(&[0x00, 0x00]); // reserved
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]); // total_sfnt_size (12 bytes)
        woff_data.extend_from_slice(&[0x00, 0x01]); // major_version
        woff_data.extend_from_slice(&[0x00, 0x00]); // minor_version
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_offset
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_length
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_orig_length
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_offset
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_length

        // When OpenTypeFont::parse is called on WOFF data
        let result = OpenTypeFont::parse(woff_data);

        // Then it should attempt to parse (may fail due to minimal data, but not on format detection)
        // The important thing is that it recognized WOFF format and tried to decompress
        assert!(result.is_ok() || matches!(result, Err(ParseError::CorruptedData(_))));
    }

    #[test]
    fn test_opentype_font_detects_woff2_format() {
        // Given a minimal WOFF2 font structure
        let mut woff2_data = vec![0x77, 0x4F, 0x46, 0x32]; // "wOF2" signature
        woff2_data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // flavor (TrueType)
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x30]); // length (48 bytes)
        woff2_data.extend_from_slice(&[0x00, 0x00]); // num_tables (0)
        woff2_data.extend_from_slice(&[0x00, 0x00]); // reserved
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]); // total_sfnt_size (12 bytes)
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // total_compressed_size
        woff2_data.extend_from_slice(&[0x00, 0x01]); // major_version
        woff2_data.extend_from_slice(&[0x00, 0x00]); // minor_version
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_offset
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_length
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_orig_length
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_offset
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_length

        // When OpenTypeFont::parse is called on WOFF2 data
        let result = OpenTypeFont::parse(woff2_data);

        // Then it should attempt to parse (may fail due to minimal data, but not on format detection)
        assert!(result.is_ok() || matches!(result, Err(ParseError::CorruptedData(_))));
    }

    #[test]
    fn test_woff_font_parses_valid_signature() {
        // Given a WOFF font with valid signature
        let mut woff_data = vec![0x77, 0x4F, 0x46, 0x46]; // "wOFF"
        woff_data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // flavor
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x2C]); // length
        woff_data.extend_from_slice(&[0x00, 0x00]); // num_tables
        woff_data.extend_from_slice(&[0x00, 0x00]); // reserved
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]); // total_sfnt_size
        woff_data.extend_from_slice(&[0x00, 0x01]); // major_version
        woff_data.extend_from_slice(&[0x00, 0x00]); // minor_version
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_offset
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_length
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_orig_length
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_offset
        woff_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_length

        // When WoffFont::parse is called
        let result = WoffFont::parse(&woff_data);

        // Then it should succeed (or fail with CorruptedData, not InvalidFormat)
        assert!(result.is_ok() || matches!(result, Err(ParseError::CorruptedData(_))));
    }

    #[test]
    fn test_woff_font_rejects_invalid_signature() {
        // Given data with invalid WOFF signature
        let invalid_data = vec![0x00, 0x01, 0x00, 0x00]; // Not WOFF

        // When WoffFont::parse is called
        let result = WoffFont::parse(&invalid_data);

        // Then it should fail with InvalidFormat
        assert!(matches!(result, Err(ParseError::InvalidFormat)));
    }

    #[test]
    fn test_woff2_font_parses_valid_signature() {
        // Given a WOFF2 font with valid signature
        let mut woff2_data = vec![0x77, 0x4F, 0x46, 0x32]; // "wOF2"
        woff2_data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]); // flavor
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x30]); // length
        woff2_data.extend_from_slice(&[0x00, 0x00]); // num_tables
        woff2_data.extend_from_slice(&[0x00, 0x00]); // reserved
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]); // total_sfnt_size
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // total_compressed_size
        woff2_data.extend_from_slice(&[0x00, 0x01]); // major_version
        woff2_data.extend_from_slice(&[0x00, 0x00]); // minor_version
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_offset
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_length
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // meta_orig_length
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_offset
        woff2_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // priv_length

        // When Woff2Font::parse is called
        let result = Woff2Font::parse(&woff2_data);

        // Then it should succeed (or fail with CorruptedData, not InvalidFormat)
        assert!(result.is_ok() || matches!(result, Err(ParseError::CorruptedData(_))));
    }

    #[test]
    fn test_woff2_font_rejects_invalid_signature() {
        // Given data with invalid WOFF2 signature
        let invalid_data = vec![0x00, 0x01, 0x00, 0x00]; // Not WOFF2

        // When Woff2Font::parse is called
        let result = Woff2Font::parse(&invalid_data);

        // Then it should fail with InvalidFormat
        assert!(matches!(result, Err(ParseError::InvalidFormat)));
    }

    #[test]
    fn test_opentype_font_still_parses_ttf() {
        // Given a minimal TrueType font
        let mut ttf_data = vec![0x00, 0x01, 0x00, 0x00]; // TrueType signature
        ttf_data.extend_from_slice(&[0x00, 0x00]); // num_tables (0)
        ttf_data.extend_from_slice(&[0x00, 0x10]); // search_range
        ttf_data.extend_from_slice(&[0x00, 0x00]); // entry_selector
        ttf_data.extend_from_slice(&[0x00, 0x00]); // range_shift

        // When OpenTypeFont::parse is called
        let result = OpenTypeFont::parse(ttf_data);

        // Then it should parse successfully
        assert!(result.is_ok());
    }

    #[test]
    fn test_opentype_font_still_parses_otf() {
        // Given a minimal OpenType/CFF font
        let mut otf_data = vec![0x4F, 0x54, 0x54, 0x4F]; // 'OTTO' signature
        otf_data.extend_from_slice(&[0x00, 0x00]); // num_tables (0)
        otf_data.extend_from_slice(&[0x00, 0x10]); // search_range
        otf_data.extend_from_slice(&[0x00, 0x00]); // entry_selector
        otf_data.extend_from_slice(&[0x00, 0x00]); // range_shift

        // When OpenTypeFont::parse is called
        let result = OpenTypeFont::parse(otf_data);

        // Then it should parse successfully
        assert!(result.is_ok());
    }
}
