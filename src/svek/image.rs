// svek::image - Entity image rendering abstractions
// Port of Java PlantUML's svek.IEntityImage, AbstractEntityImage, HeaderLayout,
// DirectionalTextBlock, InnerActivity, InnerStateAutonom

use crate::klimt::geom::XDimension2D;

// ── IEntityImage trait ──────────────────────────────────────────────

/// Interface for entity image rendering.
/// Java: `svek.IEntityImage`
pub trait IEntityImage {
    /// Corner radius for rounded shapes.
    const CORNER: f64 = 25.0;
    /// Margin around entity content.
    const MARGIN: f64 = 5.0;
    /// Margin for separator lines.
    const MARGIN_LINE: f64 = 5.0;

    /// Get the shape type for DOT generation.
    fn shape_type(&self) -> super::shape_type::ShapeType;

    /// Get dimensions of this entity image.
    fn dimension(&self) -> XDimension2D;

    /// Get shield margins.
    fn shield(&self) -> super::Margins {
        super::Margins::none()
    }

    /// Horizontal overscan (extra width for edge attachment).
    fn overscan_x(&self) -> f64 {
        0.0
    }

    /// Whether this image is hidden.
    fn is_hidden(&self) -> bool {
        false
    }

    /// Whether this image has crashed during rendering.
    fn is_crash(&self) -> bool {
        false
    }
}

// ── AbstractEntityImage ─────────────────────────────────────────────

/// Base struct for entity images providing common boilerplate.
/// Java: `svek.AbstractEntityImage`
///
/// Provides entity reference, skin parameters, and default implementations.
/// Concrete entity images embed this to inherit shared behavior.
#[derive(Debug, Clone)]
pub struct AbstractEntityImage {
    /// Entity identifier
    pub entity_id: String,
    /// Whether the entity is hidden
    pub hidden: bool,
    /// Background color (CSS hex string)
    pub back_color: Option<String>,
    /// Style name for diagram type
    pub style_name: Option<String>,
}

impl AbstractEntityImage {
    pub fn new(entity_id: &str) -> Self {
        Self {
            entity_id: entity_id.to_string(),
            hidden: false,
            back_color: None,
            style_name: None,
        }
    }

    pub fn with_back_color(mut self, color: &str) -> Self {
        self.back_color = Some(color.to_string());
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
}

// ── EntityImageDegenerated ──────────────────────────────────────────

/// Wraps a single-entity diagram image, providing background fill.
/// Java: `svek.EntityImageDegenerated`
///
/// Used when a diagram contains exactly one entity -- bypasses Graphviz
/// entirely and renders just that entity's image.
pub struct EntityImageDegenerated {
    pub inner_dim: XDimension2D,
    pub inner_shape: super::shape_type::ShapeType,
    pub back_color: Option<String>,
    pub hidden: bool,
}

impl EntityImageDegenerated {
    pub fn new(
        inner_dim: XDimension2D,
        inner_shape: super::shape_type::ShapeType,
        back_color: Option<String>,
    ) -> Self {
        Self {
            inner_dim,
            inner_shape,
            back_color,
            hidden: false,
        }
    }
}

impl IEntityImage for EntityImageDegenerated {
    fn shape_type(&self) -> super::shape_type::ShapeType {
        self.inner_shape
    }

    fn dimension(&self) -> XDimension2D {
        self.inner_dim
    }

    fn is_hidden(&self) -> bool {
        self.hidden
    }
}

// ── EntityImageSimpleEmpty ──────────────────────────────────────────

/// Minimal empty entity image (10x10 placeholder).
/// Java: `GraphvizImageBuilder.EntityImageSimpleEmpty`
///
/// Used when a diagram has zero entities.
pub struct EntityImageSimpleEmpty {
    pub back_color: Option<String>,
}

impl EntityImageSimpleEmpty {
    pub fn new(back_color: Option<String>) -> Self {
        Self { back_color }
    }
}

impl IEntityImage for EntityImageSimpleEmpty {
    fn shape_type(&self) -> super::shape_type::ShapeType {
        super::shape_type::ShapeType::Rectangle
    }

