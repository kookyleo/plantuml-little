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
pub use value::{Value, ValueImpl, ValueColor, ValueNull, DarkString, LengthAdjust, MergeStrategy};
pub use skin_param::ISkinParam;
pub use style_def::{Style, StyleBuilder, StyleStorage, StyleLoader, ClockwiseTopRightBottomLeft};
pub use style_def::{DELTA_PRIORITY_FOR_STEREOTYPE, STYLE_ID_TITLE, STYLE_ID_CAPTION, STYLE_ID_LEGEND};
pub use signature::{StyleKey, StyleSignatureBasic, StyleSignature, StyleSignatures, Styleable};

// Backward compatibility: re-export everything from the old style.rs
mod compat;
pub use compat::*;
