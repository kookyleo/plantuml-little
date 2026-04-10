// svek::node - Graph node representation for Graphviz layout
// Port of Java PlantUML's svek.SvekNode

use crate::klimt::geom::{RectangleArea, XDimension2D, XPoint2D};

use super::shape_type::ShapeType;
use super::{ColorSequence, Margins};

/// Format f64 matching Java's Double.toString():
/// - Integer values get ".0" suffix: 0.0 → "0.0", 1.0 → "1.0"
/// - Non-integer values display normally: 18.296875 → "18.296875"
fn java_double_to_string(v: f64) -> String {
    if v == v.floor() && v.is_finite() {
        format!("{:.1}", v)
    } else {
        format!("{}", v)
    }
}

// ── EntityPosition ──────────────────────────────────────────────────

/// Entity position within a diagram (normal vs boundary/port).
/// Java: `abel.EntityPosition`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityPosition {
    #[default]
    Normal,
    EntryPoint,
    ExitPoint,
    InputPin,
    OutputPin,
    ExpansionInput,
    ExpansionOutput,
    PortIn,
    PortOut,
}

impl EntityPosition {
    pub const RADIUS: f64 = 6.0;

    pub fn is_input(&self) -> bool {
        matches!(
            self,
            Self::EntryPoint | Self::InputPin | Self::ExpansionInput | Self::PortIn
        )
    }

    pub fn is_output(&self) -> bool {
        matches!(
            self,
            Self::ExitPoint | Self::OutputPin | Self::ExpansionOutput | Self::PortOut
        )
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Whether this position is a port type.
    /// Java: `EntityPosition.isPort()`
    pub fn is_port(&self) -> bool {
        matches!(self, Self::PortIn | Self::PortOut)
    }

    /// Whether this position uses port-style DOT addressing.
    /// Java: `EntityPosition.usePortP()`
    pub fn use_port_p(&self) -> bool {
        self.is_port() || matches!(self, Self::ExitPoint | Self::EntryPoint)
    }

    /// Resolve an `EntityPosition` from a stereotype label string.
    /// Java: `EntityPosition.fromStereotype(String)`
    pub fn from_stereotype(label: &str) -> Self {
        let lower = label.to_lowercase();
        if lower == "<<entrypoint>>" {
            Self::EntryPoint
        } else if lower == "<<exitpoint>>" {
            Self::ExitPoint
        } else if lower == "<<inputpin>>" {
            Self::InputPin
        } else if lower == "<<outputpin>>" {
            Self::OutputPin
        } else if lower == "<<expansioninput>>" {
            Self::ExpansionInput
        } else if lower == "<<expansionoutput>>" {
            Self::ExpansionOutput
        } else {
            Self::Normal
        }
    }
}

// ── Together ────────────────────────────────────────────────────────

/// Group identifier for "together" layout grouping.
/// Java: `abel.Together`
///
/// Nodes in the same Together group are placed in the same DOT subgraph
/// to keep them adjacent in the layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Together {
    pub id: usize,
    pub parent: Option<Box<Together>>,
}

impl Together {
    pub fn new(id: usize, parent: Option<Together>) -> Self {
        Self {
            id,
            parent: parent.map(Box::new),
        }
    }
}

// ── PortGeometry ────────────────────────────────────────────────────

/// Geometry for a single port in an HTML-for-ports node.
/// Java: `svek.PortGeometry`
#[derive(Debug, Clone)]
pub struct PortGeometry {
    pub id: String,
    pub position: f64,
    pub height: f64,
}

impl PortGeometry {
    pub fn new(id: &str, position: f64, height: f64) -> Self {
        Self {
            id: id.to_string(),
            position,
            height,
        }
    }
}

// ── SvekNode ────────────────────────────────────────────────────────

