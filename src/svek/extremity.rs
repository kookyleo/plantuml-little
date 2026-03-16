// svek::extremity - Arrow endpoint shapes
// Port of Java PlantUML's svek.extremity package (52 files)
//
// Each Extremity draws a small shape at the end of an edge:
// arrows, diamonds, circles, crowfeet, etc.

use crate::klimt::geom::XPoint2D;
use crate::klimt::UTranslate;

/// Base trait for arrow endpoint shapes.
/// Java: `svek.extremity.Extremity`
pub trait Extremity {
    /// Draw this extremity using the given UGraphic context.
    fn draw(&self, ug: &mut dyn crate::klimt::UGraphic);

    /// A reference point on this extremity (for layout calculations).
    fn some_point(&self) -> XPoint2D;

    /// Length of the decoration along the edge direction.
    fn decoration_length(&self) -> f64 { 8.0 }

    /// Delta for Kal edge adjustment.
    fn delta_for_kal(&self) -> UTranslate { UTranslate::none() }
}

/// Round an angle to nearest cardinal direction (0, 90, 180, 270) if very close.
/// Java: `Extremity.manageround()`
pub fn manage_round(angle: f64) -> f64 {
    let deg = angle * 180.0 / std::f64::consts::PI;
    for &cardinal in &[0.0, 90.0, 180.0, 270.0, 360.0] {
        if (cardinal - deg).abs() < 0.05 {
            return if cardinal == 360.0 { 0.0 } else { cardinal * std::f64::consts::PI / 180.0 };
        }
    }
    angle
}

/// Factory for creating extremities at a given point and angle.
/// Java: `svek.extremity.ExtremityFactory`
pub trait ExtremityFactory {
    fn create(&self, point: XPoint2D, angle: f64) -> Box<dyn Extremity>;
}

// ── Extremity types (stubs for parallel filling) ─────────────────────

// TODO: Each extremity type will be filled by the parallel agent.
// Types needed: Arrow, Diamond, Circle, Extends, Crowfoot, Plus,
// Triangle, Square, DoubleLine, NotNavigable, Parenthesis,
// HalfArrow, CircleCross, CircleLine, CircleConnect, etc.
// Also: MiddleCircle, MiddleSubset for edge midpoint decorations.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manage_round_cardinal() {
        let pi = std::f64::consts::PI;
        assert!((manage_round(0.0001) - 0.0).abs() < 1e-9);
        assert!((manage_round(pi / 2.0 + 0.0001) - pi / 2.0).abs() < 1e-9);
        assert!((manage_round(pi - 0.0001) - pi).abs() < 1e-9);
    }

    #[test]
    fn manage_round_non_cardinal() {
        let angle = 0.7854; // ~45 degrees
        assert!((manage_round(angle) - angle).abs() < 1e-9); // unchanged
    }
}
