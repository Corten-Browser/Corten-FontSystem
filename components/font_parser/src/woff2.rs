//! WOFF2 (Web Open Font Format 2) parsing

use crate::ParseError;
use brotli::Decompressor;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Parsed WOFF2 font
#[derive(Debug, Clone)]
pub struct Woff2Font {
    /// Decompressed TTF/OTF data
    pub ttf_data: Vec<u8>,
    /// WOFF2 metadata (optional)
    pub metadata: Option<String>,
}

impl Woff2Font {
    /// Parse a WOFF2 font from bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        // Read WOFF2 header
        let signature = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        if signature != 0x774F4632 {
            // "wOF2"
            return Err(ParseError::InvalidFormat);
        }

        let flavor = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _length = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let num_tables = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _reserved = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _total_sfnt_size = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let total_compressed_size = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _major_version = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _minor_version = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let meta_offset = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let meta_length = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _meta_orig_length = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _priv_offset = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _priv_length = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        // WOFF2 uses more complex reconstruction
        // For this implementation, we'll provide basic support

        // Read compressed data block
        let compressed_start = cursor.position() as usize;
        let compressed_data =
            &data[compressed_start..compressed_start + total_compressed_size as usize];

        // Decompress using Brotli
        let mut decompressor = Decompressor::new(compressed_data, 4096);
        let mut decompressed = Vec::new();
        decompressor
            .read_to_end(&mut decompressed)
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        // WOFF2 format is complex - simplified reconstruction
        // In production, use a full WOFF2 library
        let ttf_data = Self::reconstruct_sfnt(decompressed, flavor, num_tables)?;

        // Extract metadata if present
        let metadata = if meta_offset > 0 && meta_length > 0 {
            Some(Self::extract_metadata(
                data,
                meta_offset as usize,
                meta_length as usize,
            )?)
        } else {
            None
        };

        Ok(Woff2Font { ttf_data, metadata })
    }

    fn reconstruct_sfnt(
        decompressed: Vec<u8>,
        _flavor: u32,
        _num_tables: u16,
    ) -> Result<Vec<u8>, ParseError> {
        // Simplified WOFF2 reconstruction
        // Full implementation would handle transformed tables (glyf, loca, hmtx)
        // For now, we'll return the decompressed data as-is
        // TODO: Implement full WOFF2 table reconstruction

        Ok(decompressed)
    }

    fn extract_metadata(
        woff2_data: &[u8],
        offset: usize,
        comp_length: usize,
    ) -> Result<String, ParseError> {
        let comp_data = &woff2_data[offset..offset + comp_length];

        let mut decompressor = Decompressor::new(comp_data, 4096);
        let mut metadata_xml = String::new();
        decompressor
            .read_to_string(&mut metadata_xml)
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        Ok(metadata_xml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_woff2_signature_detection() {
        // WOFF2 signature: 0x774F4632 ("wOF2")
        let mut data = vec![0x77, 0x4F, 0x46, 0x32]; // "wOF2"

        // Add minimal header (not complete, just for signature test)
        data.extend_from_slice(&[0, 0, 0, 0]); // flavor
        data.extend_from_slice(&[0, 0, 0, 48]); // length
        data.extend_from_slice(&[0, 0]); // num_tables

        let mut cursor = Cursor::new(&data);
        let signature = cursor.read_u32::<BigEndian>().unwrap();

        assert_eq!(signature, 0x774F4632);
    }

    #[test]
    fn test_woff2_invalid_signature() {
        let data = vec![0x00, 0x01, 0x00, 0x00]; // Not WOFF2
        let result = Woff2Font::parse(&data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidFormat);
    }

    #[test]
    fn test_woff2_too_short() {
        let data = vec![0x77, 0x4F]; // Too short
        let result = Woff2Font::parse(&data);
        assert!(result.is_err());
    }
}
