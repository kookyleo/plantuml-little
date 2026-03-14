use std::collections::HashMap;
use std::fmt::Write;
use std::io::Write as IoWrite;

use flate2::write::DeflateEncoder;
use flate2::Compression;

use crate::layout::graphviz::{ClassNoteLayout, EdgeLayout, GraphLayout, NodeLayout};
use crate::layout::DiagramLayout;
use crate::model::{
    ArrowHead, ClassDiagram, ClassHideShowRule, ClassPortion, ClassRuleTarget, Diagram,
    DiagramMeta, Entity, EntityKind, LineStyle, Link, Member, Visibility,
};
use crate::style::SkinParams;
use crate::Result;

use crate::font_metrics;

use super::svg_richtext::{
    count_creole_lines, max_creole_plain_line_len, render_creole_text, set_default_font_family,
};
use super::svg_sequence;

// ── Style constants ──────────────────────────────────────────────────

const FONT_SIZE: f64 = 14.0;
const LINE_HEIGHT: f64 = 8.0;
const PADDING: f64 = 3.0;
const HEADER_HEIGHT: f64 = 32.0;
/// Java PlantUML: moveDelta(6 - minX, 6 - minY) — SvekResult.java:133
/// Entity rects start at x=7 (MARGIN + 1px border inset in draw_entity_box).
const MARGIN: f64 = 6.0;
const EDGE_OFFSET: f64 = MARGIN + 1.0;
/// Java PlantUML SvekResult: delta(15, 15) added to content bounding box.
const CANVAS_DELTA: f64 = 15.0;
/// Java PlantUML: default document margin right=5, bottom=5 (from plantuml.skin style).
const DOC_MARGIN_RIGHT: f64 = 5.0;
const DOC_MARGIN_BOTTOM: f64 = 5.0;
const CIRCLE_LEFT_PAD: f64 = 4.0;
const CIRCLE_DIAMETER: f64 = 22.0;
const EMPTY_COMPARTMENT: f64 = 8.0;
const HEADER_CIRCLE_BLOCK_WIDTH: f64 = 26.0;
const HEADER_CIRCLE_BLOCK_HEIGHT: f64 = 32.0;
const HEADER_NAME_BLOCK_HEIGHT: f64 = 16.2969;
const HEADER_NAME_BASELINE: f64 = 12.9951;
const HEADER_NAME_BLOCK_MARGIN_X: f64 = 6.0;
const HEADER_STEREO_FONT_SIZE: f64 = 12.0;
const HEADER_STEREO_LINE_HEIGHT: f64 = 13.9688;
const HEADER_STEREO_BASELINE: f64 = 11.1387;
const HEADER_STEREO_NAME_GAP: f64 = 10.0;
const MEMBER_ROW_HEIGHT: f64 = 16.2969;
const MEMBER_BLOCK_HEIGHT_ONE_ROW: f64 = 24.2969;
const MEMBER_ICON_X_OFFSET: f64 = 6.0;
const MEMBER_ICON_Y_OFFSET: f64 = 8.6484;
const MEMBER_TEXT_X_WITH_ICON: f64 = 20.0;
const MEMBER_TEXT_X_NO_ICON: f64 = 6.0;
const MEMBER_TEXT_Y_OFFSET: f64 = 16.9951;

const CLASS_BG: &str = "#F1F1F1";
const CLASS_BORDER: &str = "#181818";
const IFACE_BG: &str = "#F1F1F1";
const IFACE_BORDER: &str = "#181818";
const ENUM_BG: &str = "#F1F1F1";
const ENUM_BORDER: &str = "#181818";
const ABSTRACT_BG: &str = "#F1F1F1";
const ABSTRACT_BORDER: &str = "#181818";

const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const NOTE_FOLD: f64 = 8.0;
const NOTE_TEXT_PADDING: f64 = 6.0;

const LINK_COLOR: &str = "#181818";
const LABEL_COLOR: &str = "#000000";
const PLANTUML_VERSION: &str = "1.2026.3beta4";

// ── Meta rendering constants ────────────────────────────────────────

const META_TITLE_FONT_SIZE: f64 = 18.0;
const META_LINE_HEIGHT: f64 = 18.0;
const META_GAP: f64 = 8.0;
const LEGEND_PADDING: f64 = 8.0;
const LEGEND_BORDER_COLOR: &str = "#000000";
const LEGEND_BG: &str = "#FEFFDD";

// ── Helpers ─────────────────────────────────────────────────────────

/// Format a coordinate value matching Java PlantUML's `SvgGraphics.format()`:
/// - Up to 4 decimal places
/// - Trailing zeros stripped
/// - Integer values without decimal point
/// - "0" for zero
///
/// Reference: SvgGraphics.java:944
pub(crate) fn fmt_coord(value: f64) -> String {
    // Java's SvgGraphics.format(): "%.4f" with half-up rounding, trailing zero stripping.
    // Handles negative zero: -0.00004 → "0" not "-0".
    if value == 0.0 {
        return "0".into();
    }
    let rounded = java_round_4(value);
    // Guard against negative zero after rounding
    if rounded == 0.0 {
        return "0".into();
    }
    let s = format!("{:.4}", rounded);
    let bytes = s.as_bytes();
    let dot = s.find('.').unwrap();
    let mut end = s.len();
    while end > dot + 1 && bytes[end - 1] == b'0' {
        end -= 1;
    }
    if end == dot + 1 {
        end = dot;
    }
    s[..end].to_string()
}

/// Round a f64 to 4 decimal places using Java's half-up rounding.
/// Java: Math.round(x * 10000) / 10000.0 (effectively)
fn java_round_4(v: f64) -> f64 {
    let factor = 10000.0_f64;
    let scaled = v * factor;
    // Java half-up: if fractional part is exactly 0.5, round away from zero
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5).floor()
    } else {
        (scaled - 0.5).ceil()
    };
    rounded / factor
}

/// Write a Java PlantUML-compatible SVG root element and open a `<g>` wrapper.
pub(crate) fn write_svg_root(buf: &mut String, w: f64, h: f64, diagram_type: &str) {
    let wi = if w.is_finite() && w > 0.0 { w.ceil() as i32 } else { 100 };
    let hi = if h.is_finite() && h > 0.0 { h.ceil() as i32 } else { 100 };
    write!(
        buf,
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg""#,
            r#" xmlns:xlink="http://www.w3.org/1999/xlink""#,
            r#" contentStyleType="text/css""#,
            r#" data-diagram-type="{dtype}""#,
            r#" height="{hi}px""#,
            r#" preserveAspectRatio="none""#,
            r#" style="width:{wi}px;height:{hi}px;background:#FFFFFF;""#,
            r#" version="1.1""#,
            r#" viewBox="0 0 {wi} {hi}""#,
            r#" width="{wi}px""#,
            r#" zoomAndPan="magnify">"#,
        ),
        dtype = diagram_type,
        hi = hi,
        wi = wi,
    )
    .unwrap();
    write!(buf, "<?plantuml {PLANTUML_VERSION}?>").unwrap();
}

fn sanitize_id(name: &str) -> String {
    name.replace('<', "_LT_")
        .replace('>', "_GT_")
        .replace(',', "_COMMA_")
        .replace(' ', "_")
}

pub(crate) fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

// ── Public entry point ───────────────────────────────────────────────

/// Return the `data-diagram-type` string for a `Diagram` variant.

/// Render a Diagram + DiagramLayout into an SVG string.
pub fn render(
    diagram: &Diagram,
    layout: &DiagramLayout,
    skin: &SkinParams,
    meta: &DiagramMeta,
) -> Result<String> {
    render_with_source(diagram, layout, skin, meta, None)
}

pub fn render_with_source(
    diagram: &Diagram,
    layout: &DiagramLayout,
    skin: &SkinParams,
    meta: &DiagramMeta,
    source: Option<&str>,
) -> Result<String> {
    // Apply handwritten font override if enabled
    set_default_font_family(skin.handwritten_font_family().map(|s| s.to_string()));
    let body_svg = render_body(diagram, layout, skin)?;
    set_default_font_family(None);
    let mut svg = if meta.is_empty() {
        body_svg
    } else {
        // Extract diagram type from body SVG's data-diagram-type attribute
        let dtype = body_svg
            .find("data-diagram-type=\"")
            .and_then(|pos| {
                let start = pos + 19;
                body_svg[start..]
                    .find('"')
                    .map(|end| &body_svg[start..start + end])
            })
            .unwrap_or("CLASS");
        wrap_with_meta(&body_svg, meta, dtype)?
    };

    if let Some(source) = source {
        svg = inject_plantuml_source(svg, source)?;
    }

    Ok(svg)
}

fn render_body(diagram: &Diagram, layout: &DiagramLayout, skin: &SkinParams) -> Result<String> {
    match (diagram, layout) {
        (Diagram::Class(cd), DiagramLayout::Class(gl)) => render_class(cd, gl, skin),
        (Diagram::Sequence(sd), DiagramLayout::Sequence(sl)) => {
            svg_sequence::render_sequence(sd, sl, skin)
        }
        (Diagram::Activity(ad), DiagramLayout::Activity(al)) => {
            super::svg_activity::render_activity(ad, al, skin)
        }
        (Diagram::State(sd), DiagramLayout::State(sl)) => {
            super::svg_state::render_state(sd, sl, skin)
        }
        (Diagram::Component(cd), DiagramLayout::Component(cl)) => {
            super::svg_component::render_component(cd, cl, skin)
        }
        (Diagram::Ditaa(dd), DiagramLayout::Ditaa(dl)) => {
            super::svg_ditaa::render_ditaa(dd, dl, skin)
        }
        (Diagram::Erd(ed), DiagramLayout::Erd(el)) => super::svg_erd::render_erd(ed, el, skin),
        (Diagram::Gantt(gd), DiagramLayout::Gantt(gl)) => {
            super::svg_gantt::render_gantt(gd, gl, skin)
        }
        (Diagram::Json(jd), DiagramLayout::Json(jl)) => super::svg_json::render_json(jd, jl, skin),
        (Diagram::Mindmap(md), DiagramLayout::Mindmap(ml)) => {
            super::svg_mindmap::render_mindmap(md, ml, skin)
        }
        (Diagram::Nwdiag(nd), DiagramLayout::Nwdiag(nl)) => {
            super::svg_nwdiag::render_nwdiag(nd, nl, skin)
        }
        (Diagram::Salt(sd), DiagramLayout::Salt(sl)) => super::svg_salt::render_salt(sd, sl, skin),
        (Diagram::Timing(td), DiagramLayout::Timing(tl)) => {
            super::svg_timing::render_timing(td, tl, skin)
        }
        (Diagram::Wbs(wd), DiagramLayout::Wbs(wl)) => super::svg_wbs::render_wbs(wd, wl, skin),
        (Diagram::Yaml(yd), DiagramLayout::Yaml(yl)) => super::svg_json::render_yaml(yd, yl, skin),
        (Diagram::UseCase(ud), DiagramLayout::UseCase(ul)) => {
            super::svg_usecase::render_usecase(ud, ul, skin)
        }
        (Diagram::Dot(dd), DiagramLayout::Dot(_gl)) => {
            // DOT passthrough: render using vizoxide directly
            render_dot_passthrough(&dd.source)
        }
        _ => Err(crate::Error::Render("diagram/layout type mismatch".into())),
    }
}

