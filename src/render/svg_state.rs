use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::state::{StateLayout, StateNodeLayout, StateNoteLayout, TransitionLayout};
use crate::model::state::{StateDiagram, StateKind};
use crate::render::svg::{
    ensure_visible_int, write_bg_rect, write_svg_root_bg, BoundsTracker, CANVAS_DELTA,
    DOC_MARGIN_BOTTOM, DOC_MARGIN_RIGHT,
};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

thread_local! { static ENT_COUNTER: Cell<u32> = const { Cell::new(2) }; }
thread_local! { static LNK_COUNTER: Cell<u32> = const { Cell::new(3) }; }
fn next_ent_id() -> String {
    ENT_COUNTER.with(|c| {
        let id = c.get();
        c.set(id + 1);
        format!("ent{:04}", id)
    })
}
fn next_lnk_id() -> String {
    LNK_COUNTER.with(|c| {
        let id = c.get();
        c.set(id + 1);
        format!("lnk{}", id)
    })
}
fn reset_ent_counter() {
    ENT_COUNTER.with(|c| c.set(2));
}
fn reset_lnk_counter() {
    LNK_COUNTER.with(|c| c.set(3));
}

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
/// Java ExtremityArrow.getDecorationLength() = 6.
const ARROW_DECORATION_LEN: f64 = 6.0;

// ── Public entry point ──────────────────────────────────────────────

/// Render a state diagram to SVG.
/// Returns (svg_string, raw_body_dim) where raw_body_dim is the precise
/// body content size matching Java SvekResult.calculateDimension().
pub fn render_state(
    _diagram: &StateDiagram,
    layout: &StateLayout,
    skin: &SkinParams,
) -> Result<(String, Option<(f64, f64)>)> {
    let mut buf = String::with_capacity(4096);
    reset_ent_counter();
    reset_lnk_counter();

    let state_bg = skin.background_color("state", ENTITY_BG);
    let state_border = skin.border_color("state", BORDER_COLOR);
    let state_font = skin.font_color("state", TEXT_COLOR);

    let mut sg = SvgGraphic::new(0, 1.0);
    let mut tracker = BoundsTracker::new();

    // Build state_id → ent_id mapping (pre-assign for ordering consistency).
    let mut ent_id_map: HashMap<String, String> = HashMap::new();

    // Collect all states including composite children for ent_id assignment
    fn collect_all_states<'a>(states: &'a [StateNodeLayout], out: &mut Vec<&'a StateNodeLayout>) {
        for state in states {
            out.push(state);
            collect_all_states(&state.children, out);
        }
    }
    let mut all_states_flat: Vec<&StateNodeLayout> = Vec::new();
    collect_all_states(&layout.state_layouts, &mut all_states_flat);

    // Pass 1: assign ent_ids to regular entities first (matching Java order).
    for state in &all_states_flat {
        if !state.is_initial
            && !state.is_final
            && !matches!(
                state.kind,
                StateKind::EntryPoint
                    | StateKind::ExitPoint
                    | StateKind::End
                    | StateKind::Fork
                    | StateKind::Join
                    | StateKind::Choice
                    | StateKind::History
                    | StateKind::DeepHistory
            )
        {
            ent_id_map.insert(state.id.clone(), next_ent_id());
        }
    }
    // Pass 2: assign ent_ids to special entities. For [*] initial states,
    // Java reuses the UID of the target entity from the first [*] transition.
    for state in &all_states_flat {
        if state.is_initial
            || state.is_final
            || matches!(
                state.kind,
                StateKind::EntryPoint
                    | StateKind::ExitPoint
                    | StateKind::End
                    | StateKind::Fork
                    | StateKind::Join
                    | StateKind::Choice
                    | StateKind::History
                    | StateKind::DeepHistory
            )
        {
            if state.is_initial && (state.id == "[*]" || state.id.starts_with("[*]__start") || state.id.starts_with("[*]")) {
                // Find the target of the first transition from [*]
                let target_ent_id = layout
                    .transition_layouts
                    .iter()
                    .find(|t| t.from_id == state.id)
                    .and_then(|t| ent_id_map.get(&t.to_id))
                    .cloned();
                if let Some(id) = target_ent_id {
                    ent_id_map.insert(state.id.clone(), id);
                } else {
                    ent_id_map.insert(state.id.clone(), next_ent_id());
                }
            } else if !ent_id_map.contains_key(&state.id) {
                ent_id_map.insert(state.id.clone(), next_ent_id());
            }
        }
    }

    // Build set of child IDs for each composite state to identify internal transitions.
    let mut rendered_transitions: HashSet<usize> = HashSet::new();
    fn collect_child_ids(node: &StateNodeLayout, ids: &mut HashSet<String>) {
        for child in &node.children {
            ids.insert(child.id.clone());
            collect_child_ids(child, ids);
        }
    }

    // Java renders cluster (composite) states first: their header and children
    // appear before simple top-level entities. Internal transitions are rendered
    // immediately after their composite state's children.
    // Pass 1: composite states (clusters) — header + children + internal transitions
    for state in &layout.state_layouts {
        if state.is_composite
            && !state.is_initial
            && !state.is_final
            && !matches!(
                state.kind,
                StateKind::EntryPoint
                    | StateKind::ExitPoint
                    | StateKind::End
                    | StateKind::Fork
                    | StateKind::Join
                    | StateKind::Choice
                    | StateKind::History
                    | StateKind::DeepHistory
            )
        {
            render_state_node(
                &mut sg,
                &mut tracker,
                state,
                state_bg,
                state_border,
                state_font,
                &ent_id_map,
            );

            // Render internal transitions (both endpoints are children of this composite)
            let mut child_ids = HashSet::new();
            collect_child_ids(state, &mut child_ids);
            for (ti, transition) in layout.transition_layouts.iter().enumerate() {
                if !rendered_transitions.contains(&ti)
                    && child_ids.contains(&transition.from_id)
                    && child_ids.contains(&transition.to_id)
                {
                    render_transition(&mut sg, &mut tracker, transition, &ent_id_map);
                    rendered_transitions.insert(ti);
                }
            }
        }
    }
    // Pass 2: regular non-composite entities
    for state in &layout.state_layouts {
        if !state.is_composite
            && !state.is_initial
            && !state.is_final
            && !matches!(
                state.kind,
                StateKind::EntryPoint
                    | StateKind::ExitPoint
                    | StateKind::End
                    | StateKind::Fork
                    | StateKind::Join
                    | StateKind::Choice
                    | StateKind::History
                    | StateKind::DeepHistory
            )
        {
            render_state_node(
                &mut sg,
                &mut tracker,
                state,
                state_bg,
                state_border,
                state_font,
                &ent_id_map,
            );
        }
    }
    // Pass 3: special entities (initial, final, fork, join, choice, history, etc.)
    for state in &layout.state_layouts {
        if state.is_initial
            || state.is_final
            || matches!(
                state.kind,
                StateKind::EntryPoint
                    | StateKind::ExitPoint
                    | StateKind::End
                    | StateKind::Fork
                    | StateKind::Join
                    | StateKind::Choice
                    | StateKind::History
                    | StateKind::DeepHistory
            )
        {
            render_state_node(
                &mut sg,
                &mut tracker,
                state,
                state_bg,
                state_border,
                state_font,
                &ent_id_map,
            );
        }
    }

    // Notes
    for note in &layout.note_layouts {
        render_note(&mut sg, &mut tracker, note);
    }

    // Remaining transitions (top-level, not rendered as internal above)
    for (ti, transition) in layout.transition_layouts.iter().enumerate() {
        if !rendered_transitions.contains(&ti) {
            render_transition(&mut sg, &mut tracker, transition, &ent_id_map);
        }
    }

    // Compute raw body dimensions from BoundsTracker span
    // Java: SvekResult.calculateDimension = LF_span + delta(15, 15)
    let (span_w, span_h) = tracker.span();
    let raw_body_dim = (span_w + CANVAS_DELTA, span_h + CANVAS_DELTA);
    log::debug!(
        "state viewport: span=({span_w:.2}, {span_h:.2}) raw_body_dim=({:.2}, {:.2})",
        raw_body_dim.0,
        raw_body_dim.1,
    );

    // Java ensureVisible: maxX = (int)(x + 1)
    let svg_w = ensure_visible_int(raw_body_dim.0 + DOC_MARGIN_RIGHT) as f64;
    let svg_h = ensure_visible_int(raw_body_dim.1 + DOC_MARGIN_BOTTOM) as f64;

    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, svg_w, svg_h, "STATE", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok((buf, Some(raw_body_dim)))
}

