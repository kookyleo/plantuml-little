// Style signature types — CSS-like selectors for the style system.
// Port of Java PlantUML's StyleSignature, StyleSignatureBasic, StyleKey

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

use super::sname::SName;

// ── StyleKey ─────────────────────────────────────────────────────────

/// The non-stereotype part of a style signature.
///
/// Holds a set of SName selectors, an optional nesting level, and a
/// "starred" flag (for wildcard-matching rules).
///
/// Java: `style.StyleKey`
#[derive(Debug, Clone)]
pub struct StyleKey {
    /// The set of SName selectors (e.g. {root, element, sequenceDiagram, arrow}).
    pub snames: BTreeSet<SName>,
    /// Nesting level (-1 = no level constraint).
    pub level: i32,
    /// Whether this key has a wildcard ("*") suffix.
    pub is_starred: bool,
}

impl StyleKey {
    /// Empty key: no SNames, level=-1, not starred.
    pub fn empty() -> Self {
        Self {
            snames: BTreeSet::new(),
            level: -1,
            is_starred: false,
        }
    }

    /// Create from a slice of SNames.
    pub fn of(names: &[SName]) -> Self {
        Self {
            snames: names.iter().copied().collect(),
            level: -1,
            is_starred: false,
        }
    }

    /// Add a clickable marker. Java: `StyleKey.addClickable(Url)`
    pub fn add_clickable(&self) -> Self {
        let mut result = self.snames.clone();
        result.insert(SName::Clickable);
        Self {
            snames: result,
            level: self.level,
            is_starred: self.is_starred,
        }
    }

    /// Add a nesting level constraint. Java: `StyleKey.addLevel(int)`
    pub fn add_level(&self, level: i32) -> Self {
        Self {
            snames: self.snames.clone(),
            level,
            is_starred: self.is_starred,
        }
    }

    /// Add an SName to the set. Java: `StyleKey.addSName(SName)`
    pub fn add_sname(&self, name: SName) -> Self {
        let mut result = self.snames.clone();
        result.insert(name);
        Self {
            snames: result,
            level: self.level,
            is_starred: self.is_starred,
        }
    }

    /// Mark as starred (wildcard). Java: `StyleKey.addStar()`
    pub fn add_star(&self) -> Self {
        Self {
            snames: self.snames.clone(),
            level: self.level,
            is_starred: true,
        }
    }

    /// Merge two keys: union of SNames, max of levels, OR of starred.
    /// Java: `StyleKey.mergeWith(StyleKey)`
    pub fn merge_with(&self, other: &StyleKey) -> Self {
        let mut merged = self.snames.clone();
        merged.extend(&other.snames);
        Self {
            snames: merged,
            level: self.level.max(other.level),
            is_starred: self.is_starred || other.is_starred,
        }
    }
}

impl PartialEq for StyleKey {
    fn eq(&self, other: &Self) -> bool {
        self.snames == other.snames
            && self.is_starred == other.is_starred
            && self.level == other.level
    }
}

impl Eq for StyleKey {}

impl Hash for StyleKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the sorted SName set deterministically
        for sname in &self.snames {
            sname.hash(state);
        }
        self.is_starred.hash(state);
        self.level.hash(state);
    }
}

impl std::fmt::Display for StyleKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.snames)?;
        if self.level != -1 {
            write!(f, " {}", self.level)?;
        }
        if self.is_starred {
            write!(f, " (*)")?;
        }
        Ok(())
    }
}

// ── StyleSignatureBasic ──────────────────────────────────────────────

/// A CSS-like style selector: a `StyleKey` (SName set + level + star)
/// plus zero or more stereotype strings.
///
/// This is the workhorse of style matching. When asking "what styles
/// apply to this element?", each stored style's signature is checked
/// against the element's signature via `match_all`.
///
/// Java: `style.StyleSignatureBasic`
#[derive(Debug, Clone)]
pub struct StyleSignatureBasic {
    key: StyleKey,
    stereotypes: BTreeSet<String>,
}

