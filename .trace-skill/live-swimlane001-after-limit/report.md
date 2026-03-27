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
- Line/col: `1:2231`
- Context: `expected=...width:1;" x1="75.3242" x2="75.3242" y1="225.6045" y2="245.6045"/><polygon fill="... actual=...width:1;" x1="75.3242" x2="75.3242" y1="117.667" y2="127.667"/><line style="stro...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `points`
- Underlying signals: `family-stage-trace`

## Fix Suggestions

- Sprite renderer (medium): Sprite, transform, or path-data mismatches usually come from the SVG sprite renderer rather than parser logic.
  files: `/ext/plantuml/plantuml-little/src/render/svg_sprite.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_activity_swimlane001_puml' (4051305) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/activity/swimlane001.puml: output differs from reference at line 1 col 2231
expected: ...width:1;" x1="75.3242" x2="75.3242" y1="225.6045" y2="245.6045"/><polygon fill="...
actual:   ...width:1;" x1="75.3242" x2="75.3242" y1="117.667" y2="127.667"/><line style="stro...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-swimlane001-after-limit/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Sprite renderer`, then add stage-boundary traces if the first diff is still ambiguous.

