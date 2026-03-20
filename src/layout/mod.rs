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
pub mod sequence_teoz;
pub mod state;
pub mod timing;
pub mod usecase;
pub mod wbs;

pub use graphviz::{
    layout as layout_graph, layout_with_svek, ClassNoteLayout, EdgeLayout, GraphLayout,
    LayoutEdge, LayoutGraph, LayoutNode, NodeLayout, RankDir,
};

use std::collections::HashMap;

use crate::font_metrics;
use crate::model::{
    ClassDiagram, ClassHideShowRule, ClassPortion, ClassRuleTarget, Diagram, Direction, Entity,
    EntityKind, Member, Stereotype,
};
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

// ── Class entity sizing constants — sourced from Java PlantUML ───────
//
// All font metric values from Java AWT FontMetrics at full f64 precision.
// See tests/tools/ExtractFontMetrics.java for extraction method.

/// FontParam.CLASS = 12pt but EntityImageClassHeader renders name at 14pt.
const CLASS_FONT_SIZE: f64 = 14.0;
/// FontParam.CLASS_ATTRIBUTE = 10pt.
const CLASS_ATTR_FONT_SIZE: f64 = 10.0;
/// MethodsOrFieldsArea: empty compartment = margin_top(4) + margin_bottom(4).
const LINE_HEIGHT_PT: f64 = 8.0;
/// EntityImageClassHeader.java:150 — withMargin(circledChar, left=4, ...).
const CIRCLE_LEFT_PAD: f64 = 4.0;
/// SkinParam.circledCharacterRadius = 17/3+6 = 11. Diameter = 2 * 11 = 22.
const CIRCLE_DIAMETER: f64 = 22.0;
/// Gap between circle block right edge and name text left edge.
/// HeaderLayout: name block starts right after circle block (no explicit gap).
/// But EntityImageClassHeader name margin left=3, and circleBlock right margin=0.
/// So effective gap = name_margin_left(3). This 3 is the same as RIGHT_PAD.
const CIRCLE_TEXT_GAP: f64 = 3.0;
/// EntityImageClassHeader.java:105 — withMargin(name, 3, 3, 0, 0): right=3.
const RIGHT_PAD: f64 = 3.0;
/// HeaderLayout height = max(circleDim.h=32, ...) = 32.
const HEADER_HEIGHT_PT: f64 = 32.0;
/// MethodsOrFieldsArea: empty section = margin_top(4) + margin_bottom(4) = 8.
const EMPTY_COMPARTMENT: f64 = 8.0;
/// CircledChar block: diameter(22) + marginLeft(4) + marginRight(0) = 26.
const HEADER_CIRCLE_BLOCK_WIDTH: f64 = 26.0;
/// CircledChar block: diameter(22) + marginTop(5) + marginBottom(5) = 32.
const HEADER_CIRCLE_BLOCK_HEIGHT: f64 = 32.0;
/// SansSerif 14pt: ascent(12.995117) + descent(3.301758) = 16.296875.
const HEADER_NAME_BLOCK_HEIGHT: f64 = 16.296875;
/// Name margin: withMargin(name, 3, 3, 0, 0) → left(3) + right(3) = 6.
const HEADER_NAME_BLOCK_MARGIN_X: f64 = 6.0;
/// FontParam.CLASS_STEREOTYPE = 12pt.
const HEADER_STEREO_FONT_SIZE: f64 = 12.0;
/// SansSerif 12pt italic: ascent(11.138672) + descent(2.830078) = 13.96875.
const HEADER_STEREO_LINE_HEIGHT: f64 = 13.96875;
/// HeaderLayout.java:77 — height includes stereoDim.h + nameDim.h + 10 (gap).
const HEADER_STEREO_NAME_GAP: f64 = 10.0;
/// SansSerif 14pt height = 16.296875 (used for member row layout).
const MEMBER_ROW_HEIGHT: f64 = 16.296875;
/// margin_top(4) + MEMBER_ROW_HEIGHT(16.296875) + margin_bottom(4) = 24.296875.
const MEMBER_BLOCK_HEIGHT_ONE_ROW: f64 = 24.296875;
const MEMBER_TEXT_LEFT_WITH_ICON: f64 = 26.0;
const MEMBER_TEXT_LEFT_NO_ICON: f64 = 6.0;
/// VisibilityModifier.getUBlock: (size+1, size+1) where size = circledCharacterRadius - 1 = 10.
/// Block width = 11. Used when entity has a visibility modifier (e.g. -class foo).
const ENTITY_VIS_ICON_BLOCK_WIDTH: f64 = 11.0;

