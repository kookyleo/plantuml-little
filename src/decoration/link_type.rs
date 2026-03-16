// decoration::link_type - Complete link type (both endpoints + line style)
// Port of Java PlantUML's decoration.LinkType
// Stub - to be filled by agent

use super::link_decor::{LinkDecor, LinkMiddleDecor};
use super::link_style::LinkStyle;

/// Complete specification of a link's visual type.
/// Java: `decoration.LinkType`
#[derive(Debug, Clone)]
pub struct LinkType {
    pub decor1: LinkDecor,
    pub decor2: LinkDecor,
    pub middle_decor: LinkMiddleDecor,
    pub style: LinkStyle,
}

impl LinkType {
    pub fn new(decor1: LinkDecor, decor2: LinkDecor) -> Self {
        Self { decor1, decor2, middle_decor: LinkMiddleDecor::None, style: LinkStyle::normal() }
    }

    pub fn with_style(mut self, style: LinkStyle) -> Self {
        self.style = style; self
    }

    pub fn with_middle(mut self, middle: LinkMiddleDecor) -> Self {
        self.middle_decor = middle; self
    }

    pub fn is_double_decorated(&self) -> bool {
        self.decor1 != LinkDecor::None && self.decor2 != LinkDecor::None
    }

    pub fn reversed(&self) -> Self {
        Self {
            decor1: self.decor2,
            decor2: self.decor1,
            middle_decor: self.middle_decor,
            style: self.style.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        assert!(!lt.is_double_decorated());
        let rev = lt.reversed();
        assert_eq!(rev.decor1, LinkDecor::None);
        assert_eq!(rev.decor2, LinkDecor::Arrow);
    }
}
