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
    fn test_parse_basic_chronology() {
        let src = "@startchronology\n[2020-01-01] Task A\n[2020-06-01] Task B\n@endchronology";
        let d = parse_chronology_diagram(src).unwrap();
        assert_eq!(d.events.len(), 2);
        assert_eq!(d.events[0].date, "2020-01-01");
        assert_eq!(d.events[0].label, "Task A");
    }
}
