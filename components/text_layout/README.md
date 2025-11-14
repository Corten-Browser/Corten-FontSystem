# Text Layout Component

**Type**: Feature
**Tech Stack**: Rust, unicode-bidi, unicode-linebreak, unicode-segmentation
**Version**: 0.1.0

## Responsibility

This component provides high-level text layout capabilities including:
- Paragraph layout (multi-line text rendering)
- Line breaking algorithms (Unicode UAX #14)
- Text justification (left, right, center, justify)
- Vertical text layout (for CJK languages)
- Text wrapping and reflow

## Architecture

```
text_layout/
├── src/
│   ├── lib.rs              // Public API exports
│   ├── paragraph.rs        // Paragraph layout engine
│   ├── line_breaker.rs     // Line breaking algorithm (UAX #14)
│   ├── justification.rs    // Text justification logic
│   ├── vertical.rs         // Vertical text support (CJK)
│   └── types.rs            // Layout-specific types
├── tests/
│   ├── unit/               // Unit tests
│   └── integration/        // Integration tests
└── benches/                // Performance benchmarks
```

## Public API

### Core Types

- `ParagraphLayout` - Main layout engine
- `LayoutOptions` - Configuration for layout behavior
- `LineBreakIterator` - Iterator for line break opportunities
- `JustificationMode` - Alignment and justification modes
- `LayoutResult` - Result of layout operation with positioned text runs

### Usage Example

```rust
use text_layout::{ParagraphLayout, LayoutOptions, JustificationMode};

let options = LayoutOptions {
    max_width: 500.0,
    justification: JustificationMode::Justify,
    ..Default::default()
};

let layout = ParagraphLayout::new();
let result = layout.layout_paragraph(shaped_text, &options)?;

for line in result.lines {
    // Render each line
}
```

## Dependencies

- `font_types` - Shared font types
- `text_shaper` - Text shaping for individual runs
- `unicode-bidi` - Bidirectional text algorithm
- `unicode-linebreak` - Line breaking algorithm (UAX #14)
- `unicode-segmentation` - Text segmentation

## Implementation Notes

### Line Breaking Algorithm

Implements Unicode UAX #14 (Line Breaking Properties) for proper line break opportunities:
- Break opportunities based on Unicode properties
- Handles various scripts (Latin, CJK, Arabic, etc.)
- Respects non-breaking spaces and word boundaries

### Justification

Supports multiple justification modes:
- **Left**: Align text to left edge
- **Right**: Align text to right edge
- **Center**: Center text
- **Justify**: Distribute space evenly across line

### Vertical Text

Supports vertical text layout for CJK languages:
- Top-to-bottom text flow
- Right-to-left line progression
- Proper glyph rotation

## Testing

- Unit tests for each algorithm
- Integration tests with text_shaper
- UAX #14 test suite compliance
- Visual regression tests for justification

## Performance Targets

- Paragraph layout: < 5ms for 1000 characters
- Line breaking: < 1ms for 1000 characters
- Memory: < 10KB per paragraph

## Related Components

- `text_shaper` - Provides shaped text runs
- `font_system_api` - Uses layout for complete rendering
