// klimt::svg - SVG output driver
// Port of Java PlantUML's klimt.drawing.svg package
//
// Core types:
// - SvgGraphic: low-level SVG element generation (ports SvgGraphics.java)
// - UGraphicSvg: UGraphic trait implementation for SVG output

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Write;

use super::color::HColor;
use super::font::StringBounder;
use super::geom::USegmentType;
use super::shape::UPath;
use super::{UChange, UParam, UStroke, UTranslate};

const PLANTUML_VERSION: &str = "1.2026.2";

thread_local! {
    static SVG_ID_SEED_OVERRIDE: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Java: `StringUtils.seed(source.getPlainString("\n"))`.
///
/// Java's `UmlSource.getPlainString("\n")` iterates over the diagram lines
/// held in the `source` list (after the preprocessor has dropped comment
/// lines that begin with a single quote) and appends each line followed by
/// `\n`. The resulting string is hashed with `h = 31 * h + ch`, starting
/// from the Java prime `1125899906842597L`.
///
/// To match byte-exact, we:
/// 1. Split the raw source on `\n` and drop the trailing empty segment that
///    `split` produces for a file ending in `\n`.
/// 2. Skip lines whose first non-whitespace character is `'` (Java's
///    single-line comment marker).
/// 3. For each surviving line, fold each character into `h`, then fold a
///    trailing `\n` regardless of whether the original file had one after
///    that line.
pub fn java_source_seed(source: &str) -> i64 {
    let mut h: i64 = 1125899906842597;
    let mut lines: Vec<&str> = source.split('\n').collect();
    if matches!(lines.last(), Some(&"")) {
        // split yields a trailing empty segment when source ends with '\n'
        lines.pop();
    }
    for line in lines {
        if line.trim_start().starts_with('\'') {
            continue;
        }
        for ch in line.chars() {
            h = h.wrapping_mul(31).wrapping_add(ch as i64);
        }
        h = h.wrapping_mul(31).wrapping_add('\n' as i64);
    }
    h
}

/// Override SVG id generation so filter/shadow ids use the Java source seed.
pub fn set_svg_id_seed_override(seed: Option<i64>) {
    SVG_ID_SEED_OVERRIDE.with(|cell| cell.set(seed.map(i64::unsigned_abs)));
}

pub fn current_shadow_id() -> String {
    format!("f{}", current_seed_string())
}

pub fn current_filter_uid_prefix() -> String {
    format!("b{}", current_seed_string())
}

fn current_seed_string() -> String {
    let seed = SVG_ID_SEED_OVERRIDE.with(|cell| cell.get()).unwrap_or(0);
    SvgGraphic::format_seed(seed)
}

// ── Number formatting ───────────────────────────────────────────────

/// Format a coordinate value at scale 1.0 (convenience for renderers).
pub fn fmt_coord(value: f64) -> String {
    fmt(value, 1.0)
}

/// Format a coordinate value matching Java PlantUML's `SvgGraphics.format()`:
/// - `String.format(Locale.US, "%.4f", x * scale)`
/// - Trailing zeros stripped after decimal point
/// - If only zeros remain after dot, the dot is stripped too
/// - "0" for zero
fn fmt(value: f64, scale: f64) -> String {
    let x = value * scale;
    if x == 0.0 {
        return "0".into();
    }
    let rounded = java_round_4(x);
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

/// Round to 4 decimal places using Java's half-up rounding.
fn java_round_4(v: f64) -> f64 {
    let factor = 10000.0_f64;
    let scaled = v * factor;
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5).floor()
    } else {
        (scaled - 0.5).ceil()
    };
    rounded / factor
}

/// Format a boolean flag for SVG path arc: 0 or 1.
fn fmt_bool(x: f64) -> &'static str {
    if x == 0.0 {
        "0"
    } else {
        "1"
    }
}

