// decoration::link_decor - Arrow endpoint decoration types
// Port of Java PlantUML's decoration.LinkDecor + LinkMiddleDecor

use std::collections::HashMap;
use std::sync::LazyLock;

/// Arrow endpoint decoration style.
/// Java: `decoration.LinkDecor`
///
/// Each variant carries metadata identical to the Java enum fields:
/// `(margin, fill, arrow_size)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkDecor {
    None,
    Extends,
    Composition,
    Agregation,
    NotNavigable,
    Redefines,
    DefinedBy,
    Crowfoot,
    CircleCrowfoot,
    CircleLine,
    DoubleLine,
    LineCrowfoot,
    Arrow,
    ArrowTriangle,
    ArrowAndCircle,
    Circle,
    CircleFill,
    CircleConnect,
    Parenthesis,
    Square,
    CircleCross,
    Plus,
    HalfArrowUp,
    HalfArrowDown,
    SquareToBeRemoved,
}

impl Default for LinkDecor {
    fn default() -> Self {
        Self::None
    }
}

/// Metadata for each `LinkDecor` variant.
/// Java fields: `margin`, `fill`, `arrowSize`.
#[derive(Debug, Clone, Copy)]
pub struct LinkDecorMeta {
    pub margin: i32,
    pub fill: bool,
    pub arrow_size: f64,
}

impl LinkDecor {
    /// Return metadata (margin, fill, arrow_size) matching the Java enum fields.
    pub fn meta(self) -> LinkDecorMeta {
        use LinkDecor::*;
        let (margin, fill, arrow_size) = match self {
            None                => (2,  false, 0.0),
            Extends             => (30, false, 2.0),
            Composition         => (15, true,  1.3),
            Agregation          => (15, false, 1.3),
            NotNavigable        => (1,  false, 0.5),
            Redefines           => (30, false, 2.0),
            DefinedBy           => (30, false, 2.0),
            Crowfoot            => (10, true,  0.8),
            CircleCrowfoot      => (14, false, 0.8),
            CircleLine          => (10, false, 0.8),
            DoubleLine          => (7,  false, 0.7),
            LineCrowfoot        => (10, false, 0.8),
            Arrow               => (10, true,  0.5),
            ArrowTriangle       => (10, true,  0.8),
            ArrowAndCircle      => (10, false, 0.5),
            Circle              => (0,  false, 0.5),
            CircleFill          => (0,  false, 0.5),
            CircleConnect       => (0,  false, 0.5),
            Parenthesis         => (0,  false, 1.0),
            Square              => (0,  false, 0.5),
            CircleCross         => (0,  false, 0.5),
            Plus                => (0,  false, 1.5),
            HalfArrowUp         => (0,  false, 1.5),
            HalfArrowDown       => (0,  false, 1.5),
            SquareToBeRemoved   => (30, false, 0.0),
        };
        LinkDecorMeta { margin, fill, arrow_size }
    }

    /// Shorthand for `self.meta().margin`.
    pub fn margin(self) -> i32 {
        self.meta().margin
    }

    /// Shorthand for `self.meta().fill`.
    pub fn is_fill(self) -> bool {
        self.meta().fill
    }

    /// Shorthand for `self.meta().arrow_size`.
    pub fn arrow_size(self) -> f64 {
        self.meta().arrow_size
    }

    /// Whether this decoration draws an extends-like triangle head.
    /// Java: `isExtendsLike()`
    pub fn is_extends_like(self) -> bool {
        matches!(self, Self::Extends | Self::Redefines | Self::DefinedBy)
    }

    // ── Decor lookup tables (matching Java's DECORS1 / DECORS2) ──

    /// Lookup a left/start-side decoration from its text symbol.
    /// Java: `lookupDecors1(String)`
    pub fn lookup_decors1(s: &str) -> Self {
        DECORS1.get(s.trim()).copied().unwrap_or(Self::None)
    }

