// abel::link - Link (edge) between two entities
// Port of Java PlantUML's abel.Link
//
// A Link connects two entities with a typed decoration, optional label,
// quantifiers, notes, and layout constraints.

use std::sync::atomic::{AtomicU64, Ordering};

use super::entity::VisibilityModifier;
use super::{CucaNote, LinkArg, LinkArrow, NoteLinkStrategy};
use crate::decoration::LinkType;

// ── Unique-ID generator ──────────────────────────────────────────────

static LINK_SEQ: AtomicU64 = AtomicU64::new(1);

fn next_link_uid() -> String {
    let n = LINK_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("lnk{}", n)
}

/// Reset the global link UID counter (for test isolation).
#[cfg(test)]
pub(crate) fn reset_link_uid_counter() {
    LINK_SEQ.store(1, Ordering::Relaxed);
}

// ── LinkStrategy ─────────────────────────────────────────────────────

/// Link rendering strategy.
/// Java: `abel.LinkStrategy`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkStrategy {
    /// Legacy mode: arrow decorations in Graphviz DOT.
    LegacyToBeRemoved,
    /// Simplified mode: decorations computed from Bezier data.
    #[default]
    Simplier,
}

// ── Link ─────────────────────────────────────────────────────────────

/// A directed link between two entities with decoration, label, and metadata.
/// Java: `abel.Link`
#[derive(Debug, Clone)]
pub struct Link {
    // ── identity ──
    uid: String,

    // ── endpoints ──
    entity1_uid: String,
    entity2_uid: String,
    port1: Option<String>,
    port2: Option<String>,

    // ── type ──
    link_type: LinkType,
    link_arg: LinkArg,

    // ── decoration modifiers ──
    invis: bool,
    constraint: bool,
    inverted: bool,
    link_arrow: LinkArrow,
    opale: bool,
    horizontal_solitary: bool,
    sametail: Option<String>,

    // ── weight ──
    weight: f64,

    // ── note ──
    note: Option<CucaNote>,

    // ── metadata ──
    stereotype: Option<String>,
    url: Option<String>,
    hidden: bool,
    removed: bool,
}

// ── Constructors ─────────────────────────────────────────────────────

impl Link {
    /// Create a new link between two entity UIDs.
    /// Java: `Link(LineLocation, CucaDiagram, StyleBuilder, Entity, Entity, LinkType, LinkArg)`
    pub fn new(
        entity1_uid: &str,
        entity2_uid: &str,
        link_type: LinkType,
        link_arg: LinkArg,
    ) -> Self {
        assert!(link_arg.length() >= 1, "link length must be >= 1");
        Self {
            uid: next_link_uid(),
            entity1_uid: entity1_uid.to_string(),
            entity2_uid: entity2_uid.to_string(),
            port1: None,
            port2: None,
            link_type,
            link_arg,
            invis: false,
            constraint: true,
            inverted: false,
            link_arrow: LinkArrow::NoneOrSeveral,
            opale: false,
            horizontal_solitary: false,
            sametail: None,
            weight: 1.0,
            note: None,
            stereotype: None,
            url: None,
            hidden: false,
            removed: false,
        }
    }

    /// Create an inverted copy (swap endpoints).
    /// Java: `Link.getInv()`
    pub fn inverted(&self) -> Self {
        let inv_arg = self.link_arg.inverted();
        Self {
            uid: next_link_uid(),
            entity1_uid: self.entity2_uid.clone(),
            entity2_uid: self.entity1_uid.clone(),
            port1: self.port2.clone(),
            port2: self.port1.clone(),
            link_type: self.link_type.inversed(),
            link_arg: inv_arg,
            invis: self.invis,
            constraint: self.constraint,
            inverted: !self.inverted,
            link_arrow: self.link_arrow,
            opale: self.opale,
            horizontal_solitary: self.horizontal_solitary,
            sametail: self.sametail.clone(),
            weight: self.weight,
            note: self.note.clone(),
            stereotype: self.stereotype.clone(),
            url: self.url.clone(),
            hidden: self.hidden,
            removed: self.removed,
        }
    }
}

