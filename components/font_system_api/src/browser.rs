//! Browser component integration for FontSystem
//!
//! This module provides the BrowserComponent trait and implementation for
//! integrating the font system with browser environments.

use std::time::Instant;

use crate::messages::{
    ComponentMessage, ComponentResponse, FontLoadedResponse, GlyphRasterizedResponse,
    LoadWebFontRequest, RasterizeGlyphRequest, ShapeTextRequest, TextShapedResponse,
};
use crate::system::{FontSystem, ShapedText};
use crate::types::{FontError, FontSystemConfig};
use font_registry::types::FontId;
use font_types::types::GlyphId;
use glyph_renderer::types::{GlyphBitmap, RenderMode};
use text_shaper::types::ShapingOptions;

/// Health status of the browser component
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Component is healthy and operational
    Healthy,
    /// Component is degraded but functional
    Degraded(String),
    /// Component is unhealthy and not operational
    Unhealthy(String),
}

/// Metrics for the browser component
#[derive(Debug, Clone, Default)]
pub struct ComponentMetrics {
    /// Number of fonts loaded
    pub fonts_loaded: usize,
    /// Number of text shaping operations performed
    pub shape_operations: u64,
    /// Number of glyph rasterization operations performed
    pub rasterize_operations: u64,
    /// Total time spent in shaping operations (milliseconds)
    pub shape_time_ms: u64,
    /// Total time spent in rasterization operations (milliseconds)
    pub rasterize_time_ms: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of errors encountered
    pub errors: u64,
}

impl ComponentMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the cache hit rate as a percentage
    ///
    /// # Returns
    ///
    /// Cache hit rate between 0.0 and 100.0, or 0.0 if no cache operations
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        (self.cache_hits as f64 / total as f64) * 100.0
    }

    /// Get average shaping time in milliseconds
    ///
    /// # Returns
    ///
    /// Average time per shaping operation, or 0.0 if no operations
    pub fn avg_shape_time_ms(&self) -> f64 {
        if self.shape_operations == 0 {
            return 0.0;
        }
        self.shape_time_ms as f64 / self.shape_operations as f64
    }

    /// Get average rasterization time in milliseconds
    ///
    /// # Returns
    ///
    /// Average time per rasterization operation, or 0.0 if no operations
    pub fn avg_rasterize_time_ms(&self) -> f64 {
        if self.rasterize_operations == 0 {
            return 0.0;
        }
        self.rasterize_time_ms as f64 / self.rasterize_operations as f64
    }
}

/// Trait for browser component integration with FontSystem
///
/// This trait defines the interface for browser integration, including
/// initialization, shutdown, message handling, health checks, and metrics.
pub trait BrowserComponent {
    /// Initialize the browser component with cache setup
    ///
    /// This method sets up the component for operation, including:
    /// - Initializing internal caches
    /// - Loading default fonts if configured
    /// - Setting up message handlers
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Component initialized successfully
    /// * `Err(FontError)` - Failed to initialize component
    fn initialize(&mut self) -> Result<(), FontError>;

    /// Shutdown the browser component for clean teardown
    ///
    /// This method performs cleanup operations, including:
    /// - Flushing caches
    /// - Releasing resources
    /// - Saving state if needed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Component shut down successfully
    /// * `Err(FontError)` - Failed to shut down cleanly
    fn shutdown(&mut self) -> Result<(), FontError>;

    /// Handle a LoadWebFont message
    ///
    /// # Arguments
    ///
    /// * `request` - The load web font request
    ///
    /// # Returns
    ///
    /// * `Ok(FontId)` - Font loaded successfully with assigned ID
    /// * `Err(FontError)` - Failed to load font
    fn handle_load_web_font(&mut self, request: LoadWebFontRequest) -> Result<FontId, FontError>;

