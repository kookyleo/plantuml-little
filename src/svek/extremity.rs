// svek::extremity - Arrow endpoint shapes
// Port of Java PlantUML's svek.extremity package
//
// Each Extremity draws a small shape at the end of an edge:
// arrows, diamonds, circles, crowfeet, etc.

use std::f64::consts::PI;

use crate::klimt::color::HColor;
use crate::klimt::geom::{Side, XPoint2D};
use crate::klimt::shape::UPath;
use crate::klimt::{UBackground, UStroke, UTranslate};

// ── Geometry helpers ─────────────────────────────────────────────────

/// Rotate a 2D point (x, y) by `angle` radians around the origin.
fn rotate_point(x: f64, y: f64, angle: f64) -> (f64, f64) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, x * sin + y * cos)
}

/// Build a polygon from a list of (x,y) points, apply rotation and translation.
fn build_rotated_polygon(points: &[(f64, f64)], angle: f64, tx: f64, ty: f64) -> Vec<(f64, f64)> {
    points
        .iter()
        .map(|&(x, y)| {
            let (rx, ry) = rotate_point(x, y, angle);
            (rx + tx, ry + ty)
        })
        .collect()
}

/// Round an angle to nearest cardinal direction (0, 90, 180, 270) if very close.
/// Java: `Extremity.manageround()`
pub fn manage_round(angle: f64) -> f64 {
    let deg = angle * 180.0 / PI;
    for &cardinal in &[0.0, 90.0, 180.0, 270.0, 360.0] {
        if (cardinal - deg).abs() < 0.05 {
            return if cardinal == 360.0 {
                0.0
            } else {
                cardinal * PI / 180.0
            };
        }
    }
    angle
}

// ── Extremity trait ──────────────────────────────────────────────────

/// Base trait for arrow endpoint shapes.
/// Java: `svek.extremity.Extremity`
pub trait Extremity {
    /// Draw this extremity using the given UGraphic context.
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic);

    /// A reference point on this extremity (for layout calculations).
    fn some_point(&self) -> Option<XPoint2D>;

    /// Length of the decoration along the edge direction.
    fn decoration_length(&self) -> f64 {
        8.0
    }

    /// Delta for Kal edge adjustment.
    fn delta_for_kal(&self) -> UTranslate {
        UTranslate::none()
    }
}

/// Factory for creating extremities at a given point, angle, and side.
/// Java: `svek.extremity.ExtremityFactory`
pub trait ExtremityFactory {
    fn create(&self, point: XPoint2D, angle: f64, side: Side) -> Box<dyn Extremity>;

    /// Legacy 3-point creation (used for TBR drawable).
    fn create_tbr_legacy(
        &self,
        p0: XPoint2D,
        p1: XPoint2D,
        p2: XPoint2D,
        side: Side,
    ) -> Box<dyn Extremity> {
        let ortho = (p2.y - p0.y).atan2(p2.x - p0.x);
        self.create(p1, ortho, side)
    }
}

/// Factory for creating middle decorations at a given angle.
/// Java: `svek.extremity.MiddleFactory`
pub trait MiddleFactory {
    fn create(&self, angle: f64) -> Box<dyn Extremity>;
}

// ══════════════════════════════════════════════════════════════════════
//  1. ExtremityArrow - filled triangle arrowhead
// ══════════════════════════════════════════════════════════════════════

/// Filled triangle arrowhead. Java: `ExtremityArrow`
///
/// Polygon: tip at (0,0), wings at (-9, -4) and (-9, +4),
/// contact notch at (-5, 0).
pub struct ExtremityArrow {
    polygon: Vec<(f64, f64)>,
    contact: XPoint2D,
    /// Optional line from contact to center (3-point constructor).
    line: Option<(f64, f64)>,
}

impl ExtremityArrow {
    const X_WING: f64 = 9.0;
    const Y_APERTURE: f64 = 4.0;
    const X_CONTACT: f64 = 5.0;

    fn raw_points() -> Vec<(f64, f64)> {
        vec![
            (0.0, 0.0),
            (-Self::X_WING, -Self::Y_APERTURE),
            (-Self::X_CONTACT, 0.0),
            (-Self::X_WING, Self::Y_APERTURE),
            (0.0, 0.0),
        ]
    }

    /// Simple constructor (angle only). Java: `ExtremityArrow(XPoint2D p0, double angle)`
    pub fn new(p0: XPoint2D, angle: f64) -> Self {
        let angle = manage_round(angle);
        let polygon = build_rotated_polygon(&Self::raw_points(), angle, p0.x, p0.y);
        Self {
            polygon,
            contact: p0,
            line: None,
        }
    }

    /// 3-point constructor (with center). Java: `ExtremityArrow(XPoint2D p1, double angle, XPoint2D center)`
    pub fn with_center(p1: XPoint2D, angle: f64, center: XPoint2D) -> Self {
        let angle = manage_round(angle);
        let polygon = build_rotated_polygon(&Self::raw_points(), angle + PI / 2.0, p1.x, p1.y);
        let contact = XPoint2D::new(
            p1.x - Self::X_CONTACT * (angle + PI / 2.0).cos(),
            p1.y - Self::X_CONTACT * (angle + PI / 2.0).sin(),
        );
        let line_dx = center.x - contact.x;
        let line_dy = center.y - contact.y;
        let line_len = (line_dx * line_dx + line_dy * line_dy).sqrt();
        Self {
            polygon,
            contact,
            line: if line_len > 2.0 {
                Some((line_dx, line_dy))
            } else {
                None
            },
        }
    }
}

impl Extremity for ExtremityArrow {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        // Fill with foreground color (Java: ug.apply(color.bg()))
        let color = ug.param().color.clone();
        if color != HColor::None {
            ug.apply(&UBackground::Color(color));
        } else {
            ug.apply(&UBackground::None);
        }
        ug.draw_polygon(&self.polygon);
        if let Some((dx, dy)) = self.line {
            ug.apply(&UTranslate::new(self.contact.x, self.contact.y));
            ug.draw_line(dx, dy);
        }
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        6.0
    }
}

// Factory
pub struct ExtremityFactoryArrow;

impl ExtremityFactory for ExtremityFactoryArrow {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityArrow::new(point, angle))
    }

    fn create_tbr_legacy(
        &self,
        p0: XPoint2D,
        p1: XPoint2D,
        p2: XPoint2D,
        _side: Side,
    ) -> Box<dyn Extremity> {
        let ortho = (p2.y - p0.y).atan2(p2.x - p0.x);
        let center = XPoint2D::new((p0.x + p2.x) / 2.0, (p0.y + p2.y) / 2.0);
        Box::new(ExtremityArrow::with_center(p1, ortho, center))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  2. ExtremityExtends - open triangle for inheritance
// ══════════════════════════════════════════════════════════════════════

/// Open triangle for extends/implements. Java: `ExtremityExtends`
pub struct ExtremityExtends {
    polygon: Vec<(f64, f64)>,
    fill_color: HColor,
    contact: XPoint2D,
}

impl ExtremityExtends {
    const X_WING: f64 = 19.0;
    const Y_APERTURE: f64 = 7.0;

    pub fn new(p1: XPoint2D, angle: f64, background_color: HColor) -> Self {
        let angle = manage_round(angle);
        let raw = vec![
            (0.0, 0.0),
            (-Self::X_WING, -Self::Y_APERTURE),
            (-Self::X_WING, Self::Y_APERTURE),
            (0.0, 0.0),
        ];
        let polygon = build_rotated_polygon(&raw, angle + PI / 2.0, p1.x, p1.y);
        Self {
            polygon,
            fill_color: background_color,
            contact: p1,
        }
    }
}

impl Extremity for ExtremityExtends {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UBackground::Color(self.fill_color.clone()));
        ug.draw_polygon(&self.polygon);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        18.0
    }
}

// Factory
pub struct ExtremityFactoryExtends {
    pub background_color: HColor,
}

