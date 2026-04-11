// klimt::hand - Handwritten-style drawing
// Port of Java PlantUML's klimt.drawing.hand package
//
// Converts clean geometric shapes into "hand-drawn" wobbly versions
// using a deterministic seeded PRNG for reproducible output.

/// Java-compatible `Random(long seed)` using Java's LCG algorithm.
/// Produces identical `nextDouble()` sequences to `java.util.Random`.
pub struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    const MULTIPLIER: i64 = 0x5DEECE66D;
    const INCREMENT: i64 = 0xB;
    const MASK: i64 = (1i64 << 48) - 1;

    pub fn new(seed: i64) -> Self {
        // Java: this.seed = (seed ^ multiplier) & mask
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    /// Generate next n bits (Java's `next(int bits)`)
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT))
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    /// Java's `nextDouble()`: two calls to next(), producing a double in [0, 1)
    pub fn next_double(&mut self) -> f64 {
        let hi = self.next(26) as i64;
        let lo = self.next(27) as i64;
        ((hi << 27) + lo) as f64 / ((1i64 << 53) as f64)
    }
}

/// Represents a sequence of jiggled points, built by tracing line segments
/// with random perturbation. Port of Java `HandJiggle`.
pub struct HandJiggle {
    points: Vec<(f64, f64)>,
    start_x: f64,
    start_y: f64,
    default_variation: f64,
    rnd: JavaRandom,
}

impl HandJiggle {
    pub fn new(start_x: f64, start_y: f64, default_variation: f64, rnd: JavaRandom) -> Self {
        let mut hj = Self {
            points: Vec::new(),
            start_x,
            start_y,
            default_variation,
            rnd,
        };
        hj.points.push((start_x, start_y));
        hj
    }

    /// Create from a starting point, taking ownership of the RNG.
    pub fn create(start: (f64, f64), default_variation: f64, rnd: JavaRandom) -> Self {
        Self::new(start.0, start.1, default_variation, rnd)
    }

    pub fn line_to(&mut self, end_x: f64, end_y: f64) {
        let diff_x = (end_x - self.start_x).abs();
        let diff_y = (end_y - self.start_y).abs();
        let distance = (diff_x * diff_x + diff_y * diff_y).sqrt();
        if distance < 0.001 {
            return;
        }

        let mut segments = (distance / 10.0).round() as i32;
        let mut variation = self.default_variation;
        if segments < 5 {
            segments = 5;
            variation /= 3.0;
        }

        let step_x = (end_x - self.start_x).signum() * diff_x / segments as f64;
        let step_y = (end_y - self.start_y).signum() * diff_y / segments as f64;

        let fx = diff_x / distance;
        let fy = diff_y / distance;

        for s in 0..segments {
            let x = step_x * s as f64 + self.start_x;
            let y = step_y * s as f64 + self.start_y;

            let offset = (self.rnd.next_double() - 0.5) * variation;
            self.points.push((x - offset * fy, y - offset * fx));
        }
        self.points.push((end_x, end_y));

        self.start_x = end_x;
        self.start_y = end_y;
    }

    pub fn line_to_pt(&mut self, end: (f64, f64)) {
        self.line_to(end.0, end.1);
    }

    pub fn arc_to(
        &mut self,
        angle0: f64,
        angle1: f64,
        center_x: f64,
        center_y: f64,
        rx: f64,
        ry: f64,
    ) {
        let mid = point_on_circle(center_x, center_y, (angle0 + angle1) / 2.0, rx, ry);
        self.line_to_pt(mid);
        let end = point_on_circle(center_x, center_y, angle1, rx, ry);
        self.line_to_pt(end);
    }

    /// Convert to polygon points (for filled shapes).
    pub fn to_polygon(&self) -> Vec<(f64, f64)> {
        self.points.clone()
    }

    /// Convert to UPath-style segments (moveTo + lineTos).
    pub fn to_path_segments(&self) -> Vec<PathSegment> {
        let mut segs = Vec::new();
        for (i, &(x, y)) in self.points.iter().enumerate() {
            if i == 0 {
                segs.push(PathSegment::MoveTo(x, y));
            } else {
                segs.push(PathSegment::LineTo(x, y));
            }
        }
        segs
    }

    /// Consume self, return the RNG for reuse.
    pub fn into_rng(self) -> JavaRandom {
        self.rnd
    }

    /// Return reference to RNG (for passing to further jiggle operations
    /// without consuming self).
    pub fn rng_mut(&mut self) -> &mut JavaRandom {
        &mut self.rnd
    }
}

fn point_on_circle(center_x: f64, center_y: f64, angle: f64, rx: f64, ry: f64) -> (f64, f64) {
    let x = center_x + angle.cos() * rx;
    let y = center_y + angle.sin() * ry;
    (x, y)
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    MoveTo(f64, f64),
    LineTo(f64, f64),
}

// ── Shape hand-drawing converters ────────────────────────────────────