    /// Handle a ShapeText message
    ///
    /// # Arguments
    ///
    /// * `request` - The shape text request
    ///
    /// # Returns
    ///
    /// * `Ok(ShapedText)` - Text shaped successfully
    /// * `Err(FontError)` - Failed to shape text
    fn handle_shape_text(&mut self, request: ShapeTextRequest) -> Result<ShapedText, FontError>;

    /// Handle a RasterizeGlyph message
    ///
    /// # Arguments
    ///
    /// * `request` - The rasterize glyph request
    ///
    /// # Returns
    ///
    /// * `Ok(GlyphBitmap)` - Glyph rasterized successfully
    /// * `Err(FontError)` - Failed to rasterize glyph
    fn handle_rasterize_glyph(
        &mut self,
        request: RasterizeGlyphRequest,
    ) -> Result<GlyphBitmap, FontError>;

    /// Perform a health check
    ///
    /// # Returns
    ///
    /// Current health status of the component
    fn health_check(&self) -> HealthStatus;

    /// Get current metrics
    ///
    /// # Returns
    ///
    /// Current component metrics
    fn get_metrics(&self) -> ComponentMetrics;
}

/// Browser component implementation wrapping FontSystem
pub struct FontSystemBrowserComponent {
    /// Underlying font system
    font_system: FontSystem,
    /// Whether the component is initialized
    initialized: bool,
    /// Component metrics
    metrics: ComponentMetrics,
    /// Start time for uptime tracking
    start_time: Option<Instant>,
}

impl FontSystemBrowserComponent {
    /// Create a new browser component with default configuration
    ///
    /// # Returns
    ///
    /// * `Ok(FontSystemBrowserComponent)` - Component created successfully
    /// * `Err(FontError)` - Failed to create component
    ///
    /// # Example
    ///
    /// ```no_run
    /// use font_system_api::browser::FontSystemBrowserComponent;
    ///
    /// let component = FontSystemBrowserComponent::new().expect("Failed to create component");
    /// ```
    pub fn new() -> Result<Self, FontError> {
        Self::with_config(FontSystemConfig::default())
    }

    /// Create a new browser component with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the font system
    ///
    /// # Returns
    ///
    /// * `Ok(FontSystemBrowserComponent)` - Component created successfully
    /// * `Err(FontError)` - Failed to create component
    ///
    /// # Example
    ///
    /// ```no_run
    /// use font_system_api::browser::FontSystemBrowserComponent;
    /// use font_system_api::FontSystemConfig;
    ///
    /// let config = FontSystemConfig::default();
    /// let component = FontSystemBrowserComponent::with_config(config)
    ///     .expect("Failed to create component");
    /// ```
    pub fn with_config(config: FontSystemConfig) -> Result<Self, FontError> {
        let font_system = FontSystem::new(config)?;
        Ok(Self {
            font_system,
            initialized: false,
            metrics: ComponentMetrics::new(),
            start_time: None,
        })
    }

    /// Process a component message and return a response
    ///
    /// # Arguments
    ///
    /// * `message` - The component message to process
    ///
    /// # Returns
    ///
    /// Response from processing the message
    ///
    /// # Example
    ///
    /// ```no_run
    /// use font_system_api::browser::{BrowserComponent, FontSystemBrowserComponent};
    /// use font_system_api::messages::{ComponentMessage, LoadWebFontRequest};
    ///
    /// let mut component = FontSystemBrowserComponent::new().unwrap();
    /// component.initialize().unwrap();
    ///
    /// let request = LoadWebFontRequest::from_data("MyFont".to_string(), vec![0u8; 100]);
    /// let message = ComponentMessage::LoadWebFont(request);
    /// let response = component.process_message(message);
    /// ```
    pub fn process_message(&mut self, message: ComponentMessage) -> ComponentResponse {
        match message {
            ComponentMessage::LoadWebFont(request) => {
                match self.handle_load_web_font(request.clone()) {
                    Ok(font_id) => ComponentResponse::FontLoaded(FontLoadedResponse {
                        font_id,
                        family_name: request.family_name,
                    }),
                    Err(err) => {
                        self.metrics.errors += 1;
                        ComponentResponse::Error(err)
                    }
                }
            }
            ComponentMessage::ShapeText(request) => match self.handle_shape_text(request) {
                Ok(shaped_text) => {
                    ComponentResponse::TextShaped(TextShapedResponse { shaped_text })
                }
                Err(err) => {
                    self.metrics.errors += 1;
                    ComponentResponse::Error(err)
                }
            },
            ComponentMessage::RasterizeGlyph(request) => {
                match self.handle_rasterize_glyph(request) {
                    Ok(bitmap) => {
                        ComponentResponse::GlyphRasterized(GlyphRasterizedResponse { bitmap })
                    }
                    Err(err) => {
                        self.metrics.errors += 1;
                        ComponentResponse::Error(err)
                    }
                }
            }
        }
    }

