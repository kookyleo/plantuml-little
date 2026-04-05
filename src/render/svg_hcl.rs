use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
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
    sg.set_fill_color(ENTITY_BG);
    sg.set_stroke_color(Some(ENTITY_BG));
    sg.set_stroke_width(1.5, None);
    sg.svg_rectangle(bx, by, bw, bh, 5.0, 5.0, 0.0);

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
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_line(
            layout.separator_x,
            row.y_top,
            layout.separator_x,
            row.y_top + row.height,
            0.0,
        );

        // Horizontal separator between rows
        if i < layout.rows.len() - 1 {
            let ly = row.y_top + row.height;
            sg.set_stroke_color(Some(BORDER_COLOR));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(bx, ly, bx + bw, ly, 0.0);
        }
    }

    // Border rect
    sg.set_fill_color("none");
    sg.set_stroke_color(Some(BORDER_COLOR));
    sg.set_stroke_width(1.5, None);
    sg.svg_rectangle(bx, by, bw, bh, 5.0, 5.0, 0.0);

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
