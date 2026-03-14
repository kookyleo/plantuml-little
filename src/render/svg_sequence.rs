use std::fmt::Write;

use crate::layout::sequence::{
    ActivationLayout, DelayLayout, DestroyLayout, DividerLayout, FragmentLayout, GroupLayout,
    MessageLayout, NoteLayout, ParticipantLayout, RefLayout, SeqLayout,
};
use crate::model::sequence::ParticipantKind;
use crate::model::SequenceDiagram;
use crate::style::SkinParams;
use crate::Result;

use super::svg::xml_escape;
use super::svg::write_svg_root;
use super::svg_richtext::render_creole_text;

// ── Style constants ─────────────────────────────────────────────────

const FONT_SIZE: f64 = 13.0;
const CHAR_WIDTH: f64 = 7.2;
const LINE_HEIGHT: f64 = 16.0;
const PARTICIPANT_BG: &str = "#F1F1F1";
const PARTICIPANT_BORDER: &str = "#181818";
const LIFELINE_COLOR: &str = "#181818";
const ARROW_COLOR: &str = "#181818";
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const GROUP_BG: &str = "#EEEEEE";
const GROUP_BORDER: &str = "#000000";
const ACTIVATION_BG: &str = "#F1F1F1";
const ACTIVATION_BORDER: &str = "#181818";
const FRAGMENT_BG: &str = "#F1F1F1";
const FRAGMENT_BORDER: &str = "#181818";
const REF_BG: &str = "#F1F1F1";
const REF_BORDER: &str = "#181818";
const DIVIDER_COLOR: &str = "#888888";
const TEXT_COLOR: &str = "#000000";

const MARGIN: f64 = 20.0;

// ── Arrow marker defs ───────────────────────────────────────────────

fn write_seq_defs(buf: &mut String) {
    buf.push_str("<defs>\n");

    // Filled triangle arrow marker (for solid arrowheads)
    write!(
        buf,
        concat!(
            r##"<marker id="seq-arrow-filled" viewBox="0 0 10 10" refX="10" refY="5""##,
            r##" markerWidth="8" markerHeight="8" orient="auto-start-reverse">"##,
            r##"<path d="M 0 0 L 10 5 L 0 10 Z" fill="{}" stroke="none"/>"##,
            r##"</marker>"##,
        ),
        ARROW_COLOR
    )
    .unwrap();
    buf.push('\n');

    // Open arrow marker (for open arrowheads)
    write!(
        buf,
        concat!(
            r##"<marker id="seq-arrow-open" viewBox="0 0 10 10" refX="10" refY="5""##,
            r##" markerWidth="8" markerHeight="8" orient="auto-start-reverse">"##,
            r##"<path d="M 0 0 L 10 5 L 0 10" fill="none" stroke="{}" stroke-width="1.2"/>"##,
            r##"</marker>"##,
        ),
        ARROW_COLOR
    )
    .unwrap();
    buf.push('\n');

    buf.push_str("</defs>\n");
}

// ── Lifelines ───────────────────────────────────────────────────────

