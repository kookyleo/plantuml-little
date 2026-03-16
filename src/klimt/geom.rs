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

    pub fn distance_sq(&self, other: &XPoint2D) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    pub fn distance_xy(&self, px: f64, py: f64) -> f64 {
        ((self.x - px).powi(2) + (self.y - py).powi(2)).sqrt()
    }

    /// Static two-point distance. Java: `XPoint2D.distance(x1, y1, x2, y2)`
    pub fn distance_between(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let dx = x1 - x2;
        let dy = y1 - y2;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn moved(&self, dx: f64, dy: f64) -> Self {
        Self { x: self.x + dx, y: self.y + dy }
    }

    pub fn move_point(&self, delta: &XPoint2D) -> Self {
        Self { x: self.x + delta.x, y: self.y + delta.y }
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

    pub fn min_x(&self) -> f64 { self.x }
    pub fn min_y(&self) -> f64 { self.y }

    pub fn intersects(&self, other: &XRectangle2D) -> bool {
        self.width > 0.0 && self.height > 0.0
            && other.width > 0.0 && other.height > 0.0
            && other.x < self.x + self.width && other.x + other.width > self.x
            && other.y < self.y + self.height && other.y + other.height > self.y
    }

    /// Line-rectangle intersection: find where `line` crosses the rectangle boundary.
    pub fn intersect_line(&self, line: &XLine2D) -> Option<XPoint2D> {
        let a = XPoint2D::new(self.x, self.y);
        let b = XPoint2D::new(self.x + self.width, self.y);
        let c = XPoint2D::new(self.x + self.width, self.y + self.height);
        let d = XPoint2D::new(self.x, self.y + self.height);
        let edges = [
            XLine2D::from_points(a, b),
            XLine2D::from_points(b, c),
            XLine2D::from_points(c, d),
            XLine2D::from_points(d, a),
        ];
        for edge in &edges {
            if let Some(pt) = line.intersect(edge) {
                return Some(pt);
            }
        }
        None
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
        Self::pt_seg_dist_sq_static(self.x1, self.y1, self.x2, self.y2, px, py)
    }

    /// Static point-to-segment distance squared (matches Java `XLine2D.ptSegDistSq`).
    pub fn pt_seg_dist_sq_static(
        x1: f64, y1: f64, x2: f64, y2: f64, px: f64, py: f64,
    ) -> f64 {
        let rx2 = x2 - x1;
        let ry2 = y2 - y1;
        let mut rpx = px - x1;
        let mut rpy = py - y1;
        let mut dotprod = rpx * rx2 + rpy * ry2;
        let proj_len_sq;
        if dotprod <= 0.0 {
            proj_len_sq = 0.0;
        } else {
            rpx = rx2 - rpx;
            rpy = ry2 - rpy;
            dotprod = rpx * rx2 + rpy * ry2;
            if dotprod <= 0.0 {
                proj_len_sq = 0.0;
            } else {
                proj_len_sq = dotprod * dotprod / (rx2 * rx2 + ry2 * ry2);
            }
        }
        let mut len_sq = rpx * rpx + rpy * rpy - proj_len_sq;
        if len_sq < 0.0 {
            len_sq = 0.0;
        }
        len_sq
    }

    pub fn p1(&self) -> XPoint2D { XPoint2D::new(self.x1, self.y1) }
    pub fn p2(&self) -> XPoint2D { XPoint2D::new(self.x2, self.y2) }

    pub fn with_point1(&self, p: XPoint2D) -> Self {
        Self { x1: p.x, y1: p.y, x2: self.x2, y2: self.y2 }
    }

    pub fn with_point2(&self, p: XPoint2D) -> Self {
        Self { x1: self.x1, y1: self.y1, x2: p.x, y2: p.y }
    }

    /// Segment-segment intersection (returns `None` if no intersection).
    pub fn intersect(&self, other: &XLine2D) -> Option<XPoint2D> {
        let s1x = self.x2 - self.x1;
        let s1y = self.y2 - self.y1;
        let s2x = other.x2 - other.x1;
        let s2y = other.y2 - other.y1;
        let denom = -s2x * s1y + s1x * s2y;
        if denom.abs() < 1e-15 {
            return None;
        }
        let s = (-s1y * (self.x1 - other.x1) + s1x * (self.y1 - other.y1)) / denom;
        let t = (s2x * (self.y1 - other.y1) - s2y * (self.x1 - other.x1)) / denom;
        if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
            Some(XPoint2D::new(self.x1 + t * s1x, self.y1 + t * s1y))
        } else {
            None
        }
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

// ── Side ────────────────────────────────────────────────────────────

/// Cardinal direction. Java: `klimt.geom.Side`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    North,
    East,
    South,
    West,
}

// ── XCubicCurve2D ───────────────────────────────────────────────────

/// Cubic Bezier curve. Java: `klimt.geom.XCubicCurve2D`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XCubicCurve2D {
    pub x1: f64,
    pub y1: f64,
    pub ctrlx1: f64,
    pub ctrly1: f64,
    pub ctrlx2: f64,
    pub ctrly2: f64,
    pub x2: f64,
    pub y2: f64,
}

