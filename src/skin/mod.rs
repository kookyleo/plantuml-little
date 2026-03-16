// skin - Diagram component rendering
// Port of Java PlantUML's net.sourceforge.plantuml.skin package
//
// Defines the visual components used to render diagrams:
// - Arrow configurations (head, body, direction)
// - Actor styles (stickman, awesome, hollow)
// - Component types (participant, note, divider, etc.)
// - Rose theme (the default PlantUML skin)

pub mod arrow;
pub mod component;
pub mod actor;
pub mod rose;

// Re-exports
pub use arrow::{ArrowConfiguration, ArrowDirection, ArrowHead, ArrowBody, ArrowPart, ArrowDressing};
pub use component::{ComponentType, ComponentStyle};
pub use actor::ActorStyle;
