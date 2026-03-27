/// Component/Deployment diagram IR

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentKind {
    Component,
    Interface,
    Rectangle,
    Node,
    Database,
    Cloud,
    Package,
    Card,
    // Deployment diagram kinds
    Artifact,
    Storage,
    Folder,
    Frame,
    Agent,
    Stack,
    Queue,
    // Port kinds (used inside component groups)
    PortIn,
    PortOut,
}

#[derive(Debug, Clone)]
pub struct ComponentEntity {
    pub name: String,
    pub id: String,
    pub kind: ComponentKind,
    pub stereotype: Option<String>,
    pub description: Vec<String>,
    /// Parent group name (if nested inside a rectangle/package)
    pub parent: Option<String>,
    /// Optional background color (e.g. "#FF0000" or "LightBlue")
    pub color: Option<String>,
    /// Source line number (0-based) for data-source-line attribute
    pub source_line: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ComponentLink {
    pub from: String,
    pub to: String,
    pub label: String,
    pub dashed: bool,
    pub direction_hint: Option<String>,
    /// Arrow stem length (dash/dot count). 1=horizontal, 2+=vertical.
    pub arrow_len: usize,
    /// Source line number (1-based) for data-source-line attribute
    pub source_line: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ComponentGroup {
    pub name: String,
    pub id: String,
    pub kind: ComponentKind,
    pub stereotype: Option<String>,
    pub children: Vec<String>,
    /// Source line number (1-based) for data-source-line attribute
    pub source_line: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ComponentDiagram {
    pub entities: Vec<ComponentEntity>,
    pub links: Vec<ComponentLink>,
    pub groups: Vec<ComponentGroup>,
    pub notes: Vec<ComponentNote>,
    pub direction: super::diagram::Direction,
}

#[derive(Debug, Clone)]
pub struct ComponentNote {
    pub text: String,
    pub position: String,
    pub target: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_kind_eq() {
        assert_eq!(ComponentKind::Component, ComponentKind::Component);
        assert_ne!(ComponentKind::Component, ComponentKind::Rectangle);
    }

    #[test]
    fn test_component_entity_creation() {
        let e = ComponentEntity {
            name: "test".to_string(),
            id: "test".to_string(),
            kind: ComponentKind::Component,
            stereotype: None,
            description: vec![],
            parent: None,
            color: None,
            source_line: None,
        };
        assert_eq!(e.name, "test");
        assert_eq!(e.kind, ComponentKind::Component);
    }

    #[test]
    fn test_component_link_creation() {
        let l = ComponentLink {
            from: "A".to_string(),
            to: "B".to_string(),
            label: "uses".to_string(),
            dashed: false,
            direction_hint: Some("right".to_string()),
            arrow_len: 2,
            source_line: Some(3),
        };
        assert_eq!(l.from, "A");
        assert_eq!(l.direction_hint, Some("right".to_string()));
    }

    #[test]
    fn test_component_note_creation() {
        let n = ComponentNote {
            text: "hello\nworld".to_string(),
            position: "top".to_string(),
            target: Some("comp1".to_string()),
        };
        assert_eq!(n.position, "top");
        assert!(n.target.is_some());
    }

    #[test]
    fn test_component_group_creation() {
        let g = ComponentGroup {
            name: "My Group".to_string(),
            id: "my_group".to_string(),
            kind: ComponentKind::Rectangle,
            stereotype: Some("$businessProcess".to_string()),
            children: vec!["src".to_string(), "tgt".to_string()],
            source_line: Some(3),
        };
        assert_eq!(g.children.len(), 2);
    }

    #[test]
    fn test_component_diagram_creation() {
        let d = ComponentDiagram {
            entities: vec![],
            links: vec![],
            groups: vec![],
            notes: vec![],

            direction: Default::default(),
        };
        assert!(d.entities.is_empty());
        assert!(d.links.is_empty());
    }

    #[test]
    fn test_entity_with_description() {
        let e = ComponentEntity {
            name: "A".to_string(),
            id: "A".to_string(),
            kind: ComponentKind::Rectangle,
            stereotype: None,
            description: vec!["line 1".to_string(), "line 2".to_string()],
            parent: None,
            color: None,
            source_line: None,
        };
        assert_eq!(e.description.len(), 2);
    }

    #[test]
    fn test_entity_with_parent() {
        let e = ComponentEntity {
            name: "inner".to_string(),
            id: "inner".to_string(),
            kind: ComponentKind::Rectangle,
            stereotype: None,
            description: vec![],
            parent: Some("outer".to_string()),
            color: None,
            source_line: None,
        };
        assert_eq!(e.parent, Some("outer".to_string()));
    }

    #[test]
    fn test_component_kind_clone() {
        let k = ComponentKind::Database;
        let k2 = k.clone();
        assert_eq!(k, k2);
    }

    #[test]
    fn test_dashed_link() {
        let l = ComponentLink {
            from: "A".to_string(),
            to: "B".to_string(),
            label: String::new(),
            dashed: true,
            direction_hint: None,
            arrow_len: 2,
            source_line: None,
        };
        assert!(l.dashed);
        assert!(l.label.is_empty());
    }

    #[test]
    fn test_note_without_target() {
        let n = ComponentNote {
            text: "floating note".to_string(),
            position: "left".to_string(),
            target: None,
        };
        assert!(n.target.is_none());
    }

    #[test]
    fn test_all_component_kinds() {
        let kinds = [
            ComponentKind::Component,
            ComponentKind::Interface,
            ComponentKind::Rectangle,
            ComponentKind::Node,
            ComponentKind::Database,
            ComponentKind::Cloud,
            ComponentKind::Package,
            ComponentKind::Artifact,
            ComponentKind::Storage,
            ComponentKind::Folder,
            ComponentKind::Frame,
            ComponentKind::Agent,
            ComponentKind::Stack,
            ComponentKind::Queue,
            ComponentKind::PortIn,
            ComponentKind::PortOut,
        ];
        assert_eq!(kinds.len(), 16);
    }
}
