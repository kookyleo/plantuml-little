// klimt::font - Font metrics and string measurement
// Port of Java PlantUML's klimt.font package
//
// Wraps the existing font_metrics.rs (47k lines of Java AWT-compatible
// per-character advance width tables).

use super::geom::XDimension2D;

// ── StringBounder trait ──────────────────────────────────────────────

/// Measures text dimensions for layout calculations.
/// Java: `klimt.font.StringBounder`
///
/// This is the bridge between the klimt abstraction and our existing
/// font_metrics module.
pub trait StringBounder {
    /// Calculate the rendered dimensions of a string.
    fn calculate_dimension(
        &self,
        font_family: &str,
        font_size: f64,
        bold: bool,
        italic: bool,
        text: &str,
    ) -> XDimension2D;
}

// ── Default implementation using font_metrics ────────────────────────

/// Default StringBounder backed by the Java AWT-compatible font metrics table.
pub struct DefaultStringBounder;

impl StringBounder for DefaultStringBounder {
    fn calculate_dimension(
        &self,
        font_family: &str,
        font_size: f64,
        bold: bool,
        italic: bool,
        text: &str,
    ) -> XDimension2D {
        let width = crate::font_metrics::text_width(text, font_family, font_size, bold, italic);
        let height = crate::font_metrics::line_height(font_family, font_size, bold, italic);
        XDimension2D::new(width, height)
    }
}

impl Default for DefaultStringBounder {
    fn default() -> Self {
        Self
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bounder_measures_text() {
        let sb = DefaultStringBounder;
        let dim = sb.calculate_dimension("SansSerif", 14.0, false, false, "Hello");
        assert!(dim.width > 0.0);
        assert!(dim.height > 0.0);
    }

    #[test]
    fn bold_is_wider() {
        let sb = DefaultStringBounder;
        let normal = sb.calculate_dimension("SansSerif", 14.0, false, false, "Test");
        let bold = sb.calculate_dimension("SansSerif", 14.0, true, false, "Test");
        assert!(bold.width >= normal.width);
    }
}
