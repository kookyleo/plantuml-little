# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 254/296 passed (85.8%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain**: +33 tests
- **Lib tests**: 2600/2601

## Remaining 42 failures by root cause

| Category | Tests | Root cause |
|----------|-------|-----------|
| ERD/Chen | 7 | Chen notation shapes (ellipse/diamond) not implemented |
| Graphviz layout | 8+ | Node positioning diffs (component/usecase/state clusters) |
| Preprocessor | 5+ | !include, C4 macros, subdiagram {{ }} |
| Creole rendering | 4+ | Block-level creole in class/component entity bodies |
| Special features | 5+ | Handwritten warning, timing messages, gradient sprites |
| Detail diffs | 10+ | Teoz group width, deployment sizing, XMI metadata |

## Session commits summary

Over 40 commits covering: svek LimitFinder, state composites, sequence arrows (bidirectional, cross, circle, boundary, half-arrow), teoz fragments, notes, activation bars, skin rose theme, sprites, creole rendering, ERD pipeline, class parser, mindmap positioning.

All changes follow Java-first alignment principle.
