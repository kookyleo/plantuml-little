# Diagnosis Report: tests/fixtures/state/state_note001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/state/state_note001.puml`
- Reference test: `reference_fixtures_state_state_note001_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`

## Final Artifact Summary

- rust: viewport={'width': '284px', 'height': '368px'} elements={'rect': 3, 'path': 7, 'text': 6, 'ellipse': 2, 'polygon': 2, 'group': 7}
- java: viewport={'width': '371px', 'height': '366px'} elements={'rect': 3, 'path': 8, 'text': 6, 'ellipse': 2, 'polygon': 3, 'group': 10}
- reference: viewport={'width': '371px', 'height': '366px'} elements={'rect': 3, 'path': 8, 'text': 6, 'ellipse': 2, 'polygon': 3, 'group': 10}

## First Final Diff

- Target: `reference`
- Line/col: `1:148`
- Context: `expected=...ss" data-diagram-type="STATE" height="366px" preserveAspectRatio="none" style="w... actual=...ss" data-diagram-type="STATE" height="368px" preserveAspectRatio="none" style="w...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `dw=-87, dh=+2`
- First coordinate signal: `x -95`

## Fix Suggestions

- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_state_state_note001_puml' (3930233) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/state/state_note001.puml: output differs from reference at line 1 col 148
expected: ...ss" data-diagram-type="STATE" height="366px" preserveAspectRatio="none" style="w...
actual:   ...ss" data-diagram-type="STATE" height="368px" preserveAspectRatio="none" style="w...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__state_note001/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Graphviz coordinate chain`, then add stage-boundary traces if the first diff is still ambiguous.

