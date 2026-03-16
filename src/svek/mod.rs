// svek - Graphviz layout engine wrapper
// Port of Java PlantUML's net.sourceforge.plantuml.svek package
//
// Named "svek" = SVG + Graphviz Engine (K?)
// Workflow: Model → DOT string → Graphviz → SVG → parse coordinates → redraw via klimt

pub mod node;
pub mod edge;
pub mod cluster;
pub mod extremity;
pub mod shape_type;
pub mod image;
pub mod builder;
pub mod snake;
pub mod svg_result;

use crate::klimt::geom::XPoint2D;

// ── DotMode ──────────────────────────────────────────────────────────

/// Controls DOT generation mode.
/// Java: `svek.DotMode`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DotMode {
    #[default]
    Normal,
    NoLeftRightAndXlabel,
}

// ── DotSplines ───────────────────────────────────────────────────────

/// Graphviz splines mode.
/// Java: `dot.DotSplines`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DotSplines {
    #[default]
    Spline,
    Polyline,
    Ortho,
    Curved,
}

// ── ColorSequence ────────────────────────────────────────────────────

/// Generates unique colors for matching DOT elements in SVG output.
/// Java: `svek.ColorSequence`
///
/// Each node/edge is assigned a unique stroke color in the DOT source.
/// After Graphviz renders SVG, we find each element by its color to
/// extract its coordinates.
#[derive(Debug)]
pub struct ColorSequence {
    current: u32,
}

impl ColorSequence {
    pub fn new() -> Self {
        Self { current: 0x0001_0100 }
    }

    /// Get next unique color as RGB integer.
    pub fn next_color(&mut self) -> u32 {
        let result = self.current;
        self.current += 0x0001_0100;
        // Skip problematic colors
        if (self.current & 0xFF00) == 0 {
            self.current += 0x0100;
        }
        if (self.current & 0xFF_0000) == 0 {
            self.current += 0x01_0000;
        }
        result
    }

    /// Format as hex color string: "#RRGGBB"
    pub fn color_to_hex(color: u32) -> String {
        format!("#{:06x}", color)
    }
}

// ── SvekUtils ────────────────────────────────────────────────────────

/// Utility functions for DOT generation.
/// Java: `svek.SvekUtils`
pub mod utils {
    /// Convert pixel measurement to inches for Graphviz (72 DPI).
    pub fn pixel_to_inches(px: f64) -> f64 {
        px / 72.0
    }

    /// Format a pixel value as DOT inches string.
    pub fn px_to_dot(px: f64) -> String {
        format!("{:.6}", pixel_to_inches(px))
    }

    /// Default nodesep in inches. Java: `AbstractEntityDiagram.java:61`
    pub const DEFAULT_NODESEP_IN: f64 = 0.35;

    /// Default ranksep in inches.
    pub const DEFAULT_RANKSEP_IN: f64 = 0.65;
}

// ── Bibliotekon ──────────────────────────────────────────────────────

/// Registry of all nodes and edges for a diagram.
/// Java: `svek.Bibliotekon`
///
/// Used during DOT generation to lookup nodes by entity ID,
/// and during SVG parsing to match colors to entities.
pub struct Bibliotekon {
    pub nodes: Vec<node::SvekNode>,
    pub edges: Vec<edge::SvekEdge>,
    pub clusters: Vec<cluster::Cluster>,
}

impl Bibliotekon {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), clusters: Vec::new() }
    }

    pub fn add_node(&mut self, node: node::SvekNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: edge::SvekEdge) {
        self.edges.push(edge);
    }

    pub fn find_node(&self, uid: &str) -> Option<&node::SvekNode> {
        self.nodes.iter().find(|n| n.uid == uid)
    }

    pub fn find_node_mut(&mut self, uid: &str) -> Option<&mut node::SvekNode> {
        self.nodes.iter_mut().find(|n| n.uid == uid)
    }

    pub fn add_cluster(&mut self, cluster: cluster::Cluster) {
        self.clusters.push(cluster);
    }

    pub fn all_nodes(&self) -> &[node::SvekNode] { &self.nodes }
    pub fn all_edges(&self) -> &[edge::SvekEdge] { &self.edges }
}

