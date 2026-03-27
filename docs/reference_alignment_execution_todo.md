# Reference Alignment Execution TODO

## Current Baseline

- Current baseline after state standalone-note fix on 2026-03-28: `172 passed / 124 failed / 296 total`
- Fixed: standalone `note as ALIAS` in state diagrams now laid out by graphviz (was detached-right)
- Fixed: note fold stroke-width 0.5→1, note bounds tracked as UPath (no HACK_X_FOR_POLYGON)
- Fixed: SVG0004 reference CDATA formatting aligned to Java output
- Previous baseline after edge coordinate fix on 2026-03-28: `170 passed / 126 failed / 296 total`
- Fixed: 2px x/y shift in svek edge path coordinates — Graphviz SVG translate(tx,ty) and svek YDelta(full_height)+moveDelta use different transforms; now parsed edge data is corrected to match svek node space before merging
- Fixed: DOT label TABLE border polygons (fill="none") incorrectly identified as arrowheads; now filtered alongside stroke="transparent" label backgrounds
- Fixed: state note fold corner stroke-width from 1.0 to 0.5 to match Java SkinParam default
- Previous baseline after polygon + fold fixes on 2026-03-28: `168 passed / 128 failed / 296 total`
- Previous comparable baseline after LimitFinder image-width fix on 2026-03-28: `168 passed / 128 failed / 296 total`
- Previous baseline after svek overhaul + forward-fix on 2026-03-27: `160 passed / 136 failed / 296 total`
- The svek overhaul (class edge SIMPLIER, cluster shapes, shield ports) gained 9 tests (hideshow002/003, SVG0005, qualifiedassoc001/002, component/colors001) but regressed 9 others.
- Forward-fix session restored 8 of 9 regressions: CDATA CSS newline, entity UID ordering, track_empty viewport, generic protrusion offset, entity render order.
- Remaining 9 regressions (offset by 9 improvements): SVG0004_Smetana (CDATA inconsistency), hideshow004 (2px x-offset), class_funcparam_arrow_01 (x-offset from SIMPLIER node positions), scxml0001 (state width from svek node structure changes).
- Older local logs (`144/152` and one exploratory `123/173`) should now be treated as historical context only, not as the active authority baseline.
- The residual failures are still dominated by SVG height mismatches, especially in `CLASS`, `DESCRIPTION`, and `COMPONENT`-style outputs.
- The current bottleneck is no longer the old `SvekResult` or `ensureVisible` constant layer.
- The primary technical bottleneck is now split in three:
  - fragmented display/text/body semantics across parser, layout, and render
  - incomplete Java `LimitFinder` / `SvekResult` / cluster-shape semantics inside `svek`
  - missing Java `port/group` DOT semantics inside `svek` for component-style diagrams
- The primary execution bottleneck is baseline drift: pass counts from different local contexts are currently being mixed.
- Latest recheck after the `SvgResult` transform-chain fix still reports the same authority baseline: `160 / 136`.
- That means the recent `svek` repair removed latent focused regressions, but did not yet move the suite-wide pass count.
- Focused class-side work since then has improved `qualifiedassoc001/002`, but a new full-suite baseline has not been rerun yet.
- Focused component-side work has now re-stabilized `SVG0005_*` in the current workspace, so those fixtures should again be treated as guards rather than active failures.
- The latest focused state for `qualifiedassoc001/002` is:
  - shielded class endpoints now emit Java-style DOT ports: `"Map"->"HashMap":h` and `"HashMap":h->"Customer"`
  - interface header parity is no longer the active divergence for these fixtures
  - both fixtures shrank from `+2px` width error to `+1px`
- The latest focused state for `SVG0005_*` is:
  - both `SVG0005_Smetana` and `SVG0005_Svek` pass again
  - the transient regression was not a component text/body issue
  - the actual divergence was lower-level: component `raw_path_d` was arriving in a different coordinate space from the solved bezier `points` and arrow polygon, producing a stable `+6,+6` path offset
