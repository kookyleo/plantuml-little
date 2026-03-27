# Diagnosis Report: tests/fixtures/activity/activity_creole_table_02.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/activity/activity_creole_table_02.puml`
- Reference test: `reference_fixtures_activity_activity_creole_table_02_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`

## Final Artifact Summary

- rust: viewport={'width': '237px', 'height': '245px'} elements={'rect': 3, 'path': 0, 'text': 7, 'ellipse': 0, 'polygon': 2, 'group': 1}
- java: viewport={'width': '154px', 'height': '301px'} elements={'rect': 3, 'path': 0, 'text': 17, 'ellipse': 0, 'polygon': 2, 'group': 1}
- reference: viewport={'width': '154px', 'height': '301px'} elements={'rect': 3, 'path': 0, 'text': 17, 'ellipse': 0, 'polygon': 2, 'group': 1}

## First Final Diff

- Target: `reference`
- Line/col: `1:149`
- Context: `expected=...s" data-diagram-type="ACTIVITY" height="301px" preserveAspectRatio="none" style=... actual=...s" data-diagram-type="ACTIVITY" height="245px" preserveAspectRatio="none" style=...`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `dw=+83, dh=-56`
- First coordinate signal: `x2 +75.9024`
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

thread 'reference_fixtures_activity_activity_creole_table_02_puml' (4022556) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/activity/activity_creole_table_02.puml: output differs from reference at line 1 col 149
expected: ...s" data-diagram-type="ACTIVITY" height="301px" preserveAspectRatio="none" style=...
actual:   ...s" data-diagram-type="ACTIVITY" height="245px" preserveAspectRatio="none" style=...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/java-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-activity-creole-after-jaws3/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Shared richtext table/display`, then add stage-boundary traces if the first diff is still ambiguous.

