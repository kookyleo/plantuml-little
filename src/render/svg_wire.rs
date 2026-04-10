use crate::font_metrics;
use crate::klimt::drawable::{DrawStyle, Drawable, LineShape, PathShape, RectShape, TextShape};
use crate::klimt::shape::UPath;
use crate::klimt::svg::SvgGraphic;
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

    let outline_style = DrawStyle::outline(TEXT_COLOR, 1.0);
    let text_style = DrawStyle::fill_only(TEXT_COLOR);

    // Draw blocks (shifted by MARGIN)
    for bl in &l.blocks {
        let rx = bl.x + MARGIN;
        let ry = bl.y + MARGIN;

        // Rect with no fill, black stroke (matches Java WBlock.drawBox)
        RectShape {
            x: rx,
            y: ry,
            w: bl.width,
            h: bl.height,
            rx: 0.0,
            ry: 0.0,
        }
        .draw(&mut sg, &outline_style);

        // Name label at (x + 5, y + ascent) — Java uses sansSerif 12
        let baseline = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
        let tw = font_metrics::text_width(&bl.name, "SansSerif", FONT_SIZE, false, false);
        TextShape {
            x: rx + LABEL_OFFSET_X,
            y: ry + baseline,
            text: bl.name.clone(),
            font_family: "sans-serif".into(),
            font_size: FONT_SIZE,
            text_length: tw,
            bold: false,
            italic: false,
        }
        .draw(&mut sg, &text_style);
    }

    // Top nbsp text (shifted by MARGIN)
    {
        let tw = font_metrics::text_width("\u{00a0}", "SansSerif", FONT_SIZE, false, false);
        TextShape {
            x: TOP_TEXT_X + MARGIN,
            y: l.top_text_y + MARGIN,
            text: "\u{00a0}".into(),
            font_family: "sans-serif".into(),
            font_size: FONT_SIZE,
            text_length: tw,
            bold: false,
            italic: false,
        }
        .draw(&mut sg, &text_style);
    }

    // Draw vertical links (arrows, shifted by MARGIN).
    // Java renders: path (arrowhead) then line, for each link.
    let arrow_style = DrawStyle::fill_only(TEXT_COLOR);
    for vl in &l.vlinks {
        let vx = vl.x + MARGIN;
        let arrow_y = vl.arrow_tip_y + MARGIN;
        let line_y_start = vl.line_y_start + MARGIN;
        let line_y_end = vl.line_y_end + MARGIN;

        // Arrow triangle (UPath): M(0,0) L(5,-5) L(-5,-5) L(0,0) closePath
        // Drawn at translate (vx, arrow_y)
        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(5.0, -5.0);
        path.line_to(-5.0, -5.0);
        path.line_to(0.0, 0.0);
        path.close();
        PathShape {
            x: vx,
            y: arrow_y,
            path,
        }
        .draw(&mut sg, &arrow_style);

        // Line from (vx, line_y_start) of length (line_y_end - line_y_start)
        LineShape {
            x1: vx,
            y1: line_y_start,
            x2: vx,
            y2: line_y_end,
        }
        .draw(&mut sg, &outline_style);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
