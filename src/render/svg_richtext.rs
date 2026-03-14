use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;

use crate::font_metrics;
use crate::model::hyperlink::Hyperlink;
use crate::model::richtext::{RichText, TextSpan};
use crate::parser::creole::parse_creole;
use crate::render::svg::{fmt_coord, xml_escape};
use crate::render::svg_hyperlink::wrap_with_link;

thread_local! {
    static SVG_SPRITES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static DEFAULT_FONT_FAMILY: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Set the sprite registry for the current rendering pass.
pub fn set_sprites(sprites: HashMap<String, String>) {
    SVG_SPRITES.with(|s| *s.borrow_mut() = sprites);
}

/// Clear the sprite registry after rendering.
pub fn clear_sprites() {
    SVG_SPRITES.with(|s| s.borrow_mut().clear());
}

/// Override the default font family for all subsequent `render_creole_text` calls.
pub fn set_default_font_family(family: Option<String>) {
    DEFAULT_FONT_FAMILY.with(|f| *f.borrow_mut() = family);
}

/// Get the current default font family (or "sans-serif").
fn get_default_font_family() -> String {
    DEFAULT_FONT_FAMILY.with(|f| {
        f.borrow()
            .clone()
            .unwrap_or_else(|| "sans-serif".to_string())
    })
}

fn get_sprite(name: &str) -> Option<String> {
    SVG_SPRITES.with(|s| s.borrow().get(name).cloned())
}

#[derive(Clone, Default)]
struct SpanStyle {
    font_weight: Option<&'static str>,
    font_style: Option<&'static str>,
    font_family: Option<&'static str>,
    font_family_owned: Option<String>,
    font_size: Option<f64>,
    font_size_em: Option<&'static str>,
    baseline_shift: Option<&'static str>,
    fill: Option<String>,
    background: Option<String>,
    decorations: Vec<&'static str>,
}

impl SpanStyle {
    fn with_decoration(mut self, decoration: &'static str) -> Self {
        if !self.decorations.contains(&decoration) {
            self.decorations.push(decoration);
        }
        self
    }
}

pub fn count_creole_lines(text: &str) -> usize {
    flatten_rich_lines(&parse_creole(text)).len().max(1)
}

pub fn max_creole_plain_line_len(text: &str) -> usize {
    flatten_plain_lines(&parse_creole(text))
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
pub fn render_creole_text(
    buf: &mut String,
    text: &str,
    x: f64,
    y: f64,
    line_height: f64,
    fill: &str,
    text_anchor: Option<&str>,
    outer_attrs: &str,
) -> usize {
    let lines = flatten_rich_lines(&parse_creole(text));
    let lines = if lines.is_empty() {
        vec![vec![TextSpan::Plain(String::new())]]
    } else {
        lines
    };

    // Collect sprite references from all lines for deferred rendering.
    let sprite_refs: Vec<(String, Option<String>)> = lines
        .iter()
        .flat_map(|line| {
            line.iter().filter_map(|span| {
                if let TextSpan::InlineSvg { name } = span {
                    Some((name.clone(), get_sprite(name)))
                } else {
                    None
                }
            })
        })
        .collect();

    // Compute textLength for the <text> element.
    let plain = lines
        .iter()
        .map(|line| plain_text_spans(line))
        .collect::<Vec<_>>()
        .join("");
    let (font_family, font_size, bold, italic) = parse_font_props(outer_attrs);
    let text_length =
        font_metrics::text_width(&plain, &font_family, font_size, bold, italic);

    if lines.len() == 1 {
        write_text_open(buf, x, y, fill, text_anchor, outer_attrs, text_length);
        if let Some(text) = simple_plain_line(&lines[0]) {
            buf.push_str(&xml_escape(text));
        } else {
            render_spans(buf, &lines[0], &SpanStyle::default(), fill);
        }
        buf.push_str("</text>\n");
        render_deferred_sprites(buf, &sprite_refs, x, y);
        return 1;
    }

    write_text_open(buf, x, y, fill, text_anchor, outer_attrs, text_length);
    for (idx, line) in lines.iter().enumerate() {
        let dy = if idx == 0 { 0.0 } else { line_height };
        write!(buf, r#"<tspan x="{x:.1}" dy="{dy:.1}">"#).unwrap();
        if let Some(text) = simple_plain_line(line) {
            buf.push_str(&xml_escape(text));
        } else {
            render_spans(buf, line, &SpanStyle::default(), fill);
        }
        buf.push_str("</tspan>");
    }
    buf.push_str("</text>\n");
    render_deferred_sprites(buf, &sprite_refs, x, y);

    lines.len()
}

/// Parse font properties from `outer_attrs` for `textLength` computation.
///
/// Returns `(font_family, font_size, bold, italic)`.
fn parse_font_props(outer_attrs: &str) -> (String, f64, bool, bool) {
    let mut font_family = get_default_font_family();
    let mut font_size = 14.0_f64;
    let mut bold = false;
    let mut italic = false;

    let mut remaining = outer_attrs.trim();
    while !remaining.is_empty() {
        if let Some(eq_pos) = remaining.find('=') {
            let attr_name = remaining[..eq_pos].trim();
            let after_eq = &remaining[eq_pos + 1..];
            if let Some(stripped) = after_eq.strip_prefix('"') {
                if let Some(end_quote) = stripped.find('"') {
                    let value = &stripped[..end_quote];
                    match attr_name {
                        "font-size" => {
                            font_size = value.parse::<f64>().unwrap_or(14.0);
                        }
                        "font-weight" => {
                            bold = value == "bold";
                        }
                        "font-style" => {
                            italic = value == "italic";
                        }
                        "font-family" => {
                            font_family = value.to_string();
                        }
                        _ => {}
                    }
                    remaining = remaining[eq_pos + 1 + end_quote + 2..].trim_start();
                    continue;
                }
            }
        }
        break;
    }
    (font_family, font_size, bold, italic)
}

/// Write the opening `<text ...>` tag with attributes in Java PlantUML
/// alphabetical order: fill, font-family, font-size, font-style, font-weight,
/// lengthAdjust, text-anchor, text-decoration, textLength, x, y.
///
/// `outer_attrs` may contain additional attributes such as `font-size="14"`,
/// `font-weight="bold"`, or `font-style="italic"`.  They are parsed and merged
/// into the correct positions.
fn write_text_open(
    buf: &mut String,
    x: f64,
    y: f64,
    fill: &str,
    text_anchor: Option<&str>,
    outer_attrs: &str,
    text_length: f64,
) {
    // Parse outer_attrs into key=value pairs for ordered insertion
    let mut font_size_attr: Option<&str> = None;
    let mut font_style_attr: Option<&str> = None;
    let mut font_weight_attr: Option<&str> = None;
    let mut text_decoration_attr: Option<&str> = None;
    let mut extra_attrs = Vec::new();

    if !outer_attrs.is_empty() {
        // Simple attribute parser: split on space before attr names
        let mut remaining = outer_attrs.trim();
        while !remaining.is_empty() {
            if let Some(eq_pos) = remaining.find('=') {
                let attr_name = remaining[..eq_pos].trim();
                let after_eq = &remaining[eq_pos + 1..];
                // Find the quoted value
                if let Some(stripped) = after_eq.strip_prefix('"') {
                    if let Some(end_quote) = stripped.find('"') {
                        let value_with_quotes = &remaining[eq_pos + 1..eq_pos + 1 + end_quote + 2];
                        match attr_name {
                            "font-size" => font_size_attr = Some(value_with_quotes),
                            "font-style" => font_style_attr = Some(value_with_quotes),
                            "font-weight" => font_weight_attr = Some(value_with_quotes),
                            "text-decoration" => text_decoration_attr = Some(value_with_quotes),
                            _ => extra_attrs.push((attr_name, value_with_quotes)),
                        }
                        remaining = remaining[eq_pos + 1 + end_quote + 2..].trim_start();
                        continue;
                    }
                }
            }
            // If parsing fails, just append as-is and break
            extra_attrs.push((outer_attrs, ""));
            break;
        }
    }

    // Alphabetical order: fill, font-family, font-size, font-style, font-weight,
    // text-anchor, text-decoration, x, y
    write!(buf, r#"<text fill="{}""#, xml_escape(fill)).unwrap();
    let default_font = get_default_font_family();
    write!(buf, r#" font-family="{}""#, xml_escape(&default_font)).unwrap();
    if let Some(fs) = font_size_attr {
        write!(buf, r#" font-size={fs}"#).unwrap();
    }
    if let Some(fst) = font_style_attr {
        write!(buf, r#" font-style={fst}"#).unwrap();
    }
    if let Some(fw) = font_weight_attr {
        write!(buf, r#" font-weight={fw}"#).unwrap();
    }
    write!(buf, r#" lengthAdjust="spacing""#).unwrap();
    if let Some(anchor) = text_anchor {
        write!(buf, r#" text-anchor="{}""#, xml_escape(anchor)).unwrap();
    }
    if let Some(td) = text_decoration_attr {
        write!(buf, r#" text-decoration={td}"#).unwrap();
    }
    write!(buf, r#" textLength="{}""#, fmt_coord(text_length)).unwrap();
    // Any unknown extra attrs
    for (name, value) in &extra_attrs {
        if value.is_empty() {
            write!(buf, " {name}").unwrap();
        } else {
            write!(buf, " {name}={value}").unwrap();
        }
    }
    write!(buf, r#" x="{x:.1}" y="{y:.1}">"#).unwrap();
}

fn flatten_rich_lines(rich: &RichText) -> Vec<Vec<TextSpan>> {
    let mut out = Vec::new();
    flatten_rich_lines_into(rich, &mut out);
    out
}

fn flatten_rich_lines_into(rich: &RichText, out: &mut Vec<Vec<TextSpan>>) {
    match rich {
        RichText::Line(spans) => out.push(spans.clone()),
        RichText::Block(items) => {
            for item in items {
                flatten_rich_lines_into(item, out);
            }
        }
        RichText::BulletList(items) => {
            for item in items {
                let mut lines = flatten_rich_lines(item);
                prefix_first_line(&mut lines, "- ");
                out.extend(lines);
            }
        }
        RichText::NumberedList(items) => {
            for (idx, item) in items.iter().enumerate() {
                let mut lines = flatten_rich_lines(item);
                prefix_first_line(&mut lines, &format!("{}. ", idx + 1));
                out.extend(lines);
            }
        }
        RichText::Table { headers, rows } => {
            if !headers.is_empty() {
                out.push(join_cells(headers));
            }
            for row in rows {
                out.push(join_cells(row));
            }
        }
        RichText::HorizontalRule => out.push(vec![TextSpan::Plain("----".to_string())]),
    }
}

fn flatten_plain_lines(rich: &RichText) -> Vec<String> {
    flatten_rich_lines(rich)
        .into_iter()
        .map(|line| plain_text_spans(&line))
        .collect()
}

fn prefix_first_line(lines: &mut Vec<Vec<TextSpan>>, prefix: &str) {
    if lines.is_empty() {
        lines.push(vec![TextSpan::Plain(prefix.to_string())]);
        return;
    }
    lines[0].insert(0, TextSpan::Plain(prefix.to_string()));
}

fn join_cells(cells: &[Vec<TextSpan>]) -> Vec<TextSpan> {
    let mut line = Vec::new();
    for (idx, cell) in cells.iter().enumerate() {
        if idx > 0 {
            line.push(TextSpan::Plain(" | ".to_string()));
        }
        line.extend(cell.clone());
    }
    line
}

fn plain_text_spans(spans: &[TextSpan]) -> String {
    let mut out = String::new();
    for span in spans {
        collect_plain_span(span, &mut out);
    }
    out
}

fn collect_plain_span(span: &TextSpan, out: &mut String) {
    match span {
        TextSpan::Plain(text) | TextSpan::Monospace(text) => out.push_str(text),
        TextSpan::Bold(inner)
        | TextSpan::Italic(inner)
        | TextSpan::Underline(inner)
        | TextSpan::Strikethrough(inner)
        | TextSpan::Subscript(inner)
        | TextSpan::Superscript(inner) => {
            for inner_span in inner {
                collect_plain_span(inner_span, out);
            }
        }
        TextSpan::Colored { content, .. }
        | TextSpan::Sized { content, .. }
        | TextSpan::BackHighlight { content, .. }
        | TextSpan::FontFamily { content, .. } => {
            for inner_span in content {
                collect_plain_span(inner_span, out);
            }
        }
        TextSpan::Link { url, label, .. } => {
            if let Some(label) = label {
                out.push_str(label);
            } else {
                out.push_str(url);
            }
        }
        TextSpan::InlineSvg { .. } => {}
    }
}

fn render_spans(buf: &mut String, spans: &[TextSpan], style: &SpanStyle, default_fill: &str) {
    for span in spans {
        render_span(buf, span, style.clone(), default_fill);
    }
}

fn simple_plain_line(spans: &[TextSpan]) -> Option<&str> {
    if spans.len() == 1 {
        if let TextSpan::Plain(text) = &spans[0] {
            return Some(text);
        }
    }
    None
}

fn render_span(buf: &mut String, span: &TextSpan, style: SpanStyle, default_fill: &str) {
    match span {
        TextSpan::Plain(text) => render_leaf(buf, text, None, &style, default_fill),
        TextSpan::Monospace(text) => {
            let mut style = style;
            style.font_family = Some("monospace");
            render_leaf(buf, text, None, &style, default_fill);
        }
        TextSpan::Bold(inner) => {
            let mut style = style;
            style.font_weight = Some("bold");
            render_spans(buf, inner, &style, default_fill);
        }
        TextSpan::Italic(inner) => {
            let mut style = style;
            style.font_style = Some("italic");
            render_spans(buf, inner, &style, default_fill);
        }
        TextSpan::Underline(inner) => {
            render_spans(
                buf,
                inner,
                &style.with_decoration("underline"),
                default_fill,
            );
        }
        TextSpan::Strikethrough(inner) => {
            render_spans(
                buf,
                inner,
                &style.with_decoration("line-through"),
                default_fill,
            );
        }
        TextSpan::Colored { color, content } => {
            let mut style = style;
            style.fill = Some(color.clone());
            render_spans(buf, content, &style, default_fill);
        }
        TextSpan::Sized { size, content } => {
            let mut style = style;
            style.font_size = Some(*size);
            render_spans(buf, content, &style, default_fill);
        }
        TextSpan::Subscript(inner) => {
            let mut style = style;
            style.font_size_em = Some("0.7em");
            style.baseline_shift = Some("sub");
            render_spans(buf, inner, &style, default_fill);
        }
        TextSpan::Superscript(inner) => {
            let mut style = style;
            style.font_size_em = Some("0.7em");
            style.baseline_shift = Some("super");
            render_spans(buf, inner, &style, default_fill);
        }
        TextSpan::BackHighlight { color, content } => {
            let mut style = style;
            style.background = Some(color.clone());
            render_spans(buf, content, &style, default_fill);
        }
        TextSpan::FontFamily { family, content } => {
            let mut style = style;
            style.font_family_owned = Some(family.clone());
            render_spans(buf, content, &style, default_fill);
        }
        TextSpan::Link {
            url,
            tooltip,
            label,
        } => {
            let visible = label.as_deref().unwrap_or(url.as_str());
            let link = Hyperlink {
                url: url.clone(),
                tooltip: tooltip.clone(),
                label: label.clone(),
            };
            render_leaf(buf, visible, Some(&link), &style, default_fill);
        }
        TextSpan::InlineSvg { .. } => {
            // Sprite SVGs are rendered after the <text> element, not inline.
            // See render_deferred_sprites() called from render_creole_text().
        }
    }
}

fn render_leaf(
    buf: &mut String,
    text: &str,
    link: Option<&Hyperlink>,
    style: &SpanStyle,
    default_fill: &str,
) {
    let escaped = xml_escape(text);
    let attrs = style_attrs(style, default_fill);
    let leaf = if attrs.is_empty() {
        format!("<tspan>{escaped}</tspan>")
    } else {
        format!(r"<tspan{attrs}>{escaped}</tspan>")
    };
    if let Some(link) = link {
        buf.push_str(&wrap_with_link(&leaf, link));
    } else {
        buf.push_str(&leaf);
    }
}

fn style_attrs(style: &SpanStyle, default_fill: &str) -> String {
    let mut attrs = String::new();
    if let Some(font_weight) = style.font_weight {
        write!(attrs, r#" font-weight="{font_weight}""#).unwrap();
    }
    if let Some(font_style) = style.font_style {
        write!(attrs, r#" font-style="{font_style}""#).unwrap();
    }
    if let Some(ref family) = style.font_family_owned {
        write!(attrs, r#" font-family="{}""#, xml_escape(family)).unwrap();
    } else if let Some(font_family) = style.font_family {
        write!(attrs, r#" font-family="{font_family}""#).unwrap();
    }
    if let Some(font_size_em) = style.font_size_em {
        write!(attrs, r#" font-size="{font_size_em}""#).unwrap();
    } else if let Some(font_size) = style.font_size {
        write!(attrs, r#" font-size="{font_size:.1}""#).unwrap();
    }
    if let Some(baseline_shift) = style.baseline_shift {
        write!(attrs, r#" baseline-shift="{baseline_shift}""#).unwrap();
    }
    if let Some(fill) = &style.fill {
        if fill != default_fill {
            write!(attrs, r#" fill="{}""#, xml_escape(fill)).unwrap();
        }
    }
    if let Some(ref bg) = style.background {
        write!(attrs, r#" background-color="{}""#, xml_escape(bg)).unwrap();
    }
    if !style.decorations.is_empty() {
        write!(
            attrs,
            r#" text-decoration="{}""#,
            style.decorations.join(" ")
        )
        .unwrap();
    }
    attrs
}

/// Render deferred inline SVG sprites after the `<text>` element.
///
/// Each sprite is rendered as a `<g>` element positioned relative to the
/// text anchor, with the SVG content embedded directly.
fn render_deferred_sprites(
    buf: &mut String,
    sprite_refs: &[(String, Option<String>)],
    x: f64,
    y: f64,
) {
    let mut offset_x = 0.0;
    for (_name, svg_content) in sprite_refs {
        if let Some(svg) = svg_content {
            // Parse viewBox to determine sprite dimensions for scaling
            let (vb_w, vb_h) = parse_viewbox(svg);
            let display_h = 16.0_f64; // Match line height
            let scale = if vb_h > 0.0 { display_h / vb_h } else { 1.0 };
            let display_w = vb_w * scale;
            let sprite_x = x + offset_x;
            let sprite_y = y - display_h;
            writeln!(
                buf,
                r#"<g transform="translate({sprite_x:.1},{sprite_y:.1}) scale({scale:.4})">{svg}</g>"#
            )
            .unwrap();
            offset_x += display_w + 4.0;
        }
    }
}

/// Parse `viewBox` attribute from an SVG element to extract width and height.
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
    let w = parse_svg_attr(svg, "width").unwrap_or(100.0);
    let h = parse_svg_attr(svg, "height").unwrap_or(50.0);
    (w, h)
}

fn parse_svg_attr(svg: &str, attr: &str) -> Option<f64> {
    let pattern = format!("{attr}=\"");
    if let Some(start) = svg.find(&pattern) {
        let rest = &svg[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return rest[..end].trim_end_matches("px").parse::<f64>().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_bold_and_italic_spans() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "**bold** //italic//",
            10.0,
            20.0,
            16.0,
            "#000000",
            Some("middle"),
            "",
        );
        assert!(buf.contains(r#"font-weight="bold""#));
        assert!(buf.contains(r#"font-style="italic""#));
        assert!(buf.contains(r#"text-anchor="middle""#));
    }

    #[test]
    fn renders_multiple_lines() {
        let mut buf = String::new();
        let lines = render_creole_text(
            &mut buf,
            "line1\\nline2",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );
        assert_eq!(lines, 2);
        assert_eq!(buf.matches("<text ").count(), 1);
        assert_eq!(buf.matches("<tspan").count(), 2);
    }

    #[test]
    fn renders_link_with_tooltip() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "[[https://example.com{hover} Example]]",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );
        assert!(buf.contains(r#"href="https://example.com""#));
        assert!(buf.contains("<title>hover</title>"));
        assert!(buf.contains("Example"));
    }

    #[test]
    fn plain_line_metrics_strip_markup() {
        assert_eq!(count_creole_lines("a\\nb"), 2);
        assert_eq!(max_creole_plain_line_len("**abc**"), 3);
    }

    #[test]
    fn renders_subscript() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "H<sub>2</sub>O",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );
        assert!(buf.contains(r#"font-size="0.7em""#));
        assert!(buf.contains(r#"baseline-shift="sub""#));
        assert!(buf.contains(">2<"));
    }

    #[test]
    fn renders_superscript() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "E = mc<sup>2</sup>",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );
        assert!(buf.contains(r#"font-size="0.7em""#));
        assert!(buf.contains(r#"baseline-shift="super""#));
    }

    #[test]
    fn renders_back_highlight() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "<back:yellow>important</back>",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );
        assert!(buf.contains(r#"background-color="yellow""#));
        assert!(buf.contains("important"));
    }

    #[test]
    fn renders_font_family() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "<font:courier>code</font>",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );
        assert!(buf.contains(r#"font-family="courier""#));
        assert!(buf.contains("code"));
    }

    #[test]
    fn renders_inline_svg_sprite() {
        let mut sprites = HashMap::new();
        sprites.insert(
            "test".to_string(),
            r#"<svg viewBox="0 0 100 50"><rect fill="red"/></svg>"#.to_string(),
        );
        set_sprites(sprites);

        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "before <$test> after",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );

        assert!(buf.contains("before"), "text before sprite");
        assert!(buf.contains("after"), "text after sprite");
        assert!(
            buf.contains(r#"fill="red""#),
            "sprite SVG content must be embedded"
        );
        assert!(buf.contains("<g transform="), "sprite must be in a group");

        clear_sprites();
    }

    #[test]
    fn renders_text_without_sprites_unchanged() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "plain text",
            10.0,
            20.0,
            16.0,
            "#000000",
            None,
            "",
        );
        assert!(buf.contains("plain text"));
        assert!(!buf.contains("<g transform="));
    }

    #[test]
    fn parse_viewbox_basic() {
        assert_eq!(
            parse_viewbox(r#"<svg viewBox="0 0 200 100"><rect/></svg>"#),
            (200.0, 100.0)
        );
    }
}
