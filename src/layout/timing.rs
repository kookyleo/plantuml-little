use std::collections::{BTreeSet, HashMap};

use log::debug;

use crate::model::richtext::plain_text;
use crate::model::timing::{TimingDiagram, TimingParticipantKind};
use crate::parser::creole::parse_creole;
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned timing diagram ready for rendering.
#[derive(Debug)]
pub struct TimingLayout {
    pub tracks: Vec<TimingTrackLayout>,
    pub messages: Vec<TimingMsgLayout>,
    pub constraints: Vec<TimingConstraintLayout>,
    pub notes: Vec<TimingNoteLayout>,
    pub time_axis: TimingTimeAxis,
    pub width: f64,
    pub height: f64,
}

/// A single participant track (horizontal lane).
#[derive(Debug)]
pub struct TimingTrackLayout {
    pub name: String,
    pub y: f64,
    pub height: f64,
    pub segments: Vec<TimingSegmentLayout>,
}

/// A horizontal segment in which a participant stays at a given state.
#[derive(Debug)]
pub struct TimingSegmentLayout {
    pub state: String,
    pub x_start: f64,
    pub x_end: f64,
    pub y: f64,
    pub is_robust: bool,
}

/// A message arrow between participants.
#[derive(Debug)]
pub struct TimingMsgLayout {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
    pub label: String,
}

/// A constraint annotation (double-ended arrow).
#[derive(Debug)]
pub struct TimingConstraintLayout {
    pub x_start: f64,
    pub x_end: f64,
    pub y: f64,
    pub label: String,
}

/// A positioned note attached to a participant track.
#[derive(Debug, Clone)]
pub struct TimingNoteLayout {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub connector: Option<(f64, f64, f64, f64)>,
}

/// The time axis drawn below all tracks.
#[derive(Debug)]
pub struct TimingTimeAxis {
    pub y: f64,
    pub ticks: Vec<TimingTick>,
}