impl StyleSignatureBasic {
    /// Empty signature (no SNames, no stereotypes).
    pub fn empty() -> Self {
        Self {
            key: StyleKey::empty(),
            stereotypes: BTreeSet::new(),
        }
    }

    /// Create from a slice of SNames (no stereotypes).
    pub fn of(names: &[SName]) -> Self {
        Self {
            key: StyleKey::of(names),
            stereotypes: BTreeSet::new(),
        }
    }

    /// Create a stereotype-only signature (used for stereotype style lookup).
    /// Java: `StyleSignatureBasic.createStereotype(String)`
    pub fn create_stereotype(name: &str) -> Self {
        Self::empty().add_stereotype(name)
    }

    /// Add an SName to the selector.
    /// Java: `StyleSignatureBasic.addSName(SName)`
    pub fn add_sname(&self, name: SName) -> Self {
        Self {
            key: self.key.add_sname(name),
            stereotypes: self.stereotypes.clone(),
        }
    }

    /// Add a stereotype string (cleaned: lowercased, underscores/dots removed).
    /// Java: `StyleSignatureBasic.addStereotype(String)`
    pub fn add_stereotype(&self, stereo: &str) -> Self {
        let mut result = self.stereotypes.clone();
        result.insert(clean_stereotype(stereo));
        Self {
            key: self.key.clone(),
            stereotypes: result,
        }
    }

    /// Add the "clickable" SName.
    pub fn add_clickable(&self) -> Self {
        Self {
            key: self.key.add_clickable(),
            stereotypes: self.stereotypes.clone(),
        }
    }

    /// Add a nesting level constraint.
    pub fn add_level(&self, level: i32) -> Self {
        Self {
            key: self.key.add_level(level),
            stereotypes: self.stereotypes.clone(),
        }
    }

    /// Add a star (wildcard) suffix.
    pub fn add_star(&self) -> Self {
        Self {
            key: self.key.add_star(),
            stereotypes: self.stereotypes.clone(),
        }
    }

    /// Whether this signature is starred (wildcard).
    pub fn is_starred(&self) -> bool {
        self.key.is_starred
    }

    /// Whether this signature contains stereotypes.
    pub fn is_with_dot(&self) -> bool {
        !self.stereotypes.is_empty()
    }

    /// Whether the signature is completely empty.
    pub fn is_empty(&self) -> bool {
        self.key.snames.is_empty() && self.stereotypes.is_empty()
    }

    /// Get the underlying StyleKey (non-stereotype portion).
    pub fn get_key(&self) -> &StyleKey {
        &self.key
    }

    /// Get the stereotype set.
    pub fn get_stereotypes(&self) -> &BTreeSet<String> {
        &self.stereotypes
    }

    /// CSS selector matching: does this (declaration) signature match the
    /// given (element) signature?
    ///
    /// A declaration matches an element when:
    /// 1. Level constraints are satisfied (if declared)
    /// 2. The element is not starred unless the declaration is also starred
    /// 3. The element's SNames are a superset of the declaration's SNames
    /// 4. The element's stereotypes are a superset of the declaration's stereotypes
    ///
    /// Java: `StyleSignatureBasic.matchAll(StyleSignatureBasic)`
    pub fn match_all(&self, element: &StyleSignatureBasic) -> bool {
        // Level check
        if self.key.level != -1 {
            if self.key.is_starred {
                if element.key.level == -1 || element.key.level < self.key.level {
                    return false;
                }
            } else if element.key.level == -1 || element.key.level != self.key.level {
                return false;
            }
        }

        // Star check: starred element must only match starred declarations
        if element.is_starred() && !self.is_starred() {
            return false;
        }

        // SName containment: element must contain all declaration SNames
        if !element.key.snames.is_superset(&self.key.snames) {
            return false;
        }

        // Stereotype containment
        if !element.stereotypes.is_superset(&self.stereotypes) {
            return false;
        }

        true
    }

