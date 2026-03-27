# Diagnosis Report: tests/fixtures/class/qualifiedassoc001.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/class/qualifiedassoc001.puml`
- Reference test: `reference_fixtures_class_qualifiedassoc001_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '662px', 'height': '374px'} elements={'rect': 21, 'path': 16, 'text': 21, 'ellipse': 10, 'polygon': 10, 'group': 17}
- java: viewport={'width': '662px', 'height': '374px'} elements={'rect': 21, 'path': 16, 'text': 21, 'ellipse': 10, 'polygon': 10, 'group': 17}
- reference: viewport={'width': '662px', 'height': '374px'} elements={'rect': 21, 'path': 16, 'text': 21, 'ellipse': 10, 'polygon': 10, 'group': 17}

## First Final Diff

- Target: `reference`
- Line/col: `1:18898`
- Context: `expected=...>z: boolean</text></g><?plantuml-src PSm_3u8m48VXdK_nczKsuSz428kBYM4o8IhfKA02IQM... actual=...>z: boolean</text></g><?plantuml-src PSmz3u8m4CRndK_np6gRy38Hmk9YOfXCY4ew5AX2KYc...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `path_d`
- Underlying signals: `graphviz-coordinate-chain`
- Authority note: `reference_test` passed, so any cargo-run SVG diff here is advisory only.

## Fix Suggestions

- No repair suggested: the authoritative `reference_test` already passes for this fixture.

## Reference Test

- Status: `passed`
- Return code: `0`
- Failure excerpt:

```text
warning: unused import: `NoteLinkStrategy`
  --> src/abel/entity.rs:12:58
   |
12 | use super::{CucaNote, DisplayPositioned, EntityPosition, NoteLinkStrategy, Together};
   |                                                          ^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of 
...
 `reference_fixtures_sprite_test_polyline_sprites_puml`

warning: `plantuml-little` (test "reference_tests") generated 112 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/reference-test.stdout.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/java-render.stderr.log`
- reference_test: returncode=0
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-qualifiedassoc001-skill3/reference-test.stderr.log`

## Next Step

- No action required for reference alignment; investigate cargo-run artifact drift only if it matters.

