//! Convert SVG sprite content to path-based elements for inline rendering.
//!
//! Java PlantUML converts SVG sprite elements (rect, circle, ellipse, line,
//! polyline, polygon) to `<path>` elements with absolute positioning. SVG
//! `<text>` elements are preserved as `<text>` elements.  Gradients and defs
//! are extracted and re-emitted in the parent SVG `<defs>` block.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;

use crate::klimt::svg::{fmt_coord, xml_escape};

thread_local! {
    static COLLECTED_GRADIENT_DEFS: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
}

pub fn clear_gradient_defs() {
    COLLECTED_GRADIENT_DEFS.with(|g| g.borrow_mut().clear());
}

pub fn take_gradient_defs() -> Vec<(String, String)> {
    COLLECTED_GRADIENT_DEFS.with(|g| std::mem::take(&mut *g.borrow_mut()))
}

/// Information about a sprite's viewBox dimensions.
#[derive(Debug, Clone)]
pub struct SpriteInfo {
    pub vb_width: f64,
    pub vb_height: f64,
}

/// Parse the viewBox from SVG content and return sprite dimensions.
pub fn sprite_info(svg: &str) -> SpriteInfo {
    let (w, h) = parse_viewbox(svg);
    SpriteInfo {
        vb_width: w,
        vb_height: h,
    }
}

/// Compute the gap between text and sprite: the space character width at the given font.
/// Java: the gap equals the space advance from getStringBounds.
pub fn sprite_text_gap(font_family: &str, font_size: f64, bold: bool, italic: bool) -> f64 {
    crate::font_metrics::char_width(' ', font_family, font_size, bold, italic)
}

/// Convert SVG sprite elements to path-based elements with absolute positioning.
///
/// `offset_x` and `offset_y` are added to all coordinates to position the sprite
/// content in the output SVG.  Returns a string containing `<path>`, `<text>`,
/// and other converted elements.
pub fn convert_svg_elements(svg: &str, offset_x: f64, offset_y: f64) -> String {
    let grad_defs = extract_gradient_defs(svg);
    if !grad_defs.is_empty() {
        COLLECTED_GRADIENT_DEFS.with(|collected| {
            let mut collected = collected.borrow_mut();
            for (id, def_xml) in &grad_defs {
                if !collected.iter().any(|(eid, _)| eid == id) {
                    collected.push((id.clone(), def_xml.clone()));
                }
            }
        });
    }
    let mut buf = String::new();
    let inner = strip_svg_wrapper(svg);
    convert_elements(&mut buf, inner.trim(), offset_x, offset_y, None);
    buf
}

/// Extract gradient `<defs>` from SVG content for inclusion in the parent SVG.
///
/// Returns a list of `(id, definition_xml)` pairs.  The caller must emit these
/// inside the root `<defs>` block and update fill references accordingly.
pub fn extract_gradient_defs(svg: &str) -> Vec<(String, String)> {
    let mut defs = Vec::new();
    let inner = strip_svg_wrapper(svg);
    collect_gradient_defs(inner.trim(), &mut defs);
    defs
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Strip the outermost `<svg ...>...</svg>` wrapper, returning inner content.
fn strip_svg_wrapper(svg: &str) -> &str {
    let trimmed = svg.trim();
    // Find end of opening <svg ...> tag
    if let Some(gt) = trimmed.find('>') {
        let after_open = &trimmed[gt + 1..];
        // Remove closing </svg>
        if let Some(close) = after_open.rfind("</svg>") {
            return &after_open[..close];
        }
        return after_open;
    }
    trimmed
}

/// Parse viewBox attribute from SVG content.
fn parse_viewbox(svg: &str) -> (f64, f64) {
    if let Some(vb_start) = svg.find("viewBox=\"") {
        let rest = &svg[vb_start + 9..];
        if let Some(vb_end) = rest.find('"') {
            let vb_str = &rest[..vb_end];
            let parts: Vec<&str> = vb_str.split_whitespace().collect();
            if parts.len() == 4 {
                let w = parts[2].parse::<f64>().unwrap_or(100.0);
                let h = parts[3].parse::<f64>().unwrap_or(50.0);
                return (w, h);
            }
        }
    }
    // Fallback: try width/height attributes
    let w = parse_attr_f64(svg, "width").unwrap_or(100.0);
    let h = parse_attr_f64(svg, "height").unwrap_or(50.0);
    (w, h)
}

fn parse_attr_f64(s: &str, attr: &str) -> Option<f64> {
    let pattern = format!("{attr}=\"");
    if let Some(start) = s.find(&pattern) {
        let rest = &s[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return rest[..end].trim_end_matches("px").parse::<f64>().ok();
        }
    }
    None
}

/// Parse a single XML attribute value from an element string.
fn get_attr<'a>(element: &'a str, attr: &str) -> Option<&'a str> {
    let pattern = format!("{attr}=\"");
    if let Some(start) = element.find(&pattern) {
        let rest = &element[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return Some(&rest[..end]);
        }
    }
    None
}

