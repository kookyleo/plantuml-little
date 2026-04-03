# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 276/296 passed (93.2%)
- **Unit tests**: 2615/2615 (100%)

## Progress this session (272→276, +4)

### Engines supplemented:
1. Activity sync bars + edge routing (A0004 height exact match 697px)
2. Timing skinparam fonts (5 font sizes, track formula)
3. Gantt scale (horizontal + fonts)
4. Teoz fragment backgrounds + LifeEvent extent
5. Class notes in graphviz
6. Sequence !! destroy
7. Handwritten warning banner
8. State history001 DOT ordering + hidden node bbox

### Key fixes:
- TeozTimelineIssues_0009 ×2: LifeEvent activation level in fragment extent
- State_history001: DOT declaration order, hidden node exclusion, cluster child preservation

## Remaining 20 failures

### Structural (need rendering changes):
- handwritten001 (col 387): Shape→polygon conversion for ALL sequence shapes
- A0004 (width 396→213): Old-style activity needs graphviz-based horizontal layout
- TimingArrowFont ×2 (col 362): Track rendering rewrite (headers, signals)
- state_history001 (col 482): Cluster separator y, source-line attr

### Layout precision:
- SCXML0004 ×2 (16px height): Pin cluster nesting
- deployment_mono_multi ×2 (10px height): Entity body line counting
- TeozTimeline_0007 ×2 (9px width): Gate message width
- A0003 (7px height): Gantt weekly header structure
- svgFillColour (9px height): Sprite height in legend
- jaws7 ×2 (8px width): Note text width measurement

### Small precision:
- link_tooltip_04 (15px width): URL tooltip text width
- deployment01 (1px width): Viewport rounding
- SCXML0003 (1px height): Viewport rounding
- testGradient (1px width): Viewport rounding
- testPolyline (0.5px): Sub-pixel lifeline height
