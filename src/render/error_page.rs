use std::fmt::Write;
use std::path::Path;

use crate::klimt::svg::{fmt_coord, java_source_seed, xml_escape};
use crate::render::svg::{inject_plantuml_source, PLANTUML_VERSION};
use crate::Result;

const VERSION_TEXT_LEN: f64 = 128.0098;
const BLANK_TEXT_LEN: f64 = 4.874;
const RED_TEXT_X: f64 = 9.874;
const RIGHT_MARGIN: f64 = 6.0;
const BOTTOM_MARGIN: f64 = 8.0;
const VERSION_Y: f64 = 17.0;
const LABEL_RECT_Y: f64 = 26.9688;
const LABEL_RECT_H: f64 = 21.2969;
const LABEL_TEXT_Y: f64 = 41.9688;
const BLANK_Y: f64 = 62.2656;
const FIRST_LINE_Y: f64 = 78.5625;
const LINE_H: f64 = 16.2969;

const RELEASE_UNSUPPORTED_TITLE: &str = "Diagram not supported by this release of PlantUML";
const RELEASE_UNSUPPORTED_VERSION: &str =
    "PlantUML version 1.2026.2 / bb8550d [2026-04-09 14:37:49 UTC]";
const RELEASE_UNSUPPORTED_LICENSE: &str = "License GPL";

fn pseudo_error_pixel(source: &str) -> String {
    let seed = java_source_seed(source).unsigned_abs() as u32;
    let r = ((seed >> 16) & 0x1f) as u8;
    let g = ((seed >> 8) & 0x1f) as u8;
    let b = (seed & 0x1f) as u8;
    format!("{:02X}{:02X}{:02X}", r, g, b)
}

pub(crate) fn render_compact_error_svg(
    source: &str,
    input_path: Option<&Path>,
    line: usize,
    message: &str,
) -> Result<String> {
    let all_lines: Vec<&str> = source.lines().collect();
    let highlight_idx = line.saturating_sub(1).min(all_lines.len().saturating_sub(1));
    let start_idx = all_lines[..=highlight_idx]
        .iter()
        .rposition(|line| line.trim().starts_with("@start"))
        .unwrap_or(0);
    let shown_lines: Vec<&str> = all_lines[start_idx..=highlight_idx]
        .iter()
        .map(|line| line.trim())
        .collect();

    let input_name = input_path
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("block");
    let label = format!("[From {input_name} (line {line}) ]");
    let label_text_w = crate::font_metrics::text_width(&label, "sans-serif", 14.0, true, false);
    let label_rect_w = label_text_w + 2.0;

    let mut max_right = 5.0 + VERSION_TEXT_LEN;
    max_right = max_right.max(5.0 + label_rect_w);
    max_right = max_right.max(5.0 + BLANK_TEXT_LEN);

    for shown in &shown_lines {
        let text_w = crate::font_metrics::text_width(shown, "sans-serif", 14.0, true, false);
        max_right = max_right.max(5.0 + text_w);
    }

    let message_w = crate::font_metrics::text_width(message, "sans-serif", 14.0, true, false);
    max_right = max_right.max(RED_TEXT_X + message_w);

    let width = (max_right + RIGHT_MARGIN).ceil() as i32;

    let red_y = FIRST_LINE_Y + (shown_lines.len() as f64) * LINE_H;
    let height = (red_y + BOTTOM_MARGIN).ceil() as i32;

    let rp = pseudo_error_pixel(source);
    let mut svg = String::with_capacity(4096);
    write!(
        svg,
        concat!(
            "<?plantuml {ver}?>",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" ",
            "contentStyleType=\"text/css\" height=\"{height}px\" preserveAspectRatio=\"none\" ",
            "style=\"width:{width}px;height:{height}px;background:#000000;\" version=\"1.1\" ",
            "viewBox=\"0 0 {width} {height}\" width=\"{width}px\" zoomAndPan=\"magnify\"><defs/><g>"
        ),
        ver = PLANTUML_VERSION,
        width = width,
        height = height
    )
    .unwrap();

    write!(
        svg,
        "<rect fill=\"#{rp}\" height=\"1\" style=\"stroke:#{rp};stroke-width:1;\" width=\"1\" x=\"0\" y=\"0\"/>",
    )
    .unwrap();
    write!(
        svg,
        "<text fill=\"#33FF02\" font-family=\"sans-serif\" font-size=\"12\" font-style=\"italic\" font-weight=\"bold\" lengthAdjust=\"spacing\" textLength=\"{VERSION_TEXT_LEN}\" x=\"5\" y=\"{y}\">PlantUML {PLANTUML_VERSION}</text>",
        y = fmt_coord(VERSION_Y),
    )
    .unwrap();
    write!(
        svg,
        "<rect fill=\"#33FF02\" height=\"{h}\" style=\"stroke:#33FF02;stroke-width:1;\" width=\"{w}\" x=\"5\" y=\"{y}\"/>",
        h = fmt_coord(LABEL_RECT_H),
        w = fmt_coord(label_rect_w),
        y = fmt_coord(LABEL_RECT_Y),
    )
    .unwrap();
    write!(
        svg,
        "<text fill=\"#000000\" font-family=\"sans-serif\" font-size=\"14\" font-weight=\"bold\" lengthAdjust=\"spacing\" textLength=\"{w}\" x=\"6\" y=\"{y}\">{label}</text>",
        w = fmt_coord(label_text_w),
        y = fmt_coord(LABEL_TEXT_Y),
        label = xml_escape(&label),
    )
    .unwrap();
    write!(
        svg,
        "<text fill=\"#33FF02\" font-family=\"sans-serif\" font-size=\"14\" font-weight=\"bold\" lengthAdjust=\"spacing\" textLength=\"{BLANK_TEXT_LEN}\" x=\"5\" y=\"{y}\">&#160;</text>",
        y = fmt_coord(BLANK_Y),
    )
    .unwrap();

    let mut y = FIRST_LINE_Y;
    for (idx, shown) in shown_lines.iter().enumerate() {
        let escaped = xml_escape(shown);
        let text_w = crate::font_metrics::text_width(shown, "sans-serif", 14.0, true, false);
        if idx + 1 == shown_lines.len() {
            write!(
                svg,
                "<text fill=\"#33FF02\" font-family=\"sans-serif\" font-size=\"14\" font-weight=\"bold\" lengthAdjust=\"spacing\" text-decoration=\"wavy underline\" textLength=\"{w}\" x=\"5\" y=\"{y}\">{text}</text>",
                w = fmt_coord(text_w),
                y = fmt_coord(y),
                text = escaped,
            )
            .unwrap();
        } else {
            write!(
                svg,
                "<text fill=\"#33FF02\" font-family=\"sans-serif\" font-size=\"14\" font-weight=\"bold\" lengthAdjust=\"spacing\" textLength=\"{w}\" x=\"5\" y=\"{y}\">{text}</text>",
                w = fmt_coord(text_w),
                y = fmt_coord(y),
                text = escaped,
            )
            .unwrap();
        }
        y += LINE_H;
    }

    write!(
        svg,
        "<text fill=\"#FF0000\" font-family=\"sans-serif\" font-size=\"14\" font-weight=\"bold\" lengthAdjust=\"spacing\" textLength=\"{w}\" x=\"{x}\" y=\"{y}\">{text}</text>",
        w = fmt_coord(message_w),
        x = fmt_coord(RED_TEXT_X),
        y = fmt_coord(red_y),
        text = xml_escape(message),
    )
    .unwrap();
    svg.push_str("</g></svg>");
    inject_plantuml_source(svg, source)
}