    fn dimension(&self) -> XDimension2D {
        XDimension2D::new(10.0, 10.0)
    }
}

// ── HeaderLayout ────────────────────────────────────────────────────

/// Layout manager for entity headers (circled character + stereotype + name + generic).
/// Java: `svek.HeaderLayout`
///
/// Arranges up to four blocks horizontally:
///   [circledCharacter] [stereotype/name] [generic]
/// with appropriate spacing.
#[derive(Debug, Clone)]
pub struct HeaderLayout {
    pub name_dim: XDimension2D,
    pub stereo_dim: XDimension2D,
    pub generic_dim: XDimension2D,
    pub circle_dim: XDimension2D,
}

impl HeaderLayout {
    pub fn new(
        circle_dim: XDimension2D,
        stereo_dim: XDimension2D,
        name_dim: XDimension2D,
        generic_dim: XDimension2D,
    ) -> Self {
        Self {
            name_dim,
            stereo_dim,
            generic_dim,
            circle_dim,
        }
    }

    /// Calculate the combined dimension of the header.
    /// Java: `HeaderLayout.getDimension()`
    pub fn dimension(&self) -> XDimension2D {
        let width = self.circle_dim.width
            + self.stereo_dim.width.max(self.name_dim.width)
            + self.generic_dim.width;
        let height = self
            .circle_dim
            .height
            .max(self.stereo_dim.height + self.name_dim.height + 10.0)
            .max(self.generic_dim.height);
        XDimension2D::new(width, height)
    }

    /// Calculate positioned offsets for drawing each header block.
    /// Returns `HeaderOffsets` with (x, y) pairs for each sub-block.
    /// Java: `HeaderLayout.drawU()`
    pub fn layout_offsets(&self, total_width: f64, total_height: f64) -> HeaderOffsets {
        let width_stereo_and_name = self.stereo_dim.width.max(self.name_dim.width);
        let supp_width =
            (total_width - self.circle_dim.width - width_stereo_and_name - self.generic_dim.width)
                .max(0.0);

        let h2 = (self.circle_dim.width / 4.0).min(supp_width * 0.1);
        let h1 = ((supp_width - h2) / 2.0).max(0.0);

        let x_circle = h1;
        let y_circle = (total_height - self.circle_dim.height) / 2.0;

        let diff_height = total_height - self.stereo_dim.height - self.name_dim.height;
        let x_stereo =
            self.circle_dim.width + (width_stereo_and_name - self.stereo_dim.width) / 2.0 + h1 + h2;
        let y_stereo = diff_height / 2.0;

        let x_name =
            self.circle_dim.width + (width_stereo_and_name - self.name_dim.width) / 2.0 + h1 + h2;
        let y_name = diff_height / 2.0 + self.stereo_dim.height;

        let (x_generic, y_generic) = if self.generic_dim.width > 0.0 {
            let delta = 4.0;
            (total_width - self.generic_dim.width + delta, -delta)
        } else {
            (0.0, 0.0)
        };

        HeaderOffsets {
            circle: (x_circle, y_circle),
            stereo: (x_stereo, y_stereo),
            name: (x_name, y_name),
            generic: (x_generic, y_generic),
        }
    }
}

/// Calculated offsets for header sub-blocks.
#[derive(Debug, Clone)]
pub struct HeaderOffsets {
    pub circle: (f64, f64),
    pub stereo: (f64, f64),
    pub name: (f64, f64),
    pub generic: (f64, f64),
}

// ── DirectionalTextBlock ────────────────────────────────────────────

/// Selects one of four text blocks based on edge direction.
/// Java: `svek.DirectionalTextBlock`
///
/// Used for edge labels that need to rotate/mirror based on edge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    Right,
    Left,
    Up,
    Down,
}

/// Holds dimensions for each direction variant.
/// Java: `svek.DirectionalTextBlock`
#[derive(Debug, Clone)]
pub struct DirectionalDimensions {
    pub right: XDimension2D,
    pub left: XDimension2D,
    pub up: XDimension2D,
    pub down: XDimension2D,
}

impl DirectionalDimensions {
    pub fn new(
        right: XDimension2D,
        left: XDimension2D,
        up: XDimension2D,
        down: XDimension2D,
    ) -> Self {
        Self {
            right,
            left,
            up,
            down,
        }
    }

