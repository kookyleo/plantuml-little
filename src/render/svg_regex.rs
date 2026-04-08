use std::fmt::Write;
use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape};
use crate::layout::regex_diagram::{RegexElement, RegexLayout};
use crate::model::regex_diagram::RegexDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
const STROKE: &str = "#181818";
const TEXT_C: &str = "#000000";

pub fn render_regex(_d: &RegexDiagram, l: &RegexLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(8192);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let (sw, sh) = (ensure_visible_int(l.width) as f64, ensure_visible_int(l.height) as f64);
    write_svg_root_bg(&mut buf, sw, sh, "REGEX", bg);
    // Java Regex does not draw a background rect — background is in SVG root style.
    buf.push_str("<defs/><g>");
    for e in &l.elements {
        match e {
            RegexElement::LiteralBox { x, y, width, height, text, dashed } => {
                let sty = if *dashed { format!("stroke:{};stroke-width:1;stroke-dasharray:5,5;", STROKE) } else { format!("stroke:{};stroke-width:0.5;", STROKE) };
                write!(buf, r#"<rect fill="none" height="{}" style="{}" width="{}" x="{}" y="{}"/>"#, ff(*height), sty, ff(*width), ff(*x), ff(*y)).unwrap();
                let lines: Vec<&str> = text.split('\n').collect();
                let asc = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
                let desc = font_metrics::descent("SansSerif", FONT_SIZE, false, false);
                for (i, ln) in lines.iter().enumerate() {
                    let tw = font_metrics::text_width(ln, "SansSerif", FONT_SIZE, false, false);
                    write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                        TEXT_C, FONT_SIZE as i32, ff(tw), ff(x + 5.0), ff(y + asc + 5.0 + i as f64 * (asc + desc)), xml_escape(ln)).unwrap();
                }
            }
            RegexElement::HLine { x1, y1, x2, y2, stroke_width } => {
                write!(buf, r#"<line style="stroke:{};stroke-width:{};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#, STROKE, ff(*stroke_width), ff(*x1), ff(*x2), ff(*y1), ff(*y2)).unwrap();
            }
            RegexElement::Path { d, fill, stroke_width } => {
                let f = if *fill { STROKE } else { "none" };
                write!(buf, r#"<path d="{}" fill="{}" style="stroke:{};stroke-width:{};"/>"#, d, f, STROKE, ff(*stroke_width)).unwrap();
            }
            RegexElement::Arrow { points, .. } => {
                write!(buf, r#"<path d="M{},{} L{},{} L{},{} L{},{}" fill="{}"/>"#,
                    ff(points[0].0), ff(points[0].1), ff(points[1].0), ff(points[1].1), ff(points[2].0), ff(points[2].1), ff(points[0].0), ff(points[0].1), STROKE).unwrap();
            }
            RegexElement::Text { x, y, text, font_size } => {
                let tw = font_metrics::text_width(text, "SansSerif", *font_size, false, false);
                write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    TEXT_C, *font_size as i32, ff(tw), ff(*x), ff(*y), xml_escape(text)).unwrap();
            }
        }
    }
    buf.push_str("</g></svg>"); Ok(buf)
}

#[inline]
fn ff(v: f64) -> String { fmt_coord(v) }
