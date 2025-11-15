//! Variable font support (OpenType Font Variations)
//!
//! This module implements parsing for variable fonts, which allow fonts to interpolate
//! between different design axes (weight, width, slant, optical size, etc.).

use crate::types::Tag;
use crate::ParseError;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

/// Variable font axis definition
#[derive(Debug, Clone, PartialEq)]
pub struct VariationAxis {
    /// Axis tag (e.g., "wght" for weight, "wdth" for width)
    pub tag: Tag,
    /// Name ID referencing the 'name' table
    pub name_id: u16,
    /// Minimum value for this axis
    pub min_value: f32,
    /// Default value for this axis
    pub default_value: f32,
    /// Maximum value for this axis
    pub max_value: f32,
}

/// Named instance in variable font
#[derive(Debug, Clone, PartialEq)]
pub struct NamedInstance {
    /// Subfamily name ID referencing the 'name' table
    pub subfamily_name_id: u16,
    /// Coordinates for each axis
    pub coordinates: Vec<f32>,
    /// Optional PostScript name ID (if present)
    pub postscript_name_id: Option<u16>,
}

/// Font Variations Table (fvar)
///
/// The fvar table defines the available variation axes and named instances
/// in a variable font. Each axis specifies a range of values (min, default, max)
/// and named instances provide pre-defined coordinate sets.
#[derive(Debug, Clone, PartialEq)]
pub struct FvarTable {
    /// Available variation axes
    pub axes: Vec<VariationAxis>,
    /// Named instances (e.g., "Bold", "Light", "Condensed")
    pub instances: Vec<NamedInstance>,
}

impl FvarTable {
    /// Parse fvar table from raw data
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the table data is invalid or corrupted.
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        // Read fvar header
        let major_version = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("fvar version: {}", e)))?;
        let minor_version = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("fvar minor version: {}", e)))?;

        // Version must be 1.0
        if major_version != 1 || minor_version != 0 {
            return Err(ParseError::CorruptedData(format!(
                "Unsupported fvar version {}.{}",
                major_version, minor_version
            )));
        }

        let axes_array_offset = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("axes offset: {}", e)))?;
        let _reserved = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("reserved field: {}", e)))?;
        let axis_count = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("axis count: {}", e)))?;
        let axis_size = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("axis size: {}", e)))?;
        let instance_count = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("instance count: {}", e)))?;
        let instance_size = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("instance size: {}", e)))?;

        // Parse axes
        let mut axes = Vec::new();
        cursor.set_position(axes_array_offset as u64);

        for i in 0..axis_count {
            let tag_bytes = cursor
                .read_u32::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(format!("axis {} tag: {}", i, e)))?;
            let tag = Tag::from_bytes(tag_bytes);

            let min_value = Fixed::read(&mut cursor)
                .map_err(|e| ParseError::CorruptedData(format!("axis {} min: {}", i, e)))?
                .to_f32();
            let default_value = Fixed::read(&mut cursor)
                .map_err(|e| ParseError::CorruptedData(format!("axis {} default: {}", i, e)))?
                .to_f32();
            let max_value = Fixed::read(&mut cursor)
                .map_err(|e| ParseError::CorruptedData(format!("axis {} max: {}", i, e)))?
                .to_f32();

            let _flags = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(format!("axis {} flags: {}", i, e)))?;
            let name_id = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(format!("axis {} name_id: {}", i, e)))?;

            axes.push(VariationAxis {
                tag,
                name_id,
                min_value,
                default_value,
                max_value,
            });

            // Skip any extra bytes if axis_size > 20
            if axis_size > 20 {
                let skip_bytes = axis_size - 20;
                cursor.set_position(cursor.position() + skip_bytes as u64);
            }
        }

        // Parse named instances
        let mut instances = Vec::new();
        for i in 0..instance_count {
            let subfamily_name_id = cursor.read_u16::<BigEndian>().map_err(|e| {
                ParseError::CorruptedData(format!("instance {} subfamily_name_id: {}", i, e))
            })?;
            let _flags = cursor
                .read_u16::<BigEndian>()
                .map_err(|e| ParseError::CorruptedData(format!("instance {} flags: {}", i, e)))?;

            let mut coordinates = Vec::new();
            for j in 0..axis_count {
                let coord = Fixed::read(&mut cursor).map_err(|e| {
                    ParseError::CorruptedData(format!("instance {} coord {}: {}", i, j, e))
                })?;
                coordinates.push(coord.to_f32());
            }

            // Optional postScriptNameID (if instance_size > base_size)
            let base_size = 4 + (axis_count as usize * 4);
            let postscript_name_id = if instance_size as usize > base_size {
                Some(cursor.read_u16::<BigEndian>().map_err(|e| {
                    ParseError::CorruptedData(format!("instance {} postscript_name_id: {}", i, e))
                })?)
            } else {
                None
            };

            // Skip any extra bytes if instance_size > expected
            let expected_size = if postscript_name_id.is_some() {
                base_size + 2
            } else {
                base_size
            };
            if instance_size as usize > expected_size {
                let skip_bytes = instance_size as usize - expected_size;
                cursor.set_position(cursor.position() + skip_bytes as u64);
            }

            instances.push(NamedInstance {
                subfamily_name_id,
                coordinates,
                postscript_name_id,
            });
        }

        Ok(FvarTable { axes, instances })
    }

    /// Get axis by tag (e.g., "wght" for weight)
    pub fn get_axis(&self, tag: Tag) -> Option<&VariationAxis> {
        self.axes.iter().find(|axis| axis.tag == tag)
    }

    /// Check if font is variable
    pub fn is_variable(&self) -> bool {
        !self.axes.is_empty()
    }
}