- The remaining failure map still shows the same leverage points:
  - `svg_height` dominates the suite
  - `newline_func` and `creole_markup` are still the strongest cross-cutting keywords
  - `SEQUENCE` remains the largest single diagram family, but `CLASS` plus `DESCRIPTION` still offer the faster shared-root-cause path

## Core Conclusion

- Do not keep attacking single failing fixtures one by one.
- Do not start from viewport constants unless Java tracing proves a viewport-only divergence.
- Treat the remaining failures as a small number of repeated root-cause clusters.
- Fix shared semantics first, then consume the cluster-specific tails.
- Promote truly foundational `svek` gaps when Java tracing proves the drift is below parser/layout.
- Treat baseline freeze as first-order work, not bookkeeping.
- `SVG0004_*` and `SVG0005_*` are now mainly regression guards for the recent `svek` and serialization fixes.
- `hideshow002` remains a useful class/group offset guard, but it is no longer the best active representative for the class-edge tail.
- The next active class-side representative is `tests/fixtures/class/qualifiedassoc002.puml`, with `qualifiedassoc001` as the sibling guard.
- The next active description/component representatives should come from still-failing component fixtures, not from `SVG0005_*`.

## Fastest Path Now

- First freeze one authority baseline and one authority failure list.
- Then split work explicitly:
  - use `tests/fixtures/class/qualifiedassoc002.puml` and `tests/fixtures/class/qualifiedassoc001.puml` as the current class-edge / shield-port representatives
  - keep `tests/fixtures/class/hideshow002.puml` and `tests/fixtures/class/hideshow003.puml` as the class-group / cluster-shape guards
  - use still-failing component/description fixtures as the shared text/body representatives
- Attack shared `Display/Text/body` semantics in parser plus layout before touching more downstream SVG serialization.
- In parallel, keep `svek` coordinate reconstruction honest:
  - `LimitFinder` span must follow Java shape semantics, not raw cluster bounds
  - rectangle-like clusters must contribute `x-1/y-1/...-1` the same way Java `drawRectangle()` does
  - package/path-like clusters must not be forced through the rectangle approximation
- In parallel, finish the class-edge SVG handoff from `svek`:
  - preserve shield-node endpoint ports in DOT
  - preserve Graphviz end-arrow polygon/tip data through `svek -> GraphLayout`
  - stop rotating class triangle/diamond heads with the extra `+PI/2` that Java already cancels in the factory layer
  - keep Java `LinkStrategy.SIMPLIER` scoped to the class path until component/state no longer depend on legacy DOT-arrow behavior
- In parallel, treat component ports as a separate upstream `svek` problem:
  - port node DOT shape must follow Java `RectanglePort`
  - cluster DOT must follow Java `ClusterDotString` source/sink/empty-node behavior when non-normal positions exist
  - SVG metadata must use qualified names for nested component ports
- Use `SVG0004_*` and `SVG0005_*` as guard rails to ensure the already-fixed interactive, self-loop, and `SvgResult` transform chain do not regress.
- Keep sequence work behind shared text/body work unless a Java trace proves a truly independent sequence-only divergence.

## Non-Negotiable Rules

- Always choose representative fixtures for a cluster before editing code.
- Always trace both Rust and Java for the chosen representative.
- Always fix at the earliest divergence point.
- Always rerun `cargo test --lib` before a full reference run.
- Always record pass count before and after each cluster fix.
- Never patch downstream SVG output if the divergence started in parser or layout.

## Workstream 0: Freeze Baseline And Cluster Map

- [ ] Pick one authority baseline run and record it explicitly before further fixes.
- [ ] Run `python3 scripts/analyze_failures.py --quick` and save the current cluster summary in the work log.
- [ ] Deduplicate mirrored fixtures mentally before planning work. Many failures are duplicated across `preprocessor`, `nonreg`, and main fixture trees.
- [ ] Keep one representative list per cluster instead of tracking 149 raw test names.
- [ ] Use the full reference suite as the only real success metric.
- [ ] Compare future runs against the frozen failure set, not against memory or mixed historical counts.

