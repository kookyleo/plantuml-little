// abel::leaf_type - Leaf entity type enumeration
// Port of Java PlantUML's abel.LeafType

/// Type of a leaf (non-group) entity.
/// Java: `abel.LeafType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeafType {
    EmptyPackage,

    // Class-diagram family
    AbstractClass,
    Class,
    Interface,
    Annotation,
    Protocol,
    Struct,
    Exception,
    Metaclass,
    Stereotype,
    LollipopFull,
    LollipopHalf,
    Note,
    Tips,
    Object,
    Map,
    Json,
    Association,
    Enum,
    Circle,
    Dataclass,
    Record,

    // Use-case
    Usecase,
    UsecaseBusiness,

    // Description / component
    Description,

    ArcCircle,

    // Activity
    Activity,
    Branch,
    SynchroBar,
    CircleStart,
    CircleEnd,
    PointForAssociation,
    ActivityConcurrent,

    // State
    State,
    StateConcurrent,
    PseudoState,
    DeepHistory,
    StateChoice,
    StateForkJoin,
    StateTransitionLabel,

    // Block / deployment
    Block,
    Entity,

    // Domain / requirement
    Domain,
    Requirement,

    // Ports
    PortIn,
    PortOut,

    // Chen ERD
    ChenEntity,
    ChenRelationship,
    ChenAttribute,
    ChenCircle,

    /// Placeholder for undetermined types.
    StillUnknown,
}

/// Set of leaf types that behave like a class (have bodier, members, etc.).
const LIKE_CLASS: &[LeafType] = &[
    LeafType::Annotation,
    LeafType::AbstractClass,
    LeafType::Class,
    LeafType::Interface,
    LeafType::Enum,
    LeafType::Entity,
    LeafType::Protocol,
    LeafType::Struct,
    LeafType::Exception,
    LeafType::Metaclass,
    LeafType::Stereotype,
    LeafType::Dataclass,
    LeafType::Record,
];

impl LeafType {
    /// Parse a leaf type from a string (case-insensitive).
    /// Java: `LeafType.getLeafType(String)`
    pub fn from_str_loose(s: &str) -> Option<Self> {
        let upper = s.to_uppercase();
        if upper.starts_with("ABSTRACT") {
            return Some(LeafType::AbstractClass);
        }
        if upper.starts_with("DIAMOND") {
            return Some(LeafType::StateChoice);
        }
        if upper.starts_with("STATIC") {
            return Some(LeafType::Class);
        }
        Self::from_name(&upper)
    }

    /// Exact match by enum variant name (UPPER_CASE).
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "EMPTY_PACKAGE" => Some(Self::EmptyPackage),
            "ABSTRACT_CLASS" => Some(Self::AbstractClass),
            "CLASS" => Some(Self::Class),
            "INTERFACE" => Some(Self::Interface),
            "ANNOTATION" => Some(Self::Annotation),
            "PROTOCOL" => Some(Self::Protocol),
            "STRUCT" => Some(Self::Struct),
            "EXCEPTION" => Some(Self::Exception),
            "METACLASS" => Some(Self::Metaclass),
            "STEREOTYPE" => Some(Self::Stereotype),
            "LOLLIPOP_FULL" => Some(Self::LollipopFull),
            "LOLLIPOP_HALF" => Some(Self::LollipopHalf),
            "NOTE" => Some(Self::Note),
            "TIPS" => Some(Self::Tips),
            "OBJECT" => Some(Self::Object),
            "MAP" => Some(Self::Map),
            "JSON" => Some(Self::Json),
            "ASSOCIATION" => Some(Self::Association),
            "ENUM" => Some(Self::Enum),
            "CIRCLE" => Some(Self::Circle),
            "DATACLASS" => Some(Self::Dataclass),
            "RECORD" => Some(Self::Record),
            "USECASE" => Some(Self::Usecase),
            "USECASE_BUSINESS" => Some(Self::UsecaseBusiness),
            "DESCRIPTION" => Some(Self::Description),
            "ARC_CIRCLE" => Some(Self::ArcCircle),
            "ACTIVITY" => Some(Self::Activity),
            "BRANCH" => Some(Self::Branch),
            "SYNCHRO_BAR" => Some(Self::SynchroBar),
            "CIRCLE_START" => Some(Self::CircleStart),
            "CIRCLE_END" => Some(Self::CircleEnd),
            "POINT_FOR_ASSOCIATION" => Some(Self::PointForAssociation),
            "ACTIVITY_CONCURRENT" => Some(Self::ActivityConcurrent),
            "STATE" => Some(Self::State),
            "STATE_CONCURRENT" => Some(Self::StateConcurrent),
            "PSEUDO_STATE" => Some(Self::PseudoState),
            "DEEP_HISTORY" => Some(Self::DeepHistory),
            "STATE_CHOICE" => Some(Self::StateChoice),
            "STATE_FORK_JOIN" => Some(Self::StateForkJoin),
            "STATE_TRANSITION_LABEL" => Some(Self::StateTransitionLabel),
            "BLOCK" => Some(Self::Block),
            "ENTITY" => Some(Self::Entity),
            "DOMAIN" => Some(Self::Domain),
            "REQUIREMENT" => Some(Self::Requirement),
            "PORTIN" => Some(Self::PortIn),
            "PORTOUT" => Some(Self::PortOut),
            "CHEN_ENTITY" => Some(Self::ChenEntity),
            "CHEN_RELATIONSHIP" => Some(Self::ChenRelationship),
            "CHEN_ATTRIBUTE" => Some(Self::ChenAttribute),
            "CHEN_CIRCLE" => Some(Self::ChenCircle),
            "STILL_UNKNOWN" => Some(Self::StillUnknown),
            _ => None,
        }
    }

    /// Whether this type behaves like a class (has members, bodier, etc.).
    /// Java: `LeafType.isLikeClass()`
    pub fn is_like_class(&self) -> bool {
        LIKE_CLASS.contains(self)
    }

    /// Human-readable HTML label.
    /// Java: `LeafType.toHtml()`
    pub fn to_html(&self) -> String {
        let raw = format!("{:?}", self);
        // Insert space before uppercase letters (CamelCase -> words)
        let mut result = String::new();
        for (i, ch) in raw.chars().enumerate() {
            if i > 0 && ch.is_uppercase() {
                result.push(' ');
            }
            result.push(ch);
        }
        // Lowercase everything, then capitalize first letter
        let lower = result.to_lowercase();
        let mut chars = lower.chars();
        match chars.next() {
            Some(first) => {
                let upper: String = first.to_uppercase().collect();
                upper + chars.as_str()
            }
            None => String::new(),
        }
    }
}

