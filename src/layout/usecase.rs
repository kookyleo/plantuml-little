use std::collections::HashMap;

use crate::font_metrics;
use crate::model::diagram::Direction;
use crate::model::usecase::{UseCaseDiagram, UseCaseLinkStyle};
use crate::Result;

/// Layout result for use case diagrams.
#[derive(Debug)]
pub struct UseCaseLayout {
    pub actors: Vec<ActorLayout>,
    pub usecases: Vec<UseCaseNodeLayout>,
    pub edges: Vec<UseCaseEdgeLayout>,
    pub boundaries: Vec<BoundaryLayout>,
    pub total_width: f64,
    pub total_height: f64,
}

#[derive(Debug)]
pub struct ActorLayout {
    pub id: String,
    pub name: String,
    pub cx: f64,
    pub cy: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug)]
pub struct UseCaseNodeLayout {
    pub id: String,
    pub name: String,
    pub cx: f64,
    pub cy: f64,
    pub rx: f64,
    pub ry: f64,
}

#[derive(Debug)]
pub struct UseCaseEdgeLayout {
    pub from_x: f64,
    pub from_y: f64,
    pub to_x: f64,
    pub to_y: f64,
    pub label: String,
    pub dashed: bool,
    pub has_arrow: bool,
    pub stereotype: Option<String>,
}

#[derive(Debug)]
pub struct BoundaryLayout {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Nesting depth: 0 for top-level boundaries, 1+ for nested boundaries.
    pub nesting_depth: u32,
}

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const ACTOR_WIDTH: f64 = 50.0;
const ACTOR_HEIGHT: f64 = 80.0;
const UC_RY: f64 = 25.0;
const UC_RX_MIN: f64 = 60.0;
/// Font size for use case labels.
const FONT_SIZE: f64 = 14.0;
/// Spacing between items in the same column/row.
const ITEM_SPACING: f64 = 40.0;
/// Padding around the diagram border.
const MARGIN: f64 = 30.0;
/// Padding inside a boundary rectangle around its contents.
const BOUNDARY_PADDING: f64 = 20.0;

// ---------------------------------------------------------------------------
// Boundary nesting detection
// ---------------------------------------------------------------------------