Current note:

- Focused runs may already pass `SVG0004_*`, but that does not establish a new suite baseline by itself.
- Do not update the baseline again until the full-suite context is stable and reproducible.

Representative clusters to track first:

- Shared text/display height cluster
- `svek` LimitFinder / cluster-shape semantics cluster
- Component port/group DOT semantics cluster
- Activity plus sprite-transform cluster
- Sequence Teoz vertical cluster
- Sequence left self-message width cluster
- Subdiagram theme misclassification cluster
- CHEN_EER vertical expansion cluster
- State/timing/usecase/yaml tail cluster

## Workstream 1: Unify Display And Text Semantics

Goal:

- Build one canonical rule set for how PlantUML text becomes visual lines and visual height.

Why this is first:

- This is still the highest-leverage root cause.
- It cuts across `CLASS`, `DESCRIPTION`, `ACTIVITY`, `SEQUENCE`, `STATE`, and some sprite-related cases.
- `SVG0005_*` is no longer the active failing representative after the recent `svek` repair, so do not keep using it as the only driver.
- The remaining text/body work now needs to be driven by still-failing description/component fixtures plus the class-group tail in `hideshow002`.

Files to audit first:

- `src/layout/mod.rs`
- `src/render/svg.rs`
- `src/render/svg_richtext.rs`
- `src/parser/class.rs`
- `src/parser/component.rs`
- `src/layout/sequence.rs`

Known split points already diverging:

- Member line splitting in `src/layout/mod.rs`
- Class description block handling in `src/parser/class.rs`
- Component description block handling in `src/parser/component.rs`
- Rich text line height in `src/render/svg_richtext.rs`
- Sequence sprite extra-height logic in `src/layout/sequence.rs`
- Class name measurement and rendering in `src/layout/mod.rs` and `src/render/svg.rs`

Execution checklist:

- [ ] Define the exact Java-compatible behavior for literal `\n`.
- [ ] Define the exact Java-compatible behavior for physical newlines.
- [ ] Define the exact Java-compatible behavior for `U+E100` from `%newline()`.
- [ ] Define the exact Java-compatible behavior for `%chr(10)`.
- [ ] Define the exact Java-compatible behavior for `\l` and `\r`.
- [ ] Define the exact Java-compatible behavior for creole `<size>`, `<sub>`, and `<sup>`.
- [ ] Define the exact Java-compatible behavior for inline SVG sprite height inside text.
- [ ] Move duplicated split-and-measure logic toward one shared model.

Representative fixtures:

- `tests/fixtures/nonreg/svg/SVG0004_Smetana.puml`
- `tests/fixtures/nonreg/svg/SVG0005_Smetana.puml`
- `tests/fixtures/nonreg/svg/SVG0005_Svek.puml`
- `tests/fixtures/component/colors001.puml`
- `tests/fixtures/class/qualifiedassoc002.puml`
- `tests/fixtures/component/deployment01.puml`
- `tests/fixtures/component/jaws5.puml`

Success criteria:

- These representatives pass without introducing a regression in already-passing text-heavy cases.
- `SVG0005_*` should collapse toward Java height before sequence work is revisited.

Current focused note:

- The recent `svek` review found a deeper transform bug than expected:
  - Rust `SvgResult.substring()`, `PointListIterator`, and `extract_dot_path()` were dropping the active `Point2DFunction`
  - Java preserves that transform through every substring-based parsing path
- That bug is now fixed in `src/svek/svg_result.rs`.
- Focused verification after that fix:
  - `SVG0005_Smetana` passes
  - `SVG0005_Svek` passes
  - `hideshow003` passes
- The active class-edge root cause has since moved again:
  - shielded nodes now emit `:h` ports correctly in DOT
  - `qualifiedassoc001/002` improved from `+2px` width drift to `+1px`
  - `Map -> HashMap` is now anchored on the correct vertical column
