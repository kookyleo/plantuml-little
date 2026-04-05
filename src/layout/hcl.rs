use crate::font_metrics;
use crate::model::hcl::HclDiagram;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const PADDING: f64 = 5.0;
const ROW_V_PAD: f64 = 2.0;
const MARGIN: f64 = 10.0;

fn text_w(text: &str, bold: bool) -> f64 {
    font_metrics::text_width(text, "SansSerif", FONT_SIZE, bold, false)
}

fn row_h() -> f64 {
    font_metrics::line_height("SansSerif", FONT_SIZE, false, false) + ROW_V_PAD * 2.0
}

/// Layout for an HCL row (key-value pair).
#[derive(Debug, Clone)]
pub struct HclRowLayout {
    pub key: String,
    pub value: String,
    pub y_top: f64,
    pub height: f64,
}

/// Full HCL diagram layout, modeled after JSON tree-table.
#[derive(Debug)]
pub struct HclLayout {
    pub width: f64,
    pub height: f64,
    pub box_x: f64,
    pub box_y: f64,
    pub box_w: f64,
    pub box_h: f64,
    pub separator_x: f64,
    pub rows: Vec<HclRowLayout>,
}

pub fn layout_hcl(d: &HclDiagram) -> Result<HclLayout> {
    let rh = row_h();
    let n = d.entries.len();

    // Compute max key width and max value width
    let max_key_w = d
        .entries
        .iter()
        .map(|e| text_w(&e.key, true))
        .fold(0.0_f64, f64::max);
    let max_val_w = d
        .entries
        .iter()
        .map(|e| text_w(&e.value, false))
        .fold(0.0_f64, f64::max);

    // Box dimensions matching Java JSON tree-table style
    let sep_offset = PADDING + max_key_w + PADDING;
    let box_w = sep_offset + PADDING + max_val_w + PADDING;
    let box_h = n as f64 * rh;

    let box_x = MARGIN;
    let box_y = MARGIN;
    let separator_x = box_x + sep_offset;

    let mut rows = Vec::new();
    for (i, entry) in d.entries.iter().enumerate() {
        rows.push(HclRowLayout {
            key: entry.key.clone(),
            value: entry.value.clone(),
            y_top: box_y + i as f64 * rh,
            height: rh,
        });
    }

    // +1 accounts for the border stroke (1.5px → ⌈0.75⌉ = 1px overshoot)
    // matching Java's LimitFinder which tracks stroke extents.
    let width = MARGIN + box_w + MARGIN + 1.0;
    let height = MARGIN + box_h + MARGIN + 1.0;

    Ok(HclLayout {
        width,
        height,
        box_x,
        box_y,
        box_w,
        box_h,
        separator_x,
        rows,
    })
}
