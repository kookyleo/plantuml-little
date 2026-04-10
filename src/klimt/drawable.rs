// klimt::drawable - Shape abstraction for SVG rendering
//
// Provides a `Drawable` trait and concrete shape types that encapsulate
// the pattern of setting SvgGraphic state (fill/stroke/stroke-width)
// then calling the corresponding svg_* method.
//
// This is the Rust analogue of Java PlantUML's UGraphic → Driver pattern,
// where each shape knows how to render itself given a graphics context.

use super::shape::UPath;
use super::svg::SvgGraphic;

// ── DrawStyle ──────────────────────────────────────────────────────

/// Visual style applied before drawing a shape.
///
/// Maps to the SvgGraphic state setters: `set_fill_color`,
/// `set_stroke_color`, `set_stroke_width`.
#[derive(Debug, Clone)]
pub struct DrawStyle {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f64,
    pub dash_array: Option<(f64, f64)>,
    pub delta_shadow: f64,
}

impl DrawStyle {
    /// Convenience: outline-only style (no fill, given stroke color and width).
    pub fn outline(stroke: &str, width: f64) -> Self {
        Self {
            fill: Some("none".into()),
            stroke: Some(stroke.into()),
            stroke_width: width,
            dash_array: None,
            delta_shadow: 0.0,
        }
    }

    /// Convenience: filled shape with stroke.
    pub fn filled(fill: &str, stroke: &str, width: f64) -> Self {
        Self {
            fill: Some(fill.into()),
            stroke: Some(stroke.into()),
            stroke_width: width,
            dash_array: None,
            delta_shadow: 0.0,
        }
    }

    /// Convenience: fill-only style (no visible stroke).
    pub fn fill_only(fill: &str) -> Self {
        Self {
            fill: Some(fill.into()),
            stroke: None,
            stroke_width: 0.0,
            dash_array: None,
            delta_shadow: 0.0,
        }
    }

    /// Apply this style to an SvgGraphic, setting fill/stroke/stroke-width.
    pub fn apply(&self, sg: &mut SvgGraphic) {
        if let Some(ref fill) = self.fill {
            sg.set_fill_color(fill);
        }
        sg.set_stroke_color(self.stroke.as_deref());
        sg.set_stroke_width(self.stroke_width, self.dash_array);
    }
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self {
            fill: None,
            stroke: Some("#000000".into()),
            stroke_width: 1.0,
            dash_array: None,
            delta_shadow: 0.0,
        }
    }
}

// ── Drawable trait ─────────────────────────────────────────────────

/// A shape that can render itself onto an SvgGraphic.
///
/// Usage:
/// ```ignore
/// let rect = RectShape { x: 10.0, y: 20.0, w: 100.0, h: 50.0, rx: 5.0, ry: 5.0 };
/// let style = DrawStyle::filled("#F1F1F1", "#181818", 0.5);
/// rect.draw(&mut sg, &style);
/// ```
pub trait Drawable {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle);
}

// ── RectShape ──────────────────────────────────────────────────────

/// Rectangle with optional rounded corners.
/// Maps to `SvgGraphic::svg_rectangle`.
#[derive(Debug, Clone)]
pub struct RectShape {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub rx: f64,
    pub ry: f64,
}

impl Drawable for RectShape {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle) {
        style.apply(sg);
        sg.svg_rectangle(self.x, self.y, self.w, self.h, self.rx, self.ry, style.delta_shadow);
    }
}

// ── LineShape ──────────────────────────────────────────────────────

/// A line segment between two points.
/// Maps to `SvgGraphic::svg_line`.
#[derive(Debug, Clone)]
pub struct LineShape {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Drawable for LineShape {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle) {
        style.apply(sg);
        sg.svg_line(self.x1, self.y1, self.x2, self.y2, style.delta_shadow);
    }
}

// ── EllipseShape ───────────────────────────────────────────────────

/// An ellipse centered at (cx, cy) with radii (rx, ry).
/// Maps to `SvgGraphic::svg_ellipse`.
#[derive(Debug, Clone)]
pub struct EllipseShape {
    pub cx: f64,
    pub cy: f64,
    pub rx: f64,
    pub ry: f64,
}

impl Drawable for EllipseShape {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle) {
        style.apply(sg);
        sg.svg_ellipse(self.cx, self.cy, self.rx, self.ry, style.delta_shadow);
    }
}

// ── PathShape ──────────────────────────────────────────────────────

/// A vector path (UPath) drawn at an offset (x, y).
/// Maps to `SvgGraphic::svg_path`.
#[derive(Debug, Clone)]
pub struct PathShape {
    pub x: f64,
    pub y: f64,
    pub path: UPath,
}

impl Drawable for PathShape {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle) {
        style.apply(sg);
        sg.svg_path(self.x, self.y, &self.path, style.delta_shadow);
    }
}

// ── CircleShape ───────────────────────────────────────────────────

/// A circle centered at (cx, cy) with radius r.
/// Maps to `SvgGraphic::svg_circle`.
#[derive(Debug, Clone)]
pub struct CircleShape {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}

impl Drawable for CircleShape {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle) {
        style.apply(sg);
        sg.svg_circle(self.cx, self.cy, self.r, style.delta_shadow);
    }
}

// ── PolygonShape ───────────────────────────────────────────────────

