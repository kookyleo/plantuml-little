use std::fmt::Write;

use crate::layout::sequence::{
    ActivationLayout, DelayLayout, DestroyLayout, DividerLayout, FragmentLayout, GroupLayout,
    MessageLayout, NoteLayout, ParticipantLayout, RefLayout, SeqLayout,
};
use crate::model::sequence::ParticipantKind;
use crate::model::SequenceDiagram;
use crate::style::SkinParams;
use crate::Result;

use crate::font_metrics;

use super::svg::{write_svg_root_bg, write_bg_rect};
use super::svg::{fmt_coord, xml_escape};
use super::svg_richtext::render_creole_text;

// ── Style constants ─────────────────────────────────────────────────

const FONT_SIZE: f64 = 13.0;
const LINE_HEIGHT: f64 = 16.0;
const PARTICIPANT_BG: &str = "#E2E2F0";
const PARTICIPANT_BORDER: &str = "#181818";
const LIFELINE_COLOR: &str = "#181818";
const ARROW_COLOR: &str = "#181818";
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const GROUP_BG: &str = "#EEEEEE";
const GROUP_BORDER: &str = "#000000";
const ACTIVATION_BG: &str = "#FFFFFF";
const ACTIVATION_BORDER: &str = "#181818";
const FRAGMENT_BG: &str = "#F1F1F1";
const FRAGMENT_BORDER: &str = "#181818";
const REF_BG: &str = "#F1F1F1";
const REF_BORDER: &str = "#181818";
const DIVIDER_COLOR: &str = "#888888";
const TEXT_COLOR: &str = "#000000";

const MARGIN: f64 = 5.0;

// ── Arrow marker defs ───────────────────────────────────────────────

fn write_seq_defs(buf: &mut String) {
    buf.push_str("<defs/>");
}

// ── Lifelines ───────────────────────────────────────────────────────

