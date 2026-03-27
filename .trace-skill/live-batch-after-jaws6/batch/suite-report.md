# Reference Suite Diagnosis

## Summary

- Passed: `160` / Failed: `136` / Total: `296`
- Pass rate: `54.1%`
- Case identity source: `refreshed failures.json (cached diff inventory)`
- Analysis tier: `cached-diff-inventory`
- Worktree: `dirty` (17 changed paths)
- Authority baseline: `101` / `296` passed (`34.1%`)
- Cached vs authority gap: `+59` passed, `-59` failed, `+19.9%` pass-rate
- Authority warning: `cached failure inventory and authoritative pass-rate disagree; refresh or validate before using this ranking as a progress signal`
- Validation probes: `10` checked, `8` still failing, `2` now passing
- Trace probes: `5` checked, `0` semantic-equivalent, `0` now passing, `0` errors

## Diff Classes

- viewport-only: `122`
- coordinate-only: `14`
- structural: `0`

## Top 10 Closest To Passing

- reference_fixtures_state_state_concurrent001_puml: `coordinate-only` `dh=-2` `STATE`
  fixture: `tests/fixtures/state/state_concurrent001.puml`
  underlying: `graphviz-coordinate-chain`
  likely chain: `shared-text-body-height`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_state_state_note001_puml: `coordinate-only` `dh=+2` `STATE`
  fixture: `tests/fixtures/state/state_note001.puml`
  underlying: `element-structure-drift`
  likely chain: `shared-text-body-height`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_nonreg_scxml_SCXML0005_puml: `coordinate-only` `dh=+3` `STATE`
  fixture: `tests/fixtures/nonreg/scxml/SCXML0005.puml`
  underlying: `element-structure-drift`
  likely chain: `shared-text-body-height`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_state_scxml0005_puml: `coordinate-only` `dh=+3` `STATE`
  fixture: `tests/fixtures/state/scxml0005.puml`
  underlying: `element-structure-drift`
  likely chain: `shared-text-body-height`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_nwdiag_basic_puml: `coordinate-only` `dh=-5` `NWDIAG`
  fixture: `tests/fixtures/nwdiag/basic.puml`
  underlying: `family-stage-trace`
  likely chain: `family-stage-trace`
  validation: `failed`
  trace: authority=`failed` surface=`coordinate-only` semantic_equivalent=`False`
- reference_fixtures_erd_chenrankdir_puml: `viewport-only` `dh=-6` `CHEN_EER`
  fixture: `tests/fixtures/erd/chenrankdir.puml`
  underlying: `family-stage-trace`
  likely chain: `family-stage-trace`
  validation: `failed`
- reference_fixtures_nonreg_simple_ChenRankDir_puml: `viewport-only` `dh=-6` `CHEN_EER`
  fixture: `tests/fixtures/nonreg/simple/ChenRankDir.puml`
  underlying: `family-stage-trace`
  likely chain: `family-stage-trace`
  validation: `failed`
- reference_fixtures_nonreg_simple_SequenceLayout_0001c_puml: `viewport-only` `dw=-6` `SEQUENCE`
  fixture: `tests/fixtures/nonreg/simple/SequenceLayout_0001c.puml`
  underlying: `sequence-core`
  likely chain: `sequence-core`
  validation: `failed`
- reference_fixtures_sequence_sequencelayout_0001c_puml: `viewport-only` `dw=-6` `SEQUENCE`
  fixture: `tests/fixtures/sequence/sequencelayout_0001c.puml`
  underlying: `sequence-core`
  likely chain: `sequence-core`
- reference_fixtures_nonreg_simple_SequenceLeftMessageAndActiveLifeLines_0001_puml: `viewport-only` `dh=-8` `SEQUENCE`
  fixture: `tests/fixtures/nonreg/simple/SequenceLeftMessageAndActiveLifeLines_0001.puml`
  underlying: `sequence-core`
  likely chain: `sequence-teoz-core`

## Cached But Now Passing

- reference_fixtures_dev_jaws_jaws6_puml: validation=`passed` fixture=`tests/fixtures/dev/jaws/jaws6.puml`
- reference_fixtures_preprocessor_jaws6_puml: validation=`passed` fixture=`tests/fixtures/preprocessor/jaws6.puml`

## Suggested Workstreams

- shared-text-body-height: `58`
  label: `Shared text/body height`
- sequence-teoz-core: `32`
  label: `Sequence Teoz core`
- family-stage-trace: `25`
  label: `Stage trace first`
- sequence-core: `17`
  label: `Sequence layout core`
- sprite-renderer: `2`
  label: `Sprite renderer`
- graphviz-coordinate-chain: `2`
  label: `Graphviz coordinate chain`