impl ExtremityFactory for ExtremityFactoryExtends {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityExtends::new(
            point,
            angle - PI / 2.0,
            self.background_color.clone(),
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  3. ExtremityDiamond - filled/open diamond for composition/aggregation
// ══════════════════════════════════════════════════════════════════════

/// Diamond shape for composition (filled) or aggregation (open).
/// Java: `ExtremityDiamond`
pub struct ExtremityDiamond {
    polygon: Vec<(f64, f64)>,
    fill: bool,
    contact: XPoint2D,
    delta_for_kal: UTranslate,
}

impl ExtremityDiamond {
    const X_WING: f64 = 6.0;
    const Y_APERTURE: f64 = 4.0;

    pub fn new(p1: XPoint2D, angle: f64, fill: bool) -> Self {
        let angle = manage_round(angle);
        let raw = vec![
            (0.0, 0.0),
            (-Self::X_WING, -Self::Y_APERTURE),
            (-Self::X_WING * 2.0, 0.0),
            (-Self::X_WING, Self::Y_APERTURE),
            (0.0, 0.0),
        ];
        // Compute delta_for_kal from point index 2 (the far tip) before translation
        let rotated_only: Vec<(f64, f64)> = raw
            .iter()
            .map(|&(x, y)| rotate_point(x, y, angle + PI / 2.0))
            .collect();
        let (kal_x, kal_y) = rotated_only[2];
        let delta_for_kal = UTranslate::new(-kal_x, -kal_y);

        let polygon = build_rotated_polygon(&raw, angle + PI / 2.0, p1.x, p1.y);
        Self {
            polygon,
            fill,
            contact: p1,
            delta_for_kal,
        }
    }
}

impl Extremity for ExtremityDiamond {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        if self.fill {
            // Filled: use foreground as background (Java: HColors.changeBack)
            let color = ug.param().color.clone();
            ug.apply(&UBackground::Color(color));
        } else {
            ug.apply(&UBackground::None);
        }
        ug.draw_polygon(&self.polygon);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        12.0
    }

    fn delta_for_kal(&self) -> UTranslate {
        self.delta_for_kal
    }
}

// Factory
pub struct ExtremityFactoryDiamond {
    pub fill: bool,
}

impl ExtremityFactory for ExtremityFactoryDiamond {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityDiamond::new(point, angle - PI / 2.0, self.fill))
    }

    fn create_tbr_legacy(
        &self,
        p0: XPoint2D,
        p1: XPoint2D,
        p2: XPoint2D,
        _side: Side,
    ) -> Box<dyn Extremity> {
        let ortho = (p2.y - p0.y).atan2(p2.x - p0.x);
        Box::new(ExtremityDiamond::new(p1, ortho, self.fill))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  4. ExtremityCircle - circle endpoint
// ══════════════════════════════════════════════════════════════════════

/// Circle endpoint shape. Java: `ExtremityCircle`
pub struct ExtremityCircle {
    dest: XPoint2D,
    fill: bool,
    background_color: HColor,
    radius: f64,
}

impl ExtremityCircle {
    const RADIUS: f64 = 6.0;

    pub fn new(center: XPoint2D, fill: bool, angle: f64, background_color: HColor) -> Self {
        let dest = XPoint2D::new(
            center.x - Self::RADIUS * (angle + PI / 2.0).cos(),
            center.y - Self::RADIUS * (angle + PI / 2.0).sin(),
        );
        Self {
            dest,
            fill,
            background_color,
            radius: Self::RADIUS,
        }
    }
}

impl Extremity for ExtremityCircle {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UStroke::with_thickness(1.5));
        if self.fill {
            let color = ug.param().color.clone();
            ug.apply(&UBackground::Color(color));
        } else {
            ug.apply(&UBackground::Color(self.background_color.clone()));
        }
        ug.apply(&UTranslate::new(
            self.dest.x - self.radius,
            self.dest.y - self.radius,
        ));
        ug.draw_ellipse(self.radius * 2.0, self.radius * 2.0);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.dest)
    }

    fn decoration_length(&self) -> f64 {
        12.0
    }
}

// Factory
pub struct ExtremityFactoryCircle {
    pub fill: bool,
    pub background_color: HColor,
}

impl ExtremityFactory for ExtremityFactoryCircle {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityCircle::new(
            point,
            self.fill,
            angle - PI / 2.0,
            self.background_color.clone(),
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  5. ExtremityCrowfoot - ERD crow's foot
// ══════════════════════════════════════════════════════════════════════

/// Crow's foot for ERD "many" relationship. Java: `ExtremityCrowfoot`
pub struct ExtremityCrowfoot {
    contact: XPoint2D,
    angle: f64,
    side: Side,
}

impl ExtremityCrowfoot {
    const X_WING: f64 = 8.0;
    const Y_APERTURE: f64 = 8.0;

    pub fn new(p1: XPoint2D, angle: f64, side: Side) -> Self {
        Self {
            contact: p1,
            angle: manage_round(angle + PI / 2.0),
            side,
        }
    }
}

impl Extremity for ExtremityCrowfoot {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        let middle = (0.0_f64, 0.0_f64);
        let mut left = (0.0, -Self::Y_APERTURE);
        let mut base = (-Self::X_WING, 0.0);
        let mut right = (0.0, Self::Y_APERTURE);

        left = rotate_point(left.0, left.1, self.angle);
        base = rotate_point(base.0, base.1, self.angle);
        right = rotate_point(right.0, right.1, self.angle);

        if self.side == Side::West || self.side == Side::East {
            left = (middle.0, left.1);
            right = (middle.0, right.1);
        }
        if self.side == Side::South || self.side == Side::North {
            left = (left.0, middle.1);
            right = (right.0, middle.1);
        }

        let cx = self.contact.x;
        let cy = self.contact.y;
        draw_line_between(ug, cx, cy, base, left);
        draw_line_between(ug, cx, cy, base, right);
        draw_line_between(ug, cx, cy, base, middle);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        8.0
    }
}

// Factory
pub struct ExtremityFactoryCrowfoot;

impl ExtremityFactory for ExtremityFactoryCrowfoot {
    fn create(&self, point: XPoint2D, angle: f64, side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityCrowfoot::new(point, angle - PI / 2.0, side))
    }

    fn create_tbr_legacy(
        &self,
        p0: XPoint2D,
        p1: XPoint2D,
        p2: XPoint2D,
        side: Side,
    ) -> Box<dyn Extremity> {
        let ortho = (p2.y - p0.y).atan2(p2.x - p0.x);
        Box::new(ExtremityCrowfoot::new(p1, ortho, side))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  6. ExtremityPlus - plus sign inside circle
// ══════════════════════════════════════════════════════════════════════

/// Plus sign inside a circle. Java: `ExtremityPlus`
pub struct ExtremityPlus {
    px: f64,
    py: f64,
    angle: f64,
    background_color: HColor,
    radius: f64,
}

impl ExtremityPlus {
    const RADIUS: f64 = 8.0;

    pub fn new(p1: XPoint2D, angle: f64, background_color: HColor) -> Self {
        let x = p1.x - Self::RADIUS + Self::RADIUS * angle.sin();
        let y = p1.y - Self::RADIUS - Self::RADIUS * angle.cos();
        Self {
            px: x,
            py: y,
            angle,
            background_color,
            radius: Self::RADIUS,
        }
    }

    fn point_on_circle(&self, angle: f64) -> (f64, f64) {
        (
            self.px + self.radius + self.radius * angle.cos(),
            self.py + self.radius + self.radius * angle.sin(),
        )
    }
}

impl Extremity for ExtremityPlus {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UBackground::Color(self.background_color.clone()));
        ug.apply(&UTranslate::new(self.px, self.py));
        ug.draw_ellipse(self.radius * 2.0, self.radius * 2.0);

        // Horizontal cross line
        let p1 = self.point_on_circle(self.angle - PI / 2.0);
        let p2 = self.point_on_circle(self.angle + PI / 2.0);
        draw_line_between(ug, 0.0, 0.0, p1, p2);

        // Vertical cross line
        let p3 = self.point_on_circle(self.angle);
        let p4 = self.point_on_circle(self.angle + PI);
        draw_line_between(ug, 0.0, 0.0, p3, p4);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(XPoint2D::new(self.px, self.py))
    }

    fn decoration_length(&self) -> f64 {
        16.0
    }
}

// Factory
pub struct ExtremityFactoryPlus {
    pub background_color: HColor,
}

impl ExtremityFactory for ExtremityFactoryPlus {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityPlus::new(
            point,
            angle - PI / 2.0,
            self.background_color.clone(),
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  7. ExtremityTriangle - generic triangle with configurable size
// ══════════════════════════════════════════════════════════════════════

/// Configurable triangle endpoint. Java: `ExtremityTriangle`
pub struct ExtremityTriangle {
    polygon: Vec<(f64, f64)>,
    fill: bool,
    background_color: Option<HColor>,
    contact: XPoint2D,
    dec_length: f64,
}

impl ExtremityTriangle {
    pub fn new(
        p1: XPoint2D,
        angle: f64,
        fill: bool,
        background_color: Option<HColor>,
        x_wing: i32,
        y_aperture: i32,
        decoration_length: i32,
    ) -> Self {
        let angle = manage_round(angle);
        let xw = x_wing as f64;
        let ya = y_aperture as f64;
        let raw = vec![(0.0, 0.0), (-xw, -ya), (-xw, ya), (0.0, 0.0)];
        let polygon = build_rotated_polygon(&raw, angle + PI / 2.0, p1.x, p1.y);
        Self {
            polygon,
            fill,
            background_color,
            contact: p1,
            dec_length: decoration_length as f64,
        }
    }
}

impl Extremity for ExtremityTriangle {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        if let Some(ref bg) = self.background_color {
            ug.apply(&UBackground::Color(bg.clone()));
        } else if self.fill {
            let color = ug.param().color.clone();
            ug.apply(&UBackground::Color(color));
        }
        ug.draw_polygon(&self.polygon);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        self.dec_length
    }
}

// Factory
pub struct ExtremityFactoryTriangle {
    pub background_color: Option<HColor>,
    pub x_wing: i32,
    pub y_aperture: i32,
    pub decoration_length: i32,
}

impl ExtremityFactory for ExtremityFactoryTriangle {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityTriangle::new(
            point,
            angle - PI / 2.0,
            false,
            self.background_color.clone(),
            self.x_wing,
            self.y_aperture,
            self.decoration_length,
        ))
    }

    fn create_tbr_legacy(
        &self,
        p0: XPoint2D,
        p1: XPoint2D,
        p2: XPoint2D,
        _side: Side,
    ) -> Box<dyn Extremity> {
        let ortho = (p2.y - p0.y).atan2(p2.x - p0.x);
        Box::new(ExtremityTriangle::new(
            p1,
            ortho,
            true,
            self.background_color.clone(),
            self.x_wing,
            self.y_aperture,
            self.decoration_length,
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  8. ExtremitySquare - square endpoint
// ══════════════════════════════════════════════════════════════════════

/// Square endpoint. Java: `ExtremitySquare`
pub struct ExtremitySquare {
    dest: XPoint2D,
    background_color: HColor,
    radius: f64,
}

impl ExtremitySquare {
    const RADIUS: f64 = 5.0;

    pub fn new(p1: XPoint2D, background_color: HColor) -> Self {
        Self {
            dest: p1,
            background_color,
            radius: Self::RADIUS,
        }
    }
}

impl Extremity for ExtremitySquare {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UStroke::with_thickness(1.5));
        ug.apply(&UBackground::Color(self.background_color.clone()));
        ug.apply(&UTranslate::new(
            self.dest.x - self.radius,
            self.dest.y - self.radius,
        ));
        ug.draw_rect(self.radius * 2.0, self.radius * 2.0, 0.0);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.dest)
    }

    fn decoration_length(&self) -> f64 {
        5.0
    }
}

