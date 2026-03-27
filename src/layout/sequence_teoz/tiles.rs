// layout::sequence_teoz::tiles - Tile enum and implementations
//
// Port of Java PlantUML's sequencediagram.teoz tile classes:
//   CommunicationTile, CommunicationTileSelf, LifeEventTile,
//   NoteTile, GroupingTile, DelayTile, DividerTile, HSpaceTile.
//
// Uses an enum-based approach instead of Java's class hierarchy.
// Each variant stores the data needed for layout computation (heights,
// constraints). Rendering is handled separately via SeqLayout.

use super::real::{RealId, RealLine};
use super::tile::{Tile, TileState, TimeHook};
use crate::klimt::geom::XDimension2D;
use crate::skin::rose;

/// Activation box width adjustment for arrow endpoints.
/// Java: `CommunicationTile.LIVE_DELTA_SIZE`
pub const LIVE_DELTA_SIZE: f64 = 5.0;

/// Position of a note relative to a participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotePosition {
    Left,
    Right,
    Over,
    OverSeveral,
}

/// All tile types as an enum.
///
/// Each variant holds the layout-relevant data for its tile type.
/// The Tile trait is implemented via match dispatch.
#[derive(Debug)]
pub enum TileKind {
    /// Normal message between two different participants.
    /// Java: `CommunicationTile`
    Communication {
        /// Index of the sending participant
        participant1: usize,
        /// Index of the receiving participant
        participant2: usize,
        /// Real variable for participant 1 center
        pos_c1: RealId,
        /// Real variable for participant 2 center
        pos_c2: RealId,
        /// Preferred height from the arrow component
        preferred_height: f64,
        /// Preferred width of the arrow (needed for constraints)
        preferred_width: f64,
        /// Whether this message creates participant2
        is_create: bool,
        /// Activation level at participant 1 (for LIVE_DELTA_SIZE adjustment)
        level1: i32,
        /// Activation level at participant 2
        level2: i32,
        /// Whether participant2 is left of participant1
        is_reverse: bool,
        /// Whether this is a parallel message
        is_parallel: bool,
        /// Common tile state
        state: TileState,
    },

    /// Self-message (participant sends to itself).
    /// Java: `CommunicationTileSelf`
    CommunicationSelf {
        /// Index of the participant
        participant: usize,
        /// Real variable for participant center
        pos_c: RealId,
        /// Real variable for the right side of participant center (pos_c2)
        pos_c2: RealId,
        /// Preferred height from the self-arrow component
        preferred_height: f64,
        /// Preferred width of the self-arrow
        preferred_width: f64,
        /// Whether arrow has reverse-define direction
        is_reverse_define: bool,
        /// Index of the next participant (for constraint), or None
        next_participant_pos_c: Option<RealId>,
        /// Index of the previous participant (for constraint), or None
        prev_participant_pos_c2: Option<RealId>,
        /// Activation level (for LIVE_DELTA_SIZE adjustment)
        level: i32,
        /// Common tile state
        state: TileState,
    },

    /// Activation/deactivation/destroy event.
    /// Java: `LifeEventTile`
    LifeEvent {
        /// Index of the participant
        participant: usize,
        /// Real variable for participant center
        pos_c: RealId,
        /// Whether this is a destroy without an associated message
        is_destroy_without_message: bool,
        /// Preferred height (0 for normal events, cross-size for standalone destroy)
        preferred_height: f64,
        /// Activation level at this participant
        level: i32,
        /// Common tile state
        state: TileState,
    },

    /// Note attached to a participant.
    /// Java: `NoteTile`
    Note {
        /// Real variable for participant 1 center
        pos_c1: RealId,
        /// Real variable for participant 1 left edge
        pos_b1: RealId,
        /// Real variable for participant 1 right edge
        pos_d1: RealId,
        /// Real variable for participant 2 center (used for OverSeveral)
        pos_c2: Option<RealId>,
        /// Real variable for participant 2 left edge (used for OverSeveral)
        pos_b2: Option<RealId>,
        /// Real variable for participant 2 right edge (used for OverSeveral)
        pos_d2: Option<RealId>,
        /// Note position relative to participant(s)
        position: NotePosition,
        /// Preferred dimensions of the note component
        preferred_dim: XDimension2D,
        /// Activation level at participant 1 (for right-side offset)
        level: i32,
        /// Common tile state
        state: TileState,
    },

