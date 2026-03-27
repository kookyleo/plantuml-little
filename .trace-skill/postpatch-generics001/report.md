# Diagnosis Report: tests/fixtures/class/generics001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/class/generics001.puml`
- Reference test: `reference_fixtures_class_generics001_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '181px', 'height': '246px'} elements={'rect': 4, 'path': 3, 'text': 8, 'ellipse': 6, 'polygon': 1, 'group': 8}
- java: viewport={'width': '181px', 'height': '246px'} elements={'rect': 4, 'path': 3, 'text': 8, 'ellipse': 6, 'polygon': 1, 'group': 8}
- reference: viewport={'width': '181px', 'height': '246px'} elements={'rect': 4, 'path': 3, 'text': 8, 'ellipse': 6, 'polygon': 1, 'group': 8}

## First Final Diff

- Target: `reference`
- Line/col: `1:568`
- Context: `expected=...ke-width:0.5;" width="155.042" x="7" y="10"/><ellipse cx="44.0654" cy="26" fill=... actual=...ke-width:0.5;" width="155.042" x="7" y="7"/><ellipse cx="44.0654" cy="23" fill="...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `y -3`

## Fix Suggestions

- Svek offset normalization (high): A small repeated y-offset in graphviz-backed diagrams usually points to move_delta, normalize_offset, or generic protrusion handling.
  files: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `not-run`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-generics001/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-generics001/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-generics001/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-generics001/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-generics001/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-generics001/java-render.stderr.log`

## Next Step

- Start with the top suggested chain: `Svek offset normalization`, then add stage-boundary traces if the first diff is still ambiguous.

