use crate::font_metrics;
use crate::model::ebnf::{EbnfDiagram, EbnfExpr, EbnfRule};
use crate::Result;

#[derive(Debug)]
pub struct EbnfLayout {
    pub width: f64,
    pub height: f64,
    pub elements: Vec<EbnfElement>,
}

#[derive(Debug, Clone)]
pub enum EbnfElement {
    Title { x: f64, y: f64, text: String },
    Comment { x: f64, y: f64, width: f64, height: f64, text: String },
    RuleName { x: f64, y: f64, text: String },
    TerminalBox { x: f64, y: f64, width: f64, height: f64, text: String },
    NonTerminalBox { x: f64, y: f64, width: f64, height: f64, text: String },
    HLine { x1: f64, y1: f64, x2: f64, y2: f64, stroke_width: f64 },
    VLine { x1: f64, y1: f64, x2: f64, y2: f64, stroke_width: f64 },
    Path { d: String, fill: bool, stroke_width: f64 },
    StartCircle { cx: f64, cy: f64, r: f64 },
    EndCircle { cx: f64, cy: f64, r: f64 },
    Arrow { x: f64, y: f64 },
    LeftArrow { x: f64, y: f64 },
}

// ── Java ETile constants ──────────────────────────────────────────
const FONT_SIZE: f64 = 14.0;
const TITLE_FONT_SIZE: f64 = 14.0;
const COMMENT_FONT_SIZE: f64 = 13.0;

/// ETileBox: box padding = 5 each side → box_w = text_w + 10, box_h = text_h + 10
const BOX_PAD: f64 = 5.0;

/// ETileAlternation.marginx = 12, adds 2*2*marginx to width
const ALT_MARGINX: f64 = 12.0;
/// Gap between alternation tiles = 10
const ALT_GAP: f64 = 10.0;

/// ETileConcatenation.marginx = 20
const CONCAT_MARGINX: f64 = 20.0;

/// ETileOptional2.deltax = 24
const OPT2_DELTAX: f64 = 24.0;
/// ETileOptional2.h1 = 10 (no notes)
const OPT2_H1: f64 = 10.0;
/// ETileOptional2.deltay = 20 (no notes)
const OPT2_DELTAY: f64 = 20.0;

/// ETileOneOrMore.deltax = 15
const OOM_DELTAX: f64 = 15.0;
/// ETileOneOrMore.deltay = 12
const OOM_DELTAY: f64 = 12.0;
/// ETileOneOrMore corner delta = 8
const OOM_CORNER: f64 = 8.0;

/// ETileWithCircles.deltax = 30
const WC_DELTAX: f64 = 30.0;
/// ETileWithCircles.SIZE (circle diameter) = 8
const WC_SIZE: f64 = 8.0;

/// EbnfExpression: withMargin(main, 0, 0, 10, 15) — top margin before WithCircles
const EXPR_MARGIN_TOP: f64 = 10.0;
/// EbnfExpression: withMargin(main, 0, 0, 10, 15) — bottom margin after WithCircles
const EXPR_MARGIN_BOTTOM: f64 = 15.0;

/// PSystemEbnf.addNote: withMargin(note, 0, 0, 5, 15)
const NOTE_MARGIN_TOP: f64 = 5.0;
const NOTE_MARGIN_BOTTOM: f64 = 15.0;
/// Opale note margins: marginX1=6 (left), marginX2=15 (right+fold), marginY=5 (top/bottom)
const OPALE_MARGIN_X1: f64 = 6.0;
const OPALE_MARGIN_X2: f64 = 15.0;
const OPALE_MARGIN_Y: f64 = 5.0;

/// Framework margin (TextBlockExporter12026 default margin = 10 all sides)
const FW_MARGIN: f64 = 10.0;

/// Document title style: Padding=5, Margin=5
const TITLE_PAD: f64 = 5.0;
const TITLE_MARGIN: f64 = 5.0;

/// Line stroke for rail lines
const STROKE: f64 = 1.5;

// ── ETile dimension model ─────────────────────────────────────────
// Each tile has (width, h1, h2) where h1 = distance from top to the rail center,
// h2 = distance from rail center to bottom. Total height = h1 + h2.

