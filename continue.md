# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 260/296 passed (87.8%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain this session**: +39 tests
- **Unit tests**: 2605/2605 (100%)

## Remaining 36 failures — grouped by next action

### Group A: Component body/note rendering (7 tests, ~2 sessions)
- `deployment01` — height 460=460 ✅, width 625 vs 623 (2px)
- `xmi0001` ×2 — note positioning with ear connector
- `deployment_mono_multi` ×2 — `<code>` block + `<u:blue>` + `<color:green>` in node name
- `jaws12` ×2 — C4 sprite in component body (mindmap copy)

**Next action**: Fix component edge path coordinate transform (2px width), then note ear connector path.

### Group B: Subdiagram `{{ }}` embedding (3 tests, ~1 session)
- `subdiagram_theme_02`, `subdiagram_theme_01`, `dev_newline_subdiagram_theme`

**Next action**: Parse `{{ }}` blocks, recursively render inner diagram, embed as `<g>`.

### Group C: Chen ERD ISA (2 tests)
- `chenmoviealias`, `chenmovieextended` — ISA circle implemented, width diff from node ordering

**Next action**: Match Java's entity/attribute/ISA interleaving order in DOT.

### Group D: Teoz timeline (4 tests, ~1 session)
- `TeozTimelineIssues_0007` ×2 — complex group height with `?` participant
- `TeozTimelineIssues_0009` ×2 — group activation height model

**Next action**: Port Java GroupingTile recursive height model.

### Group E: Class features (4 tests)
- `jaws7` ×2 — bold display_name 2px height diff (note positioning)
- `link_url_tooltip_04` — `[[url{tooltip}]]` + title table `<#color>` cell
- `mindmap_jaws12` — mindmap tree Y balancing (4px)

**Next action**: Fix class note positioning, implement URL link wrapper.

### Group F: State architecture (3 tests)
- `state_history001` — Java 5-level cluster nesting (50px height diff)
- `scxml0003` — 1px precision from render_dy
- `scxml0004` — pin state rendering as compact 12×12 port

**Next action**: Port Java cluster a/p0/i/p1 nesting for state composites.

### Group G: Sprite rendering (3 tests)
- `testGradientSprite` — gradient fill for sprites (64px height)
- `testPolylineSprites` — polyline sprite shapes (64px height)
- `svgFillColourTest_2174` — SVG fill colour test

**Next action**: Fix sequence layout freeY tracking for sprite messages.

### Group H: Special engines (5 tests)
- `TimingMessageArrowFont` ×2 — timing diagram message rendering
- `A0003` — Gantt `printscale weekly` (scale factor)
- `A0004` — legacy activity `(*)` syntax
- `handwritten001` — handwritten mode SVG post-processing

**Next action**: Each is an independent engine feature.

### Group I: Misc (3 tests)
- `usecase_basic` — needs svek pipeline for usecase (actor rendering)
- `jaws1` ×2 — C4 `!include` stdlib macros

**Next action**: Port usecase to svek, implement C4 stdlib subset.

### Group J: SCXML precision (1 test)
- `scxml0003` — 1px from render_dy calculation

**Next action**: Already very close, fix render_dy for mixed rect/ellipse.

## Recommended execution order

1. **Group A** (component 2px + note) — highest ROI, 7 tests
2. **Group E** (class features) — 4 tests, small work
3. **Group C** (Chen ISA order) — 2 tests
4. **Group F** (state cluster) — 3 tests, medium work
5. **Group G** (sprite freeY) — 3 tests
6. **Group D** (teoz timeline) — 4 tests
7. **Group B** (subdiagram) — 3 tests, large work
8. **Group H** (special engines) — 5 tests, each independent
9. **Group I** (misc) — 3 tests, large work
