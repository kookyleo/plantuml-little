// abel::entity - Core entity data model
// Port of Java PlantUML's abel.Entity
//
// An Entity is either a leaf (class, note, state, ...) or a group
// (package, namespace, ...).  Exactly one of `leaf_type` / `group_type`
// is `Some`; the other is `None`.

use std::collections::{BTreeMap, BTreeSet};

use super::group_type::GroupType;
use super::leaf_type::LeafType;
use super::{
    CucaNote, DisplayPositioned, EntityPosition, NoteLinkStrategy, Together,
};
use crate::klimt::color::HColor;

// ── Unique-ID generator ──────────────────────────────────────────────

use std::sync::atomic::{AtomicU64, Ordering};

static ENTITY_SEQ: AtomicU64 = AtomicU64::new(1);

fn next_uid(prefix: &str) -> String {
    let n = ENTITY_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{}{}", prefix, n)
}

/// Reset the global entity UID counter (for test isolation).
#[cfg(test)]
pub(crate) fn reset_uid_counter() {
    ENTITY_SEQ.store(1, Ordering::Relaxed);
}

// ── Entity ───────────────────────────────────────────────────────────

/// Core entity for all diagram types.
///
/// Mirrors Java `abel.Entity`: it holds display text, stereotype,
/// leaf/group typing, hierarchy (parent/children via name-based paths),
/// colors, notes, etc.
///
/// In the Rust port the heavy "Quark<Entity>" tree from Java is replaced
/// by simple name strings; the tree structure is managed externally
/// (e.g. by the diagram container).
#[derive(Debug, Clone)]
pub struct Entity {
    // ── identity ──
    uid: String,
    name: String,
    is_root: bool,

    // ── display ──
    display: Vec<String>,
    stereotype: Option<String>,
    stereotags: BTreeSet<String>,
    generic: Option<String>,

    // ── typing (exactly one is Some) ──
    leaf_type: Option<LeafType>,
    group_type: Option<GroupType>,

    // ── hierarchy ──
    parent_name: Option<String>,

    // ── bodier (raw body lines for class members etc.) ──
    body_lines: Vec<String>,

    // ── notes ──
    notes_top: Vec<CucaNote>,
    notes_bottom: Vec<CucaNote>,

    // ── colors ──
    colors: EntityColors,

    // ── layout hints ──
    raw_layout: i32,
    xposition: i32,
    concurrent_separator: char,
    together: Option<Together>,
    packed: bool,
    is_static: bool,
    hidden: bool,
    removed: bool,

    // ── misc ──
    port_short_names: BTreeSet<String>,
    tips: BTreeMap<String, Vec<String>>,
    url: Option<String>,
    legend: Option<DisplayPositioned>,
    visibility: Option<VisibilityModifier>,
}

/// Simplified color storage for an entity.
/// Java: `klimt.color.Colors` (a map of ColorType -> HColor).
#[derive(Debug, Clone, Default)]
pub struct EntityColors {
    pub back: Option<HColor>,
    pub line: Option<HColor>,
    pub text: Option<HColor>,
    pub header: Option<HColor>,
}

impl EntityColors {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.back.is_none()
            && self.line.is_none()
            && self.text.is_none()
            && self.header.is_none()
    }
}

/// Visibility modifier on an entity or link.
/// Java: `skin.VisibilityModifier`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisibilityModifier {
    Public,
    Private,
    Protected,
    Package,
}

impl VisibilityModifier {
    /// Check if a character is a visibility prefix.
    pub fn is_visibility_char(c: char) -> bool {
        matches!(c, '+' | '-' | '#' | '~')
    }

    /// Parse visibility from the leading character.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Public),
            '-' => Some(Self::Private),
            '#' => Some(Self::Protected),
            '~' => Some(Self::Package),
            _ => None,
        }
    }
}

// ── Constructors ─────────────────────────────────────────────────────

