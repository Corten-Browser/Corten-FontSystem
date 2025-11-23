//! Component message types for browser integration
//!
//! This module defines the message types used for communication between
//! the browser and the font system component.

use font_registry::types::{FontDescriptor, FontId};
use font_types::types::GlyphId;
use glyph_renderer::types::RenderMode;
use text_shaper::types::ShapingOptions;

use crate::system::ShapedText;
use crate::types::FontError;
use glyph_renderer::types::GlyphBitmap;

/// Messages that can be sent to the BrowserComponent
#[derive(Debug, Clone)]
pub enum ComponentMessage {
    /// Load a web font from URL or data
    LoadWebFont(LoadWebFontRequest),
    /// Shape text with the given parameters
    ShapeText(ShapeTextRequest),
    /// Rasterize a glyph to a bitmap
    RasterizeGlyph(RasterizeGlyphRequest),
}

/// Request to load a web font
#[derive(Debug, Clone)]
pub struct LoadWebFontRequest {
    /// Font family name to register
    pub family_name: String,
    /// URL to load font from (optional, use data if None)
    pub url: Option<String>,
    /// Raw font data (optional, use url if None)
    pub data: Option<Vec<u8>>,
    /// Font weight (CSS numeric value, e.g., 400 for normal)
    pub weight: Option<u16>,
    /// Font style ("normal", "italic", "oblique")
    pub style: Option<String>,
}

impl LoadWebFontRequest {
    /// Create a new request to load font from URL
    ///
    /// # Arguments
    ///
    /// * `family_name` - Font family name to register
    /// * `url` - URL to load font from
    ///
    /// # Example
    ///
    /// ```
    /// use font_system_api::messages::LoadWebFontRequest;
    ///
    /// let request = LoadWebFontRequest::from_url(
    ///     "MyFont".to_string(),
    ///     "https://example.com/font.woff2".to_string()
    /// );
    /// ```
    pub fn from_url(family_name: String, url: String) -> Self {
        Self {
            family_name,
            url: Some(url),
            data: None,
            weight: None,
            style: None,
        }
    }

    /// Create a new request to load font from data
    ///
    /// # Arguments
    ///
    /// * `family_name` - Font family name to register
    /// * `data` - Raw font data bytes
    ///
    /// # Example
    ///
    /// ```
    /// use font_system_api::messages::LoadWebFontRequest;
    ///
    /// let data = vec![0u8; 100]; // Font data
    /// let request = LoadWebFontRequest::from_data(
    ///     "MyFont".to_string(),
    ///     data
    /// );
    /// ```
    pub fn from_data(family_name: String, data: Vec<u8>) -> Self {
        Self {
            family_name,
            url: None,
            data: Some(data),
            weight: None,
            style: None,
        }
    }

    /// Set the font weight
    pub fn with_weight(mut self, weight: u16) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Set the font style
    pub fn with_style(mut self, style: String) -> Self {
        self.style = Some(style);
        self
    }
}

/// Request to shape text
#[derive(Debug, Clone)]
pub struct ShapeTextRequest {
    /// Text content to shape
    pub text: String,
    /// Font descriptor for font matching
    pub font_descriptor: FontDescriptor,
    /// Shaping options
    pub options: ShapingOptions,
}

impl ShapeTextRequest {
    /// Create a new shape text request
    ///
    /// # Arguments
    ///
    /// * `text` - Text content to shape
    /// * `font_descriptor` - Font descriptor for font matching
    /// * `options` - Shaping options
    pub fn new(text: String, font_descriptor: FontDescriptor, options: ShapingOptions) -> Self {
        Self {
            text,
            font_descriptor,
            options,
        }
    }
}

/// Request to rasterize a glyph
#[derive(Debug, Clone)]
pub struct RasterizeGlyphRequest {
    /// Font ID to use
    pub font_id: FontId,
    /// Glyph ID to rasterize
    pub glyph_id: GlyphId,
    /// Font size in points
    pub size: f32,
    /// Render mode
    pub mode: RenderMode,
}

impl RasterizeGlyphRequest {
    /// Create a new rasterize glyph request
    ///
    /// # Arguments
    ///
    /// * `font_id` - Font ID to use
    /// * `glyph_id` - Glyph ID to rasterize
    /// * `size` - Font size in points
    /// * `mode` - Render mode
    pub fn new(font_id: FontId, glyph_id: GlyphId, size: f32, mode: RenderMode) -> Self {
        Self {
            font_id,
            glyph_id,
            size,
            mode,
        }
    }
}

