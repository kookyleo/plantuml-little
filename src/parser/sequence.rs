use log::{debug, warn};
use regex::Regex;

use crate::model::sequence::FragmentKind;
use crate::model::{
    Message, Participant, ParticipantKind, SeqArrowHead, SeqArrowStyle, SeqDirection, SeqEvent,
    SequenceDiagram,
};
use crate::Result;

/// Parse sequence diagram source text into SequenceDiagram IR
pub fn parse_sequence_diagram(source: &str) -> Result<SequenceDiagram> {
    let block = super::common::extract_block(source).unwrap_or_else(|| source.to_string());

    let mut declared_participants: Vec<Participant> = Vec::new();
    let mut auto_participants: Vec<Participant> = Vec::new();
    let mut events: Vec<SeqEvent> = Vec::new();
    let mut last_to_participant: Option<String> = None;
    let mut in_style_block = false;
    let mut in_skinparam_block = false;
    // Multiline note collection
    let mut in_note_block = false;
    let mut note_kind: Option<&str> = None; // "right", "left", "over"
    let mut note_participant: Option<String> = None;
    let mut note_participants: Vec<String> = Vec::new();
    let mut note_lines: Vec<String> = Vec::new();
    // Track fragment nesting depth so "end" emits FragmentEnd when inside fragments
    let mut fragment_depth: usize = 0;
    // Whether `!pragma teoz true` was encountered
    let mut teoz_mode = false;
    // Whether `hide footbox` was encountered
    let mut hide_footbox = false;

    let participant_re = Regex::new(
        r"(?i)^(participant|actor|boundary|control|entity|database|collections|queue)\s+(.+)$",
    )
    .unwrap();

    // Arrow regex: full PlantUML arrow syntax including half-arrows and decorators.
    let arrow_re = Regex::new(
        r"^(.+?)\s+([oxX]*(?:<<?)?(?:[/\\]{1,2})?(?:\[#[^\]]+\])?-+(?:\[#[^\]]+\])?-*(?:[/\\]{1,2})?(?:>?>?)?[oxX]*)\s+(.+?)(?:\s*:\s*(.*))?$",
    )
    .unwrap();
    let arrow_nospace_re = Regex::new(
        r"^([A-Za-z_]\w*)([oxX]*(?:<<?)?(?:[/\\]{1,2})?(?:\[#[^\]]+\])?-+(?:\[#[^\]]+\])?-*(?:[/\\]{1,2})?(?:>?>?)?[oxX]*)([A-Za-z_]\w*)(?:\s*:\s*(.*))?$",
    )
    .unwrap();
    // Boundary arrow from left: [-> or [<-> participant
    let boundary_left_re = Regex::new(
        r"^\[([oxX]*(?:<<?)?(?:[/\\]{1,2})?(?:\[#[^\]]+\])?-+(?:\[#[^\]]+\])?-*(?:[/\\]{1,2})?(?:>?>?)?[oxX]*)\s+(.+?)(?:\s*:\s*(.*))?$",
    )
    .unwrap();
    // Boundary arrow to right: participant ->]
    let boundary_right_re = Regex::new(
        r"^(.+?)\s+([oxX]*(?:<<?)?(?:[/\\]{1,2})?(?:\[#[^\]]+\])?-+(?:\[#[^\]]+\])?-*(?:[/\\]{1,2})?(?:>?>?)?[oxX]*\])\s*(?:\s*:\s*(.*))?$",
    )
    .unwrap();

    // Gate/found message: ?->X or X->?  (? is a gate/lost/found participant)
    let gate_left_re = Regex::new(
        r"^\?([ox]*(?:[/\\]{1,2})?-+(?:[/\\]{1,2})?(?:>?>?)?[ox]*)(.+?)(?:\s*:\s*(.*))?$",
    )
    .unwrap();
    let gate_right_re = Regex::new(
        r"^(.+?)([oxX]*(?:<<?)?(?:[/\\]{1,2})?-+(?:[/\\]{1,2})?(?:>?>?)?[oxX]*)\?\s*(?::\s*(.*))?$",
    )
    .unwrap();

    let divider_re = Regex::new(r"^==\s*(.*?)\s*==$").unwrap();
    let delay_re = Regex::new(r"^\|\|\|$|^\|\|(\d+)\|\|$").unwrap();
    // Delay with text: ...text... or just ...
    let delay_text_re = Regex::new(r"^\.\.\.(.*)?\.\.\.$|^\.\.\.$").unwrap();
    // Spacing: || N || (with space around number)
    let spacing_re = Regex::new(r"^\|\|\s*(\d+)\s*\|\|$").unwrap();
    // Ref over: ref over A, B : label
    let ref_re = Regex::new(r"(?i)^ref\s+over\s+(.+?)\s*:\s*(.+)$").unwrap();
    // Autonumber: autonumber or autonumber N
    let autonumber_re = Regex::new(r"(?i)^autonumber(?:\s+(\d+))?$").unwrap();

    for line in block.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip comments (lines starting with ')
        if trimmed.starts_with('\'') {
            continue;
        }

        // Handle <style>...</style> blocks
        if trimmed.to_lowercase().starts_with("<style>") {
            in_style_block = true;
            debug!("entering <style> block");
            continue;
        }
        if in_style_block {
            if trimmed.to_lowercase().starts_with("</style>") {
                in_style_block = false;
                debug!("leaving <style> block");
            }
            continue;
        }

        // Handle skinparam blocks
        if trimmed.to_lowercase().starts_with("skinparam") {
            if trimmed.contains('{') {
                in_skinparam_block = true;
                debug!("entering skinparam block");
            }
            // Single-line skinparam is also skipped
            continue;
        }
        if in_skinparam_block {
            if trimmed.contains('}') {
                in_skinparam_block = false;
                debug!("leaving skinparam block");
            }
            continue;
        }

        // Collect multiline note content
        if in_note_block {
            if trimmed.eq_ignore_ascii_case("end note")
                || trimmed.eq_ignore_ascii_case("endnote")
            {
                let text = note_lines.join("\n");
                let evt = match note_kind {
                    Some("right") => SeqEvent::NoteRight {
                        participant: note_participant.take().unwrap_or_default(),
                        text,
                    },
                    Some("left") => SeqEvent::NoteLeft {
                        participant: note_participant.take().unwrap_or_default(),
                        text,
                    },
                    _ => SeqEvent::NoteOver {
                        participants: std::mem::take(&mut note_participants),
                        text,
                    },
                };
                debug!("parsed multiline note event");
                events.push(evt);
                in_note_block = false;
                note_kind = None;
                note_lines.clear();
            } else {
                note_lines.push(trimmed.to_string());
            }
            continue;
        }

        // Skip title, legend, footer, header, caption, hide, show, !pragma
        {
            let lower = trimmed.to_lowercase();
            if lower.starts_with("!pragma") {
                // Capture `!pragma teoz true`
                let rest = lower["!pragma".len()..].trim();
                if rest == "teoz true" {
                    debug!("pragma teoz true enabled");
                    teoz_mode = true;
                } else {
                    debug!("skipping pragma: {trimmed}");
                }
                continue;
            }
            if lower.starts_with("title ")
                || lower == "title"
                || lower.starts_with("legend")
                || lower.starts_with("footer")
                || lower.starts_with("header")
                || lower.starts_with("caption")
                || (lower.starts_with("hide ") && !lower.contains("footbox"))
                || lower.starts_with("show ")
            {
                debug!("skipping directive: {trimmed}");
                continue;
            }
            if lower == "hide footbox" {
                hide_footbox = true;
                debug!("hide footbox enabled");
                continue;
            }
            if lower.starts_with("skin ") {
                debug!("skipping directive: {trimmed}");
                continue;
            }
        }

        // Parse divider: == text ==
        if let Some(caps) = divider_re.captures(trimmed) {
            let text = caps.get(1).map(|m| m.as_str().trim().to_string());
            let text = text.filter(|t| !t.is_empty());
            debug!("parsed divider: {text:?}");
            events.push(SeqEvent::Divider { text });
            continue;
        }

        // Parse spacing: || N || (must check before delay_re since ||| overlaps)
        if let Some(caps) = spacing_re.captures(trimmed) {
            let pixels: u32 = caps.get(1).unwrap().as_str().parse().unwrap_or(20);
            debug!("parsed spacing: {pixels} px");
            events.push(SeqEvent::Spacing { pixels });
            continue;
        }

        // Parse delay: ||| or ||N|| (legacy, N treated as spacing)
        if let Some(caps) = delay_re.captures(trimmed) {
            let text = caps.get(1).map(|m| m.as_str().to_string());
            debug!("parsed delay: {text:?}");
            events.push(SeqEvent::Delay { text });
            continue;
        }

        // Parse delay with text: ...text... or ...
        if let Some(caps) = delay_text_re.captures(trimmed) {
            let text = caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .filter(|t| !t.is_empty());
            debug!("parsed delay text: {text:?}");
            events.push(SeqEvent::Delay { text });
            continue;
        }

        // Parse autonumber
        if let Some(caps) = autonumber_re.captures(trimmed) {
            let start = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok());
            debug!("parsed autonumber: start={start:?}");
            events.push(SeqEvent::AutoNumber { start });
            continue;
        }

        // Parse ref over
        if let Some(caps) = ref_re.captures(trimmed) {
            let participants_str = caps.get(1).unwrap().as_str();
            let label = caps.get(2).unwrap().as_str().trim().to_string();
            let participants: Vec<String> = participants_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            debug!("parsed ref over {participants:?} : {label}");
            events.push(SeqEvent::Ref {
                participants,
                label,
            });
            continue;
        }

        // Parse activate/deactivate/destroy
        {
            let lower = trimmed.to_lowercase();
            if lower.starts_with("activate ") {
                let name = trimmed[9..].trim().to_string();
                debug!("parsed activate: {name}");
                ensure_participant(&mut declared_participants, &mut auto_participants, &name);
                events.push(SeqEvent::Activate(name));
                continue;
            }
            if lower.starts_with("deactivate ") {
                let name = trimmed[11..].trim().to_string();
                debug!("parsed deactivate: {name}");
                events.push(SeqEvent::Deactivate(name));
                continue;
            }
            if lower.starts_with("destroy ") {
                let name = trimmed[8..].trim().to_string();
                debug!("parsed destroy: {name}");
                events.push(SeqEvent::Destroy(name));
                continue;
            }
        }

        // Parse note right/left/over (single-line or start multiline block)
        {
            let lower = trimmed.to_lowercase();
            if lower.starts_with("note ") {
                match parse_note(trimmed, &last_to_participant) {
                    Some(evt) => {
                        debug!("parsed note event");
                        events.push(evt);
                        continue;
                    }
                    None => {
                        // Check if this starts a multiline note block
                        let rest = trimmed[5..].trim();
                        let rest_lower = rest.to_lowercase();
                        if rest_lower.starts_with("right") {
                            in_note_block = true;
                            note_kind = Some("right");
                            note_participant = last_to_participant.clone();
                            note_lines.clear();
                            debug!("starting multiline note right");
                            continue;
                        } else if rest_lower.starts_with("left") {
                            in_note_block = true;
                            note_kind = Some("left");
                            note_participant = last_to_participant.clone();
                            note_lines.clear();
                            debug!("starting multiline note left");
                            continue;
                        } else if rest_lower.starts_with("over") {
                            in_note_block = true;
                            note_kind = Some("over");
                            let after = rest[4..].trim();
                            note_participants = after
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            note_lines.clear();
                            debug!("starting multiline note over");
                            continue;
                        }
                    }
                }
            }
        }

        // Parse combined fragments and group/end/else
        {
            let lower = trimmed.to_lowercase();

            // "end" closes a fragment or legacy group
            if lower == "end" {
                if fragment_depth > 0 {
                    fragment_depth -= 1;
                    debug!("parsed fragment end (depth now {fragment_depth})");
                    events.push(SeqEvent::FragmentEnd);
                } else {
                    debug!("parsed group end");
                    events.push(SeqEvent::GroupEnd);
                }
                continue;
            }

            // "else" within a fragment
            if lower.starts_with("else") && fragment_depth > 0 {
                let rest = trimmed[4..].trim();
                let label = rest.to_string();
                debug!("parsed fragment separator: {label:?}");
                events.push(SeqEvent::FragmentSeparator { label });
                continue;
            }

            // Fragment start keywords: alt, loop, opt, par, break, critical
            if let Some((kind, rest_start)) = parse_fragment_keyword(&lower) {
                let label = trimmed[rest_start..].trim().to_string();
                fragment_depth += 1;
                debug!(
                    "parsed fragment start {kind:?} label={label:?} (depth now {fragment_depth})"
                );
                events.push(SeqEvent::FragmentStart { kind, label });
                continue;
            }

            // Legacy "group" keyword
            if lower.starts_with("group") {
                let rest = trimmed[5..].trim();
                let label = if rest.is_empty() {
                    None
                } else {
                    Some(rest.to_string())
                };
                // Track as fragment for proper "end" matching
                fragment_depth += 1;
                debug!("parsed group start: {label:?} (depth now {fragment_depth})");
                events.push(SeqEvent::FragmentStart {
                    kind: FragmentKind::Group,
                    label: label.unwrap_or_default(),
                });
                continue;
            }
        }

        // Parse message arrows: boundary arrows first, then regular.
        // Arrow check must come before participant declarations so that lines
        // like "Database -> Bob : ack" are parsed as messages, not as a
        // database participant declaration with rest "-> Bob : ack".

        // Teoz parallel message prefix: "& A -> B : msg" means this message
        // is parallel with the previous one. Strip the "&" prefix and treat
        // as a normal message (parallel layout is not yet fully implemented,
        // but this prevents "& foo" from being created as a participant).
        let trimmed_arrow = if trimmed.starts_with("& ") {
            trimmed[2..].trim()
        } else {
            trimmed
        };

        // Gate/found messages: ?->X (incoming) and X->? (outgoing)
        if let Some(caps) = gate_left_re.captures(trimmed_arrow) {
            let arrow = caps.get(1).unwrap().as_str();
            let right = caps.get(2).unwrap().as_str().trim();
            let text = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            if let Some(msg) = parse_arrow("[", &format!("[{arrow}"), right, &text) {
                debug!("parsed gate-left message: ?-> {} : {}", msg.to, msg.text);
                ensure_participant(&mut declared_participants, &mut auto_participants, &msg.to);
                last_to_participant = Some(msg.to.clone());
                events.push(SeqEvent::Message(msg));
                continue;
            }
        }
        if let Some(caps) = gate_right_re.captures(trimmed_arrow) {
            let left = caps.get(1).unwrap().as_str().trim();
            let arrow = caps.get(2).unwrap().as_str();
            let text = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            if let Some(msg) = parse_arrow(left, &format!("{arrow}]"), "]", &text) {
                debug!("parsed gate-right message: {} ->? : {}", msg.from, msg.text);
                ensure_participant(&mut declared_participants, &mut auto_participants, &msg.from);
                last_to_participant = Some(msg.from.clone());
                events.push(SeqEvent::Message(msg));
                continue;
            }
        }

        // Boundary arrow from left: [-> participant : text
        if let Some(caps) = boundary_left_re.captures(trimmed_arrow) {
            let arrow = caps.get(1).unwrap().as_str();
            let right = caps.get(2).unwrap().as_str().trim();
            let text = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            if let Some(msg) = parse_arrow("[", &format!("[{arrow}"), right, &text) {
                debug!("parsed boundary-left message: [-> {} : {}", msg.to, msg.text);
                ensure_participant(&mut declared_participants, &mut auto_participants, &msg.to);
                last_to_participant = Some(msg.to.clone());
                events.push(SeqEvent::Message(msg));
                continue;
            }
        }

        // Boundary arrow to right: participant ->] : text
        if let Some(caps) = boundary_right_re.captures(trimmed_arrow) {
            let left = caps.get(1).unwrap().as_str().trim();
            let arrow = caps.get(2).unwrap().as_str();
            let text = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            if let Some(msg) = parse_arrow(left, arrow, "]", &text) {
                debug!(
                    "parsed boundary-right message: {} ->] : {}",
                    msg.from, msg.text
                );
                ensure_participant(&mut declared_participants, &mut auto_participants, &msg.from);
                last_to_participant = Some(msg.from.clone());
                events.push(SeqEvent::Message(msg));
                continue;
            }
        }

        // Regular arrows (spaced and no-space variants)
        let arrow_caps = arrow_re
            .captures(trimmed_arrow)
            .or_else(|| arrow_nospace_re.captures(trimmed_arrow));
        if let Some(caps) = arrow_caps {
            let left = caps.get(1).unwrap().as_str().trim();
            let arrow = caps.get(2).unwrap().as_str();
            let mut right = caps.get(3).unwrap().as_str().trim().to_string();
            let text = caps
                .get(4)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            // Handle inline activation/deactivation: ++ or -- after participant
            let mut inline_activate = false;
            let mut inline_deactivate = false;
            if right.ends_with("++") {
                right = right[..right.len() - 2].trim().to_string();
                inline_activate = true;
            } else if right.ends_with("--") {
                right = right[..right.len() - 2].trim().to_string();
                inline_deactivate = true;
            }
            // Strip #color suffix from participant (message color annotation)
            // e.g. "Testing #red" -> "Testing", color goes on message
            if let Some(hash_pos) = right.rfind(" #") {
                right = right[..hash_pos].trim().to_string();
            } else if right.starts_with('#') {
                // Entire right is a color - shouldn't happen, skip
            }

            if let Some(msg) = parse_arrow(left, arrow, &right, &text) {
                debug!("parsed message: {} -> {} : {}", msg.from, msg.to, msg.text);

                // Auto-create participants
                ensure_participant(
                    &mut declared_participants,
                    &mut auto_participants,
                    &msg.from,
                );
                ensure_participant(
                    &mut declared_participants,
                    &mut auto_participants,
                    &msg.to,
                );

                last_to_participant = Some(msg.to.clone());
                let target = msg.to.clone();
                events.push(SeqEvent::Message(msg));
                if inline_activate {
                    events.push(SeqEvent::Activate(target));
                } else if inline_deactivate {
                    events.push(SeqEvent::Deactivate(target));
                }
                continue;
            }
        }

        // Parse participant declarations
        if let Some(caps) = participant_re.captures(trimmed) {
            let kind_str = caps.get(1).unwrap().as_str().to_lowercase();
            let rest = caps.get(2).unwrap().as_str().trim();

            let kind = match kind_str.as_str() {
                "participant" => ParticipantKind::Default,
                "actor" => ParticipantKind::Actor,
                "boundary" => ParticipantKind::Boundary,
                "control" => ParticipantKind::Control,
                "entity" => ParticipantKind::Entity,
                "database" => ParticipantKind::Database,
                "collections" => ParticipantKind::Collections,
                "queue" => ParticipantKind::Queue,
                _ => ParticipantKind::Default,
            };

            let (name, display_name, color) = parse_participant_details(rest);
            debug!(
                "parsed participant declaration: name={name}, display={display_name:?}, color={color:?}, kind={kind:?}"
            );

            // Remove from auto_participants if it was auto-created
            auto_participants.retain(|p| p.name != name);

            // Avoid duplicate declarations
            if !declared_participants.iter().any(|p| p.name == name) {
                declared_participants.push(Participant {
                    name,
                    display_name,
                    kind,
                    color,
                });
            }
            continue;
        }

        warn!("unrecognized sequence diagram line: {trimmed}");
    }

    // Merge participants: declared first, then auto-created
    let mut participants = declared_participants;
    participants.append(&mut auto_participants);

    Ok(SequenceDiagram {
        participants,
        events,
        teoz_mode,
        hide_footbox,
    })
}

