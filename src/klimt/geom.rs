// klimt::geom - Geometry primitives
// Port of Java PlantUML's klimt.geom package

// ── XPoint2D ─────────────────────────────────────────────────────────

/// 2D point. Java: `klimt.geom.XPoint2D`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XPoint2D {
    pub x: f64,
    pub y: f64,
}

impl XPoint2D {
    pub fn new(x: f64, y: f64) -> Self { Self { x, y } }

    pub fn distance(&self, other: &XPoint2D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

// ── XDimension2D ─────────────────────────────────────────────────────

/// 2D dimension (width, height). Java: `klimt.geom.XDimension2D`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XDimension2D {
    pub width: f64,
    pub height: f64,
}

impl XDimension2D {
    pub fn new(width: f64, height: f64) -> Self { Self { width, height } }
    pub fn zero() -> Self { Self { width: 0.0, height: 0.0 } }

    pub fn delta(&self, dw: f64, dh: f64) -> Self {
        Self { width: self.width + dw, height: self.height + dh }
    }

    pub fn max(&self, other: &XDimension2D) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }

    pub fn merge_vertical(&self, other: &XDimension2D) -> Self {
        Self {
            width: self.width.max(other.width),
            height: self.height + other.height,
        }
    }

    pub fn merge_horizontal(&self, other: &XDimension2D) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height.max(other.height),
        }
    }
}

// ── XRectangle2D ─────────────────────────────────────────────────────

/// Axis-aligned rectangle. Java: `klimt.geom.XRectangle2D`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XRectangle2D {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl XRectangle2D {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn center_x(&self) -> f64 { self.x + self.width / 2.0 }
    pub fn center_y(&self) -> f64 { self.y + self.height / 2.0 }
    pub fn max_x(&self) -> f64 { self.x + self.width }
    pub fn max_y(&self) -> f64 { self.y + self.height }

    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.max_x() && py >= self.y && py <= self.max_y()
    }

    pub fn intersects(&self, other: &XRectangle2D) -> bool {
        self.x < other.max_x() && self.max_x() > other.x
            && self.y < other.max_y() && self.max_y() > other.y
    }
}

// ── XLine2D ──────────────────────────────────────────────────────────

/// Line segment. Java: `klimt.geom.XLine2D`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XLine2D {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl XLine2D {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn from_points(p1: XPoint2D, p2: XPoint2D) -> Self {
        Self { x1: p1.x, y1: p1.y, x2: p2.x, y2: p2.y }
    }

    pub fn middle(&self) -> XPoint2D {
        XPoint2D::new((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }

    pub fn angle(&self) -> f64 {
        (self.y2 - self.y1).atan2(self.x2 - self.x1)
    }

    pub fn length(&self) -> f64 {
        ((self.x2 - self.x1).powi(2) + (self.y2 - self.y1).powi(2)).sqrt()
    }

    /// Point-to-segment distance squared.
    pub fn pt_seg_dist_sq(&self, px: f64, py: f64) -> f64 {
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        let len_sq = dx * dx + dy * dy;
        if len_sq == 0.0 {
            return (px - self.x1).powi(2) + (py - self.y1).powi(2);
        }
        let t = ((px - self.x1) * dx + (py - self.y1) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);
        let proj_x = self.x1 + t * dx;
        let proj_y = self.y1 + t * dy;
        (px - proj_x).powi(2) + (py - proj_y).powi(2)
    }
}

// ── MinMax ───────────────────────────────────────────────────────────

/// Tracks bounding box from a series of points.
/// Java: `klimt.geom.MinMax`
#[derive(Debug, Clone, Copy)]
pub struct MinMax {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl MinMax {
    pub fn empty() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    pub fn add_rect(&mut self, r: &XRectangle2D) {
        self.add_point(r.x, r.y);
        self.add_point(r.max_x(), r.max_y());
    }

    pub fn width(&self) -> f64 { self.max_x - self.min_x }
    pub fn height(&self) -> f64 { self.max_y - self.min_y }

    pub fn is_empty(&self) -> bool { self.min_x > self.max_x }

    pub fn to_rect(&self) -> XRectangle2D {
        XRectangle2D::new(self.min_x, self.min_y, self.width(), self.height())
    }
}

// ── Alignment enums ──────────────────────────────────────────────────

/// Java: `klimt.geom.HorizontalAlignment`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HorizontalAlignment {
    Left,
    #[default]
    Center,
    Right,
}

/// Java: `klimt.geom.VerticalAlignment`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlignment {
    Top,
    #[default]
    Center,
    Bottom,
}

/// Java: `klimt.geom.Rankdir`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rankdir {
    #[default]
    TopToBottom,
    LeftToRight,
    BottomToTop,
    RightToLeft,
}

// ── USegment ─────────────────────────────────────────────────────────

/// Path segment type. Java: `klimt.geom.USegmentType`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum USegmentType {
    MoveTo,
    LineTo,
    CubicTo,
    ArcTo,
    Close,
}

/// A single segment in a UPath. Java: `klimt.geom.USegment`
#[derive(Debug, Clone)]
pub struct USegment {
    pub kind: USegmentType,
    pub coords: Vec<f64>,
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_distance() {
        let a = XPoint2D::new(0.0, 0.0);
        let b = XPoint2D::new(3.0, 4.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn dimension_merge() {
        let a = XDimension2D::new(100.0, 50.0);
        let b = XDimension2D::new(80.0, 30.0);
        let v = a.merge_vertical(&b);
        assert_eq!(v.width, 100.0);
        assert_eq!(v.height, 80.0);
        let h = a.merge_horizontal(&b);
        assert_eq!(h.width, 180.0);
        assert_eq!(h.height, 50.0);
    }

    #[test]
    fn rect_contains() {
        let r = XRectangle2D::new(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(50.0, 40.0));
        assert!(!r.contains(5.0, 40.0));
    }

    #[test]
    fn line_middle_and_angle() {
        let l = XLine2D::new(0.0, 0.0, 10.0, 0.0);
        let m = l.middle();
        assert_eq!(m.x, 5.0);
        assert_eq!(m.y, 0.0);
        assert!((l.angle() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn minmax_tracking() {
        let mut mm = MinMax::empty();
        assert!(mm.is_empty());
        mm.add_point(10.0, 20.0);
        mm.add_point(50.0, 5.0);
        assert_eq!(mm.min_x, 10.0);
        assert_eq!(mm.max_y, 20.0);
        assert_eq!(mm.width(), 40.0);
        assert_eq!(mm.height(), 15.0);
    }
}