/// Response from processing a ComponentMessage
#[derive(Debug, Clone)]
pub enum ComponentResponse {
    /// Font loaded successfully
    FontLoaded(FontLoadedResponse),
    /// Text shaped successfully
    TextShaped(TextShapedResponse),
    /// Glyph rasterized successfully
    GlyphRasterized(GlyphRasterizedResponse),
    /// Error occurred
    Error(FontError),
}

/// Response after successfully loading a font
#[derive(Debug, Clone)]
pub struct FontLoadedResponse {
    /// Assigned font ID
    pub font_id: FontId,
    /// Registered family name
    pub family_name: String,
}

/// Response after successfully shaping text
#[derive(Debug, Clone)]
pub struct TextShapedResponse {
    /// Shaped text result
    pub shaped_text: ShapedText,
}

/// Response after successfully rasterizing a glyph
#[derive(Debug, Clone)]
pub struct GlyphRasterizedResponse {
    /// Rasterized glyph bitmap
    pub bitmap: GlyphBitmap,
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_types::types::Direction;
    use std::collections::HashMap;
    use text_shaper::types::{Language, Script};

    #[test]
    fn test_load_web_font_request_from_url() {
        let request = LoadWebFontRequest::from_url(
            "TestFont".to_string(),
            "https://test.com/font.woff2".to_string(),
        );

        assert_eq!(request.family_name, "TestFont");
        assert_eq!(request.url, Some("https://test.com/font.woff2".to_string()));
        assert!(request.data.is_none());
        assert!(request.weight.is_none());
        assert!(request.style.is_none());
    }

    #[test]
    fn test_load_web_font_request_from_data() {
        let data = vec![0u8, 1, 2, 3];
        let request = LoadWebFontRequest::from_data("TestFont".to_string(), data.clone());

        assert_eq!(request.family_name, "TestFont");
        assert!(request.url.is_none());
        assert_eq!(request.data, Some(data));
    }

    #[test]
    fn test_load_web_font_request_with_weight() {
        let request = LoadWebFontRequest::from_url(
            "TestFont".to_string(),
            "https://test.com/font.woff2".to_string(),
        )
        .with_weight(700);

        assert_eq!(request.weight, Some(700));
    }

    #[test]
    fn test_load_web_font_request_with_style() {
        let request = LoadWebFontRequest::from_url(
            "TestFont".to_string(),
            "https://test.com/font.woff2".to_string(),
        )
        .with_style("italic".to_string());

        assert_eq!(request.style, Some("italic".to_string()));
    }

    #[test]
    fn test_shape_text_request_new() {
        let font_descriptor = FontDescriptor::default();
        let options = ShapingOptions {
            script: Script::Latin,
            language: Language {
                tag: "en-US".to_string(),
            },
            direction: Direction::LeftToRight,
            features: HashMap::new(),
            kerning: true,
            ligatures: true,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        };

        let request = ShapeTextRequest::new(
            "Hello".to_string(),
            font_descriptor.clone(),
            options.clone(),
        );

        assert_eq!(request.text, "Hello");
        assert_eq!(request.font_descriptor, font_descriptor);
    }

    #[test]
    fn test_rasterize_glyph_request_new() {
        let request = RasterizeGlyphRequest::new(1, GlyphId { id: 65 }, 12.0, RenderMode::Gray);

        assert_eq!(request.font_id, 1);
        assert_eq!(request.glyph_id, GlyphId { id: 65 });
        assert!((request.size - 12.0).abs() < f32::EPSILON);
        assert_eq!(request.mode, RenderMode::Gray);
    }

    #[test]
    fn test_component_message_load_web_font() {
        let request = LoadWebFontRequest::from_url(
            "Test".to_string(),
            "https://test.com/font.woff2".to_string(),
        );
        let message = ComponentMessage::LoadWebFont(request);

        match message {
            ComponentMessage::LoadWebFont(req) => {
                assert_eq!(req.family_name, "Test");
            }
            _ => panic!("Expected LoadWebFont message"),
        }
    }

    #[test]
    fn test_component_response_font_loaded() {
        let response = ComponentResponse::FontLoaded(FontLoadedResponse {
            font_id: 42,
            family_name: "TestFont".to_string(),
        });

        match response {
            ComponentResponse::FontLoaded(resp) => {
                assert_eq!(resp.font_id, 42);
                assert_eq!(resp.family_name, "TestFont");
            }
            _ => panic!("Expected FontLoaded response"),
        }
    }

    #[test]
    fn test_component_response_error() {
        let response = ComponentResponse::Error(FontError::FontNotFound);

        match response {
            ComponentResponse::Error(err) => {
                assert_eq!(err, FontError::FontNotFound);
            }
            _ => panic!("Expected Error response"),
        }
    }
}
