use crate::font_metrics;
use crate::klimt::drawable::{DrawStyle, Drawable, LineShape, RectShape};
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::hcl::HclLayout;
use crate::model::hcl::HclDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const PADDING: f64 = 5.0;

use crate::skin::rose::{ENTITY_BG, TEXT_COLOR};
const BORDER_COLOR: &str = "#000000";

fn baseline_offset() -> f64 {
    font_metrics::ascent("SansSerif", FONT_SIZE, false, false) + 2.0
}

pub fn render_hcl(_d: &HclDiagram, layout: &HclLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = ensure_visible_int(layout.width) as f64;
    let svg_h = ensure_visible_int(layout.height) as f64;
    write_svg_root_bg(&mut buf, svg_w, svg_h, "HCL", bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);
    let bl = baseline_offset();

    let (bx, by, bw, bh) = (layout.box_x, layout.box_y, layout.box_w, layout.box_h);

    // Background fill
    RectShape {
        x: bx,
        y: by,
        w: bw,
        h: bh,
        rx: 5.0,
        ry: 5.0,
    }
    .draw(&mut sg, &DrawStyle::filled(ENTITY_BG, ENTITY_BG, 1.5));

    let separator_style = DrawStyle::outline(BORDER_COLOR, 1.0);

    // Rows
    for (i, row) in layout.rows.iter().enumerate() {
        let text_y = row.y_top + bl;

        // Key (bold)
        let key_x = bx + PADDING;
        let key_tl = font_metrics::text_width(&row.key, "SansSerif", FONT_SIZE, true, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &row.key,
            key_x,
            text_y,
            Some("sans-serif"),
            FONT_SIZE,
            Some("bold"),
            None,
            None,
            key_tl,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );

        // Value
        let val_x = layout.separator_x + PADDING;
        let val_tl = font_metrics::text_width(&row.value, "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &row.value,
            val_x,
            text_y,
            Some("sans-serif"),
            FONT_SIZE,
            None,
            None,
            None,
            val_tl,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );

        // Vertical separator
        LineShape {
            x1: layout.separator_x,
            y1: row.y_top,
            x2: layout.separator_x,
            y2: row.y_top + row.height,
        }
        .draw(&mut sg, &separator_style);

        // Horizontal separator between rows
        if i < layout.rows.len() - 1 {
            let ly = row.y_top + row.height;
            LineShape {
                x1: bx,
                y1: ly,
                x2: bx + bw,
                y2: ly,
            }
            .draw(&mut sg, &separator_style);
        }
    }

    // Border rect
    RectShape {
        x: bx,
        y: by,
        w: bw,
        h: bh,
        rx: 5.0,
        ry: 5.0,
    }
    .draw(&mut sg, &DrawStyle::outline(BORDER_COLOR, 1.5));

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
