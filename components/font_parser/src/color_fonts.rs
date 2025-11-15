//! Color fonts support (COLR/CPAL, CBDT/CBLC, sbix, SVG)
//!
//! This module provides parsing for color font tables in OpenType fonts,
//! including emoji and multi-color glyph support.

use crate::types::GlyphId;
use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// RGBA color definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Blue component (0-255)
    pub blue: u8,
    /// Green component (0-255)
    pub green: u8,
    /// Red component (0-255)
    pub red: u8,
    /// Alpha component (0-255, 255 = opaque)
    pub alpha: u8,
}

impl Color {
    /// Create a color from RGBA components
    pub fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Convert color to a 32-bit RGBA value
    pub fn to_rgba_u32(&self) -> u32 {
        ((self.alpha as u32) << 24)
            | ((self.red as u32) << 16)
            | ((self.green as u32) << 8)
            | (self.blue as u32)
    }
}

/// CPAL - Color Palette Table
///
/// Defines color palettes used by COLR table for layered color glyphs.
#[derive(Debug, Clone)]
pub struct CpalTable {
    /// Color palettes (each palette is a list of colors)
    pub palettes: Vec<Vec<Color>>,
    /// Optional palette labels (name table IDs)
    pub palette_labels: Vec<Option<u16>>,
    /// Optional palette type flags
    pub palette_types: Vec<u32>,
}

impl CpalTable {
    /// Parse CPAL table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        // Read CPAL header
        let version = cursor.read_u16::<BigEndian>()?;
        let num_palette_entries = cursor.read_u16::<BigEndian>()?;
        let num_palettes = cursor.read_u16::<BigEndian>()?;
        let num_color_records = cursor.read_u16::<BigEndian>()?;
        let color_records_array_offset = cursor.read_u32::<BigEndian>()?;

        // Read color record indices for each palette
        let mut color_record_indices = Vec::new();
        for _ in 0..num_palettes {
            color_record_indices.push(cursor.read_u16::<BigEndian>()?);
        }

        // Read color records
        cursor.set_position(color_records_array_offset as u64);
        let mut color_records = Vec::new();
        for _ in 0..num_color_records {
            let blue = cursor.read_u8()?;
            let green = cursor.read_u8()?;
            let red = cursor.read_u8()?;
            let alpha = cursor.read_u8()?;
            color_records.push(Color {
                blue,
                green,
                red,
                alpha,
            });
        }

        // Build palettes from color records
        let mut palettes = Vec::new();
        for &start_color_index in &color_record_indices {
            let start_index = start_color_index as usize;
            let end_index = start_index + num_palette_entries as usize;
            if end_index > color_records.len() {
                return Err(ParseError::CorruptedData(
                    "CPAL palette index out of range".to_string(),
                ));
            }
            let palette = color_records[start_index..end_index].to_vec();
            palettes.push(palette);
        }

        // Version 1 has additional data (labels, types)
        let (palette_labels, palette_types) = if version >= 1 {
            // For version 1, there are optional palette labels and types
            // For simplicity, we'll initialize with defaults
            // Full implementation would parse these from the extended data
            (
                vec![None; num_palettes as usize],
                vec![0; num_palettes as usize],
            )
        } else {
            (
                vec![None; num_palettes as usize],
                vec![0; num_palettes as usize],
            )
        };

        Ok(CpalTable {
            palettes,
            palette_labels,
            palette_types,
        })
    }

    /// Get a specific palette by index
    pub fn get_palette(&self, index: usize) -> Option<&Vec<Color>> {
        self.palettes.get(index)
    }

    /// Get the default palette (palette 0)
    pub fn default_palette(&self) -> Option<&Vec<Color>> {
        self.get_palette(0)
    }

    /// Get number of palettes
    pub fn palette_count(&self) -> usize {
        self.palettes.len()
    }
}

/// COLR - Color Layer Table
///
/// Defines layered color glyphs using palette colors from CPAL.
#[derive(Debug, Clone)]
pub struct ColrTable {
    /// Base glyphs with their color layers
    pub base_glyphs: Vec<BaseGlyph>,
}

/// A base glyph with color layers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseGlyph {
    /// The base glyph ID
    pub glyph_id: GlyphId,
    /// Color layers for this glyph
    pub layers: Vec<Layer>,
}

/// A color layer in a glyph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer {
    /// Glyph ID for this layer
    pub glyph_id: GlyphId,
    /// Palette index for this layer's color
    pub palette_index: u16,
}

impl ColrTable {
    /// Parse COLR table from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        // Read COLR header
        let _version = cursor.read_u16::<BigEndian>()?;
        let num_base_glyph_records = cursor.read_u16::<BigEndian>()?;
        let base_glyph_records_offset = cursor.read_u32::<BigEndian>()?;
        let layer_records_offset = cursor.read_u32::<BigEndian>()?;
        let num_layer_records = cursor.read_u16::<BigEndian>()?;