/// A node in the Graphviz layout graph.
/// Java: `svek.SvekNode`
///
/// Holds the entity's dimensions (for DOT input) and
/// positioned coordinates (from SVG output parsing).
#[derive(Debug, Clone)]
pub struct SvekNode {
    /// Unique identifier for DOT, format "shNNNN"
    pub uid: String,
    /// Node width in pixels
    pub width: f64,
    /// Node height in pixels
    pub height: f64,
    /// Top-left X after Graphviz layout
    pub min_x: f64,
    /// Top-left Y after Graphviz layout
    pub min_y: f64,
    /// DOT color (RGB integer) for SVG matching
    pub color: u32,
    /// Shape type for DOT shape attribute
    pub shape_type: ShapeType,
    /// Cluster membership (if any)
    pub cluster_id: Option<String>,
    /// Entity position (normal, entry/exit point, pin, etc.)
    pub entity_position: EntityPosition,
    /// Together group for adjacency layout
    pub together: Option<Together>,
    /// Shield margins around the node (for qualified associations)
    pub shield: Margins,
    /// Whether this node is explicitly shielded
    pub shielded: bool,
    /// Whether this node is hidden
    pub hidden: bool,
    /// Port geometries for HTML-for-ports nodes
    pub ports: Vec<PortGeometry>,
    /// Max label width for entry/exit ports
    pub max_label_width: f64,
    /// Width of the external port label used by LimitFinder simulation.
    pub port_label_width: f64,
    /// Polygon points after SVG parsing (translated to node-local coords)
    pub polygon: Option<Vec<XPoint2D>>,
    /// Center X from SVG parsing (used to derive min_x)
    pub cx: f64,
    /// Center Y from SVG parsing (used to derive min_y)
    pub cy: f64,
    /// Extra LimitFinder left extension from entity image content (e.g.
    /// visibility modifier polygons with HACK_X_FOR_POLYGON=10).
    /// Applied as: lf_min_x = min(node_min_x - 1, node_min_x - 1 - extra).
    pub lf_extra_left: f64,
    /// Whether LimitFinder.drawRectangle -1 correction applies.
    /// False for notes (drawn with UPath instead of URectangle).
    pub lf_rect_correction: bool,
    /// Whether the entity draws a full-width body separator ULine.
    pub lf_has_body_separator: bool,
    pub lf_node_polygon: bool,
    pub lf_polygon_hack: bool,
    /// Actor stickman: LimitFinder uses min_corr_y = -0.5, max_corr = (0, 0).
    pub lf_actor_stickman: bool,
}

impl SvekNode {
    fn append_quoted_uid(&self, sb: &mut String) {
        sb.push('"');
        sb.push_str(&self.uid);
        sb.push('"');
    }

    /// Create a new node with given uid and dimensions.
    pub fn new(uid: &str, width: f64, height: f64) -> Self {
        Self {
            uid: uid.to_string(),
            width,
            height,
            min_x: 0.0,
            min_y: 0.0,
            color: 0,
            shape_type: ShapeType::Rectangle,
            cluster_id: None,
            entity_position: EntityPosition::Normal,
            together: None,
            shield: Margins::none(),
            shielded: false,
            hidden: false,
            ports: Vec::new(),
            max_label_width: 0.0,
            port_label_width: 0.0,
            polygon: None,
            cx: 0.0,
            cy: 0.0,
            lf_extra_left: 0.0,
            lf_rect_correction: true,
            lf_has_body_separator: false,
            lf_node_polygon: false,
            lf_polygon_hack: false,
            lf_actor_stickman: false,
        }
    }

    /// Create a node from a ColorSequence (assigns color + uid).
    /// Java: `SvekNode(Entity ent, IEntityImage image, ColorSequence colorSequence, ...)`
    pub fn with_color_sequence(
        width: f64,
        height: f64,
        shape_type: ShapeType,
        color_seq: &mut ColorSequence,
    ) -> Self {
        let color = color_seq.next_color();
        let uid = format!("sh{:04}", color & 0xFFFF);
        Self {
            uid,
            width,
            height,
            min_x: 0.0,
            min_y: 0.0,
            color,
            shape_type,
            cluster_id: None,
            entity_position: EntityPosition::Normal,
            together: None,
            shield: Margins::none(),
            shielded: false,
            hidden: false,
            ports: Vec::new(),
            max_label_width: 0.0,
            port_label_width: 0.0,
            polygon: None,
            cx: 0.0,
            cy: 0.0,
            lf_extra_left: 0.0,
            lf_rect_correction: true,
            lf_has_body_separator: false,
            lf_node_polygon: false,
            lf_polygon_hack: false,
            lf_actor_stickman: false,
        }
    }

    /// Node dimension.
    pub fn dimension(&self) -> XDimension2D {
        XDimension2D::new(self.width, self.height)
    }

    /// Top-left x after layout (from center).
    pub fn x(&self) -> f64 {
        self.min_x
    }

    /// Top-left y after layout (from center).
    pub fn y(&self) -> f64 {
        self.min_y
    }

    /// Position as a point. Java: `getPosition()`
    pub fn position(&self) -> XPoint2D {
        XPoint2D::new(self.min_x, self.min_y)
    }

    /// Size as dimension. Java: `getSize()`
    pub fn size(&self) -> XDimension2D {
        XDimension2D::new(self.width, self.height)
    }

    /// Rectangle area. Java: `getRectangleArea()`
    pub fn rectangle_area(&self) -> RectangleArea {
        RectangleArea {
            min_x: self.min_x,
            min_y: self.min_y,
            max_x: self.min_x + self.width,
            max_y: self.min_y + self.height,
        }
    }

    /// Get a point offset from the node's top-left position.
    /// Java: `getPoint2D(double x, double y)`
    pub fn point_2d(&self, x: f64, y: f64) -> XPoint2D {
        XPoint2D::new(self.min_x + x, self.min_y + y)
    }

    /// Reset position to origin. Java: `resetMove()`
    pub fn reset_move(&mut self) {
        self.min_x = 0.0;
        self.min_y = 0.0;
    }

