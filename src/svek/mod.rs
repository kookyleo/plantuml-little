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

        // Java: DotStringFactory.createDotString() — remincross + searchsize
        if !self.is_activity {
            sb.push_str("remincross=true;\n");
        }
        sb.push_str("searchsize=500;\n");

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
                node.append_shape(&mut sb);
            }
        }

        // Edges — Java: DotStringFactory iterates biblio.allLines()
        for edge in &self.bibliotekon.edges {
            edge.append_line(&mut sb);
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
                node.append_shape(sb);
            }
        }
        sb.push_str("}\n");
    }

    /// Parse SVG output and position nodes/edges.
    /// Java: `DotStringFactory.solve(SvgResult)`
    ///
    /// 1. For each node: find polygon/ellipse by color → extract position
    /// 2. For each edge: call `solve_line()` to extract path + labels
    /// 3. Normalize coordinates (shift so min position = (6, 6))
    pub fn solve(&mut self, svg: &str) -> Result<(), String> {
        use crate::svek::svg_result::SvgResult;

        // Parse translate(tx, ty) from Graphviz SVG top-level <g> transform.
        // Graphviz coordinates are in internal space (Y negative); translate
        // converts to SVG viewport coordinates.
        let (tx, ty) = parse_svg_translate(svg);
        let svg_result = SvgResult::new(svg.to_string());

        // Position nodes by finding their polygons via color
        for node in &mut self.bibliotekon.nodes {
            if node.hidden {
                continue;
            }
            let Some(idx) = svg_result.find_by_color(node.color) else {
                continue;
            };
            // Extract polygon points from SVG and apply translate
            let raw_points = svg_result.extract_points_at(idx);
            if !raw_points.is_empty() {
                let points: Vec<XPoint2D> = raw_points
                    .iter()
                    .map(|p| XPoint2D::new(p.x + tx, p.y + ty))
                    .collect();
                let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
                let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
                node.min_x = min_x;
                node.min_y = min_y;
                node.cx = min_x + node.width / 2.0;
                node.cy = min_y + node.height / 2.0;
                node.set_polygon(min_x, min_y, &points);
            } else {
                // Try ellipse: cx/cy attributes near the color position
                let svg_str = svg_result.svg();
                if let Some(cx) = parse_xml_attr_near(svg_str, idx, "cx") {
                    if let Some(cy) = parse_xml_attr_near(svg_str, idx, "cy") {
                        let rx = parse_xml_attr_near(svg_str, idx, "rx").unwrap_or(0.0);
                        let ry = parse_xml_attr_near(svg_str, idx, "ry").unwrap_or(0.0);
                        node.min_x = (cx + tx) - rx;
                        node.min_y = (cy + ty) - ry;
                        node.cx = cx + tx;
                        node.cy = cy + ty;
                    }
                }
            }
        }

        // Position edges — extract_dot_path handles raw SVG coords;
        // apply translate to resulting paths
        for edge in &mut self.bibliotekon.edges {
            edge.solve_line(&svg_result);
            // Apply translate to extracted path
            if tx != 0.0 || ty != 0.0 {
                if let Some(ref mut path) = edge.dot_path {
                    path.move_delta(tx, ty);
                }
                if let Some(ref mut path) = edge.dot_path_init {
                    path.move_delta(tx, ty);
                }
                if let Some(ref mut pt) = edge.label_xy {
                    pt.x += tx;
                    pt.y += ty;
                }
                if let Some(ref mut pt) = edge.start_tail_label_xy {
                    pt.x += tx;
                    pt.y += ty;
                }
                if let Some(ref mut pt) = edge.end_head_label_xy {
                    pt.x += tx;
                    pt.y += ty;
                }
            }
        }

        // Normalize: compute bounding box and shift to origin + margin(6)
        // Java: SvekResult.java:133 — moveDelta(6 - minMax.getMinX(), 6 - minMax.getMinY())
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        for node in &self.bibliotekon.nodes {
            if node.hidden {
                continue;
            }
            if node.min_x < min_x {
                min_x = node.min_x;
            }
            if node.min_y < min_y {
                min_y = node.min_y;
            }
        }
        if min_x.is_finite() && min_y.is_finite() {
            let dx = 6.0 - min_x;
            let dy = 6.0 - min_y;
            self.move_delta(dx, dy);
        }

        Ok(())
    }

    /// Move all positioned elements by delta.
    /// Java: `SvekResult.moveDelta()` — moves nodes and edges.
    pub fn move_delta(&mut self, dx: f64, dy: f64) {
        for node in &mut self.bibliotekon.nodes {
            node.cx += dx;
            node.cy += dy;
            node.min_x += dx;
            node.min_y += dy;
        }
        for edge in &mut self.bibliotekon.edges {
            edge.move_delta(dx, dy);
        }
    }
}

