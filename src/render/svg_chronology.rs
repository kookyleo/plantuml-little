use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
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
    sg.set_stroke_color(Some(LINE_COLOR));
    sg.set_stroke_width(2.0, None);
    sg.svg_line(l.line_x1, l.line_y, l.line_x2, l.line_y, 0.0);

    // Events
    for ev in &l.events {
        // Marker circle
        sg.push_raw(&format!(
            r#"<circle cx="{:.4}" cy="{:.4}" r="{MARKER_RADIUS}" fill="{MARKER_COLOR}" style="stroke:#FFFFFF;stroke-width:2;"/>"#,
            ev.x, ev.y,
        ));

        // Vertical connector line
        sg.set_stroke_color(Some(LINE_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_line(ev.x, ev.y - MARKER_RADIUS, ev.x, ev.label_y + 4.0, 0.0);

        // Label
        let label_w =
            font_metrics::text_width(&ev.label, "SansSerif", LABEL_FONT_SIZE, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &ev.label,
            ev.label_x,
            ev.label_y,
            Some("sans-serif"),
            LABEL_FONT_SIZE,
            None,
            None,
            None,
            label_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );

        // Date
        let date_w =
            font_metrics::text_width(&ev.date, "SansSerif", DATE_FONT_SIZE, false, false);
        sg.set_fill_color(DATE_COLOR);
        sg.svg_text(
            &ev.date,
            ev.date_x,
            ev.date_y,
            Some("sans-serif"),
            DATE_FONT_SIZE,
            None,
            None,
            None,
            date_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
