# Diagnosis Report: tests/fixtures/dev/jaws/jaws6.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/dev/jaws/jaws6.puml`
- Reference test: `reference_fixtures_dev_jaws_jaws6_puml`
- Family: `sequence`
- Diagram type: `SEQUENCE`
- Authority tier: `reference-test`
- Worktree: `dirty` (17 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '527px', 'height': '185px'} elements={'rect': 11, 'path': 4, 'text': 12, 'ellipse': 4, 'polygon': 0, 'group': 21}
- reference: viewport={'width': '527px', 'height': '185px'} elements={'rect': 11, 'path': 4, 'text': 12, 'ellipse': 4, 'polygon': 0, 'group': 21}

## Final Diffs

- Raw first diff: target=`reference` at `1:2495`
- Raw context: `expected=...xt><ellipse cx="70.7056" cy="14" fill="#FFFFFF" rx="8" ry="8" style="stroke:#000... actual=...xt><ellipse cx="70.7056" cy="14" fill="#E2E2F0" rx="8" ry="8" style="stroke:#000...`
- Semantic first diff: target=`reference` at `1:2407`
- Semantic context: `expected=...xt><ellipse cx="70.7056" cy="14" fill="#FFFFFF" rx="8" ry="8" style="stroke:#000... actual=...xt><ellipse cx="70.7056" cy="14" fill="#E2E2F0" rx="8" ry="8" style="stroke:#000...`
- Object first diff: index=`29` target=`reference`
- Expected object: `{'tag': 'ellipse', 'attrs': {'cx': '70.7056', 'cy': '14', 'fill': '#FFFFFF', 'rx': '8', 'ry': '8', 'style': 'stroke:#000000;stroke-width:1;'}}`
- Actual object: `{'tag': 'ellipse', 'attrs': {'cx': '70.7056', 'cy': '14', 'fill': '#E2E2F0', 'rx': '8', 'ry': '8', 'style': 'stroke:#000000;stroke-width:1;'}}`

## Diff Classification

- Surface category: `coordinate-only`
- Viewport delta: `unknown`
- First coordinate signal: `path_d`
- Underlying signals: `sequence-core`

## Fix Suggestions

- Sequence layout core (high): Sequence mismatch without teoz usually belongs to lifeline width, self-message width, or message layout.
  files: `/ext/plantuml/plantuml-little/src/layout/sequence.rs`, `/ext/plantuml/plantuml-little/src/render/svg_sequence.rs`
- Sprite renderer (medium): Sprite, transform, or path-data mismatches usually come from the SVG sprite renderer rather than parser logic.
  files: `/ext/plantuml/plantuml-little/src/render/svg_sprite.rs`

## Code Anchors

- Sequence core: Sequence mismatches usually belong to tile spacing, participant width, or self-message placement.
  java: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/sequencediagram/teoz/TileBuilder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/sequencediagram/graphic/ParticipantBox.java`
  rust: `/ext/plantuml/plantuml-little/src/layout/sequence.rs`, `/ext/plantuml/plantuml-little/src/layout/sequence_teoz/builder.rs`, `/ext/plantuml/plantuml-little/src/render/svg_sequence.rs`

## Reference Test

- Status: `failed`
- Return code: `101`
- Failure excerpt:

```text
Running tests/reference_tests.rs (target/debug/deps/reference_tests-69b2c048eac2359d)

thread 'reference_fixtures_dev_jaws_jaws6_puml' (4104389) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/dev/jaws/jaws6.puml: output differs from reference at line 1 col 2495
expected: ...xt><ellipse cx="70.7056" cy="14" fill="#FFFFFF" rx="8" ry="8" style="stroke:#000...
actual:   ...xt><ellipse cx="70.7056" cy="14" fill="#E2E2F0" rx="8" ry="8" style="stroke:#000...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws6-after-style-fix/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dispatch, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/layout/sequence.rs`, `/ext/plantuml/plantuml-little/src/layout/sequence_teoz/builder.rs`, `/ext/plantuml/plantuml-little/src/render/svg_sequence.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/sequencediagram/teoz/TileBuilder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/sequencediagram/graphic/ParticipantBox.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws6-after-style-fix/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws6-after-style-fix/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws6-after-style-fix/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws6-after-style-fix/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-jaws6-after-style-fix/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Sequence layout core`, then add stage-boundary traces if the first diff is still ambiguous.