- The remaining `qualifiedassoc` tail is now split in two low-level pieces:
  - class triangle/diamond arrowheads are rotated with an extra quarter-turn compared with Java factory semantics
  - Graphviz end-arrow polygon/tip data is not preserved all the way through `svek -> GraphLayout -> render`, so right-end arrowheads still use the path endpoint as the tip
- The current execution boundary is:
  - class diagrams now intentionally use Java `LinkStrategy.SIMPLIER` in the DOT builder
  - component/state still remain on the legacy DOT-arrow path until their own `svek` handoff is fully aligned
- `hideshow002` still matters, but it is no longer the first class target while this class-edge handoff remains unfinished.

## Workstream 1B: Svek LimitFinder And Cluster-Shape Semantics

Goal:

- Finish the low-level Java `LimitFinder` / `SvekResult` shape semantics so class/group alignment does not depend on downstream offset patches.

Why this is priority:

- This is now a confirmed foundational gap, not a fixture-specific tail.
- Java tracing for `hideshow002` and `hideshow003` shows the remaining divergence is below parser/body semantics.
- The current Rust path reconstructs `LimitFinder` span in `src/svek/mod.rs`, but cluster participation is still only a partial approximation.

Files to audit first:

- `src/svek/mod.rs`
- `src/svek/cluster.rs`
- `src/layout/graphviz.rs`
- `src/render/svg.rs`

Representative fixtures:

- `tests/fixtures/class/qualifiedassoc001.puml`
- `tests/fixtures/class/qualifiedassoc002.puml`
- `tests/fixtures/class/hideshow002.puml`
- `tests/fixtures/class/hideshow003.puml`
- `tests/fixtures/nonreg/svg/SVG0006_Svek.puml`

Execution checklist:

- [ ] Confirm which cluster shapes in Java contribute through `drawRectangle()` versus path geometry.
- [ ] Reconstruct Java `LimitFinder` min/max using cluster style semantics, not just raw `cluster.x/y/width/height`.
- [ ] Recheck the relationship between `moveDelta`, `normalize_offset`, and `render_offset` after the cluster fix.
- [ ] Preserve Graphviz end-arrow polygon/tip data for class edges instead of dropping it when converting `svek` output into `GraphLayout`.
- [ ] Align class triangle/diamond arrowhead rotation with Java `ExtremityFactoryExtends` / `ExtremityFactoryDiamond` semantics.
- [ ] Keep `SVG0004_*` and `SVG0005_*` green while adjusting `svek`.

Success criteria:

- `hideshow002` and `hideshow003` stop failing as generic class/group offset cases.
- The `qualifiedassoc*` and `SVG0006_Svek` class-group tails move with the same fix instead of needing per-fixture handling.
- The remaining class-side failures become dominated by text/body semantics rather than by post-`svek` coordinate reconstruction.

## Workstream 2: Parser Consistency For Bracket Bodies, Names, And Notes

Goal:

- Remove parser-level semantic drift before touching more renderer code.

Immediate checks:

- [ ] Recheck component bracket-body handling. It currently uses the wrong newline expander for Java compatibility.
- [ ] Verify class bracket-body handling stays aligned with Java semantics.
- [ ] Verify multi-line entity names preserve display semantics consistently across class and component paths.
- [ ] Verify note text and body text are not accidentally normalized the same way when Java treats them differently.

Files:

- `src/parser/component.rs`
- `src/parser/class.rs`
- `src/parser/common.rs`

Representative fixtures:

- `tests/fixtures/component/colors001.puml`
- `tests/fixtures/component/componentextraarrows_0001.puml`
- `tests/fixtures/dev/newline/subdiagram_theme.puml`

Success criteria:

- Parser output matches Java's display line structure before layout is even considered.

## Workstream 3: Class And Description Measurement/Render Convergence

Goal:

- Make class names, stereotypes, rectangle bodies, and description text use the same visual-line model in layout and render.

Current signs of drift:

- Class names are still often measured as a single line.
- Some description-heavy outputs are too short by a large fixed amount.
- Some description-heavy outputs are too tall because body semantics diverged before render.

Files:

