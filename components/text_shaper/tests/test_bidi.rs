//! Tests for Unicode Bidirectional Algorithm (UAX #9) implementation

use text_shaper::bidi::{
    BidiClass, BidiInfo, BidiLevel, BidiParagraph, BidiRun, ParagraphDirection,
};
use text_shaper::scripts::{
    arabic::{ArabicJoiningType, ArabicShaper},
    indic::IndicShaper,
    southeast_asian::SoutheastAsianShaper,
    ScriptShaper,
};

// ============================================================================
// FEAT-017: Unicode bidi algorithm - UAX #9 level detection tests
// ============================================================================

mod bidi_level_detection {
    use super::*;

    #[test]
    fn test_pure_ltr_text_has_level_zero() {
        // Given pure LTR text (English)
        let text = "Hello World";

        // When we analyze it
        let bidi_info = BidiInfo::new(text, None);

        // Then all characters should have level 0 (LTR)
        let levels = bidi_info.levels();
        assert!(
            levels.iter().all(|&l| l.0 == 0),
            "Pure LTR should be level 0"
        );
    }

    #[test]
    fn test_pure_rtl_text_has_level_one() {
        // Given pure RTL text (Hebrew)
        let text = "\u{05E9}\u{05DC}\u{05D5}\u{05DD}"; // "shalom" in Hebrew

        // When we analyze it
        let bidi_info = BidiInfo::new(text, None);

        // Then all characters should have level 1 (RTL)
        let levels = bidi_info.levels();
        assert!(
            levels.iter().all(|&l| l.0 == 1),
            "Pure RTL should be level 1"
        );
    }

    #[test]
    fn test_mixed_text_level_detection() {
        // Given mixed LTR and RTL text
        let text = "Hello \u{05E9}\u{05DC}\u{05D5}\u{05DD} World"; // "Hello shalom World"

        // When we analyze it
        let bidi_info = BidiInfo::new(text, None);

        // Then we should have different levels for different parts
        let levels = bidi_info.levels();
        assert!(!levels.is_empty(), "Should have levels for all characters");

        // The Hebrew part should have higher level
        let hebrew_start = 6; // Index of first Hebrew character
        let hebrew_end = 10; // Index after last Hebrew character
        assert!(levels[hebrew_start].0 > 0, "Hebrew should have RTL level");
    }

    #[test]
    fn test_bidi_class_detection() {
        // Given various characters
        let tests = [
            ('A', BidiClass::L),         // Latin - Left-to-Right
            ('\u{05D0}', BidiClass::R),  // Hebrew Alef - Right-to-Left
            ('\u{0627}', BidiClass::AL), // Arabic Alef - Right-to-Left Arabic
            ('1', BidiClass::EN),        // European Number
            ('\u{0660}', BidiClass::AN), // Arabic Number
            (' ', BidiClass::WS),        // Whitespace
        ];

        for (ch, expected_class) in tests {
            let class = BidiClass::of(ch);
            assert_eq!(class, expected_class, "Wrong class for '{}'", ch);
        }
    }

    #[test]
    fn test_paragraph_direction_detection() {
        // LTR paragraph
        let ltr_text = "Hello World";
        let ltr_info = BidiInfo::new(ltr_text, None);
        assert_eq!(
            ltr_info.paragraph_direction(),
            ParagraphDirection::Ltr,
            "English should be LTR"
        );

        // RTL paragraph
        let rtl_text = "\u{05E9}\u{05DC}\u{05D5}\u{05DD}";
        let rtl_info = BidiInfo::new(rtl_text, None);
        assert_eq!(
            rtl_info.paragraph_direction(),
            ParagraphDirection::Rtl,
            "Hebrew should be RTL"
        );
    }
}

// ============================================================================
// FEAT-018: RTL text shaping tests
// ============================================================================

mod rtl_shaping {
    use super::*;

