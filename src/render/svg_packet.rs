use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::packet::PacketLayout;
use crate::model::packet::PacketDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 12.0;
const HEADER_FONT_SIZE: f64 = 10.0;

/// Color palette for packet cells.
const CELL_FILL: &str = "#FEFECE";
const CELL_STROKE: &str = "#A80036";
const TEXT_COLOR: &str = "#000000";
const HEADER_TEXT_COLOR: &str = "#888888";

pub fn render_packet(_d: &PacketDiagram, l: &PacketLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;

    write_svg_root_bg(&mut buf, sw, sh, "PACKET", &bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, sw, sh, &bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Draw bit number headers
    sg.set_fill_color(HEADER_TEXT_COLOR);
    for (x, label) in &l.bit_labels {
        let tw = font_metrics::text_width(label, "SansSerif", HEADER_FONT_SIZE, false, false);
        sg.svg_text(
            label,
            x - tw / 2.0,
            16.0,
            Some("sans-serif"),
            HEADER_FONT_SIZE,
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

    // Draw cells
    for cell in &l.cells {
        // Fill rectangle
        sg.set_fill_color(CELL_FILL);
        sg.set_stroke_color(Some(CELL_STROKE));
        sg.set_stroke_width(1.0, None);
        sg.svg_rectangle(cell.x, cell.y, cell.width, cell.height, 0.0, 0.0, 0.0);

        // Draw label text centered in cell
        if !cell.label.is_empty() {
            sg.set_fill_color(TEXT_COLOR);
            sg.set_stroke_color(None);
            let tw =
                font_metrics::text_width(&cell.label, "SansSerif", FONT_SIZE, false, false);
            let tx = cell.x + (cell.width - tw) / 2.0;
            let ty = cell.y + cell.height / 2.0 + FONT_SIZE / 3.0;
            sg.svg_text(
                &cell.label,
                tx,
                ty,
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
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
