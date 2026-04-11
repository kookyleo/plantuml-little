use std::collections::HashMap;
use std::fmt::Write;

use crate::font_metrics;
use crate::klimt::drawable::{
    DrawStyle, Drawable, EllipseShape, LineShape, PolygonShape, RectShape,
};
use crate::klimt::sanitize_group_metadata_value;
use crate::klimt::svg::{fmt_coord, svg_comment_escape, xml_escape, LengthAdjust, SvgGraphic};
use crate::layout::class_group_header_metrics;
use crate::layout::graphviz::{
    has_link_arrow_indicator, is_link_arrow_backward, strip_link_arrow_text, ClassNoteLayout,
    ClusterLayout, EdgeLayout, GraphLayout, NodeLayout,
};
use crate::layout::split_member_lines;
use crate::model::{
    ArrowHead, ClassDiagram, ClassHideShowRule, ClassPortion, ClassRuleTarget, Entity, EntityKind,
    GroupKind, LineStyle, Link, Member, RectSymbol, Visibility,
};
use crate::skin::rose::{
    BORDER_COLOR, ENTITY_BG, NOTE_BG, NOTE_BORDER, NOTE_FOLD, NOTE_PADDING as NOTE_TEXT_PADDING,
    TEXT_COLOR,
};
use crate::style::SkinParams;
use crate::svek::edge::LineOfSegments;
use crate::Result;

use super::svg::{
    compute_viewport, write_bg_rect, write_svg_root_bg, BodyResult, BoundsTracker, ViewportConfig,
};
use super::svg_meta::text_block_h;
use super::svg_richtext::{
    clear_section_title_bounds, get_default_font_family_pub, render_creole_display_lines,
    render_creole_text, set_section_title_bounds, SectionTitleBounds,
};

// ── Class diagram constants — all sourced from Java PlantUML code ────

/// FontParam.CLASS = 12, but class name renders at 14 in SVG (EntityImageClassHeader uses 14pt).
const FONT_SIZE: f64 = 14.0;
/// MethodsOrFieldsArea: empty compartment margin_top(4) + margin_bottom(4) = 8.
#[allow(dead_code)] // Java-ported rendering constant
const LINE_HEIGHT: f64 = 8.0;
/// EntityImageClassHeader name margin: withMargin(name, 3, 3, 0, 0) -> right padding = 3.
#[allow(dead_code)] // Java-ported rendering constant
const PADDING: f64 = 3.0;
/// HeaderLayout height when no stereotype: max(circleDim.h(32), nameDim.h(16.3)+10=26.3) = 32.
const HEADER_HEIGHT: f64 = 32.0;
/// SvekResult.java:133 -- moveDelta(6 - minMax.getMinX(), 6 - minMax.getMinY()).
#[allow(dead_code)] // Java-ported rendering constant
const MARGIN: f64 = 6.0;
/// EntityImageClassHeader.java:150 -- withMargin(circledChar, left=4, right=0, top=5, bottom=5).
#[allow(dead_code)] // Java-ported rendering constant
const CIRCLE_LEFT_PAD: f64 = 4.0;
/// SkinParam.circledCharacterRadius = 17/3+6 = 11. Diameter = 22.
#[allow(dead_code)] // Java-ported rendering constant
const CIRCLE_DIAMETER: f64 = 22.0;
/// MethodsOrFieldsArea: empty compartment = margin_top(4) + margin_bottom(4).
const EMPTY_COMPARTMENT: f64 = 8.0;
/// Circled character block: diameter(22) + marginLeft(4) + marginRight(0) = 26.
const HEADER_CIRCLE_BLOCK_WIDTH: f64 = 26.0;
/// Circled character block: diameter(22) + marginTop(5) + marginBottom(5) = 32.
const HEADER_CIRCLE_BLOCK_HEIGHT: f64 = 32.0;
/// SansSerif 14pt plain: ascent(12.995117) + descent(3.301758) from Java AWT FontMetrics.
const HEADER_NAME_BLOCK_HEIGHT: f64 = 16.296875;
/// SansSerif 14pt plain ascent from Java AWT FontMetrics.
const HEADER_NAME_BASELINE: f64 = 12.995117;
/// EntityImageClassHeader.java:105 -- withMargin(name, 3, 3, 0, 0): left(3) + right(3) = 6.
const HEADER_NAME_BLOCK_MARGIN_X: f64 = 6.0;
/// FontParam.CLASS_STEREOTYPE = 12pt.
const HEADER_STEREO_FONT_SIZE: f64 = 12.0;
/// SansSerif 12pt italic: ascent(11.138672) + descent(2.830078) from Java AWT FontMetrics.
const HEADER_STEREO_LINE_HEIGHT: f64 = 13.96875;
/// SansSerif 12pt italic ascent from Java AWT FontMetrics.
const HEADER_STEREO_BASELINE: f64 = 11.138672;
/// HeaderLayout.java:77 -- max(..., stereoDim.h + nameDim.h + 10, ...) -> gap = 10.
const HEADER_STEREO_NAME_GAP: f64 = 10.0;
const HEADER_STEREO_BLOCK_MARGIN: f64 = 2.0;

// -- Member area (fields/methods) constants --
/// SansSerif 14pt height from Java AWT FontMetrics. Used as row height in member area.
#[allow(dead_code)] // Java-ported rendering constant
const MEMBER_ROW_HEIGHT: f64 = 16.296875;
/// margin_top(4) + MEMBER_ROW_HEIGHT + margin_bottom(4).
#[allow(dead_code)] // Java-ported rendering constant
const MEMBER_BLOCK_HEIGHT_ONE_ROW: f64 = 24.296875;
/// Icon y from section separator: margin_top(4) + nudge(2) + (16.296875 - 11) / 2 = 8.6484375.
#[allow(dead_code)] // Java-ported rendering constant
const MEMBER_ICON_Y_FROM_SEP: f64 = 8.6484375;
/// VisibilityModifier.drawCircle: UTranslate offset (+2, +2).
#[allow(dead_code)] // Java-ported rendering constant
const MEMBER_ICON_DRAW_OFFSET: f64 = 2.0;
/// UEllipse(6, 6): rx = ry = 3.
#[allow(dead_code)] // Java-ported rendering constant
const MEMBER_ICON_RADIUS: f64 = 3.0;
/// MethodsOrFieldsArea margin left = 6.
const MEMBER_ICON_X_OFFSET: f64 = 6.0;
/// margin_left(6) + col2(circledCharRadius(11) + 3) = 20.
const MEMBER_TEXT_X_WITH_ICON: f64 = 20.0;
/// margin_left(6) when no visibility icon column.
const MEMBER_TEXT_X_NO_ICON: f64 = 6.0;
/// margin_top(4) + SansSerif 14pt ascent(12.995117) = 16.995117.
#[allow(dead_code)] // Java-ported rendering constant
const MEMBER_TEXT_Y_OFFSET: f64 = 16.995117;

/// Entity-level visibility icon block size (SkinParam.circledCharacterRadius = 11).
const ENTITY_VIS_ICON_BLOCK_SIZE: f64 = 11.0;

// -- Generic type box rendering constants --
/// Generic text font size (FontParam.CLASS_STEREOTYPE = 12pt italic).
const GENERIC_FONT_SIZE: f64 = 12.0;
/// SansSerif 12pt italic ascent from Java AWT FontMetrics.
const GENERIC_BASELINE: f64 = 11.138672;
/// SansSerif 12pt italic: ascent + descent = 13.96875.
const GENERIC_TEXT_HEIGHT: f64 = 13.96875;
/// Inner margin around generic text (withMargin(genericBlock, 1, 1)).
const GENERIC_INNER_MARGIN: f64 = 1.0;
/// Outer margin around TextBlockGeneric (withMargin(genericBlock, 1, 1)).
const GENERIC_OUTER_MARGIN: f64 = 1.0;
/// HeaderLayout.java:112 -- delta = 4 for positioning.
const GENERIC_DELTA: f64 = 4.0;
/// Protrusion above entity rect = delta - outer_margin = 3.
const GENERIC_PROTRUSION: f64 = GENERIC_DELTA - GENERIC_OUTER_MARGIN;

