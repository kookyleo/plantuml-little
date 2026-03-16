use std::fmt::Write;

use crate::font_metrics;
use crate::layout::json_diagram::{JsonArrow, JsonBox, JsonLayout};
use crate::model::json_diagram::JsonDiagram;
use crate::render::svg::fmt_coord;
use crate::render::svg::{write_svg_root_bg, write_bg_rect};
use crate::render::svg::xml_escape;
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const PADDING: f64 = 5.0;
const BOX_FILL: &str = "#F1F1F1";
const BORDER_COLOR: &str = "#000000";
const TEXT_COLOR: &str = "#000000";

fn baseline_offset() -> f64 {
    font_metrics::ascent("SansSerif", FONT_SIZE, false, false) + 2.0
}

fn line_height() -> f64 {
    font_metrics::line_height("SansSerif", FONT_SIZE, false, false)
}

pub fn render_json(jd: &JsonDiagram, layout: &JsonLayout, skin: &SkinParams) -> Result<String> {
    render_with_type(jd, layout, skin, "JSON")
}

pub fn render_yaml(jd: &JsonDiagram, layout: &JsonLayout, skin: &SkinParams) -> Result<String> {
    render_with_type(jd, layout, skin, "YAML")
}

fn render_with_type(_jd: &JsonDiagram, layout: &JsonLayout, skin: &SkinParams, dtype: &str) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, dtype, bg);
    buf.push_str("<defs/><g>");

    for jbox in &layout.boxes { render_box(&mut buf, jbox); }
    for arrow in &layout.arrows { render_arrow(&mut buf, arrow); }

    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_box(buf: &mut String, jbox: &JsonBox) {
    let (x, y, w, h) = (jbox.x, jbox.y, jbox.width, jbox.height);

    // Background fill
    write!(buf, r#"<rect fill="{BOX_FILL}" height="{}" rx="5" ry="5" style="stroke:{BOX_FILL};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y)).unwrap();

    let has_keys = jbox.rows.iter().any(|r| r.key.is_some());
    let bl = baseline_offset();
    let lh = line_height();

    for (i, row) in jbox.rows.iter().enumerate() {
        let text_y = row.y_top + bl;

        if let Some(ref key) = row.key {
            let key_x = x + PADDING;
            let key_esc = xml_escape(key);
            let key_tl = fmt_coord(font_metrics::text_width(key, "SansSerif", FONT_SIZE, true, false));
            write!(buf, r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="14" font-weight="700" lengthAdjust="spacing" textLength="{key_tl}" x="{}" y="{}">{key_esc}</text>"#,
                fmt_coord(key_x), fmt_coord(text_y)).unwrap();
        }

        let val_x = if has_keys { jbox.separator_x + PADDING } else { x + PADDING };
        for (li, line) in row.value_lines.iter().enumerate() {
            let line_y = text_y + li as f64 * lh;
            let val_esc = xml_escape(line);
            let val_tl = fmt_coord(font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false));
            write!(buf, r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="14" lengthAdjust="spacing" textLength="{val_tl}" x="{}" y="{}">{val_esc}</text>"#,
                fmt_coord(val_x), fmt_coord(line_y)).unwrap();
        }

        if has_keys {
            let sx = fmt_coord(jbox.separator_x);
            write!(buf, r#"<line style="stroke:{BORDER_COLOR};stroke-width:1;" x1="{sx}" x2="{sx}" y1="{}" y2="{}"/>"#,
                fmt_coord(row.y_top), fmt_coord(row.y_top + row.height)).unwrap();
        }

        if i < jbox.rows.len() - 1 {
            let ly = fmt_coord(row.y_top + row.height);
            write!(buf, r#"<line style="stroke:{BORDER_COLOR};stroke-width:1;" x1="{}" x2="{}" y1="{ly}" y2="{ly}"/>"#,
                fmt_coord(x), fmt_coord(x + w)).unwrap();
        }
    }

    // Border rect
    write!(buf, r#"<rect fill="none" height="{}" rx="5" ry="5" style="stroke:{BORDER_COLOR};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y)).unwrap();
}

fn render_arrow(buf: &mut String, arrow: &JsonArrow) {
    let (fx, fy, tx, ty) = (arrow.from_x, arrow.from_y, arrow.to_x, arrow.to_y);
    let mid_x = (fx + tx) / 2.0;
    write!(buf, r#"<path d="M{},{} L{},{} C{},{} {},{} {},{}" fill="none" style="stroke:{BORDER_COLOR};stroke-width:1;stroke-dasharray:3,3;"/>"#,
        fmt_coord(fx), fmt_coord(fy), fmt_coord(fx + 13.0), fmt_coord(fy),
        fmt_coord(mid_x), fmt_coord(fy), fmt_coord(mid_x), fmt_coord(ty),
        fmt_coord(tx - 7.0), fmt_coord(ty)).unwrap();

    let sz = 3.1073;
    write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{}" fill="{BORDER_COLOR}"/>"#,
        fmt_coord(tx - 7.0), fmt_coord(ty + sz), fmt_coord(tx - 5.0), fmt_coord(ty),
        fmt_coord(tx - 7.0), fmt_coord(ty - sz), fmt_coord(tx), fmt_coord(ty),
        fmt_coord(tx - 7.0), fmt_coord(ty + sz)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::json_diagram::layout_json;
    use crate::model::json_diagram::{JsonDiagram, JsonValue};
    use crate::style::SkinParams;

    #[test]
    fn test_simple_render() {
        let jd = JsonDiagram { root: JsonValue::Object(vec![
            ("name".into(), JsonValue::Str("Alice".into())),
        ]) };
        let layout = layout_json(&jd).unwrap();
        let svg = render_json(&jd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("name"));
        assert!(svg.contains("Alice"));
    }

    #[test]
    fn test_boolean_rendering() {
        let jd = JsonDiagram { root: JsonValue::Object(vec![
            ("a".into(), JsonValue::Bool(true)),
        ]) };
        let layout = layout_json(&jd).unwrap();
        let svg = render_json(&jd, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("\u{2611}") || svg.contains("&#9745;"));
    }
}
