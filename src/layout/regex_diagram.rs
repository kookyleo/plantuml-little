use crate::font_metrics;
use crate::model::regex_diagram::{RegexDiagram, RegexNode};
use crate::Result;

#[derive(Debug)]
pub struct RegexLayout {
    pub width: f64,
    pub height: f64,
    pub elements: Vec<RegexElement>,
}

#[derive(Debug, Clone)]
pub enum RegexElement {
    LiteralBox {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        text: String,
        dashed: bool,
    },
    HLine {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke_width: f64,
    },
    Path {
        d: String,
        fill: bool,
        stroke_width: f64,
    },
    Arrow {
        x: f64,
        y: f64,
        points: [(f64, f64); 3],
    },
    Text {
        x: f64,
        y: f64,
        text: String,
        font_size: f64,
    },
}

const FONT_SIZE: f64 = 14.0;
const FONT_SIZE_QUANT: f64 = 12.0;
const BOX_PAD_X: f64 = 5.0;
const BOX_HEIGHT: f64 = 26.2969;
const GAP: f64 = 8.0;
const BRANCH_GAP: f64 = 6.0;
const LOOP_GAP: f64 = 20.0;

pub fn layout_regex(diagram: &RegexDiagram) -> Result<RegexLayout> {
    let mut elements = Vec::new();
    let mid_y = 55.1484;
    let start_x = 15.0;
    let (w, _) = measure_node(&diagram.node);
    layout_node(&diagram.node, start_x, mid_y, &mut elements);
    let (min_y, max_y) = compute_y_bounds(&elements);
    let total_w = start_x + w + 15.0;
    let total_h = (max_y - min_y) + 12.0;
    Ok(RegexLayout {
        width: total_w,
        height: total_h,
        elements,
    })
}

fn measure_node(node: &RegexNode) -> (f64, f64) {
    match node {
        RegexNode::Literal(text) => {
            let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
            (tw + 2.0 * BOX_PAD_X + GAP, BOX_HEIGHT)
        }
        RegexNode::CharClass(items) => {
            let mut mw = 0.0f64;
            for it in items {
                mw = mw.max(font_metrics::text_width(
                    it,
                    "SansSerif",
                    FONT_SIZE,
                    false,
                    false,
                ));
            }
            (
                mw + 2.0 * BOX_PAD_X + GAP + BRANCH_GAP * 2.0,
                BOX_HEIGHT + (items.len() as f64 - 1.0) * 16.2969,
            )
        }
        RegexNode::Concat(nodes) => {
            let mut tw = 0.0;
            let mut mh = BOX_HEIGHT;
            for n in nodes {
                let (w, h) = measure_node(n);
                tw += w;
                mh = mh.max(h);
            }
            (tw, mh)
        }
        RegexNode::Alternate(branches) => {
            let mut mw = 0.0f64;
            let mut th = 0.0;
            for (i, b) in branches.iter().enumerate() {
                let (w, h) = measure_node(b);
                mw = mw.max(w);
                th += h;
                if i > 0 {
                    th += 12.0;
                }
            }
            (mw + BRANCH_GAP * 4.0, th)
        }
        RegexNode::Quantifier { inner, label, .. } => {
            let (iw, ih) = measure_node(inner);
            let lw = font_metrics::text_width(label, "SansSerif", FONT_SIZE_QUANT, false, false);
            (iw.max(lw + 10.0) + LOOP_GAP, ih + 20.0)
        }
        RegexNode::Optional(inner) => {
            let (iw, ih) = measure_node(inner);
            (iw + LOOP_GAP + 12.0, ih + 23.1484)
        }
        RegexNode::Group(inner) => measure_node(inner),
    }
}

