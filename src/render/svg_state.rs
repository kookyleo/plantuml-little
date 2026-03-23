use std::cell::Cell;
use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::state::{StateLayout, StateNodeLayout, StateNoteLayout, TransitionLayout};
use crate::model::state::{StateDiagram, StateKind};
use crate::render::svg::{write_svg_root_bg, write_bg_rect, DOC_MARGIN_RIGHT, DOC_MARGIN_BOTTOM};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

thread_local! { static ENT_COUNTER: Cell<u32> = const { Cell::new(2) }; }
fn next_ent_id() -> String { ENT_COUNTER.with(|c| { let id = c.get(); c.set(id + 1); format!("ent{:04}", id) }) }
fn reset_ent_counter() { ENT_COUNTER.with(|c| c.set(2)); }

// ── Style constants (PlantUML rose theme) ───────────────────────────

const FONT_SIZE: f64 = 13.0;
const DESC_FONT_SIZE: f64 = 12.0;
/// Java SansSerif 12pt: ascent(11.138671875) + descent(2.830078125) = 13.96875
const DESC_LINE_HEIGHT: f64 = 13.96875;
const LINE_HEIGHT: f64 = 16.0;
/// 8 spaces at 12pt SansSerif: 8 × (651/2048 × 12) = 30.515625
const TAB_WIDTH: f64 = 30.515625;
use crate::skin::rose::{BORDER_COLOR, ENTITY_BG, INITIAL_FILL, NOTE_BG, NOTE_BORDER, TEXT_COLOR};
const FINAL_OUTER: &str = "#000000";
const FINAL_INNER: &str = "#000000";

// ── Public entry point ──────────────────────────────────────────────

