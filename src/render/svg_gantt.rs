use std::fmt::Write;

use crate::layout::gantt::{
    GanttBarLayout, GanttDepLayout, GanttLayout, GanttNoteLayout, GanttTimeAxis,
};
use crate::model::gantt::GanttDiagram;
use crate::render::svg::xml_escape;
use crate::render::svg::write_svg_root;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
const DEFAULT_BAR_FILL: &str = "#A4C2F4";
const DEFAULT_BAR_STROKE: &str = "#3D85C6";
const ARROW_COLOR: &str = "#555555";
const TEXT_FILL: &str = "#000000";
const GRID_COLOR: &str = "#DDDDDD";
const AXIS_TEXT_COLOR: &str = "#333333";
const LABEL_PADDING: f64 = 8.0;
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const NOTE_FOLD: f64 = 8.0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render a Gantt chart diagram to SVG.
pub fn render_gantt(
    _diagram: &GanttDiagram,
    layout: &GanttLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // SVG header
    write_svg_root(&mut buf, layout.width, layout.height);
    buf.push_str("<defs/><g>");

    // Defs: arrow marker
    write_defs(&mut buf);

    // Grid lines
    render_grid(&mut buf, layout);

    // Time axis
    render_time_axis(&mut buf, &layout.time_axis);

    let gantt_font = skin.font_color("gantt", TEXT_FILL);

    // Task bars with labels
    for bar in &layout.bars {
        render_bar(&mut buf, bar, gantt_font);
    }

    // Dependency arrows
    for dep in &layout.dependencies {
        render_dependency(&mut buf, dep);
    }

    for note in &layout.notes {
        render_note(&mut buf, note, gantt_font);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Defs
// ---------------------------------------------------------------------------

fn write_defs(buf: &mut String) {
    buf.push_str("<defs>\n");
    write!(
        buf,
        concat!(
            r#"<marker id="gantt-arrow" viewBox="0 0 10 10" refX="10" refY="5""#,
            r#" markerWidth="8" markerHeight="8" orient="auto-start-reverse">"#,
            r#"<path d="M 0 0 L 10 5 L 0 10 Z" fill="{}" stroke="none"/>"#,
            r#"</marker>"#,
        ),
        ARROW_COLOR,
    )
    .unwrap();
    buf.push('\n');
    buf.push_str("</defs>\n");
}

// ---------------------------------------------------------------------------
// Grid lines
// ---------------------------------------------------------------------------

fn render_grid(buf: &mut String, layout: &GanttLayout) {
    // Vertical grid lines at each time label position
    for label in &layout.time_axis.labels {
        write!(
            buf,
            r#"<line style="stroke:{GRID_COLOR};stroke-width:0.5;" x1="{x:.1}" x2="{x:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
            x = label.x,
            y1 = layout.time_axis.y,
            y2 = layout.height,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ---------------------------------------------------------------------------
// Time axis
// ---------------------------------------------------------------------------

fn render_time_axis(buf: &mut String, axis: &GanttTimeAxis) {
    for label in &axis.labels {
        let escaped = xml_escape(&label.text);
        write!(
            buf,
            r#"<text x="{x:.1}" y="{y:.1}" text-anchor="middle" font-size="{fs:.0}" fill="{AXIS_TEXT_COLOR}">{escaped}</text>"#,
            x = label.x,
            y = axis.y + FONT_SIZE + 2.0,
            fs = FONT_SIZE - 1.0,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ---------------------------------------------------------------------------
// Task bar
// ---------------------------------------------------------------------------

fn render_bar(buf: &mut String, bar: &GanttBarLayout, font_color: &str) {
    // Determine fill color
    let fill = bar.color.as_ref().map_or(DEFAULT_BAR_FILL, |c| {
        // Handle "Color/Color" format: use the first color
        if let Some(slash_pos) = c.find('/') {
            &c[..slash_pos]
        } else {
            c.as_str()
        }
    });

    let stroke = bar.color.as_ref().map_or(DEFAULT_BAR_STROKE, |c| {
        // Use second color as stroke if "Color/Color" format
        if let Some(slash_pos) = c.find('/') {
            &c[slash_pos + 1..]
        } else {
            DEFAULT_BAR_STROKE
        }
    });

    // Bar rectangle
    write!(
        buf,
        r#"<rect fill="{fill}" height="{h:.1}" rx="3" ry="3" style="stroke:{stroke};stroke-width:1;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = bar.x,
        y = bar.y,
        w = bar.width,
        h = bar.height,
    )
    .unwrap();
    buf.push('\n');

    // Task label to the left of the bar
    let label_x = bar.x - LABEL_PADDING;
    let label_y = bar.y + bar.height / 2.0 + FONT_SIZE * 0.35;
    render_creole_text(
        buf,
        &bar.label,
        label_x,
        label_y,
        FONT_SIZE + 4.0,
        font_color,
        Some("end"),
        "",
    );
}

// ---------------------------------------------------------------------------
// Dependency arrow
// ---------------------------------------------------------------------------

fn render_dependency(buf: &mut String, dep: &GanttDepLayout) {
    if dep.points.is_empty() {
        return;
    }

    if dep.points.len() == 2 {
        let (x1, y1) = dep.points[0];
        let (x2, y2) = dep.points[1];
        write!(
            buf,
            r#"<line marker-end="url(#gantt-arrow)" style="stroke:{ARROW_COLOR};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        )
        .unwrap();
        buf.push('\n');
    } else {
        let points_str: String = dep
            .points
            .iter()
            .map(|(px, py)| format!("{px:.1},{py:.1}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            buf,
            r#"<polyline fill="none" marker-end="url(#gantt-arrow)" points="{points_str}" style="stroke:{ARROW_COLOR};stroke-width:1;"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

fn render_note(buf: &mut String, note: &GanttNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        write!(
            buf,
            r#"<line style="stroke:{NOTE_BORDER};stroke-width:1;stroke-dasharray:4,4;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    write!(
        buf,
        r#"<polygon fill="{NOTE_BG}" points="{x:.1},{y:.1} {fx:.1},{y:.1} {x2:.1},{fy:.1} {x2:.1},{y2:.1} {x:.1},{y2:.1}" style="stroke:{NOTE_BORDER};stroke-width:1;"/>"#,
        x = note.x,
        y = note.y,
        fx = fold_x,
        fy = fold_y,
        x2 = note.x + note.width,
        y2 = note.y + note.height,
    )
    .unwrap();
    buf.push('\n');

    write!(
        buf,
        r#"<path d="M {fx:.1},{y:.1} L {fx:.1},{fy:.1} L {x2:.1},{fy:.1}" fill="none" style="stroke:{NOTE_BORDER};stroke-width:1;"/>"#,
        fx = fold_x,
        fy = fold_y,
        x2 = note.x + note.width,
        y = note.y,
    )
    .unwrap();
    buf.push('\n');

    render_creole_text(
        buf,
        &note.text,
        note.x + 6.0,
        note.y + NOTE_FOLD + FONT_SIZE,
        FONT_SIZE + 4.0,
        font_color,
        None,
        "",
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::gantt::{
        GanttBarLayout, GanttDepLayout, GanttLayout, GanttNoteLayout, GanttTimeAxis, GanttTimeLabel,
    };
    use crate::model::gantt::GanttDiagram;
    use crate::style::SkinParams;

    fn empty_model() -> GanttDiagram {
        GanttDiagram {
            tasks: vec![],
            dependencies: vec![],
            project_start: None,
            closed_days: vec![],
            colored_ranges: vec![],
            scale: None,
            print_scale: None,
            notes: vec![],
        }
    }

    fn empty_layout() -> GanttLayout {
        GanttLayout {
            bars: vec![],
            dependencies: vec![],
            notes: vec![],
            time_axis: GanttTimeAxis {
                labels: vec![],
                y: 20.0,
            },
            width: 400.0,
            height: 200.0,
        }
    }

    fn make_bar(id: &str, label: &str, x: f64, y: f64, w: f64) -> GanttBarLayout {
        GanttBarLayout {
            id: id.to_string(),
            label: label.to_string(),
            x,
            y,
            width: w,
            height: 20.0,
            color: None,
        }
    }

    // 1. Empty diagram produces valid SVG
    #[test]
    fn test_empty_svg() {
        let model = empty_model();
        let layout = empty_layout();
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    // 2. SVG contains defs with marker
    #[test]
    fn test_defs_marker() {
        let model = empty_model();
        let layout = empty_layout();
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("gantt-arrow"));
        assert!(svg.contains("</defs>"));
    }

    // 3. Single bar renders rect and label
    #[test]
    fn test_single_bar() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout
            .bars
            .push(make_bar("Design", "Design", 180.0, 50.0, 200.0));
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<rect"), "must contain bar rect");
        assert!(svg.contains("Design"), "must contain task label");
        assert!(
            svg.contains(r##"fill="#A4C2F4""##),
            "default fill color"
        );
        assert!(
            svg.contains("stroke:#3D85C6"),
            "default stroke color"
        );
    }

    // 4. Bar with custom color
    #[test]
    fn test_bar_with_color() {
        let model = empty_model();
        let mut layout = empty_layout();
        let mut bar = make_bar("T1", "Task 1", 180.0, 50.0, 100.0);
        bar.color = Some("Lavender/LightBlue".to_string());
        layout.bars.push(bar);
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"fill="Lavender""#), "first color as fill");
        assert!(
            svg.contains("stroke:LightBlue"),
            "second color as stroke"
        );
    }

    // 5. Bar with single color (no slash)
    #[test]
    fn test_bar_single_color() {
        let model = empty_model();
        let mut layout = empty_layout();
        let mut bar = make_bar("T1", "Task 1", 180.0, 50.0, 100.0);
        bar.color = Some("salmon".to_string());
        layout.bars.push(bar);
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"fill="salmon""#));
    }

    // 6. Time axis labels
    #[test]
    fn test_time_axis_labels() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.time_axis.labels.push(GanttTimeLabel {
            text: "W1".to_string(),
            x: 200.0,
        });
        layout.time_axis.labels.push(GanttTimeLabel {
            text: "W2".to_string(),
            x: 340.0,
        });
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("W1"), "time label W1");
        assert!(svg.contains("W2"), "time label W2");
    }

    // 7. Grid lines
    #[test]
    fn test_grid_lines() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.time_axis.labels.push(GanttTimeLabel {
            text: "D1".to_string(),
            x: 200.0,
        });
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("stroke:#DDDDDD"),
            "grid lines must use GRID_COLOR"
        );
    }

    // 8. Dependency arrow (2-point)
    #[test]
    fn test_dependency_2point() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.dependencies.push(GanttDepLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(100.0, 60.0), (200.0, 90.0)],
        });
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<line "), "2-point dep should use <line>");
        assert!(
            svg.contains(r#"marker-end="url(#gantt-arrow)""#),
            "must have arrow marker"
        );
    }

    // 9. Dependency arrow (polyline)
    #[test]
    fn test_dependency_polyline() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.dependencies.push(GanttDepLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(100.0, 60.0), (150.0, 60.0), (150.0, 90.0), (200.0, 90.0)],
        });
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polyline"),
            "multi-point dep should use <polyline>"
        );
        assert!(
            svg.contains(r#"marker-end="url(#gantt-arrow)""#),
            "polyline must also have arrow marker"
        );
    }

    // 10. Empty dependency points
    #[test]
    fn test_empty_dependency_points() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.dependencies.push(GanttDepLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![],
        });
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(!svg.contains("<line x1="), "no line for empty points");
        assert!(!svg.contains("<polyline"), "no polyline for empty points");
    }

    // 11. Task label position (text-anchor=end, to left of bar)
    #[test]
    fn test_label_position() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout
            .bars
            .push(make_bar("T", "My Task", 200.0, 50.0, 100.0));
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"text-anchor="end""#),
            "label should be right-aligned"
        );
        assert!(svg.contains("My Task"));
    }

    // 12. SVG dimensions match layout
    #[test]
    fn test_svg_dimensions() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.width = 600.0;
        layout.height = 300.0;
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"width="600px""#));
        assert!(svg.contains(r#"height="300px""#));
        assert!(svg.contains(r#"viewBox="0 0 600 300""#));
    }

    // 13. XML escaping in labels
    #[test]
    fn test_xml_escaping() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout
            .bars
            .push(make_bar("T", "A & B < C", 200.0, 50.0, 100.0));
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("A &amp; B &lt; C"),
            "label must be XML-escaped"
        );
    }

    // 14. Multiple bars and deps together
    #[test]
    fn test_full_chart() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.width = 500.0;
        layout.height = 200.0;
        layout
            .bars
            .push(make_bar("A", "Design", 200.0, 50.0, 100.0));
        layout.bars.push(make_bar("B", "Build", 300.0, 80.0, 60.0));
        layout.time_axis.labels.push(GanttTimeLabel {
            text: "D1".to_string(),
            x: 200.0,
        });
        layout.dependencies.push(GanttDepLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(300.0, 60.0), (300.0, 90.0)],
        });

        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.contains("</svg>"));
        assert_eq!(svg.matches("<rect").count(), 2, "2 bars expected");
        assert!(svg.contains("Design"));
        assert!(svg.contains("Build"));
        assert!(svg.contains("D1"));
        assert!(
            svg.matches(r#"marker-end="url(#gantt-arrow)""#).count() >= 1,
            "at least 1 dep arrow"
        );
    }

    // 15. Bar with rounded corners
    #[test]
    fn test_bar_rounded_corners() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.bars.push(make_bar("T", "Task", 200.0, 50.0, 100.0));
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"rx="3""#),
            "bars should have rounded corners"
        );
        assert!(
            svg.contains(r#"ry="3""#),
            "bars should have rounded corners"
        );
    }

    #[test]
    fn test_note_rendering() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.notes.push(GanttNoteLayout {
            text: "**note**".to_string(),
            x: 320.0,
            y: 40.0,
            width: 90.0,
            height: 42.0,
            connector: Some((300.0, 60.0, 320.0, 55.0)),
        });
        let svg = render_gantt(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<polygon"), "note body must be rendered");
        assert!(svg.contains("stroke-dasharray:4,4;"));
        assert!(
            svg.contains("font-weight=\"bold\""),
            "creole note text should be rendered"
        );
    }
}