// ── State node rendering ────────────────────────────────────────────

fn render_state_node(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
    ent_id_map: &HashMap<String, String>,
) {
    render_state_node_with_parent(sg, tracker, node, bg, border, font_color, ent_id_map, None);
}

fn render_state_node_with_parent(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
    ent_id_map: &HashMap<String, String>,
    parent_name: Option<&str>,
) {
    match &node.kind {
        StateKind::Fork | StateKind::Join => {
            render_fork_join(sg, tracker, node);
        }
        StateKind::Choice => {
            render_choice(sg, tracker, node, border);
        }
        StateKind::History => {
            render_history(sg, tracker, node, border, font_color, false);
        }
        StateKind::DeepHistory => {
            render_history(sg, tracker, node, border, font_color, true);
        }
        StateKind::End => {
            render_final(sg, tracker, node, ent_id_map, parent_name);
        }
        StateKind::EntryPoint => {
            render_initial(sg, tracker, node, ent_id_map, parent_name);
        }
        StateKind::ExitPoint => {
            render_exit_point(sg, tracker, node, border);
        }
        StateKind::Normal => {
            if node.is_initial {
                render_initial(sg, tracker, node, ent_id_map, parent_name);
            } else if node.is_final {
                render_final(sg, tracker, node, ent_id_map, parent_name);
            } else if node.is_composite {
                render_composite(sg, tracker, node, bg, border, font_color, ent_id_map);
            } else {
                render_simple(
                    sg,
                    tracker,
                    node,
                    bg,
                    border,
                    font_color,
                    ent_id_map,
                    parent_name,
                );
            }
        }
    }
}

