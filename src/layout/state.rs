//! State diagram layout engine.
//!
//! Converts a `StateDiagram` into a fully positioned `StateLayout` ready for
//! SVG rendering.  The algorithm uses a rank-based placement strategy where
//! states connected by transitions are placed on successive rows, while
//! unconnected states share the same row side-by-side.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::model::state::{State, StateDiagram, StateKind, Transition};
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned state diagram ready for rendering.
#[derive(Debug)]
pub struct StateLayout {
    pub width: f64,
    pub height: f64,
    pub state_layouts: Vec<StateNodeLayout>,
    pub transition_layouts: Vec<TransitionLayout>,
    pub note_layouts: Vec<StateNoteLayout>,
}

/// A single positioned state node.
#[derive(Debug, Clone)]
pub struct StateNodeLayout {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub description: Vec<String>,
    pub stereotype: Option<String>,
    pub is_initial: bool,
    pub is_final: bool,
    pub is_composite: bool,
    pub children: Vec<StateNodeLayout>,
    /// Pseudo-state kind (fork, join, choice, history, etc.)
    pub kind: StateKind,
    /// Y positions of concurrent region separators (dashed lines)
    pub region_separators: Vec<f64>,
}

/// A transition edge between two states.
#[derive(Debug, Clone)]
pub struct TransitionLayout {
    pub from_id: String,
    pub to_id: String,
    pub label: String,
    pub points: Vec<(f64, f64)>,
}

/// A positioned note.
#[derive(Debug, Clone)]
pub struct StateNoteLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CHAR_WIDTH: f64 = 7.2;
const LINE_HEIGHT: f64 = 16.0;
const PADDING: f64 = 10.0;
/// Java: state name uses FontParam.STATE = SansSerif 14pt.
const STATE_NAME_FONT_SIZE: f64 = 14.0;
/// Java: state body/description uses FontParam.STATE_ATTRIBUTE = SansSerif 12pt.
const STATE_DESC_FONT_SIZE: f64 = 12.0;
/// Minimum state dimensions matching Java PlantUML defaults.
const STATE_MIN_WIDTH: f64 = 50.0;
const STATE_MIN_HEIGHT: f64 = 50.0;
/// Vertical gap between rows of states (includes arrow space).
const STATE_SPACING: f64 = 50.0;
const SPECIAL_STATE_RADIUS: f64 = 10.0;
/// Padding inside composite states around children.
const COMPOSITE_PADDING: f64 = 12.0;
/// Header height for composite state name area.
const COMPOSITE_HEADER: f64 = 26.0;
const NOTE_OFFSET: f64 = 30.0;
const FORK_BAR_WIDTH: f64 = 80.0;
const FORK_BAR_HEIGHT: f64 = 8.0;
/// Choice diamond side length.
const CHOICE_SIZE: f64 = 24.0;
const HISTORY_DIAMETER: f64 = 22.0;
const NOTE_MAX_WIDTH: f64 = 200.0;
const MARGIN: f64 = 7.0;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

/// Estimate the pixel width of a single line of text.
fn text_width(text: &str) -> f64 {
    text.len() as f64 * CHAR_WIDTH
}

/// Estimate the size of a simple (non-composite, non-special) state.
/// Returns `(width, height)`.
///
/// Matches Java PlantUML sizing: simple state is 50x50 minimum,
/// header area is ~26px, description lines add ~14px each.
fn estimate_state_size(state: &State) -> (f64, f64) {
    let name_w = text_width(&state.name) + 2.0 * PADDING;

    let desc_w = state
        .description
        .iter()
        .map(|line| text_width(line) + 2.0 * PADDING)
        .fold(0.0_f64, f64::max);

    let stereo_w = state
        .stereotype
        .as_ref()
        .map_or(0.0, |s| text_width(s) + 2.0 * PADDING);

    let width = name_w.max(desc_w).max(stereo_w).max(STATE_MIN_WIDTH);

    // Header (name at 14pt) + optional stereotype + description (at 12pt).
    // Java: EntityImageState layout uses different fonts for name vs body.
    let name_h = crate::font_metrics::line_height("SansSerif", STATE_NAME_FONT_SIZE, false, false);
    let desc_h = crate::font_metrics::line_height("SansSerif", STATE_DESC_FONT_SIZE, false, false);
    let stereo_h = if state.stereotype.is_some() { desc_h } else { 0.0 };
    let desc_total = state.description.len() as f64 * desc_h;
    let height = (name_h + stereo_h + desc_total + 2.0 * PADDING).max(STATE_MIN_HEIGHT);

    (width, height)
}

/// Estimate the size of a note, clamped to `NOTE_MAX_WIDTH`.
fn estimate_note_size(text: &str) -> (f64, f64) {
    let lines: Vec<&str> = text.lines().collect();
    let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let width = (max_line_len as f64 * CHAR_WIDTH + 2.0 * PADDING).min(NOTE_MAX_WIDTH);
    let width = width.max(60.0);
    let height = (lines.len().max(1) as f64 * LINE_HEIGHT + 2.0 * PADDING).max(STATE_MIN_HEIGHT);
    (width, height)
}

// ---------------------------------------------------------------------------
// Determine initial / final status
// ---------------------------------------------------------------------------

/// Determine which `[*]` state IDs serve as initial and which serve as final.
///
/// A `[*]` state is **initial** if it appears as the `from` of a transition.
/// A `[*]` state is **final** if it appears as the `to` of a transition.
fn classify_special_states(
    states: &[State],
    transitions: &[Transition],
) -> (HashSet<String>, HashSet<String>) {
    let special_ids: HashSet<String> = states
        .iter()
        .filter(|s| s.is_special)
        .map(|s| s.id.clone())
        .collect();

    let mut initial_ids = HashSet::new();
    let mut final_ids = HashSet::new();

    for tr in transitions {
        if special_ids.contains(&tr.from) {
            initial_ids.insert(tr.from.clone());
        }
        if special_ids.contains(&tr.to) {
            final_ids.insert(tr.to.clone());
        }
    }

    // If a special state has no transitions at all, default to initial
    for id in &special_ids {
        if !initial_ids.contains(id) && !final_ids.contains(id) {
            initial_ids.insert(id.clone());
        }
    }

    (initial_ids, final_ids)
}

// ---------------------------------------------------------------------------
// Collect implicit states
// ---------------------------------------------------------------------------

