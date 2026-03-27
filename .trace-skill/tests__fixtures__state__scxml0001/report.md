# Diagnosis Report: tests/fixtures/state/scxml0001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/state/scxml0001.puml`
- Reference test: `reference_fixtures_state_scxml0001_puml`
- Family: `graphviz-svek`
- Diagram type: `STATE`

## Final Artifact Summary

- rust: viewport={'width': '84px', 'height': '278px'} elements={'rect': 2, 'path': 2, 'text': 3, 'ellipse': 1, 'polygon': 2, 'group': 6}
- java: viewport={'width': '76px', 'height': '278px'} elements={'rect': 2, 'path': 2, 'text': 3, 'ellipse': 1, 'polygon': 2, 'group': 6}
- reference: viewport={'width': '76px', 'height': '278px'} elements={'rect': 2, 'path': 2, 'text': 3, 'ellipse': 1, 'polygon': 2, 'group': 6}

## First Final Diff

- Target: `reference`
- Line/col: `1:193`
- Context: `expected=...preserveAspectRatio="none" style="width:76px;height:278px;background:#FFFFFF;" v... actual=...preserveAspectRatio="none" style="width:84px;height:278px;background:#FFFFFF;" v...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `dw=+8`
- First coordinate signal: `path_d`

## Fix Suggestions

- Sprite renderer (medium): Sprite, transform, or path-data mismatches usually come from the SVG sprite renderer rather than parser logic.
  files: `/ext/plantuml/plantuml-little/src/render/svg_sprite.rs`
- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_state_scxml0001_puml' (3947737) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/state/scxml0001.puml: output differs from reference at line 1 col 193
expected: ...preserveAspectRatio="none" style="width:76px;height:278px;background:#FFFFFF;" v...
actual:   ...preserveAspectRatio="none" style="width:84px;height:278px;background:#FFFFFF;" v...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__state__scxml0001/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Sprite renderer`, then add stage-boundary traces if the first diff is still ambiguous.