// Factory
pub struct ExtremityFactorySquare {
    pub background_color: HColor,
}

impl ExtremityFactory for ExtremityFactorySquare {
    fn create(&self, point: XPoint2D, _angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremitySquare::new(point, self.background_color.clone()))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  9. ExtremityNotNavigable - X mark
// ══════════════════════════════════════════════════════════════════════

/// X mark for not-navigable association. Java: `ExtremityNotNavigable`
pub struct ExtremityNotNavigable {
    path: UPath,
    contact: XPoint2D,
}

impl ExtremityNotNavigable {
    const SIZE: f64 = 4.0;
    const MOVE: f64 = 5.0;

    pub fn new(p1: XPoint2D, angle: f64) -> Self {
        let angle = manage_round(angle);
        let mut path = UPath::new();
        // Draw X shape: two crossing lines
        path.move_to(-Self::SIZE, 0.0);
        path.line_to(Self::SIZE, 2.0 * Self::SIZE);
        path.move_to(Self::SIZE, 0.0);
        path.line_to(-Self::SIZE, 2.0 * Self::SIZE);

        // Translate, rotate, then translate to final position
        path = translate_path(&path, 0.0, Self::MOVE);
        path = rotate_path(&path, angle + PI);
        path = translate_path(&path, p1.x, p1.y);

        Self { path, contact: p1 }
    }
}

impl Extremity for ExtremityNotNavigable {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.draw_path(&self.path);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        8.0
    }
}

// Factory
pub struct ExtremityFactoryNotNavigable;

impl ExtremityFactory for ExtremityFactoryNotNavigable {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityNotNavigable::new(point, angle - PI / 2.0))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  10. ExtremityDoubleLine - double bar
// ══════════════════════════════════════════════════════════════════════

/// Double line (bar) endpoint. Java: `ExtremityDoubleLine`
pub struct ExtremityDoubleLine {
    contact: XPoint2D,
    angle: f64,
    line_height: f64,
}

impl ExtremityDoubleLine {
    const LINE_HEIGHT: f64 = 4.0;

    pub fn new(p1: XPoint2D, angle: f64) -> Self {
        Self {
            contact: p1,
            angle: manage_round(angle + PI / 2.0),
            line_height: Self::LINE_HEIGHT,
        }
    }
}

impl Extremity for ExtremityDoubleLine {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        let x_wing: f64 = 4.0;
        let first_line_top = rotate_point(-x_wing, -self.line_height, self.angle);
        let first_line_bottom = rotate_point(-x_wing, self.line_height, self.angle);
        let second_line_top = rotate_point(-x_wing - 3.0, -self.line_height, self.angle);
        let second_line_bottom = rotate_point(-x_wing - 3.0, self.line_height, self.angle);
        let middle = rotate_point(0.0, 0.0, self.angle);
        let base = rotate_point(-x_wing - 4.0, 0.0, self.angle);

        let cx = self.contact.x;
        let cy = self.contact.y;
        draw_line_between(ug, cx, cy, first_line_top, first_line_bottom);
        draw_line_between(ug, cx, cy, second_line_top, second_line_bottom);
        draw_line_between(ug, cx, cy, base, middle);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        8.0
    }
}

// Factory
pub struct ExtremityFactoryDoubleLine;

impl ExtremityFactory for ExtremityFactoryDoubleLine {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityDoubleLine::new(point, angle - PI / 2.0))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  11. ExtremityHalfArrow - half arrowhead
// ══════════════════════════════════════════════════════════════════════

/// Half arrowhead (one wing only). Java: `ExtremityHalfArrow`
pub struct ExtremityHalfArrow {
    contact: XPoint2D,
    line: Option<(f64, f64)>,
    other_line: (f64, f64),
}

impl ExtremityHalfArrow {
    /// Simple constructor (angle + direction).
    pub fn new(p0: XPoint2D, angle: f64, direction: i32) -> Self {
        let angle = manage_round(angle);
        let x_wing: f64 = 9.0;
        let y_aperture = 4.0 * direction as f64;

        let other = rotate_point(-x_wing, -y_aperture, angle);
        let other2 = rotate_point(-8.0, 0.0, angle);

        Self {
            contact: p0,
            line: Some((other.0, other.1)),
            other_line: (other2.0, other2.1),
        }
    }

    /// 3-point constructor.
    pub fn with_center(p1: XPoint2D, angle: f64, center: XPoint2D, direction: i32) -> Self {
        let angle = manage_round(angle);
        let x_wing: f64 = 9.0;
        let y_aperture = 4.0 * direction as f64;
        let other = rotate_point(-x_wing, -y_aperture, angle + PI / 2.0);

        let line_dx = center.x - p1.x;
        let line_dy = center.y - p1.y;
        let line_len = (line_dx * line_dx + line_dy * line_dy).sqrt();
        Self {
            contact: p1,
            line: if line_len > 2.0 {
                Some((line_dx, line_dy))
            } else {
                None
            },
            other_line: (other.0, other.1),
        }
    }
}

impl Extremity for ExtremityHalfArrow {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        let color = ug.param().color.clone();
        ug.apply(&UBackground::Color(color));
        if let Some((dx, dy)) = self.line {
            let len = (dx * dx + dy * dy).sqrt();
            if len > 2.0 {
                ug.apply(&UTranslate::new(self.contact.x, self.contact.y));
                ug.draw_line(dx, dy);
                ug.apply(&UTranslate::new(self.contact.x, self.contact.y));
                ug.draw_line(self.other_line.0, self.other_line.1);
            }
        }
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }
}

// Factory
pub struct ExtremityFactoryHalfArrow {
    pub direction: i32,
}

impl ExtremityFactory for ExtremityFactoryHalfArrow {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityHalfArrow::new(point, angle, self.direction))
    }

    fn create_tbr_legacy(
        &self,
        p0: XPoint2D,
        p1: XPoint2D,
        p2: XPoint2D,
        _side: Side,
    ) -> Box<dyn Extremity> {
        let ortho = (p2.y - p0.y).atan2(p2.x - p0.x);
        let center = XPoint2D::new((p0.x + p2.x) / 2.0, (p0.y + p2.y) / 2.0);
        Box::new(ExtremityHalfArrow::with_center(
            p1,
            ortho,
            center,
            self.direction,
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  12. ExtremityCircleCross - circle with X cross
// ══════════════════════════════════════════════════════════════════════

/// Circle with X cross inside. Java: `ExtremityCircleCross`
pub struct ExtremityCircleCross {
    px: f64,
    py: f64,
    dest: XPoint2D,
    background_color: HColor,
    radius: f64,
}

impl ExtremityCircleCross {
    const RADIUS: f64 = 7.0;

    pub fn new(p1: XPoint2D, background_color: HColor) -> Self {
        Self {
            px: p1.x - Self::RADIUS,
            py: p1.y - Self::RADIUS,
            dest: p1,
            background_color,
            radius: Self::RADIUS,
        }
    }

    fn point_on_circle(&self, angle: f64) -> (f64, f64) {
        (
            self.px + self.radius + self.radius * angle.cos(),
            self.py + self.radius + self.radius * angle.sin(),
        )
    }
}

impl Extremity for ExtremityCircleCross {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UBackground::Color(self.background_color.clone()));
        ug.apply(&UStroke::with_thickness(1.5));
        ug.apply(&UTranslate::new(
            self.dest.x - self.radius,
            self.dest.y - self.radius,
        ));
        ug.draw_ellipse(self.radius * 2.0, self.radius * 2.0);

        let p1 = self.point_on_circle(PI / 4.0);
        let p2 = self.point_on_circle(PI + PI / 4.0);
        draw_line_between(ug, 0.0, 0.0, p1, p2);

        let p3 = self.point_on_circle(-PI / 4.0);
        let p4 = self.point_on_circle(PI - PI / 4.0);
        draw_line_between(ug, 0.0, 0.0, p3, p4);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.dest)
    }
}

