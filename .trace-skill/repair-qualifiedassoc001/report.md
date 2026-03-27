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
- First coordinate signal: `y +3`

## Fix Suggestions

- Svek offset normalization (high): A small repeated y-offset in graphviz-backed diagrams usually points to move_delta, normalize_offset, or generic protrusion handling.
  files: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/layout/graphviz.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_class_qualifiedassoc001_puml' (3997068) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/class/qualifiedassoc001.puml: output differs from reference at line 1 col 195
expected: ...eserveAspectRatio="none" style="width:662px;height:374px;background:#FFFFFF;" ve...
actual:   ...eserveAspectRatio="none" style="width:663px;height:374px;background:#FFFFFF;" ve...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/repair-qualifiedassoc001/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Svek offset normalization`, then add stage-boundary traces if the first diff is still ambiguous.

