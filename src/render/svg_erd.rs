use std::fmt::Write;

use crate::layout::erd::{
    ErdAttrLayout, ErdEdgeLayout, ErdIsaLayout, ErdLayout, ErdNodeLayout, ErdNoteLayout,
};
use crate::model::erd::ErdDiagram;
use crate::render::svg::write_svg_root;
use crate::render::svg::{fmt_coord, xml_escape};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

// ── Style constants ──────────────────────────────────────────────────

const FONT_SIZE: f64 = 14.0;
const ENTITY_BG: &str = "#F1F1F1";
const ENTITY_BORDER: &str = "#181818";
const RELATIONSHIP_BG: &str = "#F1F1F1";
const RELATIONSHIP_BORDER: &str = "#181818";
const ATTR_BG: &str = "#F1F1F1";
const ATTR_BORDER: &str = "#181818";
const EDGE_COLOR: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const ISA_BG: &str = "#F1F1F1";
const ISA_BORDER: &str = "#181818";
const NOTE_BG: &str = "#FEFFDD";
const NOTE_BORDER: &str = "#181818";
const NOTE_FOLD: f64 = 8.0;

// ── Helper: render a straight-line path ─────────────────────────────

/// Emit a `<path d="M ... L ...">` element matching Java PlantUML edge style.
fn render_path_line(buf: &mut String, x1: f64, y1: f64, x2: f64, y2: f64) {
    write!(
        buf,
        r#"<path d="M{},{} L{},{} " fill="none" style="stroke:{EDGE_COLOR};stroke-width:1;"/>"#,
        fmt_coord(x1),
        fmt_coord(y1),
        fmt_coord(x2),
        fmt_coord(y2),
    )
    .unwrap();
}

/// Format a point as "x,y" for polygon points attributes.
fn fmt_pt(x: f64, y: f64) -> String {
    format!("{},{}", fmt_coord(x), fmt_coord(y))
}

// ── Public entry point ──────────────────────────────────────────────

/// Render an ERD diagram to SVG.
pub fn render_erd(_ed: &ErdDiagram, layout: &ErdLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    // SVG header
    write_svg_root(&mut buf, layout.width, layout.height, "CHEN_EER");
    buf.push_str("<defs/><g>");

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

    buf.push_str("</g></svg>");
    Ok(buf)
}

// ── Entity rendering ────────────────────────────────────────────────

