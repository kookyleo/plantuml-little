// Style selector names used by the style system.
// Port of Java PlantUML's `net.sourceforge.plantuml.style.SName`

use std::collections::HashMap;
use std::sync::LazyLock;

/// Style selector name — identifies an element type or context in style matching.
///
/// Java: `style.SName`
///
/// Variant names follow Java conventions (lowercase start) and trailing
/// underscores map to Java's keyword-avoiding suffixes (e.g. `class_` in Java
/// becomes `Class` in Rust because `class` is not a reserved word in Rust but
/// we keep Java's naming intent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SName {
    Action,
    ActivationBox,
    Activity,
    ActivityBar,
    ActivityDiagram,
    Actor,
    Agent,
    Analog,
    Archimate,
    Arrow,
    Artifact,
    Binary,
    Boundary,
    Box,
    Boxless,
    Business,
    Caption,
    Card,
    Cardinality,
    Circle,
    ClassDiagram,
    Class,
    Clickable,
    Cloud,
    Closed,
    Collection,
    Collections,
    Component,
    Composite,
    Robust,
    ChenAttribute,
    ChenEerDiagram,
    ChenEntity,
    ChenRelationship,
    Concise,
    Clock,
    ComponentDiagram,
    ConstraintArrow,
    Control,
    Database,
    Day,
    Delay,
    Destroy,
    Diamond,
    Document,
    Ebnf,
    Element,
    Entity,
    End,
    Start,
    Stop,
    File,
    FilesDiagram,
    Folder,
    Footer,
    Frame,
    GanttDiagram,
    Generic,
    Goto,
    Group,
    GroupHeader,
    Header,
    Hexagon,
    Highlight,
    Hnote,
    Interface,
    Json,
    JsonDiagram,
    GitDiagram,
    Label,
    LeafNode,
    Legend,
    LifeLine,
    Mainframe,
    Map,
    Milestone,
    MindmapDiagram,
    Month,
    Network,
    Newpage,
    Node,
    Note,
    NwdiagDiagram,
    PacketdiagDiagram,
    ObjectDiagram,
    Object,
    Package,
    Participant,
    Partition,
    Person,
    Port,
    Process,
    Qualified,
    Queue,
    Rectangle,
    Reference,
    ReferenceHeader,
    Regex,
    Requirement,
    Rnote,
    Root,
    RootNode,
    SaltDiagram,
    Separator,
    SequenceDiagram,
    Server,
    Stack,
    StateDiagram,
    State,
    StateBody,
    Stereotype,
    Storage,
    Swimlane,
    Task,
    Timegrid,
    Timeline,
    TimingDiagram,
    Title,
    Undone,
    Unstarted,
    Usecase,
    VerticalSeparator,
    Year,
    VisibilityIcon,
    Private,
    Protected,
    Public,
    IEMandatory,
    Spot,
    SpotAnnotation,
    SpotInterface,
    SpotEnum,
    SpotProtocol,
    SpotStruct,
    SpotEntity,
    SpotException,
    SpotClass,
    SpotAbstractClass,
    SpotMetaClass,
    SpotStereotype,
    SpotDataClass,
    SpotRecord,
    WbsDiagram,
    YamlDiagram,
    ChartDiagram,
    // Chart elements
    Bar,
    Line,
    Area,
    Scatter,
    Axis,
    HAxis,
    VAxis,
    Grid,
    Annotation,
}

/// Lookup table: lowercased name (with trailing underscores removed) -> SName.
static SNAME_MAP: LazyLock<HashMap<String, SName>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(ALL_SNAMES.len());
    for &(sname, java_name) in ALL_SNAMES {
        // Java does: sname.name().replace("_", "").toLowerCase()
        let key = java_name.replace('_', "").to_ascii_lowercase();
        map.insert(key, sname);
    }
    map
});

impl SName {
    /// Retrieve an SName by its CSS/Java name (case-insensitive, underscores stripped).
    /// Java: `SName.retrieve(String)`
    pub fn retrieve(s: &str) -> Option<SName> {
        SNAME_MAP.get(&s.to_ascii_lowercase()).copied()
    }

