use log::{debug, trace};

use crate::model::json_diagram::{JsonDiagram, JsonValue};
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned JSON tree-table, ready for SVG rendering.
#[derive(Debug)]
pub struct JsonLayout {
    pub rows: Vec<JsonRowLayout>,
    pub width: f64,
    pub height: f64,
}

/// One visual row in the flattened JSON tree-table.
#[derive(Debug)]
pub struct JsonRowLayout {
    /// Nesting depth (0 = root level).
    pub depth: usize,
    /// Key label (None for array items and structural rows like `{`, `}`).
    pub key: Option<String>,
    /// Value text to display.
    pub value: String,
    /// Absolute x position.
    pub x: f64,
    /// Absolute y position.
    pub y: f64,
    /// Row width.
    pub width: f64,
    /// Row height.
    pub height: f64,
    /// Whether this row represents a container that has children.
    pub has_children: bool,
    /// Connection points for parent-to-child lines: (x, y) pairs.
    pub connector_points: Vec<(f64, f64)>,
    /// True if this row is a structural bracket/header (e.g., `{`, `}`, `[`, `]`).
    pub is_header: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const ROW_HEIGHT: f64 = 24.0;
const CHAR_WIDTH: f64 = 7.2;
const INDENT_PX: f64 = 20.0;
const PADDING_H: f64 = 10.0;
const MIN_KEY_WIDTH: f64 = 40.0;
const MIN_VAL_WIDTH: f64 = 60.0;
const MARGIN: f64 = 10.0;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

fn text_width(text: &str) -> f64 {
    text.len() as f64 * CHAR_WIDTH
}

// ---------------------------------------------------------------------------
// Flattening: recursive JSON -> flat row list
// ---------------------------------------------------------------------------

/// Intermediate row before position assignment.
#[derive(Debug)]
struct FlatRow {
    depth: usize,
    key: Option<String>,
    value: String,
    has_children: bool,
    is_header: bool,
}

/// Recursively flatten a JsonValue into display rows.
fn flatten_value(value: &JsonValue, key: Option<&str>, depth: usize, rows: &mut Vec<FlatRow>) {
    match value {
        JsonValue::Object(entries) => {
            // Opening header row
            let label = match key {
                Some(k) => k.to_string(),
                None => String::new(),
            };
            rows.push(FlatRow {
                depth,
                key: if label.is_empty() { None } else { Some(label) },
                value: "{".to_string(),
                has_children: !entries.is_empty(),
                is_header: true,
            });

            for (k, v) in entries {
                if v.is_container() {
                    flatten_value(v, Some(k), depth + 1, rows);
                } else {
                    rows.push(FlatRow {
                        depth: depth + 1,
                        key: Some(k.clone()),
                        value: v.display_value(),
                        has_children: false,
                        is_header: false,
                    });
                }
            }

            // Closing brace row
            rows.push(FlatRow {
                depth,
                key: None,
                value: "}".to_string(),
                has_children: false,
                is_header: true,
            });
        }
        JsonValue::Array(items) => {
            let label = match key {
                Some(k) => k.to_string(),
                None => String::new(),
            };
            rows.push(FlatRow {
                depth,
                key: if label.is_empty() { None } else { Some(label) },
                value: "[".to_string(),
                has_children: !items.is_empty(),
                is_header: true,
            });

            for (i, item) in items.iter().enumerate() {
                let idx_key = format!("{i}");
                if item.is_container() {
                    flatten_value(item, Some(&idx_key), depth + 1, rows);
                } else {
                    rows.push(FlatRow {
                        depth: depth + 1,
                        key: Some(idx_key),
                        value: item.display_value(),
                        has_children: false,
                        is_header: false,
                    });
                }
            }

            rows.push(FlatRow {
                depth,
                key: None,
                value: "]".to_string(),
                has_children: false,
                is_header: true,
            });
        }
        // Leaf values at root level (unusual but valid)
        _ => {
            rows.push(FlatRow {
                depth,
                key: key.map(std::string::ToString::to_string),
                value: value.display_value(),
                has_children: false,
                is_header: false,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Compute the tree-table layout for a JSON diagram.
pub fn layout_json(jd: &JsonDiagram) -> Result<JsonLayout> {
    debug!("layout_json: root type = {}", jd.root.type_label());

    // Step 1: flatten the JSON tree into rows
    let mut flat_rows: Vec<FlatRow> = Vec::new();
    flatten_value(&jd.root, None, 0, &mut flat_rows);

    debug!("layout_json: {} flat rows", flat_rows.len());

    if flat_rows.is_empty() {
        return Ok(JsonLayout {
            rows: Vec::new(),
            width: 2.0 * MARGIN,
            height: 2.0 * MARGIN,
        });
    }

    // Step 2: compute column widths
    let mut max_key_width: f64 = MIN_KEY_WIDTH;
    let mut max_val_width: f64 = MIN_VAL_WIDTH;

    for row in &flat_rows {
        let key_w = match &row.key {
            Some(k) => text_width(k) + row.depth as f64 * INDENT_PX + 2.0 * PADDING_H,
            None => row.depth as f64 * INDENT_PX + 2.0 * PADDING_H,
        };
        max_key_width = max_key_width.max(key_w);

        let val_w = text_width(&row.value) + 2.0 * PADDING_H;
        max_val_width = max_val_width.max(val_w);
    }

    let total_width = max_key_width + max_val_width;

    // Step 3: assign positions to each row
    let mut layout_rows: Vec<JsonRowLayout> = Vec::new();
    let mut y_cursor = MARGIN;

    for (i, flat) in flat_rows.iter().enumerate() {
        let x = MARGIN;
        let y = y_cursor;
        let w = total_width;
        let h = ROW_HEIGHT;

        // Build connector points: from the indented key area to the row below (if applicable)
        let mut connectors = Vec::new();
        if flat.has_children {
            let cx = x + flat.depth as f64 * INDENT_PX + PADDING_H;
            let cy = y + h;
            connectors.push((cx, cy));
        }

        trace!(
            "layout_json: row[{}] depth={} key={:?} value={:?} @ ({:.1}, {:.1})",
            i,
            flat.depth,
            flat.key,
            flat.value,
            x,
            y,
        );

        layout_rows.push(JsonRowLayout {
            depth: flat.depth,
            key: flat.key.clone(),
            value: flat.value.clone(),
            x,
            y,
            width: w,
            height: h,
            has_children: flat.has_children,
            connector_points: connectors,
            is_header: flat.is_header,
        });

        y_cursor += ROW_HEIGHT;
    }

    let total_height = y_cursor + MARGIN;
    let total_width = total_width + 2.0 * MARGIN;

    debug!(
        "layout_json done: {:.0}x{:.0}, {} rows",
        total_width,
        total_height,
        layout_rows.len()
    );

    Ok(JsonLayout {
        rows: layout_rows,
        width: total_width,
        height: total_height,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::json_diagram::{JsonDiagram, JsonValue};

    fn make_diagram(root: JsonValue) -> JsonDiagram {
        JsonDiagram { root }
    }

    // 1. Empty object produces header rows
    #[test]
    fn test_empty_object_layout() {
        let jd = make_diagram(JsonValue::Object(vec![]));
        let layout = layout_json(&jd).unwrap();
        // "{" and "}" rows
        assert_eq!(layout.rows.len(), 2);
        assert!(layout.rows[0].is_header);
        assert_eq!(layout.rows[0].value, "{");
        assert_eq!(layout.rows[1].value, "}");
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 2. Simple key-value produces correct row count
    #[test]
    fn test_simple_kv() {
        let jd = make_diagram(JsonValue::Object(vec![(
            "name".into(),
            JsonValue::Str("Alice".into()),
        )]));
        let layout = layout_json(&jd).unwrap();
        // "{", "name: Alice", "}"
        assert_eq!(layout.rows.len(), 3);
        assert_eq!(layout.rows[1].key, Some("name".into()));
        assert_eq!(layout.rows[1].depth, 1);
        assert!(!layout.rows[1].is_header);
    }

    // 3. Multiple keys
    #[test]
    fn test_multiple_keys() {
        let jd = make_diagram(JsonValue::Object(vec![
            ("a".into(), JsonValue::Bool(true)),
            ("b".into(), JsonValue::Number(42.0)),
            ("c".into(), JsonValue::Null),
        ]));
        let layout = layout_json(&jd).unwrap();
        // "{", a, b, c, "}"
        assert_eq!(layout.rows.len(), 5);
        assert_eq!(layout.rows[1].key, Some("a".into()));
        assert_eq!(layout.rows[2].key, Some("b".into()));
        assert_eq!(layout.rows[3].key, Some("c".into()));
    }

    // 4. Nested object indentation
    #[test]
    fn test_nested_object_depth() {
        let jd = make_diagram(JsonValue::Object(vec![(
            "outer".into(),
            JsonValue::Object(vec![("inner".into(), JsonValue::Str("val".into()))]),
        )]));
        let layout = layout_json(&jd).unwrap();

        // Root "{", outer "{", inner val, outer "}", root "}"
        assert_eq!(layout.rows.len(), 5);
        assert_eq!(layout.rows[0].depth, 0); // root {
        assert_eq!(layout.rows[1].depth, 1); // outer {
        assert_eq!(layout.rows[2].depth, 2); // inner: val
        assert_eq!(layout.rows[3].depth, 1); // outer }
        assert_eq!(layout.rows[4].depth, 0); // root }
    }

    // 5. Array items get index keys
    #[test]
    fn test_array_index_keys() {
        let jd = make_diagram(JsonValue::Array(vec![
            JsonValue::Str("a".into()),
            JsonValue::Str("b".into()),
            JsonValue::Str("c".into()),
        ]));
        let layout = layout_json(&jd).unwrap();

        // "[", 0: "a", 1: "b", 2: "c", "]"
        assert_eq!(layout.rows.len(), 5);
        assert_eq!(layout.rows[1].key, Some("0".into()));
        assert_eq!(layout.rows[2].key, Some("1".into()));
        assert_eq!(layout.rows[3].key, Some("2".into()));
    }

    // 6. Row height is consistent
    #[test]
    fn test_row_heights() {
        let jd = make_diagram(JsonValue::Object(vec![
            ("a".into(), JsonValue::Number(1.0)),
            ("b".into(), JsonValue::Number(2.0)),
        ]));
        let layout = layout_json(&jd).unwrap();
        for row in &layout.rows {
            assert_eq!(row.height, ROW_HEIGHT);
        }
    }

    // 7. Y positions increment correctly
    #[test]
    fn test_y_positions_sequential() {
        let jd = make_diagram(JsonValue::Object(vec![
            ("a".into(), JsonValue::Bool(true)),
            ("b".into(), JsonValue::Bool(false)),
        ]));
        let layout = layout_json(&jd).unwrap();
        for i in 1..layout.rows.len() {
            let expected_y = layout.rows[i - 1].y + ROW_HEIGHT;
            assert!(
                (layout.rows[i].y - expected_y).abs() < 0.01,
                "row {} y={} expected {}",
                i,
                layout.rows[i].y,
                expected_y,
            );
        }
    }

    // 8. Bounding box contains all rows
    #[test]
    fn test_bounding_box() {
        let jd = make_diagram(JsonValue::Object(vec![(
            "key".into(),
            JsonValue::Str("value".into()),
        )]));
        let layout = layout_json(&jd).unwrap();
        for row in &layout.rows {
            assert!(
                row.x + row.width <= layout.width,
                "row right edge {} exceeds layout width {}",
                row.x + row.width,
                layout.width,
            );
            assert!(
                row.y + row.height <= layout.height,
                "row bottom edge {} exceeds layout height {}",
                row.y + row.height,
                layout.height,
            );
        }
    }

    // 9. Container rows have has_children flag
    #[test]
    fn test_has_children_flag() {
        let jd = make_diagram(JsonValue::Object(vec![
            (
                "items".into(),
                JsonValue::Array(vec![JsonValue::Number(1.0)]),
            ),
            ("leaf".into(), JsonValue::Str("x".into())),
        ]));
        let layout = layout_json(&jd).unwrap();

        // The root "{" row has children
        assert!(layout.rows[0].has_children);

        // The "items" "[" row has children
        let items_header = layout
            .rows
            .iter()
            .find(|r| r.key == Some("items".into()) && r.value == "[")
            .unwrap();
        assert!(items_header.has_children);

        // Leaf rows do not have children
        let leaf_row = layout
            .rows
            .iter()
            .find(|r| r.key == Some("leaf".into()))
            .unwrap();
        assert!(!leaf_row.has_children);
    }

    // 10. Connector points present for container headers
    #[test]
    fn test_connector_points() {
        let jd = make_diagram(JsonValue::Object(vec![(
            "k".into(),
            JsonValue::Str("v".into()),
        )]));
        let layout = layout_json(&jd).unwrap();

        let header = &layout.rows[0]; // "{"
        assert!(header.has_children);
        assert!(!header.connector_points.is_empty());
    }

    // 11. Width accommodates long strings
    #[test]
    fn test_width_for_long_values() {
        let long_val = "a".repeat(100);
        let jd = make_diagram(JsonValue::Object(vec![(
            "k".into(),
            JsonValue::Str(long_val.clone()),
        )]));
        let layout = layout_json(&jd).unwrap();

        let display_val = format!("\"{}\"", long_val);
        let expected_min_width = text_width(&display_val) + 2.0 * PADDING_H;
        assert!(
            layout.width >= expected_min_width,
            "layout width {} should be >= {}",
            layout.width,
            expected_min_width,
        );
    }

    // 12. Leaf value at root level
    #[test]
    fn test_leaf_root() {
        let jd = make_diagram(JsonValue::Number(42.0));
        let layout = layout_json(&jd).unwrap();
        assert_eq!(layout.rows.len(), 1);
        assert_eq!(layout.rows[0].value, "42");
        assert_eq!(layout.rows[0].depth, 0);
    }

    // 13. Complex nested structure row count
    #[test]
    fn test_complex_structure_rows() {
        // { "a": true, "b": { "c": 1 }, "d": [2, 3] }
        let jd = make_diagram(JsonValue::Object(vec![
            ("a".into(), JsonValue::Bool(true)),
            (
                "b".into(),
                JsonValue::Object(vec![("c".into(), JsonValue::Number(1.0))]),
            ),
            (
                "d".into(),
                JsonValue::Array(vec![JsonValue::Number(2.0), JsonValue::Number(3.0)]),
            ),
        ]));
        let layout = layout_json(&jd).unwrap();
        // root: { a b{ c b} d[ 0 1 d] }
        // = 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 = 10 rows
        assert_eq!(layout.rows.len(), 10);
    }

    // 14. Empty array layout
    #[test]
    fn test_empty_array_layout() {
        let jd = make_diagram(JsonValue::Array(vec![]));
        let layout = layout_json(&jd).unwrap();
        assert_eq!(layout.rows.len(), 2);
        assert_eq!(layout.rows[0].value, "[");
        assert_eq!(layout.rows[1].value, "]");
    }

    // 15. Total height = margin + rows * ROW_HEIGHT + margin
    #[test]
    fn test_total_height() {
        let jd = make_diagram(JsonValue::Object(vec![(
            "a".into(),
            JsonValue::Number(1.0),
        )]));
        let layout = layout_json(&jd).unwrap();
        let expected = MARGIN + layout.rows.len() as f64 * ROW_HEIGHT + MARGIN;
        assert!(
            (layout.height - expected).abs() < 0.01,
            "height {} expected {}",
            layout.height,
            expected,
        );
    }
}
