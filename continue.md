# Continue: State Diagram Alignment

## Scope

Continue from the current state-diagram alignment work. The active target is now `tests/fixtures/state/state_history001.puml`.

Use `python3`, not `python`, for all local scripts.

## Current Progress

1. `state_monoline_03` was split into isolated special ext cases, fixed, and the original reference test now passes.
2. The special ext split workflow is stable. `cargo test --test special_ext_reference_state_split -- --ignored --nocapture` currently passes `12/12`.
3. `state_history001` has been significantly improved:
   - History node ownership, rendering, and transition endpoints are correct
   - Exposed history nodes now participate in the outer Graphviz solve
   - Composite children use Graphviz inner solve (fixes cycle handling)
   - Cluster grouping groups composite + history for rank assignment

## What Was Fixed Already

1. Nested history ownership is preserved in the parser and model, so `Active[H]` is treated as a child of `Active` instead of a stray top-level node.
2. State history rendering is much closer to Java now:
   - history marker uses filled ellipse styling
   - `H` text is regular-weight at Java-like sizing
   - transition endpoint display maps to `*historical*Active` / `*deephistorical*...`
3. The outer state Graphviz solve now includes exposed nested history nodes when they are referenced across the top-level boundary.
4. A lightweight cluster path was added for this special history case so the outer solve can include the composite state and its exposed history node in the same cluster.
5. Composite children now use `layout_children_with_graphviz` (real Graphviz solve) instead of `layout_states_ranked` (which collapsed cycles incorrectly).

## Current Remaining Gap

The active failing reference case is:

- `cargo test --test reference_tests reference_fixtures_state_state_history001_puml -- --nocapture`

Current result:

- Rust root height: `454px`
- Reference root height: `404px`

The remaining 50px gap is due to a fundamental architectural difference:

### Java Cluster Approach (desired behavior)
Java puts ALL inner children of the composite state directly into the outer DOT as a Graphviz cluster subgraph (with 5 nested levels: a/p0/main/i/p1). A `zaent` point node serves as the cluster entry/exit for external edges. The cluster's inner children use the outer ranksep, and Graphviz determines the cluster bounds. The rank assignment allows Paused to be positioned ABOVE Active (since `Paused -> Active[H]` is a back-edge).

### Rust Single-Rect Approach (current behavior)
Rust pre-computes the composite's inner layout (via Graphviz inner solve with ranksep=36pt), then represents the composite as a single tall rect in the outer DOT. The tall rect forces Graphviz to allocate more vertical space, and the rank ordering puts Paused BELOW Active (following `Active -> Paused` direction).

### Root Cause Analysis
The single-rect approach produces different Graphviz rank assignments than Java's cluster approach. In Java, `Paused -> Active[H]` where Active[H] is inside the cluster creates a back-edge constraint that places Paused at rank 0 (above the cluster). In Rust, `Active -> Paused` dominates and places Paused at rank 2 (below Active).

### Attempted Full Cluster Approach
A full cluster approach was attempted (putting children as individual nodes in the outer DOT inside nested cluster subgraphs). The cluster bounds matched Java exactly (152x309), but the svek solve's index-based node mapping couldn't correctly extract positions for nodes inside clusters (the color-based position extraction works correctly, but normalization/remapping was misconfigured). This approach is architecturally correct but needs svek solve fixes.

### Next Steps
1. Fix the svek solve to correctly handle nodes inside clusters (the color match works but normalization offsets need per-solve isolation)
2. OR: compute the composite height using the Java cluster formula instead of the single-rect formula
3. OR: match Java's rank ordering by reversing the `Active -> Paused` edge direction when history creates a back-edge constraint

## Key Files
- `src/layout/state.rs` — state layout, composite sizing, DOT generation
- `src/layout/graphviz.rs` — LayoutGraph to DOT conversion
- `src/svek/builder.rs` — DOT builder, clusters
- `src/svek/mod.rs` — svek solve, cluster handling
- `src/render/svg_state.rs` — rendering order

## Verified Commands

```bash
cargo test --test special_ext_reference_state_split -- --ignored --nocapture
cargo test --test reference_tests reference_fixtures_state_state_history001_puml -- --nocapture
RUST_LOG=debug cargo run -- tests/fixtures/state/state_history001.puml -o /tmp/state_history001.svg
```

Java DOT debug:
```bash
java -jar /ext/plantuml/plantuml/build/libs/$(ls /ext/plantuml/plantuml/build/libs/ | grep 'plantuml-.*\.jar$' | grep -v sources | grep -v javadoc | sort | tail -1) -debugsvek tests/ext_fixtures/state/state_ext_history_simple.puml
```
