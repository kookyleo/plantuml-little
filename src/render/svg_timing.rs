use super::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::klimt::svg::{fmt_coord, SvgGraphic};
use crate::layout::timing::{
    TimingConstraintLayout, TimingLayout, TimingMsgLayout, TimingNoteLayout, TimingSegmentLayout,
    TimingTimeAxis, TimingTrackLayout,
};
use crate::model::timing::TimingDiagram;
use crate::render::svg_richtext::{
    count_creole_lines, render_creole_text, set_default_font_family,
};
use crate::style::SkinParams;
use crate::Result;

use crate::skin::rose::{BORDER_COLOR, ENTITY_BG, NOTE_BG, NOTE_BORDER, NOTE_FOLD, TEXT_COLOR};
const CONCISE_STROKE: &str = "#2E8B57";
const ARROW_COLOR: &str = "#555555";
const CONSTRAINT_COLOR: &str = "#FF6600";
const AXIS_LINE_COLOR: &str = "#888888";
const AXIS_TEXT_COLOR: &str = "#333333";
const GRID_LINE_COLOR: &str = "#333333";
const LABEL_PADDING: f64 = 8.0;
const ROBUST_BAND_HEIGHT: f64 = 16.0;

pub fn render_timing(
    _td: &TimingDiagram,
    layout: &TimingLayout,
    skin: &SkinParams,
) -> Result<String> {
    let font = skin.default_font_name().map(|name| {
        let normalized = name.trim_matches(|c| c == '"' || c == '\'');
        if normalized.eq_ignore_ascii_case("sansserif") || normalized.eq_ignore_ascii_case("dialog") {
            "'sans-serif'".to_string()
        } else if normalized.eq_ignore_ascii_case("monospaced") {
            "monospace".to_string()
        } else {
            normalized.to_string()
        }
    });
    set_default_font_family(font);
    let result = render_timing_inner(layout, skin);
    set_default_font_family(None);
    result
}

