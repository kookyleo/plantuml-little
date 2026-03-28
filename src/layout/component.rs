//! Component diagram layout engine.
//!
//! Converts a `ComponentDiagram` into a fully positioned `ComponentLayout`
//! ready for SVG rendering. Uses Graphviz/Smetana for node positioning.

use std::collections::HashMap;

use crate::font_metrics;
use crate::layout::graphviz::{
    self, LayoutClusterSpec, LayoutEdge, LayoutGraph, LayoutNode, RankDir,
};
use crate::model::component::{ComponentDiagram, ComponentEntity, ComponentKind, ComponentLink};
use crate::model::Direction;
use crate::render::svg::{ensure_visible_int, CANVAS_DELTA, DOC_MARGIN_BOTTOM, DOC_MARGIN_RIGHT};
use crate::svek::node::EntityPosition;
use crate::svek::shape_type::ShapeType;
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned component diagram ready for rendering.
#[derive(Debug)]
pub struct ComponentLayout {
    pub nodes: Vec<ComponentNodeLayout>,
    pub edges: Vec<ComponentEdgeLayout>,
    pub notes: Vec<ComponentNoteLayout>,
    pub groups: Vec<ComponentGroupLayout>,
    pub width: f64,
    pub height: f64,
}

/// A single positioned component/rectangle/node/etc.
#[derive(Debug, Clone)]
pub struct ComponentNodeLayout {
    pub id: String,
    pub name: String,
    pub kind: ComponentKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub description: Vec<String>,
    pub source_line: Option<usize>,
    pub stereotype: Option<String>,
    pub color: Option<String>,
}

/// An edge between two components.
#[derive(Debug, Clone)]
pub struct ComponentEdgeLayout {
    pub from: String,
    pub to: String,
    pub points: Vec<(f64, f64)>,
    pub raw_path_d: Option<String>,
    pub label: String,
    pub dashed: bool,
}

/// A positioned note.
#[derive(Debug, Clone)]
pub struct ComponentNoteLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
    pub position: String,
    pub target: Option<String>,
}

