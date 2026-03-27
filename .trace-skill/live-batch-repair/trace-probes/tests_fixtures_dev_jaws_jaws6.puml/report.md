# Diagnosis Report: tests/fixtures/dev/jaws/jaws6.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/dev/jaws/jaws6.puml`
- Reference test: `reference_fixtures_dev_jaws_jaws6_puml`
- Family: `sequence`
- Diagram type: `SEQUENCE`
- Authority tier: `reference-test`
- Worktree: `dirty` (15 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '527px', 'height': '185px'} elements={'rect': 11, 'path': 4, 'text': 12, 'ellipse': 4, 'polygon': 0, 'group': 21}
- reference: viewport={'width': '527px', 'height': '185px'} elements={'rect': 11, 'path': 4, 'text': 12, 'ellipse': 4, 'polygon': 0, 'group': 21}

## Final Diffs

- Raw first diff: target=`reference` at `1:839`
- Raw context: `expected=...-line="4" id="part2-lifeline"><g><title>.Order</title><rect fill="#000000" fill-... actual=...-line="4" id="part2-lifeline"><g><title>:Order</title><rect fill="#000000" fill-...`
- Semantic first diff: target=`reference` at `1:791`
- Semantic context: `expected=...data-source-line="4" id="REF"><g><title>.Order</title><rect fill="#000000" fill-... actual=...data-source-line="4" id="REF"><g><title>:Order</title><rect fill="#000000" fill-...`
- Object first diff: index=`0` target=`reference`
- Expected object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'SEQUENCE', 'height': '185px', 'preserveAspectRatio': 'none', 'style': 'width:527px;height:185px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 527 185', 'width': '527px', 'zoomAndPan': 'magnify'}, 'text': 'Purchase Officer.OrderBudget.datastore.SupplierPurchase OfficerPurchase Officer:Order:OrderBudgetBudget«datastore»Orders«datastore»OrdersSupplierSupplier'}`
- Actual object: `{'tag': 'svg', 'attrs': {'contentStyleType': 'text/css', 'data-diagram-type': 'SEQUENCE', 'height': '185px', 'preserveAspectRatio': 'none', 'style': 'width:527px;height:185px;background:#FFFFFF;', 'version': '1.1', 'viewBox': '0 0 527 185', 'width': '527px', 'zoomAndPan': 'magnify'}, 'text': 'Purchase Officer:OrderBudget«datastore»\ue100OrdersSupplierPurchase OfficerPurchase Officer:Order:OrderBudgetBudget«datastore»Orders«datastore»OrdersSupplierSupplier'}`

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

thread 'reference_fixtures_dev_jaws_jaws6_puml' (4098298) panicked at tests/reference_tests.rs:226:9:
tests/fixtures/dev/jaws/jaws6.puml: output differs from reference at line 1 col 839
expected: ...-line="4" id="part2-lifeline"><g><title>.Order</title><rect fill="#000000" fill-...
actual:   ...-line="4" id="part2-lifeline"><g><title>:Order</title><rect fill="#000000" fill-...
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_dev_jaws_jaws6.puml/reference-test.stderr.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dispatch, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/layout/sequence.rs`, `/ext/plantuml/plantuml-little/src/layout/sequence_teoz/builder.rs`, `/ext/plantuml/plantuml-little/src/render/svg_sequence.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/sequencediagram/teoz/TileBuilder.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/sequencediagram/graphic/ParticipantBox.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_dev_jaws_jaws6.puml/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_dev_jaws_jaws6.puml/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_dev_jaws_jaws6.puml/rust-render.stderr.log`
- reference_test: returncode=101
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_dev_jaws_jaws6.puml/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/live-batch-repair/trace-probes/tests_fixtures_dev_jaws_jaws6.puml/reference-test.stderr.log`

## Next Step

- Start with the top suggested chain: `Sequence layout core`, then add stage-boundary traces if the first diff is still ambiguous.

