# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 277/296 passed (93.6%)
- **Unit tests**: 2615/2615 (100%)
- **Session gain**: +5 (from 272)

## Remaining 19 failures — root cause classification

### Font metrics / text width (Java AWT vs Rust):
- jaws7 ×2 (8px width): Note text width at 13pt
- link_tooltip_04 (15px width): URL tooltip text measurement
- testGradientSprite (1px width): Viewport from text width diff
- svgFillColour (9px height): Sprite legend height

### Rendering structural:
- deployment_mono_multi ×2 (col 490): Node 3D polygon inner line details
- deployment01 (col 454): Node polygon point coordinates
- TimingArrowFont ×2 (col 362): Timing track rendering (grid, headers)
- TeozTimeline_0007 ×2 (col 3472): Gate circle vs ellipse rendering
- state_history001 (col 482): Cluster separator y 5px off

### Layout engine:
- A0004 (width 396→213): Activity horizontal branching (needs graphviz)
- SCXML0004 ×2 (16px height): Pin cluster rank constraints
- A0003 (7px height): Gantt weekly header structure

### Viewport rounding:
- SCXML0003 (1px height)
- testPolylineSprites (0.57px): Sub-pixel lifeline height
- handwritten001 (col 387): RNG seed/variation params

## Key commits this session
- Activity sync bars + edge routing (A0004 height exact match)
- State history001 DOT ordering + hidden node bbox
- Teoz fragment LifeEvent extent + gate border constraint
- Timing skinparam fonts (5 per-element sizes)
- Gantt scale (horizontal + fonts only)
- Class notes as graphviz nodes
- Creole text width with markup stripping
- Node 3D polygon rendering
- LF polygon hack (HACK_X_FOR_POLYGON)
- Handwritten warning banner + wavy lifelines
