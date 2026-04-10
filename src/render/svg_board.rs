use crate::font_metrics;
use crate::klimt::drawable::{DrawStyle, Drawable, RectShape};
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::board::BoardLayout;
use crate::model::board::BoardDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const CARD_PAD_H: f64 = 8.0;
const CARD_PAD_V: f64 = 6.0;
const COL_BG: &str = "#F0F0F0";
const CARD_BG: &str = "#FFFFFF";
const HEADER_BG: &str = "#4E79A7";
const HEADER_FG: &str = "#FFFFFF";
const TEXT_COLOR: &str = "#000000";
const BORDER_COLOR: &str = "#CCCCCC";

pub fn render_board(_d: &BoardDiagram, l: &BoardLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;
    write_svg_root_bg(&mut buf, sw, sh, "BOARD", bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);

    let col_style = DrawStyle::filled(COL_BG, BORDER_COLOR, 1.0);
    let header_style = DrawStyle {
        fill: Some(HEADER_BG.into()),
        stroke: None,
        stroke_width: 0.0,
        dash_array: None,
        delta_shadow: 0.0,
    };
    let card_style = DrawStyle::filled(CARD_BG, BORDER_COLOR, 0.5);

    for col in &l.columns {
        // Column background
        RectShape {
            x: col.x,
            y: col.y,
            w: col.width,
            h: col.height,
            rx: 5.0,
            ry: 5.0,
        }
        .draw(&mut sg, &col_style);

        // Column header
        RectShape {
            x: col.x,
            y: col.y,
            w: col.width,
            h: 24.0,
            rx: 5.0,
            ry: 5.0,
        }
        .draw(&mut sg, &header_style);

        let tw = font_metrics::text_width(&col.header, "SansSerif", FONT_SIZE, true, false);
        let baseline = font_metrics::ascent("SansSerif", FONT_SIZE, true, false);
        sg.set_fill_color(HEADER_FG);
        sg.svg_text(
            &col.header,
            col.x + CARD_PAD_H,
            col.y + 4.0 + baseline,
            Some("sans-serif"),
            FONT_SIZE,
            Some("bold"),
            None,
            None,
            tw,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );

        // Cards
        for card in &col.cards {
            RectShape {
                x: card.x + 4.0,
                y: card.y,
                w: card.width - 8.0,
                h: card.height,
                rx: 3.0,
                ry: 3.0,
            }
            .draw(&mut sg, &card_style);

            let ctw = font_metrics::text_width(&card.label, "SansSerif", FONT_SIZE, false, false);
            let cbl = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
            sg.set_fill_color(TEXT_COLOR);
            sg.svg_text(
                &card.label,
                card.x + 4.0 + CARD_PAD_H,
                card.y + CARD_PAD_V + cbl,
                Some("sans-serif"),
                FONT_SIZE,
                None,
                None,
                None,
                ctw,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
        }
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
