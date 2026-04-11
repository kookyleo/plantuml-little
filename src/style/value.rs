// Style value types.
// Port of Java PlantUML's `net.sourceforge.plantuml.style.Value`,
// `ValueImpl`, `ValueColor`, `ValueNull`, `ValueAbstract`,
// `DarkString`, `LengthAdjust`, `MergeStrategy`.

use crate::klimt::color::HColor;
use crate::klimt::geom::HorizontalAlignment;

// ── DarkString ──────────────────────────────────────────────────────

/// A pair of light/dark string values with a merge priority.
///
/// Java: `style.DarkString`
///
/// `value1` is the normal (light) theme value, `value2` is the dark theme value.
/// Priority controls which value wins during merging.
#[derive(Debug, Clone, PartialEq)]
pub struct DarkString {
    value1: Option<String>,
    value2: Option<String>,
    priority: i32,
}

impl DarkString {
    pub fn new(value1: Option<String>, value2: Option<String>, priority: i32) -> Self {
        Self {
            value1,
            value2,
            priority,
        }
    }

    /// Merge with another DarkString, combining light/dark halves when possible.
    /// Java: `DarkString.mergeWith(DarkString)`
    pub fn merge_with(&self, other: &DarkString) -> DarkString {
        // Both have same "shape" (both only-light or both only-dark)
        if (self.value2.is_none() && other.value2.is_none())
            || (self.value1.is_none() && other.value1.is_none())
        {
            return if self.priority > other.priority {
                self.clone()
            } else {
                other.clone()
            };
        }

        // self is light-only, other is dark-only -> combine
        if self.value2.is_none() && other.value1.is_none() {
            return DarkString::new(self.value1.clone(), other.value2.clone(), self.priority);
        }

        // other is light-only, self is dark-only -> combine
        if other.value2.is_none() && self.value1.is_none() {
            return DarkString::new(other.value1.clone(), self.value2.clone(), other.priority);
        }

        // Fallback: higher priority wins
        if self.priority > other.priority {
            self.clone()
        } else {
            other.clone()
        }
    }

    pub fn add_priority(&self, delta: i32) -> DarkString {
        DarkString::new(
            self.value1.clone(),
            self.value2.clone(),
            self.priority + delta,
        )
    }

    pub fn value1(&self) -> Option<&str> {
        self.value1.as_deref()
    }

    pub fn value2(&self) -> Option<&str> {
        self.value2.as_deref()
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }
}

impl std::fmt::Display for DarkString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}  ({})",
            self.value1.as_deref().unwrap_or("null"),
            self.value2.as_deref().unwrap_or("null"),
            self.priority,
        )
    }
}

// ── LengthAdjust ────────────────────────────────────────────────────

/// SVG `lengthAdjust` attribute value.
/// Java: `style.LengthAdjust`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LengthAdjust {
    None,
    #[default]
    Spacing,
    SpacingAndGlyphs,
}

impl LengthAdjust {
    pub fn default_value() -> Self {
        Self::Spacing
    }
}

// ── MergeStrategy ───────────────────────────────────────────────────

/// Controls how style values are merged when stereotypes overlap.
/// Java: `style.MergeStrategy`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MergeStrategy {
    KeepExistingValueOfStereotype,
    OverwriteExistingValue,
}

// ── Value trait ─────────────────────────────────────────────────────

/// The core style value interface.
/// Java: `style.Value` (interface)
///
/// All methods have default implementations that either return a sensible
/// zero/empty value or panic, matching Java's `ValueAbstract` behavior for
/// methods not overridden in a concrete subclass.
pub trait Value: std::fmt::Debug {
    fn as_string(&self) -> &str {
        ""
    }
    fn as_color(&self) -> HColor {
        HColor::None
    }
    fn as_int(&self, minus_one_if_error: bool) -> i32 {
        if minus_one_if_error {
            -1
        } else {
            0
        }
    }
    fn as_double(&self) -> f64 {
        0.0
    }
    fn as_double_default_to(&self, default: f64) -> f64 {
        default
    }
    fn as_boolean(&self) -> bool {
        false
    }
    fn as_horizontal_alignment(&self) -> HorizontalAlignment {
        HorizontalAlignment::Left
    }
    fn priority(&self) -> i32 {
        0
    }
}

