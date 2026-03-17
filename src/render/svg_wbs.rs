use std::collections::HashMap;
use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::wbs::{WbsEdgeLayout, WbsLayout, WbsNodeLayout, WbsNoteLayout};
use crate::model::wbs::WbsDiagram;
use crate::render::svg::{write_bg_rect, write_svg_root_bg};
use crate::skin::rose::{NOTE_BG, NOTE_BORDER, NOTE_FOLD};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 12.0;
const ASCENT: f64 = 11.138672;
const LINE_HEIGHT: f64 = 13.96875;
const NODE_BG: &str = "#F1F1F1";
const NODE_BORDER: &str = "#181818";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const STROKE_WIDTH: f64 = 1.5;
pub const PAD: f64 = 10.0;

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

    let mut sg = SvgGraphic::new(0, 1.0);

    // Build parent_node_index -> [(edge_index, child_node_index)] map
    let mut parent_children: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    let mut child_nodes: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (ei, edge) in layout.edges.iter().enumerate() {
        let parent_idx = layout.nodes.iter().position(|n| {
            let cx = n.x + n.width / 2.0;
            let by = n.y + n.height;
            (cx - edge.from_x).abs() < 0.01 && (by - edge.from_y).abs() < 0.01
        });
        let child_idx = layout.nodes.iter().position(|n| {
            let cx = n.x + n.width / 2.0;
            (cx - edge.to_x).abs() < 0.01 && (n.y - edge.to_y).abs() < 0.01
        });
        if let (Some(pi), Some(ci)) = (parent_idx, child_idx) {
            parent_children.entry(pi).or_default().push((ei, ci));
            child_nodes.insert(ci);
        }
    }

    if !layout.nodes.is_empty() {
        let root_idx = (0..layout.nodes.len()).find(|i| !child_nodes.contains(i)).unwrap_or(0);
        render_subtree(
            &mut sg, layout, root_idx, &parent_children,
            wbs_bg, wbs_border, wbs_font, edge_color,
        );
    }

    for link in &layout.extra_links {
        render_extra_link(&mut sg, link, edge_color);
    }

    for note in &layout.notes {
        render_note(&mut sg, note, wbs_font);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_subtree(
    sg: &mut SvgGraphic,
    layout: &WbsLayout,
    node_idx: usize,
    parent_children: &HashMap<usize, Vec<(usize, usize)>>,
    bg: &str, border: &str, font_color: &str, edge_color: &str,
) {
    let children = parent_children.get(&node_idx);

    if let Some(child_list) = children {
        for &(ei, ci) in child_list {
            let edge = &layout.edges[ei];
            let connector_y = (edge.from_y + edge.to_y) / 2.0;
            // Vertical drop from connector to child top
            sg.set_stroke_color(Some(edge_color));
            sg.set_stroke_width(STROKE_WIDTH, None);
            sg.svg_line(edge.to_x, connector_y, edge.to_x, edge.to_y, 0.0);

            render_subtree(sg, layout, ci, parent_children, bg, border, font_color, edge_color);
        }

        // Horizontal connector bar
        let edges: Vec<&WbsEdgeLayout> = child_list.iter().map(|&(ei, _)| &layout.edges[ei]).collect();
        let connector_y = (edges[0].from_y + edges[0].to_y) / 2.0;
        if edges.len() > 1 {
            let min_x = edges.iter().map(|e| e.to_x).fold(f64::INFINITY, f64::min);
            let max_x = edges.iter().map(|e| e.to_x).fold(f64::NEG_INFINITY, f64::max);
            sg.set_stroke_color(Some(edge_color));
            sg.set_stroke_width(STROKE_WIDTH, None);
            sg.svg_line(min_x, connector_y, max_x, connector_y, 0.0);
        }

        // Parent rect + text
        render_node(sg, &layout.nodes[node_idx], bg, border, font_color);

        // Parent vertical drop
        let from_y = edges[0].from_y;
        let from_x = edges[0].from_x;
        sg.set_stroke_color(Some(edge_color));
        sg.set_stroke_width(STROKE_WIDTH, None);
        sg.svg_line(from_x, from_y, from_x, connector_y, 0.0);
    } else {
        render_node(sg, &layout.nodes[node_idx], bg, border, font_color);
    }
}

fn render_node(sg: &mut SvgGraphic, node: &WbsNodeLayout, bg: &str, border: &str, font_color: &str) {
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(STROKE_WIDTH, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 0.0, 0.0, 0.0);

    // For nodes with hyperlinks or complex creole, use render_creole_text
    if node.text.contains("[[") {
        use crate::render::svg_richtext::render_creole_text;
        let text_x = node.x + PAD;
        let text_y = node.y + PAD + ASCENT;
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp, &node.text, text_x, text_y, LINE_HEIGHT,
            font_color, None, &format!(r#"font-size="{FONT_SIZE:.0}""#),
        );
        sg.push_raw(&tmp);
        return;
    }

    // Simple text: each line as a separate <text> element
    let visible = crate::model::hyperlink::extract_hyperlinks(&node.text).0;
    let lines: Vec<&str> = visible.lines().collect();
    let text_x = node.x + PAD;
    for (i, line) in lines.iter().enumerate() {
        let text_y = node.y + PAD + ASCENT + i as f64 * LINE_HEIGHT;
        let text_len = font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color(font_color);
        sg.svg_text(
            line, text_x, text_y,
            Some("sans-serif"), FONT_SIZE,
            None, None, None,
            text_len, LengthAdjust::Spacing,
            None, 0, None,
        );
    }
}

fn render_note(sg: &mut SvgGraphic, note: &WbsNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        sg.set_stroke_color(Some(NOTE_BORDER));
        sg.set_stroke_width(0.5, Some((4.0, 4.0)));
        sg.svg_line(x1, y1, x2, y2, 0.0);
    }

    // Note polygon
    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    let x2 = note.x + note.width;
    let y2 = note.y + note.height;
    sg.set_fill_color(NOTE_BG);
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(0.5, None);
    sg.svg_polygon(0.0, &[
        note.x, note.y, fold_x, note.y,
        x2, fold_y, x2, y2, note.x, y2,
    ]);

    // Fold path
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(fold_x), fmt_coord(note.y), fmt_coord(fold_x), fmt_coord(fold_y),
        fmt_coord(x2), fmt_coord(fold_y),
    ));

    let mut tmp = String::new();
    use crate::render::svg_richtext::render_creole_text;
    render_creole_text(&mut tmp, &note.text, note.x + 6.0, note.y + NOTE_FOLD + FONT_SIZE,
        LINE_HEIGHT, font_color, None, r#"font-size="13""#);
    sg.push_raw(&tmp);
}

fn render_extra_link(sg: &mut SvgGraphic, link: &WbsEdgeLayout, color: &str) {
    sg.set_stroke_color(Some(color));
    sg.set_stroke_width(1.0, None);
    sg.svg_line(link.from_x, link.from_y, link.to_x, link.to_y, 0.0);

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
        sg.set_fill_color(color);
        sg.set_stroke_color(Some(color));
        sg.set_stroke_width(1.0, None);
        sg.svg_polygon(0.0, &[
            tip_x, tip_y, left_x, left_y,
            mid_x, mid_y, right_x, right_y,
            tip_x, tip_y,
        ]);
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
