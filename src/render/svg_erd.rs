use crate::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::erd::{
    ErdAttrLayout, ErdEdgeLayout, ErdIsaLayout, ErdLayout, ErdNodeLayout, ErdNoteLayout,
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
    for node in &layout.entity_nodes {
        render_entity(&mut sg, node, ent_bg, ent_border, ent_font);
    }
    for node in &layout.relationship_nodes {
        render_relationship(&mut sg, node);
    }
    for attr in &layout.attribute_nodes {
        render_attribute(&mut sg, attr);
    }
    render_attr_parent_lines(
        &mut sg,
        &layout.attribute_nodes,
        &layout.entity_nodes,
        &layout.relationship_nodes,
    );
    for (i, edge) in layout.edges.iter().enumerate() {
        render_edge(&mut sg, edge, i);
    }
    for isa in &layout.isa_layouts {
        render_isa(&mut sg, isa);
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
    sg.push_raw("<g>");
    if node.is_weak {
        sg.set_fill_color(bg);
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(0.5, None);
        sg.svg_rectangle(x, y, w, h, 0.0, 0.0, 0.0);
        let inset = 3.0;
        sg.set_fill_color(bg);
        sg.set_stroke_color(Some(border));
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
        sg.set_fill_color(bg);
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(0.5, None);
        sg.svg_rectangle(x, y, w, h, 0.0, 0.0, 0.0);
    }
    let tx = x + 10.0;
    let ty = y + h / 2.0 + FONT_SIZE * 0.35;
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
    sg.push_raw("<g>");
    if node.is_identifying {
        sg.set_fill_color(ENTITY_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_polygon(0.0, &[x, cy, cx, y, x + w, cy, cx, y + h]);
        let inset = 4.0;
        sg.set_fill_color(ENTITY_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_polygon(
            0.0,
            &[
                x + inset * 1.5,
                cy,
                cx,
                y + inset,
                x + w - inset * 1.5,
                cy,
                cx,
                y + h - inset,
            ],
        );
    } else {
        sg.set_fill_color(ENTITY_BG);
        sg.set_stroke_color(Some(BORDER_COLOR));
        sg.set_stroke_width(0.5, None);
        sg.svg_polygon(0.0, &[x, cy, cx, y, x + w, cy, cx, y + h]);
    }
    let ty = cy + FONT_SIZE * 0.35;
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
    let ty = cy + FONT_SIZE * 0.35;
    let text_w = rx * 2.0 - 10.0;
    let text_x = cx - rx + 5.0;
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
    if let Some(ref type_label) = attr.type_label {
        let type_y = cy + FONT_SIZE * 0.35 + FONT_SIZE + 2.0;
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            type_label,
            text_x,
            type_y,
            Some("sans-serif"),
            FONT_SIZE - 2.0,
            None,
            Some("italic"),
            None,
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
    for child in &attr.children {
        render_attribute(sg, child);
        render_path_line(sg, cx, cy - ry, child.x, child.y + child.ry);
    }
}

fn render_attr_parent_lines(
    sg: &mut SvgGraphic,
    attrs: &[ErdAttrLayout],
    entities: &[ErdNodeLayout],
    relationships: &[ErdNodeLayout],
) {
    for attr in attrs {
        if let Some((px, py)) = entities
            .iter()
            .chain(relationships.iter())
            .find(|n| n.id == attr.parent)
            .map(|n| (n.x + n.width / 2.0, n.y + n.height / 2.0))
        {
            render_path_line(sg, attr.x, attr.y, px, py);
        }
    }
}

fn render_edge(sg: &mut SvgGraphic, edge: &ErdEdgeLayout, link_idx: usize) {
    let (x1, y1) = edge.from_point;
    let (x2, y2) = edge.to_point;
    // Java wraps each link in <!--link From to To--> comment and <g class="link" ...>
    sg.push_raw(&format!(
        "<!--link {} to {}-->",
        xml_escape(&edge.from_name),
        xml_escape(&edge.to_name)
    ));
    let ent_from = format!("ent{:04}", edge.entity_idx_from + 2);
    let ent_to = format!("ent{:04}", edge.entity_idx_to + 2);
    sg.push_raw(&format!(
        r#"<g class="link" data-entity-1="{}" data-entity-2="{}" data-link-type="association" data-source-line="{}" id="lnk{}">"#,
        ent_from, ent_to, edge.source_line, link_idx + 2 + edge.entity_idx_to
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
    } else {
        // Java uses cubic bezier C command even for straight lines:
        // M x1,y1 C cx1,cy1 cx2,cy2 x2,y2
        // For a straight line, control points are at 1/3 and 2/3 positions
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
    if !edge.label.is_empty() {
        let mx = (x1 + x2) / 2.0;
        let my = (y1 + y2) / 2.0 - 6.0;
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

fn render_isa(sg: &mut SvgGraphic, isa: &ErdIsaLayout) {
    let (cx, cy) = isa.triangle_center;
    let s = isa.triangle_size;
    let top_y = cy - s * 0.5;
    let bot_y = cy + s * 0.5;
    let left_x = cx - s * 0.6;
    let right_x = cx + s * 0.6;
    sg.set_fill_color(ENTITY_BG);
    sg.set_stroke_color(Some(BORDER_COLOR));
    sg.set_stroke_width(0.5, None);
    sg.svg_polygon(0.0, &[cx, top_y, right_x, bot_y, left_x, bot_y]);
    let label_y = cy + FONT_SIZE * 0.2;
    let escaped = xml_escape(&isa.kind_label);
    let text_w = escaped.len() as f64 * 7.0;
    sg.set_fill_color(TEXT_COLOR);
    sg.svg_text(
        &isa.kind_label,
        cx - text_w / 2.0,
        label_y,
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
    let (ppx, ppy) = isa.parent_point;
    render_path_line(sg, ppx, ppy, cx, top_y);
    for (_child_id, (child_x, child_y)) in &isa.child_points {
        render_path_line(sg, cx, bot_y, *child_x, *child_y);
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
            isa_layouts: vec![],
            notes: vec![],
            width: 400.0,
            height: 300.0,
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
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.matches("<path").count() >= 2);
    }
    #[test]
    fn test_isa_triangle() {
        let mut l = empty_layout();
        l.isa_layouts.push(ErdIsaLayout {
            parent_id: "PARENT".into(),
            kind_label: "d".into(),
            triangle_center: (200.0, 200.0),
            triangle_size: 24.0,
            parent_point: (200.0, 170.0),
            child_points: vec![("C1".into(), (160.0, 250.0)), ("C2".into(), (240.0, 250.0))],
            is_double: true,
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("<polygon"));
        assert!(svg.matches("<path").count() >= 3);
        assert!(svg.contains(">d<"));
    }
    #[test]
    fn test_attr_parent_lines() {
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(make_attr("X", "E", 140.0, 40.0));
        l.attribute_nodes.push(make_attr("Y", "E", 100.0, 40.0));
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
        let mut l = empty_layout();
        l.entity_nodes
            .push(make_entity_node("E", 100.0, 100.0, 80.0, 36.0));
        l.attribute_nodes.push(ErdAttrLayout {
            has_type: true,
            type_label: Some("DATE".into()),
            ..make_attr("Born", "E", 100.0, 40.0)
        });
        let svg = render_erd(&empty_diagram(), &l, &SkinParams::default()).unwrap();
        assert!(svg.contains("DATE"));
        assert!(svg.contains("font-style=\"italic\""));
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
