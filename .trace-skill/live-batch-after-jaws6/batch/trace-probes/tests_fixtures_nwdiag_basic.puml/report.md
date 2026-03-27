# Diagnosis Report: tests/fixtures/nwdiag/basic.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/nwdiag/basic.puml`
- Reference test: `reference_fixtures_nwdiag_basic_puml`
- Family: `self-layout`
- Diagram type: `NWDIAG`
- Authority tier: `reference-test`
- Worktree: `dirty` (17 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '671px', 'height': '276px'} elements={'rect': 6, 'path': 0, 'text': 12, 'ellipse': 0, 'polygon': 0, 'group': 2}
- reference: viewport={'width': '254px', 'height': '281px'} elements={'rect': 5, 'path': 4, 'text': 8, 'ellipse': 0, 'polygon': 0, 'group': 2}

## Final Diffs

- Raw first diff: target=`reference` at `1:148`
- Raw context: `expected=...ss" data-diagram-type="NWDIAG" height="281px" preserveAspectRatio="none" style="... actual=...ss" data-diagram-type="NWDIAG" height="276px" preserveAspectRatio="none" style="...`
- Semantic first diff: target=`reference` at `1:148`
- Semantic context: `expected=...ss" data-diagram-type="NWDIAG" height="281px" preserveAspectRatio="none" style="... actual=...ss" data-diagram-type="NWDIAG" height="276px" preserveAspectRatio="none" style="...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'NWDIAG', 'height': '281px', 'preserveAspectRatio': 'none', 'style': 'width:254px;height:281px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 254 281', 'width': '254px', 'zoomAndPan': 'magnify'}, 'text': 'Infrastructuredmz10.0.0.0/24lan10.0.0.10appdb01app01'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'NWDIAG', 'height': '276px', 'preserveAspectRatio': 'none', 'style': 'width:671px;height:276px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 671 276', 'width': '671px', 'zoomAndPan': 'magnify'}, 'text': 'InfrastructureInfrastructuredmz10.0.0.0/24lanweb0110.0.0.10frontenddb01web01appapp01'}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=+417, dh=-5`
- First coordinate signal: `height`
- Underlying signals: `family-stage-trace`

## Fix Suggestions

- Stage trace first (low): No strong heuristic matched. Add stage-boundary JSONL traces around the detected family, then compare the first divergent stage.
  files: `/ext/plantuml/plantuml-little/src/lib.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Code Anchors

- No Java/Rust anchor hints available.

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_nwdiag_basic_puml' (4108730) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/nwdiag/basic.puml: output differs from reference at line 1 col 148
expected: ...ss" data-diagram-type="NWDIAG" height="281px" preserveAspectRatio="none" style="...
actual:   ...ss" data-diagram-type="NWDIAG" height="276px" preserveAspectRatio="none" style="...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_nwdiag_basic.puml/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dispatch, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/lib.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/SourceStringReader.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_nwdiag_basic.puml/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_nwdiag_basic.puml/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_nwdiag_basic.puml/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_nwdiag_basic.puml/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-after-jaws6/batch/trace-probes/tests_fixtures_nwdiag_basic.puml/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Stage trace first`, then add stage-boundary traces if the first diff is still ambiguous.