    /// Lookup a right/end-side decoration from its text symbol.
    /// Java: `lookupDecors2(String)`
    pub fn lookup_decors2(s: &str) -> Self {
        DECORS2.get(s.trim()).copied().unwrap_or(Self::None)
    }

    /// Build a regex alternation matching all known left-side decor symbols,
    /// longest-first to avoid prefix conflicts.
    /// Java: `getRegexDecors1()`
    pub fn regex_decors1() -> String {
        build_regex_from_keys(&DECORS1)
    }

    /// Build a regex alternation matching all known right-side decor symbols,
    /// longest-first to avoid prefix conflicts.
    /// Java: `getRegexDecors2()`
    pub fn regex_decors2() -> String {
        build_regex_from_keys(&DECORS2)
    }

    /// Return the extremity factory kind for this decoration.
    /// Java: `getExtremityFactoryComplete` / `getExtremityFactoryLegacy`
    ///
    /// Returns `None` for `LinkDecor::None` and `SquareToBeRemoved`.
    pub fn extremity_kind(self) -> Option<ExtremityKind> {
        use LinkDecor::*;
        match self {
            Extends         => Some(ExtremityKind::Triangle { w: 18.0, h: 6.0, len: 18.0 }),
            Redefines       => Some(ExtremityKind::ExtendsLike { has_dot: false }),
            DefinedBy       => Some(ExtremityKind::ExtendsLike { has_dot: true }),
            Plus            => Some(ExtremityKind::Plus),
            HalfArrowUp     => Some(ExtremityKind::HalfArrow { direction: 1 }),
            HalfArrowDown   => Some(ExtremityKind::HalfArrow { direction: -1 }),
            ArrowTriangle   => Some(ExtremityKind::Triangle { w: 8.0, h: 3.0, len: 8.0 }),
            Crowfoot        => Some(ExtremityKind::Crowfoot),
            CircleCrowfoot  => Some(ExtremityKind::CircleCrowfoot),
            LineCrowfoot    => Some(ExtremityKind::LineCrowfoot),
            CircleLine      => Some(ExtremityKind::CircleLine),
            DoubleLine      => Some(ExtremityKind::DoubleLine),
            CircleCross     => Some(ExtremityKind::CircleCross),
            Arrow           => Some(ExtremityKind::Arrow),
            ArrowAndCircle  => Some(ExtremityKind::ArrowAndCircle),
            NotNavigable    => Some(ExtremityKind::NotNavigable),
            Agregation      => Some(ExtremityKind::Diamond { filled: false }),
            Composition     => Some(ExtremityKind::Diamond { filled: true }),
            Circle          => Some(ExtremityKind::Circle { filled: false }),
            CircleFill      => Some(ExtremityKind::Circle { filled: true }),
            Square          => Some(ExtremityKind::Square),
            Parenthesis     => Some(ExtremityKind::Parenthesis),
            CircleConnect   => Some(ExtremityKind::CircleConnect),
            None | SquareToBeRemoved => Option::None,
        }
    }
}

/// Extremity factory discriminant - describes *what shape* to draw at the
/// arrow endpoint. Each variant carries the parameters the Java
/// `ExtremityFactory*` constructors receive.
///
/// Consumers (svg_render, dot export, etc.) pattern-match on this to produce
/// the appropriate drawing commands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtremityKind {
    Arrow,
    ArrowAndCircle,
    Circle { filled: bool },
    CircleConnect,
    CircleCross,
    CircleCrowfoot,
    CircleLine,
    Crowfoot,
    Diamond { filled: bool },
    DoubleLine,
    ExtendsLike { has_dot: bool },
    HalfArrow { direction: i32 },
    LineCrowfoot,
    NotNavigable,
    Parenthesis,
    Plus,
    Square,
    Triangle { w: f64, h: f64, len: f64 },
}

// ── Lookup tables ──────────────────────────────────────────────────

