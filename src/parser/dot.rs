/// DOT passthrough parser -- extracts raw DOT source for direct Graphviz rendering.
use crate::Result;

/// Extract the DOT source from a @startdot block.
/// Returns the raw DOT lines between @startdot and @enddot.
pub fn parse_dot_source(block: &str) -> Result<String> {
    Ok(block.to_string())
}
