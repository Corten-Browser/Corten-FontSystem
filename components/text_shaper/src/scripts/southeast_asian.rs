//! Southeast Asian script shaping
//!
//! Implements shaping for:
//! - Thai
//! - Lao
//! - Khmer

use super::ScriptShaper;

/// Character categories for Southeast Asian scripts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SEACategory {
    /// Consonant
    Consonant,
    /// Leading vowel (appears before consonant)
    LeadingVowel,
    /// Following vowel (appears after consonant)
    FollowingVowel,
    /// Above vowel
    AboveVowel,
    /// Below vowel
    BelowVowel,
    /// Tone mark
    ToneMark,
    /// Sign/modifier
    Sign,
    /// Other
    Other,
}

/// Southeast Asian script shaper
#[derive(Debug, Default)]
pub struct SoutheastAsianShaper {
    // Future: configuration options
}

impl SoutheastAsianShaper {
    /// Create a new Southeast Asian shaper
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a character is Thai
    pub fn is_thai(&self, ch: char) -> bool {
        matches!(ch, '\u{0E00}'..='\u{0E7F}')
    }

    /// Check if a character is Lao
    pub fn is_lao(&self, ch: char) -> bool {
        matches!(ch, '\u{0E80}'..='\u{0EFF}')
    }

    /// Check if a character is Khmer
    pub fn is_khmer(&self, ch: char) -> bool {
        matches!(ch, '\u{1780}'..='\u{17FF}' | '\u{19E0}'..='\u{19FF}')
    }

    /// Check if a character is from any Southeast Asian script we handle
    pub fn is_southeast_asian(&self, ch: char) -> bool {
        self.is_thai(ch) || self.is_lao(ch) || self.is_khmer(ch)
    }

    /// Get the category of a Thai character
    fn thai_category(&self, ch: char) -> SEACategory {
        match ch {
            // Thai consonants
            '\u{0E01}'..='\u{0E2E}' => SEACategory::Consonant,
            // Leading vowels (appear before consonant visually)
            '\u{0E40}'..='\u{0E44}' => SEACategory::LeadingVowel,
            // Following vowels
            '\u{0E30}' | '\u{0E32}'..='\u{0E33}' | '\u{0E45}' => SEACategory::FollowingVowel,
            // Above vowels
            '\u{0E31}' | '\u{0E34}'..='\u{0E37}' | '\u{0E47}' => SEACategory::AboveVowel,
            // Below vowels
            '\u{0E38}'..='\u{0E3A}' => SEACategory::BelowVowel,
            // Tone marks
            '\u{0E48}'..='\u{0E4B}' => SEACategory::ToneMark,
            // Signs
            '\u{0E4C}'..='\u{0E4E}' => SEACategory::Sign,
            // Thai digits and punctuation
            '\u{0E2F}' | '\u{0E3F}' | '\u{0E46}' | '\u{0E4F}'..='\u{0E5B}' => SEACategory::Other,
            _ => SEACategory::Other,
        }
    }

    /// Get the category of a Lao character
    fn lao_category(&self, ch: char) -> SEACategory {
        match ch {
            // Lao consonants
            '\u{0E81}'..='\u{0E82}'
            | '\u{0E84}'
            | '\u{0E87}'..='\u{0E88}'
            | '\u{0E8A}'
            | '\u{0E8D}'
            | '\u{0E94}'..='\u{0E97}'
            | '\u{0E99}'..='\u{0E9F}'
            | '\u{0EA1}'..='\u{0EA3}'
            | '\u{0EA5}'
            | '\u{0EA7}'
            | '\u{0EAA}'..='\u{0EAB}'
            | '\u{0EAD}'..='\u{0EB0}' => SEACategory::Consonant,
            // Leading vowels
            '\u{0EC0}'..='\u{0EC4}' => SEACategory::LeadingVowel,
            // Following vowels
            '\u{0EB2}'..='\u{0EB3}' => SEACategory::FollowingVowel,
            // Above vowels
            '\u{0EB1}' | '\u{0EB4}'..='\u{0EB7}' | '\u{0EBB}' => SEACategory::AboveVowel,
            // Below vowels
            '\u{0EB8}'..='\u{0EB9}' => SEACategory::BelowVowel,
            // Tone marks
            '\u{0EC8}'..='\u{0ECB}' => SEACategory::ToneMark,
            // Signs
            '\u{0EBC}'..='\u{0EBD}' | '\u{0ECC}'..='\u{0ECD}' => SEACategory::Sign,
            _ => SEACategory::Other,
        }
    }

    /// Get the category of a Khmer character
    fn khmer_category(&self, ch: char) -> SEACategory {
        match ch {
            // Khmer consonants
            '\u{1780}'..='\u{17A2}' => SEACategory::Consonant,
            // Independent vowels
            '\u{17A3}'..='\u{17B3}' => SEACategory::LeadingVowel,
            // Dependent vowels - various positions
            '\u{17B4}'..='\u{17B5}' => SEACategory::AboveVowel, // Inherent vowels
            '\u{17B6}' => SEACategory::FollowingVowel,          // AA
            '\u{17B7}'..='\u{17BA}' => SEACategory::AboveVowel, // I, II, Y, YY
            '\u{17BB}'..='\u{17BD}' => SEACategory::BelowVowel, // U, UU, UA
            '\u{17BE}'..='\u{17C5}' => SEACategory::FollowingVowel, // E, AI, etc.
            // Khmer signs
            '\u{17C6}'..='\u{17C8}' => SEACategory::Sign,
            // Consonant shifters
            '\u{17C9}'..='\u{17CA}' => SEACategory::Sign,
            // Other signs (includes Virama/coeng at U+17D2)
            '\u{17CB}'..='\u{17D3}' => SEACategory::Sign,
            _ => SEACategory::Other,
        }
    }

