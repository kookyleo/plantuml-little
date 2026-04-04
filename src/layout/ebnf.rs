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
    HLine { x1: f64, y1: f64, x2: f64, y2: f64, stroke_width: f64 },
    VLine { x1: f64, y1: f64, x2: f64, y2: f64, stroke_width: f64 },
    Path { d: String, fill: bool, stroke_width: f64 },
    StartCircle { cx: f64, cy: f64, r: f64 },
    EndCircle { cx: f64, cy: f64, r: f64 },
    Arrow { x: f64, y: f64 },
}

const FONT_SIZE: f64 = 14.0;
const BOX_PAD_X: f64 = 5.0;
const BOX_HEIGHT: f64 = 26.2969;
const STROKE_WIDTH: f64 = 1.5;
const CIRCLE_R: f64 = 4.0;
const RAIL_LEFT: f64 = 40.0;
const RAIL_RIGHT_PAD: f64 = 30.0;
const RULE_START_X: f64 = 10.0;
const BRANCH_RADIUS: f64 = 12.0;
const BRANCH_GAP_Y: f64 = 36.2969;
const TITLE_FONT_SIZE: f64 = 14.0;
const COMMENT_FONT_SIZE: f64 = 13.0;

pub fn layout_ebnf(diagram: &EbnfDiagram) -> Result<EbnfLayout> {
    let mut elements = Vec::new();
    let mut y = 10.0;
    let mut max_width = 100.0f64;
    if let Some(title) = &diagram.title {
        let tw = font_metrics::text_width(title, "SansSerif", TITLE_FONT_SIZE, true, false);
        let asc = font_metrics::ascent("SansSerif", TITLE_FONT_SIZE, true, false);
        let desc = font_metrics::descent("SansSerif", TITLE_FONT_SIZE, true, false);
        elements.push(EbnfElement::Title { x: (max_width - tw) / 2.0, y: y + asc, text: title.clone() });
        y += asc + desc + 16.0;
    }
    if let Some(comment) = &diagram.comment {
        let cw = font_metrics::text_width(comment, "SansSerif", COMMENT_FONT_SIZE, false, false);
        let ch = font_metrics::ascent("SansSerif", COMMENT_FONT_SIZE, false, false) + font_metrics::descent("SansSerif", COMMENT_FONT_SIZE, false, false);
        let bw = cw + 30.0; let bh = ch + 15.0;
        elements.push(EbnfElement::Comment { x: RULE_START_X, y, width: bw, height: bh, text: comment.clone() });
        y += bh + 5.0; max_width = max_width.max(RULE_START_X + bw + 10.0);
    }
    for rule in &diagram.rules {
        let (re, rh, rw) = layout_rule(rule, y)?;
        elements.extend(re); y += rh + 10.0; max_width = max_width.max(rw);
    }
    if diagram.title.is_some() {
        if let Some(EbnfElement::Title { x, text, .. }) = elements.first_mut() {
            let tw = font_metrics::text_width(text, "SansSerif", TITLE_FONT_SIZE, true, false);
            *x = (max_width - tw) / 2.0;
        }
    }
    Ok(EbnfLayout { width: max_width, height: y, elements })
}

fn layout_rule(rule: &EbnfRule, start_y: f64) -> Result<(Vec<EbnfElement>, f64, f64)> {
    let mut elements = Vec::new();
    let asc = font_metrics::ascent("SansSerif", FONT_SIZE, true, false);
    let desc = font_metrics::descent("SansSerif", FONT_SIZE, true, false);
    let name_y = start_y + asc + desc + 10.0;
    elements.push(EbnfElement::RuleName { x: RULE_START_X, y: name_y, text: rule.name.clone() });
    let rail_y = name_y + 13.0;
    let (ee, ew, eh) = layout_expr(&rule.expr, RAIL_LEFT, rail_y)?;
    elements.extend(ee);
    let rail_end = RAIL_LEFT + ew;
    let total_w = rail_end + RAIL_RIGHT_PAD;
    let cx = RULE_START_X + CIRCLE_R;
    elements.push(EbnfElement::StartCircle { cx, cy: rail_y, r: CIRCLE_R });
    elements.push(EbnfElement::HLine { x1: cx + CIRCLE_R, y1: rail_y, x2: RAIL_LEFT, y2: rail_y, stroke_width: STROKE_WIDTH });
    let ecx = total_w - CIRCLE_R - 4.0;
    elements.push(EbnfElement::HLine { x1: rail_end, y1: rail_y, x2: ecx - CIRCLE_R, y2: rail_y, stroke_width: STROKE_WIDTH });
    elements.push(EbnfElement::Arrow { x: (rail_end + ecx) / 2.0, y: rail_y });
    elements.push(EbnfElement::EndCircle { cx: ecx, cy: rail_y, r: CIRCLE_R });
    let rh = (name_y - start_y) + 13.0 + eh + 10.0;
    Ok((elements, rh, total_w))
}

