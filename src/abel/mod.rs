// abel - Entity/Link data model
// Port of Java PlantUML's net.sourceforge.plantuml.abel package
//
// The core data model shared by all diagram types: entities (leaves and
// groups), links between them, and associated metadata types.

pub mod entity;
pub mod group_type;
pub mod leaf_type;
pub mod link;

pub use entity::{Entity, EntityColors, NotePosition, VisibilityModifier};
pub use group_type::GroupType;
pub use leaf_type::LeafType;
pub use link::Link;

// Re-export EntityPosition and Together from svek::node where they live.
pub use crate::svek::node::{EntityPosition, Together};

// ── LinkArrow ────────────────────────────────────────────────────────

/// Direction hint for a link arrow label.
/// Java: `abel.LinkArrow`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LinkArrow {
    #[default]
    NoneOrSeveral,
    DirectNormal,
    Backward,
}

impl LinkArrow {
    /// Reverse the arrow direction.
    /// Java: `LinkArrow.reverse()`
    pub fn reverse(self) -> Self {
        match self {
            Self::DirectNormal => Self::Backward,
            Self::Backward => Self::DirectNormal,
            Self::NoneOrSeveral => Self::NoneOrSeveral,
        }
    }
}

// ── LinkArg ──────────────────────────────────────────────────────────

/// Construction arguments for a Link (label, length, quantifiers, etc.).
/// Java: `abel.LinkArg`
///
/// This is an immutable-ish builder: `with_*` methods return a new
/// copy; `set_length` and `set_visibility_modifier` mutate in place
/// (matching the Java API).
#[derive(Debug, Clone)]
pub struct LinkArg {
    label: Vec<String>,
    length: i32,
    quantifier1: Option<String>,
    quantifier2: Option<String>,
    role1: Option<String>,
    role2: Option<String>,
    label_distance: Option<String>,
    label_angle: Option<String>,
    kal1: Option<String>,
    kal2: Option<String>,
    visibility_modifier: Option<VisibilityModifier>,
}

impl LinkArg {
    /// Primary constructor.
    pub fn new(label: Vec<String>, length: i32) -> Self {
        Self {
            label,
            length,
            quantifier1: None,
            quantifier2: None,
            role1: None,
            role2: None,
            label_distance: None,
            label_angle: None,
            kal1: None,
            kal2: None,
            visibility_modifier: None,
        }
    }

    /// No-display constructor with only length.
    /// Java: `LinkArg.noDisplay(int)`
    pub fn no_display(length: i32) -> Self {
        Self::new(vec![], length)
    }

    /// Return a copy with quantifiers set.
    /// Java: `LinkArg.withQuantifier(String, String)`
    pub fn with_quantifier(
        mut self,
        quantifier1: Option<String>,
        quantifier2: Option<String>,
    ) -> Self {
        self.quantifier1 = quantifier1;
        self.quantifier2 = quantifier2;
        self
    }

    /// Return a copy with roles set.
    /// Java: `LinkArg.withRole(String, String)`
    pub fn with_role(mut self, role1: Option<String>, role2: Option<String>) -> Self {
        self.role1 = role1;
        self.role2 = role2;
        self
    }

    /// Return a copy with kal strings set.
    /// Java: `LinkArg.withKal(String, String)`
    pub fn with_kal(mut self, kal1: Option<String>, kal2: Option<String>) -> Self {
        self.kal1 = kal1;
        self.kal2 = kal2;
        self
    }

    /// Return a copy with label distance and angle set.
    /// Java: `LinkArg.withDistanceAngle(String, String)`
    pub fn with_distance_angle(
        mut self,
        label_distance: Option<String>,
        label_angle: Option<String>,
    ) -> Self {
        self.label_distance = label_distance;
        self.label_angle = label_angle;
        self
    }

