use super::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::klimt::drawable::{DrawStyle, Drawable, LineShape, PolygonShape, RectShape};
use crate::klimt::svg::{fmt_coord, SvgGraphic};
use crate::layout::mindmap::{
    DrawItem, MindmapEdgeLayout, MindmapLayout, MindmapNodeLayout, MindmapNoteLayout,
};
use crate::model::mindmap::MindmapDiagram;
use crate::render::svg_richtext::render_creole_text;
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
    let svg_w = ensure_visible_int(layout.width) as f64;
    let svg_h = ensure_visible_int(layout.height) as f64;
    write_svg_root_bg(&mut buf, svg_w, svg_h, "MINDMAP", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);

    // Node style: colors, border, corner radius
    let has_node_bg_style = skin.get("node.backgroundcolor").is_some();
    let node_bg = skin
        .get("node.backgroundcolor")
        .unwrap_or_else(|| skin.background_color("mindmap", ENTITY_BG));
    // Root uses gold by default, but style override applies to all nodes
    let root_bg = if has_node_bg_style {
        node_bg
    } else {
        ROOT_FILL
    };
    let child_bg = node_bg;
    let node_border = skin
        .get("node.linecolor")
        .unwrap_or_else(|| skin.border_color("mindmap", BORDER_COLOR));
    let node_border_width: f64 = skin
        .get("node.linethickness")
        .and_then(|s| s.parse().ok())
        .unwrap_or(BORDER_WIDTH);
    let node_corner: f64 = skin
        .get("node.roundcorner")
        .and_then(|s| s.parse::<f64>().ok())
        .map(|v| v / 2.0) // Java: RoundCorner N → rx/ry = N/2
        .unwrap_or(CORNER_RADIUS);
    let mm_font = skin.font_color("mindmap", TEXT_COLOR);
    // Java: edge color comes from arrow style (root.element.mindmapDiagram.arrow),
    // defaulting to #181818. It does NOT inherit node.LineColor.
    let edge_color = skin
        .get("arrow.linecolor")
        .unwrap_or_else(|| skin.arrow_color(BORDER_COLOR));
    // Java: edge stroke width comes from arrow style, defaulting to 1.
    let edge_stroke_width: f64 = skin
        .get("arrow.linethickness")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Render in Java draw order: interleaved nodes and edges
    if layout.draw_order.is_empty() {
        // Fallback for test layouts without draw_order
        for node in &layout.nodes {
            let bg = if node.level == 1 { root_bg } else { child_bg };
            render_node_styled(
                &mut sg,
                node,
                bg,
                node_border,
                mm_font,
                node_border_width,
                node_corner,
            );
        }
        for edge in &layout.edges {
            render_edge_styled(&mut sg, edge, edge_color, edge_stroke_width);
        }
    } else {
        for item in &layout.draw_order {
            match item {
                DrawItem::Node(idx) => {
                    let node = &layout.nodes[*idx];
                    let bg = if node.level == 1 { root_bg } else { child_bg };
                    render_node_styled(
                        &mut sg,
                        node,
                        bg,
                        node_border,
                        mm_font,
                        node_border_width,
                        node_corner,
                    );
                }
                DrawItem::Edge(idx) => {
                    let edge = &layout.edges[*idx];
                    render_edge_styled(&mut sg, edge, edge_color, edge_stroke_width);
                }
            }
        }
    }

    for note in &layout.notes {
        render_note(&mut sg, note, mm_font);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

#[allow(dead_code)] // convenience wrapper for default stroke
fn render_edge(sg: &mut SvgGraphic, edge: &MindmapEdgeLayout, color: &str) {
    render_edge_styled(sg, edge, color, BORDER_WIDTH);
}

fn render_edge_styled(sg: &mut SvgGraphic, edge: &MindmapEdgeLayout, color: &str, stroke_w: f64) {
    // Java FtileBox draws: M start L start+10 C mid,... mid,... end-10 L end
    // A horizontal line segment, then a cubic bezier, then another horizontal line.
    let line_seg = 10.0;
    let l1_x = edge.from_x + line_seg;
    let l2_x = edge.to_x - line_seg;
    let cx1 = (l1_x + l2_x) / 2.0;
    let cy1 = edge.from_y;
    let cx2 = (l1_x + l2_x) / 2.0;
    let cy2 = edge.to_y;

    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} C{},{} {},{} {},{} L{},{}" fill="none" style="stroke:{color};stroke-width:{};"/>"#,
        fmt_coord(edge.from_x), fmt_coord(edge.from_y),
        fmt_coord(l1_x), fmt_coord(edge.from_y),
        fmt_coord(cx1), fmt_coord(cy1),
        fmt_coord(cx2), fmt_coord(cy2),
        fmt_coord(l2_x), fmt_coord(edge.to_y),
        fmt_coord(edge.to_x), fmt_coord(edge.to_y),
        fmt_coord(stroke_w),
    ));
}