/// Parse combined fragment keyword, return fragment kind and label start position
fn parse_fragment_keyword(lower: &str) -> Option<(FragmentKind, usize)> {
    // Order matters: check longer keywords first to avoid prefix conflicts
    if lower.starts_with("critical")
        && (lower.len() == 8 || lower.as_bytes()[8].is_ascii_whitespace())
    {
        Some((FragmentKind::Critical, 8))
    } else if lower.starts_with("break")
        && (lower.len() == 5 || lower.as_bytes()[5].is_ascii_whitespace())
    {
        Some((FragmentKind::Break, 5))
    } else if lower.starts_with("loop")
        && (lower.len() == 4 || lower.as_bytes()[4].is_ascii_whitespace())
    {
        Some((FragmentKind::Loop, 4))
    } else if lower.starts_with("alt")
        && (lower.len() == 3 || lower.as_bytes()[3].is_ascii_whitespace())
    {
        Some((FragmentKind::Alt, 3))
    } else if lower.starts_with("opt")
        && (lower.len() == 3 || lower.as_bytes()[3].is_ascii_whitespace())
    {
        Some((FragmentKind::Opt, 3))
    } else if lower.starts_with("par")
        && (lower.len() == 3 || lower.as_bytes()[3].is_ascii_whitespace())
    {
        Some((FragmentKind::Par, 3))
    } else {
        None
    }
}

