use std::collections::HashMap;
use std::fmt::Write;
use std::io::Write as IoWrite;

use flate2::write::DeflateEncoder;
use flate2::Compression;

use crate::layout::class_group_header_metrics;
use crate::layout::graphviz::{
    has_link_arrow_indicator, is_link_arrow_backward, strip_link_arrow_text, ClassNoteLayout,
    ClusterLayout, EdgeLayout, GraphLayout, NodeLayout,
};
use crate::layout::split_member_lines;
use crate::layout::DiagramLayout;
use crate::model::{
    ArrowHead, ClassDiagram, ClassHideShowRule, ClassPortion, ClassRuleTarget, Diagram,
    DiagramMeta, Entity, EntityKind, GroupKind, LineStyle, Link, Member, RectSymbol, Visibility,
};
use crate::style::SkinParams;
use crate::Result;

use crate::font_metrics;
use crate::klimt::sanitize_group_metadata_value;
use crate::klimt::svg::{svg_comment_escape, LengthAdjust, SvgGraphic};
use crate::svek::edge::LineOfSegments;

use super::svg_richtext::{
    count_creole_lines, creole_plain_text, creole_table_width, get_default_font_family_pub,
    max_creole_plain_line_len, render_creole_display_lines, render_creole_text,
    set_default_font_family,
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
const HEADER_STEREO_BLOCK_MARGIN: f64 = 2.0;

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
    BORDER_COLOR, DIVIDER_COLOR, ENTITY_BG, LEGEND_BG, LEGEND_BORDER, NOTE_BG, NOTE_BORDER,
    NOTE_FOLD, NOTE_PADDING as NOTE_TEXT_PADDING, TEXT_COLOR,
};
const CLASS_NOTE_FOLD: f64 = 10.0;
const LINK_COLOR: &str = BORDER_COLOR;
/// Java PlantUML renders link labels at font-size 13 (not 14).
const LINK_LABEL_FONT_SIZE: f64 = 13.0;
const PLANTUML_VERSION: &str = "1.2026.2";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum QualifierEndpoint {
    Tail,
    Head,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct QualifierKey {
    link_idx: usize,
    endpoint: QualifierEndpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum KalPosition {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
struct KalPlacement {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    shift_x: f64,
}

pub(crate) use crate::klimt::svg::fmt_coord;

/// Write a Java PlantUML-compatible SVG root element and open a `<g>` wrapper.
pub(crate) fn write_svg_root(buf: &mut String, w: f64, h: f64, diagram_type: &str) {
    write_svg_root_bg(buf, w, h, diagram_type, "#FFFFFF");
}

pub(crate) fn write_svg_root_bg(buf: &mut String, w: f64, h: f64, diagram_type: &str, bg: &str) {
    write_svg_root_bg_opt(buf, w, h, Some(diagram_type), bg);
}

/// Write an SVG `<title>` element (the document-level title, not a visible element).
/// Java emits this via `SvgGraphics` when the diagram has a `title` directive.
/// Must be called right after `write_svg_root_bg*`, before `<defs/>`.
pub(crate) fn write_svg_title(buf: &mut String, title: &str) {
    use crate::klimt::svg::xml_escape;
    write!(buf, "<title>{}</title>", xml_escape(title)).unwrap();
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
    let wi = if w.is_finite() && w > 0.0 {
        w as i32
    } else {
        100
    };
    let hi = if h.is_finite() && h > 0.0 {
        h as i32
    } else {
        100
    };
    write!(buf, "<?plantuml {PLANTUML_VERSION}?>").unwrap();
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
}

fn sanitize_id(name: &str) -> String {
    name.replace('<', "_LT_")
        .replace('>', "_GT_")
        .replace(',', "_COMMA_")
        .replace('.', "_DOT_")
        .replace(' ', "_")
}

pub(crate) use crate::klimt::svg::xml_escape;

fn svg_group_metadata_attr(value: &str) -> String {
    xml_escape(&sanitize_group_metadata_value(value))
}

fn class_link_id_for_svg(link: &Link) -> String {
    let from = crate::layout::class_entity_display_name(&link.from);
    let to = crate::layout::class_entity_display_name(&link.to);
    if link_looks_reverted_for_svg(link) {
        format!("{from}-backto-{to}")
    } else if link_looks_no_decor_at_all_svg(link) {
        format!("{from}-{to}")
    } else {
        format!("{from}-to-{to}")
    }
}

fn link_looks_reverted_for_svg(link: &Link) -> bool {
    link.left_head != ArrowHead::None && link.right_head == ArrowHead::None
}

fn link_looks_no_decor_at_all_svg(link: &Link) -> bool {
    (link.left_head == ArrowHead::None && link.right_head == ArrowHead::None)
        || (link.left_head != ArrowHead::None && link.right_head != ArrowHead::None)
}

/// Write a background `<rect>` covering the entire canvas when the background
/// color differs from the default #FFFFFF. Java PlantUML emits this rect as the
/// first child of `<g>` when `skinparam backgroundColor` is set.
pub(crate) fn write_bg_rect(buf: &mut String, w: f64, h: f64, bg: &str) {
    if !bg.eq_ignore_ascii_case("#FFFFFF") {
        let wi = if w.is_finite() && w > 0.0 {
            w as i32
        } else {
            100
        };
        let hi = if h.is_finite() && h > 0.0 {
            h as i32
        } else {
            100
        };
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
    struct SvgSeedGuard;
    impl Drop for SvgSeedGuard {
        fn drop(&mut self) {
            crate::klimt::svg::set_svg_id_seed_override(None);
        }
    }
    crate::klimt::svg::set_svg_id_seed_override(source.map(crate::klimt::svg::java_source_seed));
    let _svg_seed_guard = SvgSeedGuard;

    // For activity diagrams with meta elements (title/header/footer),
    // pre-compute the body offset so the body renderer can emit absolute
    // coordinates directly, avoiding lossy string-level coordinate shifting.
    let activity_body_offset = if matches!(diagram, Diagram::Activity(_)) && !meta.is_empty() {
        Some(compute_meta_body_offset(meta, skin))
    } else {
        None
    };

    // Note: handwritten mode does NOT change fonts, only jiggling shapes.
    set_default_font_family(None);
    let body_result = render_body(diagram, layout, skin, activity_body_offset)?;
    set_default_font_family(None);

    // Extract diagram type from body SVG
    let dtype = body_result
        .svg
        .find("data-diagram-type=\"")
        .and_then(|pos| {
            let start = pos + 19;
            body_result.svg[start..]
                .find('"')
                .map(|end| body_result.svg[start..start + end].to_string())
        })
        .unwrap_or_else(|| "CLASS".to_string());

    // EBNF and Regex diagrams handle their own title rendering in the body.
    // Clear meta.title so wrap_with_meta doesn't add a duplicate visible title.
    let meta_for_wrap;
    let effective_meta = if matches!(dtype.as_str(), "EBNF" | "REGEX") && meta.title.is_some() {
        meta_for_wrap = DiagramMeta {
            title: None,
            title_line: None,
            ..meta.clone()
        };
        &meta_for_wrap
    } else {
        meta
    };
    let mut svg = if effective_meta.is_empty() && !meta.pragmas.contains_key("svginteractive") {
        body_result.svg
    } else {
        // Document-level BackGroundColor from <style> is stored as "document.backgroundcolor";
        // skinparam BackGroundColor is stored as "backgroundcolor". Try both.
        let bg = skin
            .get("document.backgroundcolor")
            .or_else(|| skin.get("backgroundcolor"))
            .unwrap_or("#FFFFFF");
        wrap_with_meta(
            &body_result.svg,
            effective_meta,
            &dtype,
            bg,
            body_result.raw_body_dim,
            body_result.body_pre_offset,
            skin,
        )?
    };

    // Inject svginteractive CSS/JS if pragma is set
    if meta
        .pragmas
        .get("svginteractive")
        .map_or(false, |v| v == "true")
    {
        svg = inject_svginteractive(svg, &dtype);
    }

    // Java PlantUML suppresses DOT rendering with a simple notice SVG
    // that does not include the plantuml-src processing instruction.
    let is_dot = matches!(diagram, Diagram::Dot(_));
    if !is_dot {
        if let Some(source) = source {
            svg = inject_plantuml_source(svg, source)?;
        }
    }

    Ok(svg)
}

/// Compute the body (dx, dy) offset for meta wrapping.
///
/// This is the offset from SVG origin to the body content start, accounting
/// for header and title dimensions. Used to pre-apply the offset in the body
/// renderer, avoiding lossy string-level coordinate shifting.
fn compute_meta_body_offset(meta: &DiagramMeta, skin: &SkinParams) -> (f64, f64) {
    let title_font_size = skin
        .get("document.title.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_TITLE_FONT_SIZE);
    let title_bold = title_font_size == META_TITLE_FONT_SIZE;

    let hdr_font_size = skin
        .get("document.header.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_HF_FONT_SIZE);
    let hdr_text_h = if meta.header.is_some() {
        text_block_h(hdr_font_size, false)
    } else {
        0.0
    };
    let hdr_text_w = meta
        .header
        .as_ref()
        .map(|t| creole_text_w(t, hdr_font_size, false))
        .unwrap_or(0.0);
    let hdr_dim = if meta.header.is_some() {
        block_dim(hdr_text_w, hdr_text_h, 0.0, 0.0)
    } else {
        (0.0, 0.0)
    };

    let title_text_h = if let Some(ref t) = meta.title {
        let lh = font_metrics::line_height("SansSerif", title_font_size, title_bold, false);
        let n_lines = t.split(crate::NEWLINE_CHAR).flat_map(|s| s.lines()).count().max(1);
        let mut h = n_lines as f64 * lh;
        // Java: tables inside title use AtomWithMargin(table, 2, 2) adding 4px.
        let has_table = t.split(crate::NEWLINE_CHAR)
            .flat_map(|s| s.lines())
            .any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('|')
                    || (trimmed.starts_with('<') && trimmed.contains(">|"))
            });
        if has_table {
            h += 4.0;
        }
        h
    } else {
        0.0
    };
    let title_text_w = meta
        .title
        .as_ref()
        .map(|t| {
            creole_table_width(t, title_font_size, title_bold)
                .unwrap_or_else(|| creole_text_w(t, title_font_size, title_bold))
        })
        .unwrap_or(0.0);
    let title_dim = if meta.title.is_some() {
        block_dim(title_text_w, title_text_h, TITLE_PADDING, TITLE_MARGIN)
    } else {
        (0.0, 0.0)
    };

    // body_abs_y = hdr_dim.1 + title_dim.1
    // body_abs_x = centering terms (typically 0 when body is wider than meta)
    (0.0, hdr_dim.1 + title_dim.1)
}

/// Body rendering result: (svg_string, raw_body_content_dimensions).
/// The raw dimensions are the precise body content size (Java SvekResult.calculateDimension)
/// before DOC_MARGIN and ensureVisible integer truncation. When present, wrap_with_meta
/// uses these instead of extracting lossy integer dimensions from the SVG header.
struct BodyResult {
    svg: String,
    raw_body_dim: Option<(f64, f64)>,
    /// When true, body coordinates already include the meta offset (body_abs_x/y).
    /// wrap_with_meta should NOT apply offset_svg_coords.
    body_pre_offset: bool,
}

fn render_body(
    diagram: &Diagram,
    layout: &DiagramLayout,
    skin: &SkinParams,
    activity_body_offset: Option<(f64, f64)>,
) -> Result<BodyResult> {
    match (diagram, layout) {
        (Diagram::Class(cd), DiagramLayout::Class(gl)) => render_class(cd, gl, skin),
        (Diagram::Sequence(sd), DiagramLayout::Sequence(sl)) => {
            // Sequence layout total_width/total_height include document margins
            // (top=5, right=5, bottom=5 for Puma2). Recover raw textBlock dimensions.
            let margin_top = 5.0;
            let margin_right = DOC_MARGIN_RIGHT;
            let margin_bottom = DOC_MARGIN_BOTTOM;
            // For diagrams with right-boundary arrows (`A ->]`), the layout's
            // `total_width` does not include the trailing arrow-head polygon
            // (Java's LimitFinder HACK_X_FOR_POLYGON = 10). `render_sequence`
            // runs a LimitFinder-style bounds pass and bakes the correct width
            // into the inner SVG header — we pick that up here. Guard on the
            // presence of `]` (right-border) or `[` (left-border) endpoints so
            // normal diagrams keep using `sl.total_width` and avoid the
            // off-by-one ceiling slop that measure_sequence_body_dim can
            // introduce for tightly packed layouts.
            let has_boundary_arrow =
                sd.events.iter().any(|e| matches!(e,
                    crate::model::sequence::SeqEvent::Message(m)
                        if m.to == "]" || m.from == "["));
            svg_sequence::render_sequence(sd, sl, skin).map(|svg| {
                let (body_w, body_h) = if has_boundary_arrow {
                    let (rendered_w, rendered_h) = extract_dimensions(&svg);
                    (
                        (rendered_w - margin_right).max(sl.total_width - margin_right),
                        (rendered_h - margin_top - margin_bottom)
                            .max(sl.total_height - margin_top - margin_bottom),
                    )
                } else {
                    (
                        sl.total_width - margin_right,
                        sl.total_height - margin_top - margin_bottom,
                    )
                };
                BodyResult {
                    svg,
                    raw_body_dim: Some((body_w, body_h)),
                    body_pre_offset: false,
                }
            })
        }
        (Diagram::Activity(ad), DiagramLayout::Activity(al)) => {
            super::svg_activity::render_activity(ad, al, skin, activity_body_offset).map(
                |(svg, raw_body_dim)| BodyResult {
                    svg,
                    raw_body_dim,
                    body_pre_offset: activity_body_offset.is_some(),
                },
            )
        }
        (Diagram::State(sd), DiagramLayout::State(sl)) => {
            super::svg_state::render_state(sd, sl, skin).map(|(svg, raw_body_dim)| BodyResult {
                svg,
                raw_body_dim,
                body_pre_offset: false,
            })
        }
        (Diagram::Component(cd), DiagramLayout::Component(cl)) => {
            super::svg_component::render_component(cd, cl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Chart(cd), DiagramLayout::Chart(cl)) => {
            super::svg_chart::render_chart(cd, cl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Files(fd), DiagramLayout::Files(fl)) => {
            super::svg_files::render_files(fd, fl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Ditaa(dd), DiagramLayout::Ditaa(dl)) => {
            super::svg_ditaa::render_ditaa(dd, dl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Erd(ed), DiagramLayout::Erd(el)) => {
            super::svg_erd::render_erd(ed, el, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Gantt(gd), DiagramLayout::Gantt(gl)) => {
            super::svg_gantt::render_gantt(gd, gl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Json(jd), DiagramLayout::Json(jl)) => super::svg_json::render_json(jd, jl, skin)
            .map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            }),
        (Diagram::Mindmap(md), DiagramLayout::Mindmap(ml)) => {
            let raw_body_dim = ml.raw_body_dim;
            super::svg_mindmap::render_mindmap(md, ml, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim,
                body_pre_offset: false,
            })
        }
        (Diagram::Nwdiag(nd), DiagramLayout::Nwdiag(nl)) => {
            super::svg_nwdiag::render_nwdiag(nd, nl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Salt(sd), DiagramLayout::Salt(sl)) => super::svg_salt::render_salt(sd, sl, skin)
            .map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            }),
        (Diagram::Timing(td), DiagramLayout::Timing(tl)) => {
            super::svg_timing::render_timing(td, tl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Wbs(wd), DiagramLayout::Wbs(wl)) => {
            super::svg_wbs::render_wbs(wd, wl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Yaml(yd), DiagramLayout::Yaml(yl)) => super::svg_json::render_yaml(yd, yl, skin)
            .map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            }),
        (Diagram::UseCase(ud), DiagramLayout::Component(cl)) => {
            // Use case diagrams are routed through the component rendering pipeline.
            let cd = crate::model::component::ComponentDiagram::from(ud);
            super::svg_component::render_component(&cd, cl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Dot(_dd), DiagramLayout::Dot(_gl)) => {
            // Java PlantUML suppresses DOT rendering
            Ok(BodyResult {
                svg: render_dot_suppressed(),
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Packet(pd), DiagramLayout::Packet(pl)) => {
            super::svg_packet::render_packet(pd, pl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Git(gd), DiagramLayout::Git(gl)) => {
            super::svg_git::render_git(gd, gl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Regex(rd), DiagramLayout::Regex(rl)) => {
            super::svg_regex::render_regex(rd, rl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Ebnf(ed), DiagramLayout::Ebnf(el)) => {
            super::svg_ebnf::render_ebnf(ed, el, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Pie(pd), DiagramLayout::Pie(pl)) => {
            super::svg_pie::render_pie(pd, pl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Board(bd), DiagramLayout::Board(bl)) => {
            super::svg_board::render_board(bd, bl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Chronology(cd), DiagramLayout::Chronology(cl)) => {
            super::svg_chronology::render_chronology(cd, cl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        (Diagram::Hcl(hd), DiagramLayout::Hcl(hl)) => {
            super::svg_hcl::render_hcl(hd, hl, skin).map(|svg| BodyResult {
                svg,
                raw_body_dim: None,
                body_pre_offset: false,
            })
        }
        _ => Err(crate::Error::Render("diagram/layout type mismatch".into())),
    }
}

/// Render a suppressed-feature notice for DOT diagrams, matching Java PlantUML.
fn render_dot_suppressed() -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\">\n",
    );
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
    (
        text_w + 2.0 * padding + BORDERED_EXTRA,
        text_h + 2.0 * padding + BORDERED_EXTRA,
    )
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
        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    /// Java LimitFinder.drawRectangle: (x-1, y-1) to (x+w-1+shadow*2, y+h-1+shadow*2)
    pub fn track_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.track_rect_shadow(x, y, w, h, 0.0);
    }

    /// Java LimitFinder.drawRectangle with delta shadow
    pub fn track_rect_shadow(&mut self, x: f64, y: f64, w: f64, h: f64, shadow: f64) {
        log::trace!(
            "BoundsTracker.drawRect x={:.2} y={:.2} w={:.2} h={:.2} shadow={:.2}",
            x,
            y,
            w,
            h,
            shadow
        );
        self.add_point(x - 1.0, y - 1.0);
        self.add_point(x + w - 1.0 + shadow * 2.0, y + h - 1.0 + shadow * 2.0);
    }

    /// Java LimitFinder.drawEmpty: (x, y) to (x+w, y+h) — NO -1 adjustment
    pub fn track_empty(&mut self, x: f64, y: f64, w: f64, h: f64) {
        log::trace!(
            "BoundsTracker.drawEmpty x={:.2} y={:.2} w={:.2} h={:.2}",
            x,
            y,
            w,
            h
        );
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
        log::trace!(
            "BoundsTracker.drawEllipse x={:.2} y={:.2} w={:.2} h={:.2} shadow={:.2}",
            x,
            y,
            w,
            h,
            shadow
        );
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
        log::trace!(
            "BoundsTracker.drawPolygon minX={:.2} maxX={:.2} minY={:.2} maxY={:.2}",
            min_x,
            max_x,
            min_y,
            max_y
        );
        self.add_point(min_x - 10.0, min_y);
        self.add_point(max_x + 10.0, max_y);
    }

    /// Java LimitFinder.drawULine
    pub fn track_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        log::trace!(
            "BoundsTracker.drawLine ({:.2},{:.2})-({:.2},{:.2})",
            x1,
            y1,
            x2,
            y2
        );
        self.add_point(x1, y1);
        self.add_point(x2, y2);
    }

    /// Java LimitFinder.drawDotPath — path bounding box
    pub fn track_path_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        log::trace!(
            "BoundsTracker.drawDotPath min=({:.2},{:.2}) max=({:.2},{:.2})",
            min_x,
            min_y,
            max_x,
            max_y
        );
        self.add_point(min_x, min_y);
        self.add_point(max_x, max_y);
    }

    /// Java LimitFinder.drawText:
    ///   y_adj = y - h + 1.5
    ///   addPoint(x, y_adj), addPoint(x, y_adj+h), addPoint(x+w, y_adj), addPoint(x+w, y_adj+h)
    ///   i.e. (x, y-h+1.5) to (x+w, y+1.5)
    pub fn track_text(&mut self, x: f64, y: f64, text_width: f64, text_height: f64) {
        let y_adj = y - text_height + 1.5;
        log::trace!(
            "BoundsTracker.drawText x={:.4} y={:.4} w={:.4} h={:.4} y_adj={:.4}",
            x,
            y,
            text_width,
            text_height,
            y_adj
        );
        self.add_point(x, y_adj);
        self.add_point(x, y_adj + text_height);
        self.add_point(x + text_width, y_adj);
        self.add_point(x + text_width, y_adj + text_height);
    }

    /// Span: max - min in each dimension. Used with CANVAS_DELTA + DOC_MARGIN
    /// to compute final SVG dimensions matching Java's ensureVisible.
    pub fn span(&self) -> (f64, f64) {
        if self.max_x.is_finite() && self.min_x.is_finite() {
            (self.max_x - self.min_x, self.max_y - self.min_y)
        } else {
            (0.0, 0.0)
        }
    }

    pub fn min_point(&self) -> (f64, f64) {
        if self.max_x.is_finite() && self.min_x.is_finite() {
            (self.min_x, self.min_y)
        } else {
            (0.0, 0.0)
        }
    }

    pub fn max_point(&self) -> (f64, f64) {
        if self.max_x.is_finite() && self.min_x.is_finite() {
            (self.max_x, self.max_y)
        } else {
            (0.0, 0.0)
        }
    }
}

fn extract_svg_content(svg: &str) -> String {
    let mut body = svg;
    if body.starts_with("<?plantuml ") {
        if let Some(end) = body.find("?>") {
            body = &body[end + 2..];
        }
    }
    if let Some(tag_end) = body.find('>') {
        let after_open = &body[tag_end + 1..];
        if let Some(close_pos) = after_open.rfind("</svg>") {
            return after_open[..close_pos].to_string();
        }
        return after_open.to_string();
    }
    body.to_string()
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

    // Java wraps CSS content inside CDATA block.
    let defs_content = format!(
        "<style type=\"text/css\"><![CDATA[{}]]></style><script>{}</script>",
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

fn wrap_with_meta(
    body_svg: &str,
    meta: &DiagramMeta,
    diagram_type: &str,
    bg: &str,
    raw_body_dim: Option<(f64, f64)>,
    body_pre_offset: bool,
    skin: &crate::style::SkinParams,
) -> Result<String> {
    // SEQUENCE diagrams have a distinct chrome layout (SequenceDiagramArea) that
    // differs from the TextBlockExporter path used by other diagram types.
    // Java disables the annotation/chrome wrapping in SequenceDiagram.createImageBuilder
    // (annotations(false)) and instead lets SequenceDiagramFileMakerPuma2.createUDrawable
    // compose the chrome directly around the body.  Route to a dedicated function.
    if diagram_type == "SEQUENCE" {
        let has_meta = meta.title.is_some()
            || meta.header.is_some()
            || meta.footer.is_some()
            || meta.caption.is_some()
            || meta.legend.is_some();
        if has_meta {
            return wrap_with_meta_sequence(body_svg, meta, bg, raw_body_dim, skin);
        }
    }

    let (svg_w, svg_h) = extract_dimensions(body_svg);
    let body_content = extract_svg_content(body_svg);

    // Document-level margin: Java TextBlockExporter12026 applies diagram.getDefaultMargins().
    // Sequence diagrams (Puma2 classic): margin(top=5, right=5, bottom=5, left=0)
    // Sequence diagrams (Teoz):          margin(top=5, right=5, bottom=5, left=5)
    // CucaDiagram (class/component/etc): margin(top=0, right=5, bottom=5, left=0)
    // The body SVG viewport already bakes in these margins via the layout engine,
    // so we recover the raw textBlock dimensions by subtracting them.
    // Java ImageBuilder default margins per diagram type.
    // Activity (FTile): body viewport already includes internal padding from
    // compute_bounds (TOP_MARGIN + BOTTOM_MARGIN + 3) which absorbs the external
    // margin budget.  title_margin_top=10 shifts meta elements down.
    // Sequence (Puma2/Teoz): margin_top=5, body includes right margin.
    // CucaDiagram (class/component/etc): margin_top=0.
    // Java ImageBuilder default margins per diagram type.
    let doc_margin_top = match diagram_type {
        "SEQUENCE" => 5.0,
        "ACTIVITY" => 10.0,
        // Java TitledDiagram.getDefaultMargins() = 10 all sides for mindmap.
        "MINDMAP" => 10.0,
        _ => 0.0,
    };
    // Activity body viewport already includes right padding from compute_bounds,
    // so DOC_MARGIN_RIGHT must not be added again for the canvas width.
    let doc_margin_right = match diagram_type {
        "ACTIVITY" => 0.0,
        // Mindmap: Java uses 10+10=20 (margin_left + margin_right).
        "MINDMAP" => 20.0,
        _ => DOC_MARGIN_RIGHT,
    };
    let doc_margin_bottom = match diagram_type {
        // Mindmap: Java uses 10+10=20 total vertical margins.
        "MINDMAP" => 10.0,
        _ => DOC_MARGIN_BOTTOM,
    };

    // Use raw body dimensions if available (avoids integer truncation loss).
    // Otherwise fall back to extracting from SVG header (lossy).
    let (body_w, body_h) = if let Some((rw, rh)) = raw_body_dim {
        // Java SvekResult.calculateDimension() returns getDimension().delta(0, 12)
        // where getDimension() = (maxX - minX, maxY - minY) is the LimitFinder span.
        // raw_body_dim is the absolute max_point; the moveDelta(6,6) ensures minX=minY=6,
        // so span = (rw - 6, rh - 6) and calculateDimension = (rw - 6, rh - 6 + 12).
        // Apply span conversion only for CLASS (svek-based) diagrams with meta elements
        // that need centering; other diagram types use raw dimensions directly.
        let has_meta = meta.title.is_some()
            || meta.header.is_some()
            || meta.footer.is_some()
            || meta.caption.is_some()
            || meta.legend.is_some();
        let (svek_delta_w, svek_delta_h) = if diagram_type == "CLASS" && has_meta && rh > 0.0 {
            (-6.0, 6.0) // span: subtract minX=6; height: span - 6 + 12 = +6
        } else if diagram_type == "SEQUENCE" && has_meta && rh > 0.0 {
            // Java's LimitFinder tracks the participant tail bottom + extra border.
            // The layout formula undershoots by ~2.5px vs the actual drawn bounds.
            (0.0, 2.5)
        } else {
            (0.0, 0.0)
        };
        (rw + svek_delta_w, rh + svek_delta_h)
    } else {
        // Body SVG includes DOC_MARGIN + 1: recover raw textBlock dimensions.
        (
            svg_w - DOC_MARGIN_RIGHT - 1.0,
            svg_h - doc_margin_top - doc_margin_bottom - 1.0,
        )
    };
    log::trace!("wrap_with_meta: svg_w={svg_w} svg_h={svg_h} body_w={body_w} body_h={body_h} doc_margin_top={doc_margin_top}");

    // ── Resolve document section styles ──────────────────────────────
    let hdr_font_size = skin
        .get("document.header.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_HF_FONT_SIZE);
    let hdr_font_color = skin.get("document.header.fontcolor").map(|s| s.to_string());
    let hdr_bg_color = skin
        .get("document.header.backgroundcolor")
        .map(|s| s.to_string());

    let ftr_font_size = skin
        .get("document.footer.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_HF_FONT_SIZE);
    let ftr_font_color = skin.get("document.footer.fontcolor").map(|s| s.to_string());
    let ftr_bg_color = skin
        .get("document.footer.backgroundcolor")
        .map(|s| s.to_string());

    let title_font_size = skin
        .get("document.title.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_TITLE_FONT_SIZE);
    let title_font_color = skin.get("document.title.fontcolor").map(|s| s.to_string());
    let title_bg_color = skin
        .get("document.title.backgroundcolor")
        .map(|s| s.to_string());

    let leg_font_size = skin
        .get("document.legend.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_LEGEND_FONT_SIZE);
    let leg_font_color = skin.get("document.legend.fontcolor").map(|s| s.to_string());
    let leg_bg_color = skin
        .get("document.legend.backgroundcolor")
        .map(|s| s.to_string());

    let cap_font_size = skin
        .get("document.caption.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_CAPTION_FONT_SIZE);
    let cap_font_color = skin
        .get("document.caption.fontcolor")
        .map(|s| s.to_string());
    let cap_bg_color = skin
        .get("document.caption.backgroundcolor")
        .map(|s| s.to_string());

    let title_bold = title_font_size == META_TITLE_FONT_SIZE; // default title is bold

    // ── 1. Compute block dimensions for each meta element ───────────
    let hdr_text_w = meta
        .header
        .as_ref()
        .map(|t| creole_text_w(t, hdr_font_size, false))
        .unwrap_or(0.0);
    let hdr_text_h = if meta.header.is_some() {
        text_block_h(hdr_font_size, false)
    } else {
        0.0
    };
    let hdr_dim = if meta.header.is_some() {
        block_dim(hdr_text_w, hdr_text_h, 0.0, 0.0)
    } else {
        (0.0, 0.0)
    };

    let ftr_text_w = meta
        .footer
        .as_ref()
        .map(|t| creole_text_w(t, ftr_font_size, false))
        .unwrap_or(0.0);
    let ftr_text_h = if meta.footer.is_some() {
        text_block_h(ftr_font_size, false)
    } else {
        0.0
    };
    let ftr_dim = if meta.footer.is_some() {
        block_dim(ftr_text_w, ftr_text_h, 0.0, 0.0)
    } else {
        (0.0, 0.0)
    };

    let title_text_w = meta
        .title
        .as_ref()
        .map(|t| {
            // For tables, compute width from table layout (column-based) instead of raw text
            creole_table_width(t, title_font_size, title_bold)
                .unwrap_or_else(|| creole_text_w(t, title_font_size, title_bold))
        })
        .unwrap_or(0.0);
    let title_text_h = if let Some(ref t) = meta.title {
        let lh = font_metrics::line_height("SansSerif", title_font_size, title_bold, false);
        let n_lines = t.split(crate::NEWLINE_CHAR).flat_map(|s| s.lines()).count().max(1);
        let mut h = n_lines as f64 * lh;
        // Java: tables inside title use AtomWithMargin(table, 2, 2) which adds 4px.
        // Detect table content: any line starts with '|' or color prefix '<#...>|'.
        let has_table = t.split(crate::NEWLINE_CHAR)
            .flat_map(|s| s.lines())
            .any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('|')
                    || (trimmed.starts_with('<') && trimmed.contains(">|"))
            });
        if has_table {
            h += 4.0; // TABLE_MARGIN_Y * 2 (Java AtomWithMargin top+bottom)
        }
        h
    } else {
        0.0
    };
    let title_dim = if meta.title.is_some() {
        block_dim(title_text_w, title_text_h, TITLE_PADDING, TITLE_MARGIN)
    } else {
        (0.0, 0.0)
    };
    log::trace!("wrap_with_meta: title text_w={title_text_w:.10} text_h={title_text_h:.10} title_dim={title_dim:?}");

    let cap_text_w = meta
        .caption
        .as_ref()
        .map(|t| creole_text_w(t, cap_font_size, false))
        .unwrap_or(0.0);
    let cap_text_h = if meta.caption.is_some() {
        text_block_h(cap_font_size, false)
    } else {
        0.0
    };
    let cap_dim = if meta.caption.is_some() {
        block_dim(cap_text_w, cap_text_h, CAPTION_PADDING, CAPTION_MARGIN)
    } else {
        (0.0, 0.0)
    };

    let leg_text_w = meta
        .legend
        .as_ref()
        .map(|t| creole_text_w(t, leg_font_size, false))
        .unwrap_or(0.0);
    let leg_text_h = if let Some(ref leg) = meta.legend {
        crate::render::svg_richtext::compute_creole_note_text_height(leg, leg_font_size)
    } else {
        0.0
    };
    let leg_dim = if meta.legend.is_some() {
        block_dim(leg_text_w, leg_text_h, LEGEND_PADDING, LEGEND_MARGIN)
    } else {
        (0.0, 0.0)
    };

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
    // Java viewport = SvgGraphics.ensureVisible(getFinalDimension) where:
    //   getFinalDimension = lf_maxX + 1 + margins
    //   ensureVisible(x) = (int)(x + 1)
    // ensure_visible_int applies the ensureVisible +1.
    // CucaDiagram (class/object/component) and MINDMAP both expose raw
    // textBlock dimensions that mirror Java's MindMap.calculateDimension /
    // svek limitFinder span; the extra +1 from Java's getFinalDimension must
    // be added here so the canvas size grows by 1px while keeping caption
    // centering aligned to the unpadded body width.
    // Other diagram types (sequence, activity) bake the +1 into their layout
    // arithmetic already.
    let get_final_dim_extra = if matches!(diagram_type, "CLASS" | "MINDMAP") { 1.0 } else { 0.0 };
    let canvas_w = ensure_visible_int(tb_w + get_final_dim_extra + doc_margin_right) as f64;
    let canvas_h = ensure_visible_int(tb_h + get_final_dim_extra + doc_margin_top + doc_margin_bottom) as f64;
    log::trace!(
        "wrap_with_meta: tb_w={tb_w:.6} tb_h={tb_h:.6} canvas_w={canvas_w} canvas_h={canvas_h}"
    );
    log::trace!("wrap_with_meta: body_dim=({body_w},{body_h}) after_legend={after_legend:?} after_title={after_title:?} after_caption={after_caption:?}");

    // ── 3. Compute absolute drawing positions ──────────────────────
    let outer_inner_x = ((tb_w - after_caption.0) / 2.0).max(0.0);
    let cap_inner_x = ((after_caption.0 - after_title.0) / 2.0).max(0.0);
    let title_inner_x = ((after_title.0 - after_legend.0) / 2.0).max(0.0);
    let leg_inner_x = ((after_legend.0 - body_w) / 2.0).max(0.0);

    let body_abs_x = outer_inner_x + cap_inner_x + title_inner_x + leg_inner_x;
    let body_abs_y = hdr_dim.1 + title_dim.1;
    // Java TextBlockExporter12026 applies UTranslate(margin_left, margin_top) to the
    // whole textBlock. For sequence diagrams margin_top=5 shifts all meta elements down.
    // The body content already has this margin baked into its internal coordinates
    // (layout MARGIN=5), so only meta elements need the shift.
    let meta_dy = doc_margin_top;
    // Horizontal margin for meta elements. For mindmap/WBS, Java ImageBuilder
    // shifts everything by margin_left=10, including meta elements.
    let meta_dx = match diagram_type {
        "MINDMAP" => 10.0,
        _ => 0.0,
    };
    log::trace!(
        "body_pos: body_abs_x={body_abs_x:.6} body_abs_y={body_abs_y:.6} meta_dy={meta_dy}"
    );

    // ── 4. Render SVG ──────────────────────────────────────────────
    let mut buf = String::with_capacity(body_svg.len() + 2048);
    write_svg_root_bg(&mut buf, canvas_w, canvas_h, diagram_type, bg);
    // SVG document title (metadata, not the visible title block)
    // Java joins multi-line title Display with "\n" and XML-escapes it for the <title> element.
    // The raw text is used, preserving link/table markup — creole is NOT stripped.
    if let Some(ref t) = meta.title {
        if !t.is_empty() {
            write_svg_title(&mut buf, t);
        }
    }
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, canvas_w, canvas_h, bg);

    // Header (RIGHT-aligned)
    if let Some(ref hdr) = meta.header {
        let hdr_x = tb_w - hdr_dim.0;
        let text_y = meta_dy + font_metrics::ascent("SansSerif", hdr_font_size, false, false);
        let text_color = hdr_font_color.as_deref().unwrap_or(DIVIDER_COLOR);
        write!(buf, r#"<g class="header""#).unwrap();
        if let Some(sl) = meta.header_line {
            write!(buf, r#" data-source-line="{sl}""#).unwrap();
        }
        buf.push('>');
        if let Some(ref bg) = hdr_bg_color {
            write!(buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                bg, fmt_coord(hdr_text_h), fmt_coord(hdr_text_w), fmt_coord(hdr_x), fmt_coord(meta_dy)
            ).unwrap();
        }
        render_creole_text(
            &mut buf,
            hdr,
            hdr_x,
            text_y,
            text_block_h(hdr_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, hdr_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
        buf.push_str("</g>");
    }

    // Title (CENTER-aligned)
    if let Some(ref title) = meta.title {
        // Java centres the title using the bordered dimension directly.
        // No extra BORDERED_EXTRA needed: it would shift the title 0.5px left.
        let title_block_x =
            outer_inner_x + cap_inner_x + ((after_title.0 - title_dim.0) / 2.0).max(0.0);
        let text_x = title_block_x + TITLE_MARGIN + TITLE_PADDING;
        let text_y = meta_dy
            + hdr_dim.1
            + TITLE_MARGIN
            + TITLE_PADDING
            + font_metrics::ascent("SansSerif", title_font_size, title_bold, false);
        let text_color = title_font_color.as_deref().unwrap_or(TEXT_COLOR);
        write!(buf, r#"<g class="title""#).unwrap();
        if let Some(sl) = meta.title_line {
            write!(buf, r#" data-source-line="{sl}""#).unwrap();
        }
        buf.push('>');
        if let Some(ref bg) = title_bg_color {
            let rect_x = title_block_x + TITLE_MARGIN;
            let rect_y = meta_dy + hdr_dim.1 + TITLE_MARGIN;
            let rect_w = title_text_w + 2.0 * TITLE_PADDING;
            let rect_h = title_text_h + 2.0 * TITLE_PADDING;
            write!(buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                bg, fmt_coord(rect_h), fmt_coord(rect_w), fmt_coord(rect_x), fmt_coord(rect_y)
            ).unwrap();
        }
        let weight_str = if title_bold {
            r#" font-weight="bold""#
        } else {
            ""
        };
        let outer_attrs = format!(r#"font-size="{}"{}"#, title_font_size as i32, weight_str);
        // Detect table content: use block-level rendering which handles tables
        let title_lines: Vec<String> = title
            .split(crate::NEWLINE_CHAR)
            .flat_map(|s| s.lines())
            .map(|s| s.to_string())
            .collect();
        let has_table = creole_table_width(title, title_font_size, title_bold).is_some();
        if has_table {
            render_creole_display_lines(
                &mut buf,
                &title_lines,
                text_x,
                meta_dy + hdr_dim.1 + TITLE_MARGIN + TITLE_PADDING,
                text_color,
                &outer_attrs,
                false,
            );
        } else {
            render_creole_text(
                &mut buf,
                title,
                text_x,
                text_y,
                text_block_h(title_font_size, title_bold),
                text_color,
                None,
                &outer_attrs,
            );
        }
        if buf.ends_with('\n') {
            buf.pop();
        }
        buf.push_str("</g>");
    }

    // Body — Java renders body at absolute coordinates (no <g transform>).
    // Strip the <defs/><g>...</g> wrapper from body_content (already have top-level <defs/>)
    // and shift coordinates by (body_abs_x, body_abs_y).
    let body_inner = body_content
        .strip_prefix("<defs/><g>")
        .unwrap_or(&body_content);
    let body_inner = body_inner.strip_suffix("</g>").unwrap_or(body_inner);
    // Strip body-level background rect if present (wrap_with_meta provides its own).
    // Pattern: <rect fill="..." height="N" style="stroke:none;stroke-width:1;" width="N" x="0" y="0"/>
    let body_inner = if body_inner.starts_with("<rect fill=\"") {
        if let Some(end) = body_inner.find("/>") {
            let rect_tag = &body_inner[..end + 2];
            if rect_tag.contains("stroke:none")
                && rect_tag.contains("x=\"0\"")
                && rect_tag.contains("y=\"0\"")
            {
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
        if body_pre_offset || (body_abs_x.abs() < 0.001 && body_abs_y.abs() < 0.001) {
            // Body already has absolute coordinates (pre-offset applied by renderer).
            buf.push_str(body_inner);
        } else {
            let shifted = offset_svg_coords(body_inner, body_abs_x, body_abs_y);
            buf.push_str(&shifted);
        }
    }

    // Legend (CENTER-aligned)
    if let Some(ref leg) = meta.legend {
        let leg_wrapper_x = outer_inner_x + cap_inner_x + title_inner_x;
        let leg_wrapper_y = meta_dy + hdr_dim.1 + title_dim.1 + body_h;
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
        let has_style = leg_bg_color.is_some()
            || title_bg_color.is_some()
            || hdr_bg_color.is_some()
            || ftr_bg_color.is_some()
            || cap_bg_color.is_some();
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
        let text_y = rect_y
            + LEGEND_PADDING
            + font_metrics::ascent("SansSerif", leg_font_size, false, false);
        render_creole_text(
            &mut buf,
            leg,
            text_x,
            text_y,
            text_block_h(leg_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, leg_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
        buf.push_str("</g>");
    }

    // Caption (CENTER-aligned)
    if let Some(ref cap) = meta.caption {
        let cap_y_start = meta_dy + hdr_dim.1 + after_title.1;
        let cap_block_x = meta_dx + outer_inner_x + ((after_caption.0 - cap_dim.0) / 2.0).max(0.0);
        let text_x = cap_block_x + CAPTION_MARGIN + CAPTION_PADDING;
        let text_y = cap_y_start
            + CAPTION_MARGIN
            + CAPTION_PADDING
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
            &mut buf,
            cap,
            text_x,
            text_y,
            text_block_h(cap_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, cap_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
        buf.push_str("</g>");
    }

    // Footer (CENTER-aligned)
    if let Some(ref ftr) = meta.footer {
        let ftr_y_start = meta_dy + hdr_dim.1 + after_caption.1;
        let ftr_x = ((tb_w - ftr_dim.0) / 2.0).max(0.0);
        let text_y = ftr_y_start + font_metrics::ascent("SansSerif", ftr_font_size, false, false);
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
            &mut buf,
            ftr,
            ftr_x,
            text_y,
            text_block_h(ftr_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, ftr_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
        buf.push_str("</g>");
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

/// Sequence diagram chrome wrapping — mirrors Java's `SequenceDiagramFileMakerPuma2`
/// + `SequenceDiagramArea` composition.  Key differences from the generic
/// `wrap_with_meta` code path:
///
/// 1. Drawing order (top-to-bottom in SVG DOM): title → caption → body → header
///    → footer → legend.  Java draws them in exactly this order (see
///    `SequenceDiagramFileMakerPuma2.createUDrawable`, lines 214-233).
/// 2. Chrome elements (title/caption/header/footer/legend) are rendered as bare
///    `<rect>` + `<text>` without a surrounding `<g class="...">` wrapper.  Java
///    does NOT wrap annotation chrome for sequence because
///    `SequenceDiagram.createImageBuilder` disables `AnnotatedWorker` via
///    `annotations(false)` (SequenceDiagram.java:308-311).
/// 3. Canvas width model: `sequenceWidth = drawableSet.getDimension().width` which,
///    for Puma2, equals `lastParticipant.startX + headWidth + 2*outMargin`.
///    Rust's `svg.rs::BodyResult` builds `raw_body_dim = (sl.total_width - 5,
///    sl.total_height - 10)` from the layout (subtracting doc margins), and
///    `sl.total_width` already equals Java's `freeX`.  So `sequenceWidth =
///    raw_body_dim.0 + 5`.  Empirically `sequenceHeight = raw_body_dim.1 + 2`
///    (the 2px accounts for a layout-vs-render accounting difference).
/// 4. `area.getWidth() = max(sequenceWidth, headerWidth, titleWidth, footerWidth,
///    captionWidth)` — this is what Java's LimitFinder reads as `maxX` because any
///    `TextBlockMarged` inside chrome draws a `UEmpty` spanning its full (margined)
///    dimension.  Right-aligned header in particular ensures `lf.maxX >= area.getWidth()`.
/// 5. Final canvas width = `(int)(area.getWidth() + 1 + margin.left + margin.right + 1)`
///    = `(int)(area.getWidth() + 7)` for Puma2 (margin left=0, right=5) because
///    `getFinalDimension` adds +1 and `SvgGraphics.ensureVisible` adds another +1
///    via `(int)(x + 1)`.
///
/// Y positioning mirrors `SequenceDiagramArea` getters (lines 133-178):
/// ```text
/// headerY   = 0
/// titleY    = headerHeight + headerMargin
/// seqAreaY  = titleY + titleHeight           (+legendHeight if legend-top)
/// legendY   = sequenceHeight + headerHeight + titleHeight
/// captionY  = legendY + legendHeight
/// footerY   = captionY + captionHeight
/// ```
/// The ImageBuilder applies `UTranslate(margin.left, margin.top)` = `(0, 5)` to the
/// whole drawable, so every drawn element gets +5 on Y.  Each chrome element's own
/// margin is added on top (e.g. title margin 5, legend margin 12, caption margin 1).
#[allow(clippy::too_many_arguments)]
fn wrap_with_meta_sequence(
    body_svg: &str,
    meta: &DiagramMeta,
    bg: &str,
    raw_body_dim: Option<(f64, f64)>,
    skin: &crate::style::SkinParams,
) -> Result<String> {
    let body_content = extract_svg_content(body_svg);

    // ── Document margins (SequenceDiagram.getDefaultMargins for Puma2 mode) ──
    // Puma2: ClockwiseTopRightBottomLeft(top=5, right=5, bottom=5, left=0).
    let doc_margin_top = 5.0;
    let doc_margin_left = 0.0;
    let doc_margin_right = 5.0;
    let doc_margin_bottom = 5.0;

    // ── Resolve document section styles ─────────────────────────────
    let hdr_font_size = skin
        .get("document.header.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_HF_FONT_SIZE);
    let hdr_font_color = skin.get("document.header.fontcolor").map(|s| s.to_string());
    let hdr_bg_color = skin
        .get("document.header.backgroundcolor")
        .map(|s| s.to_string());

    let ftr_font_size = skin
        .get("document.footer.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_HF_FONT_SIZE);
    let ftr_font_color = skin.get("document.footer.fontcolor").map(|s| s.to_string());
    let ftr_bg_color = skin
        .get("document.footer.backgroundcolor")
        .map(|s| s.to_string());

    let title_font_size = skin
        .get("document.title.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_TITLE_FONT_SIZE);
    let title_font_color = skin.get("document.title.fontcolor").map(|s| s.to_string());
    let title_bg_color = skin
        .get("document.title.backgroundcolor")
        .map(|s| s.to_string());

    let leg_font_size = skin
        .get("document.legend.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_LEGEND_FONT_SIZE);
    let leg_font_color = skin.get("document.legend.fontcolor").map(|s| s.to_string());
    let leg_bg_color = skin
        .get("document.legend.backgroundcolor")
        .map(|s| s.to_string());

    let cap_font_size = skin
        .get("document.caption.fontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(META_CAPTION_FONT_SIZE);
    let cap_font_color = skin.get("document.caption.fontcolor").map(|s| s.to_string());
    let cap_bg_color = skin
        .get("document.caption.backgroundcolor")
        .map(|s| s.to_string());

    let title_bold = title_font_size == META_TITLE_FONT_SIZE;

    // ── Chrome block dimensions (matches Java TextBlockBordered+Marged) ──
    // For each block:
    //   bordered_dim = text + 2*padding + 1  (TextBlockBordered +1)
    //   full_dim     = bordered_dim + 2*margin (TextBlockMarged)
    let hdr_text_w = meta
        .header
        .as_ref()
        .map(|t| creole_text_w(t, hdr_font_size, false))
        .unwrap_or(0.0);
    let hdr_text_h = if meta.header.is_some() {
        text_block_h(hdr_font_size, false)
    } else {
        0.0
    };
    let hdr_dim = if meta.header.is_some() {
        block_dim(hdr_text_w, hdr_text_h, 0.0, 0.0)
    } else {
        (0.0, 0.0)
    };

    let ftr_text_w = meta
        .footer
        .as_ref()
        .map(|t| creole_text_w(t, ftr_font_size, false))
        .unwrap_or(0.0);
    let ftr_text_h = if meta.footer.is_some() {
        text_block_h(ftr_font_size, false)
    } else {
        0.0
    };
    let ftr_dim = if meta.footer.is_some() {
        block_dim(ftr_text_w, ftr_text_h, 0.0, 0.0)
    } else {
        (0.0, 0.0)
    };

    let title_text_w = meta
        .title
        .as_ref()
        .map(|t| {
            creole_table_width(t, title_font_size, title_bold)
                .unwrap_or_else(|| creole_text_w(t, title_font_size, title_bold))
        })
        .unwrap_or(0.0);
    let title_text_h = if let Some(ref t) = meta.title {
        let lh = font_metrics::line_height("SansSerif", title_font_size, title_bold, false);
        let n_lines = t.split(crate::NEWLINE_CHAR).flat_map(|s| s.lines()).count().max(1);
        let mut h = n_lines as f64 * lh;
        let has_table = t.split(crate::NEWLINE_CHAR).flat_map(|s| s.lines()).any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('|') || (trimmed.starts_with('<') && trimmed.contains(">|"))
        });
        if has_table {
            h += 4.0;
        }
        h
    } else {
        0.0
    };
    let title_dim = if meta.title.is_some() {
        block_dim(title_text_w, title_text_h, TITLE_PADDING, TITLE_MARGIN)
    } else {
        (0.0, 0.0)
    };

    let cap_text_w = meta
        .caption
        .as_ref()
        .map(|t| creole_text_w(t, cap_font_size, false))
        .unwrap_or(0.0);
    let cap_text_h = if meta.caption.is_some() {
        text_block_h(cap_font_size, false)
    } else {
        0.0
    };
    let cap_dim = if meta.caption.is_some() {
        block_dim(cap_text_w, cap_text_h, CAPTION_PADDING, CAPTION_MARGIN)
    } else {
        (0.0, 0.0)
    };

    let leg_text_w = meta
        .legend
        .as_ref()
        .map(|t| creole_text_w(t, leg_font_size, false))
        .unwrap_or(0.0);
    let leg_text_h = if let Some(ref leg) = meta.legend {
        crate::render::svg_richtext::compute_creole_note_text_height(leg, leg_font_size)
    } else {
        0.0
    };
    let leg_dim = if meta.legend.is_some() {
        block_dim(leg_text_w, leg_text_h, LEGEND_PADDING, LEGEND_MARGIN)
    } else {
        (0.0, 0.0)
    };

    // ── Body dimensions ──────────────────────────────────────────────
    // `raw_body_dim` for SEQUENCE is populated by `render::svg::body_result`
    // from `SeqLayout.total_{width,height}` minus doc margins:
    //   raw_w = sl.total_width  - DOC_MARGIN_RIGHT   (= total_width - 5)
    //   raw_h = sl.total_height - DOC_MARGIN_TOP - DOC_MARGIN_BOTTOM (= total_height - 10)
    // Empirically (verified by instrumenting Java's `SequenceDiagramFileMakerPuma2`
    // on `sequence/a0006.puml`), `sl.total_width == drawableSet.getDimension().width`
    // == Java's `freeX` (for Puma2 mode).  So:
    //   Java sequenceWidth  = raw_w + 5
    //   Java sequenceHeight = raw_h + 2   (2px layout-vs-render accounting delta)
    let (raw_w, raw_h) = raw_body_dim.unwrap_or((0.0, 0.0));
    let sequence_width = raw_w + 5.0;
    let sequence_height_java = raw_h + 2.0;

    // ── area.getWidth() = max(sequenceWidth, chrome widths) ─────────
    // Each chrome's dim.0 here is the post-margin width (what TextBlockMarged
    // reports via calculateDimension), matching Java's area width inputs.
    let area_width = sequence_width
        .max(hdr_dim.0)
        .max(title_dim.0)
        .max(ftr_dim.0)
        .max(cap_dim.0);

    // ── Y positions in SequenceDiagramArea coordinates (pre margin shift) ──
    // See SequenceDiagramArea.java:133-178.  Legend is assumed non-top
    // (isLegendTop == false) which matches default behaviour when legend has
    // no explicit vertical alignment.  TODO: handle top-aligned legend.
    let is_legend_top = false;
    let header_height = hdr_dim.1;
    let header_margin_internal = 0.0; // initHeader sets headerMargin=0
    let title_height = title_dim.1;
    let legend_height = leg_dim.1;
    let caption_height = cap_dim.1;
    let footer_height = ftr_dim.1;
    let footer_margin_internal = 0.0; // initFooter sets footerMargin=0
    let sequence_height = sequence_height_java;

    let title_y_area = header_height + header_margin_internal;
    let sequence_area_y = if is_legend_top {
        title_y_area + title_height + legend_height
    } else {
        title_y_area + title_height
    };
    let legend_y_area = if is_legend_top {
        title_height + header_height + header_margin_internal
    } else {
        sequence_height + header_height + header_margin_internal + title_height
    };
    let caption_y_area =
        sequence_height + header_height + header_margin_internal + title_height + legend_height;
    let footer_y_area = sequence_height
        + header_height
        + header_margin_internal
        + title_height
        + footer_margin_internal
        + caption_height
        + legend_height;

    // ── Canvas dimensions ────────────────────────────────────────────
    // Java: getFinalDimension = lf.maxX + 1 + margin.left + margin.right.
    // SvgGraphics.ensureVisible(dim) sets maxX = (int)(dim + 1).
    // lf.maxX = area.getWidth() when a header is present (it draws a UEmpty
    // spanning full area width); for other cases the max drawn element sets it.
    // We conservatively use area_width, which is >= any drawn extent.
    let body_end_y =
        sequence_height + header_height + header_margin_internal + title_height + legend_height
            + caption_height + footer_height + footer_margin_internal;
    let final_dim_w = area_width + 1.0 + doc_margin_left + doc_margin_right;
    let final_dim_h = body_end_y + 1.0 + doc_margin_top + doc_margin_bottom;
    let canvas_w = ensure_visible_int(final_dim_w) as f64;
    let canvas_h = ensure_visible_int(final_dim_h) as f64;

    log::trace!(
        "wrap_with_meta_sequence: sequence_width={sequence_width:.4} area_width={area_width:.4} \
        canvas_w={canvas_w} canvas_h={canvas_h} sequence_height={sequence_height}"
    );

    // ── Render SVG ───────────────────────────────────────────────────
    let mut buf = String::with_capacity(body_svg.len() + 2048);
    write_svg_root_bg(&mut buf, canvas_w, canvas_h, "SEQUENCE", bg);
    if let Some(ref t) = meta.title {
        if !t.is_empty() {
            write_svg_title(&mut buf, t);
        }
    }
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, canvas_w, canvas_h, bg);

    // ── Draw order (matches Java SequenceDiagramFileMakerPuma2.createUDrawable):
    //   1. title, 2. caption, 3. body, 4. header, 5. footer, 6. legend.
    // No <g class="..."> wrappers — Java emits raw rect+text for chrome when
    // annotations(false) is set in the ImageBuilder.

    // 1. Title (CENTER-aligned, drawn at area coords + (0, img_margin_top))
    if let Some(ref title) = meta.title {
        // area.getTitleX() = (getWidth() - titleWidth) / 2; then + title margin (5).
        let title_x_area = ((area_width - title_dim.0) / 2.0).max(0.0);
        let rect_x = doc_margin_left + title_x_area + TITLE_MARGIN;
        let rect_y = doc_margin_top + title_y_area + TITLE_MARGIN;
        let rect_w = title_text_w + 2.0 * TITLE_PADDING;
        let rect_h = title_text_h + 2.0 * TITLE_PADDING;
        let title_fill = title_bg_color.as_deref();
        if let Some(fill) = title_fill {
            write!(
                buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                fill,
                fmt_coord(rect_h),
                fmt_coord(rect_w),
                fmt_coord(rect_x),
                fmt_coord(rect_y),
            )
            .unwrap();
        }
        let text_x = rect_x + TITLE_PADDING;
        let text_y =
            rect_y + TITLE_PADDING + font_metrics::ascent("SansSerif", title_font_size, title_bold, false);
        let text_color = title_font_color.as_deref().unwrap_or(TEXT_COLOR);
        let weight_str = if title_bold { r#" font-weight="bold""# } else { "" };
        let outer_attrs = format!(r#"font-size="{}"{}"#, title_font_size as i32, weight_str);
        let title_lines: Vec<String> = title
            .split(crate::NEWLINE_CHAR)
            .flat_map(|s| s.lines())
            .map(|s| s.to_string())
            .collect();
        let has_table = creole_table_width(title, title_font_size, title_bold).is_some();
        if has_table {
            render_creole_display_lines(
                &mut buf,
                &title_lines,
                text_x,
                rect_y + TITLE_PADDING,
                text_color,
                &outer_attrs,
                false,
            );
        } else {
            render_creole_text(
                &mut buf,
                title,
                text_x,
                text_y,
                text_block_h(title_font_size, title_bold),
                text_color,
                None,
                &outer_attrs,
            );
        }
        if buf.ends_with('\n') {
            buf.pop();
        }
    }

    // 2. Caption (CENTER-aligned)
    if let Some(ref cap) = meta.caption {
        let cap_x_area = ((area_width - cap_dim.0) / 2.0).max(0.0);
        let rect_x = doc_margin_left + cap_x_area + CAPTION_MARGIN;
        let rect_y = doc_margin_top + caption_y_area + CAPTION_MARGIN;
        let rect_w = cap_text_w + 2.0 * CAPTION_PADDING;
        let rect_h = cap_text_h + 2.0 * CAPTION_PADDING;
        if let Some(ref fill) = cap_bg_color {
            write!(
                buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                fill,
                fmt_coord(rect_h),
                fmt_coord(rect_w),
                fmt_coord(rect_x),
                fmt_coord(rect_y),
            )
            .unwrap();
        }
        let text_x = rect_x + CAPTION_PADDING;
        let text_y =
            rect_y + CAPTION_PADDING + font_metrics::ascent("SansSerif", cap_font_size, false, false);
        let text_color = cap_font_color.as_deref().unwrap_or(TEXT_COLOR);
        render_creole_text(
            &mut buf,
            cap,
            text_x,
            text_y,
            text_block_h(cap_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, cap_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
    }

    // 3. Body (rendered by svg_sequence::render_sequence).  Java draws at
    //    (sequenceAreaX + delta1/2, sequenceAreaY).  For Puma2 with no wide
    //    legend, delta1=0 and sequenceAreaX = (area_width - sequenceWidth)/2.
    let body_inner = body_content
        .strip_prefix("<defs/><g>")
        .unwrap_or(&body_content);
    let body_inner = body_inner.strip_suffix("</g>").unwrap_or(body_inner);
    let body_inner = if body_inner.starts_with("<rect fill=\"") {
        if let Some(end) = body_inner.find("/>") {
            let rect_tag = &body_inner[..end + 2];
            if rect_tag.contains("stroke:none")
                && rect_tag.contains("x=\"0\"")
                && rect_tag.contains("y=\"0\"")
            {
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
    let delta1 = (leg_dim.0 - area_width).max(0.0);
    let sequence_area_x = ((area_width - sequence_width) / 2.0).max(0.0);
    // Rust's svg_sequence already bakes in a 5px top/left margin into the body
    // internal coordinates (layout MARGIN=5).  This coincidentally equals Java's
    // ImageBuilder top margin (=5) and left margin (=0).  So we DO NOT add
    // doc_margin_top here — the internal +5 already provides the image margin
    // shift.  For X, doc_margin_left=0 so it doesn't matter.
    let body_abs_x = doc_margin_left + sequence_area_x + delta1 / 2.0;
    let body_abs_y = sequence_area_y;
    if !body_inner.trim().is_empty() {
        if body_abs_x.abs() < 0.001 && body_abs_y.abs() < 0.001 {
            buf.push_str(body_inner);
        } else {
            let shifted = offset_svg_coords(body_inner, body_abs_x, body_abs_y);
            buf.push_str(&shifted);
        }
    }

    // 4. Header (RIGHT-aligned)
    if let Some(ref hdr) = meta.header {
        // area.getHeaderX(RIGHT) = getWidth() - headerWidth.  Header has no margin/pad.
        let hdr_x_area = area_width - hdr_dim.0;
        let rect_x = doc_margin_left + hdr_x_area;
        let rect_y = doc_margin_top + 0.0; // headerY = 0
        if let Some(ref fill) = hdr_bg_color {
            write!(
                buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                fill,
                fmt_coord(hdr_text_h),
                fmt_coord(hdr_text_w),
                fmt_coord(rect_x),
                fmt_coord(rect_y),
            )
            .unwrap();
        }
        let text_y = rect_y + font_metrics::ascent("SansSerif", hdr_font_size, false, false);
        let text_color = hdr_font_color.as_deref().unwrap_or(DIVIDER_COLOR);
        render_creole_text(
            &mut buf,
            hdr,
            rect_x,
            text_y,
            text_block_h(hdr_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, hdr_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
    }

    // 5. Footer (CENTER-aligned by default)
    if let Some(ref ftr) = meta.footer {
        let ftr_x_area = ((area_width - ftr_dim.0) / 2.0).max(0.0);
        let rect_x = doc_margin_left + ftr_x_area;
        let rect_y = doc_margin_top + footer_y_area;
        if let Some(ref fill) = ftr_bg_color {
            write!(
                buf,
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                fill,
                fmt_coord(ftr_text_h),
                fmt_coord(ftr_text_w),
                fmt_coord(rect_x),
                fmt_coord(rect_y),
            )
            .unwrap();
        }
        let text_y = rect_y + font_metrics::ascent("SansSerif", ftr_font_size, false, false);
        let text_color = ftr_font_color.as_deref().unwrap_or(DIVIDER_COLOR);
        render_creole_text(
            &mut buf,
            ftr,
            rect_x,
            text_y,
            text_block_h(ftr_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, ftr_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
    }

    // 6. Legend (CENTER-aligned by default; rounded rect + border)
    if let Some(ref leg) = meta.legend {
        let leg_x_area = ((area_width - leg_dim.0) / 2.0).max(0.0);
        let rect_x = doc_margin_left + leg_x_area + LEGEND_MARGIN;
        let rect_y = doc_margin_top + legend_y_area + LEGEND_MARGIN;
        let draw_w = leg_text_w + 2.0 * LEGEND_PADDING;
        let draw_h = leg_text_h + 2.0 * LEGEND_PADDING;
        let half_rc = LEGEND_ROUND_CORNER / 2.0;
        let legend_fill = leg_bg_color.as_deref().unwrap_or(LEGEND_BG);
        let text_color = leg_font_color.as_deref().unwrap_or(TEXT_COLOR);
        write!(
            buf,
            r#"<rect fill="{}" height="{}" rx="{}" ry="{}" style="stroke:{LEGEND_BORDER};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
            legend_fill,
            fmt_coord(draw_h),
            fmt_coord(half_rc),
            fmt_coord(half_rc),
            fmt_coord(draw_w),
            fmt_coord(rect_x),
            fmt_coord(rect_y),
        )
        .unwrap();
        let text_x = rect_x + LEGEND_PADDING;
        let text_y =
            rect_y + LEGEND_PADDING + font_metrics::ascent("SansSerif", leg_font_size, false, false);
        render_creole_text(
            &mut buf,
            leg,
            text_x,
            text_y,
            text_block_h(leg_font_size, false),
            text_color,
            None,
            &format!(r#"font-size="{}""#, leg_font_size as i32),
        );
        if buf.ends_with('\n') {
            buf.pop();
        }
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

    let re_x = RE_X.get_or_init(|| {
        Regex::new(r#"(?P<attr>(?:^| )(?:x|cx|x1|x2))="(?P<val>-?[\d.]+)""#).unwrap()
    });
    let re_y = RE_Y
        .get_or_init(|| Regex::new(r#"(?P<attr> (?:y|cy|y1|y2))="(?P<val>-?[\d.]+)""#).unwrap());
    let re_points = RE_POINTS.get_or_init(|| Regex::new(r#"points="([^"]*)""#).unwrap());
    let re_path_d = RE_PATH_D.get_or_init(|| Regex::new(r#" d="([^"]*)""#).unwrap());

    let mut result = svg.to_string();

    // Shift x-coordinate attributes
    result = re_x
        .replace_all(&result, |caps: &regex::Captures| {
            let attr = &caps["attr"];
            let val: f64 = caps["val"].parse().unwrap_or(0.0);
            format!("{}=\"{}\"", attr, fmt_coord(val + dx))
        })
        .to_string();

    // Shift y-coordinate attributes
    result = re_y
        .replace_all(&result, |caps: &regex::Captures| {
            let attr = &caps["attr"];
            let val: f64 = caps["val"].parse().unwrap_or(0.0);
            format!("{}=\"{}\"", attr, fmt_coord(val + dy))
        })
        .to_string();

    // Shift polygon points="x,y x,y ..."
    result = re_points
        .replace_all(&result, |caps: &regex::Captures| {
            let points = &caps[1];
            let shifted: Vec<String> = points
                .split(',')
                .collect::<Vec<_>>()
                .chunks(2)
                .filter_map(|pair| {
                    if pair.len() == 2 {
                        let x: f64 = pair[0].trim().parse().unwrap_or(0.0);
                        let y: f64 = pair[1].trim().parse().unwrap_or(0.0);
                        Some(format!("{},{}", fmt_coord(x + dx), fmt_coord(y + dy)))
                    } else {
                        None
                    }
                })
                .collect();
            format!("points=\"{}\"", shifted.join(","))
        })
        .to_string();

    // Shift path d="M x,y L x,y C x,y x,y x,y ..."
    result = re_path_d
        .replace_all(&result, |caps: &regex::Captures| {
            let d = &caps[1];
            let shifted = offset_path_data(d, dx, dy);
            format!(" d=\"{}\"", shifted)
        })
        .to_string();

    result
}

/// Offset all coordinates in an SVG path data string by (dx, dy).
///
/// SVG-path-command-aware: correctly handles arc commands (A/a) where
/// rx, ry, x-rotation, and flags must NOT be offset.
fn offset_path_data(d: &str, dx: f64, dy: f64) -> String {
    let mut result = String::with_capacity(d.len());
    let mut chars = d.chars().peekable();
    let mut cmd = ' ';

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace()) {
            result.push(chars.next().unwrap());
        }
        if chars.peek().is_none() {
            break;
        }

        let c = *chars.peek().unwrap();
        if c.is_alphabetic() {
            cmd = chars.next().unwrap();
            result.push(cmd);
            continue;
        }

        match cmd.to_ascii_uppercase() {
            'Z' => {
                // No parameters
                if let Some(ch) = chars.next() {
                    result.push(ch);
                }
            }
            'H' => {
                // Horizontal line: 1 x-value
                if let Some(x) = parse_path_number(&mut chars) {
                    result.push_str(&fmt_coord(x + dx));
                }
            }
            'V' => {
                // Vertical line: 1 y-value
                if let Some(y) = parse_path_number(&mut chars) {
                    result.push_str(&fmt_coord(y + dy));
                }
            }
            'A' => {
                // Arc: rx,ry x-rotation large-arc-flag sweep-flag x,y
                // rx,ry and rotation/flags are NOT offset
                if let Some(rx) = parse_path_number(&mut chars) {
                    result.push_str(&fmt_coord(rx)); // rx (no offset)
                    skip_path_sep(&mut chars, &mut result);
                    if let Some(ry) = parse_path_number(&mut chars) {
                        result.push_str(&fmt_coord(ry)); // ry (no offset)
                        skip_path_sep(&mut chars, &mut result);
                        if let Some(rot) = parse_path_number(&mut chars) {
                            result.push_str(&fmt_coord(rot)); // x-rotation (no offset)
                            skip_path_sep(&mut chars, &mut result);
                            if let Some(la) = parse_path_number(&mut chars) {
                                result.push_str(&fmt_coord(la)); // large-arc-flag (no offset)
                                skip_path_sep(&mut chars, &mut result);
                                if let Some(sw) = parse_path_number(&mut chars) {
                                    result.push_str(&fmt_coord(sw)); // sweep-flag (no offset)
                                    skip_path_sep(&mut chars, &mut result);
                                    if let Some(x) = parse_path_number(&mut chars) {
                                        result.push_str(&fmt_coord(x + dx)); // endpoint x
                                        skip_path_sep(&mut chars, &mut result);
                                        if let Some(y) = parse_path_number(&mut chars) {
                                            result.push_str(&fmt_coord(y + dy));
                                            // endpoint y
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            'C' => {
                // Cubic bezier: x1,y1 x2,y2 x,y (3 pairs)
                for _ in 0..3 {
                    if let Some(x) = parse_path_number(&mut chars) {
                        result.push_str(&fmt_coord(x + dx));
                        skip_path_sep(&mut chars, &mut result);
                        if let Some(y) = parse_path_number(&mut chars) {
                            result.push_str(&fmt_coord(y + dy));
                            skip_path_sep(&mut chars, &mut result);
                        }
                    }
                }
            }
            'S' | 'Q' => {
                // Smooth cubic / quadratic: 2 pairs
                for _ in 0..2 {
                    if let Some(x) = parse_path_number(&mut chars) {
                        result.push_str(&fmt_coord(x + dx));
                        skip_path_sep(&mut chars, &mut result);
                        if let Some(y) = parse_path_number(&mut chars) {
                            result.push_str(&fmt_coord(y + dy));
                            skip_path_sep(&mut chars, &mut result);
                        }
                    }
                }
            }
            _ => {
                // M, L, T and others: 1 coordinate pair
                if let Some(x) = parse_path_number(&mut chars) {
                    result.push_str(&fmt_coord(x + dx));
                    skip_path_sep(&mut chars, &mut result);
                    if let Some(y) = parse_path_number(&mut chars) {
                        result.push_str(&fmt_coord(y + dy));
                    }
                } else if let Some(ch) = chars.next() {
                    result.push(ch);
                }
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
    while chars
        .peek()
        .map_or(false, |c| c.is_ascii_digit() || *c == '.')
    {
        s.push(chars.next().unwrap());
    }
    if s.is_empty() || s == "-" {
        None
    } else {
        s.parse().ok()
    }
}

fn skip_path_sep(chars: &mut std::iter::Peekable<std::str::Chars>, result: &mut String) {
    while chars
        .peek()
        .map_or(false, |c| *c == ',' || c.is_whitespace())
    {
        result.push(chars.next().unwrap());
    }
}

// ── Class diagram rendering ─────────────────────────────────────────

fn render_class(
    cd: &crate::model::ClassDiagram,
    layout: &GraphLayout,
    skin: &SkinParams,
) -> Result<BodyResult> {
    let node_map: HashMap<&str, &NodeLayout> =
        layout.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    // Rust normalizes Svek coordinates back to the origin for rendering, but
    // Java renders at the post-Svek coordinates directly. `render_offset`
    // re-applies the exact per-axis delta needed to reconstruct the Java space.
    let edge_offset_x = layout.render_offset.0;
    // Java's LimitFinder sees generic boxes protruding above the owning entity
    // header. That only changes the global min_y when a generic entity is also
    // on the diagram's topmost entity row; lower generic entities do not affect
    // the final moveDelta-derived y origin.
    let min_entity_top = cd
        .entities
        .iter()
        .filter_map(|entity| {
            node_map
                .get(sanitize_id(&entity.name).as_str())
                .map(|node| node.cy - node.height / 2.0)
        })
        .fold(f64::INFINITY, f64::min);
    let generic_y_adjust = if cd
        .entities
        .iter()
        .filter(|entity| entity.generic.is_some())
        .filter_map(|entity| {
            node_map
                .get(sanitize_id(&entity.name).as_str())
                .map(|node| node.cy - node.height / 2.0)
        })
        .any(|top| (top - min_entity_top).abs() <= 0.001)
    {
        GENERIC_PROTRUSION
    } else {
        0.0
    };
    let edge_offset_y = layout.render_offset.1 + generic_y_adjust;
    let mut tracker = BoundsTracker::new();
    let mut sg = SvgGraphic::new(0, 1.0);
    let arrow_color = skin.arrow_color(LINK_COLOR);
    let group_meta: HashMap<&str, &crate::model::Group> = cd
        .groups
        .iter()
        .map(|group| (group.name.as_str(), group))
        .collect();

    // Build entity and group id map — IDs assigned by DEFINITION order (source_line),
    // interleaved between entities and groups. Java assigns entity UIDs at parse time.
    let mut entity_ids: HashMap<String, String> = HashMap::new();
    let mut group_ids: HashMap<String, String> = HashMap::new();

    // Collect all entities and groups with their source lines for interleaved ordering
    enum IdSlot<'a> {
        Entity(&'a Entity),
        Group(&'a ClusterLayout),
    }
    let mut all_slots: Vec<(usize, IdSlot)> = Vec::new();
    for entity in &cd.entities {
        all_slots.push((
            entity.source_line.unwrap_or(usize::MAX),
            IdSlot::Entity(entity),
        ));
    }
    for cluster in &layout.clusters {
        let source_line = group_meta
            .get(cluster.qualified_name.as_str())
            .and_then(|group| group.source_line)
            .unwrap_or(usize::MAX);
        all_slots.push((source_line, IdSlot::Group(cluster)));
    }
    all_slots.sort_by_key(|(sl, _)| *sl);

    let mut ent_counter = 2u32; // Java starts entity IDs at ent0002
    for (_, slot) in &all_slots {
        match slot {
            IdSlot::Entity(entity) => {
                let ent_id = entity
                    .uid
                    .clone()
                    .unwrap_or_else(|| format!("ent{:04}", ent_counter));
                entity_ids.insert(sanitize_id(&entity.name), ent_id);
            }
            IdSlot::Group(cluster) => {
                let ent_id = group_meta
                    .get(cluster.qualified_name.as_str())
                    .and_then(|group| group.uid.clone())
                    .unwrap_or_else(|| format!("ent{:04}", ent_counter));
                group_ids.insert(cluster.qualified_name.clone(), ent_id);
            }
        }
        ent_counter += 1;
    }

    // Build sorted group list for rendering
    let mut groups_by_def_order: Vec<&ClusterLayout> = layout.clusters.iter().collect();
    groups_by_def_order.sort_by_key(|cluster| {
        (
            group_meta
                .get(cluster.qualified_name.as_str())
                .and_then(|group| group.source_line)
                .unwrap_or(usize::MAX),
            cluster.qualified_name.matches('.').count(),
            cluster.qualified_name.clone(),
        )
    });

    // Java: object diagrams do NOT emit <!--class X--> comments for entities,
    // only class diagrams do.
    let is_object_diagram = cd.entities.iter().all(|e| matches!(e.kind, EntityKind::Object | EntityKind::Map));

    for cluster in &groups_by_def_order {
        let ent_id = group_ids
            .get(cluster.qualified_name.as_str())
            .map(|s| s.as_str())
            .unwrap_or("ent0000");
        let group = group_meta.get(cluster.qualified_name.as_str()).copied();
        draw_class_group(
            &mut sg,
            &mut tracker,
            cd,
            cluster,
            group,
            ent_id,
            skin,
            edge_offset_x,
            edge_offset_y,
        );
    }

    let mut entity_group_order: HashMap<&str, usize> = HashMap::new();
    let mut entity_qualified_names: HashMap<&str, String> = HashMap::new();
    for group in &cd.groups {
        let group_order = group.source_line.unwrap_or(usize::MAX);
        for entity_name in &group.entities {
            entity_group_order
                .entry(entity_name.as_str())
                .or_insert(group_order);
            entity_qualified_names
                .entry(entity_name.as_str())
                .or_insert_with(|| {
                    // If the entity name already starts with the group prefix
                    // (implicit package groups), don't prepend it again.
                    let prefix = format!("{}.", group.name);
                    if entity_name.starts_with(&prefix) {
                        entity_name.clone()
                    } else {
                        format!("{}.{}", group.name, entity_name)
                    }
                });
        }
    }
    // Build a definition-order index matching Java's entity creation order.
    // cd.entities is already sorted by sort_entities_by_order() in the parser,
    // which accounts for hide/show rules that implicitly reserve entity slots
    // before their explicit class declarations.
    let entity_def_order: HashMap<&str, usize> = cd
        .entities
        .iter()
        .enumerate()
        .map(|(i, e)| (e.name.as_str(), i))
        .collect();
    let mut entities_by_render_order: Vec<&Entity> = cd.entities.iter().collect();
    entities_by_render_order.sort_by_key(|entity| {
        (
            entity_group_order
                .get(entity.name.as_str())
                .copied()
                .unwrap_or(usize::MAX),
            entity_def_order
                .get(entity.name.as_str())
                .copied()
                .unwrap_or(usize::MAX),
        )
    });

    for entity in entities_by_render_order {
        let sid = sanitize_id(&entity.name);
        if let Some(nl) = node_map.get(sid.as_str()) {
            let ent_id = entity_ids
                .get(&sid)
                .map(|s| s.as_str())
                .unwrap_or("ent0000");
            let display_name = crate::layout::class_entity_display_name(&entity.name);
            if is_object_diagram {
                sg.push_raw(&format!(
                    "<g class=\"entity\" data-qualified-name=\"{}\"",
                    svg_group_metadata_attr(
                        entity_qualified_names
                            .get(entity.name.as_str())
                            .map(|s| s.as_str())
                            .unwrap_or(entity.name.as_str()),
                    ),
                ));
            } else {
                sg.push_raw(&format!(
                    "<!--{} {}--><g class=\"entity\" data-qualified-name=\"{}\"",
                    // Java uses "class" for class entities, "entity" for others (rectangle, component, etc.)
                    if matches!(entity.kind, EntityKind::Rectangle | EntityKind::Component) {
                        "entity"
                    } else {
                        "class"
                    },
                    svg_comment_escape(&display_name),
                    svg_group_metadata_attr(
                        entity_qualified_names
                            .get(entity.name.as_str())
                            .map(|s| s.as_str())
                            .unwrap_or(entity.name.as_str()),
                    ),
                ));
            }
            if let Some(source_line) = entity.source_line {
                sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
            }
            sg.push_raw(&format!(" id=\"{ent_id}\">"));
            draw_entity_box(
                &mut sg,
                &mut tracker,
                cd,
                entity,
                nl,
                skin,
                edge_offset_x,
                edge_offset_y,
            );
            sg.push_raw("</g>");
        }
    }

    let qualifier_placements =
        compute_qualifier_placements(cd, layout, edge_offset_x, edge_offset_y);
    let mut link_counter = ent_counter;
    for (link_idx, link) in cd.links.iter().enumerate() {
        let from_id = sanitize_id(&link.from);
        let to_id = sanitize_id(&link.to);
        if let Some(el) = layout.edges.get(link_idx).filter(|e| e.from == from_id && e.to == to_id)
        {
            let from_ent = entity_ids.get(&from_id).map(|s| s.as_str()).unwrap_or("");
            let to_ent = entity_ids.get(&to_id).map(|s| s.as_str()).unwrap_or("");
            let link_type = derive_link_type(link);
            let from_display = crate::layout::class_entity_display_name(&link.from);
            let to_display = crate::layout::class_entity_display_name(&link.to);
            let comment_prefix = if link_looks_reverted_for_svg(link) {
                "reverse link"
            } else {
                "link"
            };
            sg.push_raw(&format!(
                "<!--{} {} to {}--><g class=\"link\" data-entity-1=\"{}\" data-entity-2=\"{}\" data-link-type=\"{}\"",
                comment_prefix,
                svg_comment_escape(&from_display),
                svg_comment_escape(&to_display),
                from_ent,
                to_ent,
                link_type,
            ));
            if let Some(source_line) = link.source_line {
                sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
            }
            let link_id = link
                .uid
                .clone()
                .unwrap_or_else(|| format!("lnk{link_counter}"));
            sg.push_raw(&format!(" id=\"{link_id}\">"));
            draw_edge(
                &mut sg,
                &mut tracker,
                layout,
                link,
                el,
                link_idx,
                &qualifier_placements,
                skin,
                arrow_color,
                edge_offset_x,
                edge_offset_y,
            );
            sg.push_raw("</g>");
            link_counter += 1;
        }
    }

    // Notes — Java wraps each note in <g class="entity" data-qualified-name="GMN{i}">
    // Java note IDs start after all entities: entity count + 1 (0-indexed quark offset)
    // Java quark numbering: entities are numbered from 2 (0=root, 1=diagram), notes after that
    let note_id_base = cd.entities.len() + cd.links.len() + 2;
    for (ni, note) in layout.notes.iter().enumerate() {
        let note_qname = format!("GMN{}", note_id_base + ni);
        sg.push_raw(&format!(
            "<g class=\"entity\" data-qualified-name=\"{note_qname}\" id=\"ent{:04}\">",
            cd.entities.len() + ni
        ));
        draw_class_note(&mut sg, &mut tracker, note, edge_offset_x, edge_offset_y);
        sg.push_raw("</g>");
    }

    // Stable Java now sizes cuca/svek diagrams from ImageBuilder.getFinalDimension():
    // it runs LimitFinder on the already moveDelta-shifted drawing, then adds the
    // document margins. The rendered max point therefore is the authority, not
    // lf_span + delta(15,15).
    let is_degenerated = layout.nodes.len() <= 1 && layout.edges.is_empty() && layout.notes.is_empty();
    let (max_x, max_y) = tracker.max_point();
    // raw_body_dim: the LimitFinder extent (no +1). Used by wrap_with_meta for
    // merge_tb — the global getFinalDimension +1 is applied at the canvas level.
    let raw_body_dim = if is_degenerated {
        if let Some(node) = layout.nodes.first() {
            const DEGENERATED_DELTA: f64 = 7.0;
            Some((
                node.width + DEGENERATED_DELTA * 2.0,
                node.height + DEGENERATED_DELTA * 2.0,
            ))
        } else {
            // Empty diagram: body draws nothing, no LimitFinder extent.
            Some((0.0, 0.0))
        }
    } else if max_x.is_finite() && max_y.is_finite() {
        Some((max_x, max_y))
    } else {
        None
    };
    // For the standalone body SVG viewport, add the getFinalDimension +1.
    let (svg_w, svg_h) = if let Some((raw_w, raw_h)) = raw_body_dim {
        (
            ensure_visible_int(raw_w + 1.0 + DOC_MARGIN_RIGHT) as f64,
            ensure_visible_int(raw_h + 1.0 + DOC_MARGIN_BOTTOM) as f64,
        )
    } else {
        // Keep the empty-diagram fallback non-zero.
        (
            ensure_visible_int(DOC_MARGIN_RIGHT + 10.0) as f64,
            ensure_visible_int(DOC_MARGIN_BOTTOM + 10.0) as f64,
        )
    };

    let mut buf = String::with_capacity(sg.body().len() + 512);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, svg_w, svg_h, "CLASS", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(BodyResult {
        svg: buf,
        raw_body_dim,
        body_pre_offset: false,
    })
}

fn draw_class_group(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    cd: &crate::model::ClassDiagram,
    cluster: &ClusterLayout,
    group: Option<&crate::model::Group>,
    ent_id: &str,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if cluster.width <= 0.0 || cluster.height <= 0.0 {
        return;
    }
    let group_kind = group.map(|g| &g.kind).unwrap_or(&GroupKind::Package);
    let qname = &cluster.qualified_name;
    let title = cluster.title.as_deref().unwrap_or(qname);
    sg.push_raw(&format!(
        "<!--cluster {}--><g class=\"cluster\" data-qualified-name=\"{}\"",
        svg_comment_escape(title),
        svg_group_metadata_attr(qname),
    ));
    if let Some(source_line) = group.and_then(|g| g.source_line) {
        sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
    }
    sg.push_raw(&format!(" id=\"{ent_id}\">"));

    let x = cluster.x + edge_offset_x;
    let y = cluster.y + edge_offset_y;
    let w = cluster.width;
    let h = cluster.height;
    let group_header = group.map(|group| class_group_header_metrics(group, &cd.hide_show_rules));
    let visible_stereotypes = group_header
        .as_ref()
        .map(|metrics| metrics.visible_stereotypes.as_slice())
        .unwrap_or(&[]);
    let title_ascent = font_metrics::ascent("SansSerif", 14.0, true, false);
    let title_line_height = font_metrics::line_height("SansSerif", 14.0, true, false);
    let stereo_ascent = font_metrics::ascent("SansSerif", 14.0, false, true);
    let stereo_line_height = font_metrics::line_height("SansSerif", 14.0, false, true);

    match group_kind {
        GroupKind::Rectangle => {
            let border = skin.border_color("rectangle", "#181818");
            let font_color = skin.font_color("rectangle", "#000000");
            let fill = class_group_fill_color(cd, group).unwrap_or_else(|| "none".to_string());
            sg.set_fill_color(&fill);
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);
            tracker.track_rect(x, y, w, h);
            for (idx, label) in visible_stereotypes.iter().enumerate() {
                let stereo_text = format!("\u{00AB}{label}\u{00BB}");
                let stereo_w =
                    font_metrics::text_width(&stereo_text, "SansSerif", 14.0, false, true);
                let stereo_x = x + (w - stereo_w) / 2.0;
                let stereo_y = y + 2.0 + stereo_ascent + idx as f64 * stereo_line_height;
                sg.push_raw(&format!(
                    r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    fmt_coord(stereo_w),
                    fmt_coord(stereo_x),
                    fmt_coord(stereo_y),
                    xml_escape(&stereo_text),
                ));
                tracker.track_rect(
                    stereo_x,
                    stereo_y - stereo_ascent,
                    stereo_w,
                    stereo_line_height,
                );
            }
            let text_w = font_metrics::text_width(title, "SansSerif", 14.0, true, false);
            let text_x = x + (w - text_w) / 2.0;
            let text_y =
                y + 2.0 + visible_stereotypes.len() as f64 * stereo_line_height + title_ascent;
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(text_w),
                fmt_coord(text_x),
                fmt_coord(text_y),
                xml_escape(title),
            ));
            tracker.track_rect(
                text_x,
                text_y - HEADER_NAME_BASELINE,
                text_w,
                HEADER_NAME_BLOCK_HEIGHT,
            );
        }
        _ => {
            let border = skin.border_color("package", "#000000");
            let font_color = skin.font_color("package", "#000000");
            let text_w = font_metrics::text_width(title, "SansSerif", 14.0, true, false);
            let r = 2.5_f64;
            let tab_bottom = y + 22.2969;
            let tab_right = (x + w - r).min(x + text_w + 13.0);
            let tab_notch = (tab_right - 9.5).max(x + r);
            let tab_arc_end_x = (tab_right - 7.0).max(tab_notch);
            sg.push_raw(&format!(
                concat!(
                    r#"<path d="M{},{} L{},{}"#,
                    r#" A3.75,3.75 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}" fill="none" style="stroke:{};stroke-width:1.5;"/>"#
                ),
                fmt_coord(x + r),
                fmt_coord(y),
                fmt_coord(tab_notch),
                fmt_coord(y),
                fmt_coord(tab_arc_end_x),
                fmt_coord(y + r),
                fmt_coord(tab_right),
                fmt_coord(tab_bottom),
                fmt_coord(x + w - r),
                fmt_coord(tab_bottom),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x + w),
                fmt_coord(tab_bottom + r),
                fmt_coord(x + w),
                fmt_coord(y + h - r),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x + w - r),
                fmt_coord(y + h),
                fmt_coord(x + r),
                fmt_coord(y + h),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x),
                fmt_coord(y + h - r),
                fmt_coord(x),
                fmt_coord(y + r),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x + r),
                fmt_coord(y),
                border,
            ));
            sg.push_raw(&format!(
                r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(x),
                fmt_coord(tab_right),
                fmt_coord(tab_bottom),
                fmt_coord(tab_bottom),
            ));
            let text_x = x + 4.0;
            let text_y = y + 2.0 + title_ascent;
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(text_w),
                fmt_coord(text_x),
                fmt_coord(text_y),
                xml_escape(title),
            ));
            let title_h = if text_w == 0.0 {
                10.0
            } else {
                title_line_height + 6.0
            };
            for (idx, label) in visible_stereotypes.iter().enumerate() {
                let stereo_text = format!("\u{00AB}{label}\u{00BB}");
                let stereo_w =
                    font_metrics::text_width(&stereo_text, "SansSerif", 14.0, false, true);
                let stereo_x = x + 4.0 + (w - stereo_w) / 2.0;
                let stereo_y = y + 2.0 + title_h + stereo_ascent + idx as f64 * stereo_line_height;
                sg.push_raw(&format!(
                    r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    fmt_coord(stereo_w),
                    fmt_coord(stereo_x),
                    fmt_coord(stereo_y),
                    xml_escape(&stereo_text),
                ));
                tracker.track_rect(
                    stereo_x,
                    stereo_y - stereo_ascent,
                    stereo_w,
                    stereo_line_height,
                );
            }
            tracker.track_path_bounds(x, y, x + w, y + h);
            tracker.track_line(x, tab_bottom, tab_right, tab_bottom);
            tracker.track_rect(
                text_x,
                text_y - HEADER_NAME_BASELINE,
                text_w,
                HEADER_NAME_BLOCK_HEIGHT,
            );
        }
    }

    sg.push_raw("</g>");
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
/// If `spot_char` is Some, use that character's glyph instead of the entity-kind default.
fn emit_circle_glyph(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    kind: &EntityKind,
    circle_cx: f64,
    circle_cy: f64,
) {
    emit_circle_glyph_with_char(sg, tracker, kind, circle_cx, circle_cy, None);
}

fn emit_circle_glyph_with_char(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    kind: &EntityKind,
    circle_cx: f64,
    circle_cy: f64,
    spot_char: Option<char>,
) {
    let (glyph_raw, center) = if let Some(ch) = spot_char {
        match ch.to_ascii_uppercase() {
            'C' => (GLYPH_C_RAW, GLYPH_C_CENTER),
            'A' => (GLYPH_A_RAW, GLYPH_A_CENTER),
            'I' => (GLYPH_I_RAW, GLYPH_I_CENTER),
            'E' => (GLYPH_E_RAW, GLYPH_E_CENTER),
            _ => {
                // For characters we don't have pre-rendered glyphs for,
                // fall back to entity kind default
                match kind {
                    EntityKind::Class | EntityKind::Object => (GLYPH_C_RAW, GLYPH_C_CENTER),
                    EntityKind::Abstract => (GLYPH_A_RAW, GLYPH_A_CENTER),
                    EntityKind::Interface => (GLYPH_I_RAW, GLYPH_I_CENTER),
                    EntityKind::Enum => (GLYPH_E_RAW, GLYPH_E_CENTER),
                    EntityKind::Annotation | EntityKind::Rectangle | EntityKind::Component | EntityKind::Map => return,
                }
            }
        }
    } else {
        match kind {
            EntityKind::Class | EntityKind::Object => (GLYPH_C_RAW, GLYPH_C_CENTER),
            EntityKind::Abstract => (GLYPH_A_RAW, GLYPH_A_CENTER),
            EntityKind::Interface => (GLYPH_I_RAW, GLYPH_I_CENTER),
            EntityKind::Enum => (GLYPH_E_RAW, GLYPH_E_CENTER),
            EntityKind::Annotation | EntityKind::Rectangle | EntityKind::Component | EntityKind::Map => return,
        }
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
            if final_x < path_min_x {
                path_min_x = final_x;
            }
            if final_y < path_min_y {
                path_min_y = final_y;
            }
            if final_x > path_max_x {
                path_max_x = final_x;
            }
            if final_y > path_max_y {
                path_max_y = final_y;
            }
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
        EntityKind::Interface => "#B4A7E5",
        EntityKind::Enum => "#EB937F",
        EntityKind::Abstract => "#A9DCDF",
        EntityKind::Annotation => "#A9DCDF",
        EntityKind::Object | EntityKind::Map => "#ADD1B2",
        EntityKind::Rectangle => "#F1F1F1",
        EntityKind::Component => "#F1F1F1",
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
    if entity.kind == EntityKind::Object || entity.kind == EntityKind::Map {
        draw_object_box(sg, tracker, entity, nl, skin, edge_offset_x, edge_offset_y);
        return;
    }

    if entity.kind == EntityKind::Rectangle {
        draw_rectangle_entity_box(sg, tracker, entity, nl, skin, edge_offset_x, edge_offset_y);
        return;
    }

    // Java: after `layout_with_svek()` re-normalizes to origin, class entities
    // render back at the plain Svek margin offset (= 6).
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
        EntityKind::Component => (ENTITY_BG, BORDER_COLOR, "component"),
        EntityKind::Object | EntityKind::Map => unreachable!(),
    };
    let default_fill = skin.background_color(element_type, default_bg);
    let fill = entity
        .color
        .as_deref()
        .map(crate::style::normalize_color)
        .or_else(|| class_stereotype_fill_color(&cd.stereotype_backgrounds, &entity.stereotypes))
        .unwrap_or_else(|| default_fill.to_string());
    let stroke = skin.border_color(element_type, default_border);
    let font_color = skin.font_color(element_type, TEXT_COLOR);

    // Java URectangle.rounded(roundCorner): rx = roundCorner / 2.
    // Default roundCorner from style = 5 → rx = 2.5.
    // Java URectangle.rounded(roundCorner): SVG rx = roundCorner / 2.
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    // Rect with rx="2.5" ry="2.5" to match Java PlantUML
    sg.set_fill_color(&fill);
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, rx, rx, 0.0);
    tracker.track_rect(x, y, w, h);
    // Java entity image wrapper draws UEmpty(imageDim) at translate position,
    // which LimitFinder tracks with addPoint(x+w, y+h) — NO -1 adjustment.
    // This pushes max_y 1px beyond what the URectangle alone contributes.
    // Use image_width (not expanded DOT width) to match Java's calculateDimension.
    tracker.track_empty(x, y, nl.image_width, h);

    // Java font resolution:
    // - classFontSize controls the class name font size
    // - classAttributeFontSize controls member (field/method) font size
    // When classAttributeFontSize is set, it overrides classFontSize for both
    // header name and attributes (matching Java style priority).
    let explicit_attr_fs = skin
        .get("classattributefontsize")
        .and_then(|s| s.parse::<f64>().ok());
    let explicit_class_fs = skin
        .get("classfontsize")
        .and_then(|s| s.parse::<f64>().ok());
    let attr_font_size = explicit_attr_fs.unwrap_or_else(|| explicit_class_fs.unwrap_or(FONT_SIZE));
    let class_font_size =
        explicit_attr_fs.unwrap_or_else(|| explicit_class_fs.unwrap_or(FONT_SIZE));

    // Entity name WITHOUT generic parameter — generic is rendered separately in draw_generic_box
    // When `as Alias` is used, display_name holds the original quoted label.
    let name_display_raw = entity
        .display_name
        .as_deref()
        .map(String::from)
        .unwrap_or_else(|| crate::layout::class_entity_display_name(&entity.name));
    // Strip HTML markup tags (<b>, <i>, etc.) — Java interprets these as formatting.
    let markup_info = crate::layout::strip_html_markup(&name_display_raw);
    let name_display = if markup_info.bold || markup_info.italic {
        markup_info.text.clone()
    } else {
        name_display_raw
    };
    let name_markup_bold = markup_info.bold;
    let visible_stereotypes = visible_stereotype_labels(&cd.hide_show_rules, entity);
    let raw_field_count = entity.members.iter().filter(|m| !m.is_method).count();
    let raw_method_count = entity.members.iter().filter(|m| m.is_method).count();
    let show_fields = show_portion(&cd.hide_show_rules, ClassPortion::Field, &entity.name, raw_field_count);
    let show_methods = show_portion(&cd.hide_show_rules, ClassPortion::Method, &entity.name, raw_method_count);
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
    let has_kind_label = matches!(entity.kind, EntityKind::Enum | EntityKind::Annotation);
    let italic_name =
        markup_info.italic || matches!(entity.kind, EntityKind::Abstract | EntityKind::Interface);

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
            tracker.track_rect(
                cx,
                kind_y - kind_ascent,
                kind_tl_val,
                kind_ascent + kind_descent,
            );
        }
        let name_tl_val =
            font_metrics::text_width(&name_display, "SansSerif", class_font_size, true, false);
        {
            let name_tl = fmt_coord(name_tl_val);
            let name_escaped = xml_escape(&name_display);
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="{class_font_size:.0}" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{name_tl}" x="{}" y="{}">{name_escaped}</text>"#,
                fmt_coord(cx), fmt_coord(name_y),
            ));
        }
        {
            let name_ascent = font_metrics::ascent("SansSerif", class_font_size, true, false);
            let name_descent = font_metrics::descent("SansSerif", class_font_size, true, false);
            tracker.track_rect(
                cx,
                name_y - name_ascent,
                name_tl_val,
                name_ascent + name_descent,
            );
        }
    } else {
        let name_block = crate::layout::split_name_display(&name_display);
        let n_name_lines = name_block.lines.len();
        let name_line_metrics: Vec<(f64, f64)> = name_block
            .lines
            .iter()
            .map(|line| {
                crate::layout::display_line_metrics(
                    line,
                    class_font_size,
                    name_markup_bold,
                    italic_name,
                )
            })
            .collect();
        let name_width = name_line_metrics
            .iter()
            .map(|(visible_width, indent_width)| visible_width + indent_width)
            .fold(0.0_f64, f64::max);
        // Compute name block height and baseline dynamically from actual font size
        let name_ascent =
            font_metrics::ascent("SansSerif", class_font_size, name_markup_bold, italic_name);
        let name_descent =
            font_metrics::descent("SansSerif", class_font_size, name_markup_bold, italic_name);
        let single_line_height = name_ascent + name_descent;
        let name_block_height = n_name_lines as f64 * single_line_height;
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
        let stereo_block_width = stereo_widths.iter().copied().fold(0.0_f64, f64::max) + HEADER_STEREO_BLOCK_MARGIN;
        let width_stereo_and_name = name_block_width.max(stereo_block_width);
        let stereo_height = visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT;
        let header_height = HEADER_CIRCLE_BLOCK_HEIGHT
            .max(stereo_height + name_block_height + HEADER_STEREO_NAME_GAP);
        let vis_icon_w = if entity.visibility.is_some() {
            ENTITY_VIS_ICON_BLOCK_SIZE
        } else {
            0.0
        };
        let gen_dim_w = if let Some(ref g) = entity.generic {
            let text_w = font_metrics::text_width(g, "SansSerif", GENERIC_FONT_SIZE, false, true);
            text_w + 2.0 * GENERIC_INNER_MARGIN + 2.0 * GENERIC_OUTER_MARGIN
        } else {
            0.0
        };
        let supp_width =
            (w - HEADER_CIRCLE_BLOCK_WIDTH - vis_icon_w - width_stereo_and_name - gen_dim_w)
                .max(0.0);
        let h2 = (HEADER_CIRCLE_BLOCK_WIDTH / 4.0).min(supp_width * 0.1);
        let h1 = (supp_width - h2) / 2.0;

        let spot = extract_entity_spot(entity);
        let circle_color = if let Some(ref sp) = spot {
            if let Some(ref c) = sp.color {
                crate::style::normalize_color(c)
            } else {
                stereotype_circle_color(&entity.kind).to_string()
            }
        } else {
            stereotype_circle_color(&entity.kind).to_string()
        };
        let circle_block_x = x + h1;
        let ecx = circle_block_x + 15.0;
        let ecy = y + header_height / 2.0;
        sg.set_fill_color(&circle_color);
        sg.set_stroke_color(Some("#181818"));
        sg.set_stroke_width(1.0, None);
        sg.svg_ellipse(ecx, ecy, 11.0, 11.0, 0.0);
        tracker.track_ellipse(ecx, ecy, 11.0, 11.0);
        emit_circle_glyph_with_char(sg, tracker, &entity.kind, ecx, ecy, spot.as_ref().map(|s| s.character));

        let header_top_offset = (header_height - stereo_height - name_block_height) / 2.0;
        let name_block_x = x
            + HEADER_CIRCLE_BLOCK_WIDTH
            + vis_icon_w
            + (width_stereo_and_name - name_block_width) / 2.0
            + h1
            + h2;
        let name_inner_x = name_block_x + 3.0;

        if let Some(ref vis) = entity.visibility {
            let icon_x = name_inner_x - ENTITY_VIS_ICON_BLOCK_SIZE;
            // Java: EntityImageClassHeader wraps visibility UBlock with
            // withMargin(top=4), then mergeLR(uBlock, name, CENTER).
            // uBlock dim = (11, 11), with margin: (11, 15).
            // name dim height = HEADER_NAME_BLOCK_HEIGHT (≈16.3).
            // merged height = max(15, name_h).
            // icon in merged: (merged_h - 15) / 2 + 4 (margin top).
            // merged in header: (header_h - merged_h) / 2.
            let icon_margin_top = 0.0;
            let icon_block_h = ENTITY_VIS_ICON_BLOCK_SIZE + icon_margin_top;
            let merged_h = name_block_height.max(icon_block_h);
            let merged_y = (header_height - stereo_height - merged_h) / 2.0;
            let icon_in_merged = (merged_h - icon_block_h) / 2.0 + icon_margin_top;
            let icon_y = y + merged_y + icon_in_merged;
            draw_visibility_icon(sg, tracker, vis, true, icon_x, icon_y);
        }

        for (idx, label) in visible_stereotypes.iter().enumerate() {
            let stereo_text = format!("\u{00AB}{label}\u{00BB}");
            let stereo_x = x
                + HEADER_CIRCLE_BLOCK_WIDTH
                + vis_icon_w
                + (width_stereo_and_name - stereo_widths[idx]) / 2.0
                + h1
                + h2;
            let stereo_y = y
                + header_top_offset
                + HEADER_STEREO_BASELINE
                + idx as f64 * HEADER_STEREO_LINE_HEIGHT;
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="12" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(stereo_widths[idx]),
                fmt_coord(stereo_x),
                fmt_coord(stereo_y),
                xml_escape(&stereo_text),
            ));
            tracker.track_rect(
                stereo_x,
                stereo_y - HEADER_STEREO_BASELINE,
                stereo_widths[idx],
                HEADER_STEREO_LINE_HEIGHT,
            );
        }

        let font_style = if italic_name {
            Some("italic")
        } else {
            None
        };
        let font_weight = if name_markup_bold { Some("bold") } else { None };
        sg.set_fill_color(font_color);
        // Render each name line as a separate <text> element
        for (line_idx, line) in name_block.lines.iter().enumerate() {
            let display_line = if line.text.is_empty() {
                "\u{00A0}".to_string()
            } else {
                line.text.clone()
            };
            let line_y = y
                + header_top_offset
                + stereo_height
                + name_baseline
                + line_idx as f64 * single_line_height;
            let (visible_width, indent_width) = name_line_metrics[line_idx];
            let measured_width = visible_width + indent_width;
            let align_offset = match name_block.alignment {
                crate::layout::DisplayAlignment::Left => 0.0,
                crate::layout::DisplayAlignment::Center => (name_width - measured_width) / 2.0,
                crate::layout::DisplayAlignment::Right => name_width - measured_width,
            };
            let line_x = name_inner_x + align_offset + indent_width;
            sg.svg_text(
                &display_line,
                line_x,
                line_y,
                Some("sans-serif"),
                class_font_size,
                font_weight,
                font_style,
                None,
                visible_width,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
            tracker.track_rect(
                line_x,
                line_y - name_baseline,
                visible_width,
                single_line_height,
            );
        }
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
        let n_lines = crate::layout::split_name_display(&name_display).lines.len();
        let single_h = font_metrics::ascent("SansSerif", class_font_size, false, italic_name)
            + font_metrics::descent("SansSerif", class_font_size, false, italic_name);
        let dynamic_name_h = n_lines as f64 * single_h;
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
    // Java's LimitFinder first-pass sees separator lines at entity IMAGE width
    // (ULine(imageWidth,0) at entity translate). For non-expanded nodes where
    // imageWidth == nodeWidth, this adds max_x = x + imageWidth (1px beyond
    // the rect's x + nodeWidth - 1). For Graphviz-expanded nodes (qualifiers),
    // imageWidth < nodeWidth and the line doesn't extend beyond the rect.
    // Use image_width from the layout to match Java's LimitFinder.
    tracker.track_line(x, y, x + nl.image_width, y);
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
    let text_w =
        font_metrics::text_width(generic_text, "SansSerif", GENERIC_FONT_SIZE, false, true);
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

    let text_x = rect_x + GENERIC_INNER_MARGIN;
    let text_y = rect_y + GENERIC_INNER_MARGIN + GENERIC_BASELINE;
    sg.set_fill_color("#000000");
    sg.svg_text(
        generic_text,
        text_x,
        text_y,
        Some("sans-serif"),
        12.0,
        None,
        Some("italic"),
        None,
        text_w,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    tracker.track_rect(
        text_x,
        text_y - GENERIC_BASELINE,
        text_w,
        GENERIC_TEXT_HEIGHT,
    );
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

    match entity.rect_symbol {
        RectSymbol::File => {
            draw_file_shape(sg, x, y, w, h, rx, fill, stroke);
            // Java USymbolFile draws the outline as a UPath, so LimitFinder uses
            // the drawDotPath path — no -1 adjustment. Use track_path_bounds to
            // match the +1 extent versus track_rect.
            tracker.track_path_bounds(x, y, x + w, y + h);
        }
        _ => {
            sg.set_fill_color(fill);
            sg.set_stroke_color(Some(stroke));
            sg.set_stroke_width(0.5, None);
            sg.svg_rectangle(x, y, w, h, rx, rx, 0.0);
            tracker.track_rect(x, y, w, h);
        }
    }

    // Java: description text at font-size 14, left-aligned, padding 10px.
    // Use creole rendering to handle inline markup (<i>, <b>, etc.) and table syntax (|= ... |).
    // preserve_backslash_n=true: Java keeps literal \n as displayable text in bracket bodies.
    let text_x = x + 10.0;
    let top_y = y + 10.0;

    let mut tmp = String::new();
    render_creole_display_lines(
        &mut tmp,
        &entity.description,
        text_x,
        top_y,
        font_color,
        r#"font-size="14""#,
        true,
    );
    sg.push_raw(&tmp);
}

/// Render a `file` symbol outline — rectangle with a folded top-right corner.
/// Java: `USymbolFile.drawFile()` with rounded path (cornersize=10, r=roundCorner/2).
fn draw_file_shape(
    sg: &mut SvgGraphic,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    r: f64,
    fill: &str,
    stroke: &str,
) {
    const CORNERSIZE: f64 = 10.0;
    // Outer body path. Matches Java `drawFile(...)` with roundCorner != 0 branch:
    //   M 0, r
    //   L 0, h-r
    //   A r,r 0 0 0 r, h
    //   L w-r, h
    //   A r,r 0 0 0 w, h-r
    //   L w, cornersize
    //   L w-cornersize, 0
    //   L r, 0
    //   A r,r 0 0 0 0, r
    let d_outer = format!(
        "M{},{} L{},{} A{},{} 0 0 0 {},{} L{},{} A{},{} 0 0 0 {},{} L{},{} L{},{} L{},{} A{},{} 0 0 0 {},{}",
        fmt_coord(x),              fmt_coord(y + r),
        fmt_coord(x),              fmt_coord(y + h - r),
        fmt_coord(r),              fmt_coord(r),
        fmt_coord(x + r),          fmt_coord(y + h),
        fmt_coord(x + w - r),      fmt_coord(y + h),
        fmt_coord(r),              fmt_coord(r),
        fmt_coord(x + w),          fmt_coord(y + h - r),
        fmt_coord(x + w),          fmt_coord(y + CORNERSIZE),
        fmt_coord(x + w - CORNERSIZE), fmt_coord(y),
        fmt_coord(x + r),          fmt_coord(y),
        fmt_coord(r),              fmt_coord(r),
        fmt_coord(x),              fmt_coord(y + r),
    );
    sg.push_raw(&format!(
        r#"<path d="{d_outer}" fill="{fill}" style="stroke:{stroke};stroke-width:0.5;"/>"#,
    ));

    // Fold dog-ear decoration path (the small cut that visually separates the
    // corner from the main body). Java uses the same fill as body, so the
    // triangle is painted over the upper-right cut-off area.
    //   M w-cornersize, 0
    //   L w-cornersize, cornersize - r
    //   A r,r 0 0 0 w-cornersize+r, cornersize
    //   L w, cornersize
    let d_fold = format!(
        "M{},{} L{},{} A{},{} 0 0 0 {},{} L{},{}",
        fmt_coord(x + w - CORNERSIZE), fmt_coord(y),
        fmt_coord(x + w - CORNERSIZE), fmt_coord(y + CORNERSIZE - r),
        fmt_coord(r),                  fmt_coord(r),
        fmt_coord(x + w - CORNERSIZE + r), fmt_coord(y + CORNERSIZE),
        fmt_coord(x + w),              fmt_coord(y + CORNERSIZE),
    );
    sg.push_raw(&format!(
        r#"<path d="{d_fold}" fill="{fill}" style="stroke:{stroke};stroke-width:0.5;"/>"#,
    ));
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
    // Use display_name (from `as Alias` syntax) — it includes the "Map" keyword for map entities.
    let nd = entity
        .display_name
        .as_deref()
        .unwrap_or(&entity.name);
    let has_creole = nd.contains("**") || nd.contains("//");
    let name_width = if has_creole {
        crate::render::svg_richtext::measure_creole_display_lines(
            &[nd.to_string()],
            "SansSerif",
            class_font_size,
            false,
            false,
            false,
        )
        .0
    } else {
        font_metrics::text_width(nd, "SansSerif", class_font_size, false, false)
    };
    let name_block_width = name_width + 2.0 * OBJ_NAME_MARGIN;
    let name_block_height = HEADER_NAME_BLOCK_HEIGHT + 2.0 * OBJ_NAME_MARGIN;

    // PlacementStrategyY1Y2 with 1 element: x = (totalWidth - blockWidth) / 2
    // height = titleHeight = name_block_height, so space = 0, y = 0
    let name_offset_x = (w - name_block_width) / 2.0;
    let text_x = x + name_offset_x + OBJ_NAME_MARGIN;
    let text_y = y + OBJ_NAME_MARGIN + HEADER_NAME_BASELINE;

    if has_creole {
        // Render with creole richtext support (handles **bold**, //italic//, etc.)
        let outer_attrs = format!(r#"font-size="{}""#, class_font_size as i32);
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            nd,
            text_x,
            text_y,
            text_block_h(class_font_size, false),
            font_color,
            None,
            &outer_attrs,
        );
        sg.push_raw(&tmp);
    } else {
        sg.set_fill_color(font_color);
        sg.svg_text(
            nd,
            text_x,
            text_y,
            Some("sans-serif"),
            class_font_size,
            None,
            None,
            None,
            name_width,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
    tracker.track_rect(
        text_x,
        text_y - HEADER_NAME_BASELINE,
        name_width,
        HEADER_NAME_BLOCK_HEIGHT,
    );

    // Separator line at y + titleHeight
    let title_height = name_block_height;
    let sep_y = y + title_height;
    let x1 = x + 1.0;
    let x2 = x + w - 1.0;

    // Map entities: render key => value table body
    // Java EntityImageMap: each row uses withMargin(text, 2, 2), adding 4px vertical padding.
    if entity.kind == EntityKind::Map && !entity.map_entries.is_empty() {
        let attr_font_size = skin.font_size("classattribute", class_font_size);
        let text_line_h = font_metrics::line_height("SansSerif", attr_font_size, false, false);
        let row_margin_top = 2.0; // withMargin(2,2): top inset before text baseline
        let row_margin = 4.0; // withMargin(2,2) → 2 top + 2 bottom
        let row_h = text_line_h + row_margin;
        let ascent = font_metrics::ascent("SansSerif", attr_font_size, false, false);
        // Java TextBlockMap: withMargin(result, 5, 2) → 5px left + 5px right per column
        let cell_margin_lr = 5.0;
        let col_a_width: f64 = entity.map_entries.iter()
            .map(|(key, _)| font_metrics::text_width(key, "SansSerif", attr_font_size, false, false) + 2.0 * cell_margin_lr)
            .fold(0.0_f64, f64::max);
        let mut cur_y = sep_y;
        for (key, value) in &entity.map_entries {
            sg.set_stroke_color(Some(stroke_color));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(x, cur_y, x + w, cur_y, 0.0);
            tracker.track_line(x, cur_y, x + w, cur_y);
            let key_w = font_metrics::text_width(key, "SansSerif", attr_font_size, false, false);
            let text_y_row = cur_y + row_margin_top + ascent;
            sg.set_fill_color(font_color);
            sg.svg_text(key, x + cell_margin_lr, text_y_row, Some("sans-serif"), attr_font_size, None, None, None, key_w, LengthAdjust::Spacing, None, 0, None);
            let val_w = font_metrics::text_width(value, "SansSerif", attr_font_size, false, false);
            sg.svg_text(value, x + col_a_width + cell_margin_lr, text_y_row, Some("sans-serif"), attr_font_size, None, None, None, val_w, LengthAdjust::Spacing, None, 0, None);
            sg.set_stroke_color(Some(stroke_color));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(x + col_a_width, cur_y, x + col_a_width, cur_y + row_h, 0.0);
            tracker.track_line(x + col_a_width, cur_y, x + col_a_width, cur_y + row_h);
            cur_y += row_h;
        }
    } else {
        // Render object fields in the body section
        let visible_fields: Vec<&Member> = entity.members.iter().filter(|m| !m.is_method).collect();
        if !visible_fields.is_empty() {
            // draw_member_section draws its own separator at section_y
            let attr_font_size = skin.font_size("classattribute", class_font_size);
            let x1_val = fmt_coord(x1);
            let x2_val = fmt_coord(x2);
            draw_member_section(
                sg, tracker, &visible_fields, sep_y, x, &x1_val, &x2_val, font_color, attr_font_size, stroke_color,
            );
        } else {
            // No fields: draw the separator line explicitly
            sg.set_stroke_color(Some(stroke_color));
            sg.set_stroke_width(0.5, None);
            sg.svg_line(x1, sep_y, x2, sep_y, 0.0);
            tracker.track_line(x1, sep_y, x2, sep_y);
        }
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
    let (section_w, section_h) = member_section_block_dimensions(members, attr_font_size);
    tracker.track_empty(x, section_y, section_w, section_h);

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

        let base_text_x = x + if member.visibility.is_some() {
            MEMBER_TEXT_X_WITH_ICON
        } else {
            MEMBER_TEXT_X_NO_ICON
        };

        for (line_idx, (line_text, indent)) in lines.iter().enumerate() {
            let text_y = section_y + text_y_offset + (visual_row + line_idx) as f64 * row_h;
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
                line_text,
                text_x,
                text_y,
                Some("sans-serif"),
                attr_font_size,
                None,
                font_style_attr,
                text_deco_attr,
                text_width_val,
                LengthAdjust::Spacing,
                None,
                0,
                None,
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

fn member_section_block_dimensions(members: &[&Member], attr_font_size: f64) -> (f64, f64) {
    if members.is_empty() {
        return (12.0, EMPTY_COMPARTMENT);
    }

    // Java MethodsOrFieldsArea wraps the member content block with
    // TextBlockUtils.withMargin(..., 6, 4), which contributes a UEmpty
    // wrapper to LimitFinder even when inner text/icon primitives are tracked
    // separately.
    let has_small_icon = members.iter().any(|m| m.visibility.is_some());
    let icon_col_w = if has_small_icon { 14.0 } else { 0.0 };
    let text_w = members
        .iter()
        .map(|member| {
            let text = member_text(member);
            split_member_lines(&text)
                .iter()
                .enumerate()
                .map(|(idx, (line_text, indent))| {
                    let line_w = font_metrics::text_width(
                        line_text,
                        "SansSerif",
                        attr_font_size,
                        false,
                        member.modifiers.is_abstract,
                    );
                    if idx == 0 { line_w } else { indent + line_w }
                })
                .fold(0.0_f64, f64::max)
        })
        .fold(0.0_f64, f64::max);
    let content_w = icon_col_w + text_w;
    let content_h = section_height_with_fs(members, attr_font_size) - 8.0;
    (content_w + 12.0, content_h + 8.0)
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
        one_row_h + (total_visual_lines.saturating_sub(1)) as f64 * row_h
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
            sg.svg_polygon(
                0.0,
                &[
                    poly_pts[0].0,
                    poly_pts[0].1,
                    poly_pts[1].0,
                    poly_pts[1].1,
                    poly_pts[2].0,
                    poly_pts[2].1,
                    poly_pts[3].0,
                    poly_pts[3].1,
                ],
            );
            tracker.track_polygon(&poly_pts);
        }
        Visibility::Package => {
            // VisibilityModifier.drawTriangle: size -= 2 (10→8), translate(x+1,y+0)
            // Points: (size/2,1),(0,size-1),(size,size-1) where size=8
            let ox = x + 1.0;
            let oy = y;
            let fill = if is_method { "#4177AF" } else { "none" };
            let poly_pts = [
                (ox + 4.0, oy + 1.0), // (size/2=4, 1)
                (ox, oy + 7.0),       // (0, size-1=7)
                (ox + 8.0, oy + 7.0), // (size=8, size-1=7)
            ];
            sg.set_fill_color(fill);
            sg.set_stroke_color(Some("#1963A0"));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(
                0.0,
                &[
                    poly_pts[0].0,
                    poly_pts[0].1,
                    poly_pts[1].0,
                    poly_pts[1].1,
                    poly_pts[2].0,
                    poly_pts[2].1,
                ],
            );
            tracker.track_polygon(&poly_pts);
        }
    }
    sg.push_raw("</g>");
}

fn show_portion(
    rules: &[ClassHideShowRule],
    portion: ClassPortion,
    entity_name: &str,
    member_count: usize,
) -> bool {
    let mut result = true;
    for rule in rules {
        if rule.portion != portion {
            continue;
        }
        if rule.empty_only && member_count > 0 {
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
        .map(|st| {
            // Extract spot notation and return cleaned label
            let (_, cleaned) = st.extract_spot();
            cleaned
        })
        .filter(|label| !label.is_empty() && stereotype_label_visible(rules, label))
        .collect()
}

/// Extract spot info from entity stereotypes.
/// Returns the first spot found (character + resolved color), if any.
fn extract_entity_spot(entity: &Entity) -> Option<crate::model::entity::StereotypeSpot> {
    for st in &entity.stereotypes {
        let (spot, _) = st.extract_spot();
        if let Some(s) = spot {
            return Some(s);
        }
    }
    None
}

fn class_group_fill_color(
    cd: &ClassDiagram,
    group: Option<&crate::model::Group>,
) -> Option<String> {
    let group = group?;
    group
        .color
        .as_deref()
        .map(crate::style::normalize_color)
        .or_else(|| class_stereotype_fill_color(&cd.stereotype_backgrounds, &group.stereotypes))
}

fn class_stereotype_fill_color(
    stereotype_backgrounds: &HashMap<String, String>,
    stereotypes: &[crate::model::Stereotype],
) -> Option<String> {
    stereotypes
        .iter()
        .filter_map(|stereotype| stereotype_backgrounds.get(&stereotype.0))
        .map(|color| crate::style::normalize_color(color))
        .last()
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
    let left = &link.left_head;
    let right = &link.right_head;
    if matches!(left, ArrowHead::Diamond) || matches!(right, ArrowHead::Diamond) {
        "composition"
    } else if matches!(left, ArrowHead::DiamondHollow) || matches!(right, ArrowHead::DiamondHollow)
    {
        "aggregation"
    } else if matches!(left, ArrowHead::Triangle) || matches!(right, ArrowHead::Triangle) {
        "extension"
    } else if matches!(left, ArrowHead::Arrow) || matches!(right, ArrowHead::Arrow) {
        "dependency"
    } else if matches!(left, ArrowHead::Plus) || matches!(right, ArrowHead::Plus) {
        "innerclass"
    } else {
        "association"
    }
}

fn edge_label_margin(link: &Link) -> f64 {
    if link.from == link.to {
        6.0
    } else {
        1.0
    }
}

/// Parse the start and end points from an SVG path d-string.
/// Returns ((start_x, start_y), (end_x, end_y)) or None.
fn parse_path_start_end(d: &str) -> Option<((f64, f64), (f64, f64))> {
    // Start: first M command
    let d = d.trim();
    if !d.starts_with('M') {
        return None;
    }
    let rest = &d[1..];
    // Parse start coordinates: "x,y" or "x y"
    let mut chars = rest.chars().peekable();
    let sx_str: String = chars
        .by_ref()
        .take_while(|c| *c != ',' && *c != ' ')
        .collect();
    let sy_str: String = chars
        .by_ref()
        .take_while(|c| *c != ' ' && *c != 'C' && *c != 'L' && *c != 'c' && *c != 'l')
        .collect();
    let sx = sx_str.parse::<f64>().ok()?;
    let sy = sy_str.parse::<f64>().ok()?;

    // End: last numeric pair in the path
    // Find all numbers at the end of the path
    let bytes = d.as_bytes();
    let mut end = d.len();
    // Skip trailing whitespace
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    // Walk backwards to find the last y coordinate
    let mut ey_end = end;
    while ey_end > 0
        && (bytes[ey_end - 1].is_ascii_digit()
            || bytes[ey_end - 1] == b'.'
            || bytes[ey_end - 1] == b'-')
    {
        ey_end -= 1;
    }
    let ey_str = &d[ey_end..end];
    // Skip separator (comma or space)
    let mut ex_end = ey_end;
    if ex_end > 0 && (bytes[ex_end - 1] == b',' || bytes[ex_end - 1] == b' ') {
        ex_end -= 1;
    }
    // Walk backwards to find the last x coordinate
    let mut ex_start = ex_end;
    while ex_start > 0
        && (bytes[ex_start - 1].is_ascii_digit()
            || bytes[ex_start - 1] == b'.'
            || bytes[ex_start - 1] == b'-')
    {
        ex_start -= 1;
    }
    let ex_str = &d[ex_start..ex_end];
    let ex = ex_str.parse::<f64>().ok()?;
    let ey = ey_str.parse::<f64>().ok()?;

    Some(((sx, sy), (ex, ey)))
}

/// Draw a TextBlockArrow2 polygon for a link label arrow indicator.
/// Java: `TextBlockArrow2.drawU()` renders a small triangle whose direction
/// is determined by the edge path angle, font size, and link arrow direction.
///
/// `origin_x`, `origin_y` is the top-left of the arrow text block (13×13),
/// already inside the TextBlockMarged margin.
fn draw_label_arrow_polygon(
    sg: &mut SvgGraphic,
    origin_x: f64,
    origin_y: f64,
    angle: f64,
    font_size: f64,
) {
    let tri_size = (font_size * 0.80) as i32;
    let tri_size_f = tri_size as f64;
    let cx = origin_x + tri_size_f / 2.0;
    let cy = origin_y + font_size / 2.0;
    let radius = tri_size_f / 2.0;
    let beta = std::f64::consts::PI * 4.0 / 5.0;

    let p0x = cx + radius * angle.sin();
    let p0y = cy + radius * angle.cos();
    let p1x = cx + radius * (angle + beta).sin();
    let p1y = cy + radius * (angle + beta).cos();
    let p2x = cx + radius * (angle - beta).sin();
    let p2y = cy + radius * (angle - beta).cos();

    let points_str = format!(
        "{},{},{},{},{},{},{},{}",
        crate::klimt::svg::fmt_coord(p0x),
        crate::klimt::svg::fmt_coord(p0y),
        crate::klimt::svg::fmt_coord(p1x),
        crate::klimt::svg::fmt_coord(p1y),
        crate::klimt::svg::fmt_coord(p2x),
        crate::klimt::svg::fmt_coord(p2y),
        crate::klimt::svg::fmt_coord(p0x),
        crate::klimt::svg::fmt_coord(p0y),
    );
    sg.push_raw(&format!(
        "<polygon fill=\"#000000\" points=\"{}\" style=\"stroke:#000000;stroke-width:1;\"/>",
        points_str,
    ));
}

fn draw_edge(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    layout: &GraphLayout,
    link: &Link,
    el: &EdgeLayout,
    link_idx: usize,
    qualifier_placements: &HashMap<QualifierKey, KalPlacement>,
    skin: &SkinParams,
    link_color: &str,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if el.points.is_empty() {
        return;
    }

    let mut decor_points = el.points.clone();
    if let Some(placement) = qualifier_placements.get(&QualifierKey {
        link_idx,
        endpoint: QualifierEndpoint::Tail,
    }) {
        if let Some((dx, dy)) =
            qualifier_edge_translation(link, QualifierEndpoint::Tail, placement)
        {
            move_edge_start_point(&mut decor_points, dx, dy);
        }
    }
    if let Some(placement) = qualifier_placements.get(&QualifierKey {
        link_idx,
        endpoint: QualifierEndpoint::Head,
    }) {
        if let Some((dx, dy)) =
            qualifier_edge_translation(link, QualifierEndpoint::Head, placement)
        {
            move_edge_end_point(&mut decor_points, dx, dy);
        }
    }

    let mut path_points = decor_points.clone();
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
            if ax < p_min_x {
                p_min_x = ax;
            }
            if ay < p_min_y {
                p_min_y = ay;
            }
            if ax > p_max_x {
                p_max_x = ax;
            }
            if ay > p_max_y {
                p_max_y = ay;
            }
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
    let path_id = class_link_id_for_svg(link);
    {
        let mut path_elt = String::from("<path");
        if let Some(source_line) = link.source_line {
            write!(path_elt, r#" codeLine="{source_line}""#).unwrap();
        }
        write!(
            path_elt,
            r#" d="{d}" fill="none" id="{}" style="stroke:{link_color};stroke-width:1;{dash_style}"/>"#,
            crate::klimt::svg::xml_escape_attr(&path_id),
        )
        .unwrap();
        sg.push_raw(&path_elt);
    }

    if link.left_head != ArrowHead::None {
        emit_arrowhead(
            sg,
            tracker,
            &link.left_head,
            &decor_points,
            true,
            link_color,
            edge_offset_x,
            edge_offset_y,
        );
    }
    if link.right_head != ArrowHead::None {
        emit_arrowhead(
            sg,
            tracker,
            &link.right_head,
            &decor_points,
            false,
            link_color,
            edge_offset_x,
            edge_offset_y,
        );
    }

    if let Some(label) = &link.label {
        let margin_label = edge_label_margin(link);
        if let Some((lx, ly)) = el.label_xy.map(|(x, y)| {
            (
                x + layout.move_delta.0 - layout.normalize_offset.0 + edge_offset_x,
                y + layout.move_delta.1 - layout.normalize_offset.1 + edge_offset_y,
            )
        }) {
            let has_arrow = has_link_arrow_indicator(label);
            let label_text = if has_arrow {
                strip_link_arrow_text(label)
            } else {
                label.clone()
            };
            // Java Display.create() converts << >> to guillemets « »
            let label_text = label_text.replace("<<", "\u{00AB}").replace(">>", "\u{00BB}");
            let arrow_w = if has_arrow { LINK_LABEL_FONT_SIZE } else { 0.0 };

            if has_arrow {
                // Compute arrow direction from the rendered edge path.
                // Java uses dotPath.getStartPoint()/getEndPoint() which correspond
                // to the rendered SVG path M start and last C/L end coordinates.
                //
                // Java SvekEdge.solveLine() checks whether GraphViz inverted the
                // edge direction: if the path start is closer to entity2 than
                // entity1, it reverses the dotPath.  We replicate this check here.
                let angle_points = parse_path_start_end(&d).unwrap_or_else(|| {
                    (el.points[0], el.points[el.points.len() - 1])
                });
                let (mut sx, mut sy) = angle_points.0;
                let (mut ex, mut ey) = angle_points.1;

                // Check for Graphviz path inversion: find entity centers and
                // compare distances.  If start is closer to the link's "to"
                // entity, the path was laid out in reverse.
                let find_center = |name: &str| -> Option<(f64, f64)> {
                    layout.nodes.iter().find(|n| n.id == name).map(|n| (n.cx, n.cy))
                };
                if let (Some(pos1), Some(pos2)) =
                    (find_center(&link.from), find_center(&link.to))
                {
                    let dist = |a: (f64, f64), b: (f64, f64)| -> f64 {
                        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
                    };
                    let normal = dist((sx, sy), pos1) + dist((ex, ey), pos2);
                    let inversed = dist((sx, sy), pos2) + dist((ex, ey), pos1);
                    if inversed < normal {
                        std::mem::swap(&mut sx, &mut ex);
                        std::mem::swap(&mut sy, &mut ey);
                    }
                }

                let mut angle = (ex - sx).atan2(ey - sy);
                if is_link_arrow_backward(label) {
                    angle += std::f64::consts::PI;
                }
                // Java: addMagicArrow merges TextBlockArrow2 LEFT of the margin-wrapped text.
                // The arrow is NOT inside the margin — only the text has the margin.
                // Outer height = max(arrow_h=13, text_h + 2*margin).
                // dy_arrow = (outer_h - 13) / 2.
                let text_h = font_metrics::line_height(
                    "SansSerif",
                    LINK_LABEL_FONT_SIZE,
                    false,
                    false,
                );
                let text_marged_h = text_h + 2.0 * margin_label;
                let outer_h = text_marged_h.max(LINK_LABEL_FONT_SIZE);
                let dy_arrow = (outer_h - LINK_LABEL_FONT_SIZE) / 2.0;
                draw_label_arrow_polygon(
                    sg,
                    lx,
                    ly + dy_arrow,
                    angle,
                    LINK_LABEL_FONT_SIZE,
                );
            }

            draw_edge_label_block(
                sg,
                tracker,
                &label_text,
                lx + arrow_w,
                ly,
                el.label_wh.map(|(w, h)| (w - arrow_w, h)),
                margin_label,
                LINK_LABEL_FONT_SIZE,
                false,
                skin,
            );
        } else {
            let mid_idx = path_points.len() / 2;
            let (mx, my) = path_points[mid_idx];
            let label_x = mx + edge_offset_x;
            let label_y = my + edge_offset_y - 6.0;
            draw_label(sg, label, label_x, label_y);
            let lines = split_label_lines(label);
            let line_height =
                font_metrics::line_height("SansSerif", LINK_LABEL_FONT_SIZE, false, false);
            let ascent = font_metrics::ascent("SansSerif", LINK_LABEL_FONT_SIZE, false, false);
            let widths: Vec<f64> = lines
                .iter()
                .map(|(t, _)| {
                    font_metrics::text_width(t, "SansSerif", LINK_LABEL_FONT_SIZE, false, false)
                })
                .collect();
            let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
            let total_h = lines.len() as f64 * line_height;
            let block_x = label_x + 1.0;
            let base_y = label_y - total_h / 2.0 + ascent;
            for (idx, _line_text) in lines.iter().map(|(t, _)| t).enumerate() {
                let text_w = widths[idx];
                let ly = base_y + idx as f64 * line_height;
                tracker.track_text(block_x, ly, text_w, line_height);
            }
            tracker.track_empty(label_x, base_y, max_width + 2.0, 0.0);
        }
    }

    if let Some((text, x, y)) = edge_side_label_origin(
        layout,
        el.tail_label.as_deref(),
        el.tail_label_xy,
        edge_offset_x,
        edge_offset_y,
    ) {
        draw_edge_label_block(
            sg,
            tracker,
            text,
            x,
            y,
            el.tail_label_wh,
            if el.tail_label_boxed { 2.0 } else { 0.0 },
            if el.tail_label_boxed { 14.0 } else { LINK_LABEL_FONT_SIZE },
            el.tail_label_boxed,
            skin,
        );
    }

    if let Some((text, x, y)) = edge_side_label_origin(
        layout,
        el.head_label.as_deref(),
        el.head_label_xy,
        edge_offset_x,
        edge_offset_y,
    ) {
        draw_edge_label_block(
            sg,
            tracker,
            text,
            x,
            y,
            el.head_label_wh,
            if el.head_label_boxed { 2.0 } else { 0.0 },
            if el.head_label_boxed { 14.0 } else { LINK_LABEL_FONT_SIZE },
            el.head_label_boxed,
            skin,
        );
    }

    if let Some(text) = link.from_qualifier.as_deref() {
        if let Some(placement) = qualifier_placements.get(&QualifierKey {
            link_idx,
            endpoint: QualifierEndpoint::Tail,
        }) {
            draw_kal_box(
                sg,
                tracker,
                text,
                placement.x,
                placement.y,
                placement.width,
                placement.height,
                skin,
            );
        }
    }

    if let Some(text) = link.to_qualifier.as_deref() {
        if let Some(placement) = qualifier_placements.get(&QualifierKey {
            link_idx,
            endpoint: QualifierEndpoint::Head,
        }) {
            draw_kal_box(
                sg,
                tracker,
                text,
                placement.x,
                placement.y,
                placement.width,
                placement.height,
                skin,
            );
        }
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
    // Java extremity factories snap near-cardinal angles before drawing.
    let base_angle = crate::svek::extremity::manage_round(base_angle);

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
            // Java class-path `LinkDecor.EXTENDS` uses `ExtremityFactoryTriangle`
            // in the complete extremity chain: xWing=18, yAperture=6.
            &[(0.0, 0.0), (-18.0, -6.0), (-18.0, 6.0), (0.0, 0.0)],
            base_angle,
            tip_x,
            tip_y,
            "none",
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
            base_angle,
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
            base_angle,
            tip_x,
            tip_y,
            "none",
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

fn emit_plus_head(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    tip_x: f64,
    tip_y: f64,
    angle: f64,
    link_color: &str,
) {
    let radius = 8.0;
    let center_x = tip_x - radius * angle.cos();
    let center_y = tip_y - radius * angle.sin();
    let cross_angle = angle - std::f64::consts::FRAC_PI_2;
    sg.set_fill_color("#FFFFFF");
    sg.set_stroke_color(Some(link_color));
    sg.set_stroke_width(1.0, None);
    sg.svg_ellipse(center_x, center_y, radius, radius, 0.0);
    tracker.track_ellipse(center_x, center_y, radius, radius);

    let p1 = point_on_circle(
        center_x,
        center_y,
        radius,
        cross_angle - std::f64::consts::FRAC_PI_2,
    );
    let p2 = point_on_circle(
        center_x,
        center_y,
        radius,
        cross_angle + std::f64::consts::FRAC_PI_2,
    );
    let p3 = point_on_circle(center_x, center_y, radius, cross_angle);
    let p4 = point_on_circle(
        center_x,
        center_y,
        radius,
        cross_angle + std::f64::consts::PI,
    );
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

fn edge_side_label_origin<'a>(
    layout: &GraphLayout,
    text: Option<&'a str>,
    xy: Option<(f64, f64)>,
    edge_offset_x: f64,
    edge_offset_y: f64,
) -> Option<(&'a str, f64, f64)> {
    let text = text?;
    let (x, y) = xy?;
    Some((
        text,
        x + layout.move_delta.0 - layout.normalize_offset.0 + edge_offset_x,
        y + layout.move_delta.1 - layout.normalize_offset.1 + edge_offset_y,
    ))
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

fn kal_block_dimensions(text: &str) -> (f64, f64) {
    let font_family = "SansSerif";
    let font_size = 14.0;
    let lines = split_label_lines(text);
    let max_width = lines
        .iter()
        .map(|(t, _)| font_metrics::text_width(t, font_family, font_size, false, false))
        .fold(0.0_f64, f64::max);
    let height =
        lines.len() as f64 * font_metrics::line_height(font_family, font_size, false, false);
    (max_width + 4.0, height + 2.0)
}

fn kal_origin(anchor_x: f64, anchor_y: f64, width: f64, height: f64, pos: KalPosition) -> (f64, f64) {
    match pos {
        KalPosition::Right => (anchor_x, anchor_y - height / 2.0),
        KalPosition::Left => (anchor_x - width + 0.5, anchor_y - height / 2.0),
        KalPosition::Down => (anchor_x - width / 2.0, anchor_y),
        KalPosition::Up => (anchor_x - width / 2.0, anchor_y - height + 0.5),
    }
}

fn kal_position_for_link(link: &Link, endpoint: QualifierEndpoint) -> Option<KalPosition> {
    match endpoint {
        QualifierEndpoint::Tail => {
            if link.from_qualifier.is_none() {
                None
            } else if link.arrow_len == 1 {
                Some(KalPosition::Right)
            } else {
                Some(KalPosition::Down)
            }
        }
        QualifierEndpoint::Head => {
            if link.to_qualifier.is_none() {
                None
            } else if link.arrow_len == 1 {
                Some(KalPosition::Left)
            } else {
                Some(KalPosition::Up)
            }
        }
    }
}

fn qualifier_edge_translation(
    link: &Link,
    endpoint: QualifierEndpoint,
    placement: &KalPlacement,
) -> Option<(f64, f64)> {
    let pos = kal_position_for_link(link, endpoint)?;
    let mut dx = 0.0;
    let mut dy = 0.0;

    match endpoint {
        QualifierEndpoint::Tail => {
            // Java Kal.moveX() only moves the start point for kal1/entity1.
            if matches!(pos, KalPosition::Up | KalPosition::Down) {
                dx += placement.shift_x;
            }
            if link.left_head != ArrowHead::None {
                match pos {
                    KalPosition::Right => dx += placement.width,
                    KalPosition::Left => dx -= placement.width,
                    KalPosition::Down => dy += placement.height,
                    KalPosition::Up => dy -= placement.height,
                }
            }
        }
        QualifierEndpoint::Head => {
            if link.right_head != ArrowHead::None {
                match pos {
                    KalPosition::Right => dx += placement.width,
                    KalPosition::Left => dx -= placement.width,
                    KalPosition::Down => dy += placement.height,
                    KalPosition::Up => dy -= placement.height,
                }
            }
        }
    }

    if dx.abs() <= f64::EPSILON && dy.abs() <= f64::EPSILON {
        None
    } else {
        Some((dx, dy))
    }
}

fn compute_qualifier_placements(
    cd: &ClassDiagram,
    layout: &GraphLayout,
    edge_offset_x: f64,
    edge_offset_y: f64,
) -> HashMap<QualifierKey, KalPlacement> {
    #[derive(Debug, Clone)]
    struct PendingKal {
        key: QualifierKey,
        entity: String,
        pos: KalPosition,
        orig_x: f64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }

    let mut pending = Vec::new();
    for (link_idx, link) in cd.links.iter().enumerate() {
        let Some(edge) = layout.edges.get(link_idx) else {
            continue;
        };
        let Some(&(sx, sy)) = edge.points.first() else {
            continue;
        };
        let Some(&(ex, ey)) = edge.points.last() else {
            continue;
        };

        if let (Some(text), Some(pos)) = (
            link.from_qualifier.as_deref(),
            kal_position_for_link(link, QualifierEndpoint::Tail),
        ) {
            let (width, height) = kal_block_dimensions(text);
            let (x, y) = kal_origin(
                sx + edge_offset_x,
                sy + edge_offset_y,
                width,
                height,
                pos,
            );
            pending.push(PendingKal {
                key: QualifierKey {
                    link_idx,
                    endpoint: QualifierEndpoint::Tail,
                },
                entity: link.from.clone(),
                pos,
                orig_x: x,
                x,
                y,
                width,
                height,
            });
        }

        if let (Some(text), Some(pos)) = (
            link.to_qualifier.as_deref(),
            kal_position_for_link(link, QualifierEndpoint::Head),
        ) {
            let (width, height) = kal_block_dimensions(text);
            let (x, y) = kal_origin(
                ex + edge_offset_x,
                ey + edge_offset_y,
                width,
                height,
                pos,
            );
            pending.push(PendingKal {
                key: QualifierKey {
                    link_idx,
                    endpoint: QualifierEndpoint::Head,
                },
                entity: link.to.clone(),
                pos,
                orig_x: x,
                x,
                y,
                width,
                height,
            });
        }
    }

    let mut grouped: HashMap<(String, KalPosition), Vec<usize>> = HashMap::new();
    for (idx, item) in pending.iter().enumerate() {
        if matches!(item.pos, KalPosition::Up | KalPosition::Down) {
            grouped
                .entry((item.entity.clone(), item.pos))
                .or_default()
                .push(idx);
        }
    }

    for indices in grouped.values() {
        if indices.len() < 2 {
            continue;
        }
        let mut los = LineOfSegments::new();
        for idx in indices {
            let item = &pending[*idx];
            los.add_segment(item.x - 5.0, item.x + item.width + 5.0);
        }
        let resolved = los.solve_overlaps();
        for (order, idx) in indices.iter().enumerate() {
            let item = &mut pending[*idx];
            let old_x1 = item.x - 5.0;
            let dx = resolved[order] - old_x1;
            item.x += dx;
        }
    }

    pending
        .into_iter()
        .map(|item| {
            (
                item.key,
                KalPlacement {
                    x: item.x,
                    y: item.y,
                    width: item.width,
                    height: item.height,
                    shift_x: item.x - item.orig_x,
                },
            )
        })
        .collect()
}

fn draw_kal_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    text: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    skin: &SkinParams,
) {
    let font_family = "SansSerif";
    let font_size = 14.0;
    let line_height = font_metrics::line_height(font_family, font_size, false, false);
    let ascent = font_metrics::ascent(font_family, font_size, false, false);
    let fill = skin.background_color("class", "#F1F1F1");
    let border = skin.border_color("class", "#181818");
    let default_font = get_default_font_family_pub();

    sg.set_fill_color(&fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, width, height, 0.0, 0.0, 0.0);
    tracker.track_rect(x, y, width, height);

    for (idx, (line_text, _)) in split_label_lines(text).iter().enumerate() {
        let text_x = x + 2.0;
        let text_y = y + 1.0 + ascent + idx as f64 * line_height;
        let text_w = font_metrics::text_width(line_text, font_family, font_size, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            line_text,
            text_x,
            text_y,
            Some(&default_font),
            font_size,
            None,
            None,
            None,
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        tracker.track_text(text_x, text_y, text_w, line_height);
    }
}

fn draw_edge_label_block(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    text: &str,
    x: f64,
    y: f64,
    block_wh: Option<(f64, f64)>,
    margin: f64,
    font_size: f64,
    boxed: bool,
    skin: &SkinParams,
) {
    let font_family = "SansSerif";
    let lines = split_label_lines(text);
    let line_height = font_metrics::line_height(font_family, font_size, false, false);
    let ascent = font_metrics::ascent(font_family, font_size, false, false);
    let widths: Vec<f64> = lines
        .iter()
        .map(|(t, _)| font_metrics::text_width(t, font_family, font_size, false, false))
        .collect();
    let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
    let outer_width = block_wh.map(|(w, _)| w).unwrap_or(max_width + 2.0 * margin);
    let outer_height = block_wh
        .map(|(_, h)| h)
        .unwrap_or(lines.len() as f64 * line_height + 2.0 * margin);

    if boxed {
        let fill = skin.background_color("class", "#F1F1F1");
        let border = skin.border_color("class", "#181818");
        sg.set_fill_color(&fill);
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(0.5, None);
        sg.svg_rectangle(x, y, outer_width, outer_height, 0.0, 0.0, 0.0);
        tracker.track_rect(x, y, outer_width, outer_height);
    } else if let Some((bw, bh)) = block_wh {
        tracker.track_empty(x, y, bw, bh);
    }

    // Java: TextBlockMarged translates by (left=margin, top=margin) before
    // drawing the inner text block.  For boxed labels the margin is the box
    // inset; for non-boxed center edge labels (margin=1) this produces the
    // +1 px shift that addVisibilityModifier's TextBlockMarged applies.
    let base_x = x + margin;
    let base_y = y + margin + ascent;
    let align_width = if boxed {
        (outer_width - 2.0 * margin).max(max_width)
    } else {
        max_width
    };
    let default_font = get_default_font_family_pub();

    for (idx, (line_text, align)) in lines.iter().enumerate() {
        let text_w = widths[idx];
        let line_x = if boxed {
            base_x
        } else {
            match align {
                LabelAlign::Left => base_x,
                LabelAlign::Center => base_x + (align_width - text_w) / 2.0,
                LabelAlign::Right => base_x + (align_width - text_w),
            }
        };
        let line_y = base_y + idx as f64 * line_height;
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            line_text,
            line_x,
            line_y,
            Some(&default_font),
            font_size,
            None,
            None,
            None,
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        tracker.track_text(line_x, line_y, text_w, line_height);
    }
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
            line_text,
            line_x,
            line_y,
            Some(&default_font),
            font_size,
            None,
            None,
            None,
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
}

/// Draw a note in class diagrams (yellow sticky box with folded corner).
///
/// For left/right positioned notes with connectors (Opale style), the connector
/// arrow is integrated into the body path shape, matching Java Opale rendering.
fn draw_class_note(sg: &mut SvgGraphic, tracker: &mut BoundsTracker, note: &ClassNoteLayout, offset_x: f64, offset_y: f64) {
    let x = note.x + offset_x;
    let y = note.y + offset_y;
    let w = note.width;
    let h = note.height;

    let is_opale = matches!(note.position.as_str(), "left" | "right");
    let fold = if is_opale { CLASS_NOTE_FOLD } else { NOTE_FOLD };

    // Java Opale uses delta=4 for the connector arrow half-width on the body edge.
    const OPALE_DELTA: f64 = 4.0;

    if is_opale && note.connector.is_some() {
        // Opale note with connector: render body as <path> with embedded connector arrow.
        let (from_x_g, from_y_g, to_x_g, to_y_g) = note.connector.unwrap();
        let from_x = from_x_g + offset_x;
        let from_y = from_y_g + offset_y;
        let to_x = to_x_g + offset_x;
        let to_y = to_y_g + offset_y;
        let pp1_y_local = from_y - y;
        let pp2_x_local = to_x - x;
        let pp2_y_local = to_y - y;

        let mut d = String::with_capacity(512);
        match note.position.as_str() {
            "left" => {
                // Note is left of entity -> connector points RIGHT
                let mut y1 = pp1_y_local - OPALE_DELTA;
                y1 = y1.max(fold).min(h - 2.0 * OPALE_DELTA);

                write!(d, "M{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + y1 + 2.0 * OPALE_DELTA)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + pp2_x_local), fmt_coord(y + pp2_y_local)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + y1)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + y1)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w - fold), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y)).unwrap();
            }
            "right" => {
                // Note is right of entity -> connector points LEFT
                let mut y1 = pp1_y_local - OPALE_DELTA;
                y1 = y1.max(0.0).min(h - 2.0 * OPALE_DELTA);

                write!(d, "M{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + y1)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + pp2_x_local), fmt_coord(y + pp2_y_local)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + y1 + 2.0 * OPALE_DELTA)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + fold)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w - fold), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y)).unwrap();
            }
            _ => unreachable!(),
        }
        sg.push_raw(&format!(
            r#"<path d="{d}" fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ));
        let all_x = [x, x + w, to_x];
        let all_y = [y, y + h, to_y];
        tracker.track_path_bounds(
            all_x.iter().copied().fold(f64::INFINITY, f64::min),
            all_y.iter().copied().fold(f64::INFINITY, f64::min),
            all_x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            all_y.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        );
    } else if is_opale {
        // Opale note without connector: normal polygon as <path>
        let d = format!(
            "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{}",
            fmt_coord(x), fmt_coord(y),
            fmt_coord(x), fmt_coord(y + h),
            fmt_coord(x + w), fmt_coord(y + h),
            fmt_coord(x + w), fmt_coord(y + fold),
            fmt_coord(x + w - fold), fmt_coord(y),
            fmt_coord(x), fmt_coord(y),
        );
        sg.push_raw(&format!(
            r#"<path d="{d}" fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ));
        tracker.track_path_bounds(x, y, x + w, y + h);
    } else {
        // Non-opale note: use <polygon>
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
        sg.svg_polygon(
            0.0,
            &[
                note_poly[0].0, note_poly[0].1,
                note_poly[1].0, note_poly[1].1,
                note_poly[2].0, note_poly[2].1,
                note_poly[3].0, note_poly[3].1,
                note_poly[4].0, note_poly[4].1,
            ],
        );
        tracker.track_polygon(&note_poly);
    }

    // Fold corner triangle
    {
        let fx = fmt_coord(x + w - fold);
        let fy_top = fmt_coord(y);
        let fy_bot = fmt_coord(y + fold);
        let fx_right = fmt_coord(x + w);
        if is_opale {
            // Opale fold: (w-fold,0) -> (w-fold,fold) -> (w,fold) matching Java Opale.getCorner
            sg.push_raw(&format!(
                r#"<path d="M{fx},{fy_top} L{fx},{fy_bot} L{fx_right},{fy_bot} L{fx},{fy_top}" fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
                bg = NOTE_BG,
                border = NOTE_BORDER,
            ));
        } else {
            // Non-opale fold: existing shape (w-fold,0) -> (w-fold,fold) -> (w,0)
            sg.push_raw(&format!(
                r#"<path d="M{fx},{fy_top} L{fx},{fy_bot} L{fx_right},{fy_top} Z " fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
                bg = NOTE_BG,
                border = NOTE_BORDER,
            ));
        }
        tracker.track_path_bounds(x + w - fold, y, x + w, y + fold);
    }

    // text content -- Java Opale: marginX1=6, marginY=5, font 13pt SansSerif
    const NOTE_MARGIN_Y: f64 = 5.0;
    const NOTE_FONT_SIZE: f64 = 13.0;
    const NOTE_ASCENT: f64 = 1901.0 / 2048.0 * 13.0; // 12.0669
    const NOTE_LINE_HT: f64 = 15.1328; // SansSerif 13pt: ascent+descent

    let text_x = x + NOTE_TEXT_PADDING;
    if let Some(ref emb) = note.embedded {
        // Embedded diagram: render before-text, image, after-text
        let mut cursor_y = y + NOTE_MARGIN_Y;

        if !emb.text_before.is_empty() {
            let ty = cursor_y + NOTE_ASCENT;
            let mut tmp = String::new();
            let before_lines = render_creole_text(
                &mut tmp,
                &emb.text_before,
                text_x,
                ty,
                NOTE_LINE_HT,
                TEXT_COLOR,
                None,
                &format!(r#"font-size="{}""#, NOTE_FONT_SIZE as u32),
            );
            sg.push_raw(&tmp);
            cursor_y += before_lines as f64 * NOTE_LINE_HT;
        }

        // Emit embedded SVG as <image> element
        sg.push_raw(&format!(
            r#"<image height="{}" width="{}" x="{}" xlink:href="{}" y="{}"/>"#,
            emb.height as u32,
            emb.width as u32,
            fmt_coord(text_x),
            emb.data_uri,
            fmt_coord(cursor_y),
        ));
        cursor_y += emb.height;

        if !emb.text_after.is_empty() {
            let ty = cursor_y + NOTE_ASCENT;
            let mut tmp = String::new();
            render_creole_text(
                &mut tmp,
                &emb.text_after,
                text_x,
                ty,
                NOTE_LINE_HT,
                TEXT_COLOR,
                None,
                &format!(r#"font-size="{}""#, NOTE_FONT_SIZE as u32),
            );
            sg.push_raw(&tmp);
        }
    } else {
        let text_y = y + NOTE_MARGIN_Y + NOTE_ASCENT;
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &note.text,
            text_x,
            text_y,
            NOTE_LINE_HT,
            TEXT_COLOR,
            None,
            &format!(r#"font-size="{}""#, NOTE_FONT_SIZE as u32),
        );
        sg.push_raw(&tmp);
    }

    // For non-opale notes, draw a separate dashed connector line.
    // Opale notes embed the connector arrow in the body path.
    if !is_opale {
        if let Some((from_x, from_y, to_x, to_y)) = note.connector {
            let lx1 = from_x + offset_x;
            let ly1 = from_y + offset_y;
            let lx2 = to_x + offset_x;
            let ly2 = to_y + offset_y;
            sg.set_stroke_color(Some(NOTE_BORDER));
            sg.set_stroke_width(1.0, Some((5.0, 3.0)));
            sg.svg_line(lx1, ly1, lx2, ly2, 0.0);
            tracker.track_line(lx1, ly1, lx2, ly2);
        }
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

    fn assert_point_eq(actual: (f64, f64), expected: (f64, f64)) {
        assert!(
            (actual.0 - expected.0).abs() < 0.0001,
            "x mismatch: actual={} expected={}",
            actual.0,
            expected.0
        );
        assert!(
            (actual.1 - expected.1).abs() < 0.0001,
            "y mismatch: actual={} expected={}",
            actual.1,
            expected.1
        );
    }

    #[test]
    fn move_edge_start_point_moves_first_control_point_like_java_dotpath() {
        let mut points = vec![
            (201.0, 61.11),
            (201.0, 89.21),
            (201.0, 112.39),
            (201.0, 140.62),
        ];
        move_edge_start_point(&mut points, 0.0, 18.2969);

        assert_point_eq(points[0], (201.0, 79.4069));
        assert_point_eq(points[1], (201.0, 107.5069));
        assert_point_eq(points[2], (201.0, 112.39));
        assert_point_eq(points[3], (201.0, 140.62));
    }

    #[test]
    fn move_edge_end_point_moves_last_control_point_like_java_dotpath() {
        let mut points = vec![
            (201.0, 79.4069),
            (201.0, 107.5069),
            (201.0, 112.39),
            (201.0, 140.62),
        ];
        move_edge_end_point(&mut points, 0.0, -18.2969);

        assert_point_eq(points[0], (201.0, 79.4069));
        assert_point_eq(points[1], (201.0, 107.5069));
        assert_point_eq(points[2], (201.0, 94.0931));
        assert_point_eq(points[3], (201.0, 122.3231));
    }

    #[test]
    fn emit_diamond_hollow_arrowhead_uses_none_fill_like_java() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let mut tracker = BoundsTracker::new();
        emit_arrowhead(
            &mut sg,
            &mut tracker,
            &ArrowHead::DiamondHollow,
            &[(201.0, 237.4069), (201.0, 265.5069), (201.0, 258.0931), (201.0, 286.3231)],
            true,
            "#181818",
            0.0,
            0.0,
        );
        assert!(
            sg.body().contains(r#"<polygon fill="none""#),
            "expected hollow diamond fill to match Java aggregation output"
        );
    }

    #[test]
    fn emit_plus_head_horizontal_matches_java_geometry() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let mut tracker = BoundsTracker::new();
        emit_plus_head(&mut sg, &mut tracker, 118.9061, 183.0, 0.0, "#181818");
        let body = sg.body();
        assert!(body.contains(r##"<ellipse cx="110.9061" cy="183" fill="#FFFFFF" rx="8" ry="8""##));
        assert!(body.contains(r#"x1="102.9061" x2="118.9061" y1="183" y2="183""#));
        assert!(body.contains(r#"x1="110.9061" x2="110.9061" y1="175" y2="191""#));
    }

    fn make_link() -> Link {
        Link {
            uid: None,
            from: "Foo".into(),
            to: "Bar".into(),
            left_head: ArrowHead::None,
            right_head: ArrowHead::None,
            line_style: LineStyle::Solid,
            label: None,
            from_label: None,
            to_label: None,
            from_qualifier: None,
            to_qualifier: None,
            source_line: None,
            arrow_len: 2,
        }
    }

    #[test]
    fn qualifier_without_start_decoration_does_not_push_start_point_down() {
        let mut link = make_link();
        link.from_qualifier = Some("x: String".into());
        let placement = KalPlacement {
            x: 436.7054,
            y: 55.11,
            width: 63.2334,
            height: 18.2969,
            shift_x: 0.0,
        };

        assert_eq!(
            qualifier_edge_translation(&link, QualifierEndpoint::Tail, &placement),
            None
        );
    }

    #[test]
    fn qualifier_with_start_decoration_matches_java_downward_translation() {
        let mut link = make_link();
        link.from_qualifier = Some("c3".into());
        link.left_head = ArrowHead::DiamondHollow;
        let placement = KalPlacement {
            x: 175.0,
            y: 55.11,
            width: 21.7939,
            height: 18.2969,
            shift_x: 0.0,
        };

        assert_eq!(
            qualifier_edge_translation(&link, QualifierEndpoint::Tail, &placement),
            Some((0.0, 18.2969))
        );
    }

    #[test]
    fn downward_qualifier_overlap_only_moves_start_point_horizontally_without_decoration() {
        let mut link = make_link();
        link.from_qualifier = Some("x".into());
        let placement = KalPlacement {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 18.2969,
            shift_x: 7.5,
        };

        assert_eq!(
            qualifier_edge_translation(&link, QualifierEndpoint::Tail, &placement),
            Some((7.5, 0.0))
        );
    }

    fn simple_diagram() -> (Diagram, DiagramLayout) {
        let entity = Entity {
            name: "Foo".into(),
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
            ..Entity::default()
        };
        let entity2 = Entity {
            name: "Bar".into(),
            kind: EntityKind::Interface,
            ..Entity::default()
        };
        let link = Link {
            uid: None,
            from: "Foo".into(),
            to: "Bar".into(),
            left_head: ArrowHead::None,
            right_head: ArrowHead::Triangle,
            line_style: LineStyle::Dashed,
            label: Some("implements".into()),
            from_label: None,
            to_label: None,
            from_qualifier: None,
            to_qualifier: None,
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
                    image_width: 120.0,
                    min_x: 40.0,
                    min_y: 10.0,
                },
                NodeLayout {
                    id: "Bar".into(),
                    cx: 100.0,
                    cy: 180.0,
                    width: 120.0,
                    height: 40.0,
                    image_width: 120.0,
                    min_x: 40.0,
                    min_y: 160.0,
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
                tail_label: None,
                tail_label_xy: None,
                tail_label_wh: None,
                tail_label_boxed: false,
                head_label: None,
                head_label_xy: None,
                head_label_wh: None,
                head_label_boxed: false,
                label_xy: None,
                label_wh: None,
            }],
            clusters: vec![],
            notes: vec![],
            total_width: 240.0,
            total_height: 220.0,
            move_delta: (7.0, 7.0),
            normalize_offset: (0.0, 0.0),
            lf_span: (240.0, 220.0),
            lf_max: (240.0, 220.0),
            render_offset: (7.0, 7.0),
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
        assert!(svg.contains("font-style=\"italic\""));
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
            ..Entity::default()
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
                image_width: 100.0,
                min_x: 30.0,
                min_y: 20.0,
            }],
            edges: vec![],
            clusters: vec![],
            notes: vec![],
            total_width: 200.0,
            total_height: 100.0,
            move_delta: (7.0, 7.0),
            normalize_offset: (0.0, 0.0),
            lf_span: (200.0, 100.0),
            lf_max: (200.0, 100.0),
            render_offset: (7.0, 7.0),
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
            ..Entity::default()
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
                image_width: 100.0,
                min_x: 30.0,
                min_y: 20.0,
            }],
            edges: vec![],
            clusters: vec![],
            notes: vec![],
            total_width: 200.0,
            total_height: 100.0,
            move_delta: (7.0, 7.0),
            normalize_offset: (0.0, 0.0),
            lf_span: (200.0, 100.0),
            lf_max: (200.0, 100.0),
            render_offset: (7.0, 7.0),
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
        assert!(svg.contains("font-weight"));
        assert!(svg.contains("font-size=\"14\""));
        // Body coordinates are now shifted inline (no <g transform>)
        assert!(
            !svg.contains("translate("),
            "body should use inline coordinate offset, not <g transform>"
        );
    }

    #[test]
    fn test_meta_title_can_expand_canvas_width() {
        let (d, l) = simple_diagram();
        let body_result = render_body(&d, &l, &default_skin(), None).unwrap();
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
        assert!(svg.contains(r#"font-weight="bold""#));
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
        let svg = r#"<?plantuml 1.2026.2?><svg xmlns="http://www.w3.org/2000/svg"><defs/><g/></svg>"#;
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
        assert!(
            svg.contains("suppressed"),
            "must contain suppressed message"
        );
        assert!(svg.contains("2495"), "must reference issue 2495");
    }

    // ── Note rendering tests ────────────────────────────────────────

    #[test]
    fn test_note_renders_polygon_and_text() {
        use crate::layout::graphviz::ClassNoteLayout;

        let entity = Entity {
            name: "Foo".into(),
            ..Entity::default()
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
                image_width: 120.0,
                min_x: 40.0,
                min_y: 10.0,
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
                embedded: None,
                position: "left".into(),
            }],
            clusters: vec![],
            total_width: 300.0,
            total_height: 120.0,
            move_delta: (7.0, 7.0),
            normalize_offset: (0.0, 0.0),
            lf_span: (200.0, 100.0),
            lf_max: (200.0, 100.0),
            render_offset: (7.0, 7.0),
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
        // Opale note with connector renders body as <path> with embedded connector arrow
        assert!(
            svg.contains("<path d=\"M"),
            "opale note should render as <path> with connector arrow"
        );
        // No separate dashed connector line for opale notes
        assert_eq!(
            svg.matches("stroke-dasharray").count(),
            0,
            "opale connector is embedded in path, not a dashed line"
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
                embedded: None,
                position: "left".into(),
            }],
            clusters: vec![],
            total_width: 100.0,
            total_height: 60.0,
            move_delta: (7.0, 7.0),
            normalize_offset: (0.0, 0.0),
            lf_span: (200.0, 100.0),
            lf_max: (200.0, 100.0),
            render_offset: (7.0, 7.0),
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