    /// Move position by delta. Java: `moveDelta(double, double)`
    pub fn move_delta(&mut self, dx: f64, dy: f64) {
        self.min_x += dx;
        self.min_y += dy;
    }

    /// Set position from SVG center coordinates.
    /// Converts from center to top-left.
    pub fn set_center_position(&mut self, cx: f64, cy: f64) {
        self.cx = cx;
        self.cy = cy;
        self.min_x = cx - self.width / 2.0;
        self.min_y = cy - self.height / 2.0;
    }

    /// Store polygon points from SVG parsing (translated to node-local coords).
    /// Java: `setPolygon(double minX, double minY, List<XPoint2D> points)`
    pub fn set_polygon(&mut self, min_x: f64, min_y: f64, points: &[XPoint2D]) {
        self.polygon = Some(
            points
                .iter()
                .map(|p| XPoint2D::new(p.x - min_x, p.y - min_y))
                .collect(),
        );
    }

    /// Whether the node is shielded (has qualifier labels outside its bounding box).
    /// Java: `isShielded()`
    pub fn is_shielded(&self) -> bool {
        if self.shielded {
            return true;
        }
        !self.shield.is_zero()
    }

    // ── DOT generation ──────────────────────────────────────────────

    /// Generate the DOT shape declaration for this node.
    /// Java: `appendShape(StringBuilder sb, StringBounder stringBounder)`
    pub fn append_shape(&self, sb: &mut String) {
        match self.shape_type {
            ShapeType::RectangleHtmlForPorts => {
                self.append_label_html_special_for_link(sb);
                sb.push('\n');
                return;
            }
            ShapeType::RectanglePort => {
                self.append_label_html_special_for_port(sb);
                sb.push('\n');
                return;
            }
            ShapeType::RectangleWithCircleInside => {
                self.append_html(sb);
                sb.push('\n');
                return;
            }
            ShapeType::Rectangle if self.is_shielded() => {
                self.append_html(sb);
                sb.push('\n');
                return;
            }
            _ => {}
        }

        // Quote node UID to handle dots, spaces, and special characters
        self.append_quoted_uid(sb);
        sb.push_str(" [");
        self.append_shape_internal(sb);
        sb.push(',');
        sb.push_str("label=\"\"");
        sb.push(',');
        sb.push_str(&format!("width={}", super::utils::px_to_dot(self.width)));
        sb.push(',');
        sb.push_str(&format!("height={}", super::utils::px_to_dot(self.height)));
        sb.push(',');
        sb.push_str(&format!(
            "color=\"{}\"",
            ColorSequence::color_to_hex(self.color)
        ));
        sb.push_str("];");
        sb.push('\n');
    }

    /// Append DOT shape attribute. Java: `appendShapeInternal(StringBuilder)`
    fn append_shape_internal(&self, sb: &mut String) {
        match self.shape_type {
            ShapeType::Rectangle if self.is_shielded() => {
                // Should not reach here - handled in append_shape
                panic!("shielded Rectangle should use HTML label");
            }
            ShapeType::Rectangle | ShapeType::RectangleWithCircleInside | ShapeType::Folder => {
                sb.push_str("shape=rect");
            }
            ShapeType::RectangleHtmlForPorts => {
                panic!("RectangleHtmlForPorts should use HTML label");
            }
            ShapeType::Octagon => {
                sb.push_str("shape=octagon");
            }
            ShapeType::Hexagon => {
                sb.push_str("shape=hexagon");
            }
            ShapeType::Diamond => {
                sb.push_str("shape=diamond");
            }
            ShapeType::Circle => {
                sb.push_str("shape=circle");
            }
            ShapeType::Oval => {
                sb.push_str("shape=ellipse");
            }
            ShapeType::RoundRectangle => {
                sb.push_str("shape=rect,style=rounded");
            }
            _ => {
                panic!("unsupported shape type: {:?}", self.shape_type);
            }
        }
    }

    /// Generate HTML table label for shielded/circle-inside nodes.
    /// Java: `appendHtml(StringBuilder)`
    fn append_html(&self, sb: &mut String) {
        self.append_quoted_uid(sb);
        sb.push_str(" [");
        sb.push_str("shape=plaintext,");
        sb.push_str("label=<");
        self.append_label_html(sb);
        sb.push('>');
        sb.push_str("];");
        sb.push('\n');
    }

    /// Generate the inner HTML table for shielded nodes.
    /// Java: `appendLabelHtml(StringBuilder)`
    fn append_label_html(&self, sb: &mut String) {
        let shield = &self.shield;
        sb.push_str("<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"0\">");

        // Top row
        sb.push_str("<TR>");
        Self::append_td_empty(sb);
        Self::append_td_sized(sb, 1.0, shield.y1);
        Self::append_td_empty(sb);
        sb.push_str("</TR>");

        // Middle row with main cell
        sb.push_str("<TR>");
        Self::append_td_sized(sb, shield.x1, 1.0);
        sb.push_str(&format!(
            "<TD BGCOLOR=\"{}\" FIXEDSIZE=\"TRUE\" WIDTH=\"{}\" HEIGHT=\"{}\" PORT=\"h\">",
            ColorSequence::color_to_hex(self.color),
            java_double_to_string(self.width),
            java_double_to_string(self.height)
        ));
        sb.push_str("</TD>");
        Self::append_td_sized(sb, shield.x2, 1.0);
        sb.push_str("</TR>");

        // Bottom row
        sb.push_str("<TR>");
        Self::append_td_empty(sb);
        Self::append_td_sized(sb, 1.0, shield.y2);
        Self::append_td_empty(sb);
        sb.push_str("</TR>");

        sb.push_str("</TABLE>");
    }

