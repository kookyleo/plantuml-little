// svek::shape_type - Enums for node shapes and package styles
// Port of Java PlantUML's svek.ShapeType + svek.PackageStyle

/// DOT node shape type.
/// Java: `svek.ShapeType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShapeType {
    #[default]
    Rectangle,
    RectanglePort,
    RectangleWithCircleInside,
    RectangleHtmlForPorts,
    RoundRectangle,
    Circle,
    Oval,
    Diamond,
    Octagon,
    Folder,
    Hexagon,
    Port,
}

impl ShapeType {
    /// DOT shape attribute value.
    pub fn dot_shape(&self) -> &'static str {
        match self {
            Self::Rectangle
            | Self::RectanglePort
            | Self::RectangleWithCircleInside
            | Self::RectangleHtmlForPorts => "rect",
            Self::RoundRectangle => "rect",
            Self::Circle => "circle",
            Self::Oval => "ellipse",
            Self::Diamond => "diamond",
            Self::Octagon => "octagon",
            Self::Folder => "rect",
            Self::Hexagon => "hexagon",
            Self::Port => "rect",
        }
    }
}

/// Package/namespace visual style.
/// Java: `svek.PackageStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PackageStyle {
    #[default]
    Folder,
    Rectangle,
    Node,
    Frame,
    Cloud,
    Database,
    Agent,
    Storage,
    Component1,
    Component2,
    Artifact,
    Card,
}

impl PackageStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "folder" => Some(Self::Folder),
            "rectangle" | "rect" => Some(Self::Rectangle),
            "node" => Some(Self::Node),
            "frame" => Some(Self::Frame),
            "cloud" => Some(Self::Cloud),
            "database" => Some(Self::Database),
            "agent" => Some(Self::Agent),
            "storage" => Some(Self::Storage),
            "component" | "component2" => Some(Self::Component2),
            "component1" => Some(Self::Component1),
            "artifact" => Some(Self::Artifact),
            "card" => Some(Self::Card),
            _ => None,
        }
    }
}

/// Condition end style for activity diagrams.
/// Java: `svek.ConditionEndStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConditionEndStyle {
    #[default]
    Diamond,
    Hline,
}

/// Condition style for activity diagrams.
/// Java: `svek.ConditionStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConditionStyle {
    #[default]
    Diamond,
    Inside,
    Foo1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_type_dot() {
        assert_eq!(ShapeType::Rectangle.dot_shape(), "rect");
        assert_eq!(ShapeType::Diamond.dot_shape(), "diamond");
        assert_eq!(ShapeType::Circle.dot_shape(), "circle");
    }

    #[test]
    fn package_style_parse() {
        assert_eq!(PackageStyle::from_str("folder"), Some(PackageStyle::Folder));
        assert_eq!(
            PackageStyle::from_str("RECT"),
            Some(PackageStyle::Rectangle)
        );
        assert_eq!(PackageStyle::from_str("Cloud"), Some(PackageStyle::Cloud));
        assert!(PackageStyle::from_str("unknown").is_none());
    }
}
