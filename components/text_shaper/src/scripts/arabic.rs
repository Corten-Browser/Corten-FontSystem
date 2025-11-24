//! Arabic script shaping
//!
//! Implements contextual Arabic letter forms (isolated, initial, medial, final)
//! and required ligatures like Lam-Alef.

use super::ScriptShaper;

/// Arabic joining type for a character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArabicJoiningType {
    /// Right-joining (only joins to the right, e.g., Alef)
    RightJoining,
    /// Left-joining (only joins to the left)
    LeftJoining,
    /// Dual-joining (joins both sides, e.g., Ba, Ta)
    DualJoining,
    /// Join-causing (causes joining but doesn't join, e.g., ZWJ)
    JoinCausing,
    /// Non-joining (doesn't join, e.g., Hamza)
    NonJoining,
    /// Transparent (doesn't affect joining, e.g., diacritics)
    Transparent,
}

impl ArabicJoiningType {
    /// Get the joining type of an Arabic character
    pub fn of(ch: char) -> Self {
        match ch {
            // Alef variants - Right-joining only
            '\u{0627}' | '\u{0622}' | '\u{0623}' | '\u{0625}' | '\u{0671}' => {
                ArabicJoiningType::RightJoining
            }
            // Dal, Thal, Ra, Zain, Waw - Right-joining
            '\u{062F}' | '\u{0630}' | '\u{0631}' | '\u{0632}' | '\u{0648}' => {
                ArabicJoiningType::RightJoining
            }
            // Hamza - Non-joining
            '\u{0621}' => ArabicJoiningType::NonJoining,
            // Tatweel - Join-causing
            '\u{0640}' => ArabicJoiningType::JoinCausing,
            // Diacritics (harakat) - Transparent
            '\u{064B}'..='\u{065F}' | '\u{0670}' => ArabicJoiningType::Transparent,
            // Most other Arabic letters are dual-joining
            '\u{0628}'..='\u{064A}' => ArabicJoiningType::DualJoining,
            // Default to non-joining for non-Arabic
            _ => ArabicJoiningType::NonJoining,
        }
    }

    /// Check if this type joins to the right
    pub fn joins_right(&self) -> bool {
        matches!(
            self,
            ArabicJoiningType::RightJoining
                | ArabicJoiningType::DualJoining
                | ArabicJoiningType::JoinCausing
        )
    }

    /// Check if this type joins to the left
    pub fn joins_left(&self) -> bool {
        matches!(
            self,
            ArabicJoiningType::LeftJoining
                | ArabicJoiningType::DualJoining
                | ArabicJoiningType::JoinCausing
        )
    }
}

/// Contextual form for Arabic letters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArabicForm {
    /// Isolated form (not connected)
    Isolated,
    /// Initial form (connected to left only)
    Initial,
    /// Medial form (connected both sides)
    Medial,
    /// Final form (connected to right only)
    Final,
}

/// Arabic text shaper
#[derive(Debug, Default)]
pub struct ArabicShaper {
    // Future: configuration options
}

impl ArabicShaper {
    /// Create a new Arabic shaper
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a character is Arabic
    pub fn is_arabic(ch: char) -> bool {
        matches!(ch,
            '\u{0600}'..='\u{06FF}' |  // Arabic block
            '\u{0750}'..='\u{077F}' |  // Arabic Supplement
            '\u{08A0}'..='\u{08FF}' |  // Arabic Extended-A
            '\u{FB50}'..='\u{FDFF}' |  // Arabic Presentation Forms-A
            '\u{FE70}'..='\u{FEFF}'    // Arabic Presentation Forms-B
        )
    }

