//! font_system_api - Public API and orchestration layer for the complete font system
//!
//! This crate provides the main public API for the Corten Font System, orchestrating
//! all font-related operations including loading, matching, shaping, and rendering.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod profiling;
mod system;
pub mod types;

// Re-export public types
pub use profiling::{
    ComponentMemoryBreakdown, ComponentMemoryPercentages, MemoryProfiler, MemoryStats,
};
pub use system::{FontSystem, ShapedText};
pub use types::{CacheConfig, FontError, FontSystemConfig};

// Re-export types from dependencies
pub use font_registry::types::{FontDescriptor, FontId, FontMetrics};
pub use font_types::types::GlyphId;
pub use glyph_renderer::types::{GlyphBitmap, GlyphOutline, RenderMode};
pub use text_shaper::types::ShapingOptions;
