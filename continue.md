# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 274/296 passed (92.6%)
- **Unit tests**: 2615/2615 (100%)

## Engines supplemented this session

1. **Activity sync bars** — `===Name===` parse + layout + render + edge convergence (A0004 exact height match 697px)
2. **Timing skinparam fonts** — 5 per-element font sizes, track height formula matching Java
3. **Gantt full scale** — `scale N` applies to all dimensions including vertical + fonts
4. **Teoz fragment backgrounds** — color rects before lifelines, full color pipeline
5. **Class notes in graphviz** — notes as layout-participating nodes
6. **Sequence !! destroy** — inline destroy suffix parsing

## Remaining 22 failures by category

### Still need engine work:
- handwritten001 (22px): Full handwritten wavy shape rendering (hand.rs integration)
- A0004 (width 213→396): Sequential layout can't do horizontal branching like DOT

### Layout precision:
- state_history001 (31px): Cluster composite history node positioning
- SCXML0004 ×2 (16px): Pin cluster nesting
- deployment_mono_multi ×2 (10px): Description body vs name \n handling
- TeozTimeline_0009 ×2 (5px): Fragment x/width offset
- TeozTimeline_0007 ×2 (9px): Fragment/gate width
- A0003 (25px): Gantt weekly header structure differs from Java
- svgFillColour (9px): Sprite height in legend
- TimingArrowFont ×2: SVG element ordering differs from Java

### Small precision:
- jaws7 ×2 (8px): Note graphviz width
- link_tooltip_04 (15px): URL tooltip text width
- deployment01 (1px): Viewport rounding
- SCXML0003 (1px): Viewport rounding
- testGradient (1px): Viewport rounding
- testPolyline (0.5px): Sub-pixel precision
