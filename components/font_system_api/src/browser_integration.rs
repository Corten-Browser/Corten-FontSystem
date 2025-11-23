//! Browser integration layer for the font system
//!
//! This module provides integration points for browser components:
//! - CSS Engine font requests (FEAT-091)
//! - Rendering engine integration (FEAT-092)
//! - Network stack web fonts (FEAT-093)
//! - CORS validation (FEAT-094)
//! - Browser shell preferences (FEAT-095)

use crate::types::FontError;
use font_registry::types::{FontDescriptor, FontId, FontMetrics, FontStretch, FontStyle, FontWeight};
use std::collections::HashMap;

// ============================================================================
// FEAT-091: CSS Engine Font Requests
// ============================================================================

/// Request for font resolution from CSS engine
#[derive(Debug, Clone)]
pub struct CssFontRequest {
    /// Font family stack (e.g., ["Arial", "Helvetica", "sans-serif"])
    pub family_stack: Vec<String>,
    /// Requested font weight
    pub weight: CssFontWeight,
    /// Requested font style
    pub style: CssFontStyle,
    /// Requested font stretch
    pub stretch: CssFontStretch,
    /// Font size in CSS pixels
    pub size: f32,
    /// Language hint for font selection
    pub language: Option<String>,
    /// Whether to synthesize bold if not available
    pub allow_synthetic_bold: bool,
    /// Whether to synthesize italic if not available
    pub allow_synthetic_italic: bool,
}

impl Default for CssFontRequest {
    fn default() -> Self {
        Self {
            family_stack: vec!["sans-serif".to_string()],
            weight: CssFontWeight::Normal,
            style: CssFontStyle::Normal,
            stretch: CssFontStretch::Normal,
            size: 16.0,
            language: None,
            allow_synthetic_bold: true,
            allow_synthetic_italic: true,
        }
    }
}

/// CSS font-weight values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CssFontWeight {
    /// 100
    Thin,
    /// 200
    ExtraLight,
    /// 300
    Light,
    /// 400
    Normal,
    /// 500
    Medium,
    /// 600
    SemiBold,
    /// 700
    Bold,
    /// 800
    ExtraBold,
    /// 900
    Black,
    /// Custom numeric value (1-1000)
    Custom(u16),
}

impl CssFontWeight {
    /// Convert to numeric value
    pub fn to_numeric(&self) -> u16 {
        match self {
            CssFontWeight::Thin => 100,
            CssFontWeight::ExtraLight => 200,
            CssFontWeight::Light => 300,
            CssFontWeight::Normal => 400,
            CssFontWeight::Medium => 500,
            CssFontWeight::SemiBold => 600,
            CssFontWeight::Bold => 700,
            CssFontWeight::ExtraBold => 800,
            CssFontWeight::Black => 900,
            CssFontWeight::Custom(v) => *v,
        }
    }

    /// Convert to FontWeight enum
    pub fn to_font_weight(&self) -> FontWeight {
        match self.to_numeric() {
            0..=150 => FontWeight::Thin,
            151..=250 => FontWeight::ExtraLight,
            251..=350 => FontWeight::Light,
            351..=450 => FontWeight::Regular,
            451..=550 => FontWeight::Medium,
            551..=650 => FontWeight::SemiBold,
            651..=750 => FontWeight::Bold,
            751..=850 => FontWeight::ExtraBold,
            _ => FontWeight::Black,
        }
    }
}

/// CSS font-style values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CssFontStyle {
    /// Normal upright text
    Normal,
    /// Italic text
    Italic,
    /// Oblique text with optional angle
    Oblique(Option<f32>),
}

impl CssFontStyle {
    /// Convert to FontStyle enum
    pub fn to_font_style(&self) -> FontStyle {
        match self {
            CssFontStyle::Normal => FontStyle::Normal,
            CssFontStyle::Italic => FontStyle::Italic,
            CssFontStyle::Oblique(angle) => FontStyle::Oblique(angle.unwrap_or(14.0)),
        }
    }
}

