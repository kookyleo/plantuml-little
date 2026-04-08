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

## Current Parity Baseline (2026-04-08)

- `cargo test --lib`: `2641/2641`
- `cargo test --test reference_tests`: `296/320` (92.50%)
- Byte-compare authority remains the 318 stable-Java SVGs indexed by `tests/reference/INDEX.tsv`.

### 2026-04-08 Fixes (294 → 296)
- **sequence sprite-bearing messages and notes (layout/sequence.rs, parser/sequence.rs, render/svg_richtext.rs)**:
  When a sequence message contains an inline `<$sprite>` taller than the
  default text line height, Java places any following note as a standalone
  `NoteBox` below the arrow tile (not as a combined `ArrowAndNoteBox`).
  Apply that path for sprite messages: skip arrow centering, position the
  note polygon at `msg_y + (arrowDeltaY + paddingY) + notePaddingY`,
  finalize the lifeline at `note_y + note_pref_h + 5`, and skip the +3px
  overlay-baseline tweak. Note text rendering aligns each row's text
  baseline to `row.bottom - descent` so sprite-bearing rows space
  correctly. Note width measurement now sums sprite atom widths and
  preserves trailing whitespace inside multi-line note text (Java
  `BodyEnhanced2` does not trim). Use the runtime sans-13 line height for
  the sprite-replacement threshold (was a 4-decimal constant) so
  `sprite_extra` math stays byte-exact with Java. Fixes
  `sprite/testGradientSprite` and `sprite/testPolylineSprites`.
- **style block: BorderColor falls back to LineColor (style/compat.rs)**:
  Java's PName has only `LineColor` (no separate `BorderColor`); element
  `<style>` blocks set `LineColor` which becomes the visible border for
  bounded shapes. `border_color()` now picks up `participantlinecolor` /
  `participant.linecolor` before reaching the root/theme defaults.

### 2026-04-07 Fixes (293 → 294)
- **class/map plaintext padding (layout/mod.rs, layout/graphviz.rs, render/svg.rs)**:
  Java's `EntityImageMap` uses `ShapeType.RECTANGLE_HTML_FOR_PORTS`, which
  emits `shape=plaintext` with an HTML table label. Graphviz's default
  plaintext margin (`0.055in` ≈ 4pt) inflates the node bbox by ~8px
  (4 top + 4 bottom), widening the rank gap. Mirror this by inflating the
  DOT `height_pt` by 8 for Map entities while tracking the natural render
  height in a new `image_height_pt` field; `parse_svg_node` and the
  svek-fast-path now use `image_height_pt` so the rendered rect stays the
  natural size but sits centered within the larger DOT bbox. Also fix the
  map row text baseline: Java's `TextBlockMap` wraps each cell in
  `withMargin(2,2)` so the baseline needs a +2 top inset. Fixes
  `object/map`.

### 2026-04-07 Fixes (290 → 291)
- **ArrowAndNoteBox arrow centering (layout/sequence.rs)**: Mirror Java's
  `ArrowAndNoteBox.pushToDown`: when notePH > arrowPH, the arrow line is
  shifted down by `(notePH - arrowPH) / 2` so it sits at the vertical
  midpoint of the combined tile. Use `lp.message_spacing` (Java's
  `arrow.getPreferredHeight`) as the centering arrow_ph rather than the
  back-offset-inflated `y_cursor - note_y` value, and subtract the
  `note_extra` baseline shift (3 px) so the centered arrow does not
  double-count it. Also extend the lifeline to `note_y + note_pref_h + 5`
  to match Java's Frontier advance after the centered tile. Fixes
  `misc/creole_note001`.

### 2026-04-07 Investigation Notes (291/320 plateau)

Triaged remaining 29 failures and confirmed all are non-trivial:

- **sprite/styleFontWeightRoboto, testGradientSprite, testPolylineSprites**:
  All fail due to inline `<$sprite>` in sequence message text. Java's
  `AtomSprite.calculateDimension` returns full sprite (`UImageSvg`) height,
  and `Sea.doAlign` lays atoms with text/sprite vertical centering.
  Currently `message_sprite_extra_height()` adds `(sprite_h - 15.13)` to the
  arrow `msg_y`, but Java's actual `textBlock` height is computed by
  `Sea.getMaxY - getMinY` which includes the AtomText `startingAltitude`
  (font space). Reproducing this requires implementing the full sea-style
  layout: see `Sea.java`, `AtomSprite.java`, `SpriteSvg.java` and
  `AbstractTextualComponent.getTextHeight()`. The styleFontWeightRoboto case
  also needs `<style>@import url(...)</style>` defs emission for fonts +
  honoring style sheet `stroke`/`font-family`/`font-style` overrides.
