// Style, StyleBuilder, StyleStorage, StyleLoader — the core style engine.
// Port of Java PlantUML's style.Style, StyleBuilder, StyleStorage, StyleLoader

use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};

use log::info;

use super::pname::PName;
use super::signature::{StyleKey, StyleSignatureBasic};
use super::value::{MergeStrategy, Value, ValueImpl};
use crate::klimt::geom::HorizontalAlignment;
use crate::klimt::{LineBreakStrategy, UStroke};

// ── ClockwiseTopRightBottomLeft ──────────────────────────────────────

/// Padding / margin specification: four values in CSS clockwise order
/// (top, right, bottom, left).
///
/// Java: `style.ClockwiseTopRightBottomLeft`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockwiseTopRightBottomLeft {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl ClockwiseTopRightBottomLeft {
    pub fn none() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub fn same(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn margin1_margin2(m1: f64, m2: f64) -> Self {
        Self {
            top: m1,
            right: m2,
            bottom: m1,
            left: m2,
        }
    }

    pub fn inc_top(&self, delta: f64) -> Self {
        Self {
            top: self.top + delta,
            ..*self
        }
    }

    /// Parse a CSS-style padding/margin string.
    /// Java: `ClockwiseTopRightBottomLeft.read(String)`
    ///
    /// Supports 1-4 space-separated integer values:
    /// - 1 value: same on all sides
    /// - 2 values: (top+bottom, left+right)
    /// - 3 values: (top, left+right, bottom)
    /// - 4 values: (top, right, bottom, left)
    pub fn read(value: &str) -> Self {
        // Only allow digits and spaces
        if !value.chars().all(|c| c.is_ascii_digit() || c == ' ') {
            return Self::none();
        }

        let parts: Vec<&str> = value.split_whitespace().collect();
        match parts.len() {
            1 => {
                if let Ok(v) = parts[0].parse::<f64>() {
                    Self::same(v)
                } else {
                    Self::none()
                }
            }
            2 => {
                let a = parts[0].parse::<f64>().unwrap_or(0.0);
                let b = parts[1].parse::<f64>().unwrap_or(0.0);
                Self::new(a, b, a, b)
            }
            3 => {
                let a = parts[0].parse::<f64>().unwrap_or(0.0);
                let b = parts[1].parse::<f64>().unwrap_or(0.0);
                let c = parts[2].parse::<f64>().unwrap_or(0.0);
                Self::new(a, b, c, b)
            }
            4 => {
                let a = parts[0].parse::<f64>().unwrap_or(0.0);
                let b = parts[1].parse::<f64>().unwrap_or(0.0);
                let c = parts[2].parse::<f64>().unwrap_or(0.0);
                let d = parts[3].parse::<f64>().unwrap_or(0.0);
                Self::new(a, b, c, d)
            }
            _ => Self::none(),
        }
    }
}

impl Default for ClockwiseTopRightBottomLeft {
    fn default() -> Self {
        Self::none()
    }
}

impl std::fmt::Display for ClockwiseTopRightBottomLeft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.top, self.right, self.bottom, self.left
        )
    }
}

// ── Style ────────────────────────────────────────────────────────────

/// A resolved style rule: a signature (selector) plus a property map.
///
/// Properties are stored as `PName -> ValueImpl` pairs. `ValueImpl` is
/// the most common value type (string-backed with priority); direct
/// color overrides are handled by storing hex strings.
///
/// Java: `style.Style`
#[derive(Debug, Clone)]
pub struct Style {
    signature: StyleSignatureBasic,
    map: HashMap<PName, ValueImpl>,
}

impl Style {
    pub fn new(signature: StyleSignatureBasic, map: HashMap<PName, ValueImpl>) -> Self {
        Self { signature, map }
    }

    /// Convenience: create an empty style with the given signature.
    pub fn empty(signature: StyleSignatureBasic) -> Self {
        Self {
            signature,
            map: HashMap::new(),
        }
    }

