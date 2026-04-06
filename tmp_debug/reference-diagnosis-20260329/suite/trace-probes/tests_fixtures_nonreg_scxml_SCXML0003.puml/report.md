# Diagnosis Report: tests/fixtures/nonreg/scxml/SCXML0003.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/nonreg/scxml/SCXML0003.puml`
- Reference test: `reference_fixtures_nonreg_scxml_SCXML0003_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`
- Authority tier: `reference-test`
- Worktree: `dirty` (9 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '1360px', 'height': '430px'} elements={'rect': 23, 'path': 13, 'text': 41, 'ellipse': 2, 'polygon': 9, 'group': 31}
- reference: viewport={'width': '1177px', 'height': '436px'} elements={'rect': 20, 'path': 13, 'text': 29, 'ellipse': 2, 'polygon': 9, 'group': 23}

## Final Diffs

- Raw first diff: target=`reference` at `1:148`
- Raw context: `expected=...ss" data-diagram-type="STATE" height="436px" preserveAspectRatio="none" style="w... actual=...ss" data-diagram-type="STATE" height="430px" preserveAspectRatio="none" style="w...`
- Semantic first diff: target=`reference` at `1:148`
- Semantic context: `expected=...ss" data-diagram-type="STATE" height="436px" preserveAspectRatio="none" style="w... actual=...ss" data-diagram-type="STATE" height="430px" preserveAspectRatio="none" style="w...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '436px', 'preserveAspectRatio': 'none', 'style': 'width:1177px;height:436px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 1177 436', 'width': '1177px', 'zoomAndPan': 'magnify'}, 'text': 'moduleSompflopcounterexexitAxentry1entry2sinsin2sig_insig_ffflop_0sig_ff := 0flop_1sig_ff := 1count_startcount_donecount_val[3:0]count_idlecount_val := 0count_ongoingcount_val := count_val +1count_finishcount_done:=1sig_incount_startcount_val != MAX_VAL"!"'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '430px', 'preserveAspectRatio': 'none', 'style': 'width:1360px;height:430px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 1360 430', 'width': '1360px', 'zoomAndPan': 'magnify'}, 'text': 'moduleSomp«inputPin»entry1«inputPin»entry2sinsin2flop«inputPin»sig_in«outputPin»sig_ffflop_0sig_ff := 0flop_1sig_ff := 1counter«inputPin»count_start«outputPin»count_done«outputPin»count_val[3:0]count_idlecount_val := 0count_ongoingcount_val := count_val +1count_finishcount_done:=1«inputPin»ex«inputPin»exitAxentry1sig_ffentry2sig_incount_startcount_val != MAX_VAL"!"'}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=+183, dh=-6`
- First coordinate signal: `height`
- Underlying signals: `element-structure-drift`

## Fix Suggestions

- Element structure drift (high): Rendered element counts already diverge, so compare earlier render structure or shared text assembly before post-dot coordinate extraction.
  files: `/ext/plantuml/plantuml-little/src/render/svg_state.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`

## Code Anchors

- Graphviz/Svek core: Graphviz-backed diagrams usually diverge in SvekResult bounds, LimitFinder participation, or final SVG coordinate normalization.
  java: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/LimitFinder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`
  rust: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Element structure drift: Element counts already diverge before the first visible coordinate diff, so inspect earlier render structure or shared text assembly before post-dot normalization.
  java: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/TextBlockExporter12026.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/shape/TextBlock.java`
  rust: `/ext/plantuml/plantuml-little/src/render/svg.rs`, `/ext/plantuml/plantuml-little/src/render/svg_richtext.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_nonreg_scxml_SCXML0003_puml' (1940846) panicked at tests/reference_tests.rs:307:9:
tests/fixtures/nonreg/scxml/SCXML0003.puml: output differs from reference at line 1 col 148
expected: ...ss" data-diagram-type="STATE" height="436px" preserveAspectRatio="none" style="w...
actual:   ...ss" data-diagram-type="STATE" height="430px" preserveAspectRatio="none" style="w...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_scxml_SCXML0003.puml/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dot_input, layout.dot_output, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/LimitFinder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_scxml_SCXML0003.puml/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_scxml_SCXML0003.puml/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_scxml_SCXML0003.puml/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_scxml_SCXML0003.puml/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_scxml_SCXML0003.puml/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Element structure drift`, then add stage-boundary traces if the first diff is still ambiguous.

