use crate::error::Error;
use crate::render::svg::fmt_coord;
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
    pub minlen: u32,
    pub invisible: bool,
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
    /// Raw SVG path d-string from Graphviz (with transform applied),
    /// preserving original M/C/L commands for faithful reproduction.
    pub raw_path_d: Option<String>,
    /// Arrowhead polygon points from Graphviz SVG (with transform applied).
    pub arrow_polygon_points: Option<Vec<(f64, f64)>>,
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
    use crate::svek::builder::{BuilderConfig, EntityDescriptor, GraphvizImageBuilder, LinkDescriptor};
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
        ..Default::default()
    };

    let mut builder = GraphvizImageBuilder::new(config);

    // Register entities
    for node in &graph.nodes {
        builder.add_entity(EntityDescriptor::new(&node.id, node.width_pt, node.height_pt));
    }

    // Register links (including invisible edges for layout constraint)
    for edge in &graph.edges {
        let mut ld = LinkDescriptor::new(&edge.from, &edge.to);
        if let Some(ref label) = edge.label {
            ld = ld.with_label(label);
        }
        if edge.invisible {
            ld.invisible = true;
        }
        ld.minlen = Some(edge.minlen);
        builder.add_link(ld);
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

    // Solve: parse SVG and position nodes/edges
    builder
        .solve(&svg)
        .map_err(|e| Error::Layout(format!("svek solve error: {e}")))?;

    // Convert svek results to GraphLayout, normalizing to origin (0,0)
    // since the renderer adds its own MARGIN offset.
    let svek_nodes = builder.nodes();
    let svek_edges = builder.edges();

    // Build initial node layouts
    let mut nodes_out: Vec<NodeLayout> = svek_nodes.iter().enumerate().map(|(i, sn)| {
        let id = if i < graph.nodes.len() {
            graph.nodes[i].id.clone()
        } else {
            sn.uid.clone()
        };
        NodeLayout { id, cx: sn.cx, cy: sn.cy, width: sn.width, height: sn.height }
    }).collect();

    // Build initial edge layouts
    let active_edges: Vec<&crate::layout::graphviz::LayoutEdge> = graph.edges
        .iter()
        .filter(|e| !e.invisible)
        .collect();
    let mut edges_out: Vec<EdgeLayout> = svek_edges.iter().enumerate().map(|(i, se)| {
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
        EdgeLayout {
            from, to, points,
            arrow_tip: se.end_contact_point().map(|p| (p.x, p.y)),
            raw_path_d,
            arrow_polygon_points: None,
        }
    }).collect();

    // Compute bounding box
    let min_x = nodes_out.iter().map(|n| n.cx - n.width / 2.0).fold(f64::INFINITY, f64::min);
    let min_y = nodes_out.iter().map(|n| n.cy - n.height / 2.0).fold(f64::INFINITY, f64::min);
    let max_x = nodes_out.iter().map(|n| n.cx + n.width / 2.0).fold(0.0_f64, f64::max);
    let max_y = nodes_out.iter().map(|n| n.cy + n.height / 2.0).fold(0.0_f64, f64::max);
    let total_width = max_x - min_x;
    let total_height = max_y - min_y;

    // debug removed

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
    }

    Ok(GraphLayout {
        nodes: nodes_out,
        edges: edges_out,
        notes: Vec::new(),
        total_width,
        total_height,
    })
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
    }

    Ok(GraphLayout {
        nodes,
        edges: edge_layouts,
        notes: vec![],
        total_width,
        total_height,
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
    let path_end = g
        .find("<path")
        .and_then(|p| g[p..].find("/>").map(|e| p + e + 2));
    let polygon_search_start = path_end.unwrap_or(0);
    let (arrow_tip, arrow_polygon_points) =
        if let Some(poly_pos) = g[polygon_search_start..].find("<polygon") {
            let polygon = &g[polygon_search_start + poly_pos..];
            if let Some(pts_str) = parse_xml_attr_str(polygon, "points") {
                let poly_pts = parse_polygon_points(pts_str, tx, ty);
                let tip = if poly_pts.len() >= 2 {
                    Some(poly_pts[1])
                } else {
                    poly_pts.first().copied()
                };
                (tip, Some(poly_pts))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

    Some(EdgeLayout {
        from,
        to,
        points,
        arrow_tip,
        raw_path_d,
        arrow_polygon_points,
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
    let mut result = String::new();
    let mut chars = d.chars().peekable();
    let mut had_coords = false; // track if we just emitted coordinates

    while let Some(&c) = chars.peek() {
        match c {
            'M' | 'C' | 'L' | 'Z' => {
                // Add space before command if preceded by coordinates
                if had_coords && !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push(c);
                chars.next();
                had_coords = false;
            }
            '-' | '0'..='9' | '.' => {
                let x = parse_next_number(&mut chars);
                skip_separators(&mut chars);
                let y = parse_next_number(&mut chars);
                if let (Some(x), Some(y)) = (x, y) {
                    let nx = tx + x;
                    let ny = ty + y;
                    result.push_str(&fmt_coord(nx));
                    result.push(',');
                    result.push_str(&fmt_coord(ny));
                    had_coords = true;
                }
                // Consume trailing separators and add space if more data follows
                if let Some(&next) = chars.peek() {
                    if next == ' ' || next == ',' {
                        skip_separators(&mut chars);
                        // Add space only if more data follows (not end, not another separator)
                        if let Some(&next2) = chars.peek() {
                            if next2 != ' ' && next2 != ',' {
                                result.push(' ');
                            }
                        }
                    }
                }
            }
            ' ' | ',' | '\t' | '\n' | '\r' => {
                chars.next();
                // Skip, spaces are managed above
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
                minlen: 1,
                invisible: false,
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