/// Ensure participant exists in either the declared or auto-created list
fn ensure_participant(
    declared: &mut [Participant],
    auto_created: &mut Vec<Participant>,
    name: &str,
) {
    if declared.iter().any(|p| p.name == name) {
        return;
    }
    if auto_created.iter().any(|p| p.name == name) {
        return;
    }
    debug!("auto-creating participant: {name}");
    auto_created.push(Participant {
        name: name.to_string(),
        display_name: None,
        kind: ParticipantKind::Default,
        color: None,
    });
}

/// Strip `[[url text]]` markup from a display name, returning just the display text.
/// E.g. `"[[http://example.com Line 1 Line 2]]"` → `"Line 1 Line 2"`
fn strip_url_markup(s: &str) -> String {
    let mut result = s.to_string();
    while let Some(start) = result.find("[[") {
        if let Some(end) = result[start..].find("]]") {
            let inner = &result[start + 2..start + end];
            // Extract text after URL (URL is the first token)
            let display = if let Some(space_pos) = inner.find(' ') {
                inner[space_pos + 1..].to_string()
            } else {
                String::new()
            };
            result.replace_range(start..start + end + 2, &display);
        } else {
            break;
        }
    }
    result
}

/// Parse participant declaration details: name, display name, and color
fn parse_participant_details(rest: &str) -> (String, Option<String>, Option<String>) {
    // Patterns:
    //   "Display Name" as Name #color
    //   "Display Name" as Name
    //   Name as "Display Name" #color
    //   Name #color
    //   Name

    let mut remaining = rest.trim();
    let mut name: String;
    let mut display_name: Option<String> = None;

    if remaining.starts_with('"') {
        // Quoted display name first: "Display Name" as Name ...
        if let Some(end_quote) = remaining[1..].find('"') {
            let quoted = remaining[1..=end_quote].to_string();
            remaining = remaining[end_quote + 2..].trim();
            // Strip [[url text]] patterns — extract just the display text
            let cleaned = strip_url_markup(&quoted);
            display_name = Some(cleaned);

            // Expect "as Name" next
            let lower = remaining.to_lowercase();
            if lower.starts_with("as ") {
                remaining = remaining[3..].trim();
            }
            // Extract name (next token)
            let (n, rest_after) = take_token(remaining);
            name = n;
            remaining = rest_after;
        } else {
            // No closing quote, treat whole thing as name
            name = remaining.to_string();
            remaining = "";
        }
    } else {
        // Name first
        let (n, rest_after) = take_token(remaining);
        name = n;
        remaining = rest_after.trim();

        // Check for "as" — "Name as Alias" means alias is the canonical name,
        // and Name becomes the display name
        let lower = remaining.to_lowercase();
        if lower.starts_with("as ") {
            remaining = remaining[3..].trim();
            let original_name = name.clone();
            if remaining.starts_with('"') {
                // as "Display Name" ... (alias is quoted - unusual but handle it)
                if let Some(end_quote) = remaining[1..].find('"') {
                    name = remaining[1..=end_quote].to_string();
                    display_name = Some(original_name);
                    remaining = remaining[end_quote + 2..].trim();
                }
            } else {
                let (alias, rest_after2) = take_token(remaining);
                name = alias;
                display_name = Some(original_name);
                remaining = rest_after2;
            }
        }
    }

    // Check for color at the end
    let remaining = remaining.trim();
    let color = if remaining.starts_with('#') {
        Some(remaining.to_string())
    } else {
        None
    };

    (name, display_name, color)
}

