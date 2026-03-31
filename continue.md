# Continue: Reference Test Alignment

## Current State (2026-03-31)

- **Reference tests**: 233/296 passed (78.7%)
- **Baseline at session start**: 221/296 (74.7%)
- **Net gain this session**: +12 tests
- **Git**: main branch

## Next Priority: SequenceArrows remaining issues

Bidirectional arrows (`<->`, `o<->o`, `x<->x`) are now implemented. SequenceArrows_0001
diverges at col 34856 due to Bob left self-message text x-position (participant position
precision issue, same root cause as SequenceLeftMsg 0001/0002).

## Recently Fixed (this session)

| Commit | Fix | Tests |
|--------|-----|-------|
| 1a5bda8 | state composite explicit_source_line | +1 |
| ead403d | Choice node shape=diamond | +1 |
| 325d729 | Puma self-msg note width | +2 |
| 1d20218 | activity word-by-word note | +2 |
| 523add5 | teoz fragment width, hidden arrow, asymmetric spacing | +8 |
| 619cae0 | left self-msg deltaY, arrowhead | +1 |
| a56e64d | teoz note preferred height | 0 (structural) |
| 4caf01f | composite inner height delta +14→+15 | 0 (structural) |
| 46a1c10/f5d940b | inner composite graphviz solve | 0 (architectural) |
| 477b5fc | inner composite child positions | 0 (structural) |
| e5d1a2d | activation bar positioning | +2 |
| bbcd512 | cross (X) decoration + circle offset | 0 (feature) |
| 3a4b63f | bidirectional arrows + decoration order fix | 0 (feature) |

## Diagnosed Remaining Issues

| Issue | Root cause | Priority |
|-------|-----------|----------|
| SequenceArrows 0001/0002 | Bob left self-msg text x-position | MED |
| state_history001 (-50px) | Java 5-level cluster vs single-rect | MED |
| state fork/SCXML (-17px) | inner ranksep vs edge labels | MED |
| TeozTimeline 0007/0009 | group height model | LOW |
| component viewport 2px | Smetana precision | LOW |
| SequenceLeftMsg 0001/0002 | participant x-position 0.5px | LOW |

## Ext Test Infrastructure

| Harness | Tests | Pass |
|---------|-------|------|
| special_ext_reference_state_split.rs | 13 | 8/13 |
| special_ext_reference_seq_split.rs | ~20 | most pass |
| special_ext_reference_activity_split.rs | 2 | 2/2 |
