# Diagnosis Report: tests/fixtures/class/qualifiedassoc001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/class/qualifiedassoc001.puml`
- Reference test: `reference_fixtures_class_qualifiedassoc001_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '663px', 'height': '374px'} elements={'rect': 21, 'path': 16, 'text': 21, 'ellipse': 10, 'polygon': 10, 'group': 17}
- java: viewport={'width': '662px', 'height': '374px'} elements={'rect': 21, 'path': 16, 'text': 21, 'ellipse': 10, 'polygon': 10, 'group': 17}
- reference: viewport={'width': '662px', 'height': '374px'} elements={'rect': 21, 'path': 16, 'text': 21, 'ellipse': 10, 'polygon': 10, 'group': 17}

## First Final Diff

- Target: `reference`
- Line/col: `1:195`
- Context: `expected=...eserveAspectRatio="none" style="width:662px;height:374px;background:#FFFFFF;" ve... actual=...eserveAspectRatio="none" style="width:663px;height:374px;background:#FFFFFF;" ve...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `dw=+1`
- First coordinate signal: `path_d`

## Fix Suggestions

- Sprite renderer (medium): Sprite, transform, or path-data mismatches usually come from the SVG sprite renderer rather than parser logic.
  files: `/ext/plantuml/plantuml-little/src/render/svg_sprite.rs`
- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `not-run`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc001/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc001/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc001/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc001/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc001/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc001/java-render.stderr.log`

## Next Step

- Start with the top suggested chain: `Sprite renderer`, then add stage-boundary traces if the first diff is still ambiguous.

