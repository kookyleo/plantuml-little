use super::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::klimt::svg::{fmt_coord, SvgGraphic};
use crate::layout::timing::{
    TimingConstraintLayout, TimingLayout, TimingMsgLayout, TimingNoteLayout, TimingTimeAxis,
    TimingTrackLayout,
};
use crate::model::timing::TimingDiagram;
use crate::render::svg_richtext::{
    count_creole_lines, render_creole_text, set_default_font_family,
};
use crate::style::SkinParams;
use crate::Result;

use crate::skin::rose::{NOTE_BG, NOTE_BORDER, NOTE_FOLD};
const TIMING_LINE_COLOR: &str = "#006400";
const TIMING_FILL_COLOR: &str = "#E2E2F0";
const ARROW_COLOR: &str = "#00008B";
const CONSTRAINT_COLOR: &str = "#8B0000";
const AXIS_LINE_COLOR: &str = "#333333";
const GRID_LINE_COLOR: &str = "#333333";
/// Default font color for timing diagrams, from Java rose.skin `timingDiagram { FontColor #3 }`
const TIMING_FONT_COLOR: &str = "#333333";
const CONCISE_RIBBON_HEIGHT: f64 = 24.0;
const CONCISE_SHAPE_DELTA: f64 = 12.0;

fn timing_element_font_color<'a>(
    skin: &'a SkinParams,
    element: &str,
    default: &'a str,
) -> &'a str {
    let key1 = format!("{element}fontcolor");
    let key2 = format!("{element}.fontcolor");
    skin.get(&key1)
        .or_else(|| skin.get(&key2))
        .or_else(|| skin.get("defaultfontcolor"))
        .or_else(|| skin.get("fontcolor"))
        .or_else(|| skin.get("root.fontcolor"))
        .unwrap_or(default)
}

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
    // Java rose.skin: timingDiagram { FontColor #3 } => #333333 as base default.
    // In Java, the style system overrides skinparam defaults (including defaultFontColor).
    // Only timing-specific skinparam overrides apply.
    let timing_font = skin.get("timingfontcolor")
        .or_else(|| skin.get("timing.fontcolor"))
        .or_else(|| skin.get("defaultfontcolor"))
        .or_else(|| skin.get("fontcolor"))
        .or_else(|| skin.get("root.fontcolor"))
        .unwrap_or(TIMING_FONT_COLOR);
    let timing_line = skin
        .get("timing.linecolor")
        .or_else(|| skin.get("timing.bordercolor"))
        .or_else(|| skin.get("timingcolor"))
        .unwrap_or(TIMING_LINE_COLOR);
    let timing_fill = skin
        .get("timing.backgroundcolor")
        .or_else(|| skin.get("timing.backcolor"))
        .unwrap_or(TIMING_FILL_COLOR);
    let arrow_font = timing_element_font_color(skin, "arrow", timing_font);
    let arrow_color = skin
        .get("arrow.linecolor")
        .or_else(|| skin.get("arrowcolor"))
        .unwrap_or(ARROW_COLOR);
    let constraint_line_color = skin
        .get("constraintarrow.linecolor")
        .or_else(|| skin.get("constraintarrowcolor"))
        .unwrap_or(CONSTRAINT_COLOR);
    let constraint_font = skin
        .get("constraintarrowfontcolor")
        .or_else(|| skin.get("constraintarrow.fontcolor"))
        .or_else(|| skin.get("defaultfontcolor"))
        .or_else(|| skin.get("fontcolor"))
        .unwrap_or(constraint_line_color);
    let axis_font = timing_element_font_color(skin, "timeline", timing_font);
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
        render_track(
            &mut sg,
            track,
            timing_fill,
            timing_line,
            timing_font,
            name_fs,
            state_fs,
            layout.chart_left,
            layout.chart_right,
            layout.chart_top,
        );
    }
    for c in &layout.constraints {
        render_constraint(
            &mut sg,
            c,
            constraint_line_color,
            constraint_font,
            constraint_fs,
        );
    }
    for note in &layout.notes {
        render_note(&mut sg, note, &timing_font, state_fs);
    }
    render_time_axis(&mut sg, &layout.time_axis, axis_font, axis_fs);
    for msg in &layout.messages {
        render_message(&mut sg, msg, arrow_color, arrow_font, arrow_fs);
    }
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
    fill_color: &str,
    line_color: &str,
    font_color: &str,
    name_fs: f64,
    state_fs: f64,
    chart_left: f64,
    chart_right: f64,
    chart_top: f64,
) {
    if (track.y - chart_top).abs() > f64::EPSILON {
        sg.push_raw(&format!(
            r#"<line style="stroke:{GRID_LINE_COLOR};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(chart_left),
            fmt_coord(chart_right),
            fmt_coord(track.y),
            fmt_coord(track.y),
        ));
    }
    let label_x = chart_left + 5.0;
    let label_y = track.y + crate::font_metrics::ascent("SansSerif", name_fs, false, false);
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &track.name,
        label_x,
        label_y,
        name_fs + 4.0,
        font_color,
        None,
        &format!(r#"font-size="{:.0}" font-weight="bold""#, name_fs),
    );
    sg.push_raw(&tmp);
    let text_len = crate::font_metrics::text_width(&track.name, "SansSerif", name_fs, true, false);
    let underline_y = track.y + track.header_height;
    let tab_end_x = label_x + text_len + 1.0;
    sg.push_raw(&format!(
        r#"<line style="stroke:{GRID_LINE_COLOR};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(chart_left),
        fmt_coord(tab_end_x),
        fmt_coord(underline_y),
        fmt_coord(underline_y),
    ));
    sg.push_raw(&format!(
        r#"<line style="stroke:{GRID_LINE_COLOR};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(tab_end_x),
        fmt_coord(tab_end_x + 10.0),
        fmt_coord(underline_y),
        fmt_coord(track.y),
    ));
    if track.is_robust {
        render_robust_track(sg, track, line_color, font_color, state_fs, chart_left);
    } else {
        render_concise_track(sg, track, fill_color, line_color, font_color, state_fs);
    }
}

