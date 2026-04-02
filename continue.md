# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 260/296 passed (87.8%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain this session**: +39 tests
- **Unit tests**: 2605/2605 (100%)

## Remaining 36 failures — root cause classification

### Root Cause 1: Subdiagram `{{ }}` not implemented (3 tests)
`subdiagram_theme_02`, `subdiagram_theme_01`, `dev_newline_subdiagram_theme`

viewBox: J=640×510 R=60×68 — the inner diagram is completely missing.
**Fix**: Parse `{{ }}`, recursively render, embed as `<g>`.

### Root Cause 2: Sequence freeY not tracking sprite height (5 tests)
`testGradientSprite` (h: 336→272, -64), `testPolylineSprites` (h: 410→346, -64), `svgFillColourTest_2174` (w: 203→640)
+ `deployment_mono_multi` ×2 (h: 232→116, body sizing)

All share the same -64px pattern: notes after sprite messages are positioned too high because `y_cursor` doesn't advance by sprite preferred height. The `svgFillColourTest` has a different width issue (sprite in stereotype).
**Fix**: Track `freeY` properly in classic sequence layout for sprite-containing messages.

### Root Cause 3: DOT node ordering for graphviz layout (4 tests)
`chenmoviealias` ×2 (w: 1492→1428, -64), `chenmovieextended` ×2 (w: 1531→1475, -56)

Height matches. Width differs because graphviz places nodes at different horizontal positions depending on DOT declaration order.
**Fix**: Match Java's source-order entity/attribute/ISA interleaving in DOT generation.

### Root Cause 4: Teoz group recursive height model (4 tests)
`TeozTimelineIssues_0007` ×2 (h: 437→399, -38), `TeozTimelineIssues_0009` ×2 (w: 235→363, +128)

Complex teoz with `?` participant and nested groups.
**Fix**: Port Java GroupingTile.getPreferredHeight() recursive model.

### Root Cause 5: State composite cluster architecture (3 tests)
`state_history001` (h: 404→454, +50), `scxml0004` (w: 266→340, +74), `scxml0003` (h: 436→435, -1)

Java uses 5-level cluster nesting (a/p0/main/i/p1) putting composite children in the OUTER DOT. Our code uses separate inner graphviz solve.
**Fix**: Port Java cluster nesting for state composites (large architectural change).

### Root Cause 6: Component edge path coordinate transform (3 tests)
`deployment01` (w: 623→625, +2), `xmi0001` ×2 (h: 267→189, note ear connector)

Edge path SVG coordinates have sub-pixel offset from the svek-to-component transform chain.
**Fix**: Align the moveDelta → normalize → render_offset chain precisely.

### Root Cause 7: Class note positioning (4 tests)
`jaws7` ×2 (h: 78→76, -2), `jaws12`/`mindmap_jaws12` ×2 (h: 129→125, -4)

Note is positioned outside entity bounds; viewport doesn't include it. Mindmap has tree Y-balancing diff.
**Fix**: Include notes in graphviz layout, fix mindmap Tetris algorithm.

### Root Cause 8: Missing engine features (5 tests)
- `TimingMessageArrowFont` ×2 — timing diagram message font height
- `A0003` — Gantt `printscale weekly`
- `A0004` — legacy activity `(*)` syntax
- `handwritten001` — SVG post-processing with PRNG jiggle

Each is an independent engine. **Fix**: Implement each separately.

### Root Cause 9: C4 stdlib macros (2 tests)
`jaws1` ×2 — needs `!include <C4/C4_Container>` stdlib support.
**Fix**: Implement C4 macro subset or bundle C4 stdlib files.

### Root Cause 10: Usecase svek pipeline (1 test)
`usecase_basic` — needs actors/usecases routed through svek/graphviz.
**Fix**: Port usecase layout to svek pipeline.

### Root Cause 11: Title table cell color parsing (1 test)
`link_url_tooltip_04` (w: 558→730) — `<#color>` cell background prefix inflates width.
**Fix**: Strip `<#color>` prefix before measuring cell text width.

## Recommended execution order (by ROI)

| Priority | Root Cause | Tests | Effort |
|----------|-----------|-------|--------|
| 1 | RC11: title table cell color | 1 | Small |
| 2 | RC3: DOT node ordering | 4 | Small |
| 3 | RC6: component edge transform | 3 | Medium |
| 4 | RC2: sprite freeY tracking | 5 | Medium |
| 5 | RC7: class note + mindmap | 4 | Medium |
| 6 | RC4: teoz group height | 4 | Medium |
| 7 | RC1: subdiagram {{ }} | 3 | Large |
| 8 | RC5: state cluster arch | 3 | Large |
| 9 | RC8: engine features | 5 | Large (each) |
| 10 | RC9: C4 stdlib | 2 | Large |
| 11 | RC10: usecase svek | 1 | Large |
