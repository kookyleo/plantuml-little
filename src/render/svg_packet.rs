use crate::font_metrics;
use crate::klimt::drawable::{DrawStyle, Drawable, RectShape, TextShape};
use crate::klimt::svg::SvgGraphic;
use crate::layout::packet::PacketLayout;
use crate::model::packet::PacketDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg_opt};
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

    write_svg_root_bg_opt(&mut buf, sw, sh, None, bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, sw, sh, bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Draw bit number headers
    let header_style = DrawStyle::fill_only(HEADER_TEXT_COLOR);
    for (x, label) in &l.bit_labels {
        let tw = font_metrics::text_width(label, "SansSerif", HEADER_FONT_SIZE, false, false);
        TextShape {
            x: x - tw / 2.0,
            y: 16.0,
            text: label.clone(),
            font_family: "sans-serif".into(),
            font_size: HEADER_FONT_SIZE,
            text_length: tw,
            bold: false,
            italic: false,
        }
        .draw(&mut sg, &header_style);
    }

    // Draw cells
    let cell_rect_style = DrawStyle::filled(CELL_FILL, CELL_STROKE, 1.0);
    let cell_text_style = DrawStyle::fill_only(TEXT_COLOR);
    for cell in &l.cells {
        // Fill rectangle
        RectShape {
            x: cell.x,
            y: cell.y,
            w: cell.width,
            h: cell.height,
            rx: 0.0,
            ry: 0.0,
        }
        .draw(&mut sg, &cell_rect_style);

        // Draw label text centered in cell
        if !cell.label.is_empty() {
            let tw = font_metrics::text_width(&cell.label, "SansSerif", FONT_SIZE, false, false);
            let tx = cell.x + (cell.width - tw) / 2.0;
            let ty = cell.y + cell.height / 2.0 + FONT_SIZE / 3.0;
            TextShape {
                x: tx,
                y: ty,
                text: cell.label.clone(),
                font_family: "sans-serif".into(),
                font_size: FONT_SIZE,
                text_length: tw,
                bold: false,
                italic: false,
            }
            .draw(&mut sg, &cell_text_style);
        }
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
