use std::collections::HashMap;

use crate::font_metrics;
use crate::model::sequence::{
    FragmentKind, ParticipantKind, SeqArrowHead, SeqArrowStyle, SeqDirection, SeqEvent,
};
use crate::model::SequenceDiagram;
use crate::Result;

// ── Constants ────────────────────────────────────────────────────────────────

const FONT_SIZE: f64 = 14.0;
const LINE_HEIGHT: f64 = 16.0;
const PARTICIPANT_PADDING: f64 = 7.0;
const PARTICIPANT_HEIGHT: f64 = 30.2969;
const MESSAGE_SPACING: f64 = 29.1328;
const SELF_MSG_WIDTH: f64 = 42.0;
const SELF_MSG_HEIGHT: f64 = 13.0;
const ACTIVATION_WIDTH: f64 = 10.0;
const NOTE_PADDING: f64 = 6.0;
const NOTE_FOLD: f64 = 10.0;
const NOTE_FONT_SIZE: f64 = 13.0;
const GROUP_PADDING: f64 = 10.0;
const FRAGMENT_HEADER_HEIGHT: f64 = 17.1328;
const FRAGMENT_PADDING: f64 = 10.0;

// Fragment y-spacing constants reverse-engineered from Java PlantUML SVG output.
// The model: y_cursor tracks the arrow-center y of the next message.  Fragment
// boundaries are placed by *backing off* from that cursor, then y_cursor is
// reset to the position of the next expected message.
const FRAG_Y_BACKOFF: f64 = 14.1328; // frag_y  = y_cursor - FRAG_Y_BACKOFF
const FRAG_AFTER_HEADER: f64 = 38.2656; // next msg y = frag_y + FRAG_AFTER_HEADER
const FRAG_SEP_BACKOFF: f64 = 20.1328; // sep_y   = y_cursor - FRAG_SEP_BACKOFF
const FRAG_AFTER_SEP: f64 = 34.9375; // next msg y = sep_y  + FRAG_AFTER_SEP
const FRAG_END_BACKOFF: f64 = 21.1328; // frag_bottom = y_cursor - FRAG_END_BACKOFF
const FRAG_AFTER_END: f64 = 28.1328; // y_cursor = frag_bottom + FRAG_AFTER_END
const DIVIDER_HEIGHT: f64 = 30.0;
const DELAY_HEIGHT: f64 = 30.0;
const REF_HEIGHT: f64 = 39.1016;
const REF_Y_BACKOFF: f64 = 21.1328;
const REF_AFTER_END: f64 = 26.1328;
const REF_EDGE_PAD: f64 = 3.0;
const MARGIN: f64 = 5.0;
const MSG_FONT_SIZE: f64 = 13.0;

/// Fragment stack entry: (y_start, kind, label, separators, min_part_idx, max_part_idx, depth_at_push)
type FragmentStackEntry = (f64, FragmentKind, String, Vec<(f64, String)>, Option<usize>, Option<usize>, usize);

// ── Layout output types ──────────────────────────────────────────────────────

/// Participant layout info
#[derive(Debug, Clone)]
pub struct ParticipantLayout {
    pub name: String,
    pub x: f64,
    pub box_width: f64,
    pub box_height: f64,
    pub kind: ParticipantKind,
    pub color: Option<String>,
}

/// Message layout info
#[derive(Debug, Clone)]
pub struct MessageLayout {
    pub from_x: f64,
    pub to_x: f64,
    pub y: f64,
    pub text: String,
    pub text_lines: Vec<String>,
    pub is_self: bool,
    pub is_dashed: bool,
    pub is_left: bool,
    pub has_open_head: bool,
    /// Autonumber string (e.g. "1", "2") — rendered as separate text element
    pub autonumber: Option<String>,
    /// For self-messages: the effective left x for the return arrow, accounting
    /// for any activation bar that overlaps at the return y.
    /// `return_x = max(from_x, activation_bar_right) + 1`
    pub self_return_x: f64,
}

/// Activation bar layout
#[derive(Debug, Clone)]
pub struct ActivationLayout {
    pub x: f64,
    pub y_start: f64,
    pub y_end: f64,
}

/// Destroy marker layout
#[derive(Debug, Clone)]
pub struct DestroyLayout {
    pub x: f64,
    pub y: f64,
}

/// Note layout
#[derive(Debug, Clone)]
pub struct NoteLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
    pub is_left: bool,
}

/// Group box layout
#[derive(Debug, Clone)]
pub struct GroupLayout {
    pub x: f64,
    pub y_start: f64,
    pub y_end: f64,
    pub width: f64,
    pub label: Option<String>,
}

/// Combined fragment layout
#[derive(Debug, Clone)]
pub struct FragmentLayout {
    pub kind: FragmentKind,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// (y_position, label) for each separator (else) within the fragment
    pub separators: Vec<(f64, String)>,
}

/// Divider layout
#[derive(Debug, Clone)]
pub struct DividerLayout {
    pub y: f64,
    pub x: f64,
    pub width: f64,
    pub text: Option<String>,
}

/// Delay indicator layout
#[derive(Debug, Clone)]
pub struct DelayLayout {
    pub y: f64,
    pub height: f64,
    pub x: f64,
    pub width: f64,
    pub text: Option<String>,
}

/// Ref layout
#[derive(Debug, Clone)]
pub struct RefLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub label: String,
}

