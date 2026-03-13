//! ERD (Chen notation) layout engine.
//!
//! Converts an `ErdDiagram` into a fully positioned `ErdLayout` ready for SVG
//! rendering.  Uses a self-layout (no Graphviz) approach: entities and
//! relationships are placed in a grid, attributes are arranged radially around
//! their parent, and edges connect linked elements.

use std::collections::HashMap;

use log::debug;

use crate::model::erd::{ErdAttribute, ErdDiagram, ErdDirection, ErdIsa};
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned ERD ready for rendering.
#[derive(Debug)]
pub struct ErdLayout {
    pub entity_nodes: Vec<ErdNodeLayout>,
    pub relationship_nodes: Vec<ErdNodeLayout>,
    pub attribute_nodes: Vec<ErdAttrLayout>,
    pub edges: Vec<ErdEdgeLayout>,
    pub isa_layouts: Vec<ErdIsaLayout>,
    pub notes: Vec<ErdNoteLayout>,
    pub width: f64,
    pub height: f64,
}

/// A positioned entity or relationship node.
#[derive(Debug, Clone)]
pub struct ErdNodeLayout {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub is_weak: bool,
    pub is_identifying: bool,
}

/// A positioned attribute ellipse.
#[derive(Debug, Clone)]
pub struct ErdAttrLayout {
    pub id: String,
    pub label: String,
    pub parent: String,
    pub x: f64,
    pub y: f64,
    pub rx: f64,
    pub ry: f64,
    pub is_key: bool,
    pub is_derived: bool,
    pub is_multi: bool,
    pub has_type: bool,
    pub type_label: Option<String>,
    /// Sub-attributes for nested attributes
    pub children: Vec<ErdAttrLayout>,
}

/// An edge connecting two positioned elements.
#[derive(Debug, Clone)]
pub struct ErdEdgeLayout {
    pub from_id: String,
    pub to_id: String,
    pub from_point: (f64, f64),
    pub to_point: (f64, f64),
    pub label: String,
    pub is_double: bool,
}

/// A positioned note annotation.
#[derive(Debug, Clone)]
pub struct ErdNoteLayout {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub lines: Vec<String>,
    pub connector: Option<(f64, f64, f64, f64)>,
}

