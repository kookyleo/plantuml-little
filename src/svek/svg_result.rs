// svek::svg_result - Graphviz SVG output parser
// Port of Java PlantUML's svek.SvgResult + PointListIterator + PointListIteratorImpl

use crate::klimt::geom::XPoint2D;
use crate::svek::Point2DFunction;
use std::rc::Rc;

// ── Constants ────────────────────────────────────────────────────────

pub const D_EQUALS: &str = "d=\"";
pub const POINTS_EQUALS: &str = "points=\"";

// ── SvgResult ────────────────────────────────────────────────────────

/// Parsed Graphviz SVG output with coordinate extraction utilities.
/// Java: `svek.SvgResult`
///
/// Wraps an SVG string slice and a coordinate-transform function.
/// Provides substring/indexOf operations that propagate the transform,
/// and methods to extract point lists for node/edge positions.
pub struct SvgResult {
    svg: String,
    function: Rc<dyn Point2DFunction>,
}

impl SvgResult {
    pub fn new(svg: String) -> Self {
        Self::with_shared_function(svg, Rc::new(crate::svek::IdentityFunction))
    }

    pub fn with_function(svg: String, function: Box<dyn Point2DFunction>) -> Self {
        Self::with_shared_function(svg, Rc::from(function))
    }

    fn with_shared_function(svg: String, function: Rc<dyn Point2DFunction>) -> Self {
        Self { svg, function }
    }

    pub fn svg(&self) -> &str {
        &self.svg
    }

    // ── String operations (Java: indexOf, substring) ─────────────

    /// Find index of `needle` starting from `pos`.
    /// Java: `SvgResult.indexOf(String, int)`
    pub fn index_of(&self, needle: &str, pos: usize) -> Option<usize> {
        if pos > self.svg.len() {
            return None;
        }
        self.svg[pos..].find(needle).map(|i| i + pos)
    }

    /// Create a sub-result from `pos` to end, keeping the transform.
    /// Java: `SvgResult.substring(int)`
    pub fn substring_from(&self, pos: usize) -> SvgResult {
        let s = if pos >= self.svg.len() {
            String::new()
        } else {
            self.svg[pos..].to_string()
        };
        SvgResult::with_shared_function(s, self.function.clone())
    }

    /// Create a sub-result from `start` to `end`, keeping the transform.
    /// Java: `SvgResult.substring(int, int)`
    pub fn substring(&self, start: usize, end: usize) -> SvgResult {
        let s = if start >= self.svg.len() || start >= end {
            String::new()
        } else {
            let actual_end = end.min(self.svg.len());
            self.svg[start..actual_end].to_string()
        };
        SvgResult::with_shared_function(s, self.function.clone())
    }

    // ── Color lookup ─────────────────────────────────────────────────

    /// Find the SVG index of an element with the given stroke/fill color.
    /// Java: `SvgResult.getIndexFromColor(int)`
    pub fn get_index_from_color(&self, color: u32) -> Option<usize> {
        let hex = format!("#{:06x}", color);

        // Try stroke="..."
        let needle1 = format!("stroke=\"{}\"", hex);
        if let Some(idx) = self.svg.find(&needle1) {
            return Some(idx);
        }

        // Try ;stroke:...;
        let needle2 = format!(";stroke:{};", hex);
        if let Some(idx) = self.svg.find(&needle2) {
            return Some(idx);
        }

        // Try fill="..."
        let needle3 = format!("fill=\"{}\"", hex);
        if let Some(idx) = self.svg.find(&needle3) {
            return Some(idx);
        }

        None
    }

    /// Convenience alias matching existing API.
    pub fn find_by_color(&self, color: u32) -> Option<usize> {
        self.get_index_from_color(color)
    }

    // ── Point extraction ─────────────────────────────────────────────

    /// Create a PointListIterator that yields point lists for elements
    /// with the given stroke color.
    /// Java: `SvgResult.getPointsWithThisColor(int)`
    pub fn get_points_with_this_color(&self, line_color: u32) -> PointListIterator {
        PointListIterator::create(self, line_color)
    }

