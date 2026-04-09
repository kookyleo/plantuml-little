use crate::font_metrics;
use crate::klimt::shape::UPath;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::wire::WireLayout;
use crate::model::wire::WireDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 12.0;
const TEXT_COLOR: &str = "#000000";
/// Java ImageBuilder margin = 10 applied as shift to all drawing.
const MARGIN: f64 = 10.0;
/// Label X offset from block left edge (Java WBlock uses 5).
const LABEL_OFFSET_X: f64 = 5.0;
/// Java WBlock nbsp text at cursor_x - 5 = 10 - 5 = 5.
const TOP_TEXT_X: f64 = 5.0;

pub fn render_wire(_d: &WireDiagram, l: &WireLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;
    write_svg_root_bg(&mut buf, sw, sh, "WIRE", bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);

    // Draw blocks (shifted by MARGIN)
    for bl in &l.blocks {
        let rx = bl.x + MARGIN;
        let ry = bl.y + MARGIN;

        // Rect with no fill, black stroke (matches Java WBlock.drawBox)
        sg.set_fill_color("none");
        sg.set_stroke_color(Some(TEXT_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_rectangle(rx, ry, bl.width, bl.height, 0.0, 0.0, 0.0);

        // Name label at (x + 5, y + ascent) — Java uses sansSerif 12
        let baseline = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
        let tw = font_metrics::text_width(&bl.name, "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &bl.name,
            rx + LABEL_OFFSET_X,
            ry + baseline,
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

    // Top nbsp text (shifted by MARGIN)
    {
        let tw = font_metrics::text_width("\u{00a0}", "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            "\u{00a0}",
            TOP_TEXT_X + MARGIN,
            l.top_text_y + MARGIN,
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

    // Draw vertical links (arrows, shifted by MARGIN).
    // Java renders: path (arrowhead) then line, for each link.
    for vl in &l.vlinks {
        let vx = vl.x + MARGIN;
        let arrow_y = vl.arrow_tip_y + MARGIN;
        let line_y_start = vl.line_y_start + MARGIN;
        let line_y_end = vl.line_y_end + MARGIN;

        // Arrow triangle (UPath): M(0,0) L(5,-5) L(-5,-5) L(0,0) closePath
        // Drawn at translate (vx, arrow_y)
        sg.set_fill_color(TEXT_COLOR);
        sg.set_stroke_color(None);
        sg.set_stroke_width(0.0, None);
        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(5.0, -5.0);
        path.line_to(-5.0, -5.0);
        path.line_to(0.0, 0.0);
        path.close();
        sg.svg_path(vx, arrow_y, &path, 0.0);

        // Line from (vx, line_y_start) of length (line_y_end - line_y_start)
        sg.set_fill_color("none");
        sg.set_stroke_color(Some(TEXT_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_line(vx, line_y_start, vx, line_y_end, 0.0);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
