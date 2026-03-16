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

#[derive(Debug)]
pub struct WbsLayout {
    pub nodes: Vec<WbsNodeLayout>,
    pub edges: Vec<WbsEdgeLayout>,
    pub extra_links: Vec<WbsEdgeLayout>,
    pub notes: Vec<WbsNoteLayout>,
    pub width: f64,
    pub height: f64,
}

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

#[derive(Debug)]
pub struct WbsEdgeLayout {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
}

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
// Constants — derived from Java PlantUML WBS reference output
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
/// AWT line height for SansSerif 12pt = ascent + descent = 13.96875
const LINE_HEIGHT: f64 = 13.96875;
const PAD_H: f64 = 10.0;
const PAD_V: f64 = 10.0;
/// Vertical gap: parent bottom to connector, and connector to child top
const EDGE_GAP: f64 = 20.0;
const NODE_SPACING: f64 = 20.0;
const MARGIN: f64 = 20.0;
const NOTE_GAP: f64 = 16.0;
const MIN_NOTE_WIDTH: f64 = 60.0;
const MIN_NOTE_HEIGHT: f64 = 28.0;

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

fn text_width(text: &str) -> f64 {
    extract_hyperlinks(text)
        .0
        .lines()
        .map(|l| font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max)
}

fn node_size(text: &str) -> (f64, f64) {
    let visible = extract_hyperlinks(text).0;
    let line_count = visible.lines().count().max(1) as f64;
    let w = text_width(text) + 2.0 * PAD_H;
    let h = line_count * LINE_HEIGHT + 2.0 * PAD_V;
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

fn layout_node(
    node: &WbsNode, cx: f64, y: f64,
    nodes: &mut Vec<WbsNodeLayout>, edges: &mut Vec<WbsEdgeLayout>,
) {
    let (w, h) = node_size(&node.text);
    let x = cx - w / 2.0;
    nodes.push(WbsNodeLayout {
        text: node.text.clone(), alias: node.alias.clone(),
        x, y, width: w, height: h, level: node.level,
    });

    if node.children.is_empty() { return; }

    // Child top = parent bottom + 2*EDGE_GAP
    let child_y = y + h + 2.0 * EDGE_GAP;

    let child_widths: Vec<f64> = node.children.iter().map(subtree_width).collect();
    let total_children_width: f64 =
        child_widths.iter().sum::<f64>() + (node.children.len() as f64 - 1.0) * NODE_SPACING;
    let mut child_cx = cx - total_children_width / 2.0;

    for (i, child) in node.children.iter().enumerate() {
        let cw = child_widths[i];
        let this_cx = child_cx + cw / 2.0;
        edges.push(WbsEdgeLayout {
            from_x: cx, from_y: y + h, to_x: this_cx, to_y: child_y,
        });
        layout_node(child, this_cx, child_y, nodes, edges);
        child_cx += cw + NODE_SPACING;
    }
}

fn layout_notes(notes: &[WbsNote], root: &WbsNodeLayout) -> Vec<WbsNoteLayout> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut result = Vec::new();
    let rcx = root.x + root.width / 2.0;
    let rcy = root.y + root.height / 2.0;

    for note in notes {
        let (width, height) = note_size(&note.text);
        let si = {
            let c = counts.entry(note.position.as_str()).or_insert(0);
            let v = *c as f64; *c += 1; v
        };
        let (x, y, conn) = match note.position.as_str() {
            "left" => {
                let x = root.x - NOTE_GAP - width;
                let y = root.y + si * (height + NOTE_GAP);
                (x, y, Some((root.x, rcy, x + width, y + height / 2.0)))
            }
            "top" => {
                let x = rcx - width / 2.0 + si * (NOTE_GAP + 20.0);
                let y = root.y - NOTE_GAP - height;
                (x, y, Some((rcx, root.y, x + width / 2.0, y + height)))
            }
            "bottom" => {
                let x = rcx - width / 2.0 + si * (NOTE_GAP + 20.0);
                let y = root.y + root.height + NOTE_GAP;
                (x, y, Some((rcx, root.y + root.height, x + width / 2.0, y)))
            }
            _ => {
                let x = root.x + root.width + NOTE_GAP;
                let y = root.y + si * (height + NOTE_GAP);
                (x, y, Some((root.x + root.width, rcy, x, y + height / 2.0)))
            }
        };
        result.push(WbsNoteLayout { text: note.text.clone(), x, y, width, height, connector: conn });
    }
    result
}

