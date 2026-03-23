use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
use crate::layout::gantt::{GanttBarLayout, GanttDepLayout, GanttLayout, GanttNoteLayout, GanttTimeAxis};
use crate::model::gantt::GanttDiagram;
use crate::render::svg::{write_svg_root_bg, write_bg_rect};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 12.0;
const DEFAULT_BAR_FILL: &str = "#A4C2F4";
const DEFAULT_BAR_STROKE: &str = "#3D85C6";
const ARROW_COLOR: &str = "#555555";
const GRID_COLOR: &str = "#DDDDDD";
const AXIS_TEXT_COLOR: &str = "#333333";
const LABEL_PADDING: f64 = 8.0;
use crate::skin::rose::{NOTE_BG, NOTE_BORDER, NOTE_FOLD, TEXT_COLOR};

pub fn render_gantt(_diagram: &GanttDiagram, layout: &GanttLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "GANTT", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);

    let mut sg = SvgGraphic::new(0, 1.0);
    render_grid(&mut sg, layout);
    render_time_axis(&mut sg, &layout.time_axis);
    let gantt_font = skin.font_color("gantt", TEXT_COLOR);
    for bar in &layout.bars { render_bar(&mut sg, bar, gantt_font); }
    for dep in &layout.dependencies { render_dependency(&mut sg, dep); }
    for note in &layout.notes { render_note(&mut sg, note, gantt_font); }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_grid(sg: &mut SvgGraphic, layout: &GanttLayout) {
    sg.set_stroke_color(Some(GRID_COLOR));
    sg.set_stroke_width(0.5, None);
    for label in &layout.time_axis.labels { sg.svg_line(label.x, layout.time_axis.y, label.x, layout.height, 0.0); }
}

fn render_time_axis(sg: &mut SvgGraphic, axis: &GanttTimeAxis) {
    let axis_fs = FONT_SIZE - 1.0;
    for label in &axis.labels {
        let tl = font_metrics::text_width(&label.text, "SansSerif", axis_fs, false, false);
        sg.set_fill_color(AXIS_TEXT_COLOR);
        sg.svg_text(&label.text, label.x, axis.y + FONT_SIZE + 2.0, Some("sans-serif"), axis_fs, None, None, None, tl, LengthAdjust::Spacing, None, 0, Some("middle"));
    }
}

fn render_bar(sg: &mut SvgGraphic, bar: &GanttBarLayout, font_color: &str) {
    let fill = bar.color.as_ref().map_or(DEFAULT_BAR_FILL, |c| { if let Some(p) = c.find('/') { &c[..p] } else { c.as_str() } });
    let stroke = bar.color.as_ref().map_or(DEFAULT_BAR_STROKE, |c| { if let Some(p) = c.find('/') { &c[p + 1..] } else { DEFAULT_BAR_STROKE } });
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(bar.x, bar.y, bar.width, bar.height, 3.0, 3.0, 0.0);
    let label_x = bar.x - LABEL_PADDING;
    let label_y = bar.y + bar.height / 2.0 + FONT_SIZE * 0.35;
    let mut tmp = String::new();
    render_creole_text(&mut tmp, &bar.label, label_x, label_y, FONT_SIZE + 4.0, font_color, Some("end"), r#"font-size="12""#);
    sg.push_raw(&tmp);
}

fn render_dependency(sg: &mut SvgGraphic, dep: &GanttDepLayout) {
    if dep.points.is_empty() { return; }
    if dep.points.len() == 2 {
        let (x1, y1) = dep.points[0]; let (x2, y2) = dep.points[1];
        sg.set_stroke_color(Some(ARROW_COLOR)); sg.set_stroke_width(1.0, None);
        sg.svg_line(x1, y1, x2, y2, 0.0);
    } else {
        let flat: Vec<f64> = dep.points.iter().flat_map(|(px, py)| [*px, *py]).collect();
        sg.set_fill_color("none"); sg.set_stroke_color(Some(ARROW_COLOR)); sg.set_stroke_width(1.0, None);
        sg.svg_polyline(&flat);
    }
    if dep.points.len() >= 2 {
        let (tx, ty) = dep.points[dep.points.len() - 1]; let (fx, fy) = dep.points[dep.points.len() - 2];
        let dx = tx - fx; let dy = ty - fy; let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            let ux = dx / len; let uy = dy / len; let px = -uy; let py = ux;
            let p1x = tx - ux * 9.0 + px * 4.0; let p1y = ty - uy * 9.0 + py * 4.0;
            let p3x = tx - ux * 9.0 - px * 4.0; let p3y = ty - uy * 9.0 - py * 4.0;
            sg.set_fill_color(ARROW_COLOR); sg.set_stroke_color(Some(ARROW_COLOR)); sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[p1x, p1y, tx, ty, p3x, p3y, p1x, p1y]);
        }
    }
}