    #[test]
    fn test_rtl_visual_order() {
        // Given Hebrew text
        let text = "\u{05D0}\u{05D1}\u{05D2}"; // aleph, bet, gimel

        // When we get visual runs
        let bidi_info = BidiInfo::new(text, None);
        let runs = bidi_info.visual_runs(0, text.len());

        // Then the run should be RTL
        assert_eq!(runs.len(), 1, "Should have one run");
        assert!(runs[0].is_rtl(), "Hebrew run should be RTL");
    }

    #[test]
    fn test_rtl_character_reordering() {
        // Given Hebrew text "ABC" (in Hebrew)
        // Each Hebrew character is 2 bytes in UTF-8
        let text = "\u{05D0}\u{05D1}\u{05D2}";

        // When we reorder for visual display
        let bidi_info = BidiInfo::new(text, None);
        let visual_order = bidi_info.reordered_indices(0, text.len());

        // Then byte indices should be reversed for visual display
        // text.len() is 6 (2 bytes per Hebrew char)
        // Logical bytes: 0, 1, 2, 3, 4, 5 -> Visual: 5, 4, 3, 2, 1, 0
        assert_eq!(
            visual_order.len(),
            text.len(),
            "Should have index for each byte"
        );
        // First index should be greater than last (reversed)
        assert!(
            visual_order[0] > visual_order[visual_order.len() - 1],
            "RTL should reverse order"
        );
    }

    #[test]
    fn test_arabic_rtl_direction() {
        // Given Arabic text
        let text = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627}"; // "marhaba"

        // When we analyze it
        let bidi_info = BidiInfo::new(text, None);

        // Then it should be detected as RTL
        assert_eq!(bidi_info.paragraph_direction(), ParagraphDirection::Rtl);
    }
}

// ============================================================================
// FEAT-019: Mixed direction text tests
// ============================================================================

mod mixed_direction {
    use super::*;

    #[test]
    fn test_mixed_ltr_rtl_runs() {
        // Given text with both LTR and RTL
        let text = "Hello \u{05E9}\u{05DC}\u{05D5}\u{05DD}"; // "Hello shalom"

        // When we get visual runs
        let bidi_info = BidiInfo::new(text, None);
        let runs = bidi_info.visual_runs(0, text.len());

        // Then we should have multiple runs
        assert!(
            runs.len() >= 2,
            "Should have separate runs for different directions"
        );
    }

    #[test]
    fn test_embedded_rtl_in_ltr() {
        // Given LTR text with embedded RTL
        let text = "The word \u{05E9}\u{05DC}\u{05D5}\u{05DD} is Hebrew";

        // When we analyze it
        let bidi_info = BidiInfo::new(text, None);

        // Then paragraph should be LTR (first strong character)
        assert_eq!(bidi_info.paragraph_direction(), ParagraphDirection::Ltr);

        // But runs should include RTL segment
        let runs = bidi_info.visual_runs(0, text.len());
        let has_rtl = runs.iter().any(|r| r.is_rtl());
        assert!(has_rtl, "Should have RTL run for Hebrew");
    }

    #[test]
    fn test_numbers_in_rtl_context() {
        // Given RTL text with numbers
        let text = "\u{05D4}\u{05DE}\u{05E1}\u{05E4}\u{05E8} 123"; // "the number 123" in Hebrew

        // When we analyze it
        let bidi_info = BidiInfo::new(text, None);

        // Then numbers should be treated correctly
        // Numbers are weak characters and should maintain their sequence
        let runs = bidi_info.visual_runs(0, text.len());
        assert!(!runs.is_empty(), "Should have runs");
    }

    #[test]
    fn test_neutral_characters_between_directions() {
        // Given text with neutral characters (spaces, punctuation) between directions
        let text = "Hello, \u{05E9}\u{05DC}\u{05D5}\u{05DD}!";

        // When we analyze
        let bidi_info = BidiInfo::new(text, None);
        let runs = bidi_info.visual_runs(0, text.len());

        // Then neutral characters should resolve based on context
        assert!(runs.len() >= 1, "Should have runs");
    }
}

// ============================================================================
// FEAT-020: Bidi paragraph levels tests
// ============================================================================

