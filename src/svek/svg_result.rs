// svek::svg_result - Graphviz SVG output parser
// Port of Java PlantUML's svek.SvgResult + PointListIterator

use crate::klimt::geom::XPoint2D;

/// Parsed Graphviz SVG output with coordinate extraction utilities.
/// Java: `svek.SvgResult`
pub struct SvgResult {
    svg: String,
}

impl SvgResult {
    pub fn new(svg: String) -> Self {
        Self { svg }
    }

    pub fn svg(&self) -> &str {
        &self.svg
    }

    /// Find index of a string starting from `from`.
    pub fn index_of(&self, needle: &str, from: usize) -> Option<usize> {
        self.svg[from..].find(needle).map(|i| i + from)
    }

    /// Extract coordinate points from a `points="..."` or `d="..."` attribute.
    pub fn extract_points(&self, searched: &str) -> Vec<XPoint2D> {
        let Some(start) = self.svg.find(searched) else { return vec![] };
        let after = start + searched.len();
        let Some(end) = self.svg[after..].find('"') else { return vec![] };
        let coords_str = &self.svg[after..after + end];
        parse_points(coords_str)
    }

    /// Find elements by stroke color (used to match DOT nodes/edges).
    pub fn find_by_color(&self, color: u32) -> Option<usize> {
        let hex = format!("#{:06x}", color);
        let needle1 = format!("stroke=\"{}\"", hex);
        let needle2 = format!(";stroke:{};", hex);
        self.svg.find(&needle1).or_else(|| self.svg.find(&needle2))
    }

    /// Extract DotPath from SVG path `d` attribute at given position.
    pub fn extract_dot_path(&self, from: usize) -> Option<(crate::klimt::shape::DotPath, usize)> {
        let d_start = self.svg[from..].find("d=\"")?;
        let d_pos = from + d_start + 3;
        let d_end = self.svg[d_pos..].find('"')?;
        let d_str = &self.svg[d_pos..d_pos + d_end];
        let path = parse_svg_path_to_dotpath(d_str)?;
        Some((path, d_pos + d_end))
    }
}

/// Parse SVG coordinate string into points.
/// Handles formats: "x1,y1 x2,y2 ..." and "M x y C x1 y1 x2 y2 x y ..."
fn parse_points(s: &str) -> Vec<XPoint2D> {
    let mut points = Vec::new();
    let clean = s.replace(',', " ");
    let nums: Vec<f64> = clean.split_whitespace()
        .filter_map(|t| {
            // Skip path commands
            if t.len() == 1 && t.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) {
                return None;
            }
            t.parse::<f64>().ok()
        })
        .collect();
    for pair in nums.chunks(2) {
        if pair.len() == 2 {
            points.push(XPoint2D::new(pair[0], pair[1]));
        }
    }
    points
}

/// Parse SVG path `d` attribute into a DotPath (series of cubic beziers).
fn parse_svg_path_to_dotpath(d: &str) -> Option<crate::klimt::shape::DotPath> {
    use crate::klimt::geom::XCubicCurve2D;

    let mut beziers = Vec::new();
    let mut current_x = 0.0_f64;
    let mut current_y = 0.0_f64;
    let mut nums = Vec::new();
    let mut cmd = ' ';

    for token in d.split_whitespace() {
        if token.len() == 1 && token.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) {
            cmd = token.chars().next().unwrap();
            continue;
        }
        // Handle "x,y" format
        for part in token.split(',') {
            if let Ok(v) = part.parse::<f64>() {
                nums.push(v);
            }
        }

        match cmd {
            'M' if nums.len() >= 2 => {
                current_x = nums[0];
                current_y = nums[1];
                nums.clear();
            }
            'C' if nums.len() >= 6 => {
                beziers.push(XCubicCurve2D::new(
                    current_x, current_y,
                    nums[0], nums[1], nums[2], nums[3], nums[4], nums[5],
                ));
                current_x = nums[4];
                current_y = nums[5];
                nums.clear();
            }
            'L' if nums.len() >= 2 => {
                // Straight line as degenerate cubic
                beziers.push(XCubicCurve2D::new(
                    current_x, current_y,
                    current_x, current_y, nums[0], nums[1], nums[0], nums[1],
                ));
                current_x = nums[0];
                current_y = nums[1];
                nums.clear();
            }
            _ => {}
        }
    }

    if beziers.is_empty() {
        None
    } else {
        Some(crate::klimt::shape::DotPath::from_beziers(beziers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_points_basic() {
        let pts = parse_points("10,20 30,40 50,60");
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], XPoint2D::new(10.0, 20.0));
        assert_eq!(pts[2], XPoint2D::new(50.0, 60.0));
    }

    #[test]
    fn parse_svg_path_cubic() {
        let path = parse_svg_path_to_dotpath("M 0,0 C 10,0 20,10 30,20").unwrap();
        assert_eq!(path.beziers.len(), 1);
        assert_eq!(path.start_point(), XPoint2D::new(0.0, 0.0));
        assert_eq!(path.end_point(), XPoint2D::new(30.0, 20.0));
    }

    #[test]
    fn svg_result_find_by_color() {
        let svg = r##"<line stroke="#010200" x1="10" y1="20"/>"##;
        let sr = SvgResult::new(svg.to_string());
        assert!(sr.find_by_color(0x010200).is_some());
        assert!(sr.find_by_color(0xFF0000).is_none());
    }

    #[test]
    fn svg_result_extract_points() {
        let svg = r#"<polygon points="10,20 30,40 50,60"/>"#;
        let sr = SvgResult::new(svg.to_string());
        let pts = sr.extract_points("points=\"");
        assert_eq!(pts.len(), 3);
    }
}
