// Port of net.sourceforge.plantuml.dot.DotSplines
//
// Represents the Graphviz `splines` attribute value that controls
// edge routing mode. Java original defines: POLYLINE, ORTHO, SPLINES.

use std::fmt;

/// Edge routing mode for Graphviz layout.
///
/// Maps to the `splines` graph attribute in DOT language:
/// - `Splines` -> `splines=true` (default curved splines)
/// - `Polyline` -> `splines=polyline` (straight line segments)
/// - `Ortho` -> `splines=ortho` (axis-aligned segments)
/// - `Curved` -> `splines=curved` (curved, non-Bezier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DotSplines {
    #[default]
    Splines,
    Polyline,
    Ortho,
    Curved,
}

impl DotSplines {
    /// Return the DOT attribute value string.
    pub fn as_dot_value(&self) -> &'static str {
        match self {
            DotSplines::Splines => "true",
            DotSplines::Polyline => "polyline",
            DotSplines::Ortho => "ortho",
            DotSplines::Curved => "curved",
        }
    }

    /// Parse from a string (case-insensitive).
    /// Returns `None` for unrecognized values.
    pub fn from_str_opt(s: &str) -> Option<DotSplines> {
        match s.to_lowercase().as_str() {
            "splines" | "true" => Some(DotSplines::Splines),
            "polyline" => Some(DotSplines::Polyline),
            "ortho" | "ortho " => Some(DotSplines::Ortho),
            "curved" => Some(DotSplines::Curved),
            _ => None,
        }
    }
}

impl fmt::Display for DotSplines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_dot_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_splines() {
        assert_eq!(DotSplines::default(), DotSplines::Splines);
    }

    #[test]
    fn dot_value_roundtrip() {
        for variant in &[
            DotSplines::Splines,
            DotSplines::Polyline,
            DotSplines::Ortho,
            DotSplines::Curved,
        ] {
            let s = variant.as_dot_value();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(DotSplines::from_str_opt("ORTHO"), Some(DotSplines::Ortho));
        assert_eq!(
            DotSplines::from_str_opt("polyline"),
            Some(DotSplines::Polyline)
        );
        assert_eq!(
            DotSplines::from_str_opt("Splines"),
            Some(DotSplines::Splines)
        );
        assert_eq!(DotSplines::from_str_opt("true"), Some(DotSplines::Splines));
        assert_eq!(
            DotSplines::from_str_opt("Curved"),
            Some(DotSplines::Curved)
        );
        assert_eq!(DotSplines::from_str_opt("unknown"), None);
    }

    #[test]
    fn display_format() {
        assert_eq!(format!("{}", DotSplines::Ortho), "ortho");
        assert_eq!(format!("{}", DotSplines::Splines), "true");
    }
}
