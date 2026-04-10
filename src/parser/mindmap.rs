use log::{debug, trace, warn};

use crate::model::mindmap::{MindmapDiagram, MindmapNode, MindmapNote};
use crate::Result;

/// Parse mode to track style blocks and other skippable regions.
#[derive(Debug)]
enum ParseMode {
    Normal,
    StyleBlock,
}

/// Parse a `@startmindmap` ... `@endmindmap` source into a `MindmapDiagram`.
pub fn parse_mindmap_diagram(source: &str) -> Result<MindmapDiagram> {
    let block = extract_mindmap_block(source);
    let body = block.as_deref().unwrap_or(source);

    debug!("parse_mindmap_diagram: body length = {}", body.len());

    let mut flat_nodes: Vec<MindmapNode> = Vec::new();
    let mut notes: Vec<MindmapNote> = Vec::new();
    let mut caption: Option<String> = None;
    let mut mode = ParseMode::Normal;
    let mut in_note_block = false;
    let mut note_block_position = String::new();
    let mut note_block_lines: Vec<String> = Vec::new();

    for (line_num, line) in body.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        // Handle style block
        match mode {
            ParseMode::StyleBlock => {
                if trimmed.to_lowercase().starts_with("</style>") {
                    debug!("line {line_num}: leaving <style> block");
                    mode = ParseMode::Normal;
                } else {
                    trace!("line {line_num}: skipping style content");
                }
                continue;
            }
            ParseMode::Normal => {}
        }

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Enter style block
        if trimmed.to_lowercase().starts_with("<style>") {
            debug!("line {line_num}: entering <style> block");
            mode = ParseMode::StyleBlock;
            continue;
        }

        // Skip comments (lines starting with ')
        if trimmed.starts_with('\'') {
            trace!("line {line_num}: skipping comment");
            continue;
        }

        // Parse caption
        if let Some(cap_text) = trimmed.strip_prefix("caption ") {
            caption = Some(cap_text.to_string());
            trace!("line {line_num}: captured caption '{cap_text}'");
            continue;
        }
        if trimmed == "caption" {
            trace!("line {line_num}: skipping empty caption");
            continue;
        }

        // Skip `title` lines
        if trimmed.starts_with("title ") || trimmed == "title" {
            trace!("line {line_num}: skipping title");
            continue;
        }

        // Skip `header`, `footer`, `legend` lines
        if trimmed.starts_with("header")
            || trimmed.starts_with("footer")
            || trimmed.starts_with("legend")
        {
            trace!("line {line_num}: skipping header/footer/legend");
            continue;
        }

        // Handle multi-line note accumulation
        if in_note_block {
            if trimmed == "end note" || trimmed == "endnote" {
                let text = note_block_lines.join("\n");
                debug!("line {line_num}: end note block, text={text:?}");
                notes.push(MindmapNote {
                    text,
                    position: note_block_position.clone(),
                });
                note_block_lines.clear();
                in_note_block = false;
            } else {
                note_block_lines.push(trimmed.to_string());
            }
            continue;
        }

        // Note parsing: `note left : text` or `note right`
        if let Some(note_result) = try_parse_mindmap_note(trimmed) {
            match note_result {
                MindmapNoteParseResult::SingleLine(note) => {
                    debug!("line {line_num}: single-line note");
                    notes.push(note);
                }
                MindmapNoteParseResult::MultiLineStart { position } => {
                    debug!("line {line_num}: start multi-line note");
                    in_note_block = true;
                    note_block_position = position;
                    note_block_lines.clear();
                }
            }
            continue;
        }

        // Parse `*`-prefixed node lines
        if trimmed.starts_with('*') {
            if let Some(node) = parse_node_line(trimmed, line_num) {
                flat_nodes.push(node);
            }
            continue;
        }

        // Skip unrecognized lines with a warning
        if !trimmed.is_empty() {
            warn!("line {line_num}: unrecognized mindmap line: {trimmed:?}");
        }
    }

    if flat_nodes.is_empty() {
        return Err(crate::Error::Parse {
            line: 1,
            column: Some(1),
            message: "no mindmap nodes found".to_string(),
        });
    }

    let root = build_tree(&flat_nodes)?;
    debug!(
        "parse_mindmap_diagram: root='{}', depth={}",
        root.text,
        root.depth()
    );

    Ok(MindmapDiagram {
        root,
        notes,
        caption,
    })
}

