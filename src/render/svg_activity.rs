use crate::layout::activity::{
    ActivityEdgeLayout, ActivityLayout, ActivityNodeKindLayout, ActivityNodeLayout,
    NotePositionLayout, SwimlaneLayout,
};
use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
use crate::model::activity::ActivityDiagram;
use crate::render::svg::{write_svg_root_bg, write_bg_rect};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// -- Style constants (PlantUML rose theme) ------------------------------------

const FONT_SIZE: f64 = 13.0;
/// Note line height from font metrics (Java dy=15.1328 at size 13).
/// Action/diamond text uses 16.0 for legacy compat, but notes use the
/// precise font-metrics value to match Java's SheetBlock rendering.
const LINE_HEIGHT: f64 = 16.0;

use crate::skin::rose::{BORDER_COLOR, ENTITY_BG, FORK_FILL, INITIAL_FILL, NOTE_BG, NOTE_BORDER, TEXT_COLOR};

// -- Public entry point -------------------------------------------------------

/// Render an activity diagram to SVG.
pub fn render_activity(
    _diagram: &ActivityDiagram,
    layout: &ActivityLayout,
    skin: &SkinParams,
) -> Result<String> {
    // Skin color lookups
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let act_bg = skin.background_color("activity", ENTITY_BG);
    let act_border = skin.border_color("activity", BORDER_COLOR);
    let act_font = skin.font_color("activity", TEXT_COLOR);
    let diamond_bg = skin.background_color("activityDiamond", ENTITY_BG);
    let diamond_border = skin.border_color("activityDiamond", BORDER_COLOR);
    let swimlane_border = skin.border_color("swimlane", TEXT_COLOR);
    let swimlane_font = skin.font_color("swimlane", TEXT_COLOR);
    let arrow_color = skin.arrow_color(BORDER_COLOR);

    // --- Render all elements into SvgGraphic (tracks ensureVisible) --------
    let mut sg = SvgGraphic::new(0, 1.0);
    // Java: SvgGraphics(minDim) calls ensureVisible(minDim.w, minDim.h).
    // Java's minDim comes from calculateDimension() which does a full dry-run
    // render via LimitFinder.  We replicate this: layout.width/height is our
    // pre-calculated content bounding box from compute_bounds().
    sg.track_rect(0.0, 0.0, layout.width, layout.height);

    // Swimlanes (behind everything) — use layout.height for lane line length
    for sw in &layout.swimlane_layouts {
        render_swimlane(&mut sg, sw, layout.height, swimlane_border, swimlane_font);
    }
    if let Some(last) = layout.swimlane_layouts.last() {
        let right_x = last.x + last.width;
        sg.set_stroke_color(Some(swimlane_border));
        sg.set_stroke_width(1.5, None);
        sg.svg_line(right_x, 0.0, right_x, layout.height, 0.0);
    }

    for edge in &layout.edges {
        render_edge(&mut sg, edge, arrow_color, act_font);
    }

    for node in &layout.nodes {
        render_node(
            &mut sg, node,
            act_bg, act_border, act_font,
            diamond_bg, diamond_border,
            arrow_color,
        );
    }

    // --- SVG dimensions from ensureVisible tracking (Java compat) ----------
    let (max_x, max_y) = sg.max_dimensions();
    let svg_w = max_x as f64;
    let svg_h = max_y as f64;

    // --- Assemble final SVG: header + body --------------------------------
    let mut buf = String::with_capacity(4096);
    write_svg_root_bg(&mut buf, svg_w, svg_h, "ACTIVITY", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

// -- Node rendering -----------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_node(
    sg: &mut SvgGraphic,
    node: &ActivityNodeLayout,
    act_bg: &str,
    act_border: &str,
    act_font: &str,
    diamond_bg: &str,
    diamond_border: &str,
    arrow_color: &str,
) {
    match &node.kind {
        ActivityNodeKindLayout::Start => render_start(sg, node),
        ActivityNodeKindLayout::Stop => render_stop(sg, node),
        ActivityNodeKindLayout::End => render_stop(sg, node),
        ActivityNodeKindLayout::Action => render_action(sg, node, act_bg, act_border, act_font),
        ActivityNodeKindLayout::Diamond => render_diamond(sg, node, diamond_bg, diamond_border),
        ActivityNodeKindLayout::ForkBar => render_fork_bar(sg, node),
        ActivityNodeKindLayout::Note { position } => render_note(sg, node, position),
        ActivityNodeKindLayout::FloatingNote { position } => render_note(sg, node, position),
        ActivityNodeKindLayout::Detach => render_detach(sg, node, arrow_color),
    }
}

/// Start node: filled ellipse
fn render_start(sg: &mut SvgGraphic, node: &ActivityNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    sg.set_fill_color(INITIAL_FILL);
    sg.set_stroke_color(Some(INITIAL_FILL));
    sg.set_stroke_width(1.0, None);
    sg.svg_ellipse(cx, cy, 10.0, 10.0, 0.0);
}

/// Stop / End node: double ellipse (outer ring + inner filled)
fn render_stop(sg: &mut SvgGraphic, node: &ActivityNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    sg.set_fill_color("none");
    sg.set_stroke_color(Some(INITIAL_FILL));
    sg.set_stroke_width(1.0, None);
    sg.svg_ellipse(cx, cy, 11.0, 11.0, 0.0);
    sg.set_fill_color(INITIAL_FILL);
    sg.set_stroke_color(Some(INITIAL_FILL));
    sg.set_stroke_width(1.0, None);
    sg.svg_ellipse(cx, cy, 6.0, 6.0, 0.0);
}

/// Action node: rounded rectangle with (possibly multi-line) text
fn render_action(
    sg: &mut SvgGraphic,
    node: &ActivityNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 12.5, 12.5, 0.0);

    let cx = node.x + node.width / 2.0;
    let lines: Vec<&str> = node.text.split('\n').collect();
    let total_text_height = lines.len() as f64 * LINE_HEIGHT;
    let first_baseline = node.y + (node.height - total_text_height) / 2.0 + FONT_SIZE;

    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &node.text,
        cx,
        first_baseline,
        LINE_HEIGHT,
        font_color,
        Some("middle"),
        r#"font-size="12""#,
    );
    sg.push_raw(&tmp);
}

