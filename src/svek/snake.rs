// svek::snake - Edge path rendering with S-curves
// Port of Java PlantUML's svek.UGraphicForSnake
//
// Wraps a UGraphic to add snake-line edge routing:
// edges that need to avoid obstacles get routed through
// horizontal/vertical segments with smooth S-curve transitions.

use super::Point2DFunction;
use crate::klimt::geom::XPoint2D;

/// Y-axis offset function. Java: `svek.YDelta`
#[derive(Debug, Clone)]
pub struct YDelta {
    pub delta: f64,
}

impl YDelta {
    pub fn new(delta: f64) -> Self {
        Self { delta }
    }
}

impl Point2DFunction for YDelta {
    fn apply(&self, pt: XPoint2D) -> XPoint2D {
        XPoint2D::new(pt.x, pt.y + self.delta)
    }
}

/// Oscillator for edge routing. Java: `svek.Oscillator`
#[derive(Debug)]
pub struct Oscillator {
    values: Vec<f64>,
}

impl Oscillator {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn add_value(&mut self, v: f64) {
        self.values.push(v);
    }

    pub fn get_value_at(&self, idx: usize) -> f64 {
        self.values.get(idx).copied().unwrap_or(0.0)
    }
}

// TODO: UGraphicForSnake, FrontierCalculator

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ydelta_apply() {
        let yd = YDelta::new(50.0);
        let pt = yd.apply(XPoint2D::new(10.0, 20.0));
        assert_eq!(pt.x, 10.0);
        assert_eq!(pt.y, 70.0);
    }

    #[test]
    fn oscillator_basic() {
        let mut o = Oscillator::new();
        o.add_value(1.0);
        o.add_value(2.0);
        assert_eq!(o.get_value_at(0), 1.0);
        assert_eq!(o.get_value_at(1), 2.0);
        assert_eq!(o.get_value_at(5), 0.0);
    }
}
