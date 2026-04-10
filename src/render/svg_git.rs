use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::git::GitLayout;
use crate::model::git::GitDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg_opt};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 13.0;

/// Color palette for git nodes.
const COLORS: &[&str] = &[
    "#4E79A7", "#F28E2B", "#E15759", "#76B7B2", "#59A14F", "#EDC948", "#B07AA1", "#FF9DA7",
];
const EDGE_COLOR: &str = "#555555";
const TEXT_COLOR: &str = "#333333";

pub fn render_git(_d: &GitDiagram, l: &GitLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;

    write_svg_root_bg_opt(&mut buf, sw, sh, None, &bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, sw, sh, &bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Draw edges first (behind nodes)
    sg.set_stroke_color(Some(EDGE_COLOR));
    sg.set_stroke_width(2.0, None);
    for edge in &l.edges {
        sg.svg_line(edge.x1, edge.y1, edge.x2, edge.y2, 0.0);
    }

    // Draw nodes
    for (_i, node) in l.nodes.iter().enumerate() {
        let color = COLORS[(node.depth - 1) % COLORS.len()];

        // Draw filled circle
        sg.set_fill_color(color);
        sg.set_stroke_color(Some("#333333"));
        sg.set_stroke_width(1.5, None);
        sg.svg_ellipse(
            node.cx,
            node.cy,
            node.radius,
            node.radius,
            0.0,
        );

        // Draw label
        let tw = font_metrics::text_width(&node.label, "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.set_stroke_color(None);
        sg.set_stroke_width(0.0, None);
        sg.svg_text(
            &node.label,
            node.label_x,
            node.label_y,
            Some("sans-serif"),
            FONT_SIZE,
            None,
            None,
            None,
            tw,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
