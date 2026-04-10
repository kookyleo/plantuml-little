use crate::font_metrics;
use crate::klimt::drawable::{DrawStyle, Drawable, EllipseShape, LineShape, TextShape};
use crate::klimt::svg::SvgGraphic;
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

    write_svg_root_bg_opt(&mut buf, sw, sh, None, bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, sw, sh, bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Draw edges first (behind nodes)
    let edge_style = DrawStyle::outline(EDGE_COLOR, 2.0);
    for edge in &l.edges {
        LineShape {
            x1: edge.x1,
            y1: edge.y1,
            x2: edge.x2,
            y2: edge.y2,
        }
        .draw(&mut sg, &edge_style);
    }

    // Draw nodes
    for node in l.nodes.iter() {
        let color = COLORS[(node.depth - 1) % COLORS.len()];

        // Draw filled circle
        EllipseShape {
            cx: node.cx,
            cy: node.cy,
            rx: node.radius,
            ry: node.radius,
        }
        .draw(&mut sg, &DrawStyle::filled(color, "#333333", 1.5));

        // Draw label
        let tw = font_metrics::text_width(&node.label, "SansSerif", FONT_SIZE, false, false);
        TextShape {
            x: node.label_x,
            y: node.label_y,
            text: node.label.clone(),
            font_family: "sans-serif".into(),
            font_size: FONT_SIZE,
            text_length: tw,
            bold: false,
            italic: false,
        }
        .draw(&mut sg, &DrawStyle::fill_only(TEXT_COLOR));
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