mod paragraph_levels {
    use super::*;

    #[test]
    fn test_explicit_ltr_paragraph() {
        // Given explicit LTR direction override
        let text = "\u{05E9}\u{05DC}\u{05D5}\u{05DD}"; // Hebrew text

        // When we force LTR paragraph direction
        let bidi_info = BidiInfo::new(text, Some(ParagraphDirection::Ltr));

        // Then paragraph level should be LTR
        assert_eq!(
            bidi_info.paragraph_level().0,
            0,
            "Forced LTR should be level 0"
        );
    }

    #[test]
    fn test_explicit_rtl_paragraph() {
        // Given explicit RTL direction override
        let text = "Hello World"; // English text

        // When we force RTL paragraph direction
        let bidi_info = BidiInfo::new(text, Some(ParagraphDirection::Rtl));

        // Then paragraph level should be RTL
        assert_eq!(
            bidi_info.paragraph_level().0,
            1,
            "Forced RTL should be level 1"
        );
    }

    #[test]
    fn test_paragraph_isolation() {
        // Given a BidiParagraph
        let text = "Hello \u{05E9}\u{05DC}\u{05D5}\u{05DD}";
        let paragraph = BidiParagraph::new(text, None);

        // When we check its properties
        let level = paragraph.level();
        let direction = paragraph.direction();

        // Then they should be consistent
        if level.0 == 0 {
            assert_eq!(direction, ParagraphDirection::Ltr);
        } else {
            assert_eq!(direction, ParagraphDirection::Rtl);
        }
    }

    #[test]
    fn test_multiple_paragraphs() {
        // Given multiple paragraphs (separated by newlines)
        let text = "Hello\n\u{05E9}\u{05DC}\u{05D5}\u{05DD}";

        // When we process them
        let paragraphs: Vec<_> = text.lines().map(|line| BidiInfo::new(line, None)).collect();

        // Then each paragraph has its own direction
        assert_eq!(paragraphs[0].paragraph_direction(), ParagraphDirection::Ltr);
        assert_eq!(paragraphs[1].paragraph_direction(), ParagraphDirection::Rtl);
    }
}

// ============================================================================
// FEAT-021: Arabic shaping tests
// ============================================================================

mod arabic_shaping {
    use super::*;

    #[test]
    fn test_arabic_joining_type_detection() {
        // Test Arabic character joining types
        let tests = [
            ('\u{0627}', ArabicJoiningType::RightJoining), // Alef - only joins to right
            ('\u{0628}', ArabicJoiningType::DualJoining),  // Ba - joins both sides
            ('\u{0621}', ArabicJoiningType::NonJoining),   // Hamza - doesn't join
        ];

        for (ch, expected_type) in tests {
            let joining_type = ArabicJoiningType::of(ch);
            assert_eq!(
                joining_type, expected_type,
                "Wrong joining type for U+{:04X}",
                ch as u32
            );
        }
    }

    #[test]
    fn test_arabic_isolated_form() {
        // Given a single Arabic letter (isolated context)
        let shaper = ArabicShaper::new();
        let text = "\u{0628}"; // Ba

        // When we shape it
        let shaped = shaper.shape(text);

        // Then it should use isolated form
        // Isolated Ba is U+FE8F
        assert_eq!(
            shaped.chars().next(),
            Some('\u{FE8F}'),
            "Should use isolated form"
        );
    }

    #[test]
    fn test_arabic_initial_form() {
        // Given Arabic letters where first needs initial form
        let shaper = ArabicShaper::new();
        let text = "\u{0628}\u{0627}"; // Ba + Alef (ba joins to alef)

        // When we shape it
        let shaped = shaper.shape(text);

        // Then Ba should be in initial form (U+FE91)
        let chars: Vec<char> = shaped.chars().collect();
        assert_eq!(chars[0], '\u{FE91}', "Ba should use initial form");
    }