impl XCubicCurve2D {
    pub fn new(
        x1: f64, y1: f64,
        ctrlx1: f64, ctrly1: f64,
        ctrlx2: f64, ctrly2: f64,
        x2: f64, y2: f64,
    ) -> Self {
        Self { x1, y1, ctrlx1, ctrly1, ctrlx2, ctrly2, x2, y2 }
    }

    pub fn none() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    pub fn p1(&self) -> XPoint2D { XPoint2D::new(self.x1, self.y1) }
    pub fn p2(&self) -> XPoint2D { XPoint2D::new(self.x2, self.y2) }
    pub fn ctrl_p1(&self) -> XPoint2D { XPoint2D::new(self.ctrlx1, self.ctrly1) }
    pub fn ctrl_p2(&self) -> XPoint2D { XPoint2D::new(self.ctrlx2, self.ctrly2) }

    pub fn set_curve(
        &mut self,
        x1: f64, y1: f64,
        ctrlx1: f64, ctrly1: f64,
        ctrlx2: f64, ctrly2: f64,
        x2: f64, y2: f64,
    ) {
        self.x1 = x1;
        self.y1 = y1;
        self.ctrlx1 = ctrlx1;
        self.ctrly1 = ctrly1;
        self.ctrlx2 = ctrlx2;
        self.ctrly2 = ctrly2;
        self.x2 = x2;
        self.y2 = y2;
    }

    pub fn set_curve_from(&mut self, other: &XCubicCurve2D) {
        // Note: Java source copies ctrlx2/ctrly2 for x2/y2 (matching original)
        self.set_curve(
            other.x1, other.y1,
            other.ctrlx1, other.ctrly1,
            other.ctrlx2, other.ctrly2,
            other.ctrlx2, other.ctrly2,
        );
    }

    /// Chord length (straight-line distance from P1 to P2).
    pub fn length(&self) -> f64 {
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        (dx * dx + dy * dy).sqrt()
    }

    /// De Casteljau subdivision at t = 0.5.
    pub fn subdivide(&self, left: &mut XCubicCurve2D, right: &mut XCubicCurve2D) {
        let x1 = self.x1;
        let y1 = self.y1;
        let x2 = self.x2;
        let y2 = self.y2;
        let mut cx1 = self.ctrlx1;
        let mut cy1 = self.ctrly1;
        let mut cx2 = self.ctrlx2;
        let mut cy2 = self.ctrly2;

        let mut center_x = (cx1 + cx2) / 2.0;
        let mut center_y = (cy1 + cy2) / 2.0;
        cx1 = (x1 + cx1) / 2.0;
        cy1 = (y1 + cy1) / 2.0;
        cx2 = (x2 + cx2) / 2.0;
        cy2 = (y2 + cy2) / 2.0;
        let ctrlx12 = (cx1 + center_x) / 2.0;
        let ctrly12 = (cy1 + center_y) / 2.0;
        let ctrlx21 = (cx2 + center_x) / 2.0;
        let ctrly21 = (cy2 + center_y) / 2.0;
        center_x = (ctrlx12 + ctrlx21) / 2.0;
        center_y = (ctrly12 + ctrly21) / 2.0;

        left.set_curve(x1, y1, cx1, cy1, ctrlx12, ctrly12, center_x, center_y);
        right.set_curve(center_x, center_y, ctrlx21, ctrly21, cx2, cy2, x2, y2);
    }

    /// Flatness squared: max distance-squared of control points from the chord.
    pub fn flatness_sq(&self) -> f64 {
        let d1 = XLine2D::pt_seg_dist_sq_static(
            self.x1, self.y1, self.x2, self.y2, self.ctrlx1, self.ctrly1,
        );
        let d2 = XLine2D::pt_seg_dist_sq_static(
            self.x1, self.y1, self.x2, self.y2, self.ctrlx2, self.ctrly2,
        );
        d1.max(d2)
    }

    pub fn flatness(&self) -> f64 {
        self.flatness_sq().sqrt()
    }
}

// ── BezierUtils ─────────────────────────────────────────────────────

/// Bezier curve utilities. Java: `klimt.geom.BezierUtils`
pub struct BezierUtils;

impl BezierUtils {
    /// Angle from ctrl_p2 to p2 (or p1 to p2 if they coincide).
    pub fn ending_angle(curve: &XCubicCurve2D) -> f64 {
        let cp2 = curve.ctrl_p2();
        let p2 = curve.p2();
        if cp2 == p2 {
            Self::angle(curve.p1(), p2)
        } else {
            Self::angle(cp2, p2)
        }
    }

    /// Angle from p1 to ctrl_p1 (or p1 to p2 if they coincide).
    pub fn starting_angle(curve: &XCubicCurve2D) -> f64 {
        let p1 = curve.p1();
        let cp1 = curve.ctrl_p1();
        if p1 == cp1 {
            Self::angle(p1, curve.p2())
        } else {
            Self::angle(p1, cp1)
        }
    }

    /// Angle in radians from `p1` to `p2`. Panics if points are equal.
    pub fn angle(p1: XPoint2D, p2: XPoint2D) -> f64 {
        assert!(p1 != p2, "BezierUtils::angle: points must differ");
        (p2.y - p1.y).atan2(p2.x - p1.x)
    }