impl Entity {
    /// Create a new leaf entity.
    pub fn new_leaf(name: &str, leaf_type: LeafType) -> Self {
        Self {
            uid: next_uid("ent"),
            name: name.to_string(),
            is_root: false,
            display: vec![name.to_string()],
            stereotype: None,
            stereotags: BTreeSet::new(),
            generic: None,
            leaf_type: Some(leaf_type),
            group_type: None,
            parent_name: None,
            body_lines: Vec::new(),
            notes_top: Vec::new(),
            notes_bottom: Vec::new(),
            colors: EntityColors::empty(),
            raw_layout: 0,
            xposition: 0,
            concurrent_separator: '\0',
            together: None,
            packed: false,
            is_static: false,
            hidden: false,
            removed: false,
            port_short_names: BTreeSet::new(),
            tips: BTreeMap::new(),
            url: None,
            legend: None,
            visibility: None,
        }
    }

    /// Create a new group entity.
    pub fn new_group(name: &str, group_type: GroupType) -> Self {
        Self {
            uid: next_uid("ent"),
            name: name.to_string(),
            is_root: false,
            display: vec![name.to_string()],
            stereotype: None,
            stereotags: BTreeSet::new(),
            generic: None,
            leaf_type: None,
            group_type: Some(group_type),
            parent_name: None,
            body_lines: Vec::new(),
            notes_top: Vec::new(),
            notes_bottom: Vec::new(),
            colors: EntityColors::empty(),
            raw_layout: 0,
            xposition: 0,
            concurrent_separator: '\0',
            together: None,
            packed: false,
            is_static: false,
            hidden: false,
            removed: false,
            port_short_names: BTreeSet::new(),
            tips: BTreeMap::new(),
            url: None,
            legend: None,
            visibility: None,
        }
    }

    /// Create the root entity (top-level container).
    pub fn new_root() -> Self {
        Self {
            uid: "entroot".to_string(),
            name: String::new(),
            is_root: true,
            display: Vec::new(),
            stereotype: None,
            stereotags: BTreeSet::new(),
            generic: None,
            leaf_type: None,
            group_type: Some(GroupType::Root),
            parent_name: None,
            body_lines: Vec::new(),
            notes_top: Vec::new(),
            notes_bottom: Vec::new(),
            colors: EntityColors::empty(),
            raw_layout: 0,
            xposition: 0,
            concurrent_separator: '\0',
            together: None,
            packed: false,
            is_static: false,
            hidden: false,
            removed: false,
            port_short_names: BTreeSet::new(),
            tips: BTreeMap::new(),
            url: None,
            legend: None,
            visibility: None,
        }
    }
}

// ── Identity ─────────────────────────────────────────────────────────

impl Entity {
    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_root(&self) -> bool {
        self.is_root
    }
}

// ── Typing ───────────────────────────────────────────────────────────

impl Entity {
    /// Whether this entity is a group (package, namespace, state, ...).
    /// Java: `Entity.isGroup()`
    pub fn is_group(&self) -> bool {
        match (self.group_type, self.leaf_type) {
            (Some(_), None) => true,
            (None, Some(_)) => false,
            _ => {
                // Root with both None is treated as group
                self.is_root
            }
        }
    }

    pub fn leaf_type(&self) -> Option<LeafType> {
        self.leaf_type
    }

    pub fn group_type(&self) -> Option<GroupType> {
        self.group_type
    }

    /// Mutate to a different leaf type.
    /// Java: `Entity.muteToType(LeafType)`
    pub fn mute_to_leaf_type(&mut self, new_type: LeafType) {
        self.group_type = None;
        self.leaf_type = Some(new_type);
    }

    /// Mutate to a different group type.
    /// Java: `Entity.muteToGroupType(GroupType)`
    pub fn mute_to_group_type(&mut self, new_type: GroupType) {
        self.group_type = Some(new_type);
        self.leaf_type = None;
    }