/// Initial state: filled ellipse, rx=10 ry=10 (matches Java PlantUML)
fn render_initial(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    ent_id_map: &HashMap<String, String>,
    parent_name: Option<&str>,
) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let ent_id = ent_id_map
        .get(&node.id)
        .cloned()
        .unwrap_or_else(|| "ent0002".to_string());
    // Java qualified name: ".start." for top-level, "Parent..start.Parent" for nested
    let qname = match parent_name {
        Some(p) => format!("{}..start.{}", p, p),
        None => ".start.".to_string(),
    };
    let mut attrs = format!(r#" data-qualified-name="{}""#, xml_escape(&qname));
    if let Some(sl) = node.source_line {
        write!(attrs, r#" data-source-line="{}""#, sl).unwrap();
    }
    write!(attrs, r#" id="{}""#, ent_id).unwrap();
    sg.push_raw(&format!(
        r#"<g class="start_entity"{attrs}><ellipse cx="{}" cy="{}" fill="{INITIAL_FILL}" rx="10" ry="10" style="stroke:{INITIAL_FILL};stroke-width:1;"/></g>"#,
        fmt_coord(cx), fmt_coord(cy),
    ));
    // Java LimitFinder.drawEllipse: addPoint(x, y), addPoint(x+w-1, y+h-1)
    tracker.track_ellipse(cx, cy, 10.0, 10.0);
}

/// Final state: double circle (outer ring + inner filled)
/// Java: EntityImageCircleEnd renders two UEllipses (outer 22x22 + inner 12x12)
fn render_final(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    ent_id_map: &HashMap<String, String>,
    parent_name: Option<&str>,
) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let ent_id = ent_id_map
        .get(&node.id)
        .cloned()
        .unwrap_or_else(|| next_ent_id());
    // Java qualified name: ".end." for top-level, "Parent..end.Parent" for nested
    let qname = match parent_name {
        Some(p) => format!("{}..end.{}", p, p),
        None => ".end.".to_string(),
    };
    let mut attrs = format!(r#" data-qualified-name="{}""#, xml_escape(&qname));
    if let Some(sl) = node.source_line {
        write!(attrs, r#" data-source-line="{}""#, sl).unwrap();
    }
    write!(attrs, r#" id="{}""#, ent_id).unwrap();
    // Outer ring: stroke only, no fill
    sg.push_raw(&format!(
        r#"<g class="end_entity"{attrs}><ellipse cx="{}" cy="{}" fill="none" rx="11" ry="11" style="stroke:{INITIAL_FILL};stroke-width:1;"/>"#,
        fmt_coord(cx),
        fmt_coord(cy),
    ));
    // Inner filled dot
    sg.push_raw(&format!(
        r#"<ellipse cx="{}" cy="{}" fill="{INITIAL_FILL}" rx="6" ry="6" style="stroke:{INITIAL_FILL};stroke-width:1;"/></g>"#,
        fmt_coord(cx),
        fmt_coord(cy),
    ));
    // Java LimitFinder.drawEllipse: outer ring r=11
    tracker.track_ellipse(cx, cy, 11.0, 11.0);
}

/// Fork/Join bar: filled black horizontal rectangle
fn render_fork_join(sg: &mut SvgGraphic, tracker: &mut BoundsTracker, node: &StateNodeLayout) {
    sg.push_raw(&format!(
        r#"<rect fill="{INITIAL_FILL}" height="{}" rx="2" ry="2" stroke="none" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    ));
    tracker.track_rect(node.x, node.y, node.width, node.height);
}

/// Choice diamond: small rotated square
fn render_choice(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    border: &str,
) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let half = node.width / 2.0;
    sg.set_fill_color("#F1F1F1");
    sg.set_stroke_color(Some(border));
    // Java: style.getStroke() default for state diamond is 0.5
    sg.set_stroke_width(0.5, None);
    // Java: EntityImageBranch.drawU adds 5 points (last = first to close polygon)
    sg.svg_polygon(
        0.0,
        &[cx, cy - half, cx + half, cy, cx, cy + half, cx - half, cy, cx, cy - half],
    );
    // Java LimitFinder.drawUPolygon with HACK_X_FOR_POLYGON=10
    tracker.track_polygon(&[
        (cx, cy - half),
        (cx + half, cy),
        (cx, cy + half),
        (cx - half, cy),
    ]);
}

/// History circle: small circle with "H" (or "H*") text inside
fn render_history(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
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
        label,
        cx,
        cy + FONT_SIZE * 0.35,
        Some("sans-serif"),
        FONT_SIZE,
        Some("bold"),
        None,
        None,
        tl,
        LengthAdjust::Spacing,
        None,
        0,
        Some("middle"),
    );
    tracker.track_ellipse(cx, cy, r, r);
}

/// Exit point: circle with X inside
fn render_exit_point(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    border: &str,
) {
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
    tracker.track_ellipse(cx, cy, r, r);
}

/// Simple state: rounded rectangle with name + optional description
fn render_simple(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
    ent_id_map: &HashMap<String, String>,
    parent_name: Option<&str>,
) {
    // Open semantic <g> wrapper with entity ID
    // Java qualified name: "Name" for top-level, "Parent.Name" for nested
    let qname = match parent_name {
        Some(p) => format!("{}.{}", p, node.name),
        None => node.name.clone(),
    };
    let qname_escaped = xml_escape(&qname);
    let ent_id = ent_id_map
        .get(&node.id)
        .cloned()
        .unwrap_or_else(next_ent_id);
    sg.push_raw(&format!(
        r#"<g class="entity" data-qualified-name="{}" id="{}">"#,
        qname_escaped, ent_id,
    ));

    // Background rounded rectangle
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 12.5, 12.5, 0.0);
    // Java LimitFinder.drawRectangle: addPoint(x-1, y-1), addPoint(x+w-1, y+h-1)
    tracker.track_rect(node.x, node.y, node.width, node.height);

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
            &stereo_text,
            cx_s,
            stereo_y,
            Some("sans-serif"),
            stereo_fs,
            None,
            Some("italic"),
            None,
            tl,
            LengthAdjust::Spacing,
            None,
            0,
            Some("middle"),
        );
        name_y_offset = LINE_HEIGHT;
    }

    // Fixed header layout matching Java PlantUML
    let sep_y = node.y + 26.2969 + name_y_offset;
    let name_y = node.y + 17.9951 + name_y_offset;
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(node.x, sep_y, node.x + node.width, sep_y, 0.0);
    tracker.track_line(node.x, sep_y, node.x + node.width, sep_y);

    // State name text (centered)
    let name_width = font_metrics::text_width(&node.name, "SansSerif", 14.0, false, false);
    let name_x = node.x + (node.width - name_width) / 2.0;
    sg.set_fill_color(font_color);
    sg.svg_text(
        &node.name,
        name_x,
        name_y,
        Some("sans-serif"),
        14.0,
        None,
        None,
        None,
        name_width,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    // Java LimitFinder.drawText: addPoint(x, y-h+1.5), addPoint(x+w, y+h)
    let name_text_h = font_metrics::line_height("SansSerif", 14.0, false, false);
    tracker.track_text(name_x, name_y, name_width, name_text_h);

    // Description lines: each visual line is a separate <text> element
    if !node.description.is_empty() {
        let base_x = node.x + 5.0;
        let first_y = sep_y + 16.1386;
        let visual_lines = expand_description_lines(&node.description);
        let desc_text_h = font_metrics::line_height("SansSerif", DESC_FONT_SIZE, false, false);
        for (i, vline) in visual_lines.iter().enumerate() {
            let x = base_x + vline.tab_count as f64 * TAB_WIDTH;
            let y = first_y + i as f64 * DESC_LINE_HEIGHT;
            render_desc_line(sg, &vline.text, x, y, font_color);
            let text_w =
                font_metrics::text_width(&vline.text, "SansSerif", DESC_FONT_SIZE, false, false);
            tracker.track_text(x, y, text_w, desc_text_h);
        }
    }

    // Close <g>
    sg.push_raw("</g>");
}

/// Composite state: rounded rectangle containing child states
fn render_composite(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
    ent_id_map: &HashMap<String, String>,
) {
    let r = 12.5; // corner radius
    let sep_y = node.y + 26.2969;
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    // 1. Tab header path (filled background, matching Java USymbolFrame)
    //    Rounded top-left and right leading into a flat bottom at the separator line.
    //    Java: path fills the header area from top to separator line.
    let name_tl = font_metrics::text_width(&node.name, "SansSerif", 14.0, false, false);
    // Java tab extends full width; the path traces: top-left rounded → top-right →
    // arc down → right side to sep → left along sep → left side up → arc back.
    sg.push_raw(&format!(
        "<path d=\"M{},{} L{},{} A{r},{r} 0 0 1 {},{} L{},{} L{},{} L{},{} A{r},{r} 0 0 1 {},{}\" fill=\"{bg}\"/>",
        fmt_coord(x + r), fmt_coord(y),           // start: top-left + radius
        fmt_coord(x + w - r), fmt_coord(y),        // top-right before arc
        fmt_coord(x + w), fmt_coord(y + r),        // arc end: right side at radius
        fmt_coord(x + w), fmt_coord(sep_y),        // right side down to separator
        fmt_coord(x), fmt_coord(sep_y),            // bottom-left at separator
        fmt_coord(x), fmt_coord(y + r),            // left side up to radius
        fmt_coord(x + r), fmt_coord(y),            // arc back to start
    ));

    // 2. Outer rounded rect (no fill, border only)
    sg.push_raw(&format!(
        "<rect fill=\"none\" height=\"{}\" rx=\"{r}\" ry=\"{r}\" style=\"stroke:{border};stroke-width:0.5;\" width=\"{}\" x=\"{}\" y=\"{}\"/>",
        fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
    ));
    tracker.track_rect(x, y, w, h);

    // 3. Separator line below the header
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(x, sep_y, x + w, sep_y, 0.0);

    // 4. Composite state name text
    let name_x = x + (w - name_tl) / 2.0;
    let name_y = y + 17.9951;
    sg.set_fill_color(font_color);
    sg.svg_text(
        &node.name,
        name_x,
        name_y,
        Some("sans-serif"),
        14.0,
        None,
        None,
        None,
        name_tl,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    let name_text_h = font_metrics::line_height("SansSerif", 14.0, false, false);
    tracker.track_text(name_x, name_y, name_tl, name_text_h);

    // Open semantic <g> wrapper (no children inside, just for closing)
    let name_escaped = xml_escape(&node.name);
    let ent_id = ent_id_map
        .get(&node.id)
        .cloned()
        .unwrap_or_else(next_ent_id);
    // Note: Java wraps the entity in <g class="entity"> but we output the
    // header elements before the entity <g> to match reference SVG order.

    // Recursively render children with parent name for qualified naming
    for child in &node.children {
        render_state_node_with_parent(
            sg,
            tracker,
            child,
            bg,
            border,
            font_color,
            &HashMap::new(),
            Some(&node.name),
        );
    }

    // Render concurrent region separators (dashed lines)
    // Java: ConcurrentStates renders separator as ULine with stroke(1.5) dashVisible=8 dashSpace=10
    for &sep_y in &node.region_separators {
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(1.5, Some((8.0, 10.0)));
        sg.svg_line(x + 5.0, sep_y, x + w - 7.0, sep_y, 0.0);
    }
}

// ── Transition rendering ────────────────────────────────────────────

fn render_transition(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    transition: &TransitionLayout,
    ent_id_map: &HashMap<String, String>,
) {
    if transition.points.is_empty() && transition.raw_path_d.is_none() {
        return;
    }

    // Resolve entity IDs for attributes
    let from_ent = ent_id_map
        .get(&transition.from_id)
        .cloned()
        .unwrap_or_default();
    let to_ent = ent_id_map
        .get(&transition.to_id)
        .cloned()
        .unwrap_or_default();

    // Build display name for the comment: use ".start." for [*] states
    let from_display = special_transition_endpoint_display(&transition.from_id, true)
        .unwrap_or_else(|| transition.from_id.clone());
    let to_display = special_transition_endpoint_display(&transition.to_id, false)
        .unwrap_or_else(|| transition.to_id.clone());

    // Open semantic <g> wrapper with link attributes
    let from_escaped = xml_escape(&from_display);
    let to_escaped = xml_escape(&to_display);
    let lnk_id = next_lnk_id();
    let mut link_attrs = String::new();
    if !from_ent.is_empty() {
        write!(link_attrs, r#" data-entity-1="{}""#, from_ent).unwrap();
    }
    if !to_ent.is_empty() {
        write!(link_attrs, r#" data-entity-2="{}""#, to_ent).unwrap();
    }
    write!(link_attrs, r#" data-link-type="dependency""#).unwrap();
    if let Some(sl) = transition.source_line {
        write!(link_attrs, r#" data-source-line="{}""#, sl).unwrap();
    }
    write!(link_attrs, r#" id="{}""#, lnk_id).unwrap();
    sg.push_raw(&format!(
        r#"<!--link {} to {}--><g class="link"{link_attrs}>"#,
        from_escaped, to_escaped,
    ));

    // Build path ID: "from-to-to" (Java-style link IDs)
    let path_id = format!("{}-to-{}", from_display, to_display);

    // Path data: prefer raw graphviz Bezier path when available.
    // Java adjusts the edge endpoint by the arrow decoration length (6px)
    // to prevent the path from overlapping the arrowhead polygon.
    if let Some(ref raw_d) = transition.raw_path_d {
        let adjusted_d = adjust_path_endpoint(raw_d, ARROW_DECORATION_LEN);
        sg.push_raw(&format!(
            r#"<path d="{adjusted_d}" fill="none" id="{path_id}" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
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
            r#"<path d="{d}" fill="none" id="{path_id}" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
        ));
    }
    // Track edge path bounds (Java LimitFinder.drawDotPath)
    if !transition.points.is_empty() {
        let p_min_x = transition
            .points
            .iter()
            .map(|p| p.0)
            .fold(f64::INFINITY, f64::min);
        let p_min_y = transition
            .points
            .iter()
            .map(|p| p.1)
            .fold(f64::INFINITY, f64::min);
        let p_max_x = transition
            .points
            .iter()
            .map(|p| p.0)
            .fold(f64::NEG_INFINITY, f64::max);
        let p_max_y = transition
            .points
            .iter()
            .map(|p| p.1)
            .fold(f64::NEG_INFINITY, f64::max);
        tracker.track_path_bounds(p_min_x, p_min_y, p_max_x, p_max_y);
    }

    // Arrowhead polygon: prefer graphviz arrow polygon when available
    if let Some(ref poly_pts) = transition.arrow_polygon {
        if !poly_pts.is_empty() {
            let points_str: String = poly_pts
                .iter()
                .map(|(x, y)| format!("{},{}", fmt_coord(*x), fmt_coord(*y)))
                .collect::<Vec<_>>()
                .join(",");
            sg.push_raw(&format!(
                r#"<polygon fill="{BORDER_COLOR}" points="{points_str}" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
            ));
            // Track polygon bounds (Java LimitFinder.drawUPolygon with HACK_X_FOR_POLYGON)
            let pts: Vec<(f64, f64)> = poly_pts.iter().copied().collect();
            tracker.track_polygon(&pts);
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
            // Java: right wing first (+perp), then left wing (-perp)
            let p2x = tx - ux * back - px * side;
            let p2y = ty - uy * back - py * side;
            let p3x = tx - ux * mid_back;
            let p3y = ty - uy * mid_back;
            let p4x = tx - ux * back + px * side;
            let p4y = ty - uy * back + py * side;

            sg.set_fill_color(BORDER_COLOR);
            sg.set_stroke_color(Some(BORDER_COLOR));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[p1x, p1y, p2x, p2y, p3x, p3y, p4x, p4y, p1x, p1y]);
            tracker.track_polygon(&[(p1x, p1y), (p2x, p2y), (p3x, p3y), (p4x, p4y)]);
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
        // Java: TextBlock is drawn at (labelXY.x + shield, labelXY.y + shield).
        // Text is at +1 x-offset, baseline at +margin + ascent.
        // The label_xy we receive is the TABLE polygon min_xy + MARGIN offset.
        let margin_label = 1.0;
        let text_x = lx + margin_label;
        let text_h = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);
        let text_asc = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
        let text_y = ly + margin_label + text_asc;
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &transition.label,
            text_x,
            text_y,
            Some("sans-serif"),
            FONT_SIZE,
            None,
            None,
            None,
            tl,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        // Java LimitFinder tracks:
        // 1. UEmpty for the label block: addPoint(x, y), addPoint(x+w, y+h)
        // 2. UText inside the block: addPoint(x, y-h+1.5), addPoint(x+w, y+1.5)
        // We track both for accurate viewport computation.
        if let Some((bw, bh)) = transition.label_wh {
            // Track label block as drawEmpty (matches Java SvekEdge label positioning)
            tracker.track_empty(lx, ly, bw, bh);
        }
        tracker.track_text(text_x, text_y, tl, text_h);
    }

    // Close <g>
    sg.push_raw("</g>");
}

/// Adjust the endpoint of an SVG path by moving it back `decoration_len` pixels
/// along the arrow direction.  Java `DotPath.moveEndPoint()` moves both the
/// endpoint (x2,y2) and the last control point (ctrlx2,ctrly2) by the same delta.
///
/// For a cubic Bezier `C x1,y1 x2,y2 x3,y3`, this adjusts both (x2,y2) and (x3,y3).
fn adjust_path_endpoint(d: &str, decoration_len: f64) -> String {
    let parts: Vec<&str> = d.split_whitespace().collect();
    if parts.len() < 2 {
        return d.to_string();
    }

    // Parse all coordinate pairs with their string positions.
    let mut coord_positions: Vec<(usize, usize, f64, f64)> = Vec::new(); // (start, end, x, y)
    let mut search_from = 0;
    for part in &parts {
        let cleaned = part.trim_start_matches(|c: char| c.is_ascii_alphabetic());
        if let Some((x_str, y_str)) = cleaned.split_once(',') {
            if let (Ok(x), Ok(y)) = (x_str.parse::<f64>(), y_str.parse::<f64>()) {
                // Find the coordinate string in the original path
                let coord_str = format!("{},{}", fmt_coord(x), fmt_coord(y));
                if let Some(pos) = d[search_from..].find(&coord_str) {
                    let abs_pos = search_from + pos;
                    coord_positions.push((abs_pos, abs_pos + coord_str.len(), x, y));
                    search_from = abs_pos + coord_str.len();
                } else {
                    coord_positions.push((0, 0, x, y)); // fallback
                }
            }
        }
    }

    if coord_positions.len() < 3 {
        return d.to_string();
    }

    // Compute the direction from the second-to-last control point to the endpoint.
    let n = coord_positions.len();
    let (_, _, x_end, y_end) = coord_positions[n - 1];
    let (_, _, x_ctrl2, _y_ctrl2) = coord_positions[n - 2];
    // Use the first control point to endpoint direction for angle computation
    let (_, _, x_prev, y_prev) = coord_positions[n - 3];
    _ = x_ctrl2; // the 2nd control point, not used for direction
    _ = x_prev;

    // Direction from penultimate ctrl to endpoint
    let (_, _, cx2, cy2) = coord_positions[n - 2];
    let dx = x_end - cx2;
    let dy = y_end - cy2;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return d.to_string();
    }

    // Delta to apply (move back along the arrow direction)
    let move_dx = -decoration_len * dx / len;
    let move_dy = -decoration_len * dy / len;

    // Apply delta to both the second control point and endpoint
    let mut result = d.to_string();
    // Process from end to start so positions remain valid
    let (pos_end, end_end, xe, ye) = coord_positions[n - 1];
    let (pos_ctrl, end_ctrl, xc, yc) = coord_positions[n - 2];
    if pos_end > 0 && pos_ctrl > 0 {
        let new_end = format!("{},{}", fmt_coord(xe + move_dx), fmt_coord(ye + move_dy));
        result.replace_range(pos_end..end_end, &new_end);
        let new_ctrl = format!("{},{}", fmt_coord(xc + move_dx), fmt_coord(yc + move_dy));
        result.replace_range(pos_ctrl..end_ctrl, &new_ctrl);
    }

    result
}

fn special_transition_endpoint_display(id: &str, _is_source: bool) -> Option<String> {
    // After parser split: [*]__start / [*]__end already encode direction
    if id == "[*]__start" {
        return Some("*start*".to_string());
    }
    if id == "[*]__end" {
        return Some("*end*".to_string());
    }
    // Legacy: plain [*] (shouldn't happen after parser fix)
    if id == "[*]" {
        return Some("*start*".to_string());
    }
    // Scoped: [*]__startActive, [*]__endActive, etc.
    if let Some(scope) = id.strip_prefix("[*]__start") {
        return Some(format!("*start*{scope}"));
    }
    if let Some(scope) = id.strip_prefix("[*]__end") {
        return Some(format!("*end*{scope}"));
    }
    let scope = id.strip_prefix("[*]")?;
    Some(format!("*start*{scope}"))
}

// ── Note rendering ──────────────────────────────────────────────────

fn render_note(sg: &mut SvgGraphic, tracker: &mut BoundsTracker, note: &StateNoteLayout) {
    let x = note.x;
    let y = note.y;
    let w = note.width;
    let h = note.height;
    let fold = 10.0;
    let notch_half = 4.0;
    let ent_id = next_ent_id();
    let qualified_name = note.entity_id.as_deref().unwrap_or("GMN");

    sg.push_raw(&format!(
        r#"<g class="entity" data-qualified-name="{}""#,
        xml_escape(qualified_name)
    ));
    if let Some(source_line) = note.source_line {
        sg.push_raw(&format!(r#" data-source-line="{}""#, source_line));
    }
    sg.push_raw(&format!(r#" id="{}">"#, ent_id));

    let body_path = if let Some((ax, ay)) = note.anchor {
        match note.position.as_str() {
            "left" => format!(
                "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{}",
                fmt_coord(x),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(ay + notch_half),
                fmt_coord(ax),
                fmt_coord(ay),
                fmt_coord(x + w),
                fmt_coord(ay - notch_half),
                fmt_coord(x + w),
                fmt_coord(y + fold),
                fmt_coord(x + w - fold),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(y),
            ),
            "top" => format!(
                "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{}",
                fmt_coord(x),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(y + fold),
                fmt_coord(x + w - fold),
                fmt_coord(y),
                fmt_coord(ax + notch_half),
                fmt_coord(y),
                fmt_coord(ax),
                fmt_coord(ay),
                fmt_coord(ax - notch_half),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(y),
            ),
            "bottom" => format!(
                "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{}",
                fmt_coord(x),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(y + h),
                fmt_coord(ax - notch_half),
                fmt_coord(y + h),
                fmt_coord(ax),
                fmt_coord(ay),
                fmt_coord(ax + notch_half),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(y + fold),
                fmt_coord(x + w - fold),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(y),
            ),
            _ => format!(
                "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{} L{},{}",
                fmt_coord(x),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(ay - notch_half),
                fmt_coord(ax),
                fmt_coord(ay),
                fmt_coord(x),
                fmt_coord(ay + notch_half),
                fmt_coord(x),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(y + h),
                fmt_coord(x + w),
                fmt_coord(y + fold),
                fmt_coord(x + w - fold),
                fmt_coord(y),
                fmt_coord(x),
                fmt_coord(y),
            ),
        }
    } else {
        format!(
            "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{}",
            fmt_coord(x),
            fmt_coord(y),
            fmt_coord(x),
            fmt_coord(y + h),
            fmt_coord(x + w),
            fmt_coord(y + h),
            fmt_coord(x + w),
            fmt_coord(y + fold),
            fmt_coord(x + w - fold),
            fmt_coord(y),
            fmt_coord(x),
            fmt_coord(y),
        )
    };

    sg.push_raw(&format!(
        r#"<path d="{}" fill="{}" style="stroke:{};stroke-width:0.5;"/>"#,
        body_path,
        NOTE_BG,
        NOTE_BORDER,
    ));

    // Fold corner path uses stroke-width:1 (Java default for note fold triangle).
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} L{},{}" fill="{}" style="stroke:{};stroke-width:1;"/>"#,
        fmt_coord(x + w - fold),
        fmt_coord(y),
        fmt_coord(x + w - fold),
        fmt_coord(y + fold),
        fmt_coord(x + w),
        fmt_coord(y + fold),
        fmt_coord(x + w - fold),
        fmt_coord(y),
        NOTE_BG,
        NOTE_BORDER,
    ));

    // Track note bounds — notes are drawn as UPath in Java, not UPolygon,
    // so they do NOT get HACK_X_FOR_POLYGON offsets.
    tracker.track_path_bounds(x, y, x + w, y + h);

    let text_x = x + 6.0;
    let text_y = y + 17.0669;
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        text_x,
        text_y,
        font_metrics::line_height("SansSerif", FONT_SIZE, false, false),
        TEXT_COLOR,
        None,
        r#"font-size="13""#,
    );
    sg.push_raw(&tmp);
    sg.push_raw("</g>");
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

struct VisualLine {
    tab_count: usize,
    text: String,
}
fn expand_description_lines(descriptions: &[String]) -> Vec<VisualLine> {
    let mut vl = Vec::new();
    for desc in descriptions {
        for part in split_backslash_n(desc) {
            let (tabs, text) = count_leading_tabs(part);
            let text = if text.is_empty() {
                "\u{00A0}".to_string()
            } else {
                text.to_string()
            };
            vl.push(VisualLine {
                tab_count: tabs,
                text,
            });
        }
    }
    vl
}
fn split_backslash_n(s: &str) -> Vec<&str> {
    let mut r = Vec::new();
    let mut start = 0;
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() && b[i + 1] == b'n' {
            r.push(&s[start..i]);
            start = i + 2;
            i += 2;
        } else {
            i += 1;
        }
    }
    r.push(&s[start..]);
    r
}
fn render_desc_line(sg: &mut SvgGraphic, text: &str, x: f64, y: f64, fc: &str) {
    if text.contains("**") {
        render_desc_line_bold(sg, text, x, y, fc);
        return;
    }
    let (d, tl) = if text == "\u{00A0}" {
        (
            "&#160;".to_string(),
            font_metrics::text_width("\u{00A0}", "SansSerif", DESC_FONT_SIZE, false, false),
        )
    } else {
        (
            xml_escape(text),
            font_metrics::text_width(text, "SansSerif", DESC_FONT_SIZE, false, false),
        )
    };
    sg.push_raw(&format!(r#"<text fill="{fc}" font-family="sans-serif" font-size="12" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{d}</text>"#,
        fmt_coord(tl), fmt_coord(x), fmt_coord(y)));
}
fn render_desc_line_bold(sg: &mut SvgGraphic, text: &str, x: f64, y: f64, fc: &str) {
    let mut cx = x;
    let mut ib = false;
    for part in text.split("**") {
        if part.is_empty() {
            ib = !ib;
            continue;
        }
        let e = xml_escape(part);
        let tl = font_metrics::text_width(part, "SansSerif", DESC_FONT_SIZE, ib, false);
        if ib {
            sg.push_raw(&format!(r#"<text fill="{fc}" font-family="sans-serif" font-size="12" font-weight="700" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{e}</text>"#, fmt_coord(tl), fmt_coord(cx), fmt_coord(y)));
        } else {
            sg.push_raw(&format!(r#"<text fill="{fc}" font-family="sans-serif" font-size="12" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{e}</text>"#, fmt_coord(tl), fmt_coord(cx), fmt_coord(y)));
        }
        cx += tl;
        ib = !ib;
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
        StateDiagram {
            states: vec![],
            transitions: vec![],
            notes: vec![],
            direction: Default::default(),
        }
    }

    fn empty_layout() -> StateLayout {
        StateLayout {
            width: 300.0,
            height: 200.0,
            state_layouts: vec![],
            transition_layouts: vec![],
            note_layouts: vec![],
            move_delta: (7.0, 7.0),
            lf_span: (300.0, 200.0),
        }
    }

    fn make_initial(x: f64, y: f64) -> StateNodeLayout {
        StateNodeLayout {
            id: "[*]_initial".to_string(),
            name: String::new(),
            x,
            y,
            width: 20.0,
            height: 20.0,
            description: vec![],
            stereotype: None,
            is_initial: true,
            is_final: false,
            is_composite: false,
            children: vec![],
            kind: crate::model::state::StateKind::default(),
            region_separators: Vec::new(),
            source_line: None,
        }
    }

    fn make_final(x: f64, y: f64) -> StateNodeLayout {
        StateNodeLayout {
            id: "[*]_final".to_string(),
            name: String::new(),
            x,
            y,
            width: 22.0,
            height: 22.0,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: true,
            is_composite: false,
            source_line: None,
            children: vec![],
            kind: crate::model::state::StateKind::default(),
            region_separators: Vec::new(),
        }
    }

    fn make_simple(id: &str, name: &str, x: f64, y: f64, w: f64, h: f64) -> StateNodeLayout {
        StateNodeLayout {
            id: id.to_string(),
            name: name.to_string(),
            x,
            y,
            width: w,
            height: h,
            source_line: None,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: false,
            is_composite: false,
            children: vec![],
            kind: crate::model::state::StateKind::default(),
            region_separators: Vec::new(),
        }
    }

    #[test]
    fn test_empty_diagram() {
        let diagram = empty_diagram();
        let layout = empty_layout();
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
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
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"rx="10""#),
            "initial ellipse must have rx=10"
        );
        assert!(
            svg.contains(r#"ry="10""#),
            "initial ellipse must have ry=10"
        );
        assert!(
            svg.contains(&format!(r#"fill="{INITIAL_FILL}""#)),
            "initial ellipse must be filled"
        );
        assert_eq!(
            svg.matches("<ellipse").count(),
            1,
            "initial state must produce exactly one ellipse"
        );
        assert!(
            svg.contains(r#"class="start_entity""#),
            "initial state must be wrapped in start_entity group"
        );
    }

    #[test]
    fn test_final_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(make_final(90.0, 80.0));
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert_eq!(
            svg.matches("<ellipse").count(),
            2,
            "final state must produce two ellipses"
        );
        assert!(svg.contains(r#"rx="11""#), "final outer ring must have rx=11");
        assert!(svg.contains(r#"rx="6""#), "final inner ellipse must have rx=6");
        assert!(
            svg.contains("stroke-width:1;"),
            "outer ring must have stroke-width=1"
        );
    }

    #[test]
    fn test_simple_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout
            .state_layouts
            .push(make_simple("Idle", "Idle", 30.0, 40.0, 100.0, 40.0));
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"rx="12.5""#),
            "state must have rounded corners rx=12.5"
        );
        assert!(
            svg.contains(r#"ry="12.5""#),
            "state must have rounded corners ry=12.5"
        );
        assert!(
            svg.contains(r##"fill="#F1F1F1""##),
            "state must use default theme state_bg fill"
        );
        assert!(svg.contains("Idle"), "state name must appear in SVG");
        assert!(
            svg.contains(r#"class="entity""#),
            "state must be wrapped in entity group"
        );
        assert!(
            svg.contains("stroke-width:0.5;"),
            "state border must have stroke-width:0.5"
        );
    }

    #[test]
    fn test_state_with_description() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("Active", "Active", 20.0, 30.0, 140.0, 80.0);
        node.description = vec![
            "entry / start timer".to_string(),
            "exit / stop timer".to_string(),
        ];
        layout.state_layouts.push(node);
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Active"), "state name must appear");
        assert!(
            svg.contains("entry / start timer"),
            "first description line must appear"
        );
        assert!(
            svg.contains("exit / stop timer"),
            "second description line must appear"
        );
        assert!(
            svg.contains("<line"),
            "separator line must exist between name and description"
        );
    }

    #[test]
    fn test_state_with_stereotype() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("InputPin", "InputPin", 20.0, 30.0, 120.0, 50.0);
        node.stereotype = Some("inputPin".to_string());
        layout.state_layouts.push(node);
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("InputPin"), "state name must appear");
        assert!(
            svg.contains("&#171;inputPin&#187;"),
            "stereotype must appear with guillemets"
        );
        assert!(
            svg.contains("font-style=\"italic\""),
            "stereotype must be italic"
        );
    }

    #[test]
    fn test_composite_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let child = make_simple("Inner", "Inner", 50.0, 80.0, 80.0, 36.0);
        let composite = StateNodeLayout {
            id: "Outer".to_string(),
            name: "Outer".to_string(),
            x: 20.0,
            y: 30.0,
            width: 200.0,
            height: 120.0,
            description: vec![],
            stereotype: None,
            source_line: None,
            is_initial: false,
            is_final: false,
            is_composite: true,
            children: vec![child],
            kind: crate::model::state::StateKind::default(),
            region_separators: Vec::new(),
        };
        layout.state_layouts.push(composite);
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Outer"), "composite name must appear");
        assert!(svg.contains("Inner"), "child state name must appear");
        let rect_count = svg.matches("<rect").count();
        assert!(
            rect_count >= 2,
            "composite must produce at least 2 rects, got {rect_count}"
        );
        assert!(
            svg.contains("<line"),
            "composite must have separator line below header"
        );
    }

    #[test]
    fn test_transition_with_arrow() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(),
            to_id: "B".to_string(),
            label: String::new(),
            points: vec![(100.0, 50.0), (100.0, 120.0)],
            raw_path_d: None,
            arrow_polygon: None,
            label_xy: None,
            label_wh: None,
            source_line: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polygon"),
            "transition must have inline polygon arrowhead"
        );
        assert!(
            svg.contains("stroke:#181818"),
            "transition must use BORDER_COLOR in style"
        );
        assert!(svg.contains("<path "), "transition must use <path>");
        assert!(
            svg.contains(r#"class="link""#),
            "transition must be in link group"
        );
    }

    #[test]
    fn test_transition_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "Idle".to_string(),
            to_id: "Active".to_string(),
            label: "start".to_string(),
            points: vec![(80.0, 40.0), (80.0, 100.0)],
            source_line: None,
            raw_path_d: None,
            arrow_polygon: None,
            label_xy: None,
            label_wh: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("start"), "transition label must appear in SVG");
        assert!(
            svg.contains(r#"lengthAdjust="spacing""#),
            "label must have lengthAdjust"
        );
    }

    #[test]
    fn test_polyline_transition() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(),
            to_id: "B".to_string(),
            label: String::new(),
            source_line: None,
            points: vec![(50.0, 20.0), (50.0, 50.0), (100.0, 50.0), (100.0, 80.0)],
            raw_path_d: None,
            arrow_polygon: None,
            label_xy: None,
            label_wh: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<path"),
            "multi-point transition must use <path>"
        );
        assert!(
            svg.contains("<polygon"),
            "multi-point transition must have inline polygon arrowhead"
        );
    }

    #[test]
    fn test_note_rendering() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.note_layouts.push(StateNoteLayout {
            x: 10.0,
            y: 20.0,
            width: 120.0,
            height: 40.0,
            text: "important note".to_string(),
            position: "right".to_string(),
            target: None,
            entity_id: Some("GMN2".to_string()),
            source_line: Some(1),
            anchor: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(&format!(r#"fill="{NOTE_BG}""#)),
            "note must use yellow background"
        );
        assert!(svg.contains("important note"), "note text must appear");
        assert!(
            svg.matches("<path").count() >= 2,
            "note must use <path> for body and fold corner"
        );
    }

    #[test]
    fn test_multiline_note() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.note_layouts.push(StateNoteLayout {
            x: 10.0,
            y: 20.0,
            width: 120.0,
            height: 60.0,
            text: "line one\nline two".to_string(),
            position: "right".to_string(),
            target: None,
            entity_id: Some("GMN2".to_string()),
            source_line: Some(1),
            anchor: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // Java renders each line as a separate <text> element (no tspan)
        assert!(!svg.contains("<tspan"), "multiline note must not use tspan");
        assert!(svg.contains("line one"), "first line must appear");
        assert!(svg.contains("line two"), "second line must appear");
        // Two lines must produce two separate <text> elements for the note body
        let text_count =
            svg.matches(">line one</text>").count() + svg.matches(">line two</text>").count();
        assert_eq!(
            text_count, 2,
            "two lines must produce two separate text elements"
        );
    }

    #[test]
    fn test_xml_escaping() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("test", "A & B < C", 10.0, 10.0, 120.0, 40.0);
        node.description = vec!["x > y & z".to_string()];
        layout.state_layouts.push(node);
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("A &amp; B &lt; C"),
            "state name must be XML-escaped"
        );
        assert!(
            svg.contains("x &gt; y &amp; z"),
            "description must be XML-escaped"
        );
    }

    #[test]
    fn test_full_svg_structure() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout.state_layouts.push(make_initial(180.0, 10.0));
        layout
            .state_layouts
            .push(make_simple("Running", "Running", 130.0, 50.0, 120.0, 40.0));
        layout.state_layouts.push(make_final(180.0, 120.0));
        layout.transition_layouts.push(TransitionLayout {
            from_id: "[*]_initial".to_string(),
            to_id: "Running".to_string(),
            label: String::new(),
            points: vec![(190.0, 30.0), (190.0, 50.0)],
            raw_path_d: None,
            arrow_polygon: None,
            label_xy: None,
            label_wh: None,
            source_line: None,
        });
        layout.transition_layouts.push(TransitionLayout {
            from_id: "Running".to_string(),
            to_id: "[*]_final".to_string(),
            label: "done".to_string(),
            points: vec![(190.0, 90.0), (190.0, 120.0)],
            raw_path_d: None,
            arrow_polygon: None,
            label_xy: None,
            label_wh: None,
            source_line: None,
        });
        let (svg, raw_dim) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.contains("</svg>"), "SVG must end with </svg>");
        // Viewport is computed from BoundsTracker span + CANVAS_DELTA(15) + DOC_MARGIN(5)
        assert!(raw_dim.is_some(), "raw_body_dim must be present");
        assert!(svg.contains("viewBox="), "must have viewBox");
        assert!(svg.contains("<defs/>"), "must have <defs/>");
        assert_eq!(svg.matches("<ellipse").count(), 3, "3 ellipses expected (1 initial + 2 final)");
        assert_eq!(svg.matches("<circle").count(), 0, "0 circles expected");
        assert_eq!(svg.matches("<rect").count(), 1, "1 rect expected");
        assert_eq!(
            svg.matches(r#"class="link""#).count(),
            2,
            "2 transitions with link groups expected"
        );
        assert!(svg.contains("done"), "transition label 'done' must appear");
    }

    #[test]
    fn test_empty_transition_points() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(),
            to_id: "B".to_string(),
            label: "skip".to_string(),
            points: vec![],
            raw_path_d: None,
            arrow_polygon: None,
            label_xy: None,
            label_wh: None,
            source_line: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            !svg.contains("<path"),
            "empty points should not produce a path"
        );
        assert!(
            !svg.contains("skip"),
            "empty points should not produce a label"
        );
    }

    #[test]
    fn test_fork_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "fork1".to_string(),
            name: "fork1".to_string(),
            source_line: None,
            x: 30.0,
            y: 40.0,
            width: 80.0,
            height: 6.0,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: false,
            is_composite: false,
            children: vec![],
            kind: StateKind::Fork,
            region_separators: Vec::new(),
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "fork bar must produce a rect");
        assert!(
            svg.contains(&format!(r#"fill="{INITIAL_FILL}""#)),
            "fork bar must be filled"
        );
        assert!(
            svg.contains(r#"rx="2""#),
            "fork bar must have minimal rounding"
        );
    }

    #[test]
    fn test_join_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            source_line: None,
            id: "join1".to_string(),
            name: "join1".to_string(),
            x: 30.0,
            y: 40.0,
            width: 80.0,
            height: 6.0,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: false,
            is_composite: false,
            children: vec![],
            kind: StateKind::Join,
            region_separators: Vec::new(),
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "join bar must produce a rect");
    }

    #[test]
    fn test_choice_diamond() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "choice1".to_string(),
            name: "choice1".to_string(),
            x: 50.0,
            y: 50.0,
            width: 20.0,
            height: 20.0,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: false,
            is_composite: false,
            children: vec![],
            kind: StateKind::Choice,
            region_separators: Vec::new(),
            source_line: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polygon"),
            "choice must produce a polygon (diamond)"
        );
    }

    #[test]
    fn test_history_circle() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "Active[H]".to_string(),
            name: "Active[H]".to_string(),
            x: 50.0,
            y: 50.0,
            width: 24.0,
            height: 24.0,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: false,
            is_composite: false,
            children: vec![],
            kind: StateKind::History,
            region_separators: Vec::new(),
            source_line: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<circle"), "history must produce a circle");
        assert!(svg.contains(">H<"), "history must contain 'H' text");
    }

    #[test]
    fn test_deep_history_circle() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "Active[H*]".to_string(),
            name: "Active[H*]".to_string(),
            x: 50.0,
            y: 50.0,
            width: 24.0,
            height: 24.0,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: false,
            is_composite: false,
            children: vec![],
            kind: StateKind::DeepHistory,
            region_separators: Vec::new(),
            source_line: None,
        });
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<circle"),
            "deep history must produce a circle"
        );
        assert!(svg.contains(">H*<"), "deep history must contain 'H*' text");
    }

    #[test]
    fn test_concurrent_separator() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let child1 = make_simple("Sub1", "Sub1", 40.0, 60.0, 80.0, 36.0);
        let child2 = make_simple("Sub3", "Sub3", 40.0, 140.0, 80.0, 36.0);
        let composite = StateNodeLayout {
            id: "Active".to_string(),
            name: "Active".to_string(),
            x: 20.0,
            y: 30.0,
            width: 200.0,
            height: 180.0,
            description: vec![],
            stereotype: None,
            is_initial: false,
            is_final: false,
            is_composite: true,
            children: vec![child1, child2],
            kind: StateKind::Normal,
            region_separators: vec![110.0],
            source_line: None,
        };
        layout.state_layouts.push(composite);
        let (svg, _) =
            render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("stroke-dasharray"),
            "concurrent separator must be dashed"
        );
    }
}
