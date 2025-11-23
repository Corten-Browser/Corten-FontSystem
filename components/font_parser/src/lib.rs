//! font_parser - Parse OpenType, TrueType, WOFF, and WOFF2 font files
//!
//! This crate provides comprehensive parsing for font files including:
//! - OpenType and TrueType fonts
//! - WOFF and WOFF2 compressed formats
//! - Variable fonts (fvar, avar tables)
//! - Color fonts (COLR/CPAL, CBDT/CBLC, SVG)
//! - CFF/CFF2 PostScript outlines
//! - TrueType hinting (cvt, fpgm, prep tables)
//! - Embedded bitmap fonts (EBDT/EBLC)
//! - OpenType features (GSUB, GPOS)
//! - Security validation and limits

#![warn(missing_docs)]
#![warn(clippy::all)]

// Internal modules
mod bitmap;
mod cff;
mod color_fonts;
mod error;
pub mod features;
pub mod gpos;
pub mod gsub;
mod hinting;
pub mod limits;
pub mod security;
mod svg;
pub mod types;
pub mod validation;
mod variable_fonts;
mod woff;
mod woff2;

// Public exports
pub use color_fonts::{
    BaseGlyph, CbdtTable, Color, ColorFormat, ColrTable, CpalTable, Layer, SvgTable,
};
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

// GSUB exports
pub use gsub::{
    AlternateSubst, ChainContextRule, ChainingContextSubst, ContextRule, ContextSubst, Coverage,
    FeatureList, FeatureRecord, GsubTable, LangSys, Ligature, LigatureSubst, Lookup as GsubLookup,
    LookupFlags, LookupList as GsubLookupList, LookupType as GsubLookupType, MultipleSubst,
    ScriptList, ScriptRecord, SingleSubst, SubstLookupRecord, SubtableData as GsubSubtableData,
};

// GPOS exports
pub use gpos::{
    AnchorPoint, GposTable, Lookup as GposLookup, LookupList as GposLookupList,
    LookupType as GposLookupType, MarkRecord, MarkToBaseSubtable, PairAdjustmentSubtable,
    PairValueRecord, SingleAdjustmentSubtable, SubtableData as GposSubtableData, ValueRecord,
};

// Feature selection exports
pub use features::{
    default_features, kerning_and_ligatures, kerning_only, tags, FeatureApplicator, FeatureQuery,
    FeatureSelection,
};

// CFF/CFF2 exports (PostScript outlines)
pub use cff::{
    Cff2FontDict, Cff2Table, Cff2TopDict, CffTable, CharStringCommand, ItemVariationData,
    ItemVariationStore, PrivateDict, TopDict,
};

// TrueType hinting exports
pub use hinting::{
    decode_instructions, CvtTable, FpgmTable, GraphicsState, HintInstruction, HintingState,
    MaxpHintingInfo, PrepTable, RoundState,
};

// SVG-in-OpenType exports
pub use svg::{
    extract_glyph_svg, parse_viewbox, validate_svg, SvgDocumentRecord, SvgDocumentTable,
    SvgElement, SvgGlyphInfo, SvgParseOptions,
};

// Bitmap fonts exports (EBDT/EBLC)
pub use bitmap::{
    BigGlyphMetrics, BitmapGlyph, BitmapScaleRecord, BitmapSizeRecord, EbdtTable, EblcTable,
    EbscTable, GlyphIdOffsetPair, IndexSubTable, IndexSubTableHeader, SbitLineMetrics,
    SmallGlyphMetrics,
};

// Security module exports
pub use limits::{
    LimitExceeded, SecurityLimits, MAX_FONT_SIZE, MAX_GLYPH_COUNT, MAX_RECURSION_DEPTH,
    MAX_TABLE_COUNT, OPERATION_TIMEOUT_MS,
};

pub use security::{
    calculate_checksum, calculate_head_checksum, validate_checksum, BoundsChecker,
    IpcMessageHeader, IpcMessageType, IpcValidator, MemoryTracker, RecursionGuard, RecursionScope,
    SandboxConfig, SandboxResult, SecurityContext, TimeoutGuard,
};

pub use validation::{
    quick_validate, FontSanitizer, FontValidator, ValidationError, ValidationResult,
    ValidationWarning,
};
