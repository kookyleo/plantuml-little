use crate::font_metrics;
use crate::model::wire::WireDiagram;
use crate::Result;

/// Default block size when not explicitly specified (matches Java WBlock).
const DEFAULT_SIZE: f64 = 100.0;
/// Starting Y for blocks inside root (Java WBlock.STARTING_Y = 10).
const STARTING_Y: f64 = 10.0;
/// Gap before/after each block (Java WBlock.addBlock: cursor += dy(10) before and after).
const BLOCK_GAP: f64 = 10.0;
/// Left margin for blocks (Java WBlock.cursor starts at x=10).
const LEFT_MARGIN: f64 = 10.0;
/// Java ImageBuilder margin = 10.
const CANVAS_MARGIN: f64 = 10.0;
/// Font size for block name labels (Java WBlock uses sansSerif 12).
const FONT_SIZE: f64 = 12.0;
/// Java WBlock label offset from block left edge = 5.
const LABEL_OFFSET_X: f64 = 5.0;
/// Java renders a nbsp text at cursor_x - 5 (root cursor x=10, so x=5).
const TOP_TEXT_X: f64 = 5.0;

/// Layout for a single wire block.
#[derive(Debug, Clone)]
pub struct WireBlockLayout {
    pub name: String,
    /// Position relative to root (before ImageBuilder margin shift).
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<String>,
}

/// Layout for a vertical link.
#[derive(Debug, Clone)]
pub struct WireVLinkLayout {
    /// X position of the link line (content coords, before margin).
    pub x: f64,
    /// Y of line start (source block bottom + 1, content coords).
    pub line_y_start: f64,
    /// Y of arrow tip (dest block top - 2, content coords).
    pub arrow_tip_y: f64,
    /// Y of line end (dest block top - 1, content coords).
    pub line_y_end: f64,
}

/// Full wire diagram layout.
#[derive(Debug)]
pub struct WireLayout {
    pub width: f64,
    pub height: f64,
    pub blocks: Vec<WireBlockLayout>,
    pub vlinks: Vec<WireVLinkLayout>,
    /// Y position for the nbsp text (unshifted).
    pub top_text_y: f64,
}

