# Diagnosis Report: tests/fixtures/state/state_monoline_03.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/state/state_monoline_03.puml`
- Reference test: `reference_fixtures_state_state_monoline_03_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`
- Authority tier: `reference-test`
- Worktree: `dirty` (9 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '274px', 'height': '287px'} elements={'rect': 2, 'path': 4, 'text': 9, 'ellipse': 3, 'polygon': 4, 'group': 9}
- reference: viewport={'width': '274px', 'height': '288px'} elements={'rect': 2, 'path': 4, 'text': 9, 'ellipse': 3, 'polygon': 4, 'group': 9}

## Final Diffs

- Raw first diff: target=`reference` at `1:148`
- Raw context: `expected=...ss" data-diagram-type="STATE" height="288px" preserveAspectRatio="none" style="w... actual=...ss" data-diagram-type="STATE" height="287px" preserveAspectRatio="none" style="w...`
- Semantic first diff: target=`reference` at `1:148`
- Semantic context: `expected=...ss" data-diagram-type="STATE" height="288px" preserveAspectRatio="none" style="w... actual=...ss" data-diagram-type="STATE" height="287px" preserveAspectRatio="none" style="w...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '288px', 'preserveAspectRatio': 'none', 'style': 'width:274px;height:288px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 274 288', 'width': '274px', 'zoomAndPan': 'magnify'}, 'text': 'State1this is a stringaddingsome code:main() {printf("Hello world");}State2'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '287px', 'preserveAspectRatio': 'none', 'style': 'width:274px;height:287px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 274 287', 'width': '274px', 'zoomAndPan': 'magnify'}, 'text': 'State1this is a stringaddingsome code:main() {printf("Hello world");}State2'}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dh=-1`
- First coordinate signal: `height`
- Underlying signals: `graphviz-coordinate-chain`

## Fix Suggestions

- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

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

thread 'reference_fixtures_state_state_monoline_03_puml' (1940800) panicked at tests/reference_tests.rs:307:9:
tests/fixtures/state/state_monoline_03.puml: output differs from reference at line 1 col 148
expected: ...ss" data-diagram-type="STATE" height="288px" preserveAspectRatio="none" style="w...
actual:   ...ss" data-diagram-type="STATE" height="287px" preserveAspectRatio="none" style="w...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_state_state_monoline_03.puml/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dot_input, layout.dot_output, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/LimitFinder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_state_state_monoline_03.puml/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_state_state_monoline_03.puml/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_state_state_monoline_03.puml/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_state_state_monoline_03.puml/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_state_state_monoline_03.puml/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Graphviz coordinate chain`, then add stage-boundary traces if the first diff is still ambiguous.