    /// Return an inverted copy (swap quantifier1/2, role1/2, kal1/2).
    /// Java: `LinkArg.getInv()`
    pub fn inverted(&self) -> Self {
        Self {
            label: self.label.clone(),
            length: self.length,
            quantifier1: self.quantifier2.clone(),
            quantifier2: self.quantifier1.clone(),
            role1: self.role2.clone(),
            role2: self.role1.clone(),
            label_distance: self.label_distance.clone(),
            label_angle: self.label_angle.clone(),
            kal1: self.kal2.clone(),
            kal2: self.kal1.clone(),
            visibility_modifier: self.visibility_modifier,
        }
    }

    // ── Accessors ──

    pub fn label(&self) -> &[String] {
        &self.label
    }

    pub fn length(&self) -> i32 {
        self.length
    }

    pub fn set_length(&mut self, length: i32) {
        self.length = length;
    }

    pub fn quantifier1(&self) -> Option<&str> {
        self.quantifier1.as_deref()
    }

    pub fn quantifier2(&self) -> Option<&str> {
        self.quantifier2.as_deref()
    }

    pub fn role1(&self) -> Option<&str> {
        self.role1.as_deref()
    }

    pub fn role2(&self) -> Option<&str> {
        self.role2.as_deref()
    }

    pub fn label_distance(&self) -> Option<&str> {
        self.label_distance.as_deref()
    }

    pub fn label_angle(&self) -> Option<&str> {
        self.label_angle.as_deref()
    }

    pub fn kal1(&self) -> Option<&str> {
        self.kal1.as_deref()
    }

    pub fn kal2(&self) -> Option<&str> {
        self.kal2.as_deref()
    }

    pub fn has_kal1(&self) -> bool {
        self.kal1.as_ref().is_some_and(|s| !s.is_empty())
    }

    pub fn has_kal2(&self) -> bool {
        self.kal2.as_ref().is_some_and(|s| !s.is_empty())
    }

    pub fn visibility_modifier(&self) -> Option<VisibilityModifier> {
        self.visibility_modifier
    }

    pub fn set_visibility_modifier(&mut self, vis: Option<VisibilityModifier>) {
        self.visibility_modifier = vis;
    }
}

// ── NoteLinkStrategy ─────────────────────────────────────────────────

/// Strategy for rendering a note attached to a link.
/// Java: `abel.NoteLinkStrategy`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NoteLinkStrategy {
    #[default]
    Normal,
    HalfPrintedFull,
    HalfNotPrinted,
}

impl NoteLinkStrategy {
    /// Compute effective dimension given natural width/height.
    /// Java: `NoteLinkStrategy.computeDimension(double, double)`
    pub fn compute_dimension(&self, width: f64, height: f64) -> (f64, f64) {
        match self {
            Self::Normal => (width, height),
            Self::HalfPrintedFull => (width / 2.0, height),
            Self::HalfNotPrinted => (0.0, 0.0),
        }
    }
}

// ── CucaNote ─────────────────────────────────────────────────────────

/// A note annotation attached to an entity or link.
/// Java: `abel.CucaNote`
#[derive(Debug, Clone)]
pub struct CucaNote {
    pub display: Vec<String>,
    pub position: NotePosition,
    pub strategy: NoteLinkStrategy,
}

impl CucaNote {
    /// Create a new note.
    /// Java: `CucaNote.build(Display, Position, Colors)`
    pub fn new(display: Vec<String>, position: NotePosition) -> Self {
        Self {
            display,
            position,
            strategy: NoteLinkStrategy::Normal,
        }
    }

    /// Return a copy with a different strategy.
    /// Java: `CucaNote.withStrategy(NoteLinkStrategy)`
    pub fn with_strategy(&self, strategy: NoteLinkStrategy) -> Self {
        Self {
            display: self.display.clone(),
            position: self.position,
            strategy,
        }
    }
}

// ── DisplayPositioned ────────────────────────────────────────────────

