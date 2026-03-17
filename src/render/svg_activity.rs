use std::fmt::Write;

use crate::layout::activity::{
    ActivityEdgeLayout, ActivityLayout, ActivityNodeKindLayout, ActivityNodeLayout,
    NotePositionLayout, SwimlaneLayout,
};
use crate::font_metrics;
use crate::model::activity::ActivityDiagram;
use crate::render::svg::fmt_coord;
use crate::render::svg::{write_svg_root_bg, write_bg_rect};
use crate::render::svg::xml_escape;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// -- Style constants (PlantUML rose theme) ------------------------------------

const FONT_SIZE: f64 = 13.0;
const LINE_HEIGHT: f64 = 16.0;

const ACTION_BG: &str = "#F1F1F1";
const ACTION_BORDER: &str = "#181818";
const START_FILL: &str = "#222222";
const START_STROKE: &str = "#222222";
const STOP_FILL: &str = "#222222";
const STOP_STROKE: &str = "#222222";
const DIAMOND_BG: &str = "#F1F1F1";
const DIAMOND_BORDER: &str = "#181818";
const FORK_FILL: &str = "#000000";
use crate::skin::rose::{NOTE_BG, NOTE_BORDER};
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const SWIMLANE_BORDER: &str = "#000000";

// -- Public entry point -------------------------------------------------------

/// Render an activity diagram to SVG.
pub fn render_activity(
    _diagram: &ActivityDiagram,
    layout: &ActivityLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // SVG header
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "ACTIVITY", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);

    // Skin color lookups
    let act_bg = skin.background_color("activity", ACTION_BG);
    let act_border = skin.border_color("activity", ACTION_BORDER);
    let act_font = skin.font_color("activity", TEXT_FILL);
    let diamond_bg = skin.background_color("activityDiamond", DIAMOND_BG);
    let diamond_border = skin.border_color("activityDiamond", DIAMOND_BORDER);
    let swimlane_border = skin.border_color("swimlane", SWIMLANE_BORDER);
    let swimlane_font = skin.font_color("swimlane", TEXT_FILL);
    let arrow_color = skin.arrow_color(EDGE_COLOR);

    // Swimlanes (behind everything)
    for sw in &layout.swimlane_layouts {
        render_swimlane(&mut buf, sw, layout.height, swimlane_border, swimlane_font);
    }
    // Right border line for the last swimlane
    if let Some(last) = layout.swimlane_layouts.last() {
        let right_x = last.x + last.width;
        write!(
            buf,
            r#"<line style="stroke:{swimlane_border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(right_x),
            fmt_coord(right_x),
            fmt_coord(0.0),
            fmt_coord(layout.height),
        )
        .unwrap();
    }

    // Edges
    for edge in &layout.edges {
        render_edge(&mut buf, edge, arrow_color, act_font);
    }

    // Nodes (on top)
    for node in &layout.nodes {
        render_node(
            &mut buf,
            node,
            act_bg,
            act_border,
            act_font,
            diamond_bg,
            diamond_border,
            arrow_color,
        );
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// -- Node rendering -----------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_node(
    buf: &mut String,
    node: &ActivityNodeLayout,
    act_bg: &str,
    act_border: &str,
    act_font: &str,
    diamond_bg: &str,
    diamond_border: &str,
    arrow_color: &str,
) {
    match &node.kind {
        ActivityNodeKindLayout::Start => render_start(buf, node),
        ActivityNodeKindLayout::Stop => render_stop(buf, node),
        ActivityNodeKindLayout::End => render_stop(buf, node),
        ActivityNodeKindLayout::Action => render_action(buf, node, act_bg, act_border, act_font),
        ActivityNodeKindLayout::Diamond => render_diamond(buf, node, diamond_bg, diamond_border),
        ActivityNodeKindLayout::ForkBar => render_fork_bar(buf, node),
        ActivityNodeKindLayout::Note { position } => render_note(buf, node, position),
        ActivityNodeKindLayout::FloatingNote { position } => render_note(buf, node, position),
        ActivityNodeKindLayout::Detach => render_detach(buf, node, arrow_color),
    }
}