    /// Generate HTML table label for ports (links).
    /// Java: `appendLabelHtmlSpecialForLink(StringBuilder, StringBounder)`
    fn append_label_html_special_for_link(&self, sb: &mut String) {
        self.append_quoted_uid(sb);
        sb.push_str(" [");
        sb.push_str("shape=plaintext,");
        sb.push_str("label=<");

        sb.push_str(&format!(
            "<TABLE BGCOLOR=\"{}\" BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"0\">",
            ColorSequence::color_to_hex(self.color)
        ));

        let mut sum: i32 = 0;
        for geom in &self.ports {
            let missing = (geom.position as i32) - sum;
            sum += missing;
            Self::append_tr(sb, None, missing, self.width);

            let int_height = geom.height as i32;
            Self::append_tr(sb, Some(&geom.id), int_height, self.width);
            sum += int_height;
        }

        let diff = (self.height as i32) - sum;
        Self::append_tr(sb, None, diff, self.width);

        sb.push_str("</TABLE>");
        sb.push('>');
        sb.push_str("];");
        sb.push('\n');
    }

    /// Generate HTML label for port-type nodes.
    /// Java: `appendLabelHtmlSpecialForPort(StringBuilder, StringBounder)`
    fn append_label_html_special_for_port(&self, sb: &mut String) {
        let width2 = self.max_label_width as i32;
        if width2 > 40 {
            self.append_label_html_special_for_port_html(sb, width2 - 40);
        } else {
            self.append_label_html_special_for_port_basic(sb);
        }
    }

    /// HTML table version for port nodes with wide labels.
    /// Java: `appendLabelHtmlSpecialForPortHtml(StringBuilder, StringBounder, int)`
    fn append_label_html_special_for_port_html(&self, sb: &mut String, mut full_width: i32) {
        if full_width < 10 {
            full_width = 10;
        }

        self.append_quoted_uid(sb);
        sb.push_str(" [");
        sb.push_str("shape=plaintext");
        sb.push(',');
        sb.push_str("label=<");

        sb.push_str("<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"0\">");
        sb.push_str(&format!(
            "<TR><TD WIDTH=\"{}\" HEIGHT=\"1\" COLSPAN=\"3\"></TD></TR>",
            full_width
        ));
        sb.push_str(&format!(
            "<TR><TD></TD><TD FIXEDSIZE=\"TRUE\" PORT=\"P\"  BORDER=\"1\" COLOR=\"{}\" WIDTH=\"{}\" HEIGHT=\"{}\"></TD><TD></TD></TR>",
            ColorSequence::color_to_hex(self.color),
            self.width as i32,
            self.height as i32
        ));
        sb.push_str(&format!(
            "<TR><TD WIDTH=\"{}\" HEIGHT=\"1\" COLSPAN=\"3\"></TD></TR>",
            full_width
        ));
        sb.push_str("</TABLE>");

        sb.push_str(">];");
    }

    /// Simple rect version for port nodes with narrow labels.
    /// Java: `appendLabelHtmlSpecialForPortBasic(StringBuilder, StringBounder)`
    fn append_label_html_special_for_port_basic(&self, sb: &mut String) {
        self.append_quoted_uid(sb);
        sb.push_str(" [");
        sb.push_str("shape=rect");
        sb.push(',');
        sb.push_str("label=\"\"");
        sb.push(',');
        sb.push_str(&format!("width={}", super::utils::px_to_dot(self.width)));
        sb.push(',');
        sb.push_str(&format!("height={}", super::utils::px_to_dot(self.height)));
        sb.push(',');
        sb.push_str(&format!(
            "color=\"{}\"",
            ColorSequence::color_to_hex(self.color)
        ));
        sb.push_str("];");
    }

    /// Append a table row element for port HTML labels.
    /// Java: `appendTr(StringBuilder, String portId, int height)`
    fn append_tr(sb: &mut String, port_id: Option<&str>, height: i32, width: f64) {
        if height <= 0 {
            return;
        }

        sb.push_str("<TR>");
        sb.push_str("<TD ");
        sb.push_str(&format!(
            " FIXEDSIZE=\"TRUE\" WIDTH=\"{}\" HEIGHT=\"{}\"",
            width, height
        ));
        if let Some(pid) = port_id {
            sb.push_str(&format!(" PORT=\"{}\"", pid));
        }
        sb.push('>');
        sb.push_str("</TD>");
        sb.push_str("</TR>");
    }