/// Collect state IDs that are referenced in transitions but not declared in the
/// state list.  These need synthesized layout entries.
fn collect_implicit_states(states: &[State], transitions: &[Transition]) -> Vec<State> {
    let mut declared: HashSet<String> = HashSet::new();
    collect_declared_ids(states, &mut declared);

    let mut implicit = Vec::new();
    let mut seen = HashSet::new();

    for tr in transitions {
        for id in [&tr.from, &tr.to] {
            if !declared.contains(id.as_str()) && seen.insert(id.clone()) {
                let is_special = id == "[*]" || id.starts_with("[*]");
                let kind = if id.ends_with("[H*]") {
                    StateKind::DeepHistory
                } else if id.ends_with("[H]") {
                    StateKind::History
                } else {
                    StateKind::default()
                };
                implicit.push(State {
                    name: id.clone(),
                    id: id.clone(),
                    description: Vec::new(),
                    stereotype: None,
                    children: Vec::new(),
                    is_special,
                    kind,
                    regions: Vec::new(),
                });
            }
        }
    }

    implicit
}

/// Deduplicate states by ID, preferring composite states over simple ones.
/// When two states have the same ID, the one with children (composite) wins.
/// Descriptions and stereotypes are merged.
fn dedup_states(states: &mut Vec<State>) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut to_remove: Vec<usize> = Vec::new();

    for i in 0..states.len() {
        if let Some(&prev_idx) = seen.get(&states[i].id) {
            let prev_is_composite = !states[prev_idx].children.is_empty()
                || !states[prev_idx].regions.is_empty();
            let curr_is_composite = !states[i].children.is_empty()
                || !states[i].regions.is_empty();

            if curr_is_composite && !prev_is_composite {
                // Current is composite, previous is simple -> remove previous
                to_remove.push(prev_idx);
                seen.insert(states[i].id.clone(), i);
            } else {
                // Previous is composite or both are simple -> remove current
                to_remove.push(i);
            }
        } else {
            seen.insert(states[i].id.clone(), i);
        }
    }

    // Remove duplicates in reverse order to preserve indices
    to_remove.sort_unstable();
    to_remove.dedup();
    for &idx in to_remove.iter().rev() {
        states.remove(idx);
    }
}

/// Recursively collect all declared state IDs (including regions).
fn collect_declared_ids(states: &[State], ids: &mut HashSet<String>) {
    for s in states {
        ids.insert(s.id.clone());
        collect_declared_ids(&s.children, ids);
        for region in &s.regions {
            collect_declared_ids(region, ids);
        }
    }
}

// ---------------------------------------------------------------------------
// Core layout logic
// ---------------------------------------------------------------------------

/// Compute the layout node for a single state (sizing, children layout, etc.)
/// without assigning position. Returns (node, width, height).
fn compute_state_node(
    state: &State,
    transitions: &[Transition],
    initial_ids: &HashSet<String>,
    final_ids: &HashSet<String>,
) -> (StateNodeLayout, f64, f64) {
    let is_initial = initial_ids.contains(&state.id);
    let is_final = final_ids.contains(&state.id);
    let is_composite = !state.children.is_empty() || !state.regions.is_empty();

    if state.is_special {
        let diameter = 2.0 * SPECIAL_STATE_RADIUS;
        return (
            StateNodeLayout {
                id: state.id.clone(),
                name: state.name.clone(),
                x: 0.0,
                y: 0.0,
                width: diameter,
                height: diameter,
                description: Vec::new(),
                stereotype: None,
                is_initial,
                is_final,
                is_composite: false,
                children: Vec::new(),
                kind: state.kind.clone(),
                region_separators: Vec::new(),
            },
            diameter,
            diameter,
        );
    }

    if matches!(state.kind, StateKind::Fork | StateKind::Join) {
        let w = FORK_BAR_WIDTH;
        let h = FORK_BAR_HEIGHT;
        return (
            StateNodeLayout {
                id: state.id.clone(),
                name: state.name.clone(),
                x: 0.0,
                y: 0.0,
                width: w,
                height: h,
                description: Vec::new(),
                stereotype: state.stereotype.clone(),
                is_initial: false,
                is_final: false,
                is_composite: false,
                children: Vec::new(),
                kind: state.kind.clone(),
                region_separators: Vec::new(),
            },
            w,
            h,
        );
    }

    if state.kind == StateKind::Choice {
        let s = CHOICE_SIZE;
        return (
            StateNodeLayout {
                id: state.id.clone(),
                name: state.name.clone(),
                x: 0.0,
                y: 0.0,
                width: s,
                height: s,
                description: Vec::new(),
                stereotype: state.stereotype.clone(),
                is_initial: false,
                is_final: false,
                is_composite: false,
                children: Vec::new(),
                kind: state.kind.clone(),
                region_separators: Vec::new(),
            },
            s,
            s,
        );
    }

    if matches!(state.kind, StateKind::History | StateKind::DeepHistory) {
        let d = HISTORY_DIAMETER;
        return (
            StateNodeLayout {
                id: state.id.clone(),
                name: state.name.clone(),
                x: 0.0,
                y: 0.0,
                width: d,
                height: d,
                description: Vec::new(),
                stereotype: state.stereotype.clone(),
                is_initial: false,
                is_final: false,
                is_composite: false,
                children: Vec::new(),
                kind: state.kind.clone(),
                region_separators: Vec::new(),
            },
            d,
            d,
        );
    }

    if state.kind == StateKind::End {
        let diameter = 2.0 * SPECIAL_STATE_RADIUS;
        return (
            StateNodeLayout {
                id: state.id.clone(),
                name: state.name.clone(),
                x: 0.0,
                y: 0.0,
                width: diameter,
                height: diameter,
                description: Vec::new(),
                stereotype: state.stereotype.clone(),
                is_initial: false,
                is_final: true,
                is_composite: false,
                children: Vec::new(),
                kind: state.kind.clone(),
                region_separators: Vec::new(),
            },
            diameter,
            diameter,
        );
    }

    if is_composite {
        // Composite state: recursively layout children
        let mut all_child_layouts = Vec::new();
        let mut region_separators = Vec::new();
        let mut total_child_w = 0.0_f64;
        let total_child_h: f64;

        // Collect all regions: regions[] + children (last region)
        let mut all_regions: Vec<&[State]> = Vec::new();
        for region in &state.regions {
            all_regions.push(region);
        }
        if !state.children.is_empty() {
            all_regions.push(&state.children);
        }

        if all_regions.len() > 1 {
            // Multiple concurrent regions
            let mut region_y = 0.0;
            for (i, region) in all_regions.iter().enumerate() {
                let (child_layouts, child_w, child_h) = layout_states_ranked(
                    region,
                    transitions,
                    initial_ids,
                    final_ids,
                    0.0,
                    region_y,
                );
                total_child_w = total_child_w.max(child_w);
                region_y += child_h;
                all_child_layouts.extend(child_layouts);

                if i < all_regions.len() - 1 {
                    region_y += STATE_SPACING / 2.0;
                    region_separators.push(region_y);
                    region_y += STATE_SPACING / 2.0;
                }
            }
            total_child_h = region_y;
        } else {
            let (child_layouts, child_w, child_h) = layout_states_ranked(
                &state.children,
                transitions,
                initial_ids,
                final_ids,
                0.0,
                0.0,
            );
            total_child_w = child_w;
            total_child_h = child_h;
            all_child_layouts = child_layouts;
        }

        let inner_width = total_child_w + 2.0 * COMPOSITE_PADDING;
        let inner_height = total_child_h + COMPOSITE_HEADER + COMPOSITE_PADDING;

        let name_w = text_width(&state.name) + 2.0 * PADDING;
        let width = inner_width.max(name_w).max(STATE_MIN_WIDTH);
        let height = inner_height.max(STATE_MIN_HEIGHT);

        return (
            StateNodeLayout {
                id: state.id.clone(),
                name: state.name.clone(),
                x: 0.0,
                y: 0.0,
                width,
                height,
                description: state.description.clone(),
                stereotype: state.stereotype.clone(),
                is_initial,
                is_final,
                is_composite: true,
                children: all_child_layouts,
                kind: state.kind.clone(),
                region_separators,
            },
            width,
            height,
        );
    }

    // Simple state
    let (w, h) = estimate_state_size(state);
    (
        StateNodeLayout {
            id: state.id.clone(),
            name: state.name.clone(),
            x: 0.0,
            y: 0.0,
            width: w,
            height: h,
            description: state.description.clone(),
            stereotype: state.stereotype.clone(),
            is_initial,
            is_final,
            is_composite: false,
            children: Vec::new(),
            kind: state.kind.clone(),
            region_separators: Vec::new(),
        },
        w,
        h,
    )
}

