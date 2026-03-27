# Diagnosis Report: tests/fixtures/class/hideshow004.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/class/hideshow004.puml`
- Reference test: `reference_fixtures_class_hideshow004_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '226px', 'height': '170px'} elements={'rect': 4, 'path': 4, 'text': 5, 'ellipse': 4, 'polygon': 1, 'group': 6}
- java: viewport={'width': '226px', 'height': '170px'} elements={'rect': 4, 'path': 4, 'text': 5, 'ellipse': 4, 'polygon': 1, 'group': 6}
- reference: viewport={'width': '226px', 'height': '170px'} elements={'rect': 4, 'path': 4, 'text': 5, 'ellipse': 4, 'polygon': 1, 'group': 6}

## First Final Diff

- Target: `reference`
- Line/col: `1:548`
- Context: `expected=...;stroke-width:0.5;" width="125.5654" x="9" y="7"/><ellipse cx="62.7305" cy="23" ... actual=...;stroke-width:0.5;" width="125.5654" x="7" y="7"/><ellipse cx="60.7305" cy="23" ...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `x -2`

## Fix Suggestions

- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
render_class: render_offset=(7.00,7.00) edge_offset=(7.00,7.00) move_delta=(6.00,-2.00) normalize_offset=(6.00,6.00)

thread 'reference_fixtures_class_hideshow004_puml' (3949805) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/class/hideshow004.puml: output differs from reference at line 1 col 548
expected: ...;stroke-width:0.5;" width="125.5654" x="9" y="7"/><ellipse cx="62.7305" cy="23" ...
actual:   ...;stroke-width:0.5;" width="125.5654" x="7" y="7"/><ellipse cx="60.7305" cy="23" ...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__class__hideshow004/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Graphviz coordinate chain`, then add stage-boundary traces if the first diff is still ambiguous.

