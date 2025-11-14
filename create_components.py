#!/usr/bin/env python3
"""
Create all font system components with proper structure.
"""
import os
import json
from pathlib import Path

# Project root
PROJECT_ROOT = Path("/home/user/Corten-FontSystem")
PROJECT_VERSION = "0.1.0"

# Component definitions
COMPONENTS = [
    {
        "name": "font_types",
        "type": "library",
        "responsibility": "Common types, traits, enums, and interfaces for the font system",
        "tech_stack": "Rust",
        "dependencies": [],
        "estimated_tokens": 8000
    },
    {
        "name": "font_parser",
        "type": "library",
        "responsibility": "Parse OpenType, TrueType, WOFF, and WOFF2 font files",
        "tech_stack": "Rust, ttf-parser, byteorder",
        "dependencies": ["font_types"],
        "estimated_tokens": 24000
    },
    {
        "name": "font_registry",
        "type": "library",
        "responsibility": "Font discovery, loading, caching, and font matching algorithms",
        "tech_stack": "Rust, fontconfig (Linux)",
        "dependencies": ["font_types"],
        "estimated_tokens": 22000
    },
    {
        "name": "text_shaper",
        "type": "library",
        "responsibility": "Text shaping, bidirectional text, line breaking, and OpenType features",
        "tech_stack": "Rust, harfbuzz_rs (initial), unicode-bidi",
        "dependencies": ["font_types", "font_parser", "font_registry"],
        "estimated_tokens": 28000
    },
    {
        "name": "glyph_renderer",
        "type": "library",
        "responsibility": "Glyph rasterization, hinting, subpixel rendering, and glyph caching",
        "tech_stack": "Rust, freetype-rs (initial)",
        "dependencies": ["font_types", "font_parser"],
        "estimated_tokens": 22000
    },
    {
        "name": "platform_integration",
        "type": "library",
        "responsibility": "Platform-specific font discovery (Linux, Windows, macOS)",
        "tech_stack": "Rust, fontconfig, dwrote, core-text",
        "dependencies": ["font_types", "font_registry"],
        "estimated_tokens": 12000
    },
    {
        "name": "font_system_api",
        "type": "application",
        "responsibility": "Public API and orchestration layer for the complete font system",
        "tech_stack": "Rust",
        "dependencies": ["font_types", "font_parser", "font_registry", "text_shaper", "glyph_renderer", "platform_integration"],
        "estimated_tokens": 14000
    }
]

def create_component_dirs(component_name):
    """Create component directory structure."""
    base_path = PROJECT_ROOT / "components" / component_name

    dirs = [
        base_path / "src",
        base_path / "tests" / "unit",
        base_path / "tests" / "integration",
        base_path / "tests" / "contracts",
        base_path / "benches"
    ]

    for dir_path in dirs:
        dir_path.mkdir(parents=True, exist_ok=True)

    return base_path

def create_cargo_toml(component_path, component):
    """Create Cargo.toml for Rust component."""
    cargo_toml = f"""[package]
name = "{component['name']}"
version = "0.1.0"
edition = "2021"
authors = ["CortenBrowser Team"]
license = "MIT OR Apache-2.0"
description = "{component['responsibility']}"

[dependencies]
# Add dependencies based on component needs

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
test-case = "3.1"

[lib]
name = "{component['name']}"
path = "src/lib.rs"

[[bench]]
name = "benchmarks"
harness = false
"""

    with open(component_path / "Cargo.toml", "w") as f:
        f.write(cargo_toml)

def create_lib_rs(component_path, component):
    """Create src/lib.rs stub."""
    lib_rs = f"""//! {component['name']} - {component['responsibility']}

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Module exports will be added during implementation
pub mod types;

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_placeholder() {{
        // Tests will be added during TDD
        assert!(true);
    }}
}}
"""

    with open(component_path / "src" / "lib.rs", "w") as f:
        f.write(lib_rs)

    # Create types.rs stub
    types_rs = f"""//! Common types for {component['name']}

// Type definitions will be added during implementation
"""

    with open(component_path / "src" / "types.rs", "w") as f:
        f.write(types_rs)

def create_readme(component_path, component):
    """Create README.md."""
    readme = f"""# {component['name']}

**Type**: {component['type']}
**Tech Stack**: {component['tech_stack']}
**Version**: {PROJECT_VERSION}

## Responsibility

{component['responsibility']}

## Structure

```
├── src/           # Source code
├── tests/         # Tests (unit, integration, contracts)
├── benches/       # Benchmarks
├── Cargo.toml     # Rust package manifest
├── CLAUDE.md      # Component-specific instructions for Claude Code
└── README.md      # This file
```

## Dependencies

{chr(10).join(f"- {dep}" for dep in component['dependencies']) if component['dependencies'] else "None (base component)"}

## Usage

This component is ready for immediate use via Task tool orchestration.

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

### Build and Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code
cargo clippy

# Format code
cargo fmt
```

## Architecture

Implementation details will be added during development following the specifications in `/home/user/Corten-FontSystem/font-system-specification.md`.
"""

    with open(component_path / "README.md", "w") as f:
        f.write(readme)