        // Read base glyph records
        cursor.set_position(base_glyph_records_offset as u64);
        let mut base_glyph_records = Vec::new();
        for _ in 0..num_base_glyph_records {
            let glyph_id = cursor.read_u16::<BigEndian>()?;
            let first_layer_index = cursor.read_u16::<BigEndian>()?;
            let num_layers = cursor.read_u16::<BigEndian>()?;
            base_glyph_records.push((glyph_id, first_layer_index, num_layers));
        }

        // Read layer records
        cursor.set_position(layer_records_offset as u64);
        let mut layer_records = Vec::new();
        for _ in 0..num_layer_records {
            let glyph_id = cursor.read_u16::<BigEndian>()?;
            let palette_index = cursor.read_u16::<BigEndian>()?;
            layer_records.push(Layer {
                glyph_id,
                palette_index,
            });
        }

        // Build base glyphs with their layers
        let mut base_glyphs = Vec::new();
        for (glyph_id, first_layer_index, num_layers) in base_glyph_records {
            let start = first_layer_index as usize;
            let end = start + num_layers as usize;
            if end > layer_records.len() {
                return Err(ParseError::CorruptedData(
                    "COLR layer index out of range".to_string(),
                ));
            }
            let layers = layer_records[start..end].to_vec();
            base_glyphs.push(BaseGlyph { glyph_id, layers });
        }

        Ok(ColrTable { base_glyphs })
    }

    /// Get layers for a specific glyph
    pub fn get_layers(&self, glyph_id: GlyphId) -> Option<&Vec<Layer>> {
        self.base_glyphs
            .iter()
            .find(|bg| bg.glyph_id == glyph_id)
            .map(|bg| &bg.layers)
    }

    /// Check if a glyph has color layers
    pub fn is_color_glyph(&self, glyph_id: GlyphId) -> bool {
        self.base_glyphs.iter().any(|bg| bg.glyph_id == glyph_id)
    }

    /// Get number of base color glyphs
    pub fn color_glyph_count(&self) -> usize {
        self.base_glyphs.len()
    }
}

/// CBDT/CBLC - Color Bitmap Data Tables
///
/// Provides embedded color bitmap data for glyphs (commonly used for emoji).
/// This is a simplified implementation - full CBDT parsing is complex.
#[derive(Debug, Clone)]
pub struct CbdtTable {
    /// Major version
    pub major_version: u16,
    /// Minor version
    pub minor_version: u16,
    // Note: Full implementation would include bitmap data storage
    // and CBLC location table parsing
}

impl CbdtTable {
    /// Parse CBDT table from raw bytes (simplified)
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        let major_version = cursor.read_u16::<BigEndian>()?;
        let minor_version = cursor.read_u16::<BigEndian>()?;

        // TODO: Full CBDT parsing would include:
        // - Bitmap size tables
        // - Bitmap location tables (from CBLC)
        // - Bitmap data in various formats
        // This is left as a future enhancement

        Ok(CbdtTable {
            major_version,
            minor_version,
        })
    }
}

/// SVG - SVG Table
///
/// Provides SVG glyph definitions.
/// This is a simplified implementation - full SVG parsing is complex.
#[derive(Debug, Clone)]
pub struct SvgTable {
    /// Table version
    pub version: u16,
    // Note: Full implementation would include SVG document storage
}

impl SvgTable {
    /// Parse SVG table from raw bytes (simplified)
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        let version = cursor.read_u16::<BigEndian>()?;
        let _svg_document_list_offset = cursor.read_u32::<BigEndian>()?;
        let _reserved = cursor.read_u32::<BigEndian>()?;

        // TODO: Full SVG parsing would include:
        // - SVG document list
        // - SVG document records
        // - Actual SVG data extraction
        // This is left as a future enhancement

        Ok(SvgTable { version })
    }
}

/// Color font format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    /// COLR/CPAL - Vector color layers
    ColrCpal,
    /// CBDT/CBLC - Bitmap color (emoji)
    Cbdt,
    /// sbix - Apple's standard bitmap graphics
    Sbix,
    /// SVG - SVG-in-OpenType
    Svg,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_rgba() {
        let color = Color::from_rgba(255, 128, 64, 255);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_color_to_rgba_u32() {
        let color = Color::from_rgba(255, 128, 64, 192);
        let rgba = color.to_rgba_u32();
        assert_eq!(rgba, 0xC0FF8040);
    }

    #[test]
    fn test_color_format_variants() {
        // Test all variants can be created
        let _ = ColorFormat::ColrCpal;
        let _ = ColorFormat::Cbdt;
        let _ = ColorFormat::Sbix;
        let _ = ColorFormat::Svg;
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
        assert_eq!(base_glyph.layers, layers);
    }
}
