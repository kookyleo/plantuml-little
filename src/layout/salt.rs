//! Salt diagram layout — mirrors Java `ElementPyramid` / `AbstractElementText`
//! so the resulting geometry matches Java PlantUML byte-for-byte.
//!
//! The model:
//! * Each `SaltPyramid` computes column widths and row heights from the
//!   preferred dimensions of its cells. Widths are padded by `+2` (one pixel
//!   per side for the `+1` translate the pyramid applies when drawing each
//!   cell). Row heights add `+2` likewise.
//! * `ElementText`/`ElementButton`/… `getPreferredDimension` honours a
//!   character-based minimum: `max(text_width, char_length * 8)` where
//!   `char_length` is the number of characters in the raw token (Java uses
//!   `StringUtils.trin(text)`). This heuristic is what makes `[Cancel]`
//!   render wider than the raw text length would suggest.
//! * The overall diagram width/height is derived from a dry-run of the
//!   drawing that mimics Java's `LimitFinder`. `LimitFinder` uses
//!   `x + width - 1` for rectangles/ellipses and shifts text bounds so the
//!   result is slightly different from a naive `colsStart[cols]` sum.
//!
//! The `SaltLayout` returned here is a flat list of primitive drawing commands
//! referenced by coordinates (text, line, rect, ellipse, polygon). The
//! renderer then emits SVG directly from that list.

use crate::font_metrics;
use crate::model::salt::{SaltCell, SaltDiagram, SaltElement, SaltPyramid, TableStrategy};
use crate::Result;

const FONT: &str = "SansSerif";
const FONT_SIZE: f64 = 12.0;
/// Ascent of 12 pt SansSerif = 11.1386719.  Used to convert the pyramid's
/// y-offset into an SVG text baseline.
const ASCENT: f64 = 11.138_671_875;
/// Height of a 12 pt SansSerif line = 13.96875. Java's `ElementText.getPreferredDimension`
/// returns this for the height of a single line of text.
const LINE_HEIGHT: f64 = 13.968_75;
/// Additional padding added by Java's `ElementPyramid.init` per cell
/// (`dim.getWidth() + 2` / `dim.getHeight() + 2`).
const PYRAMID_PAD: f64 = 2.0;
/// Character width heuristic used by Java's `AbstractElementText.getSingleSpace`.
const SINGLE_SPACE: f64 = 8.0;
/// Document-level margin emitted by `PSystemSalt.getDefaultMargins()`.
const MARGIN: f64 = 5.0;
/// Extra `+1` that `ImageBuilder` adds to the `LimitFinder.maxX` before
/// applying margins (`limitFinder.getMaxX() + 1 + margin.left + margin.right`).
const MARGIN_EXTRA: f64 = 1.0;

/// Flat drawable command list. The renderer emits one SVG primitive per entry.
#[derive(Debug, Clone)]
pub struct SaltLayout {
    pub width: f64,
    pub height: f64,
    pub commands: Vec<DrawCmd>,
}

/// Low-level drawing command emitted by the layout pass. All coordinates are
/// in final SVG space (root margins already applied).
#[derive(Debug, Clone)]
pub enum DrawCmd {
    /// `<text>` element with pre-computed text length.
    Text {
        x: f64,
        y: f64,
        text: String,
        text_length: f64,
    },
    /// `<line>` element with stroke width 1.
    Line { x1: f64, y1: f64, x2: f64, y2: f64 },
    /// `<rect fill="none" stroke-width=...>` used by checkbox boxes.
    RectOutline {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        stroke_width: f64,
    },
    /// `<rect fill="#EEEEEE" ...>` rounded, used by buttons.
    RectFilled {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        rx: f64,
        fill: String,
        stroke_width: f64,
    },
    /// `<ellipse fill="none">` used by radios.
    Ellipse {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        stroke_width: f64,
    },
    /// Solid `<ellipse>` used by selected radios.
    EllipseFilled {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        stroke_width: f64,
    },
    /// `<polygon>` used for checkbox tick marks.
    Polygon {
        points: Vec<(f64, f64)>,
        stroke_width: f64,
    },
}

