// layout::sequence_teoz - Teoz sequence diagram layout engine
//
// Port of Java PlantUML's sequencediagram.teoz package.
// Tile-based layout with Real constraint propagation.
//
// Activated by `!pragma teoz true` in the diagram source.
// Produces the same SeqLayout output as the Puma engine (layout::sequence).

pub mod builder;
pub mod living;
pub mod real;
pub mod tile;
pub mod tiles;

use crate::model::sequence::SequenceDiagram;
use crate::style::SkinParams;
use crate::Result;

// Re-use the same output types as the Puma engine
use crate::layout::sequence::SeqLayout;

/// Teoz layout engine entry point.
///
/// Converts a parsed SequenceDiagram into a SeqLayout using the tile-based
/// constraint propagation approach (matching Java's Teoz engine).
pub fn layout_sequence_teoz(sd: &SequenceDiagram, skin: &SkinParams) -> Result<SeqLayout> {
    builder::build_teoz_layout(sd, skin)
}
