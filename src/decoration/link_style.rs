// decoration::link_style - Link line style
// Port of Java PlantUML's decoration.LinkStyle

use crate::klimt::UStroke;

/// Line style for links/edges.
/// Java: `decoration.LinkStyle`
#[derive(Debug, Clone, PartialEq)]
pub struct LinkStyle {
    kind: LinkStyleKind,
    thickness: Option<f64>,
}

/// Internal kind discriminant, matching Java's `LinkStyle.Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkStyleKind {
    Normal,
    Dashed,
    Dotted,
    Bold,
    Invisible,
}

impl LinkStyle {
    // ── Factory constructors (matching Java static methods) ──

    pub fn normal() -> Self {
        Self { kind: LinkStyleKind::Normal, thickness: None }
    }

    pub fn dashed() -> Self {
        Self { kind: LinkStyleKind::Dashed, thickness: None }
    }

    pub fn dotted() -> Self {
        Self { kind: LinkStyleKind::Dotted, thickness: None }
    }

    pub fn bold() -> Self {
        Self { kind: LinkStyleKind::Bold, thickness: None }
    }

    pub fn invisible() -> Self {
        Self { kind: LinkStyleKind::Invisible, thickness: None }
    }

    // ── Queries ──

    pub fn is_normal(&self) -> bool {
        self.kind == LinkStyleKind::Normal
    }

    pub fn is_invisible(&self) -> bool {
        self.kind == LinkStyleKind::Invisible
    }

    pub fn is_thickness_overridden(&self) -> bool {
        self.thickness.is_some()
    }

    pub fn kind(&self) -> LinkStyleKind {
        self.kind
    }

    // ── Derivation ──

    /// Return a new `LinkStyle` with the same kind but an explicit thickness.
    /// Java: `goThickness(double)`
    pub fn go_thickness(&self, thickness: f64) -> Self {
        Self { kind: self.kind, thickness: Some(thickness) }
    }

    // ── Stroke conversion ──

    /// Non-zero thickness: returns the explicit value, or 1.0 if unset.
    /// Java: `nonZeroThickness()`
    fn non_zero_thickness(&self) -> f64 {
        self.thickness.unwrap_or(1.0)
    }

    /// Convert to `UStroke` (dash pattern + thickness).
    /// Java: `getStroke3()`
    pub fn get_stroke3(&self) -> UStroke {
        match self.kind {
            LinkStyleKind::Dashed => UStroke::new(7.0, 7.0, self.non_zero_thickness()),
            LinkStyleKind::Dotted => UStroke::new(1.0, 3.0, self.non_zero_thickness()),
            LinkStyleKind::Bold => UStroke::with_thickness(2.0),
            _ => UStroke::with_thickness(self.non_zero_thickness()),
        }
    }

    /// Mutate a stroke according to this style. If the style is dashed/dotted/bold
    /// the stroke's dash pattern is overridden; otherwise the passed stroke is
    /// returned unchanged.
    /// Java: `muteStroke(UStroke)`
    pub fn mute_stroke(&self, stroke: UStroke) -> UStroke {
        match self.kind {
            LinkStyleKind::Dashed | LinkStyleKind::Dotted | LinkStyleKind::Bold => {
                self.get_stroke3()
            }
            _ => stroke,
        }
    }

    // ── Parsing ──

    /// Parse from a string; returns `Normal` if unrecognised.
    /// Java: `fromString1(String)`
    pub fn from_string1(s: &str) -> Self {
        Self::from_string2(s).unwrap_or_else(Self::normal)
    }

    /// Parse from a string; returns `None` if unrecognised.
    /// Java: `fromString2(String)`
    pub fn from_string2(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "dashed" => Some(Self::dashed()),
            "dotted" => Some(Self::dotted()),
            "bold" => Some(Self::bold()),
            "hidden" => Some(Self::invisible()),
            _ => None,
        }
    }
}

impl Default for LinkStyle {
    fn default() -> Self {
        Self::normal()
    }
}

impl std::fmt::Display for LinkStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({:?})", self.kind, self.thickness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_stroke() {
        let s = LinkStyle::normal().get_stroke3();
        assert!(s.dasharray_svg().is_none());
        assert_eq!(s.thickness, 1.0);
    }

    #[test]
    fn dashed_stroke() {
        let s = LinkStyle::dashed().get_stroke3();
        assert_eq!(s.dasharray_svg(), Some((7.0, 7.0)));
        assert_eq!(s.thickness, 1.0);
    }

    #[test]
    fn dotted_stroke() {
        let s = LinkStyle::dotted().get_stroke3();
        assert_eq!(s.dasharray_svg(), Some((1.0, 3.0)));
        assert_eq!(s.thickness, 1.0);
    }

    #[test]
    fn bold_stroke() {
        let s = LinkStyle::bold().get_stroke3();
        assert!(s.dasharray_svg().is_none());
        assert_eq!(s.thickness, 2.0);
    }

    #[test]
    fn thickness_override() {
        let s = LinkStyle::normal().go_thickness(3.5);
        assert!(s.is_thickness_overridden());
        let stroke = s.get_stroke3();
        assert_eq!(stroke.thickness, 3.5);
    }

    #[test]
    fn dashed_with_thickness() {
        let s = LinkStyle::dashed().go_thickness(2.0);
        let stroke = s.get_stroke3();
        assert_eq!(stroke.dasharray_svg(), Some((7.0, 7.0)));
        assert_eq!(stroke.thickness, 2.0);
    }

    #[test]
    fn from_string_round_trip() {
        assert!(LinkStyle::from_string1("dashed").kind == LinkStyleKind::Dashed);
        assert!(LinkStyle::from_string1("DOTTED").kind == LinkStyleKind::Dotted);
        assert!(LinkStyle::from_string1("Bold").kind == LinkStyleKind::Bold);
        assert!(LinkStyle::from_string1("hidden").is_invisible());
        assert!(LinkStyle::from_string1("unknown").is_normal());
    }

    #[test]
    fn from_string2_none_on_unknown() {
        assert!(LinkStyle::from_string2("plain").is_none());
        assert!(LinkStyle::from_string2("").is_none());
    }

    #[test]
    fn mute_stroke_passthrough() {
        let original = UStroke::with_thickness(5.0);
        let result = LinkStyle::normal().mute_stroke(original.clone());
        assert_eq!(result, original);
    }

    #[test]
    fn mute_stroke_overrides() {
        let original = UStroke::with_thickness(5.0);
        let result = LinkStyle::dashed().mute_stroke(original);
        assert_eq!(result.dasharray_svg(), Some((7.0, 7.0)));
    }

    #[test]
    fn default_is_normal() {
        assert!(LinkStyle::default().is_normal());
    }

    #[test]
    fn display_format() {
        let s = LinkStyle::dashed().go_thickness(2.0);
        let display = format!("{}", s);
        assert!(display.contains("Dashed"));
        assert!(display.contains("2.0"));
    }
}
