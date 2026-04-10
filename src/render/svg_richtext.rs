use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape, xml_escape_attr};
use crate::model::hyperlink::Hyperlink;
use crate::model::richtext::{RichText, TextSpan};
use crate::parser::creole::{parse_creole, parse_creole_opts, parse_inline};
use crate::render::svg_hyperlink::wrap_with_link;

use crate::klimt::color::resolve_color;
use crate::parser::common::SpriteGrayData;

thread_local! {
    static SVG_SPRITES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static SPRITE_GRAY: RefCell<HashMap<String, SpriteGrayData>> = RefCell::new(HashMap::new());
    static DEFAULT_FONT_FAMILY: RefCell<Option<String>> = const { RefCell::new(None) };
    static PATH_BASED_SPRITES: RefCell<bool> = const { RefCell::new(false) };
    static BACK_FILTERS: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
    /// Stencil + stroke for Creole section titles (`==title==`).
    /// Caller installs this before invoking a `render_creole_text*` call
    /// on content that may contain a section title, so the renderer knows
    /// how wide to draw the decorating horizontal lines and which stroke
    /// color to use.  Cleared via `clear_section_title_bounds`.
    static SECTION_TITLE_BOUNDS: RefCell<Option<SectionTitleBounds>> = const { RefCell::new(None) };
}

/// Horizontal-line bounds and stroke for rendering a Creole section title
/// (`==text==`) inside a richtext block.  `x_start`/`x_end` span the full
/// decorating line range (Java `UHorizontalLine` stencil).
#[derive(Debug, Clone)]
pub struct SectionTitleBounds {
    pub x_start: f64,
    pub x_end: f64,
    pub stroke: String,
}

/// Install the stencil used by the section-title renderer for the next
/// `render_creole_text*` call.  Must be followed by
/// `clear_section_title_bounds` once the rendering pass is complete, so
/// later callers are not accidentally picked up.
pub fn set_section_title_bounds(bounds: SectionTitleBounds) {
    SECTION_TITLE_BOUNDS.with(|b| *b.borrow_mut() = Some(bounds));
}

/// Drop any previously installed section-title stencil.
pub fn clear_section_title_bounds() {
    SECTION_TITLE_BOUNDS.with(|b| *b.borrow_mut() = None);
}

fn current_section_title_bounds() -> Option<SectionTitleBounds> {
    SECTION_TITLE_BOUNDS.with(|b| b.borrow().clone())
}

/// Set the sprite registry for the current rendering pass.
pub fn set_sprites(sprites: HashMap<String, String>) {
    SVG_SPRITES.with(|s| *s.borrow_mut() = sprites);
}

/// Set the sprite gray data registry for context-dependent PNG re-generation.
pub fn set_sprite_gray_data(data: HashMap<String, SpriteGrayData>) {
    SPRITE_GRAY.with(|s| *s.borrow_mut() = data);
}

/// Clear the sprite registry after rendering.
pub fn clear_sprites() {
    SVG_SPRITES.with(|s| s.borrow_mut().clear());
    SPRITE_GRAY.with(|s| s.borrow_mut().clear());
    PATH_BASED_SPRITES.with(|p| *p.borrow_mut() = false);
    BACK_FILTERS.with(|f| f.borrow_mut().clear());
}

pub fn take_back_filters() -> Vec<(String, String)> {
    BACK_FILTERS.with(|f| std::mem::take(&mut *f.borrow_mut()))
}

fn register_back_filter(color: &str) -> String {
    use crate::style::normalize_color;
    let hex_color = normalize_color(color);
    BACK_FILTERS.with(|f| {
        let mut filters = f.borrow_mut();
        if let Some((id, _)) = filters.iter().find(|(_, existing)| existing == &hex_color) {
            return id.clone();
        }

        let id = format!(
            "{}{}",
            crate::klimt::svg::current_filter_uid_prefix(),
            filters.len()
        );
        filters.push((id.clone(), hex_color));
        id
    })
}

pub fn enable_path_sprites() {
    PATH_BASED_SPRITES.with(|p| *p.borrow_mut() = true);
}

pub fn disable_path_sprites() {
    PATH_BASED_SPRITES.with(|p| *p.borrow_mut() = false);
}

fn is_path_sprites_enabled() -> bool {
    PATH_BASED_SPRITES.with(|p| *p.borrow())
}

/// Process a link title like Java's `SvgGraphics.LinkData.getXlinkTitle()`.
/// 1. Decode `<U+XXXX>` Unicode escapes to actual characters
/// 2. Replace literal `\n` with newline character
fn process_xlink_title(title: &str) -> String {
    // Step 1: Decode <U+XXXX> patterns
    let mut result = String::with_capacity(title.len());
    let mut rest = title;
    while !rest.is_empty() {
        if let Some(start) = rest.find("<U+") {
            result.push_str(&rest[..start]);
            let after = &rest[start + 3..];
            if let Some(end) = after.find('>') {
                let hex = &after[..end];
                if let Ok(code) = u32::from_str_radix(hex, 16) {
                    if let Some(ch) = char::from_u32(code) {
                        result.push(ch);
                        rest = &after[end + 1..];
                        continue;
                    }
                }
                // Failed to decode — keep literal
                result.push_str(&rest[..start + 3 + end + 1]);
                rest = &after[end + 1..];
            } else {
                result.push_str(rest);
                break;
            }
        } else {
            result.push_str(rest);
            break;
        }
    }
    // Step 2: Replace backslash-n sequences with newline.
    // Java `SvgGraphics.LinkData.getXlinkTitle()` uses
    // `replaceAll("\\\\n", "\n")` — i.e. the regex matches the literal
    // 2-char sequence `\n` and replaces it with a newline. Backslashes
    // themselves are preserved, so `\\n` becomes `\` + newline (the first
    // backslash stays literal, the trailing `\n` becomes a newline).
    // Do NOT collapse `\\n` to a single newline — that would drop the
    // user-visible escape backslash that Java keeps in the output.
    result.replace("\\n", "\n")
}

/// Override the default font family for all subsequent `render_creole_text` calls.
pub fn set_default_font_family(family: Option<String>) {
    DEFAULT_FONT_FAMILY.with(|f| *f.borrow_mut() = family);
}

