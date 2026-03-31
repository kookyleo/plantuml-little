// layout::sequence_teoz::builder - TileBuilder + PlayingSpace orchestration
//
// Port of Java PlantUML's TileBuilder, PlayingSpace, and
// SequenceDiagramFileMakerTeoz into a single build_teoz_layout() function.
//
// Pipeline:
//   1. Create RealLine (constraint arena)
//   2. Create LivingSpaces for each participant (with Real positions)
//   3. Build Tiles from events (TileBuilder logic)
//   4. Add constraints from tiles
//   5. Compile constraints (solve)
//   6. Assign Y positions (fillPositionelTiles)
//   7. Extract SeqLayout from positioned tiles

use std::collections::HashMap;

use crate::font_metrics;
use crate::model::sequence::{
    FragmentKind, ParticipantKind, SeqArrowHead, SeqArrowStyle, SeqDirection, SeqEvent,
    SequenceDiagram,
};
use crate::skin::rose::{self, TextMetrics};
use crate::style::SkinParams;
use crate::Result;

use crate::layout::sequence::{
    ActivationLayout, DelayLayout, DestroyLayout, DividerLayout, FragmentLayout, GroupLayout,
    MessageLayout, NoteLayout, ParticipantLayout, RefLayout, SeqLayout,
};

use super::living::LivingSpace;
use super::real::{RealId, RealLine};

// ── Constants ────────────────────────────────────────────────────────────────

const FONT_SIZE: f64 = 14.0;
const MSG_FONT_SIZE: f64 = 13.0;
const NOTE_FONT_SIZE: f64 = 13.0;
const ACTIVATION_WIDTH: f64 = 10.0;
const SELF_MSG_WIDTH: f64 = 42.0;
const NOTE_PADDING: f64 = rose::NOTE_PADDING;
const NOTE_FOLD: f64 = rose::SEQ_NOTE_FOLD;
/// Java Rose.paddingX = 5; ComponentRoseNote.getPreferredWidth includes 2*paddingX = 10
/// beyond the drawn polygon width (getTextWidth). The extent calculations must use
/// the full preferred width, not just the drawn width.
const NOTE_EXTENT_PADDING: f64 = 10.0;
/// Java teoz: PlayingSpace.startingY = 8 for tile positioning, but the
/// SVG coordinates include UTranslate(5,5) + defaultMargins(5,5,5,5),
/// giving an effective offset of 10 for participant heads and lifelines.
/// We use 10 here so SVG y coordinates match Java's output directly.
const STARTING_Y: f64 = 10.0;
/// Minimum gap between adjacent participant right-edge and next left-edge.
const PARTICIPANT_GAP: f64 = 5.0;
/// Java teoz: SequenceDiagramFileMakerTeoz applies UTranslate(5,5) to
/// the drawing, and SequenceDiagram.getDefaultMargins() returns (5,5,5,5).
/// Combined x_offset = 5 + 5 - min1 = 10 - min1.
/// Total viewport width = body_width + 10 + margins(5+5) = body_width + 20.
const DOC_MARGIN_X: f64 = 10.0;
/// Java: GroupingTile.MARGINX = 16 (internal padding between frame and content)
const GROUP_MARGINX: f64 = 16.0;
/// Java: GroupingTile.EXTERNAL_MARGINX1 = 3 (left frame margin)
const GROUP_EXTERNAL_MARGINX1: f64 = 3.0;
/// Java: GroupingTile.EXTERNAL_MARGINX2 = 9 (right frame margin)
const GROUP_EXTERNAL_MARGINX2: f64 = 9.0;
/// Java: PlayingSpace.startingY = 8. Tiles start at this offset within the PlayingSpace.
const PLAYINGSPACE_STARTING_Y: f64 = 8.0;

// ── Tile types (inline, simplified) ──────────────────────────────────────────

/// Simplified tile kind for the builder pipeline.
/// Each variant carries the data needed for constraint generation and
/// layout extraction. This will later be replaced by the full tile module.
#[derive(Debug)]
#[allow(dead_code)]
enum TeozTile {
    /// Normal message between two different participants
    Communication {
        from_name: String,
        to_name: String,
        from_idx: usize,
        to_idx: usize,
        text: String,
        text_lines: Vec<String>,
        is_dashed: bool,
        has_open_head: bool,
        arrow_head: SeqArrowHead,
        /// Minimum pixel width needed by the message text
        text_width: f64,
        /// Preferred height of this tile
        height: f64,
        /// Y position (assigned in step 6)
        y: Option<f64>,
        /// Autonumber label if any
        autonumber: Option<String>,
        /// RealId of the source participant center
        from_center: RealId,
        /// RealId of the target participant center
        to_center: RealId,
        /// Circle decoration on from end
        circle_from: bool,
        /// Circle decoration on to end
        circle_to: bool,
        /// Cross (X) decoration on from end
        cross_from: bool,
        /// Cross (X) decoration on to end
        cross_to: bool,
        /// Teoz parallel: shares y with previous tile
        is_parallel: bool,
        /// Activation level of the sender at this message
        from_level: usize,
        /// Activation level of the receiver at this message
        /// (IGNORE_FUTURE_DEACTIVATE: includes activations from this message)
        to_level: usize,
        /// Hidden arrow: occupies space but is not drawn
        hidden: bool,
        /// Bidirectional arrow: arrowheads at both ends
        bidirectional: bool,
    },
    /// Self-message (from == to)
    SelfMessage {
        participant_idx: usize,
        text: String,
        text_lines: Vec<String>,
        is_dashed: bool,
        has_open_head: bool,
        arrow_head: SeqArrowHead,
        text_width: f64,
        height: f64,
        y: Option<f64>,
        autonumber: Option<String>,
        center: RealId,
        direction: SeqDirection,
        is_reverse_define: bool,
        /// Activation level at the time of this self-message
        active_level: usize,
        /// Circle decoration on from end
        circle_from: bool,
        /// Circle decoration on to end
        circle_to: bool,
        /// Cross (X) decoration on from end
        cross_from: bool,
        /// Cross (X) decoration on to end
        cross_to: bool,
        /// Teoz parallel: shares y with previous tile
        is_parallel: bool,
        /// Hidden arrow: occupies space but is not drawn
        hidden: bool,
        /// Bidirectional arrow: arrowheads at both ends
        bidirectional: bool,
    },
    /// Activate / Deactivate / Destroy life event
    LifeEvent { height: f64, y: Option<f64> },
    /// Note on a participant
    Note {
        participant_idx: usize,
        text: String,
        is_left: bool,
        width: f64,
        height: f64,
        y: Option<f64>,
        center: RealId,
        /// True if this note follows a self-message (shares Y, no height contribution).
        is_note_on_message: bool,
    },
    /// Note spanning two participants
    NoteOver {
        participants: Vec<String>,
        text: String,
        width: f64,
        height: f64,
        y: Option<f64>,
    },
    /// Divider line
    Divider {
        text: Option<String>,
        height: f64,
        y: Option<f64>,
    },
    /// Delay section
    Delay {
        text: Option<String>,
        height: f64,
        y: Option<f64>,
    },
    /// Reference over participants
    Ref {
        participants: Vec<String>,
        label: String,
        height: f64,
        y: Option<f64>,
    },
    /// Fragment (alt/loop/opt/etc.) start
    FragmentStart {
        kind: FragmentKind,
        label: String,
        height: f64,
        y: Option<f64>,
        /// Teoz parallel: shares y with previous tile block
        is_parallel: bool,
    },
    /// Fragment separator (else)
    FragmentSeparator {
        label: String,
        height: f64,
        y: Option<f64>,
    },
    /// Fragment end
    FragmentEnd { height: f64, y: Option<f64> },
    /// Spacing
    Spacing { pixels: f64, y: Option<f64> },
    /// Group start (legacy)
    GroupStart {
        _label: Option<String>,
        height: f64,
        y: Option<f64>,
    },
    /// Group end (legacy)
    GroupEnd { height: f64, y: Option<f64> },
}

impl TeozTile {
    fn preferred_height(&self) -> f64 {
        match self {
            Self::Communication { height, .. } => *height,
            Self::SelfMessage { height, .. } => *height,
            Self::LifeEvent { height, .. } => *height,
            Self::Note { height, .. } => *height,
            Self::NoteOver { height, .. } => *height,
            Self::Divider { height, .. } => *height,
            Self::Delay { height, .. } => *height,
            Self::Ref { height, .. } => *height,
            Self::FragmentStart { height, .. } => *height,
            Self::FragmentSeparator { height, .. } => *height,
            Self::FragmentEnd { height, .. } => *height,
            Self::Spacing { pixels, .. } => *pixels,
            Self::GroupStart { height, .. } => *height,
            Self::GroupEnd { height, .. } => *height,
        }
    }

    fn set_y(&mut self, val: f64) {
        match self {
            Self::Communication { y, .. } => *y = Some(val),
            Self::SelfMessage { y, .. } => *y = Some(val),
            Self::LifeEvent { y, .. } => *y = Some(val),
            Self::Note { y, .. } => *y = Some(val),
            Self::NoteOver { y, .. } => *y = Some(val),
            Self::Divider { y, .. } => *y = Some(val),
            Self::Delay { y, .. } => *y = Some(val),
            Self::Ref { y, .. } => *y = Some(val),
            Self::FragmentStart { y, .. } => *y = Some(val),
            Self::FragmentSeparator { y, .. } => *y = Some(val),
            Self::FragmentEnd { y, .. } => *y = Some(val),
            Self::Spacing { y, .. } => *y = Some(val),
            Self::GroupStart { y, .. } => *y = Some(val),
            Self::GroupEnd { y, .. } => *y = Some(val),
        }
    }

    fn get_y(&self) -> Option<f64> {
        match self {
            Self::Communication { y, .. } => *y,
            Self::SelfMessage { y, .. } => *y,
            Self::LifeEvent { y, .. } => *y,
            Self::Note { y, .. } => *y,
            Self::NoteOver { y, .. } => *y,
            Self::Divider { y, .. } => *y,
            Self::Delay { y, .. } => *y,
            Self::Ref { y, .. } => *y,
            Self::FragmentStart { y, .. } => *y,
            Self::FragmentSeparator { y, .. } => *y,
            Self::FragmentEnd { y, .. } => *y,
            Self::Spacing { y, .. } => *y,
            Self::GroupStart { y, .. } => *y,
            Self::GroupEnd { y, .. } => *y,
        }
    }

    /// Java TileParallel contact-point alignment.
    /// Returns the distance from the tile top to the arrow "contact" point.
    /// For Communication tiles: `text_height + ARROW_PADDING_Y` = `height - 8`.
    /// For SelfMessage tiles: `text_height + 11.5` = `height - 13.5`.
    /// For non-message tiles: 0 (top-aligned).
    fn contact_point_relative(&self) -> f64 {
        match self {
            Self::Communication { height, .. } => {
                // height = tm.text_height() + ARROW_DELTA_Y + 2*ARROW_PADDING_Y
                // contact = tm.text_height() + ARROW_PADDING_Y
                height - (rose::ARROW_DELTA_Y + rose::ARROW_PADDING_Y)
            }
            Self::SelfMessage { height, .. } => {
                // height = tm.text_height() + ARROW_DELTA_Y + SELF_ARROW_ONLY_HEIGHT + 2*ARROW_PADDING_Y
                // Java: contact = getYPoint = (text_h + text_h + arrowOnly) / 2 + getPaddingX()
                // getPaddingX() = 0 for ComponentRoseSelfArrow (not ARROW_PADDING_X)
                let tm_text_h = height - rose::ARROW_DELTA_Y - rose::SELF_ARROW_ONLY_HEIGHT
                    - 2.0 * rose::ARROW_PADDING_Y;
                tm_text_h + rose::SELF_ARROW_ONLY_HEIGHT / 2.0
            }
            _ => 0.0,
        }
    }

    /// Distance from contact point to tile bottom (Java `getZZZ()`).
    fn zzz(&self) -> f64 {
        self.preferred_height() - self.contact_point_relative()
    }
}

/// Apply Java TileParallel contact-point alignment to a block of parallel tiles.
///
/// Each tile in a parallel block is shifted down by `(maxContact - itsContact)`
/// so that all arrows align at the same Y coordinate. This matches Java's
/// `TileParallel.drawU()` which translates each sub-tile by that delta.
fn apply_contact_point_alignment(tiles: &mut [TeozTile], indices: &[usize]) {
    if indices.len() <= 1 {
        return; // no alignment needed for single tiles
    }
    let max_contact = indices
        .iter()
        .map(|&i| tiles[i].contact_point_relative())
        .fold(0.0_f64, f64::max);
    for &i in indices {
        let contact = tiles[i].contact_point_relative();
        let shift = max_contact - contact;
        if shift > 0.0 {
            if let Some(old_y) = tiles[i].get_y() {
                tiles[i].set_y(old_y + shift);
            }
        }
    }
}

// ── Layout parameters ────────────────────────────────────────────────────────

#[allow(dead_code)]
struct TeozParams {
    message_spacing: f64,
    self_msg_height: f64,
    participant_height: f64,
    msg_line_height: f64,
    frag_header_height: f64,
    /// Java teoz ElseTile: ComponentRoseGroupingElse.getPreferredHeight() = textHeight + 16
    /// textHeight = textBlock.height + 2*marginY(1) = h11 + 2  (11pt style font)
    /// So frag_separator_height_teoz = h11 + 18
    frag_separator_height_teoz: f64,
    divider_height: f64,
    delay_height: f64,
    ref_height: f64,
}

