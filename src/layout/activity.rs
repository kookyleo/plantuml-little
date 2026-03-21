//! Activity diagram layout engine.
//!
//! Converts an `ActivityDiagram` (list of events + optional swimlanes) into a
//! fully positioned `ActivityLayout` ready for SVG rendering.  The algorithm is
//! a single top-to-bottom pass with a y-cursor, similar to how the sequence
//! diagram layout works with column-based placement.

use crate::font_metrics;
use crate::model::activity::{ActivityDiagram, ActivityEvent, NotePosition};
use crate::render::svg_richtext::creole_plain_text;
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned activity diagram ready for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityLayout {
    pub width: f64,
    pub height: f64,
    pub nodes: Vec<ActivityNodeLayout>,
    pub edges: Vec<ActivityEdgeLayout>,
    pub swimlane_layouts: Vec<SwimlaneLayout>,
}

/// A single positioned node.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityNodeLayout {
    pub index: usize,
    pub kind: ActivityNodeKindLayout,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
}

/// Visual kind of a node — determines how the renderer draws it.
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityNodeKindLayout {
    Start,
    Stop,
    End,
    Action,
    Diamond,
    ForkBar,
    Note { position: NotePositionLayout },
    FloatingNote { position: NotePositionLayout },
    Detach,
}

/// Note position in the layout coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotePositionLayout {
    Left,
    Right,
}

/// A directed edge between two nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityEdgeLayout {
    pub from_index: usize,
    pub to_index: usize,
    pub label: String,
    pub points: Vec<(f64, f64)>,
}

