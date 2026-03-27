# Diagnosis Report: tests/fixtures/class/hideshow003.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/class/hideshow003.puml`
- Reference test: `reference_fixtures_class_hideshow003_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '255px', 'height': '121px'} elements={'rect': 4, 'path': 4, 'text': 5, 'ellipse': 3, 'polygon': 0, 'group': 6}
- java: viewport={'width': '255px', 'height': '121px'} elements={'rect': 4, 'path': 4, 'text': 5, 'ellipse': 3, 'polygon': 0, 'group': 6}
- reference: viewport={'width': '255px', 'height': '121px'} elements={'rect': 4, 'path': 4, 'text': 5, 'ellipse': 3, 'polygon': 0, 'group': 6}

## First Final Diff

- No final diff found.

## Diff Classification

- Category: `viewport-only`
- Viewport delta: `unknown`

## Fix Suggestions

- Class cluster/protrusion chain (medium): CLASS viewport drift often comes from group bounds, qualifier spacing, or protrusion normalization after svek.
  files: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/cluster.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/reference-test.stdout.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/java-render.stderr.log`
- reference_test: returncode=0
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-hideshow003/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Class cluster/protrusion chain`, then add stage-boundary traces if the first diff is still ambiguous.