def create_component_yaml(component_path, component):
    """Create component.yaml manifest."""
    manifest = {
        "name": component['name'],
        "version": PROJECT_VERSION,
        "type": component['type'],
        "language": "rust",
        "tech_stack": component['tech_stack'],
        "responsibility": component['responsibility'],
        "estimated_tokens": component['estimated_tokens'],
        "dependencies": {
            "imports": [
                {
                    "name": dep,
                    "version": "^0.1.0",
                    "import_from": f"components.{dep}"
                }
                for dep in component['dependencies']
            ]
        },
        "exports": {
            "module": f"components.{component['name']}",
            "public_api": []
        }
    }

    with open(component_path / "component.yaml", "w") as f:
        import yaml
        yaml.safe_dump(manifest, f, default_flow_style=False)

def create_claude_md(component_path, component):
    """Create CLAUDE.md from template."""
    template_path = PROJECT_ROOT / "claude-orchestration-system" / "templates" / "component-generic.md"

    with open(template_path, "r") as f:
        template = f.read()

    # Perform substitutions
    claude_md = template.replace("{{COMPONENT_NAME}}", component['name'])
    claude_md = claude_md.replace("{{TECH_STACK}}", component['tech_stack'])
    claude_md = claude_md.replace("{{CURRENT_TOKENS}}", "0")
    claude_md = claude_md.replace("{{COMPONENT_RESPONSIBILITY}}", component['responsibility'])
    claude_md = claude_md.replace("{{PROJECT_VERSION}}", PROJECT_VERSION)
    claude_md = claude_md.replace("{{STYLE_GUIDE}}", "Rust Style Guide")
    claude_md = claude_md.replace("{{FORMATTER}}", "cargo fmt")
    claude_md = claude_md.replace("{{LINTER}}", "cargo clippy")
    claude_md = claude_md.replace("{{LINT_COMMAND}}", "cargo clippy")
    claude_md = claude_md.replace("{{ADDITIONAL_INSTRUCTIONS}}", f"""
## Rust-Specific Instructions

### Code Organization
- Use modules for logical grouping
- Keep public API surface small
- Use re-exports in lib.rs for convenience
- Document all public items

### Error Handling
- Use Result<T, E> for fallible operations
- Define custom error types with thiserror
- Provide helpful error messages

### Testing
- Unit tests in same file as code (#[cfg(test)] mod tests)
- Integration tests in tests/ directory
- Use cargo-tarpaulin for coverage

### Performance
- Profile with cargo-flamegraph
- Benchmark with criterion
- Use cargo-bench for regression detection

### Unsafe Code
- Avoid unsafe unless absolutely necessary
- Document why unsafe is needed
- Ensure memory safety invariants

## Specification Reference

Full specification: `/home/user/Corten-FontSystem/font-system-specification.md`

Component sections:
{_get_spec_sections(component['name'])}
""")

    with open(component_path / "CLAUDE.md", "w") as f:
        f.write(claude_md)

def _get_spec_sections(component_name):
    """Get relevant specification sections for component."""
    sections = {
        "font_types": "- Core Types (lines 91-235)\n- Public API Specification (lines 237-295)",
        "font_parser": "- Phase 2: Font Parser Implementation (lines 387-438)\n- OpenType Parser (lines 390-438)",
        "font_registry": "- Font Registry (lines 42-46)\n- Font Matching (lines 47)",
        "text_shaper": "- Phase 3: Text Shaping Engine (lines 440-505)\n- Harfbuzz Integration (lines 357-385)",
        "glyph_renderer": "- Phase 4: Rasterization Engine (lines 507-580)",
        "platform_integration": "- Phase 5: Platform Integration (lines 582-628)",
        "font_system_api": "- Main API Interface (lines 238-295)\n- Browser Integration Interface (lines 297-354)"
    }
    return sections.get(component_name, "- See full specification")

def main():
    """Create all components."""
    print("Creating Font System components...")

    for component in COMPONENTS:
        print(f"\n Creating component: {component['name']}")

        # Create directories
        component_path = create_component_dirs(component['name'])
        print(f"  ✓ Created directory structure")

        # Create files
        create_cargo_toml(component_path, component)
        print(f"  ✓ Created Cargo.toml")

        create_lib_rs(component_path, component)
        print(f"  ✓ Created src/lib.rs")

        create_readme(component_path, component)
        print(f"  ✓ Created README.md")

        try:
            create_component_yaml(component_path, component)
            print(f"  ✓ Created component.yaml")
        except ImportError:
            print(f"  ⚠ Skipped component.yaml (PyYAML not installed)")

        create_claude_md(component_path, component)
        print(f"  ✓ Created CLAUDE.md")

    print("\n✅ All components created successfully!")
    print(f"\nCreated components:")
    for component in COMPONENTS:
        print(f"  - {component['name']} ({component['estimated_tokens']} est. tokens)")

if __name__ == "__main__":
    main()
