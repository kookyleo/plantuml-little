use std::fmt::Write;

use crate::font_metrics;
use crate::layout::component::{
    ComponentEdgeLayout, ComponentGroupLayout, ComponentLayout, ComponentNodeLayout,
    ComponentNoteLayout,
};
use crate::model::component::{ComponentDiagram, ComponentKind};
use crate::render::svg::fmt_coord;
use crate::render::svg::write_svg_root;
use crate::render::svg::xml_escape;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants (PlantUML defaults)
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
const LINE_HEIGHT: f64 = 16.0;
const COMPONENT_BG: &str = "#F1F1F1";
const COMPONENT_BORDER: &str = "#181818";
const RECT_BG: &str = "#F1F1F1";
const RECT_BORDER: &str = "#181818";
const NODE_BG: &str = "#F1F1F1";
const NODE_BORDER: &str = "#181818";
const DATABASE_BG: &str = "#F1F1F1";
const DATABASE_BORDER: &str = "#181818";
const CLOUD_BG: &str = "#F1F1F1";
const CLOUD_BORDER: &str = "#181818";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const GROUP_BG: &str = "#FFFFFF";
const GROUP_BORDER: &str = "#181818";
// Deployment diagram element colors
const ARTIFACT_BG: &str = "#F1F1F1";
const ARTIFACT_BORDER: &str = "#181818";
const STORAGE_BG: &str = "#F1F1F1";
const STORAGE_BORDER: &str = "#181818";
const FOLDER_BG: &str = "#F1F1F1";
const FOLDER_BORDER: &str = "#181818";
const FRAME_BG: &str = "#FFFFFF";
const FRAME_BORDER: &str = "#181818";
const AGENT_BG: &str = "#F1F1F1";
const AGENT_BORDER: &str = "#181818";
const STACK_BG: &str = "#F1F1F1";
const STACK_BORDER: &str = "#181818";
const QUEUE_BG: &str = "#F1F1F1";
const QUEUE_BORDER: &str = "#181818";

