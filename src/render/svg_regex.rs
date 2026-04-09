use std::fmt::Write;
use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape};
use crate::layout::ebnf::{EbnfElement, EbnfLayout};
use crate::render::svg::{ensure_visible_int, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const STROKE: &str = "#181818";
const TEXT_C: &str = "#000000";

/// Render a regex diagram using the EBNF element layout.
/// Identical to render_ebnf but uses diagram type "REGEX" and no title.
pub fn render_regex_ebnf(l: &EbnfLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(8192);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let (sw, sh) = (ensure_visible_int(l.width) as f64, ensure_visible_int(l.height) as f64);
    write_svg_root_bg(&mut buf, sw, sh, "REGEX", bg);
    buf.push_str("<defs/><g>");
    for e in &l.elements {
        match e {
            EbnfElement::TerminalBox { x, y, width, height, text } => {
                write!(buf, r#"<rect fill="none" height="{}" style="stroke:{};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
                    ff(*height), STROKE, ff(*width), ff(*x), ff(*y)).unwrap();
                let asc = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
                let tw = font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false);
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, FONT_SIZE as i32, ff(tw), ff(*x + 5.0), ff(*y + asc + 5.0), xml_escape(text)).unwrap();
            }
            EbnfElement::DashedBox { x, y, width, height } => {
                write!(buf, r#"<rect fill="none" height="{}" style="stroke:{};stroke-width:1;stroke-dasharray:5,5;" width="{}" x="{}" y="{}"/>"#,
                    ff(*height), STROKE, ff(*width), ff(*x), ff(*y)).unwrap();
            }
            EbnfElement::TerminalText { x, y, width, text } => {
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, FONT_SIZE as i32, ff(*width), ff(*x), ff(*y), xml_escape(text)).unwrap();
            }
            EbnfElement::RepetitionLabel { x, y, width, text, font_size } => {
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, *font_size as i32, ff(*width), ff(*x), ff(*y), xml_escape(text)).unwrap();
            }
            EbnfElement::HLine { x1, y1, x2, y2, stroke_width }
            | EbnfElement::VLine { x1, y1, x2, y2, stroke_width } => {
                write!(buf, r#"<line style="stroke:{};stroke-width:{};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                    STROKE, ff(*stroke_width), ff(*x1), ff(*x2), ff(*y1), ff(*y2)).unwrap();
            }
            EbnfElement::Path { d, fill, stroke_width } => {
                let f = if *fill { STROKE } else { "none" };
                write!(buf, r#"<path d="{}" fill="{}" style="stroke:{};stroke-width:{};"/>"#, d, f, STROKE, ff(*stroke_width)).unwrap();
            }
            EbnfElement::Arrow { x, y } => {
                write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{}" fill="{}"/>"#,
                    ff(*x), ff(*y), ff(*x), ff(*y - 3.0), ff(*x + 6.0), ff(*y), ff(*x), ff(*y + 3.0), ff(*x), ff(*y), STROKE).unwrap();
            }
            EbnfElement::LeftArrow { x, y } => {
                write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{}" fill="{}"/>"#,
                    ff(*x), ff(*y), ff(*x), ff(*y - 3.0), ff(*x - 6.0), ff(*y), ff(*x), ff(*y + 3.0), ff(*x), ff(*y), STROKE).unwrap();
            }
            // Ignore EBNF-only elements
            EbnfElement::Title { .. } | EbnfElement::Comment { .. } | EbnfElement::RuleName { .. }
            | EbnfElement::NonTerminalBox { .. } | EbnfElement::StartCircle { .. }
            | EbnfElement::EndCircle { .. } => {}
        }
    }
    buf.push_str("</g></svg>");
    Ok(buf)
}

#[inline]
fn ff(v: f64) -> String { fmt_coord(v) }
