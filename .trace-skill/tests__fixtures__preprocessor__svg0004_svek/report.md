# Diagnosis Report: tests/fixtures/preprocessor/svg0004_svek.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/preprocessor/svg0004_svek.puml`
- Reference test: `reference_fixtures_preprocessor_svg0004_svek_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '903px', 'height': '214px'} elements={'rect': 1, 'path': 4, 'text': 6, 'ellipse': 1, 'polygon': 1, 'group': 5}
- java: viewport={'width': '903px', 'height': '214px'} elements={'rect': 1, 'path': 4, 'text': 6, 'ellipse': 1, 'polygon': 1, 'group': 5}
- reference: viewport={'width': '903px', 'height': '214px'} elements={'rect': 1, 'path': 4, 'text': 6, 'ellipse': 1, 'polygon': 1, 'group': 5}

## First Final Diff

- Target: `reference`
- Line/col: `1:368`
- Context: `expected=...?><defs><style type="text/css"><![CDATA[
svg .entity {
    cursor: pointer;
}
sv... actual=...?><defs><style type="text/css"><![CDATA[svg .entity {
    cursor: pointer;
}
svg...`

## Diff Classification

- Category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `cx -1`

## Fix Suggestions

- Graphviz coordinate chain (medium): Graphviz-backed coordinate drift usually belongs to post-dot coordinate extraction or edge/node handoff.
  files: `/ext/plantuml/plantuml-little/src/svek/svg_result.rs`, `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
edge label 'Hello' raw_xy=(831.00,126.00) move_delta=(-10.00,-18.00) normalize=(6.00,6.00) edge_offset=(6.00,6.00)

thread 'reference_fixtures_preprocessor_svg0004_svek_puml' (3943597) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/preprocessor/svg0004_svek.puml: output differs from reference at line 1 col 368
expected: ...?><defs><style type="text/css"><![CDATA[
svg .entity {
    cursor: pointer;
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/tests__fixtures__preprocessor__svg0004_svek/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Graphviz coordinate chain`, then add stage-boundary traces if the first diff is still ambiguous.