/// Parse a style attribute and extract a specific property.
fn get_style_prop<'a>(style: &'a str, prop: &str) -> Option<&'a str> {
    let prefix = format!("{prop}:");
    for part in style.split(';') {
        let trimmed = part.trim();
        if let Some(rest) = trimmed.strip_prefix(prefix.as_str()) {
            return Some(rest.trim());
        }
    }
    None
}

/// Collect gradient definitions from the SVG content.
fn collect_gradient_defs(content: &str, defs: &mut Vec<(String, String)>) {
    // Find <defs>...</defs> blocks and extract gradients
    let mut pos = 0;
    while let Some(start) = content[pos..].find("<defs") {
        let abs_start = pos + start;
        if let Some(end) = content[abs_start..].find("</defs>") {
            let defs_content = &content[abs_start..abs_start + end + 7];
            // Extract individual gradient definitions
            extract_gradients_from_defs(defs_content, defs);
            pos = abs_start + end + 7;
        } else {
            break;
        }
    }
}

/// Extract gradient elements from a <defs> block.
fn extract_gradients_from_defs(defs_block: &str, out: &mut Vec<(String, String)>) {
    for tag in &["linearGradient", "radialGradient"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut pos = 0;
        while let Some(start) = defs_block[pos..].find(open.as_str()) {
            let abs_start = pos + start;
            if let Some(end) = defs_block[abs_start..].find(close.as_str()) {
                let grad = &defs_block[abs_start..abs_start + end + close.len()];
                if let Some(id) = get_attr(grad, "id") {
                    out.push((id.to_string(), normalize_gradient(grad, tag)));
                }
                pos = abs_start + end + close.len();
            } else {
                break;
            }
        }
    }
}

/// Normalize gradient XML to match Java's DOM serializer output:
/// - Attribute order: id, x1, x2, y1, y2 (for linear) or id, cx, cy, r, fx, fy (for radial)
/// - Child elements on same line, no extra whitespace
fn normalize_gradient(raw: &str, tag: &str) -> String {
    use std::fmt::Write;
    let mut result = String::new();

    // Build the opening tag with canonical attribute order
    let id = get_attr(raw, "id").unwrap_or("");
    write!(result, "<{tag} id=\"{id}\"").unwrap();
    // Java: spreadMethod (if not "pad") comes before coordinates
    if let Some(sm) = get_attr(raw, "spreadMethod") {
        if sm != "pad" {
            write!(result, " spreadMethod=\"{sm}\"").unwrap();
        }
    }
    if tag == "linearGradient" {
        for attr in &["x1", "x2", "y1", "y2", "gradientUnits", "gradientTransform"] {
            if let Some(v) = get_attr(raw, attr) { write!(result, " {attr}=\"{v}\"").unwrap(); }
        }
    } else {
        for attr in &["cx", "cy", "r", "fx", "fy", "gradientUnits", "gradientTransform"] {
            if let Some(v) = get_attr(raw, attr) { write!(result, " {attr}=\"{v}\"").unwrap(); }
        }
    }
    result.push('>');

    // Extract and append child <stop> elements without extra whitespace
    let close_tag = format!("</{tag}>");
    if let Some(inner_start) = raw.find('>') {
        let inner = &raw[inner_start + 1..raw.len() - close_tag.len()];
        for stop in inner.split("<stop") {
            let s = stop.trim();
            if s.is_empty() || !s.contains("offset") { continue; }
            result.push_str("<stop ");
            // Normalize whitespace: collapse multiple spaces to single space
            let attrs = s.trim_end_matches('>').trim_end_matches('/').trim();
            let normalized: String = attrs.split_whitespace().collect::<Vec<_>>().join(" ");
            result.push_str(&normalized);
            result.push_str("/>");
        }
    }

    result.push_str(&close_tag);
    result
}

/// Recursively convert SVG elements to path-based output.
fn convert_elements(
    buf: &mut String,
    content: &str,
    ox: f64,
    oy: f64,
    parent_transform: Option<&str>,
) {
    convert_elements_inner(buf, content, ox, oy, ox, oy, parent_transform);
}

