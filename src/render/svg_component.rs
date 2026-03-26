use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, xml_escape, SvgGraphic, LengthAdjust};
use crate::layout::component::{
    ComponentEdgeLayout, ComponentGroupLayout, ComponentLayout, ComponentNodeLayout,
    ComponentNoteLayout,
};
use crate::model::component::{ComponentDiagram, ComponentKind};
use crate::render::svg::{write_svg_root_bg, write_bg_rect};
use crate::render::svg_richtext::render_creole_text;
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

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_component(
    cd: &ComponentDiagram,
    layout: &ComponentLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    reset_entity_counter();

    // Build entity ID map: entity name → "ent0002", "ent0003", etc.
    // Java assigns IDs in definition order (source_line).
    let mut entity_ids: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut entities_sorted: Vec<&crate::model::component::ComponentEntity> = cd.entities.iter().collect();
    entities_sorted.sort_by_key(|e| e.source_line.unwrap_or(usize::MAX));
    let mut ent_counter = 2u32;
    for ent in &entities_sorted {
        let ent_id = format!("ent{:04}", ent_counter);
        entity_ids.insert(ent.id.clone(), ent_id);
        ent_counter += 1;
    }

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
        render_group(&mut sg, group, group_bg, group_border, group_font);
    }

    // Nodes
    for node in &layout.nodes {
        render_node(
            &mut sg,
            node,
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

    // Edges — link IDs start after entity IDs
    let mut link_counter = ent_counter;
    for (ei, edge) in layout.edges.iter().enumerate() {
        let source_line = cd.links.get(ei).and_then(|l| {
            // Component links don't have source_line yet, derive from entity position
            None::<usize>
        });
        render_edge(&mut sg, edge, arrow_color, comp_font, &entity_ids, link_counter, source_line);
        link_counter += 1;
    }

    // Notes
    for note in &layout.notes {
        render_note(&mut sg, note, note_bg, note_border, note_font);
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
    _bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = group.x;
    let y = group.y;
    let w = group.width;
    let h = group.height;

    // HTML comment
    sg.push_raw(&format!("<!--cluster {}-->", xml_escape(&group.id)));

    // Allocate entity ID for this cluster
    let ent_num = ENTITY_COUNTER.with(|c| { let v = c.get(); c.set(v + 1); v });
    let ent_id = format!("ent{:04}", ent_num);

    // Open semantic <g> with Java-matching attributes
    let mut g_open = format!(
        r#"<g class="cluster" data-qualified-name="{}""#,
        xml_escape(&group.id)
    );
    if let Some(sl) = group.source_line {
        g_open.push_str(&format!(r#" data-source-line="{}""#, sl));
    }
    g_open.push_str(&format!(r#" id="{ent_id}">"#));
    sg.push_raw(&g_open);

    match group.kind {
        ComponentKind::Component => {
            // Component cluster: rect with component icon (two small rects)
            sg.set_fill_color("none"); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

            // Component icon on right side
            let icon_w: f64 = 15.0;
            let icon_h: f64 = 10.0;
            let icon_x = x + w - icon_w - 5.0 + 2.0;
            let icon_y1 = y + 5.0;
            sg.set_fill_color("none"); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(icon_x, icon_y1, icon_w, icon_h, 0.0, 0.0, 0.0);
            sg.set_fill_color("none"); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(icon_x - 2.0, icon_y1 + 2.0, 4.0, 2.0, 0.0, 0.0, 0.0);
            sg.set_fill_color("none"); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(icon_x - 2.0, icon_y1 + 6.0, 4.0, 2.0, 0.0, 0.0, 0.0);

            let tl = text_len(&group.name, 14.0, true);
            let text_x = x + (w - tl) / 2.0;
            let text_y = y + 25.9951;
            sg.set_fill_color(font_color);
            sg.svg_text(&group.name, text_x, text_y, Some("sans-serif"), 14.0, Some("bold"), None, None, tl, LengthAdjust::Spacing, None, 0, None);
        }
        ComponentKind::Frame => {
            // Frame: rect with rx/ry 2.5, path-based label tab
            sg.set_fill_color("none"); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
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
            sg.svg_text(&group.name, text_x, text_y, Some("sans-serif"), 14.0, Some("bold"), None, None, tl, LengthAdjust::Spacing, None, 0, None);
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
            sg.set_fill_color("none"); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[
                p_tl.0, p_tl.1,
                p_tlb.0, p_tlb.1,
                p_trb.0, p_trb.1,
                p_trb.0, p_tr.1,
                p_br.0, p_br.1,
                p_bl.0, p_bl.1,
            ]);

            sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
            sg.svg_line(p_br.0, p_tl.1, p_trb.0, p_tlb.1, 0.0);
            sg.svg_line(p_tl.0, p_tl.1, p_br.0, p_tl.1, 0.0);
            sg.svg_line(p_br.0, p_tl.1, p_br.0, p_br.1, 0.0);

            let tl = text_len(&group.name, 14.0, true);
            let text_x = x + (w - depth) / 2.0 - tl / 2.0;
            let text_y = y + depth + 15.9951;
            sg.set_fill_color(font_color);
            sg.svg_text(&group.name, text_x, text_y, Some("sans-serif"), 14.0, Some("bold"), None, None, tl, LengthAdjust::Spacing, None, 0, None);
        }
        _ => {
            // Default package/rectangle/card: simple rect
            sg.set_fill_color("none"); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
            sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

            let tl = text_len(&group.name, 14.0, true);
            let text_x = x + (w - tl) / 2.0;
            let text_y = y + 15.9951;
            sg.set_fill_color(font_color);
            sg.svg_text(&group.name, text_x, text_y, Some("sans-serif"), 14.0, Some("bold"), None, None, tl, LengthAdjust::Spacing, None, 0, None);
        }
    }

    sg.push_raw("</g>");
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
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
            render_component_node(sg, node, comp_bg, comp_border, comp_font);
        }
        ComponentKind::Rectangle => {
            render_rectangle_node(sg, node, rect_bg, rect_border, comp_font);
        }
        ComponentKind::Database => render_database_node(sg, node, db_bg, db_border, comp_font),
        ComponentKind::Cloud => render_cloud_node(sg, node, cloud_bg, cloud_border, comp_font),
        ComponentKind::Node => render_box_node(sg, node, node_bg, node_border, comp_font),
        ComponentKind::Package => render_box_node(sg, node, rect_bg, rect_border, comp_font),
        ComponentKind::Interface => {
            render_interface_node(sg, node, comp_bg, comp_border, comp_font);
        }
        ComponentKind::Card => render_rectangle_node(sg, node, rect_bg, rect_border, comp_font),
        ComponentKind::Artifact => {
            render_artifact_node(sg, node, artifact_bg, artifact_border, comp_font);
        }
        ComponentKind::Storage => {
            render_storage_node(sg, node, storage_bg, storage_border, comp_font);
        }
        ComponentKind::Folder => render_folder_node(sg, node, folder_bg, folder_border, comp_font),
        ComponentKind::Frame => render_frame_node(sg, node, frame_bg, frame_border, comp_font),
        ComponentKind::Agent => render_agent_node(sg, node, agent_bg, agent_border, comp_font),
        ComponentKind::Stack => render_stack_node(sg, node, stack_bg, stack_border, comp_font),
        ComponentKind::Queue => render_queue_node(sg, node, queue_bg, queue_border, comp_font),
        ComponentKind::PortIn | ComponentKind::PortOut => {
            render_port_node(sg, node, comp_bg, comp_border, comp_font);
        }
    }
}

thread_local! {
    /// Java entity counter — starts at 2 (first two IDs reserved for diagram-level)
    static ENTITY_COUNTER: std::cell::Cell<usize> = const { std::cell::Cell::new(2) };
}

/// Reset entity counter (call before rendering a new diagram)
fn reset_entity_counter() {
    ENTITY_COUNTER.with(|c| c.set(2));
}

/// Emit HTML comment + open `<g class="entity">` with Java-matching attributes.
fn open_entity_g(sg: &mut SvgGraphic, node: &ComponentNodeLayout) {
    let ent_num = ENTITY_COUNTER.with(|c| { let v = c.get(); c.set(v + 1); v });
    let ent_id = format!("ent{:04}", ent_num);
    sg.push_raw(&format!("<!--entity {}-->", xml_escape(&node.id)));
    let source_line = node.source_line.map_or(String::new(), |l| {
        format!(r#" data-source-line="{}""#, l)
    });
    // Java: data-qualified-name uses the entity's qualified ID (e.g. "c1.web_app"),
    // not the display text (which may contain Creole markup like <size:12>).
    sg.push_raw(&format!(
        r#"<g class="entity" data-qualified-name="{}"{source_line} id="{ent_id}">"#,
        xml_escape(&node.id),
    ));
}

/// Component: rounded rect with component icon (two small rects on right side)
fn render_component_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

    // Component icon on right side
    let icon_w: f64 = 15.0;
    let icon_h: f64 = 10.0;
    let icon_x = x + w - icon_w - 5.0;
    let icon_y1 = y + 5.0;
    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(icon_x, icon_y1, icon_w, icon_h, 0.0, 0.0, 0.0);
    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(icon_x - 2.0, icon_y1 + 2.0, 4.0, 2.0, 0.0, 0.0, 0.0);
    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(icon_x - 2.0, icon_y1 + 6.0, 4.0, 2.0, 0.0, 0.0, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Rectangle: simple rectangle
fn render_rectangle_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 2.5, 2.5, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Database: cylinder shape via cubic path curves
fn render_database_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

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

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Cloud: rounded rect with large radius
fn render_cloud_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 20.0, 20.0, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Generic box (used for Node, Package)
fn render_box_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    fill: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    sg.set_fill_color(fill); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 2.5, 2.5, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Interface: small circle with name below
fn render_interface_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    let cx = node.x + node.width / 2.0;
    let cy = node.y + 12.0;
    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_circle(cx, cy, 8.0, 0.0);

    let name_y = cy + 20.0;
    let tl = text_len(&node.name, 14.0, false);
    sg.set_fill_color(font_color);
    sg.svg_text(&node.name, cx - tl / 2.0, name_y, Some("sans-serif"), 14.0, None, None, None, tl, LengthAdjust::Spacing, None, 0, None);

    sg.push_raw("</g>");
}

/// Artifact: rect with folded-corner icon
fn render_artifact_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

    // Folded corner icon (small polygon at top right)
    let fold: f64 = 6.0;
    let ix = x + w - 17.0;
    let iy = y + 5.0;
    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_polygon(0.0, &[
        ix, iy,
        ix, iy + 14.0,
        ix + 12.0, iy + 14.0,
        ix + 12.0, iy + fold,
        ix + fold, iy,
    ]);

    sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_line(ix + fold, iy, ix + fold, iy + fold, 0.0);
    sg.svg_line(ix + 12.0, iy + fold, ix + fold, iy + fold, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Storage: rounded rect with large rx/ry
fn render_storage_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    let rx = 35.0_f64.min(node.width / 4.0);
    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, rx, rx, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Folder: path with tab, body rect, separator line
fn render_folder_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

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

    sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_line(x, y + tab_h, x + w, y + tab_h, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Frame: rect with label tab
fn render_frame_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    _font_color: &str,
) {
    open_entity_g(sg, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let tab_w = (w * 0.4).min(70.0);
    let tab_h = FONT_SIZE + 6.0;

    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, w, h, 2.5, 2.5, 0.0);

    sg.set_fill_color(border); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(x, y, tab_w, tab_h, 0.0, 0.0, 0.0);

    let label_cx = x + tab_w / 2.0;
    let label_cy = y + tab_h / 2.0 + FONT_SIZE * 0.35;
    let tl = text_len(&node.name, FONT_SIZE - 1.0, true);
    sg.set_fill_color("#FFFFFF");
    sg.svg_text(&node.name, label_cx, label_cy, Some("sans-serif"), FONT_SIZE - 1.0, Some("700"), None, None, tl, LengthAdjust::Spacing, None, 0, Some("middle"));

    sg.push_raw("</g>");
}

/// Agent: rounded rect with rx 2.5
fn render_agent_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(node.x, node.y, node.width, node.height, 2.5, 2.5, 0.0);

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Stack: rect with frame path
fn render_stack_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    // Main body rect (stroke:none)
    sg.set_fill_color(bg); sg.set_stroke_color(Some("none")); sg.set_stroke_width(0.5, None);
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

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Queue: path body with double-curved right edge
fn render_queue_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

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

    render_node_text(sg, node, font_color);
    sg.push_raw("</g>");
}

/// Port: small 12x12 square with text label
fn render_port_node(
    sg: &mut SvgGraphic,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(sg, node);

    let port_size: f64 = 12.0;
    let cx = node.x + node.width / 2.0;

    // Text label (centered below/above the port square)
    let tl = text_len(&node.name, FONT_SIZE, false);
    let text_x = cx - tl / 2.0;
    let text_y = node.y + port_size + LINE_HEIGHT + 2.0;
    sg.set_fill_color(font_color);
    sg.svg_text(&node.name, text_x, text_y, Some("sans-serif"), FONT_SIZE, None, None, None, tl, LengthAdjust::Spacing, None, 0, None);

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
fn render_node_text(sg: &mut SvgGraphic, node: &ComponentNodeLayout, font_color: &str) {
    let cx = node.x + node.width / 2.0;
    let has_desc = !node.description.is_empty();

    // Stereotype
    let mut y_offset = 0.0;
    if let Some(ref stereotype) = node.stereotype {
        let stereo_text = format!("\u{00AB}{stereotype}\u{00BB}");
        let sy = node.y + FONT_SIZE + 4.0;
        let tl = font_metrics::text_width(&stereo_text, "sans-serif", FONT_SIZE - 2.0, false, true);
        sg.set_fill_color(font_color);
        sg.svg_text(&stereo_text, cx - tl / 2.0, sy, Some("sans-serif"), FONT_SIZE - 2.0, None, Some("italic"), None, tl, LengthAdjust::Spacing, None, 0, None);
        y_offset = LINE_HEIGHT;
    }

    // Name — Java: left-aligned, regular weight, x = rect_x + PADDING
    let name_y = if has_desc {
        node.y + FONT_SIZE + 4.0 + y_offset
    } else {
        // Java: baseline = rect_y + 20 + ascent (15 padding + 5 component internal margin)
        node.y + 20.0 + font_metrics::ascent("SansSerif", FONT_SIZE, false, false)
    };
    let name_x = node.x + 15.0;
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
        sg.set_stroke_color(Some(BORDER_COLOR)); sg.set_stroke_width(1.0, None);
        sg.svg_line(node.x, sep_y, node.x + node.width, sep_y, 0.0);

        let text_x = node.x + 8.0;
        let desc_text = node.description.join("\n");
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &desc_text,
            text_x,
            sep_y + LINE_HEIGHT,
            LINE_HEIGHT,
            font_color,
            None,
            r#"font-size="12""#,
        );
        sg.push_raw(&tmp);
    }
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
) {
    if edge.points.is_empty() {
        return;
    }

    // HTML comment
    sg.push_raw(&format!(
        "<!--link {} to {}-->",
        xml_escape(&edge.from),
        xml_escape(&edge.to),
    ));

    // Semantic group with data attributes matching Java format
    let from_ent = entity_ids.get(&edge.from).map(|s| s.as_str()).unwrap_or("");
    let to_ent = entity_ids.get(&edge.to).map(|s| s.as_str()).unwrap_or("");
    let link_type = if edge.dashed { "dependency" } else { "dependency" };
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

    // Build SVG path data using cubic bezier curves
    let mut d = String::new();
    let pts = &edge.points;
    if !pts.is_empty() {
        write!(d, "M{},{} ", fmt_coord(pts[0].0), fmt_coord(pts[0].1)).unwrap();
        // Points come in groups of 3 for cubic bezier (C command)
        let mut i = 1;
        while i + 2 < pts.len() {
            write!(
                d,
                "C{},{} {},{} {},{} ",
                fmt_coord(pts[i].0), fmt_coord(pts[i].1),
                fmt_coord(pts[i + 1].0), fmt_coord(pts[i + 1].1),
                fmt_coord(pts[i + 2].0), fmt_coord(pts[i + 2].1),
            )
            .unwrap();
            i += 3;
        }
        // Remaining points as line segments
        while i < pts.len() {
            write!(d, "L{},{} ", fmt_coord(pts[i].0), fmt_coord(pts[i].1)).unwrap();
            i += 1;
        }
    }

    let path_id = format!("{}-to-{}", xml_escape(&edge.from), xml_escape(&edge.to));
    sg.push_raw(&format!(
        r#"<path d="{d}" fill="none" id="{path_id}" style="stroke:{arrow_color};stroke-width:1;{dash_style}"/>"#,
    ));

    // Arrowhead polygon at the last point
    if pts.len() >= 2 {
        let (tx, ty) = pts[pts.len() - 1];
        let (fx, fy) = pts[pts.len() - 2];
        let dx = tx - fx;
        let dy = ty - fy;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            let ux = dx / len;
            let uy = dy / len;
            let nx = -uy;
            let ny = ux;
            // Java arrowhead: tip at (tx,ty), two wings 9px back, 4px wide
            let p1x = tx - ux * 9.0 + nx * 4.0;
            let p1y = ty - uy * 9.0 + ny * 4.0;
            let p2x = tx;
            let p2y = ty;
            let p3x = tx - ux * 9.0 - nx * 4.0;
            let p3y = ty - uy * 9.0 - ny * 4.0;

            sg.set_fill_color(arrow_color);
            sg.set_stroke_color(Some(arrow_color));
            sg.set_stroke_width(1.0, None);
            sg.svg_polygon(0.0, &[p1x, p1y, p2x, p2y, p3x, p3y, p1x, p1y]);
        }
    }

    // Label at midpoint
    if !edge.label.is_empty() {
        let mid = pts.len() / 2;
        let (mx, my) = if pts.len() == 2 {
            let (x1, y1) = pts[0];
            let (x2, y2) = pts[1];
            ((x1 + x2) / 2.0, (y1 + y2) / 2.0 - 6.0)
        } else {
            pts[mid]
        };

        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &edge.label,
            mx,
            my,
            LINE_HEIGHT,
            font_color,
            Some("middle"),
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
        sg.push_raw(&tmp);
    }

    sg.push_raw("</g>");
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
) {
    let x = note.x;
    let y = note.y;
    let w = note.width;
    let h = note.height;
    let fold = 8.0;

    sg.set_fill_color(bg); sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
    sg.svg_polygon(0.0, &[x, y, x + w - fold, y, x + w, y + fold, x + w, y + h, x, y + h]);

    sg.set_stroke_color(Some(border)); sg.set_stroke_width(1.0, None);
    sg.svg_line(x + w - fold, y, x + w - fold, y + fold, 0.0);
    sg.svg_line(x + w - fold, y + fold, x + w, y + fold, 0.0);

    let text_x = x + 6.0;
    let text_y = y + fold + FONT_SIZE;
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        text_x,
        text_y,
        LINE_HEIGHT,
        font_color,
        None,
        r#"font-size="13""#,
    );
    sg.push_raw(&tmp);
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
            color: None, source_line: None,
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
            color: None, source_line: None,
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
            color: None, source_line: None,
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
            color: None, source_line: None,
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
            label: String::new(),
            dashed: false,
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
            label: String::new(),
            dashed: true,
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
            label: "uses".to_string(),
            dashed: false,
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
            color: None, source_line: None,
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
            color: None, source_line: None,
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
            color: None, source_line: None,
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
            color: None, source_line: None,
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
            label: "uses".to_string(),
            dashed: false,
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
            color: None, source_line: None,
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
            label: String::new(),
            dashed: false,
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains("<path"), "multi-point edge must use path");
    }
}
