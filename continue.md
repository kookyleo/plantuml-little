# Continue: Stable Java Authority

## Current Baseline (2026-04-04)

- Java reference authority: official PlantUML `v1.2026.2`
- Authority checkout: `/ext/plantuml/plantuml-official-stable-v1.2026.2`
- Authority SHA: `bb8550d720e93f3e7f016a987848fb769e0222f5`
- Cargo package version: `1.2026.2`

## Reference Corpus

- Reference SVGs were regenerated from the stable Java checkout in file mode.
- `tests/reference/VERSION` records the exact jar, Git SHA, Java version, Graphviz version, and generation time.
- `tests/reference/INDEX.tsv` is the source of truth for fixture-to-reference mapping.
- `tests/reference_tests.rs` is regenerated and now covers all 322 fixtures; 318 fixtures have a stable Java SVG to byte-compare against.

## Known Stable-Java Coverage Gaps

The following fixtures currently produce no SVG with official PlantUML `v1.2026.2`, so they are not part of the byte-compare corpus:

- `tests/fixtures/chart/pie_basic.puml`
- `tests/fixtures/packet/basic.puml`
- `tests/fixtures/packet/tcp.puml`
- `tests/fixtures/pie/basic.puml`

## Development Rule

Any future Java/Rust parity work must target the stable `v1.2026.2` reference corpus now checked into `tests/reference/`.

## Current Parity Baseline (2026-04-05)

- `cargo test --lib`: `2636/2636`
- `cargo test --test reference_tests`: `212/322`
- Byte-compare authority remains the 318 stable-Java SVGs indexed by `tests/reference/INDEX.tsv`.

## Latest Push (2026-04-05)

- Focus area: viewport formula alignment + preprocessor define fix + zlib backend
- Core fixes:
  - **State viewport**: replaced span+CANVAS_DELTA with max-based formula matching Java `ImageBuilder.getFinalDimension()` in `src/render/svg_state.rs`
  - **Component/ERD viewport**: used `lf_span + 6` (Java `moveDelta = 6 - lf_min`) instead of `span + CANVAS_DELTA(15)` in `src/layout/component.rs` and `src/layout/erd.rs`
  - **Class/Component degenerated**: added +1 to match Java entity sizing in `src/render/svg.rs` and `src/layout/component.rs`
  - **Preprocessor legacy define**: fixed `expand_defines()` to use word-boundary matching (`\b` regex equivalent) in `src/preproc/expr.rs`, preventing substring replacement in words like "data" when define name is "t"
  - **svek lf_max**: exposed absolute LF max from `solve()` for viewport calculations in `src/svek/mod.rs`
  - **flate2 zlib backend**: switched from miniz_oxide to zlib for Java-compatible deflate output in plantuml-src encoding
- Verified guardrails:
  - `cargo test --lib` stays green at `2636/2636`
  - full stable reference suite moved from `133/322` to `188/322` (`+55`)
  - **Sprite renderer**: aligned with Java SvgNanoParser — drop unsupported elements, circle→ellipse, text plain, no gradient hoisting (sprite: 1→23)
  - **WBS margin**: 20→10 matching Java ImageBuilder default (+4)
  - **font-weight**: "700"→"bold" matching Java DriverTextSvg
    - back-highlight filter ids in `src/render/svg_richtext.rs`
    - sequence shadow filter id in `src/render/svg_sequence.rs`
- Verified guardrails:
  - `cargo test --lib` stays green at `2636/2636`
  - full stable reference suite moved from `118/322` to `133/322` (`+15`)
- Direct cluster effect:
  - `activity/*` multiline/table/swimlane/A0002 fixtures: `8 -> 0`
  - mirrored newline activity fixtures (`dev/newline*` + `dev/newlinev2*`): `5 -> 0`
  - shared back-highlight parity case `misc/creole_back001`: `1 -> 0`
  - mirrored old-activity parity case `nonreg/simple/A0002`: `1 -> 0`
  - remaining old-style activity tail is now down to `3`:
    - `nonreg/simple/A0003`
    - `nonreg/simple/A0004`
    - `misc/a0004`

## Failure Cluster Ranking (Highest Leverage First)

Updated after viewport-formula + preprocessor-define pass. 160 failures remain.

### P0 — Sprite bounds / transform / gradient cluster (`39` fails)

- 4 root causes identified:
  1. Shape elements converted to `<path>` instead of native SVG (9 tests) — `svg_sprite.rs`
  2. Gradient defs hoisted into `<defs>` block (11 tests) — Java resolves inline
  3. Missing `<title>` element + height mismatches (11 tests)
  4. Extra font attributes on `<text>` (8 tests)
- Primary files: `src/render/svg_sprite.rs`, `src/klimt/svg.rs`

### P1 — Shared newline / multiline / rendering diffs (`~50` fails)

- Includes: `dev/newline`, `preprocessor`, `component`, `misc`, `wbs`
- Legacy define substring bug now fixed; remaining failures have deeper rendering/layout diffs
- Primary files: `src/render/svg_richtext.rs`, `src/preproc/`, per-diagram `src/layout/`

### P2 — State / SCXML composite cluster (`8` fails)

- Viewport-only cases now fixed (6 passed)
- Remaining 8 have composite state layout + coordinate diffs
- Primary files: `src/layout/state.rs`, `src/render/svg_state.rs`

### P3 — Component / description / jaws cluster (`~10` fails)

- Viewport-only cases now fixed (7 passed)
- Remaining have structural diffs (C4, deployment, jaws rendering)
- Primary files: `src/layout/component.rs`, `src/render/svg_component.rs`

### P4 — Sequence viewport-only tail (`8` fails)

- SequenceArrows: -9px width (text width tracking issue)
- SVG0002: -1px (sequence body measure)
- gantt/a0003 + A0003: -2px (gantt label width)
- Primary files: `src/render/svg_sequence.rs`, `src/render/svg_gantt.rs`

### P5 — Timing arrow-font cluster (`4` fails)

- `TimingMessageArrowFont_0001/0002` + timing-directory mirrors
- Primary files: `src/render/svg_timing.rs`

### P6 — Small tail cases

- `regex` (3), `usecase` (3), `wire` (2), `ebnf` (2), `git` (2), `files` (2), `chart` (2)
- Each has different rendering/layout diffs requiring individual investigation