fn layout_node(node: &RegexNode, x: f64, cy: f64, elts: &mut Vec<RegexElement>) -> f64 {
    match node {
        RegexNode::Literal(text) => {
            let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
            let bw = tw + 2.0 * BOX_PAD_X;
            elts.push(RegexElement::LiteralBox {
                x,
                y: cy - BOX_HEIGHT / 2.0,
                width: bw,
                height: BOX_HEIGHT,
                text: text.clone(),
                dashed: false,
            });
            elts.push(RegexElement::HLine {
                x1: x - GAP / 2.0,
                y1: cy,
                x2: x,
                y2: cy,
                stroke_width: 1.0,
            });
            let ax = x + bw;
            elts.push(RegexElement::HLine {
                x1: ax,
                y1: cy,
                x2: ax + GAP / 2.0,
                y2: cy,
                stroke_width: 1.0,
            });
            ax + GAP / 2.0
        }
        RegexNode::CharClass(items) => {
            let mut mtw = 0.0f64;
            for it in items {
                mtw = mtw.max(font_metrics::text_width(
                    it,
                    "SansSerif",
                    FONT_SIZE,
                    false,
                    false,
                ));
            }
            let bw = mtw + 2.0 * BOX_PAD_X;
            let bh = BOX_HEIGHT + (items.len() as f64 - 1.0) * 16.2969;
            let bx = x + BRANCH_GAP;
            elts.push(RegexElement::LiteralBox {
                x: bx,
                y: cy - BOX_HEIGHT / 2.0,
                width: bw,
                height: bh,
                text: items.join("\n"),
                dashed: true,
            });
            elts.push(RegexElement::HLine {
                x1: bx,
                y1: cy,
                x2: bx + bw,
                y2: cy,
                stroke_width: 0.3,
            });
            elts.push(RegexElement::HLine {
                x1: x,
                y1: cy,
                x2: bx,
                y2: cy,
                stroke_width: 1.0,
            });
            let ax = bx + bw;
            elts.push(RegexElement::HLine {
                x1: ax,
                y1: cy,
                x2: ax + BRANCH_GAP,
                y2: cy,
                stroke_width: 1.0,
            });
            ax + BRANCH_GAP + GAP / 2.0
        }
        RegexNode::Concat(nodes) => {
            let mut cx = x;
            for n in nodes {
                cx = layout_node(n, cx, cy, elts);
            }
            cx
        }
        RegexNode::Alternate(branches) => {
            let (bw, _) = measure_node(node);
            let rx = x + bw;
            let mut by = cy;
            for (i, branch) in branches.iter().enumerate() {
                let (iw, ih) = measure_node(branch);
                let bx = x + (bw - iw) / 2.0;
                if i == 0 {
                    layout_node(branch, bx, by, elts);
                } else {
                    by += ih + 12.0;
                    layout_node(branch, bx, by, elts);
                    elts.push(RegexElement::Path {
                        d: format!(
                            "M{},{} C{},{} {},{} {},{}",
                            x,
                            cy,
                            x,
                            by,
                            x + BRANCH_GAP,
                            by,
                            bx,
                            by
                        ),
                        fill: false,
                        stroke_width: 1.0,
                    });
                    elts.push(RegexElement::Path {
                        d: format!(
                            "M{},{} C{},{} {},{} {},{}",
                            bx + iw,
                            by,
                            rx - BRANCH_GAP,
                            by,
                            rx,
                            by,
                            rx,
                            cy
                        ),
                        fill: false,
                        stroke_width: 1.0,
                    });
                }
            }
            rx
        }
        RegexNode::Quantifier {
            inner, label, max, ..
        } => {
            let (iw, _) = measure_node(inner);
            let inner_end = layout_node(inner, x, cy, elts);
            let ly = cy - 20.0;
            if *max != Some(1) {
                let ax = x + iw / 2.0 + 3.0;
                elts.push(RegexElement::Arrow {
                    x: ax,
                    y: ly,
                    points: [(ax, ly - 3.0), (ax - 6.0, ly), (ax, ly + 3.0)],
                });
            }
            if !label.is_empty() {
                let lw =
                    font_metrics::text_width(label, "SansSerif", FONT_SIZE_QUANT, false, false);
                elts.push(RegexElement::Text {
                    x: x + (iw - lw) / 2.0,
                    y: ly + 5.0,
                    text: label.clone(),
                    font_size: FONT_SIZE_QUANT,
                });
            }
            let r = BRANCH_GAP;
            elts.push(RegexElement::Path {
                d: format!(
                    "M{},{} C{},{} {},{} {},{}",
                    x,
                    cy - 5.0,
                    x,
                    ly,
                    x + r,
                    ly,
                    x + r * 2.0,
                    ly
                ),
                fill: false,
                stroke_width: 0.5,
            });
            elts.push(RegexElement::Path {
                d: format!(
                    "M{},{} C{},{} {},{} {},{}",
                    inner_end - r * 2.0,
                    ly,
                    inner_end - r,
                    ly,
                    inner_end,
                    ly,
                    inner_end,
                    cy - 5.0
                ),
                fill: false,
                stroke_width: 0.5,
            });
            elts.push(RegexElement::HLine {
                x1: x + r * 2.0,
                y1: ly,
                x2: inner_end - r * 2.0,
                y2: ly,
                stroke_width: 0.5,
            });
            inner_end
        }
        RegexNode::Optional(inner) => {
            let inner_end = layout_node(inner, x, cy, elts);
            let by = cy + 23.1484;
            let r = 9.0;
            elts.push(RegexElement::Path {
                d: format!(
                    "M{},{} C{},{} {},{} {},{}",
                    x,
                    cy,
                    x,
                    by - r,
                    x,
                    by,
                    x + r,
                    by
                ),
                fill: false,
                stroke_width: 1.0,
            });
            elts.push(RegexElement::Path {
                d: format!(
                    "M{},{} C{},{} {},{} {},{}",
                    inner_end - r,
                    by,
                    inner_end,
                    by,
                    inner_end,
                    by - r,
                    inner_end,
                    cy
                ),
                fill: false,
                stroke_width: 1.0,
            });
            inner_end
        }
        RegexNode::Group(inner) => layout_node(inner, x, cy, elts),
    }
}

fn compute_y_bounds(elements: &[RegexElement]) -> (f64, f64) {
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for e in elements {
        match e {
            RegexElement::LiteralBox { y, height, .. } => {
                min_y = min_y.min(*y);
                max_y = max_y.max(*y + *height);
            }
            RegexElement::HLine { y1, y2, .. } => {
                min_y = min_y.min(y1.min(*y2));
                max_y = max_y.max(y1.max(*y2));
            }
            RegexElement::Text { y, .. } => {
                min_y = min_y.min(*y - 12.0);
                max_y = max_y.max(*y + 4.0);
            }
            RegexElement::Arrow { points, .. } => {
                for (_, py) in points {
                    min_y = min_y.min(*py);
                    max_y = max_y.max(*py);
                }
            }
            RegexElement::Path { .. } => {}
        }
    }
    if min_y > max_y {
        (0.0, 100.0)
    } else {
        (min_y, max_y)
    }
}
