use std::collections::HashMap;
use std::fmt::Write;
use std::io::Write as IoWrite;

use flate2::write::DeflateEncoder;
use flate2::Compression;

use crate::layout::graphviz::{ClassNoteLayout, EdgeLayout, GraphLayout, NodeLayout};
use crate::layout::split_member_lines;
use crate::layout::DiagramLayout;
use crate::model::{
    ArrowHead, ClassDiagram, ClassHideShowRule, ClassPortion, ClassRuleTarget, Diagram,
    DiagramMeta, Entity, EntityKind, LineStyle, Link, Member, Visibility,
};
use crate::style::SkinParams;
use crate::Result;

use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};

use super::svg_richtext::{
    count_creole_lines, creole_plain_text, get_default_font_family_pub,
    max_creole_plain_line_len, render_creole_text, set_default_font_family,
};
use super::svg_sequence;

// ── Style constants ──────────────────────────────────────────────────

// ── Class diagram constants — all sourced from Java PlantUML code ────
//
// FontParam.java: CLASS = 14pt (name), CLASS_ATTRIBUTE = 10pt,
//   CLASS_STEREOTYPE = 12pt italic, CIRCLED_CHARACTER = 17pt Monospaced Bold.
// SkinParam.java:526: circledCharacterRadius = fontSize/3 + 6 = 17/3+6 = 11 (int).
// EntityImageClassHeader.java:150: withMargin(circledChar, 4, 0, 5, 5)
//   → block width = diameter(22) + marginLeft(4) + marginRight(0) = 26
//   → block height = diameter(22) + marginTop(5) + marginBottom(5) = 32
// EntityImageClassHeader.java:105: withMargin(name, 3, 3, 0, 0)
//   → name margin left=3, right=3
// HeaderLayout.java:74-77: width = circleDim.w + max(stereoDim.w, nameDim.w) + genericDim.w
//   height = max(circleDim.h, stereoDim.h + nameDim.h + 10, genericDim.h)

/// FontParam.CLASS = 12, but class name renders at 14 in SVG (EntityImageClassHeader uses 14pt).
const FONT_SIZE: f64 = 14.0;
/// MethodsOrFieldsArea: empty compartment margin_top(4) + margin_bottom(4) = 8.
const LINE_HEIGHT: f64 = 8.0;
/// EntityImageClassHeader name margin: withMargin(name, 3, 3, 0, 0) → right padding = 3.
const PADDING: f64 = 3.0;
/// HeaderLayout height when no stereotype: max(circleDim.h(32), nameDim.h(16.3)+10=26.3) = 32.
const HEADER_HEIGHT: f64 = 32.0;
/// SvekResult.java:133 — moveDelta(6 - minMax.getMinX(), 6 - minMax.getMinY()).
const MARGIN: f64 = 6.0;
/// SvekResult.java:135 — minMax.getDimension().delta(15, 15).
pub(crate) const CANVAS_DELTA: f64 = 15.0;
/// TextBlockExporter12026.java:196 — margin from plantuml.skin root.document style: right=5.
pub(crate) const DOC_MARGIN_RIGHT: f64 = 5.0;
/// TextBlockExporter12026.java:197 — margin from plantuml.skin root.document style: bottom=5.
pub(crate) const DOC_MARGIN_BOTTOM: f64 = 5.0;
/// EntityImageClassHeader.java:150 — withMargin(circledChar, left=4, right=0, top=5, bottom=5).
const CIRCLE_LEFT_PAD: f64 = 4.0;
/// SkinParam.circledCharacterRadius = 17/3+6 = 11. Diameter = 22.
const CIRCLE_DIAMETER: f64 = 22.0;
/// MethodsOrFieldsArea: empty compartment = margin_top(4) + margin_bottom(4).
const EMPTY_COMPARTMENT: f64 = 8.0;
/// Circled character block: diameter(22) + marginLeft(4) + marginRight(0) = 26.
const HEADER_CIRCLE_BLOCK_WIDTH: f64 = 26.0;
/// Circled character block: diameter(22) + marginTop(5) + marginBottom(5) = 32.
const HEADER_CIRCLE_BLOCK_HEIGHT: f64 = 32.0;
/// SansSerif 14pt plain: ascent(12.995117) + descent(3.301758) from Java AWT FontMetrics.
const HEADER_NAME_BLOCK_HEIGHT: f64 = 16.296875;
/// SansSerif 14pt plain ascent from Java AWT FontMetrics.
const HEADER_NAME_BASELINE: f64 = 12.995117;
/// EntityImageClassHeader.java:105 — withMargin(name, 3, 3, 0, 0): left(3) + right(3) = 6.
const HEADER_NAME_BLOCK_MARGIN_X: f64 = 6.0;
/// FontParam.CLASS_STEREOTYPE = 12pt.
const HEADER_STEREO_FONT_SIZE: f64 = 12.0;
/// SansSerif 12pt italic: ascent(11.138672) + descent(2.830078) from Java AWT FontMetrics.
const HEADER_STEREO_LINE_HEIGHT: f64 = 13.96875;
/// SansSerif 12pt italic ascent from Java AWT FontMetrics.
const HEADER_STEREO_BASELINE: f64 = 11.138672;
/// HeaderLayout.java:77 — max(..., stereoDim.h + nameDim.h + 10, ...) → gap = 10.
const HEADER_STEREO_NAME_GAP: f64 = 10.0;

// ── Member area (fields/methods) constants ──────────────────────────
//
// MethodsOrFieldsArea.java:85 — asBlockMemberImpl: TextBlockUtils.withMargin(this, 6, 4)
//   → margin left=6, right=6, top=4, bottom=4.
// PlacementStrategyVisibility.java:67 — icon y: 2 + y + (maxHeight - iconHeight) / 2.
// VisibilityModifier.java:101 — getUBlock: dimension = (size+1, size+1) = (11, 11).
// VisibilityModifier.java:182 — drawCircle: UTranslate(x+2, y+2), UEllipse(size-4, size-4) = (6, 6).
// DriverEllipseSvg: cx = x + width/2, cy = y + height/2.

/// SansSerif 14pt height from Java AWT FontMetrics. Used as row height in member area.
const MEMBER_ROW_HEIGHT: f64 = 16.296875;
/// margin_top(4) + MEMBER_ROW_HEIGHT + margin_bottom(4).
const MEMBER_BLOCK_HEIGHT_ONE_ROW: f64 = 24.296875;
/// Icon y from section separator: margin_top(4) + nudge(2) + (16.296875 - 11) / 2 = 8.6484375.
const MEMBER_ICON_Y_FROM_SEP: f64 = 8.6484375;
/// VisibilityModifier.drawCircle: UTranslate offset (+2, +2).
const MEMBER_ICON_DRAW_OFFSET: f64 = 2.0;
/// UEllipse(6, 6): rx = ry = 3.
const MEMBER_ICON_RADIUS: f64 = 3.0;
/// MethodsOrFieldsArea margin left = 6.
const MEMBER_ICON_X_OFFSET: f64 = 6.0;
/// margin_left(6) + col2(circledCharRadius(11) + 3) = 20.
const MEMBER_TEXT_X_WITH_ICON: f64 = 20.0;
/// margin_left(6) when no visibility icon column.
const MEMBER_TEXT_X_NO_ICON: f64 = 6.0;
/// margin_top(4) + SansSerif 14pt ascent(12.995117) = 16.995117.
const MEMBER_TEXT_Y_OFFSET: f64 = 16.995117;

/// Entity-level visibility icon block size (SkinParam.circledCharacterRadius = 11).
const ENTITY_VIS_ICON_BLOCK_SIZE: f64 = 11.0;

// -- Generic type box rendering constants --
/// Generic text font size (FontParam.CLASS_STEREOTYPE = 12pt italic).
const GENERIC_FONT_SIZE: f64 = 12.0;
/// SansSerif 12pt italic ascent from Java AWT FontMetrics.
const GENERIC_BASELINE: f64 = 11.138672;
/// SansSerif 12pt italic: ascent + descent = 13.96875.
const GENERIC_TEXT_HEIGHT: f64 = 13.96875;
/// Inner margin around generic text (withMargin(genericBlock, 1, 1)).
const GENERIC_INNER_MARGIN: f64 = 1.0;
/// Outer margin around TextBlockGeneric (withMargin(genericBlock, 1, 1)).
const GENERIC_OUTER_MARGIN: f64 = 1.0;
/// HeaderLayout.java:112 -- delta = 4 for positioning.
const GENERIC_DELTA: f64 = 4.0;
/// Protrusion above entity rect = delta - outer_margin = 3.
const GENERIC_PROTRUSION: f64 = GENERIC_DELTA - GENERIC_OUTER_MARGIN;

use crate::skin::rose::{
    BORDER_COLOR, DIVIDER_COLOR, ENTITY_BG, LEGEND_BG, LEGEND_BORDER,
    NOTE_BG, NOTE_BORDER, NOTE_FOLD, NOTE_PADDING as NOTE_TEXT_PADDING, TEXT_COLOR,
};
const LINK_COLOR: &str = BORDER_COLOR;
/// Java PlantUML renders link labels at font-size 13 (not 14).
const LINK_LABEL_FONT_SIZE: f64 = 13.0;
const PLANTUML_VERSION: &str = "1.2026.3beta5";

// ── Meta rendering constants ────────────────────────────────────────

const META_TITLE_FONT_SIZE: f64 = 14.0;
const META_HF_FONT_SIZE: f64 = 10.0;
const META_CAPTION_FONT_SIZE: f64 = 14.0;
const META_LEGEND_FONT_SIZE: f64 = 14.0;
/// Java TextBlockBordered.calculateDimension() returns (width+1, height+1).
/// See TextBlockBordered.java:98.
const BORDERED_EXTRA: f64 = 1.0;
const TITLE_PADDING: f64 = 5.0;
const TITLE_MARGIN: f64 = 5.0;
const CAPTION_PADDING: f64 = 0.0;
const CAPTION_MARGIN: f64 = 1.0;
const LEGEND_PADDING: f64 = 5.0;
const LEGEND_MARGIN: f64 = 12.0;
const LEGEND_ROUND_CORNER: f64 = 15.0;

// ── Helpers ─────────────────────────────────────────────────────────

pub(crate) use crate::klimt::svg::fmt_coord;

/// Write a Java PlantUML-compatible SVG root element and open a `<g>` wrapper.
pub(crate) fn write_svg_root(buf: &mut String, w: f64, h: f64, diagram_type: &str) {
    write_svg_root_bg(buf, w, h, diagram_type, "#FFFFFF");
}

pub(crate) fn write_svg_root_bg(buf: &mut String, w: f64, h: f64, diagram_type: &str, bg: &str) {
    write_svg_root_bg_opt(buf, w, h, Some(diagram_type), bg);
}

/// Java `SvgGraphics.ensureVisible` truncation: `maxX = (int)(x + 1)`.
/// Converts a floating-point dimension to the integer viewport value used by
/// Java PlantUML.  Callers must pass the RAW dimension BEFORE the +1 truncation.
/// The minimum is 10, matching Java's `SvgGraphics.maxX/maxY` initial value.
pub(crate) fn ensure_visible_int(x: f64) -> i32 {
    if x.is_finite() && x > 0.0 {
        ((x + 1.0) as i32).max(10)
    } else {
        10 // Java default
    }
}

