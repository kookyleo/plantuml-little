pub mod activity;
pub mod component;
pub mod ditaa;
pub mod erd;
pub mod gantt;
pub mod graphviz;
pub mod json_diagram;
pub mod mindmap;
pub mod nwdiag;
pub mod salt;
pub mod sequence;
pub mod state;
pub mod timing;
pub mod usecase;
pub mod wbs;

pub use graphviz::{
    layout as layout_graph, ClassNoteLayout, EdgeLayout, GraphLayout, LayoutEdge, LayoutGraph,
    LayoutNode, NodeLayout, RankDir,
};

use std::collections::HashMap;

use crate::model::{ClassDiagram, Diagram, Direction, Entity, EntityKind};
use crate::Result;

/// Unified layout result
#[derive(Debug)]
pub enum DiagramLayout {
    Class(GraphLayout),
    Sequence(sequence::SeqLayout),
    Activity(activity::ActivityLayout),
    State(state::StateLayout),
    Component(component::ComponentLayout),
    Ditaa(ditaa::DitaaLayout),
    Erd(erd::ErdLayout),
    Gantt(gantt::GanttLayout),
    Json(json_diagram::JsonLayout),
    Mindmap(mindmap::MindmapLayout),
    Nwdiag(nwdiag::NwdiagLayout),
    Salt(salt::SaltLayout),
    Timing(timing::TimingLayout),
    Wbs(wbs::WbsLayout),
    Yaml(json_diagram::JsonLayout),
    Dot(GraphLayout),
    UseCase(usecase::UseCaseLayout),
}

/// Estimated character width (pt)
const CHAR_WIDTH_PT: f64 = 7.2;
/// Line height (pt)
const LINE_HEIGHT_PT: f64 = 16.0;
/// Padding (pt)
const PADDING_PT: f64 = 10.0;
/// Header area height (pt)
const HEADER_HEIGHT_PT: f64 = 28.0;

/// Perform layout on a Diagram
pub fn layout(diagram: &Diagram) -> Result<DiagramLayout> {
    match diagram {
        Diagram::Class(cd) => {
            let gl = layout_class_diagram(cd)?;
            Ok(DiagramLayout::Class(gl))
        }
        Diagram::Sequence(sd) => {
            let sl = sequence::layout_sequence(sd)?;
            Ok(DiagramLayout::Sequence(sl))
        }
        Diagram::Activity(ad) => {
            let al = activity::layout_activity(ad)?;
            Ok(DiagramLayout::Activity(al))
        }
        Diagram::State(sd) => {
            let sl = state::layout_state(sd)?;
            Ok(DiagramLayout::State(sl))
        }
        Diagram::Component(cd) => {
            let cl = component::layout_component(cd)?;
            Ok(DiagramLayout::Component(cl))
        }
        Diagram::Ditaa(dd) => {
            let dl = ditaa::layout_ditaa(dd)?;
            Ok(DiagramLayout::Ditaa(dl))
        }
        Diagram::Erd(ed) => {
            let el = erd::layout_erd(ed)?;
            Ok(DiagramLayout::Erd(el))
        }
        Diagram::Gantt(gd) => {
            let gl = gantt::layout_gantt(gd)?;
            Ok(DiagramLayout::Gantt(gl))
        }
        Diagram::Json(jd) => {
            let jl = json_diagram::layout_json(jd)?;
            Ok(DiagramLayout::Json(jl))
        }
        Diagram::Mindmap(md) => {
            let ml = mindmap::layout_mindmap(md)?;
            Ok(DiagramLayout::Mindmap(ml))
        }
        Diagram::Nwdiag(nd) => {
            let nl = nwdiag::layout_nwdiag(nd)?;
            Ok(DiagramLayout::Nwdiag(nl))
        }
        Diagram::Salt(sd) => {
            let sl = salt::layout_salt(sd)?;
            Ok(DiagramLayout::Salt(sl))
        }
        Diagram::Timing(td) => {
            let tl = timing::layout_timing(td)?;
            Ok(DiagramLayout::Timing(tl))
        }
        Diagram::Wbs(wd) => {
            let wl = wbs::layout_wbs(wd)?;
            Ok(DiagramLayout::Wbs(wl))
        }
        Diagram::Yaml(yd) => {
            let yl = json_diagram::layout_json(yd)?;
            Ok(DiagramLayout::Yaml(yl))
        }
        Diagram::UseCase(ud) => {
            let ul = usecase::layout_usecase(ud)?;
            Ok(DiagramLayout::UseCase(ul))
        }
        Diagram::Dot(dd) => {
            // DOT passthrough: use a minimal placeholder layout
            let lg = LayoutGraph {
                nodes: vec![LayoutNode {
                    id: "dot_root".into(),
                    label: "DOT".into(),
                    width_pt: 200.0,
                    height_pt: 100.0,
                }],
                edges: vec![],
                rankdir: RankDir::TopToBottom,
            };
            let gl = graphviz::layout(&lg)?;
            let _ = &dd.source;
            Ok(DiagramLayout::Dot(gl))
        }
    }
}