fn draw_lifelines(buf: &mut String, layout: &SeqLayout, skin: &SkinParams) {
    let ll_color = skin.sequence_lifeline_border_color(LIFELINE_COLOR);
    for p in &layout.participants {
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:1;stroke-dasharray:5,5;" x1="{x:.1}" x2="{x:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
            x = p.x,
            y1 = layout.lifeline_top,
            y2 = layout.lifeline_bottom,
            color = ll_color,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Participant box ─────────────────────────────────────────────────

fn draw_participant_box(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let fill = p.color.as_deref().unwrap_or(bg);

    match &p.kind {
        ParticipantKind::Actor => {
            draw_participant_actor(buf, p, y, display_name, border, text_color);
        }
        ParticipantKind::Boundary => {
            draw_participant_boundary(buf, p, y, display_name, fill, border, text_color);
        }
        ParticipantKind::Control => {
            draw_participant_control(buf, p, y, display_name, fill, border, text_color);
        }
        ParticipantKind::Entity => {
            draw_participant_entity(buf, p, y, display_name, fill, border, text_color);
        }
        ParticipantKind::Database => {
            draw_participant_database(buf, p, y, display_name, fill, border, text_color);
        }
        ParticipantKind::Collections => {
            draw_participant_collections(buf, p, y, display_name, fill, border, text_color);
        }
        ParticipantKind::Queue => {
            draw_participant_queue(buf, p, y, display_name, fill, border, text_color);
        }
        ParticipantKind::Default => {
            draw_participant_rect(buf, p, y, display_name, fill, border, text_color);
        }
    }
}

/// Default rectangle participant
fn draw_participant_rect(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let x = p.x - p.box_width / 2.0;
    let rect_h = 36.0;

    write!(
        buf,
        r#"<rect fill="{bg}" height="{h:.1}" style="stroke:{border};stroke-width:1.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        w = p.box_width,
        h = rect_h,
    )
    .unwrap();
    buf.push('\n');

    let text_y = y + rect_h / 2.0 + FONT_SIZE * 0.35;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{ty:.1}">{text}</text>"#,
        cx = p.x,
        ty = text_y,
        color = text_color,
        text = escaped,
    )
    .unwrap();
    buf.push('\n');
}

/// Actor: stick figure (circle head + body + arms + legs) with name below
fn draw_participant_actor(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let head_r = 8.0;
    let head_cy = y + head_r + 2.0;
    let body_top = head_cy + head_r;
    let body_len = 20.0;
    let body_bot = body_top + body_len;
    let arm_y = body_top + body_len * 0.35;
    let arm_spread = 14.0;
    let leg_spread = 10.0;
    let leg_drop = 16.0;

    // Head
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{head_cy:.1}" fill="none" r="{head_r}" style="stroke:{border};stroke-width:1.5;"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Body
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{cx:.1}" x2="{cx:.1}" y1="{body_top:.1}" y2="{body_bot:.1}"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Left arm
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{cx:.1}" x2="{lx:.1}" y1="{ay:.1}" y2="{ay:.1}"/>"#,
        ay = arm_y,
        lx = cx - arm_spread,
    )
    .unwrap();
    buf.push('\n');

    // Right arm
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{cx:.1}" x2="{rx:.1}" y1="{ay:.1}" y2="{ay:.1}"/>"#,
        ay = arm_y,
        rx = cx + arm_spread,
    )
    .unwrap();
    buf.push('\n');

    // Left leg
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{cx:.1}" x2="{lx:.1}" y1="{ly:.1}" y2="{lby:.1}"/>"#,
        ly = body_bot,
        lx = cx - leg_spread,
        lby = body_bot + leg_drop,
    )
    .unwrap();
    buf.push('\n');

    // Right leg
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{cx:.1}" x2="{rx:.1}" y1="{ly:.1}" y2="{lby:.1}"/>"#,
        ly = body_bot,
        rx = cx + leg_spread,
        lby = body_bot + leg_drop,
    )
    .unwrap();
    buf.push('\n');

    // Name below figure
    let name_y = body_bot + leg_drop + FONT_SIZE + 4.0;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Boundary: circle on left + vertical line + horizontal connector
