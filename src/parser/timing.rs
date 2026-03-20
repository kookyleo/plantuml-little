use log::{debug, trace, warn};

use crate::model::timing::{
    TimingConstraint, TimingDiagram, TimingMessage, TimingNote, TimingParticipant,
    TimingParticipantKind, TimingStateChange,
};
use crate::Result;

/// Parser mode for skipping multi-line blocks
#[derive(Debug)]
enum ParseMode {
    Normal,
    StyleBlock,
    SkinparamBlock,
}

/// Parse a timing diagram source into the TimingDiagram IR.
///
/// All relative time offsets (`@+N`) are resolved to absolute values
/// during parsing.
pub fn parse_timing_diagram(source: &str) -> Result<TimingDiagram> {
    let block = super::common::extract_block(source).unwrap_or_else(|| source.to_string());

    let mut participants: Vec<TimingParticipant> = Vec::new();
    let mut state_changes: Vec<TimingStateChange> = Vec::new();
    let mut messages: Vec<TimingMessage> = Vec::new();
    let mut constraints: Vec<TimingConstraint> = Vec::new();
    let mut notes: Vec<TimingNote> = Vec::new();

    let mut current_time: i64 = 0;
    let mut current_participant_context: Option<String> = None;
    let mut mode = ParseMode::Normal;
    let mut in_note_block = false;
    let mut note_block_position = String::new();
    let mut note_block_target: Option<String> = None;
    let mut note_block_lines: Vec<String> = Vec::new();

    for (line_num, line) in block.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        // Handle multi-line block skipping
        match mode {
            ParseMode::StyleBlock => {
                if trimmed == "</style>" {
                    trace!("line {line_num}: end <style> block");
                    mode = ParseMode::Normal;
                }
                continue;
            }
            ParseMode::SkinparamBlock => {
                if trimmed == "}" {
                    trace!("line {line_num}: end skinparam block");
                    mode = ParseMode::Normal;
                }
                continue;
            }
            ParseMode::Normal => {}
        }

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        // Skip <style> blocks
        if trimmed.starts_with("<style>") {
            trace!("line {line_num}: entering <style> block");
            if !trimmed.contains("</style>") {
                mode = ParseMode::StyleBlock;
            }
            continue;
        }

        // Skip skinparam lines
        if trimmed.starts_with("skinparam ") {
            trace!("line {line_num}: skinparam (skipped): {trimmed}");
            if trimmed.contains('{') && !trimmed.contains('}') {
                mode = ParseMode::SkinparamBlock;
            }
            continue;
        }

        // Skip other known noise
        if trimmed.starts_with("hide ")
            || trimmed.starts_with("show ")
            || trimmed.starts_with("scale ")
            || trimmed.starts_with("title ")
            || trimmed.starts_with("header ")
            || trimmed.starts_with("footer ")
            || trimmed.starts_with("legend")
        {
            continue;
        }

        // Handle multi-line note accumulation
        if in_note_block {
            if trimmed == "end note" || trimmed == "endnote" {
                let text = note_block_lines.join("\n");
                debug!("line {line_num}: end note block, text={text:?}");
                notes.push(TimingNote {
                    text,
                    position: note_block_position.clone(),
                    target: note_block_target.take(),
                });
                note_block_lines.clear();
                in_note_block = false;
            } else {
                note_block_lines.push(trimmed.to_string());
            }
            continue;
        }

        // Parse participant declarations: robust "Name" as ALIAS
        if let Some(p) = try_parse_participant(trimmed) {
            debug!(
                "line {}: participant {:?} (alias={:?}, kind={:?})",
                line_num, p.name, p.alias, p.kind
            );
            participants.push(p);
            continue;
        }

        // Parse time marker: @0, @+100, @ALIAS
        if let Some(time_or_ctx) = try_parse_time_marker(trimmed, &participants) {
            match time_or_ctx {
                TimeMarkerResult::AbsoluteTime(t) => {
                    debug!("line {line_num}: time marker -> absolute {t}");
                    current_time = t;
                    current_participant_context = None;
                }
                TimeMarkerResult::RelativeTime(offset) => {
                    current_time += offset;
                    debug!(
                        "line {line_num}: time marker -> relative +{offset}, now={current_time}"
                    );
                    current_participant_context = None;
                }
                TimeMarkerResult::ParticipantContext(alias) => {
                    debug!("line {line_num}: participant context -> {alias}");
                    current_participant_context = Some(alias);
                }
            }
            continue;
        }

        // Parse constraint: @TIME1 <-> @TIME2 : label (within participant context)
        if let Some(c) = try_parse_constraint(trimmed, &current_participant_context) {
            debug!(
                "line {}: constraint on {:?} from {} to {}: {}",
                line_num, c.participant, c.start_time, c.end_time, c.label
            );
            constraints.push(c);
            continue;
        }

        // Parse message: FROM -> TO : label  or  FROM -> TO@+OFFSET : label
        if let Some(msg) = try_parse_message(trimmed, current_time) {
            debug!(
                "line {}: message {} -> {} at {}->{}: {}",
                line_num, msg.from, msg.to, msg.from_time, msg.to_time, msg.label
            );
            messages.push(msg);
            continue;
        }

        // Parse state change: ALIAS is STATE
        if let Some(sc) = try_parse_state_change(trimmed, current_time) {
            debug!(
                "line {}: state change {} -> {} at {}",
                line_num, sc.participant, sc.state, sc.time
            );
            state_changes.push(sc);
            continue;
        }

        // Note parsing: `note on Participant : text` or `note top/bottom of Participant : text`
        if let Some(note_result) = try_parse_timing_note(trimmed) {
            match note_result {
                TimingNoteParseResult::SingleLine(note) => {
                    debug!("line {}: single-line note for {:?}", line_num, note.target);
                    notes.push(note);
                }
                TimingNoteParseResult::MultiLineStart { position, target } => {
                    debug!("line {line_num}: start multi-line note for {target:?}");
                    in_note_block = true;
                    note_block_position = position;
                    note_block_target = target;
                    note_block_lines.clear();
                }
            }
            continue;
        }

        warn!("line {line_num}: unrecognized timing syntax: {trimmed}");
    }

    debug!(
        "parsed timing diagram: {} participants, {} state_changes, {} messages, {} constraints",
        participants.len(),
        state_changes.len(),
        messages.len(),
        constraints.len()
    );

    Ok(TimingDiagram {
        participants,
        state_changes,
        messages,
        constraints,
        notes,
    })
}

