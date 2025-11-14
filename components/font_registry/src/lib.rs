//! font_registry - Font discovery, loading, caching, and font matching algorithms
//!
//! This component provides a font registry system for loading fonts from various sources
//! (files, memory, system fonts) and matching fonts based on descriptors (family, weight, style, stretch).
//!
//! # Examples
//!
//! ```
//! use font_registry::{FontRegistry, FontDescriptor};
//!
//! // Create a new registry
//! let mut registry = FontRegistry::new();
//!
//! // Match a font (returns None if no fonts loaded)
//! let descriptor = FontDescriptor::default();
//! let result = registry.match_font(&descriptor);
//! assert_eq!(result, None);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod registry;
pub mod types;

// Re-export main types for convenience
pub use registry::FontRegistry;
pub use types::{
    FontDescriptor, FontFace, FontId, FontMetrics, FontStretch, FontStyle, FontWeight,
    RegistryError,
};