    /// Distance between start and end of a cubic curve.
    pub fn curve_dist(curve: &XCubicCurve2D) -> f64 {
        XPoint2D::distance_between(curve.x1, curve.y1, curve.x2, curve.y2)
    }

    /// Distance between start and end of a line.
    pub fn line_dist(line: &XLine2D) -> f64 {
        XPoint2D::distance_between(line.x1, line.y1, line.x2, line.y2)
    }

    /// Midpoint of a line segment.
    pub fn line_middle(seg: &XLine2D) -> XPoint2D {
        XPoint2D::new((seg.x1 + seg.x2) / 2.0, (seg.y1 + seg.y2) / 2.0)
    }

    /// Midpoint of two points.
    pub fn point_middle(p1: XPoint2D, p2: XPoint2D) -> XPoint2D {
        XPoint2D::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0)
    }

    /// Binary-search intersection of a line segment with a rectangle border.
    /// One endpoint must be inside the rectangle, the other outside.
    pub fn line_rect_intersect(line: &XLine2D, shape: &XRectangle2D) -> XPoint2D {
        let contains1 = shape.contains(line.x1, line.y1);
        let contains2 = shape.contains(line.x2, line.y2);
        assert!(
            contains1 != contains2,
            "BezierUtils::line_rect_intersect: exactly one endpoint must be inside"
        );
        let mut copy = *line;
        loop {
            let m = copy.middle();
            let contains_mid = shape.contains(m.x, m.y);
            if contains_mid == contains1 {
                copy = copy.with_point1(m);
            } else {
                copy = copy.with_point2(m);
            }
            if Self::line_dist(&copy) < 0.1 {
                return if contains1 { copy.p2() } else { copy.p1() };
            }
        }
    }

    /// Weighted subdivision of a cubic Bezier (used by `shorten`).
    pub fn subdivide_weighted(
        src: &XCubicCurve2D,
        left: &mut XCubicCurve2D,
        right: &mut XCubicCurve2D,
        coef: f64,
    ) {
        let coef1 = coef;
        let coef2 = 1.0 - coef;
        let center_xa = src.ctrlx1 * coef1 + src.ctrlx2 * coef2;
        let center_ya = src.ctrly1 * coef1 + src.ctrly2 * coef2;

        let x1 = src.x1;
        let y1 = src.y1;
        let x2 = src.x2;
        let y2 = src.y2;
        let cx1 = x1 * coef1 + src.ctrlx1 * coef1;
        let cy1 = y1 * coef1 + src.ctrly1 * coef1;
        let cx2 = x2 * coef1 + src.ctrlx2 * coef1;
        let cy2 = y2 * coef1 + src.ctrly2 * coef1;

        let cx12 = cx1 * coef1 + center_xa * coef1;
        let cy12 = cy1 * coef1 + center_ya * coef1;
        let cx21 = cx2 * coef1 + center_xa * coef1;
        let cy21 = cy2 * coef1 + center_ya * coef1;
        let cxb = cx12 * coef1 + cx21 * coef1;
        let cyb = cy12 * coef1 + cy21 * coef1;
        left.set_curve(x1, y1, cx1, cy1, cx12, cy12, cxb, cyb);
        right.set_curve(cxb, cyb, cx21, cy21, cx2, cy2, x2, y2);
    }

    fn is_cutting(bez: &XCubicCurve2D, shape: &XRectangle2D) -> bool {
        let c1 = shape.contains(bez.x1, bez.y1);
        let c2 = shape.contains(bez.x2, bez.y2);
        c1 != c2
    }

    /// Shorten a cubic Bezier so that it only covers the cutting portion
    /// through a rectangle border.
    pub fn shorten(bez: &mut XCubicCurve2D, shape: &XRectangle2D) {
        let c1 = shape.contains(bez.x1, bez.y1);
        let c2 = shape.contains(bez.x2, bez.y2);
        assert!(c1 != c2, "BezierUtils::shorten: exactly one endpoint must be inside");

        if !c1 {
            // Reverse so that p1 is inside
            let rev = XCubicCurve2D::new(
                bez.x2, bez.y2, bez.ctrlx2, bez.ctrly2,
                bez.ctrlx1, bez.ctrly1, bez.x1, bez.y1,
            );
            *bez = rev;
        }

        let mut left = XCubicCurve2D::none();
        let mut right = XCubicCurve2D::none();
        Self::subdivide_weighted(bez, &mut left, &mut right, 0.5);

        let cut_left = Self::is_cutting(&left, shape);
        if cut_left {
            bez.set_curve(
                left.x1, left.y1, left.ctrlx1, left.ctrly1,
                left.ctrlx2, left.ctrly2, left.x2, left.y2,
            );
        } else {
            bez.set_curve(
                right.x1, right.y1, right.ctrlx1, right.ctrly1,
                right.ctrlx2, right.ctrly2, right.x2, right.y2,
            );
        }
    }
}

// ── MinMaxMutable ───────────────────────────────────────────────────

