// svek::edge - Graph edge representation for Graphviz layout
// Port of Java PlantUML's svek.SvekEdge (1350 lines)

use std::fmt;

use crate::klimt::geom::{XDimension2D, XPoint2D};
use crate::klimt::shape::DotPath;

// ── Direction ───────────────────────────────────────────────────────

/// Cardinal direction for arrow pointing.
/// Java: `utils.Direction`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Return the inverse direction.
    pub fn inv(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

// ── LinkArrow ───────────────────────────────────────────────────────

/// Arrow annotation direction on labels.
/// Java: `abel.LinkArrow`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkArrow {
    #[default]
    NoneOrSeveral,
    DirectNormal,
    Backward,
}

// ── LinkStyle ───────────────────────────────────────────────────────

/// Visual line style for edges.
/// Java: partial from `decoration.LinkType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkStyle {
    #[default]
    Normal,
    Dashed,
    Dotted,
    Bold,
    Hidden,
}

impl LinkStyle {
    /// DOT style attribute string.
    pub fn dot_style(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Dashed => "style=dashed,",
            Self::Dotted => "style=dotted,",
            Self::Bold => "style=bold,",
            Self::Hidden => "style=invis,",
        }
    }

    pub fn is_normal(&self) -> bool {
        *self == Self::Normal
    }
}

// ── LinkDecoration ──────────────────────────────────────────────────

/// Arrow-head decoration type at each end.
/// Java: `decoration.LinkDecor`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkDecoration {
    #[default]
    None,
    Arrow,
    Extends,
    Composition,
    Aggregation,
    Circle,
    CircleFill,
    CircleCross,
    Plus,
    HalfArrow,
    Crowfoot,
    NotNavigable,
    SquareSingle,
    Parenthesis,
}

impl LinkDecoration {
    /// DOT arrowhead attribute value.
    pub fn dot_arrow(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Arrow => "open",
            Self::Extends => "empty",
            Self::Composition => "diamond",
            Self::Aggregation => "odiamond",
            Self::Circle => "dot",
            Self::CircleFill => "dot",
            Self::CircleCross => "odot",
            Self::Plus => "obox",
            Self::HalfArrow => "halfopen",
            Self::Crowfoot => "crow",
            Self::NotNavigable => "tee",
            Self::SquareSingle => "box",
            Self::Parenthesis => "none",
        }
    }

    /// Decoration margin size for spacing calculation.
    pub fn margin(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::Arrow | Self::HalfArrow => 4,
            Self::Extends | Self::Composition | Self::Aggregation => 7,
            Self::Circle | Self::CircleFill | Self::CircleCross => 5,
            _ => 3,
        }
    }

    /// Whether this decoration is filled with the edge color.
    pub fn is_fill(&self) -> bool {
        matches!(
            self,
            Self::Composition | Self::CircleFill | Self::Arrow | Self::HalfArrow
        )
    }
}

// ── LinkMiddleDecoration ────────────────────────────────────────────

/// Decoration at the midpoint of an edge.
/// Java: `decoration.LinkMiddleDecor`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkMiddleDecoration {
    #[default]
    None,
    Circle,
    CircleFill,
}

// ── EntityPort ──────────────────────────────────────────────────────

/// Node identifier with optional port suffix.
/// Java: `cucadiagram.EntityPort`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPort {
    uid: String,
    port: Option<String>,
}

impl EntityPort {
    pub fn new(uid: &str) -> Self {
        Self {
            uid: uid.to_string(),
            port: None,
        }
    }

    pub fn with_port(uid: &str, port: &str) -> Self {
        Self {
            uid: uid.to_string(),
            port: Some(port.to_string()),
        }
    }

    /// Returns the full DOT string "uid:port" or just "uid".
    pub fn full_string(&self) -> String {
        match &self.port {
            Some(p) => format!("{}:{}", self.uid, p),
            None => self.uid.clone(),
        }
    }

    /// Returns just the uid prefix (no port).
    pub fn prefix(&self) -> &str {
        &self.uid
    }

    /// Check if this port starts with a specific prefix.
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.uid.starts_with(prefix)
    }

    /// Check if two ports reference the same entity id.
    pub fn equals_id(&self, other: &EntityPort) -> bool {
        self.uid == other.uid
    }
}

impl fmt::Display for EntityPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_string())
    }
}

// ── LabelDimension ──────────────────────────────────────────────────

/// Dimension info for an edge label.
#[derive(Debug, Clone, Copy)]
pub struct LabelDimension {
    pub width: f64,
    pub height: f64,
}

impl LabelDimension {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    pub fn from_dim(dim: XDimension2D) -> Self {
        Self {
            width: dim.width,
            height: dim.height,
        }
    }

    pub fn to_dim(&self) -> XDimension2D {
        XDimension2D::new(self.width, self.height)
    }

    pub fn delta(&self, d: f64) -> Self {
        Self {
            width: self.width + 2.0 * d,
            height: self.height + 2.0 * d,
        }
    }
}

// ── LineOfSegments ──────────────────────────────────────────────────

/// Segment overlap resolver for edge label placement.
/// Java: `svek.LineOfSegments`
///
/// Arranges segments along a line so they don't overlap,
/// preserving the overall mean position.
#[derive(Debug, Clone)]
struct Segment {
    idx: usize,
    middle: f64,
    half_size: f64,
}

impl Segment {
    fn new(idx: usize, x1: f64, x2: f64) -> Self {
        Self {
            idx,
            middle: (x1 + x2) / 2.0,
            half_size: (x2 - x1) / 2.0,
        }
    }

    fn overlap(&self, other: &Segment) -> f64 {
        let distance = other.middle - self.middle;
        debug_assert!(distance >= -1e-9, "segments must be sorted");
        let diff = distance - self.half_size - other.half_size;
        if diff > 0.0 {
            0.0
        } else {
            -diff
        }
    }

    fn push(&mut self, delta: f64) {
        self.middle += delta;
    }
}

#[derive(Debug, Clone, Default)]
pub struct LineOfSegments {
    all: Vec<Segment>,
}

impl LineOfSegments {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_segment(&mut self, x1: f64, x2: f64) {
        self.all.push(Segment::new(self.all.len(), x1, x2));
    }

    pub fn get_mean(&self) -> f64 {
        if self.all.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.all.iter().map(|s| s.middle).sum();
        sum / self.all.len() as f64
    }

    fn solve_overlaps_internal(&mut self) {
        if self.all.len() < 2 {
            return;
        }
        self.all
            .sort_by(|a, b| a.middle.partial_cmp(&b.middle).unwrap());
        let limit = self.all.len();
        for _ in 0..limit {
            if !self.one_loop() {
                return;
            }
        }
    }

