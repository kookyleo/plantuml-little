// klimt::shape - Drawing shape primitives
// Port of Java PlantUML's UShape implementations

use std::fmt;

use super::geom::{USegment, XCubicCurve2D, XDimension2D, XPoint2D};

// ── Shadowable trait ────────────────────────────────────────────────

/// Trait for shapes that support drop shadows.
/// Java: `klimt.Shadowable` + `klimt.AbstractShadowable`
pub trait Shadowable {
    fn delta_shadow(&self) -> f64;
    fn set_delta_shadow(&mut self, delta: f64);
}

// ── UPath ────────────────────────────────────────────────────────────

/// General-purpose vector path built from segments.
/// Java: `klimt.UPath`
#[derive(Debug, Clone, Default)]
pub struct UPath {
    pub segments: Vec<USegment>,
    pub shadow: f64,
    pub comment: Option<String>,
}

impl UPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.segments.push(USegment {
            kind: super::geom::USegmentType::MoveTo,
            coords: vec![x, y],
        });
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.segments.push(USegment {
            kind: super::geom::USegmentType::LineTo,
            coords: vec![x, y],
        });
    }

    pub fn cubic_to(&mut self, cx1: f64, cy1: f64, cx2: f64, cy2: f64, x: f64, y: f64) {
        self.segments.push(USegment {
            kind: super::geom::USegmentType::CubicTo,
            coords: vec![cx1, cy1, cx2, cy2, x, y],
        });
    }

    pub fn arc_to(
        &mut self,
        rx: f64,
        ry: f64,
        x_rot: f64,
        large_arc: f64,
        sweep: f64,
        x: f64,
        y: f64,
    ) {
        self.segments.push(USegment {
            kind: super::geom::USegmentType::ArcTo,
            coords: vec![rx, ry, x_rot, large_arc, sweep, x, y],
        });
    }

    pub fn close(&mut self) {
        self.segments.push(USegment {
            kind: super::geom::USegmentType::Close,
            coords: vec![],
        });
    }

    /// Convert to SVG path `d` attribute string.
    pub fn to_svg_path_d(&self) -> String {
        use super::geom::USegmentType::*;
        let mut d = String::new();
        for seg in &self.segments {
            match seg.kind {
                MoveTo => {
                    if !d.is_empty() {
                        d.push(' ');
                    }
                    d.push_str(&format!("M{:.4} {:.4}", seg.coords[0], seg.coords[1]));
                }
                LineTo => d.push_str(&format!(" L{:.4} {:.4}", seg.coords[0], seg.coords[1])),
                CubicTo => d.push_str(&format!(
                    " C{:.4} {:.4} {:.4} {:.4} {:.4} {:.4}",
                    seg.coords[0],
                    seg.coords[1],
                    seg.coords[2],
                    seg.coords[3],
                    seg.coords[4],
                    seg.coords[5]
                )),
                ArcTo => d.push_str(&format!(
                    " A{:.4} {:.4} {:.4} {} {} {:.4} {:.4}",
                    seg.coords[0],
                    seg.coords[1],
                    seg.coords[2],
                    seg.coords[3] as i32,
                    seg.coords[4] as i32,
                    seg.coords[5],
                    seg.coords[6]
                )),
                Close => d.push_str(" Z"),
            }
        }
        d
    }
}

impl Shadowable for UPath {
    fn delta_shadow(&self) -> f64 {
        self.shadow
    }
    fn set_delta_shadow(&mut self, delta: f64) {
        self.shadow = delta;
    }
}

// ── URectangle ──────────────────────────────────────────────────────

/// Rectangle shape with optional rounded corners and shadow.
/// Java: `klimt.shape.URectangle`
#[derive(Debug, Clone)]
pub struct URectangle {
    pub width: f64,
    pub height: f64,
    pub rx: f64,
    pub ry: f64,
    pub shadow: f64,
    pub comment: Option<String>,
}