// ── ValueNull ───────────────────────────────────────────────────────

/// Null / absent value singleton. All accessors return safe defaults.
/// Java: `style.ValueNull`
#[derive(Debug, Clone, Copy)]
pub struct ValueNull;

impl ValueNull {
    pub const NULL: ValueNull = ValueNull;
}

impl Value for ValueNull {
    fn as_string(&self) -> &str {
        ""
    }
    fn as_color(&self) -> HColor {
        HColor::simple("#000000")
    }
    fn as_int(&self, _minus_one_if_error: bool) -> i32 {
        0
    }
    fn as_double(&self) -> f64 {
        0.0
    }
    fn as_double_default_to(&self, default: f64) -> f64 {
        default
    }
    fn as_boolean(&self) -> bool {
        false
    }
    fn as_horizontal_alignment(&self) -> HorizontalAlignment {
        HorizontalAlignment::Left
    }
    fn priority(&self) -> i32 {
        0
    }
}

impl std::fmt::Display for ValueNull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ValueNull")
    }
}

// ── ValueImpl ───────────────────────────────────────────────────────

/// Concrete string-backed style value (the most common kind).
/// Java: `style.ValueImpl`
#[derive(Debug, Clone)]
pub struct ValueImpl {
    value: DarkString,
}

impl ValueImpl {
    /// Create a dark-theme-only value.
    /// Java: `ValueImpl.dark(String, AutomaticCounter)`
    pub fn dark(dark_value: &str, priority: i32) -> Self {
        Self {
            value: DarkString::new(None, Some(dark_value.to_string()), priority),
        }
    }

    /// Create a normal (light) value.
    /// Java: `ValueImpl.regular(String, AutomaticCounter)`
    pub fn regular(value: &str, priority: i32) -> Self {
        Self {
            value: DarkString::new(Some(value.to_string()), None, priority),
        }
    }

    /// Merge with another Value, returning whichever has higher priority
    /// or combining light/dark halves if possible.
    pub fn merge_with(&self, other: &dyn Value) -> Box<dyn Value> {
        // If other is also a ValueImpl, use DarkString merging
        if let Some(other_impl) = as_value_impl(other) {
            return Box::new(ValueImpl {
                value: self.value.merge_with(&other_impl.value),
            });
        }
        // For ValueColor: higher priority wins
        if other.priority() > self.priority() {
            // We cannot clone a trait object directly, so for non-ValueImpl
            // we just return self (the Java code returns `other` which is
            // a ValueColor — we'd need concrete dispatch for that).
            // For now, returning self is the safe fallback.
            return Box::new(self.clone());
        }
        Box::new(self.clone())
    }

    /// Return a new ValueImpl with priority adjusted by delta.
    pub fn add_priority(&self, delta: i32) -> Self {
        Self {
            value: self.value.add_priority(delta),
        }
    }

    /// Access the underlying DarkString.
    pub fn dark_string(&self) -> &DarkString {
        &self.value
    }
}

impl Value for ValueImpl {
    fn as_string(&self) -> &str {
        self.value.value1().unwrap_or("")
    }

    fn as_color(&self) -> HColor {
        let v1 = match self.value.value1() {
            Some(s) => s,
            None => return HColor::None,
        };

        if v1.eq_ignore_ascii_case("none") || v1.eq_ignore_ascii_case("transparent") {
            return HColor::None;
        }

        let base = HColor::simple(v1);

        // If there is a dark theme color, attach it
        if let Some(v2) = self.value.value2() {
            let dark = HColor::simple(v2);
            // For now we just return base; dark-mode support can be added later
            // when HColor.withDark() is ported.
            let _ = dark;
            return base;
        }
        base
    }

