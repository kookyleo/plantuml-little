# Reference Suite Diagnosis

## Summary

- Passed: `221` / Failed: `75` / Total: `296`
- Pass rate: `74.7%`
- Case identity source: `refreshed failures.json (cached diff inventory)`
- Analysis tier: `cached-diff-inventory`
- Worktree: `dirty` (9 changed paths)
- Authority baseline: `221` / `296` passed (`74.7%`)
- Cached vs authority gap: `+0` passed, `+0` failed, `+0.0%` pass-rate
- Validation probes: `5` checked, `5` still failing, `0` now passing
- Trace probes: `3` checked, `0` semantic-equivalent, `0` now passing, `0` errors

## Diff Classes

- viewport-only: `68`
- coordinate-only: `5`
- structural: `2`

## Top 10 Closest To Passing

- reference_fixtures_state_state_monoline_03_puml: `coordinate-only` `dh=-1` `STATE`
  fixture: `tests/fixtures/state/state_monoline_03.puml`
  underlying: `graphviz-coordinate-chain`
  likely chain: `shared-text-body-height`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_nonreg_scxml_SCXML0003_puml: `coordinate-only` `dh=-6` `STATE`
  fixture: `tests/fixtures/nonreg/scxml/SCXML0003.puml`
  underlying: `element-structure-drift`
  likely chain: `shared-text-body-height`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_nonreg_simple_ChenRankDir_puml: `coordinate-only` `dh=-6` `CHEN_EER`
  fixture: `tests/fixtures/nonreg/simple/ChenRankDir.puml`
  underlying: `family-stage-trace`
  likely chain: `family-stage-trace`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_state_scxml0003_puml: `viewport-only` `dh=-6` `STATE`
  fixture: `tests/fixtures/state/scxml0003.puml`
  underlying: `graphviz-coordinate-chain`
  likely chain: `shared-text-body-height`
  validation: `failed`
- reference_fixtures_nonreg_simple_SequenceArrows_0002_puml: `viewport-only` `dw=+8` `SEQUENCE`
  fixture: `tests/fixtures/nonreg/simple/SequenceArrows_0002.puml`
  underlying: `sequence-core`
  likely chain: `sequence-teoz-core`
  validation: `failed`
- reference_fixtures_nonreg_simple_TeozTimelineIssues_0002_puml: `viewport-only` `dh=-12` `SEQUENCE`
  fixture: `tests/fixtures/nonreg/simple/TeozTimelineIssues_0002.puml`
  underlying: `sequence-core`
  likely chain: `sequence-teoz-core`
- reference_fixtures_preprocessor_teoztimelineissues_0002_puml: `viewport-only` `dh=-12` `SEQUENCE`
  fixture: `tests/fixtures/preprocessor/teoztimelineissues_0002.puml`
  underlying: `sequence-core`
  likely chain: `sequence-teoz-core`
- reference_fixtures_dev_jaws_jaws3_puml: `viewport-only` `dh=+13` `CLASS`
  fixture: `tests/fixtures/dev/jaws/jaws3.puml`
  underlying: `shared-text-body`
  likely chain: `shared-text-body-height`
- reference_fixtures_nonreg_simple_SequenceLeftMessageAndActiveLifeLines_0001_puml: `viewport-only` `dw=-13` `SEQUENCE`
  fixture: `tests/fixtures/nonreg/simple/SequenceLeftMessageAndActiveLifeLines_0001.puml`
  underlying: `sequence-core`
  likely chain: `sequence-teoz-core`
- reference_fixtures_preprocessor_jaws3_puml: `viewport-only` `dh=+13` `CLASS`
  fixture: `tests/fixtures/preprocessor/jaws3.puml`
  underlying: `shared-text-body`
  likely chain: `shared-text-body-height`

## Suggested Workstreams

- shared-text-body-height: `33`
  label: `Shared text/body height`
- sequence-teoz-core: `23`
  label: `Sequence Teoz core`
- family-stage-trace: `11`
  label: `Stage trace first`
- sequence-core: `5`
  label: `Sequence layout core`
- sprite-renderer: `1`
  label: `Sprite renderer`
- cluster-shape-limitfinder: `1`
  label: `Cluster shape / LimitFinder`
- graphviz-coordinate-chain: `1`
  label: `Graphviz coordinate chain`

