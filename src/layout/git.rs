use crate::font_metrics;
use crate::model::git::GitDiagram;
use crate::Result;

/// Layout for a single commit node in the git graph.
#[derive(Debug, Clone)]
pub struct GitNodeLayout {
    /// Center X of the commit circle.
    pub cx: f64,
    /// Center Y of the commit circle.
    pub cy: f64,
    /// Radius of the commit circle.
    pub radius: f64,
    /// X position for the label text.
    pub label_x: f64,
    /// Y position for the label text baseline.
    pub label_y: f64,
    /// Label text.
    pub label: String,
    /// Depth level (1-based).
    pub depth: usize,
}

/// Layout for a connection line between nodes.
#[derive(Debug, Clone)]
pub struct GitEdgeLayout {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Full layout for the git diagram.
#[derive(Debug, Clone)]
pub struct GitLayout {
    pub nodes: Vec<GitNodeLayout>,
    pub edges: Vec<GitEdgeLayout>,
    pub width: f64,
    pub height: f64,
}

/// Commit circle radius.
const NODE_RADIUS: f64 = 8.0;
/// Vertical spacing between nodes.
const ROW_HEIGHT: f64 = 40.0;
/// Horizontal indent per depth level.
const DEPTH_INDENT: f64 = 30.0;
/// Margin around the diagram.
const MARGIN: f64 = 15.0;
/// Gap between the circle and the label text.
const LABEL_GAP: f64 = 10.0;
/// Font size for labels.
const FONT_SIZE: f64 = 13.0;

pub fn layout_git(d: &GitDiagram) -> Result<GitLayout> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut max_x: f64 = 0.0;

    for (i, node) in d.nodes.iter().enumerate() {
        let cx = MARGIN + (node.depth as f64) * DEPTH_INDENT;
        let cy = MARGIN + NODE_RADIUS + i as f64 * ROW_HEIGHT;

        let label_x = cx + NODE_RADIUS + LABEL_GAP;
        let label_y = cy + FONT_SIZE / 3.0; // approximate baseline centering

        let tw = font_metrics::text_width(&node.label, "SansSerif", FONT_SIZE, false, false);
        let right = label_x + tw;
        if right > max_x {
            max_x = right;
        }

        nodes.push(GitNodeLayout {
            cx,
            cy,
            radius: NODE_RADIUS,
            label_x,
            label_y,
            label: node.label.clone(),
            depth: node.depth,
        });
    }

    // Build edges: connect each node to the preceding node at the same or lesser depth
    // (finding parent). This models a simple git branch topology.
    for i in 1..d.nodes.len() {
        let child_depth = d.nodes[i].depth;
        // Walk backward to find the parent
        for j in (0..i).rev() {
            if d.nodes[j].depth < child_depth {
                // This is the branching parent
                edges.push(GitEdgeLayout {
                    x1: nodes[j].cx,
                    y1: nodes[j].cy,
                    x2: nodes[i].cx,
                    y2: nodes[i].cy,
                });
                break;
            } else if d.nodes[j].depth == child_depth {
                // Sibling at same depth — connect to the same parent
                // (the parent was already found for node j)
                // Actually, connect to immediate predecessor at same depth
                edges.push(GitEdgeLayout {
                    x1: nodes[j].cx,
                    y1: nodes[j].cy,
                    x2: nodes[i].cx,
                    y2: nodes[i].cy,
                });
                break;
            }
        }
    }

    let width = max_x + MARGIN;
    let height = MARGIN * 2.0 + NODE_RADIUS * 2.0 + (d.nodes.len().saturating_sub(1)) as f64 * ROW_HEIGHT;

    Ok(GitLayout {
        nodes,
        edges,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::git::{GitDiagram, GitNode};

    #[test]
    fn test_layout_basic() {
        let d = GitDiagram {
            nodes: vec![
                GitNode { depth: 1, label: "main".into(), index: 0 },
                GitNode { depth: 2, label: "feature1".into(), index: 1 },
                GitNode { depth: 2, label: "feature2".into(), index: 2 },
            ],
        };
        let l = layout_git(&d).unwrap();
        assert_eq!(l.nodes.len(), 3);
        assert_eq!(l.edges.len(), 2);
        assert!(l.width > 0.0);
        assert!(l.height > 0.0);
    }
}
