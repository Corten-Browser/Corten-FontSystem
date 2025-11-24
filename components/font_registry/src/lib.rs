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

pub mod coverage;
pub mod matching;
pub mod registry;
pub mod types;
pub mod web_fonts;

// Re-export main types for convenience
pub use registry::FontRegistry;
pub use types::{
    FontDescriptor, FontFace, FontId, FontMetrics, FontStretch, FontStyle, FontWeight,
    RegistryError,
};

// Re-export coverage types
pub use coverage::{CoverageFallbackManager, CoverageStats, FontCoverage};

// Re-export matching types
pub use matching::{FontMatch, FontMatcher, FontSubstitution, SubstitutionConfig};

// Re-export web font types
pub use web_fonts::{
    FontDisplay, FontFaceDeclaration, FontLoadState, FontSource, FontWeightRange, UnicodeRange,
    WebFont, WebFontManager,
};
