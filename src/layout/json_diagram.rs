use log::debug;

use crate::font_metrics;
use crate::model::json_diagram::{JsonDiagram, JsonValue};
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types — tree-table style matching Java PlantUML
// ---------------------------------------------------------------------------

/// A positioned box in the JSON tree-table layout.
#[derive(Debug, Clone)]
pub struct JsonBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rows: Vec<JsonBoxRow>,
    /// x coordinate of the vertical key/value separator line (absolute).
    pub separator_x: f64,
}

/// A single row inside a JsonBox.
#[derive(Debug, Clone)]
pub struct JsonBoxRow {
    pub key: Option<String>,
    pub value_lines: Vec<String>,
    pub has_child: bool,
    pub child_box_idx: Option<usize>,
    pub y_top: f64,
    pub height: f64,
}

/// An arrow connector between a parent box row and a child box.
#[derive(Debug, Clone)]
pub struct JsonArrow {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
}

/// Fully positioned JSON/YAML tree-table layout.
#[derive(Debug)]
pub struct JsonLayout {
    pub boxes: Vec<JsonBox>,
    pub arrows: Vec<JsonArrow>,
    pub width: f64,
    pub height: f64,
    /// Legacy field (kept for backward compat).
    pub rows: Vec<JsonRowLayout>,
}

/// Legacy row layout (kept for backward compat).
#[derive(Debug)]
pub struct JsonRowLayout {
    pub depth: usize,
    pub key: Option<String>,
    pub value: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub has_children: bool,
    pub connector_points: Vec<(f64, f64)>,
    pub is_header: bool,
}

// ---------------------------------------------------------------------------
// Constants — matching Java PlantUML JSON renderer
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 14.0;
const PADDING: f64 = 5.0;
const ROW_V_PAD: f64 = 2.0;
const MARGIN: f64 = 10.0;
const CHILD_GAP: f64 = 60.0;

fn text_w(text: &str, bold: bool) -> f64 {
    font_metrics::text_width(text, "SansSerif", FONT_SIZE, bold, false)
}

fn row_height() -> f64 {
    let asc = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let desc = font_metrics::descent("SansSerif", FONT_SIZE, false, false);
    asc + desc + 2.0 * ROW_V_PAD
}

fn line_height() -> f64 {
    font_metrics::line_height("SansSerif", FONT_SIZE, false, false)
}

fn baseline_offset() -> f64 {
    font_metrics::ascent("SansSerif", FONT_SIZE, false, false) + ROW_V_PAD
}

// ---------------------------------------------------------------------------
// Intermediate structures
// ---------------------------------------------------------------------------

struct BoxRowSpec {
    key: Option<String>,
    value_lines: Vec<String>,
    has_child: bool,
    child_spec_idx: Option<usize>,
}

struct BoxSpec {
    rows: Vec<BoxRowSpec>,
    max_key_w: f64,
    max_val_w: f64,
}

fn build_box_spec(value: &JsonValue, specs: &mut Vec<BoxSpec>) -> usize {
    let idx = specs.len();
    specs.push(BoxSpec { rows: vec![], max_key_w: 0.0, max_val_w: 0.0 });

    match value {
        JsonValue::Object(entries) => {
            for (key, val) in entries {
                let key_w = text_w(key, true);
                specs[idx].max_key_w = specs[idx].max_key_w.max(key_w);
                if val.is_container() {
                    let child_idx = build_box_spec(val, specs);
                    let placeholder = "\u{00A0}\u{00A0}\u{00A0}";
                    specs[idx].max_val_w = specs[idx].max_val_w.max(text_w(placeholder, false));
                    specs[idx].rows.push(BoxRowSpec {
                        key: Some(key.clone()), value_lines: vec![placeholder.to_string()],
                        has_child: true, child_spec_idx: Some(child_idx),
                    });
                } else {
                    let (display, lines) = format_leaf_value(val);
                    for line in &lines { specs[idx].max_val_w = specs[idx].max_val_w.max(text_w(line, false)); }
                    if lines.is_empty() { specs[idx].max_val_w = specs[idx].max_val_w.max(text_w(&display, false)); }
                    specs[idx].rows.push(BoxRowSpec {
                        key: Some(key.clone()),
                        value_lines: if lines.is_empty() { vec![display] } else { lines },
                        has_child: false, child_spec_idx: None,
                    });
                }
            }
        }
        JsonValue::Array(items) => {
            for item in items {
                if item.is_container() {
                    let child_idx = build_box_spec(item, specs);
                    let placeholder = "\u{00A0}\u{00A0}\u{00A0}";
                    specs[idx].max_val_w = specs[idx].max_val_w.max(text_w(placeholder, false));
                    specs[idx].rows.push(BoxRowSpec {
                        key: None, value_lines: vec![placeholder.to_string()],
                        has_child: true, child_spec_idx: Some(child_idx),
                    });
                } else {
                    let (display, _) = format_leaf_value(item);
                    specs[idx].max_val_w = specs[idx].max_val_w.max(text_w(&display, false));
                    specs[idx].rows.push(BoxRowSpec {
                        key: None, value_lines: vec![display],
                        has_child: false, child_spec_idx: None,
                    });
                }
            }
        }
        _ => {
            let (display, _) = format_leaf_value(value);
            specs[idx].max_val_w = specs[idx].max_val_w.max(text_w(&display, false));
            specs[idx].rows.push(BoxRowSpec {
                key: None, value_lines: vec![display],
                has_child: false, child_spec_idx: None,
            });
        }
    }
    idx
}