impl std::fmt::Display for LeafType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_loose_abstract() {
        assert_eq!(
            LeafType::from_str_loose("abstract"),
            Some(LeafType::AbstractClass)
        );
        assert_eq!(
            LeafType::from_str_loose("ABSTRACT_CLASS"),
            Some(LeafType::AbstractClass)
        );
    }

    #[test]
    fn from_str_loose_diamond() {
        assert_eq!(
            LeafType::from_str_loose("diamond"),
            Some(LeafType::StateChoice)
        );
    }

    #[test]
    fn from_str_loose_static() {
        assert_eq!(LeafType::from_str_loose("static"), Some(LeafType::Class));
    }

    #[test]
    fn from_str_loose_exact() {
        assert_eq!(LeafType::from_str_loose("CLASS"), Some(LeafType::Class));
        assert_eq!(LeafType::from_str_loose("class"), Some(LeafType::Class));
        assert_eq!(LeafType::from_str_loose("NOTE"), Some(LeafType::Note));
        assert_eq!(
            LeafType::from_str_loose("INTERFACE"),
            Some(LeafType::Interface)
        );
    }

    #[test]
    fn from_str_loose_unknown() {
        assert_eq!(LeafType::from_str_loose("NONEXISTENT"), None);
    }

    #[test]
    fn is_like_class() {
        assert!(LeafType::Class.is_like_class());
        assert!(LeafType::Interface.is_like_class());
        assert!(LeafType::Enum.is_like_class());
        assert!(LeafType::Annotation.is_like_class());
        assert!(LeafType::Record.is_like_class());
        assert!(!LeafType::Note.is_like_class());
        assert!(!LeafType::State.is_like_class());
        assert!(!LeafType::Usecase.is_like_class());
    }

    #[test]
    fn to_html_format() {
        assert_eq!(LeafType::Class.to_html(), "Class");
        assert_eq!(LeafType::AbstractClass.to_html(), "Abstract class");
        assert_eq!(LeafType::StillUnknown.to_html(), "Still unknown");
    }

    #[test]
    fn display_impl() {
        assert_eq!(format!("{}", LeafType::Class), "Class");
        assert_eq!(format!("{}", LeafType::StateForkJoin), "StateForkJoin");
    }

    #[test]
    fn all_variants_distinct() {
        use std::collections::HashSet;
        let variants = vec![
            LeafType::EmptyPackage,
            LeafType::AbstractClass,
            LeafType::Class,
            LeafType::Interface,
            LeafType::Annotation,
            LeafType::Protocol,
            LeafType::Struct,
            LeafType::Exception,
            LeafType::Metaclass,
            LeafType::Stereotype,
            LeafType::LollipopFull,
            LeafType::LollipopHalf,
            LeafType::Note,
            LeafType::Tips,
            LeafType::Object,
            LeafType::Map,
            LeafType::Json,
            LeafType::Association,
            LeafType::Enum,
            LeafType::Circle,
            LeafType::Dataclass,
            LeafType::Record,
            LeafType::Usecase,
            LeafType::UsecaseBusiness,
            LeafType::Description,
            LeafType::ArcCircle,
            LeafType::Activity,
            LeafType::Branch,
            LeafType::SynchroBar,
            LeafType::CircleStart,
            LeafType::CircleEnd,
            LeafType::PointForAssociation,
            LeafType::ActivityConcurrent,
            LeafType::State,
            LeafType::StateConcurrent,
            LeafType::PseudoState,
            LeafType::DeepHistory,
            LeafType::StateChoice,
            LeafType::StateForkJoin,
            LeafType::StateTransitionLabel,
            LeafType::Block,
            LeafType::Entity,
            LeafType::Domain,
            LeafType::Requirement,
            LeafType::PortIn,
            LeafType::PortOut,
            LeafType::ChenEntity,
            LeafType::ChenRelationship,
            LeafType::ChenAttribute,
            LeafType::ChenCircle,
            LeafType::StillUnknown,
        ];
        let set: HashSet<_> = variants.iter().collect();
        assert_eq!(set.len(), variants.len(), "all variants must be distinct");
    }
}