    /// Get the character category
    fn get_category(&self, ch: char) -> SEACategory {
        if self.is_thai(ch) {
            self.thai_category(ch)
        } else if self.is_lao(ch) {
            self.lao_category(ch)
        } else if self.is_khmer(ch) {
            self.khmer_category(ch)
        } else {
            SEACategory::Other
        }
    }

    /// Process a syllable cluster
    fn process_cluster(&self, chars: &[char]) -> String {
        if chars.is_empty() {
            return String::new();
        }

        let mut result = String::with_capacity(chars.len() * 3);

        // Collect characters by category
        let mut leading_vowels = Vec::new();
        let mut consonants = Vec::new();
        let mut above_marks = Vec::new();
        let mut below_marks = Vec::new();
        let mut following = Vec::new();
        let mut tone_marks = Vec::new();
        let mut others = Vec::new();

        for &ch in chars {
            match self.get_category(ch) {
                SEACategory::LeadingVowel => leading_vowels.push(ch),
                SEACategory::Consonant => consonants.push(ch),
                SEACategory::AboveVowel => above_marks.push(ch),
                SEACategory::BelowVowel => below_marks.push(ch),
                SEACategory::FollowingVowel => following.push(ch),
                SEACategory::ToneMark => tone_marks.push(ch),
                SEACategory::Sign => following.push(ch),
                SEACategory::Other => others.push(ch),
            }
        }

        // Build output in visual order for Thai/Lao
        // (Leading vowels appear before consonant visually but after logically)
        for ch in &leading_vowels {
            result.push(*ch);
        }
        for ch in &consonants {
            result.push(*ch);
        }
        for ch in &above_marks {
            result.push(*ch);
        }
        for ch in &below_marks {
            result.push(*ch);
        }
        for ch in &following {
            result.push(*ch);
        }
        for ch in &tone_marks {
            result.push(*ch);
        }
        for ch in &others {
            result.push(*ch);
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
                SEACategory::Consonant => {
                    // A consonant starts a new cluster (unless current is empty
                    // or ends with a leading vowel)
                    let should_start_new = !current.is_empty()
                        && !current
                            .iter()
                            .all(|&c| matches!(self.get_category(c), SEACategory::LeadingVowel));

                    if should_start_new {
                        clusters.push(std::mem::take(&mut current));
                    }
                    current.push(ch);
                }
                SEACategory::LeadingVowel => {
                    // Leading vowels can start a new cluster or join existing
                    if !current.is_empty() {
                        clusters.push(std::mem::take(&mut current));
                    }
                    current.push(ch);
                }
                SEACategory::FollowingVowel
                | SEACategory::AboveVowel
                | SEACategory::BelowVowel
                | SEACategory::ToneMark
                | SEACategory::Sign => {
                    // These attach to the current cluster
                    current.push(ch);
                }
                SEACategory::Other => {
                    // Flush and add standalone
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
}

impl ScriptShaper for SoutheastAsianShaper {
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
        text.chars().any(|ch| self.is_southeast_asian(ch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_thai() {
        let shaper = SoutheastAsianShaper::new();
        assert!(shaper.is_thai('\u{0E01}')); // Ko Kai
        assert!(shaper.is_thai('\u{0E40}')); // Sara E
        assert!(!shaper.is_thai('A'));
    }

    #[test]
    fn test_is_lao() {
        let shaper = SoutheastAsianShaper::new();
        assert!(shaper.is_lao('\u{0E81}')); // Ko
        assert!(!shaper.is_lao('\u{0E01}')); // Thai Ko Kai
    }

    #[test]
    fn test_is_khmer() {
        let shaper = SoutheastAsianShaper::new();
        assert!(shaper.is_khmer('\u{1780}')); // Ka
        assert!(!shaper.is_khmer('\u{0E01}')); // Thai Ko Kai
    }

    #[test]
    fn test_thai_category() {
        let shaper = SoutheastAsianShaper::new();
        assert_eq!(shaper.get_category('\u{0E01}'), SEACategory::Consonant);
        assert_eq!(shaper.get_category('\u{0E40}'), SEACategory::LeadingVowel);
        assert_eq!(shaper.get_category('\u{0E48}'), SEACategory::ToneMark);
    }

    #[test]
    fn test_can_shape() {
        let shaper = SoutheastAsianShaper::new();
        assert!(shaper.can_shape("\u{0E01}\u{0E31}"));
        assert!(shaper.can_shape("\u{0E81}")); // Lao
        assert!(shaper.can_shape("\u{1780}")); // Khmer
        assert!(!shaper.can_shape("Hello"));
    }

    #[test]
    fn test_shape() {
        let shaper = SoutheastAsianShaper::new();
        // Basic test that shaping produces output
        let result = shaper.shape("\u{0E40}\u{0E01}");
        assert!(!result.is_empty());
    }
}