    /// Extract a point list from the first occurrence of `searched` attribute.
    /// Java: `SvgResult.extractList(String)`
    ///
    /// Finds `searched` (e.g., `points="`), reads until closing `"`,
    /// parses the coordinate string with " MC" as separators.
    pub fn extract_list(&self, searched: &str) -> Vec<XPoint2D> {
        let Some(p2) = self.index_of(searched, 0) else {
            return Vec::new();
        };
        let after = p2 + searched.len();
        let Some(p3) = self.index_of("\"", after) else {
            return Vec::new();
        };
        let sub = self.substring(after, p3);
        sub.get_points(" MC")
    }

    /// Extract coordinate points from a `points="..."` or `d="..."` attribute.
    /// Convenience wrapper over extract_list.
    pub fn extract_points(&self, searched: &str) -> Vec<XPoint2D> {
        self.extract_list(searched)
    }

    /// Parse the SVG string as a list of coordinate pairs, splitting on
    /// characters in `separator`.
    /// Java: `SvgResult.getPoints(String)`
    pub fn get_points(&self, separator: &str) -> Vec<XPoint2D> {
        let mut result = Vec::new();
        for token in split_by_chars(&self.svg, separator) {
            if let Some(pt) = self.parse_first_point(token) {
                result.push(pt);
            }
        }
        result
    }

    /// Parse the first coordinate pair from the SVG string.
    /// Java: `SvgResult.getNextPoint()`
    pub fn get_next_point(&self) -> Option<XPoint2D> {
        self.parse_first_point(&self.svg)
    }

    /// Parse "x,y" from a string, applying the transform function.
    /// Java: `SvgResult.getFirstPoint(String)`
    fn parse_first_point(&self, s: &str) -> Option<XPoint2D> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() < 2 {
            return None;
        }
        let x: f64 = parts[0].trim().parse().ok()?;
        let y: f64 = parts[1].trim().parse().ok()?;
        Some(self.function.apply(XPoint2D::new(x, y)))
    }

    // ── DotPath extraction ───────────────────────────────────────────

    /// Convert a path string starting with "M" into a DotPath.
    /// Java: `SvgResult.toDotPath()`
    pub fn to_dot_path(&self) -> Option<crate::klimt::shape::DotPath> {
        if !self.is_path_consistent() {
            return None;
        }
        parse_svg_path_to_dotpath_with_fn(&self.svg, &*self.function)
    }

    /// Check path consistency (must start with "M").
    /// Java: `SvgResult.isPathConsistent()`
    pub fn is_path_consistent(&self) -> bool {
        self.svg.starts_with('M')
    }

    /// Extract point list from a `points="..."` attribute searching from `from`.
    /// Java: used by `SvekEdge.getLabelXY()` and node position extraction.
    pub fn extract_points_at(&self, from: usize) -> Vec<XPoint2D> {
        if from >= self.svg.len() {
            return Vec::new();
        }
        let search_str = &self.svg[from..];
        let Some(points_pos) = search_str.find(POINTS_EQUALS) else {
            return Vec::new();
        };
        let after = from + points_pos + POINTS_EQUALS.len();
        let Some(end_quote) = self.svg[after..].find('"') else {
            return Vec::new();
        };
        let sub = self.substring(after, after + end_quote);
        sub.get_points(" MC")
    }

    /// Extract DotPath from SVG path `d` attribute at given position.
    pub fn extract_dot_path(&self, from: usize) -> Option<(crate::klimt::shape::DotPath, usize)> {
        let d_start = self.svg[from..].find("d=\"")?;
        let d_pos = from + d_start + 3;
        let d_end = self.svg[d_pos..].find('"')?;
        let d_str = &self.svg[d_pos..d_pos + d_end];
        let path = parse_svg_path_to_dotpath_with_fn(d_str, &*self.function)?;
        Some((path, d_pos + d_end))
    }
}

