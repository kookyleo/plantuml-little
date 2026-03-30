use crate::error::Error;
use crate::render::svg::fmt_coord;
use std::io::Write;
use std::process::{Command, Stdio};

/// Input: a graph node (abstract description independent of diagram type)
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub label: String,
    pub width_pt: f64,  // node width in pt (72pt = 1 inch), may be expanded for qualifiers
    pub height_pt: f64, // node height in pt
    /// DOT shape override (default: Rectangle → "rect").
    pub shape: Option<crate::svek::shape_type::ShapeType>,
    /// Java `SvekNode.shield()` margins for shielded HTML labels.
    pub shield: Option<crate::svek::Margins>,
    /// Java svek entity position for special boundary nodes such as ports.
    pub entity_position: Option<crate::svek::node::EntityPosition>,
    /// Java `EntityImagePort.getMaxWidthFromLabelForEntryExit()` equivalent.
    pub max_label_width: Option<f64>,
    /// Source/declaration order used to preserve Java DOT emission ordering.
    pub order: Option<usize>,
    /// Entity image natural width before qualifier expansion (px).
    /// Java's LimitFinder uses this for separator line bounds instead of
    /// the expanded DOT node width.
    pub image_width_pt: Option<f64>,
    /// Extra LimitFinder min_x extension from entity image content.
    /// Java's HACK_X_FOR_POLYGON=10 pushes polygon visibility modifiers
    /// 10px left of their actual min_x, extending the LF boundary beyond
    /// the node rect. Value is the extra negative x offset relative to
    /// the normal rect LF contribution (node_min_x - 1).
    /// E.g. for PACKAGE/PROTECTED polygons at node_x+7, HACK gives
    /// node_x-3, which is -2 beyond node_x-1 → lf_extra_left = 2.
    pub lf_extra_left: f64,
    /// Java LimitFinder.drawRectangle applies a -1 correction on both axes.
    /// Nodes whose entity images use UPath instead of URectangle (e.g. notes)
    /// don't get this correction. When false, the LF simulation skips the -1.
    pub lf_rect_correction: bool,
}

/// Input: a graph edge
#[derive(Debug, Clone)]
pub struct LayoutEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub tail_label: Option<String>,
    pub tail_label_boxed: bool,
    pub head_label: Option<String>,
    pub head_label_boxed: bool,
    pub tail_decoration: crate::svek::edge::LinkDecoration,
    pub head_decoration: crate::svek::edge::LinkDecoration,
    pub line_style: crate::svek::edge::LinkStyle,
    pub minlen: u32,
    pub invisible: bool,
    /// When true, set constraint=false in DOT (cross-axis direction hints).
    pub no_constraint: bool,
}

/// Input: a graph cluster (package / namespace / rectangle container).
#[derive(Debug, Clone)]
pub struct LayoutClusterSpec {
    pub id: String,
    pub qualified_name: String,
    pub title: Option<String>,
    pub style: crate::svek::cluster::ClusterStyle,
    pub label_width: Option<f64>,
    pub label_height: Option<f64>,
    pub node_ids: Vec<String>,
    pub sub_clusters: Vec<LayoutClusterSpec>,
    /// Source/declaration order used to preserve Java DOT emission ordering.
    pub order: Option<usize>,
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
    pub clusters: Vec<LayoutClusterSpec>,
    pub rankdir: RankDir,
    /// Temporary execution switch: class diagrams now follow Java's
    /// `LinkStrategy.SIMPLIER` for DOT arrows, while other diagram families
    /// still rely on legacy Graphviz arrow emission until their svek pipelines
    /// are ported far enough to render decorations purely from bezier geometry.
    pub use_simplier_dot_link_strategy: bool,
}