fn render_timing_inner(layout: &TimingLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let timing_bg = skin.background_color("timing", ENTITY_BG);
    let timing_border = skin.border_color("timing", BORDER_COLOR);
    let timing_font = skin.get_or("defaultfontcolor", skin.font_color("timing", TEXT_COLOR));
    let arrow_font = skin.font_color("arrow", timing_font);
    let constraint_color = skin.font_color("constraint", CONSTRAINT_COLOR);
    let arrow_color = skin.arrow_color(ARROW_COLOR);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = ensure_visible_int(layout.width) as f64;
    let svg_h = ensure_visible_int(layout.height) as f64;
    write_svg_root_bg(&mut buf, svg_w, svg_h, "TIMING", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);
    let mut sg = SvgGraphic::new(0, 1.0);
    let name_fs = layout.name_font_size;
    let state_fs = layout.state_font_size;
    let arrow_fs = layout.arrow_font_size;
    let constraint_fs = layout.constraint_font_size;
    let axis_fs = layout.axis_font_size;
    render_chart_borders(&mut sg, layout);
    render_tick_grid(&mut sg, layout);
    render_top_border(&mut sg, layout);
    for track in &layout.tracks {
        render_track(&mut sg, track, &timing_bg, &timing_border, &timing_font, name_fs, state_fs);
    }
    for msg in &layout.messages {
        render_message(&mut sg, msg, &arrow_color, arrow_font, arrow_fs);
    }
    for c in &layout.constraints {
        render_constraint(&mut sg, c, &constraint_color, constraint_fs);
    }
    for note in &layout.notes {
        render_note(&mut sg, note, &timing_font, state_fs);
    }
    render_time_axis(&mut sg, &layout.time_axis, axis_fs);
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

/// Render solid left and right vertical border lines of the chart area.
fn render_chart_borders(sg: &mut SvgGraphic, layout: &TimingLayout) {
    let y_top = layout.chart_top;
    let y_bot = layout.time_axis.y;
    // Left vertical border
    sg.push_raw(&format!(
        "<line style=\"stroke:{GRID_LINE_COLOR};stroke-width:0.5;\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
        fmt_coord(layout.chart_left), fmt_coord(layout.chart_left),
        fmt_coord(y_top), fmt_coord(y_bot),
    ));
    // Right vertical border
    sg.push_raw(&format!(
        "<line style=\"stroke:{GRID_LINE_COLOR};stroke-width:0.5;\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
        fmt_coord(layout.chart_right), fmt_coord(layout.chart_right),
        fmt_coord(y_top), fmt_coord(y_bot),
    ));
}

/// Render dashed vertical tick grid lines.
fn render_tick_grid(sg: &mut SvgGraphic, layout: &TimingLayout) {
    let y_top = layout.chart_top;
    let y_bot = layout.time_axis.y;
    for tick in &layout.time_axis.grid_ticks {
        sg.push_raw(&format!(
            "<line style=\"stroke:{GRID_LINE_COLOR};stroke-width:0.5;stroke-dasharray:3,5;\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
            fmt_coord(tick.x), fmt_coord(tick.x),
            fmt_coord(y_top), fmt_coord(y_bot),
        ));
    }
}

/// Render the solid top horizontal border line of the chart area.
fn render_top_border(sg: &mut SvgGraphic, layout: &TimingLayout) {
    sg.push_raw(&format!(
        "<line style=\"stroke:{GRID_LINE_COLOR};stroke-width:0.5;\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
        fmt_coord(layout.chart_left), fmt_coord(layout.chart_right),
        fmt_coord(layout.chart_top), fmt_coord(layout.chart_top),
    ));
}

fn render_track(
    sg: &mut SvgGraphic,
    track: &TimingTrackLayout,
    bg: &str,
    border: &str,
    font_color: &str,
    name_fs: f64,
    state_fs: f64,
) {
    // Participant name label first (matches Java rendering order)
    let label_x = track
        .segments
        .first()
        .map_or(LABEL_PADDING, |s| s.x_start - LABEL_PADDING);
    let label_y = track.y + track.height * 0.5 + name_fs * 0.35;
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &track.name,
        label_x,
        label_y,
        name_fs + 4.0,
        font_color,
        Some("end"),
        &format!(r#"font-size="{:.0}" font-weight="700""#, name_fs),
    );
    sg.push_raw(&tmp);
    // Track background rect
    if !track.segments.is_empty() {
        let x_min = track
            .segments
            .iter()
            .map(|s| s.x_start)
            .fold(f64::INFINITY, f64::min);
        let x_max = track
            .segments
            .iter()
            .map(|s| s.x_end)
            .fold(f64::NEG_INFINITY, f64::max);
        let w = (x_max - x_min).max(0.0);
        sg.push_raw(&format!(r#"<rect fill="{bg}" height="{}" opacity="0.30000" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#, fmt_coord(track.height), fmt_coord(w), fmt_coord(x_min), fmt_coord(track.y)));
        sg.push_raw("\n");
    }
    // Segments (signal lines and transitions)
    for (i, seg) in track.segments.iter().enumerate() {
        render_segment(sg, seg, i, &track.segments, state_fs);
    }
}

fn render_segment(
    sg: &mut SvgGraphic,
    seg: &TimingSegmentLayout,
    index: usize,
    all_segments: &[TimingSegmentLayout],
    state_fs: f64,
) {
    let stroke = if seg.is_robust {
        BORDER_COLOR
    } else {
        CONCISE_STROKE
    };
    if seg.is_robust {
        let band_top = seg.y - ROBUST_BAND_HEIGHT * 0.5;
        let w = seg.x_end - seg.x_start;
        if w > 0.0 {
            sg.set_fill_color(ENTITY_BG);
            sg.set_stroke_color(Some(stroke));
            sg.set_stroke_width(0.5, None);
            sg.svg_rectangle(seg.x_start, band_top, w, ROBUST_BAND_HEIGHT, 0.0, 0.0, 0.0);
        }
        if w > 10.0 {
            let cx = seg.x_start + w * 0.5;
            let cy = seg.y + state_fs * 0.35;
            let mut tmp = String::new();
            render_creole_text(
                &mut tmp,
                &seg.state,
                cx,
                cy,
                state_fs + 4.0,
                TEXT_COLOR,
                Some("middle"),
                &format!(r#"font-size="{:.0}""#, state_fs),
            );
            sg.push_raw(&tmp);
        }
        if index > 0 {
            let prev = &all_segments[index - 1];
            let tx = seg.x_start;
            let pbt = prev.y - ROBUST_BAND_HEIGHT * 0.5;
            let pbb = prev.y + ROBUST_BAND_HEIGHT * 0.5;
            let cbt = seg.y - ROBUST_BAND_HEIGHT * 0.5;
            let cbb = seg.y + ROBUST_BAND_HEIGHT * 0.5;
            let yf = if seg.y < prev.y { pbt } else { pbb };
            let yt = if seg.y < prev.y { cbb } else { cbt };
            sg.set_stroke_color(Some(stroke));
            sg.set_stroke_width(0.5, None);
            sg.svg_line(tx, yf, tx, yt, 0.0);
        }
    } else {
        if seg.x_end > seg.x_start {
            sg.set_stroke_color(Some(stroke));
            sg.set_stroke_width(0.5, None);
            sg.svg_line(seg.x_start, seg.y, seg.x_end, seg.y, 0.0);
        }
        if (seg.x_end - seg.x_start) > 10.0 {
            let cx = seg.x_start + (seg.x_end - seg.x_start) * 0.5;
            let cy = seg.y - 4.0;
            let mut tmp = String::new();
            render_creole_text(
                &mut tmp,
                &seg.state,
                cx,
                cy,
                state_fs + 4.0,
                TEXT_COLOR,
                Some("middle"),
                &format!(r#"font-size="{:.0}""#, state_fs),
            );
            sg.push_raw(&tmp);
        }
        if index > 0 {
            let prev = &all_segments[index - 1];
            let tx = seg.x_start;
            sg.set_stroke_color(Some(stroke));
            sg.set_stroke_width(0.5, None);
            sg.svg_line(tx, prev.y, tx, seg.y, 0.0);
        }
    }
}

fn render_message(sg: &mut SvgGraphic, msg: &TimingMsgLayout, arrow_color: &str, font_color: &str, arrow_fs: f64) {
    sg.set_stroke_color(Some(arrow_color));
    sg.set_stroke_width(1.0, None);
    sg.svg_line(msg.from_x, msg.from_y, msg.to_x, msg.to_y, 0.0);
    let dx = msg.to_x - msg.from_x;
    let dy = msg.to_y - msg.from_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        let ux = dx / len;
        let uy = dy / len;
        let px = -uy;
        let py = ux;
        let p1x = msg.to_x - ux * 9.0 + px * 4.0;
        let p1y = msg.to_y - uy * 9.0 + py * 4.0;
        let p2x = msg.to_x;
        let p2y = msg.to_y;
        let p3x = msg.to_x - ux * 9.0 - px * 4.0;
        let p3y = msg.to_y - uy * 9.0 - py * 4.0;
        sg.set_fill_color(arrow_color);
        sg.set_stroke_color(Some(arrow_color));
        sg.set_stroke_width(1.0, None);
        sg.svg_polygon(0.0, &[p1x, p1y, p2x, p2y, p3x, p3y, p1x, p1y]);
    }
    if !msg.label.is_empty() {
        let mx = (msg.from_x + msg.to_x) * 0.5;
        let my = (msg.from_y + msg.to_y) * 0.5 - 4.0;
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &msg.label,
            mx,
            my,
            arrow_fs + 4.0,
            font_color,
            Some("middle"),
            &format!(r#"font-size="{:.0}""#, arrow_fs),
        );
        sg.push_raw(&tmp);
    }
}

fn render_constraint(sg: &mut SvgGraphic, c: &TimingConstraintLayout, cc: &str, constraint_fs: f64) {
    sg.set_stroke_color(Some(cc));
    sg.set_stroke_width(1.0, None);
    sg.svg_line(c.x_start, c.y, c.x_end, c.y, 0.0);
    for &(tip_x, dir) in &[(c.x_start, 1.0_f64), (c.x_end, -1.0_f64)] {
        let p1x = tip_x + dir * 7.0;
        let p1y = c.y - 4.0;
        let p2x = tip_x;
        let p2y = c.y;
        let p3x = tip_x + dir * 7.0;
        let p3y = c.y + 4.0;
        sg.set_fill_color(cc);
        sg.set_stroke_color(Some(cc));
        sg.set_stroke_width(1.0, None);
        sg.svg_polygon(0.0, &[p1x, p1y, p2x, p2y, p3x, p3y, p1x, p1y]);
    }
    let mx = (c.x_start + c.x_end) * 0.5;
    let my = c.y - 4.0;
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &c.label,
        mx,
        my,
        constraint_fs + 4.0,
        cc,
        Some("middle"),
        &format!(r#"font-size="{:.0}""#, constraint_fs),
    );
    sg.push_raw(&tmp);
}

fn render_time_axis(sg: &mut SvgGraphic, axis: &TimingTimeAxis, axis_fs: f64) {
    // Axis horizontal line spans from first to last grid tick
    if let (Some(first), Some(last)) = (axis.grid_ticks.first(), axis.grid_ticks.last()) {
        sg.set_stroke_color(Some(AXIS_LINE_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_line(first.x, axis.y, last.x, axis.y, 0.0);
    }
    // Tick marks at every grid position
    for tick in &axis.grid_ticks {
        sg.set_stroke_color(Some(AXIS_LINE_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_line(tick.x, axis.y, tick.x, axis.y + 6.0, 0.0);
    }
    // Labels only at state-change event times
    for tick in &axis.ticks {
        let ly = axis.y + 6.0 + axis_fs + 2.0;
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &tick.label,
            tick.x,
            ly,
            axis_fs + 4.0,
            AXIS_TEXT_COLOR,
            Some("middle"),
            &format!(r#"font-size="{:.0}""#, axis_fs),
        );
        sg.push_raw(&tmp);
    }
}

fn render_note(sg: &mut SvgGraphic, note: &TimingNoteLayout, font_color: &str, note_fs: f64) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        sg.set_stroke_color(Some(NOTE_BORDER));
        sg.set_stroke_width(0.5, Some((4.0, 4.0)));
        sg.svg_line(x1, y1, x2, y2, 0.0);
    }
    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    let x2 = note.x + note.width;
    let y2 = note.y + note.height;
    sg.set_fill_color(NOTE_BG);
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(0.5, None);
    sg.svg_polygon(
        0.0,
        &[
            note.x, note.y, fold_x, note.y, x2, fold_y, x2, y2, note.x, y2,
        ],
    );
    sg.push_raw(&format!(r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#, fmt_coord(fold_x), fmt_coord(note.y), fmt_coord(fold_x), fmt_coord(fold_y), fmt_coord(x2), fmt_coord(fold_y)));
    let lc = count_creole_lines(&note.text) as f64;
    let sy = note.y + NOTE_FOLD + (note.height - lc * (note_fs + 4.0)).max(0.0) / 2.0 + note_fs;
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        note.x + 6.0,
        sy,
        note_fs + 4.0,
        font_color,
        None,
        &format!(r#"font-size="{:.0}""#, note_fs + 1.0),
    );
    sg.push_raw(&tmp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::timing::{
        TimingConstraintLayout, TimingLayout, TimingMsgLayout, TimingNoteLayout,
        TimingSegmentLayout, TimingTick, TimingTimeAxis, TimingTrackLayout,
    };
    use crate::model::timing::TimingDiagram;
    fn empty_model() -> TimingDiagram {
        TimingDiagram {
            participants: vec![],
            state_changes: vec![],
            messages: vec![],
            constraints: vec![],
            notes: vec![],
        }
    }
    fn empty_layout() -> TimingLayout {
        TimingLayout {
            tracks: vec![],
            messages: vec![],
            constraints: vec![],
            notes: vec![],
            time_axis: TimingTimeAxis {
                y: 100.0,
                grid_ticks: vec![],
                ticks: vec![],
            },
            width: 400.0,
            height: 200.0,
            chart_left: 20.0,
            chart_right: 380.0,
            chart_top: 20.0,
            name_font_size: 14.0,
            state_font_size: 12.0,
            arrow_font_size: 13.0,
            constraint_font_size: 12.0,
            axis_font_size: 11.0,
        }
    }
    fn make_segment(
        state: &str,
        x_start: f64,
        x_end: f64,
        y: f64,
        is_robust: bool,
    ) -> TimingSegmentLayout {
        TimingSegmentLayout {
            state: state.to_string(),
            x_start,
            x_end,
            y,
            is_robust,
        }
    }
    fn make_track(
        name: &str,
        y: f64,
        height: f64,
        segments: Vec<TimingSegmentLayout>,
    ) -> TimingTrackLayout {
        TimingTrackLayout {
            name: name.to_string(),
            y,
            height,
            segments,
            state_labels: vec![],
            header_height: 17.2969,
        }
    }
    #[test]
    fn test_empty_svg() {
        let svg = render_timing(&empty_model(), &empty_layout(), &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }
    #[test]
    fn test_defs_empty() {
        let svg = render_timing(&empty_model(), &empty_layout(), &SkinParams::default()).unwrap();
        assert!(svg.contains("<defs/>"));
    }
    #[test]
    fn test_svg_dimensions() {
        let mut l = empty_layout();
        l.width = 600.0;
        l.height = 300.0;
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"width="601px""#));
        assert!(svg.contains(r#"height="301px""#));
        assert!(svg.contains(r#"viewBox="0 0 601 301""#));
    }
    #[test]
    fn test_robust_segments() {
        let mut l = empty_layout();
        l.tracks.push(make_track(
            "DNS",
            20.0,
            40.0,
            vec![make_segment("Idle", 200.0, 350.0, 40.0, true)],
        ));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("DNS"));
        assert!(svg.contains("Idle"));
    }
    #[test]
    fn test_concise_segments() {
        let mut l = empty_layout();
        l.tracks.push(make_track(
            "WU",
            20.0,
            24.0,
            vec![make_segment("Waiting", 200.0, 400.0, 32.0, false)],
        ));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("Waiting"));
        assert!(svg.contains("WU"));
    }
    #[test]
    fn test_message_arrow() {
        let mut l = empty_layout();
        l.messages.push(TimingMsgLayout {
            from_x: 200.0,
            from_y: 40.0,
            to_x: 200.0,
            to_y: 80.0,
            label: "URL".into(),
        });
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("URL"));
    }
    #[test]
    fn test_message_no_label() {
        let mut l = empty_layout();
        l.messages.push(TimingMsgLayout {
            from_x: 200.0,
            from_y: 40.0,
            to_x: 200.0,
            to_y: 80.0,
            label: "".into(),
        });
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert_eq!(svg.matches("<text").count(), 0);
    }
    #[test]
    fn test_constraint_rendering() {
        let mut l = empty_layout();
        l.constraints.push(TimingConstraintLayout {
            x_start: 200.0,
            x_end: 350.0,
            y: 90.0,
            label: "{150 ms}".into(),
        });
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.matches("<polygon").count() >= 2);
        assert!(svg.contains("{150 ms}"));
    }
    #[test]
    fn test_time_axis() {
        let mut l = empty_layout();
        l.time_axis.grid_ticks.push(TimingTick {
            x: 200.0,
            label: "0".into(),
        });
        l.time_axis.grid_ticks.push(TimingTick {
            x: 350.0,
            label: "100".into(),
        });
        l.time_axis.ticks.push(TimingTick {
            x: 200.0,
            label: "0".into(),
        });
        l.time_axis.ticks.push(TimingTick {
            x: 350.0,
            label: "100".into(),
        });
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("0"));
        assert!(svg.contains("100"));
        assert!(svg.contains(&format!("stroke:{AXIS_LINE_COLOR}")));
    }
    #[test]
    fn test_xml_escaping() {
        let mut l = empty_layout();
        l.tracks.push(make_track(
            "A & B",
            20.0,
            40.0,
            vec![make_segment("S<1>", 200.0, 400.0, 40.0, true)],
        ));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("A &amp; B"));
        assert!(svg.contains("S&lt;1&gt;"));
    }
    #[test]
    fn test_tick_grid() {
        let mut l = empty_layout();
        l.time_axis.grid_ticks.push(TimingTick {
            x: 250.0,
            label: "50".into(),
        });
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("stroke-dasharray"));
    }
    #[test]
    fn test_robust_transition() {
        let mut l = empty_layout();
        l.tracks.push(make_track(
            "Sig",
            20.0,
            40.0,
            vec![
                make_segment("Low", 200.0, 300.0, 50.0, true),
                make_segment("High", 300.0, 400.0, 30.0, true),
            ],
        ));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.matches("<line").count() >= 1);
    }
    #[test]
    fn test_concise_transition() {
        let mut l = empty_layout();
        l.tracks.push(make_track(
            "Sig",
            20.0,
            24.0,
            vec![
                make_segment("Off", 200.0, 300.0, 32.0, false),
                make_segment("On", 300.0, 400.0, 28.0, false),
            ],
        ));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.matches("<line").count() >= 2);
    }
    #[test]
    fn test_track_background() {
        let mut l = empty_layout();
        l.tracks.push(make_track(
            "A",
            20.0,
            40.0,
            vec![make_segment("Idle", 200.0, 400.0, 40.0, true)],
        ));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("opacity=\"0.30000\""));
    }
    #[test]
    fn test_full_diagram() {
        let mut l = empty_layout();
        l.width = 600.0;
        l.height = 250.0;
        l.tracks.push(make_track(
            "DNS Resolver",
            20.0,
            40.0,
            vec![
                make_segment("Idle", 200.0, 400.0, 40.0, true),
                make_segment("Processing", 400.0, 550.0, 30.0, true),
            ],
        ));
        l.tracks.push(make_track(
            "Web User",
            76.0,
            24.0,
            vec![
                make_segment("Idle", 200.0, 300.0, 88.0, false),
                make_segment("Waiting", 300.0, 550.0, 82.0, false),
            ],
        ));
        l.messages.push(TimingMsgLayout {
            from_x: 300.0,
            from_y: 88.0,
            to_x: 300.0,
            to_y: 40.0,
            label: "URL".into(),
        });
        l.constraints.push(TimingConstraintLayout {
            x_start: 350.0,
            x_end: 500.0,
            y: 110.0,
            label: "{150 ms}".into(),
        });
        l.time_axis.ticks = vec![
            TimingTick {
                x: 200.0,
                label: "0".into(),
            },
            TimingTick {
                x: 300.0,
                label: "100".into(),
            },
            TimingTick {
                x: 550.0,
                label: "700".into(),
            },
        ];
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("DNS Resolver"));
        assert!(svg.contains("Web User"));
        assert!(svg.contains("URL"));
        assert!(svg.contains("{150 ms}"));
        assert!(svg.contains("0"));
        assert!(svg.contains("700"));
    }
    #[test]
    fn test_participant_label_bold() {
        let mut l = empty_layout();
        l.tracks.push(make_track(
            "Signal",
            20.0,
            40.0,
            vec![make_segment("Low", 200.0, 400.0, 40.0, true)],
        ));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("font-weight=\"700\""));
    }
    #[test]
    fn test_constraint_label_color() {
        let mut l = empty_layout();
        l.constraints.push(TimingConstraintLayout {
            x_start: 200.0,
            x_end: 350.0,
            y: 90.0,
            label: "test".into(),
        });
        let skin = SkinParams::default();
        let svg = render_timing(&empty_model(), &l, &skin).unwrap();
        let c = skin.font_color("constraint", CONSTRAINT_COLOR);
        assert!(svg.contains(&format!(r#"fill="{}""#, c)));
    }
    #[test]
    fn test_track_no_segments() {
        let mut l = empty_layout();
        l.tracks.push(make_track("Empty", 20.0, 40.0, vec![]));
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("Empty"));
    }
    #[test]
    fn test_end_to_end() {
        use crate::layout::timing::layout_timing;
        use crate::parser::timing::parse_timing_diagram;
        let src = "@startuml\nrobust \"DNS Resolver\" as DNS\nrobust \"Web Browser\" as WB\nconcise \"Web User\" as WU\n\n@0\nWU is Idle\nWB is Idle\nDNS is Idle\n\n@+100\nWU is Waiting\nWB is Processing\n\n@+200\nWB is Waiting\n\n@+100\nDNS is Processing\n\n@+300\nDNS is Idle\n@enduml";
        let td = parse_timing_diagram(src).unwrap();
        let lo = layout_timing(&td, &SkinParams::new()).unwrap();
        let svg = render_timing(&td, &lo, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("DNS Resolver"));
        assert!(svg.contains("Web Browser"));
        assert!(svg.contains("Web User"));
        assert!(svg.contains("Idle"));
        assert!(svg.contains("Processing"));
        assert!(svg.contains("Waiting"));
    }
    #[test]
    fn test_note_rendering() {
        let mut l = empty_layout();
        l.notes.push(TimingNoteLayout {
            text: "**watch**".to_string(),
            x: 250.0,
            y: 40.0,
            width: 100.0,
            height: 44.0,
            connector: Some((230.0, 50.0, 250.0, 56.0)),
        });
        let svg = render_timing(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.contains("font-weight=\"700\""));
    }
}
