use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::math::MathLayout;
use crate::model::math::MathDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg_opt};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const TEXT_COLOR: &str = "#000000";

pub fn render_math(_d: &MathDiagram, l: &MathLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(2048);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;
    // Math/Latex are AbstractPSystem — no data-diagram-type attribute
    write_svg_root_bg_opt(&mut buf, sw, sh, None, bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);
    sg.set_fill_color(TEXT_COLOR);
    sg.svg_text(
        &l.display_text,
        l.text_x,
        l.text_y,
        Some("monospace"),
        FONT_SIZE,
        None,
        None,
        None,
        l.text_width,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

/// Render a @startdef diagram — raw text display of the start tag.
///
/// Java PSystemDefinition renders via UgSimpleDiagram with sans-serif 14pt.
/// No data-diagram-type attribute, no margin.
pub fn render_def(_d: &MathDiagram, l: &MathLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(2048);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;
    // Def is AbstractPSystem — no data-diagram-type attribute
    write_svg_root_bg_opt(&mut buf, sw, sh, None, bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);
    sg.set_fill_color(TEXT_COLOR);
    sg.svg_text(
        &l.display_text,
        l.text_x,
        l.text_y,
        Some("sans-serif"),
        14.0,
        None,
        None,
        None,
        l.text_width,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