impl URectangle {
    pub fn build(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            rx: 0.0,
            ry: 0.0,
            shadow: 0.0,
            comment: None,
        }
    }

    pub fn from_dim(dim: XDimension2D) -> Self {
        Self::build(dim.width, dim.height)
    }

    /// Return a copy with the given rounded corner radius (rx = ry = round).
    pub fn rounded(&self, round: f64) -> Self {
        Self {
            rx: round,
            ry: round,
            ..self.clone()
        }
    }

    pub fn with_height(&self, new_height: f64) -> Self {
        Self {
            height: new_height,
            ..self.clone()
        }
    }

    pub fn with_width(&self, new_width: f64) -> Self {
        Self {
            width: new_width,
            ..self.clone()
        }
    }

    pub fn with_comment(&self, comment: String) -> Self {
        Self {
            comment: Some(comment),
            ..self.clone()
        }
    }

    /// Create a UPath representing a rectangle with diagonal-cut corners.
    pub fn diagonal_corner(&self, d: f64) -> UPath {
        let mut path = UPath::new();
        if d == 0.0 {
            path.move_to(0.0, 0.0);
            path.line_to(self.width, 0.0);
            path.line_to(self.width, self.height);
            path.line_to(0.0, self.height);
            path.close();
        } else {
            path.move_to(d, 0.0);
            path.line_to(self.width - d, 0.0);
            path.line_to(self.width, d);
            path.line_to(self.width, self.height - d);
            path.line_to(self.width - d, self.height);
            path.line_to(d, self.height);
            path.line_to(0.0, self.height - d);
            path.line_to(0.0, d);
            path.line_to(d, 0.0);
        }
        path
    }

    /// Create a UPath for a half-rounded rectangle (rounded top, flat bottom).
    pub fn half_rounded(&self, round_corner: f64) -> UPath {
        let mut path = UPath::new();
        if round_corner == 0.0 {
            path.move_to(0.0, 0.0);
            path.line_to(self.width, 0.0);
            path.line_to(self.width, self.height);
            path.line_to(0.0, self.height);
            path.close();
        } else {
            let r = round_corner / 2.0;
            path.move_to(r, 0.0);
            path.line_to(self.width - r, 0.0);
            path.arc_to(r, r, 0.0, 0.0, 1.0, self.width, r);
            path.line_to(self.width, self.height);
            path.line_to(0.0, self.height);
            path.line_to(0.0, r);
            path.arc_to(r, r, 0.0, 0.0, 1.0, r, 0.0);
        }
        path
    }
}

impl fmt::Display for URectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "width={} height={}", self.width, self.height)
    }
}

impl Shadowable for URectangle {
    fn delta_shadow(&self) -> f64 {
        self.shadow
    }
    fn set_delta_shadow(&mut self, delta: f64) {
        self.shadow = delta;
    }
}

// ── UEllipse ────────────────────────────────────────────────────────

/// Ellipse shape. Full ellipse or arc segment.
/// Java: `klimt.shape.UEllipse`
///
/// `width`/`height` define the bounding box.
/// `start` and `extend` define arc angles in degrees (0 = full ellipse).
#[derive(Debug, Clone)]
pub struct UEllipse {
    pub width: f64,
    pub height: f64,
    pub start: f64,
    pub extend: f64,
    pub shadow: f64,
}

impl UEllipse {
    pub fn build(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            start: 0.0,
            extend: 0.0,
            shadow: 0.0,
        }
    }

    pub fn arc(width: f64, height: f64, start: f64, extend: f64) -> Self {
        Self {
            width,
            height,
            start,
            extend,
            shadow: 0.0,
        }
    }

    pub fn dimension(&self) -> XDimension2D {
        XDimension2D::new(self.width, self.height)
    }

    /// Return a larger ellipse (add `more` to both width and height).
    pub fn bigger(&self, more: f64) -> Self {
        let mut result = Self::build(self.width + more, self.height + more);
        result.shadow = self.shadow;
        result
    }

    /// Return a scaled ellipse.
    pub fn scale(&self, factor: f64) -> Self {
        let mut result = Self::build(self.width * factor, self.height * factor);
        result.shadow = self.shadow;
        result
    }

    /// X coordinate on the left edge of the ellipse at given y.
    pub fn starting_x(&self, y: f64) -> f64 {
        let yn = y / self.height * 2.0;
        let x = 1.0 - (1.0 - (yn - 1.0) * (yn - 1.0)).sqrt();
        x * self.width / 2.0
    }

    /// X coordinate on the right edge of the ellipse at given y.
    pub fn ending_x(&self, y: f64) -> f64 {
        let yn = y / self.height * 2.0;
        let x = 1.0 + (1.0 - (yn - 1.0) * (yn - 1.0)).sqrt();
        x * self.width / 2.0
    }

    /// Point on the ellipse boundary at the given angle (radians).
    pub fn point_at_angle(&self, alpha: f64) -> XPoint2D {
        let x = self.width / 2.0 + self.width / 2.0 * alpha.cos();
        let y = self.height / 2.0 + self.height / 2.0 * alpha.sin();
        XPoint2D::new(x, y)
    }
}

impl fmt::Display for UEllipse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UEllipse({}x{})", self.width, self.height)
    }
}

impl Shadowable for UEllipse {
    fn delta_shadow(&self) -> f64 {
        self.shadow
    }
    fn set_delta_shadow(&mut self, delta: f64) {
        self.shadow = delta;
    }
}

// ── ULine ───────────────────────────────────────────────────────────

/// Relative line segment (dx, dy from current position).
/// Java: `klimt.shape.ULine`
#[derive(Debug, Clone, Copy)]
pub struct ULine {
    pub dx: f64,
    pub dy: f64,
    pub shadow: f64,
}

impl ULine {
    pub fn new(dx: f64, dy: f64) -> Self {
        Self {
            dx,
            dy,
            shadow: 0.0,
        }
    }

    pub fn from_points(p1: XPoint2D, p2: XPoint2D) -> Self {
        Self::new(p2.x - p1.x, p2.y - p1.y)
    }

