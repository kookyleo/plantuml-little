// ISkinParam trait — full interface for diagram skin configuration.
// Port of Java PlantUML's style.ISkinParam and style.ISkinSimple

use std::collections::HashMap;

use crate::klimt::color::HColor;
use crate::klimt::geom::{HorizontalAlignment, Rankdir};
use crate::klimt::{LineBreakStrategy, UStroke};

use super::style_def::{Style, StyleBuilder};
use super::value::LengthAdjust;

// ── Enums referenced by ISkinParam ──────────────────────────────────

/// Arrow direction for alignment queries.
/// Java: `skin.ArrowDirection`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
    Self_,
}

/// Actor rendering style.
/// Java: `skin.ActorStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActorStyle {
    #[default]
    Stickman,
    StickmanBusiness,
    Awesome,
    Hollow,
}

/// Component rendering style.
/// Java: `skin.ComponentStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComponentStyle {
    #[default]
    Uml2,
    Uml1,
    Rectangle,
}

/// Package rendering style.
/// Java: `svek.PackageStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PackageStyle {
    #[default]
    Folder,
    Rectangle,
    Node,
    Frame,
    Cloud,
    Database,
    Card,
}

/// Condition (diamond) rendering style in activity diagrams.
/// Java: `svek.ConditionStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ConditionStyle {
    #[default]
    Diamond,
    InsideDiamond,
    Foo1,
}

/// Condition-end rendering style in activity diagrams.
/// Java: `svek.ConditionEndStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ConditionEndStyle {
    #[default]
    Diamond,
    Hline,
}

/// Graphviz edge routing strategy.
/// Java: `dot.DotSplines`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DotSplines {
    #[default]
    Spline,
    Line,
    Ortho,
    Polyline,
    Curved,
}

/// Diagram type identifier.
/// Java: `core.DiagramType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DiagramType {
    #[default]
    Sequence,
    Class,
    Activity,
    State,
    Component,
    Object,
    Usecase,
    Timing,
    Gantt,
    Mindmap,
    Wbs,
    Json,
    Yaml,
    Salt,
    Nwdiag,
    Git,
    Ebnf,
    Files,
    Other,
}

/// Guillemet style for stereotypes.
/// Java: `text.Guillemet`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Guillemet {
    #[default]
    DoubleAngle,
    HtmlEntity,
    None,
}

/// Split-page parameters.
/// Java: `skin.SplitParam`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SplitParam {
    pub border_color: Option<HColor>,
    pub external_color: Option<HColor>,
}

/// TikZ font distortion parameters.
/// Java: `TikzFontDistortion`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TikzFontDistortion {
    pub distortion: f64,
    pub shrink: f64,
    pub extend: f64,
}

/// Padder for sequence diagram lifelines.
/// Java: `skin.Padder`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Padder {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Arrows configuration.
/// Java: `klimt.Arrows`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Arrows;

/// Alignment parameter identifiers.
/// Java: `skin.AlignmentParam`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignmentParam {
    ArrowMessageAlignment,
    SequenceMessageAlignment,
    SequenceMessageTextAlignment,
    SequenceReferenceAlignment,
}

/// Color parameter identifiers for legacy skinparam.
/// Java: `skin.ColorParam`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorParam {
    Background,
    // Sequence
    ParticipantBackground,
    ParticipantBorder,
    SequenceLifelineBackground,
    SequenceLifelineBorder,
    SequenceGroupBackground,
    SequenceGroupBorder,
    SequenceBoxBackground,
    SequenceBoxBorder,
    SequenceReferenceBackground,
    SequenceReferenceBorder,
    SequenceDividerBackground,
    SequenceDividerBorder,
    // Note
    NoteBackground,
    NoteBorder,
    // Class
    ClassBackground,
    ClassBorder,
    ClassHeader,
    // State
    StateBackground,
    StateBorder,
    // Activity
    ActivityBackground,
    ActivityBorder,
    ActivityDiamondBackground,
    ActivityDiamondBorder,
    // Other
    ArrowHead,
    ArrowColor,
    ObjectBackground,
    ObjectBorder,
    PackageBackground,
    PackageBorder,
    ComponentBackground,
    ComponentBorder,
    Swimlane,
    SwimlaneBorder,
    SwimlaneTitle,
    IconPrivate,
    IconProtected,
    IconPublic,
    IconPackage,
}

