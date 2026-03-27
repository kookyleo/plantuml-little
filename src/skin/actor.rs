// skin::actor - Actor rendering styles and geometry
// Port of Java PlantUML's skin.ActorStyle + ActorStickMan + ActorAwesome + ActorHollow

/// Actor visual style. Java: `skin.ActorStyle`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActorStyle {
    #[default]
    Stickman,
    StickmanBusiness,
    Awesome,
    Hollow,
}

impl ActorStyle {
    /// Parse actor style from a string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "stickman" | "stick" => Some(Self::Stickman),
            "stickman_business" | "stickman-business" => Some(Self::StickmanBusiness),
            "awesome" => Some(Self::Awesome),
            "hollow" => Some(Self::Hollow),
            _ => None,
        }
    }

    /// Get the geometry for this actor style.
    pub fn geometry(self) -> ActorGeometry {
        match self {
            ActorStyle::Stickman => ActorStickMan::new(false).geometry(),
            ActorStyle::StickmanBusiness => ActorStickMan::new(true).geometry(),
            ActorStyle::Awesome => ActorAwesome::new().geometry(),
            ActorStyle::Hollow => ActorHollow::new().geometry(),
        }
    }
}

/// General actor geometry: width and height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActorGeometry {
    pub width: f64,
    pub height: f64,
}

// ---------------------------------------------------------------------------
// ActorStickMan - Java: skin.ActorStickMan
// ---------------------------------------------------------------------------

/// Drawing parameters for the stickman actor.
/// Java fields: armsY=8, armsLenght=13, bodyLenght=27, legsX=13, legsY=15, headDiam=16
#[derive(Debug, Clone)]
pub struct ActorStickMan {
    pub arms_y: f64,
    pub arms_length: f64,
    pub body_length: f64,
    pub legs_x: f64,
    pub legs_y: f64,
    pub head_diam: f64,
    pub is_business: bool,
}

impl ActorStickMan {
    pub fn new(is_business: bool) -> Self {
        Self {
            arms_y: 8.0,
            arms_length: 13.0,
            body_length: 27.0,
            legs_x: 13.0,
            legs_y: 15.0,
            head_diam: 16.0,
            is_business,
        }
    }

    /// The X offset to the head's left edge (stroke thickness = 0 assumed).
    /// Java: `double startX = Math.max(armsLenght, legsX) - headDiam / 2.0 + thickness()`
    pub fn start_x(&self, thickness: f64) -> f64 {
        self.arms_length.max(self.legs_x) - self.head_diam / 2.0 + thickness
    }

    /// Center X of the stickman.
    pub fn center_x(&self, thickness: f64) -> f64 {
        self.start_x(thickness) + self.head_diam / 2.0
    }

    /// Preferred width.
    /// Java: `Math.max(armsLenght, legsX) * 2 + 2 * thickness()`
    pub fn preferred_width(&self, thickness: f64) -> f64 {
        self.arms_length.max(self.legs_x) * 2.0 + 2.0 * thickness
    }

    /// Preferred height.
    /// Java: `headDiam + bodyLenght + legsY + 2 * thickness() + deltaShadow + 1`
    pub fn preferred_height(&self, thickness: f64, delta_shadow: f64) -> f64 {
        self.head_diam + self.body_length + self.legs_y + 2.0 * thickness + delta_shadow + 1.0
    }

    /// Geometry with default stroke (thickness=0, no shadow).
    pub fn geometry(&self) -> ActorGeometry {
        ActorGeometry {
            width: self.preferred_width(0.0),
            height: self.preferred_height(0.0, 0.0),
        }
    }

    /// Build path segments for the stickman body (relative to center).
    /// Returns a list of line segments: each is (from_x, from_y, to_x, to_y).
    /// Caller translates by (center_x, head_diam + thickness).
    pub fn body_segments(&self) -> Vec<(f64, f64, f64, f64)> {
        let mut segs = Vec::new();
        // spine
        segs.push((0.0, 0.0, 0.0, self.body_length));
        // arms
        segs.push((
            -self.arms_length,
            self.arms_y,
            self.arms_length,
            self.arms_y,
        ));
        // left leg
        segs.push((
            0.0,
            self.body_length,
            -self.legs_x,
            self.body_length + self.legs_y,
        ));
        // right leg
        segs.push((
            0.0,
            self.body_length,
            self.legs_x,
            self.body_length + self.legs_y,
        ));
        segs
    }