fn draw_lifelines(buf: &mut String, layout: &SeqLayout, skin: &SkinParams, sd: &SequenceDiagram) {
    let ll_color = skin.sequence_lifeline_border_color(LIFELINE_COLOR);
    for (i, p) in layout.participants.iter().enumerate() {
        let part_idx = i + 1;
        let display = sd
            .participants
            .get(i)
            .and_then(|pp| pp.display_name.as_deref())
            .unwrap_or(&p.name);
        let escaped_name = xml_escape(display);
        let ll_height = layout.lifeline_bottom - layout.lifeline_top;

        write!(
            buf,
            r#"<g class="participant-lifeline" data-entity-uid="part{idx}" data-qualified-name="{name}" id="part{idx}-lifeline"><g><title>{name}</title>"#,
            idx = part_idx,
            name = escaped_name,
        )
        .unwrap();

        // Java lifeline position: box_x + (int)(box_width) / 2 (Java integer division)
        // This differs from exact center (box_x + box_width/2.0) due to truncation
        let box_x = p.x - p.box_width / 2.0;
        let lifeline_x = box_x + (p.box_width as i32 / 2) as f64;

        // Transparent click-target rect over lifeline
        let _ = write!(
            buf,
            "<rect fill=\"#000000\" fill-opacity=\"0.00000\" height=\"{h}\" width=\"8\" x=\"{x}\" y=\"{y}\"/>",
            h = fmt_coord(ll_height),
            x = fmt_coord(p.x - 4.0),
            y = fmt_coord(layout.lifeline_top),
        );

        // Dashed lifeline
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:0.5;stroke-dasharray:5,5;" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
            x = fmt_coord(lifeline_x),
            y1 = fmt_coord(layout.lifeline_top),
            y2 = fmt_coord(layout.lifeline_bottom),
            color = ll_color,
        )
        .unwrap();

        buf.push_str("</g></g>");
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
    let text_width = font_metrics::text_width(name, "SansSerif", 14.0, false, false);
    let padding = 7.0;
    let box_width = text_width + 2.0 * padding;
    let box_height = 30.2969;
    let x = p.x - box_width / 2.0;
    let text_x = x + padding;
    let text_y = y + 19.9951;

    write!(
        buf,
        r#"<rect fill="{bg}" height="{h}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{w}" x="{x}" y="{y}"/>"#,
        h = fmt_coord(box_height),
        w = fmt_coord(box_width),
        x = fmt_coord(x),
        y = fmt_coord(y),
    )
    .unwrap();

    let escaped = xml_escape(name);
    write!(
        buf,
        r#"<text fill="{color}" font-family="sans-serif" font-size="14" lengthAdjust="spacing" textLength="{tl}" x="{tx}" y="{ty}">{text}</text>"#,
        tl = fmt_coord(text_width),
        tx = fmt_coord(text_x),
        ty = fmt_coord(text_y),
        color = text_color,
        text = escaped,
    )
    .unwrap();
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
        r#"<circle cx="{}" cy="{}" fill="none" r="{head_r}" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(cx), fmt_coord(head_cy),
    )
    .unwrap();
    buf.push('\n');

    // Body
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx), fmt_coord(cx), fmt_coord(body_top), fmt_coord(body_bot),
    )
    .unwrap();
    buf.push('\n');

    // Left arm
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx), fmt_coord(cx - arm_spread), fmt_coord(arm_y), fmt_coord(arm_y),
    )
    .unwrap();
    buf.push('\n');

    // Right arm
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx), fmt_coord(cx + arm_spread), fmt_coord(arm_y), fmt_coord(arm_y),
    )
    .unwrap();
    buf.push('\n');

    // Left leg
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx), fmt_coord(cx - leg_spread), fmt_coord(body_bot), fmt_coord(body_bot + leg_drop),
    )
    .unwrap();
    buf.push('\n');

    // Right leg
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(cx), fmt_coord(cx + leg_spread), fmt_coord(body_bot), fmt_coord(body_bot + leg_drop),
    )
    .unwrap();
    buf.push('\n');

    // Name below figure
    let name_y = body_bot + leg_drop + FONT_SIZE + 4.0;
    let escaped = xml_escape(name);
    let tl = fmt_coord(font_metrics::text_width(name, "SansSerif", 14.0, true, false));
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
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
        r#"<circle cx="{}" cy="{}" fill="{bg}" r="{icon_r}" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(icon_cx), fmt_coord(icon_y + icon_r),
    )
    .unwrap();
    buf.push('\n');

    // Vertical line to the left of circle
    {
        let lx = fmt_coord(icon_cx - icon_r - 4.0);
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:1.5;" x1="{lx}" x2="{lx}" y1="{}" y2="{}"/>"#,
            fmt_coord(icon_y), fmt_coord(icon_y + 2.0 * icon_r),
        )
        .unwrap();
    }
    buf.push('\n');

    // Horizontal connector from vertical bar to circle
    {
        let ly = fmt_coord(icon_y + icon_r);
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{ly}" y2="{ly}"/>"#,
            fmt_coord(icon_cx - icon_r - 4.0), fmt_coord(icon_cx - icon_r),
        )
        .unwrap();
    }
    buf.push('\n');

    // Name below
    let name_y = icon_y + 2.0 * icon_r + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    let tl = fmt_coord(font_metrics::text_width(name, "SansSerif", 14.0, true, false));
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
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
        r#"<circle cx="{}" cy="{}" fill="{bg}" r="{icon_r}" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(cx), fmt_coord(icon_cy),
    )
    .unwrap();
    buf.push('\n');

    // Small arrow on top of circle
    let arrow_y = icon_cy - icon_r;
    write!(
        buf,
        r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(cx - 5.0), fmt_coord(arrow_y - 6.0),
        fmt_coord(cx + 2.0), fmt_coord(arrow_y - 1.0),
        fmt_coord(cx - 5.0), fmt_coord(arrow_y + 3.0),
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = icon_cy + icon_r + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    let tl = fmt_coord(font_metrics::text_width(name, "SansSerif", 14.0, true, false));
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
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
        r#"<circle cx="{}" cy="{}" fill="{bg}" r="{icon_r}" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(cx), fmt_coord(icon_cy),
    )
    .unwrap();
    buf.push('\n');

    // Horizontal underline
    let line_y = icon_cy + icon_r + 2.0;
    {
        let ly = fmt_coord(line_y);
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{ly}" y2="{ly}"/>"#,
            fmt_coord(cx - icon_r), fmt_coord(cx + icon_r),
        )
        .unwrap();
    }
    buf.push('\n');

    // Name below
    let name_y = line_y + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    let tl = fmt_coord(font_metrics::text_width(name, "SansSerif", 14.0, true, false));
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
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
    {
        let rx_s = fmt_coord(cyl_w / 2.0);
        let ry_s = fmt_coord(arc_h);
        write!(
            buf,
            r#"<path d="M{},{} A{rx_s},{ry_s} 0 0,0 {},{} L{},{} A{rx_s},{ry_s} 0 0,0 {},{} Z " fill="{bg}" style="stroke:{border};stroke-width:1.5;"/>"#,
            fmt_coord(cyl_x), fmt_coord(cyl_y + arc_h),
            fmt_coord(cyl_x + cyl_w), fmt_coord(cyl_y + arc_h),
            fmt_coord(cyl_x + cyl_w), fmt_coord(cyl_y + cyl_h),
            fmt_coord(cyl_x), fmt_coord(cyl_y + cyl_h),
        )
        .unwrap();
    }
    buf.push('\n');

    // Top ellipse
    write!(
        buf,
        r#"<ellipse cx="{}" cy="{}" fill="{bg}" rx="{}" ry="{}" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(cx), fmt_coord(cyl_y + arc_h),
        fmt_coord(cyl_w / 2.0), fmt_coord(arc_h),
    )
    .unwrap();
    buf.push('\n');

    // Name below cylinder
    let name_y = cyl_y + cyl_h + arc_h + FONT_SIZE + 4.0;
    let escaped = xml_escape(name);
    let tl = fmt_coord(font_metrics::text_width(name, "SansSerif", 14.0, true, false));
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
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
        r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(rect_h), fmt_coord(rect_w), fmt_coord(rx + offset), fmt_coord(ry - offset),
    )
    .unwrap();
    buf.push('\n');

    // Front (main) rectangle
    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(rect_h), fmt_coord(rect_w), fmt_coord(rx), fmt_coord(ry),
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = ry + rect_h + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    let tl = fmt_coord(font_metrics::text_width(name, "SansSerif", 14.0, true, false));
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
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
    {
        let lx = fmt_coord(cyl_x);
        let ty_s = fmt_coord(cyl_y);
        let rx_s = fmt_coord(cyl_x + cyl_w - arc_w);
        let aw = fmt_coord(arc_w);
        let ah = fmt_coord(cyl_h / 2.0);
        let by = fmt_coord(cyl_y + cyl_h);
        write!(
            buf,
            r#"<path d="M{lx},{ty_s} L{rx_s},{ty_s} A{aw},{ah} 0 0,1 {rx_s},{by} L{lx},{by} A{aw},{ah} 0 0,1 {lx},{ty_s} Z " fill="{bg}" style="stroke:{border};stroke-width:1.5;"/>"#,
        )
        .unwrap();
    }
    buf.push('\n');

    // Right end cap ellipse
    write!(
        buf,
        r#"<ellipse cx="{}" cy="{}" fill="{bg}" rx="{}" ry="{}" style="stroke:{border};stroke-width:1.5;"/>"#,
        fmt_coord(cyl_x + cyl_w - arc_w), fmt_coord(cyl_y + cyl_h / 2.0),
        fmt_coord(arc_w), fmt_coord(cyl_h / 2.0),
    )
    .unwrap();
    buf.push('\n');

    // Name below
    let name_y = cyl_y + cyl_h + FONT_SIZE + 6.0;
    let escaped = xml_escape(name);
    let tl = fmt_coord(font_metrics::text_width(name, "SansSerif", 14.0, true, false));
    write!(
        buf,
        r#"<text fill="{text_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(cx), fmt_coord(name_y),
    )
    .unwrap();
    buf.push('\n');
}

