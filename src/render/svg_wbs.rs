use std::fmt::Write;

use super::svg::write_svg_root;
use crate::layout::wbs::{WbsEdgeLayout, WbsLayout, WbsNodeLayout, WbsNoteLayout};
use crate::model::wbs::WbsDiagram;
use crate::render::svg::fmt_coord;
use crate::render::svg_richtext::{count_creole_lines, render_creole_text};
use crate::style::SkinParams;
use crate::Result;

// ── Style constants ─────────────────────────────────────────────────

const FONT_SIZE: f64 = 14.0;
const LINE_HEIGHT: f64 = 16.0;
const NODE_BG: &str = "#F1F1F1";
const ROOT_BG: &str = "#FFD700";
const NODE_BORDER: &str = "#181818";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const RX: f64 = 4.0;
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const NOTE_FOLD: f64 = 8.0;

// ── XML escaping (test helper) ──────────────────────────────────────

#[cfg(test)]
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

// ── Public entry point ──────────────────────────────────────────────

/// Render a WBS diagram to SVG.
pub fn render_wbs(_wd: &WbsDiagram, layout: &WbsLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // SVG header
    write_svg_root(&mut buf, layout.width, layout.height, "WBS");
    buf.push_str("<defs/><g>");

    let wbs_bg = skin.background_color("wbs", NODE_BG);
    let wbs_border = skin.border_color("wbs", NODE_BORDER);
    let wbs_font = skin.font_color("wbs", TEXT_FILL);
    let edge_color = skin.arrow_color(EDGE_COLOR);

    // Edges (parent-child connections)
    for edge in &layout.edges {
        render_edge(&mut buf, edge, edge_color);
    }

    // Extra links (alias-to-alias)
    for link in &layout.extra_links {
        render_extra_link(&mut buf, link, edge_color);
    }

    // Nodes (rendered after edges so they appear on top)
    for node in &layout.nodes {
        render_node(&mut buf, node, wbs_bg, wbs_border, wbs_font);
    }

    for note in &layout.notes {
        render_note(&mut buf, note, wbs_font);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── Node rendering ──────────────────────────────────────────────────

fn render_node(buf: &mut String, node: &WbsNodeLayout, bg: &str, border: &str, font_color: &str) {
    let fill = if node.level == 1 { ROOT_BG } else { bg };

    // Rectangle
    write!(
        buf,
        r#"<rect fill="{fill}" height="{}" rx="{RX}" ry="{RX}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    )
    .unwrap();
    buf.push('\n');

    // Text
    let cx = node.x + node.width / 2.0;

    let line_count = count_creole_lines(&node.text);
    let total_text_h = line_count as f64 * LINE_HEIGHT;
    let start_y = if line_count == 1 {
        node.y + node.height / 2.0 + FONT_SIZE * 0.35
    } else {
        node.y + (node.height - total_text_h) / 2.0 + FONT_SIZE
    };
    render_creole_text(
        buf,
        &node.text,
        cx,
        start_y,
        LINE_HEIGHT,
        font_color,
        Some("middle"),
        r#"font-size="14""#,
    );
}

fn render_note(buf: &mut String, note: &WbsNoteLayout, font_color: &str) {
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

// ── Edge rendering ──────────────────────────────────────────────────

/// Parent-child edge: vertical line down from parent, horizontal to child,
/// then vertical down to child top.
fn render_edge(buf: &mut String, edge: &WbsEdgeLayout, color: &str) {
    let mid_y = (edge.from_y + edge.to_y) / 2.0;

    write!(
        buf,
        r#"<path d="M{},{} L{},{} L{},{} L{},{} " fill="none" style="stroke:{color};stroke-width:0.5;"/>"#,
        fmt_coord(edge.from_x), fmt_coord(edge.from_y),
        fmt_coord(edge.from_x), fmt_coord(mid_y),
        fmt_coord(edge.to_x), fmt_coord(mid_y),
        fmt_coord(edge.to_x), fmt_coord(edge.to_y),
    )
    .unwrap();
    buf.push('\n');
}

/// Extra link between aliased nodes: dashed line.
fn render_extra_link(buf: &mut String, link: &WbsEdgeLayout, color: &str) {
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:0.5;stroke-dasharray:4,4;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(link.from_x), fmt_coord(link.to_x), fmt_coord(link.from_y), fmt_coord(link.to_y),
    )
    .unwrap();
    buf.push('\n');
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::wbs::{WbsEdgeLayout, WbsLayout, WbsNodeLayout, WbsNoteLayout};
    use crate::model::wbs::{WbsDiagram, WbsDirection, WbsNode};
    use crate::style::SkinParams;

    fn empty_wbs() -> WbsDiagram {
        WbsDiagram {
            root: WbsNode {
                text: "R".to_string(),
                children: vec![],
                direction: WbsDirection::Default,
                alias: None,
                level: 1,
            },
            links: vec![],
            notes: vec![],
        }
    }

    fn empty_layout() -> WbsLayout {
        WbsLayout {
            nodes: vec![],
            edges: vec![],
            extra_links: vec![],
            notes: vec![],
            width: 200.0,
            height: 100.0,
        }
    }

    fn make_node(text: &str, level: usize, x: f64, y: f64, w: f64, h: f64) -> WbsNodeLayout {
        WbsNodeLayout {
            text: text.to_string(),
            alias: None,
            x,
            y,
            width: w,
            height: h,
            level,
        }
    }

    #[test]
    fn test_svg_header_footer() {
        let wd = empty_wbs();
        let layout = empty_layout();
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn test_svg_dimensions() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("width=\"400px\""), "width must match layout");
        assert!(svg.contains("height=\"300px\""), "height must match layout");
        assert!(
            svg.contains("viewBox=\"0 0 400 300\""),
            "viewBox must match"
        );
    }

    #[test]
    fn test_root_node_gold() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_node("Root", 1, 50.0, 10.0, 80.0, 30.0));
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains(&format!(r#"fill="{ROOT_BG}""#)),
            "root node must use gold fill"
        );
        assert!(svg.contains("Root"), "root text must appear");
    }

    #[test]
    fn test_non_root_node_fill() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_node("Child", 2, 50.0, 80.0, 80.0, 30.0));
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains(&format!(r#"fill="{NODE_BG}""#)),
            "non-root node must use default fill"
        );
    }

    #[test]
    fn test_node_border() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout.nodes.push(make_node("N", 1, 10.0, 10.0, 40.0, 24.0));
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains(&format!("stroke:{NODE_BORDER}")),
            "node must have border stroke"
        );
        assert!(
            svg.contains(&format!(r#"rx="{RX}""#)),
            "node must have rounded corners"
        );
    }

    #[test]
    fn test_single_line_text() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_node("Hello", 1, 10.0, 10.0, 80.0, 30.0));
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("Hello"), "text must appear");
        assert!(
            svg.contains(r#"text-anchor="middle""#),
            "text must be centered"
        );
        assert!(
            !svg.contains("<tspan"),
            "single-line text should not use tspan"
        );
    }

    #[test]
    fn test_multiline_text() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_node("Line 1\nLine 2", 2, 10.0, 10.0, 100.0, 50.0));
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<tspan"), "multiline must use tspan");
        assert!(svg.contains("Line 1"), "first line must appear");
        assert!(svg.contains("Line 2"), "second line must appear");
        assert_eq!(
            svg.matches("<tspan").count(),
            2,
            "two lines must produce two tspan elements"
        );
    }

    #[test]
    fn test_edge_rendering() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout.edges.push(WbsEdgeLayout {
            from_x: 100.0,
            from_y: 40.0,
            to_x: 80.0,
            to_y: 80.0,
        });
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<path"), "edge must use <path>");
        assert!(
            svg.contains(&format!("stroke:{EDGE_COLOR}")),
            "edge must use edge color"
        );
        assert!(
            svg.contains(r#"fill="none""#),
            "edge path must have no fill"
        );
    }

    #[test]
    fn test_extra_link_dashed() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout.extra_links.push(WbsEdgeLayout {
            from_x: 50.0,
            from_y: 50.0,
            to_x: 150.0,
            to_y: 50.0,
        });
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<line"), "extra link must use <line>");
        assert!(
            svg.contains("stroke-dasharray"),
            "extra link must be dashed"
        );
    }

    #[test]
    fn test_xml_escaping() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_node("A & B < C", 1, 10.0, 10.0, 120.0, 30.0));
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("A &amp; B &lt; C"), "text must be XML-escaped");
    }

    #[test]
    fn test_xml_escape_fn() {
        assert_eq!(xml_escape("A & B"), "A &amp; B");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape(r#"a"b"#), "a&quot;b");
        assert_eq!(xml_escape("plain"), "plain");
    }

    #[test]
    fn test_empty_layout() {
        let wd = empty_wbs();
        let layout = empty_layout();
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(!svg.contains("<rect"), "empty layout has no rects");
        assert!(!svg.contains("<path"), "empty layout has no paths");
        assert!(!svg.contains("<line"), "empty layout has no lines");
    }

    #[test]
    fn test_multiple_nodes() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_node("Root", 1, 80.0, 10.0, 60.0, 28.0));
        layout.nodes.push(make_node("A", 2, 30.0, 80.0, 50.0, 28.0));
        layout
            .nodes
            .push(make_node("B", 2, 120.0, 80.0, 50.0, 28.0));
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert_eq!(
            svg.matches("<rect").count(),
            3,
            "three nodes should produce three rects"
        );
    }

    #[test]
    fn test_font_attributes() {
        let wd = empty_wbs();
        let layout = empty_layout();
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("contentStyleType=\"text/css\""),
            "must have contentStyleType attribute"
        );
        assert!(
            svg.contains("zoomAndPan=\"magnify\""),
            "must have zoomAndPan attribute"
        );
    }

    #[test]
    fn test_full_render_pipeline() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout.width = 300.0;
        layout.height = 200.0;
        layout
            .nodes
            .push(make_node("Root", 1, 120.0, 10.0, 60.0, 28.0));
        layout
            .nodes
            .push(make_node("Left", 2, 40.0, 80.0, 60.0, 28.0));
        layout
            .nodes
            .push(make_node("Right", 2, 200.0, 80.0, 60.0, 28.0));
        layout.edges.push(WbsEdgeLayout {
            from_x: 150.0,
            from_y: 38.0,
            to_x: 70.0,
            to_y: 80.0,
        });
        layout.edges.push(WbsEdgeLayout {
            from_x: 150.0,
            from_y: 38.0,
            to_x: 230.0,
            to_y: 80.0,
        });

        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();

        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.contains("</svg>"), "SVG must end with </svg>");
        assert_eq!(svg.matches("<rect").count(), 3, "3 rects expected");
        assert_eq!(svg.matches("<path").count(), 2, "2 edges expected");
        assert!(svg.contains("Root"), "root text expected");
        assert!(svg.contains("Left"), "left child text expected");
        assert!(svg.contains("Right"), "right child text expected");
    }

    #[test]
    fn test_note_rendering() {
        let wd = empty_wbs();
        let mut layout = empty_layout();
        layout.notes.push(WbsNoteLayout {
            text: "**note**".to_string(),
            x: 120.0,
            y: 20.0,
            width: 90.0,
            height: 40.0,
            connector: Some((90.0, 35.0, 120.0, 40.0)),
        });
        let svg = render_wbs(&wd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"), "note body must be rendered");
        assert!(svg.contains("stroke-dasharray:4,4;"));
        assert!(
            svg.contains("font-weight=\"bold\""),
            "creole note text should be rendered"
        );
    }
}
