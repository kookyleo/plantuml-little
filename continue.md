# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 281/296 passed (94.9%)
- **Unit tests**: 2615/2615 (100%)
- **Session gain**: +9 (from 272)

## Remaining 15 failures

### Rendering structural:
- TeozTimeline_0007 ×2: Note-message interleave order
- TimingArrowFont ×2: Font color/family in track text
- handwritten001: RNG polygon coordinates
- deployment01: Node polygon y-offset from graphviz

### Layout:
- SCXML0004 ×2: Pin cluster width 340→266
- A0004: Activity width 213→396
- A0003: Gantt height 149→156
- SCXML0003: 1px viewport height

### Precision:
- svgFillColour: Sprite legend height 9px
- testGradient: Lifeline height 0.8px
- testPolyline: Lifeline height 0.6px
