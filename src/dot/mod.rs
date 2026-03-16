// dot - Graphviz integration
// Port of Java PlantUML's net.sourceforge.plantuml.dot package
// Process management for calling the `dot` command.

pub mod graphviz;
pub mod dot_data;
pub mod dot_splines;
pub mod version;

pub use dot_splines::DotSplines;
pub use version::GraphvizVersion;