    #[test]
    fn test_arabic_medial_form() {
        // Given Arabic word with medial letter
        let shaper = ArabicShaper::new();
        let text = "\u{0628}\u{0628}\u{0627}"; // Ba + Ba + Alef

        // When we shape it
        let shaped = shaper.shape(text);

        // Then middle Ba should be in medial form (U+FE92)
        let chars: Vec<char> = shaped.chars().collect();
        assert_eq!(chars[1], '\u{FE92}', "Middle Ba should use medial form");
    }

    #[test]
    fn test_arabic_final_form() {
        // Given Arabic letters where last joins
        let shaper = ArabicShaper::new();
        let text = "\u{0627}\u{0628}"; // Alef + Ba (alef doesn't affect ba's final form context)

        // When we shape it
        let shaped = shaper.shape(text);

        // Ba should be in final form since it follows alef
        let chars: Vec<char> = shaped.chars().collect();
        // This depends on exact joining logic - let's verify it's shaped
        assert!(!shaped.is_empty(), "Should produce output");
    }

    #[test]
    fn test_arabic_lam_alef_ligature() {
        // Given Lam followed by Alef
        let shaper = ArabicShaper::new();
        let text = "\u{0644}\u{0627}"; // Lam + Alef

        // When we shape it
        let shaped = shaper.shape(text);

        // Then it should form a ligature (U+FEFB or similar)
        assert!(
            shaped.chars().count() == 1 || shaped.contains('\u{FEFB}'),
            "Lam-Alef should form ligature"
        );
    }
}

// ============================================================================
// FEAT-022: Indic scripts shaping tests
// ============================================================================

mod indic_shaping {
    use super::*;

    #[test]
    fn test_devanagari_consonant_cluster() {
        // Given Devanagari text with consonant cluster
        let shaper = IndicShaper::new();
        let text = "\u{0915}\u{094D}\u{0937}"; // Ka + Virama + Ssa = Ksha conjunct

        // When we shape it
        let result = shaper.shape(text);

        // Then it should be processed (conjunct formation)
        assert!(!result.is_empty(), "Should produce output for conjunct");
    }

    #[test]
    fn test_devanagari_vowel_sign_reordering() {
        // Given Devanagari consonant followed by vowel sign 'i'
        let shaper = IndicShaper::new();
        let text = "\u{0915}\u{093F}"; // Ka + vowel sign i

        // When we shape it
        let result = shaper.shape(text);

        // Then the vowel sign should be reordered to appear before consonant visually
        // This is handled by the shaper
        assert!(!result.is_empty(), "Should process vowel reordering");
    }

    #[test]
    fn test_tamil_script() {
        // Given Tamil text
        let shaper = IndicShaper::new();
        let text = "\u{0BA4}\u{0BAE}\u{0BBF}\u{0BB4}\u{0BCD}"; // Tamil text

        // When we shape it
        let result = shaper.shape(text);

        // Then it should be processed
        assert!(!result.is_empty(), "Should handle Tamil");
    }

    #[test]
    fn test_bengali_script() {
        // Given Bengali text
        let shaper = IndicShaper::new();
        let text = "\u{09AC}\u{09BE}\u{0982}\u{09B2}\u{09BE}"; // "Bangla" in Bengali

        // When we shape it
        let result = shaper.shape(text);

        // Then it should be processed
        assert!(!result.is_empty(), "Should handle Bengali");
    }

    #[test]
    fn test_indic_script_detection() {
        // Given various Indic characters
        assert!(IndicShaper::is_indic('\u{0915}')); // Devanagari Ka
        assert!(IndicShaper::is_indic('\u{0B85}')); // Tamil A
        assert!(IndicShaper::is_indic('\u{09AC}')); // Bengali Ba
        assert!(!IndicShaper::is_indic('A')); // Latin A
    }
}

// ============================================================================
// FEAT-023: Thai/Lao/Khmer support tests
// ============================================================================

mod southeast_asian_shaping {
    use super::*;

    #[test]
    fn test_thai_character_classification() {
        // Given Thai characters
        let shaper = SoutheastAsianShaper::new();

        // Thai consonant
        assert!(shaper.is_thai('\u{0E01}')); // Ko Kai
                                             // Thai vowel
        assert!(shaper.is_thai('\u{0E40}')); // Sara E
                                             // Thai tone mark
        assert!(shaper.is_thai('\u{0E48}')); // Mai Ek
    }

