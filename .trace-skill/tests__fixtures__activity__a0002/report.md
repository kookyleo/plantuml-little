# Diagnosis Report: tests/fixtures/activity/a0002.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/activity/a0002.puml`
- Reference test: `reference_fixtures_activity_a0002_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`

## Final Artifact Summary

- rust: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 46, 'ellipse': 3, 'polygon': 3, 'group': 1}
- java: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}
- reference: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}

## First Final Diff

- Target: `reference`
- Line/col: `1:362`
- Context: `expected=...26.3beta5?><defs><filter height="1" id="b1d3v29bgce2h80" width="1" x="0" y="0"><... actual=...26.3beta5?><defs><filter height="1" id="inkoj4fwrplg3000" width="1" x="0" y="0">...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `cx -11.6358`

## Fix Suggestions

- Stage trace first (low): No strong heuristic matched. Add stage-boundary JSONL traces around the detected family, then compare the first divergent stage.
  files: `/ext/plantuml/plantuml-little/src/lib.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_activity_a0002_puml' (3935627) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/activity/a0002.puml: output differs from reference at line 1 col 599
expected: ...="stroke:none;stroke-width:1;" width="515.2031" x="20" y="17.7451"/><ellipse cx=...
actual:   ...="stroke:none;stroke-width:1;" width="513.3423" x="20" y="17.7451"/><ellipse cx=...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__activity__a0002/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Stage trace first`, then add stage-boundary traces if the first diff is still ambiguous.

