//! ERD (Chen notation) layout engine.
//!
//! Converts an `ErdDiagram` into a fully positioned `ErdLayout` ready for SVG
//! rendering.  Assigns ranks via BFS over the link graph so that connected
//! nodes form chains, then spreads each rank along the cross-axis.

use std::collections::HashMap;

use log::debug;

use crate::font_metrics;
use crate::layout::graphviz::{self, LayoutEdge, LayoutGraph, LayoutNode, RankDir};
use crate::model::erd::{ErdAttribute, ErdDiagram, ErdDirection, ErdIsa};
use crate::render::svg::{CANVAS_DELTA, DOC_MARGIN_BOTTOM, DOC_MARGIN_RIGHT};
use crate::svek::shape_type::ShapeType;
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
    pub from_name: String,
    pub to_name: String,
    pub from_point: (f64, f64),
    pub to_point: (f64, f64),
    pub label: String,
    pub is_double: bool,
    pub source_line: usize,
    pub entity_idx_from: usize,
    pub entity_idx_to: usize,
    /// Raw SVG path d-string from graphviz (via svek pipeline).
    pub raw_path_d: Option<String>,
    /// Label position from svek solve (x, y).
    pub label_xy: Option<(f64, f64)>,
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
// Constants – tuned to match Java PlantUML reference output
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 14.0;
const ENTITY_PADDING: f64 = 10.0;
const ENTITY_MIN_WIDTH: f64 = 0.0;
const ENTITY_HEIGHT: f64 = 36.2969;
/// Java MARGIN constant from IEntityImage (used for diamond calculation)
const JAVA_ENTITY_MARGIN: f64 = 5.0;
const ATTR_RY: f64 = 14.5236;
const ATTR_SPACING: f64 = 70.0;
/// Gap between nodes placed side-by-side in the same rank.
const RANK_NODE_GAP: f64 = 80.0;
/// Gap between consecutive ranks (edge of one rank to edge of the next).
const RANK_SEP: f64 = 140.0;
/// Distance from a parent node center to its attribute ellipse center.
const ATTR_DISTANCE: f64 = 80.0;
const MARGIN: f64 = 7.0;
const ISA_TRIANGLE_SIZE: f64 = 24.0;
const ISA_CHILD_SPACING: f64 = 140.0;
const NOTE_PADDING: f64 = 10.0;
const NOTE_LINE_HEIGHT: f64 = 16.0;
const NOTE_GAP: f64 = 16.0;

// ---------------------------------------------------------------------------
// Text measurement
// ---------------------------------------------------------------------------

fn text_width(text: &str) -> f64 {
    font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false)
}

fn entity_width(name: &str) -> f64 {
    (text_width(name) + 2.0 * ENTITY_PADDING).max(ENTITY_MIN_WIDTH)
}

/// Compute relationship diamond dimensions matching Java's ChenRelationship formula:
/// diagonal = (dimTitle.width + 2 * dimTitle.height) / sqrt(5) + 2 * MARGIN
/// totalWidth = diagonal * sqrt(5)
/// totalHeight = diagonal * sqrt(5) / 2
fn relationship_diamond_size(name: &str) -> (f64, f64) {
    let tw = text_width(name);
    let th = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);
    let diagonal = (tw + 2.0 * th) / 5.0_f64.sqrt() + 2.0 * JAVA_ENTITY_MARGIN;
    let total_w = diagonal * 5.0_f64.sqrt();
    let total_h = diagonal * 5.0_f64.sqrt() / 2.0;
    (total_w, total_h)
}

/// Compute attribute ellipse rx from label text.
fn attr_rx_for(label: &str) -> f64 {
    text_width(label) / 2.0 + 10.0
}

// ---------------------------------------------------------------------------
// Rank assignment (BFS over link graph)
// ---------------------------------------------------------------------------

