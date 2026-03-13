use std::fmt::Write;

use crate::layout::timing::{
    TimingConstraintLayout, TimingLayout, TimingMsgLayout, TimingNoteLayout, TimingSegmentLayout,
    TimingTimeAxis, TimingTrackLayout,
};
use crate::model::timing::TimingDiagram;
use crate::render::svg_richtext::{count_creole_lines, render_creole_text};
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
const FONT_FAMILY: &str = "monospace";
const TRACK_BG_FILL: &str = "#FEFECE";
const TRACK_BORDER: &str = "#A80036";
const SIGNAL_STROKE: &str = "#A80036";
const CONCISE_STROKE: &str = "#2E8B57";
const ARROW_COLOR: &str = "#555555";
const CONSTRAINT_COLOR: &str = "#FF6600";
const TEXT_FILL: &str = "#000000";
const AXIS_LINE_COLOR: &str = "#888888";
const AXIS_TEXT_COLOR: &str = "#333333";
const TICK_COLOR: &str = "#CCCCCC";
const LABEL_PADDING: f64 = 8.0;
const ROBUST_BAND_HEIGHT: f64 = 16.0;
const NOTE_BG: &str = "#FBFB77";
const NOTE_BORDER: &str = "#A80036";
const NOTE_FOLD: f64 = 8.0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render a timing diagram to SVG.
pub fn render_timing(
    _td: &TimingDiagram,
    layout: &TimingLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // Skin color lookups
    let timing_bg = skin.background_color("timing", TRACK_BG_FILL);
    let timing_border = skin.border_color("timing", TRACK_BORDER);
    let timing_font = skin.font_color("timing", TEXT_FILL);
    let constraint_color = skin.font_color("constraint", CONSTRAINT_COLOR);
    let arrow_color = skin.arrow_color(ARROW_COLOR);

    // SVG header
    write!(
        buf,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w:.0} {h:.0}" width="{w:.0}" height="{h:.0}" font-family="{FONT_FAMILY}" font-size="{FONT_SIZE}">"#,
        w = layout.width,
        h = layout.height,
    )
    .unwrap();
    buf.push('\n');

    // Defs
    write_defs(&mut buf, arrow_color, constraint_color);

    // Tick grid lines (vertical)
    render_tick_grid(&mut buf, layout);

    // Tracks (participant lanes)
    for track in &layout.tracks {
        render_track(&mut buf, track, timing_bg, timing_border, timing_font);
    }

    // Messages
    for msg in &layout.messages {
        render_message(&mut buf, msg, arrow_color, timing_font);
    }

    // Constraints
    for c in &layout.constraints {
        render_constraint(&mut buf, c, constraint_color);
    }

    for note in &layout.notes {
        render_note(&mut buf, note, timing_font);
    }

    // Time axis
    render_time_axis(&mut buf, &layout.time_axis);

    buf.push_str("</svg>\n");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Defs
// ---------------------------------------------------------------------------

fn write_defs(buf: &mut String, arrow_color: &str, constraint_color: &str) {
    buf.push_str("<defs>\n");
    // Arrow marker for messages
    write!(
        buf,
        concat!(
            r#"<marker id="timing-arrow" viewBox="0 0 10 10" refX="10" refY="5""#,
            r#" markerWidth="8" markerHeight="8" orient="auto-start-reverse">"#,
            r#"<path d="M 0 0 L 10 5 L 0 10 Z" fill="{}" stroke="none"/>"#,
            r#"</marker>"#,
        ),
        arrow_color,
    )
    .unwrap();
    buf.push('\n');
    // Constraint arrow (double-ended)
    write!(
        buf,
        concat!(
            r#"<marker id="timing-constraint-arrow" viewBox="0 0 10 10" refX="10" refY="5""#,
            r#" markerWidth="6" markerHeight="6" orient="auto-start-reverse">"#,
            r#"<path d="M 0 0 L 10 5 L 0 10 Z" fill="{}" stroke="none"/>"#,
            r#"</marker>"#,
        ),
        constraint_color,
    )
    .unwrap();
    buf.push('\n');
    buf.push_str("</defs>\n");
}

