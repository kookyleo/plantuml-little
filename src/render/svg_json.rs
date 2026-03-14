use std::fmt::Write;

use crate::layout::json_diagram::{JsonLayout, JsonRowLayout};
use crate::model::json_diagram::JsonDiagram;
use crate::render::svg::xml_escape;
use crate::render::svg::write_svg_root;
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants (PlantUML JSON theme)
// ---------------------------------------------------------------------------

const CELL_FILL: &str = "#F1F1F1";
const CELL_FILL_ALT: &str = "#F5F5DC";
const HEADER_FILL: &str = "#E2E2F0";
const BORDER_COLOR: &str = "#181818";
const TEXT_COLOR: &str = "#000000";
const CONNECTOR_COLOR: &str = "#181818";
const INDENT_PX: f64 = 20.0;
const PADDING_H: f64 = 10.0;
const TEXT_BASELINE_OFFSET: f64 = 16.0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render a JSON diagram to SVG.
pub fn render_json(_jd: &JsonDiagram, layout: &JsonLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // Skin color lookups
    let json_border = skin.border_color("json", BORDER_COLOR);
    let json_font = skin.font_color("json", TEXT_COLOR);
    let header_fill = skin.background_color("jsonHeader", HEADER_FILL);

    // SVG header
    write_svg_root(&mut buf, layout.width, layout.height);
    buf.push_str("<defs/><g>");

    // Render rows
    for (i, row) in layout.rows.iter().enumerate() {
        render_row(&mut buf, row, i, json_border, json_font, header_fill);
    }

    // Render connector lines (from parent headers to child rows)
    render_connectors(&mut buf, layout);

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Row rendering
// ---------------------------------------------------------------------------

fn render_row(
    buf: &mut String,
    row: &JsonRowLayout,
    index: usize,
    border_color: &str,
    font_color: &str,
    header_fill: &str,
) {
    let x = row.x;
    let y = row.y;
    let w = row.width;
    let h = row.height;

    // Determine fill color
    let fill = if row.is_header {
        header_fill
    } else if index.is_multiple_of(2) {
        CELL_FILL
    } else {
        CELL_FILL_ALT
    };

    // Background rectangle
    write!(
        buf,
        r#"<rect fill="{fill}" height="{h:.1}" style="stroke:{border_color};stroke-width:0.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
    )
    .unwrap();
    buf.push('\n');

    let text_y = y + TEXT_BASELINE_OFFSET;
    let indent = row.depth as f64 * INDENT_PX;

    if row.is_header {
        // Header row: show key (if present) + bracket
        let text_x = x + PADDING_H + indent;
        let mut label = String::new();
        if let Some(ref key) = row.key {
            label.push_str(key);
            label.push(' ');
        }
        label.push_str(&row.value);
        let escaped = xml_escape(&label);
        write!(
            buf,
            r#"<text x="{text_x:.1}" y="{text_y:.1}" font-weight="bold" fill="{font_color}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    } else {
        // Data row: key column (bold) and value column (normal)
        let key_x = x + PADDING_H + indent;

        if let Some(ref key) = row.key {
            let key_escaped = xml_escape(key);
            write!(
                buf,
                r#"<text x="{key_x:.1}" y="{text_y:.1}" font-weight="bold" fill="{font_color}">{key_escaped}</text>"#,
            )
            .unwrap();
            buf.push('\n');
        }

        // Value text: positioned after the key area
        // Use a fixed offset based on key column width
        let val_x = x + w * 0.5 + PADDING_H;
        let val_escaped = xml_escape(&row.value);
        write!(
            buf,
            r#"<text x="{val_x:.1}" y="{text_y:.1}" fill="{font_color}">{val_escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ---------------------------------------------------------------------------
// Connector lines
// ---------------------------------------------------------------------------

fn render_connectors(buf: &mut String, layout: &JsonLayout) {
    // Draw small vertical connector lines from container headers to their
    // content region.  We identify parent-child by depth: a header row at
    // depth d is followed by rows at depth d+1 until we see a closing
    // bracket at depth d.
    for row in &layout.rows {
        if !row.connector_points.is_empty() {
            for &(cx, cy) in &row.connector_points {
                // Short tick mark indicating children
                let y2 = cy + 4.0;
                write!(
                    buf,
                    r#"<line style="stroke:{CONNECTOR_COLOR};stroke-width:1;" x1="{cx:.1}" x2="{cx:.1}" y1="{cy:.1}" y2="{y2:.1}"/>"#,
                )
                .unwrap();
                buf.push('\n');
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::json_diagram::{layout_json, JsonLayout, JsonRowLayout};
    use crate::model::json_diagram::{JsonDiagram, JsonValue};
    use crate::style::SkinParams;

    fn empty_diagram() -> JsonDiagram {
        JsonDiagram {
            root: JsonValue::Object(vec![]),
        }
    }

    fn make_layout(rows: Vec<JsonRowLayout>, width: f64, height: f64) -> JsonLayout {
        JsonLayout {
            rows,
            width,
            height,
        }
    }

    fn make_row(
        depth: usize,
        key: Option<&str>,
        value: &str,
        y: f64,
        width: f64,
        is_header: bool,
    ) -> JsonRowLayout {
        JsonRowLayout {
            depth,
            key: key.map(|s| s.to_string()),
            value: value.to_string(),
            x: 10.0,
            y,
            width,
            height: 24.0,
            has_children: false,
            connector_points: vec![],
            is_header,
        }
    }

    // 1. Empty layout produces valid SVG
    #[test]
    fn test_empty_layout() {
        let jd = empty_diagram();
        let layout = make_layout(vec![], 100.0, 50.0);
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(!svg.contains("<rect"), "empty layout has no rects");
    }

    // 2. SVG has correct dimensions
    #[test]
    fn test_svg_dimensions() {
        let jd = empty_diagram();
        let layout = make_layout(vec![], 320.0, 240.0);
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("width=\"320px\""));
        assert!(svg.contains("height=\"240px\""));
        assert!(svg.contains("viewBox=\"0 0 320 240\""));
    }

    // 3. Header row uses header fill color
    #[test]
    fn test_header_fill_color() {
        let jd = empty_diagram();
        let layout = make_layout(vec![make_row(0, None, "{", 10.0, 200.0, true)], 220.0, 50.0);
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(&format!("fill=\"{}\"", HEADER_FILL)),
            "header row must use HEADER_FILL"
        );
    }

    // 4. Data row uses cell fill color
    #[test]
    fn test_data_row_fill() {
        let jd = empty_diagram();
        let layout = make_layout(
            vec![make_row(1, Some("key"), "\"value\"", 10.0, 200.0, false)],
            220.0,
            50.0,
        );
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        // First data row at index 0 (even) should use CELL_FILL
        assert!(
            svg.contains(&format!("fill=\"{}\"", CELL_FILL)),
            "data row must use CELL_FILL"
        );
    }

    // 5. Key text is bold
    #[test]
    fn test_key_bold() {
        let jd = empty_diagram();
        let layout = make_layout(
            vec![make_row(1, Some("name"), "\"Alice\"", 10.0, 200.0, false)],
            220.0,
            50.0,
        );
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("font-weight=\"bold\""),
            "key text must be bold"
        );
        assert!(svg.contains("name"), "key text must appear");
    }

    // 6. Value text appears in SVG
    #[test]
    fn test_value_text() {
        let jd = empty_diagram();
        let layout = make_layout(
            vec![make_row(1, Some("x"), "42", 10.0, 200.0, false)],
            220.0,
            50.0,
        );
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(">42<"), "value text must appear in SVG");
    }

    // 7. XML escaping in keys and values
    #[test]
    fn test_xml_escaping() {
        let jd = empty_diagram();
        let layout = make_layout(
            vec![make_row(0, Some("a&b"), "<val>", 10.0, 200.0, false)],
            220.0,
            50.0,
        );
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("a&amp;b"), "key with & must be XML-escaped");
        assert!(
            svg.contains("&lt;val&gt;"),
            "value with <> must be XML-escaped"
        );
    }

    // 8. Multiple rows have rects
    #[test]
    fn test_multiple_rows() {
        let jd = empty_diagram();
        let layout = make_layout(
            vec![
                make_row(0, None, "{", 10.0, 200.0, true),
                make_row(1, Some("a"), "true", 34.0, 200.0, false),
                make_row(1, Some("b"), "false", 58.0, 200.0, false),
                make_row(0, None, "}", 82.0, 200.0, true),
            ],
            220.0,
            120.0,
        );
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        let rect_count = svg.matches("<rect").count();
        assert_eq!(rect_count, 4, "4 rows should produce 4 rects");
    }

    // 9. Alternating colors for data rows
    #[test]
    fn test_alternating_colors() {
        let jd = empty_diagram();
        let layout = make_layout(
            vec![
                make_row(1, Some("a"), "1", 10.0, 200.0, false),
                make_row(1, Some("b"), "2", 34.0, 200.0, false),
            ],
            220.0,
            80.0,
        );
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        // Index 0 -> CELL_FILL, Index 1 -> CELL_FILL_ALT
        assert!(svg.contains(CELL_FILL), "first row uses CELL_FILL");
        assert!(svg.contains(CELL_FILL_ALT), "second row uses CELL_FILL_ALT");
    }

    // 10. Connector lines appear for container headers
    #[test]
    fn test_connector_lines() {
        let jd = empty_diagram();
        let mut header_row = make_row(0, None, "{", 10.0, 200.0, true);
        header_row.has_children = true;
        header_row.connector_points = vec![(20.0, 34.0)];

        let layout = make_layout(
            vec![
                header_row,
                make_row(1, Some("k"), "v", 34.0, 200.0, false),
                make_row(0, None, "}", 58.0, 200.0, true),
            ],
            220.0,
            100.0,
        );
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<line"),
            "connector line must be present for container headers"
        );
        assert!(
            svg.contains(&format!("stroke:{}", CONNECTOR_COLOR)),
            "connector must use CONNECTOR_COLOR"
        );
    }

    // 11. Full end-to-end: parse -> layout -> render
    #[test]
    fn test_end_to_end() {
        let src = r#"@startjson
{"name": "Alice", "age": 30}
@endjson"#;
        let jd = crate::parser::json_diagram::parse_json_diagram(src).unwrap();
        let layout = layout_json(&jd).unwrap();
        let svg = render_json(&jd, &layout, &SkinParams::default()).unwrap();

        assert!(svg.contains("<svg"), "must produce SVG");
        assert!(svg.contains("</svg>"), "must close SVG");
        assert!(svg.contains("name"), "key 'name' must appear");
        assert!(svg.contains("Alice"), "value 'Alice' must appear");
        assert!(svg.contains("age"), "key 'age' must appear");
        assert!(svg.contains("30"), "value '30' must appear");
    }

    // 12. Nested structure end-to-end
    #[test]
    fn test_nested_end_to_end() {
        let src = r#"@startjson
{
    "a": true,
    "b": {"c": 1},
    "d": [2, 3]
}
@endjson"#;
        let jd = crate::parser::json_diagram::parse_json_diagram(src).unwrap();
        let layout = layout_json(&jd).unwrap();
        let svg = render_json(&jd, &layout, &SkinParams::default()).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Check that structural brackets appear
        assert!(svg.contains("{"), "must show opening brace");
        assert!(svg.contains("}"), "must show closing brace");
        assert!(svg.contains("["), "must show opening bracket");
        assert!(svg.contains("]"), "must show closing bracket");
    }

    // 13. SVG root uses new format (no font-family/font-size on root)
    #[test]
    fn test_font_family() {
        let jd = empty_diagram();
        let layout = make_layout(vec![], 100.0, 50.0);
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("contentStyleType=\"text/css\""),
            "must have contentStyleType attribute"
        );
    }

    // 14. SVG root uses new format (no font-size on root, uses style attribute)
    #[test]
    fn test_font_size() {
        let jd = empty_diagram();
        let layout = make_layout(vec![], 100.0, 50.0);
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("zoomAndPan=\"magnify\""),
            "must have zoomAndPan attribute"
        );
    }

    // 15. Border color on rects
    #[test]
    fn test_border_color() {
        let jd = empty_diagram();
        let layout = make_layout(vec![make_row(0, None, "{", 10.0, 200.0, true)], 220.0, 50.0);
        let svg = render_json(&jd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(&format!("stroke:{}", BORDER_COLOR)),
            "rects must have BORDER_COLOR stroke"
        );
    }

    // 16. Full fixture test
    #[test]
    fn test_fixture_json_escaped() {
        let src = r#"@startjson
{
    "a": true,
    "desc": "a\\nb\\nc\\nd\\ne\\nf",
    "required": [
        "r1",
        "r2",
        "r3"
    ],
    "addP": false,
    "properties": {
        "P": "{ ... }"
    },
    "allOf": [
        "{ ... }",
        "{ ... }",
        "{ ... }"
    ]
}
@endjson"#;
        let jd = crate::parser::json_diagram::parse_json_diagram(src).unwrap();
        let layout = layout_json(&jd).unwrap();
        let svg = render_json(&jd, &layout, &SkinParams::default()).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("desc"));
        assert!(svg.contains("required"));
        assert!(svg.contains("properties"));
        assert!(svg.contains("allOf"));
        // Verify row count is non-trivial
        let rect_count = svg.matches("<rect").count();
        assert!(
            rect_count > 10,
            "fixture should produce many rows, got {}",
            rect_count
        );
    }
}
