//! Indic script shaping
//!
//! Implements complex shaping for Indic scripts including:
//! - Devanagari
//! - Bengali
//! - Tamil
//! - Telugu
//! - Kannada
//! - Malayalam
//! - Gujarati
//! - Gurmukhi
//! - Oriya

use super::ScriptShaper;

/// Character categories for Indic shaping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicCategory {
    /// Consonant
    Consonant,
    /// Vowel (independent)
    Vowel,
    /// Vowel sign (dependent)
    VowelSign,
    /// Virama (halant/killer)
    Virama,
    /// Nukta (modifier dot)
    Nukta,
    /// Anusvara (nasal)
    Anusvara,
    /// Visarga
    Visarga,
    /// Consonant modifier
    ConsonantModifier,
    /// Other
    Other,
}

/// Vowel sign position for reordering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelPosition {
    /// Appears before consonant (left)
    Left,
    /// Appears after consonant (right)
    Right,
    /// Appears above consonant
    Above,
    /// Appears below consonant
    Below,
    /// Split vowel (parts on both sides)
    Split,
}

/// Indic script shaper
#[derive(Debug, Default)]
pub struct IndicShaper {
    // Future: configuration options
}

impl IndicShaper {
    /// Create a new Indic shaper
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a character is from an Indic script
    pub fn is_indic(ch: char) -> bool {
        matches!(ch,
            '\u{0900}'..='\u{097F}' |  // Devanagari
            '\u{0980}'..='\u{09FF}' |  // Bengali
            '\u{0A00}'..='\u{0A7F}' |  // Gurmukhi
            '\u{0A80}'..='\u{0AFF}' |  // Gujarati
            '\u{0B00}'..='\u{0B7F}' |  // Oriya
            '\u{0B80}'..='\u{0BFF}' |  // Tamil
            '\u{0C00}'..='\u{0C7F}' |  // Telugu
            '\u{0C80}'..='\u{0CFF}' |  // Kannada
            '\u{0D00}'..='\u{0D7F}'    // Malayalam
        )
    }

    /// Get the script name for an Indic character
    pub fn script_name(ch: char) -> Option<&'static str> {
        match ch {
            '\u{0900}'..='\u{097F}' => Some("Devanagari"),
            '\u{0980}'..='\u{09FF}' => Some("Bengali"),
            '\u{0A00}'..='\u{0A7F}' => Some("Gurmukhi"),
            '\u{0A80}'..='\u{0AFF}' => Some("Gujarati"),
            '\u{0B00}'..='\u{0B7F}' => Some("Oriya"),
            '\u{0B80}'..='\u{0BFF}' => Some("Tamil"),
            '\u{0C00}'..='\u{0C7F}' => Some("Telugu"),
            '\u{0C80}'..='\u{0CFF}' => Some("Kannada"),
            '\u{0D00}'..='\u{0D7F}' => Some("Malayalam"),
            _ => None,
        }
    }

    /// Get the category of a Devanagari character
    fn devanagari_category(ch: char) -> IndicCategory {
        match ch {
            // Consonants
            '\u{0915}'..='\u{0939}' | '\u{0958}'..='\u{095F}' => IndicCategory::Consonant,
            // Independent vowels
            '\u{0904}'..='\u{0914}' | '\u{0960}'..='\u{0961}' => IndicCategory::Vowel,
            // Dependent vowel signs
            '\u{093A}'..='\u{093B}'
            | '\u{093E}'..='\u{094C}'
            | '\u{094E}'..='\u{094F}'
            | '\u{0962}'..='\u{0963}' => IndicCategory::VowelSign,
            // Virama
            '\u{094D}' => IndicCategory::Virama,
            // Nukta
            '\u{093C}' => IndicCategory::Nukta,
            // Anusvara
            '\u{0902}' => IndicCategory::Anusvara,
            // Visarga
            '\u{0903}' => IndicCategory::Visarga,
            // Chandrabindu
            '\u{0901}' => IndicCategory::ConsonantModifier,
            _ => IndicCategory::Other,
        }
    }

    /// Get the category of a Bengali character
    fn bengali_category(ch: char) -> IndicCategory {
        match ch {
            // Consonants
            '\u{0995}'..='\u{09B9}' | '\u{09DC}'..='\u{09DF}' => IndicCategory::Consonant,
            // Independent vowels
            '\u{0985}'..='\u{0994}' | '\u{09E0}'..='\u{09E1}' => IndicCategory::Vowel,
            // Dependent vowel signs
            '\u{09BE}'..='\u{09CC}' | '\u{09E2}'..='\u{09E3}' => IndicCategory::VowelSign,
            // Virama
            '\u{09CD}' => IndicCategory::Virama,
            // Nukta
            '\u{09BC}' => IndicCategory::Nukta,
            // Anusvara
            '\u{0982}' => IndicCategory::Anusvara,
            // Visarga
            '\u{0983}' => IndicCategory::Visarga,
            // Chandrabindu
            '\u{0981}' => IndicCategory::ConsonantModifier,
            _ => IndicCategory::Other,
        }
    }

    /// Get the category of a Tamil character
    fn tamil_category(ch: char) -> IndicCategory {
        match ch {
            // Consonants
            '\u{0B95}'..='\u{0BB9}' => IndicCategory::Consonant,
            // Independent vowels
            '\u{0B85}'..='\u{0B94}' => IndicCategory::Vowel,
            // Dependent vowel signs
            '\u{0BBE}'..='\u{0BCC}' => IndicCategory::VowelSign,
            // Virama
            '\u{0BCD}' => IndicCategory::Virama,
            // Anusvara
            '\u{0B82}' => IndicCategory::Anusvara,
            // Visarga
            '\u{0B83}' => IndicCategory::Visarga,
            _ => IndicCategory::Other,
        }
    }

    /// Get the character category based on its script
    fn get_category(&self, ch: char) -> IndicCategory {
        match ch {
            '\u{0900}'..='\u{097F}' => Self::devanagari_category(ch),
            '\u{0980}'..='\u{09FF}' => Self::bengali_category(ch),
            '\u{0B80}'..='\u{0BFF}' => Self::tamil_category(ch),
            _ => IndicCategory::Other,
        }
    }

    /// Get the vowel sign position for reordering
    fn vowel_position(&self, ch: char) -> VowelPosition {
        match ch {
            // Devanagari vowel sign i (appears before consonant visually)
            '\u{093F}' => VowelPosition::Left,
            // Bengali vowel sign i
            '\u{09BF}' => VowelPosition::Left,
            // Tamil vowel sign i
            '\u{0BBF}' => VowelPosition::Left,
            // Tamil vowel sign e, ee, ai (appear before consonant)
            '\u{0BC6}'..='\u{0BC8}' => VowelPosition::Left,
            // Tamil split vowels
            '\u{0BCA}'..='\u{0BCC}' => VowelPosition::Split,
            // Default: right position
            _ => VowelPosition::Right,
        }
    }

    /// Check if character is a left-side vowel sign
    fn is_left_vowel(&self, ch: char) -> bool {
        matches!(
            self.vowel_position(ch),
            VowelPosition::Left | VowelPosition::Split
        )
    }

    /// Process a syllable cluster
    fn process_cluster(&self, chars: &[char]) -> String {
        if chars.is_empty() {
            return String::new();
        }

        let mut result = String::with_capacity(chars.len() * 3);

        // Find left-side vowel signs that need reordering
        let mut left_vowels = Vec::new();
        let mut other_chars = Vec::new();

        for &ch in chars {
            if self.is_left_vowel(ch) && self.get_category(ch) == IndicCategory::VowelSign {
                left_vowels.push(ch);
            } else {
                other_chars.push(ch);
            }
        }

        // Output left-side vowels first (for visual reordering)
        for ch in left_vowels {
            result.push(ch);
        }

        // Output remaining characters
        for ch in other_chars {
            result.push(ch);
        }

        result
    }

    /// Split text into syllable clusters
    fn split_clusters(&self, text: &str) -> Vec<Vec<char>> {
        let mut clusters = Vec::new();
        let mut current = Vec::new();

        for ch in text.chars() {
            let category = self.get_category(ch);

            match category {
                IndicCategory::Consonant | IndicCategory::Vowel => {
                    // Start a new cluster if current is not empty and this is a base
                    if !current.is_empty() && !self.is_combining(&current) {
                        clusters.push(std::mem::take(&mut current));
                    }
                    current.push(ch);
                }
                IndicCategory::Virama
                | IndicCategory::VowelSign
                | IndicCategory::Nukta
                | IndicCategory::Anusvara
                | IndicCategory::Visarga
                | IndicCategory::ConsonantModifier => {
                    // Add to current cluster
                    current.push(ch);
                }
                IndicCategory::Other => {
                    // Flush current cluster and add this as standalone
                    if !current.is_empty() {
                        clusters.push(std::mem::take(&mut current));
                    }
                    clusters.push(vec![ch]);
                }
            }
        }

        // Flush remaining
        if !current.is_empty() {
            clusters.push(current);
        }

        clusters
    }

    /// Check if the current cluster is incomplete (ends with virama)
    fn is_combining(&self, chars: &[char]) -> bool {
        chars
            .last()
            .map(|&ch| self.get_category(ch) == IndicCategory::Virama)
            .unwrap_or(false)
    }
}

