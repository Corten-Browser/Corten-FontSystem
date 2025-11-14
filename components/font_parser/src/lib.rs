//! font_parser - Parse OpenType, TrueType, WOFF, and WOFF2 font files

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
pub mod types;
mod variable_fonts;
mod woff;
mod woff2;

// Public exports
pub use error::ParseError;
pub use types::{
    BoundingBox, CMapTable, Contour, FontMetrics, GlyphId, GlyphOutline, OpenTypeFont, Tag,
    TagParseError,
};
pub use variable_fonts::{
    AvarTable, AxisSegmentMap, FvarTable, NamedInstance, VariationAxis, VariationCoordinates,
};
pub use woff::WoffFont;
pub use woff2::Woff2Font;
