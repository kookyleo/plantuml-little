# Diagnosis Report: tests/fixtures/activity/a0002.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/activity/a0002.puml`
- Reference test: `reference_fixtures_activity_a0002_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`

## Final Artifact Summary

- rust: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 46, 'ellipse': 3, 'polygon': 3, 'group': 1}
- java: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}
- reference: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}

## First Final Diff

- Target: `reference`
- Line/col: `1:362`
- Context: `expected=...26.3beta5?><defs><filter height="1" id="b1d3v29bgce2h80" width="1" x="0" y="0"><... actual=...26.3beta5?><defs><filter height="1" id="inkoj4fwrplg3000" width="1" x="0" y="0">...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `x1 -189.144`
- Underlying signals: `shared-richtext-table, shared-text-body`

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

thread 'reference_fixtures_activity_a0002_puml' (4048022) panicked at tests/reference_tests.rs:219:21:
tests/fixtures/activity/a0002.puml: output differs from reference at line 1 col 764
expected: ...idth:1;"/><path d="M35,209.7607 L35,234.8936 L142.6011,234.8936 L142.6011,219.76...
actual:   ...idth:1;"/><path d="M35,209.7607 L35,234.9007 L142.6011,234.9007 L142.6011,219.76...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-a0002-now/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Shared richtext table/display`, then add stage-boundary traces if the first diff is still ambiguous.

