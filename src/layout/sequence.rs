use std::collections::HashMap;

use crate::model::sequence::{
    FragmentKind, ParticipantKind, SeqArrowHead, SeqArrowStyle, SeqDirection, SeqEvent,
};
use crate::model::SequenceDiagram;
use crate::Result;

// ── Constants ────────────────────────────────────────────────────────────────

const CHAR_WIDTH: f64 = 7.2;
const LINE_HEIGHT: f64 = 16.0;
const PARTICIPANT_PADDING: f64 = 16.0;
const PARTICIPANT_HEIGHT: f64 = 36.0;
const PARTICIPANT_GAP: f64 = 100.0;
const MESSAGE_SPACING: f64 = 40.0;
const SELF_MSG_WIDTH: f64 = 30.0;
const SELF_MSG_HEIGHT: f64 = 24.0;
const ACTIVATION_WIDTH: f64 = 10.0;
const NOTE_PADDING: f64 = 8.0;
const NOTE_WIDTH: f64 = 120.0;
const GROUP_PADDING: f64 = 10.0;
const FRAGMENT_HEADER_HEIGHT: f64 = 24.0;
const FRAGMENT_PADDING: f64 = 10.0;
const DIVIDER_HEIGHT: f64 = 30.0;
const DELAY_HEIGHT: f64 = 30.0;
const REF_HEIGHT: f64 = 32.0;
const MARGIN: f64 = 20.0;

/// Fragment stack entry: (y_start, kind, label, separators)
type FragmentStackEntry = (f64, FragmentKind, String, Vec<(f64, String)>);

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
    pub is_self: bool,
    pub is_dashed: bool,
    pub is_left: bool,
    pub has_open_head: bool,
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

/// Estimate note height: line count * LINE_HEIGHT + top/bottom padding
fn estimate_note_height(text: &str) -> f64 {
    let lines = text.lines().count().max(1) as f64;
    lines * LINE_HEIGHT + 2.0 * NOTE_PADDING
}

// ── Main layout function ─────────────────────────────────────────────────────

