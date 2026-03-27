# Diagnosis Report: tests/fixtures/dev/newline/activity_creole_table.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/dev/newline/activity_creole_table.puml`
- Reference test: `reference_fixtures_dev_newline_activity_creole_table_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`

## Final Artifact Summary

- rust: viewport={'width': '176px', 'height': '83px'} elements={'rect': 1, 'path': 0, 'text': 2, 'ellipse': 0, 'polygon': 0, 'group': 1}
- java: viewport={'width': '168px', 'height': '87px'} elements={'rect': 4, 'path': 0, 'text': 19, 'ellipse': 0, 'polygon': 2, 'group': 2}
- reference: viewport={'width': '168px', 'height': '87px'} elements={'rect': 1, 'path': 0, 'text': 2, 'ellipse': 0, 'polygon': 0, 'group': 1}

## First Final Diff

- Target: `reference`
- Line/col: `1:150`
- Context: `expected=..." data-diagram-type="ACTIVITY" height="87px" preserveAspectRatio="none" style="w... actual=..." data-diagram-type="ACTIVITY" height="83px" preserveAspectRatio="none" style="w...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=+8, dh=-4`
- First coordinate signal: `width +8.0859`
- Underlying signals: `family-stage-trace`

## Fix Suggestions

- Stage trace first (low): No strong heuristic matched. Add stage-boundary JSONL traces around the detected family, then compare the first divergent stage.
  files: `/ext/plantuml/plantuml-little/src/lib.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_dev_newline_activity_creole_table_puml' (4029033) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/dev/newline/activity_creole_table.puml: output differs from reference at line 1 col 150
expected: ..." data-diagram-type="ACTIVITY" height="87px" preserveAspectRatio="none" style="w...
actual:   ..." data-diagram-type="ACTIVITY" height="83px" preserveAspectRatio="none" style="w...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-dev-activity-creole-singlecol/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Stage trace first`, then add stage-boundary traces if the first diff is still ambiguous.