/// Compute the `textLength` attribute value for a text string at the given
/// font-size using the font-metrics table.
fn text_len(text: &str, size: f64, bold: bool) -> String {
    let w = font_metrics::text_width(text, "sans-serif", size, bold, false);
    fmt_coord(w)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_component(
    _cd: &ComponentDiagram,
    layout: &ComponentLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // Skin color lookups
    let comp_bg = skin.background_color("component", COMPONENT_BG);
    let comp_border = skin.border_color("component", COMPONENT_BORDER);
    let comp_font = skin.font_color("component", TEXT_FILL);
    let rect_bg = skin.background_color("rectangle", RECT_BG);
    let rect_border = skin.border_color("rectangle", RECT_BORDER);
    let db_bg = skin.background_color("database", DATABASE_BG);
    let db_border = skin.border_color("database", DATABASE_BORDER);
    let cloud_bg = skin.background_color("cloud", CLOUD_BG);
    let cloud_border = skin.border_color("cloud", CLOUD_BORDER);
    let node_bg = skin.background_color("node", NODE_BG);
    let node_border = skin.border_color("node", NODE_BORDER);
    let note_bg = skin.background_color("note", NOTE_BG);
    let note_border = skin.border_color("note", NOTE_BORDER);
    let note_font = skin.font_color("note", TEXT_FILL);
    let group_bg = skin.background_color("package", GROUP_BG);
    let group_border = skin.border_color("package", GROUP_BORDER);
    let group_font = skin.font_color("package", TEXT_FILL);
    let arrow_color = skin.arrow_color(EDGE_COLOR);
    // Deployment diagram skin lookups
    let artifact_bg = skin.background_color("artifact", ARTIFACT_BG);
    let artifact_border = skin.border_color("artifact", ARTIFACT_BORDER);
    let storage_bg = skin.background_color("storage", STORAGE_BG);
    let storage_border = skin.border_color("storage", STORAGE_BORDER);
    let folder_bg = skin.background_color("folder", FOLDER_BG);
    let folder_border = skin.border_color("folder", FOLDER_BORDER);
    let frame_bg = skin.background_color("frame", FRAME_BG);
    let frame_border = skin.border_color("frame", FRAME_BORDER);
    let agent_bg = skin.background_color("agent", AGENT_BG);
    let agent_border = skin.border_color("agent", AGENT_BORDER);
    let stack_bg = skin.background_color("stack", STACK_BG);
    let stack_border = skin.border_color("stack", STACK_BORDER);
    let queue_bg = skin.background_color("queue", QUEUE_BG);
    let queue_border = skin.border_color("queue", QUEUE_BORDER);

    // SVG header
    write_svg_root(&mut buf, layout.width, layout.height, "DESCRIPTION");

    // Empty defs to match Java PlantUML
    buf.push_str("<defs/>");
    buf.push_str("<g>");

    // Groups (render before nodes so they appear behind)
    for group in &layout.groups {
        render_group(&mut buf, group, group_bg, group_border, group_font);
    }

    // Nodes
    for node in &layout.nodes {
        render_node(
            &mut buf,
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

    // Edges
    for edge in &layout.edges {
        render_edge(&mut buf, edge, arrow_color, comp_font);
    }

    // Notes
    for note in &layout.notes {
        render_note(&mut buf, note, note_bg, note_border, note_font);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Group rendering (cluster)
// ---------------------------------------------------------------------------

fn render_group(
    buf: &mut String,
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
    write!(buf, "<!--cluster {}-->", xml_escape(&group.id)).unwrap();

    // Open semantic <g>
    write!(buf, r#"<g class="cluster" id="{}">"#, xml_escape(&group.id),).unwrap();

    match group.kind {
        ComponentKind::Frame => {
            // Frame: rect with rx/ry 2.5, path-based label tab
            write!(
                buf,
                r#"<rect fill="none" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
            )
            .unwrap();

            let name_escaped = xml_escape(&group.name);
            let tl = text_len(&group.name, 14.0, true);
            let tl_f: f64 = tl.parse().unwrap_or(40.0);
            let tab_w = tl_f + 9.7041;
            let tab_h = 19.2969;
            let tab_x2 = x + tab_w;
            let tab_y2 = y + tab_h;
            write!(
                buf,
                r#"<path d="M{},{} L{},{} L{},{} L{},{} " fill="none" style="stroke:{border};stroke-width:1;"/>"#,
                fmt_coord(tab_x2), fmt_coord(y),
                fmt_coord(tab_x2), fmt_coord(tab_y2 - 10.0),
                fmt_coord(tab_x2 - 10.0), fmt_coord(tab_y2),
                fmt_coord(x), fmt_coord(tab_y2),
            )
            .unwrap();

            let text_x = x + 3.0;
            let text_y = y + 13.9951;
            write!(
                buf,
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{tl}" x="{}" y="{}">{name_escaped}</text>"#,
                fmt_coord(text_x), fmt_coord(text_y),
            )
            .unwrap();
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
            write!(
                buf,
                r#"<polygon fill="none" points="{},{},{},{},{},{},{},{},{},{},{},{}" style="stroke:{border};stroke-width:1;"/>"#,
                fmt_coord(p_tl.0), fmt_coord(p_tl.1),
                fmt_coord(p_tlb.0), fmt_coord(p_tlb.1),
                fmt_coord(p_trb.0), fmt_coord(p_trb.1),
                fmt_coord(p_trb.0), fmt_coord(p_tr.1),
                fmt_coord(p_br.0), fmt_coord(p_br.1),
                fmt_coord(p_bl.0), fmt_coord(p_bl.1),
            )
            .unwrap();

            write!(
                buf,
                r#"<line style="stroke:{border};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(p_br.0), fmt_coord(p_trb.0),
                fmt_coord(p_tl.1), fmt_coord(p_tlb.1),
            )
            .unwrap();
            write!(
                buf,
                r#"<line style="stroke:{border};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(p_tl.0), fmt_coord(p_br.0),
                fmt_coord(p_tl.1), fmt_coord(p_tl.1),
            )
            .unwrap();
            write!(
                buf,
                r#"<line style="stroke:{border};stroke-width:1;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(p_br.0), fmt_coord(p_br.0),
                fmt_coord(p_tl.1), fmt_coord(p_br.1),
            )
            .unwrap();

            let name_escaped = xml_escape(&group.name);
            let tl = text_len(&group.name, 14.0, true);
            let tl_f: f64 = tl.parse().unwrap_or(40.0);
            let text_x = x + (w - depth) / 2.0 - tl_f / 2.0;
            let text_y = y + depth + 15.9951;
            write!(
                buf,
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{tl}" x="{}" y="{}">{name_escaped}</text>"#,
                fmt_coord(text_x), fmt_coord(text_y),
            )
            .unwrap();
        }
        _ => {
            // Default package/rectangle: simple rect
            write!(
                buf,
                r#"<rect fill="none" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:1;" width="{}" x="{}" y="{}"/>"#,
                fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
            )
            .unwrap();

            let name_escaped = xml_escape(&group.name);
            let tl = text_len(&group.name, 14.0, true);
            let tl_f: f64 = tl.parse().unwrap_or(40.0);
            let text_x = x + (w - tl_f) / 2.0;
            let text_y = y + 15.9951;
            write!(
                buf,
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{tl}" x="{}" y="{}">{name_escaped}</text>"#,
                fmt_coord(text_x), fmt_coord(text_y),
            )
            .unwrap();
        }
    }

    buf.push_str("</g>");
}

// ---------------------------------------------------------------------------
// Node rendering
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_node(
    buf: &mut String,
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
            render_component_node(buf, node, comp_bg, comp_border, comp_font);
        }
        ComponentKind::Rectangle => {
            render_rectangle_node(buf, node, rect_bg, rect_border, comp_font);
        }
        ComponentKind::Database => render_database_node(buf, node, db_bg, db_border, comp_font),
        ComponentKind::Cloud => render_cloud_node(buf, node, cloud_bg, cloud_border, comp_font),
        ComponentKind::Node => render_box_node(buf, node, node_bg, node_border, comp_font),
        ComponentKind::Package => render_box_node(buf, node, rect_bg, rect_border, comp_font),
        ComponentKind::Interface => {
            render_interface_node(buf, node, comp_bg, comp_border, comp_font);
        }
        ComponentKind::Card => render_rectangle_node(buf, node, rect_bg, rect_border, comp_font),
        ComponentKind::Artifact => {
            render_artifact_node(buf, node, artifact_bg, artifact_border, comp_font);
        }
        ComponentKind::Storage => {
            render_storage_node(buf, node, storage_bg, storage_border, comp_font);
        }
        ComponentKind::Folder => render_folder_node(buf, node, folder_bg, folder_border, comp_font),
        ComponentKind::Frame => render_frame_node(buf, node, frame_bg, frame_border, comp_font),
        ComponentKind::Agent => render_agent_node(buf, node, agent_bg, agent_border, comp_font),
        ComponentKind::Stack => render_stack_node(buf, node, stack_bg, stack_border, comp_font),
        ComponentKind::Queue => render_queue_node(buf, node, queue_bg, queue_border, comp_font),
    }
}

/// Emit HTML comment + open `<g class="entity">` for a node.
fn open_entity_g(buf: &mut String, node: &ComponentNodeLayout) {
    write!(buf, "<!--entity {}-->", xml_escape(&node.id)).unwrap();
    write!(buf, r#"<g class="entity" id="{}">"#, xml_escape(&node.id),).unwrap();
}

/// Component: rounded rect with component icon (two small rects on right side)
fn render_component_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
    )
    .unwrap();

    // Component icon on right side
    let icon_w: f64 = 15.0;
    let icon_h: f64 = 10.0;
    let icon_x = x + w - 5.0;
    let icon_y1 = y + 5.0;
    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(icon_h), fmt_coord(icon_w), fmt_coord(icon_x), fmt_coord(icon_y1),
    )
    .unwrap();
    write!(
        buf,
        r#"<rect fill="{bg}" height="2" style="stroke:{border};stroke-width:0.5;" width="4" x="{}" y="{}"/>"#,
        fmt_coord(icon_x - 2.0), fmt_coord(icon_y1 + 2.0),
    )
    .unwrap();
    write!(
        buf,
        r#"<rect fill="{bg}" height="2" style="stroke:{border};stroke-width:0.5;" width="4" x="{}" y="{}"/>"#,
        fmt_coord(icon_x - 2.0), fmt_coord(icon_y1 + 6.0),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Rectangle: simple rectangle
fn render_rectangle_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Database: cylinder shape via cubic path curves
fn render_database_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let ry: f64 = 10.0;
    let cx = x + w / 2.0;

    // Body
    write!(
        buf,
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
    )
    .unwrap();

    // Top ellipse
    write!(
        buf,
        r#"<path d="M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} " fill="none" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(x), fmt_coord(y + ry),
        fmt_coord(x), fmt_coord(y + ry + ry),
        fmt_coord(cx), fmt_coord(y + ry + ry),
        fmt_coord(cx), fmt_coord(y + ry + ry),
        fmt_coord(cx), fmt_coord(y + ry + ry),
        fmt_coord(x + w), fmt_coord(y + ry + ry),
        fmt_coord(x + w), fmt_coord(y + ry),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Cloud: rounded rect with large radius
fn render_cloud_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="20" ry="20" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Generic box (used for Node, Package)
fn render_box_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    fill: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    write!(
        buf,
        r#"<rect fill="{fill}" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Interface: small circle with name below
fn render_interface_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let cx = node.x + node.width / 2.0;
    let cy = node.y + 12.0;
    write!(
        buf,
        r#"<circle cx="{}" cy="{}" fill="{bg}" r="8" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(cx),
        fmt_coord(cy),
    )
    .unwrap();

    let name_escaped = xml_escape(&node.name);
    let name_y = cy + 20.0;
    let tl = text_len(&node.name, 14.0, false);
    let tl_f: f64 = tl.parse().unwrap_or(0.0);
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" lengthAdjust="spacing" textLength="{tl}" x="{}" y="{}">{name_escaped}</text>"#,
        fmt_coord(cx - tl_f / 2.0), fmt_coord(name_y),
    )
    .unwrap();

    buf.push_str("</g>");
}

/// Artifact: rect with folded-corner icon
fn render_artifact_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
    )
    .unwrap();

    // Folded corner icon (small polygon at top right)
    let fold: f64 = 6.0;
    let ix = x + w - 17.0;
    let iy = y + 5.0;
    write!(
        buf,
        r#"<polygon fill="{bg}" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(ix), fmt_coord(iy),
        fmt_coord(ix), fmt_coord(iy + 14.0),
        fmt_coord(ix + 12.0), fmt_coord(iy + 14.0),
        fmt_coord(ix + 12.0), fmt_coord(iy + fold),
        fmt_coord(ix + fold), fmt_coord(iy),
    )
    .unwrap();

    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(ix + fold),
        fmt_coord(ix + fold),
        fmt_coord(iy),
        fmt_coord(iy + fold),
    )
    .unwrap();
    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(ix + 12.0),
        fmt_coord(ix + fold),
        fmt_coord(iy + fold),
        fmt_coord(iy + fold),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Storage: rounded rect with large rx/ry
fn render_storage_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let rx = 35.0_f64.min(node.width / 4.0);
    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="{}" ry="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(rx), fmt_coord(rx),
        fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Folder: path with tab, body rect, separator line
fn render_folder_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let tab_w = 41.0_f64.min(w * 0.4);
    let tab_h: f64 = 21.0;
    let r: f64 = 2.5;

    write!(
        buf,
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
    )
    .unwrap();

    write!(
        buf,
        r#"<line style="stroke:{border};stroke-width:0.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(x),
        fmt_coord(x + w),
        fmt_coord(y + tab_h),
        fmt_coord(y + tab_h),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Frame: rect with label tab
fn render_frame_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    _font_color: &str,
) {
    open_entity_g(buf, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let tab_w = (w * 0.4).min(70.0);
    let tab_h = FONT_SIZE + 6.0;

    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
    )
    .unwrap();

    write!(
        buf,
        r#"<rect fill="{border}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(tab_h), fmt_coord(tab_w), fmt_coord(x), fmt_coord(y),
    )
    .unwrap();

    let name_escaped = xml_escape(&node.name);
    let label_cx = x + tab_w / 2.0;
    let label_cy = y + tab_h / 2.0 + FONT_SIZE * 0.35;
    let tl = text_len(&node.name, FONT_SIZE - 1.0, true);
    write!(
        buf,
        "<text fill=\"#FFFFFF\" font-family=\"sans-serif\" font-size=\"{}\" font-weight=\"700\" lengthAdjust=\"spacing\" text-anchor=\"middle\" textLength=\"{tl}\" x=\"{}\" y=\"{}\">{name_escaped}</text>",
        fmt_coord(FONT_SIZE - 1.0),
        fmt_coord(label_cx),
        fmt_coord(label_cy),
    )
    .unwrap();

    buf.push_str("</g>");
}

/// Agent: rounded rect with rx 2.5
fn render_agent_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="2.5" ry="2.5" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(node.height), fmt_coord(node.width), fmt_coord(node.x), fmt_coord(node.y),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Stack: rect with frame path
fn render_stack_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    // Main body rect (stroke:none)
    write!(
        buf,
        r#"<rect fill="{bg}" height="{}" rx="2.5" ry="2.5" style="stroke:none;stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
    )
    .unwrap();

    // Frame path
    let bar_left = x - 15.0;
    write!(
        buf,
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
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Queue: path body with double-curved right edge
fn render_queue_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    open_entity_g(buf, node);

    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let cap: f64 = 5.0;
    let mid_y = y + h / 2.0;

    // Left side curve (filled)
    write!(
        buf,
        r#"<path d="M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} " fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(x + cap), fmt_coord(y),
        fmt_coord(x + cap + cap), fmt_coord(y),
        fmt_coord(x + cap + cap), fmt_coord(mid_y),
        fmt_coord(x + cap + cap), fmt_coord(mid_y),
        fmt_coord(x + cap + cap), fmt_coord(mid_y),
        fmt_coord(x + cap + cap), fmt_coord(y + h),
        fmt_coord(x + cap), fmt_coord(y + h),
    )
    .unwrap();

    // Right endcap (open)
    write!(
        buf,
        r#"<path d="M{},{} C{},{} {},{} {},{} C{},{} {},{} {},{} " fill="none" style="stroke:{border};stroke-width:0.5;"/>"#,
        fmt_coord(x + w - cap), fmt_coord(y),
        fmt_coord(x + w - cap - cap), fmt_coord(y),
        fmt_coord(x + w - cap - cap), fmt_coord(mid_y),
        fmt_coord(x + w - cap - cap), fmt_coord(mid_y),
        fmt_coord(x + w - cap - cap), fmt_coord(mid_y),
        fmt_coord(x + w - cap - cap), fmt_coord(y + h),
        fmt_coord(x + w - cap), fmt_coord(y + h),
    )
    .unwrap();

    render_node_text(buf, node, font_color);
    buf.push_str("</g>");
}

/// Render name, stereotype, and description text for a node
fn render_node_text(buf: &mut String, node: &ComponentNodeLayout, font_color: &str) {
    let cx = node.x + node.width / 2.0;
    let has_desc = !node.description.is_empty();

    // Stereotype
    let mut y_offset = 0.0;
    if let Some(ref stereotype) = node.stereotype {
        let stereo_text = format!("\u{00AB}{stereotype}\u{00BB}");
        let escaped = xml_escape(&stereo_text);
        let sy = node.y + FONT_SIZE + 4.0;
        let tl = font_metrics::text_width(&stereo_text, "sans-serif", FONT_SIZE - 2.0, false, true);
        write!(
            buf,
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{fs:.0}" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(tl),
            fmt_coord(cx - tl / 2.0),
            fmt_coord(sy),
            fs = FONT_SIZE - 2.0,
        )
        .unwrap();
        y_offset = LINE_HEIGHT;
    }

    // Name
    let name_y = if has_desc {
        node.y + FONT_SIZE + 4.0 + y_offset
    } else {
        node.y + node.height / 2.0 + FONT_SIZE * 0.35 + y_offset
    };
    render_creole_text(
        buf,
        &node.name,
        cx,
        name_y,
        LINE_HEIGHT,
        font_color,
        Some("middle"),
        r#"font-size="14" font-weight="bold""#,
    );

    // Description
    if has_desc {
        let sep_y = name_y + 6.0;
        write!(
            buf,
            r#"<line style="stroke:{COMPONENT_BORDER};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(node.x),
            fmt_coord(node.x + node.width),
            fmt_coord(sep_y),
            fmt_coord(sep_y),
        )
        .unwrap();

        let text_x = node.x + 8.0;
        let desc_text = node.description.join("\n");
        render_creole_text(
            buf,
            &desc_text,
            text_x,
            sep_y + LINE_HEIGHT,
            LINE_HEIGHT,
            font_color,
            None,
            r#"font-size="12""#,
        );
    }
}

// ---------------------------------------------------------------------------
// Edge rendering
// ---------------------------------------------------------------------------

fn render_edge(buf: &mut String, edge: &ComponentEdgeLayout, arrow_color: &str, font_color: &str) {
    if edge.points.is_empty() {
        return;
    }

    // HTML comment + semantic group
    write!(
        buf,
        "<!--link {} to {}-->",
        xml_escape(&edge.from),
        xml_escape(&edge.to),
    )
    .unwrap();
    write!(
        buf,
        r#"<g class="link" id="{}-to-{}">"#,
        xml_escape(&edge.from),
        xml_escape(&edge.to),
    )
    .unwrap();

    let dash = if edge.dashed {
        r#" stroke-dasharray="7,5""#
    } else {
        ""
    };

    // Build path data
    let mut d = String::new();
    for (i, (px, py)) in edge.points.iter().enumerate() {
        if i == 0 {
            write!(d, "M{},{} ", fmt_coord(*px), fmt_coord(*py)).unwrap();
        } else {
            write!(
                d,
                "C{},{} {},{} {},{} ",
                fmt_coord(*px),
                fmt_coord(*py),
                fmt_coord(*px),
                fmt_coord(*py),
                fmt_coord(*px),
                fmt_coord(*py),
            )
            .unwrap();
        }
    }

    write!(
        buf,
        r#"<path d="{d}" fill="none" id="{from}-to-{to}" style="stroke:{arrow_color};stroke-width:1;"{dash}/>"#,
        from = xml_escape(&edge.from),
        to = xml_escape(&edge.to),
    )
    .unwrap();

    // Inline polygon arrowhead at the last point
    if edge.points.len() >= 2 {
        let (tx, ty) = edge.points[edge.points.len() - 1];
        let (fx, fy) = edge.points[edge.points.len() - 2];
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
            let p2x = tx;
            let p2y = ty;
            let p3x = tx - ux * 9.0 - px * 4.0;
            let p3y = ty - uy * 9.0 - py * 4.0;

            write!(
                buf,
                r#"<polygon fill="{arrow_color}" points="{},{},{},{},{},{},{},{}" style="stroke:{arrow_color};stroke-width:1;"/>"#,
                fmt_coord(p1x), fmt_coord(p1y),
                fmt_coord(p2x), fmt_coord(p2y),
                fmt_coord(p3x), fmt_coord(p3y),
                fmt_coord(p1x), fmt_coord(p1y),
            )
            .unwrap();
        }
    }

    // Label at midpoint
    if !edge.label.is_empty() {
        let mid = edge.points.len() / 2;
        let (mx, my) = if edge.points.len() == 2 {
            let (x1, y1) = edge.points[0];
            let (x2, y2) = edge.points[1];
            ((x1 + x2) / 2.0, (y1 + y2) / 2.0 - 6.0)
        } else {
            edge.points[mid]
        };

        render_creole_text(
            buf,
            &edge.label,
            mx,
            my,
            LINE_HEIGHT,
            font_color,
            Some("middle"),
            &format!(r#"font-size="{FONT_SIZE}""#),
        );
    }

    buf.push_str("</g>");
}

// ---------------------------------------------------------------------------
// Note rendering
// ---------------------------------------------------------------------------

fn render_note(
    buf: &mut String,
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

    write!(
        buf,
        r#"<polygon fill="{bg}" points="{},{},{},{},{},{},{},{},{},{}" style="stroke:{border};"/>"#,
        fmt_coord(x),
        fmt_coord(y),
        fmt_coord(x + w - fold),
        fmt_coord(y),
        fmt_coord(x + w),
        fmt_coord(y + fold),
        fmt_coord(x + w),
        fmt_coord(y + h),
        fmt_coord(x),
        fmt_coord(y + h),
    )
    .unwrap();

    write!(
        buf,
        r#"<line style="stroke:{border};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(x + w - fold),
        fmt_coord(x + w - fold),
        fmt_coord(y),
        fmt_coord(y + fold),
    )
    .unwrap();
    write!(
        buf,
        r#"<line style="stroke:{border};" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
        fmt_coord(x + w - fold),
        fmt_coord(x + w),
        fmt_coord(y + fold),
        fmt_coord(y + fold),
    )
    .unwrap();

    let text_x = x + 6.0;
    let text_y = y + fold + FONT_SIZE;
    render_creole_text(
        buf,
        &note.text,
        text_x,
        text_y,
        LINE_HEIGHT,
        font_color,
        None,
        r#"font-size="13""#,
    );
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
        assert!(svg.contains("<tspan"), "multiline note must use tspan");
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
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(
            svg.contains("\u{00AB}service\u{00BB}"),
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
        });
        let svg =
            render_component(&diagram, &layout, &SkinParams::default()).expect("render failed");
        assert!(svg.contains(r#"font-weight="bold""#));
        assert!(svg.contains(r#"href="https://example.com""#));
        assert!(svg.contains("<title>hover</title>"));
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
            color: None,
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
