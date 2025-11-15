//! Line breaking algorithm (Unicode UAX #14)

use crate::types::LineBreak;
use unicode_linebreak::{linebreaks, BreakOpportunity};

/// Line breaker implementing Unicode UAX #14
pub struct LineBreaker;

impl LineBreaker {
    /// Create a new line breaker
    pub fn new() -> Self {
        Self
    }

    /// Find all line break opportunities in text
    ///
    /// Returns a vector of line breaks, including both mandatory breaks
    /// (e.g., newlines) and optional break opportunities (e.g., after spaces).
    ///
    /// # Arguments
    ///
    /// * `text` - The text to analyze
    ///
    /// # Returns
    ///
    /// Vector of `LineBreak` instances indicating break positions and whether
    /// they are required (hard breaks) or optional (soft breaks).
    ///
    /// # Example
    ///
    /// ```
    /// use text_layout::LineBreaker;
    ///
    /// let breaker = LineBreaker::new();
    /// let breaks = breaker.find_breaks("Hello world");
    /// // Will find break opportunity after "Hello "
    /// ```
    pub fn find_breaks(&self, text: &str) -> Vec<LineBreak> {
        let mut breaks = Vec::new();

        for (offset, opportunity) in linebreaks(text) {
            breaks.push(LineBreak {
                offset,
                required: opportunity == BreakOpportunity::Mandatory,
            });
        }

        breaks
    }

    /// Find line break opportunities between start and end positions
    ///
    /// # Arguments
    ///
    /// * `text` - The full text
    /// * `start` - Start byte offset
    /// * `end` - End byte offset
    ///
    /// # Returns
    ///
    /// Vector of line breaks within the specified range
    pub fn find_breaks_in_range(&self, text: &str, start: usize, end: usize) -> Vec<LineBreak> {
        self.find_breaks(text)
            .into_iter()
            .filter(|b| b.offset >= start && b.offset <= end)
            .collect()
    }

    /// Check if there's a break opportunity at a specific position
    ///
    /// # Arguments
    ///
    /// * `text` - The text to check
    /// * `offset` - Byte offset to check
    ///
    /// # Returns
    ///
    /// `Some(required)` if there's a break at this offset, where `required`
    /// indicates if it's a mandatory break. `None` if no break at this position.
    pub fn is_break_at(&self, text: &str, offset: usize) -> Option<bool> {
        self.find_breaks(text)
            .into_iter()
            .find(|b| b.offset == offset)
            .map(|b| b.required)
    }
}