    /// Uniform dimensions for all directions.
    pub fn uniform(dim: XDimension2D) -> Self {
        Self {
            right: dim,
            left: dim,
            up: dim,
            down: dim,
        }
    }

    /// Get dimension for a specific direction.
    pub fn for_direction(&self, dir: ArrowDirection) -> XDimension2D {
        match dir {
            ArrowDirection::Right => self.right,
            ArrowDirection::Left => self.left,
            ArrowDirection::Up => self.up,
            ArrowDirection::Down => self.down,
        }
    }

    /// The canonical dimension (right) is used for overall size calculation.
    /// Java: `DirectionalTextBlock.calculateDimension()`
    pub fn canonical_dimension(&self) -> XDimension2D {
        self.right
    }
}

// ── InnerActivity ───────────────────────────────────────────────────

/// Wrapper that draws an entity image inside a rounded rectangle border.
/// Java: `svek.InnerActivity`
///
/// Used for activity diagram nodes.
pub struct InnerActivity {
    /// Inner entity image dimensions
    pub inner_dim: XDimension2D,
    /// Border color (CSS hex)
    pub border_color: String,
    /// Background color (CSS hex)
    pub back_color: String,
    /// Shadow offset
    pub shadowing: f64,
    /// Whether the inner is hidden
    pub hidden: bool,
}

impl InnerActivity {
    /// Border thickness constant.
    /// Java: `InnerActivity.THICKNESS_BORDER`
    pub const THICKNESS_BORDER: f64 = 1.5;

    pub fn new(
        inner_dim: XDimension2D,
        border_color: &str,
        back_color: &str,
        shadowing: f64,
    ) -> Self {
        Self {
            inner_dim,
            border_color: border_color.to_string(),
            back_color: back_color.to_string(),
            shadowing,
            hidden: false,
        }
    }
}

impl IEntityImage for InnerActivity {
    fn shape_type(&self) -> super::shape_type::ShapeType {
        super::shape_type::ShapeType::RoundRectangle
    }

    fn dimension(&self) -> XDimension2D {
        self.inner_dim
    }

    fn is_hidden(&self) -> bool {
        self.hidden
    }
}

// ── InnerStateAutonom ───────────────────────────────────────────────

/// Autonomous state container with title, attributes, and inner content.
/// Java: `svek.InnerStateAutonom`
///
/// Renders:
///   +--------------------+
///   |  Title             |
///   +--------------------+
///   |  Attributes        |
///   +--------------------+
///   |  Inner content     |
///   +--------------------+
pub struct InnerStateAutonom {
    /// Dimensions of the inner content
    pub inner_dim: XDimension2D,
    /// Dimensions of the title block
    pub title_dim: XDimension2D,
    /// Dimensions of the attribute block
    pub attribute_dim: XDimension2D,
    /// Border color (CSS hex)
    pub border_color: String,
    /// Background color (CSS hex)
    pub back_color: String,
    /// Body background color (CSS hex)
    pub body_color: String,
    /// Rounded corner radius
    pub rounded: f64,
    /// Shadow offset
    pub shadowing: f64,
    /// Whether to draw the OO symbol
    pub with_symbol: bool,
    /// Whether the inner is hidden
    pub hidden: bool,
}

impl InnerStateAutonom {
    pub fn new(
        inner_dim: XDimension2D,
        title_dim: XDimension2D,
        attribute_dim: XDimension2D,
        border_color: &str,
        back_color: &str,
        body_color: &str,
        rounded: f64,
        shadowing: f64,
    ) -> Self {
        Self {
            inner_dim,
            title_dim,
            attribute_dim,
            border_color: border_color.to_string(),
            back_color: back_color.to_string(),
            body_color: body_color.to_string(),
            rounded,
            shadowing,
            with_symbol: false,
            hidden: false,
        }
    }

