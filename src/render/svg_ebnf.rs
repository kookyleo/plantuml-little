use std::fmt::Write;
use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape};
use crate::layout::ebnf::{EbnfElement, EbnfLayout};
use crate::model::ebnf::EbnfDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const COMMENT_FS: f64 = 13.0;
const STROKE: &str = "#181818";
const TEXT_C: &str = "#000000";
const NOTE_BG: &str = "#FEFFDD";

pub fn render_ebnf(_d: &EbnfDiagram, l: &EbnfLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(8192);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let (sw, sh) = (ensure_visible_int(l.width) as f64, ensure_visible_int(l.height) as f64);
    write_svg_root_bg(&mut buf, sw, sh, "EBNF", bg);
    // Emit SVG <title> metadata from layout title (Java does this via SvgGraphics)
    if let Some(title_text) = l.elements.iter().find_map(|e| match e {
        EbnfElement::Title { text, .. } => Some(text.as_str()),
        _ => None,
    }) {
        crate::render::svg::write_svg_title(&mut buf, title_text);
    }
    // Java EBNF does not draw a background rect — background is in SVG root style.
    buf.push_str("<defs/><g>");
    for e in &l.elements {
        match e {
            EbnfElement::Title { x, y, text } => {
                let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, true, false);
                write!(buf, r#"<g class="title" data-source-line="1"><text fill="{}" font-family="sans-serif" font-size="{}" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text></g>"#,
                    TEXT_C, FONT_SIZE as i32, ff(tw), ff(*x), ff(*y), xml_escape(text)).unwrap();
            }
            EbnfElement::Comment { x, y, width, height, text } => {
                let fold = 10.0;
                write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{} L{},{}" fill="{}" style="stroke:{};stroke-width:0.5;"/>"#,
                    ff(*x), ff(*y), ff(*x), ff(*y + *height), ff(*x + *width), ff(*y + *height), ff(*x + *width), ff(*y + fold), ff(*x + *width - fold), ff(*y), ff(*x), ff(*y), NOTE_BG, STROKE).unwrap();
                write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{}" fill="{}" style="stroke:{};stroke-width:0.5;"/>"#,
                    ff(*x + *width - fold), ff(*y), ff(*x + *width - fold), ff(*y + fold), ff(*x + *width), ff(*y + fold), ff(*x + *width - fold), ff(*y), NOTE_BG, STROKE).unwrap();
                let asc = font_metrics::ascent("SansSerif", COMMENT_FS, false, false);
                let tw = font_metrics::text_width(text, "SansSerif", COMMENT_FS, false, false);
                // Opale draws textBlock at (marginX1=6, marginY=5).
                // The text " comment " (with spaces from parser) has the leading space width
                // offset. Text x = note_x + marginX1 + space_width.
                let space_w = font_metrics::text_width(" ", "SansSerif", COMMENT_FS, false, false);
                let text_x = *x + 6.0 + space_w; // Opale marginX1 + space offset
                let text_y = *y + 5.0 + asc; // Opale marginY + ascent
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, COMMENT_FS as i32, ff(tw), ff(text_x), ff(text_y), xml_escape(text)).unwrap();
            }
            EbnfElement::RuleName { x, y, text } => {
                let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, true, false);
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, FONT_SIZE as i32, ff(tw), ff(*x), ff(*y), xml_escape(text)).unwrap();
            }
            EbnfElement::TerminalBox { x, y, width, height, text } => {
                write!(buf, r#"<rect fill="none" height="{}" style="stroke:{};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
                    ff(*height), STROKE, ff(*width), ff(*x), ff(*y)).unwrap();
                let asc = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
                let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, FONT_SIZE as i32, ff(tw), ff(*x + 5.0), ff(*y + asc + 5.0), xml_escape(text)).unwrap();
            }
            EbnfElement::HLine { x1, y1, x2, y2, stroke_width } | EbnfElement::VLine { x1, y1, x2, y2, stroke_width } => {
                write!(buf, r#"<line style="stroke:{};stroke-width:{};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                    STROKE, ff(*stroke_width), ff(*x1), ff(*x2), ff(*y1), ff(*y2)).unwrap();
            }
            EbnfElement::Path { d, fill, stroke_width } => {
                let f = if *fill { STROKE } else { "none" };
                write!(buf, r#"<path d="{}" fill="{}" style="stroke:{};stroke-width:{};"/>"#, d, f, STROKE, ff(*stroke_width)).unwrap();
            }
            EbnfElement::StartCircle { cx, cy, r } => {
                write!(buf, r#"<ellipse cx="{}" cy="{}" fill="none" rx="{}" ry="{}" style="stroke:{};stroke-width:2;"/>"#, ff(*cx), ff(*cy), ff(*r), ff(*r), STROKE).unwrap();
            }
            EbnfElement::EndCircle { cx, cy, r } => {
                write!(buf, r#"<ellipse cx="{}" cy="{}" fill="{}" rx="{}" ry="{}" style="stroke:{};stroke-width:1;"/>"#, ff(*cx), ff(*cy), STROKE, ff(*r), ff(*r), STROKE).unwrap();
            }
            EbnfElement::NonTerminalBox { x, y, width, height, text } => {
                // Rounded rect with fill #F1F1F1, stroke 1.5, corner 10
                write!(buf, r##"<rect fill="#F1F1F1" height="{}" rx="5" ry="5" style="stroke:{};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"##,
                    ff(*height), STROKE, ff(*width), ff(*x), ff(*y)).unwrap();
                let asc = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
                let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, FONT_SIZE as i32, ff(tw), ff(*x + 5.0), ff(*y + asc + 5.0), xml_escape(text)).unwrap();
            }
            EbnfElement::Arrow { x, y } => {
                write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{}" fill="{}"/>"#,
                    ff(*x), ff(*y), ff(*x), ff(*y - 3.0), ff(*x + 6.0), ff(*y), ff(*x), ff(*y + 3.0), ff(*x), ff(*y), STROKE).unwrap();
            }
            EbnfElement::LeftArrow { x, y } => {
                write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{}" fill="{}"/>"#,
                    ff(*x), ff(*y), ff(*x), ff(*y - 3.0), ff(*x - 6.0), ff(*y), ff(*x), ff(*y + 3.0), ff(*x), ff(*y), STROKE).unwrap();
            }
        }
    }
    buf.push_str("</g></svg>"); Ok(buf)
}

#[inline]
fn ff(v: f64) -> String { fmt_coord(v) }
