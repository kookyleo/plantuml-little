//! Mindmap diagram layout engine.
//!
//! Converts a `MindmapDiagram` into a fully positioned `MindmapLayout` ready
//! for SVG rendering. The algorithm positions the root node on the left and
//! spreads children horizontally to the right in a tree layout.

use std::collections::HashMap;

use crate::font_metrics;
use crate::model::mindmap::{MindmapDiagram, MindmapNode, MindmapNote};
use crate::model::richtext::plain_text;
use crate::parser::creole::parse_creole;
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned mindmap layout ready for rendering.
#[derive(Debug)]
pub struct MindmapLayout {
    pub nodes: Vec<MindmapNodeLayout>,
    pub edges: Vec<MindmapEdgeLayout>,
    pub notes: Vec<MindmapNoteLayout>,
    pub width: f64,
    pub height: f64,
}

/// A positioned mindmap node.
#[derive(Debug, Clone)]
pub struct MindmapNodeLayout {
    /// Display text (may contain `\n` for multiline).
    pub text: String,
    /// Top-left x coordinate.
    pub x: f64,
    /// Top-left y coordinate.
    pub y: f64,
    /// Box width.
    pub width: f64,
    /// Box height.
    pub height: f64,
    /// Tree depth level (1 = root).
    pub level: usize,
    /// Text lines (split on `\n`).
    pub lines: Vec<String>,
}

/// A connection line between parent and child nodes.
#[derive(Debug, Clone)]
pub struct MindmapEdgeLayout {
    /// Parent node right-center x.
    pub from_x: f64,
    /// Parent node right-center y.
    pub from_y: f64,
    /// Child node left-center x.
    pub to_x: f64,
    /// Child node left-center y.
    pub to_y: f64,
}