// ── Messages ────────────────────────────────────────────────────────

fn draw_message(
    buf: &mut String,
    msg: &MessageLayout,
    arrow_color: &str,
    arrow_thickness: f64,
    from_idx: usize,
    to_idx: usize,
    msg_idx: usize,
) {
    write!(
        buf,
        r#"<g class="message" data-entity-1="part{}" data-entity-2="part{}" id="msg{}">"#,
        from_idx, to_idx, msg_idx,
    )
    .unwrap();

    let sw = arrow_thickness as u32;

    // Determine arrow tip position and line endpoints
    // Java insets the arrow tip 2px from the participant center
    let (tip_x, line_x1, line_x2) = if msg.is_left {
        // Right-to-left: arrow points left
        (msg.to_x + 2.0, msg.from_x, msg.to_x)
    } else {
        // Left-to-right: arrow points right
        (msg.to_x - 2.0, msg.from_x, msg.to_x)
    };

    // Draw inline polygon arrowhead
    if msg.has_open_head {
        // Open arrowhead: just two lines forming a V
        let (ax1, ax2) = if msg.is_left {
            (tip_x + 10.0, tip_x + 10.0)
        } else {
            (tip_x - 10.0, tip_x - 10.0)
        };
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
            color = arrow_color,
            ax = fmt_coord(ax1),
            tx = fmt_coord(tip_x),
            y1 = fmt_coord(msg.y - 4.0),
            y = fmt_coord(msg.y),
        )
        .unwrap();
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
            color = arrow_color,
            ax = fmt_coord(ax2),
            tx = fmt_coord(tip_x),
            y1 = fmt_coord(msg.y + 4.0),
            y = fmt_coord(msg.y),
        )
        .unwrap();
    } else {
        // Filled arrowhead polygon: 4-point diamond with inner point 6px from tip
        let (p1x, p2x, p3x, p4x) = if msg.is_left {
            (tip_x + 10.0, tip_x, tip_x + 10.0, tip_x + 6.0)
        } else {
            (tip_x - 10.0, tip_x, tip_x - 10.0, tip_x - 6.0)
        };
        write!(
            buf,
            r#"<polygon fill="{color}" points="{p1x},{p1y},{p2x},{p2y},{p3x},{p3y},{p4x},{p4y}" style="stroke:{color};stroke-width:{sw};"/>"#,
            color = arrow_color,
            p1x = fmt_coord(p1x),
            p1y = fmt_coord(msg.y - 4.0),
            p2x = fmt_coord(p2x),
            p2y = fmt_coord(msg.y),
            p3x = fmt_coord(p3x),
            p3y = fmt_coord(msg.y + 4.0),
            p4x = fmt_coord(p4x),
            p4y = fmt_coord(msg.y),
        )
        .unwrap();
    }

    // Message line
    let dash_style = if msg.is_dashed {
        "stroke-dasharray:2,2;"
    } else {
        ""
    };
    // Line stops at polygon inner edge (4px from tip)
    let adjusted_x2 = if msg.has_open_head {
        tip_x
    } else if msg.is_left {
        tip_x + 4.0
    } else {
        tip_x - 4.0
    };
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x1}" x2="{x2}" y1="{y}" y2="{y}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x1 = fmt_coord(line_x1),
        x2 = fmt_coord(adjusted_x2),
        y = fmt_coord(msg.y),
    )
    .unwrap();

    // Label text above the line — each line as a separate <text> element
    if !msg.text.is_empty() {
        let text_x = if msg.from_x < msg.to_x {
            msg.from_x + 7.0
        } else {
            msg.to_x + 7.0
        };
        let msg_line_spacing = 15.1328; // ascent + descent at font-size 13
        let num_lines = msg.text_lines.len();
        let first_text_y = msg.y - 5.0659 - (num_lines as f64 - 1.0) * msg_line_spacing;
        for (i, line) in msg.text_lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let line_y = first_text_y + i as f64 * msg_line_spacing;
            let tl = font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false);
            write!(
                buf,
                r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(tl),
                fmt_coord(text_x),
                fmt_coord(line_y),
                xml_escape(line),
            )
            .unwrap();
        }
    }

    buf.push_str("</g>");
}