pub(crate) fn render_unsupported_release_svg(source: &str) -> Result<String> {
    let title_w =
        crate::font_metrics::text_width(RELEASE_UNSUPPORTED_TITLE, "sans-serif", 12.0, true, false);
    let version_w = crate::font_metrics::text_width(
        RELEASE_UNSUPPORTED_VERSION,
        "sans-serif",
        12.0,
        false,
        false,
    );
    let license_w = crate::font_metrics::text_width(
        RELEASE_UNSUPPORTED_LICENSE,
        "sans-serif",
        12.0,
        false,
        false,
    );
    let mut svg = String::with_capacity(1024);
    crate::render::svg::write_svg_root_bg_opt(&mut svg, 402.0, 47.0, None, "#FFFFFF");
    svg.push_str("<defs/><g>");
    let mut sg = crate::klimt::svg::SvgGraphic::new(0, 1.0);
    sg.set_fill_color("#000000");
    sg.svg_text(
        RELEASE_UNSUPPORTED_TITLE,
        5.0,
        16.1387,
        Some("sans-serif"),
        12.0,
        Some("bold"),
        None,
        None,
        title_w,
        crate::klimt::svg::LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    sg.svg_text(
        RELEASE_UNSUPPORTED_VERSION,
        5.0,
        30.1074,
        Some("sans-serif"),
        12.0,
        None,
        None,
        None,
        version_w,
        crate::klimt::svg::LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    sg.svg_text(
        RELEASE_UNSUPPORTED_LICENSE,
        5.0,
        44.0762,
        Some("sans-serif"),
        12.0,
        None,
        None,
        None,
        license_w,
        crate::klimt::svg::LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    svg.push_str(sg.body());
    svg.push_str("</g></svg>");
    inject_plantuml_source(svg, source)
}