fn render_entity(buf: &mut String, node: &ErdNodeLayout, bg: &str, border: &str, font_color: &str) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    if node.is_weak {
        // Double-bordered rectangle for weak entity
        write!(
            buf,
            r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
        )
        .unwrap();
        // Inner rectangle (inset by 3px)
        let inset = 3.0;
        write!(
            buf,
            r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(h - 2.0 * inset),
            fmt_coord(w - 2.0 * inset),
            fmt_coord(x + inset),
            fmt_coord(y + inset),
        )
        .unwrap();
    } else {
        write!(
            buf,
            r#"<rect fill="{bg}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
            fmt_coord(h), fmt_coord(w), fmt_coord(x), fmt_coord(y),
        )
        .unwrap();
    }

    // Entity name text (matching Java PlantUML: left-aligned with lengthAdjust)
    let tx = x + 10.0;
    let ty = y + h / 2.0 + FONT_SIZE * 0.35;
    let escaped = xml_escape(&node.label);
    write!(
        buf,
        r#"<text fill="{font_color}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(FONT_SIZE),
        fmt_coord(w - 20.0),
        fmt_coord(tx),
        fmt_coord(ty),
    )
    .unwrap();
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
    let top = fmt_pt(cx, y);
    let right = fmt_pt(x + w, cy);
    let bottom = fmt_pt(cx, y + h);
    let left = fmt_pt(x, cy);

    if node.is_identifying {
        let points = format!("{top} {right} {bottom} {left}");
        write!(
            buf,
            r#"<polygon fill="{RELATIONSHIP_BG}" points="{points}" style="stroke:{RELATIONSHIP_BORDER};stroke-width:0.5;"/>"#,
        )
        .unwrap();

        // Inner diamond (inset)
        let inset = 4.0;
        let inner_top = fmt_pt(cx, y + inset);
        let inner_right = fmt_pt(x + w - inset * 1.5, cy);
        let inner_bottom = fmt_pt(cx, y + h - inset);
        let inner_left = fmt_pt(x + inset * 1.5, cy);
        let inner_points = format!("{inner_top} {inner_right} {inner_bottom} {inner_left}");
        write!(
            buf,
            r#"<polygon fill="{RELATIONSHIP_BG}" points="{inner_points}" style="stroke:{RELATIONSHIP_BORDER};stroke-width:0.5;"/>"#,
        )
        .unwrap();
    } else {
        let points = format!("{top} {right} {bottom} {left}");
        write!(
            buf,
            r#"<polygon fill="{RELATIONSHIP_BG}" points="{points}" style="stroke:{RELATIONSHIP_BORDER};stroke-width:0.5;"/>"#,
        )
        .unwrap();
    }

    // Relationship name centered
    let ty = cy + FONT_SIZE * 0.35;
    let escaped = xml_escape(&node.label);
    let text_w = w - 2.0 * 20.0;
    let text_x = cx - text_w / 2.0;
    write!(
        buf,
        r#"<text fill="{TEXT_FILL}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(FONT_SIZE),
        fmt_coord(text_w),
        fmt_coord(text_x),
        fmt_coord(ty),
    )
    .unwrap();
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
            r#"<ellipse cx="{}" cy="{}" fill="{ATTR_BG}" rx="{}" ry="{}" style="stroke:{ATTR_BORDER};stroke-width:0.5;stroke-dasharray:10,10;"/>"#,
            fmt_coord(cx), fmt_coord(cy), fmt_coord(rx), fmt_coord(ry),
        )
        .unwrap();
    } else if attr.is_multi {
        // Double ellipse for multi-valued attribute
        write!(
            buf,
            r#"<ellipse cx="{}" cy="{}" fill="{ATTR_BG}" rx="{}" ry="{}" style="stroke:{ATTR_BORDER};stroke-width:0.5;"/>"#,
            fmt_coord(cx), fmt_coord(cy), fmt_coord(rx), fmt_coord(ry),
        )
        .unwrap();
        // Inner ellipse (inset by 3px in both directions, matching Java)
        let inner_rx = rx - 3.0;
        let inner_ry = ry - 3.0;
        write!(
            buf,
            r#"<ellipse cx="{}" cy="{}" fill="{ATTR_BG}" rx="{}" ry="{}" style="stroke:{ATTR_BORDER};stroke-width:0.5;"/>"#,
            fmt_coord(cx), fmt_coord(cy), fmt_coord(inner_rx), fmt_coord(inner_ry),
        )
        .unwrap();
    } else {
        // Simple ellipse
        write!(
            buf,
            r#"<ellipse cx="{}" cy="{}" fill="{ATTR_BG}" rx="{}" ry="{}" style="stroke:{ATTR_BORDER};stroke-width:0.5;"/>"#,
            fmt_coord(cx), fmt_coord(cy), fmt_coord(rx), fmt_coord(ry),
        )
        .unwrap();
    }

    // Attribute label
    let escaped = xml_escape(&attr.label);
    let ty = cy + FONT_SIZE * 0.35;
    let text_w = rx * 2.0 - 10.0;
    let text_x = cx - rx + 5.0;

    if attr.is_key {
        write!(
            buf,
            r#"<text fill="{TEXT_FILL}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" text-decoration="underline" textLength="{}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(FONT_SIZE),
            fmt_coord(text_w),
            fmt_coord(text_x),
            fmt_coord(ty),
        )
        .unwrap();
    } else {
        write!(
            buf,
            r#"<text fill="{TEXT_FILL}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(FONT_SIZE),
            fmt_coord(text_w),
            fmt_coord(text_x),
            fmt_coord(ty),
        )
        .unwrap();
    }

    // Type annotation (if any)
    if let Some(ref type_label) = attr.type_label {
        let type_escaped = xml_escape(type_label);
        let type_y = cy + FONT_SIZE * 0.35 + FONT_SIZE + 2.0;
        write!(
            buf,
            r#"<text fill="{TEXT_FILL}" font-family="sans-serif" font-size="{}" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{type_escaped}</text>"#,
            fmt_coord(FONT_SIZE - 2.0),
            fmt_coord(text_w),
            fmt_coord(text_x),
            fmt_coord(type_y),
        )
        .unwrap();
    }

    // Render children recursively
    for child in &attr.children {
        render_attribute(buf, child);
        render_path_line(buf, cx, cy - ry, child.x, child.y + child.ry);
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
        let parent_center = entities
            .iter()
            .chain(relationships.iter())
            .find(|n| n.id == attr.parent)
            .map(|n| (n.x + n.width / 2.0, n.y + n.height / 2.0));

        if let Some((px, py)) = parent_center {
            render_path_line(buf, attr.x, attr.y, px, py);
        }
    }
}

// ── Edge rendering ──────────────────────────────────────────────────