    /// Merge two signatures: union SNames, union stereotypes.
    /// Java: `StyleSignatureBasic.mergeWith(StyleSignatureBasic)`
    pub fn merge_with(&self, other: &StyleSignatureBasic) -> Self {
        let mut stereos = self.stereotypes.clone();
        stereos.extend(other.stereotypes.iter().cloned());
        Self {
            key: self.key.merge_with(&other.key),
            stereotypes: stereos,
        }
    }

    // ── Convenience constructors for common selectors ───────────────

    pub fn activity() -> Self {
        Self::of(&[SName::Root, SName::Element, SName::ActivityDiagram, SName::Activity])
    }

    pub fn activity_diamond() -> Self {
        Self::of(&[SName::Root, SName::Element, SName::ActivityDiagram, SName::Activity, SName::Diamond])
    }

    pub fn activity_arrow() -> Self {
        Self::of(&[SName::Root, SName::Element, SName::ActivityDiagram, SName::Activity, SName::Arrow])
    }
}

impl PartialEq for StyleSignatureBasic {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.stereotypes == other.stereotypes
    }
}

impl Eq for StyleSignatureBasic {}

impl Hash for StyleSignatureBasic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        for s in &self.stereotypes {
            s.hash(state);
        }
    }
}

impl std::fmt::Display for StyleSignatureBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.key, self.stereotypes)
    }
}

// ── StyleSignature trait ─────────────────────────────────────────────

/// Trait for objects that can be resolved into a Style.
/// Java: `style.StyleSignature`
pub trait StyleSignature {
    /// Get the merged style from a StyleBuilder.
    fn get_merged_style(&self, builder: &super::style_def::StyleBuilder) -> Option<super::style_def::Style>;
}

impl StyleSignature for StyleSignatureBasic {
    fn get_merged_style(&self, builder: &super::style_def::StyleBuilder) -> Option<super::style_def::Style> {
        Some(builder.get_merged_style(self))
    }
}

// ── StyleSignatures (multi-signature) ────────────────────────────────

/// A collection of StyleSignatures that merges results from all of them.
/// Java: `style.StyleSignatures`
#[derive(Debug, Clone)]
pub struct StyleSignatures {
    all: Vec<StyleSignatureBasic>,
}

impl StyleSignatures {
    pub fn new() -> Self {
        Self { all: Vec::new() }
    }

    pub fn add(&mut self, sig: StyleSignatureBasic) {
        self.all.push(sig);
    }

    pub fn is_empty(&self) -> bool {
        self.all.is_empty()
    }
}

