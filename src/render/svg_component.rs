use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, svg_comment_escape, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::component::{
    ComponentEdgeLayout, ComponentGroupLayout, ComponentLayout, ComponentNodeLayout,
    ComponentNoteLayout,
};
use crate::model::component::{ComponentDiagram, ComponentKind};
use crate::render::svg::{write_bg_rect, write_svg_root_bg};
use crate::render::svg_richtext::{get_sprite_svg, render_creole_text, render_creole_text_opts};
use crate::render::svg_sprite;
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants (PlantUML defaults)
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 14.0;
const LINE_HEIGHT: f64 = 16.0;
use crate::skin::rose::{ACTIVATION_BG, BORDER_COLOR, ENTITY_BG, NOTE_BG, NOTE_BORDER, TEXT_COLOR};

/// Compute the `textLength` attribute value for a text string at the given
/// font-size using the font-metrics table.
fn text_len(text: &str, size: f64, bold: bool) -> f64 {
    font_metrics::text_width(text, "sans-serif", size, bold, false)
}

/// Parse a CSS hex color string like "#F1F1F1" into (r, g, b) components.
fn parse_hex_color(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.strip_prefix('#')?;
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_component(
    cd: &ComponentDiagram,
    layout: &ComponentLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // Build entity ID map: entity name → "ent0002", "ent0003", etc.
    // Java assigns IDs in definition order (source_line), including notes
    // as real entities (GMN*). We interleave entities and notes by source_line.
    let mut entity_ids: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Collect all items that need ent IDs: entities + notes, sorted by source_line
    enum EntItem<'a> {
        Entity(&'a crate::model::component::ComponentEntity),
        Note(usize), // index into layout.notes
    }
    let mut all_items: Vec<(usize, EntItem<'_>)> = Vec::new();
    for ent in &cd.entities {
        all_items.push((ent.source_line.unwrap_or(usize::MAX), EntItem::Entity(ent)));
    }
    for (i, note) in layout.notes.iter().enumerate() {
        all_items.push((note.source_line.unwrap_or(usize::MAX), EntItem::Note(i)));
    }
    all_items.sort_by_key(|(sl, _)| *sl);

    let mut ent_counter = 2u32;
    let mut note_ent_ids: std::collections::HashMap<usize, String> =
        std::collections::HashMap::new();
    for (_, item) in &all_items {
        let ent_id = format!("ent{:04}", ent_counter);
        match item {
            EntItem::Entity(ent) => {
                entity_ids.insert(ent.id.clone(), ent_id);
            }
            EntItem::Note(idx) => {
                note_ent_ids.insert(*idx, ent_id);
            }
        }
        ent_counter += 1;
    }
    let qualified_names = build_component_qualified_names(cd);
    let entity_parents: std::collections::HashMap<String, Option<String>> = cd
        .entities
        .iter()
        .map(|ent| (ent.id.clone(), ent.parent.clone()))
        .collect();
    let group_center_y: std::collections::HashMap<String, f64> = layout
        .groups
        .iter()
        .map(|group| (group.id.clone(), group.y + group.height / 2.0))
        .collect();

    // Skin color lookups
    let comp_bg = skin.background_color("component", ENTITY_BG);
    let comp_border = skin.border_color("component", BORDER_COLOR);
    let comp_font = skin.font_color("component", TEXT_COLOR);
    let rect_bg = skin.background_color("rectangle", ENTITY_BG);
    let rect_border = skin.border_color("rectangle", BORDER_COLOR);
    let db_bg = skin.background_color("database", ENTITY_BG);
    let db_border = skin.border_color("database", BORDER_COLOR);
    let cloud_bg = skin.background_color("cloud", ENTITY_BG);
    let cloud_border = skin.border_color("cloud", BORDER_COLOR);
    let node_bg = skin.background_color("node", ENTITY_BG);
    let node_border = skin.border_color("node", BORDER_COLOR);
    let note_bg = skin.background_color("note", NOTE_BG);
    let note_border = skin.border_color("note", NOTE_BORDER);
    let note_font = skin.font_color("note", TEXT_COLOR);
    let group_bg = skin.background_color("package", ACTIVATION_BG);
    let group_border = skin.border_color("package", BORDER_COLOR);
    let group_font = skin.font_color("package", TEXT_COLOR);
    let arrow_color = skin.arrow_color(BORDER_COLOR);
    // Deployment diagram skin lookups
    let artifact_bg = skin.background_color("artifact", ENTITY_BG);
    let artifact_border = skin.border_color("artifact", BORDER_COLOR);
    let storage_bg = skin.background_color("storage", ENTITY_BG);
    let storage_border = skin.border_color("storage", BORDER_COLOR);
    let folder_bg = skin.background_color("folder", ENTITY_BG);
    let folder_border = skin.border_color("folder", BORDER_COLOR);
    let frame_bg = skin.background_color("frame", ACTIVATION_BG);
    let frame_border = skin.border_color("frame", BORDER_COLOR);
    let agent_bg = skin.background_color("agent", ENTITY_BG);
    let agent_border = skin.border_color("agent", BORDER_COLOR);
    let stack_bg = skin.background_color("stack", ENTITY_BG);
    let stack_border = skin.border_color("stack", BORDER_COLOR);
    let queue_bg = skin.background_color("queue", ENTITY_BG);
    let queue_border = skin.border_color("queue", BORDER_COLOR);

    // SVG header
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "DESCRIPTION", bg);

    // Empty defs to match Java PlantUML
    buf.push_str("<defs/>");
    buf.push_str("<g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Groups (render before nodes so they appear behind)
    for group in &layout.groups {
        let ent_id = entity_ids
            .get(&group.id)
            .map(String::as_str)
            .unwrap_or_default();
        let qualified_name = qualified_names
            .get(&group.id)
            .map(String::as_str)
            .unwrap_or(group.id.as_str());
        render_group(
            &mut sg,
            group,
            ent_id,
            qualified_name,
            group_bg,
            group_border,
            group_font,
        );
    }

    // Nodes
    for node in &layout.nodes {
        let parent_id = entity_parents
            .get(&node.id)
            .and_then(|parent| parent.as_deref());
        let port_label_above = matches!(node.kind, ComponentKind::PortIn | ComponentKind::PortOut)
            && parent_id
                .and_then(|parent| group_center_y.get(parent))
                .is_some_and(|center_y| node.y < *center_y);
        let meta = EntitySvgMeta {
            ent_id: entity_ids
                .get(&node.id)
                .map(String::as_str)
                .unwrap_or_default(),
            qualified_name: qualified_names
                .get(&node.id)
                .map(String::as_str)
                .unwrap_or(node.id.as_str()),
            emit_comment: !matches!(node.kind, ComponentKind::PortIn | ComponentKind::PortOut),
            port_label_above,
        };
        render_node(
            &mut sg,
            node,
            meta,
            comp_bg,
            comp_border,
            comp_font,
            rect_bg,
            rect_border,
            db_bg,
            db_border,
            cloud_bg,
            cloud_border,
            node_bg,
            node_border,
            artifact_bg,
            artifact_border,
            storage_bg,
            storage_border,
            folder_bg,
            folder_border,
            frame_bg,
            frame_border,
            agent_bg,
            agent_border,
            stack_bg,
            stack_border,
            queue_bg,
            queue_border,
        );
    }

    // Edges — link IDs start after entity IDs.
    // Java uses a shared counter for entities and links. When a forward arrow has
    // direction UP/LEFT, Java calls Link.getInv() which creates a second Link
    // consuming an extra counter value. We replicate that by bumping by 2 for
    // direction-inverted links and using the second value.
    let mut path_id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut link_counter = ent_counter;
    for (ei, edge) in layout.edges.iter().enumerate() {
        let source_line = cd.links.get(ei).and_then(|l| l.source_line);
        let direction_inverted = cd.links.get(ei).is_some_and(|l| l.direction_inverted);
        if direction_inverted {
            // Forward arrow with UP/LEFT: Java creates original link (counter N)
            // then getInv() (counter N+1). The inverted copy is kept.
            link_counter += 1;
        }
        render_edge(
            &mut sg,
            edge,
            arrow_color,
            comp_font,
            &entity_ids,
            link_counter,
            source_line,
            &mut path_id_counts,
            direction_inverted,
            &layout.nodes,
        );
        link_counter += 1;
    }

    // Notes — wrapped in <g class="entity"> like Java's EntityImageNote
    for (i, note) in layout.notes.iter().enumerate() {
        let ent_id = note_ent_ids
            .get(&i)
            .map(String::as_str)
            .unwrap_or("ent9999");
        render_note(&mut sg, note, note_bg, note_border, note_font, ent_id);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Group rendering (cluster)
// ---------------------------------------------------------------------------

fn render_group(
    sg: &mut SvgGraphic,
    group: &ComponentGroupLayout,
    ent_id: &str,
    qualified_name: &str,
    _bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = group.x;
    let y = group.y;
    let w = group.width;
    let h = group.height;

    // HTML comment — Java replaces non-ASCII and newlines with '?'
    let comment_id = group.id.replace('\n', "?").replace(crate::NEWLINE_CHAR, "?");
    sg.push_raw(&format!(
        "<!--cluster {}-->",
        svg_comment_escape(&comment_id)
    ));

    // Open semantic <g> with Java-matching attributes.
    // Java uses '.' for newlines in qualified names (from entity code/name).
    let qn_for_attr = qualified_name
        .replace('\n', ".")
        .replace(crate::NEWLINE_CHAR, ".");
    let mut g_open = format!(
        r#"<g class="cluster" data-qualified-name="{}""#,
        xml_escape(&qn_for_attr)
    );
    if let Some(sl) = group.source_line {
        g_open.push_str(&format!(r#" data-source-line="{}""#, sl));
    }
    g_open.push_str(&format!(r#" id="{ent_id}">"#));
    sg.push_raw(&g_open);

    match group.kind {
        ComponentKind::Component => {
            // Component cluster: rect with component icon (two small rects)
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

            // Component icon on right side
            let icon_w: f64 = 15.0;
            let icon_h: f64 = 10.0;
            let icon_x = x + w - icon_w - 5.0;
            let icon_y1 = y + 5.0;
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(icon_x, icon_y1, icon_w, icon_h, 0.0, 0.0, 0.0);
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(icon_x - 2.0, icon_y1 + 2.0, 4.0, 2.0, 0.0, 0.0, 0.0);
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(icon_x - 2.0, icon_y1 + 6.0, 4.0, 2.0, 0.0, 0.0, 0.0);

            let tl = text_len(&group.name, 14.0, true);
            let text_x = x + (w - tl) / 2.0;
            let text_y = y + 25.9951;
            sg.set_fill_color(font_color);
            sg.svg_text(
                &group.name,
                text_x,
                text_y,
                Some("sans-serif"),
                14.0,
                Some("bold"),
                None,
                None,
                tl,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
        }
        ComponentKind::Frame => {
            // Frame: rect with rx/ry 2.5, path-based label tab
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

            let tl = text_len(&group.name, 14.0, true);
            let tab_w = tl + 9.7041;
            let tab_h = 19.2969;
            let tab_x2 = x + tab_w;
            let tab_y2 = y + tab_h;
            sg.push_raw(&format!(
                r#"<path d="M{},{} L{},{} L{},{} L{},{} " fill="none" style="stroke:{border};stroke-width:1;"/>"#,
                fmt_coord(tab_x2), fmt_coord(y),
                fmt_coord(tab_x2), fmt_coord(tab_y2 - 10.0),
                fmt_coord(tab_x2 - 10.0), fmt_coord(tab_y2),
                fmt_coord(x), fmt_coord(tab_y2),
            ));

            let text_x = x + 3.0;
            let text_y = y + 13.9951;
            sg.set_fill_color(font_color);
            sg.svg_text(
                &group.name,
                text_x,
                text_y,
                Some("sans-serif"),
                14.0,
                Some("bold"),
                None,
                None,
                tl,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
        }
        ComponentKind::Node => {
            // Node: 3D polygon box with depth lines
            let depth = 10.0;
            let p_tl = (x, y + depth);
            let p_tlb = (x + depth, y);
            let p_trb = (x + w, y);
            let p_tr = (x + w, y + depth);
            let p_br = (x + w - depth, y + h);
            let p_bl = (x, y + h);
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(
                0.0,
                &[
                    p_tl.0, p_tl.1, p_tlb.0, p_tlb.1, p_trb.0, p_trb.1, p_trb.0, p_tr.1, p_br.0,
                    p_br.1, p_bl.0, p_bl.1,
                ],
            );

            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(p_br.0, p_tl.1, p_trb.0, p_tlb.1, 0.0);
            sg.svg_line(p_tl.0, p_tl.1, p_br.0, p_tl.1, 0.0);
            sg.svg_line(p_br.0, p_tl.1, p_br.0, p_br.1, 0.0);

            let tl = text_len(&group.name, 14.0, true);
            let text_x = x + (w - depth) / 2.0 - tl / 2.0;
            let text_y = y + depth + 15.9951;
            sg.set_fill_color(font_color);
            sg.svg_text(
                &group.name,
                text_x,
                text_y,
                Some("sans-serif"),
                14.0,
                Some("bold"),
                None,
                None,
                tl,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
        }
        _ => {
            // Default package/rectangle/card: simple rect
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

            // Check for sprite stereotype
            let sprite_h = render_group_sprite(sg, group, x, y, w);

            if matches!(group.kind, ComponentKind::Card) {
                // Card groups: creole-aware name rendering + full-width separator.
                // Java USymbolCard.asBig draws separator; USymbolRectangle.asBig does not.
                let title_h = crate::render::svg_richtext::compute_creole_entity_name_height(
                    &group.name,
                    FONT_SIZE,
                );
                let sep_y = y + 2.0 + sprite_h + title_h + 2.0;
                sg.set_stroke_color(Some(border));
                sg.set_stroke_width(1.0, None);
                sg.svg_line(x, sep_y, x + w, sep_y, 0.0);

                let mut name_buf = String::new();
                crate::render::svg_richtext::render_creole_entity_name(
                    &mut name_buf,
                    &group.name,
                    x,
                    y + sprite_h,
                    w,
                    font_color,
                    border,
                    FONT_SIZE,
                );
                sg.push_raw(&name_buf);
            } else {
                // Non-card groups: plain text rendering with leading-space handling.
                let name_lines: Vec<&str> = group.name.lines().collect();
                let line_h =
                    font_metrics::line_height("SansSerif", FONT_SIZE, true, false);
                let space_w =
                    font_metrics::char_width(' ', "SansSerif", FONT_SIZE, true, false);
                let untrimmed_widths: Vec<f64> = name_lines
                    .iter()
                    .map(|line| font_metrics::text_width(line, "SansSerif", FONT_SIZE, true, false))
                    .collect();
                let max_untrimmed_w = untrimmed_widths.iter().cloned().fold(0.0_f64, f64::max);
                let block_x = x + (w - max_untrimmed_w) / 2.0;
                let name_y_start = y + 2.0 + sprite_h;
                for (li, line) in name_lines.iter().enumerate() {
                    let leading_spaces = line.len() - line.trim_start().len();
                    let leading_w = leading_spaces as f64 * space_w;
                    let display_line = line.trim();
                    let tl = text_len(display_line, 14.0, true);
                    let untrimmed_w = untrimmed_widths[li];
                    let text_x = block_x + (max_untrimmed_w - untrimmed_w) / 2.0 + leading_w;
                    let ascent = font_metrics::ascent("SansSerif", FONT_SIZE, true, false);
                    let text_y = name_y_start + li as f64 * line_h + ascent;
                    sg.set_fill_color(font_color);
                    sg.svg_text(
                        display_line,
                        text_x,
                        text_y,
                        Some("sans-serif"),
                        14.0,
                        Some("bold"),
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
        }
    }

    sg.push_raw("</g>");
}

/// Render a sprite stereotype image for a group, if applicable.
/// Returns the sprite height (0.0 if no sprite).
fn render_group_sprite(
    sg: &mut SvgGraphic,
    group: &ComponentGroupLayout,
    x: f64,
    y: f64,
    w: f64,
) -> f64 {
    let stereo = match &group.stereotype {
        Some(s) if s.starts_with('$') => &s[1..],
        _ => return 0.0,
    };
    let svg_content = match get_sprite_svg(stereo) {
        Some(s) => s,
        None => return 0.0,
    };
    let info = svg_sprite::sprite_info(&svg_content);
    let sprite_w = info.vb_width;
    let sprite_h = info.vb_height;
    // Java: stereotype sprite centered at y=cluster_y+2
    let sprite_x = x + (w - sprite_w) / 2.0;
    let sprite_y = y + 2.0;
    render_sprite_image(sg, &svg_content, sprite_x, sprite_y, sprite_w, sprite_h);
    sprite_h
}

/// Render a sprite as an `<image>` element with inline PNG data URI.
/// Java PlantUML renders monochrome sprites directly as PNG `<image>` elements
/// (not wrapped in SVG containers like stdlib SVG sprites).
fn render_sprite_image(
    sg: &mut SvgGraphic,
    svg_content: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) {
    // Extract the PNG data URI from the sprite SVG.
    // The sprite SVG format: <svg ...><image ... xlink:href="data:image/png;base64,..."/></svg>
    if let Some(href_start) = svg_content.find("xlink:href=\"") {
        let href_val_start = href_start + "xlink:href=\"".len();
        if let Some(href_end) = svg_content[href_val_start..].find('"') {
            let href = &svg_content[href_val_start..href_val_start + href_end];
            if href.starts_with("data:image/png;base64,") {
                sg.push_raw(&format!(
                    r#"<image height="{}" width="{}" x="{}" xlink:href="{}" y="{}"/>"#,
                    h as u32,
                    w as u32,
                    fmt_coord(x),
                    href,
                    fmt_coord(y),
                ));
                return;
            }
        }
    }
    // Fallback: use convert_svg_elements for non-PNG sprites
    let converted = svg_sprite::convert_svg_elements(svg_content, x, y);
    sg.push_raw(&converted);
}

/// Render a sprite with a context-dependent background color.
///
/// Java's `SpriteMonochrome.toUImage` uses the UGraphic back color when generating
/// the sprite image, so transparent pixels have the entity's fill color in their RGB.
/// This function re-generates the sprite PNG with the correct background.
fn render_sprite_with_bg(
    sg: &mut SvgGraphic,
    sprite_name: &str,
    svg_content: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
) {
    use crate::render::svg_richtext::get_sprite_data_uri_with_bg;
    if let Some(data_uri) = get_sprite_data_uri_with_bg(sprite_name, bg_r, bg_g, bg_b) {
        sg.push_raw(&format!(
            r#"<image height="{}" width="{}" x="{}" xlink:href="{}" y="{}"/>"#,
            h as u32,
            w as u32,
            fmt_coord(x),
            data_uri,
            fmt_coord(y),
        ));
        return;
    }
    // Fallback to default rendering
    render_sprite_image(sg, svg_content, x, y, w, h);
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct EntitySvgMeta<'a> {
    ent_id: &'a str,
    qualified_name: &'a str,
    emit_comment: bool,
    port_label_above: bool,
}

#[allow(clippy::too_many_arguments)]
fn render_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    comp_bg: &str,
    comp_border: &str,
    comp_font: &str,
    rect_bg: &str,
    rect_border: &str,
    db_bg: &str,
    db_border: &str,
    cloud_bg: &str,
    cloud_border: &str,
    node_bg: &str,
    node_border: &str,
    artifact_bg: &str,
    artifact_border: &str,
    storage_bg: &str,
    storage_border: &str,
    folder_bg: &str,
    folder_border: &str,
    frame_bg: &str,
    frame_border: &str,
    agent_bg: &str,
    agent_border: &str,
    stack_bg: &str,
    stack_border: &str,
    queue_bg: &str,
    queue_border: &str,
) {
    let color_ref = node.color.as_deref();
    let comp_bg = color_ref.unwrap_or(comp_bg);
    let rect_bg = color_ref.unwrap_or(rect_bg);
    let db_bg = color_ref.unwrap_or(db_bg);
    let cloud_bg = color_ref.unwrap_or(cloud_bg);
    let node_bg = color_ref.unwrap_or(node_bg);
    let artifact_bg = color_ref.unwrap_or(artifact_bg);
    let storage_bg = color_ref.unwrap_or(storage_bg);
    let folder_bg = color_ref.unwrap_or(folder_bg);
    let frame_bg = color_ref.unwrap_or(frame_bg);
    let agent_bg = color_ref.unwrap_or(agent_bg);
    let stack_bg = color_ref.unwrap_or(stack_bg);
    let queue_bg = color_ref.unwrap_or(queue_bg);

    match node.kind {
        ComponentKind::Component => {
            render_component_node(sg, node, meta, comp_bg, comp_border, comp_font);
        }
        ComponentKind::Rectangle => {
            render_rectangle_node(sg, node, meta, rect_bg, rect_border, comp_font);
        }
        ComponentKind::Database => {
            render_database_node(sg, node, meta, db_bg, db_border, comp_font)
        }
        ComponentKind::Cloud => {
            render_cloud_node(sg, node, meta, cloud_bg, cloud_border, comp_font)
        }
        ComponentKind::Node => render_box_node(sg, node, meta, node_bg, node_border, comp_font),
        ComponentKind::Package => render_box_node(sg, node, meta, rect_bg, rect_border, comp_font),
        ComponentKind::Interface => {
            render_interface_node(sg, node, meta, comp_bg, comp_border, comp_font);
        }
        ComponentKind::Card => {
            render_rectangle_node(sg, node, meta, rect_bg, rect_border, comp_font)
        }
        ComponentKind::Artifact => {
            render_artifact_node(sg, node, meta, artifact_bg, artifact_border, comp_font);
        }
        ComponentKind::Storage => {
            render_storage_node(sg, node, meta, storage_bg, storage_border, comp_font);
        }
        ComponentKind::Folder => {
            render_folder_node(sg, node, meta, folder_bg, folder_border, comp_font)
        }
        ComponentKind::Frame => {
            render_frame_node(sg, node, meta, frame_bg, frame_border, comp_font)
        }
        ComponentKind::Agent => {
            render_agent_node(sg, node, meta, agent_bg, agent_border, comp_font)
        }
        ComponentKind::Stack => {
            render_stack_node(sg, node, meta, stack_bg, stack_border, comp_font)
        }
        ComponentKind::Queue => {
            render_queue_node(sg, node, meta, queue_bg, queue_border, comp_font)
        }
        ComponentKind::PortIn | ComponentKind::PortOut => {
            render_port_node(sg, node, meta, comp_bg, comp_border, comp_font);
        }
    }
}

/// Emit HTML comment + open `<g class="entity">` with Java-matching attributes.
fn open_entity_g(sg: &mut SvgGraphic, node: &ComponentNodeLayout, meta: EntitySvgMeta<'_>) {
    if meta.emit_comment {
        sg.push_raw(&format!(
            "<!--entity {}-->",
            svg_comment_escape(&node.id)
        ));
    }
    let source_line = node
        .source_line
        .map_or(String::new(), |l| format!(r#" data-source-line="{}""#, l));
    // Java uses '.' for newlines in qualified names (from entity code/name).
    let qn_for_attr = meta
        .qualified_name
        .replace('\n', ".")
        .replace(crate::NEWLINE_CHAR, ".");
    sg.push_raw(&format!(
        r#"<g class="entity" data-qualified-name="{}"{source_line} id="{ent_id}">"#,
        xml_escape(&qn_for_attr),
        ent_id = meta.ent_id,
    ));
}

/// Component: rounded rect with component icon (two small rects on right side)
fn render_component_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

    // Component icon on right side
    let icon_w: f64 = 15.0;
    let icon_h: f64 = 10.0;
    let icon_x = x + w - icon_w - 5.0;
    let icon_y1 = y + 5.0;
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(icon_x, icon_y1, icon_w, icon_h, 0.0, 0.0, 0.0);
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(icon_x - 2.0, icon_y1 + 2.0, 4.0, 2.0, 0.0, 0.0, 0.0);
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(icon_x - 2.0, icon_y1 + 6.0, 4.0, 2.0, 0.0, 0.0, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Rectangle: simple rectangle
fn render_rectangle_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 2.5, 2.5, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Database: cylinder shape via cubic path curves
fn render_database_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let ry: f64 = 10.0;
    let cx = x + w / 2.0;

    // Body
    sg.push_raw(&format!(
        r#"<path d="M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} L{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} L{},{} " fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(x), fmt_coord(y + ry),
        fmt_coord(x), fmt_coord(y),
        fmt_coord(cx), fmt_coord(y),
        fmt_coord(cx), fmt_coord(y),
        fmt_coord(cx), fmt_coord(y),
        fmt_coord(x + w), fmt_coord(y),
        fmt_coord(x + w), fmt_coord(y + ry),
        fmt_coord(x + w), fmt_coord(y + h - ry),
        fmt_coord(x + w), fmt_coord(y + h),
        fmt_coord(cx), fmt_coord(y + h),
        fmt_coord(cx), fmt_coord(y + h),
        fmt_coord(cx), fmt_coord(y + h),
        fmt_coord(x), fmt_coord(y + h),
        fmt_coord(x), fmt_coord(y + h - ry),
        fmt_coord(x), fmt_coord(y + ry),
    ));

    // Top ellipse
    sg.push_raw(&format!(
        r#"<path d="M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} " fill="none" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(x), fmt_coord(y + ry),
        fmt_coord(x), fmt_coord(y + ry + ry),
        fmt_coord(cx), fmt_coord(y + ry + ry),
        fmt_coord(cx), fmt_coord(y + ry + ry),
        fmt_coord(cx), fmt_coord(y + ry + ry),
        fmt_coord(x + w), fmt_coord(y + ry + ry),
        fmt_coord(x + w), fmt_coord(y + ry),
    ));

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Cloud: rounded rect with large radius
fn render_cloud_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 20.0, 20.0, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Generic box (used for Node, Package)
fn render_box_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 2.5, 2.5, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Interface: small circle with name below
fn render_interface_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let cx = node.x + node.width / 2.0;
    let cy = node.y + 12.0;
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_circle(cx, cy, 8.0, 0.0);

    let name_y = cy + 20.0;
    let tl = text_len(&node.name, 14.0, false);
    sg.set_fill_color(font_color);
    sg.svg_text(
        &node.name,
        cx - tl / 2.0,
        name_y,
        Some("sans-serif"),
        14.0,
        None,
        None,
        None,
        tl,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );

    sg.push_raw("</g>");
}

/// Artifact: rect with folded-corner icon
fn render_artifact_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

    // Folded corner icon (small polygon at top right)
    let fold: f64 = 6.0;
    let ix = x + w - 17.0;
    let iy = y + 5.0;
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_polygon(
        0.0,
        &[
            ix,
            iy,
            ix,
            iy + 14.0,
            ix + 12.0,
            iy + 14.0,
            ix + 12.0,
            iy + fold,
            ix + fold,
            iy,
        ],
    );

    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(ix + fold, iy, ix + fold, iy + fold, 0.0);
    sg.svg_line(ix + 12.0, iy + fold, ix + fold, iy + fold, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Storage: rounded rect with large rx/ry
fn render_storage_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let rx = 35.0_f64.min(node.width / 4.0);
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, rx, rx, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Folder: path with tab, body rect, separator line
fn render_folder_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let tab_w = 41.0_f64.min(w * 0.4);
    let tab_h: f64 = 21.0;
    let r: f64 = 2.5;

    sg.push_raw(&format!(
        concat!(
            r#"<path d="M{},{} L{},{}"#,
            r#" A{},{} 0 0 1 {},{}"#,
            r#" L{},{}"#,
            r#" L{},{}"#,
            r#" A{},{} 0 0 1 {},{}"#,
            r#" L{},{}"#,
            r#" A{},{} 0 0 1 {},{}"#,
            r#" L{},{}"#,
            r#" A{},{} 0 0 1 {},{}"#,
            r#" L{},{} " fill="{}" style="stroke:{};stroke-width:0.5;"/>"#,
        ),
        fmt_coord(x + r),
        fmt_coord(y),
        fmt_coord(x + tab_w),
        fmt_coord(y),
        fmt_coord(r),
        fmt_coord(r),
        fmt_coord(x + tab_w + r),
        fmt_coord(y + r),
        fmt_coord(x + tab_w + r + 7.0),
        fmt_coord(y + tab_h),
        fmt_coord(x + w - r),
        fmt_coord(y + tab_h),
        fmt_coord(r),
        fmt_coord(r),
        fmt_coord(x + w),
        fmt_coord(y + tab_h + r),
        fmt_coord(x + w),
        fmt_coord(y + h - r),
        fmt_coord(r),
        fmt_coord(r),
        fmt_coord(x + w - r),
        fmt_coord(y + h),
        fmt_coord(x + r),
        fmt_coord(y + h),
        fmt_coord(r),
        fmt_coord(r),
        fmt_coord(x),
        fmt_coord(y + h - r),
        fmt_coord(x),
        fmt_coord(y + r),
        bg,
        border,
    ));

    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_line(x, y + tab_h, x + w, y + tab_h, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Frame: rect with label tab
fn render_frame_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    _font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let tab_w = (w * 0.4).min(70.0);
    let tab_h = FONT_SIZE + 6.0;

    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

    sg.set_fill_color(border);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, tab_w, tab_h, 0.0, 0.0, 0.0);

    let label_cx = x + tab_w / 2.0;
    let label_cy = y + tab_h / 2.0 + FONT_SIZE * 0.35;
    let tl = text_len(&node.name, FONT_SIZE - 1.0, true);
    sg.set_fill_color("#FFFFFF");
    sg.svg_text(
        &node.name,
        label_cx,
        label_cy,
        Some("sans-serif"),
        FONT_SIZE - 1.0,
        Some("700"),
        None,
        None,
        tl,
        LengthAdjust::Spacing,
        None,
        0,
        Some("middle"),
    );

    sg.push_raw("</g>");
}

/// Agent: rounded rect with rx 2.5
fn render_agent_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 2.5, 2.5, 0.0);

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Stack: rect with frame path
fn render_stack_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    // Main body rect (stroke:none)
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some("none"));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

    // Frame path
    let bar_left = x - 15.0;
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} A2.5,2.5 0 0 1 {},{} L{},{} A2.5,2.5 0 0 0 {},{} L{},{} A2.5,2.5 0 0 0 {},{} L{},{} A2.5,2.5 0 0 1 {},{} L{},{} " fill="none" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(bar_left), fmt_coord(y),
        fmt_coord(bar_left + 12.5), fmt_coord(y),
        fmt_coord(x), fmt_coord(y + 2.5),
        fmt_coord(x), fmt_coord(y + h - 2.5),
        fmt_coord(x + 2.5), fmt_coord(y + h),
        fmt_coord(x + w - 2.5), fmt_coord(y + h),
        fmt_coord(x + w), fmt_coord(y + h - 2.5),
        fmt_coord(x + w), fmt_coord(y + 2.5),
        fmt_coord(x + w + 2.5), fmt_coord(y),
        fmt_coord(x + w + 15.0), fmt_coord(y),
    ));

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Queue: path body with double-curved right edge
fn render_queue_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let cap: f64 = 5.0;
    let mid_y = y + h / 2.0;

    // Left side curve (filled)
    sg.push_raw(&format!(
        r#"<path d="M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} " fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(x + cap), fmt_coord(y),
        fmt_coord(x + cap + cap), fmt_coord(y),
        fmt_coord(x + cap + cap), fmt_coord(mid_y),
        fmt_coord(x + cap + cap), fmt_coord(mid_y),
        fmt_coord(x + cap + cap), fmt_coord(mid_y),
        fmt_coord(x + cap + cap), fmt_coord(y + h),
        fmt_coord(x + cap), fmt_coord(y + h),
    ));

    // Right endcap (open)
    sg.push_raw(&format!(
        r#"<path d="M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} " fill="none" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(x + w - cap), fmt_coord(y),
        fmt_coord(x + w - cap - cap), fmt_coord(y),
        fmt_coord(x + w - cap - cap), fmt_coord(mid_y),
        fmt_coord(x + w - cap - cap), fmt_coord(mid_y),
        fmt_coord(x + w - cap - cap), fmt_coord(mid_y),
        fmt_coord(x + w - cap - cap), fmt_coord(y + h),
        fmt_coord(x + w - cap), fmt_coord(y + h),
    ));

    render_node_text(sg, node, font_color, bg);
    sg.push_raw("</g>");
}

/// Port: small 12x12 square with text label
fn render_port_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    meta: EntitySvgMeta<'_>,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node, meta);

    let port_size: f64 = 12.0;
    let cx = node.x + node.width / 2.0;
    let ascent = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let descent = font_metrics::descent("SansSerif", FONT_SIZE, false, false);

    // Text label (centered below/above the port square)
    let tl = text_len(&node.name, FONT_SIZE, false);
    let text_x = cx - tl / 2.0;
    let text_y = if meta.port_label_above {
        node.y - port_size - descent
    } else {
        node.y + port_size + ascent
    };
    sg.set_fill_color(font_color);
    sg.svg_text(
        &node.name,
        text_x,
        text_y,
        Some("sans-serif"),
        FONT_SIZE,
        None,
        None,
        None,
        tl,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );

    // Port square
    let port_x = cx - port_size / 2.0;
    let port_y = node.y;
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(1.5, None);
    sg.svg_rectangle(port_x, port_y, port_size, port_size, 0.0, 0.0, 0.0);

    sg.push_raw("</g>");
}

/// Render name, stereotype, and description text for a node
fn render_node_text(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    font_color: &str,
    entity_bg: &str,
) {
    let cx = node.x + node.width / 2.0;
    let has_desc = !node.description.is_empty();

    // Parse entity background color for sprite rendering.
    // Java passes the UGraphic back color to SpriteMonochrome.toUImage, which uses it
    // as the gradient start color (affecting RGB of transparent pixels in the PNG).
    let (bg_r, bg_g, bg_b) = parse_hex_color(entity_bg).unwrap_or((255, 255, 255));

    // Check for sprite stereotype
    let sprite_rendered = if let Some(ref stereotype) = node.stereotype {
        if stereotype.starts_with('$') {
            let sprite_name = &stereotype[1..];
            if let Some(svg_content) = get_sprite_svg(sprite_name) {
                let info = svg_sprite::sprite_info(&svg_content);
                let sprite_w = info.vb_width;
                let sprite_h = info.vb_height;
                // Java USymbolRectangle.asSmall: margin(10,10,10,10)
                // Sprite centered at (cx - sprite_w/2, node.y + margin_top)
                let sprite_x = cx - sprite_w / 2.0;
                let sprite_y = node.y + 10.0; // margin top = 10
                render_sprite_with_bg(
                    sg,
                    sprite_name,
                    &svg_content,
                    sprite_x,
                    sprite_y,
                    sprite_w,
                    sprite_h,
                    bg_r,
                    bg_g,
                    bg_b,
                );
                Some(sprite_h)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Stereotype text (only for non-sprite stereotypes)
    let mut y_offset = 0.0;
    if sprite_rendered.is_none() {
        if let Some(ref stereotype) = node.stereotype {
            let stereo_text = format!("\u{00AB}{stereotype}\u{00BB}");
            let sy = node.y + FONT_SIZE + 4.0;
            let tl =
                font_metrics::text_width(&stereo_text, "sans-serif", FONT_SIZE - 2.0, false, true);
            sg.set_fill_color(font_color);
            sg.svg_text(
                &stereo_text,
                cx - tl / 2.0,
                sy,
                Some("sans-serif"),
                FONT_SIZE - 2.0,
                None,
                Some("italic"),
                None,
                tl,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
            y_offset = LINE_HEIGHT;
        }
    }

    // Type-specific margins (from Java USymbol subclasses)
    let (margin_left, _margin_right, margin_top, _margin_bottom) =
        crate::layout::component::entity_margins(&node.kind);

    // Name positioning
    let name_y = if let Some(sprite_h) = sprite_rendered {
        // Java USymbol.asSmall: label drawn at margin_top + sprite_h + ascent
        let ascent = font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
        node.y + margin_top + sprite_h + ascent
    } else if has_desc {
        node.y + FONT_SIZE + 4.0 + y_offset
    } else {
        // Java: baseline = rect_y + margin_top + ascent
        node.y + margin_top + font_metrics::ascent("SansSerif", FONT_SIZE, false, false)
    };

    // Name text — centered for sprite stereotype, left-aligned otherwise
    let name_x = if sprite_rendered.is_some() {
        let tl = font_metrics::text_width(&node.name, "SansSerif", FONT_SIZE, false, false);
        cx - tl / 2.0
    } else {
        node.x + margin_left
    };
    let tl = font_metrics::text_width(&node.name, "SansSerif", FONT_SIZE, false, false);
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &node.name,
        name_x,
        name_y,
        LINE_HEIGHT,
        font_color,
        None,
        r#"font-size="14""#,
    );
    sg.push_raw(&tmp);

    // Description
    if has_desc {
        let sep_y = name_y + 6.0;
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(1.0, None);
        sg.svg_line(node.x, sep_y, node.x + node.width, sep_y, 0.0);

        let text_x = node.x + 8.0;

        // Check for <code>...</code> wrapper: render as monospace literal
        let is_code_block = node.description.len() >= 2
            && node
                .description
                .first()
                .map_or(false, |l| l.trim().eq_ignore_ascii_case("<code>"))
            && node
                .description
                .last()
                .map_or(false, |l| l.trim().eq_ignore_ascii_case("</code>"));

        if is_code_block {
            // Join inner lines as literal monospace text
            let inner_lines = &node.description[1..node.description.len() - 1];
            let code_text = inner_lines.join("\n");
            let mut tmp = String::new();
            // Render as a single monospace text element with literal content
            let tl = crate::font_metrics::text_width(&code_text, "monospace", 14.0, false, false);
            let text_y = sep_y + LINE_HEIGHT;
            use crate::klimt::svg::{fmt_coord, xml_escape, LengthAdjust};
            sg.set_fill_color(font_color);
            sg.svg_text(
                &code_text,
                text_x + 23.4287,
                text_y,
                Some("monospace"),
                14.0,
                None,
                None,
                None,
                tl,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
        } else {
            // Normal description: preserve literal \n (body context)
            let desc_text = node.description.join("\n");
            let mut tmp = String::new();
            render_creole_text_opts(
                &mut tmp,
                &desc_text,
                text_x,
                sep_y + LINE_HEIGHT,
                LINE_HEIGHT,
                font_color,
                None,
                r#"font-size="12""#,
                true,
            );
            sg.push_raw(&tmp);
        }
    }
}

fn build_component_qualified_names(
    cd: &ComponentDiagram,
) -> std::collections::HashMap<String, String> {
    let parents: std::collections::HashMap<&str, Option<&str>> = cd
        .entities
        .iter()
        .map(|ent| (ent.id.as_str(), ent.parent.as_deref()))
        .collect();

    fn resolve(
        id: &str,
        parents: &std::collections::HashMap<&str, Option<&str>>,
        memo: &mut std::collections::HashMap<String, String>,
    ) -> String {
        if let Some(existing) = memo.get(id) {
            return existing.clone();
        }
        let qualified = match parents.get(id).copied().flatten() {
            Some(parent) => format!("{}.{}", resolve(parent, parents, memo), id),
            None => id.to_string(),
        };
        memo.insert(id.to_string(), qualified.clone());
        qualified
    }

    let mut memo = std::collections::HashMap::new();
    for ent in &cd.entities {
        resolve(&ent.id, &parents, &mut memo);
    }
    memo
}

// ---------------------------------------------------------------------------
// Edge rendering
// ---------------------------------------------------------------------------

fn render_edge(
    sg: &mut SvgGraphic,
    edge: &ComponentEdgeLayout,
    arrow_color: &str,
    font_color: &str,
    entity_ids: &std::collections::HashMap<String, String>,
    link_id: u32,
    source_line: Option<usize>,
    path_id_counts: &mut std::collections::HashMap<String, usize>,
    direction_inverted: bool,
    nodes: &[ComponentNodeLayout],
) {
    if edge.points.is_empty() {
        return;
    }

    // HTML comment — Java: "reverse link X to Y" when LinkType.looksLikeRevertedForSvg()
    if edge.reversed_for_svg {
        sg.push_raw(&format!(
            "<!--reverse link {} to {}-->",
            xml_escape(&edge.from),
            xml_escape(&edge.to),
        ));
    } else {
        sg.push_raw(&format!(
            "<!--link {} to {}-->",
            xml_escape(&edge.from),
            xml_escape(&edge.to),
        ));
    }

    // Semantic group with data attributes matching Java format
    let from_ent = entity_ids.get(&edge.from).map(|s| s.as_str()).unwrap_or("");
    let to_ent = entity_ids.get(&edge.to).map(|s| s.as_str()).unwrap_or("");
    let link_type = if edge.dashed {
        "dependency"
    } else {
        "dependency"
    };
    sg.push_raw(&format!(
        r#"<g class="link" data-entity-1="{from_ent}" data-entity-2="{to_ent}" data-link-type="{link_type}""#,
    ));
    if let Some(sl) = source_line {
        sg.push_raw(&format!(r#" data-source-line="{sl}""#));
    }
    sg.push_raw(&format!(r#" id="lnk{link_id}">"#));

    let dash_style = if edge.dashed {
        "stroke-dasharray:7,7;"
    } else {
        ""
    };

    let pts = &edge.points;
    let arrow_at_start = edge.reversed_for_svg;
    let d = if let Some(ref raw_d) = edge.raw_path_d {
        if arrow_at_start {
            adjust_path_startpoint(raw_d, 6.0)
        } else {
            adjust_path_endpoint(raw_d, 6.0)
        }
    } else {
        let mut d = String::new();
        if !pts.is_empty() {
            write!(d, "M{},{} ", fmt_coord(pts[0].0), fmt_coord(pts[0].1)).unwrap();
            // Points come in groups of 3 for cubic bezier (C command)
            let mut i = 1;
            while i + 2 < pts.len() {
                write!(
                    d,
                    "C{},{} {},{} {},{} ",
                    fmt_coord(pts[i].0),
                    fmt_coord(pts[i].1),
                    fmt_coord(pts[i + 1].0),
                    fmt_coord(pts[i + 1].1),
                    fmt_coord(pts[i + 2].0),
                    fmt_coord(pts[i + 2].1),
                )
                .unwrap();
                i += 3;
            }
            while i < pts.len() {
                write!(d, "L{},{} ", fmt_coord(pts[i].0), fmt_coord(pts[i].1)).unwrap();
                i += 1;
            }
        }
        d.trim_end().to_string()
    };
    let base_path_id = if edge.reversed_for_svg {
        format!("{}-backto-{}", xml_escape(&edge.from), xml_escape(&edge.to))
    } else {
        format!("{}-to-{}", xml_escape(&edge.from), xml_escape(&edge.to))
    };
    let count = path_id_counts.entry(base_path_id.clone()).or_insert(0);
    let path_id = if *count == 0 {
        base_path_id.clone()
    } else {
        format!("{}-{}", base_path_id, count)
    };
    *count += 1;
    sg.push_raw(&format!(
        r#"<path d="{d}" fill="none" id="{path_id}" style="stroke:{arrow_color};stroke-width:1;{dash_style}"/>"#,
    ));

    // Java `ExtremityArrow`: 5-point arrowhead with a contact notch.
    // For reversed ("backto") links the arrow points at the START of the path;
    // for normal links the arrow points at the END.
    if pts.len() >= 2 {
        let (tx, ty, fx, fy) = if arrow_at_start {
            // Arrow at start: tip = pts[0], direction from pts[1] towards pts[0]
            (pts[0].0, pts[0].1, pts[1].0, pts[1].1)
        } else {
            // Arrow at end: tip = last point, direction from second-to-last towards last
            (pts[pts.len() - 1].0, pts[pts.len() - 1].1, pts[pts.len() - 2].0, pts[pts.len() - 2].1)
        };
        let dx = tx - fx;
        let dy = ty - fy;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            let ux = dx / len;
            let uy = dy / len;
            let px = -uy;
            let py = ux;
            let back = 9.0;
            let side = 4.0;
            let mid_back = 5.0;
            let p1x = tx;
            let p1y = ty;
            let p2x = tx - ux * back - px * side;
            let p2y = ty - uy * back - py * side;
            let p3x = tx - ux * mid_back;
            let p3y = ty - uy * mid_back;
            let p4x = tx - ux * back + px * side;
            let p4y = ty - uy * back + py * side;

            sg.set_fill_color(arrow_color);
            sg.set_stroke_color(Some(arrow_color));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[p1x, p1y, p2x, p2y, p3x, p3y, p4x, p4y, p1x, p1y]);
        }
    }

    // Link label rendering matching Java's StringWithArrow + SvekEdge.drawMiddleDecoration().
    // Java uses font-size 13 for link labels and renders direction indicators (> / <) as
    // small triangular polygons. Bold segments get separate <text> elements.
    if !edge.label.is_empty() {
        // Use label_xy from graphviz if available, otherwise fall back to midpoint.
        let (lx, ly) = if let Some((lx, ly)) = edge.label_xy {
            (lx, ly)
        } else {
            let mid = pts.len() / 2;
            if pts.len() == 2 {
                let (x1, y1) = pts[0];
                let (x2, y2) = pts[1];
                ((x1 + x2) / 2.0, (y1 + y2) / 2.0 - 6.0)
            } else {
                pts[mid]
            }
        };

        // Compute edge angle for TextBlockArrow2 direction indicator.
        // Java uses dotPath.getStartPoint()/getEndPoint() AFTER extremity shortening
        // (adjust_path_startpoint/endpoint), so we parse the rendered SVG path `d`.
        // Java SvekEdge.solveLine() also checks whether GraphViz inverted the edge
        // direction by comparing distances to entity centers.
        let edge_angle = {
            let parsed = parse_path_start_end_simple(&d);
            if let Some(((mut sx, mut sy), (mut ex, mut ey))) = parsed {
                // Check for Graphviz path inversion: find entity centers and compare distances.
                let find_center = |name: &str| -> Option<(f64, f64)> {
                    nodes.iter().find(|n| n.id == name).map(|n| (n.x + n.width / 2.0, n.y + n.height / 2.0))
                };
                if let (Some(pos1), Some(pos2)) = (find_center(&edge.from), find_center(&edge.to)) {
                    let dist = |a: (f64, f64), b: (f64, f64)| -> f64 {
                        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
                    };
                    let normal = dist((sx, sy), pos1) + dist((ex, ey), pos2);
                    let inversed = dist((sx, sy), pos2) + dist((ex, ey), pos1);
                    if inversed < normal {
                        std::mem::swap(&mut sx, &mut ex);
                        std::mem::swap(&mut sy, &mut ey);
                    }
                }
                Some((ex - sx).atan2(ey - sy))
            } else {
                None
            }
        };

        render_link_label(sg, &edge.label, lx, ly, font_color, edge_angle, direction_inverted);
    }

    sg.push_raw("</g>");
}

/// Render a link label matching Java's StringWithArrow format.
/// Handles direction indicators (> / <) as triangular polygons and renders
/// text segments with font-size 13. Bold (**text**) gets separate <text> elements.
///
/// `edge_angle`: the radian angle of the edge path (from atan2(dx, dy) like Java).
/// `direction_inverted`: true when Java's Link.getInv() was called (UP/LEFT direction),
/// which flips the FORWARD/BACKWARD semantics of the label arrow indicator.
fn render_link_label(
    sg: &mut SvgGraphic,
    label: &str,
    label_x: f64,
    label_y: f64,
    font_color: &str,
    edge_angle: Option<f64>,
    direction_inverted: bool,
) {
    const LINK_FONT_SIZE: f64 = 13.0;

    // Parse direction indicator (> or <) from the label.
    // Java: StringWithArrow detects leading/trailing > or < characters.
    let trimmed = label.trim();
    let (has_indicator, mut is_backward, text) = if trimmed.starts_with("> ") || trimmed == ">" {
        (true, false, trimmed.strip_prefix("> ").or_else(|| trimmed.strip_prefix('>')).unwrap_or("").trim())
    } else if trimmed.starts_with("< ") || trimmed == "<" {
        (true, true, trimmed.strip_prefix("< ").or_else(|| trimmed.strip_prefix('<')).unwrap_or("").trim())
    } else if trimmed.ends_with(" >") {
        (true, false, trimmed.strip_suffix(" >").unwrap_or(trimmed).trim())
    } else if trimmed.ends_with(" <") {
        (true, true, trimmed.strip_suffix(" <").unwrap_or(trimmed).trim())
    } else {
        (false, false, trimmed)
    };

    // Java: when Link.getInv() was called (direction_inverted), getLinkArrow()
    // reverses the arrow: FORWARD↔BACKWARD.
    if direction_inverted {
        is_backward = !is_backward;
    }

    // Parse bold segments: **text** → bold, rest → normal
    let segments = parse_creole_bold_segments(text);

    // Compute text widths for positioning (using advance_text when available)
    let mut total_text_width = 0.0;
    for seg in &segments {
        let t = seg.advance_text.unwrap_or(seg.text);
        total_text_width += font_metrics::text_width(t, "SansSerif", LINK_FONT_SIZE, seg.bold, false);
    }

    // Direction indicator triangle width (Java TextBlockArrow2: size = font_size)
    let indicator_width = if has_indicator { LINK_FONT_SIZE } else { 0.0 };

    // label_x, label_y is the top-left of the label bounding box from Graphviz.
    // Java's StringWithArrow.addMagicArrow merges the arrow LEFT of the text with
    // vertical CENTER alignment.  The text is margin-wrapped (margin=1).
    // Merged height = max(arrow_h=13, text_h + 2*margin).
    // dy_text = (merged_h - text_marged_h) / 2.
    // Text baseline = label_y + dy_text + margin + ascent.
    let text_h = font_metrics::line_height("SansSerif", LINK_FONT_SIZE, false, false);
    let margin = 1.0;
    let text_marged_h = text_h + 2.0 * margin;
    let merged_h = text_marged_h.max(LINK_FONT_SIZE);
    let dy_text = (merged_h - text_marged_h) / 2.0;
    let text_ascent = font_metrics::ascent("SansSerif", LINK_FONT_SIZE, false, false);
    let text_y = label_y + dy_text + margin + text_ascent;

    // Render direction indicator triangle using TextBlockArrow2 algorithm.
    // Java TextBlockArrow2.drawU() draws a 3-point triangle rotated by the edge angle.
    if has_indicator {
        let mut angle = edge_angle.unwrap_or(0.0);
        if is_backward {
            angle += std::f64::consts::PI;
        }

        let tri_size = (LINK_FONT_SIZE * 0.80) as i32;
        let tri_size_f = tri_size as f64;
        // Java: addMagicArrow merges TextBlockArrow2 LEFT of the margin-wrapped text.
        // Arrow block is LINK_FONT_SIZE×LINK_FONT_SIZE (13×13).
        // Text block is margin-wrapped: height = line_height + 2*margin.
        // Merged height = max(arrow_h, text_marged_h).
        // dy_arrow = (merged_h - arrow_h) / 2.
        let text_h = font_metrics::line_height("SansSerif", LINK_FONT_SIZE, false, false);
        let margin = 1.0; // Java standard edge label margin
        let text_marged_h = text_h + 2.0 * margin;
        let outer_h = text_marged_h.max(LINK_FONT_SIZE);
        let dy_arrow = (outer_h - LINK_FONT_SIZE) / 2.0;

        // Java: UTranslate(triSize/2, size/2) — origin offset to center
        let cx = label_x + tri_size_f / 2.0;
        let cy = label_y + dy_arrow + LINK_FONT_SIZE / 2.0;
        let radius = tri_size_f / 2.0;
        let beta = std::f64::consts::PI * 4.0 / 5.0;

        let p0x = cx + radius * angle.sin();
        let p0y = cy + radius * angle.cos();
        let p1x = cx + radius * (angle + beta).sin();
        let p1y = cy + radius * (angle + beta).cos();
        let p2x = cx + radius * (angle - beta).sin();
        let p2y = cy + radius * (angle - beta).cos();

        sg.push_raw(&format!(
            "<polygon fill=\"#000000\" points=\"{},{},{},{},{},{},{},{}\" style=\"stroke:#000000;stroke-width:1;\"/>",
            fmt_coord(p0x), fmt_coord(p0y),
            fmt_coord(p1x), fmt_coord(p1y),
            fmt_coord(p2x), fmt_coord(p2y),
            fmt_coord(p0x), fmt_coord(p0y),
        ));
    }

    // Render text segments.
    // Java: the text block is margin-wrapped (TextBlockMarged, margin=1) before being
    // merged with the arrow.  The text inside starts at arrow_width + margin.
    let mut text_x = label_x + indicator_width + margin;
    for seg in &segments {
        let w = font_metrics::text_width(seg.text, "SansSerif", LINK_FONT_SIZE, seg.bold, false);
        // Java: trailing whitespace is trimmed from rendered text but the cursor
        // still advances by the full (untrimmed) width.
        let advance_w = if let Some(advance) = seg.advance_text {
            font_metrics::text_width(advance, "SansSerif", LINK_FONT_SIZE, seg.bold, false)
        } else {
            w
        };
        if !seg.text.is_empty() {
            let bold_attr = if seg.bold {
                r#" font-weight="700""#
            } else {
                ""
            };
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="{LINK_FONT_SIZE}"{bold_attr} lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(w),
                fmt_coord(text_x),
                fmt_coord(text_y),
                xml_escape(seg.text),
            ));
        }
        text_x += advance_w;
    }
}

/// A segment of text with optional bold formatting.
struct TextSegment<'a> {
    text: &'a str,
    bold: bool,
    /// When set, use this text for width/advance calculation instead of `text`.
    /// Java trims trailing whitespace from rendered text but advances the cursor
    /// by the full (untrimmed) width.
    advance_text: Option<&'a str>,
}

/// Parse Creole bold markers (**text**) into segments of normal and bold text.
fn parse_creole_bold_segments(text: &str) -> Vec<TextSegment<'_>> {
    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if let Some(bold_start) = remaining.find("**") {
            // Text before the bold marker.
            // Java StripeSimple trims trailing whitespace from the text atom
            // but advances the cursor by the full (untrimmed) width.
            // We strip trailing whitespace for rendering but store the full
            // width so the next segment is positioned correctly.
            let pre_raw = &remaining[..bold_start];
            let pre = pre_raw.trim_end();
            if !pre.is_empty() {
                segments.push(TextSegment {
                    text: pre,
                    bold: false,
                    advance_text: if pre.len() != pre_raw.len() { Some(pre_raw) } else { None },
                });
            } else if !pre_raw.is_empty() {
                // All whitespace: still advance cursor
                segments.push(TextSegment {
                    text: "",
                    bold: false,
                    advance_text: Some(pre_raw),
                });
            }
            let after_start = &remaining[bold_start + 2..];
            if let Some(bold_end) = after_start.find("**") {
                segments.push(TextSegment {
                    text: &after_start[..bold_end],
                    bold: true,
                    advance_text: None,
                });
                remaining = &after_start[bold_end + 2..];
            } else {
                // No closing **, treat rest as bold
                segments.push(TextSegment {
                    text: after_start,
                    bold: true,
                    advance_text: None,
                });
                remaining = "";
            }
        } else {
            segments.push(TextSegment {
                text: remaining,
                bold: false,
                advance_text: None,
            });
            remaining = "";
        }
    }

    segments
}

/// Parse the start and end coordinates from an SVG path d-string.
/// Returns ((start_x, start_y), (end_x, end_y)).
fn parse_path_start_end_simple(d: &str) -> Option<((f64, f64), (f64, f64))> {
    let d = d.trim();
    if !d.starts_with('M') {
        return None;
    }
    // Parse all numbers from the path
    let nums: Vec<f64> = d
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    if nums.len() < 4 {
        return None;
    }
    let sx = nums[0];
    let sy = nums[1];
    let ex = nums[nums.len() - 2];
    let ey = nums[nums.len() - 1];
    Some(((sx, sy), (ex, ey)))
}

fn adjust_path_endpoint(d: &str, decoration_len: f64) -> String {
    let parts: Vec<&str> = d.split_whitespace().collect();
    if parts.len() < 2 {
        return d.to_string();
    }

    let parse_pair = |s: &str| -> Option<(f64, f64)> {
        let mut it = s.split(',');
        Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
    };

    let Some((tx, ty)) = parse_pair(parts[parts.len() - 1]) else {
        return d.to_string();
    };
    let Some((fx, fy)) = parse_pair(parts[parts.len() - 2]) else {
        return d.to_string();
    };
    let dx = tx - fx;
    let dy = ty - fy;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 {
        return d.to_string();
    }

    let ux = dx / len;
    let uy = dy / len;
    let mut out: Vec<String> = parts.iter().map(|part| (*part).to_string()).collect();
    out[parts.len() - 2] = format!(
        "{},{}",
        fmt_coord(fx - ux * decoration_len),
        fmt_coord(fy - uy * decoration_len)
    );
    out[parts.len() - 1] = format!(
        "{},{}",
        fmt_coord(tx - ux * decoration_len),
        fmt_coord(ty - uy * decoration_len)
    );
    out.join(" ")
}

/// Shorten the START of a raw SVG path `d` attribute by `decoration_len` pixels.
/// Mirrors `adjust_path_endpoint` but operates on the first two coordinate tokens.
/// Used for reversed ("backto") links where the arrowhead is at the path start.
fn adjust_path_startpoint(d: &str, decoration_len: f64) -> String {
    let parts: Vec<&str> = d.split_whitespace().collect();
    if parts.len() < 2 {
        return d.to_string();
    }

    fn strip_cmd(s: &str) -> &str {
        if s.starts_with(|c: char| c.is_ascii_alphabetic()) {
            &s[1..]
        } else {
            s
        }
    }
    fn cmd_prefix(s: &str) -> &str {
        if s.starts_with(|c: char| c.is_ascii_alphabetic()) {
            &s[..1]
        } else {
            ""
        }
    }

    let parse_pair = |s: &str| -> Option<(f64, f64)> {
        let coords = strip_cmd(s);
        let mut it = coords.split(',');
        Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
    };

    // First token is the start point (M...), second is the first control point (C... or coords)
    let Some((sx, sy)) = parse_pair(parts[0]) else {
        return d.to_string();
    };
    let Some((cx, cy)) = parse_pair(parts[1]) else {
        return d.to_string();
    };

    // Direction from start towards first control point
    let dx = cx - sx;
    let dy = cy - sy;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 {
        return d.to_string();
    }

    let ux = dx / len;
    let uy = dy / len;
    let mut out: Vec<String> = parts.iter().map(|part| (*part).to_string()).collect();
    out[0] = format!(
        "{}{},{}",
        cmd_prefix(parts[0]),
        fmt_coord(sx + ux * decoration_len),
        fmt_coord(sy + uy * decoration_len)
    );
    out[1] = format!(
        "{}{},{}",
        cmd_prefix(parts[1]),
        fmt_coord(cx + ux * decoration_len),
        fmt_coord(cy + uy * decoration_len)
    );
    out.join(" ")
}

// ---------------------------------------------------------------------------
// Note rendering
// ---------------------------------------------------------------------------

fn render_note(
    sg: &mut SvgGraphic,
    note: &ComponentNoteLayout,
    bg: &str,
    border: &str,
    font_color: &str,
    ent_id: &str,
) {
    // Wrap note in <g class="entity"> like Java's EntityImageNote
    let source_line_attr = note
        .source_line
        .map_or(String::new(), |l| format!(r#" data-source-line="{}""#, l));
    sg.push_raw(&format!(
        r#"<g class="entity" data-qualified-name="{}"{} id="{}">"#,
        xml_escape(&note.qualified_name),
        source_line_attr,
        ent_id,
    ));

    let x = note.x;
    let y = note.y;
    let w = note.width;
    let h = note.height;
    let fold = 10.0; // Java: CORNER = 10

    // Java renders notes attached to entities using an "Opale" path style
    // with a connector ear pointing towards the target entity.
    let has_ear = note.ear_tip_y.is_some() && note.ear_tip_x.is_some();

    if has_ear {
        let ear_tip_y = note.ear_tip_y.unwrap();
        let ear_tip_x = note.ear_tip_x.unwrap();
        // Ear base: 8px wide centered on the ear_tip_x
        let ear_base_left = ear_tip_x - 4.0;
        let ear_base_right = ear_tip_x + 4.0;

        // Use fmt_coord for Java-matching coordinate formatting (4dp, strip trailing zeros)
        let fc = fmt_coord;

        // Build the Opale note path
        let mut d = String::new();
        if note.position == "top" {
            // Ear on bottom edge pointing down
            use std::fmt::Write;
            write!(d, "M{},{}", fc(x), fc(y)).unwrap();
            write!(d, " L{},{}", fc(x), fc(y + h)).unwrap();
            write!(d, " A0,0 0 0 0 {},{}", fc(x), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(ear_base_left), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(ear_tip_x), fc(ear_tip_y)).unwrap();
            write!(d, " L{},{}", fc(ear_base_right), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(x + w), fc(y + h)).unwrap();
            write!(d, " A0,0 0 0 0 {},{}", fc(x + w), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(x + w), fc(y + fold)).unwrap();
            write!(d, " L{},{}", fc(x + w - fold), fc(y)).unwrap();
            write!(d, " L{},{}", fc(x), fc(y)).unwrap();
            write!(d, " A0,0 0 0 0 {},{}", fc(x), fc(y)).unwrap();
        } else if note.position == "bottom" {
            // Ear on top edge pointing up
            use std::fmt::Write;
            write!(d, "M{},{}", fc(x), fc(y)).unwrap();
            write!(d, " L{},{}", fc(x), fc(y + h)).unwrap();
            write!(d, " A0,0 0 0 0 {},{}", fc(x), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(x + w), fc(y + h)).unwrap();
            write!(d, " A0,0 0 0 0 {},{}", fc(x + w), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(x + w), fc(y + fold)).unwrap();
            write!(d, " L{},{}", fc(x + w - fold), fc(y)).unwrap();
            write!(d, " L{},{}", fc(ear_base_right), fc(y)).unwrap();
            write!(d, " L{},{}", fc(ear_tip_x), fc(ear_tip_y)).unwrap();
            write!(d, " L{},{}", fc(ear_base_left), fc(y)).unwrap();
            write!(d, " L{},{}", fc(x), fc(y)).unwrap();
            write!(d, " A0,0 0 0 0 {},{}", fc(x), fc(y)).unwrap();
        } else {
            // Fallback for left/right: simple polygon path without ear
            use std::fmt::Write;
            write!(d, "M{},{}", fc(x), fc(y)).unwrap();
            write!(d, " L{},{}", fc(x), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(x + w), fc(y + h)).unwrap();
            write!(d, " L{},{}", fc(x + w), fc(y + fold)).unwrap();
            write!(d, " L{},{}", fc(x + w - fold), fc(y)).unwrap();
            write!(d, " Z").unwrap();
        }

        sg.set_fill_color(bg);
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(0.5, None);
        sg.push_raw(&format!(
            r#"<path d="{}" fill="{}" style="stroke:{};stroke-width:0.5;"/>"#,
            d, bg, border
        ));
    } else {
        // Simple polygon note (no attached target)
        sg.set_fill_color(bg);
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(1.0, None);
        sg.svg_polygon(
            0.0,
            &[
                x,
                y,
                x + w - fold,
                y,
                x + w,
                y + fold,
                x + w,
                y + h,
                x,
                y + h,
            ],
        );
    }

    // Corner fold
    sg.set_fill_color(bg);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} L{},{}" fill="{}" style="stroke:{};stroke-width:0.5;"/>"#,
        fmt_coord(x + w - fold),
        fmt_coord(y),
        fmt_coord(x + w - fold),
        fmt_coord(y + fold),
        fmt_coord(x + w),
        fmt_coord(y + fold),
        fmt_coord(x + w - fold),
        fmt_coord(y),
        bg,
        border,
    ));

    // Java EntityImageNote: marginX1=6, marginY=5.
    // Text baseline = note_y + marginY + ascent_13.
    // SansSerif 13pt: ascent = 1901/2048 * 13 = 12.069...
    const NOTE_MARGIN_X1: f64 = 6.0;
    const NOTE_MARGIN_Y: f64 = 5.0;
    const NOTE_FONT_SIZE: f64 = 13.0;
    const NOTE_ASCENT: f64 = 1901.0 / 2048.0 * 13.0; // 12.0669...
    const NOTE_LINE_HEIGHT: f64 = 15.1328; // (1901+483)/2048*13

    let text_x = x + NOTE_MARGIN_X1;
    let text_y = y + NOTE_MARGIN_Y + NOTE_ASCENT;
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        text_x,
        text_y,
        NOTE_LINE_HEIGHT,
        font_color,
        None,
        &format!(r#"font-size="{}""#, NOTE_FONT_SIZE as u32),
    );
    sg.push_raw(&tmp);

    // Close entity group
    sg.push_raw("</g>");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::component::{
        ComponentEdgeLayout, ComponentGroupLayout, ComponentLayout, ComponentNodeLayout,
        ComponentNoteLayout,
    };
    use crate::model::component::ComponentDiagram;
    use crate::style::SkinParams;

    fn empty_diagram() -> ComponentDiagram {
        ComponentDiagram {
            entities: vec![],
            links: vec![],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        }
    }

    fn empty_layout() -> ComponentLayout {
        ComponentLayout {
            width: 300.0,
            height: 200.0,
            nodes: vec![],
            edges: vec![],
            notes: vec![],
            groups: vec![],
        }
    }

    fn make_component(id: &str, x: f64, y: f64, w: f64, h: f64) -> ComponentNodeLayout {
        ComponentNodeLayout {
            id: id.to_string(),
            name: id.to_string(),
            kind: ComponentKind::Component,
            x,
            y,
            width: w,
            height: h,
            description: vec![],
            stereotype: None,
            color: None,
            source_line: None,
        }
    }

    // 1. Empty diagram renders valid SVG
    #[test]
    fn test_empty_diagram() {
        let diagram = empty_diagram();
        let layout = empty_layout();
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("<defs/>"), "must have empty defs");
    }

    // 2. Component node rendering
    #[test]
    fn test_component_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_component("comp1", 20.0, 30.0, 120.0, 40.0));
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("comp1"), "component name must appear");
        let rect_count = svg.matches("<rect").count();
        assert!(
            rect_count >= 3,
            "component must have at least 3 rects, got {}",
            rect_count
        );
    }

    // 3. Rectangle node rendering
    #[test]
    fn test_rectangle_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "rect1".to_string(),
            name: "MyRect".to_string(),
            kind: ComponentKind::Rectangle,
            x: 20.0,
            y: 30.0,
            width: 120.0,
            height: 40.0,
            description: vec![],
            stereotype: None,
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("MyRect"), "rectangle name must appear");
        assert!(svg.contains("<rect"), "must contain rect element");
    }

    // 4. Database node rendering
    #[test]
    fn test_database_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "db1".to_string(),
            name: "MyDB".to_string(),
            kind: ComponentKind::Database,
            x: 20.0,
            y: 30.0,
            width: 100.0,
            height: 60.0,
            description: vec![],
            stereotype: None,
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("MyDB"), "database name must appear");
        assert!(svg.contains("<path"), "database uses path for cylinder");
    }

    // 5. Cloud node rendering
    #[test]
    fn test_cloud_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "cloud1".to_string(),
            name: "MyCloud".to_string(),
            kind: ComponentKind::Cloud,
            x: 20.0,
            y: 30.0,
            width: 100.0,
            height: 60.0,
            description: vec![],
            stereotype: None,
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("MyCloud"), "cloud name must appear");
        assert!(
            svg.contains(r#"rx="20""#),
            "cloud should have large corner radius"
        );
    }

    // 6. Edge rendering with arrow
    #[test]
    fn test_edge_with_arrow() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ComponentEdgeLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(100.0, 50.0), (100.0, 120.0)],
            raw_path_d: None,
            label: String::new(),
            dashed: false,
            reversed_for_svg: false,
            label_xy: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("<polygon"),
            "edge must have inline polygon arrowhead"
        );
        assert!(
            svg.contains("stroke:#181818"),
            "edge must use EDGE_COLOR in style"
        );
    }

    // 7. Dashed edge
    #[test]
    fn test_dashed_edge() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ComponentEdgeLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(100.0, 50.0), (100.0, 120.0)],
            raw_path_d: None,
            label: String::new(),
            dashed: true,
            reversed_for_svg: false,
            label_xy: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("stroke-dasharray"),
            "dashed edge must have dasharray"
        );
    }

    // 8. Edge with label
    #[test]
    fn test_edge_with_label() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ComponentEdgeLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(80.0, 40.0), (80.0, 100.0)],
            raw_path_d: None,
            label: "uses".to_string(),
            dashed: false,
            reversed_for_svg: false,
            label_xy: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("uses"), "edge label must appear");
    }

    // 9. Note rendering
    #[test]
    fn test_note_rendering() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.notes.push(ComponentNoteLayout {
            x: 10.0,
            y: 20.0,
            width: 120.0,
            height: 40.0,
            text: "important note".to_string(),
            position: "top".to_string(),
            target: None,
            ear_tip_y: None,
            ear_tip_x: None,
            qualified_name: "GMN0".to_string(),
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains(r##"fill="#FEFFDD""##),
            "note must use default theme note background"
        );
        assert!(svg.contains("important note"), "note text must appear");
        assert!(svg.contains("<polygon"), "note body must be a polygon");
    }

    // 10. Multiline note
    #[test]
    fn test_multiline_note() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.notes.push(ComponentNoteLayout {
            x: 10.0,
            y: 20.0,
            width: 120.0,
            height: 60.0,
            text: "line one\nline two".to_string(),
            position: "bottom".to_string(),
            target: None,
            ear_tip_y: None,
            ear_tip_x: None,
            qualified_name: "GMN0".to_string(),
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(!svg.contains("<tspan"), "multiline note must not use tspan");
        assert!(svg.contains("line one"), "first line must appear");
        assert!(svg.contains("line two"), "second line must appear");
    }

    // 11. Group rendering
    #[test]
    fn test_group_rendering() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.groups.push(ComponentGroupLayout {
            id: "grp1".to_string(),
            name: "My Group".to_string(),
            kind: ComponentKind::Rectangle,
            x: 10.0,
            y: 10.0,
            width: 200.0,
            height: 150.0,
            source_line: None,
            stereotype: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("My Group"), "group name must appear");
        assert!(svg.contains("<rect"), "group must have rect background");
    }

    // 12. XML escaping
    #[test]
    fn test_xml_escaping() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "test".to_string(),
            name: "A & B < C".to_string(),
            kind: ComponentKind::Component,
            x: 10.0,
            y: 10.0,
            width: 120.0,
            height: 40.0,
            description: vec!["x > y".to_string()],
            stereotype: None,
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("A &amp; B &lt; C"), "name must be XML-escaped");
        assert!(svg.contains("x &gt; y"), "description must be XML-escaped");
    }

    // 13. Component with stereotype
    #[test]
    fn test_component_with_stereotype() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "test".to_string(),
            name: "MyComp".to_string(),
            kind: ComponentKind::Component,
            x: 10.0,
            y: 10.0,
            width: 120.0,
            height: 60.0,
            description: vec![],
            stereotype: Some("service".to_string()),
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("&#171;service&#187;"),
            "stereotype must appear with guillemets"
        );
        assert!(
            svg.contains("font-style=\"italic\""),
            "stereotype must be italic"
        );
    }

    // 14. Component with description
    #[test]
    fn test_component_with_description() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "test".to_string(),
            name: "MyComp".to_string(),
            kind: ComponentKind::Component,
            x: 10.0,
            y: 10.0,
            width: 120.0,
            height: 80.0,
            description: vec!["desc line 1".to_string(), "desc line 2".to_string()],
            stereotype: None,
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("desc line 1"),
            "description line 1 must appear"
        );
        assert!(
            svg.contains("desc line 2"),
            "description line 2 must appear"
        );
        assert!(
            svg.contains("<line"),
            "separator line between name and description"
        );
    }

    #[test]
    fn test_component_description_renders_creole_and_link() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "test".to_string(),
            name: "MyComp".to_string(),
            kind: ComponentKind::Component,
            x: 10.0,
            y: 10.0,
            width: 140.0,
            height: 90.0,
            description: vec!["**bold** [[https://example.com{hover} label]]".to_string()],
            stereotype: None,
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"font-weight="700""#));
        assert!(svg.contains(r#"href="https://example.com""#));
        assert!(svg.contains(r#"title="hover""#));
        assert!(svg.contains("label"));
    }

    // 15. Empty edges list
    #[test]
    fn test_no_edges() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout
            .nodes
            .push(make_component("A", 20.0, 20.0, 100.0, 40.0));
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            !svg.contains(r#"class="link""#),
            "no edges should mean no link groups"
        );
    }

    // 16. Full SVG structure
    #[test]
    fn test_full_svg_structure() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.width = 400.0;
        layout.height = 300.0;
        layout
            .nodes
            .push(make_component("A", 20.0, 20.0, 100.0, 40.0));
        layout
            .nodes
            .push(make_component("B", 20.0, 100.0, 100.0, 40.0));
        layout.edges.push(ComponentEdgeLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(70.0, 60.0), (70.0, 100.0)],
            raw_path_d: None,
            label: "uses".to_string(),
            dashed: false,
            reversed_for_svg: false,
            label_xy: None,
        });

        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");

        assert!(svg.starts_with("<svg"), "SVG must start with <svg");
        assert!(svg.contains("</svg>"), "SVG must end with </svg>");
        assert!(
            svg.contains("viewBox=\"0 0 400 300\""),
            "viewBox must match"
        );
        assert!(svg.contains("width=\"400px\""), "width must match");
        assert!(svg.contains("<defs/>"), "must have empty defs");
    }

    // 17. Interface node rendering
    #[test]
    fn test_interface_node() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.nodes.push(ComponentNodeLayout {
            id: "iface1".to_string(),
            name: "MyInterface".to_string(),
            kind: ComponentKind::Interface,
            x: 20.0,
            y: 30.0,
            width: 100.0,
            height: 50.0,
            description: vec![],
            stereotype: None,
            color: None,
            source_line: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("MyInterface"), "interface name must appear");
        assert!(svg.contains("<circle"), "interface uses circle icon");
    }

    // 18. Polyline edge (multiple points) - now uses <path>
    #[test]
    fn test_polyline_edge() {
        let diagram = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ComponentEdgeLayout {
            from: "A".to_string(),
            to: "B".to_string(),
            points: vec![(10.0, 10.0), (50.0, 50.0), (100.0, 50.0), (150.0, 100.0)],
            raw_path_d: None,
            label: String::new(),
            dashed: false,
            reversed_for_svg: false,
            label_xy: None,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<path"), "multi-point edge must use path");
    }
}
