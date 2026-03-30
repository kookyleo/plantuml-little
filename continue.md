# Continue: Reference Test Alignment

## Current State (2026-03-30)

- **Reference tests**: 231/296 passed (78.0%)
- **Special ext split tests**: all pass
- **Git**: main branch, clean

## Active Target: state_history001

- Rust=449px, Java=404px, gap=45px
- Root cause: Java uses 5-level nested cluster (a/p0/main/i/p1) for composites in outer DOT; we use simpler 2-level
- An inner-solve attempt (commit 46a1c10) was tried and reverted — made it worse
- Next: port Java's cluster nesting structure, or use Smetana inner solve properly

## Recently Fixed (this session, +10 tests)

| Test | Fix | Commit |
|------|-----|--------|
| state_monoline_03 | composite explicit_source_line | 1a5bda8 |
| state_choice001 | Diamond shape in DOT | ead403d |
| SequenceLayout_0003 ×2 | note width (left overflow + right extent) | 325d729 |
| activity_a0002 ×2 | word-by-word note + style block detect | 1d20218 |
| TeozAltElseParallel ×8 | hidden arrow, fragment header width, nested extent, draw order | 523add5 |
| SequenceLeftMsg_0003 ×1 | self-msg deltaY, return endpoint | 619cae0 |

## Diagnosed But Unresolved

| Issue | Root cause | Fix direction |
|-------|-----------|---------------|
| state_history001 (-45px) | 5-level cluster nesting vs 2-level | Port Java cluster structure |
| state_fork001/scxml0002 (-17px) | Inner composite ranksep doesn't account for edge labels | Use Smetana inner solve |
| TeozTimeline 0007/0009 | group/activation height model | Align GroupingTile.getPreferredHeight |
| SequenceArrows self-msg | SVG element order | Align drawU output order |
| SequenceLeftMsg 0001/0002 | teoz left self-msg width + multi-level activation | Continue ext case splitting |

## Ext Ref Split Workflow

Methodology doc: `docs/special_ext_reference_split_workflow.md`

Test harnesses:
- `tests/special_ext_reference_state_split.rs` (state)
- `tests/special_ext_reference_seq_split.rs` (sequence)
- `tests/special_ext_reference_activity_split.rs` (activity)

Fixtures: `tests/ext_fixtures/{state,sequence,activity}/`