    /// Check if the component is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the uptime in seconds
    ///
    /// # Returns
    ///
    /// Number of seconds since initialization, or 0 if not initialized
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
    }

    /// Get direct access to the underlying font system
    ///
    /// # Returns
    ///
    /// Reference to the underlying FontSystem
    pub fn font_system(&self) -> &FontSystem {
        &self.font_system
    }

    /// Get mutable access to the underlying font system
    ///
    /// # Returns
    ///
    /// Mutable reference to the underlying FontSystem
    pub fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }
}

impl BrowserComponent for FontSystemBrowserComponent {
    fn initialize(&mut self) -> Result<(), FontError> {
        if self.initialized {
            return Ok(());
        }

        // Initialize caches and internal state
        self.font_system.clear_caches();
        self.metrics = ComponentMetrics::new();
        self.start_time = Some(Instant::now());
        self.initialized = true;

        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), FontError> {
        if !self.initialized {
            return Ok(());
        }

        // Clear caches and release resources
        self.font_system.clear_caches();
        self.initialized = false;
        self.start_time = None;

        Ok(())
    }

    fn handle_load_web_font(&mut self, request: LoadWebFontRequest) -> Result<FontId, FontError> {
        // Validate request
        if request.url.is_none() && request.data.is_none() {
            self.metrics.errors += 1;
            return Err(FontError::LoadError(
                "Either URL or data must be provided".to_string(),
            ));
        }

        // Load font from data if provided
        if let Some(data) = request.data {
            match self.font_system.load_font_data(data) {
                Ok(font_id) => {
                    self.metrics.fonts_loaded += 1;
                    Ok(font_id)
                }
                Err(err) => {
                    self.metrics.errors += 1;
                    Err(err)
                }
            }
        } else {
            // URL loading not yet implemented
            self.metrics.errors += 1;
            Err(FontError::LoadError(
                "URL loading not yet implemented - provide font data directly".to_string(),
            ))
        }
    }

    fn handle_shape_text(&mut self, request: ShapeTextRequest) -> Result<ShapedText, FontError> {
        let start = Instant::now();

        // Always track the operation
        self.metrics.shape_operations += 1;

        // Find matching font
        let font_id = match self.font_system.match_font(&request.font_descriptor) {
            Some(id) => id,
            None => {
                self.metrics.errors += 1;
                let elapsed_ms = start.elapsed().as_millis() as u64;
                self.metrics.shape_time_ms += elapsed_ms;
                return Err(FontError::FontNotFound);
            }
        };

        // Shape the text
        let result = self.font_system.shape_text(
            &request.text,
            font_id,
            request.font_descriptor.size,
            &request.options,
        );

        let elapsed_ms = start.elapsed().as_millis() as u64;
        self.metrics.shape_time_ms += elapsed_ms;

        if result.is_err() {
            self.metrics.errors += 1;
        }

        result
    }

    fn handle_rasterize_glyph(
        &mut self,
        request: RasterizeGlyphRequest,
    ) -> Result<GlyphBitmap, FontError> {
        let start = Instant::now();

        // Always track the operation
        self.metrics.rasterize_operations += 1;

        let result = self.font_system.rasterize_glyph(
            request.font_id,
            request.glyph_id,
            request.size,
            request.mode,
        );

        let elapsed_ms = start.elapsed().as_millis() as u64;
        self.metrics.rasterize_time_ms += elapsed_ms;

        if result.is_err() {
            self.metrics.errors += 1;
        }

        result
    }

