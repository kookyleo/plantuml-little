# Diagnosis Report: tests/fixtures/state/state_concurrent001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/state/state_concurrent001.puml`
- Reference test: `reference_fixtures_state_state_concurrent001_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`

## Final Artifact Summary

- rust: viewport={'width': '105px', 'height': '628px'} elements={'rect': 5, 'path': 3, 'text': 5, 'ellipse': 3, 'polygon': 2, 'group': 10}
- java: viewport={'width': '113px', 'height': '630px'} elements={'rect': 5, 'path': 7, 'text': 5, 'ellipse': 5, 'polygon': 6, 'group': 15}
- reference: viewport={'width': '113px', 'height': '630px'} elements={'rect': 5, 'path': 7, 'text': 5, 'ellipse': 5, 'polygon': 6, 'group': 15}

## First Final Diff

- Target: `reference`
- Line/col: `1:147`
- Context: `expected=...css" data-diagram-type="STATE" height="630px" preserveAspectRatio="none" style="... actual=...css" data-diagram-type="STATE" height="628px" preserveAspectRatio="none" style="...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `dw=-8, dh=-2`
- First coordinate signal: `cx -0.2231`

## Fix Suggestions

- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_state_state_concurrent001_puml' (3928762) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/state/state_concurrent001.puml: output differs from reference at line 1 col 147
expected: ...css" data-diagram-type="STATE" height="630px" preserveAspectRatio="none" style="...
actual:   ...css" data-diagram-type="STATE" height="628px" preserveAspectRatio="none" style="...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_concurrent001/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Graphviz coordinate chain`, then add stage-boundary traces if the first diff is still ambiguous.