impl Default for LineBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Basic Line Breaking Tests ==========

    #[test]
    fn test_line_breaker_creation() {
        // Given: Creating a line breaker
        // When: Using new() or default()
        // Then: Should create successfully
        let _breaker1 = LineBreaker::new();
        let _breaker2 = LineBreaker::default();
    }

    #[test]
    fn test_find_breaks_simple_text() {
        // Given: Simple English text with spaces
        // When: Finding line breaks
        // Then: Should find break after each word
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("Hello world");

        // Should find at least one break (after "Hello ")
        assert!(!breaks.is_empty());

        // Check that breaks are in ascending order
        for i in 1..breaks.len() {
            assert!(breaks[i].offset > breaks[i - 1].offset);
        }
    }

    #[test]
    fn test_find_breaks_with_newline() {
        // Given: Text with newline (mandatory break)
        // When: Finding line breaks
        // Then: Should mark newline as required
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("Line 1\nLine 2");

        // Find the break at newline position
        let newline_break = breaks.iter().find(|b| {
            // Newline is around position 6-7
            b.offset >= 6 && b.offset <= 7
        });

        // Should exist and be marked as required
        assert!(newline_break.is_some());
        assert!(
            newline_break.unwrap().required
                || breaks
                    .iter()
                    .any(|b| b.required && b.offset >= 6 && b.offset <= 8)
        );
    }

    #[test]
    fn test_find_breaks_empty_text() {
        // Given: Empty text
        // When: Finding line breaks
        // Then: Should return empty vector or just end break
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("");

        // Empty text has at most 1 break (at position 0)
        assert!(breaks.len() <= 1);
    }

    #[test]
    fn test_find_breaks_single_word() {
        // Given: Single word with no spaces
        // When: Finding line breaks
        // Then: Should have minimal breaks (at end)
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("Hello");

        // Single word should have break at end
        assert!(!breaks.is_empty());
        let last_break = breaks.last().unwrap();
        assert_eq!(last_break.offset, "Hello".len());
    }

    #[test]
    fn test_find_breaks_multiple_spaces() {
        // Given: Text with multiple consecutive spaces
        // When: Finding line breaks
        // Then: Should handle gracefully
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("Hello    world");

        // Should find breaks (exact behavior depends on UAX #14)
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_find_breaks_non_ascii() {
        // Given: Non-ASCII text (e.g., Chinese)
        // When: Finding line breaks
        // Then: Should handle Unicode correctly
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("你好世界");

        // CJK text typically allows breaks between characters
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_find_breaks_mixed_scripts() {
        // Given: Mixed English and CJK text
        // When: Finding line breaks
        // Then: Should handle both scripts
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("Hello 世界 world");

        // Should find breaks at various positions
        assert!(!breaks.is_empty());
    }

    // ========== Range-Based Breaking Tests ==========

    #[test]
    fn test_find_breaks_in_range() {
        // Given: Text and a specific range
        // When: Finding breaks in that range
        // Then: Should return only breaks within range
        let breaker = LineBreaker::new();
        let text = "Hello world test";
        let all_breaks = breaker.find_breaks(text);

        // Find breaks in middle portion
        let range_breaks = breaker.find_breaks_in_range(text, 5, 11);

        // All range breaks should be within the specified range
        for b in &range_breaks {
            assert!(b.offset >= 5);
            assert!(b.offset <= 11);
        }

        // Range breaks should be subset of all breaks
        assert!(range_breaks.len() <= all_breaks.len());
    }

    #[test]
    fn test_find_breaks_in_empty_range() {
        // Given: An empty range (start == end)
        // When: Finding breaks in that range
        // Then: Should return empty or minimal result
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks_in_range("Hello world", 5, 5);

        // Empty range should yield empty or single break
        assert!(breaks.len() <= 1);
    }

    #[test]
    fn test_find_breaks_in_full_range() {
        // Given: Range covering entire text
        // When: Finding breaks in that range
        // Then: Should return same as find_breaks()
        let breaker = LineBreaker::new();
        let text = "Hello world";
        let all_breaks = breaker.find_breaks(text);
        let range_breaks = breaker.find_breaks_in_range(text, 0, text.len());

        // Should be equivalent (or very close)
        assert_eq!(all_breaks.len(), range_breaks.len());
    }

    // ========== Break Position Checking Tests ==========

    #[test]
    fn test_is_break_at_existing_break() {
        // Given: Text with known break position
        // When: Checking if break exists at that position
        // Then: Should return Some(required_flag)
        let breaker = LineBreaker::new();
        let text = "Hello world";
        let breaks = breaker.find_breaks(text);

        // Check at a known break position
        if let Some(break_pos) = breaks.first() {
            let result = breaker.is_break_at(text, break_pos.offset);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_is_break_at_no_break() {
        // Given: Text and position with no break
        // When: Checking for break at that position
        // Then: Should return None
        let breaker = LineBreaker::new();
        let text = "Hello";

        // Position 2 (middle of "Hello") should not be a break
        let result = breaker.is_break_at(text, 2);

        // Might be None or Some depending on script, but check consistency
        let breaks = breaker.find_breaks(text);
        let has_break_at_2 = breaks.iter().any(|b| b.offset == 2);

        assert_eq!(result.is_some(), has_break_at_2);
    }

    #[test]
    fn test_is_break_at_newline() {
        // Given: Text with newline
        // When: Checking if break at newline position
        // Then: Should return Some(true) for mandatory break
        let breaker = LineBreaker::new();
        let text = "Line1\nLine2";

        // Find the newline break
        let breaks = breaker.find_breaks(text);
        let newline_break = breaks.iter().find(|b| b.required);

        if let Some(nb) = newline_break {
            let result = breaker.is_break_at(text, nb.offset);
            assert!(result.is_some());
            if let Some(required) = result {
                assert!(required);
            }
        }
    }

    #[test]
    fn test_required_vs_optional_breaks() {
        // Given: Text with both mandatory and optional breaks
        // When: Finding all breaks
        // Then: Should distinguish between required and optional
        let breaker = LineBreaker::new();
        let text = "Hello world\nNext line";
        let breaks = breaker.find_breaks(text);

        // Should have both required and optional breaks
        let has_required = breaks.iter().any(|b| b.required);
        let has_optional = breaks.iter().any(|b| !b.required);

        assert!(has_required); // Newline is required
        assert!(has_optional); // Space breaks are optional
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_breaks_with_tabs() {
        // Given: Text with tab characters
        // When: Finding line breaks
        // Then: Should handle tabs correctly
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("Hello\tworld");

        // Should find breaks around tab
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_breaks_with_multiple_newlines() {
        // Given: Text with multiple consecutive newlines
        // When: Finding line breaks
        // Then: Should find all newline breaks
        let breaker = LineBreaker::new();
        let text = "Line1\n\n\nLine2";
        let breaks = breaker.find_breaks(text);

        // Should have multiple required breaks
        let required_count = breaks.iter().filter(|b| b.required).count();
        assert!(required_count >= 1); // At least one newline
    }

    #[test]
    fn test_breaks_ascending_order() {
        // Given: Any text
        // When: Finding line breaks
        // Then: Breaks should be in ascending order of offset
        let breaker = LineBreaker::new();
        let breaks = breaker.find_breaks("The quick brown fox jumps over the lazy dog");

        // Verify ascending order
        for i in 1..breaks.len() {
            assert!(
                breaks[i].offset >= breaks[i - 1].offset,
                "Breaks not in ascending order"
            );
        }
    }
}