impl Default for Bibliotekon {
    fn default() -> Self { Self::new() }
}

// ── Margins ──────────────────────────────────────────────────────────

/// Spacing margins around an entity.
/// Java: `svek.Margins`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Margins {
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

impl Margins {
    pub fn none() -> Self {
        Self { x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    pub fn uniform(margin: f64) -> Self {
        Self { x1: margin, x2: margin, y1: margin, y2: margin }
    }

    pub fn new(x1: f64, x2: f64, y1: f64, y2: f64) -> Self {
        Self { x1, x2, y1, y2 }
    }

    pub fn total_width(&self) -> f64 { self.x1 + self.x2 }
    pub fn total_height(&self) -> f64 { self.y1 + self.y2 }

    /// Check if all margins are zero. Java: `Margins.isZero()`
    pub fn is_zero(&self) -> bool {
        self.x1 == 0.0 && self.x2 == 0.0 && self.y1 == 0.0 && self.y2 == 0.0
    }
}

// ── Point2DFunction ──────────────────────────────────────────────────

/// Coordinate transformation function applied when parsing SVG output.
/// Java: `svek.Point2DFunction`
pub trait Point2DFunction {
    fn apply(&self, pt: XPoint2D) -> XPoint2D;
}

/// Identity transform (no coordinate change).
pub struct IdentityFunction;
impl Point2DFunction for IdentityFunction {
    fn apply(&self, pt: XPoint2D) -> XPoint2D { pt }
}

// ── DotStringFactory ─────────────────────────────────────────────────

/// Generates DOT string from a Bibliotekon and parses Graphviz SVG output.
/// Java: `svek.DotStringFactory`
pub struct DotStringFactory {
    pub bibliotekon: Bibliotekon,
    pub rankdir: crate::klimt::geom::Rankdir,
    pub splines: DotSplines,
    pub is_activity: bool,
    pub nodesep_override: Option<f64>,
    pub ranksep_override: Option<f64>,
}

impl DotStringFactory {
    pub fn new(bib: Bibliotekon) -> Self {
        Self {
            bibliotekon: bib,
            rankdir: crate::klimt::geom::Rankdir::TopToBottom,
            splines: DotSplines::Spline,
            is_activity: false,
            nodesep_override: None,
            ranksep_override: None,
        }
    }

    pub fn with_rankdir(mut self, r: crate::klimt::geom::Rankdir) -> Self {
        self.rankdir = r; self
    }
    pub fn with_splines(mut self, s: DotSplines) -> Self {
        self.splines = s; self
    }
    pub fn with_activity(mut self, a: bool) -> Self {
        self.is_activity = a; self
    }

    /// Generate the complete DOT string.
    pub fn create_dot_string(&self, _mode: DotMode) -> String {
        use crate::klimt::geom::Rankdir;
        let mut sb = String::with_capacity(4096);
        sb.push_str("digraph unix {\n");

        // Graph attributes
        let rd = match self.rankdir {
            Rankdir::TopToBottom => "TB",
            Rankdir::LeftToRight => "LR",
            Rankdir::BottomToTop => "BT",
            Rankdir::RightToLeft => "RL",
        };
        sb.push_str(&format!("rankdir={};\n", rd));

        let nodesep = self.nodesep_override
            .map(|px| utils::pixel_to_inches(px))
            .unwrap_or(utils::DEFAULT_NODESEP_IN);
        let ranksep = self.ranksep_override
            .map(|px| utils::pixel_to_inches(px))
            .unwrap_or(utils::DEFAULT_RANKSEP_IN);
        sb.push_str(&format!("nodesep={:.6};\n", nodesep));
        sb.push_str(&format!("ranksep={:.6};\n", ranksep));

        match self.splines {
            DotSplines::Spline => sb.push_str("splines=spline;\n"),
            DotSplines::Polyline => sb.push_str("splines=polyline;\n"),
            DotSplines::Ortho => sb.push_str("splines=ortho;\n"),
            DotSplines::Curved => sb.push_str("splines=curved;\n"),
        }

        // Clusters
        for cluster in &self.bibliotekon.clusters {
            self.write_cluster(&mut sb, cluster);
        }

        // Nodes (not in clusters)
        let clustered: std::collections::HashSet<&str> = self.bibliotekon.clusters.iter()
            .flat_map(|c| c.node_uids.iter().map(|s| s.as_str()))
            .collect();

        for node in &self.bibliotekon.nodes {
            if !clustered.contains(node.uid.as_str()) {
                sb.push_str(&format!(
                    "\"{}\" [shape={},label=\"\",width={:.6},height={:.6},color=\"{}\"];\n",
                    node.uid,
                    node.shape_type.dot_shape(),
                    utils::pixel_to_inches(node.width),
                    utils::pixel_to_inches(node.height),
                    ColorSequence::color_to_hex(node.color),
                ));
            }
        }

        // Edges
        for edge in &self.bibliotekon.edges {
            sb.push_str(&format!("\"{}\" -> \"{}\"", edge.from_uid, edge.to_uid));
            let mut attrs = Vec::new();
            attrs.push(format!("color=\"{}\"", ColorSequence::color_to_hex(edge.color)));
            if let Some(ref label) = edge.label {
                attrs.push(format!("label=\"{}\"", label));
            }
            if !attrs.is_empty() {
                sb.push_str(&format!(" [{}]", attrs.join(",")));
            }
            sb.push_str(";\n");
        }

        sb.push_str("}\n");
        sb
    }

    fn write_cluster(&self, sb: &mut String, cluster: &cluster::Cluster) {
        sb.push_str(&format!("subgraph cluster_{} {{\n", cluster.id));
        if let Some(ref title) = cluster.title {
            sb.push_str(&format!("label=\"{}\";\n", title));
        }
        sb.push_str("margin=\"14\";\n");
        for sub in &cluster.sub_clusters {
            self.write_cluster(sb, sub);
        }
        for uid in &cluster.node_uids {
            if let Some(node) = self.bibliotekon.find_node(uid) {
                sb.push_str(&format!(
                    "\"{}\" [shape={},label=\"\",width={:.6},height={:.6},color=\"{}\"];\n",
                    node.uid,
                    node.shape_type.dot_shape(),
                    utils::pixel_to_inches(node.width),
                    utils::pixel_to_inches(node.height),
                    ColorSequence::color_to_hex(node.color),
                ));
            }
        }
        sb.push_str("}\n");
    }

    /// Parse SVG output and position nodes/edges.
    pub fn solve(&mut self, _svg: &str) -> Result<(), String> {
        // TODO: Full SVG parsing implementation
        Ok(())
    }

    /// Move all positioned elements by delta.
    pub fn move_delta(&mut self, dx: f64, dy: f64) {
        for node in &mut self.bibliotekon.nodes {
            node.cx += dx;
            node.cy += dy;
            node.min_x += dx;
            node.min_y += dy;
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_sequence_unique() {
        let mut cs = ColorSequence::new();
        let c1 = cs.next_color();
        let c2 = cs.next_color();
        let c3 = cs.next_color();
        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
        assert_ne!(c1, c3);
    }

    #[test]
    fn color_to_hex() {
        assert_eq!(ColorSequence::color_to_hex(0xFF0000), "#ff0000");
        assert_eq!(ColorSequence::color_to_hex(0x010100), "#010100");
    }

    #[test]
    fn pixel_to_inches() {
        assert!((utils::pixel_to_inches(72.0) - 1.0).abs() < 1e-10);
        assert!((utils::pixel_to_inches(36.0) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn margins_total() {
        let m = Margins::new(10.0, 20.0, 5.0, 15.0);
        assert_eq!(m.total_width(), 30.0);
        assert_eq!(m.total_height(), 20.0);
    }

    #[test]
    fn bibliotekon_find() {
        let mut b = Bibliotekon::new();
        b.add_node(node::SvekNode::new("n1", 100.0, 50.0));
        b.add_node(node::SvekNode::new("n2", 80.0, 40.0));
        assert!(b.find_node("n1").is_some());
        assert!(b.find_node("n3").is_none());
    }
}
