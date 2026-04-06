# Diagnosis Report: tests/fixtures/nonreg/simple/ChenRankDir.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/nonreg/simple/ChenRankDir.puml`
- Reference test: `reference_fixtures_nonreg_simple_ChenRankDir_puml`
- Family: `self-layout`
- Diagram type: `CHEN_EER`
- Authority tier: `reference-test`
- Worktree: `dirty` (9 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '568px', 'height': '78px'} elements={'rect': 2, 'path': 2, 'text': 5, 'ellipse': 0, 'polygon': 1, 'group': 6}
- reference: viewport={'width': '556px', 'height': '84px'} elements={'rect': 2, 'path': 2, 'text': 5, 'ellipse': 0, 'polygon': 1, 'group': 6}

## Final Diffs

- Raw first diff: target=`reference` at `1:149`
- Raw context: `expected=...s" data-diagram-type="CHEN_EER" height="84px" preserveAspectRatio="none" style="... actual=...s" data-diagram-type="CHEN_EER" height="78px" preserveAspectRatio="none" style="...`
- Semantic first diff: target=`reference` at `1:149`
- Semantic context: `expected=...s" data-diagram-type="CHEN_EER" height="84px" preserveAspectRatio="none" style="... actual=...s" data-diagram-type="CHEN_EER" height="78px" preserveAspectRatio="none" style="...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'CHEN_EER', 'height': '84px', 'preserveAspectRatio': 'none', 'style': 'width:556px;height:84px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 556 84', 'width': '556px', 'zoomAndPan': 'magnify'}, 'text': 'PersonLocationBirthplaceN1'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'CHEN_EER', 'height': '78px', 'preserveAspectRatio': 'none', 'style': 'width:568px;height:78px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 568 78', 'width': '568px', 'zoomAndPan': 'magnify'}, 'text': 'PersonLocationBirthplaceN1'}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=+12, dh=-6`
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

thread 'reference_fixtures_nonreg_simple_ChenRankDir_puml' (1940891) panicked at tests/reference_tests.rs:307:9:
tests/fixtures/nonreg/simple/ChenRankDir.puml: output differs from reference at line 1 col 149
expected: ...s" data-diagram-type="CHEN_EER" height="84px" preserveAspectRatio="none" style="...
actual:   ...s" data-diagram-type="CHEN_EER" height="78px" preserveAspectRatio="none" style="...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_simple_ChenRankDir.puml/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dispatch, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/lib.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/SourceStringReader.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_simple_ChenRankDir.puml/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_simple_ChenRankDir.puml/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_simple_ChenRankDir.puml/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_simple_ChenRankDir.puml/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/tmp_debug/reference-diagnosis-20260329/suite/trace-probes/tests_fixtures_nonreg_simple_ChenRankDir.puml/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Stage trace first`, then add stage-boundary traces if the first diff is still ambiguous.

