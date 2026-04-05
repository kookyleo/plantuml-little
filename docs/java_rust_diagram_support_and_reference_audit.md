# Java/Rust Stable-Reference Audit

> Generated 2026-04-04.
> This document supersedes earlier beta-based authority notes.

## 1. Executive Summary

- The repository now uses the official PlantUML stable release `v1.2026.2` as its only Java reference authority.
- The authority checkout is `/ext/plantuml/plantuml-official-stable-v1.2026.2` at Git SHA `bb8550d720e93f3e7f016a987848fb769e0222f5`.
- `tests/reference/` was regenerated from that stable checkout in file mode, not `-pipe` mode.
- `tests/reference/VERSION` now records the stable jar path, Git SHA, Java version, Graphviz version, and generation time.
- `tests/reference/INDEX.tsv` is now the fixture-to-reference source of truth. It captures nontrivial Java output names such as `ditaa/-r.svg` and `movies.svg`.
- `tests/reference_tests.rs` was regenerated and now contains 322 tests, one per fixture.
- Of the 322 fixtures under `tests/fixtures/`, 318 now have a stable-Java SVG that can be byte-compared by the Rust harness.
- 4 fixtures currently produce no SVG at all with official PlantUML `v1.2026.2`, so they are outside the byte-compare corpus for now:
  - `tests/fixtures/chart/pie_basic.puml`
  - `tests/fixtures/packet/basic.puml`
  - `tests/fixtures/packet/tcp.puml`
  - `tests/fixtures/pie/basic.puml`
- Rust still does not cover the full stable Java surface. Major non-UML gaps remain: `BPM`, `PROJECT`, `JCCKIT`, `FLOW`, standalone `CREOLE`, `MATH`, `LATEX`, `DEFINITION`, and `WIRE`.

## 2. Authority Definition

The project now has a single Java authority:

- checkout: `/ext/plantuml/plantuml-official-stable-v1.2026.2`
- release: `v1.2026.2`
- Git SHA: `bb8550d720e93f3e7f016a987848fb769e0222f5`
- built jar: `/ext/plantuml/plantuml-official-stable-v1.2026.2/build/libs/plantuml-1.2026.2.jar`

Development rule:

- Rust SVG parity work must target the stable `v1.2026.2` reference corpus checked into `tests/reference/`.
- The local dirty checkout at `/ext/plantuml/plantuml` is no longer a reference authority.
- Earlier beta-based notes are historical only and should not guide current parity work.

## 3. Reference Generation Rules

Reference generation is now anchored in `tests/generate_reference.sh`.

Key rules:

- default Java source: `/ext/plantuml/plantuml-official-stable-v1.2026.2`
- generation mode: file mode via `--svg --output-dir`, not `-pipe`
- output retention rule: if Java emits an SVG, the SVG is kept even when Java exits nonzero
- mapping source: `tests/reference/INDEX.tsv`

Why this matters:

- file mode matches real fixture execution better than `-pipe`
- error SVGs are part of actual Java output and therefore part of the authority corpus
- alternate Java output names are preserved instead of being normalized away

## 4. Stable Java Taxonomy vs Rust Support

The stable Java taxonomy comes from:

- `/ext/plantuml/plantuml-official-stable-v1.2026.2/src/main/java/net/sourceforge/plantuml/core/DiagramType.java`

The main Rust entry points remain:

- `src/model/diagram.rs`
- `src/parser/common.rs`
- `src/parser/mod.rs`

Legend:

- `Implemented`: Rust has a dedicated or clearly folded parse/layout/render path.
- `Partial`: behavior exists, but the Java stable surface is not matched cleanly.
- `Missing`: no first-class Rust support was found for that stable Java type.

