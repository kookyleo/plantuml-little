use std::fmt::Write;

use crate::layout::component::{
    ComponentEdgeLayout, ComponentGroupLayout, ComponentLayout, ComponentNodeLayout,
    ComponentNoteLayout,
};
use crate::model::component::{ComponentDiagram, ComponentKind};
use crate::render::svg::xml_escape;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// ---------------------------------------------------------------------------
// Style constants (PlantUML defaults)
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 12.0;
const FONT_FAMILY: &str = "monospace";
const LINE_HEIGHT: f64 = 16.0;
const COMPONENT_BG: &str = "#FEFECE";
const COMPONENT_BORDER: &str = "#A80036";
const RECT_BG: &str = "#FEFECE";
const RECT_BORDER: &str = "#A80036";
const NODE_BG: &str = "#FEFECE";
const NODE_BORDER: &str = "#A80036";
const DATABASE_BG: &str = "#FEFECE";
const DATABASE_BORDER: &str = "#A80036";
const CLOUD_BG: &str = "#F1F1F1";
const CLOUD_BORDER: &str = "#A80036";
const EDGE_COLOR: &str = "#A80036";
const TEXT_FILL: &str = "#000000";
const NOTE_BG: &str = "#FBFB77";
const NOTE_BORDER: &str = "#A80036";
const GROUP_BG: &str = "#FFFFFF";
const GROUP_BORDER: &str = "#A80036";
// Deployment diagram element colors
const ARTIFACT_BG: &str = "#FEFECE";
const ARTIFACT_BORDER: &str = "#A80036";
const STORAGE_BG: &str = "#FEFECE";
const STORAGE_BORDER: &str = "#A80036";
const FOLDER_BG: &str = "#FEFECE";
const FOLDER_BORDER: &str = "#A80036";
const FRAME_BG: &str = "#FFFFFF";
const FRAME_BORDER: &str = "#A80036";
const AGENT_BG: &str = "#FEFECE";
const AGENT_BORDER: &str = "#A80036";
const STACK_BG: &str = "#FEFECE";
const STACK_BORDER: &str = "#A80036";
const QUEUE_BG: &str = "#FEFECE";
const QUEUE_BORDER: &str = "#A80036";

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
    write!(
        buf,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w:.0} {h:.0}" width="{w:.0}" height="{h:.0}" font-family="{FONT_FAMILY}" font-size="{FONT_SIZE}">"#,
        w = layout.width,
        h = layout.height,
    )
    .unwrap();
    buf.push('\n');

    // Defs: arrow marker
    write_defs(&mut buf, arrow_color);

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

    buf.push_str("</svg>\n");
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Defs
// ---------------------------------------------------------------------------

fn write_defs(buf: &mut String, arrow_color: &str) {
    buf.push_str("<defs>\n");
    write!(
        buf,
        concat!(
            r#"<marker id="comp-arrow" viewBox="0 0 10 10" refX="10" refY="5""#,
            r#" markerWidth="8" markerHeight="8" orient="auto-start-reverse">"#,
            r#"<path d="M 0 0 L 10 5 L 0 10 Z" fill="{}" stroke="none"/>"#,
            r#"</marker>"#,
        ),
        arrow_color,
    )
    .unwrap();
    buf.push('\n');
    buf.push_str("</defs>\n");
}

// ---------------------------------------------------------------------------
// Group rendering
// ---------------------------------------------------------------------------

