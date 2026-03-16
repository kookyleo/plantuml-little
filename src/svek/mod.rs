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