fn format_leaf_value(val: &JsonValue) -> (String, Vec<String>) {
    match val {
        JsonValue::Bool(true) => ("\u{2611} true".to_string(), vec![]),
        JsonValue::Bool(false) => ("\u{2610} false".to_string(), vec![]),
        JsonValue::Null => ("null".to_string(), vec![]),
        JsonValue::Number(n) => {
            if *n == (*n as i64) as f64 && n.is_finite() { (format!("{}", *n as i64), vec![]) }
            else { (format!("{n}"), vec![]) }
        }
        JsonValue::Str(s) => {
            if s.contains("\\n") || s.contains(crate::NEWLINE_CHAR) {
                let lines: Vec<String> = s.split("\\n").flat_map(|l| l.split(crate::NEWLINE_CHAR)).map(|l| l.to_string()).collect();
                (s.clone(), lines)
            } else { (s.clone(), vec![]) }
        }
        _ => (val.display_value(), vec![]),
    }
}

fn row_spec_height(row: &BoxRowSpec) -> f64 {
    let rh = row_height();
    let lh = line_height();
    let n = row.value_lines.len().max(1);
    if n <= 1 { rh } else { baseline_offset() + (n as f64 - 1.0) * lh + (rh - baseline_offset()) }
}

fn box_spec_height(spec: &BoxSpec) -> f64 { spec.rows.iter().map(row_spec_height).sum() }

fn box_spec_width(spec: &BoxSpec) -> f64 {
    let has_keys = spec.rows.iter().any(|r| r.key.is_some());
    if has_keys { PADDING + spec.max_key_w + PADDING + PADDING + spec.max_val_w + PADDING }
    else { PADDING + spec.max_val_w + PADDING }
}

// ---------------------------------------------------------------------------
// Positioning
// ---------------------------------------------------------------------------

fn position_boxes(
    spec_idx: usize, specs: &[BoxSpec], x: f64, y: f64,
    boxes: &mut Vec<JsonBox>, arrows: &mut Vec<JsonArrow>,
) -> (f64, f64) {
    let spec = &specs[spec_idx];
    let box_w = box_spec_width(spec);
    let box_h = box_spec_height(spec);
    let has_keys = spec.rows.iter().any(|r| r.key.is_some());
    let sep_x = if has_keys { x + PADDING + spec.max_key_w + PADDING } else { x };

    let box_idx = boxes.len();
    let mut jbox = JsonBox { x, y, width: box_w, height: box_h, rows: vec![], separator_x: sep_x };

    let mut row_y = y;
    for row_spec in &spec.rows {
        let rh = row_spec_height(row_spec);
        jbox.rows.push(JsonBoxRow {
            key: row_spec.key.clone(), value_lines: row_spec.value_lines.clone(),
            has_child: row_spec.has_child, child_box_idx: None, y_top: row_y, height: rh,
        });
        row_y += rh;
    }
    boxes.push(jbox);

    let mut max_right = x + box_w;
    let mut max_bottom = y + box_h;
    let mut child_y_cursor = y;

    for (i, row_spec) in spec.rows.iter().enumerate() {
        let rh = row_spec_height(row_spec);
        if let Some(child_spec_idx) = row_spec.child_spec_idx {
            let child_x = x + box_w + CHILD_GAP;
            let row_center_y = child_y_cursor + rh / 2.0;

            let child_h = box_spec_height(&specs[child_spec_idx]);
            let child_y = (row_center_y - child_h / 2.0).max(child_y_cursor);

            let (cr, cb) = position_boxes(child_spec_idx, specs, child_x, child_y, boxes, arrows);

            let child_box = &boxes[boxes.len() - count_subtree_boxes(child_spec_idx, specs)];
            let child_center_y = child_box.y + child_box.height / 2.0;
            arrows.push(JsonArrow { from_x: x + box_w, from_y: row_center_y, to_x: child_x, to_y: child_center_y });

            max_right = max_right.max(cr);
            max_bottom = max_bottom.max(cb);
        }
        child_y_cursor += rh;
    }

    (max_right, max_bottom)
}

