use crate::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

/// Input: a graph node (abstract description independent of diagram type)
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub label: String,
    pub width_pt: f64,  // node width in pt (72pt = 1 inch)
    pub height_pt: f64, // node height in pt
}

/// Input: a graph edge
#[derive(Debug, Clone)]
pub struct LayoutEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// Layout direction
#[derive(Debug, Clone, Default)]
pub enum RankDir {
    #[default]
    TopToBottom,
    LeftToRight,
    BottomToTop,
    RightToLeft,
}

impl RankDir {
    fn as_str(&self) -> &'static str {
        match self {
            RankDir::TopToBottom => "TB",
            RankDir::LeftToRight => "LR",
            RankDir::BottomToTop => "BT",
            RankDir::RightToLeft => "RL",
        }
    }
}

/// Input: complete abstract graph description
#[derive(Debug, Clone)]
pub struct LayoutGraph {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    pub rankdir: RankDir,
}

/// Output: node position after layout (SVG coordinates, origin top-left, Y downward)
#[derive(Debug, Clone)]
pub struct NodeLayout {
    pub id: String,
    pub cx: f64,     // center x (converted from Graphviz pt, Y-axis flipped)
    pub cy: f64,     // center y
    pub width: f64,  // width
    pub height: f64, // height
}

/// Output: edge path after layout (SVG coordinates)
#[derive(Debug, Clone)]
pub struct EdgeLayout {
    pub from: String,
    pub to: String,
    /// Bezier control points (converted to SVG coordinates)
    pub points: Vec<(f64, f64)>,
    /// Arrow tip (SVG coordinates), parsed from "e,x,y ..."
    pub arrow_tip: Option<(f64, f64)>,
}

/// Note layout info (used for class diagrams, etc.)
#[derive(Debug, Clone)]
pub struct ClassNoteLayout {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub lines: Vec<String>,
    /// Connector line: from note edge to target entity edge (from_x, from_y, to_x, to_y)
    pub connector: Option<(f64, f64, f64, f64)>,
}

/// Output: layout result for the entire graph
#[derive(Debug, Clone)]
pub struct GraphLayout {
    pub nodes: Vec<NodeLayout>,
    pub edges: Vec<EdgeLayout>,
    pub notes: Vec<ClassNoteLayout>,
    pub total_width: f64,
    pub total_height: f64,
}

/// Java PlantUML default Graphviz parameters (from AbstractEntityDiagram.java)
const DEFAULT_NODESEP_IN: f64 = 0.35;
const DEFAULT_RANKSEP_IN: f64 = 0.8;

/// Minimum separation values in pixels (from DotStringFactory.java:238-253)
const MIN_RANK_SEP_PX: f64 = 60.0; // class/state/component
const MIN_NODE_SEP_PX: f64 = 35.0;

/// Convert pixels to Graphviz inches (72 DPI, from SvekUtils.java:99)
fn px_to_inches(px: f64) -> f64 {
    px / 72.0
}

/// Serialize a LayoutGraph into a DOT format string
fn to_dot(graph: &LayoutGraph) -> String {
    // Java: clamp to max(default, minSep/72) — DotStringFactory.java
    let nodesep = DEFAULT_NODESEP_IN.max(px_to_inches(MIN_NODE_SEP_PX));
    let ranksep = DEFAULT_RANKSEP_IN.max(px_to_inches(MIN_RANK_SEP_PX));
    let mut dot = format!(
        "digraph G {{\n  rankdir={};\n  nodesep={nodesep:.4};\n  ranksep={ranksep:.4};\n  node [fixedsize=true];\n",
        graph.rankdir.as_str()
    );
    for node in &graph.nodes {
        let w_in = node.width_pt / 72.0;
        let h_in = node.height_pt / 72.0;
        // wrap node id in quotes, escape double quotes in label
        let label = node.label.replace('"', "\\\"");
        dot.push_str(&format!(
            "  \"{}\" [label=\"{}\", width={:.4}, height={:.4}];\n",
            node.id, label, w_in, h_in
        ));
    }
    for edge in &graph.edges {
        match &edge.label {
            Some(lbl) => {
                let lbl = lbl.replace('"', "\\\"");
                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                    edge.from, edge.to, lbl
                ));
            }
            None => {
                dot.push_str(&format!("  \"{}\" -> \"{}\";\n", edge.from, edge.to));
            }
        }
    }
    dot.push_str("}\n");
    dot
}