// ── PointListIterator ────────────────────────────────────────────────

/// Sequential iterator that yields point lists from SVG `points="..."` attributes.
/// Java: `svek.PointListIterator` + `svek.PointListIteratorImpl`
///
/// Starting from a color-matched position, each call to `next()` finds the next
/// `points="..."` attribute and parses its coordinates.
pub struct PointListIterator {
    /// The full SVG string being searched.
    svg_text: String,
    /// Coordinate transform shared with the parent SvgResult.
    function: Rc<dyn Point2DFunction>,
    /// Current search position: >= 0 means active, -1 means color not found, -2 means exhausted.
    pos: i64,
}

impl PointListIterator {
    /// Create an iterator starting from the position of `line_color` in the SVG.
    /// Java: `PointListIteratorImpl.create(SvgResult, int)`
    ///
    /// Java creates a substring from the color position, so we store the
    /// color index as the starting search position.
    pub fn create(svg_result: &SvgResult, line_color: u32) -> Self {
        let pos = match svg_result.get_index_from_color(line_color) {
            Some(idx) => idx as i64,
            None => -1,
        };
        Self {
            svg_text: svg_result.svg.clone(),
            function: svg_result.function.clone(),
            pos,
        }
    }

    /// Whether more point lists can be extracted.
    /// Java: `PointListIteratorImpl.hasNext()`
    pub fn has_next(&self) -> bool {
        self.pos != -2
    }

    /// Clone the iterator state.
    /// Java: `PointListIterator.cloneMe()`
    pub fn clone_me(&self) -> Self {
        Self {
            svg_text: self.svg_text.clone(),
            function: self.function.clone(),
            pos: self.pos,
        }
    }
}

impl Iterator for PointListIterator {
    type Item = Vec<XPoint2D>;

    /// Extract the next point list from the SVG.
    /// Java: `PointListIteratorImpl.next()`
    fn next(&mut self) -> Option<Vec<XPoint2D>> {
        if self.pos == -2 {
            return None;
        }
        if self.pos < 0 {
            self.pos = -2;
            return Some(Vec::new());
        }

        let pos = self.pos as usize;

        // Build a temporary SvgResult for the substring
        let sub_svg = if pos < self.svg_text.len() {
            self.svg_text[pos..].to_string()
        } else {
            self.pos = -2;
            return Some(Vec::new());
        };
        let sub = SvgResult::with_shared_function(sub_svg, self.function.clone());
        let result = sub.extract_list(POINTS_EQUALS);

        if result.is_empty() {
            self.pos = -2;
        } else {
            // Advance past this points="..." attribute
            match self.svg_text[pos..].find(POINTS_EQUALS) {
                Some(offset) => {
                    self.pos = (pos + offset + POINTS_EQUALS.len() + 1) as i64;
                }
                None => {
                    self.pos = -2;
                }
            }
        }
        Some(result)
    }
}

// ── Helper functions ─────────────────────────────────────────────────

/// Split a string by any character in `chars` (like Java's StringTokenizer).
fn split_by_chars<'a>(s: &'a str, chars: &str) -> Vec<&'a str> {
    s.split(|c: char| chars.contains(c))
        .filter(|t| !t.is_empty())
        .collect()
}