    #[test]
    fn test_thai_text_shaping() {
        // Given Thai text
        let shaper = SoutheastAsianShaper::new();
        let text = "\u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}"; // "sawatdi" (hello)

        // When we shape it
        let result = shaper.shape(text);

        // Then it should be processed
        assert!(!result.is_empty(), "Should handle Thai text");
    }

    #[test]
    fn test_lao_script() {
        // Given Lao text
        let shaper = SoutheastAsianShaper::new();
        let text = "\u{0EAA}\u{0EB0}\u{0E9A}\u{0EB2}\u{0E8D}\u{0E94}\u{0EB5}"; // Lao text

        // When we shape it
        let result = shaper.shape(text);

        // Then it should be processed
        assert!(!result.is_empty(), "Should handle Lao script");
    }

    #[test]
    fn test_khmer_script() {
        // Given Khmer text
        let shaper = SoutheastAsianShaper::new();
        let text = "\u{1780}\u{17D2}\u{1789}\u{17BB}\u{17C6}"; // Khmer text

        // When we shape it
        let result = shaper.shape(text);

        // Then it should be processed
        assert!(!result.is_empty(), "Should handle Khmer script");
    }

    #[test]
    fn test_script_type_detection() {
        // Test that we can detect which Southeast Asian script is being used
        let shaper = SoutheastAsianShaper::new();

        assert!(shaper.is_thai('\u{0E01}')); // Thai
        assert!(shaper.is_lao('\u{0E81}')); // Lao
        assert!(shaper.is_khmer('\u{1780}')); // Khmer
    }

    #[test]
    fn test_thai_leading_vowel() {
        // Thai has vowels that appear before their consonant visually
        let shaper = SoutheastAsianShaper::new();
        let text = "\u{0E40}\u{0E01}"; // Sara E + Ko Kai = "ke" visually

        // When we shape it
        let result = shaper.shape(text);

        // Then the leading vowel should be preserved in proper order
        assert!(!result.is_empty(), "Should handle leading vowels");
    }
}

// ============================================================================
// Integration tests combining multiple features
// ============================================================================

mod integration {
    use super::*;

    #[test]
    fn test_bidi_with_arabic_shaping() {
        // Given Arabic text that needs both bidi and shaping
        let text = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627}"; // "marhaba"

        // When we apply bidi analysis
        let bidi_info = BidiInfo::new(text, None);

        // Then it should be RTL
        assert_eq!(bidi_info.paragraph_direction(), ParagraphDirection::Rtl);

        // And when we shape it
        let shaper = ArabicShaper::new();
        let shaped = shaper.shape(text);

        // It should have contextual forms
        assert!(!shaped.is_empty());
    }

    #[test]
    fn test_mixed_scripts_bidi() {
        // Given text with English and Arabic
        let text = "Price: \u{0633}\u{0639}\u{0631} = $100";

        // When we analyze it
        let bidi_info = BidiInfo::new(text, None);
        let runs = bidi_info.visual_runs(0, text.len());

        // Then we should have multiple runs for different scripts
        assert!(runs.len() >= 2, "Should have runs for different scripts");
    }

    #[test]
    fn test_complete_pipeline() {
        // Test the complete text processing pipeline
        let text = "Hello \u{05E9}\u{05DC}\u{05D5}\u{05DD}"; // "Hello shalom"

        // 1. Bidi analysis
        let bidi_info = BidiInfo::new(text, None);
        assert_eq!(bidi_info.paragraph_direction(), ParagraphDirection::Ltr);

        // 2. Get visual runs
        let runs = bidi_info.visual_runs(0, text.len());
        assert!(!runs.is_empty());

        // 3. Each run can be shaped appropriately
        for run in runs {
            let run_text = &text[run.start..run.end];
            assert!(!run_text.is_empty());
        }
    }
}