/// CSS font-stretch values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CssFontStretch {
    /// 50%
    UltraCondensed,
    /// 62.5%
    ExtraCondensed,
    /// 75%
    Condensed,
    /// 87.5%
    SemiCondensed,
    /// 100%
    Normal,
    /// 112.5%
    SemiExpanded,
    /// 125%
    Expanded,
    /// 150%
    ExtraExpanded,
    /// 200%
    UltraExpanded,
    /// Custom percentage
    Custom(f32),
}

impl CssFontStretch {
    /// Convert to FontStretch enum
    pub fn to_font_stretch(&self) -> FontStretch {
        match self {
            CssFontStretch::UltraCondensed => FontStretch::UltraCondensed,
            CssFontStretch::ExtraCondensed => FontStretch::ExtraCondensed,
            CssFontStretch::Condensed => FontStretch::Condensed,
            CssFontStretch::SemiCondensed => FontStretch::SemiCondensed,
            CssFontStretch::Normal => FontStretch::Normal,
            CssFontStretch::SemiExpanded => FontStretch::SemiExpanded,
            CssFontStretch::Expanded => FontStretch::Expanded,
            CssFontStretch::ExtraExpanded => FontStretch::ExtraExpanded,
            CssFontStretch::UltraExpanded | CssFontStretch::Custom(_) => FontStretch::UltraExpanded,
        }
    }
}

/// Response for CSS font request
#[derive(Debug, Clone)]
pub struct CssFontResponse {
    /// Resolved font ID
    pub font_id: FontId,
    /// Actual family name used
    pub actual_family: String,
    /// Whether bold was synthesized
    pub synthetic_bold: bool,
    /// Whether italic was synthesized
    pub synthetic_italic: bool,
    /// Font metrics at requested size
    pub metrics: FontMetrics,
}

/// CSS font request handler
#[derive(Debug, Default)]
pub struct CssFontResolver {
    /// Cache of resolved fonts
    cache: HashMap<String, CssFontResponse>,
}

impl CssFontResolver {
    /// Create a new CSS font resolver
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Resolve a CSS font request to a font descriptor
    pub fn resolve_request(&self, request: &CssFontRequest) -> FontDescriptor {
        FontDescriptor {
            family: request.family_stack.clone(),
            weight: request.weight.to_font_weight(),
            style: request.style.to_font_style(),
            stretch: request.stretch.to_font_stretch(),
            size: request.size,
        }
    }

    /// Cache a font resolution result
    pub fn cache_result(&mut self, key: String, response: CssFontResponse) {
        self.cache.insert(key, response);
    }

    /// Get cached font resolution
    pub fn get_cached(&self, key: &str) -> Option<&CssFontResponse> {
        self.cache.get(key)
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Generate cache key for request
    pub fn cache_key(request: &CssFontRequest) -> String {
        format!(
            "{}:{:?}:{:?}:{:?}:{}",
            request.family_stack.join(","),
            request.weight,
            request.style,
            request.stretch,
            request.size
        )
    }
}

// ============================================================================
// FEAT-092: Rendering Engine Integration
// ============================================================================

/// Shaped glyph for rendering
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u16,
    /// X offset from current position (in pixels)
    pub x_offset: f32,
    /// Y offset from current position (in pixels)
    pub y_offset: f32,
    /// X advance after this glyph (in pixels)
    pub x_advance: f32,
    /// Y advance after this glyph (in pixels)
    pub y_advance: f32,
    /// Character index in original text
    pub cluster: usize,
}

/// Shaped text run for rendering
#[derive(Debug, Clone)]
pub struct ShapedRun {
    /// Font ID used for this run
    pub font_id: FontId,
    /// Glyphs in this run
    pub glyphs: Vec<ShapedGlyph>,
    /// Start index in original text
    pub start: usize,
    /// End index in original text
    pub end: usize,
    /// Run direction (true = RTL)
    pub rtl: bool,
}

/// Text to render with font information
#[derive(Debug, Clone)]
pub struct RenderText {
    /// Text runs with shaped glyphs
    pub runs: Vec<ShapedRun>,
    /// Total width of the text
    pub width: f32,
    /// Total height of the text
    pub height: f32,
    /// Baseline offset from top
    pub baseline: f32,
}