    /// Try to mutate to a new leaf type, returning `false` if the
    /// transition is disallowed.
    /// Java: `Entity.muteToType(LeafType, USymbol)`
    pub fn try_mute_to_leaf_type(&mut self, new_type: LeafType) -> bool {
        if let Some(current) = self.leaf_type {
            if current != LeafType::StillUnknown {
                if new_type == current {
                    return true;
                }
                let mutable_from = matches!(
                    current,
                    LeafType::Annotation
                        | LeafType::AbstractClass
                        | LeafType::Class
                        | LeafType::Enum
                        | LeafType::Interface
                );
                let mutable_to = matches!(
                    new_type,
                    LeafType::Annotation
                        | LeafType::AbstractClass
                        | LeafType::Class
                        | LeafType::Enum
                        | LeafType::Interface
                        | LeafType::Object
                );
                if !mutable_from || !mutable_to {
                    return false;
                }
            }
        }
        self.leaf_type = Some(new_type);
        true
    }
}

// ── Entity position ──────────────────────────────────────────────────

impl Entity {
    /// Derive `EntityPosition` from leaf type and stereotype.
    /// Java: `Entity.getEntityPosition()`
    pub fn entity_position(&self) -> EntityPosition {
        match self.leaf_type {
            Some(LeafType::PortIn) => EntityPosition::PortIn,
            Some(LeafType::PortOut) => EntityPosition::PortOut,
            Some(LeafType::State) => {
                if self.is_root {
                    return EntityPosition::Normal;
                }
                if let Some(ref stereo) = self.stereotype {
                    EntityPosition::from_stereotype(stereo)
                } else {
                    EntityPosition::Normal
                }
            }
            _ => EntityPosition::Normal,
        }
    }
}

// ── Display / stereotype ─────────────────────────────────────────────

impl Entity {
    pub fn display(&self) -> &[String] {
        &self.display
    }

    pub fn set_display(&mut self, display: Vec<String>) {
        self.display = display;
    }

    pub fn stereotype(&self) -> Option<&str> {
        self.stereotype.as_deref()
    }

    pub fn set_stereotype(&mut self, stereotype: Option<String>) {
        self.stereotype = stereotype;
    }

    pub fn add_stereotag(&mut self, tag: String) {
        self.stereotags.insert(tag);
    }

    pub fn stereotags(&self) -> &BTreeSet<String> {
        &self.stereotags
    }

    pub fn generic(&self) -> Option<&str> {
        self.generic.as_deref()
    }

    pub fn set_generic(&mut self, generic: Option<String>) {
        self.generic = generic;
    }
}

// ── Body (members) ───────────────────────────────────────────────────

impl Entity {
    pub fn body_lines(&self) -> &[String] {
        &self.body_lines
    }

    pub fn add_body_line(&mut self, line: String) {
        self.body_lines.push(line);
    }

    pub fn set_body_lines(&mut self, lines: Vec<String>) {
        self.body_lines = lines;
    }
}

// ── Notes ────────────────────────────────────────────────────────────

impl Entity {
    /// Add a note at the given position (top or bottom).
    /// Java: `Entity.addNote(Display, Position, Colors)`
    pub fn add_note(&mut self, note: CucaNote) {
        match note.position {
            NotePosition::Top => self.notes_top.push(note),
            NotePosition::Bottom => self.notes_bottom.push(note),
        }
    }

    pub fn notes_top(&self) -> &[CucaNote] {
        &self.notes_top
    }

    pub fn notes_bottom(&self) -> &[CucaNote] {
        &self.notes_bottom
    }
}

/// Note attachment position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotePosition {
    Top,
    Bottom,
}

// ── Colors ───────────────────────────────────────────────────────────

impl Entity {
    pub fn colors(&self) -> &EntityColors {
        &self.colors
    }

    pub fn set_colors(&mut self, colors: EntityColors) {
        self.colors = colors;
    }