/// Output: node position after layout (SVG coordinates, origin top-left, Y downward)
#[derive(Debug, Clone)]
pub struct NodeLayout {
    pub id: String,
    pub cx: f64,     // center x (converted from Graphviz pt, Y-axis flipped)
    pub cy: f64,     // center y
    pub width: f64,  // width (from Graphviz, may be expanded)
    pub height: f64, // height (from Graphviz, may be expanded)
    /// Entity image natural width (DOT input minimum, in px).
    /// When Graphviz expands a node beyond the image dimensions (e.g. for
    /// qualifier shields), `image_width < width`.
    pub image_width: f64,
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
    /// Raw SVG path d-string from Graphviz (with transform applied),
    /// preserving original M/C/L commands for faithful reproduction.
    pub raw_path_d: Option<String>,
    /// Arrowhead polygon points from Graphviz SVG (with transform applied).
    pub arrow_polygon_points: Option<Vec<(f64, f64)>>,
    /// Edge label text (if any), carried from input for width expansion.
    pub label: Option<String>,
    /// Tail-side label text (quantifier or qualifier).
    pub tail_label: Option<String>,
    /// Tail-side label position from svek solve.
    pub tail_label_xy: Option<(f64, f64)>,
    /// Tail-side label block dimension.
    pub tail_label_wh: Option<(f64, f64)>,
    /// Whether the tail-side label is a boxed qualifier.
    pub tail_label_boxed: bool,
    /// Head-side label text (quantifier or qualifier).
    pub head_label: Option<String>,
    /// Head-side label position from svek solve.
    pub head_label_xy: Option<(f64, f64)>,
    /// Head-side label block dimension.
    pub head_label_wh: Option<(f64, f64)>,
    /// Whether the head-side label is a boxed qualifier.
    pub head_label_boxed: bool,
    /// Label center position from svek solve, used for LimitFinder-style tracking.
    pub label_xy: Option<(f64, f64)>,
    /// Label block dimension (width, height) from label_dimension + shield.
    pub label_wh: Option<(f64, f64)>,
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

/// Output: cluster/package bounds after Graphviz layout.
#[derive(Debug, Clone)]
pub struct ClusterLayout {
    pub id: String,
    pub qualified_name: String,
    pub title: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Output: layout result for the entire graph
#[derive(Debug, Clone)]
pub struct GraphLayout {
    pub nodes: Vec<NodeLayout>,
    pub edges: Vec<EdgeLayout>,
    pub clusters: Vec<ClusterLayout>,
    pub notes: Vec<ClassNoteLayout>,
    pub total_width: f64,
    pub total_height: f64,
    /// moveDelta applied by svek solve: (dx, dy). Used by renderer for coordinate alignment.
    pub move_delta: (f64, f64),
    /// LimitFinder span (width, height) computed before moveDelta.
    /// Java: `minMax.getDimension()` from `SvekResult.calculateDimension()`.
    /// Used for viewport calculation: `SVG_size = (int)(span + DELTA(15) + DOC_MARGIN(5) + 1)`.
    pub lf_span: (f64, f64),
    /// Normalization offset: the min (x, y) subtracted during origin normalization.
    /// Used to align label_xy (which is pre-moveDelta, pre-normalization) with
    /// path/node coordinates (which are post-moveDelta, post-normalization).
    pub normalize_offset: (f64, f64),
    /// Render offset needed after origin normalization to reconstruct Java's
    /// final post-Svek coordinates.
    ///
    /// Java moves by `6 - LimitFinder.min`, while the Rust normalization step
    /// subtracts the post-solve geometric min. The extra offset is therefore
    /// `6 + geometric_min - limitfinder_min`, and it can differ by axis.
    pub render_offset: (f64, f64),
}

/// AbstractEntityDiagram.java:61 — default nodesep = 0.35 inches.
const DEFAULT_NODESEP_IN: f64 = 0.35;
/// AbstractEntityDiagram.java:61 — default ranksep = 0.8 inches.
const DEFAULT_RANKSEP_IN: f64 = 0.8;
/// DotStringFactory.java:238-245 — getMinRankSep: class/state/component = 60px.
const MIN_RANK_SEP_PX: f64 = 60.0;
/// DotStringFactory.java:248-253 — getMinNodeSep: default = 35px.
const MIN_NODE_SEP_PX: f64 = 35.0;

/// SvekUtils.java:99-102 — pixelToInches: 72 DPI.
fn px_to_inches(px: f64) -> f64 {
    px / 72.0
}

/// Check if a label contains a link arrow direction indicator.
/// Java: `StringWithArrow` recognizes " >", " <", "> ", "< ", ">", "<".
pub(crate) fn has_link_arrow_indicator(label: &str) -> bool {
    let s = label.trim();
    s == ">" || s == "<"
        || s.ends_with(" >") || s.ends_with(" <")
        || s.starts_with("> ") || s.starts_with("< ")
}

/// Strip link arrow direction indicators from label text.
/// Java: `StringWithArrow` extracts " >", " <", "> ", "< " from the label
/// and renders them as arrow polygons. The label text for dimension
/// calculation does not include these indicators.
pub(crate) fn strip_link_arrow_text(label: &str) -> String {
    let s = label.trim();
    if s == ">" || s == "<" {
        return String::new();
    }
    if let Some(rest) = s.strip_suffix(" >") {
        return rest.trim().to_string();
    }
    if let Some(rest) = s.strip_suffix(" <") {
        return rest.trim().to_string();
    }
    if let Some(rest) = s.strip_prefix("> ") {
        return rest.trim().to_string();
    }
    if let Some(rest) = s.strip_prefix("< ") {
        return rest.trim().to_string();
    }
    s.to_string()
}

/// Return true if the label's arrow indicator is "backward" (pointing left / <).
/// Java: StringWithArrow sets LinkArrow.BACKWARD for "<" and " <" and "< ".
pub(crate) fn is_link_arrow_backward(label: &str) -> bool {
    let s = label.trim();
    s == "<" || s.ends_with(" <") || s.starts_with("< ")
}

fn measure_edge_text_block(text: &str, font_size: f64) -> (f64, f64) {
    let lines: Vec<&str> = text
        .split("\\n")
        .flat_map(|s| s.split("\\l"))
        .flat_map(|s| s.split("\\r"))
        .flat_map(|s| s.split(crate::NEWLINE_CHAR))
        .collect();
    let max_line_w = lines
        .iter()
        .map(|l| crate::font_metrics::text_width(l, "SansSerif", font_size, false, false))
        .fold(0.0_f64, f64::max);
    let line_h = crate::font_metrics::line_height("SansSerif", font_size, false, false);
    (max_line_w, lines.len() as f64 * line_h)
}

/// Serialize a LayoutGraph into a DOT format string
fn to_dot(graph: &LayoutGraph) -> String {
    // Java: clamp to max(default, minSep/72) — DotStringFactory.java
    let nodesep = DEFAULT_NODESEP_IN.max(px_to_inches(MIN_NODE_SEP_PX));
    let ranksep = DEFAULT_RANKSEP_IN.max(px_to_inches(MIN_RANK_SEP_PX));
    let mut dot = format!(
        "digraph G {{\n  rankdir={};\n  nodesep={nodesep:.4};\n  ranksep={ranksep:.4};\n  node [fixedsize=true, shape=rect];\n",
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
        let style = if edge.invisible { ", style=invis" } else { "" };
        match &edge.label {
            Some(lbl) => {
                let lbl = lbl.replace('"', "\\\"");
                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\", arrowtail=none, arrowhead=none, minlen={}{}];\n",
                    edge.from, edge.to, lbl, edge.minlen, style
                ));
            }
            None => {
                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\" [arrowtail=none, arrowhead=none, minlen={}{}];\n",
                    edge.from, edge.to, edge.minlen, style
                ));
            }
        }
    }
    for edge in &graph.edges {
        if edge.invisible && edge.minlen == 0 {
            dot.push_str(&format!(
                "  {{rank=same; \"{}\"; \"{}\";}}\n",
                edge.from, edge.to
            ));
        }
    }
    dot.push_str("}\n");
    log::trace!("DOT input:\n{}", dot);
    dot
}