/// Write SVG root element. `diagram_type` is optional — Java's PSystemSalt and
/// PSystemDot don't go through TitledDiagram, so they omit `data-diagram-type`.
///
/// `w` and `h` should already be integer-valued viewport dimensions (having
/// gone through `ensure_visible_int` or equivalent truncation).
/// The function rounds via `as i32` for safety but does NOT add +1.
pub(crate) fn write_svg_root_bg_opt(
    buf: &mut String,
    w: f64,
    h: f64,
    diagram_type: Option<&str>,
    bg: &str,
) {
    let wi = if w.is_finite() && w > 0.0 { w as i32 } else { 100 };
    let hi = if h.is_finite() && h > 0.0 { h as i32 } else { 100 };
    buf.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg""#);
    buf.push_str(r#" xmlns:xlink="http://www.w3.org/1999/xlink""#);
    buf.push_str(r#" contentStyleType="text/css""#);
    if let Some(dtype) = diagram_type {
        write!(buf, r#" data-diagram-type="{dtype}""#).unwrap();
    }
    write!(
        buf,
        concat!(
            r#" height="{hi}px""#,
            r#" preserveAspectRatio="none""#,
            r#" style="width:{wi}px;height:{hi}px;background:{bg};""#,
            r#" version="1.1""#,
            r#" viewBox="0 0 {wi} {hi}""#,
            r#" width="{wi}px""#,
            r#" zoomAndPan="magnify">"#,
        ),
        hi = hi,
        wi = wi,
        bg = bg,
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

pub(crate) use crate::klimt::svg::xml_escape;

/// Write a background `<rect>` covering the entire canvas when the background
/// color differs from the default #FFFFFF. Java PlantUML emits this rect as the
/// first child of `<g>` when `skinparam backgroundColor` is set.
pub(crate) fn write_bg_rect(buf: &mut String, w: f64, h: f64, bg: &str) {
    if !bg.eq_ignore_ascii_case("#FFFFFF") {
        let wi = if w.is_finite() && w > 0.0 { w as i32 } else { 100 };
        let hi = if h.is_finite() && h > 0.0 { h as i32 } else { 100 };
        write!(
            buf,
            r#"<rect fill="{bg}" height="{hi}" style="stroke:none;stroke-width:1;" width="{wi}" x="0" y="0"/>"#,
        )
        .unwrap();
    }
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
    let body_result = render_body(diagram, layout, skin)?;
    set_default_font_family(None);

    // Extract diagram type from body SVG
    let dtype = body_result.svg
        .find("data-diagram-type=\"")
        .and_then(|pos| {
            let start = pos + 19;
            body_result.svg[start..]
                .find('"')
                .map(|end| body_result.svg[start..start + end].to_string())
        })
        .unwrap_or_else(|| "CLASS".to_string());

    let mut svg = if meta.is_empty() && !meta.pragmas.contains_key("svginteractive") {
        body_result.svg
    } else {
        // Document-level BackGroundColor from <style> is stored as "document.backgroundcolor";
        // skinparam BackGroundColor is stored as "backgroundcolor". Try both.
        let bg = skin.get("document.backgroundcolor")
            .or_else(|| skin.get("backgroundcolor"))
            .unwrap_or("#FFFFFF");
        wrap_with_meta(&body_result.svg, meta, &dtype, bg, body_result.raw_body_dim, skin)?
    };

    // Inject svginteractive CSS/JS if pragma is set
    if meta.pragmas.get("svginteractive").map_or(false, |v| v == "true") {
        svg = inject_svginteractive(svg, &dtype);
    }

    if let Some(source) = source {
        svg = inject_plantuml_source(svg, source)?;
    }

    Ok(svg)
}

/// Body rendering result: (svg_string, raw_body_content_dimensions).
/// The raw dimensions are the precise body content size (Java SvekResult.calculateDimension)
/// before DOC_MARGIN and ensureVisible integer truncation. When present, wrap_with_meta
/// uses these instead of extracting lossy integer dimensions from the SVG header.
struct BodyResult {
    svg: String,
    raw_body_dim: Option<(f64, f64)>,
}

fn render_body(diagram: &Diagram, layout: &DiagramLayout, skin: &SkinParams) -> Result<BodyResult> {
    match (diagram, layout) {
        (Diagram::Class(cd), DiagramLayout::Class(gl)) => render_class(cd, gl, skin),
        (Diagram::Sequence(sd), DiagramLayout::Sequence(sl)) => {
            svg_sequence::render_sequence(sd, sl, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Activity(ad), DiagramLayout::Activity(al)) => {
            super::svg_activity::render_activity(ad, al, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::State(sd), DiagramLayout::State(sl)) => {
            super::svg_state::render_state(sd, sl, skin).map(|(svg, raw_body_dim)| BodyResult { svg, raw_body_dim })
        }
        (Diagram::Component(cd), DiagramLayout::Component(cl)) => {
            super::svg_component::render_component(cd, cl, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Ditaa(dd), DiagramLayout::Ditaa(dl)) => {
            super::svg_ditaa::render_ditaa(dd, dl, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Erd(ed), DiagramLayout::Erd(el)) => super::svg_erd::render_erd(ed, el, skin).map(|svg| BodyResult { svg, raw_body_dim: None }),
        (Diagram::Gantt(gd), DiagramLayout::Gantt(gl)) => {
            super::svg_gantt::render_gantt(gd, gl, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Json(jd), DiagramLayout::Json(jl)) => super::svg_json::render_json(jd, jl, skin).map(|svg| BodyResult { svg, raw_body_dim: None }),
        (Diagram::Mindmap(md), DiagramLayout::Mindmap(ml)) => {
            super::svg_mindmap::render_mindmap(md, ml, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Nwdiag(nd), DiagramLayout::Nwdiag(nl)) => {
            super::svg_nwdiag::render_nwdiag(nd, nl, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Salt(sd), DiagramLayout::Salt(sl)) => super::svg_salt::render_salt(sd, sl, skin).map(|svg| BodyResult { svg, raw_body_dim: None }),
        (Diagram::Timing(td), DiagramLayout::Timing(tl)) => {
            super::svg_timing::render_timing(td, tl, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Wbs(wd), DiagramLayout::Wbs(wl)) => super::svg_wbs::render_wbs(wd, wl, skin).map(|svg| BodyResult { svg, raw_body_dim: None }),
        (Diagram::Yaml(yd), DiagramLayout::Yaml(yl)) => super::svg_json::render_yaml(yd, yl, skin).map(|svg| BodyResult { svg, raw_body_dim: None }),
        (Diagram::UseCase(ud), DiagramLayout::UseCase(ul)) => {
            super::svg_usecase::render_usecase(ud, ul, skin).map(|svg| BodyResult { svg, raw_body_dim: None })
        }
        (Diagram::Dot(_dd), DiagramLayout::Dot(_gl)) => {
            // Java PlantUML suppresses DOT rendering
            Ok(BodyResult { svg: render_dot_suppressed(), raw_body_dim: None })
        }
        _ => Err(crate::Error::Render("diagram/layout type mismatch".into())),
    }
}

/// Render a suppressed-feature notice for DOT diagrams, matching Java PlantUML.
fn render_dot_suppressed() -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\">\n");
    s.push_str("<a xlink:href=\"https://github.com/plantuml/plantuml/issues/2495\">\n");
    s.push_str("<text x=\"10\" y=\"30\" font-family=\"sans-serif\" font-size=\"14\" fill=\"blue\" text-decoration=\"underline\">This feature has been suppressed</text>\n");
    s.push_str("</a>\n");
    s.push_str("</svg>");
    s
}

// ── Meta wrapping ───────────────────────────────────────────────────

fn creole_text_w(text: &str, font_size: f64, bold: bool) -> f64 {
    let plain = creole_plain_text(text);
    font_metrics::text_width(&plain, "SansSerif", font_size, bold, false)
}
fn text_block_h(font_size: f64, bold: bool) -> f64 {
    font_metrics::ascent("SansSerif", font_size, bold, false)
        + font_metrics::descent("SansSerif", font_size, bold, false)
}
fn bordered_dim(text_w: f64, text_h: f64, padding: f64) -> (f64, f64) {
    (text_w + 2.0 * padding + BORDERED_EXTRA, text_h + 2.0 * padding + BORDERED_EXTRA)
}
fn block_dim(text_w: f64, text_h: f64, padding: f64, margin: f64) -> (f64, f64) {
    let (bw, bh) = bordered_dim(text_w, text_h, padding);
    (bw + 2.0 * margin, bh + 2.0 * margin)
}
fn merge_tb(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0.max(b.0), a.1 + b.1)
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

/// Inline bounding-box tracker mirroring Java's LimitFinder.
/// Intercepts every draw call during rendering to compute the exact canvas size.
/// Tracks drawing bounds, mirroring Java `LimitFinder`.
///
/// Java uses a two-pass model:
///   Pass 1: LimitFinder tracks min/max of all draw operations
///   Pass 2: SvgGraphics ensureVisible uses (int)(x+1)
///
/// We use LimitFinder semantics (min/max tracking). The final SVG dimensions
/// are computed as: `(int)(span + CANVAS_DELTA + DOC_MARGIN + 1)` which is
/// equivalent to Java's `ensureVisible(span + delta + margin)`.
pub(crate) struct BoundsTracker {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl BoundsTracker {
    pub fn new() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    fn add_point(&mut self, x: f64, y: f64) {
        log::trace!("BoundsTracker.addPoint({:.4}, {:.4})", x, y);
        if x < self.min_x { self.min_x = x; }
        if y < self.min_y { self.min_y = y; }
        if x > self.max_x { self.max_x = x; }
        if y > self.max_y { self.max_y = y; }
    }

    /// Java LimitFinder.drawRectangle: (x-1, y-1) to (x+w-1+shadow*2, y+h-1+shadow*2)
    pub fn track_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.track_rect_shadow(x, y, w, h, 0.0);
    }

    /// Java LimitFinder.drawRectangle with delta shadow
    pub fn track_rect_shadow(&mut self, x: f64, y: f64, w: f64, h: f64, shadow: f64) {
        log::trace!("BoundsTracker.drawRect x={:.2} y={:.2} w={:.2} h={:.2} shadow={:.2}", x, y, w, h, shadow);
        self.add_point(x - 1.0, y - 1.0);
        self.add_point(x + w - 1.0 + shadow * 2.0, y + h - 1.0 + shadow * 2.0);
    }

    /// Java LimitFinder.drawEmpty: (x, y) to (x+w, y+h) — NO -1 adjustment
    pub fn track_empty(&mut self, x: f64, y: f64, w: f64, h: f64) {
        log::trace!("BoundsTracker.drawEmpty x={:.2} y={:.2} w={:.2} h={:.2}", x, y, w, h);
        self.add_point(x, y);
        self.add_point(x + w, y + h);
    }

    /// Java LimitFinder.drawEllipse: (x, y) to (x+w-1+shadow*2, y+h-1+shadow*2)
    /// where x,y is top-left of bounding box
    pub fn track_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64) {
        self.track_ellipse_shadow(cx, cy, rx, ry, 0.0);
    }

    /// Java LimitFinder.drawEllipse with delta shadow
    pub fn track_ellipse_shadow(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, shadow: f64) {
        // Java draws UEllipse at translate position (x, y) with width=2*rx, height=2*ry
        // LimitFinder.drawEllipse(x, y, ellipse): addPoint(x, y), addPoint(x+w-1+s*2, y+h-1+s*2)
        let x = cx - rx;
        let y = cy - ry;
        let w = 2.0 * rx;
        let h = 2.0 * ry;
        log::trace!("BoundsTracker.drawEllipse x={:.2} y={:.2} w={:.2} h={:.2} shadow={:.2}", x, y, w, h, shadow);
        self.add_point(x, y);
        self.add_point(x + w - 1.0 + shadow * 2.0, y + h - 1.0 + shadow * 2.0);
    }

    /// Java LimitFinder.drawUPolygon: HACK_X_FOR_POLYGON = 10
    pub fn track_polygon(&mut self, points: &[(f64, f64)]) {
        if points.is_empty() {
            return;
        }
        let min_x = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let max_x = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let min_y = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let max_y = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        log::trace!("BoundsTracker.drawPolygon minX={:.2} maxX={:.2} minY={:.2} maxY={:.2}", min_x, max_x, min_y, max_y);
        self.add_point(min_x - 10.0, min_y);
        self.add_point(max_x + 10.0, max_y);
    }

    /// Java LimitFinder.drawULine
    pub fn track_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        log::trace!("BoundsTracker.drawLine ({:.2},{:.2})-({:.2},{:.2})", x1, y1, x2, y2);
        self.add_point(x1, y1);
        self.add_point(x2, y2);
    }

    /// Java LimitFinder.drawDotPath — path bounding box
    pub fn track_path_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        log::trace!("BoundsTracker.drawDotPath min=({:.2},{:.2}) max=({:.2},{:.2})", min_x, min_y, max_x, max_y);
        self.add_point(min_x, min_y);
        self.add_point(max_x, max_y);
    }

    /// Java LimitFinder.drawText:
    ///   y_adj = y - h + 1.5
    ///   addPoint(x, y_adj), addPoint(x, y_adj+h), addPoint(x+w, y_adj), addPoint(x+w, y_adj+h)
    ///   i.e. (x, y-h+1.5) to (x+w, y+1.5)
    pub fn track_text(&mut self, x: f64, y: f64, text_width: f64, text_height: f64) {
        let y_adj = y - text_height + 1.5;
        log::trace!("BoundsTracker.drawText x={:.4} y={:.4} w={:.4} h={:.4} y_adj={:.4}", x, y, text_width, text_height, y_adj);
        self.add_point(x, y_adj);
        self.add_point(x, y_adj + text_height);
        self.add_point(x + text_width, y_adj);
        self.add_point(x + text_width, y_adj + text_height);
    }

    /// Span: max - min in each dimension. Used with CANVAS_DELTA + DOC_MARGIN
    /// to compute final SVG dimensions matching Java's ensureVisible.
    pub fn span(&self) -> (f64, f64) {
        if self.max_x.is_finite() && self.min_x.is_finite() {
            log::trace!("BoundsTracker.span: min=({:.4},{:.4}) max=({:.4},{:.4}) span=({:.4},{:.4})",
                self.min_x, self.min_y, self.max_x, self.max_y,
                self.max_x - self.min_x, self.max_y - self.min_y);
            (self.max_x - self.min_x, self.max_y - self.min_y)
        } else {
            (0.0, 0.0)
        }
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

// ── SVG interactive CSS/JS resources ─────────────────────────────────

/// CSS for sequence diagrams when `!pragma svginteractive true`
const SEQUENCE_INTERACTIVE_CSS: &str = include_str!("interactive/sequencediagram.css");
/// JS for sequence diagrams when `!pragma svginteractive true`
const SEQUENCE_INTERACTIVE_JS: &str = include_str!("interactive/sequencediagram.js");
/// CSS for non-sequence diagrams when `!pragma svginteractive true`
const DEFAULT_INTERACTIVE_CSS: &str = include_str!("interactive/default.css");
/// JS for non-sequence diagrams when `!pragma svginteractive true`
const DEFAULT_INTERACTIVE_JS: &str = include_str!("interactive/default.js");

/// Ensure text ends with a newline (matches Java FileUtils.readText behavior).
fn ensure_trailing_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{}\n", s)
    }
}

/// XML-escape text for embedding in SVG `<script>` elements.
fn xml_escape_js(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / 10);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Inject interactive CSS and JS into the SVG `<defs>` section.
fn inject_svginteractive(svg: String, diagram_type: &str) -> String {
    let (css, js) = if diagram_type == "SEQUENCE" {
        (SEQUENCE_INTERACTIVE_CSS, SEQUENCE_INTERACTIVE_JS)
    } else {
        (DEFAULT_INTERACTIVE_CSS, DEFAULT_INTERACTIVE_JS)
    };

    // Java readText() reads line-by-line and appends \n after each line,
    // effectively ensuring trailing newline. Replicate that behavior.
    let css_text = ensure_trailing_newline(css);
    let js_text = ensure_trailing_newline(js);

    let defs_content = format!(
        "<style type=\"text/css\"><![CDATA[\n{}]]></style><script>{}</script>",
        css_text,
        xml_escape_js(&js_text)
    );

    // Replace empty <defs/> with populated <defs>
    if let Some(pos) = svg.find("<defs/>") {
        let mut result = String::with_capacity(svg.len() + defs_content.len());
        result.push_str(&svg[..pos]);
        result.push_str("<defs>");
        result.push_str(&defs_content);
        result.push_str("</defs>");
        result.push_str(&svg[pos + 7..]);
        result
    } else if let Some(pos) = svg.find("<defs>") {
        // Already has <defs>...</defs> — inject at start of defs content
        let insert_pos = pos + 6;
        let mut result = String::with_capacity(svg.len() + defs_content.len());
        result.push_str(&svg[..insert_pos]);
        result.push_str(&defs_content);
        result.push_str(&svg[insert_pos..]);
        result
    } else {
        // No defs section found — insert before <g>
        if let Some(pos) = svg.find("<g>") {
            let mut result = String::with_capacity(svg.len() + defs_content.len() + 14);
            result.push_str(&svg[..pos]);
            result.push_str("<defs>");
            result.push_str(&defs_content);
            result.push_str("</defs>");
            result.push_str(&svg[pos..]);
            result
        } else {
            svg
        }
    }
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

fn wrap_with_meta(body_svg: &str, meta: &DiagramMeta, diagram_type: &str, bg: &str, raw_body_dim: Option<(f64, f64)>, skin: &crate::style::SkinParams) -> Result<String> {
    let (svg_w, svg_h) = extract_dimensions(body_svg);
    let body_content = extract_svg_content(body_svg);
    // Use raw body dimensions if available (avoids integer truncation loss).
    // Otherwise fall back to extracting from SVG header (lossy).
    let (body_w, body_h) = if let Some((rw, rh)) = raw_body_dim {
        (rw, rh)
    } else {
        // Body SVG includes DOC_MARGIN + 1: recover raw textBlock dimensions.
        (svg_w - DOC_MARGIN_RIGHT - 1.0, svg_h - DOC_MARGIN_BOTTOM - 1.0)
    };
    log::trace!("wrap_with_meta: svg_w={svg_w} svg_h={svg_h} body_w={body_w} body_h={body_h}");

    // ── Resolve document section styles ──────────────────────────────
    let hdr_font_size = skin.get("document.header.fontsize")
        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(META_HF_FONT_SIZE);
    let hdr_font_color = skin.get("document.header.fontcolor")
        .map(|s| s.to_string());
    let hdr_bg_color = skin.get("document.header.backgroundcolor")
        .map(|s| s.to_string());

    let ftr_font_size = skin.get("document.footer.fontsize")
        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(META_HF_FONT_SIZE);
    let ftr_font_color = skin.get("document.footer.fontcolor")
        .map(|s| s.to_string());
    let ftr_bg_color = skin.get("document.footer.backgroundcolor")
        .map(|s| s.to_string());

    let title_font_size = skin.get("document.title.fontsize")
        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(META_TITLE_FONT_SIZE);
    let title_font_color = skin.get("document.title.fontcolor")
        .map(|s| s.to_string());
    let title_bg_color = skin.get("document.title.backgroundcolor")
        .map(|s| s.to_string());

    let leg_font_size = skin.get("document.legend.fontsize")
        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(META_LEGEND_FONT_SIZE);
    let leg_font_color = skin.get("document.legend.fontcolor")
        .map(|s| s.to_string());
    let leg_bg_color = skin.get("document.legend.backgroundcolor")
        .map(|s| s.to_string());

    let cap_font_size = skin.get("document.caption.fontsize")
        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(META_CAPTION_FONT_SIZE);
    let cap_font_color = skin.get("document.caption.fontcolor")
        .map(|s| s.to_string());
    let cap_bg_color = skin.get("document.caption.backgroundcolor")
        .map(|s| s.to_string());

    let title_bold = title_font_size == META_TITLE_FONT_SIZE; // default title is bold

    // ── 1. Compute block dimensions for each meta element ───────────
    let hdr_text_w = meta.header.as_ref().map(|t| creole_text_w(t, hdr_font_size, false)).unwrap_or(0.0);
    let hdr_text_h = if meta.header.is_some() { text_block_h(hdr_font_size, false) } else { 0.0 };
    let hdr_dim = if meta.header.is_some() { block_dim(hdr_text_w, hdr_text_h, 0.0, 0.0) } else { (0.0, 0.0) };

    let ftr_text_w = meta.footer.as_ref().map(|t| creole_text_w(t, ftr_font_size, false)).unwrap_or(0.0);
    let ftr_text_h = if meta.footer.is_some() { text_block_h(ftr_font_size, false) } else { 0.0 };
    let ftr_dim = if meta.footer.is_some() { block_dim(ftr_text_w, ftr_text_h, 0.0, 0.0) } else { (0.0, 0.0) };

    let title_text_w = meta.title.as_ref().map(|t| creole_text_w(t, title_font_size, title_bold)).unwrap_or(0.0);
    let title_text_h = if meta.title.is_some() { text_block_h(title_font_size, title_bold) } else { 0.0 };
    let title_dim = if meta.title.is_some() { block_dim(title_text_w, title_text_h, TITLE_PADDING, TITLE_MARGIN) } else { (0.0, 0.0) };
    log::trace!("wrap_with_meta: title text_w={title_text_w:.10} text_h={title_text_h:.10} title_dim={title_dim:?}");

    let cap_text_w = meta.caption.as_ref().map(|t| creole_text_w(t, cap_font_size, false)).unwrap_or(0.0);
    let cap_text_h = if meta.caption.is_some() { text_block_h(cap_font_size, false) } else { 0.0 };
    let cap_dim = if meta.caption.is_some() { block_dim(cap_text_w, cap_text_h, CAPTION_PADDING, CAPTION_MARGIN) } else { (0.0, 0.0) };

    let leg_text_w = meta.legend.as_ref().map(|t| creole_text_w(t, leg_font_size, false)).unwrap_or(0.0);
    let leg_text_h = if meta.legend.is_some() { text_block_h(leg_font_size, false) } else { 0.0 };
    let leg_dim = if meta.legend.is_some() { block_dim(leg_text_w, leg_text_h, LEGEND_PADDING, LEGEND_MARGIN) } else { (0.0, 0.0) };

    // ── 2. Compute total dimensions (inside-out stacking) ──────────
    let body_dim = (body_w, body_h);
    let after_legend = merge_tb(body_dim, leg_dim);
    let after_title = merge_tb(title_dim, after_legend);
    let after_caption = merge_tb(after_title, cap_dim);
    let hf_dim = merge_tb(hdr_dim, ftr_dim);
    let total_dim = merge_tb(after_caption, hf_dim);
    // textBlock dimensions for positioning
    let tb_w = total_dim.0;
    let tb_h = total_dim.1;
    // Java ensureVisible: maxX = (int)(x + 1)
    let canvas_w = ensure_visible_int(tb_w + DOC_MARGIN_RIGHT) as f64;
    let canvas_h = ensure_visible_int(tb_h + DOC_MARGIN_BOTTOM) as f64;
    log::trace!("wrap_with_meta: tb_w={tb_w:.6} tb_h={tb_h:.6} canvas_w={canvas_w} canvas_h={canvas_h}");
    log::trace!("wrap_with_meta: body_dim=({body_w},{body_h}) after_legend={after_legend:?} after_title={after_title:?} after_caption={after_caption:?}");

    // ── 3. Compute absolute drawing positions ──────────────────────
    let outer_inner_x = ((tb_w - after_caption.0) / 2.0).max(0.0);
    let cap_inner_x = ((after_caption.0 - after_title.0) / 2.0).max(0.0);
    let title_inner_x = ((after_title.0 - after_legend.0) / 2.0).max(0.0);
    let leg_inner_x = ((after_legend.0 - body_w) / 2.0).max(0.0);

    let body_abs_x = outer_inner_x + cap_inner_x + title_inner_x + leg_inner_x;
    let body_abs_y = hdr_dim.1 + title_dim.1;
    log::trace!("body_pos: body_abs_x={body_abs_x:.6} body_abs_y={body_abs_y:.6}");

    // ── 4. Render SVG ──────────────────────────────────────────────
    let mut buf = String::with_capacity(body_svg.len() + 2048);
    write_svg_root_bg(&mut buf, canvas_w, canvas_h, diagram_type, bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, canvas_w, canvas_h, bg);

    // Header (RIGHT-aligned)
    if let Some(ref hdr) = meta.header {
        let hdr_x = tb_w - hdr_dim.0;
        let text_y = font_metrics::ascent("SansSerif", hdr_font_size, false, false);
        let text_color = hdr_font_color.as_deref().unwrap_or(DIVIDER_COLOR);
        write!(buf, r#"<g class="header""#).unwrap();
        if let Some(sl) = meta.header_line {
            write!(buf, r#" data-source-line="{sl}""#).unwrap();
        }
        buf.push('>');
        if let Some(ref bg) = hdr_bg_color {
            write!(buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="0"/>"#,
                bg, fmt_coord(hdr_text_h), fmt_coord(hdr_text_w), fmt_coord(hdr_x)
            ).unwrap();
        }
        render_creole_text(
            &mut buf, hdr, hdr_x, text_y,
            text_block_h(hdr_font_size, false),
            text_color, None,
            &format!(r#"font-size="{}""#, hdr_font_size as i32),
        );
        if buf.ends_with('\n') { buf.pop(); }
        buf.push_str("</g>");
    }

    // Title (CENTER-aligned)
    if let Some(ref title) = meta.title {
        let title_block_x = outer_inner_x + cap_inner_x
            + ((after_title.0 - title_dim.0) / 2.0).max(0.0);
        let text_x = title_block_x + TITLE_MARGIN + TITLE_PADDING;
        let text_y = hdr_dim.1 + TITLE_MARGIN + TITLE_PADDING
            + font_metrics::ascent("SansSerif", title_font_size, title_bold, false);
        let text_color = title_font_color.as_deref().unwrap_or(TEXT_COLOR);
        write!(buf, r#"<g class="title""#).unwrap();
        if let Some(sl) = meta.title_line {
            write!(buf, r#" data-source-line="{sl}""#).unwrap();
        }
        buf.push('>');
        if let Some(ref bg) = title_bg_color {
            let rect_x = title_block_x + TITLE_MARGIN;
            let rect_y = hdr_dim.1 + TITLE_MARGIN;
            let rect_w = title_text_w + 2.0 * TITLE_PADDING;
            let rect_h = title_text_h + 2.0 * TITLE_PADDING;
            write!(buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                bg, fmt_coord(rect_h), fmt_coord(rect_w), fmt_coord(rect_x), fmt_coord(rect_y)
            ).unwrap();
        }
        let weight_str = if title_bold { r#" font-weight="700""# } else { "" };
        render_creole_text(
            &mut buf, title, text_x, text_y,
            text_block_h(title_font_size, title_bold),
            text_color, None,
            &format!(r#"font-size="{}"{}"#, title_font_size as i32, weight_str),
        );
        if buf.ends_with('\n') { buf.pop(); }
        buf.push_str("</g>");
    }

    // Body — Java renders body at absolute coordinates (no <g transform>).
    // Strip the <defs/><g>...</g> wrapper from body_content (already have top-level <defs/>)
    // and shift coordinates by (body_abs_x, body_abs_y).
    let body_inner = body_content
        .strip_prefix("<defs/><g>").unwrap_or(&body_content);
    let body_inner = body_inner
        .strip_suffix("</g>").unwrap_or(body_inner);
    // Strip body-level background rect if present (wrap_with_meta provides its own).
    // Pattern: <rect fill="..." height="N" style="stroke:none;stroke-width:1;" width="N" x="0" y="0"/>
    let body_inner = if body_inner.starts_with("<rect fill=\"") {
        if let Some(end) = body_inner.find("/>") {
            let rect_tag = &body_inner[..end + 2];
            if rect_tag.contains("stroke:none") && rect_tag.contains("x=\"0\"") && rect_tag.contains("y=\"0\"") {
                &body_inner[end + 2..]
            } else {
                body_inner
            }
        } else {
            body_inner
        }
    } else {
        body_inner
    };
    if !body_inner.trim().is_empty() {
        if body_abs_x.abs() < 0.001 && body_abs_y.abs() < 0.001 {
            buf.push_str(body_inner);
        } else {
            let shifted = offset_svg_coords(body_inner, body_abs_x, body_abs_y);
            buf.push_str(&shifted);
        }
    }

    // Legend (CENTER-aligned)
    if let Some(ref leg) = meta.legend {
        let leg_wrapper_x = outer_inner_x + cap_inner_x + title_inner_x;
        let leg_wrapper_y = hdr_dim.1 + title_dim.1 + body_h;
        let leg_block_x = ((after_legend.0 - leg_dim.0) / 2.0).max(0.0);
        let rect_x = leg_wrapper_x + leg_block_x + LEGEND_MARGIN;
        let rect_y = leg_wrapper_y + LEGEND_MARGIN;
        let draw_w = leg_text_w + 2.0 * LEGEND_PADDING;
        let draw_h = leg_text_h + 2.0 * LEGEND_PADDING;
        let half_rc = LEGEND_ROUND_CORNER / 2.0;

        let legend_fill = leg_bg_color.as_deref().unwrap_or(LEGEND_BG);
        let text_color = leg_font_color.as_deref().unwrap_or(TEXT_COLOR);

        write!(buf, r#"<g class="legend""#).unwrap();
        // Java: legend includes data-source-line only when no document <style> block is used
        let has_style = leg_bg_color.is_some() || title_bg_color.is_some()
            || hdr_bg_color.is_some() || ftr_bg_color.is_some() || cap_bg_color.is_some();
        if !has_style {
            if let Some(sl) = meta.legend_line {
                write!(buf, r#" data-source-line="{sl}""#).unwrap();
            }
        }
        buf.push('>');
        write!(buf,
            r#"<rect fill="{}" height="{}" rx="{}" ry="{}" style="stroke:{LEGEND_BORDER};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
            legend_fill, fmt_coord(draw_h), fmt_coord(half_rc), fmt_coord(half_rc),
            fmt_coord(draw_w), fmt_coord(rect_x), fmt_coord(rect_y),
        ).unwrap();
        let text_x = rect_x + LEGEND_PADDING;
        let text_y = rect_y + LEGEND_PADDING
            + font_metrics::ascent("SansSerif", leg_font_size, false, false);
        render_creole_text(
            &mut buf, leg, text_x, text_y,
            text_block_h(leg_font_size, false),
            text_color, None,
            &format!(r#"font-size="{}""#, leg_font_size as i32),
        );
        if buf.ends_with('\n') { buf.pop(); }
        buf.push_str("</g>");
    }

    // Caption (CENTER-aligned)
    if let Some(ref cap) = meta.caption {
        let cap_y_start = hdr_dim.1 + after_title.1;
        let cap_block_x = outer_inner_x
            + ((after_caption.0 - cap_dim.0) / 2.0).max(0.0);
        let text_x = cap_block_x + CAPTION_MARGIN + CAPTION_PADDING;
        let text_y = cap_y_start + CAPTION_MARGIN + CAPTION_PADDING
            + font_metrics::ascent("SansSerif", cap_font_size, false, false);
        let text_color = cap_font_color.as_deref().unwrap_or(TEXT_COLOR);
        write!(buf, r#"<g class="caption""#).unwrap();
        if let Some(sl) = meta.caption_line {
            write!(buf, r#" data-source-line="{sl}""#).unwrap();
        }
        buf.push('>');
        if let Some(ref bg) = cap_bg_color {
            let rect_x = cap_block_x + CAPTION_MARGIN;
            let rect_y = cap_y_start + CAPTION_MARGIN;
            let rect_w = cap_text_w + 2.0 * CAPTION_PADDING;
            let rect_h = cap_text_h + 2.0 * CAPTION_PADDING;
            write!(buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                bg, fmt_coord(rect_h), fmt_coord(rect_w), fmt_coord(rect_x), fmt_coord(rect_y)
            ).unwrap();
        }
        render_creole_text(
            &mut buf, cap, text_x, text_y,
            text_block_h(cap_font_size, false),
            text_color, None,
            &format!(r#"font-size="{}""#, cap_font_size as i32),
        );
        if buf.ends_with('\n') { buf.pop(); }
        buf.push_str("</g>");
    }

    // Footer (CENTER-aligned)
    if let Some(ref ftr) = meta.footer {
        let ftr_y_start = hdr_dim.1 + after_caption.1;
        let ftr_x = ((tb_w - ftr_dim.0) / 2.0).max(0.0);
        let text_y = ftr_y_start
            + font_metrics::ascent("SansSerif", ftr_font_size, false, false);
        let text_color = ftr_font_color.as_deref().unwrap_or(DIVIDER_COLOR);
        write!(buf, r#"<g class="footer""#).unwrap();
        if let Some(sl) = meta.footer_line {
            write!(buf, r#" data-source-line="{sl}""#).unwrap();
        }
        buf.push('>');
        if let Some(ref bg) = ftr_bg_color {
            write!(buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                bg, fmt_coord(ftr_text_h), fmt_coord(ftr_text_w), fmt_coord(ftr_x), fmt_coord(ftr_y_start)
            ).unwrap();
        }
        render_creole_text(
            &mut buf, ftr, ftr_x, text_y,
            text_block_h(ftr_font_size, false),
            text_color, None,
            &format!(r#"font-size="{}""#, ftr_font_size as i32),
        );
        if buf.ends_with('\n') { buf.pop(); }
        buf.push_str("</g>");
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

/// Shift all coordinate attributes in SVG content by (dx, dy).
/// Java renders body at absolute coordinates; this replaces <g transform="translate">.
fn offset_svg_coords(svg: &str, dx: f64, dy: f64) -> String {
    use regex::Regex;
    use std::sync::OnceLock;

    // Match position attributes: x="N", y="N", cx="N", cy="N", x1="N", y1="N", x2="N", y2="N"
    static RE_X: OnceLock<Regex> = OnceLock::new();
    static RE_Y: OnceLock<Regex> = OnceLock::new();
    static RE_POINTS: OnceLock<Regex> = OnceLock::new();
    static RE_PATH_D: OnceLock<Regex> = OnceLock::new();

    let re_x = RE_X.get_or_init(|| Regex::new(r#"(?P<attr>(?:^| )(?:x|cx|x1|x2))="(?P<val>-?[\d.]+)""#).unwrap());
    let re_y = RE_Y.get_or_init(|| Regex::new(r#"(?P<attr> (?:y|cy|y1|y2))="(?P<val>-?[\d.]+)""#).unwrap());
    let re_points = RE_POINTS.get_or_init(|| Regex::new(r#"points="([^"]*)""#).unwrap());
    let re_path_d = RE_PATH_D.get_or_init(|| Regex::new(r#" d="([^"]*)""#).unwrap());

    let mut result = svg.to_string();

    // Shift x-coordinate attributes
    result = re_x.replace_all(&result, |caps: &regex::Captures| {
        let attr = &caps["attr"];
        let val: f64 = caps["val"].parse().unwrap_or(0.0);
        format!("{}=\"{}\"", attr, fmt_coord(val + dx))
    }).to_string();

    // Shift y-coordinate attributes
    result = re_y.replace_all(&result, |caps: &regex::Captures| {
        let attr = &caps["attr"];
        let val: f64 = caps["val"].parse().unwrap_or(0.0);
        format!("{}=\"{}\"", attr, fmt_coord(val + dy))
    }).to_string();

    // Shift polygon points="x,y x,y ..."
    result = re_points.replace_all(&result, |caps: &regex::Captures| {
        let points = &caps[1];
        let shifted: Vec<String> = points.split(',').collect::<Vec<_>>()
            .chunks(2)
            .filter_map(|pair| {
                if pair.len() == 2 {
                    let x: f64 = pair[0].trim().parse().unwrap_or(0.0);
                    let y: f64 = pair[1].trim().parse().unwrap_or(0.0);
                    Some(format!("{},{}", fmt_coord(x + dx), fmt_coord(y + dy)))
                } else { None }
            })
            .collect();
        format!("points=\"{}\"", shifted.join(","))
    }).to_string();

    // Shift path d="M x,y L x,y C x,y x,y x,y ..."
    result = re_path_d.replace_all(&result, |caps: &regex::Captures| {
        let d = &caps[1];
        let shifted = offset_path_data(d, dx, dy);
        format!(" d=\"{}\"", shifted)
    }).to_string();

    result
}

/// Offset all coordinates in an SVG path data string by (dx, dy).
fn offset_path_data(d: &str, dx: f64, dy: f64) -> String {
    let mut result = String::with_capacity(d.len());
    let mut chars = d.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            result.push(chars.next().unwrap());
        }
        if chars.peek().is_none() { break; }

        let c = *chars.peek().unwrap();
        if c.is_alphabetic() {
            result.push(chars.next().unwrap());
            continue;
        }

        // Parse number (x coordinate)
        if let Some(x) = parse_path_number(&mut chars) {
            result.push_str(&fmt_coord(x + dx));
            // Skip comma/space
            skip_path_sep(&mut chars, &mut result);
            // Parse y coordinate
            if let Some(y) = parse_path_number(&mut chars) {
                result.push_str(&fmt_coord(y + dy));
            }
        } else {
            // Unknown char, pass through
            if let Some(ch) = chars.next() {
                result.push(ch);
            }
        }
    }
    result
}

fn parse_path_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<f64> {
    let mut s = String::new();
    if chars.peek() == Some(&'-') {
        s.push(chars.next().unwrap());
    }
    while chars.peek().map_or(false, |c| c.is_ascii_digit() || *c == '.') {
        s.push(chars.next().unwrap());
    }
    if s.is_empty() || s == "-" { None } else { s.parse().ok() }
}

fn skip_path_sep(chars: &mut std::iter::Peekable<std::str::Chars>, result: &mut String) {
    while chars.peek().map_or(false, |c| *c == ',' || c.is_whitespace()) {
        result.push(chars.next().unwrap());
    }
}

// ── Class diagram rendering ─────────────────────────────────────────

fn render_class(
    cd: &crate::model::ClassDiagram,
    layout: &GraphLayout,
    skin: &SkinParams,
) -> Result<BodyResult> {
    // Java SvekResult: moveDelta(6 - minMax.getMinX(), 6 - minMax.getMinY())
    // minX depends on elements: rect gives (x-1), polygon HACK gives (x + local_minX - 10).
    // For class diagrams with protected/package visibility icons (UPolygon):
    //   icon at entity_x + margin(6) + translate(1) = entity_x + 7
    //   polygon local_minX = 0 → HACK min = entity_x + 7 - 10 = entity_x - 3
    //   vs rect: entity_x - 1
    // After normalization entity_x = 0: HACK min = -3, rect min = -1.
    // moveDelta = 6 - min(-3, -1) = 6 - (-3) = 9.
    //
    // Without polygon icons: moveDelta = 6 - (-1) = 7.
    // Java has two paths:
    // 1. EntityImageDegenerated (single entity, no links): delta=7, always offset=7.
    // 2. SvekResult (multi-entity): moveDelta(6 - minX, 6 - minY).
    //    minX = -1 (rect) or -3 (polygon HACK for protected/package member icons).
    let is_degenerated = layout.nodes.len() <= 1 && layout.edges.is_empty();
    let has_member_polygon_icon = !is_degenerated && cd.entities.iter().any(|e| {
        e.members.iter().any(|m| {
            matches!(m.visibility, Some(Visibility::Protected) | Some(Visibility::Package))
        })
    });
    let has_generic = !is_degenerated && cd.entities.iter().any(|e| e.generic.is_some());
    // Java SvekResult: moveDelta(6 - LimitFinder_minX, 6 - LimitFinder_minY).
    // LimitFinder_minX = polygon_minX - 1 (rect offset), so moveDelta = 7 default.
    // Our svek uses moveDelta = 6 - polygon_minX. Entity renders at polygon_minX + moveDelta = 6.
    // edge_offset = moveDelta + 1 (the LimitFinder rect -1 offset) = 7.
    let edge_offset_x = if has_member_polygon_icon { 9.0 } else { 7.0 };
    let edge_offset_y = if has_generic { 10.0 } else { 7.0 };
    let mut tracker = BoundsTracker::new();
    let mut sg = SvgGraphic::new(0, 1.0);
    let arrow_color = skin.arrow_color(LINK_COLOR);

    let node_map: HashMap<&str, &NodeLayout> =
        layout.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Build entity id map — IDs assigned by DEFINITION order (source_line),
    // not rendering order. Java assigns entity UIDs at parse time.
    let mut entity_ids: HashMap<String, String> = HashMap::new();
    let mut entities_by_def_order: Vec<&Entity> = cd.entities.iter().collect();
    entities_by_def_order.sort_by_key(|e| e.source_line.unwrap_or(usize::MAX));
    let mut ent_counter = 2u32; // Java starts entity IDs at ent0002
    for entity in &entities_by_def_order {
        let ent_id = format!("ent{:04}", ent_counter);
        entity_ids.insert(sanitize_id(&entity.name), ent_id);
        ent_counter += 1;
    }

    // Java: object diagrams do NOT emit <!--class X--> comments for entities,
    // only class diagrams do.
    let is_object_diagram = cd.entities.iter().all(|e| e.kind == EntityKind::Object);

    for entity in &cd.entities {
        let sid = sanitize_id(&entity.name);
        if let Some(nl) = node_map.get(sid.as_str()) {
            let ent_id = entity_ids
                .get(&sid)
                .map(|s| s.as_str())
                .unwrap_or("ent0000");
            if is_object_diagram {
                sg.push_raw(&format!(
                    "<g class=\"entity\" data-qualified-name=\"{}\"",
                    xml_escape(&entity.name),
                ));
            } else {
                sg.push_raw(&format!(
                    "<!--{} {}--><g class=\"entity\" data-qualified-name=\"{}\"",
                    // Java uses "class" for class entities, "entity" for others (rectangle, etc.)
                    if entity.kind == EntityKind::Rectangle { "entity" } else { "class" },
                    xml_escape(&entity.name),
                    xml_escape(&entity.name),
                ));
            }
            if let Some(source_line) = entity.source_line {
                sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
            }
            sg.push_raw(&format!(" id=\"{ent_id}\">"));
            draw_entity_box(&mut sg, &mut tracker, cd, entity, nl, skin, edge_offset_x, edge_offset_y);
            sg.push_raw("</g>");
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
            sg.push_raw(&format!(
                "<!--link {} to {}--><g class=\"link\" data-entity-1=\"{}\" data-entity-2=\"{}\" data-link-type=\"{}\"",
                xml_escape(&link.from),
                xml_escape(&link.to),
                from_ent,
                to_ent,
                link_type,
            ));
            if let Some(source_line) = link.source_line {
                sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
            }
            sg.push_raw(&format!(" id=\"lnk{link_counter}\">"));
            draw_edge(&mut sg, &mut tracker, link, el, arrow_color, edge_offset_x, edge_offset_y);
            sg.push_raw("</g>");
            link_counter += 1;
        }
    }

    // Notes
    for note in &layout.notes {
        draw_class_note(&mut sg, &mut tracker, note);
    }

    // Canvas size calculation depends on diagram complexity.
    //
    // Java uses two different paths:
    // 1. Multi-entity with links (SvekResult): LimitFinder_span + delta(15, 15) + doc_margin(5, 5)
    // 2. Single entity, no links (EntityImageDegenerated): entity_dim + delta(14, 14) + doc_margin(5, 5)
    //    EntityImageDegenerated.java: delta = 7, calculateDimension = orig.dim + delta*2 = orig.dim + 14
    //
    // We detect the case by checking layout structure.
    let is_degenerated = layout.nodes.len() <= 1 && layout.edges.is_empty();

    // Compute raw body content dimensions (Java SvekResult.calculateDimension).
    // These preserve full fractional precision for meta-wrapping.
    let raw_body_dim = if is_degenerated {
        let entity_w = if layout.nodes.is_empty() { 0.0 } else { layout.nodes[0].width };
        let entity_h = if layout.nodes.is_empty() { 0.0 } else { layout.nodes[0].height };
        let (calc_w, calc_h) = if layout.nodes.is_empty() {
            (10.0, 10.0)
        } else {
            const DEGENERATED_DELTA: f64 = 7.0;
            (entity_w + DEGENERATED_DELTA * 2.0, entity_h + DEGENERATED_DELTA * 2.0)
        };
        (calc_w, calc_h)
    } else {
        let (span_w, span_h) = tracker.span();
        (span_w + CANVAS_DELTA, span_h + CANVAS_DELTA)
    };

    let (svg_w, svg_h) = {
        let dim_w = raw_body_dim.0 + DOC_MARGIN_RIGHT;
        let dim_h = raw_body_dim.1 + DOC_MARGIN_BOTTOM;
        // Java ensureVisible: maxX = (int)(x + 1)
        let w = ensure_visible_int(dim_w);
        let h = ensure_visible_int(dim_h);
        (w as f64, h as f64)
    };

    let mut buf = String::with_capacity(sg.body().len() + 512);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, svg_w, svg_h, "CLASS", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(BodyResult { svg: buf, raw_body_dim: Some(raw_body_dim) })
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
fn emit_circle_glyph(sg: &mut SvgGraphic, tracker: &mut BoundsTracker, kind: &EntityKind, circle_cx: f64, circle_cy: f64) {
    let (glyph_raw, center) = match kind {
        EntityKind::Class | EntityKind::Object => (GLYPH_C_RAW, GLYPH_C_CENTER),
        EntityKind::Abstract => (GLYPH_A_RAW, GLYPH_A_CENTER),
        EntityKind::Interface => (GLYPH_I_RAW, GLYPH_I_CENTER),
        EntityKind::Enum => (GLYPH_E_RAW, GLYPH_E_CENTER),
        EntityKind::Annotation | EntityKind::Rectangle => return,
    };

    // Java DriverCenteredCharacterSvg algorithm:
    //   xpos = circle_center_in_ug - centerX - 0.5
    //   ypos = circle_center_in_ug - centerY - 0.5
    //   final = path_coord + (xpos, ypos)
    let dx = circle_cx - center.0 - 0.5;
    let dy = circle_cy - center.1 - 0.5;

    let mut d = String::with_capacity(512);
    let mut path_min_x = f64::INFINITY;
    let mut path_min_y = f64::INFINITY;
    let mut path_max_x = f64::NEG_INFINITY;
    let mut path_max_y = f64::NEG_INFINITY;
    for (cmd, points) in glyph_raw {
        d.push(*cmd);
        for (i, &(px, py)) in points.iter().enumerate() {
            if i > 0 {
                d.push(' ');
            }
            let final_x = px + dx;
            let final_y = py + dy;
            d.push_str(&fmt_coord(final_x));
            d.push(',');
            d.push_str(&fmt_coord(final_y));
            if final_x < path_min_x { path_min_x = final_x; }
            if final_y < path_min_y { path_min_y = final_y; }
            if final_x > path_max_x { path_max_x = final_x; }
            if final_y > path_max_y { path_max_y = final_y; }
        }
        // Java SvgGraphics: every command (including Z) has a trailing space
        d.push(' ');
    }

    sg.push_raw(&format!(r##"<path d="{d}" fill="#000000"/>"##));
    if path_min_x.is_finite() {
        tracker.track_path_bounds(path_min_x, path_min_y, path_max_x, path_max_y);
    }
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
        EntityKind::Rectangle => "#F1F1F1",
    }
}

fn draw_entity_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    cd: &ClassDiagram,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if entity.kind == EntityKind::Object {
        draw_object_box(sg, tracker, entity, nl, skin, edge_offset_x, edge_offset_y);
        return;
    }

    if entity.kind == EntityKind::Rectangle {
        draw_rectangle_entity_box(sg, tracker, entity, nl, skin, edge_offset_x, edge_offset_y);
        return;
    }

    // Java: entity rect starts at (moveDelta_offset + 1, moveDelta_offset + 1)
    // where the +1 is the border inset (rect drawn 1px inside the Graphviz node boundary)
    let x = nl.cx - nl.width / 2.0 + edge_offset_x;
    let y = nl.cy - nl.height / 2.0 + edge_offset_y;
    let w = nl.width;
    let h = nl.height;

    let (default_bg, default_border, element_type) = match entity.kind {
        EntityKind::Class => (ENTITY_BG, BORDER_COLOR, "class"),
        EntityKind::Interface => (ENTITY_BG, BORDER_COLOR, "interface"),
        EntityKind::Enum => (ENTITY_BG, BORDER_COLOR, "enum"),
        EntityKind::Abstract => (ENTITY_BG, BORDER_COLOR, "abstract"),
        EntityKind::Annotation => (ENTITY_BG, BORDER_COLOR, "annotation"),
        EntityKind::Rectangle => (ENTITY_BG, BORDER_COLOR, "rectangle"),
        EntityKind::Object => unreachable!(),
    };
    let default_fill = skin.background_color(element_type, default_bg);
    let fill = entity.color.as_deref().unwrap_or(default_fill);
    let stroke = skin.border_color(element_type, default_border);
    let font_color = skin.font_color(element_type, TEXT_COLOR);

    // Java URectangle.rounded(roundCorner): rx = roundCorner / 2.
    // Default roundCorner from style = 5 → rx = 2.5.
    // Java URectangle.rounded(roundCorner): SVG rx = roundCorner / 2.
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    // Rect with rx="2.5" ry="2.5" to match Java PlantUML
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, rx, rx, 0.0);
    tracker.track_rect(x, y, w, h);

    // Java font resolution:
    // - classFontSize controls the class name font size
    // - classAttributeFontSize controls member (field/method) font size
    // When only classFontSize is set, it applies to everything.
    // When both are set, classFontSize → name, classAttributeFontSize → members.
    let explicit_attr_fs = skin.get("classattributefontsize").and_then(|s| s.parse::<f64>().ok());
    let explicit_class_fs = skin.get("classfontsize").and_then(|s| s.parse::<f64>().ok());
    let attr_font_size = explicit_attr_fs.unwrap_or_else(|| explicit_class_fs.unwrap_or(FONT_SIZE));
    let class_font_size = explicit_class_fs.unwrap_or_else(|| explicit_attr_fs.unwrap_or(FONT_SIZE));

    // Entity name WITHOUT generic parameter — generic is rendered separately in draw_generic_box
    let name_display = entity.name.clone();
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
        let kind_fs = class_font_size - 2.0;
        let kind_tl_val = font_metrics::text_width(kind_text, "SansSerif", kind_fs, false, true);
        {
            let kind_tl = fmt_coord(kind_tl_val);
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="{fs:.0}" font-style="italic" lengthAdjust="spacing" text-anchor="middle" textLength="{kind_tl}" x="{}" y="{}">{kind_text}</text>"#,
                fmt_coord(cx), fmt_coord(kind_y), fs = kind_fs,
            ));
        }
        {
            let kind_ascent = font_metrics::ascent("SansSerif", kind_fs, false, true);
            let kind_descent = font_metrics::descent("SansSerif", kind_fs, false, true);
            tracker.track_rect(cx, kind_y - kind_ascent, kind_tl_val, kind_ascent + kind_descent);
        }
        let name_tl_val = font_metrics::text_width(&name_display, "SansSerif", class_font_size, true, false);
        {
            let name_tl = fmt_coord(name_tl_val);
            let name_escaped = xml_escape(&name_display);
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="{class_font_size:.0}" font-weight="700" lengthAdjust="spacing" text-anchor="middle" textLength="{name_tl}" x="{}" y="{}">{name_escaped}</text>"#,
                fmt_coord(cx), fmt_coord(name_y),
            ));
        }
        {
            let name_ascent = font_metrics::ascent("SansSerif", class_font_size, true, false);
            let name_descent = font_metrics::descent("SansSerif", class_font_size, true, false);
            tracker.track_rect(cx, name_y - name_ascent, name_tl_val, name_ascent + name_descent);
        }
    } else {
        let italic_name = entity.kind == EntityKind::Abstract;
        let name_width = font_metrics::text_width(
            &name_display,
            "SansSerif",
            class_font_size,
            false,
            italic_name,
        );
        // Compute name block height and baseline dynamically from actual font size
        let name_ascent = font_metrics::ascent("SansSerif", class_font_size, false, italic_name);
        let name_descent = font_metrics::descent("SansSerif", class_font_size, false, italic_name);
        let name_block_height = name_ascent + name_descent;
        let name_baseline = name_ascent;
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
            .max(stereo_height + name_block_height + HEADER_STEREO_NAME_GAP);
        let vis_icon_w = if entity.visibility.is_some() { ENTITY_VIS_ICON_BLOCK_SIZE } else { 0.0 };
        let gen_dim_w = if let Some(ref g) = entity.generic {
            let text_w = font_metrics::text_width(g, "SansSerif", GENERIC_FONT_SIZE, false, true);
            text_w + 2.0 * GENERIC_INNER_MARGIN + 2.0 * GENERIC_OUTER_MARGIN
        } else {
            0.0
        };
        let supp_width = (w - HEADER_CIRCLE_BLOCK_WIDTH - vis_icon_w - width_stereo_and_name - gen_dim_w).max(0.0);
        let h2 = (HEADER_CIRCLE_BLOCK_WIDTH / 4.0).min(supp_width * 0.1);
        let h1 = (supp_width - h2) / 2.0;

        let circle_color = stereotype_circle_color(&entity.kind);
        let circle_block_x = x + h1;
        let ecx = circle_block_x + 15.0;
        let ecy = y + header_height / 2.0;
        sg.set_fill_color(circle_color);
        sg.set_stroke_color(Some("#181818"));
        sg.set_stroke_width(1.0, None);
        sg.svg_ellipse(ecx, ecy, 11.0, 11.0, 0.0);
        tracker.track_ellipse(ecx, ecy, 11.0, 11.0);
        emit_circle_glyph(sg, tracker, &entity.kind, ecx, ecy);

        let header_top_offset = (header_height - stereo_height - name_block_height) / 2.0;
        let name_x = x + HEADER_CIRCLE_BLOCK_WIDTH + vis_icon_w + (width_stereo_and_name - name_block_width) / 2.0 + h1 + h2 + 3.0;

        if let Some(ref vis) = entity.visibility {
            let icon_x = name_x - ENTITY_VIS_ICON_BLOCK_SIZE;
            // Java: EntityImageClassHeader wraps visibility UBlock with
            // withMargin(top=4), then mergeLR(uBlock, name, CENTER).
            // uBlock dim = (11, 11), with margin: (11, 15).
            // name dim height = HEADER_NAME_BLOCK_HEIGHT (≈16.3).
            // merged height = max(15, name_h).
            // icon in merged: (merged_h - 15) / 2 + 4 (margin top).
            // merged in header: (header_h - merged_h) / 2.
            let icon_margin_top = 4.0;
            let icon_block_h = ENTITY_VIS_ICON_BLOCK_SIZE + icon_margin_top;
            let merged_h = name_block_height.max(icon_block_h);
            let merged_y = (header_height - stereo_height - merged_h) / 2.0;
            let icon_in_merged = (merged_h - icon_block_h) / 2.0 + icon_margin_top;
            let icon_y = y + merged_y + icon_in_merged;
            draw_visibility_icon(sg, tracker, vis, true, icon_x, icon_y);
        }

        for (idx, label) in visible_stereotypes.iter().enumerate() {
            let stereo_text = format!("\u{00AB}{label}\u{00BB}");
            let stereo_x = x + HEADER_CIRCLE_BLOCK_WIDTH + vis_icon_w + (width_stereo_and_name - stereo_widths[idx]) / 2.0 + h1 + h2;
            let stereo_y = y + header_top_offset + HEADER_STEREO_BASELINE + idx as f64 * HEADER_STEREO_LINE_HEIGHT;
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="12" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(stereo_widths[idx]),
                fmt_coord(stereo_x),
                fmt_coord(stereo_y),
                xml_escape(&stereo_text),
            ));
            tracker.track_rect(stereo_x, stereo_y - HEADER_STEREO_BASELINE, stereo_widths[idx], HEADER_STEREO_LINE_HEIGHT);
        }

        let name_y = y + header_top_offset + stereo_height + name_baseline;
        let font_style = if entity.kind == EntityKind::Abstract {
            Some("italic")
        } else {
            None
        };
        sg.set_fill_color(font_color);
        sg.svg_text(
            &name_display, name_x, name_y,
            Some("sans-serif"), class_font_size,
            None, font_style, None,
            name_width, LengthAdjust::Spacing,
            None, 0, None,
        );
        tracker.track_rect(name_x, name_y - name_baseline, name_width, name_block_height);
    }

    // Draw generic type box at top-right corner of entity rect
    if let Some(ref generic_text) = entity.generic {
        draw_generic_box(sg, tracker, generic_text, x, y, w);
    }

    let x1_val = fmt_coord(x + 1.0);
    let x2_val = fmt_coord(x + w - 1.0);
    let header_height = if has_kind_label {
        HEADER_HEIGHT
    } else {
        let dynamic_name_h = font_metrics::ascent("SansSerif", class_font_size, false, false)
            + font_metrics::descent("SansSerif", class_font_size, false, false);
        HEADER_CIRCLE_BLOCK_HEIGHT.max(
            visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT
                + dynamic_name_h
                + HEADER_STEREO_NAME_GAP,
        )
    };
    let mut section_y = y + header_height;
    // Java: member text uses classAttributeFontColor (defaults to TEXT_COLOR, not classFontColor)
    let attr_font_color = skin.font_color("classattribute", TEXT_COLOR);
    if show_fields {
        draw_member_section(
            sg,
            tracker,
            &visible_fields,
            section_y,
            x,
            &x1_val,
            &x2_val,
            attr_font_color,
            attr_font_size,
            stroke,
        );
        section_y += section_height_with_fs(&visible_fields, attr_font_size);
    }
    if show_methods {
        draw_member_section(
            sg,
            tracker,
            &visible_methods,
            section_y,
            x,
            &x1_val,
            &x2_val,
            attr_font_color,
            attr_font_size,
            stroke,
        );
    }
    // UEmpty: Java body.drawU emits an empty shape at the bottom-right of each entity.
    // drawEmpty(x, y, 1, 1) adds (x, y) to (x+1, y+1), but since the entity rect
    // already covers (x-1,y-1) to (x+w-1,y+h-1), this just extends max to (x+w, y+h).
    tracker.track_empty(x + w, y + h, 0.0, 0.0);
}

/// Draw the generic type box (dashed rect + italic text) at top-right of entity.
fn draw_generic_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    generic_text: &str,
    entity_x: f64,
    entity_y: f64,
    entity_w: f64,
) {
    let text_w = font_metrics::text_width(generic_text, "SansSerif", GENERIC_FONT_SIZE, false, true);
    let rect_w = text_w + 2.0 * GENERIC_INNER_MARGIN;
    let rect_h = GENERIC_TEXT_HEIGHT + 2.0 * GENERIC_INNER_MARGIN;
    let gen_dim_w = rect_w + 2.0 * GENERIC_OUTER_MARGIN;
    let gen_dim_h = rect_h + 2.0 * GENERIC_OUTER_MARGIN;

    // Outer block position: HeaderLayout.java:112
    let x_generic = entity_x + entity_w - gen_dim_w + GENERIC_DELTA;
    let y_generic = entity_y - GENERIC_DELTA;

    // Track outer margin wrapper UEmpty (Java withMargin draws UEmpty)
    tracker.track_empty(x_generic, y_generic, gen_dim_w, gen_dim_h);

    let rect_x = x_generic + GENERIC_OUTER_MARGIN;
    let rect_y = y_generic + GENERIC_OUTER_MARGIN;

    sg.set_fill_color("#FFFFFF");
    sg.set_stroke_color(Some("#181818"));
    sg.set_stroke_width(1.0, Some((2.0, 2.0)));
    sg.svg_rectangle(rect_x, rect_y, rect_w, rect_h, 0.0, 0.0, 0.0);
    tracker.track_rect(rect_x, rect_y, rect_w, rect_h);
    tracker.track_empty(rect_x, rect_y, rect_w, rect_h);

    let text_x = rect_x + GENERIC_INNER_MARGIN;
    let text_y = rect_y + GENERIC_INNER_MARGIN + GENERIC_BASELINE;
    sg.set_fill_color("#000000");
    sg.svg_text(
        generic_text, text_x, text_y,
        Some("sans-serif"), 12.0,
        None, Some("italic"), None,
        text_w, LengthAdjust::Spacing,
        None, 0, None,
    );
    tracker.track_rect(text_x, text_y - GENERIC_BASELINE, text_w, GENERIC_TEXT_HEIGHT);
}

/// Draw an Object entity box (EntityImageObject.java layout).
/// Render a rectangle entity with bracket-body description.
///
/// Java: rectangle entities have NO stereotype circle, NO title text, NO separator.
/// Only the bracket-body description lines are rendered as left-aligned text
/// at font-size 14 inside a rounded rect (rx=2.5).
fn draw_rectangle_entity_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    let x = nl.cx - nl.width / 2.0 + edge_offset_x;
    let y = nl.cy - nl.height / 2.0 + edge_offset_y;
    let w = nl.width;
    let h = nl.height;

    let fill = entity.color.as_deref().unwrap_or(ENTITY_BG);
    let stroke = skin.border_color("rectangle", BORDER_COLOR);
    let font_color = skin.font_color("rectangle", TEXT_COLOR);
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, rx, rx, 0.0);
    tracker.track_rect(x, y, w, h);

    // Java: description text at font-size 14, left-aligned, padding 10px
    let desc_font_size = 14.0_f64;
    let desc_lh = font_metrics::line_height("SansSerif", desc_font_size, false, false);
    let desc_ascent = font_metrics::ascent("SansSerif", desc_font_size, false, false);
    let text_x = x + 10.0;
    // Java: first text y = rect_y + padding(10) + ascent
    let first_y = y + 10.0 + desc_ascent;

    for (i, line) in entity.description.iter().enumerate() {
        let text_y = first_y + i as f64 * desc_lh;
        let tl = font_metrics::text_width(line, "SansSerif", desc_font_size, false, false);
        sg.set_fill_color(font_color);
        sg.svg_text(
            line, text_x, text_y,
            Some("sans-serif"), desc_font_size,
            None, None, None,
            tl,
            crate::klimt::svg::LengthAdjust::Spacing,
            None,
            0, // horizontal
            None,
        );
    }
}

///
/// Objects have NO stereotype circle icon, NO glyph path.
/// Name is centered with margin(2,2,2,2), no underline (default, non-strict UML).
/// Body is a single separator line followed by empty space (TextBlockEmpty(10, 16)).
fn draw_object_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    let x = nl.cx - nl.width / 2.0 + edge_offset_x;
    let y = nl.cy - nl.height / 2.0 + edge_offset_y;
    let w = nl.width;
    let h = nl.height;

    let default_fill = skin.background_color("object", ENTITY_BG);
    let fill = entity.color.as_deref().unwrap_or(default_fill);
    let stroke_color = skin.border_color("object", BORDER_COLOR);
    let font_color = skin.font_color("object", TEXT_COLOR);

    // Java URectangle.rounded(roundCorner): SVG rx = roundCorner / 2.
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    // Rect
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(stroke_color));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, rx, rx, 0.0);
    tracker.track_rect(x, y, w, h);

    // Object name constants — EntityImageObject.java
    // withMargin(tmp, 2, 2) → margin(top=2, right=2, bottom=2, left=2)
    const OBJ_NAME_MARGIN: f64 = 2.0;

    let class_font_size = skin.font_size("class", FONT_SIZE);
    let name_width = font_metrics::text_width(
        &entity.name,
        "SansSerif",
        class_font_size,
        false,
        false,
    );
    let name_block_width = name_width + 2.0 * OBJ_NAME_MARGIN;
    let name_block_height = HEADER_NAME_BLOCK_HEIGHT + 2.0 * OBJ_NAME_MARGIN;

    // PlacementStrategyY1Y2 with 1 element: x = (totalWidth - blockWidth) / 2
    // height = titleHeight = name_block_height, so space = 0, y = 0
    let name_offset_x = (w - name_block_width) / 2.0;
    let text_x = x + name_offset_x + OBJ_NAME_MARGIN;
    let text_y = y + OBJ_NAME_MARGIN + HEADER_NAME_BASELINE;

    sg.set_fill_color(font_color);
    sg.svg_text(
        &entity.name, text_x, text_y,
        Some("sans-serif"), class_font_size,
        None, None, None,
        name_width, LengthAdjust::Spacing,
        None, 0, None,
    );
    tracker.track_rect(text_x, text_y - HEADER_NAME_BASELINE, name_width, HEADER_NAME_BLOCK_HEIGHT);

    // Separator line at y + titleHeight
    let title_height = name_block_height;
    let sep_y = y + title_height;
    let x1 = x + 1.0;
    let x2 = x + w - 1.0;
    sg.set_stroke_color(Some(stroke_color));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(x1, sep_y, x2, sep_y, 0.0);
    tracker.track_line(x1, sep_y, x2, sep_y);

    // Render object fields in the body section
    let visible_fields: Vec<&Member> = entity
        .members
        .iter()
        .filter(|m| !m.is_method)
        .collect();
    if !visible_fields.is_empty() {
        let attr_font_size = skin.font_size("classattribute", class_font_size);
        let x1_val = fmt_coord(x1);
        let x2_val = fmt_coord(x2);
        draw_member_section(
            sg,
            tracker,
            &visible_fields,
            sep_y,
            x,
            &x1_val,
            &x2_val,
            font_color,
            attr_font_size,
            stroke_color,
        );
    }
}

fn draw_member_section(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    members: &[&Member],
    section_y: f64,
    x: f64,
    x1_val: &str,
    x2_val: &str,
    font_color: &str,
    attr_font_size: f64,
    sep_color: &str,
) {
    // Compute dynamic row metrics from attr_font_size (matches Java FontParam.CLASS_ATTRIBUTE)
    let row_h = font_metrics::line_height("SansSerif", attr_font_size, false, false);
    let attr_ascent = font_metrics::ascent("SansSerif", attr_font_size, false, false);
    let margin_top = 4.0;
    let text_y_offset = margin_top + attr_ascent;
    let icon_y_from_sep = margin_top + 2.0 + (row_h - 11.0) / 2.0;

    let _sep_y_str = fmt_coord(section_y);
    // Parse x1/x2 for line tracking
    let x1_f: f64 = x1_val.parse().unwrap_or(x + 1.0);
    let x2_f: f64 = x2_val.parse().unwrap_or(x);
    sg.set_stroke_color(Some(sep_color));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(x1_f, section_y, x2_f, section_y, 0.0);
    tracker.track_line(x1_f, section_y, x2_f, section_y);

    // visual_row tracks the current visual line index across all members
    let mut visual_row: usize = 0;

    for member in members.iter() {
        let text = member_text(member);
        let lines = split_member_lines(&text);
        let num_lines = lines.len();

        // Visibility icon: centered vertically across all visual lines of this member
        if let Some(visibility) = &member.visibility {
            let icon_y = section_y
                + icon_y_from_sep
                + visual_row as f64 * row_h
                + (num_lines.saturating_sub(1)) as f64 * row_h / 2.0;
            draw_visibility_icon(
                sg,
                tracker,
                visibility,
                member.is_method,
                x + MEMBER_ICON_X_OFFSET,
                icon_y,
            );
        }

        let font_style_attr: Option<&str> = if member.modifiers.is_abstract {
            Some("italic")
        } else {
            None
        };
        let text_deco_attr: Option<&str> = if member.modifiers.is_static {
            Some("underline")
        } else {
            None
        };

        let base_text_x = x
            + if member.visibility.is_some() {
                MEMBER_TEXT_X_WITH_ICON
            } else {
                MEMBER_TEXT_X_NO_ICON
            };

        for (line_idx, (line_text, indent)) in lines.iter().enumerate() {
            let text_y = section_y
                + text_y_offset
                + (visual_row + line_idx) as f64 * row_h;
            let text_x = if line_idx == 0 {
                base_text_x
            } else {
                base_text_x + indent
            };
            let text_width_val = font_metrics::text_width(
                line_text,
                "SansSerif",
                attr_font_size,
                false,
                member.modifiers.is_abstract,
            );
            sg.set_fill_color(font_color);
            sg.svg_text(
                line_text, text_x, text_y,
                Some("sans-serif"), attr_font_size,
                None, font_style_attr, text_deco_attr,
                text_width_val, LengthAdjust::Spacing,
                None, 0, None,
            );
            {
                let text_ascent = font_metrics::ascent(
                    "SansSerif",
                    attr_font_size,
                    false,
                    member.modifiers.is_abstract,
                );
                let text_descent = font_metrics::descent(
                    "SansSerif",
                    attr_font_size,
                    false,
                    member.modifiers.is_abstract,
                );
                tracker.track_rect(
                    text_x,
                    text_y - text_ascent,
                    text_width_val,
                    text_ascent + text_descent,
                );
            }
        }

        visual_row += num_lines;
    }
}

fn section_height_with_fs(members: &[&Member], attr_font_size: f64) -> f64 {
    if members.is_empty() {
        EMPTY_COMPARTMENT
    } else {
        let row_h = font_metrics::line_height("SansSerif", attr_font_size, false, false);
        let one_row_h = row_h + 8.0; // margin_top(4) + row_h + margin_bottom(4)
        let total_visual_lines: usize = members
            .iter()
            .map(|m| {
                let text = member_text(m);
                split_member_lines(&text).len()
            })
            .sum();
        one_row_h
            + (total_visual_lines.saturating_sub(1)) as f64 * row_h
    }
}

/// Java MemberImpl.getDisplay() format:
/// Uses raw display text when available (preserves original formatting).
/// Fallback: methods "name(): type", fields "name : type".
fn member_text(m: &Member) -> String {
    if let Some(ref display) = m.display {
        return display.clone();
    }
    match &m.return_type {
        Some(rt) if m.name.ends_with(')') => format!("{}: {rt}", m.name),
        Some(rt) => format!("{} : {rt}", m.name),
        None => m.name.clone(),
    }
}

/// Draw visibility modifier icon matching Java VisibilityModifier.java.
/// Colors and shapes from VisibilityModifier.java:
///   PUBLIC:    circle, fill=#84BE84(method)/none(field), stroke=#038048
///   PRIVATE:   square, fill=#F24D5C(method)/none(field), stroke=#C82930
///   PROTECTED: diamond, fill=#B38D22(method)/none(field), stroke=#B38D22
///   PACKAGE:   triangle, fill=#4177AF(method)/none(field), stroke=#1963A0
fn draw_visibility_icon(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    visibility: &Visibility,
    is_method: bool,
    x: f64,
    y: f64,
) {
    let modifier = match (visibility, is_method) {
        (Visibility::Public, true) => "PUBLIC_METHOD",
        (Visibility::Public, false) => "PUBLIC_FIELD",
        (Visibility::Private, true) => "PRIVATE_METHOD",
        (Visibility::Private, false) => "PRIVATE_FIELD",
        (Visibility::Protected, true) => "PROTECTED_METHOD",
        (Visibility::Protected, false) => "PROTECTED_FIELD",
        (Visibility::Package, true) => "PACKAGE_PRIVATE_METHOD",
        (Visibility::Package, false) => "PACKAGE_PRIVATE_FIELD",
    };
    sg.push_raw(&format!(r#"<g data-visibility-modifier="{modifier}">"#));
    match visibility {
        Visibility::Public => {
            // VisibilityModifier.drawCircle: translate(x+2,y+2), UEllipse(6,6)
            let ecx = x + 2.0 + 3.0;
            let ecy = y + 2.0 + 3.0;
            let cx = fmt_coord(ecx);
            let cy = fmt_coord(ecy);
            let fill = if is_method { "#84BE84" } else { "none" };
            sg.set_fill_color(fill);
            sg.set_stroke_color(Some("#038048"));
            sg.set_stroke_width(1.0, None);
            sg.svg_ellipse(ecx, ecy, 3.0, 3.0, 0.0);
            tracker.track_ellipse(ecx, ecy, 3.0, 3.0);
        }
        Visibility::Private => {
            // VisibilityModifier.drawSquare: translate(x+2,y+2), URectangle(6,6)
            let rect_x = x + 2.0;
            let rect_y = y + 2.0;
            let rx = fmt_coord(rect_x);
            let ry = fmt_coord(rect_y);
            let fill = if is_method { "#F24D5C" } else { "none" };
            sg.set_fill_color(fill);
            sg.set_stroke_color(Some("#C82930"));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(rect_x, rect_y, 6.0, 6.0, 0.0, 0.0, 0.0);
            tracker.track_rect(rect_x, rect_y, 6.0, 6.0);
        }
        Visibility::Protected => {
            // VisibilityModifier.drawDiamond: size -= 2 (10→8), translate(x+1,y+0), UPolygon
            // Points: (size/2,0),(size,size/2),(size/2,size),(0,size/2) where size=8
            let ox = x + 1.0;
            let oy = y;
            let fill = if is_method { "#B38D22" } else { "none" };
            let poly_pts = [
                (ox + 4.0, oy),
                (ox + 8.0, oy + 4.0),
                (ox + 4.0, oy + 8.0),
                (ox, oy + 4.0),
            ];
            sg.set_fill_color(fill);
            sg.set_stroke_color(Some("#B38D22"));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[
                poly_pts[0].0, poly_pts[0].1,
                poly_pts[1].0, poly_pts[1].1,
                poly_pts[2].0, poly_pts[2].1,
                poly_pts[3].0, poly_pts[3].1,
            ]);
            tracker.track_polygon(&poly_pts);
        }
        Visibility::Package => {
            // VisibilityModifier.drawTriangle: size -= 2 (10→8), translate(x+1,y+0)
            // Points: (size/2,1),(0,size-1),(size,size-1) where size=8
            let ox = x + 1.0;
            let oy = y;
            let fill = if is_method { "#4177AF" } else { "none" };
            let poly_pts = [
                (ox + 4.0, oy + 1.0),   // (size/2=4, 1)
                (ox, oy + 7.0),          // (0, size-1=7)
                (ox + 8.0, oy + 7.0),   // (size=8, size-1=7)
            ];
            sg.set_fill_color(fill);
            sg.set_stroke_color(Some("#1963A0"));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[
                poly_pts[0].0, poly_pts[0].1,
                poly_pts[1].0, poly_pts[1].1,
                poly_pts[2].0, poly_pts[2].1,
            ]);
            tracker.track_polygon(&poly_pts);
        }
    }
    sg.push_raw("</g>");
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

fn draw_edge(sg: &mut SvgGraphic, tracker: &mut BoundsTracker, link: &Link, el: &EdgeLayout, link_color: &str, edge_offset_x: f64, edge_offset_y: f64) {
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

    let d = build_edge_path_d(&path_points, edge_offset_x, edge_offset_y);

    // Track the edge path bounds (UPath style)
    {
        let mut p_min_x = f64::INFINITY;
        let mut p_min_y = f64::INFINITY;
        let mut p_max_x = f64::NEG_INFINITY;
        let mut p_max_y = f64::NEG_INFINITY;
        for &(px, py) in &path_points {
            let ax = px + edge_offset_x;
            let ay = py + edge_offset_y;
            if ax < p_min_x { p_min_x = ax; }
            if ay < p_min_y { p_min_y = ay; }
            if ax > p_max_x { p_max_x = ax; }
            if ay > p_max_y { p_max_y = ay; }
        }
        if p_min_x.is_finite() {
            tracker.track_path_bounds(p_min_x, p_min_y, p_max_x, p_max_y);
        }
    }

    // Java: dashed lines use stroke-dasharray:7,7; INSIDE the style attribute.
    let dash_style = if link.line_style == LineStyle::Dashed {
        "stroke-dasharray:7,7;"
    } else {
        ""
    };
    // Java Link.idCommentForSvg(): separator depends on decorations.
    // Java decor1 = head decoration (right_head), decor2 = tail decoration (left_head).
    let path_id = {
        let head = link.right_head != ArrowHead::None; // Java decor1
        let tail = link.left_head != ArrowHead::None;  // Java decor2
        if !head && tail {
            // looksLikeRevertedForSvg: decor1=NONE, decor2≠NONE
            format!("{}-backto-{}", link.from, link.to)
        } else if (!head && !tail) || (head && tail) {
            // looksLikeNoDecorAtAllSvg: both NONE or both non-NONE
            format!("{}-{}", link.from, link.to)
        } else {
            // default: decor1≠NONE, decor2=NONE → "FROM-to-TO"
            format!("{}-to-{}", link.from, link.to)
        }
    };
    {
        let mut path_elt = String::from("<path");
        if let Some(source_line) = link.source_line {
            write!(path_elt, r#" codeLine="{source_line}""#).unwrap();
        }
        write!(
            path_elt,
            r#" d="{d}" fill="none" id="{path_id}" style="stroke:{link_color};stroke-width:1;{dash_style}"/>"#,
        )
        .unwrap();
        sg.push_raw(&path_elt);
    }

    if link.left_head != ArrowHead::None {
        emit_arrowhead(sg, tracker, &link.left_head, &el.points, true, link_color, edge_offset_x, edge_offset_y);
    }
    if link.right_head != ArrowHead::None {
        emit_arrowhead(sg, tracker, &link.right_head, &el.points, false, link_color, edge_offset_x, edge_offset_y);
    }

    if let Some(label) = &link.label {
        let mid_idx = path_points.len() / 2;
        let (mx, _my) = path_points[mid_idx];
        let label_x = mx + edge_offset_x;
        // Java: label positioned at labelXY from Graphviz, not edge midpoint.
        // labelXY is top-left of the label area. Text center is offset by
        // wh.h/2 - 4 (SheetBlock internal top margin of 4px).
        // Java: label positioned at labelXY from Graphviz, not edge midpoint.
        // labelXY is top-left of the label area. Text center is offset by
        // wh.h/2 - 4 (SheetBlock internal top margin of 4px).
        let label_y = if let (Some((_, ly)), Some((_, wh))) = (el.label_xy, el.label_wh) {
            ly + edge_offset_y + wh / 2.0 - 4.0
        } else {
            _my + edge_offset_y - 6.0
        };
        draw_label(sg, label, label_x, label_y);
        // Track label text extent for bounding box (Java LimitFinder.drawText).
        let font_size = LINK_LABEL_FONT_SIZE;
        let lines = split_label_lines(label);
        let max_w = lines
            .iter()
            .map(|(t, _)| font_metrics::text_width(t, "SansSerif", font_size, false, false))
            .fold(0.0_f64, f64::max);
        let line_h = font_metrics::line_height("SansSerif", font_size, false, false);
        let block_x = label_x + 1.0;
        let total_h = lines.len() as f64 * line_h;
        let ascent = font_metrics::ascent("SansSerif", font_size, false, false);
        let base_y = label_y - total_h / 2.0 + ascent;
        for (idx, (line_text, _)) in lines.iter().enumerate() {
            let text_w = font_metrics::text_width(line_text, "SansSerif", font_size, false, false);
            let ly = base_y + idx as f64 * line_h;
            tracker.track_text(block_x, ly, text_w, line_h);
        }
        // Java: SvekEdge.addVisibilityModifier wraps the label TextBlock with
        // TextBlockMarged(marginLabel=1). TextBlockMarged.drawU emits UEmpty with
        // the full marged dimension (inner_w + 2, inner_h + 2). This extends the
        // bounding box 1px beyond the widest text line on the right.
        tracker.track_empty(label_x, base_y, max_w + 2.0, 0.0);
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

fn build_edge_path_d(points: &[(f64, f64)], offset_x: f64, offset_y: f64) -> String {
    let mut d = String::new();
    if points.is_empty() {
        return d;
    }

    write!(
        d,
        "M{},{} ",
        fmt_coord(points[0].0 + offset_x),
        fmt_coord(points[0].1 + offset_y),
    )
    .unwrap();

    let rest = &points[1..];
    if is_cubic_edge_path(points) {
        for chunk in rest.chunks(3) {
            write!(
                d,
                "C{},{} {},{} {},{} ",
                fmt_coord(chunk[0].0 + offset_x),
                fmt_coord(chunk[0].1 + offset_y),
                fmt_coord(chunk[1].0 + offset_x),
                fmt_coord(chunk[1].1 + offset_y),
                fmt_coord(chunk[2].0 + offset_x),
                fmt_coord(chunk[2].1 + offset_y),
            )
            .unwrap();
        }
    } else {
        for &(x, y) in rest {
            write!(
                d,
                "L{},{} ",
                fmt_coord(x + offset_x),
                fmt_coord(y + offset_y),
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
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    head: &ArrowHead,
    points: &[(f64, f64)],
    is_start: bool,
    link_color: &str,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if points.is_empty() || *head == ArrowHead::None {
        return;
    }

    let (tip_x, tip_y) = if is_start {
        (points[0].0 + edge_offset_x, points[0].1 + edge_offset_y)
    } else {
        let (x, y) = points[points.len() - 1];
        (x + edge_offset_x, y + edge_offset_y)
    };

    let base_angle = if is_start {
        edge_start_angle(points) + std::f64::consts::PI
    } else {
        edge_end_angle(points)
    };

    match head {
        ArrowHead::Arrow => emit_rotated_polygon(
            sg,
            tracker,
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
            sg,
            tracker,
            &[(0.0, 0.0), (-19.0, -7.0), (-19.0, 7.0), (0.0, 0.0)],
            base_angle + std::f64::consts::FRAC_PI_2,
            tip_x,
            tip_y,
            ENTITY_BG,
            link_color,
        ),
        ArrowHead::Diamond => emit_rotated_polygon(
            sg,
            tracker,
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
            sg,
            tracker,
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
        ArrowHead::Plus => emit_plus_head(sg, tracker, tip_x, tip_y, base_angle, link_color),
        ArrowHead::None => {}
    }
}

fn emit_rotated_polygon(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
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
    let mut rotated_points = Vec::with_capacity(points.len());
    for (idx, &(x, y)) in points.iter().enumerate() {
        if idx > 0 {
            pts.push(',');
        }
        let rx = x * cos - y * sin + tx;
        let ry = x * sin + y * cos + ty;
        write!(pts, "{},{}", fmt_coord(rx), fmt_coord(ry)).unwrap();
        rotated_points.push((rx, ry));
    }
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(1.0, None);
    sg.svg_polygon(0.0, &{
        let mut flat = Vec::with_capacity(rotated_points.len() * 2);
        for &(rx, ry) in &rotated_points {
            flat.push(rx);
            flat.push(ry);
        }
        flat
    });
    tracker.track_polygon(&rotated_points);
}

fn emit_plus_head(sg: &mut SvgGraphic, tracker: &mut BoundsTracker, tip_x: f64, tip_y: f64, angle: f64, link_color: &str) {
    let radius = 8.0;
    let center_x = tip_x + radius * angle.sin();
    let center_y = tip_y - radius * angle.cos();
    sg.set_fill_color("#FFFFFF");
    sg.set_stroke_color(Some(link_color));
    sg.set_stroke_width(1.0, None);
    sg.svg_circle(center_x, center_y, 8.0, 0.0);
    tracker.track_ellipse(center_x, center_y, radius, radius);

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
    sg.set_stroke_color(Some(link_color));
    sg.set_stroke_width(1.0, None);
    sg.svg_line(p1.0, p1.1, p2.0, p2.1, 0.0);
    tracker.track_line(p1.0, p1.1, p2.0, p2.1);
    sg.svg_line(p3.0, p3.1, p4.0, p4.1, 0.0);
    tracker.track_line(p3.0, p3.1, p4.0, p4.1);
}

fn point_on_circle(cx: f64, cy: f64, radius: f64, angle: f64) -> (f64, f64) {
    (cx + radius * angle.cos(), cy + radius * angle.sin())
}

/// Alignment type for a link label line segment.
#[derive(Clone, Copy, PartialEq)]
enum LabelAlign {
    Center,
    Left,
    Right,
}

/// Split a link label on `\n`, `\l`, `\r` break sequences.
///
/// Returns `(line_text, alignment)` pairs.  The alignment is determined by the
/// break character that *follows* the text: `\n` → center, `\l` → left,
/// `\r` → right.  The last segment (with no trailing break) defaults to center.
fn split_label_lines(text: &str) -> Vec<(String, LabelAlign)> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut buf = String::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => {
                    result.push((buf.clone(), LabelAlign::Center));
                    buf.clear();
                    i += 2;
                    continue;
                }
                'l' => {
                    result.push((buf.clone(), LabelAlign::Left));
                    buf.clear();
                    i += 2;
                    continue;
                }
                'r' => {
                    result.push((buf.clone(), LabelAlign::Right));
                    buf.clear();
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() || result.is_empty() {
        // Last segment: inherit alignment from previous segments, default center
        let align = result.last().map(|(_, a)| *a).unwrap_or(LabelAlign::Center);
        result.push((buf, align));
    }
    result
}

/// Render a link label.
///
/// Java PlantUML renders multiline labels (`\n`, `\l`, `\r`) as separate
/// `<text>` elements with font-size 13.  Alignment is per-line:
/// - `\n` = center-aligned (each line centered relative to the widest)
/// - `\l` = left-aligned   (all lines at the same left x)
/// - `\r` = right-aligned  (all lines right-aligned to the widest)
fn draw_label(sg: &mut SvgGraphic, text: &str, x: f64, y: f64) {
    let lines = split_label_lines(text);
    let font_family = "SansSerif";
    let font_size = LINK_LABEL_FONT_SIZE;
    let line_height = font_metrics::line_height(font_family, font_size, false, false);

    // Compute text widths for each line
    let widths: Vec<f64> = lines
        .iter()
        .map(|(t, _)| font_metrics::text_width(t, font_family, font_size, false, false))
        .collect();
    let max_width = widths.iter().cloned().fold(0.0_f64, f64::max);

    // Total block height
    let total_height = lines.len() as f64 * line_height;

    // Base x: left edge of the label block, positioned so the block center is at x
    let base_x = x + 1.0; // Java PlantUML offsets label 1px to the right of edge midpoint

    // Base y: center the label block vertically at y, then add ascent for first baseline
    let ascent = font_metrics::ascent(font_family, font_size, false, false);
    let base_y = y - total_height / 2.0 + ascent;

    let default_font = get_default_font_family_pub();

    for (idx, (line_text, align)) in lines.iter().enumerate() {
        let text_w = widths[idx];
        let line_x = match align {
            LabelAlign::Left => base_x,
            LabelAlign::Center => base_x + (max_width - text_w) / 2.0,
            LabelAlign::Right => base_x + (max_width - text_w),
        };
        let line_y = base_y + idx as f64 * line_height;

        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            line_text, line_x, line_y,
            Some(&default_font), font_size,
            None, None, None,
            text_w, LengthAdjust::Spacing,
            None, 0, None,
        );
    }
}

/// Draw a note in class diagrams (yellow sticky box with folded corner)
fn draw_class_note(sg: &mut SvgGraphic, tracker: &mut BoundsTracker, note: &ClassNoteLayout) {
    let x = note.x + MARGIN;
    let y = note.y + MARGIN;
    let w = note.width;
    let h = note.height;

    // body shape (use polygon instead of rect to clip the top-right fold area)
    let fold = NOTE_FOLD;
    // pentagon path: top-left -> top-right(minus fold) -> fold inner corner -> bottom-right -> bottom-left
    let note_poly = [
        (x, y),
        (x + w - fold, y),
        (x + w, y + fold),
        (x + w, y + h),
        (x, y + h),
    ];
    sg.set_fill_color(NOTE_BG);
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(1.0, None);
    sg.svg_polygon(0.0, &[
        note_poly[0].0, note_poly[0].1,
        note_poly[1].0, note_poly[1].1,
        note_poly[2].0, note_poly[2].1,
        note_poly[3].0, note_poly[3].1,
        note_poly[4].0, note_poly[4].1,
    ]);
    tracker.track_polygon(&note_poly);

    // fold corner triangle
    {
        let fold_pts = [
            (x + w - fold, y),
            (x + w - fold, y + fold),
            (x + w, y),
        ];
        let cx = fmt_coord(fold_pts[0].0);
        let cy = fmt_coord(fold_pts[0].1);
        let cy2 = fmt_coord(fold_pts[1].1);
        let cx2 = fmt_coord(fold_pts[2].0);
        sg.push_raw(&format!(
            r#"<path d="M{cx},{cy} L{cx},{cy2} L{cx2},{cy} Z " fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ));
        tracker.track_path_bounds(
            fold_pts[0].0.min(fold_pts[2].0),
            fold_pts[0].1.min(fold_pts[1].1),
            fold_pts[0].0.max(fold_pts[2].0),
            fold_pts[0].1.max(fold_pts[1].1),
        );
    }

    // text content
    let text_x = x + NOTE_TEXT_PADDING;
    let text_y = y + LINE_HEIGHT;
    {
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &note.text,
        text_x,
        text_y,
        LINE_HEIGHT,
        TEXT_COLOR,
        None,
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
        sg.push_raw(&tmp);
    }

    // connector line (dashed)
    if let Some((from_x, from_y, to_x, to_y)) = note.connector {
        let lx1 = from_x + MARGIN;
        let ly1 = from_y + MARGIN;
        let lx2 = to_x + MARGIN;
        let ly2 = to_y + MARGIN;
        sg.set_stroke_color(Some(NOTE_BORDER));
        sg.set_stroke_width(1.0, Some((5.0, 3.0)));
        sg.svg_line(lx1, ly1, lx2, ly2, 0.0);
        tracker.track_line(lx1, ly1, lx2, ly2);
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
            direction_explicit: false,
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
                    display: None,
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
                    display: None,
                },
            ],
            description: vec![],
            color: None,
            generic: None,
            source_line: None,
            visibility: None,
        };
        let entity2 = Entity {
            name: "Bar".into(),
            kind: EntityKind::Interface,
            stereotypes: vec![],
            members: vec![],
            description: vec![],
            color: None,
            generic: None,
            source_line: None,
            visibility: None,
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
            arrow_len: 2,
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
                label: None,
                label_xy: None,
                label_wh: None,
            }],
            notes: vec![],
            total_width: 240.0,
            total_height: 220.0, move_delta: (7.0, 7.0), normalize_offset: (0.0, 0.0), lf_span: (240.0, 220.0),
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
        assert_eq!(xml_escape(r#"a"b"#), r#"a"b"#);
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
            display: None,
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
            description: vec![],
            color: None,
            generic: None,
            source_line: None,
            visibility: None,
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
            total_height: 100.0, move_delta: (7.0, 7.0), normalize_offset: (0.0, 0.0), lf_span: (200.0, 100.0),
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
    fn test_object_entity_renders_without_circle_icon() {
        let entity = Entity {
            name: "myObj".into(),
            kind: EntityKind::Object,
            stereotypes: vec![],
            members: vec![],
            description: vec![],
            color: None,
            generic: None,
            source_line: None,
            visibility: None,
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
            total_height: 100.0, move_delta: (7.0, 7.0), normalize_offset: (0.0, 0.0), lf_span: (200.0, 100.0),
        };
        let svg = render(
            &Diagram::Class(cd),
            &DiagramLayout::Class(gl),
            &default_skin(),
            &default_meta(),
        )
        .expect("render failed");
        assert!(svg.contains("myObj"), "SVG must contain object name");
        // EntityImageObject: no underline by default (only in strict UML mode)
        assert!(
            !svg.contains(r#"text-decoration="underline""#),
            "object name must NOT have underline text-decoration by default"
        );
        // EntityImageObject: no stereotype circle icon
        assert!(
            !svg.contains("ellipse"),
            "object entity must NOT have stereotype circle"
        );
        // Must have exactly one separator line
        assert!(
            svg.contains("<line"),
            "object entity must have a separator line"
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
        assert!(svg.contains(&format!(r#"fill="{ENTITY_BG}""#)));
        assert!(svg.contains(&format!(r#"stroke:{BORDER_COLOR}"#)));
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
        assert!(svg.contains("font-weight=\"700\""));
        assert!(svg.contains("font-size=\"14\""));
        // Body coordinates are now shifted inline (no <g transform>)
        assert!(!svg.contains("translate("), "body should use inline coordinate offset, not <g transform>");
    }

    #[test]
    fn test_meta_title_can_expand_canvas_width() {
        let (d, l) = simple_diagram();
        let body_result = render_body(&d, &l, &default_skin()).unwrap();
        let (body_w, _) = extract_dimensions(&body_result.svg);
        let meta = DiagramMeta {
            title: Some(
                "This is a deliberately very long title with [[https://example.com Link]]".into(),
            ),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        let (svg_w, _) = extract_dimensions(&svg);
        assert!(svg_w > body_w);
        // Body coordinates are shifted inline, not via <g transform>
    }

    #[test]
    fn test_meta_title_renders_creole_and_link() {
        let (d, l) = simple_diagram();
        let meta = DiagramMeta {
            title: Some("**Bold** [[https://example.com{hover} Link]]".into()),
            ..Default::default()
        };
        let svg = render(&d, &l, &default_skin(), &meta).unwrap();
        assert!(svg.contains(r#"font-weight="700""#));
        assert!(svg.contains(r#"href="https://example.com""#));
        assert!(svg.contains(r#"title="hover""#));
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
        assert!(svg.contains(LEGEND_BORDER));
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
            ..Default::default()
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
    fn test_dot_suppressed_produces_valid_svg() {
        let svg = render_dot_suppressed();
        assert!(svg.contains("<svg"), "must contain <svg tag");
        assert!(svg.contains("</svg>"), "must contain </svg> tag");
        assert!(svg.contains("suppressed"), "must contain suppressed message");
        assert!(svg.contains("2495"), "must reference issue 2495");
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
            description: vec![],
            color: None,
            generic: None,
            source_line: None,
            visibility: None,
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
            total_height: 120.0, move_delta: (7.0, 7.0), normalize_offset: (0.0, 0.0), lf_span: (200.0, 100.0),
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
            total_height: 60.0, move_delta: (7.0, 7.0), normalize_offset: (0.0, 0.0), lf_span: (200.0, 100.0),
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