struct TileDim {
    width: f64,
    h1: f64,
    h2: f64,
}

fn tile_dim(expr: &EbnfExpr) -> TileDim {
    match expr {
        EbnfExpr::Terminal(text) | EbnfExpr::NonTerminal(text) | EbnfExpr::Special(text) => {
            let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
            let th = font_metrics::ascent("SansSerif", FONT_SIZE, false, false)
                + font_metrics::descent("SansSerif", FONT_SIZE, false, false);
            let bw = tw + 2.0 * BOX_PAD;
            let bh1 = (th + 2.0 * BOX_PAD) / 2.0;
            TileDim { width: bw, h1: bh1, h2: bh1 }
        }
        EbnfExpr::Alternation(alts) => {
            let mut max_w = 0.0f64;
            for a in alts {
                let d = tile_dim(a);
                max_w = max_w.max(d.width);
            }
            let width = max_w + 4.0 * ALT_MARGINX;
            let first = tile_dim(&alts[0]);
            let h1 = first.h1;
            let mut h2 = first.h2;
            for a in &alts[1..] {
                let d = tile_dim(a);
                h2 += d.h1 + d.h2 + ALT_GAP;
            }
            TileDim { width, h1, h2 }
        }
        EbnfExpr::Sequence(parts) => {
            let mut width = 0.0;
            let mut max_h1 = 0.0f64;
            let mut max_h2 = 0.0f64;
            for (i, p) in parts.iter().enumerate() {
                let d = tile_dim(p);
                width += d.width;
                if i < parts.len() - 1 {
                    width += CONCAT_MARGINX;
                }
                max_h1 = max_h1.max(d.h1);
                max_h2 = max_h2.max(d.h2);
            }
            TileDim { width, h1: max_h1, h2: max_h2 }
        }
        EbnfExpr::Optional(inner) => {
            // ETileOptional2: h1=10, h2=10+orig.h1+orig.h2, width=orig.w+2*24
            let d = tile_dim(inner);
            TileDim {
                width: d.width + 2.0 * OPT2_DELTAX,
                h1: OPT2_H1,
                h2: OPT2_H1 + d.h1 + d.h2,
            }
        }
        EbnfExpr::Repetition(inner) => {
            // ETileOneOrMore: h1=deltay(12)+orig.h1, h2=orig.h2, width=orig.w+2*deltax(15)
            let d = tile_dim(inner);
            TileDim {
                width: d.width + 2.0 * OOM_DELTAX,
                h1: OOM_DELTAY + d.h1,
                h2: d.h2,
            }
        }
        EbnfExpr::Group(inner) => tile_dim(inner),
    }
}

