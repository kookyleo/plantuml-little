# Reference Test Alignment Plan

**Current**: 19/296 passing (6.4%) — 56 commits
**Target**: 296/296 (100%)

## Root Cause Diagnosis (277 failing fixtures)

| Priority | Root Cause | Fixtures | Effort | Notes |
|----------|-----------|----------|--------|-------|
| **P0** | WRONG_DIAGRAM_TYPE | 33 | Low | Parser detects wrong type for preprocessor fixtures |
| **P1** | SEQUENCE_LAYOUT | 90 | High | Sequence renderer sizing completely off |
| **P2** | SKINPARAM_STYLE | 26 | Medium | Colors/fonts not flowing through skinparam system |
| **P3** | GROUP_CONTAINER | 22 | Medium | package/rectangle/folder container rendering |
| **P4** | COMPONENT_LAYOUT | 19 | Medium | Component diagram Graphviz params |
| **P5** | STATE_LAYOUT | 17 | Medium | State diagram Graphviz params + sizing |
| **P6** | ACTIVITY_LAYOUT | 16 | Medium | Activity diagram sizing |
| **P7** | META_ELEMENTS | 12 | Low | title/header/footer/caption/legend sizing |
| **P8** | CLASS_LAYOUT_OTHER | 9 | Medium | Remaining class issues (funcparam, jaws) |
| **P9** | ERD_LAYOUT | 9 | High | ERD uses completely different Graphviz layout |
| **P10** | WBS_LAYOUT | 6 | Low | WBS tree sizing |
| **P11** | GENERICS | 5 | Low | Generic type box (<T>) |
| **P12** | Other (GANTT, JSON, SALT, etc.) | 13 | Varies | Individual diagram types |

## Execution Plan

### Phase 1: Quick Wins (P0 + P7 + P11) — Target: +50 fixtures

#### P0: Fix Diagram Type Detection (33 fixtures)

**Problem**: Preprocessor-expanded `.puml` files get detected as wrong diagram type.
The type detection happens in `src/parser/mod.rs` after preprocessing.

**Specific mismatches**:
- `CLASS → SEQUENCE` (16): Teoz/seq fixtures detected as class because `!pragma teoz` or
  participant declarations aren't recognized after preprocessing
- `DESCRIPTION → CLASS` (10): Component fixtures detected as description instead of class
- `ACTIVITY → CLASS` (3): Activity fixtures misdetected
- `DESCRIPTION → ACTIVITY` (2): Old activity syntax not recognized
- `DESCRIPTION → SEQUENCE` (2): Sequence fixtures misdetected

**Fix**: Improve `detect_diagram_type()` in `src/parser/common.rs`:
- Check for sequence keywords BEFORE class keywords (participant, actor, ->)
- Check for old activity syntax ((*), if/then/else)
- Check for component keywords (component, node, database, etc.)

**Files**: `src/parser/mod.rs`, `src/parser/common.rs`

#### P7: Meta Element Sizing (12 fixtures)

**Problem**: `wrap_with_meta()` in `src/render/svg.rs` computes wrong canvas size
when title/header/footer/caption/legend are present.

**Fix**: Read Java's `AnnotatedBuilder` / `AnnotatedWorker` to understand
exact meta element sizing and positioning.

**Files**: `src/render/svg.rs` (wrap_with_meta)

#### P11: Generic Type Box (5 fixtures)

**Problem**: Class entities with generics like `class MyList<T>` should render a small
dashed rect at the top-right corner showing `T`.

**Fix**: Read Java's `EntityImageClassHeader` generic block rendering.
Add `genericBlock` rendering in `draw_entity_box`.

**Files**: `src/render/svg.rs` (draw_entity_box)

### Phase 2: Diagram-Type Layout Alignment — Target: +150 fixtures

Each diagram type needs its Graphviz parameters and canvas calculation aligned.
These can be done **in parallel** (each touches different renderer files).

#### P1: Sequence Diagram (90 fixtures)

The largest single category. Requires:
1. Correct participant sizing (text width + padding)
2. Correct message spacing (vertical gaps between messages)
3. Correct activation box rendering
4. Correct fragment/group/alt/ref sizing
5. Canvas calculation matching Java's sequence-specific logic

**Key Java files**: `SequenceDiagram.java`, `SequenceDiagramArea.java`,
`SequenceDiagramFileMakerTeoz.java`
**Our files**: `src/layout/sequence.rs`, `src/render/svg_sequence.rs`

#### P4: Component Layout (19 fixtures)

**Fix**: Align Graphviz params for component diagrams.
**Our files**: `src/layout/component.rs`, `src/render/svg_component.rs`

#### P5: State Layout (17 fixtures)

**Fix**: Align Graphviz params and entity sizing for state diagrams.
**Our files**: `src/layout/state.rs`, `src/render/svg_state.rs`

#### P6: Activity Layout (16 fixtures)

**Fix**: Align activity diagram sizing.
**Our files**: `src/layout/activity.rs`, `src/render/svg_activity.rs`

### Phase 3: Cross-Cutting Features — Target: +50 fixtures

#### P2: Skinparam Style System (26 fixtures)

**Problem**: Many skinparam properties not correctly applied:
- Font sizes from skinparam not reaching renderers
- Colors from skinparam not applied to all elements (e.g., separator lines)
- Background color rects not rendered

**Fix**: Audit every skinparam accessor and trace through to SVG output.

#### P3: Group/Container Rendering (22 fixtures)

**Problem**: `package`, `rectangle`, `folder`, `frame` containers not rendered
or sized correctly. Affects both class and component diagrams.

**Fix**: Implement group/container rendering with correct nesting,
borders, and label positioning.

### Phase 4: Remaining Diagram Types — Target: +30 fixtures

- ERD (9): Complete layout overhaul needed
- WBS (6): Tree sizing alignment
- GANTT (2): Timeline sizing
- JSON/YAML (4): Tree/table sizing
- SALT (1): Widget sizing
- Other individual fixes

## Parallel Execution Strategy

```
Phase 1 (Quick Wins):
  Agent A: P0 (diagram type detection) → parser/mod.rs, parser/common.rs
  Agent B: P7 (meta elements) → render/svg.rs wrap_with_meta
  Agent C: P11 (generics box) → render/svg.rs draw_entity_box

Phase 2 (per-type, parallel):
  Agent D: P1 (sequence) → layout/sequence.rs, render/svg_sequence.rs
  Agent E: P4 (component) → layout/component.rs, render/svg_component.rs
  Agent F: P5 (state) → layout/state.rs, render/svg_state.rs
  Agent G: P6 (activity) → layout/activity.rs, render/svg_activity.rs

Phase 3 (cross-cutting, sequential):
  P2 (skinparam) → style.rs, all renderers
  P3 (groups) → parser/class.rs, layout/mod.rs, render/svg.rs

Phase 4 (remaining):
  Individual fixes per diagram type
```

## Standard Fix Workflow

For each root cause:
1. Find the fixture with smallest viewBox diff (softest target)
2. Read the Java source for the specific feature
3. Instrument Java if needed for precise values
4. Port the algorithm to Rust — exact equivalence, no approximation
5. Verify the target fixture passes
6. Run full `cargo test --test reference_tests` to count improvements
7. Commit with clear description of Java source references

## Iron Rule

**Never fit numbers. Read the Java source code and use the exact same
algorithms and parameters. No shortcuts, no approximations. Every
constant must have a Java source code reference.**
