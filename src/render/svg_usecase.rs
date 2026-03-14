use std::fmt::Write;

use crate::layout::usecase::{
    ActorLayout, BoundaryLayout, UseCaseEdgeLayout, UseCaseLayout, UseCaseNodeLayout,
};
use crate::model::usecase::UseCaseDiagram;
use crate::render::svg::fmt_coord;
use crate::render::svg::write_svg_root;
use crate::render::svg::xml_escape;
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants (PlantUML defaults)
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
const ACTOR_STROKE: &str = "#181818";
const UC_BG: &str = "#F1F1F1";
const UC_BORDER: &str = "#181818";
const BOUNDARY_BORDER: &str = "#444444";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";

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

    let uc_bg = skin.background_color("usecase", UC_BG);
    let uc_border = skin.border_color("usecase", UC_BORDER);
    let uc_font = skin.font_color("usecase", TEXT_FILL);
    let actor_stroke = skin.border_color("actor", ACTOR_STROKE);
    let actor_font = skin.font_color("actor", TEXT_FILL);
    let boundary_border = skin.border_color("boundary", BOUNDARY_BORDER);
    let boundary_font = skin.font_color("boundary", TEXT_FILL);
    let arrow_color = skin.arrow_color(EDGE_COLOR);

    write_svg_root(
        &mut buf,
        layout.total_width,
        layout.total_height,
        "DESCRIPTION",
    );
    buf.push_str("<defs/><g>");

    // Boundaries first (behind everything)
    for boundary in &layout.boundaries {
        render_boundary(&mut buf, boundary, boundary_border, boundary_font);
    }

    // Use cases
    for uc in &layout.usecases {
        render_usecase_oval(&mut buf, uc, uc_bg, uc_border, uc_font);
    }

    // Actors
    for actor in &layout.actors {
        render_actor(&mut buf, actor, actor_stroke, actor_font);
    }

    // Edges (on top of everything)
    for edge in &layout.edges {
        render_edge(&mut buf, edge, arrow_color, TEXT_FILL);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Actor (stick figure)
// ---------------------------------------------------------------------------

fn render_actor(buf: &mut String, actor: &ActorLayout, stroke: &str, font_color: &str) {
    let cx = actor.cx;
    // The actor's cy is the vertical center of the figure.
    // Distribute: head center is above cy, legs end below cy.
    // Total figure height ≈ 2*HEAD_R + BODY_LEN + LEG_DROP = 10+10+30+22 = 72 ~ ACTOR_HEIGHT(80)
    let fig_top = actor.cy - actor.height / 2.0 + HEAD_R;
    let head_cy = fig_top + HEAD_R; // center of head circle

    let body_top_y = head_cy + BODY_TOP_OFFSET;
    let body_bot_y = body_top_y + BODY_LEN;
    let arm_y = body_top_y + BODY_LEN * ARM_MID_FRAC;
    let leg_y = body_bot_y;
    let name_y = leg_y + LEG_DROP + NAME_OFFSET + FONT_SIZE;

    // Head circle
    write!(
        buf,
        r#"<circle cx="{}" cy="{}" fill="none" r="{HEAD_R}" style="stroke:{stroke};stroke-width:0.5;"/>"#,
        fmt_coord(cx), fmt_coord(head_cy),
    )
    .unwrap();
    buf.push('\n');

    // Body
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx),
        fmt_coord(cx),
        fmt_coord(body_top_y),
        fmt_coord(body_bot_y),
    )
    .unwrap();
    buf.push('\n');

    // Left arm
    let la_x = cx - ARM_SPREAD;
    let la_y = arm_y - ARM_RAISE;
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx),
        fmt_coord(la_x),
        fmt_coord(arm_y),
        fmt_coord(la_y),
    )
    .unwrap();
    buf.push('\n');

    // Right arm
    let ra_x = cx + ARM_SPREAD;
    let ra_y = arm_y - ARM_RAISE;
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx),
        fmt_coord(ra_x),
        fmt_coord(arm_y),
        fmt_coord(ra_y),
    )
    .unwrap();
    buf.push('\n');

    // Left leg
    let ll_x = cx - LEG_SPREAD;
    let ll_y = leg_y + LEG_DROP;
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx),
        fmt_coord(ll_x),
        fmt_coord(leg_y),
        fmt_coord(ll_y),
    )
    .unwrap();
    buf.push('\n');

    // Right leg
    let rl_x = cx + LEG_SPREAD;
    let rl_y = leg_y + LEG_DROP;
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx),
        fmt_coord(rl_x),
        fmt_coord(leg_y),
        fmt_coord(rl_y),
    )
    .unwrap();
    buf.push('\n');

    // Name label centered below
    let name_escaped = xml_escape(&actor.name);
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="{FONT_SIZE}" text-anchor="middle" x="{}" y="{}">{name_escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
    )
    .unwrap();
    buf.push('\n');
}