/// A display block with alignment information.
/// Java: `abel.DisplayPositioned`
#[derive(Debug, Clone)]
pub struct DisplayPositioned {
    pub display: Vec<String>,
    pub horizontal_alignment: HorizontalAlignment,
    pub vertical_alignment: VerticalAlignment,
}

use crate::klimt::geom::{HorizontalAlignment, VerticalAlignment};

impl DisplayPositioned {
    /// Create a positioned display.
    pub fn single(
        display: Vec<String>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self {
        Self {
            display,
            horizontal_alignment,
            vertical_alignment,
        }
    }

    /// Create a "none" (empty) display.
    pub fn none() -> Self {
        Self {
            display: Vec::new(),
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
        }
    }

    pub fn is_null(&self) -> bool {
        self.display.is_empty()
    }
}

// ── EntityPortion ────────────────────────────────────────────────────

/// Visibility portion of an entity (what parts to show/hide).
/// Java: `abel.EntityPortion`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityPortion {
    Field,
    Method,
    Member,
    CircledCharacter,
    Stereotype,
}

impl EntityPortion {
    /// Expand `Member` into `{Field, Method}`; otherwise return a
    /// single-element set.
    /// Java: `EntityPortion.asSet()`
    pub fn as_set(&self) -> Vec<EntityPortion> {
        match self {
            Self::Member => vec![Self::Field, Self::Method],
            other => vec![*other],
        }
    }
}

// ── EntityGender ─────────────────────────────────────────────────────

/// Filter predicate for entity selection (hide/remove commands).
/// Java: `abel.EntityGender` + `abel.EntityGenderUtils`
///
/// In Rust we use an enum-of-closures approach rather than the Java
/// interface + anonymous-class pattern.
#[derive(Clone)]
pub enum EntityGender {
    /// Match entities of a specific leaf type.
    ByEntityType(LeafType),
    /// Match a single entity by UID.
    ByEntityAlone(String),
    /// Match by stereotype label.
    ByStereotype(String),
    /// Match by class name.
    ByClassName(String),
    /// Match entities whose parent is a specific group.
    ByPackage(String),
    /// Match all entities.
    All,
    /// Logical AND of two genders.
    And(Box<EntityGender>, Box<EntityGender>),
}

impl EntityGender {
    /// Test whether a given entity matches this gender filter.
    pub fn contains(&self, entity: &Entity) -> bool {
        match self {
            Self::ByEntityType(lt) => entity.leaf_type() == Some(*lt),
            Self::ByEntityAlone(uid) => entity.uid() == uid,
            Self::ByStereotype(stereo) => entity
                .stereotype()
                .is_some_and(|s| s.contains(stereo.as_str())),
            Self::ByClassName(name) => entity.name() == name,
            Self::ByPackage(parent_name) => entity.parent_name().is_some_and(|p| p == parent_name),
            Self::All => true,
            Self::And(g1, g2) => g1.contains(entity) && g2.contains(entity),
        }
    }

    /// Human-readable description.
    pub fn gender(&self) -> Option<&str> {
        match self {
            Self::ByEntityType(_) => None, // would need format
            Self::ByEntityAlone(uid) => Some(uid),
            Self::ByStereotype(s) => Some(s),
            Self::ByClassName(n) => Some(n),
            Self::ByPackage(_) => None,
            Self::All => None,
            Self::And(_, _) => None,
        }
    }
}

impl std::fmt::Debug for EntityGender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByEntityType(lt) => write!(f, "ByEntityType({:?})", lt),
            Self::ByEntityAlone(uid) => write!(f, "ByEntityAlone({})", uid),
            Self::ByStereotype(s) => write!(f, "ByStereotype({})", s),
            Self::ByClassName(n) => write!(f, "ByClassName({})", n),
            Self::ByPackage(p) => write!(f, "ByPackage({})", p),
            Self::All => write!(f, "All"),
            Self::And(g1, g2) => write!(f, "And({:?}, {:?})", g1, g2),
        }
    }
}

