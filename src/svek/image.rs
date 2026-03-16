// svek::image - Entity image rendering abstractions
// Port of Java PlantUML's svek.IEntityImage, EntityImage*, HeaderLayout

use crate::klimt::geom::XDimension2D;

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
    fn shield(&self) -> super::Margins { super::Margins::none() }

    /// Horizontal overscan (extra width for edge attachment).
    fn overscan_x(&self) -> f64 { 0.0 }
}

// TODO: Full port of AbstractEntityImage, EntityImageDegenerated, HeaderLayout

#[cfg(test)]
mod tests {
    #[test]
    fn constants() {
        use super::IEntityImage;
        // Verify constants match Java
        struct Dummy;
        impl IEntityImage for Dummy {
            fn shape_type(&self) -> super::super::shape_type::ShapeType {
                super::super::shape_type::ShapeType::Rectangle
            }
            fn dimension(&self) -> super::super::super::klimt::geom::XDimension2D {
                super::super::super::klimt::geom::XDimension2D::new(0.0, 0.0)
            }
        }
        assert_eq!(Dummy::CORNER, 25.0);
        assert_eq!(Dummy::MARGIN, 5.0);
    }
}