const CLASS_NOTE_FOLD: f64 = 10.0;
const LINK_COLOR: &str = BORDER_COLOR;
/// Java PlantUML renders link labels at font-size 13 (not 14).
const LINK_LABEL_FONT_SIZE: f64 = 13.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum QualifierEndpoint {
    Tail,
    Head,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct QualifierKey {
    link_idx: usize,
    endpoint: QualifierEndpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum KalPosition {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct KalPlacement {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) shift_x: f64,
}

pub(super) fn sanitize_id(name: &str) -> String {
    name.replace('<', "_LT_")
        .replace('>', "_GT_")
        .replace(',', "_COMMA_")
        .replace('.', "_DOT_")
        .replace(' ', "_")
}

fn svg_group_metadata_attr(value: &str) -> String {
    xml_escape(&sanitize_group_metadata_value(value))
}

fn class_link_id_for_svg(link: &Link) -> String {
    let from = crate::layout::class_entity_display_name(&link.from);
    let to = crate::layout::class_entity_display_name(&link.to);
    if link_looks_reverted_for_svg(link) {
        format!("{from}-backto-{to}")
    } else if link_looks_no_decor_at_all_svg(link) {
        format!("{from}-{to}")
    } else {
        format!("{from}-to-{to}")
    }
}

fn link_looks_reverted_for_svg(link: &Link) -> bool {
    link.left_head != ArrowHead::None && link.right_head == ArrowHead::None
}

fn link_looks_no_decor_at_all_svg(link: &Link) -> bool {
    (link.left_head == ArrowHead::None && link.right_head == ArrowHead::None)
        || (link.left_head != ArrowHead::None && link.right_head != ArrowHead::None)
}

// ── Class diagram rendering ─────────────────────────────────────────

pub(super) fn render_class(
    cd: &crate::model::ClassDiagram,
    layout: &GraphLayout,
    skin: &SkinParams,
    class_body_offset: Option<(f64, f64)>,
) -> Result<BodyResult> {
    let node_map: HashMap<&str, &NodeLayout> =
        layout.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    // Pre-shift offset (degenerated svek case): when the wrap_with_meta layer
    // already knows the body's final position, we add it directly here so the
    // emitted coordinates skip the lossy format→parse→shift→reformat round
    // trip in `offset_svg_coords`.
    let (pre_off_x, pre_off_y) = class_body_offset.unwrap_or((0.0, 0.0));
    // Rust normalizes Svek coordinates back to the origin for rendering, but
    // Java renders at the post-Svek coordinates directly. `render_offset`
    // re-applies the exact per-axis delta needed to reconstruct the Java space.
    let edge_offset_x = layout.render_offset.0 + pre_off_x;
    // Java's LimitFinder sees generic boxes protruding above the owning entity
    // header. That only changes the global min_y when a generic entity is also
    // on the diagram's topmost entity row; lower generic entities do not affect
    // the final moveDelta-derived y origin.
    let min_entity_top = cd
        .entities
        .iter()
        .filter_map(|entity| {
            node_map
                .get(sanitize_id(&entity.name).as_str())
                .map(|node| node.cy - node.height / 2.0)
        })
        .fold(f64::INFINITY, f64::min);
    let generic_y_adjust = if cd
        .entities
        .iter()
        .filter(|entity| entity.generic.is_some())
        .filter_map(|entity| {
            node_map
                .get(sanitize_id(&entity.name).as_str())
                .map(|node| node.cy - node.height / 2.0)
        })
        .any(|top| (top - min_entity_top).abs() <= 0.001)
    {
        GENERIC_PROTRUSION
    } else {
        0.0
    };
    let edge_offset_y = layout.render_offset.1 + generic_y_adjust + pre_off_y;
    let mut tracker = BoundsTracker::new();
    let mut sg = SvgGraphic::new(0, 1.0);
    let arrow_color = skin.arrow_color(LINK_COLOR);
    let group_meta: HashMap<&str, &crate::model::Group> = cd
        .groups
        .iter()
        .map(|group| (group.name.as_str(), group))
        .collect();

    // Build entity and group id map — IDs assigned by DEFINITION order (source_line),
    // interleaved between entities and groups. Java assigns entity UIDs at parse time.
    let mut entity_ids: HashMap<String, String> = HashMap::new();
    let mut group_ids: HashMap<String, String> = HashMap::new();

    // Collect all entities and groups with their source lines for interleaved ordering
    enum IdSlot<'a> {
        Entity(&'a Entity),
        Group(&'a ClusterLayout),
    }
    let mut all_slots: Vec<(usize, IdSlot)> = Vec::new();
    for entity in &cd.entities {
        all_slots.push((
            entity.source_line.unwrap_or(usize::MAX),
            IdSlot::Entity(entity),
        ));
    }
    for cluster in &layout.clusters {
        let source_line = group_meta
            .get(cluster.qualified_name.as_str())
            .and_then(|group| group.source_line)
            .unwrap_or(usize::MAX);
        all_slots.push((source_line, IdSlot::Group(cluster)));
    }
    all_slots.sort_by_key(|(sl, _)| *sl);

    let mut ent_counter = 2u32; // Java starts entity IDs at ent0002
    for (_, slot) in &all_slots {
        match slot {
            IdSlot::Entity(entity) => {
                let ent_id = entity
                    .uid
                    .clone()
                    .unwrap_or_else(|| format!("ent{:04}", ent_counter));
                entity_ids.insert(sanitize_id(&entity.name), ent_id);
            }
            IdSlot::Group(cluster) => {
                let ent_id = group_meta
                    .get(cluster.qualified_name.as_str())
                    .and_then(|group| group.uid.clone())
                    .unwrap_or_else(|| format!("ent{:04}", ent_counter));
                group_ids.insert(cluster.qualified_name.clone(), ent_id);
            }
        }
        ent_counter += 1;
    }

    // Build sorted group list for rendering
    let mut groups_by_def_order: Vec<&ClusterLayout> = layout.clusters.iter().collect();
    groups_by_def_order.sort_by_key(|cluster| {
        (
            group_meta
                .get(cluster.qualified_name.as_str())
                .and_then(|group| group.source_line)
                .unwrap_or(usize::MAX),
            cluster.qualified_name.matches('.').count(),
            cluster.qualified_name.clone(),
        )
    });

    // Java: object diagrams do NOT emit <!--class X--> comments for entities,
    // only class diagrams do.
    let is_object_diagram = cd
        .entities
        .iter()
        .all(|e| matches!(e.kind, EntityKind::Object | EntityKind::Map));

    for cluster in &groups_by_def_order {
        let ent_id = group_ids
            .get(cluster.qualified_name.as_str())
            .map(|s| s.as_str())
            .unwrap_or("ent0000");
        let group = group_meta.get(cluster.qualified_name.as_str()).copied();
        draw_class_group(
            &mut sg,
            &mut tracker,
            cd,
            cluster,
            group,
            ent_id,
            skin,
            edge_offset_x,
            edge_offset_y,
        );
    }

    let mut entity_group_order: HashMap<&str, usize> = HashMap::new();
    let mut entity_qualified_names: HashMap<&str, String> = HashMap::new();
    for group in &cd.groups {
        let group_order = group.source_line.unwrap_or(usize::MAX);
        for entity_name in &group.entities {
            entity_group_order
                .entry(entity_name.as_str())
                .or_insert(group_order);
            entity_qualified_names
                .entry(entity_name.as_str())
                .or_insert_with(|| {
                    // If the entity name already starts with the group prefix
                    // (implicit package groups), don't prepend it again.
                    let prefix = format!("{}.", group.name);
                    if entity_name.starts_with(&prefix) {
                        entity_name.clone()
                    } else {
                        format!("{}.{}", group.name, entity_name)
                    }
                });
        }
    }
    // Build a definition-order index matching Java's entity creation order.
    // cd.entities is already sorted by sort_entities_by_order() in the parser,
    // which accounts for hide/show rules that implicitly reserve entity slots
    // before their explicit class declarations.
    let entity_def_order: HashMap<&str, usize> = cd
        .entities
        .iter()
        .enumerate()
        .map(|(i, e)| (e.name.as_str(), i))
        .collect();
    let mut entities_by_render_order: Vec<&Entity> = cd.entities.iter().collect();
    entities_by_render_order.sort_by_key(|entity| {
        (
            entity_group_order
                .get(entity.name.as_str())
                .copied()
                .unwrap_or(usize::MAX),
            entity_def_order
                .get(entity.name.as_str())
                .copied()
                .unwrap_or(usize::MAX),
        )
    });

    for entity in entities_by_render_order {
        let sid = sanitize_id(&entity.name);
        if let Some(nl) = node_map.get(sid.as_str()) {
            let ent_id = entity_ids
                .get(&sid)
                .map(|s| s.as_str())
                .unwrap_or("ent0000");
            let display_name = crate::layout::class_entity_display_name(&entity.name);
            if is_object_diagram {
                sg.push_raw(&format!(
                    "<g class=\"entity\" data-qualified-name=\"{}\"",
                    svg_group_metadata_attr(
                        entity_qualified_names
                            .get(entity.name.as_str())
                            .map(|s| s.as_str())
                            .unwrap_or(entity.name.as_str()),
                    ),
                ));
            } else {
                sg.push_raw(&format!(
                    "<!--{} {}--><g class=\"entity\" data-qualified-name=\"{}\"",
                    // Java uses "class" for class entities, "entity" for others (rectangle, component, etc.)
                    if matches!(entity.kind, EntityKind::Rectangle | EntityKind::Component) {
                        "entity"
                    } else {
                        "class"
                    },
                    svg_comment_escape(&display_name),
                    svg_group_metadata_attr(
                        entity_qualified_names
                            .get(entity.name.as_str())
                            .map(|s| s.as_str())
                            .unwrap_or(entity.name.as_str()),
                    ),
                ));
            }
            if let Some(source_line) = entity.source_line {
                sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
            }
            sg.push_raw(&format!(" id=\"{ent_id}\">"));
            draw_entity_box(
                &mut sg,
                &mut tracker,
                cd,
                entity,
                nl,
                skin,
                edge_offset_x,
                edge_offset_y,
            );
            sg.push_raw("</g>");
        }
    }

    let qualifier_placements =
        compute_qualifier_placements(cd, layout, edge_offset_x, edge_offset_y);
    let mut link_counter = ent_counter;
    for (link_idx, link) in cd.links.iter().enumerate() {
        let from_id = sanitize_id(&link.from);
        let to_id = sanitize_id(&link.to);
        if let Some(el) = layout
            .edges
            .get(link_idx)
            .filter(|e| e.from == from_id && e.to == to_id)
        {
            let from_ent = entity_ids.get(&from_id).map(|s| s.as_str()).unwrap_or("");
            let to_ent = entity_ids.get(&to_id).map(|s| s.as_str()).unwrap_or("");
            let link_type = derive_link_type(link);
            let from_display = crate::layout::class_entity_display_name(&link.from);
            let to_display = crate::layout::class_entity_display_name(&link.to);
            let comment_prefix = if link_looks_reverted_for_svg(link) {
                "reverse link"
            } else {
                "link"
            };
            sg.push_raw(&format!(
                "<!--{} {} to {}--><g class=\"link\" data-entity-1=\"{}\" data-entity-2=\"{}\" data-link-type=\"{}\"",
                comment_prefix,
                svg_comment_escape(&from_display),
                svg_comment_escape(&to_display),
                from_ent,
                to_ent,
                link_type,
            ));
            if let Some(source_line) = link.source_line {
                sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
            }
            let link_id = link
                .uid
                .clone()
                .unwrap_or_else(|| format!("lnk{link_counter}"));
            sg.push_raw(&format!(" id=\"{link_id}\">"));
            draw_edge(
                &mut sg,
                &mut tracker,
                layout,
                link,
                el,
                link_idx,
                &qualifier_placements,
                skin,
                arrow_color,
                edge_offset_x,
                edge_offset_y,
            );
            sg.push_raw("</g>");
            link_counter += 1;
        }
    }

    // Notes — Java wraps each note in <g class="entity" data-qualified-name="GMN{i}">
    // Java note IDs start after all entities: entity count + 1 (0-indexed quark offset)
    // Java quark numbering: entities are numbered from 2 (0=root, 1=diagram), notes after that
    let note_id_base = cd.entities.len() + cd.links.len() + 2;
    for (ni, note) in layout.notes.iter().enumerate() {
        let note_qname = format!("GMN{}", note_id_base + ni);
        sg.push_raw(&format!(
            "<g class=\"entity\" data-qualified-name=\"{note_qname}\" id=\"ent{:04}\">",
            cd.entities.len() + ni
        ));
        draw_class_note(&mut sg, &mut tracker, note, edge_offset_x, edge_offset_y);
        sg.push_raw("</g>");
    }

    // Stable Java now sizes cuca/svek diagrams from ImageBuilder.getFinalDimension():
    // it runs LimitFinder on the already moveDelta-shifted drawing, then adds the
    // document margins. The rendered max point therefore is the authority, not
    // lf_span + delta(15,15).
    let is_degenerated =
        layout.nodes.len() <= 1 && layout.edges.is_empty() && layout.notes.is_empty();
    let (max_x, max_y) = tracker.max_point();
    // raw_body_dim: the LimitFinder extent (no +1). Used by wrap_with_meta for
    // merge_tb — the global getFinalDimension +1 is applied at the canvas level.
    let raw_body_dim = if is_degenerated {
        if let Some(node) = layout.nodes.first() {
            const DEGENERATED_DELTA: f64 = 7.0;
            Some((
                node.width + DEGENERATED_DELTA * 2.0,
                node.height + DEGENERATED_DELTA * 2.0,
            ))
        } else {
            // Empty diagram: body draws nothing, no LimitFinder extent.
            Some((0.0, 0.0))
        }
    } else if max_x.is_finite() && max_y.is_finite() {
        Some((max_x, max_y))
    } else {
        None
    };
    // For the standalone body SVG viewport, add the getFinalDimension +1.
    let (svg_w, svg_h) = if let Some((raw_w, raw_h)) = raw_body_dim {
        compute_viewport(raw_w, raw_h, &ViewportConfig::SVEK)
    } else {
        // Keep the empty-diagram fallback non-zero.
        compute_viewport(10.0, 10.0, &ViewportConfig::COMPONENT)
    };

    let mut buf = String::with_capacity(sg.body().len() + 512);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, svg_w, svg_h, "CLASS", bg);
    let defs = sg.defs();
    if defs.is_empty() {
        buf.push_str("<defs/>");
    } else {
        buf.push_str("<defs>");
        buf.push_str(defs);
        buf.push_str("</defs>");
    }
    buf.push_str("<g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(BodyResult {
        svg: buf,
        raw_body_dim,
        body_pre_offset: class_body_offset.is_some(),
        body_degenerated: is_degenerated,
    })
}

fn draw_class_group(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    cd: &crate::model::ClassDiagram,
    cluster: &ClusterLayout,
    group: Option<&crate::model::Group>,
    ent_id: &str,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if cluster.width <= 0.0 || cluster.height <= 0.0 {
        return;
    }
    let group_kind = group.map(|g| &g.kind).unwrap_or(&GroupKind::Package);
    let qname = &cluster.qualified_name;
    let title = cluster.title.as_deref().unwrap_or(qname);
    sg.push_raw(&format!(
        "<!--cluster {}--><g class=\"cluster\" data-qualified-name=\"{}\"",
        svg_comment_escape(title),
        svg_group_metadata_attr(qname),
    ));
    if let Some(source_line) = group.and_then(|g| g.source_line) {
        sg.push_raw(&format!(" data-source-line=\"{source_line}\""));
    }
    sg.push_raw(&format!(" id=\"{ent_id}\">"));

    let x = cluster.x + edge_offset_x;
    let y = cluster.y + edge_offset_y;
    let w = cluster.width;
    let h = cluster.height;
    let group_header = group.map(|group| class_group_header_metrics(group, &cd.hide_show_rules));
    let visible_stereotypes = group_header
        .as_ref()
        .map(|metrics| metrics.visible_stereotypes.as_slice())
        .unwrap_or(&[]);
    let title_ascent = font_metrics::ascent("SansSerif", 14.0, true, false);
    let title_line_height = font_metrics::line_height("SansSerif", 14.0, true, false);
    let stereo_ascent = font_metrics::ascent("SansSerif", 14.0, false, true);
    let stereo_line_height = font_metrics::line_height("SansSerif", 14.0, false, true);

    match group_kind {
        GroupKind::Rectangle => {
            // Stereotype-keyed skinparams (C4 stdlib: rectangle<<system_boundary>>,
            // etc.) take precedence when the cluster carries stereotypes.
            let stereo_names: Vec<&str> = group
                .map(|g| g.stereotypes.iter().map(|s| s.0.as_str()).collect())
                .unwrap_or_default();
            let border = skin.border_color_for("rectangle", &stereo_names, "#181818");
            let font_color = skin.font_color_for("rectangle", &stereo_names, "#000000");
            let border_style = skin.border_style_for("rectangle", &stereo_names);
            let fill = class_group_fill_color(cd, group).unwrap_or_else(|| "none".to_string());
            // Map skinparam BorderStyle values to SVG stroke-dasharray (Java
            // LinkStyle): dashed -> "7,7", dotted -> "1,3", solid (or unset)
            // -> no dasharray.
            let dash_pattern = match border_style.map(str::to_ascii_lowercase).as_deref() {
                Some("dashed") => Some((7.0_f64, 7.0_f64)),
                Some("dotted") => Some((1.0_f64, 3.0_f64)),
                _ => None,
            };
            RectShape {
                x,
                y,
                w,
                h,
                rx: 2.5,
                ry: 2.5,
            }
            .draw(
                sg,
                &DrawStyle {
                    fill: Some(fill.clone()),
                    stroke: Some(border.to_string()),
                    stroke_width: 1.0,
                    dash_array: dash_pattern,
                    delta_shadow: 0.0,
                },
            );
            if dash_pattern.is_some() {
                sg.set_stroke_width(1.0, None);
            }
            tracker.track_rect(x, y, w, h);
            for (idx, label) in visible_stereotypes.iter().enumerate() {
                let stereo_text = format!("\u{00AB}{label}\u{00BB}");
                let stereo_w =
                    font_metrics::text_width(&stereo_text, "SansSerif", 14.0, false, true);
                let stereo_x = x + (w - stereo_w) / 2.0;
                let stereo_y = y + 2.0 + stereo_ascent + idx as f64 * stereo_line_height;
                sg.push_raw(&format!(
                    r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    fmt_coord(stereo_w),
                    fmt_coord(stereo_x),
                    fmt_coord(stereo_y),
                    xml_escape(&stereo_text),
                ));
                tracker.track_rect(
                    stereo_x,
                    stereo_y - stereo_ascent,
                    stereo_w,
                    stereo_line_height,
                );
            }
            // Strip Creole `==heading==` prefix if present (Java parses this).
            let display_title = crate::parser::creole::strip_heading_prefix(title).unwrap_or(title);
            let text_w = font_metrics::text_width(display_title, "SansSerif", 14.0, true, false);
            let text_x = x + (w - text_w) / 2.0;
            let text_y =
                y + 2.0 + visible_stereotypes.len() as f64 * stereo_line_height + title_ascent;
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(text_w),
                fmt_coord(text_x),
                fmt_coord(text_y),
                xml_escape(display_title),
            ));
            tracker.track_rect(
                text_x,
                text_y - HEADER_NAME_BASELINE,
                text_w,
                HEADER_NAME_BLOCK_HEIGHT,
            );
        }
        _ => {
            let border = skin.border_color("package", "#000000");
            let font_color = skin.font_color("package", "#000000");
            let text_w = font_metrics::text_width(title, "SansSerif", 14.0, true, false);
            let r = 2.5_f64;
            let tab_bottom = y + 22.2969;
            let tab_right = (x + w - r).min(x + text_w + 13.0);
            let tab_notch = (tab_right - 9.5).max(x + r);
            let tab_arc_end_x = (tab_right - 7.0).max(tab_notch);
            sg.push_raw(&format!(
                concat!(
                    r#"<path d="M{},{} L{},{}"#,
                    r#" A3.75,3.75 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}"#,
                    r#" L{},{}"#,
                    r#" A{},{} 0 0 1 {},{}" fill="none" style="stroke:{};stroke-width:1.5;"/>"#
                ),
                fmt_coord(x + r),
                fmt_coord(y),
                fmt_coord(tab_notch),
                fmt_coord(y),
                fmt_coord(tab_arc_end_x),
                fmt_coord(y + r),
                fmt_coord(tab_right),
                fmt_coord(tab_bottom),
                fmt_coord(x + w - r),
                fmt_coord(tab_bottom),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x + w),
                fmt_coord(tab_bottom + r),
                fmt_coord(x + w),
                fmt_coord(y + h - r),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x + w - r),
                fmt_coord(y + h),
                fmt_coord(x + r),
                fmt_coord(y + h),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x),
                fmt_coord(y + h - r),
                fmt_coord(x),
                fmt_coord(y + r),
                fmt_coord(r),
                fmt_coord(r),
                fmt_coord(x + r),
                fmt_coord(y),
                border,
            ));
            sg.push_raw(&format!(
                r#"<line style="stroke:{border};stroke-width:1.5;" x1="{}" x2="{}" y1="{}" y2="{}"/>"#,
                fmt_coord(x),
                fmt_coord(tab_right),
                fmt_coord(tab_bottom),
                fmt_coord(tab_bottom),
            ));
            let text_x = x + 4.0;
            let text_y = y + 2.0 + title_ascent;
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(text_w),
                fmt_coord(text_x),
                fmt_coord(text_y),
                xml_escape(title),
            ));
            let title_h = if text_w == 0.0 {
                10.0
            } else {
                title_line_height + 6.0
            };
            for (idx, label) in visible_stereotypes.iter().enumerate() {
                let stereo_text = format!("\u{00AB}{label}\u{00BB}");
                let stereo_w =
                    font_metrics::text_width(&stereo_text, "SansSerif", 14.0, false, true);
                let stereo_x = x + 4.0 + (w - stereo_w) / 2.0;
                let stereo_y = y + 2.0 + title_h + stereo_ascent + idx as f64 * stereo_line_height;
                sg.push_raw(&format!(
                    r#"<text fill="{font_color}" font-family="sans-serif" font-size="14" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                    fmt_coord(stereo_w),
                    fmt_coord(stereo_x),
                    fmt_coord(stereo_y),
                    xml_escape(&stereo_text),
                ));
                tracker.track_rect(
                    stereo_x,
                    stereo_y - stereo_ascent,
                    stereo_w,
                    stereo_line_height,
                );
            }
            tracker.track_path_bounds(x, y, x + w, y + h);
            tracker.track_line(x, tab_bottom, tab_right, tab_bottom);
            tracker.track_rect(
                text_x,
                text_y - HEADER_NAME_BASELINE,
                text_w,
                HEADER_NAME_BLOCK_HEIGHT,
            );
        }
    }

    sg.push_raw("</g>");
}

// ── Stereotype circle glyph paths ───────────────────────────────────
// Raw glyph outline coordinates from Java AWT TextLayout.getOutline().
// Font: Monospaced Bold 17pt (PlantUML FontParam.CIRCLED_CHARACTER).
// Coordinates are relative to the text draw position (0, 0).
//
// UnusedSpace center offsets from PlantUML's UnusedSpace algorithm,
// extracted via Java instrumentation on the reference generation machine.
//
// At render time:
//   offset_x = circle_abs_cx - CENTER_X - 0.5
//   offset_y = circle_abs_cy - CENTER_Y - 0.5
//   final_coord = raw_coord + offset

// UnusedSpace centers from PlantUML's actual runtime values.
// Extracted via Java instrumentation: char='X' centerX=... centerY=...
// These depend on font rendering and MUST match the reference generation machine.
const GLYPH_C_CENTER: (f64, f64) = (5.5, -6.5);
const GLYPH_I_CENTER: (f64, f64) = (5.0, -6.5);
const GLYPH_E_CENTER: (f64, f64) = (4.5, -6.5);
const GLYPH_A_CENTER: (f64, f64) = (4.5, -6.0);

// Raw glyph path segments from Java AWT TextLayout.getOutline().
// Coordinates at full f64 precision (all are exact binary fractions from TrueType hinting).
const GLYPH_C_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(8.96875, -0.359375)]),
    ('Q', &[(8.390625, -0.0625), (7.75, 0.078125)]),
    ('Q', &[(7.109375, 0.234375), (6.40625, 0.234375)]),
    ('Q', &[(3.90625, 0.234375), (2.578125, -1.40625)]),
    ('Q', &[(1.265625, -3.0625), (1.265625, -6.1875)]),
    ('Q', &[(1.265625, -9.3125), (2.578125, -10.96875)]),
    ('Q', &[(3.90625, -12.625), (6.40625, -12.625)]),
    ('Q', &[(7.109375, -12.625), (7.75, -12.46875)]),
    ('Q', &[(8.40625, -12.3125), (8.96875, -12.015625)]),
    ('L', &[(8.96875, -9.296875)]),
    ('Q', &[(8.34375, -9.875), (7.75, -10.140625)]),
    ('Q', &[(7.15625, -10.421875), (6.53125, -10.421875)]),
    ('Q', &[(5.1875, -10.421875), (4.5, -9.34375)]),
    ('Q', &[(3.8125, -8.28125), (3.8125, -6.1875)]),
    ('Q', &[(3.8125, -4.09375), (4.5, -3.015625)]),
    ('Q', &[(5.1875, -1.953125), (6.53125, -1.953125)]),
    ('Q', &[(7.15625, -1.953125), (7.75, -2.21875)]),
    ('Q', &[(8.34375, -2.5), (8.96875, -3.078125)]),
    ('L', &[(8.96875, -0.359375)]),
    ('Z', &[]),
];

const GLYPH_I_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(1.421875, -10.234375)]),
    ('L', &[(1.421875, -12.390625)]),
    ('L', &[(8.8125, -12.390625)]),
    ('L', &[(8.8125, -10.234375)]),
    ('L', &[(6.34375, -10.234375)]),
    ('L', &[(6.34375, -2.15625)]),
    ('L', &[(8.8125, -2.15625)]),
    ('L', &[(8.8125, 0.0)]),
    ('L', &[(1.421875, 0.0)]),
    ('L', &[(1.421875, -2.15625)]),
    ('L', &[(3.890625, -2.15625)]),
    ('L', &[(3.890625, -10.234375)]),
    ('L', &[(1.421875, -10.234375)]),
    ('Z', &[]),
];

const GLYPH_E_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(9.109375, 0.0)]),
    ('L', &[(1.390625, 0.0)]),
    ('L', &[(1.390625, -12.390625)]),
    ('L', &[(9.109375, -12.390625)]),
    ('L', &[(9.109375, -10.234375)]),
    ('L', &[(3.84375, -10.234375)]),
    ('L', &[(3.84375, -7.5625)]),
    ('L', &[(8.609375, -7.5625)]),
    ('L', &[(8.609375, -5.40625)]),
    ('L', &[(3.84375, -5.40625)]),
    ('L', &[(3.84375, -2.15625)]),
    ('L', &[(9.109375, -2.15625)]),
    ('L', &[(9.109375, 0.0)]),
    ('Z', &[]),
];

const GLYPH_A_RAW: &[(char, &[(f64, f64)])] = &[
    ('M', &[(5.109375, -10.15625)]),
    ('L', &[(3.953125, -5.078125)]),
    ('L', &[(6.28125, -5.078125)]),
    ('L', &[(5.109375, -10.15625)]),
    ('Z', &[]),
    ('M', &[(3.625, -12.390625)]),
    ('L', &[(6.609375, -12.390625)]),
    ('L', &[(9.96875, 0.0)]),
    ('L', &[(7.515625, 0.0)]),
    ('L', &[(6.75, -3.0625)]),
    ('L', &[(3.46875, -3.0625)]),
    ('L', &[(2.71875, 0.0)]),
    ('L', &[(0.28125, 0.0)]),
    ('L', &[(3.625, -12.390625)]),
    ('Z', &[]),
];

/// Emit a stereotype circle glyph path element.
/// `circle_cx` and `circle_cy` are the absolute SVG coordinates of the circle center.
/// If `spot_char` is Some, use that character's glyph instead of the entity-kind default.
#[allow(dead_code)] // reserved for circle glyph rendering
fn emit_circle_glyph(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    kind: &EntityKind,
    circle_cx: f64,
    circle_cy: f64,
) {
    emit_circle_glyph_with_char(sg, tracker, kind, circle_cx, circle_cy, None);
}