/// Convert a rectangle to a hand-drawn polygon.
/// Port of Java `URectangleHand`.
pub fn rect_to_hand_polygon(
    width: f64,
    height: f64,
    rx: f64,
    ry: f64,
    rnd: &mut JavaRandom,
) -> Vec<(f64, f64)> {
    let rx = (rx / 2.0).min(width / 2.0);
    let ry = (ry / 2.0).min(height / 2.0);

    // We need to pass the RNG to HandJiggle, but we still need it after.
    // Use a temporary clone approach: consume the jiggle and get back the RNG.
    // Actually, HandJiggle takes ownership. We'll create inline.
    let points;

    if rx == 0.0 && ry == 0.0 {
        let mut jiggle = HandJiggle {
            points: vec![(0.0, 0.0)],
            start_x: 0.0,
            start_y: 0.0,
            default_variation: 1.5,
            rnd: take_rng(rnd),
        };
        jiggle.line_to(width, 0.0);
        jiggle.line_to(width, height);
        jiggle.line_to(0.0, height);
        jiggle.line_to(0.0, 0.0);
        points = jiggle.to_polygon();
        *rnd = jiggle.into_rng();
    } else {
        let mut jiggle = HandJiggle {
            points: vec![(rx, 0.0)],
            start_x: rx,
            start_y: 0.0,
            default_variation: 1.5,
            rnd: take_rng(rnd),
        };
        jiggle.line_to(width - rx, 0.0);
        jiggle.arc_to(-std::f64::consts::FRAC_PI_2, 0.0, width - rx, ry, rx, ry);
        jiggle.line_to(width, height - ry);
        jiggle.arc_to(
            0.0,
            std::f64::consts::FRAC_PI_2,
            width - rx,
            height - ry,
            rx,
            ry,
        );
        jiggle.line_to(rx, height);
        jiggle.arc_to(
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
            rx,
            height - ry,
            rx,
            ry,
        );
        jiggle.line_to(0.0, ry);
        jiggle.arc_to(
            std::f64::consts::PI,
            3.0 * std::f64::consts::FRAC_PI_2,
            rx,
            ry,
            rx,
            ry,
        );
        points = jiggle.to_polygon();
        *rnd = jiggle.into_rng();
    }

    points
}

/// Convert a line to a hand-drawn path.
/// Port of Java `ULineHand`.
pub fn line_to_hand_path(end_x: f64, end_y: f64, rnd: &mut JavaRandom) -> Vec<PathSegment> {
    let mut jiggle = HandJiggle {
        points: vec![(0.0, 0.0)],
        start_x: 0.0,
        start_y: 0.0,
        default_variation: 2.0,
        rnd: take_rng(rnd),
    };
    jiggle.line_to(end_x, end_y);
    let segs = jiggle.to_path_segments();
    *rnd = jiggle.into_rng();
    segs
}

/// Re-jiggle polygon points for hand-drawn effect.
/// Port of Java `UPolygonHand`.
pub fn polygon_to_hand(points: &[(f64, f64)], rnd: &mut JavaRandom) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut jiggle = HandJiggle::create(points[0], 1.5, take_rng(rnd));
    for &pt in &points[1..] {
        jiggle.line_to_pt(pt);
    }
    jiggle.line_to_pt(points[0]);
    let result = jiggle.to_polygon();
    *rnd = jiggle.into_rng();
    result
}

/// Helper: take the RNG state, leaving a dummy in its place.
/// This is needed because HandJiggle takes ownership.
fn take_rng(rnd: &mut JavaRandom) -> JavaRandom {
    let seed = rnd.seed;
    JavaRandom { seed }
}

// ── SVG formatting helpers ──────────────────────────────────────────

use crate::klimt::svg::fmt_coord;

/// Format polygon points as SVG `points` attribute value.
pub fn polygon_points_svg(points: &[(f64, f64)]) -> String {
    points
        .iter()
        .map(|(x, y)| format!("{},{}", fmt_coord(*x), fmt_coord(*y)))
        .collect::<Vec<_>>()
        .join(",")
}

/// Format path segments as SVG `d` attribute value.
pub fn path_d_svg(segments: &[PathSegment]) -> String {
    let mut parts = Vec::new();
    for seg in segments {
        match seg {
            PathSegment::MoveTo(x, y) => {
                parts.push(format!("M{},{}", fmt_coord(*x), fmt_coord(*y)));
            }
            PathSegment::LineTo(x, y) => {
                parts.push(format!("L{},{}", fmt_coord(*x), fmt_coord(*y)));
            }
        }
    }
    parts.join(" ")
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_random_sequence() {
        let mut rnd = JavaRandom::new(424242);
        // Values from Java's Random(424242L).nextDouble()
        let expected = [
            0.3598786908134424,
            0.7762135801574173,
            0.3711423630848399,
            0.2335470217250903,
            0.9755965438203709,
            0.5518086714513357,
            0.7843291403286345,
            0.7556659328585267,
            0.5454377095706181,
            0.9609126174802709,
        ];
        for &exp in &expected {
            let got = rnd.next_double();
            assert!((got - exp).abs() < 1e-15, "expected {exp}, got {got}");
        }
    }

    #[test]
    fn rect_hand_is_deterministic() {
        let mut rnd1 = JavaRandom::new(424242);
        let poly1 = rect_to_hand_polygon(100.0, 50.0, 0.0, 0.0, &mut rnd1);
        let mut rnd2 = JavaRandom::new(424242);
        let poly2 = rect_to_hand_polygon(100.0, 50.0, 0.0, 0.0, &mut rnd2);
        assert_eq!(poly1.len(), poly2.len());
        for i in 0..poly1.len() {
            assert_eq!(poly1[i].0, poly2[i].0);
            assert_eq!(poly1[i].1, poly2[i].1);
        }
    }

    #[test]
    fn line_hand_produces_path() {
        let mut rnd = JavaRandom::new(424242);
        let segs = line_to_hand_path(50.0, 0.0, &mut rnd);
        // First segment should be MoveTo(0,0)
        assert!(matches!(segs[0], PathSegment::MoveTo(0.0, 0.0)));
        // Should have multiple segments (jiggled)
        assert!(segs.len() > 2);
    }

    #[test]
    fn polygon_hand_rejiggle() {
        let square = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let mut rnd = JavaRandom::new(424242);
        let jiggled = polygon_to_hand(&square, &mut rnd);
        // Should have more points than original (jiggled along edges)
        assert!(jiggled.len() > square.len());
    }
}