    /// Returns the Java-compatible enum name (lowercase with trailing underscore
    /// for keyword-colliding names like `class_`, `interface_`, etc.).
    pub fn java_name(self) -> &'static str {
        for &(sname, name) in ALL_SNAMES {
            if sname == self {
                return name;
            }
        }
        unreachable!()
    }
}

impl std::fmt::Display for SName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.java_name())
    }
}

/// All SName variants paired with their Java enum name strings.
/// Ordering matches the Java source exactly.
const ALL_SNAMES: &[(SName, &str)] = &[
    (SName::Action, "action"),
    (SName::ActivationBox, "activationBox"),
    (SName::Activity, "activity"),
    (SName::ActivityBar, "activityBar"),
    (SName::ActivityDiagram, "activityDiagram"),
    (SName::Actor, "actor"),
    (SName::Agent, "agent"),
    (SName::Analog, "analog"),
    (SName::Archimate, "archimate"),
    (SName::Arrow, "arrow"),
    (SName::Artifact, "artifact"),
    (SName::Binary, "binary"),
    (SName::Boundary, "boundary"),
    (SName::Box, "box"),
    (SName::Boxless, "boxless"),
    (SName::Business, "business"),
    (SName::Caption, "caption"),
    (SName::Card, "card"),
    (SName::Cardinality, "cardinality"),
    (SName::Circle, "circle"),
    (SName::ClassDiagram, "classDiagram"),
    (SName::Class, "class_"),
    (SName::Clickable, "clickable"),
    (SName::Cloud, "cloud"),
    (SName::Closed, "closed"),
    (SName::Collection, "collection"),
    (SName::Collections, "collections"),
    (SName::Component, "component"),
    (SName::Composite, "composite"),
    (SName::Robust, "robust"),
    (SName::ChenAttribute, "chenAttribute"),
    (SName::ChenEerDiagram, "chenEerDiagram"),
    (SName::ChenEntity, "chenEntity"),
    (SName::ChenRelationship, "chenRelationship"),
    (SName::Concise, "concise"),
    (SName::Clock, "clock"),
    (SName::ComponentDiagram, "componentDiagram"),
    (SName::ConstraintArrow, "constraintArrow"),
    (SName::Control, "control"),
    (SName::Database, "database"),
    (SName::Day, "day"),
    (SName::Delay, "delay"),
    (SName::Destroy, "destroy"),
    (SName::Diamond, "diamond"),
    (SName::Document, "document"),
    (SName::Ebnf, "ebnf"),
    (SName::Element, "element"),
    (SName::Entity, "entity"),
    (SName::End, "end"),
    (SName::Start, "start"),
    (SName::Stop, "stop"),
    (SName::File, "file"),
    (SName::FilesDiagram, "filesDiagram"),
    (SName::Folder, "folder"),
    (SName::Footer, "footer"),
    (SName::Frame, "frame"),
    (SName::GanttDiagram, "ganttDiagram"),
    (SName::Generic, "generic"),
    (SName::Goto, "goto_"),
    (SName::Group, "group"),
    (SName::GroupHeader, "groupHeader"),
    (SName::Header, "header"),
    (SName::Hexagon, "hexagon"),
    (SName::Highlight, "highlight"),
    (SName::Hnote, "hnote"),
    (SName::Interface, "interface_"),
    (SName::Json, "json"),
    (SName::JsonDiagram, "jsonDiagram"),
    (SName::GitDiagram, "gitDiagram"),
    (SName::Label, "label"),
    (SName::LeafNode, "leafNode"),
    (SName::Legend, "legend"),
    (SName::LifeLine, "lifeLine"),
    (SName::Mainframe, "mainframe"),
    (SName::Map, "map"),
    (SName::Milestone, "milestone"),
    (SName::MindmapDiagram, "mindmapDiagram"),
    (SName::Month, "month"),
    (SName::Network, "network"),
    (SName::Newpage, "newpage"),
    (SName::Node, "node"),
    (SName::Note, "note"),
    (SName::NwdiagDiagram, "nwdiagDiagram"),
    (SName::PacketdiagDiagram, "packetdiagDiagram"),
    (SName::ObjectDiagram, "objectDiagram"),
    (SName::Object, "object"),
    (SName::Package, "package_"),
    (SName::Participant, "participant"),
    (SName::Partition, "partition"),
    (SName::Person, "person"),
    (SName::Port, "port"),
    (SName::Process, "process"),
    (SName::Qualified, "qualified"),
    (SName::Queue, "queue"),
    (SName::Rectangle, "rectangle"),
    (SName::Reference, "reference"),
    (SName::ReferenceHeader, "referenceHeader"),
    (SName::Regex, "regex"),
    (SName::Requirement, "requirement"),
    (SName::Rnote, "rnote"),
    (SName::Root, "root"),
    (SName::RootNode, "rootNode"),
    (SName::SaltDiagram, "saltDiagram"),
    (SName::Separator, "separator"),
    (SName::SequenceDiagram, "sequenceDiagram"),
    (SName::Server, "server"),
    (SName::Stack, "stack"),
    (SName::StateDiagram, "stateDiagram"),
    (SName::State, "state"),
    (SName::StateBody, "stateBody"),
    (SName::Stereotype, "stereotype"),
    (SName::Storage, "storage"),
    (SName::Swimlane, "swimlane"),
    (SName::Task, "task"),
    (SName::Timegrid, "timegrid"),
    (SName::Timeline, "timeline"),
    (SName::TimingDiagram, "timingDiagram"),
    (SName::Title, "title"),
    (SName::Undone, "undone"),
    (SName::Unstarted, "unstarted"),
    (SName::Usecase, "usecase"),
    (SName::VerticalSeparator, "verticalSeparator"),
    (SName::Year, "year"),
    (SName::VisibilityIcon, "visibilityIcon"),
    (SName::Private, "private_"),
    (SName::Protected, "protected_"),
    (SName::Public, "public_"),
    (SName::IEMandatory, "IEMandatory"),
    (SName::Spot, "spot"),
    (SName::SpotAnnotation, "spotAnnotation"),
    (SName::SpotInterface, "spotInterface"),
    (SName::SpotEnum, "spotEnum"),
    (SName::SpotProtocol, "spotProtocol"),
    (SName::SpotStruct, "spotStruct"),
    (SName::SpotEntity, "spotEntity"),
    (SName::SpotException, "spotException"),
    (SName::SpotClass, "spotClass"),
    (SName::SpotAbstractClass, "spotAbstractClass"),
    (SName::SpotMetaClass, "spotMetaClass"),
    (SName::SpotStereotype, "spotStereotype"),
    (SName::SpotDataClass, "spotDataClass"),
    (SName::SpotRecord, "spotRecord"),
    (SName::WbsDiagram, "wbsDiagram"),
    (SName::YamlDiagram, "yamlDiagram"),
    (SName::ChartDiagram, "chartDiagram"),
    (SName::Bar, "bar"),
    (SName::Line, "line"),
    (SName::Area, "area"),
    (SName::Scatter, "scatter"),
    (SName::Axis, "axis"),
    (SName::HAxis, "hAxis"),
    (SName::VAxis, "vAxis"),
    (SName::Grid, "grid"),
    (SName::Annotation, "annotation"),
];

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_count_matches_java() {
        // Java SName has 154 variants (action..annotation)
        assert_eq!(ALL_SNAMES.len(), 154);
    }

    #[test]
    fn retrieve_simple() {
        assert_eq!(SName::retrieve("arrow"), Some(SName::Arrow));
        assert_eq!(SName::retrieve("root"), Some(SName::Root));
        assert_eq!(SName::retrieve("note"), Some(SName::Note));
    }

    #[test]
    fn retrieve_case_insensitive() {
        assert_eq!(SName::retrieve("Arrow"), Some(SName::Arrow));
        assert_eq!(SName::retrieve("ARROW"), Some(SName::Arrow));
        assert_eq!(
            SName::retrieve("SequenceDiagram"),
            Some(SName::SequenceDiagram)
        );
    }

    #[test]
    fn retrieve_keyword_variants_strip_underscore() {
        // Java: class_ -> lookup key "class"
        assert_eq!(SName::retrieve("class"), Some(SName::Class));
        assert_eq!(SName::retrieve("interface"), Some(SName::Interface));
        assert_eq!(SName::retrieve("package"), Some(SName::Package));
        assert_eq!(SName::retrieve("goto"), Some(SName::Goto));
        assert_eq!(SName::retrieve("private"), Some(SName::Private));
        assert_eq!(SName::retrieve("protected"), Some(SName::Protected));
        assert_eq!(SName::retrieve("public"), Some(SName::Public));
    }

    #[test]
    fn retrieve_unknown_returns_none() {
        assert_eq!(SName::retrieve("noSuchElement"), None);
        assert_eq!(SName::retrieve(""), None);
    }

    #[test]
    fn java_name_matches() {
        assert_eq!(SName::Arrow.java_name(), "arrow");
        assert_eq!(SName::Class.java_name(), "class_");
        assert_eq!(SName::Interface.java_name(), "interface_");
        assert_eq!(SName::IEMandatory.java_name(), "IEMandatory");
    }

    #[test]
    fn display_uses_java_name() {
        assert_eq!(format!("{}", SName::Arrow), "arrow");
        assert_eq!(format!("{}", SName::Class), "class_");
    }

    #[test]
    fn all_variants_unique() {
        let mut seen = std::collections::HashSet::new();
        for &(sname, _) in ALL_SNAMES {
            assert!(seen.insert(sname), "duplicate variant: {:?}", sname);
        }
    }

    #[test]
    fn all_java_names_unique() {
        let mut seen = std::collections::HashSet::new();
        for &(_, name) in ALL_SNAMES {
            assert!(seen.insert(name), "duplicate java_name: {}", name);
        }
    }

    #[test]
    fn retrieve_roundtrip_all() {
        for &(sname, java_name) in ALL_SNAMES {
            let key = java_name.replace('_', "");
            let result = SName::retrieve(&key);
            assert_eq!(
                result,
                Some(sname),
                "roundtrip failed for java_name={}",
                java_name
            );
        }
    }

    #[test]
    fn chart_elements_present() {
        assert_eq!(SName::retrieve("bar"), Some(SName::Bar));
        assert_eq!(SName::retrieve("line"), Some(SName::Line));
        assert_eq!(SName::retrieve("area"), Some(SName::Area));
        assert_eq!(SName::retrieve("scatter"), Some(SName::Scatter));
        assert_eq!(SName::retrieve("axis"), Some(SName::Axis));
        assert_eq!(SName::retrieve("haxis"), Some(SName::HAxis));
        assert_eq!(SName::retrieve("vaxis"), Some(SName::VAxis));
        assert_eq!(SName::retrieve("grid"), Some(SName::Grid));
        assert_eq!(SName::retrieve("annotation"), Some(SName::Annotation));
    }

    #[test]
    fn spot_variants_present() {
        assert_eq!(SName::retrieve("spot"), Some(SName::Spot));
        assert_eq!(
            SName::retrieve("spotannotation"),
            Some(SName::SpotAnnotation)
        );
        assert_eq!(SName::retrieve("spotinterface"), Some(SName::SpotInterface));
        assert_eq!(SName::retrieve("spotenum"), Some(SName::SpotEnum));
        assert_eq!(SName::retrieve("spotclass"), Some(SName::SpotClass));
        assert_eq!(
            SName::retrieve("spotabstractclass"),
            Some(SName::SpotAbstractClass)
        );
        assert_eq!(SName::retrieve("spotrecord"), Some(SName::SpotRecord));
    }
}