// -- Generic type box constants -- sourced from EntityImageClassHeader.java --
//
// EntityImageClassHeader.java:136-145: generic block =
//   text(12pt italic) + innerMargin(1,1) + TextBlockGeneric(dashed rect) + outerMargin(1,1)
// HeaderLayout.java:112: delta=4, xGeneric=width-genericDim.w+4, yGeneric=-4

/// Generic type text font size (FontParam.CLASS_STEREOTYPE = 12pt italic).
const GENERIC_FONT_SIZE: f64 = 12.0;
/// Inner margin around generic text (withMargin(genericBlock, 1, 1), line 139).
const GENERIC_INNER_MARGIN: f64 = 1.0;
/// Outer margin around TextBlockGeneric (withMargin(genericBlock, 1, 1), line 145).
const GENERIC_OUTER_MARGIN: f64 = 1.0;

// ── Object entity sizing constants — sourced from EntityImageObject.java ──
//
// EntityImageObject.java:98 — withMargin(tmp, 2, 2) → margin(top=2, right=2, bottom=2, left=2).
// EntityImageObject.java:228 — xMarginCircle = 5.
// EntityImageObject.java:110-112 — empty fields = TextBlockLineBefore(lineThickness,
//   TextBlockEmpty(10, 16)) → dim = (10, 16).

/// EntityImageObject.java:98 — name block margin (all sides).
const OBJ_NAME_MARGIN: f64 = 2.0;
/// EntityImageObject.java:228 — xMarginCircle = 5.
const OBJ_X_MARGIN_CIRCLE: f64 = 5.0;
/// EntityImageObject.java:112 — TextBlockEmpty(10, 16).height = 16.
const OBJ_EMPTY_BODY_HEIGHT: f64 = 16.0;
/// EntityImageObject.java:112 — TextBlockEmpty(10, 16).width = 10.
const OBJ_EMPTY_BODY_WIDTH: f64 = 10.0;