    /// Calculate the overall dimension including title, attributes, margins.
    /// Java: `InnerStateAutonom.calculateDimensionSlow()`
    pub fn calculate_full_dimension(&self) -> XDimension2D {
        let margin = <Self as IEntityImage>::MARGIN;
        let margin_line = <Self as IEntityImage>::MARGIN_LINE;

        let merged_width = self
            .title_dim
            .width
            .max(self.attribute_dim.width)
            .max(self.inner_dim.width);
        let merged_height =
            self.title_dim.height + self.attribute_dim.height + self.inner_dim.height;

        let margin_for_fields = if self.attribute_dim.height > 0.0 {
            margin
        } else {
            0.0
        };

        XDimension2D::new(
            merged_width + margin * 2.0,
            merged_height + margin * 2.0 + 2.0 * margin_line + margin_for_fields,
        )
    }

    /// Y offset for the inner content.
    /// Java: `InnerStateAutonom.getSpaceYforURL()`
    pub fn inner_y_offset(&self) -> f64 {
        let margin = <Self as IEntityImage>::MARGIN;
        let margin_line = <Self as IEntityImage>::MARGIN_LINE;
        let margin_for_fields = if self.attribute_dim.height > 0.0 {
            margin
        } else {
            0.0
        };
        let title_height = margin + self.title_dim.height + margin_line;
        title_height + margin_for_fields + self.attribute_dim.height + margin_line
    }
}

impl IEntityImage for InnerStateAutonom {
    fn shape_type(&self) -> super::shape_type::ShapeType {
        super::shape_type::ShapeType::RoundRectangle
    }

    fn dimension(&self) -> XDimension2D {
        self.calculate_full_dimension()
    }