// ── Time marker parsing ─────────────────────────────────────────────

enum TimeMarkerResult {
    AbsoluteTime(i64),
    RelativeTime(i64),
    ParticipantContext(String),
}

fn try_parse_time_marker(
    trimmed: &str,
    participants: &[TimingParticipant],
) -> Option<TimeMarkerResult> {
    // Must start with '@' and be a standalone time marker (not a constraint line)
    if !trimmed.starts_with('@') {
        return None;
    }

    let rest = trimmed[1..].trim();

    // Constraint lines contain `<->`, skip them
    if rest.contains("<->") {
        return None;
    }

    // @+OFFSET (relative time)
    if let Some(num_str) = rest.strip_prefix('+') {
        let num_str = num_str.trim();
        if let Ok(offset) = num_str.parse::<i64>() {
            return Some(TimeMarkerResult::RelativeTime(offset));
        }
    }

    // @NUMBER (absolute time)
    if let Ok(t) = rest.parse::<i64>() {
        return Some(TimeMarkerResult::AbsoluteTime(t));
    }

    // @ALIAS (participant context switch)
    let alias = rest.to_string();
    let is_participant = participants.iter().any(|p| p.id() == alias);
    if is_participant {
        return Some(TimeMarkerResult::ParticipantContext(alias));
    }

    None
}

// ── Participant parsing ─────────────────────────────────────────────

