// skin::arrow - Arrow configuration for sequence diagrams
// Port of Java PlantUML's skin.Arrow* classes

use std::fmt;

/// Arrow head style. Java: `skin.ArrowHead`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowHead {
    #[default]
    Normal,
    Async,
    CrossX,
    None,
}

impl fmt::Display for ArrowHead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrowHead::Normal => write!(f, "NORMAL"),
            ArrowHead::Async => write!(f, "ASYNC"),
            ArrowHead::CrossX => write!(f, "CROSSX"),
            ArrowHead::None => write!(f, "NONE"),
        }
    }
}

/// Arrow body line style. Java: `skin.ArrowBody`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowBody {
    #[default]
    Normal,
    Dotted,
    Dashed,
    Hidden,
    Bold,
}

impl fmt::Display for ArrowBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrowBody::Normal => write!(f, "NORMAL"),
            ArrowBody::Dotted => write!(f, "DOTTED"),
            ArrowBody::Dashed => write!(f, "DASHED"),
            ArrowBody::Hidden => write!(f, "HIDDEN"),
            ArrowBody::Bold => write!(f, "BOLD"),
        }
    }
}

/// Arrow direction. Java: `skin.ArrowDirection`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    Self_,
    Both,
}

impl ArrowDirection {
    /// Reverse the direction (LEFT_TO_RIGHT <-> RIGHT_TO_LEFT).
    /// Panics for Self_ and Both, matching the Java behavior.
    pub fn reverse(self) -> Self {
        match self {
            ArrowDirection::LeftToRight => ArrowDirection::RightToLeft,
            ArrowDirection::RightToLeft => ArrowDirection::LeftToRight,
            _ => panic!("cannot reverse {:?}", self),
        }
    }
}

/// Arrow part (which half to draw for self-messages). Java: `skin.ArrowPart`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowPart {
    #[default]
    Full,
    TopPart,
    BottomPart,
}

/// Arrow endpoint decoration. Java: `skin.ArrowDecoration`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowDecoration {
    #[default]
    None,
    Circle,
}

impl fmt::Display for ArrowDecoration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrowDecoration::None => write!(f, "NONE"),
            ArrowDecoration::Circle => write!(f, "CIRCLE"),
        }
    }
}

/// Arrow endpoint dressing (head + part). Java: `skin.ArrowDressing`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowDressing {
    pub head: ArrowHead,
    pub part: ArrowPart,
}

impl ArrowDressing {
    /// Create a dressing with a specific head and full part.
    pub fn new(head: ArrowHead) -> Self {
        Self {
            head,
            part: ArrowPart::Full,
        }
    }

    /// Create a default dressing with no head. Java: `ArrowDressing.create()`
    pub fn none() -> Self {
        Self {
            head: ArrowHead::None,
            part: ArrowPart::Full,
        }
    }

    /// Return a new dressing with the given head, preserving part.
    pub fn with_head(self, head: ArrowHead) -> Self {
        Self { head, ..self }
    }

    /// Return a new dressing with the given part, preserving head.
    pub fn with_part(self, part: ArrowPart) -> Self {
        Self { part, ..self }
    }
}

impl Default for ArrowDressing {
    fn default() -> Self {
        Self {
            head: ArrowHead::Normal,
            part: ArrowPart::Full,
        }
    }
}

impl fmt::Display for ArrowDressing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.head)
    }
}

/// Complete arrow configuration. Java: `skin.ArrowConfiguration`
///
/// This is an immutable value type -- all `with_*` methods return a new instance,
/// mirroring the Java builder pattern.
#[derive(Debug, Clone)]
pub struct ArrowConfiguration {
    body: ArrowBody,
    dressing1: ArrowDressing,
    dressing2: ArrowDressing,
    decoration1: ArrowDecoration,
    decoration2: ArrowDecoration,
    color: Option<String>,
    is_self: bool,
    thickness: f64,
    reverse_define: bool,
    inclination: i32,
}

impl fmt::Display for ArrowConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ArrowConfiguration {
    // ---- private constructor ----

    #[allow(clippy::too_many_arguments)]
    fn new(
        body: ArrowBody,
        dressing1: ArrowDressing,
        dressing2: ArrowDressing,
        decoration1: ArrowDecoration,
        decoration2: ArrowDecoration,
        color: Option<String>,
        is_self: bool,
        thickness: f64,
        reverse_define: bool,
        inclination: i32,
    ) -> Self {
        Self {
            body,
            dressing1,
            dressing2,
            decoration1,
            decoration2,
            color,
            is_self,
            thickness,
            reverse_define,
            inclination,
        }
    }