- `src/layout/mod.rs`
- `src/render/svg.rs`
- `src/layout/component.rs`
- `src/render/svg_component.rs`

Representative fixtures:

- `tests/fixtures/nonreg/svg/SVG0004_Smetana.puml`
- `tests/fixtures/nonreg/svg/SVG0004_Svek.puml`
- `tests/fixtures/nonreg/svg/SVG0005_Smetana.puml`
- `tests/fixtures/nonreg/svg/SVG0005_Svek.puml`
- `tests/fixtures/class/hideshow002.puml`
- `tests/fixtures/class/hideshow003.puml`
- `tests/fixtures/class/qualifiedassoc001.puml`
- `tests/fixtures/class/qualifiedassoc002.puml`
- `tests/fixtures/component/deployment01.puml`
- `tests/fixtures/component/jaws5.puml`

Success criteria:

- The large `CLASS` and `DESCRIPTION` height clusters shrink materially after one coherent fix.
- `SVG0004_*` should remain green as a regression check while `SVG0005_*` becomes the primary forward driver.

## Workstream 3B: Component Port And Group DOT Semantics

Goal:

- Port `DESCRIPTION` diagrams must follow Java `svek` cluster semantics instead of plain Graphviz cluster bounds.

Why this is separate:

- This is upstream layout behavior, not downstream SVG serialization.
- `SVG0005_*` is currently green in focused runs, so treat it as a regression guard rather than proof that this workstream is done.
- Remaining component/description failures still need the Java `port/group` semantics to be finished.

Files:

- `src/layout/component.rs`
- `src/layout/graphviz.rs`
- `src/svek/builder.rs`
- `src/svek/mod.rs`
- `src/svek/node.rs`
- `src/render/svg_component.rs`

Representative fixtures:

- `tests/fixtures/nonreg/svg/SVG0005_Smetana.puml`
- `tests/fixtures/nonreg/svg/SVG0005_Svek.puml`
- `tests/fixtures/component/deployment01.puml`
- `tests/fixtures/component/jaws5.puml`

Execution checklist:

- [ ] Carry parent-qualified names through component layout/render for nested ports.
- [ ] Pass Java-like `RectanglePort` shape and max label width into `svek`.
- [ ] Port Java `ClusterDotString` source/sink/empty-node behavior for clusters with non-normal positions.
- [ ] Recheck whether port entity comments should be omitted for Java parity.
- [ ] Recheck port label above/below placement against parent cluster center.

Success criteria:

- `SVG0005_*` no longer behaves like a raw cluster-bounds failure.
- Port metadata, cluster height, and port placement move together toward Java rather than improving one axis at a time.

Current focused note:

- The recent `svek` repair restored Java-like transform propagation, but that was not the last component-side bug.
- A second low-level mismatch was found in the current workspace:
  - component `raw_path_d` and solved bezier `points` were not in the same coordinate space
  - the symptom was a stable `+6,+6` path drift on `SVG0005_*` while the arrow polygon still matched Java
  - the current fix aligns component `raw_path_d` to the solved bezier start point before applying the renderer margin
  - after that alignment, both `SVG0005_Smetana` and `SVG0005_Svek` are green again
- Do not infer from that that component port semantics are complete.
- The remaining work is now the unfinished Java behavior, not the broken transform chain:
  - qualified names for nested ports
  - Java `RectanglePort` sizing and placement
  - Java `ClusterDotString` and `FrontierCalculator` behavior for non-normal component nodes
  - parent/child group semantics that affect cluster bounds before render

## Workstream 4: Activity And Sprite Transform Cluster

Goal:

- Resolve the shared activity and sprite-transform failures without mixing them into sequence work.

Observed patterns:

- Small but repeated width drift in `swimlane001` and `a0002`
- Large repeated height drift in `activity_creole_table_02`
- A stable `-79px` cluster in sprite transform fixtures

Current status:

