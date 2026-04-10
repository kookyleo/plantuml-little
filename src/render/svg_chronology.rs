use crate::font_metrics;
use crate::klimt::drawable::{DrawStyle, Drawable, LineShape, TextShape};
use crate::klimt::svg::SvgGraphic;
use crate::layout::chronology::ChronologyLayout;
use crate::model::chronology::ChronologyDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const LABEL_FONT_SIZE: f64 = 12.0;
const DATE_FONT_SIZE: f64 = 11.0;
const MARKER_RADIUS: f64 = 6.0;
const LINE_COLOR: &str = "#4E79A7";
const MARKER_COLOR: &str = "#4E79A7";
const TEXT_COLOR: &str = "#000000";
const DATE_COLOR: &str = "#666666";

pub fn render_chronology(
    _d: &ChronologyDiagram,
    l: &ChronologyLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;
    write_svg_root_bg(&mut buf, sw, sh, "CHRONOLOGY", bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);

    // Main horizontal line
    LineShape {
        x1: l.line_x1,
        y1: l.line_y,
        x2: l.line_x2,
        y2: l.line_y,
    }
    .draw(&mut sg, &DrawStyle::outline(LINE_COLOR, 2.0));

    // Events
    let label_style = DrawStyle::fill_only(TEXT_COLOR);
    let date_style = DrawStyle::fill_only(DATE_COLOR);
    for ev in &l.events {
        // Marker circle (uses inline style for stroke — keep as push_raw)
        sg.push_raw(&format!(
            r#"<circle cx="{:.4}" cy="{:.4}" r="{MARKER_RADIUS}" fill="{MARKER_COLOR}" style="stroke:#FFFFFF;stroke-width:2;"/>"#,
            ev.x, ev.y,
        ));

        // Vertical connector line
        LineShape {
            x1: ev.x,
            y1: ev.y - MARKER_RADIUS,
            x2: ev.x,
            y2: ev.label_y + 4.0,
        }
        .draw(&mut sg, &DrawStyle::outline(LINE_COLOR, 1.0));

        // Label
        let label_w =
            font_metrics::text_width(&ev.label, "SansSerif", LABEL_FONT_SIZE, false, false);
        TextShape {
            x: ev.label_x,
            y: ev.label_y,
            text: ev.label.clone(),
            font_family: "sans-serif".into(),
            font_size: LABEL_FONT_SIZE,
            text_length: label_w,
            bold: false,
            italic: false,
        }
        .draw(&mut sg, &label_style);

        // Date
        let date_w = font_metrics::text_width(&ev.date, "SansSerif", DATE_FONT_SIZE, false, false);
        TextShape {
            x: ev.date_x,
            y: ev.date_y,
            text: ev.date.clone(),
            font_family: "sans-serif".into(),
            font_size: DATE_FONT_SIZE,
            text_length: date_w,
            bold: false,
            italic: false,
        }
        .draw(&mut sg, &date_style);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
