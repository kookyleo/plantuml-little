# plantuml-little — Agent Guidelines

## Project Identity

**plantuml-little** is a lightweight Rust port of PlantUML, targeting a single use case: `.puml` → `.svg` conversion, delivered as a library + CLI tool.

**Current phase**: Upstream alignment — SVG output must be byte-identical to Java PlantUML.

## Hard Boundaries

### In Scope
- Parse PlantUML text (`.puml` files)
- Render to SVG only
- Library crate (`plantuml-little`) + binary crate
- Graphviz layout via system `dot` command (`dot -Tsvg`, parsed for coordinates)
- Self-contained layout for other diagram types

### Out of Scope — Do NOT implement
- Any output format other than SVG (no PNG, PDF, EPS, HTML5, ASCII, etc.)
- GUI, web server (Picoweb), FTP server, pipe mode
- PlantUML Server URL encoding/transcoding
- Security sandbox / profile system
- ELK layout engine
- TeaVM / JS compilation
- PNG metadata embedding/extraction
- **DITAA** (@startditaa): Java delegates to a third-party bitmap-only library
  (ASCII art → BufferedImage). No SVG output exists. Would require building a
  full ASCII art → SVG renderer from scratch.
- **JCCKIT** (@startjcckit): Java AWT charting library, raster-only, no SVG
  mode, extremely low adoption.

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
├── style.rs            # Skinparam parsing + theme engine
├── parser/             # PlantUML text → Diagram IR
├── model/              # Internal representation
├── layout/             # Positioning engines
│   ├── graphviz.rs     # vizoxide integration (DOT → positions)
│   └── [per-type].rs
├── klimt/              # Low-level SVG graphics abstraction
│   └── svg.rs          # SvgGraphic, ensure_visible, font rendering
├── svek/               # Graphviz coordinate extraction pipeline
└── render/             # IR + Layout → SVG
    ├── svg.rs          # Shared: BoundsTracker, wrap_with_meta, ensure_visible_int
    ├── svg_richtext.rs # Creole → SVG text rendering + sprite
    └── [per-type].rs