/// Mutable bounding-box tracker. Java: `klimt.geom.MinMaxMutable`
#[derive(Debug, Clone, Copy)]
pub struct MinMaxMutable {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl MinMaxMutable {
    pub fn empty(init_to_zero: bool) -> Self {
        if init_to_zero {
            Self { min_x: 0.0, min_y: 0.0, max_x: 0.0, max_y: 0.0 }
        } else {
            Self {
                min_x: f64::MAX,
                min_y: f64::MAX,
                max_x: -f64::MAX,
                max_y: -f64::MAX,
            }
        }
    }

    pub fn from_max(max_x: f64, max_y: f64) -> Self {
        let mut result = Self::empty(true);
        result.add_point(max_x, max_y);
        result
    }

    pub fn is_infinity(&self) -> bool {
        self.min_x == f64::MAX
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        debug_assert!(!x.is_nan(), "MinMaxMutable::add_point: x is NaN");
        debug_assert!(!y.is_nan(), "MinMaxMutable::add_point: y is NaN");
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
    }

    pub fn add_point_pt(&mut self, pt: XPoint2D) {
        self.add_point(pt.x, pt.y);
    }

    pub fn dimension(&self) -> XDimension2D {
        XDimension2D::new(self.max_x - self.min_x, self.max_y - self.min_y)
    }

    pub fn reset(&mut self) {
        self.min_x = 0.0;
        self.min_y = 0.0;
        self.max_x = 0.0;
        self.max_y = 0.0;
    }
}

impl std::fmt::Display for MinMaxMutable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "X={} to {} and Y={} to {}", self.min_x, self.max_x, self.min_y, self.max_y)
    }
}

// ── CoordinateChange ────────────────────────────────────────────────

/// Coordinate transform from one line segment to a local frame.
/// Java: `klimt.geom.CoordinateChange`
#[derive(Debug, Clone, Copy)]
pub struct CoordinateChange {
    x1: f64,
    y1: f64,
    vect_u_x: f64,
    vect_u_y: f64,
    vect_v_x: f64,
    vect_v_y: f64,
    len: f64,
}

impl CoordinateChange {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let len = XPoint2D::distance_between(x1, y1, x2, y2);
        assert!(len != 0.0, "CoordinateChange: zero-length segment");
        let ux = (x2 - x1) / len;
        let uy = (y2 - y1) / len;
        Self {
            x1,
            y1,
            vect_u_x: ux,
            vect_u_y: uy,
            vect_v_x: -uy,
            vect_v_y: ux,
            len,
        }
    }

    pub fn from_points(p1: XPoint2D, p2: XPoint2D) -> Self {
        Self::new(p1.x, p1.y, p2.x, p2.y)
    }

    /// Map local coordinates `(a, b)` to world coordinates.
    pub fn true_coordinate(&self, a: f64, b: f64) -> XPoint2D {
        let x = a * self.vect_u_x + b * self.vect_v_x;
        let y = a * self.vect_u_y + b * self.vect_v_y;
        XPoint2D::new(self.x1 + x, self.y1 + y)
    }

    pub fn length(&self) -> f64 { self.len }
}

// ── PointAndAngle ───────────────────────────────────────────────────

/// Point with an associated angle. Java: `klimt.geom.PointAndAngle`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointAndAngle {
    pub pt: XPoint2D,
    pub angle: f64,
}

impl PointAndAngle {
    pub fn new(pt: XPoint2D, angle: f64) -> Self { Self { pt, angle } }
    pub fn x(&self) -> f64 { self.pt.x }
    pub fn y(&self) -> f64 { self.pt.y }
}

// ── PointDirected ───────────────────────────────────────────────────

/// Point with a direction (angle). Java: `klimt.geom.PointDirected`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointDirected {
    x: f64,
    y: f64,
    angle: f64,
}

impl PointDirected {
    pub fn new(p: XPoint2D, angle: f64) -> Self {
        Self { x: p.x, y: p.y, angle }
    }

    pub fn point(&self) -> XPoint2D { XPoint2D::new(self.x, self.y) }
    pub fn angle(&self) -> f64 { self.angle }
}

// ── MinFinder ───────────────────────────────────────────────────────

/// Tracks the minimum x and y independently. Java: `klimt.geom.MinFinder`
#[derive(Debug, Clone, Copy)]
pub struct MinFinder {
    min_x: f64,
    min_y: f64,
}

impl MinFinder {
    pub fn new() -> Self {
        Self { min_x: f64::MAX, min_y: f64::MAX }
    }

    pub fn manage(&mut self, x: f64, y: f64) {
        if x < self.min_x { self.min_x = x; }
        if y < self.min_y { self.min_y = y; }
    }

    pub fn manage_point(&mut self, p: XPoint2D) {
        self.manage(p.x, p.y);
    }

    pub fn manage_other(&mut self, other: &MinFinder) {
        self.manage(other.min_x, other.min_y);
    }

    pub fn min_x(&self) -> f64 { self.min_x }
    pub fn min_y(&self) -> f64 { self.min_y }
}

impl Default for MinFinder {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for MinFinder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "minX={} minY={}", self.min_x, self.min_y)
    }
}

// ── RectangleArea ───────────────────────────────────────────────────

