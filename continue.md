# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 277/296 passed (93.6%)
- **Unit tests**: 2615/2615 (100%)
- **Session gain**: +5 (from 272)

## Remaining 19 failures

### Font metrics (text width/height vs Java AWT):
- jaws7 ×2 (8px width): Note text width
- deployment_mono_multi ×2 (width): Entity name text width with creole markup
- link_tooltip_04 (15px width): URL tooltip text width
- testGradientSprite (1px width): Viewport rounding
- testPolylineSprites (0.57px): Sub-pixel lifeline height

### Layout/engine gaps:
- A0004 (183px width): Old-style activity needs graphviz horizontal branching
- SCXML0004 ×2 (16px height): Pin cluster inner solve lacks rank constraints
- SCXML0003 (1px height): State viewport 1px off
- state_history001 (col 482): Cluster separator y offset 5px
- deployment01 (1px width): DOT layout coordinate difference

### Rendering structural:
- TimingArrowFont ×2: Track rendering differs (headers, signal styles)
- handwritten001: Polygon coordinates differ (seed/variation params)
- TeozTimeline_0007 ×2 (9px width): Gate message width
- A0003 (7px height): Gantt weekly header height
- svgFillColour (9px height): Sprite legend height
