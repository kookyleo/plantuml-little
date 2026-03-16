// skin::component - Component type definitions
// Port of Java PlantUML's skin.ComponentType + ComponentStyle
// Stub - to be filled by agent

/// Sequence diagram component types. Java: `skin.ComponentType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Arrow,
    ReturnArrow,
    SelfArrow,
    Participant,
    ParticipantTail,
    Line,
    ActiveLine,
    Note,
    NoteBox,
    NoteHexagonal,
    Divider,
    Reference,
    Delay,
    DelayText,
    Destroy,
    GroupingHeader,
    GroupingElse,
    GroupingSpace,
    Newpage,
    Englober,
    Actor,
}

/// Participant visual style. Java: `skin.ComponentStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentStyle {
    #[default]
    Uml2,
    Uml1,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_types_exist() {
        let _ = ComponentType::Arrow;
        let _ = ComponentType::Participant;
        let _ = ComponentType::Note;
    }
}
