//! State diagram layout engine.
//!
//! Converts a `StateDiagram` into a fully positioned `StateLayout` ready for
//! SVG rendering.  The algorithm uses a top-to-bottom vertical placement
//! strategy similar to the activity diagram layout, with recursive handling
//! of composite (nested) states.

use std::collections::{HashMap, HashSet};

use crate::font_metrics;
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

const FONT_SIZE: f64 = 14.0;
const LINE_HEIGHT: f64 = 16.0;
const PADDING: f64 = 10.0;
const STATE_MIN_WIDTH: f64 = 80.0;
const STATE_MIN_HEIGHT: f64 = 40.0;
const STATE_SPACING: f64 = 40.0;
const SPECIAL_STATE_RADIUS: f64 = 10.0;
const COMPOSITE_PADDING: f64 = 20.0;
const COMPOSITE_HEADER: f64 = 30.0;
const NOTE_OFFSET: f64 = 30.0;
const FORK_BAR_WIDTH: f64 = 80.0;
const FORK_BAR_HEIGHT: f64 = 6.0;
const CHOICE_SIZE: f64 = 20.0;
const HISTORY_DIAMETER: f64 = 24.0;
const NOTE_MAX_WIDTH: f64 = 200.0;
const MARGIN: f64 = 20.0;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

/// Estimate the pixel width of a single line of text.
fn text_width(text: &str) -> f64 {
    font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false)
}

/// Estimate the size of a simple (non-composite, non-special) state.
/// Returns `(width, height)`.
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

    // Header line (name) + optional stereotype line + description lines
    let stereo_lines = if state.stereotype.is_some() { 1.0 } else { 0.0 };
    let desc_lines = state.description.len() as f64;
    let total_lines = 1.0 + stereo_lines + desc_lines;
    let height = (total_lines * LINE_HEIGHT + 2.0 * PADDING).max(STATE_MIN_HEIGHT);

    (width, height)
}

/// Estimate the size of a note, clamped to `NOTE_MAX_WIDTH`.
fn estimate_note_size(text: &str) -> (f64, f64) {
    let lines: Vec<&str> = text.lines().collect();
    let max_line_width = lines
        .iter()
        .map(|l| font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    let width = (max_line_width + 2.0 * PADDING).min(NOTE_MAX_WIDTH);
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

/// Recursively collect all declared state IDs.
fn collect_declared_ids(states: &[State], ids: &mut HashSet<String>) {
    for s in states {
        ids.insert(s.id.clone());
        collect_declared_ids(&s.children, ids);
    }
}

// ---------------------------------------------------------------------------
// Core layout logic
// ---------------------------------------------------------------------------

/// Layout a list of states vertically, starting at `(start_x, start_y)`.
/// The `center_x` parameter controls horizontal centering.
///
/// Returns `(laid_out_nodes, content_width, content_height)`.
#[allow(clippy::only_used_in_recursion)]
fn layout_states_vertical(
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

    // First pass: compute sizes and children for all states
    let mut entries: Vec<(StateNodeLayout, f64, f64)> = Vec::new();

    for state in states {
        let is_initial = initial_ids.contains(&state.id);
        let is_final = final_ids.contains(&state.id);
        let is_composite = !state.children.is_empty() || !state.regions.is_empty();

        if state.is_special {
            // Special [*] state: small circle
            let diameter = 2.0 * SPECIAL_STATE_RADIUS;
            entries.push((
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
            ));
        } else if matches!(state.kind, StateKind::Fork | StateKind::Join) {
            // Fork/Join: thin horizontal bar
            let w = FORK_BAR_WIDTH;
            let h = FORK_BAR_HEIGHT;
            entries.push((
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
            ));
        } else if state.kind == StateKind::Choice {
            // Choice: small diamond
            let s = CHOICE_SIZE;
            entries.push((
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
            ));
        } else if matches!(state.kind, StateKind::History | StateKind::DeepHistory) {
            // History: small circle with H text
            let d = HISTORY_DIAMETER;
            entries.push((
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
            ));
        } else if state.kind == StateKind::End {
            // End pseudo-state: renders like final
            let diameter = 2.0 * SPECIAL_STATE_RADIUS;
            entries.push((
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
            ));
        } else if is_composite {
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
                    let (child_layouts, child_w, child_h) = layout_states_vertical(
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
                let (child_layouts, child_w, child_h) = layout_states_vertical(
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

            entries.push((
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
            ));
        } else {
            // Simple state
            let (w, h) = estimate_state_size(state);
            entries.push((
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
            ));
        }
    }

    // Compute the maximum width across all states
    let max_width = entries.iter().map(|(_, w, _)| *w).fold(0.0_f64, f64::max);

    // Second pass: assign absolute positions (centered horizontally)
    let mut y_cursor = start_y;
    let mut nodes = Vec::new();

    for (mut node, w, h) in entries {
        let x = start_x + (max_width - w) / 2.0;
        node.x = x;
        node.y = y_cursor;

        // Offset children to absolute positions within the composite
        if node.is_composite {
            let child_offset_x = x + COMPOSITE_PADDING;
            let child_offset_y = y_cursor + COMPOSITE_HEADER;
            offset_children(&mut node.children, child_offset_x, child_offset_y);
            // Offset region separators to absolute Y positions
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

        y_cursor += h + STATE_SPACING;
        nodes.push(node);
    }

    let total_height = if states.is_empty() {
        0.0
    } else {
        y_cursor - start_y - STATE_SPACING // subtract trailing spacing
    };

    (nodes, max_width, total_height)
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

    // Merge declared + implicit states
    let mut all_states: Vec<State> = diagram.states.clone();
    all_states.extend(implicit_states);

    // Re-classify after adding implicit states
    let (initial_ids, final_ids) = classify_special_states(&all_states, &diagram.transitions);

    // Layout states vertically
    let (state_layouts, content_width, content_height) = layout_states_vertical(
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
            crate::font_metrics::text_width("a much longer description line", "SansSerif", FONT_SIZE, false, false) + 2.0 * PADDING;
        assert!(
            node.width >= expected_min_w,
            "width {} should be >= {}",
            node.width,
            expected_min_w
        );

        // Height should accommodate name + 3 description lines
        let expected_min_h = 4.0 * LINE_HEIGHT + 2.0 * PADDING;
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