#[allow(dead_code)] // convenience wrapper for default style
fn render_node(
    sg: &mut SvgGraphic,
    node: &MindmapNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    render_node_styled(
        sg,
        node,
        bg,
        border,
        font_color,
        BORDER_WIDTH,
        CORNER_RADIUS,
    );
}

fn render_node_styled(
    sg: &mut SvgGraphic,
    node: &MindmapNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
    stroke_w: f64,
    corner_r: f64,
) {
    RectShape {
        x: node.x,
        y: node.y,
        w: node.width,
        h: node.height,
        rx: corner_r,
        ry: corner_r,
    }
    .draw(sg, &DrawStyle::filled(bg, border, stroke_w));

    // Java renders each line individually with explicit x positioning.
    // Lines are centered using their full width (including leading whitespace),
    // but rendered with trimmed text and adjusted x.
    let font_size = 14.0;
    let outer_attrs = r#"font-size="14""#;
    // Split on \n, preserving whitespace (Java behavior for width calculation)
    let raw_lines: Vec<String> = node
        .text
        .split("\\n")
        .flat_map(|s| s.split(crate::NEWLINE_CHAR))
        .map(|s| s.to_string())
        .collect();
    let n_lines = raw_lines.len().max(1);
    let text_h = crate::font_metrics::line_height("SansSerif", font_size, false, false);
    let total_text_height = n_lines as f64 * text_h;
    let ascent = crate::font_metrics::ascent("SansSerif", font_size, false, false);
    let text_start_y = node.y + (node.height - total_text_height) / 2.0 + ascent;

    for (idx, raw_line) in raw_lines.iter().enumerate() {
        // Center using full width (including leading spaces)
        let full_w =
            crate::font_metrics::text_width(raw_line, "SansSerif", font_size, false, false);
        let line_x = node.x + (node.width - full_w) / 2.0;
        // Render trimmed text at adjusted x (skip leading/trailing whitespace visually)
        let trimmed = raw_line.trim();
        let trimmed_w =
            crate::font_metrics::text_width(trimmed, "SansSerif", font_size, false, false);
        let leading_space_w = if raw_line.starts_with(' ') {
            let left_trimmed = raw_line.trim_start();
            full_w
                - crate::font_metrics::text_width(
                    left_trimmed,
                    "SansSerif",
                    font_size,
                    false,
                    false,
                )
        } else {
            0.0
        };
        let _ = trimmed_w;
        let render_x = line_x + leading_space_w;
        let line_y = text_start_y + idx as f64 * text_h;
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            trimmed,
            render_x,
            line_y,
            text_h,
            font_color,
            None,
            outer_attrs,
        );
        sg.push_raw(&tmp);
    }
}

