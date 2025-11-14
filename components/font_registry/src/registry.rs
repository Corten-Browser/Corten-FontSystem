//! Font registry implementation with in-memory cache

use crate::types::{
    FontDescriptor, FontFace, FontId, FontMetrics, FontStretch, FontStyle, FontWeight,
    RegistryError,
};
use std::collections::HashMap;
use std::path::Path;

/// Font registry for font discovery, loading, and caching
#[derive(Debug)]
pub struct FontRegistry {
    /// Cache of loaded fonts
    fonts: HashMap<FontId, FontFace>,
    /// Next font ID to assign
    next_id: FontId,
}

impl FontRegistry {
    /// Create a new empty font registry
    ///
    /// # Returns
    ///
    /// A new `FontRegistry` instance with no fonts loaded
    ///
    /// # Example
    ///
    /// ```
    /// use font_registry::FontRegistry;
    ///
    /// let registry = FontRegistry::new();
    /// assert_eq!(registry.font_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            next_id: 0,
        }
    }

    /// Get the number of fonts currently loaded
    ///
    /// # Returns
    ///
    /// The count of loaded fonts
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Load font from raw data
    ///
    /// # Arguments
    ///
    /// * `data` - Raw font file data (TrueType, OpenType, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(FontId)` - ID of the loaded font
    /// * `Err(RegistryError)` - If font data is invalid or loading fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use font_registry::FontRegistry;
    /// use std::fs;
    ///
    /// let mut registry = FontRegistry::new();
    /// let font_data = fs::read("path/to/font.ttf").unwrap();
    /// let font_id = registry.load_font_data(font_data).unwrap();
    /// ```
    pub fn load_font_data(&mut self, data: Vec<u8>) -> Result<FontId, RegistryError> {
        // Validate data is not empty
        if data.is_empty() {
            return Err(RegistryError::InvalidFont("Empty font data".to_string()));
        }

        // Parse font using ttf-parser
        let face = ttf_parser::Face::parse(&data, 0)
            .map_err(|e| RegistryError::InvalidFont(format!("Failed to parse font: {:?}", e)))?;

        // Extract font metadata
        let family_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FAMILY)
            .and_then(|name| name.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let postscript_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
            .and_then(|name| name.to_string())
            .unwrap_or_else(|| family_name.clone());

        // Map ttf-parser weight to our FontWeight enum
        let weight = match face.weight().to_number() {
            100 => FontWeight::Thin,
            200 => FontWeight::ExtraLight,
            300 => FontWeight::Light,
            400 => FontWeight::Regular,
            500 => FontWeight::Medium,
            600 => FontWeight::SemiBold,
            700 => FontWeight::Bold,
            800 => FontWeight::ExtraBold,
            900 => FontWeight::Black,
            _ => FontWeight::Regular, // Default to regular for unknown weights
        };

        // Map ttf-parser style to our FontStyle enum
        let style = if face.is_italic() {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };

        // Default stretch (ttf-parser doesn't expose width class easily)
        let stretch = FontStretch::Normal;

        // Extract font metrics
        let units_per_em = face.units_per_em();
        let ascent = face.ascender() as f32;
        let descent = face.descender() as f32;
        let line_gap = face.line_gap() as f32;

        // Default values for metrics not directly available
        let cap_height = face.capital_height().unwrap_or(700) as f32;
        let x_height = face.x_height().unwrap_or(500) as f32;
        let underline_position = face
            .underline_metrics()
            .map(|m| m.position as f32)
            .unwrap_or(-150.0);
        let underline_thickness = face
            .underline_metrics()
            .map(|m| m.thickness as f32)
            .unwrap_or(50.0);

        let metrics = FontMetrics {
            units_per_em,
            ascent,
            descent,
            line_gap,
            cap_height,
            x_height,
            underline_position,
            underline_thickness,
        };

        // Create FontFace with eagerly loaded data
        let font_id = self.next_id;
        let font_face = FontFace {
            id: font_id,
            family_name,
            postscript_name,
            weight,
            style,
            stretch,
            metrics,
            file_path: None,  // No file path for directly loaded data
            data: Some(data), // Data is eagerly loaded
            is_system_font: false,
        };

        // Store in cache
        self.fonts.insert(font_id, font_face);
        self.next_id += 1;

        Ok(font_id)
    }

    /// Load font from file path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to font file
    ///
    /// # Returns
    ///
    /// * `Ok(FontId)` - ID of the loaded font
    /// * `Err(RegistryError)` - If file not found or font data is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use font_registry::FontRegistry;
    /// use std::path::Path;
    ///
    /// let mut registry = FontRegistry::new();
    /// let font_id = registry.load_font_file(Path::new("/path/to/font.ttf")).unwrap();
    /// ```
    pub fn load_font_file(&mut self, path: &Path) -> Result<FontId, RegistryError> {
        let data = std::fs::read(path)
            .map_err(|_| RegistryError::FileNotFound(path.display().to_string()))?;

        self.load_font_data(data)
    }

    /// Load system fonts (platform-specific)
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of fonts loaded
    /// * `Err(RegistryError)` - If system fonts unavailable
    ///
    /// # Implementation
    ///
    /// Uses platform_integration to discover system fonts.
    /// Fonts are loaded with metadata but data is loaded lazily on-demand.
    pub fn load_system_fonts(&mut self) -> Result<usize, RegistryError> {
        // Discover system fonts using platform_integration
        let platform_fonts = platform_integration::discover_system_fonts_detailed();

        if platform_fonts.is_empty() {
            // No fonts found, but this is not necessarily an error
            return Ok(0);
        }

        let mut loaded_count = 0;

        for platform_font in platform_fonts {
            // Convert platform FontWeight to our FontWeight
            let weight = match platform_font.weight {
                platform_integration::FontWeight::Thin => FontWeight::Thin,
                platform_integration::FontWeight::ExtraLight => FontWeight::ExtraLight,
                platform_integration::FontWeight::Light => FontWeight::Light,
                platform_integration::FontWeight::Regular => FontWeight::Regular,
                platform_integration::FontWeight::Medium => FontWeight::Medium,
                platform_integration::FontWeight::SemiBold => FontWeight::SemiBold,
                platform_integration::FontWeight::Bold => FontWeight::Bold,
                platform_integration::FontWeight::ExtraBold => FontWeight::ExtraBold,
                platform_integration::FontWeight::Black => FontWeight::Black,
            };

            // Convert platform FontStyle to our FontStyle
            let style = match platform_font.style {
                platform_integration::FontStyle::Normal => FontStyle::Normal,
                platform_integration::FontStyle::Italic => FontStyle::Italic,
                platform_integration::FontStyle::Oblique(angle) => FontStyle::Oblique(angle),
            };

            // Load font data from file to extract metrics
            // (We keep the file path and can reload data later if needed)
            let font_data = match std::fs::read(&platform_font.path) {
                Ok(data) => data,
                Err(_) => {
                    // Skip fonts we can't read
                    eprintln!(
                        "Warning: Could not read font file: {}",
                        platform_font.path.display()
                    );
                    continue;
                }
            };

            // Parse font to extract metrics
            let face = match ttf_parser::Face::parse(&font_data, 0) {
                Ok(face) => face,
                Err(_) => {
                    // Skip invalid fonts
                    eprintln!(
                        "Warning: Could not parse font file: {}",
                        platform_font.path.display()
                    );
                    continue;
                }
            };

            // Extract font metrics
            let units_per_em = face.units_per_em();
            let ascent = face.ascender() as f32;
            let descent = face.descender() as f32;
            let line_gap = face.line_gap() as f32;
            let cap_height = face.capital_height().unwrap_or(700) as f32;
            let x_height = face.x_height().unwrap_or(500) as f32;
            let underline_position = face
                .underline_metrics()
                .map(|m| m.position as f32)
                .unwrap_or(-150.0);
            let underline_thickness = face
                .underline_metrics()
                .map(|m| m.thickness as f32)
                .unwrap_or(50.0);

            let metrics = FontMetrics {
                units_per_em,
                ascent,
                descent,
                line_gap,
                cap_height,
                x_height,
                underline_position,
                underline_thickness,
            };

            // Get PostScript name (use family name as fallback)
            let postscript_name = face
                .names()
                .into_iter()
                .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
                .and_then(|name| name.to_string())
                .unwrap_or_else(|| platform_font.family_name.clone());

            // Create FontFace entry with lazy loading support
            let font_id = self.next_id;
            let font_face = FontFace {
                id: font_id,
                family_name: platform_font.family_name,
                postscript_name,
                weight,
                style,
                stretch: FontStretch::Normal, // Platform doesn't provide stretch yet
                metrics,
                file_path: Some(platform_font.path),
                data: Some(font_data), // For now, keep data in memory (optimization: lazy load later)
                is_system_font: platform_font.is_system_font,
            };

            // Store in cache
            self.fonts.insert(font_id, font_face);
            self.next_id += 1;
            loaded_count += 1;
        }

        Ok(loaded_count)
    }

    /// Find best matching font for given descriptor
    ///
    /// # Arguments
    ///
    /// * `descriptor` - Font selection criteria
    ///
    /// # Returns
    ///
    /// * `Some(FontId)` - ID of best matching font
    /// * `None` - If no fonts loaded or no match found
    ///
    /// # Example
    ///
    /// ```
    /// use font_registry::{FontRegistry, FontDescriptor};
    ///
    /// let registry = FontRegistry::new();
    /// let descriptor = FontDescriptor::default();
    /// let result = registry.match_font(&descriptor);
    /// assert_eq!(result, None); // No fonts loaded
    /// ```
    pub fn match_font(&self, descriptor: &FontDescriptor) -> Option<FontId> {
        if self.fonts.is_empty() {
            return None;
        }

        // Font matching algorithm:
        // 1. Try exact family name match first
        // 2. Score each font based on weight, style, stretch proximity
        // 3. Return best match

        let mut best_match: Option<(FontId, i32)> = None;

        for (font_id, font) in &self.fonts {
            // Check if family matches (case-insensitive)
            let family_matches = descriptor.family.iter().any(|requested_family| {
                requested_family.to_lowercase() == font.family_name.to_lowercase()
            });

            if !family_matches {
                continue;
            }

            // Calculate match score (lower is better)
            let mut score = 0;

            // Weight difference (0-800 range)
            let weight_diff = (descriptor.weight as i32 - font.weight as i32).abs();
            score += weight_diff;

            // Style mismatch penalty
            if descriptor.style != font.style {
                score += 1000; // High penalty for style mismatch
            }

            // Stretch difference (0-150 range)
            let stretch_diff = (descriptor.stretch as i32 - font.stretch as i32).abs();
            score += stretch_diff;

            // Update best match if this is better
            if let Some((_, best_score)) = best_match {
                if score < best_score {
                    best_match = Some((*font_id, score));
                }
            } else {
                best_match = Some((*font_id, score));
            }
        }

        best_match.map(|(font_id, _)| font_id)
    }

    /// Get loaded font face by ID
    ///
    /// # Arguments
    ///
    /// * `font_id` - Font identifier
    ///
    /// # Returns
    ///
    /// * `Some(&FontFace)` - Reference to font face
    /// * `None` - If font ID not found
    ///
    /// # Example
    ///
    /// ```
    /// use font_registry::FontRegistry;
    ///
    /// let registry = FontRegistry::new();
    /// let result = registry.get_font_face(0);
    /// assert_eq!(result, None); // Font ID 0 not loaded
    /// ```
    pub fn get_font_face(&self, font_id: FontId) -> Option<&FontFace> {
        self.fonts.get(&font_id)
    }

    /// Get font metrics for given font ID and size
    ///
    /// # Arguments
    ///
    /// * `font_id` - Font identifier
    /// * `size` - Font size in pixels
    ///
    /// # Returns
    ///
    /// * `Some(FontMetrics)` - Scaled font metrics
    /// * `None` - If font ID not found or size invalid
    ///
    /// # Example
    ///
    /// ```
    /// use font_registry::FontRegistry;
    ///
    /// let registry = FontRegistry::new();
    /// let result = registry.get_font_metrics(0, 16.0);
    /// assert_eq!(result, None); // Font ID 0 not loaded
    /// ```
    pub fn get_font_metrics(&self, font_id: FontId, size: f32) -> Option<FontMetrics> {
        // Validate size
        if size <= 0.0 {
            return None;
        }

        let font = self.fonts.get(&font_id)?;

        // Scale metrics from font units to pixel size
        let scale = size / font.metrics.units_per_em as f32;

        Some(FontMetrics {
            units_per_em: font.metrics.units_per_em,
            ascent: font.metrics.ascent * scale,
            descent: font.metrics.descent * scale,
            line_gap: font.metrics.line_gap * scale,
            cap_height: font.metrics.cap_height * scale,
            x_height: font.metrics.x_height * scale,
            underline_position: font.metrics.underline_position * scale,
            underline_thickness: font.metrics.underline_thickness * scale,
        })
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = FontRegistry::new();
        assert_eq!(registry.font_count(), 0);
    }

    #[test]
    fn test_match_font_returns_none_when_empty() {
        let registry = FontRegistry::new();
        let descriptor = FontDescriptor::default();
        assert_eq!(registry.match_font(&descriptor), None);
    }
}