/// Replace DOT-incompatible characters with safe identifiers
fn sanitize_id(name: &str) -> String {
    name.replace('<', "_LT_")
        .replace('>', "_GT_")
        .replace(',', "_COMMA_")
        .replace(' ', "_")
}

/// Estimate entity rendering size (width_pt, height_pt)
fn estimate_entity_size(entity: &Entity) -> (f64, f64) {
    // entity display name (including generic parameters)
    let mut name_display = entity.name.clone();
    if let Some(ref g) = entity.generic {
        name_display.push('<');
        name_display.push_str(g);
        name_display.push('>');
    }

    // check if a stereotype line is needed (interface / enum / abstract / custom stereotype)
    let has_stereotype_line = !entity.stereotypes.is_empty()
        || matches!(
            entity.kind,
            EntityKind::Interface | EntityKind::Enum | EntityKind::Abstract
        );

    // max stereotype text length (for width calculation)
    let stereotype_text_len = if has_stereotype_line {
        let kind_stereo_len = match entity.kind {
            EntityKind::Interface => "<<interface>>".len(),
            EntityKind::Enum => "<<enum>>".len(),
            EntityKind::Abstract => "<<abstract>>".len(),
            _ => 0,
        };
        let custom_stereo_len = entity
            .stereotypes
            .iter()
            .map(|s| s.0.len() + 4) // "<<" + text + ">>"
            .max()
            .unwrap_or(0);
        kind_stereo_len.max(custom_stereo_len)
    } else {
        0
    };

    // display text length for each member
    let max_member_len = entity
        .members
        .iter()
        .map(|m| {
            let vis_len = if m.visibility.is_some() { 2 } else { 0 }; // symbol + space
            let type_len = match &m.return_type {
                Some(t) => 2 + t.len(), // ": " + type
                None => 0,
            };
            vis_len + m.name.len() + type_len
        })
        .max()
        .unwrap_or(0);

    // width = longest text line * char width + padding on both sides
    let max_text_len = name_display
        .len()
        .max(stereotype_text_len)
        .max(max_member_len);
    let width = (max_text_len as f64 * CHAR_WIDTH_PT + 2.0 * PADDING_PT).max(60.0);

    // height = header + stereotype line (optional) + member lines * line height + bottom padding
    let stereotype_extra = if has_stereotype_line {
        LINE_HEIGHT_PT
    } else {
        0.0
    };
    let height = (HEADER_HEIGHT_PT
        + stereotype_extra
        + entity.members.len() as f64 * LINE_HEIGHT_PT
        + PADDING_PT)
        .max(36.0);

    log::debug!(
        "estimate_entity_size: {} -> ({:.1}, {:.1})",
        entity.name,
        width,
        height
    );

    (width, height)
}

/// Direction -> RankDir mapping
fn direction_to_rankdir(dir: &Direction) -> RankDir {
    match dir {
        Direction::TopToBottom => RankDir::TopToBottom,
        Direction::LeftToRight => RankDir::LeftToRight,
        Direction::BottomToTop => RankDir::BottomToTop,
        Direction::RightToLeft => RankDir::RightToLeft,
    }
}

/// Note size estimation constants
const NOTE_CHAR_WIDTH: f64 = 7.2;
const NOTE_LINE_HEIGHT: f64 = 16.0;
const NOTE_PADDING: f64 = 10.0;
/// Gap between note and target entity
const NOTE_GAP: f64 = 16.0;

