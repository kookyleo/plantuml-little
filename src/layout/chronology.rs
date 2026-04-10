use crate::font_metrics;
use crate::model::chronology::ChronologyDiagram;
use crate::Result;

const FONT_SIZE: f64 = 12.0;
const DATE_FONT_SIZE: f64 = 11.0;
const MARGIN: f64 = 10.0;
const LINE_Y: f64 = 50.0;
const EVENT_SPACING: f64 = 150.0;
#[allow(dead_code)] // Java-ported layout constant
const MARKER_RADIUS: f64 = 6.0;
const LABEL_OFFSET_Y: f64 = -20.0;
const DATE_OFFSET_Y: f64 = 18.0;

/// Layout for a single chronology event.
#[derive(Debug, Clone)]
pub struct ChronologyEventLayout {
    pub x: f64,
    pub y: f64,
    pub date: String,
    pub label: String,
    pub label_x: f64,
    pub label_y: f64,
    pub date_x: f64,
    pub date_y: f64,
}

/// Full chronology layout.
#[derive(Debug)]
pub struct ChronologyLayout {
    pub width: f64,
    pub height: f64,
    pub line_y: f64,
    pub line_x1: f64,
    pub line_x2: f64,
    pub events: Vec<ChronologyEventLayout>,
}

pub fn layout_chronology(d: &ChronologyDiagram) -> Result<ChronologyLayout> {
    let n = d.events.len();
    if n == 0 {
        return Ok(ChronologyLayout {
            width: 100.0,
            height: 80.0,
            line_y: LINE_Y,
            line_x1: MARGIN,
            line_x2: 90.0,
            events: Vec::new(),
        });
    }

    let mut events = Vec::new();
    let mut max_x = 0.0_f64;

    for (i, ev) in d.events.iter().enumerate() {
        let x = MARGIN + 30.0 + i as f64 * EVENT_SPACING;
        let y = LINE_Y;

        let label_w = font_metrics::text_width(&ev.label, "SansSerif", FONT_SIZE, false, false);
        let date_w = font_metrics::text_width(&ev.date, "SansSerif", DATE_FONT_SIZE, false, false);

        let label_x = x - label_w / 2.0;
        let label_y = y + LABEL_OFFSET_Y;
        let date_x = x - date_w / 2.0;
        let date_y = y + DATE_OFFSET_Y;

        max_x = max_x.max(x + label_w / 2.0).max(x + date_w / 2.0);

        events.push(ChronologyEventLayout {
            x,
            y,
            date: ev.date.clone(),
            label: ev.label.clone(),
            label_x,
            label_y,
            date_x,
            date_y,
        });
    }

    let w = max_x + MARGIN + 30.0;
    let h = LINE_Y + 40.0;
    let line_x1 = MARGIN;
    let line_x2 = w - MARGIN;

    Ok(ChronologyLayout {
        width: w,
        height: h,
        line_y: LINE_Y,
        line_x1,
        line_x2,
        events,
    })
}