/// Perform columnar layout on a SequenceDiagram
pub fn layout_sequence(sd: &SequenceDiagram) -> Result<SeqLayout> {
    log::debug!(
        "layout_sequence: {} participants, {} events",
        sd.participants.len(),
        sd.events.len()
    );

    // 1. Participant positioning (center-to-center is PARTICIPANT_GAP)
    let mut participants: Vec<ParticipantLayout> = Vec::with_capacity(sd.participants.len());
    let mut prev_center: Option<f64> = None;
    for p in &sd.participants {
        let display = p.display_name.as_deref().unwrap_or(&p.name);
        let box_width = (display.len() as f64 * CHAR_WIDTH + 2.0 * PARTICIPANT_PADDING).max(60.0);

        // Icon-based shapes need extra height for the figure + label below
        let box_height = match p.kind {
            ParticipantKind::Actor => PARTICIPANT_HEIGHT + 40.0,
            ParticipantKind::Boundary
            | ParticipantKind::Control
            | ParticipantKind::Entity
            | ParticipantKind::Database
            | ParticipantKind::Collections
            | ParticipantKind::Queue => PARTICIPANT_HEIGHT + 20.0,
            ParticipantKind::Default => PARTICIPANT_HEIGHT,
        };

        let center_x = match prev_center {
            None => MARGIN + box_width / 2.0,
            Some(pc) => pc + PARTICIPANT_GAP,
        };

        participants.push(ParticipantLayout {
            name: p.name.clone(),
            x: center_x,
            box_width,
            box_height,
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
    let mut y_cursor = MARGIN + max_ph + 20.0;

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

    for event in &sd.events {
        match event {
            SeqEvent::Message(msg) => {
                let from_x = find_participant_x(&participants, &msg.from);
                let to_x = find_participant_x(&participants, &msg.to);
                let is_self = msg.from == msg.to;
                let is_dashed = msg.arrow_style == SeqArrowStyle::Dashed
                    || msg.arrow_style == SeqArrowStyle::Dotted;
                let is_left = msg.direction == SeqDirection::RightToLeft;
                let has_open_head = msg.arrow_head == SeqArrowHead::Open;

                messages.push(MessageLayout {
                    from_x,
                    to_x: if is_self {
                        from_x + SELF_MSG_WIDTH
                    } else {
                        to_x
                    },
                    y: y_cursor,
                    text: msg.text.clone(),
                    is_self,
                    is_dashed,
                    is_left,
                    has_open_head,
                });

                if is_self {
                    y_cursor += MESSAGE_SPACING + SELF_MSG_HEIGHT;
                } else {
                    y_cursor += MESSAGE_SPACING;
                }
            }

            SeqEvent::Activate(name) => {
                log::debug!("activate '{name}' at y={y_cursor:.1}");
                activation_stack
                    .entry(name.clone())
                    .or_default()
                    .push(y_cursor);
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
                destroys.push(DestroyLayout { x: px, y: y_cursor });
                y_cursor += MESSAGE_SPACING;
                log::debug!("destroy '{name}' at y={y_cursor:.1}");
            }

            SeqEvent::NoteRight { participant, text } => {
                let px = find_participant_x(&participants, participant);
                let note_height = estimate_note_height(text);
                notes.push(NoteLayout {
                    x: px + ACTIVATION_WIDTH,
                    y: y_cursor,
                    width: NOTE_WIDTH,
                    height: note_height,
                    text: text.clone(),
                    is_left: false,
                });
                y_cursor += note_height;
            }

            SeqEvent::NoteLeft { participant, text } => {
                let px = find_participant_x(&participants, participant);
                let note_height = estimate_note_height(text);
                notes.push(NoteLayout {
                    x: px - ACTIVATION_WIDTH - NOTE_WIDTH,
                    y: y_cursor,
                    width: NOTE_WIDTH,
                    height: note_height,
                    text: text.clone(),
                    is_left: true,
                });
                y_cursor += note_height;
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
                    let width = (x2 - x1).abs().max(NOTE_WIDTH);
                    notes.push(NoteLayout {
                        x: center - width / 2.0,
                        y: y_cursor,
                        width,
                        height: note_height,
                        text: text.clone(),
                        is_left: false,
                    });
                    y_cursor += note_height;
                }
            }

            SeqEvent::GroupStart { label } => {
                group_stack.push((y_cursor, label.clone()));
                y_cursor += GROUP_PADDING;
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
            }

            SeqEvent::FragmentStart { kind, label } => {
                fragment_stack.push((y_cursor, kind.clone(), label.clone(), Vec::new()));
                y_cursor += FRAGMENT_HEADER_HEIGHT;
            }

            SeqEvent::FragmentSeparator { label } => {
                if let Some(entry) = fragment_stack.last_mut() {
                    entry.3.push((y_cursor, label.clone()));
                    y_cursor += FRAGMENT_PADDING;
                } else {
                    log::warn!("FragmentSeparator without matching FragmentStart");
                }
            }

            SeqEvent::FragmentEnd => {
                if let Some((y_start, kind, label, separators)) = fragment_stack.pop() {
                    y_cursor += FRAGMENT_PADDING;
                    let frag_x = leftmost - FRAGMENT_PADDING;
                    let frag_height = y_cursor - y_start;
                    fragments.push(FragmentLayout {
                        kind,
                        label,
                        x: frag_x,
                        y: y_start,
                        width: full_width,
                        height: frag_height,
                        separators,
                    });
                } else {
                    log::warn!("FragmentEnd without matching FragmentStart");
                }
            }

            SeqEvent::Ref {
                participants: parts,
                label,
            } => {
                if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
                    let x1 = find_participant_x(&participants, first);
                    let x2 = find_participant_x(&participants, last);
                    let left_x = x1.min(x2) - 30.0;
                    let right_x = x1.max(x2) + 30.0;
                    refs.push(RefLayout {
                        x: left_x,
                        y: y_cursor,
                        width: right_x - left_x,
                        height: REF_HEIGHT,
                        label: label.clone(),
                    });
                    y_cursor += REF_HEIGHT + FRAGMENT_PADDING;
                }
            }

            SeqEvent::Spacing { pixels } => {
                y_cursor += *pixels as f64;
            }

            SeqEvent::AutoNumber { start } => {
                autonumber_enabled = true;
                if let Some(n) = start {
                    autonumber_start = *n;
                }
            }
        }
    }

    // Close any remaining activations (unmatched)
    for (name, stack) in &activation_stack {
        for &y_start in stack {
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
    let lifeline_top = MARGIN + max_participant_height;
    let lifeline_bottom = y_cursor + 20.0;

    let total_width = participants
        .last()
        .map_or(2.0 * MARGIN, |p| p.x + p.box_width / 2.0 + MARGIN);

    let total_height = lifeline_bottom + max_participant_height + MARGIN;

    // Close any remaining fragments (unmatched)
    for (y_start, kind, label, separators) in fragment_stack.drain(..) {
        let frag_x = leftmost - FRAGMENT_PADDING;
        let frag_height = y_cursor - y_start;
        fragments.push(FragmentLayout {
            kind,
            label,
            x: frag_x,
            y: y_start,
            width: full_width,
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

        // box_width = max(5 * 7.2 + 2 * 16.0, 60.0) = max(68.0, 60.0) = 68.0
        let expected_bw = (5.0_f64 * CHAR_WIDTH + 2.0 * PARTICIPANT_PADDING).max(60.0);
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
        assert!(
            (bob_x - alice_x - PARTICIPANT_GAP).abs() < 0.01,
            "gap between centers should be PARTICIPANT_GAP"
        );

        let msg = &layout.messages[0];
        assert!(!msg.is_self);
        assert!((msg.from_x - alice_x).abs() < 0.01);
        assert!((msg.to_x - bob_x).abs() < 0.01);
        assert_eq!(msg.text, "hello");
        assert!(!msg.is_dashed);
    }

    #[test]
    fn self_message_increases_height() {
        let sd_self = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::Message(make_message("A", "A", "self"))],
        };
        let sd_normal = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![SeqEvent::Message(make_message("A", "B", "normal"))],
        };

        let layout_self = layout_sequence(&sd_self).unwrap();
        let layout_normal = layout_sequence(&sd_normal).unwrap();

        // Self-message should produce a taller layout (more y consumed)
        assert!(
            layout_self.lifeline_bottom > layout_normal.lifeline_bottom,
            "self-msg lifeline_bottom {} should exceed normal {}",
            layout_self.lifeline_bottom,
            layout_normal.lifeline_bottom
        );

        let msg = &layout_self.messages[0];
        assert!(msg.is_self);
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
}
