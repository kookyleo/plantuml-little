# Reference Alignment Execution TODO

## Current Baseline

- Reference baseline on 2026-03-26: `147 passed / 149 failed / 296 total`
- The residual failures are still dominated by SVG height mismatches.
- The current bottleneck is no longer the old `SvekResult` or `ensureVisible` constant layer.
- The primary bottleneck is fragmented display/text semantics across parser, layout, and render.

## Core Conclusion

- Do not keep attacking single failing fixtures one by one.
- Do not start from viewport constants unless Java tracing proves a viewport-only divergence.
- Treat the remaining failures as a small number of repeated root-cause clusters.
- Fix shared semantics first, then consume the cluster-specific tails.

## Non-Negotiable Rules

- Always choose representative fixtures for a cluster before editing code.
- Always trace both Rust and Java for the chosen representative.
- Always fix at the earliest divergence point.
- Always rerun `cargo test --lib` before a full reference run.
- Always record pass count before and after each cluster fix.
- Never patch downstream SVG output if the divergence started in parser or layout.

## Workstream 0: Freeze Baseline And Cluster Map

- [ ] Run `python3 scripts/analyze_failures.py --quick` and save the current cluster summary in the work log.
- [ ] Deduplicate mirrored fixtures mentally before planning work. Many failures are duplicated across `preprocessor`, `nonreg`, and main fixture trees.
- [ ] Keep one representative list per cluster instead of tracking 149 raw test names.
- [ ] Use the full reference suite as the only real success metric.

Representative clusters to track first:

- Shared text/display height cluster
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

- This is the highest-leverage root cause.
- It cuts across `CLASS`, `DESCRIPTION`, `ACTIVITY`, `SEQUENCE`, `STATE`, and some sprite-related cases.

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
- `tests/fixtures/component/colors001.puml`
- `tests/fixtures/class/hideshow003.puml`

Success criteria:

- These representatives pass without introducing a regression in already-passing text-heavy cases.

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

## Workstream 4: Activity And Sprite Transform Cluster

Goal:

- Resolve the shared activity and sprite-transform failures without mixing them into sequence work.

Observed patterns:

- Small but repeated width drift in `swimlane001` and `a0002`
- Large repeated height drift in `activity_creole_table_02`
- A stable `-79px` cluster in sprite transform fixtures

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

## Definition Of Real Progress

- A real fix removes a repeated cluster, not just one named fixture.
- A real fix raises the pass count or removes one full class of divergence.
- If a change only moves failures around, it is not done.
