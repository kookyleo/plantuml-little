// layout::sequence_teoz::real - Real constraint propagation
//
// Port of Java PlantUML's net.sourceforge.plantuml.real package.
// Arena-based one-directional constraint solver: values only increase.
//
// The constraint system works as follows:
// - RealLine is an arena that owns all Real nodes and PositiveForce constraints.
// - Each Real node has a kind: Base (moveable value), Delta (fixed offset from
//   another Real), Max (maximum of children), Min (minimum of children), or
//   Middle (midpoint of two Reals).
// - PositiveForce represents a one-directional constraint: movable >= fixed + min_distance.
// - compile() iteratively applies forces until all constraints are satisfied (values
//   only increase, never decrease -- monotonic relaxation).

/// Index into the RealLine arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RealId(pub usize);

/// A single inequality constraint: movable >= fixed + min_distance.
/// Java: `real.PositiveForce`
#[derive(Debug)]
struct PositiveForce {
	fixed: RealId,
	movable: RealId,
	min_distance: f64,
}

/// Kind of Real node in the arena.
#[derive(Debug)]
enum RealKind {
	/// Moveable base value. Java: `RealImpl`
	Base { value: f64 },
	/// Fixed offset from another Real: value = base + delta. Java: `RealDelta`
	Delta { base: RealId, delta: f64 },
	/// Maximum of multiple Reals. Java: `RealMax`
	Max { children: Vec<RealId> },
	/// Minimum of multiple Reals. Java: `RealMin`
	Min { children: Vec<RealId> },
	/// Midpoint of two Reals. Java: `RealMiddle2`
	/// Value = (p1 + p2) / 2.
	Middle { p1: RealId, p2: RealId },
}

/// Arena-based constraint solver.
/// Java: `real.RealLine` + all Real node types.
///
/// All Real nodes live in a single arena. Constraints (PositiveForce) are
/// collected and solved by iterative relaxation in `compile()`.
#[derive(Debug)]
pub struct RealLine {
	nodes: Vec<RealKind>,
	forces: Vec<PositiveForce>,
}

impl RealLine {
	pub fn new() -> Self {
		Self {
			nodes: Vec::new(),
			forces: Vec::new(),
		}
	}

	/// Create a new moveable origin with initial value 0.
	/// Java: `RealUtils.createOrigin()`
	pub fn create_origin(&mut self) -> RealId {
		self.create_base(0.0)
	}

	/// Create a moveable base value.
	/// Java: `new RealImpl(name, line, value)`
	pub fn create_base(&mut self, value: f64) -> RealId {
		let id = RealId(self.nodes.len());
		self.nodes.push(RealKind::Base { value });
		id
	}

	/// Create an immutable offset: result = base + delta.
	/// Java: `real.addFixed(delta)` → `new RealDelta(real, delta)`
	pub fn add_fixed(&mut self, base: RealId, delta: f64) -> RealId {
		let id = RealId(self.nodes.len());
		self.nodes.push(RealKind::Delta { base, delta });
		id
	}

	/// Create a new base with constraint: result >= base + delta.
	/// Java: `RealImpl.addAtLeast(delta)`
	pub fn add_at_least(&mut self, base: RealId, delta: f64) -> RealId {
		let base_val = self.get_value(base);
		let result = self.create_base(base_val + delta);
		self.forces.push(PositiveForce {
			fixed: base,
			movable: result,
			min_distance: delta,
		});
		result
	}

	/// Add constraint: a >= b.
	/// Java: `a.ensureBiggerThan(b)` → `new PositiveForce(b, a, 0)`
	pub fn ensure_bigger_than(&mut self, a: RealId, b: RealId) {
		self.forces.push(PositiveForce {
			fixed: b,
			movable: a,
			min_distance: 0.0,
		});
	}

	/// Add constraint: a >= b + min_distance.
	pub fn ensure_bigger_than_with_margin(&mut self, a: RealId, b: RealId, min_distance: f64) {
		self.forces.push(PositiveForce {
			fixed: b,
			movable: a,
			min_distance,
		});
	}