pub fn layout_wbs(wd: &WbsDiagram) -> Result<WbsLayout> {
    debug!("layout_wbs: root='{}'", wd.root.text);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let total_width = subtree_width(&wd.root);
    let cx = MARGIN + total_width / 2.0;
    layout_node(&wd.root, cx, MARGIN, &mut nodes, &mut edges);

    // Build alias -> node rect for edge-to-edge arrow connections
    let alias_rect: HashMap<String, (f64, f64, f64, f64)> = nodes.iter()
        .filter_map(|n| n.alias.as_ref().map(|a| (a.clone(), (n.x, n.y, n.width, n.height))))
        .collect();

    let mut extra_links = Vec::new();
    for link in &wd.links {
        if let (Some(&(fx, fy, fw, fh)), Some(&(tx, _ty, tw, _th))) =
            (alias_rect.get(&link.from), alias_rect.get(&link.to))
        {
            let from_cx = fx + fw / 2.0;
            let to_cx = tx + tw / 2.0;
            let link_y = fy + fh / 2.0;
            // Arrow from near edge of source to near edge of target
            let (lx_from, lx_to) = if from_cx > to_cx {
                // Source is to the right, arrow points left
                (fx, tx + tw)
            } else {
                // Source is to the left, arrow points right
                (fx + fw, tx)
            };
            extra_links.push(WbsEdgeLayout {
                from_x: lx_from, from_y: link_y,
                to_x: lx_to, to_y: link_y,
            });
        }
    }

    let root_layout = nodes.iter().find(|n| n.level == 1).cloned().unwrap_or_else(|| nodes[0].clone());
    let mut notes = layout_notes(&wd.notes, &root_layout);

    let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
    let (mut max_x, mut max_y) = (0.0_f64, 0.0_f64);
    for n in &nodes {
        min_x = min_x.min(n.x); min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + n.width); max_y = max_y.max(n.y + n.height);
    }
    for n in &notes {
        min_x = min_x.min(n.x); min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + n.width); max_y = max_y.max(n.y + n.height);
    }

    let sx = if min_x < MARGIN { MARGIN - min_x } else { 0.0 };
    let sy = if min_y < MARGIN { MARGIN - min_y } else { 0.0 };
    if sx > 0.0 || sy > 0.0 {
        for n in &mut nodes { n.x += sx; n.y += sy; }
        for e in &mut edges { e.from_x += sx; e.to_x += sx; e.from_y += sy; e.to_y += sy; }
        for l in &mut extra_links { l.from_x += sx; l.to_x += sx; l.from_y += sy; l.to_y += sy; }
        for n in &mut notes {
            n.x += sx; n.y += sy;
            if let Some((x1, y1, x2, y2)) = n.connector.as_mut() {
                *x1 += sx; *x2 += sx; *y1 += sy; *y2 += sy;
            }
        }
        max_x += sx; max_y += sy;
    }

    Ok(WbsLayout { nodes, edges, extra_links, notes, width: max_x + MARGIN, height: max_y + MARGIN })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::wbs::{WbsDiagram, WbsDirection, WbsLink, WbsNode, WbsNote};

    fn leaf(text: &str, level: usize) -> WbsNode {
        WbsNode { text: text.to_string(), children: vec![], direction: WbsDirection::Default, alias: None, level }
    }
    fn leaf_alias(text: &str, alias: &str, level: usize) -> WbsNode {
        WbsNode { text: text.to_string(), children: vec![], direction: WbsDirection::Default, alias: Some(alias.into()), level }
    }
    fn mkd(root: WbsNode) -> WbsDiagram { WbsDiagram { root, links: vec![], notes: vec![] } }

    #[test] fn test_single_root() {
        let l = layout_wbs(&mkd(leaf("Root", 1))).unwrap();
        assert_eq!(l.nodes.len(), 1); assert!(l.edges.is_empty());
    }
    #[test] fn test_root_with_children() {
        let r = WbsNode { text: "Root".into(), children: vec![leaf("A",2),leaf("B",2)], direction: WbsDirection::Default, alias: None, level: 1 };
        let l = layout_wbs(&mkd(r)).unwrap();
        assert_eq!(l.nodes.len(), 3); assert_eq!(l.edges.len(), 2);
    }
    #[test] fn test_children_below() {
        let r = WbsNode { text: "Root".into(), children: vec![leaf("A",2)], direction: WbsDirection::Default, alias: None, level: 1 };
        let l = layout_wbs(&mkd(r)).unwrap();
        assert!(l.nodes[1].y > l.nodes[0].y);
    }
    #[test] fn test_multiline() {
        let (_,h1) = node_size("One"); let (_,h2) = node_size("A\nB");
        assert!(h2 > h1);
    }
    #[test] fn test_extra_links() {
        let r = WbsNode { text: "R".into(), children: vec![leaf_alias("A","AA",2), leaf_alias("B","BB",2)], direction: WbsDirection::Default, alias: None, level: 1 };
        let d = WbsDiagram { root: r, links: vec![WbsLink{from:"AA".into(),to:"BB".into()}], notes: vec![] };
        assert_eq!(layout_wbs(&d).unwrap().extra_links.len(), 1);
    }
    #[test] fn test_note() {
        let d = WbsDiagram { root: leaf("R",1), links: vec![], notes: vec![WbsNote{text:"hi".into(),position:"right".into()}] };
        let l = layout_wbs(&d).unwrap();
        assert_eq!(l.notes.len(), 1);
        assert!(l.notes[0].x > l.nodes[0].x + l.nodes[0].width);
    }
    #[test] fn test_bbox() {
        let r = WbsNode { text: "R".into(), children: vec![leaf("A",2),leaf("B",2)], direction: WbsDirection::Default, alias: None, level: 1 };
        let l = layout_wbs(&mkd(r)).unwrap();
        for n in &l.nodes { assert!(n.x+n.width <= l.width); assert!(n.y+n.height <= l.height); }
    }
}
