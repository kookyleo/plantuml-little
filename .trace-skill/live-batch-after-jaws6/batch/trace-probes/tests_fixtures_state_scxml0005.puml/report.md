# Diagnosis Report: tests/fixtures/state/scxml0005.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/state/scxml0005.puml`
- Reference test: `reference_fixtures_state_scxml0005_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`
- Authority tier: `reference-test`
- Worktree: `dirty` (17 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '320px', 'height': '74px'} elements={'rect': 1, 'path': 2, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}
- reference: viewport={'width': '332px', 'height': '71px'} elements={'rect': 1, 'path': 2, 'text': 3, 'ellipse': 0, 'polygon': 0, 'group': 3}

## Final Diffs

- Raw first diff: target=`reference` at `1:147`
- Raw context: `expected=...css" data-diagram-type="STATE" height="71px" preserveAspectRatio="none" style="w... actual=...css" data-diagram-type="STATE" height="74px" preserveAspectRatio="none" style="w...`
- Semantic first diff: target=`reference` at `1:147`
- Semantic context: `expected=...css" data-diagram-type="STATE" height="71px" preserveAspectRatio="none" style="w... actual=...css" data-diagram-type="STATE" height="74px" preserveAspectRatio="none" style="w...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '71px', 'preserveAspectRatio': 'none', 'style': 'width:332px;height:71px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 332 71', 'width': '332px', 'zoomAndPan': 'magnify'}, 'text': 'modulelocalparam MAX_VAL 10parameter COUNT_WIDTH 4'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '74px', 'preserveAspectRatio': 'none', 'style': 'width:320px;height:74px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 320 74', 'width': '320px', 'zoomAndPan': 'magnify'}, 'text': 'module\n\nlocalparam\xa0MAX_VAL\xa010parameter\xa0COUNT_WIDTH\xa04'}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=-12, dh=+3`
- First coordinate signal: `height`
- Underlying signals: `element-structure-drift`

## Fix Suggestions

- Stage trace first (low): No strong heuristic matched. Add stage-boundary JSONL traces around the detected family, then compare the first divergent stage.
  files: `/ext/plantuml/plantuml-little/src/lib.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Code Anchors

- Graphviz/Svek core: Graphviz-backed diagrams usually diverge in SvekResult bounds, LimitFinder participation, or final SVG coordinate normalization.
  java: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/LimitFinder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`
  rust: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_state_scxml0005_puml' (4108683) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/state/scxml0005.puml: output differs from reference at line 1 col 147
expected: ...css" data-diagram-type="STATE" height="71px" preserveAspectRatio="none" style="w...
actual:   ...css" data-diagram-type="STATE" height="74px" preserveAspectRatio="none" style="w...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_state_scxml0005.puml/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dot_input, layout.dot_output, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/LimitFinder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_state_scxml0005.puml/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_state_scxml0005.puml/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_state_scxml0005.puml/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_state_scxml0005.puml/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_state_scxml0005.puml/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Stage trace first`, then add stage-boundary traces if the first diff is still ambiguous.

