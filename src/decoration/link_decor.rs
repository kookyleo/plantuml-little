// decoration::link_decor - Arrow endpoint decoration types
// Port of Java PlantUML's decoration.LinkDecor + LinkMiddleDecor
// Stub - to be filled by agent

/// Arrow endpoint decoration style.
/// Java: `decoration.LinkDecor`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LinkDecor {
    #[default]
    None,
    Arrow,
    ArrowTriangle,
    ArrowAndCircle,
    Extends,
    Composition,
    Agregation,
    Circle,
    CircleConnect,
    CircleCross,
    CircleFill,
    CircleLine,
    Crowfoot,
    LineCrowfoot,
    CircleCrowfoot,
    Plus,
    HalfArrow,
    Square,
    DoubleLine,
    NotNavigable,
    Parenthesis,
}

/// Middle decoration on a link.
/// Java: `decoration.LinkMiddleDecor`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkMiddleDecor {
    #[default]
    None,
    Circle,
    CircleCircled,
    CircleCircledThin,
    Subset,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_none() {
        assert_eq!(LinkDecor::default(), LinkDecor::None);
        assert_eq!(LinkMiddleDecor::default(), LinkMiddleDecor::None);
    }
}