/// Diamond node: rotated square for if/while conditions
fn render_diamond(sg: &mut SvgGraphic, node: &ActivityNodeLayout, bg: &str, border: &str) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_polygon(0.0, &[cx, y, x + w, cy, cx, y + h, x, cy]);
}

/// Fork bar: thin black horizontal rectangle
fn render_fork_bar(sg: &mut SvgGraphic, node: &ActivityNodeLayout) {
    sg.push_raw(&format!(
        r#"<rect fill="{FORK_FILL}" height="{}" stroke="none" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    ));
}

/// Detach node: an X marker
fn render_detach(sg: &mut SvgGraphic, node: &ActivityNodeLayout, arrow_color: &str) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let r = node.width / 2.0;
    sg.set_stroke_color(Some(arrow_color));
    sg.set_stroke_width(2.0, None);
    sg.svg_line(cx - r, cy - r, cx + r, cy + r, 0.0);
    sg.set_stroke_color(Some(arrow_color));
    sg.set_stroke_width(2.0, None);
    sg.svg_line(cx + r, cy - r, cx - r, cy + r, 0.0);
}

/// Note (or floating note): path-based note shape with folded corner + text
fn render_note(sg: &mut SvgGraphic, node: &ActivityNodeLayout, _position: &NotePositionLayout) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let fold = 10.0;

    // Note body as <path>
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} " fill="{NOTE_BG}" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(x), fmt_coord(y),
        fmt_coord(x), fmt_coord(y + h),
        fmt_coord(x + w), fmt_coord(y + h),
        fmt_coord(x + w), fmt_coord(y + fold),
        fmt_coord(x + w - fold), fmt_coord(y),
        fmt_coord(x), fmt_coord(y),
    ));
    // Track note bounding box for ensureVisible (push_raw bypasses tracking)
    sg.track_rect(x, y, w, h);

    // Fold triangle as <path>
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} L{},{} " fill="{NOTE_BG}" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(x + w - fold), fmt_coord(y),
        fmt_coord(x + w - fold), fmt_coord(y + fold),
        fmt_coord(x + w), fmt_coord(y + fold),
        fmt_coord(x + w - fold), fmt_coord(y),
    ));

    // Render each line as a separate <text> element (matches Java's per-line rendering).
    // This avoids the multi-line textLength issue where a single <text> with tspans
    // gets an incorrect total textLength.
    let note_lh = crate::font_metrics::line_height("SansSerif", FONT_SIZE, false, false);
    let text_x = x + 6.0;
    // Java top margin: fold(10) + ascent(~7.07) = first text baseline y
    let mut text_y = y + fold + FONT_SIZE;
    for line in node.text.split('\n') {
        // Horizontal separator gets less vertical space (Java: 10px)
        let trimmed = line.trim();
        let is_sep = trimmed.len() >= 4
            && (trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-'));
        if is_sep {
            // TODO: render as <line> pair; for now just skip text and add 10px
            text_y += crate::layout::activity::NOTE_SEPARATOR_HEIGHT;
            continue;
        }
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            line,
            text_x,
            text_y,
            note_lh,
            TEXT_COLOR,
            None,
            r#"font-size="13""#,
        );
        sg.push_raw(&tmp);
        text_y += note_lh;
    }
}

