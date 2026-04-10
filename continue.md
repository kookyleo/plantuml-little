# Continue: Stable Java Authority

## Current Baseline (2026-04-10)

- Java reference authority: official PlantUML `v1.2026.2`
- Authority checkout: `/ext/plantuml/plantuml-official-stable-v1.2026.2`
- Authority SHA: `bb8550d720e93f3e7f016a987848fb769e0222f5`
- Cargo package version: `1.2026.2`

## Verified Results

- `cargo test --lib` -> `2681/2681`
- `cargo test --test reference_tests` -> `329 passed / 0 failed / 3 ignored`
- The reference harness uses `tests/reference/INDEX.tsv` as the authoritative fixture-to-reference map.

## Active Blockers

These are the only remaining non-green items, and none are ordinary Rust parser/layout/render gaps.

- `tests/fixtures/ditaa/basic.puml`
  - Ignored because Java stable writes raw PNG bytes even under `-tsvg`.
  - This repo is SVG-only and `convert*()` returns `String`, so there is no byte-exact SVG authority to compare.
- `tests/fixtures/jcckit/basic.puml`
  - Ignored for the same reason: Java stable writes PNG bytes instead of UTF-8 SVG under `-tsvg`.
- `tests/fixtures/sprite/svg2GroupsWithStyle.puml`
  - Ignored because Java stable `v1.2026.2` throws `NullPointerException` on the fixture.

## 2026-04-10 FLOW / PROJECT / JCCKIT Pass

- Added first-class `FLOW` support and byte-exact stable references:
  - fixtures: `tests/fixtures/flow/basic.puml`, `tests/fixtures/flow/link_back.puml`
  - refs: `tests/reference/flow/basic.svg`, `tests/reference/flow/link_back.svg`
- Added `PROJECT` parity for the stable-Java behavior:
  - Java stable does not render a project diagram here; it emits the white "Diagram not supported by this release of PlantUML" SVG page.
  - Rust now matches that output byte-exactly for `tests/fixtures/project/basic.puml`.
- Added `JCCKIT` detection and fixture coverage:
  - fixture: `tests/fixtures/jcckit/basic.puml`
  - reference artifact: `tests/reference/jcckit/basic.svg`
  - The committed `.svg` path contains PNG bytes from Java stable, so the reference test is intentionally ignored under the SVG-only contract.

## Practical Verdict

- All Java stable diagram families that currently provide UTF-8 SVG authority under the project contract are now green in byte-exact reference tests.
- The remaining gaps are product-boundary blockers (`DITAA`, `JCCKIT`) plus one Java-side crash fixture (`svg2GroupsWithStyle`).

## Files To Revisit If Product Scope Changes

- `src/parser/common.rs`
- `src/parser/mod.rs`
- `src/parser/flow.rs`
- `src/layout/flow.rs`
- `src/render/svg_flow.rs`
- `src/render/error_page.rs`
- `src/lib.rs`
- `tests/reference_tests.rs`
