# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 274/296 passed (92.6%)
- **Session baseline**: 272/296 (91.9%)
- **Net gain this session**: +2 tests  
- **Unit tests**: 2615/2615 (100%)

## Recent changes this session

1. **!! destroy suffix parsing** — Fixed inline `!!` operator for sequence messages (+2 tests)
2. **State pin entity_position** — Set InputPin/OutputPin entity_position for rank=source/sink
3. **Class notes as graphviz nodes** — Notes participate in layout (jaws7 height fixed, width 8px off)
4. **Teoz fragment background rects** — Fragment color background before lifelines (structural fix, still 5px positioning off)
5. **Fragment color propagation** — Full pipeline for classic+teoz sequence fragment colors

## Remaining 22 failures

### By root cause (depth-ordered):

**Deep architecture gaps:**
- A0004 (441px): Legacy activity sync bars — missing `===Name===` parsing + non-linear layout
- state_history001 (31px): History node rank placement in cluster — graphviz rank optimization
- SCXML0004 ×2 (16px): Pin cluster nesting — needs proper cluster approach without regression
- TimingArrowFont ×2 (34-51px): Timing engine height gap

**Layout precision:**
- deployment_mono_multi ×2 (10px): `<U+000A>` counted as extra line break
- TeozTimeline_0009 ×2 (5px): Fragment x/width offset
- TeozTimeline_0007 ×2 (9px): Fragment/gate width calculation
- handwritten001 (22px): Handwritten font metrics
- A0003 (25px): Gantt row height
- svgFillColour (9px): Sprite height in legend

**Small precision:**
- jaws7 ×2 (8px width): Note graphviz positioning vs Java
- link_tooltip_04 (15px width): Tooltip URL text measurement
- deployment01 (1px width): Viewport rounding
- SCXML0003 (1px height): Viewport rounding
- testGradient (1px width): Viewport rounding
- testPolyline (0.5px height): Sub-pixel precision