/// Extract the first whitespace-delimited token from the beginning of the string
fn take_token(s: &str) -> (String, &str) {
    let s = s.trim();
    if s.is_empty() {
        return (String::new(), "");
    }
    match s.find(char::is_whitespace) {
        Some(pos) => (s[..pos].to_string(), &s[pos..]),
        None => (s.to_string(), ""),
    }
}

/// Extract `[#color]` from an arrow string, returning (color, cleaned_arrow).
/// E.g. `"[#blue]->"` → `(Some("blue"), "->")`, `"-[#green]>"` → `(Some("green"), "->")`
fn extract_arrow_color(arrow: &str) -> (Option<String>, String) {
    if let Some(start) = arrow.find("[#") {
        if let Some(end) = arrow[start..].find(']') {
            let color = arrow[start + 2..start + end].to_string();
            let cleaned = format!("{}{}", &arrow[..start], &arrow[start + end + 1..]);
            return (Some(color), cleaned);
        }
    }
    (None, arrow.to_string())
}

/// Parse arrow syntax and return a Message.
/// Handles full PlantUML syntax: heads (>, >>), half-arrows (/, \),
/// decorators (o, x), boundary ([, ]), and shaft (-, --).
fn parse_arrow(left: &str, arrow: &str, right: &str, text: &str) -> Option<Message> {
    let (color, clean_arrow) = extract_arrow_color(arrow);

    // Strip outer decorators (o, x, X)
    let stripped =
        clean_arrow.trim_start_matches(|c: char| c == 'o' || c == 'x' || c == 'X');
    let stripped = stripped.trim_end_matches(|c: char| c == 'o' || c == 'x' || c == 'X');

    // Check for boundary markers
    let has_left_boundary = stripped.starts_with('[');
    let has_right_boundary = stripped.ends_with(']');
    let stripped = stripped.trim_start_matches('[').trim_end_matches(']');

    // Check for arrow heads / half-arrows on left and right
    let has_left_arrow =
        stripped.starts_with('<') || stripped.starts_with('/') || stripped.starts_with('\\');
    let has_open_left = stripped.starts_with("<<");
    let has_right_arrow =
        stripped.ends_with('>') || stripped.ends_with('/') || stripped.ends_with('\\');
    let has_open_right = stripped.ends_with(">>");

    // Must have at least one arrowhead, half-arrow, or boundary marker
    if !has_left_arrow && !has_right_arrow && !has_left_boundary && !has_right_boundary {
        return None;
    }

    // Direction: left-pointing heads mean right-to-left
    let direction = if stripped.starts_with('<')
        || stripped.starts_with('/')
        || stripped.starts_with('\\')
    {
        SeqDirection::RightToLeft
    } else {
        SeqDirection::LeftToRight
    };

    let arrow_head = if has_open_left || has_open_right {
        SeqArrowHead::Open
    } else {
        SeqArrowHead::Filled
    };

    // Shaft style: -- is dashed, - is solid
    let shaft = stripped
        .trim_start_matches(|c: char| c == '<' || c == '/' || c == '\\')
        .trim_end_matches(|c: char| c == '>' || c == '/' || c == '\\');
    let arrow_style = if shaft.contains("--") {
        SeqArrowStyle::Dashed
    } else {
        SeqArrowStyle::Solid
    };

    let (from, to) = if has_left_boundary {
        (left.to_string(), right.to_string())
    } else if has_right_boundary {
        (left.to_string(), "]".to_string())
    } else {
        match direction {
            SeqDirection::LeftToRight => (left.to_string(), right.to_string()),
            SeqDirection::RightToLeft => (right.to_string(), left.to_string()),
        }
    };

    Some(Message {
        from,
        to,
        text: text.to_string(),
        arrow_style,
        arrow_head,
        direction,
        color,
    })
}

