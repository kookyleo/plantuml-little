# Diagnosis Report: tests/fixtures/state/state_note001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/state/state_note001.puml`
- Reference test: `reference_fixtures_state_state_note001_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`
- Authority tier: `reference-test`
- Worktree: `dirty` (21 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '385px', 'height': '368px'} elements={'rect': 4, 'path': 8, 'text': 7, 'ellipse': 1, 'polygon': 3, 'group': 10}
- java: viewport={'width': '371px', 'height': '366px'} elements={'rect': 3, 'path': 8, 'text': 6, 'ellipse': 2, 'polygon': 3, 'group': 10}
- reference: viewport={'width': '371px', 'height': '366px'} elements={'rect': 3, 'path': 8, 'text': 6, 'ellipse': 2, 'polygon': 3, 'group': 10}

## Final Diffs

- Raw first diff: target=`reference` at `1:148`
- Raw context: `expected=...ss" data-diagram-type="STATE" height="366px" preserveAspectRatio="none" style="w... actual=...ss" data-diagram-type="STATE" height="368px" preserveAspectRatio="none" style="w...`
- Semantic first diff: target=`reference` at `1:148`
- Semantic context: `expected=...ss" data-diagram-type="STATE" height="366px" preserveAspectRatio="none" style="w... actual=...ss" data-diagram-type="STATE" height="368px" preserveAspectRatio="none" style="w...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '366px', 'preserveAspectRatio': 'none', 'style': 'width:371px;height:366px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 371 366', 'width': '371px', 'zoomAndPan': 'magnify'}, 'text': 'ActiveRunningInactiveThis is activeMulti linenote text'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '368px', 'preserveAspectRatio': 'none', 'style': 'width:385px;height:368px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 385 368', 'width': '385px', 'zoomAndPan': 'magnify'}, 'text': 'Active[*]RunningInactive\n\nThis is active\n\nMulti linenote text'}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=+14, dh=+2`
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

thread 'reference_fixtures_state_state_note001_puml' (4124784) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/state/state_note001.puml: output differs from reference at line 1 col 148
expected: ...ss" data-diagram-type="STATE" height="366px" preserveAspectRatio="none" style="w...
actual:   ...ss" data-diagram-type="STATE" height="368px" preserveAspectRatio="none" style="w...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dot_input, layout.dot_output, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/LimitFinder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-state-note001-iter3/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Element structure drift`, then add stage-boundary traces if the first diff is still ambiguous.

