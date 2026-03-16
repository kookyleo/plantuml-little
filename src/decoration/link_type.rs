// decoration::link_type - Complete link type (both endpoints + line style)
// Port of Java PlantUML's decoration.LinkType

use crate::klimt::UStroke;
use super::link_decor::{LinkDecor, LinkMiddleDecor};
use super::link_style::LinkStyle;

/// Link layout strategy hint (for DOT/svek).
/// Java: `abel.LinkStrategy`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LinkStrategy {
    #[default]
    Normal,
    Simplier,
}

/// Complete specification of a link's visual type: two endpoint decorations,
/// a line style, and an optional middle decoration.
/// Java: `decoration.LinkType`
#[derive(Debug, Clone, PartialEq)]
pub struct LinkType {
    decor1: LinkDecor,
    decor2: LinkDecor,
    middle_decor: LinkMiddleDecor,
    style: LinkStyle,
}

impl LinkType {
    /// Create with two endpoint decorations, normal style, no middle decor.
    /// Java: `LinkType(LinkDecor, LinkDecor)`
    pub fn new(decor1: LinkDecor, decor2: LinkDecor) -> Self {
        Self {
            decor1,
            decor2,
            middle_decor: LinkMiddleDecor::None,
            style: LinkStyle::normal(),
        }
    }

    /// Full constructor.
    fn new_full(
        decor1: LinkDecor,
        decor2: LinkDecor,
        middle_decor: LinkMiddleDecor,
        style: LinkStyle,
    ) -> Self {
        Self { decor1, decor2, middle_decor, style }
    }

    // ── Accessors ──

    pub fn decor1(&self) -> LinkDecor {
        self.decor1
    }

    pub fn decor2(&self) -> LinkDecor {
        self.decor2
    }

    pub fn middle_decor(&self) -> LinkMiddleDecor {
        self.middle_decor
    }

    pub fn style(&self) -> &LinkStyle {
        &self.style
    }

    // ── Queries ──

    /// Both endpoints have decorations.
    /// Java: `isDoubleDecorated()`
    pub fn is_double_decorated(&self) -> bool {
        self.decor1 != LinkDecor::None && self.decor2 != LinkDecor::None
    }

    /// In SVG, a link that only has decor2 "looks reverted" because DOT draws
    /// arrowhead on decor1 side.
    /// Java: `looksLikeRevertedForSvg()`
    pub fn looks_like_reverted_for_svg(&self) -> bool {
        self.decor1 == LinkDecor::None && self.decor2 != LinkDecor::None
    }

    /// Both sides are empty or both have decorations => no SVG arrow visible.
    /// Java: `looksLikeNoDecorAtAllSvg()`
    pub fn looks_like_no_decor_at_all_svg(&self) -> bool {
        (self.decor1 == LinkDecor::None && self.decor2 == LinkDecor::None)
            || (self.decor1 != LinkDecor::None && self.decor2 != LinkDecor::None)
    }

    /// Line style is invisible.
    /// Java: `isInvisible()`
    pub fn is_invisible(&self) -> bool {
        self.style.is_invisible()
    }

    /// Either endpoint is `Extends`.
    /// Java: `isExtends()`
    pub fn is_extends(&self) -> bool {
        self.decor1 == LinkDecor::Extends || self.decor2 == LinkDecor::Extends
    }

    // ── Style derivation ──