| Stable Java type | Rust status | Notes |
|------------------|-------------|-------|
| `UML` | Partial | Rust implements the major UML families, but Java stable groups them under one umbrella while Rust models them separately |
| `BPM` | Missing | No `@startbpm` handling found |
| `DITAA` | Implemented | Stable Java emits `-r.svg`; mapped through `tests/reference/INDEX.tsv` |
| `DOT` | Implemented | Dedicated Rust path exists |
| `PROJECT` | Missing | No first-class Rust project/gantt-style project parser found |
| `JCCKIT` | Missing | No Rust `@startjcckit` handling found |
| `SALT` | Implemented | Dedicated Rust path exists |
| `FLOW` | Missing | No Rust `@startflow` handling found |
| `CREOLE` | Missing | Rich text exists in Rust, but not as a standalone Java diagram type |
| `MATH` | Missing | No Rust `@startmath` handling found |
| `LATEX` | Missing | No Rust `@startlatex` handling found |
| `DEFINITION` | Missing | No Rust `@startdef` handling found |
| `GANTT` | Implemented | Dedicated Rust path exists |
| `CHRONOLOGY` | Implemented | Dedicated Rust path exists; the stable Java fixture currently emits an error SVG |
| `NW` | Implemented | Rust `Nwdiag` path exists |
| `MINDMAP` | Implemented | Dedicated Rust path exists |
| `WBS` | Implemented | Dedicated Rust path exists |
| `WIRE` | Missing | Stable Java recognizes it; Rust still lacks real `@startwire` support |
| `JSON` | Implemented | Dedicated Rust path exists |
| `GIT` | Implemented | Dedicated Rust path exists; current stable fixture emits an error SVG |
| `BOARD` | Implemented | Dedicated Rust path exists; current stable fixture emits an error SVG |
| `YAML` | Implemented | Dedicated Rust path exists |
| `HCL` | Implemented | Dedicated Rust path exists |
| `EBNF` | Implemented | Dedicated Rust path exists |
| `REGEX` | Implemented | Dedicated Rust path exists |
| `FILES` | Implemented | Dedicated Rust path exists |
| `CHEN_EER` | Implemented | Rust models this as `Erd` |
| `CHART` | Implemented | Rust chart path exists; one chart fixture has no stable Java SVG output |
| `PACKET` | Implemented | Rust packet path exists; current stable Java produces no SVG for the repo fixtures |
| `UNKNOWN` | Sentinel only | Not a user-facing support target |

## 5. Reference Coverage Audit

Current repository counts:

- fixtures: 322 `.puml`
- generated reference tests: 322
- generated reference SVG files: 323
- indexed primary byte-compare mappings: 318

Primary byte-compare coverage is therefore:

- `318 / 322 = 98.76%`

The remaining 4 fixtures have no stable Java SVG output and therefore no byte-compare reference mapping:

- `tests/fixtures/chart/pie_basic.puml`
- `tests/fixtures/packet/basic.puml`
- `tests/fixtures/packet/tcp.puml`
- `tests/fixtures/pie/basic.puml`

## 6. Nontrivial Reference Mapping Cases

Stable Java does not always name output files after the input `.puml`.

The current authoritative examples are:

- `tests/fixtures/ditaa/basic.puml` -> `tests/reference/ditaa/-r.svg`
- `tests/fixtures/erd/chenmovie.puml` -> `tests/reference/erd/movies.svg`
- `tests/fixtures/erd/chenmoviealias.puml` -> `tests/reference/erd/movies.svg`
- `tests/fixtures/erd/chenmovieextended.puml` -> `tests/reference/erd/movies.svg`
- `tests/fixtures/nonreg/simple/ChenMovie.puml` -> `tests/reference/nonreg/simple/movies.svg`
- `tests/fixtures/nonreg/simple/ChenMovieAlias.puml` -> `tests/reference/nonreg/simple/movies.svg`
- `tests/fixtures/nonreg/simple/ChenMovieExtended.puml` -> `tests/reference/nonreg/simple/movies.svg`

These mappings are recorded in `tests/reference/INDEX.tsv` and consumed by `tests/reference_tests.rs`.

## 7. Stable Java Error-SVG Cases

Official PlantUML `v1.2026.2` sometimes exits nonzero but still emits an SVG. Those SVGs are now preserved as authoritative outputs.

Confirmed fixture set:

- `tests/fixtures/board/basic.puml`
- `tests/fixtures/chart/bar_basic.puml`
- `tests/fixtures/chart/single_series.puml`
- `tests/fixtures/chronology/basic.puml`
- `tests/fixtures/git/basic.puml`
- `tests/fixtures/git/branches.puml`
- `tests/fixtures/sequence/seq_divider001.puml`
- `tests/fixtures/wire/basic.puml`
- `tests/fixtures/wire/multi.puml`

Development rule:

- If stable Java emits an SVG, Rust is expected to match that SVG, even when Java also returns a nonzero exit code.

## 8. Secondary Multi-SVG Outputs

The regenerated stable corpus also contains additional secondary SVG pages from some fixtures. These files are preserved in `tests/reference/`, but the current harness only byte-compares the primary SVG selected by `tests/reference/INDEX.tsv`.

Current secondary-only preserved outputs:

- `tests/reference/dev/newline/activity_creole_table_001.svg`
- `tests/reference/dev/newline/class_funcparam_arrow_001.svg`
- `tests/reference/dev/newline/link_URL_tooltip_001.svg`
- `tests/reference/dev/newline/link_URL_tooltip_002.svg`
- `tests/reference/dev/newline/link_URL_tooltip_003.svg`
- `tests/reference/dev/newline/link_URL_tooltip_004.svg`
- `tests/reference/dev/newline/state_monoline_001.svg`
- `tests/reference/dev/newline/state_monoline_002.svg`
- `tests/reference/dev/newline/subdiagram_theme_001.svg`