// ---------------------------------------------------------------------------
// Tick grid
// ---------------------------------------------------------------------------

fn render_tick_grid(buf: &mut String, layout: &TimingLayout) {
    for tick in &layout.time_axis.ticks {
        write!(
            buf,
            r#"<line x1="{x:.1}" y1="0" x2="{x:.1}" y2="{y2:.1}" stroke="{TICK_COLOR}" stroke-width="0.5" stroke-dasharray="4,4"/>"#,
            x = tick.x,
            y2 = layout.time_axis.y,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ---------------------------------------------------------------------------
// Track rendering
// ---------------------------------------------------------------------------

fn render_track(
    buf: &mut String,
    track: &TimingTrackLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
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

        write!(
            buf,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{bg}" stroke="{border}" stroke-width="0.5" opacity="0.3"/>"#,
            x = x_min,
            y = track.y,
            h = track.height,
        )
        .unwrap();
        buf.push('\n');
    }

    // Participant label
    let label_x = track
        .segments
        .first()
        .map_or(LABEL_PADDING, |s| s.x_start - LABEL_PADDING);
    let label_y = track.y + track.height * 0.5 + FONT_SIZE * 0.35;
    render_creole_text(
        buf,
        &track.name,
        label_x,
        label_y,
        FONT_SIZE + 4.0,
        font_color,
        Some("end"),
        r#"font-weight="bold""#,
    );

    // Segments with state level lines and transitions
    for (i, seg) in track.segments.iter().enumerate() {
        render_segment(buf, seg, i, &track.segments);
    }
}

fn render_segment(
    buf: &mut String,
    seg: &TimingSegmentLayout,
    index: usize,
    all_segments: &[TimingSegmentLayout],
) {
    let stroke = if seg.is_robust {
        SIGNAL_STROKE
    } else {
        CONCISE_STROKE
    };

    if seg.is_robust {
        // Robust: draw a filled band
        let band_top = seg.y - ROBUST_BAND_HEIGHT * 0.5;
        let w = seg.x_end - seg.x_start;
        if w > 0.0 {
            write!(
                buf,
                r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{bh:.0}" fill="{TRACK_BG_FILL}" stroke="{stroke}" stroke-width="1"/>"#,
                x = seg.x_start,
                y = band_top,
                bh = ROBUST_BAND_HEIGHT,
            )
            .unwrap();
            buf.push('\n');
        }

        // State label inside the band
        if w > 10.0 {
            let cx = seg.x_start + w * 0.5;
            let cy = seg.y + FONT_SIZE * 0.35;
            render_creole_text(
                buf,
                &seg.state,
                cx,
                cy,
                FONT_SIZE + 4.0,
                TEXT_FILL,
                Some("middle"),
                &format!(r#"font-size="{:.0}""#, FONT_SIZE - 1.0),
            );
        }

        // Transition: stepped (rectangular) waveform – vertical line at boundary
        if index > 0 {
            let prev = &all_segments[index - 1];
            let trans_x = seg.x_start;
            let prev_band_top = prev.y - ROBUST_BAND_HEIGHT * 0.5;
            let prev_band_bot = prev.y + ROBUST_BAND_HEIGHT * 0.5;
            let cur_band_top = seg.y - ROBUST_BAND_HEIGHT * 0.5;
            let cur_band_bot = seg.y + ROBUST_BAND_HEIGHT * 0.5;
            // Vertical step from previous band edge to current band edge
            let step_y_from = if seg.y < prev.y {
                prev_band_top
            } else {
                prev_band_bot
            };
            let step_y_to = if seg.y < prev.y {
                cur_band_bot
            } else {
                cur_band_top
            };
            write!(
                buf,
                r#"<line x1="{trans_x:.1}" y1="{step_y_from:.1}" x2="{trans_x:.1}" y2="{step_y_to:.1}" stroke="{stroke}" stroke-width="1.5"/>"#,
            )
            .unwrap();
            buf.push('\n');
        }
    } else {
        // Concise: draw a horizontal line at the state level
        if seg.x_end > seg.x_start {
            write!(
                buf,
                r#"<line x1="{x1:.1}" y1="{y:.1}" x2="{x2:.1}" y2="{y:.1}" stroke="{stroke}" stroke-width="1.5"/>"#,
                x1 = seg.x_start,
                y = seg.y,
                x2 = seg.x_end,
            )
            .unwrap();
            buf.push('\n');
        }

        // State label above the line
        if (seg.x_end - seg.x_start) > 10.0 {
            let cx = seg.x_start + (seg.x_end - seg.x_start) * 0.5;
            let cy = seg.y - 4.0;
            render_creole_text(
                buf,
                &seg.state,
                cx,
                cy,
                FONT_SIZE + 4.0,
                TEXT_FILL,
                Some("middle"),
                &format!(r#"font-size="{:.0}""#, FONT_SIZE - 1.0),
            );
        }

        // Transition: vertical line at the boundary
        if index > 0 {
            let prev = &all_segments[index - 1];
            let trans_x = seg.x_start;
            write!(
                buf,
                r#"<line x1="{x:.1}" y1="{y1:.1}" x2="{x:.1}" y2="{y2:.1}" stroke="{stroke}" stroke-width="1"/>"#,
                x = trans_x,
                y1 = prev.y,
                y2 = seg.y,
            )
            .unwrap();
            buf.push('\n');
        }
    }
}

// ---------------------------------------------------------------------------
// Message rendering
// ---------------------------------------------------------------------------

fn render_message(buf: &mut String, msg: &TimingMsgLayout, arrow_color: &str, font_color: &str) {
    write!(
        buf,
        r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{arrow_color}" stroke-width="1" marker-end="url(#timing-arrow)"/>"#,
        x1 = msg.from_x,
        y1 = msg.from_y,
        x2 = msg.to_x,
        y2 = msg.to_y,
    )
    .unwrap();
    buf.push('\n');

    // Label at midpoint
    if !msg.label.is_empty() {
        let mx = (msg.from_x + msg.to_x) * 0.5;
        let my = (msg.from_y + msg.to_y) * 0.5 - 4.0;
        render_creole_text(
            buf,
            &msg.label,
            mx,
            my,
            FONT_SIZE + 4.0,
            font_color,
            Some("middle"),
            &format!(r#"font-size="{:.0}""#, FONT_SIZE - 1.0),
        );
    }
}

// ---------------------------------------------------------------------------
// Constraint rendering
// ---------------------------------------------------------------------------

fn render_constraint(buf: &mut String, c: &TimingConstraintLayout, constraint_color: &str) {
    // Double-ended arrow line
    write!(
        buf,
        r#"<line x1="{x1:.1}" y1="{y:.1}" x2="{x2:.1}" y2="{y:.1}" stroke="{constraint_color}" stroke-width="1" marker-start="url(#timing-constraint-arrow)" marker-end="url(#timing-constraint-arrow)"/>"#,
        x1 = c.x_start,
        y = c.y,
        x2 = c.x_end,
    )
    .unwrap();
    buf.push('\n');

    // Label above the line
    let mx = (c.x_start + c.x_end) * 0.5;
    let my = c.y - 4.0;
    render_creole_text(
        buf,
        &c.label,
        mx,
        my,
        FONT_SIZE + 4.0,
        constraint_color,
        Some("middle"),
        &format!(r#"font-size="{:.0}""#, FONT_SIZE - 1.0),
    );
}

// ---------------------------------------------------------------------------
// Time axis rendering
// ---------------------------------------------------------------------------

fn render_time_axis(buf: &mut String, axis: &TimingTimeAxis) {
    // Horizontal axis line
    if let (Some(first), Some(last)) = (axis.ticks.first(), axis.ticks.last()) {
        write!(
            buf,
            r#"<line x1="{x1:.1}" y1="{y:.1}" x2="{x2:.1}" y2="{y:.1}" stroke="{AXIS_LINE_COLOR}" stroke-width="1"/>"#,
            x1 = first.x,
            y = axis.y,
            x2 = last.x,
        )
        .unwrap();
        buf.push('\n');
    }

    // Tick marks and labels
    for tick in &axis.ticks {
        // Small vertical tick mark
        write!(
            buf,
            r#"<line x1="{x:.1}" y1="{y1:.1}" x2="{x:.1}" y2="{y2:.1}" stroke="{AXIS_LINE_COLOR}" stroke-width="1"/>"#,
            x = tick.x,
            y1 = axis.y,
            y2 = axis.y + 6.0,
        )
        .unwrap();
        buf.push('\n');

        // Label below tick
        let label_y = axis.y + 6.0 + FONT_SIZE + 2.0;
        render_creole_text(
            buf,
            &tick.label,
            tick.x,
            label_y,
            FONT_SIZE + 4.0,
            AXIS_TEXT_COLOR,
            Some("middle"),
            &format!(r#"font-size="{:.0}""#, FONT_SIZE - 1.0),
        );
    }
}

fn render_note(buf: &mut String, note: &TimingNoteLayout, font_color: &str) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        write!(
            buf,
            r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{NOTE_BORDER}" stroke-width="1" stroke-dasharray="4,4"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    let fold_x = note.x + note.width - NOTE_FOLD;
    let fold_y = note.y + NOTE_FOLD;
    write!(
        buf,
        r#"<polygon points="{x:.1},{y:.1} {fx:.1},{y:.1} {x2:.1},{fy:.1} {x2:.1},{y2:.1} {x:.1},{y2:.1}" fill="{NOTE_BG}" stroke="{NOTE_BORDER}" stroke-width="1"/>"#,
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
        r#"<path d="M {fx:.1},{y:.1} L {fx:.1},{fy:.1} L {x2:.1},{fy:.1}" fill="none" stroke="{NOTE_BORDER}" stroke-width="1"/>"#,
        fx = fold_x,
        fy = fold_y,
        x2 = note.x + note.width,
        y = note.y,
    )
    .unwrap();
    buf.push('\n');

    let line_count = count_creole_lines(&note.text) as f64;
    let start_y = note.y
        + NOTE_FOLD
        + (note.height - line_count * (FONT_SIZE + 4.0)).max(0.0) / 2.0
        + FONT_SIZE;
    render_creole_text(
        buf,
        &note.text,
        note.x + 6.0,
        start_y,
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
                ticks: vec![],
            },
            width: 400.0,
            height: 200.0,
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
        }
    }

    // 1. Empty diagram produces valid SVG
    #[test]
    fn test_empty_svg() {
        let model = empty_model();
        let layout = empty_layout();
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    // 2. SVG contains defs with markers
    #[test]
    fn test_defs_markers() {
        let model = empty_model();
        let layout = empty_layout();
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("timing-arrow"));
        assert!(svg.contains("timing-constraint-arrow"));
        assert!(svg.contains("</defs>"));
    }

    // 3. SVG dimensions match layout
    #[test]
    fn test_svg_dimensions() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.width = 600.0;
        layout.height = 300.0;
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"width="600""#));
        assert!(svg.contains(r#"height="300""#));
        assert!(svg.contains(r#"viewBox="0 0 600 300""#));
    }

    // 4. Track with robust segments renders rect bands
    #[test]
    fn test_robust_segments() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track(
            "DNS",
            20.0,
            40.0,
            vec![make_segment("Idle", 200.0, 350.0, 40.0, true)],
        ));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<rect"),
            "robust segments should produce rects"
        );
        assert!(svg.contains("DNS"), "track label must appear");
        assert!(svg.contains("Idle"), "state name must appear");
    }

    // 5. Track with concise segments renders lines
    #[test]
    fn test_concise_segments() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track(
            "WU",
            20.0,
            24.0,
            vec![make_segment("Waiting", 200.0, 400.0, 32.0, false)],
        ));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        // Concise draws lines, not band rects
        assert!(svg.contains("Waiting"), "state label must appear");
        assert!(svg.contains("WU"), "track label must appear");
    }

    // 6. Message renders arrow line
    #[test]
    fn test_message_arrow() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.messages.push(TimingMsgLayout {
            from_x: 200.0,
            from_y: 40.0,
            to_x: 200.0,
            to_y: 80.0,
            label: "URL".into(),
        });
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r#"marker-end="url(#timing-arrow)""#),
            "message must have arrow marker"
        );
        assert!(svg.contains("URL"), "message label must appear");
    }

    // 7. Message with empty label does not render label text
    #[test]
    fn test_message_no_label() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.messages.push(TimingMsgLayout {
            from_x: 200.0,
            from_y: 40.0,
            to_x: 200.0,
            to_y: 80.0,
            label: "".into(),
        });
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        // There should be no text element for the message label
        // (only defs text, track labels etc might exist, but no empty-label text)
        assert!(svg.contains(r#"marker-end="url(#timing-arrow)""#));
        // Count text elements: should not have a message-label text
        let text_count = svg.matches("<text").count();
        // Just the defs, no extra text for empty label
        assert!(
            text_count == 0,
            "no text elements expected for empty label, got {text_count}"
        );
    }

    // 8. Constraint renders double-ended arrow
    #[test]
    fn test_constraint_rendering() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.constraints.push(TimingConstraintLayout {
            x_start: 200.0,
            x_end: 350.0,
            y: 90.0,
            label: "{150 ms}".into(),
        });
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("marker-start"),
            "constraint must have start marker"
        );
        assert!(
            svg.contains("marker-end"),
            "constraint must have end marker"
        );
        assert!(svg.contains("{150 ms}"), "constraint label must appear");
    }

    // 9. Time axis ticks render
    #[test]
    fn test_time_axis() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.time_axis.ticks.push(TimingTick {
            x: 200.0,
            label: "0".into(),
        });
        layout.time_axis.ticks.push(TimingTick {
            x: 350.0,
            label: "100".into(),
        });
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("0"), "tick label '0' must appear");
        assert!(svg.contains("100"), "tick label '100' must appear");
        assert!(
            svg.contains(&format!(r#"stroke="{AXIS_LINE_COLOR}""#)),
            "axis line must use AXIS_LINE_COLOR"
        );
    }

    // 10. XML escaping in labels
    #[test]
    fn test_xml_escaping() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track(
            "A & B",
            20.0,
            40.0,
            vec![make_segment("S<1>", 200.0, 400.0, 40.0, true)],
        ));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("A &amp; B"), "track name must be XML-escaped");
        assert!(svg.contains("S&lt;1&gt;"), "state name must be XML-escaped");
    }

    // 11. Tick grid lines (vertical dashed lines)
    #[test]
    fn test_tick_grid() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.time_axis.ticks.push(TimingTick {
            x: 250.0,
            label: "50".into(),
        });
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("stroke-dasharray"),
            "grid lines must be dashed"
        );
    }

    // 12. Robust transition draws diagonal line
    #[test]
    fn test_robust_transition() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track(
            "Sig",
            20.0,
            40.0,
            vec![
                make_segment("Low", 200.0, 300.0, 50.0, true),
                make_segment("High", 300.0, 400.0, 30.0, true),
            ],
        ));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        // Count line elements: should have transition line(s)
        let line_count = svg.matches("<line").count();
        assert!(line_count >= 1, "should have transition line(s)");
    }

    // 13. Concise transition draws vertical line
    #[test]
    fn test_concise_transition() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track(
            "Sig",
            20.0,
            24.0,
            vec![
                make_segment("Off", 200.0, 300.0, 32.0, false),
                make_segment("On", 300.0, 400.0, 28.0, false),
            ],
        ));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        let line_count = svg.matches("<line").count();
        assert!(
            line_count >= 2,
            "should have at least signal + transition lines, got {line_count}"
        );
    }

    // 14. Track background rect
    #[test]
    fn test_track_background() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track(
            "A",
            20.0,
            40.0,
            vec![make_segment("Idle", 200.0, 400.0, 40.0, true)],
        ));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        // Should have a background rect with opacity
        assert!(
            svg.contains("opacity=\"0.3\""),
            "track background should have opacity"
        );
    }

    // 15. Full diagram integration
    #[test]
    fn test_full_diagram() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.width = 600.0;
        layout.height = 250.0;
        layout.tracks.push(make_track(
            "DNS Resolver",
            20.0,
            40.0,
            vec![
                make_segment("Idle", 200.0, 400.0, 40.0, true),
                make_segment("Processing", 400.0, 550.0, 30.0, true),
            ],
        ));
        layout.tracks.push(make_track(
            "Web User",
            76.0,
            24.0,
            vec![
                make_segment("Idle", 200.0, 300.0, 88.0, false),
                make_segment("Waiting", 300.0, 550.0, 82.0, false),
            ],
        ));
        layout.messages.push(TimingMsgLayout {
            from_x: 300.0,
            from_y: 88.0,
            to_x: 300.0,
            to_y: 40.0,
            label: "URL".into(),
        });
        layout.constraints.push(TimingConstraintLayout {
            x_start: 350.0,
            x_end: 500.0,
            y: 110.0,
            label: "{150 ms}".into(),
        });
        layout.time_axis.ticks = vec![
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

        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("DNS Resolver"));
        assert!(svg.contains("Web User"));
        assert!(svg.contains("URL"));
        assert!(svg.contains("{150 ms}"));
        assert!(svg.contains("0"));
        assert!(svg.contains("700"));
    }

    // 16. Participant label is bold
    #[test]
    fn test_participant_label_bold() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track(
            "Signal",
            20.0,
            40.0,
            vec![make_segment("Low", 200.0, 400.0, 40.0, true)],
        ));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("font-weight=\"bold\""),
            "participant label should be bold"
        );
    }

    // 17. Constraint label uses constraint color
    #[test]
    fn test_constraint_label_color() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.constraints.push(TimingConstraintLayout {
            x_start: 200.0,
            x_end: 350.0,
            y: 90.0,
            label: "test".into(),
        });
        let skin = SkinParams::default();
        let svg = render_timing(&model, &layout, &skin).expect("render failed");
        let expected_color = skin.font_color("constraint", CONSTRAINT_COLOR);
        assert!(
            svg.contains(&format!(r#"fill="{}""#, expected_color)),
            "constraint label must use constraint color"
        );
    }

    // 18. Track with no segments renders only label
    #[test]
    fn test_track_no_segments() {
        let model = empty_model();
        let mut layout = empty_layout();
        layout.tracks.push(make_track("Empty", 20.0, 40.0, vec![]));
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("Empty"), "label should still appear");
        // No background rect (no segments means no x range)
        // The label still renders though
    }

    // 19. End-to-end: parse + layout + render
    #[test]
    fn test_end_to_end() {
        use crate::layout::timing::layout_timing;
        use crate::parser::timing::parse_timing_diagram;

        let src = r#"@startuml
robust "DNS Resolver" as DNS
robust "Web Browser" as WB
concise "Web User" as WU

@0
WU is Idle
WB is Idle
DNS is Idle

@+100
WU is Waiting
WB is Processing

@+200
WB is Waiting

@+100
DNS is Processing

@+300
DNS is Idle
@enduml"#;

        let td = parse_timing_diagram(src).unwrap();
        let layout = layout_timing(&td).unwrap();
        let svg = render_timing(&td, &layout, &SkinParams::default()).unwrap();
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
        let model = empty_model();
        let mut layout = empty_layout();
        layout.notes.push(TimingNoteLayout {
            text: "**watch**".to_string(),
            x: 250.0,
            y: 40.0,
            width: 100.0,
            height: 44.0,
            connector: Some((230.0, 50.0, 250.0, 56.0)),
        });
        let svg = render_timing(&model, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<polygon"), "note body must render");
        assert!(svg.contains("stroke-dasharray=\"4,4\""));
        assert!(
            svg.contains("font-weight=\"bold\""),
            "creole note text should render"
        );
    }
}