fn count_subtree_boxes(spec_idx: usize, specs: &[BoxSpec]) -> usize {
    let mut count = 1;
    for row in &specs[spec_idx].rows {
        if let Some(ci) = row.child_spec_idx { count += count_subtree_boxes(ci, specs); }
    }
    count
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn layout_json(jd: &JsonDiagram) -> Result<JsonLayout> {
    debug!("layout_json: root type = {}", jd.root.type_label());

    if !jd.root.is_container() {
        let (display, _) = format_leaf_value(&jd.root);
        let w = text_w(&display, false) + 2.0 * PADDING + 2.0 * MARGIN;
        let h = row_height() + 2.0 * MARGIN;
        return Ok(JsonLayout {
            boxes: vec![JsonBox {
                x: MARGIN, y: MARGIN, width: w - 2.0 * MARGIN, height: h - 2.0 * MARGIN,
                rows: vec![JsonBoxRow {
                    key: None, value_lines: vec![display], has_child: false,
                    child_box_idx: None, y_top: MARGIN, height: row_height(),
                }],
                separator_x: MARGIN,
            }],
            arrows: vec![], width: w, height: h, rows: vec![],
        });
    }

    let mut specs: Vec<BoxSpec> = Vec::new();
    build_box_spec(&jd.root, &mut specs);

    let mut boxes: Vec<JsonBox> = Vec::new();
    let mut arrows: Vec<JsonArrow> = Vec::new();
    let (max_right, max_bottom) = position_boxes(0, &specs, MARGIN, MARGIN, &mut boxes, &mut arrows);

    let width = max_right + MARGIN;
    let height = max_bottom + MARGIN;

    debug!("layout_json: {} boxes, {} arrows, {:.0}x{:.0}", boxes.len(), arrows.len(), width, height);
    Ok(JsonLayout { boxes, arrows, width, height, rows: vec![] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::json_diagram::{JsonDiagram, JsonValue};

    #[test]
    fn test_simple_object() {
        let jd = JsonDiagram { root: JsonValue::Object(vec![
            ("a".into(), JsonValue::Bool(true)),
            ("b".into(), JsonValue::Number(42.0)),
        ]) };
        let layout = layout_json(&jd).unwrap();
        assert!(!layout.boxes.is_empty());
        assert_eq!(layout.boxes[0].rows.len(), 2);
    }

    #[test]
    fn test_nested_creates_child_boxes() {
        let jd = JsonDiagram { root: JsonValue::Object(vec![
            ("items".into(), JsonValue::Array(vec![JsonValue::Str("x".into())])),
        ]) };
        let layout = layout_json(&jd).unwrap();
        assert!(layout.boxes.len() >= 2);
        assert!(!layout.arrows.is_empty());
    }

    #[test]
    fn test_leaf_root() {
        let jd = JsonDiagram { root: JsonValue::Number(42.0) };
        let layout = layout_json(&jd).unwrap();
        assert!(!layout.boxes.is_empty());
    }

    #[test]
    fn test_escaped_newline_value_produces_multiline() {
        // desc: "a\nb\nc\nd\ne\nf" should produce 6 value lines
        let jd = JsonDiagram { root: JsonValue::Object(vec![
            ("desc".into(), JsonValue::Str("a\\nb\\nc\\nd\\ne\\nf".into())),
        ]) };
        let layout = layout_json(&jd).unwrap();
        assert_eq!(layout.boxes[0].rows[0].value_lines.len(), 6,
            "Expected 6 value lines, got: {:?}", layout.boxes[0].rows[0].value_lines);
    }
}