	/// Create max(children). Java: `RealMax`
	pub fn max_of(&mut self, children: Vec<RealId>) -> RealId {
		let id = RealId(self.nodes.len());
		self.nodes.push(RealKind::Max { children });
		id
	}

	/// Create min(children). Java: `RealMin`
	pub fn min_of(&mut self, children: Vec<RealId>) -> RealId {
		let id = RealId(self.nodes.len());
		self.nodes.push(RealKind::Min { children });
		id
	}

	/// Create middle(p1, p2) -- value is (p1 + p2) / 2. Java: `RealMiddle2`
	pub fn middle_of(&mut self, p1: RealId, p2: RealId) -> RealId {
		let id = RealId(self.nodes.len());
		self.nodes.push(RealKind::Middle { p1, p2 });
		id
	}

	/// Return the number of nodes in the arena.
	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	/// Return whether the arena is empty.
	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}

	/// Return the number of forces (constraints) in the arena.
	pub fn force_count(&self) -> usize {
		self.forces.len()
	}

	/// Get the minimum value across all Base nodes in the arena.
	/// Corresponds to RealLine.getAbsoluteMin() in Java (computed after compile).
	pub fn get_absolute_min(&self) -> f64 {
		if self.nodes.is_empty() {
			return 0.0;
		}
		let mut min = f64::MAX;
		for i in 0..self.nodes.len() {
			let v = self.get_value(RealId(i));
			if v < min {
				min = v;
			}
		}
		min
	}

	/// Get the maximum value across all Base nodes in the arena.
	/// Corresponds to RealLine.getAbsoluteMax() in Java (computed after compile).
	pub fn get_absolute_max(&self) -> f64 {
		if self.nodes.is_empty() {
			return 0.0;
		}
		let mut max = f64::MIN;
		for i in 0..self.nodes.len() {
			let v = self.get_value(RealId(i));
			if v > max {
				max = v;
			}
		}
		max
	}

	/// Get current value of a Real node.
	/// Recursively evaluates Delta/Max/Min/Middle compositions.
	pub fn get_value(&self, id: RealId) -> f64 {
		self.get_value_with_depth(id, 0)
	}

	/// Internal recursive value computation with depth guard to detect cycles.
	fn get_value_with_depth(&self, id: RealId, depth: usize) -> f64 {
		if depth > 1000 {
			panic!("Infinite recursion detected in get_value");
		}
		match &self.nodes[id.0] {
			RealKind::Base { value } => *value,
			RealKind::Delta { base, delta } => self.get_value_with_depth(*base, depth + 1) + delta,
			RealKind::Max { children } => {
				let mut result = self.get_value_with_depth(children[0], depth + 1);
				for i in 1..children.len() {
					let v = self.get_value_with_depth(children[i], depth + 1);
					if v > result {
						result = v;
					}
				}
				result
			}
			RealKind::Min { children } => {
				let mut result = self.get_value_with_depth(children[0], depth + 1);
				for i in 1..children.len() {
					let v = self.get_value_with_depth(children[i], depth + 1);
					if v < result {
						result = v;
					}
				}
				result
			}
			RealKind::Middle { p1, p2 } => {
				let v1 = self.get_value_with_depth(*p1, depth + 1);
				let v2 = self.get_value_with_depth(*p2, depth + 1);
				(v1 + v2) / 2.0
			}
		}
	}

	/// Move a node's value forward by delta.
	/// Follows the same delegation pattern as Java:
	/// - Base: directly mutate the value
	/// - Delta: delegate to the underlying base
	/// - Middle: split the delta equally between p1 and p2
	/// - Max/Min: cannot be moved (logged as warning)
	fn move_forward(&mut self, id: RealId, delta: f64) {
		match self.nodes[id.0] {
			RealKind::Base { ref mut value } => {
				*value += delta;
			}
			RealKind::Delta { base, .. } => {
				self.move_forward(base, delta);
			}
			RealKind::Middle { p1, p2 } => {
				self.move_forward(p1, delta / 2.0);
				self.move_forward(p2, delta / 2.0);
			}
			RealKind::Max { .. } | RealKind::Min { .. } => {
				log::warn!("move_forward on non-moveable Max/Min node");
			}
		}
	}

	/// Apply one force. Returns true if the movable point was adjusted.
	/// Java: `PositiveForce.apply()`
	fn apply_force(&mut self, force_idx: usize) -> bool {
		let fixed_val = self.get_value(self.forces[force_idx].fixed);
		let movable_val = self.get_value(self.forces[force_idx].movable);
		let min_dist = self.forces[force_idx].min_distance;
		let distance = movable_val - fixed_val;
		let diff = distance - min_dist;
		if diff >= 0.0 {
			return false;
		}
		// Push movable forward
		let movable = self.forces[force_idx].movable;
		self.move_forward(movable, -diff);
		true
	}

	/// Solve all constraints by iterative relaxation.
	/// Java: `RealLine.compile()`
	///
	/// Iterates through all forces, applying each one.
	/// Repeats until no force triggers a change (convergence).
	/// Panics after 100K iterations (cycle detection).
	pub fn compile(&mut self) {
		let mut iterations = 0u32;
		loop {
			let mut changed = false;
			for i in 0..self.forces.len() {
				if self.apply_force(i) {
					changed = true;
				}
			}
			if !changed {
				return;
			}
			iterations += 1;
			if iterations > 99999 {
				panic!("Real constraint solver: infinite loop after 100K iterations");
			}
		}
	}
}

