use crate::model::flow::{FlowDiagram, FlowDirection, FlowLink, FlowNode};
use crate::{Error, Result};

fn extract_flow_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if inside {
            if trimmed.starts_with("@endflow") {
                break;
            }
            lines.push(line);
        } else if trimmed.starts_with("@startflow") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn parse_direction_and_rest(line: &str) -> (FlowDirection, &str) {
    let trimmed = line.trim();
    if let Some(first) = trimmed.chars().next() {
        if let Some(direction) = FlowDirection::from_char(first) {
            let rest = &trimmed[first.len_utf8()..];
            if rest.starts_with(char::is_whitespace) {
                return (direction, rest.trim_start());
            }
        }
    }
    (FlowDirection::South, trimmed)
}

pub fn parse_flow_diagram(source: &str) -> Result<FlowDiagram> {
    let block = extract_flow_block(source).unwrap_or_else(|| source.to_string());
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    let mut last_id: Option<String> = None;

    for (line_idx, raw_line) in block.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('"') || trimmed.starts_with('\'') {
            continue;
        }

        let (direction, rest) = parse_direction_and_rest(trimmed);
        if let Some((id, label)) = parse_simple_line(rest) {
            nodes.push(FlowNode {
                id: id.to_string(),
                label: label.to_string(),
                placement: if nodes.is_empty() { None } else { Some(direction) },
            });
            if let Some(from) = last_id.take() {
                links.push(FlowLink {
                    from,
                    to: id.to_string(),
                    direction,
                });
            }
            last_id = Some(id.to_string());
            continue;
        }

        if let Some(id_dest) = parse_link_line(rest) {
            let from = last_id.clone().ok_or_else(|| Error::Parse {
                line: line_idx + 1,
                column: None,
                message: "flow link requires a previous node".into(),
            })?;
            links.push(FlowLink {
                from,
                to: id_dest.to_string(),
                direction,
            });
            continue;
        }

        return Err(Error::Parse {
            line: line_idx + 1,
            column: None,
            message: format!("unsupported flow command: {trimmed}"),
        });
    }

    if nodes.is_empty() {
        return Err(Error::Parse {
            line: 1,
            column: None,
            message: "empty flow diagram".into(),
        });
    }

    Ok(FlowDiagram { nodes, links })
}

fn parse_simple_line(line: &str) -> Option<(&str, &str)> {
    let (id, rest) = line.split_once(char::is_whitespace)?;
    let label = rest.trim();
    let label = label.strip_prefix('"')?.strip_suffix('"')?;
    if id.chars().all(|c| c == '_' || c.is_ascii_alphanumeric()) {
        Some((id, label))
    } else {
        None
    }
}

fn parse_link_line(line: &str) -> Option<&str> {
    if line.chars().all(|c| c == '_' || c.is_ascii_alphanumeric()) {
        Some(line)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flow_nodes_and_links() {
        let diagram = parse_flow_diagram("@startflow\none \"Start\"\ns two \"Second\"\nn one\n@endflow")
            .unwrap();
        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.links.len(), 2);
        assert_eq!(diagram.nodes[1].placement, Some(FlowDirection::South));
        assert_eq!(diagram.links[1].from, "two");
        assert_eq!(diagram.links[1].to, "one");
        assert_eq!(diagram.links[1].direction, FlowDirection::North);
    }
}