/// Rectangle defined by min/max corners. Java: `klimt.geom.RectangleArea`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectangleArea {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl RectangleArea {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub fn from_points(p1: XPoint2D, p2: XPoint2D) -> Self {
        Self {
            min_x: p1.x.min(p2.x),
            min_y: p1.y.min(p2.y),
            max_x: p1.x.max(p2.x),
            max_y: p1.y.max(p2.y),
        }
    }

    pub fn moved(&self, dx: f64, dy: f64) -> Self {
        Self {
            min_x: self.min_x + dx,
            min_y: self.min_y + dy,
            max_x: self.max_x + dx,
            max_y: self.max_y + dy,
        }
    }

    pub fn width(&self) -> f64 { self.max_x - self.min_x }
    pub fn height(&self) -> f64 { self.max_y - self.min_y }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x < self.max_x && y >= self.min_y && y < self.max_y
    }

    pub fn contains_point(&self, p: XPoint2D) -> bool {
        self.contains(p.x, p.y)
    }

    pub fn merge(&self, other: &RectangleArea) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    pub fn merge_point(&self, p: XPoint2D) -> Self {
        Self {
            min_x: self.min_x.min(p.x),
            min_y: self.min_y.min(p.y),
            max_x: self.max_x.max(p.x),
            max_y: self.max_y.max(p.y),
        }
    }

    pub fn center(&self) -> XPoint2D {
        XPoint2D::new((self.min_x + self.max_x) / 2.0, (self.min_y + self.max_y) / 2.0)
    }

    pub fn dimension(&self) -> XDimension2D {
        XDimension2D::new(self.width(), self.height())
    }

    pub fn with_min_x(&self, v: f64) -> Self { Self { min_x: v, ..*self } }
    pub fn with_max_x(&self, v: f64) -> Self { Self { max_x: v, ..*self } }
    pub fn with_min_y(&self, v: f64) -> Self { Self { min_y: v, ..*self } }
    pub fn with_max_y(&self, v: f64) -> Self { Self { max_y: v, ..*self } }

    pub fn add_min_x(&self, d: f64) -> Self { Self { min_x: self.min_x + d, ..*self } }
    pub fn add_min_y(&self, d: f64) -> Self { Self { min_y: self.min_y + d, ..*self } }
    pub fn add_max_x(&self, d: f64) -> Self { Self { max_x: self.max_x + d, ..*self } }
    pub fn add_max_y(&self, d: f64) -> Self { Self { max_y: self.max_y + d, ..*self } }

    pub fn delta(&self, m1: f64, m2: f64) -> Self {
        Self { max_x: self.max_x + m1, max_y: self.max_y + m2, ..*self }
    }

    /// Find intersection of a cubic Bezier curve with this rectangle border.
    /// Returns `None` if both endpoints are on the same side.
    pub fn intersection_bezier(&self, bez: &XCubicCurve2D) -> Option<PointDirected> {
        if self.contains(bez.x1, bez.y1) == self.contains(bez.x2, bez.y2) {
            return None;
        }
        let dist = bez.p1().distance(&bez.p2());
        if dist < 2.0 {
            let angle = BezierUtils::starting_angle(bez);
            return Some(PointDirected::new(bez.p1(), angle));
        }
        let mut left = XCubicCurve2D::none();
        let mut right = XCubicCurve2D::none();
        bez.subdivide(&mut left, &mut right);
        if let Some(r) = self.intersection_bezier(&left) {
            return Some(r);
        }
        self.intersection_bezier(&right)
    }

    /// Find the closest side to a point.
    pub fn closest_side(&self, pt: XPoint2D) -> Option<Side> {
        let dn = (self.min_y - pt.y).abs();
        let ds = (self.max_y - pt.y).abs();
        let dw = (self.min_x - pt.x).abs();
        let de = (self.max_x - pt.x).abs();

        if dn <= dw && dn <= de && dn <= ds { return Some(Side::North); }
        if ds <= dn && ds <= dw && ds <= de { return Some(Side::South); }
        if de <= dn && de <= dw && de <= ds { return Some(Side::East); }
        if dw <= dn && dw <= de && dw <= ds { return Some(Side::West); }
        None
    }
}

impl std::fmt::Display for RectangleArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "minX={} maxX={} minY={} maxY={}", self.min_x, self.max_x, self.min_y, self.max_y)
    }
}

// ── GraphicPosition ─────────────────────────────────────────────────

/// Java: `klimt.geom.GraphicPosition`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicPosition {
    Bottom,
    BackgroundCornerBottomRight,
    BackgroundCornerTopRight,
}

// ── VerticalPosition ────────────────────────────────────────────────

/// Java: `klimt.geom.VerticalPosition`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalPosition {
    Top,
    Bottom,
}

// ── ImgValign ───────────────────────────────────────────────────────

/// Java: `klimt.geom.ImgValign`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImgValign {
    Top,
    Bottom,
    Middle,
}

// ── PathIterator constants ──────────────────────────────────────────

/// Path iteration segment type constants. Java: `klimt.geom.PathIterator`
pub mod path_iterator {
    pub const WIND_EVEN_ODD: i32 = 0;
    pub const WIND_NON_ZERO: i32 = 1;
    pub const SEG_MOVETO: i32 = 0;
    pub const SEG_LINETO: i32 = 1;
    pub const SEG_QUADTO: i32 = 2;
    pub const SEG_CUBICTO: i32 = 3;
    pub const SEG_CLOSE: i32 = 4;
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

