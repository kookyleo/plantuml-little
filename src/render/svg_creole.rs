use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::creole_diagram::{CreoleLayout, CreoleLayoutElement};
use crate::model::creole_diagram::CreoleDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg_opt};
use crate::style::SkinParams;
use crate::Result;

const TEXT_FONT_SIZE: f64 = 14.0;
const TEXT_COLOR: &str = "#000000";
const BULLET_RADIUS: f64 = 2.5;

pub fn render_creole(_d: &CreoleDiagram, l: &CreoleLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;
    // Creole is AbstractPSystem — no data-diagram-type
    write_svg_root_bg_opt(&mut buf, sw, sh, None, bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);

    for elem in &l.elements {
        match elem {
            CreoleLayoutElement::Heading {
                text,
                x,
                y,
                text_width,
                font_size,
            } => {
                sg.set_fill_color(TEXT_COLOR);
                sg.svg_text(
                    text,
                    *x,
                    *y,
                    Some("Serif"),
                    *font_size,
                    Some("bold"),
                    None,
                    None,
                    *text_width,
                    LengthAdjust::Spacing,
                    None,
                    0,
                    None,
                );
            }
            CreoleLayoutElement::Bullet {
                cx,
                cy,
                text,
                text_x,
                text_y,
                text_width,
            } => {
                // Bullet circle
                sg.set_fill_color(TEXT_COLOR);
                sg.set_stroke_color(None);
                sg.set_stroke_width(0.0, None);
                sg.svg_ellipse(*cx, *cy, BULLET_RADIUS, BULLET_RADIUS, 0.0);

                // Bullet text
                sg.set_fill_color(TEXT_COLOR);
                sg.svg_text(
                    text,
                    *text_x,
                    *text_y,
                    Some("Serif"),
                    TEXT_FONT_SIZE,
                    None,
                    None,
                    None,
                    *text_width,
                    LengthAdjust::Spacing,
                    None,
                    0,
                    None,
                );
            }
            CreoleLayoutElement::Text {
                text,
                x,
                y,
                text_width,
            } => {
                sg.set_fill_color(TEXT_COLOR);
                sg.svg_text(
                    text,
                    *x,
                    *y,
                    Some("Serif"),
                    TEXT_FONT_SIZE,
                    None,
                    None,
                    None,
                    *text_width,
                    LengthAdjust::Spacing,
                    None,
                    0,
                    None,
                );
            }
        }
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
