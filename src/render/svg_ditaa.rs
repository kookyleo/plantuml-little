use std::fmt::Write;

use super::svg::{fmt_coord, write_svg_root};
use crate::layout::ditaa::{DitaaBox, DitaaLayout, DitaaLine, DitaaText};
use crate::model::ditaa::DitaaDiagram;
use crate::render::svg_richtext::{count_creole_lines, render_creole_text};
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 12.0;
const LINE_HEIGHT: f64 = 16.0;
const BACKGROUND: &str = "#FFFFFF";
const BOX_FILL: &str = "#F1F1F1";
const BOX_BORDER: &str = "#333333";
const TEXT_FILL: &str = "#000000";
const SHADOW_FILL: &str = "#000000";
const SHADOW_OPACITY: f64 = 0.15;
const SHADOW_OFFSET: f64 = 4.0;

pub fn render_ditaa(
    diagram: &DitaaDiagram,
    layout: &DitaaLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let border = skin.border_color("ditaa", BOX_BORDER);
    let font = skin.font_color("ditaa", TEXT_FILL);
    let background = skin.background_color("ditaabg", BACKGROUND);

    write_svg_root(&mut buf, layout.width, layout.height, "DITAA");
    buf.push_str("<defs/><g>");
    write!(
        buf,
        r#"<defs><marker id="ditaa-arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto-start-reverse"><path d="M0,0 L8,4 L0,8 Z " fill="{border}"/></marker></defs>"#
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<rect fill="{background}" height="{h:.0}" width="{w:.0}" x="0" y="0"/>"#,
        w = layout.width,
        h = layout.height,
    )
    .unwrap();
    buf.push('\n');

    for ditaa_box in &layout.boxes {
        render_box(&mut buf, ditaa_box, diagram, border, font);
    }
    for line in &layout.lines {
        render_line(&mut buf, line, border);
    }
    for text in &layout.texts {
        render_text(&mut buf, text, font);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_box(
    buf: &mut String,
    ditaa_box: &DitaaBox,
    diagram: &DitaaDiagram,
    border: &str,
    font: &str,
) {
    let fill = ditaa_box.color.as_deref().unwrap_or(BOX_FILL);
    let radius = if ditaa_box.round { 8.0 } else { 0.0 };
    let shadow_offset = diagram.options.scale.unwrap_or(1.0) * SHADOW_OFFSET;

    if !diagram.options.no_shadows {
        write!(
            buf,
            r#"<rect fill="{SHADOW_FILL}" height="{}" opacity="{SHADOW_OPACITY:.2}" rx="{}" ry="{}" stroke="none" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(ditaa_box.height), fmt_coord(radius), fmt_coord(radius),
            fmt_coord(ditaa_box.width),
            fmt_coord(ditaa_box.x + shadow_offset), fmt_coord(ditaa_box.y + shadow_offset),
        )
        .unwrap();
        buf.push('\n');
    }

    write!(
        buf,
        r#"<rect fill="{fill}" height="{}" rx="{}" ry="{}" style="stroke:{border};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(ditaa_box.height), fmt_coord(radius), fmt_coord(radius),
        fmt_coord(ditaa_box.width),
        fmt_coord(ditaa_box.x), fmt_coord(ditaa_box.y),
    )
    .unwrap();
    buf.push('\n');

    if let Some(text) = &ditaa_box.text {
        let lines = count_creole_lines(text) as f64;
        let text_height = lines * LINE_HEIGHT;
        let start_y = ditaa_box.y + (ditaa_box.height - text_height).max(0.0) / 2.0 + FONT_SIZE;
        render_creole_text(
            buf,
            text,
            ditaa_box.x + ditaa_box.width / 2.0,
            start_y,
            LINE_HEIGHT,
            font,
            Some("middle"),
            r#"font-size="12""#,
        );
    }
}

fn render_line(buf: &mut String, line: &DitaaLine, border: &str) {
    if line.points.is_empty() {
        return;
    }

    let mut points = String::new();
    for (idx, (x, y)) in line.points.iter().enumerate() {
        if idx > 0 {
            points.push(' ');
        }
        write!(points, "{},{}", fmt_coord(*x), fmt_coord(*y)).unwrap();
    }

    let dash = if line.dashed {
        "stroke-dasharray:6,4;"
    } else {
        ""
    };
    let marker_start = if line.arrow_start {
        r#" marker-start="url(#ditaa-arrow)""#
    } else {
        ""
    };
    let marker_end = if line.arrow_end {
        r#" marker-end="url(#ditaa-arrow)""#
    } else {
        ""
    };
    write!(
        buf,
        r#"<polyline fill="none"{marker_start}{marker_end} points="{points}" style="stroke:{border};stroke-width:1.5;{dash}"/>"#
    )
    .unwrap();
    buf.push('\n');
}

fn render_text(buf: &mut String, text: &DitaaText, font: &str) {
    render_creole_text(
        buf,
        &text.text,
        text.x,
        text.y,
        LINE_HEIGHT,
        font,
        None,
        r#"font-size="12""#,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::ditaa::{DitaaBox, DitaaLayout, DitaaLine, DitaaText};
    use crate::model::ditaa::{DitaaDiagram, DitaaOptions};

    fn sample_layout() -> (DitaaDiagram, DitaaLayout) {
        let diagram = DitaaDiagram {
            source: "+--+  +--+\n|A |->|B |\n+--+  +--+\nlegend".to_string(),
            options: DitaaOptions {
                round_corners: true,
                ..DitaaOptions::default()
            },
        };
        let layout = DitaaLayout {
            boxes: vec![
                DitaaBox {
                    x: 0.0,
                    y: 0.0,
                    width: 40.0,
                    height: 28.0,
                    round: true,
                    color: Some("#66CC66".to_string()),
                    text: Some("A".to_string()),
                },
                DitaaBox {
                    x: 56.0,
                    y: 0.0,
                    width: 40.0,
                    height: 28.0,
                    round: true,
                    color: None,
                    text: Some("B".to_string()),
                },
            ],
            lines: vec![DitaaLine {
                points: vec![(40.0, 14.0), (56.0, 14.0)],
                dashed: false,
                arrow_start: false,
                arrow_end: true,
            }],
            texts: vec![DitaaText {
                x: 0.0,
                y: 54.0,
                text: "legend".to_string(),
            }],
            width: 120.0,
            height: 72.0,
        };
        (diagram, layout)
    }

    #[test]
    fn render_contains_boxes_and_arrow_marker() {
        let (diagram, layout) = sample_layout();
        let svg = render_ditaa(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("marker-end=\"url(#ditaa-arrow)\""));
        assert!(svg.contains("#66CC66"));
        assert!(svg.contains(">legend<"));
    }

    #[test]
    fn render_skips_shadow_when_disabled() {
        let (mut diagram, layout) = sample_layout();
        diagram.options.no_shadows = true;
        let svg = render_ditaa(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(!svg.contains(&format!(r#"opacity="{SHADOW_OPACITY:.2}""#)));
    }
}