// ── EntityUtils ──────────────────────────────────────────────────────

/// Utility functions for entity graph analysis.
/// Java: `abel.EntityUtils`
pub mod entity_utils {
    use super::*;

    /// Check if a group is a parent (ancestor) of another group.
    fn is_parent_by_name(group_name: &str, candidate_name: &str, entities: &[Entity]) -> bool {
        let mut current = candidate_name.to_string();
        loop {
            if current == group_name {
                return true;
            }
            // Find the entity and get its parent
            let parent = entities
                .iter()
                .find(|e| e.name() == current)
                .and_then(|e| e.parent_name().map(|s| s.to_string()));
            match parent {
                Some(p) => current = p,
                None => return false,
            }
        }
    }

    /// Check if a link's both endpoints are inside a given group.
    /// Java: `EntityUtils.isPureInnerLink12(Entity, Link)`
    pub fn is_pure_inner_link12(group_name: &str, link: &Link, entities: &[Entity]) -> bool {
        let e1_parent = entities
            .iter()
            .find(|e| e.uid() == link.entity1_uid())
            .and_then(|e| e.parent_name());
        let e2_parent = entities
            .iter()
            .find(|e| e.uid() == link.entity2_uid())
            .and_then(|e| e.parent_name());

        match (e1_parent, e2_parent) {
            (Some(p1), Some(p2)) => {
                is_parent_by_name(group_name, p1, entities)
                    && is_parent_by_name(group_name, p2, entities)
            }
            _ => false,
        }
    }