/// Like convert_elements but with separate text offset.
/// Java: group transforms apply to shapes but NOT to text.
fn convert_elements_with_text_offset(
    buf: &mut String,
    content: &str,
    ox: f64,
    oy: f64,
    text_ox: f64,
    text_oy: f64,
) {
    convert_elements_inner(buf, content, ox, oy, text_ox, text_oy, None);
}

fn convert_elements_inner(
    buf: &mut String,
    content: &str,
    ox: f64,
    oy: f64,
    text_ox: f64,
    text_oy: f64,
    parent_transform: Option<&str>,
) {
    let mut pos = 0;
    let mut iterations = 0;
    while pos < content.len() {
        iterations += 1;
        if iterations > 500 {
            log::warn!("svg_sprite: exceeded 500 iterations at pos={}/{}, aborting", pos, content.len());
            break;
        }
        // Skip whitespace, comments, and non-element text
        if content[pos..].starts_with("<!--") {
            if let Some(end) = content[pos..].find("-->") {
                pos += end + 3;
                continue;
            }
        }

        if content.as_bytes()[pos] != b'<' {
            pos += 1;
            continue;
        }

        // Skip processing instructions, closing tags, defs, style
        if content[pos..].starts_with("</")
            || content[pos..].starts_with("<?")
            || content[pos..].starts_with("<defs")
            || content[pos..].starts_with("<style")
        {
            // Skip to end of tag
            if let Some(end) = content[pos..].find('>') {
                let tag = &content[pos..pos + end + 1];
                // For <defs> and <style>, skip to closing tag
                if tag.starts_with("<defs") && !tag.ends_with("/>") {
                    if let Some(close) = content[pos..].find("</defs>") {
                        pos += close + 7;
                        continue;
                    }
                }
                if tag.starts_with("<style") && !tag.ends_with("/>") {
                    if let Some(close) = content[pos..].find("</style>") {
                        pos += close + 8;
                        continue;
                    }
                }
                pos += end + 1;
            } else {
                pos += 1;
            }
            continue;
        }

        // Try to parse an element
        if let Some((element, consumed)) = parse_element(&content[pos..]) {
            if consumed == 0 {
                // Safety: prevent infinite loop on zero-length parse
                pos += 1;
                continue;
            }
            convert_single_element_ext(buf, &element, ox, oy, text_ox, text_oy, parent_transform);
            pos += consumed;
        } else {
            pos += 1;
        }
    }
}

/// Parse a single XML element (self-closing or with children).
/// Returns (element_text, bytes_consumed).
fn parse_element(s: &str) -> Option<(String, usize)> {
    if !s.starts_with('<') {
        return None;
    }

    // Get tag name
    let tag_name_end = s[1..]
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .map(|i| i + 1)?;
    let tag_name = &s[1..tag_name_end];

    // Self-closing tag: only check for /> before the first >
    let gt = s.find('>')?;
    if gt >= 2 && &s[gt - 1..gt + 1] == "/>" {
        return Some((s[..gt + 1].to_string(), gt + 1));
    }

    // Find end of opening tag
    let gt = s.find('>')?;

    // Self-closing
    if s[..gt].ends_with('/') {
        return Some((s[..gt + 1].to_string(), gt + 1));
    }

    // Find matching closing tag
    let close_tag = format!("</{tag_name}>");
    let mut depth = 1;
    let mut search_pos = gt + 1;
    let mut guard = 0;
    while depth > 0 && search_pos < s.len() {
        guard += 1;
        if guard > 100 || depth > 5 { break; }
        let open_tag = format!("<{tag_name}");
        let next_open = s[search_pos..].find(open_tag.as_str());
        let next_close = s[search_pos..].find(close_tag.as_str());

        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                // Check if it's a real open tag (not just a substring match)
                let after_name = search_pos + o + open_tag.len();
                if after_name < s.len()
                    && (s.as_bytes()[after_name] == b' '
                        || s.as_bytes()[after_name] == b'>'
                        || s.as_bytes()[after_name] == b'/')
                {
                    depth += 1;
                }
                search_pos += o + 1;
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    let end = search_pos + c + close_tag.len();
                    return Some((s[..end].to_string(), end));
                }
                search_pos += c + 1;
            }
            _ => break,
        }
    }

    // Fallback: treat as self-closing
    Some((s[..gt + 1].to_string(), gt + 1))
}

/// Convert a single SVG element to path-based output.
fn convert_single_element(
    buf: &mut String,
    element: &str,
    ox: f64,
    oy: f64,
    _parent_transform: Option<&str>,
) {
    convert_single_element_ext(buf, element, ox, oy, ox, oy, _parent_transform);
}