fn emit_circle_glyph_with_char(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    kind: &EntityKind,
    circle_cx: f64,
    circle_cy: f64,
    spot_char: Option<char>,
) {
    let (glyph_raw, center) = if let Some(ch) = spot_char {
        match ch.to_ascii_uppercase() {
            'C' => (GLYPH_C_RAW, GLYPH_C_CENTER),
            'A' => (GLYPH_A_RAW, GLYPH_A_CENTER),
            'I' => (GLYPH_I_RAW, GLYPH_I_CENTER),
            'E' => (GLYPH_E_RAW, GLYPH_E_CENTER),
            _ => {
                // For characters we don't have pre-rendered glyphs for,
                // fall back to entity kind default
                match kind {
                    EntityKind::Class | EntityKind::Object => (GLYPH_C_RAW, GLYPH_C_CENTER),
                    EntityKind::Abstract => (GLYPH_A_RAW, GLYPH_A_CENTER),
                    EntityKind::Interface => (GLYPH_I_RAW, GLYPH_I_CENTER),
                    EntityKind::Enum => (GLYPH_E_RAW, GLYPH_E_CENTER),
                    EntityKind::Annotation
                    | EntityKind::Rectangle
                    | EntityKind::Component
                    | EntityKind::Map => return,
                }
            }
        }
    } else {
        match kind {
            EntityKind::Class | EntityKind::Object => (GLYPH_C_RAW, GLYPH_C_CENTER),
            EntityKind::Abstract => (GLYPH_A_RAW, GLYPH_A_CENTER),
            EntityKind::Interface => (GLYPH_I_RAW, GLYPH_I_CENTER),
            EntityKind::Enum => (GLYPH_E_RAW, GLYPH_E_CENTER),
            EntityKind::Annotation
            | EntityKind::Rectangle
            | EntityKind::Component
            | EntityKind::Map => return,
        }
    };

    // Java DriverCenteredCharacterSvg algorithm:
    //   xpos = circle_center_in_ug - centerX - 0.5
    //   ypos = circle_center_in_ug - centerY - 0.5
    //   final = path_coord + (xpos, ypos)
    let dx = circle_cx - center.0 - 0.5;
    let dy = circle_cy - center.1 - 0.5;

    let mut d = String::with_capacity(512);
    let mut path_min_x = f64::INFINITY;
    let mut path_min_y = f64::INFINITY;
    let mut path_max_x = f64::NEG_INFINITY;
    let mut path_max_y = f64::NEG_INFINITY;
    for (cmd, points) in glyph_raw {
        d.push(*cmd);
        for (i, &(px, py)) in points.iter().enumerate() {
            if i > 0 {
                d.push(' ');
            }
            let final_x = px + dx;
            let final_y = py + dy;
            d.push_str(&fmt_coord(final_x));
            d.push(',');
            d.push_str(&fmt_coord(final_y));
            if final_x < path_min_x {
                path_min_x = final_x;
            }
            if final_y < path_min_y {
                path_min_y = final_y;
            }
            if final_x > path_max_x {
                path_max_x = final_x;
            }
            if final_y > path_max_y {
                path_max_y = final_y;
            }
        }
        // Java SvgGraphics: every command (including Z) has a trailing space
        d.push(' ');
    }

    sg.push_raw(&format!(r##"<path d="{d}" fill="#000000"/>"##));
    if path_min_x.is_finite() {
        tracker.track_path_bounds(path_min_x, path_min_y, path_max_x, path_max_y);
    }
}

/// Offset all coordinates in a glyph path string by (dx, dy).
/// The path uses M, Q, L, Z commands with absolute coordinates.
/// Format: "Mx,y Qx,y x,y Lx,y Z"
#[allow(dead_code)] // reserved for glyph rendering
fn offset_glyph_path_xy(path: &str, dx: f64, dy: f64) -> String {
    if dx == 0.0 && dy == 0.0 {
        return path.to_string();
    }
    let mut result = String::with_capacity(path.len() + 64);
    let mut chars = path.chars().peekable();
    let mut is_x = true; // alternates: first number is X, second is Y

    while let Some(&c) = chars.peek() {
        match c {
            'M' | 'Q' | 'L' | 'C' | 'Z' => {
                result.push(c);
                chars.next();
                is_x = true; // reset after command
            }
            '-' | '0'..='9' | '.' => {
                // Parse number
                let mut s = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_ascii_digit() || nc == '.' || nc == '-' {
                        s.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(val) = s.parse::<f64>() {
                    if is_x {
                        result.push_str(&fmt_coord(val + dx));
                    } else {
                        result.push_str(&fmt_coord(val + dy));
                    }
                    is_x = !is_x;
                } else {
                    result.push_str(&s);
                }
            }
            ',' => {
                result.push(',');
                chars.next();
            }
            ' ' => {
                result.push(' ');
                chars.next();
            }
            _ => {
                result.push(c);
                chars.next();
            }
        }
    }
    result
}

fn stereotype_circle_color(kind: &EntityKind) -> &'static str {
    match kind {
        EntityKind::Class => "#ADD1B2",
        EntityKind::Interface => "#B4A7E5",
        EntityKind::Enum => "#EB937F",
        EntityKind::Abstract => "#A9DCDF",
        EntityKind::Annotation => "#A9DCDF",
        EntityKind::Object | EntityKind::Map => "#ADD1B2",
        EntityKind::Rectangle => "#F1F1F1",
        EntityKind::Component => "#F1F1F1",
    }
}

fn draw_entity_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    cd: &ClassDiagram,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if entity.kind == EntityKind::Object || entity.kind == EntityKind::Map {
        draw_object_box(sg, tracker, entity, nl, skin, edge_offset_x, edge_offset_y);
        return;
    }

    if entity.kind == EntityKind::Rectangle {
        draw_rectangle_entity_box(sg, tracker, entity, nl, skin, edge_offset_x, edge_offset_y);
        return;
    }

    if entity.kind == EntityKind::Component {
        draw_component_description_box(
            sg,
            tracker,
            cd,
            entity,
            nl,
            skin,
            edge_offset_x,
            edge_offset_y,
        );
        return;
    }

    // Java: after `layout_with_svek()` re-normalizes to origin, class entities
    // render back at the plain Svek margin offset (= 6).
    let x = nl.cx - nl.width / 2.0 + edge_offset_x;
    let y = nl.cy - nl.height / 2.0 + edge_offset_y;
    let w = nl.width;
    let h = nl.height;

    let (default_bg, default_border, element_type) = match entity.kind {
        EntityKind::Class => (ENTITY_BG, BORDER_COLOR, "class"),
        EntityKind::Interface => (ENTITY_BG, BORDER_COLOR, "interface"),
        EntityKind::Enum => (ENTITY_BG, BORDER_COLOR, "enum"),
        EntityKind::Abstract => (ENTITY_BG, BORDER_COLOR, "abstract"),
        EntityKind::Annotation => (ENTITY_BG, BORDER_COLOR, "annotation"),
        EntityKind::Rectangle => (ENTITY_BG, BORDER_COLOR, "rectangle"),
        EntityKind::Component => (ENTITY_BG, BORDER_COLOR, "component"),
        EntityKind::Object | EntityKind::Map => unreachable!(),
    };
    let default_fill = skin.background_color(element_type, default_bg);
    let fill = entity
        .color
        .as_deref()
        .map(crate::style::normalize_color)
        .or_else(|| class_stereotype_fill_color(&cd.stereotype_backgrounds, &entity.stereotypes))
        .unwrap_or_else(|| default_fill.to_string());
    let stroke = skin.border_color(element_type, default_border);
    let font_color = skin.font_color(element_type, TEXT_COLOR);

    // Java URectangle.rounded(roundCorner): rx = roundCorner / 2.
    // Default roundCorner from style = 5 → rx = 2.5.
    // Java URectangle.rounded(roundCorner): SVG rx = roundCorner / 2.
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    // Java font resolution:
    // - classFontSize controls the class name font size
    // - classAttributeFontSize controls member (field/method) font size
    // When classAttributeFontSize is set, it overrides classFontSize for both
    // header name and attributes (matching Java style priority).
    let explicit_attr_fs = skin
        .get("classattributefontsize")
        .and_then(|s| s.parse::<f64>().ok());
    let explicit_class_fs = skin
        .get("classfontsize")
        .and_then(|s| s.parse::<f64>().ok());
    let attr_font_size = explicit_attr_fs.unwrap_or_else(|| explicit_class_fs.unwrap_or(FONT_SIZE));
    let class_font_size =
        explicit_attr_fs.unwrap_or_else(|| explicit_class_fs.unwrap_or(FONT_SIZE));

    // Entity name WITHOUT generic parameter — generic is rendered separately in draw_generic_box
    // When `as Alias` is used, display_name holds the original quoted label.
    let name_display_raw = entity
        .display_name
        .as_deref()
        .map(String::from)
        .unwrap_or_else(|| crate::layout::class_entity_display_name(&entity.name));
    // Strip HTML markup tags (<b>, <i>, etc.) — Java interprets these as formatting.
    let markup_info = crate::layout::strip_html_markup(&name_display_raw);
    let name_display = if markup_info.bold || markup_info.italic {
        markup_info.text.clone()
    } else {
        name_display_raw
    };
    let name_markup_bold = markup_info.bold;
    let visible_stereotypes = visible_stereotype_labels(&cd.hide_show_rules, entity);
    let raw_field_count = entity.members.iter().filter(|m| !m.is_method).count();
    let raw_method_count = entity.members.iter().filter(|m| m.is_method).count();
    let show_fields = show_portion(
        &cd.hide_show_rules,
        ClassPortion::Field,
        &entity.name,
        raw_field_count,
    );
    let show_methods = show_portion(
        &cd.hide_show_rules,
        ClassPortion::Method,
        &entity.name,
        raw_method_count,
    );
    let visible_fields: Vec<&Member> = entity
        .members
        .iter()
        .filter(|m| !m.is_method)
        .filter(|_| show_fields)
        .collect();
    let visible_methods: Vec<&Member> = entity
        .members
        .iter()
        .filter(|m| m.is_method)
        .filter(|_| show_methods)
        .collect();
    let has_kind_label = matches!(entity.kind, EntityKind::Enum | EntityKind::Annotation);
    let italic_name =
        markup_info.italic || matches!(entity.kind, EntityKind::Abstract | EntityKind::Interface);

    // Compute header height early (needed for gradient header rects).
    // This duplicates the logic at the separator-line section below.
    let header_height_early = if has_kind_label {
        HEADER_HEIGHT
    } else {
        let n_lines = crate::layout::split_name_display(&name_display).lines.len();
        let single_h = font_metrics::ascent("SansSerif", class_font_size, false, italic_name)
            + font_metrics::descent("SansSerif", class_font_size, false, italic_name);
        let dynamic_name_h = n_lines as f64 * single_h;
        HEADER_CIRCLE_BLOCK_HEIGHT.max(
            visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT
                + dynamic_name_h
                + HEADER_STEREO_NAME_GAP,
        )
    };

    // Java EntityImageClass.drawInternal: when the background is a gradient
    // and roundCorner != 0, Java draws 4 layered rects:
    // 1. Body rect with gradient fill + border stroke
    // 2. Header rect with gradient fill + gradient stroke (rounded)
    // 3. Separator rect with gradient fill + gradient stroke (no rounding)
    // 4. Outline rect with no fill + border stroke
    // When the background is a plain color, a single rect suffices.
    let is_gradient = crate::klimt::color::resolve_color(&fill)
        .map(|c| c.is_gradient())
        .unwrap_or(false);

    // Rect 1: body rect with fill + border stroke
    RectShape {
        x,
        y,
        w,
        h,
        rx,
        ry: rx,
    }
    .draw(sg, &DrawStyle::filled(&fill, stroke, 0.5));

    if is_gradient {
        // Rect 2: header rect with gradient fill + gradient stroke (rounded)
        RectShape {
            x,
            y,
            w,
            h: header_height_early,
            rx,
            ry: rx,
        }
        .draw(sg, &DrawStyle::filled(&fill, &fill, 0.5));

        // Rect 3: separator rect (height = rx, positioned at header bottom - rx)
        RectShape {
            x,
            y: y + header_height_early - rx,
            w,
            h: rx,
            rx: 0.0,
            ry: 0.0,
        }
        .draw(sg, &DrawStyle::filled(&fill, &fill, 0.5));

        // Rect 4: outline rect with no fill + border stroke
        RectShape {
            x,
            y,
            w,
            h,
            rx,
            ry: rx,
        }
        .draw(sg, &DrawStyle::filled("none", stroke, 0.5));
    }

    tracker.track_rect(x, y, w, h);
    // Java entity image wrapper draws UEmpty(imageDim) at translate position,
    // which LimitFinder tracks with addPoint(x+w, y+h) — NO -1 adjustment.
    // This pushes max_y 1px beyond what the URectangle alone contributes.
    // Use image_width (not expanded DOT width) to match Java's calculateDimension.
    tracker.track_empty(x, y, nl.image_width, h);

    if has_kind_label {
        let kind_text = match entity.kind {
            EntityKind::Interface => "\u{00AB}interface\u{00BB}",
            EntityKind::Enum => "\u{00AB}enumeration\u{00BB}",
            EntityKind::Annotation => "\u{00AB}annotation\u{00BB}",
            _ => "",
        };
        let kind_y = y + HEADER_HEIGHT * 0.38;
        let name_y = y + HEADER_HEIGHT * 0.82;
        let cx = x + w / 2.0;
        let kind_fs = class_font_size - 2.0;
        let kind_tl_val = font_metrics::text_width(kind_text, "SansSerif", kind_fs, false, true);
        {
            let kind_tl = fmt_coord(kind_tl_val);
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="{fs:.0}" font-style="italic" lengthAdjust="spacing" text-anchor="middle" textLength="{kind_tl}" x="{}" y="{}">{kind_text}</text>"#,
                fmt_coord(cx), fmt_coord(kind_y), fs = kind_fs,
            ));
        }
        {
            let kind_ascent = font_metrics::ascent("SansSerif", kind_fs, false, true);
            let kind_descent = font_metrics::descent("SansSerif", kind_fs, false, true);
            tracker.track_rect(
                cx,
                kind_y - kind_ascent,
                kind_tl_val,
                kind_ascent + kind_descent,
            );
        }
        let name_tl_val =
            font_metrics::text_width(&name_display, "SansSerif", class_font_size, true, false);
        {
            let name_tl = fmt_coord(name_tl_val);
            let name_escaped = xml_escape(&name_display);
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="{class_font_size:.0}" font-weight="bold" lengthAdjust="spacing" text-anchor="middle" textLength="{name_tl}" x="{}" y="{}">{name_escaped}</text>"#,
                fmt_coord(cx), fmt_coord(name_y),
            ));
        }
        {
            let name_ascent = font_metrics::ascent("SansSerif", class_font_size, true, false);
            let name_descent = font_metrics::descent("SansSerif", class_font_size, true, false);
            tracker.track_rect(
                cx,
                name_y - name_ascent,
                name_tl_val,
                name_ascent + name_descent,
            );
        }
    } else {
        let name_block = crate::layout::split_name_display(&name_display);
        let n_name_lines = name_block.lines.len();
        let name_line_metrics: Vec<(f64, f64)> = name_block
            .lines
            .iter()
            .map(|line| {
                crate::layout::display_line_metrics(
                    line,
                    class_font_size,
                    name_markup_bold,
                    italic_name,
                )
            })
            .collect();
        let name_width = name_line_metrics
            .iter()
            .map(|(visible_width, indent_width)| visible_width + indent_width)
            .fold(0.0_f64, f64::max);
        // Compute name block height and baseline dynamically from actual font size
        let name_ascent =
            font_metrics::ascent("SansSerif", class_font_size, name_markup_bold, italic_name);
        let name_descent =
            font_metrics::descent("SansSerif", class_font_size, name_markup_bold, italic_name);
        let single_line_height = name_ascent + name_descent;
        let name_block_height = n_name_lines as f64 * single_line_height;
        let name_baseline = name_ascent;
        let name_block_width = name_width + HEADER_NAME_BLOCK_MARGIN_X;
        let stereo_widths: Vec<f64> = visible_stereotypes
            .iter()
            .map(|label| {
                font_metrics::text_width(
                    &format!("\u{00AB}{label}\u{00BB}"),
                    "SansSerif",
                    HEADER_STEREO_FONT_SIZE,
                    false,
                    true,
                )
            })
            .collect();
        let stereo_block_width =
            stereo_widths.iter().copied().fold(0.0_f64, f64::max) + HEADER_STEREO_BLOCK_MARGIN;
        let width_stereo_and_name = name_block_width.max(stereo_block_width);
        let stereo_height = visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT;
        let header_height = HEADER_CIRCLE_BLOCK_HEIGHT
            .max(stereo_height + name_block_height + HEADER_STEREO_NAME_GAP);
        let vis_icon_w = if entity.visibility.is_some() {
            ENTITY_VIS_ICON_BLOCK_SIZE
        } else {
            0.0
        };
        let gen_dim_w = if let Some(ref g) = entity.generic {
            let text_w = font_metrics::text_width(g, "SansSerif", GENERIC_FONT_SIZE, false, true);
            text_w + 2.0 * GENERIC_INNER_MARGIN + 2.0 * GENERIC_OUTER_MARGIN
        } else {
            0.0
        };
        let supp_width =
            (w - HEADER_CIRCLE_BLOCK_WIDTH - vis_icon_w - width_stereo_and_name - gen_dim_w)
                .max(0.0);
        let h2 = (HEADER_CIRCLE_BLOCK_WIDTH / 4.0).min(supp_width * 0.1);
        let h1 = (supp_width - h2) / 2.0;

        let spot = extract_entity_spot(entity);
        let circle_color = if let Some(ref sp) = spot {
            if let Some(ref c) = sp.color {
                crate::style::normalize_color(c)
            } else {
                stereotype_circle_color(&entity.kind).to_string()
            }
        } else {
            stereotype_circle_color(&entity.kind).to_string()
        };
        let circle_block_x = x + h1;
        let ecx = circle_block_x + 15.0;
        let ecy = y + header_height / 2.0;
        EllipseShape {
            cx: ecx,
            cy: ecy,
            rx: 11.0,
            ry: 11.0,
        }
        .draw(sg, &DrawStyle::filled(&circle_color, "#181818", 1.0));
        tracker.track_ellipse(ecx, ecy, 11.0, 11.0);
        emit_circle_glyph_with_char(
            sg,
            tracker,
            &entity.kind,
            ecx,
            ecy,
            spot.as_ref().map(|s| s.character),
        );

        let header_top_offset = (header_height - stereo_height - name_block_height) / 2.0;
        let name_block_x = x
            + HEADER_CIRCLE_BLOCK_WIDTH
            + vis_icon_w
            + (width_stereo_and_name - name_block_width) / 2.0
            + h1
            + h2;
        let name_inner_x = name_block_x + 3.0;

        if let Some(ref vis) = entity.visibility {
            let icon_x = name_inner_x - ENTITY_VIS_ICON_BLOCK_SIZE;
            // Java: EntityImageClassHeader wraps visibility UBlock with
            // withMargin(top=4), then mergeLR(uBlock, name, CENTER).
            // uBlock dim = (11, 11), with margin: (11, 15).
            // name dim height = HEADER_NAME_BLOCK_HEIGHT (≈16.3).
            // merged height = max(15, name_h).
            // icon in merged: (merged_h - 15) / 2 + 4 (margin top).
            // merged in header: (header_h - merged_h) / 2.
            let icon_margin_top = 0.0;
            let icon_block_h = ENTITY_VIS_ICON_BLOCK_SIZE + icon_margin_top;
            let merged_h = name_block_height.max(icon_block_h);
            let merged_y = (header_height - stereo_height - merged_h) / 2.0;
            let icon_in_merged = (merged_h - icon_block_h) / 2.0 + icon_margin_top;
            let icon_y = y + merged_y + icon_in_merged;
            draw_visibility_icon(sg, tracker, vis, true, icon_x, icon_y);
        }

        for (idx, label) in visible_stereotypes.iter().enumerate() {
            let stereo_text = format!("\u{00AB}{label}\u{00BB}");
            let stereo_x = x
                + HEADER_CIRCLE_BLOCK_WIDTH
                + vis_icon_w
                + (width_stereo_and_name - stereo_widths[idx]) / 2.0
                + h1
                + h2;
            let stereo_y = y
                + header_top_offset
                + HEADER_STEREO_BASELINE
                + idx as f64 * HEADER_STEREO_LINE_HEIGHT;
            sg.push_raw(&format!(
                r#"<text fill="{font_color}" font-family="sans-serif" font-size="12" font-style="italic" lengthAdjust="spacing" textLength="{}" x="{}" y="{}">{}</text>"#,
                fmt_coord(stereo_widths[idx]),
                fmt_coord(stereo_x),
                fmt_coord(stereo_y),
                xml_escape(&stereo_text),
            ));
            tracker.track_rect(
                stereo_x,
                stereo_y - HEADER_STEREO_BASELINE,
                stereo_widths[idx],
                HEADER_STEREO_LINE_HEIGHT,
            );
        }

        let font_style = if italic_name { Some("italic") } else { None };
        let font_weight = if name_markup_bold { Some("bold") } else { None };
        sg.set_fill_color(font_color);
        // Render each name line as a separate <text> element
        for (line_idx, line) in name_block.lines.iter().enumerate() {
            let display_line = if line.text.is_empty() {
                "\u{00A0}".to_string()
            } else {
                line.text.clone()
            };
            let line_y = y
                + header_top_offset
                + stereo_height
                + name_baseline
                + line_idx as f64 * single_line_height;
            let (visible_width, indent_width) = name_line_metrics[line_idx];
            let measured_width = visible_width + indent_width;
            let align_offset = match name_block.alignment {
                crate::layout::DisplayAlignment::Left => 0.0,
                crate::layout::DisplayAlignment::Center => (name_width - measured_width) / 2.0,
                crate::layout::DisplayAlignment::Right => name_width - measured_width,
            };
            let line_x = name_inner_x + align_offset + indent_width;
            sg.svg_text(
                &display_line,
                line_x,
                line_y,
                Some("sans-serif"),
                class_font_size,
                font_weight,
                font_style,
                None,
                visible_width,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
            tracker.track_rect(
                line_x,
                line_y - name_baseline,
                visible_width,
                single_line_height,
            );
        }
    }

    // Draw generic type box at top-right corner of entity rect
    if let Some(ref generic_text) = entity.generic {
        draw_generic_box(sg, tracker, generic_text, x, y, w);
    }

    let x1_val = fmt_coord(x + 1.0);
    let x2_val = fmt_coord(x + w - 1.0);
    let header_height = if has_kind_label {
        HEADER_HEIGHT
    } else {
        let n_lines = crate::layout::split_name_display(&name_display).lines.len();
        let single_h = font_metrics::ascent("SansSerif", class_font_size, false, italic_name)
            + font_metrics::descent("SansSerif", class_font_size, false, italic_name);
        let dynamic_name_h = n_lines as f64 * single_h;
        HEADER_CIRCLE_BLOCK_HEIGHT.max(
            visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT
                + dynamic_name_h
                + HEADER_STEREO_NAME_GAP,
        )
    };
    let mut section_y = y + header_height;
    // Java: member text uses classAttributeFontColor (defaults to TEXT_COLOR, not classFontColor)
    let attr_font_color = skin.font_color("classattribute", TEXT_COLOR);
    if show_fields {
        draw_member_section(
            sg,
            tracker,
            &visible_fields,
            section_y,
            x,
            &x1_val,
            &x2_val,
            attr_font_color,
            attr_font_size,
            stroke,
        );
        section_y += section_height_with_fs(&visible_fields, attr_font_size);
    }
    if show_methods {
        draw_member_section(
            sg,
            tracker,
            &visible_methods,
            section_y,
            x,
            &x1_val,
            &x2_val,
            attr_font_color,
            attr_font_size,
            stroke,
        );
    }
    // Java's LimitFinder first-pass sees separator lines at entity IMAGE width
    // (ULine(imageWidth,0) at entity translate). For non-expanded nodes where
    // imageWidth == nodeWidth, this adds max_x = x + imageWidth (1px beyond
    // the rect's x + nodeWidth - 1). For Graphviz-expanded nodes (qualifiers),
    // imageWidth < nodeWidth and the line doesn't extend beyond the rect.
    // Use image_width from the layout to match Java's LimitFinder.
    tracker.track_line(x, y, x + nl.image_width, y);
}

