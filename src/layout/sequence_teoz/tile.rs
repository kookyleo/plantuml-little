// layout::sequence_teoz::tile - Tile trait and base types
//
// Port of Java PlantUML's sequencediagram.teoz.Tile interface,
// CommonTile, and AbstractTile.
//
// A Tile represents one layout element (message, note, group, etc.)
// with a preferred height and X-axis constraint variables.

use super::real::RealId;

/// Hook type for Y-coordinate callbacks.
/// Java: `teoz.HookType`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    /// Normal start position
    Start,
    /// Continuation (e.g., else branch)
    Continue,
}

/// Y-coordinate assignment for a tile.
/// Java: `teoz.TimeHook`
#[derive(Debug, Clone, Copy)]
pub struct TimeHook {
    pub value: f64,
    pub hook_type: HookType,
}

impl TimeHook {
    pub fn new(value: f64, hook_type: HookType) -> Self {
        Self { value, hook_type }
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

/// The core Tile trait.
///
/// Each diagram element (message, note, activation, group, etc.)
/// implements this trait to participate in the Teoz layout.
///
/// Java: `teoz.Tile` interface
pub trait Tile {
    /// Preferred height of this tile.
    /// Java: `getPreferredHeight(StringBounder)`
    fn preferred_height(&self) -> f64;

    /// Called by the layout engine to assign a Y position.
    /// Java: `callbackY(TimeHook)`
    fn callback_y(&mut self, y: TimeHook);

    /// Get the assigned Y position (after callback_y).
    fn get_y(&self) -> Option<f64>;

    /// Add X-axis constraints to the RealLine.
    /// Java: `addConstraints()`
    fn add_constraints(&self);

    /// Minimum X extent (left bound) as a Real variable.
    /// Java: `getMinX()`
    fn min_x(&self) -> RealId;

    /// Maximum X extent (right bound) as a Real variable.
    /// Java: `getMaxX()`
    fn max_x(&self) -> RealId;
}

/// Common base state shared by all tile implementations.
/// Java: `CommonTile` fields
#[derive(Debug)]
pub struct TileState {
    /// Assigned Y position (set by callback_y)
    pub y: Option<f64>,
    /// Hook type from the last callback
    pub hook_type: HookType,
}

impl TileState {
    pub fn new() -> Self {
        Self {
            y: None,
            hook_type: HookType::Start,
        }
    }

    pub fn callback_y(&mut self, hook: TimeHook) {
        self.y = Some(hook.value);
        self.hook_type = hook.hook_type;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_hook_value() {
        let h = TimeHook::new(42.5, HookType::Start);
        assert!((h.get_value() - 42.5).abs() < f64::EPSILON);
        assert_eq!(h.hook_type, HookType::Start);
    }

    #[test]
    fn tile_state_initial() {
        let s = TileState::new();
        assert!(s.y.is_none());
        assert_eq!(s.hook_type, HookType::Start);
    }

    #[test]
    fn tile_state_callback() {
        let mut s = TileState::new();
        s.callback_y(TimeHook::new(100.0, HookType::Continue));
        assert!((s.y.unwrap() - 100.0).abs() < f64::EPSILON);
        assert_eq!(s.hook_type, HookType::Continue);
    }
}