    /// Combined fragment (alt, loop, opt, etc.) containing children.
    /// Java: `GroupingTile`
    Grouping {
        /// Child tiles
        children: Vec<TileKind>,
        /// Preferred height of the header component
        header_height: f64,
        /// Accumulated body height (sum of children)
        body_height: f64,
        /// Minimum X as a Real variable (computed from children)
        min_x: RealId,
        /// Maximum X as a Real variable (computed from children)
        max_x: RealId,
        /// Common tile state
        state: TileState,
    },

    /// Delay (...). Java: `DelayTile`
    Delay {
        /// Preferred height from the delay text component
        preferred_height: f64,
        /// Real variable for the midpoint between first and last participants
        middle: RealId,
        /// Half-width of the delay text
        half_width: f64,
        /// Common tile state
        state: TileState,
    },

    /// Divider (== text ==). Java: `DividerTile`
    Divider {
        /// Preferred dimensions from the divider component
        preferred_dim: XDimension2D,
        /// Real variable for the X origin
        x_origin: RealId,
        /// Common tile state
        state: TileState,
    },

    /// Explicit vertical spacing. Java: `HSpaceTile`
    HSpace {
        /// Number of pixels of spacing
        pixels: f64,
        /// Real variable for the X origin
        x_origin: RealId,
        /// Common tile state
        state: TileState,
    },

    /// Parallel tile container. Java: `TileParallel`
    Parallel {
        /// Child tiles to lay out in parallel
        children: Vec<TileKind>,
        /// Common tile state
        state: TileState,
    },

    /// Empty placeholder tile. Java: `EmptyTile`
    Empty {
        /// Real variable for the X origin
        x_origin: RealId,
        /// Common tile state
        state: TileState,
    },
}

// ── Grouping constants ──────────────────────────────────────────────

/// External margin X1 for grouping. Java: `GroupingTile.EXTERNAL_MARGINX1`
pub const GROUPING_EXTERNAL_MARGINX1: f64 = 3.0;
/// External margin X2 for grouping. Java: `GroupingTile.EXTERNAL_MARGINX2`
pub const GROUPING_EXTERNAL_MARGINX2: f64 = 9.0;
/// Internal margin X for grouping. Java: `GroupingTile.MARGINX`
#[allow(dead_code)]
const GROUPING_MARGINX: f64 = 16.0;
/// Magic margin Y for grouping. Java: `GroupingTile.MARGINY_MAGIC`
const GROUPING_MARGINY_MAGIC: f64 = 20.0;

// ══════════════════════════════════════════════════════════════════════
// Construction helpers
// ══════════════════════════════════════════════════════════════════════

impl TileKind {
    /// Create a Communication tile.
    ///
    /// `text_metrics` is the measured text for the arrow label.
    /// `inclination` values default to 0.0 for simple arrows.
    pub fn communication(
        participant1: usize,
        participant2: usize,
        pos_c1: RealId,
        pos_c2: RealId,
        text: &rose::TextMetrics,
        inclination1: f64,
        inclination2: f64,
        is_create: bool,
        level1: i32,
        level2: i32,
        is_reverse: bool,
        is_parallel: bool,
    ) -> Self {
        let dim = rose::arrow_preferred_size(text, inclination1, inclination2);
        TileKind::Communication {
            participant1,
            participant2,
            pos_c1,
            pos_c2,
            preferred_height: dim.height,
            preferred_width: dim.width,
            is_create,
            level1,
            level2,
            is_reverse,
            is_parallel,
            state: TileState::new(),
        }
    }

    /// Create a CommunicationSelf tile.
    ///
    /// `text_metrics` is the measured text for the self-arrow label.
    pub fn communication_self(
        participant: usize,
        pos_c: RealId,
        pos_c2: RealId,
        text: &rose::TextMetrics,
        is_reverse_define: bool,
        next_participant_pos_c: Option<RealId>,
        prev_participant_pos_c2: Option<RealId>,
        level: i32,
    ) -> Self {
        let dim = rose::self_arrow_preferred_size(text);
        TileKind::CommunicationSelf {
            participant,
            pos_c,
            pos_c2,
            preferred_height: dim.height,
            preferred_width: dim.width,
            is_reverse_define,
            next_participant_pos_c,
            prev_participant_pos_c2,
            level,
            state: TileState::new(),
        }
    }