/// Font parameter identifiers for legacy skinparam.
/// Java: `klimt.font.FontParam`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontParam {
    Default,
    Title,
    Caption,
    Header,
    Footer,
    Legend,
    Note,
    Arrow,
    Activity,
    ActivityDiamond,
    Participant,
    Sequence,
    SequenceGroup,
    SequenceGroupHeader,
    SequenceDelay,
    SequenceDivider,
    SequenceReference,
    SequenceBox,
    Class,
    ClassAttribute,
    Object,
    ObjectAttribute,
    State,
    StateAttribute,
    Component,
    Package,
    Swimlane,
    Node,
}

/// Line parameter identifiers for legacy skinparam.
/// Java: `skin.LineParam`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineParam {
    ArrowThickness,
    NoteBorderThickness,
    PackageBorderThickness,
    ClassBorderThickness,
    StateBorderThickness,
    ObjectBorderThickness,
    SequenceDividerBorderThickness,
    SequenceGroupBorderThickness,
    ActivityBorderThickness,
    SequenceReferenceBorderThickness,
    TitleBorderThickness,
    LegendBorderThickness,
    SwimlaneBorderThickness,
}

/// Corner rounding parameter identifiers.
/// Java: `skin.CornerParam`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerParam {
    Default,
    Activity,
    Class,
    Participant,
    Component,
    Package,
    Node,
    State,
    Object,
    Title,
    Legend,
}

/// Padding parameter identifiers.
/// Java: `skin.PaddingParam`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaddingParam {
    ParticipantBox,
    Participant,
    LifeLine,
}

// ── ISkinSimple ──────────────────────────────────────────────────────

/// Simplified skin interface (subset used by rendering).
/// Java: `style.ISkinSimple`
///
/// Provides key-value parameter access, color set, DPI, and text creation.
pub trait ISkinSimple {
    /// Get a string parameter value by key.
    fn get_value(&self, key: &str) -> Option<&str>;

    /// All string key-value pairs.
    fn values(&self) -> &HashMap<String, String>;

    /// Global padding value.
    fn get_padding(&self) -> f64 { 0.0 }

    /// Monospaced font family name.
    fn monospaced_family(&self) -> &str { "monospace" }

    /// Tab size in spaces.
    fn tab_size(&self) -> i32 { 8 }

    /// DPI for the output.
    fn dpi(&self) -> i32 { 96 }
}

// ── ISkinParam ───────────────────────────────────────────────────────

/// Full skin parameter interface for diagram rendering.
///
/// Java: `style.ISkinParam extends ISkinSimple`
///
/// This is the central configuration interface that diagram renderers
/// query for all visual parameters: colors, fonts, strokes, layout
/// options, etc.
///
/// The methods here correspond 1:1 with the Java interface. Default
/// implementations return sensible values matching PlantUML's "Rose"
/// theme defaults.
#[allow(unused_variables)]
pub trait ISkinParam: ISkinSimple {
    // ── Swimlane width sentinel ─────────────────────────────────────
    /// Sentinel value meaning "make all swimlanes the same width".
    /// Java: `ISkinParam.SWIMLANE_WIDTH_SAME`
    const SWIMLANE_WIDTH_SAME: i32 = -1;

    // ── Colors ──────────────────────────────────────────────────────

    fn hyperlink_color(&self) -> HColor { HColor::simple("#0000FF") }
    fn use_underline_for_hyperlink(&self) -> UStroke { UStroke::with_thickness(1.0) }
    fn background_color(&self) -> HColor { HColor::simple("#FFFFFF") }