/// Left/start decoration symbols -> LinkDecor.
/// Populated from the Java enum's `decors1` arrays.
static DECORS1: LazyLock<HashMap<&'static str, LinkDecor>> = LazyLock::new(|| {
    let entries: &[(&[&str], LinkDecor)] = &[
        (&["<|", "^"],      LinkDecor::Extends),
        (&["*"],             LinkDecor::Composition),
        (&["o"],             LinkDecor::Agregation),
        (&["x"],             LinkDecor::NotNavigable),
        (&["<||"],           LinkDecor::Redefines),
        (&["<|:"],           LinkDecor::DefinedBy),
        (&["}"],             LinkDecor::Crowfoot),
        (&["}o"],            LinkDecor::CircleCrowfoot),
        (&["|o"],            LinkDecor::CircleLine),
        (&["||"],            LinkDecor::DoubleLine),
        (&["}|"],            LinkDecor::LineCrowfoot),
        (&["<", "<_"],       LinkDecor::Arrow),
        (&["<<"],            LinkDecor::ArrowTriangle),
        (&["0"],             LinkDecor::Circle),
        (&["@"],             LinkDecor::CircleFill),
        (&["0)"],            LinkDecor::CircleConnect),
        (&[")"],             LinkDecor::Parenthesis),
        (&["#"],             LinkDecor::Square),
        (&["+"],             LinkDecor::Plus),
    ];
    let mut m = HashMap::new();
    for (keys, decor) in entries {
        for k in *keys {
            m.insert(*k, *decor);
        }
    }
    m
});

/// Right/end decoration symbols -> LinkDecor.
/// Populated from the Java enum's `decors2` arrays.
static DECORS2: LazyLock<HashMap<&'static str, LinkDecor>> = LazyLock::new(|| {
    let entries: &[(&[&str], LinkDecor)] = &[
        (&["|>", "^"],      LinkDecor::Extends),
        (&["*"],             LinkDecor::Composition),
        (&["o"],             LinkDecor::Agregation),
        (&["x"],             LinkDecor::NotNavigable),
        (&["||>"],           LinkDecor::Redefines),
        (&[":|>"],           LinkDecor::DefinedBy),
        (&["{"],             LinkDecor::Crowfoot),
        (&["o{"],            LinkDecor::CircleCrowfoot),
        (&["o|"],            LinkDecor::CircleLine),
        (&["||"],            LinkDecor::DoubleLine),
        (&["|{"],            LinkDecor::LineCrowfoot),
        (&[">", "_>"],       LinkDecor::Arrow),
        (&[">>"],            LinkDecor::ArrowTriangle),
        (&["0"],             LinkDecor::Circle),
        (&["@"],             LinkDecor::CircleFill),
        (&["(0"],            LinkDecor::CircleConnect),
        (&["("],             LinkDecor::Parenthesis),
        (&["#"],             LinkDecor::Square),
        (&["+"],             LinkDecor::Plus),
        (&["\\\\"],          LinkDecor::HalfArrowUp),
        (&["//"],            LinkDecor::HalfArrowDown),
    ];
    let mut m = HashMap::new();
    for (keys, decor) in entries {
        for k in *keys {
            m.insert(*k, *decor);
        }
    }
    m
});

/// Build a regex alternation from a lookup table's keys, sorted longest-first
/// to prevent prefix conflicts. Keys starting/ending with 'o' get `\b` word
/// boundaries appended/prepended.
/// Java: `buildRegexFromDecorKeys(Set<String>)`
fn build_regex_from_keys(map: &HashMap<&str, LinkDecor>) -> String {
    let mut keys: Vec<&str> = map.keys().copied().collect();
    keys.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));

    let parts: Vec<String> = keys
        .iter()
        .map(|k| {
            let quoted = regex::escape(k);
            let starts_o = k.starts_with('o');
            let ends_o = k.ends_with('o');
            match (starts_o, ends_o) {
                (true, true) => format!("\\b{}\\b", quoted),
                (true, false) => format!("\\b{}", quoted),
                (false, true) => format!("{}\\b", quoted),
                _ => quoted,
            }
        })
        .collect();

    format!("({})?", parts.join("|"))
}