    // ---- factory methods ----

    /// Create an arrow pointing left-to-right (normal direction).
    /// Java: `ArrowConfiguration.withDirectionNormal()`
    pub fn with_direction_normal() -> Self {
        Self::new(
            ArrowBody::Normal,
            ArrowDressing::none(),
            ArrowDressing::none().with_head(ArrowHead::Normal),
            ArrowDecoration::None,
            ArrowDecoration::None,
            None,
            false,
            1.0,
            false,
            0,
        )
    }

    /// Create a bidirectional arrow.
    /// Java: `ArrowConfiguration.withDirectionBoth()`
    pub fn with_direction_both() -> Self {
        Self::new(
            ArrowBody::Normal,
            ArrowDressing::none().with_head(ArrowHead::Normal),
            ArrowDressing::none().with_head(ArrowHead::Normal),
            ArrowDecoration::None,
            ArrowDecoration::None,
            None,
            false,
            1.0,
            false,
            0,
        )
    }

    /// Create a self-referencing arrow.
    /// Java: `ArrowConfiguration.withDirectionSelf(boolean)`
    pub fn with_direction_self(reverse_define: bool) -> Self {
        Self::new(
            ArrowBody::Normal,
            ArrowDressing::none(),
            ArrowDressing::none().with_head(ArrowHead::Normal),
            ArrowDecoration::None,
            ArrowDecoration::None,
            None,
            true,
            1.0,
            reverse_define,
            0,
        )
    }

    /// Create an arrow pointing right-to-left (reverse direction).
    /// Java: `ArrowConfiguration.withDirectionReverse()`
    pub fn with_direction_reverse() -> Self {
        Self::with_direction_normal().reverse()
    }

    // ---- builder methods (return new instance) ----