    fn one_loop(&mut self) -> bool {
        let len = self.all.len();
        if len < 2 {
            return false;
        }
        for i in (0..len - 1).rev() {
            let overlap = self.all[i].overlap(&self.all[i + 1]);
            if overlap > 0.0 {
                for k in (i + 1)..len {
                    self.all[k].push(overlap);
                }
                return true;
            }
        }
        false
    }

    /// Solve overlaps and return the new x1 position for each segment
    /// (indexed by original insertion order).
    pub fn solve_overlaps(&mut self) -> Vec<f64> {
        let mean1 = self.get_mean();
        self.solve_overlaps_internal();
        let mean2 = self.get_mean();
        let diff = mean1 - mean2;
        if diff != 0.0 {
            for seg in &mut self.all {
                seg.push(diff);
            }
        }
        let mut result = vec![0.0; self.all.len()];
        for seg in &self.all {
            result[seg.idx] = seg.middle - seg.half_size;
        }
        result
    }
}

// ── SvekEdge ────────────────────────────────────────────────────────

/// An edge in the Graphviz layout graph.
/// Java: `svek.SvekEdge`
///
/// Lifecycle:
/// 1. Created with source/target endpoints and decorations
/// 2. `append_line()` generates DOT edge string
/// 3. After Graphviz runs, `solve_line()` parses SVG to extract path
/// 4. `draw()` renders the final edge with decorations
#[derive(Debug, Clone)]
pub struct SvekEdge {
    // ── Endpoints ──
    pub start_uid: EntityPort,
    pub end_uid: EntityPort,
    pub from_uid: String,
    pub to_uid: String,

    // ── DOT matching colors ──
    /// Primary DOT color for SVG matching (used by DotStringFactory)
    pub color: u32,
    /// Label color for SVG matching
    pub note_label_color: u32,
    /// Tail label color for SVG matching
    pub start_tail_color: u32,
    /// Head label color for SVG matching
    pub end_head_color: u32,

    // ── Layout result (filled after SVG parsing) ──
    /// Bezier path after layout
    pub dot_path: Option<DotPath>,
    /// Copy of dot_path at parse time (before adjustments)
    pub dot_path_init: Option<DotPath>,

    // ── Label positions (filled after SVG parsing) ──
    pub label_xy: Option<XPoint2D>,
    pub start_tail_label_xy: Option<XPoint2D>,
    pub end_head_label_xy: Option<XPoint2D>,

    // ── Labels ──
    pub label: Option<String>,
    pub label_dimension: Option<LabelDimension>,
    pub start_tail_text: Option<String>,
    pub start_tail_dimension: Option<LabelDimension>,
    pub end_head_text: Option<String>,
    pub end_head_dimension: Option<LabelDimension>,

    // ── Link properties ──
    pub link_length: i32,
    pub link_style: LinkStyle,
    pub link_arrow: LinkArrow,
    pub decor1: LinkDecoration,
    pub decor2: LinkDecoration,
    pub middle_decor: LinkMiddleDecoration,
    pub is_invis: bool,
    pub is_constraint: bool,
    pub same_tail: Option<String>,

    // ── Label layout control ──
    pub divide_label_width_by_two: bool,
    pub label_shield: f64,

    // ── Cluster connections ──
    pub ltail: Option<String>,
    pub lhead: Option<String>,

    // ── Translation offset (applied during draw) ──
    pub dx: f64,
    pub dy: f64,

    // ── Flags ──
    pub opale: bool,
    pub use_rank_same: bool,

    // ── Arrow polygon points (from SVG parsing) ──
    pub arrow_head: Option<Vec<XPoint2D>>,
    pub arrow_tail: Option<Vec<XPoint2D>>,
}