fn try_parse_participant(trimmed: &str) -> Option<TimingParticipant> {
    // Pattern: robust "Name" as ALIAS
    //          concise "Name" as ALIAS
    //          robust "Name"
    //          concise "Name"
    let kind = if trimmed.starts_with("robust ") {
        TimingParticipantKind::Robust
    } else if trimmed.starts_with("concise ") {
        TimingParticipantKind::Concise
    } else {
        return None;
    };

    let after_keyword = if kind == TimingParticipantKind::Robust {
        trimmed["robust ".len()..].trim()
    } else {
        trimmed["concise ".len()..].trim()
    };

    // Extract quoted name
    if let Some(after_open_quote) = after_keyword.strip_prefix('"') {
        if let Some(end_quote) = after_open_quote.find('"') {
            let name = after_open_quote[..end_quote].to_string();
            let remainder = after_open_quote[end_quote + 1..].trim();

            let alias = remainder
                .strip_prefix("as ")
                .map(|rest| rest.trim().to_string());

            return Some(TimingParticipant { name, alias, kind });
        }
    }

    // Unquoted name (no spaces)
    let parts: Vec<&str> = after_keyword.splitn(3, ' ').collect();
    match parts.len() {
        1 => Some(TimingParticipant {
            name: parts[0].to_string(),
            alias: None,
            kind,
        }),
        3 if parts[1] == "as" => Some(TimingParticipant {
            name: parts[0].to_string(),
            alias: Some(parts[2].to_string()),
            kind,
        }),
        _ => None,
    }
}

// ── State change parsing ────────────────────────────────────────────

fn try_parse_state_change(trimmed: &str, current_time: i64) -> Option<TimingStateChange> {
    // Pattern: ALIAS is STATE
    let is_pos = trimmed.find(" is ")?;
    let participant = trimmed[..is_pos].trim().to_string();
    let state = trimmed[is_pos + 4..].trim().to_string();

    // Sanity: participant name should not contain special chars that indicate
    // this line is something else (time marker, message, constraint)
    if participant.contains('@')
        || participant.contains('<')
        || participant.contains('>')
        || participant.starts_with('"')
    {
        return None;
    }

    Some(TimingStateChange {
        participant,
        time: current_time,
        state,
    })
}

// ── Message parsing ─────────────────────────────────────────────────

fn try_parse_message(trimmed: &str, current_time: i64) -> Option<TimingMessage> {
    // Pattern: FROM -> TO : label
    // Pattern: FROM -> TO@+OFFSET : label
    // Pattern: FROM <- TO : label

    // Check for constraint first (<->)
    if trimmed.contains("<->") {
        return None;
    }

    let arrow_pos = trimmed.find(" -> ").or_else(|| trimmed.find(" <- "))?;
    let is_left_arrow = trimmed[arrow_pos..].starts_with(" <- ");

    let left = trimmed[..arrow_pos].trim();
    let right_and_label = trimmed[arrow_pos + 4..].trim();

    // Split right side by " : " to get target and label
    let (right, label) = if let Some(colon_pos) = right_and_label.find(" : ") {
        (
            right_and_label[..colon_pos].trim(),
            right_and_label[colon_pos + 3..].trim().to_string(),
        )
    } else {
        (right_and_label, String::new())
    };

    // Parse target: might have @+OFFSET (e.g., DNS@+50)
    let (target, target_offset) = if let Some(at_pos) = right.find('@') {
        let name = &right[..at_pos];
        let offset_str = &right[at_pos + 1..];
        if let Some(after_plus) = offset_str.strip_prefix('+') {
            let val = after_plus.parse::<i64>().unwrap_or(0);
            (name.to_string(), val)
        } else if let Ok(val) = offset_str.parse::<i64>() {
            // absolute time on target
            (name.to_string(), val - current_time)
        } else {
            (right.to_string(), 0)
        }
    } else {
        (right.to_string(), 0)
    };

    let (from, to, from_time, to_time) = if is_left_arrow {
        (
            target.clone(),
            left.to_string(),
            current_time + target_offset,
            current_time,
        )
    } else {
        (
            left.to_string(),
            target,
            current_time,
            current_time + target_offset,
        )
    };

    Some(TimingMessage {
        from,
        to,
        label,
        from_time,
        to_time,
    })
}

// ── Constraint parsing ──────────────────────────────────────────────