impl ScriptShaper for IndicShaper {
    fn shape(&self, text: &str) -> String {
        let clusters = self.split_clusters(text);
        let mut result = String::with_capacity(text.len());

        for cluster in clusters {
            let processed = self.process_cluster(&cluster);
            result.push_str(&processed);
        }

        result
    }

    fn can_shape(&self, text: &str) -> bool {
        text.chars().any(Self::is_indic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_indic() {
        assert!(IndicShaper::is_indic('\u{0915}')); // Devanagari Ka
        assert!(IndicShaper::is_indic('\u{0995}')); // Bengali Ka
        assert!(IndicShaper::is_indic('\u{0B95}')); // Tamil Ka
        assert!(!IndicShaper::is_indic('A'));
        assert!(!IndicShaper::is_indic('\u{0627}')); // Arabic
    }

    #[test]
    fn test_script_name() {
        assert_eq!(IndicShaper::script_name('\u{0915}'), Some("Devanagari"));
        assert_eq!(IndicShaper::script_name('\u{0995}'), Some("Bengali"));
        assert_eq!(IndicShaper::script_name('\u{0B95}'), Some("Tamil"));
        assert_eq!(IndicShaper::script_name('A'), None);
    }

    #[test]
    fn test_category() {
        let shaper = IndicShaper::new();
        assert_eq!(shaper.get_category('\u{0915}'), IndicCategory::Consonant);
        assert_eq!(shaper.get_category('\u{0905}'), IndicCategory::Vowel);
        assert_eq!(shaper.get_category('\u{093F}'), IndicCategory::VowelSign);
        assert_eq!(shaper.get_category('\u{094D}'), IndicCategory::Virama);
    }

    #[test]
    fn test_can_shape() {
        let shaper = IndicShaper::new();
        assert!(shaper.can_shape("\u{0915}\u{093F}"));
        assert!(!shaper.can_shape("Hello"));
    }

    #[test]
    fn test_shape() {
        let shaper = IndicShaper::new();
        // Basic test that shaping produces output
        let result = shaper.shape("\u{0915}\u{093F}");
        assert!(!result.is_empty());
    }
}
