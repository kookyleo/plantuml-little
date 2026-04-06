use super::diagram::Direction;

/// Use Case diagram intermediate representation.
#[derive(Debug, Clone)]
pub struct UseCaseDiagram {
    pub actors: Vec<UseCaseActor>,
    pub usecases: Vec<UseCase>,
    pub links: Vec<UseCaseLink>,
    pub boundaries: Vec<UseCaseBoundary>,
    pub notes: Vec<UseCaseNote>,
    pub direction: Direction,
}

/// An actor in the use case diagram.
#[derive(Debug, Clone)]
pub struct UseCaseActor {
    pub id: String,
    pub name: String,
    /// Java "code": alias if given, else display name.
    pub code: String,
    pub stereotype: Option<String>,
    pub color: Option<String>,
}

/// A use case (oval) in the diagram.
#[derive(Debug, Clone)]
pub struct UseCase {
    pub id: String,
    pub name: String,
    /// Java "code": alias if given, else display name.
    pub code: String,
    pub stereotype: Option<String>,
    pub color: Option<String>,
    /// Parent boundary id, if inside a package/rectangle
    pub parent: Option<String>,
}

/// A boundary (package/rectangle) grouping use cases.
#[derive(Debug, Clone)]
pub struct UseCaseBoundary {
    pub id: String,
    pub name: String,
    pub children: Vec<String>,
}

/// Relationship between actors and use cases.
#[derive(Debug, Clone)]
pub struct UseCaseLink {
    pub from: String,
    pub to: String,
    pub label: String,
    pub style: UseCaseLinkStyle,
    pub direction_hint: Option<String>,
    /// 0-based source line number for data-source-line attribute.
    pub source_line: Option<usize>,
}

/// Link style.
#[derive(Debug, Clone, PartialEq)]
pub enum UseCaseLinkStyle {
    /// Solid line with arrow (association)
    Association,
    /// Dashed line with open arrow (include/extend)
    Dashed,
    /// Dotted line with open arrow
    Dotted,
    /// Inheritance arrow (triangle head)
    Inheritance,
}

/// A note annotation.
#[derive(Debug, Clone)]
pub struct UseCaseNote {
    pub text: String,
    pub position: String,
    pub target: Option<String>,
}
