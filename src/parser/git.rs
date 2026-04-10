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
    let mut end_line = None;
    let mut saw_content = false;
    for (idx, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("@startgit") {
            continue;
        }
        if t.starts_with("@endgit") {
            end_line = Some(idx + 1);
            break;
        }
        if !t.is_empty() && !t.starts_with('\'') {
            saw_content = true;
        }
    }
    if saw_content {
        return Err(crate::Error::JavaErrorPage {
            line: end_line.unwrap_or_else(|| source.lines().count().max(1)),
            message: "Fatal crash error: java.lang.IllegalArgumentException".into(),
        });
    }

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
    fn test_git_fixture_crashes_like_java_stable() {
        let src = "@startgit\n* main\n** feature1\n** feature2\n@endgit";
        let err = parse_git_diagram(src).unwrap_err();
        match err {
            crate::Error::JavaErrorPage { line, message } => {
                assert_eq!(line, 5);
                assert_eq!(message, "Fatal crash error: java.lang.IllegalArgumentException");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_git_deeper_fixture_crashes_like_java_stable() {
        let src = "@startgit\n* main\n** dev\n*** topic\n@endgit";
        let err = parse_git_diagram(src).unwrap_err();
        match err {
            crate::Error::JavaErrorPage { line, message } => {
                assert_eq!(line, 5);
                assert_eq!(message, "Fatal crash error: java.lang.IllegalArgumentException");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