    fn html_color(&self, param: ColorParam, stereotype: Option<&str>, clickable: bool) -> Option<HColor> {
        None
    }

    fn font_html_color(&self, stereotype: Option<&str>, params: &[FontParam]) -> Option<HColor> {
        None
    }

    fn thickness(&self, param: LineParam, stereotype: Option<&str>) -> UStroke {
        UStroke::with_thickness(1.0)
    }

    // ── Fonts ───────────────────────────────────────────────────────

    fn font(&self, stereotype: Option<&str>, in_package_title: bool, params: &[FontParam]) -> Option<(String, f64, bool, bool)> {
        None
    }

    // ── Alignment ───────────────────────────────────────────────────

    fn horizontal_alignment(
        &self,
        param: AlignmentParam,
        arrow_direction: Option<ArrowDirection>,
        is_reverse_define: bool,
        override_default: Option<HorizontalAlignment>,
    ) -> HorizontalAlignment {
        override_default.unwrap_or(HorizontalAlignment::Center)
    }

    fn default_text_alignment(&self, default: HorizontalAlignment) -> HorizontalAlignment {
        default
    }

    fn stereotype_alignment(&self) -> HorizontalAlignment { HorizontalAlignment::Center }

    // ── Class / Object ──────────────────────────────────────────────

    fn circled_character_radius(&self) -> i32 { 11 }
    fn circled_character(&self, stereotype: Option<&str>) -> char { ' ' }
    fn class_attribute_icon_size(&self) -> i32 { 10 }

    // ── Layout ──────────────────────────────────────────────────────

    fn dot_splines(&self) -> DotSplines { DotSplines::Spline }
    fn nodesep(&self) -> f64 { 25.0 }
    fn ranksep(&self) -> f64 { 40.0 }
    fn rankdir(&self) -> Rankdir { Rankdir::TopToBottom }

    // ── Shadowing ───────────────────────────────────────────────────

    fn shadowing(&self, stereotype: Option<&str>) -> bool { false }
    fn shadowing_for_note(&self, stereotype: Option<&str>) -> bool { false }

    // ── Styles ──────────────────────────────────────────────────────

    fn package_style(&self) -> PackageStyle { PackageStyle::Folder }
    fn component_style(&self) -> ComponentStyle { ComponentStyle::Uml2 }
    fn actor_style(&self) -> ActorStyle { ActorStyle::Stickman }

    // ── Corners ─────────────────────────────────────────────────────

    fn round_corner(&self, param: CornerParam, stereotype: Option<&str>) -> f64 { 0.0 }
    fn diagonal_corner(&self, param: CornerParam, stereotype: Option<&str>) -> f64 { 0.0 }

    // ── Behavior flags ──────────────────────────────────────────────

    fn stereotype_position_top(&self) -> bool { true }
    fn use_swimlanes(&self, dtype: DiagramType) -> bool { false }
    fn strict_uml_style(&self) -> bool { false }
    fn force_sequence_participant_underlined(&self) -> bool { false }
    fn same_class_width(&self) -> bool { false }
    fn use_octagon_for_activity(&self, stereotype: Option<&str>) -> bool { false }
    fn handwritten(&self) -> bool { false }
    fn use_rank_same(&self) -> bool { false }
    fn display_generic_with_old_fashion(&self) -> bool { false }
    fn response_message_below_arrow(&self) -> bool { false }
    fn svg_dimension_style(&self) -> bool { true }
    fn fix_circle_label_overlapping(&self) -> bool { false }

    // ── VizJs ───────────────────────────────────────────────────────

    fn set_use_vizjs(&mut self, use_vizjs: bool) {}
    fn is_use_vizjs(&self) -> bool { false }

    // ── Conditions (activity diagrams) ──────────────────────────────