/// Perform layout on a class diagram
fn layout_class_diagram(cd: &ClassDiagram) -> Result<GraphLayout> {
    log::debug!(
        "layout_class_diagram: {} entities, {} links, {} notes",
        cd.entities.len(),
        cd.links.len(),
        cd.notes.len()
    );

    // build name -> sanitized id mapping
    let name_to_id: HashMap<String, String> = cd
        .entities
        .iter()
        .map(|e| (e.name.clone(), sanitize_id(&e.name)))
        .collect();

    // build LayoutNode list
    let nodes: Vec<LayoutNode> = cd
        .entities
        .iter()
        .map(|e| {
            let (w, h) = estimate_entity_size(e);
            LayoutNode {
                id: name_to_id
                    .get(&e.name)
                    .cloned()
                    .unwrap_or_else(|| sanitize_id(&e.name)),
                label: e.name.clone(),
                width_pt: w,
                height_pt: h,
            }
        })
        .collect();

    // build LayoutEdge list
    let edges: Vec<LayoutEdge> = cd
        .links
        .iter()
        .map(|link| {
            let from_id = name_to_id
                .get(&link.from)
                .cloned()
                .unwrap_or_else(|| sanitize_id(&link.from));
            let to_id = name_to_id
                .get(&link.to)
                .cloned()
                .unwrap_or_else(|| sanitize_id(&link.to));
            LayoutEdge {
                from: from_id,
                to: to_id,
                label: link.label.clone(),
            }
        })
        .collect();

    let graph = LayoutGraph {
        nodes,
        edges,
        rankdir: direction_to_rankdir(&cd.direction),
    };

    let mut layout = layout_graph(&graph)?;

    // compute note layout
    layout.notes = compute_note_layouts(&cd.notes, &layout.nodes, &name_to_id);

    // expand total_width / total_height to accommodate notes
    for note in &layout.notes {
        let right_edge = note.x + note.width;
        let bottom_edge = note.y + note.height;
        if right_edge > layout.total_width {
            layout.total_width = right_edge;
        }
        if bottom_edge > layout.total_height {
            layout.total_height = bottom_edge;
        }
    }
    // notes may produce negative coordinates on left or top, shift if needed
    let min_x = layout.notes.iter().map(|n| n.x).fold(0.0_f64, f64::min);
    let min_y = layout.notes.iter().map(|n| n.y).fold(0.0_f64, f64::min);
    if min_x < 0.0 || min_y < 0.0 {
        let shift_x = if min_x < 0.0 { -min_x } else { 0.0 };
        let shift_y = if min_y < 0.0 { -min_y } else { 0.0 };
        for n in &mut layout.nodes {
            n.cx += shift_x;
            n.cy += shift_y;
        }
        for e in &mut layout.edges {
            for pt in &mut e.points {
                pt.0 += shift_x;
                pt.1 += shift_y;
            }
            if let Some(ref mut tip) = e.arrow_tip {
                tip.0 += shift_x;
                tip.1 += shift_y;
            }
        }
        for n in &mut layout.notes {
            n.x += shift_x;
            n.y += shift_y;
            if let Some(ref mut conn) = n.connector {
                conn.0 += shift_x;
                conn.1 += shift_y;
                conn.2 += shift_x;
                conn.3 += shift_y;
            }
        }
        layout.total_width += shift_x;
        layout.total_height += shift_y;
    }

    Ok(layout)
}

