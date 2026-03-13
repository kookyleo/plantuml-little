use std::fmt::Write;

use crate::layout::erd::{
    ErdAttrLayout, ErdEdgeLayout, ErdIsaLayout, ErdLayout, ErdNodeLayout, ErdNoteLayout,
};
use crate::model::erd::ErdDiagram;
use crate::render::svg::xml_escape;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// ── Style constants ──────────────────────────────────────────────────

const FONT_SIZE: f64 = 12.0;
const FONT_FAMILY: &str = "monospace";
const ENTITY_BG: &str = "#FEFECE";
const ENTITY_BORDER: &str = "#A80036";
const RELATIONSHIP_BG: &str = "#FEFECE";
const RELATIONSHIP_BORDER: &str = "#A80036";
const ATTR_BG: &str = "#FEFECE";
const ATTR_BORDER: &str = "#A80036";
const EDGE_COLOR: &str = "#A80036";
const TEXT_FILL: &str = "#000000";
const ISA_BG: &str = "#FEFECE";
const ISA_BORDER: &str = "#A80036";
const NOTE_BG: &str = "#FBFB77";
const NOTE_BORDER: &str = "#A80036";
const NOTE_FOLD: f64 = 8.0;

// ── Public entry point ──────────────────────────────────────────────

/// Render an ERD diagram to SVG.
pub fn render_erd(_ed: &ErdDiagram, layout: &ErdLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);

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
    write_defs(&mut buf);

    let ent_bg = skin.background_color("entity", ENTITY_BG);
    let ent_border = skin.border_color("entity", ENTITY_BORDER);
    let ent_font = skin.font_color("entity", TEXT_FILL);

    // Entity nodes
    for node in &layout.entity_nodes {
        render_entity(&mut buf, node, ent_bg, ent_border, ent_font);
    }

    // Relationship nodes
    for node in &layout.relationship_nodes {
        render_relationship(&mut buf, node);
    }

    // Attributes
    for attr in &layout.attribute_nodes {
        render_attribute(&mut buf, attr);
        // Line from parent center to attribute
        // (parent center will be resolved from node positions)
    }

    // Attribute-to-parent lines
    render_attr_parent_lines(
        &mut buf,
        &layout.attribute_nodes,
        &layout.entity_nodes,
        &layout.relationship_nodes,
    );

    // Edges (links)
    for edge in &layout.edges {
        render_edge(&mut buf, edge);
    }

    // ISA triangles
    for isa in &layout.isa_layouts {
        render_isa(&mut buf, isa);
    }

    // Notes
    for note in &layout.notes {
        render_note(&mut buf, note);
    }

    buf.push_str("</svg>\n");
    Ok(buf)
}

// ── Defs ────────────────────────────────────────────────────────────

fn write_defs(buf: &mut String) {
    buf.push_str("<defs>\n");
    // No arrow markers needed for Chen notation (uses cardinality labels instead)
    buf.push_str("</defs>\n");
}

// ── Entity rendering ────────────────────────────────────────────────

