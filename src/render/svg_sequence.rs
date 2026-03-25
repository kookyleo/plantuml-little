use std::fmt::Write;

use crate::layout::sequence::{
    ActivationLayout, DelayLayout, DestroyLayout, DividerLayout, FragmentLayout, GroupLayout,
    MessageLayout, NoteLayout, ParticipantLayout, RefLayout, SeqLayout,
};
use crate::model::sequence::{FragmentKind, ParticipantKind, SeqArrowHead};
use crate::model::SequenceDiagram;
use crate::style::SkinParams;
use crate::Result;

use crate::font_metrics;

use super::svg::{write_svg_root_bg, write_bg_rect, ensure_visible_int};
use crate::klimt::svg::{fmt_coord, xml_escape, SvgGraphic, LengthAdjust};
use super::svg_richtext::{disable_path_sprites, enable_path_sprites, render_creole_text, set_default_font_family, take_back_filters};

// ── Style constants ─────────────────────────────────────────────────

const FONT_SIZE: f64 = 13.0;
use crate::skin::rose::{
    ACTIVATION_BG, BORDER_COLOR, DESTROY_COLOR, GROUP_BG, NOTE_BG,
    NOTE_BORDER, PARTICIPANT_BG, TEXT_COLOR,
};

const MARGIN: f64 = 5.0;

// Fragment tab geometry (from Java AWT font metrics)
const FRAG_TAB_LEFT_PAD: f64 = 15.0;
const FRAG_TAB_RIGHT_PAD: f64 = 30.0;
const FRAG_TAB_HEIGHT: f64 = 17.1328;
const FRAG_TAB_NOTCH: f64 = 10.0;
const FRAG_KIND_LABEL_Y_OFFSET: f64 = 13.0669;
const FRAG_GUARD_FONT_SIZE: f64 = 11.0;
const FRAG_GUARD_GAP: f64 = 15.0;
const FRAG_GUARD_LABEL_Y_OFFSET: f64 = 12.2104;
const FRAG_SEP_LABEL_Y_OFFSET: f64 = 10.2104;

const DELAY_FONT_SIZE: f64 = 11.0;

const REF_TAB_HEIGHT: f64 = 17.0;
const REF_TAB_NOTCH: f64 = 10.0;
const REF_TAB_LEFT_PAD: f64 = 13.0;
const REF_KIND_LABEL_Y_OFFSET: f64 = 14.0669;
const REF_LABEL_FONT_SIZE: f64 = 12.0;
const REF_FRAME_STROKE: &str = "#000000";

fn svg_font_family_attr(font_family: &str) -> &str {
    match font_family {
        "SansSerif" => "sans-serif",
        "Serif" => "serif",
        "Monospaced" => "monospace",
        _ => font_family,
    }
}

// ── Arrow marker defs ───────────────────────────────────────────────

fn write_seq_defs(sg: &mut SvgGraphic) {
    sg.push_raw("<defs/>");
}

/// Encode a `[[url text]]` link for the SVG `<title>` element.
/// Java replaces `:`, `/`, `\` with `.` and wraps with `..` prefix/suffix.
fn encode_link_title(url: &str, display_text: &str) -> String {
    let encoded_url: String = url.chars().map(|c| match c {
        ':' | '/' | '\\' => '.',
        _ => c,
    }).collect();
    let encoded_text: String = display_text.chars().map(|c| match c {
        '\\' => '.',
        _ => c,
    }).collect();
    format!("..{encoded_url} {encoded_text}..")
}

// ── Lifelines ───────────────────────────────────────────────────────