    /// Return a copy with dashed line style.
    /// Java: `goDashed()`
    pub fn go_dashed(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, self.middle_decor, LinkStyle::dashed())
    }

    /// Return a copy with dotted line style.
    /// Java: `goDotted()`
    pub fn go_dotted(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, self.middle_decor, LinkStyle::dotted())
    }

    /// Return a copy with bold line style.
    /// Java: `goBold()`
    pub fn go_bold(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, self.middle_decor, LinkStyle::bold())
    }

    /// Return a copy with invisible line style.
    /// Java: `getInvisible()`
    pub fn go_invisible(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, self.middle_decor, LinkStyle::invisible())
    }

    /// Return a copy with an explicit thickness override.
    /// Java: `goThickness(double)`
    pub fn go_thickness(&self, thickness: f64) -> Self {
        Self::new_full(
            self.decor1,
            self.decor2,
            self.middle_decor,
            self.style.go_thickness(thickness),
        )
    }

    // ── Decoration derivation ──

    /// Swap endpoints and invert the middle decor.
    /// Java: `getInversed()`
    pub fn inversed(&self) -> Self {
        Self::new_full(
            self.decor2,
            self.decor1,
            self.middle_decor.inversed(),
            self.style.clone(),
        )
    }

    /// Remove decor1 (set to None).
    /// Java: `withoutDecors1()`
    pub fn without_decors1(&self) -> Self {
        Self::new_full(LinkDecor::None, self.decor2, self.middle_decor, self.style.clone())
    }

    /// Remove decor2 (set to None).
    /// Java: `withoutDecors2()`
    pub fn without_decors2(&self) -> Self {
        Self::new_full(self.decor1, LinkDecor::None, self.middle_decor, self.style.clone())
    }

    /// Keep only decor1 (decor2 -> None).
    /// Java: `getPart1()`
    pub fn part1(&self) -> Self {
        Self::new_full(self.decor1, LinkDecor::None, self.middle_decor, self.style.clone())
    }

    /// Keep only decor2 (decor1 -> None).
    /// Java: `getPart2()`
    pub fn part2(&self) -> Self {
        Self::new_full(LinkDecor::None, self.decor2, self.middle_decor, self.style.clone())
    }

    // ── Middle decor derivation ──

    pub fn with_middle_circle(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, LinkMiddleDecor::Circle, self.style.clone())
    }

    pub fn with_middle_circle_circled(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, LinkMiddleDecor::CircleCircled, self.style.clone())
    }

    pub fn with_middle_circle_circled1(&self) -> Self {
        Self::new_full(
            self.decor1,
            self.decor2,
            LinkMiddleDecor::CircleCircled1,
            self.style.clone(),
        )
    }

    pub fn with_middle_circle_circled2(&self) -> Self {
        Self::new_full(
            self.decor1,
            self.decor2,
            LinkMiddleDecor::CircleCircled2,
            self.style.clone(),
        )
    }

    pub fn with_middle_subset(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, LinkMiddleDecor::Subset, self.style.clone())
    }

    pub fn with_middle_superset(&self) -> Self {
        Self::new_full(self.decor1, self.decor2, LinkMiddleDecor::Superset, self.style.clone())
    }

    // ── Lollipop interface helpers ──

    /// Java: `withLollipopInterfaceEye1()`
    pub fn with_lollipop_interface_eye1(&self) -> Self {
        Self::new_full(self.decor1, LinkDecor::None, self.middle_decor, self.style.clone())
    }

    /// Java: `withLollipopInterfaceEye2()`
    pub fn with_lollipop_interface_eye2(&self) -> Self {
        Self::new_full(LinkDecor::None, self.decor2, self.middle_decor, self.style.clone())
    }

    // ── Stroke resolution ──

    /// Resolve the effective stroke, respecting thickness overrides.
    /// Java: `getStroke3(UStroke defaultThickness)`
    pub fn get_stroke3(&self, default_thickness: Option<&UStroke>) -> UStroke {
        if self.style.is_thickness_overridden() {
            return self.style.get_stroke3();
        }

        let Some(dt) = default_thickness else {
            return self.style.get_stroke3();
        };

        if dt.dash_visible == 0.0 && dt.dash_space == 0.0 {
            return self.style.go_thickness(dt.thickness).get_stroke3();
        }

        dt.clone()
    }

    // ── DOT arrow attributes ──

    /// Build the `arrowtail`/`arrowhead`/`dir`/`arrowsize` string for DOT/svek.
    /// Java: `getSpecificDecorationSvek(LinkStrategy)`
    pub fn specific_decoration_svek(&self, strategy: LinkStrategy) -> String {
        if strategy == LinkStrategy::Simplier {
            return "arrowtail=none,arrowhead=none".to_string();
        }

        let mut sb = String::new();

        let empty1 = self.decor1 == LinkDecor::None;
        let empty2 = self.decor2 == LinkDecor::None;

        if empty1 && empty2 {
            sb.push_str("arrowtail=none,arrowhead=none");
        } else if !empty1 && !empty2 {
            sb.push_str("dir=both,arrowtail=empty,arrowhead=empty");
        } else if empty1 && !empty2 {
            sb.push_str("arrowtail=empty,arrowhead=none,dir=back");
        }

        let arrow_size = self.decor1.arrow_size().max(self.decor2.arrow_size());
        if arrow_size > 0.0 {
            if !sb.is_empty() {
                sb.push(',');
            }
            sb.push_str(&format!("arrowsize={}", arrow_size));
        }

        sb
    }

    // ── Semantic link type name ──

    /// Returns the semantic link type name for SVG data-link-type attribute.
    /// Java: `getLinkTypeName()`
    pub fn link_type_name(&self) -> Option<&'static str> {
        if self.has(LinkDecor::Composition) {
            return Some("composition");
        }
        if self.has(LinkDecor::Agregation) {
            return Some("aggregation");
        }
        if self.has(LinkDecor::Extends) {
            return Some("extension");
        }
        if self.has(LinkDecor::Redefines) {
            return Some("redefines");
        }
        if self.has(LinkDecor::DefinedBy) {
            return Some("definedby");
        }
        if self.has_any(&[LinkDecor::Arrow, LinkDecor::ArrowTriangle]) {
            return Some("dependency");
        }
        if self.has(LinkDecor::NotNavigable) {
            return Some("not_navigable");
        }
        if self.has_any(&[LinkDecor::Crowfoot, LinkDecor::CircleCrowfoot, LinkDecor::LineCrowfoot])
        {
            return Some("crowfoot");
        }
        if self.has_any(&[LinkDecor::CircleLine, LinkDecor::DoubleLine]) || self.both_none() {
            return Some("association");
        }
        if self.has(LinkDecor::Plus) {
            return Some("nested");
        }
        Option::None
    }

    // ── Private helpers ──

    fn has(&self, decor: LinkDecor) -> bool {
        self.decor1 == decor || self.decor2 == decor
    }

    fn has_any(&self, decors: &[LinkDecor]) -> bool {
        decors.iter().any(|d| self.has(*d))
    }

    fn both_none(&self) -> bool {
        self.decor1 == LinkDecor::None && self.decor2 == LinkDecor::None
    }
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{}-{:?}", self.decor1, self.style, self.decor2)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::link_style::LinkStyleKind;

    #[test]
    fn basic_new() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        assert_eq!(lt.decor1(), LinkDecor::Arrow);
        assert_eq!(lt.decor2(), LinkDecor::None);
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::None);
        assert!(lt.style().is_normal());
    }

    #[test]
    fn is_double_decorated() {
        assert!(LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).is_double_decorated());
        assert!(!LinkType::new(LinkDecor::Arrow, LinkDecor::None).is_double_decorated());
        assert!(!LinkType::new(LinkDecor::None, LinkDecor::None).is_double_decorated());
    }

    #[test]
    fn inversed() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let inv = lt.inversed();
        assert_eq!(inv.decor1(), LinkDecor::Extends);
        assert_eq!(inv.decor2(), LinkDecor::Arrow);
    }

    #[test]
    fn inversed_preserves_style() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_dashed();
        let inv = lt.inversed();
        assert_eq!(inv.style().kind(), LinkStyleKind::Dashed);
    }

    #[test]
    fn inversed_middle_decor() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None)
            .with_middle_circle_circled1();
        let inv = lt.inversed();
        assert_eq!(inv.middle_decor(), LinkMiddleDecor::CircleCircled2);
    }

    #[test]
    fn looks_like_reverted_for_svg() {
        assert!(LinkType::new(LinkDecor::None, LinkDecor::Arrow).looks_like_reverted_for_svg());
        assert!(
            !LinkType::new(LinkDecor::Arrow, LinkDecor::None).looks_like_reverted_for_svg()
        );
        assert!(
            !LinkType::new(LinkDecor::Arrow, LinkDecor::Arrow).looks_like_reverted_for_svg()
        );
    }

    #[test]
    fn looks_like_no_decor_at_all_svg() {
        // Both none
        assert!(
            LinkType::new(LinkDecor::None, LinkDecor::None).looks_like_no_decor_at_all_svg()
        );
        // Both present
        assert!(
            LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).looks_like_no_decor_at_all_svg()
        );
        // One side only
        assert!(
            !LinkType::new(LinkDecor::Arrow, LinkDecor::None).looks_like_no_decor_at_all_svg()
        );
    }

    #[test]
    fn style_derivation() {
        let base = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        assert_eq!(base.go_dashed().style().kind(), LinkStyleKind::Dashed);
        assert_eq!(base.go_dotted().style().kind(), LinkStyleKind::Dotted);
        assert_eq!(base.go_bold().style().kind(), LinkStyleKind::Bold);
        assert!(base.go_invisible().is_invisible());
    }

    #[test]
    fn go_thickness() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_thickness(3.0);
        assert!(lt.style().is_thickness_overridden());
        assert_eq!(lt.style().get_stroke3().thickness, 3.0);
    }

    #[test]
    fn without_decors() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        assert_eq!(lt.without_decors1().decor1(), LinkDecor::None);
        assert_eq!(lt.without_decors1().decor2(), LinkDecor::Extends);
        assert_eq!(lt.without_decors2().decor1(), LinkDecor::Arrow);
        assert_eq!(lt.without_decors2().decor2(), LinkDecor::None);
    }

    #[test]
    fn parts() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let p1 = lt.part1();
        assert_eq!(p1.decor1(), LinkDecor::Arrow);
        assert_eq!(p1.decor2(), LinkDecor::None);
        let p2 = lt.part2();
        assert_eq!(p2.decor1(), LinkDecor::None);
        assert_eq!(p2.decor2(), LinkDecor::Extends);
    }

    #[test]
    fn is_extends() {
        assert!(LinkType::new(LinkDecor::Extends, LinkDecor::None).is_extends());
        assert!(LinkType::new(LinkDecor::None, LinkDecor::Extends).is_extends());
        assert!(!LinkType::new(LinkDecor::Arrow, LinkDecor::None).is_extends());
    }

    // ── Middle decor derivation ──

    #[test]
    fn middle_circle() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).with_middle_circle();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::Circle);
    }

    #[test]
    fn middle_subset_superset() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        assert_eq!(lt.with_middle_subset().middle_decor(), LinkMiddleDecor::Subset);
        assert_eq!(lt.with_middle_superset().middle_decor(), LinkMiddleDecor::Superset);
    }

    // ── Stroke resolution ──

    #[test]
    fn get_stroke3_no_default() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        let s = lt.get_stroke3(Option::None);
        assert_eq!(s.thickness, 1.0);
        assert!(s.dasharray_svg().is_none());
    }

    #[test]
    fn get_stroke3_with_thickness_default() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        let default_t = UStroke::with_thickness(2.5);
        let s = lt.get_stroke3(Some(&default_t));
        assert_eq!(s.thickness, 2.5);
    }

    #[test]
    fn get_stroke3_dash_default_passthrough() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        let default_t = UStroke::new(5.0, 5.0, 2.0);
        let s = lt.get_stroke3(Some(&default_t));
        // When default has dash pattern, it passes through
        assert_eq!(s.dasharray_svg(), Some((5.0, 5.0)));
        assert_eq!(s.thickness, 2.0);
    }

    #[test]
    fn get_stroke3_overridden_ignores_default() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_thickness(4.0);
        let default_t = UStroke::with_thickness(2.0);
        let s = lt.get_stroke3(Some(&default_t));
        assert_eq!(s.thickness, 4.0);
    }

    // ── DOT attributes ──

    #[test]
    fn specific_decoration_svek_simplier() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let s = lt.specific_decoration_svek(LinkStrategy::Simplier);
        assert_eq!(s, "arrowtail=none,arrowhead=none");
    }

    #[test]
    fn specific_decoration_svek_both_none() {
        let lt = LinkType::new(LinkDecor::None, LinkDecor::None);
        let s = lt.specific_decoration_svek(LinkStrategy::Normal);
        assert!(s.contains("arrowtail=none"));
        assert!(s.contains("arrowhead=none"));
    }

    #[test]
    fn specific_decoration_svek_both_present() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let s = lt.specific_decoration_svek(LinkStrategy::Normal);
        assert!(s.contains("dir=both"));
        assert!(s.contains("arrowtail=empty"));
        assert!(s.contains("arrowhead=empty"));
        assert!(s.contains("arrowsize="));
    }

    #[test]
    fn specific_decoration_svek_only_decor2() {
        let lt = LinkType::new(LinkDecor::None, LinkDecor::Arrow);
        let s = lt.specific_decoration_svek(LinkStrategy::Normal);
        assert!(s.contains("arrowtail=empty"));
        assert!(s.contains("arrowhead=none"));
        assert!(s.contains("dir=back"));
    }

    // ── Semantic type name ──

    #[test]
    fn link_type_name_composition() {
        let lt = LinkType::new(LinkDecor::Composition, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("composition"));
    }

    #[test]
    fn link_type_name_aggregation() {
        let lt = LinkType::new(LinkDecor::None, LinkDecor::Agregation);
        assert_eq!(lt.link_type_name(), Some("aggregation"));
    }

    #[test]
    fn link_type_name_extension() {
        let lt = LinkType::new(LinkDecor::Extends, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("extension"));
    }

    #[test]
    fn link_type_name_dependency() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("dependency"));
    }

    #[test]
    fn link_type_name_association_both_none() {
        let lt = LinkType::new(LinkDecor::None, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("association"));
    }

    #[test]
    fn link_type_name_crowfoot() {
        let lt = LinkType::new(LinkDecor::Crowfoot, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("crowfoot"));
    }

    #[test]
    fn link_type_name_nested() {
        let lt = LinkType::new(LinkDecor::Plus, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("nested"));
    }

    #[test]
    fn link_type_name_not_navigable() {
        let lt = LinkType::new(LinkDecor::NotNavigable, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("not_navigable"));
    }

    #[test]
    fn link_type_name_redefines() {
        let lt = LinkType::new(LinkDecor::Redefines, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("redefines"));
    }

    #[test]
    fn link_type_name_definedby() {
        let lt = LinkType::new(LinkDecor::DefinedBy, LinkDecor::None);
        assert_eq!(lt.link_type_name(), Some("definedby"));
    }

    // ── Display ──

    #[test]
    fn display_format() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let s = format!("{}", lt);
        assert!(s.contains("Arrow"));
        assert!(s.contains("Extends"));
    }

    // ── Lollipop helpers ──

    #[test]
    fn lollipop_eye1() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let eye = lt.with_lollipop_interface_eye1();
        assert_eq!(eye.decor1(), LinkDecor::Arrow);
        assert_eq!(eye.decor2(), LinkDecor::None);
    }

    #[test]
    fn lollipop_eye2() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let eye = lt.with_lollipop_interface_eye2();
        assert_eq!(eye.decor1(), LinkDecor::None);
        assert_eq!(eye.decor2(), LinkDecor::Extends);
    }
}
