# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 256/296 passed (86.5%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain**: +35 tests
- **Unit tests**: 2605/2605 (100%)

## Session summary

Over 60+ commits covering deep Java-first alignment across all diagram types:
- svek/LimitFinder precision (DotPath bounds, body separator, cluster corrections)
- State composites (inner graphviz solve, concurrent regions, pin states, history)
- Sequence arrows (bidirectional, cross, circle, boundary, half-arrow, activation bars)
- Teoz fragments (hidden arrows, width/height, note positioning, activation levels)
- Component (creole names, sprites, clusters, direction hints, note layout)
- ERD (graphviz pipeline, Chen shapes, attribute rendering)
- Creole (block-level: tables, bullets, rules; inline: `<u:blue>`, `<U+XXXX>`, `<code>`)
- Skin theme (rose borders, shadow, line-thickness)
- Class (as alias, spot stereotypes, display_name)

## Remaining 40 failures

All require deep structural or feature work:
- Graphviz coordinate precision (7+ ERD/component)
- State cluster architecture (3)
- Teoz timeline/group calculations (4)
- C4/subdiagram features (5)
- Timing messages (2)
- Sprite rendering (3)
- Gantt/handwritten/legacy (5+)
- Deployment/usecase layout (5+)
- Class link tooltips (1)