/// Render a state diagram to SVG.
pub fn render_state(
    _diagram: &StateDiagram,
    layout: &StateLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    reset_ent_counter();

    // Compute viewport using Java-compatible LimitFinder simulation.
    // Java: SvekResult draws all elements to LimitFinder, then:
    //   moveDelta = (6 - LF_minX, 6 - LF_minY)
    //   dimension = LF_span + delta(15, 15)
    //   svg_size = (int)(dimension + DOC_MARGIN + 1)
    let (svg_w, svg_h) = compute_viewport(layout);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, svg_w, svg_h, "STATE", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);

    let state_bg = skin.background_color("state", ENTITY_BG);
    let state_border = skin.border_color("state", BORDER_COLOR);
    let state_font = skin.font_color("state", TEXT_COLOR);

    let mut sg = SvgGraphic::new(0, 1.0);

    // States (including composite with children)
    for state in &layout.state_layouts {
        render_state_node(&mut sg, state, state_bg, state_border, state_font);
    }

    // Transitions
    for transition in &layout.transition_layouts {
        render_transition(&mut sg, transition);
    }

    // Notes
    for note in &layout.note_layouts {
        render_note(&mut sg, note);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── State node rendering ────────────────────────────────────────────

fn render_state_node(
    sg: &mut SvgGraphic,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    match &node.kind {
        StateKind::Fork | StateKind::Join => {
            render_fork_join(sg, node);
        }
        StateKind::Choice => {
            render_choice(sg, node, border);
        }
        StateKind::History => {
            render_history(sg, node, border, font_color, false);
        }
        StateKind::DeepHistory => {
            render_history(sg, node, border, font_color, true);
        }
        StateKind::End => {
            render_final(sg, node);
        }
        StateKind::EntryPoint => {
            render_initial(sg, node);
        }
        StateKind::ExitPoint => {
            render_exit_point(sg, node, border);
        }
        StateKind::Normal => {
            if node.is_initial {
                render_initial(sg, node);
            } else if node.is_final {
                render_final(sg, node);
            } else if node.is_composite {
                render_composite(sg, node, bg, border, font_color);
            } else {
                render_simple(sg, node, bg, border, font_color);
            }
        }
    }
}

/// Initial state: filled ellipse, rx=10 ry=10 (matches Java PlantUML)
fn render_initial(sg: &mut SvgGraphic, node: &StateNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    sg.push_raw(&format!(
        r#"<g class="start_entity"><ellipse cx="{}" cy="{}" fill="{INITIAL_FILL}" rx="10" ry="10" style="stroke:{INITIAL_FILL};stroke-width:1;"/></g>"#,
        fmt_coord(cx), fmt_coord(cy),
    ));
}

/// Final state: double circle (outer ring + inner filled)
fn render_final(sg: &mut SvgGraphic, node: &StateNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    sg.set_fill_color("none");
    sg.set_stroke_color(Some(FINAL_OUTER));
    sg.set_stroke_width(2.0, None);
    sg.svg_circle(cx, cy, 11.0, 0.0);
    sg.push_raw(&format!(
        r#"<circle cx="{}" cy="{}" fill="{FINAL_INNER}" r="7"/>"#,
        fmt_coord(cx), fmt_coord(cy),
    ));
}

/// Fork/Join bar: filled black horizontal rectangle
fn render_fork_join(sg: &mut SvgGraphic, node: &StateNodeLayout) {
    sg.push_raw(&format!(
        r#"<rect fill="{INITIAL_FILL}" height="{}" rx="2" ry="2" stroke="none" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    ));
}

/// Choice diamond: small rotated square
fn render_choice(sg: &mut SvgGraphic, node: &StateNodeLayout, border: &str) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let half = node.width / 2.0;
    sg.set_fill_color("#F1F1F1");
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_polygon(0.0, &[cx, cy - half, cx + half, cy, cx, cy + half, cx - half, cy]);
}

/// History circle: small circle with "H" (or "H*") text inside
fn render_history(
    sg: &mut SvgGraphic,
    node: &StateNodeLayout,
    border: &str,
    font_color: &str,
    deep: bool,
) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let r = node.width / 2.0;
    sg.set_fill_color("none");
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_circle(cx, cy, r, 0.0);
    let label = if deep { "H*" } else { "H" };
    let tl = font_metrics::text_width(label, "SansSerif", FONT_SIZE, true, false);
    sg.set_fill_color(font_color);
    sg.svg_text(
        label, cx, cy + FONT_SIZE * 0.35,
        Some("sans-serif"), FONT_SIZE,
        Some("bold"), None, None,
        tl, LengthAdjust::Spacing,
        None, 0, Some("middle"),
    );
}

/// Exit point: circle with X inside
fn render_exit_point(sg: &mut SvgGraphic, node: &StateNodeLayout, border: &str) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let r = node.width / 2.0;
    sg.set_fill_color("none");
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_circle(cx, cy, r, 0.0);
    // X cross inside
    let d = r * 0.5;
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_line(cx - d, cy - d, cx + d, cy + d, 0.0);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_line(cx + d, cy - d, cx - d, cy + d, 0.0);
}

/// Simple state: rounded rectangle with name + optional description
fn render_simple(
    sg: &mut SvgGraphic,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    // Open semantic <g> wrapper with entity ID
    let name_escaped = xml_escape(&node.name);
    let ent_id = next_ent_id();
    sg.push_raw(&format!(
        r#"<g class="entity" data-qualified-name="{}" id="{}">"#,
        name_escaped, ent_id,
    ));

    // Background rounded rectangle
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 12.5, 12.5, 0.0);

    // Stereotype (shown above the name in smaller text)
    let mut name_y_offset = 0.0;
    if let Some(ref stereotype) = node.stereotype {
        let stereo_text = format!("\u{00AB}{stereotype}\u{00BB}");
        let cx_s = node.x + node.width / 2.0;
        let stereo_y = node.y + FONT_SIZE + 4.0;
        let stereo_fs = FONT_SIZE - 2.0;
        let tl = font_metrics::text_width(&stereo_text, "SansSerif", stereo_fs, false, true);
        sg.set_fill_color(font_color);
        sg.svg_text(
            &stereo_text, cx_s, stereo_y,
            Some("sans-serif"), stereo_fs,
            None, Some("italic"), None,
            tl, LengthAdjust::Spacing,
            None, 0, Some("middle"),
        );
        name_y_offset = LINE_HEIGHT;
    }

    // Fixed header layout matching Java PlantUML
    let sep_y = node.y + 26.2969 + name_y_offset;
    let name_y = node.y + 17.9951 + name_y_offset;
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(node.x, sep_y, node.x + node.width, sep_y, 0.0);

    // State name text (centered)
    let name_width = font_metrics::text_width(&node.name, "SansSerif", 14.0, false, false);
    let name_x = node.x + (node.width - name_width) / 2.0;
    sg.set_fill_color(font_color);
    sg.svg_text(
        &node.name, name_x, name_y,
        Some("sans-serif"), 14.0,
        None, None, None,
        name_width, LengthAdjust::Spacing,
        None, 0, None,
    );

    // Description lines: each visual line is a separate <text> element
    if !node.description.is_empty() {
        let base_x = node.x + 5.0;
        let first_y = sep_y + 16.1386;
        let visual_lines = expand_description_lines(&node.description);
        for (i, vline) in visual_lines.iter().enumerate() {
            let x = base_x + vline.tab_count as f64 * TAB_WIDTH;
            let y = first_y + i as f64 * DESC_LINE_HEIGHT;
            render_desc_line(sg, &vline.text, x, y, font_color);
        }
    }

    // Close <g>
    sg.push_raw("</g>");
}