// Factory
pub struct ExtremityFactoryCircleCross {
    pub background_color: HColor,
}

impl ExtremityFactory for ExtremityFactoryCircleCross {
    fn create(&self, point: XPoint2D, _angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityCircleCross::new(
            point,
            self.background_color.clone(),
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  13. ExtremityCircleLine - circle with vertical line
// ══════════════════════════════════════════════════════════════════════

/// Circle with a vertical bar. Java: `ExtremityCircleLine`
pub struct ExtremityCircleLine {
    contact: XPoint2D,
    angle: f64,
}

impl ExtremityCircleLine {
    pub fn new(p1: XPoint2D, angle: f64) -> Self {
        Self {
            contact: p1,
            angle: manage_round(angle + PI / 2.0),
        }
    }
}

impl Extremity for ExtremityCircleLine {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        let thickness = ug.param().stroke.thickness;
        let radius = 4.0 + thickness - 1.0;
        let line_height = 4.0 + thickness - 1.0;
        let x_wing: f64 = 4.0;

        let middle = rotate_point(0.0, 0.0, self.angle);
        let base = rotate_point(-x_wing - radius - 3.0, 0.0, self.angle);
        let circle_base = rotate_point(-x_wing - radius - 3.0, 0.0, self.angle);
        let line_top = rotate_point(-x_wing, -line_height, self.angle);
        let line_bottom = rotate_point(-x_wing, line_height, self.angle);

        let cx = self.contact.x;
        let cy = self.contact.y;
        draw_line_between(ug, cx, cy, base, middle);

        ug.apply(&UStroke::with_thickness(thickness));
        ug.apply(&UTranslate::new(
            cx + circle_base.0 - radius,
            cy + circle_base.1 - radius,
        ));
        ug.draw_ellipse(2.0 * radius, 2.0 * radius);

        ug.apply(&UStroke::with_thickness(thickness));
        draw_line_between(ug, cx, cy, line_top, line_bottom);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        15.0
    }
}

// Factory
pub struct ExtremityFactoryCircleLine;

impl ExtremityFactory for ExtremityFactoryCircleLine {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityCircleLine::new(point, angle - PI / 2.0))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  14. ExtremityCircleConnect - circle with arc
// ══════════════════════════════════════════════════════════════════════

/// Circle with surrounding arc. Java: `ExtremityCircleConnect`
pub struct ExtremityCircleConnect {
    dest: XPoint2D,
    _ortho: f64,
    background_color: HColor,
    radius: f64,
    _radius2: f64,
}

impl ExtremityCircleConnect {
    const RADIUS: f64 = 6.0;
    const RADIUS2: f64 = 10.0;

    pub fn new(p1: XPoint2D, ortho: f64, background_color: HColor) -> Self {
        Self {
            dest: p1,
            _ortho: ortho,
            background_color,
            radius: Self::RADIUS,
            _radius2: Self::RADIUS2,
        }
    }
}

impl Extremity for ExtremityCircleConnect {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UStroke::with_thickness(1.5));
        ug.apply(&UBackground::Color(self.background_color.clone()));
        ug.apply(&UTranslate::new(
            self.dest.x - self.radius,
            self.dest.y - self.radius,
        ));
        ug.draw_ellipse(self.radius * 2.0, self.radius * 2.0);
        // Arc rendering depends on SVG-level arc support via UEllipse.
        // Omitted for now; the inner circle is the primary visual.
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.dest)
    }

    fn decoration_length(&self) -> f64 {
        10.0
    }
}

// Factory
pub struct ExtremityFactoryCircleConnect {
    pub background_color: HColor,
}

impl ExtremityFactory for ExtremityFactoryCircleConnect {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityCircleConnect::new(
            point,
            angle - PI / 2.0,
            self.background_color.clone(),
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  15. ExtremityArrowAndCircle - arrowhead with trailing circle
// ══════════════════════════════════════════════════════════════════════

/// Arrow with circle. Java: `ExtremityArrowAndCircle`
pub struct ExtremityArrowAndCircle {
    polygon: Vec<(f64, f64)>,
    contact: XPoint2D,
    dest: XPoint2D,
    background_color: HColor,
    radius: f64,
}

impl ExtremityArrowAndCircle {
    const RADIUS: f64 = 5.0;
    const X_WING: f64 = 9.0;
    const Y_APERTURE: f64 = 4.0;
    const X_CONTACT: f64 = 5.0;

    pub fn new(p1: XPoint2D, angle: f64, background_color: HColor) -> Self {
        let angle = manage_round(angle);
        let raw = vec![
            (0.0, 0.0),
            (-Self::X_WING, -Self::Y_APERTURE),
            (-Self::X_CONTACT, 0.0),
            (-Self::X_WING, Self::Y_APERTURE),
            (0.0, 0.0),
        ];
        let tx = p1.x + Self::RADIUS * angle.sin();
        let ty = p1.y - Self::RADIUS * angle.cos();
        let polygon = build_rotated_polygon(&raw, angle + PI / 2.0, tx, ty);
        let contact = XPoint2D::new(
            p1.x - Self::X_CONTACT * (angle + PI / 2.0).cos(),
            p1.y - Self::X_CONTACT * (angle + PI / 2.0).sin(),
        );
        Self {
            polygon,
            contact,
            dest: p1,
            background_color,
            radius: Self::RADIUS,
        }
    }
}

impl Extremity for ExtremityArrowAndCircle {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        let color = ug.param().color.clone();
        ug.apply(&UBackground::Color(color));
        ug.draw_polygon(&self.polygon);

        ug.apply(&UStroke::with_thickness(1.5));
        ug.apply(&UBackground::Color(self.background_color.clone()));
        ug.apply(&UTranslate::new(
            self.dest.x - self.radius,
            self.dest.y - self.radius,
        ));
        ug.draw_ellipse(self.radius * 2.0, self.radius * 2.0);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }
}

// Factory
pub struct ExtremityFactoryArrowAndCircle {
    pub background_color: HColor,
}

impl ExtremityFactory for ExtremityFactoryArrowAndCircle {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityArrowAndCircle::new(
            point,
            angle - PI / 2.0,
            self.background_color.clone(),
        ))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  16. ExtremityExtendsLike - extends with optional bar or dots
// ══════════════════════════════════════════════════════════════════════

/// Extends-like triangle with optional redefines bar or definedBy dots.
/// Java: `ExtremityExtendsLike`
pub struct ExtremityExtendsLike {
    polygon: Vec<(f64, f64)>,
    fill_color: HColor,
    contact: XPoint2D,
    variant: ExtendsLikeVariant,
}

/// Variant decoration beyond the base triangle.
pub enum ExtendsLikeVariant {
    /// Plain extends triangle only.
    Plain,
    /// Redefines: bar across the triangle base.
    Redefines {
        bar_p1: (f64, f64),
        bar_dx: f64,
        bar_dy: f64,
    },
    /// DefinedBy: two dots at the triangle base.
    DefinedBy {
        dot1_pos: (f64, f64),
        dot2_pos: (f64, f64),
        dot_size: f64,
    },
}

impl ExtremityExtendsLike {
    const X_LEN: f64 = -19.0;
    const HALF_WIDTH: f64 = 7.0;

    /// Base triangle only.
    pub fn new_plain(p_orig: XPoint2D, angle: f64, background_color: HColor) -> Self {
        let angle = manage_round(angle);
        Self::build(p_orig, angle, background_color, ExtendsLikeVariant::Plain)
    }

    /// Redefines variant.
    pub fn new_redefines(p_orig: XPoint2D, angle: f64, background_color: HColor) -> Self {
        let angle = manage_round(angle);
        let x_suffix = Self::X_LEN * 1.2;
        let (p1x, p1y) = Self::rotated_point(x_suffix, -Self::HALF_WIDTH, angle);
        let (p2x, p2y) = Self::rotated_point(x_suffix, Self::HALF_WIDTH, angle);
        let variant = ExtendsLikeVariant::Redefines {
            bar_p1: (p1x + p_orig.x, p1y + p_orig.y),
            bar_dx: p2x - p1x,
            bar_dy: p2y - p1y,
        };
        Self::build(p_orig, angle, background_color, variant)
    }

    /// DefinedBy variant.
    pub fn new_defined_by(p_orig: XPoint2D, angle: f64, background_color: HColor) -> Self {
        let angle = manage_round(angle);
        let x_suffix = Self::X_LEN * 1.3;
        let dot_size = 2.0;
        let w = Self::HALF_WIDTH - dot_size;
        let (d1x, d1y) = Self::rotated_point(x_suffix, -w, angle);
        let (d2x, d2y) = Self::rotated_point(x_suffix, w, angle);
        let variant = ExtendsLikeVariant::DefinedBy {
            dot1_pos: (d1x + p_orig.x - dot_size, d1y + p_orig.y - dot_size),
            dot2_pos: (d2x + p_orig.x - dot_size, d2y + p_orig.y - dot_size),
            dot_size: dot_size * 2.0,
        };
        Self::build(p_orig, angle, background_color, variant)
    }