// ── Identity ─────────────────────────────────────────────────────────

impl Link {
    pub fn uid(&self) -> &str {
        &self.uid
    }
}

// ── Endpoints ────────────────────────────────────────────────────────

impl Link {
    pub fn entity1_uid(&self) -> &str {
        &self.entity1_uid
    }

    pub fn entity2_uid(&self) -> &str {
        &self.entity2_uid
    }

    pub fn port1(&self) -> Option<&str> {
        self.port1.as_deref()
    }

    pub fn port2(&self) -> Option<&str> {
        self.port2.as_deref()
    }

    pub fn set_port_members(&mut self, port1: Option<String>, port2: Option<String>) {
        self.port1 = port1;
        self.port2 = port2;
    }

    /// Whether this link connects the given entity UID.
    /// Java: `Link.contains(Entity)`
    pub fn contains(&self, entity_uid: &str) -> bool {
        self.entity1_uid == entity_uid || self.entity2_uid == entity_uid
    }

    /// Return the other endpoint's UID given one endpoint.
    /// Java: `Link.getOther(Entity)`
    pub fn other(&self, entity_uid: &str) -> Option<&str> {
        if self.entity1_uid == entity_uid {
            Some(&self.entity2_uid)
        } else if self.entity2_uid == entity_uid {
            Some(&self.entity1_uid)
        } else {
            None
        }
    }

    /// Whether both endpoints are the same (self-loop).
    /// Java: `Link.isAutolink()`
    pub fn is_autolink(&self) -> bool {
        self.entity1_uid == self.entity2_uid
    }

    /// Whether this link connects the same pair of entities as another.
    /// Java: `Link.sameConnections(Link)`
    pub fn same_connections(&self, other: &Link) -> bool {
        (self.entity1_uid == other.entity1_uid && self.entity2_uid == other.entity2_uid)
            || (self.entity1_uid == other.entity2_uid && self.entity2_uid == other.entity1_uid)
    }

    /// Whether this link shares at least one endpoint with another.
    /// Java: `Link.doesTouch(Link)`
    pub fn does_touch(&self, other: &Link) -> bool {
        self.entity1_uid == other.entity1_uid
            || self.entity1_uid == other.entity2_uid
            || self.entity2_uid == other.entity1_uid
            || self.entity2_uid == other.entity2_uid
    }

    /// Whether this link connects entity1 and entity2 (in either order).
    /// Java: `Link.isBetween(Entity, Entity)`
    pub fn is_between(&self, uid1: &str, uid2: &str) -> bool {
        (self.entity1_uid == uid1 && self.entity2_uid == uid2)
            || (self.entity1_uid == uid2 && self.entity2_uid == uid1)
    }
}

// ── Type ─────────────────────────────────────────────────────────────

impl Link {
    pub fn link_type(&self) -> &LinkType {
        &self.link_type
    }

    pub fn set_link_type(&mut self, link_type: LinkType) {
        self.link_type = link_type;
    }

    pub fn link_arg(&self) -> &LinkArg {
        &self.link_arg
    }

    pub fn link_arg_mut(&mut self) -> &mut LinkArg {
        &mut self.link_arg
    }

    /// SVG comment id for this link.
    /// Java: `Link.idCommentForSvg()`
    pub fn id_comment_for_svg(&self) -> String {
        // Simplified: we don't have access to entity names here,
        // so use UIDs instead.
        format!("{}-to-{}", self.entity1_uid, self.entity2_uid)
    }
}

// ── Label / quantifiers ──────────────────────────────────────────────

impl Link {
    pub fn label(&self) -> &[String] {
        self.link_arg.label()
    }

    pub fn length(&self) -> i32 {
        self.link_arg.length()
    }