/// Run Graphviz dot layout, returning node coordinates and edge paths.
///
/// Strategy: serialize the graph to DOT format, run layout via `dot -Tsvg`
/// subprocess, and parse the SVG output to obtain node coordinates and edge
/// paths with full pt precision (no inches→pt rounding).
pub fn layout(graph: &LayoutGraph) -> Result<GraphLayout, Error> {
    log::debug!(
        "layout: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    let dot_src = to_dot(graph);
    log::debug!("dot input:\n{dot_src}");

    // invoke dot -Tsvg, pipe DOT via stdin, read SVG from stdout
    let mut child = Command::new("dot")
        .arg("-Tsvg")
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

    let svg = String::from_utf8_lossy(&output.stdout);
    log::debug!("dot svg output:\n{svg}");

    parse_svg_output(&svg, graph)
}

/// Alternative layout function using the svek pipeline.
///
/// Same interface as `layout()` but uses `DotStringFactory` for DOT generation
/// and color-based SVG parsing. Converts svek results back to `GraphLayout`.
pub fn layout_with_svek(graph: &LayoutGraph) -> Result<GraphLayout, Error> {
    use crate::klimt::geom::Rankdir;
    use crate::svek::builder::{
        BuilderConfig, EntityDescriptor, GraphvizImageBuilder, LinkDescriptor,
    };
    use crate::svek::DotSplines;

    log::debug!(
        "layout_with_svek: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    let rankdir = match graph.rankdir {
        RankDir::TopToBottom => Rankdir::TopToBottom,
        RankDir::LeftToRight => Rankdir::LeftToRight,
        RankDir::BottomToTop => Rankdir::BottomToTop,
        RankDir::RightToLeft => Rankdir::RightToLeft,
    };

    let config = BuilderConfig {
        rankdir,
        dot_splines: DotSplines::Spline,
        nodesep: Some(MIN_NODE_SEP_PX),
        ranksep: Some(MIN_RANK_SEP_PX),
        use_simplier_dot_link_strategy: graph.use_simplier_dot_link_strategy,
        ..Default::default()
    };

    let mut builder = GraphvizImageBuilder::new(config);
    let mut node_cluster_ids = std::collections::HashMap::new();
    collect_node_cluster_assignments(&graph.clusters, &mut node_cluster_ids);
    let shielded_node_ids: std::collections::HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|node| node.shield.is_some())
        .map(|node| node.id.as_str())
        .collect();

    // Register entities
    for node in &graph.nodes {
        let mut ed = EntityDescriptor::new(&node.id, node.width_pt, node.height_pt);
        if let Some(shape) = node.shape {
            ed = ed.with_shape(shape);
        }
        if let Some(shield) = node.shield {
            ed = ed.with_shield(shield);
        }
        if let Some(entity_position) = node.entity_position {
            ed = ed.with_entity_position(entity_position);
        }
        if let Some(max_label_width) = node.max_label_width {
            ed = ed.with_max_label_width(max_label_width);
        }
        if let Some(order) = node.order {
            ed = ed.with_order(order);
        }
        if let Some(cluster_id) = node_cluster_ids.get(&node.id) {
            ed = ed.with_cluster(cluster_id);
        }
        if node.lf_extra_left > 0.0 {
            ed = ed.with_lf_extra_left(node.lf_extra_left);
        }
        if !node.lf_rect_correction {
            ed = ed.with_lf_rect_correction(false);
        }
        builder.add_entity(ed);
    }

    // Register links (including invisible edges for layout constraint)
    for edge in &graph.edges {
        let mut ld = LinkDescriptor::new(&edge.from, &edge.to);
        if let Some(ref label) = edge.label {
            ld = ld.with_label(label);
            // Compute label dimensions from font metrics for DOT sizing.
            // Java: labelText = StringWithArrow.addMagicArrow(label, guide, font)
            //   then addVisibilityModifier wraps with TextBlockMarged(marginLabel).
            // Edge labels use SansSerif 13pt (FontParam.CLASS = 13 for links).
            //
            // Java label dimension breakdown:
            // 1. Raw text block: text_width × text_height
            // 2. If link has direction arrow (" >", " <", etc.): mergeLR with
            //    TextBlockArrow2(size=fontSize=13) → adds 13px width
            // 3. TextBlockMarged(marginLabel=1 for normal, 6 for self): adds 2*margin
            // Result: (text_w + arrow_w + 2*margin) × (max(text_h, arrow_h) + 2*margin)
            let has_arrow = has_link_arrow_indicator(label);
            let label_text = strip_link_arrow_text(label);
            let (text_w, text_h) = measure_edge_text_block(&label_text, 13.0);
            let arrow_w = if has_arrow { 13.0 } else { 0.0 };
            let margin_label = if edge.from == edge.to { 6.0 } else { 1.0 };
            let inner_w = text_w + arrow_w;
            let inner_h = if has_arrow { text_h.max(13.0) } else { text_h };
            let dim_w = inner_w + 2.0 * margin_label;
            let dim_h = inner_h + 2.0 * margin_label;
            log::debug!("edge label={:?} text_w={:.4} arrow_w={} margin={} dim=({:.4},{:.4})",
                label, text_w, arrow_w, margin_label, dim_w, dim_h);
            ld.label_dimension = Some((dim_w, dim_h));
        }
        if let Some(ref tail_label) = edge.tail_label {
            let font_size = if edge.tail_label_boxed { 14.0 } else { 13.0 };
            let (text_w, text_h) = measure_edge_text_block(tail_label, font_size);
            ld.tail_label = Some(tail_label.clone());
            ld.tail_label_dimension = Some(if edge.tail_label_boxed {
                (text_w + 4.0, text_h + 2.0)
            } else {
                (text_w, text_h)
            });
        }
        if let Some(ref head_label) = edge.head_label {
            let font_size = if edge.head_label_boxed { 14.0 } else { 13.0 };
            let (text_w, text_h) = measure_edge_text_block(head_label, font_size);
            ld.head_label = Some(head_label.clone());
            ld.head_label_dimension = Some(if edge.head_label_boxed {
                (text_w + 4.0, text_h + 2.0)
            } else {
                (text_w, text_h)
            });
        }
        ld = ld.with_decorations(edge.head_decoration, edge.tail_decoration);
        ld = ld.with_style(edge.line_style);
        let from_port = shielded_node_ids.contains(edge.from.as_str()).then_some("h");
        let to_port = shielded_node_ids.contains(edge.to.as_str()).then_some("h");
        if from_port.is_some() || to_port.is_some() {
            ld = ld.with_ports(from_port, to_port);
        }
        if edge.invisible {
            ld.invisible = true;
        }
        if edge.no_constraint {
            ld.no_constraint = true;
        }
        ld.minlen = Some(edge.minlen);
        builder.add_link(ld);
    }

    for cluster in &graph.clusters {
        builder.add_cluster(layout_cluster_to_builder(cluster));
    }

    // Generate DOT
    let dot = builder.build_dot();
    log::debug!("svek dot input:\n{dot}");

    // Run Graphviz (same subprocess approach)
    let mut child = std::process::Command::new("dot")
        .arg("-Tsvg")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| Error::Layout(format!("failed to spawn dot: {e}")))?;

    child
        .stdin
        .take()
        .unwrap()
        .write_all(dot.as_bytes())
        .map_err(|e| Error::Layout(format!("failed to write to dot stdin: {e}")))?;

    let output = child
        .wait_with_output()
        .map_err(|e| Error::Layout(format!("dot process error: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Layout(format!("dot exited with error: {stderr}")));
    }

    let svg = String::from_utf8_lossy(&output.stdout);
    log::debug!("svek dot svg output:\n{svg}");

    // Parse edges directly from Graphviz SVG for arrowhead polygon data.
    // These coordinates use Graphviz's translate(tx,ty) transform.
    let (gv_tx, gv_ty) = parse_transform_translate(&svg);
    let parsed_svg_edges = parse_svg_edges_pre_normalize(&svg);

    // Solve: parse SVG and position nodes/edges via svek's YDelta(full_height)
    // transform, then apply moveDelta normalization.
    let (move_delta, lf_span, render_offset) = builder
        .solve(&svg)
        .map_err(|e| Error::Layout(format!("svek solve error: {e}")))?;

    // Graphviz SVG parsed edges use translate(tx,ty), while svek uses
    // YDelta(full_height) + moveDelta(dx,dy). These differ by a constant:
    //   correction_x = moveDelta_x - tx
    //   correction_y = full_height - ty + moveDelta_y
    // Apply this correction so parsed edge data aligns with svek node positions.
    let full_height = {
        let p = svg.find(" height=\"").map(|p| p + 9).unwrap_or(0);
        let e = svg[p..].find("pt\"").unwrap_or(0);
        svg[p..p + e].trim().parse::<f64>().unwrap_or(0.0)
    };
    let correction_x = move_delta.0 - gv_tx;
    let correction_y = full_height - gv_ty + move_delta.1;
    log::debug!(
        "parsed-edge correction: dx={correction_x:.2} dy={correction_y:.2} (tx={gv_tx}, ty={gv_ty}, fh={full_height}, md={:?})",
        move_delta
    );

    let mut parsed_svg_edges_by_key: std::collections::HashMap<
        (String, String),
        std::collections::VecDeque<EdgeLayout>,
    > = std::collections::HashMap::new();
    for mut edge in parsed_svg_edges {
        // Shift parsed edge coordinates from Graphviz-translate space to svek space.
        if correction_x.abs() > 1e-6 || correction_y.abs() > 1e-6 {
            for p in &mut edge.points {
                p.0 += correction_x;
                p.1 += correction_y;
            }
            if let Some(ref mut tip) = edge.arrow_tip {
                tip.0 += correction_x;
                tip.1 += correction_y;
            }
            if let Some(ref raw_d) = edge.raw_path_d {
                edge.raw_path_d = Some(transform_path_d(raw_d, correction_x, correction_y));
            }
            if let Some(ref mut pts) = edge.arrow_polygon_points {
                for p in pts.iter_mut() {
                    p.0 += correction_x;
                    p.1 += correction_y;
                }
            }
        }
        let key = (
            strip_entity_port(&edge.from).to_string(),
            strip_entity_port(&edge.to).to_string(),
        );
        parsed_svg_edges_by_key.entry(key).or_default().push_back(edge);
    }

    // Convert svek results to GraphLayout, normalizing to origin (0,0)
    // since the renderer adds its own MARGIN offset.
    let svek_nodes = builder.nodes();
    let svek_edges = builder.edges();
    let svek_clusters = builder.clusters();

    // Build initial node layouts
    let mut nodes_out: Vec<NodeLayout> = svek_nodes
        .iter()
        .enumerate()
        .map(|(i, sn)| {
            let id = if i < graph.nodes.len() {
                graph.nodes[i].id.clone()
            } else {
                sn.uid.clone()
            };
            let iw = if i < graph.nodes.len() {
                graph.nodes[i].image_width_pt.unwrap_or(graph.nodes[i].width_pt)
            } else {
                sn.width
            };
            NodeLayout {
                id,
                cx: sn.cx,
                cy: sn.cy,
                width: sn.width,
                height: sn.height,
                image_width: iw,
            }
        })
        .collect();

    // Build initial edge layouts
    let active_edges: Vec<&crate::layout::graphviz::LayoutEdge> =
        graph.edges.iter().filter(|e| !e.invisible).collect();
    let mut edges_out: Vec<EdgeLayout> = svek_edges
        .iter()
        .enumerate()
        .map(|(i, se)| {
            let (from, to) = if i < active_edges.len() {
                (active_edges[i].from.clone(), active_edges[i].to.clone())
            } else {
                (se.from_uid.clone(), se.to_uid.clone())
            };
            let mut points = Vec::new();
            let mut raw_path_d = None;
            if let Some(ref dp) = se.get_dot_path() {
                for bez in &dp.beziers {
                    if points.is_empty() {
                        points.push((bez.x1, bez.y1));
                    }
                    points.push((bez.ctrlx1, bez.ctrly1));
                    points.push((bez.ctrlx2, bez.ctrly2));
                    points.push((bez.x2, bez.y2));
                }
                raw_path_d = Some(dp.to_upath().to_svg_path_d());
            }
            let parsed_edge = parsed_svg_edges_by_key
                .get_mut(&(strip_entity_port(&from).to_string(), strip_entity_port(&to).to_string()))
                .and_then(|edges| edges.pop_front());
            EdgeLayout {
                from,
                to,
                points,
                arrow_tip: parsed_edge
                    .as_ref()
                    .and_then(|edge| edge.arrow_tip)
                    .or_else(|| se.end_contact_point().map(|p| (p.x, p.y))),
                raw_path_d: parsed_edge
                    .as_ref()
                    .and_then(|edge| edge.raw_path_d.clone())
                    .or(raw_path_d),
                arrow_polygon_points: parsed_edge
                    .as_ref()
                    .and_then(|edge| edge.arrow_polygon_points.clone()),
                label: se.label.clone(),
                tail_label: se.start_tail_text.clone(),
                tail_label_xy: se.start_tail_label_xy.map(|p| (p.x, p.y)),
                tail_label_wh: se.start_tail_dimension.map(|d| (d.width, d.height)),
                tail_label_boxed: active_edges.get(i).is_some_and(|edge| edge.tail_label_boxed),
                head_label: se.end_head_text.clone(),
                head_label_xy: se.end_head_label_xy.map(|p| (p.x, p.y)),
                head_label_wh: se.end_head_dimension.map(|d| (d.width, d.height)),
                head_label_boxed: active_edges.get(i).is_some_and(|edge| edge.head_label_boxed),
                label_xy: se.label_xy.map(|p| (p.x, p.y)),
                label_wh: se.label_dimension.map(|d| {
                    let dim_w = if se.divide_label_width_by_two {
                        d.width / 2.0
                    } else {
                        d.width
                    };
                    let dim_h = d.height;
                    // Add shield (same as DOT table sizing)
                    (dim_w + 2.0 * se.label_shield, dim_h + 2.0 * se.label_shield)
                }),
            }
        })
        .collect();

    let mut cluster_specs_by_id: std::collections::HashMap<&str, &LayoutClusterSpec> =
        std::collections::HashMap::new();
    collect_cluster_specs_by_id(&graph.clusters, &mut cluster_specs_by_id);
    let mut clusters_out = Vec::new();
    flatten_cluster_layouts(svek_clusters, &cluster_specs_by_id, &mut clusters_out);

    // Compute bounding box
    let min_x_nodes = nodes_out
        .iter()
        .map(|n| n.cx - n.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    let min_y_nodes = nodes_out
        .iter()
        .map(|n| n.cy - n.height / 2.0)
        .fold(f64::INFINITY, f64::min);
    let max_x_nodes = nodes_out
        .iter()
        .map(|n| n.cx + n.width / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y_nodes = nodes_out
        .iter()
        .map(|n| n.cy + n.height / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_x_clusters = clusters_out
        .iter()
        .map(|c| c.x)
        .fold(f64::INFINITY, f64::min);
    let min_y_clusters = clusters_out
        .iter()
        .map(|c| c.y)
        .fold(f64::INFINITY, f64::min);
    let max_x_clusters = clusters_out
        .iter()
        .map(|c| c.x + c.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y_clusters = clusters_out
        .iter()
        .map(|c| c.y + c.height)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_x = min_x_nodes.min(min_x_clusters);
    let min_y = min_y_nodes.min(min_y_clusters);
    let max_x = max_x_nodes.max(max_x_clusters);
    let max_y = max_y_nodes.max(max_y_clusters);
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;

    let normalize_offset = (min_x, min_y);
    log::debug!(
        "layout_with_svek normalize: min=({:.2},{:.2}) max=({:.2},{:.2})",
        min_x,
        min_y,
        max_x,
        max_y
    );
    for e in &edges_out {
        if let Some(ref lxy) = e.label_xy {
            log::debug!(
                "  edge label_xy before normalize: ({:.2},{:.2})",
                lxy.0,
                lxy.1
            );
        }
    }

    // Normalize to origin: shift so top-left entity corner is at (0, 0)
    for n in &mut nodes_out {
        n.cx -= min_x;
        n.cy -= min_y;
    }
    for e in &mut edges_out {
        for p in &mut e.points {
            p.0 -= min_x;
            p.1 -= min_y;
        }
        if let Some(ref mut tip) = e.arrow_tip {
            tip.0 -= min_x;
            tip.1 -= min_y;
        }
        if let Some(ref raw_d) = e.raw_path_d {
            e.raw_path_d = Some(transform_path_d(raw_d, -min_x, -min_y));
        }
        if let Some(ref mut pts) = e.arrow_polygon_points {
            for p in pts.iter_mut() {
                p.0 -= min_x;
                p.1 -= min_y;
            }
        }
        // Java keeps head/tail label positions in pre-normalized Svek space.
        // They are translated later by SvekEdge.drawU() via moveDelta only.
    }
    for c in &mut clusters_out {
        c.x -= min_x;
        c.y -= min_y;
    }

    Ok(GraphLayout {
        nodes: nodes_out,
        edges: edges_out,
        clusters: clusters_out,
        notes: Vec::new(),
        total_width,
        total_height,
        move_delta,
        lf_span,
        normalize_offset,
        render_offset,
    })
}

fn parse_svg_edges_pre_normalize(svg: &str) -> Vec<EdgeLayout> {
    let (tx, ty) = parse_transform_translate(svg);
    let mut result = Vec::new();
    let mut search_from = 0;
    while let Some(rel_pos) = svg[search_from..].find("<g id=") {
        let g_start = search_from + rel_pos;
        let tag_end = match svg[g_start..].find('>') {
            Some(pos) => g_start + pos + 1,
            None => break,
        };
        let open_tag = &svg[g_start..tag_end];
        if !open_tag.contains("class=\"edge\"") {
            search_from = tag_end;
            continue;
        }
        let g_end = match svg[tag_end..].find("</g>") {
            Some(pos) => tag_end + pos + 4,
            None => break,
        };
        let g_content = &svg[g_start..g_end];
        search_from = g_end;
        if let Some(edge) = parse_svg_edge(g_content, tx, ty) {
            result.push(edge);
        }
    }
    result
}

fn strip_entity_port(uid: &str) -> &str {
    uid.split(':').next().unwrap_or(uid)
}

fn collect_node_cluster_assignments(
    clusters: &[LayoutClusterSpec],
    out: &mut std::collections::HashMap<String, String>,
) {
    for cluster in clusters {
        for node_id in &cluster.node_ids {
            out.insert(node_id.clone(), cluster.id.clone());
        }
        collect_node_cluster_assignments(&cluster.sub_clusters, out);
    }
}

fn layout_cluster_to_builder(
    cluster: &LayoutClusterSpec,
) -> crate::svek::builder::ClusterDescriptor {
    let mut result = crate::svek::builder::ClusterDescriptor::new(&cluster.id).with_style(cluster.style);
    if let Some(ref title) = cluster.title {
        result = result.with_title(title);
    }
    if let (Some(label_width), Some(label_height)) = (cluster.label_width, cluster.label_height) {
        result = result.with_label_size(label_width, label_height);
    }
    if let Some(order) = cluster.order {
        result = result.with_order(order);
    }
    for node_id in &cluster.node_ids {
        result = result.add_entity(node_id);
    }
    for sub in &cluster.sub_clusters {
        result = result.add_sub_cluster(layout_cluster_to_builder(sub));
    }
    result
}

fn collect_cluster_specs_by_id<'a>(
    clusters: &'a [LayoutClusterSpec],
    out: &mut std::collections::HashMap<&'a str, &'a LayoutClusterSpec>,
) {
    for cluster in clusters {
        out.insert(cluster.id.as_str(), cluster);
        collect_cluster_specs_by_id(&cluster.sub_clusters, out);
    }
}

fn flatten_cluster_layouts(
    clusters: &[crate::svek::cluster::Cluster],
    specs_by_id: &std::collections::HashMap<&str, &LayoutClusterSpec>,
    out: &mut Vec<ClusterLayout>,
) {
    for cluster in clusters {
        let qualified_name = specs_by_id
            .get(cluster.id.as_str())
            .map(|spec| spec.qualified_name.clone())
            .unwrap_or_else(|| cluster.id.clone());
        let title = specs_by_id
            .get(cluster.id.as_str())
            .and_then(|spec| spec.title.clone())
            .or_else(|| cluster.title.clone());
        out.push(ClusterLayout {
            id: cluster.id.clone(),
            qualified_name,
            title,
            x: cluster.x,
            y: cluster.y,
            width: cluster.width,
            height: cluster.height,
        });
        flatten_cluster_layouts(&cluster.sub_clusters, specs_by_id, out);
    }
}

/// Parse `dot -Tsvg` output to extract node positions and edge paths.
///
/// Graphviz SVG coordinate system:
/// - The `<svg>` element has width/height in pt (e.g., `width="116pt"`)
/// - The top-level `<g>` has `transform="scale(s s) rotate(0) translate(tx ty)"`
/// - Internal element coordinates use Y=0 at bottom (PostScript convention),
///   with negative Y values. Applying the translate converts to SVG Y-down coords.
/// - Node positions come from `<ellipse cx= cy=>` or `<polygon points=>`
/// - Edge paths come from `<path d="M... C...">` and arrowhead `<polygon>`
fn parse_svg_output(svg: &str, graph: &LayoutGraph) -> Result<GraphLayout, Error> {
    // Extract translate(tx, ty) from the top-level <g> transform.
    // This converts Graphviz internal coords to SVG viewport coords.
    let (tx, ty) = parse_transform_translate(svg);
    log::debug!("svg transform translate: tx={tx}, ty={ty}");

    let mut node_map: std::collections::HashMap<String, NodeLayout> =
        std::collections::HashMap::new();
    let mut edge_layouts: Vec<EdgeLayout> = Vec::new();

    // Find node and edge <g> groups. Graphviz SVG uses:
    //   <g id="node1" class="node"> ... </g>
    //   <g id="edge1" class="edge"> ... </g>
    // These are leaf groups (no nested <g>), so the first </g> closes them.
    let mut search_from = 0;
    while let Some(rel_pos) = svg[search_from..].find("<g id=") {
        let g_start = search_from + rel_pos;
        // Extract the opening <g ...> tag to check class
        let tag_end = match svg[g_start..].find('>') {
            Some(pos) => g_start + pos + 1,
            None => break,
        };
        let open_tag = &svg[g_start..tag_end];

        // Only process node and edge groups, skip the outer graph group
        let is_node = open_tag.contains("class=\"node\"");
        let is_edge = open_tag.contains("class=\"edge\"");
        if !is_node && !is_edge {
            search_from = tag_end;
            continue;
        }

        // Find the closing </g> for this leaf group
        let g_end = match svg[tag_end..].find("</g>") {
            Some(pos) => tag_end + pos + 4,
            None => break,
        };
        let g_content = &svg[g_start..g_end];
        search_from = g_end;

        if is_node {
            if let Some(nl) = parse_svg_node(g_content, tx, ty, graph) {
                node_map.insert(nl.id.clone(), nl);
            }
        } else if is_edge {
            if let Some(el) = parse_svg_edge(g_content, tx, ty) {
                edge_layouts.push(el);
            }
        }
    }

    // Order output nodes according to LayoutGraph node order
    let nodes: Vec<NodeLayout> = graph
        .nodes
        .iter()
        .filter_map(|n| node_map.remove(&n.id))
        .collect();

    if nodes.len() != graph.nodes.len() {
        log::warn!(
            "layout: expected {} nodes, got {} from dot svg output",
            graph.nodes.len(),
            nodes.len()
        );
    }

    // Compute bounding box BEFORE normalization (Java computes minMax before moveDelta)
    let min_x = nodes
        .iter()
        .map(|n| n.cx - n.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    let min_y = nodes
        .iter()
        .map(|n| n.cy - n.height / 2.0)
        .fold(f64::INFINITY, f64::min);
    let max_x = nodes
        .iter()
        .map(|n| n.cx + n.width / 2.0)
        .fold(0.0_f64, f64::max);
    let max_y = nodes
        .iter()
        .map(|n| n.cy + n.height / 2.0)
        .fold(0.0_f64, f64::max);

    // Content span (used for canvas dimension calculation)
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;

    // Normalize coordinates: shift so top-left entity corner is at (0, 0).
    // The renderer adds its own MARGIN offset.
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
        if let Some(ref mut tip) = e.arrow_tip {
            tip.0 -= min_x;
            tip.1 -= min_y;
        }
        // Shift raw_path_d by re-transforming with the offset
        if let Some(ref raw_d) = e.raw_path_d {
            e.raw_path_d = Some(transform_path_d(raw_d, -min_x, -min_y));
        }
        if let Some(ref mut pts) = e.arrow_polygon_points {
            for p in pts.iter_mut() {
                p.0 -= min_x;
                p.1 -= min_y;
            }
        }
        // Java keeps head/tail label positions in pre-normalized Svek space.
        // They are translated later by SvekEdge.drawU() via moveDelta only.
    }

    Ok(GraphLayout {
        nodes,
        edges: edge_layouts,
        clusters: vec![],
        notes: vec![],
        total_width,
        total_height,
        move_delta: (0.0, 0.0),
        lf_span: (total_width, total_height),
        normalize_offset: (0.0, 0.0),
        render_offset: (0.0, 0.0),
    })
}

/// Extract `translate(tx, ty)` from the top-level `<g>` transform attribute.
fn parse_transform_translate(svg: &str) -> (f64, f64) {
    // Look for transform="... translate(tx ty) ..." or translate(tx,ty)
    if let Some(pos) = svg.find("translate(") {
        let after = &svg[pos + 10..];
        if let Some(end) = after.find(')') {
            let inner = &after[..end];
            // May be separated by space or comma
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

/// Parse a `<g class="node">` group to extract node ID and center position.
fn parse_svg_node(g: &str, tx: f64, ty: f64, graph: &LayoutGraph) -> Option<NodeLayout> {
    let id = parse_title(g)?;
    log::trace!("parse_svg_node: id={id}");

    // Java PlantUML algorithm (DotStringFactory.solve):
    // 1. Extract polygon points from Graphviz SVG
    // 2. Apply YDelta transform (flip Y axis)
    // 3. Take min(x), min(y) as node top-left corner (moveDelta)
    //
    // For rectangles: read polygon points, compute bounding box min corner
    // For circles/ovals: read cx,cy,rx,ry, compute (cx-rx, cy-ry)
    let (gviz_min_x, gviz_min_y) = if let Some(polygon_pos) = g.find("<polygon") {
        let polygon = &g[polygon_pos..];
        let points_str = parse_xml_attr_str(polygon, "points")?;
        let (min_x, min_y, _max_x, _max_y) = polygon_bounding_box(&points_str)?;
        (min_x, min_y)
    } else if let Some(ellipse_pos) = g.find("<ellipse") {
        let ellipse = &g[ellipse_pos..];
        let ecx = parse_xml_attr(ellipse, "cx")?;
        let ecy = parse_xml_attr(ellipse, "cy")?;
        let rx = parse_xml_attr(ellipse, "rx").unwrap_or(18.0);
        let ry = parse_xml_attr(ellipse, "ry").unwrap_or(18.0);
        (ecx - rx, ecy - ry)
    } else {
        log::warn!("node {id}: no polygon or ellipse found");
        return None;
    };

    // Apply translate transform: Graphviz SVG coords → viewport coords
    let min_x = tx + gviz_min_x;
    let min_y = ty + gviz_min_y;

    // Use original precise size from the input graph (not Graphviz's rounded values)
    let orig_size = graph.nodes.iter().find(|n| n.id == id);
    let (w, h) = match orig_size {
        Some(n) => (n.width_pt, n.height_pt),
        None => {
            log::warn!("node {id}: not found in input graph, using graphviz size");
            (36.0, 36.0)
        }
    };

    // Store as center point (NodeLayout uses cx/cy convention)
    Some(NodeLayout {
        id,
        cx: min_x + w / 2.0,
        cy: min_y + h / 2.0,
        width: w,
        height: h,
        image_width: w,
    })
}

/// Parse a `<g class="edge">` group to extract edge endpoints and path.
fn parse_svg_edge(g: &str, tx: f64, ty: f64) -> Option<EdgeLayout> {
    let title = parse_title(g)?;
    // Edge title format: "FROM&#45;&gt;TO" (XML-decoded: "FROM->TO")
    let (from, to) = parse_edge_title(&title)?;
    log::trace!("parse_svg_edge: {from} -> {to}");

    // Parse <path d="M... C..."/> for Bezier control points
    let mut points = Vec::new();
    let mut raw_path_d = None;
    if let Some(path_pos) = g.find("<path") {
        let path_elem = &g[path_pos..];
        if let Some(d_str) = parse_xml_attr_str(path_elem, "d") {
            points = parse_svg_path_d(d_str, tx, ty);
            raw_path_d = Some(transform_path_d(d_str, tx, ty));
        }
    }

    // Parse arrowhead <polygon> for arrow tip and full polygon points.
    // Skip non-arrowhead polygons:
    // - stroke="transparent": label background (Java SvekEdge label shield)
    // - fill="none": label TABLE cell/row borders from DOT HTML labels
    // Arrowhead polygons always have a solid fill color (fill="#RRGGBB").
    let path_end = g
        .find("<path")
        .and_then(|p| g[p..].find("/>").map(|e| p + e + 2));
    let polygon_search_start = path_end.unwrap_or(0);
    let (arrow_tip, arrow_polygon_points) = {
        let mut result = (None, None);
        let mut search = polygon_search_start;
        while let Some(poly_pos) = g[search..].find("<polygon") {
            let abs_pos = search + poly_pos;
            let polygon = &g[abs_pos..];
            let tag_end = polygon.find("/>").unwrap_or(usize::MAX);
            // Skip label background polygons with transparent stroke
            let is_label_bg = polygon
                .find("stroke=\"transparent\"")
                .map_or(false, |s| s < tag_end);
            // Skip label table border polygons with fill="none"
            let is_table_border = polygon
                .find("fill=\"none\"")
                .map_or(false, |s| s < tag_end);
            if !is_label_bg && !is_table_border {
                if let Some(pts_str) = parse_xml_attr_str(polygon, "points") {
                    let poly_pts = parse_polygon_points(pts_str, tx, ty);
                    let tip = if poly_pts.len() >= 2 {
                        Some(poly_pts[1])
                    } else {
                        poly_pts.first().copied()
                    };
                    result = (tip, Some(poly_pts));
                    break;
                }
            }
            // Move past this polygon
            search = abs_pos + 9; // past "<polygon"
        }
        result
    };

    Some(EdgeLayout {
        from,
        to,
        points,
        arrow_tip,
        raw_path_d,
        arrow_polygon_points,
        label: None,
        tail_label: None,
        tail_label_xy: None,
        tail_label_wh: None,
        tail_label_boxed: false,
        head_label: None,
        head_label_xy: None,
        head_label_wh: None,
        head_label_boxed: false,
        label_xy: None,
        label_wh: None,
    })
}

/// Extract text content from the first `<title>` element in a string.
/// Decodes basic XML entities.
fn parse_title(s: &str) -> Option<String> {
    let start_tag = "<title>";
    let end_tag = "</title>";
    let start = s.find(start_tag)? + start_tag.len();
    let end = s[start..].find(end_tag)? + start;
    let raw = &s[start..end];
    Some(decode_xml_entities(raw))
}

/// Parse edge title "FROM->TO" (after XML entity decoding).
fn parse_edge_title(title: &str) -> Option<(String, String)> {
    // The arrow separator is "->" in the decoded title
    let arrow_pos = title.find("->")?;
    let from = title[..arrow_pos].to_string();
    let to = title[arrow_pos + 2..].to_string();
    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some((from, to))
}

/// Parse a numeric XML attribute value, e.g., `cx="54"` -> 54.0
fn parse_xml_attr(elem: &str, attr_name: &str) -> Option<f64> {
    let pattern = format!("{}=\"", attr_name);
    let pos = elem.find(&pattern)?;
    let after = &elem[pos + pattern.len()..];
    let end = after.find('"')?;
    after[..end].parse::<f64>().ok()
}

/// Parse a string XML attribute value, e.g., `points="1,2 3,4"` -> "1,2 3,4"
fn parse_xml_attr_str<'a>(elem: &'a str, attr_name: &str) -> Option<&'a str> {
    let pattern = format!("{}=\"", attr_name);
    let pos = elem.find(&pattern)?;
    let after = &elem[pos + pattern.len()..];
    let end = after.find('"')?;
    Some(&after[..end])
}

/// Compute bounding box from a polygon `points` attribute string.
/// Points format: "x1,y1 x2,y2 x3,y3 ..."
fn polygon_bounding_box(points_str: &str) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut count = 0;

    for pair in points_str.split_whitespace() {
        let coords: Vec<&str> = pair.split(',').collect();
        if coords.len() == 2 {
            if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                count += 1;
            }
        }
    }

    if count > 0 {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// Parse polygon points attribute and apply transform.
/// Returns list of (x, y) points in SVG viewport coordinates.
fn parse_polygon_points(points_str: &str, tx: f64, ty: f64) -> Vec<(f64, f64)> {
    let mut result = Vec::new();
    for pair in points_str.split_whitespace() {
        let coords: Vec<&str> = pair.split(',').collect();
        if coords.len() == 2 {
            if let (Ok(x), Ok(y)) = (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                result.push((tx + x, ty + y));
            }
        }
    }
    result
}

/// Transform an SVG path `d` attribute string by applying translate(tx, ty),
/// preserving the original M/C/L command structure.
///
/// Returns a new d-string with all coordinates offset by (tx, ty),
/// formatted to match Java PlantUML coordinate style.
pub fn transform_path_d(d: &str, tx: f64, ty: f64) -> String {
    // Java renders DotPath via UPath→SvgGraphics.svgPath() which emits an explicit
    // command letter (M/C/L) for each segment.  Graphviz SVG may use implicit
    // continuation (one C followed by multiple triplets).  We re-emit with explicit
    // command letters per segment to match Java output.
    let mut result = String::new();
    let mut chars = d.chars().peekable();
    let mut current_cmd = ' ';
    let mut coord_pairs_in_segment = 0u32;

    // How many coordinate pairs per segment for each command
    fn pairs_per_segment(cmd: char) -> u32 {
        match cmd {
            'M' | 'L' => 1,
            'C' => 3,
            'Q' => 2,
            _ => 1,
        }
    }

    while let Some(&c) = chars.peek() {
        match c {
            'M' | 'C' | 'L' | 'Q' | 'Z' => {
                if c == 'Z' {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push('Z');
                    chars.next();
                    current_cmd = ' ';
                    coord_pairs_in_segment = 0;
                } else {
                    current_cmd = c;
                    coord_pairs_in_segment = 0;
                    chars.next();
                    // Don't emit command yet — it will be emitted when we see coordinates
                }
            }
            '-' | '0'..='9' | '.' => {
                let x = parse_next_number(&mut chars);
                skip_separators(&mut chars);
                let y = parse_next_number(&mut chars);
                if let (Some(x), Some(y)) = (x, y) {
                    // Check if we need to emit a new command letter
                    let pps = pairs_per_segment(current_cmd);
                    if coord_pairs_in_segment % pps == 0 {
                        // Start of a new segment — emit command letter
                        if !result.is_empty() && !result.ends_with(' ') {
                            result.push(' ');
                        }
                        result.push(current_cmd);
                    } else {
                        result.push(' ');
                    }
                    coord_pairs_in_segment += 1;

                    let nx = tx + x;
                    let ny = ty + y;
                    result.push_str(&fmt_coord(nx));
                    result.push(',');
                    result.push_str(&fmt_coord(ny));
                }
                // Consume trailing separators
                while let Some(&next) = chars.peek() {
                    if next == ' ' || next == ',' || next == '\t' || next == '\n' || next == '\r' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            ' ' | ',' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }
    result
}

/// Parse SVG path `d` attribute (M/C/L commands) and apply transform.
///
/// Graphviz edge paths typically use:
/// - `M x,y` — move to start
/// - `C x1,y1 x2,y2 x3,y3` — cubic Bezier (may have multiple triplets)
/// - `L x,y` — line to (less common)
///
/// Returns all control points in SVG viewport coordinates.
fn parse_svg_path_d(d: &str, tx: f64, ty: f64) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    // Tokenize: split by command letters, keeping numbers together
    // Strategy: iterate character by character, collecting coordinate pairs
    let mut chars = d.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            'M' | 'C' | 'L' => {
                chars.next(); // consume command letter
            }
            '-' | '0'..='9' | '.' => {
                // Parse a coordinate pair: x,y or x y (separated by comma or space)
                let x = parse_next_number(&mut chars);
                skip_separators(&mut chars);
                let y = parse_next_number(&mut chars);
                if let (Some(x), Some(y)) = (x, y) {
                    points.push((tx + x, ty + y));
                }
            }
            _ => {
                chars.next(); // skip whitespace and other chars
            }
        }
    }
    points
}

/// Parse the next floating-point number from the character iterator.
fn parse_next_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<f64> {
    let mut s = String::new();
    // Optional leading minus sign
    if let Some(&'-') = chars.peek() {
        s.push('-');
        chars.next();
    }
    // Digits and decimal point
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if s.is_empty() || s == "-" {
        None
    } else {
        s.parse::<f64>().ok()
    }
}

/// Skip commas and whitespace between coordinates.
fn skip_separators(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c == ',' || c == ' ' || c == '\t' || c == '\n' || c == '\r' {
            chars.next();
        } else {
            break;
        }
    }
}

/// Decode basic XML entities used in Graphviz SVG output.
fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#45;", "-")
        .replace("&#39;", "'")
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
                    shape: None,
                    shield: None,
                    entity_position: None,
                    max_label_width: None,
                    order: None,
                    image_width_pt: None,
                    lf_extra_left: 0.0,
                    lf_rect_correction: true,
                },
                LayoutNode {
                    id: "B".into(),
                    label: "ClassB".into(),
                    width_pt: 108.0,
                    height_pt: 36.0,
                    shape: None,
                    shield: None,
                    entity_position: None,
                    max_label_width: None,
                    order: None,
                    image_width_pt: None,
                    lf_extra_left: 0.0,
                    lf_rect_correction: true,
                },
            ],
            edges: vec![LayoutEdge {
                from: "A".into(),
                to: "B".into(),
                label: None,
                tail_label: None,
                tail_label_boxed: false,
                head_label: None,
                head_label_boxed: false,
                tail_decoration: crate::svek::edge::LinkDecoration::None,
                head_decoration: crate::svek::edge::LinkDecoration::None,
                line_style: crate::svek::edge::LinkStyle::Normal,
                minlen: 1,
                invisible: false,
                no_constraint: false,
            }],
            clusters: vec![],
            rankdir: RankDir::TopToBottom,
            use_simplier_dot_link_strategy: false,
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
                shape: None,
                shield: None,
                entity_position: None,
                max_label_width: None,
                order: None,
                image_width_pt: None,
                lf_extra_left: 0.0,
                    lf_rect_correction: true,
            }],
            edges: vec![],
            clusters: vec![],
            rankdir: RankDir::LeftToRight,
            use_simplier_dot_link_strategy: false,
        };
        let result = layout(&graph).expect("single node layout failed");
        assert_eq!(result.nodes.len(), 1);
        assert!(result.nodes[0].cx >= 0.0);
    }
}
