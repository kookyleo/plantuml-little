# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 247/296 passed (83.4%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain**: +26 tests

## Recent deep alignment

| Commit | Java alignment |
|--------|---------------|
| `2a3c11a` | boundary arrow border1/border2 |
| `9462e92` | reverse self-msg posC2 constraint |
| `36396fc` | shadow deltaShadow in participant/note extent |
| `343b15a` | skin rose border colors and stroke-width |
| `e44ee0a` | self-msg text width (creole), cross offsets |
| `60931e4` | cluster title centering, sprite bg color |
| `95f2d3c` | teoz note Y contact-point, rendering order |

## Remaining 49 failures

| Domain | Count | Key blockers |
|--------|-------|-------------|
| Sequence (teoz) | ~12 | SequenceArrows_0002 multi-level activation, timeline width/height |
| Component | ~8 | deployment, subdiagram, C4 |
| State | 3 | scxml0004 width, history001 cluster, scxml0003 precision |
| LeftMsg_0001 | 2 | skin rose activation rect positions |
| Misc | ~24 | sprite, ERD, timing, gantt, handwritten |