impl SvekEdge {
    /// Create a new edge between two node UIDs.
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            start_uid: EntityPort::new(from),
            end_uid: EntityPort::new(to),
            from_uid: from.to_string(),
            to_uid: to.to_string(),
            color: 0,
            note_label_color: 0,
            start_tail_color: 0,
            end_head_color: 0,
            dot_path: None,
            dot_path_init: None,
            label_xy: None,
            start_tail_label_xy: None,
            end_head_label_xy: None,
            label: None,
            label_dimension: None,
            start_tail_text: None,
            start_tail_dimension: None,
            end_head_text: None,
            end_head_dimension: None,
            link_length: 2,
            link_style: LinkStyle::default(),
            link_arrow: LinkArrow::default(),
            decor1: LinkDecoration::default(),
            decor2: LinkDecoration::default(),
            middle_decor: LinkMiddleDecoration::default(),
            is_invis: false,
            is_constraint: true,
            same_tail: None,
            divide_label_width_by_two: false,
            label_shield: 0.0,
            ltail: None,
            lhead: None,
            dx: 0.0,
            dy: 0.0,
            opale: false,
            use_rank_same: false,
            arrow_head: None,
            arrow_tail: None,
        }
    }

    /// Builder: set entity ports with optional port suffixes.
    pub fn with_ports(mut self, start: EntityPort, end: EntityPort) -> Self {
        self.from_uid = start.uid.clone();
        self.to_uid = end.uid.clone();
        self.start_uid = start;
        self.end_uid = end;
        self
    }

    /// Builder: assign DOT matching colors from a ColorSequence.
    pub fn with_colors(
        mut self,
        line: u32,
        note_label: u32,
        start_tail: u32,
        end_head: u32,
    ) -> Self {
        self.color = line;
        self.note_label_color = note_label;
        self.start_tail_color = start_tail;
        self.end_head_color = end_head;
        self
    }

    /// Builder: set decorations.
    pub fn with_decorations(mut self, decor1: LinkDecoration, decor2: LinkDecoration) -> Self {
        self.decor1 = decor1;
        self.decor2 = decor2;
        self
    }

    /// Builder: set link style.
    pub fn with_style(mut self, style: LinkStyle) -> Self {
        self.link_style = style;
        self
    }

    /// Builder: set link length (rank distance).
    pub fn with_length(mut self, length: i32) -> Self {
        self.link_length = length;
        self
    }

    /// Builder: set label text and dimension.
    pub fn with_label(mut self, text: &str, dim: LabelDimension) -> Self {
        self.label = Some(text.to_string());
        self.label_dimension = Some(dim);
        self
    }

    /// Builder: set tail label.
    pub fn with_tail_label(mut self, text: &str, dim: LabelDimension) -> Self {
        self.start_tail_text = Some(text.to_string());
        self.start_tail_dimension = Some(dim);
        self
    }

    /// Builder: set head label.
    pub fn with_head_label(mut self, text: &str, dim: LabelDimension) -> Self {
        self.end_head_text = Some(text.to_string());
        self.end_head_dimension = Some(dim);
        self
    }

    // ── DOT generation ──

    /// Generate the DOT edge line.
    /// Java: `SvekEdge.appendLine()`
    ///
    /// Produces: `startUid->endUid[decoration,minlen=N,color="#RRGGBB",label=<TABLE...>,...];`
    pub fn append_line(&self, sb: &mut String) {
        sb.push_str(&self.start_uid.full_string());
        sb.push_str("->");
        sb.push_str(&self.end_uid.full_string());
        sb.push('[');

        // Decoration (style attribute)
        let decoration = self.get_decoration_svek();
        if !decoration.is_empty() {
            sb.push_str(&decoration);
            if !decoration.ends_with(',') {
                sb.push(',');
            }
        }

        // minlen
        let minlen = self.link_length - 1;
        if self.use_rank_same {
            if self.is_invis || self.link_length != 1 {
                sb.push_str(&format!("minlen={},", minlen));
            }
        } else {
            sb.push_str(&format!("minlen={},", minlen));
        }

        // color
        sb.push_str(&format!("color=\"{}\"", color_to_hex(self.color)));

        // Label (center label)
        if self.has_label() {
            sb.push(',');
            sb.push_str("label=<");
            if let Some(dim) = &self.label_dimension {
                let dim_with_shield = dim.delta(self.label_shield);
                let final_dim = self.eventually_divide_by_two(dim_with_shield);
                append_table(sb, final_dim, self.note_label_color);
            }
            sb.push('>');
        }

        // Tail label
        if let Some(dim) = &self.start_tail_dimension {
            sb.push(',');
            sb.push_str("taillabel=<");
            append_table(sb, *dim, self.start_tail_color);
            sb.push('>');
        }

        // Head label
        if let Some(dim) = &self.end_head_dimension {
            sb.push(',');
            sb.push_str("headlabel=<");
            append_table(sb, *dim, self.end_head_color);
            sb.push('>');
        }

        // Invisible
        if self.is_invis {
            sb.push_str(",style=invis");
        }

        // Constraint
        if !self.is_constraint {
            sb.push_str(",constraint=false");
        }

        // Same tail
        if let Some(ref st) = self.same_tail {
            sb.push_str(&format!(",sametail={}", st));
        }

        sb.push_str("];\n");
    }

    /// Get DOT decoration string for the link style + arrowheads.
    fn get_decoration_svek(&self) -> String {
        let mut result = String::new();

        // Arrow head/tail types
        let head = self.decor1.dot_arrow();
        let tail = self.decor2.dot_arrow();
        if head != "open" || tail != "none" {
            result.push_str(&format!("arrowhead={},arrowtail={},", head, tail));
        }

        // Line style
        result.push_str(self.link_style.dot_style());

        // Direction (both = show both arrowheads)
        if self.decor1 != LinkDecoration::None && self.decor2 != LinkDecoration::None {
            result.push_str("dir=both,");
        }

        result
    }

    /// Check whether this edge has a center label.
    pub fn has_label(&self) -> bool {
        self.label.is_some() || self.label_dimension.is_some()
    }

    fn eventually_divide_by_two(&self, dim: LabelDimension) -> LabelDimension {
        if self.divide_label_width_by_two {
            LabelDimension::new(dim.width / 2.0, dim.height)
        } else {
            dim
        }
    }

    // ── Rank same constraint ──

    /// Generate `{rank=same; start; end}` string if this is a horizontal edge.
    /// Java: `SvekEdge.rankSame()`
    pub fn rank_same(&self) -> Option<String> {
        if self.link_length == 1 && !self.is_invis {
            Some(format!(
                "{{rank=same; {}; {}}}",
                self.start_uid.prefix(),
                self.end_uid.prefix()
            ))
        } else {
            None
        }
    }

    // ── SVG parsing ──

    /// Parse the Graphviz SVG output to extract path and label positions.
    /// Java: `SvekEdge.solveLine(SvgResult fullSvg)`
    ///
    /// After calling this, `dot_path`, `label_xy`, etc. are populated.
    pub fn solve_line(&mut self, svg: &crate::svek::svg_result::SvgResult) {
        if self.is_invis {
            return;
        }

        // Find edge path by color
        let idx = match svg.find_by_color(self.color) {
            Some(i) => i,
            None => return,
        };

        // Extract DotPath
        if let Some((path, _end_pos)) = svg.extract_dot_path(idx) {
            self.dot_path = Some(path.clone());
            self.dot_path_init = Some(path);

            // Parse label positions
            self.solve_label_positions(svg);
        }
    }

    /// Parse label positions from SVG colors.
    fn solve_label_positions(&mut self, svg: &crate::svek::svg_result::SvgResult) {
        // Center label
        if self.has_label() {
            self.label_xy = self.get_label_xy(svg, self.note_label_color);
        }

        // Tail label
        if self.start_tail_text.is_some() || self.start_tail_dimension.is_some() {
            self.start_tail_label_xy = self.get_label_xy(svg, self.start_tail_color);
        }

        // Head label
        if self.end_head_text.is_some() || self.end_head_dimension.is_some() {
            self.end_head_label_xy = self.get_label_xy(svg, self.end_head_color);
        }
    }

    /// Extract label position from SVG by color.
    fn get_label_xy(
        &self,
        svg: &crate::svek::svg_result::SvgResult,
        color: u32,
    ) -> Option<XPoint2D> {
        let idx = svg.find_by_color(color)?;
        let points = svg.extract_points_at(idx);
        if points.is_empty() {
            return None;
        }
        Some(get_min_xy(&points))
    }

    // ── Arrow direction detection ──

    /// Determine the arrow direction from the path endpoints.
    /// Java: `SvekEdge.getArrowDirection()`
    pub fn arrow_direction(&self) -> Option<Direction> {
        let path = self.dot_path.as_ref()?;
        let dir = self.arrow_direction_internal(path);
        if self.link_arrow == LinkArrow::Backward {
            Some(dir.inv())
        } else {
            Some(dir)
        }
    }

    fn arrow_direction_internal(&self, path: &DotPath) -> Direction {
        if self.is_autolink() {
            return Direction::Left;
        }
        let start = path.start_point();
        let end = path.end_point();
        let ang = (end.x - start.x).atan2(end.y - start.y);
        if ang > -std::f64::consts::FRAC_PI_4 && ang < std::f64::consts::FRAC_PI_4 {
            Direction::Down
        } else if ang > std::f64::consts::FRAC_PI_4 * 3.0
            || ang < -std::f64::consts::FRAC_PI_4 * 3.0
        {
            Direction::Up
        } else if end.x > start.x {
            Direction::Right
        } else {
            Direction::Left
        }
    }

    /// Arrow direction as angle in radians.
    pub fn arrow_direction_radian(&self) -> Option<f64> {
        let path = self.dot_path.as_ref()?;
        let angle = self.arrow_direction_radian_internal(path);
        if self.link_arrow == LinkArrow::Backward {
            Some(std::f64::consts::PI + angle)
        } else {
            Some(angle)
        }
    }

    fn arrow_direction_radian_internal(&self, path: &DotPath) -> f64 {
        if self.is_autolink() {
            return path.start_angle();
        }
        let start = path.start_point();
        let end = path.end_point();
        (end.x - start.x).atan2(end.y - start.y)
    }

    // ── Self-loop detection ──

    /// Whether this edge connects a node to itself.
    pub fn is_autolink(&self) -> bool {
        self.start_uid.equals_id(&self.end_uid)
    }

    /// Whether this is a horizontal link (length == 1).
    pub fn is_horizontal(&self) -> bool {
        self.link_length == 1
    }

    // ── Delta / translation ──

    /// Accumulate translation delta for rendering offset.
    /// Java: `SvekEdge.moveDelta()`
    pub fn move_delta(&mut self, delta_x: f64, delta_y: f64) {
        self.dx += delta_x;
        self.dy += delta_y;
    }

    /// Get the DotPath with accumulated delta applied.
    pub fn get_dot_path(&self) -> Option<DotPath> {
        let mut path = self.dot_path.clone()?;
        path.move_delta(self.dx, self.dy);
        Some(path)
    }

    /// Get start contact point (with delta applied).
    pub fn start_contact_point(&self) -> Option<XPoint2D> {
        let path = self.dot_path.as_ref()?;
        let start = path.start_point();
        Some(XPoint2D::new(self.dx + start.x, self.dy + start.y))
    }

    /// Get end contact point (with delta applied).
    pub fn end_contact_point(&self) -> Option<XPoint2D> {
        let path = self.dot_path.as_ref()?;
        let end = path.end_point();
        Some(XPoint2D::new(self.dx + end.x, self.dy + end.y))
    }

    /// Move the start point of both dot_path and dot_path_init.
    pub fn move_start_point(&mut self, dx: f64, dy: f64) {
        if let Some(ref mut path) = self.dot_path {
            path.move_start_point(dx, dy);
        }
        if let Some(ref mut path) = self.dot_path_init {
            path.move_start_point(dx, dy);
        }
    }

    /// Move the end point of both dot_path and dot_path_init.
    pub fn move_end_point(&mut self, dx: f64, dy: f64) {
        if let Some(ref mut path) = self.dot_path {
            path.move_end_point(dx, dy);
        }
        if let Some(ref mut path) = self.dot_path_init {
            path.move_end_point(dx, dy);
        }
    }

    /// Replace the dot path entirely.
    pub fn replace_dot_path(&mut self, new_path: DotPath) {
        self.dot_path_init = Some(new_path.clone());
        self.dot_path = Some(new_path);
    }

    // ── Spacing calculations ──

    /// Decoration length contribution from both ends.
    fn decor_dzeta(&self) -> f64 {
        (self.decor1.margin() + self.decor2.margin()) as f64
    }

    /// Total horizontal space contribution of this edge (for nodesep calculation).
    /// Java: `SvekEdge.getHorizontalDzeta()`
    pub fn horizontal_dzeta(&self) -> f64 {
        if self.is_autolink() {
            return self.decor_dzeta();
        }
        if !self.is_horizontal() {
            return 0.0;
        }
        let mut total = 0.0;
        if let Some(dim) = &self.label_dimension {
            total += dim.width;
        }
        if let Some(dim) = &self.start_tail_dimension {
            total += dim.width;
        }
        if let Some(dim) = &self.end_head_dimension {
            total += dim.width;
        }
        total + self.decor_dzeta()
    }

    /// Total vertical space contribution of this edge (for ranksep calculation).
    /// Java: `SvekEdge.getVerticalDzeta()`
    pub fn vertical_dzeta(&self) -> f64 {
        if self.is_autolink() {
            return self.decor_dzeta();
        }
        if self.is_horizontal() {
            return 0.0;
        }
        let mut total = 0.0;
        if let Some(dim) = &self.label_dimension {
            total += dim.height;
        }
        if let Some(dim) = &self.start_tail_dimension {
            total += dim.height;
        }
        if let Some(dim) = &self.end_head_dimension {
            total += dim.height;
        }
        total + self.decor_dzeta()
    }

    // ── Opale (hidden edge) ──

    pub fn set_opale(&mut self, opale: bool) {
        self.opale = opale;
    }

    pub fn is_opale(&self) -> bool {
        self.opale
    }

    /// Check if the path is simple enough for opale treatment.
    fn is_opalisable(&self) -> bool {
        match &self.dot_path {
            Some(path) => path.beziers.len() <= 1,
            None => true,
        }
    }

    // ── Connection checks ──

    /// Check if this edge connects the same pair as another.
    pub fn same_connections(&self, other: &SvekEdge) -> bool {
        self.from_uid == other.from_uid && self.to_uid == other.to_uid
    }

    /// Get point on this edge closest to a given entity.
    pub fn get_point_for(&self, entity_uid: &str) -> Option<XPoint2D> {
        let path = self.dot_path.as_ref()?;
        if self.from_uid == entity_uid {
            let pt = path.start_point();
            Some(XPoint2D::new(pt.x + self.dx, pt.y + self.dy))
        } else if self.to_uid == entity_uid {
            let pt = path.end_point();
            Some(XPoint2D::new(pt.x + self.dx, pt.y + self.dy))
        } else {
            None
        }
    }
}