impl RenderText {
    /// Create empty render text
    pub fn empty() -> Self {
        Self {
            runs: Vec::new(),
            width: 0.0,
            height: 0.0,
            baseline: 0.0,
        }
    }

    /// Get total glyph count
    pub fn glyph_count(&self) -> usize {
        self.runs.iter().map(|r| r.glyphs.len()).sum()
    }
}

/// Rendering engine integration interface
pub trait RenderingEngineInterface {
    /// Submit shaped text for rendering
    fn submit_text(&mut self, text: &RenderText, x: f32, y: f32) -> Result<(), FontError>;

    /// Invalidate cached glyphs for a font
    fn invalidate_font_cache(&mut self, font_id: FontId);

    /// Get current rendering statistics
    fn get_stats(&self) -> RenderingStats;
}

/// Rendering statistics
#[derive(Debug, Clone, Default)]
pub struct RenderingStats {
    /// Number of glyphs rendered
    pub glyphs_rendered: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
}

// ============================================================================
// FEAT-093: Network Stack Web Fonts
// ============================================================================

/// Web font download request
#[derive(Debug, Clone)]
pub struct WebFontRequest {
    /// URL to download from
    pub url: String,
    /// Expected MIME type
    pub mime_type: Option<String>,
    /// Origin of the requesting document
    pub origin: String,
    /// Priority (higher = more urgent)
    pub priority: u8,
}

/// Web font download response
#[derive(Debug, Clone)]
pub struct WebFontResponse {
    /// HTTP status code
    pub status: u16,
    /// Font data (if successful)
    pub data: Option<Vec<u8>>,
    /// Content type header
    pub content_type: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// State of a web font download
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebFontDownloadState {
    /// Not started
    Pending,
    /// Download in progress
    Downloading,
    /// Download complete, parsing
    Parsing,
    /// Font ready to use
    Ready,
    /// Download or parsing failed
    Failed,
}

/// Web font download manager
#[derive(Debug)]
pub struct WebFontDownloader {
    /// Active downloads
    downloads: HashMap<String, WebFontDownloadState>,
    /// Maximum concurrent downloads
    max_concurrent: usize,
    /// Timeout in milliseconds
    timeout_ms: u32,
}

impl WebFontDownloader {
    /// Create a new web font downloader
    pub fn new() -> Self {
        Self {
            downloads: HashMap::new(),
            max_concurrent: 4,
            timeout_ms: 30000,
        }
    }

    /// Queue a font download
    pub fn queue_download(&mut self, request: &WebFontRequest) -> bool {
        let active_count = self.downloads.values()
            .filter(|s| **s == WebFontDownloadState::Downloading)
            .count();

        if active_count >= self.max_concurrent {
            return false;
        }

        self.downloads.insert(request.url.clone(), WebFontDownloadState::Pending);
        true
    }

    /// Get download state
    pub fn get_state(&self, url: &str) -> Option<WebFontDownloadState> {
        self.downloads.get(url).copied()
    }

    /// Update download state
    pub fn set_state(&mut self, url: &str, state: WebFontDownloadState) {
        if let Some(entry) = self.downloads.get_mut(url) {
            *entry = state;
        }
    }

    /// Remove completed download
    pub fn remove(&mut self, url: &str) -> Option<WebFontDownloadState> {
        self.downloads.remove(url)
    }

    /// Set maximum concurrent downloads
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max;
    }

    /// Set timeout
    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    /// Get number of active downloads
    pub fn active_count(&self) -> usize {
        self.downloads.values()
            .filter(|s| **s == WebFontDownloadState::Downloading)
            .count()
    }
}

impl Default for WebFontDownloader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FEAT-094: CORS Validation
// ============================================================================

/// CORS validation result
#[derive(Debug, Clone, PartialEq)]
pub enum CorsResult {
    /// Request is allowed
    Allowed,
    /// Request is blocked
    Blocked(CorsError),
}