/// A positioned ISA triangle.
#[derive(Debug, Clone)]
pub struct ErdIsaLayout {
    pub parent_id: String,
    pub kind_label: String,
    pub triangle_center: (f64, f64),
    pub triangle_size: f64,
    pub parent_point: (f64, f64),
    pub child_points: Vec<(String, (f64, f64))>,
    pub is_double: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CHAR_WIDTH: f64 = 7.2;
const ENTITY_PADDING: f64 = 16.0;
const ENTITY_MIN_WIDTH: f64 = 80.0;
const ENTITY_HEIGHT: f64 = 36.0;
const RELATIONSHIP_PADDING: f64 = 20.0;
const RELATIONSHIP_MIN_WIDTH: f64 = 80.0;
const RELATIONSHIP_HEIGHT: f64 = 40.0;
const ATTR_RX: f64 = 40.0;
const ATTR_RY: f64 = 16.0;
const ATTR_SPACING: f64 = 70.0;
const NODE_SPACING_H: f64 = 180.0;
const NODE_SPACING_V: f64 = 160.0;
const ATTR_DISTANCE: f64 = 80.0;
const MARGIN: f64 = 40.0;
const ISA_TRIANGLE_SIZE: f64 = 24.0;
const ISA_CHILD_SPACING: f64 = 140.0;
const NOTE_PADDING: f64 = 10.0;
const NOTE_LINE_HEIGHT: f64 = 16.0;
const NOTE_GAP: f64 = 16.0;

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

fn text_width(text: &str) -> f64 {
    text.len() as f64 * CHAR_WIDTH
}

fn entity_width(name: &str) -> f64 {
    (text_width(name) + 2.0 * ENTITY_PADDING).max(ENTITY_MIN_WIDTH)
}

fn relationship_width(name: &str) -> f64 {
    (text_width(name) + 2.0 * RELATIONSHIP_PADDING).max(RELATIONSHIP_MIN_WIDTH)
}

// ---------------------------------------------------------------------------
// Core layout
// ---------------------------------------------------------------------------

/// Perform the complete layout of an ERD.
pub fn layout_erd(diagram: &ErdDiagram) -> Result<ErdLayout> {
    debug!(
        "layout_erd: {} entities, {} relationships, {} links, {} ISAs, direction={:?}",
        diagram.entities.len(),
        diagram.relationships.len(),
        diagram.links.len(),
        diagram.isas.len(),
        diagram.direction
    );

    let is_lr = diagram.direction == ErdDirection::LeftToRight;

    // Collect all node IDs and compute sizes
    let mut node_sizes: HashMap<String, (f64, f64)> = HashMap::new();

    for e in &diagram.entities {
        let w = entity_width(&e.name);
        node_sizes.insert(e.id.clone(), (w, ENTITY_HEIGHT));
    }
    for r in &diagram.relationships {
        let w = relationship_width(&r.name);
        node_sizes.insert(r.id.clone(), (w, RELATIONSHIP_HEIGHT));
    }

    // Place nodes in a vertical or horizontal list.
    // Order: entities first, then relationships (interleaved if linked).
    // For simplicity, arrange them in declaration order.

    let all_ids: Vec<&str> = diagram
        .entities
        .iter()
        .map(|e| e.id.as_str())
        .chain(diagram.relationships.iter().map(|r| r.id.as_str()))
        .collect();

    let mut positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new(); // id -> (x, y, w, h)

    let mut cursor_main = MARGIN;
    let attr_band = ATTR_DISTANCE + ATTR_RY + 20.0;
    let cross_offset = MARGIN + attr_band;

    for id in &all_ids {
        let (w, h) = node_sizes
            .get(*id)
            .copied()
            .unwrap_or((ENTITY_MIN_WIDTH, ENTITY_HEIGHT));
        if is_lr {
            let x = cursor_main;
            let y = cross_offset;
            positions.insert(id.to_string(), (x, y, w, h));
            cursor_main += w + NODE_SPACING_H;
        } else {
            let x = cross_offset;
            let y = cursor_main;
            positions.insert(id.to_string(), (x, y, w, h));
            cursor_main += h + NODE_SPACING_V;
        }
    }

    // Build entity node layouts
    let entity_nodes: Vec<ErdNodeLayout> = diagram
        .entities
        .iter()
        .filter_map(|e| {
            let (x, y, w, h) = positions.get(&e.id).copied()?;
            Some(ErdNodeLayout {
                id: e.id.clone(),
                label: e.name.clone(),
                x,
                y,
                width: w,
                height: h,
                is_weak: e.is_weak,
                is_identifying: false,
            })
        })
        .collect();

    // Build relationship node layouts
    let relationship_nodes: Vec<ErdNodeLayout> = diagram
        .relationships
        .iter()
        .filter_map(|r| {
            let (x, y, w, h) = positions.get(&r.id).copied()?;
            Some(ErdNodeLayout {
                id: r.id.clone(),
                label: r.name.clone(),
                x,
                y,
                width: w,
                height: h,
                is_weak: false,
                is_identifying: r.is_identifying,
            })
        })
        .collect();

    // Layout attributes around their parent nodes
    let mut attribute_nodes: Vec<ErdAttrLayout> = Vec::new();
    let mut attr_idx = 0;

    for e in &diagram.entities {
        if let Some(&(px, py, pw, ph)) = positions.get(&e.id) {
            let parent_cx = px + pw / 2.0;
            let parent_cy = py + ph / 2.0;
            layout_attributes(
                &e.attributes,
                &e.id,
                parent_cx,
                parent_cy,
                is_lr,
                &mut attribute_nodes,
                &mut attr_idx,
            );
        }
    }

    for r in &diagram.relationships {
        if let Some(&(rx, ry, rw, rh)) = positions.get(&r.id) {
            let parent_cx = rx + rw / 2.0;
            let parent_cy = ry + rh / 2.0;
            layout_attributes(
                &r.attributes,
                &r.id,
                parent_cx,
                parent_cy,
                is_lr,
                &mut attribute_nodes,
                &mut attr_idx,
            );
        }
    }

    // Layout edges (links)
    let edges = layout_edges(&diagram.links, &positions);

    // Layout ISAs
    let isa_layouts = layout_isas(&diagram.isas, &positions, is_lr);

    // Compute bounding box
    let mut max_x = 0.0_f64;
    let mut max_y = 0.0_f64;

    for node in entity_nodes.iter().chain(relationship_nodes.iter()) {
        max_x = max_x.max(node.x + node.width);
        max_y = max_y.max(node.y + node.height);
    }

    for attr in &attribute_nodes {
        max_x = max_x.max(attr.x + attr.rx);
        max_y = max_y.max(attr.y + attr.ry);
    }

    for edge in &edges {
        max_x = max_x.max(edge.from_point.0).max(edge.to_point.0);
        max_y = max_y.max(edge.from_point.1).max(edge.to_point.1);
    }

    for isa in &isa_layouts {
        let (tx, ty) = isa.triangle_center;
        max_x = max_x.max(tx + isa.triangle_size);
        max_y = max_y.max(ty + isa.triangle_size);
        for (_, (cx, cy)) in &isa.child_points {
            max_x = max_x.max(*cx);
            max_y = max_y.max(*cy);
        }
    }

    let notes = layout_notes(&diagram.notes, &positions, max_x, max_y);

    for note in &notes {
        max_x = max_x.max(note.x + note.width);
        max_y = max_y.max(note.y + note.height);
        if let Some((x1, y1, x2, y2)) = note.connector {
            max_x = max_x.max(x1).max(x2);
            max_y = max_y.max(y1).max(y2);
        }
    }

    let width = max_x + MARGIN;
    let height = max_y + MARGIN;

    debug!(
        "layout_erd done: {:.0}x{:.0}, {} entities, {} relationships, {} attrs, {} edges, {} ISAs, {} notes",
        width,
        height,
        entity_nodes.len(),
        relationship_nodes.len(),
        attribute_nodes.len(),
        edges.len(),
        isa_layouts.len(),
        notes.len()
    );

    Ok(ErdLayout {
        entity_nodes,
        relationship_nodes,
        attribute_nodes,
        edges,
        isa_layouts,
        notes,
        width,
        height,
    })
}

// ---------------------------------------------------------------------------
// Attribute layout
// ---------------------------------------------------------------------------

fn layout_attributes(
    attrs: &[ErdAttribute],
    parent_id: &str,
    parent_cx: f64,
    parent_cy: f64,
    is_lr: bool,
    out: &mut Vec<ErdAttrLayout>,
    idx: &mut usize,
) {
    if attrs.is_empty() {
        return;
    }

    let count = attrs.len() as f64;

    for (i, attr) in attrs.iter().enumerate() {
        let attr_id = format!("{}__attr_{}", parent_id, *idx);
        *idx += 1;

        // Position attributes above/below (for TB) or left/right (for LR)
        let (ax, ay) = if is_lr {
            // Spread vertically above the parent
            let total_span = (count - 1.0) * ATTR_SPACING;
            let start_y = parent_cy - total_span / 2.0;
            let y = start_y + i as f64 * ATTR_SPACING;
            let x = parent_cx;
            (x, y - ATTR_DISTANCE)
        } else {
            // Spread horizontally to the left of the parent
            let total_span = (count - 1.0) * ATTR_SPACING;
            let start_x = parent_cx - total_span / 2.0;
            let x = start_x + i as f64 * ATTR_SPACING;
            let y = parent_cy;
            (x, y - ATTR_DISTANCE)
        };

        let display = attr.display_name.as_deref().unwrap_or(&attr.name);
        let label_w = text_width(display);
        let rx = (label_w / 2.0 + 10.0).max(ATTR_RX);

        // Layout children of nested attribute
        let mut child_layouts = Vec::new();
        if !attr.children.is_empty() {
            let child_count = attr.children.len() as f64;
            let child_span = (child_count - 1.0) * (ATTR_SPACING * 0.7);

            for (ci, child) in attr.children.iter().enumerate() {
                let child_id = format!("{}__attr_{}", parent_id, *idx);
                *idx += 1;
                let cx = if is_lr {
                    ax
                } else {
                    ax - child_span / 2.0 + ci as f64 * (ATTR_SPACING * 0.7)
                };
                let cy = ay - ATTR_DISTANCE * 0.6;

                let child_display = child.display_name.as_deref().unwrap_or(&child.name);
                let child_rx = (text_width(child_display) / 2.0 + 10.0).max(ATTR_RX * 0.8);

                child_layouts.push(ErdAttrLayout {
                    id: child_id,
                    label: child_display.to_string(),
                    parent: attr_id.clone(),
                    x: cx,
                    y: cy,
                    rx: child_rx,
                    ry: ATTR_RY * 0.8,
                    is_key: child.is_key,
                    is_derived: child.is_derived,
                    is_multi: child.is_multi,
                    has_type: child.attr_type.is_some(),
                    type_label: child.attr_type.clone(),
                    children: Vec::new(),
                });
            }
        }

        out.push(ErdAttrLayout {
            id: attr_id,
            label: display.to_string(),
            parent: parent_id.to_string(),
            x: ax,
            y: ay,
            rx,
            ry: ATTR_RY,
            is_key: attr.is_key,
            is_derived: attr.is_derived,
            is_multi: attr.is_multi,
            has_type: attr.attr_type.is_some(),
            type_label: attr.attr_type.clone(),
            children: child_layouts,
        });
    }
}

// ---------------------------------------------------------------------------
// Edge layout
// ---------------------------------------------------------------------------

fn layout_edges(
    links: &[crate::model::erd::ErdLink],
    positions: &HashMap<String, (f64, f64, f64, f64)>,
) -> Vec<ErdEdgeLayout> {
    let mut edges = Vec::new();

    for link in links {
        let from_pos = positions.get(&link.from);
        let to_pos = positions.get(&link.to);

        let (from_x, from_y, from_w, from_h) = if let Some(p) = from_pos {
            *p
        } else {
            log::warn!("link source '{}' not found in layout", link.from);
            continue;
        };
        let (to_x, to_y, to_w, to_h) = if let Some(p) = to_pos {
            *p
        } else {
            log::warn!("link target '{}' not found in layout", link.to);
            continue;
        };

        let from_cx = from_x + from_w / 2.0;
        let from_cy = from_y + from_h / 2.0;
        let to_cx = to_x + to_w / 2.0;
        let to_cy = to_y + to_h / 2.0;

        // Connect centers, clipping to node boundary
        let from_point = clip_to_rect(from_cx, from_cy, from_w, from_h, to_cx, to_cy);
        let to_point = clip_to_rect(to_cx, to_cy, to_w, to_h, from_cx, from_cy);

        edges.push(ErdEdgeLayout {
            from_id: link.from.clone(),
            to_id: link.to.clone(),
            from_point,
            to_point,
            label: link.cardinality.clone(),
            is_double: link.is_double,
        });
    }

    edges
}

/// Clip a line from (cx, cy) to (target_x, target_y) to the rectangle boundary.
fn clip_to_rect(cx: f64, cy: f64, w: f64, h: f64, target_x: f64, target_y: f64) -> (f64, f64) {
    let dx = target_x - cx;
    let dy = target_y - cy;

    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return (cx, cy);
    }

