use log::debug;

use crate::model::board::{BoardDiagram, BoardTask};
use crate::Result;

fn extract_board_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endboard") || t.starts_with("@enduml") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startboard") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn parse_board_diagram(source: &str) -> Result<BoardDiagram> {
    let has_outer_uml = source
        .lines()
        .any(|line| line.trim().starts_with("@startuml"));
    if has_outer_uml {
        let mut inside_board = false;
        for (idx, line) in source.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("@startboard") {
                inside_board = true;
                continue;
            }
            if inside_board && !t.is_empty() && !t.starts_with('\'') {
                let message = format!(
                    "You should send a mail to plantuml@gmail.com or post to https://plantuml.com/qa with this log (V{ver}) java.lang.IndexOutOfBoundsException: Index -1 out of bounds for length 0 (Assumed diagram type: board)",
                    ver = crate::render::svg::PLANTUML_VERSION
                );
                return Err(crate::Error::JavaErrorPage {
                    line: idx + 1,
                    message,
                });
            }
        }
    }

    let block = extract_board_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_board_diagram: {} bytes", block.len());

    let mut tasks = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') {
            continue;
        }

        // Count '+' prefix for nesting level
        let plus_count = t.chars().take_while(|&c| c == '+').count();
        if plus_count > 0 {
            let label = t[plus_count..].trim().to_string();
            tasks.push(BoardTask {
                label,
                level: plus_count,
                children: Vec::new(),
            });
        }
    }

    // Build tree structure from flat list
    let tree = build_tree(&tasks);
    Ok(BoardDiagram { tasks: tree })
}

fn build_tree(flat: &[BoardTask]) -> Vec<BoardTask> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < flat.len() {
        let task = &flat[i];
        if task.level == 1 {
            let mut node = BoardTask {
                label: task.label.clone(),
                level: 1,
                children: Vec::new(),
            };
            i += 1;
            // Collect children at level 2+
            while i < flat.len() && flat[i].level > 1 {
                node.children.push(BoardTask {
                    label: flat[i].label.clone(),
                    level: flat[i].level,
                    children: Vec::new(),
                });
                i += 1;
            }
            result.push(node);
        } else {
            i += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_board() {
        let src = "@startboard\n+ Task 1\n++ Sub 1.1\n+ Task 2\n@endboard";
        let d = parse_board_diagram(src).unwrap();
        assert_eq!(d.tasks.len(), 2);
        assert_eq!(d.tasks[0].label, "Task 1");
        assert_eq!(d.tasks[0].children.len(), 1);
        assert_eq!(d.tasks[0].children[0].label, "Sub 1.1");
    }
}