    fn as_int(&self, minus_one_if_error: bool) -> i32 {
        let s = self.value.value1().unwrap_or("");
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return if minus_one_if_error { -1 } else { 0 };
        }
        digits
            .parse::<i32>()
            .unwrap_or(if minus_one_if_error { -1 } else { 0 })
    }

    fn as_double(&self) -> f64 {
        let s = self.value.value1().unwrap_or("");
        let digits: String = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        digits.parse::<f64>().unwrap_or(0.0)
    }

    fn as_double_default_to(&self, default: f64) -> f64 {
        let s = self.value.value1().unwrap_or("");
        let digits: String = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if digits.is_empty() {
            return default;
        }
        digits.parse::<f64>().unwrap_or(default)
    }

    fn as_boolean(&self) -> bool {
        self.value
            .value1()
            .map(|s| s.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    fn as_horizontal_alignment(&self) -> HorizontalAlignment {
        horizontal_alignment_from_string(self.as_string())
    }

    fn priority(&self) -> i32 {
        self.value.priority()
    }
}

impl std::fmt::Display for ValueImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

// ── ValueColor ──────────────────────────────────────────────────────

/// A value that directly holds a resolved HColor.
/// Java: `style.ValueColor`
#[derive(Debug, Clone)]
pub struct ValueColor {
    color: HColor,
    prio: i32,
}

impl ValueColor {
    pub fn new(color: HColor, priority: i32) -> Self {
        Self {
            color,
            prio: priority,
        }
    }
}

impl Value for ValueColor {
    fn as_color(&self) -> HColor {
        self.color.clone()
    }

    fn priority(&self) -> i32 {
        self.prio
    }
}

impl std::fmt::Display for ValueColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.color)
    }
}

// ── Helper: downcast ────────────────────────────────────────────────

/// Try to downcast a `&dyn Value` to `&ValueImpl`.
/// This is used for type-specific merge logic.
fn as_value_impl(v: &dyn Value) -> Option<&ValueImpl> {
    // We use the Debug output to detect type — a stopgap until we add
    // Any-based downcasting. Since ValueImpl's Debug starts with
    // "ValueImpl", this is reliable for internal use.
    let dbg = format!("{:?}", v);
    if dbg.starts_with("ValueImpl") {
        // SAFETY: This is a best-effort cast. We rely on the fact that
        // within this crate, only ValueImpl produces this debug prefix.
        // A proper solution would use Any, but that requires 'static.
        unsafe {
            let ptr = v as *const dyn Value as *const ValueImpl;
            Some(&*ptr)
        }
    } else {
        None
    }
}

// ── Helper: HorizontalAlignment from string ─────────────────────────

