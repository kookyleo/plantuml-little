# Diagnosis Report: tests/fixtures/state/state_concurrent001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/state/state_concurrent001.puml`
- Reference test: `reference_fixtures_state_state_concurrent001_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`
- Authority tier: `reference-test`
- Worktree: `dirty` (15 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '105px', 'height': '628px'} elements={'rect': 5, 'path': 3, 'text': 5, 'ellipse': 3, 'polygon': 2, 'group': 10}
- reference: viewport={'width': '113px', 'height': '630px'} elements={'rect': 5, 'path': 7, 'text': 5, 'ellipse': 5, 'polygon': 6, 'group': 15}

## Final Diffs

- Raw first diff: target=`reference` at `1:147`
- Raw context: `expected=...css" data-diagram-type="STATE" height="630px" preserveAspectRatio="none" style="... actual=...css" data-diagram-type="STATE" height="628px" preserveAspectRatio="none" style="...`
- Semantic first diff: target=`reference` at `1:147`
- Semantic context: `expected=...css" data-diagram-type="STATE" height="630px" preserveAspectRatio="none" style="... actual=...css" data-diagram-type="STATE" height="628px" preserveAspectRatio="none" style="...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '630px', 'preserveAspectRatio': 'none', 'style': 'width:113px;height:630px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 113 630', 'width': '113px', 'zoomAndPan': 'magnify'}, 'text': 'ActiveSub1Sub2Sub3Sub4'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'STATE', 'height': '628px', 'preserveAspectRatio': 'none', 'style': 'width:105px;height:628px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 105 628', 'width': '105px', 'zoomAndPan': 'magnify'}, 'text': 'ActiveSub1Sub2Sub3Sub4'}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=-8, dh=-2`
- First coordinate signal: `cx -0.2231`
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

thread 'reference_fixtures_state_state_concurrent001_puml' (4098390) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/state/state_concurrent001.puml: output differs from reference at line 1 col 147
expected: ...css" data-diagram-type="STATE" height="630px" preserveAspectRatio="none" style="...
actual:   ...css" data-diagram-type="STATE" height="628px" preserveAspectRatio="none" style="...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_state_state_concurrent001.puml/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dot_input, layout.dot_output, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dot_input,layout.dot_output,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/SvekResult.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/LimitFinder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/klimt/drawing/svg/SvgGraphics.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_state_state_concurrent001.puml/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_state_state_concurrent001.puml/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_state_state_concurrent001.puml/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_state_state_concurrent001.puml/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_state_state_concurrent001.puml/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Graphviz coordinate chain`, then add stage-boundary traces if the first diff is still ambiguous.

