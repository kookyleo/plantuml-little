use std::fmt::Write;

use crate::klimt::svg::{fmt_coord, SvgGraphic};
use crate::layout::ditaa::{DitaaBox, DitaaLayout, DitaaLine, DitaaText};
use crate::model::ditaa::DitaaDiagram;
use crate::render::svg::{write_svg_root_bg, write_bg_rect};
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

    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "DITAA", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);
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

    let mut sg = SvgGraphic::new(0, 1.0);

    for ditaa_box in &layout.boxes {
        render_box(&mut sg, ditaa_box, diagram, border, font);
    }
    for line in &layout.lines {
        render_line(&mut sg, line, border);
    }
    for text in &layout.texts {
        render_text(&mut sg, text, font);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_box(
    sg: &mut SvgGraphic,
    ditaa_box: &DitaaBox,
    diagram: &DitaaDiagram,
    border: &str,
    font: &str,
) {
    let fill = ditaa_box.color.as_deref().unwrap_or(BOX_FILL);
    let radius = if ditaa_box.round { 8.0 } else { 0.0 };
    let shadow_offset = diagram.options.scale.unwrap_or(1.0) * SHADOW_OFFSET;

    // Shadow rect (uses opacity attribute — not supported by SvgGraphic, keep as push_raw)
    if !diagram.options.no_shadows {
        sg.push_raw(&format!(
            r#"<rect fill="{SHADOW_FILL}" height="{}" opacity="{SHADOW_OPACITY:.5}" rx="{}" ry="{}" stroke="none" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(ditaa_box.height), fmt_coord(radius), fmt_coord(radius),
            fmt_coord(ditaa_box.width),
            fmt_coord(ditaa_box.x + shadow_offset), fmt_coord(ditaa_box.y + shadow_offset),
        ));
        sg.push_raw("\n");
    }

    // Box rect
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_rectangle(ditaa_box.x, ditaa_box.y, ditaa_box.width, ditaa_box.height, radius, radius, 0.0);
    sg.push_raw("\n");

    if let Some(text) = &ditaa_box.text {
        let lines = count_creole_lines(text) as f64;
        let text_height = lines * LINE_HEIGHT;
        let start_y = ditaa_box.y + (ditaa_box.height - text_height).max(0.0) / 2.0 + FONT_SIZE;
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            text,
            ditaa_box.x + ditaa_box.width / 2.0,
            start_y,
            LINE_HEIGHT,
            font,
            Some("middle"),
            r#"font-size="12""#,
        );
        sg.push_raw(&tmp);
    }
}

fn render_line(sg: &mut SvgGraphic, line: &DitaaLine, border: &str) {
    if line.points.is_empty() {
        return;
    }

    // Polylines with markers — keep as push_raw (SvgGraphic doesn't support markers)
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
    sg.push_raw(&format!(
        r#"<polyline fill="none"{marker_start}{marker_end} points="{points}" style="stroke:{border};stroke-width:1.5;{dash}"/>"#
    ));
    sg.push_raw("\n");
}

fn render_text(sg: &mut SvgGraphic, text: &DitaaText, font: &str) {
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &text.text,
        text.x,
        text.y,
        LINE_HEIGHT,
        font,
        None,
        r#"font-size="12""#,
    );
    sg.push_raw(&tmp);
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
        assert!(!svg.contains(&format!(r#"opacity="{SHADOW_OPACITY:.5}""#)));
    }
}