    /// Look up a property value.
    /// Java: `Style.value(PName)` — returns `ValueNull.NULL` for absent keys.
    pub fn value(&self, name: PName) -> &dyn Value {
        match self.map.get(&name) {
            Some(vi) => vi,
            None => &super::value::ValueNull::NULL,
        }
    }

    /// Check whether a property is present.
    pub fn has_value(&self, name: PName) -> bool {
        self.map.contains_key(&name)
    }

    /// Get the style's selector signature.
    pub fn signature(&self) -> &StyleSignatureBasic {
        &self.signature
    }

    // ── Property accessors ──────────────────────────────────────────

    /// Shadowing amount. Java: `Style.getShadowing()`
    pub fn shadowing(&self) -> f64 {
        match self.map.get(&PName::Shadowing) {
            None => 0.0,
            Some(_) => self.value(PName::Shadowing).as_double_default_to(1.5),
        }
    }

    /// Get the UStroke for line rendering.
    /// Java: `Style.getStroke()`
    pub fn stroke(&self) -> UStroke {
        self.stroke_from(PName::LineThickness, PName::LineStyle)
    }

    /// Get a UStroke from specific property names.
    fn stroke_from(&self, thickness_param: PName, style_param: PName) -> UStroke {
        let thickness = self.value(thickness_param).as_double();
        let dash = self.value(style_param).as_string();
        if dash.is_empty() {
            return UStroke::with_thickness(thickness);
        }
        // Parse "visible-space" or "visible;space" dash pattern
        let parts: Vec<&str> = dash.split(&['-', ';', ','][..]).collect();
        if let Some(first) = parts.first() {
            if let Ok(dash_visible) = first.trim().parse::<f64>() {
                let dash_space = parts
                    .get(1)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .unwrap_or(dash_visible);
                return UStroke::new(dash_visible, dash_space, thickness);
            }
        }
        UStroke::with_thickness(thickness)
    }

    /// Line break / text wrapping strategy.
    /// Java: `Style.wrapWidth()`
    pub fn wrap_width(&self) -> LineBreakStrategy {
        let val = self.value(PName::MaximumWidth).as_string();
        LineBreakStrategy::from_value(if val.is_empty() { None } else { Some(val) })
    }

    /// Get padding as ClockwiseTopRightBottomLeft.
    pub fn padding(&self) -> ClockwiseTopRightBottomLeft {
        ClockwiseTopRightBottomLeft::read(self.value(PName::Padding).as_string())
    }

    /// Get margin as ClockwiseTopRightBottomLeft.
    pub fn margin(&self) -> ClockwiseTopRightBottomLeft {
        ClockwiseTopRightBottomLeft::read(self.value(PName::Margin).as_string())
    }

    /// Get horizontal alignment.
    pub fn horizontal_alignment(&self) -> HorizontalAlignment {
        self.value(PName::HorizontalAlignment)
            .as_horizontal_alignment()
    }

    /// Get font name.
    pub fn font_name(&self) -> &str {
        self.value(PName::FontName).as_string()
    }

    /// Get font size (defaults to 14 if missing).
    pub fn font_size(&self) -> i32 {
        let size = self.value(PName::FontSize).as_int(true);
        if size == -1 {
            14
        } else {
            size
        }
    }

    /// Get font color.
    pub fn font_color(&self) -> crate::klimt::color::HColor {
        self.value(PName::FontColor).as_color()
    }

    /// Get background color.
    pub fn background_color(&self) -> crate::klimt::color::HColor {
        self.value(PName::BackGroundColor).as_color()
    }

    /// Get line color.
    pub fn line_color(&self) -> crate::klimt::color::HColor {
        self.value(PName::LineColor).as_color()
    }

    /// Get round corner radius.
    pub fn round_corner(&self) -> f64 {
        self.value(PName::RoundCorner).as_double()
    }

    /// Get diagonal corner radius.
    pub fn diagonal_corner(&self) -> f64 {
        self.value(PName::DiagonalCorner).as_double()
    }

    // ── Merging ─────────────────────────────────────────────────────