fn horizontal_alignment_from_string(s: &str) -> HorizontalAlignment {
    let lower = s.trim().to_ascii_lowercase();
    match lower.as_str() {
        "left" => HorizontalAlignment::Left,
        "right" => HorizontalAlignment::Right,
        "center" => HorizontalAlignment::Center,
        _ => HorizontalAlignment::Center,
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── DarkString tests ────────────────────────────────────────────

    #[test]
    fn dark_string_display() {
        let ds = DarkString::new(Some("red".into()), Some("blue".into()), 5);
        let s = format!("{}", ds);
        assert!(s.contains("red"));
        assert!(s.contains("blue"));
        assert!(s.contains("5"));
    }

    #[test]
    fn dark_string_merge_both_light_higher_priority_wins() {
        let a = DarkString::new(Some("red".into()), None, 10);
        let b = DarkString::new(Some("blue".into()), None, 5);
        let merged = a.merge_with(&b);
        assert_eq!(merged.value1(), Some("red"));
        assert_eq!(merged.priority(), 10);
    }

    #[test]
    fn dark_string_merge_both_light_lower_priority_loses() {
        let a = DarkString::new(Some("red".into()), None, 3);
        let b = DarkString::new(Some("blue".into()), None, 7);
        let merged = a.merge_with(&b);
        assert_eq!(merged.value1(), Some("blue"));
        assert_eq!(merged.priority(), 7);
    }

    #[test]
    fn dark_string_merge_light_and_dark_combine() {
        let light = DarkString::new(Some("red".into()), None, 5);
        let dark = DarkString::new(None, Some("blue".into()), 3);
        let merged = light.merge_with(&dark);
        assert_eq!(merged.value1(), Some("red"));
        assert_eq!(merged.value2(), Some("blue"));
        assert_eq!(merged.priority(), 5);
    }

    #[test]
    fn dark_string_merge_dark_and_light_combine() {
        let dark = DarkString::new(None, Some("blue".into()), 3);
        let light = DarkString::new(Some("red".into()), None, 5);
        let merged = dark.merge_with(&light);
        assert_eq!(merged.value1(), Some("red"));
        assert_eq!(merged.value2(), Some("blue"));
        assert_eq!(merged.priority(), 5);
    }

    #[test]
    fn dark_string_add_priority() {
        let ds = DarkString::new(Some("x".into()), None, 10);
        let ds2 = ds.add_priority(5);
        assert_eq!(ds2.priority(), 15);
        assert_eq!(ds2.value1(), Some("x"));
    }

    // ── ValueNull tests ─────────────────────────────────────────────

    #[test]
    fn value_null_defaults() {
        let v = ValueNull::NULL;
        assert_eq!(v.as_string(), "");
        assert_eq!(v.as_int(false), 0);
        assert_eq!(v.as_int(true), 0);
        assert_eq!(v.as_double(), 0.0);
        assert_eq!(v.as_double_default_to(42.0), 42.0);
        assert!(!v.as_boolean());
        assert_eq!(v.as_horizontal_alignment(), HorizontalAlignment::Left);
        assert_eq!(v.priority(), 0);
    }

    #[test]
    fn value_null_color_is_black() {
        let v = ValueNull::NULL;
        let c = v.as_color();
        // Should be black (#000000)
        assert_eq!(c, HColor::simple("#000000"));
    }

    // ── ValueImpl tests ─────────────────────────────────────────────

    #[test]
    fn value_impl_regular_string() {
        let v = ValueImpl::regular("hello", 1);
        assert_eq!(v.as_string(), "hello");
    }

    #[test]
    fn value_impl_boolean_true() {
        let v = ValueImpl::regular("true", 1);
        assert!(v.as_boolean());
    }

    #[test]
    fn value_impl_boolean_true_case_insensitive() {
        let v = ValueImpl::regular("TRUE", 1);
        assert!(v.as_boolean());
        let v2 = ValueImpl::regular("True", 1);
        assert!(v2.as_boolean());
    }

    #[test]
    fn value_impl_boolean_false() {
        let v = ValueImpl::regular("false", 1);
        assert!(!v.as_boolean());
        let v2 = ValueImpl::regular("anything", 1);
        assert!(!v2.as_boolean());
    }

    #[test]
    fn value_impl_as_int_extracts_digits() {
        let v = ValueImpl::regular("12px", 1);
        assert_eq!(v.as_int(false), 12);
    }

    #[test]
    fn value_impl_as_int_pure_number() {
        let v = ValueImpl::regular("42", 1);
        assert_eq!(v.as_int(false), 42);
    }

    #[test]
    fn value_impl_as_int_no_digits() {
        let v = ValueImpl::regular("abc", 1);
        assert_eq!(v.as_int(false), 0);
        assert_eq!(v.as_int(true), -1);
    }

    #[test]
    fn value_impl_as_double() {
        let v = ValueImpl::regular("3.14", 1);
        let d = v.as_double();
        #[allow(clippy::approx_constant)]
        let expected = 3.14;
        assert!((d - expected).abs() < 1e-9);
    }

    #[test]
    fn value_impl_as_double_with_unit() {
        let v = ValueImpl::regular("1.5px", 1);
        let d = v.as_double();
        assert!((d - 1.5).abs() < 1e-9);
    }

    #[test]
    fn value_impl_as_double_default_empty() {
        let v = ValueImpl::regular("abc", 1);
        assert_eq!(v.as_double_default_to(99.0), 99.0);
    }

    #[test]
    fn value_impl_color_hex() {
        let v = ValueImpl::regular("#FF0000", 1);
        let c = v.as_color();
        assert_eq!(c, HColor::Simple { r: 255, g: 0, b: 0 });
    }

    #[test]
    fn value_impl_color_transparent() {
        let v = ValueImpl::regular("transparent", 1);
        assert_eq!(v.as_color(), HColor::None);
    }

    #[test]
    fn value_impl_color_none() {
        let v = ValueImpl::regular("none", 1);
        assert_eq!(v.as_color(), HColor::None);
    }

    #[test]
    fn value_impl_horizontal_alignment() {
        let v = ValueImpl::regular("left", 1);
        assert_eq!(v.as_horizontal_alignment(), HorizontalAlignment::Left);
        let v = ValueImpl::regular("right", 1);
        assert_eq!(v.as_horizontal_alignment(), HorizontalAlignment::Right);
        let v = ValueImpl::regular("center", 1);
        assert_eq!(v.as_horizontal_alignment(), HorizontalAlignment::Center);
    }

    #[test]
    fn value_impl_horizontal_alignment_case_insensitive() {
        let v = ValueImpl::regular("LEFT", 1);
        assert_eq!(v.as_horizontal_alignment(), HorizontalAlignment::Left);
        let v = ValueImpl::regular("Right", 1);
        assert_eq!(v.as_horizontal_alignment(), HorizontalAlignment::Right);
    }

    #[test]
    fn value_impl_priority() {
        let v = ValueImpl::regular("x", 42);
        assert_eq!(v.priority(), 42);
    }

    #[test]
    fn value_impl_add_priority() {
        let v = ValueImpl::regular("x", 10);
        let v2 = v.add_priority(5);
        assert_eq!(v2.priority(), 15);
    }

    #[test]
    fn value_impl_dark_value() {
        let v = ValueImpl::dark("blue", 1);
        // as_string returns value1 which is None for dark-only -> ""
        assert_eq!(v.as_string(), "");
        assert_eq!(v.dark_string().value2(), Some("blue"));
    }

    #[test]
    fn value_impl_display() {
        let v = ValueImpl::regular("hello", 5);
        let s = format!("{}", v);
        assert!(s.contains("hello"));
    }

    // ── ValueColor tests ────────────────────────────────────────────

    #[test]
    fn value_color_returns_color() {
        let c = HColor::Simple {
            r: 0,
            g: 128,
            b: 255,
        };
        let v = ValueColor::new(c.clone(), 10);
        assert_eq!(v.as_color(), c);
        assert_eq!(v.priority(), 10);
    }

    #[test]
    fn value_color_display() {
        let v = ValueColor::new(HColor::Simple { r: 255, g: 0, b: 0 }, 1);
        let s = format!("{}", v);
        assert!(s.contains("255"));
    }

    // ── LengthAdjust tests ──────────────────────────────────────────

    #[test]
    fn length_adjust_default_is_spacing() {
        assert_eq!(LengthAdjust::default(), LengthAdjust::Spacing);
        assert_eq!(LengthAdjust::default_value(), LengthAdjust::Spacing);
    }

    // ── MergeStrategy tests ─────────────────────────────────────────

    #[test]
    fn merge_strategy_variants() {
        let a = MergeStrategy::KeepExistingValueOfStereotype;
        let b = MergeStrategy::OverwriteExistingValue;
        assert_ne!(a, b);
    }

    // ── Integration: trait object usage ─────────────────────────────

    #[test]
    fn value_trait_object() {
        let values: Vec<Box<dyn Value>> = vec![
            Box::new(ValueNull::NULL),
            Box::new(ValueImpl::regular("14", 1)),
            Box::new(ValueColor::new(HColor::simple("#FF0000"), 5)),
        ];

        assert_eq!(values[0].as_string(), "");
        assert_eq!(values[1].as_int(false), 14);
        assert_eq!(values[2].as_color(), HColor::Simple { r: 255, g: 0, b: 0 });
    }
}