fn robust_state_baseline(y: f64, state_fs: f64) -> f64 {
    let ascent = crate::font_metrics::ascent("SansSerif", state_fs, false, false);
    let line_height = crate::font_metrics::line_height("SansSerif", state_fs, false, false);
    y + ascent - line_height * 0.5 + 1.0
}

fn render_robust_track(
    sg: &mut SvgGraphic,
    track: &TimingTrackLayout,
    line_color: &str,
    font_color: &str,
    state_fs: f64,
    chart_left: f64,
) {
    let mut state_y: std::collections::HashMap<&str, f64> = std::collections::HashMap::new();
    for seg in &track.segments {
        state_y.entry(seg.state.as_str()).or_insert(seg.y);
    }

    let labels: Vec<&str> = if track.state_labels.is_empty() {
        let mut labels = Vec::new();
        for seg in &track.segments {
            let state = seg.state.as_str();
            if !labels.contains(&state) {
                labels.push(state);
            }
        }
        labels
    } else {
        track.state_labels.iter().map(String::as_str).collect()
    };
    let state_label_x = chart_left + 5.0;

    for state in labels {
        if let Some(&y) = state_y.get(state) {
            let mut tmp = String::new();
            render_creole_text(
                &mut tmp,
                state,
                state_label_x,
                robust_state_baseline(y, state_fs),
                state_fs + 4.0,
                font_color,
                None,
                &format!(r#"font-size="{:.0}""#, state_fs),
            );
            sg.push_raw(&tmp);
        }
    }

    for seg in &track.segments {
        if seg.x_end > seg.x_start {
            sg.push_raw(&format!(
                r#"<line style="stroke:{line_color};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(seg.x_start),
                fmt_coord(seg.x_end),
                fmt_coord(seg.y),
                fmt_coord(seg.y),
            ));
        }
    }

    for pair in track.segments.windows(2) {
        let prev = &pair[0];
        let curr = &pair[1];
        if (prev.y - curr.y).abs() < f64::EPSILON {
            continue;
        }
        sg.push_raw(&format!(
            r#"<line style="stroke:{line_color};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(curr.x_start),
            fmt_coord(curr.x_start),
            fmt_coord(prev.y.min(curr.y)),
            fmt_coord(prev.y.max(curr.y)),
        ));
    }
}

fn concise_state_label_baseline(track: &TimingTrackLayout, state_fs: f64) -> f64 {
    let center_y = track.y + track.height - 22.0;
    let line_height = crate::font_metrics::line_height("SansSerif", state_fs, true, false);
    let ascent = crate::font_metrics::ascent("SansSerif", state_fs, true, false);
    center_y - line_height * 0.5 + ascent
}

fn render_concise_track(
    sg: &mut SvgGraphic,
    track: &TimingTrackLayout,
    fill_color: &str,
    line_color: &str,
    font_color: &str,
    state_fs: f64,
) {
    let center_y = track.y + track.height - 22.0;
    let top = center_y - CONCISE_RIBBON_HEIGHT * 0.5;
    let bottom = center_y + CONCISE_RIBBON_HEIGHT * 0.5;
    let label_y = concise_state_label_baseline(track, state_fs);
    let mut label_positions: Vec<(String, f64)> = Vec::new();

    for (index, seg) in track.segments.iter().enumerate() {
        let len = seg.x_end - seg.x_start;
        if len <= 0.0 {
            continue;
        }

        if index == 0 && track.segments.len() == 1 {
            sg.push_raw(&format!(
                r#"<path d="M{} {} L{} {} L{} {} L{} {} Z" fill="{}" style="stroke:{};stroke-width:1.5;"/>"#,
                fmt_coord(seg.x_start),
                fmt_coord(top),
                fmt_coord(seg.x_end),
                fmt_coord(top),
                fmt_coord(seg.x_end),
                fmt_coord(bottom),
                fmt_coord(seg.x_start),
                fmt_coord(bottom),
                fill_color,
                line_color,
            ));
        } else if index == 0 {
            sg.push_raw(&format!(
                r#"<polygon fill="{}" points="{},{},{},{},{},{},{},{},{},{},{},{}" style="stroke:{};stroke-width:1.5;"/>"#,
                fill_color,
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(top),
                fmt_coord(seg.x_end - CONCISE_SHAPE_DELTA),
                fmt_coord(top),
                fmt_coord(seg.x_end),
                fmt_coord(center_y),
                fmt_coord(seg.x_end - CONCISE_SHAPE_DELTA),
                fmt_coord(bottom),
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(bottom),
                fmt_coord(seg.x_start),
                fmt_coord(center_y),
                line_color,
            ));
        } else if index + 1 == track.segments.len() {
            sg.push_raw(&format!(
                r#"<polygon fill="{}" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{};stroke-width:1.5;"/>"#,
                fill_color,
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(top),
                fmt_coord(seg.x_end),
                fmt_coord(top),
                fmt_coord(seg.x_end),
                fmt_coord(bottom),
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(bottom),
                fmt_coord(seg.x_start),
                fmt_coord(center_y),
                fill_color,
            ));
            sg.push_raw(&format!(
                r#"<path d="M{},{} L{},{} L{},{} L{},{} L{},{}" fill="{}" style="stroke:{};stroke-width:1.5;"/>"#,
                fmt_coord(seg.x_end),
                fmt_coord(top),
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(top),
                fmt_coord(seg.x_start),
                fmt_coord(center_y),
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(bottom),
                fmt_coord(seg.x_end),
                fmt_coord(bottom),
                fill_color,
                line_color,
            ));
        } else {
            sg.push_raw(&format!(
                r#"<polygon fill="{}" points="{},{},{},{},{},{},{},{},{},{},{},{}" style="stroke:{};stroke-width:1.5;"/>"#,
                fill_color,
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(top),
                fmt_coord(seg.x_end - CONCISE_SHAPE_DELTA),
                fmt_coord(top),
                fmt_coord(seg.x_end),
                fmt_coord(center_y),
                fmt_coord(seg.x_end - CONCISE_SHAPE_DELTA),
                fmt_coord(bottom),
                fmt_coord(seg.x_start + CONCISE_SHAPE_DELTA),
                fmt_coord(bottom),
                fmt_coord(seg.x_start),
                fmt_coord(center_y),
                line_color,
            ));
        }

        let text_width = crate::font_metrics::text_width(&seg.state, "SansSerif", state_fs, true, false);
        let text_x = if index + 1 == track.segments.len() {
            seg.x_start + CONCISE_SHAPE_DELTA
        } else {
            (seg.x_start + seg.x_end) * 0.5 - text_width * 0.5
        };
        label_positions.push((seg.state.clone(), text_x));
    }

    for (state, text_x) in label_positions {
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &state,
            text_x,
            label_y,
            state_fs + 4.0,
            font_color,
            None,
            &format!(r#"font-size="{:.0}" font-weight="bold""#, state_fs),
        );
        sg.push_raw(&tmp);
    }
}

fn arrow_head_points(from_x: f64, from_y: f64, to_x: f64, to_y: f64) -> Option<[(f64, f64); 3]> {
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return None;
    }
    let angle = dx.atan2(dy);
    let delta = 20.0_f64.to_radians();
    let radius = 8.0;
    let p1 = (
        to_x - (angle + delta).sin() * radius,
        to_y - (angle + delta).cos() * radius,
    );
    let p2 = (
        to_x - (angle - delta).sin() * radius,
        to_y - (angle - delta).cos() * radius,
    );
    Some([p1, p2, (to_x, to_y)])
}