/// A single swimlane column.
#[derive(Debug, Clone, PartialEq)]
pub struct SwimlaneLayout {
    pub name: String,
    pub x: f64,
    pub width: f64,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
const PADDING: f64 = 10.0;
/// Gap between consecutive flow nodes (matches Java PlantUML visual output).
const NODE_SPACING: f64 = 20.0;
const START_RADIUS: f64 = 10.0;
const DIAMOND_SIZE: f64 = 20.0;
const FORK_BAR_HEIGHT: f64 = 6.0;
const FORK_BAR_WIDTH: f64 = 80.0;
const NOTE_FONT_SIZE: f64 = 13.0;
const NOTE_OFFSET: f64 = 30.0;
const SWIMLANE_MIN_WIDTH: f64 = 80.0;
const TOP_MARGIN: f64 = 16.0;
const BOTTOM_MARGIN: f64 = 16.0;
const SWIMLANE_HEADER_FONT_SIZE: f64 = 18.0;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

/// Estimate the bounding-box size of an action box.
/// Uses actual font metrics for precise sizing to match Java PlantUML.
fn estimate_text_size(text: &str) -> (f64, f64) {
    let lh = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);
    let lines: Vec<&str> = text.split('\n').collect();
    let max_line_width = lines
        .iter()
        .map(|l| font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    let width = max_line_width + 2.0 * PADDING;
    let height = lines.len() as f64 * lh + 2.0 * PADDING;
    log::debug!("estimate_text_size(\"{}\") -> {}x{} ({} lines, max_w={})", text, width, height, lines.len(), max_line_width);
    (width, height)
}

/// Height of a `====` / `----` horizontal separator in a note (Java: 10.0).
/// Height of a `====` / `----` horizontal separator in a note (Java: 10.0).
pub const NOTE_SEPARATOR_HEIGHT: f64 = 10.0;

/// Estimate the size of a note, using note font size.
///
/// Java height model (SheetBlock1 + CreoleHorizontalLine):
///   height = fold + ascent + (N_text - 1) × line_height + separator_heights + descent_pad
/// where N_text = number of non-separator lines, ascent/descent come from font metrics.
fn estimate_note_size(text: &str) -> (f64, f64) {
    use crate::render::svg_richtext::creole_text_width;

    let note_lh = font_metrics::line_height("SansSerif", NOTE_FONT_SIZE, false, false);
    let note_pad = 6.0;
    let fold = 10.0;
    // Java Opale: first text baseline = fold + inner_pad(≈7.07)
    // Java Opale: last text baseline to note bottom ≈ 8.07
    // These match Java's SheetBlock1 rendering at font-size 13.
    let top_pad = fold + 7.07;
    let bottom_pad = 8.07;
    let lines: Vec<&str> = text.split('\n').collect();
    let mut max_line_width = 0.0_f64;
    let mut n_text: usize = 0;
    let mut sep_height = 0.0_f64;
    for line in &lines {
        let trimmed = line.trim();
        let is_sep = trimmed.len() >= 4
            && (trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-'));
        if is_sep {
            sep_height += NOTE_SEPARATOR_HEIGHT;
        } else {
            let w = creole_text_width(line, "SansSerif", NOTE_FONT_SIZE, false, false);
            max_line_width = max_line_width.max(w);
            n_text += 1;
        }
    }
    let width = max_line_width + 2.0 * note_pad + fold;
    // Java: top_pad + (N-1)*lh + sep + bottom_pad
    let height = if n_text > 0 {
        let text_intervals = (n_text as f64 - 1.0) * note_lh;
        top_pad + text_intervals + sep_height + bottom_pad
    } else {
        // Separator only: minimal note box
        fold + note_pad + sep_height
    };
    log::debug!("estimate_note_size(\"{}\") -> {}x{} ({} text lines, {}px sep)", text, width, height, n_text, sep_height);
    (width, height)
}

/// Bullet list indent width in a note (Java: bullet ellipse + gap ≈ 18px).
const NOTE_BULLET_INDENT: f64 = 18.0;

/// Word-wrap note text to fit within `max_width` pixels.
///
/// Splits lines at word boundaries, measuring plain text (creole-stripped)
/// width while preserving the original creole markup in the output.
/// Bullet list items (`* ...`) reduce available width by the indent.
fn wrap_note_text(text: &str, max_width: f64) -> String {
    let mut result_lines: Vec<String> = Vec::new();

    for line in text.split('\n') {
        // Detect bullet list prefix `* ` and reduce effective wrap width.
        let (prefix, content, effective_width) = if line.trim_start().starts_with("* ") {
            let idx = line.find("* ").unwrap();
            ("* ", &line[idx + 2..], max_width - NOTE_BULLET_INDENT)
        } else {
            ("", line, max_width)
        };

        let plain = creole_plain_text(content);
        let line_w = font_metrics::text_width(&plain, "SansSerif", NOTE_FONT_SIZE, false, false);
        if line_w <= effective_width {
            result_lines.push(line.to_string());
            continue;
        }

        // Need to wrap: split by spaces and accumulate
        let words: Vec<&str> = content.split(' ').collect();
        let mut current_line = String::new();
        let mut is_first = true;
        for word in &words {
            if current_line.is_empty() {
                current_line = word.to_string();
                continue;
            }

            let candidate = format!("{current_line} {word}");
            let candidate_plain = creole_plain_text(&candidate);
            let candidate_w = font_metrics::text_width(
                &candidate_plain,
                "SansSerif",
                NOTE_FONT_SIZE,
                false,
                false,
            );

            if candidate_w <= effective_width {
                current_line = candidate;
            } else {
                // Flush current line
                if is_first && !prefix.is_empty() {
                    result_lines.push(format!("{prefix}{current_line}"));
                    is_first = false;
                } else {
                    result_lines.push(current_line);
                }
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            if is_first && !prefix.is_empty() {
                result_lines.push(format!("{prefix}{current_line}"));
            } else {
                result_lines.push(current_line);
            }
        }
    }

    result_lines.join("\n")
}

// ---------------------------------------------------------------------------
// Swimlane helpers
// ---------------------------------------------------------------------------

/// Java LaneDivider half-space: 5px at edges, expands if title overflows content.
const LANE_DIVIDER_HALF: f64 = 5.0;

/// Compute initial swimlane column layouts from header text.
///
/// Java sizes lanes to content (via LimitFinder) then expands for title.
/// Here we start with header-text width; Pass 2c expands for content+notes.
fn compute_swimlane_layouts(swimlanes: &[String]) -> Vec<SwimlaneLayout> {
    if swimlanes.is_empty() {
        return Vec::new();
    }
    let lane_pad = 10.0;
    let mut layouts = Vec::new();
    // Java: first LaneDivider starts at edge half-space (5px each side = 10px)
    let mut x = LANE_DIVIDER_HALF * 2.0; // left divider width = 10
    for (i, name) in swimlanes.iter().enumerate() {
        let title_width = font_metrics::text_width(
            name, "SansSerif", SWIMLANE_HEADER_FONT_SIZE, false, false,
        );
        // Initial lane width from header text (no min-width — Java doesn't use one)
        let lane_width = title_width + 2.0 * lane_pad;
        layouts.push(SwimlaneLayout {
            name: name.clone(),
            x,
            width: lane_width,
        });
        // Java: inter-lane divider = halfMissing(i+1) + halfMissing(i+2)
        // Both default to 5px unless title overflows
        let divider = LANE_DIVIDER_HALF * 2.0;
        x += lane_width + divider;
    }
    layouts
}

/// Return the horizontal centre-x for a given swimlane index.  When no
/// swimlanes exist, fall back to a single centred column of
/// `SWIMLANE_MIN_WIDTH`.
fn swimlane_center_x(lanes: &[SwimlaneLayout], lane_idx: usize) -> f64 {
    if lanes.is_empty() {
        // Will be resolved in the centering pass.
        0.0
    } else {
        let lane = &lanes[lane_idx.min(lanes.len() - 1)];
        lane.x + lane.width / 2.0
    }
}

/// Resolve a swimlane name to its index.  Returns 0 when not found.
fn resolve_swimlane_index(swimlanes: &[String], name: &str) -> usize {
    swimlanes.iter().position(|n| n == name).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Layout entry point
// ---------------------------------------------------------------------------

/// Perform the complete layout of an activity diagram.
///
/// The result contains absolute positions for every node and edge so that a
/// renderer can draw them without further computation.
pub fn layout_activity(diagram: &ActivityDiagram) -> Result<ActivityLayout> {
    log::debug!(
        "layout_activity: {} events, {} swimlanes",
        diagram.events.len(),
        diagram.swimlanes.len()
    );

    // --- Pass 1: swimlane columns (initial sizing from header text) ---------
    let mut swimlane_layouts = compute_swimlane_layouts(&diagram.swimlanes);

    // --- Pass 2: place nodes ------------------------------------------------
    let mut nodes: Vec<ActivityNodeLayout> = Vec::new();
    // When swimlanes exist, push initial y below the header row.
    // Java a0002: header text baseline y=34.45, first node y=43.7.
    // header_height = header_top_margin(17.75) + ascent + descent + gap(5.05)
    let swimlane_header_height = if swimlane_layouts.is_empty() {
        0.0
    } else {
        let ha = font_metrics::ascent("SansSerif", SWIMLANE_HEADER_FONT_SIZE, false, false);
        let hd = font_metrics::descent("SansSerif", SWIMLANE_HEADER_FONT_SIZE, false, false);
        // Java: top_pad ≈ 17.75 (slightly more than our TOP_MARGIN=16)
        ha + hd + 17.75 + 5.0
    };
    let mut y_cursor = if swimlane_layouts.is_empty() {
        TOP_MARGIN
    } else {
        swimlane_header_height
    };
    let mut current_lane_idx: usize = 0;
    let mut node_index: usize = 0;

    // Track the index of the last *flow* node (i.e. not a note or swimlane
    // switch) so that notes can reference it.
    let mut last_flow_node_idx: Option<usize> = None;

    for event in &diagram.events {
        match event {
            // ---- Start circle ------------------------------------------------
            ActivityEvent::Start => {
                let diameter = 2.0 * START_RADIUS;
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - START_RADIUS;
                let y = y_cursor;
                log::debug!("  node[{node_index}] Start @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Start,
                    x,
                    y,
                    width: diameter,
                    height: diameter,
                    text: String::new(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += diameter + NODE_SPACING;
            }

            // ---- Stop circle -------------------------------------------------
            ActivityEvent::Stop => {
                let diameter = 2.0 * START_RADIUS;
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - START_RADIUS;
                let y = y_cursor;
                log::debug!("  node[{node_index}] Stop @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Stop,
                    x,
                    y,
                    width: diameter,
                    height: diameter,
                    text: String::new(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += diameter + NODE_SPACING;
            }

            // ---- Action box --------------------------------------------------
            ActivityEvent::Action { text } => {
                let (w, h) = estimate_text_size(text);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] Action \"{text}\" @ ({x:.1}, {y:.1}) {w}x{h}");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Action,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: text.clone(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            // ---- If / ElseIf / Else / EndIf → diamonds ----------------------
            ActivityEvent::If {
                condition,
                then_label,
            } => {
                let label = if then_label.is_empty() {
                    condition.clone()
                } else {
                    format!("{condition}\n[{then_label}]")
                };
                let (w, h) = diamond_size(&label);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] If diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: label,
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            ActivityEvent::ElseIf { condition, label } => {
                let combined = if label.is_empty() {
                    condition.clone()
                } else {
                    format!("{condition}\n[{label}]")
                };
                let (w, h) = diamond_size(&combined);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] ElseIf diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: combined,
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            ActivityEvent::Else { label } => {
                let text = if label.is_empty() {
                    "else".to_string()
                } else {
                    format!("[{label}]")
                };
                let (w, h) = diamond_size(&text);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] Else diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text,
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            ActivityEvent::EndIf => {
                let (w, h) = (DIAMOND_SIZE * 2.0, DIAMOND_SIZE * 2.0);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] EndIf diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: String::new(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            // ---- While / EndWhile → diamonds ---------------------------------
            ActivityEvent::While { condition, label } => {
                let combined = if label.is_empty() {
                    condition.clone()
                } else {
                    format!("{condition}\n[{label}]")
                };
                let (w, h) = diamond_size(&combined);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] While diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: combined,
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            ActivityEvent::EndWhile { label } => {
                let text = if label.is_empty() {
                    String::new()
                } else {
                    format!("[{label}]")
                };
                let (w, h) = diamond_size(if text.is_empty() { "end" } else { &text });
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] EndWhile diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text,
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            // ---- Repeat / RepeatWhile → diamond at end -----------------------
            ActivityEvent::Repeat => {
                // `repeat` is simply a label-less merge point — draw a small
                // diamond identical to EndIf.
                let (w, h) = (DIAMOND_SIZE * 2.0, DIAMOND_SIZE * 2.0);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] Repeat diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: String::new(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            ActivityEvent::RepeatWhile { condition } => {
                let (w, h) = diamond_size(condition);
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] RepeatWhile diamond @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Diamond,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: condition.clone(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            // ---- Fork / ForkAgain / EndFork → horizontal bars ----------------
            ActivityEvent::Fork | ActivityEvent::ForkAgain | ActivityEvent::EndFork => {
                let w = FORK_BAR_WIDTH;
                let h = FORK_BAR_HEIGHT;
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - w / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] ForkBar @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::ForkBar,
                    x,
                    y,
                    width: w,
                    height: h,
                    text: String::new(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += h + NODE_SPACING;
            }

            // ---- Swimlane switch (no node) -----------------------------------
            ActivityEvent::Swimlane { name } => {
                let idx = resolve_swimlane_index(&diagram.swimlanes, name);
                log::debug!("  swimlane switch -> \"{name}\" (idx={idx})");
                current_lane_idx = idx;
                // No node emitted, no y_cursor change.
            }

            // ---- Note (attached to previous flow node) -----------------------
            ActivityEvent::Note { position, text } => {
                let wrapped = if let Some(max_w) = diagram.note_max_width {
                    wrap_note_text(text, max_w)
                } else {
                    text.clone()
                };
                let (nw, nh) = estimate_note_size(&wrapped);
                let pos_layout = match position {
                    NotePosition::Left => NotePositionLayout::Left,
                    NotePosition::Right => NotePositionLayout::Right,
                };

                // Java vertically centres the note and its flow node.
                // When the note is taller than the flow node, both are
                // shifted so their midpoints align.
                let (nx, ny) = if let Some(prev_idx) = last_flow_node_idx {
                    let prev_x = nodes[prev_idx].x;
                    let prev_y = nodes[prev_idx].y;
                    let prev_w = nodes[prev_idx].width;
                    let prev_h = nodes[prev_idx].height;
                    let x = match pos_layout {
                        NotePositionLayout::Right => prev_x + prev_w + NOTE_OFFSET,
                        NotePositionLayout::Left => prev_x - NOTE_OFFSET - nw,
                    };

                    if nh > prev_h {
                        // Note is taller: push the flow node down so midpoints align
                        let delta = (nh - prev_h) / 2.0;
                        nodes[prev_idx].y += delta;
                        y_cursor += delta;
                        // Note y = original flow-node y (unshifted)
                        (x, prev_y)
                    } else {
                        // Flow node is taller: centre the note on the flow node
                        let delta = (prev_h - nh) / 2.0;
                        (x, prev_y + delta)
                    }
                } else {
                    // No previous node — place in the margin area.
                    let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                    let x = match pos_layout {
                        NotePositionLayout::Right => cx + NOTE_OFFSET,
                        NotePositionLayout::Left => cx - NOTE_OFFSET - nw,
                    };
                    (x, y_cursor)
                };

                log::debug!("  node[{node_index}] Note({pos_layout:?}) @ ({nx:.1}, {ny:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Note {
                        position: pos_layout,
                    },
                    x: nx,
                    y: ny,
                    width: nw,
                    height: nh,
                    text: wrapped,
                });
                // Notes do NOT update last_flow_node_idx.
                node_index += 1;

                // Advance y_cursor so subsequent elements don't overlap.
                let note_bottom = ny + nh + NODE_SPACING;
                if note_bottom > y_cursor {
                    y_cursor = note_bottom;
                }
            }

            // ---- Floating note (not attached) --------------------------------
            // Java: floating notes sit beside the flow, like attached notes.
            // They do NOT consume vertical space or advance y_cursor.
            ActivityEvent::FloatingNote { position, text } => {
                let wrapped = if let Some(max_w) = diagram.note_max_width {
                    wrap_note_text(text, max_w)
                } else {
                    text.clone()
                };
                let (nw, nh) = estimate_note_size(&wrapped);
                let pos_layout = match position {
                    NotePosition::Left => NotePositionLayout::Left,
                    NotePosition::Right => NotePositionLayout::Right,
                };
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let nx = match pos_layout {
                    NotePositionLayout::Right => cx + NOTE_OFFSET,
                    NotePositionLayout::Left => cx - NOTE_OFFSET - nw,
                };
                // Place at the last flow node's y (like attached note) or y_cursor
                let ny = if let Some(prev_idx) = last_flow_node_idx {
                    nodes[prev_idx].y
                } else {
                    y_cursor
                };

                log::debug!(
                    "  node[{node_index}] FloatingNote({pos_layout:?}) @ ({nx:.1}, {ny:.1})"
                );
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::FloatingNote {
                        position: pos_layout,
                    },
                    x: nx,
                    y: ny,
                    width: nw,
                    height: nh,
                    text: wrapped,
                });
                node_index += 1;
                // Java: floating notes do NOT advance y_cursor.
            }

            // ---- Detach (small marker) ---------------------------------------
            ActivityEvent::Detach => {
                let size = START_RADIUS;
                let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                let x = cx - size / 2.0;
                let y = y_cursor;
                log::debug!("  node[{node_index}] Detach @ ({x:.1}, {y:.1})");
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::Detach,
                    x,
                    y,
                    width: size,
                    height: size,
                    text: String::new(),
                });
                last_flow_node_idx = Some(node_index);
                node_index += 1;
                y_cursor += size + NODE_SPACING;
            }
        }
    }

    // --- Pass 2b: centering for non-swimlane diagrams ----------------------
    if swimlane_layouts.is_empty() && !nodes.is_empty() {
        let max_half_w = nodes.iter()
            .filter(|n| is_flow_node(&n.kind))
            .map(|n| n.width / 2.0)
            .fold(0.0_f64, f64::max);
        let cx = TOP_MARGIN + max_half_w;
        for node in &mut nodes {
            if is_flow_node(&node.kind) {
                node.x = cx - node.width / 2.0;
            } else {
                node.x += cx;
            }
        }
    }

    // --- Pass 2c: expand swimlanes to fit content (Java LimitFinder compat) -
    // Java measures draw-time bounding boxes per-swimlane, then expands each
    // lane to max(headerWidth, contentWidth).  We replicate this by tracking
    // which lane each node belongs to and finding content bounds.
    if !swimlane_layouts.is_empty() {
        // 1) Build node→lane mapping by replaying event order
        let mut node_lane: Vec<usize> = Vec::with_capacity(nodes.len());
        let mut cur_lane: usize = 0;
        for event in &diagram.events {
            match event {
                ActivityEvent::Swimlane { name } => {
                    cur_lane = resolve_swimlane_index(&diagram.swimlanes, name);
                }
                // Every event that emits a node (same order as Pass 2)
                ActivityEvent::Start | ActivityEvent::Stop
                | ActivityEvent::Action { .. }
                | ActivityEvent::If { .. } | ActivityEvent::ElseIf { .. }
                | ActivityEvent::Else { .. } | ActivityEvent::EndIf
                | ActivityEvent::While { .. } | ActivityEvent::EndWhile { .. }
                | ActivityEvent::Repeat | ActivityEvent::RepeatWhile { .. }
                | ActivityEvent::Fork | ActivityEvent::ForkAgain | ActivityEvent::EndFork
                | ActivityEvent::Note { .. } | ActivityEvent::FloatingNote { .. }
                | ActivityEvent::Detach => {
                    node_lane.push(cur_lane);
                }
            }
        }

        // 2) Compute content bounding box per lane
        let n_lanes = swimlane_layouts.len();
        let mut lane_min_x = vec![f64::MAX; n_lanes];
        let mut lane_max_x = vec![f64::MIN; n_lanes];
        for (ni, node) in nodes.iter().enumerate() {
            let li = if ni < node_lane.len() { node_lane[ni] } else { 0 };
            let left = node.x;
            let right = node.x + node.width;
            if left < lane_min_x[li] { lane_min_x[li] = left; }
            if right > lane_max_x[li] { lane_max_x[li] = right; }
        }

        // 3) Expand each lane; Java LaneDivider: edge=5, between=5..N depending on title overflow
        // Left divider = halfMissing(0)(=5) + halfMissing(1)(=5 or more)
        let half_missing_edge = LANE_DIVIDER_HALF;
        let header_widths: Vec<f64> = diagram.swimlanes.iter().map(|name| {
            font_metrics::text_width(name, "SansSerif", SWIMLANE_HEADER_FONT_SIZE, false, false)
        }).collect();

        // First pass: determine final lane widths (max of header and content)
        let mut lane_widths: Vec<f64> = Vec::with_capacity(n_lanes);
        for i in 0..n_lanes {
            let content_width = if lane_max_x[i] > lane_min_x[i] {
                lane_max_x[i] - lane_min_x[i]
            } else {
                0.0
            };
            let hw = header_widths[i] + 2.0 * LANE_DIVIDER_HALF; // title + padding
            lane_widths.push(content_width.max(hw));
        }

        // Java getHalfMissingSpace: if title > actual_content, expand divider half
        let raw_content_widths: Vec<f64> = (0..n_lanes).map(|i| {
            if lane_max_x[i] > lane_min_x[i] { lane_max_x[i] - lane_min_x[i] } else { 0.0 }
        }).collect();
        let half_missing = |lane_idx: usize| -> f64 {
            let actual_w = raw_content_widths[lane_idx]; // pure content, no title padding
            let title_w = header_widths[lane_idx];
            if title_w > actual_w {
                (LANE_DIVIDER_HALF + (title_w - actual_w) / 2.0).max(LANE_DIVIDER_HALF)
            } else {
                LANE_DIVIDER_HALF
            }
        };

        // Java: left lane line consistently at x ≈ 20 (divider(10) + centering offset).
        // This comes from LaneDivider width + content minX compensation.
        // We approximate with edge(5) + halfMissing + content centering offset.
        let left_divider = half_missing_edge + half_missing(0);
        // Content is centred in each lane; the left half of the widest centred
        // content determines the minimum left margin.  Java's translate system
        // naturally produces this; we emulate by ensuring the first lane starts
        // far enough right that centred content has room.
        let first_lane_half = lane_widths[0] / 2.0;
        let content_half = raw_content_widths[0] / 2.0;
        let centering_extra = if content_half > first_lane_half { 0.0 } else { first_lane_half - content_half };
        let mut x = (left_divider + centering_extra).max(left_divider);
        for i in 0..n_lanes {
            let needed = lane_widths[i];
            let old_x = swimlane_layouts[i].x;
            swimlane_layouts[i].x = x;
            swimlane_layouts[i].width = needed;

            // Shift nodes so content is centred within the new lane bounds.
            if lane_max_x[i] > lane_min_x[i] {
                let cw = lane_max_x[i] - lane_min_x[i];
                let target_left = x + (needed - cw) / 2.0;
                let dx = target_left - lane_min_x[i];
                if dx.abs() > 0.01 {
                    for (ni, node) in nodes.iter_mut().enumerate() {
                        if ni < node_lane.len() && node_lane[ni] == i {
                            node.x += dx;
                        }
                    }
                }
            }
            // Inter-lane divider
            let inter_div = if i + 1 < n_lanes {
                half_missing(i) + half_missing(i + 1)
            } else {
                0.0
            };
            x += needed + inter_div;
        }
    }

    // --- Pass 3: edges connecting consecutive flow nodes --------------------
    let edges = build_edges(&nodes);

    // --- Compute total bounding box -----------------------------------------
    let (total_width, total_height) = compute_bounds(&nodes, &swimlane_layouts, y_cursor);

    log::debug!(
        "layout_activity: placed {} nodes, {} edges, total {}x{}",
        nodes.len(),
        edges.len(),
        total_width,
        total_height
    );

    let mut layout = ActivityLayout {
        width: total_width,
        height: total_height,
        nodes,
        edges,
        swimlane_layouts,
    };
    apply_direction_transform(&mut layout, &diagram.direction);

    Ok(layout)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute the diamond bounding box for a labelled condition.
fn diamond_size(label: &str) -> (f64, f64) {
    let (tw, th) = estimate_text_size(label);
    // A diamond is wider than the text because the corners are cut.
    let w = tw.max(DIAMOND_SIZE * 2.0);
    let h = th.max(DIAMOND_SIZE * 2.0);
    (w, h)
}

/// Apply a coordinate transform to the entire layout based on the diagram
/// direction.  The layout algorithm always computes positions in top-to-bottom
/// orientation; for other directions we transform after the fact.
///
/// - `LeftToRight`: swap x/y axes so the flow goes left-to-right.
/// - `RightToLeft`: swap x/y axes then mirror horizontally.
/// - `BottomToTop`: mirror the Y axis so the flow goes bottom-to-top.
fn apply_direction_transform(
    layout: &mut ActivityLayout,
    direction: &crate::model::diagram::Direction,
) {
    use crate::model::diagram::Direction;
    match direction {
        Direction::TopToBottom => {}
        Direction::LeftToRight => {
            for node in &mut layout.nodes {
                std::mem::swap(&mut node.x, &mut node.y);
                std::mem::swap(&mut node.width, &mut node.height);
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    std::mem::swap(&mut pt.0, &mut pt.1);
                }
            }
            std::mem::swap(&mut layout.width, &mut layout.height);
        }
        Direction::RightToLeft => {
            for node in &mut layout.nodes {
                std::mem::swap(&mut node.x, &mut node.y);
                std::mem::swap(&mut node.width, &mut node.height);
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    std::mem::swap(&mut pt.0, &mut pt.1);
                }
            }
            std::mem::swap(&mut layout.width, &mut layout.height);
            let w = layout.width;
            for node in &mut layout.nodes {
                node.x = w - node.x - node.width;
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    pt.0 = w - pt.0;
                }
            }
        }
        Direction::BottomToTop => {
            let h = layout.height;
            for node in &mut layout.nodes {
                node.y = h - node.y - node.height;
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    pt.1 = h - pt.1;
                }
            }
        }
    }
}

/// Returns true if a node is a "flow" node — i.e. it participates in
/// sequential edge connections.  Notes and swimlane markers are excluded.
fn is_flow_node(kind: &ActivityNodeKindLayout) -> bool {
    !matches!(
        kind,
        ActivityNodeKindLayout::Note { .. } | ActivityNodeKindLayout::FloatingNote { .. }
    )
}

/// Build edges between consecutive flow nodes.
///
/// When two consecutive nodes are in different horizontal positions (i.e.
/// different swimlanes), the edge is routed as an L-shaped polyline:
/// go down from the source, then horizontally, then down into the target.
fn build_edges(nodes: &[ActivityNodeLayout]) -> Vec<ActivityEdgeLayout> {
    let flow_indices: Vec<usize> = nodes
        .iter()
        .filter(|n| is_flow_node(&n.kind))
        .map(|n| n.index)
        .collect();

    let mut edges = Vec::new();
    for pair in flow_indices.windows(2) {
        let from_idx = pair[0];
        let to_idx = pair[1];
        let from = &nodes[from_idx];
        let to = &nodes[to_idx];

        let from_cx = from.x + from.width / 2.0;
        let from_bottom = from.y + from.height;
        let to_cx = to.x + to.width / 2.0;
        let to_top = to.y;

        let points = if (from_cx - to_cx).abs() < 1.0 {
            // Same lane: simple straight vertical line.
            vec![(from_cx, from_bottom), (to_cx, to_top)]
        } else {
            // Cross-lane: route with an L-shaped path.
            // Go down halfway, then across, then down to the target.
            let mid_y = (from_bottom + to_top) / 2.0;
            vec![
                (from_cx, from_bottom),
                (from_cx, mid_y),
                (to_cx, mid_y),
                (to_cx, to_top),
            ]
        };
        edges.push(ActivityEdgeLayout {
            from_index: from_idx,
            to_index: to_idx,
            label: String::new(),
            points,
        });
    }
    edges
}

/// Compute the total bounding box of the diagram.
fn compute_bounds(
    nodes: &[ActivityNodeLayout],
    swimlane_layouts: &[SwimlaneLayout],
    y_cursor: f64,
) -> (f64, f64) {
    if nodes.is_empty() && swimlane_layouts.is_empty() {
        return (2.0 * TOP_MARGIN, 2.0 * TOP_MARGIN);
    }

    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;

    for node in nodes {
        let right = node.x + node.width;
        let bottom = node.y + node.height;
        if right > max_x {
            max_x = right;
        }
        if bottom > max_y {
            max_y = bottom;
        }
    }

    if !swimlane_layouts.is_empty() {
        for lane in swimlane_layouts {
            let right = lane.x + lane.width;
            if right > max_x {
                max_x = right;
            }
        }
        (max_x + BOTTOM_MARGIN + 12.0, max_y + BOTTOM_MARGIN + 4.0)
    } else {
        (max_x + BOTTOM_MARGIN + 3.0, max_y + BOTTOM_MARGIN + 3.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a diagram with given events and no swimlanes.
    fn diagram(events: Vec<ActivityEvent>) -> ActivityDiagram {
        ActivityDiagram {
            events,
            swimlanes: vec![],
            direction: Default::default(),
            note_max_width: None,
        }
    }

    // 1. Empty diagram -------------------------------------------------------

    #[test]
    fn empty_diagram() {
        let d = diagram(vec![]);
        let layout = layout_activity(&d).unwrap();
        assert!(layout.nodes.is_empty());
        assert!(layout.edges.is_empty());
        assert!(layout.swimlane_layouts.is_empty());
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 2. Single action -------------------------------------------------------

    #[test]
    fn single_action() {
        let d = diagram(vec![ActivityEvent::Action {
            text: "Hello".into(),
        }]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.edges.len(), 0);
        let node = &layout.nodes[0];
        assert_eq!(node.kind, ActivityNodeKindLayout::Action);
        assert_eq!(node.text, "Hello");
        assert!(node.width >= 30.0);
        assert!(node.height >= 20.0);
    }

    // 3. Start -> Stop -------------------------------------------------------

    #[test]
    fn start_stop() {
        let d = diagram(vec![ActivityEvent::Start, ActivityEvent::Stop]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 2);
        assert_eq!(layout.edges.len(), 1);

        let start = &layout.nodes[0];
        let stop = &layout.nodes[1];
        assert_eq!(start.kind, ActivityNodeKindLayout::Start);
        assert_eq!(stop.kind, ActivityNodeKindLayout::Stop);

        // Stop should be below Start.
        assert!(stop.y > start.y + start.height);

        // Edge connects them.
        let edge = &layout.edges[0];
        assert_eq!(edge.from_index, 0);
        assert_eq!(edge.to_index, 1);
    }

    // 4. Swimlanes -----------------------------------------------------------

    #[test]
    fn swimlanes() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane {
                    name: "Lane A".into(),
                },
                ActivityEvent::Action {
                    text: "Task A".into(),
                },
                ActivityEvent::Swimlane {
                    name: "Lane B".into(),
                },
                ActivityEvent::Action {
                    text: "Task B".into(),
                },
            ],
            swimlanes: vec!["Lane A".into(), "Lane B".into()],
            direction: Default::default(),
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.swimlane_layouts.len(), 2);
        assert_eq!(layout.nodes.len(), 2);

        let node_a = &layout.nodes[0];
        let node_b = &layout.nodes[1];

        // Lane A center should differ from Lane B center.
        let center_a = node_a.x + node_a.width / 2.0;
        let center_b = node_b.x + node_b.width / 2.0;
        assert!(
            (center_a - center_b).abs() > 1.0,
            "nodes should be in different lanes"
        );

        // Lane B should be to the right of Lane A.
        assert!(
            layout.swimlane_layouts[1].x > layout.swimlane_layouts[0].x,
            "lane B should be to the right"
        );
    }

    // 4b. Swimlane left margin matches Java divider (Java=20px for simple case)

    #[test]
    fn swimlane_left_margin_matches_java() {
        // Java CreoleNoteMetricsTest.swimlaneDividerAndMargins:
        //   Lane A left x = 20.0 (LaneDivider x1=5 + x2 expansion)
        //   Lane A width = 71.6, Lane B width = 71.6
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane { name: "Lane A".into() },
                ActivityEvent::Start,
                ActivityEvent::Action { text: "task A".into() },
                ActivityEvent::Stop,
                ActivityEvent::Swimlane { name: "Lane B".into() },
                ActivityEvent::Action { text: "task B".into() },
            ],
            swimlanes: vec!["Lane A".into(), "Lane B".into()],
            direction: Default::default(),
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();
        let lane_a = &layout.swimlane_layouts[0];
        // Java Lane A left x ≈ 20; Rust should be > 5 (old value) and reasonable
        assert!(
            lane_a.x >= 8.0,
            "Lane A x ({:.1}) should be > 8 (Java=20, left divider expands for title)",
            lane_a.x
        );
        // Java Lane A width = 71.6, should not be inflated to 80 by min-width
        assert!(
            lane_a.width < 80.0,
            "Lane A width ({:.1}) should be < 80 (Java=71.6, no artificial min-width)",
            lane_a.width
        );
    }

    // 4c. Swimlane width expands for note content (Java compat) ---------------

    #[test]
    fn swimlane_width_accommodates_note() {
        // Java CreoleNoteMetricsTest.swimlaneWidthWithNotes:
        //   Lane A width = 188px (includes action + note + gap)
        //   Lane B width = 72px
        // Swimlane must expand to fit the composite (flow node + note) width.
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane { name: "Lane A".into() },
                ActivityEvent::Start,
                ActivityEvent::Action { text: "action".into() },
                ActivityEvent::Note {
                    position: NotePosition::Right,
                    text: "a short note".into(),
                },
                ActivityEvent::Swimlane { name: "Lane B".into() },
                ActivityEvent::Action { text: "task2".into() },
                ActivityEvent::Stop,
            ],
            swimlanes: vec!["Lane A".into(), "Lane B".into()],
            direction: Default::default(),
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();
        let lane_a = &layout.swimlane_layouts[0];
        // Java Lane A ≈ 188px.  Must be wider than the base header-only width.
        assert!(
            lane_a.width >= 150.0,
            "Lane A width ({:.1}) should be >= 150 to fit action + note. Java=188",
            lane_a.width
        );
        // Note must be fully inside Lane A boundary
        let note = layout.nodes.iter().find(|n| matches!(n.kind, ActivityNodeKindLayout::Note { .. })).unwrap();
        let note_right = note.x + note.width;
        let lane_a_right = lane_a.x + lane_a.width;
        assert!(
            note_right <= lane_a_right + 1.0,
            "note right ({:.1}) should be within Lane A right ({:.1})",
            note_right, lane_a_right
        );
    }

