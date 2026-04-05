use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::pie::PieLayout;
use crate::model::pie::PieDiagram;
use crate::render::svg::{ensure_visible_int, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const LEGEND_BOX: f64 = 12.0;

const COLORS: &[&str] = &[
    "#F44336", "#E91E63", "#9C27B0", "#673AB7", "#3F51B5", "#2196F3", "#03A9F4", "#00BCD4",
    "#009688", "#4CAF50", "#8BC34A", "#CDDC39", "#FFEB3B", "#FFC107", "#FF9800", "#FF5722",
];

pub fn render_pie(_d: &PieDiagram, l: &PieLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(l.width) as f64;
    let sh = ensure_visible_int(l.height) as f64;
    write_svg_root_bg(&mut buf, sw, sh, "PIE", bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);

    // Title
    if let Some(ref title) = l.title {
        let tw = font_metrics::text_width(title, "SansSerif", FONT_SIZE, true, false);
        sg.set_fill_color("#000000");
        sg.svg_text(
            title,
            l.title_x - tw / 2.0,
            l.title_y,
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
    }

    // Pie slices
    let (cx, cy, r) = (l.cx, l.cy, l.radius);
    for slice in &l.slices {
        let color = COLORS[slice.color_index % COLORS.len()];
        let start_rad = (slice.start_angle - 90.0).to_radians();
        let end_rad = (slice.end_angle - 90.0).to_radians();
        let x1 = cx + r * start_rad.cos();
        let y1 = cy + r * start_rad.sin();
        let x2 = cx + r * end_rad.cos();
        let y2 = cy + r * end_rad.sin();
        let large = if slice.end_angle - slice.start_angle > 180.0 {
            1
        } else {
            0
        };
        let path = format!(
            "M{cx:.4},{cy:.4} L{x1:.4},{y1:.4} A{r:.4},{r:.4} 0 {large},1 {x2:.4},{y2:.4} Z"
        );
        sg.push_raw(&format!(
            r#"<path d="{path}" fill="{color}" style="stroke:#FFFFFF;stroke-width:1;"/>"#,
        ));
    }

    // Legend
    for entry in &l.legend {
        let color = COLORS[entry.color_index % COLORS.len()];
        sg.set_fill_color(color);
        sg.set_stroke_color(Some("#333333"));
        sg.set_stroke_width(0.5, None);
        sg.svg_rectangle(entry.x, entry.y, LEGEND_BOX, LEGEND_BOX, 0.0, 0.0, 0.0);

        let tw = font_metrics::text_width(&entry.label, "SansSerif", FONT_SIZE, false, false);
        sg.set_fill_color("#000000");
        sg.svg_text(
            &entry.label,
            entry.x + LEGEND_BOX + 6.0,
            entry.y + LEGEND_BOX - 1.0,
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

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}
