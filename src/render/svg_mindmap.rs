use super::svg::{write_svg_root_bg, write_bg_rect};
use crate::klimt::svg::{fmt_coord, SvgGraphic};
use crate::layout::mindmap::{
    MindmapEdgeLayout, MindmapLayout, MindmapNodeLayout, MindmapNoteLayout,
};
use crate::model::mindmap::MindmapDiagram;
use crate::render::svg_richtext::{count_creole_lines, render_creole_text};
use crate::style::SkinParams;
use crate::Result;

// ── Style constants ──────────────────────────────────────────────────

const FONT_SIZE: f64 = 12.0;
const LINE_HEIGHT: f64 = 16.0;
use crate::skin::rose::{BORDER_COLOR, ENTITY_BG, NOTE_BG, NOTE_BORDER, NOTE_FOLD, TEXT_COLOR};
const ROOT_FILL: &str = "#FFD700";
const BORDER_WIDTH: f64 = 0.5;
const CORNER_RADIUS: f64 = 10.0;

// ── Public entry point ──────────────────────────────────────────────

pub fn render_mindmap(
    _diagram: &MindmapDiagram,
    layout: &MindmapLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "MINDMAP", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);

    let mm_bg = skin.background_color("mindmap", ENTITY_BG);
    let mm_border = skin.border_color("mindmap", BORDER_COLOR);
    let mm_font = skin.font_color("mindmap", TEXT_COLOR);
    let edge_color = skin.arrow_color(BORDER_COLOR);

    let mut sg = SvgGraphic::new(0, 1.0);

    for edge in &layout.edges {
        render_edge(&mut sg, edge, edge_color);
    }

    for node in &layout.nodes {
        render_node(&mut sg, node, mm_bg, mm_border, mm_font);
    }

    for note in &layout.notes {
        render_note(&mut sg, note, mm_font);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_edge(sg: &mut SvgGraphic, edge: &MindmapEdgeLayout, color: &str) {
    let dx = (edge.to_x - edge.from_x) / 2.0;
    let cx1 = edge.from_x + dx;
    let cy1 = edge.from_y;
    let cx2 = edge.to_x - dx;
    let cy2 = edge.to_y;

    sg.push_raw(&format!(
        r#"<path d="M{},{} C{},{} {},{} {},{} " fill="none" style="stroke:{color};stroke-width:{BORDER_WIDTH};"/>"#,
        fmt_coord(edge.from_x), fmt_coord(edge.from_y),
        fmt_coord(cx1), fmt_coord(cy1),
        fmt_coord(cx2), fmt_coord(cy2),
        fmt_coord(edge.to_x), fmt_coord(edge.to_y),
    ));
    sg.push_raw("\n");
}

fn render_node(
    sg: &mut SvgGraphic,
    node: &MindmapNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let fill = if node.level == 1 { ROOT_FILL } else { bg };

    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(BORDER_WIDTH, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, CORNER_RADIUS, CORNER_RADIUS, 0.0);

    let total_text_height = count_creole_lines(&node.text) as f64 * LINE_HEIGHT;
    let text_start_y = node.y + (node.height - total_text_height) / 2.0 + LINE_HEIGHT * 0.75;
    let cx = node.x + node.width / 2.0;
    let outer_attrs = if node.level == 1 {
        r#"font-size="14" font-weight="bold""#
    } else {
        r#"font-size="12""#
    };
    let mut tmp = String::new();
    render_creole_text(&mut tmp, &node.text, cx, text_start_y, LINE_HEIGHT, font_color, Some("middle"), outer_attrs);
    sg.push_raw(&tmp);
}

fn render_note(sg: &mut SvgGraphic, note: &MindmapNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        sg.set_stroke_color(Some(NOTE_BORDER));
        sg.set_stroke_width(0.5, Some((4.0, 4.0)));
        sg.svg_line(x1, y1, x2, y2, 0.0);
    }

    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    let x2 = note.x + note.width;
    let y2 = note.y + note.height;

    sg.set_fill_color(NOTE_BG);
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(0.5, None);
    sg.svg_polygon(0.0, &[note.x, note.y, fold_x, note.y, x2, fold_y, x2, y2, note.x, y2]);

    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(fold_x), fmt_coord(note.y),
        fmt_coord(fold_x), fmt_coord(fold_y),
        fmt_coord(x2), fmt_coord(fold_y),
    ));
    sg.push_raw("\n");

    let mut tmp = String::new();
    render_creole_text(&mut tmp, &note.text, note.x + 6.0, note.y + NOTE_FOLD + FONT_SIZE, LINE_HEIGHT, font_color, None, r#"font-size="13""#);
    sg.push_raw(&tmp);
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::mindmap::{MindmapEdgeLayout, MindmapLayout, MindmapNodeLayout, MindmapNoteLayout};
    use crate::model::mindmap::{MindmapDiagram, MindmapNode};
    use crate::style::SkinParams;

    fn simple_layout() -> (MindmapDiagram, MindmapLayout) {
        let mut root = MindmapNode::new("Root", 1);
        root.children.push(MindmapNode::new("Child1", 2));
        root.children.push(MindmapNode::new("Child2", 2));
        let diagram = MindmapDiagram { root, notes: vec![] };
        let layout = MindmapLayout {
            nodes: vec![
                MindmapNodeLayout { text: "Root".into(), x: 20.0, y: 40.0, width: 80.0, height: 36.0, level: 1, lines: vec!["Root".into()] },
                MindmapNodeLayout { text: "Child1".into(), x: 220.0, y: 20.0, width: 70.0, height: 28.0, level: 2, lines: vec!["Child1".into()] },
                MindmapNodeLayout { text: "Child2".into(), x: 220.0, y: 70.0, width: 70.0, height: 28.0, level: 2, lines: vec!["Child2".into()] },
            ],
            edges: vec![
                MindmapEdgeLayout { from_x: 100.0, from_y: 58.0, to_x: 220.0, to_y: 34.0 },
                MindmapEdgeLayout { from_x: 100.0, from_y: 58.0, to_x: 220.0, to_y: 84.0 },
            ],
            notes: vec![], width: 320.0, height: 120.0,
        };
        (diagram, layout)
    }

    #[test] fn render_produces_valid_svg_wrapper() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains("<svg")); assert!(svg.contains("</svg>")); assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\"")); }
    #[test] fn render_contains_node_rects() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert_eq!(svg.matches("<rect").count(), 3); }
    #[test] fn render_root_has_gold_fill() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains(ROOT_FILL)); }
    #[test] fn render_child_has_default_fill() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains(ENTITY_BG)); }
    #[test] fn render_contains_text_nodes() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains("Root")); assert!(svg.contains("Child1")); assert!(svg.contains("Child2")); }
    #[test] fn render_contains_edges() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert_eq!(svg.matches("<path").count(), 2); }
    #[test] fn render_edges_use_cubic_bezier() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains("C")); }
    #[test] fn render_root_text_is_bold() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains("font-weight=\"bold\"")); }
    #[test] fn render_rounded_rects() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains("rx=\"10\"")); }

    #[test]
    fn render_xml_escapes_text() {
        let d = MindmapDiagram { root: MindmapNode::new("A & B <C>", 1), notes: vec![] };
        let l = MindmapLayout { nodes: vec![MindmapNodeLayout { text: "A & B <C>".into(), x: 10.0, y: 10.0, width: 100.0, height: 30.0, level: 1, lines: vec!["A & B <C>".into()] }], edges: vec![], notes: vec![], width: 130.0, height: 50.0 };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("A &amp; B &lt;C&gt;")); assert!(!svg.contains("A & B <C>"));
    }

    #[test]
    fn render_multiline_node() {
        let d = MindmapDiagram { root: MindmapNode::new("L1\\nL2", 1), notes: vec![] };
        let l = MindmapLayout { nodes: vec![MindmapNodeLayout { text: "L1\\nL2".into(), x: 10.0, y: 10.0, width: 80.0, height: 40.0, level: 1, lines: vec!["L1".into(), "L2".into()] }], edges: vec![], notes: vec![], width: 110.0, height: 60.0 };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert_eq!(svg.matches("<text").count(), 1); assert!(svg.contains("<tspan"));
    }

    #[test]
    fn render_empty_layout() {
        let d = MindmapDiagram { root: MindmapNode::new("Only", 1), notes: vec![] };
        let l = MindmapLayout { nodes: vec![MindmapNodeLayout { text: "Only".into(), x: 10.0, y: 10.0, width: 60.0, height: 30.0, level: 1, lines: vec!["Only".into()] }], edges: vec![], notes: vec![], width: 90.0, height: 50.0 };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg")); assert!(svg.contains("</svg>")); assert_eq!(svg.matches("<path").count(), 0);
    }

    #[test] fn render_viewbox_matches_dimensions() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains("viewBox=\"0 0 320 120\"")); assert!(svg.contains("width=\"320px\"")); assert!(svg.contains("height=\"120px\"")); }
    #[test] fn render_edge_stroke_color() { let (d, l) = simple_layout(); let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap(); assert!(svg.contains(&format!("stroke:{}", BORDER_COLOR))); }

    #[test]
    fn render_note_with_connector() {
        let d = MindmapDiagram { root: MindmapNode::new("Root", 1), notes: vec![] };
        let l = MindmapLayout {
            nodes: vec![MindmapNodeLayout { text: "Root".into(), x: 20.0, y: 30.0, width: 80.0, height: 36.0, level: 1, lines: vec!["Root".into()] }],
            edges: vec![],
            notes: vec![MindmapNoteLayout { text: "**note**".into(), x: 120.0, y: 24.0, width: 90.0, height: 42.0, connector: Some((100.0, 48.0, 120.0, 45.0)) }],
            width: 240.0, height: 100.0,
        };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon")); assert!(svg.contains("stroke-dasharray")); assert!(svg.contains("font-weight=\"bold\""));
    }
}
