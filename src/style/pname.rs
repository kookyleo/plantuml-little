// CSS property names used by the style system.
// Port of Java PlantUML's `net.sourceforge.plantuml.style.PName`

/// CSS property name — identifies which visual property a style rule sets.
///
/// Java: `style.PName`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PName {
    Shadowing,
    FontName,
    FontColor,
    FontSize,
    FontStyle,
    /// CSS font-weight: keywords (normal, bold, lighter, bolder) or numeric 100-900.
    FontWeight,
    BackGroundColor,
    RoundCorner,
    LineThickness,
    DiagonalCorner,
    HyperLinkColor,
    HyperlinkUnderlineStyle,
    HyperlinkUnderlineThickness,
    HeadColor,
    LineColor,
    LineStyle,
    Padding,
    Margin,
    MaximumWidth,
    MinimumWidth,
    ExportedName,
    Image,
    HorizontalAlignment,
    ShowStereotype,
    ImagePosition,
    // Chart-specific properties
    MarkerShape,
    MarkerSize,
    MarkerColor,
    BarWidth,
}

impl PName {
    /// Case-insensitive lookup by name string.
    /// Java: `PName.getFromName(String, StyleScheme)`
    pub fn from_name(name: &str) -> Option<PName> {
        ALL_PNAMES.iter().copied().find(|p| {
            let variant = p.as_str();
            variant.eq_ignore_ascii_case(name)
        })
    }

    /// Returns the variant name as a string (matches the Java enum name exactly).
    pub fn as_str(self) -> &'static str {
        match self {
            PName::Shadowing => "Shadowing",
            PName::FontName => "FontName",
            PName::FontColor => "FontColor",
            PName::FontSize => "FontSize",
            PName::FontStyle => "FontStyle",
            PName::FontWeight => "FontWeight",
            PName::BackGroundColor => "BackGroundColor",
            PName::RoundCorner => "RoundCorner",
            PName::LineThickness => "LineThickness",
            PName::DiagonalCorner => "DiagonalCorner",
            PName::HyperLinkColor => "HyperLinkColor",
            PName::HyperlinkUnderlineStyle => "HyperlinkUnderlineStyle",
            PName::HyperlinkUnderlineThickness => "HyperlinkUnderlineThickness",
            PName::HeadColor => "HeadColor",
            PName::LineColor => "LineColor",
            PName::LineStyle => "LineStyle",
            PName::Padding => "Padding",
            PName::Margin => "Margin",
            PName::MaximumWidth => "MaximumWidth",
            PName::MinimumWidth => "MinimumWidth",
            PName::ExportedName => "ExportedName",
            PName::Image => "Image",
            PName::HorizontalAlignment => "HorizontalAlignment",
            PName::ShowStereotype => "ShowStereotype",
            PName::ImagePosition => "ImagePosition",
            PName::MarkerShape => "MarkerShape",
            PName::MarkerSize => "MarkerSize",
            PName::MarkerColor => "MarkerColor",
            PName::BarWidth => "BarWidth",
        }
    }
}

/// All PName variants for iteration.
const ALL_PNAMES: &[PName] = &[
    PName::Shadowing,
    PName::FontName,
    PName::FontColor,
    PName::FontSize,
    PName::FontStyle,
    PName::FontWeight,
    PName::BackGroundColor,
    PName::RoundCorner,
    PName::LineThickness,
    PName::DiagonalCorner,
    PName::HyperLinkColor,
    PName::HyperlinkUnderlineStyle,
    PName::HyperlinkUnderlineThickness,
    PName::HeadColor,
    PName::LineColor,
    PName::LineStyle,
    PName::Padding,
    PName::Margin,
    PName::MaximumWidth,
    PName::MinimumWidth,
    PName::ExportedName,
    PName::Image,
    PName::HorizontalAlignment,
    PName::ShowStereotype,
    PName::ImagePosition,
    PName::MarkerShape,
    PName::MarkerSize,
    PName::MarkerColor,
    PName::BarWidth,
];

impl std::fmt::Display for PName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_count_matches_java() {
        // Java PName has 29 variants
        assert_eq!(ALL_PNAMES.len(), 29);
    }

    #[test]
    fn from_name_exact_case() {
        assert_eq!(PName::from_name("FontSize"), Some(PName::FontSize));
        assert_eq!(PName::from_name("BackGroundColor"), Some(PName::BackGroundColor));
        assert_eq!(PName::from_name("BarWidth"), Some(PName::BarWidth));
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(PName::from_name("fontsize"), Some(PName::FontSize));
        assert_eq!(PName::from_name("FONTSIZE"), Some(PName::FontSize));
        assert_eq!(PName::from_name("backgroundcolor"), Some(PName::BackGroundColor));
    }

    #[test]
    fn from_name_unknown_returns_none() {
        assert_eq!(PName::from_name("NoSuchProperty"), None);
        assert_eq!(PName::from_name(""), None);
    }

    #[test]
    fn as_str_roundtrip() {
        for p in ALL_PNAMES {
            assert_eq!(PName::from_name(p.as_str()), Some(*p));
        }
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", PName::FontName), "FontName");
        assert_eq!(format!("{}", PName::HyperLinkColor), "HyperLinkColor");
    }

    #[test]
    fn all_variants_unique() {
        let mut seen = std::collections::HashSet::new();
        for p in ALL_PNAMES {
            assert!(seen.insert(p), "duplicate variant: {:?}", p);
        }
    }
}
