// skin::component - Component type definitions
// Port of Java PlantUML's skin.ComponentType + ComponentStyle

/// Sequence diagram component types. Java: `skin.ComponentType`
///
/// Covers all visual components used in sequence diagram rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType {
    // Arrow
    Arrow,

    // Actor head / tail pairs
    ActorHead,
    ActorTail,

    // Boundary head / tail
    BoundaryHead,
    BoundaryTail,

    // Control head / tail
    ControlHead,
    ControlTail,

    // Entity head / tail
    EntityHead,
    EntityTail,

    // Queue head / tail
    QueueHead,
    QueueTail,

    // Database head / tail
    DatabaseHead,
    DatabaseTail,

    // Collections head / tail
    CollectionsHead,
    CollectionsTail,

    // Activation boxes (open/close combinations)
    ActivationBoxCloseClose,
    ActivationBoxCloseOpen,
    ActivationBoxOpenClose,
    ActivationBoxOpenOpen,

    // Delay
    DelayText,
    Destroy,
    DelayLine,

    // Participant line
    ParticipantLine,

    // Grouping
    GroupingElseLegacy,
    GroupingElseTeoz,
    GroupingHeaderLegacy,
    GroupingHeaderTeoz,
    GroupingSpace,

    // Misc
    Newpage,
    Note,
    NoteHexagonal,
    NoteBox,
    Divider,
    Reference,
    Englober,

    // Participant head / tail
    ParticipantHead,
    ParticipantTail,
}

impl ComponentType {
    /// Whether this component type is an arrow.
    pub fn is_arrow(self) -> bool {
        self == ComponentType::Arrow
    }

    /// Whether this is a "head" participant type.
    pub fn is_head(self) -> bool {
        matches!(
            self,
            ComponentType::ActorHead
                | ComponentType::BoundaryHead
                | ComponentType::ControlHead
                | ComponentType::EntityHead
                | ComponentType::QueueHead
                | ComponentType::DatabaseHead
                | ComponentType::CollectionsHead
                | ComponentType::ParticipantHead
        )
    }

    /// Whether this is a "tail" participant type.
    pub fn is_tail(self) -> bool {
        matches!(
            self,
            ComponentType::ActorTail
                | ComponentType::BoundaryTail
                | ComponentType::ControlTail
                | ComponentType::EntityTail
                | ComponentType::QueueTail
                | ComponentType::DatabaseTail
                | ComponentType::CollectionsTail
                | ComponentType::ParticipantTail
        )
    }

    /// Whether this is an activation box type.
    pub fn is_activation_box(self) -> bool {
        matches!(
            self,
            ComponentType::ActivationBoxCloseClose
                | ComponentType::ActivationBoxCloseOpen
                | ComponentType::ActivationBoxOpenClose
                | ComponentType::ActivationBoxOpenOpen
        )
    }

    /// Whether this is a grouping component.
    pub fn is_grouping(self) -> bool {
        matches!(
            self,
            ComponentType::GroupingElseLegacy
                | ComponentType::GroupingElseTeoz
                | ComponentType::GroupingHeaderLegacy
                | ComponentType::GroupingHeaderTeoz
                | ComponentType::GroupingSpace
        )
    }

    /// Whether this is a delay-related component.
    pub fn is_delay(self) -> bool {
        matches!(self, ComponentType::DelayText | ComponentType::DelayLine)
    }
}

/// Participant visual style. Java: `skin.ComponentStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentStyle {
    Uml1,
    #[default]
    Uml2,
    Rectangle,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ComponentType basic existence ----

    #[test]
    fn component_type_arrow() {
        assert!(ComponentType::Arrow.is_arrow());
        assert!(!ComponentType::Note.is_arrow());
    }

    #[test]
    fn component_type_heads() {
        assert!(ComponentType::ActorHead.is_head());
        assert!(ComponentType::BoundaryHead.is_head());
        assert!(ComponentType::ControlHead.is_head());
        assert!(ComponentType::EntityHead.is_head());
        assert!(ComponentType::QueueHead.is_head());
        assert!(ComponentType::DatabaseHead.is_head());
        assert!(ComponentType::CollectionsHead.is_head());
        assert!(ComponentType::ParticipantHead.is_head());
        assert!(!ComponentType::Arrow.is_head());
    }

    #[test]
    fn component_type_tails() {
        assert!(ComponentType::ActorTail.is_tail());
        assert!(ComponentType::BoundaryTail.is_tail());
        assert!(ComponentType::ControlTail.is_tail());
        assert!(ComponentType::EntityTail.is_tail());
        assert!(ComponentType::QueueTail.is_tail());
        assert!(ComponentType::DatabaseTail.is_tail());
        assert!(ComponentType::CollectionsTail.is_tail());
        assert!(ComponentType::ParticipantTail.is_tail());
        assert!(!ComponentType::Arrow.is_tail());
    }

    #[test]
    fn component_type_activation_boxes() {
        assert!(ComponentType::ActivationBoxCloseClose.is_activation_box());
        assert!(ComponentType::ActivationBoxCloseOpen.is_activation_box());
        assert!(ComponentType::ActivationBoxOpenClose.is_activation_box());
        assert!(ComponentType::ActivationBoxOpenOpen.is_activation_box());
        assert!(!ComponentType::Arrow.is_activation_box());
    }

    #[test]
    fn component_type_grouping() {
        assert!(ComponentType::GroupingElseLegacy.is_grouping());
        assert!(ComponentType::GroupingElseTeoz.is_grouping());
        assert!(ComponentType::GroupingHeaderLegacy.is_grouping());
        assert!(ComponentType::GroupingHeaderTeoz.is_grouping());
        assert!(ComponentType::GroupingSpace.is_grouping());
        assert!(!ComponentType::Note.is_grouping());
    }

    #[test]
    fn component_type_delay() {
        assert!(ComponentType::DelayText.is_delay());
        assert!(ComponentType::DelayLine.is_delay());
        assert!(!ComponentType::Destroy.is_delay());
    }

    #[test]
    fn component_type_misc() {
        let _ = ComponentType::Newpage;
        let _ = ComponentType::Note;
        let _ = ComponentType::NoteHexagonal;
        let _ = ComponentType::NoteBox;
        let _ = ComponentType::Divider;
        let _ = ComponentType::Reference;
        let _ = ComponentType::Englober;
        let _ = ComponentType::Destroy;
        let _ = ComponentType::ParticipantLine;
    }

    #[test]
    fn component_type_eq_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ComponentType::Arrow);
        set.insert(ComponentType::Note);
        set.insert(ComponentType::Arrow); // duplicate
        assert_eq!(set.len(), 2);
    }

    // ---- ComponentStyle ----

    #[test]
    fn component_style_default_is_uml2() {
        assert_eq!(ComponentStyle::default(), ComponentStyle::Uml2);
    }

    #[test]
    fn component_style_variants() {
        let _ = ComponentStyle::Uml1;
        let _ = ComponentStyle::Uml2;
        let _ = ComponentStyle::Rectangle;
    }

    #[test]
    fn component_style_equality() {
        assert_ne!(ComponentStyle::Uml1, ComponentStyle::Uml2);
        assert_ne!(ComponentStyle::Uml2, ComponentStyle::Rectangle);
    }
}