fn draw_self_message(
    buf: &mut String,
    msg: &MessageLayout,
    arrow_color: &str,
    arrow_thickness: f64,
    from_idx: usize,
    msg_idx: usize,
) {
    let sw = arrow_thickness as u32;
    let x = msg.from_x;
    let y = msg.y;
    let loop_width = 47.0;
    let loop_height = 13.0;

    write!(
        buf,
        r#"<g class="message" data-entity-1="part{}" data-entity-2="part{}" id="msg{}">"#,
        from_idx, from_idx, msg_idx,
    )
    .unwrap();

    let dash_style = if msg.is_dashed {
        "stroke-dasharray:2,2;"
    } else {
        ""
    };

    // 3-line self-message: horizontal right, vertical down, horizontal left
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x1}" x2="{x2}" y1="{y1}" y2="{y1}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x1 = fmt_coord(x),
        x2 = fmt_coord(x + loop_width),
        y1 = fmt_coord(y),
    )
    .unwrap();

    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x = fmt_coord(x + loop_width),
        y1 = fmt_coord(y),
        y2 = fmt_coord(y + loop_height),
    )
    .unwrap();

    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x1}" x2="{x2}" y1="{y}" y2="{y}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x1 = fmt_coord(x + 1.0),
        x2 = fmt_coord(x + loop_width),
        y = fmt_coord(y + loop_height),
    )
    .unwrap();

    // Polygon arrowhead pointing left at return
    if msg.has_open_head {
        let tip_x = x;
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
            color = arrow_color,
            ax = fmt_coord(tip_x + 10.0),
            tx = fmt_coord(tip_x),
            y1 = fmt_coord(y + loop_height - 4.0),
            y = fmt_coord(y + loop_height),
        )
        .unwrap();
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
            color = arrow_color,
            ax = fmt_coord(tip_x + 10.0),
            tx = fmt_coord(tip_x),
            y1 = fmt_coord(y + loop_height + 4.0),
            y = fmt_coord(y + loop_height),
        )
        .unwrap();
    } else {
        let tip_x = x + 1.0;
        let ret_y = y + loop_height;
        write!(
            buf,
            r#"<polygon fill="{color}" points="{p1x},{p1y},{p2x},{p2y},{p3x},{p3y},{p4x},{p4y}" style="stroke:{color};stroke-width:{sw};"/>"#,
            color = arrow_color,
            p1x = fmt_coord(tip_x + 10.0),
            p1y = fmt_coord(ret_y - 4.0),
            p2x = fmt_coord(tip_x),
            p2y = fmt_coord(ret_y),
            p3x = fmt_coord(tip_x + 10.0),
            p3y = fmt_coord(ret_y + 4.0),
            p4x = fmt_coord(tip_x + 4.0),
            p4y = fmt_coord(ret_y),
        )
        .unwrap();
    }

    // Label text above the first horizontal line — each line as separate <text>
    if !msg.text.is_empty() {
        let text_x = x + 7.0;
        let msg_line_spacing = 15.1328;
        let num_lines = msg.text_lines.len();
        let first_text_y = y - 5.0659 - (num_lines as f64 - 1.0) * msg_line_spacing;
        for (i, line) in msg.text_lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let line_y = first_text_y + i as f64 * msg_line_spacing;
            let tl = font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false);
            write!(
                buf,
                r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(tl),
                fmt_coord(text_x),
                fmt_coord(line_y),
                xml_escape(line),
            )
            .unwrap();
        }
    }

    buf.push_str("</g>");
}

