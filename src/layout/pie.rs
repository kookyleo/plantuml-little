use crate::font_metrics;
use crate::model::pie::PieDiagram;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const MARGIN: f64 = 10.0;
const RADIUS: f64 = 80.0;
const LEGEND_GAP: f64 = 20.0;
const LEGEND_BOX: f64 = 12.0;
const LEGEND_SPACING: f64 = 18.0;
const TITLE_FONT_SIZE: f64 = 14.0;
const TITLE_GAP: f64 = 10.0;

/// Layout for a single pie slice.
#[derive(Debug, Clone)]
pub struct PieSliceLayout {
    pub start_angle: f64,
    pub end_angle: f64,
    pub label: String,
    pub value: f64,
    pub percentage: f64,
    pub color_index: usize,
}

/// Layout for a pie legend entry.
#[derive(Debug, Clone)]
pub struct PieLegendEntry {
    pub x: f64,
    pub y: f64,
    pub label: String,
    pub color_index: usize,
}

/// Full pie chart layout.
#[derive(Debug)]
pub struct PieLayout {
    pub width: f64,
    pub height: f64,
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
    pub slices: Vec<PieSliceLayout>,
    pub legend: Vec<PieLegendEntry>,
    pub title: Option<String>,
    pub title_x: f64,
    pub title_y: f64,
}

pub fn layout_pie(d: &PieDiagram) -> Result<PieLayout> {
    let total: f64 = d.slices.iter().map(|s| s.value).sum();
    let total = if total <= 0.0 { 1.0 } else { total };

    // Title height
    let title_h = if d.title.is_some() {
        font_metrics::line_height("SansSerif", TITLE_FONT_SIZE, true, false) + TITLE_GAP
    } else {
        0.0
    };

    let cx = MARGIN + RADIUS;
    let cy = MARGIN + title_h + RADIUS;

    // Build slice layouts
    let mut slices = Vec::new();
    let mut angle = 0.0_f64;
    for (i, s) in d.slices.iter().enumerate() {
        let pct = s.value / total * 100.0;
        let sweep = s.value / total * 360.0;
        slices.push(PieSliceLayout {
            start_angle: angle,
            end_angle: angle + sweep,
            label: s.label.clone(),
            value: s.value,
            percentage: pct,
            color_index: i,
        });
        angle += sweep;
    }

    // Legend layout (right side of pie)
    let legend_x = MARGIN + RADIUS * 2.0 + LEGEND_GAP;
    let mut legend = Vec::new();
    for (i, s) in d.slices.iter().enumerate() {
        let ly = cy - RADIUS + i as f64 * LEGEND_SPACING;
        legend.push(PieLegendEntry {
            x: legend_x,
            y: ly,
            label: s.label.clone(),
            color_index: i,
        });
    }

    // Compute total width/height
    let legend_max_w = d
        .slices
        .iter()
        .map(|s| {
            LEGEND_BOX
                + 6.0
                + font_metrics::text_width(&s.label, "SansSerif", FONT_SIZE, false, false)
        })
        .fold(0.0_f64, f64::max);
    let legend_w = if d.slices.is_empty() {
        0.0
    } else {
        LEGEND_GAP + legend_max_w
    };

    let w = MARGIN * 2.0 + RADIUS * 2.0 + legend_w;
    let h = MARGIN * 2.0 + title_h + RADIUS * 2.0;

    let title_x = w / 2.0;
    let title_y = MARGIN + font_metrics::ascent("SansSerif", TITLE_FONT_SIZE, true, false);

    Ok(PieLayout {
        width: w,
        height: h,
        cx,
        cy,
        radius: RADIUS,
        slices,
        legend,
        title: d.title.clone(),
        title_x,
        title_y,
    })
}
