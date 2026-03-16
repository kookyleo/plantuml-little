// decoration::symbol - UML symbol shapes (component, database, cloud, etc.)
// Port of Java PlantUML's decoration.symbol package (30 files)
// Stub - to be filled by agent

use crate::klimt::geom::XDimension2D;

/// Margin specification for a UML symbol.
/// Java: `USymbol.Margin`
#[derive(Debug, Clone, Copy)]
pub struct SymbolMargin {
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

impl SymbolMargin {
    pub fn new(x1: f64, x2: f64, y1: f64, y2: f64) -> Self {
        Self { x1, x2, y1, y2 }
    }
    pub fn width(&self) -> f64 { self.x1 + self.x2 }
    pub fn height(&self) -> f64 { self.y1 + self.y2 }
    pub fn add_dimension(&self, dim: XDimension2D) -> XDimension2D {
        XDimension2D::new(dim.width + self.x1 + self.x2, dim.height + self.y1 + self.y2)
    }
}

/// UML symbol type enumeration.
/// Java: `decoration.symbol.USymbols` registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum USymbolKind {
    Action,
    ActorStickman,
    ActorAwesome,
    ActorHollow,
    ActorBusiness,
    Agent,
    Archimate,
    Artifact,
    Boundary,
    Card,
    Cloud,
    Collections,
    Component1,
    Component2,
    ComponentRectangle,
    Control,
    Database,
    EntityDomain,
    File,
    Folder,
    Frame,
    Group,
    Hexagon,
    Interface,
    Label,
    Node,
    Package,
    Person,
    Process,
    Queue,
    Rectangle,
    SimpleAbstract,
    Stack,
    Storage,
    Usecase,
}

impl USymbolKind {
    /// Get the margin for this symbol type.
    /// Java: each USymbol subclass defines its own margin
    pub fn margin(&self) -> SymbolMargin {
        match self {
            Self::Component1 => SymbolMargin::new(15.0, 5.0, 5.0, 5.0),
            Self::Component2 => SymbolMargin::new(15.0, 25.0, 20.0, 10.0),
            Self::Database => SymbolMargin::new(10.0, 10.0, 24.0, 10.0),
            Self::Cloud => SymbolMargin::new(25.0, 25.0, 15.0, 15.0),
            Self::Folder => SymbolMargin::new(10.0, 10.0, 30.0, 10.0),
            Self::Frame => SymbolMargin::new(10.0, 10.0, 30.0, 10.0),
            Self::Node => SymbolMargin::new(10.0, 20.0, 20.0, 10.0),
            Self::Storage => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            Self::Artifact => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            Self::Card => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            Self::Package => SymbolMargin::new(10.0, 10.0, 30.0, 10.0),
            Self::Queue => SymbolMargin::new(12.0, 12.0, 5.0, 5.0),
            Self::Stack => SymbolMargin::new(10.0, 10.0, 10.0, 15.0),
            Self::Hexagon => SymbolMargin::new(20.0, 20.0, 10.0, 10.0),
            Self::Person => SymbolMargin::new(10.0, 10.0, 30.0, 10.0),
            Self::File => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            _ => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
        }
    }

    /// Resolve a symbol name to a kind.
    /// Java: `USymbols.fromString()`
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "ACTION" => Some(Self::Action),
            "ACTOR" | "ACTOR_STICKMAN" => Some(Self::ActorStickman),
            "ACTOR_AWESOME" => Some(Self::ActorAwesome),
            "ACTOR_HOLLOW" => Some(Self::ActorHollow),
            "ACTOR_STICKMAN_BUSINESS" => Some(Self::ActorBusiness),
            "AGENT" => Some(Self::Agent),
            "ARCHIMATE" => Some(Self::Archimate),
            "ARTIFACT" => Some(Self::Artifact),
            "BOUNDARY" => Some(Self::Boundary),
            "CARD" => Some(Self::Card),
            "CLOUD" => Some(Self::Cloud),
            "COLLECTIONS" => Some(Self::Collections),
            "COMPONENT" | "COMPONENT2" => Some(Self::Component2),
            "COMPONENT1" => Some(Self::Component1),
            "COMPONENT_RECTANGLE" => Some(Self::ComponentRectangle),
            "CONTROL" => Some(Self::Control),
            "DATABASE" => Some(Self::Database),
            "ENTITY" | "ENTITY_DOMAIN" => Some(Self::EntityDomain),
            "FILE" => Some(Self::File),
            "FOLDER" => Some(Self::Folder),
            "FRAME" => Some(Self::Frame),
            "GROUP" => Some(Self::Group),
            "HEXAGON" => Some(Self::Hexagon),
            "INTERFACE" => Some(Self::Interface),
            "LABEL" => Some(Self::Label),
            "NODE" => Some(Self::Node),
            "PACKAGE" => Some(Self::Package),
            "PERSON" => Some(Self::Person),
            "PROCESS" => Some(Self::Process),
            "QUEUE" => Some(Self::Queue),
            "RECTANGLE" | "RECT" => Some(Self::Rectangle),
            "STACK" => Some(Self::Stack),
            "STORAGE" => Some(Self::Storage),
            "USECASE" => Some(Self::Usecase),
            _ => None,
        }
    }
}

// TODO: Each USymbol's draw methods (asSmall/asBig) will be ported
// by the parallel fill agent. They produce UPath/URect/ULine for klimt.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_from_name() {
        assert_eq!(USymbolKind::from_name("database"), Some(USymbolKind::Database));
        assert_eq!(USymbolKind::from_name("CLOUD"), Some(USymbolKind::Cloud));
        assert_eq!(USymbolKind::from_name("component"), Some(USymbolKind::Component2));
        assert_eq!(USymbolKind::from_name("ACTOR"), Some(USymbolKind::ActorStickman));
        assert!(USymbolKind::from_name("nonexistent").is_none());
    }

    #[test]
    fn symbol_margins() {
        let m = USymbolKind::Database.margin();
        assert_eq!(m.x1, 10.0);
        assert_eq!(m.y1, 24.0); // Database has tall top for cylinder
    }

    #[test]
    fn margin_add_dimension() {
        let m = SymbolMargin::new(10.0, 20.0, 5.0, 15.0);
        let dim = m.add_dimension(XDimension2D::new(100.0, 50.0));
        assert_eq!(dim.width, 130.0);
        assert_eq!(dim.height, 70.0);
    }
}
