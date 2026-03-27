# Diagnosis Report: tests/fixtures/dev/jaws/jaws3.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/dev/jaws/jaws3.puml`
- Reference test: `reference_fixtures_dev_jaws_jaws3_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '184px', 'height': '105px'} elements={'rect': 1, 'path': 0, 'text': 4, 'ellipse': 0, 'polygon': 0, 'group': 2}
- java: viewport={'width': '241px', 'height': '92px'} elements={'rect': 1, 'path': 0, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}
- reference: viewport={'width': '241px', 'height': '92px'} elements={'rect': 1, 'path': 0, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}

## First Final Diff

- Target: `reference`
- Line/col: `1:146`
- Context: `expected=.../css" data-diagram-type="CLASS" height="92px" preserveAspectRatio="none" style="... actual=.../css" data-diagram-type="CLASS" height="105px" preserveAspectRatio="none" style=...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `dw=-57, dh=+13`
- First coordinate signal: `width -56.9981`

## Fix Suggestions

- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_dev_jaws_jaws3_puml' (4005467) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/dev/jaws/jaws3.puml: output differs from reference at line 1 col 146
expected: .../css" data-diagram-type="CLASS" height="92px" preserveAspectRatio="none" style="...
actual:   .../css" data-diagram-type="CLASS" height="105px" preserveAspectRatio="none" style=...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Graphviz coordinate chain`, then add stage-boundary traces if the first diff is still ambiguous.

