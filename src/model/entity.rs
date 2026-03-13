/// Entity kind
#[derive(Debug, Clone, PartialEq)]
pub enum EntityKind {
    Class,
    Interface,
    Enum,
    Abstract,
    Annotation,
    Object,
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
}

/// Stereotype (e.g. <<Entity>>)
#[derive(Debug, Clone, PartialEq)]
pub struct Stereotype(pub String);

/// Entity (class, interface, enum, etc.)
#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub kind: EntityKind,
    pub stereotypes: Vec<Stereotype>,
    pub members: Vec<Member>,
    pub color: Option<String>,
    pub generic: Option<String>,
}
