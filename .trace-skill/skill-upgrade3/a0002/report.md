# Diagnosis Report: tests/fixtures/activity/a0002.puml

## Case

- Fixture: `/ext/plantuml/plantuml-little/tests/fixtures/activity/a0002.puml`
- Reference test: `reference_fixtures_activity_a0002_puml`
- Family: `self-layout`
- Diagram type: `ACTIVITY`
- Authority tier: `reference-test`
- Worktree: `dirty` (15 changed paths)

## Final Artifact Summary

- rust: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}
- java: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}
- reference: viewport={'width': '562px', 'height': '736px'} elements={'rect': 3, 'path': 6, 'text': 115, 'ellipse': 5, 'polygon': 3, 'group': 1}

## Final Diffs

- Raw first diff: target=`reference` at `1:362`
- Raw context: `expected=...26.3beta5?><defs><filter height="1" id="b1d3v29bgce2h80" width="1" x="0" y="0"><... actual=...26.3beta5?><defs><filter height="1" id="inkoj4fwrplg3000" width="1" x="0" y="0">...`
- Semantic first diff: none (`raw` diff appears to be volatile-only noise)
- Object first diff: none

## Diff Classification

- Surface category: `semantic-equivalent`
- Viewport delta: `unknown`
- Semantic note: `normalized SVG is equivalent; treat remaining raw diff as advisory noise unless traces show otherwise.`
- Underlying signals: `volatile-svg-noise`
- Authority note: `reference_test` passed, so any cargo-run SVG diff here is advisory only.

## Fix Suggestions

- No repair suggested: the authoritative `reference_test` already passes for this fixture.

## Code Anchors

- Activity note chain: Activity note mismatches often split between FtileWithNotes/FtileWithNoteOpale selection and Opale polygon routing.
  java: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/activitydiagram3/ftile/vcompact/FtileWithNoteOpale.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/activitydiagram3/ftile/vcompact/FtileWithNotes.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/image/Opale.java`
  rust: `/ext/plantuml/plantuml-little/src/layout/activity.rs`, `/ext/plantuml/plantuml-little/src/render/svg_activity.rs`

## Reference Test

- Status: `passed`
- Return code: `0`
- Primary log: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/reference-test.stdout.log`

## Trace Diff

- No JSONL trace diff available.
- Suggested stages: `preproc.done, parse.done, layout.dispatch, layout.done, render.prep, render.bounds, svg.final`
- Suggested Rust env: `{'PUML_TRACE_JSONL': '/tmp/rust-trace.jsonl', 'PUML_TRACE_STAGES': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Suggested Java properties: `{'plantuml.trace.jsonl': '/tmp/java-trace.jsonl', 'plantuml.trace.stages': 'preproc.done,parse.done,layout.dispatch,layout.done,render.prep,render.bounds,svg.final'}`
- Rust hooks: `/ext/plantuml/plantuml-little/src/layout/activity.rs`, `/ext/plantuml/plantuml-little/src/render/svg_activity.rs`
- Java hooks: `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/activitydiagram3/ftile/vcompact/FtileWithNoteOpale.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/activitydiagram3/ftile/vcompact/FtileWithNotes.java`, `/ext/plantuml/plantuml/src/main/java/net/sourceforge/plantuml/svek/image/Opale.java`

## Artifacts

- rust_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/rust.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/rust-render.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/rust-render.stderr.log`
- java_render: returncode=0
  artifact: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/java.svg`
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/java-render.stdout.svg`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/java-render.stderr.log`
- reference_test: returncode=0
  stdout: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/reference-test.stdout.log`
  stderr: `/ext/plantuml/plantuml-little/.trace-skill/skill-upgrade3/a0002/reference-test.stderr.log`

## Next Step

- No action required for reference alignment; investigate cargo-run artifact drift only if it matters.