/// Render a DOT passthrough diagram using the Graphviz `dot` command.
///
/// Pipes the raw DOT source through `dot -Tsvg` and returns the resulting SVG.
fn render_dot_passthrough(dot_source: &str) -> Result<String> {
    use std::process::{Command, Stdio};

    log::debug!(
        "render_dot_passthrough: {} bytes of DOT source",
        dot_source.len()
    );

    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            crate::Error::Render(format!("failed to spawn dot: {e} (is graphviz installed?)"))
        })?;

    child
        .stdin
        .take()
        .unwrap()
        .write_all(dot_source.as_bytes())
        .map_err(|e| crate::Error::Render(format!("failed to write to dot stdin: {e}")))?;

    let output = child
        .wait_with_output()
        .map_err(|e| crate::Error::Render(format!("dot process error: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::Error::Render(format!(
            "dot exited with error: {stderr}"
        )));
    }

    let svg = String::from_utf8(output.stdout)
        .map_err(|e| crate::Error::Render(format!("dot output is not valid UTF-8: {e}")))?;

    log::debug!(
        "render_dot_passthrough: produced {} bytes of SVG",
        svg.len()
    );
    Ok(svg)
}

// ── Meta wrapping ───────────────────────────────────────────────────

fn meta_top_height(meta: &DiagramMeta) -> f64 {
    let mut h = 0.0;
    if let Some(ref hdr) = meta.header {
        h += count_creole_lines(hdr) as f64 * META_LINE_HEIGHT + META_GAP;
    }
    if let Some(ref title) = meta.title {
        h += count_creole_lines(title) as f64 * META_TITLE_FONT_SIZE + META_GAP;
    }
    h
}

fn meta_bottom_height(meta: &DiagramMeta) -> f64 {
    let mut h = 0.0;
    if let Some(ref caption) = meta.caption {
        h += count_creole_lines(caption) as f64 * META_LINE_HEIGHT + META_GAP;
    }
    if let Some(ref ftr) = meta.footer {
        h += count_creole_lines(ftr) as f64 * META_LINE_HEIGHT + META_GAP;
    }
    if let Some(ref leg) = meta.legend {
        let lc = count_creole_lines(leg) as f64;
        h += lc * META_LINE_HEIGHT + LEGEND_PADDING * 2.0 + META_GAP;
    }
    h
}

fn estimate_creole_width(text: &str, font_size: f64) -> f64 {
    max_creole_plain_line_len(text) as f64
        * font_metrics::char_width('a', "SansSerif", font_size, false, false)
}

fn meta_required_width(meta: &DiagramMeta) -> f64 {
    let mut width = 2.0 * MARGIN;

    if let Some(ref hdr) = meta.header {
        width = width.max(estimate_creole_width(hdr, FONT_SIZE) + 2.0 * MARGIN);
    }
    if let Some(ref title) = meta.title {
        width = width.max(estimate_creole_width(title, META_TITLE_FONT_SIZE) + 2.0 * MARGIN);
    }
    if let Some(ref caption) = meta.caption {
        width = width.max(estimate_creole_width(caption, FONT_SIZE) + 2.0 * MARGIN);
    }
    if let Some(ref ftr) = meta.footer {
        width = width.max(estimate_creole_width(ftr, FONT_SIZE) + 2.0 * MARGIN);
    }
    if let Some(ref leg) = meta.legend {
        let legend_w = max_creole_plain_line_len(leg).max(6) as f64
            * font_metrics::char_width('a', "SansSerif", FONT_SIZE, false, false)
            + LEGEND_PADDING * 2.0
            + 2.0 * MARGIN;
        width = width.max(legend_w);
    }

    width
}

fn extract_dimensions(svg: &str) -> (f64, f64) {
    if let Some(vb_start) = svg.find("viewBox=\"") {
        let after = &svg[vb_start + 9..];
        if let Some(vb_end) = after.find('"') {
            let parts: Vec<&str> = after[..vb_end].split_whitespace().collect();
            if parts.len() == 4 {
                let w = parts[2].parse::<f64>().unwrap_or(400.0);
                let h = parts[3].parse::<f64>().unwrap_or(300.0);
                return (w, h);
            }
        }
    }
    let w = extract_attr(svg, "width").unwrap_or(400.0);
    let h = extract_attr(svg, "height").unwrap_or(300.0);
    (w, h)
}

fn extract_attr(svg: &str, attr: &str) -> Option<f64> {
    let needle = format!("{attr}=\"");
    if let Some(pos) = svg.find(&needle) {
        let after = &svg[pos + needle.len()..];
        if let Some(end) = after.find('"') {
            return after[..end].parse::<f64>().ok();
        }
    }
    None
}

/// Bounding box of rendered SVG content, simulating Java's LimitFinder.
struct SvgBounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

/// Compute the bounding box of all rendered elements in an SVG body string,
/// matching Java's LimitFinder behavior:
/// - rect: (x-1, y-1) to (x+w-1, y+h-1)
/// - ellipse: (cx-rx, cy-ry) to (cx+rx, cy+ry)
/// - line: (x1,y1) to (x2,y2)
/// - text: (x, y-ascent) to (x+textLength, y)
/// - polygon: (min_point_x - HACK_X(10), min_point_y) to (max_point_x + HACK_X(10), max_point_y)
fn compute_svg_bounds(svg_body: &str) -> SvgBounds {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let mut add_point = |x: f64, y: f64| {
        if x < min_x { min_x = x; }
        if y < min_y { min_y = y; }
        if x > max_x { max_x = x; }
        if y > max_y { max_y = y; }
    };

    // Helper to parse a numeric attribute value from an element string
    let parse_attr = |elem: &str, attr: &str| -> Option<f64> {
        let needle = format!("{attr}=\"");
        let pos = elem.find(&needle)? + needle.len();
        let end = elem[pos..].find('"')? + pos;
        elem[pos..end].parse::<f64>().ok()
    };

    // Process each SVG element
    let mut pos = 0;
    while pos < svg_body.len() {
        if let Some(start) = svg_body[pos..].find('<') {
            let abs_start = pos + start;
            if let Some(end) = svg_body[abs_start..].find('>') {
                let elem = &svg_body[abs_start..abs_start + end + 1];
                pos = abs_start + end + 1;

                if elem.starts_with("<rect ") {
                    // LimitFinder: (x-1, y-1) to (x+w-1, y+h-1)
                    if let (Some(x), Some(y), Some(w), Some(h)) = (
                        parse_attr(elem, "x"),
                        parse_attr(elem, "y"),
                        parse_attr(elem, "width"),
                        parse_attr(elem, "height"),
                    ) {
                        add_point(x - 1.0, y - 1.0);
                        add_point(x + w - 1.0, y + h - 1.0);
                        // UEmpty correction: entity body adds +1 to max
                        add_point(x + w, y + h);
                    }
                } else if elem.starts_with("<ellipse ") {
                    if let (Some(cx), Some(cy), Some(rx), Some(ry)) = (
                        parse_attr(elem, "cx"),
                        parse_attr(elem, "cy"),
                        parse_attr(elem, "rx"),
                        parse_attr(elem, "ry"),
                    ) {
                        add_point(cx - rx, cy - ry);
                        add_point(cx + rx - 1.0, cy + ry - 1.0);
                    }
                } else if elem.starts_with("<line ") {
                    if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                        parse_attr(elem, "x1"),
                        parse_attr(elem, "y1"),
                        parse_attr(elem, "x2"),
                        parse_attr(elem, "y2"),
                    ) {
                        add_point(x1, y1);
                        add_point(x2, y2);
                    }
                } else if elem.starts_with("<text ") {
                    if let (Some(x), Some(y)) = (parse_attr(elem, "x"), parse_attr(elem, "y")) {
                        let tl = parse_attr(elem, "textLength").unwrap_or(0.0);
                        add_point(x, y);
                        add_point(x + tl, y);
                    }
                } else if elem.starts_with("<polygon ") {
                    // HACK_X_FOR_POLYGON = 10 in Java LimitFinder
                    if let Some(pts_start) = elem.find("points=\"") {
                        let after = &elem[pts_start + 8..];
                        if let Some(pts_end) = after.find('"') {
                            let pts_str = &after[..pts_end];
                            let mut poly_min_x = f64::INFINITY;
                            let mut poly_min_y = f64::INFINITY;
                            let mut poly_max_x = f64::NEG_INFINITY;
                            let mut poly_max_y = f64::NEG_INFINITY;
                            for pair in pts_str.split(' ') {
                                let coords: Vec<&str> = pair.split(',').collect();
                                if coords.len() == 2 {
                                    if let (Ok(px), Ok(py)) =
                                        (coords[0].parse::<f64>(), coords[1].parse::<f64>())
                                    {
                                        if px < poly_min_x { poly_min_x = px; }
                                        if py < poly_min_y { poly_min_y = py; }
                                        if px > poly_max_x { poly_max_x = px; }
                                        if py > poly_max_y { poly_max_y = py; }
                                    }
                                }
                            }
                            if poly_min_x.is_finite() {
                                add_point(poly_min_x - 10.0, poly_min_y); // HACK_X_FOR_POLYGON
                                add_point(poly_max_x + 10.0, poly_max_y);
                            }
                        }
                    }
                } else if elem.starts_with("<path ") {
                    // Parse path d attribute for min/max coordinates
                    if let Some(d_start) = elem.find("d=\"") {
                        let after = &elem[d_start + 3..];
                        if let Some(d_end) = after.find('"') {
                            let d = &after[..d_end];
                            for token in d.split(|c: char| c == 'M' || c == 'C' || c == 'L' || c == 'Q' || c == 'Z' || c == ' ') {
                                let coords: Vec<&str> = token.split(',').collect();
                                if coords.len() == 2 {
                                    if let (Ok(px), Ok(py)) =
                                        (coords[0].parse::<f64>(), coords[1].parse::<f64>())
                                    {
                                        add_point(px, py);
                                    }
                                }
                            }
                        }
                    }
                }
                continue;
            }
        }
        break;
    }

    SvgBounds {
        min_x: if min_x.is_finite() { min_x } else { 0.0 },
        min_y: if min_y.is_finite() { min_y } else { 0.0 },
        max_x: if max_x.is_finite() { max_x } else { 0.0 },
        max_y: if max_y.is_finite() { max_y } else { 0.0 },
    }
}

fn extract_svg_content(svg: &str) -> String {
    if let Some(tag_end) = svg.find('>') {
        let mut after_open = &svg[tag_end + 1..];
        if after_open.starts_with("<?plantuml ") {
            if let Some(end) = after_open.find("?>") {
                after_open = &after_open[end + 2..];
            }
        }
        if let Some(close_pos) = after_open.rfind("</svg>") {
            return after_open[..close_pos].to_string();
        }
        return after_open.to_string();
    }
    svg.to_string()
}

