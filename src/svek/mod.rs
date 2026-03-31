// svek - Graphviz layout engine wrapper
// Port of Java PlantUML's net.sourceforge.plantuml.svek package
//
// Named "svek" = SVG + Graphviz Engine (K?)
// Workflow: Model → DOT string → Graphviz → SVG → parse coordinates → redraw via klimt

pub mod builder;
pub mod cluster;
pub mod edge;
pub mod extremity;
pub mod image;
pub mod node;
pub mod shape_type;
pub mod snake;
pub mod svg_result;

use crate::klimt::geom::{Rankdir, RectangleArea, XPoint2D};
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
        Self {
            current: 0x0001_0100,
        }
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
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            clusters: Vec::new(),
        }
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

    pub fn all_nodes(&self) -> &[node::SvekNode] {
        &self.nodes
    }
    pub fn all_edges(&self) -> &[edge::SvekEdge] {
        &self.edges
    }
}

impl Default for Bibliotekon {
    fn default() -> Self {
        Self::new()
    }
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
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn uniform(margin: f64) -> Self {
        Self {
            x1: margin,
            x2: margin,
            y1: margin,
            y2: margin,
        }
    }

    pub fn new(x1: f64, x2: f64, y1: f64, y2: f64) -> Self {
        Self { x1, x2, y1, y2 }
    }

    pub fn total_width(&self) -> f64 {
        self.x1 + self.x2
    }
    pub fn total_height(&self) -> f64 {
        self.y1 + self.y2
    }

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
    fn apply(&self, pt: XPoint2D) -> XPoint2D {
        pt
    }
}

// ── DotStringFactory ─────────────────────────────────────────────────

/// Generates DOT string from a Bibliotekon and parses Graphviz SVG output.
/// Java: `svek.DotStringFactory`
#[derive(Debug, Clone)]
pub enum TopLevelDotItem {
    Node(String),
    Cluster(String),
}