// ── Activation bars ─────────────────────────────────────────────────

fn draw_activation(buf: &mut String, act: &ActivationLayout) {
    let width = 10.0;
    let height = act.y_end - act.y_start;

    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(height), fmt_coord(width), fmt_coord(act.x), fmt_coord(act.y_start),
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
        r#"<line style="stroke:{color};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(d.x - size), fmt_coord(d.x + size),
        fmt_coord(d.y - size), fmt_coord(d.y + size),
        color = ARROW_COLOR,
    )
    .unwrap();
    buf.push('\n');

    // Second diagonal: top-right to bottom-left
    write!(
        buf,
        r#"<line style="stroke:{color};stroke-width:2;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(d.x + size), fmt_coord(d.x - size),
        fmt_coord(d.y - size), fmt_coord(d.y + size),
        color = ARROW_COLOR,
    )
    .unwrap();
    buf.push('\n');
}

// ── Notes ───────────────────────────────────────────────────────────

fn draw_note(buf: &mut String, note: &NoteLayout) {
    let fold = 10.0; // folded corner size

    // Background rect
    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(note.height), fmt_coord(note.width), fmt_coord(note.x), fmt_coord(note.y),
        bg = NOTE_BG,
        border = NOTE_BORDER,
    )
    .unwrap();
    buf.push('\n');

    // Folded corner triangle in top-right
    let cx = note.x + note.width - fold;
    let cy = note.y;
    {
        let cx_s = fmt_coord(cx);
        let cy_s = fmt_coord(cy);
        let cy2 = fmt_coord(cy + fold);
        let cx2 = fmt_coord(note.x + note.width);
        write!(
            buf,
            r#"<path d="M{cx_s},{cy_s} L{cx_s},{cy2} L{cx2},{cy_s} Z " fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
            bg = NOTE_BG,
            border = NOTE_BORDER,
        )
        .unwrap();
    }
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
        r#"<rect fill="{bg}" fill-opacity="0.30000" height="{}" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(height), fmt_coord(group.width), fmt_coord(group.x), fmt_coord(group.y_start),
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
        let label_width =
            font_metrics::text_width(label, "SansSerif", FONT_SIZE, true, false) + 12.0;
        let label_height = FONT_SIZE + 6.0;
        write!(
            buf,
            r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(label_height), fmt_coord(label_width), fmt_coord(group.x), fmt_coord(group.y_start),
            bg = GROUP_BG,
            border = GROUP_BORDER,
        )
        .unwrap();
        buf.push('\n');

        let tl = fmt_coord(font_metrics::text_width(label, "SansSerif", FONT_SIZE, true, false));
        write!(
            buf,
            r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" lengthAdjust="spacing" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(label_x), fmt_coord(label_y),
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
        r#"<rect fill="{bg}" fill-opacity="0.10000" height="{}" rx="2" style="stroke:{border};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(frag.height), fmt_coord(frag.width), fmt_coord(frag.x), fmt_coord(frag.y),
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
    let tab_width = font_metrics::text_width(&tab_text, "SansSerif", FONT_SIZE, true, false) + 16.0;
    let tab_height = FONT_SIZE + 8.0;
    let notch = 6.0;

    // Pentagon path: top-left corner with a notch at bottom-right
    {
        let fx = fmt_coord(frag.x);
        let fy = fmt_coord(frag.y);
        write!(
            buf,
            r#"<path d="M{fx},{fy} L{},{fy} L{},{} L{},{} L{fx},{} Z " fill="{bg}" style="stroke:{border};stroke-width:1.5;"/>"#,
            fmt_coord(frag.x + tab_width),
            fmt_coord(frag.x + tab_width), fmt_coord(frag.y + tab_height - notch),
            fmt_coord(frag.x + tab_width - notch), fmt_coord(frag.y + tab_height),
            fmt_coord(frag.y + tab_height),
            bg = FRAGMENT_BG,
            border = FRAGMENT_BORDER,
        )
        .unwrap();
    }
    buf.push('\n');

    // Kind label text
    let text_x = frag.x + 6.0;
    let text_y = frag.y + FONT_SIZE + 2.0;
    let escaped = xml_escape(&tab_text);
    let frag_tl = fmt_coord(font_metrics::text_width(&tab_text, "SansSerif", FONT_SIZE, true, false));
    write!(
        buf,
        r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" lengthAdjust="spacing" textLength="{frag_tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(text_x), fmt_coord(text_y),
    )
    .unwrap();
    buf.push('\n');

    // Separator lines (else)
    for (sep_y, sep_label) in &frag.separators {
        // Dashed horizontal line
        {
            let y_s = fmt_coord(*sep_y);
            write!(
                buf,
                r#"<line style="stroke:{border};stroke-width:1;stroke-dasharray:5,5;" x1="{}" x2="{}" y1="{y_s}" y2="{y_s}"/>"#,
                fmt_coord(frag.x), fmt_coord(frag.x + frag.width),
                border = FRAGMENT_BORDER,
            )
            .unwrap();
        }
        buf.push('\n');

        // Separator label
        if !sep_label.is_empty() {
            let label_x = frag.x + 10.0;
            let label_y = sep_y + FONT_SIZE + 2.0;
            let escaped_label = xml_escape(sep_label);
            let bracket_text = format!("[{}]", sep_label);
            let sep_tl = fmt_coord(font_metrics::text_width(&bracket_text, "SansSerif", FONT_SIZE, false, true));
            write!(
                buf,
                r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-style="italic" lengthAdjust="spacing" textLength="{sep_tl}" x="{}" y="{}">[{escaped_label}]</text>"#,
                fmt_coord(label_x), fmt_coord(label_y),
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
        r#"<rect fill="{color}" fill-opacity="0.20000" height="5" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(divider.width), fmt_coord(divider.x), fmt_coord(center_y - 2.5),
        color = DIVIDER_COLOR,
    )
    .unwrap();
    buf.push('\n');

    // Horizontal lines
    {
        let y1 = fmt_coord(center_y - 2.5);
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:1;" x1="{}" x2="{}" y1="{y1}" y2="{y1}"/>"#,
            fmt_coord(divider.x), fmt_coord(divider.x + divider.width),
            color = DIVIDER_COLOR,
        )
        .unwrap();
    }
    buf.push('\n');
    {
        let y2 = fmt_coord(center_y + 2.5);
        write!(
            buf,
            r#"<line style="stroke:{color};stroke-width:1;" x1="{}" x2="{}" y1="{y2}" y2="{y2}"/>"#,
            fmt_coord(divider.x), fmt_coord(divider.x + divider.width),
            color = DIVIDER_COLOR,
        )
        .unwrap();
    }
    buf.push('\n');

    // Centered label text
    if let Some(text) = &divider.text {
        let mid_x = divider.x + divider.width / 2.0;
        let text_y = center_y + FONT_SIZE * 0.35;
        let escaped = xml_escape(text);

        // Text background
        let text_width =
            font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false) + 16.0;
        write!(
            buf,
            r#"<rect fill="white" height="{}" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(FONT_SIZE * 1.2), fmt_coord(text_width),
            fmt_coord(mid_x - text_width / 2.0), fmt_coord(center_y - FONT_SIZE * 0.6),
        )
        .unwrap();
        buf.push('\n');

        let div_tl = fmt_coord(font_metrics::text_width(text, "SansSerif", FONT_SIZE, true, false));
        write!(
            buf,
            r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{div_tl}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(mid_x), fmt_coord(text_y),
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
            r#"<circle cx="{}" cy="{}" fill="{color}" r="1.5"/>"#,
            fmt_coord(mid_x), fmt_coord(center_y + dy),
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
        let tl = fmt_coord(font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false));
        write!(
            buf,
            r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" lengthAdjust="spacing" textLength="{tl}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(text_x), fmt_coord(text_y),
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
        r#"<rect fill="{bg}" height="{}" rx="2" style="stroke:{border};stroke-width:1.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(r.height), fmt_coord(r.width), fmt_coord(r.x), fmt_coord(r.y),
        bg = REF_BG,
        border = REF_BORDER,
    )
    .unwrap();
    buf.push('\n');

    // "ref" label tab in top-left
    let tab_width = font_metrics::text_width("ref", "SansSerif", FONT_SIZE, true, false) + 12.0;
    let tab_height = FONT_SIZE + 6.0;
    let notch = 5.0;
    {
        let rx_s = fmt_coord(r.x);
        let ry_s = fmt_coord(r.y);
        write!(
            buf,
            r#"<path d="M{rx_s},{ry_s} L{},{ry_s} L{},{} L{},{} L{rx_s},{} Z " fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
            fmt_coord(r.x + tab_width),
            fmt_coord(r.x + tab_width), fmt_coord(r.y + tab_height - notch),
            fmt_coord(r.x + tab_width - notch), fmt_coord(r.y + tab_height),
            fmt_coord(r.y + tab_height),
            bg = REF_BG,
            border = REF_BORDER,
        )
        .unwrap();
    }
    buf.push('\n');

    let ref_tl = fmt_coord(font_metrics::text_width("ref", "SansSerif", FONT_SIZE, true, false));
    write!(
        buf,
        r#"<text fill="{color}" font-family="sans-serif" font-size="{FONT_SIZE}" font-weight="bold" lengthAdjust="spacing" textLength="{ref_tl}" x="{}" y="{}">ref</text>"#,
        fmt_coord(r.x + 5.0), fmt_coord(r.y + FONT_SIZE + 1.0),
        color = TEXT_COLOR,
    )
    .unwrap();
    buf.push('\n');

    // Centered label text
    let mid_x = r.x + r.width / 2.0;
    let mid_y = r.y + r.height / 2.0 + FONT_SIZE * 0.35;
    let escaped = xml_escape(&r.label);
    let label_tl = fmt_coord(font_metrics::text_width(&r.label, "SansSerif", FONT_SIZE, false, false));
    write!(
        buf,
        r#"<text fill="{TEXT_COLOR}" font-family="sans-serif" font-size="{FONT_SIZE}" lengthAdjust="spacing" text-anchor="middle" textLength="{label_tl}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(mid_x), fmt_coord(mid_y),
    )
    .unwrap();
    buf.push('\n');
}

// ── Public entry point ──────────────────────────────────────────────

/// Build a mapping from participant name -> 1-based index for data-entity-uid.
fn build_participant_index(sd: &SequenceDiagram) -> std::collections::HashMap<String, usize> {
    sd.participants
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.clone(), i + 1))
        .collect()
}