    fn condition_style(&self) -> ConditionStyle { ConditionStyle::Diamond }
    fn condition_end_style(&self) -> ConditionEndStyle { ConditionEndStyle::Diamond }

    // ── Line break / wrapping ───────────────────────────────────────

    fn max_message_size(&self) -> LineBreakStrategy { LineBreakStrategy::None }
    fn swimlane_wrap_title_width(&self) -> LineBreakStrategy { LineBreakStrategy::None }

    // ── Group inheritance ───────────────────────────────────────────

    fn group_inheritance(&self) -> i32 { 0 }

    // ── Guillemets ──────────────────────────────────────────────────

    fn guillemet(&self) -> Guillemet { Guillemet::DoubleAngle }

    // ── Output config ───────────────────────────────────────────────

    fn svg_link_target(&self) -> &str { "_top" }
    fn preserve_aspect_ratio(&self) -> &str { "none" }
    fn max_ascii_message_length(&self) -> i32 { -1 }
    fn color_arrow_separation_space(&self) -> i32 { 0 }
    fn swimlane_width(&self) -> i32 { 0 }
    fn length_adjust(&self) -> LengthAdjust { LengthAdjust::Spacing }
    fn param_same_class_width(&self) -> f64 { 0.0 }

    // ── Diagram type ────────────────────────────────────────────────

    fn diagram_type(&self) -> DiagramType { DiagramType::Sequence }

    // ── Misc ────────────────────────────────────────────────────────

    fn hover_path_color(&self) -> Option<HColor> { None }
    fn tikz_font_distortion(&self) -> TikzFontDistortion { TikzFontDistortion::default() }
    fn padding(&self, param: PaddingParam) -> f64 { 0.0 }
    fn split_param(&self) -> SplitParam { SplitParam::default() }
    fn arrows(&self) -> Arrows { Arrows }

    // ── Style builder ───────────────────────────────────────────────

    fn current_style_builder(&self) -> Option<&StyleBuilder> { None }
    fn mute_style(&mut self, modified: &[Style]) {}

    // ── Sprites ─────────────────────────────────────────────────────

    fn all_sprite_names(&self) -> Vec<String> { Vec::new() }

    // ── Skin ────────────────────────────────────────────────────────

    fn default_skin(&self) -> &str { "plantuml.skin" }
    fn set_default_skin(&mut self, skin: &str) {}

    // ── SVG size ────────────────────────────────────────────────────

    fn set_svg_size(&mut self, origin: &str, size_to_use: &str) {}
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal concrete implementation for testing
    struct TestSkinParam {
        values: HashMap<String, String>,
    }

    impl TestSkinParam {
        fn new() -> Self {
            Self { values: HashMap::new() }
        }
    }

    impl ISkinSimple for TestSkinParam {
        fn get_value(&self, key: &str) -> Option<&str> {
            self.values.get(key).map(|s| s.as_str())
        }
        fn values(&self) -> &HashMap<String, String> {
            &self.values
        }
    }

    impl ISkinParam for TestSkinParam {}

    #[test]
    fn default_trait_methods() {
        let skin = TestSkinParam::new();
        assert_eq!(skin.background_color(), HColor::simple("#FFFFFF"));
        assert_eq!(skin.dot_splines(), DotSplines::Spline);
        assert_eq!(skin.nodesep(), 25.0);
        assert_eq!(skin.ranksep(), 40.0);
        assert!(!skin.handwritten());
        assert!(!skin.strict_uml_style());
        assert_eq!(skin.circled_character_radius(), 11);
        assert_eq!(skin.class_attribute_icon_size(), 10);
        assert!(skin.stereotype_position_top());
        assert!(!skin.is_use_vizjs());
        assert_eq!(skin.max_ascii_message_length(), -1);
        assert_eq!(skin.svg_link_target(), "_top");
        assert_eq!(skin.preserve_aspect_ratio(), "none");
        assert_eq!(skin.default_skin(), "plantuml.skin");
        assert_eq!(skin.diagram_type(), DiagramType::Sequence);
        assert_eq!(skin.rankdir(), Rankdir::TopToBottom);
    }