    /// Create a LifeEvent tile.
    ///
    /// For standalone destroy events (no associated message), pass
    /// `is_destroy_without_message = true` and the destroy cross
    /// preferred height will be used.
    pub fn life_event(
        participant: usize,
        pos_c: RealId,
        is_destroy_without_message: bool,
        level: i32,
    ) -> Self {
        let preferred_height = if is_destroy_without_message {
            rose::destroy_preferred_size().height
        } else {
            0.0
        };
        TileKind::LifeEvent {
            participant,
            pos_c,
            is_destroy_without_message,
            preferred_height,
            level,
            state: TileState::new(),
        }
    }

    /// Create a Note tile.
    pub fn note(
        pos_c1: RealId,
        pos_b1: RealId,
        pos_d1: RealId,
        pos_c2: Option<RealId>,
        pos_b2: Option<RealId>,
        pos_d2: Option<RealId>,
        position: NotePosition,
        preferred_dim: XDimension2D,
        level: i32,
    ) -> Self {
        TileKind::Note {
            pos_c1,
            pos_b1,
            pos_d1,
            pos_c2,
            pos_b2,
            pos_d2,
            position,
            preferred_dim,
            level,
            state: TileState::new(),
        }
    }

    /// Create a Grouping tile with pre-computed children.
    pub fn grouping(
        children: Vec<TileKind>,
        header_height: f64,
        min_x: RealId,
        max_x: RealId,
    ) -> Self {
        let body_height: f64 = children.iter().map(|c| c.preferred_height()).sum();
        TileKind::Grouping {
            children,
            header_height,
            body_height,
            min_x,
            max_x,
            state: TileState::new(),
        }
    }

    /// Create a Delay tile.
    pub fn delay(preferred_height: f64, middle: RealId, half_width: f64) -> Self {
        TileKind::Delay {
            preferred_height,
            middle,
            half_width,
            state: TileState::new(),
        }
    }

    /// Create a Divider tile.
    pub fn divider(preferred_dim: XDimension2D, x_origin: RealId) -> Self {
        TileKind::Divider {
            preferred_dim,
            x_origin,
            state: TileState::new(),
        }
    }

    /// Create an HSpace tile.
    pub fn hspace(pixels: f64, x_origin: RealId) -> Self {
        TileKind::HSpace {
            pixels,
            x_origin,
            state: TileState::new(),
        }
    }

    /// Create a Parallel tile container.
    pub fn parallel(children: Vec<TileKind>) -> Self {
        TileKind::Parallel {
            children,
            state: TileState::new(),
        }
    }