    /// Append a sized TD element.
    /// Java: `appendTd(StringBuilder, double w, double h)`
    fn append_td_sized(sb: &mut String, w: f64, h: f64) {
        sb.push_str("<TD");
        sb.push_str(&format!(
            " FIXEDSIZE=\"TRUE\" WIDTH=\"{}\" HEIGHT=\"{}\"",
            java_double_to_string(w),
            java_double_to_string(h),
        ));
        sb.push('>');
        sb.push_str("</TD>");
    }

    /// Append an empty TD element.
    /// Java: `appendTd(StringBuilder)`
    fn append_td_empty(sb: &mut String) {
        sb.push_str("<TD>");
        sb.push_str("</TD>");
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_new_defaults() {
        let n = SvekNode::new("sh0001", 100.0, 50.0);
        assert_eq!(n.uid, "sh0001");
        assert_eq!(n.width, 100.0);
        assert_eq!(n.height, 50.0);
        assert_eq!(n.min_x, 0.0);
        assert_eq!(n.min_y, 0.0);
        assert_eq!(n.color, 0);
        assert_eq!(n.shape_type, ShapeType::Rectangle);
        assert!(n.cluster_id.is_none());
        assert_eq!(n.entity_position, EntityPosition::Normal);
        assert!(n.together.is_none());
        assert!(!n.hidden);
        assert!(n.ports.is_empty());
        assert!(n.polygon.is_none());
    }

    #[test]
    fn node_with_color_sequence() {
        let mut cs = ColorSequence::new();
        let n = SvekNode::with_color_sequence(120.0, 60.0, ShapeType::Diamond, &mut cs);
        assert!(!n.uid.is_empty());
        assert!(n.uid.starts_with("sh"));
        assert_ne!(n.color, 0);
        assert_eq!(n.width, 120.0);
        assert_eq!(n.height, 60.0);
        assert_eq!(n.shape_type, ShapeType::Diamond);
    }

    #[test]
    fn node_position_from_center() {
        let mut n = SvekNode::new("test", 100.0, 50.0);
        n.set_center_position(150.0, 125.0);
        assert_eq!(n.x(), 100.0);
        assert_eq!(n.y(), 100.0);
        assert_eq!(n.cx, 150.0);
        assert_eq!(n.cy, 125.0);
    }

    #[test]
    fn node_dimension() {
        let n = SvekNode::new("d", 80.0, 40.0);
        let dim = n.dimension();
        assert_eq!(dim.width, 80.0);
        assert_eq!(dim.height, 40.0);
    }

    #[test]
    fn node_rectangle_area() {
        let mut n = SvekNode::new("ra", 100.0, 50.0);
        n.min_x = 10.0;
        n.min_y = 20.0;
        let ra = n.rectangle_area();
        assert_eq!(ra.min_x, 10.0);
        assert_eq!(ra.min_y, 20.0);
        assert_eq!(ra.max_x, 110.0);
        assert_eq!(ra.max_y, 70.0);
    }

    #[test]
    fn node_point_2d() {
        let mut n = SvekNode::new("p", 100.0, 50.0);
        n.min_x = 10.0;
        n.min_y = 20.0;
        let pt = n.point_2d(5.0, 10.0);
        assert_eq!(pt.x, 15.0);
        assert_eq!(pt.y, 30.0);
    }

    #[test]
    fn node_reset_move() {
        let mut n = SvekNode::new("rm", 100.0, 50.0);
        n.min_x = 42.0;
        n.min_y = 17.0;
        n.reset_move();
        assert_eq!(n.min_x, 0.0);
        assert_eq!(n.min_y, 0.0);
    }

    #[test]
    fn node_move_delta() {
        let mut n = SvekNode::new("md", 100.0, 50.0);
        n.min_x = 10.0;
        n.min_y = 20.0;
        n.move_delta(5.0, -3.0);
        assert_eq!(n.min_x, 15.0);
        assert_eq!(n.min_y, 17.0);
    }

    #[test]
    fn node_set_polygon() {
        let mut n = SvekNode::new("poly", 100.0, 50.0);
        let points = vec![
            XPoint2D::new(10.0, 20.0),
            XPoint2D::new(110.0, 20.0),
            XPoint2D::new(110.0, 70.0),
            XPoint2D::new(10.0, 70.0),
        ];
        n.set_polygon(10.0, 20.0, &points);
        let poly = n.polygon.as_ref().unwrap();
        assert_eq!(poly.len(), 4);
        assert_eq!(poly[0], XPoint2D::new(0.0, 0.0));
        assert_eq!(poly[1], XPoint2D::new(100.0, 0.0));
        assert_eq!(poly[2], XPoint2D::new(100.0, 50.0));
        assert_eq!(poly[3], XPoint2D::new(0.0, 50.0));
    }

    // ── DOT shape generation tests ──────────────────────────────────

    #[test]
    fn append_shape_rectangle() {
        let mut n = SvekNode::new("sh0001", 100.0, 50.0);
        n.color = 0x010100;
        n.shape_type = ShapeType::Rectangle;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("\"sh0001\" ["));
        assert!(dot.contains("shape=rect"));
        assert!(dot.contains("label=\"\""));
        assert!(dot.contains("width="));
        assert!(dot.contains("height="));
        assert!(dot.contains("color=\"#010100\""));
        assert!(dot.ends_with("];\n"));
    }