fn render_message(sg: &mut SvgGraphic, msg: &TimingMsgLayout, arrow_color: &str, font_color: &str, arrow_fs: f64) {
    sg.push_raw(&format!(
        r#"<line style="stroke:{arrow_color};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(msg.from_x),
        fmt_coord(msg.to_x),
        fmt_coord(msg.from_y),
        fmt_coord(msg.to_y),
    ));
    let Some([(p1x, p1y), (p2x, p2y), (p3x, p3y)]) =
        arrow_head_points(msg.from_x, msg.from_y, msg.to_x, msg.to_y)
    else {
        return;
    };
    sg.push_raw(&format!(
        r#"<polygon fill="{}" points="{},{},{},{},{},{}" style="stroke:{};stroke-width:1.5;"/>"#,
        arrow_color,
        fmt_coord(p1x),
        fmt_coord(p1y),
        fmt_coord(p2x),
        fmt_coord(p2y),
        fmt_coord(p3x),
        fmt_coord(p3y),
        arrow_color,
    ));
    if !msg.label.is_empty() {
        let mut text_y = (p1y + p2y) * 0.5;
        if msg.from_y < msg.to_y {
            text_y -= crate::font_metrics::line_height("SansSerif", arrow_fs, false, false);
        }
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &msg.label,
            (p1x + p2x) * 0.5,
            text_y + crate::font_metrics::ascent("SansSerif", arrow_fs, false, false),
            arrow_fs + 4.0,
            font_color,
            None,
            &format!(r#"font-size="{:.0}""#, arrow_fs),
        );
        sg.push_raw(&tmp);
    }
}

