//! Activity diagram layout engine.
//!
//! Converts an `ActivityDiagram` (list of events + optional swimlanes) into a
//! fully positioned `ActivityLayout` ready for SVG rendering.  The algorithm is
//! a single top-to-bottom pass with a y-cursor, similar to how the sequence
//! diagram layout works with column-based placement.

use crate::font_metrics;
use crate::layout::graphviz::{
    layout_with_svek, transform_path_d, LayoutEdge, LayoutGraph, LayoutNode, RankDir,
};
use crate::model::activity::{
    ActivityDiagram, ActivityEvent, NotePosition, OldActivityGraph, OldActivityNodeKind,
};
use crate::render::svg_richtext::{
    creole_line_height, creole_plain_text, creole_text_width, measure_creole_display_lines,
};
use crate::Result;
use std::collections::HashMap;

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
    pub old_style_graphviz: bool,
    pub old_node_meta: Vec<Option<ActivityGraphvizNodeMeta>>,
    pub old_edge_meta: Vec<Option<ActivityGraphvizEdgeMeta>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityNoteModeLayout {
    Grouped,
    Single,
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
    SyncBar,
    Note {
        position: NotePositionLayout,
        mode: ActivityNoteModeLayout,
    },
    FloatingNote {
        position: NotePositionLayout,
        mode: ActivityNoteModeLayout,
    },
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

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityGraphvizNodeMeta {
    pub id: String,
    pub uid: String,
    pub qualified_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityGraphvizEdgeMeta {
    pub uid: String,
    pub from_id: String,
    pub to_id: String,
    pub raw_path_d: Option<String>,
    pub arrow_polygon_points: Option<Vec<(f64, f64)>>,
    pub label_xy: Option<(f64, f64)>,
    pub head_label: Option<String>,
    pub head_label_xy: Option<(f64, f64)>,
}

/// A single swimlane column.
#[derive(Debug, Clone, PartialEq)]
pub struct SwimlaneLayout {
    pub name: String,
    pub x: f64,
    pub width: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ActivityTableKind {
    SingleColumn { rows: Vec<String> },
    MultiColumn,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
const PADDING: f64 = 10.0;
/// Gap between consecutive flow nodes (matches Java PlantUML visual output).
const NODE_SPACING: f64 = 20.0;
/// Gap for old-style activity diagrams (emulates DOT ranksep ≈ 40px).
const OLD_STYLE_NODE_SPACING: f64 = 29.1;
/// Java FtileCircleStart: SIZE = 20, so radius = 10.
const START_RADIUS: f64 = 10.0;
/// Java FtileCircleStop: SIZE = 22, so radius = 11.
const STOP_RADIUS: f64 = 11.0;
const DIAMOND_SIZE: f64 = 20.0;
const FORK_BAR_HEIGHT: f64 = 6.0;
const FORK_BAR_WIDTH: f64 = 80.0;
/// Java sync bar height (old-style activity `===NAME===`).
const SYNC_BAR_HEIGHT: f64 = 8.0;
const NOTE_FONT_SIZE: f64 = 13.0;
const NOTE_MARGIN_X1: f64 = 6.0;
const NOTE_MARGIN_X2: f64 = 15.0;
const NOTE_MARGIN_Y: f64 = 5.0;
/// Java activity notes leave a 10px visible gap between the flow tile and the
/// note body. Wider spacing is handled separately in the lane composite-width
/// calculation via note margins, not by the placement gap itself.
const NOTE_OFFSET: f64 = 10.0;
const SWIMLANE_MIN_WIDTH: f64 = 80.0;
const TOP_MARGIN: f64 = 11.0;
const BOTTOM_MARGIN: f64 = 7.0;
const SWIMLANE_HEADER_FONT_SIZE: f64 = 18.0;
/// Java activity cross-swimlane connections keep a short fixed vertical stub
/// before the horizontal transfer instead of routing at the arithmetic midline.
const CROSS_LANE_VERTICAL_STUB: f64 = 5.0;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

/// Java creole table cell padding (from skinParam.getPadding(), default 2).
/// Applied as top+bottom padding on SheetBlock1 wrapping each table cell.
pub(crate) const TABLE_CELL_PADDING: f64 = 2.0;

pub(crate) fn classify_activity_table_lines(lines: &[&str]) -> Option<ActivityTableKind> {
    let mut saw_table = false;
    let mut saw_multi_column = false;
    let mut saw_nonempty_non_table = false;
    let mut single_column_rows = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !(trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2) {
            saw_nonempty_non_table = true;
            continue;
        }

        let inner = &trimmed[1..trimmed.len() - 1];
        let cell_count = inner.split('|').count();
        saw_table = true;

        if cell_count >= 2 {
            saw_multi_column = true;
        } else {
            single_column_rows.push(inner.trim().to_string());
        }
    }

    if !saw_table {
        return None;
    }
    if saw_multi_column {
        return Some(ActivityTableKind::MultiColumn);
    }
    if saw_nonempty_non_table {
        return None;
    }
    Some(ActivityTableKind::SingleColumn {
        rows: single_column_rows,
    })
}

/// Estimate the bounding-box size of an action box.
/// Uses actual font metrics for precise sizing to match Java PlantUML.
/// Detects creole tables (`|...|` rows) and adds cell padding.
/// Detects inline sprite references (`<$name>`) and uses sprite viewBox
/// dimensions (scaled by `fontSize / (fontSize + 1)`) for sizing.
fn estimate_text_size(text: &str) -> (f64, f64) {
    // Java: Display.create() does NOT trim lines; leading/trailing spaces
    // are preserved and measured for width (AtomText includes spaces).
    let lines: Vec<&str> = text.split('\n').collect();
    match classify_activity_table_lines(&lines) {
        Some(ActivityTableKind::MultiColumn) => {
            let display_lines: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();
            let (content_width, content_height) = measure_creole_display_lines(
                &display_lines,
                "SansSerif",
                FONT_SIZE,
                false,
                false,
                false,
            );
            let width = content_width + 2.0 * PADDING;
            let height = content_height + 2.0 * PADDING;
            log::debug!(
                "estimate_text_size(table) -> {}x{} ({} lines)",
                width,
                height,
                lines.len()
            );
            return (width, height);
        }
        Some(ActivityTableKind::SingleColumn { rows }) => {
            let content_width = rows.iter().fold(0.0_f64, |acc, row| {
                acc.max(creole_text_width(row, "SansSerif", FONT_SIZE, false, false))
            });
            let content_height = rows
                .iter()
                .map(|row| creole_line_height(row, "SansSerif", FONT_SIZE))
                .sum::<f64>()
                + 2.0 * TABLE_CELL_PADDING;
            let width = content_width + 2.0 * PADDING;
            let height = content_height + 2.0 * PADDING;
            log::debug!(
                "estimate_text_size(single-col-table) -> {}x{} ({} rows)",
                width,
                height,
                rows.len()
            );
            return (width, height);
        }
        None => {}
    }

    // Java AtomImgSvg: sprite visual size = viewBox × fontSize / (fontSize + 1).
    let sprite_scale = FONT_SIZE / (FONT_SIZE + 1.0);
    let lh = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);

    let mut max_line_width = 0.0_f64;
    let mut total_content_height = 0.0_f64;

    for l in &lines {
        let trimmed = l.trim();
        // Check for sprite-only line: `<$name>`
        if let Some(sprite_dim) = sprite_line_dimensions(trimmed, sprite_scale) {
            max_line_width = max_line_width.max(sprite_dim.0);
            total_content_height += sprite_dim.1;
        } else {
            let w = font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false);
            max_line_width = max_line_width.max(w);
            total_content_height += lh;
        }
    }

    let width = max_line_width + 2.0 * PADDING;
    let height = total_content_height + 2.0 * PADDING;
    log::debug!(
        "estimate_text_size -> {}x{} ({} lines)",
        width,
        height,
        lines.len()
    );
    (width, height)
}

/// If `line` is a sprite-only reference (e.g. `<$name>`), return its visual
/// (width, height) after scaling.  Returns `None` for normal text lines.
fn sprite_line_dimensions(line: &str, scale: f64) -> Option<(f64, f64)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("<$") || !trimmed.ends_with('>') {
        return None;
    }
    let inner = &trimmed[2..trimmed.len() - 1];
    let name = inner.split(',').next().unwrap_or(inner).trim();
    if name.is_empty() {
        return None;
    }
    let svg = crate::render::svg_richtext::get_sprite_svg(name)?;
    let (vb_w, vb_h) = parse_sprite_viewbox(&svg);
    Some((vb_w * scale, vb_h * scale))
}