fn render_edge(buf: &mut String, edge: &ErdEdgeLayout) {
    let (x1, y1) = edge.from_point;
    let (x2, y2) = edge.to_point;

    if edge.is_double {
        // Double line: two parallel lines offset by 1.5px
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.001 {
            let nx = -dy / len * 1.5;
            let ny = dx / len * 1.5;
            render_path_line(buf, x1 + nx, y1 + ny, x2 + nx, y2 + ny);
            render_path_line(buf, x1 - nx, y1 - ny, x2 - nx, y2 - ny);
        }
    } else {
        render_path_line(buf, x1, y1, x2, y2);
    }

    // Inline arrow polygon at the target end
    if edge.is_double {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.001 {
            let ux = dx / len;
            let uy = dy / len;
            let ax = x2 - ux * 8.0;
            let ay = y2 - uy * 8.0;
            let nx = -uy * 4.0;
            let ny = ux * 4.0;
            write!(
                buf,
                r#"<polygon fill="{EDGE_COLOR}" points="{} {} {}"/>"#,
                fmt_pt(x2, y2),
                fmt_pt(ax + nx, ay + ny),
                fmt_pt(ax - nx, ay - ny),
            )
            .unwrap();
        }
    }

    // Cardinality label near the midpoint
    if !edge.label.is_empty() {
        let mx = (x1 + x2) / 2.0;
        let my = (y1 + y2) / 2.0 - 6.0;
        let escaped = xml_escape(&edge.label);
        let text_w = escaped.len() as f64 * 7.0;
        write!(
            buf,
            r#"<text fill="{TEXT_FILL}" font-family="sans-serif" font-size="11" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{escaped}</text>"#,
            fmt_coord(text_w),
            fmt_coord(mx),
            fmt_coord(my),
        )
        .unwrap();
    }
}

// ── ISA rendering ───────────────────────────────────────────────────

fn render_isa(buf: &mut String, isa: &ErdIsaLayout) {
    let (cx, cy) = isa.triangle_center;
    let s = isa.triangle_size;

    let top_y = cy - s * 0.5;
    let bot_y = cy + s * 0.5;
    let left_x = cx - s * 0.6;
    let right_x = cx + s * 0.6;

    let points = format!(
        "{} {} {}",
        fmt_pt(cx, top_y),
        fmt_pt(right_x, bot_y),
        fmt_pt(left_x, bot_y)
    );

    write!(
        buf,
        r#"<polygon fill="{ISA_BG}" points="{points}" style="stroke:{ISA_BORDER};stroke-width:0.5;"/>"#,
    )
    .unwrap();

    // Kind label (d or U) inside triangle
    let label_y = cy + FONT_SIZE * 0.2;
    let escaped = xml_escape(&isa.kind_label);
    let text_w = escaped.len() as f64 * 7.0;
    write!(
        buf,
        r#"<text fill="{TEXT_FILL}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{escaped}</text>"#,
        fmt_coord(FONT_SIZE),
        fmt_coord(text_w),
        fmt_coord(cx - text_w / 2.0),
        fmt_coord(label_y),
    )
    .unwrap();

    // Path from parent to triangle top
    let (ppx, ppy) = isa.parent_point;
    render_path_line(buf, ppx, ppy, cx, top_y);

    // Paths from triangle bottom to children
    for (_child_id, (child_x, child_y)) in &isa.child_points {
        render_path_line(buf, cx, bot_y, *child_x, *child_y);
    }
}

// ── Note rendering ──────────────────────────────────────────────────

fn render_note(buf: &mut String, note: &ErdNoteLayout) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        write!(
            buf,
            r#"<line style="stroke:{NOTE_BORDER};stroke-width:1;stroke-dasharray:5,3;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(x1),
            fmt_coord(x2),
            fmt_coord(y1),
            fmt_coord(y2),
        )
        .unwrap();
    }

    let x = note.x;
    let y = note.y;
    let w = note.width;
    let h = note.height;
    let fold = NOTE_FOLD;
    write!(
        buf,
        r#"<polygon fill="{NOTE_BG}" points="{},{} {},{} {},{} {},{} {},{}" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(x), fmt_coord(y),
        fmt_coord(x + w - fold), fmt_coord(y),
        fmt_coord(x + w), fmt_coord(y + fold),
        fmt_coord(x + w), fmt_coord(y + h),
        fmt_coord(x), fmt_coord(y + h),
    )
    .unwrap();

    write!(
        buf,
        r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#,
        fmt_coord(x + w - fold), fmt_coord(y),
        fmt_coord(x + w - fold), fmt_coord(y + fold),
        fmt_coord(x + w), fmt_coord(y + fold),
    )
    .unwrap();

    let text_x = x + 10.0;
    let start_y = y + 10.0 + FONT_SIZE;
    render_creole_text(
        buf,
        &note.text,
        text_x,
        start_y,
        16.0,
        TEXT_FILL,
        None,
        r#"font-size="13""#,
    );
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
            svg.contains(r#"lengthAdjust="spacing""#),
            "entity text must use lengthAdjust"
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
        assert!(svg.contains("<path"), "edge must produce a path");
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
        let path_count = svg.matches("<path").count();
        assert!(path_count >= 2, "double edge must produce at least 2 paths");
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
        // Triangle + paths: parent-to-top + 2 bottom-to-children = 3 paths
        let path_count = svg.matches("<path").count();
        assert!(
            path_count >= 3,
            "ISA must have paths to parent and children, got {}",
            path_count
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
        // 2 paths connecting attributes to entity center
        let path_count = svg.matches("<path").count();
        assert!(
            path_count >= 2,
            "must have attribute-to-parent paths, got {}",
            path_count
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
        assert!(svg.contains("width=\"500px\""), "width must match");
        assert!(svg.contains("height=\"400px\""), "height must match");
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