/// Axis Variations Table (avar)
///
/// The avar table allows for non-linear interpolation along variation axes.
/// It maps axis values to achieve better design control.
#[derive(Debug, Clone, PartialEq)]
pub struct AvarTable {
    /// Segment maps for each axis
    pub axis_segment_maps: Vec<AxisSegmentMap>,
}

/// Segment map for a single axis
#[derive(Debug, Clone, PartialEq)]
pub struct AxisSegmentMap {
    /// Mappings from input coordinates to output coordinates
    pub mappings: Vec<(f32, f32)>, // (from_coord, to_coord)
}

impl AvarTable {
    /// Parse avar table from raw data
    ///
    /// # Arguments
    ///
    /// * `data` - Raw table data
    /// * `axis_count` - Number of axes (from fvar table)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the table data is invalid or corrupted.
    pub fn parse(data: &[u8], axis_count: usize) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        // Read avar header
        let major_version = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("avar version: {}", e)))?;
        let minor_version = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("avar minor version: {}", e)))?;

        // Version must be 1.0
        if major_version != 1 || minor_version != 0 {
            return Err(ParseError::CorruptedData(format!(
                "Unsupported avar version {}.{}",
                major_version, minor_version
            )));
        }

        let _reserved = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("reserved field: {}", e)))?;
        let axis_count_avar = cursor
            .read_u16::<BigEndian>()
            .map_err(|e| ParseError::CorruptedData(format!("axis count: {}", e)))?;

        if axis_count_avar as usize != axis_count {
            return Err(ParseError::CorruptedData(format!(
                "avar axis count mismatch: expected {}, got {}",
                axis_count, axis_count_avar
            )));
        }

        // Parse segment maps for each axis
        let mut axis_segment_maps = Vec::new();
        for i in 0..axis_count {
            let position_map_count = cursor.read_u16::<BigEndian>().map_err(|e| {
                ParseError::CorruptedData(format!("axis {} position_map_count: {}", i, e))
            })?;

            let mut mappings = Vec::new();
            for j in 0..position_map_count {
                let from_coord = Fixed::read(&mut cursor)
                    .map_err(|e| {
                        ParseError::CorruptedData(format!("axis {} mapping {} from: {}", i, j, e))
                    })?
                    .to_f32();
                let to_coord = Fixed::read(&mut cursor)
                    .map_err(|e| {
                        ParseError::CorruptedData(format!("axis {} mapping {} to: {}", i, j, e))
                    })?
                    .to_f32();
                mappings.push((from_coord, to_coord));
            }

            axis_segment_maps.push(AxisSegmentMap { mappings });
        }

        Ok(AvarTable { axis_segment_maps })
    }

    /// Apply axis value mapping
    ///
    /// Maps an input coordinate to an output coordinate using the segment map
    /// for the specified axis.
    pub fn map_value(&self, axis_index: usize, value: f32) -> f32 {
        if axis_index >= self.axis_segment_maps.len() {
            return value;
        }

        let map = &self.axis_segment_maps[axis_index];

        // Find segment containing value
        for window in map.mappings.windows(2) {
            let (from1, to1) = window[0];
            let (from2, to2) = window[1];

            if value >= from1 && value <= from2 {
                // Linear interpolation
                if (from2 - from1).abs() < f32::EPSILON {
                    return to1; // Avoid division by zero
                }
                let t = (value - from1) / (from2 - from1);
                return to1 + t * (to2 - to1);
            }
        }

        value
    }
}

/// Fixed-point number (16.16 format)
///
/// OpenType uses 16.16 fixed-point numbers for precise representation
/// of fractional values.
#[derive(Debug, Copy, Clone)]
struct Fixed(i32);

impl Fixed {
    fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, ParseError> {
        Ok(Fixed(reader.read_i32::<BigEndian>().map_err(|e| {
            ParseError::CorruptedData(format!("Fixed value: {}", e))
        })?))
    }

    fn to_f32(self) -> f32 {
        (self.0 as f32) / 65536.0
    }
}

/// Coordinates for variable font instance creation
///
/// Represents a set of axis coordinates that define a specific instance
/// of a variable font.
#[derive(Debug, Clone, PartialEq)]
pub struct VariationCoordinates {
    /// Axis tag to value mappings
    pub values: Vec<(Tag, f32)>,
}