    /// Create an Empty placeholder tile.
    pub fn empty(x_origin: RealId) -> Self {
        TileKind::Empty {
            x_origin,
            state: TileState::new(),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// Tile trait implementation
// ══════════════════════════════════════════════════════════════════════

impl Tile for TileKind {
    fn preferred_height(&self) -> f64 {
        match self {
            TileKind::Communication {
                preferred_height, ..
            } => *preferred_height,
            TileKind::CommunicationSelf {
                preferred_height, ..
            } => *preferred_height,
            TileKind::LifeEvent {
                preferred_height, ..
            } => *preferred_height,
            TileKind::Note { preferred_dim, .. } => preferred_dim.height,
            TileKind::Grouping {
                header_height,
                body_height,
                ..
            } => *header_height + *body_height + GROUPING_MARGINY_MAGIC,
            TileKind::Delay {
                preferred_height, ..
            } => *preferred_height,
            TileKind::Divider { preferred_dim, .. } => preferred_dim.height,
            TileKind::HSpace { pixels, .. } => *pixels,
            TileKind::Parallel { children, .. } => children
                .iter()
                .map(|c| c.preferred_height())
                .fold(0.0_f64, f64::max),
            TileKind::Empty { .. } => 0.0,
        }
    }

    fn callback_y(&mut self, y: TimeHook) {
        match self {
            TileKind::Communication { state, .. } => state.callback_y(y),
            TileKind::CommunicationSelf { state, .. } => state.callback_y(y),
            TileKind::LifeEvent { state, .. } => state.callback_y(y),
            TileKind::Note { state, .. } => state.callback_y(y),
            TileKind::Grouping { state, .. } => state.callback_y(y),
            TileKind::Delay { state, .. } => state.callback_y(y),
            TileKind::Divider { state, .. } => state.callback_y(y),
            TileKind::HSpace { state, .. } => state.callback_y(y),
            TileKind::Parallel { state, children } => {
                state.callback_y(y);
                for child in children.iter_mut() {
                    child.callback_y(y);
                }
            }
            TileKind::Empty { state, .. } => state.callback_y(y),
        }
    }

    fn get_y(&self) -> Option<f64> {
        match self {
            TileKind::Communication { state, .. } => state.y,
            TileKind::CommunicationSelf { state, .. } => state.y,
            TileKind::LifeEvent { state, .. } => state.y,
            TileKind::Note { state, .. } => state.y,
            TileKind::Grouping { state, .. } => state.y,
            TileKind::Delay { state, .. } => state.y,
            TileKind::Divider { state, .. } => state.y,
            TileKind::HSpace { state, .. } => state.y,
            TileKind::Parallel { state, .. } => state.y,
            TileKind::Empty { state, .. } => state.y,
        }
    }

    fn add_constraints(&self) {
        // Constraints are added via add_constraints_to() which takes &mut RealLine.
        // This no-op satisfies the trait; use add_constraints_to() in practice.
    }

    fn min_x(&self) -> RealId {
        match self {
            TileKind::Communication {
                pos_c1,
                pos_c2,
                is_reverse,
                ..
            } => {
                if *is_reverse {
                    *pos_c2
                } else {
                    *pos_c1
                }
            }
            TileKind::CommunicationSelf { pos_c, .. } => *pos_c,
            TileKind::LifeEvent { pos_c, .. } => *pos_c,
            TileKind::Note { pos_c1, .. } => *pos_c1,
            TileKind::Grouping { min_x, .. } => *min_x,
            TileKind::Delay { middle, .. } => *middle,
            TileKind::Divider { x_origin, .. } => *x_origin,
            TileKind::HSpace { x_origin, .. } => *x_origin,
            TileKind::Parallel { children, .. } => {
                // Return first child's min_x (simplified)
                children.first().map(|c| c.min_x()).unwrap_or(RealId(0))
            }
            TileKind::Empty { x_origin, .. } => *x_origin,
        }
    }

    fn max_x(&self) -> RealId {
        match self {
            TileKind::Communication {
                pos_c1,
                pos_c2,
                is_reverse,
                ..
            } => {
                if *is_reverse {
                    *pos_c1
                } else {
                    *pos_c2
                }
            }
            TileKind::CommunicationSelf { pos_c2, .. } => *pos_c2,
            TileKind::LifeEvent { pos_c, .. } => *pos_c,
            TileKind::Note { pos_c1, .. } => *pos_c1,
            TileKind::Grouping { max_x, .. } => *max_x,
            TileKind::Delay { middle, .. } => *middle,
            TileKind::Divider { x_origin, .. } => *x_origin,
            TileKind::HSpace { x_origin, .. } => *x_origin,
            TileKind::Parallel { children, .. } => {
                children.last().map(|c| c.max_x()).unwrap_or(RealId(0))
            }
            TileKind::Empty { x_origin, .. } => *x_origin,
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// Constraint addition (requires mutable RealLine)
// ══════════════════════════════════════════════════════════════════════

impl TileKind {
    /// Add X-axis constraints to the RealLine for this tile.
    ///
    /// This mirrors the Java `addConstraints()` method, but takes
    /// `&mut RealLine` explicitly since Rust doesn't have the implicit
    /// context that Java's Real objects carry.
    pub fn add_constraints_to(&self, rl: &mut RealLine) {
        match self {
            TileKind::Communication {
                pos_c1,
                pos_c2,
                preferred_width,
                is_reverse,
                level1,
                level2,
                ..
            } => {
                add_communication_constraints(
                    rl,
                    *pos_c1,
                    *pos_c2,
                    *preferred_width,
                    *is_reverse,
                    *level1,
                    *level2,
                );
            }
            TileKind::CommunicationSelf {
                pos_c,
                pos_c2,
                preferred_width,
                is_reverse_define,
                next_participant_pos_c,
                prev_participant_pos_c2,
                ..
            } => {
                add_communication_self_constraints(
                    rl,
                    *pos_c,
                    *pos_c2,
                    *preferred_width,
                    *is_reverse_define,
                    *next_participant_pos_c,
                    *prev_participant_pos_c2,
                );
            }
            TileKind::LifeEvent { .. } => {
                // No constraints needed (Java: addConstraints() is empty)
            }
            TileKind::Note { .. } => {
                // No constraints added in Java's NoteTile.addConstraints()
            }
            TileKind::Grouping { children, .. } => {
                for child in children {
                    child.add_constraints_to(rl);
                }
            }
            TileKind::Delay { .. } => {
                // No constraints (Java: addConstraints() is empty)
            }
            TileKind::Divider { .. } => {
                // No constraints (Java: addConstraints() is empty)
            }
            TileKind::HSpace { .. } => {
                // No constraints (Java: addConstraints() is empty)
            }
            TileKind::Parallel { children, .. } => {
                for child in children {
                    child.add_constraints_to(rl);
                }
            }
            TileKind::Empty { .. } => {
                // No constraints
            }
        }
    }
}

/// Add constraints for a Communication tile.
///
/// Ensures that the arrow has enough width between the two participant
/// centers. Adjusts for activation box offsets (LIVE_DELTA_SIZE).
///
/// Java: `CommunicationTile.addConstraints()`
fn add_communication_constraints(
    rl: &mut RealLine,
    pos_c1: RealId,
    pos_c2: RealId,
    width: f64,
    is_reverse: bool,
    level1: i32,
    level2: i32,
) {
    if is_reverse {
        // Reversed: participant2 is left of participant1.
        // point1 may be adjusted left if activated.
        let adjustment1 = if level1 > 0 { -LIVE_DELTA_SIZE } else { 0.0 };
        let adjustment2 = level2 as f64 * LIVE_DELTA_SIZE;

        // point1 (adjusted) >= point2 (adjusted) + width
        // i.e., pos_c1 + adjustment1 >= pos_c2 + adjustment2 + width
        // i.e., pos_c1 >= pos_c2 + (width + adjustment2 - adjustment1)
        let min_distance = width + adjustment2 - adjustment1;
        rl.ensure_bigger_than_with_margin(pos_c1, pos_c2, min_distance);
    } else {
        // Normal: participant1 is left of participant2.
        let adjustment2 = if level2 > 0 { -LIVE_DELTA_SIZE } else { 0.0 };

        // point2 (adjusted) >= point1 + width
        // i.e., pos_c2 + adjustment2 >= pos_c1 + width
        // i.e., pos_c2 >= pos_c1 + (width - adjustment2)
        let min_distance = width - adjustment2;
        rl.ensure_bigger_than_with_margin(pos_c2, pos_c1, min_distance);
    }
}

/// Add constraints for a CommunicationSelf tile.
///
/// Ensures there is enough room for the self-arrow to the right (or left
/// for reverse-define) of the participant.
///
/// Java: `CommunicationTileSelf.addConstraints()`
fn add_communication_self_constraints(
    rl: &mut RealLine,
    pos_c: RealId,
    pos_c2: RealId,
    width: f64,
    is_reverse_define: bool,
    next_participant_pos_c: Option<RealId>,
    prev_participant_pos_c2: Option<RealId>,
) {
    if is_reverse_define {
        // Self-arrow extends to the left.
        // Constraint: pos_c >= prev.pos_c2 + width
        if let Some(prev_c2) = prev_participant_pos_c2 {
            let target = rl.add_fixed(prev_c2, width);
            rl.ensure_bigger_than(pos_c, target);
        }
    } else {
        // Self-arrow extends to the right.
        // max_x = pos_c2 + width
        // Constraint: next.pos_c >= max_x
        if let Some(next_c) = next_participant_pos_c {
            let max_x = rl.add_fixed(pos_c2, width);
            rl.ensure_bigger_than(next_c, max_x);
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// min_x / max_x with Real offsets (computed at constraint-solve time)
// ══════════════════════════════════════════════════════════════════════

impl TileKind {
    /// Compute min_x with activation-level adjustments, returning a
    /// derived RealId. Call after constraints are added to `rl`.
    pub fn min_x_adjusted(&self, rl: &mut RealLine) -> RealId {
        match self {
            TileKind::Communication {
                pos_c1,
                pos_c2,
                is_reverse,
                ..
            } => {
                if *is_reverse {
                    *pos_c2
                } else {
                    *pos_c1
                }
            }
            TileKind::CommunicationSelf {
                pos_c,
                preferred_width,
                is_reverse_define,
                level,
                ..
            } => {
                if *is_reverse_define {
                    let adjustment = if *level > 0 { LIVE_DELTA_SIZE } else { 0.0 };
                    rl.add_fixed(*pos_c, -preferred_width - adjustment)
                } else {
                    *pos_c
                }
            }
            TileKind::LifeEvent { pos_c, level, .. } => {
                let adjustment = if *level > 0 { LIVE_DELTA_SIZE } else { 0.0 };
                rl.add_fixed(*pos_c, -adjustment)
            }
            TileKind::Note {
                pos_c1,
                pos_b1,
                pos_b2,
                position,
                preferred_dim,
                level,
                ..
            } => {
                let width = preferred_dim.width;
                match position {
                    NotePosition::Left => rl.add_fixed(*pos_c1, -width),
                    NotePosition::Right => {
                        let dx = *level as f64 * LIVE_DELTA_SIZE;
                        rl.add_fixed(*pos_c1, dx)
                    }
                    NotePosition::Over => rl.add_fixed(*pos_c1, -width / 2.0),
                    NotePosition::OverSeveral => {
                        let x = rl.add_fixed(*pos_c1, -width / 2.0);
                        if pos_b2.is_some() {
                            // min of note x and participant 1 left edge
                            rl.min_of(vec![x, *pos_b1])
                        } else {
                            x
                        }
                    }
                }
            }
            TileKind::Grouping { min_x, .. } => rl.add_fixed(*min_x, -GROUPING_EXTERNAL_MARGINX1),
            TileKind::Delay {
                middle, half_width, ..
            } => rl.add_fixed(*middle, -*half_width),
            TileKind::Divider { x_origin, .. } => *x_origin,
            TileKind::HSpace { x_origin, .. } => *x_origin,
            TileKind::Parallel { children, .. } => {
                if let Some(first) = children.first() {
                    first.min_x_adjusted(rl)
                } else {
                    RealId(0)
                }
            }
            TileKind::Empty { x_origin, .. } => *x_origin,
        }
    }

    /// Compute max_x with activation-level adjustments, returning a
    /// derived RealId. Call after constraints are added to `rl`.
    pub fn max_x_adjusted(&self, rl: &mut RealLine) -> RealId {
        match self {
            TileKind::Communication {
                pos_c1,
                pos_c2,
                is_reverse,
                ..
            } => {
                if *is_reverse {
                    *pos_c1
                } else {
                    *pos_c2
                }
            }
            TileKind::CommunicationSelf {
                pos_c2,
                preferred_width,
                is_reverse_define,
                ..
            } => {
                if *is_reverse_define {
                    *pos_c2
                } else {
                    rl.add_fixed(*pos_c2, *preferred_width)
                }
            }
            TileKind::LifeEvent { pos_c, level, .. } => {
                let adjustment = if *level > 0 {
                    *level as f64 * LIVE_DELTA_SIZE
                } else {
                    0.0
                };
                rl.add_fixed(*pos_c, adjustment)
            }
            TileKind::Note {
                pos_c1,
                pos_d2,
                position,
                preferred_dim,
                level,
                ..
            } => {
                let width = preferred_dim.width;
                match position {
                    NotePosition::Left => *pos_c1,
                    NotePosition::Right => {
                        let dx = *level as f64 * LIVE_DELTA_SIZE;
                        rl.add_fixed(*pos_c1, dx + width)
                    }
                    NotePosition::Over => rl.add_fixed(*pos_c1, width / 2.0),
                    NotePosition::OverSeveral => {
                        let x = rl.add_fixed(*pos_c1, width / 2.0);
                        if let Some(d2) = pos_d2 {
                            rl.max_of(vec![x, *d2])
                        } else {
                            x
                        }
                    }
                }
            }
            TileKind::Grouping { max_x, .. } => rl.add_fixed(*max_x, GROUPING_EXTERNAL_MARGINX2),
            TileKind::Delay {
                middle, half_width, ..
            } => rl.add_fixed(*middle, *half_width),
            TileKind::Divider {
                x_origin,
                preferred_dim,
                ..
            } => rl.add_fixed(*x_origin, preferred_dim.width),
            TileKind::HSpace { x_origin, .. } => rl.add_fixed(*x_origin, 10.0),
            TileKind::Parallel { children, .. } => {
                if let Some(last) = children.last() {
                    last.max_x_adjusted(rl)
                } else {
                    RealId(0)
                }
            }
            TileKind::Empty { x_origin, .. } => rl.add_fixed(*x_origin, 10.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::rose::TextMetrics;

    /// Helper: create a simple TextMetrics for testing.
    fn test_text(width: f64, height: f64) -> TextMetrics {
        TextMetrics::new(5.0, 5.0, 3.0, width, height)
    }

    // ── CommunicationTile preferred_height ──────────────────────────

    #[test]
    fn communication_preferred_height_basic() {
        let text = test_text(60.0, 12.0);
        let mut rl = RealLine::new();
        let c1 = rl.create_base(0.0);
        let c2 = rl.create_base(100.0);

        let tile =
            TileKind::communication(0, 1, c1, c2, &text, 0.0, 0.0, false, 0, 0, false, false);

        // arrow_preferred_size: h = text_height() + ARROW_DELTA_Y + 2*ARROW_PADDING_Y
        // text_height() = 12.0 + 2*3.0 = 18.0
        // h = 18.0 + 4.0 + 2*4.0 = 30.0
        let expected = 18.0 + rose::ARROW_DELTA_Y + 2.0 * rose::ARROW_PADDING_Y;
        assert!(
            (tile.preferred_height() - expected).abs() < f64::EPSILON,
            "got {}, expected {}",
            tile.preferred_height(),
            expected,
        );
    }

    #[test]
    fn communication_preferred_height_with_inclination() {
        let text = test_text(60.0, 12.0);
        let mut rl = RealLine::new();
        let c1 = rl.create_base(0.0);
        let c2 = rl.create_base(100.0);

        let tile =
            TileKind::communication(0, 1, c1, c2, &text, 5.0, 3.0, false, 0, 0, false, false);

        // h = text_height() + ARROW_DELTA_Y + 2*ARROW_PADDING_Y + incl1 + incl2
        let expected = 18.0 + rose::ARROW_DELTA_Y + 2.0 * rose::ARROW_PADDING_Y + 5.0 + 3.0;
        assert!(
            (tile.preferred_height() - expected).abs() < f64::EPSILON,
            "got {}, expected {}",
            tile.preferred_height(),
            expected,
        );
    }

    // ── CommunicationTileSelf preferred_height ──────────────────────

    #[test]
    fn communication_self_preferred_height() {
        let text = test_text(40.0, 10.0);
        let mut rl = RealLine::new();
        let c = rl.create_base(50.0);
        let c2 = rl.create_base(60.0);

        let tile = TileKind::communication_self(0, c, c2, &text, false, None, None, 0);

        // self_arrow_preferred_size: h = text_height() + ARROW_DELTA_Y + SELF_ARROW_ONLY_HEIGHT + 2*ARROW_PADDING_Y
        // text_height() = 10.0 + 2*3.0 = 16.0
        // h = 16.0 + 4.0 + 13.0 + 8.0 = 41.0
        let text_h = 10.0 + 2.0 * 3.0;
        let expected = text_h
            + rose::ARROW_DELTA_Y
            + rose::SELF_ARROW_ONLY_HEIGHT
            + 2.0 * rose::ARROW_PADDING_Y;
        assert!(
            (tile.preferred_height() - expected).abs() < f64::EPSILON,
            "got {}, expected {}",
            tile.preferred_height(),
            expected,
        );
    }

    #[test]
    fn communication_self_preferred_height_larger_text() {
        let text = test_text(100.0, 20.0);
        let mut rl = RealLine::new();
        let c = rl.create_base(50.0);
        let c2 = rl.create_base(60.0);

        let tile = TileKind::communication_self(0, c, c2, &text, false, None, None, 0);

        // text_height() = 20.0 + 2*3.0 = 26.0
        let text_h = 20.0 + 2.0 * 3.0;
        let expected = text_h
            + rose::ARROW_DELTA_Y
            + rose::SELF_ARROW_ONLY_HEIGHT
            + 2.0 * rose::ARROW_PADDING_Y;
        assert!(
            (tile.preferred_height() - expected).abs() < f64::EPSILON,
            "got {}, expected {}",
            tile.preferred_height(),
            expected,
        );
    }

    // ── Communication constraints ───────────────────────────────────

    #[test]
    fn communication_constraint_normal_direction() {
        let text = test_text(60.0, 12.0);
        let mut rl = RealLine::new();
        let c1 = rl.create_base(0.0);
        let c2 = rl.create_base(50.0);

        let tile =
            TileKind::communication(0, 1, c1, c2, &text, 0.0, 0.0, false, 0, 0, false, false);
        tile.add_constraints_to(&mut rl);
        rl.compile();

        // pos_c2 >= pos_c1 + width
        let width = rose::arrow_preferred_size(&text, 0.0, 0.0).width;
        let gap = rl.get_value(c2) - rl.get_value(c1);
        assert!(
            gap >= width - f64::EPSILON,
            "gap {} should be >= width {}",
            gap,
            width,
        );
    }

    #[test]
    fn communication_constraint_reverse_direction() {
        let text = test_text(60.0, 12.0);
        let mut rl = RealLine::new();
        let c1 = rl.create_base(100.0);
        let c2 = rl.create_base(0.0);

        let tile = TileKind::communication(0, 1, c1, c2, &text, 0.0, 0.0, false, 0, 0, true, false);
        tile.add_constraints_to(&mut rl);
        rl.compile();

        // pos_c1 >= pos_c2 + width
        let width = rose::arrow_preferred_size(&text, 0.0, 0.0).width;
        let gap = rl.get_value(c1) - rl.get_value(c2);
        assert!(
            gap >= width - f64::EPSILON,
            "gap {} should be >= width {}",
            gap,
            width,
        );
    }

    // ── CommunicationSelf constraints ───────────────────────────────

    #[test]
    fn communication_self_constraint_pushes_next() {
        let text = test_text(40.0, 10.0);
        let mut rl = RealLine::new();
        let c = rl.create_base(50.0);
        let c2 = rl.create_base(60.0);
        let next_c = rl.create_base(70.0);

        let tile = TileKind::communication_self(0, c, c2, &text, false, Some(next_c), None, 0);
        tile.add_constraints_to(&mut rl);
        rl.compile();

        // next_c >= pos_c2 + width
        let width = rose::self_arrow_preferred_size(&text).width;
        let gap = rl.get_value(next_c) - rl.get_value(c2);
        assert!(
            gap >= width - f64::EPSILON,
            "gap {} should be >= width {}",
            gap,
            width,
        );
    }

    // ── LifeEvent ───────────────────────────────────────────────────

    #[test]
    fn life_event_normal_zero_height() {
        let mut rl = RealLine::new();
        let c = rl.create_base(50.0);

        let tile = TileKind::life_event(0, c, false, 0);
        assert!(
            tile.preferred_height().abs() < f64::EPSILON,
            "normal life event should have 0 height"
        );
    }

    #[test]
    fn life_event_destroy_has_height() {
        let mut rl = RealLine::new();
        let c = rl.create_base(50.0);

        let tile = TileKind::life_event(0, c, true, 0);
        let expected = rose::destroy_preferred_size().height;
        assert!(
            (tile.preferred_height() - expected).abs() < f64::EPSILON,
            "standalone destroy should have cross height"
        );
    }

    // ── HSpace ──────────────────────────────────────────────────────

    #[test]
    fn hspace_preferred_height() {
        let mut rl = RealLine::new();
        let x = rl.create_base(0.0);

        let tile = TileKind::hspace(42.0, x);
        assert!(
            (tile.preferred_height() - 42.0).abs() < f64::EPSILON,
            "HSpace should return the configured pixel height"
        );
    }

    // ── Divider ─────────────────────────────────────────────────────

    #[test]
    fn divider_preferred_height() {
        let text = test_text(30.0, 10.0);
        let dim = rose::divider_preferred_size(&text);
        let mut rl = RealLine::new();
        let x = rl.create_base(0.0);

        let tile = TileKind::divider(dim, x);
        assert!(
            (tile.preferred_height() - dim.height).abs() < f64::EPSILON,
            "divider should return component's preferred height"
        );
    }

    // ── Delay ───────────────────────────────────────────────────────

    #[test]
    fn delay_preferred_height() {
        let mut rl = RealLine::new();
        let mid = rl.create_base(50.0);

        let tile = TileKind::delay(35.0, mid, 20.0);
        assert!(
            (tile.preferred_height() - 35.0).abs() < f64::EPSILON,
            "delay should return configured height"
        );
    }

    // ── Parallel ────────────────────────────────────────────────────

    #[test]
    fn parallel_height_is_max_of_children() {
        let mut rl = RealLine::new();
        let x = rl.create_base(0.0);

        let children = vec![
            TileKind::hspace(10.0, x),
            TileKind::hspace(30.0, x),
            TileKind::hspace(20.0, x),
        ];
        let tile = TileKind::parallel(children);
        assert!(
            (tile.preferred_height() - 30.0).abs() < f64::EPSILON,
            "parallel height should be max of children"
        );
    }

    // ── Empty ───────────────────────────────────────────────────────

    #[test]
    fn empty_zero_height() {
        let mut rl = RealLine::new();
        let x = rl.create_base(0.0);

        let tile = TileKind::empty(x);
        assert!(
            tile.preferred_height().abs() < f64::EPSILON,
            "empty tile should have 0 height"
        );
    }

    // ── Callback Y ──────────────────────────────────────────────────

    #[test]
    fn callback_y_sets_position() {
        let mut rl = RealLine::new();
        let x = rl.create_base(0.0);

        let mut tile = TileKind::hspace(10.0, x);
        assert!(tile.get_y().is_none());

        tile.callback_y(TimeHook::new(55.0, super::super::tile::HookType::Start));
        assert!((tile.get_y().unwrap() - 55.0).abs() < f64::EPSILON,);
    }
}