/// Draw the generic type box (dashed rect + italic text) at top-right of entity.
fn draw_generic_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    generic_text: &str,
    entity_x: f64,
    entity_y: f64,
    entity_w: f64,
) {
    let text_w =
        font_metrics::text_width(generic_text, "SansSerif", GENERIC_FONT_SIZE, false, true);
    let rect_w = text_w + 2.0 * GENERIC_INNER_MARGIN;
    let rect_h = GENERIC_TEXT_HEIGHT + 2.0 * GENERIC_INNER_MARGIN;
    let gen_dim_w = rect_w + 2.0 * GENERIC_OUTER_MARGIN;
    let gen_dim_h = rect_h + 2.0 * GENERIC_OUTER_MARGIN;

    // Outer block position: HeaderLayout.java:112
    let x_generic = entity_x + entity_w - gen_dim_w + GENERIC_DELTA;
    let y_generic = entity_y - GENERIC_DELTA;

    // Track outer margin wrapper UEmpty (Java withMargin draws UEmpty)
    tracker.track_empty(x_generic, y_generic, gen_dim_w, gen_dim_h);

    let rect_x = x_generic + GENERIC_OUTER_MARGIN;
    let rect_y = y_generic + GENERIC_OUTER_MARGIN;

    RectShape {
        x: rect_x,
        y: rect_y,
        w: rect_w,
        h: rect_h,
        rx: 0.0,
        ry: 0.0,
    }
    .draw(
        sg,
        &DrawStyle {
            fill: Some("#FFFFFF".into()),
            stroke: Some("#181818".into()),
            stroke_width: 1.0,
            dash_array: Some((2.0, 2.0)),
            delta_shadow: 0.0,
        },
    );
    tracker.track_rect(rect_x, rect_y, rect_w, rect_h);

    let text_x = rect_x + GENERIC_INNER_MARGIN;
    let text_y = rect_y + GENERIC_INNER_MARGIN + GENERIC_BASELINE;
    sg.set_fill_color("#000000");
    sg.svg_text(
        generic_text,
        text_x,
        text_y,
        Some("sans-serif"),
        12.0,
        None,
        Some("italic"),
        None,
        text_w,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    tracker.track_rect(
        text_x,
        text_y - GENERIC_BASELINE,
        text_w,
        GENERIC_TEXT_HEIGHT,
    );
}

/// Draw an Object entity box (EntityImageObject.java layout).
/// Render a rectangle entity with bracket-body description.
///
/// Java: rectangle entities have NO stereotype circle, NO title text, NO separator.
/// Only the bracket-body description lines are rendered as left-aligned text
/// at font-size 14 inside a rounded rect (rx=2.5).
fn draw_rectangle_entity_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    let x = nl.cx - nl.width / 2.0 + edge_offset_x;
    let y = nl.cy - nl.height / 2.0 + edge_offset_y;
    let w = nl.width;
    let h = nl.height;

    // Stereotype-keyed skinparams (C4 stdlib: rectangle<<container>>, etc.)
    // take precedence over generic element styling.
    let stereo_names: Vec<&str> = entity.stereotypes.iter().map(|s| s.0.as_str()).collect();
    let explicit_color = entity.color.as_deref();
    let bg_lookup = skin.background_color_for("rectangle", &stereo_names, ENTITY_BG);
    let fill: &str = explicit_color.unwrap_or(bg_lookup);
    let stroke = skin.border_color_for("rectangle", &stereo_names, BORDER_COLOR);
    let font_color = skin.font_color_for("rectangle", &stereo_names, TEXT_COLOR);
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    match entity.rect_symbol {
        RectSymbol::File => {
            draw_file_shape(sg, x, y, w, h, rx, fill, stroke);
            // Java USymbolFile draws the outline as a UPath, so LimitFinder uses
            // the drawDotPath path — no -1 adjustment. Use track_path_bounds to
            // match the +1 extent versus track_rect.
            tracker.track_path_bounds(x, y, x + w, y + h);
        }
        _ => {
            RectShape {
                x,
                y,
                w,
                h,
                rx,
                ry: rx,
            }
            .draw(sg, &DrawStyle::filled(fill, stroke, 0.5));
            tracker.track_rect(x, y, w, h);
        }
    }

    // Java: description text at font-size 14, left-aligned, padding 10px.
    // Use creole rendering to handle inline markup (<i>, <b>, etc.) and table syntax (|= ... |).
    // preserve_backslash_n=true: Java keeps literal \n as displayable text in bracket bodies.
    let text_x = x + 10.0;
    let top_y = y + 10.0;

    let mut tmp = String::new();
    render_creole_display_lines(
        &mut tmp,
        &entity.description,
        text_x,
        top_y,
        font_color,
        r#"font-size="14""#,
        true,
    );
    sg.push_raw(&tmp);
}