impl VariationCoordinates {
    /// Create a new empty coordinate set
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Set coordinate for a specific axis
    pub fn set_axis(&mut self, tag: Tag, value: f32) {
        if let Some(coord) = self.values.iter_mut().find(|(t, _)| *t == tag) {
            coord.1 = value;
        } else {
            self.values.push((tag, value));
        }
    }

    /// Get coordinate for a specific axis
    pub fn get_axis(&self, tag: Tag) -> Option<f32> {
        self.values.iter().find(|(t, _)| *t == tag).map(|(_, v)| *v)
    }
}

impl Default for VariationCoordinates {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_to_f32() {
        // 1.0 in 16.16 fixed point is 65536
        let fixed = Fixed(65536);
        assert!((fixed.to_f32() - 1.0).abs() < 0.0001);

        // 0.5 in 16.16 fixed point is 32768
        let fixed = Fixed(32768);
        assert!((fixed.to_f32() - 0.5).abs() < 0.0001);

        // -1.5 in 16.16 fixed point is -98304
        let fixed = Fixed(-98304);
        assert!((fixed.to_f32() - (-1.5)).abs() < 0.0001);
    }

    #[test]
    fn test_variation_coordinates_new() {
        let coords = VariationCoordinates::new();
        assert!(coords.values.is_empty());
    }

    #[test]
    fn test_variation_coordinates_set_get() {
        let mut coords = VariationCoordinates::new();
        let tag = Tag::from_bytes(0x77676874); // 'wght'

        coords.set_axis(tag, 700.0);
        assert_eq!(coords.get_axis(tag), Some(700.0));

        // Update existing axis
        coords.set_axis(tag, 800.0);
        assert_eq!(coords.get_axis(tag), Some(800.0));
        assert_eq!(coords.values.len(), 1); // Should not add duplicate
    }

    #[test]
    fn test_variation_coordinates_multiple_axes() {
        let mut coords = VariationCoordinates::new();
        let wght = Tag::from_bytes(0x77676874); // 'wght'
        let wdth = Tag::from_bytes(0x77647468); // 'wdth'

        coords.set_axis(wght, 700.0);
        coords.set_axis(wdth, 100.0);

        assert_eq!(coords.get_axis(wght), Some(700.0));
        assert_eq!(coords.get_axis(wdth), Some(100.0));
        assert_eq!(coords.values.len(), 2);
    }

    #[test]
    fn test_variation_coordinates_get_nonexistent() {
        let coords = VariationCoordinates::new();
        let tag = Tag::from_bytes(0x77676874); // 'wght'
        assert_eq!(coords.get_axis(tag), None);
    }

    #[test]
    fn test_avar_map_value_linear_interpolation() {
        let map = AxisSegmentMap {
            mappings: vec![(-1.0, -1.0), (0.0, 0.0), (1.0, 1.0)],
        };
        let avar = AvarTable {
            axis_segment_maps: vec![map],
        };

        // Test exact points
        assert!((avar.map_value(0, -1.0) - (-1.0)).abs() < 0.0001);
        assert!((avar.map_value(0, 0.0) - 0.0).abs() < 0.0001);
        assert!((avar.map_value(0, 1.0) - 1.0).abs() < 0.0001);

        // Test interpolated point
        assert!((avar.map_value(0, 0.5) - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_avar_map_value_nonlinear() {
        // Non-linear mapping: input -1 to 0 maps to output -1 to -0.5
        //                     input 0 to 1 maps to output -0.5 to 1
        let map = AxisSegmentMap {
            mappings: vec![(-1.0, -1.0), (0.0, -0.5), (1.0, 1.0)],
        };
        let avar = AvarTable {
            axis_segment_maps: vec![map],
        };

        // At input 0.0, output should be -0.5
        assert!((avar.map_value(0, 0.0) - (-0.5)).abs() < 0.0001);

        // At input 0.5, should interpolate between -0.5 and 1.0
        // t = 0.5, output = -0.5 + 0.5 * (1.0 - (-0.5)) = -0.5 + 0.75 = 0.25
        assert!((avar.map_value(0, 0.5) - 0.25).abs() < 0.0001);
    }

    #[test]
    fn test_avar_map_value_out_of_bounds() {
        let map = AxisSegmentMap {
            mappings: vec![(-1.0, -1.0), (0.0, 0.0), (1.0, 1.0)],
        };
        let avar = AvarTable {
            axis_segment_maps: vec![map],
        };

        // Values outside range should be returned unchanged
        assert!((avar.map_value(0, -2.0) - (-2.0)).abs() < 0.0001);
        assert!((avar.map_value(0, 2.0) - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_avar_map_value_invalid_axis() {
        let map = AxisSegmentMap {
            mappings: vec![(-1.0, -1.0), (0.0, 0.0), (1.0, 1.0)],
        };
        let avar = AvarTable {
            axis_segment_maps: vec![map],
        };

        // Invalid axis index should return value unchanged
        assert!((avar.map_value(5, 0.5) - 0.5).abs() < 0.0001);
    }
}
