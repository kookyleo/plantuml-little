// style - CSS-like style system
// Port of Java PlantUML's net.sourceforge.plantuml.style package
//
// Provides property resolution for diagram elements via a cascade of
// style rules, matching Java's ISkinParam / Style / StyleSignature system.

pub mod pname;
pub mod signature;
pub mod skin_param;
pub mod sname;
pub mod style_def;
pub mod value;

pub use pname::PName;
pub use signature::{StyleKey, StyleSignature, StyleSignatureBasic, StyleSignatures, Styleable};
pub use skin_param::ISkinParam;
pub use sname::SName;
pub use style_def::{ClockwiseTopRightBottomLeft, Style, StyleBuilder, StyleLoader, StyleStorage};
pub use style_def::{
    DELTA_PRIORITY_FOR_STEREOTYPE, STYLE_ID_CAPTION, STYLE_ID_LEGEND, STYLE_ID_TITLE,
};
pub use value::{DarkString, LengthAdjust, MergeStrategy, Value, ValueColor, ValueImpl, ValueNull};

// Backward compatibility: re-export everything from the old style.rs
mod compat;
pub use compat::*;