fn inject_plantuml_source(mut svg: String, source: &str) -> Result<String> {
    let encoded = encode_plantuml_source(source)?;
    let pi = format!("<?plantuml-src {encoded}?>");
    if let Some(pos) = svg.rfind("</g></svg>") {
        svg.insert_str(pos, &pi);
        return Ok(svg);
    }
    if let Some(pos) = svg.rfind("</svg>") {
        svg.insert_str(pos, &pi);
        return Ok(svg);
    }
    Err(crate::Error::Render(
        "rendered SVG missing closing tag for plantuml-src injection".into(),
    ))
}

fn encode_plantuml_source(source: &str) -> Result<String> {
    let compressed_source = compress_plantuml_source_for_pi(source);
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(compressed_source.as_bytes())
        .map_err(|e| crate::Error::Render(format!("failed to deflate PlantUML source: {e}")))?;
    let compressed = encoder
        .finish()
        .map_err(|e| crate::Error::Render(format!("failed to finish PlantUML deflate: {e}")))?;
    Ok(encode_plantuml_ascii(&compressed))
}

fn compress_plantuml_source_for_pi(source: &str) -> String {
    let mut body = Vec::new();
    let mut in_diagram = false;

    for line in source.lines() {
        if !in_diagram {
            if line.starts_with("@startuml") {
                in_diagram = true;
            }
            continue;
        }
        if line.starts_with("@enduml") {
            break;
        }
        body.push(line);
    }

    let body = if in_diagram {
        body.join("\n")
    } else {
        source.to_string()
    };
    trim_plantuml_source(&body)
}

fn trim_plantuml_source(source: &str) -> String {
    source
        .trim_matches(|c| matches!(c, ' ' | '\t' | '\r' | '\n' | '\0'))
        .to_string()
}

fn encode_plantuml_ascii(data: &[u8]) -> String {
    let mut result = String::with_capacity((data.len() * 4 + 2) / 3);
    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = *chunk.get(1).unwrap_or(&0);
        let b3 = *chunk.get(2).unwrap_or(&0);
        append_plantuml_3bytes(&mut result, b1, b2, b3);
    }
    result
}

fn append_plantuml_3bytes(buf: &mut String, b1: u8, b2: u8, b3: u8) {
    let c1 = b1 >> 2;
    let c2 = ((b1 & 0x03) << 4) | (b2 >> 4);
    let c3 = ((b2 & 0x0F) << 2) | (b3 >> 6);
    let c4 = b3 & 0x3F;
    buf.push(encode6bit(c1 & 0x3F));
    buf.push(encode6bit(c2 & 0x3F));
    buf.push(encode6bit(c3 & 0x3F));
    buf.push(encode6bit(c4 & 0x3F));
}

fn encode6bit(b: u8) -> char {
    match b {
        0..=9 => (b'0' + b) as char,
        10..=35 => (b'A' + (b - 10)) as char,
        36..=61 => (b'a' + (b - 36)) as char,
        62 => '-',
        63 => '_',
        _ => '?',
    }
}

