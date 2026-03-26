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
use crate::svek::edge::{append_table, LabelDimension};

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
        // Java: DotStringFactory omits rankdir=TB (the default)
        if self.rankdir != Rankdir::TopToBottom {
            let rd = match self.rankdir {
                Rankdir::TopToBottom => "TB",
                Rankdir::LeftToRight => "LR",
                Rankdir::BottomToTop => "BT",
                Rankdir::RightToLeft => "RL",
            };
            sb.push_str(&format!("rankdir={};\n", rd));
        }

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

        // Java: DotStringFactory omits splines=spline (the default)
        match self.splines {
            DotSplines::Spline => { /* default, omit */ }
            DotSplines::Polyline => sb.push_str("splines=polyline;\n"),
            DotSplines::Ortho => sb.push_str("splines=ortho;\n"),
            DotSplines::Curved => sb.push_str("splines=curved;\n"),
        }

        // Java DotStringFactory:
        //   root.printCluster1(...)
        //   for line in lines0()
        //   root.printCluster2(...)
        //   for line in lines1()
        //
        // We do not yet port EntityPosition.TOP ordering, but for class/package
        // diagrams the important behavior is:
        // 1. length==1 links are emitted before cluster bodies
        // 2. remaining links are emitted after cluster bodies
        for edge in &self.bibliotekon.edges {
            if edge.is_horizontal() {
                edge.append_line(&mut sb);
            }
        }

        // Clusters + nodes
        for cluster in &self.bibliotekon.clusters {
            self.write_cluster(&mut sb, cluster);
        }

        let mut clustered = std::collections::HashSet::new();
        for cluster in &self.bibliotekon.clusters {
            collect_clustered_nodes(cluster, &mut clustered);
        }
        for node in &self.bibliotekon.nodes {
            if !clustered.contains(node.uid.as_str()) {
                node.append_shape(&mut sb);
            }
        }

        for edge in &self.bibliotekon.edges {
            if !edge.is_horizontal() {
                edge.append_line(&mut sb);
            }
        }

        sb.push_str("}\n");
        sb
    }

    fn write_cluster(&self, sb: &mut String, cluster: &cluster::Cluster) {
        // Java ClusterDotString wraps the actual cluster inside unlabeled p0/p1
        // protection clusters. Those wrappers materially affect Graphviz
        // geometry, especially for nested packages and self-loop labels.
        sb.push_str(&format!("subgraph cluster_{}p0 {{\n", cluster.id));
        sb.push_str("label=\"\";\n");
        sb.push_str(&format!("subgraph cluster_{} {{\n", cluster.id));
        sb.push_str("style=solid;\n");
        sb.push_str("color=\"#000000\";\n");
        if let Some(ref title) = cluster.title {
            sb.push_str("labeljust=\"c\";\n");
            sb.push_str("label=<");
            let (title_width, title_height) = cluster_title_label_dim(title);
            append_table(
                sb,
                LabelDimension::new(title_width as f64, ((title_height - 5).max(1)) as f64),
                0x000000,
            );
            sb.push_str(">;\n");
        } else {
            sb.push_str("label=\"\";\n");
        }
        sb.push_str(&format!("subgraph cluster_{}p1 {{\n", cluster.id));
        sb.push_str("label=\"\";\n");

        // Java Cluster.printCluster2() writes normal nodes before child
        // clusters. That ordering affects nested package geometry.
        for uid in &cluster.node_uids {
            if let Some(node) = self.bibliotekon.find_node(uid) {
                node.append_shape(sb);
            }
        }
        for sub in &cluster.sub_clusters {
            self.write_cluster(sb, sub);
        }

        sb.push_str("}\n");
        sb.push_str("}\n");
        sb.push_str("}\n");
    }

    /// Parse SVG output and position nodes/edges.
    /// Java: `DotStringFactory.solve(SvgResult)`
    ///
    /// 1. For each node: find polygon/ellipse by color → extract position
    /// 2. For each edge: call `solve_line()` to extract path + labels
    /// 3. Normalize coordinates (shift so min position = (6, 6))
    /// Returns (moveDelta, limitFinder_span) from normalization.
    pub fn solve(&mut self, svg: &str) -> Result<((f64, f64), (f64, f64)), String> {
        use crate::svek::svg_result::SvgResult;

        // Parse translate(tx, ty) from Graphviz SVG top-level <g> transform.
        // Graphviz coordinates are in internal space (Y negative); translate
        // converts to SVG viewport coordinates.
        let (tx, ty) = parse_svg_translate(svg);
        let svg_result = SvgResult::new(svg.to_string());
        for cluster in &mut self.bibliotekon.clusters {
            solve_cluster_positions(svg, cluster, tx, ty);
        }

        // Position nodes by finding their polygons via color
        for node in &mut self.bibliotekon.nodes {
            if node.hidden {
                continue;
            }
            let Some(idx) = svg_result.find_by_color(node.color) else {
                continue;
            };
            // Try ellipse first: if cx/cy attributes exist near the color
            // position, this is a circle node (shape=circle in DOT).
            let svg_str = svg_result.svg();
            let is_ellipse = parse_xml_attr_near(svg_str, idx, "cx").is_some();
            if is_ellipse {
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
            } else {
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

        // ── LimitFinder span (before moveDelta) ──
        // Java Pass 1: LimitFinder.drawRectangle(x, y, w, h) → addPoint(x-1, y-1), addPoint(x+w-1, y+h-1)
        // Java Pass 1: LimitFinder.drawEmpty for edge labels → addPoint(x, y), addPoint(x+w, y+h)
        // We simulate this on the unshifted svek coordinates to get the exact span.
        let mut lf_min_x = f64::INFINITY;
        let mut lf_min_y = f64::INFINITY;
        let mut lf_max_x = f64::NEG_INFINITY;
        let mut lf_max_y = f64::NEG_INFINITY;
        for node in &self.bibliotekon.nodes {
            if node.hidden { continue; }
            // LimitFinder.drawRectangle: (x-1, y-1) to (x+w-1, y+h-1)
            let rx = node.min_x - 1.0;
            let ry = node.min_y - 1.0;
            let rr = node.min_x + node.width - 1.0;
            let rb = node.min_y + node.height - 1.0;
            if rx < lf_min_x { lf_min_x = rx; }
            if ry < lf_min_y { lf_min_y = ry; }
            if rr > lf_max_x { lf_max_x = rr; }
            if rb > lf_max_y { lf_max_y = rb; }
        }
        for edge in &self.bibliotekon.edges {
            // LimitFinder.drawEmpty for edge label block
            if let (Some(ref pt), Some(ref dim)) = (&edge.label_xy, &edge.label_dimension) {
                let dim_w = if edge.divide_label_width_by_two { dim.width / 2.0 } else { dim.width };
                let shielded_w = dim_w + 2.0 * edge.label_shield;
                let shielded_h = dim.height + 2.0 * edge.label_shield;
                let lx = pt.x - shielded_w / 2.0;
                let ly = pt.y - shielded_h / 2.0;
                // drawEmpty: (x, y) to (x+w, y+h) — no -1
                if lx < lf_min_x { lf_min_x = lx; }
                if ly < lf_min_y { lf_min_y = ly; }
                let lr = lx + shielded_w;
                let lb = ly + shielded_h;
                if lr > lf_max_x { lf_max_x = lr; }
                if lb > lf_max_y { lf_max_y = lb; }
            }
        }
        for cluster in &self.bibliotekon.clusters {
            extend_lf_with_cluster(cluster, &mut lf_min_x, &mut lf_min_y, &mut lf_max_x, &mut lf_max_y);
        }
        log::debug!("svek solve LF: min=({:.1},{:.1}) max=({:.1},{:.1})", lf_min_x, lf_min_y, lf_max_x, lf_max_y);
        let lf_span = if lf_max_x.is_finite() && lf_min_x.is_finite() {
            (lf_max_x - lf_min_x, lf_max_y - lf_min_y)
        } else {
            (0.0, 0.0)
        };

        // ── moveDelta ──
        // Use node polygon min (not LimitFinder min) for moveDelta.
        // Java: moveDelta(6 - LF_minX, 6 - LF_minY). But our renderer uses
        // edge_offset = moveDelta + 1 to compensate. So we keep moveDelta = 6 - polygon_min.
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        for node in &self.bibliotekon.nodes {
            if node.hidden { continue; }
            if node.min_x < min_x { min_x = node.min_x; }
            if node.min_y < min_y { min_y = node.min_y; }
        }
        for cluster in &self.bibliotekon.clusters {
            extend_min_with_cluster(cluster, &mut min_x, &mut min_y);
        }
        let (dx, dy) = if min_x.is_finite() && min_y.is_finite() {
            let dx = 6.0 - min_x;
            let dy = 6.0 - min_y;
            self.move_delta(dx, dy);
            (dx, dy)
        } else {
            (0.0, 0.0)
        };

        Ok(((dx, dy), lf_span))
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
        for cluster in &mut self.bibliotekon.clusters {
            move_cluster_delta(cluster, dx, dy);
        }
    }
}

fn cluster_title_label_dim(title: &str) -> (i32, i32) {
    let width = crate::font_metrics::text_width(title, "SansSerif", 14.0, true, false).floor() as i32;
    let height =
        crate::font_metrics::line_height("SansSerif", 14.0, true, false).floor() as i32;
    (width.max(0), height.max(0))
}

fn collect_clustered_nodes<'a>(
    cluster: &'a cluster::Cluster,
    clustered: &mut std::collections::HashSet<&'a str>,
) {
    for uid in &cluster.node_uids {
        clustered.insert(uid.as_str());
    }
    for sub in &cluster.sub_clusters {
        collect_clustered_nodes(sub, clustered);
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

fn move_cluster_delta(cluster: &mut cluster::Cluster, dx: f64, dy: f64) {
    cluster.x += dx;
    cluster.y += dy;
    for sub in &mut cluster.sub_clusters {
        move_cluster_delta(sub, dx, dy);
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

fn solve_cluster_positions(svg: &str, cluster: &mut cluster::Cluster, tx: f64, ty: f64) {
    if let Some((x, y, width, height)) = parse_svg_cluster_bounds(svg, &cluster.id, tx, ty) {
        cluster.x = x;
        cluster.y = y;
        cluster.width = width;
        cluster.height = height;
    }
    for sub in &mut cluster.sub_clusters {
        solve_cluster_positions(svg, sub, tx, ty);
    }
}

fn parse_svg_cluster_bounds(svg: &str, cluster_id: &str, tx: f64, ty: f64) -> Option<(f64, f64, f64, f64)> {
    let title = format!("<title>cluster_{cluster_id}</title>");
    let title_pos = svg.find(&title)?;
    let start = title_pos;
    let mut end = (start + 600).min(svg.len());
    while end > start && !svg.is_char_boundary(end) {
        end -= 1;
    }
    let search = &svg[start..end];
    let polygon_pos = search.find("<polygon")?;
    let polygon = &search[polygon_pos..];
    let points = parse_points_attr(polygon)?;
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for pair in points.split_whitespace() {
        let mut coords = pair.split(',');
        let x: f64 = coords.next()?.parse().ok()?;
        let y: f64 = coords.next()?.parse().ok()?;
        min_x = min_x.min(x + tx);
        min_y = min_y.min(y + ty);
        max_x = max_x.max(x + tx);
        max_y = max_y.max(y + ty);
    }
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

fn parse_points_attr(elem: &str) -> Option<&str> {
    let needle = "points=\"";
    let pos = elem.find(needle)?;
    let after = &elem[pos + needle.len()..];
    let end = after.find('"')?;
    Some(&after[..end])
}

fn extend_lf_with_cluster(
    cluster: &cluster::Cluster,
    min_x: &mut f64,
    min_y: &mut f64,
    max_x: &mut f64,
    max_y: &mut f64,
) {
    if cluster.width > 0.0 && cluster.height > 0.0 {
        if cluster.x < *min_x { *min_x = cluster.x; }
        if cluster.y < *min_y { *min_y = cluster.y; }
        let right = cluster.x + cluster.width;
        let bottom = cluster.y + cluster.height;
        if right > *max_x { *max_x = right; }
        if bottom > *max_y { *max_y = bottom; }
    }
    for sub in &cluster.sub_clusters {
        extend_lf_with_cluster(sub, min_x, min_y, max_x, max_y);
    }
}

fn extend_min_with_cluster(cluster: &cluster::Cluster, min_x: &mut f64, min_y: &mut f64) {
    if cluster.width > 0.0 && cluster.height > 0.0 {
        if cluster.x < *min_x { *min_x = cluster.x; }
        if cluster.y < *min_y { *min_y = cluster.y; }
    }
    for sub in &cluster.sub_clusters {
        extend_min_with_cluster(sub, min_x, min_y);
    }
}

/// Parse a numeric XML attribute value near a given position.
/// Searches forward from `from` within a reasonable range for `attr_name="value"`.
fn parse_xml_attr_near(svg: &str, from: usize, attr_name: &str) -> Option<f64> {
    // Search within the next ~200 bytes, but keep UTF-8 boundaries intact.
    let mut start = from.min(svg.len());
    while start < svg.len() && !svg.is_char_boundary(start) {
        start += 1;
    }
    let mut end = (start + 200).min(svg.len());
    while end > start && !svg.is_char_boundary(end) {
        end -= 1;
    }
    let search = &svg[start..end];
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
    fn parse_xml_attr_near_handles_multibyte_window_end() {
        let svg = format!("{}cx=\"54\"≤z", "a".repeat(191));
        assert_eq!(parse_xml_attr_near(&svg, 0, "cx"), Some(54.0));
    }

    #[test]
    fn parse_svg_cluster_bounds_handles_multibyte_window_end() {
        let prefix = "a".repeat(560);
        let svg = format!(
            r#"{prefix}<title>cluster_demo</title><polygon points="10,20 30,20 30,40 10,40"/>´"#
        );
        assert_eq!(
            parse_svg_cluster_bounds(&svg, "demo", 0.0, 0.0),
            Some((10.0, 20.0, 20.0, 20.0))
        );
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

    #[test]
    fn create_dot_string_does_not_repeat_nested_cluster_nodes() {
        let mut bib = Bibliotekon::new();
        bib.add_node(node::SvekNode::new("A", 100.0, 50.0));
        bib.add_node(node::SvekNode::new("B", 80.0, 40.0));

        let mut outer = cluster::Cluster::new("outer");
        outer.add_node("A");
        let mut inner = cluster::Cluster::new("inner");
        inner.add_node("B");
        outer.sub_clusters.push(inner);
        bib.add_cluster(outer);

        let factory = DotStringFactory::new(bib);
        let dot = factory.create_dot_string(DotMode::Normal);

        assert_eq!(dot.matches("\"A\" [").count(), 1);
        assert_eq!(dot.matches("\"B\" [").count(), 1);
    }

    #[test]
    fn cluster_title_label_dim_matches_java_cluster_header() {
        assert_eq!(cluster_title_label_dim("pkg1"), (39, 16));
    }
}
