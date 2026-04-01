# Continue: Reference Test Alignment

## Current State (2026-04-01)

- **Reference tests**: 253/296 passed (85.5%)
- **Session baseline**: 221/296 (74.7%)
- **Net gain**: +32 tests

## Remaining 43 failures

Most remaining failures require deeper structural work:
- Graphviz layout precision differences (state composite clusters)
- Missing preprocessor features (!include, !define for C4)
- Creole graphical elements (horizontal rules, bullet points in entities)
- Teoz group width/height calculations
- Special features (handwritten mode, timing messages, gradient sprites)
- Subdiagram embedding ({{ }})
