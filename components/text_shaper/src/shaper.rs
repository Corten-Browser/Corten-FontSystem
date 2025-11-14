//! Text shaper implementation using Harfbuzz

use std::str::FromStr;

use crate::types::{Script, ShapingError, ShapingOptions};
use font_registry::{FontDescriptor as RegistryFontDescriptor, FontRegistry};
use font_types::types::{
    Direction, FontDescriptor, FontId, GlyphId, Point, PositionedGlyph, ShapedText, Vector,
};
use harfbuzz_rs::{Face, Font, Tag, UnicodeBuffer};

/// Text shaping engine
pub struct TextShaper<'a> {
    /// Reference to font registry
    registry: &'a FontRegistry,
}

impl<'a> TextShaper<'a> {
    /// Create new text shaper
    ///
    /// # Arguments
    ///
    /// * `registry` - Font registry containing loaded fonts
    ///
    /// # Returns
    ///
    /// New TextShaper instance
    pub fn new(registry: &'a FontRegistry) -> Self {
        Self { registry }
    }

    /// Shape text with specific font
    ///
    /// # Arguments
    ///
    /// * `text` - Text to shape
    /// * `font_id` - Font identifier
    /// * `size` - Font size in pixels
    /// * `options` - Shaping options
    ///
    /// # Returns
    ///
    /// Result containing shaped text or error
    pub fn shape_text(
        &self,
        text: &str,
        font_id: FontId,
        size: f32,
        options: &ShapingOptions,
    ) -> Result<ShapedText, ShapingError> {
        // Validate input
        if text.is_empty() {
            return Ok(ShapedText {
                glyphs: Vec::new(),
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
            });
        }

        // Get font face from registry
        let font_face = self
            .registry
            .get_font_face(font_id)
            .ok_or(ShapingError::FontNotFound)?;

        // Get font data
        let font_data = font_face.data();

        // Create Harfbuzz font from font data
        let hb_face = Face::from_bytes(font_data, 0);
        let mut hb_font = Font::new(hb_face);

        // Set font size (Harfbuzz uses 26.6 fixed point format)
        let font_units_per_em = font_face.metrics.units_per_em as i32;
        let scale = (size * 64.0) as i32; // Convert to 26.6 fixed point
        hb_font.set_scale(scale, scale);
        hb_font.set_ppem(size as u32, size as u32);

        // Create buffer with text
        let mut buffer = UnicodeBuffer::new();
        buffer = buffer.add_str(text);

        // Set buffer properties
        buffer = buffer.set_direction(direction_to_hb_direction(options.direction));
        buffer = buffer.set_script(script_to_tag(options.script));

        if let Ok(lang) = harfbuzz_rs::Language::from_str(&options.language.tag) {
            buffer = buffer.set_language(lang);
        }

        // Apply OpenType features
        let features: Vec<harfbuzz_rs::Feature> = options
            .features
            .iter()
            .filter_map(|(tag, value)| {
                if tag.len() == 4 {
                    let tag_bytes = tag.as_bytes();
                    Some(harfbuzz_rs::Feature::new(
                        Tag::new(
                            tag_bytes[0] as char,
                            tag_bytes[1] as char,
                            tag_bytes[2] as char,
                            tag_bytes[3] as char,
                        ),
                        *value,
                        ..
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Shape the text
        let output = harfbuzz_rs::shape(&hb_font, buffer, &features);

        // Extract glyph info and positions
        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        // Convert to our format
        let mut glyphs = Vec::with_capacity(infos.len());
        let mut cursor_x = 0.0;
        let mut cursor_y = 0.0;
        let mut total_width = 0.0;

        for (info, pos) in infos.iter().zip(positions.iter()) {
            // Convert from 26.6 fixed point to float
            let x_advance = pos.x_advance as f32 / 64.0;
            let y_advance = pos.y_advance as f32 / 64.0;
            let x_offset = pos.x_offset as f32 / 64.0;
            let y_offset = pos.y_offset as f32 / 64.0;

            // Apply letter spacing
            let adjusted_x_advance = x_advance + options.letter_spacing;

            glyphs.push(PositionedGlyph {
                glyph_id: GlyphId {
                    id: info.codepoint,
                },
                font_id,
                position: Point {
                    x: cursor_x + x_offset,
                    y: cursor_y + y_offset,
                },
                advance: Vector {
                    x: adjusted_x_advance,
                    y: y_advance,
                },
                offset: Vector {
                    x: x_offset,
                    y: y_offset,
                },
            });

            cursor_x += adjusted_x_advance;
            cursor_y += y_advance;
            total_width = cursor_x;
        }

        // Calculate height and baseline from font metrics
        let scale_factor = size / font_units_per_em as f32;
        let height = (font_face.metrics.ascent - font_face.metrics.descent) * scale_factor;
        let baseline = font_face.metrics.ascent * scale_factor;

        Ok(ShapedText {
            glyphs,
            width: total_width,
            height,
            baseline,
        })
    }

    /// Shape text with font fallback
    ///
    /// # Arguments
    ///
    /// * `text` - Text to shape
    /// * `descriptor` - Font descriptor with fallback chain
    /// * `options` - Shaping options
    ///
    /// # Returns
    ///
    /// Result containing shaped text or error
    pub fn shape_text_with_fallback(
        &self,
        text: &str,
        descriptor: &FontDescriptor,
        options: &ShapingOptions,
    ) -> Result<ShapedText, ShapingError> {
        // Validate input
        if text.is_empty() {
            return Ok(ShapedText {
                glyphs: Vec::new(),
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
            });
        }

        // Convert font_types::FontDescriptor to font_registry::FontDescriptor
        let registry_descriptor = RegistryFontDescriptor {
            family: descriptor.family.clone(),
            weight: descriptor.weight,
            style: descriptor.style,
            stretch: descriptor.stretch,
            size: descriptor.size,
        };

        // Try to match font using descriptor
        let font_id = self
            .registry
            .match_font(&registry_descriptor)
            .ok_or(ShapingError::FontNotFound)?;

        // Use shape_text with the matched font
        self.shape_text(text, font_id, descriptor.size, options)
    }
}

/// Convert Script to harfbuzz Tag
fn script_to_tag(script: Script) -> Tag {
    match script {
        Script::Latin => Tag::new('l', 'a', 't', 'n'),
        Script::Arabic => Tag::new('a', 'r', 'a', 'b'),
        Script::Hebrew => Tag::new('h', 'e', 'b', 'r'),
        Script::Cyrillic => Tag::new('c', 'y', 'r', 'l'),
        Script::Greek => Tag::new('g', 'r', 'e', 'k'),
        Script::Han => Tag::new('h', 'a', 'n', 'i'),
        Script::Hangul => Tag::new('h', 'a', 'n', 'g'),
        Script::Hiragana => Tag::new('h', 'i', 'r', 'a'),
        Script::Katakana => Tag::new('k', 'a', 'n', 'a'),
        Script::Common => Tag::new('z', 'y', 'y', 'y'),
    }
}

/// Convert direction to harfbuzz direction
fn direction_to_hb_direction(direction: Direction) -> harfbuzz_rs::Direction {
    match direction {
        Direction::LeftToRight => harfbuzz_rs::Direction::Ltr,
        Direction::RightToLeft => harfbuzz_rs::Direction::Rtl,
        Direction::TopToBottom => harfbuzz_rs::Direction::Ttb,
        Direction::BottomToTop => harfbuzz_rs::Direction::Btt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_to_tag() {
        // Test that script conversion works
        assert_eq!(
            script_to_tag(Script::Latin),
            Tag::new('l', 'a', 't', 'n')
        );
        assert_eq!(
            script_to_tag(Script::Arabic),
            Tag::new('a', 'r', 'a', 'b')
        );
    }

    #[test]
    fn test_direction_conversion() {
        // Test that direction conversion works
        assert_eq!(
            direction_to_hb_direction(Direction::LeftToRight),
            harfbuzz_rs::Direction::Ltr
        );
        assert_eq!(
            direction_to_hb_direction(Direction::RightToLeft),
            harfbuzz_rs::Direction::Rtl
        );
    }
}