/// CORS error types
#[derive(Debug, Clone, PartialEq)]
pub enum CorsError {
    /// Origin not in allowed list
    OriginNotAllowed(String),
    /// Missing Access-Control-Allow-Origin header
    MissingAllowOrigin,
    /// Invalid Access-Control-Allow-Origin header
    InvalidAllowOrigin,
    /// Credentials not allowed
    CredentialsNotAllowed,
    /// Method not allowed
    MethodNotAllowed,
}

impl std::fmt::Display for CorsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorsError::OriginNotAllowed(origin) => {
                write!(f, "Origin '{}' is not allowed", origin)
            }
            CorsError::MissingAllowOrigin => {
                write!(f, "Missing Access-Control-Allow-Origin header")
            }
            CorsError::InvalidAllowOrigin => {
                write!(f, "Invalid Access-Control-Allow-Origin header")
            }
            CorsError::CredentialsNotAllowed => {
                write!(f, "Credentials not allowed for cross-origin request")
            }
            CorsError::MethodNotAllowed => {
                write!(f, "HTTP method not allowed for cross-origin request")
            }
        }
    }
}

/// CORS validator for web font requests
#[derive(Debug)]
pub struct CorsValidator {
    /// Whether to enforce CORS
    enforce: bool,
    /// Allowed origins (None = allow all with wildcard)
    allowed_origins: Option<Vec<String>>,
}

impl CorsValidator {
    /// Create a new CORS validator
    pub fn new() -> Self {
        Self {
            enforce: true,
            allowed_origins: None,
        }
    }

    /// Validate a web font request
    pub fn validate(
        &self,
        request_origin: &str,
        response_allow_origin: Option<&str>,
        response_allow_credentials: bool,
    ) -> CorsResult {
        if !self.enforce {
            return CorsResult::Allowed;
        }

        // Check if response has ACAO header
        let allow_origin = match response_allow_origin {
            Some(origin) => origin,
            None => return CorsResult::Blocked(CorsError::MissingAllowOrigin),
        };

        // Check if origin is allowed
        if allow_origin == "*" {
            // Wildcard allows any origin (without credentials)
            if response_allow_credentials {
                return CorsResult::Blocked(CorsError::CredentialsNotAllowed);
            }
            return CorsResult::Allowed;
        }

        // Check exact match
        if allow_origin == request_origin {
            return CorsResult::Allowed;
        }

        // Check against allowed origins list
        if let Some(ref allowed) = self.allowed_origins {
            if allowed.iter().any(|o| o == request_origin) {
                return CorsResult::Allowed;
            }
        }

        CorsResult::Blocked(CorsError::OriginNotAllowed(request_origin.to_string()))
    }

    /// Set whether to enforce CORS
    pub fn set_enforce(&mut self, enforce: bool) {
        self.enforce = enforce;
    }

    /// Add an allowed origin
    pub fn add_allowed_origin(&mut self, origin: String) {
        if self.allowed_origins.is_none() {
            self.allowed_origins = Some(Vec::new());
        }
        if let Some(ref mut origins) = self.allowed_origins {
            origins.push(origin);
        }
    }

    /// Check if same-origin
    pub fn is_same_origin(url1: &str, url2: &str) -> bool {
        let origin1 = extract_origin(url1);
        let origin2 = extract_origin(url2);
        origin1 == origin2
    }
}

impl Default for CorsValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract origin from URL
fn extract_origin(url: &str) -> String {
    // Simple origin extraction (protocol + host + port)
    if let Some(proto_end) = url.find("://") {
        let after_proto = &url[proto_end + 3..];
        if let Some(path_start) = after_proto.find('/') {
            return url[..proto_end + 3 + path_start].to_string();
        }
        return url.to_string();
    }
    url.to_string()
}

// ============================================================================
// FEAT-095: Browser Shell Preferences
// ============================================================================