/// Perform layout on a Diagram
pub fn layout(diagram: &Diagram, skin: &crate::style::SkinParams) -> Result<DiagramLayout> {
    match diagram {
        Diagram::Class(cd) => {
            let gl = layout_class_diagram(cd, skin)?;
            Ok(DiagramLayout::Class(gl))
        }
        Diagram::Sequence(sd) => {
            let sl = if sd.teoz_mode {
                sequence_teoz::layout_sequence_teoz(sd, skin)?
            } else {
                sequence::layout_sequence(sd, skin)?
            };
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

/// Compute generic block outer dimension width (genericDim.width in Java).
/// Returns 0 when entity has no generic parameter.
fn generic_dim_width(entity: &Entity) -> f64 {
    match entity.generic {
        Some(ref g) => {
            let text_w = font_metrics::text_width(g, "SansSerif", GENERIC_FONT_SIZE, false, true);
            text_w + 2.0 * GENERIC_INNER_MARGIN + 2.0 * GENERIC_OUTER_MARGIN
        }
        None => 0.0,
    }
}

/// Estimate entity rendering size (width_pt, height_pt)
fn estimate_entity_size(cd: &ClassDiagram, entity: &Entity, member_row_h: f64) -> (f64, f64) {
    if matches!(
        entity.kind,
        EntityKind::Interface | EntityKind::Enum | EntityKind::Annotation
    ) {
        return estimate_entity_size_legacy(entity);
    }

    if entity.kind == EntityKind::Object {
        return estimate_object_size(entity);
    }

    // Entity name WITHOUT generic parameter -- generic is rendered separately
    let name_display = entity.name.clone();

    let visible_stereotypes = visible_stereotype_labels(&cd.hide_show_rules, &entity.stereotypes);
    let italic_name = entity.kind == EntityKind::Abstract;
    let name_width = font_metrics::text_width(
        &name_display,
        "SansSerif",
        CLASS_FONT_SIZE,
        false,
        italic_name,
    );
    let name_block_width = name_width + HEADER_NAME_BLOCK_MARGIN_X;
    let stereo_block_width = visible_stereotypes
        .iter()
        .map(|label| {
            let stereo_text = format!("\u{00AB}{label}\u{00BB}");
            font_metrics::text_width(
                &stereo_text,
                "SansSerif",
                HEADER_STEREO_FONT_SIZE,
                false,
                true,
            )
        })
        .fold(0.0_f64, f64::max);
    let vis_icon_w = if entity.visibility.is_some() { ENTITY_VIS_ICON_BLOCK_WIDTH } else { 0.0 };
    // HeaderLayout.java:74 -- width = circleDim.w + max(stereoDim.w, nameDim.w) + genericDim.w
    let gen_w = generic_dim_width(entity);
    let header_width = HEADER_CIRCLE_BLOCK_WIDTH + vis_icon_w + name_block_width.max(stereo_block_width) + gen_w;
    let stereo_height = visible_stereotypes.len() as f64 * HEADER_STEREO_LINE_HEIGHT;
    let header_height = HEADER_CIRCLE_BLOCK_HEIGHT
        .max(stereo_height + HEADER_NAME_BLOCK_HEIGHT + HEADER_STEREO_NAME_GAP);

    let visible_fields: Vec<&Member> = entity
        .members
        .iter()
        .filter(|m| !m.is_method)
        .filter(|_| show_portion(&cd.hide_show_rules, ClassPortion::Field, &entity.name))
        .collect();
    let visible_methods: Vec<&Member> = entity
        .members
        .iter()
        .filter(|m| m.is_method)
        .filter(|_| show_portion(&cd.hide_show_rules, ClassPortion::Method, &entity.name))
        .collect();

    let show_fields = show_portion(&cd.hide_show_rules, ClassPortion::Field, &entity.name);
    let show_methods = show_portion(&cd.hide_show_rules, ClassPortion::Method, &entity.name);

    let body_width =
        estimate_members_width(&visible_fields).max(estimate_members_width(&visible_methods));
    let body_height = section_height(show_fields, &visible_fields, member_row_h)
        + section_height(show_methods, &visible_methods, member_row_h);

    let width = header_width.max(body_width);
    let height = header_height + body_height;

    log::debug!(
        "estimate_entity_size: {} -> ({}, {})",
        entity.name,
        width,
        height
    );

    (width, height)
}

/// Estimate size for Object entities (EntityImageObject.java layout).
///
/// Object header: name with margin(2,2,2,2) centered, no circle icon.
/// Body: TextBlockLineBefore(lineThickness, TextBlockEmpty(10, 16)) for empty fields.
/// Width = max(bodyWidth, titleWidth + 2 * xMarginCircle).
/// Height = titleHeight + bodyHeight.
fn estimate_object_size(entity: &Entity) -> (f64, f64) {
    let name_width = font_metrics::text_width(
        &entity.name,
        "SansSerif",
        CLASS_FONT_SIZE,
        false,
        false,
    );
    // name block: text + margin(2, 2, 2, 2)
    let name_block_width = name_width + 2.0 * OBJ_NAME_MARGIN;
    let name_block_height = HEADER_NAME_BLOCK_HEIGHT + 2.0 * OBJ_NAME_MARGIN;

    // title dim = name dim (no stereotype)
    let title_width = name_block_width;
    let title_height = name_block_height;

    // body: empty fields = TextBlockEmpty(10, 16)
    let body_width = OBJ_EMPTY_BODY_WIDTH;
    let body_height = OBJ_EMPTY_BODY_HEIGHT;

    let width = body_width.max(title_width + 2.0 * OBJ_X_MARGIN_CIRCLE);
    let height = title_height + body_height;

    log::debug!(
        "estimate_object_size: {} -> ({}, {})",
        entity.name,
        width,
        height,
    );

    (width, height)
}

fn estimate_entity_size_legacy(entity: &Entity) -> (f64, f64) {
    // Entity name WITHOUT generic parameter -- generic is rendered separately
    let name_display = entity.name.clone();

    // check if a stereotype line is needed (interface / enum / abstract / custom stereotype)
    let has_stereotype_line = !entity.stereotypes.is_empty()
        || matches!(
            entity.kind,
            EntityKind::Interface | EntityKind::Enum | EntityKind::Abstract
        );

    // max stereotype text width (for width calculation)
    let stereotype_text_width = if has_stereotype_line {
        let kind_stereo_w = match entity.kind {
            EntityKind::Interface => font_metrics::text_width(
                "\u{00AB}interface\u{00BB}",
                "SansSerif",
                CLASS_FONT_SIZE,
                false,
                false,
            ),
            EntityKind::Enum => font_metrics::text_width(
                "\u{00AB}enum\u{00BB}",
                "SansSerif",
                CLASS_FONT_SIZE,
                false,
                false,
            ),
            EntityKind::Abstract => font_metrics::text_width(
                "\u{00AB}abstract\u{00BB}",
                "SansSerif",
                CLASS_FONT_SIZE,
                false,
                false,
            ),
            _ => 0.0,
        };
        let custom_stereo_w = entity
            .stereotypes
            .iter()
            .map(|s| {
                let stereo_text = format!("\u{00AB}{}\u{00BB}", s.0);
                font_metrics::text_width(&stereo_text, "SansSerif", CLASS_FONT_SIZE, false, false)
            })
            .fold(0.0_f64, f64::max);
        kind_stereo_w.max(custom_stereo_w)
    } else {
        0.0
    };

    // display text width for each member
    let max_member_width = entity
        .members
        .iter()
        .map(|m| {
            let mut member_text = String::new();
            if m.visibility.is_some() {
                member_text.push_str("+ "); // approximate visibility prefix
            }
            member_text.push_str(&m.name);
            if let Some(ref t) = m.return_type {
                member_text.push_str(": ");
                member_text.push_str(t);
            }
            font_metrics::text_width(&member_text, "SansSerif", CLASS_FONT_SIZE, false, false)
        })
        .fold(0.0_f64, f64::max);

    // Width: Java formula = circle_left_pad + circle_dia + gap + text_width + right_pad + generic
    let name_width =
        font_metrics::text_width(&name_display, "SansSerif", CLASS_FONT_SIZE, false, false);
    let gen_w = generic_dim_width(entity);
    let circle_plus_name =
        CIRCLE_LEFT_PAD + CIRCLE_DIAMETER + CIRCLE_TEXT_GAP + name_width + RIGHT_PAD + gen_w;
    let max_text_width = circle_plus_name
        .max(stereotype_text_width + CIRCLE_LEFT_PAD + RIGHT_PAD)
        .max(max_member_width + 2.0 * RIGHT_PAD);
    let width = max_text_width;

    // Height: Java formula = header(32) + fields_compartment + methods_compartment
    // Each compartment: empty=8, with N members = N * line_height + padding
    let _stereotype_extra = if has_stereotype_line {
        LINE_HEIGHT_PT
    } else {
        0.0
    };
    let fields_height = EMPTY_COMPARTMENT; // no field/method separation in our model yet
    let methods_height = if entity.members.is_empty() {
        EMPTY_COMPARTMENT
    } else {
        entity.members.len() as f64 * LINE_HEIGHT_PT + EMPTY_COMPARTMENT
    };
    let height = HEADER_HEIGHT_PT + fields_height + methods_height;

    log::debug!(
        "estimate_entity_size: {} -> ({}, {})",
        entity.name,
        width,
        height
    );

    (width, height)
}

fn estimate_members_width(members: &[&Member]) -> f64 {
    members
        .iter()
        .map(|m| {
            let text = member_text(m);
            let lines = split_member_lines(&text);
            let base_left = if m.visibility.is_some() {
                MEMBER_TEXT_LEFT_WITH_ICON
            } else {
                MEMBER_TEXT_LEFT_NO_ICON
            };
            lines
                .iter()
                .enumerate()
                .map(|(i, (line_text, indent))| {
                    let w = font_metrics::text_width(
                        line_text,
                        "SansSerif",
                        CLASS_FONT_SIZE,
                        false,
                        m.modifiers.is_abstract,
                    );
                    if i == 0 {
                        base_left + w
                    } else {
                        base_left + indent + w
                    }
                })
                .fold(0.0_f64, f64::max)
        })
        .fold(0.0_f64, f64::max)
}

fn section_height(show: bool, members: &[&Member], member_row_h: f64) -> f64 {
    if !show {
        return 0.0;
    }
    if members.is_empty() {
        return EMPTY_COMPARTMENT;
    }
    let total_visual_lines: usize = members.iter().map(|m| member_visual_lines(m)).sum();
    // Java: margin_top(4) + total_lines * member_row_height + margin_bottom(4)
    let one_row_h = member_row_h + 8.0;
    one_row_h + (total_visual_lines.saturating_sub(1)) as f64 * member_row_h
}

/// Java MemberImpl.getDisplay() format:
/// - Methods: "name(): type" (colon directly after parenthesis)
/// - Fields:  "name : type" (space-colon-space)
fn member_text(m: &Member) -> String {
    match &m.return_type {
        Some(t) if m.name.ends_with(')') => format!("{}: {t}", m.name),
        Some(t) => format!("{} : {t}", m.name),
        None => m.name.clone(),
    }
}

/// Count the number of visual lines a member occupies.
fn member_visual_lines(m: &Member) -> usize {
    let text = member_text(m);
    split_member_lines(&text).len()
}

/// Split member display text by literal `\n` sequences.
/// Returns a vec of (trimmed_text, leading_space_width_at_14pt).
/// The first line always has indent=0; continuation lines use the width
/// of the leading whitespace as an indent offset from the first line.
pub(crate) fn split_member_lines(text: &str) -> Vec<(String, f64)> {
    let parts: Vec<&str> = text.split("\\n").collect();
    let mut result = Vec::with_capacity(parts.len());
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            result.push((part.to_string(), 0.0));
        } else {
            let trimmed = part.trim_start();
            let leading = &part[..part.len() - trimmed.len()];
            let indent = font_metrics::text_width(
                leading,
                "SansSerif",
                CLASS_FONT_SIZE,
                false,
                false,
            );
            result.push((trimmed.to_string(), indent));
        }
    }
    result
}

fn show_portion(rules: &[ClassHideShowRule], portion: ClassPortion, entity_name: &str) -> bool {
    let mut result = true;
    for rule in rules {
        if rule.portion != portion {
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

fn visible_stereotype_labels(
    rules: &[ClassHideShowRule],
    stereotypes: &[Stereotype],
) -> Vec<String> {
    stereotypes
        .iter()
        .map(|st| st.0.clone())
        .filter(|label| stereotype_label_visible(rules, label))
        .collect()
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

/// Direction -> RankDir mapping
fn direction_to_rankdir(dir: &Direction) -> RankDir {
    match dir {
        Direction::TopToBottom => RankDir::TopToBottom,
        Direction::LeftToRight => RankDir::LeftToRight,
        Direction::BottomToTop => RankDir::BottomToTop,
        Direction::RightToLeft => RankDir::RightToLeft,
    }
}

/// Note font size
const NOTE_FONT_SIZE: f64 = 13.0;
const NOTE_LINE_HEIGHT: f64 = 16.0;
const NOTE_PADDING: f64 = 10.0;
/// Gap between note and target entity
const NOTE_GAP: f64 = 16.0;

/// Perform layout on a class diagram
fn layout_class_diagram(cd: &ClassDiagram, skin: &crate::style::SkinParams) -> Result<GraphLayout> {
    log::debug!(
        "layout_class_diagram: {} entities, {} links, {} notes",
        cd.entities.len(),
        cd.links.len(),
        cd.notes.len()
    );

    // Resolve member row height from skinparams.
    // Java default: FontParam.CLASS_ATTRIBUTE renders at 14pt (same as CLASS).
    // When classAttributeFontSize is explicitly set, use its line_height.
    let member_row_h: f64 = skin
        .get("classattributefontsize")
        .and_then(|s| s.parse::<f64>().ok())
        .map(|sz| font_metrics::line_height("SansSerif", sz, false, false))
        .unwrap_or(MEMBER_ROW_HEIGHT);

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
            let (w, h) = estimate_entity_size(cd, e, member_row_h);
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
    // Java: DotStringFactory uses minlen = link.getLength() - 1.
    // arrow_len=1 (single dash/dot) -> minlen=0 (same rank = horizontal).
    // arrow_len=2+ (double dash/dot) -> minlen=1+ (different ranks = vertical).
    let mut edges: Vec<LayoutEdge> = cd
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
                minlen: link.arrow_len.saturating_sub(1) as u32,
                invisible: false,
            }
        })
        .collect();

    let standalone_by_container = collect_standalone_square_edges(cd, &name_to_id);
    edges.extend(standalone_by_container);

    // Java: rankdir=LR is only emitted when `left to right direction` was explicitly written.
    // When direction is inferred from arrow length, rankdir stays TB (default) and
    // layout is controlled via edge minlen values.
    let rankdir = if cd.direction_explicit {
        direction_to_rankdir(&cd.direction)
    } else {
        RankDir::TopToBottom
    };

    let graph = LayoutGraph {
        nodes,
        edges,
        rankdir,
    };

    let mut layout = layout_with_svek(&graph)?;

    // Expand total_width/total_height to include edge label extents.
    // Java: LimitFinder.ensureVisible tracks all drawn elements including text.
    // Edge labels are drawn at the edge midpoint; their text can extend beyond nodes.
    let link_label_font_size = 13.0_f64; // Java: FontParam.CLASS uses 13pt for link labels
    for el in &layout.edges {
        if let Some(ref label) = el.label {
            if el.points.is_empty() {
                continue;
            }
            let mid_idx = el.points.len() / 2;
            let (mx, _my) = el.points[mid_idx];
            // Label is drawn at mx+1 (1px offset in draw_label), extending right
            let lines: Vec<&str> = label.split("\\n").flat_map(|s| s.split("\\l")).flat_map(|s| s.split("\\r")).collect();
            let max_line_w = lines
                .iter()
                .map(|l| font_metrics::text_width(l, "SansSerif", link_label_font_size, false, false))
                .fold(0.0_f64, f64::max);
            let label_right = mx + 1.0 + max_line_w;
            if label_right > layout.total_width {
                layout.total_width = label_right;
            }
        }
    }

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
            if let Some(ref raw_d) = e.raw_path_d {
                e.raw_path_d = Some(graphviz::transform_path_d(raw_d, shift_x, shift_y));
            }
            if let Some(ref mut pts) = e.arrow_polygon_points {
                for p in pts.iter_mut() {
                    p.0 += shift_x;
                    p.1 += shift_y;
                }
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

fn collect_standalone_square_edges(
    cd: &ClassDiagram,
    name_to_id: &HashMap<String, String>,
) -> Vec<LayoutEdge> {
    let mut result = Vec::new();

    let linked_entities: std::collections::HashSet<&str> = cd
        .links
        .iter()
        .flat_map(|link| [link.from.as_str(), link.to.as_str()])
        .collect();

    let grouped_entities: std::collections::HashSet<&str> = cd
        .groups
        .iter()
        .flat_map(|group| group.entities.iter().map(String::as_str))
        .collect();

    let root_standalones: Vec<&str> = cd
        .entities
        .iter()
        .map(|entity| entity.name.as_str())
        .filter(|name| !linked_entities.contains(name))
        .filter(|name| !grouped_entities.contains(name))
        .collect();
    result.extend(square_edges_for_entities(&root_standalones, name_to_id));

    for group in &cd.groups {
        let standalones: Vec<&str> = group
            .entities
            .iter()
            .map(String::as_str)
            .filter(|name| !linked_entities.contains(name))
            .collect();
        result.extend(square_edges_for_entities(&standalones, name_to_id));
    }

    result
}

fn square_edges_for_entities(
    entity_names: &[&str],
    name_to_id: &HashMap<String, String>,
) -> Vec<LayoutEdge> {
    if entity_names.len() < 3 {
        return Vec::new();
    }

    let branch = compute_square_branch(entity_names.len());
    let ids: Vec<String> = entity_names
        .iter()
        .map(|name| {
            name_to_id
                .get(*name)
                .cloned()
                .unwrap_or_else(|| sanitize_id(name))
        })
        .collect();

    let mut result = Vec::new();
    let mut head_branch = 0usize;
    for i in 1..ids.len() {
        let dist = i - head_branch;
        if dist == branch {
            result.push(LayoutEdge {
                from: ids[head_branch].clone(),
                to: ids[i].clone(),
                label: None,
                minlen: 1,
                invisible: true,
            });
            head_branch = i;
        } else {
            result.push(LayoutEdge {
                from: ids[i - 1].clone(),
                to: ids[i].clone(),
                label: None,
                minlen: 0,
                invisible: true,
            });
        }
    }

    result
}

fn compute_square_branch(size: usize) -> usize {
    let sqrt = (size as f64).sqrt() as usize;
    if sqrt * sqrt == size {
        sqrt
    } else {
        sqrt + 1
    }
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
            let max_line_width = lines
                .iter()
                .map(|l| font_metrics::text_width(l, "SansSerif", NOTE_FONT_SIZE, false, false))
                .fold(0.0_f64, f64::max);
            let note_width = (max_line_width + NOTE_PADDING * 2.0).max(60.0);
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
    use std::collections::HashMap;

    fn empty_entity(name: &str) -> Entity {
        Entity {
            name: name.to_string(),
            kind: EntityKind::Class,
            stereotypes: vec![],
            members: vec![],
            color: None,
            generic: None,
            source_line: None,
            visibility: None,
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

    fn empty_diagram() -> ClassDiagram {
        ClassDiagram {
            entities: vec![],
            links: vec![],
            groups: vec![],
            direction: Direction::TopToBottom,
            direction_explicit: false,
            notes: vec![],
            hide_show_rules: vec![],
            stereotype_backgrounds: HashMap::new(),
        }
    }

    #[test]
    fn estimate_size_empty_class_returns_minimum() {
        let e = empty_entity("Foo");
        let (w, h) = estimate_entity_size(&empty_diagram(), &e, MEMBER_ROW_HEIGHT);
        // Width = circle(4+22) + gap(3) + text_width("Foo",14) + pad(3) ≈ 57
        assert!(w >= 40.0, "width should be >= 40, got {w}");
        // Height = header(32) + fields(8) + methods(8) = 48
        assert!(h >= 48.0, "height should be >= 48, got {h}");
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
            source_line: None,
            visibility: None,
        };
        let (w, h) = estimate_entity_size(&empty_diagram(), &e, MEMBER_ROW_HEIGHT);

        // height = header(32) + fields(8) + members(2*8+8) = 64
        let expected_min_height =
            HEADER_HEIGHT_PT + EMPTY_COMPARTMENT + 2.0 * LINE_HEIGHT_PT + EMPTY_COMPARTMENT;
        assert!(
            h >= expected_min_height,
            "height {h} should be >= {expected_min_height}"
        );

        let member_text = "- longFieldNameHere: String";
        let expected_min_width = crate::font_metrics::text_width(
            member_text,
            "SansSerif",
            CLASS_ATTR_FONT_SIZE,
            false,
            false,
        ) + 2.0 * RIGHT_PAD;
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
            source_line: None,
            visibility: None,
        };
        let (_, h) = estimate_entity_size(&empty_diagram(), &e, MEMBER_ROW_HEIGHT);

        let expected_min = HEADER_HEIGHT_PT + 2.0 * EMPTY_COMPARTMENT;
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
        let diagram = empty_diagram();
        let (w_plain, _) = estimate_entity_size(&diagram, &plain, MEMBER_ROW_HEIGHT);
        let (w_generic, _) = estimate_entity_size(&diagram, &generic, MEMBER_ROW_HEIGHT);
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
