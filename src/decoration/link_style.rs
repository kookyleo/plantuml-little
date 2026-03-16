// decoration::link_style - Link line style
// Port of Java PlantUML's decoration.LinkStyle
// Stub - to be filled by agent

use crate::klimt::UStroke;

/// Line style for links/edges.
/// Java: `decoration.LinkStyle`
#[derive(Debug, Clone, PartialEq)]
pub enum LinkStyle {
    Normal,
    Dashed,
    Dotted,
    Bold,
    Hidden,
}

impl LinkStyle {
    /// Convert to UStroke.
    pub fn to_stroke(&self, thickness: f64) -> UStroke {
        match self {
            Self::Normal => UStroke::with_thickness(thickness),
            Self::Dashed => UStroke::new(7.0, 7.0, thickness),
            Self::Dotted => UStroke::new(1.0, 3.0, thickness),
            Self::Bold => UStroke::with_thickness(2.0),
            Self::Hidden => UStroke::with_thickness(0.0),
        }
    }
}

impl Default for LinkStyle {
    fn default() -> Self { Self::Normal }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_stroke() {
        let s = LinkStyle::Normal.to_stroke(1.0);
        assert!(s.dasharray_svg().is_none());
        assert_eq!(s.thickness, 1.0);
    }
    #[test]
    fn dashed_stroke() {
        let s = LinkStyle::Dashed.to_stroke(1.0);
        assert!(s.dasharray_svg().is_some());
    }
}
