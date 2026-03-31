# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 245/296 passed (82.8%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain**: +24 tests

## Key bottom-up alignments done this session

| Area | Java alignment |
|------|---------------|
| svek LimitFinder | DotPath bounds, body separator ULine, note max_corr |
| State composites | inner nodesep, concurrent regions, pin states, entity order |
| State notes | graphviz edge endpoints for anchors, Opale path format |
| Sequence arrows | bidirectional, cross/circle decorations, half-arrow polygons |
| Sequence self-msg | deltaY shift, activation bars, text width (creole) |
| Teoz fragments | hidden arrows, fragment width/height, note Y contact-point |
| Component | hex sprites, cluster title centering, sprite bg color |
| ERD | svek/graphviz pipeline for layout |
| Activity | word-by-word note rendering |
| Class | creole table rendering in rectangle bodies |

## Remaining 51 failures

| Domain | Count | Key blockers |
|--------|-------|-------------|
| Sequence (teoz) | ~14 | gate/boundary arrows, timeline height, left-msg width |
| Component | ~8 | deployment height, subdiagram, C4 |
| State | 3 | scxml0004 width, history001 cluster, scxml0003 precision |
| Sprite | 3 | gradient, polyline, fill-colour |
| Chen ERD | 3 | ChenMovie label positions |
| Misc | ~20 | timing, handwritten, gantt, A0003/A0004 |