- **mindmap_jaws12, dev_jaws_jaws12, dev_jaws_jaws1, preprocessor_jaws1**:
  total mindmap/C4 width differs by `~9 px` from Java. Width is
  `fullElongation + getX12(30) + 2*MARGIN(10)`, and Java's
  `FingerImpl.getPhalanxElongation` calls phalanx `calculateDimension`
  width which uses creole text block sizing (different padding). Our
  `Finger::full_elongation` accumulates per-level box widths but the
  spacing math drifts. Needs side-by-side trace against
  `FingerImpl.getFullElongation`.
- **sprite_svgFillColourTest_2174, svgFillColourTest_2174 (legend)**:
  20-line diff because we render legend sprites as nested `<svg>` tags
  while Java emits raw `<path>` from the sprite SAX parser. The sprite
  renderer (`svg_sprite.rs`) needs to inline path data via the SAX nano
  parser instead of wrapping in inline SVG.
- **dev_newline_subdiagram_theme, component_subdiagram_theme_02,
  preprocessor_subdiagram_theme_01**: subdiagram embedded as base64
  `<image>` of an inner SVG. We render the inner SVG with substantially
  fewer elements (no fallback theme info, missing syntax error block).
  The subdiagram renderer (`render_subdiagram` path) needs a sequence
  fallback for failed theme parses.
- **misc_link_url_tooltip_05**: file note shape needs the path
  `M7,46.7969 L7,249.0625 A2.5,2.5 0 0 0 9.5,251.5625 L...`-style fold
  rectangle (Java `BlockBoxStyle.create`) instead of plain rounded rect.
  Also `\\nb` (literal backslash) tooltips need different escape handling.
- **dev_newline_json_escaped, json_json_escaped**: JSON entry rendering
  emits extra ellipses on table rows our renderer omits, and link path
  geometry differs. JSON is currently using a simplified renderer.
- **builtin_newline (dev/newline + preprocessor)**: our SALT/grid
  renderer paints background rect+gridlines while Java only emits text
  cells with line strokes. SALT uses a fundamentally different approach
  in our codebase (`svg_richtext`-style grid).
- **Skip permanently**: ebnf, regex, salt/basic, nwdiag, files_diagram,
  yaml_basic, object_map, sprite_svg2GroupsWithStyle (Java NPE),
  skinparam_handwritten001 (major handwritten/sketch theme).

The remaining fixable failures require either: implementing the full
Java `Sea`/`Atom` layout for inline sprites in text, or porting the
sprite SAX parser to emit raw SVG primitives. Both are multi-day
efforts beyond a single short session.

### 2026-04-07 Fixes (280 → 286)
- **Sequence polygon HACK_X_FOR_POLYGON (svg_sequence.rs)**: Java's LimitFinder
  inflates polygon bounds by 10px on both ends of x. Mirror this in
  `track_polygon_points` so teoz diagrams with `->]` / `[->` boundary arrows
  match Java viewport width (fixed SequenceArrows_0001/0002 + preprocessor
  mirrors).
- **SvgGraphics ensureVisible shadow padding (svg_sequence.rs)**: Track two
  extents in parallel — LimitFinder-style and SvgGraphics-ensureVisible-style
  — so shadowed paths/rects/lines push the viewport the way Java's
  SvgGraphics does via `2*deltaShadow`. Final viewport = max of both
  (fixes SequenceLeftMessageAndActiveLifeLines_0001 + preprocessor mirror).
- **Note right x-offset (layout/sequence.rs)**: Java's NoteBox.getStartingX
  uses `(int)(posC + rightShift)`, then AbstractComponent.drawU adds
  `paddingX=5`. Changed from `posC + ACTIVATION_WIDTH(10)` to
  `(int)(posC + active_right_shift) + NOTE_COMPONENT_PADDING_X` with a
  look-ahead for pending `activate target` (matches Java DrawableSet-
  Initializer line 495 which records activation stairs at the message y).
- **SVG seed source hashing (klimt/svg.rs)**: Strip comment lines (leading
  `'`) and concatenate each surviving line + `\n` before hashing, matching
  Java's `getPlainString("\n")` on the preprocessor-filtered source list.
  Aligns filter/shadow/gradient IDs byte-exact with Java.

### 2026-04-05 Fixes (268→271)
- **Preprocessor backslash boundary**: Java Define.apply2() translates `\n` to private-use Unicode before word-boundary matching so `!TEST=something` correctly substitutes in `test:\nTEST`. (src/preproc/mod.rs)
- **CLASS body centering**: Java MinMax.getDimension() returns span (maxX-minX), not absolute max. For CLASS diagrams with meta elements, subtract the moveDelta margin (6px) from body_w to match Java's centering calculation. Fixes class/a0005, nonreg/A0005, misc/meta_title_header_footer. (src/render/svg.rs)

### 2026-04-05 Fixes (260/320)
- **Salt data-diagram-type**: Always emit `data-diagram-type="SALT"` for both @startsalt and inline salt
- **CSS diagram-type wrappers**: Support `sequenceDiagram { participant { ... } }` in `<style>` blocks
- **Legacy skinparam keys**: Store both dotted and concatenated forms for style lookup compat
- **Old-style activity viewport**: Reduced padding from +13 to +5 to match Java margins

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