pub fn layout_ebnf(diagram: &EbnfDiagram) -> Result<EbnfLayout> {
    let mut elements = Vec::new();
    // body_width: the Java TextBlock body width (for centering title)
    let mut body_width = 0.0f64;

    // Global offset: framework margin + title block padding/margin
    let mut y = FW_MARGIN + TITLE_MARGIN + TITLE_PAD;

    // Diagram title block dimensions (from root.document.title style)
    // TextBlockBordered adds +1 to both width and height
    let title_block_w;

    // Diagram title (rendered by framework as root.document.title style)
    if let Some(title) = &diagram.title {
        let tw = font_metrics::text_width(title, "SansSerif", TITLE_FONT_SIZE, true, false);
        let asc = font_metrics::ascent("SansSerif", TITLE_FONT_SIZE, true, false);
        let desc = font_metrics::descent("SansSerif", TITLE_FONT_SIZE, true, false);
        // Title baseline within frame: y + ascent
        let title_baseline = y + asc;
        // Title x will be centered later
        elements.push(EbnfElement::Title {
            x: 0.0, // placeholder, centered below
            y: title_baseline,
            text: title.clone(),
        });
        // Title block = bordered(text, padding=5) + margin(5):
        // bordered dim = (text_w + 2*pad + 1, text_h + 2*pad + 1) [TextBlockBordered adds +1]
        // with margin = bordered + 2*margin
        title_block_w = tw + 2.0 * TITLE_PAD + 1.0 + 2.0 * TITLE_MARGIN;
        let title_block_h = asc + desc + 2.0 * TITLE_PAD + 1.0 + 2.0 * TITLE_MARGIN;
        // Advance y past the title block
        y += title_block_h - (TITLE_MARGIN + TITLE_PAD);
        // y is now at: FW_MARGIN + title_block_h
    } else {
        title_block_w = 0.0;
    }

    // Comment note (from PSystemEbnf.addNote: FloatingNote + withMargin(0,0,5,15))
    if let Some(comment) = &diagram.comment {
        y += NOTE_MARGIN_TOP;
        // Java captures the comment text WITH leading/trailing spaces from "(* comment *)"
        // → " comment ". The Opale layout uses the full text width including spaces.
        // We display the trimmed text but compute width from the space-padded version.
        let space_w = font_metrics::text_width(" ", "SansSerif", COMMENT_FONT_SIZE, false, false);
        let cw = font_metrics::text_width(comment, "SansSerif", COMMENT_FONT_SIZE, false, false);
        let full_text_w = space_w + cw + space_w; // " comment " with spaces
        let ch = font_metrics::ascent("SansSerif", COMMENT_FONT_SIZE, false, false)
            + font_metrics::descent("SansSerif", COMMENT_FONT_SIZE, false, false);
        // Opale note: width = textBlock.w + marginX1(6) + marginX2(15)
        // height = textBlock.h + 2*marginY(5)
        let bw = full_text_w + OPALE_MARGIN_X1 + OPALE_MARGIN_X2;
        let bh = ch + 2.0 * OPALE_MARGIN_Y;
        elements.push(EbnfElement::Comment {
            x: FW_MARGIN,
            y,
            width: bw,
            height: bh,
            text: comment.clone(),
        });
        y += bh + NOTE_MARGIN_BOTTOM;
        body_width = body_width.max(bw);
    }

    // Each rule expression: mergeTB(TitleBox(name), withMargin(WithCircles(inner), 0,0,10,15))
    for rule in &diagram.rules {
        let (re, wc_w, rule_h) = layout_rule(rule, y)?;
        elements.extend(re);
        body_width = body_width.max(wc_w);
        y += rule_h;
    }

    // Center the diagram title using the body width (Java DecorateEntityImage centering)
    // Java: dimTotal.width = max(body_w, title_block_w), centering within that
    let centering_width = body_width.max(title_block_w);
    if diagram.title.is_some() {
        if let Some(EbnfElement::Title { x, text, .. }) = elements.first_mut() {
            let tw = font_metrics::text_width(text, "SansSerif", TITLE_FONT_SIZE, true, false);
            // title_block x within centering area:
            let title_block_x = (centering_width - title_block_w) / 2.0;
            // text x within title_block: margin + padding
            *x = FW_MARGIN + title_block_x + TITLE_MARGIN + TITLE_PAD;
        }
    }

    // Canvas dimensions
    // Width: content extends to fw_margin + wc_w + end_circle_overshoot(WC_SIZE/2)
    // Total width includes fw_margin on both sides + the end circle radius
    let max_display_w = FW_MARGIN + body_width + WC_SIZE / 2.0;
    let canvas_w = max_display_w + FW_MARGIN; // right margin
    // Height: content + bottom framework margin.
    // When the diagram has a title, the Java TextBlockBordered adds +1 to
    // its calculated height, which propagates into the final SVG viewport.
    let title_extra = if diagram.title.is_some() { 1.0 } else { 0.0 };
    let canvas_h = y + FW_MARGIN + title_extra;

    Ok(EbnfLayout {
        width: canvas_w,
        height: canvas_h,
        elements,
    })
}