/// Parse viewBox from SVG content to get (width, height).
fn parse_sprite_viewbox(svg: &str) -> (f64, f64) {
    if let Some(vb_start) = svg.find("viewBox=\"") {
        let rest = &svg[vb_start + 9..];
        if let Some(vb_end) = rest.find('"') {
            let parts: Vec<&str> = rest[..vb_end].split_whitespace().collect();
            if parts.len() == 4 {
                return (
                    parts[2].parse().unwrap_or(100.0),
                    parts[3].parse().unwrap_or(50.0),
                );
            }
        }
    }
    (100.0, 50.0)
}

/// Height of a `====` / `----` horizontal separator in a note (Java: 10.0).
/// Height of a `====` / `----` horizontal separator in a note (Java: 10.0).
pub const NOTE_SEPARATOR_HEIGHT: f64 = 10.0;

/// Estimate the size of a note, using note font size.
///
/// Java height model (FloatingNote -> SheetBlock1/2 -> Opale):
///   height = text_block_height + 2 * marginY
/// where text block height is the sum of stripe heights. For plain text lines
/// that is one `line_height` per line; `====`/`----` separators contribute the
/// `CreoleHorizontalLine` height directly.
fn estimate_note_size(text: &str) -> (f64, f64) {
    use crate::render::svg_richtext::creole_text_width;

    let note_lh = font_metrics::line_height("SansSerif", NOTE_FONT_SIZE, false, false);
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
    let width = max_line_width + NOTE_MARGIN_X1 + NOTE_MARGIN_X2;
    let text_height = n_text as f64 * note_lh + sep_height;
    let height = text_height + 2.0 * NOTE_MARGIN_Y;
    log::debug!(
        "estimate_note_size: {:.4}x{:.4} ({} text, max_lw={:.4})",
        width,
        height,
        n_text,
        max_line_width
    );
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
        let mut carry_prefix = String::new();
        let mut is_first = true;
        for word in &words {
            if current_line.is_empty() {
                current_line = format!("{carry_prefix}{word}");
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
                carry_prefix = collect_unclosed_creole_prefix(
                    result_lines.last().map(String::as_str).unwrap_or(""),
                );
                current_line = format!("{carry_prefix}{word}");
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

fn collect_unclosed_creole_prefix(line: &str) -> String {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum TagKind {
        Bold,
        Italic,
        Underline,
        Strike,
        Back,
        Font,
        Color,
        Size,
    }

    fn starts_with_ci(haystack: &str, needle: &str) -> bool {
        haystack.len() >= needle.len() && haystack[..needle.len()].eq_ignore_ascii_case(needle)
    }

    let mut stack: Vec<(TagKind, String)> = Vec::new();
    let mut i = 0usize;
    while i < line.len() {
        let rest = &line[i..];
        let mut matched = false;
        for (open, close, kind) in [
            ("<b>", "</b>", TagKind::Bold),
            ("<i>", "</i>", TagKind::Italic),
            ("<u>", "</u>", TagKind::Underline),
            ("<s>", "</s>", TagKind::Strike),
        ] {
            if starts_with_ci(rest, open) {
                stack.push((kind, open.to_string()));
                i += open.len();
                matched = true;
                break;
            }
            if starts_with_ci(rest, close) {
                if let Some(pos) = stack.iter().rposition(|(k, _)| *k == kind) {
                    stack.remove(pos);
                }
                i += close.len();
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        for (prefix, close, kind) in [
            ("<back:", "</back>", TagKind::Back),
            ("<font:", "</font>", TagKind::Font),
            ("<color:", "</color>", TagKind::Color),
            ("<size:", "</size>", TagKind::Size),
        ] {
            if starts_with_ci(rest, prefix) {
                if let Some(end) = rest.find('>') {
                    stack.push((kind, rest[..=end].to_string()));
                    i += end + 1;
                    matched = true;
                    break;
                }
            }
            if starts_with_ci(rest, close) {
                if let Some(pos) = stack.iter().rposition(|(k, _)| *k == kind) {
                    stack.remove(pos);
                }
                i += close.len();
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        i += rest.chars().next().map(char::len_utf8).unwrap_or(1);
    }

    stack.into_iter().map(|(_, open)| open).collect()
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
        let title_width =
            font_metrics::text_width(name, "SansSerif", SWIMLANE_HEADER_FONT_SIZE, false, false);
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
    if let Some(old_graph) = diagram.old_graph.as_ref() {
        return layout_old_style_activity_graph(diagram, old_graph);
    }

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
        // Java: content_start = header_top + titles_height + 5.0
        // header_top = 2019 font units at header font size (DejaVu Sans).
        // This value (=ascender(1901) + 118) comes from Java's global MinMax
        // y-offset in the rendering framework.
        let header_top = 2019.0 / 2048.0 * SWIMLANE_HEADER_FONT_SIZE;
        header_top + (ha + hd) + 5.0
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
    let node_gap = if diagram.is_old_style { OLD_STYLE_NODE_SPACING } else { NODE_SPACING };
    let mut last_flow_node_idx: Option<usize> = None;
    // --- Old-style sync bar deferred placement ---
    // Pre-scan: find the LAST event index that references each sync bar name
    // (either SyncBar or GotoSyncBar). The bar is placed when we reach that event.
    let mut sync_bar_last_ref: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    // Count incoming references (GotoSyncBar) per sync bar name.
    // Also find the last incoming-reference event index for deferred placement.
    let mut sync_bar_goto_count: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    if diagram.is_old_style {
        for (ev_idx, ev) in diagram.events.iter().enumerate() {
            match ev {
                ActivityEvent::GotoSyncBar(name) => {
                    *sync_bar_goto_count.entry(name.clone()).or_insert(0) += 1;
                    sync_bar_last_ref.insert(name.clone(), ev_idx);
                }
                ActivityEvent::SyncBar(name) => {
                    // Include SyncBar in last_ref tracking only if there are
                    // also GotoSyncBar references — this is updated by
                    // GotoSyncBar above to the LAST incoming event.
                    // If no GotoSyncBar exists, the bar is placed immediately.
                    sync_bar_last_ref.entry(name.clone()).or_insert(ev_idx);
                }
                _ => {}
            }
        }
    }
    // For old-style diagrams, find the LAST Stop event index so intermediate
    // stops can be skipped (Java shares a single final stop node in DOT layout).
    let last_stop_idx: Option<usize> = if diagram.is_old_style {
        diagram.events.iter().enumerate()
            .filter(|(_, e)| matches!(e, ActivityEvent::Stop))
            .map(|(i, _)| i)
            .last()
    } else {
        None
    };

    // Deferred sync bar info: name → (pending, max_y_of_incoming_branches)
    let mut deferred_sync_bars: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();
    // Track placed sync bar y positions: name → y_below_bar
    let mut placed_sync_bars: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

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
                y_cursor += diameter + node_gap;
            }

            // ---- Stop circle (Java FtileCircleStop: SIZE=22) ------------------
            ActivityEvent::Stop => {
                let ev_idx = diagram.events.iter().position(|e| std::ptr::eq(e, event)).unwrap_or(0);
                let is_intermediate = last_stop_idx.map_or(false, |last| ev_idx < last);
                if diagram.is_old_style && is_intermediate {
                    // Old-style: intermediate stops share the final stop node.
                    // Skip placing a visual node here.
                    log::debug!("  skipping intermediate Stop (old-style, ev_idx={ev_idx})");
                } else {
                    let diameter = 2.0 * STOP_RADIUS;
                    let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                    let x = cx - STOP_RADIUS;
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
                    y_cursor += diameter + node_gap;
                }
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
                y_cursor += h + node_gap;
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
                y_cursor += h + node_gap;
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
                y_cursor += h + node_gap;
            }

            ActivityEvent::Else { label } => {
                if diagram.is_old_style {
                    log::debug!("  skipping Else diamond (old-style)");
                } else {
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
                    y_cursor += h + node_gap;
                }
            }

            ActivityEvent::EndIf => {
                if diagram.is_old_style {
                    log::debug!("  skipping EndIf diamond (old-style)");
                } else {
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
                    y_cursor += h + node_gap;
                }
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
                y_cursor += h + node_gap;
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
                y_cursor += h + node_gap;
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
                y_cursor += h + node_gap;
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
                y_cursor += h + node_gap;
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
                y_cursor += h + node_gap;
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
                        mode: ActivityNoteModeLayout::Grouped,
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
                let note_bottom = ny + nh + node_gap;
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
                let (nx, ny) = if let Some(prev_idx) = last_flow_node_idx {
                    let prev_x = nodes[prev_idx].x;
                    let prev_y = nodes[prev_idx].y;
                    let prev_w = nodes[prev_idx].width;
                    let prev_h = nodes[prev_idx].height;
                    let x = match pos_layout {
                        NotePositionLayout::Right => prev_x + prev_w + NOTE_OFFSET,
                        NotePositionLayout::Left => prev_x - NOTE_OFFSET - nw,
                    };
                    // Java floating notes are visually attached to the previous
                    // flow tile, so keep their midpoints aligned with the tile.
                    let y = prev_y + (prev_h - nh) / 2.0;
                    (x, y)
                } else {
                    let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                    let x = match pos_layout {
                        NotePositionLayout::Right => cx + NOTE_OFFSET,
                        NotePositionLayout::Left => cx - NOTE_OFFSET - nw,
                    };
                    (x, y_cursor)
                };

                log::debug!(
                    "  node[{node_index}] FloatingNote({pos_layout:?}) @ ({nx:.1}, {ny:.1})"
                );
                nodes.push(ActivityNodeLayout {
                    index: node_index,
                    kind: ActivityNodeKindLayout::FloatingNote {
                        position: pos_layout,
                        mode: ActivityNoteModeLayout::Grouped,
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
                y_cursor += size + node_gap;
            }

            // ---- Sync bar (old-style ===NAME===) ----------------------------
            ActivityEvent::SyncBar(name) => {
                let ev_idx = diagram.events.iter().position(|e| std::ptr::eq(e, event)).unwrap_or(0);
                let has_gotos = sync_bar_goto_count.get(name).copied().unwrap_or(0) > 0;
                let is_last_ref = sync_bar_last_ref.get(name).copied() == Some(ev_idx);
                if diagram.is_old_style && has_gotos && !is_last_ref {
                    // Defer placement: just record the current y_cursor as a
                    // candidate position for this bar.
                    let entry = deferred_sync_bars.entry(name.clone()).or_insert(0.0_f64);
                    *entry = entry.max(y_cursor);
                    log::debug!("  SyncBar({name}) deferred, max_y={:.1}", *entry);
                } else {
                    // Place immediately (either new-style or this is the last ref)
                    let bar_y = if diagram.is_old_style {
                        // Use the max y from all deferred references
                        let deferred_y = deferred_sync_bars.remove(name).unwrap_or(0.0);
                        deferred_y.max(y_cursor)
                    } else {
                        y_cursor
                    };
                    let w = FORK_BAR_WIDTH;
                    let h = SYNC_BAR_HEIGHT;
                    let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                    let x = cx - w / 2.0;
                    log::debug!("  node[{node_index}] SyncBar({name}) @ ({x:.1}, {bar_y:.1})");
                    nodes.push(ActivityNodeLayout {
                        index: node_index,
                        kind: ActivityNodeKindLayout::SyncBar,
                        x,
                        y: bar_y,
                        width: w,
                        height: h,
                        text: String::new(),
                    });
                    placed_sync_bars.insert(name.clone(), bar_y + h + node_gap);
                    last_flow_node_idx = Some(node_index);
                    node_index += 1;
                    y_cursor = bar_y + h + node_gap;
                }
            }

            // ---- Goto sync bar (old-style convergence) ----------------------
            ActivityEvent::GotoSyncBar(name) => {
                let ev_idx = diagram.events.iter().position(|e| std::ptr::eq(e, event)).unwrap_or(0);
                let is_last_ref = sync_bar_last_ref.get(name).copied() == Some(ev_idx);
                // Update the deferred max-y for this bar
                let entry = deferred_sync_bars.entry(name.clone()).or_insert(0.0_f64);
                *entry = entry.max(y_cursor);
                log::debug!("  GotoSyncBar({name}), max_y={:.1}, is_last={}", *entry, is_last_ref);
                if is_last_ref {
                    // This is the last reference — place the bar NOW
                    let bar_y = *entry;
                    deferred_sync_bars.remove(name);
                    let w = FORK_BAR_WIDTH;
                    let h = SYNC_BAR_HEIGHT;
                    let cx = swimlane_center_x(&swimlane_layouts, current_lane_idx);
                    let x = cx - w / 2.0;
                    log::debug!("  node[{node_index}] SyncBar({name}) placed @ ({x:.1}, {bar_y:.1})");
                    nodes.push(ActivityNodeLayout {
                        index: node_index,
                        kind: ActivityNodeKindLayout::SyncBar,
                        x,
                        y: bar_y,
                        width: w,
                        height: h,
                        text: String::new(),
                    });
                    placed_sync_bars.insert(name.clone(), bar_y + h + node_gap);
                    last_flow_node_idx = Some(node_index);
                    node_index += 1;
                    y_cursor = bar_y + h + node_gap;
                }
            }

            // ---- Resume from sync bar (old-style source) --------------------
            ActivityEvent::ResumeFromSyncBar(name) => {
                // Outgoing reference: ===Y1=== --> target.
                // In a sequential layout we cannot go backwards, so we only
                // advance forward (y_cursor keeps its current value, or moves
                // forward to below the bar if the bar is ahead).
                if let Some(bar_y_below) = placed_sync_bars.get(name) {
                    if *bar_y_below > y_cursor {
                        log::debug!("  ResumeFromSyncBar({name}) — y_cursor {y_cursor:.1} -> {bar_y_below:.1}");
                        y_cursor = *bar_y_below;
                    } else {
                        log::debug!("  ResumeFromSyncBar({name}) — bar below at {bar_y_below:.1}, keeping y_cursor at {y_cursor:.1}");
                    }
                } else {
                    log::debug!("  ResumeFromSyncBar({name}) — bar not yet placed, keeping y_cursor at {y_cursor:.1}");
                }
            }
        }
    }

    // --- Pass 2b: centering for non-swimlane diagrams ----------------------
    if swimlane_layouts.is_empty() && !nodes.is_empty() {
        let max_half_w = nodes
            .iter()
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

    assign_note_modes(&mut nodes);

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
                ActivityEvent::Start
                | ActivityEvent::Stop
                | ActivityEvent::Action { .. }
                | ActivityEvent::If { .. }
                | ActivityEvent::ElseIf { .. }
                | ActivityEvent::Else { .. }
                | ActivityEvent::EndIf
                | ActivityEvent::While { .. }
                | ActivityEvent::EndWhile { .. }
                | ActivityEvent::Repeat
                | ActivityEvent::RepeatWhile { .. }
                | ActivityEvent::Fork
                | ActivityEvent::ForkAgain
                | ActivityEvent::EndFork
                | ActivityEvent::Note { .. }
                | ActivityEvent::FloatingNote { .. }
                | ActivityEvent::Detach
                | ActivityEvent::SyncBar(_) => {
                    node_lane.push(cur_lane);
                }
                ActivityEvent::GotoSyncBar(_) | ActivityEvent::ResumeFromSyncBar(_) => {}
            }
        }

        // 2) Compute content width per lane.
        //    Java FtileWithNotes: width = tile.w + left_notes.w + right_notes.w.
        //    We simulate this by finding each flow node's composite width
        //    (including adjacent notes) and tracking the max composite.
        let n_lanes = swimlane_layouts.len();
        let mut lane_max_composite_w = vec![0.0_f64; n_lanes];
        let mut lane_max_composite_single = vec![false; n_lanes];
        let mut lane_min_x = vec![f64::MAX; n_lanes];
        let mut lane_max_x = vec![f64::MIN; n_lanes];

        // For each flow node, find adjacent notes and compute composite width
        for (ni, node) in nodes.iter().enumerate() {
            let li = if ni < node_lane.len() {
                node_lane[ni]
            } else {
                0
            };
            let (left, right) = limitfinder_x_bounds(node);
            if left < lane_min_x[li] {
                lane_min_x[li] = left;
            }
            if right > lane_max_x[li] {
                lane_max_x[li] = right;
            }

            if is_flow_node(&node.kind) {
                // Find adjacent notes (immediately following this flow node)
                let mut left_note_w = 0.0_f64;
                let mut right_note_w = 0.0_f64;
                let mut note_count = 0usize;
                for j in (ni + 1)..nodes.len() {
                    match &nodes[j].kind {
                        ActivityNodeKindLayout::Note { position, .. }
                        | ActivityNodeKindLayout::FloatingNote { position, .. } => {
                            note_count += 1;
                            match position {
                                NotePositionLayout::Left => left_note_w += nodes[j].width,
                                NotePositionLayout::Right => right_note_w += nodes[j].width,
                            }
                        }
                        _ => break, // next flow node — stop looking
                    }
                }
                // Java FtileWithNotes: each note Opale is wrapped with
                // TextBlockUtils.withMargin(opale, 10, 10) → +20 per note side.
                let note_margin = 20.0; // Java: withMargin(opale, 10, 10)
                let left_total = if left_note_w > 0.0 {
                    left_note_w + note_margin
                } else {
                    0.0
                };
                let right_total = if right_note_w > 0.0 {
                    right_note_w + note_margin
                } else {
                    0.0
                };
                let composite_w = node.width + left_total + right_total;
                if composite_w > lane_max_composite_w[li] {
                    lane_max_composite_w[li] = composite_w;
                    lane_max_composite_single[li] = note_count == 1;
                }
            }
        }

        // 3) Expand each lane; Java LaneDivider: edge=5, between=5..N depending on title overflow
        // Left divider = halfMissing(0)(=5) + halfMissing(1)(=5 or more)
        let half_missing_edge = LANE_DIVIDER_HALF;
        let header_widths: Vec<f64> = diagram
            .swimlanes
            .iter()
            .map(|name| {
                font_metrics::text_width(name, "SansSerif", SWIMLANE_HEADER_FONT_SIZE, false, false)
            })
            .collect();

        // First pass: determine final lane widths (max of header and content)
        let mut lane_widths: Vec<f64> = Vec::with_capacity(n_lanes);
        for i in 0..n_lanes {
            // Use the max composite width (Java FtileWithNotes model).
            // Java LimitFinder tracks 1px wider than FtileWithNoteOpale.
            // calculateDimension for single-side note lanes (from Opale
            // stencil rendering offset in SheetBlock). FtileWithNotes
            // (both-side notes) doesn't have this offset.
            let stencil_correction = if lane_max_composite_w[i] > 0.0 && lane_max_composite_single[i] {
                1.0
            } else {
                0.0
            };
            let content_width = if lane_max_composite_w[i] > 0.0 {
                lane_max_composite_w[i] + stencil_correction
            } else if lane_max_x[i] > lane_min_x[i] {
                lane_max_x[i] - lane_min_x[i]
            } else {
                0.0
            };
            let hw = header_widths[i] + 2.0 * LANE_DIVIDER_HALF;
            // Java: lane visual width = actualWidth + dividerWidth.
            // When content > header, add divider to the lane width itself.
            let cw_with_div = if content_width > hw {
                content_width + 2.0 * LANE_DIVIDER_HALF
            } else {
                content_width
            };
            lane_widths.push(cw_with_div.max(hw));
        }

        // Java getHalfMissingSpace: if title > actualWidth, expand divider.
        // Since lane_widths already includes max(content, header+pad), title
        // overflow is already absorbed. half_missing returns the base 5px.
        let half_missing = |_lane_idx: usize| -> f64 { LANE_DIVIDER_HALF };

        // Java: left lane line consistently at x ≈ 20 (divider(10) + centering offset).
        // This comes from LaneDivider width + content minX compensation.
        // We approximate with edge(5) + halfMissing + content centering offset.
        let left_divider = half_missing_edge + half_missing(0);
        // Java: internal lane lines start at x≈5, then the entire diagram gets
        // a global +15 offset from the SVG rendering framework's MinMax margin.
        // We apply this combined offset directly: first lane starts at x ≈ 20.
        // Java internal lane lines start at x≈5; SVG renders them at x≈20
        // due to framework-level MinMax offset (~15px).  We apply the combined
        // left_divider + framework offset directly.
        let global_margin = 5.0;
        let mut x = left_divider + global_margin;
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
            // Java: xpos += actualWidth + dividerWidth.
            // When lane width is header-driven (hw includes divider padding),
            // the divider is already absorbed. When content-driven (content > hw),
            // add the divider width explicitly.
            // Divider is already included in lane_widths for content-driven lanes.
            x += needed;
        }

        // 4) Re-normalize note groups around their flow node.
        // Java uses two distinct composite geometries:
        // - `FtileWithNotes` when notes exist on both sides: each side reserves
        //   `note_width + 20`, but the visible gap to the action box is 10.
        // - `FtileWithNoteOpale` for a single-side note: the side reserves
        //   `note_width + 19`, matching the 1px stencil correction seen in
        //   Java's LimitFinder path for one-sided note tiles.
        let mut i = 0usize;
        while i < nodes.len() {
            if !is_flow_node(&nodes[i].kind) {
                i += 1;
                continue;
            }

            let mut left_indices = Vec::new();
            let mut right_indices = Vec::new();
            let mut j = i + 1;
            while j < nodes.len() {
                match &nodes[j].kind {
                    ActivityNodeKindLayout::Note { position, .. }
                    | ActivityNodeKindLayout::FloatingNote { position, .. } => match position {
                        NotePositionLayout::Left => left_indices.push(j),
                        NotePositionLayout::Right => right_indices.push(j),
                    },
                    _ => break,
                }
                j += 1;
            }

            if left_indices.is_empty() && right_indices.is_empty() {
                i = j;
                continue;
            }

            let has_left = !left_indices.is_empty();
            let has_right = !right_indices.is_empty();
            let total_notes = left_indices.len() + right_indices.len();
            let single_group = total_notes == 1;
            let left_max_w = left_indices
                .iter()
                .map(|&idx| nodes[idx].width)
                .fold(0.0_f64, f64::max);
            let right_max_w = right_indices
                .iter()
                .map(|&idx| nodes[idx].width)
                .fold(0.0_f64, f64::max);
            let left_band = if has_left {
                left_max_w + if single_group { 19.0 } else { 20.0 }
            } else {
                0.0
            };
            let right_band = if has_right {
                right_max_w + if single_group { 19.0 } else { 20.0 }
            } else {
                0.0
            };

            let mut group_min_x = nodes[i].x;
            let mut group_max_x = nodes[i].x + nodes[i].width;
            for &idx in left_indices.iter().chain(right_indices.iter()) {
                group_min_x = group_min_x.min(nodes[idx].x);
                group_max_x = group_max_x.max(nodes[idx].x + nodes[idx].width);
            }
            let group_center = (group_min_x + group_max_x) / 2.0;
            let group_width = left_band + nodes[i].width + right_band;
            let group_left = group_center - group_width / 2.0;

            if has_left {
                let left_x = if single_group { group_left } else { group_left + 10.0 };
                for &idx in &left_indices {
                    nodes[idx].x = left_x;
                }
            }

            nodes[i].x = group_left + left_band;

            if has_right {
                let right_gap = if single_group { 20.0 } else { 10.0 };
                let right_x = nodes[i].x + nodes[i].width + right_gap;
                for &idx in &right_indices {
                    nodes[idx].x = right_x;
                }
            }

            i = j;
        }

        // 5) Subsequent flow groups in the same swimlane should keep following
        // the previous flow column. Java activity tiles do not snap back to
        // the swimlane center after a one-sided note shifts the column.
        align_flow_groups_to_lane_columns(&mut nodes, &node_lane);
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
        old_style_graphviz: false,
        old_node_meta: Vec::new(),
        old_edge_meta: Vec::new(),
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

fn assign_note_modes(nodes: &mut [ActivityNodeLayout]) {
    let mut i = 0usize;
    while i < nodes.len() {
        if !is_flow_node(&nodes[i].kind) {
            i += 1;
            continue;
        }

        let mut note_indices = Vec::new();
        let mut j = i + 1;
        while j < nodes.len() {
            match nodes[j].kind {
                ActivityNodeKindLayout::Note { .. } | ActivityNodeKindLayout::FloatingNote { .. } => {
                    note_indices.push(j);
                }
                _ => break,
            }
            j += 1;
        }

        let mode = if note_indices.len() == 1 {
            ActivityNoteModeLayout::Single
        } else {
            ActivityNoteModeLayout::Grouped
        };
        for idx in note_indices {
            match &mut nodes[idx].kind {
                ActivityNodeKindLayout::Note { mode: note_mode, .. }
                | ActivityNodeKindLayout::FloatingNote {
                    mode: note_mode, ..
                } => *note_mode = mode,
                _ => {}
            }
        }

        i = j;
    }
}

fn align_flow_groups_to_lane_columns(nodes: &mut [ActivityNodeLayout], node_lane: &[usize]) {
    let lane_count = node_lane.iter().copied().max().map(|idx| idx + 1).unwrap_or(0);
    let mut lane_flow_centers: Vec<Option<f64>> = vec![None; lane_count];
    let mut i = 0usize;

    while i < nodes.len() {
        if !is_flow_node(&nodes[i].kind) {
            i += 1;
            continue;
        }

        let mut j = i + 1;
        while j < nodes.len() {
            match nodes[j].kind {
                ActivityNodeKindLayout::Note { .. } | ActivityNodeKindLayout::FloatingNote { .. } => {
                    j += 1;
                }
                _ => break,
            }
        }

        let lane_idx = node_lane.get(i).copied().unwrap_or(0);
        if let Some(prev_center) = lane_flow_centers.get(lane_idx).copied().flatten() {
            let desired_x = prev_center - nodes[i].width / 2.0;
            let dx = desired_x - nodes[i].x;
            if dx.abs() > 0.01 {
                for node in &mut nodes[i..j] {
                    node.x += dx;
                }
            }
        }

        lane_flow_centers[lane_idx] = Some(nodes[i].x + nodes[i].width / 2.0);
        i = j;
    }
}

/// Java `LimitFinder` uses shape-specific bounds when computing swimlane
/// `MinMax`.  Activity swimlane centering must follow those bounds rather than
/// the plain layout box, otherwise simple action lanes end up 1px too far left.
fn limitfinder_x_bounds(node: &ActivityNodeLayout) -> (f64, f64) {
    match node.kind {
        ActivityNodeKindLayout::Action
        | ActivityNodeKindLayout::ForkBar
        | ActivityNodeKindLayout::SyncBar => {
            (node.x - 1.0, node.x + node.width - 1.0)
        }
        ActivityNodeKindLayout::Diamond => (node.x - 10.0, node.x + node.width + 10.0),
        ActivityNodeKindLayout::Start
        | ActivityNodeKindLayout::Stop
        | ActivityNodeKindLayout::End
        | ActivityNodeKindLayout::Detach => (node.x, node.x + node.width - 1.0),
        ActivityNodeKindLayout::Note { position, mode }
        | ActivityNodeKindLayout::FloatingNote { position, mode } => match (position, mode) {
            (NotePositionLayout::Right, ActivityNoteModeLayout::Single) => {
                (node.x, node.x + node.width + 1.0)
            }
            (NotePositionLayout::Left, ActivityNoteModeLayout::Single) => {
                (node.x - 1.0, node.x + node.width)
            }
            _ => (node.x, node.x + node.width),
        },
    }
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
            // Cross-lane: default to a short source stub. When the target flow
            // group has notes protruding above the target action, Java lifts
            // the horizontal crossing to just above that group.
            let target_group_top = flow_group_top(nodes, to_idx);
            let mid_y = if target_group_top + 0.01 < to_top {
                (target_group_top - CROSS_LANE_VERTICAL_STUB).max(from_bottom)
            } else {
                let dy = to_top - from_bottom;
                let stub = CROSS_LANE_VERTICAL_STUB.min(dy.abs()).copysign(dy);
                from_bottom + stub
            };
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

fn flow_group_top(nodes: &[ActivityNodeLayout], flow_idx: usize) -> f64 {
    let mut top = nodes[flow_idx].y;
    let mut j = flow_idx + 1;
    while j < nodes.len() {
        match nodes[j].kind {
            ActivityNodeKindLayout::Note { .. } | ActivityNodeKindLayout::FloatingNote { .. } => {
                top = top.min(nodes[j].y);
            }
            _ => break,
        }
        j += 1;
    }
    top
}

const OLD_ACTIVITY_BRANCH_SIZE: f64 = 24.0;
const OLD_ACTIVITY_EDGE_FONT_SIZE: f64 = 11.0;

fn old_activity_center_label_dimension(text: &str) -> (f64, f64) {
    let line_h = font_metrics::line_height("SansSerif", OLD_ACTIVITY_EDGE_FONT_SIZE, false, false);
    let text_w = font_metrics::text_width(text, "SansSerif", OLD_ACTIVITY_EDGE_FONT_SIZE, false, false);
    (text_w + 2.0, line_h + 2.0)
}

fn old_activity_side_label_dimension(text: &str) -> (f64, f64) {
    let display = if text.is_empty() { " " } else { text };
    let line_h = font_metrics::line_height("SansSerif", OLD_ACTIVITY_EDGE_FONT_SIZE, false, false);
    let text_w =
        font_metrics::text_width(display, "SansSerif", OLD_ACTIVITY_EDGE_FONT_SIZE, false, false);
    (text_w, line_h)
}

fn layout_old_style_activity_graph(
    _diagram: &ActivityDiagram,
    old_graph: &OldActivityGraph,
) -> Result<ActivityLayout> {
    let nodes: Vec<LayoutNode> = old_graph
        .nodes
        .iter()
        .map(|node| {
            let (shape, width, height, text) = match node.kind {
                OldActivityNodeKind::Start => (
                    Some(crate::svek::shape_type::ShapeType::Circle),
                    20.0,
                    20.0,
                    String::new(),
                ),
                OldActivityNodeKind::End => (
                    Some(crate::svek::shape_type::ShapeType::Circle),
                    22.0,
                    22.0,
                    String::new(),
                ),
                OldActivityNodeKind::Action => {
                    let (w, h) = estimate_text_size(&node.text);
                    (
                        Some(crate::svek::shape_type::ShapeType::RoundRectangle),
                        w,
                        h,
                        node.text.clone(),
                    )
                }
                OldActivityNodeKind::Branch => (
                    Some(crate::svek::shape_type::ShapeType::Diamond),
                    OLD_ACTIVITY_BRANCH_SIZE,
                    OLD_ACTIVITY_BRANCH_SIZE,
                    String::new(),
                ),
                OldActivityNodeKind::SyncBar => (
                    Some(crate::svek::shape_type::ShapeType::Rectangle),
                    80.0,
                    8.0,
                    String::new(),
                ),
            };
            LayoutNode {
                id: node.id.clone(),
                label: text,
                width_pt: width,
                height_pt: height,
                shape,
                shield: None,
                entity_position: None,
                max_label_width: None,
                port_label_width: None,
                order: None,
                image_width_pt: None,
                lf_extra_left: 0.0,
                lf_rect_correction: true,
                lf_has_body_separator: false,
                lf_node_polygon: false,
                lf_polygon_hack: false,
                lf_actor_stickman: false,
                hidden: false,
            }
        })
        .collect();

    let edges: Vec<LayoutEdge> = old_graph
        .links
        .iter()
        .map(|link| LayoutEdge {
            from: link.from_id.clone(),
            to: link.to_id.clone(),
            label: link.label.clone(),
            label_dimension: link
                .label
                .as_deref()
                .map(old_activity_center_label_dimension),
            tail_label: None,
            tail_label_boxed: false,
            head_label: link.head_label.clone(),
            head_label_boxed: false,
            tail_decoration: crate::svek::edge::LinkDecoration::None,
            head_decoration: crate::svek::edge::LinkDecoration::None,
            line_style: crate::svek::edge::LinkStyle::Normal,
            minlen: link.length.saturating_sub(1),
            invisible: false,
            no_constraint: false,
            tail_label_dimension: None,
            head_label_dimension: link
                .head_label
                .as_deref()
                .map(old_activity_side_label_dimension),
        })
        .collect();

    let graph = LayoutGraph {
        nodes,
        edges,
        clusters: Vec::new(),
        rankdir: RankDir::TopToBottom,
        is_activity: false,
        ranksep_override: Some(40.0),
        nodesep_override: Some(20.0),
        use_simplier_dot_link_strategy: false,
    };

    let gl = layout_with_svek(&graph)?;
    let edge_offset_x = gl.render_offset.0;
    let edge_offset_y = gl.render_offset.1;

    let node_by_id: std::collections::HashMap<&str, &crate::layout::graphviz::NodeLayout> =
        gl.nodes.iter().map(|node| (node.id.as_str(), node)).collect();
    let mut activity_nodes = Vec::with_capacity(old_graph.nodes.len());
    let mut old_node_meta = Vec::with_capacity(old_graph.nodes.len());
    let mut node_layout_index = HashMap::new();

    for (idx, node) in old_graph.nodes.iter().enumerate() {
        let gv = node_by_id
            .get(node.id.as_str())
            .copied()
            .ok_or_else(|| crate::Error::Layout(format!("missing old-style activity node {}", node.id)))?;
        let kind = match node.kind {
            OldActivityNodeKind::Start => ActivityNodeKindLayout::Start,
            OldActivityNodeKind::End => ActivityNodeKindLayout::Stop,
            OldActivityNodeKind::Action => ActivityNodeKindLayout::Action,
            OldActivityNodeKind::Branch => ActivityNodeKindLayout::Diamond,
            OldActivityNodeKind::SyncBar => ActivityNodeKindLayout::SyncBar,
        };
        activity_nodes.push(ActivityNodeLayout {
            index: idx,
            kind,
            x: gv.min_x + edge_offset_x,
            y: gv.min_y + edge_offset_y,
            width: gv.width,
            height: gv.height,
            text: node.text.clone(),
        });
        old_node_meta.push(Some(ActivityGraphvizNodeMeta {
            id: node.id.clone(),
            uid: node.uid.clone(),
            qualified_name: node.qualified_name.clone(),
        }));
        node_layout_index.insert(node.id.clone(), idx);
    }

    let mut activity_edges = Vec::with_capacity(old_graph.links.len());
    let mut old_edge_meta = Vec::with_capacity(old_graph.links.len());
    for (idx, link) in old_graph.links.iter().enumerate() {
        let gv = gl
            .edges
            .get(idx)
            .ok_or_else(|| crate::Error::Layout(format!("missing old-style activity edge {}", link.uid)))?;
        let from_index = *node_layout_index
            .get(&link.from_id)
            .ok_or_else(|| crate::Error::Layout(format!("missing activity edge source {}", link.from_id)))?;
        let to_index = *node_layout_index
            .get(&link.to_id)
            .ok_or_else(|| crate::Error::Layout(format!("missing activity edge target {}", link.to_id)))?;
        let shifted_points: Vec<(f64, f64)> = gv
            .points
            .iter()
            .map(|&(x, y)| (x + edge_offset_x, y + edge_offset_y))
            .collect();
        let label_xy = gv.label_xy.map(|(x, y)| {
            (
                x + gl.move_delta.0 - gl.normalize_offset.0 + edge_offset_x,
                y + gl.move_delta.1 - gl.normalize_offset.1 + edge_offset_y,
            )
        });
        let head_label_xy = gv.head_label_xy.map(|(x, y)| {
            (
                x + gl.move_delta.0 - gl.normalize_offset.0 + edge_offset_x,
                y + gl.move_delta.1 - gl.normalize_offset.1 + edge_offset_y,
            )
        });
        activity_edges.push(ActivityEdgeLayout {
            from_index,
            to_index,
            label: link.label.clone().unwrap_or_default(),
            points: shifted_points,
        });
        old_edge_meta.push(Some(ActivityGraphvizEdgeMeta {
            uid: link.uid.clone(),
            from_id: link.from_id.clone(),
            to_id: link.to_id.clone(),
            raw_path_d: gv
                .raw_path_d
                .as_ref()
                .map(|raw| transform_path_d(raw, edge_offset_x, edge_offset_y)),
            arrow_polygon_points: gv.arrow_polygon_points.as_ref().map(|pts| {
                pts.iter()
                    .map(|&(x, y)| (x + edge_offset_x, y + edge_offset_y))
                    .collect()
            }),
            label_xy,
            head_label: link.head_label.clone(),
            head_label_xy,
        }));
    }

    Ok(ActivityLayout {
        width: gl.total_width + 12.0,
        height: gl.total_height + 12.0,
        nodes: activity_nodes,
        edges: activity_edges,
        swimlane_layouts: Vec::new(),
        old_style_graphviz: true,
        old_node_meta,
        old_edge_meta,
    })
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
            is_old_style: false,
            old_graph: None,
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

    // 1b. Creole table height includes cell padding (Java +4px) ---------------

    #[test]
    fn creole_table_height_includes_cell_padding() {
        // Java CreoleTableMetricsTest: table row adds 4px total to action height
        // Plain "text": action_h = 33.97 (line_height + 2*PADDING)
        // Table "|text|": action_h = 37.97 (+4 from cell padding 2+2)
        let (_, h_plain) = estimate_text_size("plain text");
        let (_, h_table) = estimate_text_size("|table cell|");
        let diff = h_table - h_plain;
        assert!(
            (diff - 4.0).abs() < 0.1,
            "table should be 4px taller than plain: diff={diff:.1} (table={h_table:.1} plain={h_plain:.1})"
        );
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

    // 2b. Java circle sizes: start=20, stop=22 (FtileCircleStart/Stop) ------

    #[test]
    fn stop_circle_size_matches_java() {
        // Java: FtileCircleStart SIZE=20, FtileCircleStop SIZE=22
        // start diameter=20, stop diameter=22 (outer ring r=11)
        let d = diagram(vec![ActivityEvent::Start, ActivityEvent::Stop]);
        let layout = layout_activity(&d).unwrap();
        let start = &layout.nodes[0];
        let stop = &layout.nodes[1];
        assert!(
            (start.height - 20.0).abs() < 0.1,
            "start height should be 20 (Java FtileCircleStart SIZE=20), got {}",
            start.height
        );
        assert!(
            (stop.height - 22.0).abs() < 0.1,
            "stop height should be 22 (Java FtileCircleStop SIZE=22), got {}",
            stop.height
        );
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
            is_old_style: false,
            old_graph: None,
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
                ActivityEvent::Swimlane {
                    name: "Lane A".into(),
                },
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "task A".into(),
                },
                ActivityEvent::Stop,
                ActivityEvent::Swimlane {
                    name: "Lane B".into(),
                },
                ActivityEvent::Action {
                    text: "task B".into(),
                },
            ],
            swimlanes: vec!["Lane A".into(), "Lane B".into()],
            direction: Default::default(),
            note_max_width: None,
            is_old_style: false,
            old_graph: None,
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
                ActivityEvent::Swimlane {
                    name: "Lane A".into(),
                },
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "action".into(),
                },
                ActivityEvent::Note {
                    position: NotePosition::Right,
                    text: "a short note".into(),
                },
                ActivityEvent::Swimlane {
                    name: "Lane B".into(),
                },
                ActivityEvent::Action {
                    text: "task2".into(),
                },
                ActivityEvent::Stop,
            ],
            swimlanes: vec!["Lane A".into(), "Lane B".into()],
            direction: Default::default(),
            note_max_width: None,
            is_old_style: false,
            old_graph: None,
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
        let note = layout
            .nodes
            .iter()
            .find(|n| matches!(n.kind, ActivityNodeKindLayout::Note { .. }))
            .unwrap();
        let note_right = note.x + note.width;
        let lane_a_right = lane_a.x + lane_a.width;
        assert!(
            note_right <= lane_a_right + 1.0,
            "note right ({:.1}) should be within Lane A right ({:.1})",
            note_right,
            lane_a_right
        );
    }

    #[test]
    fn swimlane_content_shift_uses_limitfinder_rectangle_bounds() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane {
                    name: "Swimlane1".into(),
                },
                ActivityEvent::Start,
                ActivityEvent::Action {
                    text: "Action 1".into(),
                },
                ActivityEvent::Swimlane {
                    name: "Swimlane2".into(),
                },
                ActivityEvent::Action {
                    text: "Action 2".into(),
                },
            ],
            swimlanes: vec!["Swimlane1".into(), "Swimlane2".into()],
            direction: Default::default(),
            note_max_width: None,
            is_old_style: false,
            old_graph: None,
        };
        let layout = layout_activity(&d).unwrap();
        let lane_a = &layout.swimlane_layouts[0];
        let lane_b = &layout.swimlane_layouts[1];
        let action_a = layout
            .nodes
            .iter()
            .find(|n| matches!(n.kind, ActivityNodeKindLayout::Action) && n.text == "Action 1")
            .unwrap();
        let action_b = layout
            .nodes
            .iter()
            .find(|n| matches!(n.kind, ActivityNodeKindLayout::Action) && n.text == "Action 2")
            .unwrap();

        let expected_a = lane_a.x + (lane_a.width - action_a.width) / 2.0 + 1.0;
        let expected_b = lane_b.x + (lane_b.width - action_b.width) / 2.0 + 1.0;
        assert!(
            (action_a.x - expected_a).abs() < 0.01,
            "lane A action.x ({:.4}) should include the LimitFinder rectangle shift ({:.4})",
            action_a.x,
            expected_a
        );
        assert!(
            (action_b.x - expected_b).abs() < 0.01,
            "lane B action.x ({:.4}) should include the LimitFinder rectangle shift ({:.4})",
            action_b.x,
            expected_b
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
                mode: ActivityNoteModeLayout::Single,
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
            action2.y,
            expected_action2_y,
            note.y,
            note.height
        );
    }

    #[test]
    fn floating_note_is_attached_to_previous_flow_node() {
        let d = diagram(vec![
            ActivityEvent::Action {
                text: "work".into(),
            },
            ActivityEvent::FloatingNote {
                position: NotePosition::Left,
                text: "floating".into(),
            },
        ]);
        let layout = layout_activity(&d).unwrap();
        assert_eq!(layout.nodes.len(), 2);
        let action = &layout.nodes[0];
        let note = &layout.nodes[1];
        assert!(
            (note.x - (action.x - NOTE_OFFSET - note.width)).abs() < 0.01,
            "floating note.x ({:.4}) should sit {}px left of the previous flow node ({:.4})",
            note.x,
            NOTE_OFFSET,
            action.x
        );
        assert!(
            (note.y - (action.y + (action.height - note.height) / 2.0)).abs() < 0.01,
            "floating note.y ({:.4}) should be vertically centered on action.y ({:.4})",
            note.y,
            action.y
        );
    }

    #[test]
    fn stop_keeps_previous_flow_column_after_single_note_group() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane {
                    name: "Lane".into(),
                },
                ActivityEvent::Action {
                    text: "work".into(),
                },
                ActivityEvent::Note {
                    position: NotePosition::Right,
                    text: "single note".into(),
                },
                ActivityEvent::Stop,
            ],
            swimlanes: vec!["Lane".into()],
            direction: Default::default(),
            note_max_width: None,
            is_old_style: false,
            old_graph: None,
        };
        let layout = layout_activity(&d).unwrap();
        let action = layout
            .nodes
            .iter()
            .find(|n| matches!(n.kind, ActivityNodeKindLayout::Action))
            .unwrap();
        let stop = layout
            .nodes
            .iter()
            .find(|n| matches!(n.kind, ActivityNodeKindLayout::Stop))
            .unwrap();
        let action_cx = action.x + action.width / 2.0;
        let stop_cx = stop.x + stop.width / 2.0;
        assert!(
            (action_cx - stop_cx).abs() < 0.01,
            "stop center ({stop_cx:.4}) should follow previous flow column ({action_cx:.4})"
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

    #[test]
    fn cross_lane_edge_routes_above_target_note_group() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Swimlane { name: "A".into() },
                ActivityEvent::Action { text: "A1".into() },
                ActivityEvent::Swimlane { name: "B".into() },
                ActivityEvent::Action { text: "B1".into() },
                ActivityEvent::Note {
                    position: NotePosition::Right,
                    text: "line1\nline2\nline3\nline4".into(),
                },
            ],
            swimlanes: vec!["A".into(), "B".into()],
            direction: Default::default(),
            note_max_width: None,
            is_old_style: false,
            old_graph: None,
        };
        let layout = layout_activity(&d).unwrap();
        let cross = layout
            .edges
            .iter()
            .find(|edge| edge.from_index == 0 && edge.to_index == 1)
            .unwrap();
        let note = layout
            .nodes
            .iter()
            .find(|node| matches!(node.kind, ActivityNodeKindLayout::Note { .. }))
            .unwrap();
        let target = layout
            .nodes
            .iter()
            .find(|node| matches!(node.kind, ActivityNodeKindLayout::Action) && node.text == "B1")
            .unwrap();
        assert_eq!(cross.points.len(), 4);
        assert!(
            note.y < target.y,
            "test fixture must make the note protrude above the target action"
        );
        assert!(
            (cross.points[1].1 - (note.y - CROSS_LANE_VERTICAL_STUB)).abs() < 0.01,
            "cross-lane horizontal level ({:.4}) should route above the target note group ({:.4})",
            cross.points[1].1,
            note.y - CROSS_LANE_VERTICAL_STUB
        );
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
            is_old_style: false,
            old_graph: None,
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
            is_old_style: false,
            old_graph: None,
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
            is_old_style: false,
            old_graph: None,
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
            is_old_style: false,
            old_graph: None,
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
            is_old_style: false,
            old_graph: None,
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
        assert!(
            ((y1 - edge.points[0].1) - CROSS_LANE_VERTICAL_STUB).abs() < 0.01,
            "cross-lane edge should use a {CROSS_LANE_VERTICAL_STUB}px source stub"
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
            is_old_style: false,
            old_graph: None,
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
        // <b>HTML</b> should measure based on the TEXT "HTML", not include literal tag chars.
        // Bold text is slightly wider than plain text due to font weight,
        // but must be much narrower than if the tags were included literally.
        let (w_markup, _) = estimate_note_size("contain <b>HTML</b>");
        let (w_literal, _) = estimate_note_size("contain <b>HTML</b>EXTRA");
        assert!(
            w_markup < w_literal,
            "creole markup should be stripped: markup_w={w_markup} should be less than literal_w={w_literal}"
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
        println!(
            "note lh={lh:.4}, asc={asc:.4}, desc={desc:.4}, asc+desc={:.4}",
            asc + desc
        );
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
    fn estimate_note_size_one_line_matches_java_opale_height() {
        let (_, h) = estimate_note_size("This is a note");
        let expected = font_metrics::line_height("SansSerif", NOTE_FONT_SIZE, false, false)
            + 2.0 * NOTE_MARGIN_Y;
        assert!(
            (h - expected).abs() < 0.0001,
            "one-line note height should be text height + 2*marginY: {h:.4} vs {expected:.4}"
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
    fn wrap_note_text_carries_unclosed_back_highlight_to_continuation_lines() {
        let wrapped = wrap_note_text(r#"* Calling the method is <back:red>prohibited overlap"#, 100.0);
        let lines: Vec<&str> = wrapped.split('\n').collect();
        assert!(
            lines.iter().any(|line| line.contains("<back:red>prohibited")),
            "first wrapped highlight line should keep the opening tag: {lines:?}"
        );
        assert!(
            lines.iter().any(|line| line.contains("<back:red>overlap")),
            "continuation line should inherit the unclosed <back:...> tag: {lines:?}"
        );
    }

    #[test]
    fn wrap_with_max_width_integrates_in_layout() {
        let d = ActivityDiagram {
            events: vec![
                ActivityEvent::Action {
                    text: "work".into(),
                },
                ActivityEvent::Note {
                    position: NotePosition::Right,
                    text: "A Long Long Long Long Long Long Long Long Long note".into(),
                },
            ],
            swimlanes: vec![],
            direction: Default::default(),
            note_max_width: Some(80.0),
            is_old_style: false,
            old_graph: None,
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
