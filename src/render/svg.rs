use std::collections::HashMap;
use std::fmt::Write;

use crate::layout::graphviz::{ClassNoteLayout, EdgeLayout, GraphLayout, NodeLayout};
use crate::layout::DiagramLayout;
use crate::model::{
    ArrowHead, Diagram, DiagramMeta, Entity, EntityKind, LineStyle, Link, Member, Visibility,
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
/// Java PlantUML: entities start at (7, 7) from the SVG edge.
const MARGIN: f64 = 7.0;
/// Java PlantUML: delta(15, 15) added to final canvas dimensions.
const CANVAS_PADDING: f64 = 15.0;
const CIRCLE_LEFT_PAD: f64 = 4.0;
const CIRCLE_DIAMETER: f64 = 22.0;
const EMPTY_COMPARTMENT: f64 = 8.0;

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
    if value == 0.0 {
        return "0".into();
    }
    let s = format!("{:.4}", value);
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

/// Write a Java PlantUML-compatible SVG root element and open a `<g>` wrapper.
pub(crate) fn write_svg_root(buf: &mut String, w: f64, h: f64, diagram_type: &str) {
    let wi = w.ceil() as i32;
    let hi = h.ceil() as i32;
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
    buf.push('\n');
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
    // Apply handwritten font override if enabled
    set_default_font_family(skin.handwritten_font_family().map(|s| s.to_string()));
    let body_svg = render_body(diagram, layout, skin)?;
    set_default_font_family(None);
    if meta.is_empty() {
        return Ok(body_svg);
    }
    // Extract diagram type from body SVG's data-diagram-type attribute
    let dtype = body_svg
        .find("data-diagram-type=\"")
        .and_then(|pos| {
            let start = pos + 19;
            body_svg[start..].find('"').map(|end| &body_svg[start..start + end])
        })
        .unwrap_or("CLASS");
    wrap_with_meta(&body_svg, meta, dtype)
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
        (Diagram::Yaml(yd), DiagramLayout::Yaml(yl)) => super::svg_json::render_json(yd, yl, skin),
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
    use std::io::Write as IoWrite;
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
    max_creole_plain_line_len(text) as f64 * font_metrics::char_width('a', "SansSerif", font_size, false, false)
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

fn extract_svg_content(svg: &str) -> String {
    if let Some(tag_end) = svg.find('>') {
        let after_open = &svg[tag_end + 1..];
        if let Some(close_pos) = after_open.rfind("</svg>") {
            return after_open[..close_pos].to_string();
        }
        return after_open.to_string();
    }
    svg.to_string()
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
    write!(buf, r#"<g transform="translate({body_x:.1},{top_h:.1})">"#).unwrap();
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
            max_len * font_metrics::char_width('a', "SansSerif", FONT_SIZE, false, false) + LEGEND_PADDING * 2.0
        };
        let leg_x = total_w - leg_w - MARGIN;
        let leg_y = y_bottom;
        write!(buf,
            r#"<rect fill="{LEGEND_BG}" height="{leg_h:.1}" style="stroke:{LEGEND_BORDER_COLOR};stroke-width:1;" width="{leg_w:.1}" x="{leg_x:.1}" y="{leg_y:.1}"/>"#,
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
    // Java: margin(7) on left + content + delta(15) padding
    // But total_width is now normalized (0-based), so add margin + padding
    let svg_w = layout.total_width + MARGIN + CANVAS_PADDING;
    let svg_h = layout.total_height + MARGIN + CANVAS_PADDING;
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
            let ent_id = entity_ids.get(&sid).map(|s| s.as_str()).unwrap_or("ent0000");
            write!(buf, "<!--class {}--><g class=\"entity\" data-qualified-name=\"{}\" id=\"{}\">",
                xml_escape(&entity.name), xml_escape(&entity.name), ent_id).unwrap();
            draw_entity_box(&mut buf, entity, nl, skin);
            buf.push_str("</g>");
        }
    }

    let mut link_counter = ent_counter + 1;
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
            write!(buf, "<!--link {} to {}--><g class=\"link\" data-entity-1=\"{}\" data-entity-2=\"{}\" id=\"lnk{}\">",
                xml_escape(&link.from), xml_escape(&link.to), from_ent, to_ent, link_counter).unwrap();
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

fn draw_entity_box(buf: &mut String, entity: &Entity, nl: &NodeLayout, skin: &SkinParams) {
    let x = nl.cx - nl.width / 2.0 + MARGIN;
    let y = nl.cy - nl.height / 2.0 + MARGIN;
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
    buf.push('\n');

    let class_font_size = skin.font_size("class", FONT_SIZE);
    let attr_font_size = skin.font_size("classattribute", class_font_size);

    let name_display = if let Some(ref g) = entity.generic {
        format!("{}<{}>", entity.name, g)
    } else {
        entity.name.clone()
    };
    let name_escaped = xml_escape(&name_display);
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
        buf.push('\n');
        write!(buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{class_font_size:.0}" font-weight="bold" text-anchor="middle" x="{}" y="{}">{name_escaped}</text>"#,
            fmt_coord(cx), fmt_coord(name_y),
        ).unwrap();
        buf.push('\n');
    } else {
        // Stereotype circle icon (ellipse)
        let circle_color = stereotype_circle_color(&entity.kind);
        let ecx = x + CIRCLE_LEFT_PAD + CIRCLE_DIAMETER / 2.0; // x + 15
        let ecy = y + 16.0;
        write!(buf,
            r#"<ellipse cx="{}" cy="{}" fill="{circle_color}" rx="11" ry="11" style="stroke:#181818;stroke-width:1;"/>"#,
            fmt_coord(ecx), fmt_coord(ecy),
        ).unwrap();
        buf.push('\n');

        // Class name text: vertically centered in header, right of circle
        let text_w = font_metrics::text_width(&name_display, "SansSerif", class_font_size, false, false);
        let ascent = font_metrics::ascent("SansSerif", class_font_size, false, false);
        let descent = font_metrics::descent("SansSerif", class_font_size, false, false);
        let text_h = ascent + descent;
        let vert_offset = (HEADER_HEIGHT - text_h) / 2.0;
        let name_y = y + vert_offset + ascent;
        let name_x = x + CIRCLE_LEFT_PAD + CIRCLE_DIAMETER + 3.0; // right of circle + gap
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
        let tl = fmt_coord(text_w);
        write!(buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{class_font_size:.0}"{font_style_attr} lengthAdjust="spacing" textLength="{tl}"{text_deco_attr} x="{}" y="{}">{name_escaped}</text>"#,
            fmt_coord(name_x), fmt_coord(name_y),
        ).unwrap();
        buf.push('\n');
    }

    // First separator line (fields)
    let sep_y = y + HEADER_HEIGHT;
    let x1_val = fmt_coord(x + 1.0);
    let x2_val = fmt_coord(x + w - 1.0);
    let sep_y_str = fmt_coord(sep_y);
    write!(buf,
        r#"<line style="stroke:{stroke};stroke-width:0.5;" x1="{x1_val}" x2="{x2_val}" y1="{sep_y_str}" y2="{sep_y_str}"/>"#,
    ).unwrap();
    buf.push('\n');

    // Members section
    let members_x = x + PADDING;
    let mut members_end_y = sep_y;
    for (i, member) in entity.members.iter().enumerate() {
        let my = sep_y + LINE_HEIGHT * (i as f64 + 0.75);
        let text = format_member(member);
        let text_escaped = xml_escape(&text);
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
        write!(buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{attr_font_size:.0}"{font_style_attr}{text_deco_attr} x="{}" y="{}">{text_escaped}</text>"#,
            fmt_coord(members_x), fmt_coord(my),
        ).unwrap();
        buf.push('\n');
        members_end_y = sep_y + LINE_HEIGHT * (i as f64 + 1.0);
    }

    // Second separator line (methods)
    let sep2_y = if entity.members.is_empty() {
        sep_y + 8.0
    } else {
        members_end_y
    };
    let sep2_y_str = fmt_coord(sep2_y);
    write!(buf,
        r#"<line style="stroke:{stroke};stroke-width:0.5;" x1="{x1_val}" x2="{x2_val}" y1="{sep2_y_str}" y2="{sep2_y_str}"/>"#,
    ).unwrap();
    buf.push('\n');
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
        Some(rt) => format!("{vis}{} : {rt}", m.name),
        None => format!("{vis}{}", m.name),
    }
}