    fn rotated_point(x: f64, y: f64, angle: f64) -> (f64, f64) {
        // Java uses a custom rotation: ct=cos, st=-sin; nx = x*ct - y*st; ny = -x*st - y*ct
        let ct = angle.cos();
        let st = -angle.sin();
        let nx = x * ct - y * st;
        let ny = -x * st - y * ct;
        (nx, ny)
    }

    fn build(
        p_orig: XPoint2D,
        angle: f64,
        background_color: HColor,
        variant: ExtendsLikeVariant,
    ) -> Self {
        let (t1x, t1y) = Self::rotated_point(Self::X_LEN, -Self::HALF_WIDTH, angle);
        let (t2x, t2y) = Self::rotated_point(Self::X_LEN, Self::HALF_WIDTH, angle);
        let polygon = vec![
            (p_orig.x, p_orig.y),
            (t1x + p_orig.x, t1y + p_orig.y),
            (t2x + p_orig.x, t2y + p_orig.y),
            (p_orig.x, p_orig.y),
        ];
        Self {
            polygon,
            fill_color: background_color,
            contact: p_orig,
            variant,
        }
    }
}

impl Extremity for ExtremityExtendsLike {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UBackground::Color(self.fill_color.clone()));
        ug.draw_polygon(&self.polygon);

        match &self.variant {
            ExtendsLikeVariant::Plain => {}
            ExtendsLikeVariant::Redefines {
                bar_p1,
                bar_dx,
                bar_dy,
            } => {
                ug.apply(&UStroke::with_thickness(2.0));
                ug.apply(&UTranslate::new(bar_p1.0, bar_p1.1));
                ug.draw_line(*bar_dx, *bar_dy);
            }
            ExtendsLikeVariant::DefinedBy {
                dot1_pos,
                dot2_pos,
                dot_size,
            } => {
                let color = ug.param().color.clone();
                if color != HColor::None {
                    ug.apply(&UBackground::Color(color));
                }
                ug.apply(&UTranslate::new(dot1_pos.0, dot1_pos.1));
                ug.draw_ellipse(*dot_size, *dot_size);
                ug.apply(&UTranslate::new(dot2_pos.0, dot2_pos.1));
                ug.draw_ellipse(*dot_size, *dot_size);
            }
        }
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        18.0
    }
}

// Factory
pub struct ExtremityFactoryExtendsLike {
    pub background_color: HColor,
    /// 0 = plain, 1 = redefines, 2 = defined_by
    pub variant: u8,
}

impl ExtremityFactory for ExtremityFactoryExtendsLike {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        match self.variant {
            1 => Box::new(ExtremityExtendsLike::new_redefines(
                point,
                angle,
                self.background_color.clone(),
            )),
            2 => Box::new(ExtremityExtendsLike::new_defined_by(
                point,
                angle,
                self.background_color.clone(),
            )),
            _ => Box::new(ExtremityExtendsLike::new_plain(
                point,
                angle,
                self.background_color.clone(),
            )),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
//  17. ExtremityParenthesis - arc parenthesis
// ══════════════════════════════════════════════════════════════════════

/// Parenthesis arc endpoint. Java: `ExtremityParenthesis`
pub struct ExtremityParenthesis {
    dest: XPoint2D,
    _ortho: f64,
    radius2: f64,
    _ang: f64,
}

impl ExtremityParenthesis {
    const RADIUS2: f64 = 9.0;
    const ANG: f64 = 70.0;

    pub fn new(p1: XPoint2D, ortho: f64) -> Self {
        Self {
            dest: p1,
            _ortho: ortho,
            radius2: Self::RADIUS2,
            _ang: Self::ANG,
        }
    }
}

impl Extremity for ExtremityParenthesis {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UStroke::with_thickness(1.5));
        ug.apply(&UTranslate::new(
            self.dest.x - self.radius2,
            self.dest.y - self.radius2,
        ));
        // In the full SVG renderer this draws an arc via UEllipse start/extend.
        ug.draw_ellipse(self.radius2 * 2.0, self.radius2 * 2.0);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.dest)
    }

    fn decoration_length(&self) -> f64 {
        10.0
    }
}

// Factory
pub struct ExtremityFactoryParenthesis;

impl ExtremityFactory for ExtremityFactoryParenthesis {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityParenthesis::new(point, angle - PI / 2.0))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  18. ExtremityCircleCrowfoot - crowfoot with trailing circle
// ══════════════════════════════════════════════════════════════════════

/// Crow's foot with a trailing circle. Java: `ExtremityCircleCrowfoot`
pub struct ExtremityCircleCrowfoot {
    contact: XPoint2D,
    angle: f64,
    radius: f64,
}

impl ExtremityCircleCrowfoot {
    const RADIUS: f64 = 4.0;

    pub fn new(p1: XPoint2D, angle: f64) -> Self {
        Self {
            contact: p1,
            angle: manage_round(angle + PI / 2.0),
            radius: Self::RADIUS,
        }
    }
}

impl Extremity for ExtremityCircleCrowfoot {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        let x_wing: f64 = 8.0;
        let y_aperture: f64 = 6.0;

        let left = rotate_point(0.0, -y_aperture, self.angle);
        let base = rotate_point(-x_wing, 0.0, self.angle);
        let right = rotate_point(0.0, y_aperture, self.angle);
        let middle = (0.0, 0.0);
        let circle_base = rotate_point(-x_wing - self.radius - 2.0, 0.0, self.angle);

        let cx = self.contact.x;
        let cy = self.contact.y;
        draw_line_between(ug, cx, cy, base, left);
        draw_line_between(ug, cx, cy, base, right);
        draw_line_between(ug, cx, cy, base, middle);

        ug.apply(&UTranslate::new(
            cx + circle_base.0 - self.radius,
            cy + circle_base.1 - self.radius,
        ));
        ug.draw_ellipse(2.0 * self.radius, 2.0 * self.radius);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        18.0
    }
}

// Factory
pub struct ExtremityFactoryCircleCrowfoot;

impl ExtremityFactory for ExtremityFactoryCircleCrowfoot {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityCircleCrowfoot::new(point, angle - PI / 2.0))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  19. ExtremityLineCrowfoot - crowfoot with trailing bar
// ══════════════════════════════════════════════════════════════════════

/// Crow's foot with a trailing vertical bar. Java: `ExtremityLineCrowfoot`
pub struct ExtremityLineCrowfoot {
    contact: XPoint2D,
    angle: f64,
    line_height: f64,
}

impl ExtremityLineCrowfoot {
    const LINE_HEIGHT: f64 = 4.0;

    pub fn new(p1: XPoint2D, angle: f64) -> Self {
        Self {
            contact: p1,
            angle: manage_round(angle + PI / 2.0),
            line_height: Self::LINE_HEIGHT,
        }
    }
}

impl Extremity for ExtremityLineCrowfoot {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        let x_wing: f64 = 8.0;
        let y_aperture: f64 = 6.0;

        let left = rotate_point(0.0, -y_aperture, self.angle);
        let base = rotate_point(-x_wing, 0.0, self.angle);
        let right = rotate_point(0.0, y_aperture, self.angle);
        let middle = (0.0, 0.0);
        let line_top = rotate_point(-x_wing - 2.0, -self.line_height, self.angle);
        let line_bottom = rotate_point(-x_wing - 2.0, self.line_height, self.angle);

        let cx = self.contact.x;
        let cy = self.contact.y;
        draw_line_between(ug, cx, cy, base, left);
        draw_line_between(ug, cx, cy, base, right);
        draw_line_between(ug, cx, cy, base, middle);
        draw_line_between(ug, cx, cy, line_top, line_bottom);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        Some(self.contact)
    }

    fn decoration_length(&self) -> f64 {
        8.0
    }
}

// Factory
pub struct ExtremityFactoryLineCrowfoot;

impl ExtremityFactory for ExtremityFactoryLineCrowfoot {
    fn create(&self, point: XPoint2D, angle: f64, _side: Side) -> Box<dyn Extremity> {
        Box::new(ExtremityLineCrowfoot::new(point, angle - PI / 2.0))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  20. ExtremityOther - raw polygon passthrough
// ══════════════════════════════════════════════════════════════════════

/// Raw polygon passthrough. Java: `ExtremityOther`
pub struct ExtremityOther {
    polygon: Vec<(f64, f64)>,
}

impl ExtremityOther {
    pub fn new(polygon: Vec<(f64, f64)>) -> Self {
        Self { polygon }
    }
}

impl Extremity for ExtremityOther {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.draw_polygon(&self.polygon);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        self.polygon.first().map(|&(x, y)| XPoint2D::new(x, y))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  Middle decorations
// ══════════════════════════════════════════════════════════════════════

// ── MiddleCircleCircledMode ──────────────────────────────────────────

/// Mode for MiddleCircleCircled arc rendering.
/// Java: `MiddleCircleCircledMode`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiddleCircleCircledMode {
    Both,
    Mode1,
    Mode2,
}

// ── MiddleCircle ─────────────────────────────────────────────────────

/// Simple circle at edge midpoint. Java: `MiddleCircle`
pub struct MiddleCircle {
    back_color: HColor,
    radius: f64,
}

impl MiddleCircle {
    const RADIUS: f64 = 6.0;

    pub fn new(back_color: HColor) -> Self {
        Self {
            back_color,
            radius: Self::RADIUS,
        }
    }
}

impl Extremity for MiddleCircle {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UBackground::Color(self.back_color.clone()));
        ug.apply(&UStroke::with_thickness(1.5));
        ug.apply(&UTranslate::new(-self.radius, -self.radius));
        ug.draw_ellipse(self.radius * 2.0, self.radius * 2.0);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        None
    }