    /// Horizontal line of length dx.
    pub fn hline(dx: f64) -> Self {
        Self::new(dx, 0.0)
    }

    /// Vertical line of length dy.
    pub fn vline(dy: f64) -> Self {
        Self::new(0.0, dy)
    }

    /// Rotate the line vector by the given angle in radians.
    pub fn rotate(&self, theta: f64) -> Self {
        if theta == 0.0 {
            return *self;
        }
        let cos = theta.cos();
        let sin = theta.sin();
        Self::new(self.dx * cos - self.dy * sin, self.dx * sin + self.dy * cos)
    }

    pub fn length(&self) -> f64 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }
}

impl fmt::Display for ULine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ULine dx={} dy={}", self.dx, self.dy)
    }
}

impl Shadowable for ULine {
    fn delta_shadow(&self) -> f64 {
        self.shadow
    }
    fn set_delta_shadow(&mut self, delta: f64) {
        self.shadow = delta;
    }
}

// ── UText ───────────────────────────────────────────────────────────

/// Text string with font configuration.
/// Java: `klimt.shape.UText`
///
/// Unlike Java, font configuration is stored inline (family, size, bold, italic)
/// rather than referencing a separate FontConfiguration object, since the Rust
/// codebase does not yet have a full FontConfiguration type.
#[derive(Debug, Clone)]
pub struct UText {
    pub text: String,
    pub font_family: String,
    pub font_size: f64,
    pub bold: bool,
    pub italic: bool,
    pub orientation: i32,
}

impl UText {
    pub fn build(text: &str, font_family: &str, font_size: f64, bold: bool, italic: bool) -> Self {
        Self {
            text: text.to_string(),
            font_family: font_family.to_string(),
            font_size,
            bold,
            italic,
            orientation: 0,
        }
    }

    pub fn with_orientation(mut self, orientation: i32) -> Self {
        self.orientation = orientation;
        self
    }
}

impl fmt::Display for UText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UText[{}]", self.text)
    }
}

// ── DotPath ─────────────────────────────────────────────────────────

/// Series of cubic Bezier curves forming a smooth path (used for edges).
/// Java: `klimt.shape.DotPath`
#[derive(Debug, Clone, Default)]
pub struct DotPath {
    pub beziers: Vec<XCubicCurve2D>,
    pub comment: Option<String>,
}

impl DotPath {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_beziers(beziers: Vec<XCubicCurve2D>) -> Self {
        Self {
            beziers,
            comment: None,
        }
    }

    /// Append a cubic curve from four points.
    pub fn add_curve_4(&self, p1: XPoint2D, p2: XPoint2D, p3: XPoint2D, p4: XPoint2D) -> Self {
        let mut copy = self.beziers.clone();
        copy.push(XCubicCurve2D::new(
            p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, p4.x, p4.y,
        ));
        Self::from_beziers(copy)
    }

    /// Append a cubic curve continuing from the end of the last segment.
    pub fn add_curve(&self, p2: XPoint2D, p3: XPoint2D, p4: XPoint2D) -> Self {
        let last = self.beziers.last().expect("DotPath is empty");
        let p1 = last.p2();
        self.add_curve_4(p1, p2, p3, p4)
    }

    pub fn start_point(&self) -> XPoint2D {
        self.beziers[0].p1()
    }

    pub fn end_point(&self) -> XPoint2D {
        self.beziers.last().unwrap().p2()
    }

    /// Translate the start point by (dx, dy), adjusting first bezier.
    pub fn move_start_point(&mut self, dx: f64, dy: f64) {
        if self.beziers.len() > 1 {
            let dist = (dx * dx + dy * dy).sqrt();
            if dist >= self.beziers[0].length() {
                let dx2 = dx - (self.beziers[1].x1 - self.beziers[0].x1);
                let dy2 = dy - (self.beziers[1].y1 - self.beziers[0].y1);
                self.beziers.remove(0);
                self.beziers[0].x1 += dx2;
                self.beziers[0].y1 += dy2;
                self.beziers[0].ctrlx1 += dx2;
                self.beziers[0].ctrly1 += dy2;
                return;
            }
        }
        self.beziers[0].x1 += dx;
        self.beziers[0].y1 += dy;
        self.beziers[0].ctrlx1 += dx;
        self.beziers[0].ctrly1 += dy;
    }

    /// Translate the end point by (dx, dy), adjusting last bezier.
    pub fn move_end_point(&mut self, dx: f64, dy: f64) {
        let last = self.beziers.last_mut().unwrap();
        last.x2 += dx;
        last.y2 += dy;
        last.ctrlx2 += dx;
        last.ctrly2 += dy;
    }

    /// Move all points by (dx, dy).
    pub fn move_delta(&mut self, dx: f64, dy: f64) {
        for c in &mut self.beziers {
            *c = XCubicCurve2D::new(
                c.x1 + dx,
                c.y1 + dy,
                c.ctrlx1 + dx,
                c.ctrly1 + dy,
                c.ctrlx2 + dx,
                c.ctrly2 + dy,
                c.x2 + dx,
                c.y2 + dy,
            );
        }
    }