fn render_group(
    buf: &mut String,
    group: &ComponentGroupLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    // Draw background rectangle
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="4" ry="4" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x = group.x,
        y = group.y,
        w = group.width,
        h = group.height,
    )
    .unwrap();
    buf.push('\n');

    // Group name at top
    let name_escaped = xml_escape(&group.name);
    let name_x = group.x + 10.0;
    let name_y = group.y + FONT_SIZE + 6.0;
    write!(
        buf,
        r#"<text x="{name_x:.1}" y="{name_y:.1}" font-weight="bold" fill="{font_color}">{name_escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');

    // Separator line below header
    let sep_y = name_y + 6.0;
    write!(
        buf,
        r#"<line x1="{x1:.1}" y1="{sy:.1}" x2="{x2:.1}" y2="{sy:.1}" stroke="{border}"/>"#,
        x1 = group.x,
        sy = sep_y,
        x2 = group.x + group.width,
    )
    .unwrap();
    buf.push('\n');
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
    // If node has an explicit color, use it as the background fill override
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
        // Deployment diagram kinds with distinct shapes
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

/// Component: rounded rect with a small "component icon" (two small rectangles on the left)
fn render_component_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    // Main rounded rectangle
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="6" ry="6" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

    // Component icon: two small rectangles on the left border
    let icon_x = node.x - 5.0;
    let icon_y1 = node.y + node.height * 0.25 - 4.0;
    let icon_y2 = node.y + node.height * 0.60 - 4.0;
    for iy in [icon_y1, icon_y2] {
        write!(
            buf,
            r#"<rect x="{icon_x:.1}" y="{iy:.1}" width="10" height="8" rx="1" ry="1" fill="{bg}" stroke="{border}" stroke-width="1"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    // Text content
    render_node_text(buf, node, font_color);
}

/// Rectangle: simple rectangle with optional description
fn render_rectangle_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Database: cylinder shape (ellipses at top and bottom)
fn render_database_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let ry = 8.0; // ellipse radius for cylinder top/bottom

    // Body path: rectangle with elliptical top and bottom
    write!(
        buf,
        r#"<path d="M {x:.1} {y1:.1} A {rx:.1} {ry:.1} 0 0 1 {x2:.1} {y1:.1} L {x2:.1} {y2:.1} A {rx:.1} {ry:.1} 0 0 1 {x:.1} {y2:.1} Z" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x = x,
        y1 = y + ry,
        rx = w / 2.0,
        ry = ry,
        x2 = x + w,
        y2 = y + h - ry,
    )
    .unwrap();
    buf.push('\n');

    // Top ellipse
    write!(
        buf,
        r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        cx = x + w / 2.0,
        cy = y + ry,
        rx = w / 2.0,
        ry = ry,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Cloud: rendered as a rounded rect with large radius (simplified)
fn render_cloud_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="20" ry="20" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Generic box (used for Node, Package)
fn render_box_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    fill: &str,
    border: &str,
    font_color: &str,
) {
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="4" ry="4" fill="{fill}" stroke="{border}" stroke-width="1.5"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Interface: small circle with name below
fn render_interface_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let cx = node.x + node.width / 2.0;
    let cy = node.y + 12.0;
    write!(
        buf,
        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="8" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Name below the circle
    let name_escaped = xml_escape(&node.name);
    let name_y = cy + 20.0;
    write!(
        buf,
        r#"<text x="{cx:.1}" y="{name_y:.1}" text-anchor="middle" fill="{font_color}">{name_escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

/// Artifact: rectangle with a folded corner (document icon)
fn render_artifact_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let fold = 10.0; // size of the folded corner

    // Body polygon with folded top-right corner
    write!(
        buf,
        r#"<polygon points="{x:.1},{y:.1} {xf:.1},{y:.1} {xw:.1},{yf:.1} {xw:.1},{yh:.1} {x:.1},{yh:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        xf = x + w - fold,
        xw = x + w,
        yf = y + fold,
        yh = y + h,
    )
    .unwrap();
    buf.push('\n');

    // Fold lines to indicate the corner fold
    write!(
        buf,
        r#"<line x1="{xf:.1}" y1="{y:.1}" x2="{xf:.1}" y2="{yf:.1}" stroke="{border}" stroke-width="1"/>"#,
        xf = x + w - fold,
        yf = y + fold,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<line x1="{xf:.1}" y1="{yf:.1}" x2="{xw:.1}" y2="{yf:.1}" stroke="{border}" stroke-width="1"/>"#,
        xf = x + w - fold,
        xw = x + w,
        yf = y + fold,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Storage: horizontal cylinder shape (ellipses on left and right sides)
fn render_storage_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let rx = 8.0; // ellipse x-radius for horizontal cylinder

    // Body: rectangle body with elliptical left/right ends
    write!(
        buf,
        r#"<path d="M {x1:.1} {y:.1} L {x2:.1} {y:.1} A {rx:.1} {ry:.1} 0 0 1 {x2:.1} {yh:.1} L {x1:.1} {yh:.1} A {rx:.1} {ry:.1} 0 0 1 {x1:.1} {y:.1} Z" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x1 = x + rx,
        x2 = x + w - rx,
        ry = h / 2.0,
        yh = y + h,
    )
    .unwrap();
    buf.push('\n');

    // Right ellipse (visible end cap)
    write!(
        buf,
        r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        cx = x + w - rx,
        cy = y + h / 2.0,
        ry = h / 2.0,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Folder: rectangle with a small tab on the top-left
fn render_folder_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let tab_w = (w * 0.35).min(60.0); // tab width
    let tab_h = 8.0; // tab height above the main body

    // Tab on top-left
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{tab_w:.1}" height="{tab_h:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Main folder body (shifted down by tab_h)
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{by:.1}" width="{w:.1}" height="{bh:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        by = y + tab_h,
        bh = h - tab_h,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Frame: rectangle with a small label tab on the top-left corner
fn render_frame_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    _font_color: &str,
) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let tab_w = (w * 0.4).min(70.0);
    let tab_h = FONT_SIZE + 6.0;

    // Outer border rectangle
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Small label inset box on top-left corner
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{tab_w:.1}" height="{tab_h:.1}" fill="{border}" stroke="{border}" stroke-width="1"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Name inside the label tab (white text for contrast)
    let name_escaped = xml_escape(&node.name);
    let label_cx = x + tab_w / 2.0;
    let label_cy = y + tab_h / 2.0 + FONT_SIZE * 0.35;
    write!(
        buf,
        "<text x=\"{label_cx:.1}\" y=\"{label_cy:.1}\" text-anchor=\"middle\" font-weight=\"bold\" fill=\"#FFFFFF\" font-size=\"{fs:.0}\">{name_escaped}</text>",
        fs = FONT_SIZE - 1.0,
    )
    .unwrap();
    buf.push('\n');
}

/// Agent: rounded rectangle (rx=10) — deployment monitoring/process shape
fn render_agent_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="10" ry="10" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x = node.x,
        y = node.y,
        w = node.width,
        h = node.height,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Stack: three stacked rectangles to suggest layering
fn render_stack_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let shadow_offset = 4.0;

    // Render two shadow layers behind (bottom-most first)
    for i in [2u32, 1] {
        let off = shadow_offset * f64::from(i);
        write!(
            buf,
            r#"<rect x="{sx:.1}" y="{sy:.1}" width="{w:.1}" height="{h:.1}" fill="{bg}" stroke="{border}" stroke-width="1" opacity="0.6"/>"#,
            sx = x + off,
            sy = y + off,
        )
        .unwrap();
        buf.push('\n');
    }

    // Front (main) rectangle
    write!(
        buf,
        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
}

/// Queue: rectangle with a wavy/elliptical right end to suggest FIFO
fn render_queue_node(
    buf: &mut String,
    node: &ComponentNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let cap_rx = 8.0; // ellipse x-radius for end caps

    // Main body as a rounded rect with ellipse at each end
    // Left cap (closed ellipse), right cap (open ellipse, visible)
    write!(
        buf,
        r#"<path d="M {x1:.1} {y:.1} L {x2:.1} {y:.1} A {cap_rx:.1} {ry:.1} 0 0 1 {x2:.1} {yh:.1} L {x1:.1} {yh:.1} A {cap_rx:.1} {ry:.1} 0 0 1 {x1:.1} {y:.1} Z" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        x1 = x + cap_rx,
        x2 = x + w - cap_rx,
        ry = h / 2.0,
        yh = y + h,
    )
    .unwrap();
    buf.push('\n');

    // Left end cap (fully closed)
    write!(
        buf,
        r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{cap_rx:.1}" ry="{ry:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        cx = x + cap_rx,
        cy = y + h / 2.0,
        ry = h / 2.0,
    )
    .unwrap();
    buf.push('\n');

    // Right end cap (visible opening)
    write!(
        buf,
        r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{cap_rx:.1}" ry="{ry:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        cx = x + w - cap_rx,
        cy = y + h / 2.0,
        ry = h / 2.0,
    )
    .unwrap();
    buf.push('\n');

    render_node_text(buf, node, font_color);
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
        write!(
            buf,
            r#"<text x="{cx:.1}" y="{sy:.1}" text-anchor="middle" font-size="{fs:.0}" font-style="italic" fill="{font_color}">{escaped}</text>"#,
            fs = FONT_SIZE - 2.0,
        )
        .unwrap();
        buf.push('\n');
        y_offset = LINE_HEIGHT;
    }

    // Name
    let name_escaped = xml_escape(&node.name);
    let name_y = if has_desc {
        node.y + FONT_SIZE + 4.0 + y_offset
    } else {
        node.y + node.height / 2.0 + FONT_SIZE * 0.35 + y_offset
    };
    write!(
        buf,
        r#"<text x="{cx:.1}" y="{name_y:.1}" text-anchor="middle" font-weight="bold" fill="{font_color}">{name_escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');

    // Description
    if has_desc {
        let sep_y = name_y + 6.0;
        write!(
            buf,
            r#"<line x1="{x1:.1}" y1="{sy:.1}" x2="{x2:.1}" y2="{sy:.1}" stroke="{COMPONENT_BORDER}"/>"#,
            x1 = node.x,
            sy = sep_y,
            x2 = node.x + node.width,
        )
        .unwrap();
        buf.push('\n');

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
            "",
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

    let dash = if edge.dashed {
        r#" stroke-dasharray="7,5""#
    } else {
        ""
    };

    if edge.points.len() == 2 {
        let (x1, y1) = edge.points[0];
        let (x2, y2) = edge.points[1];
        write!(
            buf,
            r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{arrow_color}" stroke-width="1"{dash} marker-end="url(#comp-arrow)"/>"#,
        )
        .unwrap();
        buf.push('\n');
    } else {
        let points_str: String = edge
            .points
            .iter()
            .map(|(px, py)| format!("{px:.1},{py:.1}"))
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            buf,
            r#"<polyline points="{points_str}" fill="none" stroke="{arrow_color}" stroke-width="1"{dash} marker-end="url(#comp-arrow)"/>"#,
        )
        .unwrap();
        buf.push('\n');
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

        let escaped = xml_escape(&edge.label);
        write!(
            buf,
            r#"<text x="{mx:.1}" y="{my:.1}" text-anchor="middle" font-size="{FONT_SIZE}" fill="{font_color}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
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

    // Note body polygon (with folded corner)
    write!(
        buf,
        r#"<polygon points="{x:.1},{y:.1} {xf:.1},{y:.1} {xw:.1},{yf:.1} {xw:.1},{yh:.1} {x:.1},{yh:.1}" fill="{bg}" stroke="{border}"/>"#,
        xf = x + w - fold,
        xw = x + w,
        yf = y + fold,
        yh = y + h,
    )
    .unwrap();
    buf.push('\n');

    // Fold lines
    write!(
        buf,
        r#"<line x1="{xf:.1}" y1="{y:.1}" x2="{xf:.1}" y2="{yf:.1}" stroke="{border}"/>"#,
        xf = x + w - fold,
        yf = y + fold,
    )
    .unwrap();
    buf.push('\n');
    write!(
        buf,
        r#"<line x1="{xf:.1}" y1="{yf:.1}" x2="{xw:.1}" y2="{yf:.1}" stroke="{border}"/>"#,
        xf = x + w - fold,
        yf = y + fold,
        xw = x + w,
    )
    .unwrap();
    buf.push('\n');

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
        "",
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
        assert!(svg.contains("comp-arrow"), "must define comp-arrow marker");
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
        // Component has 3 rects: main + 2 icon pieces
        let rect_count = svg.matches("<rect").count();
        assert!(
            rect_count >= 3,
            "component must have 3 rects, got {}",
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
        assert!(svg.contains("<ellipse"), "database uses ellipse for top");
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
            svg.contains(r#"marker-end="url(#comp-arrow)""#),
            "edge must reference comp-arrow marker"
        );
        assert!(
            svg.contains(&format!(r#"stroke="{EDGE_COLOR}""#)),
            "edge must use EDGE_COLOR"
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
            svg.contains(&format!(r#"fill="{NOTE_BG}""#)),
            "note must use yellow background"
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
            !svg.contains("marker-end"),
            "no edges should mean no arrow markers on elements"
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
        assert!(svg.contains("width=\"400\""), "width must match");
        assert!(svg.contains("<defs>"), "must have defs");
        assert!(svg.contains("</defs>"), "must have closing defs");
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

    // 18. Polyline edge (multiple points)
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
        assert!(
            svg.contains("<polyline"),
            "multi-point edge must use polyline"
        );
    }
}