fn draw_participant_boundary(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let icon_y = y + 4.0;
    let icon_r = 10.0;
    let icon_cx = cx - 8.0;

    // Circle on the right side
    write!(
        buf,
        r#"<circle cx="{icx:.1}" cy="{icy:.1}" fill="{bg}" r="{r}" style="stroke:{border};stroke-width:1.5;"/>"#,
        icx = icon_cx,
        icy = icon_y + icon_r,
        r = icon_r,
    )
    .unwrap();
    buf.push('\n');

    // Vertical line to the left of circle
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{lx:.1}" x2="{lx:.1}" y1="{ly1:.1}" y2="{ly2:.1}"/>"#,
        lx = icon_cx - icon_r - 4.0,
        ly1 = icon_y,
        ly2 = icon_y + 2.0 * icon_r,
    )
    .unwrap();
    buf.push('\n');

    // Horizontal connector from vertical bar to circle
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{lx:.1}" x2="{rx:.1}" y1="{ly:.1}" y2="{ly:.1}"/>"#,
        lx = icon_cx - icon_r - 4.0,
        ly = icon_y + icon_r,
        rx = icon_cx - icon_r,
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = icon_y + 2.0 * icon_r + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Control: circle with a small arrow on top
fn draw_participant_control(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let icon_r = 12.0;
    let icon_cy = y + icon_r + 8.0;

    // Circle
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{icon_cy:.1}" fill="{bg}" r="{icon_r}" style="stroke:{border};stroke-width:1.5;"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Small arrow on top of circle
    let arrow_y = icon_cy - icon_r;
    write!(
        buf,
        r#"<path d="M {x1:.1},{y1:.1} L {x2:.1},{y2:.1} L {x3:.1},{y3:.1}" fill="none" style="stroke:{border};stroke-width:1.5;"/>"#,
        x1 = cx - 5.0,
        y1 = arrow_y - 6.0,
        x2 = cx + 2.0,
        y2 = arrow_y - 1.0,
        x3 = cx - 5.0,
        y3 = arrow_y + 3.0,
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = icon_cy + icon_r + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Entity: circle with a horizontal underline
fn draw_participant_entity(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let icon_r = 12.0;
    let icon_cy = y + icon_r + 4.0;

    // Circle
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{icon_cy:.1}" fill="{bg}" r="{icon_r}" style="stroke:{border};stroke-width:1.5;"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Horizontal underline
    let line_y = icon_cy + icon_r + 2.0;
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{x1:.1}" x2="{x2:.1}" y1="{ly:.1}" y2="{ly:.1}"/>"#,
        x1 = cx - icon_r,
        ly = line_y,
        x2 = cx + icon_r,
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = line_y + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Database: cylinder shape (rect with rounded top/bottom arcs)
fn draw_participant_database(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let cyl_w = 40.0;
    let cyl_h = 30.0;
    let arc_h = 6.0;
    let cyl_x = cx - cyl_w / 2.0;
    let cyl_y = y + 4.0;

    // Cylinder body
    write!(
        buf,
        r#"<path d="M {x:.1},{ty:.1} A {rx:.1},{ry:.1} 0 0,0 {x2:.1},{ty:.1} L {x2:.1},{by:.1} A {rx:.1},{ry:.1} 0 0,0 {x:.1},{by:.1} Z" fill="{bg}" style="stroke:{border};stroke-width:1.5;"/>"#,
        x = cyl_x,
        ty = cyl_y + arc_h,
        rx = cyl_w / 2.0,
        ry = arc_h,
        x2 = cyl_x + cyl_w,
        by = cyl_y + cyl_h,
    )
    .unwrap();
    buf.push('\n');

    // Top ellipse
    write!(
        buf,
        r#"<ellipse cx="{cx:.1}" cy="{ey:.1}" fill="{bg}" rx="{rx:.1}" ry="{ry:.1}" style="stroke:{border};stroke-width:1.5;"/>"#,
        ey = cyl_y + arc_h,
        rx = cyl_w / 2.0,
        ry = arc_h,
    )
    .unwrap();
    buf.push('\n');

    // Name below cylinder
    let name_y = cyl_y + cyl_h + arc_h + FONT_SIZE + 4.0;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Collections: stacked rectangles (shadow rect behind main rect)
fn draw_participant_collections(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let rect_w = p.box_width.min(60.0);
    let rect_h = 28.0;
    let offset = 4.0;
    let rx = cx - rect_w / 2.0;
    let ry = y + 8.0;

    // Back (shadow) rectangle
    write!(
        buf,
        r#"<rect fill="{bg}" height="{h:.1}" style="stroke:{border};stroke-width:1.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = rx + offset,
        y = ry - offset,
        w = rect_w,
        h = rect_h,
    )
    .unwrap();
    buf.push('\n');

    // Front (main) rectangle
    write!(
        buf,
        r#"<rect fill="{bg}" height="{rect_h:.1}" style="stroke:{border};stroke-width:1.5;" width="{rect_w:.1}" x="{rx:.1}" y="{ry:.1}"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = ry + rect_h + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Queue: horizontal cylinder (cylinder rotated 90 degrees)
fn draw_participant_queue(
    buf: &mut String,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let cyl_w = 44.0;
    let cyl_h = 28.0;
    let arc_w = 6.0;
    let cyl_x = cx - cyl_w / 2.0;
    let cyl_y = y + 6.0;

    // Cylinder body (horizontal)
    write!(
        buf,
        r#"<path d="M {lx:.1},{ty:.1} L {rx:.1},{ty:.1} A {aw:.1},{ah:.1} 0 0,1 {rx:.1},{by:.1} L {lx:.1},{by:.1} A {aw:.1},{ah:.1} 0 0,1 {lx:.1},{ty:.1} Z" fill="{bg}" style="stroke:{border};stroke-width:1.5;"/>"#,
        lx = cyl_x,
        ty = cyl_y,
        rx = cyl_x + cyl_w - arc_w,
        aw = arc_w,
        ah = cyl_h / 2.0,
        by = cyl_y + cyl_h,
    )
    .unwrap();
    buf.push('\n');

    // Right end cap ellipse
    write!(
        buf,
        r#"<ellipse cx="{ecx:.1}" cy="{ecy:.1}" fill="{bg}" rx="{erx:.1}" ry="{ery:.1}" style="stroke:{border};stroke-width:1.5;"/>"#,
        ecx = cyl_x + cyl_w - arc_w,
        ecy = cyl_y + cyl_h / 2.0,
        erx = arc_w,
        ery = cyl_h / 2.0,
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = cyl_y + cyl_h + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" text-anchor="middle" x="{cx:.1}" y="{name_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

// ── Messages ────────────────────────────────────────────────────────

fn draw_message(buf: &mut String, msg: &MessageLayout, arrow_color: &str, arrow_thickness: f64) {
    let marker_id = if msg.has_open_head {
        "seq-arrow-open"
    } else {
        "seq-arrow-filled"
    };

    let dash = if msg.is_dashed {
        r#" stroke-dasharray="7,5""#
    } else {
        ""
    };

    // Determine marker placement based on direction
    let (marker_attr, x1, x2) = if msg.is_left {
        // Right-to-left: marker on the start (left end)
        (
            format!(r#" marker-start="url(#{marker_id})""#),
            msg.to_x,
            msg.from_x,
        )
    } else {
        // Left-to-right: marker on the end (right end)
        (
            format!(r#" marker-end="url(#{marker_id})""#),
            msg.from_x,
            msg.to_x,
        )
    };

    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:{sw};" x1="{x1:.1}" x2="{x2:.1}" y1="{y:.1}" y2="{y:.1}"{dash}{marker}/>"#,
        y = msg.y,
        color = arrow_color,
        sw = arrow_thickness as u32,
        marker = marker_attr,
    )
    .unwrap();
    buf.push('\n');

    // Label text centered above the line
    if !msg.text.is_empty() {
        let mid_x = (msg.from_x + msg.to_x) / 2.0;
        let text_y = msg.y - 6.0;
        render_creole_text(
            buf,
            &msg.text,
            mid_x,
            text_y,
            LINE_HEIGHT,
            TEXT_COLOR,
            Some("middle"),
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
    }
}

fn draw_self_message(
    buf: &mut String,
    msg: &MessageLayout,
    arrow_color: &str,
    arrow_thickness: f64,
) {
    let dash = if msg.is_dashed {
        r#" stroke-dasharray="7,5""#
    } else {
        ""
    };

    let marker_id = if msg.has_open_head {
        "seq-arrow-open"
    } else {
        "seq-arrow-filled"
    };

    let x = msg.from_x;
    let y = msg.y;
    let loop_width = 30.0;
    let loop_height = 24.0;

    // Cubic bezier loop: goes right and comes back
    write!(
        buf,
        r#"<path d="M {x:.1},{y:.1} C {cx1:.1},{y:.1} {cx1:.1},{y2:.1} {x:.1},{y2:.1}" fill="none" marker-end="url(#{marker})" style="stroke:{color};stroke-width:{sw};"{dash}/>"#,
        cx1 = x + loop_width,
        y2 = y + loop_height,
        color = arrow_color,
        sw = arrow_thickness as u32,
        marker = marker_id,
    )
    .unwrap();
    buf.push('\n');

    // Label to the right of the loop
    if !msg.text.is_empty() {
        let text_x = x + loop_width + 4.0;
        let text_y = y + loop_height / 2.0 + FONT_SIZE * 0.35;
        render_creole_text(
            buf,
            &msg.text,
            text_x,
            text_y,
            LINE_HEIGHT,
            TEXT_COLOR,
            None,
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
    }
}

// ── Activation bars ─────────────────────────────────────────────────

fn draw_activation(buf: &mut String, act: &ActivationLayout) {
    let width = 10.0;
    let height = act.y_end - act.y_start;

    write!(
        buf,
        r#"<rect fill="{bg}" height="{h:.1}" style="stroke:{border};stroke-width:1;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = act.x,
        y = act.y_start,
        w = width,
        h = height,
        bg = ACTIVATION_BG,
        border = ACTIVATION_BORDER,
    )
    .unwrap();
    buf.push('\n');
}

// ── Destroy marker ──────────────────────────────────────────────────

fn draw_destroy(buf: &mut String, d: &DestroyLayout) {
    let size = 10.0;
    // First diagonal: top-left to bottom-right
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:2;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        x1 = d.x - size,
        y1 = d.y - size,
        x2 = d.x + size,
        y2 = d.y + size,
        color = ARROW_COLOR,
    )
    .unwrap();
    buf.push('\n');

    // Second diagonal: top-right to bottom-left
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:2;" x1="{x1:.1}" x2="{x2:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
        x1 = d.x + size,
        y1 = d.y - size,
        x2 = d.x - size,
        y2 = d.y + size,
        color = ARROW_COLOR,
    )
    .unwrap();
    buf.push('\n');
}

// ── Notes ───────────────────────────────────────────────────────────

fn draw_note(buf: &mut String, note: &NoteLayout) {
    let fold = 8.0; // folded corner size

    // Background rect
    write!(
        buf,
        r#"<rect fill="{bg}" height="{h:.1}" style="stroke:{border};stroke-width:1;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = note.x,
        y = note.y,
        w = note.width,
        h = note.height,
        bg = NOTE_BG,
        border = NOTE_BORDER,
    )
    .unwrap();
    buf.push('\n');

    // Folded corner triangle in top-right
    let cx = note.x + note.width - fold;
    let cy = note.y;
    write!(
        buf,
        r#"<path d="M {cx:.1},{cy:.1} L {cx:.1},{cy2:.1} L {cx2:.1},{cy:.1} Z" fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
        cy2 = cy + fold,
        cx2 = note.x + note.width,
        bg = NOTE_BG,
        border = NOTE_BORDER,
    )
    .unwrap();
    buf.push('\n');

    let text_x = note.x + 6.0;
    render_creole_text(
        buf,
        &note.text,
        text_x,
        note.y + LINE_HEIGHT,
        LINE_HEIGHT,
        TEXT_COLOR,
        None,
        &format!(r#"font-size="{FONT_SIZE}""#),
    );
}

// ── Group frames ────────────────────────────────────────────────────

fn draw_group(buf: &mut String, group: &GroupLayout) {
    let height = group.y_end - group.y_start;

    // Frame rectangle
    write!(
        buf,
        r#"<rect fill="{bg}" fill-opacity="0.3" height="{h:.1}" style="stroke:{border};stroke-width:1;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = group.x,
        y = group.y_start,
        w = group.width,
        h = height,
        bg = GROUP_BG,
        border = GROUP_BORDER,
    )
    .unwrap();
    buf.push('\n');

    // Label in top-left corner
    if let Some(label) = &group.label {
        let label_x = group.x + 6.0;
        let label_y = group.y_start + FONT_SIZE + 2.0;
        let escaped = xml_escape(label);

        // Label background tab
        let label_width = label.len() as f64 * CHAR_WIDTH + 12.0;
        let label_height = FONT_SIZE + 6.0;
        write!(
            buf,
            r#"<rect fill="{bg}" height="{h:.1}" style="stroke:{border};stroke-width:1;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
            x = group.x,
            y = group.y_start,
            w = label_width,
            h = label_height,
            bg = GROUP_BG,
            border = GROUP_BORDER,
        )
        .unwrap();
        buf.push('\n');

        write!(
            buf,
            r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" x="{label_x:.1}" y="{label_y:.1}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Fragment frames ──────────────────────────────────────────────────

fn draw_fragment(buf: &mut String, frag: &FragmentLayout) {
    // Frame rectangle with semi-transparent fill
    write!(
        buf,
        r#"<rect fill="{bg}" fill-opacity="0.1" height="{h:.1}" rx="2" style="stroke:{border};stroke-width:1.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = frag.x,
        y = frag.y,
        w = frag.width,
        h = frag.height,
        bg = FRAGMENT_BG,
        border = FRAGMENT_BORDER,
    )
    .unwrap();
    buf.push('\n');

    // Label tab (pentagon-like shape in top-left)
    let kind_label = frag.kind.label();
    let tab_text = if frag.label.is_empty() {
        kind_label.to_string()
    } else {
        format!("{} {}", kind_label, frag.label)
    };
    let tab_width = tab_text.len() as f64 * CHAR_WIDTH + 16.0;
    let tab_height = FONT_SIZE + 8.0;
    let notch = 6.0;

    // Pentagon path: top-left corner with a notch at bottom-right
    write!(
        buf,
        r#"<path d="M {x:.1},{y:.1} L {x2:.1},{y:.1} L {x2:.1},{y2:.1} L {x3:.1},{y3:.1} L {x:.1},{y3:.1} Z" fill="{bg}" style="stroke:{border};stroke-width:1.5;"/>"#,
        x = frag.x,
        y = frag.y,
        x2 = frag.x + tab_width,
        y2 = frag.y + tab_height - notch,
        x3 = frag.x + tab_width - notch,
        y3 = frag.y + tab_height,
        bg = FRAGMENT_BG,
        border = FRAGMENT_BORDER,
    )
    .unwrap();
    buf.push('\n');

    // Kind label text
    let text_x = frag.x + 6.0;
    let text_y = frag.y + FONT_SIZE + 2.0;
    let escaped = xml_escape(&tab_text);
    write!(
        buf,
        r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" x="{text_x:.1}" y="{text_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');

    // Separator lines (else)
    for (sep_y, sep_label) in &frag.separators {
        // Dashed horizontal line
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:1;stroke-dasharray:5,5;" x1="{x1:.1}" x2="{x2:.1}" y1="{y:.1}" y2="{y:.1}"/>"#,
            x1 = frag.x,
            y = sep_y,
            x2 = frag.x + frag.width,
            border = FRAGMENT_BORDER,
        )
        .unwrap();
        buf.push('\n');

        // Separator label
        if !sep_label.is_empty() {
            let label_x = frag.x + 10.0;
            let label_y = sep_y + FONT_SIZE + 2.0;
            let escaped_label = xml_escape(sep_label);
            write!(
                buf,
                r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-style="italic" x="{label_x:.1}" y="{label_y:.1}">[{escaped_label}]</text>"#,
            )
            .unwrap();
            buf.push('\n');
        }
    }
}

// ── Divider ──────────────────────────────────────────────────────────

fn draw_divider(buf: &mut String, divider: &DividerLayout) {
    let center_y = divider.y + 15.0;

    // Background stripe
    write!(
        buf,
        r#"<rect fill="{color}" fill-opacity="0.2" height="5" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = divider.x,
        y = center_y - 2.5,
        w = divider.width,
        color = DIVIDER_COLOR,
    )
    .unwrap();
    buf.push('\n');

    // Horizontal lines
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y:.1}" y2="{y:.1}"/>"#,
        x1 = divider.x,
        y = center_y - 2.5,
        x2 = divider.x + divider.width,
        color = DIVIDER_COLOR,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y:.1}" y2="{y:.1}"/>"#,
        x1 = divider.x,
        y = center_y + 2.5,
        x2 = divider.x + divider.width,
        color = DIVIDER_COLOR,
    )
    .unwrap();
    buf.push('\n');

    // Centered label text
    if let Some(text) = &divider.text {
        let mid_x = divider.x + divider.width / 2.0;
        let text_y = center_y + FONT_SIZE * 0.35;
        let escaped = xml_escape(text);

        // Text background
        let text_width = text.len() as f64 * CHAR_WIDTH + 16.0;
        write!(
            buf,
            r#"<rect fill="white" height="{h:.1}" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
            x = mid_x - text_width / 2.0,
            y = center_y - FONT_SIZE * 0.6,
            w = text_width,
            h = FONT_SIZE * 1.2,
        )
        .unwrap();
        buf.push('\n');

        write!(
            buf,
            r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" text-anchor="middle" x="{mid_x:.1}" y="{text_y:.1}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Delay ────────────────────────────────────────────────────────────

fn draw_delay(buf: &mut String, delay: &DelayLayout) {
    let center_y = delay.y + delay.height / 2.0;
    let mid_x = delay.x + delay.width / 2.0;

    // Three dots to indicate delay
    for dy in [-4.0, 0.0, 4.0] {
        write!(
            buf,
            r#"<circle cx="{cx:.1}" cy="{cy:.1}" fill="{color}" r="1.5"/>"#,
            cx = mid_x,
            cy = center_y + dy,
            color = DIVIDER_COLOR,
        )
        .unwrap();
        buf.push('\n');
    }

    // Label text
    if let Some(text) = &delay.text {
        let text_x = mid_x + 12.0;
        let text_y = center_y + FONT_SIZE * 0.35;
        let escaped = xml_escape(text);
        write!(
            buf,
            r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" x="{text_x:.1}" y="{text_y:.1}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Ref ──────────────────────────────────────────────────────────────

fn draw_ref(buf: &mut String, r: &RefLayout) {
    // Filled rectangle
    write!(
        buf,
        r#"<rect fill="{bg}" height="{h:.1}" rx="2" style="stroke:{border};stroke-width:1.5;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = r.x,
        y = r.y,
        w = r.width,
        h = r.height,
        bg = REF_BG,
        border = REF_BORDER,
    )
    .unwrap();
    buf.push('\n');

    // "ref" label tab in top-left
    let tab_width = 3.0 * CHAR_WIDTH + 12.0;
    let tab_height = FONT_SIZE + 6.0;
    let notch = 5.0;
    write!(
        buf,
        r#"<path d="M {x:.1},{y:.1} L {x2:.1},{y:.1} L {x2:.1},{y2:.1} L {x3:.1},{y3:.1} L {x:.1},{y3:.1} Z" fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
        x = r.x,
        y = r.y,
        x2 = r.x + tab_width,
        y2 = r.y + tab_height - notch,
        x3 = r.x + tab_width - notch,
        y3 = r.y + tab_height,
        bg = REF_BG,
        border = REF_BORDER,
    )
    .unwrap();
    buf.push('\n');

    write!(
        buf,
        r#"<text fill="{color}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" x="{tx:.1}" y="{ty:.1}">ref</text>"#,
        tx = r.x + 5.0,
        ty = r.y + FONT_SIZE + 1.0,
        color = TEXT_COLOR,
    )
    .unwrap();
    buf.push('\n');

    // Centered label text
    let mid_x = r.x + r.width / 2.0;
    let mid_y = r.y + r.height / 2.0 + FONT_SIZE * 0.35;
    let escaped = xml_escape(&r.label);
    write!(
        buf,
        r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" text-anchor="middle" x="{mid_x:.1}" y="{mid_y:.1}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

// ── Public entry point ──────────────────────────────────────────────

/// Render a SequenceDiagram + SeqLayout into an SVG string.
pub fn render_sequence(
    sd: &SequenceDiagram,
    layout: &SeqLayout,
    skin: &SkinParams,
) -> Result<String> {
    let svg_w = layout.total_width + MARGIN * 2.0;
    let svg_h = layout.total_height + MARGIN * 2.0;

    let mut buf = String::with_capacity(4096);

    // 1. SVG header
    write_svg_root(&mut buf, svg_w, svg_h, "SEQUENCE");

    // 2. Defs: arrow markers
    write_seq_defs(&mut buf);
    buf.push_str("<g>");

    // 3. Lifelines (dashed vertical lines)
    draw_lifelines(&mut buf, layout, skin);

    // 4. Fragment frames (drawn before groups so they appear behind)
    for frag in &layout.fragments {
        draw_fragment(&mut buf, frag);
    }

    // 4b. Group frames (legacy)
    for group in &layout.groups {
        draw_group(&mut buf, group);
    }

    // 5. Activation bars
    for act in &layout.activations {
        draw_activation(&mut buf, act);
    }

    // 5b. Dividers
    for divider in &layout.dividers {
        draw_divider(&mut buf, divider);
    }

    // 5c. Delays
    for delay in &layout.delays {
        draw_delay(&mut buf, delay);
    }

    // 5d. Refs
    for r in &layout.refs {
        draw_ref(&mut buf, r);
    }

    // 6. Messages (with optional autonumber)
    let seq_arrow_color = skin.sequence_arrow_color(ARROW_COLOR);
    let seq_arrow_thickness = skin.sequence_arrow_thickness().unwrap_or(1.0);
    let mut msg_counter: u32 = layout.autonumber_start;
    for msg in &layout.messages {
        if layout.autonumber_enabled {
            // Create a message copy with autonumber prefix
            let numbered_text = if msg.text.is_empty() {
                format!("{msg_counter}")
            } else {
                format!("{} {}", msg_counter, msg.text)
            };
            let numbered_msg = MessageLayout {
                text: numbered_text,
                ..msg.clone()
            };
            if numbered_msg.is_self {
                draw_self_message(
                    &mut buf,
                    &numbered_msg,
                    seq_arrow_color,
                    seq_arrow_thickness,
                );
            } else {
                draw_message(
                    &mut buf,
                    &numbered_msg,
                    seq_arrow_color,
                    seq_arrow_thickness,
                );
            }
            msg_counter += 1;
        } else if msg.is_self {
            draw_self_message(&mut buf, msg, seq_arrow_color, seq_arrow_thickness);
        } else {
            draw_message(&mut buf, msg, seq_arrow_color, seq_arrow_thickness);
        }
    }

    // 7. Notes
    for note in &layout.notes {
        draw_note(&mut buf, note);
    }

    // 8. Destroy markers
    for d in &layout.destroys {
        draw_destroy(&mut buf, d);
    }

    // 9. Participant boxes (top)
    // Build a name -> display_name lookup from the diagram
    let display_names: std::collections::HashMap<&str, &str> = sd
        .participants
        .iter()
        .filter_map(|p| p.display_name.as_deref().map(|dn| (p.name.as_str(), dn)))
        .collect();

    let part_bg = skin.background_color("participant", PARTICIPANT_BG);
    let part_border = skin.border_color("participant", PARTICIPANT_BORDER);
    let part_font = skin.font_color("participant", TEXT_COLOR);

    let top_y = MARGIN;
    for p in &layout.participants {
        let dn = display_names.get(p.name.as_str()).copied();
        draw_participant_box(&mut buf, p, top_y, dn, part_bg, part_border, part_font);
    }

    // Bottom participant boxes (below lifeline)
    let bottom_y = layout.lifeline_bottom;
    for p in &layout.participants {
        let dn = display_names.get(p.name.as_str()).copied();
        draw_participant_box(&mut buf, p, bottom_y, dn, part_bg, part_border, part_font);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::sequence::SeqLayout;
    use crate::model::sequence::{
        Message, Participant, ParticipantKind, SeqArrowHead, SeqArrowStyle, SeqDirection, SeqEvent,
        SequenceDiagram,
    };
    use crate::style::SkinParams;

    fn make_participant(name: &str) -> Participant {
        Participant {
            name: name.to_string(),
            display_name: None,
            kind: ParticipantKind::Default,
            color: None,
        }
    }

    fn make_message(from: &str, to: &str, text: &str) -> Message {
        Message {
            from: from.to_string(),
            to: to.to_string(),
            text: text.to_string(),
            arrow_style: SeqArrowStyle::Solid,
            arrow_head: SeqArrowHead::Filled,
            direction: SeqDirection::LeftToRight,
        }
    }

    fn simple_layout() -> (SequenceDiagram, SeqLayout) {
        let sd = SequenceDiagram {
            participants: vec![make_participant("Alice"), make_participant("Bob")],
            events: vec![SeqEvent::Message(make_message("Alice", "Bob", "hello"))],
        };
        let layout = crate::layout::sequence::layout_sequence(&sd).unwrap();
        (sd, layout)
    }

    #[test]
    fn basic_render_produces_valid_svg() {
        let (sd, layout) = simple_layout();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "output must contain <svg");
        assert!(svg.contains("</svg>"), "output must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("contentStyleType=\"text/css\""));
    }

    #[test]
    fn participant_name_appears_in_output() {
        let (sd, layout) = simple_layout();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("Alice"),
            "SVG must contain participant name Alice"
        );
        assert!(svg.contains("Bob"), "SVG must contain participant name Bob");
    }

    #[test]
    fn message_renders_line_element() {
        let (sd, layout) = simple_layout();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        // Normal message produces a <line> element (not a <path>)
        assert!(svg.contains("<line"), "SVG must contain <line for messages");
        assert!(svg.contains("hello"), "SVG must contain message text");
    }

    #[test]
    fn self_message_renders_path_element() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::Message(make_message("A", "A", "self call"))],
        };
        let layout = crate::layout::sequence::layout_sequence(&sd).unwrap();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        // Self-message produces a <path> element (cubic bezier)
        assert!(
            svg.contains("<path"),
            "SVG must contain <path for self-message"
        );
        assert!(
            svg.contains("self call"),
            "SVG must contain self-message text"
        );
    }

    #[test]
    fn dashed_message_has_stroke_dasharray() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![SeqEvent::Message(Message {
                from: "A".to_string(),
                to: "B".to_string(),
                text: "reply".to_string(),
                arrow_style: SeqArrowStyle::Dashed,
                arrow_head: SeqArrowHead::Open,
                direction: SeqDirection::LeftToRight,
            })],
        };
        let layout = crate::layout::sequence::layout_sequence(&sd).unwrap();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("stroke-dasharray"),
            "dashed message must have stroke-dasharray"
        );
        assert!(
            svg.contains("seq-arrow-open"),
            "open-head message must reference seq-arrow-open marker"
        );
    }

    #[test]
    fn destroy_marker_renders_cross() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A"), make_participant("B")],
            events: vec![
                SeqEvent::Message(make_message("A", "B", "kill")),
                SeqEvent::Destroy("B".to_string()),
            ],
        };
        let layout = crate::layout::sequence::layout_sequence(&sd).unwrap();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        // Destroy marker is an X made of two <line> elements with stroke-width:2 in style
        let cross_count = svg.matches("stroke-width:2;").count();
        assert!(
            cross_count >= 2,
            "destroy marker should produce 2 lines with stroke-width:2, found {cross_count}"
        );
    }

    #[test]
    fn note_renders_rect_and_text() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::NoteRight {
                participant: "A".to_string(),
                text: "important note".to_string(),
            }],
        };
        let layout = crate::layout::sequence::layout_sequence(&sd).unwrap();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(NOTE_BG), "note should use yellow background");
        assert!(
            svg.contains("important note"),
            "note text must appear in SVG"
        );
    }

    #[test]
    fn empty_diagram_renders_valid_svg() {
        let sd = SequenceDiagram {
            participants: vec![],
            events: vec![],
        };
        let layout = crate::layout::sequence::layout_sequence(&sd).unwrap();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "empty diagram must produce valid SVG");
        assert!(svg.contains("</svg>"));
    }
}
