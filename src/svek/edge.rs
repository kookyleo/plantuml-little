// svek::edge - Graph edge representation for Graphviz layout
// Port of Java PlantUML's svek.SvekEdge (1350 lines)
//
// TODO: Full port pending - this is the largest svek file

use crate::klimt::geom::XPoint2D;
use crate::klimt::shape::DotPath;

/// An edge in the Graphviz layout graph.
/// Java: `svek.SvekEdge`
#[derive(Debug, Clone)]
pub struct SvekEdge {
    pub from_uid: String,
    pub to_uid: String,
    /// DOT color for SVG matching
    pub color: u32,
    /// Bezier path after layout
    pub dot_path: Option<DotPath>,
    /// Arrow polygon points (if any)
    pub arrow_head: Option<Vec<XPoint2D>>,
    pub arrow_tail: Option<Vec<XPoint2D>>,
    /// Edge label text
    pub label: Option<String>,
    pub label_xy: Option<XPoint2D>,
}

impl SvekEdge {
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            from_uid: from.to_string(),
            to_uid: to.to_string(),
            color: 0,
            dot_path: None,
            arrow_head: None,
            arrow_tail: None,
            label: None,
            label_xy: None,
        }
    }
}

// TODO: Full port of appendLine(), DOT generation, SVG parsing, extremity attachment

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_basic() {
        let e = SvekEdge::new("A", "B");
        assert_eq!(e.from_uid, "A");
        assert_eq!(e.to_uid, "B");
        assert!(e.dot_path.is_none());
    }
}