fn wrap_with_meta(body_svg: &str, meta: &DiagramMeta, diagram_type: &str) -> Result<String> {
    let (body_w, body_h) = extract_dimensions(body_svg);
    let body_content = extract_svg_content(body_svg);
    let top_h = meta_top_height(meta);
    let bottom_h = meta_bottom_height(meta);
    let total_w = body_w.max(meta_required_width(meta));
    let total_h = top_h + body_h + bottom_h;
    let body_x = ((total_w - body_w) / 2.0).max(0.0);

    let mut buf = String::with_capacity(body_svg.len() + 1024);
    write_svg_root(&mut buf, total_w, total_h, diagram_type);
    buf.push_str("<defs/><g>");

    let cx = total_w / 2.0;
    let mut y_cursor = 0.0;

    // Header
    if let Some(ref hdr) = meta.header {
        let start_y = y_cursor + META_LINE_HEIGHT;
        let lines = render_creole_text(
            &mut buf,
            hdr,
            cx,
            start_y,
            META_LINE_HEIGHT,
            LABEL_COLOR,
            Some("middle"),
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
        y_cursor += lines as f64 * META_LINE_HEIGHT + META_GAP;
    }

    // Title
    if let Some(ref title) = meta.title {
        y_cursor += META_TITLE_FONT_SIZE;
        let lines = render_creole_text(
            &mut buf,
            title,
            cx,
            y_cursor,
            META_TITLE_FONT_SIZE,
            LABEL_COLOR,
            Some("middle"),
            &format!(r#"font-size="{META_TITLE_FONT_SIZE}" font-weight="bold""#),
        );
        let _ = lines;
    }

    // Body
    write!(buf, r#"<g transform="translate({},{})">"#, fmt_coord(body_x), fmt_coord(top_h)).unwrap();
    buf.push('\n');
    buf.push_str(&body_content);
    buf.push_str("</g>\n");

    let mut y_bottom = top_h + body_h + META_GAP;

    // Caption
    if let Some(ref cap) = meta.caption {
        y_bottom += META_LINE_HEIGHT;
        let lines = render_creole_text(
            &mut buf,
            cap,
            cx,
            y_bottom,
            META_LINE_HEIGHT,
            LABEL_COLOR,
            Some("middle"),
            &format!(r#"font-size="{FONT_SIZE}" font-style="italic""#),
        );
        y_bottom += (lines.saturating_sub(1)) as f64 * META_LINE_HEIGHT;
    }

    // Footer
    if let Some(ref ftr) = meta.footer {
        y_bottom += META_GAP;
        let start_y = y_bottom + META_LINE_HEIGHT;
        let lines = render_creole_text(
            &mut buf,
            ftr,
            cx,
            start_y,
            META_LINE_HEIGHT,
            LABEL_COLOR,
            Some("middle"),
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
        y_bottom += lines as f64 * META_LINE_HEIGHT;
    }

    // Legend
    if let Some(ref leg) = meta.legend {
        y_bottom += META_GAP;
        let line_count = count_creole_lines(leg) as f64;
        let leg_text_h = line_count * META_LINE_HEIGHT;
        let leg_h = leg_text_h + LEGEND_PADDING * 2.0;
        let leg_w = {
            let max_len = max_creole_plain_line_len(leg).max(6) as f64;
            max_len * font_metrics::char_width('a', "SansSerif", FONT_SIZE, false, false)
                + LEGEND_PADDING * 2.0
        };
        let leg_x = total_w - leg_w - MARGIN;
        let leg_y = y_bottom;
        write!(buf,
            r#"<rect fill="{LEGEND_BG}" height="{}" style="stroke:{LEGEND_BORDER_COLOR};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(leg_h), fmt_coord(leg_w), fmt_coord(leg_x), fmt_coord(leg_y),
        ).unwrap();
        buf.push('\n');
        let lx = leg_x + LEGEND_PADDING;
        let ly = leg_y + LEGEND_PADDING + META_LINE_HEIGHT;
        render_creole_text(
            &mut buf,
            leg,
            lx,
            ly,
            META_LINE_HEIGHT,
            LABEL_COLOR,
            None,
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── Class diagram rendering ─────────────────────────────────────────

fn render_class(
    cd: &crate::model::ClassDiagram,
    layout: &GraphLayout,
    skin: &SkinParams,
) -> Result<String> {
    // Java viewBox calculation (traced from SvgGraphics.ensureVisible):
    //   ensureVisible(minDim.width, minDim.height) where
    //   minDim = calculateDimension() + doc_margin(R=5, B=5)
    //   calculateDimension() = LimitFinder_span + delta(15, 15)
    //   LimitFinder_span = entity_span + 1 (rect x-1/y-1 + UEmpty correction)
    //   ensureVisible: maxX = (int)(x + 1)
    //
    // Full formula: (int)(entity_span + 1 + 15 + 5 + 1) = (int)(entity_span + 22)
    // Java viewBox = (int)(minDim + 1) where minDim = calculateDimension + doc_margin(5).
    // calculateDimension = LimitFinder_span + delta(15).
    // LimitFinder_span = (rect_max_x - rect_min_x) where:
    //   rect_min_x = entity_left - 1 (LimitFinder drawRectangle adds x-1)
    //   rect_max_x = entity_right - 1 (LimitFinder drawRectangle adds x+w-1)
    //   So span = (entity_right - 1) - (entity_left - 1) = entity_right - entity_left = entity_span
    //
    // But if there are multiple entities with UEmpty, the max_x gets corrected to entity_right.
    // And if there are arrow polygons, HACK_X_FOR_POLYGON(10) extends the bounding box.
    //
    // For now, use entity_span directly (rect terms cancel out) plus polygon hack contribution.
    let mut lf_min_x = f64::INFINITY;
    let mut lf_max_x = f64::NEG_INFINITY;
    let mut lf_min_y = f64::INFINITY;
    let mut lf_max_y = f64::NEG_INFINITY;
    // Entity rect bounds: LimitFinder adds (x-1) for min and (x+w-1) for max
    for node in &layout.nodes {
        let x = node.cx - node.width / 2.0 + EDGE_OFFSET;
        let y = node.cy - node.height / 2.0 + EDGE_OFFSET;
        lf_min_x = lf_min_x.min(x - 1.0);
        lf_min_y = lf_min_y.min(y - 1.0);
        lf_max_x = lf_max_x.max(x + node.width - 1.0);
        lf_max_y = lf_max_y.max(y + node.height - 1.0);
    }
    // UEmpty correction: for entities with body content, max is pushed to x+w, y+h
    // This happens when body.drawU triggers a UEmpty at the bottom/right
    // For multi-entity diagrams, this +1 is typically present
    if layout.nodes.len() > 1 {
        for node in &layout.nodes {
            let x = node.cx - node.width / 2.0 + EDGE_OFFSET;
            let y = node.cy - node.height / 2.0 + EDGE_OFFSET;
            lf_max_x = lf_max_x.max(x + node.width);
            lf_max_y = lf_max_y.max(y + node.height);
        }
    }
    // Edge arrow polygons: HACK_X_FOR_POLYGON = 10
    for edge in &layout.edges {
        if let Some(ref pts) = edge.arrow_polygon_points {
            for &(px, py) in pts {
                let ax = px + EDGE_OFFSET;
                let ay = py + EDGE_OFFSET;
                lf_min_x = lf_min_x.min(ax - 10.0);
                lf_max_x = lf_max_x.max(ax + 10.0);
                lf_min_y = lf_min_y.min(ay);
                lf_max_y = lf_max_y.max(ay);
            }
        }
    }
    let span_w = if lf_max_x.is_finite() { lf_max_x - lf_min_x } else { layout.total_width };
    let span_h = if lf_max_y.is_finite() { lf_max_y - lf_min_y } else { layout.total_height };
    let svg_w = (span_w + CANVAS_DELTA + DOC_MARGIN_RIGHT + 1.0).floor();
    let svg_h = (span_h + CANVAS_DELTA + DOC_MARGIN_BOTTOM + 1.0).floor();
    let mut buf = String::with_capacity(4096);
    write_svg_root(&mut buf, svg_w, svg_h, "CLASS");
    let arrow_color = skin.arrow_color(LINK_COLOR);
    buf.push_str("<defs/><g>");

    let node_map: HashMap<&str, &NodeLayout> =
        layout.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Build entity id map for link references
    let mut entity_ids: HashMap<String, String> = HashMap::new();
    let mut ent_counter = 2u32; // Java starts entity IDs at ent0002
    for entity in &cd.entities {
        let ent_id = format!("ent{:04}", ent_counter);
        entity_ids.insert(sanitize_id(&entity.name), ent_id);
        ent_counter += 1;
    }

    for entity in &cd.entities {
        let sid = sanitize_id(&entity.name);
        if let Some(nl) = node_map.get(sid.as_str()) {
            let ent_id = entity_ids
                .get(&sid)
                .map(|s| s.as_str())
                .unwrap_or("ent0000");
            write!(
                buf,
                "<!--class {}--><g class=\"entity\" data-qualified-name=\"{}\"",
                xml_escape(&entity.name),
                xml_escape(&entity.name),
            )
            .unwrap();
            if let Some(source_line) = entity.source_line {
                write!(buf, " data-source-line=\"{source_line}\"").unwrap();
            }
            write!(buf, " id=\"{ent_id}\">").unwrap();
            draw_entity_box(&mut buf, cd, entity, nl, skin);
            buf.push_str("</g>");
        }
    }

    let mut link_counter = ent_counter;
    for link in &cd.links {
        let from_id = sanitize_id(&link.from);
        let to_id = sanitize_id(&link.to);
        if let Some(el) = layout
            .edges
            .iter()
            .find(|e| e.from == from_id && e.to == to_id)
        {
            let from_ent = entity_ids.get(&from_id).map(|s| s.as_str()).unwrap_or("");
            let to_ent = entity_ids.get(&to_id).map(|s| s.as_str()).unwrap_or("");
            let link_type = derive_link_type(link);
            write!(
                buf,
                "<!--link {} to {}--><g class=\"link\" data-entity-1=\"{}\" data-entity-2=\"{}\" data-link-type=\"{}\"",
                xml_escape(&link.from),
                xml_escape(&link.to),
                from_ent,
                to_ent,
                link_type,
            )
            .unwrap();
            if let Some(source_line) = link.source_line {
                write!(buf, " data-source-line=\"{source_line}\"").unwrap();
            }
            write!(buf, " id=\"lnk{link_counter}\">").unwrap();
            draw_edge(&mut buf, link, el, arrow_color);
            buf.push_str("</g>");
            link_counter += 1;
        }
    }

    // Notes
    for note in &layout.notes {
        draw_class_note(&mut buf, note);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── Stereotype circle glyph paths ───────────────────────────────────
// Raw glyph outline coordinates from Java AWT TextLayout.getOutline().
// Font: Monospaced Bold 17pt (PlantUML FontParam.CIRCLED_CHARACTER).
// Coordinates are relative to the text draw position (0, 0).
//
// UnusedSpace center offsets from PlantUML's UnusedSpace algorithm,
// extracted via Java instrumentation on the reference generation machine.
//
// At render time:
//   offset_x = circle_abs_cx - CENTER_X - 0.5
//   offset_y = circle_abs_cy - CENTER_Y - 0.5
//   final_coord = raw_coord + offset

// UnusedSpace centers from PlantUML's actual runtime values.
// Extracted via Java instrumentation: char='X' centerX=... centerY=...
// These depend on font rendering and MUST match the reference generation machine.
const GLYPH_C_CENTER: (f64, f64) = (5.5, -6.5);
const GLYPH_I_CENTER: (f64, f64) = (5.0, -6.5);
const GLYPH_E_CENTER: (f64, f64) = (4.5, -6.5);
const GLYPH_A_CENTER: (f64, f64) = (4.5, -6.0);

// Raw glyph path segments from Java AWT TextLayout.getOutline().
// Coordinates at full f64 precision (all are exact binary fractions from TrueType hinting).
const GLYPH_C_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(8.96875, -0.359375)]),
    ('Q', &[(8.390625, -0.0625), (7.75, 0.078125)]),
    ('Q', &[(7.109375, 0.234375), (6.40625, 0.234375)]),
    ('Q', &[(3.90625, 0.234375), (2.578125, -1.40625)]),
    ('Q', &[(1.265625, -3.0625), (1.265625, -6.1875)]),
    ('Q', &[(1.265625, -9.3125), (2.578125, -10.96875)]),
    ('Q', &[(3.90625, -12.625), (6.40625, -12.625)]),
    ('Q', &[(7.109375, -12.625), (7.75, -12.46875)]),
    ('Q', &[(8.40625, -12.3125), (8.96875, -12.015625)]),
    ('L', &[(8.96875, -9.296875)]),
    ('Q', &[(8.34375, -9.875), (7.75, -10.140625)]),
    ('Q', &[(7.15625, -10.421875), (6.53125, -10.421875)]),
    ('Q', &[(5.1875, -10.421875), (4.5, -9.34375)]),
    ('Q', &[(3.8125, -8.28125), (3.8125, -6.1875)]),
    ('Q', &[(3.8125, -4.09375), (4.5, -3.015625)]),
    ('Q', &[(5.1875, -1.953125), (6.53125, -1.953125)]),
    ('Q', &[(7.15625, -1.953125), (7.75, -2.21875)]),
    ('Q', &[(8.34375, -2.5), (8.96875, -3.078125)]),
    ('L', &[(8.96875, -0.359375)]),
    ('Z', &[]),
];

const GLYPH_I_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(1.421875, -10.234375)]),
    ('L', &[(1.421875, -12.390625)]),
    ('L', &[(8.8125, -12.390625)]),
    ('L', &[(8.8125, -10.234375)]),
    ('L', &[(6.34375, -10.234375)]),
    ('L', &[(6.34375, -2.15625)]),
    ('L', &[(8.8125, -2.15625)]),
    ('L', &[(8.8125, 0.0)]),
    ('L', &[(1.421875, 0.0)]),
    ('L', &[(1.421875, -2.15625)]),
    ('L', &[(3.890625, -2.15625)]),
    ('L', &[(3.890625, -10.234375)]),
    ('L', &[(1.421875, -10.234375)]),
    ('Z', &[]),
];

const GLYPH_E_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(9.109375, 0.0)]),
    ('L', &[(1.390625, 0.0)]),
    ('L', &[(1.390625, -12.390625)]),
    ('L', &[(9.109375, -12.390625)]),
    ('L', &[(9.109375, -10.234375)]),
    ('L', &[(3.84375, -10.234375)]),
    ('L', &[(3.84375, -7.5625)]),
    ('L', &[(8.609375, -7.5625)]),
    ('L', &[(8.609375, -5.40625)]),
    ('L', &[(3.84375, -5.40625)]),
    ('L', &[(3.84375, -2.15625)]),
    ('L', &[(9.109375, -2.15625)]),
    ('L', &[(9.109375, 0.0)]),
    ('Z', &[]),
];

const GLYPH_A_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(5.109375, -10.15625)]),
    ('L', &[(3.953125, -5.078125)]),
    ('L', &[(6.28125, -5.078125)]),
    ('L', &[(5.109375, -10.15625)]),
    ('Z', &[]),
    ('M', &[(3.625, -12.390625)]),
    ('L', &[(6.609375, -12.390625)]),
    ('L', &[(9.96875, 0.0)]),
    ('L', &[(7.515625, 0.0)]),
    ('L', &[(6.75, -3.0625)]),
    ('L', &[(3.46875, -3.0625)]),
    ('L', &[(2.71875, 0.0)]),
    ('L', &[(0.28125, 0.0)]),
    ('L', &[(3.625, -12.390625)]),
    ('Z', &[]),
];