    #[test]
    fn append_shape_diamond() {
        let mut n = SvekNode::new("sh0002", 80.0, 80.0);
        n.color = 0x020200;
        n.shape_type = ShapeType::Diamond;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=diamond"));
        assert!(dot.contains("color=\"#020200\""));
    }

    #[test]
    fn append_shape_circle() {
        let mut n = SvekNode::new("sh0003", 40.0, 40.0);
        n.color = 0x030300;
        n.shape_type = ShapeType::Circle;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=circle"));
    }

    #[test]
    fn append_shape_oval() {
        let mut n = SvekNode::new("sh0004", 60.0, 30.0);
        n.color = 0x040400;
        n.shape_type = ShapeType::Oval;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=ellipse"));
    }

    #[test]
    fn append_shape_round_rectangle() {
        let mut n = SvekNode::new("sh0005", 90.0, 45.0);
        n.color = 0x050500;
        n.shape_type = ShapeType::RoundRectangle;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=rect,style=rounded"));
    }

    #[test]
    fn append_shape_octagon() {
        let mut n = SvekNode::new("sh0006", 70.0, 70.0);
        n.color = 0x060600;
        n.shape_type = ShapeType::Octagon;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=octagon"));
    }

    #[test]
    fn append_shape_hexagon() {
        let mut n = SvekNode::new("sh0007", 70.0, 70.0);
        n.color = 0x070700;
        n.shape_type = ShapeType::Hexagon;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=hexagon"));
    }

    #[test]
    fn append_shape_folder() {
        let mut n = SvekNode::new("sh0008", 120.0, 80.0);
        n.color = 0x080800;
        n.shape_type = ShapeType::Folder;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=rect"));
    }

