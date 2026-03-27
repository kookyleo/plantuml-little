# Diagnosis Report: tests/fixtures/dev/jaws/jaws3.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/dev/jaws/jaws3.puml`
- Reference test: `reference_fixtures_dev_jaws_jaws3_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '241px', 'height': '92px'} elements={'rect': 1, 'path': 0, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}
- java: viewport={'width': '241px', 'height': '92px'} elements={'rect': 1, 'path': 0, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}
- reference: viewport={'width': '241px', 'height': '92px'} elements={'rect': 1, 'path': 0, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}

## First Final Diff

- Target: `reference`
- Line/col: `1:412`
- Context: `expected=...ta-qualified-name="r" data-source-line="9" id="ent0002"><rect fill="#F1F1F1" hei... actual=...ta-qualified-name="r" data-source-line="5" id="ent0002"><rect fill="#F1F1F1" hei...`

## Diff Classification

- Surface category: `viewport-only`
- Viewport delta: `unknown`
- Underlying signals: `graphviz-coordinate-chain`

## Fix Suggestions

- Class cluster/protrusion chain (medium): CLASS viewport drift often comes from group bounds, qualifier spacing, or protrusion normalization after svek.
  files: `/ext/plantuml/plantuml-little/src/svek/mod.rs`, `/ext/plantuml/plantuml-little/src/svek/cluster.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_dev_jaws_jaws3_puml' (4014215) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/dev/jaws/jaws3.puml: output differs from reference at line 1 col 412
expected: ...ta-qualified-name="r" data-source-line="9" id="ent0002"><rect fill="#F1F1F1" hei...
actual:   ...ta-qualified-name="r" data-source-line="5" id="ent0002"><rect fill="#F1F1F1" hei...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-after-shared/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Class cluster/protrusion chain`, then add stage-boundary traces if the first diff is still ambiguous.