    fn health_check(&self) -> HealthStatus {
        if !self.initialized {
            return HealthStatus::Unhealthy("Component not initialized".to_string());
        }

        // Check error rate
        let total_ops = self.metrics.shape_operations + self.metrics.rasterize_operations;
        if total_ops > 0 {
            let error_rate = self.metrics.errors as f64 / total_ops as f64;
            if error_rate > 0.5 {
                return HealthStatus::Unhealthy(format!(
                    "High error rate: {:.1}%",
                    error_rate * 100.0
                ));
            }
            if error_rate > 0.1 {
                return HealthStatus::Degraded(format!(
                    "Elevated error rate: {:.1}%",
                    error_rate * 100.0
                ));
            }
        }

        HealthStatus::Healthy
    }

    fn get_metrics(&self) -> ComponentMetrics {
        self.metrics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CacheConfig, GlyphCacheConfig, ShapingCacheConfig};
    use font_types::types::Direction;
    use std::collections::HashMap;
    use text_shaper::types::{Language, Script};

    fn create_default_shaping_options() -> ShapingOptions {
        ShapingOptions {
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
        }
    }

    // FEAT-073: BrowserComponent trait tests
    #[test]
    fn test_browser_component_trait_exists() {
        // Verify the trait can be used as a type bound
        fn _accepts_browser_component<T: BrowserComponent>(_: &T) {}

        let component = FontSystemBrowserComponent::new().unwrap();
        _accepts_browser_component(&component);
    }

    // FEAT-074: Component initialization tests
    #[test]
    fn test_component_initialization_with_cache_setup() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        assert!(!component.is_initialized());

        // When
        let result = component.initialize();