/// Extract the content between `@startmindmap` and `@endmindmap`.
fn extract_mindmap_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if inside {
            if trimmed.starts_with("@endmindmap") {
                break;
            }
            lines.push(line);
        } else if trimmed.starts_with("@startmindmap") {
            inside = true;
            continue;
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Parse a single `*`-prefixed line into a flat `MindmapNode`.
///
/// Examples:
/// - `* Root text` -> level=1, text="Root text"
/// - `** Child text` -> level=2, text="Child text"
/// - `*** Deep child` -> level=3, text="Deep child"
fn parse_node_line(trimmed: &str, line_num: usize) -> Option<MindmapNode> {
    // Count leading `*` characters
    let star_count = trimmed.chars().take_while(|&c| c == '*').count();
    if star_count == 0 {
        return None;
    }

    let rest = &trimmed[star_count..];

    // Handle optional color/style markers like `*[#color]` - strip them
    let text = if rest.starts_with('[') {
        // Skip bracketed section
        if let Some(end) = rest.find(']') {
            rest[end + 1..].trim()
        } else {
            rest.trim()
        }
    } else {
        rest.trim()
    };

    // Handle optional `:` prefix (alternate syntax `*: text`)
    let text = if let Some(stripped) = text.strip_prefix(':') {
        stripped.trim()
    } else {
        text
    };

    // Handle trailing `;` (alternate syntax)
    let text = text.trim_end_matches(';').trim();

    debug!("line {line_num}: node level={star_count}, text={text:?}");

    Some(MindmapNode::new(text, star_count))
}

/// Build a tree from a flat list of nodes based on their levels.
///
/// The first node must be level 1 (root). Subsequent nodes are attached
/// to the tree based on their level relative to their parent.
fn build_tree(flat_nodes: &[MindmapNode]) -> Result<MindmapNode> {
    if flat_nodes.is_empty() {
        return Err(crate::Error::Parse {
            line: 1,
            column: Some(1),
            message: "no nodes to build tree from".to_string(),
        });
    }

    let mut root = flat_nodes[0].clone();

    // Stack of (level, index-path-to-parent)
    // We'll use a simpler recursive approach: track a stack of mutable references
    // by rebuilding the path each time.
    let mut stack: Vec<(usize, Vec<usize>)> = vec![(root.level, vec![])];

    for node in &flat_nodes[1..] {
        // Pop stack until we find a parent (a node with level < current)
        while let Some(&(lvl, _)) = stack.last() {
            if lvl >= node.level {
                stack.pop();
            } else {
                break;
            }
        }

        if stack.is_empty() {
            warn!(
                "node at level {} has no parent, attaching to root: {:?}",
                node.level, node.text
            );
            let idx = root.children.len();
            root.children.push(node.clone());
            stack.push((node.level, vec![idx]));
        } else {
            let (_, ref parent_path) = stack.last().unwrap().clone();
            let parent_path = parent_path.clone();

            // Navigate to the parent node
            let parent = navigate_mut(&mut root, &parent_path);
            let idx = parent.children.len();
            parent.children.push(node.clone());

            let mut new_path = parent_path;
            new_path.push(idx);
            stack.push((node.level, new_path));
        }
    }

    Ok(root)
}

/// Navigate to a node in the tree by following a path of child indices.
fn navigate_mut<'a>(root: &'a mut MindmapNode, path: &[usize]) -> &'a mut MindmapNode {
    let mut current = root;
    for &idx in path {
        current = &mut current.children[idx];
    }
    current
}

// ---------------------------------------------------------------------------
// Note parsing
// ---------------------------------------------------------------------------

enum MindmapNoteParseResult {
    SingleLine(MindmapNote),
    MultiLineStart { position: String },
}