fn layout_rule(rule: &EbnfRule, start_y: f64) -> Result<(Vec<EbnfElement>, f64, f64)> {
    let mut elements = Vec::new();

    // TitleBox: draws rule name as bold text
    let asc = font_metrics::ascent("SansSerif", FONT_SIZE, true, false);
    let desc = font_metrics::descent("SansSerif", FONT_SIZE, true, false);
    let title_h = asc + desc;
    let title_baseline = start_y + asc;
    elements.push(EbnfElement::RuleName {
        x: FW_MARGIN,
        y: title_baseline,
        text: rule.name.clone(),
    });

    // Main tile (WithCircles wrapping the expression)
    let inner_dim = tile_dim(&rule.expr);
    // WithCircles: width = inner + 2*deltax, h1/h2 = inner h1/h2
    let wc_w = inner_dim.width + 2.0 * WC_DELTAX;
    let wc_h = inner_dim.h1 + inner_dim.h2;

    // main starts at y = start_y + title_h + margin_top
    let main_y = start_y + title_h + EXPR_MARGIN_TOP;
    // linePos (rail center) = main_y + wc.h1
    let line_pos = main_y + inner_dim.h1;

    // Draw inner expression within WithCircles
    let inner_x = FW_MARGIN + WC_DELTAX;
    draw_tile(&rule.expr, inner_x, main_y, line_pos, inner_dim.width, &inner_dim, &mut elements)?;

    // Draw WithCircles: circles + connecting lines
    let full_w = wc_w;
    // Start circle: at (0, linePos - SIZE/2) → cx = 0 + SIZE/2
    let start_cx = FW_MARGIN + WC_SIZE / 2.0;
    elements.push(EbnfElement::StartCircle {
        cx: start_cx,
        cy: line_pos,
        r: WC_SIZE / 2.0,
    });
    // End circle: at (fullW - SIZE/2, linePos - SIZE/2) → cx = fullW - SIZE/2 + SIZE/2
    // Actually: drawn at (fullW - SIZE/2, linePos - SIZE/2), UEllipse(SIZE, SIZE)
    // cx = fullW - SIZE/2 + SIZE/2 = fullW. Wait no.
    // draw position x = fullW - SIZE/2, ellipse width = SIZE, so center = fullW - SIZE/2 + SIZE/2 = fullW
    // Hmm but reference shows end cx = 137.7754 and fullW = 127.7754 + 10(fw_margin) = 137.7754
    // Actually: WithCircles.drawU at (deltax=30), inner drawn
    // end circle at (fullDim.width - SIZE/2, linePos - SIZE/2) = (127.7754 - 4, ...)
    // UEllipse center = position + SIZE/2 = (127.7754 - 4 + 4, ...) = (127.7754, ...)
    // With fw_margin offset: cx = 10 + 127.7754 = 137.7754 ✓
    let end_cx = FW_MARGIN + full_w;
    elements.push(EbnfElement::EndCircle {
        cx: end_cx,
        cy: line_pos,
        r: WC_SIZE / 2.0,
    });

    // Connecting lines from start circle to inner, and inner to end circle
    // Start: from SIZE to deltax
    let hline_start_x1 = FW_MARGIN + WC_SIZE;
    let hline_start_x2 = FW_MARGIN + WC_DELTAX;
    elements.push(EbnfElement::HLine {
        x1: hline_start_x1,
        y1: line_pos,
        x2: hline_start_x2,
        y2: line_pos,
        stroke_width: STROKE,
    });

    // End: from inner right to end circle - SIZE/2
    let hline_end_x1 = FW_MARGIN + full_w - WC_DELTAX;
    let hline_end_x2 = FW_MARGIN + full_w - WC_SIZE / 2.0;
    elements.push(EbnfElement::HLine {
        x1: hline_end_x1,
        y1: line_pos,
        x2: hline_end_x2,
        y2: line_pos,
        stroke_width: STROKE,
    });

    // Arrow on the end connecting line (coef=0.5, threshold=25)
    if hline_end_x2 > hline_end_x1 + 25.0 {
        let arrow_x = hline_end_x1 * 0.5 + hline_end_x2 * 0.5 - 2.0;
        elements.push(EbnfElement::Arrow { x: arrow_x, y: line_pos });
    }

    // Return wc_w (the WithCircles body width, used for title centering)
    // and total_h (expression height including margins)
    let total_h = title_h + EXPR_MARGIN_TOP + wc_h + EXPR_MARGIN_BOTTOM;

    Ok((elements, wc_w, total_h))
}

