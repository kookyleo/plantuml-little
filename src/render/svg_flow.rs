use crate::klimt::drawable::{DrawStyle, Drawable, EllipseShape, LineShape, RectShape};
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::flow::FlowLayout;
use crate::model::flow::FlowDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg_opt};
use crate::style::SkinParams;
use crate::Result;

const BOX_FILL: &str = "#FEFECE";
const BOX_STROKE: &str = "#A80036";
const TEXT_COLOR: &str = "#000000";
const CORNER_RADIUS: f64 = 12.5;
const FONT_SIZE: f64 = 14.0;

pub fn render_flow(
    _diagram: &FlowDiagram,
    layout: &FlowLayout,
    _skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    write_svg_root_bg_opt(
        &mut buf,
        ensure_visible_int(layout.width) as f64,
        ensure_visible_int(layout.height) as f64,
        None,
        "#FFFFFF",
    );
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);
    let box_style = DrawStyle::filled(BOX_FILL, BOX_STROKE, 1.5);
    let text_style = DrawStyle::fill_only(TEXT_COLOR);
    let line_style = DrawStyle::outline(BOX_STROKE, 1.0);
    let dot_style = DrawStyle::filled(BOX_STROKE, BOX_STROKE, 1.0);

    for node in layout.nodes.iter() {
        RectShape {
            x: node.x,
            y: node.y,
            w: node.width,
            h: node.height,
            rx: CORNER_RADIUS,
            ry: CORNER_RADIUS,
        }
        .draw(&mut sg, &box_style);

        sg.set_fill_color(TEXT_COLOR);
        sg.set_stroke_color(None);
        sg.svg_text(
            &node.label,
            node.text_x,
            node.text_y,
            Some("Serif"),
            FONT_SIZE,
            None,
            None,
            None,
            node.text_length,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }

    for path in &layout.paths {
        LineShape {
            x1: path.x1,
            y1: path.y1,
            x2: path.x2,
            y2: path.y2,
        }
        .draw(
            &mut sg,
            &DrawStyle {
                fill: Some("none".into()),
                ..line_style.clone()
            },
        );

        EllipseShape {
            cx: path.ellipse_cx,
            cy: path.ellipse_cy,
            rx: 3.5,
            ry: 3.5,
        }
        .draw(&mut sg, &dot_style);
    }

    let _ = text_style;
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
