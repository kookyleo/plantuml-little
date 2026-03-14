use std::fmt::Write;

use crate::layout::activity::{
    ActivityEdgeLayout, ActivityLayout, ActivityNodeKindLayout, ActivityNodeLayout,
    NotePositionLayout, SwimlaneLayout,
};
use crate::model::activity::ActivityDiagram;
use crate::render::svg::xml_escape;
use crate::render::svg::write_svg_root;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// ── Style constants (PlantUML rose theme) ───────────────────────────

const FONT_SIZE: f64 = 13.0;
const LINE_HEIGHT: f64 = 16.0;

const ACTION_BG: &str = "#F1F1F1";
const ACTION_BORDER: &str = "#181818";
const START_FILL: &str = "#000000";
const STOP_FILL: &str = "#000000";
const STOP_OUTER: &str = "#000000";
const DIAMOND_BG: &str = "#F1F1F1";
const DIAMOND_BORDER: &str = "#181818";
const FORK_FILL: &str = "#000000";
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const SWIMLANE_BORDER: &str = "#181818";
const SWIMLANE_HEADER_BG: &str = "#F1F1F1";

// ── Public entry point ──────────────────────────────────────────────

/// Render an activity diagram to SVG.
pub fn render_activity(
    _diagram: &ActivityDiagram,
    layout: &ActivityLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // SVG header
    write_svg_root(&mut buf, layout.width, layout.height, "ACTIVITY");
    buf.push_str("<defs/><g>");

    // Skin color lookups
    let act_bg = skin.background_color("activity", ACTION_BG);
    let act_border = skin.border_color("activity", ACTION_BORDER);
    let act_font = skin.font_color("activity", TEXT_FILL);
    let diamond_bg = skin.background_color("activityDiamond", DIAMOND_BG);
    let diamond_border = skin.border_color("activityDiamond", DIAMOND_BORDER);
    let swimlane_border = skin.border_color("swimlane", SWIMLANE_BORDER);
    let swimlane_header_bg = skin.background_color("swimlane", SWIMLANE_HEADER_BG);
    let swimlane_font = skin.font_color("swimlane", TEXT_FILL);
    let arrow_color = skin.arrow_color(EDGE_COLOR);

    // Defs: arrow marker
    write_defs(&mut buf, arrow_color);

    // Swimlanes (behind everything)
    for sw in &layout.swimlane_layouts {
        render_swimlane(
            &mut buf,
            sw,
            layout.height,
            swimlane_border,
            swimlane_header_bg,
            swimlane_font,
        );
    }
    // Right border line for the last swimlane
    if let Some(last) = layout.swimlane_layouts.last() {
        let right_x = last.x + last.width;
        write!(
            buf,
            r#"<line style="stroke:{swimlane_border};stroke-width:1;" x1="{rx:.1}" x2="{rx:.1}" y1="0" y2="{h:.1}"/>"#,
            rx = right_x,
            h = layout.height,
        )
        .unwrap();
        buf.push('\n');
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

// ── Defs ────────────────────────────────────────────────────────────

fn write_defs(buf: &mut String, arrow_color: &str) {
    buf.push_str("<defs>\n");
    write!(
        buf,
        concat!(
            r#"<marker id="act-arrow" viewBox="0 0 10 10" refX="10" refY="5""#,
            r#" markerWidth="8" markerHeight="8" orient="auto-start-reverse">"#,
            r#"<path d="M 0 0 L 10 5 L 0 10 Z" fill="{}" stroke="none"/>"#,
            r#"</marker>"#,
        ),
        arrow_color,
    )
    .unwrap();
    buf.push('\n');
    buf.push_str("</defs>\n");
}

// ── Node rendering ──────────────────────────────────────────────────

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

/// Start node: filled black circle
fn render_start(buf: &mut String, node: &ActivityNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="{START_FILL}" r="10" style="stroke:{START_FILL};"/>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Stop / End node: double circle (outer ring + inner filled)
fn render_stop(buf: &mut String, node: &ActivityNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="none" r="11" style="stroke:{STOP_OUTER};stroke-width:2;"/>"#,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="{STOP_FILL}" r="7" stroke="none"/>"#,
    )
    .unwrap();
    buf.push('\n');
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
        r#"<rect fill="{bg}" height="{h:.1}" rx="10" ry="10" style="stroke:{border};stroke-width:1.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

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
        r#"<polygon fill="{bg}" points="{cx:.1},{y:.1} {r:.1},{cy:.1} {cx:.1},{b:.1} {l:.1},{cy:.1}" style="stroke:{border};stroke-width:1.5;"/>"#,
        r = x + w,
        b = y + h,
        l = x,
    )
    .unwrap();
    buf.push('\n');
}

/// Fork bar: thin black horizontal rectangle
fn render_fork_bar(buf: &mut String, node: &ActivityNodeLayout) {
    write!(
        buf,
        r#"<rect fill="{FORK_FILL}" height="{h:.1}" stroke="none" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');
}

/// Detach node: an X marker
fn render_detach(buf: &mut String, node: &ActivityNodeLayout, arrow_color: &str) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let r = node.width / 2.0;
    // Draw an X
    write!(
        buf,
        r#"<line style="stroke:{arrow_color};stroke-width:2;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        x1 = cx - r, y1 = cy - r, x2 = cx + r, y2 = cy + r,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<line style="stroke:{arrow_color};stroke-width:2;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        x1 = cx + r, y1 = cy - r, x2 = cx - r, y2 = cy + r,
    )
    .unwrap();
    buf.push('\n');
}

/// Note (or floating note): yellow rectangle with folded corner + text
fn render_note(buf: &mut String, node: &ActivityNodeLayout, _position: &NotePositionLayout) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let fold = 8.0;

    // Note body polygon (top-left, pre-fold top-right, fold corner, bottom-right, bottom-left)
    write!(
        buf,
        r#"<polygon fill="{NOTE_BG}" points="{x:.1},{y:.1} {xf:.1},{y:.1} {xw:.1},{yf:.1} {xw:.1},{yh:.1} {x:.1},{yh:.1}" style="stroke:{NOTE_BORDER};"/>"#,
        xf = x + w - fold,
        xw = x + w,
        yf = y + fold,
        yh = y + h,
    )
    .unwrap();
    buf.push('\n');

    // Fold lines (vertical + horizontal)
    write!(
        buf,
        r#"<line style="stroke:{NOTE_BORDER};" x1="{xf:.1}" x2="{xf:.1}" y1="{y:.1}" y2="{yf:.1}"/>"#,
        xf = x + w - fold,
        yf = y + fold,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<line style="stroke:{NOTE_BORDER};" x1="{xf:.1}" x2="{xw:.1}" y1="{yf:.1}" y2="{yf:.1}"/>"#,
        xf = x + w - fold,
        yf = y + fold,
        xw = x + w,
    )
    .unwrap();
    buf.push('\n');

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

// ── Edge rendering ──────────────────────────────────────────────────

fn render_edge(buf: &mut String, edge: &ActivityEdgeLayout, arrow_color: &str, text_color: &str) {
    if edge.points.is_empty() {
        return;
    }

    if edge.points.len() == 2 {
        // Simple straight line
        let (x1, y1) = edge.points[0];
        let (x2, y2) = edge.points[1];
        write!(
            buf,
            r#"<line marker-end="url(#act-arrow)" style="stroke:{arrow_color};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        )
        .unwrap();
        buf.push('\n');
    } else {
        // Multi-segment polyline
        let points_str: String = edge
            .points
            .iter()
            .map(|(px, py)| format!("{px:.1},{py:.1}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            buf,
            r#"<polyline fill="none" marker-end="url(#act-arrow)" points="{points_str}" style="stroke:{arrow_color};stroke-width:1;"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    // Edge label (centered on midpoint)
    if !edge.label.is_empty() {
        let mid = edge.points.len() / 2;
        let (mx, my) = edge.points[mid];
        let escaped = xml_escape(&edge.label);
        write!(
            buf,
            r#"<text fill="{text_color}" font-family="sans-serif" font-size="{FONT_SIZE}" text-anchor="middle" x="{mx:.1}" y="{my:.1}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Swimlane rendering ──────────────────────────────────────────────

fn render_swimlane(
    buf: &mut String,
    sw: &SwimlaneLayout,
    total_height: f64,
    border: &str,
    header_bg: &str,
    font_color: &str,
) {
    // Vertical divider line
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1;" x1="{x:.1}" x2="{x:.1}" y1="0" y2="{h:.1}"/>"#,
        x = sw.x,
        h = total_height,
    )
    .unwrap();
    buf.push('\n');

    // Header background
    write!(
        buf,
        r#"<rect fill="{header_bg}" height="24" style="stroke:{border};" width="{w:.1}" x="{x:.1}" y="0"/>"#,
        x = sw.x,
        w = sw.width,
    )
    .unwrap();
    buf.push('\n');

    // Header label (bold)
    let label_x = sw.x + sw.width / 2.0;
    let escaped = xml_escape(&sw.name);
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" text-anchor="middle" x="{label_x:.1}" y="16">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

// ── Tests ───────────────────────────────────────────────────────────

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
        assert!(svg.contains("act-arrow"));
        // No nodes or edges
        assert!(!svg.contains("<circle"));
        assert!(!svg.contains("<rect"));
        assert!(!svg.contains("<line x1"));
    }

    #[test]
    fn test_start_circle() {
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
        assert!(svg.contains(r#"r="10""#), "start circle must have r=10");
        assert!(
            svg.contains(&format!(r#"fill="{START_FILL}""#)),
            "start circle must be black filled"
        );
        // Only one circle element (start has single circle)
        assert_eq!(
            svg.matches("<circle").count(),
            1,
            "start node must produce exactly one circle"
        );
    }

    #[test]
    fn test_stop_circle() {
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
        // Stop produces two circles: outer ring + inner filled
        assert_eq!(
            svg.matches("<circle").count(),
            2,
            "stop node must produce two circles"
        );
        assert!(svg.contains(r#"r="11""#), "stop outer ring must have r=11");
        assert!(svg.contains(r#"r="7""#), "stop inner circle must have r=7");
        assert!(
            svg.contains(r#"stroke-width:2;"#),
            "outer ring must have stroke-width=2"
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
            svg.contains(r#"rx="10""#),
            "action must have rounded corners"
        );
        assert!(
            svg.contains(r#"ry="10""#),
            "action must have rounded corners"
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
        // cx,y r,cy cx,b l,cy  =>  80.0,50.0 100.0,70.0 80.0,90.0 60.0,70.0
        assert!(svg.contains("80.0,50.0"), "diamond top vertex");
        assert!(svg.contains("100.0,70.0"), "diamond right vertex");
        assert!(svg.contains("80.0,90.0"), "diamond bottom vertex");
        assert!(svg.contains("60.0,70.0"), "diamond left vertex");
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
        // Folded corner produces a polygon + two fold lines
        assert!(svg.contains("<polygon"), "note body must be a polygon");
        // Two fold lines for the corner
        let line_count = svg.matches("<line").count();
        assert!(
            line_count >= 2,
            "note must have at least 2 fold lines, got {line_count}"
        );
    }

    #[test]
    fn test_edge_with_marker() {
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
            svg.contains(r#"marker-end="url(#act-arrow)""#),
            "edge must reference act-arrow marker"
        );
        assert!(
            svg.contains("stroke:#181818"),
            "edge must use EDGE_COLOR"
        );
        // Simple 2-point edge uses <line>
        assert!(svg.contains("<line "), "2-point edge must use <line>");
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
    fn test_polyline_edge() {
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
        assert!(
            svg.contains("<polyline"),
            "multi-point edge must use <polyline>"
        );
        assert!(
            svg.contains(r#"marker-end="url(#act-arrow)""#),
            "polyline must also have arrow marker"
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
            svg.contains(r##"fill="#F1F1F1""##),
            "swimlane header must have background"
        );
        assert!(
            svg.contains("stroke:#181818"),
            "swimlane must have border"
        );
        // Check divider lines extend full height
        assert!(
            svg.contains(r#"y2="300.0""#),
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
        // End renders the same as Stop: two circles
        assert_eq!(
            svg.matches("<circle").count(),
            2,
            "End node must produce two circles like Stop"
        );
    }

    #[test]
    fn test_swimlane_bold_headers() {
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
        // Headers must be bold
        assert!(
            svg.contains("font-weight=\"bold\""),
            "swimlane headers must use bold text"
        );
        // Right border of last lane must be present
        assert!(
            svg.contains("x1=\"400.0\""),
            "right border of last swimlane must be present"
        );
    }

    #[test]
    fn test_cross_lane_polyline_edge() {
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
        assert!(
            svg.contains("<polyline"),
            "4-point cross-lane edge must render as polyline"
        );
        assert!(
            svg.contains(r#"marker-end="url(#act-arrow)""#),
            "polyline must have arrow marker"
        );
    }
}