    fn decoration_length(&self) -> f64 {
        self.radius
    }
}

/// Factory for MiddleCircle.
pub struct MiddleFactoryCircle {
    pub back_color: HColor,
}

impl MiddleFactory for MiddleFactoryCircle {
    fn create(&self, _angle: f64) -> Box<dyn Extremity> {
        Box::new(MiddleCircle::new(self.back_color.clone()))
    }
}

// ── MiddleCircleCircled ──────────────────────────────────────────────

/// Circle with optional arcs. Java: `MiddleCircleCircled`
pub struct MiddleCircleCircled {
    _angle: f64,
    mode: MiddleCircleCircledMode,
    back_color: HColor,
    diagram_back_color: HColor,
    radius1: f64,
    radius2: f64,
}

impl MiddleCircleCircled {
    const RADIUS1: f64 = 6.0;
    const RADIUS2: f64 = 10.0;

    pub fn new(
        angle: f64,
        mode: MiddleCircleCircledMode,
        back_color: HColor,
        diagram_back_color: HColor,
    ) -> Self {
        Self {
            _angle: angle,
            mode,
            back_color,
            diagram_back_color,
            radius1: Self::RADIUS1,
            radius2: Self::RADIUS2,
        }
    }
}

impl Extremity for MiddleCircleCircled {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        if self.mode == MiddleCircleCircledMode::Both {
            ug.apply(&UBackground::Color(self.diagram_back_color.clone()));
            ug.apply(&UTranslate::new(-self.radius2, -self.radius2));
            ug.draw_ellipse(self.radius2 * 2.0, self.radius2 * 2.0);
        }

        ug.apply(&UBackground::Color(self.back_color.clone()));
        ug.apply(&UStroke::with_thickness(1.5));

        // Arcs would be drawn with UEllipse arc params in the full implementation.
        ug.apply(&UTranslate::new(-self.radius1, -self.radius1));
        ug.draw_ellipse(self.radius1 * 2.0, self.radius1 * 2.0);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        None
    }
}

/// Factory for MiddleCircleCircled.
pub struct MiddleFactoryCircleCircled {
    pub mode: MiddleCircleCircledMode,
    pub back_color: HColor,
    pub diagram_back_color: HColor,
}

impl MiddleFactory for MiddleFactoryCircleCircled {
    fn create(&self, angle: f64) -> Box<dyn Extremity> {
        Box::new(MiddleCircleCircled::new(
            angle,
            self.mode,
            self.back_color.clone(),
            self.diagram_back_color.clone(),
        ))
    }
}

// ── MiddleSubset ─────────────────────────────────────────────────────

/// Subset arc with tangent lines. Java: `MiddleSubset`
pub struct MiddleSubset {
    angle: f64,
    reverse: bool,
    radius: f64,
    length: f64,
}

impl MiddleSubset {
    const RADIUS: f64 = 6.0;
    const LENGTH: f64 = 10.0;

    pub fn new(angle: f64, reverse: bool) -> Self {
        Self {
            angle,
            reverse,
            radius: Self::RADIUS,
            length: Self::LENGTH,
        }
    }
}

impl Extremity for MiddleSubset {
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic) {
        ug.apply(&UStroke::with_thickness(1.5));

        let rotate_deg = if self.reverse { -45.0 } else { 135.0 };
        let total_angle = self.angle + rotate_deg;
        let total_rad = total_angle.to_radians();

        // Arc: drawn as a partial ellipse placeholder.
        ug.apply(&UTranslate::new(-self.radius, -self.radius));
        ug.draw_ellipse(self.radius * 2.0, self.radius * 2.0);

        // Two tangent lines from the arc endpoints.
        let sin_val = total_rad.sin();
        let cos_val = total_rad.cos();
        ug.apply(&UTranslate::new(
            self.radius * cos_val,
            self.radius * -sin_val,
        ));
        ug.draw_line(self.length * sin_val, self.length * cos_val);
        ug.apply(&UTranslate::new(
            self.radius * -cos_val,
            self.radius * sin_val,
        ));
        ug.draw_line(self.length * sin_val, self.length * cos_val);
    }

    fn some_point(&self) -> Option<XPoint2D> {
        None
    }

    fn decoration_length(&self) -> f64 {
        self.radius
    }
}

/// Factory for MiddleSubset.
pub struct MiddleFactorySubset {
    pub reverse: bool,
}

impl MiddleFactory for MiddleFactorySubset {
    fn create(&self, angle: f64) -> Box<dyn Extremity> {
        Box::new(MiddleSubset::new(angle, self.reverse))
    }
}

// ══════════════════════════════════════════════════════════════════════
//  Helpers
// ══════════════════════════════════════════════════════════════════════

/// Draw a line from p1 to p2, offset by (offset_x, offset_y).
/// Java pattern: `ug.apply(new UTranslate(x + p1.x, y + p1.y)).draw(new ULine(dx, dy))`
fn draw_line_between(
    ug: &mut dyn crate::klimt::UGraphic,
    offset_x: f64,
    offset_y: f64,
    p1: (f64, f64),
    p2: (f64, f64),
) {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    ug.apply(&UTranslate::new(offset_x + p1.0, offset_y + p1.1));
    ug.draw_line(dx, dy);
}

/// Translate all segments of a UPath by (dx, dy).
fn translate_path(path: &UPath, dx: f64, dy: f64) -> UPath {
    use crate::klimt::geom::USegmentType;
    let mut result = UPath::new();
    for seg in &path.segments {
        let mut coords = seg.coords.clone();
        match seg.kind {
            USegmentType::MoveTo | USegmentType::LineTo => {
                coords[0] += dx;
                coords[1] += dy;
            }
            USegmentType::CubicTo => {
                coords[0] += dx;
                coords[1] += dy;
                coords[2] += dx;
                coords[3] += dy;
                coords[4] += dx;
                coords[5] += dy;
            }
            USegmentType::ArcTo => {
                coords[5] += dx;
                coords[6] += dy;
            }
            USegmentType::Close => {}
        }
        result.segments.push(crate::klimt::geom::USegment {
            kind: seg.kind,
            coords,
        });
    }
    result
}

/// Rotate all segments of a UPath by `angle` radians around the origin.
fn rotate_path(path: &UPath, angle: f64) -> UPath {
    use crate::klimt::geom::USegmentType;
    let mut result = UPath::new();
    for seg in &path.segments {
        let mut coords = seg.coords.clone();
        match seg.kind {
            USegmentType::MoveTo | USegmentType::LineTo => {
                let (rx, ry) = rotate_point(coords[0], coords[1], angle);
                coords[0] = rx;
                coords[1] = ry;
            }
            USegmentType::CubicTo => {
                let (r0, r1) = rotate_point(coords[0], coords[1], angle);
                let (r2, r3) = rotate_point(coords[2], coords[3], angle);
                let (r4, r5) = rotate_point(coords[4], coords[5], angle);
                coords[0] = r0;
                coords[1] = r1;
                coords[2] = r2;
                coords[3] = r3;
                coords[4] = r4;
                coords[5] = r5;
            }
            USegmentType::ArcTo => {
                let (r5, r6) = rotate_point(coords[5], coords[6], angle);
                coords[5] = r5;
                coords[6] = r6;
            }
            USegmentType::Close => {}
        }
        result.segments.push(crate::klimt::geom::USegment {
            kind: seg.kind,
            coords,
        });
    }
    result
}

// ══════════════════════════════════════════════════════════════════════
//  Bridge: LinkDecoration → ExtremityFactory → draw via UGraphicSvg
// ══════════════════════════════════════════════════════════════════════

use crate::svek::edge::LinkDecoration;

/// Create an `ExtremityFactory` for a given `LinkDecoration`.
/// Returns `None` for `LinkDecoration::None`.
pub fn factory_for_decoration(
    decor: LinkDecoration,
    _fill: bool,
) -> Option<Box<dyn ExtremityFactory>> {
    match decor {
        LinkDecoration::None => None,
        LinkDecoration::Arrow => Some(Box::new(ExtremityFactoryArrow)),
        LinkDecoration::Extends => Some(Box::new(ExtremityFactoryExtends {
            background_color: HColor::simple("#FFFFFF"),
        })),
        LinkDecoration::Composition => Some(Box::new(ExtremityFactoryDiamond { fill: true })),
        LinkDecoration::Aggregation => Some(Box::new(ExtremityFactoryDiamond { fill: false })),
        LinkDecoration::Circle => Some(Box::new(ExtremityFactoryCircle {
            fill: false,
            background_color: HColor::None,
        })),
        LinkDecoration::CircleFill => Some(Box::new(ExtremityFactoryCircle {
            fill: true,
            background_color: HColor::None,
        })),
        LinkDecoration::CircleCross => Some(Box::new(ExtremityFactoryCircleCross {
            background_color: HColor::None,
        })),
        LinkDecoration::Plus => Some(Box::new(ExtremityFactoryPlus {
            background_color: HColor::simple("#FFFFFF"),
        })),
        LinkDecoration::HalfArrow => Some(Box::new(ExtremityFactoryHalfArrow { direction: 1 })),
        LinkDecoration::Crowfoot => Some(Box::new(ExtremityFactoryCrowfoot)),
        LinkDecoration::NotNavigable => Some(Box::new(ExtremityFactoryNotNavigable)),
        LinkDecoration::SquareSingle => Some(Box::new(ExtremityFactorySquare {
            background_color: HColor::simple("#FFFFFF"),
        })),
        LinkDecoration::Parenthesis => Some(Box::new(ExtremityFactoryParenthesis)),
    }
}