/// Complete sequence diagram layout result
#[derive(Debug, Clone)]
pub struct SeqLayout {
    pub participants: Vec<ParticipantLayout>,
    pub messages: Vec<MessageLayout>,
    pub activations: Vec<ActivationLayout>,
    pub destroys: Vec<DestroyLayout>,
    pub notes: Vec<NoteLayout>,
    pub groups: Vec<GroupLayout>,
    pub fragments: Vec<FragmentLayout>,
    pub dividers: Vec<DividerLayout>,
    pub delays: Vec<DelayLayout>,
    pub refs: Vec<RefLayout>,
    pub autonumber_enabled: bool,
    pub autonumber_start: u32,
    pub lifeline_top: f64,
    pub lifeline_bottom: f64,
    pub total_width: f64,
    pub total_height: f64,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Find the center x coordinate for a participant by name
fn find_participant_x(participants: &[ParticipantLayout], name: &str) -> f64 {
    for p in participants {
        if p.name == name {
            return p.x;
        }
    }
    log::warn!("participant '{name}' not found in layout, defaulting to 0");
    0.0
}

/// Find the index of a participant by name
fn find_participant_idx(name_to_idx: &HashMap<String, usize>, name: &str) -> Option<usize> {
    name_to_idx.get(name).copied()
}

/// Update min/max participant indices for all open fragments on the stack
fn update_fragment_participant_range(
    fragment_stack: &mut [FragmentStackEntry],
    idx: usize,
) {
    for entry in fragment_stack.iter_mut() {
        entry.4 = Some(entry.4.map_or(idx, |cur| cur.min(idx)));
        entry.5 = Some(entry.5.map_or(idx, |cur| cur.max(idx)));
    }
}

/// Estimate note height: line count * NOTE_FONT_SIZE + top/bottom padding.
/// Uses NOTE_FONT_SIZE (13) rather than LINE_HEIGHT (16) to match Java PlantUML.
fn estimate_note_height(text: &str) -> f64 {
    let lines = text.lines().count().max(1) as f64;
    (lines * NOTE_FONT_SIZE + 2.0 * NOTE_PADDING).max(25.0)
}

/// Compute note width based on text content using font metrics.
/// Width = left_pad + max_line_width + right_pad (includes fold corner).
fn estimate_note_width(text: &str) -> f64 {
    let max_line_w = text
        .lines()
        .map(|line| font_metrics::text_width(line, "SansSerif", NOTE_FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    // left pad (6) + text + right pad (4) + fold (10) = text + 20
    let w = max_line_w + NOTE_PADDING + NOTE_PADDING / 2.0 + NOTE_FOLD + 2.0;
    w.max(30.0)
}

// -- Sprite width/height helpers --

const SPRITE_TEXT_GAP: f64 = 4.1323;
const SPRITE_HEIGHT_THRESHOLD: f64 = 15.1328;

fn message_line_width(line: &str) -> f64 {
    if !line.contains("<$") {
        // Compute width respecting font-family changes in creole markup
        return crate::render::svg_richtext::creole_text_width(line, "SansSerif", MSG_FONT_SIZE, false, false);
    }
    let gap = SPRITE_TEXT_GAP;
    let mut total = 0.0_f64;
    let mut first = true;
    let mut pos = 0;
    let mut had_sprite = false;
    while let Some(start) = line[pos..].find("<$") {
        let abs_start = pos + start;
        if abs_start > pos {
            let text = &line[pos..abs_start];
            let text = if had_sprite { text.strip_prefix(' ').unwrap_or(text) } else { text };
            let text = text.strip_suffix(' ').unwrap_or(text);
            if !text.is_empty() {
                let w = font_metrics::text_width(text, "SansSerif", MSG_FONT_SIZE, false, false);
                if w > 0.0 { if !first { total += gap; } total += w; first = false; }
            }
        }
        let name_start = abs_start + 2;
        if let Some(end) = line[name_start..].find('>') {
            let name_part = &line[name_start..name_start + end];
            let name = name_part.split(',').next().unwrap_or(name_part).trim();
            if let Some(svg) = crate::render::svg_richtext::get_sprite_svg(name) {
                let (w, _) = parse_sprite_viewbox(&svg);
                if !first { total += gap; } total += w; first = false;
            }
            pos = name_start + end + 1; had_sprite = true;
        } else { break; }
    }
    if pos < line.len() {
        let text = &line[pos..];
        let text = if had_sprite { text.strip_prefix(' ').unwrap_or(text) } else { text };
        if !text.is_empty() {
            let w = font_metrics::text_width(text, "SansSerif", MSG_FONT_SIZE, false, false);
            if w > 0.0 { if !first { total += gap; } total += w; }
        }
    }
    total
}

fn message_sprite_extra_height(line: &str) -> f64 {
    if !line.contains("<$") { return 0.0; }
    let mut max_extra = 0.0_f64;
    let mut pos = 0;
    while let Some(start) = line[pos..].find("<$") {
        let abs_start = pos + start + 2;
        if let Some(end) = line[abs_start..].find('>') {
            let name_part = &line[abs_start..abs_start + end];
            let name = name_part.split(',').next().unwrap_or(name_part).trim();
            if let Some(svg) = crate::render::svg_richtext::get_sprite_svg(name) {
                let (_, h) = parse_sprite_viewbox(&svg);
                let extra = (h - SPRITE_HEIGHT_THRESHOLD).max(0.0);
                max_extra = max_extra.max(extra);
            }
            pos = abs_start + end + 1;
        } else { break; }
    }
    max_extra
}

fn parse_sprite_viewbox(svg: &str) -> (f64, f64) {
    if let Some(vb_start) = svg.find("viewBox=\"") {
        let rest = &svg[vb_start + 9..];
        if let Some(vb_end) = rest.find('"') {
            let parts: Vec<&str> = rest[..vb_end].split_whitespace().collect();
            if parts.len() == 4 {
                return (parts[2].parse().unwrap_or(100.0), parts[3].parse().unwrap_or(50.0));
            }
        }
    }
    (100.0, 50.0)
}

// ── Main layout function ─────────────────────────────────────────────────────

/// Perform columnar layout on a SequenceDiagram
pub fn layout_sequence(sd: &SequenceDiagram) -> Result<SeqLayout> {
    log::debug!(
        "layout_sequence: {} participants, {} events",
        sd.participants.len(),
        sd.events.len()
    );

    // 1. Compute participant box widths first
    let mut box_widths: Vec<f64> = Vec::with_capacity(sd.participants.len());
    let mut box_heights: Vec<f64> = Vec::with_capacity(sd.participants.len());
    let mut part_name_to_idx: HashMap<String, usize> = HashMap::new();

    for (i, p) in sd.participants.iter().enumerate() {
        let display = p.display_name.as_deref().unwrap_or(&p.name);
        let bw = (font_metrics::text_width(display, "SansSerif", FONT_SIZE, false, false)
            + 2.0 * PARTICIPANT_PADDING)
            .max(40.0);
        let bh = match p.kind {
            ParticipantKind::Actor => PARTICIPANT_HEIGHT + 45.0,
            ParticipantKind::Boundary
            | ParticipantKind::Control
            | ParticipantKind::Entity
            | ParticipantKind::Database
            | ParticipantKind::Collections
            | ParticipantKind::Queue => PARTICIPANT_HEIGHT + 20.0,
            ParticipantKind::Default => PARTICIPANT_HEIGHT,
        };
        box_widths.push(bw);
        box_heights.push(bh);
        part_name_to_idx.insert(p.name.clone(), i);
    }

    // 2. Compute minimum gaps between adjacent participant centers
    let n = sd.participants.len();
    let mut min_gaps: Vec<f64> = if n > 1 {
        (0..n - 1)
            .map(|i| box_widths[i] / 2.0 + box_widths[i + 1] / 2.0 + 10.0)
            .collect()
    } else {
        Vec::new()
    };

    // Scan messages to widen gaps based on text width
    let mut gap_autonumber_enabled = false;
    let mut gap_autonumber_counter: u32 = 1;
    for event in &sd.events {
        match event {
            SeqEvent::AutoNumber { start } => {
                gap_autonumber_enabled = true;
                if let Some(n) = start {
                    gap_autonumber_counter = *n;
                }
            }
            SeqEvent::Message(msg) => {
                // Compute autonumber extra width
                let autonumber_extra_w = if gap_autonumber_enabled {
                    let num_str = format!("{gap_autonumber_counter}");
                    let num_w = font_metrics::text_width(
                        &num_str, "SansSerif", MSG_FONT_SIZE, true, false,
                    );
                    gap_autonumber_counter += 1;
                    num_w + 4.0 // 4px gap between number and text
                } else {
                    0.0
                };

                if msg.from == msg.to {
                    continue; // self-messages don't affect gap
                }
                if let (Some(&fi), Some(&ti)) =
                    (part_name_to_idx.get(&msg.from), part_name_to_idx.get(&msg.to))
                {
                    let (lo, hi) = if fi < ti { (fi, ti) } else { (ti, fi) };
                    // Use the longest single line for gap calculation (multiline \n)
                    let text_w = msg
                        .text
                        .split("\\n")
                        .map(|line| message_line_width(line))
                        .fold(0.0_f64, f64::max)
                        + autonumber_extra_w;
                    let needed = text_w + 24.0; // 7px text-margin + 7px gap + 10px arrow
                    let span = hi - lo; // number of gaps this message spans
                    if span > 0 {
                        let per_gap = needed / span as f64;
                        for g in lo..hi {
                            if per_gap > min_gaps[g] {
                                min_gaps[g] = per_gap;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Pre-scan: compute fragment nesting depth per participant and determine left margin.
    // max_frag_depth_per_participant[i] = max nesting depth (1-based) of fragments involving participant i.
    let mut max_frag_depth: Vec<usize> = vec![0; n];
    {
        // Track (min_idx, max_idx, depth_at_push) per open fragment level
        let mut prescan_stack: Vec<(Option<usize>, Option<usize>, usize)> = Vec::new();

        for event in &sd.events {
            match event {
                SeqEvent::FragmentStart { .. } => {
                    let depth = prescan_stack.len();
                    prescan_stack.push((None, None, depth));
                }
                SeqEvent::Message(msg) => {
                    if !prescan_stack.is_empty() {
                        let fi = part_name_to_idx.get(&msg.from).copied();
                        let ti = part_name_to_idx.get(&msg.to).copied();
                        for entry in prescan_stack.iter_mut() {
                            if let Some(idx) = fi {
                                entry.0 = Some(entry.0.map_or(idx, |cur: usize| cur.min(idx)));
                                entry.1 = Some(entry.1.map_or(idx, |cur: usize| cur.max(idx)));
                            }
                            if let Some(idx) = ti {
                                entry.0 = Some(entry.0.map_or(idx, |cur: usize| cur.min(idx)));
                                entry.1 = Some(entry.1.map_or(idx, |cur: usize| cur.max(idx)));
                            }
                        }
                    }
                }
                SeqEvent::FragmentEnd => {
                    if let Some((min_idx, max_idx, _depth)) = prescan_stack.pop() {
                        let frag_depth = prescan_stack.len() + 1; // 1-based depth
                        if let (Some(lo), Some(hi)) = (min_idx, max_idx) {
                            for pidx in lo..=hi {
                                if frag_depth > max_frag_depth[pidx] {
                                    max_frag_depth[pidx] = frag_depth;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let max_depth_for_leftmost = if n > 0 { max_frag_depth[0] } else { 0 };
    let left_margin = if max_depth_for_leftmost > 0 {
        2.0 * MARGIN + max_depth_for_leftmost as f64 * FRAGMENT_PADDING
    } else {
        MARGIN
    };

    // 3. Position participants left-to-right using computed gaps
    let mut participants: Vec<ParticipantLayout> = Vec::with_capacity(n);
    let mut prev_center: Option<f64> = None;
    for (i, p) in sd.participants.iter().enumerate() {
        let center_x = match prev_center {
            None => left_margin + box_widths[i] / 2.0,
            Some(pc) => pc + min_gaps[i - 1],
        };

        participants.push(ParticipantLayout {
            name: p.name.clone(),
            x: center_x,
            box_width: box_widths[i],
            box_height: box_heights[i],
            kind: p.kind.clone(),
            color: p.color.clone(),
        });

        prev_center = Some(center_x);
    }

    // 2. Event layout
    let max_ph = participants
        .iter()
        .map(|pp| pp.box_height)
        .fold(PARTICIPANT_HEIGHT, f64::max);

    // Pre-scan: check if any note immediately follows a non-self message.
    // Java PlantUML adds ~3px extra initial spacing when notes overlay
    // regular (non-self) messages.
    let has_regular_msg_note = sd.events.windows(2).any(|w| {
        if let SeqEvent::Message(msg) = &w[0] {
            msg.from != msg.to
                && matches!(
                    &w[1],
                    SeqEvent::NoteRight { .. }
                        | SeqEvent::NoteLeft { .. }
                        | SeqEvent::NoteOver { .. }
                )
        } else {
            false
        }
    });
    let note_extra = if has_regular_msg_note { 3.0 } else { 0.0 };
    let mut y_cursor = MARGIN + max_ph + 32.1328 + note_extra;

    // Track the bottom y of the last rendered event for lifeline sizing.
    // This stores the lifeline_bottom directly (not an intermediate value).
    let mut lifeline_extend_y: f64 = y_cursor;

    // For self-messages followed by activate: the activation bar should start
    // at the self-message return y, not at y_cursor (which has already advanced
    // to the next message position).  Keyed by participant name.
    let mut pending_self_return_y: HashMap<String, f64> = HashMap::new();

    // Track the y of the most recent message for note back-offset positioning.
    // In Java PlantUML, notes following a message are placed alongside it
    // (overlapping vertically) rather than below it.
    let mut last_message_y: Option<f64> = None;
    let mut last_message_was_self: bool = false;

    // When a note is placed alongside a message (back-offset), the next
    // activate should start at the message's y, not at y_cursor.
    let mut pending_note_activate_y: Option<f64> = None;

    let mut messages: Vec<MessageLayout> = Vec::new();
    let mut activations: Vec<ActivationLayout> = Vec::new();
    let mut destroys: Vec<DestroyLayout> = Vec::new();
    let mut notes: Vec<NoteLayout> = Vec::new();
    let mut groups: Vec<GroupLayout> = Vec::new();
    let mut fragments: Vec<FragmentLayout> = Vec::new();
    let mut dividers: Vec<DividerLayout> = Vec::new();
    let mut delays: Vec<DelayLayout> = Vec::new();
    let mut refs: Vec<RefLayout> = Vec::new();
    let mut autonumber_enabled = false;
    let mut autonumber_start: u32 = 1;
    let mut autonumber_counter: u32 = 1;

    // Activation stack: participant name -> Vec<y_start>
    let mut activation_stack: HashMap<String, Vec<f64>> = HashMap::new();
    // Group stack: (y_start, label)
    let mut group_stack: Vec<(f64, Option<String>)> = Vec::new();
    // Fragment stack: (y_start, kind, label, separators)
    let mut fragment_stack: Vec<FragmentStackEntry> = Vec::new();

    let leftmost = participants
        .first()
        .map_or(MARGIN, |p| p.x - p.box_width / 2.0);
    let rightmost = participants
        .last()
        .map_or(MARGIN, |p| p.x + p.box_width / 2.0);
    let full_width = (rightmost - leftmost).max(60.0) + 2.0 * FRAGMENT_PADDING;

    for (event_idx, event) in sd.events.iter().enumerate() {
        match event {
            SeqEvent::Message(msg) => {
                let from_x = find_participant_x(&participants, &msg.from);
                let to_x = find_participant_x(&participants, &msg.to);
                let is_self = msg.from == msg.to;
                let is_dashed = msg.arrow_style == SeqArrowStyle::Dashed
                    || msg.arrow_style == SeqArrowStyle::Dotted;
                let is_left = from_x > to_x;
                let has_open_head = msg.arrow_head == SeqArrowHead::Open;

                // Track participant indices for fragment spanning
                if !fragment_stack.is_empty() {
                    if let Some(fi) = find_participant_idx(&part_name_to_idx, &msg.from) {
                        update_fragment_participant_range(&mut fragment_stack, fi);
                    }
                    if let Some(ti) = find_participant_idx(&part_name_to_idx, &msg.to) {
                        update_fragment_participant_range(&mut fragment_stack, ti);
                    }
                }

                let text_lines: Vec<String> =
                    msg.text.split("\\n").map(|s| s.to_string()).collect();
                let num_extra_lines = if text_lines.len() > 1 {
                    text_lines.len() - 1
                } else {
                    0
                };
                // Multiline message text: extra lines push the arrow down
                let msg_line_spacing = 15.1328; // ascent + descent at font-size 13
                let multiline_extra = num_extra_lines as f64 * msg_line_spacing;
                let sprite_extra = msg.text.split("\\n")
                    .map(|line| message_sprite_extra_height(line))
                    .fold(0.0_f64, f64::max);
                let extra_height = multiline_extra + sprite_extra;
                let msg_y = y_cursor + extra_height;

                let msg_autonumber = if autonumber_enabled {
                    let num = format!("{autonumber_counter}");
                    autonumber_counter += 1;
                    Some(num)
                } else {
                    None
                };

                // For self-messages, compute activation-aware positions.
                // The return arrow and loop width must clear any activation bar
                // at the return y. This includes look-ahead: if the next event
                // is Activate for this participant, the activation bar will start
                // at the self-message return y.
                let is_activated = is_self
                    && activation_stack
                        .get(&msg.from)
                        .is_some_and(|s| !s.is_empty());
                // Check if activation is about to start (next event is Activate)
                let will_activate = is_self
                    && sd
                        .events
                        .get(event_idx + 1)
                        .is_some_and(|e| matches!(e, SeqEvent::Activate(n) if n == &msg.from));

                // When a non-activated self-message triggers an upcoming activate,
                // shift the outgoing y up by ACTIVATION_WIDTH/2 so the return y
                // aligns with the activation bar start position.
                let msg_y = if is_self && will_activate && !is_activated {
                    msg_y - ACTIVATION_WIDTH / 2.0
                } else {
                    msg_y
                };

                let (self_from_x, self_return_x, self_to_x) = if is_self {
                    let has_bar = is_activated || will_activate;
                    let act_right = if has_bar {
                        from_x + ACTIVATION_WIDTH / 2.0
                    } else {
                        from_x
                    };
                    let outgoing_x = if is_activated { act_right } else { from_x };
                    let ret_x = act_right + 1.0;
                    let to = act_right + SELF_MSG_WIDTH;
                    (outgoing_x, ret_x, to)
                } else {
                    (from_x, from_x, to_x)
                };

                messages.push(MessageLayout {
                    from_x: if is_self { self_from_x } else { from_x },
                    to_x: if is_self { self_to_x } else { to_x },
                    y: msg_y,
                    text: msg.text.clone(),
                    text_lines,
                    is_self,
                    is_dashed,
                    is_left,
                    has_open_head,
                    autonumber: msg_autonumber,
                    self_return_x,
                });

                // Only enable note back-offset for single-line messages.
                // Multi-line messages have complex text layout and the note
                // positioning follows different rules in Java PlantUML.
                if num_extra_lines == 0 {
                    last_message_y = Some(msg_y);
                    last_message_was_self = is_self;
                } else {
                    last_message_y = None;
                }

                if is_self {
                    let return_y = msg_y + SELF_MSG_HEIGHT;
                    lifeline_extend_y = return_y + 18.0;
                    // Cursor advances based on the unadjusted position to
                    // maintain consistent spacing for subsequent messages.
                    let unadjusted_return = y_cursor + extra_height + SELF_MSG_HEIGHT;
                    y_cursor = unadjusted_return + MESSAGE_SPACING;
                    pending_self_return_y.insert(msg.from.clone(), return_y);
                } else {
                    lifeline_extend_y = msg_y + 18.0;
                    y_cursor = msg_y + MESSAGE_SPACING;
                    pending_self_return_y.clear();
                }
            }

            SeqEvent::Activate(name) => {
                // Priority: 1) self-message return y, 2) note-attached message y, 3) y_cursor
                let act_y = if let Some(y) = pending_self_return_y.remove(name.as_str()) {
                    y
                } else if let Some(y) = pending_note_activate_y.take() {
                    // When activation follows a note attached to a message,
                    // add extra spacing for subsequent messages to match Java.
                    y_cursor += 3.0;
                    y
                } else {
                    y_cursor
                };
                log::debug!("activate '{name}' at y={act_y:.1}");
                activation_stack
                    .entry(name.clone())
                    .or_default()
                    .push(act_y);
            }

            SeqEvent::Deactivate(name) => {
                let px = find_participant_x(&participants, name);
                if let Some(stack) = activation_stack.get_mut(name.as_str()) {
                    if let Some(y_start) = stack.pop() {
                        activations.push(ActivationLayout {
                            x: px - ACTIVATION_WIDTH / 2.0,
                            y_start,
                            y_end: y_cursor,
                        });
                        log::debug!(
                            "deactivate '{name}' at y={y_cursor:.1}, bar from {y_start:.1}"
                        );
                    } else {
                        log::warn!("deactivate '{name}' with empty stack");
                    }
                } else {
                    log::warn!("deactivate '{name}' without prior activate");
                }
            }

            SeqEvent::Destroy(name) => {
                let px = find_participant_x(&participants, name);
                // For self-messages, the destroy should be at the return y
                let destroy_y = pending_self_return_y
                    .remove(name.as_str())
                    .unwrap_or(y_cursor);
                destroys.push(DestroyLayout { x: px, y: destroy_y });

                // Also close any active activation bar for this participant.
                // The bar ends slightly above the destroy center (offset -7
                // matches Java PlantUML visual spacing).
                if let Some(stack) = activation_stack.get_mut(name.as_str()) {
                    if let Some(y_start) = stack.pop() {
                        let bar_end = destroy_y - 7.0;
                        activations.push(ActivationLayout {
                            x: px - ACTIVATION_WIDTH / 2.0,
                            y_start,
                            y_end: bar_end,
                        });
                        log::debug!(
                            "destroy-deactivate '{name}' bar from {y_start:.1} to {bar_end:.1}"
                        );
                    }
                }

                y_cursor = destroy_y + MESSAGE_SPACING;
                last_message_y = None;
                log::debug!("destroy '{name}' at y={destroy_y:.1}");
            }

            SeqEvent::NoteRight { participant, text } => {
                let px = find_participant_x(&participants, participant);
                let note_height = estimate_note_height(text);
                let note_width = estimate_note_width(text);
                // In Java PlantUML, notes following a message are placed alongside
                // the message (with a back-offset) rather than below it.
                // The note doesn't advance y_cursor when it fits within the
                // message spacing already consumed.
                let note_y = if let Some(msg_y) = last_message_y {
                    // Place note with back-offset from message position
                    let back_offset = if last_message_was_self {
                        (note_height - 1.0) / 2.0
                    } else {
                        MESSAGE_SPACING - NOTE_FOLD
                    };
                    (msg_y - back_offset).max(MARGIN + max_ph)
                } else {
                    y_cursor
                };
                notes.push(NoteLayout {
                    x: px + ACTIVATION_WIDTH,
                    y: note_y,
                    width: note_width,
                    height: note_height,
                    text: text.clone(),
                    is_left: false,
                });
                // Only advance y_cursor if the note bottom extends below current position
                let note_bottom = note_y + note_height;
                if note_bottom > y_cursor {
                    y_cursor = note_bottom;
                }
                if last_message_y.is_some() {
                    pending_note_activate_y = last_message_y;
                }
                last_message_y = None;
            }

            SeqEvent::NoteLeft { participant, text } => {
                let px = find_participant_x(&participants, participant);
                let note_height = estimate_note_height(text);
                let note_width = estimate_note_width(text);
                let note_y = if let Some(msg_y) = last_message_y {
                    let back_offset = if last_message_was_self {
                        (note_height - 1.0) / 2.0
                    } else {
                        MESSAGE_SPACING - NOTE_FOLD
                    };
                    (msg_y - back_offset).max(MARGIN + max_ph)
                } else {
                    y_cursor
                };
                notes.push(NoteLayout {
                    x: px - ACTIVATION_WIDTH - note_width,
                    y: note_y,
                    width: note_width,
                    height: note_height,
                    text: text.clone(),
                    is_left: true,
                });
                let note_bottom = note_y + note_height;
                if note_bottom > y_cursor {
                    y_cursor = note_bottom;
                }
                if last_message_y.is_some() {
                    pending_note_activate_y = last_message_y;
                }
                last_message_y = None;
            }

            SeqEvent::NoteOver {
                participants: parts,
                text,
            } => {
                // Place note centered over the listed participants
                if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
                    let x1 = find_participant_x(&participants, first);
                    let x2 = find_participant_x(&participants, last);
                    let center = (x1 + x2) / 2.0;
                    let note_height = estimate_note_height(text);
                    let note_w = estimate_note_width(text);
                    let width = (x2 - x1).abs().max(note_w);
                    let note_y = if let Some(msg_y) = last_message_y {
                        let back_offset = if last_message_was_self {
                            (note_height - 1.0) / 2.0
                        } else {
                            MESSAGE_SPACING - NOTE_FOLD
                        };
                        (msg_y - back_offset).max(MARGIN + max_ph)
                    } else {
                        y_cursor
                    };
                    notes.push(NoteLayout {
                        x: center - width / 2.0,
                        y: note_y,
                        width,
                        height: note_height,
                        text: text.clone(),
                        is_left: false,
                    });
                    let note_bottom = note_y + note_height;
                    if note_bottom > y_cursor {
                        y_cursor = note_bottom;
                    }
                    if last_message_y.is_some() {
                        pending_note_activate_y = last_message_y;
                    }
                    last_message_y = None;
                }
            }

            SeqEvent::GroupStart { label } => {
                group_stack.push((y_cursor, label.clone()));
                y_cursor += GROUP_PADDING;
                last_message_y = None;
            }

            SeqEvent::GroupEnd => {
                if let Some((y_start, label)) = group_stack.pop() {
                    // Group spans the full width of participants
                    let leftmost = participants
                        .first()
                        .map_or(MARGIN, |p| p.x - p.box_width / 2.0);
                    let rightmost = participants
                        .last()
                        .map_or(MARGIN, |p| p.x + p.box_width / 2.0);
                    groups.push(GroupLayout {
                        x: leftmost - GROUP_PADDING,
                        y_start,
                        y_end: y_cursor,
                        width: (rightmost - leftmost) + 2.0 * GROUP_PADDING,
                        label,
                    });
                    y_cursor += GROUP_PADDING;
                } else {
                    log::warn!("GroupEnd without matching GroupStart");
                }
            }

            SeqEvent::Divider { text } => {
                dividers.push(DividerLayout {
                    y: y_cursor,
                    x: leftmost - FRAGMENT_PADDING,
                    width: full_width,
                    text: text.clone(),
                });
                y_cursor += DIVIDER_HEIGHT;
                last_message_y = None;
            }

            SeqEvent::Delay { text } => {
                delays.push(DelayLayout {
                    y: y_cursor,
                    height: DELAY_HEIGHT,
                    x: leftmost - FRAGMENT_PADDING,
                    width: full_width,
                    text: text.clone(),
                });
                y_cursor += DELAY_HEIGHT;
                last_message_y = None;
            }

            SeqEvent::FragmentStart { kind, label } => {
                let frag_y = y_cursor - FRAG_Y_BACKOFF;
                let depth = fragment_stack.len();
                fragment_stack.push((frag_y, kind.clone(), label.clone(), Vec::new(), None, None, depth));
                y_cursor = frag_y + FRAG_AFTER_HEADER;
                last_message_y = None;
            }

            SeqEvent::FragmentSeparator { label } => {
                if let Some(entry) = fragment_stack.last_mut() {
                    let sep_y = y_cursor - FRAG_SEP_BACKOFF;
                    entry.3.push((sep_y, label.clone()));
                    y_cursor = sep_y + FRAG_AFTER_SEP;
                } else {
                    log::warn!("FragmentSeparator without matching FragmentStart");
                }
            }

            SeqEvent::FragmentEnd => {
                if let Some((y_start, kind, label, separators, min_idx, max_idx, depth_at_push)) = fragment_stack.pop() {
                    let frag_end_y = y_cursor - FRAG_END_BACKOFF;
                    let frag_height = frag_end_y - y_start;

                    // Compute fragment x and width based on involved participants.
                    // Nested fragments get increasing padding: innermost uses
                    // FRAGMENT_PADDING, each outer layer adds another FRAGMENT_PADDING.
                    let (frag_left, frag_right) = if let (Some(lo), Some(hi)) = (min_idx, max_idx) {
                        let p_lo = &participants[lo];
                        let p_hi = &participants[hi];
                        let left_pad = FRAGMENT_PADDING * (max_frag_depth[lo] - depth_at_push) as f64;
                        let right_pad = FRAGMENT_PADDING * (max_frag_depth[hi] - depth_at_push) as f64;
                        let fl = p_lo.x - p_lo.box_width / 2.0 - left_pad;
                        let fr = p_hi.x + p_hi.box_width / 2.0 + right_pad;
                        (fl, fr)
                    } else {
                        // Fallback: span all participants
                        (leftmost - FRAGMENT_PADDING, leftmost - FRAGMENT_PADDING + full_width)
                    };

                    // Compute min width for label tab + guard text.
                    // For Group, the tab displays the label directly (no keyword).
                    // For others, the tab shows the keyword and the guard text
                    // "[label]" is rendered separately to its right.
                    let label_min_w = if kind == FragmentKind::Group {
                        let tab_text = if label.is_empty() {
                            kind.label().to_string()
                        } else {
                            label.clone()
                        };
                        let tab_text_w = font_metrics::text_width(
                            &tab_text, "SansSerif", 13.0, true, false,
                        );
                        tab_text_w + 50.0 // 15(left) + text + 30(right+notch) + 5(margin)
                    } else {
                        let kind_text_w = font_metrics::text_width(
                            kind.label(), "SansSerif", 13.0, true, false,
                        );
                        // Tab: 15(left) + kind_text_w + 30(right+notch)
                        let tab_right = kind_text_w + 45.0;
                        if !label.is_empty() {
                            let guard_text = format!("[{label}]");
                            let guard_w = font_metrics::text_width(
                                &guard_text, "SansSerif", 11.0, true, false,
                            );
                            tab_right + 15.0 + guard_w + 5.0
                        } else {
                            tab_right + 5.0
                        }
                    };
                    let frag_w = (frag_right - frag_left).max(label_min_w);

                    fragments.push(FragmentLayout {
                        kind,
                        label,
                        x: frag_left,
                        y: y_start,
                        width: frag_w,
                        height: frag_height,
                        separators,
                    });
                    lifeline_extend_y = frag_end_y + 17.0;
                    y_cursor = frag_end_y + FRAG_AFTER_END;
                } else {
                    log::warn!("FragmentEnd without matching FragmentStart");
                }
            }

            SeqEvent::Ref {
                participants: parts,
                label,
            } => {
                if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
                    let ref_y = y_cursor - REF_Y_BACKOFF;
                    let first_idx = part_name_to_idx.get(first.as_str()).copied();
                    let last_idx = part_name_to_idx.get(last.as_str()).copied();
                    let (left_x, right_x) = if let (Some(fi), Some(li)) = (first_idx, last_idx) {
                        let lo = fi.min(li);
                        let hi = fi.max(li);
                        let p_lo = &participants[lo];
                        let p_hi = &participants[hi];
                        (p_lo.x - p_lo.box_width / 2.0 - REF_EDGE_PAD,
                         p_hi.x + p_hi.box_width / 2.0 + REF_EDGE_PAD)
                    } else {
                        let x1 = find_participant_x(&participants, first);
                        let x2 = find_participant_x(&participants, last);
                        (x1.min(x2) - 30.0, x1.max(x2) + 30.0)
                    };
                    refs.push(RefLayout {
                        x: left_x,
                        y: ref_y,
                        width: right_x - left_x,
                        height: REF_HEIGHT,
                        label: label.clone(),
                    });
                    lifeline_extend_y = ref_y + REF_HEIGHT + 17.0;
                    y_cursor = ref_y + REF_HEIGHT + REF_AFTER_END;
                    last_message_y = None;
                }
            }

            SeqEvent::Spacing { pixels } => {
                y_cursor += *pixels as f64;
                last_message_y = None;
            }

            SeqEvent::AutoNumber { start } => {
                autonumber_enabled = true;
                if let Some(n) = start {
                    autonumber_start = *n;
                    autonumber_counter = *n;
                }
            }
        }
    }

    // Close any remaining activations (unmatched).
    // Iterate in participant declaration order for deterministic output.
    for p in &participants {
        let Some(stack) = activation_stack.get(&p.name) else {
            continue;
        };
        for &y_start in stack {
            let name = &p.name;
            let px = find_participant_x(&participants, name);
            activations.push(ActivationLayout {
                x: px - ACTIVATION_WIDTH / 2.0,
                y_start,
                y_end: y_cursor,
            });
            log::warn!(
                "unclosed activation for '{name}' from y={y_start:.1}, closing at y={y_cursor:.1}"
            );
        }
    }

    // 3. Finalize
    let max_participant_height = participants
        .iter()
        .map(|pp| pp.box_height)
        .fold(PARTICIPANT_HEIGHT, f64::max);
    let lifeline_top = MARGIN + max_participant_height + 1.0;
    let lifeline_bottom = lifeline_extend_y;

    let right_margin = 2.0 * MARGIN;
    let mut total_width = participants
        .last()
        .map_or(2.0 * MARGIN, |p| p.x + p.box_width / 2.0 + right_margin);

    // Expand total_width if any note extends beyond the participant area
    for note in &notes {
        let note_right = note.x + note.width + MARGIN;
        if note_right > total_width {
            log::debug!(
                "note extends beyond participants: note_right={note_right:.1}, expanding total_width from {total_width:.1}"
            );
            total_width = note_right;
        }
    }

    // Expand total_width if any fragment extends beyond participants
    for frag in &fragments {
        let frag_right = frag.x + frag.width + MARGIN + FRAGMENT_PADDING;
        if frag_right > total_width {
            total_width = frag_right;
        }
    }

    // Account for self-message loops extending to the right
    let self_msg_right = messages
        .iter()
        .filter(|m| m.is_self)
        .map(|m| m.from_x + SELF_MSG_WIDTH + MARGIN)
        .fold(0.0_f64, f64::max);
    if self_msg_right > total_width {
        total_width = self_msg_right;
    }

    // Tail box at lifeline_bottom - 1, then add box height + bottom margin (~7)
    let total_height = (lifeline_bottom - 1.0) + max_participant_height + 7.0;

    // Close any remaining fragments (unmatched)
    for (y_start, kind, label, separators, min_idx, max_idx, depth_at_push) in fragment_stack.drain(..) {
        let (frag_x, frag_w) = if let (Some(lo), Some(hi)) = (min_idx, max_idx) {
            let p_lo = &participants[lo];
            let p_hi = &participants[hi];
            let left_pad = FRAGMENT_PADDING * (max_frag_depth[lo] - depth_at_push) as f64;
            let right_pad = FRAGMENT_PADDING * (max_frag_depth[hi] - depth_at_push) as f64;
            let fl = p_lo.x - p_lo.box_width / 2.0 - left_pad;
            let fr = p_hi.x + p_hi.box_width / 2.0 + right_pad;
            (fl, fr - fl)
        } else {
            (leftmost - FRAGMENT_PADDING, full_width)
        };
        let frag_height = y_cursor - y_start;
        fragments.push(FragmentLayout {
            kind,
            label,
            x: frag_x,
            y: y_start,
            width: frag_w,
            height: frag_height,
            separators,
        });
        log::warn!("unclosed fragment, closing at y={y_cursor:.1}");
    }

    log::debug!(
        "layout_sequence done: {:.0}x{:.0}, {} messages, {} activations, {} fragments",
        total_width,
        total_height,
        messages.len(),
        activations.len(),
        fragments.len()
    );

    Ok(SeqLayout {
        participants,
        messages,
        activations,
        destroys,
        notes,
        groups,
        fragments,
        dividers,
        delays,
        refs,
        autonumber_enabled,
        autonumber_start,
        lifeline_top,
        lifeline_bottom,
        total_width,
        total_height,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::sequence::{
        FragmentKind, Message, Participant, ParticipantKind, SeqArrowHead, SeqArrowStyle,
        SeqDirection, SeqEvent, SequenceDiagram,
    };

    fn make_participant(name: &str) -> Participant {
        Participant {
            name: name.to_string(),
            display_name: None,
            kind: ParticipantKind::Default,
            color: None,
        }
    }

    fn make_message(from: &str, to: &str, text: &str) -> Message {
        Message {
            from: from.to_string(),
            to: to.to_string(),
            text: text.to_string(),
            arrow_style: SeqArrowStyle::Solid,
            arrow_head: SeqArrowHead::Filled,
            direction: SeqDirection::LeftToRight,
            color: None,
        }
    }

    #[test]
    fn single_participant_layout_dimensions() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("Alice")],
            events: vec![],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.participants.len(), 1);
        let p = &layout.participants[0];
        assert_eq!(p.name, "Alice");
        assert_eq!(p.box_height, PARTICIPANT_HEIGHT);

        let expected_bw =
            (crate::font_metrics::text_width("Alice", "SansSerif", FONT_SIZE, false, false)
                + 2.0 * PARTICIPANT_PADDING)
                .max(40.0);
        assert!(
            (p.box_width - expected_bw).abs() < 0.01,
            "box_width {}, expected {}",
            p.box_width,
            expected_bw
        );

        // center x = MARGIN + box_width / 2
        let expected_x = MARGIN + expected_bw / 2.0;
        assert!(
            (p.x - expected_x).abs() < 0.01,
            "x {}, expected {}",
            p.x,
            expected_x
        );

        // total width = center + box_width/2 + MARGIN
        assert!(layout.total_width > 0.0);
        assert!(layout.total_height > 0.0);
    }

    #[test]
    fn two_participants_one_message() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("Alice"), make_participant("Bob")],
            events: vec![SeqEvent::Message(make_message("Alice", "Bob", "hello"))],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.participants.len(), 2);
        assert_eq!(layout.messages.len(), 1);

        let alice_x = layout.participants[0].x;
        let bob_x = layout.participants[1].x;
        // Gap is now content-adaptive; just verify Bob is to the right of Alice
        assert!(
            bob_x > alice_x,
            "Bob center {bob_x} should be right of Alice center {alice_x}"
        );

        let msg = &layout.messages[0];
        assert!(!msg.is_self);
        assert!((msg.from_x - alice_x).abs() < 0.01);
        assert!((msg.to_x - bob_x).abs() < 0.01);
        assert_eq!(msg.text, "hello");
        assert!(!msg.is_dashed);
    }

    #[test]
    fn self_message_layout() {
        let sd_self = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::Message(make_message("A", "A", "self"))],
        };

        let layout_self = layout_sequence(&sd_self).unwrap();

        let msg = &layout_self.messages[0];
        assert!(msg.is_self);
        // Self-message to_x should be offset by SELF_MSG_WIDTH from from_x
        assert!(
            (msg.to_x - msg.from_x - SELF_MSG_WIDTH).abs() < 0.01,
            "self-msg width {} should be SELF_MSG_WIDTH={}",
            msg.to_x - msg.from_x,
            SELF_MSG_WIDTH
        );
        assert!(layout_self.lifeline_bottom > layout_self.lifeline_top);
    }

    #[test]
    fn activation_bar_tracking() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![
                SeqEvent::Message(make_message("A", "B", "req")),
                SeqEvent::Activate("B".to_string()),
                SeqEvent::Message(make_message("B", "A", "resp")),
                SeqEvent::Deactivate("B".to_string()),
            ],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.activations.len(), 1);
        let act = &layout.activations[0];
        assert!(
            act.y_end > act.y_start,
            "activation bar must have positive height"
        );

        let bob_x = layout.participants[1].x;
        assert!(
            (act.x - (bob_x - ACTIVATION_WIDTH / 2.0)).abs() < 0.01,
            "activation x should be centered on participant"
        );
    }

    #[test]
    fn empty_diagram_produces_valid_layout() {
        let sd = SequenceDiagram {
            participants: vec![],
            events: vec![],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert!(layout.participants.is_empty());
        assert!(layout.messages.is_empty());
        assert!(layout.activations.is_empty());
        assert!(layout.total_width > 0.0);
        assert!(layout.total_height > 0.0);
        assert!(layout.lifeline_bottom > layout.lifeline_top);
    }

    #[test]
    fn note_right_advances_cursor() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![
                SeqEvent::NoteRight {
                    participant: "A".to_string(),
                    text: "a note".to_string(),
                },
                SeqEvent::Message(make_message("A", "A", "after note")),
            ],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.notes.len(), 1);
        assert!(!layout.notes[0].is_left);
        // Message should be positioned below the note
        assert!(layout.messages[0].y > layout.notes[0].y);
    }

    #[test]
    fn group_creates_frame() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![
                SeqEvent::GroupStart {
                    label: Some("loop".to_string()),
                },
                SeqEvent::Message(make_message("A", "B", "ping")),
                SeqEvent::GroupEnd,
            ],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.groups.len(), 1);
        let grp = &layout.groups[0];
        assert_eq!(grp.label.as_deref(), Some("loop"));
        assert!(grp.y_end > grp.y_start);
        assert!(grp.width > 0.0);
    }

    #[test]
    fn dashed_arrow_and_open_head() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![SeqEvent::Message(Message {
                from: "A".to_string(),
                to: "B".to_string(),
                text: "reply".to_string(),
                arrow_style: SeqArrowStyle::Dashed,
                arrow_head: SeqArrowHead::Open,
                direction: SeqDirection::LeftToRight,
                color: None,
            })],
        };
        let layout = layout_sequence(&sd).unwrap();

        let msg = &layout.messages[0];
        assert!(msg.is_dashed);
        assert!(msg.has_open_head);
    }

    #[test]
    fn destroy_advances_cursor() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![
                SeqEvent::Message(make_message("A", "B", "kill")),
                SeqEvent::Destroy("B".to_string()),
            ],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.destroys.len(), 1);
        let d = &layout.destroys[0];
        let bob_x = layout.participants[1].x;
        assert!((d.x - bob_x).abs() < 0.01);
        // destroy y should be after the message
        assert!(d.y > layout.messages[0].y);
    }

    #[test]
    fn fragment_creates_layout() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![
                SeqEvent::FragmentStart {
                    kind: FragmentKind::Alt,
                    label: "success".to_string(),
                },
                SeqEvent::Message(make_message("A", "B", "ok")),
                SeqEvent::FragmentSeparator {
                    label: "failure".to_string(),
                },
                SeqEvent::Message(make_message("A", "B", "err")),
                SeqEvent::FragmentEnd,
            ],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.fragments.len(), 1);
        let frag = &layout.fragments[0];
        assert_eq!(frag.kind, FragmentKind::Alt);
        assert_eq!(frag.label, "success");
        assert!(frag.height > 0.0);
        assert!(frag.width > 0.0);
        assert_eq!(frag.separators.len(), 1);
        assert_eq!(frag.separators[0].1, "failure");
    }

    #[test]
    fn divider_creates_layout() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::Divider {
                text: Some("Phase 1".to_string()),
            }],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.dividers.len(), 1);
        assert_eq!(layout.dividers[0].text.as_deref(), Some("Phase 1"));
    }

    #[test]
    fn delay_creates_layout() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::Delay {
                text: Some("waiting".to_string()),
            }],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.delays.len(), 1);
        assert_eq!(layout.delays[0].text.as_deref(), Some("waiting"));
    }

    #[test]
    fn ref_creates_layout() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![SeqEvent::Ref {
                participants: vec!["A".to_string(), "B".to_string()],
                label: "init phase".to_string(),
            }],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.refs.len(), 1);
        assert_eq!(layout.refs[0].label, "init phase");
        assert!(layout.refs[0].width > 0.0);
    }

    #[test]
    fn spacing_advances_cursor() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![
                SeqEvent::Message(make_message("A", "B", "before")),
                SeqEvent::Spacing { pixels: 50 },
                SeqEvent::Message(make_message("A", "B", "after")),
            ],
        };
        let layout = layout_sequence(&sd).unwrap();

        assert_eq!(layout.messages.len(), 2);
        let gap = layout.messages[1].y - layout.messages[0].y;
        // gap should be at least MESSAGE_SPACING + 50
        assert!(
            gap >= MESSAGE_SPACING + 50.0 - 0.1,
            "gap {} should be at least {}",
            gap,
            MESSAGE_SPACING + 50.0
        );
    }

    #[test]
    fn note_right_expands_total_width() {
        // A single participant with a right note: the note should expand total_width
        let sd = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::NoteRight {
                participant: "A".to_string(),
                text: "a note".to_string(),
            }],
        };
        let layout = layout_sequence(&sd).unwrap();

        let note = &layout.notes[0];
        let note_right = note.x + note.width + MARGIN;
        // total_width must be at least as large as note_right
        assert!(
            layout.total_width >= note_right - 0.01,
            "total_width {:.1} should be >= note_right {:.1}",
            layout.total_width,
            note_right
        );

        // Also verify it's wider than participant-only width
        let participant_only_width = layout.participants[0].x
            + layout.participants[0].box_width / 2.0
            + 2.0 * MARGIN;
        assert!(
            layout.total_width > participant_only_width,
            "total_width {:.1} should exceed participant-only {:.1} due to note",
            layout.total_width,
            participant_only_width
        );
    }

    #[test]
    fn note_width_matches_text() {
        // Verify note width is computed based on text, not a fixed constant
        let short_text = "Hi";
        let long_text = "the location of the Comment is correct";
        let w_short = estimate_note_width(short_text);
        let w_long = estimate_note_width(long_text);
        assert!(
            w_long > w_short,
            "long note ({w_long:.1}) should be wider than short note ({w_short:.1})"
        );
        // minimum note width should be at least 30
        assert!(w_short >= 30.0, "short note width {w_short:.1} should be >= 30");
    }
}