- The known `svgTransformGroup/Matrix/Rotate/Scale/Translate` reference fixtures are now green.
- The remaining review risk is coverage, not the currently failing baseline:
  - `src/render/svg_sprite.rs` still only converts `rect`, `circle`, and `line` through the affine path
  - unsupported affine-transformed tags are skipped with a warning today
  - do not treat the sprite-transform cluster as fully closed until affine support covers the rest of the element set Java accepts

Files:

- `src/layout/activity.rs`
- `src/render/svg_activity.rs`
- `src/render/svg_sprite.rs`
- `src/render/svg_richtext.rs`

Representative fixtures:

- `tests/fixtures/activity/swimlane001.puml`
- `tests/fixtures/activity/a0002.puml`
- `tests/fixtures/activity/activity_creole_table_02.puml`
- `tests/fixtures/sprite/svgTransformGroup.puml`
- `tests/fixtures/sprite/svgTransformMatrix.puml`
- `tests/fixtures/sprite/svgTransformRotate.puml`
- `tests/fixtures/sprite/svgTransformScale.puml`
- `tests/fixtures/sprite/svgTransformTranslate.puml`

Success criteria:

- Width drift in the activity pair is eliminated.
- The sprite transform cluster no longer fails with the shared height drop.

## Workstream 5: Sequence Work, Split Into Three Independent Queues

Do not run sequence as one large bucket.

### Queue 5A: Teoz Vertical Packing

- [ ] Trace Java tile heights and vertical stacking order.
- [ ] Focus on message text height, fragment header height, note height, and spacing accumulation.

Files:

- `src/layout/sequence_teoz/builder.rs`
- `src/layout/sequence_teoz/tiles.rs`
- `src/render/svg_sequence.rs`

Representative fixtures:

- `tests/fixtures/nonreg/simple/TeozAltElseParallel_0001.puml`
- `tests/fixtures/nonreg/simple/TeozAltElseParallel_0002.puml`
- `tests/fixtures/nonreg/simple/TeozAltElseParallel_0003.puml`
- `tests/fixtures/nonreg/simple/TeozTimelineIssues_0001.puml`
- `tests/fixtures/nonreg/simple/TeozTimelineIssues_0002.puml`
- `tests/fixtures/nonreg/simple/TeozTimelineIssues_0004.puml`

### Queue 5B: Left Self-Message Width And Coordinate Drift

- [ ] Isolate self-message contact point logic.
- [ ] Recheck width growth for left self-message plus max message size interaction.

Files:

- `src/layout/sequence.rs`
- `src/layout/sequence_teoz/builder.rs`
- `src/render/svg_sequence.rs`

Representative fixtures:

- `tests/fixtures/sequence/sequencelayout_0001c.puml`
- `tests/fixtures/sequence/sequencelayout_0003.puml`
- `tests/fixtures/sequence/sequenceleftmessageandactivelifelines_0001.puml`
- `tests/fixtures/sequence/sequenceleftmessageandactivelifelines_0002.puml`
- `tests/fixtures/sequence/sequenceleftmessageandactivelifelines_0003.puml`

### Queue 5C: Interactive, Style, And URL Output

- [ ] Recheck style emission ordering and interactive SVG additions.
- [ ] Recheck URL tooltip and arrow-style serialization.

Files:

- `src/render/svg_sequence.rs`
- `src/render/svg_richtext.rs`
- `src/render/svg_hyperlink.rs`

Representative fixtures:

- `tests/fixtures/dev/jaws/jaws11.puml`
- `tests/fixtures/nonreg/simple/SequenceArrows_0001.puml`
- `tests/fixtures/nonreg/simple/SequenceArrows_0002.puml`
- `tests/fixtures/misc/link_url_tooltip_04.puml`
- `tests/fixtures/misc/link_url_tooltip_05.puml`

Success criteria:

- Sequence failures stop behaving like three unrelated subproblems.

## Workstream 6: Subdiagram Theme Misclassification

Goal:

- Stop nested subdiagram content from hijacking outer diagram routing or outer `data-diagram-type`.

Files to inspect first:

- `src/lib.rs`
- `src/parser/common.rs`
- `src/parser/mod.rs`
- `src/preproc/mod.rs`