fn render_note(sg: &mut SvgGraphic, note: &GanttNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        sg.set_stroke_color(Some(NOTE_BORDER)); sg.set_stroke_width(0.5, Some((4.0, 4.0)));
        sg.svg_line(x1, y1, x2, y2, 0.0);
    }
    let fold_x = note.x + note.width - NOTE_FOLD; let fold_y = note.y + NOTE_FOLD;
    let x2 = note.x + note.width; let y2 = note.y + note.height;
    sg.set_fill_color(NOTE_BG); sg.set_stroke_color(Some(NOTE_BORDER)); sg.set_stroke_width(0.5, None);
    sg.svg_polygon(0.0, &[note.x, note.y, fold_x, note.y, x2, fold_y, x2, y2, note.x, y2]);
    sg.push_raw(&format!(r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#, fmt_coord(fold_x), fmt_coord(note.y), fmt_coord(fold_x), fmt_coord(fold_y), fmt_coord(x2), fmt_coord(fold_y)));
    sg.push_raw("\n");
    let mut tmp = String::new();
    render_creole_text(&mut tmp, &note.text, note.x + 6.0, note.y + NOTE_FOLD + FONT_SIZE, FONT_SIZE + 4.0, font_color, None, r#"font-size="13""#);
    sg.push_raw(&tmp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::gantt::{GanttBarLayout, GanttDepLayout, GanttLayout, GanttNoteLayout, GanttTimeAxis, GanttTimeLabel};
    use crate::model::gantt::GanttDiagram;
    use crate::style::SkinParams;

    fn empty_model() -> GanttDiagram { GanttDiagram { tasks: vec![], dependencies: vec![], project_start: None, closed_days: vec![], colored_ranges: vec![], scale: None, print_scale: None, notes: vec![] } }
    fn empty_layout() -> GanttLayout { GanttLayout { bars: vec![], dependencies: vec![], notes: vec![], time_axis: GanttTimeAxis { labels: vec![], y: 20.0 }, width: 400.0, height: 200.0 } }
    fn make_bar(id: &str, label: &str, x: f64, y: f64, w: f64) -> GanttBarLayout { GanttBarLayout { id: id.into(), label: label.into(), x, y, width: w, height: 20.0, color: None } }

    #[test] fn test_empty_svg() { let svg = render_gantt(&empty_model(), &empty_layout(), &SkinParams::default()).unwrap(); assert!(svg.contains("<svg")); assert!(svg.contains("</svg>")); assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\"")); }
    #[test] fn test_defs_empty() { let svg = render_gantt(&empty_model(), &empty_layout(), &SkinParams::default()).unwrap(); assert!(svg.contains("<defs/>")); }
    #[test] fn test_single_bar() { let mut l = empty_layout(); l.bars.push(make_bar("Design", "Design", 180.0, 50.0, 200.0)); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains("<rect")); assert!(svg.contains("Design")); assert!(svg.contains(r##"fill="#A4C2F4""##)); assert!(svg.contains("stroke:#3D85C6")); }
    #[test] fn test_bar_with_color() { let mut l = empty_layout(); let mut b = make_bar("T1", "Task 1", 180.0, 50.0, 100.0); b.color = Some("Lavender/LightBlue".into()); l.bars.push(b); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains(r#"fill="Lavender""#)); assert!(svg.contains("stroke:LightBlue")); }
    #[test] fn test_bar_single_color() { let mut l = empty_layout(); let mut b = make_bar("T1", "Task 1", 180.0, 50.0, 100.0); b.color = Some("salmon".into()); l.bars.push(b); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains(r#"fill="salmon""#)); }
    #[test] fn test_time_axis_labels() { let mut l = empty_layout(); l.time_axis.labels.push(GanttTimeLabel { text: "W1".into(), x: 200.0 }); l.time_axis.labels.push(GanttTimeLabel { text: "W2".into(), x: 340.0 }); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains("W1")); assert!(svg.contains("W2")); }
    #[test] fn test_grid_lines() { let mut l = empty_layout(); l.time_axis.labels.push(GanttTimeLabel { text: "D1".into(), x: 200.0 }); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains("stroke:#DDDDDD")); }
    #[test] fn test_dependency_2point() { let mut l = empty_layout(); l.dependencies.push(GanttDepLayout { from: "A".into(), to: "B".into(), points: vec![(100.0, 60.0), (200.0, 90.0)] }); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains("<line ")); assert!(svg.contains("<polygon")); }
    #[test] fn test_dependency_polyline() { let mut l = empty_layout(); l.dependencies.push(GanttDepLayout { from: "A".into(), to: "B".into(), points: vec![(100.0, 60.0), (150.0, 60.0), (150.0, 90.0), (200.0, 90.0)] }); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains("<polyline")); assert!(svg.contains("<polygon")); }
    #[test] fn test_empty_dependency_points() { let mut l = empty_layout(); l.dependencies.push(GanttDepLayout { from: "A".into(), to: "B".into(), points: vec![] }); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(!svg.contains("<line x1=")); assert!(!svg.contains("<polyline")); }
    #[test] fn test_label_position() { let mut l = empty_layout(); l.bars.push(make_bar("T", "My Task", 200.0, 50.0, 100.0)); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains(r#"text-anchor="end""#)); assert!(svg.contains("My Task")); }
    #[test] fn test_svg_dimensions() { let mut l = empty_layout(); l.width = 600.0; l.height = 300.0; let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains(r#"width="600px""#)); assert!(svg.contains(r#"height="300px""#)); assert!(svg.contains(r#"viewBox="0 0 600 300""#)); }
    #[test] fn test_xml_escaping() { let mut l = empty_layout(); l.bars.push(make_bar("T", "A & B < C", 200.0, 50.0, 100.0)); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains("A &amp; B &lt; C")); }
    #[test] fn test_full_chart() { let mut l = empty_layout(); l.width = 500.0; l.height = 200.0; l.bars.push(make_bar("A", "Design", 200.0, 50.0, 100.0)); l.bars.push(make_bar("B", "Build", 300.0, 80.0, 60.0)); l.time_axis.labels.push(GanttTimeLabel { text: "D1".into(), x: 200.0 }); l.dependencies.push(GanttDepLayout { from: "A".into(), to: "B".into(), points: vec![(300.0, 60.0), (300.0, 90.0)] }); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.starts_with("<svg")); assert!(svg.contains("</svg>")); assert_eq!(svg.matches("<rect").count(), 2); assert!(svg.contains("Design")); assert!(svg.contains("Build")); assert!(svg.contains("D1")); assert!(svg.matches("<polygon").count() >= 1); }
    #[test] fn test_bar_rounded_corners() { let mut l = empty_layout(); l.bars.push(make_bar("T", "Task", 200.0, 50.0, 100.0)); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains(r#"rx="3""#)); assert!(svg.contains(r#"ry="3""#)); }
    #[test] fn test_note_rendering() { let mut l = empty_layout(); l.notes.push(GanttNoteLayout { text: "**note**".into(), x: 320.0, y: 40.0, width: 90.0, height: 42.0, connector: Some((300.0, 60.0, 320.0, 55.0)) }); let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap(); assert!(svg.contains("<polygon")); assert!(svg.contains("stroke-dasharray")); assert!(svg.contains("font-weight=\"700\"")); }
}