/// Run Graphviz dot layout, returning node coordinates and edge paths.
///
/// Strategy: serialize the graph to DOT format, run layout via `dot -Tplain`
/// subprocess, and parse the plain format output to obtain node coordinates
/// and edge paths.
/// Plain format spec: <https://graphviz.org/docs/outputs/plain/>
pub fn layout(graph: &LayoutGraph) -> Result<GraphLayout, Error> {
    log::debug!(
        "layout: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    let dot_src = to_dot(graph);
    log::debug!("dot input:\n{dot_src}");

    // invoke dot -Tplain, pipe DOT via stdin, read plain format from stdout
    let mut child = Command::new("dot")
        .arg("-Tplain")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Layout(format!("failed to spawn dot: {e} (is graphviz installed?)")))?;

    child
        .stdin
        .take()
        .unwrap()
        .write_all(dot_src.as_bytes())
        .map_err(|e| Error::Layout(format!("failed to write to dot stdin: {e}")))?;

    let output = child
        .wait_with_output()
        .map_err(|e| Error::Layout(format!("dot process error: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Layout(format!("dot exited with error: {stderr}")));
    }

    let plain = String::from_utf8_lossy(&output.stdout);
    log::debug!("dot plain output:\n{plain}");

    parse_plain_output(&plain, graph)
}

/// Parse `dot -Tplain` output.
///
/// Plain format line types:
/// - `graph scale width height`
/// - `node name x y width height label style shape color fillcolor`
/// - `edge tail head n x1 y1 [x2 y2 ...] [label xl yl] style color`
/// - `stop`
///
/// Origin is at bottom-left, units are inches (already multiplied by scale).
/// Y-axis must be flipped for SVG coordinates.
fn parse_plain_output(plain: &str, graph: &LayoutGraph) -> Result<GraphLayout, Error> {
    let mut total_width = 0.0_f64;
    let mut total_height = 0.0_f64;
    let mut node_map: std::collections::HashMap<String, NodeLayout> =
        std::collections::HashMap::new();
    let mut edge_layouts: Vec<EdgeLayout> = Vec::new();

    for line in plain.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "graph" => {
                // graph scale width height
                if tokens.len() >= 4 {
                    let scale: f64 = tokens[1].parse().unwrap_or(1.0);
                    let w: f64 = tokens[2].parse().unwrap_or(0.0);
                    let h: f64 = tokens[3].parse().unwrap_or(0.0);
                    // plain units are inches, multiply by 72 to convert to pt
                    total_width = w * scale * 72.0;
                    total_height = h * scale * 72.0;
                }
            }
            "node" => {
                // node name x y width height label style shape color fillcolor
                if tokens.len() >= 6 {
                    let id = unquote(tokens[1]);
                    let gx: f64 = tokens[2].parse().unwrap_or(0.0);
                    let gy: f64 = tokens[3].parse().unwrap_or(0.0);
                    let w: f64 = tokens[4].parse().unwrap_or(0.0);
                    let h: f64 = tokens[5].parse().unwrap_or(0.0);
                    // Use our original precise size instead of Graphviz's
                    // rounded inches→pt conversion to avoid precision loss.
                    let orig_size = graph.nodes.iter().find(|n| n.id == id);
                    let (precise_w, precise_h) = match orig_size {
                        Some(n) => (n.width_pt, n.height_pt),
                        None => (w * 72.0, h * 72.0),
                    };
                    node_map.insert(
                        id.clone(),
                        NodeLayout {
                            id,
                            cx: gx * 72.0,
                            cy: total_height - gy * 72.0, // flip Y-axis
                            width: precise_w,
                            height: precise_h,
                        },
                    );
                }
            }
            "edge" => {
                // edge tail head n x1 y1 x2 y2 ... [label xl yl] style color
                if tokens.len() >= 5 {
                    let from = unquote(tokens[1]);
                    let to = unquote(tokens[2]);
                    let n: usize = tokens[3].parse().unwrap_or(0);
                    let mut points = Vec::with_capacity(n);
                    for i in 0..n {
                        let xi = 4 + i * 2;
                        let yi = xi + 1;
                        if yi < tokens.len() {
                            let x: f64 = tokens[xi].parse().unwrap_or(0.0);
                            let y: f64 = tokens[yi].parse().unwrap_or(0.0);
                            points.push((x * 72.0, total_height - y * 72.0));
                        }
                    }
                    edge_layouts.push(EdgeLayout {
                        from,
                        to,
                        points,
                        arrow_tip: None,
                    });
                }
            }
            "stop" => break,
            _ => {}
        }
    }

    // order output nodes according to LayoutGraph node order
    let nodes: Vec<NodeLayout> = graph
        .nodes
        .iter()
        .filter_map(|n| node_map.remove(&n.id))
        .collect();

    if nodes.len() != graph.nodes.len() {
        log::warn!(
            "layout: expected {} nodes, got {} from dot output",
            graph.nodes.len(),
            nodes.len()
        );
    }

    // Java PlantUML: moveDelta(6 - minX, 6 - minY) normalizes coordinates
    // so the top-left entity corner starts near (7, 7) after adding the
    // render-time MARGIN offset.  We normalize to (0, 0) here; the renderer
    // adds its own MARGIN.
    let min_x = nodes
        .iter()
        .map(|n| n.cx - n.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    let min_y = nodes
        .iter()
        .map(|n| n.cy - n.height / 2.0)
        .fold(f64::INFINITY, f64::min);
    let mut nodes = nodes;
    for n in &mut nodes {
        n.cx -= min_x;
        n.cy -= min_y;
    }
    for e in &mut edge_layouts {
        for p in &mut e.points {
            p.0 -= min_x;
            p.1 -= min_y;
        }
    }
    // Recompute bounding box after normalization
    let max_x = nodes
        .iter()
        .map(|n| n.cx + n.width / 2.0)
        .fold(0.0_f64, f64::max);
    let max_y = nodes
        .iter()
        .map(|n| n.cy + n.height / 2.0)
        .fold(0.0_f64, f64::max);
    let total_width = max_x;
    let total_height = max_y;

    Ok(GraphLayout {
        nodes,
        edges: edge_layouts,
        notes: vec![],
        total_width,
        total_height,
    })
}

/// Remove surrounding quotes from node names in dot plain output, if present
fn unquote(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_node_graph() -> LayoutGraph {
        LayoutGraph {
            nodes: vec![
                LayoutNode {
                    id: "A".into(),
                    label: "ClassA".into(),
                    width_pt: 108.0,
                    height_pt: 36.0,
                },
                LayoutNode {
                    id: "B".into(),
                    label: "ClassB".into(),
                    width_pt: 108.0,
                    height_pt: 36.0,
                },
            ],
            edges: vec![LayoutEdge {
                from: "A".into(),
                to: "B".into(),
                label: None,
            }],
            rankdir: RankDir::TopToBottom,
        }
    }

    #[test]
    fn test_two_node_layout() {
        let result = layout(&two_node_graph()).expect("layout failed");
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);
        assert!(result.total_width > 0.0);
        assert!(result.total_height > 0.0);
        // verify node coordinates are reasonable
        for n in &result.nodes {
            assert!(n.cx >= 0.0, "cx must be non-negative");
            assert!(n.cy >= 0.0, "cy must be non-negative");
            assert!(n.width > 0.0);
            assert!(n.height > 0.0);
        }
        // verify edge has control points
        let edge = &result.edges[0];
        assert!(!edge.points.is_empty(), "edge must have control points");
    }

    #[test]
    fn test_single_node_layout() {
        let graph = LayoutGraph {
            nodes: vec![LayoutNode {
                id: "X".into(),
                label: "Only".into(),
                width_pt: 72.0,
                height_pt: 36.0,
            }],
            edges: vec![],
            rankdir: RankDir::LeftToRight,
        };
        let result = layout(&graph).expect("single node layout failed");
        assert_eq!(result.nodes.len(), 1);
        assert!(result.nodes[0].cx >= 0.0);
    }
}
