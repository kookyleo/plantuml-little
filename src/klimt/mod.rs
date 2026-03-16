// klimt - 2D graphics abstraction layer
// Port of Java PlantUML's net.sourceforge.plantuml.klimt package
//
// Named after Gustav Klimt, the Austrian painter.
// Provides output-format-independent drawing primitives that map 1:1
// with Java PlantUML's internal graphics API.

pub mod geom;
pub mod color;
pub mod shape;
pub mod font;
pub mod svg;

// ── UChange: marker trait for state changes applied to UGraphic ──────

/// Marker trait for changes that can be applied to a UGraphic context.
/// Java: `klimt.UChange` (empty interface)
///
/// Implementors: UStroke, UTranslate, HColor (foreground), UBackground, UPattern
pub trait UChange {}

// ── UStroke ──────────────────────────────────────────────────────────

/// Line stroke style: dash pattern + thickness.
/// Java: `klimt.UStroke`
#[derive(Debug, Clone, PartialEq)]
pub struct UStroke {
    pub dash_visible: f64,
    pub dash_space: f64,
    pub thickness: f64,
}

impl UChange for UStroke {}

impl UStroke {
    pub fn new(dash_visible: f64, dash_space: f64, thickness: f64) -> Self {
        Self { dash_visible, dash_space, thickness }
    }

    pub fn with_thickness(thickness: f64) -> Self {
        Self { dash_visible: 0.0, dash_space: 0.0, thickness }
    }

    pub fn simple() -> Self {
        Self::with_thickness(1.0)
    }

    pub fn only_thickness(&self) -> Self {
        Self { dash_visible: 0.0, dash_space: 0.0, thickness: self.thickness }
    }

    /// Returns dash array for SVG `stroke-dasharray`, or None if solid.
    pub fn dasharray_svg(&self) -> Option<(f64, f64)> {
        if self.dash_visible == 0.0 { None } else { Some((self.dash_visible, self.dash_space)) }
    }
}

impl Default for UStroke {
    fn default() -> Self { Self::simple() }
}

// ── UTranslate ───────────────────────────────────────────────────────

/// 2D translation offset.
/// Java: `klimt.UTranslate`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UTranslate {
    pub dx: f64,
    pub dy: f64,
}

impl UChange for UTranslate {}

impl UTranslate {
    pub fn new(dx: f64, dy: f64) -> Self { Self { dx, dy } }
    pub fn none() -> Self { Self { dx: 0.0, dy: 0.0 } }
    pub fn dx(dx: f64) -> Self { Self { dx, dy: 0.0 } }
    pub fn dy(dy: f64) -> Self { Self { dx: 0.0, dy } }

    pub fn compose(self, other: UTranslate) -> Self {
        Self { dx: self.dx + other.dx, dy: self.dy + other.dy }
    }

    pub fn reverse(self) -> Self {
        Self { dx: -self.dx, dy: -self.dy }
    }

    pub fn scaled(self, scale: f64) -> Self {
        Self { dx: self.dx * scale, dy: self.dy * scale }
    }
}

impl Default for UTranslate {
    fn default() -> Self { Self::none() }
}

// ── UBackground ──────────────────────────────────────────────────────

/// Background fill specification.
/// Java: `klimt.UBackground`
#[derive(Debug, Clone)]
pub enum UBackground {
    None,
    Color(color::HColor),
}

impl UChange for UBackground {}

// ── UPattern ─────────────────────────────────────────────────────────

/// Fill pattern.
/// Java: `klimt.UPattern`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UPattern {
    #[default]
    None,
    Striped,
    VerticalStriped,
}

impl UChange for UPattern {}

// ── UParam: current render state ─────────────────────────────────────

/// Accumulated rendering parameters (color, stroke, etc.)
/// Java: `klimt.UParam`
#[derive(Debug, Clone)]
pub struct UParam {
    pub color: color::HColor,
    pub backcolor: color::HColor,
    pub stroke: UStroke,
    pub pattern: UPattern,
    pub hidden: bool,
}

impl Default for UParam {
    fn default() -> Self {
        Self {
            color: color::HColor::simple("#000000"),
            backcolor: color::HColor::none(),
            stroke: UStroke::simple(),
            pattern: UPattern::None,
            hidden: false,
        }
    }
}

// ── UGraphic trait ───────────────────────────────────────────────────

/// The core drawing abstraction. All diagram renderers draw through this.
/// Java: `klimt.drawing.UGraphic`
///
/// Usage:
/// ```ignore
/// let mut ug = SvgGraphic::new(...);
/// ug.apply(UTranslate::new(10.0, 20.0));
/// ug.apply(UStroke::with_thickness(1.5));
/// ug.apply(HColor::simple("#FF0000")); // foreground
/// ug.draw_rect(100.0, 50.0, 5.0);     // rounded rect
/// ```
pub trait UGraphic {
    /// Apply a state change (translate, stroke, color, etc.)
    fn apply(&mut self, change: &dyn UChange);

    /// Get current render parameters
    fn param(&self) -> &UParam;

    /// Get the string bounder for text measurement
    fn string_bounder(&self) -> &dyn font::StringBounder;

    // ── Shape drawing methods ──
    // Instead of Java's generic `draw(UShape)` with runtime dispatch,
    // we use explicit methods for type safety.

    fn draw_rect(&mut self, width: f64, height: f64, rx: f64);
    fn draw_ellipse(&mut self, width: f64, height: f64);
    fn draw_line(&mut self, dx: f64, dy: f64);
    fn draw_text(&mut self, text: &str, font_family: &str, font_size: f64, bold: bool, italic: bool);
    fn draw_path(&mut self, path: &shape::UPath);
    fn draw_polygon(&mut self, points: &[(f64, f64)]);

    // ── Group/URL management ──
    fn start_group(&mut self, id: &str);
    fn close_group(&mut self);
    fn start_url(&mut self, url: &str, tooltip: &str);
    fn close_url(&mut self);
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ustroke_default_is_solid_1px() {
        let s = UStroke::default();
        assert_eq!(s.thickness, 1.0);
        assert!(s.dasharray_svg().is_none());
    }

    #[test]
    fn ustroke_dashed() {
        let s = UStroke::new(5.0, 5.0, 1.0);
        assert_eq!(s.dasharray_svg(), Some((5.0, 5.0)));
    }

    #[test]
    fn utranslate_compose() {
        let a = UTranslate::new(10.0, 20.0);
        let b = UTranslate::new(5.0, -3.0);
        let c = a.compose(b);
        assert_eq!(c.dx, 15.0);
        assert_eq!(c.dy, 17.0);
    }

    #[test]
    fn utranslate_reverse() {
        let t = UTranslate::new(10.0, -5.0);
        let r = t.reverse();
        assert_eq!(r.dx, -10.0);
        assert_eq!(r.dy, 5.0);
    }
}