/// Font preferences from browser shell
#[derive(Debug, Clone)]
pub struct FontPreferences {
    /// Default serif font family
    pub default_serif: String,
    /// Default sans-serif font family
    pub default_sans_serif: String,
    /// Default monospace font family
    pub default_monospace: String,
    /// Default cursive font family
    pub default_cursive: String,
    /// Default fantasy font family
    pub default_fantasy: String,
    /// Default font size in pixels
    pub default_size: f32,
    /// Minimum font size in pixels
    pub minimum_size: f32,
    /// Whether to use custom fonts
    pub allow_custom_fonts: bool,
    /// Whether to use web fonts
    pub allow_web_fonts: bool,
    /// Font smoothing preference
    pub font_smoothing: FontSmoothing,
    /// Language-specific font preferences
    pub language_fonts: HashMap<String, String>,
}

impl Default for FontPreferences {
    fn default() -> Self {
        Self {
            default_serif: "Times New Roman".to_string(),
            default_sans_serif: "Arial".to_string(),
            default_monospace: "Courier New".to_string(),
            default_cursive: "Comic Sans MS".to_string(),
            default_fantasy: "Impact".to_string(),
            default_size: 16.0,
            minimum_size: 6.0,
            allow_custom_fonts: true,
            allow_web_fonts: true,
            font_smoothing: FontSmoothing::Auto,
            language_fonts: HashMap::new(),
        }
    }
}

/// Font smoothing preference
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontSmoothing {
    /// System default
    #[default]
    Auto,
    /// No smoothing
    None,
    /// Grayscale antialiasing
    Grayscale,
    /// Subpixel antialiasing
    Subpixel,
}

/// Browser shell font preference handler
pub struct BrowserFontPreferences {
    /// Current preferences
    preferences: FontPreferences,
    /// Observers for preference changes
    observers: Vec<Box<dyn Fn(&FontPreferences) + Send + Sync>>,
}

impl std::fmt::Debug for BrowserFontPreferences {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserFontPreferences")
            .field("preferences", &self.preferences)
            .field("observers", &format!("[{} observers]", self.observers.len()))
            .finish()
    }
}

impl BrowserFontPreferences {
    /// Create with default preferences
    pub fn new() -> Self {
        Self {
            preferences: FontPreferences::default(),
            observers: Vec::new(),
        }
    }

    /// Get current preferences
    pub fn get(&self) -> &FontPreferences {
        &self.preferences
    }

    /// Update preferences
    pub fn update(&mut self, preferences: FontPreferences) {
        self.preferences = preferences;
        self.notify_observers();
    }

    /// Set default serif font
    pub fn set_default_serif(&mut self, family: String) {
        self.preferences.default_serif = family;
        self.notify_observers();
    }

    /// Set default sans-serif font
    pub fn set_default_sans_serif(&mut self, family: String) {
        self.preferences.default_sans_serif = family;
        self.notify_observers();
    }

    /// Set default monospace font
    pub fn set_default_monospace(&mut self, family: String) {
        self.preferences.default_monospace = family;
        self.notify_observers();
    }

    /// Set default font size
    pub fn set_default_size(&mut self, size: f32) {
        self.preferences.default_size = size.max(self.preferences.minimum_size);
        self.notify_observers();
    }

    /// Set minimum font size
    pub fn set_minimum_size(&mut self, size: f32) {
        self.preferences.minimum_size = size.max(1.0);
        if self.preferences.default_size < self.preferences.minimum_size {
            self.preferences.default_size = self.preferences.minimum_size;
        }
        self.notify_observers();
    }

    /// Set web fonts allowed
    pub fn set_allow_web_fonts(&mut self, allow: bool) {
        self.preferences.allow_web_fonts = allow;
        self.notify_observers();
    }

    /// Set font smoothing
    pub fn set_font_smoothing(&mut self, smoothing: FontSmoothing) {
        self.preferences.font_smoothing = smoothing;
        self.notify_observers();
    }

    /// Set language-specific font
    pub fn set_language_font(&mut self, language: String, family: String) {
        self.preferences.language_fonts.insert(language, family);
        self.notify_observers();
    }

    /// Get font for a specific language
    pub fn get_language_font(&self, language: &str) -> Option<&String> {
        self.preferences.language_fonts.get(language)
    }