/// Assign each node a rank based on link topology using BFS.
///
/// Uses link direction to find root nodes: nodes that appear as `from` in
/// links but never as `to` (or appear more as `from`) are treated as roots.
/// This matches Graphviz DOT behaviour where the first-mentioned node in
/// an edge is placed higher.
fn assign_ranks(
    all_ids: &[String],
    links: &[crate::model::erd::ErdLink],
    isas: &[ErdIsa],
) -> HashMap<String, usize> {
    use std::collections::{HashSet, VecDeque};

    // Build undirected adjacency for BFS traversal
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for id in all_ids {
        adj.entry(id.clone()).or_default();
    }
    for link in links {
        adj.entry(link.from.clone())
            .or_default()
            .push(link.to.clone());
        adj.entry(link.to.clone())
            .or_default()
            .push(link.from.clone());
    }
    for isa in isas {
        for child in &isa.children {
            adj.entry(isa.parent.clone())
                .or_default()
                .push(child.clone());
            adj.entry(child.clone())
                .or_default()
                .push(isa.parent.clone());
        }
    }

    // Count incoming edges (appear as `to` in links) to find root nodes.
    // ISA parent is treated as having incoming edges from children.
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for id in all_ids {
        in_degree.entry(id.clone()).or_insert(0);
    }
    for link in links {
        *in_degree.entry(link.to.clone()).or_insert(0) += 1;
    }
    for isa in isas {
        // ISA children are "below" the parent
        for child in &isa.children {
            *in_degree.entry(child.clone()).or_insert(0) += 1;
        }
    }

    // Order BFS starting nodes: prefer those with lowest in-degree (roots),
    // then by declaration order.
    let mut start_order: Vec<String> = all_ids.to_vec();
    start_order.sort_by_key(|id| in_degree.get(id).copied().unwrap_or(0));

    let mut ranks: HashMap<String, usize> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();

    for start in &start_order {
        if visited.contains(start) {
            continue;
        }
        let mut queue = VecDeque::new();
        queue.push_back((start.clone(), 0usize));
        visited.insert(start.clone());
        ranks.insert(start.clone(), 0);

        while let Some((node, rank)) = queue.pop_front() {
            if let Some(neighbors) = adj.get(&node) {
                for nb in neighbors {
                    if !visited.contains(nb) {
                        visited.insert(nb.clone());
                        ranks.insert(nb.clone(), rank + 1);
                        queue.push_back((nb.clone(), rank + 1));
                    }
                }
            }
        }
    }

    ranks
}

// ---------------------------------------------------------------------------
// Core layout
// ---------------------------------------------------------------------------

