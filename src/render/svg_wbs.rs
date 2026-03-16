use std::collections::HashMap;
use std::fmt::Write;

use super::svg::{write_bg_rect, write_svg_root_bg};
use crate::font_metrics;
use crate::layout::wbs::{WbsEdgeLayout, WbsLayout, WbsNodeLayout, WbsNoteLayout};
use crate::model::wbs::WbsDiagram;
use crate::render::svg::{fmt_coord, xml_escape};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 12.0;
const ASCENT: f64 = 11.138672;
const LINE_HEIGHT: f64 = 13.96875;
const NODE_BG: &str = "#F1F1F1";
const NODE_BORDER: &str = "#181818";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const STROKE_WIDTH: &str = "1.5";
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const NOTE_FOLD: f64 = 8.0;
const PAD: f64 = 10.0;

pub fn render_wbs(_wd: &WbsDiagram, layout: &WbsLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "WBS", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);

    let wbs_bg = skin.background_color("wbs", NODE_BG);
    let wbs_border = skin.border_color("wbs", NODE_BORDER);
    let wbs_font = skin.font_color("wbs", TEXT_FILL);
    let edge_color = skin.arrow_color(EDGE_COLOR);

    // Build parent_node_index -> [(edge_index, child_node_index)] map
    // Edges store from_x/from_y = parent bottom center, to_x/to_y = child top center
    // Nodes are in depth-first order. Match edges to children by to_x/to_y matching node center_x/y.
    let mut parent_children: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    let mut child_nodes: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (ei, edge) in layout.edges.iter().enumerate() {
        // Find parent node: node whose bottom center matches edge from_x, y+h = from_y
        let parent_idx = layout.nodes.iter().position(|n| {
            let cx = n.x + n.width / 2.0;
            let by = n.y + n.height;
            (cx - edge.from_x).abs() < 0.01 && (by - edge.from_y).abs() < 0.01
        });
        // Find child node: node whose top center matches edge to_x, y = to_y
        let child_idx = layout.nodes.iter().position(|n| {
            let cx = n.x + n.width / 2.0;
            (cx - edge.to_x).abs() < 0.01 && (n.y - edge.to_y).abs() < 0.01
        });
        if let (Some(pi), Some(ci)) = (parent_idx, child_idx) {
            parent_children.entry(pi).or_default().push((ei, ci));
            child_nodes.insert(ci);
        }
    }

    // Find root node (not a child of any edge)
    if !layout.nodes.is_empty() {
        let root_idx = (0..layout.nodes.len()).find(|i| !child_nodes.contains(i)).unwrap_or(0);

        // Recursive depth-first render matching Java order:
        // For a parent with children:
        //   1. For each child: vertical_drop_line, then recurse into child subtree
        //   2. Horizontal connector bar
        //   3. Parent rect + text
        //   4. Parent vertical drop
        render_subtree(
            &mut buf, layout, root_idx, &parent_children,
            wbs_bg, wbs_border, wbs_font, edge_color,
        );
    }

    // Extra links (alias-to-alias arrows)
    for link in &layout.extra_links {
        render_extra_link(&mut buf, link, edge_color);
    }

    for note in &layout.notes {
        render_note(&mut buf, note, wbs_font);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_subtree(
    buf: &mut String,
    layout: &WbsLayout,
    node_idx: usize,
    parent_children: &HashMap<usize, Vec<(usize, usize)>>,
    bg: &str, border: &str, font_color: &str, edge_color: &str,
) {
    let children = parent_children.get(&node_idx);

    if let Some(child_list) = children {
        // 1. For each child: render vertical drop line, then child subtree
        for &(ei, ci) in child_list {
            let edge = &layout.edges[ei];
            let connector_y = (edge.from_y + edge.to_y) / 2.0;
            // Vertical drop from connector to child top
            write!(
                buf,
                r#"<line style="stroke:{edge_color};stroke-width:{STROKE_WIDTH};" x1="{cx}" x2="{cx}" y1="{cy}" y2="{ty}"/>"#,
                cx = fmt_coord(edge.to_x),
                cy = fmt_coord(connector_y),
                ty = fmt_coord(edge.to_y),
            ).unwrap();

            // Recurse into child subtree
            render_subtree(buf, layout, ci, parent_children, bg, border, font_color, edge_color);
        }

        // 2. Horizontal connector bar (if >1 children)
        let edges: Vec<&WbsEdgeLayout> = child_list.iter().map(|&(ei, _)| &layout.edges[ei]).collect();
        let connector_y = (edges[0].from_y + edges[0].to_y) / 2.0;
        if edges.len() > 1 {
            let min_x = edges.iter().map(|e| e.to_x).fold(f64::INFINITY, f64::min);
            let max_x = edges.iter().map(|e| e.to_x).fold(f64::NEG_INFINITY, f64::max);
            write!(
                buf,
                r#"<line style="stroke:{edge_color};stroke-width:{STROKE_WIDTH};" x1="{x1}" x2="{x2}" y1="{y}" y2="{y}"/>"#,
                x1 = fmt_coord(min_x), x2 = fmt_coord(max_x), y = fmt_coord(connector_y),
            ).unwrap();
        }

        // 3. Parent rect + text
        render_node(buf, &layout.nodes[node_idx], bg, border, font_color);

        // 4. Parent vertical drop
        let from_y = edges[0].from_y;
        let from_x = edges[0].from_x;
        write!(
            buf,
            r#"<line style="stroke:{edge_color};stroke-width:{STROKE_WIDTH};" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
            x = fmt_coord(from_x), y1 = fmt_coord(from_y), y2 = fmt_coord(connector_y),
        ).unwrap();
    } else {
        // Leaf node: just render rect + text
        render_node(buf, &layout.nodes[node_idx], bg, border, font_color);
    }
}

fn render_node(buf: &mut String, node: &WbsNodeLayout, bg: &str, border: &str, font_color: &str) {
    write!(
        buf,
        r#"<rect fill="{bg}" height="{h}" style="stroke:{border};stroke-width:{STROKE_WIDTH};" width="{w}" x="{x}" y="{y}"/>"#,
        h = fmt_coord(node.height),
        w = fmt_coord(node.width),
        x = fmt_coord(node.x),
        y = fmt_coord(node.y),
    ).unwrap();

    // For nodes with hyperlinks or complex creole, use render_creole_text
    if node.text.contains("[[") {
        use crate::render::svg_richtext::render_creole_text;
        let text_x = node.x + PAD;
        let text_y = node.y + PAD + ASCENT;
        render_creole_text(
            buf, &node.text, text_x, text_y, LINE_HEIGHT,
            font_color, None, &format!(r#"font-size="{FONT_SIZE:.0}""#),
        );
        return;
    }

    // Simple text: each line as a separate <text> element, left-aligned
    let visible = crate::model::hyperlink::extract_hyperlinks(&node.text).0;
    let lines: Vec<&str> = visible.lines().collect();
    let text_x = node.x + PAD;
    for (i, line) in lines.iter().enumerate() {
        let text_y = node.y + PAD + ASCENT + i as f64 * LINE_HEIGHT;
        let text_len = font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false);
        write!(
            buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{FONT_SIZE:.0}" lengthAdjust="spacing" textLength="{tl}" x="{tx}" y="{ty}">{text}</text>"#,
            tl = fmt_coord(text_len),
            tx = fmt_coord(text_x),
            ty = fmt_coord(text_y),
            text = xml_escape(line),
        ).unwrap();
    }
}

fn render_note(buf: &mut String, note: &WbsNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        write!(
            buf,
            r#"<line style="stroke:{NOTE_BORDER};stroke-width:0.5;stroke-dasharray:4,4;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(x1), fmt_coord(x2), fmt_coord(y1), fmt_coord(y2),
        ).unwrap();
    }
    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    let x2 = note.x + note.width;
    let y2 = note.y + note.height;
    write!(
        buf,
        r#"<polygon fill="{NOTE_BG}" points="{},{} {},{} {},{} {},{} {},{}" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(note.x), fmt_coord(note.y), fmt_coord(fold_x), fmt_coord(note.y),
        fmt_coord(x2), fmt_coord(fold_y), fmt_coord(x2), fmt_coord(y2),
        fmt_coord(note.x), fmt_coord(y2),
    ).unwrap();
    write!(
        buf,
        r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(fold_x), fmt_coord(note.y), fmt_coord(fold_x), fmt_coord(fold_y),
        fmt_coord(x2), fmt_coord(fold_y),
    ).unwrap();

    use crate::render::svg_richtext::render_creole_text;
    render_creole_text(buf, &note.text, note.x + 6.0, note.y + NOTE_FOLD + FONT_SIZE,
        LINE_HEIGHT, font_color, None, r#"font-size="13""#);
}

fn render_extra_link(buf: &mut String, link: &WbsEdgeLayout, color: &str) {
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(link.from_x), fmt_coord(link.to_x),
        fmt_coord(link.from_y), fmt_coord(link.to_y),
    ).unwrap();

    let dx = link.to_x - link.from_x;
    let dy = link.to_y - link.from_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        let ux = dx / len;
        let uy = dy / len;
        let tip_x = link.to_x;
        let tip_y = link.to_y;
        let back = 9.0;
        let spread = 4.0;
        let base_x = tip_x - ux * back;
        let base_y = tip_y - uy * back;
        let left_x = base_x + uy * spread;
        let left_y = base_y - ux * spread;
        let mid_x = tip_x - ux * (back - 4.0);
        let mid_y = tip_y - uy * (back - 4.0);
        let right_x = base_x - uy * spread;
        let right_y = base_y + ux * spread;
        write!(
            buf,
            r#"<polygon fill="{color}" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{color};stroke-width:1;"/>"#,
            fmt_coord(tip_x), fmt_coord(tip_y),
            fmt_coord(left_x), fmt_coord(left_y),
            fmt_coord(mid_x), fmt_coord(mid_y),
            fmt_coord(right_x), fmt_coord(right_y),
            fmt_coord(tip_x), fmt_coord(tip_y),
        ).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::wbs::{WbsEdgeLayout, WbsLayout, WbsNodeLayout, WbsNoteLayout};
    use crate::model::wbs::{WbsDiagram, WbsDirection, WbsNode};
    use crate::style::SkinParams;

    fn empty_wbs() -> WbsDiagram {
        WbsDiagram {
            root: WbsNode { text: "R".into(), children: vec![], direction: WbsDirection::Default, alias: None, level: 1 },
            links: vec![], notes: vec![],
        }
    }
    fn empty_layout() -> WbsLayout {
        WbsLayout { nodes: vec![], edges: vec![], extra_links: vec![], notes: vec![], width: 200.0, height: 100.0 }
    }
    fn make_node(text: &str, level: usize, x: f64, y: f64, w: f64, h: f64) -> WbsNodeLayout {
        WbsNodeLayout { text: text.into(), alias: None, x, y, width: w, height: h, level }
    }

    #[test] fn test_svg_header() {
        let svg = render_wbs(&empty_wbs(), &empty_layout(), &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg")); assert!(svg.contains("</svg>"));
        assert!(svg.contains("contentStyleType=\"text/css\""));
    }
    #[test] fn test_node_fill() {
        let mut l = empty_layout();
        l.nodes.push(make_node("Root", 1, 50.0, 10.0, 80.0, 30.0));
        let svg = render_wbs(&empty_wbs(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r##"fill="#F1F1F1""##));
        assert!(!svg.contains("rx="));
    }
    #[test] fn test_text() {
        let mut l = empty_layout();
        l.nodes.push(make_node("Hello", 1, 10.0, 10.0, 80.0, 30.0));
        let svg = render_wbs(&empty_wbs(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("Hello"));
        assert!(svg.contains(r#"font-size="12""#));
    }
    #[test] fn test_multiline() {
        let mut l = empty_layout();
        l.nodes.push(make_node("A\nB", 2, 10.0, 10.0, 100.0, 50.0));
        let svg = render_wbs(&empty_wbs(), &l, &SkinParams::default()).unwrap();
        assert_eq!(svg.matches("<text ").count(), 2);
    }
    #[test] fn test_edge() {
        let mut l = empty_layout();
        l.nodes.push(make_node("R", 1, 90.0, 10.0, 20.0, 30.0));
        l.nodes.push(make_node("C", 2, 80.0, 80.0, 40.0, 30.0));
        l.edges.push(WbsEdgeLayout { from_x: 100.0, from_y: 40.0, to_x: 100.0, to_y: 80.0 });
        let svg = render_wbs(&empty_wbs(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<line"));
    }
    #[test] fn test_extra_link() {
        let mut l = empty_layout();
        l.extra_links.push(WbsEdgeLayout { from_x: 150.0, from_y: 50.0, to_x: 50.0, to_y: 50.0 });
        let svg = render_wbs(&empty_wbs(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
    }
}
