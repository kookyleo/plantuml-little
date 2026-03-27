# Diagnosis Report: tests/fixtures/activity/swimlane001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/activity/swimlane001.puml`
- Reference test: `reference_fixtures_activity_swimlane001_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`

## Final Artifact Summary

- rust: viewport={'width': '266px', 'height': '288px'} elements={'rect': 4, 'path': 0, 'text': 5, 'ellipse': 3, 'polygon': 4, 'group': 1}
- java: viewport={'width': '266px', 'height': '288px'} elements={'rect': 4, 'path': 0, 'text': 5, 'ellipse': 3, 'polygon': 4, 'group': 1}
- reference: viewport={'width': '266px', 'height': '288px'} elements={'rect': 4, 'path': 0, 'text': 5, 'ellipse': 3, 'polygon': 4, 'group': 1}

## First Final Diff

- Target: `reference`
- Line/col: `1:465`
- Context: `expected=...1577" x="20" y="17.7451"/><ellipse cx="75.3242" cy="53.6982" fill="#222222" rx="... actual=...1577" x="20" y="17.7451"/><ellipse cx="74.3242" cy="53.6982" fill="#222222" rx="...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `cx -1`
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

thread 'reference_fixtures_activity_swimlane001_puml' (4048025) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/activity/swimlane001.puml: output differs from reference at line 1 col 465
expected: ...1577" x="20" y="17.7451"/><ellipse cx="75.3242" cy="53.6982" fill="#222222" rx="...
actual:   ...1577" x="20" y="17.7451"/><ellipse cx="74.3242" cy="53.6982" fill="#222222" rx="...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-now/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Stage trace first`, then add stage-boundary traces if the first diff is still ambiguous.