// -- Edge rendering -----------------------------------------------------------

fn render_edge(sg: &mut SvgGraphic, edge: &ActivityEdgeLayout, arrow_color: &str, text_color: &str) {
    if edge.points.is_empty() {
        return;
    }

    // Render line segments
    if edge.points.len() == 2 {
        let (x1, y1) = edge.points[0];
        let (x2, y2) = edge.points[1];
        sg.set_stroke_color(Some(arrow_color));
        sg.set_stroke_width(1.0, None);
        sg.svg_line(x1, y1, x2, y2, 0.0);
    } else {
        // Multi-segment: render each segment as a separate <line>
        for pair in edge.points.windows(2) {
            let (x1, y1) = pair[0];
            let (x2, y2) = pair[1];
            sg.set_stroke_color(Some(arrow_color));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(x1, y1, x2, y2, 0.0);
        }
    }

    // Inline arrowhead polygon at the end of the edge
    if edge.points.len() >= 2 {
        let (tx, ty) = *edge.points.last().unwrap();
        let (fx, fy) = edge.points[edge.points.len() - 2];
        render_arrowhead(sg, fx, fy, tx, ty, arrow_color);
    }

    // Edge label (centered on midpoint)
    if !edge.label.is_empty() {
        let mid = edge.points.len() / 2;
        let (mx, my) = edge.points[mid];
        let tl = font_metrics::text_width(&edge.label, "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color(text_color);
        sg.svg_text(
            &edge.label, mx, my,
            Some("sans-serif"), FONT_SIZE,
            None, None, None,
            tl, LengthAdjust::Spacing,
            None, 0, Some("middle"),
        );
    }
}

/// Render an inline arrowhead polygon at the tip of an edge.
fn render_arrowhead(sg: &mut SvgGraphic, fx: f64, fy: f64, tx: f64, ty: f64, color: &str) {
    let dx = tx - fx;
    let dy = ty - fy;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let px = -uy;
    let py = ux;
    let arrow_len = 10.0;
    let arrow_half = 4.0;
    let lx = tx - ux * arrow_len + px * arrow_half;
    let ly = ty - uy * arrow_len + py * arrow_half;
    let rx = tx - ux * arrow_len - px * arrow_half;
    let ry = ty - uy * arrow_len - py * arrow_half;
    let mx = tx - ux * (arrow_len - 4.0);
    let my = ty - uy * (arrow_len - 4.0);
    sg.set_fill_color(color);
    sg.set_stroke_color(Some(color));
    sg.set_stroke_width(1.0, None);
    sg.svg_polygon(0.0, &[lx, ly, tx, ty, rx, ry, mx, my]);
}

// -- Swimlane rendering -------------------------------------------------------

fn render_swimlane(
    sg: &mut SvgGraphic,
    sw: &SwimlaneLayout,
    total_height: f64,
    border: &str,
    font_color: &str,
) {
    // Vertical divider line
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_line(sw.x, 0.0, sw.x, total_height, 0.0);

    // Header label text (font-size 18 to match Java PlantUML)
    let label_x = sw.x + sw.width / 2.0;
    let tl = font_metrics::text_width(&sw.name, "SansSerif", 18.0, false, false);
    sg.set_fill_color(font_color);
    sg.svg_text(
        &sw.name, label_x, 16.0,
        Some("sans-serif"), 18.0,
        None, None, None,
        tl, LengthAdjust::Spacing,
        None, 0, Some("middle"),
    );
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
        ActivityDiagram { events: vec![], swimlanes: vec![], direction: Default::default(), note_max_width: None }
    }

    fn empty_layout() -> ActivityLayout {
        ActivityLayout { width: 200.0, height: 100.0, nodes: vec![], edges: vec![], swimlane_layouts: vec![] }
    }

    fn make_node(index: usize, kind: ActivityNodeKindLayout, x: f64, y: f64, w: f64, h: f64, text: &str) -> ActivityNodeLayout {
        ActivityNodeLayout { index, kind, x, y, width: w, height: h, text: text.to_string() }
    }

    #[test]
    fn test_empty_diagram() {
        let diagram = empty_diagram();
        let layout = empty_layout();
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("<defs/>"));
        assert!(!svg.contains("<ellipse"));
        assert!(!svg.contains("<rect"));
        assert!(!svg.contains("<line "));
    }

    #[test]
    fn test_start_ellipse() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Start, 90.0, 10.0, 20.0, 20.0, ""));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"rx="10""#), "start ellipse must have rx=10");
        assert!(svg.contains(r#"ry="10""#), "start ellipse must have ry=10");
        assert!(svg.contains(&format!(r#"fill="{INITIAL_FILL}""#)), "start ellipse must be filled");
        assert_eq!(svg.matches("<ellipse").count(), 1, "start node must produce exactly one ellipse");
    }

    #[test]
    fn test_stop_ellipse() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Stop, 90.0, 80.0, 22.0, 22.0, ""));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert_eq!(svg.matches("<ellipse").count(), 2, "stop node must produce two ellipses");
        assert!(svg.contains(r#"rx="11""#), "stop outer ring must have rx=11");
        assert!(svg.contains(r#"rx="6""#), "stop inner ellipse must have rx=6");
        assert!(svg.contains(r#"stroke-width:1;"#), "ellipses must have stroke-width=1");
    }

    #[test]
    fn test_action_box() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Action, 30.0, 40.0, 140.0, 36.0, "Do something"));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"rx="12.5""#), "action must have rounded corners rx=12.5");
        assert!(svg.contains(r#"ry="12.5""#), "action must have rounded corners ry=12.5");
        assert!(svg.contains(r#"stroke-width:0.5;"#), "action border must be stroke-width 0.5");
        assert!(svg.contains(r##"fill="#F1F1F1""##), "action must use default theme activity_bg fill");
        assert!(svg.contains("Do something"), "action text must appear in SVG");
        assert!(svg.contains(r#"text-anchor="middle""#), "text must be centered");
    }

    #[test]
    fn test_action_multiline_text() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Action, 30.0, 40.0, 160.0, 52.0, "Line one\nLine two"));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<tspan"), "multi-line text must use <tspan> elements");
        assert!(svg.contains("Line one"), "first line must appear");
        assert!(svg.contains("Line two"), "second line must appear");
        assert_eq!(svg.matches("<tspan").count(), 2, "two lines must produce two tspan elements");
    }

    #[test]
    fn test_diamond_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Diamond, 60.0, 50.0, 40.0, 40.0, ""));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<polygon"), "diamond must be rendered as polygon");
        assert!(svg.contains(r##"fill="#F1F1F1""##), "diamond must use ENTITY_BG");
        assert!(svg.contains("stroke:#181818"), "diamond must use BORDER_COLOR");
        assert!(svg.contains("80,50"), "diamond top vertex");
        assert!(svg.contains("100,70"), "diamond right vertex");
        assert!(svg.contains("80,90"), "diamond bottom vertex");
        assert!(svg.contains("60,70"), "diamond left vertex");
    }

    #[test]
    fn test_fork_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::ForkBar, 40.0, 60.0, 120.0, 6.0, ""));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(&format!(r#"fill="{FORK_FILL}""#)), "fork bar must be black filled");
        assert!(svg.contains(r#"stroke="none""#), "fork bar must have no stroke");
    }

    #[test]
    fn test_note_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Note { position: NotePositionLayout::Right }, 10.0, 20.0, 100.0, 40.0, "Remember this"));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(&format!(r#"fill="{NOTE_BG}""#)), "note must use yellow background");
        assert!(svg.contains("Remember this"), "note text must appear");
        assert!(svg.contains("<path"), "note must use <path> elements");
        assert!(svg.contains("stroke-width:0.5;"), "note must have stroke-width 0.5");
    }

    #[test]
    fn test_edge_with_inline_arrow() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0, to_index: 1, label: String::new(),
            points: vec![(100.0, 30.0), (100.0, 80.0)],
        });
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<polygon"), "edge must have inline polygon arrowhead");
        assert!(svg.contains("stroke:#181818"), "edge must use BORDER_COLOR");
        assert!(svg.contains("<line "), "2-point edge must use <line>");
        assert!(!svg.contains("marker-end"), "edges must use inline polygon, not marker-end");
    }

    #[test]
    fn test_edge_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0, to_index: 1, label: "yes".to_string(),
            points: vec![(100.0, 30.0), (100.0, 80.0)],
        });
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("yes"), "edge label must appear in SVG");
    }

    #[test]
    fn test_multi_segment_edge() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0, to_index: 1, label: String::new(),
            points: vec![(50.0, 20.0), (50.0, 50.0), (100.0, 50.0), (100.0, 80.0)],
        });
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        let line_count = svg.matches("<line ").count();
        assert!(line_count >= 3, "4-point edge must produce at least 3 line segments, got {line_count}");
        assert!(svg.contains("<polygon"), "multi-segment edge must have inline polygon arrowhead");
    }

    #[test]
    fn test_swimlane_headers() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout.swimlane_layouts.push(SwimlaneLayout { name: "Lane A".to_string(), x: 0.0, width: 200.0 });
        layout.swimlane_layouts.push(SwimlaneLayout { name: "Lane B".to_string(), x: 200.0, width: 200.0 });
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Lane A"), "swimlane A header must appear");
        assert!(svg.contains("Lane B"), "swimlane B header must appear");
        assert!(svg.contains("stroke:#000000"), "swimlane must have #000000 border");
        assert!(svg.contains("stroke-width:1.5;"), "swimlane lines must have stroke-width 1.5");
        assert!(svg.contains(r#"y2="300""#), "swimlane dividers must extend full diagram height");
    }

    #[test]
    fn test_xml_escape_in_action() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Action, 10.0, 10.0, 160.0, 36.0, "A & B < C"));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("A &amp; B &lt; C"), "special characters must be XML-escaped");
    }

    #[test]
    fn test_end_node_same_as_stop() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::End, 90.0, 80.0, 22.0, 22.0, ""));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert_eq!(svg.matches("<ellipse").count(), 2, "End node must produce two ellipses like Stop");
    }

    #[test]
    fn test_swimlane_text_headers() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout.swimlane_layouts.push(SwimlaneLayout { name: "Lane X".to_string(), x: 0.0, width: 200.0 });
        layout.swimlane_layouts.push(SwimlaneLayout { name: "Lane Y".to_string(), x: 200.0, width: 200.0 });
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"font-size="18""#), "swimlane headers must use font-size 18");
        assert!(svg.contains(r#"x1="400""#), "right border of last swimlane must be present");
    }

    #[test]
    fn test_cross_lane_multi_segment_edge() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout.edges.push(ActivityEdgeLayout {
            from_index: 0, to_index: 1, label: String::new(),
            points: vec![(100.0, 50.0), (100.0, 80.0), (300.0, 80.0), (300.0, 110.0)],
        });
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        let line_count = svg.matches("<line ").count();
        assert!(line_count >= 3, "4-point cross-lane edge must produce at least 3 line segments");
        assert!(svg.contains("<polygon"), "cross-lane edge must have inline polygon arrowhead");
    }

    #[test]
    fn test_fmt_coord_in_output() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(make_node(0, ActivityNodeKindLayout::Start, 90.0, 10.0, 20.0, 20.0, ""));
        let svg = render_activity(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"cx="100""#), "fmt_coord must strip trailing .0");
        assert!(svg.contains(r#"cy="20""#), "fmt_coord must strip trailing .0");
    }
}
