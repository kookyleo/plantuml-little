/// Entity kind
#[derive(Debug, Clone, PartialEq)]
pub enum EntityKind {
    Class,
    Interface,
    Enum,
    Abstract,
    Annotation,
    Object,
    Rectangle,
    /// Component entity (rendered with component icon tabs)
    Component,
}

/// Member visibility
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
}

/// Member modifiers
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MemberModifiers {
    pub is_static: bool,
    pub is_abstract: bool,
}

/// Class member (field or method)
#[derive(Debug, Clone, PartialEq)]
pub struct Member {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub return_type: Option<String>,
    pub is_method: bool,
    pub modifiers: MemberModifiers,
    /// Raw display text (after removing visibility/modifiers), matching Java MemberImpl.getDisplay().
    /// When set, rendering uses this instead of reconstructing from name + return_type.
    pub display: Option<String>,
}

/// Stereotype (e.g. <<Entity>>)
#[derive(Debug, Clone, PartialEq)]
pub struct Stereotype(pub String);

/// Entity (class, interface, enum, etc.)
#[derive(Debug, Clone)]
pub struct Entity {
    pub uid: Option<String>,
    pub name: String,
    pub kind: EntityKind,
    pub stereotypes: Vec<Stereotype>,
    pub members: Vec<Member>,
    /// Bracket-body description lines for rectangle entities (Java: [text])
    pub description: Vec<String>,
    pub color: Option<String>,
    pub generic: Option<String>,
    pub source_line: Option<usize>,
    /// Entity-level visibility modifier (e.g. `-class foo` -> Private)
    pub visibility: Option<Visibility>,
    /// Display name (when `as Alias` is used, this holds the quoted label).
    pub display_name: Option<String>,
}
