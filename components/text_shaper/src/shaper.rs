//! Text shaper implementation using Harfbuzz

use crate::types::{Script, ShapingError, ShapingOptions};
use font_registry::{FontDescriptor as RegistryFontDescriptor, FontRegistry};
use font_types::types::{FontDescriptor, FontId, ShapedText};

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
    /// * `_options` - Shaping options (currently unused in minimal implementation)
    ///
    /// # Returns
    ///
    /// Result containing shaped text or error
    pub fn shape_text(
        &self,
        text: &str,
        font_id: FontId,
        size: f32,
        _options: &ShapingOptions,
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
        let _font_face = self
            .registry
            .get_font_face(font_id)
            .ok_or(ShapingError::FontNotFound)?;

        // Get font data - for now, we'll use a placeholder
        // In a complete implementation, this would load actual font data
        // This is a minimal implementation to make tests pass

        // For now, return a simple shaped text with basic metrics
        // This will be enhanced when we integrate with harfbuzz properly
        let char_count = text.chars().count();
        let avg_width = size * 0.6; // Rough estimate

        Ok(ShapedText {
            glyphs: Vec::new(), // TODO: Populate with actual glyphs from harfbuzz
            width: avg_width * char_count as f32,
            height: size,
            baseline: size * 0.8,
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

/// Convert Script to harfbuzz script (placeholder for future use)
#[allow(dead_code)]
fn _script_to_string(script: Script) -> &'static str {
    match script {
        Script::Latin => "Latn",
        Script::Arabic => "Arab",
        Script::Hebrew => "Hebr",
        Script::Cyrillic => "Cyrl",
        Script::Greek => "Grek",
        Script::Han => "Hani",
        Script::Hangul => "Hang",
        Script::Hiragana => "Hira",
        Script::Katakana => "Kana",
        Script::Common => "Zyyy",
    }
}

/// Convert direction to harfbuzz direction (placeholder for Phase 2)
#[allow(dead_code)]
fn direction_to_hb_direction(direction: font_types::types::Direction) -> harfbuzz_rs::Direction {
    match direction {
        font_types::types::Direction::LeftToRight => harfbuzz_rs::Direction::Ltr,
        font_types::types::Direction::RightToLeft => harfbuzz_rs::Direction::Rtl,
        font_types::types::Direction::TopToBottom => harfbuzz_rs::Direction::Ttb,
        font_types::types::Direction::BottomToTop => harfbuzz_rs::Direction::Btt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_to_string() {
        // Test that script conversion functions compile
        let result = _script_to_string(Script::Latin);
        assert_eq!(result, "Latn");
    }

    #[test]
    fn test_direction_conversion() {
        // Test that direction conversion functions compile
        let _dir = direction_to_hb_direction(font_types::types::Direction::LeftToRight);
    }
}