/// A positioned note annotation attached to the root node.
#[derive(Debug, Clone)]
pub struct MindmapNoteLayout {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub connector: Option<(f64, f64, f64, f64)>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Font size for mindmap nodes.
const FONT_SIZE: f64 = 14.0;
/// Line height in pixels.
const LINE_HEIGHT: f64 = 16.0;
/// Horizontal padding inside nodes.
const H_PADDING: f64 = 8.0;
/// Vertical padding inside nodes.
const V_PADDING: f64 = 4.0;
/// Horizontal indent between tree levels.
const LEVEL_INDENT: f64 = 200.0;
/// Vertical spacing between sibling nodes.
const SIBLING_SPACING: f64 = 20.0;
/// Minimum node width.
const MIN_NODE_WIDTH: f64 = 40.0;
/// Minimum node height.
const MIN_NODE_HEIGHT: f64 = 24.0;
/// Canvas margin around the diagram.
const MARGIN: f64 = 20.0;
/// Gap between a node and an attached note.
const NOTE_GAP: f64 = 16.0;
/// Minimum note width.
const MIN_NOTE_WIDTH: f64 = 60.0;
/// Minimum note height.
const MIN_NOTE_HEIGHT: f64 = 28.0;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

/// Split text on `\n` escape sequences and return individual lines.
fn split_text_lines(text: &str) -> Vec<String> {
    text.split("\\n").map(|s| s.trim().to_string()).collect()
}

/// Estimate the rendered size of a node based on its text.
fn estimate_node_size(text: &str, is_root: bool) -> (f64, f64, Vec<String>) {
    let lines = split_text_lines(text);
    let max_line_width = lines
        .iter()
        .map(|l| font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);

    let base_width = max_line_width + 2.0 * H_PADDING;
    let base_height = lines.len() as f64 * LINE_HEIGHT + 2.0 * V_PADDING;

    let width = if is_root {
        base_width.max(MIN_NODE_WIDTH) + 16.0 // root is slightly larger
    } else {
        base_width.max(MIN_NODE_WIDTH)
    };

    let height = if is_root {
        base_height.max(MIN_NODE_HEIGHT) + 8.0
    } else {
        base_height.max(MIN_NODE_HEIGHT)
    };

    (width, height, lines)
}

fn plain_text_lines(text: &str) -> Vec<String> {
    let plain = plain_text(&parse_creole(text));
    let normalized = plain.replace("\\n", "\n");
    let lines: Vec<String> = normalized
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn estimate_note_size(text: &str) -> (f64, f64) {
    let lines = plain_text_lines(text);
    let max_line_width = lines
        .iter()
        .map(|line| font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    let width = (max_line_width + 2.0 * H_PADDING).max(MIN_NOTE_WIDTH);
    let height = (lines.len().max(1) as f64 * LINE_HEIGHT + 2.0 * V_PADDING).max(MIN_NOTE_HEIGHT);
    (width, height)
}

// ---------------------------------------------------------------------------
// Subtree size calculation
// ---------------------------------------------------------------------------

/// Information about a subtree's vertical extent.
#[derive(Debug, Clone)]
struct SubtreeInfo {
    /// Total height of this subtree (including spacing).
    total_height: f64,
    /// Width of the node itself.
    node_width: f64,
    /// Height of the node itself.
    node_height: f64,
    /// Text lines for the node.
    lines: Vec<String>,
    /// Children subtree info.
    children: Vec<SubtreeInfo>,
}

/// Recursively compute the subtree dimensions.
fn compute_subtree_info(node: &MindmapNode) -> SubtreeInfo {
    let is_root = node.level == 1;
    let (node_width, node_height, lines) = estimate_node_size(&node.text, is_root);

    if node.children.is_empty() {
        return SubtreeInfo {
            total_height: node_height,
            node_width,
            node_height,
            lines,
            children: Vec::new(),
        };
    }

    let child_infos: Vec<SubtreeInfo> = node.children.iter().map(compute_subtree_info).collect();

    let children_total_height: f64 = child_infos.iter().map(|c| c.total_height).sum::<f64>()
        + (child_infos.len() as f64 - 1.0).max(0.0) * SIBLING_SPACING;

    let total_height = children_total_height.max(node_height);

    SubtreeInfo {
        total_height,
        node_width,
        node_height,
        lines,
        children: child_infos,
    }
}

// ---------------------------------------------------------------------------
// Positioning
// ---------------------------------------------------------------------------

/// Recursively position nodes and collect edges.
fn position_subtree(
    node: &MindmapNode,
    info: &SubtreeInfo,
    x: f64,
    y_start: f64,
    nodes_out: &mut Vec<MindmapNodeLayout>,
    edges_out: &mut Vec<MindmapEdgeLayout>,
) {
    // Center this node vertically within its subtree span
    let node_y = y_start + (info.total_height - info.node_height) / 2.0;

    let node_center_y = node_y + info.node_height / 2.0;
    let node_right_x = x + info.node_width;

    nodes_out.push(MindmapNodeLayout {
        text: node.text.clone(),
        x,
        y: node_y,
        width: info.node_width,
        height: info.node_height,
        level: node.level,
        lines: info.lines.clone(),
    });

    if node.children.is_empty() {
        return;
    }

    // Position children
    let child_x = x + LEVEL_INDENT;
    let mut child_y = y_start;

    // If the children total height is less than the node height, center them
    let children_total: f64 = info.children.iter().map(|c| c.total_height).sum::<f64>()
        + (info.children.len() as f64 - 1.0).max(0.0) * SIBLING_SPACING;
    if children_total < info.total_height {
        child_y = y_start + (info.total_height - children_total) / 2.0;
    }

    for (i, (child_node, child_info)) in node.children.iter().zip(&info.children).enumerate() {
        // Compute child center y
        let child_center_y = child_y + child_info.total_height / 2.0;

        // Edge from parent right-center to child left-center
        edges_out.push(MindmapEdgeLayout {
            from_x: node_right_x,
            from_y: node_center_y,
            to_x: child_x,
            to_y: child_center_y,
        });

        position_subtree(
            child_node, child_info, child_x, child_y, nodes_out, edges_out,
        );

        child_y += child_info.total_height;
        if i < node.children.len() - 1 {
            child_y += SIBLING_SPACING;
        }
    }
}

/// Compute the maximum x extent of all nodes.
fn max_right_extent(nodes: &[MindmapNodeLayout]) -> f64 {
    nodes.iter().map(|n| n.x + n.width).fold(0.0_f64, f64::max)
}

/// Compute the maximum y extent of all nodes.
fn max_bottom_extent(nodes: &[MindmapNodeLayout]) -> f64 {
    nodes.iter().map(|n| n.y + n.height).fold(0.0_f64, f64::max)
}

fn layout_notes(notes: &[MindmapNote], root: &MindmapNodeLayout) -> Vec<MindmapNoteLayout> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut result = Vec::new();
    let root_center_x = root.x + root.width / 2.0;
    let root_center_y = root.y + root.height / 2.0;

    for note in notes {
        let (width, height) = estimate_note_size(&note.text);
        let stack_index = {
            let count = counts.entry(note.position.as_str()).or_insert(0);
            let current = *count as f64;
            *count += 1;
            current
        };

        let (x, y, connector) = match note.position.as_str() {
            "left" => {
                let x = root.x - NOTE_GAP - width;
                let y = root.y + stack_index * (height + NOTE_GAP);
                let connector = Some((root.x, root_center_y, x + width, y + height / 2.0));
                (x, y, connector)
            }
            "top" => {
                let x = root_center_x - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                let y = root.y - NOTE_GAP - height;
                let connector = Some((root_center_x, root.y, x + width / 2.0, y + height));
                (x, y, connector)
            }
            "bottom" => {
                let x = root_center_x - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                let y = root.y + root.height + NOTE_GAP;
                let connector = Some((root_center_x, root.y + root.height, x + width / 2.0, y));
                (x, y, connector)
            }
            _ => {
                let x = root.x + root.width + NOTE_GAP;
                let y = root.y + stack_index * (height + NOTE_GAP);
                let connector = Some((root.x + root.width, root_center_y, x, y + height / 2.0));
                (x, y, connector)
            }
        };

        result.push(MindmapNoteLayout {
            text: note.text.clone(),
            x,
            y,
            width,
            height,
            connector,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Lay out a mindmap diagram into positioned nodes and edges.
pub fn layout_mindmap(diagram: &MindmapDiagram) -> Result<MindmapLayout> {
    let info = compute_subtree_info(&diagram.root);

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let start_x = MARGIN;
    let start_y = MARGIN;

    position_subtree(
        &diagram.root,
        &info,
        start_x,
        start_y,
        &mut nodes,
        &mut edges,
    );

    let root = nodes
        .iter()
        .find(|node| node.level == 1)
        .cloned()
        .unwrap_or_else(|| nodes[0].clone());
    let mut notes = layout_notes(&diagram.notes, &root);

    let mut min_x = nodes.iter().map(|n| n.x).fold(f64::INFINITY, f64::min);
    let mut min_y = nodes.iter().map(|n| n.y).fold(f64::INFINITY, f64::min);
    let mut max_x = max_right_extent(&nodes);
    let mut max_y = max_bottom_extent(&nodes);
    for note in &notes {
        min_x = min_x.min(note.x);
        min_y = min_y.min(note.y);
        max_x = max_x.max(note.x + note.width);
        max_y = max_y.max(note.y + note.height);
    }

    let shift_x = if min_x < MARGIN { MARGIN - min_x } else { 0.0 };
    let shift_y = if min_y < MARGIN { MARGIN - min_y } else { 0.0 };

    if shift_x > 0.0 || shift_y > 0.0 {
        for node in &mut nodes {
            node.x += shift_x;
            node.y += shift_y;
        }
        for edge in &mut edges {
            edge.from_x += shift_x;
            edge.to_x += shift_x;
            edge.from_y += shift_y;
            edge.to_y += shift_y;
        }
        for note in &mut notes {
            note.x += shift_x;
            note.y += shift_y;
            if let Some((x1, y1, x2, y2)) = note.connector.as_mut() {
                *x1 += shift_x;
                *x2 += shift_x;
                *y1 += shift_y;
                *y2 += shift_y;
            }
        }
        max_x += shift_x;
        max_y += shift_y;
    }

    let width = max_x + MARGIN;
    let height = max_y + MARGIN;

    log::debug!(
        "layout_mindmap: {} nodes, {} edges, canvas {}x{}",
        nodes.len(),
        edges.len(),
        width,
        height
    );

    Ok(MindmapLayout {
        nodes,
        edges,
        notes,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mindmap::{MindmapDiagram, MindmapNode, MindmapNote};

    fn simple_diagram() -> MindmapDiagram {
        let mut root = MindmapNode::new("Root", 1);
        root.children.push(MindmapNode::new("Child1", 2));
        root.children.push(MindmapNode::new("Child2", 2));
        MindmapDiagram {
            root,
            notes: vec![],
        }
    }

    fn deep_diagram() -> MindmapDiagram {
        let mut root = MindmapNode::new("Root", 1);
        let mut child = MindmapNode::new("A", 2);
        let mut grandchild = MindmapNode::new("A1", 3);
        grandchild.children.push(MindmapNode::new("A1a", 4));
        child.children.push(grandchild);
        root.children.push(child);
        root.children.push(MindmapNode::new("B", 2));
        MindmapDiagram {
            root,
            notes: vec![],
        }
    }

    #[test]
    fn layout_simple_produces_correct_node_count() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.nodes.len(), 3);
    }

    #[test]
    fn layout_simple_produces_correct_edge_count() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.edges.len(), 2);
    }

    #[test]
    fn layout_root_is_leftmost() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        let root = &layout.nodes[0];
        for node in &layout.nodes[1..] {
            assert!(
                node.x > root.x,
                "child x ({}) should be > root x ({})",
                node.x,
                root.x
            );
        }
    }

    #[test]
    fn layout_children_at_same_x() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        let child1_x = layout.nodes[1].x;
        let child2_x = layout.nodes[2].x;
        assert!(
            (child1_x - child2_x).abs() < 0.001,
            "siblings should have same x: {} vs {}",
            child1_x,
            child2_x
        );
    }

    #[test]
    fn layout_children_vertically_ordered() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        assert!(
            layout.nodes[1].y < layout.nodes[2].y,
            "first child should be above second"
        );
    }

    #[test]
    fn layout_canvas_positive_dimensions() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn layout_deep_correct_node_count() {
        let diagram = deep_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.nodes.len(), 5);
    }