/// Compute note layout positions
fn compute_note_layouts(
    notes: &[crate::model::ClassNote],
    nodes: &[graphviz::NodeLayout],
    name_to_id: &HashMap<String, String>,
) -> Vec<graphviz::ClassNoteLayout> {
    let node_map: HashMap<&str, &graphviz::NodeLayout> =
        nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    notes
        .iter()
        .map(|note| {
            let lines: Vec<String> = note
                .text
                .lines()
                .map(std::string::ToString::to_string)
                .collect();
            let max_line_len = lines
                .iter()
                .map(std::string::String::len)
                .max()
                .unwrap_or(0);
            let note_width = (max_line_len as f64 * NOTE_CHAR_WIDTH + NOTE_PADDING * 2.0).max(60.0);
            let note_height =
                (lines.len() as f64 * NOTE_LINE_HEIGHT + NOTE_PADDING * 2.0).max(30.0);

            // find the layout node for the target entity
            let target_node = note.target.as_ref().and_then(|target| {
                let sid = name_to_id
                    .get(target)
                    .cloned()
                    .unwrap_or_else(|| sanitize_id(target));
                node_map.get(sid.as_str()).copied()
            });

            let (x, y, connector) = if let Some(nl) = target_node {
                let entity_left = nl.cx - nl.width / 2.0;
                let entity_right = nl.cx + nl.width / 2.0;
                let entity_top = nl.cy - nl.height / 2.0;
                let entity_bottom = nl.cy + nl.height / 2.0;
                let entity_center_y = nl.cy;

                match note.position.as_str() {
                    "right" => {
                        let nx = entity_right + NOTE_GAP;
                        let ny = entity_center_y - note_height / 2.0;
                        let conn = (nx, entity_center_y, entity_right, entity_center_y);
                        (nx, ny, Some(conn))
                    }
                    "left" => {
                        let nx = entity_left - NOTE_GAP - note_width;
                        let ny = entity_center_y - note_height / 2.0;
                        let conn = (
                            nx + note_width,
                            entity_center_y,
                            entity_left,
                            entity_center_y,
                        );
                        (nx, ny, Some(conn))
                    }
                    "top" => {
                        let nx = nl.cx - note_width / 2.0;
                        let ny = entity_top - NOTE_GAP - note_height;
                        let conn = (nl.cx, ny + note_height, nl.cx, entity_top);
                        (nx, ny, Some(conn))
                    }
                    "bottom" => {
                        let nx = nl.cx - note_width / 2.0;
                        let ny = entity_bottom + NOTE_GAP;
                        let conn = (nl.cx, ny, nl.cx, entity_bottom);
                        (nx, ny, Some(conn))
                    }
                    _ => {
                        // default: place on right side
                        let nx = entity_right + NOTE_GAP;
                        let ny = entity_center_y - note_height / 2.0;
                        let conn = (nx, entity_center_y, entity_right, entity_center_y);
                        (nx, ny, Some(conn))
                    }
                }
            } else {
                // no target entity, place at a floating position near bottom-right
                let max_x = nodes
                    .iter()
                    .map(|n| n.cx + n.width / 2.0)
                    .fold(0.0_f64, f64::max);
                let max_y = nodes
                    .iter()
                    .map(|n| n.cy + n.height / 2.0)
                    .fold(0.0_f64, f64::max);
                (max_x + NOTE_GAP, max_y + NOTE_GAP, None)
            };

            graphviz::ClassNoteLayout {
                text: note.text.clone(),
                x,
                y,
                width: note_width,
                height: note_height,
                lines,
                connector,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Entity, EntityKind, Member, MemberModifiers, Visibility};

    fn empty_entity(name: &str) -> Entity {
        Entity {
            name: name.to_string(),
            kind: EntityKind::Class,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
        }
    }

    fn make_member(vis: Option<Visibility>, name: &str, ret: Option<&str>) -> Member {
        Member {
            visibility: vis,
            name: name.to_string(),
            return_type: ret.map(|s| s.to_string()),
            is_method: false,
            modifiers: MemberModifiers::default(),
        }
    }

    #[test]
    fn estimate_size_empty_class_returns_minimum() {
        let e = empty_entity("Foo");
        let (w, h) = estimate_entity_size(&e);
        assert!(w >= 60.0, "width should be >= 60, got {w}");
        assert!(h >= 36.0, "height should be >= 36, got {h}");
    }

    #[test]
    fn estimate_size_accounts_for_members() {
        let e = Entity {
            name: "A".to_string(),
            kind: EntityKind::Class,
            stereotypes: vec![],
            members: vec![
                make_member(
                    Some(Visibility::Private),
                    "longFieldNameHere",
                    Some("String"),
                ),
                make_member(Some(Visibility::Public), "id", Some("i32")),
            ],
            color: None,
            generic: None,
        };
        let (w, h) = estimate_entity_size(&e);

        let expected_min_height = HEADER_HEIGHT_PT + 2.0 * LINE_HEIGHT_PT + PADDING_PT;
        assert!(
            h >= expected_min_height,
            "height {h} should be >= {expected_min_height}"
        );

        let member_text_len = 2 + "longFieldNameHere".len() + 2 + "String".len();
        let expected_min_width = member_text_len as f64 * CHAR_WIDTH_PT + 2.0 * PADDING_PT;
        assert!(
            w >= expected_min_width,
            "width {w} should be >= {expected_min_width}"
        );
    }

    #[test]
    fn estimate_size_interface_adds_stereotype_line() {
        let e = Entity {
            name: "Runnable".to_string(),
            kind: EntityKind::Interface,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
        };
        let (_, h) = estimate_entity_size(&e);

        let expected_min = HEADER_HEIGHT_PT + LINE_HEIGHT_PT + PADDING_PT;
        assert!(
            h >= expected_min,
            "interface height {h} should be >= {expected_min}"
        );
    }

    #[test]
    fn estimate_size_with_generic_widens() {
        let plain = empty_entity("Map");
        let generic = Entity {
            generic: Some("K, V".to_string()),
            ..plain.clone()
        };
        let (w_plain, _) = estimate_entity_size(&plain);
        let (w_generic, _) = estimate_entity_size(&generic);
        assert!(
            w_generic > w_plain,
            "generic entity should be wider: {w_generic} > {w_plain}"
        );
    }

    #[test]
    fn sanitize_id_escapes_special_chars() {
        assert_eq!(sanitize_id("List<String>"), "List_LT_String_GT_");
        assert_eq!(sanitize_id("Map<K, V>"), "Map_LT_K_COMMA__V_GT_");
        assert_eq!(sanitize_id("Simple"), "Simple");
        assert_eq!(sanitize_id("My Class"), "My_Class");
    }

    #[test]
    fn direction_maps_to_rankdir() {
        assert!(matches!(
            direction_to_rankdir(&Direction::TopToBottom),
            RankDir::TopToBottom
        ));
        assert!(matches!(
            direction_to_rankdir(&Direction::LeftToRight),
            RankDir::LeftToRight
        ));
        assert!(matches!(
            direction_to_rankdir(&Direction::BottomToTop),
            RankDir::BottomToTop
        ));
        assert!(matches!(
            direction_to_rankdir(&Direction::RightToLeft),
            RankDir::RightToLeft
        ));
    }

    #[test]
    fn note_position_right_of_entity() {
        use crate::model::ClassNote;

        let nodes = vec![graphviz::NodeLayout {
            id: "Foo".into(),
            cx: 100.0,
            cy: 50.0,
            width: 120.0,
            height: 80.0,
        }];
        let name_to_id: HashMap<String, String> = [("Foo".to_string(), "Foo".to_string())]
            .into_iter()
            .collect();
        let notes = vec![ClassNote {
            text: "hello".to_string(),
            position: "right".to_string(),
            target: Some("Foo".to_string()),
        }];

        let result = compute_note_layouts(&notes, &nodes, &name_to_id);
        assert_eq!(result.len(), 1);
        let nl = &result[0];
        // note x should be past entity right edge + gap
        let entity_right = 100.0 + 120.0 / 2.0; // 160
        assert!(
            nl.x >= entity_right,
            "note x={} should be >= entity_right={}",
            nl.x,
            entity_right
        );
        assert!(nl.width > 0.0);
        assert!(nl.height > 0.0);
        assert!(nl.connector.is_some());
    }

    #[test]
    fn note_position_left_of_entity() {
        use crate::model::ClassNote;

        let nodes = vec![graphviz::NodeLayout {
            id: "Bar".into(),
            cx: 200.0,
            cy: 100.0,
            width: 100.0,
            height: 60.0,
        }];
        let name_to_id: HashMap<String, String> = [("Bar".to_string(), "Bar".to_string())]
            .into_iter()
            .collect();
        let notes = vec![ClassNote {
            text: "left note".to_string(),
            position: "left".to_string(),
            target: Some("Bar".to_string()),
        }];

        let result = compute_note_layouts(&notes, &nodes, &name_to_id);
        assert_eq!(result.len(), 1);
        let nl = &result[0];
        let entity_left = 200.0 - 100.0 / 2.0; // 150
                                               // note right edge should be before entity left edge
        assert!(
            nl.x + nl.width <= entity_left,
            "note right edge={} should be <= entity_left={}",
            nl.x + nl.width,
            entity_left
        );
        assert!(nl.connector.is_some());
    }

    #[test]
    fn note_without_target_floats() {
        use crate::model::ClassNote;

        let nodes = vec![graphviz::NodeLayout {
            id: "X".into(),
            cx: 50.0,
            cy: 50.0,
            width: 80.0,
            height: 40.0,
        }];
        let name_to_id: HashMap<String, String> =
            [("X".to_string(), "X".to_string())].into_iter().collect();
        let notes = vec![ClassNote {
            text: "floating".to_string(),
            position: "right".to_string(),
            target: None,
        }];

        let result = compute_note_layouts(&notes, &nodes, &name_to_id);
        assert_eq!(result.len(), 1);
        assert!(
            result[0].connector.is_none(),
            "floating note should have no connector"
        );
    }
}
