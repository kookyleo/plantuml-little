# Diagnosis Report: tests/fixtures/dev/jaws/jaws3.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/dev/jaws/jaws3.puml`
- Reference test: `reference_fixtures_dev_jaws_jaws3_puml`
- Family: `graphviz-svek`
- Diagram type: `CLASS`

## Final Artifact Summary

- rust: viewport={'width': '184px', 'height': '105px'} elements={'rect': 1, 'path': 0, 'text': 4, 'ellipse': 0, 'polygon': 0, 'group': 2}
- java: viewport={'width': '241px', 'height': '92px'} elements={'rect': 1, 'path': 0, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}
- reference: viewport={'width': '241px', 'height': '92px'} elements={'rect': 1, 'path': 0, 'text': 11, 'ellipse': 0, 'polygon': 0, 'group': 2}

## First Final Diff

- Target: `reference`
- Line/col: `1:146`
- Context: `expected=.../css" data-diagram-type="CLASS" height="92px" preserveAspectRatio="none" style="... actual=.../css" data-diagram-type="CLASS" height="105px" preserveAspectRatio="none" style=...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=-57, dh=+13`
- First coordinate signal: `width -56.9981`
- Underlying signals: `shared-richtext-table, shared-text-body, element-structure-drift`

## Fix Suggestions

- Shared richtext table/display (high): Rust and Java disagree on Creole/table structure, so start in the shared Display/Creole parser-renderer path instead of the Graphviz handoff.
  files: `/ext/plantuml/plantuml-little/src/parser/class.rs`, `/ext/plantuml/plantuml-little/src/parser/creole.rs`, `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg_richtext.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`
- Shared text/body height (high): The underlying signals point to upstream Display or Creole semantics, so investigate text splitting and block measurement before touching downstream coordinates.
  files: `/ext/plantuml/plantuml-little/src/layout/mod.rs`, `/ext/plantuml/plantuml-little/src/render/svg.rs`, `/ext/plantuml/plantuml-little/src/render/svg_richtext.rs`, `/ext/plantuml/plantuml-little/src/parser/class.rs`, `/ext/plantuml/plantuml-little/src/parser/component.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_dev_jaws_jaws3_puml' (4007704) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/dev/jaws/jaws3.puml: output differs from reference at line 1 col 146
expected: .../css" data-diagram-type="CLASS" height="92px" preserveAspectRatio="none" style="...
actual:   .../css" data-diagram-type="CLASS" height="105px" preserveAspectRatio="none" style=...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws3-current/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Shared richtext table/display`, then add stage-boundary traces if the first diff is still ambiguous.