fn draw_tile(
    expr: &EbnfExpr,
    x: f64,        // absolute x of this tile's left edge
    top_y: f64,    // absolute y of this tile's top
    line_pos: f64, // absolute y of the rail center
    _tile_w: f64,  // tile width (for padding lines)
    _dim: &TileDim,
    elements: &mut Vec<EbnfElement>,
) -> Result<()> {
    match expr {
        EbnfExpr::Terminal(text) | EbnfExpr::Special(text) => {
            let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
            let bw = tw + 2.0 * BOX_PAD;
            let bh = font_metrics::ascent("SansSerif", FONT_SIZE, false, false)
                + font_metrics::descent("SansSerif", FONT_SIZE, false, false)
                + 2.0 * BOX_PAD;
            let box_y = line_pos - bh / 2.0;
            elements.push(EbnfElement::TerminalBox {
                x,
                y: box_y,
                width: bw,
                height: bh,
                text: text.clone(),
            });
        }
        EbnfExpr::NonTerminal(text) => {
            let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
            let bw = tw + 2.0 * BOX_PAD;
            let bh = font_metrics::ascent("SansSerif", FONT_SIZE, false, false)
                + font_metrics::descent("SansSerif", FONT_SIZE, false, false)
                + 2.0 * BOX_PAD;
            let box_y = line_pos - bh / 2.0;
            elements.push(EbnfElement::NonTerminalBox {
                x,
                y: box_y,
                width: bw,
                height: bh,
                text: text.clone(),
            });
        }
        EbnfExpr::Alternation(alts) => {
            let a = 0.0_f64;
            let b = a + ALT_MARGINX;
            let c = b + ALT_MARGINX;

            let alt_dim = tile_dim(expr);
            let full_w = alt_dim.width;
            let r = full_w;
            let q = r - ALT_MARGINX;
            let p = q - ALT_MARGINX;

            // Compute max inner width for padding lines
            let mut max_inner_w = 0.0f64;
            for alt in alts {
                let d = tile_dim(alt);
                max_inner_w = max_inner_w.max(d.width);
            }

            let alt_line_pos = line_pos; // first alt rail center
            let mut tile_y = top_y; // current tile top y
            let mut last_line_pos = alt_line_pos;

            for (i, alt) in alts.iter().enumerate() {
                let d = tile_dim(alt);
                let tile_line_pos = tile_y + d.h1;
                last_line_pos = tile_line_pos;

                // Draw inner tile at x+c
                draw_tile(alt, x + c, tile_y, tile_line_pos, d.width, &d, elements)?;

                if i == 0 {
                    // First alt: direct horizontal lines
                    elements.push(EbnfElement::HLine {
                        x1: x + a, y1: tile_line_pos,
                        x2: x + c, y2: tile_line_pos,
                        stroke_width: STROKE,
                    });
                    elements.push(EbnfElement::HLine {
                        x1: x + c + d.width, y1: tile_line_pos,
                        x2: x + r, y2: tile_line_pos,
                        stroke_width: STROKE,
                    });
                } else if i > 0 && i < alts.len() - 1 {
                    // Middle alts: corner curves + padding line
                    corner_sw(elements, ALT_MARGINX, x + b, tile_line_pos);
                    elements.push(EbnfElement::HLine {
                        x1: x + c + d.width, y1: tile_line_pos,
                        x2: x + p, y2: tile_line_pos,
                        stroke_width: STROKE,
                    });
                    corner_se(elements, ALT_MARGINX, x + q, tile_line_pos);
                } else {
                    // Last alt: corner curves + padding line (no arrow check for now)
                    elements.push(EbnfElement::HLine {
                        x1: x + c + d.width, y1: tile_line_pos,
                        x2: x + p, y2: tile_line_pos,
                        stroke_width: STROKE,
                    });
                }

                tile_y += d.h1 + d.h2 + ALT_GAP;
            }

            // Draw the vertical connections and corner curves.
            // Java order: SW(bottom), VLine, NE(top) on left; SE(bottom), VLine, NW(top) on right
            let height42 = last_line_pos - alt_line_pos;

            // Left side: SW corner at bottom, VLine, NE corner at top
            corner_sw(elements, ALT_MARGINX, x + b, alt_line_pos + height42);
            if height42 > 2.0 * ALT_MARGINX {
                elements.push(EbnfElement::VLine {
                    x1: x + b, y1: alt_line_pos + ALT_MARGINX,
                    x2: x + b, y2: alt_line_pos + height42 - ALT_MARGINX,
                    stroke_width: STROKE,
                });
            }
            corner_ne(elements, ALT_MARGINX, x + b, alt_line_pos);

            // Right side: SE corner at bottom, VLine, NW corner at top
            corner_se(elements, ALT_MARGINX, x + q, alt_line_pos + height42);
            if height42 > 2.0 * ALT_MARGINX {
                elements.push(EbnfElement::VLine {
                    x1: x + q, y1: alt_line_pos + ALT_MARGINX,
                    x2: x + q, y2: alt_line_pos + height42 - ALT_MARGINX,
                    stroke_width: STROKE,
                });
            }
            corner_nw(elements, ALT_MARGINX, x + q, alt_line_pos);
        }
        EbnfExpr::Sequence(parts) => {
            // Java Concatenation.drawU: drawHline(ug, fullLinePos, 0, x=0) first (zero-length)
            let full_dim = tile_dim(expr);
            let full_line_pos = line_pos;
            let mut cx = x;
            // Initial zero-length hline (Java: drawHline(ug, fullLinePos, 0, 0))
            elements.push(EbnfElement::HLine {
                x1: cx, y1: full_line_pos, x2: cx, y2: full_line_pos, stroke_width: STROKE,
            });
            for (i, part) in parts.iter().enumerate() {
                let d = tile_dim(part);
                let part_top = top_y + (full_dim.h1 - d.h1);
                let part_line = part_top + d.h1;
                draw_tile(part, cx, part_top, part_line, d.width, &d, elements)?;
                cx += d.width;
                if i < parts.len() - 1 {
                    // drawHlineDirected(ug, fullLinePos, x, x+marginx, 0.5, 25)
                    // marginx=20 < 25, so no arrow
                    elements.push(EbnfElement::HLine {
                        x1: cx, y1: full_line_pos,
                        x2: cx + CONCAT_MARGINX, y2: full_line_pos,
                        stroke_width: STROKE,
                    });
                    cx += CONCAT_MARGINX;
                }
            }
        }
        EbnfExpr::Optional(inner) => {
            // ETileOptional2 drawU order:
            // 1. HlineDirected at linePos from 0 to fullW (coef=0.4, threshold=25)
            // 2. Zigzag pathDown at (0, linePos)
            // 3. Zigzag pathUp at (fullW - 2*corner, linePos) where corner=12
            // 4. Inner at (deltax=24, getDeltaY=20)
            let d = tile_dim(inner);
            let full_w = d.width + 2.0 * OPT2_DELTAX;
            let lp = line_pos; // linePos = top_y + h1 = top_y + 10

            // 1. Hline at linePos across full width
            elements.push(EbnfElement::HLine {
                x1: x, y1: lp, x2: x + full_w, y2: lp, stroke_width: STROKE,
            });
            // Arrow on the hline (coef=0.4, threshold=25)
            if full_w > 25.0 {
                let arrow_x = x * 0.6 + (x + full_w) * 0.4 - 2.0;
                elements.push(EbnfElement::Arrow { x: arrow_x, y: lp });
            }

            // 2. Zigzag pathDown at (0, linePos): S-curve from rail down to inner start
            // Zigzag(ctrl=9, width=2*corner=24, height=getDeltaY+orig.h1-linePos_relative)
            // getDeltaY = OPT2_DELTAY = 20 (no notes)
            // linePos_relative = OPT2_H1 = 10
            // height = 20 + d.h1 - 10 = 10 + d.h1
            let corner = OPT2_DELTAX / 2.0; // 12
            let zw = 2.0 * corner; // 24
            let zh = OPT2_DELTAY + d.h1 - OPT2_H1; // 10 + d.h1
            let ctrl = 9.0;
            zigzag_down(elements, x, lp, zw, zh, ctrl);

            // 3. Zigzag pathUp at (fullW - 2*corner, linePos)
            zigzag_up(elements, x + full_w - zw, lp, zw, zh, ctrl);

            // 4. Inner at (deltax=24, getDeltaY=20)
            let inner_x = x + OPT2_DELTAX;
            let inner_top = top_y + OPT2_DELTAY;
            let inner_lp = inner_top + d.h1;
            draw_tile(inner, inner_x, inner_top, inner_lp, d.width, &d, elements)?;
        }
        EbnfExpr::Repetition(inner) => {
            // ETileOneOrMore drawU order (no loop text, getBraceHeight=0):
            // 1. SW(8) at (8, h1)
            // 2. VLine at x=8 from y=13 to y=h1-8
            // 3. NW(8) at (8, 5)
            // 4. HlineAntiDirected at y=5 from x=deltax to x=fullW-deltax (coef=0.6)
            // 5. SE(8) at (fullW-8, h1)
            // 6. VLine at x=fullW-8 from y=13 to y=h1-8
            // 7. NE(8) at (fullW-8, 5)
            // 8. HLine at h1 from 0 to deltax
            // 9. HLine at h1 from fullW-deltax to fullW
            // 10. Inner at (deltax, deltay)
            let d = tile_dim(inner);
            let full_w = d.width + 2.0 * OOM_DELTAX;
            let h1 = OOM_DELTAY + d.h1;
            let lp = top_y + h1; // absolute linePos

            // 1. SW(8) at (8, h1)
            corner_sw(elements, OOM_CORNER, x + 8.0, lp);
            // 2. VLine at x=8 from y=8+5=13 to y=h1-8
            if h1 > 21.0 {
                elements.push(EbnfElement::VLine {
                    x1: x + 8.0, y1: top_y + 13.0,
                    x2: x + 8.0, y2: top_y + h1 - 8.0,
                    stroke_width: STROKE,
                });
            }
            // 3. NW(8) at (8, 5)
            corner_nw(elements, OOM_CORNER, x + 8.0, top_y + 5.0);
            // 4. HlineAntiDirected at y=5 from x=deltax(15) to x=fullW-deltax(15)
            let hline_y = top_y + 5.0;
            let hline_x1 = x + OOM_DELTAX;
            let hline_x2 = x + full_w - OOM_DELTAX;
            elements.push(EbnfElement::HLine {
                x1: hline_x1, y1: hline_y, x2: hline_x2, y2: hline_y, stroke_width: STROKE,
            });
            // Anti-directed arrow (points LEFT, coef=0.6)
            let anti_arrow_x = hline_x1 * (1.0 - 0.6) + hline_x2 * 0.6 - 2.0;
            elements.push(EbnfElement::LeftArrow { x: anti_arrow_x, y: hline_y });

            // 5. SE(8) at (fullW-8, h1)
            corner_se(elements, OOM_CORNER, x + full_w - 8.0, lp);
            // 6. VLine at x=fullW-8 from y=13 to y=h1-8
            if h1 > 21.0 {
                elements.push(EbnfElement::VLine {
                    x1: x + full_w - 8.0, y1: top_y + 13.0,
                    x2: x + full_w - 8.0, y2: top_y + h1 - 8.0,
                    stroke_width: STROKE,
                });
            }
            // 7. NE(8) at (fullW-8, 5)
            corner_ne(elements, OOM_CORNER, x + full_w - 8.0, top_y + 5.0);
            // 8. HLine at h1 from 0 to deltax
            elements.push(EbnfElement::HLine {
                x1: x, y1: lp, x2: x + OOM_DELTAX, y2: lp, stroke_width: STROKE,
            });
            // 9. HLine at h1 from fullW-deltax to fullW
            elements.push(EbnfElement::HLine {
                x1: x + full_w - OOM_DELTAX, y1: lp, x2: x + full_w, y2: lp, stroke_width: STROKE,
            });
            // 10. Inner at (deltax, deltay)
            let inner_top = top_y + OOM_DELTAY;
            let inner_lp = inner_top + d.h1;
            draw_tile(inner, x + OOM_DELTAX, inner_top, inner_lp, d.width, &d, elements)?;
        }
        EbnfExpr::Group(inner) => {
            draw_tile(inner, x, top_y, line_pos, _tile_w, _dim, elements)?;
        }
    }
    Ok(())
}