/// Parse a single-line note (with `:` inline text).
/// Returns None if the note has no inline text (multiline note handled by caller).
fn parse_note(line: &str, last_to: &Option<String>) -> Option<SeqEvent> {
    let rest = line.trim().strip_prefix("note ")?.trim_start();
    let lower = rest.to_lowercase();

    if lower.starts_with("right") {
        let after = rest[5..].trim();
        if let Some(text) = after.strip_prefix(':') {
            let text = text.trim().to_string();
            let participant = last_to.clone().unwrap_or_default();
            Some(SeqEvent::NoteRight { participant, text })
        } else {
            // No inline text — will be handled as multiline note
            None
        }
    } else if lower.starts_with("left") {
        let after = rest[4..].trim();
        if let Some(text) = after.strip_prefix(':') {
            let text = text.trim().to_string();
            let participant = last_to.clone().unwrap_or_default();
            Some(SeqEvent::NoteLeft { participant, text })
        } else {
            None
        }
    } else if lower.starts_with("over") {
        let after = rest[4..].trim();
        // note over A,B : text  or  note over A : text
        if let Some(colon_pos) = after.find(':') {
            let participants_str = after[..colon_pos].trim();
            let text = after[colon_pos + 1..].trim().to_string();
            let participants: Vec<String> = participants_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            Some(SeqEvent::NoteOver { participants, text })
        } else {
            // No inline text — multiline
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1. Parse simplest message
    #[test]
    fn parse_simplest_message() {
        let src = "@startuml\nalice->bob: hello\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.participants[0].name, "alice");
        assert_eq!(diagram.participants[1].name, "bob");
        assert_eq!(diagram.events.len(), 1);

        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "alice");
            assert_eq!(msg.to, "bob");
            assert_eq!(msg.text, "hello");
            assert_eq!(msg.arrow_style, SeqArrowStyle::Solid);
            assert_eq!(msg.arrow_head, SeqArrowHead::Filled);
            assert_eq!(msg.direction, SeqDirection::LeftToRight);
        } else {
            panic!("expected Message event");
        }
    }

    /// 2. Parse dashed arrow
    #[test]
    fn parse_dashed_arrow() {
        let src = "@startuml\nA --> B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "A");
            assert_eq!(msg.to, "B");
            assert_eq!(msg.text, "msg");
            assert_eq!(msg.arrow_style, SeqArrowStyle::Dashed);
            assert_eq!(msg.direction, SeqDirection::LeftToRight);
        } else {
            panic!("expected Message event");
        }
    }

    /// 3. Parse left arrow
    #[test]
    fn parse_left_arrow() {
        let src = "@startuml\nA <- B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "B");
            assert_eq!(msg.to, "A");
            assert_eq!(msg.direction, SeqDirection::RightToLeft);
            assert_eq!(msg.arrow_style, SeqArrowStyle::Solid);
        } else {
            panic!("expected Message event");
        }
    }

    /// 4. Parse self-message
    #[test]
    fn parse_self_message() {
        let src = "@startuml\nBob->Bob: hello\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.participants.len(), 1);
        assert_eq!(diagram.participants[0].name, "Bob");

        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "Bob");
            assert_eq!(msg.to, "Bob");
        } else {
            panic!("expected Message event");
        }
    }

    /// 5. Parse activate/deactivate
    #[test]
    fn parse_activate_deactivate() {
        let src = "@startuml\nA -> B : a\nactivate B\nB --> A : b\ndeactivate B\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(matches!(&diagram.events[0], SeqEvent::Message(_)));
        assert!(matches!(&diagram.events[1], SeqEvent::Activate(ref n) if n == "B"));
        assert!(matches!(&diagram.events[2], SeqEvent::Message(_)));
        assert!(matches!(&diagram.events[3], SeqEvent::Deactivate(ref n) if n == "B"));
    }

    /// 6. Parse destroy
    #[test]
    fn parse_destroy() {
        let src = "@startuml\nBob->Bob: hello\ndestroy Bob\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(matches!(&diagram.events[1], SeqEvent::Destroy(ref n) if n == "Bob"));
    }

    /// 7. Parse participant declaration with color
    #[test]
    fn parse_participant_with_color() {
        let src = "@startuml\nparticipant Alice #FFFFFF\nAlice -> Bob : hi\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.participants[0].name, "Alice");
        assert_eq!(diagram.participants[0].color.as_deref(), Some("#FFFFFF"));
        assert_eq!(diagram.participants[0].kind, ParticipantKind::Default);
    }

    /// 8. Parse actor declaration
    #[test]
    fn parse_actor_declaration() {
        let src = "@startuml\nactor Bob\nBob -> Alice : hi\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.participants[0].name, "Bob");
        assert_eq!(diagram.participants[0].kind, ParticipantKind::Actor);
    }

    /// 9. Parse group/end (now emits FragmentStart/FragmentEnd)
    #[test]
    fn parse_group_end() {
        let src = "@startuml\ngroup My Group\na -> b : msg\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(
            matches!(&diagram.events[0], SeqEvent::FragmentStart { kind, label } if *kind == FragmentKind::Group && label == "My Group")
        );
        assert!(matches!(&diagram.events[1], SeqEvent::Message(_)));
        assert!(matches!(&diagram.events[2], SeqEvent::FragmentEnd));
    }

    /// 10. Parse note right/left
    #[test]
    fn parse_note_right_left() {
        let src = "@startuml\nTest --> Test: Text\nnote right: comment\nnote left: other\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(matches!(&diagram.events[0], SeqEvent::Message(_)));
        if let SeqEvent::NoteRight {
            participant, text, ..
        } = &diagram.events[1]
        {
            assert_eq!(participant, "Test");
            assert_eq!(text, "comment");
        } else {
            panic!("expected NoteRight");
        }
        if let SeqEvent::NoteLeft {
            participant, text, ..
        } = &diagram.events[2]
        {
            assert_eq!(participant, "Test");
            assert_eq!(text, "other");
        } else {
            panic!("expected NoteLeft");
        }
    }

    /// 11. Parse divider
    #[test]
    fn parse_divider() {
        let src = "@startuml\n== My Divider ==\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        assert!(
            matches!(&diagram.events[0], SeqEvent::Divider { text } if text.as_deref() == Some("My Divider"))
        );
    }

    /// 12. Auto-create participants from messages
    #[test]
    fn auto_create_participants() {
        let src = "@startuml\nAlice -> Bob : hi\nBob -> Charlie : hey\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.participants.len(), 3);
        assert_eq!(diagram.participants[0].name, "Alice");
        assert_eq!(diagram.participants[1].name, "Bob");
        assert_eq!(diagram.participants[2].name, "Charlie");
        // All auto-created should be Default kind
        for p in &diagram.participants {
            assert_eq!(p.kind, ParticipantKind::Default);
        }
    }

    /// 13. Skip style blocks and skinparam
    #[test]
    fn skip_style_and_skinparam() {
        let src = r#"@startuml
title title
legend legend
footer footer
header header
caption caption
<style>
    document {
       BackGroundColor orange
    }
</style>
skinparam {
   Maxmessagesize 200
}
Sally --> Bob
@enduml"#;
        let diagram = parse_sequence_diagram(src).unwrap();

        // Only one message, style/skinparam/title etc. are all skipped
        assert_eq!(diagram.events.len(), 1);
        assert!(matches!(&diagram.events[0], SeqEvent::Message(_)));
        assert_eq!(diagram.participants.len(), 2);
    }

    /// 14. Parse fixture test_0.puml
    #[test]
    fn parse_fixture_test_0() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/sequence/test_0.puml"
        ))
        .unwrap();
        let diagram = parse_sequence_diagram(&src).unwrap();

        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.participants[0].name, "alice");
        assert_eq!(diagram.participants[1].name, "bob");
        assert_eq!(diagram.events.len(), 1);

        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "alice");
            assert_eq!(msg.to, "bob");
            assert_eq!(msg.text, "this is a test");
        } else {
            panic!("expected Message event");
        }
    }

    /// 15. Parse fixture a0001.puml
    #[test]
    fn parse_fixture_a0001() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/sequence/a0001.puml"
        ))
        .unwrap();
        let diagram = parse_sequence_diagram(&src).unwrap();

        // Participants: Bob and Alice (auto-created)
        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.participants[0].name, "Bob");
        assert_eq!(diagram.participants[1].name, "Alice");

        // Events: message, activate, message, destroy, message, message
        // Bob->Bob: hello1
        // activate Bob
        // Bob->Bob: hello2
        // destroy Bob
        // Bob->Bob: this is an\nexample of long\nmessage
        // Bob->Alice: And this\nis an other on\nvery long too
        assert!(
            matches!(&diagram.events[0], SeqEvent::Message(m) if m.from == "Bob" && m.to == "Bob")
        );
        assert!(matches!(&diagram.events[1], SeqEvent::Activate(ref n) if n == "Bob"));
        assert!(
            matches!(&diagram.events[2], SeqEvent::Message(m) if m.from == "Bob" && m.to == "Bob")
        );
        assert!(matches!(&diagram.events[3], SeqEvent::Destroy(ref n) if n == "Bob"));
        assert!(
            matches!(&diagram.events[4], SeqEvent::Message(m) if m.text.contains("an\\nexample"))
        );
        assert!(
            matches!(&diagram.events[5], SeqEvent::Message(m) if m.from == "Bob" && m.to == "Alice")
        );
    }

    /// 16. Parse participant with 'as' alias
    #[test]
    fn parse_participant_with_alias() {
        let src = "@startuml\nparticipant \"Long Name\" as LN\nLN -> B : hi\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.participants[0].name, "LN");
        assert_eq!(
            diagram.participants[0].display_name.as_deref(),
            Some("Long Name")
        );
    }

    /// 17. Parse open arrowhead
    #[test]
    fn parse_open_arrowhead() {
        let src = "@startuml\nA ->> B : open\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.arrow_head, SeqArrowHead::Open);
            assert_eq!(msg.arrow_style, SeqArrowStyle::Solid);
        } else {
            panic!("expected Message event");
        }
    }

    /// 18. Parse delay (|||) and spacing (||45||)
    #[test]
    fn parse_delay_and_spacing() {
        let src = "@startuml\n|||\n||45||\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 2);
        assert!(matches!(&diagram.events[0], SeqEvent::Delay { text } if text.is_none()));
        // ||45|| is explicit spacing, not delay
        assert!(matches!(&diagram.events[1], SeqEvent::Spacing { pixels } if *pixels == 45));
    }

    /// 19. Declared participants come first, then auto-created
    #[test]
    fn participant_ordering() {
        let src = "@startuml\nAlice -> Bob : hi\nparticipant Bob\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        // Bob was declared, Alice was auto-created
        assert_eq!(diagram.participants[0].name, "Bob");
        assert_eq!(diagram.participants[1].name, "Alice");
    }

    /// 20. Parse dashed left arrow
    #[test]
    fn parse_dashed_left_arrow() {
        let src = "@startuml\nA <-- B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "B");
            assert_eq!(msg.to, "A");
            assert_eq!(msg.direction, SeqDirection::RightToLeft);
            assert_eq!(msg.arrow_style, SeqArrowStyle::Dashed);
        } else {
            panic!("expected Message event");
        }
    }

    /// 21. Parse fixture sequencelayout_0003 (notes after arrows)
    #[test]
    fn parse_fixture_sequencelayout_0003() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/sequence/sequencelayout_0003.puml"
        ))
        .unwrap();
        let diagram = parse_sequence_diagram(&src).unwrap();

        // Should have 1 participant (Test), 8 events (4 messages + 4 notes)
        assert_eq!(diagram.participants.len(), 1);
        assert_eq!(diagram.participants[0].name, "Test");
        assert_eq!(diagram.events.len(), 8);
    }

    /// 22. Parse arrow without colon text (message is empty)
    #[test]
    fn parse_arrow_no_text() {
        let src = "@startuml\nSally --> Bob\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "Sally");
            assert_eq!(msg.to, "Bob");
            assert!(msg.text.is_empty());
        } else {
            panic!("expected Message event");
        }
    }

    /// 23. Parse alt/else/end
    #[test]
    fn parse_alt_else_end() {
        let src = "@startuml\nA -> B : req\nalt success\nB -> A : ok\nelse failure\nB -> A : err\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(
            matches!(&diagram.events[1], SeqEvent::FragmentStart { kind, label } if *kind == FragmentKind::Alt && label == "success")
        );
        assert!(
            matches!(&diagram.events[3], SeqEvent::FragmentSeparator { label } if label == "failure")
        );
        assert!(matches!(&diagram.events[5], SeqEvent::FragmentEnd));
    }

    /// 24. Parse loop
    #[test]
    fn parse_loop() {
        let src = "@startuml\nloop 1000 times\nA -> B : data\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(
            matches!(&diagram.events[0], SeqEvent::FragmentStart { kind, label } if *kind == FragmentKind::Loop && label == "1000 times")
        );
        assert!(matches!(&diagram.events[2], SeqEvent::FragmentEnd));
    }

    /// 25. Parse opt
    #[test]
    fn parse_opt() {
        let src = "@startuml\nopt extra processing\nA -> B : do\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(
            matches!(&diagram.events[0], SeqEvent::FragmentStart { kind, label } if *kind == FragmentKind::Opt && label == "extra processing")
        );
    }

    /// 26. Parse par with else
    #[test]
    fn parse_par_else() {
        let src = "@startuml\npar thread 1\nA -> B : t1\nelse thread 2\nA -> C : t2\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(
            matches!(&diagram.events[0], SeqEvent::FragmentStart { kind, label } if *kind == FragmentKind::Par && label == "thread 1")
        );
        assert!(
            matches!(&diagram.events[2], SeqEvent::FragmentSeparator { label } if label == "thread 2")
        );
        assert!(matches!(&diagram.events[4], SeqEvent::FragmentEnd));
    }

    /// 27. Parse break
    #[test]
    fn parse_break() {
        let src = "@startuml\nbreak\nA -> B : err\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(
            matches!(&diagram.events[0], SeqEvent::FragmentStart { kind, .. } if *kind == FragmentKind::Break)
        );
    }

    /// 28. Parse critical
    #[test]
    fn parse_critical() {
        let src = "@startuml\ncritical\nA -> B : write\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(
            matches!(&diagram.events[0], SeqEvent::FragmentStart { kind, .. } if *kind == FragmentKind::Critical)
        );
    }

    /// 29. Parse ref over
    #[test]
    fn parse_ref_over() {
        let src = "@startuml\nref over A, B : init phase\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        if let SeqEvent::Ref {
            participants,
            label,
        } = &diagram.events[0]
        {
            assert_eq!(participants, &["A", "B"]);
            assert_eq!(label, "init phase");
        } else {
            panic!("expected Ref event");
        }
    }

    /// 30. Parse delay with text
    #[test]
    fn parse_delay_with_text() {
        let src = "@startuml\n...waiting...\n...\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 2);
        assert!(
            matches!(&diagram.events[0], SeqEvent::Delay { text } if text.as_deref() == Some("waiting"))
        );
        assert!(matches!(&diagram.events[1], SeqEvent::Delay { text } if text.is_none()));
    }

    /// 31. Parse spacing
    #[test]
    fn parse_spacing() {
        let src = "@startuml\n|| 50 ||\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        assert!(matches!(&diagram.events[0], SeqEvent::Spacing { pixels } if *pixels == 50));
    }

    /// 32. Parse autonumber
    #[test]
    fn parse_autonumber() {
        let src = "@startuml\nautonumber\nA -> B : hello\nautonumber 10\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert!(matches!(&diagram.events[0], SeqEvent::AutoNumber { start } if start.is_none()));
        assert!(matches!(&diagram.events[2], SeqEvent::AutoNumber { start } if *start == Some(10)));
    }

    /// 33. Parse colored arrow [#blue]->
    #[test]
    fn parse_colored_arrow_prefix() {
        let src = "@startuml\na[#blue]->b: hello\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "a");
            assert_eq!(msg.to, "b");
            assert_eq!(msg.text, "hello");
            assert_eq!(msg.arrow_style, SeqArrowStyle::Solid);
            assert_eq!(msg.arrow_head, SeqArrowHead::Filled);
            assert_eq!(msg.direction, SeqDirection::LeftToRight);
            assert_eq!(msg.color.as_deref(), Some("blue"));
        } else {
            panic!("expected Message event");
        }
    }

    /// 34. Parse colored arrow -[#green]>
    #[test]
    fn parse_colored_arrow_infix() {
        let src = "@startuml\na-[#green]>b: world\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "a");
            assert_eq!(msg.to, "b");
            assert_eq!(msg.text, "world");
            assert_eq!(msg.arrow_style, SeqArrowStyle::Solid);
            assert_eq!(msg.arrow_head, SeqArrowHead::Filled);
            assert_eq!(msg.direction, SeqDirection::LeftToRight);
            assert_eq!(msg.color.as_deref(), Some("green"));
        } else {
            panic!("expected Message event");
        }
    }

    /// 35. Parse colored dashed arrow --[#red]>>
    #[test]
    fn parse_colored_dashed_open_arrow() {
        let src = "@startuml\nA --[#red]>> B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "A");
            assert_eq!(msg.to, "B");
            assert_eq!(msg.text, "msg");
            assert_eq!(msg.arrow_style, SeqArrowStyle::Dashed);
            assert_eq!(msg.arrow_head, SeqArrowHead::Open);
            assert_eq!(msg.direction, SeqDirection::LeftToRight);
            assert_eq!(msg.color.as_deref(), Some("red"));
        } else {
            panic!("expected Message event");
        }
    }

    /// 36. Parse colored left arrow <[#FF0000]--
    #[test]
    fn parse_colored_left_arrow() {
        let src = "@startuml\nA <[#FF0000]-- B : back\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        assert_eq!(diagram.events.len(), 1);
        if let SeqEvent::Message(msg) = &diagram.events[0] {
            assert_eq!(msg.from, "B");
            assert_eq!(msg.to, "A");
            assert_eq!(msg.text, "back");
            assert_eq!(msg.arrow_style, SeqArrowStyle::Dashed);
            assert_eq!(msg.direction, SeqDirection::RightToLeft);
            assert_eq!(msg.color.as_deref(), Some("FF0000"));
        } else {
            panic!("expected Message event");
        }
    }

    /// 37. Parse nested fragments (was #33)
    #[test]
    fn parse_nested_fragments() {
        let src = "@startuml\nalt case1\nloop retry\nalt inner\nA -> B : x\nend\nend\nelse case2\nA -> B : y\nend\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();

        // alt case1 -> loop retry -> alt inner -> msg -> end(inner) -> end(loop) -> else case2 -> msg -> end(outer alt)
        assert!(
            matches!(&diagram.events[0], SeqEvent::FragmentStart { kind, .. } if *kind == FragmentKind::Alt)
        );
        assert!(
            matches!(&diagram.events[1], SeqEvent::FragmentStart { kind, .. } if *kind == FragmentKind::Loop)
        );
        assert!(
            matches!(&diagram.events[2], SeqEvent::FragmentStart { kind, .. } if *kind == FragmentKind::Alt)
        );
        assert!(matches!(&diagram.events[4], SeqEvent::FragmentEnd)); // end inner alt
        assert!(matches!(&diagram.events[5], SeqEvent::FragmentEnd)); // end loop
        assert!(matches!(
            &diagram.events[6],
            SeqEvent::FragmentSeparator { .. }
        )); // else case2
        assert!(matches!(&diagram.events[8], SeqEvent::FragmentEnd)); // end outer alt
    }

    #[test]
    fn parse_pragma_teoz_true() {
        let src = "@startuml\n!pragma teoz true\nA -> B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();
        assert!(diagram.teoz_mode, "teoz_mode should be true when pragma teoz true is set");
    }

    #[test]
    fn parse_no_pragma_teoz_defaults_false() {
        let src = "@startuml\nA -> B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();
        assert!(diagram.teoz_mode == false, "teoz_mode should default to false");
    }

    #[test]
    fn parse_other_pragma_ignored() {
        let src = "@startuml\n!pragma graphviz_dot jdot\nA -> B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();
        assert!(diagram.teoz_mode == false, "other pragmas should not enable teoz_mode");
    }

    #[test]
    fn hide_footbox_parsed() {
        let src = "@startuml\nhide footbox\nA -> B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();
        assert!(diagram.hide_footbox, "hide footbox should be true");
    }

    #[test]
    fn hide_footbox_default_false() {
        let src = "@startuml\nA -> B : msg\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();
        assert!(diagram.hide_footbox == false, "hide footbox defaults to false");
    }

    #[test]
    fn parallel_ampersand_prefix_not_create_participant() {
        // "& foo -> foo : test" should NOT create a participant named "& foo".
        // The "&" is a teoz parallel-message prefix — strip it.
        let src = "@startuml\n!pragma teoz true\nparticipant foo\nfoo -> foo : first\n& foo -> foo : parallel\n@enduml";
        let diagram = parse_sequence_diagram(src).unwrap();
        assert_eq!(diagram.participants.len(), 1, "only 'foo', not '& foo'");
        assert_eq!(diagram.participants[0].name, "foo");
        assert_eq!(diagram.events.iter().filter(|e| matches!(e, SeqEvent::Message(_))).count(), 2);
    }
}