    pub fn set_length(&mut self, length: i32) {
        self.link_arg.set_length(length);
    }

    pub fn quantifier1(&self) -> Option<&str> {
        self.link_arg.quantifier1()
    }

    pub fn quantifier2(&self) -> Option<&str> {
        self.link_arg.quantifier2()
    }

    pub fn role1(&self) -> Option<&str> {
        self.link_arg.role1()
    }

    pub fn role2(&self) -> Option<&str> {
        self.link_arg.role2()
    }

    pub fn label_distance(&self) -> Option<&str> {
        self.link_arg.label_distance()
    }

    pub fn label_angle(&self) -> Option<&str> {
        self.link_arg.label_angle()
    }
}

// ── Decoration modifiers ─────────────────────────────────────────────

impl Link {
    /// Whether the link is invisible (either explicit or via type).
    /// Java: `Link.isInvis()`
    pub fn is_invis(&self) -> bool {
        self.invis || self.link_type.is_invisible()
    }

    pub fn set_invis(&mut self, invis: bool) {
        self.invis = invis;
    }

    pub fn is_constraint(&self) -> bool {
        self.constraint
    }

    pub fn set_constraint(&mut self, constraint: bool) {
        self.constraint = constraint;
    }

    /// Go "norank" - disable constraint.
    /// Java: `Link.goNorank()`
    pub fn go_norank(&mut self) {
        self.constraint = false;
    }

    pub fn is_inverted(&self) -> bool {
        self.inverted
    }

    pub fn link_arrow(&self) -> LinkArrow {
        if self.inverted {
            self.link_arrow.reverse()
        } else {
            self.link_arrow
        }
    }

    pub fn set_link_arrow(&mut self, arrow: LinkArrow) {
        self.link_arrow = arrow;
    }

    pub fn is_opale(&self) -> bool {
        self.opale
    }

    pub fn set_opale(&mut self, opale: bool) {
        self.opale = opale;
    }

    pub fn is_horizontal_solitary(&self) -> bool {
        self.horizontal_solitary
    }

    pub fn set_horizontal_solitary(&mut self, h: bool) {
        self.horizontal_solitary = h;
    }

    pub fn sametail(&self) -> Option<&str> {
        self.sametail.as_deref()
    }

    pub fn set_sametail(&mut self, sametail: Option<String>) {
        self.sametail = sametail;
    }
}

// ── Weight ───────────────────────────────────────────────────────────

impl Link {
    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: f64) {
        self.weight = weight;
    }
}

// ── Note ─────────────────────────────────────────────────────────────

impl Link {
    pub fn note(&self) -> Option<&CucaNote> {
        self.note.as_ref()
    }

    pub fn set_note(&mut self, note: Option<CucaNote>) {
        self.note = note;
    }

    /// Copy note from another link with a given strategy.
    /// Java: `Link.addNoteFrom(Link, NoteLinkStrategy)`
    pub fn add_note_from(&mut self, other: &Link, strategy: NoteLinkStrategy) {
        if let Some(ref note) = other.note {
            self.note = Some(note.with_strategy(strategy));
        }
    }
}

// ── Metadata ─────────────────────────────────────────────────────────

impl Link {
    pub fn stereotype(&self) -> Option<&str> {
        self.stereotype.as_deref()
    }

