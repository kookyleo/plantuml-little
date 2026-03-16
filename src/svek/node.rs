// svek::node - Graph node representation for Graphviz layout
// Port of Java PlantUML's svek.SvekNode

use crate::klimt::geom::XDimension2D;

/// A node in the Graphviz layout graph.
/// Java: `svek.SvekNode`
///
/// Holds the entity's dimensions (for DOT input) and
/// positioned coordinates (from SVG output parsing).
#[derive(Debug, Clone)]
pub struct SvekNode {
    pub uid: String,
    pub width: f64,
    pub height: f64,
    /// Position after Graphviz layout (center x, center y)
    pub cx: f64,
    pub cy: f64,
    /// DOT color used for SVG matching
    pub color: u32,
    /// Shape type for DOT shape attribute
    pub shape_type: super::shape_type::ShapeType,
    /// Cluster membership (if any)
    pub cluster_id: Option<String>,
}

impl SvekNode {
    pub fn new(uid: &str, width: f64, height: f64) -> Self {
        Self {
            uid: uid.to_string(),
            width, height,
            cx: 0.0, cy: 0.0,
            color: 0,
            shape_type: super::shape_type::ShapeType::Rectangle,
            cluster_id: None,
        }
    }

    pub fn dimension(&self) -> XDimension2D {
        XDimension2D::new(self.width, self.height)
    }

    /// Top-left x after layout
    pub fn x(&self) -> f64 { self.cx - self.width / 2.0 }
    /// Top-left y after layout
    pub fn y(&self) -> f64 { self.cy - self.height / 2.0 }
}

// TODO: Phase 2 - full port of SvekNode DOT generation + SVG parsing

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_position() {
        let mut n = SvekNode::new("test", 100.0, 50.0);
        n.cx = 150.0;
        n.cy = 125.0;
        assert_eq!(n.x(), 100.0);
        assert_eq!(n.y(), 100.0);
    }
}
