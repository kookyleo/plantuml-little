# Continue: Reference Test Alignment

## Current State (2026-03-31)

- **Reference tests**: 241/296 passed (81.4%)
- **Baseline at session start**: 221/296 (74.7%)
- **Net gain this session**: +20 tests
- **Git**: main branch

## State diagram: deep alignment achieved
- 7 state failures → 3 remaining (scxml0004 width, history001 cluster, state_note001 precision)
- Key bottom-up fixes: LimitFinder corrections, composite inner margin/nodesep, pin states, concurrent regions, note anchors, entity ordering

## Remaining 55 failures by domain
- Sequence (teoz): ~16 (timeline, arrows, left-msg)
- Component: ~11 (deployment, jaws, subdiagram)
- State: 3 (deep architectural)
- Sprite: 3
- Chen ERD: 3
- Activity: 2
- Timing: 2
- Misc: ~15 (various)