    /// Merge this style with another.
    /// Java: `Style.mergeWith(Style, MergeStrategy)`
    pub fn merge_with(&self, other: &Style, strategy: MergeStrategy) -> Style {
        let mut both = self.map.clone();
        for (key, other_val) in &other.map {
            if let Some(previous) = self.map.get(key) {
                if previous.priority() > DELTA_PRIORITY_FOR_STEREOTYPE
                    && strategy == MergeStrategy::KeepExistingValueOfStereotype
                {
                    continue;
                }
                // Merge using DarkString logic
                let merged_ds = other_val.dark_string().merge_with(previous.dark_string());
                both.insert(
                    *key,
                    ValueImpl::regular(merged_ds.value1().unwrap_or(""), merged_ds.priority()),
                );
            } else {
                both.insert(*key, other_val.clone());
            }
        }
        Style {
            signature: self.signature.merge_with(other.signature()),
            map: both,
        }
    }

    /// Shift all value priorities by a delta.
    /// Java: `Style.deltaPriority(int)`
    pub fn delta_priority(&self, delta: i32) -> Style {
        let new_map: HashMap<PName, ValueImpl> = self
            .map
            .iter()
            .map(|(k, v)| (*k, v.add_priority(delta)))
            .collect();
        Style {
            signature: self.signature.clone(),
            map: new_map,
        }
    }

    /// Override a property with a color (stored as hex string).
    /// Java: `Style.eventuallyOverride(PName, HColor)`
    pub fn override_color(&self, param: PName, color: &crate::klimt::color::HColor) -> Style {
        let mut result = self.map.clone();
        let old_priority = result.get(&param).map(|v| v.priority()).unwrap_or(0);
        result.insert(param, ValueImpl::regular(&color.to_svg(), old_priority));
        Style {
            signature: self.signature.clone(),
            map: result,
        }
    }

    /// Override a property with a string value.
    /// Java: `Style.eventuallyOverride(PName, String)`
    pub fn override_value(&self, param: PName, value: &str) -> Style {
        let mut result = self.map.clone();
        result.insert(param, ValueImpl::regular(value, i32::MAX));
        Style {
            signature: self.signature.clone(),
            map: result,
        }
    }

    /// Override a property with a numeric value.
    /// Java: `Style.eventuallyOverride(PName, double)`
    pub fn override_double(&self, param: PName, value: f64) -> Style {
        self.override_value(param, &value.to_string())
    }

    /// Override with a UStroke.
    /// Java: `Style.eventuallyOverride(UStroke)`
    pub fn override_stroke(&self, stroke: &UStroke) -> Style {
        let mut result = self.override_double(PName::LineThickness, stroke.thickness);
        result = result.override_value(
            PName::LineStyle,
            &format!("{}-{}", stroke.dash_visible, stroke.dash_space),
        );
        result
    }

    /// Access the raw property map (for debugging / iteration).
    pub fn properties(&self) -> &HashMap<PName, ValueImpl> {
        &self.map
    }

    /// Print for debugging. Java: `Style.printMe()`
    pub fn print_me(&self) {
        if self.map.is_empty() {
            return;
        }
        eprintln!("{} {{", self.signature);
        for (k, v) in &self.map {
            eprintln!("  {}: {}", k, v);
        }
        eprintln!("}}");
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:?}",
            self.signature,
            self.map.keys().collect::<Vec<_>>()
        )
    }
}

// ── StyleStorage ─────────────────────────────────────────────────────

/// Storage for style rules, split into two maps:
/// - `plain`: rules without stereotypes (keyed by StyleKey for fast lookup)
/// - `legacy`: rules with stereotypes (keyed by full StyleSignatureBasic)
///
/// Java: `style.StyleStorage`
#[derive(Debug, Clone, Default)]
pub struct StyleStorage {
    legacy: HashMap<StyleSignatureBasic, Style>,
    plain: HashMap<StyleKey, Style>,
}