/// Composite state: rounded rectangle containing child states
fn render_composite(
    sg: &mut SvgGraphic,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    // Open semantic <g> wrapper with entity ID
    let name_escaped = xml_escape(&node.name);
    let ent_id = next_ent_id();
    sg.push_raw(&format!(
        r#"<g class="entity" data-qualified-name="{}" id="{}">"#,
        name_escaped, ent_id,
    ));

    // Outer rounded rectangle
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 12.5, 12.5, 0.0);

    // Composite state name at the top
    let cx = node.x + node.width / 2.0;
    let name_y = node.y + 17.9951;
    let name_tl = font_metrics::text_width(&node.name, "SansSerif", 14.0, false, false);
    sg.set_fill_color(font_color);
    sg.svg_text(
        &node.name, cx, name_y,
        Some("sans-serif"), 14.0,
        None, None, None,
        name_tl, LengthAdjust::Spacing,
        None, 0, None,
    );

    // Separator line below the header
    let sep_y = node.y + 26.2969;
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(node.x, sep_y, node.x + node.width, sep_y, 0.0);

    // Close the entity <g> before rendering children
    sg.push_raw("</g>");

    // Recursively render children
    for child in &node.children {
        render_state_node(sg, child, bg, border, font_color);
    }

    // Render concurrent region separators (dashed lines)
    for &sep_y in &node.region_separators {
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(1.0, Some((6.0, 4.0)));
        sg.svg_line(node.x + 4.0, sep_y, node.x + node.width - 4.0, sep_y, 0.0);
    }
}

// ── Transition rendering ────────────────────────────────────────────

fn render_transition(sg: &mut SvgGraphic, transition: &TransitionLayout) {
    if transition.points.is_empty() && transition.raw_path_d.is_none() {
        return;
    }

    // Open semantic <g> wrapper
    let from_escaped = xml_escape(&transition.from_id);
    let to_escaped = xml_escape(&transition.to_id);
    sg.push_raw(&format!(
        r#"<!--link {} to {}--><g class="link">"#,
        from_escaped, to_escaped,
    ));

    // Path data: prefer raw graphviz Bezier path when available
    if let Some(ref raw_d) = transition.raw_path_d {
        sg.push_raw(&format!(
            r#"<path d="{raw_d}" fill="none" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
        ));
    } else {
        let mut d = String::new();
        for (i, &(px, py)) in transition.points.iter().enumerate() {
            if i == 0 {
                write!(d, "M{},{} ", fmt_coord(px), fmt_coord(py)).unwrap();
            } else {
                write!(d, "L{},{} ", fmt_coord(px), fmt_coord(py)).unwrap();
            }
        }
        sg.push_raw(&format!(
            r#"<path d="{d}" fill="none" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
        ));
    }

    // Arrowhead polygon: prefer graphviz arrow polygon when available
    if let Some(ref poly_pts) = transition.arrow_polygon {
        if !poly_pts.is_empty() {
            let points_str: String = poly_pts.iter()
                .map(|(x, y)| format!("{},{}", fmt_coord(*x), fmt_coord(*y)))
                .collect::<Vec<_>>()
                .join(",");
            sg.push_raw(&format!(
                r#"<polygon fill="{BORDER_COLOR}" points="{points_str}" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
            ));
        }
    } else if transition.points.len() >= 2 {
        // Fallback: compute arrowhead from last segment
        let n = transition.points.len();
        let (tx, ty) = transition.points[n - 1];
        let (fx, fy) = transition.points[n - 2];

        let dx = tx - fx;
        let dy = ty - fy;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            let ux = dx / len;
            let uy = dy / len;
            let px = -uy;
            let py = ux;
            let back = 9.0;
            let side = 4.0;
            let mid_back = 5.0;
            let p1x = tx;
            let p1y = ty;
            let p2x = tx - ux * back + px * side;
            let p2y = ty - uy * back + py * side;
            let p3x = tx - ux * mid_back;
            let p3y = ty - uy * mid_back;
            let p4x = tx - ux * back - px * side;
            let p4y = ty - uy * back - py * side;

            sg.set_fill_color(BORDER_COLOR);
            sg.set_stroke_color(Some(BORDER_COLOR));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[p1x, p1y, p2x, p2y, p3x, p3y, p4x, p4y, p1x, p1y]);
        }
    }

    // Label: use graphviz label_xy position when available
    if !transition.label.is_empty() {
        let tl = font_metrics::text_width(&transition.label, "SansSerif", FONT_SIZE, false, false);
        let (lx, ly) = if let Some((x, y)) = transition.label_xy {
            (x, y)
        } else if !transition.points.is_empty() {
            let mid = transition.points.len() / 2;
            transition.points[mid]
        } else {
            return;
        };
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &transition.label, lx, ly,
            Some("sans-serif"), FONT_SIZE,
            None, None, None,
            tl, LengthAdjust::Spacing,
            None, 0, None,
        );
    }

    // Close <g>
    sg.push_raw("</g>");
}