fn render_constraint(
    sg: &mut SvgGraphic,
    c: &TimingConstraintLayout,
    line_color: &str,
    font_color: &str,
    constraint_fs: f64,
) {
    let long_enough = c.x_end - c.x_start > 20.0;
    let (line_start, line_end) = if long_enough {
        (c.x_start + 3.0, c.x_end - 3.0)
    } else {
        (c.x_start - 1.0, c.x_end + 1.0)
    };
    sg.push_raw(&format!(
        r#"<line style="stroke:{line_color};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(line_start),
        fmt_coord(line_end),
        fmt_coord(c.y),
        fmt_coord(c.y),
    ));
    for &(tip_x, dir) in &[(c.x_start, 1.0_f64), (c.x_end, -1.0_f64)] {
        let p1x = tip_x + dir * 8.0;
        let p1y = c.y + 4.0;
        let p2x = tip_x + dir * 8.0;
        let p2y = c.y - 4.0;
        sg.push_raw(&format!(
            r#"<polygon fill="{}" points="{},{},{},{},{},{}" style="stroke:{};stroke-width:1;"/>"#,
            line_color,
            fmt_coord(p1x),
            fmt_coord(p1y),
            fmt_coord(p2x),
            fmt_coord(p2y),
            fmt_coord(tip_x),
            fmt_coord(c.y),
            line_color,
        ));
    }
    let text_width = crate::font_metrics::text_width(&c.label, "SansSerif", constraint_fs, false, false);
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &c.label,
        c.x_start + (c.x_end - c.x_start - text_width) * 0.5,
        c.y - (crate::font_metrics::line_height("SansSerif", constraint_fs, false, false) + 5.0)
            + crate::font_metrics::ascent("SansSerif", constraint_fs, false, false),
        constraint_fs + 4.0,
        font_color,
        None,
        &format!(r#"font-size="{:.0}""#, constraint_fs),
    );
    sg.push_raw(&tmp);
}