    #[test]
    fn append_shape_width_height_inches() {
        // 72 pixels = 1 inch
        let mut n = SvekNode::new("sh0009", 72.0, 144.0);
        n.color = 0x090900;
        n.shape_type = ShapeType::Rectangle;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("width=1.000000"));
        assert!(dot.contains("height=2.000000"));
    }

    // ── Shielded rectangle tests ────────────────────────────────────

    #[test]
    fn append_shape_shielded_rectangle() {
        let mut n = SvekNode::new("sh0010", 100.0, 50.0);
        n.color = 0x0A0A00;
        n.shape_type = ShapeType::Rectangle;
        n.shield = Margins::new(5.0, 10.0, 3.0, 7.0);
        let mut dot = String::new();
        n.append_shape(&mut dot);
        // Should use HTML label, not simple rect
        assert!(dot.contains("shape=plaintext"));
        assert!(dot.contains("label=<"));
        assert!(dot.contains("<TABLE"));
        assert!(dot.contains("BGCOLOR"));
        assert!(dot.contains("PORT=\"h\""));
        assert!(dot.contains("#0a0a00"));
    }

    #[test]
    fn append_shape_rectangle_with_circle_inside() {
        let mut n = SvekNode::new("sh0011", 60.0, 60.0);
        n.color = 0x0B0B00;
        n.shape_type = ShapeType::RectangleWithCircleInside;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=plaintext"));
        assert!(dot.contains("label=<"));
    }

    // ── HTML for ports tests ────────────────────────────────────────

    #[test]
    fn append_shape_html_for_ports() {
        let mut n = SvekNode::new("sh0012", 100.0, 80.0);
        n.color = 0x0C0C00;
        n.shape_type = ShapeType::RectangleHtmlForPorts;
        n.ports.push(PortGeometry::new("p1", 0.0, 20.0));
        n.ports.push(PortGeometry::new("p2", 40.0, 20.0));
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=plaintext"));
        assert!(dot.contains("BGCOLOR=\"#0c0c00\""));
        assert!(dot.contains("PORT=\"p1\""));
        assert!(dot.contains("PORT=\"p2\""));
    }

    #[test]
    fn append_shape_html_for_ports_no_ports() {
        let mut n = SvekNode::new("sh0013", 100.0, 80.0);
        n.color = 0x0D0D00;
        n.shape_type = ShapeType::RectangleHtmlForPorts;
        // No ports
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=plaintext"));
        assert!(dot.contains("<TABLE"));
    }

    // ── Rectangle port tests ────────────────────────────────────────

    #[test]
    fn append_shape_rectangle_port_narrow_label() {
        let mut n = SvekNode::new("sh0014", 20.0, 20.0);
        n.color = 0x0E0E00;
        n.shape_type = ShapeType::RectanglePort;
        n.max_label_width = 30.0; // <= 40, use basic
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=rect"));
        assert!(dot.contains("label=\"\""));
        assert!(dot.contains("color=\"#0e0e00\""));
    }

    #[test]
    fn append_shape_rectangle_port_wide_label() {
        let mut n = SvekNode::new("sh0015", 20.0, 20.0);
        n.color = 0x0F0F00;
        n.shape_type = ShapeType::RectanglePort;
        n.max_label_width = 60.0; // > 40, use HTML
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert!(dot.contains("shape=plaintext"));
        assert!(dot.contains("PORT=\"P\""));
        assert!(dot.contains("BORDER=\"1\""));
        assert!(dot.contains("COLOR=\"#0f0f00\""));
    }

    #[test]
    fn append_shape_rectangle_port_html_min_width() {
        let mut n = SvekNode::new("sh0016", 20.0, 20.0);
        n.color = 0x101000;
        n.shape_type = ShapeType::RectanglePort;
        n.max_label_width = 42.0; // width2 - 40 = 2, clamped to 10
        let mut dot = String::new();
        n.append_shape(&mut dot);
        // full_width should be clamped to 10
        assert!(dot.contains("WIDTH=\"10\""));
    }

    // ── Shield / Margins tests ──────────────────────────────────────

    #[test]
    fn margins_is_zero() {
        assert!(Margins::none().is_zero());
        assert!(Margins::new(0.0, 0.0, 0.0, 0.0).is_zero());
        assert!(!Margins::new(1.0, 0.0, 0.0, 0.0).is_zero());
        assert!(!Margins::new(0.0, 1.0, 0.0, 0.0).is_zero());
        assert!(!Margins::new(0.0, 0.0, 1.0, 0.0).is_zero());
        assert!(!Margins::new(0.0, 0.0, 0.0, 1.0).is_zero());
        assert!(!Margins::uniform(5.0).is_zero());
    }

    #[test]
    fn is_shielded_default_false() {
        let n = SvekNode::new("test", 100.0, 50.0);
        assert!(!n.is_shielded());
    }

    #[test]
    fn is_shielded_with_margins() {
        let mut n = SvekNode::new("test", 100.0, 50.0);
        n.shield = Margins::new(5.0, 5.0, 5.0, 5.0);
        assert!(n.is_shielded());
    }

    #[test]
    fn is_shielded_explicit() {
        let mut n = SvekNode::new("test", 100.0, 50.0);
        n.shielded = true;
        assert!(n.is_shielded());
    }

    // ── EntityPosition tests ────────────────────────────────────────

    #[test]
    fn entity_position_input_output() {
        assert!(EntityPosition::Normal.is_normal());
        assert!(!EntityPosition::Normal.is_input());
        assert!(!EntityPosition::Normal.is_output());

        assert!(EntityPosition::EntryPoint.is_input());
        assert!(!EntityPosition::EntryPoint.is_output());

        assert!(EntityPosition::ExitPoint.is_output());
        assert!(!EntityPosition::ExitPoint.is_input());

        assert!(EntityPosition::InputPin.is_input());
        assert!(EntityPosition::OutputPin.is_output());

        assert!(EntityPosition::ExpansionInput.is_input());
        assert!(EntityPosition::ExpansionOutput.is_output());

        assert!(EntityPosition::PortIn.is_input());
        assert!(EntityPosition::PortOut.is_output());
    }

    #[test]
    fn entity_position_default_is_normal() {
        let ep = EntityPosition::default();
        assert_eq!(ep, EntityPosition::Normal);
    }

    // ── Together tests ──────────────────────────────────────────────

    #[test]
    fn together_basic() {
        let t = Together::new(1, None);
        assert_eq!(t.id, 1);
        assert!(t.parent.is_none());
    }

    #[test]
    fn together_with_parent() {
        let parent = Together::new(0, None);
        let child = Together::new(1, Some(parent.clone()));
        assert_eq!(child.id, 1);
        assert!(child.parent.is_some());
        assert_eq!(child.parent.as_ref().unwrap().id, 0);
    }

    // ── PortGeometry tests ──────────────────────────────────────────

    #[test]
    fn port_geometry_basic() {
        let pg = PortGeometry::new("port1", 10.0, 20.0);
        assert_eq!(pg.id, "port1");
        assert_eq!(pg.position, 10.0);
        assert_eq!(pg.height, 20.0);
    }

    // ── DOT output exact format tests ───────────────────────────────

    #[test]
    fn dot_output_format_rectangle_exact() {
        let mut n = SvekNode::new("sh0001", 72.0, 36.0);
        n.color = 0x010100;
        n.shape_type = ShapeType::Rectangle;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert_eq!(
            dot,
            "\"sh0001\" [shape=rect,label=\"\",width=1.000000,height=0.500000,color=\"#010100\"];\n"
        );
    }

    #[test]
    fn dot_output_format_diamond_exact() {
        let mut n = SvekNode::new("sh0002", 144.0, 72.0);
        n.color = 0x020200;
        n.shape_type = ShapeType::Diamond;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert_eq!(
            dot,
            "\"sh0002\" [shape=diamond,label=\"\",width=2.000000,height=1.000000,color=\"#020200\"];\n"
        );
    }

    #[test]
    fn dot_output_format_round_rect_exact() {
        let mut n = SvekNode::new("sh0003", 36.0, 36.0);
        n.color = 0x030300;
        n.shape_type = ShapeType::RoundRectangle;
        let mut dot = String::new();
        n.append_shape(&mut dot);
        assert_eq!(
            dot,
            "\"sh0003\" [shape=rect,style=rounded,label=\"\",width=0.500000,height=0.500000,color=\"#030300\"];\n"
        );
    }

    #[test]
    fn dot_output_shielded_html_structure() {
        let mut n = SvekNode::new("sh0100", 80.0, 40.0);
        n.color = 0x0A0A00;
        n.shape_type = ShapeType::Rectangle;
        n.shield = Margins::new(5.0, 10.0, 3.0, 7.0);
        let mut dot = String::new();
        n.append_shape(&mut dot);

        // Verify HTML table structure
        assert!(dot.starts_with("\"sh0100\" [shape=plaintext,label=<"));
        assert!(dot
            .contains("<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"0\">"));
        // Top row: shield.y1 = 3.0
        assert!(dot.contains("HEIGHT=\"3.0\""));
        // Middle row: shield.x1 = 5.0, main cell, shield.x2 = 10.0
        assert!(dot.contains("WIDTH=\"5.0\""));
        assert!(dot.contains("WIDTH=\"10.0\""));
        // Main cell
        assert!(dot.contains("BGCOLOR=\"#0a0a00\""));
        assert!(dot.contains("WIDTH=\"80.0\""));
        assert!(dot.contains("HEIGHT=\"40.0\""));
        assert!(dot.contains("PORT=\"h\""));
        // Bottom row: shield.y2 = 7.0
        assert!(dot.contains("HEIGHT=\"7.0\""));
    }

    #[test]
    fn dot_output_ports_with_geometry() {
        let mut n = SvekNode::new("sh0200", 100.0, 100.0);
        n.color = 0x0C0C00;
        n.shape_type = ShapeType::RectangleHtmlForPorts;
        n.ports.push(PortGeometry::new("p1", 0.0, 30.0));
        n.ports.push(PortGeometry::new("p2", 50.0, 30.0));
        let mut dot = String::new();
        n.append_shape(&mut dot);

        // p1 at position=0, height=30 -> no missing gap, then port row
        // p2 at position=50, sum=30 -> missing=20 gap, then port row
        // remaining = 100 - (30+20+30) = 20
        assert!(dot.contains("PORT=\"p1\""));
        assert!(dot.contains("PORT=\"p2\""));
        assert!(dot.contains("HEIGHT=\"20\"")); // gap between p1 and p2
        assert!(dot.contains("HEIGHT=\"30\"")); // port heights
    }

    #[test]
    fn dot_output_port_basic_rect() {
        let mut n = SvekNode::new("sh0300", 20.0, 20.0);
        n.color = 0x0E0E00;
        n.shape_type = ShapeType::RectanglePort;
        n.max_label_width = 30.0;
        let mut dot = String::new();
        n.append_shape(&mut dot);

        // Basic rect mode
        assert_eq!(
            dot,
            "\"sh0300\" [shape=rect,label=\"\",width=0.277778,height=0.277778,color=\"#0e0e00\"];\n"
        );
    }

    // ── Multiple color sequence nodes ───────────────────────────────

    #[test]
    fn multiple_nodes_unique_colors() {
        let mut cs = ColorSequence::new();
        let n1 = SvekNode::with_color_sequence(100.0, 50.0, ShapeType::Rectangle, &mut cs);
        let n2 = SvekNode::with_color_sequence(80.0, 40.0, ShapeType::Diamond, &mut cs);
        let n3 = SvekNode::with_color_sequence(60.0, 60.0, ShapeType::Circle, &mut cs);

        assert_ne!(n1.color, n2.color);
        assert_ne!(n2.color, n3.color);
        assert_ne!(n1.uid, n2.uid);
        assert_ne!(n2.uid, n3.uid);
    }

    // ── Position / size accessor consistency ────────────────────────

    #[test]
    fn position_and_size_consistent() {
        let mut n = SvekNode::new("s", 100.0, 50.0);
        n.min_x = 10.0;
        n.min_y = 20.0;

        let pos = n.position();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);

        let sz = n.size();
        assert_eq!(sz.width, 100.0);
        assert_eq!(sz.height, 50.0);

        let ra = n.rectangle_area();
        assert_eq!(ra.min_x, pos.x);
        assert_eq!(ra.min_y, pos.y);
        assert_eq!(ra.max_x, pos.x + sz.width);
        assert_eq!(ra.max_y, pos.y + sz.height);
    }
}