/// Entry point: compute the salt layout for a diagram.
pub fn layout_salt(diagram: &SaltDiagram) -> Result<SaltLayout> {
    let mut builder = LayoutBuilder::default();
    // Java's PSystemSalt.getTextBlock wraps the pyramid in a `drawU(ug, 0, …)`
    // followed by `drawU(ug, 1, …)`. zIndex 0 draws everything for salt, so
    // we only need one pass here.
    builder.draw_element(&diagram.root, MARGIN, MARGIN);

    // Derive canvas dimensions from the dry-run bounds using the Java
    // `LimitFinder` → `ImageBuilder.getFinalDimension()` formula:
    // width  = (limitFinder.maxX - MARGIN) + 1 + 2*MARGIN
    // height = (limitFinder.maxY - MARGIN) + 1 + 2*MARGIN
    // The `- MARGIN` cancels out because we shifted the draw origin by
    // MARGIN earlier; that leaves `limitFinder.maxX + 1 + MARGIN` as the
    // final width.
    let content_max_x = (builder.limit_max_x - MARGIN).max(0.0);
    let content_max_y = (builder.limit_max_y - MARGIN).max(0.0);
    let dim_w = content_max_x + MARGIN_EXTRA + 2.0 * MARGIN;
    let dim_h = content_max_y + MARGIN_EXTRA + 2.0 * MARGIN;
    // Java's SvgGraphics casts `(int)(x + 1)` once when the initial ensureVisible
    // runs. For our needs the resulting integer is the canvas dimension.
    let width = ((dim_w + 1.0).floor()).max(10.0);
    let height = ((dim_h + 1.0).floor()).max(10.0);

    Ok(SaltLayout {
        width,
        height,
        commands: builder.commands,
    })
}

#[derive(Default)]
struct LayoutBuilder {
    commands: Vec<DrawCmd>,
    limit_max_x: f64,
    limit_max_y: f64,
}

impl LayoutBuilder {
    /// Draw a salt element at absolute svg coords `(x, y)` (already shifted by
    /// the document margin).
    fn draw_element(&mut self, element: &SaltElement, x: f64, y: f64) {
        match element {
            SaltElement::Pyramid(pyr) => self.draw_pyramid(pyr, x, y),
            SaltElement::Text(text) => self.draw_text_element(text, x, y),
            SaltElement::Button(text) => self.draw_button(text, x, y),
            SaltElement::TextField(text) => self.draw_textfield(text, x, y),
            SaltElement::Checkbox { label, checked } => self.draw_checkbox(label, *checked, x, y),
            SaltElement::Radio { label, selected } => self.draw_radio(label, *selected, x, y),
        }
    }

    // ── LimitFinder helpers ────────────────────────────────────────────────

    fn limit_point(&mut self, x: f64, y: f64) {
        if x > self.limit_max_x {
            self.limit_max_x = x;
        }
        if y > self.limit_max_y {
            self.limit_max_y = y;
        }
    }