    // ── XPoint2D new methods ─────────────────────────────────────

    #[test]
    fn point_distance_between() {
        let d = XPoint2D::distance_between(0.0, 0.0, 3.0, 4.0);
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn point_moved() {
        let p = XPoint2D::new(1.0, 2.0).moved(3.0, 4.0);
        assert_eq!(p, XPoint2D::new(4.0, 6.0));
    }

    #[test]
    fn point_distance_sq() {
        let a = XPoint2D::new(0.0, 0.0);
        let b = XPoint2D::new(3.0, 4.0);
        assert!((a.distance_sq(&b) - 25.0).abs() < 1e-10);
    }

    // ── XLine2D new methods ──────────────────────────────────────

    #[test]
    fn line_pt_seg_dist_sq_static() {
        // Point on the line
        let d = XLine2D::pt_seg_dist_sq_static(0.0, 0.0, 10.0, 0.0, 5.0, 0.0);
        assert!(d.abs() < 1e-10);
        // Point perpendicular
        let d = XLine2D::pt_seg_dist_sq_static(0.0, 0.0, 10.0, 0.0, 5.0, 3.0);
        assert!((d - 9.0).abs() < 1e-10);
    }

    #[test]
    fn line_with_points() {
        let l = XLine2D::new(0.0, 0.0, 10.0, 10.0);
        let l2 = l.with_point1(XPoint2D::new(5.0, 5.0));
        assert_eq!(l2.x1, 5.0);
        assert_eq!(l2.y1, 5.0);
        assert_eq!(l2.x2, 10.0);
        let l3 = l.with_point2(XPoint2D::new(20.0, 20.0));
        assert_eq!(l3.x2, 20.0);
    }

    #[test]
    fn line_intersect() {
        let l1 = XLine2D::new(0.0, 0.0, 10.0, 10.0);
        let l2 = XLine2D::new(0.0, 10.0, 10.0, 0.0);
        let pt = l1.intersect(&l2).unwrap();
        assert!((pt.x - 5.0).abs() < 1e-10);
        assert!((pt.y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn line_no_intersect() {
        let l1 = XLine2D::new(0.0, 0.0, 5.0, 0.0);
        let l2 = XLine2D::new(0.0, 1.0, 5.0, 1.0);
        assert!(l1.intersect(&l2).is_none());
    }

    // ── XRectangle2D new methods ─────────────────────────────────

    #[test]
    fn rect_intersects_java_style() {
        // Zero-width rect should not intersect
        let r1 = XRectangle2D::new(0.0, 0.0, 0.0, 10.0);
        let r2 = XRectangle2D::new(-5.0, -5.0, 10.0, 20.0);
        assert!(!r1.intersects(&r2));
    }

    #[test]
    fn rect_intersect_line() {
        let r = XRectangle2D::new(0.0, 0.0, 10.0, 10.0);
        let l = XLine2D::new(-5.0, 5.0, 15.0, 5.0);
        let pt = r.intersect_line(&l).unwrap();
        assert!((pt.y - 5.0).abs() < 1e-10);
    }

    // ── Side ─────────────────────────────────────────────────────

    #[test]
    fn side_values() {
        assert_ne!(Side::North, Side::South);
        assert_ne!(Side::East, Side::West);
    }

    // ── XCubicCurve2D ────────────────────────────────────────────

    #[test]
    fn cubic_none() {
        let c = XCubicCurve2D::none();
        assert_eq!(c.x1, 0.0);
        assert_eq!(c.x2, 0.0);
    }

    #[test]
    fn cubic_length() {
        let c = XCubicCurve2D::new(0.0, 0.0, 1.0, 1.0, 2.0, 1.0, 3.0, 0.0);
        assert!((c.length() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn cubic_points() {
        let c = XCubicCurve2D::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        assert_eq!(c.p1(), XPoint2D::new(1.0, 2.0));
        assert_eq!(c.p2(), XPoint2D::new(7.0, 8.0));
        assert_eq!(c.ctrl_p1(), XPoint2D::new(3.0, 4.0));
        assert_eq!(c.ctrl_p2(), XPoint2D::new(5.0, 6.0));
    }

    #[test]
    fn cubic_subdivide() {
        let c = XCubicCurve2D::new(0.0, 0.0, 10.0, 20.0, 30.0, 20.0, 40.0, 0.0);
        let mut left = XCubicCurve2D::none();
        let mut right = XCubicCurve2D::none();
        c.subdivide(&mut left, &mut right);
        // Left starts at P1
        assert_eq!(left.x1, 0.0);
        assert_eq!(left.y1, 0.0);
        // Right ends at P2
        assert_eq!(right.x2, 40.0);
        assert_eq!(right.y2, 0.0);
        // They share the midpoint
        assert!((left.x2 - right.x1).abs() < 1e-10);
        assert!((left.y2 - right.y1).abs() < 1e-10);
    }

    #[test]
    fn cubic_flatness() {
        // A perfectly straight "curve" should have zero flatness
        let c = XCubicCurve2D::new(0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0);
        assert!(c.flatness() < 1e-10);
        // A curve with control points far from chord should have high flatness
        let c2 = XCubicCurve2D::new(0.0, 0.0, 0.0, 100.0, 10.0, 100.0, 10.0, 0.0);
        assert!(c2.flatness() > 50.0);
    }

    // ── BezierUtils ──────────────────────────────────────────────

    #[test]
    fn bezier_angle() {
        let p1 = XPoint2D::new(0.0, 0.0);
        let p2 = XPoint2D::new(1.0, 0.0);
        assert!((BezierUtils::angle(p1, p2) - 0.0).abs() < 1e-10);
        let p3 = XPoint2D::new(0.0, 1.0);
        assert!((BezierUtils::angle(p1, p3) - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    #[should_panic]
    fn bezier_angle_same_point() {
        let p = XPoint2D::new(1.0, 2.0);
        BezierUtils::angle(p, p);
    }

    #[test]
    fn bezier_starting_ending_angle() {
        let c = XCubicCurve2D::new(0.0, 0.0, 10.0, 0.0, 20.0, 0.0, 30.0, 0.0);
        // Horizontal curve: both angles should be 0
        assert!(BezierUtils::starting_angle(&c).abs() < 1e-10);
        assert!(BezierUtils::ending_angle(&c).abs() < 1e-10);
    }

    #[test]
    fn bezier_starting_angle_ctrl_equals_p1() {
        // When ctrl_p1 == p1, should use p1->p2 angle
        let c = XCubicCurve2D::new(0.0, 0.0, 0.0, 0.0, 5.0, 5.0, 10.0, 10.0);
        let angle = BezierUtils::starting_angle(&c);
        assert!((angle - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn bezier_point_middle() {
        let p1 = XPoint2D::new(0.0, 0.0);
        let p2 = XPoint2D::new(10.0, 20.0);
        let m = BezierUtils::point_middle(p1, p2);
        assert_eq!(m, XPoint2D::new(5.0, 10.0));
    }

    #[test]
    fn bezier_line_rect_intersect() {
        let rect = XRectangle2D::new(10.0, 10.0, 20.0, 20.0);
        // Line from inside to outside
        let line = XLine2D::new(20.0, 20.0, 50.0, 20.0);
        let pt = BezierUtils::line_rect_intersect(&line, &rect);
        // Should be near x=30 (right edge)
        assert!((pt.x - 30.0).abs() < 0.2);
        assert!((pt.y - 20.0).abs() < 0.2);
    }

    // ── MinMaxMutable ────────────────────────────────────────────

    #[test]
    fn minmax_mutable_empty() {
        let mm = MinMaxMutable::empty(false);
        assert!(mm.is_infinity());
    }

    #[test]
    fn minmax_mutable_zero() {
        let mm = MinMaxMutable::empty(true);
        assert!(!mm.is_infinity());
        assert_eq!(mm.min_x, 0.0);
        assert_eq!(mm.max_x, 0.0);
    }

    #[test]
    fn minmax_mutable_add_points() {
        let mut mm = MinMaxMutable::empty(false);
        mm.add_point(5.0, 10.0);
        mm.add_point(15.0, 3.0);
        assert_eq!(mm.min_x, 5.0);
        assert_eq!(mm.max_x, 15.0);
        assert_eq!(mm.min_y, 3.0);
        assert_eq!(mm.max_y, 10.0);
        let dim = mm.dimension();
        assert_eq!(dim.width, 10.0);
        assert_eq!(dim.height, 7.0);
    }

    #[test]
    fn minmax_mutable_from_max() {
        let mm = MinMaxMutable::from_max(100.0, 200.0);
        assert_eq!(mm.min_x, 0.0);
        assert_eq!(mm.max_x, 100.0);
        assert_eq!(mm.max_y, 200.0);
    }

    #[test]
    fn minmax_mutable_reset() {
        let mut mm = MinMaxMutable::empty(false);
        mm.add_point(10.0, 20.0);
        mm.reset();
        assert_eq!(mm.min_x, 0.0);
        assert_eq!(mm.max_x, 0.0);
    }

    // ── CoordinateChange ─────────────────────────────────────────

    #[test]
    fn coordinate_change_identity() {
        // Horizontal segment from (0,0) to (10,0): u = (1,0), v = (0,1)
        let cc = CoordinateChange::new(0.0, 0.0, 10.0, 0.0);
        assert!((cc.length() - 10.0).abs() < 1e-10);
        let p = cc.true_coordinate(5.0, 0.0);
        assert!((p.x - 5.0).abs() < 1e-10);
        assert!(p.y.abs() < 1e-10);
    }

    #[test]
    fn coordinate_change_perpendicular() {
        let cc = CoordinateChange::new(0.0, 0.0, 10.0, 0.0);
        let p = cc.true_coordinate(0.0, 5.0);
        assert!(p.x.abs() < 1e-10);
        assert!((p.y - 5.0).abs() < 1e-10);
    }

    #[test]
    fn coordinate_change_diagonal() {
        let cc = CoordinateChange::new(0.0, 0.0, 10.0, 10.0);
        let len = cc.length();
        assert!((len - (200.0_f64).sqrt()).abs() < 1e-10);
        // Along the direction
        let p = cc.true_coordinate(len, 0.0);
        assert!((p.x - 10.0).abs() < 1e-10);
        assert!((p.y - 10.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic]
    fn coordinate_change_zero_length() {
        CoordinateChange::new(5.0, 5.0, 5.0, 5.0);
    }

    // ── PointAndAngle ────────────────────────────────────────────

    #[test]
    fn point_and_angle() {
        let pa = PointAndAngle::new(XPoint2D::new(1.0, 2.0), 0.5);
        assert_eq!(pa.x(), 1.0);
        assert_eq!(pa.y(), 2.0);
        assert_eq!(pa.angle, 0.5);
    }

    // ── PointDirected ────────────────────────────────────────────

    #[test]
    fn point_directed() {
        let pd = PointDirected::new(XPoint2D::new(3.0, 4.0), 1.0);
        assert_eq!(pd.point(), XPoint2D::new(3.0, 4.0));
        assert_eq!(pd.angle(), 1.0);
    }

    // ── MinFinder ────────────────────────────────────────────────

    #[test]
    fn min_finder_tracking() {
        let mut mf = MinFinder::new();
        mf.manage(10.0, 20.0);
        mf.manage(5.0, 30.0);
        mf.manage(15.0, 8.0);
        assert_eq!(mf.min_x(), 5.0);
        assert_eq!(mf.min_y(), 8.0);
    }

    #[test]
    fn min_finder_merge() {
        let mut mf1 = MinFinder::new();
        mf1.manage(10.0, 20.0);
        let mut mf2 = MinFinder::new();
        mf2.manage(5.0, 25.0);
        mf1.manage_other(&mf2);
        assert_eq!(mf1.min_x(), 5.0);
        assert_eq!(mf1.min_y(), 20.0);
    }

    // ── RectangleArea ────────────────────────────────────────────

    #[test]
    fn rect_area_basic() {
        let r = RectangleArea::new(10.0, 20.0, 50.0, 60.0);
        assert_eq!(r.width(), 40.0);
        assert_eq!(r.height(), 40.0);
        assert!(r.contains(30.0, 40.0));
        assert!(!r.contains(5.0, 40.0));
        assert!(!r.contains(50.0, 40.0)); // exclusive upper bound
    }

    #[test]
    fn rect_area_from_points() {
        let r = RectangleArea::from_points(
            XPoint2D::new(30.0, 40.0),
            XPoint2D::new(10.0, 20.0),
        );
        assert_eq!(r.min_x, 10.0);
        assert_eq!(r.min_y, 20.0);
        assert_eq!(r.max_x, 30.0);
        assert_eq!(r.max_y, 40.0);
    }

    #[test]
    fn rect_area_merge() {
        let r1 = RectangleArea::new(0.0, 0.0, 10.0, 10.0);
        let r2 = RectangleArea::new(5.0, 5.0, 20.0, 20.0);
        let m = r1.merge(&r2);
        assert_eq!(m.min_x, 0.0);
        assert_eq!(m.max_x, 20.0);
    }

    #[test]
    fn rect_area_center() {
        let r = RectangleArea::new(0.0, 0.0, 10.0, 20.0);
        let c = r.center();
        assert_eq!(c.x, 5.0);
        assert_eq!(c.y, 10.0);
    }

    #[test]
    fn rect_area_closest_side() {
        let r = RectangleArea::new(0.0, 0.0, 100.0, 100.0);
        // Point near the top
        assert_eq!(r.closest_side(XPoint2D::new(50.0, 2.0)), Some(Side::North));
        // Point near the bottom
        assert_eq!(r.closest_side(XPoint2D::new(50.0, 98.0)), Some(Side::South));
        // Point near the left
        assert_eq!(r.closest_side(XPoint2D::new(2.0, 50.0)), Some(Side::West));
        // Point near the right
        assert_eq!(r.closest_side(XPoint2D::new(98.0, 50.0)), Some(Side::East));
    }

    #[test]
    fn rect_area_with_setters() {
        let r = RectangleArea::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(r.with_min_x(5.0).min_x, 5.0);
        assert_eq!(r.with_max_x(20.0).max_x, 20.0);
        assert_eq!(r.add_min_x(3.0).min_x, 3.0);
        assert_eq!(r.add_max_y(5.0).max_y, 15.0);
    }

    #[test]
    fn rect_area_moved() {
        let r = RectangleArea::new(10.0, 20.0, 30.0, 40.0);
        let m = r.moved(5.0, -5.0);
        assert_eq!(m.min_x, 15.0);
        assert_eq!(m.min_y, 15.0);
        assert_eq!(m.max_x, 35.0);
        assert_eq!(m.max_y, 35.0);
    }

    // ── GraphicPosition / VerticalPosition / ImgValign ───────────

    #[test]
    fn enum_variants_exist() {
        let _ = GraphicPosition::Bottom;
        let _ = GraphicPosition::BackgroundCornerBottomRight;
        let _ = GraphicPosition::BackgroundCornerTopRight;
        let _ = VerticalPosition::Top;
        let _ = VerticalPosition::Bottom;
        let _ = ImgValign::Top;
        let _ = ImgValign::Bottom;
        let _ = ImgValign::Middle;
    }
}