    /// Determine the contextual form for a character at a given position
    fn determine_form(&self, chars: &[char], index: usize) -> ArabicForm {
        let ch = chars[index];
        let joining_type = ArabicJoiningType::of(ch);

        // Transparent characters don't change form
        if joining_type == ArabicJoiningType::Transparent {
            return ArabicForm::Isolated;
        }

        // Non-joining characters are always isolated
        if joining_type == ArabicJoiningType::NonJoining {
            return ArabicForm::Isolated;
        }

        // Find previous joining character (skip transparent)
        let prev_joins = self.find_prev_joining(chars, index);
        let next_joins = self.find_next_joining(chars, index);

        match (prev_joins, next_joins, joining_type) {
            // Dual-joining with both neighbors
            (true, true, ArabicJoiningType::DualJoining) => ArabicForm::Medial,
            // Dual-joining with only left neighbor
            (true, false, ArabicJoiningType::DualJoining) => ArabicForm::Final,
            // Dual-joining with only right neighbor
            (false, true, ArabicJoiningType::DualJoining) => ArabicForm::Initial,
            // Right-joining with right neighbor
            (true, _, ArabicJoiningType::RightJoining) => ArabicForm::Final,
            // Default to isolated
            _ => ArabicForm::Isolated,
        }
    }

    /// Find if previous character joins to this one
    fn find_prev_joining(&self, chars: &[char], index: usize) -> bool {
        if index == 0 {
            return false;
        }

        for i in (0..index).rev() {
            let joining_type = ArabicJoiningType::of(chars[i]);
            if joining_type == ArabicJoiningType::Transparent {
                continue;
            }
            return joining_type.joins_left();
        }
        false
    }

    /// Find if next character joins to this one
    fn find_next_joining(&self, chars: &[char], index: usize) -> bool {
        for i in (index + 1)..chars.len() {
            let joining_type = ArabicJoiningType::of(chars[i]);
            if joining_type == ArabicJoiningType::Transparent {
                continue;
            }
            return joining_type.joins_right();
        }
        false
    }

    /// Get the presentation form for a character
    fn get_presentation_form(&self, ch: char, form: ArabicForm) -> char {
        // Map Arabic characters to their presentation forms
        match ch {
            // Ba (U+0628)
            '\u{0628}' => match form {
                ArabicForm::Isolated => '\u{FE8F}',
                ArabicForm::Final => '\u{FE90}',
                ArabicForm::Initial => '\u{FE91}',
                ArabicForm::Medial => '\u{FE92}',
            },
            // Ta (U+062A)
            '\u{062A}' => match form {
                ArabicForm::Isolated => '\u{FE95}',
                ArabicForm::Final => '\u{FE96}',
                ArabicForm::Initial => '\u{FE97}',
                ArabicForm::Medial => '\u{FE98}',
            },
            // Tha (U+062B)
            '\u{062B}' => match form {
                ArabicForm::Isolated => '\u{FE99}',
                ArabicForm::Final => '\u{FE9A}',
                ArabicForm::Initial => '\u{FE9B}',
                ArabicForm::Medial => '\u{FE9C}',
            },
            // Jeem (U+062C)
            '\u{062C}' => match form {
                ArabicForm::Isolated => '\u{FE9D}',
                ArabicForm::Final => '\u{FE9E}',
                ArabicForm::Initial => '\u{FE9F}',
                ArabicForm::Medial => '\u{FEA0}',
            },
            // Ha (U+062D)
            '\u{062D}' => match form {
                ArabicForm::Isolated => '\u{FEA1}',
                ArabicForm::Final => '\u{FEA2}',
                ArabicForm::Initial => '\u{FEA3}',
                ArabicForm::Medial => '\u{FEA4}',
            },
            // Kha (U+062E)
            '\u{062E}' => match form {
                ArabicForm::Isolated => '\u{FEA5}',
                ArabicForm::Final => '\u{FEA6}',
                ArabicForm::Initial => '\u{FEA7}',
                ArabicForm::Medial => '\u{FEA8}',
            },
            // Alef (U+0627) - only isolated and final
            '\u{0627}' => match form {
                ArabicForm::Final => '\u{FE8E}',
                _ => '\u{FE8D}', // Isolated
            },
            // Lam (U+0644)
            '\u{0644}' => match form {
                ArabicForm::Isolated => '\u{FEDD}',
                ArabicForm::Final => '\u{FEDE}',
                ArabicForm::Initial => '\u{FEDF}',
                ArabicForm::Medial => '\u{FEE0}',
            },
            // Meem (U+0645)
            '\u{0645}' => match form {
                ArabicForm::Isolated => '\u{FEE1}',
                ArabicForm::Final => '\u{FEE2}',
                ArabicForm::Initial => '\u{FEE3}',
                ArabicForm::Medial => '\u{FEE4}',
            },
            // Ra (U+0631) - only isolated and final
            '\u{0631}' => match form {
                ArabicForm::Final => '\u{FEAE}',
                _ => '\u{FEAD}', // Isolated
            },
            // Seen (U+0633)
            '\u{0633}' => match form {
                ArabicForm::Isolated => '\u{FEB1}',
                ArabicForm::Final => '\u{FEB2}',
                ArabicForm::Initial => '\u{FEB3}',
                ArabicForm::Medial => '\u{FEB4}',
            },
            // Ain (U+0639)
            '\u{0639}' => match form {
                ArabicForm::Isolated => '\u{FEC9}',
                ArabicForm::Final => '\u{FECA}',
                ArabicForm::Initial => '\u{FECB}',
                ArabicForm::Medial => '\u{FECC}',
            },
            // Default: return original character
            _ => ch,
        }
    }