    #[test]
    fn default_skin_simple_methods() {
        let skin = TestSkinParam::new();
        assert_eq!(skin.get_padding(), 0.0);
        assert_eq!(skin.monospaced_family(), "monospace");
        assert_eq!(skin.tab_size(), 8);
        assert_eq!(skin.dpi(), 96);
    }

    #[test]
    fn alignment_defaults() {
        let skin = TestSkinParam::new();
        assert_eq!(
            skin.horizontal_alignment(
                AlignmentParam::ArrowMessageAlignment,
                None,
                false,
                None,
            ),
            HorizontalAlignment::Center,
        );
        assert_eq!(skin.stereotype_alignment(), HorizontalAlignment::Center);
        assert_eq!(
            skin.default_text_alignment(HorizontalAlignment::Left),
            HorizontalAlignment::Left,
        );
    }

    #[test]
    fn condition_style_defaults() {
        let skin = TestSkinParam::new();
        assert_eq!(skin.condition_style(), ConditionStyle::Diamond);
        assert_eq!(skin.condition_end_style(), ConditionEndStyle::Diamond);
    }

    #[test]
    fn component_styles() {
        let skin = TestSkinParam::new();
        assert_eq!(skin.package_style(), PackageStyle::Folder);
        assert_eq!(skin.component_style(), ComponentStyle::Uml2);
        assert_eq!(skin.actor_style(), ActorStyle::Stickman);
    }

    #[test]
    fn enum_debug() {
        // Verify all enums impl Debug
        let _ = format!("{:?}", ArrowDirection::LeftToRight);
        let _ = format!("{:?}", ActorStyle::Stickman);
        let _ = format!("{:?}", ComponentStyle::Uml2);
        let _ = format!("{:?}", PackageStyle::Folder);
        let _ = format!("{:?}", ConditionStyle::Diamond);
        let _ = format!("{:?}", ConditionEndStyle::Diamond);
        let _ = format!("{:?}", DotSplines::Spline);
        let _ = format!("{:?}", DiagramType::Sequence);
        let _ = format!("{:?}", Guillemet::DoubleAngle);
    }

    #[test]
    fn hover_and_misc() {
        let skin = TestSkinParam::new();
        assert_eq!(skin.hover_path_color(), None);
        assert_eq!(skin.split_param(), SplitParam::default());
        assert_eq!(skin.tikz_font_distortion(), TikzFontDistortion::default());
        assert_eq!(skin.padding(PaddingParam::Participant), 0.0);
        assert_eq!(skin.length_adjust(), LengthAdjust::Spacing);
    }

    #[test]
    fn skin_values() {
        let mut skin = TestSkinParam::new();
        skin.values.insert("key1".into(), "value1".into());
        assert_eq!(skin.get_value("key1"), Some("value1"));
        assert_eq!(skin.get_value("missing"), None);
        assert_eq!(skin.values().len(), 1);
    }

    #[test]
    fn swimlane_width_sentinel() {
        // Can reference the associated constant
        assert_eq!(<TestSkinParam as ISkinParam>::SWIMLANE_WIDTH_SAME, -1);
    }

    #[test]
    fn shadowing_defaults() {
        let skin = TestSkinParam::new();
        assert!(!skin.shadowing(None));
        assert!(!skin.shadowing_for_note(None));
    }

    #[test]
    fn color_returns_none() {
        let skin = TestSkinParam::new();
        assert!(skin.html_color(ColorParam::Background, None, false).is_none());
        assert!(skin.font_html_color(None, &[FontParam::Default]).is_none());
    }

    #[test]
    fn all_sprite_names_empty() {
        let skin = TestSkinParam::new();
        assert!(skin.all_sprite_names().is_empty());
    }
}
