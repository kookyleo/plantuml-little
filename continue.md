# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 274/296 passed (92.6%)
- **Session baseline**: 272/296 (91.9%)
- **Net gain this session**: +2 tests
- **Unit tests**: 2615/2615 (100%)

## Engines implemented this session

1. **Activity sync bars** (`===Name===`) — full parser + layout + render pipeline
2. **Teoz fragment background rects** — color background before lifelines
3. **Class notes as graphviz nodes** — layout participation
4. **Fragment color propagation** — full classic + teoz pipeline

## Remaining 22 failures

### Deep architecture / engine gaps:
- A0004 (182px): Sync bars exist but edges not yet routed through them
- state_history001 (31px): Cluster composite history node positioning
- SCXML0004 ×2 (16px): Pin cluster nesting needs cluster approach
- TimingArrowFont ×2 (34-51px): Timing doesn't respect skinparam font sizes
- handwritten001 (22px): Full handwritten wavy rendering needed

### Precision / layout:
- deployment_mono_multi ×2 (10px): Description body line counting
- TeozTimeline_0009 ×2 (5px): Fragment x/width offset
- TeozTimeline_0007 ×2 (9px): Fragment/gate width
- A0003 (25px): Gantt scale factor should scale all dimensions
- svgFillColour (9px): Sprite height in legend
- jaws7 ×2 (8px width): Note graphviz width precision
- link_tooltip_04 (15px): URL tooltip text width

### Viewport rounding (1px):
- deployment01 (1px width)
- SCXML0003 (1px height)
- testGradient (1px width)
- testPolyline (0.5px height)