/// Perform the complete layout of an ERD using the svek/graphviz pipeline.
///
/// Java PlantUML routes ERD diagrams through the same graphviz DOT engine
/// as class diagrams. Entities become rect nodes, relationships become
/// diamond nodes, and links become edges with `minlen=2`.
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

    // Collect node sizes
    let mut node_sizes: HashMap<String, (f64, f64)> = HashMap::new();
    for e in &diagram.entities {
        let w = entity_width(&e.name);
        node_sizes.insert(e.id.clone(), (w, ENTITY_HEIGHT));
    }
    for r in &diagram.relationships {
        let (w, dh) = relationship_diamond_size(&r.name);
        node_sizes.insert(r.id.clone(), (w, dh));
    }

    // Build name lookup: id -> display name
    let mut name_map: HashMap<String, String> = HashMap::new();
    for e in &diagram.entities {
        name_map.insert(e.id.clone(), e.name.clone());
    }
    for r in &diagram.relationships {
        name_map.insert(r.id.clone(), r.name.clone());
    }

    // Entity index map for link metadata
    let mut entity_idx: HashMap<String, usize> = HashMap::new();
    for (i, e) in diagram.entities.iter().enumerate() {
        entity_idx.insert(e.id.clone(), i);
    }
    for (i, r) in diagram.relationships.iter().enumerate() {
        entity_idx.insert(r.id.clone(), diagram.entities.len() + i);
    }

    // Build svek layout nodes
    let rankdir = if is_lr { RankDir::LeftToRight } else { RankDir::TopToBottom };

    let mut layout_nodes: Vec<LayoutNode> = Vec::new();
    for e in &diagram.entities {
        let (w, h) = node_sizes[&e.id];
        layout_nodes.push(LayoutNode {
            id: e.id.clone(),
            label: e.name.clone(),
            width_pt: w,
            height_pt: h,
            shape: Some(ShapeType::Rectangle),
            shield: None,
            entity_position: None,
            max_label_width: None,
            order: None,
            image_width_pt: None,
            lf_extra_left: 0.0,
            lf_rect_correction: true,
                    lf_has_body_separator: false,
        });
    }
    for r in &diagram.relationships {
        let (w, h) = node_sizes[&r.id];
        layout_nodes.push(LayoutNode {
            id: r.id.clone(),
            label: r.name.clone(),
            width_pt: w,
            height_pt: h,
            shape: Some(ShapeType::Diamond),
            shield: None,
            entity_position: None,
            max_label_width: None,
            order: None,
            image_width_pt: None,
            lf_extra_left: 0.0,
            lf_rect_correction: true,
                    lf_has_body_separator: false,
        });
    }

    // Measure cardinality label dimensions for DOT edge label tables.
    // Java ERD uses font-size 11 for edge labels.
    let label_dims: Vec<(f64, f64)> = diagram
        .links
        .iter()
        .map(|link| {
            let tw = font_metrics::text_width(&link.cardinality, "SansSerif", 11.0, false, false);
            let th = font_metrics::line_height("SansSerif", 11.0, false, false);
            let dim_w = (tw + 2.0).floor();
            let dim_h = (th + 2.0).floor();
            (dim_w, dim_h)
        })
        .collect();

    let layout_edges: Vec<LayoutEdge> = diagram
        .links
        .iter()
        .enumerate()
        .map(|(i, link)| {
            let (lw, lh) = label_dims[i];
            LayoutEdge {
                from: link.from.clone(),
                to: link.to.clone(),
                label: Some(link.cardinality.clone()),
                label_dimension: Some((lw, lh)),
                tail_label: None,
                tail_label_boxed: false,
                head_label: None,
                head_label_boxed: false,
                tail_decoration: crate::svek::edge::LinkDecoration::None,
                head_decoration: crate::svek::edge::LinkDecoration::None,
                line_style: crate::svek::edge::LinkStyle::Normal,
                minlen: 2,
                invisible: false,
                no_constraint: false,
            }
        })
        .collect();

    let graph = LayoutGraph {
        nodes: layout_nodes,
        edges: layout_edges,
        clusters: vec![],
        rankdir,
        ranksep_override: None,
        use_simplier_dot_link_strategy: false,
    };

    let gl = graphviz::layout_with_svek(&graph)
        .map_err(|e| crate::error::Error::Layout(format!("ERD svek layout: {e}")))?;

    // Render offsets. Nodes and edge paths use -1 y correction because the
    // Rust svek LF simulation applies lf_rect_correction to y (which Java
    // doesn't for drawRectangle). Label positions use the unadjusted offset
    // because they go through the move_delta + normalize_offset formula.
    let render_dx = gl.render_offset.0;
    let render_dy = gl.render_offset.1 - 1.0;
    let render_dy_label = gl.render_offset.1;
    

    let mut positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    for nl in &gl.nodes {
        let x = nl.cx - nl.width / 2.0 + render_dx;
        let y = nl.cy - nl.height / 2.0 + render_dy;
        positions.insert(nl.id.clone(), (x, y, nl.width, nl.height));
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
                x, y, width: w, height: h,
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
                x, y, width: w, height: h,
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
                &e.attributes, &e.id, parent_cx, parent_cy, is_lr,
                &mut attribute_nodes, &mut attr_idx,
            );
        }
    }

    for r in &diagram.relationships {
        if let Some(&(rx, ry, rw, rh)) = positions.get(&r.id) {
            let parent_cx = rx + rw / 2.0;
            let parent_cy = ry + rh / 2.0;
            layout_attributes(
                &r.attributes, &r.id, parent_cx, parent_cy, is_lr,
                &mut attribute_nodes, &mut attr_idx,
            );
        }
    }

    // Layout edges from svek results
    let mut edges: Vec<ErdEdgeLayout> = Vec::new();
    for (li, link) in diagram.links.iter().enumerate() {
        let from_name = name_map.get(&link.from).cloned().unwrap_or(link.from.clone());
        let to_name = name_map.get(&link.to).cloned().unwrap_or(link.to.clone());
        let from_idx = entity_idx.get(&link.from).copied().unwrap_or(0);
        let to_idx = entity_idx.get(&link.to).copied().unwrap_or(0);

        let svek_edge = gl.edges.get(li);
        let raw_path_d = svek_edge
            .and_then(|e| e.raw_path_d.as_ref())
            .map(|d| shift_svg_path(d, render_dx, render_dy));
        // label_xy from svek is pre-normalized, pre-moveDelta.
        // Apply: label + move_delta - normalize_offset + render_offset
        let label_xy = svek_edge.and_then(|e| {
            let (lx, ly) = e.label_xy?;
            Some((
                lx + gl.move_delta.0 - gl.normalize_offset.0 + render_dx,
                ly + gl.move_delta.1 - gl.normalize_offset.1 + render_dy_label,
            ))
        });

        let (from_point, to_point) = if let (Some(fp), Some(tp)) =
            (positions.get(&link.from), positions.get(&link.to))
        {
            let (fx, fy, fw, fh) = *fp;
            let (tx, ty, tw, th) = *tp;
            let fc = (fx + fw / 2.0, fy + fh / 2.0);
            let tc = (tx + tw / 2.0, ty + th / 2.0);
            (
                clip_to_rect(fc.0, fc.1, fw, fh, tc.0, tc.1),
                clip_to_rect(tc.0, tc.1, tw, th, fc.0, fc.1),
            )
        } else {
            ((0.0, 0.0), (0.0, 0.0))
        };

        edges.push(ErdEdgeLayout {
            from_id: link.from.clone(),
            to_id: link.to.clone(),
            from_name, to_name,
            from_point, to_point,
            label: link.cardinality.clone(),
            is_double: link.is_double,
            source_line: 0,
            entity_idx_from: from_idx,
            entity_idx_to: to_idx,
            raw_path_d,
            label_xy,
        });
    }

    // Layout ISAs
    let isa_layouts = layout_isas(&diagram.isas, &positions, is_lr);

    // Viewport: use svek lf_span + CANVAS_DELTA + DOC_MARGIN (same as class/component)
    let is_degenerated = entity_nodes.len() + relationship_nodes.len() <= 1 && edges.is_empty();
    let (raw_body_w, raw_body_h) = if is_degenerated
        && (entity_nodes.len() + relationship_nodes.len()) == 1
    {
        const DEGENERATED_DELTA: f64 = 7.0;
        let n = entity_nodes.first().or(relationship_nodes.first()).unwrap();
        (n.width + DEGENERATED_DELTA * 2.0, n.height + DEGENERATED_DELTA * 2.0)
    } else {
        (gl.lf_span.0 + CANVAS_DELTA, gl.lf_span.1 + CANVAS_DELTA)
    };

    let mut max_right = raw_body_w;
    let mut max_bottom = raw_body_h;

    for attr in &attribute_nodes {
        let ar = attr.x + attr.rx - render_dx + DOC_MARGIN_RIGHT;
        let ab = attr.y + attr.ry - render_dy + DOC_MARGIN_BOTTOM;
        max_right = max_right.max(ar);
        max_bottom = max_bottom.max(ab);
        for child in &attr.children {
            let cr = child.x + child.rx - render_dx + DOC_MARGIN_RIGHT;
            let cb = child.y + child.ry - render_dy + DOC_MARGIN_BOTTOM;
            max_right = max_right.max(cr);
            max_bottom = max_bottom.max(cb);
        }
    }

    for isa in &isa_layouts {
        let (tx, ty) = isa.triangle_center;
        max_right = max_right.max(tx + isa.triangle_size - render_dx + DOC_MARGIN_RIGHT);
        max_bottom = max_bottom.max(ty + isa.triangle_size - render_dy + DOC_MARGIN_BOTTOM);
    }

    let notes = layout_notes(&diagram.notes, &positions, max_right, max_bottom);

    for note in &notes {
        let nr = note.x + note.width - render_dx + DOC_MARGIN_RIGHT;
        let nb = note.y + note.height - render_dy + DOC_MARGIN_BOTTOM;
        max_right = max_right.max(nr);
        max_bottom = max_bottom.max(nb);
    }

    let width = max_right + DOC_MARGIN_RIGHT;
    let height = max_bottom + DOC_MARGIN_BOTTOM;

    debug!(
        "layout_erd done: {:.0}x{:.0} (lf_span={:.1}x{:.1}), {} ents, {} rels, {} attrs, {} edges, {} ISAs, {} notes",
        width, height, gl.lf_span.0, gl.lf_span.1,
        entity_nodes.len(), relationship_nodes.len(), attribute_nodes.len(),
        edges.len(), isa_layouts.len(), notes.len()
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

/// Compute the vertical (or horizontal) band needed for attribute ellipses.
fn compute_attr_band(diagram: &ErdDiagram) -> f64 {
    let has_attrs = diagram.entities.iter().any(|e| !e.attributes.is_empty())
        || diagram
            .relationships
            .iter()
            .any(|r| !r.attributes.is_empty());

    let has_nested = diagram
        .entities
        .iter()
        .any(|e| e.attributes.iter().any(|a| !a.children.is_empty()))
        || diagram
            .relationships
            .iter()
            .any(|r| r.attributes.iter().any(|a| !a.children.is_empty()));

    if has_nested {
        ATTR_DISTANCE * 2.0 + ATTR_RY * 2.0
    } else if has_attrs {
        ATTR_DISTANCE + ATTR_RY * 2.0
    } else {
        0.0
    }
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

        // Position attributes above (TB) or to the left (LR)
        let (ax, ay) = if is_lr {
            let total_span = (count - 1.0) * ATTR_SPACING;
            let start_y = parent_cy - total_span / 2.0;
            let y = start_y + i as f64 * ATTR_SPACING;
            (parent_cx, y - ATTR_DISTANCE)
        } else {
            let total_span = (count - 1.0) * ATTR_SPACING;
            let start_x = parent_cx - total_span / 2.0;
            let x = start_x + i as f64 * ATTR_SPACING;
            (x, parent_cy - ATTR_DISTANCE)
        };

        let display = attr.display_name.as_deref().unwrap_or(&attr.name);
        // For attrs with type annotation, combine "name : TYPE"
        let full_label = if let Some(ref t) = attr.attr_type {
            format!("{} : {}", display, t)
        } else {
            display.to_string()
        };
        let rx = attr_rx_for(&full_label);

        // Layout children of nested attribute
        let mut child_layouts = Vec::new();
        if !attr.children.is_empty() {
            let child_count = attr.children.len() as f64;
            let child_spacing = ATTR_SPACING * 1.5;
            let child_span = (child_count - 1.0) * child_spacing;

            for (ci, child) in attr.children.iter().enumerate() {
                let child_id = format!("{}__attr_{}", parent_id, *idx);
                *idx += 1;
                let cx = if is_lr {
                    ax
                } else {
                    ax - child_span / 2.0 + ci as f64 * child_spacing
                };
                let cy = ay - ATTR_DISTANCE * 1.4;

                let child_display = child.display_name.as_deref().unwrap_or(&child.name);
                let child_rx = attr_rx_for(child_display);

                child_layouts.push(ErdAttrLayout {
                    id: child_id,
                    label: child_display.to_string(),
                    parent: attr_id.clone(),
                    x: cx,
                    y: cy,
                    rx: child_rx,
                    ry: ATTR_RY,
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
            label: full_label,
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
        let (from_x, from_y, from_w, from_h) = match positions.get(&link.from) {
            Some(p) => *p,
            None => {
                log::warn!("link source '{}' not found in layout", link.from);
                continue;
            }
        };
        let (to_x, to_y, to_w, to_h) = match positions.get(&link.to) {
            Some(p) => *p,
            None => {
                log::warn!("link target '{}' not found in layout", link.to);
                continue;
            }
        };

        let from_cx = from_x + from_w / 2.0;
        let from_cy = from_y + from_h / 2.0;
        let to_cx = to_x + to_w / 2.0;
        let to_cy = to_y + to_h / 2.0;

        let from_point = clip_to_rect(from_cx, from_cy, from_w, from_h, to_cx, to_cy);
        let to_point = clip_to_rect(to_cx, to_cy, to_w, to_h, from_cx, from_cy);

        edges.push(ErdEdgeLayout {
            from_id: link.from.clone(),
            to_id: link.to.clone(),
            from_name: link.from.clone(),
            to_name: link.to.clone(),
            from_point,
            to_point,
            label: link.cardinality.clone(),
            is_double: link.is_double,
            source_line: 0,
            entity_idx_from: 0,
            entity_idx_to: 0,
            raw_path_d: None,
            label_xy: None,
        });
    }

    edges
}

/// Shift all numeric coordinates in an SVG path d-string by (dx, dy).
fn shift_svg_path(d: &str, dx: f64, dy: f64) -> String {
    use crate::render::svg::fmt_coord;
    let mut result = String::with_capacity(d.len() + 32);
    let mut chars = d.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == 'M' || c == 'C' || c == 'L' || c == ' ' {
            result.push(c);
            chars.next();
            continue;
        }
        if c == '-' || c == '.' || c.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '-' || nc == '.' || nc.is_ascii_digit() {
                    num_str.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            let x_val: f64 = num_str.parse().unwrap_or(0.0);
            result.push_str(&fmt_coord(x_val + dx));
            if let Some(&',') = chars.peek() {
                result.push(',');
                chars.next();
            }
            let mut num_str = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '-' || nc == '.' || nc.is_ascii_digit() {
                    num_str.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            let y_val: f64 = num_str.parse().unwrap_or(0.0);
            result.push_str(&fmt_coord(y_val + dy));
        } else {
            result.push(c);
            chars.next();
        }
    }
    result
}

/// Clip a line from (cx, cy) toward (target_x, target_y) to the rectangle.
fn clip_to_rect(cx: f64, cy: f64, w: f64, h: f64, target_x: f64, target_y: f64) -> (f64, f64) {
    let dx = target_x - cx;
    let dy = target_y - cy;

    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return (cx, cy);
    }

    let half_w = w / 2.0;
    let half_h = h / 2.0;
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
        let (px, py, pw, ph) = match positions.get(&isa.parent) {
            Some(p) => *p,
            None => {
                log::warn!("ISA parent '{}' not found", isa.parent);
                continue;
            }
        };

        let parent_cx = px + pw / 2.0;
        let parent_cy = py + ph / 2.0;

        let (tri_x, tri_y) = if is_lr {
            (parent_cx + pw / 2.0 + 50.0, parent_cy)
        } else {
            (parent_cx, parent_cy + ph / 2.0 + 50.0)
        };

        let kind_label = match isa.kind {
            crate::model::erd::IsaKind::Disjoint => "d".to_string(),
            crate::model::erd::IsaKind::Union => "U".to_string(),
        };

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
                (
                    tri_x + ISA_TRIANGLE_SIZE + 30.0,
                    tri_y - total_span / 2.0 + ci as f64 * ISA_CHILD_SPACING,
                )
            } else {
                (
                    tri_x - total_span / 2.0 + ci as f64 * ISA_CHILD_SPACING,
                    tri_y + ISA_TRIANGLE_SIZE + 30.0,
                )
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
        assert!((node.height - ENTITY_HEIGHT).abs() < 0.01);
    }

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

    #[test]
    fn test_multiple_entities_same_rank() {
        let d = ErdDiagram {
            entities: vec![simple_entity("A"), simple_entity("B"), simple_entity("C")],
            direction: ErdDirection::TopToBottom,
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        assert_eq!(layout.entity_nodes.len(), 3);
        // Unlinked → same rank → same y, increasing x
        let x0 = layout.entity_nodes[0].x;
        let x1 = layout.entity_nodes[1].x;
        let x2 = layout.entity_nodes[2].x;
        assert!(x0 < x1, "A.x < B.x: {} < {}", x0, x1);
        assert!(x1 < x2, "B.x < C.x: {} < {}", x1, x2);
    }

    #[test]
    fn test_left_to_right_direction() {
        let d = ErdDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            direction: ErdDirection::LeftToRight,
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        let y0 = layout.entity_nodes[0].y;
        let y1 = layout.entity_nodes[1].y;
        assert!(y0 < y1, "A.y < B.y: {} < {}", y0, y1);
    }

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
            assert!(node.x + node.width <= layout.width);
            assert!(node.y + node.height <= layout.height);
        }
    }

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

    #[test]
    fn test_clip_to_rect_below() {
        let (x, y) = clip_to_rect(100.0, 100.0, 80.0, 40.0, 100.0, 200.0);
        assert!((x - 100.0).abs() < 1.0);
        assert!((y - 120.0).abs() < 1.0);
    }

    #[test]
    fn test_clip_to_rect_right() {
        let (x, y) = clip_to_rect(100.0, 100.0, 80.0, 40.0, 300.0, 100.0);
        assert!((x - 140.0).abs() < 1.0);
        assert!((y - 100.0).abs() < 1.0);
    }

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

    #[test]
    fn test_topology_ranks() {
        let d = ErdDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            relationships: vec![simple_relationship("R")],
            links: vec![simple_link("A", "R", "1"), simple_link("R", "B", "N")],
            direction: ErdDirection::TopToBottom,
            ..empty_diagram()
        };
        let layout = layout_erd(&d).unwrap();
        let ay = layout.entity_nodes.iter().find(|n| n.id == "A").unwrap().y;
        let ry = layout
            .relationship_nodes
            .iter()
            .find(|n| n.id == "R")
            .unwrap()
            .y;
        let by = layout.entity_nodes.iter().find(|n| n.id == "B").unwrap().y;
        assert!(ay < ry, "A.y < R.y: {} < {}", ay, ry);
        assert!(ry < by, "R.y < B.y: {} < {}", ry, by);
    }
}