    pub fn set_stereotype(&mut self, stereotype: Option<String>) {
        self.stereotype = stereotype;
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn set_url(&mut self, url: Option<String>) {
        self.url = url;
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }

    pub fn set_removed(&mut self, removed: bool) {
        self.removed = removed;
    }

    pub fn visibility_modifier(&self) -> Option<VisibilityModifier> {
        self.link_arg.visibility_modifier()
    }

    pub fn has_url(&self) -> bool {
        self.url.is_some() || !self.link_arg.label().is_empty()
    }
}

// ── Display ──────────────────────────────────────────────────────────

impl std::fmt::Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{{}}} {}-->{}",
            self.uid,
            self.link_arg.length(),
            self.entity1_uid,
            self.entity2_uid,
        )
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoration::{LinkDecor, LinkType};

    fn simple_link_type() -> LinkType {
        LinkType::new(LinkDecor::Arrow, LinkDecor::None)
    }

    fn simple_link_arg() -> LinkArg {
        LinkArg::new(vec![], 1)
    }

    #[test]
    fn link_creation() {
        reset_link_uid_counter();
        let link = Link::new("ent1", "ent2", simple_link_type(), simple_link_arg());
        assert!(link.uid().starts_with("lnk"));
        assert_eq!(link.entity1_uid(), "ent1");
        assert_eq!(link.entity2_uid(), "ent2");
        assert_eq!(link.length(), 1);
        assert!(!link.is_invis());
        assert!(link.is_constraint());
        assert!(!link.is_inverted());
        assert!(!link.is_autolink());
    }

    #[test]
    fn link_unique_uids() {
        let l1 = Link::new("a", "b", simple_link_type(), simple_link_arg());
        let l2 = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert_ne!(l1.uid(), l2.uid());
    }

    #[test]
    fn link_contains() {
        let link = Link::new("ent1", "ent2", simple_link_type(), simple_link_arg());
        assert!(link.contains("ent1"));
        assert!(link.contains("ent2"));
        assert!(!link.contains("ent3"));
    }

    #[test]
    fn link_other() {
        let link = Link::new("ent1", "ent2", simple_link_type(), simple_link_arg());
        assert_eq!(link.other("ent1"), Some("ent2"));
        assert_eq!(link.other("ent2"), Some("ent1"));
        assert_eq!(link.other("ent3"), None);
    }

    #[test]
    fn link_autolink() {
        let link = Link::new("ent1", "ent1", simple_link_type(), simple_link_arg());
        assert!(link.is_autolink());
    }

    #[test]
    fn link_is_between() {
        let link = Link::new("ent1", "ent2", simple_link_type(), simple_link_arg());
        assert!(link.is_between("ent1", "ent2"));
        assert!(link.is_between("ent2", "ent1"));
        assert!(!link.is_between("ent1", "ent3"));
    }

    #[test]
    fn link_same_connections() {
        let l1 = Link::new("a", "b", simple_link_type(), simple_link_arg());
        let l2 = Link::new("b", "a", simple_link_type(), simple_link_arg());
        let l3 = Link::new("a", "c", simple_link_type(), simple_link_arg());
        assert!(l1.same_connections(&l2));
        assert!(!l1.same_connections(&l3));
    }

    #[test]
    fn link_does_touch() {
        let l1 = Link::new("a", "b", simple_link_type(), simple_link_arg());
        let l2 = Link::new("b", "c", simple_link_type(), simple_link_arg());
        let l3 = Link::new("d", "e", simple_link_type(), simple_link_arg());
        assert!(l1.does_touch(&l2));
        assert!(!l1.does_touch(&l3));
    }

    #[test]
    fn link_inverted() {
        let l1 = Link::new("a", "b", simple_link_type(), simple_link_arg());
        let l2 = l1.inverted();
        assert_eq!(l2.entity1_uid(), "b");
        assert_eq!(l2.entity2_uid(), "a");
        assert!(l2.is_inverted());
    }

    #[test]
    fn link_invis() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert!(!link.is_invis());
        link.set_invis(true);
        assert!(link.is_invis());
    }

    #[test]
    fn link_constraint() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert!(link.is_constraint());
        link.go_norank();
        assert!(!link.is_constraint());
    }

    #[test]
    fn link_arrow_direction() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert_eq!(link.link_arrow(), LinkArrow::NoneOrSeveral);
        link.set_link_arrow(LinkArrow::DirectNormal);
        assert_eq!(link.link_arrow(), LinkArrow::DirectNormal);
    }

    #[test]
    fn link_arrow_inverted() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        link.set_link_arrow(LinkArrow::DirectNormal);
        let inv = link.inverted();
        assert_eq!(inv.link_arrow(), LinkArrow::Backward);
    }

    #[test]
    fn link_weight() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert_eq!(link.weight(), 1.0);
        link.set_weight(2.5);
        assert_eq!(link.weight(), 2.5);
    }

    #[test]
    fn link_note() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert!(link.note().is_none());
        let note = CucaNote {
            display: vec!["test note".to_string()],
            position: super::super::entity::NotePosition::Top,
            strategy: NoteLinkStrategy::Normal,
        };
        link.set_note(Some(note));
        assert!(link.note().is_some());
        assert_eq!(link.note().unwrap().display, vec!["test note"]);
    }

    #[test]
    fn link_ports() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert!(link.port1().is_none());
        assert!(link.port2().is_none());
        link.set_port_members(Some("p1".to_string()), Some("p2".to_string()));
        assert_eq!(link.port1(), Some("p1"));
        assert_eq!(link.port2(), Some("p2"));
    }

    #[test]
    fn link_stereotype_and_url() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        link.set_stereotype(Some("<<use>>".to_string()));
        assert_eq!(link.stereotype(), Some("<<use>>"));
        link.set_url(Some("http://example.com".to_string()));
        assert_eq!(link.url(), Some("http://example.com"));
    }

    #[test]
    fn link_display() {
        let link = Link::new("ent1", "ent2", simple_link_type(), simple_link_arg());
        let s = format!("{}", link);
        assert!(s.contains("ent1"));
        assert!(s.contains("ent2"));
    }

    #[test]
    fn link_opale_and_solitary() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert!(!link.is_opale());
        link.set_opale(true);
        assert!(link.is_opale());
        assert!(!link.is_horizontal_solitary());
        link.set_horizontal_solitary(true);
        assert!(link.is_horizontal_solitary());
    }

    #[test]
    fn link_sametail() {
        let mut link = Link::new("a", "b", simple_link_type(), simple_link_arg());
        assert!(link.sametail().is_none());
        link.set_sametail(Some("tail1".to_string()));
        assert_eq!(link.sametail(), Some("tail1"));
    }

    #[test]
    #[should_panic(expected = "link length must be >= 1")]
    fn link_length_zero_panics() {
        Link::new("a", "b", simple_link_type(), LinkArg::new(vec![], 0));
    }

    #[test]
    fn link_quantifiers() {
        let arg =
            LinkArg::new(vec![], 1).with_quantifier(Some("1".to_string()), Some("*".to_string()));
        let link = Link::new("a", "b", simple_link_type(), arg);
        assert_eq!(link.quantifier1(), Some("1"));
        assert_eq!(link.quantifier2(), Some("*"));
    }

    #[test]
    fn link_roles() {
        let arg =
            LinkArg::new(vec![], 1).with_role(Some("owner".to_string()), Some("owned".to_string()));
        let link = Link::new("a", "b", simple_link_type(), arg);
        assert_eq!(link.role1(), Some("owner"));
        assert_eq!(link.role2(), Some("owned"));
    }

    #[test]
    fn add_note_from() {
        let mut l1 = Link::new("a", "b", simple_link_type(), simple_link_arg());
        let mut l2 = Link::new("c", "d", simple_link_type(), simple_link_arg());
        let note = CucaNote {
            display: vec!["shared note".to_string()],
            position: super::super::entity::NotePosition::Top,
            strategy: NoteLinkStrategy::Normal,
        };
        l1.set_note(Some(note));
        l2.add_note_from(&l1, NoteLinkStrategy::HalfPrintedFull);
        assert!(l2.note().is_some());
        assert_eq!(
            l2.note().unwrap().strategy,
            NoteLinkStrategy::HalfPrintedFull
        );
    }
}