impl Default for RealLine {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// -- Basic operations: create, add_fixed, get_value -------------------------

	#[test]
	fn create_origin_value_zero() {
		let mut rl = RealLine::new();
		let o = rl.create_origin();
		assert!((rl.get_value(o) - 0.0).abs() < f64::EPSILON);
	}

	#[test]
	fn create_base_with_value() {
		let mut rl = RealLine::new();
		let a = rl.create_base(42.0);
		assert!((rl.get_value(a) - 42.0).abs() < f64::EPSILON);
	}

	#[test]
	fn add_fixed_offset() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.add_fixed(a, 5.0);
		assert!((rl.get_value(b) - 15.0).abs() < f64::EPSILON);
	}

	#[test]
	fn add_fixed_negative_delta() {
		let mut rl = RealLine::new();
		let a = rl.create_base(50.0);
		let b = rl.add_fixed(a, -30.0);
		assert!((rl.get_value(b) - 20.0).abs() < f64::EPSILON);
	}

	#[test]
	fn add_fixed_chain() {
		let mut rl = RealLine::new();
		let a = rl.create_base(100.0);
		let b = rl.add_fixed(a, 20.0);
		let c = rl.add_fixed(b, 30.0);
		assert!((rl.get_value(a) - 100.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 120.0).abs() < f64::EPSILON);
		assert!((rl.get_value(c) - 150.0).abs() < f64::EPSILON);
	}

	// -- Constraint solving: ensure_bigger_than propagation ---------------------

	#[test]
	fn ensure_bigger_than_no_change() {
		let mut rl = RealLine::new();
		let a = rl.create_base(20.0);
		let b = rl.create_base(10.0);
		rl.ensure_bigger_than(a, b); // a >= b, already satisfied
		rl.compile();
		assert!((rl.get_value(a) - 20.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 10.0).abs() < f64::EPSILON);
	}

	#[test]
	fn ensure_bigger_than_push_forward() {
		let mut rl = RealLine::new();
		let a = rl.create_base(5.0);
		let b = rl.create_base(10.0);
		rl.ensure_bigger_than(a, b); // a >= b, need to push a to 10
		rl.compile();
		assert!((rl.get_value(a) - 10.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 10.0).abs() < f64::EPSILON);
	}

	#[test]
	fn ensure_bigger_than_with_margin() {
		let mut rl = RealLine::new();
		let a = rl.create_base(5.0);
		let b = rl.create_base(10.0);
		rl.ensure_bigger_than_with_margin(a, b, 3.0); // a >= b + 3
		rl.compile();
		assert!((rl.get_value(a) - 13.0).abs() < f64::EPSILON);
	}

	#[test]
	fn ensure_bigger_than_chain_all_zeros() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.create_base(0.0);
		let c = rl.create_base(0.0);
		rl.ensure_bigger_than(b, a);
		rl.ensure_bigger_than(c, b);
		rl.compile();
		// All at 0, constraints already satisfied
		assert!((rl.get_value(a) - 0.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 0.0).abs() < f64::EPSILON);
		assert!((rl.get_value(c) - 0.0).abs() < f64::EPSILON);
	}

	#[test]
	fn ensure_bigger_than_chain_with_initial_values() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(3.0);
		let c = rl.create_base(1.0);
		rl.ensure_bigger_than(b, a); // b >= a(10)
		rl.ensure_bigger_than(c, b); // c >= b
		rl.compile();
		assert!(rl.get_value(b) >= 10.0 - f64::EPSILON);
		assert!(rl.get_value(c) >= rl.get_value(b) - f64::EPSILON);
	}

	// -- add_at_least creates constraints ---------------------------------------

	#[test]
	fn add_at_least_creates_constraint() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.add_at_least(a, 10.0);
		assert!((rl.get_value(b) - 10.0).abs() < f64::EPSILON);
		rl.compile();
		assert!(rl.get_value(b) >= rl.get_value(a) + 10.0 - f64::EPSILON);
	}

	#[test]
	fn add_at_least_constraint_enforced_when_base_moves() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.add_at_least(a, 10.0);
		let c = rl.create_base(20.0);

		// Force a to be at least 20
		rl.ensure_bigger_than(a, c);
		rl.compile();

		// a was pushed to 20, so b must be >= 20 + 10 = 30
		assert!(rl.get_value(a) >= 20.0 - f64::EPSILON);
		assert!(rl.get_value(b) >= 30.0 - f64::EPSILON);
	}

	#[test]
	fn add_at_least_chain() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.add_at_least(a, 5.0);
		let c = rl.add_at_least(b, 5.0);
		let d = rl.add_at_least(c, 5.0);
		rl.compile();
		assert!(rl.get_value(b) >= 5.0 - f64::EPSILON);
		assert!(rl.get_value(c) >= 10.0 - f64::EPSILON);
		assert!(rl.get_value(d) >= 15.0 - f64::EPSILON);
	}

	// -- Max / Min compositions -------------------------------------------------

	#[test]
	fn max_of_two() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(20.0);
		let m = rl.max_of(vec![a, b]);
		assert!((rl.get_value(m) - 20.0).abs() < f64::EPSILON);
	}

	#[test]
	fn max_of_three() {
		let mut rl = RealLine::new();
		let a = rl.create_base(5.0);
		let b = rl.create_base(15.0);
		let c = rl.create_base(10.0);
		let m = rl.max_of(vec![a, b, c]);
		assert!((rl.get_value(m) - 15.0).abs() < f64::EPSILON);
	}

	#[test]
	fn min_of_two() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(20.0);
		let m = rl.min_of(vec![a, b]);
		assert!((rl.get_value(m) - 10.0).abs() < f64::EPSILON);
	}

	#[test]
	fn min_of_three() {
		let mut rl = RealLine::new();
		let a = rl.create_base(5.0);
		let b = rl.create_base(15.0);
		let c = rl.create_base(10.0);
		let m = rl.min_of(vec![a, b, c]);
		assert!((rl.get_value(m) - 5.0).abs() < f64::EPSILON);
	}

	#[test]
	fn max_reflects_constraint_changes() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(20.0);
		let m = rl.max_of(vec![a, b]);
		assert!((rl.get_value(m) - 20.0).abs() < f64::EPSILON);

		// Push a to 30 via a constraint
		let c = rl.create_base(30.0);
		rl.ensure_bigger_than(a, c);
		rl.compile();

		// Now max should reflect a=30
		assert!((rl.get_value(m) - 30.0).abs() < f64::EPSILON);
	}

	#[test]
	fn min_reflects_constraint_changes() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(20.0);
		let m = rl.min_of(vec![a, b]);
		assert!((rl.get_value(m) - 10.0).abs() < f64::EPSILON);

		// Push a to 25 via a constraint
		let c = rl.create_base(25.0);
		rl.ensure_bigger_than(a, c);
		rl.compile();

		// Now min should be b=20 (since a=25 > b=20)
		assert!((rl.get_value(m) - 20.0).abs() < f64::EPSILON);
	}

	#[test]
	fn add_fixed_on_max() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(30.0);
		let m = rl.max_of(vec![a, b]);
		let d = rl.add_fixed(m, 5.0);
		assert!((rl.get_value(d) - 35.0).abs() < f64::EPSILON);
	}

	#[test]
	fn add_fixed_on_min() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(30.0);
		let m = rl.min_of(vec![a, b]);
		let d = rl.add_fixed(m, -3.0);
		assert!((rl.get_value(d) - 7.0).abs() < f64::EPSILON);
	}

	// -- Multi-step constraint chains (A >= B >= C, change C) -------------------

	#[test]
	fn chain_propagation() {
		let mut rl = RealLine::new();
		let c = rl.create_base(100.0);
		let b = rl.create_base(0.0);
		let a = rl.create_base(0.0);
		rl.ensure_bigger_than(b, c); // b >= c
		rl.ensure_bigger_than(a, b); // a >= b
		rl.compile();
		assert!((rl.get_value(a) - 100.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 100.0).abs() < f64::EPSILON);
	}

	#[test]
	fn chain_with_margins() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.create_base(0.0);
		let c = rl.create_base(0.0);
		rl.ensure_bigger_than_with_margin(b, a, 10.0); // b >= a + 10
		rl.ensure_bigger_than_with_margin(c, b, 20.0); // c >= b + 20
		rl.compile();
		assert!((rl.get_value(a) - 0.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 10.0).abs() < f64::EPSILON);
		assert!((rl.get_value(c) - 30.0).abs() < f64::EPSILON);
	}

	#[test]
	fn multi_step_with_add_at_least_distances() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.add_at_least(a, 10.0);
		let c = rl.add_at_least(b, 20.0);
		let d = rl.add_at_least(c, 30.0);
		rl.compile();
		assert!((rl.get_value(a) - 0.0).abs() < f64::EPSILON);
		assert!(rl.get_value(b) >= 10.0 - f64::EPSILON);
		assert!(rl.get_value(c) >= 30.0 - f64::EPSILON);
		assert!(rl.get_value(d) >= 60.0 - f64::EPSILON);
	}

	#[test]
	fn constraint_chain_pushes_downstream() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.add_at_least(a, 10.0);
		let c = rl.add_at_least(b, 10.0);

		assert!((rl.get_value(a) - 0.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 10.0).abs() < f64::EPSILON);
		assert!((rl.get_value(c) - 20.0).abs() < f64::EPSILON);

		// Push a to at least 50
		let big = rl.create_base(50.0);
		rl.ensure_bigger_than(a, big);
		rl.compile();

		assert!((rl.get_value(a) - 50.0).abs() < f64::EPSILON);
		assert!(rl.get_value(b) >= 60.0 - f64::EPSILON);
		assert!(rl.get_value(c) >= 70.0 - f64::EPSILON);
	}

	// -- Iterative solver convergence -------------------------------------------

	#[test]
	fn solver_converges_simple() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.create_base(0.0);
		rl.ensure_bigger_than(b, a);
		rl.compile(); // Should converge immediately
	}

	#[test]
	fn solver_converges_long_chain() {
		let mut rl = RealLine::new();
		let mut prev = rl.create_base(0.0);
		for _ in 0..20 {
			prev = rl.add_at_least(prev, 5.0);
		}
		rl.compile();
		// Last node should be >= 100 (20 * 5)
		assert!(rl.get_value(prev) >= 100.0 - f64::EPSILON);
	}

	#[test]
	fn solver_handles_cross_constraints() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.create_base(0.0);
		let c = rl.create_base(0.0);

		// b >= a + 10 (via add_at_least + ensure_bigger_than)
		let b_target = rl.add_at_least(a, 10.0);
		rl.ensure_bigger_than(b, b_target);

		// c >= b + 10
		let c_target = rl.add_at_least(b, 10.0);
		rl.ensure_bigger_than(c, c_target);

		// a >= 5
		let five = rl.create_base(5.0);
		rl.ensure_bigger_than(a, five);

		rl.compile();

		assert!(rl.get_value(a) >= 5.0 - f64::EPSILON);
		assert!(rl.get_value(b) >= 15.0 - f64::EPSILON);
		assert!(rl.get_value(c) >= 25.0 - f64::EPSILON);
	}

	#[test]
	fn compile_idempotent() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.create_base(0.0);
		rl.ensure_bigger_than_with_margin(b, a, 10.0);
		rl.compile();
		let v1 = rl.get_value(b);
		rl.compile(); // second compile should not change anything
		let v2 = rl.get_value(b);
		assert!((v1 - v2).abs() < f64::EPSILON);
	}

	#[test]
	fn empty_line_compile_is_noop() {
		let mut rl = RealLine::new();
		rl.compile(); // Should not panic
	}

	#[test]
	fn no_forces_compile_is_noop() {
		let mut rl = RealLine::new();
		let _a = rl.create_base(10.0);
		let _b = rl.create_base(20.0);
		rl.compile(); // No forces, nothing to do
	}

	// -- Middle node ------------------------------------------------------------

	#[test]
	fn middle_of_two() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(30.0);
		let m = rl.middle_of(a, b);
		assert!((rl.get_value(m) - 20.0).abs() < f64::EPSILON);
	}

	#[test]
	fn middle_reflects_changes() {
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.create_base(100.0);
		let m = rl.middle_of(a, b);
		assert!((rl.get_value(m) - 50.0).abs() < f64::EPSILON);

		// Push a to 40
		let forty = rl.create_base(40.0);
		rl.ensure_bigger_than(a, forty);
		rl.compile();

		// Now middle = (40 + 100) / 2 = 70
		assert!((rl.get_value(m) - 70.0).abs() < f64::EPSILON);
	}

	// -- Values only increase (monotonic) ---------------------------------------

	#[test]
	fn values_only_increase() {
		let mut rl = RealLine::new();
		let a = rl.create_base(10.0);
		let b = rl.create_base(20.0);
		// a >= b: a will be pushed from 10 to 20, b stays at 20
		rl.ensure_bigger_than(a, b);
		rl.compile();
		assert!((rl.get_value(a) - 20.0).abs() < f64::EPSILON);
		assert!((rl.get_value(b) - 20.0).abs() < f64::EPSILON);
	}

	// -- Utility methods --------------------------------------------------------

	#[test]
	fn len_and_is_empty() {
		let mut rl = RealLine::new();
		assert!(rl.is_empty());
		assert_eq!(rl.len(), 0);

		let _a = rl.create_base(0.0);
		assert!(rl.is_empty() == false);
		assert_eq!(rl.len(), 1);
	}

	#[test]
	fn force_count() {
		let mut rl = RealLine::new();
		assert_eq!(rl.force_count(), 0);

		let a = rl.create_base(0.0);
		let _b = rl.add_at_least(a, 10.0);
		assert_eq!(rl.force_count(), 1);

		let c = rl.create_base(0.0);
		rl.ensure_bigger_than(c, a);
		assert_eq!(rl.force_count(), 2);
	}

	#[test]
	fn absolute_min_max() {
		let mut rl = RealLine::new();
		let _a = rl.create_base(-10.0);
		let _b = rl.create_base(5.0);
		let _c = rl.create_base(30.0);
		assert!((rl.get_absolute_min() - (-10.0)).abs() < f64::EPSILON);
		assert!((rl.get_absolute_max() - 30.0).abs() < f64::EPSILON);
	}

	#[test]
	fn absolute_min_max_empty() {
		let rl = RealLine::new();
		assert!((rl.get_absolute_min() - 0.0).abs() < f64::EPSILON);
		assert!((rl.get_absolute_max() - 0.0).abs() < f64::EPSILON);
	}

	#[test]
	fn default_trait() {
		let rl = RealLine::default();
		assert!(rl.is_empty());
		assert_eq!(rl.force_count(), 0);
	}

	// -- Sequence-diagram-like scenarios ----------------------------------------

	#[test]
	fn delta_with_constraint_propagation() {
		// Simulates participant layout:
		// p1.posB = origin, p1.posC = posB + 20, p1.posD = posB + 40
		// p2.posB >= p1.posD + 10 (gap between participants)
		let mut rl = RealLine::new();
		let p1_b = rl.create_base(5.0);
		let p1_c = rl.add_fixed(p1_b, 20.0);
		let p1_d = rl.add_fixed(p1_b, 40.0);
		let p2_b = rl.add_at_least(p1_d, 10.0);
		let p2_c = rl.add_fixed(p2_b, 20.0);
		let p2_d = rl.add_fixed(p2_b, 40.0);
		rl.compile();

		assert!((rl.get_value(p1_b) - 5.0).abs() < f64::EPSILON);
		assert!((rl.get_value(p1_c) - 25.0).abs() < f64::EPSILON);
		assert!((rl.get_value(p1_d) - 45.0).abs() < f64::EPSILON);
		assert!(rl.get_value(p2_b) >= 55.0 - f64::EPSILON);
		assert!((rl.get_value(p2_c) - rl.get_value(p2_b) - 20.0).abs() < f64::EPSILON);
		assert!((rl.get_value(p2_d) - rl.get_value(p2_b) - 40.0).abs() < f64::EPSILON);
	}

	#[test]
	fn message_constraint_widens_gap() {
		// Two participants too close, message constraint pushes them apart
		let mut rl = RealLine::new();
		let p1_b = rl.create_base(0.0);
		let p1_c = rl.add_fixed(p1_b, 15.0);
		let p1_d = rl.add_fixed(p1_b, 30.0);
		let p2_b = rl.add_at_least(p1_d, 5.0);
		let p2_c = rl.add_fixed(p2_b, 15.0);

		// Message requires at least 80px between centers
		rl.ensure_bigger_than_with_margin(p2_c, p1_c, 80.0);
		rl.compile();

		let gap = rl.get_value(p2_c) - rl.get_value(p1_c);
		assert!(
			gap >= 80.0 - f64::EPSILON,
			"gap {gap} should be >= 80"
		);
	}

	#[test]
	fn sequence_four_participants_spacing() {
		let mut rl = RealLine::new();
		let p0 = rl.create_base(0.0);
		let p1 = rl.add_at_least(p0, 80.0);
		let p2 = rl.add_at_least(p1, 80.0);
		let p3 = rl.add_at_least(p2, 80.0);
		rl.compile();

		let d01 = rl.get_value(p1) - rl.get_value(p0);
		let d12 = rl.get_value(p2) - rl.get_value(p1);
		let d23 = rl.get_value(p3) - rl.get_value(p2);

		assert!(d01 >= 80.0 - f64::EPSILON, "d01={d01} should be >= 80");
		assert!(d12 >= 80.0 - f64::EPSILON, "d12={d12} should be >= 80");
		assert!(d23 >= 80.0 - f64::EPSILON, "d23={d23} should be >= 80");
	}

	#[test]
	fn sequence_message_widens_spacing() {
		// Participant A, B, C at 80px spacing,
		// but a long message from A to C requires 200px minimum.
		let mut rl = RealLine::new();
		let a = rl.create_base(0.0);
		let b = rl.add_at_least(a, 80.0);
		let c = rl.add_at_least(b, 80.0);

		// Long message constraint: C >= A + 200
		let msg_target = rl.add_at_least(a, 200.0);
		rl.ensure_bigger_than(c, msg_target);
		rl.compile();

		let ac_dist = rl.get_value(c) - rl.get_value(a);
		assert!(ac_dist >= 200.0 - f64::EPSILON, "A-C distance={ac_dist} should be >= 200");
		// B is still between A and C
		assert!(rl.get_value(b) >= rl.get_value(a) + 80.0 - f64::EPSILON);
		assert!(rl.get_value(c) >= rl.get_value(b) + 80.0 - f64::EPSILON);
	}
}
