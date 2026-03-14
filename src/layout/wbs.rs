//! WBS diagram layout engine.
//!
//! Converts a `WbsDiagram` into a fully positioned `WbsLayout` ready for
//! SVG rendering.  The algorithm uses a top-down tree placement: root at
//! the top center, children spread horizontally below.

use std::collections::HashMap;

use log::debug;

use crate::font_metrics;
use crate::model::hyperlink::extract_hyperlinks;
use crate::model::richtext::plain_text;
use crate::model::wbs::{WbsDiagram, WbsNode, WbsNote};
use crate::parser::creole::parse_creole;
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned WBS diagram ready for rendering.
#[derive(Debug)]
pub struct WbsLayout {
    pub nodes: Vec<WbsNodeLayout>,
    pub edges: Vec<WbsEdgeLayout>,
    pub extra_links: Vec<WbsEdgeLayout>,
    pub notes: Vec<WbsNoteLayout>,
    pub width: f64,
    pub height: f64,
}

/// A single positioned WBS node.
#[derive(Debug, Clone)]
pub struct WbsNodeLayout {
    pub text: String,
    pub alias: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub level: usize,
}

/// An edge connecting parent to child (or extra link between aliases).
#[derive(Debug)]
pub struct WbsEdgeLayout {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
}

