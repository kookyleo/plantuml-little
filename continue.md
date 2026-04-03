# Continue: Reference Test Alignment

## Current State (2026-04-03)

- **Reference tests**: 290/296 passed (98.0%)
- **Unit tests**: 2616/2616 (100%)
- **Session gain**: +18 (from 272)

## Remaining 6 failures (deep layout gaps)

1. **SCXML0003** (1px height): State viewport bounding box rounding
2. **TimingArrowFont_0001** / **_0002**: Timing robust track rect-band vs Java line-signal rendering
3. **deployment01** (33px y-offset): Component Node polygon y from graphviz positioning
4. **A0003** (7px height): Gantt weekly header structure
5. **A0004** (183px width): Old-style activity needs graphviz horizontal branching

## Key achievements this session

### Engines implemented:
- Activity sync bars + edge convergence routing
- Timing skinparam fonts (5 per-element sizes)
- Gantt scale (horizontal + fonts)
- Teoz fragment backgrounds + LifeEvent extent
- Class notes as graphviz nodes with Opale path rendering
- Handwritten warning banner + wavy lifelines
- State cluster simulateCompound edge clipping

### Structural rendering fixes:
- Node 3D polygon with fold lines
- Timing chart borders + grid + participant name ordering
- Gate circle direction + divider height
- Creole underline/color NBSP text element splitting
- Note entity `<g>` wrapper with GMN ID numbering
- State composite separator y conditional on cluster type
- SCXML nested pin bonus propagation