/// Draw an extremity onto an SvgGraphic.
///
/// This is the bridge between the svek extremity system and the SvgGraphic
/// rendering pipeline. Creates a temporary `UGraphicSvg`, draws the extremity,
/// and pushes the result into the given `SvgGraphic`.
pub fn draw_extremity_to_svg(
    sg: &mut crate::klimt::svg::SvgGraphic,
    decor: LinkDecoration,
    point: XPoint2D,
    angle: f64,
    side: Side,
    stroke_color: &str,
) {
    let factory = match factory_for_decoration(decor, decor.is_fill()) {
        Some(f) => f,
        None => return,
    };

    let extremity = factory.create(point, angle, side);

    // Create a UGraphicSvg wrapping a temporary SvgGraphic
    let temp_sg = crate::klimt::svg::SvgGraphic::new(0, 1.0);
    let sb: Box<dyn crate::klimt::font::StringBounder> =
        Box::new(crate::klimt::font::DefaultStringBounder);
    let mut ug =
        crate::klimt::svg::UGraphicSvg::new(temp_sg, sb, crate::klimt::svg::LengthAdjust::Spacing);

    // Set stroke/fill colors
    use crate::klimt::UGraphic;
    ug.apply(&HColor::simple(stroke_color));
    ug.apply(&UBackground::Color(HColor::simple(stroke_color)));
    ug.apply(&UStroke::with_thickness(1.0));

    // Draw
    extremity.draw(&mut ug);

    // Extract rendered SVG and push into the main SvgGraphic
    let rendered = ug.into_svg();
    sg.push_raw(rendered.body());
}