fn convert_single_element_ext(
    buf: &mut String,
    element: &str,
    ox: f64,
    oy: f64,
    text_ox: f64,
    text_oy: f64,
    _parent_transform: Option<&str>,
) {
    let tag = element_tag_name(element);
    match tag {
        "rect" => convert_rect(buf, element, ox, oy),
        "circle" => convert_circle(buf, element, ox, oy),
        "ellipse" => convert_ellipse(buf, element, ox, oy),
        "line" => convert_line(buf, element, ox, oy),
        "polyline" => convert_polyline(buf, element, ox, oy),
        "polygon" => convert_polygon(buf, element, ox, oy),
        "path" => convert_path(buf, element, ox, oy),
        "text" => convert_text(buf, element, text_ox, text_oy),
        "image" => convert_image(buf, element, ox, oy),
        "g" => convert_group(buf, element, ox, oy, text_ox, text_oy),
        "use" => { /* TODO: use/defs expansion */ }
        _ => {}
    }
}

fn element_tag_name(element: &str) -> &str {
    let s = element.strip_prefix('<').unwrap_or(element);
    let end = s
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(s.len());
    &s[..end]
}

// ── Element converters ──────────────────────────────────────────────────────

fn convert_rect(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let x = get_attr(element, "x")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let y = get_attr(element, "y")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let w = get_attr(element, "width")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let h = get_attr(element, "height")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);

    let ax = x + ox;
    let ay = y + oy;
    let ax2 = ax + w;
    let ay2 = ay + h;

    // Build path: M x,y L x+w,y L x+w,y+h L x,y+h L x,y
    let d = format!(
        "M{},{} L{},{} L{},{} L{},{} L{},{}",
        fmt_coord(ax),
        fmt_coord(ay),
        fmt_coord(ax2),
        fmt_coord(ay),
        fmt_coord(ax2),
        fmt_coord(ay2),
        fmt_coord(ax),
        fmt_coord(ay2),
        fmt_coord(ax),
        fmt_coord(ay),
    );

    let fill = get_fill(element);
    let style = get_stroke_style(element);
    // Java: shapes with gradient fill and no explicit stroke get a default
    // stroke matching the fill gradient (stroke-width:1)
    let style = if style.is_empty() && fill.starts_with("url(") {
        format!("stroke:{fill};stroke-width:1;")
    } else {
        style
    };

    write!(buf, r#"<path d="{d}" fill="{fill}""#).unwrap();
    if !style.is_empty() {
        write!(buf, r#" style="{style}""#).unwrap();
    }
    buf.push_str("/>");
}

fn convert_circle(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let cx = get_attr(element, "cx")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let cy = get_attr(element, "cy")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let r = get_attr(element, "r")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);

    let acx = cx + ox;
    let acy = cy + oy;

    // Circle as 4 arcs: start at left, go top, right, bottom, back to left
    let d = format!(
        "M{},{} A{r},{r} 0 0 1 {},{} A{r},{r} 0 0 1 {},{} A{r},{r} 0 0 1 {},{} A{r},{r} 0 0 1 {},{} L{},{}",
        fmt_coord(acx - r), fmt_coord(acy),       // start: left
        fmt_coord(acx), fmt_coord(acy - r),        // top
        fmt_coord(acx + r), fmt_coord(acy),        // right
        fmt_coord(acx), fmt_coord(acy + r),        // bottom
        fmt_coord(acx - r), fmt_coord(acy),        // back to left
        fmt_coord(acx - r), fmt_coord(acy),        // L close
        r = fmt_coord_raw(r),
    );

    let fill = get_fill(element);
    let style = get_stroke_style(element);

    write!(buf, r#"<path d="{d}" fill="{fill}""#).unwrap();
    if !style.is_empty() {
        write!(buf, r#" style="{style}""#).unwrap();
    }
    buf.push_str("/>");
}

fn convert_ellipse(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let cx = get_attr(element, "cx")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let cy = get_attr(element, "cy")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let rx = get_attr(element, "rx")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let ry = get_attr(element, "ry")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);

    let acx = cx + ox;
    let acy = cy + oy;

    // Ellipse as 4 arcs
    let d = format!(
        "M{},{} A{rx},{ry} 0 0 1 {},{} A{rx},{ry} 0 0 1 {},{} A{rx},{ry} 0 0 1 {},{} A{rx},{ry} 0 0 1 {},{} L{},{}",
        fmt_coord(acx - rx), fmt_coord(acy),
        fmt_coord(acx), fmt_coord(acy - ry),
        fmt_coord(acx + rx), fmt_coord(acy),
        fmt_coord(acx), fmt_coord(acy + ry),
        fmt_coord(acx - rx), fmt_coord(acy),
        fmt_coord(acx - rx), fmt_coord(acy),
        rx = fmt_coord_raw(rx),
        ry = fmt_coord_raw(ry),
    );

    let fill = get_fill(element);
    let style = get_stroke_style(element);

    write!(buf, r#"<path d="{d}" fill="{fill}""#).unwrap();
    if !style.is_empty() {
        write!(buf, r#" style="{style}""#).unwrap();
    }
    buf.push_str("/>");
}

fn convert_line(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let x1 = get_attr(element, "x1")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let y1 = get_attr(element, "y1")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let x2 = get_attr(element, "x2")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let y2 = get_attr(element, "y2")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);

    let d = format!(
        "M{},{} L{},{}",
        fmt_coord(x1 + ox),
        fmt_coord(y1 + oy),
        fmt_coord(x2 + ox),
        fmt_coord(y2 + oy),
    );

    let fill = get_fill_or(element, "#000000");
    let style = get_stroke_style(element);

    write!(buf, r#"<path d="{d}" fill="{fill}""#).unwrap();
    if !style.is_empty() {
        write!(buf, r#" style="{style}""#).unwrap();
    }
    buf.push_str("/>");
}

fn convert_polyline(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let points_str = get_attr(element, "points").unwrap_or("");
    let points = parse_points(points_str, ox, oy);
    if points.is_empty() {
        return;
    }

    let mut d = format!("M{},{}", fmt_coord(points[0].0), fmt_coord(points[0].1));
    for &(px, py) in &points[1..] {
        write!(d, " L{},{}", fmt_coord(px), fmt_coord(py)).unwrap();
    }

    let fill = get_fill_or(element, "none");
    let style = get_stroke_style(element);

    // Java PlantUML uses the original element name as id
    let id = get_attr(element, "id").unwrap_or("polyline");

    write!(buf, r#"<path d="{d}" fill="{fill}" id="{id}""#).unwrap();
    if !style.is_empty() {
        write!(buf, r#" style="{style}""#).unwrap();
    }
    buf.push_str("/>");
}

fn convert_polygon(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let points_str = get_attr(element, "points").unwrap_or("");
    let points = parse_points(points_str, ox, oy);
    if points.is_empty() {
        return;
    }

    let mut d = format!("M{},{}", fmt_coord(points[0].0), fmt_coord(points[0].1));
    for &(px, py) in &points[1..] {
        write!(d, " L{},{}", fmt_coord(px), fmt_coord(py)).unwrap();
    }
    // Close the polygon — Java PlantUML does NOT use Z, it closes by repeating start point
    write!(d, " L{},{}", fmt_coord(points[0].0), fmt_coord(points[0].1)).unwrap();

    let fill = get_fill(element);
    let style = get_stroke_style(element);

    // Java PlantUML uses the original element name as id
    let id = get_attr(element, "id").unwrap_or("polygon");

    write!(buf, r#"<path d="{d}" fill="{fill}" id="{id}""#).unwrap();
    if !style.is_empty() {
        write!(buf, r#" style="{style}""#).unwrap();
    }
    buf.push_str("/>");
}

fn convert_path(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let d = get_attr(element, "d").unwrap_or("");
    let translated = translate_path_data(d, ox, oy);

    let fill = get_fill(element);
    let style = get_stroke_style(element);

    write!(buf, r#"<path d="{translated}" fill="{fill}""#).unwrap();
    if !style.is_empty() {
        write!(buf, r#" style="{style}""#).unwrap();
    }
    buf.push_str("/>");
}

/// Get a text attribute from either a direct attribute or the style property.
fn get_text_attr_or<'a>(element: &'a str, attr: &str, style_prop: &str, default: &'a str) -> &'a str {
    get_attr(element, attr)
        .or_else(|| get_attr(element, "style").and_then(|s| get_style_prop(s, style_prop)))
        .unwrap_or(default)
}

fn convert_text(buf: &mut String, element: &str, ox: f64, oy: f64) {
    // Extract text content
    let inner = extract_element_content(element, "text");

    // Get attributes (check both attribute and style property)
    let x = get_attr(element, "x")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let y = get_attr(element, "y")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let fill = normalize_hex_color(get_text_attr_or(element, "fill", "fill", "#000000"));
    let font_family_raw = get_text_attr_or(element, "font-family", "font-family", "sans-serif");
    // Strip "px" suffix from font-size (CSS style may use "16px")
    let font_size_raw = get_text_attr_or(element, "font-size", "font-size", "14");
    let font_size = font_size_raw.trim_end_matches("px");
    let font_family = font_family_raw;
    let font_weight_str = get_text_attr_or(element, "font-weight", "font-weight", "");
    let font_weight: Option<&str> = if font_weight_str.is_empty() { None } else { Some(font_weight_str) };
    let font_style_str = get_text_attr_or(element, "font-style", "font-style", "");
    let font_style_attr: Option<&str> = if font_style_str.is_empty() { None } else { Some(font_style_str) };
    let td_str = get_text_attr_or(element, "text-decoration", "text-decoration", "");
    let text_decoration: Option<&str> = if td_str.is_empty() { None } else { Some(td_str) };

    // Compute text width using font metrics
    let text_content = inner.trim();
    let size = font_size.parse::<f64>().unwrap_or(14.0);
    let bold = font_weight
        .map(|w| w == "bold" || w == "700" || w == "800" || w == "900")
        .unwrap_or(false);
    let italic = font_style_attr
        .map(|s| s == "italic" || s == "oblique")
        .unwrap_or(false);
    // Java maps "oblique" to "italic" in SVG output
    let font_style_output = font_style_attr.map(|s| if s == "oblique" { "italic" } else { s });
    let text_length =
        crate::font_metrics::text_width(text_content, font_family, size, bold, italic);

    // SVG text-anchor: adjust x position.
    // Java converts "middle" → x - textLength/2, "end" → x - textLength.
    let text_anchor = get_attr(element, "text-anchor").unwrap_or("start");
    let x = match text_anchor {
        "middle" => x - text_length / 2.0,
        "end" => x - text_length,
        _ => x,
    };

    // Java: "monospaced" → "monospace"
    let font_family = if font_family.eq_ignore_ascii_case("monospaced") { "monospace" } else { font_family };
    // Java: replace spaces with non-breaking space (&#160;) for monospace/courier fonts
    let text_output: std::borrow::Cow<str> = if font_family.eq_ignore_ascii_case("monospace") || font_family.eq_ignore_ascii_case("courier") {
        std::borrow::Cow::Owned(text_content.replace(' ', "\u{00A0}"))
    } else {
        std::borrow::Cow::Borrowed(text_content)
    };

    write!(
        buf,
        r#"<text fill="{fill}" font-family="{font_family}" font-size="{font_size}""#,
    )
    .unwrap();
    if let Some(fs) = font_style_output {
        if fs != "normal" {
            write!(buf, r#" font-style="{fs}""#).unwrap();
        }
    }
    if let Some(fw) = font_weight {
        // Java does not output font-weight for normal/400 (default)
        // Java maps "bold" to "700"
        if fw != "normal" && fw != "400" {
            let fw_out = if fw == "bold" { "700" } else { fw };
            write!(buf, r#" font-weight="{fw_out}""#).unwrap();
        }
    }
    write!(buf, r#" lengthAdjust="spacing""#).unwrap();
    if let Some(td) = text_decoration {
        // Java only supports underline and line-through (not overline or none)
        if td == "underline" || td == "line-through" {
            write!(buf, r#" text-decoration="{td}""#).unwrap();
        }
    }
    write!(
        buf,
        r#" textLength="{}" x="{}" y="{}">{}</text>"#,
        fmt_coord(text_length),
        fmt_coord(x + ox),
        fmt_coord(y + oy),
        xml_escape(&text_output),
    )
    .unwrap();
}

fn convert_image(buf: &mut String, element: &str, ox: f64, oy: f64) {
    let x = get_attr(element, "x")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let y = get_attr(element, "y")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let w = get_attr(element, "width")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let h = get_attr(element, "height")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let href = get_attr(element, "xlink:href")
        .or_else(|| get_attr(element, "href"))
        .unwrap_or("");

    write!(
        buf,
        r#"<image height="{}" width="{}" x="{}" xlink:href="{}""#,
        h as u32,
        w as u32,
        fmt_coord(x + ox),
        href,
    )
    .unwrap();
    if y != 0.0 {
        write!(buf, r#" y="{}""#, fmt_coord(y + oy)).unwrap();
    }
    buf.push_str("/>");
}

fn convert_group(buf: &mut String, element: &str, ox: f64, oy: f64, text_ox: f64, text_oy: f64) {
    let inner = extract_element_content(element, "g");
    // Apply transform="translate(x,y)" if present — for shapes only.
    // Java: group transforms are applied to shape coordinates but NOT to text
    // coordinates. Text retains its original SVG position + sprite base offset.
    let (tx, ty) = if let Some(transform) = get_attr(element, "transform") {
        parse_translate(&transform)
    } else {
        (0.0, 0.0)
    };
    convert_elements_with_text_offset(buf, inner.trim(), ox + tx, oy + ty, text_ox, text_oy);
}

fn parse_translate(transform: &str) -> (f64, f64) {
    if let Some(start) = transform.find("translate(") {
        let rest = &transform[start + 10..];
        if let Some(end) = rest.find(')') {
            let coords = &rest[..end];
            let parts: Vec<&str> = coords.split(',').collect();
            if parts.len() == 2 {
                let x = parts[0].trim().parse::<f64>().unwrap_or(0.0);
                let y = parts[1].trim().parse::<f64>().unwrap_or(0.0);
                return (x, y);
            }
            // Try space separator
            let parts: Vec<&str> = coords.split_whitespace().collect();
            if parts.len() == 2 {
                let x = parts[0].parse::<f64>().unwrap_or(0.0);
                let y = parts[1].parse::<f64>().unwrap_or(0.0);
                return (x, y);
            }
        }
    }
    (0.0, 0.0)
}

// ── Attribute helpers ───────────────────────────────────────────────────────

/// Normalize hex color to uppercase.  Java DOM serializes all hex colors in
/// uppercase (#RRGGBB). Expands 3-digit hex to 6-digit. Pass-through non-hex
/// values like "none" or "url(#id)".
fn normalize_hex_color(s: &str) -> String {
    if let Some(hex) = s.strip_prefix('#') {
        if hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let upper = hex.to_ascii_uppercase();
            if upper.len() == 3 {
                // Expand #RGB → #RRGGBB
                let mut expanded = String::with_capacity(7);
                expanded.push('#');
                for c in upper.chars() {
                    expanded.push(c);
                    expanded.push(c);
                }
                return expanded;
            }
            return format!("#{}", upper);
        }
    }
    s.to_string()
}

fn get_fill(element: &str) -> String {
    get_fill_or(element, "#000000")
}

fn get_fill_or(element: &str, default: &str) -> String {
    // Check fill attribute
    if let Some(fill) = get_attr(element, "fill") {
        return normalize_hex_color(fill);
    }
    // Check style attribute for fill
    if let Some(style) = get_attr(element, "style") {
        if let Some(fill) = get_style_prop(style, "fill") {
            return normalize_hex_color(fill);
        }
    }
    default.to_string()
}

fn get_stroke_style(element: &str) -> String {
    let mut parts = Vec::new();

    // Collect stroke properties from attributes
    let stroke = get_attr(element, "stroke")
        .or_else(|| {
            get_attr(element, "style")
                .and_then(|s| get_style_prop(s, "stroke"))
        });
    let stroke_width = get_attr(element, "stroke-width")
        .or_else(|| {
            get_attr(element, "style")
                .and_then(|s| get_style_prop(s, "stroke-width"))
        });
    let stroke_dasharray = get_attr(element, "stroke-dasharray")
        .or_else(|| {
            get_attr(element, "style")
                .and_then(|s| get_style_prop(s, "stroke-dasharray"))
        });

    if let Some(s) = stroke {
        parts.push(format!("stroke:{};", normalize_hex_color(s)));
    }
    if let Some(sw) = stroke_width {
        parts.push(format!("stroke-width:{sw};"));
    }
    if let Some(sd) = stroke_dasharray {
        parts.push(format!("stroke-dasharray:{sd};"));
    }

    parts.join("")
}

fn parse_points(s: &str, ox: f64, oy: f64) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let cleaned = s.replace(',', " ");
    let nums: Vec<f64> = cleaned
        .split_whitespace()
        .filter_map(|n| n.parse::<f64>().ok())
        .collect();

    for pair in nums.chunks(2) {
        if pair.len() == 2 {
            points.push((pair[0] + ox, pair[1] + oy));
        }
    }
    points
}

/// Translate path data by adding offsets to absolute coordinates.
/// This is a simplified translator that handles common path commands.
fn translate_path_data(d: &str, ox: f64, oy: f64) -> String {
    let mut result = String::new();
    let mut chars = d.chars().peekable();
    let mut current_cmd = ' ';

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        let c = *chars.peek().unwrap();
        if c.is_alphabetic() {
            current_cmd = c;
            chars.next();
            if !result.is_empty() {
                result.push(' ');
            }
            result.push(current_cmd);
        }

        // Parse numbers based on command type
        match current_cmd {
            'M' | 'L' | 'T' => {
                // Absolute move/line: translate x,y
                if let Some((x, y)) = parse_coord_pair(&mut chars) {
                    write!(result, "{},{}", fmt_coord(x + ox), fmt_coord(y + oy)).unwrap();
                }
            }
            'A' => {
                // Arc: rx,ry x-rotation large-arc sweep x,y
                // Only translate the endpoint x,y
                if let Some(rx) = parse_number(&mut chars) {
                    skip_comma(&mut chars);
                    if let Some(ry) = parse_number(&mut chars) {
                        skip_whitespace_comma(&mut chars);
                        if let Some(rot) = parse_number(&mut chars) {
                            skip_whitespace_comma(&mut chars);
                            if let Some(large) = parse_number(&mut chars) {
                                skip_whitespace_comma(&mut chars);
                                if let Some(sweep) = parse_number(&mut chars) {
                                    skip_whitespace_comma(&mut chars);
                                    if let Some((x, y)) = parse_coord_pair(&mut chars) {
                                        write!(
                                            result,
                                            "{},{} {} {} {} {},{}",
                                            fmt_coord_raw(rx),
                                            fmt_coord_raw(ry),
                                            rot as i32,
                                            large as i32,
                                            sweep as i32,
                                            fmt_coord(x + ox),
                                            fmt_coord(y + oy),
                                        )
                                        .unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            'C' => {
                // Cubic bezier: x1,y1 x2,y2 x,y
                for _ in 0..3 {
                    if let Some((x, y)) = parse_coord_pair(&mut chars) {
                        write!(result, " {},{}", fmt_coord(x + ox), fmt_coord(y + oy)).unwrap();
                    }
                }
            }
            'Z' | 'z' => {
                // Close path
            }
            _ => {
                // Unknown command, try to skip
                if let Some(_) = parse_number(&mut chars) {
                    // consumed
                }
            }
        }
    }

    result
}

fn parse_coord_pair(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<(f64, f64)> {
    skip_whitespace_comma(chars);
    let x = parse_number(chars)?;
    skip_comma(chars);
    let y = parse_number(chars)?;
    Some((x, y))
}

fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<f64> {
    skip_whitespace_comma(chars);
    let mut s = String::new();
    // Optional sign
    if chars.peek().map_or(false, |&c| c == '-' || c == '+') {
        s.push(chars.next().unwrap());
    }
    // Digits and decimal point
    while chars.peek().map_or(false, |&c| c.is_ascii_digit() || c == '.') {
        s.push(chars.next().unwrap());
    }
    if s.is_empty() || s == "-" || s == "+" {
        return None;
    }
    s.parse::<f64>().ok()
}

fn skip_comma(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while chars.peek().map_or(false, |&c| c == ',' || c.is_whitespace()) {
        chars.next();
    }
}

fn skip_whitespace_comma(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while chars.peek().map_or(false, |&c| c.is_whitespace() || c == ',') {
        chars.next();
    }
}

fn extract_element_content<'a>(element: &'a str, tag: &str) -> &'a str {
    let close = format!("</{tag}>");
    if let Some(gt) = element.find('>') {
        let after = &element[gt + 1..];
        if let Some(close_pos) = after.rfind(close.as_str()) {
            return &after[..close_pos];
        }
        return after;
    }
    ""
}

/// Format a raw f64 without trailing zeros (for use in arc radii etc.)
fn fmt_coord_raw(v: f64) -> String {
    if v == v.floor() {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.4}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_to_path() {
        let mut buf = String::new();
        let elem = "<rect x=\"10\" y=\"10\" width=\"80\" height=\"30\" fill=\"#FF0000\" stroke=\"#000000\" stroke-width=\"2\"/>";
        convert_rect(&mut buf, elem, 71.3804, 50.2969);
        assert!(buf.contains("<path"));
        assert!(buf.contains("fill=\"#FF0000\""));
        assert!(buf.contains("stroke:#000000"));
    }

    #[test]
    fn test_circle_to_path() {
        let mut buf = String::new();
        let elem = "<circle cx=\"18\" cy=\"18\" r=\"10\" fill=\"#FF0000\"/>";
        convert_circle(&mut buf, elem, 71.3804, 50.2969);
        assert!(buf.contains("<path"));
        assert!(buf.contains("A10,10"));
        assert!(buf.contains("fill=\"#FF0000\""));
    }

    #[test]
    fn test_line_to_path() {
        let mut buf = String::new();
        let elem = "<line x1=\"0\" y1=\"2\" x2=\"100\" y2=\"2\" stroke=\"#FF0000\" stroke-width=\"4\"/>";
        convert_line(&mut buf, elem, 71.3804, 61.4297);
        assert!(buf.contains("<path"));
        assert!(buf.contains("stroke:#FF0000"));
    }

    #[test]
    fn test_strip_svg_wrapper() {
        let svg = r#"<svg viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#;
        assert_eq!(strip_svg_wrapper(svg), "<rect/>");
    }

    #[test]
    fn test_viewbox_parse() {
        let info = sprite_info(
            r#"<svg viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#,
        );
        assert_eq!(info.vb_width, 100.0);
        assert_eq!(info.vb_height, 50.0);
    }
}