impl fmt::Display for SvekEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SvekEdge({}->{} color={:#08x})",
            self.start_uid, self.end_uid, self.color
        )
    }
}

// ── Module-level utility functions ──────────────────────────────────

/// Format a color integer as hex string "#RRGGBB".
pub fn color_to_hex(color: u32) -> String {
    format!("#{:06x}", color)
}

/// Append an HTML TABLE element for DOT label sizing.
/// Java: `SvekEdge.appendTable()`
pub fn append_table(sb: &mut String, dim: LabelDimension, color: u32) {
    let w = dim.width as i32;
    let h = dim.height as i32;
    sb.push_str(&format!(
        "<TABLE BGCOLOR=\"{}\" FIXEDSIZE=\"TRUE\" WIDTH=\"{}\" HEIGHT=\"{}\">",
        color_to_hex(color),
        w,
        h
    ));
    sb.push_str("<TR>");
    sb.push_str("<TD>");
    sb.push_str("</TD>");
    sb.push_str("</TR>");
    sb.push_str("</TABLE>");
}

/// Get the minimum (x, y) point from a list.
/// Java: `SvekUtils.getMinXY()`
pub fn get_min_xy(points: &[XPoint2D]) -> XPoint2D {
    let mut min_x = points[0].x;
    let mut min_y = points[0].y;
    for pt in &points[1..] {
        if pt.x < min_x {
            min_x = pt.x;
        }
        if pt.y < min_y {
            min_y = pt.y;
        }
    }
    XPoint2D::new(min_x, min_y)
}