/// XML-escape text content matching Java's DOM serializer (us-ascii encoding).
/// XML-escape text content. Java PlantUML only escapes &, <, > in text content.
/// Double quotes are NOT escaped (safe inside text elements, unlike attribute values).
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c if !c.is_ascii() => {
                write!(out, "&#{};", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out
}

/// XML-escape a string for use in attribute values.
/// Like `xml_escape` but also escapes `"` → `&quot;`, `\n` → `&#10;`,
/// `\r` → `&#13;`, `\t` → `&#9;` (matching Java's XML serializer).
pub fn xml_escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            c if !c.is_ascii() => {
                write!(out, "&#{};", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out
}

/// Match Java DOM comment serialization under `us-ascii`: keep ASCII bytes
/// verbatim and replace non-ASCII codepoints with `?`.
pub fn svg_comment_escape(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii() { c } else { '?' })
        .collect()
}

// ── SvgGraphic ──────────────────────────────────────────────────────

/// Low-level SVG element generator.
///
/// Port of Java PlantUML's `SvgGraphics` class.
/// Accumulates SVG elements into a string buffer, tracking current
/// fill/stroke/stroke-width/stroke-dasharray state.
///
/// The Java version uses a DOM Document, but since Java's XML serializer
/// outputs attributes in alphabetical order, we replicate that by writing
/// attributes in sorted order directly.
pub struct SvgGraphic {
    /// Accumulated SVG body elements (inside `<g>`)
    buf: String,
    /// Definitions block (`<defs>`)
    defs: String,

    // Current drawing state
    fill: String,
    stroke: String,
    stroke_width: String,
    stroke_dasharray: Option<String>,

    // Visibility tracking
    max_x: i32,
    max_y: i32,

    // Shadow support
    shadow_id: String,
    with_shadow: bool,

    // Gradient ID generation
    gradient_id_prefix: String,
    gradients: HashMap<String, String>,
    gradient_count: usize,

    // Filter support for text background
    filter_uid: String,
    filter_back_colors: HashMap<String, String>,
    filter_count: usize,

    // Hidden flag
    hidden: bool,

    // Scale factor
    scale: f64,

    // Group/link stack
    group_stack: Vec<GroupEntry>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // variant fields stored for future group/link context
enum GroupEntry {
    Group(String),
    Link(String),
}

impl SvgGraphic {
    /// Create a new SvgGraphic with the given seed for unique IDs.
    pub fn new(seed: u64, scale: f64) -> Self {
        let seed = if seed == 0 {
            SVG_ID_SEED_OVERRIDE.with(|cell| cell.get()).unwrap_or(0)
        } else {
            seed
        };
        let seed_str = Self::format_seed(seed);
        Self {
            buf: String::with_capacity(4096),
            defs: String::new(),
            fill: "black".into(),
            stroke: "black".into(),
            stroke_width: "1".into(),
            stroke_dasharray: None,
            max_x: 10,
            max_y: 10,
            shadow_id: format!("f{}", seed_str),
            with_shadow: false,
            gradient_id_prefix: format!("g{}", seed_str),
            gradients: HashMap::new(),
            gradient_count: 0,
            filter_uid: format!("b{}", seed_str),
            filter_back_colors: HashMap::new(),
            filter_count: 0,
            hidden: false,
            scale,
            group_stack: Vec::new(),
        }
    }

    fn format_seed(seed: u64) -> String {
        // Java: Long.toString(Math.abs(seed), 36)
        if seed == 0 {
            return "0".into();
        }
        let mut n = seed;
        let mut chars = Vec::new();
        while n > 0 {
            let digit = (n % 36) as u8;
            chars.push(if digit < 10 {
                b'0' + digit
            } else {
                b'a' + (digit - 10)
            });
            n /= 36;
        }
        chars.reverse();
        String::from_utf8(chars).unwrap()
    }

    /// Format a number with the current scale applied.
    pub fn f(&self, value: f64) -> String {
        fmt(value, self.scale)
    }

    /// Return the tracked maximum coordinates (Java `SvgGraphics.maxX/maxY`).
    pub fn max_dimensions(&self) -> (i32, i32) {
        (self.max_x, self.max_y)
    }

    /// Manually track a bounding rectangle for elements written via `push_raw`.
    /// Java: every draw operation calls `ensureVisible`; `push_raw` bypasses
    /// this, so callers must invoke this for any raw-SVG elements.
    pub fn track_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.ensure_visible(x + w, y + h);
    }

    /// Track a text element's extent (x, y baseline, textLength).
    pub fn track_text(&mut self, x: f64, y: f64, text_length: f64) {
        self.ensure_visible(x, y);
        self.ensure_visible(x + text_length, y);
    }

    fn ensure_visible(&mut self, x: f64, y: f64) {
        let xi = (x + 1.0) as i32;
        let yi = (y + 1.0) as i32;
        if xi > self.max_x {
            self.max_x = xi;
        }
        if yi > self.max_y {
            self.max_y = yi;
        }
    }

    fn fix_color(color: Option<&str>) -> &str {
        match color {
            None | Some("#00000000") => "none",
            Some(c) => c,
        }
    }

    /// Parse a gradient color string like `#AAFFAA/#55AA55` into
    /// `(color1_svg, color2_svg, policy)`. Gradient separators: `/`, `|`,
    /// `-`, `\`. Returns `None` for plain colors.
    ///
    /// Java: `HColorGradient` stores two `HColor` values plus a policy
    /// char. The policy character equals the separator used in the
    /// skinparam value (`/` = diagonal, `|` = horizontal, `-` = vertical,
    /// `\` = reverse diagonal).
    fn parse_gradient_str(s: &str) -> Option<(String, String, char)> {
        // Skip already-resolved values
        if s.starts_with("url(") || s == "none" || s.is_empty() {
            return None;
        }
        let raw = s.strip_prefix('#').unwrap_or(s);
        for (i, ch) in raw.char_indices() {
            if (ch == '/' || ch == '|' || ch == '-' || ch == '\\') && i > 0 {
                let left = &raw[..i];
                let right = &raw[i + ch.len_utf8()..];
                // Validate both sides look like colors (hex or named)
                if !left.is_empty() && !right.is_empty() {
                    let c1 = super::color::resolve_color(left)?;
                    let c2 = super::color::resolve_color(right)?;
                    return Some((c1.to_svg(), c2.to_svg(), ch));
                }
            }
        }
        None
    }

    // ── State setters ───────────────────────────────────────────────

    pub fn set_fill_color(&mut self, fill: &str) {
        let fixed = Self::fix_color(Some(fill));
        if let Some((c1, c2, policy)) = Self::parse_gradient_str(fixed) {
            let id = self.create_svg_gradient(&c1, &c2, policy);
            self.fill = format!("url(#{})", id);
        } else {
            self.fill = fixed.to_string();
        }
    }

    pub fn set_fill_color_with_opacity(&mut self, fill: &str) {
        // WITH_FILL_OPACITY: don't fix transparent to "none"
        self.fill = fill.to_string();
    }

    pub fn set_stroke_color(&mut self, stroke: Option<&str>) {
        let fixed = Self::fix_color(stroke);
        if let Some((c1, c2, policy)) = Self::parse_gradient_str(fixed) {
            let id = self.create_svg_gradient(&c1, &c2, policy);
            self.stroke = format!("url(#{})", id);
        } else {
            self.stroke = fixed.to_string();
        }
    }

    pub fn set_stroke_width(&mut self, width: f64, dasharray: Option<(f64, f64)>) {
        self.stroke_width = self.f(width);
        self.stroke_dasharray = dasharray.map(|(a, b)| format!("{},{}", self.f(a), self.f(b)));
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    // ── Fill attribute emission ─────────────────────────────────────

    /// Write fill/fill-opacity attributes matching Java's fillMe().
    fn write_fill(&self, buf: &mut String) {
        if self.fill.len() == 9 && self.fill.starts_with('#') {
            // #RRGGBBAA format: split into fill + fill-opacity
            let color = &self.fill[..7];
            let alpha = u8::from_str_radix(&self.fill[7..9], 16).unwrap_or(255);
            let opacity = alpha as f64 / 255.0;
            write!(buf, " fill=\"{}\"", color).unwrap();
            write!(buf, " fill-opacity=\"{:.5}\"", opacity).unwrap();
        } else {
            write!(buf, " fill=\"{}\"", self.fill).unwrap();
        }
    }

    /// Write style attribute matching Java's styleMe().
    fn write_style(&self, buf: &mut String) {
        if self.stroke_width == "0" {
            return;
        }
        let mut style = format!("stroke:{};stroke-width:{};", self.stroke, self.stroke_width);
        if let Some(ref da) = self.stroke_dasharray {
            write!(style, "stroke-dasharray:{};", da).unwrap();
        }
        write!(buf, " style=\"{}\"", style).unwrap();
    }

    fn write_shadow_filter(&self, buf: &mut String, delta_shadow: f64) {
        if delta_shadow > 0.0 {
            write!(buf, " filter=\"url(#{})\"", self.shadow_id).unwrap();
        }
    }

    // ── Shadow management ───────────────────────────────────────────

    fn manage_shadow(&mut self, delta_shadow: f64) {
        if delta_shadow != 0.0 && !self.with_shadow {
            write!(
                self.defs,
                "<filter height=\"300%\" id=\"{}\" width=\"300%\" x=\"-1\" y=\"-1\">",
                self.shadow_id
            )
            .unwrap();
            write!(
                self.defs,
                "<feGaussianBlur result=\"blurOut\" stdDeviation=\"{}\"/>",
                self.f(2.0)
            )
            .unwrap();
            write!(
                self.defs,
                "<feColorMatrix in=\"blurOut\" result=\"blurOut2\" type=\"matrix\" values=\"0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 .4 0\"/>",
            )
            .unwrap();
            write!(
                self.defs,
                "<feOffset dx=\"{}\" dy=\"{}\" in=\"blurOut2\" result=\"blurOut3\"/>",
                self.f(4.0),
                self.f(4.0)
            )
            .unwrap();
            write!(
                self.defs,
                "<feBlend in=\"SourceGraphic\" in2=\"blurOut3\" mode=\"normal\"/>",
            )
            .unwrap();
            write!(self.defs, "</filter>").unwrap();
            self.with_shadow = true;
        }
    }

    // ── Gradient support ────────────────────────────────────────────

    /// Create an SVG gradient definition, returning the gradient ID.
    /// Java: SvgGraphics.createSvgGradient(String, String, char)
    pub fn create_svg_gradient(&mut self, color1: &str, color2: &str, policy: char) -> String {
        let key = format!("{}:{}:{}", color1, color2, policy);
        if let Some(id) = self.gradients.get(&key) {
            return id.clone();
        }

        let id = format!("{}{}", self.gradient_id_prefix, self.gradient_count);
        self.gradient_count += 1;

        let (x1, y1, x2, y2) = match policy {
            '|' => ("0%", "50%", "100%", "50%"),
            '\\' => ("0%", "100%", "100%", "0%"),
            '-' => ("50%", "0%", "50%", "100%"),
            _ => ("0%", "0%", "100%", "100%"),
        };

        write!(
            self.defs,
            "<linearGradient id=\"{}\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\">",
            id, x1, x2, y1, y2
        )
        .unwrap();
        write!(self.defs, "<stop offset=\"0%\" stop-color=\"{}\"/>", color1).unwrap();
        write!(
            self.defs,
            "<stop offset=\"100%\" stop-color=\"{}\"/>",
            color2
        )
        .unwrap();
        write!(self.defs, "</linearGradient>").unwrap();

        self.gradients.insert(key, id.clone());
        id
    }

    // ── Text background filter ──────────────────────────────────────

    fn get_filter_back_color(&mut self, color: &str) -> String {
        if let Some(id) = self.filter_back_colors.get(color) {
            return id.clone();
        }

        let id = format!("{}{}", self.filter_uid, self.filter_count);
        self.filter_count += 1;

        write!(
            self.defs,
            "<filter height=\"1\" id=\"{}\" width=\"1\" x=\"0\" y=\"0\">",
            id
        )
        .unwrap();
        write!(
            self.defs,
            "<feFlood flood-color=\"{}\" result=\"flood\"/>",
            color
        )
        .unwrap();
        write!(
            self.defs,
            "<feComposite in=\"SourceGraphic\" in2=\"flood\" operator=\"over\"/>",
        )
        .unwrap();
        write!(self.defs, "</filter>").unwrap();

        self.filter_back_colors
            .insert(color.to_string(), id.clone());
        id
    }

    // ── SVG shape emission methods ──────────────────────────────────

    /// Emit a `<rect>` element.
    /// Java: SvgGraphics.svgRectangle()
    ///
    /// Attribute order matches Java DOM alphabetical output:
    /// fill, [fill-opacity], [filter], height, [rx], [ry], [style], width, x, y
    pub fn svg_rectangle(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        rx: f64,
        ry: f64,
        delta_shadow: f64,
    ) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }
        log::trace!(
            "svg_rectangle: x={}, y={}, w={}, h={}, rx={}, ry={}, fill={}, stroke={}",
            x,
            y,
            width,
            height,
            rx,
            ry,
            self.fill,
            self.stroke
        );
        self.manage_shadow(delta_shadow);
        if !self.hidden {
            let mut elt = String::with_capacity(128);
            elt.push_str("<rect");
            self.write_fill(&mut elt);
            self.write_shadow_filter(&mut elt, delta_shadow);
            write!(elt, " height=\"{}\"", self.f(height)).unwrap();
            if rx > 0.0 && ry > 0.0 {
                write!(elt, " rx=\"{}\"", self.f(rx)).unwrap();
                write!(elt, " ry=\"{}\"", self.f(ry)).unwrap();
            }
            self.write_style(&mut elt);
            write!(elt, " width=\"{}\"", self.f(width)).unwrap();
            write!(elt, " x=\"{}\"", self.f(x)).unwrap();
            write!(elt, " y=\"{}\"", self.f(y)).unwrap();
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
        self.ensure_visible(
            x + width + 2.0 * delta_shadow,
            y + height + 2.0 * delta_shadow,
        );
    }

    /// Emit an `<ellipse>` element.
    /// Java: SvgGraphics.svgEllipse()
    ///
    /// Attribute order: cx, cy, fill, [fill-opacity], [filter], rx, ry, [style]
    pub fn svg_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, delta_shadow: f64) {
        self.manage_shadow(delta_shadow);
        if !self.hidden {
            let mut elt = String::with_capacity(128);
            elt.push_str("<ellipse");
            write!(elt, " cx=\"{}\"", self.f(cx)).unwrap();
            write!(elt, " cy=\"{}\"", self.f(cy)).unwrap();
            self.write_fill(&mut elt);
            self.write_shadow_filter(&mut elt, delta_shadow);
            write!(elt, " rx=\"{}\"", self.f(rx)).unwrap();
            write!(elt, " ry=\"{}\"", self.f(ry)).unwrap();
            self.write_style(&mut elt);
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
        self.ensure_visible(cx + rx + delta_shadow * 2.0, cy + ry + delta_shadow * 2.0);
    }

    /// Emit an SVG arc ellipse path.
    /// Java: SvgGraphics.svgArcEllipse()
    pub fn svg_arc_ellipse(&mut self, rx: f64, ry: f64, x1: f64, y1: f64, x2: f64, y2: f64) {
        if !self.hidden {
            let d = format!(
                "M{},{} A{},{} 0 0 0 {} {}",
                self.f(x1),
                self.f(y1),
                self.f(rx),
                self.f(ry),
                self.f(x2),
                self.f(y2)
            );
            let mut elt = String::with_capacity(128);
            elt.push_str("<path");
            write!(elt, " d=\"{}\"", d).unwrap();
            self.write_fill(&mut elt);
            self.write_style(&mut elt);
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
        self.ensure_visible(x1, y1);
        self.ensure_visible(x2, y2);
    }

    /// Emit a `<line>` element.
    /// Java: SvgGraphics.svgLine()
    ///
    /// Attribute order: [filter], [style], x1, x2, y1, y2
    pub fn svg_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, delta_shadow: f64) {
        log::trace!(
            "svg_line: x1={}, y1={}, x2={}, y2={}, stroke={}",
            x1,
            y1,
            x2,
            y2,
            self.stroke
        );
        self.manage_shadow(delta_shadow);
        if !self.hidden {
            let mut elt = String::with_capacity(128);
            elt.push_str("<line");
            self.write_shadow_filter(&mut elt, delta_shadow);
            self.write_style(&mut elt);
            write!(elt, " x1=\"{}\"", self.f(x1)).unwrap();
            write!(elt, " x2=\"{}\"", self.f(x2)).unwrap();
            write!(elt, " y1=\"{}\"", self.f(y1)).unwrap();
            write!(elt, " y2=\"{}\"", self.f(y2)).unwrap();
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
        self.ensure_visible(x1 + 2.0 * delta_shadow, y1 + 2.0 * delta_shadow);
        self.ensure_visible(x2 + 2.0 * delta_shadow, y2 + 2.0 * delta_shadow);
    }

    /// Emit a `<polygon>` element.
    /// Java: SvgGraphics.svgPolygon()
    ///
    /// Points are given as a flat array [x0,y0,x1,y1,...].
    /// Attribute order: fill, [fill-opacity], [filter], points, [style]
    pub fn svg_polygon(&mut self, delta_shadow: f64, points: &[f64]) {
        debug_assert!(points.len() % 2 == 0);
        self.manage_shadow(delta_shadow);
        if !self.hidden {
            let mut pts = String::with_capacity(points.len() * 8);
            for (i, coord) in points.iter().enumerate() {
                if i > 0 {
                    pts.push(',');
                }
                pts.push_str(&self.f(*coord));
            }
            let mut elt = String::with_capacity(128);
            elt.push_str("<polygon");
            self.write_fill(&mut elt);
            self.write_shadow_filter(&mut elt, delta_shadow);
            write!(elt, " points=\"{}\"", pts).unwrap();
            self.write_style(&mut elt);
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
        for i in (0..points.len()).step_by(2) {
            self.ensure_visible(
                points[i] + 2.0 * delta_shadow,
                points[i + 1] + 2.0 * delta_shadow,
            );
        }
    }

    /// Emit a `<circle>` element.
    ///
    /// Attribute order (alphabetical): cx, cy, fill, [fill-opacity], [filter], r, [style]
    pub fn svg_circle(&mut self, cx: f64, cy: f64, r: f64, delta_shadow: f64) {
        self.manage_shadow(delta_shadow);
        if !self.hidden {
            let mut elt = String::with_capacity(128);
            elt.push_str("<circle");
            write!(elt, " cx=\"{}\"", self.f(cx)).unwrap();
            write!(elt, " cy=\"{}\"", self.f(cy)).unwrap();
            self.write_fill(&mut elt);
            self.write_shadow_filter(&mut elt, delta_shadow);
            write!(elt, " r=\"{}\"", self.f(r)).unwrap();
            self.write_style(&mut elt);
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
        self.ensure_visible(cx + r + delta_shadow * 2.0, cy + r + delta_shadow * 2.0);
    }

    /// Emit a `<polyline>` element.
    ///
    /// Points are given as a flat array [x0,y0,x1,y1,...].
    /// Attribute order (alphabetical): fill, [fill-opacity], points, [style]
    pub fn svg_polyline(&mut self, points: &[f64]) {
        debug_assert!(points.len() % 2 == 0);
        if !self.hidden {
            let mut pts = String::with_capacity(points.len() * 8);
            for (i, coord) in points.iter().enumerate() {
                if i > 0 {
                    pts.push(',');
                }
                pts.push_str(&self.f(*coord));
            }
            let mut elt = String::with_capacity(128);
            elt.push_str("<polyline");
            self.write_fill(&mut elt);
            write!(elt, " points=\"{}\"", pts).unwrap();
            self.write_style(&mut elt);
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
        for i in (0..points.len()).step_by(2) {
            self.ensure_visible(points[i], points[i + 1]);
        }
    }

    /// Emit a `<path>` element from a UPath.
    /// Java: SvgGraphics.svgPath()
    ///
    /// Attribute order: [codeLine], d, fill, [fill-opacity], [filter], [id], [style]
    pub fn svg_path(&mut self, x: f64, y: f64, path: &UPath, delta_shadow: f64) {
        self.manage_shadow(delta_shadow);
        self.ensure_visible(x, y);

        let mut d = String::with_capacity(path.segments.len() * 12);
        for seg in &path.segments {
            let c = &seg.coords;
            match seg.kind {
                USegmentType::MoveTo => {
                    write!(d, "M{},{} ", self.f(c[0] + x), self.f(c[1] + y)).unwrap();
                    self.ensure_visible(
                        c[0] + x + 2.0 * delta_shadow,
                        c[1] + y + 2.0 * delta_shadow,
                    );
                }
                USegmentType::LineTo => {
                    write!(d, "L{},{} ", self.f(c[0] + x), self.f(c[1] + y)).unwrap();
                    self.ensure_visible(
                        c[0] + x + 2.0 * delta_shadow,
                        c[1] + y + 2.0 * delta_shadow,
                    );
                }
                USegmentType::CubicTo => {
                    write!(
                        d,
                        "C{},{} {},{} {},{} ",
                        self.f(c[0] + x),
                        self.f(c[1] + y),
                        self.f(c[2] + x),
                        self.f(c[3] + y),
                        self.f(c[4] + x),
                        self.f(c[5] + y)
                    )
                    .unwrap();
                    self.ensure_visible(
                        c[0] + x + 2.0 * delta_shadow,
                        c[1] + y + 2.0 * delta_shadow,
                    );
                    self.ensure_visible(
                        c[2] + x + 2.0 * delta_shadow,
                        c[3] + y + 2.0 * delta_shadow,
                    );
                    self.ensure_visible(
                        c[4] + x + 2.0 * delta_shadow,
                        c[5] + y + 2.0 * delta_shadow,
                    );
                }
                USegmentType::ArcTo => {
                    write!(
                        d,
                        "A{},{} {} {} {} {},{} ",
                        self.f(c[0]),
                        self.f(c[1]),
                        self.f(c[2]),
                        fmt_bool(c[3]),
                        fmt_bool(c[4]),
                        self.f(c[5] + x),
                        self.f(c[6] + y)
                    )
                    .unwrap();
                    self.ensure_visible(
                        c[5] + c[0] + x + 2.0 * delta_shadow,
                        c[6] + c[1] + y + 2.0 * delta_shadow,
                    );
                }
                USegmentType::Close => {
                    // Java: nothing emitted for SEG_CLOSE
                }
            }
        }

        if !self.hidden {
            let d_trimmed = d.trim_end();
            let mut elt = String::with_capacity(128);
            elt.push_str("<path");
            if let Some(ref comment) = path.comment {
                write!(elt, " codeLine=\"{}\"", xml_escape(comment)).unwrap();
            }
            write!(elt, " d=\"{}\"", d_trimmed).unwrap();
            self.write_fill(&mut elt);
            self.write_shadow_filter(&mut elt, delta_shadow);
            self.write_style(&mut elt);
            elt.push_str("/>");
            self.buf.push_str(&elt);
        }
    }

    /// Emit a `<text>` element.
    /// Java: SvgGraphics.text()
    ///
    /// Attribute order (alphabetical):
    /// fill, [fill-opacity], [filter], [font-family], [font-size], [font-style],
    /// [font-weight], [lengthAdjust], [style], [text-anchor], [text-decoration],
    /// [textLength], [transform], x, y
    #[allow(clippy::too_many_arguments)]
    pub fn svg_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        font_family: Option<&str>,
        font_size: f64,
        font_weight: Option<&str>,
        font_style: Option<&str>,
        text_decoration: Option<&str>,
        text_length: f64,
        length_adjust: LengthAdjust,
        text_back_color: Option<&str>,
        orientation: i32,
        text_anchor: Option<&str>,
    ) {
        log::trace!(
            "svg_text: text={:?}, x={}, y={}, font_size={}, fill={}",
            text,
            x,
            y,
            font_size,
            self.fill
        );
        if self.hidden {
            self.ensure_visible(x, y);
            self.ensure_visible(x + text_length, y);
            return;
        }

        let mut text_content = text.to_string();

        // Handle monospace non-breaking spaces
        if let Some(family) = font_family {
            let lower = family.to_lowercase();
            if lower == "monospace" || lower == "courier" {
                text_content = text_content.replace(' ', "\u{00A0}");
            }
        }

        let mut elt = String::with_capacity(256);
        elt.push_str("<text");

        // Attributes in alphabetical order
        self.write_fill(&mut elt);

        if let Some(back_color) = text_back_color {
            let filter_id = self.get_filter_back_color(back_color);
            write!(elt, " filter=\"url(#{})\"", filter_id).unwrap();
        }

        let actual_family = font_family.map(|f| {
            if f.eq_ignore_ascii_case("monospaced") {
                "monospace"
            } else {
                f
            }
        });

        if let Some(family) = actual_family {
            write!(elt, " font-family=\"{}\"", family).unwrap();
        }

        write!(elt, " font-size=\"{}\"", self.f(font_size)).unwrap();

        if let Some(style) = font_style {
            write!(elt, " font-style=\"{}\"", style).unwrap();
        }

        if let Some(weight) = font_weight {
            // Stable PlantUML 1.2026.2 emits the keyword form, not numeric 700.
            write!(elt, " font-weight=\"{}\"", weight).unwrap();
        }

        match length_adjust {
            LengthAdjust::Spacing => {
                elt.push_str(" lengthAdjust=\"spacing\"");
            }
            LengthAdjust::SpacingAndGlyphs => {
                elt.push_str(" lengthAdjust=\"spacingAndGlyphs\"");
            }
            LengthAdjust::None => {}
        }

        if let Some(anchor) = text_anchor {
            write!(elt, " text-anchor=\"{}\"", anchor).unwrap();
        }

        if let Some(decoration) = text_decoration {
            write!(elt, " text-decoration=\"{}\"", decoration).unwrap();
        }

        match length_adjust {
            LengthAdjust::Spacing | LengthAdjust::SpacingAndGlyphs => {
                write!(elt, " textLength=\"{}\"", self.f(text_length)).unwrap();
            }
            LengthAdjust::None => {}
        }

        // Rotation transform
        if orientation == 90 {
            write!(
                elt,
                " transform=\"rotate(-90 {} {})\"",
                self.f(x),
                self.f(y)
            )
            .unwrap();
        } else if orientation == 270 {
            write!(elt, " transform=\"rotate(90 {} {})\"", self.f(x), self.f(y)).unwrap();
        }

        write!(elt, " x=\"{}\"", self.f(x)).unwrap();
        write!(elt, " y=\"{}\"", self.f(y)).unwrap();

        elt.push('>');
        elt.push_str(&xml_escape(&text_content));
        elt.push_str("</text>");

        self.buf.push_str(&elt);

        // Track bounds: text-anchor affects which x range the text occupies.
        // Java handles this via translate-based centering, so ensureVisible
        // always sees the real left/right edges.
        match text_anchor {
            Some("middle") => {
                let half = text_length / 2.0;
                self.ensure_visible(x - half, y);
                self.ensure_visible(x + half, y);
            }
            Some("end") => {
                self.ensure_visible(x - text_length, y);
                self.ensure_visible(x, y);
            }
            _ => {
                self.ensure_visible(x, y);
                self.ensure_visible(x + text_length, y);
            }
        }
    }

    // ── Comment ─────────────────────────────────────────────────────

    pub fn add_comment(&mut self, comment: &str) {
        write!(self.buf, "<!--{}-->", svg_comment_escape(comment)).unwrap();
    }

    // ── Group management ────────────────────────────────────────────

    pub fn start_group(&mut self, attrs: &[(&str, &str)]) {
        let mut g = String::from("<g");
        for (key, value) in attrs {
            write!(g, " {}=\"{}\"", key, xml_escape(value)).unwrap();
        }
        g.push('>');
        self.buf.push_str(&g);
        self.group_stack.push(GroupEntry::Group(String::new()));
    }

    pub fn close_group(&mut self) {
        self.buf.push_str("</g>");
        self.group_stack.pop();
    }

    // ── Link management ─────────────────────────────────────────────

    pub fn open_link(&mut self, url: &str, tooltip: Option<&str>, target: &str) {
        let title = tooltip.unwrap_or(url);
        let mut a = String::from("<a");
        write!(a, " href=\"{}\"", xml_escape(url)).unwrap();
        write!(a, " target=\"{}\"", target).unwrap();
        write!(a, " title=\"{}\"", xml_escape(title)).unwrap();
        a.push_str(" xlink:actuate=\"onRequest\"");
        write!(a, " xlink:href=\"{}\"", xml_escape(url)).unwrap();
        a.push_str(" xlink:show=\"new\"");
        write!(a, " xlink:title=\"{}\"", xml_escape(title)).unwrap();
        a.push_str(" xlink:type=\"simple\"");
        a.push('>');
        self.buf.push_str(&a);
        self.group_stack.push(GroupEntry::Link(url.to_string()));
    }

    pub fn close_link(&mut self) {
        self.buf.push_str("</a>");
        self.group_stack.pop();
    }

    // ── Full SVG document assembly ──────────────────────────────────

    /// Assemble the complete SVG document as a string.
    pub fn to_svg(&self, bg_color: Option<&str>, diagram_type: &str) -> String {
        let max_x_scaled = (self.max_x as f64 * self.scale) as i32;
        let max_y_scaled = (self.max_y as f64 * self.scale) as i32;

        let mut svg = String::with_capacity(self.buf.len() + self.defs.len() + 512);

        // SVG root element with attributes in alphabetical order
        write!(svg, "<?plantuml {}?>", PLANTUML_VERSION).unwrap();
        svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\"");
        svg.push_str(" xmlns:xlink=\"http://www.w3.org/1999/xlink\"");
        svg.push_str(" contentStyleType=\"text/css\"");
        if !diagram_type.is_empty() {
            write!(svg, " data-diagram-type=\"{}\"", diagram_type).unwrap();
        }
        write!(svg, " height=\"{}px\"", max_y_scaled).unwrap();
        svg.push_str(" preserveAspectRatio=\"none\"");

        let mut style = format!("width:{}px;height:{}px;", max_x_scaled, max_y_scaled);
        if let Some(bg) = bg_color {
            if bg != "#00000000" {
                write!(style, "background:{};", bg).unwrap();
            }
        }
        write!(svg, " style=\"{}\"", style).unwrap();

        svg.push_str(" version=\"1.1\"");
        write!(svg, " viewBox=\"0 0 {} {}\"", max_x_scaled, max_y_scaled).unwrap();
        write!(svg, " width=\"{}px\"", max_x_scaled).unwrap();
        svg.push_str(" zoomAndPan=\"magnify\">");

        // Defs
        if !self.defs.is_empty() {
            svg.push_str("<defs>");
            svg.push_str(&self.defs);
            svg.push_str("</defs>");
        }

        // Body
        svg.push_str("<g>");
        svg.push_str(&self.buf);
        svg.push_str("</g>");

        svg.push_str("</svg>");
        svg
    }

    /// Get the raw body content (without SVG wrapper).
    pub fn body(&self) -> &str {
        &self.buf
    }

    /// Get the raw defs content.
    pub fn defs(&self) -> &str {
        &self.defs
    }

    /// Push raw SVG markup into the body buffer.
    pub fn push_raw(&mut self, raw: &str) {
        self.buf.push_str(raw);
    }

    /// Push raw SVG markup into the defs buffer.
    pub fn push_raw_defs(&mut self, raw: &str) {
        self.defs.push_str(raw);
    }

    /// Get max_x (for layout calculations).
    pub fn max_x(&self) -> i32 {
        self.max_x
    }

    /// Get max_y (for layout calculations).
    pub fn max_y(&self) -> i32 {
        self.max_y
    }

    /// Manually set max_x/max_y (e.g., after full layout).
    pub fn set_max(&mut self, x: i32, y: i32) {
        self.max_x = x;
        self.max_y = y;
    }

    /// Get the scale factor.
    pub fn scale(&self) -> f64 {
        self.scale
    }
}

// ── LengthAdjust enum ──────────────────────────────────────────────

/// Controls SVG text length adjustment mode.
/// Java: `style.LengthAdjust`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LengthAdjust {
    #[default]
    Spacing,
    SpacingAndGlyphs,
    None,
}

// ── UGraphic implementation ─────────────────────────────────────────

/// High-level drawing context wrapping SvgGraphic.
/// Implements the UGraphic trait, dispatching shape drawing to the
/// appropriate SvgGraphic methods.
///
/// Java: `UGraphicSvg` + Driver*Svg classes
pub struct UGraphicSvg {
    svg: SvgGraphic,
    param: UParam,
    translate: UTranslate,
    string_bounder: Box<dyn StringBounder>,
    length_adjust: LengthAdjust,
}

impl UGraphicSvg {
    pub fn new(
        svg: SvgGraphic,
        string_bounder: Box<dyn StringBounder>,
        length_adjust: LengthAdjust,
    ) -> Self {
        Self {
            svg,
            param: UParam::default(),
            translate: UTranslate::none(),
            string_bounder,
            length_adjust,
        }
    }

    /// Get a reference to the underlying SvgGraphic.
    pub fn svg(&self) -> &SvgGraphic {
        &self.svg
    }

    /// Get a mutable reference to the underlying SvgGraphic.
    pub fn svg_mut(&mut self) -> &mut SvgGraphic {
        &mut self.svg
    }

    /// Consume self and return the SvgGraphic.
    pub fn into_svg(self) -> SvgGraphic {
        self.svg
    }

    fn apply_fill_color(&mut self) {
        let bg = &self.param.backcolor;
        self.svg.set_fill_color(&bg.to_svg());
    }

    fn apply_stroke_color(&mut self) {
        let color = &self.param.color;
        self.svg.set_stroke_color(Some(&color.to_svg()));
    }

    fn apply_stroke_width(&mut self) {
        let thickness = self.param.stroke.thickness;
        let dasharray = self.param.stroke.dasharray_svg();
        self.svg.set_stroke_width(thickness, dasharray);
    }
}

impl super::UGraphic for UGraphicSvg {
    fn apply(&mut self, change: &dyn UChange) {
        use std::any::Any;
        let any = change as &dyn Any;

        if let Some(translate) = any.downcast_ref::<UTranslate>() {
            self.translate = self.translate.compose(*translate);
        } else if let Some(stroke) = any.downcast_ref::<UStroke>() {
            self.param.stroke = stroke.clone();
        } else if let Some(color) = any.downcast_ref::<HColor>() {
            self.param.color = color.clone();
        } else if let Some(bg) = any.downcast_ref::<super::UBackground>() {
            match bg {
                super::UBackground::Color(c) => self.param.backcolor = c.clone(),
                super::UBackground::None => self.param.backcolor = HColor::none(),
            }
        }
    }

    fn param(&self) -> &UParam {
        &self.param
    }

    fn string_bounder(&self) -> &dyn StringBounder {
        self.string_bounder.as_ref()
    }

    /// Draw a rectangle. Java: DriverRectangleSvg.draw()
    fn draw_rect(&mut self, width: f64, height: f64, rx: f64) {
        let x = self.translate.dx;
        let y = self.translate.dy;
        self.apply_fill_color();
        self.apply_stroke_color();
        self.apply_stroke_width();
        self.svg
            .svg_rectangle(x, y, width, height, rx / 2.0, rx / 2.0, 0.0);
    }

    /// Draw an ellipse. Java: DriverEllipseSvg.draw()
    fn draw_ellipse(&mut self, width: f64, height: f64) {
        let x = self.translate.dx;
        let y = self.translate.dy;
        self.apply_stroke_color();
        self.apply_fill_color();
        self.apply_stroke_width();
        let cx = x + width / 2.0;
        let cy = y + height / 2.0;
        self.svg.svg_ellipse(cx, cy, width / 2.0, height / 2.0, 0.0);
    }

    /// Draw a line. Java: DriverLineSvg.draw()
    fn draw_line(&mut self, dx: f64, dy: f64) {
        let x1 = self.translate.dx;
        let y1 = self.translate.dy;
        let x2 = x1 + dx;
        let y2 = y1 + dy;
        self.svg.set_stroke_color(Some(&self.param.color.to_svg()));
        self.apply_stroke_width();
        self.svg.svg_line(x1, y1, x2, y2, 0.0);
    }

    /// Draw text. Java: DriverTextSvg.draw()
    fn draw_text(
        &mut self,
        text: &str,
        font_family: &str,
        font_size: f64,
        bold: bool,
        italic: bool,
    ) {
        let x = self.translate.dx;
        let y = self.translate.dy;

        let text_color = &self.param.color;
        self.svg.set_fill_color(&text_color.to_svg());

        let dim =
            self.string_bounder
                .calculate_dimension(font_family, font_size, bold, italic, text);
        let text_length = dim.width;

        let font_weight = if bold { Some("bold") } else { Option::None };
        let font_style = if italic { Some("italic") } else { Option::None };

        self.svg.svg_text(
            text,
            x,
            y,
            Some(font_family),
            font_size,
            font_weight,
            font_style,
            Option::None,
            text_length,
            self.length_adjust,
            Option::None,
            0,
            None,
        );
    }

    /// Draw a UPath. Java: DriverPathSvg.draw()
    fn draw_path(&mut self, path: &UPath) {
        let x = self.translate.dx;
        let y = self.translate.dy;
        self.apply_fill_color();
        self.apply_stroke_color();
        self.apply_stroke_width();
        self.svg.svg_path(x, y, path, path.shadow);
    }

    /// Draw a polygon. Java: DriverPolygonSvg.draw()
    fn draw_polygon(&mut self, points: &[(f64, f64)]) {
        let x = self.translate.dx;
        let y = self.translate.dy;
        self.apply_fill_color();
        self.apply_stroke_color();
        self.apply_stroke_width();

        let mut flat_points: Vec<f64> = Vec::with_capacity(points.len() * 2);
        for &(px, py) in points {
            flat_points.push(px + x);
            flat_points.push(py + y);
        }
        self.svg.svg_polygon(0.0, &flat_points);
    }

    fn start_group(&mut self, group: &super::UGroup) {
        let attrs: Vec<(&str, &str)> = group
            .entries()
            .iter()
            .map(|(k, v)| (k.svg_key_attribute_name(), v.as_str()))
            .collect();
        self.svg.start_group(&attrs);
    }

    fn close_group(&mut self) {
        self.svg.close_group();
    }

    fn start_url(&mut self, url: &str, tooltip: &str) {
        self.svg.open_link(url, Some(tooltip), "_top");
    }

    fn close_url(&mut self) {
        self.svg.close_link();
    }
}

// ── Test helpers (public for integration tests) ─────────────────────

#[doc(hidden)]
pub mod test_helpers {
    /// Format a number at scale 1.0 (for cross-language verification tests)
    pub fn fmt_at_scale_1(value: f64) -> String {
        super::fmt(value, 1.0)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a SvgGraphic with scale 1.0 and seed 12345.
    fn make_svg() -> SvgGraphic {
        SvgGraphic::new(12345, 1.0)
    }

    // ── Number formatting tests ────────────────────────────────

    #[test]
    fn fmt_zero() {
        assert_eq!(fmt(0.0, 1.0), "0");
    }

    #[test]
    fn fmt_integer() {
        assert_eq!(fmt(42.0, 1.0), "42");
    }

    #[test]
    fn fmt_trailing_zeros_stripped() {
        assert_eq!(fmt(1.5, 1.0), "1.5");
        assert_eq!(fmt(1.50, 1.0), "1.5");
    }

    #[test]
    fn fmt_four_decimal_places() {
        assert_eq!(fmt(1.23456, 1.0), "1.2346"); // rounded
    }

    #[test]
    fn fmt_negative_zero_near() {
        assert_eq!(fmt(-0.00004, 1.0), "0");
    }

    #[test]
    fn fmt_scale_applied() {
        assert_eq!(fmt(10.0, 2.0), "20");
    }

    #[test]
    fn fmt_bool_values() {
        assert_eq!(fmt_bool(0.0), "0");
        assert_eq!(fmt_bool(1.0), "1");
        assert_eq!(fmt_bool(42.0), "1");
    }

    // ── Seed formatting ────────────────────────────────────────

    #[test]
    fn seed_format_base36() {
        assert_eq!(SvgGraphic::format_seed(0), "0");
        assert_eq!(SvgGraphic::format_seed(35), "z");
        assert_eq!(SvgGraphic::format_seed(36), "10");
    }

    // ── Rectangle ──────────────────────────────────────────────

    #[test]
    fn svg_rect_basic() {
        let mut svg = make_svg();
        svg.set_fill_color("#E2E2F0");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(0.5, None);
        svg.svg_rectangle(5.0, 5.0, 47.667, 30.2969, 2.5, 2.5, 0.0);
        let body = svg.body();
        assert!(body.contains("<rect"));
        assert!(body.contains("fill=\"#E2E2F0\""));
        assert!(body.contains("height=\"30.2969\""));
        assert!(body.contains("rx=\"2.5\""));
        assert!(body.contains("ry=\"2.5\""));
        assert!(body.contains("style=\"stroke:#181818;stroke-width:0.5;\""));
        assert!(body.contains("width=\"47.667\""));
        assert!(body.contains("x=\"5\""));
        assert!(body.contains("y=\"5\""));
        assert!(body.ends_with("/>"));
    }

    #[test]
    fn svg_rect_no_round_corners() {
        let mut svg = make_svg();
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(10.0, 20.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(!body.contains("rx="));
        assert!(!body.contains("ry="));
    }

    #[test]
    fn svg_rect_zero_height_skipped() {
        let mut svg = make_svg();
        svg.svg_rectangle(0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0);
        assert!(svg.body().is_empty());
    }

    #[test]
    fn svg_rect_with_dash() {
        let mut svg = make_svg();
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, Some((5.0, 5.0)));
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(body.contains("stroke-dasharray:5,5;"));
    }

    #[test]
    fn svg_rect_with_fill_opacity() {
        let mut svg = make_svg();
        svg.set_fill_color("#00000000");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(body.contains("fill=\"none\""));
    }

    #[test]
    fn svg_rect_with_alpha_fill() {
        let mut svg = make_svg();
        svg.set_fill_color_with_opacity("#000000CC");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(body.contains("fill=\"#000000\""));
        assert!(body.contains("fill-opacity=\"0.80000\""));
    }

    // ── Ellipse ────────────────────────────────────────────────

    #[test]
    fn svg_ellipse_basic() {
        let mut svg = make_svg();
        svg.set_fill_color("#FEFECE");
        svg.set_stroke_color(Some("#A80036"));
        svg.set_stroke_width(1.5, None);
        svg.svg_ellipse(50.0, 30.0, 11.0, 11.0, 0.0);
        let body = svg.body();
        assert!(body.contains("<ellipse"));
        assert!(body.contains("cx=\"50\""));
        assert!(body.contains("cy=\"30\""));
        assert!(body.contains("fill=\"#FEFECE\""));
        assert!(body.contains("rx=\"11\""));
        assert!(body.contains("ry=\"11\""));
        assert!(body.contains("style=\"stroke:#A80036;stroke-width:1.5;\""));
    }

    // ── Line ───────────────────────────────────────────────────

    #[test]
    fn svg_line_basic() {
        let mut svg = make_svg();
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        svg.svg_line(10.0, 20.0, 110.0, 20.0, 0.0);
        let body = svg.body();
        assert!(body.contains("<line"));
        assert!(body.contains("style=\"stroke:#181818;stroke-width:1;\""));
        assert!(body.contains("x1=\"10\""));
        assert!(body.contains("x2=\"110\""));
        assert!(body.contains("y1=\"20\""));
        assert!(body.contains("y2=\"20\""));
    }

    #[test]
    fn svg_line_with_dash() {
        let mut svg = make_svg();
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(0.5, Some((5.0, 5.0)));
        svg.svg_line(28.0, 36.2969, 28.0, 114.5625, 0.0);
        let body = svg.body();
        assert!(body.contains("stroke-dasharray:5,5;"));
    }

    // ── Polygon ────────────────────────────────────────────────

    #[test]
    fn svg_polygon_basic() {
        let mut svg = make_svg();
        svg.set_fill_color("#181818");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        let points = vec![
            167.6152, 63.4297, 177.6152, 67.4297, 167.6152, 71.4297, 171.6152, 67.4297,
        ];
        svg.svg_polygon(0.0, &points);
        let body = svg.body();
        assert!(body.contains("<polygon"));
        assert!(body.contains("fill=\"#181818\""));
        assert!(body.contains("points=\""));
        assert!(body.contains("167.6152,63.4297,177.6152,67.4297"));
    }

    // ── Path ───────────────────────────────────────────────────

    #[test]
    fn svg_path_basic() {
        let mut svg = make_svg();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);

        let mut path = UPath::new();
        path.move_to(10.0, 20.0);
        path.line_to(110.0, 20.0);
        path.line_to(110.0, 70.0);
        path.line_to(10.0, 70.0);
        path.close();

        svg.svg_path(0.0, 0.0, &path, 0.0);
        let body = svg.body();
        assert!(body.contains("<path"));
        assert!(body.contains("d=\"M10,20 L110,20 L110,70 L10,70\""));
        assert!(body.contains("fill=\"none\""));
    }

    #[test]
    fn svg_path_with_cubic() {
        let mut svg = make_svg();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);

        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(10.0, 0.0, 20.0, 10.0, 20.0, 20.0);

        svg.svg_path(5.0, 5.0, &path, 0.0);
        let body = svg.body();
        assert!(body.contains("M5,5"));
        assert!(body.contains("C15,5 25,15 25,25"));
    }

    #[test]
    fn svg_path_with_arc() {
        let mut svg = make_svg();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);

        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.arc_to(25.0, 25.0, 0.0, 0.0, 1.0, 50.0, 50.0);

        svg.svg_path(0.0, 0.0, &path, 0.0);
        let body = svg.body();
        assert!(body.contains("A25,25 0 0 1 50,50"));
    }

    // ── Text ───────────────────────────────────────────────────

    #[test]
    fn svg_text_basic() {
        let mut svg = make_svg();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "Alice",
            12.0,
            24.9951,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            33.667,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("<text"));
        assert!(body.contains("fill=\"#000000\""));
        assert!(body.contains("font-family=\"sans-serif\""));
        assert!(body.contains("font-size=\"14\""));
        assert!(body.contains("lengthAdjust=\"spacing\""));
        assert!(body.contains("textLength=\"33.667\""));
        assert!(body.contains("x=\"12\""));
        assert!(body.contains("y=\"24.9951\""));
        assert!(body.contains(">Alice</text>"));
    }

    #[test]
    fn svg_text_bold_italic() {
        let mut svg = make_svg();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "test",
            0.0,
            0.0,
            Some("sans-serif"),
            14.0,
            Some("bold"),
            Some("italic"),
            None,
            20.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("font-style=\"italic\""));
        assert!(body.contains("font-weight=\"bold\""));
    }

    #[test]
    fn svg_text_underline() {
        let mut svg = make_svg();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "underlined",
            0.0,
            0.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            Some("underline"),
            50.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("text-decoration=\"underline\""));
    }

    #[test]
    fn svg_text_monospaced_to_monospace() {
        let mut svg = make_svg();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "code",
            0.0,
            0.0,
            Some("monospaced"),
            13.0,
            None,
            None,
            None,
            30.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("font-family=\"monospace\""));
    }

    #[test]
    fn svg_text_with_back_color() {
        let mut svg = make_svg();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "highlighted",
            0.0,
            0.0,
            Some("sans-serif"),
            13.0,
            None,
            None,
            None,
            50.0,
            LengthAdjust::Spacing,
            Some("#FFFF00"),
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("filter=\"url(#"));
        let defs = svg.defs();
        assert!(defs.contains("flood-color=\"#FFFF00\""));
    }

    #[test]
    fn svg_text_rotation() {
        let mut svg = make_svg();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "rotated",
            100.0,
            50.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            40.0,
            LengthAdjust::None,
            None,
            90,
            None,
        );
        let body = svg.body();
        assert!(body.contains("transform=\"rotate(-90 100 50)\""));
    }

    // ── Shadow ─────────────────────────────────────────────────

    #[test]
    fn svg_rect_with_shadow() {
        let mut svg = make_svg();
        svg.set_fill_color("#FEFECE");
        svg.set_stroke_color(Some("#A80036"));
        svg.set_stroke_width(1.5, None);
        svg.svg_rectangle(10.0, 10.0, 100.0, 50.0, 5.0, 5.0, 3.0);
        let body = svg.body();
        let defs = svg.defs();
        assert!(defs.contains("<filter"));
        assert!(defs.contains("feGaussianBlur"));
        assert!(defs.contains("feColorMatrix"));
        assert!(defs.contains("feOffset"));
        assert!(defs.contains("feBlend"));
        assert!(body.contains("filter=\"url(#"));
    }

    // ── Gradient ───────────────────────────────────────────────

    #[test]
    fn gradient_creation() {
        let mut svg = make_svg();
        let id1 = svg.create_svg_gradient("#FF0000", "#0000FF", '/');
        let id2 = svg.create_svg_gradient("#FF0000", "#0000FF", '/');
        assert_eq!(id1, id2);
        let defs = svg.defs();
        assert!(defs.contains("<linearGradient"));
        assert!(defs.contains("stop-color=\"#FF0000\""));
        assert!(defs.contains("stop-color=\"#0000FF\""));
    }

    #[test]
    fn gradient_different_policies() {
        let mut svg = make_svg();
        let id1 = svg.create_svg_gradient("#FF0000", "#0000FF", '|');
        let id2 = svg.create_svg_gradient("#FF0000", "#0000FF", '-');
        assert_ne!(id1, id2);
    }

    // ── Group/Link ─────────────────────────────────────────────

    #[test]
    fn group_basic() {
        let mut svg = make_svg();
        svg.start_group(&[("id", "mygroup"), ("class", "cls")]);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        svg.close_group();
        let body = svg.body();
        assert!(body.contains("<g id=\"mygroup\" class=\"cls\">"));
        assert!(body.contains("</g>"));
    }

    #[test]
    fn link_basic() {
        let mut svg = make_svg();
        svg.open_link("http://example.com", Some("Example"), "_blank");
        svg.set_fill_color("#0000FF");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        svg.close_link();
        let body = svg.body();
        assert!(body.contains("<a"));
        assert!(body.contains("href=\"http://example.com\""));
        assert!(body.contains("target=\"_blank\""));
        assert!(body.contains("title=\"Example\""));
        assert!(body.contains("</a>"));
    }

    // ── Comment ────────────────────────────────────────────────

    #[test]
    fn comment_basic() {
        let mut svg = make_svg();
        svg.add_comment("test comment");
        assert!(svg.body().contains("<!--test comment-->"));
    }

    #[test]
    fn comment_non_ascii_becomes_question_mark() {
        let mut svg = make_svg();
        svg.add_comment("A<&>é");
        assert!(svg.body().contains("<!--A<&>?-->"));
    }

    // ── Full SVG document ──────────────────────────────────────

    #[test]
    fn to_svg_document() {
        let mut svg = make_svg();
        svg.set_fill_color("#E2E2F0");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(0.5, None);
        svg.svg_rectangle(5.0, 5.0, 47.667, 30.2969, 2.5, 2.5, 0.0);

        let doc = svg.to_svg(Some("#FFFFFF"), "SEQUENCE");
        assert!(doc.starts_with("<?plantuml "));
        assert!(doc.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(doc.contains("xmlns:xlink=\"http://www.w3.org/1999/xlink\""));
        assert!(doc.contains("contentStyleType=\"text/css\""));
        assert!(doc.contains("data-diagram-type=\"SEQUENCE\""));
        assert!(doc.contains("version=\"1.1\""));
        assert!(doc.contains("zoomAndPan=\"magnify\">"));
        assert!(doc.contains("<?plantuml"));
        assert!(doc.contains("<g>"));
        assert!(doc.contains("</g>"));
        assert!(doc.ends_with("</svg>"));
    }

    // ── Hidden state ───────────────────────────────────────────

    #[test]
    fn hidden_rect_not_emitted() {
        let mut svg = make_svg();
        svg.set_hidden(true);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        assert!(svg.body().is_empty());
        assert!(svg.max_x() >= 100);
    }

    #[test]
    fn hidden_ellipse_not_emitted() {
        let mut svg = make_svg();
        svg.set_hidden(true);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_ellipse(50.0, 50.0, 25.0, 25.0, 0.0);
        assert!(svg.body().is_empty());
    }

    #[test]
    fn hidden_line_not_emitted() {
        let mut svg = make_svg();
        svg.set_hidden(true);
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_line(0.0, 0.0, 100.0, 100.0, 0.0);
        assert!(svg.body().is_empty());
    }

    // ── XML escaping ───────────────────────────────────────────

    #[test]
    fn xml_escape_basic() {
        assert_eq!(xml_escape("a<b>c&d\"e"), "a&lt;b&gt;c&amp;d\"e");
    }

    #[test]
    fn xml_escape_non_ascii() {
        assert_eq!(xml_escape("\u{00E9}"), "&#233;");
    }

    #[test]
    fn svg_comment_escape_non_ascii() {
        assert_eq!(svg_comment_escape("A<&>é"), "A<&>?");
    }

    // ── Attribute order verification ───────────────────────────

    #[test]
    fn rect_attribute_order() {
        let mut svg = make_svg();
        svg.set_fill_color("#E2E2F0");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(0.5, None);
        svg.svg_rectangle(5.0, 5.0, 47.667, 30.2969, 2.5, 2.5, 0.0);
        let body = svg.body();
        let fill_pos = body.find("fill=").unwrap();
        let height_pos = body.find("height=").unwrap();
        let rx_pos = body.find("rx=").unwrap();
        let ry_pos = body.find("ry=").unwrap();
        let style_pos = body.find("style=").unwrap();
        let width_pos = body.find("width=").unwrap();
        let x_pos = body.find(" x=").unwrap();
        let y_pos = body.find(" y=").unwrap();
        assert!(fill_pos < height_pos);
        assert!(height_pos < rx_pos);
        assert!(rx_pos < ry_pos);
        assert!(ry_pos < style_pos);
        assert!(style_pos < width_pos);
        assert!(width_pos < x_pos);
        assert!(x_pos < y_pos);
    }

    #[test]
    fn ellipse_attribute_order() {
        let mut svg = make_svg();
        svg.set_fill_color("#FEFECE");
        svg.set_stroke_color(Some("#A80036"));
        svg.set_stroke_width(1.0, None);
        svg.svg_ellipse(50.0, 30.0, 11.0, 11.0, 0.0);
        let body = svg.body();
        let cx_pos = body.find("cx=").unwrap();
        let cy_pos = body.find("cy=").unwrap();
        let fill_pos = body.find("fill=").unwrap();
        let rx_pos = body.find("rx=").unwrap();
        let ry_pos = body.find("ry=").unwrap();
        let style_pos = body.find("style=").unwrap();
        assert!(cx_pos < cy_pos);
        assert!(cy_pos < fill_pos);
        assert!(fill_pos < rx_pos);
        assert!(rx_pos < ry_pos);
        assert!(ry_pos < style_pos);
    }

    #[test]
    fn line_attribute_order() {
        let mut svg = make_svg();
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        svg.svg_line(10.0, 20.0, 110.0, 20.0, 0.0);
        let body = svg.body();
        let style_pos = body.find("style=").unwrap();
        let x1_pos = body.find("x1=").unwrap();
        let x2_pos = body.find("x2=").unwrap();
        let y1_pos = body.find("y1=").unwrap();
        let y2_pos = body.find("y2=").unwrap();
        assert!(style_pos < x1_pos);
        assert!(x1_pos < x2_pos);
        assert!(x2_pos < y1_pos);
        assert!(y1_pos < y2_pos);
    }

    #[test]
    fn text_attribute_order() {
        let mut svg = make_svg();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "Hello",
            12.0,
            25.0,
            Some("sans-serif"),
            14.0,
            Some("bold"),
            Some("italic"),
            Some("underline"),
            40.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        let fill_pos = body.find("fill=").unwrap();
        let ff_pos = body.find("font-family=").unwrap();
        let fs_pos = body.find("font-size=").unwrap();
        let fst_pos = body.find("font-style=").unwrap();
        let fw_pos = body.find("font-weight=").unwrap();
        let la_pos = body.find("lengthAdjust=").unwrap();
        let td_pos = body.find("text-decoration=").unwrap();
        let tl_pos = body.find("textLength=").unwrap();
        let x_pos = body.find(" x=").unwrap();
        let y_pos = body.find(" y=").unwrap();
        assert!(fill_pos < ff_pos);
        assert!(ff_pos < fs_pos);
        assert!(fs_pos < fst_pos);
        assert!(fst_pos < fw_pos);
        assert!(fw_pos < la_pos);
        assert!(la_pos < td_pos);
        assert!(td_pos < tl_pos);
        assert!(tl_pos < x_pos);
        assert!(x_pos < y_pos);
    }

    // ── Stroke width zero suppresses style ─────────────────────

    #[test]
    fn stroke_width_zero_no_style() {
        let mut svg = make_svg();
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(0.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(!body.contains("style="));
    }

    // ── Ensure visible tracking ────────────────────────────────

    #[test]
    fn ensure_visible_tracks_max() {
        let mut svg = make_svg();
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(100.0, 200.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        assert!(svg.max_x() >= 150);
        assert!(svg.max_y() >= 230);
    }

    // ── Pixel (simulated via tiny rect) ────────────────────────

    #[test]
    fn svg_pixel() {
        let mut svg = make_svg();
        svg.set_stroke_color(Some("#FF0000"));
        svg.set_stroke_width(0.5, None);
        svg.svg_rectangle(10.0, 20.0, 0.5, 0.5, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(body.contains("width=\"0.5\""));
        assert!(body.contains("height=\"0.5\""));
    }
}