// ── Note rendering ──────────────────────────────────────────────────

fn render_note(sg: &mut SvgGraphic, note: &StateNoteLayout) {
    let x = note.x;
    let y = note.y;
    let w = note.width;
    let h = note.height;
    let fold = 8.0;

    // Note body polygon (top-left, pre-fold top-right, fold corner, bottom-right, bottom-left)
    sg.set_fill_color(NOTE_BG);
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(1.0, None);
    sg.svg_polygon(
        0.0,
        &[x, y, x + w - fold, y, x + w, y + fold, x + w, y + h, x, y + h],
    );

    // Fold lines (vertical + horizontal)
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(1.0, None);
    sg.svg_line(x + w - fold, y, x + w - fold, y + fold, 0.0);
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(1.0, None);
    sg.svg_line(x + w - fold, y + fold, x + w, y + fold, 0.0);

    let text_x = x + 6.0;
    let text_y = y + fold + FONT_SIZE;
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        text_x,
        text_y,
        LINE_HEIGHT,
        TEXT_COLOR,
        None,
        r#"font-size="13""#,
    );
    sg.push_raw(&tmp);
}

// ── Viewport calculation ────────────────────────────────────────────

/// Compute SVG viewport dimensions matching Java PlantUML's svek model.
///
/// Java flow (SvekResult.calculateDimension):
///   1. First pass renders to LimitFinder → gets minMax bounds of drawn elements
///   2. moveDelta = (6 - LF_minX, 6 - LF_minY) shifts all positions
///   3. dimension = LF_span + delta(15, 15)
///   4. TextBlockExporter adds docMargin: finalDim = dim + (R=5, B=5)
///   5. SvgGraphics.ensureVisible: svg_size = (int)(finalDim + 1)
///
/// For state diagrams with layout.width/height already containing content + 2*MARGIN(7):
///   layout.width ≈ content_w + 14
///   Java viewport ≈ content_w + 21 + R_margin + 1 (integer rounded)
fn compute_viewport(layout: &StateLayout) -> (f64, f64) {
    // The layout width/height include 2*MARGIN = 14.
    // Java viewport = LF_span + 15 + 5 + 1 = LF_span + 21.
    // LF_span tracks rects at (x-1), lines, text, etc.
    // For a simple approximation: LF_span ≈ layout.width - 2*MARGIN + extra for LF adjustments.
    // The old formula that worked: width + DOC_MARGIN_RIGHT + 1.0 + 2.0.
    // That gives: (content + 14) + 5 + 1 + 2 = content + 22 ≈ LF_span + 21 when LF_span ≈ content + 1.
    let svg_w = (layout.width + DOC_MARGIN_RIGHT + 1.0 + 2.0) as i32 as f64;
    let svg_h = (layout.height + DOC_MARGIN_BOTTOM + 1.0 + 1.0) as i32 as f64;

    (svg_w, svg_h)
}