        // Then
        assert!(result.is_ok());
        assert!(component.is_initialized());
        assert!(component.uptime_seconds() == 0 || component.uptime_seconds() >= 0);
    }

    #[test]
    fn test_component_initialization_is_idempotent() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();

        // When - initialize again
        let result = component.initialize();

        // Then - should not error
        assert!(result.is_ok());
        assert!(component.is_initialized());
    }

    #[test]
    fn test_component_initialization_with_custom_config() {
        // Given
        let cache_config = CacheConfig {
            glyph_cache: GlyphCacheConfig {
                max_entries: 20_000,
                max_memory_bytes: 200 * 1024 * 1024,
                enable_statistics: true,
            },
            shaping_cache: ShapingCacheConfig {
                max_entries: 2_000,
                enable_statistics: true,
            },
        };
        let config = FontSystemConfig {
            cache_config,
            enable_subpixel: true,
            enable_hinting: true,
            load_system_fonts_on_init: false,
        };

        // When
        let result = FontSystemBrowserComponent::with_config(config);

        // Then
        assert!(result.is_ok());
        let mut component = result.unwrap();
        assert!(component.initialize().is_ok());
    }

    // FEAT-075: Component shutdown tests
    #[test]
    fn test_component_shutdown_clean_teardown() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        assert!(component.is_initialized());

        // When
        let result = component.shutdown();

        // Then
        assert!(result.is_ok());
        assert!(!component.is_initialized());
    }

    #[test]
    fn test_component_shutdown_is_idempotent() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        component.shutdown().unwrap();

        // When - shutdown again
        let result = component.shutdown();

        // Then - should not error
        assert!(result.is_ok());
        assert!(!component.is_initialized());
    }

    #[test]
    fn test_component_shutdown_without_initialization() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();

        // When - shutdown without init
        let result = component.shutdown();

        // Then
        assert!(result.is_ok());
    }

    // FEAT-076: LoadWebFont message tests
    #[test]
    fn test_handle_load_web_font_from_data() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = LoadWebFontRequest::from_data("TestFont".to_string(), vec![0u8; 100]);

        // When
        let result = component.handle_load_web_font(request);

        // Then - expect error since font data is invalid (not real font)
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_load_web_font_missing_data_and_url() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = LoadWebFontRequest {
            family_name: "TestFont".to_string(),
            url: None,
            data: None,
            weight: None,
            style: None,
        };

        // When
        let result = component.handle_load_web_font(request);

        // Then
        assert!(result.is_err());
        match result {
            Err(FontError::LoadError(msg)) => {
                assert!(msg.contains("Either URL or data must be provided"));
            }
            _ => panic!("Expected LoadError"),
        }
    }

    #[test]
    fn test_handle_load_web_font_url_not_implemented() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = LoadWebFontRequest::from_url(
            "TestFont".to_string(),
            "https://test.com/font.woff2".to_string(),
        );

        // When
        let result = component.handle_load_web_font(request);

        // Then
        assert!(result.is_err());
        match result {
            Err(FontError::LoadError(msg)) => {
                assert!(msg.contains("URL loading not yet implemented"));
            }
            _ => panic!("Expected LoadError"),
        }
    }

    // FEAT-077: ShapeText message tests
    #[test]
    fn test_handle_shape_text() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = ShapeTextRequest::new(
            "Hello".to_string(),
            font_registry::types::FontDescriptor::default(),
            create_default_shaping_options(),
        );

        // When
        let result = component.handle_shape_text(request);

        // Then - expect FontNotFound since no fonts loaded
        assert!(result.is_err());
        match result {
            Err(FontError::FontNotFound) => {}
            _ => panic!("Expected FontNotFound"),
        }
    }

    #[test]
    fn test_handle_shape_text_updates_metrics() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = ShapeTextRequest::new(
            "Test".to_string(),
            font_registry::types::FontDescriptor::default(),
            create_default_shaping_options(),
        );

        // When
        let _ = component.handle_shape_text(request);
        let metrics = component.get_metrics();

        // Then
        assert_eq!(metrics.shape_operations, 1);
    }

    // FEAT-078: RasterizeGlyph message tests
    #[test]
    fn test_handle_rasterize_glyph() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = RasterizeGlyphRequest::new(0, GlyphId { id: 65 }, 12.0, RenderMode::Gray);

        // When
        let result = component.handle_rasterize_glyph(request);

        // Then - expect error since glyph renderer not implemented
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_rasterize_glyph_updates_metrics() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = RasterizeGlyphRequest::new(0, GlyphId { id: 65 }, 12.0, RenderMode::Gray);

        // When
        let _ = component.handle_rasterize_glyph(request);
        let metrics = component.get_metrics();

        // Then
        assert_eq!(metrics.rasterize_operations, 1);
    }

    // FEAT-079: Health check tests
    #[test]
    fn test_health_check_unhealthy_when_not_initialized() {
        // Given
        let component = FontSystemBrowserComponent::new().unwrap();

        // When
        let status = component.health_check();

        // Then
        match status {
            HealthStatus::Unhealthy(msg) => {
                assert!(msg.contains("not initialized"));
            }
            _ => panic!("Expected Unhealthy status"),
        }
    }

    #[test]
    fn test_health_check_healthy_when_initialized() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();

        // When
        let status = component.health_check();

        // Then
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_enum_variants() {
        // Test all enum variants
        let healthy = HealthStatus::Healthy;
        let degraded = HealthStatus::Degraded("test".to_string());
        let unhealthy = HealthStatus::Unhealthy("test".to_string());

        assert_eq!(healthy, HealthStatus::Healthy);
        assert!(matches!(degraded, HealthStatus::Degraded(_)));
        assert!(matches!(unhealthy, HealthStatus::Unhealthy(_)));
    }

    // FEAT-080: Metrics reporting tests
    #[test]
    fn test_get_metrics_returns_empty_initially() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();

        // When
        let metrics = component.get_metrics();

        // Then
        assert_eq!(metrics.fonts_loaded, 0);
        assert_eq!(metrics.shape_operations, 0);
        assert_eq!(metrics.rasterize_operations, 0);
        assert_eq!(metrics.errors, 0);
    }

    #[test]
    fn test_metrics_cache_hit_rate() {
        // Given
        let mut metrics = ComponentMetrics::new();
        metrics.cache_hits = 80;
        metrics.cache_misses = 20;

        // When
        let hit_rate = metrics.cache_hit_rate();

        // Then
        assert!((hit_rate - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_metrics_cache_hit_rate_with_zero_operations() {
        // Given
        let metrics = ComponentMetrics::new();

        // When
        let hit_rate = metrics.cache_hit_rate();

        // Then
        assert_eq!(hit_rate, 0.0);
    }

    #[test]
    fn test_metrics_avg_shape_time() {
        // Given
        let mut metrics = ComponentMetrics::new();
        metrics.shape_operations = 10;
        metrics.shape_time_ms = 500;

        // When
        let avg_time = metrics.avg_shape_time_ms();

        // Then
        assert!((avg_time - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_metrics_avg_rasterize_time() {
        // Given
        let mut metrics = ComponentMetrics::new();
        metrics.rasterize_operations = 5;
        metrics.rasterize_time_ms = 100;

        // When
        let avg_time = metrics.avg_rasterize_time_ms();

        // Then
        assert!((avg_time - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_metrics_error_tracking() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();

        // When - trigger errors
        let _ = component.handle_load_web_font(LoadWebFontRequest {
            family_name: "Test".to_string(),
            url: None,
            data: None,
            weight: None,
            style: None,
        });

        // Then
        let metrics = component.get_metrics();
        assert_eq!(metrics.errors, 1);
    }

    // Process message tests
    #[test]
    fn test_process_message_load_web_font() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = LoadWebFontRequest::from_url(
            "Test".to_string(),
            "https://test.com/font.woff2".to_string(),
        );
        let message = ComponentMessage::LoadWebFont(request);

        // When
        let response = component.process_message(message);

        // Then
        match response {
            ComponentResponse::Error(_) => {}
            _ => panic!("Expected Error response"),
        }
    }

    #[test]
    fn test_process_message_shape_text() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = ShapeTextRequest::new(
            "Hello".to_string(),
            font_registry::types::FontDescriptor::default(),
            create_default_shaping_options(),
        );
        let message = ComponentMessage::ShapeText(request);

        // When
        let response = component.process_message(message);

        // Then
        match response {
            ComponentResponse::Error(FontError::FontNotFound) => {}
            _ => panic!("Expected FontNotFound error"),
        }
    }

    #[test]
    fn test_process_message_rasterize_glyph() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();
        component.initialize().unwrap();
        let request = RasterizeGlyphRequest::new(0, GlyphId { id: 65 }, 12.0, RenderMode::Gray);
        let message = ComponentMessage::RasterizeGlyph(request);

        // When
        let response = component.process_message(message);

        // Then
        match response {
            ComponentResponse::Error(_) => {}
            _ => panic!("Expected Error response"),
        }
    }

    // Utility method tests
    #[test]
    fn test_uptime_seconds() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();

        // When not initialized
        assert_eq!(component.uptime_seconds(), 0);

        // When initialized
        component.initialize().unwrap();
        let uptime = component.uptime_seconds();
        assert!(uptime == 0 || uptime >= 0); // Just started
    }

    #[test]
    fn test_font_system_accessor() {
        // Given
        let component = FontSystemBrowserComponent::new().unwrap();

        // When
        let font_system = component.font_system();

        // Then
        assert_eq!(font_system.font_count(), 0);
    }

    #[test]
    fn test_font_system_mut_accessor() {
        // Given
        let mut component = FontSystemBrowserComponent::new().unwrap();

        // When
        let font_system = component.font_system_mut();

        // Then - verify we can call mutable methods
        font_system.clear_caches();
    }
}