/// Parse SVG coordinate string into points (identity transform).
/// Handles formats: "x1,y1 x2,y2 ..." and "M x y C x1 y1 x2 y2 x y ..."
#[allow(dead_code)] // reserved for polygon point parsing
fn parse_points(s: &str) -> Vec<XPoint2D> {
    let mut points = Vec::new();
    let clean = s.replace(',', " ");
    let nums: Vec<f64> = clean
        .split_whitespace()
        .filter_map(|t| {
            // Skip path commands (single alphabetic characters)
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
#[allow(dead_code)] // convenience wrapper for identity transform
fn parse_svg_path_to_dotpath(d: &str) -> Option<crate::klimt::shape::DotPath> {
    parse_svg_path_to_dotpath_with_fn(d, &crate::svek::IdentityFunction)
}

/// Public: parse SVG path d-string into DotPath for cluster border clipping.
pub fn parse_svg_path_d_to_dotpath(d: &str) -> Option<crate::klimt::shape::DotPath> {
    parse_svg_path_to_dotpath_with_fn(d, &crate::svek::IdentityFunction)
}

/// Parse SVG path `d` attribute into a DotPath, applying a coordinate transform.
/// Java: `SvgResult.toDotPath()` logic
///
/// Handles Graphviz SVG format where commands and numbers are concatenated
/// without spaces (e.g., `M36,-85.7C36,-74.56 36,-61.38 36,-50.24`).
fn parse_svg_path_to_dotpath_with_fn(
    d: &str,
    function: &dyn Point2DFunction,
) -> Option<crate::klimt::shape::DotPath> {
    let mut beziers = Vec::new();
    let mut current_x = 0.0_f64;
    let mut current_y = 0.0_f64;
    let mut nums = Vec::new();
    let mut cmd = ' ';

    // Tokenize: split into commands (single letter) and numbers,
    // handling concatenated format like "M36,-85.7C36,-74.56"
    let tokens = tokenize_svg_path(d);

    for tok in &tokens {
        if tok.len() == 1
            && tok
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_alphabetic())
        {
            // Process accumulated numbers before switching command
            process_path_cmd(
                cmd,
                &mut nums,
                &mut beziers,
                &mut current_x,
                &mut current_y,
                function,
            );
            cmd = tok.chars().next().unwrap();
            continue;
        }
        if let Ok(v) = tok.parse::<f64>() {
            nums.push(v);
        }
        // Process immediately when enough numbers for current command
        process_path_cmd(
            cmd,
            &mut nums,
            &mut beziers,
            &mut current_x,
            &mut current_y,
            function,
        );
    }
    // Process any remaining numbers
    process_path_cmd(
        cmd,
        &mut nums,
        &mut beziers,
        &mut current_x,
        &mut current_y,
        function,
    );

    if beziers.is_empty() {
        None
    } else {
        Some(crate::klimt::shape::DotPath::from_beziers(beziers))
    }
}

fn process_path_cmd(
    cmd: char,
    nums: &mut Vec<f64>,
    beziers: &mut Vec<crate::klimt::geom::XCubicCurve2D>,
    current_x: &mut f64,
    current_y: &mut f64,
    function: &dyn Point2DFunction,
) {
    use crate::klimt::geom::XCubicCurve2D;
    match cmd {
        'M' if nums.len() >= 2 => {
            let pt = function.apply(XPoint2D::new(nums[0], nums[1]));
            *current_x = pt.x;
            *current_y = pt.y;
            nums.drain(..2);
        }
        'C' if nums.len() >= 6 => {
            let p1 = function.apply(XPoint2D::new(nums[0], nums[1]));
            let p2 = function.apply(XPoint2D::new(nums[2], nums[3]));
            let p3 = function.apply(XPoint2D::new(nums[4], nums[5]));
            beziers.push(XCubicCurve2D::new(
                *current_x, *current_y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y,
            ));
            *current_x = p3.x;
            *current_y = p3.y;
            nums.drain(..6);
        }
        'L' if nums.len() >= 2 => {
            let pt = function.apply(XPoint2D::new(nums[0], nums[1]));
            beziers.push(XCubicCurve2D::new(
                *current_x, *current_y, *current_x, *current_y, pt.x, pt.y, pt.x, pt.y,
            ));
            *current_x = pt.x;
            *current_y = pt.y;
            nums.drain(..2);
        }
        _ => {}
    }
}

/// Tokenize SVG path d-string into commands and numbers.
/// Handles concatenated format: "M36,-85.7C36,-74.56" → ["M", "36", "-85.7", "C", "36", "-74.56"]
fn tokenize_svg_path(d: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut num_buf = String::new();
    let mut chars = d.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_alphabetic() {
            // Flush number buffer
            if !num_buf.is_empty() {
                tokens.push(std::mem::take(&mut num_buf));
            }
            tokens.push(ch.to_string());
            chars.next();
        } else if ch == '-'
            && !num_buf.is_empty()
            && !num_buf.ends_with('e')
            && !num_buf.ends_with('E')
        {
            // Negative sign starts a new number (unless after exponent)
            tokens.push(std::mem::take(&mut num_buf));
            num_buf.push(ch);
            chars.next();
        } else if ch == ',' || ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            if !num_buf.is_empty() {
                tokens.push(std::mem::take(&mut num_buf));
            }
            chars.next();
        } else {
            num_buf.push(ch);
            chars.next();
        }
    }
    if !num_buf.is_empty() {
        tokens.push(num_buf);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_points ─────────────────────────────────────────────────

    #[test]
    fn parse_points_basic() {
        let pts = parse_points("10,20 30,40 50,60");
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], XPoint2D::new(10.0, 20.0));
        assert_eq!(pts[1], XPoint2D::new(30.0, 40.0));
        assert_eq!(pts[2], XPoint2D::new(50.0, 60.0));
    }

    #[test]
    fn parse_points_with_path_commands() {
        let pts = parse_points("M 10,20 C 30,40 50,60 70,80");
        assert_eq!(pts.len(), 4);
        assert_eq!(pts[0], XPoint2D::new(10.0, 20.0));
        assert_eq!(pts[3], XPoint2D::new(70.0, 80.0));
    }

    // ── parse_svg_path_to_dotpath ────────────────────────────────────

    #[test]
    fn parse_svg_path_cubic() {
        let path = parse_svg_path_to_dotpath("M 0,0 C 10,0 20,10 30,20").unwrap();
        assert_eq!(path.beziers.len(), 1);
        assert_eq!(path.start_point(), XPoint2D::new(0.0, 0.0));
        assert_eq!(path.end_point(), XPoint2D::new(30.0, 20.0));
    }

    #[test]
    fn parse_svg_path_multiple_cubics() {
        let path = parse_svg_path_to_dotpath("M 0,0 C 1,2 3,4 5,6 C 7,8 9,10 11,12");
        assert!(path.is_some());
    }

    #[test]
    fn parse_svg_path_line() {
        let path = parse_svg_path_to_dotpath("M 0,0 L 10,20").unwrap();
        assert_eq!(path.beziers.len(), 1);
        assert_eq!(path.start_point(), XPoint2D::new(0.0, 0.0));
        assert_eq!(path.end_point(), XPoint2D::new(10.0, 20.0));
    }

    #[test]
    fn parse_svg_path_empty() {
        assert!(parse_svg_path_to_dotpath("").is_none());
        assert!(parse_svg_path_to_dotpath("M 0,0").is_none());
    }

    // ── SvgResult basic operations ───────────────────────────────────

    #[test]
    fn svg_result_index_of() {
        let sr = SvgResult::new("hello world foo".to_string());
        assert_eq!(sr.index_of("world", 0), Some(6));
        assert_eq!(sr.index_of("world", 7), None);
        assert_eq!(sr.index_of("foo", 0), Some(12));
        assert_eq!(sr.index_of("bar", 0), None);
    }

    #[test]
    fn svg_result_substring() {
        let sr = SvgResult::new("abcdefgh".to_string());
        let sub = sr.substring(2, 5);
        assert_eq!(sub.svg(), "cde");
    }

    #[test]
    fn svg_result_substring_from() {
        let sr = SvgResult::new("abcdefgh".to_string());
        let sub = sr.substring_from(3);
        assert_eq!(sub.svg(), "defgh");
    }

    #[test]
    fn svg_result_substring_preserves_transform() {
        use crate::svek::snake::YDelta;

        let sr = SvgResult::with_function("x 10,20 30,40".to_string(), Box::new(YDelta::new(100.0)));
        let sub = sr.substring_from(2);
        let pts = sub.get_points(" ");
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0], XPoint2D::new(10.0, 120.0));
        assert_eq!(pts[1], XPoint2D::new(30.0, 140.0));
    }

    // ── SvgResult find_by_color ──────────────────────────────────────

    #[test]
    fn svg_result_find_by_color_stroke_attr() {
        let svg = r##"<line stroke="#010200" x1="10" y1="20"/>"##;
        let sr = SvgResult::new(svg.to_string());
        assert!(sr.find_by_color(0x010200).is_some());
        assert!(sr.find_by_color(0xFF0000).is_none());
    }

    #[test]
    fn svg_result_find_by_color_stroke_style() {
        let svg = r##"<path style="fill:none;stroke:#abcdef;stroke-width:1"/>"##;
        let sr = SvgResult::new(svg.to_string());
        assert!(sr.find_by_color(0xABCDEF).is_some());
    }

    #[test]
    fn svg_result_find_by_color_fill_attr() {
        let svg = r##"<rect fill="#112233" width="10" height="10"/>"##;
        let sr = SvgResult::new(svg.to_string());
        assert!(sr.find_by_color(0x112233).is_some());
    }

    // ── SvgResult extract_points / extract_list ──────────────────────

    #[test]
    fn svg_result_extract_points() {
        let svg = r#"<polygon points="10,20 30,40 50,60"/>"#;
        let sr = SvgResult::new(svg.to_string());
        let pts = sr.extract_points(POINTS_EQUALS);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], XPoint2D::new(10.0, 20.0));
        assert_eq!(pts[1], XPoint2D::new(30.0, 40.0));
        assert_eq!(pts[2], XPoint2D::new(50.0, 60.0));
    }

    #[test]
    fn svg_result_extract_list_d() {
        let svg = r#"<path d="M10,20 C30,40 50,60 70,80"/>"#;
        let sr = SvgResult::new(svg.to_string());
        let pts = sr.extract_list(D_EQUALS);
        // Splits by " MC" chars: tokens are "10,20", "30,40", "50,60", "70,80"
        assert_eq!(pts.len(), 4);
    }

    #[test]
    fn svg_result_extract_list_not_found() {
        let sr = SvgResult::new("<svg></svg>".to_string());
        assert!(sr.extract_list(POINTS_EQUALS).is_empty());
    }

    // ── SvgResult get_points ─────────────────────────────────────────

    #[test]
    fn svg_result_get_points_space_separator() {
        let sr = SvgResult::new("10,20 30,40 50,60".to_string());
        let pts = sr.get_points(" ");
        assert_eq!(pts.len(), 3);
    }

    #[test]
    fn svg_result_get_points_mc_separator() {
        let sr = SvgResult::new("10,20M30,40C50,60".to_string());
        let pts = sr.get_points(" MC");
        assert_eq!(pts.len(), 3);
    }

    #[test]
    fn svg_result_get_next_point() {
        let sr = SvgResult::new("42.5,99.1".to_string());
        let pt = sr.get_next_point().unwrap();
        assert!((pt.x - 42.5).abs() < 1e-10);
        assert!((pt.y - 99.1).abs() < 1e-10);
    }

    // ── SvgResult to_dot_path ────────────────────────────────────────

    #[test]
    fn svg_result_to_dot_path() {
        // Use parse_svg_path_to_dotpath directly since SvgResult.to_dot_path
        // requires path consistency checks
        let dp = super::parse_svg_path_to_dotpath("M 0,0 C 10,0 20,10 30,20").unwrap();
        assert_eq!(dp.beziers.len(), 1);
        assert_eq!(dp.start_point(), XPoint2D::new(0.0, 0.0));
        assert_eq!(dp.end_point(), XPoint2D::new(30.0, 20.0));
    }

    #[test]
    fn svg_result_is_path_consistent() {
        let sr1 = SvgResult::new("M0,0 C1,2 3,4 5,6".to_string());
        assert!(sr1.is_path_consistent());

        let sr2 = SvgResult::new("C1,2 3,4 5,6".to_string());
        assert!(!sr2.is_path_consistent());
    }

    // ── PointListIterator ────────────────────────────────────────────

    #[test]
    fn point_list_iterator_basic() {
        let svg = concat!(
            r##"<g stroke="#010200">"##,
            r#"<polygon points="10,20 30,40"/>"#,
            r#"<polygon points="50,60 70,80"/>"#,
            r#"</g>"#,
        );
        let sr = SvgResult::new(svg.to_string());
        let mut iter = sr.get_points_with_this_color(0x010200);
        assert!(iter.has_next());

        let first = iter.next().unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!(first[0], XPoint2D::new(10.0, 20.0));
        assert_eq!(first[1], XPoint2D::new(30.0, 40.0));

        let second = iter.next().unwrap();
        assert_eq!(second.len(), 2);
        assert_eq!(second[0], XPoint2D::new(50.0, 60.0));
    }

    #[test]
    fn point_list_iterator_no_color() {
        let svg = r#"<polygon points="10,20 30,40"/>"#;
        let sr = SvgResult::new(svg.to_string());
        let mut iter = sr.get_points_with_this_color(0xFF0000);
        assert!(iter.has_next());
        // First call returns empty since color not found
        let first = iter.next().unwrap();
        assert!(first.is_empty());
        // Now exhausted
        assert!(!iter.has_next());
    }

    #[test]
    fn point_list_iterator_clone() {
        let svg = concat!(
            r##"<g stroke="#020400">"##,
            r#"<polygon points="1,2 3,4"/>"#,
            r#"</g>"#,
        );
        let sr = SvgResult::new(svg.to_string());
        let iter = sr.get_points_with_this_color(0x020400);
        let cloned = iter.clone_me();
        assert_eq!(iter.pos, cloned.pos);
    }

    #[test]
    fn point_list_iterator_exhaustion() {
        let svg = r#"<polygon points="1,2"/>"#;
        let sr = SvgResult::new(svg.to_string());
        let mut iter = sr.get_points_with_this_color(0x010200);
        // Color not found => first next returns empty, then exhausted
        let _ = iter.next();
        assert!(!iter.has_next());
        assert!(iter.next().is_none());
    }

    #[test]
    fn point_list_iterator_preserves_transform() {
        use crate::svek::snake::YDelta;

        let svg = concat!(
            r##"<g stroke="#010200">"##,
            r#"<polygon points="10,20 30,40"/>"#,
            r#"</g>"#,
        );
        let sr = SvgResult::with_function(svg.to_string(), Box::new(YDelta::new(100.0)));
        let mut iter = sr.get_points_with_this_color(0x010200);
        let first = iter.next().unwrap();
        assert_eq!(first, vec![XPoint2D::new(10.0, 120.0), XPoint2D::new(30.0, 140.0)]);
    }

    // ── split_by_chars ───────────────────────────────────────────────

    #[test]
    fn split_by_chars_basic() {
        let result = split_by_chars("aMbCc", " MC");
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn split_by_chars_spaces() {
        let result = split_by_chars("10,20 30,40 50,60", " ");
        assert_eq!(result, vec!["10,20", "30,40", "50,60"]);
    }

    #[test]
    fn split_by_chars_empty() {
        let result = split_by_chars("", " MC");
        assert!(result.is_empty());
    }

    // ── with_function / YDelta integration ───────────────────────────

    #[test]
    fn svg_result_with_ydelta() {
        use crate::svek::snake::YDelta;

        let sr = SvgResult::with_function("10,20 30,40".to_string(), Box::new(YDelta::new(100.0)));
        let pts = sr.get_points(" ");
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0].x, 10.0);
        assert_eq!(pts[0].y, 120.0); // 20 + 100
        assert_eq!(pts[1].x, 30.0);
        assert_eq!(pts[1].y, 140.0); // 40 + 100
    }
}
