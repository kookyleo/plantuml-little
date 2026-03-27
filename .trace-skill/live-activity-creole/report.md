# Diagnosis Report: tests/fixtures/activity/activity_creole_table_02.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/activity/activity_creole_table_02.puml`
- Reference test: `reference_fixtures_activity_activity_creole_table_02_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`

## Final Artifact Summary

- rust: viewport={'width': '237px', 'height': '245px'} elements={'rect': 3, 'path': 0, 'text': 7, 'ellipse': 0, 'polygon': 2, 'group': 1}
- java: viewport={'width': '154px', 'height': '301px'} elements={'rect': 3, 'path': 0, 'text': 17, 'ellipse': 0, 'polygon': 2, 'group': 1}
- reference: viewport={'width': '154px', 'height': '301px'} elements={'rect': 3, 'path': 0, 'text': 17, 'ellipse': 0, 'polygon': 2, 'group': 1}

## First Final Diff

- Target: `reference`
- Line/col: `1:149`
- Context: `expected=...s" data-diagram-type="ACTIVITY" height="301px" preserveAspectRatio="none" style=... actual=...s" data-diagram-type="ACTIVITY" height="245px" preserveAspectRatio="none" style=...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `dw=+83, dh=-56`
- First coordinate signal: `x2 +75.9024`

## Fix Suggestions

- Stage trace first (low): No strong heuristic matched. Add stage-boundary JSONL traces around the detected family, then compare the first divergent stage.
  files: `/ext/plantuml/plantuml-little/src/lib.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_activity_activity_creole_table_02_puml' (3995820) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/activity/activity_creole_table_02.puml: output differs from reference at line 1 col 149
expected: ...s" data-diagram-type="ACTIVITY" height="301px" preserveAspectRatio="none" style=...
actual:   ...s" data-diagram-type="ACTIVITY" height="245px" preserveAspectRatio="none" style=...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Stage trace first`, then add stage-boundary traces if the first diff is still ambiguous.