    /// Check for and apply Lam-Alef ligature
    fn apply_lam_alef_ligature(&self, chars: &[char]) -> Option<(char, usize)> {
        if chars.len() < 2 {
            return None;
        }

        // Check for Lam (U+0644) followed by Alef variants
        if chars[0] == '\u{0644}' {
            match chars[1] {
                '\u{0627}' => Some(('\u{FEFB}', 2)), // Lam + Alef -> Isolated
                '\u{0622}' => Some(('\u{FEF5}', 2)), // Lam + Alef with Madda
                '\u{0623}' => Some(('\u{FEF7}', 2)), // Lam + Alef with Hamza Above
                '\u{0625}' => Some(('\u{FEF9}', 2)), // Lam + Alef with Hamza Below
                _ => None,
            }
        } else {
            None
        }
    }
}

impl ScriptShaper for ArabicShaper {
    fn shape(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut result = String::with_capacity(text.len());
        let mut i = 0;

        while i < chars.len() {
            // Check for Lam-Alef ligature
            if let Some((ligature, consumed)) = self.apply_lam_alef_ligature(&chars[i..]) {
                result.push(ligature);
                i += consumed;
                continue;
            }

            let ch = chars[i];

            // Only shape Arabic characters
            if Self::is_arabic(ch) {
                let form = self.determine_form(&chars, i);
                let shaped = self.get_presentation_form(ch, form);
                result.push(shaped);
            } else {
                result.push(ch);
            }

            i += 1;
        }

        result
    }

    fn can_shape(&self, text: &str) -> bool {
        text.chars().any(Self::is_arabic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joining_type() {
        assert_eq!(
            ArabicJoiningType::of('\u{0627}'),
            ArabicJoiningType::RightJoining
        );
        assert_eq!(
            ArabicJoiningType::of('\u{0628}'),
            ArabicJoiningType::DualJoining
        );
        assert_eq!(
            ArabicJoiningType::of('\u{0621}'),
            ArabicJoiningType::NonJoining
        );
    }

    #[test]
    fn test_is_arabic() {
        assert!(ArabicShaper::is_arabic('\u{0627}'));
        assert!(ArabicShaper::is_arabic('\u{0628}'));
        assert!(!ArabicShaper::is_arabic('A'));
        assert!(!ArabicShaper::is_arabic('1'));
    }

    #[test]
    fn test_isolated_form() {
        let shaper = ArabicShaper::new();
        let result = shaper.shape("\u{0628}");
        assert_eq!(result, "\u{FE8F}"); // Isolated Ba
    }

    #[test]
    fn test_can_shape() {
        let shaper = ArabicShaper::new();
        assert!(shaper.can_shape("\u{0627}\u{0628}"));
        assert!(!shaper.can_shape("Hello"));
    }
}