    /// Check if a link's both endpoints have the same inside/outside
    /// relationship to a given group.
    /// Java: `EntityUtils.isPureInnerLink3(Entity, Link)`
    pub fn is_pure_inner_link3(group_name: &str, link: &Link, entities: &[Entity]) -> bool {
        let e1_parent = entities
            .iter()
            .find(|e| e.uid() == link.entity1_uid())
            .and_then(|e| e.parent_name());
        let e2_parent = entities
            .iter()
            .find(|e| e.uid() == link.entity2_uid())
            .and_then(|e| e.parent_name());

        let in1 = e1_parent.is_some_and(|p| is_parent_by_name(group_name, p, entities));
        let in2 = e2_parent.is_some_and(|p| is_parent_by_name(group_name, p, entities));
        in1 == in2
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    // decoration types used by other integration tests (kept for reference)

    // ── LinkArrow ──

    #[test]
    fn link_arrow_reverse() {
        assert_eq!(LinkArrow::DirectNormal.reverse(), LinkArrow::Backward);
        assert_eq!(LinkArrow::Backward.reverse(), LinkArrow::DirectNormal);
        assert_eq!(LinkArrow::NoneOrSeveral.reverse(), LinkArrow::NoneOrSeveral);
    }

    #[test]
    fn link_arrow_default() {
        assert_eq!(LinkArrow::default(), LinkArrow::NoneOrSeveral);
    }

    // ── LinkArg ──

    #[test]
    fn link_arg_basic() {
        let arg = LinkArg::new(vec!["label".to_string()], 2);
        assert_eq!(arg.label(), &["label"]);
        assert_eq!(arg.length(), 2);
        assert!(arg.quantifier1().is_none());
        assert!(arg.quantifier2().is_none());
    }

    #[test]
    fn link_arg_no_display() {
        let arg = LinkArg::no_display(1);
        assert!(arg.label().is_empty());
        assert_eq!(arg.length(), 1);
    }

    #[test]
    fn link_arg_with_quantifier() {
        let arg =
            LinkArg::new(vec![], 1).with_quantifier(Some("1".to_string()), Some("*".to_string()));
        assert_eq!(arg.quantifier1(), Some("1"));
        assert_eq!(arg.quantifier2(), Some("*"));
    }

    #[test]
    fn link_arg_with_role() {
        let arg =
            LinkArg::new(vec![], 1).with_role(Some("owner".to_string()), Some("owned".to_string()));
        assert_eq!(arg.role1(), Some("owner"));
        assert_eq!(arg.role2(), Some("owned"));
    }

    #[test]
    fn link_arg_with_kal() {
        let arg = LinkArg::new(vec![], 1).with_kal(Some("k1".to_string()), Some("k2".to_string()));
        assert!(arg.has_kal1());
        assert!(arg.has_kal2());
        assert_eq!(arg.kal1(), Some("k1"));
        assert_eq!(arg.kal2(), Some("k2"));
    }

    #[test]
    fn link_arg_no_kal() {
        let arg = LinkArg::new(vec![], 1);
        assert!(!arg.has_kal1());
        assert!(!arg.has_kal2());
    }

    #[test]
    fn link_arg_empty_kal() {
        let arg = LinkArg::new(vec![], 1).with_kal(Some(String::new()), None);
        assert!(!arg.has_kal1());
        assert!(!arg.has_kal2());
    }

    #[test]
    fn link_arg_with_distance_angle() {
        let arg = LinkArg::new(vec![], 1)
            .with_distance_angle(Some("2.0".to_string()), Some("-25".to_string()));
        assert_eq!(arg.label_distance(), Some("2.0"));
        assert_eq!(arg.label_angle(), Some("-25"));
    }

    #[test]
    fn link_arg_inverted() {
        let arg = LinkArg::new(vec!["lbl".to_string()], 3)
            .with_quantifier(Some("1".to_string()), Some("*".to_string()))
            .with_role(Some("A".to_string()), Some("B".to_string()))
            .with_kal(Some("k1".to_string()), Some("k2".to_string()));
        let inv = arg.inverted();
        assert_eq!(inv.label(), &["lbl"]);
        assert_eq!(inv.length(), 3);
        assert_eq!(inv.quantifier1(), Some("*"));
        assert_eq!(inv.quantifier2(), Some("1"));
        assert_eq!(inv.role1(), Some("B"));
        assert_eq!(inv.role2(), Some("A"));
        assert_eq!(inv.kal1(), Some("k2"));
        assert_eq!(inv.kal2(), Some("k1"));
    }

    #[test]
    fn link_arg_set_length() {
        let mut arg = LinkArg::new(vec![], 1);
        assert_eq!(arg.length(), 1);
        arg.set_length(5);
        assert_eq!(arg.length(), 5);
    }

    #[test]
    fn link_arg_visibility() {
        let mut arg = LinkArg::new(vec![], 1);
        assert!(arg.visibility_modifier().is_none());
        arg.set_visibility_modifier(Some(VisibilityModifier::Private));
        assert_eq!(arg.visibility_modifier(), Some(VisibilityModifier::Private));
    }

    // ── NoteLinkStrategy ──

    #[test]
    fn note_link_strategy_normal() {
        let (w, h) = NoteLinkStrategy::Normal.compute_dimension(100.0, 50.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 50.0);
    }

    #[test]
    fn note_link_strategy_half_printed() {
        let (w, h) = NoteLinkStrategy::HalfPrintedFull.compute_dimension(100.0, 50.0);
        assert_eq!(w, 50.0);
        assert_eq!(h, 50.0);
    }

    #[test]
    fn note_link_strategy_half_not_printed() {
        let (w, h) = NoteLinkStrategy::HalfNotPrinted.compute_dimension(100.0, 50.0);
        assert_eq!(w, 0.0);
        assert_eq!(h, 0.0);
    }

    // ── CucaNote ──

    #[test]
    fn cuca_note_new() {
        let note = CucaNote::new(
            vec!["line1".to_string(), "line2".to_string()],
            NotePosition::Top,
        );
        assert_eq!(note.display.len(), 2);
        assert_eq!(note.position, NotePosition::Top);
        assert_eq!(note.strategy, NoteLinkStrategy::Normal);
    }

    #[test]
    fn cuca_note_with_strategy() {
        let note = CucaNote::new(vec!["text".to_string()], NotePosition::Bottom);
        let note2 = note.with_strategy(NoteLinkStrategy::HalfPrintedFull);
        assert_eq!(note2.strategy, NoteLinkStrategy::HalfPrintedFull);
        assert_eq!(note2.display, vec!["text"]);
        assert_eq!(note2.position, NotePosition::Bottom);
        // original unchanged
        assert_eq!(note.strategy, NoteLinkStrategy::Normal);
    }

    // ── DisplayPositioned ──

    #[test]
    fn display_positioned_single() {
        let dp = DisplayPositioned::single(
            vec!["hello".to_string()],
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
        );
        assert!(!dp.is_null());
        assert_eq!(dp.display, vec!["hello"]);
    }

    #[test]
    fn display_positioned_none() {
        let dp = DisplayPositioned::none();
        assert!(dp.is_null());
    }

    // ── EntityPortion ──

    #[test]
    fn entity_portion_as_set_member() {
        let set = EntityPortion::Member.as_set();
        assert_eq!(set.len(), 2);
        assert!(set.contains(&EntityPortion::Field));
        assert!(set.contains(&EntityPortion::Method));
    }

    #[test]
    fn entity_portion_as_set_single() {
        let set = EntityPortion::Field.as_set();
        assert_eq!(set.len(), 1);
        assert!(set.contains(&EntityPortion::Field));
    }

    // ── EntityGender ──

    #[test]
    fn entity_gender_by_type() {
        let g = EntityGender::ByEntityType(LeafType::Class);
        let e = Entity::new_leaf("C1", LeafType::Class);
        assert!(g.contains(&e));
        let e2 = Entity::new_leaf("I1", LeafType::Interface);
        assert!(!g.contains(&e2));
    }

    #[test]
    fn entity_gender_by_uid() {
        let e = Entity::new_leaf("C1", LeafType::Class);
        let g = EntityGender::ByEntityAlone(e.uid().to_string());
        assert!(g.contains(&e));
        let e2 = Entity::new_leaf("C2", LeafType::Class);
        assert!(!g.contains(&e2));
    }

    #[test]
    fn entity_gender_by_stereotype() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.set_stereotype(Some("<<entity>>".to_string()));
        let g = EntityGender::ByStereotype("entity".to_string());
        assert!(g.contains(&e));
    }