    /// Reverse the path direction.
    pub fn reverse(&self) -> Self {
        let mut rev: Vec<XCubicCurve2D> = self
            .beziers
            .iter()
            .map(|c| {
                XCubicCurve2D::new(
                    c.x2, c.y2, c.ctrlx2, c.ctrly2, c.ctrlx1, c.ctrly1, c.x1, c.y1,
                )
            })
            .collect();
        rev.reverse();
        Self::from_beziers(rev)
    }

    pub fn add_before(&self, before: XCubicCurve2D) -> Self {
        let mut copy = vec![before];
        copy.extend_from_slice(&self.beziers);
        Self::from_beziers(copy)
    }

    pub fn add_after(&self, after: XCubicCurve2D) -> Self {
        let mut copy = self.beziers.clone();
        copy.push(after);
        Self::from_beziers(copy)
    }

    /// Concatenate another DotPath after this one.
    pub fn concat_after(&self, other: &DotPath) -> Self {
        let mut copy = self.beziers.clone();
        copy.extend_from_slice(&other.beziers);
        Self::from_beziers(copy)
    }

    /// Convert to a UPath (move-to first point, then cubic-to segments).
    pub fn to_upath(&self) -> UPath {
        let mut result = UPath::new();
        for (i, bez) in self.beziers.iter().enumerate() {
            if i == 0 {
                result.move_to(bez.x1, bez.y1);
            }
            result.cubic_to(
                bez.ctrlx1, bez.ctrly1, bez.ctrlx2, bez.ctrly2, bez.x2, bez.y2,
            );
        }
        result
    }

    /// Angle of the tangent at the end of the path.
    pub fn end_angle(&self) -> f64 {
        let last = self.beziers.last().unwrap();
        let mut dx = last.x2 - last.ctrlx2;
        let mut dy = last.y2 - last.ctrly2;
        if dx == 0.0 && dy == 0.0 {
            dx = last.x2 - last.x1;
            dy = last.y2 - last.y1;
        }
        dy.atan2(dx)
    }

    /// Angle of the tangent at the start of the path.
    pub fn start_angle(&self) -> f64 {
        let first = &self.beziers[0];
        let mut dx = first.ctrlx1 - first.x1;
        let mut dy = first.ctrly1 - first.y1;
        if dx == 0.0 && dy == 0.0 {
            dx = first.x2 - first.x1;
            dy = first.y2 - first.y1;
        }
        dy.atan2(dx)
    }

    /// Check if the path is approximately a straight line.
    pub fn is_line(&self) -> bool {
        self.beziers.iter().all(|c| c.flatness_sq() <= 0.001)
    }

    /// Compute the bounding box over all bezier control points.
    /// Java: `DotPath.getMinMax()` — uses all four points of each cubic.
    pub fn min_max(&self) -> Option<(f64, f64, f64, f64)> {
        if self.beziers.is_empty() {
            return None;
        }
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for c in &self.beziers {
            for &(x, y) in &[
                (c.x1, c.y1),
                (c.ctrlx1, c.ctrly1),
                (c.ctrlx2, c.ctrly2),
                (c.x2, c.y2),
            ] {
                if x < min_x {
                    min_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if x > max_x {
                    max_x = x;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
        Some((min_x, min_y, max_x, max_y))
    }
}

impl fmt::Display for DotPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, c) in self.beziers.iter().enumerate() {
            if i > 0 {
                write!(f, " - ")?;
            }
            write!(
                f,
                "({},{}) ({},{}) ({},{}) ({},{})",
                c.x1, c.y1, c.ctrlx1, c.ctrly1, c.ctrlx2, c.ctrly2, c.x2, c.y2
            )?;
        }
        Ok(())
    }
}

// ── UEmpty ──────────────────────────────────────────────────────────

/// Invisible shape that reserves space in the layout.
/// Java: `klimt.shape.UEmpty`
#[derive(Debug, Clone, Copy)]
pub struct UEmpty {
    pub width: f64,
    pub height: f64,
}

impl UEmpty {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn from_dim(dim: XDimension2D) -> Self {
        Self {
            width: dim.width,
            height: dim.height,
        }
    }
}

impl fmt::Display for UEmpty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UEmpty({}x{})", self.width, self.height)
    }
}

// ── UComment ────────────────────────────────────────────────────────

/// SVG/XML comment node.
/// Java: `klimt.shape.UComment`
#[derive(Debug, Clone)]
pub struct UComment {
    pub comment: String,
}

impl UComment {
    pub fn new(comment: &str) -> Self {
        Self {
            comment: comment.to_string(),
        }
    }
}

impl fmt::Display for UComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UComment[{}]", self.comment)
    }
}

// ── UImage ──────────────────────────────────────────────────────────

/// Raster image shape.
/// Java: `klimt.shape.UImage`
///
/// Stores raw image data as bytes with width/height metadata.
#[derive(Debug, Clone)]
pub struct UImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub formula: Option<String>,
    pub raw_file_name: Option<String>,
}