/// Compute lifeline invisible rect height from layout bounds.
///
/// Java's `LivingParticipantBox` accumulates its preferred-size dimension
/// through multiple `addDim()` calls.  When the diagram contains a `group`
/// fragment, the grouping header's dimension causes an additional f32
fn draw_lifelines(sg: &mut SvgGraphic, layout: &SeqLayout, skin: &SkinParams, sd: &SequenceDiagram) {
    let ll_color = skin.sequence_lifeline_border_color(BORDER_COLOR);
    // Collect delay break segments sorted by y
    let mut delay_breaks: Vec<(f64, f64)> = layout.delays.iter()
        .map(|d| (d.lifeline_break_y, d.lifeline_break_y + d.height))
        .collect();
    delay_breaks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for (i, p) in layout.participants.iter().enumerate() {
        let part_idx = i + 1;
        let qualified_name = xml_escape(&p.name);
        let participant = sd.participants.get(i);
        let display = participant
            .and_then(|pp| pp.display_name.as_deref())
            .unwrap_or(&p.name);
        let title_text = if let Some(url) = participant.and_then(|pp| pp.link_url.as_deref()) {
            encode_link_title(url, display)
        } else {
            xml_escape(display)
        };

        let src_line_attr = sd.participants.get(i)
            .and_then(|pp| pp.source_line)
            .map(|sl| format!(r#" data-source-line="{sl}""#))
            .unwrap_or_default();

        // Java lifeline position: box_x + (int)(box_width) / 2 (Java integer division)
        let box_x = p.x - p.box_width / 2.0;
        let lifeline_x = box_x + (p.box_width as i32 / 2) as f64;
        let rect_x = p.x - 4.0;

        if sd.teoz_mode {
            // Teoz: single segment, no delay splitting
            let ll_height = layout.lifeline_bottom - layout.lifeline_top;
            let mut tmp = String::new();
            write!(tmp, "<g><title>{dname}</title>", dname = title_text).unwrap();
            sg.push_raw(&tmp);

            let mut tmp = String::new();
            let _ = write!(
                tmp,
                "<rect fill=\"#000000\" fill-opacity=\"0.00000\" height=\"{h}\" width=\"8\" x=\"{x}\" y=\"{y}\"/>",
                h = fmt_coord(ll_height), x = fmt_coord(rect_x), y = fmt_coord(layout.lifeline_top),
            );
            sg.push_raw(&tmp);

            let mut tmp = String::new();
            write!(
                tmp,
                r#"<line style="stroke:{color};stroke-width:0.5;stroke-dasharray:5,5;" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
                x = fmt_coord(lifeline_x),
                y1 = fmt_coord(layout.lifeline_top),
                y2 = fmt_coord(layout.lifeline_bottom),
                color = ll_color,
            ).unwrap();
            sg.push_raw(&tmp);
            sg.push_raw("</g>");
        } else if delay_breaks.is_empty() {
            // No delays: single continuous lifeline
            let ll_height = layout.lifeline_bottom - layout.lifeline_top;
            let mut tmp = String::new();
            write!(
                tmp,
                r#"<g class="participant-lifeline" data-entity-uid="part{idx}" data-qualified-name="{qname}"{src_line} id="part{idx}-lifeline"><g><title>{dname}</title>"#,
                idx = part_idx, qname = qualified_name, src_line = src_line_attr, dname = title_text,
            ).unwrap();
            sg.push_raw(&tmp);

            let mut tmp = String::new();
            let _ = write!(
                tmp,
                "<rect fill=\"#000000\" fill-opacity=\"0.00000\" height=\"{h}\" width=\"8\" x=\"{x}\" y=\"{y}\"/>",
                h = fmt_coord(ll_height), x = fmt_coord(rect_x), y = fmt_coord(layout.lifeline_top),
            );
            sg.push_raw(&tmp);

            let mut tmp = String::new();
            write!(
                tmp,
                r#"<line style="stroke:{color};stroke-width:0.5;stroke-dasharray:5,5;" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
                x = fmt_coord(lifeline_x),
                y1 = fmt_coord(layout.lifeline_top),
                y2 = fmt_coord(layout.lifeline_bottom),
                color = ll_color,
            ).unwrap();
            sg.push_raw(&tmp);
            sg.push_raw("</g></g>");
        } else {
            // Delays present: split lifeline into segments with delay-style breaks.
            // Java: LivingParticipantBox splits its lifeline at delay segments.
            // Structure:
            //   <g class="participant-lifeline" ...>
            //     <g><title>...</title> <rect/> <line dasharray=5,5/> </g>  -- segment 1
            //     <line dasharray=1,4/>  -- delay break
            //     <g><title>...</title> <rect/> <line dasharray=5,5/> </g>  -- segment 2
            //     ...
            //   </g>
            let mut tmp = String::new();
            write!(
                tmp,
                r#"<g class="participant-lifeline" data-entity-uid="part{idx}" data-qualified-name="{qname}"{src_line} id="part{idx}-lifeline">"#,
                idx = part_idx, qname = qualified_name, src_line = src_line_attr,
            ).unwrap();
            sg.push_raw(&tmp);

            // Build segment boundaries from delays
            let mut seg_start = layout.lifeline_top;
            for &(break_start, break_end) in &delay_breaks {
                // Normal segment before this delay
                let seg_height = break_start - seg_start;
                let mut tmp = String::new();
                write!(tmp, "<g><title>{dname}</title>", dname = title_text).unwrap();
                sg.push_raw(&tmp);

                let mut tmp = String::new();
                let _ = write!(
                    tmp,
                    "<rect fill=\"#000000\" fill-opacity=\"0.00000\" height=\"{h}\" width=\"8\" x=\"{x}\" y=\"{y}\"/>",
                    h = fmt_coord(seg_height), x = fmt_coord(rect_x), y = fmt_coord(seg_start),
                );
                sg.push_raw(&tmp);

                let mut tmp = String::new();
                write!(
                    tmp,
                    r#"<line style="stroke:{color};stroke-width:0.5;stroke-dasharray:5,5;" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
                    x = fmt_coord(lifeline_x),
                    y1 = fmt_coord(seg_start),
                    y2 = fmt_coord(break_start),
                    color = ll_color,
                ).unwrap();
                sg.push_raw(&tmp);
                sg.push_raw("</g>");

                // Delay break line (dotted with stroke-dasharray:1,4)
                let mut tmp = String::new();
                write!(
                    tmp,
                    r#"<line style="stroke:{color};stroke-width:0.5;stroke-dasharray:1,4;" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
                    x = fmt_coord(lifeline_x),
                    y1 = fmt_coord(break_start),
                    y2 = fmt_coord(break_end),
                    color = ll_color,
                ).unwrap();
                sg.push_raw(&tmp);

                seg_start = break_end;
            }

            // Final segment after last delay
            let seg_height = layout.lifeline_bottom - seg_start;
            let mut tmp = String::new();
            write!(tmp, "<g><title>{dname}</title>", dname = title_text).unwrap();
            sg.push_raw(&tmp);

            let mut tmp = String::new();
            let _ = write!(
                tmp,
                "<rect fill=\"#000000\" fill-opacity=\"0.00000\" height=\"{h}\" width=\"8\" x=\"{x}\" y=\"{y}\"/>",
                h = fmt_coord(seg_height), x = fmt_coord(rect_x), y = fmt_coord(seg_start),
            );
            sg.push_raw(&tmp);

            let mut tmp = String::new();
            write!(
                tmp,
                r#"<line style="stroke:{color};stroke-width:0.5;stroke-dasharray:5,5;" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
                x = fmt_coord(lifeline_x),
                y1 = fmt_coord(seg_start),
                y2 = fmt_coord(layout.lifeline_bottom),
                color = ll_color,
            ).unwrap();
            sg.push_raw(&tmp);
            sg.push_raw("</g>");

            sg.push_raw("</g>");
        }
    }
}

// ── Color utilities ─────────────────────────────────────────────────

/// Resolve a color string into SVG fill + optional fill-opacity attributes.
/// Handles: "transparent", "#RRGGBBAA" (8-digit hex), "#RRGGBB", named colors.
fn resolve_fill_attrs(color: &str) -> String {
    let c = color.trim();
    if c.eq_ignore_ascii_case("transparent") || c.eq_ignore_ascii_case("#transparent") {
        return r#"fill="none""#.to_string();
    }
    // 8-digit hex: #RRGGBBAA
    if c.starts_with('#') && c.len() == 9 {
        let rgb = &c[..7];
        if let Ok(alpha) = u8::from_str_radix(&c[7..9], 16) {
            if alpha == 0 {
                return r#"fill="none""#.to_string();
            } else if alpha == 255 {
                return format!(r#"fill="{rgb}""#);
            } else {
                let opacity = alpha as f64 / 255.0;
                return format!(r#"fill="{rgb}" fill-opacity="{opacity:.5}""#);
            }
        }
    }
    format!(r#"fill="{c}""#)
}

// ── Participant box ─────────────────────────────────────────────────

fn draw_participant_box_with_font(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    part_font_family: &str,
    part_font_size: f64,
    head: bool,
    link_url: Option<&str>,
) {
    let fill = p.color.as_deref().unwrap_or(bg);

    match &p.kind {
        ParticipantKind::Actor => {
            draw_participant_actor(sg, p, y, display_name, border, text_color);
        }
        ParticipantKind::Boundary => {
            draw_participant_boundary(sg, p, y, display_name, fill, border, text_color, head);
        }
        ParticipantKind::Control => {
            draw_participant_control(sg, p, y, display_name, fill, border, text_color, head);
        }
        ParticipantKind::Entity => {
            draw_participant_entity(sg, p, y, display_name, fill, border, text_color, head);
        }
        ParticipantKind::Database => {
            draw_participant_database(sg, p, y, display_name, fill, border, text_color, head);
        }
        ParticipantKind::Collections => {
            draw_participant_collections(sg, p, y, display_name, fill, border, text_color, head);
        }
        ParticipantKind::Queue => {
            draw_participant_queue(sg, p, y, display_name, fill, border, text_color, head);
        }
        ParticipantKind::Default => {
            draw_participant_rect_with_font(
                sg,
                p,
                y,
                display_name,
                fill,
                border,
                text_color,
                part_font_family,
                part_font_size,
                link_url,
            );
        }
    }
}

fn draw_participant_rect_with_font(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    font_family: &str,
    font_size: f64,
    link_url: Option<&str>,
) {
    let name = display_name.unwrap_or(&p.name);
    let lines: Vec<&str> = name.split("\\n").flat_map(|s| s.split(crate::NEWLINE_CHAR)).collect();
    let padding = 7.0;
    let box_width = p.box_width;
    let box_height = p.box_height;
    let x = p.x - box_width / 2.0;
    let text_x = x + padding;
    let text_y_base = y + 19.9951 + (font_size - 14.0) * 0.92825;
    let line_h = font_metrics::line_height(font_family, font_size, false, false);
    let svg_font_family = svg_font_family_attr(font_family);

    let fill_attrs = resolve_fill_attrs(bg);
    let mut tmp = String::new();
    write!(
        tmp,
        r#"<rect {fill_attrs} height="{h}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{w}" x="{x}" y="{y}"/>"#,
        h = fmt_coord(box_height),
        w = fmt_coord(box_width),
        x = fmt_coord(x),
        y = fmt_coord(y),
    )
    .unwrap();
    sg.push_raw(&tmp);

    let effective_text_color = if link_url.is_some() { "#0000FF" } else { text_color };
    let text_decoration = if link_url.is_some() { Some("underline") } else { None };

    // Java: each line of multiline text gets its own <a> wrapper
    for (line_idx, line) in lines.iter().enumerate() {
        if let Some(url) = link_url {
            sg.push_raw(&format!(
                r#"<a href="{url}" target="_top" title="{url}" xlink:actuate="onRequest" xlink:href="{url}" xlink:show="new" xlink:title="{url}" xlink:type="simple">"#
            ));
        }
        let text_y = text_y_base + line_idx as f64 * line_h;
        let line_w = font_metrics::text_width(line, font_family, font_size, false, false);
        sg.set_fill_color(effective_text_color);
        sg.svg_text(
            line,
            text_x,
            text_y,
            Some(svg_font_family),
            font_size,
            None,
            None,
            text_decoration,
            line_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        if link_url.is_some() {
            sg.push_raw("</a>");
        }
    }
}

/// Actor: Java renders TEXT first (plain), then ELLIPSE (filled head),
/// then single PATH (body+arms+legs). Stroke-width=0.5.
///
/// Java ActorStickMan constants:
///   headDiam=16, bodyLength=27, armsY=8, armsLength=13, legsX=13, legsY=15
///   thickness = stroke.thickness = 0.5
///   startX = max(armsLength,legsX) - headDiam/2 + thickness = 5.5
///   centerX = startX + headDiam/2 = 13.5
///   prefWidth = max(armsLength,legsX)*2 + 2*thickness = 27
///   prefHeight = headDiam + bodyLength + legsY + 2*thickness + deltaShadow + 1 = 60
///
/// Java ComponentRoseActor (head=true):
///   marginX1=3, marginX2=3
///   textWidth = pureTextWidth + 6
///   prefWidth = max(stickmanWidth(27), textWidth)
///   textMiddlePos = (prefWidth - textWidth) / 2
///   text rendered at: (textMiddlePos, stickmanHeight) relative to component origin
///   stickman at: (delta, 0) where delta = (prefWidth - stickmanWidth) / 2
fn draw_participant_actor(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x; // participant center x from layout

    // Java ActorStickMan constants
    let head_diam = 16.0_f64;
    let head_r = head_diam / 2.0;
    let body_length = 27.0_f64;
    let arms_y = 8.0_f64;
    let arms_length = 13.0_f64;
    let legs_x = 13.0_f64;
    let legs_y = 15.0_f64;
    let thickness = 0.5_f64;
    let stickman_width = arms_length.max(legs_x) * 2.0 + 2.0 * thickness; // 27
    let stickman_height = head_diam + body_length + legs_y + 2.0 * thickness + 1.0; // 60

    // Java: startX = max(arms,legs) - headDiam/2 + thickness = 5.5
    let start_x = arms_length.max(legs_x) - head_diam / 2.0 + thickness;
    // Java: centerX = startX + headDiam/2 = 13.5
    let center_x = start_x + head_diam / 2.0;

    // Text metrics
    let font_size = 14.0;
    let tl = font_metrics::text_width(name, "SansSerif", font_size, false, false);
    let margin_x1 = 3.0;
    let margin_x2 = 3.0;
    let text_width = tl + margin_x1 + margin_x2;
    let pref_width = stickman_width.max(text_width);
    let text_middle_pos = (pref_width - text_width) / 2.0;

    // Java: outMargin = 5, startingX = 0 for first participant
    // component_x = startingX + outMargin = p.x - pref_width/2 - outMargin + outMargin
    // Actually, p.x is the CENTER of the participant box from layout.
    // Java: getCenterX = startingX + prefWidth/2 + outMargin
    // So: component_x = p.x - pref_width/2
    // But Java adds outMargin to the drawing position.
    // Let's derive from known: Java ellipse cx = 24.8335 = component_x + startX + headR
    //   component_x + 5.5 + 8 = 24.8335 → component_x = 11.3335
    // But we need component_x from p.x. p.x = getCenterX = startingX + prefWidth/2 + outMargin
    // For Alice: prefWidth = 39.667, outMargin = 5
    //   getCenterX = 0 + 39.667/2 + 5 = 24.8335
    // So p.x = 24.8335. component_x = p.x - prefWidth/2 = 24.8335 - 19.8335 = 5.0
    // BUT Java drawU applies UTranslate(getMinX, y1) where getMinX = startingX + outMargin = 5
    // So component origin is at x=5.

    // For general case: component_x = p.x - pref_width / 2.0
    let component_x = cx - pref_width / 2.0;

    // 1. Text first
    // Java: textBlock.drawU at (textMiddlePos, stickmanHeight).
    // marginX1 is already baked into textWidth for centering calculation,
    // but does NOT add to the SVG x coordinate.
    let text_x = component_x + text_middle_pos;
    let text_y = y + stickman_height + font_metrics::ascent("SansSerif", font_size, false, false);
    sg.set_fill_color(text_color);
    sg.svg_text(
        name, text_x, text_y,
        Some("sans-serif"), font_size,
        None, None, None,
        tl,
        LengthAdjust::Spacing,
        None, 0, None,
    );

    // 2. Ellipse head
    // Java: head at (startX, thickness) relative to component + delta
    let delta = (pref_width - stickman_width) / 2.0;
    let head_cx = component_x + delta + start_x + head_r;
    let head_cy = y + thickness + head_r;
    let hcx = fmt_coord(head_cx);
    let hcy = fmt_coord(head_cy);
    let hr = fmt_coord(head_r);
    let mut el = String::new();
    write!(el,
        "<ellipse cx=\"{hcx}\" cy=\"{hcy}\" fill=\"#E2E2F0\" rx=\"{hr}\" ry=\"{hr}\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&el);

    // 3. Single path for body+arms+legs (Java: ActorStickMan, stroke 0.5)
    // Java path origin at (centerX, headDiam + thickness) relative to component + delta
    let path_ox = component_x + delta + center_x;
    let path_oy = y + head_diam + thickness;
    // Path segments (relative to path origin):
    //   M(0,0) L(0,bodyLength) M(-arms,armsY) L(arms,armsY) M(0,bodyLength) L(-legsX,bodyLength+legsY) M(0,bodyLength) L(legsX,bodyLength+legsY)
    let pcx = fmt_coord(path_ox);
    let bt = fmt_coord(path_oy);
    let bb = fmt_coord(path_oy + body_length);
    let la = fmt_coord(path_ox - arms_length);
    let ra = fmt_coord(path_ox + arms_length);
    let ay = fmt_coord(path_oy + arms_y);
    let ll = fmt_coord(path_ox - legs_x);
    let rl = fmt_coord(path_ox + legs_x);
    let lf = fmt_coord(path_oy + body_length + legs_y);
    let mut pa = String::new();
    write!(pa,
        "<path d=\"M{pcx},{bt} L{pcx},{bb} M{la},{ay} L{ra},{ay} M{pcx},{bb} L{ll},{lf} M{pcx},{bb} L{rl},{lf}\" fill=\"none\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&pa);
}

/// Actor tail: Java ComponentRoseActor(head=false).
/// Text is ABOVE, stickman BELOW. Same constants as head.
fn draw_participant_actor_tail(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    border: &str,
    text_color: &str,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;

    // Same constants as draw_participant_actor (head)
    let head_diam = 16.0_f64;
    let head_r = head_diam / 2.0;
    let body_length = 27.0_f64;
    let arms_y = 8.0_f64;
    let arms_length = 13.0_f64;
    let legs_x = 13.0_f64;
    let legs_y = 15.0_f64;
    let thickness = 0.5_f64;
    let stickman_width = arms_length.max(legs_x) * 2.0 + 2.0 * thickness;
    let stickman_height = head_diam + body_length + legs_y + 2.0 * thickness + 1.0;
    let start_x = arms_length.max(legs_x) - head_diam / 2.0 + thickness;
    let center_x = start_x + head_diam / 2.0;

    let font_size = 14.0;
    let tl = font_metrics::text_width(name, "SansSerif", font_size, false, false);
    let margin_x1 = 3.0;
    let margin_x2 = 3.0;
    let text_width = tl + margin_x1 + margin_x2;
    let pref_width = stickman_width.max(text_width);
    let text_middle_pos = (pref_width - text_width) / 2.0;
    let component_x = cx - pref_width / 2.0;

    // Java (head=false): text at (textMiddlePos, 0), stickman at (delta, textHeight)
    let text_height = font_metrics::line_height("SansSerif", font_size, false, false);
    let text_x = component_x + text_middle_pos;
    let text_y = y + font_metrics::ascent("SansSerif", font_size, false, false);

    // 1. Text first
    sg.set_fill_color(text_color);
    sg.svg_text(
        name, text_x, text_y,
        Some("sans-serif"), font_size,
        None, None, None,
        tl, LengthAdjust::Spacing,
        None, 0, None,
    );

    // 2. Stickman below text
    let delta = (pref_width - stickman_width) / 2.0;
    let stickman_y = y + text_height;

    // Ellipse head
    let head_cx = component_x + delta + start_x + head_r;
    let head_cy = stickman_y + thickness + head_r;
    let hcx = fmt_coord(head_cx);
    let hcy = fmt_coord(head_cy);
    let hr = fmt_coord(head_r);
    let mut el = String::new();
    write!(el,
        "<ellipse cx=\"{hcx}\" cy=\"{hcy}\" fill=\"#E2E2F0\" rx=\"{hr}\" ry=\"{hr}\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&el);

    // Body path
    let path_ox = component_x + delta + center_x;
    let path_oy = stickman_y + head_diam + thickness;
    let pcx = fmt_coord(path_ox);
    let bt = fmt_coord(path_oy);
    let bb = fmt_coord(path_oy + body_length);
    let la = fmt_coord(path_ox - arms_length);
    let ra = fmt_coord(path_ox + arms_length);
    let ay = fmt_coord(path_oy + arms_y);
    let ll = fmt_coord(path_ox - legs_x);
    let rl = fmt_coord(path_ox + legs_x);
    let lf = fmt_coord(path_oy + body_length + legs_y);
    let mut pa = String::new();
    write!(pa,
        "<path d=\"M{pcx},{bt} L{pcx},{bb} M{la},{ay} L{ra},{ay} M{pcx},{bb} L{ll},{lf} M{pcx},{bb} L{rl},{lf}\" fill=\"none\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&pa);
}

/// Boundary: vertical line + horizontal connector + ellipse, with text below.
/// Matches Java: Boundary.java (margin=4, radius=12, left=17) +
/// ComponentRoseBoundary.java (head=true: text at dimStickman.height, icon at delta).
/// Boundary: vertical line + horizontal connector + ellipse, with text below (head) or above (tail).
/// Matches Java: Boundary.java (margin=4, radius=12, left=17) +
/// ComponentRoseBoundary.java (head: text at dimStickman.height; tail: icon at textHeight).
fn draw_participant_boundary(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    head: bool,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;

    // Java Boundary.java constants
    let margin = 4.0_f64;
    let radius = 12.0_f64;
    let left = 17.0_f64;
    let icon_width = radius * 2.0 + left + 2.0 * margin; // 49
    let icon_height = radius * 2.0 + 2.0 * margin; // 32

    // Text metrics (Java: marginX1=3, marginX2=3)
    let font_size = 14.0;
    let margin_x1 = 3.0;
    let margin_x2 = 3.0;
    let tl = font_metrics::text_width(name, "SansSerif", font_size, false, false);
    let text_width = tl + margin_x1 + margin_x2;
    let pref_width = icon_width.max(text_width);
    let text_middle_pos = (pref_width - text_width) / 2.0;
    let component_x = cx - pref_width / 2.0;
    let delta = (pref_width - icon_width) / 2.0;
    let text_height = font_metrics::line_height("SansSerif", font_size, false, false);

    if head {
        // Head: text below icon
        // 1. Text at (textMiddlePos, icon_height)
        let text_x = component_x + text_middle_pos;
        let text_y = y + icon_height + font_metrics::ascent("SansSerif", font_size, false, false);
        sg.set_fill_color(text_color);
        sg.svg_text(
            name, text_x, text_y, Some("sans-serif"), font_size,
            None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
        );

        // 2. Path at (delta + margin, margin)
        let px = component_x + delta + margin;
        let py = y + margin;
        draw_boundary_icon(sg, px, py, radius, left, bg, border);
    } else {
        // Tail: text above icon
        // 1. Text at (textMiddlePos, 0)
        let text_x = component_x + text_middle_pos;
        let text_y = y + font_metrics::ascent("SansSerif", font_size, false, false);
        sg.set_fill_color(text_color);
        sg.svg_text(
            name, text_x, text_y, Some("sans-serif"), font_size,
            None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
        );

        // 2. Icon at (delta, textHeight)
        let px = component_x + delta + margin;
        let py = y + text_height + margin;
        draw_boundary_icon(sg, px, py, radius, left, bg, border);
    }
}

/// Draw the boundary icon (path + ellipse) at the given origin.
fn draw_boundary_icon(
    sg: &mut SvgGraphic,
    px: f64, py: f64,
    radius: f64, left: f64,
    bg: &str, border: &str,
) {
    let px_s = fmt_coord(px);
    let py_top = fmt_coord(py);
    let py_bot = fmt_coord(py + radius * 2.0);
    let py_mid = fmt_coord(py + radius);
    let px_right = fmt_coord(px + left);
    let mut pa = String::new();
    write!(pa,
        "<path d=\"M{px_s},{py_top} L{px_s},{py_bot} M{px_s},{py_mid} L{px_right},{py_mid}\" fill=\"none\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&pa);

    let ecx = px + left + radius;
    let ecy = py + radius;
    let mut el = String::new();
    write!(el,
        "<ellipse cx=\"{}\" cy=\"{}\" fill=\"{bg}\" rx=\"{r}\" ry=\"{r}\" style=\"stroke:{border};stroke-width:0.5;\"/>",
        fmt_coord(ecx), fmt_coord(ecy), r = fmt_coord(radius)
    ).unwrap();
    sg.push_raw(&el);
}

/// Control: ellipse + small arrow polygon. Matches Java Control.java.
fn draw_participant_control(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    head: bool,
) {
    draw_iconic_participant(sg, p, y, display_name, bg, border, text_color, head,
        |sg, px, py, radius, bg, border| {
            // Ellipse
            let ecx = px + radius;
            let ecy = py + radius;
            let mut el = String::new();
            write!(el,
                "<ellipse cx=\"{}\" cy=\"{}\" fill=\"{bg}\" rx=\"{r}\" ry=\"{r}\" style=\"stroke:{border};stroke-width:0.5;\"/>",
                fmt_coord(ecx), fmt_coord(ecy), r = fmt_coord(radius)
            ).unwrap();
            sg.push_raw(&el);

            // Arrow polygon (Java: Control.java xWing=6, yAperture=5, xContact=4)
            let x_wing = 6.0_f64;
            let y_aperture = 5.0_f64;
            let x_contact = 4.0_f64;
            let ax = px + radius - x_contact;
            let ay = py;
            let pts = format!("{},{},{},{},{},{},{},{},{},{}",
                fmt_coord(ax), fmt_coord(ay),
                fmt_coord(ax + x_wing), fmt_coord(ay - y_aperture),
                fmt_coord(ax + x_contact), fmt_coord(ay),
                fmt_coord(ax + x_wing), fmt_coord(ay + y_aperture),
                fmt_coord(ax), fmt_coord(ay),
            );
            let mut pg = String::new();
            write!(pg,
                "<polygon fill=\"{border}\" points=\"{pts}\" style=\"stroke:{border};stroke-width:1;\"/>"
            ).unwrap();
            sg.push_raw(&pg);
        },
    );
}

/// Entity: ellipse + horizontal underline. Matches Java EntityDomain.java.
fn draw_participant_entity(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    head: bool,
) {
    draw_iconic_participant(sg, p, y, display_name, bg, border, text_color, head,
        |sg, px, py, radius, bg, border| {
            // Ellipse
            let ecx = px + radius;
            let ecy = py + radius;
            let mut el = String::new();
            write!(el,
                "<ellipse cx=\"{}\" cy=\"{}\" fill=\"{bg}\" rx=\"{r}\" ry=\"{r}\" style=\"stroke:{border};stroke-width:0.5;\"/>",
                fmt_coord(ecx), fmt_coord(ecy), r = fmt_coord(radius)
            ).unwrap();
            sg.push_raw(&el);

            // Underline (Java: suppY=2, hline at y + 2*radius + suppY)
            let supp_y = 2.0;
            let line_y = py + 2.0 * radius + supp_y;
            let mut ln = String::new();
            write!(ln,
                "<line style=\"stroke:{border};stroke-width:0.5;\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
                fmt_coord(px), fmt_coord(px + 2.0 * radius),
                fmt_coord(line_y), fmt_coord(line_y)
            ).unwrap();
            sg.push_raw(&ln);
        },
    );
}

/// Generic iconic participant rendering (boundary/control/entity pattern).
/// Java: ComponentRose{Boundary,Control,Entity} all share the same layout:
/// head=true: text below icon, head=false (tail): text above icon.
fn draw_iconic_participant(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    head: bool,
    draw_icon: impl FnOnce(&mut SvgGraphic, f64, f64, f64, &str, &str),
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let margin = 4.0;
    let radius = 12.0;
    let icon_width: f64 = radius * 2.0 + 2.0 * margin; // 32
    let icon_height: f64 = radius * 2.0 + 2.0 * margin; // 32

    let font_size = 14.0_f64;
    let margin_x1 = 3.0_f64;
    let margin_x2 = 3.0_f64;
    let tl = font_metrics::text_width(name, "SansSerif", font_size, false, false);
    let text_width = tl + margin_x1 + margin_x2;
    let pref_width = icon_width.max(text_width);
    let text_middle_pos = (pref_width - text_width) / 2.0;
    let component_x = cx - pref_width / 2.0;
    let delta = (pref_width - icon_width) / 2.0;
    let text_height = font_metrics::line_height("SansSerif", font_size, false, false);

    if head {
        // Text at (textMiddlePos, icon_height)
        let text_x = component_x + text_middle_pos;
        let text_y = y + icon_height + font_metrics::ascent("SansSerif", font_size, false, false);
        sg.set_fill_color(text_color);
        sg.svg_text(
            name, text_x, text_y, Some("sans-serif"), font_size,
            None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
        );
        // Icon at (delta + margin, margin)
        draw_icon(sg, component_x + delta + margin, y + margin, radius, bg, border);
    } else {
        // Text at (textMiddlePos, 0)
        let text_x = component_x + text_middle_pos;
        let text_y = y + font_metrics::ascent("SansSerif", font_size, false, false);
        sg.set_fill_color(text_color);
        sg.svg_text(
            name, text_x, text_y, Some("sans-serif"), font_size,
            None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
        );
        // Icon at (delta + margin, textHeight + margin)
        draw_icon(sg, component_x + delta + margin, y + text_height + margin, radius, bg, border);
    }
}

/// Database: cylinder shape using cubic bezier paths.
/// Matches Java: USymbolDatabase.drawDatabase + ComponentRoseDatabase.
/// Stickman = asSmall(empty(16,17)) + Margin(10,10,24,5) → dim=(36, 46).
fn draw_participant_database(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    head: bool,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;

    let icon_width = 36.0_f64; // DATABASE_ICON_WIDTH
    let icon_height = 46.0_f64; // dimStickman.height
    let curve_h = 10.0_f64; // Java drawDatabase hardcoded curve constant

    let font_size = 14.0_f64;
    let margin_x1 = 3.0_f64;
    let margin_x2 = 3.0_f64;
    let tl = font_metrics::text_width(name, "SansSerif", font_size, false, false);
    let text_width = tl + margin_x1 + margin_x2;
    let pref_width = icon_width.max(text_width);
    let text_middle_pos = (pref_width - text_width) / 2.0;
    let component_x = cx - pref_width / 2.0;
    let delta = (pref_width - icon_width) / 2.0;
    let text_height = font_metrics::line_height("SansSerif", font_size, false, false);

    let (text_x, text_y, cyl_x, cyl_y);
    if head {
        text_x = component_x + text_middle_pos;
        text_y = y + icon_height + font_metrics::ascent("SansSerif", font_size, false, false);
        cyl_x = component_x + delta;
        cyl_y = y;
    } else {
        text_x = component_x + text_middle_pos;
        text_y = y + font_metrics::ascent("SansSerif", font_size, false, false);
        cyl_x = component_x + delta;
        cyl_y = y + text_height;
    }

    // 1. Text first
    sg.set_fill_color(text_color);
    sg.svg_text(
        name, text_x, text_y, Some("sans-serif"), font_size,
        None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
    );

    // 2. Cylinder body path (Java: USymbolDatabase.drawDatabase)
    draw_database_cylinder(sg, cyl_x, cyl_y, icon_width, icon_height, curve_h, bg, border);
}

/// Draw the database cylinder using cubic bezier paths matching Java USymbolDatabase.
fn draw_database_cylinder(
    sg: &mut SvgGraphic,
    x: f64, y: f64, w: f64, h: f64, ch: f64,
    bg: &str, border: &str,
) {
    let mid = w / 2.0;
    // Path 1: cylinder body
    // M(0,ch) C(0,0, mid,0, mid,0) C(mid,0, w,0, w,ch) L(w,h-ch) C(w,h, mid,h, mid,h) C(mid,h, 0,h, 0,h-ch) L(0,ch)
    let x0 = fmt_coord(x);
    let xm = fmt_coord(x + mid);
    let xw = fmt_coord(x + w);
    let yt = fmt_coord(y);       // 0
    let yc = fmt_coord(y + ch);  // ch (top of body)
    let yb = fmt_coord(y + h - ch); // h-ch (bottom of body)
    let yh = fmt_coord(y + h);   // h (bottom control)
    let mut body = String::new();
    write!(body,
        "<path d=\"M{x0},{yc} C{x0},{yt} {xm},{yt} {xm},{yt} C{xm},{yt} {xw},{yt} {xw},{yc} L{xw},{yb} C{xw},{yh} {xm},{yh} {xm},{yh} C{xm},{yh} {x0},{yh} {x0},{yb} L{x0},{yc}\" fill=\"{bg}\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&body);

    // Path 2: inner top curve (closing/front ellipse)
    let yc2 = fmt_coord(y + ch * 2.0); // 2*ch
    let mut top = String::new();
    write!(top,
        "<path d=\"M{x0},{yc} C{x0},{yc2} {xm},{yc2} {xm},{yc2} C{xm},{yc2} {xw},{yc2} {xw},{yc}\" fill=\"none\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&top);
}

/// Collections: two stacked rectangles + text inside main rect.
/// Matches Java: ComponentRoseCollections — shadow rect offset by COLLECTIONS_DELTA=4.
/// Same for both head and tail (text always inside main rect).
fn draw_participant_collections(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    _head: bool,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let delta = 4.0_f64; // COLLECTIONS_DELTA

    let font_size = 14.0_f64;
    let tl = font_metrics::text_width(name, "SansSerif", font_size, false, false);
    let rect_w = tl + 2.0 * 7.0; // text + padding (like default participant)
    let rect_h = p.box_height - delta; // base participant height (30.2969)
    let pref_width = rect_w + delta;
    let component_x = cx - pref_width / 2.0;

    // Shadow rect at (component_x + delta, y)
    let mut tmp = String::new();
    write!(tmp,
        r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(rect_h), fmt_coord(rect_w), fmt_coord(component_x + delta), fmt_coord(y),
    ).unwrap();
    sg.push_raw(&tmp);

    // Main rect at (component_x, y + delta)
    let main_y = y + delta;
    let mut tmp = String::new();
    write!(tmp,
        r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(rect_h), fmt_coord(rect_w), fmt_coord(component_x), fmt_coord(main_y),
    ).unwrap();
    sg.push_raw(&tmp);

    // Text inside main rect
    let text_x = component_x + 7.0;
    let text_y = main_y + 7.0 + font_metrics::ascent("SansSerif", font_size, false, false);
    sg.set_fill_color(text_color);
    sg.svg_text(
        name, text_x, text_y, Some("sans-serif"), font_size,
        None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
    );
}

/// Queue: rounded-right rectangle with text inside, using cubic-bezier curves.
/// Matches Java: USymbolQueue.drawQueue (dx=5, margin 5,15,5,5).
/// Text is inside the shape (no head/tail text separation).
fn draw_participant_queue(
    sg: &mut SvgGraphic,
    p: &ParticipantLayout,
    y: f64,
    display_name: Option<&str>,
    bg: &str,
    border: &str,
    text_color: &str,
    _head: bool,
) {
    let name = display_name.unwrap_or(&p.name);
    let cx = p.x;
    let dx = 5.0_f64; // Java USymbolQueue.dx

    let font_size = 14.0_f64;
    let tl = font_metrics::text_width(name, "SansSerif", font_size, false, false);
    let text_height = font_metrics::line_height("SansSerif", font_size, false, false);

    // Queue margin: x1=5, x2=15, y1=5, y2=5
    let margin_x1 = 5.0_f64;
    let margin_x2 = 15.0_f64;
    let margin_y1 = 5.0_f64;
    let w = tl + margin_x1 + margin_x2; // shape width
    let h = text_height + 10.0; // shape height (margin_y1 + margin_y2)

    let pref_width = w;
    let component_x = cx - pref_width / 2.0;
    let mid_y = h / 2.0;

    // Draw body path
    let x0 = component_x;
    let y0 = y;
    let x0s = fmt_coord(x0 + dx);
    let x1s = fmt_coord(x0 + w - dx);
    let xws = fmt_coord(x0 + w);
    let x0f = fmt_coord(x0);
    let y0s = fmt_coord(y0);
    let yms = fmt_coord(y0 + mid_y);
    let yhs = fmt_coord(y0 + h);
    let mut body = String::new();
    write!(body,
        "<path d=\"M{x0s},{y0s} L{x1s},{y0s} C{xws},{y0s} {xws},{yms} {xws},{yms} C{xws},{yms} {xws},{yhs} {x1s},{yhs} L{x0s},{yhs} C{x0f},{yhs} {x0f},{yms} {x0f},{yms} C{x0f},{yms} {x0f},{y0s} {x0s},{y0s}\" fill=\"{bg}\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&body);

    // Inner right curve (closing path)
    let x2s = fmt_coord(x0 + w - dx * 2.0);
    let mut closing = String::new();
    write!(closing,
        "<path d=\"M{x1s},{y0s} C{x2s},{y0s} {x2s},{yms} {x2s},{yms} C{x2s},{yhs} {x1s},{yhs} {x1s},{yhs}\" fill=\"none\" style=\"stroke:{border};stroke-width:0.5;\"/>"
    ).unwrap();
    sg.push_raw(&closing);

    // Text inside shape at (margin_x1, vertically centered)
    let text_x = x0 + margin_x1;
    let text_y = y0 + (h - text_height) / 2.0 + font_metrics::ascent("SansSerif", font_size, false, false);
    sg.set_fill_color(text_color);
    sg.svg_text(
        name, text_x, text_y, Some("sans-serif"), font_size,
        None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
    );
}

/// Render a single text line word-by-word (Java beta5 behavior when maxmessagesize is set).
/// Each word becomes a separate `<text>` element, with `&#160;` elements between words.
/// `metrics_font` is the internal font name for width calculations (e.g. "SansSerif").
/// `svg_font` is the SVG font-family attribute value (e.g. "sans-serif").
fn render_word_by_word(
    sg: &mut SvgGraphic,
    line: &str,
    x: f64,
    y: f64,
    metrics_font: &str,
    svg_font: &str,
    font_size: f64,
) {
    let words: Vec<&str> = line.split(' ').collect();
    let mut cur_x = x;
    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            // Render &#160; (non-breaking space) between words
            let space_w = font_metrics::text_width("\u{00a0}", metrics_font, font_size, false, false);
            sg.set_fill_color(TEXT_COLOR);
            sg.svg_text(
                "\u{00a0}",
                cur_x,
                y,
                Some(svg_font),
                font_size,
                None,
                None,
                None,
                space_w,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
            cur_x += space_w;
        }
        if word.is_empty() {
            continue;
        }
        let word_w = font_metrics::text_width(word, metrics_font, font_size, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            word,
            cur_x,
            y,
            Some(svg_font),
            font_size,
            None,
            None,
            None,
            word_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        cur_x += word_w;
    }
}

// ── Messages ────────────────────────────────────────────────────────

fn draw_message(
    sg: &mut SvgGraphic,
    msg: &MessageLayout,
    arrow_color: &str,
    arrow_thickness: f64,
    msg_font_family: &str,
    msg_svg_family: &str,
    msg_font_size: f64,
    from_idx: usize,
    to_idx: usize,
    msg_idx: usize,
    source_line: Option<usize>,
    word_by_word: bool,
) {
    let src_line_attr = source_line
        .map(|sl| format!(r#" data-source-line="{sl}""#))
        .unwrap_or_default();
    sg.push_raw(&format!(
        r#"<g class="message" data-entity-1="part{}" data-entity-2="part{}"{} id="msg{}">"#,
        from_idx, to_idx, src_line_attr, msg_idx,
    ));

    let sw = arrow_thickness as u32;

    // Java constants for circle decorations
    const DIAM_CIRCLE: f64 = 8.0;
    const THIN_CIRCLE: f64 = 1.5;

    // Draw circle decorations FIRST (before arrowhead and line)
    // Java: ComponentRoseArrow.drawDressing1/drawDressing2
    if msg.circle_from {
        let cx = msg.from_x - 0.5;
        let cy = msg.y - 0.75;
        sg.push_raw(&format!(
            r##"<ellipse cx="{}" cy="{}" fill="#000000" rx="{}" ry="{}" style="stroke:{};stroke-width:{};"/>"##,
            fmt_coord(cx), fmt_coord(cy),
            fmt_coord(DIAM_CIRCLE / 2.0), fmt_coord(DIAM_CIRCLE / 2.0),
            arrow_color, fmt_coord(THIN_CIRCLE),
        ));
    }
    if msg.circle_to {
        let cx = msg.to_x - 0.5;
        let cy = msg.y - 0.75;
        sg.push_raw(&format!(
            r##"<ellipse cx="{}" cy="{}" fill="#000000" rx="{}" ry="{}" style="stroke:{};stroke-width:{};"/>"##,
            fmt_coord(cx), fmt_coord(cy),
            fmt_coord(DIAM_CIRCLE / 2.0), fmt_coord(DIAM_CIRCLE / 2.0),
            arrow_color, fmt_coord(THIN_CIRCLE),
        ));
    }

    // Determine arrow tip position and line endpoints
    // Java insets the arrow tip 2px from the participant center
    let (tip_x, line_x1, _line_x2) = if msg.is_left {
        // Right-to-left: arrow points left, tip 1px inset from target center
        (msg.to_x + 1.0, msg.from_x - 1.0, msg.to_x)
    } else {
        // Left-to-right: arrow points right, tip 2px inset from target center
        (msg.to_x - 2.0, msg.from_x, msg.to_x)
    };

    // Draw inline polygon arrowhead
    if msg.has_open_head {
        // Open arrowhead: lines forming a V (or half-V for half-arrows)
        let (ax1, ax2) = if msg.is_left {
            (tip_x + 10.0, tip_x + 10.0)
        } else {
            (tip_x - 10.0, tip_x - 10.0)
        };
        let mut tmp = String::new();
        // Top line of V (skip for HalfBottom)
        if !matches!(msg.arrow_head, SeqArrowHead::HalfBottom) {
            write!(
                tmp,
                r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
                color = arrow_color,
                ax = fmt_coord(ax1),
                tx = fmt_coord(tip_x),
                y1 = fmt_coord(msg.y - 4.0),
                y = fmt_coord(msg.y),
            )
            .unwrap();
        }
        // Bottom line of V (skip for HalfTop)
        if !matches!(msg.arrow_head, SeqArrowHead::HalfTop) {
            write!(
                tmp,
                r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
                color = arrow_color,
                ax = fmt_coord(ax2),
                tx = fmt_coord(tip_x),
                y1 = fmt_coord(msg.y + 4.0),
                y = fmt_coord(msg.y),
            )
            .unwrap();
        }
        sg.push_raw(&tmp);
    } else {
        // Filled arrowhead polygon: 4-point diamond with inner point 6px from tip
        let (p1x, p2x, p3x, p4x) = if msg.is_left {
            (tip_x + 10.0, tip_x, tip_x + 10.0, tip_x + 6.0)
        } else {
            (tip_x - 10.0, tip_x, tip_x - 10.0, tip_x - 6.0)
        };
        let mut tmp = String::new();
        write!(
            tmp,
            r#"<polygon fill="{color}" points="{p1x},{p1y},{p2x},{p2y},{p3x},{p3y},{p4x},{p4y}" style="stroke:{color};stroke-width:1;"/>"#,
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
        sg.push_raw(&tmp);
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
    // For left-pointing arrows, swap x1/x2 so smaller x comes first
    let (lx1, lx2) = if msg.is_left {
        (adjusted_x2, line_x1)
    } else {
        (line_x1, adjusted_x2)
    };
    let mut tmp = String::new();
    write!(
        tmp,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x1}" x2="{x2}" y1="{y}" y2="{y}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x1 = fmt_coord(lx1),
        x2 = fmt_coord(lx2),
        y = fmt_coord(msg.y),
    )
    .unwrap();
    sg.push_raw(&tmp);

    // Label text above the line — each line as a separate <text> element
    let has_text = !msg.text.is_empty() || msg.autonumber.is_some();
    if has_text {
        let base_text_x = if msg.is_left {
            // Left arrow: text starts after arrowhead polygon (tip + polygon_width + gap)
            tip_x + 16.0
        } else {
            msg.from_x + 7.0
        };

        // If autonumber, compute the offset for message text (number is bold)
        let text_x = if let Some(ref num_str) = msg.autonumber {
            let num_w = font_metrics::text_width(num_str, msg_font_family, msg_font_size, true, false);
            base_text_x + num_w + 4.0
        } else {
            base_text_x
        };

        let msg_line_spacing =
            font_metrics::line_height(msg_font_family, msg_font_size, false, false);
        let num_lines = msg.text_lines.len().max(1);
        // When text has <sub>, the subscript extends below the baseline, adding
        // extra height below the text block. This shifts the text baseline up
        // relative to the arrow position. Superscript extends above but does
        // NOT shift the text baseline.
        let sub_extra = msg.text_lines.first().map(|line| {
            crate::render::svg_richtext::creole_sub_extra_height(line, msg_font_family, msg_font_size)
        }).unwrap_or(0.0);
        let first_text_y = msg.y
            - (font_metrics::descent(msg_font_family, msg_font_size, false, false) + 2.0)
            - (num_lines as f64 - 1.0) * msg_line_spacing
            - sub_extra;

        // Draw autonumber as separate bold text element
        if let Some(ref num_str) = msg.autonumber {
            let num_tl =
                font_metrics::text_width(num_str, msg_font_family, msg_font_size, true, false);
            sg.set_fill_color(TEXT_COLOR);
            sg.svg_text(
                num_str,
                base_text_x,
                first_text_y,
                Some(msg_svg_family),
                msg_font_size,
                Some("700"),
                None,
                None,
                num_tl,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
        }

        // Draw message text lines
        for (i, line) in msg.text_lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let line_y = first_text_y + i as f64 * msg_line_spacing;
            if word_by_word {
                render_word_by_word(sg, line, text_x, line_y, msg_font_family, msg_svg_family, msg_font_size);
            } else {
                let mut tmp = String::new();
                render_creole_text(
                    &mut tmp,
                    line,
                    text_x,
                    line_y,
                    msg_line_spacing,
                    TEXT_COLOR,
                    None,
                    &format!(r#"font-size="{msg_font_size}""#),
                );
                sg.push_raw(&tmp);
            }
        }
    }

    sg.push_raw("</g>");
}

fn draw_self_message(
    sg: &mut SvgGraphic,
    msg: &MessageLayout,
    arrow_color: &str,
    arrow_thickness: f64,
    msg_font_family: &str,
    msg_font_size: f64,
    from_idx: usize,
    msg_idx: usize,
    word_by_word: bool,
) {
    let sw = arrow_thickness as u32;
    let from_x = msg.from_x;
    let to_x = msg.to_x;
    let return_x = msg.self_return_x;
    let y = msg.y;
    let loop_height = 13.0;

    let src_line_attr = msg.source_line
        .map(|sl| format!(r#" data-source-line="{sl}""#))
        .unwrap_or_default();
    sg.push_raw(&format!(
        r#"<g class="message" data-entity-1="part{}" data-entity-2="part{}"{} id="msg{}">"#,
        from_idx, from_idx, src_line_attr, msg_idx,
    ));

    // Java constants for circle decorations
    const DIAM_CIRCLE: f64 = 8.0;
    const THIN_CIRCLE: f64 = 1.5;

    // Draw circle decorations for self-messages
    // circle_from → outgoing line, circle_to → return line
    // For left self-message: circle at the right side (from_x - 0.5)
    // For right self-message: circle at the right side (from_x + 0.5)
    if msg.circle_from {
        let cx = if msg.is_left { from_x - 0.5 } else { from_x + 0.5 };
        let cy = y - 0.75;
        sg.push_raw(&format!(
            r##"<ellipse cx="{}" cy="{}" fill="#000000" rx="{}" ry="{}" style="stroke:{};stroke-width:{};"/>"##,
            fmt_coord(cx), fmt_coord(cy),
            fmt_coord(DIAM_CIRCLE / 2.0), fmt_coord(DIAM_CIRCLE / 2.0),
            arrow_color, fmt_coord(THIN_CIRCLE),
        ));
    }
    if msg.circle_to {
        let cx = if msg.is_left { from_x - 0.5 } else { from_x + 0.5 };
        let cy = (y + loop_height) - 0.75;
        sg.push_raw(&format!(
            r##"<ellipse cx="{}" cy="{}" fill="#000000" rx="{}" ry="{}" style="stroke:{};stroke-width:{};"/>"##,
            fmt_coord(cx), fmt_coord(cy),
            fmt_coord(DIAM_CIRCLE / 2.0), fmt_coord(DIAM_CIRCLE / 2.0),
            arrow_color, fmt_coord(THIN_CIRCLE),
        ));
    }

    let dash_style = if msg.is_dashed {
        "stroke-dasharray:2,2;"
    } else {
        ""
    };

    // 3-line self-message: horizontal out, vertical down, horizontal return
    let mut tmp = String::new();

    // For right self-messages: right→down→left (arrowhead points left)
    // For left self-messages: left→down→right (arrowhead points right)
    // `from_x` is the start point (at lifeline/activation edge)
    // `to_x` is the far end of the horizontal
    // `return_x` is the return line endpoint (at lifeline/activation edge)

    // Line 1: outgoing horizontal
    // Java drawLeftSide: x1 starts at 0, incremented by 1 before drawing.
    // So the right end is prefTextWidth - 1 (absolute: pos2 - 1).
    // Java drawRightSide: x1 starts at 0, so left end is 0 (absolute: pos2).
    let (line1_x1, line1_x2) = if msg.is_left {
        (to_x, from_x - 1.0)
    } else {
        (from_x, to_x)
    };
    write!(
        tmp,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x1}" x2="{x2}" y1="{y1}" y2="{y1}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x1 = fmt_coord(line1_x1),
        x2 = fmt_coord(line1_x2),
        y1 = fmt_coord(y),
    )
    .unwrap();

    // Line 2: vertical down
    write!(
        tmp,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x}" x2="{x}" y1="{y1}" y2="{y2}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x = fmt_coord(to_x),
        y1 = fmt_coord(y),
        y2 = fmt_coord(y + loop_height),
    )
    .unwrap();

    // Line 3: return horizontal
    // Java drawLeftSide: extraline=1 for NORMAL arrowhead, so return right end
    // is prefTextWidth - x2 - extraline from origin = pos2 - 2 in absolute.
    let (line3_x1, line3_x2) = if msg.is_left {
        let extraline = if msg.has_open_head { 0.0 } else { 1.0 };
        (to_x, return_x - extraline)
    } else {
        (return_x, to_x)
    };
    write!(
        tmp,
        r#"<line style="stroke:{color};stroke-width:{sw};{dash}" x1="{x1}" x2="{x2}" y1="{y}" y2="{y}"/>"#,
        color = arrow_color,
        dash = dash_style,
        x1 = fmt_coord(line3_x1),
        x2 = fmt_coord(line3_x2),
        y = fmt_coord(y + loop_height),
    )
    .unwrap();

    // Arrowhead at return
    let ret_y = y + loop_height;
    if msg.is_left {
        // Left self-message: arrowhead points RIGHT at return
        // Java: after extraline+x2 adjustments, tip_x = pos2 - 2 for NORMAL head
        let tip_x = return_x - if msg.has_open_head { 0.0 } else { 1.0 };
        if msg.has_open_head {
            // Top line of V (skip for HalfBottom)
            if !matches!(msg.arrow_head, SeqArrowHead::HalfBottom) {
                write!(
                    tmp,
                    r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
                    color = arrow_color,
                    ax = fmt_coord(tip_x - 10.0),
                    tx = fmt_coord(tip_x),
                    y1 = fmt_coord(ret_y - 4.0),
                    y = fmt_coord(ret_y),
                )
                .unwrap();
            }
            // Bottom line of V (skip for HalfTop)
            if !matches!(msg.arrow_head, SeqArrowHead::HalfTop) {
                write!(
                    tmp,
                    r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
                    color = arrow_color,
                    ax = fmt_coord(tip_x - 10.0),
                    tx = fmt_coord(tip_x),
                    y1 = fmt_coord(ret_y + 4.0),
                    y = fmt_coord(ret_y),
                )
                .unwrap();
            }
        } else {
            write!(
                tmp,
                r#"<polygon fill="{color}" points="{p1x},{p1y},{p2x},{p2y},{p3x},{p3y},{p4x},{p4y}" style="stroke:{color};stroke-width:1;"/>"#,
                color = arrow_color,
                p1x = fmt_coord(tip_x - 10.0),
                p1y = fmt_coord(ret_y - 4.0),
                p2x = fmt_coord(tip_x),
                p2y = fmt_coord(ret_y),
                p3x = fmt_coord(tip_x - 10.0),
                p3y = fmt_coord(ret_y + 4.0),
                p4x = fmt_coord(tip_x - 6.0),
                p4y = fmt_coord(ret_y),
            )
            .unwrap();
        }
    } else {
        // Right self-message: arrowhead points LEFT at return
        let tip_x = return_x;
        if msg.has_open_head {
            // Top line of V (skip for HalfBottom)
            if !matches!(msg.arrow_head, SeqArrowHead::HalfBottom) {
                write!(
                    tmp,
                    r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
                    color = arrow_color,
                    ax = fmt_coord(tip_x + 10.0),
                    tx = fmt_coord(tip_x),
                    y1 = fmt_coord(ret_y - 4.0),
                    y = fmt_coord(ret_y),
                )
                .unwrap();
            }
            // Bottom line of V (skip for HalfTop)
            if !matches!(msg.arrow_head, SeqArrowHead::HalfTop) {
                write!(
                    tmp,
                    r#"<line style="stroke:{color};stroke-width:{sw};" x1="{ax}" x2="{tx}" y1="{y1}" y2="{y}"/>"#,
                    color = arrow_color,
                    ax = fmt_coord(tip_x + 10.0),
                    tx = fmt_coord(tip_x),
                    y1 = fmt_coord(ret_y + 4.0),
                    y = fmt_coord(ret_y),
                )
                .unwrap();
            }
        } else {
            write!(
                tmp,
                r#"<polygon fill="{color}" points="{p1x},{p1y},{p2x},{p2y},{p3x},{p3y},{p4x},{p4y}" style="stroke:{color};stroke-width:1;"/>"#,
                color = arrow_color,
                p1x = fmt_coord(tip_x + 10.0),
                p1y = fmt_coord(ret_y - 4.0),
                p2x = fmt_coord(tip_x),
                p2y = fmt_coord(ret_y),
                p3x = fmt_coord(tip_x + 10.0),
                p3y = fmt_coord(ret_y + 4.0),
                p4x = fmt_coord(tip_x + 6.0),
                p4y = fmt_coord(ret_y),
            )
            .unwrap();
        }
    }
    sg.push_raw(&tmp);

    // Label text above the first horizontal line — each line as separate <text>
    if !msg.text.is_empty() {
        let text_x = if msg.is_left {
            // Left self-message: text starts at from_x - preferredWidth + marginX1(7).
            // For activated participants, from_x is the activation bar left edge.
            let text_w = msg.text_lines
                .iter()
                .map(|line| crate::font_metrics::text_width(
                    line, msg_font_family, msg_font_size, false, false,
                ))
                .fold(0.0_f64, f64::max);
            let preferred = f64::max(text_w + 14.0, crate::skin::rose::SELF_ARROW_WIDTH + 5.0);
            from_x - preferred + 7.0
        } else {
            return_x + 6.0
        };
        let msg_line_spacing =
            font_metrics::line_height(msg_font_family, msg_font_size, false, false);
        let num_lines = msg.text_lines.len();
        let sub_extra = msg.text_lines.first().map(|line| {
            crate::render::svg_richtext::creole_sub_extra_height(line, msg_font_family, msg_font_size)
        }).unwrap_or(0.0);
        let first_text_y = y
            - (font_metrics::descent(msg_font_family, msg_font_size, false, false) + 2.0)
            - (num_lines as f64 - 1.0) * msg_line_spacing
            - sub_extra;
        for (i, line) in msg.text_lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let line_y = first_text_y + i as f64 * msg_line_spacing;
            if word_by_word {
                let svg_family = svg_font_family_attr(msg_font_family);
                render_word_by_word(sg, line, text_x, line_y, msg_font_family, svg_family, msg_font_size);
            } else {
                let mut tmp = String::new();
                render_creole_text(
                    &mut tmp,
                    line,
                    text_x,
                    line_y,
                    msg_line_spacing,
                    TEXT_COLOR,
                    None,
                    &format!(r#"font-size="{msg_font_size}""#),
                );
                sg.push_raw(&tmp);
            }
        }
    }

    sg.push_raw("</g>");
}

// ── Activation bars ─────────────────────────────────────────────────

fn draw_activation(sg: &mut SvgGraphic, act: &ActivationLayout, title: &str) {
    let width = 10.0;
    let height = act.y_end - act.y_start;

    let mut tmp = String::new();
    write!(
        tmp,
        r#"<g><title>{title}</title><rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/></g>"#,
        fmt_coord(height),
        fmt_coord(width),
        fmt_coord(act.x),
        fmt_coord(act.y_start),
        title = xml_escape(title),
        bg = ACTIVATION_BG,
        border = BORDER_COLOR,
    )
    .unwrap();
    sg.push_raw(&tmp);
}

// ── Destroy marker ──────────────────────────────────────────────────

fn draw_destroy(sg: &mut SvgGraphic, d: &DestroyLayout) {
    let size = 9.0;
    // First diagonal: top-left to bottom-right
    sg.set_stroke_color(Some(DESTROY_COLOR));
    sg.set_stroke_width(2.0, None);
    sg.svg_line(d.x - size, d.y - size, d.x + size, d.y + size, 0.0);

    // Second diagonal: bottom-left to top-right (matching Java PlantUML order)
    sg.svg_line(d.x - size, d.y + size, d.x + size, d.y - size, 0.0);
}

// ── Notes ───────────────────────────────────────────────────────────

fn draw_note(sg: &mut SvgGraphic, note: &NoteLayout) {
    let fold = 10.0; // folded corner size
    // Java truncates note x to int in NoteBox.getStartingX():
    //   xStart = (int)(segment.getSegment().getPos2())
    // Java truncates polygon width to int in ComponentRoseNote.drawInternalU():
    //   int x2 = (int) getTextWidth(stringBounder)
    let x = note.x.trunc();
    let y = note.y;
    let w = note.width.trunc();
    let h = note.height;

    // Body: hexagonal path with folded top-right corner (Java: Opale.getPolygonNormal)
    {
        let x0 = fmt_coord(x);
        let y0 = fmt_coord(y);
        let x1 = fmt_coord(x + w);
        let y1 = fmt_coord(y + h);
        let xf = fmt_coord(x + w - fold);
        let yf = fmt_coord(y + fold);
        sg.push_raw(&format!(
            "<path d=\"M{x0},{y0} L{x0},{y1} L{x1},{y1} L{x1},{yf} L{xf},{y0} L{x0},{y0}\" fill=\"{bg}\" style=\"stroke:{border};stroke-width:0.5;\"/>",
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ));
    }

    // Fold corner triangle (Java: Opale.getCorner)
    {
        let cx_s = fmt_coord(x + w - fold);
        let cy_s = fmt_coord(y);
        let cy2 = fmt_coord(y + fold);
        let cx2 = fmt_coord(x + w);
        sg.push_raw(&format!(
            "<path d=\"M{cx_s},{cy_s} L{cx_s},{cy2} L{cx2},{cy2} L{cx_s},{cy_s}\" fill=\"{bg}\" style=\"stroke:{border};stroke-width:0.5;\"/>",
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ));
    }

    let text_x = x + 6.0;
    // Java: ComponentRoseNote applies UTranslate(marginX1=6, marginY=5),
    // then TextBlock renders first line at y = ascent.
    let note_margin_y = 5.0; // AbstractTextualComponent.marginY for notes
    let text_y = note.y + note_margin_y + font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let note_line_height = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        text_x,
        text_y,
        note_line_height,
        TEXT_COLOR,
        None,
        &format!(r#"font-size="{FONT_SIZE}""#),
    );
    sg.push_raw(&tmp);
}

// ── Group frames ────────────────────────────────────────────────────

fn draw_group(sg: &mut SvgGraphic, group: &GroupLayout) {
    let height = group.y_end - group.y_start;

    // Frame rectangle
    let mut tmp = String::new();
    write!(
        tmp,
        r#"<rect fill="{bg}" fill-opacity="0.30000" height="{}" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(height), fmt_coord(group.width), fmt_coord(group.x), fmt_coord(group.y_start),
        bg = GROUP_BG,
        border = TEXT_COLOR,
    )
    .unwrap();
    sg.push_raw(&tmp);
    sg.push_raw("\n");

    // Label in top-left corner
    if let Some(label) = &group.label {
        let label_x = group.x + 6.0;
        let label_y = group.y_start + FONT_SIZE + 2.0;

        // Label background tab
        let label_width =
            font_metrics::text_width(label, "SansSerif", FONT_SIZE, true, false) + 12.0;
        let label_height = FONT_SIZE + 6.0;
        let mut tmp = String::new();
        write!(
            tmp,
            r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(label_height), fmt_coord(label_width), fmt_coord(group.x), fmt_coord(group.y_start),
            bg = GROUP_BG,
            border = TEXT_COLOR,
        )
        .unwrap();
        sg.push_raw(&tmp);
        sg.push_raw("\n");

        let tl = font_metrics::text_width(label, "SansSerif", FONT_SIZE, true, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            label,
            label_x,
            label_y,
            Some("sans-serif"),
            FONT_SIZE,
            Some("bold"),
            None,
            None,
            tl,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        sg.push_raw("\n");
    }
}

// ── Fragment frames ──────────────────────────────────────────────────

/// Phase 1: Draw just the frame outline rect (before lifelines)
fn draw_fragment_frame(sg: &mut SvgGraphic, frag: &FragmentLayout) {
    let fx = fmt_coord(frag.x);
    let fy = fmt_coord(frag.y);
    let fw = fmt_coord(frag.width);
    let fh = fmt_coord(frag.height);
    sg.push_raw(&format!(
        "<rect fill=\"none\" height=\"{fh}\" style=\"stroke:#000000;stroke-width:1.5;\" width=\"{fw}\" x=\"{fx}\" y=\"{fy}\"/>"
    ));
}

/// Phase 2: Draw pentagon tab, labels, separators (after messages)
fn draw_fragment_details(sg: &mut SvgGraphic, frag: &FragmentLayout) {
    let fx = fmt_coord(frag.x);
    let fy = fmt_coord(frag.y);
    let fw = fmt_coord(frag.width);
    let fh = fmt_coord(frag.height);

    // Label tab (pentagon in top-left)
    // For Group, the tab shows the label directly; for others, tab shows the keyword
    let is_group = frag.kind == FragmentKind::Group;
    let tab_text = if is_group && !frag.label.is_empty() {
        frag.label.clone()
    } else {
        frag.kind.label().to_string()
    };
    let tab_text_w = font_metrics::text_width(&tab_text, "SansSerif", FONT_SIZE, true, false);
    let tab_right = frag.x + FRAG_TAB_LEFT_PAD + tab_text_w + FRAG_TAB_RIGHT_PAD;

    // Pentagon path
    sg.push_raw(&format!(
        "<path d=\"M{fx},{fy} L{},{fy} L{},{} L{},{} L{fx},{} L{fx},{fy}\" fill=\"#EEEEEE\" style=\"stroke:#000000;stroke-width:1.5;\"/>",
        fmt_coord(tab_right),
        fmt_coord(tab_right), fmt_coord(frag.y + FRAG_TAB_HEIGHT - FRAG_TAB_NOTCH),
        fmt_coord(tab_right - FRAG_TAB_NOTCH), fmt_coord(frag.y + FRAG_TAB_HEIGHT),
        fmt_coord(frag.y + FRAG_TAB_HEIGHT),
    ));

    // Second frame rect (Java emits two)
    sg.push_raw(&format!(
        "<rect fill=\"none\" height=\"{fh}\" style=\"stroke:#000000;stroke-width:1.5;\" width=\"{fw}\" x=\"{fx}\" y=\"{fy}\"/>"
    ));

    // Tab label text (font-size 13, bold)
    let text_x = frag.x + FRAG_TAB_LEFT_PAD;
    let text_y = frag.y + FRAG_KIND_LABEL_Y_OFFSET;
    sg.set_fill_color("#000000");
    sg.svg_text(
        &tab_text,
        text_x,
        text_y,
        Some("sans-serif"),
        13.0,
        Some("700"),
        None,
        None,
        tab_text_w,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );

    // Guard text (font-size 11, bold) — only for non-Group fragments
    if !is_group && !frag.label.is_empty() {
        let guard_text = format!("[{}]", frag.label);
        let guard_w = font_metrics::text_width(&guard_text, "SansSerif", FRAG_GUARD_FONT_SIZE, true, false);
        let guard_x = tab_right + FRAG_GUARD_GAP;
        let guard_y = frag.y + FRAG_GUARD_LABEL_Y_OFFSET;
        sg.set_fill_color("#000000");
        sg.svg_text(
            &guard_text,
            guard_x,
            guard_y,
            Some("sans-serif"),
            FRAG_GUARD_FONT_SIZE,
            Some("700"),
            None,
            None,
            guard_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }

    // Note: separators are rendered inline with messages via draw_fragment_separator
}

/// Draw a single separator line + label within a fragment
fn draw_fragment_separator(sg: &mut SvgGraphic, frag: &FragmentLayout, sep_y: f64, sep_label: &str) {
    let fx = fmt_coord(frag.x);
    let y_s = fmt_coord(sep_y);
    sg.push_raw(&format!(
        "<line style=\"stroke:#000000;stroke-width:1;stroke-dasharray:2,2;\" x1=\"{fx}\" x2=\"{}\" y1=\"{y_s}\" y2=\"{y_s}\"/>",
        fmt_coord(frag.x + frag.width),
    ));

    if !sep_label.is_empty() {
        let bracket_text = format!("[{sep_label}]");
        let sep_tl = font_metrics::text_width(&bracket_text, "SansSerif", 11.0, true, false);
        let label_x = frag.x + 5.0;
        let label_y = sep_y + 10.2105;
        sg.set_fill_color("#000000");
        sg.svg_text(
            &bracket_text,
            label_x,
            label_y,
            Some("sans-serif"),
            11.0,
            Some("700"),
            None,
            None,
            sep_tl,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
}

// ── Divider ──────────────────────────────────────────────────────────

/// Draw a divider. Java: ComponentRoseDivider.drawInternalU
///
/// The divider component draws relative to its component origin (component_y),
/// which corresponds to Java's startingY (= freeY at divider creation time).
fn draw_divider(sg: &mut SvgGraphic, divider: &DividerLayout) {
    // Java: center_y = component_y + area.height / 2
    let center_y = divider.component_y + divider.height / 2.0;

    // Divider colors: Java default rose.skin separator style
    // background = #EEEEEE, borderColor = #000000
    let bg_color = "#EEEEEE";
    let border_color = "#000000";

    // Java: drawRectLong at center - 1, height=3, stroke=simple(bg), fill=bg
    let rect_y = center_y - 1.0;
    let mut tmp = String::new();
    write!(
        tmp,
        r#"<rect fill="{bg}" height="3" style="stroke:{bg};stroke-width:1;" width="{w}" x="{x}" y="{y}"/>"#,
        bg = bg_color,
        w = fmt_coord(divider.width),
        x = fmt_coord(divider.x),
        y = fmt_coord(rect_y),
    )
    .unwrap();
    sg.push_raw(&tmp);

    // Java: drawDoubleLine - two lines at center-1 and center+2, stroke=borderColor
    {
        let mut tmp = String::new();
        write!(
            tmp,
            r#"<line style="stroke:{color};stroke-width:1;" x1="{x1}" x2="{x2}" y1="{y}" y2="{y}"/>"#,
            color = border_color,
            x1 = fmt_coord(divider.x),
            x2 = fmt_coord(divider.x + divider.width),
            y = fmt_coord(center_y - 1.0),
        )
        .unwrap();
        sg.push_raw(&tmp);
    }
    {
        let mut tmp = String::new();
        write!(
            tmp,
            r#"<line style="stroke:{color};stroke-width:1;" x1="{x1}" x2="{x2}" y1="{y}" y2="{y}"/>"#,
            color = border_color,
            x1 = fmt_coord(divider.x),
            x2 = fmt_coord(divider.x + divider.width),
            y = fmt_coord(center_y + 2.0),
        )
        .unwrap();
        sg.push_raw(&tmp);
    }

    // Centered label text with bordered rect
    if let Some(text) = &divider.text {
        // Java: textHeight = textBlock.height + 2*marginY(4)
        // For single-line: textBlock.height = line_height(13) = 15.1328
        // textHeight = 15.1328 + 8 = 23.1328
        let text_line_h = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);
        let margin_y = 4.0;
        let text_height = text_line_h + 2.0 * margin_y;

        // Java: textWidth = textBlock.width + marginX1(4) + marginX2(4)
        // Java's divider regex captures the label with a trailing space
        // (e.g., "Initialization " from "== Initialization =="), so the
        // textBlock dimension includes the space advance. The SVG text
        // rendering trims it, but the rect size uses the untrimmed width.
        let text_with_space = format!("{} ", text);
        let text_block_w = font_metrics::text_width(&text_with_space, "SansSerif", FONT_SIZE, true, false);
        let text_width = text_block_w + 4.0 + 4.0;
        let delta_x = 6.0;

        // Position centered in area
        let xpos = divider.component_y; // dummy, we compute from area
        let area_width = divider.width;
        let rect_x = (area_width - text_width - delta_x) / 2.0 + divider.x;
        let rect_y = divider.component_y + (divider.height - text_height) / 2.0;

        // Java: rect with stroke=borderColor, stroke-width=2 (UStroke default)
        let mut tmp = String::new();
        write!(
            tmp,
            r#"<rect fill="{bg}" height="{h}" style="stroke:{border};stroke-width:2;" width="{w}" x="{x}" y="{y}"/>"#,
            bg = bg_color,
            h = fmt_coord(text_height),
            w = fmt_coord(text_width + delta_x),
            x = fmt_coord(rect_x),
            y = fmt_coord(rect_y),
            border = border_color,
        )
        .unwrap();
        sg.push_raw(&tmp);

        // Java: textBlock drawn at (xpos + deltaX, ypos + marginY)
        let text_x = rect_x + delta_x;
        let text_baseline_y = rect_y + margin_y
            + font_metrics::ascent("SansSerif", FONT_SIZE, true, false);
        let tl = font_metrics::text_width(text, "SansSerif", FONT_SIZE, true, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            text,
            text_x,
            text_baseline_y,
            Some("sans-serif"),
            FONT_SIZE,
            Some("bold"),
            None,
            None,
            tl,
            LengthAdjust::Spacing,
            None,
            0,
            None, // left-aligned (not centered)
        );
    }
}

// ── Delay ────────────────────────────────────────────────────────────

/// Draw delay text. Java: ComponentRoseDelayText.drawInternalU + GraphicalDelayText
///
/// The delay text is centered between the first and last participant's
/// lifeline positions. The dotted lifeline is handled by lifeline splitting.
fn draw_delay(sg: &mut SvgGraphic, delay: &DelayLayout, layout: &SeqLayout) {
    // Java: ComponentRoseDelayText only draws text, no dots/circles.
    if let Some(text) = &delay.text {
        let tl = font_metrics::text_width(text, "SansSerif", DELAY_FONT_SIZE, false, false);

        // Java: GraphicalDelayText computes middle from first/last participant getCenterX.
        // getCenterX = startingX + head.preferredWidth/2.0 + outMargin (exact, no integer div).
        // Our p.x corresponds to getCenterX.
        let first_p = layout.participants.first();
        let last_p = layout.participants.last();
        let mid_x = match (first_p, last_p) {
            (Some(fp), Some(lp)) => (fp.x + lp.x) / 2.0,
            _ => delay.x + delay.width / 2.0,
        };
        let text_x = mid_x - tl / 2.0;

        // Y position: centered in component area, then offset by marginY + ascent
        let text_line_h = font_metrics::line_height("SansSerif", DELAY_FONT_SIZE, false, false);
        let margin_y = 4.0;
        let text_height = text_line_h + 2.0 * margin_y;
        let ypos = (delay.height - text_height) / 2.0;
        let text_y = delay.lifeline_break_y + ypos + margin_y
            + font_metrics::ascent("SansSerif", DELAY_FONT_SIZE, false, false);

        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            text,
            text_x,
            text_y,
            Some("sans-serif"),
            DELAY_FONT_SIZE,
            None,
            None,
            None,
            tl,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
}

// ── Ref ──────────────────────────────────────────────────────────────

fn draw_ref(sg: &mut SvgGraphic, r: &RefLayout) {
    let ref_text_w = font_metrics::text_width("ref", "SansSerif", FONT_SIZE, true, false);
    let tab_text_w_int = ref_text_w.floor();
    let tab_right = r.x + FRAG_TAB_LEFT_PAD + tab_text_w_int + FRAG_TAB_RIGHT_PAD;
    let rx_s = fmt_coord(r.x);
    let ry_s = fmt_coord(r.y);
    sg.push_raw(&format!(
        "<rect fill=\"none\" height=\"{}\" style=\"stroke:{REF_FRAME_STROKE};stroke-width:1.5;\" width=\"{}\" x=\"{rx_s}\" y=\"{ry_s}\"/>",
        fmt_coord(r.height), fmt_coord(r.width),
    ));
    sg.push_raw(&format!(
        "<path d=\"M{rx_s},{ry_s} L{},{ry_s} L{},{} L{},{} L{rx_s},{} L{rx_s},{ry_s}\" fill=\"{GROUP_BG}\" style=\"stroke:{REF_FRAME_STROKE};stroke-width:2;\"/>",
        fmt_coord(tab_right),
        fmt_coord(tab_right), fmt_coord(r.y + REF_TAB_HEIGHT - REF_TAB_NOTCH),
        fmt_coord(tab_right - REF_TAB_NOTCH), fmt_coord(r.y + REF_TAB_HEIGHT),
        fmt_coord(r.y + REF_TAB_HEIGHT),
    ));
    sg.set_fill_color(TEXT_COLOR);
    sg.svg_text(
        "ref",
        r.x + REF_TAB_LEFT_PAD,
        r.y + REF_KIND_LABEL_Y_OFFSET,
        Some("sans-serif"),
        FONT_SIZE,
        Some("700"),
        None,
        None,
        ref_text_w,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    let label_w = font_metrics::text_width(&r.label, "SansSerif", REF_LABEL_FONT_SIZE, false, false);
    let center_x = r.x + r.width / 2.0;
    let label_x = center_x - label_w / 2.0;
    let body_top = r.y + REF_TAB_HEIGHT;
    let body_height = r.height - REF_TAB_HEIGHT;
    let line_h = font_metrics::line_height("SansSerif", REF_LABEL_FONT_SIZE, false, false);
    let asc = font_metrics::ascent("SansSerif", REF_LABEL_FONT_SIZE, false, false);
    let top_margin = ((body_height - line_h) / 2.0).floor();
    let label_y = body_top + top_margin + asc;
    sg.set_fill_color(TEXT_COLOR);
    sg.svg_text(
        &r.label,
        label_x,
        label_y,
        Some("sans-serif"),
        REF_LABEL_FONT_SIZE,
        None,
        None,
        None,
        label_w,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
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
    // Apply skinparam font overrides
    let font = skin.default_font_name()
        .or_else(|| if skin.is_handwritten() { Some("Comic Sans MS, Segoe Print, cursive") } else { None })
        .map(|s| s.to_string());
    set_default_font_family(font);
    enable_path_sprites();
    crate::render::svg_sprite::clear_gradient_defs();
    crate::render::svg_sprite::set_monochrome(skin.is_monochrome());
    let result = render_sequence_inner(sd, layout, skin);
    crate::render::svg_sprite::set_monochrome(false);
    disable_path_sprites();
    set_default_font_family(None);
    result
}

fn render_sequence_inner(
    sd: &SequenceDiagram,
    layout: &SeqLayout,
    skin: &SkinParams,
) -> Result<String> {
    // Layout includes margins; apply Java ensureVisible (int)(x+1) truncation
    let svg_w = ensure_visible_int(layout.total_width) as f64;
    let svg_h = ensure_visible_int(layout.total_height) as f64;

    let mut buf = String::with_capacity(4096);

    // 1. SVG header
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, svg_w, svg_h, "SEQUENCE", bg);

    // 2. Create SvgGraphic for all rendering helpers
    let mut sg = SvgGraphic::new(0, 1.0);

    // Write defs placeholder and open group
    write_seq_defs(&mut sg);
    sg.push_raw("<g>");
    {
        let mut tmp = String::new();
        write_bg_rect(&mut tmp, svg_w, svg_h, bg);
        sg.push_raw(&tmp);
    }

    // Build participant name -> index mapping
    let part_index = build_participant_index(sd);
    let display_names: std::collections::HashMap<&str, &str> = sd
        .participants
        .iter()
        .filter_map(|p| p.display_name.as_deref().map(|dn| (p.name.as_str(), dn)))
        .collect();

    // 3. Fragment frame rects — rendering order depends on engine.
    // Puma: fragments BEFORE lifelines (Java DrawableSet order).
    // Teoz: fragments AFTER lifelines AND participant boxes (Java MainTile order).
    if !sd.teoz_mode {
        let mut sorted_frags: Vec<&FragmentLayout> = layout.fragments.iter().collect();
        sorted_frags.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });
        for frag in &sorted_frags {
            draw_fragment_frame(&mut sg, frag);
        }
    }

    // 4/5. Activation bars and lifelines — order depends on engine:
    // Teoz: lifelines first, then activations (Java MainTile draw order)
    // Puma: activations first, then lifelines (Java DrawableSet draw order)
    if sd.teoz_mode {
        draw_lifelines(&mut sg, layout, skin, sd);
        for act in &layout.activations {
            let title = display_names
                .get(act.participant.as_str())
                .copied()
                .unwrap_or(&act.participant);
            draw_activation(&mut sg, act, title);
        }
    } else {
        for act in &layout.activations {
            let title = display_names
                .get(act.participant.as_str())
                .copied()
                .unwrap_or(&act.participant);
            draw_activation(&mut sg, act, title);
        }
        draw_lifelines(&mut sg, layout, skin, sd);
    }

    // 5b. Group frames (legacy, puma only)
    for group in &layout.groups {
        draw_group(&mut sg, group);
    }

    // 5c/5d. Dividers and delays are rendered after participant heads/tails,
    // interleaved with messages (see step 8).

    // 5e. Refs are interleaved with messages (see step 8)

    let default_font = skin
        .get("defaultfontname")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "SansSerif".to_string());
    let msg_font_size: f64 = skin
        .get("defaultfontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(FONT_SIZE);
    let seq_svg_font_family = svg_font_family_attr(&default_font);

    let monochrome = skin.is_monochrome();
    let part_bg = if monochrome {
        "#E3E3E3"
    } else {
        skin.background_color("participant", PARTICIPANT_BG)
    };
    let part_border = skin.border_color("participant", BORDER_COLOR);
    let part_font = skin.font_color("participant", TEXT_COLOR);
    let part_font_size: f64 = skin
        .get("participantfontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| skin.get("defaultfontsize").and_then(|s| s.parse::<f64>().ok()))
        .unwrap_or(14.0);

    // 6. Participant head + tail boxes (interleaved per participant, matching Java order)
    let max_ph = layout.participants.iter().map(|pp| pp.box_height).fold(0.0_f64, f64::max);
    let bottom_y = layout.lifeline_bottom - 1.0;
    for (i, p) in layout.participants.iter().enumerate() {
        let part_idx = i + 1;
        let dn = display_names.get(p.name.as_str()).copied();
        let qualified_name = xml_escape(&p.name);

        // Head (bottom-aligned within head band)
        // Puma: head y starts at MARGIN(5). Teoz: starts at lifeline_top - max_preferred_h.
        let head_base = if sd.teoz_mode {
            layout.lifeline_top - max_ph - 1.0 // teoz: preferred = box + 1
        } else {
            MARGIN
        };
        let top_y = head_base + max_ph - p.box_height;
        let src_line_attr = sd.participants.get(i)
            .and_then(|pp| pp.source_line)
            .map(|sl| format!(r#" data-source-line="{sl}""#))
            .unwrap_or_default();
        let kind = sd.participants.get(i).map(|pp| &pp.kind);
        let is_actor = matches!(kind, Some(ParticipantKind::Actor));
        let part_text_color = skin.font_color("participant", TEXT_COLOR);

        let mut tmp = String::new();
        write!(
            tmp,
            r#"<g class="participant participant-head" data-entity-uid="part{idx}" data-qualified-name="{name}"{src_line} id="part{idx}-head">"#,
            idx = part_idx,
            name = qualified_name,
            src_line = src_line_attr,
        )
        .unwrap();
        sg.push_raw(&tmp);
        let part_link_url = sd.participants.get(i).and_then(|pp| pp.link_url.as_deref());
        if is_actor {
            draw_participant_actor(&mut sg, p, top_y, dn, part_border, part_text_color);
        } else {
            draw_participant_box_with_font(
                &mut sg, p, top_y, dn, part_bg, part_border, part_font,
                &default_font, part_font_size, true, part_link_url,
            );
        }
        sg.push_raw("</g>");

        // Tail (skipped when hide footbox is set)
        if !sd.hide_footbox {
            let mut tmp = String::new();
            write!(
                tmp,
                r#"<g class="participant participant-tail" data-entity-uid="part{idx}" data-qualified-name="{name}"{src_line} id="part{idx}-tail">"#,
                idx = part_idx,
                name = qualified_name,
                src_line = src_line_attr,
            )
            .unwrap();
            sg.push_raw(&tmp);
            if is_actor {
                // Java: actor tail has text ABOVE, stickman BELOW
                draw_participant_actor_tail(&mut sg, p, bottom_y, dn, part_border, part_text_color);
            } else {
                draw_participant_box_with_font(
                    &mut sg, p, bottom_y, dn, part_bg, part_border, part_font,
                    &default_font, part_font_size, false, part_link_url,
                );
            }
            sg.push_raw("</g>");
        }
    }

    // 6b. Fragment frame rects for teoz (after participant boxes).
    // Java: MainTile renders fragments after lifelines and participant boxes.
    if sd.teoz_mode {
        let mut sorted_frags: Vec<&FragmentLayout> = layout.fragments.iter().collect();
        sorted_frags.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });
        for frag in &sorted_frags {
            draw_fragment_frame(&mut sg, frag);
        }
    }

    // 7. Activation bars foreground pass.
    for act in &layout.activations {
        let title = display_names
            .get(act.participant.as_str())
            .copied()
            .unwrap_or(&act.participant);
        draw_activation(&mut sg, act, title);
    }

    // 8. Messages interleaved with fragment details and destroy markers
    // Build a y-sorted list of interstitial events (fragment details + separators)
    // that should be emitted between messages at the appropriate y positions.
    let seq_arrow_color = skin.sequence_arrow_color(BORDER_COLOR);
    let seq_arrow_thickness = skin.sequence_arrow_thickness().unwrap_or(1.0);
    let word_by_word = skin.get("maxmessagesize").is_some();
    let mut msg_seq_counter: usize = 0;

    // Collect all interstitial events: (y, type) sorted by y
    enum InterstitialEvent<'a> {
        FragmentDetail(&'a FragmentLayout),
        Separator(&'a FragmentLayout, f64, &'a str),
        Ref(&'a RefLayout),
        Destroy(&'a DestroyLayout),
        Divider(&'a DividerLayout),
        Delay(&'a DelayLayout),
    }
    let mut interstitials: Vec<(f64, InterstitialEvent)> = Vec::new();
    for frag in &layout.fragments {
        interstitials.push((frag.y, InterstitialEvent::FragmentDetail(frag)));
        for (sep_y, sep_label) in &frag.separators {
            interstitials.push((*sep_y, InterstitialEvent::Separator(frag, *sep_y, sep_label)));
        }
    }
    for r in &layout.refs {
        interstitials.push((r.y, InterstitialEvent::Ref(r)));
    }
    for d in &layout.destroys {
        interstitials.push((d.y, InterstitialEvent::Destroy(d)));
    }
    for div in &layout.dividers {
        interstitials.push((div.y, InterstitialEvent::Divider(div)));
    }
    for delay in &layout.delays {
        interstitials.push((delay.y, InterstitialEvent::Delay(delay)));
    }
    interstitials.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut interstitial_idx = 0;
    let mut drawn_notes = std::collections::HashSet::new();
    for msg in &layout.messages {
        msg_seq_counter += 1;

        // Emit interstitial events that come before this message's y
        while interstitial_idx < interstitials.len()
            && interstitials[interstitial_idx].0 < msg.y
        {
            match &interstitials[interstitial_idx].1 {
                InterstitialEvent::FragmentDetail(frag) => {
                    draw_fragment_details(&mut sg, frag);
                }
                InterstitialEvent::Separator(frag, sep_y, sep_label) => {
                    draw_fragment_separator(&mut sg, frag, *sep_y, sep_label);
                }
                InterstitialEvent::Ref(r) => {
                    draw_ref(&mut sg, r);
                }
                InterstitialEvent::Destroy(d) => {
                    draw_destroy(&mut sg, d);
                }
                InterstitialEvent::Divider(div) => {
                    draw_divider(&mut sg, div);
                }
                InterstitialEvent::Delay(delay) => {
                    draw_delay(&mut sg, delay, layout);
                }
            }
            interstitial_idx += 1;
        }

        // Draw the message
        let from_idx = find_participant_idx_by_x(&layout.participants, msg.from_x, &part_index);
        let to_idx = if msg.is_self {
            from_idx
        } else {
            find_participant_idx_by_x(&layout.participants, msg.to_x, &part_index)
        };

        // Per-message color override from [#color] syntax
        let effective_color = msg.color.as_ref()
            .map(|c| crate::style::normalize_color(c))
            .unwrap_or_else(|| seq_arrow_color.to_string());
        let effective_color = effective_color.as_str();

        if msg.is_self {
            draw_self_message(
                &mut sg,
                msg,
                effective_color,
                seq_arrow_thickness,
                &default_font,
                msg_font_size,
                from_idx,
                msg_seq_counter,
                word_by_word,
            );
        } else {
            draw_message(
                &mut sg,
                msg,
                effective_color,
                seq_arrow_thickness,
                &default_font,
                seq_svg_font_family,
                msg_font_size,
                from_idx,
                to_idx,
                msg_seq_counter,
                msg.source_line,
                word_by_word,
            );
        }

        // Draw notes associated with this message (Java renders notes
        // inline after their associated message, not in a separate pass).
        // For multiline self-messages, the note can be far above msg.y
        // due to Java ArrowAndNoteBox centering (up to ~half the text height).
        let next_msg_y = layout.messages.get(msg_seq_counter)
            .map_or(f64::MAX, |m| m.y);
        // Use larger back-threshold for self-messages (multiline text pushes
        // the note much further above msg.y than for regular messages).
        let note_back_threshold = if msg.is_self { 200.0 } else { 30.0 };
        let mut has_note = false;
        for (ni, note) in layout.notes.iter().enumerate() {
            if !drawn_notes.contains(&ni) && note.y >= msg.y - note_back_threshold && note.y < next_msg_y {
                draw_note(&mut sg, note);
                drawn_notes.insert(ni);
                has_note = true;
            }
        }
        // In Java, when a message has notes, it's wrapped in ArrowAndNoteBox
        // which consumes an extra counter value. Advance to match Java's
        // msg id numbering.
        if has_note {
            msg_seq_counter += 1;
        }
    }

    // Draw standalone notes (not associated with any message)
    if layout.messages.is_empty() {
        for note in &layout.notes {
            draw_note(&mut sg, note);
        }
    }

    // Emit any remaining interstitial events
    while interstitial_idx < interstitials.len() {
        match &interstitials[interstitial_idx].1 {
            InterstitialEvent::FragmentDetail(frag) => {
                draw_fragment_details(&mut sg, frag);
            }
            InterstitialEvent::Separator(frag, sep_y, sep_label) => {
                draw_fragment_separator(&mut sg, frag, *sep_y, sep_label);
            }
            InterstitialEvent::Ref(r) => {
                draw_ref(&mut sg, r);
            }
            InterstitialEvent::Destroy(d) => {
                draw_destroy(&mut sg, d);
            }
            InterstitialEvent::Divider(div) => {
                draw_divider(&mut sg, div);
            }
            InterstitialEvent::Delay(delay) => {
                draw_delay(&mut sg, delay, layout);
            }
        }
        interstitial_idx += 1;
    }

    sg.push_raw("</g></svg>");

    // Append SvgGraphic body to the buf (which has the SVG root header)
    buf.push_str(sg.body());

    // Post-process: inject gradient defs and filter definitions
    let gradient_defs = crate::render::svg_sprite::take_gradient_defs();
    let filters = take_back_filters();
    if !gradient_defs.is_empty() || !filters.is_empty() {
        let mut defs_content = String::new();
        for (_id, def_xml) in &gradient_defs { defs_content.push_str(def_xml); }
        for (id, hex_color) in &filters {
            write!(
                defs_content,
                r#"<filter height="1" id="{}" width="1" x="0" y="0"><feFlood flood-color="{}" result="flood"/><feComposite in="SourceGraphic" in2="flood" operator="over"/></filter>"#,
                id, hex_color,
            )
            .unwrap();
        }
        buf = buf.replacen("<defs/>", &format!("<defs>{}</defs>", defs_content), 1);
    }

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

// Rendering correctness is verified by reference_tests.rs (full-pipeline SVG
// compared against Java gold-standard SVGs) — the same method Java uses
// (checkImage → TestResult comparison).  These smoke tests only verify:
// no panic, output is valid SVG.  All structural/coordinate assertions
// belong in reference_tests, not here.

#[cfg(test)]
mod tests {
    fn convert(puml: &str) -> String {
        crate::convert(puml).expect("convert must succeed")
    }

    #[test]
    fn smoke_simple_message() {
        let svg = convert("@startuml\nAlice -> Bob : hello\n@enduml");
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn smoke_self_message() {
        let svg = convert("@startuml\nA -> A : self\n@enduml");
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn smoke_dashed_open_head() {
        let svg = convert("@startuml\nA --> B : reply\n@enduml");
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn smoke_destroy() {
        let svg = convert("@startuml\nA -> B : kill\ndestroy B\n@enduml");
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn smoke_note() {
        let svg = convert("@startuml\nA -> B : msg\nnote right: a note\n@enduml");
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn smoke_activation() {
        let svg = convert("@startuml\nA -> B : req\nactivate B\nB --> A : resp\ndeactivate B\n@enduml");
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn smoke_all_participant_kinds() {
        let svg = convert(
            "@startuml\nactor A\nboundary B\ncontrol C\ndatabase D\n\
             entity E\ncollections F\nqueue G\nparticipant H\nA -> H : msg\n@enduml",
        );
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn smoke_empty() {
        // Empty @startuml/@enduml may not parse as sequence diagram;
        // just verify it doesn't panic
        let _ = crate::convert("@startuml\n@enduml");
    }
}