// ══════════════════════════════════════════════════════════════════════
//  Tests
// ══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── manage_round ─────────────────────────────────────────────────

    #[test]
    fn manage_round_cardinal() {
        assert!((manage_round(0.0001) - 0.0).abs() < 1e-9);
        assert!((manage_round(PI / 2.0 + 0.0001) - PI / 2.0).abs() < 1e-9);
        assert!((manage_round(PI - 0.0001) - PI).abs() < 1e-9);
    }

    #[test]
    fn manage_round_non_cardinal() {
        #[allow(clippy::approx_constant)]
        let angle = 0.7854; // ~45 degrees
        assert!((manage_round(angle) - angle).abs() < 1e-9);
    }

    #[test]
    fn manage_round_360_wraps_to_0() {
        let near_360 = 2.0 * PI - 0.0001;
        let result = manage_round(near_360);
        assert!(result.abs() < 1e-9);
    }

    // ── rotate_point ─────────────────────────────────────────────────

    #[test]
    fn rotate_point_zero() {
        let (x, y) = rotate_point(1.0, 0.0, 0.0);
        assert!((x - 1.0).abs() < 1e-9);
        assert!(y.abs() < 1e-9);
    }

    #[test]
    fn rotate_point_90() {
        let (x, y) = rotate_point(1.0, 0.0, PI / 2.0);
        assert!(x.abs() < 1e-9);
        assert!((y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rotate_point_180() {
        let (x, y) = rotate_point(1.0, 0.0, PI);
        assert!((x + 1.0).abs() < 1e-9);
        assert!(y.abs() < 1e-9);
    }

    // ── ExtremityArrow polygon ───────────────────────────────────────

    #[test]
    fn arrow_raw_points_count() {
        let pts = ExtremityArrow::raw_points();
        assert_eq!(pts.len(), 5);
        assert_eq!(pts[0], (0.0, 0.0));
        assert_eq!(pts[4], (0.0, 0.0));
    }

    #[test]
    fn arrow_polygon_at_zero_angle() {
        let arrow = ExtremityArrow::new(XPoint2D::new(100.0, 50.0), 0.0);
        assert_eq!(arrow.polygon.len(), 5);
        assert!((arrow.polygon[0].0 - 100.0).abs() < 1e-9);
        assert!((arrow.polygon[0].1 - 50.0).abs() < 1e-9);
        assert!((arrow.polygon[1].0 - (100.0 - 9.0)).abs() < 1e-9);
        assert!((arrow.polygon[1].1 - (50.0 - 4.0)).abs() < 1e-9);
    }

    #[test]
    fn arrow_decoration_length() {
        let arrow = ExtremityArrow::new(XPoint2D::new(0.0, 0.0), 0.0);
        assert_eq!(arrow.decoration_length(), 6.0);
    }

    #[test]
    fn arrow_some_point() {
        let arrow = ExtremityArrow::new(XPoint2D::new(10.0, 20.0), 0.0);
        assert_eq!(arrow.some_point(), Some(XPoint2D::new(10.0, 20.0)));
    }

    // ── ExtremityDiamond polygon ─────────────────────────────────────

    #[test]
    fn diamond_polygon_point_count() {
        let diamond = ExtremityDiamond::new(XPoint2D::new(0.0, 0.0), 0.0, true);
        assert_eq!(diamond.polygon.len(), 5);
    }

    #[test]
    fn diamond_decoration_length() {
        let diamond = ExtremityDiamond::new(XPoint2D::new(0.0, 0.0), 0.0, true);
        assert_eq!(diamond.decoration_length(), 12.0);
    }

    #[test]
    fn diamond_filled_vs_open() {
        let filled = ExtremityDiamond::new(XPoint2D::new(0.0, 0.0), 0.0, true);
        let open = ExtremityDiamond::new(XPoint2D::new(0.0, 0.0), 0.0, false);
        assert!(filled.fill);
        assert!(!open.fill);
    }

    #[test]
    fn diamond_delta_for_kal_is_set() {
        let diamond = ExtremityDiamond::new(XPoint2D::new(0.0, 0.0), PI / 2.0, true);
        let delta = diamond.delta_for_kal();
        assert!(delta.dx.abs() > 1e-3 || delta.dy.abs() > 1e-3);
    }

    // ── ExtremityExtends ─────────────────────────────────────────────

    #[test]
    fn extends_polygon_count() {
        let ext = ExtremityExtends::new(XPoint2D::new(0.0, 0.0), 0.0, HColor::None);
        assert_eq!(ext.polygon.len(), 4);
    }

    #[test]
    fn extends_decoration_length() {
        let ext = ExtremityExtends::new(XPoint2D::new(0.0, 0.0), 0.0, HColor::None);
        assert_eq!(ext.decoration_length(), 18.0);
    }

    // ── ExtremityCircle ──────────────────────────────────────────────

    #[test]
    fn circle_dest_offset_from_center() {
        let circle = ExtremityCircle::new(XPoint2D::new(100.0, 50.0), false, 0.0, HColor::None);
        let dest = circle.some_point().unwrap();
        assert!((dest.x - 100.0).abs() < 1e-9);
        assert!((dest.y - (50.0 - 6.0)).abs() < 1e-9);
    }

    #[test]
    fn circle_decoration_length() {
        let circle = ExtremityCircle::new(XPoint2D::new(0.0, 0.0), false, 0.0, HColor::None);
        assert_eq!(circle.decoration_length(), 12.0);
    }

    // ── ExtremityCrowfoot ────────────────────────────────────────────

    #[test]
    fn crowfoot_decoration_length() {
        let cf = ExtremityCrowfoot::new(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(cf.decoration_length(), 8.0);
    }

    #[test]
    fn crowfoot_some_point() {
        let cf = ExtremityCrowfoot::new(XPoint2D::new(10.0, 20.0), 0.0, Side::East);
        assert_eq!(cf.some_point(), Some(XPoint2D::new(10.0, 20.0)));
    }

    // ── ExtremityPlus ────────────────────────────────────────────────

    #[test]
    fn plus_decoration_length() {
        let plus = ExtremityPlus::new(XPoint2D::new(0.0, 0.0), 0.0, HColor::None);
        assert_eq!(plus.decoration_length(), 16.0);
    }

    // ── ExtremityTriangle ────────────────────────────────────────────

    #[test]
    fn triangle_polygon_count() {
        let tri = ExtremityTriangle::new(XPoint2D::new(0.0, 0.0), 0.0, false, None, 8, 4, 12);
        assert_eq!(tri.polygon.len(), 4);
    }

    #[test]
    fn triangle_custom_decoration_length() {
        let tri = ExtremityTriangle::new(XPoint2D::new(0.0, 0.0), 0.0, false, None, 8, 4, 15);
        assert_eq!(tri.decoration_length(), 15.0);
    }

    // ── ExtremitySquare ──────────────────────────────────────────────

    #[test]
    fn square_decoration_length() {
        let sq = ExtremitySquare::new(XPoint2D::new(0.0, 0.0), HColor::None);
        assert_eq!(sq.decoration_length(), 5.0);
    }

    // ── ExtremityNotNavigable ────────────────────────────────────────

    #[test]
    fn not_navigable_path_has_segments() {
        let nn = ExtremityNotNavigable::new(XPoint2D::new(50.0, 50.0), 0.0);
        assert!(!nn.path.segments.is_empty());
    }

    #[test]
    fn not_navigable_decoration_length() {
        let nn = ExtremityNotNavigable::new(XPoint2D::new(0.0, 0.0), 0.0);
        assert_eq!(nn.decoration_length(), 8.0);
    }

    // ── ExtremityDoubleLine ──────────────────────────────────────────

    #[test]
    fn double_line_decoration_length() {
        let dl = ExtremityDoubleLine::new(XPoint2D::new(0.0, 0.0), 0.0);
        assert_eq!(dl.decoration_length(), 8.0);
    }

    // ── build_rotated_polygon ────────────────────────────────────────

    #[test]
    fn build_polygon_translation_only() {
        let points = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let poly = build_rotated_polygon(&points, 0.0, 5.0, 5.0);
        assert_eq!(poly.len(), 3);
        assert!((poly[0].0 - 5.0).abs() < 1e-9);
        assert!((poly[0].1 - 5.0).abs() < 1e-9);
        assert!((poly[1].0 - 15.0).abs() < 1e-9);
        assert!((poly[1].1 - 5.0).abs() < 1e-9);
    }

    #[test]
    fn build_polygon_rotation_90() {
        let points = vec![(1.0, 0.0)];
        let poly = build_rotated_polygon(&points, PI / 2.0, 0.0, 0.0);
        assert!(poly[0].0.abs() < 1e-9);
        assert!((poly[0].1 - 1.0).abs() < 1e-9);
    }

    // ── translate_path / rotate_path ─────────────────────────────────

    #[test]
    fn translate_path_moves_all_coords() {
        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 10.0);
        let translated = translate_path(&path, 5.0, 3.0);
        assert_eq!(translated.segments[0].coords[0], 5.0);
        assert_eq!(translated.segments[0].coords[1], 3.0);
        assert_eq!(translated.segments[1].coords[0], 15.0);
        assert_eq!(translated.segments[1].coords[1], 13.0);
    }

    #[test]
    fn rotate_path_90_degrees() {
        let mut path = UPath::new();
        path.move_to(1.0, 0.0);
        let rotated = rotate_path(&path, PI / 2.0);
        assert!(rotated.segments[0].coords[0].abs() < 1e-9);
        assert!((rotated.segments[0].coords[1] - 1.0).abs() < 1e-9);
    }

    // ── MiddleCircle ─────────────────────────────────────────────────

    #[test]
    fn middle_circle_decoration_length() {
        let mc = MiddleCircle::new(HColor::None);
        assert_eq!(mc.decoration_length(), 6.0);
    }

    #[test]
    fn middle_circle_some_point_is_none() {
        let mc = MiddleCircle::new(HColor::None);
        assert_eq!(mc.some_point(), None);
    }

    // ── MiddleSubset ─────────────────────────────────────────────────

    #[test]
    fn middle_subset_decoration_length() {
        let ms = MiddleSubset::new(0.0, false);
        assert_eq!(ms.decoration_length(), 6.0);
    }

    #[test]
    fn middle_subset_some_point_is_none() {
        let ms = MiddleSubset::new(45.0, true);
        assert_eq!(ms.some_point(), None);
    }

    // ── ExtremityExtendsLike ─────────────────────────────────────────

    #[test]
    fn extends_like_plain_polygon_count() {
        let ext = ExtremityExtendsLike::new_plain(XPoint2D::new(0.0, 0.0), 0.0, HColor::None);
        assert_eq!(ext.polygon.len(), 4);
    }

    #[test]
    fn extends_like_decoration_length() {
        let ext = ExtremityExtendsLike::new_redefines(XPoint2D::new(0.0, 0.0), 0.0, HColor::None);
        assert_eq!(ext.decoration_length(), 18.0);
    }

    // ── Factory tests ────────────────────────────────────────────────

    #[test]
    fn factory_arrow_creates_extremity() {
        let factory = ExtremityFactoryArrow;
        let ext = factory.create(XPoint2D::new(100.0, 50.0), PI / 2.0, Side::North);
        assert!(ext.some_point().is_some());
        assert_eq!(ext.decoration_length(), 6.0);
    }

    #[test]
    fn factory_diamond_filled() {
        let factory = ExtremityFactoryDiamond { fill: true };
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 12.0);
    }

    #[test]
    fn factory_extends() {
        let factory = ExtremityFactoryExtends {
            background_color: HColor::None,
        };
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 18.0);
    }

    #[test]
    fn factory_crowfoot() {
        let factory = ExtremityFactoryCrowfoot;
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::East);
        assert_eq!(ext.decoration_length(), 8.0);
    }

    #[test]
    fn factory_circle() {
        let factory = ExtremityFactoryCircle {
            fill: false,
            background_color: HColor::None,
        };
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 12.0);
    }

    #[test]
    fn factory_plus() {
        let factory = ExtremityFactoryPlus {
            background_color: HColor::None,
        };
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 16.0);
    }

    #[test]
    fn factory_triangle() {
        let factory = ExtremityFactoryTriangle {
            background_color: None,
            x_wing: 8,
            y_aperture: 4,
            decoration_length: 10,
        };
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 10.0);
    }

    #[test]
    fn factory_square() {
        let factory = ExtremityFactorySquare {
            background_color: HColor::None,
        };
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 5.0);
    }

    #[test]
    fn factory_not_navigable() {
        let factory = ExtremityFactoryNotNavigable;
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 8.0);
    }

    #[test]
    fn factory_double_line() {
        let factory = ExtremityFactoryDoubleLine;
        let ext = factory.create(XPoint2D::new(0.0, 0.0), 0.0, Side::North);
        assert_eq!(ext.decoration_length(), 8.0);
    }

    #[test]
    fn middle_factory_circle() {
        let factory = MiddleFactoryCircle {
            back_color: HColor::None,
        };
        let ext = factory.create(0.0);
        assert_eq!(ext.some_point(), None);
        assert_eq!(ext.decoration_length(), 6.0);
    }

    #[test]
    fn middle_factory_subset() {
        let factory = MiddleFactorySubset { reverse: false };
        let ext = factory.create(45.0);
        assert_eq!(ext.some_point(), None);
        assert_eq!(ext.decoration_length(), 6.0);
    }

    #[test]
    fn middle_factory_circle_circled() {
        let factory = MiddleFactoryCircleCircled {
            mode: MiddleCircleCircledMode::Both,
            back_color: HColor::None,
            diagram_back_color: HColor::None,
        };
        let ext = factory.create(45.0);
        assert_eq!(ext.some_point(), None);
    }

    // ── Geometric accuracy tests ─────────────────────────────────────

    #[test]
    fn arrow_rotated_90_degrees() {
        let arrow = ExtremityArrow::new(XPoint2D::new(0.0, 0.0), PI / 2.0);
        assert!((arrow.polygon[0].0).abs() < 1e-9);
        assert!((arrow.polygon[0].1).abs() < 1e-9);
        assert!((arrow.polygon[1].0 - 4.0).abs() < 1e-6);
        assert!((arrow.polygon[1].1 - (-9.0)).abs() < 1e-6);
    }

    #[test]
    fn diamond_at_zero_symmetry() {
        let diamond = ExtremityDiamond::new(XPoint2D::new(0.0, 0.0), 0.0, false);
        // Diamond polygon should have 4+ points
        assert!(
            diamond.polygon.len() >= 4,
            "Diamond should have at least 4 points"
        );
        // Points should form a diamond shape (non-degenerate)
        let (min_x, max_x) = diamond
            .polygon
            .iter()
            .fold((f64::MAX, f64::MIN), |(mn, mx), p| {
                (mn.min(p.0), mx.max(p.0))
            });
        assert!(max_x - min_x > 0.1, "Diamond should have non-zero width");
    }

    #[test]
    fn extends_triangle_at_zero() {
        let ext = ExtremityExtends::new(XPoint2D::new(100.0, 100.0), 0.0, HColor::None);
        assert!((ext.polygon[0].0 - 100.0).abs() < 1e-9);
        assert!((ext.polygon[0].1 - 100.0).abs() < 1e-9);
    }

    #[test]
    fn circle_crowfoot_decoration_length() {
        let ccf = ExtremityCircleCrowfoot::new(XPoint2D::new(0.0, 0.0), 0.0);
        assert_eq!(ccf.decoration_length(), 18.0);
    }

    #[test]
    fn line_crowfoot_decoration_length() {
        let lcf = ExtremityLineCrowfoot::new(XPoint2D::new(0.0, 0.0), 0.0);
        assert_eq!(lcf.decoration_length(), 8.0);
    }
}