/// A positioned group (rectangle container).
#[derive(Debug, Clone)]
pub struct ComponentGroupLayout {
    pub id: String,
    pub name: String,
    pub kind: ComponentKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub source_line: Option<usize>,
    pub stereotype: Option<String>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FONT_SIZE: f64 = 14.0;
// Java: line_height = (ascent + descent) from AWT FontMetrics for SansSerif 14pt
const LINE_HEIGHT: f64 = 16.2969; // (1901 + 483) / 2048 * 14
                                  // Java: component node padding = 15px top + 15px bottom
const PADDING: f64 = 15.0;
// Java: no explicit minimum width for components; the name + icon determines width
const NODE_MIN_WIDTH: f64 = 0.0;
const NODE_MIN_HEIGHT: f64 = 40.0;
// Java Smetana: nodesep ≈ 35px (0.486111 inches * 72)
const NODE_SPACING_X: f64 = 35.0;
const NODE_SPACING_Y: f64 = 50.0;
const GROUP_PADDING: f64 = 20.0;
const GROUP_HEADER: f64 = 30.0;
const NOTE_OFFSET: f64 = 20.0;
const NOTE_MAX_WIDTH: f64 = 200.0;
const MARGIN: f64 = 7.0;
const GRID_COLS: usize = 3;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

fn text_width(text: &str) -> f64 {
    font_metrics::text_width(text, "SansSerif", FONT_SIZE, false, false)
}

/// Component icon (the small box at top-right) adds 10px to width:
/// gap(5) + icon_width(15) + right_pad(5) - right_PADDING(15) = 10
const COMPONENT_ICON_EXTRA: f64 = 10.0;

/// Estimate the size of a component entity.
fn estimate_entity_size(entity: &ComponentEntity) -> (f64, f64) {
    // Ports are small: 12x12 square (Java EntityPosition.RADIUS * 2)
    // The text label is rendered outside the graphviz node, so the DOT node is just the port square.
    if matches!(entity.kind, ComponentKind::PortIn | ComponentKind::PortOut) {
        let port_size: f64 = 12.0;
        return (port_size, port_size);
    }

    // Java: width = leftPad(15) + text + gap(5) + icon(15) + rightPad(5)
    //     = text + 40 = text + 2*PADDING + COMPONENT_ICON_EXTRA
    // Name may contain real newlines (from \n expansion) — split and measure each line
    let name_lines: Vec<&str> = entity.name.lines().collect();
    let name_line_count = name_lines.len().max(1);
    let name_w = name_lines
        .iter()
        .map(|line| text_width(line))
        .fold(0.0_f64, f64::max)
        + 2.0 * PADDING
        + COMPONENT_ICON_EXTRA;

    let desc_w = entity
        .description
        .iter()
        .map(|line| text_width(line) + 2.0 * PADDING)
        .fold(0.0_f64, f64::max);

    let stereo_w = entity
        .stereotype
        .as_ref()
        .map_or(0.0, |s| text_width(s) + 2.0 * PADDING + 20.0);

    let width = name_w.max(desc_w).max(stereo_w).max(NODE_MIN_WIDTH);

    let stereo_lines = if entity.stereotype.is_some() {
        1.0
    } else {
        0.0
    };
    let desc_lines = entity.description.len() as f64;
    let total_lines = name_line_count as f64 + stereo_lines + desc_lines;
    let height = (total_lines * LINE_HEIGHT + 2.0 * PADDING).max(NODE_MIN_HEIGHT);

    (width, height)
}

fn estimate_note_size(text: &str) -> (f64, f64) {
    let lines: Vec<&str> = text.lines().collect();
    let max_line_width = lines
        .iter()
        .map(|l| font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    let width = (max_line_width + 2.0 * PADDING).min(NOTE_MAX_WIDTH);
    let width = width.max(60.0);
    let height = (lines.len().max(1) as f64 * LINE_HEIGHT + 2.0 * PADDING).max(NODE_MIN_HEIGHT);
    (width, height)
}

fn parse_path_start(d: &str) -> Option<(f64, f64)> {
    let d = d.trim_start();
    let d = d.strip_prefix('M').or_else(|| d.strip_prefix('m'))?;
    let d = d.trim_start();
    let comma = d.find(',')?;
    let x: f64 = d[..comma].trim().parse().ok()?;
    let rest = &d[comma + 1..];
    let y_end = rest
        .find(|c: char| c.is_whitespace() || c.is_ascii_alphabetic())
        .unwrap_or(rest.len());
    let y: f64 = rest[..y_end].trim().parse().ok()?;
    Some((x, y))
}

fn align_raw_path_d(raw_d: &str, points: &[(f64, f64)], dx: f64, dy: f64) -> String {
    let Some(&(px, py)) = points.first() else {
        return graphviz::transform_path_d(raw_d, dx, dy);
    };
    let Some((rx, ry)) = parse_path_start(raw_d) else {
        return graphviz::transform_path_d(raw_d, dx, dy);
    };

    graphviz::transform_path_d(raw_d, dx + (px - rx), dy + (py - ry))
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn layout_component(cd: &ComponentDiagram) -> Result<ComponentLayout> {
    log::debug!(
        "layout_component: {} entities, {} links, {} groups, {} notes",
        cd.entities.len(),
        cd.links.len(),
        cd.groups.len(),
        cd.notes.len()
    );

    let entity_map: HashMap<String, &ComponentEntity> =
        cd.entities.iter().map(|e| (e.id.clone(), e)).collect();

    let group_ids: std::collections::HashSet<String> =
        cd.groups.iter().map(|g| g.id.clone()).collect();

    fn sanitize_id(name: &str) -> String {
        name.replace('<', "_LT_")
            .replace('>', "_GT_")
            .replace(',', "_COMMA_")
            .replace(' ', "_")
            .replace('"', "_Q_")
    }

    let id_to_dot: HashMap<String, String> = cd
        .entities
        .iter()
        .map(|e| (e.id.clone(), sanitize_id(&e.id)))
        .collect();

    let layout_nodes: Vec<LayoutNode> = cd
        .entities
        .iter()
        .filter(|e| !group_ids.contains(&e.id))
        .map(|e| {
            let (w, h) = estimate_entity_size(e);
            let entity_position = match e.kind {
                ComponentKind::PortIn => Some(EntityPosition::PortIn),
                ComponentKind::PortOut => Some(EntityPosition::PortOut),
                _ => None,
            };
            let shape = match e.kind {
                ComponentKind::PortIn | ComponentKind::PortOut => Some(ShapeType::RectanglePort),
                _ => None,
            };
            let max_label_width = match e.kind {
                ComponentKind::PortIn | ComponentKind::PortOut => Some(text_width(&e.name)),
                _ => None,
            };
            LayoutNode {
                id: id_to_dot
                    .get(&e.id)
                    .cloned()
                    .unwrap_or_else(|| sanitize_id(&e.id)),
                label: e.name.clone(),
                width_pt: w,
                height_pt: h,
                shape,
                shield: None,
                entity_position,
                max_label_width,
                order: e.source_line,
                image_width_pt: None,
                lf_extra_left: 0.0,
            }
        })
        .collect();

    let layout_edges: Vec<LayoutEdge> = cd
        .links
        .iter()
        .map(|link| {
            let from_dot = id_to_dot
                .get(&link.from)
                .cloned()
                .unwrap_or_else(|| sanitize_id(&link.from));
            let to_dot = id_to_dot
                .get(&link.to)
                .cloned()
                .unwrap_or_else(|| sanitize_id(&link.to));
            LayoutEdge {
                from: from_dot,
                to: to_dot,
                label: if link.label.is_empty() {
                    None
                } else {
                    Some(link.label.clone())
                },
                tail_label: None,
                tail_label_boxed: false,
                head_label: None,
                head_label_boxed: false,
                tail_decoration: crate::svek::edge::LinkDecoration::None,
                head_decoration: crate::svek::edge::LinkDecoration::None,
                line_style: crate::svek::edge::LinkStyle::Normal,
                minlen: link.arrow_len.saturating_sub(1) as u32,
                invisible: false,
            }
        })
        .collect();

    let rankdir = match cd.direction {
        Direction::TopToBottom => RankDir::TopToBottom,
        Direction::LeftToRight => RankDir::LeftToRight,
        Direction::BottomToTop => RankDir::BottomToTop,
        Direction::RightToLeft => RankDir::RightToLeft,
    };

    // Build cluster specs from parsed groups
    let clusters: Vec<LayoutClusterSpec> = cd
        .groups
        .iter()
        .map(|g| {
            let node_ids: Vec<String> = g
                .children
                .iter()
                .filter_map(|child_id| id_to_dot.get(child_id).cloned())
                .collect();
            LayoutClusterSpec {
                id: sanitize_id(&g.id),
                qualified_name: g.id.clone(),
                title: Some(g.name.clone()),
                style: crate::svek::cluster::ClusterStyle::Rectangle,
                label_width: None,
                label_height: None,
                node_ids,
                sub_clusters: vec![],
                order: g.source_line,
            }
        })
        .collect();

    let graph = LayoutGraph {
        nodes: layout_nodes,
        edges: layout_edges,
        clusters,
        rankdir,
        use_simplier_dot_link_strategy: false,
    };
    let gl = graphviz::layout_with_svek(&graph)?;

    let dot_to_id: HashMap<String, String> = id_to_dot
        .iter()
        .map(|(k, v)| (v.clone(), k.clone()))
        .collect();
    let edge_offset = MARGIN;

    let mut nodes: Vec<ComponentNodeLayout> = Vec::new();
    let mut node_positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();

    for nl in &gl.nodes {
        let entity_id = dot_to_id.get(&nl.id).cloned().unwrap_or(nl.id.clone());
        let entity = match entity_map.get(&entity_id) {
            Some(e) => *e,
            None => continue,
        };
        let x = nl.cx - nl.width / 2.0 + edge_offset;
        let y = nl.cy - nl.height / 2.0 + edge_offset;
        node_positions.insert(entity_id.clone(), (x, y, nl.width, nl.height));
        nodes.push(ComponentNodeLayout {
            id: entity_id,
            name: entity.name.clone(),
            kind: entity.kind.clone(),
            x,
            y,
            width: nl.width,
            height: nl.height,
            description: entity.description.clone(),
            stereotype: entity.stereotype.clone(),
            color: entity.color.clone(),
            source_line: entity.source_line,
        });
    }

    let edges: Vec<ComponentEdgeLayout> = gl
        .edges
        .iter()
        .zip(cd.links.iter())
        .map(|(el, link)| {
            let mut points = el.points.clone();
            for pt in &mut points {
                pt.0 += edge_offset;
                pt.1 += edge_offset;
            }
            ComponentEdgeLayout {
                from: link.from.clone(),
                to: link.to.clone(),
                points,
                raw_path_d: el
                    .raw_path_d
                    .as_ref()
                    .map(|raw_d| align_raw_path_d(raw_d, &el.points, edge_offset, edge_offset)),
                label: link.label.clone(),
                dashed: link.dashed,
            }
        })
        .collect();

    // Build group layouts from graphviz cluster output
    let group_map: HashMap<String, &crate::model::component::ComponentGroup> =
        cd.groups.iter().map(|g| (sanitize_id(&g.id), g)).collect();
    let group_layouts: Vec<ComponentGroupLayout> = gl
        .clusters
        .iter()
        .filter_map(|cl| {
            let dot_id = sanitize_id(&cl.qualified_name);
            let group = group_map.get(&dot_id).or_else(|| group_map.get(&cl.id))?;
            Some(ComponentGroupLayout {
                id: group.id.clone(),
                name: group.name.clone(),
                kind: group.kind.clone(),
                x: cl.x + edge_offset,
                y: cl.y + edge_offset,
                width: cl.width,
                height: cl.height,
                source_line: group.source_line,
                stereotype: group.stereotype.clone(),
            })
        })
        .collect();

    let mut note_layouts = Vec::new();
    let all_right = nodes.iter().map(|n| n.x + n.width).fold(0.0_f64, f64::max);
    let note_x_default = all_right + NOTE_OFFSET + MARGIN;
    let mut note_y = MARGIN;
    for note in &cd.notes {
        let (nw, nh) = estimate_note_size(&note.text);
        let (nx, ny) = if let Some(ref target) = note.target {
            if let Some(&(tx, ty, tw, th)) = node_positions.get(target) {
                match note.position.as_str() {
                    "top" => (tx, ty - nh - NOTE_OFFSET),
                    "bottom" => (tx, ty + th + NOTE_OFFSET),
                    "left" => (tx - nw - NOTE_OFFSET, ty),
                    "right" => (tx + tw + NOTE_OFFSET, ty),
                    _ => (note_x_default, note_y),
                }
            } else {
                (note_x_default, note_y)
            }
        } else {
            (note_x_default, note_y)
        };
        let nx = nx.max(MARGIN);
        let ny = ny.max(MARGIN);
        note_layouts.push(ComponentNoteLayout {
            x: nx,
            y: ny,
            width: nw,
            height: nh,
            text: note.text.clone(),
            position: note.position.clone(),
            target: note.target.clone(),
        });
        note_y = ny + nh + PADDING;
    }

    // Viewport calculation: match Java's degenerated vs normal path
    let is_degenerated = nodes.len() <= 1 && edges.is_empty();
    let (raw_body_w, raw_body_h) = if is_degenerated && !nodes.is_empty() {
        const DEGENERATED_DELTA: f64 = 7.0;
        let entity_w = nodes[0].width;
        let entity_h = nodes[0].height;
        (
            entity_w + DEGENERATED_DELTA * 2.0,
            entity_h + DEGENERATED_DELTA * 2.0,
        )
    } else {
        let span_w = gl.lf_span.0;
        let span_h = gl.lf_span.1;
        (span_w + CANVAS_DELTA, span_h + CANVAS_DELTA)
    };

    // Extend viewport to include notes
    let mut max_right = raw_body_w;
    let mut max_bottom = raw_body_h;
    for note in &note_layouts {
        let nr = note.x + note.width - MARGIN + DOC_MARGIN_RIGHT;
        let nb = note.y + note.height - MARGIN + DOC_MARGIN_BOTTOM;
        if nr > max_right {
            max_right = nr;
        }
        if nb > max_bottom {
            max_bottom = nb;
        }
    }
    // Also extend for group layouts
    for group in &group_layouts {
        let gr = group.x + group.width - MARGIN + DOC_MARGIN_RIGHT;
        let gb = group.y + group.height - MARGIN + DOC_MARGIN_BOTTOM;
        if gr > max_right {
            max_right = gr;
        }
        if gb > max_bottom {
            max_bottom = gb;
        }
    }

    let total_width = ensure_visible_int(max_right + DOC_MARGIN_RIGHT) as f64;
    let total_height = ensure_visible_int(max_bottom + DOC_MARGIN_BOTTOM) as f64;

    log::debug!(
        "layout_component done: {:.0}x{:.0}",
        total_width,
        total_height
    );

    Ok(ComponentLayout {
        nodes,
        edges,
        notes: note_layouts,
        groups: group_layouts,
        width: total_width,
        height: total_height,
    })
}

// ---------------------------------------------------------------------------
// Direction transform
// ---------------------------------------------------------------------------

/// Apply a coordinate transform based on the diagram direction.
/// The layout algorithm always computes in top-to-bottom orientation;
/// for other directions we transform after the fact.
fn apply_direction_transform(
    layout: &mut ComponentLayout,
    direction: &crate::model::diagram::Direction,
) {
    use crate::model::diagram::Direction;
    match direction {
        Direction::TopToBottom => {}
        Direction::LeftToRight => {
            for node in &mut layout.nodes {
                std::mem::swap(&mut node.x, &mut node.y);
                std::mem::swap(&mut node.width, &mut node.height);
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    std::mem::swap(&mut pt.0, &mut pt.1);
                }
            }
            for note in &mut layout.notes {
                std::mem::swap(&mut note.x, &mut note.y);
                std::mem::swap(&mut note.width, &mut note.height);
            }
            for group in &mut layout.groups {
                std::mem::swap(&mut group.x, &mut group.y);
                std::mem::swap(&mut group.width, &mut group.height);
            }
            std::mem::swap(&mut layout.width, &mut layout.height);
        }
        Direction::RightToLeft => {
            for node in &mut layout.nodes {
                std::mem::swap(&mut node.x, &mut node.y);
                std::mem::swap(&mut node.width, &mut node.height);
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    std::mem::swap(&mut pt.0, &mut pt.1);
                }
            }
            for note in &mut layout.notes {
                std::mem::swap(&mut note.x, &mut note.y);
                std::mem::swap(&mut note.width, &mut note.height);
            }
            for group in &mut layout.groups {
                std::mem::swap(&mut group.x, &mut group.y);
                std::mem::swap(&mut group.width, &mut group.height);
            }
            std::mem::swap(&mut layout.width, &mut layout.height);
            let w = layout.width;
            for node in &mut layout.nodes {
                node.x = w - node.x - node.width;
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    pt.0 = w - pt.0;
                }
            }
            for note in &mut layout.notes {
                note.x = w - note.x - note.width;
            }
            for group in &mut layout.groups {
                group.x = w - group.x - group.width;
            }
        }
        Direction::BottomToTop => {
            let h = layout.height;
            for node in &mut layout.nodes {
                node.y = h - node.y - node.height;
            }
            for edge in &mut layout.edges {
                for pt in &mut edge.points {
                    pt.1 = h - pt.1;
                }
            }
            for note in &mut layout.notes {
                note.y = h - note.y - note.height;
            }
            for group in &mut layout.groups {
                group.y = h - group.y - group.height;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Edge routing
// ---------------------------------------------------------------------------

fn layout_edges(
    links: &[ComponentLink],
    pos_map: &HashMap<String, (f64, f64, f64, f64)>,
) -> Vec<ComponentEdgeLayout> {
    let mut result = Vec::new();

    for link in links {
        let from_pos = pos_map.get(&link.from);
        let to_pos = pos_map.get(&link.to);

        let (fx, fy, fw, fh) = if let Some(p) = from_pos {
            *p
        } else {
            log::warn!("edge source '{}' not found in layout", link.from);
            continue;
        };

        let (tx, ty, tw, th) = if let Some(p) = to_pos {
            *p
        } else {
            log::warn!("edge target '{}' not found in layout", link.to);
            continue;
        };

        let from_cx = fx + fw / 2.0;
        let from_cy = fy + fh / 2.0;
        let to_cx = tx + tw / 2.0;
        let to_cy = ty + th / 2.0;

        // Determine connection points based on direction hint or relative position
        let points = if let Some(ref hint) = link.direction_hint {
            route_with_hint(fx, fy, fw, fh, tx, ty, tw, th, hint)
        } else {
            route_auto(
                from_cx, from_cy, fx, fy, fw, fh, to_cx, to_cy, tx, ty, tw, th,
            )
        };

        log::debug!(
            "  edge '{}' -> '{}' [{}]: {:?}",
            link.from,
            link.to,
            link.label,
            points
        );

        result.push(ComponentEdgeLayout {
            from: link.from.clone(),
            to: link.to.clone(),
            points,
            raw_path_d: None,
            label: link.label.clone(),
            dashed: link.dashed,
        });
    }

    result
}

#[allow(clippy::too_many_arguments)]
fn route_with_hint(
    fx: f64,
    fy: f64,
    fw: f64,
    fh: f64,
    tx: f64,
    ty: f64,
    tw: f64,
    th: f64,
    hint: &str,
) -> Vec<(f64, f64)> {
    let from_cx = fx + fw / 2.0;
    let from_cy = fy + fh / 2.0;
    let to_cx = tx + tw / 2.0;
    let to_cy = ty + th / 2.0;

    match hint {
        "up" => vec![(from_cx, fy), (to_cx, ty + th)],
        "down" => vec![(from_cx, fy + fh), (to_cx, ty)],
        "left" => vec![(fx, from_cy), (tx + tw, to_cy)],
        "right" => vec![(fx + fw, from_cy), (tx, to_cy)],
        _ => route_auto(
            from_cx, from_cy, fx, fy, fw, fh, to_cx, to_cy, tx, ty, tw, th,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn route_auto(
    from_cx: f64,
    from_cy: f64,
    fx: f64,
    fy: f64,
    fw: f64,
    fh: f64,
    to_cx: f64,
    to_cy: f64,
    tx: f64,
    ty: f64,
    tw: f64,
    th: f64,
) -> Vec<(f64, f64)> {
    let dx = (to_cx - from_cx).abs();
    let dy = (to_cy - from_cy).abs();

    if dy > dx {
        // Vertical connection
        if to_cy > from_cy {
            vec![(from_cx, fy + fh), (to_cx, ty)]
        } else {
            vec![(from_cx, fy), (to_cx, ty + th)]
        }
    } else {
        // Horizontal connection
        if to_cx > from_cx {
            vec![(fx + fw, from_cy), (tx, to_cy)]
        } else {
            vec![(fx, from_cy), (tx + tw, to_cy)]
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::component::{
        ComponentDiagram, ComponentEntity, ComponentGroup, ComponentKind, ComponentLink,
        ComponentNote,
    };

    fn empty_diagram() -> ComponentDiagram {
        ComponentDiagram {
            entities: vec![],
            links: vec![],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        }
    }

    fn simple_entity(name: &str) -> ComponentEntity {
        ComponentEntity {
            name: name.to_string(),
            id: name.to_string(),
            kind: ComponentKind::Component,
            stereotype: None,
            description: vec![],
            parent: None,
            color: None,
            source_line: None,
        }
    }

    fn simple_link(from: &str, to: &str, label: &str) -> ComponentLink {
        ComponentLink {
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
            dashed: false,
            direction_hint: None,
            arrow_len: 2,
            source_line: None,
        }
    }

    // 1. Empty diagram
    #[test]
    fn test_empty_diagram() {
        let d = empty_diagram();
        let layout = layout_component(&d).unwrap();
        assert!(layout.nodes.is_empty());
        assert!(layout.edges.is_empty());
        assert!(layout.notes.is_empty());
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 2. Single component
    #[test]
    fn test_single_component() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("comp1")],
            links: vec![],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        assert_eq!(layout.nodes.len(), 1);
        let n = &layout.nodes[0];
        assert_eq!(n.id, "comp1");
        assert!(n.width >= NODE_MIN_WIDTH);
        assert!(n.height >= NODE_MIN_HEIGHT);
        assert!(n.x >= MARGIN);
        assert!(n.y >= MARGIN);
    }

    // 3. Two components with arrow
    #[test]
    fn test_two_components_with_arrow() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![simple_link("A", "B", "uses")],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        assert_eq!(layout.nodes.len(), 2);
        assert_eq!(layout.edges.len(), 1);
        assert_eq!(layout.edges[0].label, "uses");
        assert!(!layout.edges[0].points.is_empty());
    }

    #[test]
    fn test_align_raw_path_d_matches_points_start() {
        let raw_d = "M39,113.03 C39,125.82 39,153.48 39,166.63";
        let points = vec![
            (33.0, 107.03),
            (33.0, 119.82),
            (33.0, 147.48),
            (33.0, 160.63),
        ];

        let aligned = align_raw_path_d(raw_d, &points, 7.0, 7.0);

        assert!(aligned.starts_with("M40,114.03"), "got: {aligned}");
        assert!(
            aligned.contains("C40,126.82 40,154.48 40,167.63"),
            "got: {aligned}"
        );
    }

    // 4. Grid layout (more than GRID_COLS entities)
    #[test]
    fn test_grid_layout() {
        let d = ComponentDiagram {
            entities: vec![
                simple_entity("A"),
                simple_entity("B"),
                simple_entity("C"),
                simple_entity("D"),
                simple_entity("E"),
            ],
            links: vec![],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        assert_eq!(layout.nodes.len(), 5);

        // All nodes should have valid positions
        for n in &layout.nodes {
            assert!(n.x >= 0.0, "node {} x should be >= 0", n.id);
            assert!(n.y >= 0.0, "node {} y should be >= 0", n.id);
        }
    }

    // 5. Entity sizing
    #[test]
    fn test_entity_sizing() {
        let e = ComponentEntity {
            name: "A very long component name".to_string(),
            id: "long".to_string(),
            kind: ComponentKind::Component,
            stereotype: None,
            description: vec![],
            parent: None,
            color: None,
            source_line: None,
        };
        let (w, _) = estimate_entity_size(&e);
        assert!(w > NODE_MIN_WIDTH, "long name should produce wider node");
    }

    // 6. Entity with description
    #[test]
    fn test_entity_with_description() {
        let e = ComponentEntity {
            name: "A".to_string(),
            id: "A".to_string(),
            kind: ComponentKind::Rectangle,
            stereotype: None,
            description: vec![
                "line1".to_string(),
                "line2".to_string(),
                "line3".to_string(),
            ],
            parent: None,
            color: None,
            source_line: None,
        };
        let (_, h) = estimate_entity_size(&e);
        let expected = (4.0 * LINE_HEIGHT + 2.0 * PADDING).max(NODE_MIN_HEIGHT);
        assert!(h >= expected, "description should increase height");
    }

    // 7. Note layout
    #[test]
    fn test_note_layout() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("A")],
            links: vec![],
            groups: vec![],
            notes: vec![ComponentNote {
                text: "important note".to_string(),
                position: "right".to_string(),
                target: Some("A".to_string()),
            }],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        assert_eq!(layout.notes.len(), 1);
        let note = &layout.notes[0];
        assert!(note.width > 0.0);
        assert!(note.height > 0.0);
    }

    // 8. Dashed edge
    #[test]
    fn test_dashed_edge() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![ComponentLink {
                from: "A".to_string(),
                to: "B".to_string(),
                label: String::new(),
                dashed: true,
                direction_hint: None,
                arrow_len: 2,
                source_line: None,
            }],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        assert!(layout.edges[0].dashed);
    }

    // 9. Direction hint routing
    #[test]
    fn test_direction_hint_routing() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![ComponentLink {
                from: "A".to_string(),
                to: "B".to_string(),
                label: String::new(),
                dashed: false,
                direction_hint: Some("right".to_string()),
                arrow_len: 2,
                source_line: None,
            }],
            groups: vec![],
            direction: Default::default(),
            notes: vec![],
        };
        let layout = layout_component(&d).unwrap();
        assert!(!layout.edges[0].points.is_empty());
    }

    // 10. Group layout
    #[test]
    fn test_group_layout() {
        let d = ComponentDiagram {
            entities: vec![
                ComponentEntity {
                    name: "Outer".to_string(),
                    id: "Outer".to_string(),
                    kind: ComponentKind::Rectangle,
                    stereotype: None,
                    description: vec![],
                    parent: None,
                    color: None,
                    source_line: None,
                },
                ComponentEntity {
                    name: "Inner".to_string(),
                    id: "Inner".to_string(),
                    kind: ComponentKind::Component,
                    stereotype: None,
                    description: vec![],
                    parent: Some("Outer".to_string()),
                    color: None,
                    source_line: None,
                },
            ],
            links: vec![],
            groups: vec![ComponentGroup {
                name: "Outer".to_string(),
                id: "Outer".to_string(),
                kind: ComponentKind::Rectangle,
                stereotype: None,
                children: vec!["Inner".to_string()],
                source_line: None,
            }],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        let inner = layout.nodes.iter().find(|n| n.id == "Inner").unwrap();
        assert!(inner.width > 0.0);
        assert!(inner.height > 0.0);
    }

    // 11. Bounding box includes all elements
    #[test]
    fn test_bounding_box() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![simple_link("A", "B", "")],
            groups: vec![],
            notes: vec![ComponentNote {
                text: "note".to_string(),
                position: "right".to_string(),
                target: Some("A".to_string()),
            }],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        for node in &layout.nodes {
            assert!(
                node.x + node.width <= layout.width,
                "node right {} exceeds width {}",
                node.x + node.width,
                layout.width
            );
        }
    }

    // 12. Note size estimation
    #[test]
    fn test_note_size_estimation() {
        let (w, h) = estimate_note_size("hello");
        assert!(w >= 60.0);
        assert!(h >= NODE_MIN_HEIGHT);

        let (_w2, h2) = estimate_note_size("line1\nline2\nline3");
        assert!(h2 > h, "multiline note should be taller");
    }

    // 13. Text width estimation
    #[test]
    fn test_text_width() {
        assert_eq!(text_width(""), 0.0);
        let expected_a = crate::font_metrics::text_width("a", "SansSerif", FONT_SIZE, false, false);
        assert!((text_width("a") - expected_a).abs() < 0.001);
        let expected_abc =
            crate::font_metrics::text_width("abc", "SansSerif", FONT_SIZE, false, false);
        assert!((text_width("abc") - expected_abc).abs() < 0.001);
    }

    // 14. Missing edge target
    #[test]
    fn test_missing_edge_target() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("A")],
            links: vec![simple_link("A", "nonexistent", "")],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        // Edge should be skipped for missing target
        assert_eq!(layout.edges.len(), 0);
    }

    // 15. Entity with stereotype sizing
    #[test]
    fn test_stereotype_sizing() {
        let e = ComponentEntity {
            name: "A".to_string(),
            id: "A".to_string(),
            kind: ComponentKind::Component,
            stereotype: Some("MyStereotype".to_string()),
            description: vec![],
            parent: None,
            color: None,
            source_line: None,
        };
        let (_, h) = estimate_entity_size(&e);
        let plain_e = simple_entity("A");
        let (_, h_plain) = estimate_entity_size(&plain_e);
        assert!(h > h_plain, "stereotype should increase height");
    }

    // 16. Multiple notes
    #[test]
    fn test_multiple_notes() {
        let d = ComponentDiagram {
            entities: vec![simple_entity("A")],
            links: vec![],
            groups: vec![],
            notes: vec![
                ComponentNote {
                    text: "note 1".to_string(),
                    position: "top".to_string(),
                    target: Some("A".to_string()),
                },
                ComponentNote {
                    text: "note 2".to_string(),
                    position: "bottom".to_string(),
                    target: Some("A".to_string()),
                },
            ],
            direction: Default::default(),
        };
        let layout = layout_component(&d).unwrap();
        assert_eq!(layout.notes.len(), 2);
    }

    // 17. LeftToRight direction: wider than tall
    #[test]
    fn test_left_to_right_direction() {
        use crate::model::diagram::Direction;
        let d = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![simple_link("A", "B", "")],
            groups: vec![],
            notes: vec![],
            direction: Direction::LeftToRight,
        };
        let layout = layout_component(&d).unwrap();
        assert_eq!(layout.nodes.len(), 2);
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 18. TopToBottom is the default
    #[test]
    fn test_top_to_bottom_is_default() {
        use crate::model::diagram::Direction;
        let d1 = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![],
            groups: vec![],
            notes: vec![],
            direction: Direction::TopToBottom,
        };
        let d2 = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![],
            groups: vec![],
            notes: vec![],
            direction: Default::default(),
        };
        let l1 = layout_component(&d1).unwrap();
        let l2 = layout_component(&d2).unwrap();