    /// Compute the business line endpoints for the "business" actor style.
    /// Returns ((x1, y1), (x2, y2)) relative to the head center.
    /// Java: uses angle alpha = 21*PI/64, computes points on circle at PI/4 +/- alpha.
    pub fn business_line_endpoints(&self) -> ((f64, f64), (f64, f64)) {
        let alpha = 21.0 * std::f64::consts::PI / 64.0;
        let angle1 = std::f64::consts::PI / 4.0 + alpha;
        let angle2 = std::f64::consts::PI / 4.0 - alpha;
        let r = self.head_diam / 2.0;
        let p1 = (r * angle1.cos(), r * angle1.sin());
        let p2 = (r * angle2.cos(), r * angle2.sin());
        (p1, p2)
    }
}

// ---------------------------------------------------------------------------
// ActorAwesome - Java: skin.ActorAwesome
// ---------------------------------------------------------------------------

/// Drawing parameters for the "awesome" (Android-like) actor.
/// Java fields: headDiam=32, bodyWidth=54, shoulder=16, collar=4, radius=8, bodyHeight=28
#[derive(Debug, Clone)]
pub struct ActorAwesome {
    pub head_diam: f64,
    pub body_width: f64,
    pub shoulder: f64,
    pub collar: f64,
    pub radius: f64,
    pub body_height: f64,
}

impl ActorAwesome {
    pub fn new() -> Self {
        Self {
            head_diam: 32.0,
            body_width: 54.0,
            shoulder: 16.0,
            collar: 4.0,
            radius: 8.0,
            body_height: 28.0,
        }
    }

    /// Preferred width. Java: `bodyWidth + thickness() * 2`
    pub fn preferred_width(&self, thickness: f64) -> f64 {
        self.body_width + thickness * 2.0
    }

    /// Preferred height. Java: `headDiam + bodyHeight + thickness() * 2`
    pub fn preferred_height(&self, thickness: f64) -> f64 {
        self.head_diam + self.body_height + thickness * 2.0
    }

    /// Center X. Java: `getPreferredWidth() / 2`
    pub fn center_x(&self, thickness: f64) -> f64 {
        self.preferred_width(thickness) / 2.0
    }

    /// Geometry with default stroke (thickness=0).
    pub fn geometry(&self) -> ActorGeometry {
        ActorGeometry {
            width: self.preferred_width(0.0),
            height: self.preferred_height(0.0),
        }
    }

    /// Build the body path as cubic bezier segments.
    /// Returns a list of path commands relative to (0, 0) -- caller translates
    /// by (center_x, head_diam + thickness).
    ///
    /// Commands are `AwesomePathCmd` variants matching the Java UPath calls.
    pub fn body_path(&self) -> Vec<AwesomePathCmd> {
        let bw = self.body_width;
        let sh = self.shoulder;
        let co = self.collar;
        let r = self.radius;
        let bh = self.body_height;

        vec![
            AwesomePathCmd::MoveTo(0.0, co),
            AwesomePathCmd::CubicTo(co, co, bw / 2.0 - sh - co, co, bw / 2.0 - sh, 0.0),
            AwesomePathCmd::CubicTo(bw / 2.0 - sh / 2.0, 0.0, bw / 2.0, sh / 2.0, bw / 2.0, sh),
            AwesomePathCmd::LineTo(bw / 2.0, bh - r),
            AwesomePathCmd::CubicTo(
                bw / 2.0,
                bh - r / 2.0,
                bw / 2.0 - r / 2.0,
                bh,
                bw / 2.0 - r,
                bh,
            ),
            AwesomePathCmd::LineTo(-bw / 2.0 + r, bh),
            AwesomePathCmd::CubicTo(
                -bw / 2.0 + r / 2.0,
                bh,
                -bw / 2.0,
                bh - r / 2.0,
                -bw / 2.0,
                bh - r,
            ),
            AwesomePathCmd::LineTo(-bw / 2.0, sh),
            AwesomePathCmd::CubicTo(
                -bw / 2.0,
                sh / 2.0,
                -bw / 2.0 + sh / 2.0,
                0.0,
                -bw / 2.0 + sh,
                0.0,
            ),
            AwesomePathCmd::CubicTo(-bw / 2.0 + sh + co, co, -co, co, 0.0, co),
            AwesomePathCmd::Close,
        ]
    }
}

impl Default for ActorAwesome {
    fn default() -> Self {
        Self::new()
    }
}

/// Path commands for the awesome actor body shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AwesomePathCmd {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CubicTo(f64, f64, f64, f64, f64, f64),
    Close,
}

// ---------------------------------------------------------------------------
// ActorHollow - Java: skin.ActorHollow
// ---------------------------------------------------------------------------