Representative fixtures:

- `tests/fixtures/component/subdiagram_theme_02.puml`
- `tests/fixtures/dev/newline/subdiagram_theme.puml`
- `tests/fixtures/preprocessor/subdiagram_theme_01.puml`

Execution checklist:

- [ ] Verify what source string reaches `parser::parse_with_original`.
- [ ] Verify whether nested `{{ ... }}` content is leaking into top-level diagram detection.
- [ ] Verify whether theme expansion changes outer diagram heuristics.
- [ ] Verify outer diagram type remains `CLASS` where Java does.

Success criteria:

- Diagram type matches Java and height collapses back toward the expected range.

## Workstream 7: Tail Clusters After Shared Fixes

Do not touch these before the shared text/display work unless a Java trace shows they are independent.

Cluster list:

- CHEN_EER vertical expansion
- STATE small and medium height offsets
- TIMING label/baseline offsets
- USECASE final sizing
- JSON, YAML, SALT one-offs

Representative fixtures:

- `tests/fixtures/erd/chenmoviealias.puml`
- `tests/fixtures/erd/chenmovieextended.puml`
- `tests/fixtures/erd/chenmovie.puml`
- `tests/fixtures/state/scxml0002.puml`
- `tests/fixtures/state/scxml0003.puml`
- `tests/fixtures/state/scxml0004.puml`
- `tests/fixtures/state/scxml0005.puml`
- `tests/fixtures/timing/timingmessagearrowfont_0001.puml`
- `tests/fixtures/timing/timingmessagearrowfont_0002.puml`
- `tests/fixtures/usecase/basic.puml`
- `tests/fixtures/usecase/boundary.puml`
- `tests/fixtures/usecase/colon_actor.puml`
- `tests/fixtures/json/json_escaped.puml`
- `tests/fixtures/yaml/basic.puml`
- `tests/fixtures/salt/basic.puml`

## Suggested Parallel Ownership

- Worker A: shared display semantics, class, component description, rich text
- Worker B: activity and sprite transform
- Worker C: sequence queues
- Worker D: subdiagram theme routing and parser detection
- Main thread: integration, conflict resolution, and full-suite verification

Avoid overlapping write sets where possible:

- Worker A should own `src/layout/mod.rs`, `src/render/svg.rs`, `src/render/svg_richtext.rs`, `src/parser/class.rs`, `src/parser/component.rs`
- Worker B should own `src/layout/activity.rs`, `src/render/svg_activity.rs`, `src/render/svg_sprite.rs`
- Worker C should own `src/layout/sequence.rs`, `src/layout/sequence_teoz/*`, `src/render/svg_sequence.rs`
- Worker D should own `src/lib.rs`, `src/parser/common.rs`, `src/parser/mod.rs`, `src/preproc/mod.rs`

## Verification Loop

- [ ] Run `cargo test --lib`
- [ ] Run focused reference tests for the cluster being changed
- [ ] Run `cargo test --test reference_tests`
- [ ] Run `python3 scripts/analyze_failures.py --quick`
- [ ] Record new pass count
- [ ] Record whether one whole cluster disappeared or shrank

## Priority Order

- Priority 1: shared `Display/Text/body` semantics across parser, layout, and render
- Priority 2: `svek` `LimitFinder` / `SvekResult` / cluster-shape semantics
- Priority 3: component port/group DOT semantics
- Priority 4: sequence core layout primitives (`Teoz` vertical packing, then left self-message width)
- Priority 5: sprite renderer affine/tag coverage

Not first-priority for now:

- `CHEN_EER` expansion
- small `STATE` / `TIMING` / `JSON` / `YAML` tails
- isolated style-only and stroke-width-only diffs
- `USECASE` sizing tails until the shared higher-leverage workstreams are exhausted

## Definition Of Real Progress

- A real fix removes a repeated cluster, not just one named fixture.
- A real fix raises the pass count or removes one full class of divergence.
- If a change only moves failures around, it is not done.
- If pass counts come from different baselines, they do not count as evidence.
