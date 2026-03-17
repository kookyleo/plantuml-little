// style - CSS-like style system
// Port of Java PlantUML's net.sourceforge.plantuml.style package
//
// Provides property resolution for diagram elements via a cascade of
// style rules, matching Java's ISkinParam / Style / StyleSignature system.

pub mod pname;
pub mod sname;
pub mod value;
pub mod skin_param;
pub mod style_def;
pub mod signature;

pub use pname::PName;
pub use sname::SName;
pub use value::Value;
pub use skin_param::ISkinParam;

// Backward compatibility: re-export everything from the old style.rs
mod compat;
pub use compat::*;
