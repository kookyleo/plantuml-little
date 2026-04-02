use crate::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::erd::{
    ErdAttrEdge, ErdAttrLayout, ErdEdgeLayout, ErdIsaLayout, ErdLayout, ErdNodeLayout,
    ErdNoteLayout, ErdIsaChildEdge,
};
use crate::model::erd::ErdDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

const FONT_SIZE: f64 = 14.0;
use crate::skin::rose::{BORDER_COLOR, ENTITY_BG, NOTE_BG, NOTE_BORDER, NOTE_FOLD, TEXT_COLOR};

fn render_path_line(sg: &mut SvgGraphic, x1: f64, y1: f64, x2: f64, y2: f64) {
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} " fill="none" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
        fmt_coord(x1),
        fmt_coord(y1),
        fmt_coord(x2),
        fmt_coord(y2)
    ));
}

pub fn render_erd(_ed: &ErdDiagram, layout: &ErdLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = ensure_visible_int(layout.width) as f64;
    let svg_h = ensure_visible_int(layout.height) as f64;
    write_svg_root_bg(&mut buf, svg_w, svg_h, "CHEN_EER", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);

    let ent_bg = skin.background_color("entity", ENTITY_BG);
    let ent_border = skin.border_color("entity", BORDER_COLOR);
    let ent_font = skin.font_color("entity", TEXT_COLOR);

    let mut sg = SvgGraphic::new(0, 1.0);

    // Build parent→attributes index for interleaved rendering
    let mut attrs_by_parent: std::collections::HashMap<&str, Vec<&ErdAttrLayout>> =
        std::collections::HashMap::new();
    for attr in &layout.attribute_nodes {
        attrs_by_parent
            .entry(attr.parent.as_str())
            .or_default()
            .push(attr);
    }


    // Merge entities, relationships, and ISAs into a single list sorted by
    // source_order. Java renders them in declaration order, interleaved.
    enum RenderItem<'a> {
        Node(&'a ErdNodeLayout),
        Isa(&'a ErdIsaLayout),
    }
    let mut items: Vec<(usize, RenderItem)> = Vec::new();
    for node in layout.entity_nodes.iter().chain(layout.relationship_nodes.iter()) {
        items.push((node.source_order, RenderItem::Node(node)));
    }
    for isa in &layout.isa_layouts {
        items.push((isa.source_order, RenderItem::Isa(isa)));
    }
    items.sort_by_key(|(order, _)| *order);

    for (_, item) in &items {
        match item {
            RenderItem::Node(node) => {
                if node.is_relationship {
                    render_relationship(&mut sg, node);
                } else {
                    render_entity(&mut sg, node, ent_bg, ent_border, ent_font);
                }
                if let Some(attrs) = attrs_by_parent.get(node.id.as_str()) {
                    for attr in attrs {
                        render_attribute(&mut sg, attr);
                    }
                }
            }
            RenderItem::Isa(isa) => {
                render_isa_circle(&mut sg, isa);
            }
        }
    }
    // Merge attr_edges and link_edges into a single list, sorted by source_order.
    // Within the same source_order, attr edges come first (they belong to the
    // entity/relationship at that order), then link edges.
    enum EdgeItem<'a> {
        Attr(usize, &'a ErdAttrEdge),
        Link(usize, &'a ErdEdgeLayout),
        IsaEdges(&'a ErdIsaLayout),
    }
    let mut edge_items: Vec<(usize, u8, EdgeItem)> = Vec::new();
    for (i, ae) in layout.attr_edges.iter().enumerate() {
        edge_items.push((ae.parent_source_order, 0, EdgeItem::Attr(i, ae)));
    }
    for (i, le) in layout.edges.iter().enumerate() {
        edge_items.push((le.source_order, 1, EdgeItem::Link(i, le)));
    }
    for isa in &layout.isa_layouts {
        edge_items.push((isa.source_order, 2, EdgeItem::IsaEdges(isa)));
    }
    // Sort by (source_order, type_priority): attrs first, then links, then ISA edges
    edge_items.sort_by_key(|(order, prio, _)| (*order, *prio));

    for (_, _, item) in &edge_items {
        match item {
            EdgeItem::Attr(_, ae) => {
                render_attr_edge(&mut sg, ae, &layout.svek_node_uids);
            }
            EdgeItem::Link(_, le) => {
                let link_uid = layout.link_uids
                    .get(&le.source_order)
                    .copied()
                    .unwrap_or(0);
                render_edge(&mut sg, le, link_uid, &layout.svek_node_uids);
            }
            EdgeItem::IsaEdges(isa) => {
                render_isa_edges(&mut sg, isa);
            }
        }
    }
    for note in &layout.notes {
        render_note(&mut sg, note);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_entity(
    sg: &mut SvgGraphic,
    node: &ErdNodeLayout,
    bg: &str,
    border: &str,
    font_color: &str,
) {
    let (x, y, w, h) = (node.x, node.y, node.width, node.height);
    // Use per-entity colors if specified, otherwise defaults
    let eff_bg = node.bg_color.as_deref().unwrap_or(bg);
    let eff_border = node.line_color.as_deref().unwrap_or(border);
    sg.push_raw("<g>");
    if node.is_weak {
        sg.set_fill_color(eff_bg);
        sg.set_stroke_color(Some(eff_border));
        sg.set_stroke_width(0.5, None);
        sg.svg_rectangle(x, y, w, h, 0.0, 0.0, 0.0);
        let inset = 3.0;
        sg.set_fill_color(eff_bg);
        sg.set_stroke_color(Some(eff_border));
        sg.set_stroke_width(0.5, None);
        sg.svg_rectangle(
            x + inset,
            y + inset,
            w - 2.0 * inset,
            h - 2.0 * inset,
            0.0,
            0.0,
            0.0,
        );
    } else {
        sg.set_fill_color(eff_bg);
        sg.set_stroke_color(Some(eff_border));
        sg.set_stroke_width(0.5, None);
        sg.svg_rectangle(x, y, w, h, 0.0, 0.0, 0.0);
    }
    let tx = x + 10.0;
    let asc = crate::font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let desc = crate::font_metrics::descent("SansSerif", FONT_SIZE, false, false);
    let ty = y + h / 2.0 + (asc - desc) / 2.0;
    sg.set_fill_color(font_color);
    sg.svg_text(
        &node.label,
        tx,
        ty,
        Some("sans-serif"),
        FONT_SIZE,
        None,
        None,
        None,
        w - 20.0,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    sg.push_raw("</g>");
}

fn render_relationship(sg: &mut SvgGraphic, node: &ErdNodeLayout) {
    let (x, y, w, h) = (node.x, node.y, node.width, node.height);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let eff_bg = node.bg_color.as_deref().unwrap_or(ENTITY_BG);
    let eff_border = node.line_color.as_deref().unwrap_or(BORDER_COLOR);
    sg.push_raw("<g>");
    if node.is_identifying {
        sg.set_fill_color(eff_bg);
        sg.set_stroke_color(Some(eff_border));
        sg.set_stroke_width(0.5, None);
        sg.svg_polygon(0.0, &[x, cy, cx, y, x + w, cy, cx, y + h]);
        let inset_x = 10.0;
        let inset_y = 5.0;
        sg.set_fill_color(eff_bg);
        sg.set_stroke_color(Some(eff_border));
        sg.set_stroke_width(0.5, None);
        sg.svg_polygon(
            0.0,
            &[
                x + inset_x,
                cy,
                cx,
                y + inset_y,
                x + w - inset_x,
                cy,
                cx,
                y + h - inset_y,
            ],
        );
    } else {
        sg.set_fill_color(eff_bg);
        sg.set_stroke_color(Some(eff_border));
        sg.set_stroke_width(0.5, None);
        sg.svg_polygon(0.0, &[x, cy, cx, y, x + w, cy, cx, y + h]);
    }
    let asc = crate::font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let desc = crate::font_metrics::descent("SansSerif", FONT_SIZE, false, false);
    let ty = cy + (asc - desc) / 2.0;
    let text_w = crate::font_metrics::text_width(&node.label, "SansSerif", FONT_SIZE, false, false);
    let text_x = cx - text_w / 2.0;
    sg.set_fill_color(TEXT_COLOR);
    sg.svg_text(
        &node.label,
        text_x,
        ty,
        Some("sans-serif"),
        FONT_SIZE,
        None,
        None,
        None,
        text_w,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    sg.push_raw("</g>");
}

fn render_attribute(sg: &mut SvgGraphic, attr: &ErdAttrLayout) {
    sg.push_raw("<g>");
    let (cx, cy, rx, ry) = (attr.x, attr.y, attr.rx, attr.ry);
    if attr.is_derived {
        sg.set_fill_color(ENTITY_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(0.5, Some((10.0, 10.0)));
        sg.svg_ellipse(cx, cy, rx, ry, 0.0);
    } else if attr.is_multi {
        sg.set_fill_color(ENTITY_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_ellipse(cx, cy, rx, ry, 0.0);
        sg.set_fill_color(ENTITY_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_ellipse(cx, cy, rx - 3.0, ry - 3.0, 0.0);
    } else {
        sg.set_fill_color(ENTITY_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_ellipse(cx, cy, rx, ry, 0.0);
    }
    // Java text y: entity_top_y + MARGIN(6) + ascent (TextBlockInEllipse layout)
    let asc = crate::font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let entity_top_y = cy - ry;
    let ty = entity_top_y + 6.0 + asc;
    let text_w = crate::font_metrics::text_width(&attr.label, "SansSerif", FONT_SIZE, false, false);
    let text_x = cx - text_w / 2.0;
    if attr.is_key {
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &attr.label,
            text_x,
            ty,
            Some("sans-serif"),
            FONT_SIZE,
            None,
            None,
            Some("underline"),
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    } else {
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &attr.label,
            text_x,
            ty,
            Some("sans-serif"),
            FONT_SIZE,
            None,
            None,
            None,
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
    // Note: type label is already included in attr.label (e.g. "Born : DATE"),
    // so no separate text element is needed. Java does the same.
    sg.push_raw("</g>");
    for child in &attr.children {
        render_attribute(sg, child);
        // Child→parent-attr edge paths are rendered via attr_edges from graphviz
    }
}

fn render_attr_edge(
    sg: &mut SvgGraphic,
    attr_edge: &ErdAttrEdge,
    uid_map: &std::collections::HashMap<String, usize>,
) {
    if let Some(ref path_d) = attr_edge.raw_path_d {
        sg.push_raw(&format!(
            "<!--link {} to {}-->",
            xml_escape(&attr_edge.from_name),
            xml_escape(&attr_edge.to_name)
        ));
        let from_uid = uid_map.get(&attr_edge.from_name).copied().unwrap_or(0);
        let to_uid = uid_map.get(&attr_edge.to_name).copied().unwrap_or(0);
        // Edge uid = from_uid + 1 (edge follows its from-node in the uid sequence)
        let link_uid = from_uid + 1;
        let ent_from = format!("ent{:04}", from_uid);
        let ent_to = format!("ent{:04}", to_uid);
        sg.push_raw(&format!(
            r#"<g class="link" data-entity-1="{ent_from}" data-entity-2="{ent_to}" data-link-type="association" data-source-line="{link_uid}" id="lnk{link_uid}">"#,
        ));
        sg.push_raw(&format!(
            r#"<path d="{}" fill="none" id="{}-{}" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
            path_d,
            xml_escape(&attr_edge.from_name), xml_escape(&attr_edge.to_name),
        ));
        sg.push_raw("</g>");
    }
}

fn render_edge(
    sg: &mut SvgGraphic,
    edge: &ErdEdgeLayout,
    link_uid: usize,
    uid_map: &std::collections::HashMap<String, usize>,
) {
    let (x1, y1) = edge.from_point;
    let (x2, y2) = edge.to_point;
    // Java wraps each link in <!--link From to To--> comment and <g class="link" ...>
    sg.push_raw(&format!(
        "<!--link {} to {}-->",
        xml_escape(&edge.from_name),
        xml_escape(&edge.to_name)
    ));
    let from_uid = uid_map.get(&edge.from_name).copied().unwrap_or(0);
    let to_uid = uid_map.get(&edge.to_name).copied().unwrap_or(0);
    let ent_from = format!("ent{:04}", from_uid);
    let ent_to = format!("ent{:04}", to_uid);
    sg.push_raw(&format!(
        r#"<g class="link" data-entity-1="{}" data-entity-2="{}" data-link-type="association" data-source-line="{}" id="lnk{}">"#,
        ent_from, ent_to, link_uid, link_uid
    ));
    if edge.is_double {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.001 {
            let nx = -dy / len * 1.5;
            let ny = dx / len * 1.5;
            render_path_line(sg, x1 + nx, y1 + ny, x2 + nx, y2 + ny);
            render_path_line(sg, x1 - nx, y1 - ny, x2 - nx, y2 - ny);
        }
    } else if let Some(ref path_d) = edge.raw_path_d {
        sg.push_raw(&format!(
            r#"<path d="{}" fill="none" id="{}-{}" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
            path_d,
            xml_escape(&edge.from_name), xml_escape(&edge.to_name),
        ));
    } else {
        let cx1 = x1 + (x2 - x1) / 3.0;
        let cy1 = y1 + (y2 - y1) / 3.0;
        let cx2 = x1 + 2.0 * (x2 - x1) / 3.0;
        let cy2 = y1 + 2.0 * (y2 - y1) / 3.0;
        sg.push_raw(&format!(
            r#"<path d="M{},{} C{},{} {},{} {},{}" fill="none" id="{}-{}" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
            fmt_coord(x1), fmt_coord(y1),
            fmt_coord(cx1), fmt_coord(cy1),
            fmt_coord(cx2), fmt_coord(cy2),
            fmt_coord(x2), fmt_coord(y2),
            xml_escape(&edge.from_name), xml_escape(&edge.to_name),
        ));
    }
    if edge.is_double {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.001 {
            let ux = dx / len;
            let uy = dy / len;
            let ax = x2 - ux * 8.0;
            let ay = y2 - uy * 8.0;
            let nx = -uy * 4.0;
            let ny = ux * 4.0;
            sg.set_fill_color(BORDER_COLOR);
            sg.set_stroke_color(None);
            sg.set_stroke_width(0.0, None);
            sg.svg_polygon(0.0, &[x2, y2, ax + nx, ay + ny, ax - nx, ay - ny]);
        }
    }
    // ISA arrow decoration for ->- and -<- links
    if let Some(is_superset) = edge.isa_arrow {
        if let Some(ref path_d) = edge.raw_path_d {
            render_isa_arrow_decoration(sg, path_d, is_superset);
        }
    } else if !edge.label.is_empty() {
        let (mx, my) = if let Some((lx, ly)) = edge.label_xy {
            // Java: x = label_xy.x + shield(0) + marginLabel(1)
            //        y = label_xy.y + shield(0) + marginLabel(1) + ascent
            let asc = crate::font_metrics::ascent("SansSerif", 11.0, false, false);
            (lx + 1.0, ly + 1.0 + asc)
        } else {
            ((x1 + x2) / 2.0, (y1 + y2) / 2.0 - 6.0)
        };
        let label_text_w =
            crate::font_metrics::text_width(&edge.label, "SansSerif", 11.0, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            &edge.label,
            mx,
            my,
            Some("sans-serif"),
            11.0,
            None,
            None,
            None,
            label_text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
    sg.push_raw("</g>");
}

/// Render the ISA arrow decoration (triangle arrowhead) at the midpoint of an edge.
/// Java renders this as: arc + 2 lines forming a triangle.
fn render_isa_arrow_decoration(sg: &mut SvgGraphic, path_d: &str, is_superset: bool) {
    // Parse the bezier curve from the path d-string to find the midpoint and tangent.
    // Path format: "Mx,y Ccx1,cy1 cx2,cy2 ex,ey" (cubic bezier)
    if let Some((mid, tangent)) = bezier_midpoint_tangent(path_d, is_superset) {
        let (mx, my) = mid;
        let (tx, ty) = tangent;
        let tlen = (tx * tx + ty * ty).sqrt();
        if tlen < 0.001 {
            return;
        }
        let ux = tx / tlen;
        let uy = ty / tlen;
        // Perpendicular (rotated 90° counterclockwise)
        let nx = -uy;
        let ny = ux;
        // Arc radius = 6
        let r = 6.0;
        // Arc endpoints: midpoint ± perpendicular * radius
        // Java always uses sweep=0 (counterclockwise arc).
        // For ->- (superset), "from" is the positive perpendicular side.
        // For -<- (subset), "from" is the negative perpendicular side.
        let (ax1, ay1, ax2, ay2) = if is_superset {
            (mx + nx * r, my + ny * r, mx - nx * r, my - ny * r)
        } else {
            (mx - nx * r, my - ny * r, mx + nx * r, my + ny * r)
        };
        // Line endpoints: for ->- go backward along tangent, for -<- go forward
        let back = 10.0;
        let dir = if is_superset { -1.0 } else { 1.0 };
        let lx1 = ax1 + dir * ux * back;
        let ly1 = ay1 + dir * uy * back;
        let lx2 = ax2 + dir * ux * back;
        let ly2 = ay2 + dir * uy * back;

        // Arc path - Java uses space (not comma) between arc endpoint coordinates
        sg.push_raw(&format!(
            r#"<path d="M{},{} A{r},{r} 0 0 0 {} {}" fill="none" style="stroke:{BORDER_COLOR};stroke-width:1.5;"/>"#,
            fmt_coord(ax1), fmt_coord(ay1),
            fmt_coord(ax2), fmt_coord(ay2),
        ));
        // Line 1
        sg.push_raw(&format!(
            r#"<line style="stroke:{BORDER_COLOR};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(ax1), fmt_coord(lx1), fmt_coord(ay1), fmt_coord(ly1),
        ));
        // Line 2
        sg.push_raw(&format!(
            r#"<line style="stroke:{BORDER_COLOR};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
            fmt_coord(ax2), fmt_coord(lx2), fmt_coord(ay2), fmt_coord(ly2),
        ));
    }
}

/// Parse a cubic bezier path and find the midpoint and tangent direction at the
/// overall midpoint of the path length.
/// Handles both single-segment ("M... C...") and multi-segment ("M... C... C...") paths.
fn bezier_midpoint_tangent(path_d: &str, _is_superset: bool) -> Option<((f64, f64), (f64, f64))> {
    // Parse path into cubic bezier segments
    let d = path_d.trim();
    let d = d.strip_prefix('M')?;
    let parts: Vec<f64> = d
        .replace('C', " ")
        .replace(',', " ")
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if parts.len() < 8 {
        return None;
    }

    // Build list of bezier segments: [(p0, p1, p2, p3), ...]
    let mut segments: Vec<[(f64, f64); 4]> = Vec::new();
    let start = (parts[0], parts[1]);
    let mut cur = start;
    let mut i = 2;
    while i + 5 < parts.len() {
        let p1 = (parts[i], parts[i + 1]);
        let p2 = (parts[i + 2], parts[i + 3]);
        let p3 = (parts[i + 4], parts[i + 5]);
        segments.push([cur, p1, p2, p3]);
        cur = p3;
        i += 6;
    }

    if segments.is_empty() {
        return None;
    }

    // Approximate arc length of each segment using linear approximation
    fn segment_length(seg: &[(f64, f64); 4]) -> f64 {
        let n = 10;
        let mut total = 0.0;
        let mut prev = seg[0];
        for k in 1..=n {
            let t = k as f64 / n as f64;
            let p = bezier_point(seg, t);
            let dx = p.0 - prev.0;
            let dy = p.1 - prev.1;
            total += (dx * dx + dy * dy).sqrt();
            prev = p;
        }
        total
    }

    fn bezier_point(seg: &[(f64, f64); 4], t: f64) -> (f64, f64) {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        (
            mt3 * seg[0].0 + 3.0 * mt2 * t * seg[1].0 + 3.0 * mt * t2 * seg[2].0 + t3 * seg[3].0,
            mt3 * seg[0].1 + 3.0 * mt2 * t * seg[1].1 + 3.0 * mt * t2 * seg[2].1 + t3 * seg[3].1,
        )
    }

    fn bezier_tangent(seg: &[(f64, f64); 4], t: f64) -> (f64, f64) {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;
        (
            3.0 * mt2 * (seg[1].0 - seg[0].0) + 6.0 * mt * t * (seg[2].0 - seg[1].0) + 3.0 * t2 * (seg[3].0 - seg[2].0),
            3.0 * mt2 * (seg[1].1 - seg[0].1) + 6.0 * mt * t * (seg[2].1 - seg[1].1) + 3.0 * t2 * (seg[3].1 - seg[2].1),
        )
    }

    if segments.len() == 1 {
        // Single segment: use t=0.5
        let seg = &segments[0];
        let mid = bezier_point(seg, 0.5);
        let tan = bezier_tangent(seg, 0.5);
        Some((mid, tan))
    } else {
        // Multi-segment: Java places the arrow at the junction between segments.
        // Use the midpoint junction (between segment N/2-1 and N/2).
        let junction_idx = segments.len() / 2;
        let junction = segments[junction_idx][0]; // start of segment = end of previous
        // Tangent: use the tangent at the end of the previous segment
        let prev_seg = &segments[junction_idx - 1];
        let tan = bezier_tangent(prev_seg, 1.0);
        Some((junction, tan))
    }
}

/// Render just the ISA circle and label (no edges).
fn render_isa_circle(sg: &mut SvgGraphic, isa: &ErdIsaLayout) {
    let (cx, cy) = isa.center;
    let r = isa.radius;

    // Render the ISA circle (Java: ellipse with rx=ry=12.5)
    sg.push_raw("<g>");
    sg.set_fill_color(ENTITY_BG);
    sg.set_stroke_color(Some(BORDER_COLOR));
    sg.set_stroke_width(0.5, None);
    sg.svg_ellipse(cx, cy, r, r, 0.0);

    // Label text
    let text_w = crate::font_metrics::text_width(&isa.kind_label, "SansSerif", FONT_SIZE, false, false);
    let asc = crate::font_metrics::ascent("SansSerif", FONT_SIZE, false, false);
    let desc = crate::font_metrics::descent("SansSerif", FONT_SIZE, false, false);
    let text_x = cx - text_w / 2.0;
    let text_y = cy + (asc - desc) / 2.0;
    sg.set_fill_color(TEXT_COLOR);
    sg.svg_text(
        &isa.kind_label,
        text_x,
        text_y,
        Some("sans-serif"),
        FONT_SIZE,
        None,
        None,
        None,
        text_w,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    sg.push_raw("</g>");
}

/// Render just the ISA edge paths (parent→center and center→children).
fn render_isa_edges(sg: &mut SvgGraphic, isa: &ErdIsaLayout) {
    // Render parent→center edge
    if let Some(ref path_d) = isa.parent_edge_path {
        let stroke_w = if isa.is_double { 2 } else { 1 };
        sg.push_raw(&format!(
            r#"<path d="{}" fill="none" style="stroke:{BORDER_COLOR};stroke-width:{stroke_w};"/>"#,
            path_d,
        ));
    }

    // Render center→child edges
    for child_edge in &isa.child_edges {
        if let Some(ref path_d) = child_edge.raw_path_d {
            sg.push_raw(&format!(
                r#"<path d="{}" fill="none" style="stroke:{BORDER_COLOR};stroke-width:1;"/>"#,
                path_d,
            ));
        }
    }
}

fn render_note(sg: &mut SvgGraphic, note: &ErdNoteLayout) {
    if let Some((x1, y1, x2, y2)) = note.connector {
        sg.set_stroke_color(Some(NOTE_BORDER));
        sg.set_stroke_width(1.0, Some((5.0, 3.0)));
        sg.svg_line(x1, y1, x2, y2, 0.0);
    }
    let (x, y, w, h) = (note.x, note.y, note.width, note.height);
    let fold = NOTE_FOLD;
    sg.set_fill_color(NOTE_BG);
    sg.set_stroke_color(Some(NOTE_BORDER));
    sg.set_stroke_width(0.5, None);
    sg.svg_polygon(
        0.0,
        &[
            x,
            y,
            x + w - fold,
            y,
            x + w,
            y + fold,
            x + w,
            y + h,
            x,
            y + h,
        ],
    );
    sg.push_raw(&format!(r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{NOTE_BORDER};stroke-width:0.5;"/>"#, fmt_coord(x + w - fold), fmt_coord(y), fmt_coord(x + w - fold), fmt_coord(y + fold), fmt_coord(x + w), fmt_coord(y + fold)));
    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &note.text,
        x + 10.0,
        y + 10.0 + FONT_SIZE,
        16.0,
        TEXT_COLOR,
        None,
        r#"font-size="13""#,
    );
    sg.push_raw(&tmp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::erd::*;
    use crate::model::erd::ErdDiagram;
    use crate::style::SkinParams;

    fn empty_diagram() -> ErdDiagram {
        ErdDiagram {
            entities: vec![],
            relationships: vec![],
            links: vec![],
            isas: vec![],
            direction: crate::model::erd::ErdDirection::TopToBottom,
            notes: vec![],
        }
    }
    fn empty_layout() -> ErdLayout {
        ErdLayout {
            entity_nodes: vec![],
            relationship_nodes: vec![],
            attribute_nodes: vec![],
            edges: vec![],
            attr_edges: vec![],
            isa_layouts: vec![],
            notes: vec![],
            width: 400.0,
            height: 300.0,
            svek_node_uids: std::collections::HashMap::new(),
            link_uids: std::collections::HashMap::new(),
        }
    }
    fn make_entity_node(id: &str, x: f64, y: f64, w: f64, h: f64) -> ErdNodeLayout {
        ErdNodeLayout {
            id: id.into(),
            label: id.into(),
            x,
            y,
            width: w,
            height: h,
            is_weak: false,
            is_identifying: false,
            is_relationship: false,
            source_order: 0,
            bg_color: None,
            line_color: None,
        }
    }
    fn make_attr(id: &str, parent: &str, x: f64, y: f64) -> ErdAttrLayout {
        ErdAttrLayout {
            id: id.into(),
            label: id.into(),
            parent: parent.into(),
            x,
            y,
            rx: 40.0,
            ry: 16.0,
            is_key: false,
            is_derived: false,
            is_multi: false,
            has_type: false,
            type_label: None,
            children: vec![],
        }
    }

    #[test]
    fn test_empty_diagram() {
        let svg = render_erd(&empty_diagram(), &empty_layout(), &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    }
    #[test]
    fn test_entity_rect() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("MOVIE", 50.0, 50.0, 100.0, 36.0));
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("MOVIE"));
        assert!(svg.contains(r#"lengthAdjust="spacing""#));
    }
    #[test]
    fn test_weak_entity_double_border() {
        let mut l = empty_layout();
        l.entity_nodes.push(ErdNodeLayout {
            is_weak: true,
            ..make_entity_node("CHILD", 50.0, 50.0, 100.0, 36.0)
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert_eq!(svg.matches("<rect").count(), 2);
    }
    #[test]
    fn test_relationship_diamond() {
        let mut l = empty_layout();
        l.relationship_nodes.push(ErdNodeLayout {
            id: "RENTED_TO".into(),
            label: "RENTED_TO".into(),
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 40.0,
            is_weak: false,
            is_identifying: false,
            is_relationship: true,
            source_order: 0,
            bg_color: None,
            line_color: None,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("RENTED_TO"));
    }
    #[test]
    fn test_identifying_relationship() {
        let mut l = empty_layout();
        l.relationship_nodes.push(ErdNodeLayout {
            id: "PARENT_OF".into(),
            label: "PARENT_OF".into(),
            x: 50.0,
            y: 50.0,
            width: 120.0,
            height: 40.0,
            is_weak: false,
            is_identifying: true,
            is_relationship: true,
            source_order: 0,
            bg_color: None,
            line_color: None,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert_eq!(svg.matches("<polygon").count(), 2);
    }
    #[test]
    fn test_attribute_ellipse() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(make_attr("Code", "E", 100.0, 40.0));
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<ellipse"));
        assert!(svg.contains("Code"));
    }
    #[test]
    fn test_key_attribute_underline() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(ErdAttrLayout {
            is_key: true,
            ..make_attr("Number", "E", 100.0, 40.0)
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains(r#"text-decoration="underline""#));
    }
    #[test]
    fn test_derived_attribute_dashed() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(ErdAttrLayout {
            is_derived: true,
            ..make_attr("Bonus", "E", 100.0, 40.0)
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("stroke-dasharray"));
    }
    #[test]
    fn test_multi_attribute_double_ellipse() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(ErdAttrLayout {
            is_multi: true,
            ..make_attr("Name", "E", 100.0, 40.0)
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert_eq!(svg.matches("<ellipse").count(), 2);
    }
    #[test]
    fn test_edge_rendering() {
        let mut l = empty_layout();
        l.edges.push(ErdEdgeLayout {
            from_id: "R".into(),
            to_id: "E".into(),
            from_name: "R".into(),
            to_name: "E".into(),
            from_point: (100.0, 100.0),
            to_point: (200.0, 100.0),
            label: "N".into(),
            is_double: false,
            source_line: 0,
            entity_idx_from: 0,
            entity_idx_to: 0,
            raw_path_d: None,
            label_xy: None,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("N"));
    }
    #[test]
    fn test_double_edge() {
        let mut l = empty_layout();
        l.edges.push(ErdEdgeLayout {
            from_id: "R".into(),
            to_id: "E".into(),
            from_name: "R".into(),
            to_name: "E".into(),
            from_point: (100.0, 100.0),
            to_point: (200.0, 100.0),
            label: "N".into(),
            is_double: true,
            source_line: 0,
            entity_idx_from: 0,
            entity_idx_to: 0,
            raw_path_d: None,
            label_xy: None,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.matches("<path").count() >= 2);
    }
    #[test]
    fn test_isa_circle() {
        let mut l = empty_layout();
        l.isa_layouts.push(ErdIsaLayout {
            parent_id: "PARENT".into(),
            kind_label: "d".into(),
            center: (200.0, 200.0),
            radius: 12.5,
            parent_edge_path: Some("M200,170 C200,180 200,190 200,187".to_string()),
            child_edges: vec![
                ErdIsaChildEdge {
                    child_id: "C1".into(),
                    raw_path_d: Some("M200,212 C180,230 160,240 160,250".to_string()),
                },
                ErdIsaChildEdge {
                    child_id: "C2".into(),
                    raw_path_d: Some("M200,212 C220,230 240,240 240,250".to_string()),
                },
            ],
            is_double: true,
            source_order: 0,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<ellipse"), "should render ISA as circle (ellipse)");
        assert!(svg.matches("<path").count() >= 3, "should have parent+child edge paths");
        assert!(svg.contains(">d<"), "should contain kind label");
    }
    #[test]
    fn test_attr_parent_lines() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(make_attr("X", "E", 140.0, 40.0));
        l.attribute_nodes.push(make_attr("Y", "E", 100.0, 40.0));
        l.attr_edges.push(ErdAttrEdge {
            raw_path_d: Some("M140,40 C120,60 110,80 140,118".to_string()),
            from_name: "E/X".to_string(),
            to_name: "E".to_string(),
            parent_source_order: 0,
        });
        l.attr_edges.push(ErdAttrEdge {
            raw_path_d: Some("M100,40 C110,60 120,80 140,118".to_string()),
            from_name: "E/Y".to_string(),
            to_name: "E".to_string(),
            parent_source_order: 0,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.matches("<path").count() >= 2);
    }
    #[test]
    fn test_xml_escaping() {
        let mut l = empty_layout();
        l.entity_nodes.push(ErdNodeLayout {
            label: "A & B < C".into(),
            ..make_entity_node("E", 50.0, 50.0, 120.0, 36.0)
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("A &amp; B &lt; C"));
    }
    #[test]
    fn test_attribute_type_annotation() {
        // In real usage, the label includes the type (e.g., "Born : DATE")
        // and no separate type text element is rendered.
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(ErdAttrLayout {
            has_type: true,
            type_label: Some("DATE".into()),
            label: "Born : DATE".into(),
            ..make_attr("Born", "E", 100.0, 40.0)
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("Born : DATE"));
    }
    #[test]
    fn test_svg_dimensions() {
        let l = ErdLayout {
            width: 500.0,
            height: 400.0,
            ..empty_layout()
        };
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(
            svg.contains("width=\"501px\""),
            "width should be ensure_visible_int(500)=501"
        );
        assert!(
            svg.contains("height=\"401px\""),
            "height should be ensure_visible_int(400)=401"
        );
        assert!(
            svg.contains("viewBox=\"0 0 501 401\""),
            "viewBox should use ensure_visible_int"
        );
    }
    #[test]
    fn test_nested_children_rendered() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        let mut a = make_attr("Name", "E", 100.0, 40.0);
        a.children = vec![
            make_attr("Fname", "Name", 80.0, 10.0),
            make_attr("Lname", "Name", 120.0, 10.0),
        ];
        l.attribute_nodes.push(a);
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("Fname"));
        assert!(svg.contains("Lname"));
        assert_eq!(svg.matches("<ellipse").count(), 3);
    }
    #[test]
    fn test_note_rendering() {
        let mut l = empty_layout();
        l.notes.push(ErdNoteLayout {
            text: "primary entity".into(),
            x: 180.0,
            y: 60.0,
            width: 110.0,
            height: 40.0,
            lines: vec!["primary entity".into()],
            connector: Some((180.0, 80.0, 140.0, 80.0)),
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("primary entity"));
        assert!(svg.contains("stroke-dasharray"));
    }
    #[test]
    fn test_multiline_note_rendering() {
        let mut l = empty_layout();
        l.notes.push(ErdNoteLayout {
            text: "line 1\nline 2".into(),
            x: 180.0,
            y: 60.0,
            width: 110.0,
            height: 56.0,
            lines: vec!["line 1".into(), "line 2".into()],
            connector: None,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(!svg.contains("<tspan"));
        assert!(svg.contains("line 1"));
        assert!(svg.contains("line 2"));
    }
}
