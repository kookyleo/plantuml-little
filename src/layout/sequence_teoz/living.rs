// layout::sequence_teoz::living - Participant column management
//
// Port of Java PlantUML's sequencediagram.teoz.LivingSpace,
// LivingSpaces, LiveBoxes, LiveBoxesDrawer, Stairs, and Step.
//
// Each participant has a LivingSpace that tracks:
// - X position via Real constraint variables (posB, posC, posD)
// - Activation state via LiveBoxes (nesting levels at each Y)

use super::real::RealId;

/// Activation nesting step at a specific Y position.
/// Java: `teoz.Step`
#[derive(Debug, Clone)]
pub struct Step {
    pub y: f64,
    pub indent: i32,
}

/// A sequence of activation steps for one participant.
/// Java: `teoz.Stairs`
#[derive(Debug, Clone)]
pub struct Stairs {
    steps: Vec<Step>,
}

impl Default for Stairs {
    fn default() -> Self {
        Self::new()
    }
}

impl Stairs {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
}

/// Tracks activation/deactivation events for one participant.
/// Java: `teoz.LiveBoxes`
#[derive(Debug)]
pub struct LiveBoxes {
    /// Event index → Y position mapping
    events_y: Vec<(usize, f64)>,
}

impl Default for LiveBoxes {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveBoxes {
    pub fn new() -> Self {
        Self {
            events_y: Vec::new(),
        }
    }

    /// Record an event's Y position.
    /// Java: `addStep(Event, double)`
    pub fn add_step(&mut self, event_idx: usize, y: f64) {
        self.events_y.push((event_idx, y));
    }
}

/// One participant's vertical column with Real constraint positions.
/// Java: `teoz.LivingSpace`
#[derive(Debug)]
pub struct LivingSpace {
    pub name: String,
    /// Left edge of participant box (Real variable)
    pub pos_b: RealId,
    /// Center of participant (derived: pos_b + width/2)
    pub pos_c: RealId,
    /// Right edge of participant box (derived: pos_b + width)
    pub pos_d: RealId,
    /// Activation tracking
    pub live_boxes: LiveBoxes,
}

impl LivingSpace {
    /// Create a new LivingSpace.
    /// `pos_b` is the left-edge Real; `pos_c` and `pos_d` are derived offsets.
    pub fn new(name: String, pos_b: RealId, pos_c: RealId, pos_d: RealId) -> Self {
        Self {
            name,
            pos_b,
            pos_c,
            pos_d,
            live_boxes: LiveBoxes::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stairs_add_steps() {
        let mut stairs = Stairs::new();
        stairs.add_step(Step { y: 10.0, indent: 0 });
        stairs.add_step(Step { y: 50.0, indent: 1 });
        stairs.add_step(Step { y: 80.0, indent: 0 });
        assert_eq!(stairs.steps().len(), 3);
        assert_eq!(stairs.steps()[1].indent, 1);
    }

    #[test]
    fn live_boxes_tracks_events() {
        let mut lb = LiveBoxes::new();
        lb.add_step(0, 30.0);
        lb.add_step(3, 70.0);
        assert_eq!(lb.events_y.len(), 2);
    }
}