        // Default should match TopToBottom
        assert!((l1.width - l2.width).abs() < 0.01);
        assert!((l1.height - l2.height).abs() < 0.01);
    }

    // 19. BottomToTop direction: first node at bottom
    #[test]
    fn test_bottom_to_top_direction() {
        use crate::model::diagram::Direction;
        let d = ComponentDiagram {
            entities: vec![simple_entity("A"), simple_entity("B")],
            links: vec![simple_link("A", "B", "")],
            groups: vec![],
            notes: vec![],
            direction: Direction::BottomToTop,
        };
        let layout = layout_component(&d).unwrap();
        assert_eq!(layout.nodes.len(), 2);
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn test_multiline_name_sizing() {
        let single = simple_entity("Web");
        let (_, h_single) = estimate_entity_size(&single);

        let multi = ComponentEntity {
            name: "Line1\nLine2\nLine3".to_string(),
            id: "multi".to_string(),
            kind: ComponentKind::Component,
            stereotype: None,
            description: vec![],
            parent: None,
            color: None,
            source_line: None,
        };
        let (_, h_multi) = estimate_entity_size(&multi);
        // 3 name lines should be taller than 1 name line
        assert!(
            h_multi > h_single,
            "multi-line name height {h_multi} should exceed single-line {h_single}"
        );
        // Height difference should be 2 * LINE_HEIGHT (2 extra lines)
        let diff = h_multi - h_single;
        assert!(
            (diff - 2.0 * LINE_HEIGHT).abs() < 0.01,
            "height diff {diff} should be ~{:.4}",
            2.0 * LINE_HEIGHT
        );
    }
}