/// A closed polygon defined by a flat list of coordinate pairs [x0,y0,x1,y1,...].
/// Maps to `SvgGraphic::svg_polygon`.
#[derive(Debug, Clone)]
pub struct PolygonShape {
    /// Flat coordinate array: [x0, y0, x1, y1, ...]
    pub points: Vec<f64>,
}

impl Drawable for PolygonShape {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle) {
        style.apply(sg);
        sg.svg_polygon(style.delta_shadow, &self.points);
    }
}

// ── TextShape ──────────────────────────────────────────────────────

/// A text element. Maps to `SvgGraphic::svg_text`.
///
/// Captures the common defaults used across renderers: no bold/italic,
/// `LengthAdjust::Spacing`, no text background, no rotation, no anchor.
/// For specialized text rendering, callers should use SvgGraphic directly.
#[derive(Debug, Clone)]
pub struct TextShape {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub font_family: String,
    pub font_size: f64,
    pub text_length: f64,
    pub bold: bool,
    pub italic: bool,
}

impl Drawable for TextShape {
    fn draw(&self, sg: &mut SvgGraphic, style: &DrawStyle) {
        style.apply(sg);
        let font_weight = if self.bold { Some("bold") } else { None };
        let font_style = if self.italic { Some("italic") } else { None };
        sg.svg_text(
            &self.text,
            self.x,
            self.y,
            Some(&self.font_family),
            self.font_size,
            font_weight,
            font_style,
            None, // text_decoration
            self.text_length,
            super::svg::LengthAdjust::Spacing,
            None, // text_back_color
            0,    // orientation
            None, // text_anchor
        );
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_style_outline() {
        let s = DrawStyle::outline("#FF0000", 2.0);
        assert_eq!(s.fill.as_deref(), Some("none"));
        assert_eq!(s.stroke.as_deref(), Some("#FF0000"));
        assert_eq!(s.stroke_width, 2.0);
        assert!(s.dash_array.is_none());
        assert_eq!(s.delta_shadow, 0.0);
    }

    #[test]
    fn draw_style_filled() {
        let s = DrawStyle::filled("#FFFFFF", "#000000", 1.0);
        assert_eq!(s.fill.as_deref(), Some("#FFFFFF"));
        assert_eq!(s.stroke.as_deref(), Some("#000000"));
        assert_eq!(s.stroke_width, 1.0);
    }

    #[test]
    fn draw_style_fill_only() {
        let s = DrawStyle::fill_only("#AABBCC");
        assert_eq!(s.fill.as_deref(), Some("#AABBCC"));
        assert!(s.stroke.is_none());
        assert_eq!(s.stroke_width, 0.0);
    }

    #[test]
    fn rect_shape_draw_produces_svg() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let rect = RectShape {
            x: 10.0,
            y: 20.0,
            w: 100.0,
            h: 50.0,
            rx: 5.0,
            ry: 5.0,
        };
        let style = DrawStyle::filled("#F1F1F1", "#181818", 0.5);
        rect.draw(&mut sg, &style);
        let body = sg.body();
        assert!(body.contains("<rect"));
        assert!(body.contains("width=\"100\""));
        assert!(body.contains("height=\"50\""));
        assert!(body.contains("rx=\"5\""));
    }

    #[test]
    fn line_shape_draw_produces_svg() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let line = LineShape {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 50.0,
        };
        let style = DrawStyle::outline("#000000", 1.0);
        line.draw(&mut sg, &style);
        let body = sg.body();
        assert!(body.contains("<line"));
        assert!(body.contains("x1=\"0\""));
        assert!(body.contains("x2=\"100\""));
    }

    #[test]
    fn ellipse_shape_draw_produces_svg() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let ell = EllipseShape {
            cx: 50.0,
            cy: 50.0,
            rx: 30.0,
            ry: 20.0,
        };
        let style = DrawStyle::filled("#AABB00", "#000000", 1.0);
        ell.draw(&mut sg, &style);
        let body = sg.body();
        assert!(body.contains("<ellipse"));
        assert!(body.contains("cx=\"50\""));
        assert!(body.contains("rx=\"30\""));
    }

    #[test]
    fn polygon_shape_draw_produces_svg() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let poly = PolygonShape {
            points: vec![0.0, 0.0, 10.0, 0.0, 5.0, 10.0],
        };
        let style = DrawStyle::filled("#000000", "#000000", 0.5);
        poly.draw(&mut sg, &style);
        let body = sg.body();
        assert!(body.contains("<polygon"));
        assert!(body.contains("points="));
    }

    #[test]
    fn path_shape_draw_produces_svg() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 10.0);
        let shape = PathShape {
            x: 5.0,
            y: 5.0,
            path,
        };
        let style = DrawStyle::fill_only("#000000");
        shape.draw(&mut sg, &style);
        let body = sg.body();
        assert!(body.contains("<path"));
        assert!(body.contains("d="));
    }

    #[test]
    fn text_shape_draw_produces_svg() {
        let mut sg = SvgGraphic::new(0, 1.0);
        let text = TextShape {
            x: 10.0,
            y: 20.0,
            text: "Hello".into(),
            font_family: "sans-serif".into(),
            font_size: 12.0,
            text_length: 30.0,
            bold: false,
            italic: false,
        };
        let style = DrawStyle::fill_only("#000000");
        text.draw(&mut sg, &style);
        let body = sg.body();
        assert!(body.contains("<text"));
        assert!(body.contains("Hello"));
    }
}