/// A single tick mark on the time axis.
#[derive(Debug)]
pub struct TimingTick {
    pub x: f64,
    pub label: String,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MARGIN: f64 = 20.0;
const LABEL_AREA_WIDTH: f64 = 140.0;
const CHAR_WIDTH: f64 = 7.2;
const ROBUST_TRACK_HEIGHT: f64 = 40.0;
const CONCISE_TRACK_HEIGHT: f64 = 24.0;
const TRACK_GAP: f64 = 16.0;
const STATE_LEVEL_HEIGHT: f64 = 14.0;
const TIME_AXIS_HEIGHT: f64 = 30.0;
const NOTE_GAP: f64 = 16.0;
const NOTE_PAD_H: f64 = 8.0;
const NOTE_PAD_V: f64 = 6.0;
const MIN_NOTE_WIDTH: f64 = 60.0;
const MIN_NOTE_HEIGHT: f64 = 28.0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Perform layout for a timing diagram.
pub fn layout_timing(td: &TimingDiagram) -> Result<TimingLayout> {
    debug!(
        "layout_timing: {} participants, {} state_changes, {} messages, {} constraints",
        td.participants.len(),
        td.state_changes.len(),
        td.messages.len(),
        td.constraints.len(),
    );

    // Collect all unique absolute times to build the time scale
    let all_times = collect_all_times(td);
    let (time_min, time_max) = time_range(&all_times);

    // Compute label area width based on longest participant name
    let max_label_len = td
        .participants
        .iter()
        .map(|p| p.name.len())
        .max()
        .unwrap_or(0);
    let label_area = (max_label_len as f64 * CHAR_WIDTH + 2.0 * MARGIN).max(LABEL_AREA_WIDTH);

    let chart_x = MARGIN + label_area;

    // Scale: map [time_min .. time_max] to pixel range
    let time_span = (time_max - time_min).max(1) as f64;
    let chart_width = (time_span * 0.8).max(200.0);
    let px_per_unit = chart_width / time_span;

    let time_to_x = |t: i64| -> f64 { chart_x + (t - time_min) as f64 * px_per_unit };

    // Collect distinct states per participant for level assignment
    let states_per_participant = collect_states(td);

    // --- Tracks ---
    let mut tracks: Vec<TimingTrackLayout> = Vec::new();
    let mut track_y_map: HashMap<String, (f64, f64)> = HashMap::new(); // id -> (y_center, height)
    let mut track_rect_map: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    let mut current_y = MARGIN;

    for participant in &td.participants {
        let pid = participant.id().to_string();
        let track_h = match participant.kind {
            TimingParticipantKind::Robust => ROBUST_TRACK_HEIGHT,
            TimingParticipantKind::Concise => CONCISE_TRACK_HEIGHT,
        };
        let is_robust = participant.kind == TimingParticipantKind::Robust;

        // Gather state changes for this participant, sorted by time
        let mut changes: Vec<(i64, String)> = td
            .state_changes
            .iter()
            .filter(|sc| sc.participant == pid)
            .map(|sc| (sc.time, sc.state.clone()))
            .collect();
        changes.sort_by_key(|(t, _)| *t);

        // Build segments from consecutive state changes
        let state_levels = states_per_participant.get(&pid);
        let num_levels = state_levels.map_or(1, std::vec::Vec::len).max(1);

        let mut segments: Vec<TimingSegmentLayout> = Vec::new();
        for i in 0..changes.len() {
            let (t_start, ref state) = changes[i];
            let t_end = if i + 1 < changes.len() {
                changes[i + 1].0
            } else {
                time_max
            };

            let level = state_levels
                .and_then(|levels| levels.iter().position(|s| s == state))
                .unwrap_or(0);
            let level_y = if num_levels > 1 {
                current_y + track_h - (level as f64 + 1.0) * (track_h / (num_levels as f64 + 1.0))
            } else {
                current_y + track_h * 0.5
            };

            let x_start = time_to_x(t_start);
            let x_end = time_to_x(t_end);

            segments.push(TimingSegmentLayout {
                state: state.clone(),
                x_start,
                x_end,
                y: level_y,
                is_robust,
            });
        }

        let y_center = current_y + track_h * 0.5;
        track_y_map.insert(pid.clone(), (y_center, track_h));
        let rect_x = segments.first().map_or(chart_x, |seg| seg.x_start);
        let rect_right = segments
            .last()
            .map_or(chart_x + chart_width, |seg| seg.x_end);
        track_rect_map.insert(
            pid.clone(),
            (rect_x, current_y, rect_right - rect_x, track_h),
        );

        tracks.push(TimingTrackLayout {
            name: participant.name.clone(),
            y: current_y,
            height: track_h,
            segments,
        });

        current_y += track_h + TRACK_GAP;
    }

    // --- Messages ---
    let mut messages: Vec<TimingMsgLayout> = Vec::new();
    for msg in &td.messages {
        let from_center = track_y_map.get(&msg.from).map(|(y, _)| *y);
        let to_center = track_y_map.get(&msg.to).map(|(y, _)| *y);
        if let (Some(fy), Some(ty)) = (from_center, to_center) {
            let from_x = time_to_x(msg.from_time);
            let to_x = time_to_x(msg.to_time);
            messages.push(TimingMsgLayout {
                from_x,
                from_y: fy,
                to_x,
                to_y: ty,
                label: msg.label.clone(),
            });
        }
    }

    // --- Constraints ---
    let mut constraints: Vec<TimingConstraintLayout> = Vec::new();
    for c in &td.constraints {
        let y = track_y_map
            .get(&c.participant)
            .map_or(current_y, |(center, h)| {
                center + h * 0.5 + STATE_LEVEL_HEIGHT
            });
        constraints.push(TimingConstraintLayout {
            x_start: time_to_x(c.start_time),
            x_end: time_to_x(c.end_time),
            y,
            label: c.label.clone(),
        });
    }

    // --- Time axis ---
    let axis_y = current_y;
    let ticks = build_time_ticks(&all_times, &time_to_x);

    let mut time_axis = TimingTimeAxis { y: axis_y, ticks };

    // --- Total dimensions ---
    let mut notes = layout_notes(td, &track_rect_map, chart_x, chart_width, axis_y);
    let mut min_x = MARGIN;
    let mut min_y = MARGIN;
    let mut total_width = chart_x + chart_width;
    let mut total_height = axis_y + TIME_AXIS_HEIGHT;
    for note in &notes {
        min_x = min_x.min(note.x);
        min_y = min_y.min(note.y);
        total_width = total_width.max(note.x + note.width);
        total_height = total_height.max(note.y + note.height);
    }

    let shift_x = if min_x < MARGIN { MARGIN - min_x } else { 0.0 };
    let shift_y = if min_y < MARGIN { MARGIN - min_y } else { 0.0 };

    if shift_x > 0.0 || shift_y > 0.0 {
        for track in &mut tracks {
            track.y += shift_y;
            for segment in &mut track.segments {
                segment.x_start += shift_x;
                segment.x_end += shift_x;
                segment.y += shift_y;
            }
        }
        for message in &mut messages {
            message.from_x += shift_x;
            message.to_x += shift_x;
            message.from_y += shift_y;
            message.to_y += shift_y;
        }
        for constraint in &mut constraints {
            constraint.x_start += shift_x;
            constraint.x_end += shift_x;
            constraint.y += shift_y;
        }
        for tick in &mut time_axis.ticks {
            tick.x += shift_x;
        }
        time_axis.y += shift_y;
        for note in &mut notes {
            note.x += shift_x;
            note.y += shift_y;
            if let Some((x1, y1, x2, y2)) = note.connector.as_mut() {
                *x1 += shift_x;
                *x2 += shift_x;
                *y1 += shift_y;
                *y2 += shift_y;
            }
        }
        total_width += shift_x;
        total_height += shift_y;
    }

    total_width += MARGIN;
    total_height += MARGIN;

    debug!("layout_timing done: {total_width:.0}x{total_height:.0}");

    Ok(TimingLayout {
        tracks,
        messages,
        constraints,
        notes,
        time_axis,
        width: total_width,
        height: total_height,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all unique absolute times referenced in the diagram.
fn collect_all_times(td: &TimingDiagram) -> Vec<i64> {
    let mut times = BTreeSet::new();
    for sc in &td.state_changes {
        times.insert(sc.time);
    }
    for msg in &td.messages {
        times.insert(msg.from_time);
        times.insert(msg.to_time);
    }
    for c in &td.constraints {
        times.insert(c.start_time);
        times.insert(c.end_time);
    }
    times.into_iter().collect()
}

/// Return (min, max) of the collected times, defaulting to (0, 0).
fn time_range(times: &[i64]) -> (i64, i64) {
    if times.is_empty() {
        return (0, 0);
    }
    let min = *times.first().unwrap();
    let max = *times.last().unwrap();
    (min, max)
}

/// Collect distinct state names per participant, maintaining insertion order.
fn collect_states(td: &TimingDiagram) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for sc in &td.state_changes {
        let entry = map.entry(sc.participant.clone()).or_default();
        if !entry.contains(&sc.state) {
            entry.push(sc.state.clone());
        }
    }
    map
}

fn note_size(text: &str) -> (f64, f64) {
    let plain = plain_text(&parse_creole(text)).replace("\\n", "\n");
    let lines: Vec<&str> = plain.lines().collect();
    let max_width = lines
        .iter()
        .map(|line| line.chars().count() as f64 * CHAR_WIDTH)
        .fold(0.0_f64, f64::max);
    let width = (max_width + 2.0 * NOTE_PAD_H).max(MIN_NOTE_WIDTH);
    let height = (lines.len().max(1) as f64 * 16.0 + 2.0 * NOTE_PAD_V).max(MIN_NOTE_HEIGHT);
    (width, height)
}

fn layout_notes(
    td: &TimingDiagram,
    track_rects: &HashMap<String, (f64, f64, f64, f64)>,
    chart_x: f64,
    _chart_width: f64,
    axis_y: f64,
) -> Vec<TimingNoteLayout> {
    let mut participant_rects = track_rects.clone();
    for participant in &td.participants {
        if let Some(rect) = track_rects.get(participant.id()).copied() {
            participant_rects.insert(participant.name.clone(), rect);
        }
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut notes = Vec::new();
    for note in &td.notes {
        let (width, height) = note_size(&note.text);
        let key = format!(
            "{}:{}",
            note.target.as_deref().unwrap_or("_"),
            note.position
        );
        let stack_index = {
            let count = counts.entry(key).or_insert(0);
            let current = *count as f64;
            *count += 1;
            current
        };
        let target_rect = note
            .target
            .as_ref()
            .and_then(|target| participant_rects.get(target).copied());

        let (x, y, connector) = if let Some((tx, ty, tw, th)) = target_rect {
            let cx = tx + tw / 2.0;
            let cy = ty + th / 2.0;
            match note.position.as_str() {
                "left" => {
                    let x = tx - NOTE_GAP - width;
                    let y = ty + stack_index * (height + NOTE_GAP);
                    (x, y, Some((tx, cy, x + width, y + height / 2.0)))
                }
                "bottom" => {
                    let x = cx - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                    let y = ty + th + NOTE_GAP;
                    (x, y, Some((cx, ty + th, x + width / 2.0, y)))
                }
                "right" => {
                    let x = tx + tw + NOTE_GAP;
                    let y = ty + stack_index * (height + NOTE_GAP);
                    (x, y, Some((tx + tw, cy, x, y + height / 2.0)))
                }
                _ => {
                    let x = cx - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                    let y = ty - NOTE_GAP - height;
                    (x, y, Some((cx, ty, x + width / 2.0, y + height)))
                }
            }
        } else {
            let x = chart_x + stack_index * (width + NOTE_GAP);
            let y = MARGIN.max(axis_y - 80.0);
            (x, y, None)
        };

        notes.push(TimingNoteLayout {
            text: note.text.clone(),
            x,
            y,
            width,
            height,
            connector,
        });
    }
    notes
}

/// Build tick marks for the time axis from the collected time points.
fn build_time_ticks(times: &[i64], time_to_x: &dyn Fn(i64) -> f64) -> Vec<TimingTick> {
    times
        .iter()
        .map(|&t| TimingTick {
            x: time_to_x(t),
            label: t.to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::timing::{
        TimingConstraint, TimingDiagram, TimingMessage, TimingNote, TimingParticipant,
        TimingParticipantKind, TimingStateChange,
    };

    fn empty_diagram() -> TimingDiagram {
        TimingDiagram {
            participants: vec![],
            state_changes: vec![],
            messages: vec![],
            constraints: vec![],
            notes: vec![],
        }
    }

    fn simple_participant(
        name: &str,
        alias: Option<&str>,
        kind: TimingParticipantKind,
    ) -> TimingParticipant {
        TimingParticipant {
            name: name.to_string(),
            alias: alias.map(|a| a.to_string()),
            kind,
        }
    }

    fn simple_state_change(participant: &str, time: i64, state: &str) -> TimingStateChange {
        TimingStateChange {
            participant: participant.to_string(),
            time,
            state: state.to_string(),
        }
    }

    // 1. Empty diagram produces valid layout
    #[test]
    fn test_empty_diagram() {
        let td = empty_diagram();
        let layout = layout_timing(&td).unwrap();
        assert!(layout.tracks.is_empty());
        assert!(layout.messages.is_empty());
        assert!(layout.constraints.is_empty());
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 2. Single participant creates one track
    #[test]
    fn test_single_participant() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "DNS Resolver",
            Some("DNS"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("DNS", 0, "Idle"));
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.tracks.len(), 1);
        assert_eq!(layout.tracks[0].name, "DNS Resolver");
    }

    // 3. Robust track has expected height
    #[test]
    fn test_robust_track_height() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        let layout = layout_timing(&td).unwrap();
        assert!((layout.tracks[0].height - ROBUST_TRACK_HEIGHT).abs() < 0.01);
    }

    // 4. Concise track has expected height
    #[test]
    fn test_concise_track_height() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "B",
            Some("B"),
            TimingParticipantKind::Concise,
        ));
        td.state_changes.push(simple_state_change("B", 0, "Off"));
        let layout = layout_timing(&td).unwrap();
        assert!((layout.tracks[0].height - CONCISE_TRACK_HEIGHT).abs() < 0.01);
    }

    // 5. Multiple participants stack vertically
    #[test]
    fn test_vertical_stacking() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.participants.push(simple_participant(
            "B",
            Some("B"),
            TimingParticipantKind::Concise,
        ));
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        td.state_changes.push(simple_state_change("B", 0, "Off"));
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.tracks.len(), 2);
        assert!(
            layout.tracks[0].y < layout.tracks[1].y,
            "first track must be above second"
        );
    }

    // 6. State changes produce segments
    #[test]
    fn test_segments_from_state_changes() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        td.state_changes
            .push(simple_state_change("A", 100, "Active"));
        td.state_changes.push(simple_state_change("A", 300, "Idle"));
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.tracks[0].segments.len(), 3);
        assert_eq!(layout.tracks[0].segments[0].state, "Idle");
        assert_eq!(layout.tracks[0].segments[1].state, "Active");
        assert_eq!(layout.tracks[0].segments[2].state, "Idle");
    }

    // 7. Segment x coordinates increase with time
    #[test]
    fn test_segment_x_ordering() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("A", 0, "S0"));
        td.state_changes.push(simple_state_change("A", 100, "S1"));
        td.state_changes.push(simple_state_change("A", 200, "S2"));
        let layout = layout_timing(&td).unwrap();
        let segs = &layout.tracks[0].segments;
        assert!(segs[0].x_start < segs[1].x_start);
        assert!(segs[1].x_start < segs[2].x_start);
    }

    // 8. Message layout between two participants
    #[test]
    fn test_message_layout() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.participants.push(simple_participant(
            "B",
            Some("B"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        td.state_changes.push(simple_state_change("B", 0, "Idle"));
        td.messages.push(TimingMessage {
            from: "A".into(),
            to: "B".into(),
            label: "hello".into(),
            from_time: 100,
            to_time: 100,
        });
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.messages.len(), 1);
        assert_eq!(layout.messages[0].label, "hello");
        assert!(layout.messages[0].from_y < layout.messages[0].to_y);
    }

    // 9. Message with time offset has different x coordinates
    #[test]
    fn test_message_offset() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.participants.push(simple_participant(
            "B",
            Some("B"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        td.state_changes.push(simple_state_change("B", 0, "Idle"));
        td.messages.push(TimingMessage {
            from: "A".into(),
            to: "B".into(),
            label: "req".into(),
            from_time: 100,
            to_time: 150,
        });
        let layout = layout_timing(&td).unwrap();
        assert_ne!(layout.messages[0].from_x, layout.messages[0].to_x);
    }

    // 10. Constraint layout
    #[test]
    fn test_constraint_layout() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "WU",
            Some("WU"),
            TimingParticipantKind::Concise,
        ));
        td.state_changes.push(simple_state_change("WU", 0, "Idle"));
        td.constraints.push(TimingConstraint {
            participant: "WU".into(),
            start_time: 200,
            end_time: 350,
            label: "{150 ms}".into(),
        });
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.constraints.len(), 1);
        assert_eq!(layout.constraints[0].label, "{150 ms}");
        assert!(layout.constraints[0].x_start < layout.constraints[0].x_end);
    }

    // 11. Time axis has ticks for each distinct time point
    #[test]
    fn test_time_axis_ticks() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("A", 0, "S0"));
        td.state_changes.push(simple_state_change("A", 100, "S1"));
        td.state_changes.push(simple_state_change("A", 300, "S2"));
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.time_axis.ticks.len(), 3);
        assert_eq!(layout.time_axis.ticks[0].label, "0");
        assert_eq!(layout.time_axis.ticks[1].label, "100");
        assert_eq!(layout.time_axis.ticks[2].label, "300");
    }

    // 12. Bounding box includes all elements
    #[test]
    fn test_bounding_box() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.participants.push(simple_participant(
            "B",
            Some("B"),
            TimingParticipantKind::Concise,
        ));
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        td.state_changes.push(simple_state_change("B", 0, "Off"));
        td.state_changes
            .push(simple_state_change("A", 500, "Active"));
        let layout = layout_timing(&td).unwrap();

        for track in &layout.tracks {
            let track_bottom = track.y + track.height;
            assert!(
                track_bottom <= layout.height,
                "track bottom {track_bottom} exceeds layout height {}",
                layout.height,
            );
        }
    }

    // 13. Collect_all_times helper
    #[test]
    fn test_collect_all_times() {
        let mut td = empty_diagram();
        td.state_changes.push(simple_state_change("A", 0, "S0"));
        td.state_changes.push(simple_state_change("A", 100, "S1"));
        td.state_changes.push(simple_state_change("A", 100, "S1")); // duplicate
        let times = collect_all_times(&td);
        assert_eq!(times, vec![0, 100]); // deduped + sorted
    }

    // 14. Time range helper
    #[test]
    fn test_time_range() {
        assert_eq!(time_range(&[]), (0, 0));
        assert_eq!(time_range(&[10, 20, 50]), (10, 50));
        assert_eq!(time_range(&[42]), (42, 42));
    }

    // 15. Collect states helper
    #[test]
    fn test_collect_states() {
        let mut td = empty_diagram();
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        td.state_changes
            .push(simple_state_change("A", 100, "Active"));
        td.state_changes.push(simple_state_change("A", 200, "Idle")); // repeated state
        let states = collect_states(&td);
        let a_states = states.get("A").unwrap();
        assert_eq!(a_states, &vec!["Idle".to_string(), "Active".to_string()]);
    }

    // 16. Label area adapts to long names
    #[test]
    fn test_label_area_adapts() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "Very Long Participant Name Here",
            Some("VL"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("VL", 0, "Idle"));
        let layout = layout_timing(&td).unwrap();
        // Segments should start well to the right of MARGIN
        let seg = &layout.tracks[0].segments[0];
        assert!(
            seg.x_start > MARGIN + LABEL_AREA_WIDTH,
            "segment x_start should be past expanded label area"
        );
    }

    // 17. Segment is_robust flag
    #[test]
    fn test_segment_is_robust_flag() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "R",
            Some("R"),
            TimingParticipantKind::Robust,
        ));
        td.participants.push(simple_participant(
            "C",
            Some("C"),
            TimingParticipantKind::Concise,
        ));
        td.state_changes.push(simple_state_change("R", 0, "Idle"));
        td.state_changes.push(simple_state_change("C", 0, "Off"));
        let layout = layout_timing(&td).unwrap();
        assert!(layout.tracks[0].segments[0].is_robust);
        assert!(!layout.tracks[1].segments[0].is_robust);
    }

    // 18. Full diagram layout (integration)
    #[test]
    fn test_full_diagram_layout() {
        let td = TimingDiagram {
            participants: vec![
                simple_participant("DNS Resolver", Some("DNS"), TimingParticipantKind::Robust),
                simple_participant("Web Browser", Some("WB"), TimingParticipantKind::Robust),
                simple_participant("Web User", Some("WU"), TimingParticipantKind::Concise),
            ],
            state_changes: vec![
                simple_state_change("WU", 0, "Idle"),
                simple_state_change("WB", 0, "Idle"),
                simple_state_change("DNS", 0, "Idle"),
                simple_state_change("WU", 100, "Waiting"),
                simple_state_change("WB", 100, "Processing"),
                simple_state_change("WB", 300, "Waiting"),
                simple_state_change("DNS", 400, "Processing"),
                simple_state_change("DNS", 700, "Idle"),
            ],
            messages: vec![
                TimingMessage {
                    from: "WU".into(),
                    to: "WB".into(),
                    label: "URL".into(),
                    from_time: 100,
                    to_time: 100,
                },
                TimingMessage {
                    from: "WB".into(),
                    to: "DNS".into(),
                    label: "Resolve URL".into(),
                    from_time: 300,
                    to_time: 350,
                },
            ],
            constraints: vec![TimingConstraint {
                participant: "WU".into(),
                start_time: 200,
                end_time: 350,
                label: "{150 ms}".into(),
            }],
            notes: vec![],
        };

        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.tracks.len(), 3);
        assert_eq!(layout.messages.len(), 2);
        assert_eq!(layout.constraints.len(), 1);
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);

        // Tracks should be ordered as declared
        assert_eq!(layout.tracks[0].name, "DNS Resolver");
        assert_eq!(layout.tracks[1].name, "Web Browser");
        assert_eq!(layout.tracks[2].name, "Web User");
    }

    // 19. Segment last entry extends to time_max
    #[test]
    fn test_last_segment_extends_to_max() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("A", 0, "Idle"));
        td.state_changes
            .push(simple_state_change("A", 100, "Active"));
        let layout = layout_timing(&td).unwrap();
        let segs = &layout.tracks[0].segments;
        // The last segment's x_end should match the second segment's x_end (since time_max = 100)
        assert!((segs[1].x_start - segs[1].x_end).abs() < 0.01);
    }

    // 20. Participant without state changes gets no segments
    #[test]
    fn test_participant_no_state_changes() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "A",
            Some("A"),
            TimingParticipantKind::Robust,
        ));
        // No state changes for A, but one for B to establish a time range
        td.participants.push(simple_participant(
            "B",
            Some("B"),
            TimingParticipantKind::Concise,
        ));
        td.state_changes.push(simple_state_change("B", 0, "Off"));
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.tracks[0].segments.len(), 0);
        assert_eq!(layout.tracks[1].segments.len(), 1);
    }

    #[test]
    fn test_note_layout_for_track() {
        let mut td = empty_diagram();
        td.participants.push(simple_participant(
            "Web",
            Some("WEB"),
            TimingParticipantKind::Robust,
        ));
        td.state_changes.push(simple_state_change("WEB", 0, "Idle"));
        td.notes.push(TimingNote {
            text: "watch".to_string(),
            position: "right".to_string(),
            target: Some("WEB".to_string()),
        });
        let layout = layout_timing(&td).unwrap();
        assert_eq!(layout.notes.len(), 1);
        assert!(layout.notes[0].x > layout.tracks[0].segments[0].x_end);
        assert!(layout.notes[0].connector.is_some());
    }
}