/// Drawing parameters for the "hollow" actor.
/// Java fields: headDiam=9, bodyWidth=25, bodyHeight=21,
///              neckHeight=2, armThickness=5, bodyThickness=6, legThickness=6
#[derive(Debug, Clone)]
pub struct ActorHollow {
    pub head_diam: f64,
    pub body_width: f64,
    pub body_height: f64,
    pub neck_height: f64,
    pub arm_thickness: f64,
    pub body_thickness: f64,
    pub leg_thickness: f64,
}

impl ActorHollow {
    pub fn new() -> Self {
        Self {
            head_diam: 9.0,
            body_width: 25.0,
            body_height: 21.0,
            neck_height: 2.0,
            arm_thickness: 5.0,
            body_thickness: 6.0,
            leg_thickness: 6.0,
        }
    }

    /// Preferred width. Java: `bodyWidth + thickness() * 2`
    pub fn preferred_width(&self, thickness: f64) -> f64 {
        self.body_width + thickness * 2.0
    }

    /// Preferred height. Java: `headDiam + neckHeight + bodyHeight + thickness() * 2 + deltaShadow`
    pub fn preferred_height(&self, thickness: f64, delta_shadow: f64) -> f64 {
        self.head_diam + self.neck_height + self.body_height + thickness * 2.0 + delta_shadow
    }

    /// Center X. Java: `getPreferredWidth() / 2`
    pub fn center_x(&self, thickness: f64) -> f64 {
        self.preferred_width(thickness) / 2.0
    }

    /// Geometry with default stroke (thickness=0, no shadow).
    pub fn geometry(&self) -> ActorGeometry {
        ActorGeometry {
            width: self.preferred_width(0.0),
            height: self.preferred_height(0.0, 0.0),
        }
    }

    /// Build the hollow body polygon path.
    /// All coordinates relative to (0, 0) -- caller translates by
    /// (center_x, head_diam + thickness + neck_height).
    pub fn body_path(&self) -> Vec<(f64, f64)> {
        let bw = self.body_width;
        let bh = self.body_height;
        let at = self.arm_thickness;
        let bt = self.body_thickness;
        let lt = self.leg_thickness;
        let sqrt2 = std::f64::consts::SQRT_2;

        vec![
            (-bw / 2.0, 0.0),
            (-bw / 2.0, at),
            (-bt / 2.0, at),
            (-bt / 2.0, bh - (bw + lt * sqrt2 - bt) / 2.0),
            (-bw / 2.0, bh - lt * sqrt2 / 2.0),
            (-(bw / 2.0 - lt * sqrt2 / 2.0), bh),
            (0.0, bh - (bw / 2.0 - lt * sqrt2 / 2.0)),
            (bw / 2.0 - lt * sqrt2 / 2.0, bh),
            (bw / 2.0, bh - lt * sqrt2 / 2.0),
            (bt / 2.0, bh - (bw + lt * sqrt2 - bt) / 2.0),
            (bt / 2.0, at),
            (bw / 2.0, at),
            (bw / 2.0, 0.0),
            (-bw / 2.0, 0.0),
        ]
    }
}