impl UImage {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            scale: 1.0,
            formula: None,
            raw_file_name: None,
        }
    }

    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_formula(mut self, formula: String) -> Self {
        self.formula = Some(formula);
        self
    }

    pub fn with_raw_file_name(mut self, name: String) -> Self {
        self.raw_file_name = Some(name);
        self
    }

    pub fn scaled_width(&self) -> f64 {
        self.width as f64 * self.scale
    }

    pub fn scaled_height(&self) -> f64 {
        self.height as f64 * self.scale
    }
}

// ── UImageSvg ───────────────────────────────────────────────────────

/// Inline SVG image shape.
/// Java: `klimt.shape.UImageSvg`
#[derive(Debug, Clone)]
pub struct UImageSvg {
    pub svg: String,
    pub scale: f64,
}

impl UImageSvg {
    pub fn new(svg: String, scale: f64) -> Self {
        Self { svg, scale }
    }

    /// Extract the SVG content, stripping XML prologue and outer SVG attributes.
    pub fn get_svg(&self, raw: bool) -> String {
        if raw {
            return self.svg.clone();
        }
        let mut result = self.svg.clone();
        if result.starts_with("<?xml") {
            if let Some(idx) = result.find("<svg") {
                result = result[idx..].to_string();
            }
        }
        if result.starts_with("<svg") {
            if let Some(idx) = result.find('>') {
                result = format!("<svg>{}", &result[idx + 1..]);
            }
        }
        result
    }

    /// Parse a numeric attribute from the SVG tag.
    pub fn get_data(&self, name: &str) -> Option<i32> {
        // Try viewBox first
        let viewbox_re = regex::Regex::new(
            r#"viewBox[= "']+([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)"#,
        )
        .ok()?;
        if let Some(caps) = viewbox_re.captures(&self.svg) {
            match name {
                "width" => {
                    return caps
                        .get(3)
                        .and_then(|m| m.as_str().parse::<f64>().ok())
                        .map(|v| v as i32)
                }
                "height" => {
                    return caps
                        .get(4)
                        .and_then(|m| m.as_str().parse::<f64>().ok())
                        .map(|v| v as i32)
                }
                _ => {}
            }
        }
        // Fallback: parse attribute from <svg> tag
        let pattern = format!(r#"(?i)<svg[^>]+{}\W+(\d+)"#, regex::escape(name));
        let re = regex::Regex::new(&pattern).ok()?;
        re.captures(&self.svg)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<i32>().ok())
    }

    pub fn width(&self) -> f64 {
        self.get_data("width").unwrap_or(0) as f64 * self.scale
    }

    pub fn height(&self) -> f64 {
        self.get_data("height").unwrap_or(0) as f64 * self.scale
    }
}

// ── UHorizontalLine ─────────────────────────────────────────────────

/// Horizontal divider line (e.g., separators within class boxes).
/// Java: `klimt.shape.UHorizontalLine`
#[derive(Debug, Clone)]
pub struct UHorizontalLine {
    pub skip_at_start: f64,
    pub skip_at_end: f64,
    /// `'='` for double, `'.'` for dotted, `'-'` for solid, `'\0'` for default.
    pub style: char,
    pub default_thickness: f64,
}

impl UHorizontalLine {
    pub fn infinite(
        default_thickness: f64,
        skip_at_start: f64,
        skip_at_end: f64,
        style: char,
    ) -> Self {
        Self {
            skip_at_start,
            skip_at_end,
            style,
            default_thickness,
        }
    }

    pub fn is_double(&self) -> bool {
        self.style == '='
    }
}

// ── UPixel ──────────────────────────────────────────────────────────

/// Single pixel point. Marker shape with no data.
/// Java: `klimt.shape.UPixel`
#[derive(Debug, Clone, Copy, Default)]
pub struct UPixel;

// ── UCenteredCharacter ──────────────────────────────────────────────

/// A single character centered at the drawing position.
/// Java: `klimt.shape.UCenteredCharacter`
#[derive(Debug, Clone)]
pub struct UCenteredCharacter {
    pub c: char,
    pub font_family: String,
    pub font_size: f64,
}

impl UCenteredCharacter {
    pub fn new(c: char, font_family: &str, font_size: f64) -> Self {
        Self {
            c,
            font_family: font_family.to_string(),
            font_size,
        }
    }
}