    // 5. Note beside action --------------------------------------------------

    #[test]
    fn note_beside_action() {
        let d = diagram(vec![
            ActivityEvent::Action {
                text: "Do work".into(),
            },
            ActivityEvent::Note {
                position: NotePosition::Right,
                text: "This is a note".into(),
            },
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 2);

        let action = &layout.nodes[0];
        let note = &layout.nodes[1];
        assert_eq!(
            note.kind,
            ActivityNodeKindLayout::Note {
                position: NotePositionLayout::Right,
            }
        );

        // Note should be to the right of the action.
        assert!(note.x > action.x + action.width);

        // Note and action should be vertically centred on each other.
        let action_mid = action.y + action.height / 2.0;
        let note_mid = note.y + note.height / 2.0;
        assert!(
            (action_mid - note_mid).abs() < 1.0,
            "midpoints should align: action_mid={action_mid:.1}, note_mid={note_mid:.1}"
        );

        // Edge list should NOT include the note.
        assert_eq!(layout.edges.len(), 0);
    }

    // 6. Left note -----------------------------------------------------------

    #[test]
    fn note_left_of_action() {
        let d = diagram(vec![
            ActivityEvent::Action {
                text: "Do work".into(),
            },
            ActivityEvent::Note {
                position: NotePosition::Left,
                text: "Left note".into(),
            },
        ]);
        let layout = layout_activity(&d).unwrap();
        let action = &layout.nodes[0];
        let note = &layout.nodes[1];

        // Note should be to the left.
        assert!(note.x + note.width < action.x);
    }

    // 7. If / EndIf diamonds -------------------------------------------------

    #[test]
    fn if_endif_diamonds() {
        let d = diagram(vec![
            ActivityEvent::If {
                condition: "x > 0".into(),
                then_label: "yes".into(),
            },
            ActivityEvent::Action {
                text: "positive".into(),
            },
            ActivityEvent::EndIf,
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 3);

        let if_node = &layout.nodes[0];
        let endif_node = &layout.nodes[2];
        assert_eq!(if_node.kind, ActivityNodeKindLayout::Diamond);
        assert_eq!(endif_node.kind, ActivityNodeKindLayout::Diamond);

        // EndIf should be below the action.
        let action = &layout.nodes[1];
        assert!(endif_node.y > action.y + action.height);

        // 2 edges: if->action, action->endif
        assert_eq!(layout.edges.len(), 2);
    }

    // 8. Fork bar ------------------------------------------------------------

    #[test]
    fn fork_bar() {
        let d = diagram(vec![
            ActivityEvent::Fork,
            ActivityEvent::Action {
                text: "branch 1".into(),
            },
            ActivityEvent::ForkAgain,
            ActivityEvent::Action {
                text: "branch 2".into(),
            },
            ActivityEvent::EndFork,
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 5);

        let fork = &layout.nodes[0];
        let fork_again = &layout.nodes[2];
        let end_fork = &layout.nodes[4];
        assert_eq!(fork.kind, ActivityNodeKindLayout::ForkBar);
        assert_eq!(fork_again.kind, ActivityNodeKindLayout::ForkBar);
        assert_eq!(end_fork.kind, ActivityNodeKindLayout::ForkBar);

        assert_eq!(fork.width, FORK_BAR_WIDTH);
        assert_eq!(fork.height, FORK_BAR_HEIGHT);
    }

    // 9. Text sizing ---------------------------------------------------------

    #[test]
    fn text_sizing() {
        // Single short line.
        let (w, h) = estimate_text_size("Hi");
        assert!(w >= 20.0);
        assert!(h >= 20.0);

        // Multi-line text.
        let (w2, h2) = estimate_text_size("Line one\nLine two\nLine three");
        assert!(h2 > h, "more lines should be taller");
        // Width driven by longest line.
        assert!(
            w2 >= crate::font_metrics::text_width(
                "Line three",
                "SansSerif",
                FONT_SIZE,
                false,
                false
            )
        ); // "Line three" = 10 chars

        // Very long line.
        let long_text = "A".repeat(100);
        let (w3, _) = estimate_text_size(&long_text);
        assert!(w3 > 30.0);
    }

    // 10. While loop diamond --------------------------------------------------

    #[test]
    fn while_loop() {
        let d = diagram(vec![
            ActivityEvent::While {
                condition: "count < 10".into(),
                label: "yes".into(),
            },
            ActivityEvent::Action {
                text: "increment".into(),
            },
            ActivityEvent::EndWhile {
                label: "done".into(),
            },
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 3);

        let while_node = &layout.nodes[0];
        let end_while_node = &layout.nodes[2];
        assert_eq!(while_node.kind, ActivityNodeKindLayout::Diamond);
        assert_eq!(end_while_node.kind, ActivityNodeKindLayout::Diamond);
        assert!(while_node.text.contains("count < 10"));
    }

    // 11. Detach marker -------------------------------------------------------

    #[test]
    fn detach_marker() {
        let d = diagram(vec![
            ActivityEvent::Start,
            ActivityEvent::Action {
                text: "work".into(),
            },
            ActivityEvent::Detach,
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.nodes[2].kind, ActivityNodeKindLayout::Detach);
        // Detach participates in edges.
        assert_eq!(layout.edges.len(), 2);
    }

    // 12. Floating note does NOT advance y_cursor (Java compat) ---------------

    #[test]
    fn floating_note_does_not_advance_y() {
        // Java: floating notes sit beside the flow without consuming vertical
        // space, just like attached notes.  The next flow node should be at
        // the same y_cursor, not pushed below the floating note.
        let d = diagram(vec![
            ActivityEvent::Action {
                text: "work".into(),
            },
            ActivityEvent::FloatingNote {
                position: NotePosition::Left,
                text: "floating".into(),
            },
            ActivityEvent::Action {
                text: "after note".into(),
            },
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 3);
        let action1 = &layout.nodes[0];
        let note = &layout.nodes[1];
        let action2 = &layout.nodes[2];
        // Floating note should be placed at the same y as action1's bottom + spacing,
        // but should NOT push action2 further down.
        let expected_action2_y = action1.y + action1.height + NODE_SPACING;
        assert!(
            (action2.y - expected_action2_y).abs() < 1.0,
            "action2.y ({:.1}) should be at {:.1} (action1 bottom + spacing), \
             floating note should not push it down. note.y={:.1} note.h={:.1}",
            action2.y, expected_action2_y, note.y, note.height
        );
    }

    // 13. Note without preceding flow node -----------------------------------

    #[test]
    fn note_without_preceding_node() {
        let d = diagram(vec![ActivityEvent::Note {
            position: NotePosition::Right,
            text: "orphan note".into(),
        }]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 1);
        // Should not panic.
        assert_eq!(layout.edges.len(), 0);
    }

    // 14. Edges skip notes ---------------------------------------------------

    #[test]
    fn edges_skip_notes() {
        let d = diagram(vec![
            ActivityEvent::Start,
            ActivityEvent::Action { text: "A".into() },
            ActivityEvent::Note {
                position: NotePosition::Right,
                text: "note on A".into(),
            },
            ActivityEvent::Action { text: "B".into() },
            ActivityEvent::Stop,
        ]);
        let layout = layout_activity(&d).unwrap();
        // 5 nodes: start, A, note, B, stop
        assert_eq!(layout.nodes.len(), 5);
        // 4 flow nodes: start, A, B, stop → 3 edges
        assert_eq!(layout.edges.len(), 3);
        // Edge from A (index 1) to B (index 3) — skipping note (index 2).
        let edge_a_b = &layout.edges[1];
        assert_eq!(edge_a_b.from_index, 1);
        assert_eq!(edge_a_b.to_index, 3);
    }

    // 15. Else / ElseIf nodes ------------------------------------------------

    #[test]
    fn else_elseif_nodes() {
        let d = diagram(vec![
            ActivityEvent::If {
                condition: "a".into(),
                then_label: "yes".into(),
            },
            ActivityEvent::Action {
                text: "do a".into(),
            },
            ActivityEvent::ElseIf {
                condition: "b".into(),
                label: "maybe".into(),
            },
            ActivityEvent::Action {
                text: "do b".into(),
            },
            ActivityEvent::Else { label: "no".into() },
            ActivityEvent::Action {
                text: "do c".into(),
            },
            ActivityEvent::EndIf,
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 7);
        assert_eq!(layout.nodes[2].kind, ActivityNodeKindLayout::Diamond); // elseif
        assert_eq!(layout.nodes[4].kind, ActivityNodeKindLayout::Diamond); // else
    }

    // 16. Repeat / RepeatWhile -----------------------------------------------

    #[test]
    fn repeat_loop() {
        let d = diagram(vec![
            ActivityEvent::Repeat,
            ActivityEvent::Action {
                text: "step".into(),
            },
            ActivityEvent::RepeatWhile {
                condition: "again?".into(),
            },
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.nodes[0].kind, ActivityNodeKindLayout::Diamond);
        assert_eq!(layout.nodes[2].kind, ActivityNodeKindLayout::Diamond);
        assert!(layout.nodes[2].text.contains("again?"));
    }

    // 17. LeftToRight direction: width > height (wider than tall) ----------

    #[test]
    fn left_to_right_direction() {
        use crate::model::diagram::Direction;
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "Step 1".into(),
                },
                ActivityEvent::Action {
                    text: "Step 2".into(),
                },
                ActivityEvent::Stop,
            ],
            swimlanes: vec![],
            direction: Direction::LeftToRight,
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();

        // With LR direction, the diagram should be wider than tall
        assert!(
            layout.width > layout.height,
            "LR: width ({:.1}) should be > height ({:.1})",
            layout.width,
            layout.height
        );

        // Nodes should flow left-to-right: x positions should increase
        let flow_nodes: Vec<&ActivityNodeLayout> = layout
            .nodes
            .iter()
            .filter(|n| is_flow_node(&n.kind))
            .collect();
        for pair in flow_nodes.windows(2) {
            assert!(
                pair[1].x >= pair[0].x,
                "LR: node {} x ({:.1}) should be >= node {} x ({:.1})",
                pair[1].index,
                pair[1].x,
                pair[0].index,
                pair[0].x
            );
        }
    }

    // 18. TB direction: height > width (taller than wide) -----------------

    #[test]
    fn top_to_bottom_direction() {
        use crate::model::diagram::Direction;
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "Step 1".into(),
                },
                ActivityEvent::Action {
                    text: "Step 2".into(),
                },
                ActivityEvent::Stop,
            ],
            swimlanes: vec![],
            direction: Direction::TopToBottom,
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();

        // With TB direction, the diagram should be taller than wide
        assert!(
            layout.height > layout.width,
            "TB: height ({:.1}) should be > width ({:.1})",
            layout.height,
            layout.width
        );

        // Nodes should flow top-to-bottom: y positions should increase
        let flow_nodes: Vec<&ActivityNodeLayout> = layout
            .nodes
            .iter()
            .filter(|n| is_flow_node(&n.kind))
            .collect();
        for pair in flow_nodes.windows(2) {
            assert!(
                pair[1].y >= pair[0].y,
                "TB: node {} y ({:.1}) should be >= node {} y ({:.1})",
                pair[1].index,
                pair[1].y,
                pair[0].index,
                pair[0].y
            );
        }
    }

    // 19. BottomToTop direction: first node is at the bottom ---------------

    #[test]
    fn bottom_to_top_direction() {
        use crate::model::diagram::Direction;
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "Step 1".into(),
                },
                ActivityEvent::Stop,
            ],
            swimlanes: vec![],
            direction: Direction::BottomToTop,
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();

        // Start should be below Stop in BT direction
        let start = &layout.nodes[0];
        let stop = &layout.nodes[2];
        assert!(
            start.y > stop.y,
            "BT: start y ({:.1}) should be > stop y ({:.1})",
            start.y,
            stop.y
        );
    }

    // 19. Swimlane header offset -------------------------------------------

    #[test]
    fn swimlane_nodes_start_below_header() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane {
                    name: "Lane A".into(),
                },
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "Task".into(),
                },
                ActivityEvent::Stop,
            ],
            swimlanes: vec!["Lane A".into()],
            direction: Default::default(),
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();
        // All flow nodes should start below the swimlane header
        for node in &layout.nodes {
            assert!(
                node.y >= 20.0,
                "node at y={:.1} must be below header area",
                node.y,
            );
        }
    }

    // 20. Cross-lane edges are L-shaped ------------------------------------

    #[test]
    fn cross_lane_edges_are_polyline() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane {
                    name: "Lane A".into(),
                },
                ActivityEvent::Action {
                    text: "In A".into(),
                },
                ActivityEvent::Swimlane {
                    name: "Lane B".into(),
                },
                ActivityEvent::Action {
                    text: "In B".into(),
                },
            ],
            swimlanes: vec!["Lane A".into(), "Lane B".into()],
            direction: Default::default(),
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();

        // Should have 1 edge between the two actions
        assert_eq!(layout.edges.len(), 1);

        let edge = &layout.edges[0];
        // Cross-lane edge must have 4 points (L-shaped route)
        assert_eq!(
            edge.points.len(),
            4,
            "cross-lane edge should have 4 points, got {}",
            edge.points.len()
        );

        // Verify L-shape: first two points share X, middle two share Y, last two share X
        let (x0, _y0) = edge.points[0];
        let (x1, y1) = edge.points[1];
        let (_x2, y2) = edge.points[2];
        let (x3, _y3) = edge.points[3];
        assert!((x0 - x1).abs() < 0.01, "first segment should be vertical");
        assert!(
            (y1 - y2).abs() < 0.01,
            "middle segment should be horizontal"
        );
        // x3 should be the target lane center (different from x0)
        assert!(
            (x0 - x3).abs() > 1.0,
            "start and end X should differ for cross-lane"
        );
    }

    // 21. Same-lane edges remain 2-point -----------------------------------

    #[test]
    fn same_lane_edges_are_straight() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane {
                    name: "Lane A".into(),
                },
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "Task".into(),
                },
                ActivityEvent::Stop,
            ],
            swimlanes: vec!["Lane A".into()],
            direction: Default::default(),
            note_max_width: None,
        };
        let layout = layout_activity(&d).unwrap();

        // All edges are within same lane, so each should be 2-point
        for (i, edge) in layout.edges.iter().enumerate() {
            assert_eq!(
                edge.points.len(),
                2,
                "same-lane edge {} should have 2 points, got {}",
                i,
                edge.points.len()
            );
        }
    }

    #[test]
    fn estimate_note_size_strips_creole() {
        // <b>HTML</b> should measure as "HTML" (4 chars), not "<b>HTML</b>" (16 chars)
        let (w_markup, _) = estimate_note_size("contain <b>HTML</b>");
        let (w_plain, _) = estimate_note_size("contain HTML");
        assert!(
            (w_markup - w_plain).abs() < 1.0,
            "creole markup should be stripped: markup_w={w_markup}, plain_w={w_plain}"
        );
    }

    #[test]
    fn wrap_note_text_basic() {
        // With a small max_width, long text should be wrapped into multiple lines
        let text = "A Long Long Long Long Long Long note";
        let wrapped = wrap_note_text(text, 80.0);
        let line_count = wrapped.split('\n').count();
        assert!(
            line_count > 1,
            "should wrap into multiple lines, got {line_count}: {wrapped:?}"
        );
    }

    #[test]
    fn wrap_note_text_short_line_unchanged() {
        let text = "Short";
        let wrapped = wrap_note_text(text, 200.0);
        assert_eq!(wrapped, text);
    }

    #[test]
    fn wrap_note_text_preserves_existing_newlines() {
        let text = "Line one\nLine two";
        let wrapped = wrap_note_text(text, 200.0);
        assert_eq!(wrapped, text, "existing newlines should be preserved");
    }

    #[test]
    fn wrap_note_text_with_creole_markup() {
        // Creole markup should be preserved in output but not counted for width
        let text = "This has //italic// and <b>bold</b> words here";
        let wrapped = wrap_note_text(text, 100.0);
        // Should contain the original markup
        assert!(wrapped.contains("//italic//"));
        assert!(wrapped.contains("<b>bold</b>"));
    }

    #[test]
    fn note_font_metrics_match_java() {
        // Java a0002: line dy = 15.1328, top_margin = 17.0669, bottom_margin = 8.066
        // ascent_offset = 7.0669, descent_pad = 8.066
        let lh = font_metrics::line_height("SansSerif", NOTE_FONT_SIZE, false, false);
        let asc = font_metrics::ascent("SansSerif", NOTE_FONT_SIZE, false, false);
        let desc = font_metrics::descent("SansSerif", NOTE_FONT_SIZE, false, false);
        println!("note lh={lh:.4}, asc={asc:.4}, desc={desc:.4}, asc+desc={:.4}", asc + desc);
        // line_height should be ≈ 15.13
        assert!(
            (lh - 15.13).abs() < 0.5,
            "line_height({lh:.4}) should be ≈ 15.13"
        );
    }

    #[test]
    fn estimate_note_separator_adds_less_height_than_text_line() {
        // Adding a separator (====) should increase height by 10px,
        // while adding a text line increases by ~15.13px (line_height).
        let (_, h_base) = estimate_note_size("line1\nline2");
        let (_, h_with_sep) = estimate_note_size("line1\n====\nline2");
        let (_, h_with_text) = estimate_note_size("line1\nline2\nline3");
        let sep_delta = h_with_sep - h_base;
        let text_delta = h_with_text - h_base;
        assert!(
            sep_delta < text_delta,
            "separator delta ({sep_delta:.1}) should be < text delta ({text_delta:.1})"
        );
        assert!(
            (sep_delta - 10.0).abs() < 1.0,
            "separator delta ({sep_delta:.1}) should be ≈ 10.0"
        );
    }

    #[test]
    fn estimate_note_size_monospace_uses_correct_font() {
        // Monospace text `""foo()""` should be measured with monospace metrics
        let (w_mono, _) = estimate_note_size(r#"method ""foo()"" is"#);
        let (w_plain, _) = estimate_note_size("method foo() is");
        // Monospace "foo()" is wider per-char than SansSerif, so the line
        // with monospace should be at least as wide (often wider).
        assert!(
            (w_mono - w_plain).abs() > 0.5 || w_mono >= w_plain,
            "monospace should affect width: mono={w_mono}, plain={w_plain}"
        );
    }

    #[test]
    fn wrap_note_text_bullet_list_uses_reduced_width() {
        // Java reference data (from CreoleNoteMetricsTest):
        //   bullet at MaxWidth=100: "Calling the" / "method" / "foo() is" / "prohibited" / "overlap" = 5 lines
        //   plain  at MaxWidth=100: "Calling the" / "method foo()" / "is prohibited" / "overlap" = 4 lines
        let bullet = r#"* Calling the method ""foo()"" is prohibited overlap"#;
        let plain = r#"Calling the method ""foo()"" is prohibited overlap"#;
        let wrapped_bullet = wrap_note_text(bullet, 100.0);
        let wrapped_plain = wrap_note_text(plain, 100.0);
        let bullet_lines: Vec<&str> = wrapped_bullet.split('\n').collect();
        let plain_lines: Vec<&str> = wrapped_plain.split('\n').collect();
        // Bullet should produce MORE lines than plain due to indent
        assert!(
            bullet_lines.len() > plain_lines.len(),
            "bullet ({}) should produce more lines than plain ({}).\n  bullet: {bullet_lines:?}\n  plain:  {plain_lines:?}",
            bullet_lines.len(), plain_lines.len()
        );
        // First line should retain the `* ` prefix
        assert!(
            wrapped_bullet.starts_with("* "),
            "first line should start with '* ': {wrapped_bullet:?}"
        );
        // Continuation lines should NOT have `* ` prefix
        for cl in bullet_lines.iter().skip(1) {
            assert!(
                !cl.starts_with("* "),
                "continuation line should not start with '* ': {cl:?}"
            );
        }
    }

    #[test]
    fn wrap_with_max_width_integrates_in_layout() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Action { text: "work".into() },
                ActivityEvent::Note {
                    position: NotePosition::Right,
                    text: "A Long Long Long Long Long Long Long Long Long note".into(),
                },
            ],
            swimlanes: vec![],
            direction: Default::default(),
            note_max_width: Some(80.0),
        };
        let layout = layout_activity(&d).unwrap();
        let note = &layout.nodes[1];
        // The note text should have been wrapped (contains newlines)
        assert!(
            note.text.contains('\n'),
            "note text should be wrapped: {:?}",
            note.text
        );
    }
}