/// Detect boundary nesting: returns a map from boundary id to its parent
/// boundary id and nesting depth. The parser pushes inner boundaries before
/// outer ones in the vec, so we reconstruct the tree by scanning the
/// boundaries and checking whether a boundary has direct use-case children
/// that are NOT claimed by earlier boundaries.
fn detect_boundary_nesting(diagram: &UseCaseDiagram) -> HashMap<String, u32> {
    let mut depth_map: HashMap<String, u32> = HashMap::new();

    // Inner boundaries appear earlier in the vec (parser stack order).
    let boundaries = &diagram.boundaries;
    if boundaries.len() < 2 {
        for b in boundaries {
            depth_map.insert(b.id.clone(), 0);
        }
        return depth_map;
    }

    // For each pair (i, j) where i < j, check if B_i is nested inside B_j
    // by verifying that their children sets are disjoint (inner boundaries
    // don't share use cases with outer ones). Among candidates, pick the
    // nearest one (smallest j > i).
    let mut parent_of: HashMap<String, String> = HashMap::new();
    for i in 0..boundaries.len() {
        let mut found_parent = false;
        for j in (i + 1)..boundaries.len() {
            let bi = &boundaries[i];
            let bj = &boundaries[j];

            // Disjoint children: no overlap means they are in separate scopes
            let overlap = bi.children.iter().any(|c| bj.children.contains(c));
            if overlap {
                continue;
            }

            parent_of.insert(bi.id.clone(), bj.id.clone());
            found_parent = true;
            break; // nearest parent found
        }
        if !found_parent {
            depth_map.insert(boundaries[i].id.clone(), 0);
        }
    }

    // Compute depths from parent_of chain
    for b in boundaries {
        if depth_map.contains_key(&b.id) {
            continue;
        }
        let mut depth = 0u32;
        let mut current = b.id.clone();
        while let Some(parent) = parent_of.get(&current) {
            depth += 1;
            current = parent.clone();
            if depth > boundaries.len() as u32 {
                break; // cycle guard
            }
        }
        depth_map.insert(b.id.clone(), depth);
    }

    depth_map
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Compute layout for a use case diagram.
pub fn layout_usecase(diagram: &UseCaseDiagram) -> Result<UseCaseLayout> {
    match diagram.direction {
        Direction::LeftToRight | Direction::RightToLeft => layout_left_right(diagram),
        Direction::TopToBottom | Direction::BottomToTop => layout_top_bottom(diagram),
    }
}

// ---------------------------------------------------------------------------
// LeftToRight / RightToLeft layout
//
// Actors form a vertical column on the left; use cases (optionally grouped by
// boundary) form one or more columns on the right.
// ---------------------------------------------------------------------------

fn layout_left_right(diagram: &UseCaseDiagram) -> Result<UseCaseLayout> {
    let mut actors: Vec<ActorLayout> = Vec::new();
    let mut usecases: Vec<UseCaseNodeLayout> = Vec::new();
    let mut boundaries: Vec<BoundaryLayout> = Vec::new();

    // ── Actors column (left side) ────────────────────────────────────
    let actor_col_x = MARGIN + ACTOR_WIDTH / 2.0;
    let mut actor_y = MARGIN + ACTOR_HEIGHT / 2.0;

    for actor in &diagram.actors {
        actors.push(ActorLayout {
            id: actor.id.clone(),
            name: actor.name.clone(),
            cx: actor_col_x,
            cy: actor_y,
            width: ACTOR_WIDTH,
            height: ACTOR_HEIGHT,
        });
        actor_y += ACTOR_HEIGHT + ITEM_SPACING;
    }

    let actor_col_width = if diagram.actors.is_empty() {
        0.0
    } else {
        ACTOR_WIDTH + ITEM_SPACING
    };

    // ── Use cases (right side) ───────────────────────────────────────
    // Use cases that belong to a boundary are placed inside their boundary rect.
    // Standalone use cases (no parent) form their own group.

    let uc_start_x = MARGIN + actor_col_width + MARGIN;

    // Collect boundary children sets (in order of boundary declaration)
    let mut boundary_children: Vec<Vec<String>> = Vec::new();
    for b in &diagram.boundaries {
        boundary_children.push(b.children.clone());
    }
    let all_boundary_children: Vec<String> = boundary_children.iter().flatten().cloned().collect();

    // Standalone use cases: those without a boundary parent
    let standalone: Vec<&crate::model::usecase::UseCase> = diagram
        .usecases
        .iter()
        .filter(|uc| !all_boundary_children.contains(&uc.id))
        .collect();

    // Current Y cursor for placing groups vertically
    let mut cursor_y = MARGIN;
    let mut uc_col_width: f64 = 0.0;

    // Place standalone use cases first
    if !standalone.is_empty() {
        let mut y = cursor_y + UC_RY;
        for uc in &standalone {
            let rx = uc_rx_for_name(&uc.name);
            usecases.push(UseCaseNodeLayout {
                id: uc.id.clone(),
                name: uc.name.clone(),
                cx: uc_start_x + rx,
                cy: y,
                rx,
                ry: UC_RY,
            });
            uc_col_width = uc_col_width.max(rx * 2.0);
            y += UC_RY * 2.0 + ITEM_SPACING;
        }
        cursor_y = y;
    }

    // Detect boundary nesting
    let nesting_depths = detect_boundary_nesting(diagram);

    // Separate root boundaries (depth 0) from nested boundaries (depth > 0)
    let root_boundary_indices: Vec<usize> = diagram
        .boundaries
        .iter()
        .enumerate()
        .filter(|(_, b)| nesting_depths.get(&b.id).copied().unwrap_or(0) == 0)
        .map(|(i, _)| i)
        .collect();

    let nested_boundary_indices: Vec<usize> = diagram
        .boundaries
        .iter()
        .enumerate()
        .filter(|(_, b)| nesting_depths.get(&b.id).copied().unwrap_or(0) > 0)
        .map(|(i, _)| i)
        .collect();

    // Place root boundary groups (including nested children inside them)
    for &b_idx in &root_boundary_indices {
        let boundary = &diagram.boundaries[b_idx];
        let children_ids = &boundary_children[b_idx];

        // Collect direct use-case children of this root boundary
        let children_ucs: Vec<&crate::model::usecase::UseCase> = diagram
            .usecases
            .iter()
            .filter(|uc| children_ids.contains(&uc.id))
            .collect();

        // Find nested boundaries that belong inside this root boundary
        let nested_here: Vec<usize> = nested_boundary_indices
            .iter()
            .copied()
            .filter(|&ni| {
                // A nested boundary belongs here if it appears before this root
                // boundary in the parser output (inner closes first)
                ni < b_idx
            })
            .collect();

        let b_x = uc_start_x;
        let b_inner_y = cursor_y + BOUNDARY_PADDING + 16.0; // 16 for header text
        let mut child_y = b_inner_y + UC_RY;
        let mut b_inner_width: f64 = 0.0;

        // Place nested boundaries first (as inner rectangles)
        for &ni in &nested_here {
            let nested_b = &diagram.boundaries[ni];
            let nested_children_ids = &boundary_children[ni];
            let nested_ucs: Vec<&crate::model::usecase::UseCase> = diagram
                .usecases
                .iter()
                .filter(|uc| nested_children_ids.contains(&uc.id))
                .collect();
            if nested_ucs.is_empty() {
                continue;
            }

            let depth = nesting_depths.get(&nested_b.id).copied().unwrap_or(1);
            let inner_b_x = b_x + BOUNDARY_PADDING;
            let inner_b_start_y = child_y - UC_RY;
            let inner_b_inner_y = inner_b_start_y + BOUNDARY_PADDING + 16.0;
            let mut inner_child_y = inner_b_inner_y + UC_RY;
            let mut inner_width: f64 = 0.0;

            for uc in &nested_ucs {
                let rx = uc_rx_for_name(&uc.name);
                usecases.push(UseCaseNodeLayout {
                    id: uc.id.clone(),
                    name: uc.name.clone(),
                    cx: inner_b_x + BOUNDARY_PADDING + rx,
                    cy: inner_child_y,
                    rx,
                    ry: UC_RY,
                });
                inner_width = inner_width.max(rx * 2.0);
                inner_child_y += UC_RY * 2.0 + ITEM_SPACING;
            }

            let inner_b_width = inner_width + BOUNDARY_PADDING * 2.0;
            let inner_b_height = inner_child_y - inner_b_start_y - ITEM_SPACING + BOUNDARY_PADDING;

            boundaries.push(BoundaryLayout {
                name: nested_b.name.clone(),
                x: inner_b_x,
                y: inner_b_start_y,
                width: inner_b_width,
                height: inner_b_height,
                nesting_depth: depth,
            });

            b_inner_width = b_inner_width.max(inner_b_width + BOUNDARY_PADDING);
            child_y = inner_child_y;
        }

        // Place direct use-case children of the root boundary
        for uc in &children_ucs {
            let rx = uc_rx_for_name(&uc.name);
            usecases.push(UseCaseNodeLayout {
                id: uc.id.clone(),
                name: uc.name.clone(),
                cx: b_x + BOUNDARY_PADDING + rx,
                cy: child_y,
                rx,
                ry: UC_RY,
            });
            b_inner_width = b_inner_width.max(rx * 2.0);
            child_y += UC_RY * 2.0 + ITEM_SPACING;
        }

        if children_ucs.is_empty() && nested_here.is_empty() {
            continue;
        }

        let b_width = b_inner_width + BOUNDARY_PADDING * 2.0;
        let b_height = child_y - cursor_y - ITEM_SPACING + BOUNDARY_PADDING;

        boundaries.push(BoundaryLayout {
            name: boundary.name.clone(),
            x: b_x,
            y: cursor_y,
            width: b_width,
            height: b_height,
            nesting_depth: 0,
        });

        uc_col_width = uc_col_width.max(b_width);
        cursor_y += b_height + ITEM_SPACING;
    }

    // Place remaining boundaries that have no parent (standalone, no nesting detected)
    for &b_idx in &nested_boundary_indices {
        let boundary = &diagram.boundaries[b_idx];
        // Skip if already placed as a nested boundary above
        if boundaries.iter().any(|bl| bl.name == boundary.name) {
            continue;
        }
        let children_ids = &boundary_children[b_idx];
        let children_ucs: Vec<&crate::model::usecase::UseCase> = diagram
            .usecases
            .iter()
            .filter(|uc| children_ids.contains(&uc.id))
            .collect();
        if children_ucs.is_empty() {
            continue;
        }

        let b_x = uc_start_x;
        let b_inner_y = cursor_y + BOUNDARY_PADDING + 16.0;
        let mut child_y = b_inner_y + UC_RY;
        let mut b_inner_width: f64 = 0.0;

        for uc in &children_ucs {
            let rx = uc_rx_for_name(&uc.name);
            usecases.push(UseCaseNodeLayout {
                id: uc.id.clone(),
                name: uc.name.clone(),
                cx: b_x + BOUNDARY_PADDING + rx,
                cy: child_y,
                rx,
                ry: UC_RY,
            });
            b_inner_width = b_inner_width.max(rx * 2.0);
            child_y += UC_RY * 2.0 + ITEM_SPACING;
        }

        let depth = nesting_depths.get(&boundary.id).copied().unwrap_or(0);
        let b_width = b_inner_width + BOUNDARY_PADDING * 2.0;
        let b_height = child_y - cursor_y - ITEM_SPACING + BOUNDARY_PADDING;

        boundaries.push(BoundaryLayout {
            name: boundary.name.clone(),
            x: b_x,
            y: cursor_y,
            width: b_width,
            height: b_height,
            nesting_depth: depth,
        });

        uc_col_width = uc_col_width.max(b_width);
        cursor_y += b_height + ITEM_SPACING;
    }

    // ── Total dimensions ────────────────────────────────────────────
    let actors_height = if diagram.actors.is_empty() {
        0.0
    } else {
        actor_y - ITEM_SPACING
    };
    let total_height = actors_height.max(cursor_y) + MARGIN;
    let total_width = uc_start_x + uc_col_width + MARGIN;

    // ── Edges ────────────────────────────────────────────────────────
    let edges = build_edges(diagram, &actors, &usecases);

    Ok(UseCaseLayout {
        actors,
        usecases,
        edges,
        boundaries,
        total_width,
        total_height,
    })
}

// ---------------------------------------------------------------------------
// TopToBottom / BottomToTop layout
//
// Actors form a horizontal row at the top; use cases (optionally grouped by
// boundary) are placed in the rows below.
// ---------------------------------------------------------------------------

fn layout_top_bottom(diagram: &UseCaseDiagram) -> Result<UseCaseLayout> {
    let mut actors: Vec<ActorLayout> = Vec::new();
    let mut usecases: Vec<UseCaseNodeLayout> = Vec::new();
    let mut boundaries: Vec<BoundaryLayout> = Vec::new();

    // ── Actors row (top) ─────────────────────────────────────────────
    let actor_row_y = MARGIN + ACTOR_HEIGHT / 2.0;
    let mut actor_x = MARGIN + ACTOR_WIDTH / 2.0;

    for actor in &diagram.actors {
        actors.push(ActorLayout {
            id: actor.id.clone(),
            name: actor.name.clone(),
            cx: actor_x,
            cy: actor_row_y,
            width: ACTOR_WIDTH,
            height: ACTOR_HEIGHT,
        });
        actor_x += ACTOR_WIDTH + ITEM_SPACING;
    }

    let actor_row_height = if diagram.actors.is_empty() {
        0.0
    } else {
        ACTOR_HEIGHT + ITEM_SPACING
    };

    // ── Use cases (below actors) ─────────────────────────────────────
    let uc_start_y = MARGIN + actor_row_height + MARGIN;

    let boundary_children: Vec<Vec<String>> = diagram
        .boundaries
        .iter()
        .map(|b| b.children.clone())
        .collect();
    let all_boundary_children: Vec<String> = boundary_children.iter().flatten().cloned().collect();

    let standalone: Vec<&crate::model::usecase::UseCase> = diagram
        .usecases
        .iter()
        .filter(|uc| !all_boundary_children.contains(&uc.id))
        .collect();

    let mut cursor_x = MARGIN;
    let mut uc_section_height: f64 = 0.0;

    // Standalone use cases in a row (horizontal, side by side)
    if !standalone.is_empty() {
        let y = uc_start_y + UC_RY;
        for uc in &standalone {
            let rx = uc_rx_for_name(&uc.name);
            usecases.push(UseCaseNodeLayout {
                id: uc.id.clone(),
                name: uc.name.clone(),
                cx: cursor_x + rx,
                cy: y,
                rx,
                ry: UC_RY,
            });
            cursor_x += rx * 2.0 + ITEM_SPACING;
        }
        uc_section_height = uc_section_height.max(UC_RY * 2.0);
    }

    // Detect boundary nesting
    let nesting_depths = detect_boundary_nesting(diagram);

    let root_boundary_indices: Vec<usize> = diagram
        .boundaries
        .iter()
        .enumerate()
        .filter(|(_, b)| nesting_depths.get(&b.id).copied().unwrap_or(0) == 0)
        .map(|(i, _)| i)
        .collect();

    let nested_boundary_indices: Vec<usize> = diagram
        .boundaries
        .iter()
        .enumerate()
        .filter(|(_, b)| nesting_depths.get(&b.id).copied().unwrap_or(0) > 0)
        .map(|(i, _)| i)
        .collect();

    // Boundary groups in columns side-by-side (root boundaries with nested inside)
    for &b_idx in &root_boundary_indices {
        let boundary = &diagram.boundaries[b_idx];
        let children_ids = &boundary_children[b_idx];
        let children_ucs: Vec<&crate::model::usecase::UseCase> = diagram
            .usecases
            .iter()
            .filter(|uc| children_ids.contains(&uc.id))
            .collect();

        let nested_here: Vec<usize> = nested_boundary_indices
            .iter()
            .copied()
            .filter(|&ni| ni < b_idx)
            .collect();

        let b_x = cursor_x;
        let b_y = uc_start_y;
        let b_inner_y = b_y + BOUNDARY_PADDING + 16.0;

        let mut child_y = b_inner_y + UC_RY;
        let mut b_inner_width: f64 = 0.0;

        // Place nested boundaries inside this root boundary
        for &ni in &nested_here {
            let nested_b = &diagram.boundaries[ni];
            let nested_children_ids = &boundary_children[ni];
            let nested_ucs: Vec<&crate::model::usecase::UseCase> = diagram
                .usecases
                .iter()
                .filter(|uc| nested_children_ids.contains(&uc.id))
                .collect();
            if nested_ucs.is_empty() {
                continue;
            }

            let depth = nesting_depths.get(&nested_b.id).copied().unwrap_or(1);
            let inner_b_x = b_x + BOUNDARY_PADDING;
            let inner_b_start_y = child_y - UC_RY;
            let inner_b_inner_y = inner_b_start_y + BOUNDARY_PADDING + 16.0;
            let mut inner_child_y = inner_b_inner_y + UC_RY;
            let mut inner_width: f64 = 0.0;

            for uc in &nested_ucs {
                let rx = uc_rx_for_name(&uc.name);
                usecases.push(UseCaseNodeLayout {
                    id: uc.id.clone(),
                    name: uc.name.clone(),
                    cx: inner_b_x + BOUNDARY_PADDING + rx,
                    cy: inner_child_y,
                    rx,
                    ry: UC_RY,
                });
                inner_width = inner_width.max(rx * 2.0);
                inner_child_y += UC_RY * 2.0 + ITEM_SPACING;
            }

            let inner_b_width = inner_width + BOUNDARY_PADDING * 2.0;
            let inner_b_height = inner_child_y - inner_b_start_y - ITEM_SPACING + BOUNDARY_PADDING;

            boundaries.push(BoundaryLayout {
                name: nested_b.name.clone(),
                x: inner_b_x,
                y: inner_b_start_y,
                width: inner_b_width,
                height: inner_b_height,
                nesting_depth: depth,
            });

            b_inner_width = b_inner_width.max(inner_b_width + BOUNDARY_PADDING);
            child_y = inner_child_y;
        }

        for uc in &children_ucs {
            let rx = uc_rx_for_name(&uc.name);
            usecases.push(UseCaseNodeLayout {
                id: uc.id.clone(),
                name: uc.name.clone(),
                cx: b_x + BOUNDARY_PADDING + rx,
                cy: child_y,
                rx,
                ry: UC_RY,
            });
            b_inner_width = b_inner_width.max(rx * 2.0);
            child_y += UC_RY * 2.0 + ITEM_SPACING;
        }

        if children_ucs.is_empty() && nested_here.is_empty() {
            continue;
        }

        let b_width = b_inner_width + BOUNDARY_PADDING * 2.0;
        let b_height = child_y - b_y - ITEM_SPACING + BOUNDARY_PADDING;

        boundaries.push(BoundaryLayout {
            name: boundary.name.clone(),
            x: b_x,
            y: b_y,
            width: b_width,
            height: b_height,
            nesting_depth: 0,
        });

        uc_section_height = uc_section_height.max(b_height);
        cursor_x += b_width + ITEM_SPACING;
    }

    // Place remaining nested boundaries that weren't placed inside a root
    for &b_idx in &nested_boundary_indices {
        let boundary = &diagram.boundaries[b_idx];
        if boundaries.iter().any(|bl| bl.name == boundary.name) {
            continue;
        }
        let children_ids = &boundary_children[b_idx];
        let children_ucs: Vec<&crate::model::usecase::UseCase> = diagram
            .usecases
            .iter()
            .filter(|uc| children_ids.contains(&uc.id))
            .collect();
        if children_ucs.is_empty() {
            continue;
        }

        let b_x = cursor_x;
        let b_y = uc_start_y;
        let b_inner_y = b_y + BOUNDARY_PADDING + 16.0;
        let mut child_y = b_inner_y + UC_RY;
        let mut b_inner_width: f64 = 0.0;

        for uc in &children_ucs {
            let rx = uc_rx_for_name(&uc.name);
            usecases.push(UseCaseNodeLayout {
                id: uc.id.clone(),
                name: uc.name.clone(),
                cx: b_x + BOUNDARY_PADDING + rx,
                cy: child_y,
                rx,
                ry: UC_RY,
            });
            b_inner_width = b_inner_width.max(rx * 2.0);
            child_y += UC_RY * 2.0 + ITEM_SPACING;
        }

        let depth = nesting_depths.get(&boundary.id).copied().unwrap_or(0);
        let b_width = b_inner_width + BOUNDARY_PADDING * 2.0;
        let b_height = child_y - b_y - ITEM_SPACING + BOUNDARY_PADDING;

        boundaries.push(BoundaryLayout {
            name: boundary.name.clone(),
            x: b_x,
            y: b_y,
            width: b_width,
            height: b_height,
            nesting_depth: depth,
        });

        uc_section_height = uc_section_height.max(b_height);
        cursor_x += b_width + ITEM_SPACING;
    }

    // ── Total dimensions ────────────────────────────────────────────
    let actors_width = if diagram.actors.is_empty() {
        0.0
    } else {
        actor_x - ITEM_SPACING
    };
    let total_width = actors_width.max(cursor_x) + MARGIN;
    let total_height = uc_start_y + uc_section_height + MARGIN;

    // ── Edges ────────────────────────────────────────────────────────
    let edges = build_edges(diagram, &actors, &usecases);

    Ok(UseCaseLayout {
        actors,
        usecases,
        edges,
        boundaries,
        total_width,
        total_height,
    })
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Compute the horizontal semi-axis for a use case oval based on text width.
fn uc_rx_for_name(name: &str) -> f64 {
    let text_half =
        font_metrics::text_width(name, "SansSerif", FONT_SIZE, false, false) / 2.0 + 12.0;
    text_half.max(UC_RX_MIN)
}

/// Build edge layouts by resolving actor/usecase center positions.
fn build_edges(
    diagram: &UseCaseDiagram,
    actors: &[ActorLayout],
    usecases: &[UseCaseNodeLayout],
) -> Vec<UseCaseEdgeLayout> {
    let mut edges = Vec::new();

    for link in &diagram.links {
        // Resolve source position (actor or use case)
        let from_pos = find_actor_center(actors, &link.from)
            .or_else(|| find_usecase_center(usecases, &link.from));

        // Resolve target position (use case or actor)
        let to_pos =
            find_usecase_center(usecases, &link.to).or_else(|| find_actor_center(actors, &link.to));

        let (from_x, from_y) = match from_pos {
            Some(p) => p,
            None => continue,
        };
        let (to_x, to_y) = match to_pos {
            Some(p) => p,
            None => continue,
        };

        let (dashed, stereotype) = match &link.style {
            UseCaseLinkStyle::Association => (false, None),
            UseCaseLinkStyle::Dashed => (true, Some(link.label.clone())),
            UseCaseLinkStyle::Dotted => (true, None),
            UseCaseLinkStyle::Inheritance => (false, None),
        };

        let label = match &link.style {
            UseCaseLinkStyle::Dashed => String::new(),
            _ => link.label.clone(),
        };

        edges.push(UseCaseEdgeLayout {
            from_x,
            from_y,
            to_x,
            to_y,
            label,
            dashed,
            has_arrow: true,
            stereotype,
        });
    }

    edges
}

fn find_actor_center(actors: &[ActorLayout], id: &str) -> Option<(f64, f64)> {
    actors.iter().find(|a| a.id == id).map(|a| (a.cx, a.cy))
}

fn find_usecase_center(usecases: &[UseCaseNodeLayout], id: &str) -> Option<(f64, f64)> {
    usecases
        .iter()
        .find(|uc| uc.id == id)
        .map(|uc| (uc.cx, uc.cy))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::diagram::Direction;
    use crate::model::usecase::{
        UseCase, UseCaseActor, UseCaseBoundary, UseCaseDiagram, UseCaseLink, UseCaseLinkStyle,
    };

    fn make_diagram(direction: Direction) -> UseCaseDiagram {
        UseCaseDiagram {
            actors: vec![],
            usecases: vec![],
            links: vec![],
            boundaries: vec![],
            notes: vec![],
            direction,
        }
    }

    // 1. Empty diagram produces a valid, non-zero layout
    #[test]
    fn test_empty_diagram_ltr() {
        let diagram = make_diagram(Direction::LeftToRight);
        let layout = layout_usecase(&diagram).expect("layout failed");
        assert!(layout.total_width > 0.0, "total_width must be > 0");
        assert!(layout.total_height > 0.0, "total_height must be > 0");
        assert!(layout.actors.is_empty());
        assert!(layout.usecases.is_empty());
        assert!(layout.edges.is_empty());
    }

    // 2. Actors are placed in the expected column (LeftToRight)
    #[test]
    fn test_actors_ltr_placement() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        diagram.actors.push(UseCaseActor {
            id: "a1".into(),
            name: "User".into(),
            stereotype: None,
            color: None,
        });
        diagram.actors.push(UseCaseActor {
            id: "a2".into(),
            name: "Admin".into(),
            stereotype: None,
            color: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        assert_eq!(layout.actors.len(), 2);

        // Both actors should have the same X (single column)
        let cx0 = layout.actors[0].cx;
        let cx1 = layout.actors[1].cx;
        assert_eq!(cx0, cx1, "actors in LTR must share same column X");

        // Second actor must be below the first
        assert!(
            layout.actors[1].cy > layout.actors[0].cy,
            "second actor must be further down"
        );

        // Actor dimensions match the constants
        assert_eq!(layout.actors[0].width, ACTOR_WIDTH);
        assert_eq!(layout.actors[0].height, ACTOR_HEIGHT);
    }

    // 3. Use cases are placed to the right of actors (LeftToRight)
    #[test]
    fn test_usecases_right_of_actors_ltr() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        diagram.actors.push(UseCaseActor {
            id: "a1".into(),
            name: "User".into(),
            stereotype: None,
            color: None,
        });
        diagram.usecases.push(UseCase {
            id: "uc1".into(),
            name: "Login".into(),
            stereotype: None,
            color: None,
            parent: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        let actor = &layout.actors[0];
        let uc = &layout.usecases[0];

        // Use case center must be to the right of the actor's right edge
        assert!(
            uc.cx > actor.cx + actor.width / 2.0,
            "use case must be right of actor"
        );
    }

    // 4. Actors placed at top for TopToBottom direction
    #[test]
    fn test_actors_ttb_placement() {
        let mut diagram = make_diagram(Direction::TopToBottom);
        diagram.actors.push(UseCaseActor {
            id: "a1".into(),
            name: "User".into(),
            stereotype: None,
            color: None,
        });
        diagram.actors.push(UseCaseActor {
            id: "a2".into(),
            name: "Admin".into(),
            stereotype: None,
            color: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        assert_eq!(layout.actors.len(), 2);

        // Both actors should share the same Y (single row)
        let cy0 = layout.actors[0].cy;
        let cy1 = layout.actors[1].cy;
        assert_eq!(cy0, cy1, "actors in TTB must share same row Y");

        // Second actor must be to the right of the first
        assert!(
            layout.actors[1].cx > layout.actors[0].cx,
            "second actor must be further right"
        );
    }

    // 5. Use cases below actors in TopToBottom direction
    #[test]
    fn test_usecases_below_actors_ttb() {
        let mut diagram = make_diagram(Direction::TopToBottom);
        diagram.actors.push(UseCaseActor {
            id: "a1".into(),
            name: "User".into(),
            stereotype: None,
            color: None,
        });
        diagram.usecases.push(UseCase {
            id: "uc1".into(),
            name: "Search".into(),
            stereotype: None,
            color: None,
            parent: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        let actor = &layout.actors[0];
        let uc = &layout.usecases[0];
        assert!(
            uc.cy > actor.cy + actor.height / 2.0,
            "use case must be below actor"
        );
    }

    // 6. Use case rx is at least UC_RX_MIN
    #[test]
    fn test_uc_rx_minimum() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        diagram.usecases.push(UseCase {
            id: "uc1".into(),
            name: "A".into(), // very short name
            stereotype: None,
            color: None,
            parent: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        assert!(
            layout.usecases[0].rx >= UC_RX_MIN,
            "rx must be at least {UC_RX_MIN}, got {}",
            layout.usecases[0].rx
        );
        assert_eq!(layout.usecases[0].ry, UC_RY);
    }

    // 7. Longer use case name produces wider rx
    #[test]
    fn test_uc_rx_scales_with_name() {
        let rx_short = uc_rx_for_name("Hi");
        let rx_long = uc_rx_for_name("A very long use case name here");
        assert!(
            rx_long > rx_short,
            "longer name should produce larger rx ({} > {})",
            rx_long,
            rx_short
        );
    }

    // 8. Boundary is placed and has positive dimensions
    #[test]
    fn test_boundary_layout() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        diagram.boundaries.push(UseCaseBoundary {
            id: "sys".into(),
            name: "System".into(),
            children: vec!["uc1".into(), "uc2".into()],
        });
        diagram.usecases.push(UseCase {
            id: "uc1".into(),
            name: "Login".into(),
            stereotype: None,
            color: None,
            parent: Some("sys".into()),
        });
        diagram.usecases.push(UseCase {
            id: "uc2".into(),
            name: "Logout".into(),
            stereotype: None,
            color: None,
            parent: Some("sys".into()),
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        assert_eq!(layout.boundaries.len(), 1);
        let b = &layout.boundaries[0];
        assert_eq!(b.name, "System");
        assert!(b.width > 0.0, "boundary width must be > 0");
        assert!(b.height > 0.0, "boundary height must be > 0");

        // Use cases inside must be within the boundary rect
        for uc in &layout.usecases {
            assert!(uc.cx - uc.rx >= b.x, "uc left edge must be inside boundary");
            assert!(
                uc.cx + uc.rx <= b.x + b.width,
                "uc right edge must be inside boundary"
            );
        }
    }

    // 9. Association edge: not dashed, no stereotype
    #[test]
    fn test_edge_association() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        diagram.actors.push(UseCaseActor {
            id: "a1".into(),
            name: "User".into(),
            stereotype: None,
            color: None,
        });
        diagram.usecases.push(UseCase {
            id: "uc1".into(),
            name: "Login".into(),
            stereotype: None,
            color: None,
            parent: None,
        });
        diagram.links.push(UseCaseLink {
            from: "a1".into(),
            to: "uc1".into(),
            label: String::new(),
            style: UseCaseLinkStyle::Association,
            direction_hint: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        assert_eq!(layout.edges.len(), 1);
        let edge = &layout.edges[0];
        assert!(!edge.dashed, "association must not be dashed");
        assert!(edge.stereotype.is_none(), "association has no stereotype");
        assert!(edge.has_arrow);
    }

    // 10. Dashed link carries stereotype label
    #[test]
    fn test_edge_dashed_stereotype() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        diagram.actors.push(UseCaseActor {
            id: "a1".into(),
            name: "User".into(),
            stereotype: None,
            color: None,
        });
        diagram.usecases.push(UseCase {
            id: "uc1".into(),
            name: "Login".into(),
            stereotype: None,
            color: None,
            parent: None,
        });
        diagram.usecases.push(UseCase {
            id: "uc2".into(),
            name: "Verify".into(),
            stereotype: None,
            color: None,
            parent: None,
        });
        diagram.links.push(UseCaseLink {
            from: "uc1".into(),
            to: "uc2".into(),
            label: "include".into(),
            style: UseCaseLinkStyle::Dashed,
            direction_hint: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        assert_eq!(layout.edges.len(), 1);
        let edge = &layout.edges[0];
        assert!(edge.dashed, "dashed link must be dashed");
        assert_eq!(
            edge.stereotype.as_deref(),
            Some("include"),
            "stereotype must carry the label"
        );
    }

    // 11. Link with unknown endpoints produces no edge (graceful)
    #[test]
    fn test_edge_unknown_endpoints_skipped() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        diagram.links.push(UseCaseLink {
            from: "nonexistent_actor".into(),
            to: "nonexistent_uc".into(),
            label: String::new(),
            style: UseCaseLinkStyle::Association,
            direction_hint: None,
        });

        let layout = layout_usecase(&diagram).expect("layout failed");
        assert!(
            layout.edges.is_empty(),
            "unknown endpoints must produce no edge"
        );
    }

    // 12. Total dimensions encompass all actors and use cases
    #[test]
    fn test_total_dimensions_encompass_content() {
        let mut diagram = make_diagram(Direction::LeftToRight);
        for i in 0..3 {
            diagram.actors.push(UseCaseActor {
                id: format!("a{i}"),
                name: format!("Actor {i}"),
                stereotype: None,
                color: None,
            });
        }
        for i in 0..4 {
            diagram.usecases.push(UseCase {
                id: format!("uc{i}"),
                name: format!("A fairly long use case name {i}"),
                stereotype: None,
                color: None,
                parent: None,
            });
        }

        let layout = layout_usecase(&diagram).expect("layout failed");

        // Every actor must fit within total dimensions
        for actor in &layout.actors {
            assert!(
                actor.cx + actor.width / 2.0 <= layout.total_width,
                "actor right edge must fit"
            );
            assert!(
                actor.cy + actor.height / 2.0 <= layout.total_height,
                "actor bottom edge must fit"
            );
        }

        // Every use case must fit within total dimensions
        for uc in &layout.usecases {
            assert!(
                uc.cx + uc.rx <= layout.total_width,
                "uc right edge must fit"
            );
            assert!(
                uc.cy + uc.ry <= layout.total_height,
                "uc bottom edge must fit"
            );
        }
    }
}
