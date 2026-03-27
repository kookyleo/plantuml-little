# Diagnosis Report: tests/fixtures/activity/a0002.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/activity/a0002.puml`
- Reference test: `reference_fixtures_activity_a0002_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`

## Final Artifact Summary

- rust: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}
- java: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}
- reference: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}

## First Final Diff

- Target: `reference`
- Line/col: `1:362`
- Context: `expected=...26.3beta5?><defs><filter height="1" id="b1d3v29bgce2h80" width="1" x="0" y="0"><... actual=...26.3beta5?><defs><filter height="1" id="inkoj4fwrplg3000" width="1" x="0" y="0">...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `path_d`
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

thread 'reference_fixtures_activity_a0002_puml' (4067947) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/activity/a0002.puml: output differs from reference at line 1 col 10025
expected: ...y1="17.7451" y2="715.3467"/><path d="M412.5146,380.9561 L412.5146,523.1514 L392....
actual:   ...y1="17.7451" y2="715.3467"/><path d="M413.0146,380.9561 L413.0146,673.3467 L528....
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-latest/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Sprite renderer`, then add stage-boundary traces if the first diff is still ambiguous.

