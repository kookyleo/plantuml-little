use log::debug;

use crate::model::bpm::{BpmDiagram, BpmElement, BpmElementType, BpmEvent};
use crate::Result;

/// Parse a @startbpm diagram into a BpmDiagram model.
///
/// Java BPM syntax (from CommandDockedEvent, CommandNewBranch, etc.):
///   `:Label;`     — docked event (task)
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
}