impl Default for StyleSignatures {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleSignature for StyleSignatures {
    fn get_merged_style(&self, builder: &super::style_def::StyleBuilder) -> Option<super::style_def::Style> {
        if self.all.is_empty() {
            return None;
        }
        let mut result: Option<super::style_def::Style> = None;
        for basic in &self.all {
            let tmp = builder.get_merged_style(basic);
            result = match result {
                None => Some(tmp),
                Some(existing) => Some(existing.merge_with(
                    &tmp,
                    super::value::MergeStrategy::KeepExistingValueOfStereotype,
                )),
            };
        }
        result
    }
}

// ── Styleable trait ──────────────────────────────────────────────────

/// Trait for diagram elements that have a style signature.
/// Java: `style.Styleable`
pub trait Styleable {
    fn style_signature(&self) -> &dyn StyleSignature;
}

// ── Helper: clean stereotype name ────────────────────────────────────

/// Clean a stereotype name: lowercase, remove underscores and dots.
/// Java: `StyleSignatureBasic.clean(String)`
fn clean_stereotype(name: &str) -> String {
    let mut sb = String::with_capacity(name.len());
    for c in name.chars() {
        if c != '_' && c != '.' {
            for lc in c.to_lowercase() {
                sb.push(lc);
            }
        }
    }
    sb
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── StyleKey tests ───────────────────────────────────────────────

    #[test]
    fn style_key_empty() {
        let k = StyleKey::empty();
        assert!(k.snames.is_empty());
        assert_eq!(k.level, -1);
        assert!(!k.is_starred);
    }

    #[test]
    fn style_key_of() {
        let k = StyleKey::of(&[SName::Root, SName::Arrow]);
        assert_eq!(k.snames.len(), 2);
        assert!(k.snames.contains(&SName::Root));
        assert!(k.snames.contains(&SName::Arrow));
    }

    #[test]
    fn style_key_add_sname() {
        let k = StyleKey::of(&[SName::Root]);
        let k2 = k.add_sname(SName::Arrow);
        assert_eq!(k2.snames.len(), 2);
        assert!(k2.snames.contains(&SName::Arrow));
        // Original unchanged
        assert_eq!(k.snames.len(), 1);
    }

    #[test]
    fn style_key_add_level() {
        let k = StyleKey::empty().add_level(3);
        assert_eq!(k.level, 3);
    }

    #[test]
    fn style_key_add_star() {
        let k = StyleKey::empty().add_star();
        assert!(k.is_starred);
    }

    #[test]
    fn style_key_merge() {
        let a = StyleKey::of(&[SName::Root]).add_level(2);
        let b = StyleKey::of(&[SName::Arrow]).add_level(5).add_star();
        let m = a.merge_with(&b);
        assert!(m.snames.contains(&SName::Root));
        assert!(m.snames.contains(&SName::Arrow));
        assert_eq!(m.level, 5);
        assert!(m.is_starred);
    }

    #[test]
    fn style_key_eq_and_hash() {
        let a = StyleKey::of(&[SName::Root, SName::Arrow]);
        let b = StyleKey::of(&[SName::Arrow, SName::Root]);
        assert_eq!(a, b);
        use std::collections::hash_map::DefaultHasher;
        let hash_a = {
            let mut h = DefaultHasher::new();
            a.hash(&mut h);
            h.finish()
        };
        let hash_b = {
            let mut h = DefaultHasher::new();
            b.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn style_key_display() {
        let k = StyleKey::of(&[SName::Root]).add_star();
        let s = format!("{}", k);
        assert!(s.contains("Root"));
        assert!(s.contains("(*)"));
    }

    // ── StyleSignatureBasic tests ────────────────────────────────────

    #[test]
    fn signature_empty() {
        let sig = StyleSignatureBasic::empty();
        assert!(sig.is_empty());
        assert!(!sig.is_starred());
        assert!(!sig.is_with_dot());
    }

    #[test]
    fn signature_of() {
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Element, SName::Arrow]);
        assert!(!sig.is_empty());
        assert_eq!(sig.get_key().snames.len(), 3);
    }

    #[test]
    fn signature_add_stereotype() {
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("MyType");
        assert!(sig.is_with_dot());
        assert!(sig.get_stereotypes().contains("mytype"));
    }

    #[test]
    fn signature_clean_stereotype() {
        // Underscores and dots should be removed, lowercased
        let sig = StyleSignatureBasic::empty().add_stereotype("My_Type.Name");
        assert!(sig.get_stereotypes().contains("mytypename"));
    }

    #[test]
    fn signature_add_sname() {
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let sig2 = sig.add_sname(SName::Arrow);
        assert!(sig2.get_key().snames.contains(&SName::Arrow));
    }

    #[test]
    fn signature_match_all_basic() {
        // Declaration: root, arrow
        let decl = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        // Element: root, element, sequenceDiagram, arrow
        let elem = StyleSignatureBasic::of(&[
            SName::Root, SName::Element, SName::SequenceDiagram, SName::Arrow,
        ]);
        // Declaration's SNames are a subset of element's
        assert!(decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_no_match() {
        // Declaration has SName that element doesn't
        let decl = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let elem = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        assert!(!decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_star() {
        // Starred declaration matches starred element
        let decl = StyleSignatureBasic::of(&[SName::Root]).add_star();
        let elem = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]).add_star();
        assert!(decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_star_mismatch() {
        // Non-starred declaration doesn't match starred element
        let decl = StyleSignatureBasic::of(&[SName::Root]);
        let elem = StyleSignatureBasic::of(&[SName::Root]).add_star();
        assert!(!decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_stereotype() {
        let decl = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("foo");
        let elem = StyleSignatureBasic::of(&[SName::Root, SName::Arrow])
            .add_stereotype("foo")
            .add_stereotype("bar");
        assert!(decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_stereotype_missing() {
        let decl = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("foo");
        let elem = StyleSignatureBasic::of(&[SName::Root, SName::Arrow])
            .add_stereotype("bar");
        assert!(!decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_level_exact() {
        let decl = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(3);
        let elem = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(3);
        assert!(decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_level_mismatch() {
        let decl = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(3);
        let elem = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(5);
        assert!(!decl.match_all(&elem));
    }

    #[test]
    fn signature_match_all_starred_level_gte() {
        let decl = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(3)
            .add_star();
        let elem_ok = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(5)
            .add_star();
        let elem_fail = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(2)
            .add_star();
        assert!(decl.match_all(&elem_ok));
        assert!(!decl.match_all(&elem_fail));
    }

    #[test]
    fn signature_merge() {
        let a = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("foo");
        let b = StyleSignatureBasic::of(&[SName::Arrow]).add_stereotype("bar");
        let m = a.merge_with(&b);
        assert!(m.get_key().snames.contains(&SName::Root));
        assert!(m.get_key().snames.contains(&SName::Arrow));
        assert!(m.get_stereotypes().contains("foo"));
        assert!(m.get_stereotypes().contains("bar"));
    }

    #[test]
    fn signature_eq_and_hash() {
        let a = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]).add_stereotype("x");
        let b = StyleSignatureBasic::of(&[SName::Arrow, SName::Root]).add_stereotype("x");
        assert_eq!(a, b);

        use std::collections::hash_map::DefaultHasher;
        let hash_a = {
            let mut h = DefaultHasher::new();
            a.hash(&mut h);
            h.finish()
        };
        let hash_b = {
            let mut h = DefaultHasher::new();
            b.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn signature_create_stereotype() {
        let sig = StyleSignatureBasic::create_stereotype("mytype");
        assert!(sig.is_with_dot());
        assert!(sig.get_stereotypes().contains("mytype"));
        assert!(sig.get_key().snames.is_empty());
    }

    #[test]
    fn signature_convenience_activity() {
        let sig = StyleSignatureBasic::activity();
        assert!(sig.get_key().snames.contains(&SName::Root));
        assert!(sig.get_key().snames.contains(&SName::Element));
        assert!(sig.get_key().snames.contains(&SName::ActivityDiagram));
        assert!(sig.get_key().snames.contains(&SName::Activity));
    }

    #[test]
    fn signature_convenience_activity_diamond() {
        let sig = StyleSignatureBasic::activity_diamond();
        assert!(sig.get_key().snames.contains(&SName::Diamond));
    }

    #[test]
    fn signature_convenience_activity_arrow() {
        let sig = StyleSignatureBasic::activity_arrow();
        assert!(sig.get_key().snames.contains(&SName::Arrow));
    }

    // ── StyleSignatures tests ────────────────────────────────────────

    #[test]
    fn style_signatures_empty() {
        let ss = StyleSignatures::new();
        assert!(ss.is_empty());
    }

    #[test]
    fn style_signatures_add() {
        let mut ss = StyleSignatures::new();
        ss.add(StyleSignatureBasic::of(&[SName::Root]));
        ss.add(StyleSignatureBasic::of(&[SName::Arrow]));
        assert!(!ss.is_empty());
        assert_eq!(ss.all.len(), 2);
    }

    // ── clean_stereotype tests ───────────────────────────────────────

    #[test]
    fn clean_basic() {
        assert_eq!(clean_stereotype("MyType"), "mytype");
    }

    #[test]
    fn clean_underscore_and_dot() {
        assert_eq!(clean_stereotype("My_Type.Name"), "mytypename");
    }

    #[test]
    fn clean_already_clean() {
        assert_eq!(clean_stereotype("abc"), "abc");
    }

    #[test]
    fn clean_unicode() {
        assert_eq!(clean_stereotype("TypeA"), "typea");
    }
}
