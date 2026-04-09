use crate::font_metrics;
use crate::model::creole_diagram::{CreoleDiagram, CreoleElement};
use crate::Result;

/// Heading level 1 font size (Java Serif 18pt bold).
const HEADING1_FONT_SIZE: f64 = 18.0;
/// Normal text font size (Java Serif 14pt).
const TEXT_FONT_SIZE: f64 = 14.0;
/// Bullet circle radius.
const BULLET_RADIUS: f64 = 2.5;
/// Bullet circle left margin.
const BULLET_LEFT: f64 = 3.0;
/// Text left offset after bullet.
const BULLET_TEXT_LEFT: f64 = 12.0;

/// A positioned element in the creole layout.
#[derive(Debug, Clone)]
pub enum CreoleLayoutElement {
    Heading {
        text: String,
        x: f64,
        y: f64,
        text_width: f64,
        font_size: f64,
    },
    Bullet {
        cx: f64,
        cy: f64,
        text: String,
        text_x: f64,
        text_y: f64,
        text_width: f64,
    },
    Text {
        text: String,
        x: f64,
        y: f64,
        text_width: f64,
    },
}

/// Full creole layout.
#[derive(Debug)]
pub struct CreoleLayout {
    pub width: f64,
    pub height: f64,
    pub elements: Vec<CreoleLayoutElement>,
}

pub fn layout_creole(d: &CreoleDiagram) -> Result<CreoleLayout> {
    let mut elements = Vec::new();
    let mut y: f64 = 0.0;
    let mut max_x: f64 = 0.0;

    for elem in &d.elements {
        match elem {
            CreoleElement::Heading { text, level } => {
                let fs = match level {
                    1 => HEADING1_FONT_SIZE,
                    2 => 16.0,
                    3 => 14.0,
                    _ => 14.0,
                };
                let ascent = font_metrics::ascent("Serif", fs, true, false);
                let line_h = font_metrics::line_height("Serif", fs, true, false);
                let tw = font_metrics::text_width(text, "Serif", fs, true, false);

                y += ascent;
                elements.push(CreoleLayoutElement::Heading {
                    text: text.clone(),
                    x: 0.0,
                    y,
                    text_width: tw,
                    font_size: fs,
                });
                y += line_h - ascent;
                max_x = max_x.max(tw);
            }
            CreoleElement::Bullet { text, level: _ } => {
                let ascent = font_metrics::ascent("Serif", TEXT_FONT_SIZE, false, false);
                let line_h = font_metrics::line_height("Serif", TEXT_FONT_SIZE, false, false);
                let tw = font_metrics::text_width(text, "Serif", TEXT_FONT_SIZE, false, false);

                // Bullet circle center: (BULLET_LEFT + BULLET_RADIUS, y + ascent - BULLET_RADIUS)
                let text_y = y + ascent;
                let cy = y + ascent - BULLET_RADIUS;
                elements.push(CreoleLayoutElement::Bullet {
                    cx: BULLET_LEFT + BULLET_RADIUS,
                    cy,
                    text: text.clone(),
                    text_x: BULLET_TEXT_LEFT,
                    text_y,
                    text_width: tw,
                });
                y += line_h;
                max_x = max_x.max(BULLET_TEXT_LEFT + tw);
            }
            CreoleElement::Text(text) => {
                let ascent = font_metrics::ascent("Serif", TEXT_FONT_SIZE, false, false);
                let line_h = font_metrics::line_height("Serif", TEXT_FONT_SIZE, false, false);
                let tw = font_metrics::text_width(text, "Serif", TEXT_FONT_SIZE, false, false);

                y += ascent;
                elements.push(CreoleLayoutElement::Text {
                    text: text.clone(),
                    x: 0.0,
                    y,
                    text_width: tw,
                });
                y += line_h - ascent;
                max_x = max_x.max(tw);
            }
        }
    }

    // LimitFinder: text addPoint logic. maxX from text, maxY from text bottom.
    // For AbstractPSystem margins = 0, dimension = (maxX+1, maxY+1).
    let width = max_x + 1.0;
    let height = y + 1.0;

    Ok(CreoleLayout {
        width,
        height,
        elements,
    })
}