    fn limit_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        // LimitFinder.drawRectangle uses `x - 1` / `x + w - 1`.
        self.limit_point(x - 1.0, y - 1.0);
        self.limit_point(x + w - 1.0, y + h - 1.0);
    }

    fn limit_ellipse(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.limit_point(x, y);
        self.limit_point(x + w - 1.0, y + h - 1.0);
    }

    fn limit_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.limit_point(x1, y1);
        self.limit_point(x2, y2);
    }

    fn limit_polygon(&mut self, points: &[(f64, f64)]) {
        // Java's LimitFinder.drawUPolygon uses HACK_X_FOR_POLYGON = 10 on both sides
        // in X (but not Y).
        const HACK: f64 = 10.0;
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for (px, py) in points {
            min_x = min_x.min(*px);
            max_x = max_x.max(*px);
            min_y = min_y.min(*py);
            max_y = max_y.max(*py);
        }
        self.limit_point(min_x - HACK, min_y);
        self.limit_point(max_x + HACK, max_y);
    }

    fn limit_text(&mut self, baseline_x: f64, baseline_y: f64, text: &str) {
        // LimitFinder.drawText shifts y up by `dim.height - 1.5` (Java's
        // baseline → bbox origin conversion).
        let dim_w = font_metrics::text_width(text, FONT, FONT_SIZE, false, false);
        let dim_h = LINE_HEIGHT;
        let top_y = baseline_y - (dim_h - 1.5);
        self.limit_point(baseline_x, top_y);
        self.limit_point(baseline_x, top_y + dim_h);
        self.limit_point(baseline_x + dim_w, top_y);
        self.limit_point(baseline_x + dim_w, top_y + dim_h);
    }

    // ── Pyramid layout & drawing ───────────────────────────────────────────

    fn draw_pyramid(&mut self, pyr: &SaltPyramid, x: f64, y: f64) {
        // Compute cumulative column/row offsets mirroring Java's
        // `ElementPyramid.init()`.
        let (rows_start, cols_start) = init_pyramid_grid(pyr);

        // Draw each cell with a `+1` translate (Java: `UTranslate(xcell + 1,
        // ycell + 1)` passed to the element's `drawU`).
        for cell in &pyr.cells {
            let xcell = cols_start[cell.min_col];
            let ycell = rows_start[cell.min_row];
            self.draw_element(&cell.element, x + xcell + 1.0, y + ycell + 1.0);
        }

        // Draw the grid lines according to the strategy. Java's `Grid` builds
        // a set of segments from the cell spans + `addOutside()` for DRAW_ALL.
        if pyr.strategy == TableStrategy::DrawAll {
            self.draw_grid(pyr, &rows_start, &cols_start, x, y);
        }
    }

    fn draw_grid(
        &mut self,
        pyr: &SaltPyramid,
        rows_start: &[f64],
        cols_start: &[f64],
        x: f64,
        y: f64,
    ) {
        // Collect insertion order of segments and feed them through a
        // HashSet-order simulator that mirrors Java's `HashSet<Segment>`.
        // Java's Segment.hashCode = row*47 + col and HashMap uses a default
        // capacity of 16 with bucket = (hash & (cap-1)); within a bucket
        // entries are linked in insertion order (`putVal` appends the new
        // node to the tail of the linked list).
        let nb_row = rows_start.len();
        let nb_col = cols_start.len();
        let mut h_inserted: Vec<(usize, usize)> = Vec::new();
        let mut v_inserted: Vec<(usize, usize)> = Vec::new();

        // addOutside for DRAW_ALL (inserted first, just like Java).
        for c in 0..nb_col.saturating_sub(1) {
            h_inserted.push((0, c));
            h_inserted.push((nb_row - 1, c));
        }
        for r in 0..nb_row.saturating_sub(1) {
            v_inserted.push((r, 0));
            v_inserted.push((r, nb_col - 1));
        }
        // Per-cell additions (DRAW_ALL adds both H and V segments).
        for cell in &pyr.cells {
            for c in cell.min_col..=cell.max_col {
                h_inserted.push((cell.min_row, c));
                h_inserted.push((cell.max_row + 1, c));
            }
            for r in cell.min_row..=cell.max_row {
                v_inserted.push((r, cell.min_col));
                v_inserted.push((r, cell.max_col + 1));
            }
        }

        let horizontals = java_hashset_order(&h_inserted);
        let verticals = java_hashset_order(&v_inserted);

        for (row, col) in &horizontals {
            let width = cols_start[col + 1] - cols_start[*col];
            let x1 = x + cols_start[*col];
            let y1 = y + rows_start[*row];
            self.commands.push(DrawCmd::Line {
                x1,
                y1,
                x2: x1 + width,
                y2: y1,
            });
            self.limit_line(x1, y1, x1 + width, y1);
        }
        for (row, col) in &verticals {
            let height = rows_start[row + 1] - rows_start[*row];
            let x1 = x + cols_start[*col];
            let y1 = y + rows_start[*row];
            self.commands.push(DrawCmd::Line {
                x1,
                y1,
                x2: x1,
                y2: y1 + height,
            });
            self.limit_line(x1, y1, x1, y1 + height);
        }
    }

    // ── Individual element draw routines ───────────────────────────────────

    fn draw_text_element(&mut self, text: &str, x: f64, y: f64) {
        // ElementText.drawU draws the text block at (0, 0) of its cell. The
        // text baseline is y + ascent.
        let baseline_x = x;
        let baseline_y = y + ASCENT;
        let text_len = font_metrics::text_width(text, FONT, FONT_SIZE, false, false);
        self.commands.push(DrawCmd::Text {
            x: baseline_x,
            y: baseline_y,
            text: text.to_string(),
            text_length: text_len,
        });
        self.limit_text(baseline_x, baseline_y, text);
    }

    fn draw_button(&mut self, text: &str, x: f64, y: f64) {
        // Java ElementButton: stroke=2.5, marginX=2, marginY=2.
        let stroke = 2.5_f64;
        let margin_x = 2.0_f64;
        let margin_y = 2.0_f64;
        let (text_w, _text_h) = text_dim_with_min_width(text, char_length(text));
        // dim = (text_w+4+5, text_h+4+5) = (text_w+9, text_h+9)
        let dim_w = text_w + 2.0 * margin_x + 2.0 * stroke;
        let dim_h = LINE_HEIGHT + 2.0 * margin_y + 2.0 * stroke;
        let rect_w = dim_w - 2.0 * stroke;
        let rect_h = dim_h - 2.0 * stroke;
        let rect_x = x + stroke;
        let rect_y = y + stroke;
        self.commands.push(DrawCmd::RectFilled {
            x: rect_x,
            y: rect_y,
            w: rect_w,
            h: rect_h,
            rx: 5.0,
            fill: "#EEEEEE".to_string(),
            stroke_width: stroke,
        });
        self.limit_rect(rect_x, rect_y, rect_w, rect_h);

        // Text centred horizontally using the real (unpadded) text width:
        // drawText(ug, (dim.w - dimPureText.w) / 2, stroke + marginY)
        let pure_text_w = font_metrics::text_width(text, FONT, FONT_SIZE, false, false);
        let text_x = x + (dim_w - pure_text_w) / 2.0;
        let text_y = y + stroke + margin_y + ASCENT;
        self.commands.push(DrawCmd::Text {
            x: text_x,
            y: text_y,
            text: text.to_string(),
            text_length: pure_text_w,
        });
        self.limit_text(text_x, text_y, text);
    }

    fn draw_textfield(&mut self, text: &str, x: f64, y: f64) {
        // Java ElementTextField: drawText(ug, 3, 0), then three lines forming
        // brackets around the underline.
        let (text_w_padded, _) = text_dim_with_min_width(text, char_length_quoted(text));
        let dim_w = text_w_padded + 6.0;
        let text_w = text_w_padded; // getTextDimensionAt().width
        let text_h = LINE_HEIGHT;
        let text_x = x + 3.0;
        let text_y = y + ASCENT;
        let pure_text_w = font_metrics::text_width(text, FONT, FONT_SIZE, false, false);
        self.commands.push(DrawCmd::Text {
            x: text_x,
            y: text_y,
            text: text.to_string(),
            text_length: pure_text_w,
        });
        self.limit_text(text_x, text_y, text);

        // Bottom hline: UTranslate(1, textDim.height).draw(ULine.hline(dim.width - 3))
        let line_y = y + text_h;
        let line_x1 = x + 1.0;
        let line_x2 = line_x1 + (dim_w - 3.0);
        self.commands.push(DrawCmd::Line {
            x1: line_x1,
            y1: line_y,
            x2: line_x2,
            y2: line_y,
        });
        self.limit_line(line_x1, line_y, line_x2, line_y);

        // Left vertical tick: UTranslate(1, text_h - 3).draw(ULine.vline(2))
        let tick_y_top = y + text_h - 3.0;
        let tick_y_bot = tick_y_top + 2.0;
        self.commands.push(DrawCmd::Line {
            x1: line_x1,
            y1: tick_y_top,
            x2: line_x1,
            y2: tick_y_bot,
        });
        self.limit_line(line_x1, tick_y_top, line_x1, tick_y_bot);

        // Right vertical tick: UTranslate(3 + textDim.width + 1, text_h - 3)
        let tick_rx = x + 3.0 + text_w + 1.0;
        self.commands.push(DrawCmd::Line {
            x1: tick_rx,
            y1: tick_y_top,
            x2: tick_rx,
            y2: tick_y_bot,
        });
        self.limit_line(tick_rx, tick_y_top, tick_rx, tick_y_bot);
    }

    fn draw_checkbox(&mut self, label: &str, checked: bool, x: f64, y: f64) {
        // Java ElementRadioCheckbox (radio=false): block is the label text at
        // translate(20, 0); then the 10x10 rectangle at (2, (height-10)/2)
        // with stroke 1.5; if checked, a polygon tick.
        let stroke = 1.5_f64;
        let rectangle = 10.0_f64;
        let label_x = x + 20.0;
        let label_y = y + ASCENT;
        let label_w = font_metrics::text_width(label, FONT, FONT_SIZE, false, false);
        self.commands.push(DrawCmd::Text {
            x: label_x,
            y: label_y,
            text: label.to_string(),
            text_length: label_w,
        });
        self.limit_text(label_x, label_y, label);

        let height = LINE_HEIGHT;
        let rx = x + 2.0;
        let ry = y + (height - rectangle) / 2.0;
        self.commands.push(DrawCmd::RectOutline {
            x: rx,
            y: ry,
            w: rectangle,
            h: rectangle,
            stroke_width: stroke,
        });
        self.limit_rect(rx, ry, rectangle, rectangle);

        if checked {
            // Polygon points from Java:
            //   poly.addPoint(0, 0);
            //   poly.addPoint(3, 3);
            //   poly.addPoint(10, -6);
            //   poly.addPoint(3, 1);
            // translated by UTranslate(3, 6).
            let ox = x + 3.0;
            let oy = y + 6.0;
            let points = vec![
                (ox + 0.0, oy + 0.0),
                (ox + 3.0, oy + 3.0),
                (ox + 10.0, oy - 6.0),
                (ox + 3.0, oy + 1.0),
            ];
            self.limit_polygon(&points);
            self.commands.push(DrawCmd::Polygon {
                points,
                stroke_width: stroke,
            });
        }
    }

    fn draw_radio(&mut self, label: &str, selected: bool, x: f64, y: f64) {
        let stroke = 1.5_f64;
        let ellipse = 10.0_f64;
        let ellipse2 = 4.0_f64;
        // Label text
        let label_x = x + 20.0;
        let label_y = y + ASCENT;
        let label_w = font_metrics::text_width(label, FONT, FONT_SIZE, false, false);
        self.commands.push(DrawCmd::Text {
            x: label_x,
            y: label_y,
            text: label.to_string(),
            text_length: label_w,
        });
        self.limit_text(label_x, label_y, label);

        let height = LINE_HEIGHT;
        // drawRadio: UTranslate(2, (height-ELLIPSE)/2); UEllipse.build(10, 10)
        let ex = x + 2.0;
        let ey = y + (height - ellipse) / 2.0;
        let cx = ex + ellipse / 2.0;
        let cy = ey + ellipse / 2.0;
        self.commands.push(DrawCmd::Ellipse {
            cx,
            cy,
            rx: ellipse / 2.0,
            ry: ellipse / 2.0,
            stroke_width: stroke,
        });
        self.limit_ellipse(ex, ey, ellipse, ellipse);

        if selected {
            // UTranslate(2 + (ELLIPSE - ELLIPSE2)/2, (height - ELLIPSE2)/2)
            //   .draw(UEllipse.build(ELLIPSE2, ELLIPSE2))
            let inner_x = x + 2.0 + (ellipse - ellipse2) / 2.0;
            let inner_y = y + (height - ellipse2) / 2.0;
            let cx2 = inner_x + ellipse2 / 2.0;
            let cy2 = inner_y + ellipse2 / 2.0;
            self.commands.push(DrawCmd::EllipseFilled {
                cx: cx2,
                cy: cy2,
                rx: ellipse2 / 2.0,
                ry: ellipse2 / 2.0,
                stroke_width: 0.0,
            });
            self.limit_ellipse(inner_x, inner_y, ellipse2, ellipse2);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Element preferred dimension + pyramid column/row initialisation
// ──────────────────────────────────────────────────────────────────────────

/// Simulate `java.util.HashSet<Segment>` iteration order for the `Segment`
/// class used by Java's salt Grid. The simulated HashMap uses the default
/// initial capacity of 16 and Segment's hashCode `row * 47 + col`. Within a
/// bucket, entries are linked in insertion order so iteration visits older
/// entries first; buckets are visited in ascending index order.
fn java_hashset_order(inserted: &[(usize, usize)]) -> Vec<(usize, usize)> {
    // Java HashMap starts at capacity 16 and resizes once load factor 0.75
    // is exceeded (size > cap * 0.75 → resize to cap*2). For up to 12 unique
    // segments capacity stays 16. When it resizes, existing entries are
    // redistributed so iteration order may change. Simulate the same
    // behaviour so we stay byte-identical with Java for salt tables of
    // arbitrary size.
    let mut capacity: usize = 16;
    // buckets[i] = list of (row, col) in insertion order for that bucket.
    let mut buckets: Vec<Vec<(usize, usize)>> = vec![Vec::new(); capacity];
    let mut size: usize = 0;
    let load_factor = 0.75_f64;

    let hashcode = |(row, col): (usize, usize)| -> usize { row.wrapping_mul(47).wrapping_add(col) };
    let bucket_idx = |h: usize, cap: usize| -> usize { h & (cap - 1) };

    for &seg in inserted {
        let bi = bucket_idx(hashcode(seg), capacity);
        // Check for existing equal entry
        if buckets[bi].contains(&seg) {
            continue;
        }
        buckets[bi].push(seg);
        size += 1;
        if (size as f64) > (capacity as f64) * load_factor {
            // Resize by doubling and redistribute.
            let new_capacity = capacity * 2;
            let mut new_buckets: Vec<Vec<(usize, usize)>> = vec![Vec::new(); new_capacity];
            // Java HashMap.resize preserves iteration order by iterating the
            // old buckets in order and appending each entry's new bucket.
            for bucket in &buckets {
                for &entry in bucket {
                    let nbi = bucket_idx(hashcode(entry), new_capacity);
                    new_buckets[nbi].push(entry);
                }
            }
            buckets = new_buckets;
            capacity = new_capacity;
        }
    }

    let mut result = Vec::with_capacity(size);
    for bucket in buckets {
        for entry in bucket {
            result.push(entry);
        }
    }
    result
}

fn char_length(text: &str) -> usize {
    // Java's `AbstractElementText.getCharNumber` strips sprite tags and returns
    // the resulting character count. We don't support sprite tags yet, so
    // counting Unicode chars is sufficient for current fixtures.
    text.chars().count()
}

/// `char_length` for text fields: the factory strips the quotes before
/// constructing the element, so charLength is computed on the unquoted text.
fn char_length_quoted(text: &str) -> usize {
    char_length(text)
}

/// Java `AbstractElementText.getTextDimensionAt`: returns
/// `max(text_width, char_length * dim_space)` for the width.
fn text_dim_with_min_width(text: &str, char_len: usize) -> (f64, f64) {
    let text_w = font_metrics::text_width(text, FONT, FONT_SIZE, false, false);
    let constrained = (char_len as f64) * SINGLE_SPACE;
    let w = if char_len == 0 {
        text_w
    } else {
        text_w.max(constrained)
    };
    (w, LINE_HEIGHT)
}

/// Compute a pyramid's `rowsStart`/`colsStart` arrays the same way Java's
/// `ElementPyramid.init()` does.
fn init_pyramid_grid(pyr: &SaltPyramid) -> (Vec<f64>, Vec<f64>) {
    // init: title height = 0 for all our pyramids (no title support).
    let rows = pyr.rows.max(extend_rows(&pyr.cells));
    let cols = pyr.cols.max(extend_cols(&pyr.cells));

    let mut rows_start = vec![0.0_f64; rows + 1];
    let mut cols_start = vec![0.0_f64; cols + 1];

    // Java sorts cells by LeftFirst before applying col widths, then by
    // TopFirst before applying row heights. LeftFirst order: smaller minCol
    // first (stable by insertion order). TopFirst: smaller minRow first.
    let mut by_left: Vec<&SaltCell> = pyr.cells.iter().collect();
    by_left.sort_by_key(|c| c.min_col);
    for cell in &by_left {
        let dim = preferred_dim(&cell.element);
        ensure_col_width(
            &mut cols_start,
            cell.min_col,
            cell.max_col + 1,
            dim.0 + PYRAMID_PAD,
        );
    }

    let mut by_top: Vec<&SaltCell> = pyr.cells.iter().collect();
    by_top.sort_by_key(|c| c.min_row);
    for cell in &by_top {
        let dim = preferred_dim(&cell.element);
        // supY = (minRow == 0) ? titleHeight/2 : 0 → always 0 without a title.
        ensure_row_height(
            &mut rows_start,
            cell.min_row,
            cell.max_row + 1,
            dim.1 + PYRAMID_PAD,
        );
    }

    (rows_start, cols_start)
}

fn extend_rows(cells: &[SaltCell]) -> usize {
    cells.iter().map(|c| c.max_row + 1).max().unwrap_or(1)
}

fn extend_cols(cells: &[SaltCell]) -> usize {
    cells.iter().map(|c| c.max_col + 1).max().unwrap_or(1)
}

fn ensure_col_width(cols_start: &mut [f64], first: usize, last: usize, width: f64) {
    let actual = cols_start[last] - cols_start[first];
    let missing = width - actual;
    if missing > 0.0 {
        for v in cols_start.iter_mut().skip(last) {
            *v += missing;
        }
    }
}

fn ensure_row_height(rows_start: &mut [f64], first: usize, last: usize, height: f64) {
    let actual = rows_start[last] - rows_start[first];
    let missing = height - actual;
    if missing > 0.0 {
        for v in rows_start.iter_mut().skip(last) {
            *v += missing;
        }
    }
}

/// Compute the Java `getPreferredDimension` for an element (returns `(w, h)`).
fn preferred_dim(element: &SaltElement) -> (f64, f64) {
    match element {
        SaltElement::Text(text) => {
            // Java ElementText extends AbstractElement directly (not
            // AbstractElementText) so there is no charLength heuristic —
            // the preferred dim is just the raw text block dimension.
            let w = font_metrics::text_width(text, FONT, FONT_SIZE, false, false);
            (w, LINE_HEIGHT)
        }
        SaltElement::Button(text) => {
            let stroke = 2.5;
            let margin_x = 2.0;
            let margin_y = 2.0;
            let (tw, th) = text_dim_with_min_width(text, char_length(text));
            (
                tw + 2.0 * margin_x + 2.0 * stroke,
                th + 2.0 * margin_y + 2.0 * stroke,
            )
        }
        SaltElement::TextField(text) => {
            let (tw, th) = text_dim_with_min_width(text, char_length_quoted(text));
            // delta(6, 2)
            (tw + 6.0, th + 2.0)
        }
        SaltElement::Checkbox { label, .. } | SaltElement::Radio { label, .. } => {
            // ElementRadioCheckbox: block.dim.delta(margin=20, 0)
            let lw = font_metrics::text_width(label, FONT, FONT_SIZE, false, false);
            (lw + 20.0, LINE_HEIGHT)
        }
        SaltElement::Pyramid(pyr) => pyramid_preferred_dim(pyr),
    }
}

fn pyramid_preferred_dim(pyr: &SaltPyramid) -> (f64, f64) {
    let (rows_start, cols_start) = init_pyramid_grid(pyr);
    (
        *cols_start.last().unwrap_or(&0.0),
        *rows_start.last().unwrap_or(&0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::salt::parse_salt_diagram;

    #[test]
    fn single_text_cell_has_expected_dim() {
        // `{ Title }` mirrors Java's test6-equivalent: svg 39x25.
        let src = "@startsalt\n{\nTitle\n}\n@endsalt";
        let diag = parse_salt_diagram(src).unwrap();
        let layout = layout_salt(&diag).unwrap();
        assert_eq!(layout.width as i32, 39);
        assert_eq!(layout.height as i32, 25);
    }

    #[test]
    fn single_button_cell_has_expected_dim() {
        // `{ [Cancel] }` → 66x32 in Java.
        let src = "@startsalt\n{\n[Cancel]\n}\n@endsalt";
        let diag = parse_salt_diagram(src).unwrap();
        let layout = layout_salt(&diag).unwrap();
        assert_eq!(layout.width as i32, 66);
        assert_eq!(layout.height as i32, 32);
    }

    #[test]
    fn single_textfield_cell_has_expected_dim() {
        // `{ "input text" }` → 97x26 in Java.
        let src = "@startsalt\n{\n\"input text\"\n}\n@endsalt";
        let diag = parse_salt_diagram(src).unwrap();
        let layout = layout_salt(&diag).unwrap();
        assert_eq!(layout.width as i32, 97);
        assert_eq!(layout.height as i32, 26);
    }

    #[test]
    fn checkbox_cell_has_expected_dim() {
        // `{ [X] Feature A }` → 91x25 in Java.
        let src = "@startsalt\n{\n[X] Feature A\n}\n@endsalt";
        let diag = parse_salt_diagram(src).unwrap();
        let layout = layout_salt(&diag).unwrap();
        assert_eq!(layout.width as i32, 91);
        assert_eq!(layout.height as i32, 25);
    }
}
