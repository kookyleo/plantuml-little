// klimt::shape - Drawing shape primitives
// Port of Java PlantUML's UShape implementations

use super::geom::USegment;

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
    pub fn new() -> Self { Self::default() }

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

    pub fn arc_to(&mut self, rx: f64, ry: f64, x_rot: f64, large_arc: f64, sweep: f64, x: f64, y: f64) {
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
                    if !d.is_empty() { d.push(' '); }
                    d.push_str(&format!("M{:.4} {:.4}", seg.coords[0], seg.coords[1]));
                }
                LineTo => d.push_str(&format!(" L{:.4} {:.4}", seg.coords[0], seg.coords[1])),
                CubicTo => d.push_str(&format!(
                    " C{:.4} {:.4} {:.4} {:.4} {:.4} {:.4}",
                    seg.coords[0], seg.coords[1], seg.coords[2], seg.coords[3], seg.coords[4], seg.coords[5]
                )),
                ArcTo => d.push_str(&format!(
                    " A{:.4} {:.4} {:.4} {} {} {:.4} {:.4}",
                    seg.coords[0], seg.coords[1], seg.coords[2],
                    seg.coords[3] as i32, seg.coords[4] as i32,
                    seg.coords[5], seg.coords[6]
                )),
                Close => d.push_str(" Z"),
            }
        }
        d
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
}
