//! Unicode Bidirectional Algorithm (UAX #9) implementation
//!
//! This module implements the Unicode Bidirectional Algorithm for handling
//! text that mixes left-to-right and right-to-left scripts.

use unicode_bidi::{BidiInfo as UnicodeBidiInfo, Level, ParagraphInfo};

/// Bidi level representing embedding depth and direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BidiLevel(pub u8);

impl BidiLevel {
    /// Create a new Bidi level
    pub fn new(level: u8) -> Self {
        Self(level)
    }

    /// Check if this level represents RTL direction
    pub fn is_rtl(&self) -> bool {
        self.0 % 2 == 1
    }

    /// Check if this level represents LTR direction
    pub fn is_ltr(&self) -> bool {
        self.0 % 2 == 0
    }

    /// LTR level (0)
    pub fn ltr() -> Self {
        Self(0)
    }

    /// RTL level (1)
    pub fn rtl() -> Self {
        Self(1)
    }
}

impl From<Level> for BidiLevel {
    fn from(level: Level) -> Self {
        Self(level.number())
    }
}

/// Paragraph direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParagraphDirection {
    /// Left-to-right paragraph
    Ltr,
    /// Right-to-left paragraph
    Rtl,
}

impl ParagraphDirection {
    /// Get the base level for this direction
    pub fn level(&self) -> BidiLevel {
        match self {
            ParagraphDirection::Ltr => BidiLevel::ltr(),
            ParagraphDirection::Rtl => BidiLevel::rtl(),
        }
    }
}

/// Unicode Bidi class (character type)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BidiClass {
    /// Left-to-Right (L)
    L,
    /// Right-to-Left (R)
    R,
    /// Right-to-Left Arabic (AL)
    AL,
    /// European Number (EN)
    EN,
    /// European Number Separator (ES)
    ES,
    /// European Number Terminator (ET)
    ET,
    /// Arabic Number (AN)
    AN,
    /// Common Number Separator (CS)
    CS,
    /// Nonspacing Mark (NSM)
    NSM,
    /// Boundary Neutral (BN)
    BN,
    /// Paragraph Separator (B)
    B,
    /// Segment Separator (S)
    S,
    /// Whitespace (WS)
    WS,
    /// Other Neutral (ON)
    ON,
    /// Left-to-Right Embedding (LRE)
    LRE,
    /// Left-to-Right Override (LRO)
    LRO,
    /// Right-to-Left Embedding (RLE)
    RLE,
    /// Right-to-Left Override (RLO)
    RLO,
    /// Pop Directional Format (PDF)
    PDF,
    /// Left-to-Right Isolate (LRI)
    LRI,
    /// Right-to-Left Isolate (RLI)
    RLI,
    /// First Strong Isolate (FSI)
    FSI,
    /// Pop Directional Isolate (PDI)
    PDI,
}

impl BidiClass {
    /// Get the Bidi class of a character
    pub fn of(ch: char) -> Self {
        use unicode_bidi::BidiClass as UB;

        // Use unicode-bidi crate for accurate classification
        match unicode_bidi::bidi_class(ch) {
            UB::L => BidiClass::L,
            UB::R => BidiClass::R,
            UB::AL => BidiClass::AL,
            UB::EN => BidiClass::EN,
            UB::ES => BidiClass::ES,
            UB::ET => BidiClass::ET,
            UB::AN => BidiClass::AN,
            UB::CS => BidiClass::CS,
            UB::NSM => BidiClass::NSM,
            UB::BN => BidiClass::BN,
            UB::B => BidiClass::B,
            UB::S => BidiClass::S,
            UB::WS => BidiClass::WS,
            UB::ON => BidiClass::ON,
            UB::LRE => BidiClass::LRE,
            UB::LRO => BidiClass::LRO,
            UB::RLE => BidiClass::RLE,
            UB::RLO => BidiClass::RLO,
            UB::PDF => BidiClass::PDF,
            UB::LRI => BidiClass::LRI,
            UB::RLI => BidiClass::RLI,
            UB::FSI => BidiClass::FSI,
            UB::PDI => BidiClass::PDI,
        }
    }

    /// Check if this class is strongly directional
    pub fn is_strong(&self) -> bool {
        matches!(self, BidiClass::L | BidiClass::R | BidiClass::AL)
    }

    /// Check if this is a left-to-right class
    pub fn is_ltr(&self) -> bool {
        matches!(self, BidiClass::L)
    }

    /// Check if this is a right-to-left class
    pub fn is_rtl(&self) -> bool {
        matches!(self, BidiClass::R | BidiClass::AL)
    }
}

/// A run of text with a single direction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BidiRun {
    /// Start byte index in the original text
    pub start: usize,
    /// End byte index (exclusive) in the original text
    pub end: usize,
    /// Bidi level of this run
    pub level: BidiLevel,
}

impl BidiRun {
    /// Create a new Bidi run
    pub fn new(start: usize, end: usize, level: BidiLevel) -> Self {
        Self { start, end, level }
    }

    /// Check if this run is RTL
    pub fn is_rtl(&self) -> bool {
        self.level.is_rtl()
    }

    /// Check if this run is LTR
    pub fn is_ltr(&self) -> bool {
        self.level.is_ltr()
    }

    /// Get the length of this run in bytes
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this run is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Bidi information for a text
pub struct BidiInfo<'a> {
    /// The original text
    text: &'a str,
    /// Unicode-bidi analysis
    info: UnicodeBidiInfo<'a>,
    /// Paragraph info
    paragraph: ParagraphInfo,
}