    #[test]
    fn entity_gender_by_class_name() {
        let e = Entity::new_leaf("MyClass", LeafType::Class);
        let g = EntityGender::ByClassName("MyClass".to_string());
        assert!(g.contains(&e));
        let g2 = EntityGender::ByClassName("Other".to_string());
        assert!(!g2.contains(&e));
    }

    #[test]
    fn entity_gender_all() {
        let g = EntityGender::All;
        let e = Entity::new_leaf("Anything", LeafType::Note);
        assert!(g.contains(&e));
    }

    #[test]
    fn entity_gender_and() {
        let g = EntityGender::And(
            Box::new(EntityGender::ByEntityType(LeafType::Class)),
            Box::new(EntityGender::ByClassName("C1".to_string())),
        );
        let e1 = Entity::new_leaf("C1", LeafType::Class);
        assert!(g.contains(&e1));
        let e2 = Entity::new_leaf("C2", LeafType::Class);
        assert!(!g.contains(&e2));
        let e3 = Entity::new_leaf("C1", LeafType::Interface);
        assert!(!g.contains(&e3));
    }

    #[test]
    fn entity_gender_by_package() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.set_parent_name(Some("com.example".to_string()));
        let g = EntityGender::ByPackage("com.example".to_string());
        assert!(g.contains(&e));
        let g2 = EntityGender::ByPackage("other.pkg".to_string());
        assert!(!g2.contains(&e));
    }
}