fn render_entity(buf: &mut String, node: &ErdNodeLayout, bg: &str, border: &str, font_color: &str) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    if node.is_weak {
        // Double-bordered rectangle for weak entity
        // Outer rectangle
        write!(
            buf,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        )
        .unwrap();
        buf.push('\n');
        // Inner rectangle (inset by 3px)
        let inset = 3.0;
        write!(
            buf,
            r#"<rect x="{ix:.1}" y="{iy:.1}" width="{iw:.1}" height="{ih:.1}" fill="none" stroke="{border}" stroke-width="1"/>"#,
            ix = x + inset,
            iy = y + inset,
            iw = w - 2.0 * inset,
            ih = h - 2.0 * inset,
        )
        .unwrap();
        buf.push('\n');
    } else {
        // Single rectangle
        write!(
            buf,
            r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{bg}" stroke="{border}" stroke-width="1.5"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    // Entity name centered
    let cx = x + w / 2.0;
    let cy = y + h / 2.0 + FONT_SIZE * 0.35;
    let escaped = xml_escape(&node.label);
    write!(
        buf,
        r#"<text x="{cx:.1}" y="{cy:.1}" text-anchor="middle" font-weight="bold" fill="{font_color}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

// ── Relationship rendering ──────────────────────────────────────────

fn render_relationship(buf: &mut String, node: &ErdNodeLayout) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    // Diamond shape: 4 points (top, right, bottom, left)
    let top = format!("{cx:.1},{y:.1}");
    let right = format!("{:.1},{:.1}", x + w, cy);
    let bottom = format!("{:.1},{:.1}", cx, y + h);
    let left = format!("{x:.1},{cy:.1}");

    if node.is_identifying {
        // Double-bordered diamond for identifying relationship
        let points = format!("{top} {right} {bottom} {left}");
        write!(
            buf,
            r#"<polygon points="{points}" fill="{RELATIONSHIP_BG}" stroke="{RELATIONSHIP_BORDER}" stroke-width="1.5"/>"#,
        )
        .unwrap();
        buf.push('\n');

        // Inner diamond (inset by 4px)
        let inset = 4.0;
        let inner_top = format!("{:.1},{:.1}", cx, y + inset);
        let inner_right = format!("{:.1},{:.1}", x + w - inset * 1.5, cy);
        let inner_bottom = format!("{:.1},{:.1}", cx, y + h - inset);
        let inner_left = format!("{:.1},{:.1}", x + inset * 1.5, cy);
        let inner_points = format!("{inner_top} {inner_right} {inner_bottom} {inner_left}");
        write!(
            buf,
            r#"<polygon points="{inner_points}" fill="none" stroke="{RELATIONSHIP_BORDER}" stroke-width="1"/>"#,
        )
        .unwrap();
        buf.push('\n');
    } else {
        let points = format!("{top} {right} {bottom} {left}");
        write!(
            buf,
            r#"<polygon points="{points}" fill="{RELATIONSHIP_BG}" stroke="{RELATIONSHIP_BORDER}" stroke-width="1.5"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    // Relationship name centered
    let ty = cy + FONT_SIZE * 0.35;
    let escaped = xml_escape(&node.label);
    write!(
        buf,
        r#"<text x="{cx:.1}" y="{ty:.1}" text-anchor="middle" fill="{TEXT_FILL}">{escaped}</text>"#,
    )
    .unwrap();
    buf.push('\n');
}

// ── Attribute rendering ─────────────────────────────────────────────

fn render_attribute(buf: &mut String, attr: &ErdAttrLayout) {
    let cx = attr.x;
    let cy = attr.y;
    let rx = attr.rx;
    let ry = attr.ry;

    if attr.is_derived {
        // Dashed ellipse for derived attribute
        write!(
            buf,
            r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{ATTR_BG}" stroke="{ATTR_BORDER}" stroke-width="1" stroke-dasharray="5,3"/>"#,
        )
        .unwrap();
        buf.push('\n');
    } else if attr.is_multi {
        // Double ellipse for multi-valued attribute
        write!(
            buf,
            r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{ATTR_BG}" stroke="{ATTR_BORDER}" stroke-width="1.5"/>"#,
        )
        .unwrap();
        buf.push('\n');
        // Inner ellipse
        let inner_rx = rx - 3.0;
        let inner_ry = ry - 2.0;
        write!(
            buf,
            r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{inner_rx:.1}" ry="{inner_ry:.1}" fill="none" stroke="{ATTR_BORDER}" stroke-width="1"/>"#,
        )
        .unwrap();
        buf.push('\n');
    } else {
        // Simple ellipse
        write!(
            buf,
            r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{ATTR_BG}" stroke="{ATTR_BORDER}" stroke-width="1"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    // Attribute label
    let escaped = xml_escape(&attr.label);
    let ty = cy + FONT_SIZE * 0.35;

    if attr.is_key {
        // Underlined text for key attribute
        write!(
            buf,
            r#"<text x="{cx:.1}" y="{ty:.1}" text-anchor="middle" fill="{TEXT_FILL}" text-decoration="underline">{escaped}</text>"#,
        )
        .unwrap();
    } else {
        write!(
            buf,
            r#"<text x="{cx:.1}" y="{ty:.1}" text-anchor="middle" fill="{TEXT_FILL}">{escaped}</text>"#,
        )
        .unwrap();
    }
    buf.push('\n');

    // Type annotation (if any)
    if let Some(ref type_label) = attr.type_label {
        let type_escaped = xml_escape(type_label);
        let type_y = cy + FONT_SIZE * 0.35 + FONT_SIZE + 2.0;
        write!(
            buf,
            r#"<text x="{cx:.1}" y="{ty:.1}" text-anchor="middle" font-size="{fs:.0}" font-style="italic" fill="{TEXT_FILL}">{type_escaped}</text>"#,
            ty = type_y,
            fs = FONT_SIZE - 2.0,
        )
        .unwrap();
        buf.push('\n');
    }

    // Render children recursively
    for child in &attr.children {
        render_attribute(buf, child);
        // Line from parent attribute to child attribute
        write!(
            buf,
            r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{EDGE_COLOR}" stroke-width="1"/>"#,
            x1 = cx,
            y1 = cy - ry,
            x2 = child.x,
            y2 = child.y + child.ry,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Attribute-to-parent lines ────────────────────────────────────────

fn render_attr_parent_lines(
    buf: &mut String,
    attrs: &[ErdAttrLayout],
    entities: &[ErdNodeLayout],
    relationships: &[ErdNodeLayout],
) {
    for attr in attrs {
        // Find parent center
        let parent_center = entities
            .iter()
            .chain(relationships.iter())
            .find(|n| n.id == attr.parent)
            .map(|n| (n.x + n.width / 2.0, n.y + n.height / 2.0));

        if let Some((px, py)) = parent_center {
            write!(
                buf,
                r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{EDGE_COLOR}" stroke-width="1"/>"#,
                x1 = px,
                y1 = py,
                x2 = attr.x,
                y2 = attr.y,
            )
            .unwrap();
            buf.push('\n');
        }
    }
}

// ── Edge rendering ──────────────────────────────────────────────────

fn render_edge(buf: &mut String, edge: &ErdEdgeLayout) {
    let (x1, y1) = edge.from_point;
    let (x2, y2) = edge.to_point;

    if edge.is_double {
        // Double line: two parallel lines offset by 2px
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.001 {
            let nx = -dy / len * 1.5;
            let ny = dx / len * 1.5;

            write!(
                buf,
                r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{EDGE_COLOR}" stroke-width="1"/>"#,
                x1 + nx, y1 + ny, x2 + nx, y2 + ny,
            )
            .unwrap();
            buf.push('\n');
            write!(
                buf,
                r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{EDGE_COLOR}" stroke-width="1"/>"#,
                x1 - nx, y1 - ny, x2 - nx, y2 - ny,
            )
            .unwrap();
            buf.push('\n');
        }
    } else {
        write!(
            buf,
            r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{EDGE_COLOR}" stroke-width="1"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    // Cardinality label near the midpoint
    if !edge.label.is_empty() {
        let mx = (x1 + x2) / 2.0;
        let my = (y1 + y2) / 2.0 - 6.0;
        let escaped = xml_escape(&edge.label);
        write!(
            buf,
            r#"<text x="{mx:.1}" y="{my:.1}" text-anchor="middle" font-size="{FONT_SIZE}" fill="{TEXT_FILL}">{escaped}</text>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── ISA rendering ───────────────────────────────────────────────────

fn render_isa(buf: &mut String, isa: &ErdIsaLayout) {
    let (cx, cy) = isa.triangle_center;
    let s = isa.triangle_size;

    // Triangle pointing down: (cx, cy+s), (cx-s, cy-s/2), (cx+s, cy-s/2)
    let top_y = cy - s * 0.5;
    let bot_y = cy + s * 0.5;
    let left_x = cx - s * 0.6;
    let right_x = cx + s * 0.6;

    let points = format!("{cx:.1},{top_y:.1} {right_x:.1},{bot_y:.1} {left_x:.1},{bot_y:.1}");

    if isa.is_double {
        write!(
            buf,
            r#"<polygon points="{points}" fill="{ISA_BG}" stroke="{ISA_BORDER}" stroke-width="2"/>"#,
        )
        .unwrap();
    } else {
        write!(
            buf,
            r#"<polygon points="{points}" fill="{ISA_BG}" stroke="{ISA_BORDER}" stroke-width="1.5"/>"#,
        )
        .unwrap();
    }
    buf.push('\n');

    // Kind label (d or U) inside triangle
    let label_y = cy + FONT_SIZE * 0.2;
    let escaped = xml_escape(&isa.kind_label);
    write!(
        buf,
        r#"<text x="{cx:.1}" y="{ly:.1}" text-anchor="middle" font-size="{fs:.0}" fill="{TEXT_FILL}">{escaped}</text>"#,
        ly = label_y,
        fs = FONT_SIZE - 1.0,
    )
    .unwrap();
    buf.push('\n');

    // Line from parent to triangle top
    let (ppx, ppy) = isa.parent_point;
    write!(
        buf,
        r#"<line x1="{ppx:.1}" y1="{ppy:.1}" x2="{cx:.1}" y2="{top_y:.1}" stroke="{EDGE_COLOR}" stroke-width="1.5"/>"#,
    )
    .unwrap();
    buf.push('\n');

    // Lines from triangle bottom to children
    for (child_id, (child_x, child_y)) in &isa.child_points {
        let _ = child_id;
        write!(
            buf,
            r#"<line x1="{cx:.1}" y1="{bot_y:.1}" x2="{child_x:.1}" y2="{child_y:.1}" stroke="{EDGE_COLOR}" stroke-width="1"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }
}

// ── Note rendering ──────────────────────────────────────────────────

fn render_note(buf: &mut String, note: &ErdNoteLayout) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        write!(
            buf,
            r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{NOTE_BORDER}" stroke-width="1" stroke-dasharray="5,3"/>"#,
        )
        .unwrap();
        buf.push('\n');
    }

    let x = note.x;
    let y = note.y;
    let w = note.width;
    let h = note.height;
    let fold = NOTE_FOLD;
    write!(
        buf,
        r#"<polygon points="{x:.1},{y:.1} {x1:.1},{y:.1} {x2:.1},{y1:.1} {x2:.1},{y2:.1} {x:.1},{y2:.1}" fill="{NOTE_BG}" stroke="{NOTE_BORDER}" stroke-width="1"/>"#,
        x1 = x + w - fold,
        x2 = x + w,
        y1 = y + fold,
        y2 = y + h,
    )
    .unwrap();
    buf.push('\n');

    write!(
        buf,
        r#"<path d="M {x1:.1},{y:.1} L {x1:.1},{y1:.1} L {x2:.1},{y1:.1}" fill="none" stroke="{NOTE_BORDER}" stroke-width="1"/>"#,
        x1 = x + w - fold,
        x2 = x + w,
        y1 = y + fold,
    )
    .unwrap();
    buf.push('\n');

    let text_x = x + 10.0;
    let start_y = y + 10.0 + FONT_SIZE;
    render_creole_text(buf, &note.text, text_x, start_y, 16.0, TEXT_FILL, None, "");
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::erd::*;
    use crate::model::erd::ErdDiagram;
    use crate::style::SkinParams;

    fn empty_diagram() -> ErdDiagram {
        ErdDiagram {
            entities: vec![],
            relationships: vec![],
            links: vec![],
            isas: vec![],
            direction: crate::model::erd::ErdDirection::TopToBottom,
            notes: vec![],
        }
    }

    fn empty_layout() -> ErdLayout {
        ErdLayout {
            entity_nodes: vec![],
            relationship_nodes: vec![],
            attribute_nodes: vec![],
            edges: vec![],
            isa_layouts: vec![],
            notes: vec![],
            width: 400.0,
            height: 300.0,
        }
    }

    fn make_entity_node(id: &str, x: f64, y: f64, w: f64, h: f64) -> ErdNodeLayout {
        ErdNodeLayout {
            id: id.to_string(),
            label: id.to_string(),
            x,
            y,
            width: w,
            height: h,
            is_weak: false,
            is_identifying: false,
        }
    }

    fn make_attr(id: &str, parent: &str, x: f64, y: f64) -> ErdAttrLayout {
        ErdAttrLayout {
            id: id.to_string(),
            label: id.to_string(),
            parent: parent.to_string(),
            x,
            y,
            rx: 40.0,
            ry: 16.0,
            is_key: false,
            is_derived: false,
            is_multi: false,
            has_type: false,
            type_label: None,
            children: vec![],
        }
    }

    // 1. Empty diagram renders valid SVG
    #[test]
    fn test_empty_diagram() {
        let d = empty_diagram();
        let layout = empty_layout();
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"), "must contain <svg");
        assert!(svg.contains("</svg>"), "must contain </svg>");
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }

    // 2. Entity rendered as rectangle
    #[test]
    fn test_entity_rect() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("MOVIE", 50.0, 50.0, 100.0, 36.0));
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<rect"), "entity must produce a rect");
        assert!(svg.contains("MOVIE"), "entity name must appear");
        assert!(
            svg.contains(r#"font-weight="bold""#),
            "entity name must be bold"
        );
    }

    // 3. Weak entity: double border
    #[test]
    fn test_weak_entity_double_border() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.entity_nodes.push(ErdNodeLayout {
            is_weak: true,
            ..make_entity_node("CHILD", 50.0, 50.0, 100.0, 36.0)
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        let rect_count = svg.matches("<rect").count();
        assert_eq!(
            rect_count, 2,
            "weak entity must have 2 rects (double border)"
        );
    }

    // 4. Relationship rendered as diamond
    #[test]
    fn test_relationship_diamond() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.relationship_nodes.push(ErdNodeLayout {
            id: "RENTED_TO".to_string(),
            label: "RENTED_TO".to_string(),
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 40.0,
            is_weak: false,
            is_identifying: false,
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("<polygon"),
            "relationship must produce a polygon (diamond)"
        );
        assert!(svg.contains("RENTED_TO"), "relationship name must appear");
    }

    // 5. Identifying relationship: double diamond
    #[test]
    fn test_identifying_relationship() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.relationship_nodes.push(ErdNodeLayout {
            id: "PARENT_OF".to_string(),
            label: "PARENT_OF".to_string(),
            x: 50.0,
            y: 50.0,
            width: 120.0,
            height: 40.0,
            is_weak: false,
            is_identifying: true,
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        let poly_count = svg.matches("<polygon").count();
        assert_eq!(
            poly_count, 2,
            "identifying relationship must have 2 polygons"
        );
    }

    // 6. Attribute ellipse
    #[test]
    fn test_attribute_ellipse() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        layout
            .attribute_nodes
            .push(make_attr("Code", "E", 100.0, 40.0));
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("<ellipse"),
            "attribute must produce an ellipse"
        );
        assert!(svg.contains("Code"), "attribute name must appear");
    }

    // 7. Key attribute: underlined text
    #[test]
    fn test_key_attribute_underline() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        layout.attribute_nodes.push(ErdAttrLayout {
            is_key: true,
            ..make_attr("Number", "E", 100.0, 40.0)
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains(r#"text-decoration="underline""#),
            "key attribute must be underlined"
        );
    }

    // 8. Derived attribute: dashed ellipse
    #[test]
    fn test_derived_attribute_dashed() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        layout.attribute_nodes.push(ErdAttrLayout {
            is_derived: true,
            ..make_attr("Bonus", "E", 100.0, 40.0)
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("stroke-dasharray"),
            "derived attribute must have dashed stroke"
        );
    }

    // 9. Multi-valued attribute: double ellipse
    #[test]
    fn test_multi_attribute_double_ellipse() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        layout.attribute_nodes.push(ErdAttrLayout {
            is_multi: true,
            ..make_attr("Name", "E", 100.0, 40.0)
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        let ellipse_count = svg.matches("<ellipse").count();
        assert_eq!(
            ellipse_count, 2,
            "multi-valued attribute must have 2 ellipses"
        );
    }

    // 10. Edge (link) rendering
    #[test]
    fn test_edge_rendering() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ErdEdgeLayout {
            from_id: "R".to_string(),
            to_id: "E".to_string(),
            from_point: (100.0, 100.0),
            to_point: (200.0, 100.0),
            label: "N".to_string(),
            is_double: false,
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<line"), "edge must produce a line");
        assert!(svg.contains("N"), "cardinality label must appear");
    }

    // 11. Double edge
    #[test]
    fn test_double_edge() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.edges.push(ErdEdgeLayout {
            from_id: "R".to_string(),
            to_id: "E".to_string(),
            from_point: (100.0, 100.0),
            to_point: (200.0, 100.0),
            label: "N".to_string(),
            is_double: true,
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        let line_count = svg.matches("<line").count();
        assert_eq!(line_count, 2, "double edge must produce 2 lines");
    }

    // 12. ISA triangle rendering
    #[test]
    fn test_isa_triangle() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.isa_layouts.push(ErdIsaLayout {
            parent_id: "PARENT".to_string(),
            kind_label: "d".to_string(),
            triangle_center: (200.0, 200.0),
            triangle_size: 24.0,
            parent_point: (200.0, 170.0),
            child_points: vec![
                ("C1".to_string(), (160.0, 250.0)),
                ("C2".to_string(), (240.0, 250.0)),
            ],
            is_double: true,
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("<polygon"),
            "ISA must produce a triangle polygon"
        );
        // Triangle + label "d" + line to parent + 2 lines to children = >=3 lines
        let line_count = svg.matches("<line").count();
        assert!(
            line_count >= 3,
            "ISA must have lines to parent and children, got {}",
            line_count
        );
        assert!(svg.contains(">d<"), "ISA must show kind label 'd'");
    }

    // 13. Attribute-to-parent lines
    #[test]
    fn test_attr_parent_lines() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        layout
            .attribute_nodes
            .push(make_attr("X", "E", 140.0, 40.0));
        layout
            .attribute_nodes
            .push(make_attr("Y", "E", 100.0, 40.0));
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        // 2 lines connecting attributes to entity center
        let line_count = svg.matches("<line").count();
        assert!(
            line_count >= 2,
            "must have attribute-to-parent lines, got {}",
            line_count
        );
    }

    // 14. XML escaping in entity name
    #[test]
    fn test_xml_escaping() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.entity_nodes.push(ErdNodeLayout {
            label: "A & B < C".to_string(),
            ..make_entity_node("E", 50.0, 50.0, 120.0, 36.0)
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("A &amp; B &lt; C"),
            "entity name must be XML-escaped"
        );
    }

    // 15. Attribute type annotation
    #[test]
    fn test_attribute_type_annotation() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        layout.attribute_nodes.push(ErdAttrLayout {
            has_type: true,
            type_label: Some("DATE".to_string()),
            ..make_attr("Born", "E", 100.0, 40.0)
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("DATE"), "type annotation must appear");
        assert!(
            svg.contains("font-style=\"italic\""),
            "type should be italic"
        );
    }

    // 16. SVG dimensions match layout
    #[test]
    fn test_svg_dimensions() {
        let d = empty_diagram();
        let layout = ErdLayout {
            width: 500.0,
            height: 400.0,
            ..empty_layout()
        };
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("width=\"500\""), "width must match");
        assert!(svg.contains("height=\"400\""), "height must match");
        assert!(
            svg.contains("viewBox=\"0 0 500 400\""),
            "viewBox must match"
        );
    }

    // 17. Attribute nested children
    #[test]
    fn test_nested_children_rendered() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout
            .entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        let mut attr = make_attr("Name", "E", 100.0, 40.0);
        attr.children = vec![
            make_attr("Fname", "Name", 80.0, 10.0),
            make_attr("Lname", "Name", 120.0, 10.0),
        ];
        layout.attribute_nodes.push(attr);
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("Fname"), "child attr Fname must appear");
        assert!(svg.contains("Lname"), "child attr Lname must appear");
        // 3 ellipses: Name, Fname, Lname
        let ellipse_count = svg.matches("<ellipse").count();
        assert_eq!(
            ellipse_count, 3,
            "must have 3 ellipses for parent + 2 children"
        );
    }

    // 18. Note with connector
    #[test]
    fn test_note_rendering() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.notes.push(ErdNoteLayout {
            text: "primary entity".to_string(),
            x: 180.0,
            y: 60.0,
            width: 110.0,
            height: 40.0,
            lines: vec!["primary entity".to_string()],
            connector: Some((180.0, 80.0, 140.0, 80.0)),
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("<polygon"),
            "note must render as folded polygon"
        );
        assert!(svg.contains("primary entity"), "note text must appear");
        assert!(
            svg.contains("stroke-dasharray"),
            "note connector must be dashed"
        );
    }

    // 19. Multiline note uses tspans
    #[test]
    fn test_multiline_note_rendering() {
        let d = empty_diagram();
        let mut layout = empty_layout();
        layout.notes.push(ErdNoteLayout {
            text: "line 1\nline 2".to_string(),
            x: 180.0,
            y: 60.0,
            width: 110.0,
            height: 56.0,
            lines: vec!["line 1".to_string(), "line 2".to_string()],
            connector: None,
        });
        let svg = render_erd(&d, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<tspan"), "multiline note must use tspan");
        assert!(svg.contains("line 1"), "first line must appear");
        assert!(svg.contains("line 2"), "second line must appear");
    }
}
