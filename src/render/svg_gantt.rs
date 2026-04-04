use std::collections::HashMap;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::gantt::{
    GanttBarLayout, GanttDepLayout, GanttLayout, GanttNoteLayout, GanttTimeAxis,
};
use crate::model::gantt::GanttDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

const DEFAULT_FONT_SIZE: f64 = 12.0;
const DEFAULT_BAR_FILL: &str = "#A4C2F4";
const DEFAULT_BAR_STROKE: &str = "#3D85C6";
const ARROW_COLOR: &str = "#555555";
const GRID_COLOR: &str = "#DDDDDD";
const AXIS_TEXT_COLOR: &str = "#333333";
const LABEL_PADDING: f64 = 8.0;
const CALENDAR_CLOSED_BG: &str = "#F1E5E5";
const CALENDAR_DEFAULT_FILL: &str = "#E2E2F0";
const CALENDAR_DEFAULT_STROKE: &str = "#181818";
const CALENDAR_LINE_COLOR: &str = "#C0C0C0";
const CALENDAR_ARROW_COLOR: &str = "#181818";
use crate::skin::rose::{NOTE_BG, NOTE_BORDER, NOTE_FOLD, TEXT_COLOR};

pub fn render_gantt(
    diagram: &GanttDiagram,
    layout: &GanttLayout,
    skin: &SkinParams,
) -> Result<String> {
    if let Some(svg) = render_weekly_calendar_gantt(diagram, skin) {
        return Ok(svg);
    }

    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = ensure_visible_int(layout.width) as f64;
    let svg_h = ensure_visible_int(layout.height) as f64;
    write_svg_root_bg(&mut buf, svg_w, svg_h, "GANTT", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);

    let font_size = layout.font_size;
    let mut sg = SvgGraphic::new(0, 1.0);
    render_grid(&mut sg, layout);
    render_time_axis(&mut sg, &layout.time_axis, font_size);
    let gantt_font = skin.font_color("gantt", TEXT_COLOR);
    for bar in &layout.bars {
        render_bar(&mut sg, bar, gantt_font, font_size);
    }
    for dep in &layout.dependencies {
        render_dependency(&mut sg, dep);
    }
    for note in &layout.notes {
        render_note(&mut sg, note, gantt_font, font_size);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_grid(sg: &mut SvgGraphic, layout: &GanttLayout) {
    sg.set_stroke_color(Some(GRID_COLOR));
    sg.set_stroke_width(0.5, None);
    for label in &layout.time_axis.labels {
        sg.svg_line(label.x, layout.time_axis.y, label.x, layout.height, 0.0);
    }
}

fn render_time_axis(sg: &mut SvgGraphic, axis: &GanttTimeAxis, font_size: f64) {
    let axis_fs = font_size - 1.0;
    for label in &axis.labels {
        let tl = font_metrics::text_width(&label.text, "SansSerif", axis_fs, false, false);
        sg.set_fill_color(AXIS_TEXT_COLOR);
        sg.svg_text(
            &label.text,
            label.x,
            axis.y + font_size + 2.0,
            Some("sans-serif"),
            axis_fs,
            None,
            None,
            None,
            tl,
            LengthAdjust::Spacing,
            None,
            0,
            Some("middle"),
        );
    }
}

fn render_bar(sg: &mut SvgGraphic, bar: &GanttBarLayout, font_color: &str, font_size: f64) {
    let fill = bar.color.as_ref().map_or(DEFAULT_BAR_FILL, |c| {
        if let Some(p) = c.find('/') {
            &c[..p]
        } else {
            c.as_str()
        }
    });
    let stroke = bar.color.as_ref().map_or(DEFAULT_BAR_STROKE, |c| {
        if let Some(p) = c.find('/') {
            &c[p + 1..]
        } else {
            DEFAULT_BAR_STROKE
        }
    });
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(stroke));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(bar.x, bar.y, bar.width, bar.height, 3.0, 3.0, 0.0);
    let label_x = bar.x - LABEL_PADDING;
    let label_y = bar.y + bar.height / 2.0 + font_size * 0.35;
    let fs_int = font_size as u32;
    let font_size_attr = format!(r#"font-size="{fs_int}""#);
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &bar.label,
        label_x,
        label_y,
        font_size + 4.0,
        font_color,
        Some("end"),
        &font_size_attr,
    );
    sg.push_raw(&tmp);
}

fn render_dependency(sg: &mut SvgGraphic, dep: &GanttDepLayout) {
    if dep.points.is_empty() {
        return;
    }
    if dep.points.len() == 2 {
        let (x1, y1) = dep.points[0];
        let (x2, y2) = dep.points[1];
        sg.set_stroke_color(Some(ARROW_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_line(x1, y1, x2, y2, 0.0);
    } else {
        let flat: Vec<f64> = dep.points.iter().flat_map(|(px, py)| [*px, *py]).collect();
        sg.set_fill_color("none");
        sg.set_stroke_color(Some(ARROW_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_polyline(&flat);
    }
    if dep.points.len() >= 2 {
        let (tx, ty) = dep.points[dep.points.len() - 1];
        let (fx, fy) = dep.points[dep.points.len() - 2];
        let dx = tx - fx;
        let dy = ty - fy;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            let ux = dx / len;
            let uy = dy / len;
            let px = -uy;
            let py = ux;
            let p1x = tx - ux * 9.0 + px * 4.0;
            let p1y = ty - uy * 9.0 + py * 4.0;
            let p3x = tx - ux * 9.0 - px * 4.0;
            let p3y = ty - uy * 9.0 - py * 4.0;
            sg.set_fill_color(ARROW_COLOR);
            sg.set_stroke_color(Some(ARROW_COLOR));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[p1x, p1y, tx, ty, p3x, p3y, p1x, p1y]);
        }
    }
}

fn render_note(sg: &mut SvgGraphic, note: &GanttNoteLayout, font_color: &str, font_size: f64) {
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
    sg.push_raw("\n");
    let note_fs_int = (font_size + 1.0) as u32;
    let note_font_size_attr = format!(r#"font-size="{note_fs_int}""#);
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        note.x + 6.0,
        note.y + NOTE_FOLD + font_size,
        font_size + 4.0,
        font_color,
        None,
        &note_font_size_attr,
    );
    sg.push_raw(&tmp);
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct CalendarDate {
    year: i32,
    month: u32,
    day: u32,
}

impl CalendarDate {
    fn parse(input: &str) -> Option<Self> {
        let mut parts = input.split(['-', '/']);
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        Some(Self { year, month, day })
    }

    fn add_days(self, delta: i32) -> Self {
        civil_from_days(self.days_since_epoch() + delta)
    }

    fn days_since_epoch(self) -> i32 {
        days_from_civil(self.year, self.month, self.day)
    }

    fn weekday_monday(self) -> u32 {
        ((self.days_since_epoch() + 3).rem_euclid(7) + 1) as u32
    }

    fn iso_week(self) -> u32 {
        let this_monday = self.add_days(1 - self.weekday_monday() as i32);
        let thursday = self.add_days(4 - self.weekday_monday() as i32);
        let jan4 = Self {
            year: thursday.year,
            month: 1,
            day: 4,
        };
        let week1_monday = jan4.add_days(1 - jan4.weekday_monday() as i32);
        ((this_monday.days_since_epoch() - week1_monday.days_since_epoch()) / 7 + 1) as u32
    }

    fn short_month(self) -> &'static str {
        match self.month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "",
        }
    }

    fn month_label(self) -> String {
        format!("{} {}", self.short_month(), self.year)
    }
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(days: i32) -> CalendarDate {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    CalendarDate {
        year: year + if month <= 2 { 1 } else { 0 },
        month: month as u32,
        day: day as u32,
    }
}

fn day_matches_name(day: CalendarDate, name: &str) -> bool {
    matches!(
        (day.weekday_monday(), name),
        (1, "monday")
            | (2, "tuesday")
            | (3, "wednesday")
            | (4, "thursday")
            | (5, "friday")
            | (6, "saturday")
            | (7, "sunday")
    )
}

fn add_open_days(start: CalendarDate, days: u32, closed_days: &[String]) -> CalendarDate {
    let mut current = start;
    let mut remaining = days;
    while remaining > 0 {
        if !closed_days.iter().any(|name| day_matches_name(current, name)) {
            remaining -= 1;
        }
        current = current.add_days(1);
    }
    current
}

fn normalized_color(name: &str) -> String {
    crate::style::normalize_color(name)
}

fn task_fill_and_stroke(color_spec: Option<&str>) -> (String, String) {
    if let Some(color_spec) = color_spec {
        if let Some((fill, stroke)) = color_spec.split_once('/') {
            return (normalized_color(fill), normalized_color(stroke));
        }
        return (normalized_color(color_spec), CALENDAR_DEFAULT_STROKE.to_string());
    }
    (
        CALENDAR_DEFAULT_FILL.to_string(),
        CALENDAR_DEFAULT_STROKE.to_string(),
    )
}

#[derive(Debug)]
struct CalendarBarSegment {
    rect_x: f64,
    rect_y: f64,
    rect_w: f64,
    rect_h: f64,
    visible_start: f64,
    visible_end: f64,
    fill: String,
    stroke: String,
    kind: CalendarSegmentKind,
}

#[derive(Clone, Copy, Debug)]
enum CalendarSegmentKind {
    Start,
    Middle,
    End,
    Single,
}

#[derive(Debug)]
struct CalendarBar {
    segments: Vec<CalendarBarSegment>,
    label: String,
    label_x: f64,
    label_y: f64,
    label_w: f64,
    bar_start: f64,
    bar_end: f64,
    bar_top: f64,
    bar_bottom: f64,
}

#[derive(Debug)]
struct CalendarDependency {
    x1: f64,
    y1: f64,
    y2: f64,
    x2: f64,
    head_tip_x: f64,
}

fn render_weekly_calendar_gantt(diagram: &GanttDiagram, skin: &SkinParams) -> Option<String> {
    if !diagram
        .print_scale
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("weekly"))
    {
        return None;
    }
    let project_start = CalendarDate::parse(diagram.project_start.as_deref()?)?;
    if diagram.tasks.is_empty() {
        return None;
    }

    let scale_factor = diagram.scale.unwrap_or(1) as f64;
    let day_width = 4.0 * scale_factor;
    let task_font = 11.0 * scale_factor;
    let day_font = 10.0 * scale_factor;
    let month_font = 12.0 * scale_factor;
    let task_ascent = font_metrics::ascent("SansSerif", task_font, false, false);
    let day_ascent = font_metrics::ascent("SansSerif", day_font, false, false);
    let month_ascent = font_metrics::ascent("SansSerif", month_font, true, false);
    let task_height = font_metrics::line_height("SansSerif", task_font, false, false);
    let row_height = task_height + 8.0;
    let month_band_h = month_font + 8.0;
    let header_bottom_y = month_band_h + day_font + 2.0;
    let footer_height = month_band_h;
    let footer_top_y = header_bottom_y + row_height * diagram.tasks.len() as f64;
    let footer_bottom_y = footer_top_y + footer_height;
    let chart_height = footer_top_y - header_bottom_y;
    let one_second_width = day_width / 86400.0;

    let name_to_id: HashMap<String, String> = diagram
        .tasks
        .iter()
        .map(|task| {
            (
                task.name.clone(),
                task.alias
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| task.name.clone()),
            )
        })
        .collect();

    let mut starts: HashMap<String, CalendarDate> = diagram
        .tasks
        .iter()
        .map(|task| {
            let id = task.alias.as_ref().cloned().unwrap_or_else(|| task.name.clone());
            let start = task
                .start_date
                .as_deref()
                .and_then(CalendarDate::parse)
                .unwrap_or(project_start);
            (id, start)
        })
        .collect();

    for _ in 0..=diagram.tasks.len() {
        let ends: HashMap<String, CalendarDate> = diagram
            .tasks
            .iter()
            .map(|task| {
                let id = task.alias.as_ref().cloned().unwrap_or_else(|| task.name.clone());
                let start = starts.get(&id).copied().unwrap_or(project_start);
                (id, add_open_days(start, task.duration_days, &diagram.closed_days))
            })
            .collect();

        let mut changed = false;
        for dep in &diagram.dependencies {
            let from_id = name_to_id.get(&dep.from).cloned().unwrap_or_else(|| dep.from.clone());
            let to_id = name_to_id.get(&dep.to).cloned().unwrap_or_else(|| dep.to.clone());
            let from_end = ends.get(&from_id).copied()?;
            let current_to = starts.get(&to_id).copied().unwrap_or(project_start);
            if from_end > current_to {
                starts.insert(to_id, from_end);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let ends: HashMap<String, CalendarDate> = diagram
        .tasks
        .iter()
        .map(|task| {
            let id = task.alias.as_ref().cloned().unwrap_or_else(|| task.name.clone());
            let start = starts.get(&id).copied().unwrap_or(project_start);
            (id, add_open_days(start, task.duration_days, &diagram.closed_days))
        })
        .collect();

    let mut max_day = project_start;
    for end in ends.values() {
        max_day = max_day.max(end.add_days(-1));
    }
    for range in &diagram.colored_ranges {
        if let Some(range_end) = CalendarDate::parse(&range.end) {
            max_day = max_day.max(range_end);
        }
    }

    let x_of = |date: CalendarDate| -> f64 {
        (date.days_since_epoch() - project_start.days_since_epoch()) as f64 * day_width
    };
    let chart_end_x = x_of(max_day.add_days(1));
    let chart_line_end_x = chart_end_x - one_second_width;

    let mut backgrounds: Vec<(String, f64, f64)> = Vec::new();
    let mut pending_color: Option<String> = None;
    let mut pending_start = project_start;
    let mut day = project_start;
    while day <= max_day {
        let mut color = None;
        for range in &diagram.colored_ranges {
            let start = CalendarDate::parse(&range.start)?;
            let end = CalendarDate::parse(&range.end)?;
            if day >= start && day <= end {
                color = Some(normalized_color(&range.color));
                break;
            }
        }
        if color.is_none()
            && diagram
                .closed_days
                .iter()
                .any(|name| day_matches_name(day, name))
        {
            color = Some(CALENDAR_CLOSED_BG.to_string());
        }

        if color != pending_color {
            if let Some(prev_color) = pending_color.take() {
                backgrounds.push((prev_color, x_of(pending_start), x_of(day) - x_of(pending_start)));
            }
            pending_start = day;
            pending_color = color.clone();
        }
        day = day.add_days(1);
    }
    if let Some(prev_color) = pending_color {
        backgrounds.push((
            prev_color,
            x_of(pending_start),
            x_of(max_day.add_days(1)) - x_of(pending_start),
        ));
    }

    let mut week_lines = Vec::new();
    let mut week_labels = Vec::new();
    let mut day = project_start;
    while day <= max_day {
        if day.weekday_monday() == 1 {
            let x = x_of(day);
            week_lines.push(x);
            let label = day.iso_week().to_string();
            week_labels.push((label, x + 10.0));
        }
        day = day.add_days(1);
    }

    let mut month_spans = Vec::new();
    let mut span_start_day = project_start;
    let mut day = project_start;
    while day <= max_day {
        if day.month != span_start_day.month || day.year != span_start_day.year {
            month_spans.push((span_start_day.month_label(), x_of(span_start_day), x_of(day)));
            span_start_day = day;
        }
        day = day.add_days(1);
    }
    month_spans.push((span_start_day.month_label(), x_of(span_start_day), chart_end_x));

    let mut bars = Vec::new();
    let mut task_bar_lookup: HashMap<String, usize> = HashMap::new();
    for (index, task) in diagram.tasks.iter().enumerate() {
        let id = task.alias.as_ref().cloned().unwrap_or_else(|| task.name.clone());
        let start = starts.get(&id).copied().unwrap_or(project_start);
        let end = ends.get(&id).copied().unwrap_or(start);
        let bar_start = x_of(start) + 4.0;
        let bar_end = x_of(end) - 4.0;
        let row_y = header_bottom_y + index as f64 * row_height;
        let bar_top = row_y + 4.0;
        let bar_bottom = bar_top + task_height;
        let (fill, stroke) = task_fill_and_stroke(task.color.as_deref());

        let mut runs = Vec::new();
        let mut run_start = None;
        let mut day = start;
        while day < end {
            let is_open = !diagram
                .closed_days
                .iter()
                .any(|name| day_matches_name(day, name));
            if is_open {
                if run_start.is_none() {
                    run_start = Some(day);
                }
            } else if let Some(open_start) = run_start.take() {
                runs.push((open_start, day));
            }
            day = day.add_days(1);
        }
        if let Some(open_start) = run_start {
            runs.push((open_start, end));
        }

        let mut segments = Vec::new();
        for (run_index, (run_start, run_end)) in runs.iter().enumerate() {
            let visible_start = if run_index == 0 {
                bar_start
            } else {
                x_of(*run_start)
            };
            let visible_end = if run_index + 1 == runs.len() {
                bar_end
            } else {
                x_of(*run_end)
            };
            let kind = if runs.len() == 1 {
                CalendarSegmentKind::Single
            } else if run_index == 0 {
                CalendarSegmentKind::Start
            } else if run_index + 1 == runs.len() {
                CalendarSegmentKind::End
            } else {
                CalendarSegmentKind::Middle
            };
            let rect_w = if matches!(kind, CalendarSegmentKind::End | CalendarSegmentKind::Single) {
                visible_end - visible_start
            } else {
                visible_end - visible_start + 2.0
            };
            segments.push(CalendarBarSegment {
                rect_x: visible_start,
                rect_y: bar_top,
                rect_w,
                rect_h: task_height,
                visible_start,
                visible_end,
                fill: fill.clone(),
                stroke: stroke.clone(),
                kind,
            });
        }

        let label_w = font_metrics::text_width(&task.name, "SansSerif", task_font, false, false);
        let available = (bar_end - bar_start - 12.0).max(0.0);
        let label_x = if available > label_w {
            bar_start + 8.0
        } else {
            bar_end + 8.0
        };
        let label_y = row_y + 4.0 + task_ascent;

        task_bar_lookup.insert(id, bars.len());
        bars.push(CalendarBar {
            segments,
            label: task.name.clone(),
            label_x,
            label_y,
            label_w,
            bar_start,
            bar_end,
            bar_top,
            bar_bottom,
        });
    }

    let mut deps = Vec::new();
    for dep in &diagram.dependencies {
        let from_id = name_to_id.get(&dep.from).cloned().unwrap_or_else(|| dep.from.clone());
        let to_id = name_to_id.get(&dep.to).cloned().unwrap_or_else(|| dep.to.clone());
        let from_bar = bars.get(*task_bar_lookup.get(&from_id)? )?;
        let to_bar = bars.get(*task_bar_lookup.get(&to_id)? )?;
        deps.push(CalendarDependency {
            x1: from_bar.bar_end - 12.0,
            y1: from_bar.bar_bottom,
            y2: to_bar.bar_top + task_height / 2.0,
            x2: to_bar.bar_start - 10.0,
            head_tip_x: to_bar.bar_start - 4.0,
        });
    }

    let mut max_x = chart_end_x;
    for bar in &bars {
        max_x = max_x.max(bar.label_x + bar.label_w);
    }
    let raw_width = max_x;
    let raw_height = footer_bottom_y + 2.0;

    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = ensure_visible_int(raw_width) as f64;
    let svg_h = ensure_visible_int(raw_height) as f64;

    let mut buf = String::with_capacity(8192);
    write_svg_root_bg(&mut buf, svg_w, svg_h, "GANTT", bg);
    buf.push_str("<defs/><g>");

    for (color, x, width) in backgrounds {
        buf.push_str(&format!(
            r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:2;" width="{}" x="{}" y="{}"/>"#,
            color,
            fmt_coord(chart_height),
            fmt_coord(width),
            fmt_coord(x),
            fmt_coord(header_bottom_y),
        ));
    }

    for (label, x) in week_labels {
        let text_w = font_metrics::text_width(&label, "SansSerif", day_font, false, false);
        buf.push_str(&format!(
            r##"<text fill="#000000" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"##,
            fmt_coord(day_font),
            fmt_coord(text_w),
            fmt_coord(x),
            fmt_coord(month_band_h + day_ascent),
            xml_escape(&label),
        ));
    }

    for x in &week_lines {
        buf.push_str(&format!(
            r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(*x),
            fmt_coord(*x),
            fmt_coord(month_band_h),
            fmt_coord(footer_top_y),
        ));
    }
    buf.push_str(&format!(
        r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(chart_end_x),
        fmt_coord(chart_end_x),
        fmt_coord(month_band_h),
        fmt_coord(footer_top_y),
    ));

    if let Some((_, start_x, _)) = month_spans.first() {
        buf.push_str(&format!(
            r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="{}" x2="{}" y1="0" y2="{}"/>"#,
            fmt_coord(*start_x),
            fmt_coord(*start_x),
            fmt_coord(month_band_h),
        ));
    }
    for (label, start_x, end_x) in &month_spans {
        buf.push_str(&format!(
            r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="{}" x2="{}" y1="0" y2="{}"/>"#,
            fmt_coord(*end_x),
            fmt_coord(*end_x),
            fmt_coord(month_band_h),
        ));
        let text_w = font_metrics::text_width(label, "SansSerif", month_font, true, false);
        let text_x = *start_x + (*end_x - *start_x - text_w).max(0.0) / 2.0;
        buf.push_str(&format!(
            r##"<text fill="#000000" font-family="sans-serif" font-size="{}" font-weight="700" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"##,
            fmt_coord(month_font),
            fmt_coord(text_w),
            fmt_coord(text_x),
            fmt_coord(month_ascent),
            xml_escape(label),
        ));
    }
    buf.push_str(&format!(
        r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="0" x2="{}" y1="0" y2="0"/>"#,
        fmt_coord(chart_line_end_x),
    ));
    buf.push_str(&format!(
        r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="0" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(chart_line_end_x),
        fmt_coord(month_band_h),
        fmt_coord(month_band_h),
    ));
    buf.push_str(&format!(
        r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="0" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(chart_line_end_x),
        fmt_coord(header_bottom_y),
        fmt_coord(header_bottom_y),
    ));

    for dep in &deps {
        buf.push_str(&format!(
            r#"<path d="M{},{} L{},{} L{},{}" fill="none" style="stroke:{CALENDAR_ARROW_COLOR};stroke-width:3;"/>"#,
            fmt_coord(dep.x1),
            fmt_coord(dep.y1),
            fmt_coord(dep.x1),
            fmt_coord(dep.y2),
            fmt_coord(dep.x2),
            fmt_coord(dep.y2),
        ));
        buf.push_str(&format!(
            r#"<polygon fill="{CALENDAR_ARROW_COLOR}" points="{},{},{},{},{},{},{}" style="stroke:{CALENDAR_ARROW_COLOR};stroke-width:2;"/>"#,
            fmt_coord(dep.head_tip_x - 8.0),
            fmt_coord(dep.y2 - 8.0),
            fmt_coord(dep.head_tip_x),
            fmt_coord(dep.y2),
            fmt_coord(dep.head_tip_x - 8.0),
            fmt_coord(dep.y2 + 8.0),
            format!("{},{}", fmt_coord(dep.head_tip_x - 8.0), fmt_coord(dep.y2 - 8.0)),
        ));
    }

    for bar in &bars {
        for segment in &bar.segments {
            buf.push_str(&format!(
                r#"<rect fill="{}" height="{}" style="stroke:none;stroke-width:2;" width="{}" x="{}" y="{}"/>"#,
                segment.fill,
                fmt_coord(segment.rect_h),
                fmt_coord(segment.rect_w),
                fmt_coord(segment.rect_x),
                fmt_coord(segment.rect_y),
            ));
            match segment.kind {
                CalendarSegmentKind::Start => {
                    buf.push_str(&format!(
                        r#"<path d="M{},{} L{},{} L{},{} L{},{}" fill="none" style="stroke:{};stroke-width:2;"/>"#,
                        fmt_coord(segment.visible_end),
                        fmt_coord(bar.bar_bottom),
                        fmt_coord(segment.visible_start),
                        fmt_coord(bar.bar_bottom),
                        fmt_coord(segment.visible_start),
                        fmt_coord(bar.bar_top),
                        fmt_coord(segment.visible_end),
                        fmt_coord(bar.bar_top),
                        segment.stroke,
                    ));
                }
                CalendarSegmentKind::Middle => {
                    buf.push_str(&format!(
                        r#"<line style="stroke:{};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                        segment.stroke,
                        fmt_coord(segment.visible_start),
                        fmt_coord(segment.visible_end),
                        fmt_coord(bar.bar_top),
                        fmt_coord(bar.bar_top),
                    ));
                    buf.push_str(&format!(
                        r#"<line style="stroke:{};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                        segment.stroke,
                        fmt_coord(segment.visible_start),
                        fmt_coord(segment.visible_end),
                        fmt_coord(bar.bar_bottom),
                        fmt_coord(bar.bar_bottom),
                    ));
                }
                CalendarSegmentKind::End | CalendarSegmentKind::Single => {
                    buf.push_str(&format!(
                        r#"<path d="M{},{} L{},{} L{},{} L{},{}" fill="none" style="stroke:{};stroke-width:2;"/>"#,
                        fmt_coord(segment.visible_start),
                        fmt_coord(bar.bar_top),
                        fmt_coord(segment.visible_end),
                        fmt_coord(bar.bar_top),
                        fmt_coord(segment.visible_end),
                        fmt_coord(bar.bar_bottom),
                        fmt_coord(segment.visible_start),
                        fmt_coord(bar.bar_bottom),
                        segment.stroke,
                    ));
                }
            }
        }

        for pair in bar.segments.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            let dash_x1 = left.visible_end + 6.0;
            let dash_x2 = right.visible_start - 6.0;
            if dash_x2 > dash_x1 {
                buf.push_str(&format!(
                    r#"<line style="stroke:{};stroke-width:2;stroke-dasharray:4,6;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                    left.stroke,
                    fmt_coord(dash_x1),
                    fmt_coord(dash_x2),
                    fmt_coord(bar.bar_top),
                    fmt_coord(bar.bar_top),
                ));
                buf.push_str(&format!(
                    r#"<line style="stroke:{};stroke-width:2;stroke-dasharray:4,6;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                    left.stroke,
                    fmt_coord(dash_x1),
                    fmt_coord(dash_x2),
                    fmt_coord(bar.bar_bottom),
                    fmt_coord(bar.bar_bottom),
                ));
            }
        }
    }

    for bar in &bars {
        buf.push_str(&format!(
            r##"<text fill="#000000" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"##,
            fmt_coord(task_font),
            fmt_coord(bar.label_w),
            fmt_coord(bar.label_x),
            fmt_coord(bar.label_y),
            xml_escape(&bar.label),
        ));
    }

    buf.push_str(&format!(
        r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="0" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(chart_line_end_x),
        fmt_coord(footer_top_y),
        fmt_coord(footer_top_y),
    ));
    if let Some((_, start_x, _)) = month_spans.first() {
        buf.push_str(&format!(
            r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(*start_x),
            fmt_coord(*start_x),
            fmt_coord(footer_top_y),
            fmt_coord(footer_bottom_y),
        ));
    }
    for (label, start_x, end_x) in &month_spans {
        buf.push_str(&format!(
            r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(*end_x),
            fmt_coord(*end_x),
            fmt_coord(footer_top_y),
            fmt_coord(footer_bottom_y),
        ));
        let text_w = font_metrics::text_width(label, "SansSerif", month_font, true, false);
        let text_x = *start_x + (*end_x - *start_x - text_w).max(0.0) / 2.0;
        buf.push_str(&format!(
            r##"<text fill="#000000" font-family="sans-serif" font-size="{}" font-weight="700" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"##,
            fmt_coord(month_font),
            fmt_coord(text_w),
            fmt_coord(text_x),
            fmt_coord(footer_top_y + month_ascent),
            xml_escape(label),
        ));
    }
    buf.push_str(&format!(
        r#"<line style="stroke:{CALENDAR_LINE_COLOR};stroke-width:2;" x1="0" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(chart_line_end_x),
        fmt_coord(footer_bottom_y),
        fmt_coord(footer_bottom_y),
    ));
    buf.push_str("</g></svg>");
    Some(buf)
}

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
            font_size: DEFAULT_FONT_SIZE,
        }
    }
    fn make_bar(id: &str, label: &str, x: f64, y: f64, w: f64) -> GanttBarLayout {
        GanttBarLayout {
            id: id.into(),
            label: label.into(),
            x,
            y,
            width: w,
            height: 20.0,
            color: None,
        }
    }

    #[test]
    fn test_empty_svg() {
        let svg = render_gantt(&empty_model(), &empty_layout(), &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }
    #[test]
    fn test_defs_empty() {
        let svg = render_gantt(&empty_model(), &empty_layout(), &SkinParams::default()).unwrap();
        assert!(svg.contains("<defs/>"));
    }
    #[test]
    fn test_single_bar() {
        let mut l = empty_layout();
        l.bars
            .push(make_bar("Design", "Design", 180.0, 50.0, 200.0));
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("Design"));
        assert!(svg.contains(r##"fill="#A4C2F4""##));
        assert!(svg.contains("stroke:#3D85C6"));
    }
    #[test]
    fn test_bar_with_color() {
        let mut l = empty_layout();
        let mut b = make_bar("T1", "Task 1", 180.0, 50.0, 100.0);
        b.color = Some("Lavender/LightBlue".into());
        l.bars.push(b);
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"fill="Lavender""#));
        assert!(svg.contains("stroke:LightBlue"));
    }
    #[test]
    fn test_bar_single_color() {
        let mut l = empty_layout();
        let mut b = make_bar("T1", "Task 1", 180.0, 50.0, 100.0);
        b.color = Some("salmon".into());
        l.bars.push(b);
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"fill="salmon""#));
    }
    #[test]
    fn test_time_axis_labels() {
        let mut l = empty_layout();
        l.time_axis.labels.push(GanttTimeLabel {
            text: "W1".into(),
            x: 200.0,
        });
        l.time_axis.labels.push(GanttTimeLabel {
            text: "W2".into(),
            x: 340.0,
        });
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("W1"));
        assert!(svg.contains("W2"));
    }
    #[test]
    fn test_grid_lines() {
        let mut l = empty_layout();
        l.time_axis.labels.push(GanttTimeLabel {
            text: "D1".into(),
            x: 200.0,
        });
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("stroke:#DDDDDD"));
    }
    #[test]
    fn test_dependency_2point() {
        let mut l = empty_layout();
        l.dependencies.push(GanttDepLayout {
            from: "A".into(),
            to: "B".into(),
            points: vec![(100.0, 60.0), (200.0, 90.0)],
        });
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<line "));
        assert!(svg.contains("<polygon"));
    }
    #[test]
    fn test_dependency_polyline() {
        let mut l = empty_layout();
        l.dependencies.push(GanttDepLayout {
            from: "A".into(),
            to: "B".into(),
            points: vec![(100.0, 60.0), (150.0, 60.0), (150.0, 90.0), (200.0, 90.0)],
        });
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polyline"));
        assert!(svg.contains("<polygon"));
    }
    #[test]
    fn test_empty_dependency_points() {
        let mut l = empty_layout();
        l.dependencies.push(GanttDepLayout {
            from: "A".into(),
            to: "B".into(),
            points: vec![],
        });
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(!svg.contains("<line x1="));
        assert!(!svg.contains("<polyline"));
    }
    #[test]
    fn test_label_position() {
        let mut l = empty_layout();
        l.bars.push(make_bar("T", "My Task", 200.0, 50.0, 100.0));
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"text-anchor="end""#));
        assert!(svg.contains("My Task"));
    }
    #[test]
    fn test_svg_dimensions() {
        let mut l = empty_layout();
        l.width = 600.0;
        l.height = 300.0;
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"width="601px""#));
        assert!(svg.contains(r#"height="301px""#));
        assert!(svg.contains(r#"viewBox="0 0 601 301""#));
    }
    #[test]
    fn test_xml_escaping() {
        let mut l = empty_layout();
        l.bars.push(make_bar("T", "A & B < C", 200.0, 50.0, 100.0));
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("A &amp; B &lt; C"));
    }
    #[test]
    fn test_full_chart() {
        let mut l = empty_layout();
        l.width = 500.0;
        l.height = 200.0;
        l.bars.push(make_bar("A", "Design", 200.0, 50.0, 100.0));
        l.bars.push(make_bar("B", "Build", 300.0, 80.0, 60.0));
        l.time_axis.labels.push(GanttTimeLabel {
            text: "D1".into(),
            x: 200.0,
        });
        l.dependencies.push(GanttDepLayout {
            from: "A".into(),
            to: "B".into(),
            points: vec![(300.0, 60.0), (300.0, 90.0)],
        });
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
        assert_eq!(svg.matches("<rect").count(), 2);
        assert!(svg.contains("Design"));
        assert!(svg.contains("Build"));
        assert!(svg.contains("D1"));
        assert!(svg.matches("<polygon").count() >= 1);
    }
    #[test]
    fn test_bar_rounded_corners() {
        let mut l = empty_layout();
        l.bars.push(make_bar("T", "Task", 200.0, 50.0, 100.0));
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"rx="3""#));
        assert!(svg.contains(r#"ry="3""#));
    }
    #[test]
    fn test_note_rendering() {
        let mut l = empty_layout();
        l.notes.push(GanttNoteLayout {
            text: "**note**".into(),
            x: 320.0,
            y: 40.0,
            width: 90.0,
            height: 42.0,
            connector: Some((300.0, 60.0, 320.0, 55.0)),
        });
        let svg = render_gantt(&empty_model(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.contains("font-weight=\"700\""));
    }

    #[test]
    fn test_weekly_calendar_mode_renders_java_like_header() {
        let mut d = empty_model();
        d.scale = Some(2);
        d.print_scale = Some("weekly".into());
        d.project_start = Some("2020-10-15".into());
        d.closed_days = vec!["sunday".into(), "saturday".into()];
        d.colored_ranges.push(crate::model::gantt::GanttColoredRange {
            start: "2020/10/26".into(),
            end: "2020/11/01".into(),
            color: "salmon".into(),
        });
        d.tasks.push(crate::model::gantt::GanttTask {
            name: "Prototype design".into(),
            alias: Some("TASK1".into()),
            duration_days: 25,
            color: Some("Lavender/LightBlue".into()),
            start_date: None,
        });
        d.tasks.push(crate::model::gantt::GanttTask {
            name: "Testing".into(),
            alias: None,
            duration_days: 5,
            color: None,
            start_date: None,
        });
        d.dependencies.push(crate::model::gantt::GanttDependency {
            from: "TASK1".into(),
            to: "Testing".into(),
        });

        let svg = render_gantt(&d, &empty_layout(), &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"width="422px""#));
        assert!(svg.contains(r#"height="156px""#));
        assert!(svg.contains(">Oct 2020</text>"));
        assert!(svg.contains(">43</text>"));
        assert!(svg.contains(r##"fill="#FA8072""##));
        assert!(svg.contains("Prototype design"));
    }
}