// ── LinkMiddleDecor ──────────────────────────────────────────────────

/// Middle decoration on a link.
/// Java: `decoration.LinkMiddleDecor`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LinkMiddleDecor {
    #[default]
    None,
    Circle,
    CircleCircled,
    CircleCircled1,
    CircleCircled2,
    Subset,
    Superset,
}

impl LinkMiddleDecor {
    /// Return the inversed middle decoration (swap Circled1 <-> Circled2).
    /// Java: `getInversed()`
    pub fn inversed(self) -> Self {
        match self {
            Self::CircleCircled1 => Self::CircleCircled2,
            Self::CircleCircled2 => Self::CircleCircled1,
            other => other,
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(LinkDecor::default(), LinkDecor::None);
        assert_eq!(LinkMiddleDecor::default(), LinkMiddleDecor::None);
    }

    // ── Metadata ──

    #[test]
    fn extends_meta() {
        let m = LinkDecor::Extends.meta();
        assert_eq!(m.margin, 30);
        assert!(!m.fill);
        assert_eq!(m.arrow_size, 2.0);
    }

    #[test]
    fn composition_meta() {
        let m = LinkDecor::Composition.meta();
        assert_eq!(m.margin, 15);
        assert!(m.fill);
        assert_eq!(m.arrow_size, 1.3);
    }

    #[test]
    fn arrow_meta() {
        let m = LinkDecor::Arrow.meta();
        assert_eq!(m.margin, 10);
        assert!(m.fill);
        assert_eq!(m.arrow_size, 0.5);
    }

    #[test]
    fn none_meta() {
        let m = LinkDecor::None.meta();
        assert_eq!(m.margin, 2);
        assert!(!m.fill);
        assert_eq!(m.arrow_size, 0.0);
    }

    // ── Extends-like ──

    #[test]
    fn is_extends_like() {
        assert!(LinkDecor::Extends.is_extends_like());
        assert!(LinkDecor::Redefines.is_extends_like());
        assert!(LinkDecor::DefinedBy.is_extends_like());
        assert!(!LinkDecor::Arrow.is_extends_like());
        assert!(!LinkDecor::None.is_extends_like());
    }

    // ── Lookup tables ──

    #[test]
    fn lookup_decors1_arrow() {
        assert_eq!(LinkDecor::lookup_decors1("<"), LinkDecor::Arrow);
        assert_eq!(LinkDecor::lookup_decors1("<_"), LinkDecor::Arrow);
    }

    #[test]
    fn lookup_decors1_extends() {
        assert_eq!(LinkDecor::lookup_decors1("<|"), LinkDecor::Extends);
        assert_eq!(LinkDecor::lookup_decors1("^"), LinkDecor::Extends);
    }

    #[test]
    fn lookup_decors2_arrow() {
        assert_eq!(LinkDecor::lookup_decors2(">"), LinkDecor::Arrow);
        assert_eq!(LinkDecor::lookup_decors2("_>"), LinkDecor::Arrow);
    }

    #[test]
    fn lookup_decors2_extends() {
        assert_eq!(LinkDecor::lookup_decors2("|>"), LinkDecor::Extends);
        assert_eq!(LinkDecor::lookup_decors2("^"), LinkDecor::Extends);
    }

    #[test]
    fn lookup_decors1_composition() {
        assert_eq!(LinkDecor::lookup_decors1("*"), LinkDecor::Composition);
    }

    #[test]
    fn lookup_decors2_crowfoot_variants() {
        assert_eq!(LinkDecor::lookup_decors2("{"), LinkDecor::Crowfoot);
        assert_eq!(LinkDecor::lookup_decors2("o{"), LinkDecor::CircleCrowfoot);
        assert_eq!(LinkDecor::lookup_decors2("|{"), LinkDecor::LineCrowfoot);
        assert_eq!(LinkDecor::lookup_decors2("o|"), LinkDecor::CircleLine);
    }

    #[test]
    fn lookup_decors1_crowfoot_variants() {
        assert_eq!(LinkDecor::lookup_decors1("}"), LinkDecor::Crowfoot);
        assert_eq!(LinkDecor::lookup_decors1("}o"), LinkDecor::CircleCrowfoot);
        assert_eq!(LinkDecor::lookup_decors1("}|"), LinkDecor::LineCrowfoot);
        assert_eq!(LinkDecor::lookup_decors1("|o"), LinkDecor::CircleLine);
    }

    #[test]
    fn lookup_decors_unknown_returns_none() {
        assert_eq!(LinkDecor::lookup_decors1("???"), LinkDecor::None);
        assert_eq!(LinkDecor::lookup_decors2("???"), LinkDecor::None);
    }

    #[test]
    fn lookup_decors_trims_whitespace() {
        assert_eq!(LinkDecor::lookup_decors1(" < "), LinkDecor::Arrow);
        assert_eq!(LinkDecor::lookup_decors2(" > "), LinkDecor::Arrow);
    }

    #[test]
    fn lookup_decors2_half_arrows() {
        assert_eq!(LinkDecor::lookup_decors2("\\\\"), LinkDecor::HalfArrowUp);
        assert_eq!(LinkDecor::lookup_decors2("//"), LinkDecor::HalfArrowDown);
    }

    #[test]
    fn lookup_decors_redefines_definedby() {
        assert_eq!(LinkDecor::lookup_decors1("<||"), LinkDecor::Redefines);
        assert_eq!(LinkDecor::lookup_decors2("||>"), LinkDecor::Redefines);
        assert_eq!(LinkDecor::lookup_decors1("<|:"), LinkDecor::DefinedBy);
        assert_eq!(LinkDecor::lookup_decors2(":|>"), LinkDecor::DefinedBy);
    }

    #[test]
    fn lookup_decors_circle_variants() {
        assert_eq!(LinkDecor::lookup_decors1("0"), LinkDecor::Circle);
        assert_eq!(LinkDecor::lookup_decors1("@"), LinkDecor::CircleFill);
        assert_eq!(LinkDecor::lookup_decors1("0)"), LinkDecor::CircleConnect);
        assert_eq!(LinkDecor::lookup_decors2("(0"), LinkDecor::CircleConnect);
    }

    #[test]
    fn lookup_decors_parenthesis() {
        assert_eq!(LinkDecor::lookup_decors1(")"), LinkDecor::Parenthesis);
        assert_eq!(LinkDecor::lookup_decors2("("), LinkDecor::Parenthesis);
    }

    #[test]
    fn lookup_decors_square_plus() {
        assert_eq!(LinkDecor::lookup_decors1("#"), LinkDecor::Square);
        assert_eq!(LinkDecor::lookup_decors2("#"), LinkDecor::Square);
        assert_eq!(LinkDecor::lookup_decors1("+"), LinkDecor::Plus);
        assert_eq!(LinkDecor::lookup_decors2("+"), LinkDecor::Plus);
    }

    // ── Regex ──

    #[test]
    fn regex_decors1_is_valid() {
        let pat = LinkDecor::regex_decors1();
        // Should be a valid regex and contain known symbols
        assert!(pat.starts_with('('));
        assert!(pat.ends_with(")?"));
        // Should be parseable
        regex::Regex::new(&pat).expect("regex_decors1 should produce valid regex");
    }

    #[test]
    fn regex_decors2_is_valid() {
        let pat = LinkDecor::regex_decors2();
        assert!(pat.starts_with('('));
        assert!(pat.ends_with(")?"));
        regex::Regex::new(&pat).expect("regex_decors2 should produce valid regex");
    }

    // ── Extremity kind ──

    #[test]
    fn extremity_none_for_none() {
        assert!(LinkDecor::None.extremity_kind().is_none());
    }

    #[test]
    fn extremity_extends_triangle() {
        match LinkDecor::Extends.extremity_kind() {
            Some(ExtremityKind::Triangle { w, h, len }) => {
                assert_eq!(w, 18.0);
                assert_eq!(h, 6.0);
                assert_eq!(len, 18.0);
            }
            other => panic!("Expected Triangle, got {:?}", other),
        }
    }

    #[test]
    fn extremity_arrow_triangle_smaller() {
        match LinkDecor::ArrowTriangle.extremity_kind() {
            Some(ExtremityKind::Triangle { w, h, len }) => {
                assert_eq!(w, 8.0);
                assert_eq!(h, 3.0);
                assert_eq!(len, 8.0);
            }
            other => panic!("Expected Triangle, got {:?}", other),
        }
    }

    #[test]
    fn extremity_diamond_variants() {
        assert_eq!(
            LinkDecor::Composition.extremity_kind(),
            Some(ExtremityKind::Diamond { filled: true })
        );
        assert_eq!(
            LinkDecor::Agregation.extremity_kind(),
            Some(ExtremityKind::Diamond { filled: false })
        );
    }

    #[test]
    fn extremity_circle_variants() {
        assert_eq!(
            LinkDecor::Circle.extremity_kind(),
            Some(ExtremityKind::Circle { filled: false })
        );
        assert_eq!(
            LinkDecor::CircleFill.extremity_kind(),
            Some(ExtremityKind::Circle { filled: true })
        );
    }

    #[test]
    fn extremity_half_arrow() {
        assert_eq!(
            LinkDecor::HalfArrowUp.extremity_kind(),
            Some(ExtremityKind::HalfArrow { direction: 1 })
        );
        assert_eq!(
            LinkDecor::HalfArrowDown.extremity_kind(),
            Some(ExtremityKind::HalfArrow { direction: -1 })
        );
    }

    // ── LinkMiddleDecor ──

    #[test]
    fn middle_decor_inversed() {
        assert_eq!(LinkMiddleDecor::CircleCircled1.inversed(), LinkMiddleDecor::CircleCircled2);
        assert_eq!(LinkMiddleDecor::CircleCircled2.inversed(), LinkMiddleDecor::CircleCircled1);
        assert_eq!(LinkMiddleDecor::Circle.inversed(), LinkMiddleDecor::Circle);
        assert_eq!(LinkMiddleDecor::None.inversed(), LinkMiddleDecor::None);
        assert_eq!(LinkMiddleDecor::Subset.inversed(), LinkMiddleDecor::Subset);
        assert_eq!(LinkMiddleDecor::Superset.inversed(), LinkMiddleDecor::Superset);
    }

    // ── Comprehensive variant coverage ──

    #[test]
    fn all_variants_have_meta() {
        let variants = [
            LinkDecor::None,
            LinkDecor::Extends,
            LinkDecor::Composition,
            LinkDecor::Agregation,
            LinkDecor::NotNavigable,
            LinkDecor::Redefines,
            LinkDecor::DefinedBy,
            LinkDecor::Crowfoot,
            LinkDecor::CircleCrowfoot,
            LinkDecor::CircleLine,
            LinkDecor::DoubleLine,
            LinkDecor::LineCrowfoot,
            LinkDecor::Arrow,
            LinkDecor::ArrowTriangle,
            LinkDecor::ArrowAndCircle,
            LinkDecor::Circle,
            LinkDecor::CircleFill,
            LinkDecor::CircleConnect,
            LinkDecor::Parenthesis,
            LinkDecor::Square,
            LinkDecor::CircleCross,
            LinkDecor::Plus,
            LinkDecor::HalfArrowUp,
            LinkDecor::HalfArrowDown,
            LinkDecor::SquareToBeRemoved,
        ];
        for v in &variants {
            let _ = v.meta(); // should not panic
        }
    }
}
