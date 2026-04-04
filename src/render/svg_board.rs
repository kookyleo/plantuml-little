use crate::font_metrics;
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

    for col in &l.columns {
        // Column background
        sg.set_fill_color(COL_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_rectangle(col.x, col.y, col.width, col.height, 5.0, 5.0, 0.0);

        // Column header
        sg.set_fill_color(HEADER_BG);
        sg.set_stroke_color(None);
        sg.set_stroke_width(0.0, None);
        sg.svg_rectangle(col.x, col.y, col.width, 24.0, 5.0, 5.0, 0.0);

        let tw = font_metrics::text_width(&col.header, "SansSerif", FONT_SIZE, true, false);
        let baseline =
            font_metrics::ascent("SansSerif", FONT_SIZE, true, false);
        sg.set_fill_color(HEADER_FG);
        sg.svg_text(
            &col.header,
            col.x + CARD_PAD_H,
            col.y + 4.0 + baseline,
            Some("sans-serif"),
            FONT_SIZE,
            Some("700"),
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
            sg.set_fill_color(CARD_BG);
            sg.set_stroke_color(Some(BORDER_COLOR));
            sg.set_stroke_width(0.5, None);
            sg.svg_rectangle(card.x + 4.0, card.y, card.width - 8.0, card.height, 3.0, 3.0, 0.0);

            let ctw =
                font_metrics::text_width(&card.label, "SansSerif", FONT_SIZE, false, false);
            let cbl =
                font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
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
