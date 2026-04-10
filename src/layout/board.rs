use crate::font_metrics;
use crate::model::board::BoardDiagram;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const MARGIN: f64 = 10.0;
const COL_GAP: f64 = 10.0;
const ROW_GAP: f64 = 6.0;
const CARD_PAD_H: f64 = 8.0;
const CARD_PAD_V: f64 = 6.0;
const HEADER_H: f64 = 24.0;
const MIN_COL_W: f64 = 120.0;

/// Layout for a single card on the board.
#[derive(Debug, Clone)]
pub struct BoardCardLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub label: String,
    pub level: usize,
}

/// Layout for a board column.
#[derive(Debug, Clone)]
pub struct BoardColumnLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub header: String,
    pub cards: Vec<BoardCardLayout>,
}

/// Full board layout.
#[derive(Debug)]
pub struct BoardLayout {
    pub width: f64,
    pub height: f64,
    pub columns: Vec<BoardColumnLayout>,
}

pub fn layout_board(d: &BoardDiagram) -> Result<BoardLayout> {
    let card_h = font_metrics::line_height("SansSerif", FONT_SIZE, false, false) + CARD_PAD_V * 2.0;

    let mut columns = Vec::new();
    let mut x = MARGIN;

    for task in &d.tasks {
        let header = task.label.clone();
        let header_w = font_metrics::text_width(&header, "SansSerif", FONT_SIZE, true, false)
            + CARD_PAD_H * 2.0;
        let mut cards = Vec::new();
        let mut max_w = header_w.max(MIN_COL_W);
        let mut cy = MARGIN + HEADER_H + ROW_GAP;

        for child in &task.children {
            let cw = font_metrics::text_width(&child.label, "SansSerif", FONT_SIZE, false, false)
                + CARD_PAD_H * 2.0;
            max_w = max_w.max(cw);
            cards.push(BoardCardLayout {
                x: 0.0, // will be set after column width is known
                y: cy,
                width: 0.0,
                height: card_h,
                label: child.label.clone(),
                level: child.level,
            });
            cy += card_h + ROW_GAP;
        }

        let col_w = max_w;
        let col_h = cy;

        // Update card positions with actual column x and width
        for card in &mut cards {
            card.x = x;
            card.width = col_w;
        }

        columns.push(BoardColumnLayout {
            x,
            y: MARGIN,
            width: col_w,
            height: col_h,
            header,
            cards,
        });

        x += col_w + COL_GAP;
    }

    let total_w = x - COL_GAP + MARGIN;
    let total_h = columns.iter().map(|c| c.height).fold(0.0_f64, f64::max) + MARGIN;

    Ok(BoardLayout {
        width: total_w.max(100.0),
        height: total_h.max(60.0),
        columns,
    })
}