fn draw_edge(buf: &mut String, link: &Link, el: &EdgeLayout, link_color: &str) {
    if el.points.is_empty() {
        return;
    }
    let mut d = String::new();
    for (i, &(px, py)) in el.points.iter().enumerate() {
        let ox = px + MARGIN;
        let oy = py + MARGIN;
        if i == 0 {
            write!(d, "M{},{}", fmt_coord(ox), fmt_coord(oy)).unwrap();
        } else {
            write!(d, " L{},{}", fmt_coord(ox), fmt_coord(oy)).unwrap();
        }
    }
    let dash = if link.line_style == LineStyle::Dashed {
        r#" stroke-dasharray="7,5""#
    } else {
        ""
    };
    let path_id = format!("{}-to-{}", link.from, link.to);
    write!(buf,
        r#"<path d="{d}" fill="none" id="{path_id}" style="stroke:{link_color};stroke-width:1;"{dash}/>"#,
    ).unwrap();
    buf.push('\n');

    // Emit inline polygon arrowheads (matching Java PlantUML output)
    if link.left_head != ArrowHead::None {
        let &(px, py) = &el.points[0];
        let tip_x = px + MARGIN;
        let tip_y = py + MARGIN;
        emit_arrowhead_polygon(buf, &link.left_head, tip_x, tip_y, el, true, link_color);
    }
    if link.right_head != ArrowHead::None {
        let &(px, py) = el.points.last().unwrap();
        let tip_x = px + MARGIN;
        let tip_y = py + MARGIN;
        emit_arrowhead_polygon(buf, &link.right_head, tip_x, tip_y, el, false, link_color);
    }

    if let Some(label) = &link.label {
        let mid_idx = el.points.len() / 2;
        let (mx, my) = el.points[mid_idx];
        draw_label(buf, label, mx + MARGIN, my + MARGIN - 6.0);
    }
}