fn try_parse_constraint(
    trimmed: &str,
    participant_context: &Option<String>,
) -> Option<TimingConstraint> {
    // Pattern: @TIME1 <-> @+TIME2 : label
    // Must be within a participant context (@ALIAS block)
    if !trimmed.contains("<->") {
        return None;
    }
    let participant = participant_context.as_ref()?.clone();

    let arrow_pos = trimmed.find("<->")?;
    let left = trimmed[..arrow_pos].trim();
    let right_and_label = trimmed[arrow_pos + 3..].trim();

    // Split by " : " to get right time and label
    let (right, label) = if let Some(colon_pos) = right_and_label.find(" : ") {
        (
            right_and_label[..colon_pos].trim(),
            right_and_label[colon_pos + 3..].trim().to_string(),
        )
    } else {
        (right_and_label, String::new())
    };

    // Parse left time: @NUMBER or @+OFFSET
    let start_time = parse_time_ref(left)?;

    // Parse right time: @NUMBER or @+OFFSET (relative to start_time for constraints)
    let end_time = parse_constraint_end_time(right, start_time)?;

    Some(TimingConstraint {
        participant,
        start_time,
        end_time,
        label,
    })
}

/// Parse a time reference like @100 or @+50
fn parse_time_ref(s: &str) -> Option<i64> {
    let s = s.trim();
    if !s.starts_with('@') {
        return None;
    }
    let rest = &s[1..];
    if let Some(after_plus) = rest.strip_prefix('+') {
        after_plus.parse::<i64>().ok()
    } else {
        rest.parse::<i64>().ok()
    }
}

/// Parse the end time reference in a constraint.
/// @+OFFSET means relative to start_time, @NUMBER means absolute.
fn parse_constraint_end_time(s: &str, start_time: i64) -> Option<i64> {
    let s = s.trim();
    if !s.starts_with('@') {
        return None;
    }
    let rest = &s[1..];
    if let Some(after_plus) = rest.strip_prefix('+') {
        let offset = after_plus.parse::<i64>().ok()?;
        Some(start_time + offset)
    } else {
        rest.parse::<i64>().ok()
    }
}

// ── Note parsing ────────────────────────────────────────────────────

enum TimingNoteParseResult {
    SingleLine(TimingNote),
    MultiLineStart {
        position: String,
        target: Option<String>,
    },
}