    /// Swap dressing1/dressing2 and decoration1/decoration2.
    /// Java: `ArrowConfiguration.reverse()`
    pub fn reverse(&self) -> Self {
        Self::new(
            self.body,
            self.dressing2,
            self.dressing1,
            self.decoration2,
            self.decoration1,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Mark as self-arrow.
    /// Java: `ArrowConfiguration.self()`
    pub fn self_arrow(&self) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2,
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            true,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set the arrow body style.
    /// Java: `ArrowConfiguration.withBody(ArrowBody)`
    pub fn with_body(&self, body: ArrowBody) -> Self {
        Self::new(
            body,
            self.dressing1,
            self.dressing2,
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set head on all non-NONE dressings.
    /// Java: `ArrowConfiguration.withHead(ArrowHead)`
    pub fn with_head(&self, head: ArrowHead) -> Self {
        let new_d1 = add_head(self.dressing1, head);
        let new_d2 = add_head(self.dressing2, head);
        Self::new(
            self.body,
            new_d1,
            new_d2,
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set head on dressing1 only.
    /// Java: `ArrowConfiguration.withHead1(ArrowHead)`
    pub fn with_head1(&self, head: ArrowHead) -> Self {
        Self::new(
            self.body,
            self.dressing1.with_head(head),
            self.dressing2,
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set head on dressing2 only.
    /// Java: `ArrowConfiguration.withHead2(ArrowHead)`
    pub fn with_head2(&self, head: ArrowHead) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2.with_head(head),
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set part on the dressing that has the arrow head.
    /// Java: `ArrowConfiguration.withPart(ArrowPart)`
    pub fn with_part(&self, part: ArrowPart) -> Self {
        if self.dressing2.head != ArrowHead::None {
            Self::new(
                self.body,
                self.dressing1,
                self.dressing2.with_part(part),
                self.decoration1,
                self.decoration2,
                self.color.clone(),
                self.is_self,
                self.thickness,
                self.reverse_define,
                self.inclination,
            )
        } else {
            Self::new(
                self.body,
                self.dressing1.with_part(part),
                self.dressing2,
                self.decoration1,
                self.decoration2,
                self.color.clone(),
                self.is_self,
                self.thickness,
                self.reverse_define,
                self.inclination,
            )
        }
    }

    /// Set decoration1.
    pub fn with_decoration1(&self, decoration: ArrowDecoration) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2,
            decoration,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set decoration2.
    pub fn with_decoration2(&self, decoration: ArrowDecoration) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2,
            self.decoration1,
            decoration,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set arrow color.
    pub fn with_color(&self, color: Option<String>) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2,
            self.decoration1,
            self.decoration2,
            color,
            self.is_self,
            self.thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Set line thickness.
    pub fn with_thickness(&self, thickness: f64) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2,
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            thickness,
            self.reverse_define,
            self.inclination,
        )
    }

    /// Convenience: set body to dotted.
    pub fn with_dotted(&self) -> Self {
        self.with_body(ArrowBody::Dotted)
    }

    /// Toggle reverse_define flag.
    /// Java: `ArrowConfiguration.reverseDefine()`
    pub fn reverse_define(&self) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2,
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            self.thickness,
            !self.reverse_define,
            self.inclination,
        )
    }

    /// Set inclination.
    pub fn with_inclination(&self, inclination: i32) -> Self {
        Self::new(
            self.body,
            self.dressing1,
            self.dressing2,
            self.decoration1,
            self.decoration2,
            self.color.clone(),
            self.is_self,
            self.thickness,
            self.reverse_define,
            inclination,
        )
    }

    // ---- queries ----

    /// Compute the arrow direction from dressing state.
    /// Java: `ArrowConfiguration.getArrowDirection()`
    pub fn arrow_direction(&self) -> ArrowDirection {
        if self.is_self {
            return ArrowDirection::Self_;
        }
        if self.dressing1.head == ArrowHead::None && self.dressing2.head != ArrowHead::None {
            return ArrowDirection::LeftToRight;
        }
        if self.dressing1.head != ArrowHead::None && self.dressing2.head == ArrowHead::None {
            return ArrowDirection::RightToLeft;
        }
        ArrowDirection::Both
    }

    pub fn is_self_arrow(&self) -> bool {
        self.arrow_direction() == ArrowDirection::Self_
    }

    pub fn is_dotted(&self) -> bool {
        self.body == ArrowBody::Dotted
    }

    pub fn is_hidden(&self) -> bool {
        self.body == ArrowBody::Hidden
    }

    /// Get the "active" head -- prefers dressing2 if it has a head.
    /// Java: `ArrowConfiguration.getHead()`
    pub fn head(&self) -> ArrowHead {
        if self.dressing2.head != ArrowHead::None {
            return self.dressing2.head;
        }
        self.dressing1.head
    }

    pub fn is_async1(&self) -> bool {
        self.dressing1.head == ArrowHead::Async
    }

    pub fn is_async2(&self) -> bool {
        self.dressing2.head == ArrowHead::Async
    }

    /// Get the part from the dressing that has the head.
    /// Java: `ArrowConfiguration.getPart()`
    pub fn part(&self) -> ArrowPart {
        if self.dressing2.head != ArrowHead::None {
            return self.dressing2.part;
        }
        self.dressing1.part
    }

    pub fn body(&self) -> ArrowBody {
        self.body
    }

    pub fn dressing1(&self) -> ArrowDressing {
        self.dressing1
    }

    pub fn dressing2(&self) -> ArrowDressing {
        self.dressing2
    }

    pub fn decoration1(&self) -> ArrowDecoration {
        self.decoration1
    }

    pub fn decoration2(&self) -> ArrowDecoration {
        self.decoration2
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn thickness(&self) -> f64 {
        self.thickness
    }

    pub fn is_reverse_define(&self) -> bool {
        self.reverse_define
    }

    /// Inclination for dressing1 side.
    /// Java: `ArrowConfiguration.getInclination1()`
    pub fn inclination1(&self) -> i32 {
        if self.dressing2.head == ArrowHead::None || self.dressing2.head == ArrowHead::CrossX {
            return self.inclination;
        }
        0
    }

    /// Inclination for dressing2 side.
    /// Java: `ArrowConfiguration.getInclination2()`
    pub fn inclination2(&self) -> i32 {
        if self.dressing1.head == ArrowHead::None || self.dressing1.head == ArrowHead::CrossX {
            return self.inclination;
        }
        // Java: also returns inclination when dressing1 head is NORMAL
        if self.dressing1.head == ArrowHead::Normal {
            return self.inclination;
        }
        0
    }

    /// Build a display name matching Java's `ArrowConfiguration.name()`.
    pub fn name(&self) -> String {
        format!(
            "{}({} {})({} {}){} {:?}",
            self.body,
            self.dressing1,
            self.decoration1,
            self.dressing2,
            self.decoration2,
            self.is_self,
            self.color,
        )
    }

    /// Compute stroke parameters: (dash_visible, dash_space, thickness).
    /// For dotted arrows returns (2, 2, thickness); for normal just (0, 0, thickness).
    pub fn stroke_params(&self) -> (f64, f64, f64) {
        if self.is_dotted() {
            (2.0, 2.0, self.thickness)
        } else {
            (0.0, 0.0, self.thickness)
        }
    }
}

/// Helper: only apply head to dressing if dressing already has a head.
/// Java: `ArrowConfiguration.addHead(ArrowDressing, ArrowHead)`
fn add_head(dressing: ArrowDressing, head: ArrowHead) -> ArrowDressing {
    if dressing.head == ArrowHead::None {
        return dressing;
    }
    dressing.with_head(head)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ArrowHead ----

    #[test]
    fn arrow_head_display() {
        assert_eq!(ArrowHead::Normal.to_string(), "NORMAL");
        assert_eq!(ArrowHead::Async.to_string(), "ASYNC");
        assert_eq!(ArrowHead::CrossX.to_string(), "CROSSX");
        assert_eq!(ArrowHead::None.to_string(), "NONE");
    }

    #[test]
    fn arrow_head_default() {
        assert_eq!(ArrowHead::default(), ArrowHead::Normal);
    }

    // ---- ArrowBody ----

    #[test]
    fn arrow_body_all_variants() {
        let _ = ArrowBody::Normal;
        let _ = ArrowBody::Dotted;
        let _ = ArrowBody::Dashed;
        let _ = ArrowBody::Hidden;
        let _ = ArrowBody::Bold;
    }

    #[test]
    fn arrow_body_display() {
        assert_eq!(ArrowBody::Bold.to_string(), "BOLD");
        assert_eq!(ArrowBody::Dashed.to_string(), "DASHED");
    }

    // ---- ArrowDirection ----

    #[test]
    fn arrow_direction_reverse() {
        assert_eq!(
            ArrowDirection::LeftToRight.reverse(),
            ArrowDirection::RightToLeft
        );
        assert_eq!(
            ArrowDirection::RightToLeft.reverse(),
            ArrowDirection::LeftToRight
        );
    }

    #[test]
    #[should_panic]
    fn arrow_direction_reverse_self_panics() {
        let _ = ArrowDirection::Self_.reverse();
    }

    #[test]
    #[should_panic]
    fn arrow_direction_reverse_both_panics() {
        let _ = ArrowDirection::Both.reverse();
    }

    // ---- ArrowDressing ----

    #[test]
    fn dressing_create_none() {
        let d = ArrowDressing::none();
        assert_eq!(d.head, ArrowHead::None);
        assert_eq!(d.part, ArrowPart::Full);
    }

    #[test]
    fn dressing_default_is_normal() {
        let d = ArrowDressing::default();
        assert_eq!(d.head, ArrowHead::Normal);
        assert_eq!(d.part, ArrowPart::Full);
    }

    #[test]
    fn dressing_with_head() {
        let d = ArrowDressing::none().with_head(ArrowHead::Async);
        assert_eq!(d.head, ArrowHead::Async);
        assert_eq!(d.part, ArrowPart::Full);
    }

    #[test]
    fn dressing_with_part() {
        let d = ArrowDressing::default().with_part(ArrowPart::TopPart);
        assert_eq!(d.head, ArrowHead::Normal);
        assert_eq!(d.part, ArrowPart::TopPart);
    }

    #[test]
    fn dressing_display() {
        let d = ArrowDressing::new(ArrowHead::CrossX);
        assert_eq!(d.to_string(), "CROSSX");
    }

    // ---- ArrowConfiguration factory methods ----

    #[test]
    fn direction_normal() {
        let a = ArrowConfiguration::with_direction_normal();
        assert_eq!(a.body(), ArrowBody::Normal);
        assert_eq!(a.dressing1().head, ArrowHead::None);
        assert_eq!(a.dressing2().head, ArrowHead::Normal);
        assert_eq!(a.arrow_direction(), ArrowDirection::LeftToRight);
        assert!(!a.is_self_arrow());
    }

    #[test]
    fn direction_reverse() {
        let a = ArrowConfiguration::with_direction_reverse();
        assert_eq!(a.dressing1().head, ArrowHead::Normal);
        assert_eq!(a.dressing2().head, ArrowHead::None);
        assert_eq!(a.arrow_direction(), ArrowDirection::RightToLeft);
    }

    #[test]
    fn direction_both() {
        let a = ArrowConfiguration::with_direction_both();
        assert_eq!(a.dressing1().head, ArrowHead::Normal);
        assert_eq!(a.dressing2().head, ArrowHead::Normal);
        assert_eq!(a.arrow_direction(), ArrowDirection::Both);
    }

    #[test]
    fn direction_self() {
        let a = ArrowConfiguration::with_direction_self(false);
        assert!(a.is_self_arrow());
        assert_eq!(a.arrow_direction(), ArrowDirection::Self_);
        assert!(!a.is_reverse_define());
    }

    #[test]
    fn direction_self_reverse_define() {
        let a = ArrowConfiguration::with_direction_self(true);
        assert!(a.is_reverse_define());
    }

    // ---- builder methods ----

    #[test]
    fn with_body() {
        let a = ArrowConfiguration::with_direction_normal().with_body(ArrowBody::Dotted);
        assert!(a.is_dotted());
        assert!(!a.is_hidden());
    }

    #[test]
    fn with_dotted() {
        let a = ArrowConfiguration::with_direction_normal().with_dotted();
        assert!(a.is_dotted());
    }

    #[test]
    fn with_hidden() {
        let a = ArrowConfiguration::with_direction_normal().with_body(ArrowBody::Hidden);
        assert!(a.is_hidden());
    }

    #[test]
    fn reverse_swaps_dressings_and_decorations() {
        let a =
            ArrowConfiguration::with_direction_normal().with_decoration1(ArrowDecoration::Circle);
        let b = a.reverse();
        assert_eq!(b.dressing1().head, ArrowHead::Normal);
        assert_eq!(b.dressing2().head, ArrowHead::None);
        assert_eq!(b.decoration1(), ArrowDecoration::None);
        assert_eq!(b.decoration2(), ArrowDecoration::Circle);
    }

    #[test]
    fn with_head_applies_to_non_none_dressings() {
        // Normal arrow: dressing1=None, dressing2=Normal
        let a = ArrowConfiguration::with_direction_normal().with_head(ArrowHead::Async);
        // head should be applied only to dressing2 (since dressing1 is None)
        assert_eq!(a.dressing1().head, ArrowHead::None);
        assert_eq!(a.dressing2().head, ArrowHead::Async);
    }

    #[test]
    fn with_head_applies_to_both_when_both_have_heads() {
        let a = ArrowConfiguration::with_direction_both().with_head(ArrowHead::CrossX);
        assert_eq!(a.dressing1().head, ArrowHead::CrossX);
        assert_eq!(a.dressing2().head, ArrowHead::CrossX);
    }

    #[test]
    fn with_head1() {
        let a = ArrowConfiguration::with_direction_normal().with_head1(ArrowHead::Async);
        assert_eq!(a.dressing1().head, ArrowHead::Async);
        assert_eq!(a.dressing2().head, ArrowHead::Normal);
    }

    #[test]
    fn with_head2() {
        let a = ArrowConfiguration::with_direction_normal().with_head2(ArrowHead::CrossX);
        assert_eq!(a.dressing1().head, ArrowHead::None);
        assert_eq!(a.dressing2().head, ArrowHead::CrossX);
    }

    #[test]
    fn with_part_applies_to_headed_dressing() {
        // dressing2 has the head in a normal arrow
        let a = ArrowConfiguration::with_direction_normal().with_part(ArrowPart::TopPart);
        assert_eq!(a.dressing2().part, ArrowPart::TopPart);
        assert_eq!(a.dressing1().part, ArrowPart::Full);
        assert_eq!(a.part(), ArrowPart::TopPart);
    }

    #[test]
    fn with_part_falls_back_to_dressing1_when_dressing2_none() {
        // Reverse arrow: dressing1 has head, dressing2 is None
        let a = ArrowConfiguration::with_direction_reverse().with_part(ArrowPart::BottomPart);
        assert_eq!(a.dressing1().part, ArrowPart::BottomPart);
        assert_eq!(a.dressing2().part, ArrowPart::Full);
        assert_eq!(a.part(), ArrowPart::BottomPart);
    }

    #[test]
    fn with_color() {
        let a = ArrowConfiguration::with_direction_normal().with_color(Some("red".to_string()));
        assert_eq!(a.color(), Some("red"));
    }

    #[test]
    fn with_thickness() {
        let a = ArrowConfiguration::with_direction_normal().with_thickness(2.5);
        assert!((a.thickness() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn with_inclination() {
        let a = ArrowConfiguration::with_direction_normal().with_inclination(10);
        assert_eq!(a.inclination1(), 0);
        assert_eq!(a.inclination2(), 10);
    }

    #[test]
    fn inclination_for_reverse() {
        let a = ArrowConfiguration::with_direction_reverse().with_inclination(5);
        assert_eq!(a.inclination1(), 5);
        // dressing1 has Normal head
        assert_eq!(a.inclination2(), 5);
    }

    // ---- head() / part() queries ----

    #[test]
    fn head_prefers_dressing2() {
        let a = ArrowConfiguration::with_direction_both();
        assert_eq!(a.head(), ArrowHead::Normal);
    }

    #[test]
    fn head_falls_back_to_dressing1() {
        let a = ArrowConfiguration::with_direction_reverse();
        assert_eq!(a.head(), ArrowHead::Normal);
    }

    // ---- async queries ----

    #[test]
    fn is_async() {
        let a = ArrowConfiguration::with_direction_normal()
            .with_head1(ArrowHead::Async)
            .with_head2(ArrowHead::Async);
        assert!(a.is_async1());
        assert!(a.is_async2());
    }

    // ---- self_arrow ----

    #[test]
    fn self_arrow_marks_is_self() {
        let a = ArrowConfiguration::with_direction_normal().self_arrow();
        assert!(a.is_self_arrow());
    }

    // ---- reverse_define ----

    #[test]
    fn toggle_reverse_define() {
        let a = ArrowConfiguration::with_direction_normal();
        assert!(!a.is_reverse_define());
        let b = a.reverse_define();
        assert!(b.is_reverse_define());
        let c = b.reverse_define();
        assert!(!c.is_reverse_define());
    }

    // ---- stroke_params ----

    #[test]
    fn stroke_params_normal() {
        let a = ArrowConfiguration::with_direction_normal();
        let (dv, ds, th) = a.stroke_params();
        assert!((dv - 0.0).abs() < f64::EPSILON);
        assert!((ds - 0.0).abs() < f64::EPSILON);
        assert!((th - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stroke_params_dotted() {
        let a = ArrowConfiguration::with_direction_normal()
            .with_dotted()
            .with_thickness(3.0);
        let (dv, ds, th) = a.stroke_params();
        assert!((dv - 2.0).abs() < f64::EPSILON);
        assert!((ds - 2.0).abs() < f64::EPSILON);
        assert!((th - 3.0).abs() < f64::EPSILON);
    }

    // ---- name() / display ----

    #[test]
    fn name_format() {
        let a = ArrowConfiguration::with_direction_normal();
        let name = a.name();
        assert!(name.contains("NORMAL"));
        assert!(name.contains("NONE"));
    }

    #[test]
    fn display_equals_name() {
        let a = ArrowConfiguration::with_direction_normal();
        assert_eq!(a.to_string(), a.name());
    }

    // ---- decoration ----

    #[test]
    fn decorations() {
        let a = ArrowConfiguration::with_direction_normal()
            .with_decoration1(ArrowDecoration::Circle)
            .with_decoration2(ArrowDecoration::Circle);
        assert_eq!(a.decoration1(), ArrowDecoration::Circle);
        assert_eq!(a.decoration2(), ArrowDecoration::Circle);
    }

    // ---- ArrowDecoration display ----

    #[test]
    fn decoration_display() {
        assert_eq!(ArrowDecoration::None.to_string(), "NONE");
        assert_eq!(ArrowDecoration::Circle.to_string(), "CIRCLE");
    }

    // ---- default thickness ----

    #[test]
    fn default_thickness_is_one() {
        let a = ArrowConfiguration::with_direction_normal();
        assert!((a.thickness() - 1.0).abs() < f64::EPSILON);
    }
}
