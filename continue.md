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
- `cargo test --test reference_tests`: `118/322`
- Byte-compare authority remains the 318 stable-Java SVGs indexed by `tests/reference/INDEX.tsv`.

## Latest Push (2026-04-05)

- Focus area: teoz sequence vertical-budget cluster
- Core fix: removed the stale teoz `+10` body-margin assumption and aligned both body x/y origins to the Java stable `5px` document margin in `src/layout/sequence_teoz/builder.rs`.
- Verified guardrails:
  - `cargo test --lib` stays green at `2636/2636`
  - full stable reference suite moved from `94/322` to `118/322` (`+24`)
- Direct cluster effect:
  - `TeozAltElseParallel_*`: `12 -> 0`
  - `SequenceLayout_0004/0005/0005b`: `6 -> 0`
  - `TeozTimelineIssues_*`: `18 -> 6`
  - remaining teoz tail: `9` (`TeozTimelineIssues_0001/0002/0004/0005/0007/0009`, `SequenceArrows_0001/0002`, `SequenceLeftMessageAndActiveLifeLines_0001`)

## Failure Cluster Ranking (Highest Leverage First)

This ranking is based on the current stable-reference run after the teoz margin fix and groups failures by likely shared implementation path, not by directory alone.

### P0 — Shared newline / multiline richtext cluster (`34` fails)

- Dominant symptom buckets:
  - `height +14`: still the largest bucket
  - then `+20`, `+8`, `+9`, and `-1`
- Seen across `dev/newline`, `preprocessor`, `component`, `misc`, `activity`, and `wbs`.
- Reason for priority: this is still the cleanest cross-family multiplier after teoz shrank. The same newline/multiline handling still leaks into several diagram stacks.
- Primary files:
  - `src/render/svg_richtext.rs`
  - `src/preproc/`
  - per-diagram height accounting in `src/layout/`

### P1 — Sprite bounds / transform / gradient cluster (`39` fails)

- Dominant symptom buckets:
  - mixed structure/content diffs: the majority
  - tiny coordinate drifts
  - a smaller `height +14` bucket
- Reason for priority: still the single largest family by raw count, but less uniform than newline. Treat it as a shared rendering stack with several sprite subproblems.
- Primary files:
  - `src/render/svg_richtext.rs`
  - `src/klimt/svg.rs`

### P2 — Jaws / component / description cluster (`27` fails)

- Mixed `jaws*`, `gml*`, `deployment01`, `componentextraarrows_0001`, and a few width/height component tails still remain.
- Reason for priority: broader than the old `jaws` bucket; it now covers the remaining shared description/component path after the teoz cleanup.
- Primary files:
  - `src/layout/component.rs`
  - `src/render/svg_component.rs`

### P3 — State / SCXML vertical-budget cluster (`18` fails)

- Dominant symptom buckets:
  - `height +8`: `11`
  - `height +11`: `6`
- Reason for priority: tight cluster, repeated SCXML/state topology, likely one remaining cluster-height / viewport-budget mismatch rather than many unrelated bugs.
- Primary files:
  - `src/layout/state.rs`
  - `src/render/svg_state.rs`

### P4 — Teoz remaining tail (`9` fails)

- Remaining fixtures:
  - `TeozTimelineIssues_0001/0002/0004/0005/0007/0009`
  - `SequenceArrows_0001/0002`
  - `SequenceLeftMessageAndActiveLifeLines_0001`
- Reason for priority: the first teoz margin pass already removed most of the cluster. What remains is narrower and no longer the biggest lever.
- Primary files:
  - `src/layout/sequence_teoz/builder.rs`
  - `src/render/svg_sequence.rs`

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