/// Parse a timing diagram note line.
///
/// Supported forms:
///   `note on Participant : text`       (single-line, attached to participant)
///   `note on Participant`              (multi-line start)
///   `note top of Participant : text`   (single-line with position)
///   `note bottom of Participant`       (multi-line start with position)
fn try_parse_timing_note(line: &str) -> Option<TimingNoteParseResult> {
    let trimmed = line.trim();
    if !trimmed.starts_with("note ") {
        return None;
    }

    let rest = trimmed[5..].trim();

    // `note on Participant : text` or `note on Participant`
    if let Some(after_on) = rest.strip_prefix("on ") {
        let after_on = after_on.trim();

        if let Some(colon_pos) = after_on.find(':') {
            let target = after_on[..colon_pos].trim().to_string();
            let text = after_on[colon_pos + 1..].trim().replace("\\n", "\n").replace(crate::NEWLINE_CHAR, "\n");
            return Some(TimingNoteParseResult::SingleLine(TimingNote {
                text,
                position: "top".to_string(),
                target: Some(target),
            }));
        }

        let target = after_on.trim().to_string();
        return Some(TimingNoteParseResult::MultiLineStart {
            position: "top".to_string(),
            target: if target.is_empty() {
                None
            } else {
                Some(target)
            },
        });
    }

    // `note top/bottom/left/right of Participant : text`
    for pos in &["top", "bottom", "left", "right"] {
        if !rest.starts_with(pos) {
            continue;
        }
        let after_pos = rest[pos.len()..].trim();

        if let Some(after_of) = after_pos.strip_prefix("of ") {
            let after_of = after_of.trim();

            if let Some(colon_pos) = after_of.find(':') {
                let target = after_of[..colon_pos].trim().to_string();
                let text = after_of[colon_pos + 1..].trim().replace("\\n", "\n").replace(crate::NEWLINE_CHAR, "\n");
                return Some(TimingNoteParseResult::SingleLine(TimingNote {
                    text,
                    position: pos.to_string(),
                    target: Some(target),
                }));
            }

            let target = after_of.trim().to_string();
            return Some(TimingNoteParseResult::MultiLineStart {
                position: pos.to_string(),
                target: if target.is_empty() {
                    None
                } else {
                    Some(target)
                },
            });
        }
    }

    None
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_diagram() {
        let src = "@startuml\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert!(td.participants.is_empty());
        assert!(td.state_changes.is_empty());
    }

    #[test]
    fn parse_robust_participant() {
        let src = "@startuml\nrobust \"DNS Resolver\" as DNS\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 1);
        assert_eq!(td.participants[0].name, "DNS Resolver");
        assert_eq!(td.participants[0].alias.as_deref(), Some("DNS"));
        assert_eq!(td.participants[0].kind, TimingParticipantKind::Robust);
    }

    #[test]
    fn parse_concise_participant() {
        let src = "@startuml\nconcise \"Web User\" as WU\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 1);
        assert_eq!(td.participants[0].name, "Web User");
        assert_eq!(td.participants[0].alias.as_deref(), Some("WU"));
        assert_eq!(td.participants[0].kind, TimingParticipantKind::Concise);
    }

    #[test]
    fn parse_state_changes_at_absolute_time() {
        let src = "@startuml\nrobust \"A\" as A\n@0\nA is Idle\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.state_changes.len(), 1);
        assert_eq!(td.state_changes[0].participant, "A");
        assert_eq!(td.state_changes[0].time, 0);
        assert_eq!(td.state_changes[0].state, "Idle");
    }

    #[test]
    fn parse_relative_time_offset() {
        let src = "@startuml\nrobust \"A\" as A\n@0\nA is Idle\n@+100\nA is Active\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.state_changes.len(), 2);
        assert_eq!(td.state_changes[0].time, 0);
        assert_eq!(td.state_changes[1].time, 100);
        assert_eq!(td.state_changes[1].state, "Active");
    }

    #[test]
    fn parse_multiple_relative_offsets() {
        let src =
            "@startuml\nrobust \"A\" as A\n@0\nA is S0\n@+100\nA is S1\n@+200\nA is S2\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.state_changes.len(), 3);
        assert_eq!(td.state_changes[0].time, 0);
        assert_eq!(td.state_changes[1].time, 100);
        assert_eq!(td.state_changes[2].time, 300); // 100 + 200
    }

    #[test]
    fn parse_simple_message() {
        let src =
            "@startuml\nrobust \"A\" as A\nrobust \"B\" as B\n@0\nA is Idle\nB is Idle\n@+100\nA -> B : hello\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.messages.len(), 1);
        assert_eq!(td.messages[0].from, "A");
        assert_eq!(td.messages[0].to, "B");
        assert_eq!(td.messages[0].label, "hello");
        assert_eq!(td.messages[0].from_time, 100);
        assert_eq!(td.messages[0].to_time, 100);
    }

    #[test]
    fn parse_message_with_offset() {
        let src =
            "@startuml\nrobust \"WB\" as WB\nrobust \"DNS\" as DNS\n@300\nWB -> DNS@+50 : Resolve URL\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.messages.len(), 1);
        assert_eq!(td.messages[0].from, "WB");
        assert_eq!(td.messages[0].to, "DNS");
        assert_eq!(td.messages[0].from_time, 300);
        assert_eq!(td.messages[0].to_time, 350);
        assert_eq!(td.messages[0].label, "Resolve URL");
    }

    #[test]
    fn parse_constraint() {
        let src = "@startuml\nrobust \"A\" as A\nconcise \"WU\" as WU\n@0\nA is Idle\nWU is Idle\n@WU\n@200 <-> @+150 : {150 ms}\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.constraints.len(), 1);
        assert_eq!(td.constraints[0].participant, "WU");
        assert_eq!(td.constraints[0].start_time, 200);
        assert_eq!(td.constraints[0].end_time, 350);
        assert_eq!(td.constraints[0].label, "{150 ms}");
    }

    #[test]
    fn parse_skinparam_skipped() {
        let src =
            "@startuml\nskinparam defaultFontName \"SansSerif\"\nskinparam defaultFontSize 10\nrobust \"A\" as A\n@0\nA is Idle\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 1);
        assert_eq!(td.state_changes.len(), 1);
    }

    #[test]
    fn parse_style_block_skipped() {
        let src = "@startuml\n<style>\narrow {\n  FontColor green\n}\n</style>\nrobust \"A\" as A\n@0\nA is Idle\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 1);
        assert_eq!(td.state_changes.len(), 1);
    }

    #[test]
    fn parse_full_fixture_0001() {
        let src = r#"@startuml
skinparam defaultFontName "SansSerif"
skinparam defaultFontSize 10
skinparam defaultFontColor green

robust "DNS Resolver" as DNS
robust "Web Browser" as WB
concise "Web User" as WU

@0
WU is Idle
WB is Idle
DNS is Idle

@+100
WU -> WB : URL
WU is Waiting
WB is Processing

@+200
WB is Waiting
WB -> DNS@+50 : Resolve URL

@+100
DNS is Processing

@+300
DNS is Idle

@WU
@200 <-> @+150 : {150 ms}
@enduml"#;
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 3);
        assert_eq!(td.participants[0].name, "DNS Resolver");
        assert_eq!(td.participants[1].name, "Web Browser");
        assert_eq!(td.participants[2].name, "Web User");

        // State changes: 3 at @0, 2 at @+100, 1 at @+200, 1 at @+100, 1 at @+300 = 8
        assert_eq!(td.state_changes.len(), 8);

        // Verify cumulative time: @0=0, @+100=100, @+200=300, @+100=400, @+300=700
        assert_eq!(td.state_changes[0].time, 0); // WU is Idle
        assert_eq!(td.state_changes[3].time, 100); // WU is Waiting
        assert_eq!(td.state_changes[5].time, 300); // WB is Waiting
        assert_eq!(td.state_changes[6].time, 400); // DNS is Processing
        assert_eq!(td.state_changes[7].time, 700); // DNS is Idle

        // Messages
        assert_eq!(td.messages.len(), 2);
        assert_eq!(td.messages[0].from, "WU");
        assert_eq!(td.messages[0].to, "WB");
        assert_eq!(td.messages[0].from_time, 100);
        assert_eq!(td.messages[1].from, "WB");
        assert_eq!(td.messages[1].to, "DNS");
        assert_eq!(td.messages[1].from_time, 300);
        assert_eq!(td.messages[1].to_time, 350); // @+50

        // Constraints
        assert_eq!(td.constraints.len(), 1);
        assert_eq!(td.constraints[0].participant, "WU");
        assert_eq!(td.constraints[0].start_time, 200);
        assert_eq!(td.constraints[0].end_time, 350);
    }

    #[test]
    fn parse_unquoted_participant() {
        let src = "@startuml\nrobust Sig1 as S1\n@0\nS1 is Low\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 1);
        assert_eq!(td.participants[0].name, "Sig1");
        assert_eq!(td.participants[0].alias.as_deref(), Some("S1"));
    }

    #[test]
    fn parse_participant_no_alias() {
        let src = "@startuml\nrobust \"Signal\"\n@0\nSignal is High\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 1);
        assert_eq!(td.participants[0].name, "Signal");
        assert!(td.participants[0].alias.is_none());
    }

    #[test]
    fn parse_comments_skipped() {
        let src = "@startuml\n' this is a comment\nrobust \"A\" as A\n@0\n' another comment\nA is Idle\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.participants.len(), 1);
        assert_eq!(td.state_changes.len(), 1);
    }

    #[test]
    fn parse_left_arrow_message() {
        let src =
            "@startuml\nrobust \"A\" as A\nrobust \"B\" as B\n@100\nA <- B : reply\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.messages.len(), 1);
        assert_eq!(td.messages[0].from, "B");
        assert_eq!(td.messages[0].to, "A");
        assert_eq!(td.messages[0].label, "reply");
    }

    #[test]
    fn parse_constraint_absolute_endpoints() {
        let src =
            "@startuml\nconcise \"X\" as X\n@0\nX is Off\n@X\n@100 <-> @300 : {200 ms}\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.constraints.len(), 1);
        assert_eq!(td.constraints[0].start_time, 100);
        assert_eq!(td.constraints[0].end_time, 300);
    }

    #[test]
    fn parse_note_on_participant() {
        let src = "@startuml\nrobust \"A\" as A\n@0\nA is Idle\nnote on A : test note\n@enduml\n";
        let td = parse_timing_diagram(src).unwrap();
        assert_eq!(td.notes.len(), 1);
        assert_eq!(td.notes[0].target.as_deref(), Some("A"));
        assert_eq!(td.notes[0].text, "test note");
        assert_eq!(td.notes[0].position, "top");
    }
}