impl Default for ActorHollow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ActorStyle parsing ----

    #[test]
    fn parse_style_stickman() {
        assert_eq!(ActorStyle::from_str("stickman"), Some(ActorStyle::Stickman));
        assert_eq!(ActorStyle::from_str("stick"), Some(ActorStyle::Stickman));
        assert_eq!(ActorStyle::from_str("STICKMAN"), Some(ActorStyle::Stickman));
    }

    #[test]
    fn parse_style_stickman_business() {
        assert_eq!(
            ActorStyle::from_str("stickman_business"),
            Some(ActorStyle::StickmanBusiness)
        );
        assert_eq!(
            ActorStyle::from_str("stickman-business"),
            Some(ActorStyle::StickmanBusiness)
        );
    }

    #[test]
    fn parse_style_awesome() {
        assert_eq!(ActorStyle::from_str("awesome"), Some(ActorStyle::Awesome));
    }

    #[test]
    fn parse_style_hollow() {
        assert_eq!(ActorStyle::from_str("hollow"), Some(ActorStyle::Hollow));
    }

    #[test]
    fn parse_style_unknown() {
        assert!(ActorStyle::from_str("unknown").is_none());
        assert!(ActorStyle::from_str("").is_none());
    }

    #[test]
    fn default_style_is_stickman() {
        assert_eq!(ActorStyle::default(), ActorStyle::Stickman);
    }

    // ---- ActorStyle geometry dispatch ----

    #[test]
    fn stickman_geometry_via_style() {
        let g = ActorStyle::Stickman.geometry();
        let s = ActorStickMan::new(false);
        assert!((g.width - s.preferred_width(0.0)).abs() < f64::EPSILON);
        assert!((g.height - s.preferred_height(0.0, 0.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn awesome_geometry_via_style() {
        let g = ActorStyle::Awesome.geometry();
        let a = ActorAwesome::new();
        assert!((g.width - a.preferred_width(0.0)).abs() < f64::EPSILON);
        assert!((g.height - a.preferred_height(0.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_geometry_via_style() {
        let g = ActorStyle::Hollow.geometry();
        let h = ActorHollow::new();
        assert!((g.width - h.preferred_width(0.0)).abs() < f64::EPSILON);
        assert!((g.height - h.preferred_height(0.0, 0.0)).abs() < f64::EPSILON);
    }

    // ---- ActorStickMan ----

    #[test]
    fn stickman_default_dimensions() {
        let s = ActorStickMan::new(false);
        assert!((s.head_diam - 16.0).abs() < f64::EPSILON);
        assert!((s.arms_length - 13.0).abs() < f64::EPSILON);
        assert!((s.body_length - 27.0).abs() < f64::EPSILON);
        assert!((s.legs_x - 13.0).abs() < f64::EPSILON);
        assert!((s.legs_y - 15.0).abs() < f64::EPSILON);
        assert!(!s.is_business);
    }

    #[test]
    fn stickman_business() {
        let s = ActorStickMan::new(true);
        assert!(s.is_business);
    }

    #[test]
    fn stickman_preferred_width_no_thickness() {
        let s = ActorStickMan::new(false);
        // max(13, 13) * 2 + 0 = 26
        assert!((s.preferred_width(0.0) - 26.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stickman_preferred_width_with_thickness() {
        let s = ActorStickMan::new(false);
        // max(13, 13) * 2 + 2*2 = 30
        assert!((s.preferred_width(2.0) - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stickman_preferred_height_no_extras() {
        let s = ActorStickMan::new(false);
        // 16 + 27 + 15 + 0 + 0 + 1 = 59
        assert!((s.preferred_height(0.0, 0.0) - 59.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stickman_preferred_height_with_extras() {
        let s = ActorStickMan::new(false);
        // 16 + 27 + 15 + 2*1 + 3 + 1 = 64
        assert!((s.preferred_height(1.0, 3.0) - 64.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stickman_start_x() {
        let s = ActorStickMan::new(false);
        // max(13, 13) - 16/2 + 0 = 13 - 8 = 5
        assert!((s.start_x(0.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stickman_center_x() {
        let s = ActorStickMan::new(false);
        // start_x(0) + 16/2 = 5 + 8 = 13
        assert!((s.center_x(0.0) - 13.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stickman_body_segments() {
        let s = ActorStickMan::new(false);
        let segs = s.body_segments();
        assert_eq!(segs.len(), 4);
        // Spine
        assert!((segs[0].0 - 0.0).abs() < f64::EPSILON);
        assert!((segs[0].1 - 0.0).abs() < f64::EPSILON);
        assert!((segs[0].2 - 0.0).abs() < f64::EPSILON);
        assert!((segs[0].3 - 27.0).abs() < f64::EPSILON);
        // Arms
        assert!((segs[1].0 - (-13.0)).abs() < f64::EPSILON);
        assert!((segs[1].1 - 8.0).abs() < f64::EPSILON);
        assert!((segs[1].2 - 13.0).abs() < f64::EPSILON);
        assert!((segs[1].3 - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stickman_business_line_endpoints() {
        let s = ActorStickMan::new(true);
        let (p1, p2) = s.business_line_endpoints();
        // Just verify it returns two distinct points on the circle
        let r = s.head_diam / 2.0;
        let dist1 = (p1.0 * p1.0 + p1.1 * p1.1).sqrt();
        let dist2 = (p2.0 * p2.0 + p2.1 * p2.1).sqrt();
        assert!((dist1 - r).abs() < 1e-10);
        assert!((dist2 - r).abs() < 1e-10);
        // Points should be different
        assert!((p1.0 - p2.0).abs() > 0.01 || (p1.1 - p2.1).abs() > 0.01);
    }

    // ---- ActorAwesome ----

    #[test]
    fn awesome_default_dimensions() {
        let a = ActorAwesome::new();
        assert!((a.head_diam - 32.0).abs() < f64::EPSILON);
        assert!((a.body_width - 54.0).abs() < f64::EPSILON);
        assert!((a.shoulder - 16.0).abs() < f64::EPSILON);
        assert!((a.collar - 4.0).abs() < f64::EPSILON);
        assert!((a.radius - 8.0).abs() < f64::EPSILON);
        assert!((a.body_height - 28.0).abs() < f64::EPSILON);
    }

    #[test]
    fn awesome_preferred_width_no_thickness() {
        let a = ActorAwesome::new();
        // 54 + 0 = 54
        assert!((a.preferred_width(0.0) - 54.0).abs() < f64::EPSILON);
    }

    #[test]
    fn awesome_preferred_height_no_thickness() {
        let a = ActorAwesome::new();
        // 32 + 28 + 0 = 60
        assert!((a.preferred_height(0.0) - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn awesome_center_x() {
        let a = ActorAwesome::new();
        // 54 / 2 = 27
        assert!((a.center_x(0.0) - 27.0).abs() < f64::EPSILON);
    }

    #[test]
    fn awesome_body_path_starts_and_ends() {
        let a = ActorAwesome::new();
        let path = a.body_path();
        assert!(!path.is_empty());
        // First command is MoveTo
        match path[0] {
            AwesomePathCmd::MoveTo(x, y) => {
                assert!((x - 0.0).abs() < f64::EPSILON);
                assert!((y - 4.0).abs() < f64::EPSILON); // collar = 4
            }
            _ => panic!("first command should be MoveTo"),
        }
        // Last command is Close
        match path.last().unwrap() {
            AwesomePathCmd::Close => {}
            _ => panic!("last command should be Close"),
        }
    }

    #[test]
    fn awesome_body_path_count() {
        let a = ActorAwesome::new();
        let path = a.body_path();
        // Java: moveTo, 2 cubicTo, lineTo, cubicTo, lineTo, cubicTo, lineTo, cubicTo, cubicTo, close = 11
        assert_eq!(path.len(), 11);
    }

    #[test]
    fn awesome_default_trait() {
        let a = ActorAwesome::default();
        assert!((a.head_diam - 32.0).abs() < f64::EPSILON);
    }

    // ---- ActorHollow ----

    #[test]
    fn hollow_default_dimensions() {
        let h = ActorHollow::new();
        assert!((h.head_diam - 9.0).abs() < f64::EPSILON);
        assert!((h.body_width - 25.0).abs() < f64::EPSILON);
        assert!((h.body_height - 21.0).abs() < f64::EPSILON);
        assert!((h.neck_height - 2.0).abs() < f64::EPSILON);
        assert!((h.arm_thickness - 5.0).abs() < f64::EPSILON);
        assert!((h.body_thickness - 6.0).abs() < f64::EPSILON);
        assert!((h.leg_thickness - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_preferred_width_no_thickness() {
        let h = ActorHollow::new();
        // 25 + 0 = 25
        assert!((h.preferred_width(0.0) - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_preferred_height_no_extras() {
        let h = ActorHollow::new();
        // 9 + 2 + 21 + 0 + 0 = 32
        assert!((h.preferred_height(0.0, 0.0) - 32.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_preferred_height_with_extras() {
        let h = ActorHollow::new();
        // 9 + 2 + 21 + 2*1 + 3 = 37
        assert!((h.preferred_height(1.0, 3.0) - 37.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_center_x() {
        let h = ActorHollow::new();
        // 25 / 2 = 12.5
        assert!((h.center_x(0.0) - 12.5).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_body_path_is_closed_polygon() {
        let h = ActorHollow::new();
        let path = h.body_path();
        // 14 points (last == first to close)
        assert_eq!(path.len(), 14);
        // First and last point should be the same
        assert!((path[0].0 - path[13].0).abs() < f64::EPSILON);
        assert!((path[0].1 - path[13].1).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_body_path_first_point() {
        let h = ActorHollow::new();
        let path = h.body_path();
        // First point: (-bodyWidth/2, 0) = (-12.5, 0)
        assert!((path[0].0 - (-12.5)).abs() < f64::EPSILON);
        assert!((path[0].1 - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_body_path_symmetry() {
        let h = ActorHollow::new();
        let path = h.body_path();
        // The shape should be symmetric about x=0
        // Point 1 (-12.5, 5) should mirror point 11 (12.5, 5)
        assert!((path[1].0 + path[11].0).abs() < f64::EPSILON);
        assert!((path[1].1 - path[11].1).abs() < f64::EPSILON);
    }

    #[test]
    fn hollow_default_trait() {
        let h = ActorHollow::default();
        assert!((h.head_diam - 9.0).abs() < f64::EPSILON);
    }

    // ---- ActorGeometry ----

    #[test]
    fn geometry_struct() {
        let g = ActorGeometry {
            width: 100.0,
            height: 200.0,
        };
        assert!((g.width - 100.0).abs() < f64::EPSILON);
        assert!((g.height - 200.0).abs() < f64::EPSILON);
    }
}