/// Emit an inline `<polygon>` for an arrowhead at the given tip position.
/// `is_start` indicates whether this is the start (left) or end (right) arrow.
fn emit_arrowhead_polygon(
    buf: &mut String,
    head: &ArrowHead,
    tip_x: f64,
    tip_y: f64,
    el: &EdgeLayout,
    is_start: bool,
    link_color: &str,
) {
    // Compute direction vector from prev point to tip (or tip to next for start arrows)
    let (dx, dy) = if is_start && el.points.len() >= 2 {
        let (nx, ny) = el.points[1];
        (el.points[0].0 - nx, el.points[0].1 - ny)
    } else if !is_start && el.points.len() >= 2 {
        let n = el.points.len();
        let (px, py) = el.points[n - 1];
        let (prev_x, prev_y) = el.points[n - 2];
        (px - prev_x, py - prev_y)
    } else {
        (0.0, -1.0)
    };

    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let ux = dx / len;
    let uy = dy / len;
    // Perpendicular
    let px = -uy;
    let py = ux;

    match head {
        ArrowHead::None => {}
        ArrowHead::Arrow => {
            // Open arrowhead: two lines forming a V
            // Java format: px,py, px+4,py-10, px,py-6, px-4,py-10, px,py
            let p1x = tip_x;
            let p1y = tip_y;
            let p2x = tip_x - ux * 10.0 + px * 4.0;
            let p2y = tip_y - uy * 10.0 + py * 4.0;
            let p3x = tip_x - ux * 6.0;
            let p3y = tip_y - uy * 6.0;
            let p4x = tip_x - ux * 10.0 - px * 4.0;
            let p4y = tip_y - uy * 10.0 - py * 4.0;
            write!(buf,
                r#"<polygon fill="{link_color}" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{link_color};stroke-width:1;"/>"#,
                fmt_coord(p1x), fmt_coord(p1y),
                fmt_coord(p2x), fmt_coord(p2y),
                fmt_coord(p3x), fmt_coord(p3y),
                fmt_coord(p4x), fmt_coord(p4y),
                fmt_coord(p1x), fmt_coord(p1y),
            ).unwrap();
            buf.push('\n');
        }
        ArrowHead::Triangle => {
            // Filled/hollow triangle arrowhead
            let p1x = tip_x;
            let p1y = tip_y;
            let p2x = tip_x - ux * 10.0 + px * 5.0;
            let p2y = tip_y - uy * 10.0 + py * 5.0;
            let p3x = tip_x - ux * 10.0 - px * 5.0;
            let p3y = tip_y - uy * 10.0 - py * 5.0;
            write!(buf,
                r##"<polygon fill="#F1F1F1" points="{},{},{},{},{},{},{},{}" style="stroke:{link_color};stroke-width:1;"/>"##,
                fmt_coord(p1x), fmt_coord(p1y),
                fmt_coord(p2x), fmt_coord(p2y),
                fmt_coord(p3x), fmt_coord(p3y),
                fmt_coord(p1x), fmt_coord(p1y),
            ).unwrap();
            buf.push('\n');
        }
        ArrowHead::Diamond => {
            // Filled diamond
            let p1x = tip_x;
            let p1y = tip_y;
            let p2x = tip_x - ux * 7.0 + px * 5.0;
            let p2y = tip_y - uy * 7.0 + py * 5.0;
            let p3x = tip_x - ux * 14.0;
            let p3y = tip_y - uy * 14.0;
            let p4x = tip_x - ux * 7.0 - px * 5.0;
            let p4y = tip_y - uy * 7.0 - py * 5.0;
            write!(buf,
                r#"<polygon fill="{link_color}" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{link_color};stroke-width:1;"/>"#,
                fmt_coord(p1x), fmt_coord(p1y),
                fmt_coord(p2x), fmt_coord(p2y),
                fmt_coord(p3x), fmt_coord(p3y),
                fmt_coord(p4x), fmt_coord(p4y),
                fmt_coord(p1x), fmt_coord(p1y),
            ).unwrap();
            buf.push('\n');
        }
        ArrowHead::DiamondHollow => {
            // Hollow diamond
            let p1x = tip_x;
            let p1y = tip_y;
            let p2x = tip_x - ux * 7.0 + px * 5.0;
            let p2y = tip_y - uy * 7.0 + py * 5.0;
            let p3x = tip_x - ux * 14.0;
            let p3y = tip_y - uy * 14.0;
            let p4x = tip_x - ux * 7.0 - px * 5.0;
            let p4y = tip_y - uy * 7.0 - py * 5.0;
            write!(buf,
                r##"<polygon fill="#FFFFFF" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{link_color};stroke-width:1;"/>"##,
                fmt_coord(p1x), fmt_coord(p1y),
                fmt_coord(p2x), fmt_coord(p2y),
                fmt_coord(p3x), fmt_coord(p3y),
                fmt_coord(p4x), fmt_coord(p4y),
                fmt_coord(p1x), fmt_coord(p1y),
            ).unwrap();
            buf.push('\n');
        }
        ArrowHead::Plus => {
            // Circle with plus sign - approximate with a filled polygon
            let cx = tip_x - ux * 6.0;
            let cy = tip_y - uy * 6.0;
            write!(buf,
                r##"<circle cx="{}" cy="{}" fill="#FFFFFF" r="5" style="stroke:{link_color};stroke-width:1;"/>"##,
                fmt_coord(cx), fmt_coord(cy),
            ).unwrap();
            buf.push('\n');
        }
    }
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
        r#"<polygon fill="{bg}" points="{x:.1},{y:.1} {x1:.1},{y:.1} {x2:.1},{y1:.1} {x2:.1},{y2:.1} {x:.1},{y2:.1}" style="stroke:{border};stroke-width:1;"/>"#,
        x1 = x + w - fold,
        y1 = y + fold,
        x2 = x + w,
        y2 = y + h,
        bg = NOTE_BG,
        border = NOTE_BORDER,
    ).unwrap();
    buf.push('\n');

    // fold corner triangle
    write!(buf,
        r#"<path d="M {cx:.1},{cy:.1} L {cx:.1},{cy2:.1} L {cx2:.1},{cy:.1} Z" fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
        cx = x + w - fold,
        cy = y,
        cy2 = y + fold,
        cx2 = x + w,
        bg = NOTE_BG,
        border = NOTE_BORDER,
    ).unwrap();
    buf.push('\n');

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
            r#"<line style="stroke:{border};stroke-width:1;stroke-dasharray:5,3;" x1="{fx:.1}" x2="{tx:.1}" y1="{fy:.1}" y2="{ty:.1}"/>"#,
            fx = from_x + MARGIN,
            fy = from_y + MARGIN,
            tx = to_x + MARGIN,
            ty = to_y + MARGIN,
            border = NOTE_BORDER,
        ).unwrap();
        buf.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::graphviz::{EdgeLayout, GraphLayout, NodeLayout};
    use crate::layout::DiagramLayout;
    use crate::model::{
        ArrowHead, ClassDiagram, Diagram, Direction, Entity, EntityKind, LineStyle, Link, Member,
        MemberModifiers, Visibility,
    };

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
        };
        let entity2 = Entity {
            name: "Bar".into(),
            kind: EntityKind::Interface,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
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
        };
        let cd = ClassDiagram {
            entities: vec![entity, entity2],
            links: vec![link],
            groups: vec![],
            direction: Direction::TopToBottom,
            notes: vec![],
        };
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
        assert!(svg.contains("<polygon"), "arrow should render as inline polygon");
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
        assert_eq!(format_member(&m), "# calc() : int");
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
        };
        let cd = ClassDiagram {
            entities: vec![entity],
            links: vec![],
            groups: vec![],
            direction: Direction::TopToBottom,
            notes: vec![],
        };
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
        };
        let cd = ClassDiagram {
            entities: vec![entity],
            links: vec![],
            groups: vec![],
            direction: Direction::TopToBottom,
            notes: vec![],
        };
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
        };
        let cd = ClassDiagram {
            entities: vec![entity],
            links: vec![],
            groups: vec![],
            direction: Direction::TopToBottom,
            notes: vec![crate::model::ClassNote {
                text: "test note".into(),
                position: "right".into(),
                target: Some("Foo".into()),
            }],
        };
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

        let cd = ClassDiagram {
            entities: vec![],
            links: vec![],
            groups: vec![],
            direction: Direction::TopToBottom,
            notes: vec![crate::model::ClassNote {
                text: "floating".into(),
                position: "right".into(),
                target: None,
            }],
        };
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