pub fn layout_wire(d: &WireDiagram) -> Result<WireLayout> {
    // Simulate Java WBlock cursor logic exactly.
    // cursor starts at (10, STARTING_Y=10).
    let mut cursor_y = STARTING_Y;
    let mut blocks: Vec<WireBlockLayout> = Vec::new();
    let mut added_to_cursor: Option<usize> = None; // index into blocks

    for block in &d.blocks {
        let bw = if block.width > 0.0 {
            block.width
        } else {
            DEFAULT_SIZE
        };
        let bh = if block.height > 0.0 {
            block.height
        } else {
            DEFAULT_SIZE
        };

        // Java addBlock: cursor += dy(10)
        cursor_y += BLOCK_GAP;

        // getNextPosition: if addedToCursor != null, cursor += dy(its height)
        if let Some(prev_idx) = added_to_cursor {
            cursor_y += blocks[prev_idx].height;
        }
        let _ = added_to_cursor;

        let block_y = cursor_y;

        // cursor += dy(10) after block
        cursor_y += BLOCK_GAP;
        added_to_cursor = Some(blocks.len());

        blocks.push(WireBlockLayout {
            name: block.name.clone(),
            x: LEFT_MARGIN,
            y: block_y,
            width: bw,
            height: bh,
            color: block.color.clone(),
        });
    }

    // Compute vlinks.
    // Java WBlock.getNextOutVertical: first call returns absolutePos("5", "100%")
    // = (parent_abs + block_pos.x + 5, parent_abs + block_pos.y + height)
    // Subsequent calls shift by type.spaceForNext() (15 for default).
    let mut vlinks = Vec::new();

    // Track per-block out-vertical counters
    let mut block_out_count: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for vl in &d.vlinks {
        let from_block = blocks.iter().find(|b| b.name == vl.from);
        let to_block = blocks.iter().find(|b| b.name == vl.to);
        if let (Some(fb), Some(tb)) = (from_block, to_block) {
            let count = block_out_count.entry(vl.from.clone()).or_insert(0);
            // Java getNextOutVertical: first call x = block.x + 5, subsequent += spaceForNext(15)
            let x = fb.x + 5.0 + (*count as f64) * 15.0;
            *count += 1;
            // Java drawNormalArrow:
            //   start = (x, source.y + source.height)
            //   dy = dest.y - start.y - 2
            //   arrow tip at start.y + dy = dest.y - 2
            //   line from start.y+1 to start.y+1+dy = dest.y - 1
            let start_y = fb.y + fb.height;
            let dest_y = tb.y;
            vlinks.push(WireVLinkLayout {
                x,
                line_y_start: start_y + 1.0,
                arrow_tip_y: dest_y - 2.0,
                line_y_end: dest_y - 1.0,
            });
        }
    }

    // Compute content bounds using Java LimitFinder rules:
    // - Rectangle: addPoint(x-1, y-1), addPoint(x+w-1, y+h-1)
    // - Text: addPoint(x, y_adj), addPoint(x+dimW, y_adj+dimH)
    //   where y_adj = y - (dimH - 1.5)
    // - Polygon: addPoint(x+minX-10, y+minY), addPoint(x+maxX+10, y+maxY)
    // - Line: addPoint(x1, y1), addPoint(x1+dx, y1+dy)
    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;

    let ascent = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let _line_h = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);

    for bl in &blocks {
        // Rect at (bl.x, bl.y): LimitFinder addPoint(x+w-1, y+h-1)
        max_x = max_x.max(bl.x + bl.width - 1.0);
        max_y = max_y.max(bl.y + bl.height - 1.0);
        // Text at translate (bl.x + 5, bl.y):
        // LimitFinder.drawText: y_adj = bl.y - (dimH - 1.5)
        // addPoint(bl.x+5+tw, y_adj+dimH) = addPoint(bl.x+5+tw, bl.y + 1.5)
        let tw = font_metrics::text_width(&bl.name, "SansSerif", FONT_SIZE, false, false);
        max_x = max_x.max(bl.x + LABEL_OFFSET_X + tw);
        max_y = max_y.max(bl.y + 1.5);
    }

    // Top nbsp text at translate (5, 0):
    // drawText(5, 0, "\u00a0"): y_adj = 0 - (dimH - 1.5); addPoint(5+tw, y_adj+dimH) = addPoint(5+tw, 1.5)
    let top_text_y = ascent; // content coord for renderer (0 + ascent)
    let nbsp_tw = font_metrics::text_width("\u{00a0}", "SansSerif", FONT_SIZE, false, false);
    max_x = max_x.max(TOP_TEXT_X + nbsp_tw);
    max_y = max_y.max(1.5);

    // Vlink extents:
    for vl in &vlinks {
        // UPath (arrow triangle) drawn at translate (vl.x, vl.arrow_tip_y):
        // segments: M(0,0) L(5,-5) L(-5,-5) L(0,0) close
        // UPath.minX=-5, maxX=5, minY=-5, maxY=0
        // LimitFinder.drawUPath: addPoint(vl.x+(-5), vl.arrow_tip_y+(-5)),
        //                        addPoint(vl.x+5, vl.arrow_tip_y+0)
        max_x = max_x.max(vl.x + 5.0);
        max_y = max_y.max(vl.arrow_tip_y);
        // ULine from translate (vl.x, vl.line_y_start), length = line_y_end - line_y_start
        // LimitFinder.drawULine: addPoint(vl.x, vl.line_y_start),
        //                        addPoint(vl.x+0, vl.line_y_start + (line_y_end - line_y_start))
        //                      = addPoint(vl.x, vl.line_y_end)
        max_y = max_y.max(vl.line_y_end);
    }

    // Java ImageBuilder: dimension = (maxX + 1 + margin_left + margin_right, ...)
    // margin_left = margin_right = 10
    let canvas_w = max_x + 1.0 + 2.0 * CANVAS_MARGIN;
    let canvas_h = max_y + 1.0 + 2.0 * CANVAS_MARGIN;

    Ok(WireLayout {
        width: canvas_w,
        height: canvas_h,
        blocks,
        vlinks,
        top_text_y,
    })
}
