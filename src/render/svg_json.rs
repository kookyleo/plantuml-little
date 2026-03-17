use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
use crate::layout::json_diagram::{JsonArrow, JsonBox, JsonLayout};
use crate::model::json_diagram::JsonDiagram;
use crate::render::svg::write_svg_root_bg;
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const PADDING: f64 = 5.0;
use crate::skin::rose::{ENTITY_BG, TEXT_COLOR};
const BORDER_COLOR: &str = "#000000";

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

    let mut sg = SvgGraphic::new(0, 1.0);
    for jbox in &layout.boxes { render_box(&mut sg, jbox); }
    for arrow in &layout.arrows { render_arrow(&mut sg, arrow); }
    buf.push_str(sg.body());

    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_box(sg: &mut SvgGraphic, jbox: &JsonBox) {
    let (x, y, w, h) = (jbox.x, jbox.y, jbox.width, jbox.height);

    // Background fill
    sg.set_fill_color(ENTITY_BG);
    sg.set_stroke_color(Some(ENTITY_BG));
    sg.set_stroke_width(1.5, None);
    sg.svg_rectangle(x, y, w, h, 5.0, 5.0, 0.0);

    let has_keys = jbox.rows.iter().any(|r| r.key.is_some());
    let bl = baseline_offset();
    let lh = line_height();

    for (i, row) in jbox.rows.iter().enumerate() {
        let text_y = row.y_top + bl;

        if let Some(ref key) = row.key {
            let key_x = x + PADDING;
            let key_tl = font_metrics::text_width(key, "SansSerif", FONT_SIZE, true, false);
            sg.set_fill_color(TEXT_COLOR);
            sg.svg_text(
                key, key_x, text_y,
                Some("sans-serif"), FONT_SIZE,
                Some("700"), None, None,
                key_tl, LengthAdjust::Spacing,
                None, 0, None,
            );
        }

        let val_x = if has_keys { jbox.separator_x + PADDING } else { x + PADDING };
        for (li, line) in row.value_lines.iter().enumerate() {
            let line_y = text_y + li as f64 * lh;
            let val_tl = font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false);
            sg.set_fill_color(TEXT_COLOR);
            sg.svg_text(
                line, val_x, line_y,
                Some("sans-serif"), FONT_SIZE,
                None, None, None,
                val_tl, LengthAdjust::Spacing,
                None, 0, None,
            );
        }

        if has_keys {
            sg.set_stroke_color(Some(BORDER_COLOR));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(jbox.separator_x, row.y_top, jbox.separator_x, row.y_top + row.height, 0.0);
        }

        if i < jbox.rows.len() - 1 {
            let ly = row.y_top + row.height;
            sg.set_stroke_color(Some(BORDER_COLOR));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(x, ly, x + w, ly, 0.0);
        }
    }

    // Border rect
    sg.set_fill_color("none");
    sg.set_stroke_color(Some(BORDER_COLOR));
    sg.set_stroke_width(1.5, None);
    sg.svg_rectangle(x, y, w, h, 5.0, 5.0, 0.0);
}

fn render_arrow(sg: &mut SvgGraphic, arrow: &JsonArrow) {
    let (fx, fy, tx, ty) = (arrow.from_x, arrow.from_y, arrow.to_x, arrow.to_y);
    let mid_x = (fx + tx) / 2.0;
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} C{},{} {},{} {},{}" fill="none" style="stroke:{BORDER_COLOR};stroke-width:1;stroke-dasharray:3,3;"/>"#,
        fmt_coord(fx), fmt_coord(fy), fmt_coord(fx + 13.0), fmt_coord(fy),
        fmt_coord(mid_x), fmt_coord(fy), fmt_coord(mid_x), fmt_coord(ty),
        fmt_coord(tx - 7.0), fmt_coord(ty)));

    let sz = 3.1073;
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{}" fill="{BORDER_COLOR}"/>"#,
        fmt_coord(tx - 7.0), fmt_coord(ty + sz), fmt_coord(tx - 5.0), fmt_coord(ty),
        fmt_coord(tx - 7.0), fmt_coord(ty - sz), fmt_coord(tx), fmt_coord(ty),
        fmt_coord(tx - 7.0), fmt_coord(ty + sz)));
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
