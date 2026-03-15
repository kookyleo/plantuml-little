use std::fmt::Write;

use super::svg::{write_svg_root_bg, write_bg_rect};
use crate::layout::mindmap::{
    MindmapEdgeLayout, MindmapLayout, MindmapNodeLayout, MindmapNoteLayout,
};
use crate::model::mindmap::MindmapDiagram;
use crate::render::svg::fmt_coord;
use crate::render::svg_richtext::{count_creole_lines, render_creole_text};
use crate::style::SkinParams;
use crate::Result;

// ── Style constants ──────────────────────────────────────────────────

const FONT_SIZE: f64 = 12.0;
const LINE_HEIGHT: f64 = 16.0;
const NODE_FILL: &str = "#F1F1F1";
const ROOT_FILL: &str = "#FFD700";
const NODE_BORDER: &str = "#181818";
const EDGE_COLOR: &str = "#181818";
const TEXT_COLOR: &str = "#000000";
const BORDER_WIDTH: f64 = 0.5;
const CORNER_RADIUS: f64 = 10.0;
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const NOTE_FOLD: f64 = 8.0;

// ── Public entry point ──────────────────────────────────────────────

/// Render a mindmap diagram to SVG.
pub fn render_mindmap(
    _diagram: &MindmapDiagram,
    layout: &MindmapLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // SVG header
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "MINDMAP", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);

    let mm_bg = skin.background_color("mindmap", NODE_FILL);
    let mm_border = skin.border_color("mindmap", NODE_BORDER);
    let mm_font = skin.font_color("mindmap", TEXT_COLOR);
    let edge_color = skin.arrow_color(EDGE_COLOR);

    // Render edges first (below nodes)
    for edge in &layout.edges {
        render_edge(&mut buf, edge, edge_color);
    }

    // Render nodes
    for node in &layout.nodes {
        render_node(&mut buf, node, mm_bg, mm_border, mm_font);
    }

    for note in &layout.notes {
        render_note(&mut buf, note, mm_font);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── Edge rendering ──────────────────────────────────────────────────

fn render_edge(buf: &mut String, edge: &MindmapEdgeLayout, color: &str) {
    // Cubic bezier curve from parent to child
    let dx = (edge.to_x - edge.from_x) / 2.0;
    let cx1 = edge.from_x + dx;
    let cy1 = edge.from_y;
    let cx2 = edge.to_x - dx;
    let cy2 = edge.to_y;

    write!(
        buf,
        r#"<path d="M{},{} C{},{} {},{} {},{} " fill="none" style="stroke:{color};stroke-width:{BORDER_WIDTH};"/>"#,
        fmt_coord(edge.from_x), fmt_coord(edge.from_y),
        fmt_coord(cx1), fmt_coord(cy1),
        fmt_coord(cx2), fmt_coord(cy2),
        fmt_coord(edge.to_x), fmt_coord(edge.to_y),
    )
    .unwrap();
    buf.push('\n');
}

// ── Node rendering ──────────────────────────────────────────────────

fn render_node(
    buf: &mut String,
    node: &MindmapNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let fill = if node.level == 1 { ROOT_FILL } else { bg };

    // Rounded rectangle
    write!(
        buf,
        r#"<rect fill="{fill}" height="{}" rx="{rx:.0}" style="stroke:{border};stroke-width:{BORDER_WIDTH};" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
        rx = CORNER_RADIUS,
    )
    .unwrap();
    buf.push('\n');

    // Text lines centered in the node
    let total_text_height = count_creole_lines(&node.text) as f64 * LINE_HEIGHT;
    let text_start_y = node.y + (node.height - total_text_height) / 2.0 + LINE_HEIGHT * 0.75;
    let cx = node.x + node.width / 2.0;
    let outer_attrs = if node.level == 1 {
        r#"font-size="14" font-weight="bold""#
    } else {
        r#"font-size="12""#
    };
    render_creole_text(
        buf,
        &node.text,
        cx,
        text_start_y,
        LINE_HEIGHT,
        font_color,
        Some("middle"),
        outer_attrs,
    );
}

fn render_note(buf: &mut String, note: &MindmapNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        write!(
            buf,
            r#"<line style="stroke:{NOTE_BORDER};stroke-width:0.5;stroke-dasharray:4,4;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(x1), fmt_coord(x2), fmt_coord(y1), fmt_coord(y2),
        )
        .unwrap();
        buf.push('\n');
    }

    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    let x2 = note.x + note.width;
    let y2 = note.y + note.height;
    write!(
        buf,
        r#"<polygon fill="{NOTE_BG}" points="{},{} {},{} {},{} {},{} {},{}" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(note.x), fmt_coord(note.y),
        fmt_coord(fold_x), fmt_coord(note.y),
        fmt_coord(x2), fmt_coord(fold_y),
        fmt_coord(x2), fmt_coord(y2),
        fmt_coord(note.x), fmt_coord(y2),
    )
    .unwrap();
    buf.push('\n');

    write!(
        buf,
        r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(fold_x), fmt_coord(note.y),
        fmt_coord(fold_x), fmt_coord(fold_y),
        fmt_coord(x2), fmt_coord(fold_y),
    )
    .unwrap();
    buf.push('\n');

    render_creole_text(
        buf,
        &note.text,
        note.x + 6.0,
        note.y + NOTE_FOLD + FONT_SIZE,
        LINE_HEIGHT,
        font_color,
        None,
        r#"font-size="13""#,
    );
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::mindmap::{
        MindmapEdgeLayout, MindmapLayout, MindmapNodeLayout, MindmapNoteLayout,
    };
    use crate::model::mindmap::{MindmapDiagram, MindmapNode};
    use crate::style::SkinParams;

    fn simple_layout() -> (MindmapDiagram, MindmapLayout) {
        let mut root = MindmapNode::new("Root", 1);
        root.children.push(MindmapNode::new("Child1", 2));
        root.children.push(MindmapNode::new("Child2", 2));
        let diagram = MindmapDiagram {
            root,
            notes: vec![],
        };

        let layout = MindmapLayout {
            nodes: vec![
                MindmapNodeLayout {
                    text: "Root".to_string(),
                    x: 20.0,
                    y: 40.0,
                    width: 80.0,
                    height: 36.0,
                    level: 1,
                    lines: vec!["Root".to_string()],
                },
                MindmapNodeLayout {
                    text: "Child1".to_string(),
                    x: 220.0,
                    y: 20.0,
                    width: 70.0,
                    height: 28.0,
                    level: 2,
                    lines: vec!["Child1".to_string()],
                },
                MindmapNodeLayout {
                    text: "Child2".to_string(),
                    x: 220.0,
                    y: 70.0,
                    width: 70.0,
                    height: 28.0,
                    level: 2,
                    lines: vec!["Child2".to_string()],
                },
            ],
            edges: vec![
                MindmapEdgeLayout {
                    from_x: 100.0,
                    from_y: 58.0,
                    to_x: 220.0,
                    to_y: 34.0,
                },
                MindmapEdgeLayout {
                    from_x: 100.0,
                    from_y: 58.0,
                    to_x: 220.0,
                    to_y: 84.0,
                },
            ],
            notes: vec![],
            width: 320.0,
            height: 120.0,
        };

        (diagram, layout)
    }

    #[test]
    fn render_produces_valid_svg_wrapper() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"), "must contain <svg tag");
        assert!(svg.contains("</svg>"), "must close with </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn render_contains_node_rects() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        let rect_count = svg.matches("<rect").count();
        assert_eq!(rect_count, 3, "should have 3 rects for 3 nodes");
    }

    #[test]
    fn render_root_has_gold_fill() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains(ROOT_FILL), "root should have gold fill");
    }

    #[test]
    fn render_child_has_default_fill() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains(NODE_FILL), "child should have default fill");
    }

    #[test]
    fn render_contains_text_nodes() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("Root"), "should contain root text");
        assert!(svg.contains("Child1"), "should contain child1 text");
        assert!(svg.contains("Child2"), "should contain child2 text");
    }

    #[test]
    fn render_contains_edges() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        let path_count = svg.matches("<path").count();
        assert_eq!(path_count, 2, "should have 2 paths for 2 edges");
    }

    #[test]
    fn render_edges_use_cubic_bezier() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("C"), "edges should use cubic bezier (C)");
    }

    #[test]
    fn render_root_text_is_bold() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        // Find the text element containing "Root" and check bold
        assert!(
            svg.contains("font-weight=\"bold\""),
            "root text should be bold"
        );
    }

    #[test]
    fn render_rounded_rects() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("rx=\"10\""),
            "rects should have rounded corners"
        );
    }

    #[test]
    fn render_xml_escapes_text() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("A & B <C>", 1),
            notes: vec![],
        };
        let layout = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "A & B <C>".to_string(),
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 30.0,
                level: 1,
                lines: vec!["A & B <C>".to_string()],
            }],
            edges: vec![],
            notes: vec![],
            width: 130.0,
            height: 50.0,
        };
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("A &amp; B &lt;C&gt;"),
            "special chars must be XML escaped"
        );
        assert!(
            !svg.contains("A & B <C>"),
            "raw special chars should not appear"
        );
    }

    #[test]
    fn render_multiline_node() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("L1\\nL2", 1),
            notes: vec![],
        };
        let layout = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "L1\\nL2".to_string(),
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 40.0,
                level: 1,
                lines: vec!["L1".to_string(), "L2".to_string()],
            }],
            edges: vec![],
            notes: vec![],
            width: 110.0,
            height: 60.0,
        };
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        let text_count = svg.matches("<text").count();
        assert_eq!(
            text_count, 1,
            "multiline node should produce one text element"
        );
        assert!(svg.contains("<tspan"), "multiline node should use tspans");
    }

    #[test]
    fn render_empty_layout() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("Only", 1),
            notes: vec![],
        };
        let layout = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "Only".to_string(),
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 30.0,
                level: 1,
                lines: vec!["Only".to_string()],
            }],
            edges: vec![],
            notes: vec![],
            width: 90.0,
            height: 50.0,
        };
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert_eq!(svg.matches("<path").count(), 0);
    }

    #[test]
    fn render_viewbox_matches_dimensions() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("viewBox=\"0 0 320 120\""));
        assert!(svg.contains("width=\"320px\""));
        assert!(svg.contains("height=\"120px\""));
    }

    #[test]
    fn render_edge_stroke_color() {
        let (diagram, layout) = simple_layout();
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains(&format!("stroke:{}", EDGE_COLOR)),
            "edges should use edge color"
        );
    }

    #[test]
    fn render_note_with_connector() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("Root", 1),
            notes: vec![],
        };
        let layout = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "Root".to_string(),
                x: 20.0,
                y: 30.0,
                width: 80.0,
                height: 36.0,
                level: 1,
                lines: vec!["Root".to_string()],
            }],
            edges: vec![],
            notes: vec![MindmapNoteLayout {
                text: "**note**".to_string(),
                x: 120.0,
                y: 24.0,
                width: 90.0,
                height: 42.0,
                connector: Some((100.0, 48.0, 120.0, 45.0)),
            }],
            width: 240.0,
            height: 100.0,
        };
        let svg = render_mindmap(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"), "note body should be rendered");
        assert!(
            svg.contains("stroke-dasharray:4,4;"),
            "connector should be dashed"
        );
        assert!(
            svg.contains("font-weight=\"bold\""),
            "creole note text should be rendered"
        );
    }
}