/// Assign ranks to states based on transition graph connectivity.
///
/// States are grouped into rows (ranks): source states get rank 0,
/// their targets rank 1, etc.  States not participating in any transitions
/// within this scope are placed on the same rank as their declaration
/// order peers.
fn assign_ranks(
    state_ids: &[String],
    transitions: &[Transition],
    _initial_ids: &HashSet<String>,
    _final_ids: &HashSet<String>,
) -> Vec<Vec<usize>> {
    let n = state_ids.len();
    if n == 0 {
        return Vec::new();
    }

    let id_to_idx: HashMap<&str, usize> = state_ids
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_str(), i))
        .collect();

    // Identify special [*] states that act as both initial and final.
    // Edges TO these states should not create back-edges for SCC/ranking,
    // since [*] logically represents two separate nodes (start dot + end dot).
    let special_set: HashSet<usize> = (0..n)
        .filter(|&i| {
            state_ids[i] == "[*]" || state_ids[i].starts_with("[*]")
        })
        .collect();

    // Build adjacency from transitions scoped to this level.
    // Edges to special [*] states are excluded from the rank graph
    // to avoid artificial cycles (start and end are logically separate).
    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_degree: Vec<usize> = vec![0; n];
    let mut has_edge: Vec<bool> = vec![false; n];

    for tr in transitions {
        if let (Some(&fi), Some(&ti)) = (id_to_idx.get(tr.from.as_str()), id_to_idx.get(tr.to.as_str())) {
            // Skip self-loops for ranking
            if fi == ti {
                has_edge[fi] = true;
                continue;
            }

            // Skip edges TO special [*] states for ranking purposes.
            // These represent "go to final state" and shouldn't create cycles.
            if special_set.contains(&ti) {
                has_edge[fi] = true;
                has_edge[ti] = true;
                continue;
            }

            out_edges[fi].push(ti);
            in_degree[ti] += 1;
            has_edge[fi] = true;
            has_edge[ti] = true;
        }
    }

    // Topological rank assignment with cycle breaking.
    //
    // 1. Find strongly connected components (SCCs) and collapse them.
    // 2. Rank the DAG of SCCs using longest-path from sources.
    // 3. States within the same SCC get the same rank.

    // DFS-based Tarjan's SCC algorithm
    let mut scc_id: Vec<i32> = vec![-1; n];
    let mut scc_stack: Vec<usize> = Vec::new();
    let mut on_stack: Vec<bool> = vec![false; n];
    let mut dfs_num: Vec<i32> = vec![-1; n];
    let mut dfs_low: Vec<i32> = vec![0; n];
    let mut dfs_counter: i32 = 0;
    let mut num_sccs: usize = 0;

    // Iterative Tarjan
    {
        // Use a work stack to avoid recursion
        enum Action { Visit(usize), Finish(usize) }
        let mut work: Vec<Action> = Vec::new();

        for start in 0..n {
            if dfs_num[start] >= 0 {
                continue;
            }
            work.push(Action::Visit(start));

            while let Some(action) = work.pop() {
                match action {
                    Action::Visit(u) => {
                        if dfs_num[u] >= 0 {
                            continue;
                        }
                        dfs_num[u] = dfs_counter;
                        dfs_low[u] = dfs_counter;
                        dfs_counter += 1;
                        scc_stack.push(u);
                        on_stack[u] = true;

                        // Push finish action first (will be processed after children)
                        work.push(Action::Finish(u));

                        // Push children in reverse order for correct DFS ordering
                        for &v in out_edges[u].iter().rev() {
                            if dfs_num[v] < 0 {
                                work.push(Action::Visit(v));
                            }
                        }
                    }
                    Action::Finish(u) => {
                        // Update low-link from children
                        for &v in &out_edges[u] {
                            if scc_id[v] < 0 {
                                // v is still on stack or not yet visited
                                if on_stack[v] {
                                    dfs_low[u] = dfs_low[u].min(dfs_low[v]);
                                }
                            }
                        }

                        if dfs_low[u] == dfs_num[u] {
                            // Root of an SCC
                            let scc = num_sccs;
                            num_sccs += 1;
                            while let Some(w) = scc_stack.pop() {
                                on_stack[w] = false;
                                scc_id[w] = scc as i32;
                                if w == u {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Assign any unvisited nodes to their own SCC
    for item in scc_id.iter_mut().take(n) {
        if *item < 0 {
            *item = num_sccs as i32;
            num_sccs += 1;
        }
    }

    // Build DAG of SCCs
    let mut scc_out: Vec<HashSet<usize>> = vec![HashSet::new(); num_sccs];
    let mut scc_in_degree: Vec<usize> = vec![0; num_sccs];
    let mut scc_has_edge: Vec<bool> = vec![false; num_sccs];

    for u in 0..n {
        let su = scc_id[u] as usize;
        for &v in &out_edges[u] {
            let sv = scc_id[v] as usize;
            if su != sv && scc_out[su].insert(sv) {
                scc_in_degree[sv] += 1;
                scc_has_edge[su] = true;
                scc_has_edge[sv] = true;
            }
        }
        if has_edge[u] {
            scc_has_edge[su] = true;
        }
    }

    // Topological sort + longest path for SCC DAG
    let mut scc_rank: Vec<i32> = vec![-1; num_sccs];
    let mut queue = VecDeque::new();

    for s in 0..num_sccs {
        if scc_has_edge[s] && scc_in_degree[s] == 0 {
            scc_rank[s] = 0;
            queue.push_back(s);
        }
    }

    while let Some(su) = queue.pop_front() {
        for &sv in &scc_out[su] {
            let new_rank = scc_rank[su] + 1;
            if new_rank > scc_rank[sv] {
                scc_rank[sv] = new_rank;
            }
            scc_in_degree[sv] -= 1;
            if scc_in_degree[sv] == 0 {
                queue.push_back(sv);
            }
        }
    }

    // Map SCC ranks back to state ranks
    let mut rank: Vec<i32> = vec![-1; n];
    for i in 0..n {
        let si = scc_id[i] as usize;
        rank[i] = scc_rank[si];
    }

    // Check if ANY states have edges in this scope
    let any_edges = has_edge.iter().any(|&e| e);

    if !any_edges {
        // No edges at all: fall back to vertical stacking (one state per rank)
        for (i, r) in rank.iter_mut().enumerate().take(n) {
            *r = i as i32;
        }
    } else {
        // States without edges: place at the same rank as nearest connected state
        // in declaration order, or rank 0 if none.
        let mut last_connected_rank = 0;
        for i in 0..n {
            if !has_edge[i] {
                rank[i] = last_connected_rank;
            } else if rank[i] >= 0 {
                last_connected_rank = rank[i];
            }
        }

        // Ensure all unranked nodes are at rank 0
        for r in &mut rank {
            if *r < 0 {
                *r = 0;
            }
        }
    }

    // Build rank -> [state_indices]
    let max_rank = rank.iter().copied().max().unwrap_or(0);
    let mut ranks: Vec<Vec<usize>> = vec![Vec::new(); (max_rank + 1) as usize];
    for i in 0..n {
        ranks[rank[i] as usize].push(i);
    }

    // Remove empty ranks
    ranks.retain(|r| !r.is_empty());

    ranks
}

/// Layout a list of states using rank-based placement.
///
/// States connected by transitions are placed on successive rows.
/// States on the same rank are placed side-by-side horizontally.
///
/// Returns `(laid_out_nodes, content_width, content_height)`.
fn layout_states_ranked(
    states: &[State],
    transitions: &[Transition],
    initial_ids: &HashSet<String>,
    final_ids: &HashSet<String>,
    start_x: f64,
    start_y: f64,
) -> (Vec<StateNodeLayout>, f64, f64) {
    if states.is_empty() {
        return (Vec::new(), 0.0, 0.0);
    }

    // First pass: compute sizes for all states
    let mut sized_entries: Vec<(StateNodeLayout, f64, f64)> = Vec::new();
    for state in states {
        sized_entries.push(compute_state_node(state, transitions, initial_ids, final_ids));
    }

    let state_ids: Vec<String> = states.iter().map(|s| s.id.clone()).collect();

    // Assign ranks based on transition connectivity
    let ranks = assign_ranks(&state_ids, transitions, initial_ids, final_ids);

    // Place states row by row
    let mut y_cursor = start_y;
    let mut total_width = 0.0_f64;
    let mut positioned: Vec<Option<(f64, f64)>> = vec![None; states.len()];

    for rank_indices in &ranks {
        // Get the entries in this rank
        let row_entries: Vec<usize> = rank_indices.to_vec();

        if row_entries.is_empty() {
            continue;
        }

        // Compute row dimensions
        let row_height = row_entries
            .iter()
            .map(|&i| sized_entries[i].2)
            .fold(0.0_f64, f64::max);
        let row_width: f64 = row_entries
            .iter()
            .map(|&i| sized_entries[i].1)
            .sum::<f64>()
            + STATE_SPACING * (row_entries.len() as f64 - 1.0).max(0.0);

        total_width = total_width.max(row_width);

        // Place each state in the row
        let mut x_cursor = start_x;
        for &idx in &row_entries {
            let (_, w, h) = &sized_entries[idx];
            // Vertically center within the row
            let y_offset = (row_height - h) / 2.0;
            positioned[idx] = Some((x_cursor, y_cursor + y_offset));
            x_cursor += w + STATE_SPACING;
        }

        y_cursor += row_height + STATE_SPACING;
    }

    // Remove trailing spacing
    let total_height = if ranks.is_empty() {
        0.0
    } else {
        y_cursor - start_y - STATE_SPACING
    };

    // Center each row within the total width
    for rank_indices in &ranks {
        let row_width: f64 = rank_indices
            .iter()
            .map(|&i| sized_entries[i].1)
            .sum::<f64>()
            + STATE_SPACING * (rank_indices.len() as f64 - 1.0).max(0.0);
        let offset = (total_width - row_width) / 2.0;
        if offset > 0.5 {
            for &idx in rank_indices {
                if let Some((ref mut x, _)) = positioned[idx] {
                    *x += offset;
                }
            }
        }
    }

    // Build final node list
    let mut nodes = Vec::new();
    for (idx, (mut node, _w, _h)) in sized_entries.into_iter().enumerate() {
        if let Some((x, y)) = positioned[idx] {
            node.x = x;
            node.y = y;

            // Offset children to absolute positions within the composite
            if node.is_composite {
                let child_offset_x = x + COMPOSITE_PADDING;
                let child_offset_y = y + COMPOSITE_HEADER;
                offset_children(&mut node.children, child_offset_x, child_offset_y);
                for sep_y in &mut node.region_separators {
                    *sep_y += child_offset_y;
                }
            }

            log::debug!(
                "  state '{}' @ ({:.1}, {:.1}) {}x{} composite={} initial={} final={}",
                node.id,
                node.x,
                node.y,
                node.width,
                node.height,
                node.is_composite,
                node.is_initial,
                node.is_final
            );

            nodes.push(node);
        }
    }

    (nodes, total_width, total_height)
}

/// Recursively offset children's positions from relative (0,0) to absolute.
fn offset_children(children: &mut [StateNodeLayout], offset_x: f64, offset_y: f64) {
    for child in children.iter_mut() {
        child.x += offset_x;
        child.y += offset_y;
        if child.is_composite {
            // Children of children are already relative to the child; the
            // recursive layout already set them.  But since we just moved the
            // parent, the children's absolute coords from the recursive call
            // were relative to (0,0), so we need to offset them too.
            offset_children(&mut child.children, offset_x, offset_y);
        }
    }
}

// ---------------------------------------------------------------------------
// Transition routing
// ---------------------------------------------------------------------------

/// Build a lookup from state ID to its center position.
fn build_position_map(nodes: &[StateNodeLayout]) -> HashMap<String, (f64, f64, f64, f64)> {
    let mut map = HashMap::new();
    collect_positions(nodes, &mut map);
    map
}

/// Recursively collect (x, y, w, h) for every state node.
fn collect_positions(nodes: &[StateNodeLayout], map: &mut HashMap<String, (f64, f64, f64, f64)>) {
    for node in nodes {
        map.insert(node.id.clone(), (node.x, node.y, node.width, node.height));
        collect_positions(&node.children, map);
    }
}

/// Create transition layouts by connecting state positions.
fn layout_transitions(
    transitions: &[Transition],
    pos_map: &HashMap<String, (f64, f64, f64, f64)>,
) -> Vec<TransitionLayout> {
    let mut result = Vec::new();

    for tr in transitions {
        let from_pos = pos_map.get(&tr.from);
        let to_pos = pos_map.get(&tr.to);

        let (from_x, from_y, from_w, from_h) = if let Some(p) = from_pos {
            *p
        } else {
            log::warn!("transition source '{}' not found in layout", tr.from);
            continue;
        };

        let (to_x, to_y, to_w, to_h) = if let Some(p) = to_pos {
            *p
        } else {
            log::warn!("transition target '{}' not found in layout", tr.to);
            continue;
        };

        // Determine connection direction based on relative positions
        let from_cx = from_x + from_w / 2.0;
        let from_cy = from_y + from_h / 2.0;
        let to_cx = to_x + to_w / 2.0;
        let to_cy = to_y + to_h / 2.0;

        let points = if (from_cy - to_cy).abs() < 1.0 {
            // Horizontal: connect right-center to left-center
            if from_cx < to_cx {
                vec![(from_x + from_w, from_cy), (to_x, to_cy)]
            } else {
                vec![(from_x, from_cy), (to_x + to_w, to_cy)]
            }
        } else if to_cy > from_cy {
            // Target is below: bottom-center to top-center
            vec![(from_cx, from_y + from_h), (to_cx, to_y)]
        } else {
            // Target is above: top-center to bottom-center
            vec![(from_cx, from_y), (to_cx, to_y + to_h)]
        };

        log::debug!(
            "  transition '{}' -> '{}' [{}]: {:?}",
            tr.from,
            tr.to,
            tr.label,
            points
        );

        result.push(TransitionLayout {
            from_id: tr.from.clone(),
            to_id: tr.to.clone(),
            label: tr.label.clone(),
            points,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Perform the complete layout of a state diagram.
///
/// The result contains absolute positions for every state node, transition edge,
/// and note so that a renderer can draw them without further computation.
pub fn layout_state(diagram: &StateDiagram) -> Result<StateLayout> {
    log::debug!(
        "layout_state: {} states, {} transitions, {} notes",
        diagram.states.len(),
        diagram.transitions.len(),
        diagram.notes.len()
    );

    // Classify [*] states as initial or final
    let (initial_ids, final_ids) = classify_special_states(&diagram.states, &diagram.transitions);

    log::debug!("  initial_ids: {initial_ids:?}, final_ids: {final_ids:?}");

    // Collect implicit states (referenced in transitions but not declared)
    let implicit_states = collect_implicit_states(&diagram.states, &diagram.transitions);
    log::debug!("  implicit states: {}", implicit_states.len());

    // Merge declared + implicit states, deduplicating by ID.
    // When duplicate IDs exist, prefer composite states over simple ones.
    let mut all_states: Vec<State> = diagram.states.clone();
    all_states.extend(implicit_states);
    dedup_states(&mut all_states);

    // Re-classify after adding implicit states
    let (initial_ids, final_ids) = classify_special_states(&all_states, &diagram.transitions);

    // Layout states with rank-based placement
    let (state_layouts, content_width, content_height) = layout_states_ranked(
        &all_states,
        &diagram.transitions,
        &initial_ids,
        &final_ids,
        MARGIN,
        MARGIN,
    );

    // Build position map for transition routing
    let pos_map = build_position_map(&state_layouts);

    // Layout transitions
    let transition_layouts = layout_transitions(&diagram.transitions, &pos_map);

    // Layout notes (placed to the right of the diagram body)
    let note_x = MARGIN + content_width + NOTE_OFFSET;
    let mut note_y = MARGIN;
    let mut note_layouts = Vec::new();

    for note in &diagram.notes {
        let (nw, nh) = estimate_note_size(&note.text);
        log::debug!(
            "  note @ ({:.1}, {:.1}) {}x{}: '{}'",
            note_x,
            note_y,
            nw,
            nh,
            note.text
        );
        note_layouts.push(StateNoteLayout {
            x: note_x,
            y: note_y,
            width: nw,
            height: nh,
            text: note.text.clone(),
        });
        note_y += nh + PADDING;
    }

    // Compute total bounding box
    let notes_right = if note_layouts.is_empty() {
        0.0
    } else {
        note_layouts
            .iter()
            .map(|n| n.x + n.width)
            .fold(0.0_f64, f64::max)
    };
    let states_right = MARGIN + content_width;
    let total_width = states_right.max(notes_right) + MARGIN;
    let total_width = total_width.max(2.0 * MARGIN);

    let notes_bottom = if note_layouts.is_empty() {
        0.0
    } else {
        note_layouts
            .iter()
            .map(|n| n.y + n.height)
            .fold(0.0_f64, f64::max)
    };
    let states_bottom = MARGIN + content_height;
    let total_height = states_bottom.max(notes_bottom) + MARGIN;
    let total_height = total_height.max(2.0 * MARGIN);

    log::debug!(
        "layout_state done: {:.0}x{:.0}, {} states, {} transitions, {} notes",
        total_width,
        total_height,
        state_layouts.len(),
        transition_layouts.len(),
        note_layouts.len()
    );

    let mut layout = StateLayout {
        width: total_width,
        height: total_height,
        state_layouts,
        transition_layouts,
        note_layouts,
    };
    apply_direction_transform(&mut layout, &diagram.direction);

    Ok(layout)
}

// ---------------------------------------------------------------------------
// Direction transform
// ---------------------------------------------------------------------------

/// Apply a coordinate transform based on the diagram direction.
/// The layout algorithm always computes in top-to-bottom orientation;
/// for other directions we transform after the fact.
fn apply_direction_transform(
    layout: &mut StateLayout,
    direction: &crate::model::diagram::Direction,
) {
    use crate::model::diagram::Direction;
    match direction {
        Direction::TopToBottom => {}
        Direction::LeftToRight => {
            transform_state_nodes_swap_xy(&mut layout.state_layouts);
            for tr in &mut layout.transition_layouts {
                for pt in &mut tr.points {
                    std::mem::swap(&mut pt.0, &mut pt.1);
                }
            }
            for note in &mut layout.note_layouts {
                std::mem::swap(&mut note.x, &mut note.y);
                std::mem::swap(&mut note.width, &mut note.height);
            }
            std::mem::swap(&mut layout.width, &mut layout.height);
        }
        Direction::RightToLeft => {
            transform_state_nodes_swap_xy(&mut layout.state_layouts);
            for tr in &mut layout.transition_layouts {
                for pt in &mut tr.points {
                    std::mem::swap(&mut pt.0, &mut pt.1);
                }
            }
            for note in &mut layout.note_layouts {
                std::mem::swap(&mut note.x, &mut note.y);
                std::mem::swap(&mut note.width, &mut note.height);
            }
            std::mem::swap(&mut layout.width, &mut layout.height);
            let w = layout.width;
            transform_state_nodes_mirror_x(&mut layout.state_layouts, w);
            for tr in &mut layout.transition_layouts {
                for pt in &mut tr.points {
                    pt.0 = w - pt.0;
                }
            }
            for note in &mut layout.note_layouts {
                note.x = w - note.x - note.width;
            }
        }
        Direction::BottomToTop => {
            let h = layout.height;
            transform_state_nodes_mirror_y(&mut layout.state_layouts, h);
            for tr in &mut layout.transition_layouts {
                for pt in &mut tr.points {
                    pt.1 = h - pt.1;
                }
            }
            for note in &mut layout.note_layouts {
                note.y = h - note.y - note.height;
            }
        }
    }
}

/// Recursively swap x <-> y for state nodes and their children.
fn transform_state_nodes_swap_xy(nodes: &mut [StateNodeLayout]) {
    for node in nodes.iter_mut() {
        std::mem::swap(&mut node.x, &mut node.y);
        std::mem::swap(&mut node.width, &mut node.height);
        transform_state_nodes_swap_xy(&mut node.children);
    }
}

/// Recursively mirror state nodes horizontally.
fn transform_state_nodes_mirror_x(nodes: &mut [StateNodeLayout], total_width: f64) {
    for node in nodes.iter_mut() {
        node.x = total_width - node.x - node.width;
        transform_state_nodes_mirror_x(&mut node.children, total_width);
    }
}

/// Recursively mirror state nodes vertically.
fn transform_state_nodes_mirror_y(nodes: &mut [StateNodeLayout], total_height: f64) {
    for node in nodes.iter_mut() {
        node.y = total_height - node.y - node.height;
        transform_state_nodes_mirror_y(&mut node.children, total_height);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::state::{State, StateDiagram, StateNote, Transition};

    fn empty_diagram() -> StateDiagram {
        StateDiagram {
            states: vec![],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        }
    }

    fn simple_state(name: &str) -> State {
        State {
            name: name.to_string(),
            id: name.to_string(),
            description: vec![],
            stereotype: None,
            children: vec![],
            is_special: false,
            kind: crate::model::state::StateKind::default(),
            regions: vec![],
        }
    }

    fn special_state(id: &str) -> State {
        State {
            name: "[*]".to_string(),
            id: id.to_string(),
            description: vec![],
            stereotype: None,
            children: vec![],
            is_special: true,
            kind: crate::model::state::StateKind::default(),
            regions: vec![],
        }
    }

    fn transition(from: &str, to: &str, label: &str) -> Transition {
        Transition {
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
            dashed: false,
        }
    }

    // 1. Empty diagram
    #[test]
    fn test_empty_diagram() {
        let d = empty_diagram();
        let layout = layout_state(&d).unwrap();
        assert!(layout.state_layouts.is_empty());
        assert!(layout.transition_layouts.is_empty());
        assert!(layout.note_layouts.is_empty());
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 2. Single state
    #[test]
    fn test_single_state() {
        let d = StateDiagram {
            states: vec![simple_state("Active")],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        assert_eq!(layout.state_layouts.len(), 1);
        let node = &layout.state_layouts[0];
        assert_eq!(node.id, "Active");
        assert_eq!(node.name, "Active");
        assert!(node.width >= STATE_MIN_WIDTH);
        assert!(node.height >= STATE_MIN_HEIGHT);
        assert!(!node.is_initial);
        assert!(!node.is_final);
        assert!(!node.is_composite);
    }

    // 3. Initial [*] state
    #[test]
    fn test_initial_state() {
        let d = StateDiagram {
            states: vec![special_state("[*]"), simple_state("Active")],
            transitions: vec![transition("[*]", "Active", "")],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        assert_eq!(layout.state_layouts.len(), 2);

        let initial = &layout.state_layouts[0];
        assert!(initial.is_initial);
        assert!(!initial.is_final);
        assert_eq!(initial.width, 2.0 * SPECIAL_STATE_RADIUS);
        assert_eq!(initial.height, 2.0 * SPECIAL_STATE_RADIUS);
    }

    // 4. Final [*] state
    #[test]
    fn test_final_state() {
        let d = StateDiagram {
            states: vec![simple_state("Active"), special_state("[*]_final")],
            transitions: vec![transition("Active", "[*]_final", "")],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();

        let final_node = layout
            .state_layouts
            .iter()
            .find(|n| n.id == "[*]_final")
            .unwrap();
        assert!(final_node.is_final);
        assert!(!final_node.is_initial);
    }

    // 5. Start and stop states (scxml0001 style)
    #[test]
    fn test_start_stop_with_transitions() {
        let d = StateDiagram {
            states: vec![
                special_state("[*]_start"),
                simple_state("Active"),
                simple_state("Inactive"),
                special_state("[*]_end"),
            ],
            transitions: vec![
                transition("[*]_start", "Active", ""),
                transition("Active", "Inactive", "deactivate"),
                transition("Inactive", "[*]_end", ""),
            ],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();

        assert_eq!(layout.state_layouts.len(), 4);
        assert_eq!(layout.transition_layouts.len(), 3);

        // Start should be above Active
        let start = layout
            .state_layouts
            .iter()
            .find(|n| n.id == "[*]_start")
            .unwrap();
        let active = layout
            .state_layouts
            .iter()
            .find(|n| n.id == "Active")
            .unwrap();
        assert!(start.y < active.y);

        // Transitions should have points
        for tl in &layout.transition_layouts {
            assert!(!tl.points.is_empty());
        }
    }

    // 6. Composite state
    #[test]
    fn test_composite_state() {
        let d = StateDiagram {
            states: vec![State {
                name: "Container".to_string(),
                id: "Container".to_string(),
                description: vec![],
                stereotype: None,
                children: vec![simple_state("Inner1"), simple_state("Inner2")],
                is_special: false,
                kind: crate::model::state::StateKind::default(),
                regions: vec![],
            }],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        assert_eq!(layout.state_layouts.len(), 1);

        let container = &layout.state_layouts[0];
        assert!(container.is_composite);
        assert_eq!(container.children.len(), 2);

        // Children should be inside the container
        for child in &container.children {
            assert!(child.x >= container.x);
            assert!(child.y >= container.y + COMPOSITE_HEADER);
            assert!(child.x + child.width <= container.x + container.width + 1.0);
        }
    }

    // 7. Nested composite (deeply nested)
    #[test]
    fn test_nested_composite() {
        let inner_composite = State {
            name: "Middle".to_string(),
            id: "Middle".to_string(),
            description: vec![],
            stereotype: None,
            children: vec![simple_state("Deep1"), simple_state("Deep2")],
            is_special: false,
            kind: crate::model::state::StateKind::default(),
            regions: vec![],
        };

        let d = StateDiagram {
            states: vec![State {
                name: "Outer".to_string(),
                id: "Outer".to_string(),
                description: vec![],
                stereotype: None,
                children: vec![inner_composite, simple_state("Sibling")],
                is_special: false,
                kind: crate::model::state::StateKind::default(),
                regions: vec![],
            }],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();

        let outer = &layout.state_layouts[0];
        assert!(outer.is_composite);
        assert_eq!(outer.children.len(), 2);

        let middle = &outer.children[0];
        assert!(middle.is_composite);
        assert_eq!(middle.children.len(), 2);

        // Deep children should have absolute positions inside outer
        for deep in &middle.children {
            assert!(
                deep.x >= outer.x,
                "deep child x={} should be >= outer x={}",
                deep.x,
                outer.x
            );
            assert!(
                deep.y >= outer.y,
                "deep child y={} should be >= outer y={}",
                deep.y,
                outer.y
            );
        }
    }

    // 8. Transitions connect correct positions
    #[test]
    fn test_transition_points() {
        let d = StateDiagram {
            states: vec![simple_state("A"), simple_state("B")],
            transitions: vec![transition("A", "B", "go")],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        assert_eq!(layout.transition_layouts.len(), 1);

        let tl = &layout.transition_layouts[0];
        assert_eq!(tl.from_id, "A");
        assert_eq!(tl.to_id, "B");
        assert_eq!(tl.label, "go");
        assert_eq!(tl.points.len(), 2);

        // Source point should be above target point (vertical layout)
        let (_, from_y) = tl.points[0];
        let (_, to_y) = tl.points[1];
        assert!(from_y < to_y, "from_y={} should be < to_y={}", from_y, to_y);
    }

    // 9. Notes layout
    #[test]
    fn test_notes() {
        let d = StateDiagram {
            states: vec![simple_state("A")],
            transitions: vec![],
            notes: vec![
                StateNote {
                    alias: None,
                    text: "first note".to_string(),
                },
                StateNote {
                    alias: Some("n1".to_string()),
                    text: "second note\nwith two lines".to_string(),
                },
            ],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        assert_eq!(layout.note_layouts.len(), 2);

        let n0 = &layout.note_layouts[0];
        let n1 = &layout.note_layouts[1];
        assert_eq!(n0.text, "first note");
        assert_eq!(n1.text, "second note\nwith two lines");

        // Notes should be to the right of the state
        let state_right = layout.state_layouts[0].x + layout.state_layouts[0].width;
        assert!(
            n0.x > state_right,
            "note x={} should be > state right={}",
            n0.x,
            state_right
        );

        // Second note should be below the first
        assert!(n1.y > n0.y);
    }

    // 10. Text sizing for states with descriptions
    #[test]
    fn test_description_state_sizing() {
        let d = StateDiagram {
            states: vec![State {
                name: "Described".to_string(),
                id: "Described".to_string(),
                description: vec![
                    "line one".to_string(),
                    "a much longer description line".to_string(),
                    "line three".to_string(),
                ],
                stereotype: None,
                children: vec![],
                is_special: false,
                kind: crate::model::state::StateKind::default(),
                regions: vec![],
            }],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        let node = &layout.state_layouts[0];

        // Width should accommodate the longest description line
        let expected_min_w =
            "a much longer description line".len() as f64 * CHAR_WIDTH + 2.0 * PADDING;
        assert!(
            node.width >= expected_min_w,
            "width {} should be >= {}",
            node.width,
            expected_min_w
        );

        // Height should accommodate name (14pt) + 3 description lines (12pt)
        let name_h = crate::font_metrics::line_height("SansSerif", STATE_NAME_FONT_SIZE, false, false);
        let desc_h = crate::font_metrics::line_height("SansSerif", STATE_DESC_FONT_SIZE, false, false);
        let expected_min_h = name_h + 3.0 * desc_h + 2.0 * PADDING;
        assert!(
            node.height >= expected_min_h,
            "height {} should be >= {}",
            node.height,
            expected_min_h
        );

        // Descriptions should be preserved
        assert_eq!(node.description.len(), 3);
    }

    // 11. Implicit states (referenced but not declared)
    #[test]
    fn test_implicit_states() {
        let d = StateDiagram {
            states: vec![simple_state("A")],
            transitions: vec![transition("A", "B", "go")],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();

        // "B" is implicit — it should still appear in layouts
        assert_eq!(layout.state_layouts.len(), 2);
        let b = layout.state_layouts.iter().find(|n| n.id == "B");
        assert!(b.is_some(), "implicit state B should be in layout");
    }

    // 12. State with stereotype
    #[test]
    fn test_state_with_stereotype() {
        let d = StateDiagram {
            states: vec![State {
                name: "MyState".to_string(),
                id: "MyState".to_string(),
                description: vec![],
                stereotype: Some("<<inputPin>>".to_string()),
                children: vec![],
                is_special: false,
                kind: crate::model::state::StateKind::default(),
                regions: vec![],
            }],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        let node = &layout.state_layouts[0];
        assert_eq!(node.stereotype.as_deref(), Some("<<inputPin>>"));

        // Height should be taller than a state without stereotype
        let plain = StateDiagram {
            states: vec![simple_state("MyState")],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let plain_layout = layout_state(&plain).unwrap();
        assert!(
            node.height > plain_layout.state_layouts[0].height,
            "stereotype state should be taller"
        );
    }

    // 13. Multiple states ordered vertically
    #[test]
    fn test_vertical_ordering() {
        let d = StateDiagram {
            states: vec![
                simple_state("First"),
                simple_state("Second"),
                simple_state("Third"),
            ],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        assert_eq!(layout.state_layouts.len(), 3);

        let y0 = layout.state_layouts[0].y;
        let y1 = layout.state_layouts[1].y;
        let y2 = layout.state_layouts[2].y;

        assert!(y0 < y1, "First ({}) should be above Second ({})", y0, y1);
        assert!(y1 < y2, "Second ({}) should be above Third ({})", y1, y2);
    }

    // 14. Empty composite state
    #[test]
    fn test_empty_composite() {
        let d = StateDiagram {
            states: vec![State {
                name: "Empty".to_string(),
                id: "Empty".to_string(),
                description: vec![],
                stereotype: None,
                children: vec![], // technically not composite since children is empty
                is_special: false,
                kind: crate::model::state::StateKind::default(),
                regions: vec![],
            }],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        assert_eq!(layout.state_layouts.len(), 1);
        assert!(!layout.state_layouts[0].is_composite);
    }

    // 15. Bounding box includes all elements
    #[test]
    fn test_bounding_box() {
        let d = StateDiagram {
            states: vec![simple_state("A"), simple_state("B")],
            transitions: vec![transition("A", "B", "")],
            notes: vec![StateNote {
                alias: None,
                text: "a note".to_string(),
            }],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();

        // All state nodes should be within bounds
        for node in &layout.state_layouts {
            assert!(
                node.x + node.width <= layout.width,
                "state right edge {} exceeds width {}",
                node.x + node.width,
                layout.width
            );
            assert!(
                node.y + node.height <= layout.height,
                "state bottom edge {} exceeds height {}",
                node.y + node.height,
                layout.height
            );
        }

        // Notes should be within bounds
        for note in &layout.note_layouts {
            assert!(
                note.x + note.width <= layout.width,
                "note right edge {} exceeds width {}",
                note.x + note.width,
                layout.width
            );
        }
    }

    // 16. Special state defaults to initial when no transitions
    #[test]
    fn test_special_state_default_initial() {
        let d = StateDiagram {
            states: vec![special_state("[*]")],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_state(&d).unwrap();
        let node = &layout.state_layouts[0];
        assert!(node.is_initial, "unconnected [*] should default to initial");
    }

    // 17. LeftToRight direction
    #[test]
    fn test_left_to_right_direction() {
        use crate::model::diagram::Direction;
        let d = StateDiagram {
            states: vec![
                simple_state("First"),
                simple_state("Second"),
                simple_state("Third"),
            ],
            transitions: vec![
                transition("First", "Second", ""),
                transition("Second", "Third", ""),
            ],
            notes: vec![],
            direction: Direction::LeftToRight,
        };
        let layout = layout_state(&d).unwrap();

        // With LR direction, width should be > height
        assert!(
            layout.width > layout.height,
            "LR: width ({:.1}) should be > height ({:.1})",
            layout.width,
            layout.height
        );

        // Nodes should flow left-to-right: x positions should increase
        let x0 = layout.state_layouts[0].x;
        let x1 = layout.state_layouts[1].x;
        let x2 = layout.state_layouts[2].x;
        assert!(x0 < x1, "LR: First x ({:.1}) < Second x ({:.1})", x0, x1);
        assert!(x1 < x2, "LR: Second x ({:.1}) < Third x ({:.1})", x1, x2);
    }

    // 18. TB direction: height > width
    #[test]
    fn test_top_to_bottom_direction() {
        use crate::model::diagram::Direction;
        let d = StateDiagram {
            states: vec![
                simple_state("First"),
                simple_state("Second"),
                simple_state("Third"),
            ],
            transitions: vec![],
            notes: vec![],
            direction: Direction::TopToBottom,
        };
        let layout = layout_state(&d).unwrap();

        // With TB direction, height should be > width
        assert!(
            layout.height > layout.width,
            "TB: height ({:.1}) should be > width ({:.1})",
            layout.height,
            layout.width
        );
    }

    // 19. BottomToTop direction: first state at bottom
    #[test]
    fn test_bottom_to_top_direction() {
        use crate::model::diagram::Direction;
        let d = StateDiagram {
            states: vec![simple_state("First"), simple_state("Second")],
            transitions: vec![],
            notes: vec![],
            direction: Direction::BottomToTop,
        };
        let layout = layout_state(&d).unwrap();

        // First state should be below Second in BT direction
        let y0 = layout.state_layouts[0].y;
        let y1 = layout.state_layouts[1].y;
        assert!(
            y0 > y1,
            "BT: First y ({:.1}) should be > Second y ({:.1})",
            y0,
            y1
        );
    }
}