// ── Helper functions ────────────────────────────────────────────────

fn count_leading_tabs(line: &str) -> (usize, &str) {
    let mut count = 0;
    let mut rest = line;
    while let Some(stripped) = rest.strip_prefix("\\t") {
        count += 1;
        rest = stripped;
    }
    (count, rest)
}

struct VisualLine { tab_count: usize, text: String }
fn expand_description_lines(descriptions: &[String]) -> Vec<VisualLine> {
    let mut vl = Vec::new();
    for desc in descriptions {
        for part in split_backslash_n(desc) {
            let (tabs, text) = count_leading_tabs(part);
            let text = if text.is_empty() { "\u{00A0}".to_string() } else { text.to_string() };
            vl.push(VisualLine { tab_count: tabs, text });
        }
    }
    vl
}
fn split_backslash_n(s: &str) -> Vec<&str> {
    let mut r = Vec::new(); let mut start = 0; let b = s.as_bytes(); let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() && b[i + 1] == b'n' { r.push(&s[start..i]); start = i + 2; i += 2; }
        else { i += 1; }
    }
    r.push(&s[start..]); r
}
fn render_desc_line(sg: &mut SvgGraphic, text: &str, x: f64, y: f64, fc: &str) {
    if text.contains("**") { render_desc_line_bold(sg, text, x, y, fc); return; }
    let (d, tl) = if text == "\u{00A0}" {
        ("&#160;".to_string(), font_metrics::text_width("\u{00A0}", "SansSerif", DESC_FONT_SIZE, false, false))
    } else { (xml_escape(text), font_metrics::text_width(text, "SansSerif", DESC_FONT_SIZE, false, false)) };
    sg.push_raw(&format!(r#"<text fill="{fc}" font-family="sans-serif" font-size="12" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{d}</text>"#,
        fmt_coord(tl), fmt_coord(x), fmt_coord(y)));
}
fn render_desc_line_bold(sg: &mut SvgGraphic, text: &str, x: f64, y: f64, fc: &str) {
    let mut cx = x; let mut ib = false;
    for part in text.split("**") {
        if part.is_empty() { ib = !ib; continue; }
        let e = xml_escape(part);
        let tl = font_metrics::text_width(part, "SansSerif", DESC_FONT_SIZE, ib, false);
        if ib { sg.push_raw(&format!(r#"<text fill="{fc}" font-family="sans-serif" font-size="12" font-weight="700" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{e}</text>"#, fmt_coord(tl), fmt_coord(cx), fmt_coord(y))); }
        else { sg.push_raw(&format!(r#"<text fill="{fc}" font-family="sans-serif" font-size="12" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{e}</text>"#, fmt_coord(tl), fmt_coord(cx), fmt_coord(y))); }
        cx += tl; ib = !ib;
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::state::{StateLayout, StateNodeLayout, StateNoteLayout, TransitionLayout};
    use crate::model::state::StateDiagram;
    use crate::style::SkinParams;

    fn empty_diagram() -> StateDiagram {
        StateDiagram { states: vec![], transitions: vec![], notes: vec![], direction: Default::default() }
    }

    fn empty_layout() -> StateLayout {
        StateLayout { width: 300.0, height: 200.0, state_layouts: vec![], transition_layouts: vec![], note_layouts: vec![], move_delta: (7.0, 7.0), lf_span: (300.0, 200.0) }
    }

    fn make_initial(x: f64, y: f64) -> StateNodeLayout {
        StateNodeLayout {
            id: "[*]_initial".to_string(), name: String::new(), x, y, width: 20.0, height: 20.0,
            description: vec![], stereotype: None, is_initial: true, is_final: false, is_composite: false,
            children: vec![], kind: crate::model::state::StateKind::default(), region_separators: Vec::new(),
        }
    }

    fn make_final(x: f64, y: f64) -> StateNodeLayout {
        StateNodeLayout {
            id: "[*]_final".to_string(), name: String::new(), x, y, width: 22.0, height: 22.0,
            description: vec![], stereotype: None, is_initial: false, is_final: true, is_composite: false,
            children: vec![], kind: crate::model::state::StateKind::default(), region_separators: Vec::new(),
        }
    }

    fn make_simple(id: &str, name: &str, x: f64, y: f64, w: f64, h: f64) -> StateNodeLayout {
        StateNodeLayout {
            id: id.to_string(), name: name.to_string(), x, y, width: w, height: h,
            description: vec![], stereotype: None, is_initial: false, is_final: false, is_composite: false,
            children: vec![], kind: crate::model::state::StateKind::default(), region_separators: Vec::new(),
        }
    }

    #[test]
    fn test_empty_diagram() {
        let diagram = empty_diagram();
        let layout = empty_layout();
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("<defs/>"), "must contain <defs/>");
        assert!(!svg.contains("<ellipse"), "empty diagram has no ellipses");
        assert!(!svg.contains("<rect"), "empty diagram has no rects");
    }

    #[test]
    fn test_initial_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(make_initial(90.0, 10.0));
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"rx="10""#), "initial ellipse must have rx=10");
        assert!(svg.contains(r#"ry="10""#), "initial ellipse must have ry=10");
        assert!(svg.contains(&format!(r#"fill="{INITIAL_FILL}""#)), "initial ellipse must be filled");
        assert_eq!(svg.matches("<ellipse").count(), 1, "initial state must produce exactly one ellipse");
        assert!(svg.contains(r#"class="start_entity""#), "initial state must be wrapped in start_entity group");
    }

    #[test]
    fn test_final_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(make_final(90.0, 80.0));
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert_eq!(svg.matches("<circle").count(), 2, "final state must produce two circles");
        assert!(svg.contains(r#"r="11""#), "final outer ring must have r=11");
        assert!(svg.contains(r#"r="7""#), "final inner circle must have r=7");
        assert!(svg.contains("stroke-width:2;"), "outer ring must have stroke-width=2");
    }

    #[test]
    fn test_simple_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(make_simple("Idle", "Idle", 30.0, 40.0, 100.0, 40.0));
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"rx="12.5""#), "state must have rounded corners rx=12.5");
        assert!(svg.contains(r#"ry="12.5""#), "state must have rounded corners ry=12.5");
        assert!(svg.contains(r##"fill="#F1F1F1""##), "state must use default theme state_bg fill");
        assert!(svg.contains("Idle"), "state name must appear in SVG");
        assert!(svg.contains(r#"class="entity""#), "state must be wrapped in entity group");
        assert!(svg.contains("stroke-width:0.5;"), "state border must have stroke-width:0.5");
    }

    #[test]
    fn test_state_with_description() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("Active", "Active", 20.0, 30.0, 140.0, 80.0);
        node.description = vec!["entry / start timer".to_string(), "exit / stop timer".to_string()];
        layout.state_layouts.push(node);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Active"), "state name must appear");
        assert!(svg.contains("entry / start timer"), "first description line must appear");
        assert!(svg.contains("exit / stop timer"), "second description line must appear");
        assert!(svg.contains("<line"), "separator line must exist between name and description");
    }

    #[test]
    fn test_state_with_stereotype() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("InputPin", "InputPin", 20.0, 30.0, 120.0, 50.0);
        node.stereotype = Some("inputPin".to_string());
        layout.state_layouts.push(node);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("InputPin"), "state name must appear");
        assert!(svg.contains("&#171;inputPin&#187;"), "stereotype must appear with guillemets");
        assert!(svg.contains("font-style=\"italic\""), "stereotype must be italic");
    }

    #[test]
    fn test_composite_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let child = make_simple("Inner", "Inner", 50.0, 80.0, 80.0, 36.0);
        let composite = StateNodeLayout {
            id: "Outer".to_string(), name: "Outer".to_string(),
            x: 20.0, y: 30.0, width: 200.0, height: 120.0,
            description: vec![], stereotype: None,
            is_initial: false, is_final: false, is_composite: true,
            children: vec![child], kind: crate::model::state::StateKind::default(),
            region_separators: Vec::new(),
        };
        layout.state_layouts.push(composite);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Outer"), "composite name must appear");
        assert!(svg.contains("Inner"), "child state name must appear");
        let rect_count = svg.matches("<rect").count();
        assert!(rect_count >= 2, "composite must produce at least 2 rects, got {rect_count}");
        assert!(svg.contains("<line"), "composite must have separator line below header");
    }

    #[test]
    fn test_transition_with_arrow() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(), to_id: "B".to_string(), label: String::new(),
            points: vec![(100.0, 50.0), (100.0, 120.0)],
            raw_path_d: None, arrow_polygon: None, label_xy: None,
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<polygon"), "transition must have inline polygon arrowhead");
        assert!(svg.contains("stroke:#181818"), "transition must use BORDER_COLOR in style");
        assert!(svg.contains("<path "), "transition must use <path>");
        assert!(svg.contains(r#"class="link""#), "transition must be in link group");
    }

    #[test]
    fn test_transition_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "Idle".to_string(), to_id: "Active".to_string(), label: "start".to_string(),
            points: vec![(80.0, 40.0), (80.0, 100.0)],
            raw_path_d: None, arrow_polygon: None, label_xy: None,
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("start"), "transition label must appear in SVG");
        assert!(svg.contains(r#"lengthAdjust="spacing""#), "label must have lengthAdjust");
    }

    #[test]
    fn test_polyline_transition() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(), to_id: "B".to_string(), label: String::new(),
            points: vec![(50.0, 20.0), (50.0, 50.0), (100.0, 50.0), (100.0, 80.0)],
            raw_path_d: None, arrow_polygon: None, label_xy: None,
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<path"), "multi-point transition must use <path>");
        assert!(svg.contains("<polygon"), "multi-point transition must have inline polygon arrowhead");
    }

    #[test]
    fn test_note_rendering() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.note_layouts.push(StateNoteLayout { x: 10.0, y: 20.0, width: 120.0, height: 40.0, text: "important note".to_string() });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(&format!(r#"fill="{NOTE_BG}""#)), "note must use yellow background");
        assert!(svg.contains("important note"), "note text must appear");
        assert!(svg.contains("<polygon"), "note body must be a polygon with folded corner");
        let line_count = svg.matches("<line").count();
        assert!(line_count >= 2, "note must have at least 2 fold lines, got {line_count}");
    }

    #[test]
    fn test_multiline_note() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.note_layouts.push(StateNoteLayout { x: 10.0, y: 20.0, width: 120.0, height: 60.0, text: "line one\nline two".to_string() });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<tspan"), "multiline note must use tspan");
        assert!(svg.contains("line one"), "first line must appear");
        assert!(svg.contains("line two"), "second line must appear");
        assert_eq!(svg.matches("<tspan").count(), 2, "two lines must produce two tspan elements");
    }

    #[test]
    fn test_xml_escaping() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("test", "A & B < C", 10.0, 10.0, 120.0, 40.0);
        node.description = vec!["x > y & z".to_string()];
        layout.state_layouts.push(node);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("A &amp; B &lt; C"), "state name must be XML-escaped");
        assert!(svg.contains("x &gt; y &amp; z"), "description must be XML-escaped");
    }

    #[test]
    fn test_full_svg_structure() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout.state_layouts.push(make_initial(180.0, 10.0));
        layout.state_layouts.push(make_simple("Running", "Running", 130.0, 50.0, 120.0, 40.0));
        layout.state_layouts.push(make_final(180.0, 120.0));
        layout.transition_layouts.push(TransitionLayout {
            from_id: "[*]_initial".to_string(), to_id: "Running".to_string(), label: String::new(),
            points: vec![(190.0, 30.0), (190.0, 50.0)],
            raw_path_d: None, arrow_polygon: None, label_xy: None,
        });
        layout.transition_layouts.push(TransitionLayout {
            from_id: "Running".to_string(), to_id: "[*]_final".to_string(), label: "done".to_string(),
            points: vec![(190.0, 90.0), (190.0, 120.0)],
            raw_path_d: None, arrow_polygon: None, label_xy: None,
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.contains("</svg>"), "SVG must end with </svg>");
        // SVG viewport = layout dims + DOC_MARGIN(5) + ensureVisible(1) + svek adjustment(+2w,+1h)
        assert!(svg.contains("viewBox=\"0 0 408 307\""), "viewBox must match layout + doc margin");
        assert!(svg.contains("width=\"408px\""), "width must match layout + doc margin");
        assert!(svg.contains("height=\"307px\""), "height must match layout + doc margin");
        assert!(svg.contains("<defs/>"), "must have <defs/>");
        assert_eq!(svg.matches("<ellipse").count(), 1, "1 ellipse expected");
        assert_eq!(svg.matches("<circle").count(), 2, "2 circles expected");
        assert_eq!(svg.matches("<rect").count(), 1, "1 rect expected");
        assert_eq!(svg.matches(r#"class="link""#).count(), 2, "2 transitions with link groups expected");
        assert!(svg.contains("done"), "transition label 'done' must appear");
    }

    #[test]
    fn test_empty_transition_points() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(), to_id: "B".to_string(), label: "skip".to_string(), points: vec![],
            raw_path_d: None, arrow_polygon: None, label_xy: None,
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(!svg.contains("<path"), "empty points should not produce a path");
        assert!(!svg.contains("skip"), "empty points should not produce a label");
    }

    #[test]
    fn test_fork_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "fork1".to_string(), name: "fork1".to_string(),
            x: 30.0, y: 40.0, width: 80.0, height: 6.0,
            description: vec![], stereotype: None,
            is_initial: false, is_final: false, is_composite: false,
            children: vec![], kind: StateKind::Fork, region_separators: Vec::new(),
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "fork bar must produce a rect");
        assert!(svg.contains(&format!(r#"fill="{INITIAL_FILL}""#)), "fork bar must be filled");
        assert!(svg.contains(r#"rx="2""#), "fork bar must have minimal rounding");
    }

    #[test]
    fn test_join_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "join1".to_string(), name: "join1".to_string(),
            x: 30.0, y: 40.0, width: 80.0, height: 6.0,
            description: vec![], stereotype: None,
            is_initial: false, is_final: false, is_composite: false,
            children: vec![], kind: StateKind::Join, region_separators: Vec::new(),
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "join bar must produce a rect");
    }

    #[test]
    fn test_choice_diamond() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "choice1".to_string(), name: "choice1".to_string(),
            x: 50.0, y: 50.0, width: 20.0, height: 20.0,
            description: vec![], stereotype: None,
            is_initial: false, is_final: false, is_composite: false,
            children: vec![], kind: StateKind::Choice, region_separators: Vec::new(),
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<polygon"), "choice must produce a polygon (diamond)");
    }

    #[test]
    fn test_history_circle() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "Active[H]".to_string(), name: "Active[H]".to_string(),
            x: 50.0, y: 50.0, width: 24.0, height: 24.0,
            description: vec![], stereotype: None,
            is_initial: false, is_final: false, is_composite: false,
            children: vec![], kind: StateKind::History, region_separators: Vec::new(),
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<circle"), "history must produce a circle");
        assert!(svg.contains(">H<"), "history must contain 'H' text");
    }

    #[test]
    fn test_deep_history_circle() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "Active[H*]".to_string(), name: "Active[H*]".to_string(),
            x: 50.0, y: 50.0, width: 24.0, height: 24.0,
            description: vec![], stereotype: None,
            is_initial: false, is_final: false, is_composite: false,
            children: vec![], kind: StateKind::DeepHistory, region_separators: Vec::new(),
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<circle"), "deep history must produce a circle");
        assert!(svg.contains(">H*<"), "deep history must contain 'H*' text");
    }

    #[test]
    fn test_concurrent_separator() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let child1 = make_simple("Sub1", "Sub1", 40.0, 60.0, 80.0, 36.0);
        let child2 = make_simple("Sub3", "Sub3", 40.0, 140.0, 80.0, 36.0);
        let composite = StateNodeLayout {
            id: "Active".to_string(), name: "Active".to_string(),
            x: 20.0, y: 30.0, width: 200.0, height: 180.0,
            description: vec![], stereotype: None,
            is_initial: false, is_final: false, is_composite: true,
            children: vec![child1, child2], kind: StateKind::Normal,
            region_separators: vec![110.0],
        };
        layout.state_layouts.push(composite);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("stroke-dasharray"), "concurrent separator must be dashed");
    }
}