impl StyleStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a style by its signature.
    /// Java: `StyleStorage.get(StyleSignatureBasic)`
    pub fn get(&self, signature: &StyleSignatureBasic) -> Option<&Style> {
        if signature.get_stereotypes().is_empty() {
            self.plain.get(signature.get_key())
        } else {
            self.legacy.get(signature)
        }
    }

    /// Store a style. The signature is read from the style itself.
    /// Java: `StyleStorage.put(Style)`
    pub fn put(&mut self, style: Style) {
        let sig = style.signature().clone();
        if sig.get_stereotypes().is_empty() {
            self.plain.insert(sig.get_key().clone(), style);
        } else {
            self.legacy.insert(sig, style);
        }
    }

    /// Copy all rules from another storage.
    pub fn put_all(&mut self, other: &StyleStorage) {
        self.legacy.extend(other.legacy.clone());
        self.plain.extend(other.plain.clone());
    }

    /// Iterate over all stored styles (legacy + plain).
    pub fn styles(&self) -> impl Iterator<Item = &Style> {
        self.legacy.values().chain(self.plain.values())
    }

    /// Compute a merged style for the given element signature by scanning
    /// all stored rules and merging those that match.
    /// Java: `StyleStorage.computeMergedStyle(StyleSignatureBasic)`
    pub fn compute_merged_style(&self, signature: &StyleSignatureBasic) -> Style {
        let mut merged: Option<Style> = None;
        for style in self.styles() {
            let key = style.signature();
            if !key.match_all(signature) {
                continue;
            }
            merged = match merged {
                None => Some(style.clone()),
                Some(existing) => {
                    Some(existing.merge_with(style, MergeStrategy::OverwriteExistingValue))
                }
            };
        }
        merged.unwrap_or_else(|| Style::empty(signature.clone()))
    }

    /// Print all styles for debugging.
    pub fn print_me(&self) {
        for style in self.legacy.values() {
            style.print_me();
        }
        for style in self.plain.values() {
            style.print_me();
        }
    }
}

// ── StyleBuilder ─────────────────────────────────────────────────────

