# Reference Suite Diagnosis

## Summary

- Passed: `160` / Failed: `136` / Total: `296`
- Pass rate: `54.1%`
- Case identity source: `cached failures.json`
- Validation probes: `10` checked, `10` still failing, `0` now passing

## Diff Classes

- viewport-only: `122`
- coordinate-only: `14`
- structural: `0`

## Top 10 Closest To Passing

- reference_fixtures_dev_jaws_jaws6_puml: `viewport-only` `dh=-2` `SEQUENCE`
  fixture: `tests/fixtures/dev/jaws/jaws6.puml`
  likely chain: `sequence-core`
  validation: `failed`
- reference_fixtures_preprocessor_jaws6_puml: `viewport-only` `dh=-2` `SEQUENCE`
  fixture: `tests/fixtures/preprocessor/jaws6.puml`
  likely chain: `sequence-core`
  validation: `failed`
- reference_fixtures_state_state_concurrent001_puml: `viewport-only` `dh=-2` `STATE`
  fixture: `tests/fixtures/state/state_concurrent001.puml`
  likely chain: `shared-text-body-height`
  validation: `failed`
- reference_fixtures_state_state_note001_puml: `viewport-only` `dh=+2` `STATE`
  fixture: `tests/fixtures/state/state_note001.puml`
  likely chain: `shared-text-body-height`
  validation: `failed`
- reference_fixtures_nonreg_scxml_SCXML0005_puml: `viewport-only` `dh=+3` `STATE`
  fixture: `tests/fixtures/nonreg/scxml/SCXML0005.puml`
  likely chain: `shared-text-body-height`
  validation: `failed`
- reference_fixtures_state_scxml0005_puml: `viewport-only` `dh=+3` `STATE`
  fixture: `tests/fixtures/state/scxml0005.puml`
  likely chain: `shared-text-body-height`
  validation: `failed`
- reference_fixtures_nwdiag_basic_puml: `viewport-only` `dh=-5` `NWDIAG`
  fixture: `tests/fixtures/nwdiag/basic.puml`
  likely chain: `family-stage-trace`
  validation: `failed`
- reference_fixtures_erd_chenrankdir_puml: `viewport-only` `dh=-6` `CHEN_EER`
  fixture: `tests/fixtures/erd/chenrankdir.puml`
  likely chain: `family-stage-trace`
  validation: `failed`
- reference_fixtures_nonreg_simple_ChenRankDir_puml: `viewport-only` `dh=-6` `CHEN_EER`
  fixture: `tests/fixtures/nonreg/simple/ChenRankDir.puml`
  likely chain: `family-stage-trace`
  validation: `failed`
- reference_fixtures_nonreg_simple_SequenceLayout_0001c_puml: `viewport-only` `dw=-6` `SEQUENCE`
  fixture: `tests/fixtures/nonreg/simple/SequenceLayout_0001c.puml`
  likely chain: `sequence-core`
  validation: `failed`

## Suggested Workstreams

- shared-text-body-height: `56`
  label: `Shared text/body height`
- sequence-teoz-core: `32`
  label: `Sequence Teoz core`
- family-stage-trace: `25`
  label: `Stage trace first`
- sequence-core: `17`
  label: `Sequence layout core`
- sprite-renderer: `4`
  label: `Sprite renderer`
- graphviz-coordinate-chain: `2`
  label: `Graphviz coordinate chain`