    #[test]
    fn layout_deep_correct_edge_count() {
        let diagram = deep_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.edges.len(), 4);
    }

    #[test]
    fn layout_node_levels_correct() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.nodes[0].level, 1);
        assert_eq!(layout.nodes[1].level, 2);
        assert_eq!(layout.nodes[2].level, 2);
    }

    #[test]
    fn layout_single_node() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("Alone", 1),
            notes: vec![],
        };
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.edges.len(), 0);
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn layout_nodes_have_positive_dimensions() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        for node in &layout.nodes {
            assert!(node.width > 0.0, "node width should be positive");
            assert!(node.height > 0.0, "node height should be positive");
        }
    }

    #[test]
    fn layout_edges_connect_parent_to_child() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram).unwrap();
        // Each edge from_x should be root's right side, to_x should be child's left
        let root = &layout.nodes[0];
        for edge in &layout.edges {
            assert!(
                (edge.from_x - (root.x + root.width)).abs() < 0.001,
                "edge from_x should be root right edge"
            );
        }
    }

    #[test]
    fn estimate_node_size_basic() {
        let (w, h, lines) = estimate_node_size("hello", false);
        assert!(w >= MIN_NODE_WIDTH);
        assert!(h >= MIN_NODE_HEIGHT);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn estimate_node_size_multiline() {
        let (_, h, lines) = estimate_node_size("line1\\nline2\\nline3", false);
        assert_eq!(lines.len(), 3);
        assert!(h > MIN_NODE_HEIGHT);
    }

    #[test]
    fn estimate_node_size_root_larger() {
        let (w_root, h_root, _) = estimate_node_size("test", true);
        let (w_child, h_child, _) = estimate_node_size("test", false);
        assert!(w_root > w_child);
        assert!(h_root > h_child);
    }

    #[test]
    fn split_text_lines_single() {
        let lines = split_text_lines("hello");
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn split_text_lines_multi() {
        let lines = split_text_lines("a \\n b \\n c");
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn layout_multiline_node() {
        let mut root = MindmapNode::new("Root", 1);
        root.children
            .push(MindmapNode::new("Line1\\nLine2\\nLine3", 2));
        let diagram = MindmapDiagram {
            root,
            notes: vec![],
        };
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.nodes.len(), 2);
        assert_eq!(layout.nodes[1].lines.len(), 3);
    }

    #[test]
    fn layout_wide_tree() {
        let mut root = MindmapNode::new("Root", 1);
        for i in 0..10 {
            root.children
                .push(MindmapNode::new(&format!("Child{}", i), 2));
        }
        let diagram = MindmapDiagram {
            root,
            notes: vec![],
        };
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.nodes.len(), 11);
        assert_eq!(layout.edges.len(), 10);
        // Verify all children are vertically ordered
        for i in 1..10 {
            assert!(layout.nodes[i].y < layout.nodes[i + 1].y);
        }
    }

    #[test]
    fn layout_note_attaches_to_root() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("Root", 1),
            notes: vec![MindmapNote {
                text: "hello".to_string(),
                position: "right".to_string(),
            }],
        };
        let layout = layout_mindmap(&diagram).unwrap();
        assert_eq!(layout.notes.len(), 1);
        assert!(layout.notes[0].x > layout.nodes[0].x + layout.nodes[0].width);
        assert!(layout.notes[0].connector.is_some());
    }
}
