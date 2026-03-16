// decoration - Link decoration types and UML symbol shapes
// Port of Java PlantUML's net.sourceforge.plantuml.decoration package

pub mod link_decor;
pub mod link_type;
pub mod link_style;
pub mod symbol;

pub use link_decor::{LinkDecor, LinkMiddleDecor, ExtremityKind};
pub use link_type::{LinkType, LinkStrategy};
pub use link_style::{LinkStyle, LinkStyleKind};
