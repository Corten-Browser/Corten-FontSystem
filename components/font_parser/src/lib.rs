//! font_parser - Parse OpenType, TrueType, WOFF, and WOFF2 font files

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
pub mod types;

// Public exports
pub use error::ParseError;
pub use types::{
    BoundingBox, CMapTable, Contour, FontMetrics, GlyphId, GlyphOutline, OpenTypeFont, Tag,
    TagParseError,
};
