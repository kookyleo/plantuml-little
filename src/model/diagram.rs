use super::entity::Entity;
use super::link::Link;

/// Diagram metadata (title / header / footer / legend / caption)
#[derive(Debug, Clone, Default)]
pub struct DiagramMeta {
    pub title: Option<String>,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub legend: Option<String>,
    pub caption: Option<String>,
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
    pub kind: GroupKind,
    pub name: String,
    pub entities: Vec<String>,
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

/// Class diagram IR
#[derive(Debug, Clone)]
pub struct ClassDiagram {
    pub entities: Vec<Entity>,
    pub links: Vec<Link>,
    pub groups: Vec<Group>,
    pub direction: Direction,
    pub notes: Vec<ClassNote>,
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