    fn is_hidden(&self) -> bool {
        self.hidden
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::klimt::geom::XDimension2D;

    #[test]
    fn constants() {
        struct Dummy;
        impl IEntityImage for Dummy {
            fn shape_type(&self) -> super::super::shape_type::ShapeType {
                super::super::shape_type::ShapeType::Rectangle
            }
            fn dimension(&self) -> XDimension2D {
                XDimension2D::new(0.0, 0.0)
            }
        }
        assert_eq!(Dummy::CORNER, 25.0);
        assert_eq!(Dummy::MARGIN, 5.0);
        assert_eq!(Dummy::MARGIN_LINE, 5.0);
    }

    #[test]
    fn abstract_entity_image() {
        let img = AbstractEntityImage::new("Foo")
            .with_back_color("#ffffff")
            .with_hidden(false);
        assert_eq!(img.entity_id, "Foo");
        assert_eq!(img.back_color.as_deref(), Some("#ffffff"));
        assert!(!img.hidden);
    }

    #[test]
    fn entity_image_simple_empty() {
        let img = EntityImageSimpleEmpty::new(None);
        let dim = img.dimension();
        assert_eq!(dim.width, 10.0);
        assert_eq!(dim.height, 10.0);
        assert_eq!(
            img.shape_type(),
            super::super::shape_type::ShapeType::Rectangle
        );
    }

    #[test]
    fn entity_image_degenerated() {
        let img = EntityImageDegenerated::new(
            XDimension2D::new(100.0, 50.0),
            super::super::shape_type::ShapeType::RoundRectangle,
            Some("#ffffff".to_string()),
        );
        assert_eq!(img.dimension().width, 100.0);
        assert_eq!(
            img.shape_type(),
            super::super::shape_type::ShapeType::RoundRectangle
        );
        assert!(!img.is_hidden());
    }

    #[test]
    fn header_layout_dimension() {
        let hl = HeaderLayout::new(
            XDimension2D::new(20.0, 20.0), // circle
            XDimension2D::new(50.0, 12.0), // stereo
            XDimension2D::new(80.0, 14.0), // name
            XDimension2D::new(0.0, 0.0),   // generic
        );
        let dim = hl.dimension();
        // width = 20 + max(50,80) + 0 = 100
        assert_eq!(dim.width, 100.0);
        // height = max(20, 12+14+10, 0) = 36
        assert_eq!(dim.height, 36.0);
    }

    #[test]
    fn header_layout_with_generic() {
        let hl = HeaderLayout::new(
            XDimension2D::new(0.0, 0.0),   // no circle
            XDimension2D::new(0.0, 0.0),   // no stereo
            XDimension2D::new(60.0, 14.0), // name
            XDimension2D::new(30.0, 12.0), // generic
        );
        let dim = hl.dimension();
        // width = 0 + 60 + 30 = 90
        assert_eq!(dim.width, 90.0);
        // height = max(0, 0+14+10, 12) = 24
        assert_eq!(dim.height, 24.0);
    }

    #[test]
    fn header_layout_offsets() {
        let hl = HeaderLayout::new(
            XDimension2D::new(20.0, 20.0),
            XDimension2D::new(50.0, 12.0),
            XDimension2D::new(80.0, 14.0),
            XDimension2D::new(0.0, 0.0),
        );
        let dim = hl.dimension();
        let offsets = hl.layout_offsets(dim.width, dim.height);
        // Circle centered vertically
        assert!((offsets.circle.1 - (dim.height - 20.0) / 2.0).abs() < 1e-10);
    }

    #[test]
    fn directional_dimensions() {
        let dd = DirectionalDimensions::uniform(XDimension2D::new(100.0, 20.0));
        assert_eq!(dd.for_direction(ArrowDirection::Right).width, 100.0);
        assert_eq!(dd.for_direction(ArrowDirection::Left).width, 100.0);
        assert_eq!(dd.canonical_dimension().width, 100.0);
    }

    #[test]
    fn directional_dimensions_varied() {
        let dd = DirectionalDimensions::new(
            XDimension2D::new(100.0, 20.0),
            XDimension2D::new(90.0, 18.0),
            XDimension2D::new(20.0, 100.0),
            XDimension2D::new(22.0, 95.0),
        );
        assert_eq!(dd.for_direction(ArrowDirection::Up).width, 20.0);
        assert_eq!(dd.for_direction(ArrowDirection::Down).height, 95.0);
    }

    #[test]
    fn inner_activity() {
        let ia = InnerActivity::new(XDimension2D::new(100.0, 50.0), "#000000", "#ffffff", 3.0);
        assert_eq!(ia.dimension().width, 100.0);
        assert_eq!(ia.dimension().height, 50.0);
        assert_eq!(
            ia.shape_type(),
            super::super::shape_type::ShapeType::RoundRectangle
        );
        assert_eq!(InnerActivity::THICKNESS_BORDER, 1.5);
    }

    #[test]
    fn inner_state_autonom_dimension() {
        let isa = InnerStateAutonom::new(
            XDimension2D::new(100.0, 80.0), // inner
            XDimension2D::new(60.0, 14.0),  // title
            XDimension2D::new(40.0, 10.0),  // attribute
            "#000000",
            "#ffffff",
            "#eeeeee",
            15.0,
            3.0,
        );
        let dim = isa.dimension();
        // width = max(100, 60, 40) + 5*2 = 110
        assert_eq!(dim.width, 110.0);
        // height = (14 + 10 + 80) + 5*2 + 5*2 + 5 = 129
        assert_eq!(dim.height, 129.0);
    }

    #[test]
    fn inner_state_autonom_no_attributes() {
        let isa = InnerStateAutonom::new(
            XDimension2D::new(100.0, 80.0), // inner
            XDimension2D::new(60.0, 14.0),  // title
            XDimension2D::new(0.0, 0.0),    // no attributes
            "#000000",
            "#ffffff",
            "#eeeeee",
            15.0,
            3.0,
        );
        let dim = isa.dimension();
        // width = max(100, 60, 0) + 10 = 110
        assert_eq!(dim.width, 110.0);
        // height = (14 + 0 + 80) + 10 + 10 + 0 = 114
        assert_eq!(dim.height, 114.0);
    }

    #[test]
    fn inner_state_autonom_inner_y_offset() {
        let isa = InnerStateAutonom::new(
            XDimension2D::new(100.0, 80.0),
            XDimension2D::new(60.0, 14.0),
            XDimension2D::new(40.0, 10.0),
            "#000000",
            "#ffffff",
            "#eeeeee",
            15.0,
            3.0,
        );
        let y = isa.inner_y_offset();
        // title_height = 5 + 14 + 5 = 24
        // + margin_for_fields = 5
        // + attribute_dim.height = 10
        // + margin_line = 5
        // total = 44
        assert_eq!(y, 44.0);
    }
}