/// Get the current default font family (or "sans-serif") — public accessor for sibling modules.
pub fn get_default_font_family_pub() -> String {
    get_default_font_family()
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

pub fn get_sprite_svg(name: &str) -> Option<String> {
    get_sprite(name)
}

/// Get a sprite data URI with a custom background color.
///
/// If the sprite has raw gray data, re-generates the PNG with the given background.
/// Otherwise falls back to extracting the data URI from the pre-generated SVG.
pub fn get_sprite_data_uri_with_bg(name: &str, bg_r: u8, bg_g: u8, bg_b: u8) -> Option<String> {
    // Try gray data first (hex-encoded monochrome sprites)
    let from_gray = SPRITE_GRAY.with(|s| {
        s.borrow()
            .get(name)
            .and_then(|data| crate::parser::common::sprite_gray_to_data_uri(data, bg_r, bg_g, bg_b))
    });
    if from_gray.is_some() {
        return from_gray;
    }
    // Fallback: extract data URI from pre-generated SVG
    get_sprite(name).and_then(|svg| {
        let href_start = svg.find("xlink:href=\"")? + "xlink:href=\"".len();
        let href_end = svg[href_start..].find('"')? + href_start;
        Some(svg[href_start..href_end].to_string())
    })
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
    /// Ambient font size inherited from the enclosing `<text>` element.
    /// Used to resolve heading-sentinel `Sized` spans (`size = -100 - delta`)
    /// into their effective absolute size (`ambient + delta`).
    ambient_font_size: Option<f64>,
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

pub fn creole_plain_text(text: &str) -> String {
    flatten_plain_lines(&parse_creole(text)).join("")
}

/// Compute table width when title text contains a creole table.
///
/// Returns `Some(width)` if the text is a table, `None` otherwise.
/// Used by the meta-wrapping code to calculate the correct title width
/// (raw text width over-counts since cells are laid out in columns).
pub fn creole_table_width(text: &str, font_size: f64, bold: bool) -> Option<f64> {
    let lines: Vec<String> = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.lines())
        .map(|s| s.to_string())
        .collect();
    // Check if any line is a table row (starts with |, possibly with color prefix)
    if !lines.iter().any(|line| is_table_display_line(line)) {
        return None;
    }
    // parse_display_table_rows handles stripping <#color> prefixes internally
    let rows = parse_display_table_rows(&lines, false);
    if rows.is_empty() {
        return None;
    }
    let default_font = get_default_font_family();
    let layout = layout_display_table(&rows, &default_font, font_size, bold, false);
    Some(layout.width)
}

/// Compute the effective line height for creole text, considering `<size:N>` markup
/// and `<sub>`/`<sup>` elements that extend the vertical bounds.
/// Java: `TextBlock.calculateDimension().getHeight()` uses the largest font in the display.
pub fn creole_line_height(text: &str, default_font: &str, default_font_size: f64) -> f64 {
    let max_size = max_font_size_in_creole(text, default_font_size);
    let base_h = font_metrics::line_height(default_font, max_size, false, false);
    // Check for sub/sup which adds extra vertical space
    let parsed = parse_creole(text);
    let lines = flatten_rich_lines(&parsed);
    let has_sub = lines.iter().any(|line| line.iter().any(has_subscript));
    let has_sup = lines.iter().any(|line| line.iter().any(has_superscript));
    let mut extra = 0.0_f64;
    let sub_size = (default_font_size * 0.77).round();
    if has_sub {
        // Subscript extends below: shift + descent(sub) - descent(base)
        let sub_shift = default_font_size * 0.2852;
        let desc_sub = font_metrics::descent(default_font, sub_size, false, false);
        let desc_base = font_metrics::descent(default_font, default_font_size, false, false);
        let below_extra = (sub_shift + desc_sub - desc_base).max(0.0);
        extra = extra.max(below_extra);
    }
    if has_sup {
        // Superscript extends above: shift + ascent(sup) - ascent(base)
        let sup_shift = default_font_size * 0.4071;
        let asc_sup = font_metrics::ascent(default_font, sub_size, false, false);
        let asc_base = font_metrics::ascent(default_font, default_font_size, false, false);
        let above_extra = (sup_shift + asc_sup - asc_base).max(0.0);
        extra = extra.max(above_extra);
    }
    base_h + extra
}

/// Compute the extra height below baseline from `<sub>` elements only.
/// This is used to shift the text baseline up in the renderer.
/// Superscript extends above and does NOT shift the text baseline.
pub fn creole_sub_extra_height(text: &str, default_font: &str, default_font_size: f64) -> f64 {
    let parsed = parse_creole(text);
    let lines = flatten_rich_lines(&parsed);
    let has_sub = lines.iter().any(|line| line.iter().any(has_subscript));
    if has_sub {
        let sub_size = (default_font_size * 0.77).round();
        let sub_shift = default_font_size * 0.2852;
        let desc_sub = font_metrics::descent(default_font, sub_size, false, false);
        let desc_base = font_metrics::descent(default_font, default_font_size, false, false);
        (sub_shift + desc_sub - desc_base).max(0.0)
    } else {
        0.0
    }
}

/// Find the maximum font size used in a creole text string.
fn max_font_size_in_creole(text: &str, default_font_size: f64) -> f64 {
    let parsed = parse_creole(text);
    let lines = flatten_rich_lines(&parsed);
    let mut max_size = default_font_size;
    for line in &lines {
        for span in line {
            max_font_size_in_span(span, &mut max_size);
        }
    }
    max_size
}

fn max_font_size_in_span(span: &TextSpan, max_size: &mut f64) {
    match span {
        TextSpan::Sized { size, content } => {
            // Heading sentinel: size = -100 - delta → effective = current max + delta.
            if *size <= -100.0 {
                let delta = -100.0 - *size;
                *max_size += delta;
            } else if *size > *max_size {
                *max_size = *size;
            }
            for inner in content {
                max_font_size_in_span(inner, max_size);
            }
        }
        TextSpan::Bold(inner)
        | TextSpan::Italic(inner)
        | TextSpan::Underline(inner)
        | TextSpan::Strikethrough(inner)
        | TextSpan::Subscript(inner)
        | TextSpan::Superscript(inner) => {
            for s in inner {
                max_font_size_in_span(s, max_size);
            }
        }
        TextSpan::UnderlineColored { content, .. }
        | TextSpan::Colored { content, .. }
        | TextSpan::BackHighlight { content, .. }
        | TextSpan::FontFamily { content, .. } => {
            for s in content {
                max_font_size_in_span(s, max_size);
            }
        }
        TextSpan::Plain(_)
        | TextSpan::Monospace(_)
        | TextSpan::Link { .. }
        | TextSpan::InlineSvg { .. } => {}
    }
}

fn has_subscript(span: &TextSpan) -> bool {
    match span {
        TextSpan::Subscript(_) => true,
        TextSpan::Bold(inner)
        | TextSpan::Italic(inner)
        | TextSpan::Underline(inner)
        | TextSpan::Strikethrough(inner)
        | TextSpan::Superscript(inner) => inner.iter().any(has_subscript),
        TextSpan::Colored { content, .. }
        | TextSpan::BackHighlight { content, .. }
        | TextSpan::FontFamily { content, .. }
        | TextSpan::Sized { content, .. } => content.iter().any(has_subscript),
        _ => false,
    }
}

fn has_superscript(span: &TextSpan) -> bool {
    match span {
        TextSpan::Superscript(_) => true,
        TextSpan::Bold(inner)
        | TextSpan::Italic(inner)
        | TextSpan::Underline(inner)
        | TextSpan::Strikethrough(inner)
        | TextSpan::Subscript(inner) => inner.iter().any(has_superscript),
        TextSpan::Colored { content, .. }
        | TextSpan::BackHighlight { content, .. }
        | TextSpan::FontFamily { content, .. }
        | TextSpan::Sized { content, .. } => content.iter().any(has_superscript),
        _ => false,
    }
}

/// Compute the total width of creole text, respecting font-family changes.
/// For text without font-family markup, this behaves like measuring plain text.
/// For text with `<font:family>`, each segment is measured in its own font.
pub fn creole_text_width(
    text: &str,
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> f64 {
    creole_text_width_opts(text, default_font, font_size, bold, italic, false)
}

/// Like `creole_text_width` but treats literal `\n` as displayable text.
pub fn creole_text_width_preserve_newline(
    text: &str,
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> f64 {
    creole_text_width_opts(text, default_font, font_size, bold, italic, true)
}

/// Internal: creole_text_width with `preserve_backslash_n` option.
fn creole_text_width_opts(
    text: &str,
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
    preserve_backslash_n: bool,
) -> f64 {
    let lines = flatten_rich_lines(&parse_creole_opts(text, preserve_backslash_n));
    if lines.is_empty() {
        return 0.0;
    }
    measure_line_width(&lines[0], default_font, font_size, bold, italic)
}

pub fn creole_max_line_width(
    text: &str,
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> f64 {
    flatten_rich_lines(&parse_creole(text))
        .iter()
        .map(|line| measure_line_width(line, default_font, font_size, bold, italic))
        .fold(0.0, f64::max)
}

fn measure_line_width(
    spans: &[TextSpan],
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> f64 {
    let gap = crate::render::svg_sprite::sprite_text_gap(default_font, font_size, bold, italic);

    let mut total = 0.0;
    let mut pending_gap = false;
    let mut text_buf: Vec<TextSpan> = Vec::new();

    for span in spans {
        match span {
            TextSpan::InlineSvg { name, scale, .. } => {
                if let Some(TextSpan::Plain(text)) = text_buf.last_mut() {
                    *text = text.trim_end().to_string();
                }
                let text_w =
                    measure_text_runs_width(&text_buf, default_font, font_size, bold, italic);
                if text_w > 0.0 {
                    total += text_w;
                    pending_gap = true;
                }
                text_buf.clear();

                if let Some(sprite_w) = sprite_display_width(name, *scale, font_size) {
                    if pending_gap {
                        total += gap;
                    }
                    total += sprite_w;
                    pending_gap = true;
                }
            }
            other => {
                let adjusted = if pending_gap && text_buf.is_empty() {
                    match other {
                        TextSpan::Plain(text) => {
                            let trimmed = text.trim_start();
                            if trimmed.is_empty() {
                                continue;
                            }
                            total += gap;
                            pending_gap = false;
                            TextSpan::Plain(trimmed.to_string())
                        }
                        _ => {
                            total += gap;
                            pending_gap = false;
                            other.clone()
                        }
                    }
                } else {
                    other.clone()
                };
                text_buf.push(adjusted);
            }
        }
    }

    total + measure_text_runs_width(&text_buf, default_font, font_size, bold, italic)
}

fn measure_text_runs_width(
    spans: &[TextSpan],
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> f64 {
    if spans.is_empty() {
        return 0.0;
    }
    // Java's `StringBounder.calculateDimension(font, text)` measures the raw
    // text including leading whitespace, so layout widths (used to size
    // boxes and arrows) must match. Rendering separately trims leading
    // spaces and shifts x accordingly.
    if !line_needs_split_render(spans) {
        let plain = plain_text_spans(spans);
        let plain = plain.trim_end();
        return font_metrics::text_width(plain, default_font, font_size, bold, italic);
    }
    // Styled text: measure each run with its own font/style
    let runs = flatten_to_runs(spans);
    let mut total = 0.0;
    let mut first = true;
    for run in &runs {
        let text = if !first {
            run.text.trim_start()
        } else {
            run.text.as_str()
        };
        if text.is_empty() {
            first = false;
            continue;
        }
        // Add space gap if we trimmed leading whitespace
        if !first && text.len() < run.text.len() {
            let n_spaces = run.text.len() - text.len();
            total += font_metrics::text_width(" ", default_font, font_size, false, false)
                * n_spaces as f64;
        }
        let run_font = run.font_family.as_deref().unwrap_or(default_font);
        let run_bold = run.bold || bold;
        let run_italic = run.italic || italic;
        let run_size = match run.font_size_override {
            Some(v) if v == -1.0 || v == -2.0 => (font_size * 0.77).round(),
            // Heading sentinel: size = -100 - delta → effective = base + delta.
            Some(v) if v <= -100.0 => font_size + (-100.0 - v),
            Some(v) if v > 0.0 => v,
            _ => font_size,
        };
        total += font_metrics::text_width(text, run_font, run_size, run_bold, run_italic);
        first = false;
    }
    total
}

fn sprite_scale_for_font(font_size: f64, scale: Option<f64>) -> f64 {
    scale.unwrap_or(1.0) * font_size / 13.0
}

fn sprite_display_width(name: &str, scale: Option<f64>, font_size: f64) -> Option<f64> {
    let svg = get_sprite(name)?;
    let info = crate::render::svg_sprite::sprite_info(&svg);
    Some(info.vb_width * sprite_scale_for_font(font_size, scale))
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
    render_creole_text_opts(
        buf,
        text,
        x,
        y,
        line_height,
        fill,
        text_anchor,
        outer_attrs,
        false,
    )
}

/// Like `render_creole_text` but with `preserve_backslash_n` option.
/// When true, literal `\n` in the text is treated as displayable text, not a line break.
pub fn render_creole_text_opts(
    buf: &mut String,
    text: &str,
    x: f64,
    y: f64,
    line_height: f64,
    fill: &str,
    text_anchor: Option<&str>,
    outer_attrs: &str,
    preserve_backslash_n: bool,
) -> usize {
    let parsed = parse_creole_opts(text, preserve_backslash_n);
    let lines = flatten_rich_lines(&parsed);
    let kinds = flatten_rich_display_lines(&parsed);
    let (lines, kinds) = if lines.is_empty() {
        (
            vec![vec![TextSpan::Plain(String::new())]],
            vec![DisplayLineKind::Text],
        )
    } else {
        (lines, kinds)
    };

    let has_sprites = lines.iter().any(|line| line_has_sprites(line));

    // Path-based sprite rendering for sequence diagrams
    if has_sprites && is_path_sprites_enabled() && lines.len() == 1 {
        return render_line_with_sprites(buf, &lines[0], x, y, fill, outer_attrs);
    }

    let (font_family, font_size, bold, italic) = parse_font_props(outer_attrs);

    // Section title rendered as horizontal lines + centered title text.
    // Only activated when the caller installed section-title bounds via
    // `set_section_title_bounds`; otherwise it falls back to plain text.
    if lines.len() == 1 && kinds[0] == DisplayLineKind::SectionTitle {
        if let Some(bounds) = current_section_title_bounds() {
            render_section_title_line(
                buf,
                &lines[0],
                y,
                line_height,
                fill,
                outer_attrs,
                &font_family,
                font_size,
                bold,
                italic,
                &bounds,
            );
            return 1;
        }
    }

    // Split rendering: each styled span becomes a separate <text> element.
    // This matches Java's DriverTextSvg which renders each atom separately.
    // Exception: centered text (text_anchor="middle") stays as single element
    // because split rendering would center each piece independently.
    if lines.len() == 1 && text_anchor.is_none() && line_needs_split_render(&lines[0]) {
        render_split_text_runs(
            buf,
            &lines[0],
            x,
            y,
            fill,
            outer_attrs,
            &font_family,
            font_size,
            bold,
            italic,
        );
        return 1;
    }

    // Compute textLength for the <text> element.
    if lines.len() == 1 && line_has_sprites(&lines[0]) && text_anchor.is_none() {
        render_text_line_with_sprites(
            buf,
            &lines[0],
            x,
            y,
            fill,
            outer_attrs,
            &font_family,
            font_size,
            bold,
            italic,
        );
        return 1;
    }

    if lines.len() == 1 {
        render_prepared_line(
            buf,
            &lines[0],
            x,
            y,
            fill,
            text_anchor,
            outer_attrs,
            &font_family,
            font_size,
            bold,
            italic,
        );
        return 1;
    }

    // Java advances the baseline by the actual line box height. When a line
    // contains an inline sprite, that line box grows to the sprite height
    // instead of the nominal text line height.
    let section_bounds = current_section_title_bounds();
    let mut line_y = y;
    for (idx, line) in lines.iter().enumerate() {
        let kind = kinds.get(idx).copied().unwrap_or(DisplayLineKind::Text);
        if kind == DisplayLineKind::SectionTitle {
            if let Some(ref bounds) = section_bounds {
                render_section_title_line(
                    buf,
                    line,
                    line_y,
                    line_height,
                    fill,
                    outer_attrs,
                    &font_family,
                    font_size,
                    bold,
                    italic,
                    bounds,
                );
                line_y += line_height_with_inline_sprites(line, font_size, line_height);
                continue;
            }
        }
        if text_anchor.is_none() && line_has_sprites(line) {
            render_text_line_with_sprites(
                buf,
                line,
                x,
                line_y,
                fill,
                outer_attrs,
                &font_family,
                font_size,
                bold,
                italic,
            );
        } else if text_anchor.is_none() && multiline_line_needs_split_render(line) {
            render_split_text_runs(
                buf,
                line,
                x,
                line_y,
                fill,
                outer_attrs,
                &font_family,
                font_size,
                bold,
                italic,
            );
        } else {
            render_prepared_line(
                buf,
                line,
                x,
                line_y,
                fill,
                text_anchor,
                outer_attrs,
                &font_family,
                font_size,
                bold,
                italic,
            );
        }
        line_y += line_height_with_inline_sprites(line, font_size, line_height);
    }

    lines.len()
}

/// Render a single line of text in word-by-word mode, matching Java's DriverTextSvg
/// behavior when `MaximumWidth` is set. Each word and each inter-word space
/// becomes its own `<text>` SVG element.
///
/// Returns the number of lines rendered (always 1 for a single line).
pub fn render_creole_text_word_by_word(
    buf: &mut String,
    text: &str,
    x: f64,
    y: f64,
    _line_height: f64,
    fill: &str,
    outer_attrs: &str,
) -> usize {
    let lines = flatten_rich_lines(&parse_creole(text));
    let line = if lines.is_empty() {
        vec![TextSpan::Plain(String::new())]
    } else {
        // word-by-word mode processes only one line at a time
        lines
            .into_iter()
            .next()
            .unwrap_or_else(|| vec![TextSpan::Plain(String::new())])
    };

    let (default_font, font_size, base_bold, base_italic) = parse_font_props(outer_attrs);
    let runs = flatten_to_runs(&line);

    let mut cursor_x = x;

    for run in &runs {
        let run_font = run.font_family.as_deref().unwrap_or(&default_font);
        let run_bold = run.bold || base_bold;
        let run_italic = run.italic || base_italic;
        let run_size = match run.font_size_override {
            Some(v) if v == -1.0 => (font_size * 0.77).round(), // subscript
            Some(v) if v == -2.0 => (font_size * 0.77).round(), // superscript
            // Heading sentinel: size = -100 - delta → effective = base + delta.
            Some(v) if v <= -100.0 => font_size + (-100.0 - v),
            Some(v) if v > 0.0 => v,
            _ => font_size,
        };
        let run_fill_normalized;
        let run_fill = if let Some(ref c) = run.color {
            run_fill_normalized = crate::style::normalize_color(c);
            &run_fill_normalized
        } else {
            fill
        };

        // Compute space width using the run's actual font properties (bold changes advance).
        let run_space_w = font_metrics::text_width(" ", run_font, run_size, run_bold, run_italic);

        // Split the run text into words and spaces
        let mut chars = run.text.chars().peekable();
        let mut pieces: Vec<(String, bool)> = Vec::new(); // (text, is_space)
        while chars.peek().is_some() {
            if chars.peek() == Some(&' ') {
                let mut spaces = String::new();
                while chars.peek() == Some(&' ') {
                    spaces.push(chars.next().unwrap());
                }
                pieces.push((spaces, true));
            } else {
                let mut word = String::new();
                while chars.peek().is_some() && chars.peek() != Some(&' ') {
                    word.push(chars.next().unwrap());
                }
                pieces.push((word, false));
            }
        }

        for (piece, is_space) in &pieces {
            if *is_space {
                // Render spaces as &#160; elements
                let n_spaces = piece.len();
                let total_w = run_space_w * n_spaces as f64;
                let nbsp = "\u{00A0}".repeat(n_spaces);
                write!(buf, r#"<text fill="{}""#, xml_escape(run_fill)).unwrap();
                if let Some(ref fid) = run.filter_id {
                    write!(buf, r#" filter="url(#{fid})""#).unwrap();
                }
                write!(buf, r#" font-family="{}""#, xml_escape(run_font)).unwrap();
                write!(buf, r#" font-size="{}""#, fmt_coord(run_size)).unwrap();
                if run_italic {
                    buf.push_str(r#" font-style="italic""#);
                }
                if run_bold {
                    buf.push_str(r#" font-weight="bold""#);
                }
                write!(buf, r#" lengthAdjust="spacing""#).unwrap();
                write!(buf, r#" textLength="{}""#, fmt_coord(total_w)).unwrap();
                write!(buf, r#" x="{}" y="{}">"#, fmt_coord(cursor_x), fmt_coord(y)).unwrap();
                buf.push_str(&xml_escape(&nbsp));
                buf.push_str("</text>");
                cursor_x += total_w;
            } else {
                // Render word
                let word_w =
                    font_metrics::text_width(piece, run_font, run_size, run_bold, run_italic);
                if let Some(ref url) = run.link_url {
                    let title_src = run.link_tooltip.as_deref().unwrap_or(url);
                    let title = process_xlink_title(title_src);
                    write!(buf, r#"<a href="{}" target="_top" title="{}" xlink:actuate="onRequest" xlink:href="{}" xlink:show="new" xlink:title="{}" xlink:type="simple">"#,
                        xml_escape_attr(url), xml_escape_attr(&title), xml_escape_attr(url), xml_escape_attr(&title)).unwrap();
                }
                write!(buf, r#"<text fill="{}""#, xml_escape(run_fill)).unwrap();
                if let Some(ref fid) = run.filter_id {
                    write!(buf, r#" filter="url(#{fid})""#).unwrap();
                }
                write!(buf, r#" font-family="{}""#, xml_escape(run_font)).unwrap();
                write!(buf, r#" font-size="{}""#, fmt_coord(run_size)).unwrap();
                if run_italic {
                    buf.push_str(r#" font-style="italic""#);
                }
                if run_bold {
                    buf.push_str(r#" font-weight="bold""#);
                }
                write!(buf, r#" lengthAdjust="spacing""#).unwrap();
                if run.strikethrough {
                    buf.push_str(r#" text-decoration="wavy underline""#);
                } else if run.underline {
                    buf.push_str(r#" text-decoration="underline""#);
                }
                write!(buf, r#" textLength="{}""#, fmt_coord(word_w)).unwrap();
                write!(buf, r#" x="{}" y="{}">"#, fmt_coord(cursor_x), fmt_coord(y)).unwrap();
                buf.push_str(&xml_escape(piece));
                buf.push_str("</text>");
                if run.link_url.is_some() {
                    buf.push_str("</a>");
                }
                cursor_x += word_w;
            }
        }
    }

    1
}

#[derive(Clone)]
struct DisplayTableCell {
    lines: Vec<Vec<TextSpan>>,
    is_header: bool,
    /// Cell background color from `<#color>` prefix
    bg_color: Option<String>,
    leading_spaces: usize,
    trailing_spaces: usize,
}

#[derive(Clone)]
struct DisplayTableRow {
    cells: Vec<DisplayTableCell>,
    /// Row background color from leading `<#color>` prefix on the row line
    bg_color: Option<String>,
    /// Row border color from `<#bg,#border>` prefix
    border_color: Option<String>,
}

struct DisplayTableLayout {
    col_widths: Vec<f64>,
    row_heights: Vec<f64>,
    width: f64,
    total_height: f64,
}

enum DisplayBlock {
    Text(Vec<String>),
    Table(Vec<DisplayTableRow>),
}

const TABLE_MARGIN_Y: f64 = 2.0;
const NBSP: &str = "\u{00A0}";

pub fn measure_creole_display_lines(
    lines: &[String],
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
    preserve_backslash_n: bool,
) -> (f64, f64) {
    let blocks = build_display_blocks(lines, preserve_backslash_n);
    let mut width = 0.0_f64;
    let mut height = 0.0_f64;

    for block in &blocks {
        match block {
            DisplayBlock::Text(text_lines) => {
                for line in text_lines {
                    width = width.max(creole_text_width_opts(
                        line,
                        default_font,
                        font_size,
                        bold,
                        italic,
                        preserve_backslash_n,
                    ));
                    height += creole_line_height(line, default_font, font_size);
                }
            }
            DisplayBlock::Table(rows) => {
                let layout = layout_display_table(rows, default_font, font_size, bold, italic);
                width = width.max(layout.width);
                height += layout.total_height;
            }
        }
    }

    if blocks.is_empty() {
        (
            0.0,
            font_metrics::line_height(default_font, font_size, bold, italic),
        )
    } else {
        (width, height)
    }
}

pub fn render_creole_display_lines(
    buf: &mut String,
    lines: &[String],
    x: f64,
    top_y: f64,
    fill: &str,
    outer_attrs: &str,
    preserve_backslash_n: bool,
) {
    let blocks = build_display_blocks(lines, preserve_backslash_n);
    let (default_font, font_size, bold, italic) = parse_font_props(outer_attrs);
    let mut cursor_y = top_y;

    for block in &blocks {
        match block {
            DisplayBlock::Text(text_lines) => {
                for line in text_lines {
                    let line_height = creole_line_height(line, &default_font, font_size);
                    let ascent = font_metrics::ascent(&default_font, font_size, bold, italic);
                    render_creole_text_opts(
                        buf,
                        line,
                        x,
                        cursor_y + ascent,
                        line_height,
                        fill,
                        None,
                        outer_attrs,
                        preserve_backslash_n,
                    );
                    cursor_y += line_height;
                }
            }
            DisplayBlock::Table(rows) => {
                let layout = layout_display_table(rows, &default_font, font_size, bold, italic);
                render_display_table(
                    buf,
                    rows,
                    &layout,
                    x,
                    cursor_y,
                    fill,
                    font_size,
                    &default_font,
                    bold,
                    italic,
                );
                cursor_y += layout.total_height;
            }
        }
    }
}

/// Render creole note content with proper block-level elements.
///
/// Java's BodyEnhanced2 splits note text at `----`/`====` separators.
/// Each segment is rendered independently; `----`/`====` produce a horizontal
/// rule line but add NO height (they draw on top of the segment boundary).
///
/// Within each segment, content is categorised:
/// - `* text` → bullet list with ellipse marker + indented text
/// - `|...|` → table with grid lines
/// - anything else → regular text line (inline creole)
///
/// Returns the rendered content height (textBlock.h in Java terms).
pub fn render_creole_note_content(
    buf: &mut String,
    text: &str,
    note_x: f64,
    note_y: f64,
    note_width: f64,
    fill: &str,
    font_size: f64,
    border_color: &str,
) -> f64 {
    let margin_x = 6.0; // Java marginX1
    let margin_y = 5.0; // Java marginY
    let text_x = note_x + margin_x;
    let default_font = "SansSerif";
    let lh = font_metrics::line_height(default_font, font_size, false, false);
    let ascent = font_metrics::ascent(default_font, font_size, false, false);
    let descent = font_metrics::descent(default_font, font_size, false, false);
    let outer_attrs = format!(r#"font-size="{font_size}""#);

    // Split text into raw lines (NEWLINE_CHAR, real newlines, and \n escape)
    let raw_lines: Vec<&str> = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.split('\n'))
        .flat_map(|s| s.split("\\n"))
        .collect();

    // Split at block separators (----/====)
    let mut segments: Vec<Vec<&str>> = Vec::new();
    let mut separator_positions: Vec<usize> = Vec::new(); // segment index where HR is drawn
    let mut current_segment: Vec<&str> = Vec::new();

    for line in &raw_lines {
        let trimmed = line.trim();
        if is_block_separator(trimmed) {
            segments.push(std::mem::take(&mut current_segment));
            separator_positions.push(segments.len()); // HR before next segment
        } else {
            current_segment.push(line);
        }
    }
    segments.push(current_segment);

    let mut cursor_y = note_y + margin_y; // top of textBlock in absolute coords

    for (seg_idx, segment) in segments.iter().enumerate() {
        // Draw HR separator if this segment follows a "----"/"===="
        if separator_positions.contains(&seg_idx) {
            // HR line at current cursor_y (= bottom of previous segment)
            let hr_x1 = fmt_coord(note_x + 1.0);
            let hr_x2 = fmt_coord(note_x + note_width - 1.0);
            let hr_y = fmt_coord(cursor_y);
            write!(
                buf,
                r#"<line style="stroke:{border_color};stroke-width:1;" x1="{hr_x1}" x2="{hr_x2}" y1="{hr_y}" y2="{hr_y}"/>"#,
            )
            .unwrap();
            // Decoration margin: 4px top + 4px bottom around the next block content
            // Java: TextBlockUtils.withMargin(block, 0, 4) adds 4 top + 4 bottom
            cursor_y += 4.0; // top margin of decorated block
        }

        // Parse segment into note blocks
        let blocks = parse_note_segment(segment);

        for block in &blocks {
            match block {
                NoteBlock::TextLines(lines) => {
                    for line_text in lines {
                        let effective_lh = note_line_height_with_sprites(line_text, font_size, lh);
                        // Java SeaSheet aligns text to the row's bottom: baseline
                        // = row_top + row_h - descent. For text-only rows this
                        // reduces to row_top + ascent (since row_h = ascent + descent),
                        // so single-line text rendering is unchanged.
                        let baseline = cursor_y + effective_lh - descent;
                        let mut tmp = String::new();
                        render_creole_text(
                            &mut tmp,
                            line_text,
                            text_x,
                            baseline,
                            lh,
                            fill,
                            None,
                            &outer_attrs,
                        );
                        buf.push_str(&tmp);
                        cursor_y += effective_lh;
                    }
                }
                NoteBlock::BulletItems(items) => {
                    for item_text in items {
                        let baseline = cursor_y + ascent;
                        // Bullet ellipse: Java Bullet(order=0) draws at dx=3, UEllipse(5,5)
                        // Sea.doAlign positions bullet at y = lh - bullet_h - |startingAltitude|
                        // = lh - 5 - 5 = lh - 10.  Ellipse cy = bullet_y + 2.5 = lh - 7.5
                        let ecx = text_x + 3.0 + 2.5; // cx = text_x + 5.5
                        let ecy = cursor_y + lh - 7.5; // center from Sea alignment
                        write!(
                            buf,
                            r#"<ellipse cx="{}" cy="{}" fill="{fill}" rx="2.5" ry="2.5"/>"#,
                            fmt_coord(ecx),
                            fmt_coord(ecy),
                        )
                        .unwrap();
                        // Bullet text at x+12 (Bullet atom width=12)
                        let bullet_text_x = text_x + 12.0;
                        let mut tmp = String::new();
                        render_creole_text(
                            &mut tmp,
                            item_text,
                            bullet_text_x,
                            baseline,
                            lh,
                            fill,
                            None,
                            &outer_attrs,
                        );
                        buf.push_str(&tmp);
                        cursor_y += lh;
                    }
                }
                NoteBlock::Table(raw_rows) => {
                    // AtomWithMargin adds 2px top margin
                    cursor_y += TABLE_MARGIN_Y;

                    let rows = parse_display_table_rows(
                        &raw_rows.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                        false,
                    );
                    let layout = layout_display_table(&rows, default_font, font_size, false, false);
                    render_display_table(
                        buf,
                        &rows,
                        &layout,
                        text_x,
                        cursor_y - TABLE_MARGIN_Y, // render_display_table adds TABLE_MARGIN_Y internally
                        fill,
                        font_size,
                        default_font,
                        false,
                        false,
                    );
                    cursor_y += layout.total_height;
                }
            }
        }

        // If this segment had a separator, add bottom decoration margin
        if separator_positions.contains(&seg_idx) {
            cursor_y += 4.0; // bottom margin of decorated block
        }
    }

    // Return textBlock.h = total content height consumed
    cursor_y - (note_y + margin_y)
}

/// Returns true if the line is a block separator in BodyEnhanced2 sense.
fn is_block_separator(line: &str) -> bool {
    (line.starts_with("--") && line.ends_with("--"))
        || (line.starts_with("==") && line.ends_with("=="))
        || (line.starts_with("..") && line.ends_with("..") && line != "...")
        || (line.starts_with("__") && line.ends_with("__"))
}

/// Compute note textBlock height for creole content.
///
/// Models Java's BodyEnhanced2 + SheetBlock1 height computation:
/// - Regular/bullet lines: line_height each
/// - `----`/`====` separators: 0 height (but add 8px decoration margin)
/// - Tables: AtomWithMargin(table, 2, 2) wrapping n_rows * line_height
pub fn compute_creole_note_text_height(text: &str, font_size: f64) -> f64 {
    let lh = font_metrics::line_height("SansSerif", font_size, false, false);

    let raw_lines: Vec<&str> = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.split('\n'))
        .flat_map(|s| s.split("\\n"))
        .collect();

    // Split at block separators
    let mut segments: Vec<Vec<&str>> = Vec::new();
    let mut n_separators = 0usize;
    let mut current: Vec<&str> = Vec::new();

    for line in &raw_lines {
        let trimmed = line.trim();
        if is_block_separator(trimmed) {
            segments.push(std::mem::take(&mut current));
            n_separators += 1;
        } else {
            current.push(line);
        }
    }
    segments.push(current);

    let mut total = 0.0;
    for segment in &segments {
        let blocks = parse_note_segment(segment);
        for block in &blocks {
            match block {
                NoteBlock::TextLines(lines) => {
                    for line in lines {
                        total += note_line_height_with_sprites(line, font_size, lh);
                    }
                }
                NoteBlock::BulletItems(items) => {
                    for item in items {
                        total += note_line_height_with_sprites(item, font_size, lh);
                    }
                }
                NoteBlock::Table(rows) => {
                    // AtomWithMargin(table, 2, 2): 2px top + 2px bottom
                    total += rows.len() as f64 * lh + 2.0 * TABLE_MARGIN_Y;
                }
            }
        }
    }

    // Each separator adds 8px decoration margin (4 top + 4 bottom)
    total += n_separators as f64 * 8.0;

    total
}

/// Compute the effective height of a single note line, accounting for inline sprites.
///
/// If the line contains `<$spritename>` references, the sprite's viewBox height
/// may exceed the normal line height. Java's BodyEnhanced2 uses
/// `max(sprite_height, line_height)` per atom.
fn note_line_height_with_sprites(line: &str, font_size: f64, base_lh: f64) -> f64 {
    if !line.contains("<$") {
        return base_lh;
    }
    let mut max_sprite_h = 0.0_f64;
    for span in flatten_rich_lines(&parse_creole(line))
        .into_iter()
        .flatten()
    {
        if let TextSpan::InlineSvg { name, scale, .. } = span {
            if let Some(svg_content) = get_sprite(&name) {
                let info = crate::render::svg_sprite::sprite_info(&svg_content);
                max_sprite_h =
                    max_sprite_h.max(info.vb_height * sprite_scale_for_font(font_size, scale));
            }
        }
    }
    if max_sprite_h > base_lh {
        max_sprite_h
    } else {
        base_lh
    }
}

fn line_height_with_inline_sprites(spans: &[TextSpan], font_size: f64, base_lh: f64) -> f64 {
    let mut max_sprite_h = 0.0_f64;
    for span in spans {
        if let TextSpan::InlineSvg { name, scale, .. } = span {
            if let Some(svg_content) = get_sprite(name) {
                let info = crate::render::svg_sprite::sprite_info(&svg_content);
                max_sprite_h =
                    max_sprite_h.max(info.vb_height * sprite_scale_for_font(font_size, *scale));
            }
        }
    }
    if max_sprite_h > base_lh {
        max_sprite_h
    } else {
        base_lh
    }
}

/// Compute the height of a creole-rendered entity name (bold, cluster title).
///
/// Similar to `compute_creole_note_text_height` but uses bold metrics and
/// 10px separator margin (5 before + 5 after) matching Java's ClusterHeader
/// rendering of TextBlockVertical with BodyEnhanced2 creole blocks.
pub fn compute_creole_entity_name_height(text: &str, font_size: f64) -> f64 {
    let lh = font_metrics::line_height("SansSerif", font_size, true, false);

    let raw_lines: Vec<&str> = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.lines())
        .collect();

    let mut segments: Vec<Vec<&str>> = Vec::new();
    let mut n_separators = 0usize;
    let mut current: Vec<&str> = Vec::new();

    for line in &raw_lines {
        let trimmed = line.trim();
        if is_block_separator(trimmed) {
            segments.push(std::mem::take(&mut current));
            n_separators += 1;
        } else {
            current.push(line);
        }
    }
    segments.push(current);

    let mut total = 0.0;
    for segment in &segments {
        let blocks = parse_note_segment(segment);
        for block in &blocks {
            match block {
                NoteBlock::TextLines(lines) => {
                    total += lines.len() as f64 * lh;
                }
                NoteBlock::BulletItems(items) => {
                    total += items.len() as f64 * lh;
                }
                NoteBlock::Table(_) => {
                    // Tables not expected in entity names; treat as line
                    total += lh;
                }
            }
        }
    }

    // Each separator adds 10px (5 before HR + 5 after HR)
    total += n_separators as f64 * 10.0;

    total
}

/// Render a creole-formatted entity name (bold, centered) into SVG.
///
/// Handles `----` separators (horizontal rules) and `* item` bullet lists
/// in entity names. Returns the total rendered height.
pub fn render_creole_entity_name(
    buf: &mut String,
    text: &str,
    cluster_x: f64,
    cluster_y: f64,
    cluster_w: f64,
    font_color: &str,
    border_color: &str,
    font_size: f64,
) -> f64 {
    let lh = font_metrics::line_height("SansSerif", font_size, true, false);
    let ascent = font_metrics::ascent("SansSerif", font_size, true, false);

    let raw_lines: Vec<&str> = text
        .split(crate::NEWLINE_CHAR)
        .flat_map(|s| s.lines())
        .collect();

    // Split at block separators
    let mut segments: Vec<Vec<&str>> = Vec::new();
    let mut separator_positions: Vec<usize> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in &raw_lines {
        let trimmed = line.trim();
        if is_block_separator(trimmed) {
            segments.push(std::mem::take(&mut current));
            separator_positions.push(segments.len());
        } else {
            current.push(line);
        }
    }
    segments.push(current);

    // Compute the max text width across all lines (for HR width and centering)
    let all_line_widths: Vec<f64> = raw_lines
        .iter()
        .filter(|l| !is_block_separator(l.trim()))
        .map(|line| {
            let trimmed = line.trim();
            if let Some(bullet_text) = trimmed.strip_prefix("* ") {
                // Bullet line width = bullet_indent(12) + text_width
                12.0 + font_metrics::text_width(bullet_text, "SansSerif", font_size, true, false)
            } else {
                font_metrics::text_width(trimmed, "SansSerif", font_size, true, false)
            }
        })
        .collect();
    let max_line_w = all_line_widths.iter().cloned().fold(0.0_f64, f64::max);

    let start_y = cluster_y + 2.0;
    let mut cursor_y = start_y;

    for (seg_idx, segment) in segments.iter().enumerate() {
        // Draw HR separator if this segment follows a "----"/"===="
        if separator_positions.contains(&seg_idx) {
            let hr_y = cursor_y + 5.0;
            let hr_x1 = cluster_x + (cluster_w - max_line_w) / 2.0;
            let hr_x2 = hr_x1 + max_line_w;
            write!(
                buf,
                r#"<line style="stroke:{border_color};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(hr_x1),
                fmt_coord(hr_x2),
                fmt_coord(hr_y),
                fmt_coord(hr_y),
            )
            .unwrap();
            cursor_y += 10.0; // 5 before + 5 after
        }

        let blocks = parse_note_segment(segment);

        for block in &blocks {
            match block {
                NoteBlock::TextLines(lines) => {
                    for line_text in lines {
                        let tl = font_metrics::text_width(
                            line_text,
                            "SansSerif",
                            font_size,
                            true,
                            false,
                        );
                        let text_x = cluster_x + (cluster_w - tl) / 2.0;
                        let baseline = cursor_y + ascent;
                        write!(
                            buf,
                            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{}" font-weight="bold" lengthAdjust="spacing" textLength="{}"{}>{}</text>"#,
                            font_size as u32,
                            fmt_coord(tl),
                            format!(r#" x="{}" y="{}""#, fmt_coord(text_x), fmt_coord(baseline)),
                            xml_escape(line_text),
                        )
                        .unwrap();
                        cursor_y += lh;
                    }
                }
                NoteBlock::BulletItems(items) => {
                    for item_text in items {
                        let text_w = font_metrics::text_width(
                            item_text,
                            "SansSerif",
                            font_size,
                            true,
                            false,
                        );
                        // Total bullet line width = 12 (indent) + text_w
                        let bullet_line_w = 12.0 + text_w;
                        // Center the bullet line as a unit within the cluster
                        let line_x = cluster_x + (cluster_w - bullet_line_w) / 2.0;
                        let ecx = line_x + 3.0 + 2.5;
                        let ecy = cursor_y + lh - 7.5;
                        write!(
                            buf,
                            r#"<ellipse cx="{}" cy="{}" fill="{font_color}" rx="2.5" ry="2.5"/>"#,
                            fmt_coord(ecx),
                            fmt_coord(ecy),
                        )
                        .unwrap();
                        let bullet_text_x = line_x + 12.0;
                        let baseline = cursor_y + ascent;
                        let tl = text_w;
                        write!(
                            buf,
                            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{}" font-weight="bold" lengthAdjust="spacing" textLength="{}"{}>{}</text>"#,
                            font_size as u32,
                            fmt_coord(tl),
                            format!(
                                r#" x="{}" y="{}""#,
                                fmt_coord(bullet_text_x),
                                fmt_coord(baseline)
                            ),
                            xml_escape(item_text),
                        )
                        .unwrap();
                        cursor_y += lh;
                    }
                }
                NoteBlock::Table(_) => {
                    // Tables not expected in entity names
                    cursor_y += lh;
                }
            }
        }
    }

    cursor_y - start_y
}

enum NoteBlock<'a> {
    TextLines(Vec<&'a str>),
    BulletItems(Vec<&'a str>),
    Table(Vec<&'a str>),
}

fn parse_note_segment<'a>(lines: &[&'a str]) -> Vec<NoteBlock<'a>> {
    let mut blocks: Vec<NoteBlock<'a>> = Vec::new();
    let mut text_lines: Vec<&'a str> = Vec::new();
    let mut bullet_items: Vec<&'a str> = Vec::new();
    let mut table_rows: Vec<&'a str> = Vec::new();

    for &line in lines {
        let trimmed = line.trim();

        // Table line
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2 {
            // Flush text/bullets
            if !text_lines.is_empty() {
                blocks.push(NoteBlock::TextLines(std::mem::take(&mut text_lines)));
            }
            if !bullet_items.is_empty() {
                blocks.push(NoteBlock::BulletItems(std::mem::take(&mut bullet_items)));
            }
            table_rows.push(line);
            continue;
        }

        // Flush table if switching to non-table
        if !table_rows.is_empty() {
            blocks.push(NoteBlock::Table(std::mem::take(&mut table_rows)));
        }

        // Bullet line
        if trimmed.starts_with("* ") {
            if !text_lines.is_empty() {
                blocks.push(NoteBlock::TextLines(std::mem::take(&mut text_lines)));
            }
            bullet_items.push(&trimmed[2..]);
            continue;
        }

        // Regular text line: preserve leading whitespace so render_prepared_line
        // can emulate Java's DriverTextSvg `x += space_w * n` shift. Only strip
        // trailing whitespace via trim_end.
        if !bullet_items.is_empty() {
            blocks.push(NoteBlock::BulletItems(std::mem::take(&mut bullet_items)));
        }
        text_lines.push(line.trim_end());
    }

    // Flush remaining
    if !text_lines.is_empty() {
        blocks.push(NoteBlock::TextLines(text_lines));
    }
    if !bullet_items.is_empty() {
        blocks.push(NoteBlock::BulletItems(bullet_items));
    }
    if !table_rows.is_empty() {
        blocks.push(NoteBlock::Table(table_rows));
    }

    blocks
}

fn build_display_blocks(lines: &[String], preserve_backslash_n: bool) -> Vec<DisplayBlock> {
    let mut blocks = Vec::new();
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = &lines[idx];
        if is_table_display_line(line) {
            let mut raw_rows = Vec::new();
            while idx < lines.len() && is_table_display_line(&lines[idx]) {
                raw_rows.push(lines[idx].clone());
                idx += 1;
            }
            blocks.push(DisplayBlock::Table(parse_display_table_rows(
                &raw_rows,
                preserve_backslash_n,
            )));
            continue;
        }

        blocks.push(DisplayBlock::Text(split_display_line(line)));
        idx += 1;
    }

    blocks
}

fn split_display_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    for ch in line.chars() {
        if ch == crate::NEWLINE_CHAR || ch == '\n' {
            parts.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    parts.push(current);
    parts
}

fn is_table_display_line(line: &str) -> bool {
    let trimmed = line.trim();
    // Strip optional leading color prefix: <#color> or <#color,#border>
    let after_prefix = if trimmed.starts_with("<#") {
        if let Some(gt) = trimmed.find('>') {
            trimmed[gt + 1..].trim_start()
        } else {
            trimmed
        }
    } else {
        trimmed
    };
    after_prefix.starts_with('|') && after_prefix.ends_with('|') && after_prefix.len() >= 2
}

fn parse_display_table_rows(lines: &[String], preserve_backslash_n: bool) -> Vec<DisplayTableRow> {
    lines
        .iter()
        .map(|line| {
            let (row_bg, border_color, table_line) = strip_row_color_prefix(line);
            DisplayTableRow {
                cells: parse_display_table_cells(&table_line, preserve_backslash_n),
                bg_color: row_bg,
                border_color,
            }
        })
        .collect()
}

/// Strip leading `<#color>` or `<#color,#border>` row prefix from a table line.
/// Returns (bg_color, border_color, remaining_line).
fn strip_row_color_prefix(line: &str) -> (Option<String>, Option<String>, String) {
    let trimmed = line.trim();
    if !trimmed.starts_with("<#") {
        return (None, None, trimmed.to_string());
    }
    if let Some(gt) = trimmed.find('>') {
        let color_spec = &trimmed[2..gt]; // between <# and >
        let rest = trimmed[gt + 1..].to_string();
        if let Some(comma) = color_spec.find(',') {
            let bg = color_spec[..comma].to_string();
            let border = color_spec[comma + 1..].trim_start_matches('#').to_string();
            (Some(bg), Some(border), rest)
        } else {
            (Some(color_spec.to_string()), None, rest)
        }
    } else {
        (None, None, trimmed.to_string())
    }
}

/// Apply Java `StripeTable.getWithNewlinesInternal` collapses to a table
/// cell text. In legacy mode (always on), each `\\` is replaced with a single
/// `\` and each remaining `\n` will later be split by `split_table_cell_lines`.
///
/// We must collapse `\\` BEFORE the link parser runs, otherwise the tooltip
/// extracted from `[[{a\\nb}...]]` would still contain the doubled backslash
/// and `process_xlink_title` would emit the literal `\` (instead of dropping
/// it the way Java does for table cells).
fn collapse_table_cell_escapes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            let c2 = chars[i + 1];
            if c2 == '\\' {
                // `\\` → `\`
                out.push('\\');
                i += 2;
                continue;
            }
            // `\n` and other `\X` sequences are left intact so the cell
            // splitter / inline parser can handle them.
        }
        out.push(c);
        i += 1;
    }
    out
}

fn parse_display_table_cells(line: &str, preserve_backslash_n: bool) -> Vec<DisplayTableCell> {
    let trimmed = line.trim();
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);

    inner
        .split('|')
        .map(|part| {
            let leading_spaces = part.len() - part.trim_start().len();
            let trailing_spaces = part.len() - part.trim_end().len();
            let mut cell_text = part.trim();
            let is_header = cell_text.starts_with('=');
            if is_header {
                cell_text = cell_text.trim_start_matches('=');
                let inner_leading = cell_text.len() - cell_text.trim_start().len();
                cell_text = cell_text.trim();
                let collapsed = collapse_table_cell_escapes(cell_text);
                let mut lines: Vec<Vec<TextSpan>> =
                    split_table_cell_lines(&collapsed, preserve_backslash_n)
                        .into_iter()
                        .map(|cell_line| parse_inline(&cell_line))
                        .collect();
                let mut el = 0usize;
                let mut et = 0usize;
                for line in &mut lines {
                    let (l, t) = strip_span_edge_spaces(line);
                    el = el.max(l);
                    et = et.max(t);
                }
                return DisplayTableCell {
                    lines,
                    is_header,
                    bg_color: None,
                    leading_spaces: leading_spaces + inner_leading + el,
                    trailing_spaces: trailing_spaces + et,
                };
            }
            let (cell_bg, cell_text) = extract_cell_bg_color(cell_text);
            let collapsed = collapse_table_cell_escapes(cell_text);
            let mut lines: Vec<Vec<TextSpan>> =
                split_table_cell_lines(&collapsed, preserve_backslash_n)
                    .into_iter()
                    .map(|cell_line| parse_inline(&cell_line))
                    .collect();
            let mut el = 0usize;
            let mut et = 0usize;
            for line in &mut lines {
                let (l, t) = strip_span_edge_spaces(line);
                el = el.max(l);
                et = et.max(t);
            }
            DisplayTableCell {
                lines,
                is_header,
                bg_color: cell_bg,
                leading_spaces: leading_spaces + el,
                trailing_spaces: trailing_spaces + et,
            }
        })
        .collect()
}

fn strip_span_edge_spaces(spans: &mut Vec<TextSpan>) -> (usize, usize) {
    let mut lead = 0usize;
    let mut trail = 0usize;
    if let Some(p) = find_first_plain_mut(spans) {
        let t = p.trim_start();
        lead = p.len() - t.len();
        if lead > 0 {
            *p = t.to_string();
        }
    }
    if let Some(p) = find_last_plain_mut(spans) {
        let t = p.trim_end();
        trail = p.len() - t.len();
        if trail > 0 {
            *p = t.to_string();
        }
    }
    (lead, trail)
}
fn find_first_plain_mut(spans: &mut [TextSpan]) -> Option<&mut String> {
    for span in spans.iter_mut() {
        match span {
            TextSpan::Plain(ref mut s) => return Some(s),
            TextSpan::Bold(i)
            | TextSpan::Italic(i)
            | TextSpan::Underline(i)
            | TextSpan::Strikethrough(i)
            | TextSpan::Subscript(i)
            | TextSpan::Superscript(i) => {
                if let Some(s) = find_first_plain_mut(i) {
                    return Some(s);
                }
            }
            TextSpan::Colored { content: c, .. }
            | TextSpan::UnderlineColored { content: c, .. }
            | TextSpan::BackHighlight { content: c, .. }
            | TextSpan::Sized { content: c, .. }
            | TextSpan::FontFamily { content: c, .. } => {
                if let Some(s) = find_first_plain_mut(c) {
                    return Some(s);
                }
            }
            TextSpan::Link { label, url, .. } => {
                if label.is_some() || !url.is_empty() {
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}
fn find_last_plain_mut(spans: &mut [TextSpan]) -> Option<&mut String> {
    for span in spans.iter_mut().rev() {
        match span {
            TextSpan::Plain(ref mut s) => return Some(s),
            TextSpan::Bold(i)
            | TextSpan::Italic(i)
            | TextSpan::Underline(i)
            | TextSpan::Strikethrough(i)
            | TextSpan::Subscript(i)
            | TextSpan::Superscript(i) => {
                if let Some(s) = find_last_plain_mut(i) {
                    return Some(s);
                }
            }
            TextSpan::Colored { content: c, .. }
            | TextSpan::UnderlineColored { content: c, .. }
            | TextSpan::BackHighlight { content: c, .. }
            | TextSpan::Sized { content: c, .. }
            | TextSpan::FontFamily { content: c, .. } => {
                if let Some(s) = find_last_plain_mut(c) {
                    return Some(s);
                }
            }
            TextSpan::Link { label, url, .. } => {
                if label.is_some() || !url.is_empty() {
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract leading `<#color>` cell background prefix from table cell text.
/// Returns (bg_color, remaining_text).
fn extract_cell_bg_color(text: &str) -> (Option<String>, &str) {
    let s = text.trim_start();
    if !s.starts_with("<#") {
        return (None, text);
    }
    if let Some(gt) = s.find('>') {
        let color = s[2..gt].to_string();
        (Some(color), s[gt + 1..].trim_start())
    } else {
        (None, text)
    }
}

fn split_table_cell_lines(text: &str, preserve_backslash_n: bool) -> Vec<String> {
    let mut parts = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut current = String::new();
    let mut idx = 0usize;
    let mut bracket_depth = 0u32; // track [[...]] nesting

    while idx < chars.len() {
        let ch = chars[idx];

        // Track [[...]] link brackets — don't split inside them
        if ch == '[' && idx + 1 < chars.len() && chars[idx + 1] == '[' {
            bracket_depth += 1;
            current.push(ch);
            idx += 1;
            continue;
        }
        if ch == ']' && idx + 1 < chars.len() && chars[idx + 1] == ']' && bracket_depth > 0 {
            bracket_depth -= 1;
            current.push(ch);
            current.push(chars[idx + 1]);
            idx += 2;
            continue;
        }

        if bracket_depth == 0 {
            if ch == crate::NEWLINE_CHAR || ch == '\n' {
                parts.push(std::mem::take(&mut current).trim().to_string());
                idx += 1;
                continue;
            }
            if !preserve_backslash_n && ch == '\\' && idx + 1 < chars.len() && chars[idx + 1] == 'n'
            {
                parts.push(std::mem::take(&mut current).trim().to_string());
                idx += 2;
                continue;
            }
        }

        current.push(ch);
        idx += 1;
    }

    parts.push(current.trim().to_string());
    parts
}

fn layout_display_table(
    rows: &[DisplayTableRow],
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> DisplayTableLayout {
    let col_count = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    let mut col_widths = vec![0.0_f64; col_count];
    let mut row_heights = vec![0.0_f64; rows.len()];

    for (row_idx, row) in rows.iter().enumerate() {
        let mut row_height = 0.0_f64;
        for (col_idx, cell) in row.cells.iter().enumerate() {
            let cell_bold = bold || cell.is_header;
            let cell_line_height =
                font_metrics::line_height(default_font, font_size, cell_bold, italic);
            row_height = row_height.max(cell_line_height * cell.lines.len().max(1) as f64);
            col_widths[col_idx] = col_widths[col_idx].max(measure_table_cell_width(
                cell,
                default_font,
                font_size,
                cell_bold,
                italic,
            ));
        }
        row_heights[row_idx] = row_height;
    }

    let width = col_widths.iter().sum();
    let total_height = row_heights.iter().sum::<f64>() + 2.0 * TABLE_MARGIN_Y;

    DisplayTableLayout {
        col_widths,
        row_heights,
        width,
        total_height,
    }
}

fn measure_table_cell_width(
    cell: &DisplayTableCell,
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> f64 {
    let nbsp_width = font_metrics::text_width(NBSP, default_font, font_size, bold, italic);
    let pad_left = cell.leading_spaces.max(1) as f64 * nbsp_width;
    let pad_right = cell.trailing_spaces.max(1) as f64 * nbsp_width;
    let mut width = 0.0_f64;

    for line in &cell.lines {
        let plain = plain_text_spans(line);
        let candidate = if plain.is_empty() {
            nbsp_width
        } else {
            measure_parsed_line_width(line, default_font, font_size, bold, italic)
                + pad_left
                + pad_right
        };
        width = width.max(candidate);
    }

    width
}

fn cell_has_link(spans: &[TextSpan]) -> bool {
    spans
        .iter()
        .any(|span| matches!(span, TextSpan::Link { .. }))
}

fn render_display_table(
    buf: &mut String,
    rows: &[DisplayTableRow],
    layout: &DisplayTableLayout,
    x: f64,
    top_y: f64,
    fill: &str,
    font_size: f64,
    default_font: &str,
    bold: bool,
    italic: bool,
) {
    let grid_top = top_y + TABLE_MARGIN_Y;

    // Determine grid line color: use first row's border_color if present, else default
    let grid_color = rows
        .iter()
        .find_map(|r| r.border_color.as_ref())
        .map(|c| resolve_color_to_svg(c))
        .unwrap_or_else(|| "#000000".to_string());
    // Use stroke-width 1.0 when custom border color is set, else 0.5
    let grid_stroke_w = if rows.iter().any(|r| r.border_color.is_some()) {
        "1"
    } else {
        "0.5"
    };

    let mut row_top = grid_top;

    for (row_idx, row) in rows.iter().enumerate() {
        let row_height = layout.row_heights[row_idx];

        // 1) Row background rect (rendered before all cells in this row)
        if let Some(ref bg) = row.bg_color {
            let bg_hex = resolve_color_to_svg(bg);
            write!(
                buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                bg_hex,
                fmt_coord(row_height),
                fmt_coord(layout.width),
                fmt_coord(x),
                fmt_coord(row_top),
            )
            .unwrap();
        }

        // 2) For each cell: render cell bg (if any), then cell text — interleaved
        let mut col_x = x;
        for (col_idx, col_width) in layout.col_widths.iter().enumerate() {
            if let Some(cell) = row.cells.get(col_idx) {
                let cell_bold = bold || cell.is_header;
                let cell_line_height =
                    font_metrics::line_height(default_font, font_size, cell_bold, italic);
                let cell_ascent = font_metrics::ascent(default_font, font_size, cell_bold, italic);
                let cell_attrs = build_cell_outer_attrs(font_size, cell_bold, italic);

                // Cell background rect
                if let Some(ref bg) = cell.bg_color {
                    let bg_hex = resolve_color_to_svg(bg);
                    write!(
                        buf,
                        r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                        bg_hex,
                        fmt_coord(row_height),
                        fmt_coord(*col_width),
                        fmt_coord(col_x),
                        fmt_coord(row_top),
                    )
                    .unwrap();
                }

                let nbsp_w =
                    font_metrics::text_width(NBSP, default_font, font_size, cell_bold, italic);
                let lead_count = cell.leading_spaces.max(1);
                let trail_count = cell.trailing_spaces.max(1);
                let cell_pad_left = lead_count as f64 * nbsp_w;

                for (line_idx, line) in cell.lines.iter().enumerate() {
                    let baseline = row_top + cell_ascent + line_idx as f64 * cell_line_height;
                    let plain = plain_text_spans(line);
                    if plain.is_empty() {
                        let rl = vec![TextSpan::Plain(NBSP.to_string())];
                        render_preparsed_lines(
                            buf,
                            &[rl],
                            col_x,
                            baseline,
                            cell_line_height,
                            fill,
                            None,
                            &cell_attrs,
                        );
                    } else if cell_has_link(line) {
                        let wt = if cell_bold {
                            r#" font-weight="bold""#
                        } else {
                            ""
                        };
                        if lead_count > 0 {
                            let ns = NBSP.repeat(lead_count);
                            let nw = lead_count as f64 * nbsp_w;
                            write!(buf,
                                r#"<text fill="{}" font-family="{}" font-size="{}"{} lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                                xml_escape(fill), xml_escape(default_font), font_size as i32, wt,
                                fmt_coord(nw), fmt_coord(col_x), fmt_coord(baseline),
                                xml_escape(&ns)).unwrap();
                        }
                        render_preparsed_lines(
                            buf,
                            &[line.clone()],
                            col_x + cell_pad_left,
                            baseline,
                            cell_line_height,
                            fill,
                            None,
                            &cell_attrs,
                        );
                        if trail_count > 0 {
                            let cw = measure_parsed_line_width(
                                line,
                                default_font,
                                font_size,
                                cell_bold,
                                italic,
                            );
                            let tx = col_x + cell_pad_left + cw;
                            let ns = NBSP.repeat(trail_count);
                            let nw = trail_count as f64 * nbsp_w;
                            write!(buf,
                                r#"<text fill="{}" font-family="{}" font-size="{}"{} lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                                xml_escape(fill), xml_escape(default_font), font_size as i32, wt,
                                fmt_coord(nw), fmt_coord(tx), fmt_coord(baseline),
                                xml_escape(&ns)).unwrap();
                        }
                    } else {
                        render_preparsed_lines(
                            buf,
                            &[line.clone()],
                            col_x + cell_pad_left,
                            baseline,
                            cell_line_height,
                            fill,
                            None,
                            &cell_attrs,
                        );
                    }
                }
            }

            col_x += col_width;
        }

        row_top += row_height;
    }

    // 4) Grid lines
    let grid_bottom = grid_top + layout.row_heights.iter().sum::<f64>();
    let mut y = grid_top;
    // Horizontal lines
    write!(
        buf,
        r#"<line style="stroke:{};stroke-width:{};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        grid_color,
        grid_stroke_w,
        fmt_coord(x),
        fmt_coord(x + layout.width),
        fmt_coord(y),
        fmt_coord(y)
    )
    .unwrap();
    for row_height in &layout.row_heights {
        y += row_height;
        write!(
            buf,
            r#"<line style="stroke:{};stroke-width:{};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            grid_color,
            grid_stroke_w,
            fmt_coord(x),
            fmt_coord(x + layout.width),
            fmt_coord(y),
            fmt_coord(y)
        )
        .unwrap();
    }

    // Vertical lines
    let mut vx = x;
    write!(
        buf,
        r#"<line style="stroke:{};stroke-width:{};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        grid_color,
        grid_stroke_w,
        fmt_coord(vx),
        fmt_coord(vx),
        fmt_coord(grid_top),
        fmt_coord(grid_bottom)
    )
    .unwrap();
    for width in &layout.col_widths {
        vx += width;
        write!(
            buf,
            r#"<line style="stroke:{};stroke-width:{};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            grid_color,
            grid_stroke_w,
            fmt_coord(vx),
            fmt_coord(vx),
            fmt_coord(grid_top),
            fmt_coord(grid_bottom)
        )
        .unwrap();
    }
}

/// Resolve a color name (e.g. "lightblue", "Navy", "FF0000") to SVG hex format.
fn resolve_color_to_svg(name: &str) -> String {
    if let Some(hc) = resolve_color(name) {
        hc.to_svg()
    } else if name.starts_with('#') {
        name.to_uppercase()
    } else {
        format!("#{}", name.to_uppercase())
    }
}

fn build_cell_outer_attrs(font_size: f64, bold: bool, italic: bool) -> String {
    let mut attrs = format!(r#"font-size="{font_size}""#);
    if bold {
        attrs.push_str(r#" font-weight="bold""#);
    }
    if italic {
        attrs.push_str(r#" font-style="italic""#);
    }
    attrs
}

fn measure_parsed_line_width(
    spans: &[TextSpan],
    default_font: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) -> f64 {
    if !line_needs_split_render(spans) {
        let plain = trimmed_plain_line_text(spans);
        return font_metrics::text_width(&plain, default_font, font_size, bold, italic);
    }

    let runs = flatten_to_runs(spans);
    let mut total = 0.0_f64;
    let mut first = true;
    for run in &runs {
        let text = if !first {
            run.text.trim_start()
        } else {
            run.text.as_str()
        };
        if text.is_empty() {
            first = false;
            continue;
        }
        if !first && text.len() < run.text.len() {
            let n_spaces = run.text.len() - text.len();
            total += font_metrics::text_width(" ", default_font, font_size, false, false)
                * n_spaces as f64;
        }
        let run_font = run.font_family.as_deref().unwrap_or(default_font);
        let run_bold = run.bold || bold;
        let run_italic = run.italic || italic;
        let run_size = match run.font_size_override {
            Some(v) if v == -1.0 || v == -2.0 => (font_size * 0.77).round(),
            // Heading sentinel: size = -100 - delta → effective = base + delta.
            Some(v) if v <= -100.0 => font_size + (-100.0 - v),
            Some(v) if v > 0.0 => v,
            _ => font_size,
        };
        total += font_metrics::text_width(text, run_font, run_size, run_bold, run_italic);
        first = false;
    }
    total
}

fn render_preparsed_lines(
    buf: &mut String,
    lines: &[Vec<TextSpan>],
    x: f64,
    y: f64,
    line_height: f64,
    fill: &str,
    text_anchor: Option<&str>,
    outer_attrs: &str,
) -> usize {
    let (font_family, font_size, bold, italic) = parse_font_props(outer_attrs);

    if lines.len() == 1 && text_anchor.is_none() && line_needs_split_render(&lines[0]) {
        render_split_text_runs(
            buf,
            &lines[0],
            x,
            y,
            fill,
            outer_attrs,
            &font_family,
            font_size,
            bold,
            italic,
        );
        return 1;
    }

    for (idx, line) in lines.iter().enumerate() {
        let line_y = y + idx as f64 * line_height;
        render_prepared_line(
            buf,
            line,
            x,
            line_y,
            fill,
            text_anchor,
            outer_attrs,
            &font_family,
            font_size,
            bold,
            italic,
        );
    }

    lines.len()
}

#[allow(clippy::too_many_arguments)]
fn render_prepared_line(
    buf: &mut String,
    line: &[TextSpan],
    x: f64,
    y: f64,
    fill: &str,
    text_anchor: Option<&str>,
    outer_attrs: &str,
    font_family: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) {
    let line_plain = plain_text_spans(line);
    let trimmed_plain = line_plain.trim();
    let is_visual_blank_line = trimmed_plain.is_empty();
    // Java DriverTextSvg (DriverTextSvg.draw, lines 110–116): when the text
    // starts with a space, each leading space is stripped and `x` is shifted
    // right by `stringBounder.calculateDimension(font, " ").getWidth()`.
    // Trailing spaces are also removed (StringUtils.trin).  Only the trimmed
    // text is measured and written to the SVG, but the element x-attribute
    // reflects the leading-space shift.  `text_anchor="middle"` uses the
    // original x unchanged (the centering already accounts for the full
    // untrimmed width upstream).
    let x_shifted = if text_anchor.is_none() && !is_visual_blank_line {
        let leading_spaces = line_plain.chars().take_while(|c| *c == ' ').count();
        if leading_spaces > 0 {
            let space_w = font_metrics::text_width(" ", font_family, font_size, bold, italic);
            x + space_w * leading_spaces as f64
        } else {
            x
        }
    } else {
        x
    };
    if is_visual_blank_line {
        let text_length = font_metrics::text_width(NBSP, font_family, font_size, bold, italic);
        write_text_open(buf, x, y, fill, text_anchor, outer_attrs, text_length);
        buf.push_str(&xml_escape(NBSP));
        buf.push_str("</text>");
    } else if text_anchor.is_none() {
        let text_length =
            font_metrics::text_width(trimmed_plain, font_family, font_size, bold, italic);
        write_text_open(
            buf,
            x_shifted,
            y,
            fill,
            text_anchor,
            outer_attrs,
            text_length,
        );
        if let Some(text) = simple_plain_line(line) {
            buf.push_str(&xml_escape(text.trim()));
        } else {
            let style = SpanStyle {
                ambient_font_size: Some(font_size),
                ..Default::default()
            };
            render_spans(buf, line, &style, fill);
        }
        buf.push_str("</text>");
    } else if let Some(text) = simple_plain_line(line) {
        let text_length =
            font_metrics::text_width(trimmed_plain, font_family, font_size, bold, italic);
        write_text_open(buf, x, y, fill, text_anchor, outer_attrs, text_length);
        buf.push_str(&xml_escape(text.trim()));
        buf.push_str("</text>");
    } else {
        let text_length =
            font_metrics::text_width(trimmed_plain, font_family, font_size, bold, italic);
        write_text_open(buf, x, y, fill, text_anchor, outer_attrs, text_length);
        let style = SpanStyle {
            ambient_font_size: Some(font_size),
            ..Default::default()
        };
        render_spans(buf, line, &style, fill);
        buf.push_str("</text>");
    }
}

/// Render a Creole section title (`==text==`) as two horizontal strokes
/// with the title text centered between them.  Mirrors Java's
/// `UHorizontalLine.drawLineInternal` when the stripe style is `=`: it
/// draws a stencil-wide line at y_center and another at y_center+2, with
/// the title painted between the left-half and right-half segments.
///
/// Geometry (see Java `UHorizontalLine.drawTitleInternal` and
/// `CreoleHorizontalLine.drawU`):
///   row_top    = y_baseline - ascent
///   y_center   = row_top + line_height / 2
///   text_y     = y_center - line_height/2 - 0.5 + ascent
///              = y_baseline - 0.5
///   title_x1   = stencil_start + (stencil_width - title_width) / 2
///   title_x2   = title_x1 + title_width
#[allow(clippy::too_many_arguments)]
fn render_section_title_line(
    buf: &mut String,
    line: &[TextSpan],
    y_baseline: f64,
    line_height: f64,
    fill: &str,
    outer_attrs: &str,
    font_family: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
    bounds: &SectionTitleBounds,
) {
    let ascent = font_metrics::ascent(font_family, font_size, bold, italic);
    let row_top = y_baseline - ascent;
    let y_center = row_top + line_height / 2.0;
    let y1 = y_center;
    let y2 = y_center + 2.0;

    let stencil_start = bounds.x_start;
    let stencil_end = bounds.x_end;
    let stroke = &bounds.stroke;

    // Measure the title text (trimmed of leading/trailing whitespace so the
    // centering uses the visible width only, matching Java's
    // AtomTextUtils.createLegacy pipeline).
    let trimmed_title = trimmed_plain_line_text(line);

    if trimmed_title.is_empty() {
        // No title → single continuous line pair (Java `drawHLine` with
        // null title).
        write!(
            buf,
            r#"<line style="stroke:{stroke};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(stencil_start),
            fmt_coord(stencil_end),
            fmt_coord(y1),
            fmt_coord(y1),
        )
        .unwrap();
        write!(
            buf,
            r#"<line style="stroke:{stroke};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(stencil_start),
            fmt_coord(stencil_end),
            fmt_coord(y2),
            fmt_coord(y2),
        )
        .unwrap();
        return;
    }

    let title_width =
        font_metrics::text_width(&trimmed_title, font_family, font_size, bold, italic);
    let stencil_width = stencil_end - stencil_start;
    let space = (stencil_width - title_width) / 2.0;
    let title_x1 = stencil_start + space;
    let title_x2 = title_x1 + title_width;

    // Left half: two strokes (y and y+2).
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(stencil_start),
        fmt_coord(title_x1),
        fmt_coord(y1),
        fmt_coord(y1),
    )
    .unwrap();
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(stencil_start),
        fmt_coord(title_x1),
        fmt_coord(y2),
        fmt_coord(y2),
    )
    .unwrap();

    // Title text: baseline sits 0.5px above the normal text baseline so it
    // matches Java's UHorizontalLine.drawTitleInternal offset.
    let text_y = y_baseline - 0.5;
    write_text_open(buf, title_x1, text_y, fill, None, outer_attrs, title_width);
    buf.push_str(&xml_escape(&trimmed_title));
    buf.push_str("</text>");

    // Right half: two strokes (y and y+2).
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(title_x2),
        fmt_coord(stencil_end),
        fmt_coord(y1),
        fmt_coord(y1),
    )
    .unwrap();
    write!(
        buf,
        r#"<line style="stroke:{stroke};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(title_x2),
        fmt_coord(stencil_end),
        fmt_coord(y2),
        fmt_coord(y2),
    )
    .unwrap();
}

fn line_has_sprites(spans: &[TextSpan]) -> bool {
    spans
        .iter()
        .any(|span| matches!(span, TextSpan::InlineSvg { .. }))
}

#[allow(clippy::too_many_arguments)]
fn render_text_line_with_sprites(
    buf: &mut String,
    spans: &[TextSpan],
    x: f64,
    y: f64,
    fill: &str,
    outer_attrs: &str,
    font_family: &str,
    font_size: f64,
    bold: bool,
    italic: bool,
) {
    use crate::render::svg_sprite;

    let gap = svg_sprite::sprite_text_gap(font_family, font_size, bold, italic);
    let ascent = font_metrics::ascent(font_family, font_size, bold, italic);
    let mut cursor_x = x;
    let mut pending_gap = false;
    let mut text_buf: Vec<TextSpan> = Vec::new();

    for span in spans {
        match span {
            TextSpan::InlineSvg { name, scale, color } => {
                if let Some(TextSpan::Plain(text)) = text_buf.last_mut() {
                    *text = text.trim_end().to_string();
                }
                let text_w =
                    measure_text_runs_width(&text_buf, font_family, font_size, bold, italic);
                if text_w > 0.0 {
                    render_prepared_line(
                        buf,
                        &text_buf,
                        cursor_x,
                        y,
                        fill,
                        None,
                        outer_attrs,
                        font_family,
                        font_size,
                        bold,
                        italic,
                    );
                    cursor_x += text_w;
                    pending_gap = true;
                }
                text_buf.clear();

                if let Some(svg_content) = get_sprite(name) {
                    let info = svg_sprite::sprite_info(&svg_content);
                    if info.vb_height > 0.0 {
                        if pending_gap {
                            cursor_x += gap;
                        }
                        let display_scale = sprite_scale_for_font(font_size, *scale);
                        let sprite_top_y = y - ascent;
                        let converted = svg_sprite::convert_svg_elements_scaled_with_options(
                            &svg_content,
                            cursor_x,
                            sprite_top_y,
                            display_scale,
                            color.as_deref(),
                        );
                        buf.push_str(&converted);
                        cursor_x += info.vb_width * display_scale;
                        pending_gap = true;
                    }
                }
            }
            other => {
                let adjusted = if pending_gap && text_buf.is_empty() {
                    match other {
                        TextSpan::Plain(text) => {
                            let trimmed = text.trim_start();
                            if trimmed.is_empty() {
                                continue;
                            }
                            cursor_x += gap;
                            pending_gap = false;
                            TextSpan::Plain(trimmed.to_string())
                        }
                        _ => {
                            cursor_x += gap;
                            pending_gap = false;
                            other.clone()
                        }
                    }
                } else {
                    other.clone()
                };
                text_buf.push(adjusted);
            }
        }
    }

    if !text_buf.is_empty() {
        render_prepared_line(
            buf,
            &text_buf,
            cursor_x,
            y,
            fill,
            None,
            outer_attrs,
            font_family,
            font_size,
            bold,
            italic,
        );
    }
}

fn render_line_with_sprites(
    buf: &mut String,
    spans: &[TextSpan],
    x: f64,
    y: f64,
    fill: &str,
    outer_attrs: &str,
) -> usize {
    use crate::render::svg_sprite;
    let (font_family, font_size, bold, italic) = parse_font_props(outer_attrs);
    let gap = svg_sprite::sprite_text_gap(&font_family, font_size, bold, italic);
    let arrow_y = y + 5.0659;
    let mut cursor_x = x;
    let mut in_sprite = false;
    let mut text_buf: Vec<TextSpan> = Vec::new();
    for span in spans {
        match span {
            TextSpan::InlineSvg { name, .. } => {
                if !text_buf.is_empty() {
                    if let Some(TextSpan::Plain(t)) = text_buf.last_mut() {
                        *t = t.trim_end().to_string();
                    }
                    let plain = plain_text_spans(&text_buf);
                    let text_w =
                        font_metrics::text_width(&plain, &font_family, font_size, bold, italic);
                    if !plain.is_empty() {
                        write_text_open(buf, cursor_x, y, fill, None, outer_attrs, text_w);
                        if text_buf.len() == 1 {
                            if let Some(t) = simple_plain_line(&text_buf) {
                                buf.push_str(&xml_escape(t));
                            } else {
                                render_spans(buf, &text_buf, &SpanStyle::default(), fill);
                            }
                        } else {
                            render_spans(buf, &text_buf, &SpanStyle::default(), fill);
                        }
                        buf.push_str("</text>");
                        cursor_x += text_w + gap;
                    }
                    text_buf.clear();
                }
                if let Some(svg_content) = get_sprite(name) {
                    let info = svg_sprite::sprite_info(&svg_content);
                    let sprite_y_offset = arrow_y - 2.0 - info.vb_height;
                    let converted =
                        svg_sprite::convert_svg_elements(&svg_content, cursor_x, sprite_y_offset);
                    buf.push_str(&converted);
                    cursor_x += info.vb_width + gap;
                }
                in_sprite = true;
            }
            _ => {
                if in_sprite && text_buf.is_empty() {
                    if let TextSpan::Plain(t) = span {
                        let trimmed = t.trim_start().to_string();
                        if !trimmed.is_empty() {
                            text_buf.push(TextSpan::Plain(trimmed));
                        }
                        in_sprite = false;
                        continue;
                    }
                }
                text_buf.push(span.clone());
                in_sprite = false;
            }
        }
    }
    if !text_buf.is_empty() {
        let plain = plain_text_spans(&text_buf);
        let text_w = font_metrics::text_width(&plain, &font_family, font_size, bold, italic);
        if !plain.is_empty() {
            write_text_open(buf, cursor_x, y, fill, None, outer_attrs, text_w);
            if text_buf.len() == 1 {
                if let Some(t) = simple_plain_line(&text_buf) {
                    buf.push_str(&xml_escape(t));
                } else {
                    render_spans(buf, &text_buf, &SpanStyle::default(), fill);
                }
            } else {
                render_spans(buf, &text_buf, &SpanStyle::default(), fill);
            }
            buf.push_str("</text>");
        }
    }
    1
}

fn line_needs_split_render(spans: &[TextSpan]) -> bool {
    fn has_styled(spans: &[TextSpan]) -> bool {
        spans.iter().any(|span| match span {
            TextSpan::Plain(_) | TextSpan::InlineSvg { .. } => false,
            TextSpan::Link { .. } => true,
            TextSpan::Bold(_)
            | TextSpan::Italic(_)
            | TextSpan::Underline(_)
            | TextSpan::UnderlineColored { .. }
            | TextSpan::Strikethrough(_)
            | TextSpan::Monospace(_)
            | TextSpan::BackHighlight { .. }
            | TextSpan::FontFamily { .. }
            | TextSpan::Colored { .. }
            | TextSpan::Sized { .. }
            | TextSpan::Subscript(_)
            | TextSpan::Superscript(_) => true,
        })
    }
    has_styled(spans)
}

fn multiline_line_needs_split_render(spans: &[TextSpan]) -> bool {
    if !line_needs_split_render(spans) {
        return false;
    }
    spans.len() > 1 || matches!(spans.first(), Some(TextSpan::Link { .. }))
}

/// A text run with full styling context for split rendering.
/// Java renders each styled atom as a separate `<text>` SVG element.
#[derive(Clone, Debug)]
struct TextRun {
    text: String,
    font_family: Option<String>,
    filter_id: Option<String>,
    bold: bool,
    italic: bool,
    underline: bool,
    underline_color: Option<String>,
    strikethrough: bool,
    color: Option<String>,
    font_size_override: Option<f64>,
    link_url: Option<String>,
    link_tooltip: Option<String>,
}

impl TextRun {
    fn new() -> Self {
        Self {
            text: String::new(),
            font_family: None,
            filter_id: None,
            bold: false,
            italic: false,
            underline: false,
            underline_color: None,
            strikethrough: false,
            color: None,
            font_size_override: None,
            link_url: None,
            link_tooltip: None,
        }
    }
    fn with_text(text: &str) -> Self {
        let mut r = Self::new();
        r.text = text.to_string();
        r
    }
    fn style_matches(&self, other: &RunStyle) -> bool {
        opt_eq(&self.font_family, &other.font_family)
            && opt_eq(&self.filter_id, &other.filter_id)
            && self.bold == other.bold
            && self.italic == other.italic
            && self.underline == other.underline
            && opt_eq(&self.underline_color, &other.underline_color)
            && self.strikethrough == other.strikethrough
            && opt_eq(&self.color, &other.color)
            && self.font_size_override == other.font_size_override
    }
}

fn opt_eq(a: &Option<String>, b: &Option<String>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

#[derive(Clone, Debug)]
struct RunStyle {
    font_family: Option<String>,
    filter_id: Option<String>,
    bold: bool,
    italic: bool,
    underline: bool,
    underline_color: Option<String>,
    strikethrough: bool,
    color: Option<String>,
    font_size_override: Option<f64>,
}

impl RunStyle {
    fn new() -> Self {
        Self {
            font_family: None,
            filter_id: None,
            bold: false,
            italic: false,
            underline: false,
            underline_color: None,
            strikethrough: false,
            color: None,
            font_size_override: None,
        }
    }
}

fn flatten_to_runs(spans: &[TextSpan]) -> Vec<TextRun> {
    let mut runs: Vec<TextRun> = Vec::new();
    flatten_span_runs(spans, &mut runs, &RunStyle::new());
    runs
}

fn flatten_span_runs(spans: &[TextSpan], runs: &mut Vec<TextRun>, style: &RunStyle) {
    for span in spans {
        match span {
            TextSpan::Plain(text) => {
                if let Some(run) = runs.last_mut() {
                    if run.style_matches(style) {
                        run.text.push_str(text);
                        continue;
                    }
                }
                let mut r = TextRun::with_text(text);
                r.font_family = style.font_family.clone();
                r.filter_id = style.filter_id.clone();
                r.bold = style.bold;
                r.italic = style.italic;
                r.underline = style.underline;
                r.underline_color = style.underline_color.clone();
                r.strikethrough = style.strikethrough;
                r.color = style.color.clone();
                r.font_size_override = style.font_size_override;
                runs.push(r);
            }
            TextSpan::BackHighlight { color, content } => {
                let fid = register_back_filter(color);
                let mut s = style.clone();
                s.filter_id = Some(fid);
                flatten_span_runs(content, runs, &s);
            }
            TextSpan::FontFamily { family, content } => {
                let mut s = style.clone();
                s.font_family = Some(family.clone());
                flatten_span_runs(content, runs, &s);
            }
            TextSpan::Bold(inner) => {
                let mut s = style.clone();
                s.bold = true;
                flatten_span_runs(inner, runs, &s);
            }
            TextSpan::Italic(inner) => {
                let mut s = style.clone();
                s.italic = true;
                flatten_span_runs(inner, runs, &s);
            }
            TextSpan::Underline(inner) => {
                let mut s = style.clone();
                s.underline = true;
                flatten_span_runs(inner, runs, &s);
            }
            TextSpan::UnderlineColored { color, content } => {
                let mut s = style.clone();
                s.underline = true;
                s.underline_color = Some(color.clone());
                flatten_span_runs(content, runs, &s);
            }
            TextSpan::Strikethrough(inner) => {
                let mut s = style.clone();
                s.strikethrough = true;
                flatten_span_runs(inner, runs, &s);
            }
            TextSpan::Colored { color, content } => {
                let mut s = style.clone();
                s.color = Some(color.clone());
                flatten_span_runs(content, runs, &s);
            }
            TextSpan::Sized { size, content } => {
                let mut s = style.clone();
                s.font_size_override = Some(*size);
                flatten_span_runs(content, runs, &s);
            }
            TextSpan::Subscript(inner) => {
                // Java: subscript uses font size × 0.77 (approximately 10/13)
                let base_size = style.font_size_override.unwrap_or(0.0);
                let sub_size = if base_size > 0.0 {
                    base_size * 0.77
                } else {
                    -1.0
                }; // Use -1 as marker for "subscript from default"
                let mut s = style.clone();
                s.font_size_override = Some(sub_size);
                flatten_span_runs(inner, runs, &s);
            }
            TextSpan::Superscript(inner) => {
                // Java: superscript uses font size × 0.77
                let base_size = style.font_size_override.unwrap_or(0.0);
                let sup_size = if base_size > 0.0 {
                    base_size * 0.77
                } else {
                    -2.0
                }; // Use -2 as marker for "superscript from default"
                let mut s = style.clone();
                s.font_size_override = Some(sup_size);
                flatten_span_runs(inner, runs, &s);
            }
            TextSpan::Monospace(text) => {
                let mut r = TextRun::with_text(text);
                r.font_family = Some("monospace".to_string());
                r.filter_id = style.filter_id.clone();
                r.bold = style.bold;
                r.italic = style.italic;
                r.underline = style.underline;
                r.underline_color = style.underline_color.clone();
                r.strikethrough = style.strikethrough;
                r.color = style.color.clone();
                r.font_size_override = style.font_size_override;
                runs.push(r);
            }
            TextSpan::Link {
                label,
                url,
                tooltip,
            } => {
                let visible = label.as_deref().unwrap_or(url);
                // Links always create a new run (they need <a> wrapping)
                let mut r = TextRun::with_text(visible);
                r.font_family = style.font_family.clone();
                r.filter_id = style.filter_id.clone();
                r.bold = style.bold;
                r.italic = style.italic;
                r.underline = true; // Links are underlined by default
                r.strikethrough = style.strikethrough;
                r.color = Some("#0000FF".to_string()); // Links are blue
                r.font_size_override = style.font_size_override;
                r.link_url = Some(url.clone());
                r.link_tooltip = tooltip.clone();
                runs.push(r);
            }
            TextSpan::InlineSvg { .. } => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_split_text_runs(
    buf: &mut String,
    spans: &[TextSpan],
    x: f64,
    y: f64,
    fill: &str,
    _outer_attrs: &str,
    default_font: &str,
    font_size: f64,
    base_bold: bool,
    base_italic: bool,
) {
    let runs = flatten_to_runs(spans);
    let mut cursor_x = x;

    for (idx, run) in runs.iter().enumerate() {
        let raw_text = &run.text;
        let run_font = run.font_family.as_deref().unwrap_or(default_font);
        let run_bold = run.bold || base_bold;
        let run_italic = run.italic || base_italic;
        // Handle subscript/superscript size markers
        let run_size = match run.font_size_override {
            Some(v) if v == -1.0 => (font_size * 0.77).round(), // subscript
            Some(v) if v == -2.0 => (font_size * 0.77).round(), // superscript
            // Heading sentinel: size = -100 - delta → effective = base + delta.
            Some(v) if v <= -100.0 => font_size + (-100.0 - v),
            Some(v) if v > 0.0 => v,
            _ => font_size,
        };
        if !raw_text.is_empty() && raw_text.trim().is_empty() {
            let n_spaces = raw_text.chars().count();
            let space_w = font_metrics::text_width(" ", run_font, run_size, run_bold, run_italic);
            render_nbsp_text(
                buf, cursor_x, y, n_spaces, fill, run_font, run_size, run_bold, run_italic,
            );
            cursor_x += space_w * n_spaces as f64;
            continue;
        }
        // Java: leading whitespace on non-first runs is stripped and converted
        // to cursor advancement. Trailing whitespace is also stripped.
        let trimmed_start = raw_text.trim_start();
        let n_leading = raw_text.len() - trimmed_start.len();
        if n_leading > 0 {
            let space_w = font_metrics::text_width(" ", run_font, run_size, run_bold, run_italic);
            if idx == 0 && run_font != "monospace" {
                render_nbsp_text(
                    buf, cursor_x, y, n_leading, fill, run_font, run_size, run_bold, run_italic,
                );
            }
            cursor_x += space_w * n_leading as f64;
        }
        let trimmed_both = trimmed_start.trim_end();
        let n_trailing = trimmed_start.len() - trimmed_both.len();
        let text = trimmed_both.to_string();
        if text.is_empty() {
            continue;
        }
        let run_fill_normalized;
        let run_fill = if let Some(ref c) = run.color {
            run_fill_normalized = crate::style::normalize_color(c);
            &run_fill_normalized
        } else {
            fill
        };
        // Java: for <size:N>, the y coordinate is adjusted (baseline shift).
        let run_y = if let Some(sz) = run.font_size_override {
            if sz == -1.0 {
                y + font_size * 0.2852
            } else if sz == -2.0 {
                y - font_size * 0.4071
            } else if sz > font_size {
                let desc_base = font_metrics::descent(default_font, font_size, false, false);
                let desc_large = font_metrics::descent(default_font, sz, false, false);
                y - (desc_large - desc_base)
            } else {
                y
            }
        } else {
            y
        };
        let text_w = font_metrics::text_width(&text, run_font, run_size, run_bold, run_italic);
        if let Some(ref url) = run.link_url {
            let title_src = run.link_tooltip.as_deref().unwrap_or(url);
            let title = process_xlink_title(title_src);
            write!(buf, r#"<a href="{}" target="_top" title="{}" xlink:actuate="onRequest" xlink:href="{}" xlink:show="new" xlink:title="{}" xlink:type="simple">"#,
                xml_escape_attr(url), xml_escape_attr(&title), xml_escape_attr(url), xml_escape_attr(&title)).unwrap();
        }
        write!(buf, r#"<text fill="{}""#, xml_escape(run_fill)).unwrap();
        if let Some(ref fid) = run.filter_id {
            write!(buf, r#" filter="url(#{fid})""#).unwrap();
        }
        write!(buf, r#" font-family="{}""#, xml_escape(run_font)).unwrap();
        write!(buf, r#" font-size="{}""#, fmt_coord(run_size)).unwrap();
        if run_italic {
            buf.push_str(r#" font-style="italic""#);
        }
        if run_bold {
            buf.push_str(r#" font-weight="bold""#);
        }
        write!(buf, r#" lengthAdjust="spacing""#).unwrap();
        if run.strikethrough {
            buf.push_str(r#" text-decoration="wavy underline""#);
        } else if run.underline && run.underline_color.is_none() {
            buf.push_str(r#" text-decoration="underline""#);
        }
        write!(buf, r#" textLength="{}""#, fmt_coord(text_w)).unwrap();
        write!(
            buf,
            r#" x="{}" y="{}">"#,
            fmt_coord(cursor_x),
            fmt_coord(run_y)
        )
        .unwrap();
        buf.push_str(&xml_escape(&text));
        buf.push_str("</text>");
        // Colored underline: render as a separate <line> element (Java behavior)
        if let Some(ref ul_color) = run.underline_color {
            let ul_color_hex = crate::klimt::color::resolve_color(ul_color)
                .map_or_else(|| format!("#{}", ul_color), |c| c.to_svg());
            let line_y = run_y + 1.0;
            write!(
                buf,
                r#"<line style="stroke:{};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                ul_color_hex,
                fmt_coord(cursor_x),
                fmt_coord(cursor_x + text_w),
                fmt_coord(line_y),
                fmt_coord(line_y)
            )
            .unwrap();
        }
        if run.link_url.is_some() {
            buf.push_str("</a>");
        }
        cursor_x += text_w;
        // Account for trailing whitespace that was stripped from the rendered text.
        if n_trailing > 0 {
            let space_w = font_metrics::text_width(" ", run_font, run_size, run_bold, run_italic);
            cursor_x += space_w * n_trailing as f64;
        }
    }
}

fn render_nbsp_text(
    buf: &mut String,
    x: f64,
    y: f64,
    n_spaces: usize,
    fill: &str,
    default_font: &str,
    font_size: f64,
    base_bold: bool,
    base_italic: bool,
) {
    if n_spaces == 0 {
        return;
    }
    let space_w = font_metrics::text_width(" ", default_font, font_size, false, false);
    let total_space_w = space_w * n_spaces as f64;
    let nbsp = "\u{00A0}".repeat(n_spaces);
    write!(buf, r#"<text fill="{}""#, xml_escape(fill)).unwrap();
    write!(buf, r#" font-family="{}""#, xml_escape(default_font)).unwrap();
    write!(buf, r#" font-size="{}""#, fmt_coord(font_size)).unwrap();
    if base_bold {
        buf.push_str(r#" font-weight="bold""#);
    }
    if base_italic {
        buf.push_str(r#" font-style="italic""#);
    }
    write!(buf, r#" lengthAdjust="spacing""#).unwrap();
    write!(buf, r#" textLength="{}""#, fmt_coord(total_space_w)).unwrap();
    write!(buf, r#" x="{}" y="{}">"#, fmt_coord(x), fmt_coord(y)).unwrap();
    buf.push_str(&xml_escape(&nbsp));
    buf.push_str("</text>");
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
                            // CSS: bold = 700; Java uses numeric weights >= 700 as bold
                            bold = value == "bold" || value.parse::<u32>().is_ok_and(|w| w >= 700);
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
    write!(buf, r#" x="{}" y="{}">"#, fmt_coord(x), fmt_coord(y)).unwrap();
}

fn flatten_rich_lines(rich: &RichText) -> Vec<Vec<TextSpan>> {
    let mut out = Vec::new();
    flatten_rich_lines_into(rich, &mut out);
    out
}

/// Tag produced alongside each flattened display line so the renderer can
/// distinguish ordinary text lines from Creole section titles that need
/// decorating horizontal lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayLineKind {
    Text,
    SectionTitle,
}

fn flatten_rich_display_lines(rich: &RichText) -> Vec<DisplayLineKind> {
    let mut out = Vec::new();
    flatten_rich_display_lines_into(rich, &mut out);
    out
}

fn flatten_rich_display_lines_into(rich: &RichText, out: &mut Vec<DisplayLineKind>) {
    match rich {
        RichText::Line(_) => out.push(DisplayLineKind::Text),
        RichText::Block(items) => {
            for item in items {
                flatten_rich_display_lines_into(item, out);
            }
        }
        RichText::BulletList(items) | RichText::NumberedList(items) => {
            for item in items {
                let kinds = flatten_rich_display_lines(item);
                // Mirror `flatten_rich_lines_into`: bullet/numbered lists
                // add each sub-line as-is (without special tagging).  Any
                // nested section title keeps its SectionTitle kind.
                out.extend(kinds);
            }
        }
        RichText::Table { headers, rows } => {
            if !headers.is_empty() {
                out.push(DisplayLineKind::Text);
            }
            for _ in rows {
                out.push(DisplayLineKind::Text);
            }
        }
        RichText::HorizontalRule => out.push(DisplayLineKind::Text),
        RichText::SectionTitle(_) => out.push(DisplayLineKind::SectionTitle),
    }
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
                out.push(join_multiline_cells(headers));
            }
            for row in rows {
                out.push(join_multiline_cells(row));
            }
        }
        RichText::HorizontalRule => out.push(vec![TextSpan::Plain("----".to_string())]),
        // Section title flattens to just the title text.  The decorating
        // horizontal lines are drawn by the dedicated section-title render
        // path (see `render_section_title_line`); non-rendering callsites
        // such as width measurement only need the text content here.
        RichText::SectionTitle(spans) => out.push(spans.clone()),
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

#[allow(dead_code)] // reserved for creole table cell joining
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

/// Like `join_cells` but each cell is a `Vec<Vec<TextSpan>>` (lines).
/// Joins the cell's sub-lines with a single space when flattening to a
/// single display line.
fn join_multiline_cells(cells: &[Vec<Vec<TextSpan>>]) -> Vec<TextSpan> {
    let mut line = Vec::new();
    for (idx, cell) in cells.iter().enumerate() {
        if idx > 0 {
            line.push(TextSpan::Plain(" | ".to_string()));
        }
        for (li, sub) in cell.iter().enumerate() {
            if li > 0 {
                line.push(TextSpan::Plain(" ".to_string()));
            }
            line.extend(sub.clone());
        }
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

fn trimmed_plain_line_text(spans: &[TextSpan]) -> String {
    plain_text_spans(spans).trim().to_string()
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
        TextSpan::UnderlineColored { content, .. }
        | TextSpan::Colored { content, .. }
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
        TextSpan::UnderlineColored { color: _, content } => {
            // Java renders <u:COLOR> as underline with a separate colored line,
            // but for now we use text-decoration underline (like plain <u>).
            render_spans(
                buf,
                content,
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
            // Heading sentinel: size = -100 - delta → effective = ambient base + delta.
            // Java StripeSimple.fontConfigurationForHeading uses `bigger(N)`
            // which is relative to the current font-config size.  We read
            // the ambient size set by `render_prepared_line` and add the
            // encoded delta.  If no ambient size is known, fall back to the
            // 14pt default used by the class/component pipeline.
            let effective = if *size <= -100.0 {
                let delta = -100.0 - *size;
                let base = style.font_size.or(style.ambient_font_size).unwrap_or(14.0);
                base + delta
            } else {
                *size
            };
            style.font_size = Some(effective);
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
        write!(attrs, r#" font-size="{}""#, fmt_coord(font_size)).unwrap();
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
#[allow(dead_code)] // reserved for deferred sprite rendering
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
                r#"<g transform="translate({},{}) scale({scale:.4})">{svg}</g>"#,
                fmt_coord(sprite_x),
                fmt_coord(sprite_y),
            )
            .unwrap();
            offset_x += display_w + 4.0;
        }
    }
}

/// Parse `viewBox` attribute from an SVG element to extract width and height.
#[allow(dead_code)] // reserved for SVG viewBox parsing
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

#[allow(dead_code)] // used by parse_viewbox
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
        assert!(buf.contains("font-weight"));
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
        // Java renders each line as a separate <text> element
        assert_eq!(buf.matches("<text ").count(), 2);
        assert_eq!(buf.matches("<tspan").count(), 0);
    }

    #[test]
    fn renders_empty_line_as_nbsp() {
        let mut buf = String::new();
        let lines = render_creole_text(&mut buf, "", 10.0, 20.0, 16.0, "#000000", None, "");
        assert_eq!(lines, 1);
        assert!(buf.contains("&#160;"));
        assert!(!buf.contains(r#"textLength="0""#));
    }

    #[test]
    fn renders_blank_table_cell_line_as_nbsp() {
        let mut buf = String::new();
        render_creole_display_lines(
            &mut buf,
            &[String::from("| \\nvalue | x |")],
            10.0,
            20.0,
            "#000000",
            r#"font-size="12""#,
            false,
        );
        assert!(buf.contains("&#160;"));
        assert!(!buf.contains(r#"textLength="0""#));
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
        // Java puts tooltip in title="..." and xlink:title="..." attributes, not <title> element
        assert!(buf.contains(r#"title="hover""#));
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
        // Split rendering: each piece is a separate <text> element
        assert!(buf.contains(">H<"), "should contain H text");
        assert!(buf.contains(">2<"), "should contain subscript 2");
        assert!(buf.contains(">O<"), "should contain O text");
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
        // Split rendering: each piece is a separate <text> element
        assert!(buf.contains(">E = mc<"), "should contain 'E = mc' text");
        assert!(buf.contains(">2<"), "should contain superscript 2");
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
        assert!(buf.contains(r#"filter="url(#"#));
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
            r#"<svg viewBox="0 0 100 50"><ellipse cx="50" cy="25" rx="10" ry="5" fill="red"/></svg>"#
                .to_string(),
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
            buf.contains("fill=\"red\"") || buf.contains("fill=\"#FF0000\""),
            "sprite SVG content must be embedded"
        );
        assert!(
            buf.contains("<ellipse") || buf.contains("<path"),
            "sprite primitives must be embedded directly"
        );
        assert!(
            !buf.contains("<svg"),
            "inline sprite must not emit nested svg wrappers"
        );

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
        assert_eq!(buf.matches("<text ").count(), 1);
        assert!(buf.contains(">plain text</text>"));
        assert!(!buf.contains("<g transform="));
    }

    #[test]
    fn renders_plain_line_as_single_text_element() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "This is a note",
            41.0,
            226.8276,
            15.1328,
            "#000000",
            None,
            r#"font-size="13""#,
        );
        assert_eq!(buf.matches("<text ").count(), 1);
        assert!(buf.contains(">This is a note</text>"));
    }

    #[test]
    fn centered_plain_line_keeps_single_text_element() {
        let mut buf = String::new();
        render_creole_text(
            &mut buf,
            "plain text",
            10.0,
            20.0,
            16.0,
            "#000000",
            Some("middle"),
            "",
        );
        assert_eq!(buf.matches("<text ").count(), 1);
        assert!(buf.contains(">plain text</text>"));
    }

    #[test]
    fn parse_viewbox_basic() {
        assert_eq!(
            parse_viewbox(r#"<svg viewBox="0 0 200 100"><rect/></svg>"#),
            (200.0, 100.0)
        );
    }
}