    let half_w = w / 2.0;
    let half_h = h / 2.0;

    // Calculate intersection with rectangle edges
    let mut t = f64::MAX;

    if dx.abs() > 0.001 {
        let tx = if dx > 0.0 { half_w / dx } else { -half_w / dx };
        if tx > 0.0 && tx < t {
            t = tx;
        }
    }
    if dy.abs() > 0.001 {
        let ty = if dy > 0.0 { half_h / dy } else { -half_h / dy };
        if ty > 0.0 && ty < t {
            t = ty;
        }
    }

    if t == f64::MAX {
        (cx, cy)
    } else {
        (cx + dx * t, cy + dy * t)
    }
}

// ---------------------------------------------------------------------------
// ISA layout
// ---------------------------------------------------------------------------

fn layout_isas(
    isas: &[ErdIsa],
    positions: &HashMap<String, (f64, f64, f64, f64)>,
    is_lr: bool,
) -> Vec<ErdIsaLayout> {
    let mut result = Vec::new();

    for isa in isas {
        let parent_pos = if let Some(p) = positions.get(&isa.parent) {
            *p
        } else {
            log::warn!("ISA parent '{}' not found", isa.parent);
            continue;
        };

        let (px, py, pw, ph) = parent_pos;
        let parent_cx = px + pw / 2.0;
        let parent_cy = py + ph / 2.0;

        // Place ISA triangle below (TB) or to the right (LR) of parent
        let (tri_x, tri_y) = if is_lr {
            (parent_cx + pw / 2.0 + 50.0, parent_cy)
        } else {
            (parent_cx, parent_cy + ph / 2.0 + 50.0)
        };

        let kind_label = match isa.kind {
            crate::model::erd::IsaKind::Disjoint => "d".to_string(),
            crate::model::erd::IsaKind::Union => "U".to_string(),
        };

        // Place children spread out below the triangle
        let child_count = isa.children.len() as f64;
        let total_span = (child_count - 1.0) * ISA_CHILD_SPACING;
        let parent_point = if is_lr {
            (tri_x - ISA_TRIANGLE_SIZE, tri_y)
        } else {
            (tri_x, tri_y - ISA_TRIANGLE_SIZE)
        };

        let mut child_points = Vec::new();
        for (ci, child_id) in isa.children.iter().enumerate() {
            let (cx, cy) = if is_lr {
                let x = tri_x + ISA_TRIANGLE_SIZE + 30.0;
                let y = tri_y - total_span / 2.0 + ci as f64 * ISA_CHILD_SPACING;
                (x, y)
            } else {
                let x = tri_x - total_span / 2.0 + ci as f64 * ISA_CHILD_SPACING;
                let y = tri_y + ISA_TRIANGLE_SIZE + 30.0;
                (x, y)
            };
            child_points.push((child_id.clone(), (cx, cy)));
        }

        result.push(ErdIsaLayout {
            parent_id: isa.parent.clone(),
            kind_label,
            triangle_center: (tri_x, tri_y),
            triangle_size: ISA_TRIANGLE_SIZE,
            parent_point,
            child_points,
            is_double: isa.is_double,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Note layout
// ---------------------------------------------------------------------------

fn estimate_note_size(text: &str) -> (f64, f64, Vec<String>) {
    let lines: Vec<String> = text.lines().map(std::string::ToString::to_string).collect();
    let line_refs: Vec<&str> = if lines.is_empty() {
        vec![""]
    } else {
        lines.iter().map(String::as_str).collect()
    };
    let max_width = line_refs
        .iter()
        .map(|line| text_width(line))
        .fold(0.0_f64, f64::max);
    let width = max_width + NOTE_PADDING * 2.0;
    let height = line_refs.len() as f64 * NOTE_LINE_HEIGHT + NOTE_PADDING * 2.0;
    let lines = if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    };
    (width.max(80.0), height.max(36.0), lines)
}

fn layout_notes(
    notes: &[crate::model::erd::ErdNote],
    positions: &HashMap<String, (f64, f64, f64, f64)>,
    base_max_x: f64,
    base_max_y: f64,
) -> Vec<ErdNoteLayout> {
    let mut result = Vec::new();
    let mut floating_y = MARGIN;

    for note in notes {
        let (width, height, lines) = estimate_note_size(&note.text);

        let (x, y, connector) = if let Some(target) = note.target.as_ref() {
            if let Some(&(tx, ty, tw, th)) = positions.get(target) {
                match note.position.as_str() {
                    "left" => (
                        tx - width - NOTE_GAP,
                        ty,
                        Some((tx - NOTE_GAP, ty + th / 2.0, tx, ty + th / 2.0)),
                    ),
                    "top" => (
                        tx + (tw - width) / 2.0,
                        ty - height - NOTE_GAP,
                        Some((tx + tw / 2.0, ty - NOTE_GAP, tx + tw / 2.0, ty)),
                    ),
                    "bottom" => (
                        tx + (tw - width) / 2.0,
                        ty + th + NOTE_GAP,
                        Some((tx + tw / 2.0, ty + th, tx + tw / 2.0, ty + th + NOTE_GAP)),
                    ),
                    _ => (
                        tx + tw + NOTE_GAP,
                        ty,
                        Some((tx + tw, ty + th / 2.0, tx + tw + NOTE_GAP, ty + th / 2.0)),
                    ),
                }
            } else {
                let x = base_max_x + NOTE_GAP;
                let y = floating_y;
                floating_y += height + NOTE_GAP;
                (x, y, None)
            }
        } else {
            let x = match note.position.as_str() {
                "left" => MARGIN,
                _ => base_max_x + NOTE_GAP,
            };
            let y = if note.position == "bottom" {
                base_max_y + NOTE_GAP + (floating_y - MARGIN)
            } else {
                floating_y
            };
            floating_y += height + NOTE_GAP;
            (x, y, None)
        };

        result.push(ErdNoteLayout {
            text: note.text.clone(),
            x,
            y,
            width,
            height,
            lines,
            connector,
        });
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::erd::*;

    fn empty_diagram() -> ErdDiagram {
        ErdDiagram {
            entities: vec![],
            relationships: vec![],
            links: vec![],
            isas: vec![],
            direction: ErdDirection::TopToBottom,
            notes: vec![],
        }
    }

    fn simple_entity(name: &str) -> ErdEntity {
        ErdEntity {
            id: name.to_string(),
            name: name.to_string(),
            attributes: vec![],
            is_weak: false,
            color: None,
        }
    }

    fn simple_relationship(name: &str) -> ErdRelationship {
        ErdRelationship {
            id: name.to_string(),
            name: name.to_string(),
            attributes: vec![],
            is_identifying: false,
            color: None,
        }
    }

    fn simple_attr(name: &str) -> ErdAttribute {
        ErdAttribute {
            name: name.to_string(),
            display_name: None,
            is_key: false,
            is_derived: false,
            is_multi: false,
            attr_type: None,
            children: vec![],
            color: None,
        }
    }

    fn simple_link(from: &str, to: &str, card: &str) -> ErdLink {
        ErdLink {
            from: from.to_string(),
            to: to.to_string(),
            cardinality: card.to_string(),
            is_double: false,
            color: None,
        }
    }

    // 1. Empty diagram
    #[test]
    fn test_empty_diagram() {
        let d = empty_diagram();
        let layout = layout_erd(&d).unwrap();
        assert!(layout.entity_nodes.is_empty());
        assert!(layout.relationship_nodes.is_empty());
        assert!(layout.attribute_nodes.is_empty());
        assert!(layout.edges.is_empty());
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 2. Single entity
    #[test]
    fn test_single_entity() {
        let d = ErdDiagram {
            entities: vec![simple_entity("MOVIE")],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.entity_nodes.len(), 1);
        let node = &layout.entity_nodes[0];
        assert_eq!(node.id, "MOVIE");
        assert!(node.width >= ENTITY_MIN_WIDTH);
        assert_eq!(node.height, ENTITY_HEIGHT);
    }

    // 3. Entity with attributes
    #[test]
    fn test_entity_with_attributes() {
        let d = ErdDiagram {
            entities: vec![ErdEntity {
                id: "CUSTOMER".to_string(),
                name: "CUSTOMER".to_string(),
                attributes: vec![
                    ErdAttribute {
                        is_key: true,
                        ..simple_attr("Number")
                    },
                    simple_attr("Name"),
                ],
                is_weak: false,
                color: None,
            }],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.attribute_nodes.len(), 2);
        assert!(layout.attribute_nodes[0].is_key);
        assert_eq!(layout.attribute_nodes[0].parent, "CUSTOMER");
    }

    // 4. Single relationship
    #[test]
    fn test_single_relationship() {
        let d = ErdDiagram {
            relationships: vec![simple_relationship("RENTED_TO")],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.relationship_nodes.len(), 1);
        assert_eq!(layout.relationship_nodes[0].id, "RENTED_TO");
    }

    // 5. Edges between nodes
    #[test]
    fn test_edges() {
        let d = ErdDiagram {
            entities: vec![simple_entity("CUSTOMER")],
            relationships: vec![simple_relationship("RENTED_TO")],
            links: vec![simple_link("RENTED_TO", "CUSTOMER", "1")],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.edges.len(), 1);
        assert_eq!(layout.edges[0].from_id, "RENTED_TO");
        assert_eq!(layout.edges[0].to_id, "CUSTOMER");
        assert_eq!(layout.edges[0].label, "1");
    }

    // 6. Multiple entities ordered
    #[test]
    fn test_multiple_entities_vertical() {
        let d = ErdDiagram {
            entities: vec![simple_entity("A"), simple_entity("B"), simple_entity("C")],
            direction: ErdDirection::TopToBottom,
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.entity_nodes.len(), 3);
        let y0 = layout.entity_nodes[0].y;
        let y1 = layout.entity_nodes[1].y;
        let y2 = layout.entity_nodes[2].y;
        assert!(y0 < y1, "first should be above second: {} < {}", y0, y1);
        assert!(y1 < y2, "second should be above third: {} < {}", y1, y2);
    }

    // 7. Left-to-right direction
    #[test]
    fn test_left_to_right_direction() {
        let d = ErdDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            direction: ErdDirection::LeftToRight,
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        let x0 = layout.entity_nodes[0].x;
        let x1 = layout.entity_nodes[1].x;
        assert!(x0 < x1, "first should be left of second: {} < {}", x0, x1);
    }

    // 8. Weak entity flag preserved
    #[test]
    fn test_weak_entity() {
        let d = ErdDiagram {
            entities: vec![ErdEntity {
                is_weak: true,
                ..simple_entity("CHILD")
            }],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert!(layout.entity_nodes[0].is_weak);
    }

    // 9. Identifying relationship flag preserved
    #[test]
    fn test_identifying_relationship() {
        let d = ErdDiagram {
            relationships: vec![ErdRelationship {
                is_identifying: true,
                ..simple_relationship("PARENT_OF")
            }],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert!(layout.relationship_nodes[0].is_identifying);
    }

    // 10. Bounding box includes all elements
    #[test]
    fn test_bounding_box() {
        let d = ErdDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            relationships: vec![simple_relationship("R")],
            links: vec![simple_link("R", "A", "1"), simple_link("R", "B", "N")],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();

        for node in layout
            .entity_nodes
            .iter()
            .chain(layout.relationship_nodes.iter())
        {
            assert!(
                node.x + node.width <= layout.width,
                "node right {} exceeds width {}",
                node.x + node.width,
                layout.width
            );
            assert!(
                node.y + node.height <= layout.height,
                "node bottom {} exceeds height {}",
                node.y + node.height,
                layout.height
            );
        }
    }

    // 11. Nested attributes have children
    #[test]
    fn test_nested_attributes() {
        let d = ErdDiagram {
            entities: vec![ErdEntity {
                id: "DIR".to_string(),
                name: "DIRECTOR".to_string(),
                attributes: vec![ErdAttribute {
                    name: "Name".to_string(),
                    display_name: None,
                    is_key: false,
                    is_derived: false,
                    is_multi: false,
                    attr_type: None,
                    children: vec![simple_attr("Fname"), simple_attr("Lname")],
                    color: None,
                }],
                is_weak: false,
                color: None,
            }],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.attribute_nodes.len(), 1);
        assert_eq!(layout.attribute_nodes[0].children.len(), 2);
    }

    // 12. Edge double-line flag
    #[test]
    fn test_double_edge() {
        let d = ErdDiagram {
            entities: vec![simple_entity("A")],
            relationships: vec![simple_relationship("R")],
            links: vec![ErdLink {
                from: "R".to_string(),
                to: "A".to_string(),
                cardinality: "N".to_string(),
                is_double: true,
                color: None,
            }],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert!(layout.edges[0].is_double);
    }

    // 13. Clip to rect - target directly below
    #[test]
    fn test_clip_to_rect_below() {
        let (x, y) = clip_to_rect(100.0, 100.0, 80.0, 40.0, 100.0, 200.0);
        assert!((x - 100.0).abs() < 1.0);
        assert!((y - 120.0).abs() < 1.0);
    }

    // 14. Clip to rect - target to the right
    #[test]
    fn test_clip_to_rect_right() {
        let (x, y) = clip_to_rect(100.0, 100.0, 80.0, 40.0, 300.0, 100.0);
        assert!((x - 140.0).abs() < 1.0);
        assert!((y - 100.0).abs() < 1.0);
    }

    // 15. ISA layout
    #[test]
    fn test_isa_layout() {
        let d = ErdDiagram {
            entities: vec![
                simple_entity("PARENT"),
                simple_entity("CHILD1"),
                simple_entity("CHILD2"),
            ],
            isas: vec![ErdIsa {
                parent: "PARENT".to_string(),
                kind: IsaKind::Disjoint,
                children: vec!["CHILD1".to_string(), "CHILD2".to_string()],
                is_double: true,
                color: None,
            }],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.isa_layouts.len(), 1);
        assert_eq!(layout.isa_layouts[0].kind_label, "d");
        assert_eq!(layout.isa_layouts[0].child_points.len(), 2);
        assert!(layout.isa_layouts[0].is_double);
    }

    // 16. Attribute derived flag
    #[test]
    fn test_derived_attribute() {
        let d = ErdDiagram {
            entities: vec![ErdEntity {
                id: "E".to_string(),
                name: "E".to_string(),
                attributes: vec![ErdAttribute {
                    is_derived: true,
                    ..simple_attr("Bonus")
                }],
                is_weak: false,
                color: None,
            }],
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert!(layout.attribute_nodes[0].is_derived);
    }
}