// ── Zigzag S-curve paths (match Java Zigzag.java) ────────────────

fn zigzag_down(elements: &mut Vec<EbnfElement>, ox: f64, oy: f64, width: f64, height: f64, ctrl: f64) {
    let xm = width / 2.0;
    let ym = height / 2.0;
    let d = format!(
        "M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{}",
        ff(ox), ff(oy),
        ff(ox + ctrl), ff(oy),
        ff(ox + xm), ff(oy + ym - ctrl),
        ff(ox + xm), ff(oy + ym),
        ff(ox + xm), ff(oy + ym + ctrl),
        ff(ox + width - ctrl), ff(oy + height),
        ff(ox + width), ff(oy + height)
    );
    elements.push(EbnfElement::Path { d, fill: false, stroke_width: STROKE });
}

fn zigzag_up(elements: &mut Vec<EbnfElement>, ox: f64, oy: f64, width: f64, height: f64, ctrl: f64) {
    let xm = width / 2.0;
    let ym = height / 2.0;
    let d = format!(
        "M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{}",
        ff(ox), ff(oy + height),
        ff(ox + ctrl), ff(oy + height),
        ff(ox + xm), ff(oy + ym + ctrl),
        ff(ox + xm), ff(oy + ym),
        ff(ox + xm), ff(oy + ym - ctrl),
        ff(ox + width - ctrl), ff(oy),
        ff(ox + width), ff(oy)
    );
    elements.push(EbnfElement::Path { d, fill: false, stroke_width: STROKE });
}