pub struct DotStringFactory {
    pub bibliotekon: Bibliotekon,
    pub rankdir: crate::klimt::geom::Rankdir,
    pub splines: DotSplines,
    pub is_activity: bool,
    pub nodesep_override: Option<f64>,
    pub ranksep_override: Option<f64>,
    pub top_level_items: Vec<TopLevelDotItem>,
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
            top_level_items: Vec::new(),
        }
    }

    pub fn with_rankdir(mut self, r: crate::klimt::geom::Rankdir) -> Self {
        self.rankdir = r;
        self
    }
    pub fn with_splines(mut self, s: DotSplines) -> Self {
        self.splines = s;
        self
    }
    pub fn with_activity(mut self, a: bool) -> Self {
        self.is_activity = a;
        self
    }
    pub fn with_top_level_items(mut self, items: Vec<TopLevelDotItem>) -> Self {
        self.top_level_items = items;
        self
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

        let nodesep = self
            .nodesep_override
            .map(|px| utils::pixel_to_inches(px))
            .unwrap_or(utils::DEFAULT_NODESEP_IN);
        let ranksep = self
            .ranksep_override
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

        let mut clustered = std::collections::HashSet::new();
        for cluster in &self.bibliotekon.clusters {
            collect_clustered_nodes(cluster, &mut clustered);
        }

        if self.top_level_items.is_empty() {
            for cluster in &self.bibliotekon.clusters {
                self.write_cluster(&mut sb, cluster);
            }
            for node in &self.bibliotekon.nodes {
                if !clustered.contains(node.uid.as_str()) {
                    node.append_shape(&mut sb);
                }
            }
        } else {
            let clusters_by_id: std::collections::HashMap<&str, &cluster::Cluster> = self
                .bibliotekon
                .clusters
                .iter()
                .map(|cluster| (cluster.id.as_str(), cluster))
                .collect();
            let free_nodes_by_id: std::collections::HashMap<&str, &node::SvekNode> = self
                .bibliotekon
                .nodes
                .iter()
                .filter(|node| !clustered.contains(node.uid.as_str()))
                .map(|node| (node.uid.as_str(), node))
                .collect();
            let mut emitted_clusters = std::collections::HashSet::new();
            let mut emitted_nodes = std::collections::HashSet::new();

            for item in &self.top_level_items {
                match item {
                    TopLevelDotItem::Cluster(id) => {
                        if let Some(cluster) = clusters_by_id.get(id.as_str()) {
                            self.write_cluster(&mut sb, cluster);
                            emitted_clusters.insert(id.as_str());
                        }
                    }
                    TopLevelDotItem::Node(id) => {
                        if let Some(node) = free_nodes_by_id.get(id.as_str()) {
                            node.append_shape(&mut sb);
                            emitted_nodes.insert(id.as_str());
                        }
                    }
                }
            }

            for cluster in &self.bibliotekon.clusters {
                if !emitted_clusters.contains(cluster.id.as_str()) {
                    self.write_cluster(&mut sb, cluster);
                }
            }
            for node in &self.bibliotekon.nodes {
                if !clustered.contains(node.uid.as_str())
                    && !emitted_nodes.contains(node.uid.as_str())
                {
                    node.append_shape(&mut sb);
                }
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
        let cluster_nodes: Vec<&node::SvekNode> = cluster
            .node_uids
            .iter()
            .filter_map(|uid| self.bibliotekon.find_node(uid))
            .collect();
        let has_non_normal = cluster_nodes
            .iter()
            .any(|node| !node.entity_position.is_normal());
        let has_port = cluster_nodes
            .iter()
            .any(|node| node.entity_position.is_port());
        let cluster_label = cluster_dot_label(cluster);

        if has_non_normal {
            sb.push_str(&format!("subgraph cluster_{} {{\n", cluster.id));
            sb.push_str("style=solid;\n");
            sb.push_str("color=\"#000000\";\n");

            self.write_cluster_rank(sb, "source", &cluster_nodes, cluster, |node| {
                node.entity_position.is_input()
            });
            self.write_cluster_rank(sb, "sink", &cluster_nodes, cluster, |node| {
                node.entity_position.is_output()
            });

            if has_port {
                sb.push_str(&format!("subgraph cluster_{}ee {{\n", cluster.id));
                sb.push_str("label=\"\";\n");
            } else {
                sb.push_str(&format!("subgraph cluster_{}ee {{\n", cluster.id));
                sb.push_str(&format!("label={cluster_label};\n"));
            }

            for uid in &cluster.node_uids {
                if let Some(node) = self.bibliotekon.find_node(uid) {
                    if node.entity_position.is_normal() {
                        node.append_shape(sb);
                    }
                }
            }
            for sub in &cluster.sub_clusters {
                self.write_cluster(sb, sub);
            }

            if has_port {
                sb.push_str(&format!(
                    "{} [shape=rect,width=.01,height=.01,label={}];\n",
                    cluster_special_point_id(cluster),
                    cluster_label,
                ));
            }

            sb.push_str("}\n");
            sb.push_str("}\n");
            return;
        }

        // Java ClusterDotString wraps the actual cluster inside unlabeled p0/p1
        // protection clusters. Those wrappers materially affect Graphviz
        // geometry, especially for nested packages and self-loop labels.
        sb.push_str(&format!("subgraph cluster_{}p0 {{\n", cluster.id));
        sb.push_str("label=\"\";\n");
        sb.push_str(&format!("subgraph cluster_{} {{\n", cluster.id));
        sb.push_str("style=solid;\n");
        sb.push_str("color=\"#000000\";\n");
        if cluster_label != "\"\"" {
            sb.push_str("labeljust=\"c\";\n");
            sb.push_str(&format!("label={cluster_label};\n"));
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

    fn write_cluster_rank<F>(
        &self,
        sb: &mut String,
        rank: &str,
        cluster_nodes: &[&node::SvekNode],
        cluster: &cluster::Cluster,
        predicate: F,
    ) where
        F: Fn(&node::SvekNode) -> bool,
    {
        let entries: Vec<&node::SvekNode> = cluster_nodes
            .iter()
            .copied()
            .filter(|node| predicate(node))
            .collect();
        if entries.is_empty() {
            return;
        }

        sb.push_str(&format!("{{rank={rank};"));
        for node in &entries {
            sb.push_str(&node.uid);
            sb.push(';');
        }
        sb.push_str("}\n");

        for node in &entries {
            node.append_shape(sb);
        }

        if entries.iter().any(|node| node.entity_position.is_port()) {
            let mut iter = entries.iter();
            if let Some(first) = iter.next() {
                sb.push_str(&first.uid);
                for node in iter {
                    sb.push_str("->");
                    sb.push_str(&node.uid);
                }
                sb.push_str(" [arrowhead=none];\n");
                sb.push_str(&format!(
                    "{}->{};\n",
                    entries.last().unwrap().uid,
                    cluster_special_point_id(cluster),
                ));
            }
        }
    }

    /// Parse SVG output and position nodes/edges.
    /// Java: `DotStringFactory.solve(SvgResult)`
    ///
    /// 1. For each node: find polygon/ellipse by color → extract position
    /// 2. For each edge: call `solve_line()` to extract path + labels
    /// 3. Normalize coordinates (shift so min position = (6, 6))
    /// Returns (moveDelta, limitFinder_span, render_offset) from normalization.
    pub fn solve(&mut self, svg: &str) -> Result<((f64, f64), (f64, f64), (f64, f64)), String> {
        use crate::svek::svg_result::SvgResult;

        // Java svek uses a pure YDelta(fullHeight) transform when parsing
        // Graphviz SVG output: x is left unchanged, y is flipped by adding
        // the full SVG canvas height.
        let full_height = parse_svg_full_height(svg);
        let svg_result = SvgResult::with_function(
            svg.to_string(),
            Box::new(crate::svek::snake::YDelta::new(full_height)),
        );
        for cluster in &mut self.bibliotekon.clusters {
            solve_cluster_positions(svg, cluster, full_height);
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
                        node.min_x = cx - rx;
                        node.min_y = (cy + full_height) - ry;
                        node.cx = cx;
                        node.cy = cy + full_height;
                    }
                }
            } else {
                // Extract polygon points from SVG and apply Java YDelta.
                let raw_points = svg_result.extract_points_at(idx);
                if !raw_points.is_empty() {
                    let points: Vec<XPoint2D> = raw_points.to_vec();
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

        adjust_cluster_frontiers(
            &mut self.bibliotekon.clusters,
            &self.bibliotekon.nodes,
            self.rankdir,
        );

        // Position edges — SvgResult already applies the Java YDelta transform.
        for edge in &mut self.bibliotekon.edges {
            edge.solve_line(&svg_result);
        }
        let nodes_snapshot = self.bibliotekon.nodes.clone();
        for edge in &mut self.bibliotekon.edges {
            edge.manage_collision(&nodes_snapshot);
        }

        // ── LimitFinder span (before moveDelta) ──
        // Java Pass 1: LimitFinder.drawRectangle(x, y, w, h) → addPoint(x-1, y-1), addPoint(x+w-1, y+h-1)
        // Java Pass 1: LimitFinder.drawEmpty for edge labels → addPoint(x, y), addPoint(x+w, y+h)
        // We simulate this on the unshifted svek coordinates to get the exact span.
        let active_nodes = self.bibliotekon.nodes.iter().filter(|n| !n.hidden).count();
        let is_degenerated = active_nodes <= 1 && self.bibliotekon.edges.is_empty();
        let mut lf_min_x = f64::INFINITY;
        let mut lf_min_y = f64::INFINITY;
        let mut lf_max_x = f64::NEG_INFINITY;
        let mut lf_max_y = f64::NEG_INFINITY;
        for node in &self.bibliotekon.nodes {
            if node.hidden {
                continue;
            }
            // Java LimitFinder corrections by entity image draw type:
            //
            //   drawRectangle (states, classes):
            //     addPoint(x-1, y-1), addPoint(x+w-1, y+h-1)
            //     → min_corr = 1, max_corr = 1
            //     BUT: all rect entities also draw ULine.hline(width), which adds
            //     addPoint(x+w, y+yLine) — overriding the rect's max_x by +1.
            //     Effective: max_corr_x = 0, max_corr_y = 1.
            //
            //   drawEllipse (circles):
            //     addPoint(x, y), addPoint(x+w-1, y+h-1)
            //     → min_corr = 0, max_corr = 1 (both axes)
            //
            //   drawUPath (notes):
            //     addPoint(x+minX, y+minY), addPoint(x+maxX, y+maxY)
            //     → min_corr = 0, max_corr = 0
            //
            // `lf_rect_correction` is true for rect entities, false for circles/notes.
            let extra_left = if is_degenerated { 0.0 } else { node.lf_extra_left };
            let min_corr = if node.lf_rect_correction { 1.0 } else { 0.0 };
            // For rect entities: ULine.hline(width) overrides rect's -1 on x-axis,
            // so max_corr_x = 0. For ellipse entities (Circle/Oval/Diamond): max_corr = 1.
            // For UPath entities (notes, !rect_corr, non-ellipse shape): max_corr = 0.
            let is_ellipse_shape = matches!(
                node.shape_type,
                shape_type::ShapeType::Circle
                    | shape_type::ShapeType::Oval
                    | shape_type::ShapeType::Diamond
            );
            // For rect entities with body separator (state/class): the ULine.hline(width)
            // overrides the drawRectangle -1 on max_x.
            // For rect entities without body separator (components): rect -1 stands.
            let max_corr_x = if node.lf_has_body_separator {
                0.0 // ULine.hline(width) overrides rect's -1 on x-axis
            } else if node.lf_rect_correction || is_ellipse_shape {
                1.0 // rect/ellipse: max -= 1
            } else {
                0.0 // UPath (notes): no correction
            };
            let max_corr_y = if node.lf_rect_correction || is_ellipse_shape {
                1.0 // rect/ellipse: max_y -= 1
            } else {
                0.0 // UPath (notes): no correction
            };
            let rx = node.min_x - min_corr - extra_left;
            let ry = node.min_y - min_corr;
            let rr = node.min_x + node.width - max_corr_x;
            let rb = node.min_y + node.height - max_corr_y;
            log::trace!(
                "  LF node uid={} min=({:.4},{:.4}) w={:.4} h={:.4} rect_corr={} body_sep={} => rx={:.4} ry={:.4} rr={:.4} rb={:.4}",
                node.uid, node.min_x, node.min_y, node.width, node.height,
                node.lf_rect_correction, node.lf_has_body_separator,
                rx, ry, rr, rb,
            );
            if rx < lf_min_x {
                lf_min_x = rx;
            }
            if ry < lf_min_y {
                lf_min_y = ry;
            }
            if rr > lf_max_x {
                lf_max_x = rr;
            }
            if rb > lf_max_y {
                lf_max_y = rb;
            }
        }
        for edge in &self.bibliotekon.edges {
            // LimitFinder: Java draws the label text block at
            // (labelXY.x + shield, labelXY.y + shield) where labelXY is the
            // top-left (getMinXY) of the Graphviz label polygon. The text
            // block's own drawU then invokes drawEmpty(0, 0, dim) on the
            // LimitFinder, so the LF sees (lx, ly) to (lx+w, ly+h).
            //
            // Additionally, Java's LimitFinder.drawText adjusts:
            //   y -= textHeight - 1.5
            // The label text block (TextBlockMarged) adds marginLabel before
            // drawing text. So the LF text contribution for min_y is:
            //   ly_text = label_y + shield + marginLabel - descent + 1.5
            // where descent is Java AWT SansSerif 13pt descent (3.0659).
            // This can be LOWER than the drawEmpty min_y when
            // marginLabel + 1.5 < descent (i.e. marginLabel=1).
            if let (Some(ref pt), Some(ref dim)) = (&edge.label_xy, &edge.label_dimension) {
                let dim_w = if edge.divide_label_width_by_two {
                    dim.width / 2.0
                } else {
                    dim.width
                };
                let lx = pt.x + edge.label_shield;
                let ly = pt.y + edge.label_shield;
                let lr = lx + dim_w;
                let lb = ly + dim.height;
                if lx < lf_min_x {
                    lf_min_x = lx;
                }
                if ly < lf_min_y {
                    lf_min_y = ly;
                }
                if lr > lf_max_x {
                    lf_max_x = lr;
                }
                if lb > lf_max_y {
                    lf_max_y = lb;
                }
                // Text descent correction: Java LimitFinder.drawText adjusts
                // y by -(textHeight - 1.5), so the effective min_y from text is:
                //   ly + marginLabel + 1.5 - descent
                // marginLabel = 6 for self-links, 1 otherwise.
                if edge.label.is_some() {
                    let margin_label = if edge.from_uid == edge.to_uid {
                        6.0
                    } else {
                        1.0
                    };
                    let descent =
                        crate::font_metrics::descent("SansSerif", 13.0, false, false);
                    let ly_text = ly + margin_label + 1.5 - descent;
                    if ly_text < lf_min_y {
                        lf_min_y = ly_text;
                    }
                }
            }
            // Java LimitFinder.drawDotPath: adds min/max of all bezier
            // control points. This matters when edge paths curve beyond the
            // bounding box of nodes and labels (e.g. curved arrows in
            // component diagrams).
            if let Some(ref dp) = edge.dot_path {
                if let Some((px_min, py_min, px_max, py_max)) = dp.min_max() {
                    if px_min < lf_min_x {
                        lf_min_x = px_min;
                    }
                    if py_min < lf_min_y {
                        lf_min_y = py_min;
                    }
                    if px_max > lf_max_x {
                        lf_max_x = px_max;
                    }
                    if py_max > lf_max_y {
                        lf_max_y = py_max;
                    }
                }
            }
        }
        for cluster in &self.bibliotekon.clusters {
            extend_lf_with_cluster(
                cluster,
                &mut lf_min_x,
                &mut lf_min_y,
                &mut lf_max_x,
                &mut lf_max_y,
            );
        }
        log::debug!(
            "svek solve LF: min=({:.4},{:.4}) max=({:.4},{:.4})",
            lf_min_x,
            lf_min_y,
            lf_max_x,
            lf_max_y
        );
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
        for cluster in &self.bibliotekon.clusters {
            extend_min_with_cluster(cluster, &mut min_x, &mut min_y);
        }
        let render_offset = if min_x.is_finite()
            && min_y.is_finite()
            && lf_min_x.is_finite()
            && lf_min_y.is_finite()
        {
            (6.0 + min_x - lf_min_x, 6.0 + min_y - lf_min_y)
        } else {
            (6.0, 6.0)
        };
        let (dx, dy) = if min_x.is_finite() && min_y.is_finite() {
            let dx = 6.0 - min_x;
            let dy = 6.0 - min_y;
            self.move_delta(dx, dy);
            (dx, dy)
        } else {
            (0.0, 0.0)
        };

        Ok(((dx, dy), lf_span, render_offset))
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
    let width =
        crate::font_metrics::text_width(title, "SansSerif", 14.0, true, false).floor() as i32;
    let height = crate::font_metrics::line_height("SansSerif", 14.0, true, false).floor() as i32;
    (width.max(0), height.max(0))
}

fn cluster_label_dim(cluster: &cluster::Cluster) -> Option<(i32, i32)> {
    if let Some((width, height)) = cluster.label_size {
        return Some((
            width.floor().max(0.0) as i32,
            height.floor().max(0.0) as i32,
        ));
    }
    cluster.title.as_deref().map(cluster_title_label_dim)
}

fn cluster_dot_label(cluster: &cluster::Cluster) -> String {
    if let Some((title_width, title_height)) = cluster_label_dim(cluster) {
        let mut label = String::from("<");
        append_table(
            &mut label,
            LabelDimension::new(title_width as f64, ((title_height - 5).max(1)) as f64),
            0x000000,
        );
        label.push('>');
        label
    } else {
        "\"\"".to_string()
    }
}

fn cluster_special_point_id(cluster: &cluster::Cluster) -> String {
    format!("za{}", cluster.id)
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

fn move_cluster_delta(cluster: &mut cluster::Cluster, dx: f64, dy: f64) {
    cluster.x += dx;
    cluster.y += dy;
    for sub in &mut cluster.sub_clusters {
        move_cluster_delta(sub, dx, dy);
    }
}

fn adjust_cluster_frontiers(
    clusters: &mut [cluster::Cluster],
    nodes: &[node::SvekNode],
    rankdir: Rankdir,
) {
    let node_rects: std::collections::HashMap<&str, (RectangleArea, node::EntityPosition)> = nodes
        .iter()
        .filter(|node| !node.hidden)
        .map(|node| {
            (
                node.uid.as_str(),
                (
                    RectangleArea::new(
                        node.min_x,
                        node.min_y,
                        node.min_x + node.width,
                        node.min_y + node.height,
                    ),
                    node.entity_position,
                ),
            )
        })
        .collect();
    for cluster in clusters {
        adjust_cluster_frontier(cluster, &node_rects, rankdir);
    }
}

fn adjust_cluster_frontier(
    cluster: &mut cluster::Cluster,
    node_rects: &std::collections::HashMap<&str, (RectangleArea, node::EntityPosition)>,
    rankdir: Rankdir,
) {
    for sub in &mut cluster.sub_clusters {
        adjust_cluster_frontier(sub, node_rects, rankdir);
    }

    let initial = RectangleArea::new(
        cluster.x,
        cluster.y,
        cluster.x + cluster.width,
        cluster.y + cluster.height,
    );
    if initial.width() <= 0.0 || initial.height() <= 0.0 {
        return;
    }

    let mut has_non_normal = false;
    let mut core: Option<RectangleArea> = None;
    let mut points = Vec::new();
    for uid in &cluster.node_uids {
        let Some((rect, position)) = node_rects.get(uid.as_str()) else {
            continue;
        };
        if position.is_normal() {
            core = Some(match core {
                Some(current) => current.merge(rect),
                None => *rect,
            });
        } else {
            has_non_normal = true;
            points.push(rect.center());
        }
    }
    if !has_non_normal {
        return;
    }

    for sub in &cluster.sub_clusters {
        if sub.width > 0.0 && sub.height > 0.0 {
            let rect = RectangleArea::new(sub.x, sub.y, sub.x + sub.width, sub.y + sub.height);
            core = Some(match core {
                Some(current) => current.merge(&rect),
                None => rect,
            });
        }
    }

    let mut core = core.unwrap_or_else(|| {
        let center = initial.center();
        RectangleArea::new(
            center.x - 1.0,
            center.y - 1.0,
            center.x + 1.0,
            center.y + 1.0,
        )
    });
    for point in &points {
        core = core.merge_point(*point);
    }

    let mut touch_min_x = false;
    let mut touch_max_x = false;
    let mut touch_min_y = false;
    let mut touch_max_y = false;
    for point in &points {
        if coord_eq(point.x, core.min_x) {
            touch_min_x = true;
        }
        if coord_eq(point.x, core.max_x) {
            touch_max_x = true;
        }
        if coord_eq(point.y, core.min_y) {
            touch_min_y = true;
        }
        if coord_eq(point.y, core.max_y) {
            touch_max_y = true;
        }
    }
    if !touch_min_x {
        core = core.with_min_x(initial.min_x);
    }
    if !touch_max_x {
        core = core.with_max_x(initial.max_x);
    }
    if !touch_min_y {
        core = core.with_min_y(initial.min_y);
    }
    if !touch_max_y {
        core = core.with_max_y(initial.max_y);
    }

    let delta = 3.0 * node::EntityPosition::RADIUS;
    let mut push_min_x = false;
    let mut push_max_x = false;
    let mut push_min_y = false;
    let mut push_max_y = false;
    for point in &points {
        if coord_eq(point.y, core.min_y) || coord_eq(point.y, core.max_y) {
            if (point.x - core.max_x).abs() < delta {
                push_max_x = true;
            }
            if (point.x - core.min_x).abs() < delta {
                push_min_x = true;
            }
        }
        if coord_eq(point.x, core.min_x) || coord_eq(point.x, core.max_x) {
            if (point.y - core.max_y).abs() < delta {
                push_max_y = true;
            }
            if (point.y - core.min_y).abs() < delta {
                push_min_y = true;
            }
        }
    }
    for point in &points {
        if rankdir == Rankdir::LeftToRight {
            if coord_eq(point.x, core.min_x)
                && (coord_eq(point.y, core.min_y) || coord_eq(point.y, core.max_y))
            {
                push_min_x = false;
            }
            if coord_eq(point.x, core.max_x)
                && (coord_eq(point.y, core.min_y) || coord_eq(point.y, core.max_y))
            {
                push_max_x = false;
            }
        } else {
            if coord_eq(point.y, core.min_y)
                && (coord_eq(point.x, core.min_x) || coord_eq(point.x, core.max_x))
            {
                push_min_y = false;
            }
            if coord_eq(point.y, core.max_y)
                && (coord_eq(point.x, core.min_x) || coord_eq(point.x, core.max_x))
            {
                push_max_y = false;
            }
        }
    }
    if push_max_x {
        core = core.add_max_x(delta);
    }
    if push_min_x {
        core = core.add_min_x(-delta);
    }
    if push_max_y {
        core = core.add_max_y(delta);
    }
    if push_min_y {
        core = core.add_min_y(-delta);
    }

    if let Some((title_width, title_height)) = cluster_label_dim(cluster) {
        if title_width > 0 && title_height > 0 {
            ensure_cluster_min_width(&mut core, initial, f64::from(title_width) + 10.0);
        }
    }

    cluster.x = core.min_x;
    cluster.y = core.min_y;
    cluster.width = core.width();
    cluster.height = core.height();
}

fn ensure_cluster_min_width(core: &mut RectangleArea, initial: RectangleArea, min_width: f64) {
    let delta = core.width() - min_width;
    if delta >= 0.0 {
        return;
    }
    let mut new_min_x = core.min_x + delta / 2.0;
    let mut new_max_x = core.max_x - delta / 2.0;
    let error = new_min_x - initial.min_x;
    if error < 0.0 {
        new_min_x -= error;
        new_max_x -= error;
    }
    *core = core.with_min_x(new_min_x).with_max_x(new_max_x);
}

fn coord_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// Parse the Graphviz SVG canvas height in pt.
fn parse_svg_full_height(svg: &str) -> f64 {
    let Some(pos) = svg.find(" height=\"") else {
        return 0.0;
    };
    let after = &svg[pos + 9..];
    let Some(end) = after.find("pt\"") else {
        return 0.0;
    };
    after[..end].trim().parse().unwrap_or(0.0)
}

fn solve_cluster_positions(svg: &str, cluster: &mut cluster::Cluster, full_height: f64) {
    if let Some((x, y, width, height)) = parse_svg_cluster_bounds(svg, &cluster.id, full_height) {
        cluster.x = x;
        cluster.y = y;
        cluster.width = width;
        cluster.height = height;
    }
    for sub in &mut cluster.sub_clusters {
        solve_cluster_positions(svg, sub, full_height);
    }
}

fn parse_svg_cluster_bounds(
    svg: &str,
    cluster_id: &str,
    full_height: f64,
) -> Option<(f64, f64, f64, f64)> {
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
        min_x = min_x.min(x);
        min_y = min_y.min(y + full_height);
        max_x = max_x.max(x);
        max_y = max_y.max(y + full_height);
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
        let (left, top, right, bottom) = match cluster.style {
            cluster::ClusterStyle::Rectangle | cluster::ClusterStyle::RoundedRectangle => (
                cluster.x - 1.0,
                cluster.y - 1.0,
                cluster.x + cluster.width - 1.0,
                cluster.y + cluster.height - 1.0,
            ),
            _ => (
                cluster.x,
                cluster.y,
                cluster.x + cluster.width,
                cluster.y + cluster.height,
            ),
        };
        if left < *min_x {
            *min_x = left;
        }
        if top < *min_y {
            *min_y = top;
        }
        if right > *max_x {
            *max_x = right;
        }
        if bottom > *max_y {
            *max_y = bottom;
        }
    }
    for sub in &cluster.sub_clusters {
        extend_lf_with_cluster(sub, min_x, min_y, max_x, max_y);
    }
}

fn extend_min_with_cluster(cluster: &cluster::Cluster, min_x: &mut f64, min_y: &mut f64) {
    if cluster.width > 0.0 && cluster.height > 0.0 {
        if cluster.x < *min_x {
            *min_x = cluster.x;
        }
        if cluster.y < *min_y {
            *min_y = cluster.y;
        }
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
            parse_svg_cluster_bounds(&svg, "demo", 0.0),
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