/// A positioned note annotation attached to the root node.
#[derive(Debug, Clone)]
pub struct WbsNoteLayout {
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

const FONT_SIZE: f64 = 14.0;
const LINE_HEIGHT: f64 = 16.0;
const PAD_H: f64 = 8.0;
const PAD_V: f64 = 4.0;
const LEVEL_HEIGHT: f64 = 60.0;
const NODE_SPACING: f64 = 20.0;
const MARGIN: f64 = 20.0;
const NOTE_GAP: f64 = 16.0;
const MIN_NOTE_WIDTH: f64 = 60.0;
const MIN_NOTE_HEIGHT: f64 = 28.0;

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

/// Estimate pixel width of a (possibly multi-line) text label.
fn text_width(text: &str) -> f64 {
    extract_hyperlinks(text)
        .0
        .lines()
        .map(|l| font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max)
}

/// Estimate node box size from its text.
fn node_size(text: &str) -> (f64, f64) {
    let visible = extract_hyperlinks(text).0;
    let line_count = visible.lines().count().max(1) as f64;
    let w = text_width(text) + 2.0 * PAD_H;
    let w = w.max(40.0);
    let h = line_count * LINE_HEIGHT + 2.0 * PAD_V;
    let h = h.max(24.0);
    (w, h)
}

fn note_size(text: &str) -> (f64, f64) {
    let plain = plain_text(&parse_creole(text)).replace("\\n", "\n");
    let lines: Vec<&str> = plain.lines().collect();
    let max_width = lines
        .iter()
        .map(|line| font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    let width = (max_width + 2.0 * PAD_H).max(MIN_NOTE_WIDTH);
    let height = (lines.len().max(1) as f64 * LINE_HEIGHT + 2.0 * PAD_V).max(MIN_NOTE_HEIGHT);
    (width, height)
}

// ---------------------------------------------------------------------------
// Subtree width calculation
// ---------------------------------------------------------------------------

/// Calculate the total width a subtree needs (including all descendants).
fn subtree_width(node: &WbsNode) -> f64 {
    let (self_w, _) = node_size(&node.text);

    if node.children.is_empty() {
        return self_w;
    }

    let children_total: f64 = node.children.iter().map(subtree_width).sum::<f64>()
        + (node.children.len() as f64 - 1.0) * NODE_SPACING;

    self_w.max(children_total)
}

// ---------------------------------------------------------------------------
// Layout tree recursion
// ---------------------------------------------------------------------------

/// Recursively lay out a WBS node subtree.
///
/// `cx` = center x for this subtree, `y` = top y for this node.
///
/// Populates `nodes` and `edges` vectors.
fn layout_node(
    node: &WbsNode,
    cx: f64,
    y: f64,
    nodes: &mut Vec<WbsNodeLayout>,
    edges: &mut Vec<WbsEdgeLayout>,
) {
    let (w, h) = node_size(&node.text);
    let x = cx - w / 2.0;

    nodes.push(WbsNodeLayout {
        text: node.text.clone(),
        alias: node.alias.clone(),
        x,
        y,
        width: w,
        height: h,
        level: node.level,
    });

    if node.children.is_empty() {
        return;
    }

    // The child row starts at y + LEVEL_HEIGHT
    let child_y = y + LEVEL_HEIGHT;

    // Calculate the total width needed by children
    let child_widths: Vec<f64> = node.children.iter().map(subtree_width).collect();
    let total_children_width: f64 =
        child_widths.iter().sum::<f64>() + (node.children.len() as f64 - 1.0) * NODE_SPACING;

    // Start position for the leftmost child
    let mut child_cx = cx - total_children_width / 2.0;

    for (i, child) in node.children.iter().enumerate() {
        let cw = child_widths[i];
        let this_cx = child_cx + cw / 2.0;

        // Edge from parent bottom-center to child top-center
        let parent_bottom_cx = cx;
        let parent_bottom_y = y + h;
        let (child_w, _child_h) = node_size(&child.text);
        let _ = child_w; // child center is at this_cx

        edges.push(WbsEdgeLayout {
            from_x: parent_bottom_cx,
            from_y: parent_bottom_y,
            to_x: this_cx,
            to_y: child_y,
        });

        layout_node(child, this_cx, child_y, nodes, edges);

        child_cx += cw + NODE_SPACING;
    }
}

fn layout_notes(notes: &[WbsNote], root: &WbsNodeLayout) -> Vec<WbsNoteLayout> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut result = Vec::new();
    let root_center_x = root.x + root.width / 2.0;
    let root_center_y = root.y + root.height / 2.0;

    for note in notes {
        let (width, height) = note_size(&note.text);
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

        result.push(WbsNoteLayout {
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

/// Perform the complete layout of a WBS diagram.
pub fn layout_wbs(wd: &WbsDiagram) -> Result<WbsLayout> {
    debug!("layout_wbs: root='{}'", wd.root.text);

    let mut nodes: Vec<WbsNodeLayout> = Vec::new();
    let mut edges: Vec<WbsEdgeLayout> = Vec::new();

    // Calculate root subtree width to determine diagram center
    let total_width = subtree_width(&wd.root);
    let cx = MARGIN + total_width / 2.0;
    let start_y = MARGIN;

    layout_node(&wd.root, cx, start_y, &mut nodes, &mut edges);

    // Build alias -> (center_x, center_y) map for extra links
    let alias_map: HashMap<String, (f64, f64)> = nodes
        .iter()
        .filter_map(|n| {
            n.alias
                .as_ref()
                .map(|a| (a.clone(), (n.x + n.width / 2.0, n.y + n.height / 2.0)))
        })
        .collect();

    let mut extra_links = Vec::new();
    for link in &wd.links {
        let from_pos = alias_map.get(&link.from);
        let to_pos = alias_map.get(&link.to);

        if let (Some(&(fx, fy)), Some(&(tx, ty))) = (from_pos, to_pos) {
            debug!(
                "extra link '{}' -> '{}': ({:.1},{:.1}) -> ({:.1},{:.1})",
                link.from, link.to, fx, fy, tx, ty
            );
            extra_links.push(WbsEdgeLayout {
                from_x: fx,
                from_y: fy,
                to_x: tx,
                to_y: ty,
            });
        } else {
            log::warn!(
                "extra link '{}' -> '{}': alias not found",
                link.from,
                link.to
            );
        }
    }

    let root_layout = nodes
        .iter()
        .find(|node| node.level == 1)
        .cloned()
        .unwrap_or_else(|| nodes[0].clone());
    let mut notes = layout_notes(&wd.notes, &root_layout);

    // Compute bounding box
    let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
    let (mut max_x, mut max_y) = (0.0_f64, 0.0_f64);
    for n in &nodes {
        min_x = min_x.min(n.x);
        min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + n.width);
        max_y = max_y.max(n.y + n.height);
    }
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
        for link in &mut extra_links {
            link.from_x += shift_x;
            link.to_x += shift_x;
            link.from_y += shift_y;
            link.to_y += shift_y;
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

    debug!(
        "layout_wbs done: {:.0}x{:.0}, {} nodes, {} edges, {} extra links",
        width,
        height,
        nodes.len(),
        edges.len(),
        extra_links.len()
    );

    Ok(WbsLayout {
        nodes,
        edges,
        extra_links,
        notes,
        width,
        height,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::wbs::{WbsDiagram, WbsDirection, WbsLink, WbsNode, WbsNote};

    fn leaf(text: &str, level: usize) -> WbsNode {
        WbsNode {
            text: text.to_string(),
            children: vec![],
            direction: WbsDirection::Default,
            alias: None,
            level,
        }
    }

    fn leaf_with_alias(text: &str, alias: &str, level: usize) -> WbsNode {
        WbsNode {
            text: text.to_string(),
            children: vec![],
            direction: WbsDirection::Default,
            alias: Some(alias.to_string()),
            level,
        }
    }

    fn diagram_from_root(root: WbsNode) -> WbsDiagram {
        WbsDiagram {
            root,
            links: vec![],
            notes: vec![],
        }
    }

    // ── 1. Single root ──────────────────────────────────────────────

    #[test]
    fn test_single_root() {
        let d = diagram_from_root(leaf("Root", 1));
        let layout = layout_wbs(&d).unwrap();
        assert_eq!(layout.nodes.len(), 1);
        assert!(layout.edges.is_empty());
        assert_eq!(layout.nodes[0].text, "Root");
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // ── 2. Root with children ───────────────────────────────────────

    #[test]
    fn test_root_with_children() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf("A", 2), leaf("B", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();
        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.edges.len(), 2);
    }

    // ── 3. Children are below parent ────────────────────────────────

    #[test]
    fn test_children_below_parent() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf("A", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();
        let root_node = &layout.nodes[0];
        let child_node = &layout.nodes[1];
        assert!(
            child_node.y > root_node.y,
            "child y={} should be > root y={}",
            child_node.y,
            root_node.y
        );
    }

    // ── 4. Siblings spread horizontally ─────────────────────────────

    #[test]
    fn test_siblings_spread_horizontal() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf("A", 2), leaf("B", 2), leaf("C", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();

        // children are nodes[1], nodes[2], nodes[3]
        let a = &layout.nodes[1];
        let b = &layout.nodes[2];
        let c = &layout.nodes[3];

        // All at same y
        assert!((a.y - b.y).abs() < 1.0, "siblings A and B at same y");
        assert!((b.y - c.y).abs() < 1.0, "siblings B and C at same y");

        // Left to right ordering
        assert!(a.x < b.x, "A.x={} should be < B.x={}", a.x, b.x);
        assert!(b.x < c.x, "B.x={} should be < C.x={}", b.x, c.x);
    }

    // ── 5. Node sizing ──────────────────────────────────────────────

    #[test]
    fn test_node_sizing() {
        let (w, h) = node_size("Short");
        assert!(w >= 40.0, "minimum width");
        assert!(h >= 24.0, "minimum height");

        let (w_long, _) = node_size("A very long text label here");
        assert!(
            w_long > w,
            "longer text should be wider: {} > {}",
            w_long,
            w
        );
    }

    // ── 6. Multiline node sizing ────────────────────────────────────

    #[test]
    fn test_multiline_node_sizing() {
        let (_, h1) = node_size("One line");
        let (_, h2) = node_size("Line 1\nLine 2");
        assert!(h2 > h1, "multiline should be taller: {} > {}", h2, h1);
    }

    // ── 7. Deep nesting ─────────────────────────────────────────────

    #[test]
    fn test_deep_nesting() {
        let l4 = leaf("L4", 4);
        let l3 = WbsNode {
            text: "L3".to_string(),
            children: vec![l4],
            direction: WbsDirection::Default,
            alias: None,
            level: 3,
        };
        let l2 = WbsNode {
            text: "L2".to_string(),
            children: vec![l3],
            direction: WbsDirection::Default,
            alias: None,
            level: 2,
        };
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![l2],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();

        assert_eq!(layout.nodes.len(), 4);
        assert_eq!(layout.edges.len(), 3);

        // Each level should be below the previous
        for i in 1..layout.nodes.len() {
            assert!(
                layout.nodes[i].y > layout.nodes[i - 1].y,
                "node {} should be below node {}",
                i,
                i - 1
            );
        }
    }

    // ── 8. Extra links between aliases ──────────────────────────────

    #[test]
    fn test_extra_links() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf_with_alias("A", "AA", 2), leaf_with_alias("B", "BB", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = WbsDiagram {
            root,
            links: vec![WbsLink {
                from: "AA".to_string(),
                to: "BB".to_string(),
            }],
            notes: vec![],
        };
        let layout = layout_wbs(&d).unwrap();
        assert_eq!(layout.extra_links.len(), 1);
    }

    // ── 9. Missing alias in extra link ──────────────────────────────

    #[test]
    fn test_missing_alias_link() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf_with_alias("A", "AA", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = WbsDiagram {
            root,
            links: vec![WbsLink {
                from: "AA".to_string(),
                to: "MISSING".to_string(),
            }],
            notes: vec![],
        };
        let layout = layout_wbs(&d).unwrap();
        assert_eq!(
            layout.extra_links.len(),
            0,
            "missing alias should skip link"
        );
    }

    #[test]
    fn test_note_layout() {
        let d = WbsDiagram {
            root: leaf("Root", 1),
            links: vec![],
            notes: vec![WbsNote {
                text: "hello".to_string(),
                position: "right".to_string(),
            }],
        };
        let layout = layout_wbs(&d).unwrap();
        assert_eq!(layout.notes.len(), 1);
        assert!(layout.notes[0].x > layout.nodes[0].x + layout.nodes[0].width);
        assert!(layout.notes[0].connector.is_some());
    }

    // ── 10. Bounding box includes all nodes ─────────────────────────

    #[test]
    fn test_bounding_box() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf("A", 2), leaf("B", 2), leaf("C", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();

        for n in &layout.nodes {
            assert!(
                n.x + n.width <= layout.width,
                "node right edge {:.1} exceeds width {:.1}",
                n.x + n.width,
                layout.width
            );
            assert!(
                n.y + n.height <= layout.height,
                "node bottom edge {:.1} exceeds height {:.1}",
                n.y + n.height,
                layout.height
            );
        }
    }

    // ── 11. Root is centered over children ──────────────────────────

    #[test]
    fn test_root_centered() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf("A", 2), leaf("B", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();

        let root_cx = layout.nodes[0].x + layout.nodes[0].width / 2.0;
        let child_a = &layout.nodes[1];
        let child_b = &layout.nodes[2];
        let children_cx = (child_a.x + child_a.width / 2.0 + child_b.x + child_b.width / 2.0) / 2.0;

        assert!(
            (root_cx - children_cx).abs() < 1.0,
            "root center {:.1} should be near children center {:.1}",
            root_cx,
            children_cx
        );
    }

    // ── 12. Edges connect parent bottom to child top ────────────────

    #[test]
    fn test_edge_connections() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf("A", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();

        assert_eq!(layout.edges.len(), 1);
        let edge = &layout.edges[0];

        let root_node = &layout.nodes[0];
        let child_node = &layout.nodes[1];

        // from_y should be at parent bottom
        assert!(
            (edge.from_y - (root_node.y + root_node.height)).abs() < 1.0,
            "edge from_y={:.1} should match root bottom={:.1}",
            edge.from_y,
            root_node.y + root_node.height
        );

        // to_y should be at child top
        assert!(
            (edge.to_y - child_node.y).abs() < 1.0,
            "edge to_y={:.1} should match child top={:.1}",
            edge.to_y,
            child_node.y
        );
    }

    // ── 13. Subtree width calculation ───────────────────────────────

    #[test]
    fn test_subtree_width() {
        let l = leaf("Leaf", 2);
        let w = subtree_width(&l);
        let (nw, _) = node_size("Leaf");
        assert!(
            (w - nw).abs() < 0.01,
            "leaf subtree_width should equal node width"
        );

        let parent = WbsNode {
            text: "P".to_string(),
            children: vec![leaf("AA", 2), leaf("BB", 2)],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let pw = subtree_width(&parent);
        assert!(
            pw >= w,
            "parent with children should be wider than single leaf"
        );
    }

    // ── 14. Level values preserved ──────────────────────────────────

    #[test]
    fn test_level_values() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![WbsNode {
                text: "Child".to_string(),
                children: vec![leaf("GC", 3)],
                direction: WbsDirection::Default,
                alias: None,
                level: 2,
            }],
            direction: WbsDirection::Default,
            alias: None,
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();

        assert_eq!(layout.nodes[0].level, 1);
        assert_eq!(layout.nodes[1].level, 2);
        assert_eq!(layout.nodes[2].level, 3);
    }

    // ── 15. Alias preserved in layout ───────────────────────────────

    #[test]
    fn test_alias_preserved() {
        let root = WbsNode {
            text: "Root".to_string(),
            children: vec![leaf_with_alias("Team A", "TLA", 2)],
            direction: WbsDirection::Default,
            alias: Some("R".to_string()),
            level: 1,
        };
        let d = diagram_from_root(root);
        let layout = layout_wbs(&d).unwrap();
        assert_eq!(layout.nodes[0].alias.as_deref(), Some("R"));
        assert_eq!(layout.nodes[1].alias.as_deref(), Some("TLA"));
    }
}