/// Start node: filled ellipse
fn render_start(buf: &mut String, node: &ActivityNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    write!(
        buf,
        r#"<ellipse cx="{}" cy="{}" fill="{START_FILL}" rx="10" ry="10" style="stroke:{START_STROKE};stroke-width:1;"/>"#,
        fmt_coord(cx),
        fmt_coord(cy),
    )
    .unwrap();
}

/// Stop / End node: double ellipse (outer ring + inner filled)
fn render_stop(buf: &mut String, node: &ActivityNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    write!(
        buf,
        r#"<ellipse cx="{}" cy="{}" fill="none" rx="11" ry="11" style="stroke:{STOP_STROKE};stroke-width:1;"/>"#,
        fmt_coord(cx),
        fmt_coord(cy),
    )
    .unwrap();
    write!(
        buf,
        r#"<ellipse cx="{}" cy="{}" fill="{STOP_FILL}" rx="6" ry="6" style="stroke:{STOP_STROKE};stroke-width:1;"/>"#,
        fmt_coord(cx),
        fmt_coord(cy),
    )
    .unwrap();
}

/// Action node: rounded rectangle with (possibly multi-line) text
fn render_action(
    buf: &mut String,
    node: &ActivityNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="12.5" ry="12.5" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height),
        fmt_coord(node.width),
        fmt_coord(node.x),
        fmt_coord(node.y),
    )
    .unwrap();

    let cx = node.x + node.width / 2.0;
    let lines: Vec<&str> = node.text.split('\n').collect();
    let total_text_height = lines.len() as f64 * LINE_HEIGHT;
    // Vertical center: baseline of first line
    let first_baseline = node.y + (node.height - total_text_height) / 2.0 + FONT_SIZE;

    render_creole_text(
        buf,
        &node.text,
        cx,
        first_baseline,
        LINE_HEIGHT,
        font_color,
        Some("middle"),
        r#"font-size="12""#,
    );
}

/// Diamond node: rotated square for if/while conditions
fn render_diamond(buf: &mut String, node: &ActivityNodeLayout, bg: &str, border: &str) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    write!(
        buf,
        r#"<polygon fill="{bg}" points="{},{} {},{} {},{} {},{}" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(cx), fmt_coord(y),
        fmt_coord(x + w), fmt_coord(cy),
        fmt_coord(cx), fmt_coord(y + h),
        fmt_coord(x), fmt_coord(cy),
    )
    .unwrap();
}

/// Fork bar: thin black horizontal rectangle
fn render_fork_bar(buf: &mut String, node: &ActivityNodeLayout) {
    write!(
        buf,
        r#"<rect fill="{FORK_FILL}" height="{}" stroke="none" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height),
        fmt_coord(node.width),
        fmt_coord(node.x),
        fmt_coord(node.y),
    )
    .unwrap();
}

/// Detach node: an X marker
fn render_detach(buf: &mut String, node: &ActivityNodeLayout, arrow_color: &str) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let r = node.width / 2.0;
    // Draw an X
    write!(
        buf,
        r#"<line style="stroke:{arrow_color};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx - r),
        fmt_coord(cx + r),
        fmt_coord(cy - r),
        fmt_coord(cy + r),
    )
    .unwrap();
    write!(
        buf,
        r#"<line style="stroke:{arrow_color};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx + r),
        fmt_coord(cx - r),
        fmt_coord(cy - r),
        fmt_coord(cy + r),
    )
    .unwrap();
}