/// Draw a description-style `component` entity as it appears in the class
/// pipeline (Java `USymbolComponent2.asSmall`):
///
/// 1. Main rounded rectangle (rx = roundCorner/2, default 2.5)
/// 2. UML 2 component icon at top-right of the rect:
///    - Outer 15x10 rect at (w-20, 5)
///    - Two 4x2 tab rects on the left of the icon at (w-22, 7) and (w-22, 11)
/// 3. Centered label (and optional stereotypes stacked above the name)
///
/// Java margin: `Margin(10+5, 20+5, 15+5, 5+5)` → (left=15, right=25, top=20, bottom=10).
fn draw_component_description_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    cd: &ClassDiagram,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    const MARGIN_LEFT: f64 = 15.0;
    const MARGIN_TOP: f64 = 20.0;

    let x = nl.cx - nl.width / 2.0 + edge_offset_x;
    let y = nl.cy - nl.height / 2.0 + edge_offset_y;
    let w = nl.width;
    let h = nl.height;

    let default_fill = skin.background_color("component", ENTITY_BG);
    let fill = entity
        .color
        .as_deref()
        .map(crate::style::normalize_color)
        .or_else(|| class_stereotype_fill_color(&cd.stereotype_backgrounds, &entity.stereotypes))
        .unwrap_or_else(|| default_fill.to_string());
    let stroke = skin.border_color("component", BORDER_COLOR);
    let font_color = skin.font_color("component", TEXT_COLOR);
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    // Main rect
    RectShape {
        x,
        y,
        w,
        h,
        rx,
        ry: rx,
    }
    .draw(sg, &DrawStyle::filled(&fill, stroke, 0.5));
    tracker.track_rect(x, y, w, h);

    // Component icon at top-right of the rect.
    // Java translate offsets: small at (w-20, 5), tabs at (w-22, 7) and (w-22, 11).
    let icon_w: f64 = 15.0;
    let icon_h: f64 = 10.0;
    let icon_x = x + w - 20.0;
    let icon_y = y + 5.0;
    RectShape {
        x: icon_x,
        y: icon_y,
        w: icon_w,
        h: icon_h,
        rx: 0.0,
        ry: 0.0,
    }
    .draw(sg, &DrawStyle::filled(&fill, stroke, 0.5));
    tracker.track_rect(icon_x, icon_y, icon_w, icon_h);

    let tab_w: f64 = 4.0;
    let tab_h: f64 = 2.0;
    let tab_x = x + w - 22.0;
    RectShape {
        x: tab_x,
        y: y + 7.0,
        w: tab_w,
        h: tab_h,
        rx: 0.0,
        ry: 0.0,
    }
    .draw(sg, &DrawStyle::filled(&fill, stroke, 0.5));
    tracker.track_rect(tab_x, y + 7.0, tab_w, tab_h);

    RectShape {
        x: tab_x,
        y: y + 11.0,
        w: tab_w,
        h: tab_h,
        rx: 0.0,
        ry: 0.0,
    }
    .draw(sg, &DrawStyle::filled(&fill, stroke, 0.5));
    tracker.track_rect(tab_x, y + 11.0, tab_w, tab_h);

    // Resolve display name and font sizes.
    let class_font_size = skin
        .get("classfontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(FONT_SIZE);

    let name_display_raw = entity
        .display_name
        .as_deref()
        .map(String::from)
        .unwrap_or_else(|| crate::layout::class_entity_display_name(&entity.name));
    let markup_info = crate::layout::strip_html_markup(&name_display_raw);
    let name_display = if markup_info.bold || markup_info.italic {
        markup_info.text.clone()
    } else {
        name_display_raw
    };
    let name_bold = markup_info.bold;
    let name_italic = markup_info.italic;

    let visible_stereotypes = visible_stereotype_labels(&cd.hide_show_rules, entity);

    // Java mergeTB(stereotype, label, CENTER) draws stereotype above the label.
    // Both blocks are drawn at translate(MARGIN_LEFT, MARGIN_TOP) from the rect origin.
    let stereo_line_h =
        font_metrics::line_height("SansSerif", HEADER_STEREO_FONT_SIZE, false, true);
    let stereo_ascent = font_metrics::ascent("SansSerif", HEADER_STEREO_FONT_SIZE, false, true);
    let mut cur_top = y + MARGIN_TOP;
    for label in &visible_stereotypes {
        let stereo_text = format!("\u{00AB}{label}\u{00BB}");
        let tw = font_metrics::text_width(
            &stereo_text,
            "SansSerif",
            HEADER_STEREO_FONT_SIZE,
            false,
            true,
        );
        // Center horizontally within inner region (x + MARGIN_LEFT .. x + w - MARGIN_RIGHT).
        // Java mergeTB CENTER aligns each row to the wider block; we approximate by
        // centering each line on the inner region center.
        let inner_left = x + MARGIN_LEFT;
        let inner_right = x + w - 25.0;
        let line_x = inner_left + ((inner_right - inner_left) - tw) / 2.0;
        let baseline = cur_top + stereo_ascent;
        sg.push_raw(&format!(
            r#"<text fill="{font_color}" font-family="sans-serif" font-size="{fs:.0}" font-style="italic" lengthAdjust="spacing" textLength="{tl}" x="{x_}" y="{y_}">{txt}</text>"#,
            fs = HEADER_STEREO_FONT_SIZE,
            tl = fmt_coord(tw),
            x_ = fmt_coord(line_x),
            y_ = fmt_coord(baseline),
            txt = xml_escape(&stereo_text),
        ));
        tracker.track_rect(line_x, cur_top, tw, stereo_line_h);
        cur_top += stereo_line_h;
    }

    // Label block.
    let name_block = crate::layout::split_name_display(&name_display);
    let line_h = font_metrics::line_height("SansSerif", class_font_size, name_bold, name_italic);
    let ascent = font_metrics::ascent("SansSerif", class_font_size, name_bold, name_italic);
    let label_widths: Vec<(f64, f64)> = name_block
        .lines
        .iter()
        .map(|line| {
            crate::layout::display_line_metrics(line, class_font_size, name_bold, name_italic)
        })
        .collect();
    let label_max_w = label_widths
        .iter()
        .map(|(vw, iw)| vw + iw)
        .fold(0.0_f64, f64::max);

    // Java draws the label at the same translate(MARGIN_LEFT, MARGIN_TOP). When
    // there is no stereotype the label sits at MARGIN_TOP. Otherwise the label
    // is stacked under the stereotype block (cur_top after the loop).
    let font_style = if name_italic { Some("italic") } else { None };
    let font_weight = if name_bold { Some("bold") } else { None };
    sg.set_fill_color(font_color);
    for (idx, line) in name_block.lines.iter().enumerate() {
        let display_line = if line.text.is_empty() {
            "\u{00A0}".to_string()
        } else {
            line.text.clone()
        };
        let (visible_w, indent_w) = label_widths[idx];
        let measured_w = visible_w + indent_w;
        // mergeTB CENTER: each line centered relative to label_max_w
        let inner_offset = match name_block.alignment {
            crate::layout::DisplayAlignment::Left => 0.0,
            crate::layout::DisplayAlignment::Center => (label_max_w - measured_w) / 2.0,
            crate::layout::DisplayAlignment::Right => label_max_w - measured_w,
        };
        let line_x = x + MARGIN_LEFT + inner_offset + indent_w;
        let baseline = cur_top + ascent + idx as f64 * line_h;
        sg.svg_text(
            &display_line,
            line_x,
            baseline,
            Some("sans-serif"),
            class_font_size,
            font_weight,
            font_style,
            None,
            visible_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        tracker.track_rect(line_x, baseline - ascent, visible_w, line_h);
    }
}

/// Render a `file` symbol outline — rectangle with a folded top-right corner.
/// Java: `USymbolFile.drawFile()` with rounded path (cornersize=10, r=roundCorner/2).
fn draw_file_shape(
    sg: &mut SvgGraphic,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    r: f64,
    fill: &str,
    stroke: &str,
) {
    const CORNERSIZE: f64 = 10.0;
    // Outer body path. Matches Java `drawFile(...)` with roundCorner != 0 branch:
    //   M 0, r
    //   L 0, h-r
    //   A r,r 0 0 0 r, h
    //   L w-r, h
    //   A r,r 0 0 0 w, h-r
    //   L w, cornersize
    //   L w-cornersize, 0
    //   L r, 0
    //   A r,r 0 0 0 0, r
    let d_outer = format!(
        "M{},{} L{},{} A{},{} 0 0 0 {},{} L{},{} A{},{} 0 0 0 {},{} L{},{} L{},{} L{},{} A{},{} 0 0 0 {},{}",
        fmt_coord(x),              fmt_coord(y + r),
        fmt_coord(x),              fmt_coord(y + h - r),
        fmt_coord(r),              fmt_coord(r),
        fmt_coord(x + r),          fmt_coord(y + h),
        fmt_coord(x + w - r),      fmt_coord(y + h),
        fmt_coord(r),              fmt_coord(r),
        fmt_coord(x + w),          fmt_coord(y + h - r),
        fmt_coord(x + w),          fmt_coord(y + CORNERSIZE),
        fmt_coord(x + w - CORNERSIZE), fmt_coord(y),
        fmt_coord(x + r),          fmt_coord(y),
        fmt_coord(r),              fmt_coord(r),
        fmt_coord(x),              fmt_coord(y + r),
    );
    sg.push_raw(&format!(
        r#"<path d="{d_outer}" fill="{fill}" style="stroke:{stroke};stroke-width:0.5;"/>"#,
    ));

    // Fold dog-ear decoration path (the small cut that visually separates the
    // corner from the main body). Java uses the same fill as body, so the
    // triangle is painted over the upper-right cut-off area.
    //   M w-cornersize, 0
    //   L w-cornersize, cornersize - r
    //   A r,r 0 0 0 w-cornersize+r, cornersize
    //   L w, cornersize
    let d_fold = format!(
        "M{},{} L{},{} A{},{} 0 0 0 {},{} L{},{}",
        fmt_coord(x + w - CORNERSIZE),
        fmt_coord(y),
        fmt_coord(x + w - CORNERSIZE),
        fmt_coord(y + CORNERSIZE - r),
        fmt_coord(r),
        fmt_coord(r),
        fmt_coord(x + w - CORNERSIZE + r),
        fmt_coord(y + CORNERSIZE),
        fmt_coord(x + w),
        fmt_coord(y + CORNERSIZE),
    );
    sg.push_raw(&format!(
        r#"<path d="{d_fold}" fill="{fill}" style="stroke:{stroke};stroke-width:0.5;"/>"#,
    ));
}

///
/// Objects have NO stereotype circle icon, NO glyph path.
/// Name is centered with margin(2,2,2,2), no underline (default, non-strict UML).
/// Body is a single separator line followed by empty space (TextBlockEmpty(10, 16)).
fn draw_object_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    entity: &Entity,
    nl: &NodeLayout,
    skin: &SkinParams,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    let x = nl.cx - nl.width / 2.0 + edge_offset_x;
    let y = nl.cy - nl.height / 2.0 + edge_offset_y;
    let w = nl.width;
    let h = nl.height;

    let default_fill = skin.background_color("object", ENTITY_BG);
    let fill = entity.color.as_deref().unwrap_or(default_fill);
    let stroke_color = skin.border_color("object", BORDER_COLOR);
    let font_color = skin.font_color("object", TEXT_COLOR);

    // Java URectangle.rounded(roundCorner): SVG rx = roundCorner / 2.
    let rx = skin.round_corner().map(|rc| rc / 2.0).unwrap_or(2.5);

    // Rect
    RectShape {
        x,
        y,
        w,
        h,
        rx,
        ry: rx,
    }
    .draw(sg, &DrawStyle::filled(fill, stroke_color, 0.5));
    tracker.track_rect(x, y, w, h);

    // Object name constants — EntityImageObject.java
    // withMargin(tmp, 2, 2) → margin(top=2, right=2, bottom=2, left=2)
    const OBJ_NAME_MARGIN: f64 = 2.0;

    let class_font_size = skin.font_size("class", FONT_SIZE);
    // Use display_name (from `as Alias` syntax) — it includes the "Map" keyword for map entities.
    let nd = entity.display_name.as_deref().unwrap_or(&entity.name);
    let has_creole = nd.contains("**") || nd.contains("//");
    let name_width = if has_creole {
        crate::render::svg_richtext::measure_creole_display_lines(
            &[nd.to_string()],
            "SansSerif",
            class_font_size,
            false,
            false,
            false,
        )
        .0
    } else {
        font_metrics::text_width(nd, "SansSerif", class_font_size, false, false)
    };
    let name_block_width = name_width + 2.0 * OBJ_NAME_MARGIN;
    let name_block_height = HEADER_NAME_BLOCK_HEIGHT + 2.0 * OBJ_NAME_MARGIN;

    // PlacementStrategyY1Y2 with 1 element: x = (totalWidth - blockWidth) / 2
    // height = titleHeight = name_block_height, so space = 0, y = 0
    let name_offset_x = (w - name_block_width) / 2.0;
    let text_x = x + name_offset_x + OBJ_NAME_MARGIN;
    let text_y = y + OBJ_NAME_MARGIN + HEADER_NAME_BASELINE;

    if has_creole {
        // Render with creole richtext support (handles **bold**, //italic//, etc.)
        let outer_attrs = format!(r#"font-size="{}""#, class_font_size as i32);
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            nd,
            text_x,
            text_y,
            text_block_h(class_font_size, false),
            font_color,
            None,
            &outer_attrs,
        );
        sg.push_raw(&tmp);
    } else {
        sg.set_fill_color(font_color);
        sg.svg_text(
            nd,
            text_x,
            text_y,
            Some("sans-serif"),
            class_font_size,
            None,
            None,
            None,
            name_width,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
    }
    tracker.track_rect(
        text_x,
        text_y - HEADER_NAME_BASELINE,
        name_width,
        HEADER_NAME_BLOCK_HEIGHT,
    );

    // Separator line at y + titleHeight
    let title_height = name_block_height;
    let sep_y = y + title_height;
    let x1 = x + 1.0;
    let x2 = x + w - 1.0;

    // Map entities: render key => value table body
    // Java EntityImageMap: each row uses withMargin(text, 2, 2), adding 4px vertical padding.
    if entity.kind == EntityKind::Map && !entity.map_entries.is_empty() {
        let attr_font_size = skin.font_size("classattribute", class_font_size);
        let text_line_h = font_metrics::line_height("SansSerif", attr_font_size, false, false);
        let row_margin_top = 2.0; // withMargin(2,2): top inset before text baseline
        let row_margin = 4.0; // withMargin(2,2) → 2 top + 2 bottom
        let row_h = text_line_h + row_margin;
        let ascent = font_metrics::ascent("SansSerif", attr_font_size, false, false);
        // Java TextBlockMap: withMargin(result, 5, 2) → 5px left + 5px right per column
        let cell_margin_lr = 5.0;
        let col_a_width: f64 = entity
            .map_entries
            .iter()
            .map(|(key, _)| {
                font_metrics::text_width(key, "SansSerif", attr_font_size, false, false)
                    + 2.0 * cell_margin_lr
            })
            .fold(0.0_f64, f64::max);
        let mut cur_y = sep_y;
        for (key, value) in &entity.map_entries {
            LineShape {
                x1: x,
                y1: cur_y,
                x2: x + w,
                y2: cur_y,
            }
            .draw(sg, &DrawStyle::outline(stroke_color, 1.0));
            tracker.track_line(x, cur_y, x + w, cur_y);
            let key_w = font_metrics::text_width(key, "SansSerif", attr_font_size, false, false);
            let text_y_row = cur_y + row_margin_top + ascent;
            sg.set_fill_color(font_color);
            sg.svg_text(
                key,
                x + cell_margin_lr,
                text_y_row,
                Some("sans-serif"),
                attr_font_size,
                None,
                None,
                None,
                key_w,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
            let val_w = font_metrics::text_width(value, "SansSerif", attr_font_size, false, false);
            sg.svg_text(
                value,
                x + col_a_width + cell_margin_lr,
                text_y_row,
                Some("sans-serif"),
                attr_font_size,
                None,
                None,
                None,
                val_w,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
            LineShape {
                x1: x + col_a_width,
                y1: cur_y,
                x2: x + col_a_width,
                y2: cur_y + row_h,
            }
            .draw(sg, &DrawStyle::outline(stroke_color, 1.0));
            tracker.track_line(x + col_a_width, cur_y, x + col_a_width, cur_y + row_h);
            cur_y += row_h;
        }
    } else {
        // Render object fields in the body section
        let visible_fields: Vec<&Member> = entity.members.iter().filter(|m| !m.is_method).collect();
        if !visible_fields.is_empty() {
            // draw_member_section draws its own separator at section_y
            let attr_font_size = skin.font_size("classattribute", class_font_size);
            let x1_val = fmt_coord(x1);
            let x2_val = fmt_coord(x2);
            draw_member_section(
                sg,
                tracker,
                &visible_fields,
                sep_y,
                x,
                &x1_val,
                &x2_val,
                font_color,
                attr_font_size,
                stroke_color,
            );
        } else {
            // No fields: draw the separator line explicitly
            LineShape {
                x1,
                y1: sep_y,
                x2,
                y2: sep_y,
            }
            .draw(sg, &DrawStyle::outline(stroke_color, 0.5));
            tracker.track_line(x1, sep_y, x2, sep_y);
        }
    }
}

fn draw_member_section(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    members: &[&Member],
    section_y: f64,
    x: f64,
    x1_val: &str,
    x2_val: &str,
    font_color: &str,
    attr_font_size: f64,
    sep_color: &str,
) {
    // Compute dynamic row metrics from attr_font_size (matches Java FontParam.CLASS_ATTRIBUTE)
    let row_h = font_metrics::line_height("SansSerif", attr_font_size, false, false);
    let attr_ascent = font_metrics::ascent("SansSerif", attr_font_size, false, false);
    let margin_top = 4.0;
    let text_y_offset = margin_top + attr_ascent;
    let icon_y_from_sep = margin_top + 2.0 + (row_h - 11.0) / 2.0;

    let _sep_y_str = fmt_coord(section_y);
    // Parse x1/x2 for line tracking
    let x1_f: f64 = x1_val.parse().unwrap_or(x + 1.0);
    let x2_f: f64 = x2_val.parse().unwrap_or(x);
    LineShape {
        x1: x1_f,
        y1: section_y,
        x2: x2_f,
        y2: section_y,
    }
    .draw(sg, &DrawStyle::outline(sep_color, 0.5));
    tracker.track_line(x1_f, section_y, x2_f, section_y);
    let (section_w, section_h) = member_section_block_dimensions(members, attr_font_size);
    tracker.track_empty(x, section_y, section_w, section_h);

    // visual_row tracks the current visual line index across all members
    let mut visual_row: usize = 0;

    for member in members.iter() {
        let text = member_text(member);
        let lines = split_member_lines(&text);
        let num_lines = lines.len();

        // Visibility icon: centered vertically across all visual lines of this member
        if let Some(visibility) = &member.visibility {
            let icon_y = section_y
                + icon_y_from_sep
                + visual_row as f64 * row_h
                + (num_lines.saturating_sub(1)) as f64 * row_h / 2.0;
            draw_visibility_icon(
                sg,
                tracker,
                visibility,
                member.is_method,
                x + MEMBER_ICON_X_OFFSET,
                icon_y,
            );
        }

        let font_style_attr: Option<&str> = if member.modifiers.is_abstract {
            Some("italic")
        } else {
            None
        };
        let text_deco_attr: Option<&str> = if member.modifiers.is_static {
            Some("underline")
        } else {
            None
        };

        let base_text_x = x + if member.visibility.is_some() {
            MEMBER_TEXT_X_WITH_ICON
        } else {
            MEMBER_TEXT_X_NO_ICON
        };

        for (line_idx, (line_text, indent)) in lines.iter().enumerate() {
            let text_y = section_y + text_y_offset + (visual_row + line_idx) as f64 * row_h;
            let text_x = if line_idx == 0 {
                base_text_x
            } else {
                base_text_x + indent
            };
            let text_width_val = font_metrics::text_width(
                line_text,
                "SansSerif",
                attr_font_size,
                false,
                member.modifiers.is_abstract,
            );
            sg.set_fill_color(font_color);
            sg.svg_text(
                line_text,
                text_x,
                text_y,
                Some("sans-serif"),
                attr_font_size,
                None,
                font_style_attr,
                text_deco_attr,
                text_width_val,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
            {
                let text_ascent = font_metrics::ascent(
                    "SansSerif",
                    attr_font_size,
                    false,
                    member.modifiers.is_abstract,
                );
                let text_descent = font_metrics::descent(
                    "SansSerif",
                    attr_font_size,
                    false,
                    member.modifiers.is_abstract,
                );
                tracker.track_rect(
                    text_x,
                    text_y - text_ascent,
                    text_width_val,
                    text_ascent + text_descent,
                );
            }
        }

        visual_row += num_lines;
    }
}

fn member_section_block_dimensions(members: &[&Member], attr_font_size: f64) -> (f64, f64) {
    if members.is_empty() {
        return (12.0, EMPTY_COMPARTMENT);
    }

    // Java MethodsOrFieldsArea wraps the member content block with
    // TextBlockUtils.withMargin(..., 6, 4), which contributes a UEmpty
    // wrapper to LimitFinder even when inner text/icon primitives are tracked
    // separately.
    let has_small_icon = members.iter().any(|m| m.visibility.is_some());
    let icon_col_w = if has_small_icon { 14.0 } else { 0.0 };
    let text_w = members
        .iter()
        .map(|member| {
            let text = member_text(member);
            split_member_lines(&text)
                .iter()
                .enumerate()
                .map(|(idx, (line_text, indent))| {
                    let line_w = font_metrics::text_width(
                        line_text,
                        "SansSerif",
                        attr_font_size,
                        false,
                        member.modifiers.is_abstract,
                    );
                    if idx == 0 {
                        line_w
                    } else {
                        indent + line_w
                    }
                })
                .fold(0.0_f64, f64::max)
        })
        .fold(0.0_f64, f64::max);
    let content_w = icon_col_w + text_w;
    let content_h = section_height_with_fs(members, attr_font_size) - 8.0;
    (content_w + 12.0, content_h + 8.0)
}

fn section_height_with_fs(members: &[&Member], attr_font_size: f64) -> f64 {
    if members.is_empty() {
        EMPTY_COMPARTMENT
    } else {
        let row_h = font_metrics::line_height("SansSerif", attr_font_size, false, false);
        let one_row_h = row_h + 8.0; // margin_top(4) + row_h + margin_bottom(4)
        let total_visual_lines: usize = members
            .iter()
            .map(|m| {
                let text = member_text(m);
                split_member_lines(&text).len()
            })
            .sum();
        one_row_h + (total_visual_lines.saturating_sub(1)) as f64 * row_h
    }
}

/// Java MemberImpl.getDisplay() format:
/// Uses raw display text when available (preserves original formatting).
/// Fallback: methods "name(): type", fields "name : type".
fn member_text(m: &Member) -> String {
    if let Some(ref display) = m.display {
        return display.clone();
    }
    match &m.return_type {
        Some(rt) if m.name.ends_with(')') => format!("{}: {rt}", m.name),
        Some(rt) => format!("{} : {rt}", m.name),
        None => m.name.clone(),
    }
}

/// Draw visibility modifier icon matching Java VisibilityModifier.java.
/// Colors and shapes from VisibilityModifier.java:
///   PUBLIC:    circle, fill=#84BE84(method)/none(field), stroke=#038048
///   PRIVATE:   square, fill=#F24D5C(method)/none(field), stroke=#C82930
///   PROTECTED: diamond, fill=#B38D22(method)/none(field), stroke=#B38D22
///   PACKAGE:   triangle, fill=#4177AF(method)/none(field), stroke=#1963A0
fn draw_visibility_icon(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    visibility: &Visibility,
    is_method: bool,
    x: f64,
    y: f64,
) {
    let modifier = match (visibility, is_method) {
        (Visibility::Public, true) => "PUBLIC_METHOD",
        (Visibility::Public, false) => "PUBLIC_FIELD",
        (Visibility::Private, true) => "PRIVATE_METHOD",
        (Visibility::Private, false) => "PRIVATE_FIELD",
        (Visibility::Protected, true) => "PROTECTED_METHOD",
        (Visibility::Protected, false) => "PROTECTED_FIELD",
        (Visibility::Package, true) => "PACKAGE_PRIVATE_METHOD",
        (Visibility::Package, false) => "PACKAGE_PRIVATE_FIELD",
    };
    sg.push_raw(&format!(r#"<g data-visibility-modifier="{modifier}">"#));
    match visibility {
        Visibility::Public => {
            // VisibilityModifier.drawCircle: translate(x+2,y+2), UEllipse(6,6)
            let ecx = x + 2.0 + 3.0;
            let ecy = y + 2.0 + 3.0;
            let fill = if is_method { "#84BE84" } else { "none" };
            EllipseShape {
                cx: ecx,
                cy: ecy,
                rx: 3.0,
                ry: 3.0,
            }
            .draw(sg, &DrawStyle::filled(fill, "#038048", 1.0));
            tracker.track_ellipse(ecx, ecy, 3.0, 3.0);
        }
        Visibility::Private => {
            // VisibilityModifier.drawSquare: translate(x+2,y+2), URectangle(6,6)
            let rect_x = x + 2.0;
            let rect_y = y + 2.0;
            let fill = if is_method { "#F24D5C" } else { "none" };
            RectShape {
                x: rect_x,
                y: rect_y,
                w: 6.0,
                h: 6.0,
                rx: 0.0,
                ry: 0.0,
            }
            .draw(sg, &DrawStyle::filled(fill, "#C82930", 1.0));
            tracker.track_rect(rect_x, rect_y, 6.0, 6.0);
        }
        Visibility::Protected => {
            // VisibilityModifier.drawDiamond: size -= 2 (10→8), translate(x+1,y+0), UPolygon
            // Points: (size/2,0),(size,size/2),(size/2,size),(0,size/2) where size=8
            let ox = x + 1.0;
            let oy = y;
            let fill = if is_method { "#B38D22" } else { "none" };
            let poly_pts = [
                (ox + 4.0, oy),
                (ox + 8.0, oy + 4.0),
                (ox + 4.0, oy + 8.0),
                (ox, oy + 4.0),
            ];
            PolygonShape {
                points: vec![
                    poly_pts[0].0,
                    poly_pts[0].1,
                    poly_pts[1].0,
                    poly_pts[1].1,
                    poly_pts[2].0,
                    poly_pts[2].1,
                    poly_pts[3].0,
                    poly_pts[3].1,
                ],
            }
            .draw(sg, &DrawStyle::filled(fill, "#B38D22", 1.0));
            tracker.track_polygon(&poly_pts);
        }
        Visibility::Package => {
            // VisibilityModifier.drawTriangle: size -= 2 (10→8), translate(x+1,y+0)
            // Points: (size/2,1),(0,size-1),(size,size-1) where size=8
            let ox = x + 1.0;
            let oy = y;
            let fill = if is_method { "#4177AF" } else { "none" };
            let poly_pts = [
                (ox + 4.0, oy + 1.0), // (size/2=4, 1)
                (ox, oy + 7.0),       // (0, size-1=7)
                (ox + 8.0, oy + 7.0), // (size=8, size-1=7)
            ];
            PolygonShape {
                points: vec![
                    poly_pts[0].0,
                    poly_pts[0].1,
                    poly_pts[1].0,
                    poly_pts[1].1,
                    poly_pts[2].0,
                    poly_pts[2].1,
                ],
            }
            .draw(sg, &DrawStyle::filled(fill, "#1963A0", 1.0));
            tracker.track_polygon(&poly_pts);
        }
    }
    sg.push_raw("</g>");
}

fn show_portion(
    rules: &[ClassHideShowRule],
    portion: ClassPortion,
    entity_name: &str,
    member_count: usize,
) -> bool {
    let mut result = true;
    for rule in rules {
        if rule.portion != portion {
            continue;
        }
        if rule.empty_only && member_count > 0 {
            continue;
        }
        match &rule.target {
            ClassRuleTarget::Any => result = rule.show,
            ClassRuleTarget::Entity(name) if name == entity_name => result = rule.show,
            _ => {}
        }
    }
    result
}

fn visible_stereotype_labels(rules: &[ClassHideShowRule], entity: &Entity) -> Vec<String> {
    entity
        .stereotypes
        .iter()
        .map(|st| {
            // Extract spot notation and return cleaned label
            let (_, cleaned) = st.extract_spot();
            cleaned
        })
        .filter(|label| !label.is_empty() && stereotype_label_visible(rules, label))
        .collect()
}

/// Extract spot info from entity stereotypes.
/// Returns the first spot found (character + resolved color), if any.
fn extract_entity_spot(entity: &Entity) -> Option<crate::model::entity::StereotypeSpot> {
    for st in &entity.stereotypes {
        let (spot, _) = st.extract_spot();
        if let Some(s) = spot {
            return Some(s);
        }
    }
    None
}

fn class_group_fill_color(
    cd: &ClassDiagram,
    group: Option<&crate::model::Group>,
) -> Option<String> {
    let group = group?;
    group
        .color
        .as_deref()
        .map(crate::style::normalize_color)
        .or_else(|| class_stereotype_fill_color(&cd.stereotype_backgrounds, &group.stereotypes))
}

fn class_stereotype_fill_color(
    stereotype_backgrounds: &HashMap<String, String>,
    stereotypes: &[crate::model::Stereotype],
) -> Option<String> {
    stereotypes
        .iter()
        .filter_map(|stereotype| stereotype_backgrounds.get(&stereotype.0))
        .map(|color| crate::style::normalize_color(color))
        .next_back()
}

fn stereotype_label_visible(rules: &[ClassHideShowRule], label: &str) -> bool {
    let mut result = true;
    for rule in rules {
        if rule.portion != ClassPortion::Stereotype {
            continue;
        }
        match &rule.target {
            ClassRuleTarget::Any => result = rule.show,
            ClassRuleTarget::Stereotype(name) if name == label => result = rule.show,
            _ => {}
        }
    }
    result
}

#[allow(dead_code)] // reserved for class member formatting
pub(super) fn format_member(m: &Member) -> String {
    let vis = match &m.visibility {
        Some(Visibility::Public) => "+ ",
        Some(Visibility::Private) => "- ",
        Some(Visibility::Protected) => "# ",
        Some(Visibility::Package) => "~ ",
        None => "",
    };
    match &m.return_type {
        Some(rt) => format!("{vis}{}: {rt}", m.name),
        None => format!("{vis}{}", m.name),
    }
}

/// Derive the `data-link-type` attribute value from the link's arrow and line style.
fn derive_link_type(link: &Link) -> &'static str {
    let left = &link.left_head;
    let right = &link.right_head;
    if matches!(left, ArrowHead::Diamond) || matches!(right, ArrowHead::Diamond) {
        "composition"
    } else if matches!(left, ArrowHead::DiamondHollow) || matches!(right, ArrowHead::DiamondHollow)
    {
        "aggregation"
    } else if matches!(left, ArrowHead::Triangle) || matches!(right, ArrowHead::Triangle) {
        "extension"
    } else if matches!(left, ArrowHead::Arrow) || matches!(right, ArrowHead::Arrow) {
        "dependency"
    } else if matches!(left, ArrowHead::Plus) || matches!(right, ArrowHead::Plus) {
        "innerclass"
    } else {
        "association"
    }
}

fn edge_label_margin(link: &Link) -> f64 {
    if link.from == link.to {
        6.0
    } else {
        1.0
    }
}

/// Parse the start and end points from an SVG path d-string.
/// Returns ((start_x, start_y), (end_x, end_y)) or None.
fn parse_path_start_end(d: &str) -> Option<((f64, f64), (f64, f64))> {
    // Start: first M command
    let d = d.trim();
    if !d.starts_with('M') {
        return None;
    }
    let rest = &d[1..];
    // Parse start coordinates: "x,y" or "x y"
    let mut chars = rest.chars().peekable();
    let sx_str: String = chars
        .by_ref()
        .take_while(|c| *c != ',' && *c != ' ')
        .collect();
    let sy_str: String = chars
        .by_ref()
        .take_while(|c| *c != ' ' && *c != 'C' && *c != 'L' && *c != 'c' && *c != 'l')
        .collect();
    let sx = sx_str.parse::<f64>().ok()?;
    let sy = sy_str.parse::<f64>().ok()?;

    // End: last numeric pair in the path
    // Find all numbers at the end of the path
    let bytes = d.as_bytes();
    let mut end = d.len();
    // Skip trailing whitespace
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    // Walk backwards to find the last y coordinate
    let mut ey_end = end;
    while ey_end > 0
        && (bytes[ey_end - 1].is_ascii_digit()
            || bytes[ey_end - 1] == b'.'
            || bytes[ey_end - 1] == b'-')
    {
        ey_end -= 1;
    }
    let ey_str = &d[ey_end..end];
    // Skip separator (comma or space)
    let mut ex_end = ey_end;
    if ex_end > 0 && (bytes[ex_end - 1] == b',' || bytes[ex_end - 1] == b' ') {
        ex_end -= 1;
    }
    // Walk backwards to find the last x coordinate
    let mut ex_start = ex_end;
    while ex_start > 0
        && (bytes[ex_start - 1].is_ascii_digit()
            || bytes[ex_start - 1] == b'.'
            || bytes[ex_start - 1] == b'-')
    {
        ex_start -= 1;
    }
    let ex_str = &d[ex_start..ex_end];
    let ex = ex_str.parse::<f64>().ok()?;
    let ey = ey_str.parse::<f64>().ok()?;

    Some(((sx, sy), (ex, ey)))
}

/// Draw a TextBlockArrow2 polygon for a link label arrow indicator.
/// Java: `TextBlockArrow2.drawU()` renders a small triangle whose direction
/// is determined by the edge path angle, font size, and link arrow direction.
///
/// `origin_x`, `origin_y` is the top-left of the arrow text block (13×13),
/// already inside the TextBlockMarged margin.
fn draw_label_arrow_polygon(
    sg: &mut SvgGraphic,
    origin_x: f64,
    origin_y: f64,
    angle: f64,
    font_size: f64,
) {
    let tri_size = (font_size * 0.80) as i32;
    let tri_size_f = tri_size as f64;
    let cx = origin_x + tri_size_f / 2.0;
    let cy = origin_y + font_size / 2.0;
    let radius = tri_size_f / 2.0;
    let beta = std::f64::consts::PI * 4.0 / 5.0;

    let p0x = cx + radius * angle.sin();
    let p0y = cy + radius * angle.cos();
    let p1x = cx + radius * (angle + beta).sin();
    let p1y = cy + radius * (angle + beta).cos();
    let p2x = cx + radius * (angle - beta).sin();
    let p2y = cy + radius * (angle - beta).cos();

    let points_str = format!(
        "{},{},{},{},{},{},{},{}",
        crate::klimt::svg::fmt_coord(p0x),
        crate::klimt::svg::fmt_coord(p0y),
        crate::klimt::svg::fmt_coord(p1x),
        crate::klimt::svg::fmt_coord(p1y),
        crate::klimt::svg::fmt_coord(p2x),
        crate::klimt::svg::fmt_coord(p2y),
        crate::klimt::svg::fmt_coord(p0x),
        crate::klimt::svg::fmt_coord(p0y),
    );
    sg.push_raw(&format!(
        "<polygon fill=\"#000000\" points=\"{}\" style=\"stroke:#000000;stroke-width:1;\"/>",
        points_str,
    ));
}

fn draw_edge(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    layout: &GraphLayout,
    link: &Link,
    el: &EdgeLayout,
    link_idx: usize,
    qualifier_placements: &HashMap<QualifierKey, KalPlacement>,
    skin: &SkinParams,
    link_color: &str,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if el.points.is_empty() {
        return;
    }

    let mut decor_points = el.points.clone();
    if let Some(placement) = qualifier_placements.get(&QualifierKey {
        link_idx,
        endpoint: QualifierEndpoint::Tail,
    }) {
        if let Some((dx, dy)) = qualifier_edge_translation(link, QualifierEndpoint::Tail, placement)
        {
            move_edge_start_point(&mut decor_points, dx, dy);
        }
    }
    if let Some(placement) = qualifier_placements.get(&QualifierKey {
        link_idx,
        endpoint: QualifierEndpoint::Head,
    }) {
        if let Some((dx, dy)) = qualifier_edge_translation(link, QualifierEndpoint::Head, placement)
        {
            move_edge_end_point(&mut decor_points, dx, dy);
        }
    }

    let mut path_points = decor_points.clone();
    if link.left_head != ArrowHead::None {
        shorten_edge_for_head(&mut path_points, &link.left_head, true);
    }
    if link.right_head != ArrowHead::None {
        shorten_edge_for_head(&mut path_points, &link.right_head, false);
    }

    let d = build_edge_path_d(&path_points, edge_offset_x, edge_offset_y);

    // Track the edge path bounds (UPath style)
    {
        let mut p_min_x = f64::INFINITY;
        let mut p_min_y = f64::INFINITY;
        let mut p_max_x = f64::NEG_INFINITY;
        let mut p_max_y = f64::NEG_INFINITY;
        for &(px, py) in &path_points {
            let ax = px + edge_offset_x;
            let ay = py + edge_offset_y;
            if ax < p_min_x {
                p_min_x = ax;
            }
            if ay < p_min_y {
                p_min_y = ay;
            }
            if ax > p_max_x {
                p_max_x = ax;
            }
            if ay > p_max_y {
                p_max_y = ay;
            }
        }
        if p_min_x.is_finite() {
            tracker.track_path_bounds(p_min_x, p_min_y, p_max_x, p_max_y);
        }
    }

    // Java: dashed lines use stroke-dasharray:7,7; INSIDE the style attribute.
    let dash_style = if link.line_style == LineStyle::Dashed {
        "stroke-dasharray:7,7;"
    } else {
        ""
    };
    // Java Link.idCommentForSvg(): separator depends on decorations.
    // Java decor1 = head decoration (right_head), decor2 = tail decoration (left_head).
    let path_id = class_link_id_for_svg(link);
    {
        let mut path_elt = String::from("<path");
        if let Some(source_line) = link.source_line {
            write!(path_elt, r#" codeLine="{source_line}""#).unwrap();
        }
        write!(
            path_elt,
            r#" d="{d}" fill="none" id="{}" style="stroke:{link_color};stroke-width:1;{dash_style}"/>"#,
            crate::klimt::svg::xml_escape_attr(&path_id),
        )
        .unwrap();
        sg.push_raw(&path_elt);
    }

    if link.left_head != ArrowHead::None {
        emit_arrowhead(
            sg,
            tracker,
            &link.left_head,
            &decor_points,
            true,
            link_color,
            edge_offset_x,
            edge_offset_y,
        );
    }
    if link.right_head != ArrowHead::None {
        emit_arrowhead(
            sg,
            tracker,
            &link.right_head,
            &decor_points,
            false,
            link_color,
            edge_offset_x,
            edge_offset_y,
        );
    }

    if let Some(label) = &link.label {
        let margin_label = edge_label_margin(link);
        if let Some((lx, ly)) = el.label_xy.map(|(x, y)| {
            (
                x + layout.move_delta.0 - layout.normalize_offset.0 + edge_offset_x,
                y + layout.move_delta.1 - layout.normalize_offset.1 + edge_offset_y,
            )
        }) {
            let has_arrow = has_link_arrow_indicator(label);
            let label_text = if has_arrow {
                strip_link_arrow_text(label)
            } else {
                label.clone()
            };
            // Java Display.create() converts << >> to guillemets « »
            let label_text = label_text
                .replace("<<", "\u{00AB}")
                .replace(">>", "\u{00BB}");
            let arrow_w = if has_arrow { LINK_LABEL_FONT_SIZE } else { 0.0 };

            if has_arrow {
                // Compute arrow direction from the rendered edge path.
                // Java uses dotPath.getStartPoint()/getEndPoint() which correspond
                // to the rendered SVG path M start and last C/L end coordinates.
                //
                // Java SvekEdge.solveLine() checks whether GraphViz inverted the
                // edge direction: if the path start is closer to entity2 than
                // entity1, it reverses the dotPath.  We replicate this check here.
                let angle_points = parse_path_start_end(&d)
                    .unwrap_or_else(|| (el.points[0], el.points[el.points.len() - 1]));
                let (mut sx, mut sy) = angle_points.0;
                let (mut ex, mut ey) = angle_points.1;

                // Check for Graphviz path inversion: find entity centers and
                // compare distances.  If start is closer to the link's "to"
                // entity, the path was laid out in reverse.
                let find_center = |name: &str| -> Option<(f64, f64)> {
                    layout
                        .nodes
                        .iter()
                        .find(|n| n.id == name)
                        .map(|n| (n.cx, n.cy))
                };
                if let (Some(pos1), Some(pos2)) = (find_center(&link.from), find_center(&link.to)) {
                    let dist = |a: (f64, f64), b: (f64, f64)| -> f64 {
                        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
                    };
                    let normal = dist((sx, sy), pos1) + dist((ex, ey), pos2);
                    let inversed = dist((sx, sy), pos2) + dist((ex, ey), pos1);
                    if inversed < normal {
                        std::mem::swap(&mut sx, &mut ex);
                        std::mem::swap(&mut sy, &mut ey);
                    }
                }

                let mut angle = (ex - sx).atan2(ey - sy);
                if is_link_arrow_backward(label) {
                    angle += std::f64::consts::PI;
                }
                // Java: addMagicArrow merges TextBlockArrow2 LEFT of the margin-wrapped text.
                // The arrow is NOT inside the margin — only the text has the margin.
                // Outer height = max(arrow_h=13, text_h + 2*margin).
                // dy_arrow = (outer_h - 13) / 2.
                let text_h =
                    font_metrics::line_height("SansSerif", LINK_LABEL_FONT_SIZE, false, false);
                let text_marged_h = text_h + 2.0 * margin_label;
                let outer_h = text_marged_h.max(LINK_LABEL_FONT_SIZE);
                let dy_arrow = (outer_h - LINK_LABEL_FONT_SIZE) / 2.0;
                draw_label_arrow_polygon(sg, lx, ly + dy_arrow, angle, LINK_LABEL_FONT_SIZE);
            }

            draw_edge_label_block(
                sg,
                tracker,
                &label_text,
                lx + arrow_w,
                ly,
                el.label_wh.map(|(w, h)| (w - arrow_w, h)),
                margin_label,
                LINK_LABEL_FONT_SIZE,
                false,
                skin,
            );
        } else {
            let mid_idx = path_points.len() / 2;
            let (mx, my) = path_points[mid_idx];
            let label_x = mx + edge_offset_x;
            let label_y = my + edge_offset_y - 6.0;
            draw_label(sg, label, label_x, label_y);
            let lines = split_label_lines(label);
            let line_height =
                font_metrics::line_height("SansSerif", LINK_LABEL_FONT_SIZE, false, false);
            let ascent = font_metrics::ascent("SansSerif", LINK_LABEL_FONT_SIZE, false, false);
            let widths: Vec<f64> = lines
                .iter()
                .map(|(t, _)| {
                    font_metrics::text_width(t, "SansSerif", LINK_LABEL_FONT_SIZE, false, false)
                })
                .collect();
            let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
            let total_h = lines.len() as f64 * line_height;
            let block_x = label_x + 1.0;
            let base_y = label_y - total_h / 2.0 + ascent;
            for (idx, _line_text) in lines.iter().map(|(t, _)| t).enumerate() {
                let text_w = widths[idx];
                let ly = base_y + idx as f64 * line_height;
                tracker.track_text(block_x, ly, text_w, line_height);
            }
            tracker.track_empty(label_x, base_y, max_width + 2.0, 0.0);
        }
    }

    if let Some((text, x, y)) = edge_side_label_origin(
        layout,
        el.tail_label.as_deref(),
        el.tail_label_xy,
        edge_offset_x,
        edge_offset_y,
    ) {
        draw_edge_label_block(
            sg,
            tracker,
            text,
            x,
            y,
            el.tail_label_wh,
            if el.tail_label_boxed { 2.0 } else { 0.0 },
            if el.tail_label_boxed {
                14.0
            } else {
                LINK_LABEL_FONT_SIZE
            },
            el.tail_label_boxed,
            skin,
        );
    }

    if let Some((text, x, y)) = edge_side_label_origin(
        layout,
        el.head_label.as_deref(),
        el.head_label_xy,
        edge_offset_x,
        edge_offset_y,
    ) {
        draw_edge_label_block(
            sg,
            tracker,
            text,
            x,
            y,
            el.head_label_wh,
            if el.head_label_boxed { 2.0 } else { 0.0 },
            if el.head_label_boxed {
                14.0
            } else {
                LINK_LABEL_FONT_SIZE
            },
            el.head_label_boxed,
            skin,
        );
    }

    if let Some(text) = link.from_qualifier.as_deref() {
        if let Some(placement) = qualifier_placements.get(&QualifierKey {
            link_idx,
            endpoint: QualifierEndpoint::Tail,
        }) {
            draw_kal_box(
                sg,
                tracker,
                text,
                placement.x,
                placement.y,
                placement.width,
                placement.height,
                skin,
            );
        }
    }

    if let Some(text) = link.to_qualifier.as_deref() {
        if let Some(placement) = qualifier_placements.get(&QualifierKey {
            link_idx,
            endpoint: QualifierEndpoint::Head,
        }) {
            draw_kal_box(
                sg,
                tracker,
                text,
                placement.x,
                placement.y,
                placement.width,
                placement.height,
                skin,
            );
        }
    }
}

fn shorten_edge_for_head(points: &mut Vec<(f64, f64)>, head: &ArrowHead, is_start: bool) {
    let decoration_length = decoration_length(head);
    if decoration_length == 0.0 || points.is_empty() {
        return;
    }

    if is_start {
        let angle = edge_start_angle(points);
        move_edge_start_point(
            points,
            decoration_length * angle.cos(),
            decoration_length * angle.sin(),
        );
    } else {
        let angle = edge_end_angle(points);
        move_edge_end_point(
            points,
            decoration_length * (angle - std::f64::consts::PI).cos(),
            decoration_length * (angle - std::f64::consts::PI).sin(),
        );
    }
}

fn decoration_length(head: &ArrowHead) -> f64 {
    match head {
        ArrowHead::None => 0.0,
        ArrowHead::Arrow => 6.0,
        ArrowHead::Triangle => 18.0,
        ArrowHead::Diamond | ArrowHead::DiamondHollow => 12.0,
        ArrowHead::Plus => 16.0,
    }
}

fn build_edge_path_d(points: &[(f64, f64)], offset_x: f64, offset_y: f64) -> String {
    let mut d = String::new();
    if points.is_empty() {
        return d;
    }

    write!(
        d,
        "M{},{} ",
        fmt_coord(points[0].0 + offset_x),
        fmt_coord(points[0].1 + offset_y),
    )
    .unwrap();

    let rest = &points[1..];
    if is_cubic_edge_path(points) {
        for chunk in rest.chunks(3) {
            write!(
                d,
                "C{},{} {},{} {},{} ",
                fmt_coord(chunk[0].0 + offset_x),
                fmt_coord(chunk[0].1 + offset_y),
                fmt_coord(chunk[1].0 + offset_x),
                fmt_coord(chunk[1].1 + offset_y),
                fmt_coord(chunk[2].0 + offset_x),
                fmt_coord(chunk[2].1 + offset_y),
            )
            .unwrap();
        }
    } else {
        for &(x, y) in rest {
            write!(
                d,
                "L{},{} ",
                fmt_coord(x + offset_x),
                fmt_coord(y + offset_y),
            )
            .unwrap();
        }
    }
    // Edge paths come from Graphviz SVG which doesn't add trailing space
    // (unlike SvgGraphics glyph paths which do). Trim to match.
    let d = d.trim_end().to_string();
    d
}

fn is_cubic_edge_path(points: &[(f64, f64)]) -> bool {
    points.len() >= 4 && (points.len() - 1) % 3 == 0
}

fn edge_start_angle(points: &[(f64, f64)]) -> f64 {
    let (x1, y1) = points[0];
    let (x2, y2) = if is_cubic_edge_path(points) {
        let (cx, cy) = points[1];
        if (cx - x1).abs() > f64::EPSILON || (cy - y1).abs() > f64::EPSILON {
            (cx, cy)
        } else {
            points[3]
        }
    } else {
        points.get(1).copied().unwrap_or((x1 + 1.0, y1))
    };
    (y2 - y1).atan2(x2 - x1)
}

fn edge_end_angle(points: &[(f64, f64)]) -> f64 {
    let &(x2, y2) = points.last().unwrap();
    let (x1, y1) = if is_cubic_edge_path(points) {
        let (cx, cy) = points[points.len() - 2];
        if (x2 - cx).abs() > f64::EPSILON || (y2 - cy).abs() > f64::EPSILON {
            (cx, cy)
        } else {
            points[points.len() - 4]
        }
    } else {
        points
            .get(points.len().saturating_sub(2))
            .copied()
            .unwrap_or((x2 - 1.0, y2))
    };
    (y2 - y1).atan2(x2 - x1)
}

pub(super) fn move_edge_start_point(points: &mut Vec<(f64, f64)>, dx: f64, dy: f64) {
    if points.is_empty() {
        return;
    }

    let move_len = (dx * dx + dy * dy).sqrt();
    if is_cubic_edge_path(points) && points.len() >= 7 {
        let first_seg_len =
            ((points[3].0 - points[0].0).powi(2) + (points[3].1 - points[0].1).powi(2)).sqrt();
        if move_len >= first_seg_len {
            let next_dx = dx - (points[3].0 - points[0].0);
            let next_dy = dy - (points[3].1 - points[0].1);
            points.drain(0..3);
            move_edge_start_point(points, next_dx, next_dy);
            return;
        }
    }

    points[0].0 += dx;
    points[0].1 += dy;
    if is_cubic_edge_path(points) {
        points[1].0 += dx;
        points[1].1 += dy;
    }
}

pub(super) fn move_edge_end_point(points: &mut [(f64, f64)], dx: f64, dy: f64) {
    if points.is_empty() {
        return;
    }

    let last = points.len() - 1;
    points[last].0 += dx;
    points[last].1 += dy;
    if is_cubic_edge_path(points) {
        points[last - 1].0 += dx;
        points[last - 1].1 += dy;
    }
}

pub(super) fn emit_arrowhead(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    head: &ArrowHead,
    points: &[(f64, f64)],
    is_start: bool,
    link_color: &str,
    edge_offset_x: f64,
    edge_offset_y: f64,
) {
    if points.is_empty() || *head == ArrowHead::None {
        return;
    }

    let (tip_x, tip_y) = if is_start {
        (points[0].0 + edge_offset_x, points[0].1 + edge_offset_y)
    } else {
        let (x, y) = points[points.len() - 1];
        (x + edge_offset_x, y + edge_offset_y)
    };

    let base_angle = if is_start {
        edge_start_angle(points) + std::f64::consts::PI
    } else {
        edge_end_angle(points)
    };
    // Java extremity factories snap near-cardinal angles before drawing.
    let base_angle = crate::svek::extremity::manage_round(base_angle);

    match head {
        ArrowHead::Arrow => emit_rotated_polygon(
            sg,
            tracker,
            &[
                (0.0, 0.0),
                (-9.0, -4.0),
                (-5.0, 0.0),
                (-9.0, 4.0),
                (0.0, 0.0),
            ],
            base_angle,
            tip_x,
            tip_y,
            link_color,
            link_color,
        ),
        ArrowHead::Triangle => emit_rotated_polygon(
            sg,
            tracker,
            // Java class-path `LinkDecor.EXTENDS` uses `ExtremityFactoryTriangle`
            // in the complete extremity chain: xWing=18, yAperture=6.
            &[(0.0, 0.0), (-18.0, -6.0), (-18.0, 6.0), (0.0, 0.0)],
            base_angle,
            tip_x,
            tip_y,
            "none",
            link_color,
        ),
        ArrowHead::Diamond => emit_rotated_polygon(
            sg,
            tracker,
            &[
                (0.0, 0.0),
                (-6.0, -4.0),
                (-12.0, 0.0),
                (-6.0, 4.0),
                (0.0, 0.0),
            ],
            base_angle,
            tip_x,
            tip_y,
            link_color,
            link_color,
        ),
        ArrowHead::DiamondHollow => emit_rotated_polygon(
            sg,
            tracker,
            &[
                (0.0, 0.0),
                (-6.0, -4.0),
                (-12.0, 0.0),
                (-6.0, 4.0),
                (0.0, 0.0),
            ],
            base_angle,
            tip_x,
            tip_y,
            "none",
            link_color,
        ),
        ArrowHead::Plus => emit_plus_head(sg, tracker, tip_x, tip_y, base_angle, link_color),
        ArrowHead::None => {}
    }
}

fn emit_rotated_polygon(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    points: &[(f64, f64)],
    angle: f64,
    tx: f64,
    ty: f64,
    fill: &str,
    stroke: &str,
) {
    let cos = angle.cos();
    let sin = angle.sin();
    let mut pts = String::new();
    let mut rotated_points = Vec::with_capacity(points.len());
    for (idx, &(x, y)) in points.iter().enumerate() {
        if idx > 0 {
            pts.push(',');
        }
        let rx = x * cos - y * sin + tx;
        let ry = x * sin + y * cos + ty;
        write!(pts, "{},{}", fmt_coord(rx), fmt_coord(ry)).unwrap();
        rotated_points.push((rx, ry));
    }
    PolygonShape {
        points: {
            let mut flat = Vec::with_capacity(rotated_points.len() * 2);
            for &(rx, ry) in &rotated_points {
                flat.push(rx);
                flat.push(ry);
            }
            flat
        },
    }
    .draw(sg, &DrawStyle::filled(fill, stroke, 1.0));
    tracker.track_polygon(&rotated_points);
}

pub(super) fn emit_plus_head(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    tip_x: f64,
    tip_y: f64,
    angle: f64,
    link_color: &str,
) {
    let radius = 8.0;
    let center_x = tip_x - radius * angle.cos();
    let center_y = tip_y - radius * angle.sin();
    let cross_angle = angle - std::f64::consts::FRAC_PI_2;
    EllipseShape {
        cx: center_x,
        cy: center_y,
        rx: radius,
        ry: radius,
    }
    .draw(sg, &DrawStyle::filled("#FFFFFF", link_color, 1.0));
    tracker.track_ellipse(center_x, center_y, radius, radius);

    let p1 = point_on_circle(
        center_x,
        center_y,
        radius,
        cross_angle - std::f64::consts::FRAC_PI_2,
    );
    let p2 = point_on_circle(
        center_x,
        center_y,
        radius,
        cross_angle + std::f64::consts::FRAC_PI_2,
    );
    let p3 = point_on_circle(center_x, center_y, radius, cross_angle);
    let p4 = point_on_circle(
        center_x,
        center_y,
        radius,
        cross_angle + std::f64::consts::PI,
    );
    LineShape {
        x1: p1.0,
        y1: p1.1,
        x2: p2.0,
        y2: p2.1,
    }
    .draw(sg, &DrawStyle::outline(link_color, 1.0));
    tracker.track_line(p1.0, p1.1, p2.0, p2.1);
    LineShape {
        x1: p3.0,
        y1: p3.1,
        x2: p4.0,
        y2: p4.1,
    }
    .draw(sg, &DrawStyle::outline(link_color, 1.0));
    tracker.track_line(p3.0, p3.1, p4.0, p4.1);
}

fn point_on_circle(cx: f64, cy: f64, radius: f64, angle: f64) -> (f64, f64) {
    (cx + radius * angle.cos(), cy + radius * angle.sin())
}

/// Alignment type for a link label line segment.
#[derive(Clone, Copy, PartialEq)]
enum LabelAlign {
    Center,
    Left,
    Right,
}

fn edge_side_label_origin<'a>(
    layout: &GraphLayout,
    text: Option<&'a str>,
    xy: Option<(f64, f64)>,
    edge_offset_x: f64,
    edge_offset_y: f64,
) -> Option<(&'a str, f64, f64)> {
    let text = text?;
    let (x, y) = xy?;
    Some((
        text,
        x + layout.move_delta.0 - layout.normalize_offset.0 + edge_offset_x,
        y + layout.move_delta.1 - layout.normalize_offset.1 + edge_offset_y,
    ))
}

/// Split a link label on `\n`, `\l`, `\r` break sequences.
///
/// Returns `(line_text, alignment)` pairs.  The alignment is determined by the
/// break character that *follows* the text: `\n` → center, `\l` → left,
/// `\r` → right.  The last segment (with no trailing break) defaults to center.
fn split_label_lines(text: &str) -> Vec<(String, LabelAlign)> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut buf = String::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => {
                    result.push((buf.clone(), LabelAlign::Center));
                    buf.clear();
                    i += 2;
                    continue;
                }
                'l' => {
                    result.push((buf.clone(), LabelAlign::Left));
                    buf.clear();
                    i += 2;
                    continue;
                }
                'r' => {
                    result.push((buf.clone(), LabelAlign::Right));
                    buf.clear();
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() || result.is_empty() {
        // Last segment: inherit alignment from previous segments, default center
        let align = result.last().map(|(_, a)| *a).unwrap_or(LabelAlign::Center);
        result.push((buf, align));
    }
    result
}

fn kal_block_dimensions(text: &str) -> (f64, f64) {
    let font_family = "SansSerif";
    let font_size = 14.0;
    let lines = split_label_lines(text);
    let max_width = lines
        .iter()
        .map(|(t, _)| font_metrics::text_width(t, font_family, font_size, false, false))
        .fold(0.0_f64, f64::max);
    let height =
        lines.len() as f64 * font_metrics::line_height(font_family, font_size, false, false);
    (max_width + 4.0, height + 2.0)
}

fn kal_origin(
    anchor_x: f64,
    anchor_y: f64,
    width: f64,
    height: f64,
    pos: KalPosition,
) -> (f64, f64) {
    match pos {
        KalPosition::Right => (anchor_x, anchor_y - height / 2.0),
        KalPosition::Left => (anchor_x - width + 0.5, anchor_y - height / 2.0),
        KalPosition::Down => (anchor_x - width / 2.0, anchor_y),
        KalPosition::Up => (anchor_x - width / 2.0, anchor_y - height + 0.5),
    }
}

fn kal_position_for_link(link: &Link, endpoint: QualifierEndpoint) -> Option<KalPosition> {
    match endpoint {
        QualifierEndpoint::Tail => {
            if link.from_qualifier.is_none() {
                None
            } else if link.arrow_len == 1 {
                Some(KalPosition::Right)
            } else {
                Some(KalPosition::Down)
            }
        }
        QualifierEndpoint::Head => {
            if link.to_qualifier.is_none() {
                None
            } else if link.arrow_len == 1 {
                Some(KalPosition::Left)
            } else {
                Some(KalPosition::Up)
            }
        }
    }
}

pub(super) fn qualifier_edge_translation(
    link: &Link,
    endpoint: QualifierEndpoint,
    placement: &KalPlacement,
) -> Option<(f64, f64)> {
    let pos = kal_position_for_link(link, endpoint)?;
    let mut dx = 0.0;
    let mut dy = 0.0;

    match endpoint {
        QualifierEndpoint::Tail => {
            // Java Kal.moveX() only moves the start point for kal1/entity1.
            if matches!(pos, KalPosition::Up | KalPosition::Down) {
                dx += placement.shift_x;
            }
            if link.left_head != ArrowHead::None {
                match pos {
                    KalPosition::Right => dx += placement.width,
                    KalPosition::Left => dx -= placement.width,
                    KalPosition::Down => dy += placement.height,
                    KalPosition::Up => dy -= placement.height,
                }
            }
        }
        QualifierEndpoint::Head => {
            if link.right_head != ArrowHead::None {
                match pos {
                    KalPosition::Right => dx += placement.width,
                    KalPosition::Left => dx -= placement.width,
                    KalPosition::Down => dy += placement.height,
                    KalPosition::Up => dy -= placement.height,
                }
            }
        }
    }

    if dx.abs() <= f64::EPSILON && dy.abs() <= f64::EPSILON {
        None
    } else {
        Some((dx, dy))
    }
}

fn compute_qualifier_placements(
    cd: &ClassDiagram,
    layout: &GraphLayout,
    edge_offset_x: f64,
    edge_offset_y: f64,
) -> HashMap<QualifierKey, KalPlacement> {
    #[derive(Debug, Clone)]
    struct PendingKal {
        key: QualifierKey,
        entity: String,
        pos: KalPosition,
        orig_x: f64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }

    let mut pending = Vec::new();
    for (link_idx, link) in cd.links.iter().enumerate() {
        let Some(edge) = layout.edges.get(link_idx) else {
            continue;
        };
        let Some(&(sx, sy)) = edge.points.first() else {
            continue;
        };
        let Some(&(ex, ey)) = edge.points.last() else {
            continue;
        };

        if let (Some(text), Some(pos)) = (
            link.from_qualifier.as_deref(),
            kal_position_for_link(link, QualifierEndpoint::Tail),
        ) {
            let (width, height) = kal_block_dimensions(text);
            let (x, y) = kal_origin(sx + edge_offset_x, sy + edge_offset_y, width, height, pos);
            pending.push(PendingKal {
                key: QualifierKey {
                    link_idx,
                    endpoint: QualifierEndpoint::Tail,
                },
                entity: link.from.clone(),
                pos,
                orig_x: x,
                x,
                y,
                width,
                height,
            });
        }

        if let (Some(text), Some(pos)) = (
            link.to_qualifier.as_deref(),
            kal_position_for_link(link, QualifierEndpoint::Head),
        ) {
            let (width, height) = kal_block_dimensions(text);
            let (x, y) = kal_origin(ex + edge_offset_x, ey + edge_offset_y, width, height, pos);
            pending.push(PendingKal {
                key: QualifierKey {
                    link_idx,
                    endpoint: QualifierEndpoint::Head,
                },
                entity: link.to.clone(),
                pos,
                orig_x: x,
                x,
                y,
                width,
                height,
            });
        }
    }

    let mut grouped: HashMap<(String, KalPosition), Vec<usize>> = HashMap::new();
    for (idx, item) in pending.iter().enumerate() {
        if matches!(item.pos, KalPosition::Up | KalPosition::Down) {
            grouped
                .entry((item.entity.clone(), item.pos))
                .or_default()
                .push(idx);
        }
    }

    for indices in grouped.values() {
        if indices.len() < 2 {
            continue;
        }
        let mut los = LineOfSegments::new();
        for idx in indices {
            let item = &pending[*idx];
            los.add_segment(item.x - 5.0, item.x + item.width + 5.0);
        }
        let resolved = los.solve_overlaps();
        for (order, idx) in indices.iter().enumerate() {
            let item = &mut pending[*idx];
            let old_x1 = item.x - 5.0;
            let dx = resolved[order] - old_x1;
            item.x += dx;
        }
    }

    pending
        .into_iter()
        .map(|item| {
            (
                item.key,
                KalPlacement {
                    x: item.x,
                    y: item.y,
                    width: item.width,
                    height: item.height,
                    shift_x: item.x - item.orig_x,
                },
            )
        })
        .collect()
}

fn draw_kal_box(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    text: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    skin: &SkinParams,
) {
    let font_family = "SansSerif";
    let font_size = 14.0;
    let line_height = font_metrics::line_height(font_family, font_size, false, false);
    let ascent = font_metrics::ascent(font_family, font_size, false, false);
    let fill = skin.background_color("class", "#F1F1F1");
    let border = skin.border_color("class", "#181818");
    let default_font = get_default_font_family_pub();

    RectShape {
        x,
        y,
        w: width,
        h: height,
        rx: 0.0,
        ry: 0.0,
    }
    .draw(sg, &DrawStyle::filled(fill, border, 0.5));
    tracker.track_rect(x, y, width, height);

    for (idx, (line_text, _)) in split_label_lines(text).iter().enumerate() {
        let text_x = x + 2.0;
        let text_y = y + 1.0 + ascent + idx as f64 * line_height;
        let text_w = font_metrics::text_width(line_text, font_family, font_size, false, false);
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            line_text,
            text_x,
            text_y,
            Some(&default_font),
            font_size,
            None,
            None,
            None,
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        tracker.track_text(text_x, text_y, text_w, line_height);
    }
}

fn draw_edge_label_block(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    text: &str,
    x: f64,
    y: f64,
    block_wh: Option<(f64, f64)>,
    margin: f64,
    font_size: f64,
    boxed: bool,
    skin: &SkinParams,
) {
    let font_family = "SansSerif";
    let lines = split_label_lines(text);
    let line_height = font_metrics::line_height(font_family, font_size, false, false);
    let ascent = font_metrics::ascent(font_family, font_size, false, false);
    let widths: Vec<f64> = lines
        .iter()
        .map(|(t, _)| font_metrics::text_width(t, font_family, font_size, false, false))
        .collect();
    let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
    let outer_width = block_wh.map(|(w, _)| w).unwrap_or(max_width + 2.0 * margin);
    let outer_height = block_wh
        .map(|(_, h)| h)
        .unwrap_or(lines.len() as f64 * line_height + 2.0 * margin);

    if boxed {
        let fill = skin.background_color("class", "#F1F1F1");
        let border = skin.border_color("class", "#181818");
        RectShape {
            x,
            y,
            w: outer_width,
            h: outer_height,
            rx: 0.0,
            ry: 0.0,
        }
        .draw(sg, &DrawStyle::filled(fill, border, 0.5));
        tracker.track_rect(x, y, outer_width, outer_height);
    } else if let Some((bw, bh)) = block_wh {
        tracker.track_empty(x, y, bw, bh);
    }

    // Java: TextBlockMarged translates by (left=margin, top=margin) before
    // drawing the inner text block.  For boxed labels the margin is the box
    // inset; for non-boxed center edge labels (margin=1) this produces the
    // +1 px shift that addVisibilityModifier's TextBlockMarged applies.
    let base_x = x + margin;
    let base_y = y + margin + ascent;
    let align_width = if boxed {
        (outer_width - 2.0 * margin).max(max_width)
    } else {
        max_width
    };
    let default_font = get_default_font_family_pub();

    for (idx, (line_text, align)) in lines.iter().enumerate() {
        let text_w = widths[idx];
        let line_x = if boxed {
            base_x
        } else {
            match align {
                LabelAlign::Left => base_x,
                LabelAlign::Center => base_x + (align_width - text_w) / 2.0,
                LabelAlign::Right => base_x + (align_width - text_w),
            }
        };
        let line_y = base_y + idx as f64 * line_height;
        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            line_text,
            line_x,
            line_y,
            Some(&default_font),
            font_size,
            None,
            None,
            None,
            text_w,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        tracker.track_text(line_x, line_y, text_w, line_height);
    }
}

/// Render a link label.
///
/// Java PlantUML renders multiline labels (`\n`, `\l`, `\r`) as separate
/// `<text>` elements with font-size 13.  Alignment is per-line:
/// - `\n` = center-aligned (each line centered relative to the widest)
/// - `\l` = left-aligned   (all lines at the same left x)
/// - `\r` = right-aligned  (all lines right-aligned to the widest)
fn draw_label(sg: &mut SvgGraphic, text: &str, x: f64, y: f64) {
    let lines = split_label_lines(text);
    let font_family = "SansSerif";
    let font_size = LINK_LABEL_FONT_SIZE;
    let line_height = font_metrics::line_height(font_family, font_size, false, false);

    // Compute text widths for each line
    let widths: Vec<f64> = lines
        .iter()
        .map(|(t, _)| font_metrics::text_width(t, font_family, font_size, false, false))
        .collect();
    let max_width = widths.iter().cloned().fold(0.0_f64, f64::max);

    // Total block height
    let total_height = lines.len() as f64 * line_height;

    // Base x: left edge of the label block, positioned so the block center is at x
    let base_x = x + 1.0; // Java PlantUML offsets label 1px to the right of edge midpoint

    // Base y: center the label block vertically at y, then add ascent for first baseline
    let ascent = font_metrics::ascent(font_family, font_size, false, false);
    let base_y = y - total_height / 2.0 + ascent;

    let default_font = get_default_font_family_pub();

    for (idx, (line_text, align)) in lines.iter().enumerate() {
        let text_w = widths[idx];
        let line_x = match align {
            LabelAlign::Left => base_x,
            LabelAlign::Center => base_x + (max_width - text_w) / 2.0,
            LabelAlign::Right => base_x + (max_width - text_w),
        };
        let line_y = base_y + idx as f64 * line_height;

        sg.set_fill_color(TEXT_COLOR);
        sg.svg_text(
            line_text,
            line_x,
            line_y,
            Some(&default_font),
            font_size,
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
}

/// Draw a note in class diagrams (yellow sticky box with folded corner).
///
/// For left/right positioned notes with connectors (Opale style), the connector
/// arrow is integrated into the body path shape, matching Java Opale rendering.
fn draw_class_note(
    sg: &mut SvgGraphic,
    tracker: &mut BoundsTracker,
    note: &ClassNoteLayout,
    offset_x: f64,
    offset_y: f64,
) {
    let x = note.x + offset_x;
    let y = note.y + offset_y;
    let w = note.width;
    let h = note.height;

    // All four positions (left/right/top/bottom) use the Java Opale path
    // style for class-diagram notes.  Floating notes (no position) still
    // render as a simple polygon.
    let is_opale = matches!(note.position.as_str(), "left" | "right" | "top" | "bottom");
    let fold = if is_opale { CLASS_NOTE_FOLD } else { NOTE_FOLD };

    // Java Opale uses delta=4 for the connector arrow half-width on the body edge.
    const OPALE_DELTA: f64 = 4.0;

    if let Some((from_x_g, from_y_g, to_x_g, to_y_g)) = note.connector.filter(|_| is_opale) {
        // Opale note with connector: render body as <path> with embedded connector arrow.
        let from_x = from_x_g + offset_x;
        let from_y = from_y_g + offset_y;
        let to_x = to_x_g + offset_x;
        let to_y = to_y_g + offset_y;
        let pp1_x_local = from_x - x;
        let pp1_y_local = from_y - y;
        let pp2_x_local = to_x - x;
        let pp2_y_local = to_y - y;

        let mut d = String::with_capacity(512);
        match note.position.as_str() {
            "left" => {
                // Note is left of entity -> connector points RIGHT
                let mut y1 = pp1_y_local - OPALE_DELTA;
                y1 = y1.max(fold).min(h - 2.0 * OPALE_DELTA);

                write!(d, "M{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x + w),
                    fmt_coord(y + y1 + 2.0 * OPALE_DELTA)
                )
                .unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x + pp2_x_local),
                    fmt_coord(y + pp2_y_local)
                )
                .unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + y1)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + y1)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w - fold), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y)).unwrap();
            }
            "right" => {
                // Note is right of entity -> connector points LEFT
                let mut y1 = pp1_y_local - OPALE_DELTA;
                y1 = y1.max(0.0).min(h - 2.0 * OPALE_DELTA);

                write!(d, "M{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + y1)).unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x + pp2_x_local),
                    fmt_coord(y + pp2_y_local)
                )
                .unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x),
                    fmt_coord(y + y1 + 2.0 * OPALE_DELTA)
                )
                .unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + fold)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w - fold), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y)).unwrap();
            }
            "top" => {
                // Note is above entity -> ear on bottom edge pointing DOWN.
                // Java Opale.getPolygonDown: x1 = pp1.x - delta, clamped to [0, width].
                // The ear base spans [x1, x1 + 2*delta] along the note bottom edge.
                let mut x1 = pp1_x_local - OPALE_DELTA;
                x1 = x1.max(0.0).min(w - 2.0 * OPALE_DELTA);

                write!(d, "M{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + x1), fmt_coord(y + h)).unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x + pp2_x_local),
                    fmt_coord(y + pp2_y_local)
                )
                .unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x + x1 + 2.0 * OPALE_DELTA),
                    fmt_coord(y + h)
                )
                .unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + fold)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w - fold), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y)).unwrap();
            }
            "bottom" => {
                // Note is below entity -> ear on top edge pointing UP.
                // Java Opale.getPolygonUp: x1 = pp1.x - delta,
                // clamped to [0, width - cornersize] so the ear never overlaps the fold.
                let mut x1 = pp1_x_local - OPALE_DELTA;
                x1 = x1.max(0.0).min(w - fold);

                write!(d, "M{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x + w), fmt_coord(y + h)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w), fmt_coord(y + fold)).unwrap();
                write!(d, " L{},{}", fmt_coord(x + w - fold), fmt_coord(y)).unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x + x1 + 2.0 * OPALE_DELTA),
                    fmt_coord(y)
                )
                .unwrap();
                write!(
                    d,
                    " L{},{}",
                    fmt_coord(x + pp2_x_local),
                    fmt_coord(y + pp2_y_local)
                )
                .unwrap();
                write!(d, " L{},{}", fmt_coord(x + x1), fmt_coord(y)).unwrap();
                write!(d, " L{},{}", fmt_coord(x), fmt_coord(y)).unwrap();
                write!(d, " A0,0 0 0 0 {},{}", fmt_coord(x), fmt_coord(y)).unwrap();
            }
            _ => unreachable!(),
        }
        sg.push_raw(&format!(
            r#"<path d="{d}" fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ));
        let all_x = [x, x + w, to_x];
        let all_y = [y, y + h, to_y];
        tracker.track_path_bounds(
            all_x.iter().copied().fold(f64::INFINITY, f64::min),
            all_y.iter().copied().fold(f64::INFINITY, f64::min),
            all_x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            all_y.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        );
    } else if is_opale {
        // Opale note without connector: normal polygon as <path>
        let d = format!(
            "M{},{} L{},{} L{},{} L{},{} L{},{} L{},{}",
            fmt_coord(x),
            fmt_coord(y),
            fmt_coord(x),
            fmt_coord(y + h),
            fmt_coord(x + w),
            fmt_coord(y + h),
            fmt_coord(x + w),
            fmt_coord(y + fold),
            fmt_coord(x + w - fold),
            fmt_coord(y),
            fmt_coord(x),
            fmt_coord(y),
        );
        sg.push_raw(&format!(
            r#"<path d="{d}" fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
            bg = NOTE_BG,
            border = NOTE_BORDER,
        ));
        tracker.track_path_bounds(x, y, x + w, y + h);
    } else {
        // Non-opale note: use <polygon>
        let note_poly = [
            (x, y),
            (x + w - fold, y),
            (x + w, y + fold),
            (x + w, y + h),
            (x, y + h),
        ];
        PolygonShape {
            points: vec![
                note_poly[0].0,
                note_poly[0].1,
                note_poly[1].0,
                note_poly[1].1,
                note_poly[2].0,
                note_poly[2].1,
                note_poly[3].0,
                note_poly[3].1,
                note_poly[4].0,
                note_poly[4].1,
            ],
        }
        .draw(sg, &DrawStyle::filled(NOTE_BG, NOTE_BORDER, 1.0));
        tracker.track_polygon(&note_poly);
    }

    // Fold corner triangle
    {
        let fx = fmt_coord(x + w - fold);
        let fy_top = fmt_coord(y);
        let fy_bot = fmt_coord(y + fold);
        let fx_right = fmt_coord(x + w);
        if is_opale {
            // Opale fold: (w-fold,0) -> (w-fold,fold) -> (w,fold) matching Java Opale.getCorner
            sg.push_raw(&format!(
                r#"<path d="M{fx},{fy_top} L{fx},{fy_bot} L{fx_right},{fy_bot} L{fx},{fy_top}" fill="{bg}" style="stroke:{border};stroke-width:0.5;"/>"#,
                bg = NOTE_BG,
                border = NOTE_BORDER,
            ));
        } else {
            // Non-opale fold: existing shape (w-fold,0) -> (w-fold,fold) -> (w,0)
            sg.push_raw(&format!(
                r#"<path d="M{fx},{fy_top} L{fx},{fy_bot} L{fx_right},{fy_top} Z " fill="{bg}" style="stroke:{border};stroke-width:1;"/>"#,
                bg = NOTE_BG,
                border = NOTE_BORDER,
            ));
        }
        tracker.track_path_bounds(x + w - fold, y, x + w, y + fold);
    }

    // text content -- Java Opale: marginX1=6, marginY=5, font 13pt SansSerif
    const NOTE_MARGIN_Y: f64 = 5.0;
    const NOTE_FONT_SIZE: f64 = 13.0;
    const NOTE_ASCENT: f64 = 1901.0 / 2048.0 * 13.0; // 12.0669
    const NOTE_LINE_HT: f64 = 15.1328; // SansSerif 13pt: ascent+descent

    // Creole section titles (`==title==`) inside note bodies render as
    // horizontal lines spanning the note content width (1px inset on each
    // side) with the title centered between the two half-lines.
    set_section_title_bounds(SectionTitleBounds {
        x_start: x + 1.0,
        x_end: x + w - 1.0,
        stroke: NOTE_BORDER.to_string(),
    });

    let text_x = x + NOTE_TEXT_PADDING;
    if let Some(ref emb) = note.embedded {
        // Embedded diagram: Java emits image first, then text elements.
        let mut cursor_y = y + NOTE_MARGIN_Y;

        // Calculate before-text y and advance cursor, but defer emitting
        let before_text_y = cursor_y + NOTE_ASCENT;
        let before_text = if !emb.text_before.is_empty() {
            let mut tmp = String::new();
            let before_lines = render_creole_text(
                &mut tmp,
                &emb.text_before,
                text_x,
                before_text_y,
                NOTE_LINE_HT,
                TEXT_COLOR,
                None,
                &format!(r#"font-size="{}""#, NOTE_FONT_SIZE as u32),
            );
            cursor_y += before_lines as f64 * NOTE_LINE_HT;
            Some(tmp)
        } else {
            None
        };

        // Java note body rendering order with embedded diagrams:
        // 1. Blank &#160; lines from before-text (if any)
        // 2. <image> element
        // 3. After-text &#160; lines (if trailing newline after }})
        // 4. Heading separator lines from before-text (if any)
        let has_blank_before = emb.text_before.starts_with('\n');
        let has_trailing = note.text.trim_end().ends_with("}}") && note.text.ends_with('\n');

        // Split before-text into blank prefix and heading suffix
        let (blank_part, heading_part) = if has_blank_before {
            if let Some(ref text) = before_text {
                let split_pos = text.find("<line").unwrap_or(text.len());
                (
                    Some(text[..split_pos].to_string()),
                    Some(text[split_pos..].to_string()),
                )
            } else {
                (None, None)
            }
        } else {
            (None, before_text.clone())
        };

        // 1. Blank before-text
        if let Some(ref blanks) = blank_part {
            if !blanks.is_empty() {
                sg.push_raw(blanks);
            }
        }

        // 2. Image
        sg.push_raw(&format!(
            r#"<image height="{}" width="{}" x="{}" xlink:href="{}" y="{}"/>"#,
            emb.height as u32,
            emb.width as u32,
            fmt_coord(text_x),
            emb.data_uri,
            fmt_coord(cursor_y),
        ));
        cursor_y += emb.height;

        // 3. After-text
        if !emb.text_after.is_empty() || has_trailing {
            let ty = cursor_y + NOTE_ASCENT;
            if emb.text_after.is_empty() {
                // Trailing blank line: emit a single &#160; spacer directly
                sg.push_raw(&format!(
                    r##"<text fill="{}" font-family="sans-serif" font-size="{}" lengthAdjust="spacing" textLength="4.1323" x="{}" y="{}">&#160;</text>"##,
                    TEXT_COLOR, NOTE_FONT_SIZE as u32, fmt_coord(text_x), fmt_coord(ty),
                ));
            } else {
                let mut tmp = String::new();
                render_creole_text(
                    &mut tmp,
                    &emb.text_after,
                    text_x,
                    ty,
                    NOTE_LINE_HT,
                    TEXT_COLOR,
                    None,
                    &format!(r#"font-size="{}""#, NOTE_FONT_SIZE as u32),
                );
                sg.push_raw(&tmp);
            }
        }

        // 4. Heading before-text (separator lines etc.)
        if let Some(ref heading) = heading_part {
            if !heading.is_empty() {
                sg.push_raw(heading);
            }
        } else if !has_blank_before {
            // No blank prefix: all before-text comes after image
            if let Some(text) = before_text {
                sg.push_raw(&text);
            }
        }
    } else {
        let text_y = y + NOTE_MARGIN_Y + NOTE_ASCENT;
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            &note.text,
            text_x,
            text_y,
            NOTE_LINE_HT,
            TEXT_COLOR,
            None,
            &format!(r#"font-size="{}""#, NOTE_FONT_SIZE as u32),
        );
        sg.push_raw(&tmp);
    }

    clear_section_title_bounds();

    // For non-opale notes, draw a separate dashed connector line.
    // Opale notes embed the connector arrow in the body path.
    if !is_opale {
        if let Some((from_x, from_y, to_x, to_y)) = note.connector {
            let lx1 = from_x + offset_x;
            let ly1 = from_y + offset_y;
            let lx2 = to_x + offset_x;
            let ly2 = to_y + offset_y;
            LineShape {
                x1: lx1,
                y1: ly1,
                x2: lx2,
                y2: ly2,
            }
            .draw(
                sg,
                &DrawStyle {
                    fill: None,
                    stroke: Some(NOTE_BORDER.into()),
                    stroke_width: 1.0,
                    dash_array: Some((5.0, 3.0)),
                    delta_shadow: 0.0,
                },
            );
            tracker.track_line(lx1, ly1, lx2, ly2);
        }
    }
}