// ── CornerCurved path helpers (match Java CornerCurved.java) ─────

fn corner_sw(elements: &mut Vec<EbnfElement>, delta: f64, cx: f64, cy: f64) {
    let a = delta / 4.0;
    let d = format!(
        "M{},{} C{},{} {},{} {},{}",
        ff(cx), ff(cy - delta),
        ff(cx), ff(cy - a),
        ff(cx + a), ff(cy),
        ff(cx + delta), ff(cy)
    );
    elements.push(EbnfElement::Path { d, fill: false, stroke_width: STROKE });
}

fn corner_se(elements: &mut Vec<EbnfElement>, delta: f64, cx: f64, cy: f64) {
    let a = delta / 4.0;
    let d = format!(
        "M{},{} C{},{} {},{} {},{}",
        ff(cx), ff(cy - delta),
        ff(cx), ff(cy - a),
        ff(cx - a), ff(cy),
        ff(cx - delta), ff(cy)
    );
    elements.push(EbnfElement::Path { d, fill: false, stroke_width: STROKE });
}

fn corner_ne(elements: &mut Vec<EbnfElement>, delta: f64, cx: f64, cy: f64) {
    let a = delta / 4.0;
    let d = format!(
        "M{},{} C{},{} {},{} {},{}",
        ff(cx - delta), ff(cy),
        ff(cx - a), ff(cy),
        ff(cx), ff(cy + a),
        ff(cx), ff(cy + delta)
    );
    elements.push(EbnfElement::Path { d, fill: false, stroke_width: STROKE });
}

fn corner_nw(elements: &mut Vec<EbnfElement>, delta: f64, cx: f64, cy: f64) {
    let a = delta / 4.0;
    let d = format!(
        "M{},{} C{},{} {},{} {},{}",
        ff(cx), ff(cy + delta),
        ff(cx), ff(cy + a),
        ff(cx + a), ff(cy),
        ff(cx + delta), ff(cy)
    );
    elements.push(EbnfElement::Path { d, fill: false, stroke_width: STROKE });
}

#[inline]
fn ff(v: f64) -> String {
    crate::klimt::svg::fmt_coord(v)
}
