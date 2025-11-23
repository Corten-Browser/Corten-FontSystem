//! font_system_api - Public API and orchestration layer for the complete font system
//!
//! This crate provides the main public API for the Corten Font System, orchestrating
//! all font-related operations including loading, matching, shaping, and rendering.
//!
//! # Browser Integration
//!
//! The [`browser`] module provides browser component integration through the
//! [`BrowserComponent`](browser::BrowserComponent) trait. This allows the font system
//! to be integrated with browser environments via message passing.
//!
//! # Example
//!
//! ```no_run
//! use font_system_api::{FontSystem, FontSystemConfig};
//! use font_system_api::browser::{BrowserComponent, FontSystemBrowserComponent};
//!
//! // Create and initialize a browser component
//! let mut component = FontSystemBrowserComponent::new().unwrap();
//! component.initialize().unwrap();
//!
//! // Check health status
//! let status = component.health_check();
//!
//! // Get metrics
//! let metrics = component.get_metrics();
//!
//! // Cleanup
//! component.shutdown().unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod browser;
pub mod browser_integration;
pub mod messages;
pub mod profiling;
mod system;
pub mod types;

// Re-export public types
pub use profiling::{
    ComponentMemoryBreakdown, ComponentMemoryPercentages, MemoryProfiler, MemoryStats,
};
pub use system::{FontSystem, ShapedText};
pub use types::{CacheConfig, FontError, FontSystemConfig};

// Re-export browser types for convenience
pub use browser::{BrowserComponent, ComponentMetrics, FontSystemBrowserComponent, HealthStatus};
pub use messages::{
    ComponentMessage, ComponentResponse, LoadWebFontRequest, RasterizeGlyphRequest,
    ShapeTextRequest,
};

// Re-export browser integration types (CSS, rendering, network, CORS, preferences)
pub use browser_integration::{
    BrowserFontPreferences, CorsError, CorsResult, CorsValidator, CssFontRequest,
    CssFontResolver, CssFontResponse, CssFontStretch, CssFontStyle, CssFontWeight,
    FontPreferences, FontSmoothing, RenderText, RenderingEngineInterface, RenderingStats,
    ShapedGlyph, ShapedRun, WebFontDownloadState, WebFontDownloader, WebFontRequest,
    WebFontResponse,
};

// Re-export types from dependencies
pub use font_registry::types::{FontDescriptor, FontId, FontMetrics};
pub use font_types::types::GlyphId;
pub use glyph_renderer::types::{GlyphBitmap, GlyphOutline, RenderMode};
pub use text_layout::{
    JustificationMode, LayoutLine, LayoutOptions, LayoutResult, ParagraphLayout, TextDirection,
};
pub use text_shaper::types::ShapingOptions;