/// Note (or floating note): path-based note shape with folded corner + text
fn render_note(buf: &mut String, node: &ActivityNodeLayout, _position: &NotePositionLayout) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let fold = 10.0;

    // Note body as <path> (matches Java PlantUML output)
    // Shape: top-left -> bottom-left -> bottom-right -> pre-fold top-right -> fold corner -> top-left
    write!(
        buf,
        r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} " fill="{NOTE_BG}" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(x), fmt_coord(y),
        fmt_coord(x), fmt_coord(y + h),
        fmt_coord(x + w), fmt_coord(y + h),
        fmt_coord(x + w), fmt_coord(y + fold),
        fmt_coord(x + w - fold), fmt_coord(y),
        fmt_coord(x), fmt_coord(y),
    )
    .unwrap();

    // Fold triangle as <path>
    write!(
        buf,
        r#"<path d="M{},{} L{},{} L{},{} L{},{} " fill="{NOTE_BG}" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(x + w - fold), fmt_coord(y),
        fmt_coord(x + w - fold), fmt_coord(y + fold),
        fmt_coord(x + w), fmt_coord(y + fold),
        fmt_coord(x + w - fold), fmt_coord(y),
    )
    .unwrap();

    let text_x = x + 6.0;
    let text_y = y + fold + FONT_SIZE;
    render_creole_text(
        buf,
        &node.text,
        text_x,
        text_y,
        LINE_HEIGHT,
        TEXT_FILL,
        None,
        r#"font-size="13""#,
    );
}

// -- Edge rendering -----------------------------------------------------------

fn render_edge(buf: &mut String, edge: &ActivityEdgeLayout, arrow_color: &str, text_color: &str) {
    if edge.points.is_empty() {
        return;
    }

    // Render line segments (without marker-end, arrow is a separate polygon)
    if edge.points.len() == 2 {
        let (x1, y1) = edge.points[0];
        let (x2, y2) = edge.points[1];
        write!(
            buf,
            r#"<line style="stroke:{arrow_color};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(x1), fmt_coord(x2), fmt_coord(y1), fmt_coord(y2),
        )
        .unwrap();
    } else {
        // Multi-segment: render each segment as a separate <line>
        for pair in edge.points.windows(2) {
            let (x1, y1) = pair[0];
            let (x2, y2) = pair[1];
            write!(
                buf,
                r#"<line style="stroke:{arrow_color};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(x1), fmt_coord(x2), fmt_coord(y1), fmt_coord(y2),
            )
            .unwrap();
        }
    }

    // Inline arrowhead polygon at the end of the edge
    if edge.points.len() >= 2 {
        let (tx, ty) = *edge.points.last().unwrap();
        let (fx, fy) = edge.points[edge.points.len() - 2];
        render_arrowhead(buf, fx, fy, tx, ty, arrow_color);
    }

    // Edge label (centered on midpoint)
    if !edge.label.is_empty() {
        let mid = edge.points.len() / 2;
        let (mx, my) = edge.points[mid];
        let escaped = xml_escape(&edge.label);
        let tl = fmt_coord(font_metrics::text_width(&edge.label, "SansSerif", FONT_SIZE, false, false));
        write!(
            buf,
            r#"<text fill="{text_color}" font-family="sans-serif" font-size="{FONT_SIZE}" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(mx), fmt_coord(my),
        )
        .unwrap();
    }
}