/// The main style builder: holds all loaded style rules and provides
/// merged style computation with caching.
///
/// Java: `style.StyleBuilder`
#[derive(Debug)]
pub struct StyleBuilder {
    storage: StyleStorage,
    counter: AtomicI32,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self {
            storage: StyleStorage::new(),
            counter: AtomicI32::new(0),
        }
    }

    /// Clone the builder (including all stored rules).
    /// Java: `StyleBuilder.cloneMe()`
    pub fn clone_me(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            counter: AtomicI32::new(self.counter.load(Ordering::Relaxed)),
        }
    }

    /// Get a monotonically increasing counter value (used for priority ordering).
    /// Java: `AutomaticCounter.getNextInt()`
    pub fn next_int(&self) -> i32 {
        self.counter.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Create a style for a stereotype name lookup.
    /// Java: `StyleBuilder.createStyleStereotype(String)`
    pub fn create_style_stereotype(&self, name: &str) -> Style {
        let lower = name.to_ascii_lowercase();
        let signature = StyleSignatureBasic::create_stereotype(&lower);
        match self.storage.get(&signature) {
            Some(style) => style.clone(),
            None => Style::empty(signature),
        }
    }

    /// Load a style rule (from parsing). Merges with existing if present.
    /// Java: `StyleBuilder.loadInternal(StyleSignatureBasic, Style)`
    pub fn load_internal(&mut self, signature: &StyleSignatureBasic, new_style: Style) {
        if signature.is_starred() {
            log::warn!("loadInternal called with starred signature: {}", signature);
            return;
        }
        match self.storage.get(signature).cloned() {
            Some(orig) => {
                let merged = orig.merge_with(&new_style, MergeStrategy::OverwriteExistingValue);
                self.storage.put(merged);
            }
            None => {
                self.storage.put(new_style);
            }
        }
    }

    /// Create a new builder with mutated styles applied.
    /// Java: `StyleBuilder.muteStyle(Collection<Style>)`
    pub fn mute_style(&self, modified_styles: &[Style]) -> StyleBuilder {
        let mut result = self.clone_me();
        for modified in modified_styles {
            let sig = modified.signature().clone();
            match result.storage.get(&sig).cloned() {
                Some(orig) => {
                    let merged = orig.merge_with(modified, MergeStrategy::OverwriteExistingValue);
                    result.storage.put(merged);
                }
                None => {
                    result.storage.put(modified.clone());
                }
            }
        }
        result
    }

    /// Get the merged style for a given element signature.
    /// This scans all stored rules and merges matching ones.
    ///
    /// Note: Java uses a ConcurrentHashMap cache here. Caching can be
    /// added later if profiling shows need.
    ///
    /// Java: `StyleBuilder.getMergedStyle(StyleSignatureBasic)`
    pub fn get_merged_style(&self, signature: &StyleSignatureBasic) -> Style {
        info!("Using style {}", signature);
        self.storage.compute_merged_style(signature)
    }

    /// Get merged style with an additional delta priority for starred rules.
    /// Java: `StyleBuilder.getMergedStyleSpecial(StyleSignatureBasic, int)`
    pub fn get_merged_style_special(
        &self,
        signature: &StyleSignatureBasic,
        delta_priority: i32,
    ) -> Option<Style> {
        let mut merged: Option<Style> = None;
        for style in self.storage.styles() {
            let key = style.signature();
            if !key.match_all(signature) {
                continue;
            }
            let tmp = if key.is_starred() {
                style.delta_priority(delta_priority)
            } else {
                style.clone()
            };
            merged = match merged {
                None => Some(tmp),
                Some(existing) => {
                    Some(existing.merge_with(&tmp, MergeStrategy::OverwriteExistingValue))
                }
            };
        }
        merged
    }

    /// Access the underlying storage (for debugging).
    pub fn storage(&self) -> &StyleStorage {
        &self.storage
    }

    /// Print all stored styles for debugging.
    pub fn print_me(&self) {
        self.storage.print_me();
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StyleBuilder {
    fn clone(&self) -> Self {
        self.clone_me()
    }
}

// ── StyleLoader ──────────────────────────────────────────────────────

/// Static utility for loading skin files and converting stereotype priorities.
///
/// Java: `style.StyleLoader`
pub struct StyleLoader;

/// Priority boost constant for stereotype-specific style rules.
/// Java: `StyleLoader.DELTA_PRIORITY_FOR_STEREOTYPE`
pub const DELTA_PRIORITY_FOR_STEREOTYPE: i32 = 1000;

impl StyleLoader {
    /// Boost all priorities in a property map by the stereotype delta.
    /// Java: `StyleLoader.addPriorityForStereotype(Map<PName, Value>)`
    pub fn add_priority_for_stereotype(
        map: &HashMap<PName, ValueImpl>,
    ) -> HashMap<PName, ValueImpl> {
        map.iter()
            .map(|(k, v)| (*k, v.add_priority(DELTA_PRIORITY_FOR_STEREOTYPE)))
            .collect()
    }
}

// ── Style ID constants ───────────────────────────────────────────────

/// ID for title style blocks. Java: `Style.ID_TITLE`
pub const STYLE_ID_TITLE: &str = "_title";
/// ID for caption style blocks. Java: `Style.ID_CAPTION`
pub const STYLE_ID_CAPTION: &str = "_caption";
/// ID for legend style blocks. Java: `Style.ID_LEGEND`
pub const STYLE_ID_LEGEND: &str = "_legend";

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::sname::SName;

    // ── ClockwiseTopRightBottomLeft tests ────────────────────────────

    #[test]
    fn clockwise_none() {
        let c = ClockwiseTopRightBottomLeft::none();
        assert_eq!(c.top, 0.0);
        assert_eq!(c.right, 0.0);
        assert_eq!(c.bottom, 0.0);
        assert_eq!(c.left, 0.0);
    }

    #[test]
    fn clockwise_same() {
        let c = ClockwiseTopRightBottomLeft::same(10.0);
        assert_eq!(c.top, 10.0);
        assert_eq!(c.right, 10.0);
        assert_eq!(c.bottom, 10.0);
        assert_eq!(c.left, 10.0);
    }

    #[test]
    fn clockwise_read_one_value() {
        let c = ClockwiseTopRightBottomLeft::read("5");
        assert_eq!(c, ClockwiseTopRightBottomLeft::same(5.0));
    }

    #[test]
    fn clockwise_read_two_values() {
        let c = ClockwiseTopRightBottomLeft::read("10 20");
        assert_eq!(c.top, 10.0);
        assert_eq!(c.right, 20.0);
        assert_eq!(c.bottom, 10.0);
        assert_eq!(c.left, 20.0);
    }

    #[test]
    fn clockwise_read_three_values() {
        let c = ClockwiseTopRightBottomLeft::read("10 20 30");
        assert_eq!(c.top, 10.0);
        assert_eq!(c.right, 20.0);
        assert_eq!(c.bottom, 30.0);
        assert_eq!(c.left, 20.0);
    }

    #[test]
    fn clockwise_read_four_values() {
        let c = ClockwiseTopRightBottomLeft::read("1 2 3 4");
        assert_eq!(c.top, 1.0);
        assert_eq!(c.right, 2.0);
        assert_eq!(c.bottom, 3.0);
        assert_eq!(c.left, 4.0);
    }

    #[test]
    fn clockwise_read_non_numeric() {
        let c = ClockwiseTopRightBottomLeft::read("abc");
        assert_eq!(c, ClockwiseTopRightBottomLeft::none());
    }

    #[test]
    fn clockwise_read_empty() {
        let c = ClockwiseTopRightBottomLeft::read("");
        assert_eq!(c, ClockwiseTopRightBottomLeft::none());
    }

    #[test]
    fn clockwise_inc_top() {
        let c = ClockwiseTopRightBottomLeft::same(5.0).inc_top(3.0);
        assert_eq!(c.top, 8.0);
        assert_eq!(c.right, 5.0);
    }

    #[test]
    fn clockwise_display() {
        let c = ClockwiseTopRightBottomLeft::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(format!("{}", c), "1:2:3:4");
    }

    // ── Style tests ──────────────────────────────────────────────────

    #[test]
    fn style_empty_has_no_values() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = Style::empty(sig);
        assert!(!style.has_value(PName::FontSize));
        assert_eq!(style.value(PName::FontSize).as_string(), "");
    }

    #[test]
    fn style_with_values() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 1));
        map.insert(PName::FontName, ValueImpl::regular("SansSerif", 1));
        let style = Style::new(sig, map);

        assert!(style.has_value(PName::FontSize));
        assert_eq!(style.value(PName::FontSize).as_int(false), 14);
        assert_eq!(style.value(PName::FontName).as_string(), "SansSerif");
    }

    #[test]
    fn style_font_size_default_when_missing() {
        // When FontSize is completely absent, ValueNull.as_int(true) returns 0
        // (matching Java's ValueNull behavior). The -1 default path only triggers
        // when the value string has no digits (e.g. "abc").
        let style = Style::empty(StyleSignatureBasic::of(&[SName::Root]));
        assert_eq!(style.font_size(), 0);
    }

    #[test]
    fn style_font_size_default_when_no_digits() {
        // When FontSize value has no parseable digits, as_int(true) returns -1,
        // triggering the default of 14.
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("abc", 1));
        let style = Style::new(sig, map);
        assert_eq!(style.font_size(), 14);
    }

    #[test]
    fn style_font_size_from_value() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("18", 1));
        let style = Style::new(sig, map);
        assert_eq!(style.font_size(), 18);
    }

    #[test]
    fn style_shadowing_default() {
        let style = Style::empty(StyleSignatureBasic::of(&[SName::Root]));
        assert_eq!(style.shadowing(), 0.0);
    }

    #[test]
    fn style_shadowing_with_value() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::Shadowing, ValueImpl::regular("3", 1));
        let style = Style::new(sig, map);
        assert_eq!(style.shadowing(), 3.0);
    }

    #[test]
    fn style_stroke_solid() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::LineThickness, ValueImpl::regular("2", 1));
        let style = Style::new(sig, map);
        let stroke = style.stroke();
        assert_eq!(stroke.thickness, 2.0);
        assert!(stroke.dasharray_svg().is_none());
    }

    #[test]
    fn style_stroke_dashed() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::LineThickness, ValueImpl::regular("1.5", 1));
        map.insert(PName::LineStyle, ValueImpl::regular("5-3", 1));
        let style = Style::new(sig, map);
        let stroke = style.stroke();
        assert_eq!(stroke.thickness, 1.5);
        assert_eq!(stroke.dash_visible, 5.0);
        assert_eq!(stroke.dash_space, 3.0);
    }

    #[test]
    fn style_padding_and_margin() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::Padding, ValueImpl::regular("5 10", 1));
        map.insert(PName::Margin, ValueImpl::regular("2", 1));
        let style = Style::new(sig, map);

        let pad = style.padding();
        assert_eq!(pad.top, 5.0);
        assert_eq!(pad.right, 10.0);

        let margin = style.margin();
        assert_eq!(margin, ClockwiseTopRightBottomLeft::same(2.0));
    }

    #[test]
    fn style_merge() {
        let sig_a = StyleSignatureBasic::of(&[SName::Root]);
        let sig_b = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);

        let mut map_a = HashMap::new();
        map_a.insert(PName::FontSize, ValueImpl::regular("14", 1));
        map_a.insert(PName::FontColor, ValueImpl::regular("#000000", 1));
        let style_a = Style::new(sig_a, map_a);

        let mut map_b = HashMap::new();
        map_b.insert(PName::FontSize, ValueImpl::regular("18", 5));
        map_b.insert(PName::LineColor, ValueImpl::regular("#FF0000", 5));
        let style_b = Style::new(sig_b, map_b);

        let merged = style_a.merge_with(&style_b, MergeStrategy::OverwriteExistingValue);
        // FontSize should be overwritten (higher priority)
        assert_eq!(merged.value(PName::FontSize).as_int(false), 18);
        // FontColor preserved from A
        assert!(merged.has_value(PName::FontColor));
        // LineColor added from B
        assert!(merged.has_value(PName::LineColor));
    }

    #[test]
    fn style_delta_priority() {
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_star();
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 5));
        let style = Style::new(sig, map);

        let shifted = style.delta_priority(100);
        assert_eq!(shifted.value(PName::FontSize).priority(), 105);
    }

    #[test]
    fn style_override_value() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 1));
        let style = Style::new(sig, map);

        let updated = style.override_value(PName::FontSize, "20");
        assert_eq!(updated.value(PName::FontSize).as_int(false), 20);
    }

    #[test]
    fn style_override_double() {
        let style = Style::empty(StyleSignatureBasic::of(&[SName::Root]));
        let updated = style.override_double(PName::LineThickness, 2.5);
        assert!((updated.value(PName::LineThickness).as_double() - 2.5).abs() < 0.01);
    }

    // ── StyleStorage tests ───────────────────────────────────────────

    #[test]
    fn storage_put_and_get_plain() {
        let mut storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 1));
        storage.put(Style::new(sig.clone(), map));

        let result = storage.get(&sig);
        assert!(result.is_some());
        assert_eq!(result.unwrap().value(PName::FontSize).as_int(false), 14);
    }

    #[test]
    fn storage_put_and_get_stereotype() {
        let mut storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("mytype");
        let mut map = HashMap::new();
        map.insert(PName::FontColor, ValueImpl::regular("#FF0000", 1));
        storage.put(Style::new(sig.clone(), map));

        let result = storage.get(&sig);
        assert!(result.is_some());
    }

    #[test]
    fn storage_compute_merged() {
        let mut storage = StyleStorage::new();

        // Rule 1: root { FontSize: 14 }
        let sig1 = StyleSignatureBasic::of(&[SName::Root]);
        let mut map1 = HashMap::new();
        map1.insert(PName::FontSize, ValueImpl::regular("14", 1));
        storage.put(Style::new(sig1, map1));

        // Rule 2: root, arrow { LineColor: #FF0000, FontSize: 12 }
        let sig2 = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        let mut map2 = HashMap::new();
        map2.insert(PName::LineColor, ValueImpl::regular("#FF0000", 2));
        map2.insert(PName::FontSize, ValueImpl::regular("12", 2));
        storage.put(Style::new(sig2, map2));

        // Query for root, element, arrow — should match both rules
        let query = StyleSignatureBasic::of(&[SName::Root, SName::Element, SName::Arrow]);
        let merged = storage.compute_merged_style(&query);
        assert!(merged.has_value(PName::LineColor));
        // FontSize from rule2 should win (higher priority)
        assert_eq!(merged.value(PName::FontSize).as_int(false), 12);
    }

    // ── StyleBuilder tests ───────────────────────────────────────────

    #[test]
    fn builder_new_is_empty() {
        let builder = StyleBuilder::new();
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let merged = builder.get_merged_style(&sig);
        assert!(!merged.has_value(PName::FontSize));
    }

    #[test]
    fn builder_load_and_query() {
        let mut builder = StyleBuilder::new();

        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 1));
        map.insert(PName::LineColor, ValueImpl::regular("#333333", 1));
        let style = Style::new(sig.clone(), map);
        builder.load_internal(&sig, style);

        let query = StyleSignatureBasic::of(&[SName::Root, SName::Element, SName::Arrow]);
        let merged = builder.get_merged_style(&query);
        assert_eq!(merged.value(PName::FontSize).as_int(false), 14);
    }

    #[test]
    fn builder_next_int_increments() {
        let builder = StyleBuilder::new();
        let a = builder.next_int();
        let b = builder.next_int();
        let c = builder.next_int();
        assert_eq!(b, a + 1);
        assert_eq!(c, a + 2);
    }

    #[test]
    fn builder_clone_me() {
        let mut builder = StyleBuilder::new();
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 1));
        builder.load_internal(&sig, Style::new(sig.clone(), map));

        let cloned = builder.clone_me();
        let merged = cloned.get_merged_style(&sig);
        assert_eq!(merged.value(PName::FontSize).as_int(false), 14);
    }

    #[test]
    fn builder_mute_style() {
        let mut builder = StyleBuilder::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 1));
        builder.load_internal(&sig, Style::new(sig.clone(), map));

        // Mute: change font size
        let mut new_map = HashMap::new();
        new_map.insert(PName::FontSize, ValueImpl::regular("20", 10));
        let muted = builder.mute_style(&[Style::new(sig.clone(), new_map)]);

        let query = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        let merged = muted.get_merged_style(&query);
        assert_eq!(merged.value(PName::FontSize).as_int(false), 20);
    }

    #[test]
    fn builder_create_style_stereotype() {
        let builder = StyleBuilder::new();
        let style = builder.create_style_stereotype("MyType");
        assert!(style.signature().is_with_dot());
        assert!(style.properties().is_empty());
    }

    #[test]
    fn builder_get_merged_style_special() {
        let mut builder = StyleBuilder::new();

        // Add a starred rule
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_star();
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 5));
        builder.storage.put(Style::new(sig, map));

        // Query with special delta
        let query = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]).add_star();
        let result = builder.get_merged_style_special(&query, 100);
        assert!(result.is_some());
        assert_eq!(result.unwrap().value(PName::FontSize).priority(), 105);
    }

    // ── StyleLoader tests ────────────────────────────────────────────

    #[test]
    fn loader_add_priority_for_stereotype() {
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("14", 5));
        map.insert(PName::FontColor, ValueImpl::regular("#000", 3));

        let boosted = StyleLoader::add_priority_for_stereotype(&map);
        assert_eq!(boosted.get(&PName::FontSize).unwrap().priority(), 1005);
        assert_eq!(boosted.get(&PName::FontColor).unwrap().priority(), 1003);
    }

    // ── Style ID constants test ──────────────────────────────────────

    #[test]
    fn style_id_constants() {
        assert_eq!(STYLE_ID_TITLE, "_title");
        assert_eq!(STYLE_ID_CAPTION, "_caption");
        assert_eq!(STYLE_ID_LEGEND, "_legend");
    }
}
