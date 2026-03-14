use std::fmt::Write;

use crate::layout::state::{StateLayout, StateNodeLayout, StateNoteLayout, TransitionLayout};
use crate::model::state::{StateDiagram, StateKind};
use crate::render::svg::xml_escape;
use crate::render::svg::write_svg_root;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// ── Style constants (PlantUML rose theme) ───────────────────────────

const FONT_SIZE: f64 = 13.0;
const LINE_HEIGHT: f64 = 16.0;
const STATE_BG: &str = "#F1F1F1";
const STATE_BORDER: &str = "#181818";
const INITIAL_FILL: &str = "#000000";
const FINAL_OUTER: &str = "#000000";
const FINAL_INNER: &str = "#000000";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";

// ── Public entry point ──────────────────────────────────────────────

/// Render a state diagram to SVG.
pub fn render_state(
    _diagram: &StateDiagram,
    layout: &StateLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // SVG header
    write_svg_root(&mut buf, layout.width, layout.height, "STATE");
    buf.push_str("<defs/><g>");

    // Defs: arrow marker
    write_defs(&mut buf);

    let state_bg = skin.background_color("state", STATE_BG);
    let state_border = skin.border_color("state", STATE_BORDER);
    let state_font = skin.font_color("state", TEXT_FILL);

    // States (including composite with children)
    for state in &layout.state_layouts {
        render_state_node(&mut buf, state, state_bg, state_border, state_font);
    }

    // Transitions
    for transition in &layout.transition_layouts {
        render_transition(&mut buf, transition);
    }

    // Notes
    for note in &layout.note_layouts {
        render_note(&mut buf, note);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── Defs ────────────────────────────────────────────────────────────

fn write_defs(buf: &mut String) {
    buf.push_str("<defs>\n");
    write!(
        buf,
        concat!(
            r#"<marker id="state-arrow" viewBox="0 0 10 10" refX="10" refY="5""#,
            r#" markerWidth="8" markerHeight="8" orient="auto-start-reverse">"#,
            r#"<path d="M 0 0 L 10 5 L 0 10 Z" fill="{}" stroke="none"/>"#,
            r#"</marker>"#,
        ),
        EDGE_COLOR,
    )
    .unwrap();
    buf.push('\n');
    buf.push_str("</defs>\n");
}

// ── State node rendering ────────────────────────────────────────────

fn render_state_node(
    buf: &mut String,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    // Dispatch by pseudo-state kind first, then by initial/final/composite/simple
    match &node.kind {
        StateKind::Fork | StateKind::Join => {
            render_fork_join(buf, node);
        }
        StateKind::Choice => {
            render_choice(buf, node, border);
        }
        StateKind::History => {
            render_history(buf, node, border, font_color, false);
        }
        StateKind::DeepHistory => {
            render_history(buf, node, border, font_color, true);
        }
        StateKind::End => {
            render_final(buf, node);
        }
        StateKind::EntryPoint => {
            render_initial(buf, node);
        }
        StateKind::ExitPoint => {
            render_exit_point(buf, node, border);
        }
        StateKind::Normal => {
            if node.is_initial {
                render_initial(buf, node);
            } else if node.is_final {
                render_final(buf, node);
            } else if node.is_composite {
                render_composite(buf, node, bg, border, font_color);
            } else {
                render_simple(buf, node, bg, border, font_color);
            }
        }
    }
}

/// Initial state: filled black circle, r=10
fn render_initial(buf: &mut String, node: &StateNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="{INITIAL_FILL}" r="10"/>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Final state: double circle (outer ring + inner filled)
fn render_final(buf: &mut String, node: &StateNodeLayout) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="none" r="11" style="stroke:{FINAL_OUTER};stroke-width:2;"/>"#,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="{FINAL_INNER}" r="7"/>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Fork/Join bar: filled black horizontal rectangle
fn render_fork_join(buf: &mut String, node: &StateNodeLayout) {
    write!(
        buf,
        r#"<rect fill="{INITIAL_FILL}" height="{h:.1}" rx="2" ry="2" stroke="none" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');
}

/// Choice diamond: small rotated square
fn render_choice(buf: &mut String, node: &StateNodeLayout, border: &str) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let half = node.width / 2.0;
    // Diamond points: top, right, bottom, left
    write!(
        buf,
        r##"<polygon fill="#F1F1F1" points="{cx:.1},{top:.1} {right:.1},{cy:.1} {cx:.1},{bottom:.1} {left:.1},{cy:.1}" style="stroke:{border};stroke-width:1.5;"/>"##,
        top = cy - half,
        right = cx + half,
        bottom = cy + half,
        left = cx - half,
    )
    .unwrap();
    buf.push('\n');
}

/// History circle: small circle with "H" (or "H*") text inside
fn render_history(
    buf: &mut String,
    node: &StateNodeLayout,
    border: &str,
    font_color: &str,
    deep: bool,
) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let r = node.width / 2.0;
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="none" r="{r:.1}" style="stroke:{border};stroke-width:1.5;"/>"#,
    )
    .unwrap();
    buf.push('\n');
    let label = if deep { "H*" } else { "H" };
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{ty:.1}">{label}</text>"#,
        ty = cy + FONT_SIZE * 0.35,
    )
    .unwrap();
    buf.push('\n');
}

