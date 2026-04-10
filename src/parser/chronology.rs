use log::debug;

use crate::model::chronology::{ChronologyDiagram, ChronologyEvent};
use crate::Result;

fn extract_chronology_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endchronology") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startchronology") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn parse_chronology_diagram(source: &str) -> Result<ChronologyDiagram> {
    let mut inside = false;
    for (idx, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("@startchronology") {
            inside = true;
            continue;
        }
        if inside {
            if t.starts_with("@endchronology") {
                break;
            }
            if !t.is_empty() && !t.starts_with('\'') {
                return Err(crate::Error::JavaErrorPage {
                    line: idx + 1,
                    message: "Syntax Error? (Assumed diagram type: chronology)".into(),
                });
            }
        }
    }

    let block = extract_chronology_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_chronology_diagram: {} bytes", block.len());

    let mut events = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') {
            continue;
        }

        // Parse [date] label
        if t.starts_with('[') {
            if let Some(end_bracket) = t.find(']') {
                let date = t[1..end_bracket].to_string();
                let label = t[end_bracket + 1..].trim().to_string();
                events.push(ChronologyEvent { date, label });
            }
        }
    }

    Ok(ChronologyDiagram { events })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chronology_fixture_syntax_errors_like_java_stable() {
        let src = "@startchronology\n[2020-01-01] Task A\n[2020-06-01] Task B\n@endchronology";
        let err = parse_chronology_diagram(src).unwrap_err();
        match err {
            crate::Error::JavaErrorPage { line, message } => {
                assert_eq!(line, 2);
                assert_eq!(message, "Syntax Error? (Assumed diagram type: chronology)");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
