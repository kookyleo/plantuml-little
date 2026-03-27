# Diagnosis Report: tests/fixtures/dev/jaws/jaws11.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/dev/jaws/jaws11.puml`
- Reference test: `reference_fixtures_dev_jaws_jaws11_puml`
- Family: `sequence`
- Diagram type: `SEQUENCE`

## Final Artifact Summary

- rust: viewport={'width': '186px', 'height': '126px'} elements={'rect': 6, 'path': 0, 'text': 6, 'ellipse': 0, 'polygon': 0, 'group': 9}
- java: viewport={'width': '372px', 'height': '126px'} elements={'rect': 6, 'path': 0, 'text': 6, 'ellipse': 0, 'polygon': 0, 'group': 9}
- reference: viewport={'width': '372px', 'height': '126px'} elements={'rect': 6, 'path': 0, 'text': 6, 'ellipse': 0, 'polygon': 0, 'group': 9}

## First Final Diff

- Target: `reference`
- Line/col: `1:196`
- Context: `expected=...preserveAspectRatio="none" style="width:372px;height:126px;background:#FFFFFF;" ... actual=...preserveAspectRatio="none" style="width:186px;height:126px;background:#FFFFFF;" ...`

## Diff Classification

- Category: `viewport-only`
- Viewport delta: `dw=-186`

## Fix Suggestions

- Sequence layout core (high): Sequence mismatch without teoz usually belongs to lifeline width, self-message width, or message layout.
  files: `/ext/plantuml/plantuml-little/src/layout/sequence.rs`, `/ext/plantuml/plantuml-little/src/render/svg_sequence.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_dev_jaws_jaws11_puml' (3957854) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/dev/jaws/jaws11.puml: output differs from reference at line 1 col 196
expected: ...preserveAspectRatio="none" style="width:372px;height:126px;background:#FFFFFF;" ...
actual:   ...preserveAspectRatio="none" style="width:186px;height:126px;background:#FFFFFF;" ...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__dev__jaws__jaws11/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Sequence layout core`, then add stage-boundary traces if the first diff is still ambiguous.