/// Parse a mindmap note line.
///
/// Supported forms:
///   `note left : text`    (single-line)
///   `note right`          (multi-line start)
fn try_parse_mindmap_note(line: &str) -> Option<MindmapNoteParseResult> {
    let trimmed = line.trim();
    if !trimmed.starts_with("note ") {
        return None;
    }

    let rest = trimmed[5..].trim();

    for pos in &["left", "right", "top", "bottom"] {
        if !rest.starts_with(pos) {
            continue;
        }
        let after_pos = rest[pos.len()..].trim();

        if let Some(after_colon) = after_pos.strip_prefix(':') {
            let text = after_colon
                .trim()
                .replace("\\n", "\n")
                .replace(crate::NEWLINE_CHAR, "\n");
            return Some(MindmapNoteParseResult::SingleLine(MindmapNote {
                text,
                position: pos.to_string(),
            }));
        }

        if after_pos.is_empty() {
            return Some(MindmapNoteParseResult::MultiLineStart {
                position: pos.to_string(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_mindmap() {
        let src = "@startmindmap\n* Root\n** Child1\n** Child2\n@endmindmap\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.text, "Root");
        assert_eq!(diagram.root.level, 1);
        assert_eq!(diagram.root.children.len(), 2);
        assert_eq!(diagram.root.children[0].text, "Child1");
        assert_eq!(diagram.root.children[1].text, "Child2");
    }

    #[test]
    fn parse_nested_mindmap() {
        let src = "@startmindmap\n* Root\n** A\n*** A1\n*** A2\n** B\n@endmindmap\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.children.len(), 2);
        let a = &diagram.root.children[0];
        assert_eq!(a.text, "A");
        assert_eq!(a.children.len(), 2);
        assert_eq!(a.children[0].text, "A1");
        assert_eq!(a.children[1].text, "A2");
        let b = &diagram.root.children[1];
        assert_eq!(b.text, "B");
        assert!(b.children.is_empty());
    }

    #[test]
    fn parse_with_style_block() {
        let src = "\
@startmindmap
<style>
node {
    Padding 0
}
</style>
* Root
** Child
@endmindmap
";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.text, "Root");
        assert_eq!(diagram.root.children.len(), 1);
    }

    #[test]
    fn parse_with_caption() {
        let src = "@startmindmap\n* Root\ncaption some caption\n@endmindmap\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.text, "Root");
    }

    #[test]
    fn parse_with_comments() {
        let src = "@startmindmap\n' comment line\n* Root\n** Child\n@endmindmap\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.text, "Root");
        assert_eq!(diagram.root.children.len(), 1);
    }

    #[test]
    fn parse_jaws12_fixture() {
        let src = "\
@startmindmap

<style>
node {
    Padding 0
    HorizontalAlignment center
    LineColor blue
    LineThickness 3.0
    BackgroundColor gold
    RoundCorner 40
}

</style>

* New style
** New features
** Template Files
*** They are \\ngreat and \\n should have them!

caption caption
@endmindmap
";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.text, "New style");
        assert_eq!(diagram.root.children.len(), 2);
        assert_eq!(diagram.root.children[0].text, "New features");
        assert_eq!(diagram.root.children[1].text, "Template Files");
        assert_eq!(diagram.root.children[1].children.len(), 1);
        assert_eq!(
            diagram.root.children[1].children[0].text,
            "They are \\ngreat and \\n should have them!"
        );
    }

    #[test]
    fn parse_empty_returns_error() {
        let src = "@startmindmap\n@endmindmap\n";
        let result = parse_mindmap_diagram(src);
        assert!(result.is_err());
    }

    #[test]
    fn parse_without_markers() {
        let src = "* Root\n** Child\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.text, "Root");
        assert_eq!(diagram.root.children.len(), 1);
    }

    #[test]
    fn parse_deep_nesting() {
        let src = "@startmindmap\n* L1\n** L2\n*** L3\n**** L4\n***** L5\n@endmindmap\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.depth(), 5);
        let l2 = &diagram.root.children[0];
        let l3 = &l2.children[0];
        let l4 = &l3.children[0];
        let l5 = &l4.children[0];
        assert_eq!(l5.text, "L5");
    }

    #[test]
    fn parse_sibling_return_after_deep() {
        let src = "\
@startmindmap
* Root
** A
*** A1
** B
@endmindmap
";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.children.len(), 2);
        assert_eq!(diagram.root.children[0].text, "A");
        assert_eq!(diagram.root.children[0].children.len(), 1);
        assert_eq!(diagram.root.children[1].text, "B");
        assert!(diagram.root.children[1].children.is_empty());
    }

    #[test]
    fn parse_node_with_color_marker() {
        let src = "@startmindmap\n*[#red] Root\n**[#blue] Child\n@endmindmap\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.root.text, "Root");
        assert_eq!(diagram.root.children[0].text, "Child");
    }

    #[test]
    fn extract_mindmap_block_works() {
        let src = "@startmindmap\n* root\n@endmindmap\n";
        let block = extract_mindmap_block(src).unwrap();
        assert_eq!(block, "* root");
    }

    #[test]
    fn extract_mindmap_block_none_on_empty() {
        let src = "no mindmap here";
        assert!(extract_mindmap_block(src).is_none());
    }

    #[test]
    fn parse_node_line_basic() {
        let node = parse_node_line("* Hello world", 1).unwrap();
        assert_eq!(node.level, 1);
        assert_eq!(node.text, "Hello world");
    }

    #[test]
    fn parse_node_line_level3() {
        let node = parse_node_line("*** Deep node text", 1).unwrap();
        assert_eq!(node.level, 3);
        assert_eq!(node.text, "Deep node text");
    }

    #[test]
    fn build_tree_single_root() {
        let nodes = vec![MindmapNode::new("Root", 1)];
        let tree = build_tree(&nodes).unwrap();
        assert_eq!(tree.text, "Root");
        assert!(tree.children.is_empty());
    }

    #[test]
    fn build_tree_complex() {
        let nodes = vec![
            MindmapNode::new("Root", 1),
            MindmapNode::new("A", 2),
            MindmapNode::new("A1", 3),
            MindmapNode::new("B", 2),
            MindmapNode::new("B1", 3),
            MindmapNode::new("B2", 3),
        ];
        let tree = build_tree(&nodes).unwrap();
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].children.len(), 1);
        assert_eq!(tree.children[1].children.len(), 2);
    }

    #[test]
    fn parse_note_left() {
        let src = "@startmindmap\n* Root\nnote left : a short note\n@endmindmap\n";
        let diagram = parse_mindmap_diagram(src).unwrap();
        assert_eq!(diagram.notes.len(), 1);
        assert_eq!(diagram.notes[0].position, "left");
        assert_eq!(diagram.notes[0].text, "a short note");
    }
}