/// Render an inline arrowhead polygon at the tip of an edge.
/// The arrowhead points from (fx,fy) toward (tx,ty).
fn render_arrowhead(buf: &mut String, fx: f64, fy: f64, tx: f64, ty: f64, color: &str) {
    let dx = tx - fx;
    let dy = ty - fy;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }
    // Unit vector along the edge direction
    let ux = dx / len;
    let uy = dy / len;
    // Perpendicular
    let px = -uy;
    let py = ux;
    // Arrow dimensions (matches Java PlantUML: 4 wide, 10 long)
    let arrow_len = 10.0;
    let arrow_half = 4.0;
    // Four points: left, tip, right, middle-back (diamond-style)
    let lx = tx - ux * arrow_len + px * arrow_half;
    let ly = ty - uy * arrow_len + py * arrow_half;
    let rx = tx - ux * arrow_len - px * arrow_half;
    let ry = ty - uy * arrow_len - py * arrow_half;
    let mx = tx - ux * (arrow_len - 4.0);
    let my = ty - uy * (arrow_len - 4.0);
    write!(
        buf,
        r#"<polygon fill="{color}" points="{},{},{},{},{},{},{},{}" style="stroke:{color};stroke-width:1;"/>"#,
        fmt_coord(lx), fmt_coord(ly),
        fmt_coord(tx), fmt_coord(ty),
        fmt_coord(rx), fmt_coord(ry),
        fmt_coord(mx), fmt_coord(my),
    )
    .unwrap();
}

// -- Swimlane rendering -------------------------------------------------------