impl TeozParams {
    fn compute(font_family: &str, msg_font_size: f64, part_font_size: f64) -> Self {
        let h13 = font_metrics::line_height(font_family, msg_font_size, false, false);
        let h14 = font_metrics::line_height(font_family, part_font_size, false, false);

        let arrow_tm = TextMetrics::new(7.0, 7.0, 1.0, 0.0, h13);
        let message_spacing = rose::arrow_preferred_size(&arrow_tm, 0.0, 0.0).height;

        let self_msg_height = rose::SELF_ARROW_ONLY_HEIGHT;

        // Java: ComponentRoseParticipant(style, stereo, NONE, 7, 7, 7, skinParam, display, false)
        // marginX1=7, marginX2=7, marginY=7
        // preferred_height = getTextHeight() + 1 = (lineHeight + 2*7) + 1 = 31.2969
        // But the DRAWN rect height = getTextHeight() = 30.2969 (no +1).
        // We use text_height (30.2969) as box_height for rendering consistency with puma.
        let part_tm = TextMetrics::new(7.0, 7.0, 7.0, 0.0, h14);
        let participant_preferred_h =
            rose::participant_preferred_size(&part_tm, 0.0, false, 0.0, 0.0).height;
        let participant_height = participant_preferred_h - 1.0; // text_height only (drawn rect)

        let frag_header_height = h13 + 2.0;
        // Java teoz: ElseTile preferred height = getTextHeight + 16
        // Java ElseTile uses ComponentRoseGroupingElse with style font (11pt), not 13pt.
        // getTextHeight = textBlock.height + 2*marginY(1) = h11 + 2
        let h11 = font_metrics::line_height(font_family, 11.0, false, false);
        let frag_separator_height_teoz = h11 + 18.0;

        let divider_tm = TextMetrics::new(0.0, 0.0, 5.0, 0.0, 0.0);
        let divider_height = rose::divider_preferred_size(&divider_tm).height;

        let delay_tm = TextMetrics::new(0.0, 0.0, 5.0, 0.0, 0.0);
        let delay_height = rose::delay_text_preferred_size(&delay_tm).height;

        let ref_height = h13 + h14 + rose::REF_HEIGHT_FOOTER + 2.0 + 0.671875;

        Self {
            message_spacing,
            self_msg_height,
            participant_height,
            msg_line_height: h13,
            frag_header_height,
            frag_separator_height_teoz,
            divider_height,
            delay_height,
            ref_height,
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn active_left_shift(level: usize) -> f64 {
    if level == 0 {
        0.0
    } else {
        ACTIVATION_WIDTH / 2.0
    }
}

fn active_right_shift(level: usize) -> f64 {
    level as f64 * (ACTIVATION_WIDTH / 2.0)
}

/// Unescape PlantUML text escape sequences after \\n splitting.
/// Java: Display.create() processes \\-prefixed escapes in legacy mode:
///   `\\\\` -> `\`, `\\t` -> tab. Other `\\X` pass through.
fn unescape_backslash(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                '\\' => {
                    result.push('\\');
                    i += 2;
                }
                't' => {
                    result.push('\t');
                    i += 2;
                }
                _ => {
                    result.push(chars[i]);
                    result.push(chars[i + 1]);
                    i += 2;
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn live_thickness_width(level: usize) -> f64 {
    active_left_shift(level) + active_right_shift(level)
}

/// Unified extent calculation for a self-message tile, matching Java's
/// CommunicationTileSelf.getMinX() / getMaxX().
///
/// Returns `(min_x, max_x)` in Real coordinate space (before x_offset).
///
/// Java logic:
///   Forward (L→R):  minX = posC,  maxX = posC2 + compWidth
///   Reverse (R→L):  minX = posC - compWidth - liveDeltaAdj,  maxX = posC2
///   where posC2 = posC + active_right_shift(level)
///         liveDeltaAdj = if level > 0 { LIVE_DELTA_SIZE } else { 0 }
///         LIVE_DELTA_SIZE = 5.0 (CommunicationTile.LIVE_DELTA_SIZE)
fn self_message_extent(
    center_x: f64,
    comp_width: f64,
    active_level: usize,
    direction: &SeqDirection,
) -> (f64, f64) {
    const LIVE_DELTA_SIZE: f64 = 5.0;
    let pos_c2 = center_x + active_right_shift(active_level);
    match direction {
        SeqDirection::LeftToRight => (center_x, pos_c2 + comp_width),
        SeqDirection::RightToLeft => {
            let live_delta_adj = if active_level > 0 {
                LIVE_DELTA_SIZE
            } else {
                0.0
            };
            (center_x - comp_width - live_delta_adj, pos_c2)
        }
    }
}

/// Compute the contact point for a self-message (or normal message).
/// Java: CommunicationTileSelf.getContactPointRelative()
///   = component.getYPoint() = (textHeight + textAndArrowHeight) / 2
/// For normal messages: ContactPointRelative = arrowY = textHeight + paddingY
fn compute_selfmsg_contact(tile: &TeozTile, msg_line_height: f64) -> f64 {
    match tile {
        TeozTile::SelfMessage { .. } => {
            // Java: (textHeight + textAndArrowHeight) / 2 + paddingX(=0)
            // textHeight = lineHeight + 2*marginY(1) = h13 + 2
            // textAndArrowHeight = textHeight + arrowOnlyHeight(13)
            let text_height = msg_line_height + 2.0;
            let text_and_arrow_h = text_height + rose::SELF_ARROW_ONLY_HEIGHT;
            (text_height + text_and_arrow_h) / 2.0
        }
        TeozTile::Communication { .. } => {
            // Java: ComponentRoseArrow.getYPoint = textHeight + paddingY(4)
            let text_height = msg_line_height + 2.0;
            text_height + rose::ARROW_PADDING_Y
        }
        _ => 0.0,
    }
}

/// Compute the component width for a self-message tile given its text_width
/// and message line height.
fn self_message_comp_width(text_width: f64, msg_line_height: f64) -> f64 {
    let tm = TextMetrics::new(7.0, 7.0, 1.0, text_width, msg_line_height);
    rose::self_arrow_preferred_size(&tm).width
}

/// Find the most recent SelfMessage tile before `tile_index` (skipping LifeEvent tiles)
/// and return its (participant_idx, text_width, direction, active_level).
fn find_preceding_self_message(
    tiles: &[TeozTile],
    tile_index: usize,
) -> Option<(usize, f64, SeqDirection, usize)> {
    for i in (0..tile_index).rev() {
        match &tiles[i] {
            TeozTile::SelfMessage {
                participant_idx,
                text_width,
                direction,
                active_level,
                ..
            } => {
                return Some((
                    *participant_idx,
                    *text_width,
                    direction.clone(),
                    *active_level,
                ));
            }
            TeozTile::LifeEvent { .. } => continue,
            _ => return None,
        }
    }
    None
}

/// Drawn polygon height for the note (SVG rendering).
/// Java: `(int) getTextHeight()` where `getTextHeight = textBlock.h + 2*marginY(5)`.
fn estimate_note_height(text: &str) -> f64 {
    let lines = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.lines())
        .count()
        .max(1) as f64;
    let lh = font_metrics::line_height("SansSerif", NOTE_FONT_SIZE, false, false);
    let creole_extra = creole_note_extra_height(text);
    let h = lines * lh + 10.0 + creole_extra; // marginY1(5) + marginY2(5)
    h.trunc().max(25.0)
}

/// Preferred height for note tile spacing (Y advancement).
/// Java: `ComponentRoseNote.getPreferredHeight()`
///   = `getTextHeight() + 2*paddingY + deltaShadow`
///   = `(textBlock.h + 2*marginY(5)) + 2*paddingY(5) + 0`
///   = `textBlock.h + 20`
/// This is larger than the drawn polygon height by 2*paddingY(=10).
fn note_preferred_height(text: &str, delta_shadow: f64) -> f64 {
    let lines = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.lines())
        .count()
        .max(1) as f64;
    let lh = font_metrics::line_height("SansSerif", NOTE_FONT_SIZE, false, false);
    let creole_extra = creole_note_extra_height(text);
    // getTextHeight = textBlock.h + 2*marginY(5)
    // getPreferredHeight = getTextHeight + 2*paddingY(5) + deltaShadow
    lines * lh + 20.0 + creole_extra + delta_shadow
}

/// Estimate extra height added by creole formatting in note text.
/// Tables get +4px padding per row + 6px border overhead.
/// Horizontal separators (`----` or `====`) add ~8px each.
fn creole_note_extra_height(text: &str) -> f64 {
    let mut extra = 0.0;
    let mut in_table = false;
    let mut table_rows = 0;
    for line in text.split(crate::NEWLINE_CHAR).flat_map(|s| s.lines()) {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2 {
            if !in_table {
                in_table = true;
                table_rows = 0;
            }
            table_rows += 1;
        } else {
            if in_table {
                extra += table_rows as f64 * 4.0 + 6.0;
                in_table = false;
            }
            if trimmed == "----"
                || trimmed == "===="
                || trimmed.starts_with("----")
                || trimmed.starts_with("====")
            {
                extra += 8.0;
            }
        }
    }
    if in_table {
        extra += table_rows as f64 * 4.0 + 6.0;
    }
    extra
}

fn estimate_note_width(text: &str) -> f64 {
    let max_line_w = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.lines())
        .map(|line| font_metrics::text_width(line, "SansSerif", NOTE_FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    let w = max_line_w + NOTE_PADDING + NOTE_PADDING / 2.0 + NOTE_FOLD + 2.0;
    w.max(30.0)
}

/// Compute per-fragment (min_x, max_x) extent from child tiles within the
/// range [start_idx..end_idx) in raw coordinate space.
/// This matches Java GroupingTile which computes its own min/max from children,
/// recursively including nested fragment extents.
fn compute_fragment_extent(
    tiles: &[TeozTile],
    start_idx: usize,
    end_idx: usize,
    livings: &[LivingSpace],
    rl: &RealLine,
    tp: &TeozParams,
) -> (f64, f64) {
    let mut fmin = f64::MAX;
    let mut fmax = f64::MIN;
    let mut i = start_idx;

    while i < end_idx {
        let tile = &tiles[i];
        match tile {
            TeozTile::FragmentStart { .. } | TeozTile::GroupStart { .. } => {
                // Recursively compute nested fragment extent.
                // Find matching end by counting depth.
                let nested_start = i + 1;
                let mut depth = 1usize;
                let mut nested_end = i + 1;
                while nested_end < end_idx && depth > 0 {
                    match &tiles[nested_end] {
                        TeozTile::FragmentStart { .. } | TeozTile::GroupStart { .. } => {
                            depth += 1;
                        }
                        TeozTile::FragmentEnd { .. } | TeozTile::GroupEnd { .. } => {
                            depth -= 1;
                        }
                        _ => {}
                    }
                    nested_end += 1;
                }
                // nested_end is now past the matching end tile
                let (child_min, child_max) =
                    compute_fragment_extent(tiles, nested_start, nested_end - 1, livings, rl, tp);
                // Java: parent sees nested fragment as tile.getMinX() and tile.getMaxX()
                // getMinX = this.min - EXTERNAL_MARGINX1, getMaxX = this.max + EXTERNAL_MARGINX2
                // Then parent applies tile.getMinX() - MARGINX and tile.getMaxX() + MARGINX
                // BUT: the bottom fmin -= GROUP_MARGINX applies the parent MARGINX uniformly.
                // So here we only need EXTERNAL margins. The bottom MARGINX handles the rest.
                let child_with_margin_min = child_min - GROUP_EXTERNAL_MARGINX1;
                let child_with_margin_max = child_max + GROUP_EXTERNAL_MARGINX2;
                if child_with_margin_min < fmin {
                    fmin = child_with_margin_min;
                }
                if child_with_margin_max > fmax {
                    fmax = child_with_margin_max;
                }
                // Also include the nested fragment's header label width
                // Java: max candidate = min + dim1.getWidth() + 16
                if let TeozTile::FragmentStart { label, kind, .. } = tile {
                    let kind_text_w = crate::font_metrics::text_width(
                        kind.label(),
                        "sans-serif",
                        13.0,
                        true,
                        false,
                    );
                    let header_width = if label.is_empty() {
                        kind_text_w + 45.0
                    } else {
                        let bracket_label = format!("[{}]", label);
                        let comment_w = crate::font_metrics::text_width(
                            &bracket_label,
                            "sans-serif",
                            11.0,
                            true,
                            false,
                        );
                        kind_text_w + 45.0 + 15.0 + comment_w
                    };
                    let header_right = child_min + header_width + 16.0;
                    if header_right > fmax {
                        fmax = header_right;
                    }
                }
                i = nested_end;
                continue;
            }
            TeozTile::FragmentEnd { .. } | TeozTile::GroupEnd { .. } => {
                i += 1;
                continue;
            }
            _ => {}
        }
        match tile {
            TeozTile::Communication {
                from_idx, to_idx, ..
            } => {
                let from_x = rl.get_value(livings[*from_idx].pos_c);
                let to_x = rl.get_value(livings[*to_idx].pos_c);
                let t_min = f64::min(from_x, to_x);
                let t_max = f64::max(from_x, to_x);
                if t_min < fmin {
                    fmin = t_min;
                }
                if t_max > fmax {
                    fmax = t_max;
                }
            }
            TeozTile::SelfMessage {
                participant_idx,
                text_width,
                direction,
                active_level,
                ..
            } => {
                let cx = rl.get_value(livings[*participant_idx].pos_c);
                let comp_w = self_message_comp_width(*text_width, tp.msg_line_height);
                let (t_min, t_max) = self_message_extent(cx, comp_w, *active_level, direction);
                if t_min < fmin {
                    fmin = t_min;
                }
                if t_max > fmax {
                    fmax = t_max;
                }
            }
            TeozTile::Note {
                participant_idx,
                is_left,
                width,
                is_note_on_message,
                ..
            } => {
                let cx = rl.get_value(livings[*participant_idx].pos_c);
                let extent_w = *width + NOTE_EXTENT_PADDING;
                if *is_note_on_message {
                    // Note on self-message: use self-message extent as base
                    if let Some((sm_pidx, sm_tw, sm_dir, sm_al)) =
                        find_preceding_self_message(tiles, i)
                    {
                        let sm_cx = rl.get_value(livings[sm_pidx].pos_c);
                        let sm_comp_w = self_message_comp_width(sm_tw, tp.msg_line_height);
                        let (sm_min, sm_max) =
                            self_message_extent(sm_cx, sm_comp_w, sm_al, &sm_dir);
                        let (t_min, t_max) = if *is_left {
                            (sm_min - extent_w, sm_max)
                        } else {
                            (sm_min, sm_max + extent_w)
                        };
                        if t_min < fmin {
                            fmin = t_min;
                        }
                        if t_max > fmax {
                            fmax = t_max;
                        }
                    } else {
                        // Fallback to cx-based
                        if *is_left {
                            let left = cx - extent_w - 5.0;
                            if left < fmin {
                                fmin = left;
                            }
                            if cx > fmax {
                                fmax = cx;
                            }
                        } else {
                            let right = cx + extent_w;
                            if right > fmax {
                                fmax = right;
                            }
                            if cx < fmin {
                                fmin = cx;
                            }
                        }
                    }
                } else {
                    if *is_left {
                        let left = cx - extent_w - 5.0;
                        if left < fmin {
                            fmin = left;
                        }
                        if cx > fmax {
                            fmax = cx;
                        }
                    } else {
                        let right = cx + extent_w;
                        if right > fmax {
                            fmax = right;
                        }
                        if cx < fmin {
                            fmin = cx;
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Collect fragment separator (else) labels for width contribution.
    // Java: ElseTile.getMaxX() = parent.getMinX() + elseComponentWidth
    // where elseComponentWidth = pureTextWidth + marginX1(5) + marginX2(5)
    let mut else_labels: Vec<String> = Vec::new();
    {
        let mut sep_depth: usize = 0;
        for j in start_idx..end_idx {
            match &tiles[j] {
                TeozTile::FragmentStart { .. } | TeozTile::GroupStart { .. } => {
                    sep_depth += 1;
                }
                TeozTile::FragmentEnd { .. } | TeozTile::GroupEnd { .. } => {
                    if sep_depth > 0 {
                        sep_depth -= 1;
                    }
                }
                TeozTile::FragmentSeparator { label, .. } if sep_depth == 0 => {
                    else_labels.push(label.clone());
                }
                _ => {}
            }
        }
    }

    // Fallback if no children found
    if fmin == f64::MAX {
        fmin = 0.0;
    }
    if fmax == f64::MIN {
        fmax = 0.0;
    }

    // Apply GroupingTile MARGINX (internal padding between frame and content)
    fmin -= GROUP_MARGINX;
    fmax += GROUP_MARGINX;

    // Add else separator width contributions.
    // Java: ElseTile.getMaxX() = parent.getMinX() + elseComponentWidth
    // parent.getMinX() = this.min = fmin (after MARGINX)
    // Java ComponentRoseGroupingElse wraps label in brackets: "[label]"
    // and uses marginX1=5, marginX2=5, 11pt bold font.
    for label in &else_labels {
        let bracket_label = format!("[{}]", label);
        let pure_text_w =
            crate::font_metrics::text_width(&bracket_label, "sans-serif", 11.0, true, false);
        // Java ComponentRoseGroupingElse: marginX1=5, marginX2=5
        let else_width = pure_text_w + 10.0;
        let else_max = fmin + else_width;
        if else_max > fmax {
            fmax = else_max;
        }
    }
    (fmin, fmax)
}

#[allow(dead_code)]
fn message_text_width(text: &str, font_family: &str, font_size: f64) -> f64 {
    text.split("\\n")
        .flat_map(|s| s.split(crate::NEWLINE_CHAR))
        .map(|line| font_metrics::text_width(line, font_family, font_size, false, false))
        .fold(0.0_f64, f64::max)
}

// ── Note-on-message helper ───────────────────────────────────────────────────

/// Check if the last non-LifeEvent tile in the list is a SelfMessage.
fn is_last_tile_self_message(tiles: &[TeozTile]) -> bool {
    for tile in tiles.iter().rev() {
        match tile {
            TeozTile::SelfMessage { .. } => return true,
            TeozTile::LifeEvent { .. } => continue,
            _ => return false,
        }
    }
    false
}

/// Check if the last non-LifeEvent tile is any message (Communication or SelfMessage).
/// Used for note-on-message binding.
fn is_last_tile_any_message(tiles: &[TeozTile]) -> bool {
    for tile in tiles.iter().rev() {
        match tile {
            TeozTile::Communication { .. } | TeozTile::SelfMessage { .. } => return true,
            TeozTile::LifeEvent { .. } => continue,
            _ => return false,
        }
    }
    false
}

// ── Main build function ──────────────────────────────────────────────────────

/// Build the complete Teoz layout from a parsed sequence diagram.
///
/// This is the main orchestrator matching Java's
/// SequenceDiagramFileMakerTeoz + PlayingSpace + TileBuilder.
pub fn build_teoz_layout(sd: &SequenceDiagram, skin: &SkinParams) -> Result<SeqLayout> {
    log::debug!(
        "build_teoz_layout: {} participants, {} events",
        sd.participants.len(),
        sd.events.len(),
    );

    // ── Resolve font/skin params ─────────────────────────────────────────
    let default_font = skin
        .get("defaultfontname")
        .map(|s| s.as_ref())
        .unwrap_or("SansSerif");
    let default_font_size: Option<f64> = skin
        .get("defaultfontsize")
        .and_then(|s| s.parse::<f64>().ok());
    let msg_font_size: f64 = default_font_size.unwrap_or(MSG_FONT_SIZE);
    let participant_font_size: f64 = skin
        .get("participantfontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .or(default_font_size)
        .unwrap_or(FONT_SIZE);
    let max_message_size: Option<f64> = skin
        .get("maxmessagesize")
        .and_then(|s| s.parse::<f64>().ok());

    let tp = TeozParams::compute(default_font, msg_font_size, participant_font_size);

    // ── Step 1: Create RealLine ──────────────────────────────────────────
    let mut rl = RealLine::new();
    let xorigin = rl.create_origin();

    // ── Step 2: Create LivingSpaces ──────────────────────────────────────
    // For each participant, compute box width/height and create Real
    // constraint variables for posB (left), posC (center), posD (right).
    let n_parts = sd.participants.len();
    let mut livings: Vec<LivingSpace> = Vec::with_capacity(n_parts);
    let mut part_layouts: Vec<ParticipantLayout> = Vec::with_capacity(n_parts);
    let mut box_widths: Vec<f64> = Vec::with_capacity(n_parts);
    let mut box_heights: Vec<f64> = Vec::with_capacity(n_parts);
    let mut name_to_idx: HashMap<String, usize> = HashMap::new();

    let mut xcurrent = rl.add_at_least(xorigin, 0.0);

    for (i, p) in sd.participants.iter().enumerate() {
        let display = p.display_name.as_deref().unwrap_or(&p.name);
        let display_lines: Vec<&str> = display
            .split("\\n")
            .flat_map(|s| s.split(crate::NEWLINE_CHAR))
            .collect();
        let num_lines = display_lines.len();
        let max_line_w = display_lines
            .iter()
            .map(|line| {
                font_metrics::text_width(line, default_font, participant_font_size, false, false)
            })
            .fold(0.0_f64, f64::max);
        let bw = rose::participant_preferred_width(&p.kind, max_line_w, 1.5);
        let participant_line_height =
            font_metrics::line_height(default_font, participant_font_size, false, false);
        let multiline_extra = if num_lines > 1 {
            participant_line_height * (num_lines - 1) as f64
        } else {
            0.0
        };
        let base_participant_height = tp.participant_height;
        let bh = match p.kind {
            ParticipantKind::Actor => base_participant_height + 45.0 + multiline_extra,
            ParticipantKind::Boundary
            | ParticipantKind::Control
            | ParticipantKind::Entity
            | ParticipantKind::Database
            | ParticipantKind::Collections
            | ParticipantKind::Queue => base_participant_height + 20.0 + multiline_extra,
            ParticipantKind::Default => base_participant_height + multiline_extra,
        };

        // Create Real variables: posB = xcurrent, posC = posB + w/2, posD = posB + w
        let pos_b = xcurrent;
        let half_w = bw / 2.0;
        let pos_c = rl.add_fixed(pos_b, half_w);
        let pos_d = rl.add_fixed(pos_b, bw);

        livings.push(LivingSpace::new(p.name.clone(), pos_b, pos_c, pos_d));
        box_widths.push(bw);
        box_heights.push(bh);
        name_to_idx.insert(p.name.clone(), i);

        // Next participant starts after posD.
        // Java teoz: xcurrent = livingSpace.getPosD().addAtLeast(0);
        xcurrent = rl.add_at_least(pos_d, 0.0);
    }

    // ── Step 2b: Add inter-participant constraints ───────────────────────
    // Java: LivingSpaces.addConstraints() ensures posA_next >= posE_prev + 10
    // where posA = posB - marginBefore, posE = posD + marginAfter.
    // With default margins of 0, this adds 10px gap between adjacent boxes.
    for i in 1..livings.len() {
        let prev_pos_d = livings[i - 1].pos_d;
        let curr_pos_b = livings[i].pos_b;
        rl.ensure_bigger_than_with_margin(curr_pos_b, prev_pos_d, 10.0);
    }

    // ── Step 3: Build tiles from events ──────────────────────────────────
    let mut tiles: Vec<TeozTile> = Vec::new();
    let mut autonumber_enabled = false;
    let mut autonumber_counter: u32 = 1;
    let mut autonumber_start: u32 = 1;
    let mut active_levels: HashMap<String, usize> = HashMap::new();

    for (event_idx, event) in sd.events.iter().enumerate() {
        match event {
            SeqEvent::AutoNumber { start } => {
                autonumber_enabled = true;
                if let Some(n) = start {
                    autonumber_counter = *n;
                    autonumber_start = *n;
                }
            }
            SeqEvent::Message(msg) => {
                let autonumber = if autonumber_enabled {
                    let label = format!("{autonumber_counter}");
                    autonumber_counter += 1;
                    Some(label)
                } else {
                    None
                };

                let autonumber_extra_w = autonumber.as_ref().map_or(0.0, |num| {
                    font_metrics::text_width(num, default_font, msg_font_size, true, false) + 4.0
                });

                let mut text_lines: Vec<String> = msg
                    .text
                    .split("\\n")
                    .flat_map(|s| s.split(crate::NEWLINE_CHAR))
                    .map(|s| unescape_backslash(s))
                    .collect();
                if let Some(max_w) = max_message_size {
                    text_lines = text_lines
                        .into_iter()
                        .flat_map(|line| {
                            wrap_text_to_width(&line, max_w, default_font, msg_font_size)
                        })
                        .collect();
                }
                let text_w = text_lines
                    .iter()
                    .map(|line| {
                        crate::render::svg_richtext::creole_text_width(
                            line,
                            default_font,
                            msg_font_size,
                            false,
                            false,
                        )
                    })
                    .fold(0.0_f64, f64::max)
                    + autonumber_extra_w;

                let text_h = tp.msg_line_height * text_lines.len().max(1) as f64;
                let is_dashed = msg.arrow_style == SeqArrowStyle::Dashed;
                let has_open_head = matches!(
                    msg.arrow_head,
                    SeqArrowHead::Open | SeqArrowHead::HalfTop | SeqArrowHead::HalfBottom
                );

                // Skip boundary/gate messages: "[" and "]" are not real participants.
                // They are drawn at the diagram edges and should not create constraints.
                let is_boundary_from = msg.from == "[";
                let is_boundary_to = msg.to == "]";
                if is_boundary_from || is_boundary_to {
                    // Boundary messages: create a Communication tile from/to the
                    // nearest edge participant, but don't add participant constraints.
                    let real_from = if is_boundary_from {
                        // [-> goes to the target; the "from" is the left edge
                        0 // first participant
                    } else {
                        name_to_idx.get(&msg.from).copied().unwrap_or(0)
                    };
                    let real_to = if is_boundary_to {
                        // ->] goes from the source; the "to" is the right edge
                        livings.len().saturating_sub(1)
                    } else {
                        name_to_idx.get(&msg.to).copied().unwrap_or(0)
                    };
                    let from_center = livings[real_from].pos_c;
                    let to_center = livings[real_to].pos_c;
                    let tm = TextMetrics::new(7.0, 7.0, 1.0, text_w, text_h);
                    let height = rose::arrow_preferred_size(&tm, 0.0, 0.0).height;
                    tiles.push(TeozTile::Communication {
                        from_name: msg.from.clone(),
                        to_name: msg.to.clone(),
                        from_idx: real_from,
                        to_idx: real_to,
                        text: msg.text.clone(),
                        text_lines,
                        is_dashed,
                        has_open_head,
                        arrow_head: msg.arrow_head.clone(),
                        text_width: text_w,
                        height,
                        y: None,
                        autonumber,
                        from_center,
                        to_center,
                        circle_from: msg.circle_from,
                        circle_to: msg.circle_to,
                        cross_from: msg.cross_from,
                        cross_to: msg.cross_to,
                        is_parallel: msg.parallel,
                        from_level: 0,
                        to_level: 0,
                        hidden: msg.hidden,
                        bidirectional: msg.bidirectional,
                    });
                    continue;
                }

                if msg.from == msg.to {
                    // Self-message
                    let idx = name_to_idx.get(&msg.from).copied().unwrap_or(0);
                    let center = livings[idx].pos_c;
                    let tm = TextMetrics::new(7.0, 7.0, 1.0, text_w, text_h);
                    let height = rose::self_arrow_preferred_size(&tm).height;

                    let level = active_levels.get(&msg.from).copied().unwrap_or(0);
                    tiles.push(TeozTile::SelfMessage {
                        participant_idx: idx,
                        text: msg.text.clone(),
                        text_lines,
                        is_dashed,
                        has_open_head,
                        arrow_head: msg.arrow_head.clone(),
                        text_width: text_w,
                        height,
                        y: None,
                        autonumber,
                        center,
                        direction: msg.direction.clone(),
                        is_reverse_define: msg.is_reverse_define,
                        active_level: level,
                        circle_from: msg.circle_from,
                        circle_to: msg.circle_to,
                        cross_from: msg.cross_from,
                        cross_to: msg.cross_to,
                        is_parallel: msg.parallel,
                        hidden: msg.hidden,
                        bidirectional: msg.bidirectional,
                    });
                } else {
                    // Normal message
                    let fi = name_to_idx.get(&msg.from).copied().unwrap_or(0);
                    let ti = name_to_idx.get(&msg.to).copied().unwrap_or(0);
                    let from_center = livings[fi].pos_c;
                    let to_center = livings[ti].pos_c;

                    let tm = TextMetrics::new(7.0, 7.0, 1.0, text_w, text_h);
                    let height = rose::arrow_preferred_size(&tm, 0.0, 0.0).height;

                    // Java IGNORE_FUTURE_DEACTIVATE: current levels +
                    // peek-ahead activations from this message (but not deactivations).
                    let mut fl = active_levels.get(&msg.from).copied().unwrap_or(0);
                    let mut tl = active_levels.get(&msg.to).copied().unwrap_or(0);
                    for peek in &sd.events[(event_idx + 1)..] {
                        match peek {
                            SeqEvent::Activate(name, _) => {
                                if name == &msg.from {
                                    fl += 1;
                                }
                                if name == &msg.to {
                                    tl += 1;
                                }
                            }
                            // Ignore future deactivations
                            SeqEvent::Deactivate(_) => {}
                            _ => break, // stop peeking at next non-life event
                        }
                    }

                    tiles.push(TeozTile::Communication {
                        from_name: msg.from.clone(),
                        to_name: msg.to.clone(),
                        from_idx: fi,
                        to_idx: ti,
                        text: msg.text.clone(),
                        text_lines,
                        is_dashed,
                        has_open_head,
                        arrow_head: msg.arrow_head.clone(),
                        text_width: text_w,
                        height,
                        y: None,
                        autonumber,
                        from_center,
                        to_center,
                        circle_from: msg.circle_from,
                        circle_to: msg.circle_to,
                        cross_from: msg.cross_from,
                        cross_to: msg.cross_to,
                        is_parallel: msg.parallel,
                        from_level: fl,
                        to_level: tl,
                        hidden: msg.hidden,
                        bidirectional: msg.bidirectional,
                    });
                }
            }
            SeqEvent::Activate(name, _act_color) => {
                let level = active_levels.entry(name.clone()).or_insert(0);
                *level += 1;
                tiles.push(TeozTile::LifeEvent {
                    height: 0.0,
                    y: None,
                });
            }
            SeqEvent::Deactivate(name) => {
                let level = active_levels.entry(name.clone()).or_insert(0);
                if *level > 0 {
                    *level -= 1;
                }
                tiles.push(TeozTile::LifeEvent {
                    height: 0.0,
                    y: None,
                });
            }
            SeqEvent::Destroy(_name) => {
                tiles.push(TeozTile::LifeEvent {
                    height: 0.0,
                    y: None,
                });
            }
            SeqEvent::NoteRight { participant, text } => {
                let idx = name_to_idx.get(participant).copied().unwrap_or(0);
                let center = livings[idx].pos_c;
                let w = estimate_note_width(text);
                let h = note_preferred_height(text, sd.delta_shadow);
                let is_smn = is_last_tile_any_message(&tiles);
                tiles.push(TeozTile::Note {
                    participant_idx: idx,
                    text: text.clone(),
                    is_left: false,
                    width: w,
                    height: h,
                    y: None,
                    center,
                    is_note_on_message: is_smn,
                });
            }
            SeqEvent::NoteLeft { participant, text } => {
                let idx = name_to_idx.get(participant).copied().unwrap_or(0);
                let center = livings[idx].pos_c;
                let w = estimate_note_width(text);
                let h = note_preferred_height(text, sd.delta_shadow);
                let is_smn = is_last_tile_any_message(&tiles);
                tiles.push(TeozTile::Note {
                    participant_idx: idx,
                    text: text.clone(),
                    is_left: true,
                    width: w,
                    height: h,
                    y: None,
                    center,
                    is_note_on_message: is_smn,
                });
            }
            SeqEvent::NoteOver { participants, text } => {
                let w = estimate_note_width(text);
                let h = note_preferred_height(text, sd.delta_shadow);
                tiles.push(TeozTile::NoteOver {
                    participants: participants.clone(),
                    text: text.clone(),
                    width: w,
                    height: h,
                    y: None,
                });
            }
            SeqEvent::Divider { text } => {
                tiles.push(TeozTile::Divider {
                    text: text.clone(),
                    height: tp.divider_height,
                    y: None,
                });
            }
            SeqEvent::Delay { text } => {
                tiles.push(TeozTile::Delay {
                    text: text.clone(),
                    height: tp.delay_height,
                    y: None,
                });
            }
            SeqEvent::Ref {
                participants,
                label,
            } => {
                tiles.push(TeozTile::Ref {
                    participants: participants.clone(),
                    label: label.clone(),
                    height: tp.ref_height,
                    y: None,
                });
            }
            SeqEvent::FragmentStart {
                kind,
                label,
                parallel,
            } => {
                // Java GroupingTile header: dim1.height + MARGINY_MAGIC/2
                // dim1.height = frag_header_height, MARGINY_MAGIC/2 = 10
                tiles.push(TeozTile::FragmentStart {
                    kind: kind.clone(),
                    label: label.clone(),
                    height: tp.frag_header_height + 10.0,
                    y: None,
                    is_parallel: *parallel,
                });
            }
            SeqEvent::FragmentSeparator { label } => {
                // Java teoz: ElseTile preferred height = textHeight + 16
                // textHeight = textBlock.height + 2*marginY(1) = h13 + 2
                tiles.push(TeozTile::FragmentSeparator {
                    label: label.clone(),
                    height: tp.frag_separator_height_teoz,
                    y: None,
                });
            }
            SeqEvent::FragmentEnd => {
                tiles.push(TeozTile::FragmentEnd {
                    height: 4.0,
                    y: None,
                });
            }
            SeqEvent::Spacing { pixels } => {
                tiles.push(TeozTile::Spacing {
                    pixels: *pixels as f64,
                    y: None,
                });
            }
            SeqEvent::GroupStart { label } => {
                tiles.push(TeozTile::GroupStart {
                    _label: label.clone(),
                    height: tp.frag_header_height + 10.0,
                    y: None,
                });
            }
            SeqEvent::GroupEnd => {
                tiles.push(TeozTile::GroupEnd {
                    height: 4.0,
                    y: None,
                });
            }
        }
    }

    // ── Step 4: Add constraints from tiles ───────────────────────────────
    // Communication tiles constrain participant spacing.
    // Java: CommunicationTile.addConstraints() does
    //   target_center >= source_center + arrow_preferred_width
    for tile in &tiles {
        match tile {
            TeozTile::Communication {
                from_idx,
                to_idx,
                from_name,
                to_name,
                text_width,
                from_center,
                to_center,
                from_level,
                to_level,
                ..
            } => {
                if from_name == "[" {
                    // Left-border messages: Java CommunicationExoTile.addConstraints()
                    // posC >= xOrigin + arrowWidth
                    let arrow_tm = TextMetrics::new(7.0, 7.0, 1.0, *text_width, tp.msg_line_height);
                    let arrow_w = rose::arrow_preferred_size(&arrow_tm, 0.0, 0.0).width;
                    rl.ensure_bigger_than_with_margin(*to_center, xorigin, arrow_w);
                } else if to_name == "]" {
                    // Right-border messages: no constraint in Java
                } else {
                    let fi = *from_idx;
                    let ti = *to_idx;
                    let arrow_tm = TextMetrics::new(7.0, 7.0, 1.0, *text_width, tp.msg_line_height);
                    let arrow_w = rose::arrow_preferred_size(&arrow_tm, 0.0, 0.0).width;

                    // Java CommunicationTile.addConstraints():
                    // Uses per-tile activation levels (IGNORE_FUTURE_DEACTIVATE),
                    // stored on the tile during construction.
                    const LIVE_DELTA_SIZE: f64 = 5.0;

                    if fi < ti {
                        let ti_adj = if *to_level > 0 { LIVE_DELTA_SIZE } else { 0.0 };
                        let needed = arrow_w + ti_adj;
                        rl.ensure_bigger_than_with_margin(*to_center, *from_center, needed);
                    } else {
                        let fi_adj = if *from_level > 0 {
                            LIVE_DELTA_SIZE
                        } else {
                            0.0
                        };
                        let ti_adj = *to_level as f64 * LIVE_DELTA_SIZE;
                        let needed = arrow_w + fi_adj + ti_adj;
                        rl.ensure_bigger_than_with_margin(*from_center, *to_center, needed);
                    }
                }
            }
            TeozTile::SelfMessage {
                participant_idx,
                text_width,
                center,
                is_reverse_define,
                active_level,
                ..
            } => {
                let idx = *participant_idx;
                let tm = TextMetrics::new(7.0, 7.0, 1.0, *text_width, tp.msg_line_height);
                let needed =
                    rose::self_arrow_preferred_size(&tm).width + active_right_shift(*active_level);

                // Java CommunicationTileSelf uses isReverseDefine() (not direction)
                if *is_reverse_define {
                    if idx > 0 {
                        let prev_center = livings[idx - 1].pos_c;
                        rl.ensure_bigger_than_with_margin(*center, prev_center, needed);
                    }
                } else {
                    if idx + 1 < n_parts {
                        let next_center = livings[idx + 1].pos_c;
                        rl.ensure_bigger_than_with_margin(next_center, *center, needed);
                    }
                }
            }
            TeozTile::Note {
                participant_idx,
                is_left,
                width,
                center,
                ..
            } => {
                let idx = *participant_idx;
                let note_half = (*width + NOTE_EXTENT_PADDING) / 2.0 + 5.0;
                if *is_left {
                    // Note to the left: need space before this participant
                    if idx > 0 {
                        let prev_center = livings[idx - 1].pos_c;
                        rl.ensure_bigger_than_with_margin(*center, prev_center, note_half);
                    }
                } else {
                    // Note to the right: need space after this participant
                    if idx + 1 < n_parts {
                        let next_center = livings[idx + 1].pos_c;
                        rl.ensure_bigger_than_with_margin(next_center, *center, note_half);
                    }
                }
            }
            _ => {}
        }
    }

    // ── Step 4b: LivingSpaces.addConstraints() ──────────────────────────
    // Java: current.posB >= previous.posD + 10 (ensure 10px gap between
    // adjacent participants even when no messages span between them).
    for i in 1..livings.len() {
        let prev_d = livings[i - 1].pos_d;
        let curr_b = livings[i].pos_b;
        rl.ensure_bigger_than_with_margin(curr_b, prev_d, 10.0);
    }

    // ── Step 5: Compile constraints ──────────────────────────────────────
    rl.compile();

    // ── Step 6: Assign Y positions (fillPositionelTiles) ─────────────────
    // Java PlayingSpace positions tiles starting at startingY = 8 within the
    // playing space. The playing space origin = STARTING_Y + max_preferred_height
    // (below participant heads). So tile top y = lifeline_top + 8.
    //
    // In our model, tile y represents the tile TOP (like Java), not the arrow y.
    // For Communication tiles, the arrow y = tile_y + arrowY.
    let max_box_height = box_heights.iter().copied().fold(0.0_f64, f64::max);
    // Java layout uses preferred height (= drawn + 1) for lifeline start
    let max_preferred_height = max_box_height + 1.0;
    let mut y = STARTING_Y + max_preferred_height + PLAYINGSPACE_STARTING_Y;
    // Track the previous message for note-on-message binding.
    // In Java, notes immediately following messages form a combined tile:
    //   combined height = max(message_h, note_h)
    // instead of message_h + note_h (separate tiles).
    let mut prev_msg_height: Option<f64> = None;
    let mut prev_msg_y: Option<f64> = None;
    // Java GroupingTile: MARGINY_MAGIC = 20, but getPreferredHeight uses full 20
    // while fillPositionelTiles uses header + MARGINY_MAGIC/2 = header + 10.
    // The effective bottom padding is MARGINY_MAGIC - MARGINY_MAGIC/2 = 10.
    const FRAG_BOTTOM_PADDING: f64 = 10.0;
    // Java EmptyTile(4): spacer before and after a GroupingTile.
    const EMPTY_TILE_SPACING: f64 = 4.0;

    // Track the y position where the current "block" (non-parallel group) started.
    // Parallel messages rewind y to this block start.
    let mut block_start_y: Option<f64> = None;
    let mut block_max_height: f64 = 0.0;

    // Track fragment nesting for parallel message support.
    // Java mergeParallel + TileParallel: when a parallel message follows a
    // GroupingTile, both share the same y start and the GroupingTile is offset
    // down by the message's contactPointRelative.
    let mut frag_depth: i32 = 0;
    // y before the EmptyTile(4) spacer of the outermost fragment
    let mut frag_block_y_before: Option<f64> = None;
    // Tile index range of the outermost fragment block
    let mut frag_block_start_idx: Option<usize> = None;

    // Track parallel message tile indices for contact-point alignment.
    // Java TileParallel aligns parallel tiles so their contact points (arrow y)
    // match the maximum contact point among all parallel tiles.
    // Each entry is tile_index for message tiles in the current parallel block.
    let mut parallel_block_tile_indices: Vec<usize> = Vec::new();

    // Track parallel fragment blocks.
    // When a FragmentStart has is_parallel=true, we rewind y to the start
    // of the previous block and lay out the fragment in parallel.
    // After the matching FragmentEnd, y = block_start + max(prev_height, this_height).
    let mut parallel_frag_base_y: Option<f64> = None;
    let mut parallel_frag_prev_height: f64 = 0.0;
    let mut parallel_frag_depth: i32 = 0; // nesting depth within the parallel fragment

    let tile_count = tiles.len();
    let mut tile_idx = 0;
    while tile_idx < tile_count {
        // Check if this tile is a parallel message
        let is_parallel_msg = matches!(
            tiles[tile_idx],
            TeozTile::Communication {
                is_parallel: true,
                ..
            } | TeozTile::SelfMessage {
                is_parallel: true,
                ..
            }
        );
        // Check if this tile is a parallel fragment start
        let is_parallel_frag = matches!(
            tiles[tile_idx],
            TeozTile::FragmentStart {
                is_parallel: true,
                ..
            }
        );
        let is_parallel = is_parallel_msg;

        // Check if this Note follows a message (note-on-message binding).
        let is_note_on_msg = matches!(
            tiles[tile_idx],
            TeozTile::Note {
                is_note_on_message: true,
                ..
            }
        ) && prev_msg_height.is_some();

        if is_parallel_frag {
            // Parallel fragment: rewind y to the start of the previous block.
            // Java mergeParallel creates a TileParallel where both fragments
            // share the same y start, and total height = max(frag1_h, frag2_h).
            if let Some(bs_y) = block_start_y {
                // Java removeEmptyCloseToParallel: the trailing EmptyTile(4) and
                // FRAG_BOTTOM_PADDING(10) from the previous fragment are removed.
                // Compute effective previous height without trailing padding.
                let trailing_padding = FRAG_BOTTOM_PADDING + EMPTY_TILE_SPACING; // 10 + 4 = 14
                let prev_effective = block_max_height - trailing_padding;
                // Java TileParallel contact-point alignment: when a fragment
                // (contactPointRelative = 0) is parallel with message tiles
                // (contactPointRelative = height - 8), the fragment is shifted
                // down by the max message contact point so their baselines align.
                let max_msg_contact: f64 = parallel_block_tile_indices
                    .iter()
                    .map(|&i| tiles[i].contact_point_relative())
                    .fold(0.0_f64, f64::max);
                parallel_frag_base_y = Some(bs_y);
                parallel_frag_prev_height = prev_effective;
                parallel_frag_depth = 1; // this fragment's own depth
                                         // Rewind to block start + contact shift.
                                         // No EmptyTile(4) before parallel fragment
                                         // (Java removeEmptyCloseToParallel removes it).
                y = bs_y + max_msg_contact;
                frag_depth += 1;
                tiles[tile_idx].set_y(y);
                let tile_h = tiles[tile_idx].preferred_height();
                y += tile_h;
            } else {
                // No block to parallel with — treat as normal fragment
                frag_depth += 1;
                y += EMPTY_TILE_SPACING;
                tiles[tile_idx].set_y(y);
                y += tiles[tile_idx].preferred_height();
            }
            prev_msg_height = None;
            prev_msg_y = None;
        } else if is_note_on_msg {
            let msg_h = prev_msg_height.unwrap();
            let msg_y = prev_msg_y.unwrap();
            let note_h = tiles[tile_idx].preferred_height();
            // Java CommunicationTileSelfNote: note y = startingY + push
            // where push = (selfPreferredH - noteCalcH) / 2.
            // For self-messages, this equals ARROW_PADDING_X (5px) because
            // the note calcH includes the self-message's vertical extent.
            // For regular messages, push = 0 (note starts at tile top).
            // Check if the preceding message tile (skipping LifeEvents) is a self-message
            // Java: CommunicationTileNoteRight and CommunicationTileSelfNoteRight
            // both place the note at the tile's y position (= msg_y). The polygon is
            // then rendered at tile_y + paddingY(5) by AbstractComponent.drawU().
            // We set tile_y = msg_y here; the paddingY offset is applied when
            // extracting NoteLayout for SVG rendering.
            tiles[tile_idx].set_y(msg_y);
            // Combined height = max(message_h, note_h)
            let combined_h = msg_h.max(note_h);
            y = msg_y + combined_h;
            prev_msg_height = None;
            prev_msg_y = None;
        } else if is_parallel {
            // Parallel message: rewind to block start, use max height.
            // Java: mergeParallel pulls the previous non-LifeEvent tile into
            // a TileParallel with the parallel message. Contact points align
            // the tiles vertically.
            if let Some(bs_y) = block_start_y {
                // Check if the block is a fragment block. If so, apply the
                // Java TileParallel contact-point offset: shift all fragment
                // tiles down by the message's contact point, and place the
                // message at the block start.
                if let Some(frag_start_idx) = frag_block_start_idx.take() {
                    let selfmsg_contact =
                        compute_selfmsg_contact(&tiles[tile_idx], tp.msg_line_height);
                    // Shift all fragment tiles down by selfmsg_contact
                    for shift_idx in frag_start_idx..tile_idx {
                        if let Some(old_y) = tiles[shift_idx].get_y() {
                            tiles[shift_idx].set_y(old_y + selfmsg_contact);
                        }
                    }
                    // Place parallel message at original block start
                    tiles[tile_idx].set_y(bs_y);
                    // Java removeEmptyCloseToParallel: the trailing EmptyTile(4)
                    // after the GroupingTile is removed when a parallel message
                    // follows. Our FragEnd.height(4) is the equivalent trailing
                    // spacer, so subtract it from block_max_height.
                    let trailing = EMPTY_TILE_SPACING; // 4.0
                    let effective_block = block_max_height - trailing;
                    y = bs_y + selfmsg_contact + effective_block;
                } else {
                    tiles[tile_idx].set_y(bs_y);
                    let tile_h = tiles[tile_idx].preferred_height();
                    if tile_h > block_max_height {
                        block_max_height = tile_h;
                    }
                    y = bs_y + block_max_height;
                }
            } else {
                // No block to parallel with — treat as normal
                tiles[tile_idx].set_y(y);
                y += tiles[tile_idx].preferred_height();
            }
            parallel_block_tile_indices.push(tile_idx);
            prev_msg_height = Some(tiles[tile_idx].preferred_height());
            prev_msg_y = Some(tiles[tile_idx].get_y().unwrap_or(y));
        } else {
            // Track fragment nesting depth
            let is_frag_start = matches!(
                tiles[tile_idx],
                TeozTile::FragmentStart { .. } | TeozTile::GroupStart { .. }
            );
            let is_frag_end = matches!(
                tiles[tile_idx],
                TeozTile::FragmentEnd { .. } | TeozTile::GroupEnd { .. }
            );

            if is_frag_start {
                frag_depth += 1;
                // Track parallel fragment nesting
                if parallel_frag_base_y.is_some() {
                    parallel_frag_depth += 1;
                }
            }
            if is_frag_end {
                frag_depth -= 1;
                // Track parallel fragment nesting
                if parallel_frag_base_y.is_some() {
                    parallel_frag_depth -= 1;
                }
            }

            // Java inserts EmptyTile(4) spacer before GroupingTile
            if is_frag_start {
                y += EMPTY_TILE_SPACING;
            }
            // Java GroupingTile bottom padding = MARGINY_MAGIC/2 = 10
            if is_frag_end {
                y += FRAG_BOTTOM_PADDING;
            }

            // Check if this FragmentEnd closes the parallel fragment
            if is_frag_end && parallel_frag_depth == 0 {
                if let Some(base_y) = parallel_frag_base_y.take() {
                    // Current fragment height from base, excluding trailing padding.
                    // y already includes FRAG_BOTTOM_PADDING(10) added above;
                    // exclude it along with this tile's height (EmptyTile equivalent).
                    let _trailing_padding = FRAG_BOTTOM_PADDING + tiles[tile_idx].preferred_height();
                    let this_frag_height = y - base_y; // y includes the 10px padding
                    let this_effective = this_frag_height - FRAG_BOTTOM_PADDING;
                    // Use max of previous block height and this parallel fragment height
                    let max_height = parallel_frag_prev_height.max(this_effective);
                    // After the parallel block, add back the trailing padding so
                    // subsequent normal tiles have correct spacing.
                    y = base_y + max_height + FRAG_BOTTOM_PADDING;
                    // Set block tracking for potential subsequent parallel blocks.
                    // Java removeEmptyCloseToParallel strips the trailing
                    // EmptyTile(4) when a subsequent parallel tile follows, so
                    // block_max_height should NOT include EMPTY_TILE_SPACING.
                    block_start_y = Some(base_y);
                    block_max_height = max_height + FRAG_BOTTOM_PADDING;
                    frag_block_y_before = Some(base_y);
                    // Place the FragEnd tile at the appropriate position
                    tiles[tile_idx].set_y(y);
                    y += tiles[tile_idx].preferred_height();
                    prev_msg_height = None;
                    prev_msg_y = None;
                    tile_idx += 1;
                    continue;
                }
            }

            // Record the outermost fragment start AFTER EmptyTile spacing.
            // Java's TileParallel aligns the GroupingTile (which starts
            // after the leading EmptyTile) with the parallel message.
            if is_frag_start && frag_depth == 1 {
                frag_block_y_before = Some(y);
                frag_block_start_idx = Some(tile_idx);
            }

            tiles[tile_idx].set_y(y);

            let tile_h = tiles[tile_idx].preferred_height();
            match &tiles[tile_idx] {
                TeozTile::Communication { .. } | TeozTile::SelfMessage { .. } => {
                    prev_msg_height = Some(tile_h);
                    prev_msg_y = Some(y);
                    // Start a new parallel block (only at depth 0)
                    if frag_depth == 0 {
                        // Apply contact-point alignment for the previous block
                        apply_contact_point_alignment(&mut tiles, &parallel_block_tile_indices);
                        parallel_block_tile_indices.clear();
                        // Record this tile as the first in a new parallel block
                        parallel_block_tile_indices.push(tile_idx);
                        block_start_y = Some(y);
                        block_max_height = tile_h;
                    }
                }
                TeozTile::LifeEvent { .. } => {
                    // LifeEvent tiles don't break the message-note chain
                }
                TeozTile::FragmentEnd { .. } | TeozTile::GroupEnd { .. } if frag_depth == 0 => {
                    // Outermost fragment just closed. Set block_start_y to
                    // the FragmentStart y (after EmptyTile spacing) so that
                    // a subsequent parallel message can parallel with the
                    // entire GroupingTile equivalent.
                    if let Some(fby) = frag_block_y_before {
                        block_start_y = Some(fby);
                        block_max_height = y + tile_h - fby;
                    }
                    prev_msg_height = None;
                    prev_msg_y = None;
                }
                _ => {
                    prev_msg_height = None;
                    prev_msg_y = None;
                    if frag_depth == 0 {
                        apply_contact_point_alignment(&mut tiles, &parallel_block_tile_indices);
                        parallel_block_tile_indices.clear();
                        block_start_y = None;
                        block_max_height = 0.0;
                        frag_block_y_before = None;
                        frag_block_start_idx = None;
                    }
                }
            }
            y += tile_h;
        }
        tile_idx += 1;
    }
    // Apply contact-point alignment for the last parallel block
    apply_contact_point_alignment(&mut tiles, &parallel_block_tile_indices);
    let tiles_bottom = y;
    // Java: lifeline height = getPreferredHeight = finalY + 10 (bottom padding)
    // where finalY = startingY(8) + sum_tile_heights.
    // lifeline_bottom = lifeline_top + lifeline_height = lifeline_top + sum + 18
    // tiles_bottom = lifeline_top + 8 + sum, so lifeline_bottom = tiles_bottom + 10
    let mut lifeline_bottom = tiles_bottom + 10.0;

    // ── Step 7: Extract SeqLayout ────────────────────────────────────────
    // Java: SequenceDiagramFileMakerTeoz applies UTranslate(5,5) + dx(-min1).
    // min1 = PlayingSpace.getMinX() which includes all tile minX, group
    // margins, participant positions, and the origin.
    // SVG viewport width = (maxX - minX) + 10.
    //
    // Compute raw_min/raw_max in Real coordinate space, then derive x_offset.
    let origin_val = rl.get_value(xorigin);
    let mut raw_min = origin_val;
    let mut raw_max = origin_val;
    // Include participant posB, posD, and posC2 (posC + activation delta)
    for living in &livings {
        let b = rl.get_value(living.pos_b);
        let d = rl.get_value(living.pos_d);
        let c = rl.get_value(living.pos_c);
        if b < raw_min {
            raw_min = b;
        }
        if d > raw_max {
            raw_max = d;
        }
        // Java: PlayingSpace includes posC2 = posC + activation delta.
        // For now, posC is sufficient since we track activation in extents below.
        if c > raw_max {
            raw_max = c;
        }
    }
    // Include self-message and note extents in raw space.
    // Only include tiles OUTSIDE groups/fragments — tiles inside groups
    // contribute through the group expansion below.
    //
    // Uses the unified self_message_extent() helper for consistent geometry.
    {
        let mut outer_depth: usize = 0;
        for tile_i in 0..tiles.len() {
            let tile = &tiles[tile_i];
            match tile {
                TeozTile::GroupStart { .. } | TeozTile::FragmentStart { .. } => {
                    outer_depth += 1;
                }
                TeozTile::GroupEnd { .. } | TeozTile::FragmentEnd { .. } => {
                    outer_depth = outer_depth.saturating_sub(1);
                }
                _ if outer_depth > 0 => {
                    // Skip: will be handled by group expansion below
                }
                TeozTile::SelfMessage {
                    participant_idx,
                    text_width,
                    direction,
                    active_level,
                    ..
                } => {
                    let cx = rl.get_value(livings[*participant_idx].pos_c);
                    let comp_w = self_message_comp_width(*text_width, tp.msg_line_height);
                    let (sm_min, sm_max) =
                        self_message_extent(cx, comp_w, *active_level, direction);
                    if sm_min < raw_min {
                        raw_min = sm_min;
                    }
                    if sm_max > raw_max {
                        raw_max = sm_max;
                    }
                }
                TeozTile::Note {
                    participant_idx,
                    is_left,
                    width,
                    is_note_on_message,
                    ..
                } => {
                    let cx = rl.get_value(livings[*participant_idx].pos_c);
                    // Java uses ComponentRoseNote.getPreferredWidth for extent,
                    // which includes 2*paddingX beyond the drawn polygon width.
                    let extent_w = *width + NOTE_EXTENT_PADDING;
                    if *is_note_on_message {
                        // Note attached to a self-message: use Java's
                        // CommunicationTileSelfNoteLeft/Right extent model.
                        // minX/maxX are derived from the self-message's extent.
                        if let Some((sm_pidx, sm_tw, sm_dir, sm_al)) =
                            find_preceding_self_message(&tiles, tile_i)
                        {
                            let sm_cx = rl.get_value(livings[sm_pidx].pos_c);
                            let sm_comp_w = self_message_comp_width(sm_tw, tp.msg_line_height);
                            let (sm_min, sm_max) =
                                self_message_extent(sm_cx, sm_comp_w, sm_al, &sm_dir);
                            if *is_left {
                                // Java CommunicationTileSelfNoteLeft.getMinX():
                                //   tile.getMinX() - notePreferredWidth
                                let left = sm_min - extent_w;
                                if left < raw_min {
                                    raw_min = left;
                                }
                                // maxX comes from the self-message
                                if sm_max > raw_max {
                                    raw_max = sm_max;
                                }
                            } else {
                                // Java CommunicationTileSelfNoteRight.getMaxX():
                                //   tile.getMaxX() + notePreferredWidth
                                let right = sm_max + extent_w;
                                if right > raw_max {
                                    raw_max = right;
                                }
                                // minX comes from the self-message
                                if sm_min < raw_min {
                                    raw_min = sm_min;
                                }
                            }
                        } else {
                            // Fallback: note on regular message, use cx-based
                            if *is_left {
                                let left = cx - extent_w - 5.0;
                                if left < raw_min {
                                    raw_min = left;
                                }
                            } else {
                                let right = cx + extent_w;
                                if right > raw_max {
                                    raw_max = right;
                                }
                            }
                        }
                    } else {
                        // Standalone note: simple cx-based extent
                        if *is_left {
                            let left = cx - extent_w - 5.0;
                            if left < raw_min {
                                raw_min = left;
                            }
                        } else {
                            let right = cx + extent_w;
                            if right > raw_max {
                                raw_max = right;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    // Apply group/fragment margin expansion using a recursive approach
    // matching Java's GroupingTile hierarchy.  Each group computes its own
    // internal min/max from children, adds MARGINX, then reports
    // getMinX = min - EXTERNAL_MARGINX1, getMaxX = max + EXTERNAL_MARGINX2.
    {
        /// Compute the (getMinX, getMaxX) of a group starting at `start` in
        /// the tile list, returning the index past the matching GroupEnd.
        fn compute_group_extent(
            tiles: &[TeozTile],
            start: usize,
            livings: &[LivingSpace],
            rl: &RealLine,
            tp: &TeozParams,
        ) -> (f64, f64, usize) {
            let mut group_min = f64::MAX;
            let mut group_max = f64::MIN;
            let mut else_labels: Vec<String> = Vec::new();
            let mut i = start;
            while i < tiles.len() {
                match &tiles[i] {
                    TeozTile::GroupStart { .. } | TeozTile::FragmentStart { .. } => {
                        // Recurse into nested group
                        let (child_min, child_max, next_i) =
                            compute_group_extent(tiles, i + 1, livings, rl, tp);
                        // Child reports getMinX/getMaxX; add MARGINX for this level
                        let child_with_margin_min = child_min - GROUP_MARGINX;
                        let child_with_margin_max = child_max + GROUP_MARGINX;
                        if child_with_margin_min < group_min {
                            group_min = child_with_margin_min;
                        }
                        if child_with_margin_max > group_max {
                            group_max = child_with_margin_max;
                        }
                        i = next_i;
                        continue; // Skip i += 1 at the bottom of the loop
                    }
                    TeozTile::GroupEnd { .. } | TeozTile::FragmentEnd { .. } => {
                        // End of this group — return with external margins
                        if group_min == f64::MAX {
                            group_min = 0.0;
                        }
                        if group_max == f64::MIN {
                            group_max = 0.0;
                        }
                        // Java: else tiles contribute to maxX via
                        // ElseTile.getMaxX() = parent.getMinX() + elseWidth
                        // parent.getMinX() = group_min - EXTERNAL_MARGINX1
                        for label in &else_labels {
                            let bracket_label = format!("[{}]", label);
                            let pure_text_w = crate::font_metrics::text_width(
                                &bracket_label,
                                "sans-serif",
                                11.0,
                                true,
                                false,
                            );
                            let else_width = pure_text_w + 10.0; // marginX1(5) + marginX2(5)
                            let else_max =
                                (group_min - GROUP_EXTERNAL_MARGINX1) + else_width;
                            if else_max > group_max {
                                group_max = else_max;
                            }
                        }
                        // Java: max2.add(this.min.addFixed(width + 16))
                        // where width = ComponentRoseGroupingHeader.getPreferredWidth()
                        // The parent FragmentStart/GroupStart is at start-1
                        if start > 0 {
                            match &tiles[start - 1] {
                                TeozTile::FragmentStart { kind, label, .. } => {
                                    let kind_text_w = crate::font_metrics::text_width(
                                        kind.label(), "sans-serif", 13.0, true, false,
                                    );
                                    let header_w = if label.is_empty() {
                                        kind_text_w + 45.0
                                    } else {
                                        let bl = format!("[{}]", label);
                                        let cw = crate::font_metrics::text_width(
                                            &bl, "sans-serif", 11.0, true, false,
                                        );
                                        kind_text_w + 45.0 + 15.0 + cw
                                    };
                                    let header_max = group_min + header_w + 16.0;
                                    if header_max > group_max {
                                        group_max = header_max;
                                    }
                                }
                                TeozTile::GroupStart { _label, .. } => {
                                    if let Some(lbl) = _label {
                                        let kind_text_w = crate::font_metrics::text_width(
                                            "group", "sans-serif", 13.0, true, false,
                                        );
                                        let header_w = if lbl.is_empty() {
                                            kind_text_w + 45.0
                                        } else {
                                            let bl = format!("[{}]", lbl);
                                            let cw = crate::font_metrics::text_width(
                                                &bl, "sans-serif", 11.0, true, false,
                                            );
                                            kind_text_w + 45.0 + 15.0 + cw
                                        };
                                        let header_max = group_min + header_w + 16.0;
                                        if header_max > group_max {
                                            group_max = header_max;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        return (
                            group_min - GROUP_EXTERNAL_MARGINX1,
                            group_max + GROUP_EXTERNAL_MARGINX2,
                            i + 1,
                        );
                    }
                    TeozTile::SelfMessage {
                        participant_idx,
                        text_width,
                        direction,
                        active_level,
                        ..
                    } => {
                        let cx = rl.get_value(livings[*participant_idx].pos_c);
                        let comp_w = self_message_comp_width(*text_width, tp.msg_line_height);
                        let (t_min, t_max) =
                            self_message_extent(cx, comp_w, *active_level, direction);
                        // Add MARGINX for this tile within the group
                        let child_min = t_min - GROUP_MARGINX;
                        let child_max = t_max + GROUP_MARGINX;
                        if child_min < group_min {
                            group_min = child_min;
                        }
                        if child_max > group_max {
                            group_max = child_max;
                        }
                    }
                    TeozTile::Communication {
                        from_idx, to_idx, ..
                    } => {
                        let from_x = rl.get_value(livings[*from_idx].pos_c);
                        let to_x = rl.get_value(livings[*to_idx].pos_c);
                        let child_min = f64::min(from_x, to_x) - GROUP_MARGINX;
                        let child_max = f64::max(from_x, to_x) + GROUP_MARGINX;
                        if child_min < group_min {
                            group_min = child_min;
                        }
                        if child_max > group_max {
                            group_max = child_max;
                        }
                    }
                    TeozTile::Note {
                        participant_idx,
                        is_left,
                        width,
                        is_note_on_message,
                        ..
                    } => {
                        let cx = rl.get_value(livings[*participant_idx].pos_c);
                        let extent_w = *width + NOTE_EXTENT_PADDING;
                        let (t_min, t_max) = if *is_note_on_message {
                            // Note on self-message: use self-message extent
                            if let Some((sm_pidx, sm_tw, sm_dir, sm_al)) =
                                find_preceding_self_message(tiles, i)
                            {
                                let sm_cx = rl.get_value(livings[sm_pidx].pos_c);
                                let sm_comp_w = self_message_comp_width(sm_tw, tp.msg_line_height);
                                let (sm_min, sm_max) =
                                    self_message_extent(sm_cx, sm_comp_w, sm_al, &sm_dir);
                                if *is_left {
                                    (sm_min - extent_w, sm_max)
                                } else {
                                    (sm_min, sm_max + extent_w)
                                }
                            } else {
                                // Fallback
                                if *is_left {
                                    (cx - extent_w - 5.0, cx)
                                } else {
                                    (cx, cx + extent_w)
                                }
                            }
                        } else if *is_left {
                            (cx - extent_w - 5.0, cx)
                        } else {
                            (cx, cx + extent_w)
                        };
                        let child_min = t_min - GROUP_MARGINX;
                        let child_max = t_max + GROUP_MARGINX;
                        if child_min < group_min {
                            group_min = child_min;
                        }
                        if child_max > group_max {
                            group_max = child_max;
                        }
                    }

                    TeozTile::FragmentSeparator { label, .. } => {
                        // Java: else tiles contribute only to maxX, not to minX
                        // Collected and processed at the GroupEnd/FragmentEnd
                        else_labels.push(label.clone());
                    }
                    _ => {}
                }
                i += 1;
            }
            // Reached end without GroupEnd (malformed)
            if group_min == f64::MAX {
                group_min = 0.0;
            }
            if group_max == f64::MIN {
                group_max = 0.0;
            }
            (
                group_min - GROUP_EXTERNAL_MARGINX1,
                group_max + GROUP_EXTERNAL_MARGINX2,
                i,
            )
        }

        let mut i = 0;
        while i < tiles.len() {
            match &tiles[i] {
                TeozTile::GroupStart { .. } | TeozTile::FragmentStart { .. } => {
                    let (g_min, g_max, next_i) =
                        compute_group_extent(&tiles, i + 1, &livings, &rl, &tp);
                    if g_min < raw_min {
                        raw_min = g_min;
                    }
                    if g_max > raw_max {
                        raw_max = g_max;
                    }
                    i = next_i;
                }
                _ => {
                    i += 1;
                }
            }
        }
    }
    // Also ensure group/fragment header label width is accounted for.
    // Java GroupingTile:
    //   this.min = RealUtils.min(child.getMinX() - MARGINX)
    //   max2.add(this.min.addFixed(headerWidth + 16))
    //   getMaxX = this.max.addFixed(EXTERNAL_MARGINX2)
    // headerWidth = ComponentRoseGroupingHeader.getPreferredWidth
    //             = pureTextWidth + marginX1(15) + marginX2(30) = pureTextWidth + 45
    // Combined: getMaxX contribution = (this.min + pureTextWidth + 45 + 16) + 9
    //         = this.min + pureTextWidth + 70
    // Since raw_min = this.min - EXTERNAL_MARGINX1 = this.min - 3
    //   → this.min = raw_min + 3
    //   → contribution = raw_min + 3 + pureTextWidth + 70 = raw_min + pureTextWidth + 73
    {
        let mut group_depth: usize = 0;
        // Store (kind_label, condition_label) pairs for header width computation
        let mut header_entries: Vec<(&str, String)> = Vec::new();
        for tile in &tiles {
            match tile {
                TeozTile::GroupStart { _label, .. } => {
                    group_depth += 1;
                    if let Some(l) = _label {
                        header_entries.push(("group", l.clone()));
                    }
                }
                TeozTile::FragmentStart { kind, label, .. } => {
                    group_depth += 1;
                    header_entries.push((kind.label(), label.clone()));
                }
                TeozTile::GroupEnd { .. } | TeozTile::FragmentEnd { .. } => {
                    if group_depth == 1 {
                        for (kind_lbl, condition) in &header_entries {
                            let kind_text_w = font_metrics::text_width(
                                kind_lbl,
                                default_font,
                                msg_font_size,
                                true,
                                false,
                            );
                            let header_width = if condition.is_empty() {
                                kind_text_w + 45.0
                            } else {
                                let bracket_label = format!("[{}]", condition);
                                let comment_w = font_metrics::text_width(
                                    &bracket_label,
                                    default_font,
                                    11.0,
                                    true,
                                    false,
                                );
                                kind_text_w + 45.0 + 15.0 + comment_w
                            };
                            // Java: this.min + headerWidth + 16 + EXTERNAL_MARGINX2(9)
                            // this.min = raw_min + EXTERNAL_MARGINX1(3)
                            let header_max = raw_min
                                + GROUP_EXTERNAL_MARGINX1
                                + header_width
                                + 16.0
                                + GROUP_EXTERNAL_MARGINX2;
                            if header_max > raw_max {
                                raw_max = header_max;
                            }
                        }
                        header_entries.clear();
                    }
                    group_depth = group_depth.saturating_sub(1);
                }
                _ => {}
            }
        }
    }
    let min1 = raw_min;
    let x_offset = DOC_MARGIN_X - min1;
    log::debug!("teoz width: raw_min={raw_min:.4} raw_max={raw_max:.4} x_offset={x_offset:.4} diagram_w={:.4}", raw_max - raw_min);
    // Helper: get Real x value with document margin applied.
    let get_x = |id: RealId| -> f64 { rl.get_value(id) + x_offset };

    // Build ParticipantLayout from Real-resolved positions
    for (i, p) in sd.participants.iter().enumerate() {
        let center_x = get_x(livings[i].pos_c);
        part_layouts.push(ParticipantLayout {
            name: p.name.clone(),
            x: center_x,
            box_width: box_widths[i],
            box_height: box_heights[i],
            kind: p.kind.clone(),
            color: p.color.clone(),
        });
    }

    // Extract messages, notes, etc. from tiles
    let mut messages: Vec<MessageLayout> = Vec::new();
    let mut activations: Vec<ActivationLayout> = Vec::new();
    let mut destroys: Vec<DestroyLayout> = Vec::new();
    let mut notes: Vec<NoteLayout> = Vec::new();
    let mut dividers: Vec<DividerLayout> = Vec::new();
    let mut delays: Vec<DelayLayout> = Vec::new();
    let mut refs: Vec<RefLayout> = Vec::new();
    let mut fragments: Vec<FragmentLayout> = Vec::new();
    let mut fragment_stack: Vec<(f64, FragmentKind, String, Vec<(f64, String)>, usize)> =
        Vec::new();
    let mut groups: Vec<GroupLayout> = Vec::new();
    let mut group_stack: Vec<(f64, Option<String>)> = Vec::new();

    // Diagram width is raw_max - raw_min (computed above with group expansion).
    // Rendered positions use get_x which adds x_offset, so differences are preserved.
    let diagram_width = raw_max - raw_min;
    let total_min_x = raw_min + x_offset; // = DOC_MARGIN_X = 5
    let total_max_x = raw_max + x_offset;
    log::debug!("teoz extents: raw_min={raw_min:.2} raw_max={raw_max:.2} diagram_width={diagram_width:.2} total_min_x={total_min_x:.2} total_max_x={total_max_x:.2}");

    // Track activation state for ActivationLayout generation

    for tile_i in 0..tiles.len() {
        let tile = &tiles[tile_i];
        match tile {
            TeozTile::Communication {
                from_name,
                to_name,
                from_idx,
                to_idx,
                text,
                text_lines,
                is_dashed,
                has_open_head,
                arrow_head,
                text_width,
                y,
                height,
                autonumber,
                circle_from,
                circle_to,
                cross_from,
                cross_to,
                from_level,
                to_level,
                hidden,
                bidirectional,
                ..
            } => {
                if *hidden {
                    continue;
                }
                let ty = y.unwrap_or(0.0);
                // Java: tile y = tile top. Arrow y = tile_top + arrowY.
                // arrowY = textHeight + paddingY = (height - ARROW_DELTA_Y - ARROW_PADDING_Y)
                let arrow_y = ty + (height - rose::ARROW_DELTA_Y - rose::ARROW_PADDING_Y);
                let raw_from_x = get_x(livings[*from_idx].pos_c);
                let raw_to_x = get_x(livings[*to_idx].pos_c);

                // Gate/lost/found messages: virtual endpoint is near the
                // real participant, computed from arrow preferred width.
                // Java: CommunicationTile uses getPreferredWidth() which is
                // text_width + ARROW_DELTA_X(10) + 2*paddingY(7) + inset(2).
                let is_gate_from = from_name == "[";
                let is_gate_to = to_name == "]";

                let (from_x, to_x, is_left) = if is_gate_from || is_gate_to {
                    // Gate message: compute virtual endpoint from text width.
                    // Arrow total width = 7 (left pad) + text_width + 5 (right pad)
                    //                    + 10 (arrowhead) + 2 (inset) = text_width + 24
                    let arrow_span = text_width + 24.0;
                    if is_gate_to {
                        // Lost message (A->?): arrow goes right from real participant
                        let fx = raw_from_x;
                        let tx = fx + arrow_span;
                        (fx, tx, false)
                    } else {
                        // Found message (?->E): arrow comes from left to real participant
                        let tx = raw_to_x;
                        let fx = tx - arrow_span;
                        (fx, tx, false)
                    }
                } else {
                    let is_left = raw_to_x < raw_from_x;
                    // Java CommunicationTile.drawU(): adjust x positions
                    // based on activation levels (LIVE_DELTA_SIZE = 5).
                    const LIVE_DELTA: f64 = 5.0;
                    if is_left {
                        // Reverse direction (right-to-left)
                        let mut x1 = raw_from_x;
                        let level1 = *from_level;
                        if level1 == 1 {
                            x1 -= LIVE_DELTA;
                        } else if level1 > 2 {
                            x1 += LIVE_DELTA * (level1 as f64 - 2.0);
                        }
                        let x2 = raw_to_x + LIVE_DELTA * (*to_level as f64);
                        (x1, x2, true)
                    } else {
                        // Normal direction (left-to-right)
                        let x1 = raw_from_x + LIVE_DELTA * (*from_level as f64);
                        let mut adjusted_tl = *to_level as i64;
                        if adjusted_tl > 0 {
                            adjusted_tl -= 2;
                        }
                        let x2 = raw_to_x + LIVE_DELTA * (adjusted_tl as f64);
                        (x1, x2, false)
                    }
                };
                messages.push(MessageLayout {
                    from_x,
                    to_x,
                    y: arrow_y,
                    text: text.clone(),
                    text_lines: text_lines.clone(),
                    is_self: false,
                    is_dashed: *is_dashed,
                    is_left,
                    has_open_head: *has_open_head,
                    arrow_head: arrow_head.clone(),
                    autonumber: autonumber.clone(),
                    source_line: None, // TODO: propagate from parser
                    self_return_x: from_x,
                    self_center_x: from_x,
                    color: None,
                    circle_from: *circle_from,
                    circle_to: *circle_to,
                    cross_from: *cross_from,
                    cross_to: *cross_to,
                    bidirectional: *bidirectional,
                });
            }
            TeozTile::SelfMessage {
                participant_idx,
                text,
                text_lines,
                text_width,
                is_dashed,
                has_open_head,
                arrow_head,
                y,
                autonumber,
                direction,
                active_level,
                circle_from,
                circle_to,
                cross_from,
                cross_to,
                hidden,
                bidirectional,
                ..
            } => {
                if *hidden {
                    continue;
                }
                let ty = y.unwrap_or(0.0);
                let cx = get_x(livings[*participant_idx].pos_c);
                let is_left = !*bidirectional && *direction == SeqDirection::RightToLeft;
                let has_bar = *active_level > 0;

                // Java: CommunicationTileSelf.drawU() uses
                //   getStartingY() + comp.getYPoint(stringBounder)
                // where getYPoint = self_arrow_start_point().y = text_h + ARROW_PADDING_Y
                let self_text_h = tp.msg_line_height * text_lines.len().max(1) as f64;
                let self_tm = TextMetrics::new(7.0, 7.0, 1.0, *text_width, self_text_h);
                let self_y_offset = rose::self_arrow_start_point(&self_tm).y;

                // Compute self-message from_x/to_x/return_x accounting for
                // activation bar, matching Java's LivingParticipantBox logic.
                let (self_from_x, self_return_x, self_to_x) = if is_left {
                    let act_left = if has_bar {
                        cx - ACTIVATION_WIDTH / 2.0
                    } else {
                        cx
                    };
                    let outgoing_x = if has_bar { act_left } else { cx };
                    let ret_x = act_left - 1.0;
                    let to = act_left - SELF_MSG_WIDTH;
                    (outgoing_x, ret_x, to)
                } else {
                    let act_right = if has_bar {
                        cx + ACTIVATION_WIDTH / 2.0
                    } else {
                        cx
                    };
                    let outgoing_x = if has_bar { act_right } else { cx };
                    let ret_x = act_right + 1.0;
                    let to = act_right + SELF_MSG_WIDTH;
                    (outgoing_x, ret_x, to)
                };

                messages.push(MessageLayout {
                    from_x: self_from_x,
                    to_x: self_to_x,
                    y: ty + self_y_offset,
                    text: text.clone(),
                    text_lines: text_lines.clone(),
                    is_self: true,
                    is_dashed: *is_dashed,
                    is_left,
                    has_open_head: *has_open_head,
                    arrow_head: arrow_head.clone(),
                    autonumber: autonumber.clone(),
                    source_line: None, // TODO: propagate from parser
                    self_return_x,
                    self_center_x: cx,
                    color: None,
                    circle_from: *circle_from,
                    circle_to: *circle_to,
                    cross_from: *cross_from,
                    cross_to: *cross_to,
                    bidirectional: *bidirectional,
                });
            }
            TeozTile::Note {
                participant_idx,
                text,
                is_left,
                width,
                height: _,
                y,
                is_note_on_message,
                ..
            } => {
                // Java AbstractComponent.drawU applies UTranslate(paddingX, paddingY)
                // before rendering the note polygon. For notes, Rose.paddingY = 5.
                // The tile y is the tile top; the polygon starts paddingY below it.
                let ty = y.unwrap_or(0.0) + 5.0;
                let cx = get_x(livings[*participant_idx].pos_c);
                let nx = if *is_note_on_message {
                    // Note on self-message: position relative to self-message extent.
                    // Java CommunicationTileSelf uses posC2 = posC + rightShift,
                    // where rightShift >= ACTIVATION_WIDTH/2 (the lifeline always
                    // occupies at least this width). Use max(1, active_level) so
                    // the extent always clears the lifeline.
                    if let Some((sm_pidx, sm_tw, sm_dir, sm_al)) =
                        find_preceding_self_message(&tiles, tile_i)
                    {
                        let sm_cx = get_x(livings[sm_pidx].pos_c);
                        let sm_comp_w = self_message_comp_width(sm_tw, tp.msg_line_height);
                        let sm_cx_raw = rl.get_value(livings[sm_pidx].pos_c);
                        let note_al = sm_al.max(1); // lifeline always has >=1 activation width
                        let (sm_min, sm_max) =
                            self_message_extent(sm_cx_raw, sm_comp_w, note_al, &sm_dir);
                        if *is_left {
                            // Java: tile.getMinX() - noteWidth (in raw coords) + x_offset
                            sm_min + x_offset - *width
                        } else {
                            // Java: tile.getMaxX() (in raw coords) + x_offset
                            sm_max + x_offset
                        }
                    } else {
                        // Fallback: note on regular message
                        if *is_left {
                            cx - *width - 5.0
                        } else {
                            cx + 5.0
                        }
                    }
                } else if *is_left {
                    cx - *width - 5.0
                } else {
                    cx + 5.0
                };
                // Use drawn polygon height for SVG rendering, not the
                // preferred tile height which includes 2*paddingY extra.
                let drawn_h = estimate_note_height(text);
                notes.push(NoteLayout {
                    x: nx,
                    y: ty,
                    width: *width,
                    layout_width: *width + 10.0,
                    height: drawn_h,
                    text: text.clone(),
                    is_left: *is_left,
                    is_self_msg_note: *is_note_on_message,
                    is_note_on_message: *is_note_on_message,
                    assoc_message_idx: None,
                    teoz_mode: true,
                });
            }
            TeozTile::NoteOver {
                participants,
                text,
                width,
                height: _,
                y,
            } => {
                // Same paddingY offset as Note (see above).
                let ty = y.unwrap_or(0.0) + 5.0;
                // Center the note between the first and last referenced participant
                let (left_x, right_x) = if participants.len() >= 2 {
                    let idx0 = name_to_idx.get(&participants[0]).copied().unwrap_or(0);
                    let idx1 = name_to_idx
                        .get(participants.last().unwrap())
                        .copied()
                        .unwrap_or(0);
                    (get_x(livings[idx0].pos_c), get_x(livings[idx1].pos_c))
                } else if participants.len() == 1 {
                    let idx0 = name_to_idx.get(&participants[0]).copied().unwrap_or(0);
                    let cx = get_x(livings[idx0].pos_c);
                    (cx - *width / 2.0, cx + *width / 2.0)
                } else {
                    (total_min_x, total_max_x)
                };
                let center = (left_x + right_x) / 2.0;
                let drawn_h = estimate_note_height(text);
                notes.push(NoteLayout {
                    x: center - *width / 2.0,
                    y: ty,
                    width: *width,
                    layout_width: *width + 10.0,
                    height: drawn_h,
                    text: text.clone(),
                    is_left: false,
                    is_self_msg_note: false,
                    is_note_on_message: false,
                    assoc_message_idx: None,
                    teoz_mode: true,
                });
            }
            TeozTile::Divider { text, y, .. } => {
                let ty = y.unwrap_or(0.0);
                dividers.push(DividerLayout {
                    y: ty,
                    x: total_min_x,
                    width: diagram_width,
                    text: text.clone(),
                    height: 0.0,
                    component_y: ty,
                });
            }
            TeozTile::Delay { text, height, y } => {
                let ty = y.unwrap_or(0.0);
                delays.push(DelayLayout {
                    y: ty,
                    height: *height,
                    x: total_min_x,
                    width: diagram_width,
                    text: text.clone(),
                    lifeline_break_y: ty,
                });
            }
            TeozTile::Ref {
                participants,
                label,
                height,
                y,
            } => {
                let ty = y.unwrap_or(0.0);
                let (rx, rw) = if participants.is_empty() {
                    (total_min_x, diagram_width)
                } else {
                    let idxs: Vec<usize> = participants
                        .iter()
                        .filter_map(|p| name_to_idx.get(p).copied())
                        .collect();
                    if idxs.is_empty() {
                        (total_min_x, diagram_width)
                    } else {
                        let min_idx = *idxs.iter().min().unwrap();
                        let max_idx = *idxs.iter().max().unwrap();
                        let lx = get_x(livings[min_idx].pos_b);
                        let rx = get_x(livings[max_idx].pos_d);
                        (lx, rx - lx)
                    }
                };
                refs.push(RefLayout {
                    x: rx,
                    y: ty,
                    width: rw,
                    height: *height,
                    label: label.clone(),
                });
            }
            TeozTile::FragmentStart { kind, label, y, .. } => {
                let ty = y.unwrap_or(0.0);
                fragment_stack.push((ty, kind.clone(), label.clone(), Vec::new(), tile_i + 1));
            }
            TeozTile::FragmentSeparator { label, y, .. } => {
                let ty = y.unwrap_or(0.0);
                if let Some(entry) = fragment_stack.last_mut() {
                    entry.3.push((ty, label.clone()));
                }
            }
            TeozTile::FragmentEnd { y, .. } => {
                let ty = y.unwrap_or(0.0);
                if let Some((y_start, kind, label, separators, child_start)) = fragment_stack.pop()
                {
                    let depth = fragment_stack.len(); // 0 for outermost
                                                      // Compute per-fragment width from child tiles.
                                                      // Java GroupingTile computes its own min/max from children.
                    let (frag_min, frag_max) =
                        compute_fragment_extent(&tiles, child_start, tile_i, &livings, &rl, &tp);
                    // Java: ComponentRoseGroupingHeader.getPreferredWidth():
                    //   getTextWidth() = pureTextW(kindLabel) + marginX1(15) + marginX2(30)
                    //   if condition label present:
                    //     sup = marginX1(15) + commentMargin(0) + commentTextWidth
                    //     commentText = "[condition]" at 11pt bold
                    //   else: sup = 0
                    //   width = getTextWidth() + sup
                    // Java GroupingTile: max candidate = this.min + width + 16
                    let kind_text_w =
                        font_metrics::text_width(kind.label(), default_font, msg_font_size, true, false);
                    let header_width = if label.is_empty() {
                        // No condition label: sup = 0
                        kind_text_w + 45.0  // marginX1(15) + marginX2(30)
                    } else {
                        // Condition label present: "[label]" at 11pt bold
                        let bracket_label = format!("[{}]", label);
                        let comment_w =
                            font_metrics::text_width(&bracket_label, default_font, 11.0, true, false);
                        kind_text_w + 45.0 + 15.0 + comment_w  // + marginX1(15) + commentWidth
                    };
                    let header_right = frag_min + header_width + 16.0;
                    let effective_max = frag_max.max(header_right);
                    // Convert to document coordinates
                    let frag_x = frag_min + x_offset;
                    let frag_width = effective_max - frag_min;
                    // The tile y includes FRAG_BOTTOM_PADDING (10px) which is
                    // spacing below the frame rect, not part of the frame itself.
                    let frame_height = ty - y_start - FRAG_BOTTOM_PADDING;
                    // Find the first message tile index within this fragment.
                    // Count message tiles before child_start to get the message index.
                    let first_msg_idx = {
                        let mut msg_count_before = 0;
                        let mut found = None;
                        for ti in 0..tiles.len() {
                            let is_msg = matches!(
                                tiles[ti],
                                TeozTile::Communication { .. } | TeozTile::SelfMessage { .. }
                            );
                            if ti >= child_start && ti < tile_i && is_msg && found.is_none() {
                                found = Some(msg_count_before);
                            }
                            if is_msg && ti < child_start {
                                msg_count_before += 1;
                            }
                            if is_msg && ti >= child_start && ti < tile_i && found.is_some() {
                                break;
                            }
                        }
                        found
                    };
                    fragments.push(FragmentLayout {
                        kind,
                        label,
                        x: frag_x,
                        y: y_start,
                        width: frag_width,
                        height: frame_height,
                        separators,
                        first_msg_index: first_msg_idx,
                    });
                }
            }
            TeozTile::GroupStart { _label, y, .. } => {
                let ty = y.unwrap_or(0.0);
                group_stack.push((ty, _label.clone()));
            }
            TeozTile::GroupEnd { y, .. } => {
                let ty = y.unwrap_or(0.0);
                if let Some((y_start, label)) = group_stack.pop() {
                    // Java GroupingTile: drawU uses min (not min-EXTERNAL_MARGINX1)
                    let depth = group_stack.len();
                    let inset_left = GROUP_EXTERNAL_MARGINX1 * (depth + 1) as f64;
                    let inset_right = GROUP_EXTERNAL_MARGINX2 * (depth + 1) as f64;
                    groups.push(GroupLayout {
                        x: total_min_x + inset_left,
                        y_start,
                        y_end: ty,
                        width: diagram_width - inset_left - inset_right,
                        label,
                    });
                } else {
                    log::warn!("GroupEnd without matching GroupStart");
                }
            }
            _ => {}
        }
    }

    // Build activation bars from the event stream.
    // Re-scan events to track activate/deactivate pairs.
    //
    // Java LiveBoxes: each tile records a "step" y for the living spaces:
    // - CommunicationTile records step at tile_top + arrowY (= arrow y position)
    // - LifeEventTile records step at tile_top
    // Activation bars span from the step-y of the activate event to the step-y
    // of the deactivate event.
    {
        // act_state: per-participant stack of active levels.
        // Each entry: (y_start_stairs, y_start_addstep, level, color)
        // y_start_stairs = position used in getStairs (message arrowY)
        // y_start_addstep = position used in addStep collision check
        //   (arrowY for first-message inline, msg_bottom for parallel-message inline)
        let mut act_state: HashMap<String, Vec<(f64, f64, usize, Option<String>)>> = HashMap::new();
        let mut tile_idx = 0;
        // The step y of the current/preceding tile.
        // Initial: lifeline_top + PLAYINGSPACE_STARTING_Y (before any tile)
        let lifeline_top = STARTING_Y + max_preferred_height;
        let mut last_step_y: f64 = lifeline_top + PLAYINGSPACE_STARTING_Y;

        // Track message bottom (msg_y + msg_height).
        let mut last_msg_bottom_y: f64 = lifeline_top + PLAYINGSPACE_STARTING_Y;

        // Pre-compute which events are "inside TileParallel first message".
        // In Java, when a non-parallel message is followed (after inline events)
        // by a parallel `&` message, mergeParallel puts the first message + its
        // inline LifeEvents inside a TileParallel. Contact point adjustment
        // positions those LifeEvents at arrowY instead of msg_bottom.
        // For all other cases (no following parallel, or events from the
        // parallel message itself), addStep y = msg_bottom.
        let inside_tile_parallel: Vec<bool> = {
            let mut result = vec![false; sd.events.len()];
            let mut last_msg_idx: Option<usize> = None;
            let mut inline_indices: Vec<usize> = Vec::new();
            for (i, ev) in sd.events.iter().enumerate() {
                match ev {
                    SeqEvent::Message(m) => {
                        if m.parallel {
                            // The previous message + its inline events are inside TileParallel
                            if let Some(_msg_idx) = last_msg_idx {
                                for &idx in &inline_indices {
                                    result[idx] = true;
                                }
                            }
                        }
                        last_msg_idx = Some(i);
                        inline_indices.clear();
                    }
                    SeqEvent::Activate(..) | SeqEvent::Deactivate(_) | SeqEvent::Destroy(_) => {
                        if last_msg_idx.is_some() {
                            inline_indices.push(i);
                        }
                    }
                    _ => {
                        last_msg_idx = None;
                        inline_indices.clear();
                    }
                }
            }
            result
        };
        for (event_idx, event) in sd.events.iter().enumerate() {
            // Update last_step_y when we see a tile
            if let Some(tile) = tiles.get(tile_idx) {
                match tile {
                    TeozTile::Communication { height, y, .. } => {
                        let ty = y.unwrap_or(0.0);
                        // Step y = tile_top + arrowY = tile_top + (height - 8)
                        last_step_y = ty + (height - rose::ARROW_DELTA_Y - rose::ARROW_PADDING_Y);
                        last_msg_bottom_y = ty + height;
                    }
                    TeozTile::SelfMessage { height, y, .. } => {
                        let ty = y.unwrap_or(0.0);
                        let self_text_h = tp.msg_line_height * 1.0_f64;
                        let self_tm = TextMetrics::new(7.0, 7.0, 1.0, 0.0, self_text_h);
                        last_step_y = ty + rose::self_arrow_start_point(&self_tm).y;
                        last_msg_bottom_y = ty + height;
                    }
                    TeozTile::LifeEvent { .. } => {
                        // Inline LifeEvents use the message's step_y (already
                        // set by the preceding Communication). We don't update
                        // last_step_y here.
                    }
                    _ => {}
                }
            }

            match event {
                SeqEvent::Activate(name, act_color) => {
                    // Java getStairs: position = message arrowY (= last_step_y).
                    // Java addStep: LifeEvent tile at msg_bottom, EXCEPT when
                    // inside TileParallel (first msg of parallel block) where
                    // contact point adjustment puts it at arrowY.
                    let y_stairs = last_step_y;
                    let y_addstep = if inside_tile_parallel[event_idx] {
                        last_step_y // arrowY (inside TileParallel)
                    } else {
                        last_msg_bottom_y // msg_bottom (sequential)
                    };
                    let stack = act_state.entry(name.clone()).or_default();
                    let level = stack.len() + 1; // 1-based
                    log::debug!("teoz activate {name} level={level} y_stairs={y_stairs:.4} y_addstep={y_addstep:.4}");
                    stack.push((y_stairs, y_addstep, level, act_color.clone()));
                }
                SeqEvent::Deactivate(name) => {
                    if let Some(stack) = act_state.get_mut(name) {
                        if let Some((y_start, y_start_addstep, level, color)) = stack.pop() {
                            let idx = name_to_idx.get(name).copied().unwrap_or(0);
                            let cx = get_x(livings[idx].pos_c);
                            let x = cx - ACTIVATION_WIDTH / 2.0
                                + (level - 1) as f64 * (ACTIVATION_WIDTH / 2.0);
                            // Java getStairs: inline deactivate uses message
                            // arrowY (= last_step_y). Standalone deactivate uses
                            // its own tile position (= msg_bottom after TileParallel).
                            //
                            // Detection: if last_step_y == y_start, the deactivate
                            // is at the same message arrowY as the activate. Check
                            // if there is a msg_bottom available that differs:
                            // if so, this may be a standalone deactivate.
                            let mut y_end = last_step_y;
                            if (y_end - y_start).abs() < 0.001 {
                                // Both at message arrowY. Two sub-cases:
                                if last_msg_bottom_y > y_start + 0.001 {
                                    // Standalone deactivate: tile at msg_bottom.
                                    // Java getStairs position = msg_bottom.
                                    y_end = last_msg_bottom_y;
                                    // Java addStep collision: if the deactivate's
                                    // tile_y (= msg_bottom) matches the activate's
                                    // addStep y → bump +5.
                                    if (last_msg_bottom_y - y_start_addstep).abs() < 0.001 {
                                        y_end += 5.0;
                                    }
                                } else {
                                    // True inline collision (same message
                                    // activate+deactivate). Minimum height 13px.
                                    y_end = y_start + rose::ARROW_DELTA_Y
                                        + rose::ARROW_PADDING_Y + 5.0;
                                }
                            }
                            log::debug!("teoz deactivate {name} level={level} y_start={y_start:.4} y_end={y_end:.4}");
                            activations.push(ActivationLayout {
                                participant: name.clone(),
                                x,
                                y_start,
                                y_end,
                                level,
                                color,
                            });
                        }
                    }
                }
                SeqEvent::Destroy(name) => {
                    let ty = tiles
                        .get(tile_idx)
                        .and_then(|t| t.get_y())
                        .unwrap_or(last_step_y);
                    let idx = name_to_idx.get(name).copied().unwrap_or(0);
                    let cx = get_x(livings[idx].pos_c);
                    destroys.push(DestroyLayout { x: cx, y: ty });
                    // Close any open activations
                    if let Some(stack) = act_state.get_mut(name) {
                        while let Some((y_start, _y_addstep, level, color)) = stack.pop() {
                            let x = cx - ACTIVATION_WIDTH / 2.0
                                + (level - 1) as f64 * (ACTIVATION_WIDTH / 2.0);
                            activations.push(ActivationLayout {
                                participant: name.clone(),
                                x,
                                y_start,
                                y_end: ty,
                                level,
                                color,
                            });
                        }
                    }
                }
                _ => {}
            }
            tile_idx += 1;
        }
        // Close any unclosed activations at the lifeline bottom.
        // Java: unclosed activations extend the lifeline by a minimum height
        // (approximately 18px) beyond the last message bottom.
        const MIN_UNCLOSED_ACTIVATION_HEIGHT: f64 = 18.0;
        let mut extended_lifeline_bottom = lifeline_bottom;
        for (name, stack) in act_state.drain() {
            let idx = name_to_idx.get(&name).copied().unwrap_or(0);
            let cx = get_x(livings[idx].pos_c);
            for (y_start, _y_addstep, level, color) in stack {
                let x = cx - ACTIVATION_WIDTH / 2.0 + (level - 1) as f64 * (ACTIVATION_WIDTH / 2.0);
                // Ensure the activation has at least MIN_UNCLOSED_ACTIVATION_HEIGHT
                let y_end = (y_start + MIN_UNCLOSED_ACTIVATION_HEIGHT).max(lifeline_bottom);
                if y_end > extended_lifeline_bottom {
                    extended_lifeline_bottom = y_end;
                }
                activations.push(ActivationLayout {
                    participant: name.clone(),
                    x,
                    y_start,
                    y_end,
                    level,
                    color,
                });
            }
        }
        // Update lifeline_bottom to account for unclosed activations
        lifeline_bottom = extended_lifeline_bottom;
    }

    // Java: PlayingSpaceWithParticipants.width = maxX - minX (= diagram_width)
    // SequenceDiagramFileMakerTeoz: calculateDimension returns (width + 10, height + 10)
    // SVG exporter adds getDefaultMargins() = (5,5,5,5) for teoz mode.
    // Total viewport width = body_width + 10 + 5 + 5 = body_width + 20.
    let total_width = diagram_width + 2.0 * DOC_MARGIN_X;
    // Java height chain:
    //   startingY(8) + sum_tiles → finalY (in PlayingSpace coordinates)
    //   getPreferredHeight  = finalY + 10 (bottom padding)
    //   bodyHeight          = preferred + factor*headHeight
    //   calculateDimension  = bodyHeight + 10 (TextBlock wrapper)
    //   SVG viewport        = dimension + 10 (UTranslate(5,5))
    //
    // Combined: 8 + sum + 10 + factor*head + 10 + 10 = sum + factor*head + 38
    // Java SVG viewport height = sum + factor*head + 38
    // Our tiles_bottom = STARTING_Y + head + 8 + sum = sum + head + 18
    // total = tiles_bottom + (factor-1)*head + 20 = sum + factor*head + 38  ✓
    let show_footbox = !sd.hide_footbox;
    let factor = if show_footbox { 2 } else { 1 };
    // Java: when skin has shadowing (e.g. `skin rose`, delta=4), the LimitFinder
    // extends element bounds by 2*deltaShadow. Participant boxes at the bottom of
    // the diagram push the viewport down by this amount.
    let shadow_expansion = 2.0 * sd.delta_shadow;
    let total_height =
        tiles_bottom + (factor - 1) as f64 * max_preferred_height + 20.0 + shadow_expansion;
    log::debug!("teoz_layout: total_width={total_width:.4} total_height={total_height:.4} lifeline_bottom={lifeline_bottom:.4} max_preferred_height={max_preferred_height:.4}");

    Ok(SeqLayout {
        participants: part_layouts,
        messages,
        activations,
        destroys,
        notes,
        groups,
        fragments: {
            // Sort fragments so outer (earlier y, taller) come before inner.
            // Java draws outer GroupingTile first via recursive drawU().
            let mut sorted = fragments;
            sorted.sort_by(|a, b| {
                a.y.partial_cmp(&b.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.height.partial_cmp(&a.height).unwrap_or(std::cmp::Ordering::Equal))
            });
            sorted
        },
        dividers,
        delays,
        refs,
        autonumber_enabled,
        autonumber_start,
        lifeline_top: STARTING_Y + max_preferred_height,
        lifeline_bottom,
        total_width,
        total_height,
    })
}

// ── Text wrapping helper (copied from Puma) ──────────────────────────────────

fn wrap_text_to_width(
    text: &str,
    max_width: f64,
    font_family: &str,
    font_size: f64,
) -> Vec<String> {
    let full_w = font_metrics::text_width(text, font_family, font_size, false, false);
    if full_w <= max_width {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let w = font_metrics::text_width(&candidate, font_family, font_size, false, false);
        if w > max_width && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        vec![text.to_string()]
    } else {
        lines
    }
}
