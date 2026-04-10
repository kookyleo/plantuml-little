use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
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

    // Draw internal grid lines (shifted by MARGIN)
    for gl in &l.grid_lines {
        write!(
            buf,
            r#"<line style="stroke:{};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            GRID_COLOR,
            fmt_coord(gl.x1 + MARGIN),
            fmt_coord(gl.x2 + MARGIN),
            fmt_coord(gl.y1 + MARGIN),
            fmt_coord(gl.y2 + MARGIN),
        )
        .unwrap();
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
            render_element(&mut buf, &shifted);
        } else {
            let conn = &l.connectors[idx];
            render_connector_puzzle(&mut buf, conn);
        }
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_element(buf: &mut String, cell: &BpmCellLayout) {
    let cx = cell.x + cell.width / 2.0;
    let cy = cell.y + cell.height / 2.0;

    match cell.element_type {
        BpmElementType::Start => {
            // Draw filled ellipse
            write!(
                buf,
                r#"<ellipse cx="{}" cy="{}" fill="{}" rx="{}" ry="{}" style="stroke:{};stroke-width:{};"/>"#,
                fmt_coord(cx),
                fmt_coord(cy),
                START_FILL,
                fmt_coord(START_RADIUS),
                fmt_coord(START_RADIUS),
                START_STROKE,
                START_STROKE_WIDTH,
            )
            .unwrap();
        }
        BpmElementType::Merge => {
            // Draw diamond polygon
            let top = (cx, cy - DIAMOND_HALF);
            let right = (cx + DIAMOND_HALF, cy);
            let bottom = (cx, cy + DIAMOND_HALF);
            let left = (cx - DIAMOND_HALF, cy);
            write!(
                buf,
                r#"<polygon fill="{}" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{};stroke-width:{};"/>"#,
                DIAMOND_FILL,
                fmt_coord(top.0), fmt_coord(top.1),
                fmt_coord(right.0), fmt_coord(right.1),
                fmt_coord(bottom.0), fmt_coord(bottom.1),
                fmt_coord(left.0), fmt_coord(left.1),
                fmt_coord(top.0), fmt_coord(top.1),
                DIAMOND_STROKE,
                DIAMOND_STROKE_WIDTH,
            )
            .unwrap();
        }
        BpmElementType::DockedEvent => {
            // Draw rounded rect + text
            write!(
                buf,
                r#"<rect fill="{}" height="{}" rx="{}" ry="{}" style="stroke:{};stroke-width:{};" width="{}" x="{}" y="{}"/>"#,
                BOX_FILL,
                fmt_coord(cell.height),
                fmt_coord(BOX_CORNER_RADIUS),
                fmt_coord(BOX_CORNER_RADIUS),
                BOX_STROKE,
                BOX_STROKE_WIDTH,
                fmt_coord(cell.width),
                fmt_coord(cell.x),
                fmt_coord(cell.y),
            )
            .unwrap();

            if let Some(ref label) = cell.label {
                let ascent =
                    font_metrics::ascent("SansSerif", BOX_FONT_SIZE, false, false);
                let tw = font_metrics::text_width(label, "SansSerif", BOX_FONT_SIZE, false, false);
                // Java FtileBox draws text at padding.getTop() + ascent from box origin
                let text_x = cell.x + 10.0; // padding_left
                let text_y = cell.y + 10.0 + ascent; // padding_top + ascent

                let mut sg = SvgGraphic::new(0, 1.0);
                sg.set_fill_color(BOX_TEXT_COLOR);
                sg.svg_text(
                    label,
                    text_x,
                    text_y,
                    Some("sans-serif"),
                    BOX_FONT_SIZE,
                    None,
                    None,
                    None,
                    tw,
                    LengthAdjust::Spacing,
                    None,
                    0,
                    None,
                );
                buf.push_str(sg.body());
            }
        }
        BpmElementType::End => {
            // Similar to start but with different style
            write!(
                buf,
                r#"<ellipse cx="{}" cy="{}" fill="{}" rx="{}" ry="{}" style="stroke:{};stroke-width:{};"/>"#,
                fmt_coord(cx),
                fmt_coord(cy),
                START_FILL,
                fmt_coord(START_RADIUS),
                fmt_coord(START_RADIUS),
                START_STROKE,
                START_STROKE_WIDTH * 3.0,
            )
            .unwrap();
        }
    }

    // Draw connector lines on the element in Java Where enum order: N, E, S, W
    let nesw_order = [Where::North, Where::East, Where::South, Where::West];
    for dir in &nesw_order {
        if cell.connectors.contains(dir) {
            draw_connector_line(buf, cx, cy, cell.width, cell.height, *dir, false);
        }
    }
}

fn render_connector_puzzle(buf: &mut String, conn: &crate::layout::bpm::BpmConnectorLayout) {
    // Java ConnectorPuzzleEmpty: 20x20 cell, draws blue lines at specific offsets.
    let ox = conn.x + MARGIN; // puzzle origin x
    let oy = conn.y + MARGIN; // puzzle origin y

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
        write!(
            buf,
            r#"<line style="stroke:{};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            CONNECTOR_BLUE,
            fmt_coord(x1),
            fmt_coord(x2),
            fmt_coord(y1),
            fmt_coord(y2),
        )
        .unwrap();
    }
}

fn draw_connector_line(
    buf: &mut String,
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

    write!(
        buf,
        r#"<line style="stroke:{};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        color,
        fmt_coord(x1),
        fmt_coord(x2),
        fmt_coord(y1),
        fmt_coord(y2),
    )
    .unwrap();
}
