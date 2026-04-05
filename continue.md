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
- `cargo test --test reference_tests`: `94/322`
- Byte-compare authority remains the 318 stable-Java SVGs indexed by `tests/reference/INDEX.tsv`.

## Failure Cluster Ranking (Highest Leverage First)

This ranking is based on the current stable-reference run and groups failures by likely shared implementation path, not by directory alone.

### P0 — Teoz sequence vertical-budget cluster (`42` fails)

- Subclusters:
  - `TeozTimelineIssues_*`: `18`
  - `TeozAltElseParallel_*`: `12`
  - `SequenceLayout_0004/0005/0005b`: `6`
  - `SequenceArrows_*`: `4`
  - `SequenceLeftMessageAndActiveLifeLines_*`: `2`
- Dominant symptom: root SVG height drift, usually `+5px`; the left-message activation pair is `-3px`.
- Reason for priority: the signature is unusually consistent, the fixtures are structurally close, and they all converge on the same teoz event/tile vertical accounting path.
- Primary files:
  - `src/layout/sequence_teoz/builder.rs`
  - `src/render/svg_sequence.rs`

### P1 — Shared newline / multiline richtext cluster (`39` fails)

- Dominant symptom buckets:
  - `height +14`: `11`
  - `height -1`: `4`
  - `height +20`: `4`
  - `height +8`: `3`
  - `height +9`: `3`
- Seen across `dev/newline`, `preprocessor`, `component`, `misc`, `activity`, and `wbs`.
- Reason for priority: this is a cross-family multiplier, not a single diagram bug. Fixing newline-preservation and multiline height accounting can pay down several directories at once.
- Primary files:
  - `src/render/svg_richtext.rs`
  - `src/preproc/`
  - per-diagram height accounting in `src/layout/`

### P2 — Sprite bounds / transform / gradient cluster (`39` fails)

- Dominant symptom buckets:
  - mixed structure/content diffs: `19`
  - tiny coordinate drifts: `9`
  - `height +14`: `5`
- Reason for priority: large count, but less clean than teoz or newline. It likely contains multiple sprite subproblems that all live in the same rendering stack.
- Primary files:
  - `src/render/svg_richtext.rs`
  - `src/klimt/svg.rs`

### P3 — State / SCXML vertical-budget cluster (`18` fails)

- Dominant symptom buckets:
  - `height +8`: `11`
  - `height +11`: `6`
- Reason for priority: tight cluster, repeated SCXML/state topology, likely one remaining cluster-height / viewport-budget mismatch rather than many unrelated bugs.
- Primary files:
  - `src/layout/state.rs`
  - `src/render/svg_state.rs`

### P4 — Jaws / component cluster (`16` fails)

- Mixed `jaws*`, `gml*`, and component edge cases remain under the stable corpus.
- Reason for priority: still sizable, but the signature is less uniform than teoz or SCXML.
- Primary files:
  - `src/layout/component.rs`
  - `src/render/svg_component.rs`

### P5 — Activity misc cluster (`8` fails)

- Mostly root height deltas around `+14px` and `+8px`.
- Reason for priority: smaller surface area than the clusters above, but still likely shared vertical-budget cleanup rather than isolated bugs.
- Primary files:
  - `src/layout/activity.rs`
  - `src/render/svg_activity.rs`

### P6 — Timing arrow-font cluster (`4` fails)

- Fixtures:
  - `TimingMessageArrowFont_0001`
  - `TimingMessageArrowFont_0002`
  - timing-directory mirrors of the same two cases
- Dominant symptom: `height +14px`
- Reason for priority: very coherent, but only four tests, so lower leverage than teoz/newline/state.
- Primary files:
  - `src/render/svg_timing.rs`
  - timing layout path in `src/layout/`

### P7 — Hard singletons / tail cases

- `component/deployment01`: remaining deployment clipping / group-edge path mismatch
- `sequence/seq_divider001`: stable Java error-SVG path differs sharply from normal sequence output
- `sequence/seq_nested001`: near-zero coordinate drift (`147.9058` vs `147.9057`)

These should stay behind the shared-cluster work unless they become blockers for a broader fix.
