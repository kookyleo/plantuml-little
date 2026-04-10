use std::sync::OnceLock;

use log::debug;
use regex::Regex;

use crate::model::bpm::{BpmDiagram, BpmElement, BpmElementType, BpmEvent};
use crate::Result;

/// Parse a @startbpm diagram into a BpmDiagram model.
///
/// Java BPM syntax (from CommandDockedEvent, CommandNewBranch, etc.):
///   `:Label;`     — docked event (task)
///   `ID:<+>`      — merge point (colon optional in Java)
///   `goto ID`     — jump to merge point
///   `resume ID`   — continue from merge point on a new row
///   `new branch`  — start a new branch
///   `else`        — else branch
///   `end branch`  — end branch
pub fn parse_bpm_diagram(source: &str) -> Result<BpmDiagram> {
    let block = extract_bpm_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_bpm_diagram: {} bytes", block.len());

    let mut events = Vec::new();
    // Branch counter for generating unique IDs (mirrors Java BpmBranch.uid = events.size())
    let mut branch_stack: Vec<BpmBranch> = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') {
            continue;
        }

        // Docked event: :Label;
        if t.starts_with(':') && t.ends_with(';') {
            let label = &t[1..t.len() - 1];
            let element = BpmElement {
                id: None,
                element_type: BpmElementType::DockedEvent,
                label: Some(label.to_string()),
                connectors: Vec::new(),
            };
            events.push(BpmEvent::Add(element));
            continue;
        }

        // Merge point: ID:<+> or ID<+>
        if let Some(id) = parse_merge_id(t) {
            let element = BpmElement {
                id: Some(id),
                element_type: BpmElementType::Merge,
                label: None,
                connectors: Vec::new(),
            };
            events.push(BpmEvent::Add(element));
            continue;
        }

        // goto ID
        if let Some(id) = parse_keyword_id(t, "goto") {
            events.push(BpmEvent::Goto(id));
            continue;
        }

        // resume ID
        if let Some(id) = parse_keyword_id(t, "resume") {
            events.push(BpmEvent::Resume(id));
            continue;
        }

        // new branch
        if t == "new branch" {
            let uid = events.len();
            let branch = BpmBranch::new(uid);
            let entry_element = BpmElement {
                id: Some(branch.entry_id()),
                element_type: BpmElementType::Merge,
                label: None,
                connectors: Vec::new(),
            };
            events.push(BpmEvent::Add(entry_element));
            branch_stack.push(branch);
            continue;
        }

        // else
        if t == "else" {
            if let Some(branch) = branch_stack.last_mut() {
                branch.counter += 1;
                if branch.counter == 2 {
                    // First else: add exit element, then resume at entry
                    let else_element = BpmElement {
                        id: Some(branch.exit_id()),
                        element_type: BpmElementType::Merge,
                        label: None,
                        connectors: Vec::new(),
                    };
                    events.push(BpmEvent::Add(else_element));
                    events.push(BpmEvent::Resume(branch.entry_id()));
                } else {
                    // Subsequent else: goto end, then resume at entry
                    events.push(BpmEvent::Goto(branch.exit_id()));
                    events.push(BpmEvent::Resume(branch.entry_id()));
                }
            }
            continue;
        }

        // end branch
        if t == "end branch" {
            if let Some(branch) = branch_stack.pop() {
                events.push(BpmEvent::Goto(branch.exit_id()));
            }
            continue;
        }
    }

    Ok(BpmDiagram { events })
}

fn parse_keyword_id(line: &str, keyword: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    if parts.next()? != keyword {
        return None;
    }
    let id = parts.next()?;
    if parts.next().is_some() || !is_valid_bpm_id(id) {
        return None;
    }
    Some(id.to_string())
}

fn parse_merge_id(line: &str) -> Option<String> {
    static MERGE_RE: OnceLock<Regex> = OnceLock::new();
    let re = MERGE_RE
        .get_or_init(|| Regex::new(r"^([\p{L}\p{N}_.@]+):?<\+>$").expect("valid BPM merge regex"));
    re.captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn is_valid_bpm_id(id: &str) -> bool {
    static ID_RE: OnceLock<Regex> = OnceLock::new();
    let re = ID_RE.get_or_init(|| Regex::new(r"^[\p{L}\p{N}_.@]+$").expect("valid BPM id regex"));
    re.is_match(id)
}

/// Helper for branch ID generation, mirroring Java BpmBranch.
struct BpmBranch {
    uid: usize,
    counter: usize,
}

impl BpmBranch {
    fn new(uid: usize) -> Self {
        BpmBranch { uid, counter: 1 }
    }

    fn entry_id(&self) -> String {
        format!("$branchA{}", self.uid)
    }

    fn exit_id(&self) -> String {
        format!("$branchB{}", self.uid)
    }
}

fn extract_bpm_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endbpm") || t.starts_with("@enduml") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startbpm") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_bpm() {
        let src = "@startbpm\n:Task A;\nnew branch\n:Task B;\nelse\n:Task C;\nend branch\n:Task D;\n@endbpm";
        let d = parse_bpm_diagram(src).unwrap();
        // Events: Task A, branch entry(MERGE), Task B, exit(MERGE), resume(entry), Task C, goto(exit), Task D
        assert!(d.events.len() >= 6);
    }

    #[test]
    fn test_parse_simple_bpm() {
        let src = "@startbpm\n:Hello;\n@endbpm";
        let d = parse_bpm_diagram(src).unwrap();
        assert_eq!(d.events.len(), 1);
        match &d.events[0] {
            BpmEvent::Add(e) => {
                assert_eq!(e.element_type, BpmElementType::DockedEvent);
                assert_eq!(e.label.as_deref(), Some("Hello"));
            }
            _ => panic!("expected Add event"),
        }
    }

    #[test]
    fn test_parse_merge_goto_resume() {
        let src =
            "@startbpm\njoin1:<+>\n:Task B;\ngoto join1\n:Task C;\nresume join1\n:Task D;\n@endbpm";
        let d = parse_bpm_diagram(src).unwrap();
        assert_eq!(d.events.len(), 6);
        match &d.events[0] {
            BpmEvent::Add(e) => {
                assert_eq!(e.element_type, BpmElementType::Merge);
                assert_eq!(e.id.as_deref(), Some("join1"));
            }
            _ => panic!("expected merge Add event"),
        }
        assert!(matches!(&d.events[2], BpmEvent::Goto(id) if id == "join1"));
        assert!(matches!(&d.events[4], BpmEvent::Resume(id) if id == "join1"));
    }

    #[test]
    fn test_parse_merge_without_colon_and_extended_id_chars() {
        let src = "@startbpm\njoin_1.@x<+>\ngoto join_1.@x\nresume join_1.@x\n@endbpm";
        let d = parse_bpm_diagram(src).unwrap();
        assert_eq!(d.events.len(), 3);
        match &d.events[0] {
            BpmEvent::Add(e) => {
                assert_eq!(e.element_type, BpmElementType::Merge);
                assert_eq!(e.id.as_deref(), Some("join_1.@x"));
            }
            _ => panic!("expected merge Add event"),
        }
        assert!(matches!(&d.events[1], BpmEvent::Goto(id) if id == "join_1.@x"));
        assert!(matches!(&d.events[2], BpmEvent::Resume(id) if id == "join_1.@x"));
    }
}