    /// Resolve generic font family to actual font
    ///
    /// Returns the resolved font family name. If the generic family is recognized
    /// (serif, sans-serif, monospace, cursive, fantasy), returns the configured
    /// default. Otherwise returns the input unchanged.
    pub fn resolve_generic(&self, generic: &str) -> String {
        match generic.to_lowercase().as_str() {
            "serif" => self.preferences.default_serif.clone(),
            "sans-serif" => self.preferences.default_sans_serif.clone(),
            "monospace" => self.preferences.default_monospace.clone(),
            "cursive" => self.preferences.default_cursive.clone(),
            "fantasy" => self.preferences.default_fantasy.clone(),
            _ => generic.to_string(),
        }
    }

    /// Apply minimum font size constraint
    pub fn apply_minimum_size(&self, size: f32) -> f32 {
        size.max(self.preferences.minimum_size)
    }

    /// Check if web fonts are allowed
    pub fn web_fonts_allowed(&self) -> bool {
        self.preferences.allow_web_fonts
    }

    fn notify_observers(&self) {
        for observer in &self.observers {
            observer(&self.preferences);
        }
    }
}

impl Default for BrowserFontPreferences {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_font_request_default() {
        let request = CssFontRequest::default();
        assert_eq!(request.family_stack, vec!["sans-serif"]);
        assert_eq!(request.size, 16.0);
    }

    #[test]
    fn test_css_font_weight_to_numeric() {
        assert_eq!(CssFontWeight::Normal.to_numeric(), 400);
        assert_eq!(CssFontWeight::Bold.to_numeric(), 700);
        assert_eq!(CssFontWeight::Custom(550).to_numeric(), 550);
    }

    #[test]
    fn test_css_font_resolver_cache_key() {
        let request = CssFontRequest::default();
        let key = CssFontResolver::cache_key(&request);
        assert!(key.contains("sans-serif"));
    }

    #[test]
    fn test_render_text_empty() {
        let text = RenderText::empty();
        assert_eq!(text.glyph_count(), 0);
        assert_eq!(text.width, 0.0);
    }

    #[test]
    fn test_web_font_downloader_new() {
        let downloader = WebFontDownloader::new();
        assert_eq!(downloader.active_count(), 0);
    }

    #[test]
    fn test_cors_validator_same_origin() {
        assert!(CorsValidator::is_same_origin(
            "https://example.com/font.woff2",
            "https://example.com/page.html"
        ));
        assert!(!CorsValidator::is_same_origin(
            "https://example.com/font.woff2",
            "https://other.com/page.html"
        ));
    }

    #[test]
    fn test_cors_validator_allowed() {
        let validator = CorsValidator::new();
        let result = validator.validate("https://example.com", Some("*"), false);
        assert_eq!(result, CorsResult::Allowed);
    }

    #[test]
    fn test_cors_validator_blocked() {
        let validator = CorsValidator::new();
        let result = validator.validate("https://example.com", None, false);
        assert!(matches!(result, CorsResult::Blocked(CorsError::MissingAllowOrigin)));
    }

    #[test]
    fn test_font_preferences_default() {
        let prefs = FontPreferences::default();
        assert_eq!(prefs.default_size, 16.0);
        assert!(prefs.allow_web_fonts);
    }

    #[test]
    fn test_browser_font_preferences_resolve_generic() {
        let prefs = BrowserFontPreferences::new();
        assert_eq!(prefs.resolve_generic("serif"), "Times New Roman");
        assert_eq!(prefs.resolve_generic("monospace"), "Courier New");
    }

    #[test]
    fn test_browser_font_preferences_minimum_size() {
        let mut prefs = BrowserFontPreferences::new();
        prefs.set_minimum_size(10.0);
        assert_eq!(prefs.apply_minimum_size(8.0), 10.0);
        assert_eq!(prefs.apply_minimum_size(12.0), 12.0);
    }

    #[test]
    fn test_extract_origin() {
        assert_eq!(extract_origin("https://example.com/path"), "https://example.com");
        assert_eq!(extract_origin("http://localhost:8080/font.woff"), "http://localhost:8080");
    }
}
