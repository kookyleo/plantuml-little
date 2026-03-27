# Diagnosis Report: tests/fixtures/class/qualifiedassoc002.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/class/qualifiedassoc002.puml`
- Reference test: `reference_fixtures_class_qualifiedassoc002_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '446px', 'height': '312px'} elements={'rect': 8, 'path': 7, 'text': 11, 'ellipse': 4, 'polygon': 3, 'group': 8}
- java: viewport={'width': '445px', 'height': '312px'} elements={'rect': 8, 'path': 7, 'text': 11, 'ellipse': 4, 'polygon': 3, 'group': 8}
- reference: viewport={'width': '445px', 'height': '312px'} elements={'rect': 8, 'path': 7, 'text': 11, 'ellipse': 4, 'polygon': 3, 'group': 8}

## First Final Diff

- Target: `reference`
- Line/col: `1:195`
- Context: `expected=...eserveAspectRatio="none" style="width:445px;height:312px;background:#FFFFFF;" ve... actual=...eserveAspectRatio="none" style="width:446px;height:312px;background:#FFFFFF;" ve...`

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

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_class_qualifiedassoc002_puml' (4000123) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/class/qualifiedassoc002.puml: output differs from reference at line 1 col 195
expected: ...eserveAspectRatio="none" style="width:445px;height:312px;background:#FFFFFF;" ve...
actual:   ...eserveAspectRatio="none" style="width:446px;height:312px;background:#FFFFFF;" ve...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/postpatch-qualifiedassoc002/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Sprite renderer`, then add stage-boundary traces if the first diff is still ambiguous.