fn render_note(sg: &mut SvgGraphic, note: &MindmapNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        LineShape { x1, y1, x2, y2 }.draw(
            sg,
            &DrawStyle {
                fill: None,
                stroke: Some(NOTE_BORDER.into()),
                stroke_width: 0.5,
                dash_array: Some((4.0, 4.0)),
                delta_shadow: 0.0,
            },
        );
    }

    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    let x2 = note.x + note.width;
    let y2 = note.y + note.height;

    PolygonShape {
        points: vec![
            note.x, note.y, fold_x, note.y, x2, fold_y, x2, y2, note.x, y2,
        ],
    }
    .draw(sg, &DrawStyle::filled(NOTE_BG, NOTE_BORDER, 0.5));

    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(fold_x), fmt_coord(note.y),
        fmt_coord(fold_x), fmt_coord(fold_y),
        fmt_coord(x2), fmt_coord(fold_y),
    ));
    sg.push_raw("\n");

    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        note.x + 6.0,
        note.y + NOTE_FOLD + FONT_SIZE,
        LINE_HEIGHT,
        font_color,
        None,
        r#"font-size="13""#,
    );
    sg.push_raw(&tmp);
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
            caption: None,
        };
        let layout = MindmapLayout {
            nodes: vec![
                MindmapNodeLayout {
                    text: "Root".into(),
                    x: 20.0,
                    y: 40.0,
                    width: 80.0,
                    height: 36.0,
                    level: 1,
                    lines: vec!["Root".into()],
                },
                MindmapNodeLayout {
                    text: "Child1".into(),
                    x: 220.0,
                    y: 20.0,
                    width: 70.0,
                    height: 28.0,
                    level: 2,
                    lines: vec!["Child1".into()],
                },
                MindmapNodeLayout {
                    text: "Child2".into(),
                    x: 220.0,
                    y: 70.0,
                    width: 70.0,
                    height: 28.0,
                    level: 2,
                    lines: vec!["Child2".into()],
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
            caption: None,
            raw_body_dim: None,
            draw_order: vec![],
        };
        (diagram, layout)
    }

    #[test]
    fn render_produces_valid_svg_wrapper() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }
    #[test]
    fn render_contains_node_rects() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert_eq!(svg.matches("<rect").count(), 3);
    }
    #[test]
    fn render_root_has_gold_fill() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(ROOT_FILL));
    }
    #[test]
    fn render_child_has_default_fill() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(ENTITY_BG));
    }
    #[test]
    fn render_contains_text_nodes() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("Root"));
        assert!(svg.contains("Child1"));
        assert!(svg.contains("Child2"));
    }
    #[test]
    fn render_contains_edges() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert_eq!(svg.matches("<path").count(), 2);
    }
    #[test]
    fn render_edges_use_cubic_bezier() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("C"));
    }
    #[test]
    fn render_root_text_not_bold() {
        // Java does not bold root nodes by default
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(!svg.contains("font-weight=\"700\""));
    }
    #[test]
    fn render_rounded_rects() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("rx=\"10\""));
    }

    #[test]
    fn render_xml_escapes_text() {
        let d = MindmapDiagram {
            root: MindmapNode::new("A & B <C>", 1),
            notes: vec![],
            caption: None,
        };
        let l = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "A & B <C>".into(),
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 30.0,
                level: 1,
                lines: vec!["A & B <C>".into()],
            }],
            edges: vec![],
            notes: vec![],
            width: 130.0,
            height: 50.0,
            caption: None,
            raw_body_dim: None,
            draw_order: vec![],
        };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("A &amp; B &lt;C&gt;"));
        assert!(!svg.contains("A & B <C>"));
    }

    #[test]
    fn render_multiline_node() {
        let d = MindmapDiagram {
            root: MindmapNode::new("L1\\nL2", 1),
            notes: vec![],
            caption: None,
        };
        let l = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "L1\\nL2".into(),
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 40.0,
                level: 1,
                lines: vec!["L1".into(), "L2".into()],
            }],
            edges: vec![],
            notes: vec![],
            width: 110.0,
            height: 60.0,
            caption: None,
            raw_body_dim: None,
            draw_order: vec![],
        };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        // Java renders each line as separate <text> element
        assert_eq!(svg.matches("<text").count(), 2);
        assert!(!svg.contains("<tspan"));
    }

    #[test]
    fn render_empty_layout() {
        let d = MindmapDiagram {
            root: MindmapNode::new("Only", 1),
            notes: vec![],
            caption: None,
        };
        let l = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "Only".into(),
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 30.0,
                level: 1,
                lines: vec!["Only".into()],
            }],
            edges: vec![],
            notes: vec![],
            width: 90.0,
            height: 50.0,
            caption: None,
            raw_body_dim: None,
            draw_order: vec![],
        };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert_eq!(svg.matches("<path").count(), 0);
    }

    #[test]
    fn render_viewbox_matches_dimensions() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("viewBox=\"0 0 321 121\""),
            "viewBox uses ensure_visible_int"
        );
        assert!(
            svg.contains("width=\"321px\""),
            "width uses ensure_visible_int"
        );
        assert!(
            svg.contains("height=\"121px\""),
            "height uses ensure_visible_int"
        );
    }
    #[test]
    fn render_edge_stroke_color() {
        let (d, l) = simple_layout();
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(&format!("stroke:{}", BORDER_COLOR)));
    }

    #[test]
    fn render_note_with_connector() {
        let d = MindmapDiagram {
            root: MindmapNode::new("Root", 1),
            notes: vec![],
            caption: None,
        };
        let l = MindmapLayout {
            nodes: vec![MindmapNodeLayout {
                text: "Root".into(),
                x: 20.0,
                y: 30.0,
                width: 80.0,
                height: 36.0,
                level: 1,
                lines: vec!["Root".into()],
            }],
            edges: vec![],
            notes: vec![MindmapNoteLayout {
                text: "**note**".into(),
                x: 120.0,
                y: 24.0,
                width: 90.0,
                height: 42.0,
                connector: Some((100.0, 48.0, 120.0, 45.0)),
            }],
            width: 240.0,
            height: 100.0,
            caption: None,
            raw_body_dim: None,
            draw_order: vec![],
        };
        let svg = render_mindmap(&d, &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.contains("font-weight"));
    }
}
