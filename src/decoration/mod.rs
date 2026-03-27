// decoration - Link decoration types and UML symbol shapes
// Port of Java PlantUML's net.sourceforge.plantuml.decoration package

pub mod link_decor;
pub mod link_style;
pub mod link_type;
pub mod symbol;

pub use link_decor::{ExtremityKind, LinkDecor, LinkMiddleDecor};
pub use link_style::{LinkStyle, LinkStyleKind};
pub use link_type::{LinkStrategy, LinkType};
