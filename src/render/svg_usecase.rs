use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::usecase::{
    ActorLayout, BoundaryLayout, UseCaseEdgeLayout, UseCaseLayout, UseCaseNodeLayout,
};
use crate::model::usecase::UseCaseDiagram;
use crate::render::svg::{write_svg_root_bg, write_bg_rect, ensure_visible_int};
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants (PlantUML defaults)
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
use crate::skin::rose::{BORDER_COLOR, ENTITY_BG, TEXT_COLOR};
const BOUNDARY_BORDER: &str = "#444444";

// Stick-figure proportions (relative to actor center / top of head circle).
const HEAD_R: f64 = 10.0;
/// Vertical body line: from below head to body-bottom.
const BODY_TOP_OFFSET: f64 = HEAD_R; // start just below head center
const BODY_LEN: f64 = 30.0;
/// Arms branch from mid-body.
const ARM_MID_FRAC: f64 = 0.35; // fraction along body for arm attachment
const ARM_SPREAD: f64 = 18.0;
const ARM_RAISE: f64 = 5.0;
/// Legs spread from body bottom.
const LEG_SPREAD: f64 = 14.0;
const LEG_DROP: f64 = 22.0;
/// Name text below the figure.
const NAME_OFFSET: f64 = 6.0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render a use case diagram to SVG body (no outer document wrapper).
pub fn render_usecase(
    _diagram: &UseCaseDiagram,
    layout: &UseCaseLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    let uc_bg = skin.background_color("usecase", ENTITY_BG);
    let uc_border = skin.border_color("usecase", BORDER_COLOR);
    let uc_font = skin.font_color("usecase", TEXT_COLOR);
    let actor_stroke = skin.border_color("actor", BORDER_COLOR);
    let actor_font = skin.font_color("actor", TEXT_COLOR);
    let boundary_border = skin.border_color("boundary", BOUNDARY_BORDER);
    let boundary_font = skin.font_color("boundary", TEXT_COLOR);
    let arrow_color = skin.arrow_color(BORDER_COLOR);

    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = ensure_visible_int(layout.total_width) as f64;
    let svg_h = ensure_visible_int(layout.total_height) as f64;
    write_svg_root_bg(
        &mut buf,
        svg_w,
        svg_h,
        "DESCRIPTION",
        bg,
    );
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Boundaries first (behind everything)
    for boundary in &layout.boundaries {
        render_boundary(&mut sg, boundary, boundary_border, boundary_font);
    }

    // Use cases
    for uc in &layout.usecases {
        render_usecase_oval(&mut sg, uc, uc_bg, uc_border, uc_font);
    }

    // Actors
    for actor in &layout.actors {
        render_actor(&mut sg, actor, actor_stroke, actor_font);
    }

    // Edges (on top of everything)
    for edge in &layout.edges {
        render_edge(&mut sg, edge, arrow_color, TEXT_COLOR);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Actor (stick figure)
// ---------------------------------------------------------------------------

fn render_actor(sg: &mut SvgGraphic, actor: &ActorLayout, stroke: &str, font_color: &str) {
    let cx = actor.cx;
    let fig_top = actor.cy - actor.height / 2.0 + HEAD_R;
    let head_cy = fig_top + HEAD_R;

    let body_top_y = head_cy + BODY_TOP_OFFSET;
    let body_bot_y = body_top_y + BODY_LEN;
    let arm_y = body_top_y + BODY_LEN * ARM_MID_FRAC;
    let leg_y = body_bot_y;
    let name_y = leg_y + LEG_DROP + NAME_OFFSET + FONT_SIZE;

    // Head circle
    sg.set_fill_color("none");
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_circle(cx, head_cy, HEAD_R, 0.0);

    // Body
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(cx, body_top_y, cx, body_bot_y, 0.0);

    // Left arm
    let la_x = cx - ARM_SPREAD;
    let la_y = arm_y - ARM_RAISE;
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(cx, arm_y, la_x, la_y, 0.0);

    // Right arm
    let ra_x = cx + ARM_SPREAD;
    let ra_y = arm_y - ARM_RAISE;
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(cx, arm_y, ra_x, ra_y, 0.0);

    // Left leg
    let ll_x = cx - LEG_SPREAD;
    let ll_y = leg_y + LEG_DROP;
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(cx, leg_y, ll_x, ll_y, 0.0);

    // Right leg
    let rl_x = cx + LEG_SPREAD;
    let rl_y = leg_y + LEG_DROP;
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(cx, leg_y, rl_x, rl_y, 0.0);

    // Name label centered below
    let tl = font_metrics::text_width(&actor.name, "SansSerif", FONT_SIZE, false, false);
    sg.set_fill_color(font_color);
    sg.svg_text(
        &actor.name, cx, name_y,
        Some("sans-serif"), FONT_SIZE,
        None, None, None,
        tl, LengthAdjust::Spacing,
        None, 0, Some("middle"),
    );
}

// ---------------------------------------------------------------------------
// Use case oval
// ---------------------------------------------------------------------------

fn render_usecase_oval(
    sg: &mut SvgGraphic,
    uc: &UseCaseNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_ellipse(uc.cx, uc.cy, uc.rx, uc.ry, 0.0);

    let text_y = uc.cy + FONT_SIZE * 0.35;
    let tl = font_metrics::text_width(&uc.name, "SansSerif", FONT_SIZE, false, false);
    sg.set_fill_color(font_color);
    sg.svg_text(
        &uc.name, uc.cx, text_y,
        Some("sans-serif"), FONT_SIZE,
        None, None, None,
        tl, LengthAdjust::Spacing,
        None, 0, Some("middle"),
    );
}

// ---------------------------------------------------------------------------
// Boundary rectangle
// ---------------------------------------------------------------------------

fn render_boundary(sg: &mut SvgGraphic, boundary: &BoundaryLayout, border: &str, font_color: &str) {
    let (fill, dash) = if boundary.nesting_depth > 0 {
        ("#F8F8FF", Some((6.0, 3.0)))
    } else {
        ("none", Some((8.0, 4.0)))
    };

    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, dash);
    sg.svg_rectangle(boundary.x, boundary.y, boundary.width, boundary.height, 4.0, 4.0, 0.0);

    let name_x = boundary.x + 8.0;
    let name_y = boundary.y + FONT_SIZE + 4.0;
    let tl = font_metrics::text_width(&boundary.name, "SansSerif", FONT_SIZE, true, false);
    sg.set_fill_color(font_color);
    sg.svg_text(
        &boundary.name, name_x, name_y,
        Some("sans-serif"), FONT_SIZE,
        Some("bold"), None, None,
        tl, LengthAdjust::Spacing,
        None, 0, None,
    );
}

// ---------------------------------------------------------------------------
// Edge (line with optional arrow and labels)
// ---------------------------------------------------------------------------

fn render_edge(sg: &mut SvgGraphic, edge: &UseCaseEdgeLayout, arrow_color: &str, font_color: &str) {
    let dash = if edge.dashed {
        Some((7.0, 5.0))
    } else {
        None
    };

    sg.set_stroke_color(Some(arrow_color));
    sg.set_stroke_width(1.0, dash);
    sg.svg_line(edge.from_x, edge.from_y, edge.to_x, edge.to_y, 0.0);

    // Inline polygon arrowhead
    if edge.has_arrow {
        let dx = edge.to_x - edge.from_x;
        let dy = edge.to_y - edge.from_y;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            let ux = dx / len;
            let uy = dy / len;
            let px = -uy;
            let py = ux;
            let p1x = edge.to_x - ux * 9.0 + px * 4.0;
            let p1y = edge.to_y - uy * 9.0 + py * 4.0;
            let p2x = edge.to_x;
            let p2y = edge.to_y;
            let p3x = edge.to_x - ux * 9.0 - px * 4.0;
            let p3y = edge.to_y - uy * 9.0 - py * 4.0;

            sg.set_fill_color(arrow_color);
            sg.set_stroke_color(Some(arrow_color));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[p1x, p1y, p2x, p2y, p3x, p3y, p1x, p1y]);
        }
    }

    let mid_x = (edge.from_x + edge.to_x) / 2.0;
    let mid_y = (edge.from_y + edge.to_y) / 2.0;

    // Stereotype above the midpoint (e.g. <<include>>)
    if let Some(ref stereo) = edge.stereotype {
        if !stereo.is_empty() {
            let stereo_text = format!("\u{00AB}{stereo}\u{00BB}");
            let stereo_fs = FONT_SIZE - 1.0;
            let tl = font_metrics::text_width(&stereo_text, "SansSerif", stereo_fs, false, true);
            sg.set_fill_color(font_color);
            sg.svg_text(
                &stereo_text, mid_x, mid_y - FONT_SIZE - 2.0,
                Some("sans-serif"), stereo_fs,
                None, Some("italic"), None,
                tl, LengthAdjust::Spacing,
                None, 0, Some("middle"),
            );
        }
    }

    // Edge label at midpoint
    if !edge.label.is_empty() {
        let tl = font_metrics::text_width(&edge.label, "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color(font_color);
        sg.svg_text(
            &edge.label, mid_x, mid_y + FONT_SIZE,
            Some("sans-serif"), FONT_SIZE,
            None, None, None,
            tl, LengthAdjust::Spacing,
            None, 0, Some("middle"),
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::usecase::{
        ActorLayout, BoundaryLayout, UseCaseEdgeLayout, UseCaseLayout, UseCaseNodeLayout,
    };
    use crate::model::diagram::Direction;
    use crate::model::usecase::UseCaseDiagram;
    use crate::style::SkinParams;

    fn empty_diagram() -> UseCaseDiagram {
        UseCaseDiagram {
            actors: vec![], usecases: vec![], links: vec![],
            boundaries: vec![], notes: vec![], direction: Direction::LeftToRight,
        }
    }

    fn empty_layout() -> UseCaseLayout {
        UseCaseLayout {
            actors: vec![], usecases: vec![], edges: vec![],
            boundaries: vec![], total_width: 400.0, total_height: 300.0,
        }
    }

    #[test]
    fn test_empty_diagram_svg() {
        let diagram = empty_diagram();
        let layout = empty_layout();
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
        assert!(svg.contains("<defs/>"), "must have empty defs");
    }

    #[test]
    fn test_svg_dimensions() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.total_width = 600.0;
        layout.total_height = 400.0;
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"width="601px""#), "width uses ensure_visible_int(600)=601");
        assert!(svg.contains(r#"height="401px""#), "height uses ensure_visible_int(400)=401");
        assert!(svg.contains(r#"viewBox="0 0 601 401""#), "viewBox uses ensure_visible_int");
    }

    #[test]
    fn test_actor_stick_figure() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.actors.push(ActorLayout {
            id: "a1".into(), name: "Customer".into(),
            cx: 100.0, cy: 100.0, width: 50.0, height: 80.0,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<circle"), "actor must have head circle");
        let line_count = svg.matches("<line").count();
        assert!(line_count >= 5, "actor must have 5 lines (body + 2 arms + 2 legs), got {line_count}");
        assert!(svg.contains("Customer"), "actor name must appear");
    }

    #[test]
    fn test_actor_name_xml_escaped() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.actors.push(ActorLayout {
            id: "a1".into(), name: "A & B < C".into(),
            cx: 100.0, cy: 100.0, width: 50.0, height: 80.0,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("A &amp; B &lt; C"), "actor name must be XML-escaped");
    }

    #[test]
    fn test_usecase_oval() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.usecases.push(UseCaseNodeLayout {
            id: "uc1".into(), name: "Login".into(),
            cx: 200.0, cy: 150.0, rx: 70.0, ry: 25.0,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<ellipse"), "use case must produce an ellipse");
        assert!(svg.contains("Login"), "use case name must appear");
        assert!(svg.contains("fill=\"#F1F1F1\""), "use case must use default fill");
        assert!(svg.contains(r#"text-anchor="middle""#), "use case text must be centered");
    }

    #[test]
    fn test_usecase_name_xml_escaped() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.usecases.push(UseCaseNodeLayout {
            id: "uc1".into(), name: "Search & Filter".into(),
            cx: 200.0, cy: 150.0, rx: 80.0, ry: 25.0,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Search &amp; Filter"), "use case name must be XML-escaped");
    }

    #[test]
    fn test_boundary_rect() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.boundaries.push(BoundaryLayout {
            name: "MySystem".into(), x: 50.0, y: 50.0, width: 200.0, height: 150.0, nesting_depth: 0,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "boundary must produce a rect");
        assert!(svg.contains("stroke-dasharray"), "boundary must be dashed");
        assert!(svg.contains("MySystem"), "boundary name must appear");
        assert!(svg.contains(r#"fill="none""#), "boundary must have no fill");
    }

    #[test]
    fn test_boundary_name_xml_escaped() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.boundaries.push(BoundaryLayout {
            name: "System <v2>".into(), x: 50.0, y: 50.0, width: 200.0, height: 150.0, nesting_depth: 0,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("System &lt;v2&gt;"), "boundary name must be XML-escaped");
    }

    #[test]
    fn test_edge_association() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0, from_y: 100.0, to_x: 200.0, to_y: 100.0,
            label: String::new(), dashed: false, has_arrow: true, stereotype: None,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<line"), "edge must produce a line");
        assert!(svg.contains("<polygon"), "edge must have inline polygon arrowhead");
        assert!(!svg.contains("stroke-dasharray"), "association must not be dashed");
        assert!(!svg.contains("\u{00AB}"), "association must not have stereotype guillemets");
    }

    #[test]
    fn test_edge_dashed_with_stereotype() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0, from_y: 100.0, to_x: 250.0, to_y: 100.0,
            label: String::new(), dashed: true, has_arrow: true, stereotype: Some("include".into()),
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("stroke-dasharray"), "dashed edge must have dasharray");
        assert!(svg.contains("&#171;include&#187;"), "stereotype must appear with guillemets");
        assert!(svg.contains("font-style=\"italic\""), "stereotype must be italic");
    }

    #[test]
    fn test_edge_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0, from_y: 80.0, to_x: 200.0, to_y: 80.0,
            label: "uses".into(), dashed: false, has_arrow: true, stereotype: None,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("uses"), "edge label must appear");
    }

    #[test]
    fn test_full_diagram() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.actors.push(ActorLayout {
            id: "a1".into(), name: "User".into(), cx: 50.0, cy: 100.0, width: 50.0, height: 80.0,
        });
        layout.usecases.push(UseCaseNodeLayout {
            id: "uc1".into(), name: "Login".into(), cx: 250.0, cy: 100.0, rx: 70.0, ry: 25.0,
        });
        layout.boundaries.push(BoundaryLayout {
            name: "System".into(), x: 160.0, y: 60.0, width: 180.0, height: 80.0, nesting_depth: 0,
        });
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0, from_y: 100.0, to_x: 180.0, to_y: 100.0,
            label: String::new(), dashed: false, has_arrow: true, stereotype: None,
        });
        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<ellipse"), "must have use case oval");
        assert!(svg.contains("<circle"), "must have actor head");
        assert!(svg.contains("<rect"), "must have boundary rect");
        assert!(svg.contains("<line"), "must have lines (actor body + edge)");
        assert!(svg.contains("User"), "actor name must appear");
        assert!(svg.contains("Login"), "use case name must appear");
        assert!(svg.contains("System"), "boundary name must appear");
    }
}
