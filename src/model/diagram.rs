use std::collections::HashMap;

use super::entity::Entity;
use super::link::Link;

/// Diagram metadata (title / header / footer / legend / caption / pragmas)
#[derive(Debug, Clone, Default)]
pub struct DiagramMeta {
    pub title: Option<String>,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub legend: Option<String>,
    pub caption: Option<String>,
    pub title_line: Option<usize>,
    pub header_line: Option<usize>,
    pub footer_line: Option<usize>,
    pub legend_line: Option<usize>,
    pub caption_line: Option<usize>,
    /// Pragma key-value pairs (`!pragma key value`)
    pub pragmas: HashMap<String, String>,
}

impl DiagramMeta {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.header.is_none()
            && self.footer.is_none()
            && self.legend.is_none()
            && self.caption.is_none()
    }
}

/// Layout direction
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Direction {
    #[default]
    TopToBottom,
    LeftToRight,
    BottomToTop,
    RightToLeft,
}

/// Grouping container (package / namespace / rectangle)
#[derive(Debug, Clone)]
pub struct Group {
    pub uid: Option<String>,
    pub kind: GroupKind,
    pub name: String,
    pub entities: Vec<String>,
    pub stereotypes: Vec<super::entity::Stereotype>,
    pub color: Option<String>,
    pub source_line: Option<usize>,
}

/// Group kind
#[derive(Debug, Clone, PartialEq)]
pub enum GroupKind {
    Package,
    Namespace,
    Rectangle,
}

/// A note annotation on the class diagram.
#[derive(Debug, Clone)]
pub struct ClassNote {
    pub text: String,
    pub position: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassPortion {
    Field,
    Method,
    Stereotype,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassRuleTarget {
    Any,
    Entity(String),
    Stereotype(String),
}

#[derive(Debug, Clone)]
pub struct ClassHideShowRule {
    pub target: ClassRuleTarget,
    pub portion: ClassPortion,
    pub show: bool,
    /// When true, the rule only applies to empty sections (e.g. `hide empty members`).
    pub empty_only: bool,
}

/// Class diagram IR
#[derive(Debug, Clone)]
pub struct ClassDiagram {
    pub entities: Vec<Entity>,
    pub links: Vec<Link>,
    pub groups: Vec<Group>,
    pub direction: Direction,
    /// True when `left to right direction` was explicitly written (sets rankdir=LR in DOT).
    /// False when direction was inferred from arrow length (keeps rankdir=TB, controls via minlen).
    pub direction_explicit: bool,
    pub notes: Vec<ClassNote>,
    pub hide_show_rules: Vec<ClassHideShowRule>,
    pub stereotype_backgrounds: HashMap<String, String>,
}

/// Diagram type enum
#[derive(Debug)]
pub enum Diagram {
    Class(ClassDiagram),
    Sequence(super::sequence::SequenceDiagram),
    Activity(super::activity::ActivityDiagram),
    State(super::state::StateDiagram),
    Component(super::component::ComponentDiagram),
    Ditaa(super::ditaa::DitaaDiagram),
    Erd(super::erd::ErdDiagram),
    Gantt(super::gantt::GanttDiagram),
    Json(super::json_diagram::JsonDiagram),
    Mindmap(super::mindmap::MindmapDiagram),
    Nwdiag(super::nwdiag::NwdiagDiagram),
    Salt(super::salt::SaltDiagram),
    Timing(super::timing::TimingDiagram),
    Wbs(super::wbs::WbsDiagram),
    Yaml(super::json_diagram::JsonDiagram),
    Dot(super::dot::DotDiagram),
    UseCase(super::usecase::UseCaseDiagram),
}