fn render_swimlane(
    buf: &mut String,
    sw: &SwimlaneLayout,
    total_height: f64,
    border: &str,
    font_color: &str,
) {
    // Vertical divider line
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(sw.x),
        fmt_coord(sw.x),
        fmt_coord(0.0),
        fmt_coord(total_height),
    )
    .unwrap();

    // Header label text (font-size 18 to match Java PlantUML)
    let label_x = sw.x + sw.width / 2.0;
    let escaped = xml_escape(&sw.name);
    let tl = fmt_coord(font_metrics::text_width(&sw.name, "SansSerif", 18.0, false, false));
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="18" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(label_x),
        fmt_coord(16.0),
    )
    .unwrap();
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::activity::{
        ActivityEdgeLayout, ActivityLayout, ActivityNodeKindLayout, ActivityNodeLayout,
        NotePositionLayout, SwimlaneLayout,
    };
    use crate::model::activity::ActivityDiagram;
    use crate::style::SkinParams;

    fn empty_diagram() -> ActivityDiagram {
        ActivityDiagram {
            events: vec![],
            swimlanes: vec![],
            direction: Default::default(),
        }
    }

    fn empty_layout() -> ActivityLayout {
        ActivityLayout {
            width: 200.0,
            height: 100.0,
            nodes: vec![],
            edges: vec![],
            swimlane_layouts: vec![],
        }
    }

    fn make_node(
        index: usize,
        kind: ActivityNodeKindLayout,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        text: &str,
    ) -> ActivityNodeLayout {
        ActivityNodeLayout {
            index,
            kind,
            x,
            y,
            width: w,
            height: h,
            text: text.to_string(),
        }
    }

    #[test]
    fn test_empty_diagram() {
        let diagram = empty_diagram();
        let layout = empty_layout();
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("<defs/>"));
        // No nodes or edges
        assert!(!svg.contains("<ellipse"));
        assert!(!svg.contains("<rect"));
        assert!(!svg.contains("<line "));
    }

    #[test]
    fn test_start_ellipse() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Start,
            90.0,
            10.0,
            20.0,
            20.0,
            "",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"rx="10""#), "start ellipse must have rx=10");
        assert!(svg.contains(r#"ry="10""#), "start ellipse must have ry=10");
        assert!(
            svg.contains(&format!(r#"fill="{START_FILL}""#)),
            "start ellipse must be filled"
        );
        // Only one ellipse element (start has single ellipse)
        assert_eq!(
            svg.matches("<ellipse").count(),
            1,
            "start node must produce exactly one ellipse"
        );
    }

    #[test]
    fn test_stop_ellipse() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Stop,
            90.0,
            80.0,
            22.0,
            22.0,
            "",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // Stop produces two ellipses: outer ring + inner filled
        assert_eq!(
            svg.matches("<ellipse").count(),
            2,
            "stop node must produce two ellipses"
        );
        assert!(
            svg.contains(r#"rx="11""#),
            "stop outer ring must have rx=11"
        );
        assert!(
            svg.contains(r#"rx="6""#),
            "stop inner ellipse must have rx=6"
        );
        assert!(
            svg.contains(r#"stroke-width:1;"#),
            "ellipses must have stroke-width=1"
        );
    }

    #[test]
    fn test_action_box() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Action,
            30.0,
            40.0,
            140.0,
            36.0,
            "Do something",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"rx="12.5""#),
            "action must have rounded corners rx=12.5"
        );
        assert!(
            svg.contains(r#"ry="12.5""#),
            "action must have rounded corners ry=12.5"
        );
        assert!(
            svg.contains(r#"stroke-width:0.5;"#),
            "action border must be stroke-width 0.5"
        );
        assert!(
            svg.contains(r##"fill="#F1F1F1""##),
            "action must use default theme activity_bg fill"
        );
        assert!(
            svg.contains("Do something"),
            "action text must appear in SVG"
        );
        assert!(
            svg.contains(r#"text-anchor="middle""#),
            "text must be centered"
        );
    }

    #[test]
    fn test_action_multiline_text() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Action,
            30.0,
            40.0,
            160.0,
            52.0,
            "Line one\nLine two",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<tspan"),
            "multi-line text must use <tspan> elements"
        );
        assert!(svg.contains("Line one"), "first line must appear");
        assert!(svg.contains("Line two"), "second line must appear");
        assert_eq!(
            svg.matches("<tspan").count(),
            2,
            "two lines must produce two tspan elements"
        );
    }

    #[test]
    fn test_diamond_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Diamond,
            60.0,
            50.0,
            40.0,
            40.0,
            "",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polygon"),
            "diamond must be rendered as polygon"
        );
        assert!(
            svg.contains(r##"fill="#F1F1F1""##),
            "diamond must use DIAMOND_BG"
        );
        assert!(
            svg.contains("stroke:#181818"),
            "diamond must use DIAMOND_BORDER"
        );
        // Check that 4 coordinate pairs exist in points attribute
        // cx,y r,cy cx,b l,cy  =>  80,50 100,70 80,90 60,70
        assert!(svg.contains("80,50"), "diamond top vertex");
        assert!(svg.contains("100,70"), "diamond right vertex");
        assert!(svg.contains("80,90"), "diamond bottom vertex");
        assert!(svg.contains("60,70"), "diamond left vertex");
    }

    #[test]
    fn test_fork_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::ForkBar,
            40.0,
            60.0,
            120.0,
            6.0,
            "",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(&format!(r#"fill="{FORK_FILL}""#)),
            "fork bar must be black filled"
        );
        assert!(
            svg.contains(r#"stroke="none""#),
            "fork bar must have no stroke"
        );
    }

    #[test]
    fn test_note_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Note {
                position: NotePositionLayout::Right,
            },
            10.0,
            20.0,
            100.0,
            40.0,
            "Remember this",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(&format!(r#"fill="{NOTE_BG}""#)),
            "note must use yellow background"
        );
        assert!(svg.contains("Remember this"), "note text must appear");
        // Note body and fold corner use <path> elements
        assert!(svg.contains("<path"), "note must use <path> elements");
        assert!(
            svg.contains("stroke-width:0.5;"),
            "note must have stroke-width 0.5"
        );
    }

    #[test]
    fn test_edge_with_inline_arrow() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0,
            to_index: 1,
            label: String::new(),
            points: vec![(100.0, 30.0), (100.0, 80.0)],
        });
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polygon"),
            "edge must have inline polygon arrowhead"
        );
        assert!(svg.contains("stroke:#181818"), "edge must use EDGE_COLOR");
        // Simple 2-point edge uses <line>
        assert!(svg.contains("<line "), "2-point edge must use <line>");
        // Must NOT use marker-end
        assert!(
            !svg.contains("marker-end"),
            "edges must use inline polygon, not marker-end"
        );
    }

    #[test]
    fn test_edge_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0,
            to_index: 1,
            label: "yes".to_string(),
            points: vec![(100.0, 30.0), (100.0, 80.0)],
        });
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("yes"), "edge label must appear in SVG");
    }

    #[test]
    fn test_multi_segment_edge() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0,
            to_index: 1,
            label: String::new(),
            points: vec![(50.0, 20.0), (50.0, 50.0), (100.0, 50.0), (100.0, 80.0)],
        });
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // Multi-segment edges render as individual <line> elements
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 3,
            "4-point edge must produce at least 3 line segments, got {line_count}"
        );
        assert!(
            svg.contains("<polygon"),
            "multi-segment edge must have inline polygon arrowhead"
        );
    }

    #[test]
    fn test_swimlane_headers() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout.swimlane_layouts.push(SwimlaneLayout {
            name: "Lane A".to_string(),
            x: 0.0,
            width: 200.0,
        });
        layout.swimlane_layouts.push(SwimlaneLayout {
            name: "Lane B".to_string(),
            x: 200.0,
            width: 200.0,
        });
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Lane A"), "swimlane A header must appear");
        assert!(svg.contains("Lane B"), "swimlane B header must appear");
        assert!(
            svg.contains("stroke:#000000"),
            "swimlane must have #000000 border"
        );
        assert!(
            svg.contains("stroke-width:1.5;"),
            "swimlane lines must have stroke-width 1.5"
        );
        // Check divider lines extend full height
        assert!(
            svg.contains(r#"y2="300""#),
            "swimlane dividers must extend full diagram height"
        );
    }

    #[test]
    fn test_xml_escape_in_action() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Action,
            10.0,
            10.0,
            160.0,
            36.0,
            "A & B < C",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("A &amp; B &lt; C"),
            "special characters must be XML-escaped"
        );
    }

    #[test]
    fn test_end_node_same_as_stop() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::End,
            90.0,
            80.0,
            22.0,
            22.0,
            "",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // End renders the same as Stop: two ellipses
        assert_eq!(
            svg.matches("<ellipse").count(),
            2,
            "End node must produce two ellipses like Stop"
        );
    }

    #[test]
    fn test_swimlane_text_headers() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout.swimlane_layouts.push(SwimlaneLayout {
            name: "Lane X".to_string(),
            x: 0.0,
            width: 200.0,
        });
        layout.swimlane_layouts.push(SwimlaneLayout {
            name: "Lane Y".to_string(),
            x: 200.0,
            width: 200.0,
        });
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // Headers must use font-size 18
        assert!(
            svg.contains(r#"font-size="18""#),
            "swimlane headers must use font-size 18"
        );
        // Right border of last lane must be present
        assert!(
            svg.contains(r#"x1="400""#),
            "right border of last swimlane must be present"
        );
    }

    #[test]
    fn test_cross_lane_multi_segment_edge() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        // 4-point L-shaped cross-lane edge
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0,
            to_index: 1,
            label: String::new(),
            points: vec![(100.0, 50.0), (100.0, 80.0), (300.0, 80.0), (300.0, 110.0)],
        });
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // Multi-segment edges produce individual <line> elements
        let line_count = svg.matches("<line ").count();
        assert!(
            line_count >= 3,
            "4-point cross-lane edge must produce at least 3 line segments"
        );
        assert!(
            svg.contains("<polygon"),
            "cross-lane edge must have inline polygon arrowhead"
        );
    }

    #[test]
    fn test_fmt_coord_in_output() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(
            0,
            ActivityNodeKindLayout::Start,
            90.0,
            10.0,
            20.0,
            20.0,
            "",
        ));
        let svg =
            render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // fmt_coord strips trailing zeros: 100.0 -> "100", 20.0 -> "20"
        assert!(
            svg.contains(r#"cx="100""#),
            "fmt_coord must strip trailing .0"
        );
        assert!(
            svg.contains(r#"cy="20""#),
            "fmt_coord must strip trailing .0"
        );
    }
}