fn layout_expr(expr: &EbnfExpr, x: f64, cy: f64) -> Result<(Vec<EbnfElement>, f64, f64)> {
    match expr {
        EbnfExpr::Terminal(text) | EbnfExpr::NonTerminal(text) | EbnfExpr::Special(text) => {
            let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
            let bw = tw + 2.0 * BOX_PAD_X;
            Ok((vec![EbnfElement::TerminalBox { x, y: cy - BOX_HEIGHT / 2.0, width: bw, height: BOX_HEIGHT, text: text.clone() }], bw, BOX_HEIGHT))
        }
        EbnfExpr::Alternation(alts) => {
            let mut elts = Vec::new();
            let mut mw = 0.0f64;
            for a in alts { let (_, w, _) = layout_expr(a, 0.0, 0.0)?; mw = mw.max(w); }
            let mut acy = cy;
            for (i, a) in alts.iter().enumerate() {
                let (ae, aw, _) = layout_expr(a, x, acy)?;
                elts.extend(ae);
                elts.push(EbnfElement::HLine { x1: x + aw, y1: acy, x2: x + mw, y2: acy, stroke_width: STROKE_WIDTH });
                if i > 0 {
                    let lx = x - BRANCH_RADIUS; let rx = x + mw + BRANCH_RADIUS;
                    elts.push(EbnfElement::Path { d: format!("M{},{} C{},{} {},{} {},{}", lx, acy - BRANCH_GAP_Y, lx, acy, lx + 3.0, acy, x, acy), fill: false, stroke_width: STROKE_WIDTH });
                    elts.push(EbnfElement::Path { d: format!("M{},{} C{},{} {},{} {},{}", x + mw, acy, rx - 3.0, acy, rx, acy, rx, acy - BRANCH_GAP_Y), fill: false, stroke_width: STROKE_WIDTH });
                    if i > 1 {
                        elts.push(EbnfElement::VLine { x1: lx, y1: acy - BRANCH_GAP_Y, x2: lx, y2: acy, stroke_width: STROKE_WIDTH });
                        elts.push(EbnfElement::VLine { x1: rx, y1: acy - BRANCH_GAP_Y, x2: rx, y2: acy, stroke_width: STROKE_WIDTH });
                    }
                }
                if i < alts.len() - 1 { acy += BRANCH_GAP_Y; }
            }
            Ok((elts, mw, (acy - cy) + BOX_HEIGHT))
        }
        EbnfExpr::Sequence(parts) => {
            let mut elts = Vec::new(); let mut cx = x; let mut mh = BOX_HEIGHT;
            for p in parts { let (pe, pw, ph) = layout_expr(p, cx, cy)?; elts.extend(pe); cx += pw + 8.0; mh = mh.max(ph); }
            Ok((elts, cx - x, mh))
        }
        EbnfExpr::Optional(inner) => {
            let (ie, iw, ih) = layout_expr(inner, x, cy)?;
            let mut elts = ie; let by = cy + ih + 10.0;
            elts.push(EbnfElement::Path { d: format!("M{},{} C{},{} {},{} {},{}", x, cy, x, by, x + 6.0, by, x + iw / 2.0, by), fill: false, stroke_width: STROKE_WIDTH });
            elts.push(EbnfElement::Path { d: format!("M{},{} C{},{} {},{} {},{}", x + iw / 2.0, by, x + iw - 6.0, by, x + iw, by, x + iw, cy), fill: false, stroke_width: STROKE_WIDTH });
            Ok((elts, iw, ih + 20.0))
        }
        EbnfExpr::Repetition(inner) => {
            let (ie, iw, ih) = layout_expr(inner, x, cy)?;
            let mut elts = ie; let ly = cy - 15.0;
            elts.push(EbnfElement::Path { d: format!("M{},{} C{},{} {},{} {},{}", x, cy, x, ly, x + 6.0, ly, x + iw / 2.0, ly), fill: false, stroke_width: STROKE_WIDTH });
            elts.push(EbnfElement::Path { d: format!("M{},{} C{},{} {},{} {},{}", x + iw / 2.0, ly, x + iw - 6.0, ly, x + iw, ly, x + iw, cy), fill: false, stroke_width: STROKE_WIDTH });
            Ok((elts, iw, ih + 20.0))
        }
        EbnfExpr::Group(inner) => layout_expr(inner, x, cy),
    }
}