```

## Upstream Alignment — The Iron Rule

**Goal: SVG output must be byte-identical to Java PlantUML.**

### Core Principles

1. **Java-first TDD**: Never fit numbers. Read the Java source code and use the exact same algorithms and parameters. No shortcuts, no approximations. A single bit of difference will fail the reference test.

2. **Depth-first**: 深入对照每一个底层模块，确保一致。不要选择最简单的，选择最深入的。顺着依赖关系递归深入，子孙问题拆小，达成两端一致，再逐层返回。必要时 ultrathink。

3. **Forward-fix, not revert**: 不要一遇到回退就 git checkout 回滚。只要大方向是向 Java 靠拢的，就往前推进——分析回退原因，修复级联问题，而非放弃整个改动。回滚是最后手段，不是第一反应。

4. **No fear of deep changes**: 不要畏惧深层架构变更。将问题分解为可验证的子步骤，每步都用 Java 源码和参考输出验证。

### Execution Discipline — The Debugging & Fixing Loop

Every fix must follow this strict loop. No skipping steps, no guessing.

1. **Enumerate sub-items.** Run `scripts/analyze_failures.py` to get the full failure taxonomy. Focus on **common root causes** that affect many tests, not individual test symptoms.

2. **Pick the right target.** Selection criteria (in order):
   - **Prefer tests with matching structure** — same SVG elements in roughly the same positions.
   - **Work on shared root causes, not individual tests.** If 20 tests fail because of a common constant, fix the constant.
   - State the target precisely: "make `foo.svg` match — currently differs at height (95 vs 100), root cause: member row height uses 14pt but skinparam sets 12pt."

3. **Validate constants.** Before trusting any constant, cross-check against Java output. Run through Java (`java -jar plantuml.jar -tsvg`), extract the value, confirm it matches.

4. **Trace both chains.** For the chosen test:
   - **Rust chain**: trace from `.puml` input to the differing SVG byte. Record every intermediate value.
   - **Java chain**: instrument Java with `System.err.println` to capture precise intermediate values (gold standard), or reverse-compute from reference SVG.

5. **Diff at divergence point.** Compare Rust vs Java intermediate values. Fix structural differences first, then parameter differences.

6. **Pre-check defaults.** Before committing, verify identical output for default skinparams (no overrides). Most passing tests use defaults.

7. **Fix at the source.** Apply the minimal change at the divergence point. Never patch downstream.

8. **Verify & iterate.** Run `cargo test --lib` (no regression) + `cargo test --test reference_tests` (track pass count). Each loop should either increase pass count or eliminate one dimension of difference.

### Key Java Source Files

| File | Purpose |
|------|---------|
| `SvgGraphics.java` | SVG generation, `ensureVisible()` (`(int)(x+1)` truncation) |
| `SvekResult.java` | `calculateDimension()`, `moveDelta(6, 6)`, `delta(15, 15)` |
| `LimitFinder.java` | Bounding box: `drawRectangle` adds `(x-1, y-1)` to `(x+w-1, y+h-1)` |
| `TextBlockExporter12026.java` | `calculateFinalDimension()`, document margin (R=5, B=5) |
| `DotStringFactory.java` | Graphviz params: `nodesep`, `ranksep`, `minRankSep` |
| `SvekNode.java` | Node position: `moveDelta`, `getMinX/getMinY` |
| `TileBuilder.java` | Teoz tile construction, note-on-message decorator pattern |
| `ParticipantBox.java` | `outMargin=5`, participant width model |
| `ComponentRoseNote.java` | Note `paddingX=5`, `marginX1=6`, `marginX2=15` |

### Critical Constants (verified against Java source)

```
MARGIN = 6                  (SvekResult moveDelta offset)
CANVAS_DELTA = 15           (SvekResult delta(15,15))
DOC_MARGIN_RIGHT = 5        (plantuml.skin document style)
DOC_MARGIN_BOTTOM = 5
nodesep = max(0.35, 35/72)  (DotStringFactory)
ranksep = max(0.8, 60/72)
PARTICIPANT_OUT_MARGIN = 5  (ParticipantBox)
NOTE_PADDING_X = 5          (ComponentRoseNote)
```

### Debugging Workflow

```
1. Run reference test, locate first differing byte
2. Classify: coordinate / attribute / structure / content
3. Find Java code path generating that value
4. Instrument Java, rebuild, capture exact intermediates
5. Implement same algorithm in Rust
6. Verify: cargo test --test reference_tests
7. If regression: analyze & fix forward, don't revert
8. Commit
```

Java source: `/ext/plantuml/plantuml`
Build: `cd /ext/plantuml/plantuml && ./gradlew jar`
Run: `java -jar build/libs/plantuml-*.jar -tsvg -pipe < input.puml 2>debug.txt > output.svg`

## Maintenance

### Test Strategy

Three-tier testing:
1. **Unit tests** (~2500): within each parser/layout/render module
2. **Integration tests** (~183): `.puml` → `.svg`, verify valid SVG + no markup leakage
3. **Reference tests** (296): byte-for-byte comparison against Java PlantUML 1.2026.3beta5
   - Reference SVGs in `tests/reference/`
   - Test macro: `reference_test!` in `tests/reference_tests.rs`

Font metrics extracted from Java AWT via `tests/tools/ExtractFontMetrics.java` → `src/font_metrics.rs`. Verified exact match with Java (zero divergence).

### Code Style

- License: Multi-licensed (GPL-3.0 / LGPL-3.0 / Apache-2.0 / EPL-2.0 / MIT)
- Git messages: English, concise, no AI tool mentions
- Author: `kookyleo <kookyleo@gmail.com>`
- Never use `--release` during development/debugging
- Prefer editing existing files over creating new ones
- Minimize dependencies

### Parallel Agent Work

File ownership matrix (enforced during parallel agent work):
- Each agent only modifies its assigned files
- parser/layout/render naturally isolated by diagram type
- Shared files (lib.rs, mod.rs, diagram.rs) modified sequentially by the main thread

### Pipeline Architecture

For complex multi-step fixes, use a two-agent pipeline:
1. **Analysis agent** (reads Rust + instruments Java): produces precise gap analysis with exact numeric values
2. **Implementation agent** (writes Rust + reads Java): implements fixes based on analysis, fixes regressions forward