    pub fn set_back_color(&mut self, color: HColor) {
        self.colors.back = Some(color);
    }

    pub fn set_line_color(&mut self, color: HColor) {
        self.colors.line = Some(color);
    }
}

// ── Hierarchy ────────────────────────────────────────────────────────

impl Entity {
    pub fn parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    pub fn set_parent_name(&mut self, parent: Option<String>) {
        self.parent_name = parent;
    }
}

// ── Layout hints ─────────────────────────────────────────────────────

impl Entity {
    pub fn raw_layout(&self) -> i32 {
        self.raw_layout
    }

    pub fn set_raw_layout(&mut self, raw_layout: i32) {
        self.raw_layout = raw_layout;
    }

    pub fn xposition(&self) -> i32 {
        self.xposition
    }

    pub fn set_xposition(&mut self, pos: i32) {
        self.xposition = pos;
    }

    pub fn concurrent_separator(&self) -> char {
        self.concurrent_separator
    }

    pub fn set_concurrent_separator(&mut self, sep: char) {
        self.concurrent_separator = sep;
    }

    pub fn together(&self) -> Option<&Together> {
        self.together.as_ref()
    }

    pub fn set_together(&mut self, together: Option<Together>) {
        self.together = together;
    }

    pub fn is_packed(&self) -> bool {
        self.packed
    }

    pub fn set_packed(&mut self, packed: bool) {
        self.packed = packed;
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }

    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
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
}

// ── Ports ────────────────────────────────────────────────────────────

impl Entity {
    pub fn port_short_names(&self) -> &BTreeSet<String> {
        &self.port_short_names
    }

    pub fn add_port_short_name(&mut self, name: String) {
        self.port_short_names.insert(name);
    }
}

// ── Tips ─────────────────────────────────────────────────────────────

impl Entity {
    pub fn tips(&self) -> &BTreeMap<String, Vec<String>> {
        &self.tips
    }

    pub fn put_tip(&mut self, member: String, display: Vec<String>) {
        self.tips.insert(member, display);
    }
}

// ── URL / legend / visibility ────────────────────────────────────────

impl Entity {
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn set_url(&mut self, url: Option<String>) {
        self.url = url;
    }

    pub fn legend(&self) -> Option<&DisplayPositioned> {
        self.legend.as_ref()
    }

    pub fn set_legend(&mut self, legend: Option<DisplayPositioned>) {
        self.legend = legend;
    }

    pub fn visibility(&self) -> Option<VisibilityModifier> {
        self.visibility
    }

    pub fn set_visibility(&mut self, vis: Option<VisibilityModifier>) {
        self.visibility = vis;
    }
}