/// Coordinate transform that applies translate(tx, ty).
struct TranslateFunction {
    tx: f64,
    ty: f64,
}

impl Point2DFunction for TranslateFunction {
    fn apply(&self, pt: XPoint2D) -> XPoint2D {
        XPoint2D::new(pt.x + self.tx, pt.y + self.ty)
    }
}

/// Parse `translate(tx, ty)` from Graphviz SVG top-level `<g>` transform.
fn parse_svg_translate(svg: &str) -> (f64, f64) {
    if let Some(pos) = svg.find("translate(") {
        let after = &svg[pos + 10..];
        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            let parts: Vec<&str> = inner.split(|c: char| c == ' ' || c == ',').collect();
            if parts.len() >= 2 {
                let tx: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let ty: f64 = parts[1].trim().parse().unwrap_or(0.0);
                return (tx, ty);
            }
        }
    }
    (0.0, 0.0)
}

/// Parse a numeric XML attribute value near a given position.
/// Searches forward from `from` within a reasonable range for `attr_name="value"`.
fn parse_xml_attr_near(svg: &str, from: usize, attr_name: &str) -> Option<f64> {
    // Search within next 200 chars for the element containing this attribute
    let end = (from + 200).min(svg.len());
    let search = &svg[from..end];
    let needle = format!("{}=\"", attr_name);
    let pos = search.find(&needle)?;
    let val_start = pos + needle.len();
    let val_end = search[val_start..].find('"')?;
    search[val_start..val_start + val_end].parse().ok()
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

    #[test]
    fn parse_xml_attr_near_basic() {
        let svg = r#"<ellipse cx="54" cy="18" rx="27" ry="18"/>"#;
        assert_eq!(parse_xml_attr_near(svg, 0, "cx"), Some(54.0));
        assert_eq!(parse_xml_attr_near(svg, 0, "cy"), Some(18.0));
        assert_eq!(parse_xml_attr_near(svg, 0, "rx"), Some(27.0));
        assert_eq!(parse_xml_attr_near(svg, 0, "ry"), Some(18.0));
        assert_eq!(parse_xml_attr_near(svg, 0, "zz"), None);
    }

    #[test]
    fn solve_positions_nodes_from_polygon() {
        // Simulate Graphviz SVG output with two nodes identified by color
        let svg = concat!(
            r##"<svg><g>"##,
            r##"<polygon fill="none" stroke="#010100" points="100,0 200,0 200,50 100,50 100,0"/>"##,
            r##"<polygon fill="none" stroke="#020200" points="50,80 150,80 150,130 50,130 50,80"/>"##,
            r##"<path fill="none" stroke="#030300" d="M 150,25 C 140,50 120,70 100,105"/>"##,
            r##"</g></svg>"##,
        );

        let mut bib = Bibliotekon::new();
        let mut n1 = node::SvekNode::new("n1", 100.0, 50.0);
        n1.color = 0x010100;
        let mut n2 = node::SvekNode::new("n2", 100.0, 50.0);
        n2.color = 0x020200;
        bib.add_node(n1);
        bib.add_node(n2);

        let mut e = edge::SvekEdge::new("n1", "n2");
        e.color = 0x030300;
        bib.add_edge(e);

        let mut factory = DotStringFactory::new(bib);
        factory.solve(svg).unwrap();

        // After solve + normalization (min should be at 6.0)
        let n1 = factory.bibliotekon.find_node("n1").unwrap();
        let n2 = factory.bibliotekon.find_node("n2").unwrap();

        // Original min was (50, 0), so delta = (6-50, 6-0) = (-44, 6)
        // n1 polygon min was (100, 0) → min_x = 100 + (-44) = 56
        assert!((n1.min_x - 56.0).abs() < 0.01, "n1.min_x={}", n1.min_x);
        assert!((n1.min_y - 6.0).abs() < 0.01, "n1.min_y={}", n1.min_y);
        // n2 polygon min was (50, 80) → min_x = 50 + (-44) = 6
        assert!((n2.min_x - 6.0).abs() < 0.01, "n2.min_x={}", n2.min_x);
        assert!((n2.min_y - 86.0).abs() < 0.01, "n2.min_y={}", n2.min_y);

        // Edge should have a path
        let edge = &factory.bibliotekon.edges[0];
        assert!(edge.dot_path.is_some(), "edge should have a dot_path");
    }

    #[test]
    fn solve_empty_svg() {
        let mut bib = Bibliotekon::new();
        let mut n = node::SvekNode::new("n1", 100.0, 50.0);
        n.color = 0x010100;
        bib.add_node(n);

        let mut factory = DotStringFactory::new(bib);
        // Empty SVG should not crash
        factory.solve("<svg></svg>").unwrap();

        let n = factory.bibliotekon.find_node("n1").unwrap();
        // Node not found in SVG, but normalization still shifts to (6, 6)
        assert_eq!(n.min_x, 6.0);
    }
}