impl fmt::Display for UCenteredCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UCenteredCharacter[{}]", self.c)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── UPath tests ──

    #[test]
    fn upath_rect() {
        let mut p = UPath::new();
        p.move_to(10.0, 20.0);
        p.line_to(110.0, 20.0);
        p.line_to(110.0, 70.0);
        p.line_to(10.0, 70.0);
        p.close();
        let d = p.to_svg_path_d();
        assert!(d.starts_with("M10"));
        assert!(d.contains("L110"));
        assert!(d.ends_with(" Z"));
    }

    #[test]
    fn upath_shadow() {
        let mut p = UPath::new();
        assert_eq!(p.delta_shadow(), 0.0);
        p.set_delta_shadow(3.0);
        assert_eq!(p.delta_shadow(), 3.0);
    }

    // ── URectangle tests ──

    #[test]
    fn urect_build() {
        let r = URectangle::build(100.0, 50.0);
        assert_eq!(r.width, 100.0);
        assert_eq!(r.height, 50.0);
        assert_eq!(r.rx, 0.0);
        assert_eq!(r.ry, 0.0);
    }

    #[test]
    fn urect_rounded() {
        let r = URectangle::build(100.0, 50.0).rounded(10.0);
        assert_eq!(r.rx, 10.0);
        assert_eq!(r.ry, 10.0);
        assert_eq!(r.width, 100.0);
    }

    #[test]
    fn urect_with_dimensions() {
        let r = URectangle::build(100.0, 50.0);
        let r2 = r.with_height(80.0);
        assert_eq!(r2.height, 80.0);
        assert_eq!(r2.width, 100.0);
        let r3 = r.with_width(200.0);
        assert_eq!(r3.width, 200.0);
        assert_eq!(r3.height, 50.0);
    }

    #[test]
    fn urect_shadow() {
        let mut r = URectangle::build(100.0, 50.0);
        assert_eq!(r.delta_shadow(), 0.0);
        r.set_delta_shadow(4.0);
        assert_eq!(r.delta_shadow(), 4.0);
    }

    #[test]
    fn urect_diagonal_corner() {
        let r = URectangle::build(100.0, 50.0);
        let path = r.diagonal_corner(10.0);
        assert_eq!(path.segments.len(), 9); // move + 7 lines + close... actually 8 lines + move
        let d = path.to_svg_path_d();
        assert!(d.starts_with("M10"));
    }

    #[test]
    fn urect_from_dim() {
        let dim = XDimension2D::new(120.0, 60.0);
        let r = URectangle::from_dim(dim);
        assert_eq!(r.width, 120.0);
        assert_eq!(r.height, 60.0);
    }

    #[test]
    fn urect_display() {
        let r = URectangle::build(100.0, 50.0);
        assert_eq!(format!("{}", r), "width=100 height=50");
    }

    // ── UEllipse tests ──

    #[test]
    fn uellipse_build() {
        let e = UEllipse::build(80.0, 60.0);
        assert_eq!(e.width, 80.0);
        assert_eq!(e.height, 60.0);
        assert_eq!(e.start, 0.0);
        assert_eq!(e.extend, 0.0);
    }

    #[test]
    fn uellipse_arc() {
        let e = UEllipse::arc(80.0, 60.0, 45.0, 90.0);
        assert_eq!(e.start, 45.0);
        assert_eq!(e.extend, 90.0);
    }

    #[test]
    fn uellipse_bigger() {
        let e = UEllipse::build(80.0, 60.0);
        let e2 = e.bigger(20.0);
        assert_eq!(e2.width, 100.0);
        assert_eq!(e2.height, 80.0);
    }

    #[test]
    fn uellipse_scale() {
        let e = UEllipse::build(80.0, 60.0);
        let e2 = e.scale(2.0);
        assert_eq!(e2.width, 160.0);
        assert_eq!(e2.height, 120.0);
    }

    #[test]
    fn uellipse_shadow() {
        let mut e = UEllipse::build(80.0, 60.0);
        e.set_delta_shadow(5.0);
        assert_eq!(e.delta_shadow(), 5.0);
    }

    #[test]
    fn uellipse_dimension() {
        let e = UEllipse::build(80.0, 60.0);
        let dim = e.dimension();
        assert_eq!(dim.width, 80.0);
        assert_eq!(dim.height, 60.0);
    }

    #[test]
    fn uellipse_point_at_angle() {
        let e = UEllipse::build(100.0, 100.0);
        // At angle 0: rightmost point
        let p = e.point_at_angle(0.0);
        assert!((p.x - 100.0).abs() < 1e-10);
        assert!((p.y - 50.0).abs() < 1e-10);
        // At angle PI: leftmost point
        let p2 = e.point_at_angle(std::f64::consts::PI);
        assert!(p2.x.abs() < 1e-10);
        assert!((p2.y - 50.0).abs() < 1e-10);
    }

    #[test]
    fn uellipse_starting_ending_x() {
        let e = UEllipse::build(100.0, 100.0);
        // At center y (50.0) the starting x should be ~0 and ending x should be ~100
        let sx = e.starting_x(50.0);
        let ex = e.ending_x(50.0);
        assert!(sx.abs() < 1e-10);
        assert!((ex - 100.0).abs() < 1e-10);
    }

    // ── ULine tests ──

    #[test]
    fn uline_new() {
        let l = ULine::new(30.0, 40.0);
        assert_eq!(l.dx, 30.0);
        assert_eq!(l.dy, 40.0);
    }

    #[test]
    fn uline_hline_vline() {
        let h = ULine::hline(100.0);
        assert_eq!(h.dx, 100.0);
        assert_eq!(h.dy, 0.0);
        let v = ULine::vline(50.0);
        assert_eq!(v.dx, 0.0);
        assert_eq!(v.dy, 50.0);
    }

    #[test]
    fn uline_from_points() {
        let p1 = XPoint2D::new(10.0, 20.0);
        let p2 = XPoint2D::new(40.0, 60.0);
        let l = ULine::from_points(p1, p2);
        assert_eq!(l.dx, 30.0);
        assert_eq!(l.dy, 40.0);
    }

    #[test]
    fn uline_length() {
        let l = ULine::new(3.0, 4.0);
        assert!((l.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn uline_rotate() {
        let l = ULine::hline(10.0);
        let r = l.rotate(std::f64::consts::FRAC_PI_2);
        assert!(r.dx.abs() < 1e-10);
        assert!((r.dy - 10.0).abs() < 1e-10);
    }

    #[test]
    fn uline_rotate_zero() {
        let l = ULine::new(3.0, 4.0);
        let r = l.rotate(0.0);
        assert_eq!(r.dx, 3.0);
        assert_eq!(r.dy, 4.0);
    }

    #[test]
    fn uline_shadow() {
        let mut l = ULine::hline(100.0);
        l.set_delta_shadow(2.0);
        assert_eq!(l.delta_shadow(), 2.0);
    }

    // ── UText tests ──

    #[test]
    fn utext_build() {
        let t = UText::build("Hello", "SansSerif", 14.0, false, false);
        assert_eq!(t.text, "Hello");
        assert_eq!(t.font_family, "SansSerif");
        assert_eq!(t.font_size, 14.0);
        assert!(!t.bold);
        assert!(!t.italic);
        assert_eq!(t.orientation, 0);
    }

    #[test]
    fn utext_with_orientation() {
        let t = UText::build("Rotated", "SansSerif", 12.0, true, false).with_orientation(90);
        assert_eq!(t.orientation, 90);
        assert!(t.bold);
    }

    #[test]
    fn utext_display() {
        let t = UText::build("Hello", "SansSerif", 14.0, false, false);
        assert_eq!(format!("{}", t), "UText[Hello]");
    }

    // ── DotPath tests ──

    #[test]
    fn dotpath_empty() {
        let dp = DotPath::new();
        assert!(dp.beziers.is_empty());
    }

    #[test]
    fn dotpath_add_curve_4() {
        let dp = DotPath::new();
        let dp2 = dp.add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(10.0, 20.0),
            XPoint2D::new(30.0, 20.0),
            XPoint2D::new(40.0, 0.0),
        );
        assert_eq!(dp2.beziers.len(), 1);
        assert_eq!(dp2.start_point(), XPoint2D::new(0.0, 0.0));
        assert_eq!(dp2.end_point(), XPoint2D::new(40.0, 0.0));
    }

    #[test]
    fn dotpath_add_curve_chained() {
        let dp = DotPath::new()
            .add_curve_4(
                XPoint2D::new(0.0, 0.0),
                XPoint2D::new(10.0, 20.0),
                XPoint2D::new(30.0, 20.0),
                XPoint2D::new(40.0, 0.0),
            )
            .add_curve(
                XPoint2D::new(50.0, -10.0),
                XPoint2D::new(70.0, -10.0),
                XPoint2D::new(80.0, 0.0),
            );
        assert_eq!(dp.beziers.len(), 2);
        assert_eq!(dp.end_point(), XPoint2D::new(80.0, 0.0));
    }

    #[test]
    fn dotpath_reverse() {
        let dp = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(10.0, 20.0),
            XPoint2D::new(30.0, 20.0),
            XPoint2D::new(40.0, 0.0),
        );
        let rev = dp.reverse();
        assert_eq!(rev.start_point(), XPoint2D::new(40.0, 0.0));
        assert_eq!(rev.end_point(), XPoint2D::new(0.0, 0.0));
    }

    #[test]
    fn dotpath_to_upath() {
        let dp = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(10.0, 20.0),
            XPoint2D::new(30.0, 20.0),
            XPoint2D::new(40.0, 0.0),
        );
        let up = dp.to_upath();
        assert_eq!(up.segments.len(), 2); // MoveTo + CubicTo
    }

    #[test]
    fn dotpath_move_delta() {
        let dp = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(10.0, 20.0),
            XPoint2D::new(30.0, 20.0),
            XPoint2D::new(40.0, 0.0),
        );
        let mut dp2 = dp.clone();
        dp2.move_delta(100.0, 50.0);
        assert_eq!(dp2.start_point(), XPoint2D::new(100.0, 50.0));
        assert_eq!(dp2.end_point(), XPoint2D::new(140.0, 50.0));
    }

    #[test]
    fn dotpath_angles() {
        // Horizontal line from (0,0) to (100,0)
        let dp = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(33.0, 0.0),
            XPoint2D::new(66.0, 0.0),
            XPoint2D::new(100.0, 0.0),
        );
        assert!(dp.start_angle().abs() < 1e-10);
        assert!(dp.end_angle().abs() < 1e-10);
    }

    #[test]
    fn dotpath_is_line() {
        // Straight line should be detected
        let dp = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(33.0, 0.0),
            XPoint2D::new(66.0, 0.0),
            XPoint2D::new(100.0, 0.0),
        );
        assert!(dp.is_line());

        // Curved line should not
        let dp2 = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(10.0, 50.0),
            XPoint2D::new(90.0, -50.0),
            XPoint2D::new(100.0, 0.0),
        );
        assert!(!dp2.is_line());
    }

    #[test]
    fn dotpath_concat_after() {
        let dp1 = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(10.0, 10.0),
            XPoint2D::new(20.0, 10.0),
            XPoint2D::new(30.0, 0.0),
        );
        let dp2 = DotPath::new().add_curve_4(
            XPoint2D::new(30.0, 0.0),
            XPoint2D::new(40.0, -10.0),
            XPoint2D::new(50.0, -10.0),
            XPoint2D::new(60.0, 0.0),
        );
        let combined = dp1.concat_after(&dp2);
        assert_eq!(combined.beziers.len(), 2);
        assert_eq!(combined.start_point(), XPoint2D::new(0.0, 0.0));
        assert_eq!(combined.end_point(), XPoint2D::new(60.0, 0.0));
    }

    #[test]
    fn dotpath_display() {
        let dp = DotPath::new().add_curve_4(
            XPoint2D::new(0.0, 0.0),
            XPoint2D::new(1.0, 2.0),
            XPoint2D::new(3.0, 4.0),
            XPoint2D::new(5.0, 6.0),
        );
        let s = format!("{}", dp);
        assert!(s.contains("(0,0)"));
        assert!(s.contains("(5,6)"));
    }

    // ── UEmpty tests ──

    #[test]
    fn uempty_new() {
        let e = UEmpty::new(10.0, 20.0);
        assert_eq!(e.width, 10.0);
        assert_eq!(e.height, 20.0);
    }

    #[test]
    fn uempty_from_dim() {
        let dim = XDimension2D::new(30.0, 40.0);
        let e = UEmpty::from_dim(dim);
        assert_eq!(e.width, 30.0);
        assert_eq!(e.height, 40.0);
    }

    // ── UComment tests ──

    #[test]
    fn ucomment_new() {
        let c = UComment::new("test comment");
        assert_eq!(c.comment, "test comment");
    }

    // ── UImage tests ──

    #[test]
    fn uimage_new() {
        let img = UImage::new(vec![0u8; 100], 10, 10);
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 10);
        assert_eq!(img.scale, 1.0);
    }

    #[test]
    fn uimage_scaled() {
        let img = UImage::new(vec![0u8; 100], 10, 10).with_scale(2.0);
        assert_eq!(img.scaled_width(), 20.0);
        assert_eq!(img.scaled_height(), 20.0);
    }

    // ── UImageSvg tests ──

    #[test]
    fn uimagesvg_get_data_from_viewbox() {
        let svg = r#"<svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        let img = UImageSvg::new(svg.to_string(), 1.0);
        assert_eq!(img.get_data("width"), Some(200));
        assert_eq!(img.get_data("height"), Some(100));
    }

    #[test]
    fn uimagesvg_get_data_from_attr() {
        let svg = r#"<svg width="300" height="150" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        let img = UImageSvg::new(svg.to_string(), 1.0);
        assert_eq!(img.get_data("width"), Some(300));
        assert_eq!(img.get_data("height"), Some(150));
    }

    #[test]
    fn uimagesvg_scaled() {
        let svg = r#"<svg width="100" height="50" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        let img = UImageSvg::new(svg.to_string(), 2.0);
        assert_eq!(img.width(), 200.0);
        assert_eq!(img.height(), 100.0);
    }

    #[test]
    fn uimagesvg_get_svg_raw() {
        let svg = r#"<?xml version="1.0"?><svg attr="val">content</svg>"#;
        let img = UImageSvg::new(svg.to_string(), 1.0);
        assert_eq!(img.get_svg(true), svg);
    }

    #[test]
    fn uimagesvg_get_svg_stripped() {
        let svg = r#"<?xml version="1.0"?><svg attr="val">content</svg>"#;
        let img = UImageSvg::new(svg.to_string(), 1.0);
        let stripped = img.get_svg(false);
        assert!(stripped.starts_with("<svg>"));
        assert!(stripped.contains("content"));
    }

    // ── UHorizontalLine tests ──

    #[test]
    fn uhorizontal_line_double() {
        let l = UHorizontalLine::infinite(1.0, 0.0, 0.0, '=');
        assert!(l.is_double());
        let l2 = UHorizontalLine::infinite(1.0, 0.0, 0.0, '-');
        assert!(!l2.is_double());
    }

    // ── UCenteredCharacter tests ──

    #[test]
    fn ucentered_character() {
        let c = UCenteredCharacter::new('A', "SansSerif", 14.0);
        assert_eq!(c.c, 'A');
        assert_eq!(c.font_family, "SansSerif");
        assert_eq!(c.font_size, 14.0);
    }
}