This is a known harness limitation, not a reference-authority ambiguity.

## 9. Repository Files Updated for the Stable Baseline

- `Cargo.toml`
- `Cargo.lock`
- `tests/generate_reference.sh`
- `tests/generate_test_list.py`
- `tests/reference/VERSION`
- `tests/reference/INDEX.tsv`
- `tests/reference/`
- `tests/reference_tests.rs`
- `continue.md`
- `AGENTS.md`

## 10. Audit Artifacts

Supporting audit data for the stable-baseline transition is stored at:

- `tmp_debug/java_ref_audit_official_stable_v1_2026_2_20260404.json`

## 11. Next Development Rules

1. Do not reintroduce beta or dirty-local Java baselines into reference generation.
2. Use `tests/reference/INDEX.tsv` whenever Java output names do not match fixture names.
3. Treat the 4 no-SVG fixtures as unsupported by the current stable authority unless and until the authority version changes.
4. When fixing Rust parity, compare against the stable `v1.2026.2` SVGs now committed in `tests/reference/`.

## 12. Current Failure-Priority Ranking (2026-04-05)

The stable-baseline transition reset the parity scoreboard. After the latest sequence viewport alignment pass, the live baseline is:

- `cargo test --lib`: `2636/2636`
- `cargo test --test reference_tests`: `94/322`

The remaining failures are not equally valuable. For development planning, the current work should be ranked by shared-root-cause leverage, not by fixture path.

### 12.1 Highest-Leverage Clusters

| Priority | Cluster | Failures | Common signature | Primary Rust path |
|----------|---------|----------|------------------|-------------------|
| `P0` | Teoz sequence vertical-budget | `42` | mostly root height `+5px`; left-message activation `-3px` | `src/layout/sequence_teoz/builder.rs`, `src/render/svg_sequence.rs` |
| `P1` | Shared newline / multiline richtext | `39` | repeated height drift: `+14`, `+20`, `+8`, `+9`, `-1` | `src/render/svg_richtext.rs`, `src/preproc/`, multiline layout callers |
| `P2` | Sprite bounds / transform / gradient | `39` | mixed structure/content diffs plus tiny coordinate drift | `src/render/svg_richtext.rs`, `src/klimt/svg.rs` |
| `P3` | State / SCXML vertical-budget | `18` | mostly height `+8px` or `+11px` | `src/layout/state.rs`, `src/render/svg_state.rs` |
| `P4` | Jaws / component | `16` | heterogeneous component/layout mismatches | `src/layout/component.rs`, `src/render/svg_component.rs` |
| `P5` | Activity misc | `8` | repeated height `+14px` / `+8px` | `src/layout/activity.rs`, `src/render/svg_activity.rs` |
| `P6` | Timing arrow-font | `4` | height `+14px` in the two mirrored arrow-font fixtures | `src/render/svg_timing.rs` |

### 12.2 Cluster Notes

#### Teoz sequence vertical-budget

This is the cleanest next target. The subclusters are:

- `TeozTimelineIssues_*`: `18`
- `TeozAltElseParallel_*`: `12`
- `SequenceLayout_0004/0005/0005b`: `6`
- `SequenceArrows_*`: `4`
- `SequenceLeftMessageAndActiveLifeLines_*`: `2`

These cases are structurally similar and mostly differ only in final root height. That is strong evidence for one remaining teoz tile/event vertical-accounting mismatch rather than many independent bugs.

#### Shared newline / multiline richtext

This cluster spans `dev/newline`, `preprocessor`, `component`, `misc`, `activity`, and `wbs`. It is a high-payoff cross-family target because the failures likely come from shared newline preservation and multiline height accounting, not from diagram-specific geometry alone.

#### Sprite bounds / transform / gradient

This cluster is large but less uniform. It should be treated as several related sprite-rendering subproblems inside the same stack, not as one single constant mismatch.

#### State / SCXML

The remaining SCXML/state cases are now concentrated in repeated positive height drift. That suggests the remaining gap is cluster-height or viewport budgeting, not parser coverage.

### 12.3 Deferred Tail Cases

The following cases are still important, but they are currently lower leverage than the shared clusters above:

- `component/deployment01`: deployment clipping / group-edge path mismatch
- `sequence/seq_divider001`: stable Java emits an error-SVG path that differs sharply from the normal sequence renderer
- `sequence/seq_nested001`: near-zero coordinate drift (`147.9058` vs `147.9057`)

These should remain behind the shared-cluster queue unless one of them exposes a reusable lower-level bug.
