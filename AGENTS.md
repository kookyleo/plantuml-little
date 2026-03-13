# plantuml-little — Agent Guidelines

## Project Identity

**plantuml-little** is a lightweight Rust port of PlantUML, targeting a single use case: `.puml` → `.svg` conversion, delivered as a library + CLI tool.

**Current phase**: Maintenance — initial development complete, now in upstream tracking + quality optimization phase.

## Hard Boundaries

### In Scope
- Parse PlantUML text (`.puml` files)
- Render to SVG only
- Library crate (`plantuml-little`) + binary crate
- Graphviz layout via `vizoxide` (Class / State / Component / ERD / UseCase)
- Self-contained layout for other diagram types

### Out of Scope — Do NOT implement
- Any output format other than SVG (no PNG, PDF, EPS, HTML5, ASCII, etc.)
- GUI, web server (Picoweb), FTP server, pipe mode
- PlantUML Server URL encoding/transcoding
- Security sandbox / profile system
- ELK layout engine
- TeaVM / JS compilation
- PNG metadata embedding/extraction

## Architecture

```
puml text → preprocess → parse → layout → render → SVG string
```

### Module Structure

```
src/
├── lib.rs              # Library entry point: convert() pipeline
├── main.rs             # CLI binary (clap)
├── preproc/            # Preprocessor (variables, functions, includes, themes)
│   ├── mod.rs          # Core: directive dispatch, variable expansion, conditionals
│   ├── builtins.rs     # Color parsing, date formatting, theme/stdlib listing
│   ├── expr.rs         # Expression evaluation, arithmetic, string utilities
│   └── include.rs      # File resolution, archive extraction, subpart parsing
├── style.rs            # Skinparam parsing + theme engine
├── error.rs            # Error types with line/column tracking
├── text.rs             # CJK/Unicode text width calculation
├── parser/             # PlantUML text → Diagram IR
│   ├── mod.rs          # Dispatcher (detect type → delegate)
│   ├── common.rs       # Shared: block extraction, type detection, meta, sprites
│   ├── creole.rs       # Creole markup → RichText/TextSpan
│   ├── class.rs, sequence.rs, activity.rs, state.rs, ...
│   └── json_diagram.rs, yaml_diagram.rs, ditaa.rs, salt.rs, ...
├── model/              # Internal representation
│   ├── diagram.rs      # Diagram enum (17 variants)
│   ├── entity.rs       # Class/interface/component entities
│   ├── link.rs         # Relationships
│   ├── richtext.rs     # TextSpan/RichText (Creole model)
│   ├── sequence.rs, activity.rs, state.rs, ...
│   └── hyperlink.rs
├── layout/             # Positioning engines
│   ├── mod.rs          # Dispatcher
│   ├── graphviz.rs     # vizoxide integration (DOT → positions)
│   ├── sequence.rs, activity.rs, ...
│   └── [per-diagram-type].rs
└── render/             # IR + Layout → SVG
    ├── svg.rs          # Class diagram SVG + shared utilities
    ├── svg_richtext.rs # Creole → SVG tspan rendering + sprite
    ├── svg_hyperlink.rs
    ├── svg_sequence.rs, svg_activity.rs, ...
    └── [per-diagram-type].rs
```

## Maintenance Guidelines

### Upstream Tracking

PlantUML upstream repository: `https://github.com/plantuml/plantuml`

Focus areas:
1. **New diagram types** -- evaluate for inclusion
2. **Syntax changes** -- extensions to existing diagram types
3. **Preprocessor enhancements** -- new built-in functions, new directives
4. **stdlib updates** -- vendor content updates as needed
5. **Regression tests** -- extract new upstream test cases as fixtures

### Code Quality

Known optimization targets (non-blocking):
- `write!().unwrap()` never actually fails on String writes, but style is inconsistent
- clippy pedantic level has ~508 hints (mostly documentation/annotation/precision cast style warnings)

### Test Strategy

- Extract `.puml` fixtures from upstream Java tests
- SVG output: structural comparison (element presence, attribute values), not byte-for-byte matching
- Integration tests: `.puml` → `.svg`, verify valid SVG output
- Unit tests: within each parser/layout/render module
- **Current**: 1,502 tests, 296 fixtures, 0 ignored, 0 failures

### Code Style

- License: Multi-licensed (GPL-3.0 / LGPL-3.0 / Apache-2.0 / EPL-2.0 / MIT), following upstream PlantUML
- Git messages: English, concise, no AI tool mentions
- Author: `kookyleo <kookyleo@gmail.com>`
- Prefer `thiserror` for error types, `clap` for CLI
- Minimize dependencies -- only add crates when clearly justified
- Prefer editing existing files over creating new ones

### Parallel Agent Work

File ownership matrix (enforced during parallel agent work):
- Each agent only modifies its assigned files
- parser/layout/render naturally isolated by diagram type
- Shared files (lib.rs, mod.rs, diagram.rs) modified sequentially by the main thread
