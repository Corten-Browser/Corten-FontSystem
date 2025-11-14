//! WOFF (Web Open Font Format) parsing

use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::io::{Cursor, Read};

/// WOFF table entry
#[derive(Debug, Clone)]
struct WoffTableEntry {
    tag: u32,
    offset: u32,
    comp_length: u32,
    orig_length: u32,
    orig_checksum: u32,
}

/// Parsed WOFF font
#[derive(Debug, Clone)]
pub struct WoffFont {
    /// Decompressed TTF/OTF data
    pub ttf_data: Vec<u8>,
    /// WOFF metadata (optional)
    pub metadata: Option<String>,
}

impl WoffFont {
    /// Parse a WOFF font from bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        // Read WOFF header
        let signature = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        if signature != 0x774F4646 {
            // "wOFF"
            return Err(ParseError::InvalidFormat);
        }

        let flavor = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?; // TTF or CFF
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
        let meta_orig_length = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _priv_offset = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
        let _priv_length = cursor
            .read_u32::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        // Read table directory
        let mut tables = Vec::new();
        for _ in 0..num_tables {
            let tag = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
            let offset = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
            let comp_length = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
            let orig_length = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
            let orig_checksum = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

            tables.push(WoffTableEntry {
                tag,
                offset,
                comp_length,
                orig_length,
                orig_checksum,
            });
        }

        // Decompress tables and reconstruct TTF/OTF
        let ttf_data = Self::reconstruct_sfnt(data, &tables, flavor, num_tables)?;

        // Extract metadata if present
        let metadata = if meta_offset > 0 && meta_length > 0 {
            Some(Self::extract_metadata(
                data,
                meta_offset as usize,
                meta_length as usize,
                meta_orig_length as usize,
            )?)
        } else {
            None
        };

        Ok(WoffFont { ttf_data, metadata })
    }

    fn reconstruct_sfnt(
        woff_data: &[u8],
        tables: &[WoffTableEntry],
        flavor: u32,
        num_tables: u16,
    ) -> Result<Vec<u8>, ParseError> {
        // Calculate sizes for SFNT header
        let search_range = if num_tables == 0 {
            0
        } else {
            (num_tables as u32).next_power_of_two() * 16
        };
        let entry_selector = if num_tables == 0 {
            0
        } else {
            (num_tables as f32).log2().floor() as u16
        };
        let range_shift = (num_tables * 16).saturating_sub(search_range as u16);

        let mut sfnt = Vec::new();

        // Write SFNT header
        sfnt.extend_from_slice(&flavor.to_be_bytes());
        sfnt.extend_from_slice(&num_tables.to_be_bytes());
        sfnt.extend_from_slice(&search_range.to_be_bytes()[2..]);
        sfnt.extend_from_slice(&entry_selector.to_be_bytes());
        sfnt.extend_from_slice(&range_shift.to_be_bytes());

        // Calculate table offsets in reconstructed SFNT
        let mut offset = 12 + (num_tables as usize * 16);

        // Write table directory and decompress tables
        for table in tables {
            // Write table directory entry
            sfnt.extend_from_slice(&table.tag.to_be_bytes());
            sfnt.extend_from_slice(&table.orig_checksum.to_be_bytes());
            sfnt.extend_from_slice(&(offset as u32).to_be_bytes());
            sfnt.extend_from_slice(&table.orig_length.to_be_bytes());

            offset += table.orig_length as usize;
            // Pad to 4-byte boundary
            offset = (offset + 3) & !3;
        }

        // Decompress and write table data
        for table in tables {
            let comp_data =
                &woff_data[table.offset as usize..(table.offset + table.comp_length) as usize];

            let decompressed = if table.comp_length < table.orig_length {
                // Table is compressed
                let mut decoder = ZlibDecoder::new(comp_data);
                let mut decompressed = Vec::new();
                decoder
                    .read_to_end(&mut decompressed)
                    .map_err(|e| ParseError::CorruptedData(e.to_string()))?;
                decompressed
            } else {
                // Table is not compressed
                comp_data.to_vec()
            };

            if decompressed.len() != table.orig_length as usize {
                return Err(ParseError::CorruptedData(
                    "Decompression size mismatch".to_string(),
                ));
            }

            sfnt.extend_from_slice(&decompressed);

            // Pad to 4-byte boundary
            while sfnt.len() % 4 != 0 {
                sfnt.push(0);
            }
        }

        Ok(sfnt)
    }

    fn extract_metadata(
        woff_data: &[u8],
        offset: usize,
        comp_length: usize,
        _orig_length: usize,
    ) -> Result<String, ParseError> {
        let comp_data = &woff_data[offset..offset + comp_length];

        let mut decoder = ZlibDecoder::new(comp_data);
        let mut metadata_xml = String::new();
        decoder
            .read_to_string(&mut metadata_xml)
            .map_err(|e| ParseError::CorruptedData(e.to_string()))?;

        Ok(metadata_xml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_woff_signature_detection() {
        // WOFF signature: 0x774F4646 ("wOFF")
        let mut data = vec![0x77, 0x4F, 0x46, 0x46]; // "wOFF"

        // Add minimal header (not complete, just for signature test)
        data.extend_from_slice(&[0, 0, 0, 0]); // flavor
        data.extend_from_slice(&[0, 0, 0, 44]); // length
        data.extend_from_slice(&[0, 0]); // num_tables

        let mut cursor = Cursor::new(&data);
        let signature = cursor.read_u32::<BigEndian>().unwrap();

        assert_eq!(signature, 0x774F4646);
    }

    #[test]
    fn test_woff_invalid_signature() {
        let data = vec![0x00, 0x01, 0x00, 0x00]; // Not WOFF
        let result = WoffFont::parse(&data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidFormat);
    }

    #[test]
    fn test_woff_too_short() {
        let data = vec![0x77, 0x4F]; // Too short
        let result = WoffFont::parse(&data);
        assert!(result.is_err());
    }
}