impl<'a> BidiInfo<'a> {
    /// Create new BidiInfo for the given text
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    /// * `default_direction` - Optional default paragraph direction
    ///
    /// # Returns
    ///
    /// BidiInfo with full bidirectional analysis
    pub fn new(text: &'a str, default_direction: Option<ParagraphDirection>) -> Self {
        let level = default_direction.map(|d| match d {
            ParagraphDirection::Ltr => Level::ltr(),
            ParagraphDirection::Rtl => Level::rtl(),
        });

        let info = UnicodeBidiInfo::new(text, level);

        // Get the first paragraph (we treat entire text as one paragraph)
        let paragraph = if info.paragraphs.is_empty() {
            ParagraphInfo {
                range: 0..text.len(),
                level: level.unwrap_or_else(Level::ltr),
            }
        } else {
            info.paragraphs[0].clone()
        };

        Self {
            text,
            info,
            paragraph,
        }
    }

    /// Get the paragraph direction
    pub fn paragraph_direction(&self) -> ParagraphDirection {
        if self.paragraph.level.is_rtl() {
            ParagraphDirection::Rtl
        } else {
            ParagraphDirection::Ltr
        }
    }

    /// Get the paragraph level
    pub fn paragraph_level(&self) -> BidiLevel {
        BidiLevel::from(self.paragraph.level)
    }

    /// Get the levels for each character
    pub fn levels(&self) -> Vec<BidiLevel> {
        self.info
            .levels
            .iter()
            .map(|&l| BidiLevel::from(l))
            .collect()
    }

    /// Get visual runs for a range of the text
    ///
    /// Returns runs in visual (display) order
    pub fn visual_runs(&self, start: usize, end: usize) -> Vec<BidiRun> {
        let line_range = start..end.min(self.text.len());

        let (levels, runs) = self.info.visual_runs(&self.paragraph, line_range.clone());

        runs.iter()
            .map(|range| {
                let level_idx = range.start.saturating_sub(line_range.start);
                let level = if level_idx < levels.len() {
                    BidiLevel::from(levels[level_idx])
                } else {
                    self.paragraph_level()
                };
                BidiRun::new(range.start, range.end, level)
            })
            .collect()
    }

    /// Get reordered character indices for visual display
    ///
    /// Returns indices mapping logical to visual order
    pub fn reordered_indices(&self, start: usize, end: usize) -> Vec<usize> {
        let line_range = start..end.min(self.text.len());

        // Get visual runs
        let (_, runs) = self.info.visual_runs(&self.paragraph, line_range.clone());

        // Build reordered indices
        let mut indices = Vec::with_capacity(end - start);

        for run in &runs {
            let run_len = run.end - run.start;
            // Get the level for this run
            let level_idx = run.start.saturating_sub(line_range.start);
            let is_rtl = level_idx < self.info.levels.len() && self.info.levels[level_idx].is_rtl();

            if is_rtl {
                // RTL run: reverse the indices
                for i in (0..run_len).rev() {
                    indices.push(run.start - start + i);
                }
            } else {
                // LTR run: keep natural order
                for i in 0..run_len {
                    indices.push(run.start - start + i);
                }
            }
        }

        // Handle case where runs don't cover the full range
        if indices.is_empty() {
            // Check if text is RTL
            let is_rtl = !self.info.levels.is_empty() && self.info.levels[0].is_rtl();
            let len = end - start;
            if is_rtl {
                indices = (0..len).rev().collect();
            } else {
                indices = (0..len).collect();
            }
        }

        indices
    }

    /// Get the original text
    pub fn text(&self) -> &str {
        self.text
    }
}

/// A paragraph with bidi properties
pub struct BidiParagraph<'a> {
    /// The paragraph text
    text: &'a str,
    /// Bidi info for this paragraph
    info: BidiInfo<'a>,
}

impl<'a> BidiParagraph<'a> {
    /// Create a new BidiParagraph
    pub fn new(text: &'a str, default_direction: Option<ParagraphDirection>) -> Self {
        let info = BidiInfo::new(text, default_direction);
        Self { text, info }
    }

    /// Get the paragraph level
    pub fn level(&self) -> BidiLevel {
        self.info.paragraph_level()
    }

    /// Get the paragraph direction
    pub fn direction(&self) -> ParagraphDirection {
        self.info.paragraph_direction()
    }

    /// Get visual runs
    pub fn visual_runs(&self) -> Vec<BidiRun> {
        self.info.visual_runs(0, self.text.len())
    }

    /// Get the text
    pub fn text(&self) -> &str {
        self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidi_level() {
        assert!(BidiLevel::ltr().is_ltr());
        assert!(BidiLevel::rtl().is_rtl());
        assert!(!BidiLevel::ltr().is_rtl());
        assert!(!BidiLevel::rtl().is_ltr());
    }

    #[test]
    fn test_bidi_class_strong() {
        assert!(BidiClass::L.is_strong());
        assert!(BidiClass::R.is_strong());
        assert!(BidiClass::AL.is_strong());
        assert!(!BidiClass::EN.is_strong());
        assert!(!BidiClass::WS.is_strong());
    }

    #[test]
    fn test_paragraph_direction() {
        assert_eq!(ParagraphDirection::Ltr.level(), BidiLevel::ltr());
        assert_eq!(ParagraphDirection::Rtl.level(), BidiLevel::rtl());
    }

    #[test]
    fn test_bidi_run() {
        let run = BidiRun::new(0, 10, BidiLevel::ltr());
        assert!(run.is_ltr());
        assert!(!run.is_rtl());
        assert_eq!(run.len(), 10);
        assert!(!run.is_empty());

        let empty_run = BidiRun::new(5, 5, BidiLevel::rtl());
        assert!(empty_run.is_empty());
    }
}
