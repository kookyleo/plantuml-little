use log::debug;

use crate::model::git::{GitDiagram, GitNode};
use crate::Result;

fn extract_git_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endgit") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startgit") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn parse_git_diagram(source: &str) -> Result<GitDiagram> {
    let block = extract_git_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_git_diagram: {} bytes", block.len());

    let mut nodes = Vec::new();

    for (index, line) in block.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') {
            continue;
        }

        // Count leading asterisks for depth
        let depth = t.chars().take_while(|&c| c == '*').count();
        if depth == 0 {
            continue;
        }

        let label = t[depth..].trim().to_string();
        if label.is_empty() {
            continue;
        }

        debug!("git node: depth={}, label={}", depth, label);
        nodes.push(GitNode {
            depth,
            label,
            index,
        });
    }

    Ok(GitDiagram { nodes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let src = "@startgit\n* main\n** feature1\n** feature2\n@endgit";
        let d = parse_git_diagram(src).unwrap();
        assert_eq!(d.nodes.len(), 3);
        assert_eq!(d.nodes[0].depth, 1);
        assert_eq!(d.nodes[0].label, "main");
        assert_eq!(d.nodes[1].depth, 2);
        assert_eq!(d.nodes[1].label, "feature1");
        assert_eq!(d.nodes[2].depth, 2);
        assert_eq!(d.nodes[2].label, "feature2");
    }

    #[test]
    fn test_parse_deeper() {
        let src = "@startgit\n* main\n** dev\n*** topic\n@endgit";
        let d = parse_git_diagram(src).unwrap();
        assert_eq!(d.nodes.len(), 3);
        assert_eq!(d.nodes[0].depth, 1);
        assert_eq!(d.nodes[1].depth, 2);
        assert_eq!(d.nodes[2].depth, 3);
    }
}