/// Emit a stereotype circle glyph path element.
/// `circle_cx` and `circle_cy` are the absolute SVG coordinates of the circle center.
fn emit_circle_glyph(buf: &mut String, kind: &EntityKind, circle_cx: f64, circle_cy: f64) {
    let (glyph_raw, center) = match kind {
        EntityKind::Class | EntityKind::Object => (GLYPH_C_RAW, GLYPH_C_CENTER),
        EntityKind::Abstract => (GLYPH_A_RAW, GLYPH_A_CENTER),
        EntityKind::Interface => (GLYPH_I_RAW, GLYPH_I_CENTER),
        EntityKind::Enum => (GLYPH_E_RAW, GLYPH_E_CENTER),
        EntityKind::Annotation => return,
    };

    // Java DriverCenteredCharacterSvg algorithm:
    //   xpos = circle_center_in_ug - centerX - 0.5
    //   ypos = circle_center_in_ug - centerY - 0.5
    //   final = path_coord + (xpos, ypos)
    let dx = circle_cx - center.0 - 0.5;
    let dy = circle_cy - center.1 - 0.5;

    let mut d = String::with_capacity(512);
    for (cmd, points) in glyph_raw {
        d.push(*cmd);
        for (i, &(px, py)) in points.iter().enumerate() {
            if i > 0 {
                d.push(' ');
            }
            d.push_str(&fmt_coord(px + dx));
            d.push(',');
            d.push_str(&fmt_coord(py + dy));
        }
        // Java SvgGraphics: every command (including Z) has a trailing space
        d.push(' ');
    }

    write!(buf, r##"<path d="{d}" fill="#000000"/>"##).unwrap();
}

/// Offset all coordinates in a glyph path string by (dx, dy).
/// The path uses M, Q, L, Z commands with absolute coordinates.
/// Format: "Mx,y Qx,y x,y Lx,y Z"
fn offset_glyph_path_xy(path: &str, dx: f64, dy: f64) -> String {
    if dx == 0.0 && dy == 0.0 {
        return path.to_string();
    }
    let mut result = String::with_capacity(path.len() + 64);
    let mut chars = path.chars().peekable();
    let mut is_x = true; // alternates: first number is X, second is Y

    while let Some(&c) = chars.peek() {
        match c {
            'M' | 'Q' | 'L' | 'C' | 'Z' => {
                result.push(c);
                chars.next();
                is_x = true; // reset after command
            }
            '-' | '0'..='9' | '.' => {
                // Parse number
                let mut s = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_ascii_digit() || nc == '.' || nc == '-' {
                        s.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(val) = s.parse::<f64>() {
                    if is_x {
                        result.push_str(&fmt_coord(val + dx));
                    } else {
                        result.push_str(&fmt_coord(val + dy));
                    }
                    is_x = !is_x;
                } else {
                    result.push_str(&s);
                }
            }
            ',' => {
                result.push(',');
                chars.next();
            }
            ' ' => {
                result.push(' ');
                chars.next();
            }
            _ => {
                result.push(c);
                chars.next();
            }
        }
    }
    result
}


fn stereotype_circle_color(kind: &EntityKind) -> &'static str {
    match kind {
        EntityKind::Class => "#ADD1B2",
        EntityKind::Interface => "#A9DCDF",
        EntityKind::Enum => "#EB937F",
        EntityKind::Abstract => "#A9DCDF",
        EntityKind::Annotation => "#A9DCDF",
        EntityKind::Object => "#ADD1B2",
    }
}

fn draw_entity_box(
    buf: &mut String,
    cd: &ClassDiagram,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
) {
    // Java: entity rect starts at (moveDelta_offset + 1, moveDelta_offset + 1)
    // where the +1 is the border inset (rect drawn 1px inside the Graphviz node boundary)
    let x = nl.cx - nl.width / 2.0 + MARGIN + 1.0;
    let y = nl.cy - nl.height / 2.0 + MARGIN + 1.0;
    let w = nl.width;
    let h = nl.height;

    let (default_bg, default_border, element_type) = match entity.kind {
        EntityKind::Class => (CLASS_BG, CLASS_BORDER, "class"),
        EntityKind::Interface => (IFACE_BG, IFACE_BORDER, "interface"),
        EntityKind::Enum => (ENUM_BG, ENUM_BORDER, "enum"),
        EntityKind::Abstract => (ABSTRACT_BG, ABSTRACT_BORDER, "abstract"),
        EntityKind::Annotation => (CLASS_BG, CLASS_BORDER, "annotation"),
        EntityKind::Object => (CLASS_BG, CLASS_BORDER, "object"),
    };
    let default_fill = skin.background_color(element_type, default_bg);
    let fill = entity.color.as_deref().unwrap_or(default_fill);
    let stroke = skin.border_color(element_type, default_border);
    let font_color = skin.font_color(element_type, LABEL_COLOR);

    let rx = skin.round_corner().unwrap_or(2.5);

    // Rect with rx="2.5" ry="2.5" to match Java PlantUML
    write!(buf,
        r#"<rect fill="{fill}" height="{}" rx="{}" ry="{}" style="stroke:{stroke};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(h), fmt_coord(rx), fmt_coord(rx), fmt_coord(w), fmt_coord(x), fmt_coord(y),
    ).unwrap();

    let class_font_size = skin.font_size("class", FONT_SIZE);
    let attr_font_size = skin.font_size("classattribute", class_font_size);

    let name_display = if let Some(ref g) = entity.generic {
        format!("{}<{}>", entity.name, g)
    } else {
        entity.name.clone()
    };
    let name_escaped = xml_escape(&name_display);
    let visible_stereotypes = visible_stereotype_labels(&cd.hide_show_rules, entity);
    let show_fields = show_portion(&cd.hide_show_rules, ClassPortion::Field, &entity.name);
    let show_methods = show_portion(&cd.hide_show_rules, ClassPortion::Method, &entity.name);
    let visible_fields: Vec<&Member> = entity
        .members
        .iter()
        .filter(|m| !m.is_method)
        .filter(|_| show_fields)
        .collect();
    let visible_methods: Vec<&Member> = entity
        .members
        .iter()
        .filter(|m| m.is_method)
        .filter(|_| show_methods)
        .collect();
    let has_kind_label = matches!(
        entity.kind,
        EntityKind::Interface | EntityKind::Enum | EntityKind::Annotation
    );

    if has_kind_label {
        let kind_text = match entity.kind {
            EntityKind::Interface => "\u{00AB}interface\u{00BB}",
            EntityKind::Enum => "\u{00AB}enumeration\u{00BB}",
            EntityKind::Annotation => "\u{00AB}annotation\u{00BB}",
            _ => "",
        };
        let kind_y = y + HEADER_HEIGHT * 0.38;
        let name_y = y + HEADER_HEIGHT * 0.82;
        let cx = x + w / 2.0;
        write!(buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{fs:.0}" font-style="italic" text-anchor="middle" x="{}" y="{}">{kind_text}</text>"#,
            fmt_coord(cx), fmt_coord(kind_y), fs = class_font_size - 2.0,
        ).unwrap();
        write!(buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{class_font_size:.0}" font-weight="bold" text-anchor="middle" x="{}" y="{}">{name_escaped}</text>"#,
            fmt_coord(cx), fmt_coord(name_y),
        ).unwrap();
    } else {
        let italic_name = entity.kind == EntityKind::Abstract;
        let name_width = font_metrics::text_width(
            &name_display,
            "SansSerif",
            class_font_size,
            false,
            italic_name,
        );
        let name_block_width = name_width + HEADER_NAME_BLOCK_MARGIN_X;
        let stereo_widths: Vec<f64> = visible_stereotypes
            .iter()
            .map(|label| {
                font_metrics::text_width(
                    &format!("\u{00AB}{label}\u{00BB}"),
                    "SansSerif",
                    HEADER_STEREO_FONT_SIZE,
                    false,
                    true,
                )
            })
            .collect();
        let stereo_block_width = stereo_widths.iter().copied().fold(0.0_f64, f64::max);
        let width_stereo_and_name = name_block_width.max(stereo_block_width);
        let stereo_height = visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT;
        let header_height = HEADER_CIRCLE_BLOCK_HEIGHT
            .max(stereo_height + HEADER_NAME_BLOCK_HEIGHT + HEADER_STEREO_NAME_GAP);
        let supp_width = (w - HEADER_CIRCLE_BLOCK_WIDTH - width_stereo_and_name).max(0.0);
        let h2 = (HEADER_CIRCLE_BLOCK_WIDTH / 4.0).min(supp_width * 0.1);
        let h1 = (supp_width - h2) / 2.0;

        let circle_color = stereotype_circle_color(&entity.kind);
        let circle_block_x = x + h1;
        let ecx = circle_block_x + 15.0;
        let ecy = y + header_height / 2.0;
        write!(buf,
            r#"<ellipse cx="{}" cy="{}" fill="{circle_color}" rx="11" ry="11" style="stroke:#181818;stroke-width:1;"/>"#,
            fmt_coord(ecx), fmt_coord(ecy),
        ).unwrap();
        emit_circle_glyph(buf, &entity.kind, ecx, ecy);

        let header_top_offset = (header_height - stereo_height - HEADER_NAME_BLOCK_HEIGHT) / 2.0;
        for (idx, label) in visible_stereotypes.iter().enumerate() {
            let stereo_text = format!("\u{00AB}{label}\u{00BB}");
            let stereo_x = x
                + HEADER_CIRCLE_BLOCK_WIDTH
                + (width_stereo_and_name - stereo_widths[idx]) / 2.0
                + h1
                + h2;
            let stereo_y = y
                + header_top_offset
                + HEADER_STEREO_BASELINE
                + idx as f64 * HEADER_STEREO_LINE_HEIGHT;
            write!(
                buf,
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="12" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(stereo_widths[idx]),
                fmt_coord(stereo_x),
                fmt_coord(stereo_y),
                xml_escape(&stereo_text),
            )
            .unwrap();
        }

        let name_x = x
            + HEADER_CIRCLE_BLOCK_WIDTH
            + (width_stereo_and_name - name_block_width) / 2.0
            + h1
            + h2
            + 3.0;
        let name_y = y + header_top_offset + stereo_height + HEADER_NAME_BASELINE;
        let font_style_attr = if entity.kind == EntityKind::Abstract {
            r#" font-style="italic""#
        } else {
            ""
        };
        let text_deco_attr = if entity.kind == EntityKind::Object {
            r#" text-decoration="underline""#
        } else {
            ""
        };
        let tl = fmt_coord(name_width);
        write!(buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{class_font_size:.0}"{font_style_attr} lengthAdjust="spacing" textLength="{tl}"{text_deco_attr} x="{}" y="{}">{name_escaped}</text>"#,
            fmt_coord(name_x), fmt_coord(name_y),
        ).unwrap();
    }

    let x1_val = fmt_coord(x + 1.0);
    let x2_val = fmt_coord(x + w - 1.0);
    let header_height = if has_kind_label {
        HEADER_HEIGHT
    } else {
        HEADER_CIRCLE_BLOCK_HEIGHT.max(
            visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT
                + HEADER_NAME_BLOCK_HEIGHT
                + HEADER_STEREO_NAME_GAP,
        )
    };
    let mut section_y = y + header_height;
    if show_fields {
        draw_member_section(
            buf,
            &visible_fields,
            section_y,
            x,
            &x1_val,
            &x2_val,
            font_color,
            attr_font_size,
        );
        section_y += section_height(&visible_fields);
    }
    if show_methods {
        draw_member_section(
            buf,
            &visible_methods,
            section_y,
            x,
            &x1_val,
            &x2_val,
            font_color,
            attr_font_size,
        );
    }
}

fn draw_member_section(
    buf: &mut String,
    members: &[&Member],
    section_y: f64,
    x: f64,
    x1_val: &str,
    x2_val: &str,
    font_color: &str,
    attr_font_size: f64,
) {
    let sep_y_str = fmt_coord(section_y);
    write!(
        buf,
        r#"<line style="stroke:#181818;stroke-width:0.5;" x1="{x1_val}" x2="{x2_val}" y1="{sep_y_str}" y2="{sep_y_str}"/>"#,
    )
    .unwrap();
    for (i, member) in members.iter().enumerate() {
        let icon_y = section_y + MEMBER_ICON_Y_OFFSET + i as f64 * MEMBER_ROW_HEIGHT;
        let text_y = section_y + MEMBER_TEXT_Y_OFFSET + i as f64 * MEMBER_ROW_HEIGHT;
        let text = member_text(member);
        let text_escaped = xml_escape(&text);
        if let Some(visibility) = &member.visibility {
            draw_visibility_icon(
                buf,
                visibility,
                member.is_method,
                x + MEMBER_ICON_X_OFFSET,
                icon_y,
            );
        }
        let font_style_attr = if member.modifiers.is_abstract {
            r#" font-style="italic""#
        } else {
            ""
        };
        let text_deco_attr = if member.modifiers.is_static {
            r#" text-decoration="underline""#
        } else {
            ""
        };
        let text_x = x + if member.visibility.is_some() {
            MEMBER_TEXT_X_WITH_ICON
        } else {
            MEMBER_TEXT_X_NO_ICON
        };
        write!(
            buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{attr_font_size:.0}"{font_style_attr}{text_deco_attr} lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{text_escaped}</text>"#,
            fmt_coord(font_metrics::text_width(&text, "SansSerif", attr_font_size, false, member.modifiers.is_abstract)),
            fmt_coord(text_x),
            fmt_coord(text_y),
        )
        .unwrap();
    }
}

fn section_height(members: &[&Member]) -> f64 {
    if members.is_empty() {
        EMPTY_COMPARTMENT
    } else {
        MEMBER_BLOCK_HEIGHT_ONE_ROW + (members.len().saturating_sub(1)) as f64 * MEMBER_ROW_HEIGHT
    }
}

fn member_text(m: &Member) -> String {
    match &m.return_type {
        Some(rt) => format!("{}: {rt}", m.name),
        None => m.name.clone(),
    }
}

fn draw_visibility_icon(
    buf: &mut String,
    visibility: &Visibility,
    is_method: bool,
    x: f64,
    y: f64,
) {
    let modifier = match (visibility, is_method) {
        (Visibility::Public, true) => "PUBLIC_METHOD",
        (Visibility::Package, true) => "PACKAGE_PRIVATE_METHOD",
        _ => return,
    };
    write!(buf, r#"<g data-visibility-modifier="{modifier}">"#).unwrap();
    match modifier {
        "PUBLIC_METHOD" => {
            write!(
                buf,
                r##"<ellipse cx="{}" cy="{}" fill="#84BE84" rx="3" ry="3" style="stroke:#038048;stroke-width:1;"/>"##,
                fmt_coord(x + 3.0),
                fmt_coord(y + 3.0),
            )
            .unwrap();
        }
        "PACKAGE_PRIVATE_METHOD" => {
            write!(
                buf,
                r##"<polygon fill="#4177AF" points="{},{},{},{},{},{}" style="stroke:#1963A0;stroke-width:1;"/>"##,
                fmt_coord(x + 5.0),
                fmt_coord(y + 1.0),
                fmt_coord(x + 1.0),
                fmt_coord(y + 7.0),
                fmt_coord(x + 9.0),
                fmt_coord(y + 7.0),
            )
            .unwrap();
        }
        _ => {}
    }
    buf.push_str("</g>");
}

fn show_portion(rules: &[ClassHideShowRule], portion: ClassPortion, entity_name: &str) -> bool {
    let mut result = true;
    for rule in rules {
        if rule.portion != portion {
            continue;
        }
        match &rule.target {
            ClassRuleTarget::Any => result = rule.show,
            ClassRuleTarget::Entity(name) if name == entity_name => result = rule.show,
            _ => {}
        }
    }
    result
}

fn visible_stereotype_labels(rules: &[ClassHideShowRule], entity: &Entity) -> Vec<String> {
    entity
        .stereotypes
        .iter()
        .map(|st| st.0.clone())
        .filter(|label| stereotype_label_visible(rules, label))
        .collect()
}

fn stereotype_label_visible(rules: &[ClassHideShowRule], label: &str) -> bool {
    let mut result = true;
    for rule in rules {
        if rule.portion != ClassPortion::Stereotype {
            continue;
        }
        match &rule.target {
            ClassRuleTarget::Any => result = rule.show,
            ClassRuleTarget::Stereotype(name) if name == label => result = rule.show,
            _ => {}
        }
    }
    result
}

fn format_member(m: &Member) -> String {
    let vis = match &m.visibility {
        Some(Visibility::Public) => "+ ",
        Some(Visibility::Private) => "- ",
        Some(Visibility::Protected) => "# ",
        Some(Visibility::Package) => "~ ",
        None => "",
    };
    match &m.return_type {
        Some(rt) => format!("{vis}{}: {rt}", m.name),
        None => format!("{vis}{}", m.name),
    }
}

/// Derive the `data-link-type` attribute value from the link's arrow and line style.
fn derive_link_type(link: &Link) -> &'static str {
    // Check the "dominant" arrowhead (right_head for A-->B, left_head for B<--A)
    let head = if link.right_head != ArrowHead::None {
        &link.right_head
    } else {
        &link.left_head
    };
    match head {
        ArrowHead::Triangle => {
            if link.line_style == LineStyle::Dashed {
                "realisation"
            } else {
                "extension"
            }
        }
        ArrowHead::Diamond => "composition",
        ArrowHead::DiamondHollow => "aggregation",
        ArrowHead::Arrow => "dependency",
        ArrowHead::Plus => "innerclass",
        ArrowHead::None => "association",
    }
}

fn draw_edge(buf: &mut String, link: &Link, el: &EdgeLayout, link_color: &str) {
    if el.points.is_empty() {
        return;
    }

    let mut path_points = el.points.clone();
    if link.left_head != ArrowHead::None {
        shorten_edge_for_head(&mut path_points, &link.left_head, true);
    }
    if link.right_head != ArrowHead::None {
        shorten_edge_for_head(&mut path_points, &link.right_head, false);
    }

    let d = build_edge_path_d(&path_points, EDGE_OFFSET);

    let dash = if link.line_style == LineStyle::Dashed {
        r#" stroke-dasharray="7,5""#
    } else {
        ""
    };
    let path_id = format!("{}-to-{}", link.from, link.to);
    write!(buf, "<path").unwrap();
    if let Some(source_line) = link.source_line {
        write!(buf, r#" codeLine="{source_line}""#).unwrap();
    }
    write!(
        buf,
        r#" d="{d}" fill="none" id="{path_id}" style="stroke:{link_color};stroke-width:1;"{dash}/>"#,
    )
    .unwrap();

    if link.left_head != ArrowHead::None {
        emit_arrowhead(buf, &link.left_head, &el.points, true, link_color);
    }
    if link.right_head != ArrowHead::None {
        emit_arrowhead(buf, &link.right_head, &el.points, false, link_color);
    }

    if let Some(label) = &link.label {
        let mid_idx = path_points.len() / 2;
        let (mx, my) = path_points[mid_idx];
        draw_label(buf, label, mx + EDGE_OFFSET, my + EDGE_OFFSET - 6.0);
    }
}

fn shorten_edge_for_head(points: &mut Vec<(f64, f64)>, head: &ArrowHead, is_start: bool) {
    let decoration_length = decoration_length(head);
    if decoration_length == 0.0 || points.is_empty() {
        return;
    }

    if is_start {
        let angle = edge_start_angle(points);
        move_edge_start_point(
            points,
            decoration_length * angle.cos(),
            decoration_length * angle.sin(),
        );
    } else {
        let angle = edge_end_angle(points);
        move_edge_end_point(
            points,
            decoration_length * (angle - std::f64::consts::PI).cos(),
            decoration_length * (angle - std::f64::consts::PI).sin(),
        );
    }
}

fn decoration_length(head: &ArrowHead) -> f64 {
    match head {
        ArrowHead::None => 0.0,
        ArrowHead::Arrow => 6.0,
        ArrowHead::Triangle => 18.0,
        ArrowHead::Diamond | ArrowHead::DiamondHollow => 12.0,
        ArrowHead::Plus => 16.0,
    }
}

fn build_edge_path_d(points: &[(f64, f64)], offset: f64) -> String {
    let mut d = String::new();
    if points.is_empty() {
        return d;
    }

    write!(
        d,
        "M{},{} ",
        fmt_coord(points[0].0 + offset),
        fmt_coord(points[0].1 + offset),
    )
    .unwrap();

    let rest = &points[1..];
    if is_cubic_edge_path(points) {
        for chunk in rest.chunks(3) {
            write!(
                d,
                "C{},{} {},{} {},{} ",
                fmt_coord(chunk[0].0 + offset),
                fmt_coord(chunk[0].1 + offset),
                fmt_coord(chunk[1].0 + offset),
                fmt_coord(chunk[1].1 + offset),
                fmt_coord(chunk[2].0 + offset),
                fmt_coord(chunk[2].1 + offset),
            )
            .unwrap();
        }
    } else {
        for &(x, y) in rest {
            write!(
                d,
                "L{},{} ",
                fmt_coord(x + offset),
                fmt_coord(y + offset),
            )
            .unwrap();
        }
    }
    // Edge paths come from Graphviz SVG which doesn't add trailing space
    // (unlike SvgGraphics glyph paths which do). Trim to match.
    let d = d.trim_end().to_string();
    d
}

fn is_cubic_edge_path(points: &[(f64, f64)]) -> bool {
    points.len() >= 4 && (points.len() - 1).is_multiple_of(3)
}

fn edge_start_angle(points: &[(f64, f64)]) -> f64 {
    let (x1, y1) = points[0];
    let (x2, y2) = if is_cubic_edge_path(points) {
        let (cx, cy) = points[1];
        if (cx - x1).abs() > f64::EPSILON || (cy - y1).abs() > f64::EPSILON {
            (cx, cy)
        } else {
            points[3]
        }
    } else {
        points.get(1).copied().unwrap_or((x1 + 1.0, y1))
    };
    (y2 - y1).atan2(x2 - x1)
}

fn edge_end_angle(points: &[(f64, f64)]) -> f64 {
    let &(x2, y2) = points.last().unwrap();
    let (x1, y1) = if is_cubic_edge_path(points) {
        let (cx, cy) = points[points.len() - 2];
        if (x2 - cx).abs() > f64::EPSILON || (y2 - cy).abs() > f64::EPSILON {
            (cx, cy)
        } else {
            points[points.len() - 4]
        }
    } else {
        points
            .get(points.len().saturating_sub(2))
            .copied()
            .unwrap_or((x2 - 1.0, y2))
    };
    (y2 - y1).atan2(x2 - x1)
}

fn move_edge_start_point(points: &mut Vec<(f64, f64)>, dx: f64, dy: f64) {
    if points.is_empty() {
        return;
    }

    let move_len = (dx * dx + dy * dy).sqrt();
    if is_cubic_edge_path(points) && points.len() >= 7 {
        let first_seg_len =
            ((points[3].0 - points[0].0).powi(2) + (points[3].1 - points[0].1).powi(2)).sqrt();
        if move_len >= first_seg_len {
            let next_dx = dx - (points[3].0 - points[0].0);
            let next_dy = dy - (points[3].1 - points[0].1);
            points.drain(0..3);
            move_edge_start_point(points, next_dx, next_dy);
            return;
        }
    }

    points[0].0 += dx;
    points[0].1 += dy;
    if is_cubic_edge_path(points) {
        points[1].0 += dx;
        points[1].1 += dy;
    }
}

fn move_edge_end_point(points: &mut [(f64, f64)], dx: f64, dy: f64) {
    if points.is_empty() {
        return;
    }

    let last = points.len() - 1;
    points[last].0 += dx;
    points[last].1 += dy;
    if is_cubic_edge_path(points) {
        points[last - 1].0 += dx;
        points[last - 1].1 += dy;
    }
}

fn emit_arrowhead(
    buf: &mut String,
    head: &ArrowHead,
    points: &[(f64, f64)],
    is_start: bool,
    link_color: &str,
) {
    if points.is_empty() || *head == ArrowHead::None {
        return;
    }

    let (tip_x, tip_y) = if is_start {
        (points[0].0 + EDGE_OFFSET, points[0].1 + EDGE_OFFSET)
    } else {
        let (x, y) = points[points.len() - 1];
        (x + EDGE_OFFSET, y + EDGE_OFFSET)
    };

    let base_angle = if is_start {
        edge_start_angle(points) + std::f64::consts::PI
    } else {
        edge_end_angle(points)
    };

    match head {
        ArrowHead::Arrow => emit_rotated_polygon(
            buf,
            &[
                (0.0, 0.0),
                (-9.0, -4.0),
                (-5.0, 0.0),
                (-9.0, 4.0),
                (0.0, 0.0),
            ],
            base_angle,
            tip_x,
            tip_y,
            link_color,
            link_color,
        ),
        ArrowHead::Triangle => emit_rotated_polygon(
            buf,
            &[(0.0, 0.0), (-19.0, -7.0), (-19.0, 7.0), (0.0, 0.0)],
            base_angle + std::f64::consts::FRAC_PI_2,
            tip_x,
            tip_y,
            CLASS_BG,
            link_color,
        ),
        ArrowHead::Diamond => emit_rotated_polygon(
            buf,
            &[
                (0.0, 0.0),
                (-6.0, -4.0),
                (-12.0, 0.0),
                (-6.0, 4.0),
                (0.0, 0.0),
            ],
            base_angle + std::f64::consts::FRAC_PI_2,
            tip_x,
            tip_y,
            link_color,
            link_color,
        ),
        ArrowHead::DiamondHollow => emit_rotated_polygon(
            buf,
            &[
                (0.0, 0.0),
                (-6.0, -4.0),
                (-12.0, 0.0),
                (-6.0, 4.0),
                (0.0, 0.0),
            ],
            base_angle + std::f64::consts::FRAC_PI_2,
            tip_x,
            tip_y,
            "#FFFFFF",
            link_color,
        ),
        ArrowHead::Plus => emit_plus_head(buf, tip_x, tip_y, base_angle, link_color),
        ArrowHead::None => {}
    }
}

fn emit_rotated_polygon(
    buf: &mut String,
    points: &[(f64, f64)],
    angle: f64,
    tx: f64,
    ty: f64,
    fill: &str,
    stroke: &str,
) {
    let cos = angle.cos();
    let sin = angle.sin();
    let mut pts = String::new();
    for (idx, &(x, y)) in points.iter().enumerate() {
        if idx > 0 {
            pts.push(',');
        }
        let rx = x * cos - y * sin + tx;
        let ry = x * sin + y * cos + ty;
        write!(pts, "{},{}", fmt_coord(rx), fmt_coord(ry)).unwrap();
    }
    write!(
        buf,
        r#"<polygon fill="{fill}" points="{pts}" style="stroke:{stroke};stroke-width:1;"/>"#,
    )
    .unwrap();
}

fn emit_plus_head(buf: &mut String, tip_x: f64, tip_y: f64, angle: f64, link_color: &str) {
    let radius = 8.0;
    let center_x = tip_x + radius * angle.sin();
    let center_y = tip_y - radius * angle.cos();
    write!(
        buf,
        r##"<circle cx="{}" cy="{}" fill="#FFFFFF" r="8" style="stroke:{link_color};stroke-width:1;"/>"##,
        fmt_coord(center_x),
        fmt_coord(center_y),
    )
    .unwrap();

    let p1 = point_on_circle(
        center_x,
        center_y,
        radius,
        angle - std::f64::consts::FRAC_PI_2,
    );
    let p2 = point_on_circle(
        center_x,
        center_y,
        radius,
        angle + std::f64::consts::FRAC_PI_2,
    );
    let p3 = point_on_circle(center_x, center_y, radius, angle);
    let p4 = point_on_circle(center_x, center_y, radius, angle + std::f64::consts::PI);
    write!(
        buf,
        r#"<line style="stroke:{link_color};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(p1.0),
        fmt_coord(p2.0),
        fmt_coord(p1.1),
        fmt_coord(p2.1),
    )
    .unwrap();
    write!(
        buf,
        r#"<line style="stroke:{link_color};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(p3.0),
        fmt_coord(p4.0),
        fmt_coord(p3.1),
        fmt_coord(p4.1),
    )
    .unwrap();
}

fn point_on_circle(cx: f64, cy: f64, radius: f64, angle: f64) -> (f64, f64) {
    (cx + radius * angle.cos(), cy + radius * angle.sin())
}

fn draw_label(buf: &mut String, text: &str, x: f64, y: f64) {
    render_creole_text(
        buf,
        text,
        x,
        y,
        LINE_HEIGHT,
        LABEL_COLOR,
        Some("middle"),
        &format!(r#"font-size="{FONT_SIZE}""#),
    );
}

/// Draw a note in class diagrams (yellow sticky box with folded corner)
fn draw_class_note(buf: &mut String, note: &ClassNoteLayout) {
    let x = note.x + MARGIN;
    let y = note.y + MARGIN;
    let w = note.width;
    let h = note.height;

    // body shape (use polygon instead of rect to clip the top-right fold area)
    let fold = NOTE_FOLD;
    // pentagon path: top-left -> top-right(minus fold) -> fold inner corner -> bottom-right -> bottom-left
    write!(buf,
        r#"<polygon fill="{bg}" points="{},{} {},{} {},{} {},{} {},{}" style="stroke:{border};stroke-width:1;"/>"#,
        fmt_coord(x), fmt_coord(y),
        fmt_coord(x + w - fold), fmt_coord(y),
        fmt_coord(x + w), fmt_coord(y + fold),
        fmt_coord(x + w), fmt_coord(y + h),
        fmt_coord(x), fmt_coord(y + h),
        bg = NOTE_BG,
        border = NOTE_BORDER,
    ).unwrap();

    // fold corner triangle
    {
        let cx = fmt_coord(x + w - fold);
        let cy = fmt_coord(y);
        let cy2 = fmt_coord(y + fold);
        let cx2 = fmt_coord(x + w);
        write!(buf,
            r#"<path d="M{cx},{cy} L{cx},{cy2} L{cx2},{cy} Z " fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ).unwrap();
    }

    // text content
    let text_x = x + NOTE_TEXT_PADDING;
    let text_y = y + LINE_HEIGHT;
    render_creole_text(
        buf,
        &note.text,
        text_x,
        text_y,
        LINE_HEIGHT,
        LABEL_COLOR,
        None,
        &format!(r#"font-size="{FONT_SIZE}""#),
    );

    // connector line (dashed)
    if let Some((from_x, from_y, to_x, to_y)) = note.connector {
        write!(buf,
            r#"<line style="stroke:{border};stroke-width:1;stroke-dasharray:5,3;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(from_x + MARGIN),
            fmt_coord(to_x + MARGIN),
            fmt_coord(from_y + MARGIN),
            fmt_coord(to_y + MARGIN),
            border = NOTE_BORDER,
        ).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::layout::graphviz::{EdgeLayout, GraphLayout, NodeLayout};
    use crate::layout::DiagramLayout;
    use crate::model::{
        ArrowHead, ClassDiagram, Diagram, Direction, Entity, EntityKind, LineStyle, Link, Member,
        MemberModifiers, Visibility,
    };

    fn empty_class_diagram() -> ClassDiagram {
        ClassDiagram {
            entities: vec![],
            links: vec![],
            groups: vec![],
            direction: Direction::TopToBottom,
            notes: vec![],
            hide_show_rules: vec![],
            stereotype_backgrounds: HashMap::new(),
        }
    }

    #[test]
    fn test_fmt_coord_matches_java() {
        // Matches Java SvgGraphics.format() behavior exactly
        assert_eq!(fmt_coord(0.0), "0");
        assert_eq!(fmt_coord(1.0), "1");
        assert_eq!(fmt_coord(42.0), "42");
        assert_eq!(fmt_coord(3.5), "3.5");
        assert_eq!(fmt_coord(3.50), "3.5");
        assert_eq!(fmt_coord(3.1234), "3.1234");
        assert_eq!(fmt_coord(3.12340), "3.1234");
        assert_eq!(fmt_coord(3.1200), "3.12");
        assert_eq!(fmt_coord(3.1000), "3.1");
        assert_eq!(fmt_coord(100.0), "100");
        assert_eq!(fmt_coord(-5.25), "-5.25");
        assert_eq!(fmt_coord(0.0001), "0.0001");
        assert_eq!(fmt_coord(0.00001), "0"); // rounds to 0.0000
    }

    fn simple_diagram() -> (Diagram, DiagramLayout) {
        let entity = Entity {
            name: "Foo".into(),
            kind: EntityKind::Class,
            stereotypes: vec![],
            members: vec![
                Member {
                    visibility: Some(Visibility::Public),
                    name: "bar".into(),
                    return_type: Some("String".into()),
                    is_method: false,
                    modifiers: MemberModifiers::default(),
                },
                Member {
                    visibility: Some(Visibility::Private),
                    name: "baz".into(),
                    return_type: None,
                    is_method: true,
                    modifiers: MemberModifiers {
                        is_static: true,
                        is_abstract: false,
                    },
                },
            ],
            color: None,
            generic: None,
            source_line: None,
        };
        let entity2 = Entity {
            name: "Bar".into(),
            kind: EntityKind::Interface,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
            source_line: None,
        };
        let link = Link {
            from: "Foo".into(),
            to: "Bar".into(),
            left_head: ArrowHead::None,
            right_head: ArrowHead::Triangle,
            line_style: LineStyle::Dashed,
            label: Some("implements".into()),
            from_label: None,
            to_label: None,
            source_line: None,
        };
        let mut cd = empty_class_diagram();
        cd.entities = vec![entity, entity2];
        cd.links = vec![link];
        let gl = GraphLayout {
            nodes: vec![
                NodeLayout {
                    id: "Foo".into(),
                    cx: 100.0,
                    cy: 50.0,
                    width: 120.0,
                    height: 80.0,
                },
                NodeLayout {
                    id: "Bar".into(),
                    cx: 100.0,
                    cy: 180.0,
                    width: 120.0,
                    height: 40.0,
                },
            ],
            edges: vec![EdgeLayout {
                from: "Foo".into(),
                to: "Bar".into(),
                points: vec![(100.0, 90.0), (100.0, 160.0)],
                arrow_tip: None,
                raw_path_d: None,
                arrow_polygon_points: None,
            }],
            notes: vec![],
            total_width: 240.0,
            total_height: 220.0,
        };
        (Diagram::Class(cd), DiagramLayout::Class(gl))
    }

    fn default_skin() -> SkinParams {
        SkinParams::default()
    }
    fn default_meta() -> DiagramMeta {
        DiagramMeta::default()
    }

    #[test]
    fn test_basic_render_produces_valid_svg() {
        let (d, l) = simple_diagram();
        let svg = render(&d, &l, &default_skin(), &default_meta()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    #[test]
    fn test_entity_box_contains_name() {
        let (d, l) = simple_diagram();
        let svg = render(&d, &l, &default_skin(), &default_meta()).unwrap();
        assert!(svg.contains("Foo"));
        assert!(svg.contains("Bar"));
        assert!(svg.contains("interface"));
    }

    #[test]
    fn test_edge_rendering_produces_path() {
        let (d, l) = simple_diagram();
        let svg = render(&d, &l, &default_skin(), &default_meta()).unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("stroke-dasharray"));
        assert!(
            svg.contains("<polygon"),
            "arrow should render as inline polygon"
        );
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(xml_escape("A & B"), "A &amp; B");
        assert_eq!(xml_escape("<T>"), "&lt;T&gt;");
        assert_eq!(xml_escape(r#"a"b"#), "a&quot;b");
        assert_eq!(xml_escape("plain"), "plain");
    }

    #[test]
    fn test_member_formatting() {
        let m = Member {
            visibility: Some(Visibility::Protected),
            name: "calc()".into(),
            return_type: Some("int".into()),
            is_method: true,
            modifiers: MemberModifiers::default(),
        };
        assert_eq!(format_member(&m), "# calc(): int");
    }

    #[test]
    fn test_entity_with_special_chars() {
        let entity = Entity {
            name: "Map<K, V>".into(),
            kind: EntityKind::Class,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
            source_line: None,
        };
        let mut cd = empty_class_diagram();
        cd.entities = vec![entity];
        let gl = GraphLayout {
            nodes: vec![NodeLayout {
                id: sanitize_id("Map<K, V>"),
                cx: 80.0,
                cy: 40.0,
                width: 100.0,
                height: 40.0,
            }],
            edges: vec![],
            notes: vec![],
            total_width: 200.0,
            total_height: 100.0,
        };
        let svg = render(
            &Diagram::Class(cd),
            &DiagramLayout::Class(gl),
            &default_skin(),
            &default_meta(),
        )
        .unwrap();
        assert!(svg.contains("Map&lt;K, V&gt;"));
    }

    #[test]
    fn test_object_entity_renders_underlined_name() {
        let entity = Entity {
            name: "myObj".into(),
            kind: EntityKind::Object,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
            source_line: None,
        };
        let mut cd = empty_class_diagram();
        cd.entities = vec![entity];
        let gl = GraphLayout {
            nodes: vec![NodeLayout {
                id: "myObj".into(),
                cx: 80.0,
                cy: 40.0,
                width: 100.0,
                height: 40.0,
            }],
            edges: vec![],
            notes: vec![],
            total_width: 200.0,
            total_height: 100.0,
        };
        let svg = render(
            &Diagram::Class(cd),
            &DiagramLayout::Class(gl),
            &default_skin(),
            &default_meta(),
        )
        .expect("render failed");
        assert!(svg.contains("myObj"), "SVG must contain object name");
        assert!(
            svg.contains(r#"text-decoration="underline""#),
            "object name must have underline text-decoration"
        );
    }

    // ── SkinParams tests ────────────────────────────────────────────

    #[test]
    fn test_skinparam_class_bg() {
        let (d, l) = simple_diagram();
        let mut skin = SkinParams::default();
        skin.set("ClassBackgroundColor", "#AABBCC");
        let svg = render(&d, &l, &skin, &default_meta()).unwrap();
        assert!(svg.contains(r##"fill="#AABBCC""##));
    }

    #[test]
    fn test_skinparam_class_border() {
        let (d, l) = simple_diagram();
        let mut skin = SkinParams::default();
        skin.set("ClassBorderColor", "#112233");
        let svg = render(&d, &l, &skin, &default_meta()).unwrap();
        assert!(svg.contains(r##"stroke:#112233"##));
    }

    #[test]
    fn test_skinparam_arrow_color() {
        let (d, l) = simple_diagram();
        let mut skin = SkinParams::default();
        skin.set("ArrowColor", "#00FF00");
        let svg = render(&d, &l, &skin, &default_meta()).unwrap();
        assert!(svg.contains(r##"stroke:#00FF00"##));
    }

    #[test]
    fn test_skinparam_font_color() {
        let (d, l) = simple_diagram();
        let mut skin = SkinParams::default();
        skin.set("ClassFontColor", "#FF0000");
        let svg = render(&d, &l, &skin, &default_meta()).unwrap();
        assert!(svg.contains(r##"fill="#FF0000""##));
    }

    #[test]
    fn test_default_colors() {
        let (d, l) = simple_diagram();
        let svg = render(&d, &l, &default_skin(), &default_meta()).unwrap();
        assert!(svg.contains(&format!(r#"fill="{CLASS_BG}""#)));
        assert!(svg.contains(&format!(r#"stroke:{CLASS_BORDER}"#)));
    }

    // ── Meta rendering tests ────────────────────────────────────────

    #[test]
    fn test_meta_empty_passthrough() {
        let (d, l) = simple_diagram();
        let svg = render(&d, &l, &default_skin(), &default_meta()).unwrap();
        assert!(!svg.contains("translate(0,"));
    }

    #[test]
    fn test_meta_title() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            title: Some("My Title".into()),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        assert!(svg.contains("My Title"));
        assert!(svg.contains("font-weight=\"bold\""));
        assert!(svg.contains("font-size=\"18\""));
        assert!(svg.contains("translate("));
    }

    #[test]
    fn test_meta_title_can_expand_canvas_width() {
        let (d, l) = simple_diagram();
        let body_svg = render_body(&d, &l, &default_skin()).unwrap();
        let (body_w, _) = extract_dimensions(&body_svg);
        let meta = DiagramMeta {
            title: Some(
                "This is a deliberately very long title with [[https://example.com Link]]".into(),
            ),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        let (svg_w, _) = extract_dimensions(&svg);
        assert!(svg_w > body_w);
        assert!(svg.contains("translate("));
        assert!(!svg.contains("translate(0.0,"));
    }

    #[test]
    fn test_meta_title_renders_creole_and_link() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            title: Some("**Bold** [[https://example.com{hover} Link]]".into()),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        assert!(svg.contains(r#"font-weight="bold""#));
        assert!(svg.contains(r#"href="https://example.com""#));
        assert!(svg.contains("<title>hover</title>"));
        assert!(svg.contains("Link"));
    }

    #[test]
    fn test_meta_header() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            header: Some("Page Header".into()),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        assert!(svg.contains("Page Header"));
    }

    #[test]
    fn test_meta_footer() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            footer: Some("Page Footer".into()),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        assert!(svg.contains("Page Footer"));
    }

    #[test]
    fn test_meta_caption() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            caption: Some("Figure 1".into()),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        assert!(svg.contains("Figure 1"));
        assert!(svg.contains("font-style=\"italic\""));
    }

    #[test]
    fn test_meta_legend() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            legend: Some("Legend text".into()),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        assert!(svg.contains("Legend text"));
        assert!(svg.contains(LEGEND_BG));
        assert!(svg.contains(LEGEND_BORDER_COLOR));
    }

    #[test]
    fn test_meta_all() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            title: Some("T".into()),
            header: Some("H".into()),
            footer: Some("F".into()),
            caption: Some("C".into()),
            legend: Some("L".into()),
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        for s in &["T", "H", "F", "C", "L"] {
            assert!(svg.contains(s));
        }
    }

    #[test]
    fn test_extract_dimensions() {
        let svg = r#"<svg viewBox="0 0 200.5 300.0" width="200.5" height="300.0">x</svg>"#;
        let (w, h) = extract_dimensions(svg);
        assert!((w - 200.5).abs() < 0.1);
        assert!((h - 300.0).abs() < 0.1);
    }

    #[test]
    fn test_extract_svg_content() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#;
        assert_eq!(extract_svg_content(svg), "<rect/>");
    }

    #[test]
    fn test_extract_svg_content_strips_plantuml_pi() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><?plantuml 1.2026.3beta4?><defs/><g/></svg>"#;
        assert_eq!(extract_svg_content(svg), "<defs/><g/>");
    }

    #[test]
    fn test_encode_plantuml_source_matches_java() {
        let source = "@startuml\nclass A {\n}\n\nclass B{\n}\n\nA -->B\n@enduml\n";
        assert_eq!(
            encode_plantuml_source(source).unwrap(),
            "Iyv9B2vMS5Ievghbuae6Svp0R4S5NLqx9m00"
        );
    }

    #[test]
    fn test_dot_passthrough_produces_valid_svg() {
        let dot_src = "digraph G { A -> B; B -> C; }";
        let svg = render_dot_passthrough(dot_src).expect("dot passthrough failed");
        assert!(svg.contains("<svg"), "must contain <svg tag");
        assert!(svg.contains("</svg>"), "must contain </svg> tag");
        assert!(svg.contains("A"), "must contain node A");
        assert!(svg.contains("B"), "must contain node B");
        assert!(svg.contains("C"), "must contain node C");
    }

    // ── Note rendering tests ────────────────────────────────────────

    #[test]
    fn test_note_renders_polygon_and_text() {
        use crate::layout::graphviz::ClassNoteLayout;

        let entity = Entity {
            name: "Foo".into(),
            kind: EntityKind::Class,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
            source_line: None,
        };
        let mut cd = empty_class_diagram();
        cd.entities = vec![entity];
        cd.notes = vec![crate::model::ClassNote {
            text: "test note".into(),
            position: "right".into(),
            target: Some("Foo".into()),
        }];
        let gl = GraphLayout {
            nodes: vec![NodeLayout {
                id: "Foo".into(),
                cx: 100.0,
                cy: 50.0,
                width: 120.0,
                height: 80.0,
            }],
            edges: vec![],
            notes: vec![ClassNoteLayout {
                text: "test note".into(),
                x: 180.0,
                y: 30.0,
                width: 90.0,
                height: 36.0,
                lines: vec!["test note".into()],
                connector: Some((180.0, 50.0, 160.0, 50.0)),
            }],
            total_width: 300.0,
            total_height: 120.0,
        };
        let svg = render(
            &Diagram::Class(cd),
            &DiagramLayout::Class(gl),
            &default_skin(),
            &default_meta(),
        )
        .unwrap();

        assert!(svg.contains(NOTE_BG), "note should use yellow background");
        assert!(svg.contains("test note"), "note text must appear in SVG");
        assert!(
            svg.contains("<polygon"),
            "note should render as polygon (folded corner)"
        );
        assert!(
            svg.contains("stroke-dasharray"),
            "connector should be dashed"
        );
    }

    #[test]
    fn test_note_without_connector() {
        use crate::layout::graphviz::ClassNoteLayout;

        let mut cd = empty_class_diagram();
        cd.notes = vec![crate::model::ClassNote {
            text: "floating".into(),
            position: "right".into(),
            target: None,
        }];
        let gl = GraphLayout {
            nodes: vec![],
            edges: vec![],
            notes: vec![ClassNoteLayout {
                text: "floating".into(),
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 36.0,
                lines: vec!["floating".into()],
                connector: None,
            }],
            total_width: 100.0,
            total_height: 60.0,
        };
        let svg = render(
            &Diagram::Class(cd),
            &DiagramLayout::Class(gl),
            &default_skin(),
            &default_meta(),
        )
        .unwrap();

        assert!(svg.contains("floating"), "note text must appear");
        assert!(svg.contains(NOTE_BG), "note background must appear");
        // No connector line - count dashed lines (only note polygon, no connector dash)
        let dash_count = svg.matches("stroke-dasharray=\"5,3\"").count();
        assert_eq!(dash_count, 0, "floating note should have no connector line");
    }
}
