use crate::font_metrics;
use crate::klimt::drawable::{
    DrawStyle, Drawable, EllipseShape, LineShape, PolygonShape, RectShape, TextShape,
};
use crate::klimt::svg::SvgGraphic;
use crate::layout::bpm::{BpmCellLayout, BpmLayout};
use crate::model::bpm::{BpmDiagram, BpmElementType, Where};
use crate::render::svg::{ensure_visible_int, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

/// Java BpmElement connector line length (10px).
const CONNECTOR_LEN: f64 = 10.0;

/// Start circle fill (#F1F1F1) and stroke (#181818 0.5).
const START_FILL: &str = "#F1F1F1";
const START_STROKE: &str = "#181818";
const START_STROKE_WIDTH: f64 = 0.5;
const START_RADIUS: f64 = 10.0;

/// Diamond fill (#FEFECE) and stroke (#A80036 0.5).
const DIAMOND_FILL: &str = "#FEFECE";
const DIAMOND_STROKE: &str = "#A80036";
const DIAMOND_STROKE_WIDTH: f64 = 0.5;
const DIAMOND_HALF: f64 = 12.0;

/// Task box fill (#F1F1F1) and stroke (#181818 0.5).
const BOX_FILL: &str = "#F1F1F1";
const BOX_STROKE: &str = "#181818";
const BOX_STROKE_WIDTH: f64 = 0.5;
const BOX_CORNER_RADIUS: f64 = 12.5;
const BOX_FONT_SIZE: f64 = 12.0;
const BOX_TEXT_COLOR: &str = "#000000";

/// Connector line colors.
const CONNECTOR_RED: &str = "#FF0000";
const CONNECTOR_BLUE: &str = "#0000FF";

/// Grid line color.
const GRID_COLOR: &str = "#000000";

/// Java ImageBuilder margin (10px shift on all sides).
const MARGIN: f64 = 10.0;

pub fn render_bpm(_d: &BpmDiagram, l: &BpmLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(8192);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    // SVG dimensions: Java's ImageBuilder adds margin(10) on all 4 sides
    // (TitledDiagram.getDefaultMargins = same(10)), then SvgGraphics.ensureVisible
    // truncates (int)(x+1). The +1 accounts for Java's LimitFinder rounding
    // during the draw pass that pushes maxX/maxY one pixel beyond the minDim.
    let sw = ensure_visible_int(l.width + MARGIN * 2.0 + 1.0) as f64;
    let sh = ensure_visible_int(l.height + MARGIN * 2.0 + 1.0) as f64;
    write_svg_root_bg(&mut buf, sw, sh, "BPM", bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);
    let grid_style = DrawStyle::outline(GRID_COLOR, 1.0);

    // Draw internal grid lines (shifted by MARGIN)
    for gl in &l.grid_lines {
        LineShape {
            x1: gl.x1 + MARGIN,
            y1: gl.y1 + MARGIN,
            x2: gl.x2 + MARGIN,
            y2: gl.y2 + MARGIN,
        }
        .draw(&mut sg, &grid_style);
    }

    // Render cells in grid order (row, col) — matching Java GridArray.drawU
    // which iterates lines then cols. We interleave elements and connectors.
    // Build a sorted list of (row, col, is_element, index).
    let mut render_order: Vec<(usize, usize, bool, usize)> = Vec::new();
    for (i, cell) in l.cells.iter().enumerate() {
        render_order.push((cell.row, cell.col, true, i));
    }
    for (i, conn) in l.connectors.iter().enumerate() {
        render_order.push((conn.row, conn.col, false, i));
    }
    render_order.sort_by_key(|&(r, c, is_elem, _)| (r, c, !is_elem));

    for &(_, _, is_element, idx) in &render_order {
        if is_element {
            let cell = &l.cells[idx];
            let shifted = BpmCellLayout {
                x: cell.x + MARGIN,
                y: cell.y + MARGIN,
                ..cell.clone()
            };
            render_element(&mut sg, &shifted);
        } else {
            let conn = &l.connectors[idx];
            render_connector_puzzle(&mut sg, conn);
        }
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_element(sg: &mut SvgGraphic, cell: &BpmCellLayout) {
    let cx = cell.x + cell.width / 2.0;
    let cy = cell.y + cell.height / 2.0;

    match cell.element_type {
        BpmElementType::Start => {
            EllipseShape {
                cx, cy, rx: START_RADIUS, ry: START_RADIUS,
            }
            .draw(sg, &DrawStyle::filled(START_FILL, START_STROKE, START_STROKE_WIDTH));
        }
        BpmElementType::Merge => {
            let top = (cx, cy - DIAMOND_HALF);
            let right = (cx + DIAMOND_HALF, cy);
            let bottom = (cx, cy + DIAMOND_HALF);
            let left = (cx - DIAMOND_HALF, cy);
            PolygonShape {
                points: vec![
                    top.0, top.1, right.0, right.1,
                    bottom.0, bottom.1, left.0, left.1,
                    top.0, top.1,
                ],
            }
            .draw(sg, &DrawStyle::filled(DIAMOND_FILL, DIAMOND_STROKE, DIAMOND_STROKE_WIDTH));
        }
        BpmElementType::DockedEvent => {
            RectShape {
                x: cell.x, y: cell.y, w: cell.width, h: cell.height,
                rx: BOX_CORNER_RADIUS, ry: BOX_CORNER_RADIUS,
            }
            .draw(sg, &DrawStyle::filled(BOX_FILL, BOX_STROKE, BOX_STROKE_WIDTH));

            if let Some(ref label) = cell.label {
                let ascent = font_metrics::ascent("SansSerif", BOX_FONT_SIZE, false, false);
                let tw = font_metrics::text_width(label, "SansSerif", BOX_FONT_SIZE, false, false);
                let text_x = cell.x + 10.0; // padding_left
                let text_y = cell.y + 10.0 + ascent; // padding_top + ascent
                TextShape {
                    x: text_x,
                    y: text_y,
                    text: label.clone(),
                    font_family: "sans-serif".into(),
                    font_size: BOX_FONT_SIZE,
                    text_length: tw,
                    bold: false,
                    italic: false,
                }
                .draw(sg, &DrawStyle::fill_only(BOX_TEXT_COLOR));
            }
        }
        BpmElementType::End => {
            EllipseShape {
                cx, cy, rx: START_RADIUS, ry: START_RADIUS,
            }
            .draw(sg, &DrawStyle::filled(START_FILL, START_STROKE, START_STROKE_WIDTH * 3.0));
        }
    }

    // Draw connector lines on the element in Java Where enum order: N, E, S, W
    let nesw_order = [Where::North, Where::East, Where::South, Where::West];
    for dir in &nesw_order {
        if cell.connectors.contains(dir) {
            draw_connector_line(sg, cx, cy, cell.width, cell.height, *dir, false);
        }
    }
}

fn render_connector_puzzle(sg: &mut SvgGraphic, conn: &crate::layout::bpm::BpmConnectorLayout) {
    // Java ConnectorPuzzleEmpty: 20x20 cell, draws blue lines at specific offsets.
    let ox = conn.x + MARGIN; // puzzle origin x
    let oy = conn.y + MARGIN; // puzzle origin y
    let blue_style = DrawStyle::outline(CONNECTOR_BLUE, 1.0);

    let nesw_order = [Where::North, Where::East, Where::South, Where::West];
    for dir in &nesw_order {
        if !conn.directions.contains(dir) {
            continue;
        }
        let (x1, y1, x2, y2) = match dir {
            Where::West => (ox, oy + 10.0, ox + 10.0, oy + 10.0),
            Where::East => (ox + 10.0, oy + 10.0, ox + 20.0, oy + 10.0),
            Where::North => (ox + 10.0, oy, ox + 10.0, oy + 10.0),
            Where::South => (ox + 10.0, oy + 10.0, ox + 10.0, oy + 20.0),
        };
        LineShape { x1, y1, x2, y2 }.draw(sg, &blue_style);
    }
}

fn draw_connector_line(
    sg: &mut SvgGraphic,
    cx: f64,
    cy: f64,
    width: f64,
    height: f64,
    dir: Where,
    is_puzzle: bool,
) {
    let color = if is_puzzle { CONNECTOR_BLUE } else { CONNECTOR_RED };
    let (x1, y1, x2, y2) = match dir {
        Where::East => (cx + width / 2.0, cy, cx + width / 2.0 + CONNECTOR_LEN, cy),
        Where::West => (cx - width / 2.0 - CONNECTOR_LEN, cy, cx - width / 2.0, cy),
        Where::North => (cx, cy - height / 2.0 - CONNECTOR_LEN, cx, cy - height / 2.0),
        Where::South => (cx, cy + height / 2.0, cx, cy + height / 2.0 + CONNECTOR_LEN),
    };

    LineShape { x1, y1, x2, y2 }.draw(sg, &DrawStyle::outline(color, 1.0));
}