// ---------------------------------------------------------------------------
// Use case oval
// ---------------------------------------------------------------------------

fn render_usecase_oval(
    buf: &mut String,
    uc: &UseCaseNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    write!(
        buf,
        r#"<ellipse cx="{}" cy="{}" fill="{bg}" rx="{}" ry="{}" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(uc.cx), fmt_coord(uc.cy), fmt_coord(uc.rx), fmt_coord(uc.ry),
    )
    .unwrap();
    buf.push('\n');

    let name_escaped = xml_escape(&uc.name);
    let text_y = uc.cy + FONT_SIZE * 0.35;
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="{FONT_SIZE}" text-anchor="middle" x="{}" y="{}">{name_escaped}</text>"#,
        fmt_coord(uc.cx), fmt_coord(text_y),
    )
    .unwrap();
    buf.push('\n');
}

// ---------------------------------------------------------------------------
// Boundary rectangle
// ---------------------------------------------------------------------------

fn render_boundary(buf: &mut String, boundary: &BoundaryLayout, border: &str, font_color: &str) {
    // Nested boundaries use a light background fill
    // to visually distinguish inner rectangles from outer ones.
    let (fill, dash) = if boundary.nesting_depth > 0 {
        ("#F8F8FF", r#"stroke-dasharray="6,3""#)
    } else {
        ("none", r#"stroke-dasharray="8,4""#)
    };

    write!(
        buf,
        r#"<rect fill="{fill}" height="{}" rx="4" ry="4" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"{dash}/>"#,
        fmt_coord(boundary.height), fmt_coord(boundary.width), fmt_coord(boundary.x), fmt_coord(boundary.y),
    )
    .unwrap();
    buf.push('\n');

    let name_escaped = xml_escape(&boundary.name);
    let name_x = boundary.x + 8.0;
    let name_y = boundary.y + FONT_SIZE + 4.0;
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" x="{}" y="{}">{name_escaped}</text>"#,
        fmt_coord(name_x), fmt_coord(name_y),
    )
    .unwrap();
    buf.push('\n');
}

// ---------------------------------------------------------------------------
// Edge (line with optional arrow and labels)
// ---------------------------------------------------------------------------

fn render_edge(buf: &mut String, edge: &UseCaseEdgeLayout, arrow_color: &str, font_color: &str) {
    let dash = if edge.dashed {
        r#" stroke-dasharray="7,5""#
    } else {
        ""
    };

    write!(
        buf,
        r#"<line style="stroke:{arrow_color};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"{dash}/>"#,
        fmt_coord(edge.from_x), fmt_coord(edge.to_x), fmt_coord(edge.from_y), fmt_coord(edge.to_y),
    )
    .unwrap();
    buf.push('\n');

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

            write!(
                buf,
                r#"<polygon fill="{arrow_color}" points="{},{},{},{},{},{},{},{}" style="stroke:{arrow_color};stroke-width:1;"/>"#,
                fmt_coord(p1x), fmt_coord(p1y),
                fmt_coord(p2x), fmt_coord(p2y),
                fmt_coord(p3x), fmt_coord(p3y),
                fmt_coord(p1x), fmt_coord(p1y),
            )
            .unwrap();
            buf.push('\n');
        }
    }

    let mid_x = (edge.from_x + edge.to_x) / 2.0;
    let mid_y = (edge.from_y + edge.to_y) / 2.0;

    // Stereotype above the midpoint (e.g. <<include>>)
    if let Some(ref stereo) = edge.stereotype {
        if !stereo.is_empty() {
            let stereo_text = format!("\u{00AB}{stereo}\u{00BB}");
            let escaped = xml_escape(&stereo_text);
            write!(
                buf,
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="{fs:.0}" font-style="italic" text-anchor="middle" x="{}" y="{}">{escaped}</text>"#,
                fmt_coord(mid_x), fmt_coord(mid_y - FONT_SIZE - 2.0),
                fs = FONT_SIZE - 1.0,
            )
            .unwrap();
            buf.push('\n');
        }
    }

    // Edge label at midpoint
    if !edge.label.is_empty() {
        let label_escaped = xml_escape(&edge.label);
        write!(
            buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{FONT_SIZE}" text-anchor="middle" x="{}" y="{}">{label_escaped}</text>"#,
            fmt_coord(mid_x), fmt_coord(mid_y + FONT_SIZE),
        )
        .unwrap();
        buf.push('\n');
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
            actors: vec![],
            usecases: vec![],
            links: vec![],
            boundaries: vec![],
            notes: vec![],
            direction: Direction::LeftToRight,
        }
    }

    fn empty_layout() -> UseCaseLayout {
        UseCaseLayout {
            actors: vec![],
            usecases: vec![],
            edges: vec![],
            boundaries: vec![],
            total_width: 400.0,
            total_height: 300.0,
        }
    }

    // 1. Empty diagram produces valid SVG structure
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

    // 2. SVG dimensions match layout
    #[test]
    fn test_svg_dimensions() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.total_width = 600.0;
        layout.total_height = 400.0;

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"width="600px""#), "width must match");
        assert!(svg.contains(r#"height="400px""#), "height must match");
        assert!(
            svg.contains(r#"viewBox="0 0 600 400""#),
            "viewBox must match"
        );
    }

    // 3. Actor renders as stick figure elements
    #[test]
    fn test_actor_stick_figure() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.actors.push(ActorLayout {
            id: "a1".into(),
            name: "Customer".into(),
            cx: 100.0,
            cy: 100.0,
            width: 50.0,
            height: 80.0,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");

        // Head: circle
        assert!(svg.contains("<circle"), "actor must have head circle");
        // Body and limbs: lines
        let line_count = svg.matches("<line").count();
        assert!(
            line_count >= 5,
            "actor must have 5 lines (body + 2 arms + 2 legs), got {line_count}"
        );
        assert!(svg.contains("Customer"), "actor name must appear");
    }

    // 4. Actor name is XML-escaped
    #[test]
    fn test_actor_name_xml_escaped() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.actors.push(ActorLayout {
            id: "a1".into(),
            name: "A & B < C".into(),
            cx: 100.0,
            cy: 100.0,
            width: 50.0,
            height: 80.0,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("A &amp; B &lt; C"),
            "actor name must be XML-escaped"
        );
    }

    // 5. Use case oval renders ellipse with name
    #[test]
    fn test_usecase_oval() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.usecases.push(UseCaseNodeLayout {
            id: "uc1".into(),
            name: "Login".into(),
            cx: 200.0,
            cy: 150.0,
            rx: 70.0,
            ry: 25.0,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<ellipse"), "use case must produce an ellipse");
        assert!(svg.contains("Login"), "use case name must appear");
        assert!(
            svg.contains("fill=\"#F1F1F1\""),
            "use case must use default fill"
        );
        assert!(
            svg.contains(r#"text-anchor="middle""#),
            "use case text must be centered"
        );
    }

    // 6. Use case name is XML-escaped
    #[test]
    fn test_usecase_name_xml_escaped() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.usecases.push(UseCaseNodeLayout {
            id: "uc1".into(),
            name: "Search & Filter".into(),
            cx: 200.0,
            cy: 150.0,
            rx: 80.0,
            ry: 25.0,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("Search &amp; Filter"),
            "use case name must be XML-escaped"
        );
    }

    // 7. Boundary renders as dashed rectangle with name
    #[test]
    fn test_boundary_rect() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.boundaries.push(BoundaryLayout {
            name: "MySystem".into(),
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 150.0,
            nesting_depth: 0,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "boundary must produce a rect");
        assert!(svg.contains("stroke-dasharray"), "boundary must be dashed");
        assert!(svg.contains("MySystem"), "boundary name must appear");
        assert!(svg.contains(r#"fill="none""#), "boundary must have no fill");
    }

    // 8. Boundary name is XML-escaped
    #[test]
    fn test_boundary_name_xml_escaped() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.boundaries.push(BoundaryLayout {
            name: "System <v2>".into(),
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 150.0,
            nesting_depth: 0,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("System &lt;v2&gt;"),
            "boundary name must be XML-escaped"
        );
    }

    // 9. Association edge: solid line with arrow, no stereotype text
    #[test]
    fn test_edge_association() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0,
            from_y: 100.0,
            to_x: 200.0,
            to_y: 100.0,
            label: String::new(),
            dashed: false,
            has_arrow: true,
            stereotype: None,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<line"), "edge must produce a line");
        assert!(
            svg.contains("<polygon"),
            "edge must have inline polygon arrowhead"
        );
        assert!(
            !svg.contains("stroke-dasharray"),
            "association must not be dashed"
        );
        assert!(
            !svg.contains("\u{00AB}"),
            "association must not have stereotype guillemets"
        );
    }

    // 10. Dashed edge with stereotype shows dashes and stereotype text
    #[test]
    fn test_edge_dashed_with_stereotype() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0,
            from_y: 100.0,
            to_x: 250.0,
            to_y: 100.0,
            label: String::new(),
            dashed: true,
            has_arrow: true,
            stereotype: Some("include".into()),
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("stroke-dasharray"),
            "dashed edge must have dasharray"
        );
        assert!(
            svg.contains("\u{00AB}include\u{00BB}"),
            "stereotype must appear with guillemets"
        );
        assert!(
            svg.contains("font-style=\"italic\""),
            "stereotype must be italic"
        );
    }

    // 11. Edge with label shows label text at midpoint
    #[test]
    fn test_edge_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0,
            from_y: 80.0,
            to_x: 200.0,
            to_y: 80.0,
            label: "uses".into(),
            dashed: false,
            has_arrow: true,
            stereotype: None,
        });

        let svg = render_usecase(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("uses"), "edge label must appear");
    }

    // 12. Full diagram: actors, use cases, boundary, edge
    #[test]
    fn test_full_diagram() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();

        layout.actors.push(ActorLayout {
            id: "a1".into(),
            name: "User".into(),
            cx: 50.0,
            cy: 100.0,
            width: 50.0,
            height: 80.0,
        });
        layout.usecases.push(UseCaseNodeLayout {
            id: "uc1".into(),
            name: "Login".into(),
            cx: 250.0,
            cy: 100.0,
            rx: 70.0,
            ry: 25.0,
        });
        layout.boundaries.push(BoundaryLayout {
            name: "System".into(),
            x: 160.0,
            y: 60.0,
            width: 180.0,
            height: 80.0,
            nesting_depth: 0,
        });
        layout.edges.push(UseCaseEdgeLayout {
            from_x: 50.0,
            from_y: 100.0,
            to_x: 180.0,
            to_y: 100.0,
            label: String::new(),
            dashed: false,
            has_arrow: true,
            stereotype: None,
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