/// Get the maximum (x, y) point from a list.
/// Java: `SvekUtils.getMaxXY()`
pub fn get_max_xy(points: &[XPoint2D]) -> XPoint2D {
    let mut max_x = points[0].x;
    let mut max_y = points[0].y;
    for pt in &points[1..] {
        if pt.x > max_x {
            max_x = pt.x;
        }
        if pt.y > max_y {
            max_y = pt.y;
        }
    }
    XPoint2D::new(max_x, max_y)
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::klimt::geom::XCubicCurve2D;

    // ── EntityPort tests ──

    #[test]
    fn entity_port_basic() {
        let ep = EntityPort::new("node1");
        assert_eq!(ep.full_string(), "node1");
        assert_eq!(ep.prefix(), "node1");
        assert!(!ep.starts_with("x"));
        assert!(ep.starts_with("node"));
    }

    #[test]
    fn entity_port_with_port() {
        let ep = EntityPort::with_port("node1", "p0");
        assert_eq!(ep.full_string(), "node1:p0");
        assert_eq!(ep.prefix(), "node1");
    }

    #[test]
    fn entity_port_equals_id() {
        let ep1 = EntityPort::new("A");
        let ep2 = EntityPort::with_port("A", "p1");
        let ep3 = EntityPort::new("B");
        assert!(ep1.equals_id(&ep2));
        assert!(!ep1.equals_id(&ep3));
    }

    #[test]
    fn entity_port_display() {
        let ep = EntityPort::with_port("N", "south");
        assert_eq!(format!("{}", ep), "N:south");
    }

    // ── Direction tests ──

    #[test]
    fn direction_inv() {
        assert_eq!(Direction::Up.inv(), Direction::Down);
        assert_eq!(Direction::Down.inv(), Direction::Up);
        assert_eq!(Direction::Left.inv(), Direction::Right);
        assert_eq!(Direction::Right.inv(), Direction::Left);
    }

    // ── LinkDecoration tests ──

    #[test]
    fn link_decoration_dot_arrow() {
        assert_eq!(LinkDecoration::None.dot_arrow(), "none");
        assert_eq!(LinkDecoration::Arrow.dot_arrow(), "open");
        assert_eq!(LinkDecoration::Extends.dot_arrow(), "empty");
        assert_eq!(LinkDecoration::Composition.dot_arrow(), "diamond");
    }

    #[test]
    fn link_decoration_margin() {
        assert_eq!(LinkDecoration::None.margin(), 0);
        assert_eq!(LinkDecoration::Arrow.margin(), 4);
        assert_eq!(LinkDecoration::Extends.margin(), 7);
    }

    #[test]
    fn link_decoration_is_fill() {
        assert!(LinkDecoration::Composition.is_fill());
        assert!(LinkDecoration::Arrow.is_fill());
        assert!(!LinkDecoration::None.is_fill());
        assert!(!LinkDecoration::Extends.is_fill());
    }

    // ── LinkStyle tests ──

    #[test]
    fn link_style_dot() {
        assert_eq!(LinkStyle::Normal.dot_style(), "");
        assert_eq!(LinkStyle::Dashed.dot_style(), "style=dashed,");
        assert_eq!(LinkStyle::Dotted.dot_style(), "style=dotted,");
    }

    // ── LabelDimension tests ──

    #[test]
    fn label_dimension_delta() {
        let dim = LabelDimension::new(100.0, 50.0);
        let d = dim.delta(5.0);
        assert_eq!(d.width, 110.0);
        assert_eq!(d.height, 60.0);
    }

    #[test]
    fn label_dimension_from_dim() {
        let xd = XDimension2D::new(80.0, 40.0);
        let ld = LabelDimension::from_dim(xd);
        assert_eq!(ld.width, 80.0);
        assert_eq!(ld.height, 40.0);
        let back = ld.to_dim();
        assert_eq!(back.width, 80.0);
        assert_eq!(back.height, 40.0);
    }

    // ── LineOfSegments tests ──

    #[test]
    fn line_of_segments_no_overlap() {
        let mut los = LineOfSegments::new();
        los.add_segment(0.0, 10.0);
        los.add_segment(20.0, 30.0);
        let result = los.solve_overlaps();
        assert_eq!(result.len(), 2);
        assert!((result[0] - 0.0).abs() < 1e-6);
        assert!((result[1] - 20.0).abs() < 1e-6);
    }

    #[test]
    fn line_of_segments_with_overlap() {
        let mut los = LineOfSegments::new();
        los.add_segment(0.0, 10.0);
        los.add_segment(5.0, 15.0);
        let result = los.solve_overlaps();
        assert_eq!(result.len(), 2);
        let end_0 = result[0] + 10.0;
        let start_1 = result[1];
        assert!(
            end_0 <= start_1 + 1e-6,
            "segments should not overlap: end_0={}, start_1={}",
            end_0,
            start_1
        );
    }

    #[test]
    fn line_of_segments_preserves_mean() {
        let mut los = LineOfSegments::new();
        los.add_segment(0.0, 10.0);
        los.add_segment(5.0, 15.0);
        let mean_before = los.get_mean();
        let _ = los.solve_overlaps();
        let mean_after = los.get_mean();
        assert!(
            (mean_before - mean_after).abs() < 1e-6,
            "mean should be preserved: before={}, after={}",
            mean_before,
            mean_after
        );
    }

    #[test]
    fn line_of_segments_single() {
        let mut los = LineOfSegments::new();
        los.add_segment(10.0, 20.0);
        let result = los.solve_overlaps();
        assert_eq!(result.len(), 1);
        assert!((result[0] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn line_of_segments_three_overlap() {
        let mut los = LineOfSegments::new();
        los.add_segment(0.0, 10.0);
        los.add_segment(5.0, 15.0);
        los.add_segment(8.0, 18.0);
        let result = los.solve_overlaps();
        assert_eq!(result.len(), 3);
        for i in 0..result.len() - 1 {
            let end_i = result[i] + 10.0;
            let start_next = result[i + 1];
            assert!(
                end_i <= start_next + 1e-6,
                "overlap between seg {} and {}: end={}, start={}",
                i,
                i + 1,
                end_i,
                start_next
            );
        }
    }

    // ── SvekEdge basic tests ──

    #[test]
    fn edge_basic() {
        let e = SvekEdge::new("A", "B");
        assert_eq!(e.from_uid, "A");
        assert_eq!(e.to_uid, "B");
        assert!(e.dot_path.is_none());
        assert!(!e.is_autolink());
    }

    #[test]
    fn edge_autolink() {
        let e = SvekEdge::new("A", "A");
        assert!(e.is_autolink());
    }

    #[test]
    fn edge_display() {
        let e =
            SvekEdge::new("X", "Y").with_colors(0x010200, 0x020300, 0x030400, 0x040500);
        let s = format!("{}", e);
        assert!(s.contains("X"));
        assert!(s.contains("Y"));
        // Color may be formatted as hex or decimal
        assert!(s.contains("X") && s.contains("Y"));
    }

    // ── DOT generation tests ──

    #[test]
    fn append_line_basic() {
        let e = SvekEdge::new("A", "B").with_colors(0x010200, 0, 0, 0);
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(sb.contains("A->B["));
        assert!(sb.contains("color=\"#010200\""));
        assert!(sb.ends_with("];\n"));
    }

    #[test]
    fn append_line_with_minlen() {
        let e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0, 0, 0)
            .with_length(3);
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(sb.contains("minlen=2,"), "expected minlen=2, got: {}", sb);
    }

    #[test]
    fn append_line_with_label() {
        let e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0x020300, 0, 0)
            .with_label("test", LabelDimension::new(60.0, 20.0));
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(sb.contains("label=<"), "expected label, got: {}", sb);
        assert!(sb.contains("TABLE"), "expected TABLE, got: {}", sb);
        assert!(
            sb.contains("BGCOLOR=\"#020300\""),
            "expected note_label_color, got: {}",
            sb
        );
        assert!(
            sb.contains("WIDTH=\"60\" HEIGHT=\"20\""),
            "expected dimensions, got: {}",
            sb
        );
    }

    #[test]
    fn append_line_with_tail_label() {
        let e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0, 0x030400, 0)
            .with_tail_label("1..*", LabelDimension::new(30.0, 14.0));
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("taillabel=<"),
            "expected taillabel, got: {}",
            sb
        );
        assert!(
            sb.contains("BGCOLOR=\"#030400\""),
            "expected start_tail_color, got: {}",
            sb
        );
    }

    #[test]
    fn append_line_with_head_label() {
        let e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0, 0, 0x040500)
            .with_head_label("0..1", LabelDimension::new(25.0, 14.0));
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("headlabel=<"),
            "expected headlabel, got: {}",
            sb
        );
    }

    #[test]
    fn append_line_invis() {
        let mut e = SvekEdge::new("A", "B").with_colors(0x010200, 0, 0, 0);
        e.is_invis = true;
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("style=invis"),
            "expected style=invis, got: {}",
            sb
        );
    }

    #[test]
    fn append_line_no_constraint() {
        let mut e = SvekEdge::new("A", "B").with_colors(0x010200, 0, 0, 0);
        e.is_constraint = false;
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("constraint=false"),
            "expected constraint=false, got: {}",
            sb
        );
    }

    #[test]
    fn append_line_with_sametail() {
        let mut e = SvekEdge::new("A", "B").with_colors(0x010200, 0, 0, 0);
        e.same_tail = Some("group1".to_string());
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("sametail=group1"),
            "expected sametail, got: {}",
            sb
        );
    }

    #[test]
    fn append_line_with_ports() {
        let e = SvekEdge::new("A", "B")
            .with_ports(
                EntityPort::with_port("A", "south"),
                EntityPort::with_port("B", "north"),
            )
            .with_colors(0x010200, 0, 0, 0);
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(sb.starts_with("A:south->B:north["), "got: {}", sb);
    }

    #[test]
    fn append_line_decorations() {
        let e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0, 0, 0)
            .with_decorations(LinkDecoration::Extends, LinkDecoration::Arrow);
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("arrowhead=empty"),
            "expected arrowhead=empty, got: {}",
            sb
        );
        assert!(
            sb.contains("arrowtail=open"),
            "expected arrowtail=open, got: {}",
            sb
        );
        assert!(sb.contains("dir=both"), "expected dir=both, got: {}", sb);
    }

    #[test]
    fn append_line_dashed_style() {
        let e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0, 0, 0)
            .with_style(LinkStyle::Dashed);
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("style=dashed"),
            "expected style=dashed, got: {}",
            sb
        );
    }

    // ── Rank same tests ──

    #[test]
    fn rank_same_horizontal() {
        let e = SvekEdge::new("A", "B").with_length(1);
        let rs = e.rank_same();
        assert!(rs.is_some());
        let s = rs.unwrap();
        assert!(s.contains("rank=same"));
        assert!(s.contains("A"));
        assert!(s.contains("B"));
    }

    #[test]
    fn rank_same_non_horizontal() {
        let e = SvekEdge::new("A", "B").with_length(2);
        assert!(e.rank_same().is_none());
    }

    #[test]
    fn rank_same_invis() {
        let mut e = SvekEdge::new("A", "B").with_length(1);
        e.is_invis = true;
        assert!(e.rank_same().is_none());
    }

    // ── Arrow direction tests ──

    #[test]
    fn arrow_direction_downward() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            50.0, 0.0, 50.0, 33.0, 50.0, 66.0, 50.0, 100.0,
        )]));
        assert_eq!(e.arrow_direction(), Some(Direction::Down));
    }

    #[test]
    fn arrow_direction_upward() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            50.0, 100.0, 50.0, 66.0, 50.0, 33.0, 50.0, 0.0,
        )]));
        assert_eq!(e.arrow_direction(), Some(Direction::Up));
    }

    #[test]
    fn arrow_direction_right() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            0.0, 50.0, 33.0, 50.0, 66.0, 50.0, 100.0, 50.0,
        )]));
        assert_eq!(e.arrow_direction(), Some(Direction::Right));
    }

    #[test]
    fn arrow_direction_left() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            100.0, 50.0, 66.0, 50.0, 33.0, 50.0, 0.0, 50.0,
        )]));
        assert_eq!(e.arrow_direction(), Some(Direction::Left));
    }

    #[test]
    fn arrow_direction_backward() {
        let mut e = SvekEdge::new("A", "B");
        e.link_arrow = LinkArrow::Backward;
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            50.0, 0.0, 50.0, 33.0, 50.0, 66.0, 50.0, 100.0,
        )]));
        assert_eq!(e.arrow_direction(), Some(Direction::Up)); // inverted
    }

    #[test]
    fn arrow_direction_autolink() {
        let mut e = SvekEdge::new("A", "A");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            50.0, 0.0, 80.0, -20.0, 80.0, 40.0, 50.0, 30.0,
        )]));
        assert_eq!(e.arrow_direction(), Some(Direction::Left));
    }

    #[test]
    fn arrow_direction_no_path() {
        let e = SvekEdge::new("A", "B");
        assert!(e.arrow_direction().is_none());
    }

    // ── Contact point tests ──

    #[test]
    fn contact_points() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0,
        )]));
        e.dx = 5.0;
        e.dy = 10.0;
        let start = e.start_contact_point().unwrap();
        assert_eq!(start, XPoint2D::new(15.0, 30.0));
        let end = e.end_contact_point().unwrap();
        assert_eq!(end, XPoint2D::new(75.0, 90.0));
    }

    #[test]
    fn contact_points_no_path() {
        let e = SvekEdge::new("A", "B");
        assert!(e.start_contact_point().is_none());
        assert!(e.end_contact_point().is_none());
    }

    // ── Move delta tests ──

    #[test]
    fn move_delta_accumulates() {
        let mut e = SvekEdge::new("A", "B");
        e.move_delta(10.0, 20.0);
        e.move_delta(5.0, -3.0);
        assert_eq!(e.dx, 15.0);
        assert_eq!(e.dy, 17.0);
    }

    #[test]
    fn get_dot_path_with_delta() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            0.0, 0.0, 10.0, 10.0, 20.0, 10.0, 30.0, 0.0,
        )]));
        e.dx = 100.0;
        e.dy = 50.0;
        let path = e.get_dot_path().unwrap();
        assert_eq!(path.start_point(), XPoint2D::new(100.0, 50.0));
        assert_eq!(path.end_point(), XPoint2D::new(130.0, 50.0));
    }

    // ── Spacing / dzeta tests ──

    #[test]
    fn horizontal_dzeta_horizontal_with_label() {
        let e = SvekEdge::new("A", "B")
            .with_length(1)
            .with_label("x", LabelDimension::new(60.0, 20.0));
        let dz = e.horizontal_dzeta();
        assert!(dz >= 60.0, "expected >= 60, got: {}", dz);
    }

    #[test]
    fn horizontal_dzeta_vertical() {
        let e = SvekEdge::new("A", "B")
            .with_length(2)
            .with_label("x", LabelDimension::new(60.0, 20.0));
        assert_eq!(e.horizontal_dzeta(), 0.0);
    }

    #[test]
    fn vertical_dzeta_vertical_with_label() {
        let e = SvekEdge::new("A", "B")
            .with_length(2)
            .with_label("x", LabelDimension::new(60.0, 20.0));
        let dz = e.vertical_dzeta();
        assert!(dz >= 20.0, "expected >= 20, got: {}", dz);
    }

    #[test]
    fn vertical_dzeta_horizontal() {
        let e = SvekEdge::new("A", "B")
            .with_length(1)
            .with_label("x", LabelDimension::new(60.0, 20.0));
        assert_eq!(e.vertical_dzeta(), 0.0);
    }

    #[test]
    fn dzeta_autolink() {
        let e = SvekEdge::new("A", "A")
            .with_decorations(LinkDecoration::Arrow, LinkDecoration::None);
        let hz = e.horizontal_dzeta();
        let vz = e.vertical_dzeta();
        assert_eq!(hz, vz);
        assert_eq!(hz, 4.0); // Arrow margin = 4
    }

    // ── Same connections tests ──

    #[test]
    fn same_connections() {
        let e1 = SvekEdge::new("A", "B");
        let e2 = SvekEdge::new("A", "B");
        let e3 = SvekEdge::new("A", "C");
        assert!(e1.same_connections(&e2));
        assert!(!e1.same_connections(&e3));
    }

    // ── Get point for entity tests ──

    #[test]
    fn get_point_for_entity() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0,
        )]));
        e.dx = 5.0;
        e.dy = 10.0;
        let p_a = e.get_point_for("A").unwrap();
        assert_eq!(p_a, XPoint2D::new(15.0, 30.0));
        let p_b = e.get_point_for("B").unwrap();
        assert_eq!(p_b, XPoint2D::new(75.0, 90.0));
        assert!(e.get_point_for("C").is_none());
    }

    // ── Opale tests ──

    #[test]
    fn opale_flag() {
        let mut e = SvekEdge::new("A", "B");
        assert!(!e.is_opale());
        e.set_opale(true);
        assert!(e.is_opale());
    }

    #[test]
    fn is_opalisable_empty_path() {
        let e = SvekEdge::new("A", "B");
        assert!(e.is_opalisable());
    }

    #[test]
    fn is_opalisable_single_bezier() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            0.0, 0.0, 10.0, 10.0, 20.0, 10.0, 30.0, 0.0,
        )]));
        assert!(e.is_opalisable());
    }

    #[test]
    fn is_opalisable_multi_bezier() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![
            XCubicCurve2D::new(0.0, 0.0, 10.0, 10.0, 20.0, 10.0, 30.0, 0.0),
            XCubicCurve2D::new(30.0, 0.0, 40.0, -10.0, 50.0, -10.0, 60.0, 0.0),
        ]));
        assert!(!e.is_opalisable());
    }

    // ── move_start_point / move_end_point tests ──

    #[test]
    fn move_start_end_points() {
        let mut e = SvekEdge::new("A", "B");
        let path = DotPath::from_beziers(vec![XCubicCurve2D::new(
            10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0,
        )]);
        e.dot_path = Some(path.clone());
        e.dot_path_init = Some(path);

        e.move_start_point(5.0, 10.0);
        let new_start = e.dot_path.as_ref().unwrap().start_point();
        assert_eq!(new_start, XPoint2D::new(15.0, 30.0));

        e.move_end_point(-3.0, -4.0);
        let new_end = e.dot_path.as_ref().unwrap().end_point();
        assert_eq!(new_end, XPoint2D::new(67.0, 76.0));

        // dot_path_init should also be updated
        let init_start = e.dot_path_init.as_ref().unwrap().start_point();
        assert_eq!(init_start, XPoint2D::new(15.0, 30.0));
    }

    // ── replace_dot_path test ──

    #[test]
    fn replace_dot_path() {
        let mut e = SvekEdge::new("A", "B");
        let new_path = DotPath::from_beziers(vec![XCubicCurve2D::new(
            0.0, 0.0, 10.0, 10.0, 20.0, 10.0, 30.0, 0.0,
        )]);
        e.replace_dot_path(new_path);
        assert!(e.dot_path.is_some());
        assert!(e.dot_path_init.is_some());
        assert_eq!(
            e.dot_path.as_ref().unwrap().start_point(),
            e.dot_path_init.as_ref().unwrap().start_point()
        );
    }

    // ── Utility function tests ──

    #[test]
    fn color_to_hex_format() {
        assert_eq!(color_to_hex(0xFF0000), "#ff0000");
        assert_eq!(color_to_hex(0x010200), "#010200");
        assert_eq!(color_to_hex(0x000000), "#000000");
    }

    #[test]
    fn append_table_format() {
        let mut sb = String::new();
        append_table(&mut sb, LabelDimension::new(100.0, 50.0), 0x010200);
        assert!(sb.contains("BGCOLOR=\"#010200\""));
        assert!(sb.contains("WIDTH=\"100\" HEIGHT=\"50\""));
        assert!(sb.contains("FIXEDSIZE=\"TRUE\""));
        assert!(sb.contains("<TR>"));
        assert!(sb.contains("<TD>"));
    }

    #[test]
    fn get_min_xy_basic() {
        let pts = vec![
            XPoint2D::new(10.0, 30.0),
            XPoint2D::new(5.0, 40.0),
            XPoint2D::new(20.0, 10.0),
        ];
        let min = get_min_xy(&pts);
        assert_eq!(min, XPoint2D::new(5.0, 10.0));
    }

    #[test]
    fn get_max_xy_basic() {
        let pts = vec![
            XPoint2D::new(10.0, 30.0),
            XPoint2D::new(5.0, 40.0),
            XPoint2D::new(20.0, 10.0),
        ];
        let max = get_max_xy(&pts);
        assert_eq!(max, XPoint2D::new(20.0, 40.0));
    }

    #[test]
    fn get_min_max_single() {
        let pts = vec![XPoint2D::new(7.0, 3.0)];
        assert_eq!(get_min_xy(&pts), XPoint2D::new(7.0, 3.0));
        assert_eq!(get_max_xy(&pts), XPoint2D::new(7.0, 3.0));
    }

    // ── SVG parsing integration ──

    #[test]
    fn solve_line_invis() {
        let mut e = SvekEdge::new("A", "B");
        e.is_invis = true;
        let svg = crate::svek::svg_result::SvgResult::new(String::new());
        e.solve_line(&svg);
        assert!(e.dot_path.is_none());
    }

    #[test]
    fn solve_line_color_not_found() {
        let mut e = SvekEdge::new("A", "B");
        e.color = 0xFF0000;
        let svg =
            crate::svek::svg_result::SvgResult::new("<svg></svg>".to_string());
        e.solve_line(&svg);
        assert!(e.dot_path.is_none());
    }

    #[test]
    fn solve_line_with_path() {
        let mut e = SvekEdge::new("A", "B");
        e.color = 0x010200;
        let svg_str =
            r##"<path stroke="#010200" d="M 10,20 C 30,40 50,60 70,80"/>"##;
        let svg = crate::svek::svg_result::SvgResult::new(svg_str.to_string());
        e.solve_line(&svg);
        assert!(
            e.dot_path.is_some(),
            "dot_path should be set after solve_line"
        );
        let path = e.dot_path.as_ref().unwrap();
        assert_eq!(path.start_point(), XPoint2D::new(10.0, 20.0));
        assert_eq!(path.end_point(), XPoint2D::new(70.0, 80.0));
    }

    // ── Builder chain test ──

    #[test]
    fn builder_chain() {
        let e = SvekEdge::new("Foo", "Bar")
            .with_ports(
                EntityPort::with_port("Foo", "e"),
                EntityPort::with_port("Bar", "w"),
            )
            .with_colors(0x0A0B0C, 0x0D0E0F, 0x101112, 0x131415)
            .with_decorations(LinkDecoration::Composition, LinkDecoration::Arrow)
            .with_style(LinkStyle::Dashed)
            .with_length(3)
            .with_label("uses", LabelDimension::new(30.0, 12.0))
            .with_tail_label("1", LabelDimension::new(10.0, 12.0))
            .with_head_label("*", LabelDimension::new(10.0, 12.0));

        assert_eq!(e.start_uid.full_string(), "Foo:e");
        assert_eq!(e.end_uid.full_string(), "Bar:w");
        assert_eq!(e.color, 0x0A0B0C);
        assert_eq!(e.decor1, LinkDecoration::Composition);
        assert_eq!(e.decor2, LinkDecoration::Arrow);
        assert_eq!(e.link_style, LinkStyle::Dashed);
        assert_eq!(e.link_length, 3);
        assert_eq!(e.label.as_deref(), Some("uses"));
        assert_eq!(e.start_tail_text.as_deref(), Some("1"));
        assert_eq!(e.end_head_text.as_deref(), Some("*"));
    }

    // ── Full DOT output integration ──

    #[test]
    fn full_dot_output() {
        let e = SvekEdge::new("ClassA", "ClassB")
            .with_ports(
                EntityPort::with_port("ClassA", "south"),
                EntityPort::with_port("ClassB", "north"),
            )
            .with_colors(0x010200, 0x020300, 0x030400, 0x040500)
            .with_decorations(LinkDecoration::Composition, LinkDecoration::Arrow)
            .with_style(LinkStyle::Dashed)
            .with_length(2)
            .with_label("extends", LabelDimension::new(50.0, 14.0));

        let mut sb = String::new();
        e.append_line(&mut sb);

        assert!(sb.starts_with("ClassA:south->ClassB:north["));
        assert!(sb.contains("arrowhead=diamond"));
        assert!(sb.contains("arrowtail=open"));
        assert!(sb.contains("dir=both"));
        assert!(sb.contains("style=dashed"));
        assert!(sb.contains("minlen=1"));
        assert!(sb.contains("color=\"#010200\""));
        assert!(sb.contains("label=<"));
        assert!(sb.contains("BGCOLOR=\"#020300\""));
        assert!(sb.contains("WIDTH=\"50\" HEIGHT=\"14\""));
        assert!(sb.ends_with("];\n"));
    }

    // ── label_shield test ──

    #[test]
    fn label_shield_enlarges_dimension() {
        let mut e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0x020300, 0, 0)
            .with_label("x", LabelDimension::new(50.0, 20.0));
        e.label_shield = 7.0;
        let mut sb = String::new();
        e.append_line(&mut sb);
        // Shield adds 2*7=14 to each dimension
        assert!(
            sb.contains("WIDTH=\"64\" HEIGHT=\"34\""),
            "expected shield-enlarged dimensions, got: {}",
            sb
        );
    }

    // ── divide_label_width_by_two test ──

    #[test]
    fn divide_label_width_by_two() {
        let mut e = SvekEdge::new("A", "B")
            .with_colors(0x010200, 0x020300, 0, 0)
            .with_label("x", LabelDimension::new(100.0, 20.0));
        e.divide_label_width_by_two = true;
        let mut sb = String::new();
        e.append_line(&mut sb);
        assert!(
            sb.contains("WIDTH=\"50\" HEIGHT=\"20\""),
            "expected halved width, got: {}",
            sb
        );
    }

    // ── arrow_direction_radian test ──

    #[test]
    fn arrow_direction_radian_straight_down() {
        let mut e = SvekEdge::new("A", "B");
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            50.0, 0.0, 50.0, 33.0, 50.0, 66.0, 50.0, 100.0,
        )]));
        let angle = e.arrow_direction_radian().unwrap();
        assert!(angle.abs() < 0.01, "expected ~0 rad, got: {}", angle);
    }

    #[test]
    fn arrow_direction_radian_backward() {
        let mut e = SvekEdge::new("A", "B");
        e.link_arrow = LinkArrow::Backward;
        e.dot_path = Some(DotPath::from_beziers(vec![XCubicCurve2D::new(
            50.0, 0.0, 50.0, 33.0, 50.0, 66.0, 50.0, 100.0,
        )]));
        let angle = e.arrow_direction_radian().unwrap();
        assert!(
            (angle - std::f64::consts::PI).abs() < 0.01,
            "expected ~PI rad, got: {}",
            angle
        );
    }
}