fn render_time_axis(sg: &mut SvgGraphic, axis: &TimingTimeAxis, font_color: &str, axis_fs: f64) {
    for tick in &axis.grid_ticks {
        sg.set_stroke_color(Some(AXIS_LINE_COLOR));
        sg.set_stroke_width(2.0, None);
        sg.svg_line(tick.x, axis.y, tick.x, axis.y + 5.0, 0.0);
    }
    if let (Some(first), Some(last)) = (axis.grid_ticks.first(), axis.grid_ticks.last()) {
        sg.set_stroke_color(Some(AXIS_LINE_COLOR));
        sg.set_stroke_width(2.0, None);
        sg.svg_line(first.x, axis.y, last.x, axis.y, 0.0);
    }
    for tick in &axis.ticks {
        let ly = axis.y + 6.0 + crate::font_metrics::ascent("SansSerif", axis_fs, false, false);
        let text_width =
            crate::font_metrics::text_width(&tick.label, "SansSerif", axis_fs, false, false);
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &tick.label,
            tick.x - text_width * 0.5,
            ly,
            axis_fs + 4.0,
            font_color,
            None,
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
        let is_robust = segments.first().map(|seg| seg.is_robust).unwrap_or(false);
        TimingTrackLayout {
            name: name.to_string(),
            y,
            height,
            is_robust,
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
        assert!(svg.contains("DNS"));
        assert!(svg.contains("Idle"));
        assert!(svg.contains(r#"stroke:#006400;stroke-width:2;"#));
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
        assert!(svg.contains(r#"stroke:#333333;stroke-width:0.5;"#));
        assert!(!svg.contains("opacity=\"0.30000\""));
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
        assert!(svg.starts_with("<?plantuml "));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("DNS Resolver"));
        assert!(svg.contains("Web User"));
        assert!(svg.contains("URL"));
        assert!(svg.contains("{150 ms}"));
        assert!(svg.contains("0"));
        assert!(svg.contains("bold"));
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
        assert!(svg.contains("font-weight"));
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
        assert!(svg.contains(&format!(r#"<text fill="{}""#, CONSTRAINT_COLOR)));
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
        assert!(svg.contains("font-weight"));
    }
}
