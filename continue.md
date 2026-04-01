# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 256/296 passed (86.5%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain this session**: +35 tests
- **Unit tests**: 2605/2605 (100%)

## Key finding: Smetana port is NOT needed

Full measurement of all 40 failing tests shows:
- 5 tests have **viewBox exactly matching Java** — failure is internal rendering detail only
- ~25 tests fail due to **feature gaps** (subdiagram, ISA, handwritten, etc.)
- ~5 tests have **small layout diffs** (state cluster architecture, not Smetana precision)
- ~5 tests have **node sizing diffs** (coordinate truncation, not engine differences)

External `dot` and Java's embedded Smetana produce functionally identical layouts for our test suite.

---

## Implementation Plan: 40 → 0

### Tier 1: viewBox matches, fix internal rendering (~10 tests)

These tests have **identical layout** between Java and Rust. The failures are in SVG element details (coordinate format, element order, attribute values).

| Test | viewBox | Root cause | Fix |
|------|---------|-----------|-----|
| chenmovie ×2 | 1235×366 ✅ | Coordinate int truncation, element `<g>` wrapping | Truncate rect coords to int, add entity `<g>` groups |
| ComponentExtraArrows ×2 | 544×192 ✅ | Edge path coordinates 6px off | Fix svek edge-to-component coordinate transform |
| ChenRankDir ×1 | 556×84 ✅ | Already passes (verify) | — |
| TeozTimelineIssues_0002 ×2 | 740×200 ✅ | Activation box height | Fix deactivate Y for inline events |
| a0002 ×1 | 562×736 ✅ | Already passes (verify) | — |

**Effort**: Small. Each is a targeted rendering fix.

### Tier 2: Small layout diffs (~5 tests)

| Test | Diff | Root cause | Fix |
|------|------|-----------|-----|
| weak001 | w-11px, h-1px | ERD entity width estimation | Match Java `TextBlockInEllipse` sizing |
| mindmap_jaws12 | h-4px | Mindmap tree Y balancing | Match Java Tetris algorithm |
| scxml0004 | w+74px | Pin state rendering as full rect vs 12×12 | Render `<<inputPin>>` as compact port |
| state_history001 | h+50px | Java 5-level cluster nesting | Port cluster architecture or approximate |
| testGradientSprite | h-64px | SVG gradient sprite transform | Implement gradient fill for sprites |

**Effort**: Medium. Each needs specific algorithm alignment.

### Tier 3: Feature implementations (~20 tests)

#### 3a. Subdiagram `{{ }}` embedding (3 tests: subdiagram_theme ×2, subdiagram_theme_01)
- Parse `{{ }}` blocks as nested diagram source
- Recursively render inner diagram to SVG
- Embed as `<g>` group at correct position
- **Effort**: Large (recursive rendering pipeline)

#### 3b. Chen ISA entity type (2 tests: chenmoviealias, chenmovieextended)
- Parse `<<isa>>` stereotype
- Render as triangle shape
- Route ISA edges through graphviz with correct shape
- **Effort**: Medium

#### 3c. Handwritten mode (1 test: skinparam_handwritten001)
- `klimt::hand` module already exists with Java-compatible PRNG
- Need SVG post-processing: `<rect>` → `<polygon>`, `<line>` → `<path>`
- Add deprecation warning text overlay
- **Effort**: Medium (module exists, need integration)

#### 3d. Component deployment layout (3 tests: deployment01, deployment_mono_multi ×2)
- Nested container groups (`node "X" { artifact "Y" }`) as graphviz clusters
- Multi-line entity names with `<code>` blocks and creole markup
- **Effort**: Medium (cluster infrastructure exists, need entity body rendering)

#### 3e. Class display_name with bold creole (2 tests: jaws7 ×2)
- `!define` macro expansion produces `class "<b>TYP: B</b>" as A`
- Render `<b>` in class header name
- **Effort**: Small (class parser already handles `as`, need bold rendering in header)

#### 3f. C4 diagram support (2 tests: jaws1 ×2)
- C4 uses `!include` for stdlib macros
- Needs `System_Boundary`, `Container`, `Person` primitives
- **Effort**: Large (stdlib macro system)

#### 3g. Usecase layout direction (1 test: basic)
- Java uses graphviz `rankdir=LR` equivalent for usecase with actors
- Our code produces vertical layout instead of horizontal
- **Effort**: Small (pass correct rankdir to graphviz)

#### 3h. Timing diagram rendering (2 tests: TimingMessageArrowFont ×2)
- Message arrow font height calculation
- Timing state band rendering
- **Effort**: Medium

#### 3i. Gantt weekly scale (1 test: A0003)
- `printscale weekly` → 56px per week column
- Scale factor and column rendering
- **Effort**: Medium

#### 3j. Legacy activity diagram (1 test: A0004)
- Old-style activity with `(*) -->` syntax
- Partition/swimlane support
- **Effort**: Large

#### 3k. Link URL tooltips (1 test: link_url_tooltip_04)
- `[[url{tooltip}]]` syntax on links
- Render as `<a>` wrapper with `xlink:title`
- **Effort**: Small

#### 3l. Sprite rendering improvements (3 tests)
- Gradient fill for sprites
- Polyline sprite shapes
- SVG fill colour test
- **Effort**: Medium

### Tier 4: Rendering detail alignment (ongoing)

For all tiers, after feature implementation:
- Int-truncate entity rect coordinates (`x as i32`, not `x` as float)
- Match Java element ordering within `<g>` groups
- Match Java link comment format (`<!--link X to Y-->`)
- Match Java `data-entity-1`/`data-entity-2` attribute values

---

## Recommended execution order

1. **Tier 1** first — highest ROI, ~10 tests with minimal effort
2. **Tier 3e, 3g, 3k** — small features, 4 tests
3. **Tier 3b** — Chen ISA, 2 tests
4. **Tier 2** — layout diffs, 5 tests
5. **Tier 3d, 3c** — deployment + handwritten, 4 tests
6. **Tier 3h, 3i** — timing + gantt, 3 tests
7. **Tier 3l** — sprites, 3 tests
8. **Tier 3a, 3f, 3j** — large features (subdiagram, C4, legacy activity), 6 tests

**Estimated total to 296/296: ~15 focused subagent sessions**

---

## Session commits summary

Over 60+ commits covering: svek LimitFinder, state composites, sequence arrows (bidirectional, cross, circle, boundary, half-arrow, activation bars), teoz fragments, notes, activation bars, skin rose theme, sprites, creole rendering (block-level tables/bullets/rules, inline `<u:blue>`/`<U+XXXX>`/`<code>`), ERD pipeline, class parser, mindmap positioning, component direction hints.