/// Exit point: circle with X inside
fn render_exit_point(buf: &mut String, node: &StateNodeLayout, border: &str) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + node.height / 2.0;
    let r = node.width / 2.0;
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="none" r="{r:.1}" style="stroke:{border};stroke-width:1.5;"/>"#,
    )
    .unwrap();
    buf.push('\n');
    // X cross inside
    let d = r * 0.5;
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        x1 = cx - d,
        y1 = cy - d,
        x2 = cx + d,
        y2 = cy + d,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        x1 = cx + d,
        y1 = cy - d,
        x2 = cx - d,
        y2 = cy + d,
    )
    .unwrap();
    buf.push('\n');
}

/// Simple state: rounded rectangle with name + optional description
fn render_simple(
    buf: &mut String,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    // Background rounded rectangle
    write!(
        buf,
        r#"<rect fill="{bg}" height="{h:.1}" rx="10" ry="10" style="stroke:{border};stroke-width:1.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

    let cx = node.x + node.width / 2.0;

    // Stereotype (shown above the name in smaller text)
    let mut name_y_offset = 0.0;
    if let Some(ref stereotype) = node.stereotype {
        let stereo_text = format!("\u{00AB}{stereotype}\u{00BB}");
        let escaped = xml_escape(&stereo_text);
        let stereo_y = node.y + FONT_SIZE + 4.0;
        write!(
            buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{fs:.0}" font-style="italic" text-anchor="middle" x="{cx:.1}" y="{sy:.1}">{escaped}</text>"#,
            sy = stereo_y,
            fs = FONT_SIZE - 2.0,
        )
        .unwrap();
        buf.push('\n');
        name_y_offset = LINE_HEIGHT;
    }

    // State name centered in header area
    let has_desc = !node.description.is_empty();
    let name_y = if has_desc {
        // Name in the upper portion when there is a description
        node.y + FONT_SIZE + 4.0 + name_y_offset
    } else {
        // Name vertically centered when no description
        node.y + node.height / 2.0 + FONT_SIZE * 0.35 + name_y_offset
    };
    let name_escaped = xml_escape(&node.name);
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{name_escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');

    // Separator line + description lines
    if has_desc {
        let sep_y = name_y + 6.0;
        write!(
            buf,
            r#"<line style="stroke:{border};" x1="{x1:.1}" x2="{x2:.1}" y1="{sy:.1}" y2="{sy:.1}"/>"#,
            x1 = node.x,
            sy = sep_y,
            x2 = node.x + node.width,
        )
        .unwrap();
        buf.push('\n');

        let text_x = node.x + 8.0;
        let desc_text = node.description.join("\n");
        render_creole_text(
            buf,
            &desc_text,
            text_x,
            sep_y + LINE_HEIGHT,
            LINE_HEIGHT,
            font_color,
            None,
            r#"font-size="12""#,
        );
    }
}

/// Composite state: rounded rectangle containing child states
fn render_composite(
    buf: &mut String,
    node: &StateNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    // Outer rounded rectangle
    write!(
        buf,
        r#"<rect fill="{bg}" height="{h:.1}" rx="10" ry="10" style="stroke:{border};stroke-width:1.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

    // Composite state name at the top
    let cx = node.x + node.width / 2.0;
    let name_y = node.y + FONT_SIZE + 4.0;
    let name_escaped = xml_escape(&node.name);
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{name_escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');

    // Separator line below the header
    let sep_y = name_y + 6.0;
    write!(
        buf,
        r#"<line style="stroke:{border};" x1="{x1:.1}" x2="{x2:.1}" y1="{sy:.1}" y2="{sy:.1}"/>"#,
        x1 = node.x,
        sy = sep_y,
        x2 = node.x + node.width,
    )
    .unwrap();
    buf.push('\n');

    // Recursively render children
    for child in &node.children {
        render_state_node(buf, child, bg, border, font_color);
    }

    // Render concurrent region separators (dashed lines)
    for &sep_y in &node.region_separators {
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-dasharray:6,4;" x1="{x1:.1}" x2="{x2:.1}" y1="{sy:.1}" y2="{sy:.1}"/>"#,
            x1 = node.x + 4.0,
            sy = sep_y,
            x2 = node.x + node.width - 4.0,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Transition rendering ────────────────────────────────────────────

fn render_transition(buf: &mut String, transition: &TransitionLayout) {
    if transition.points.is_empty() {
        return;
    }

    if transition.points.len() == 2 {
        let (x1, y1) = transition.points[0];
        let (x2, y2) = transition.points[1];
        write!(
            buf,
            r#"<line marker-end="url(#state-arrow)" style="stroke:{EDGE_COLOR};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        )
        .unwrap();
        buf.push('\n');
    } else {
        let points_str: String = transition
            .points
            .iter()
            .map(|(px, py)| format!("{px:.1},{py:.1}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            buf,
            r#"<polyline fill="none" marker-end="url(#state-arrow)" points="{points_str}" style="stroke:{EDGE_COLOR};stroke-width:1;"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    // Label centered near midpoint
    if !transition.label.is_empty() {
        let mid = transition.points.len() / 2;
        let (mx, my) = transition.points[mid];
        let escaped = xml_escape(&transition.label);
        write!(
            buf,
            r#"<text fill="{TEXT_FILL}" font-family="sans-serif" font-size="{FONT_SIZE}" text-anchor="middle" x="{mx:.1}" y="{my:.1}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Note rendering ──────────────────────────────────────────────────

fn render_note(buf: &mut String, note: &StateNoteLayout) {
    let x = note.x;
    let y = note.y;
    let w = note.width;
    let h = note.height;
    let fold = 8.0;

    // Note body polygon (top-left, pre-fold top-right, fold corner, bottom-right, bottom-left)
    write!(
        buf,
        r#"<polygon fill="{NOTE_BG}" points="{x:.1},{y:.1} {xf:.1},{y:.1} {xw:.1},{yf:.1} {xw:.1},{yh:.1} {x:.1},{yh:.1}" style="stroke:{NOTE_BORDER};"/>"#,
        xf = x + w - fold,
        xw = x + w,
        yf = y + fold,
        yh = y + h,
    )
    .unwrap();
    buf.push('\n');

    // Fold lines (vertical + horizontal)
    write!(
        buf,
        r#"<line style="stroke:{NOTE_BORDER};" x1="{xf:.1}" x2="{xf:.1}" y1="{y:.1}" y2="{yf:.1}"/>"#,
        xf = x + w - fold,
        yf = y + fold,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<line style="stroke:{NOTE_BORDER};" x1="{xf:.1}" x2="{xw:.1}" y1="{yf:.1}" y2="{yf:.1}"/>"#,
        xf = x + w - fold,
        yf = y + fold,
        xw = x + w,
    )
    .unwrap();
    buf.push('\n');

    let text_x = x + 6.0;
    let text_y = y + fold + FONT_SIZE;
    render_creole_text(
        buf,
        &note.text,
        text_x,
        text_y,
        LINE_HEIGHT,
        TEXT_FILL,
        None,
        r#"font-size="13""#,
    );
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

    // ── Test: empty diagram ─────────────────────────────────────────

    #[test]
    fn test_empty_diagram() {
        let diagram = empty_diagram();
        let layout = empty_layout();
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(
            svg.contains("state-arrow"),
            "must define state-arrow marker"
        );
        // No nodes or edges
        assert!(!svg.contains("<circle"), "empty diagram has no circles");
        assert!(!svg.contains("<rect"), "empty diagram has no rects");
    }

    // ── Test: initial state ─────────────────────────────────────────

    #[test]
    fn test_initial_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(make_initial(90.0, 10.0));
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"r="10""#), "initial circle must have r=10");
        assert!(
            svg.contains(&format!(r#"fill="{INITIAL_FILL}""#)),
            "initial circle must be black filled"
        );
        assert_eq!(
            svg.matches("<circle").count(),
            1,
            "initial state must produce exactly one circle"
        );
    }

    // ── Test: final state ───────────────────────────────────────────

    #[test]
    fn test_final_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(make_final(90.0, 80.0));
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert_eq!(
            svg.matches("<circle").count(),
            2,
            "final state must produce two circles"
        );
        assert!(svg.contains(r#"r="11""#), "final outer ring must have r=11");
        assert!(svg.contains(r#"r="7""#), "final inner circle must have r=7");
        assert!(
            svg.contains("stroke-width:2;"),
            "outer ring must have stroke-width=2"
        );
    }

    // ── Test: simple state ──────────────────────────────────────────

    #[test]
    fn test_simple_state() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout
            .state_layouts
            .push(make_simple("Idle", "Idle", 30.0, 40.0, 100.0, 40.0));
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"rx="10""#),
            "state must have rounded corners"
        );
        assert!(
            svg.contains(r#"ry="10""#),
            "state must have rounded corners"
        );
        assert!(
            svg.contains(r##"fill="#F1F1F1""##),
            "state must use default theme state_bg fill"
        );
        assert!(svg.contains("Idle"), "state name must appear in SVG");
        assert!(
            svg.contains(r#"text-anchor="middle""#),
            "name must be centered"
        );
    }

    // ── Test: state with description ────────────────────────────────

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
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Active"), "state name must appear");
        assert!(
            svg.contains("entry / start timer"),
            "first description line must appear"
        );
        assert!(
            svg.contains("exit / stop timer"),
            "second description line must appear"
        );
        // Separator line between name and description
        assert!(
            svg.contains("<line"),
            "separator line must exist between name and description"
        );
    }

    // ── Test: state with stereotype ─────────────────────────────────

    #[test]
    fn test_state_with_stereotype() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("InputPin", "InputPin", 20.0, 30.0, 120.0, 50.0);
        node.stereotype = Some("inputPin".to_string());
        layout.state_layouts.push(node);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("InputPin"), "state name must appear");
        assert!(
            svg.contains("\u{00AB}inputPin\u{00BB}"),
            "stereotype must appear with guillemets"
        );
        assert!(
            svg.contains("font-style=\"italic\""),
            "stereotype must be italic"
        );
    }

    // ── Test: composite state ───────────────────────────────────────

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
            is_initial: false,
            is_final: false,
            is_composite: true,
            children: vec![child],
            kind: crate::model::state::StateKind::default(),
            region_separators: Vec::new(),
        };
        layout.state_layouts.push(composite);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Outer"), "composite name must appear");
        assert!(svg.contains("Inner"), "child state name must appear");
        // At least 2 rects: composite outer + child inner
        let rect_count = svg.matches("<rect").count();
        assert!(
            rect_count >= 2,
            "composite must produce at least 2 rects, got {rect_count}"
        );
        // Separator line below composite header
        assert!(
            svg.contains("<line"),
            "composite must have separator line below header"
        );
    }

    // ── Test: transition with arrow ─────────────────────────────────

    #[test]
    fn test_transition_with_arrow() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(),
            to_id: "B".to_string(),
            label: String::new(),
            points: vec![(100.0, 50.0), (100.0, 120.0)],
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"marker-end="url(#state-arrow)""#),
            "transition must reference state-arrow marker"
        );
        assert!(
            svg.contains("stroke:#181818"),
            "transition must use EDGE_COLOR in style"
        );
        assert!(
            svg.contains("<line "),
            "2-point transition must use <line>"
        );
    }

    // ── Test: transition with label ─────────────────────────────────

    #[test]
    fn test_transition_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "Idle".to_string(),
            to_id: "Active".to_string(),
            label: "start".to_string(),
            points: vec![(80.0, 40.0), (80.0, 100.0)],
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("start"), "transition label must appear in SVG");
        assert!(
            svg.contains(r#"text-anchor="middle""#),
            "label must be centered"
        );
    }

    // ── Test: polyline transition ───────────────────────────────────

    #[test]
    fn test_polyline_transition() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(),
            to_id: "B".to_string(),
            label: String::new(),
            points: vec![(50.0, 20.0), (50.0, 50.0), (100.0, 50.0), (100.0, 80.0)],
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polyline"),
            "multi-point transition must use <polyline>"
        );
        assert!(
            svg.contains(r#"marker-end="url(#state-arrow)""#),
            "polyline must also have arrow marker"
        );
    }

    // ── Test: note rendering ────────────────────────────────────────

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
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(&format!(r#"fill="{NOTE_BG}""#)),
            "note must use yellow background"
        );
        assert!(svg.contains("important note"), "note text must appear");
        assert!(
            svg.contains("<polygon"),
            "note body must be a polygon with folded corner"
        );
        // Folded corner produces 2 fold lines
        let line_count = svg.matches("<line").count();
        assert!(
            line_count >= 2,
            "note must have at least 2 fold lines, got {line_count}"
        );
    }

    // ── Test: multiline note ────────────────────────────────────────

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
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<tspan"), "multiline note must use tspan");
        assert!(svg.contains("line one"), "first line must appear");
        assert!(svg.contains("line two"), "second line must appear");
        assert_eq!(
            svg.matches("<tspan").count(),
            2,
            "two lines must produce two tspan elements"
        );
    }

    // ── Test: XML escaping ──────────────────────────────────────────

    #[test]
    fn test_xml_escaping() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        let mut node = make_simple("test", "A & B < C", 10.0, 10.0, 120.0, 40.0);
        node.description = vec!["x > y & z".to_string()];
        layout.state_layouts.push(node);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("A &amp; B &lt; C"),
            "state name must be XML-escaped"
        );
        assert!(
            svg.contains("x &gt; y &amp; z"),
            "description must be XML-escaped"
        );
    }

    // ── Test: full SVG validity ─────────────────────────────────────

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
        });
        layout.transition_layouts.push(TransitionLayout {
            from_id: "Running".to_string(),
            to_id: "[*]_final".to_string(),
            label: "done".to_string(),
            points: vec![(190.0, 90.0), (190.0, 120.0)],
        });

        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");

        // Basic SVG structure
        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.contains("</svg>"), "SVG must end with </svg>");
        assert!(
            svg.contains("viewBox=\"0 0 400 300\""),
            "viewBox must match layout dimensions"
        );
        assert!(svg.contains("width=\"400px\""), "width must match layout");
        assert!(svg.contains("height=\"300px\""), "height must match layout");

        // Defs
        assert!(svg.contains("<defs>"), "must have <defs>");
        assert!(svg.contains("</defs>"), "must have </defs>");
        assert!(
            svg.contains("state-arrow"),
            "must define state-arrow marker"
        );

        // Nodes: 1 initial (1 circle) + 1 simple (1 rect) + 1 final (2 circles) = 3 circles, 1 rect
        assert_eq!(svg.matches("<circle").count(), 3, "3 circles expected");
        assert_eq!(svg.matches("<rect").count(), 1, "1 rect expected");

        // Transitions
        assert_eq!(
            svg.matches(r#"marker-end="url(#state-arrow)""#).count(),
            2,
            "2 transitions with arrows expected"
        );

        // Label
        assert!(svg.contains("done"), "transition label 'done' must appear");
    }

    // ── Test: empty transition points ───────────────────────────────

    #[test]
    fn test_empty_transition_points() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.transition_layouts.push(TransitionLayout {
            from_id: "A".to_string(),
            to_id: "B".to_string(),
            label: "skip".to_string(),
            points: vec![],
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // Empty points should produce no line/polyline and no label
        assert!(
            !svg.contains("<line x1="),
            "empty points should not produce a line"
        );
        assert!(
            !svg.contains("<polyline"),
            "empty points should not produce a polyline"
        );
        assert!(
            !svg.contains("skip"),
            "empty points should not produce a label"
        );
    }

    // ── Test: fork/join bar ────────────────────────────────────────

    #[test]
    fn test_fork_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
            id: "fork1".to_string(),
            name: "fork1".to_string(),
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
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        // Fork bar must produce a filled black rect
        assert!(svg.contains("<rect"), "fork bar must produce a rect");
        assert!(
            svg.contains(&format!(r#"fill="{INITIAL_FILL}""#)),
            "fork bar must be black filled"
        );
        // Should NOT have rounded corners with rx="10"
        assert!(
            svg.contains(r#"rx="2""#),
            "fork bar must have minimal rounding"
        );
    }

    // ── Test: join bar ────────────────────────────────────────────

    #[test]
    fn test_join_bar() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.state_layouts.push(StateNodeLayout {
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
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "join bar must produce a rect");
    }

    // ── Test: choice diamond ──────────────────────────────────────

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
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polygon"),
            "choice must produce a polygon (diamond)"
        );
    }

    // ── Test: history circle ──────────────────────────────────────

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
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<circle"), "history must produce a circle");
        assert!(svg.contains(">H<"), "history must contain 'H' text");
    }

    // ── Test: deep history circle ─────────────────────────────────

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
        });
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<circle"),
            "deep history must produce a circle"
        );
        assert!(svg.contains(">H*<"), "deep history must contain 'H*' text");
    }

    // ── Test: concurrent region separator ─────────────────────────

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
        };
        layout.state_layouts.push(composite);
        let svg = render_state(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("stroke-dasharray"),
            "concurrent separator must be dashed"
        );
    }
}