// ── Display ──────────────────────────────────────────────────────────

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = self.display.join(", ");
        let type_str = if let Some(lt) = self.leaf_type {
            format!("({lt})")
        } else if let Some(gt) = self.group_type {
            format!("[{gt}]")
        } else {
            "?".to_string()
        };
        write!(f, "{} {} {} {}", self.name, display_str, type_str, self.uid)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_leaf_basic() {
        reset_uid_counter();
        let e = Entity::new_leaf("MyClass", LeafType::Class);
        assert!(e.uid().starts_with("ent"));
        assert_eq!(e.name(), "MyClass");
        assert!(!e.is_root());
        assert!(!e.is_group());
        assert_eq!(e.leaf_type(), Some(LeafType::Class));
        assert_eq!(e.group_type(), None);
        assert_eq!(e.display(), &["MyClass"]);
    }

    #[test]
    fn new_group_basic() {
        let e = Entity::new_group("com.example", GroupType::Package);
        assert!(e.uid().starts_with("ent"));
        assert_eq!(e.name(), "com.example");
        assert!(!e.is_root());
        assert!(e.is_group());
        assert_eq!(e.leaf_type(), None);
        assert_eq!(e.group_type(), Some(GroupType::Package));
    }

    #[test]
    fn new_root() {
        let r = Entity::new_root();
        assert_eq!(r.uid(), "entroot");
        assert!(r.is_root());
        assert!(r.is_group());
    }

    #[test]
    fn mute_to_type() {
        let mut e = Entity::new_leaf("A", LeafType::StillUnknown);
        assert!(e.try_mute_to_leaf_type(LeafType::Class));
        assert_eq!(e.leaf_type(), Some(LeafType::Class));
    }

    #[test]
    fn mute_class_to_interface() {
        let mut e = Entity::new_leaf("A", LeafType::Class);
        assert!(e.try_mute_to_leaf_type(LeafType::Interface));
        assert_eq!(e.leaf_type(), Some(LeafType::Interface));
    }

    #[test]
    fn mute_class_to_note_fails() {
        let mut e = Entity::new_leaf("A", LeafType::Class);
        assert!(!e.try_mute_to_leaf_type(LeafType::Note));
        // leaf_type unchanged
        assert_eq!(e.leaf_type(), Some(LeafType::Class));
    }

    #[test]
    fn mute_to_same_type_ok() {
        let mut e = Entity::new_leaf("A", LeafType::Enum);
        assert!(e.try_mute_to_leaf_type(LeafType::Enum));
    }

    #[test]
    fn mute_leaf_to_group() {
        let mut e = Entity::new_leaf("A", LeafType::Class);
        e.mute_to_group_type(GroupType::Package);
        assert!(e.is_group());
        assert_eq!(e.leaf_type(), None);
        assert_eq!(e.group_type(), Some(GroupType::Package));
    }

    #[test]
    fn mute_group_to_leaf() {
        let mut e = Entity::new_group("A", GroupType::Package);
        e.mute_to_leaf_type(LeafType::Description);
        assert!(!e.is_group());
        assert_eq!(e.leaf_type(), Some(LeafType::Description));
        assert_eq!(e.group_type(), None);
    }

    #[test]
    fn entity_position_port_in() {
        let e = Entity::new_leaf("p1", LeafType::PortIn);
        assert_eq!(e.entity_position(), EntityPosition::PortIn);
    }

    #[test]
    fn entity_position_port_out() {
        let e = Entity::new_leaf("p2", LeafType::PortOut);
        assert_eq!(e.entity_position(), EntityPosition::PortOut);
    }

    #[test]
    fn entity_position_state_with_stereotype() {
        let mut e = Entity::new_leaf("S1", LeafType::State);
        e.set_stereotype(Some("<<entrypoint>>".to_string()));
        assert_eq!(e.entity_position(), EntityPosition::EntryPoint);
    }

    #[test]
    fn entity_position_state_no_stereotype() {
        let e = Entity::new_leaf("S1", LeafType::State);
        assert_eq!(e.entity_position(), EntityPosition::Normal);
    }

    #[test]
    fn entity_position_normal() {
        let e = Entity::new_leaf("C1", LeafType::Class);
        assert_eq!(e.entity_position(), EntityPosition::Normal);
    }

    #[test]
    fn display_and_stereotype() {
        let mut e = Entity::new_leaf("Foo", LeafType::Class);
        e.set_display(vec!["Display Name".to_string()]);
        e.set_stereotype(Some("<<entity>>".to_string()));
        assert_eq!(e.display(), &["Display Name"]);
        assert_eq!(e.stereotype(), Some("<<entity>>"));
    }

    #[test]
    fn body_lines() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.add_body_line("+field1: int".to_string());
        e.add_body_line("+method(): void".to_string());
        assert_eq!(e.body_lines().len(), 2);
    }

    #[test]
    fn notes() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.add_note(CucaNote {
            display: vec!["top note".to_string()],
            position: NotePosition::Top,
            strategy: NoteLinkStrategy::Normal,
        });
        e.add_note(CucaNote {
            display: vec!["bottom note".to_string()],
            position: NotePosition::Bottom,
            strategy: NoteLinkStrategy::Normal,
        });
        assert_eq!(e.notes_top().len(), 1);
        assert_eq!(e.notes_bottom().len(), 1);
    }

    #[test]
    fn colors() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        assert!(e.colors().is_empty());
        e.set_back_color(HColor::Simple { r: 255, g: 0, b: 0 });
        assert!(!e.colors().is_empty());
        assert_eq!(e.colors().back, Some(HColor::Simple { r: 255, g: 0, b: 0 }));
    }

    #[test]
    fn ports() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.add_port_short_name("p1".to_string());
        e.add_port_short_name("p2".to_string());
        e.add_port_short_name("p1".to_string()); // duplicate
        assert_eq!(e.port_short_names().len(), 2);
    }

    #[test]
    fn tips() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.put_tip("field1".to_string(), vec!["tip text".to_string()]);
        assert_eq!(e.tips().len(), 1);
        assert_eq!(e.tips()["field1"], vec!["tip text"]);
    }

    #[test]
    fn layout_hints() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.set_xposition(42);
        assert_eq!(e.xposition(), 42);
        e.set_packed(true);
        assert!(e.is_packed());
        e.set_static(true);
        assert!(e.is_static());
        e.set_hidden(true);
        assert!(e.is_hidden());
        e.set_removed(true);
        assert!(e.is_removed());
    }

    #[test]
    fn url_and_legend() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.set_url(Some("https://example.com".to_string()));
        assert_eq!(e.url(), Some("https://example.com"));
        e.set_legend(Some(DisplayPositioned::none()));
        assert!(e.legend().is_some());
    }

    #[test]
    fn visibility_modifier() {
        assert!(VisibilityModifier::is_visibility_char('+'));
        assert!(VisibilityModifier::is_visibility_char('-'));
        assert!(VisibilityModifier::is_visibility_char('#'));
        assert!(VisibilityModifier::is_visibility_char('~'));
        assert!(!VisibilityModifier::is_visibility_char('x'));

        assert_eq!(VisibilityModifier::from_char('+'), Some(VisibilityModifier::Public));
        assert_eq!(VisibilityModifier::from_char('-'), Some(VisibilityModifier::Private));
        assert_eq!(VisibilityModifier::from_char('#'), Some(VisibilityModifier::Protected));
        assert_eq!(VisibilityModifier::from_char('~'), Some(VisibilityModifier::Package));
        assert_eq!(VisibilityModifier::from_char('x'), None);
    }

    #[test]
    fn display_format() {
        let e = Entity::new_leaf("Foo", LeafType::Class);
        let s = format!("{}", e);
        assert!(s.contains("Foo"));
        assert!(s.contains("Class"));
    }

    #[test]
    fn parent_hierarchy() {
        let mut e = Entity::new_leaf("Child", LeafType::Class);
        assert_eq!(e.parent_name(), None);
        e.set_parent_name(Some("com.example".to_string()));
        assert_eq!(e.parent_name(), Some("com.example"));
    }

    #[test]
    fn stereotags() {
        let mut e = Entity::new_leaf("C1", LeafType::Class);
        e.add_stereotag("tag1".to_string());
        e.add_stereotag("tag2".to_string());
        e.add_stereotag("tag1".to_string()); // duplicate
        assert_eq!(e.stereotags().len(), 2);
    }

    #[test]
    fn unique_uids() {
        let e1 = Entity::new_leaf("A", LeafType::Class);
        let e2 = Entity::new_leaf("B", LeafType::Class);
        assert_ne!(e1.uid(), e2.uid());
    }

    #[test]
    fn together_support() {
        let mut e = Entity::new_leaf("A", LeafType::Class);
        assert!(e.together().is_none());
        let t = Together { id: 1, parent: None };
        e.set_together(Some(t.clone()));
        assert_eq!(e.together().unwrap().id, 1);
    }
}