/// Render a SequenceDiagram + SeqLayout into an SVG string.
pub fn render_sequence(
    sd: &SequenceDiagram,
    layout: &SeqLayout,
    skin: &SkinParams,
) -> Result<String> {
    // Layout already includes margins in total dimensions
    let svg_w = layout.total_width;
    let svg_h = layout.total_height;

    let mut buf = String::with_capacity(4096);

    // 1. SVG header
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, svg_w, svg_h, "SEQUENCE", bg);

    // 2. Defs (empty)
    write_seq_defs(&mut buf);
    buf.push_str("<g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);

    // Build participant name -> index mapping
    let part_index = build_participant_index(sd);

    // 3. Lifelines (dashed vertical lines with semantic grouping)
    draw_lifelines(&mut buf, layout, skin, sd);

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

    // Build a name -> display_name lookup from the diagram
    let display_names: std::collections::HashMap<&str, &str> = sd
        .participants
        .iter()
        .filter_map(|p| p.display_name.as_deref().map(|dn| (p.name.as_str(), dn)))
        .collect();

    let part_bg = skin.background_color("participant", PARTICIPANT_BG);
    let part_border = skin.border_color("participant", PARTICIPANT_BORDER);
    let part_font = skin.font_color("participant", TEXT_COLOR);

    // 6. Participant head + tail boxes (interleaved per participant, matching Java order)
    let top_y = MARGIN;
    let bottom_y = layout.lifeline_bottom - 1.0;
    for (i, p) in layout.participants.iter().enumerate() {
        let part_idx = i + 1;
        let dn = display_names.get(p.name.as_str()).copied();
        let display = dn.unwrap_or(&p.name);
        let escaped_name = xml_escape(display);

        // Head
        write!(
            buf,
            r#"<g class="participant participant-head" data-entity-uid="part{idx}" data-qualified-name="{name}" id="part{idx}-head">"#,
            idx = part_idx,
            name = escaped_name,
        )
        .unwrap();
        draw_participant_box(&mut buf, p, top_y, dn, part_bg, part_border, part_font);
        buf.push_str("</g>");

        // Tail
        write!(
            buf,
            r#"<g class="participant participant-tail" data-entity-uid="part{idx}" data-qualified-name="{name}" id="part{idx}-tail">"#,
            idx = part_idx,
            name = escaped_name,
        )
        .unwrap();
        draw_participant_box(&mut buf, p, bottom_y, dn, part_bg, part_border, part_font);
        buf.push_str("</g>");
    }

    // 8. Messages (with optional autonumber)
    let seq_arrow_color = skin.sequence_arrow_color(ARROW_COLOR);
    let seq_arrow_thickness = skin.sequence_arrow_thickness().unwrap_or(1.0);
    let mut msg_counter: u32 = layout.autonumber_start;
    let mut msg_seq_counter: usize = 0;
    for msg in &layout.messages {
        msg_seq_counter += 1;
        // Find participant indices for from/to
        let from_idx = find_participant_idx_by_x(&layout.participants, msg.from_x, &part_index);
        let to_idx = if msg.is_self {
            from_idx
        } else {
            find_participant_idx_by_x(&layout.participants, msg.to_x, &part_index)
        };

        if layout.autonumber_enabled {
            let numbered_text = if msg.text.is_empty() {
                format!("{msg_counter}")
            } else {
                format!("{} {}", msg_counter, msg.text)
            };
            let numbered_lines: Vec<String> = if msg.text_lines.is_empty() {
                vec![numbered_text.clone()]
            } else {
                let mut lines = msg.text_lines.clone();
                lines[0] = if lines[0].is_empty() {
                    format!("{msg_counter}")
                } else {
                    format!("{} {}", msg_counter, lines[0])
                };
                lines
            };
            let numbered_msg = MessageLayout {
                text: numbered_text,
                text_lines: numbered_lines,
                ..msg.clone()
            };
            if numbered_msg.is_self {
                draw_self_message(
                    &mut buf,
                    &numbered_msg,
                    seq_arrow_color,
                    seq_arrow_thickness,
                    from_idx,
                    msg_seq_counter,
                );
            } else {
                draw_message(
                    &mut buf,
                    &numbered_msg,
                    seq_arrow_color,
                    seq_arrow_thickness,
                    from_idx,
                    to_idx,
                    msg_seq_counter,
                );
            }
            msg_counter += 1;
        } else if msg.is_self {
            draw_self_message(
                &mut buf,
                msg,
                seq_arrow_color,
                seq_arrow_thickness,
                from_idx,
                msg_seq_counter,
            );
        } else {
            draw_message(
                &mut buf,
                msg,
                seq_arrow_color,
                seq_arrow_thickness,
                from_idx,
                to_idx,
                msg_seq_counter,
            );
        }
    }

    // 9. Notes
    for note in &layout.notes {
        draw_note(&mut buf, note);
    }

    // 10. Destroy markers
    for d in &layout.destroys {
        draw_destroy(&mut buf, d);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

/// Find the 1-based participant index whose center x is closest to the given x.
fn find_participant_idx_by_x(
    participants: &[ParticipantLayout],
    x: f64,
    part_index: &std::collections::HashMap<String, usize>,
) -> usize {
    let mut best_idx = 1;
    let mut best_dist = f64::MAX;
    for p in participants {
        let dist = (p.x - x).abs();
        if dist < best_dist {
            best_dist = dist;
            if let Some(&idx) = part_index.get(&p.name) {
                best_idx = idx;
            }
        }
    }
    best_idx
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
    fn self_message_renders_lines_and_polygon() {
        let sd = SequenceDiagram {
            participants: vec![make_participant("A")],
            events: vec![SeqEvent::Message(make_message("A", "A", "self call"))],
        };
        let layout = crate::layout::sequence::layout_sequence(&sd).unwrap();
        let svg = render_sequence(&sd, &layout, &SkinParams::default()).expect("render failed");
        // Self-message uses 3 lines + polygon (Java PlantUML style)
        assert!(
            svg.contains("<polygon"),
            "SVG must contain <polygon for self-message arrow"
        );
        assert!(
            svg.contains("self call"),
            "SVG must contain self-message text"
        );
        assert!(
            svg.contains(r#"class="message""#),
            "SVG must contain message group"
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
        // Open-head message now uses inline lines (not SVG markers)
        // Verify the message group wrapper exists
        assert!(
            svg.contains(r#"class="message""#),
            "open-head message must be wrapped in message group"
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
