//! Font metrics computed at runtime from embedded DejaVu .ttf files.
//!
//! Replaces the previous 47000-line static lookup table with runtime
//! computation via `ttf-parser`.  Font metric values match Java PlantUML
//! exactly (same font files, same math: `raw_units / units_per_em * size`).

use std::sync::LazyLock;

// ── Embedded font data ──────────────────────────────────────────────────

static DEJAVU_SANS_DATA: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
static DEJAVU_SANS_BOLD_DATA: &[u8] = include_bytes!("../fonts/DejaVuSans-Bold.ttf");
static DEJAVU_MONO_DATA: &[u8] = include_bytes!("../fonts/DejaVuSansMono.ttf");
static DEJAVU_MONO_BOLD_DATA: &[u8] = include_bytes!("../fonts/DejaVuSansMono-Bold.ttf");

// ── Parsed font faces (lazy, parsed once) ───────────────────────────────

struct Fonts {
    sans: ttf_parser::Face<'static>,
    sans_bold: ttf_parser::Face<'static>,
    mono: ttf_parser::Face<'static>,
    mono_bold: ttf_parser::Face<'static>,
}

static FONTS: LazyLock<Fonts> = LazyLock::new(|| Fonts {
    sans: ttf_parser::Face::parse(DEJAVU_SANS_DATA, 0).expect("DejaVuSans.ttf"),
    sans_bold: ttf_parser::Face::parse(DEJAVU_SANS_BOLD_DATA, 0).expect("DejaVuSans-Bold.ttf"),
    mono: ttf_parser::Face::parse(DEJAVU_MONO_DATA, 0).expect("DejaVuSansMono.ttf"),
    mono_bold: ttf_parser::Face::parse(DEJAVU_MONO_BOLD_DATA, 0).expect("DejaVuSansMono-Bold.ttf"),
});

// ── Font family resolution ──────────────────────────────────────────────

/// Map a logical font family name to a canonical key.
/// Java logical fonts: "SansSerif"/"Dialog"→ DejaVu Sans, "Monospaced"/"Courier"→ DejaVu Sans Mono.
/// Physical fonts not installed on the reference machine (e.g. "Courier New", "Arial")
/// fall back to Dialog (sans-serif) in Java AWT.
/// For CSS `font-family` lists like "Courier New,monospace", we resolve based on
/// the PRIMARY (first) name — Java AWT uses the first name for font lookup.
fn resolve_face(family: &str, bold: bool) -> &'static ttf_parser::Face<'static> {
    let fonts = &*FONTS;
    // Use the first name in a CSS comma-separated font-family list
    let primary = family.split(',').next().unwrap_or(family).trim();
    let p = primary.to_lowercase();
    // Java logical font "Monospaced" and its alias "Courier" (without "New") map to mono.
    // CSS generic "monospace" also maps to mono.
    // "Courier New" is a physical font — uninstalled on reference machine → Dialog fallback.
    let is_mono = p == "monospaced" || p == "monospace" || p == "courier";
    if is_mono {
        if bold {
            &fonts.mono_bold
        } else {
            &fonts.mono
        }
    } else if bold {
        &fonts.sans_bold
    } else {
        &fonts.sans
    }
}

// ── Public API (signatures preserved from previous implementation) ───────

/// Width of a single character in the given font configuration.
///
/// Computes `glyph_hor_advance / units_per_em * size`, matching Java's
/// `font.getStringBounds(ch, frc).getWidth()` with `FRACTIONALMETRICS_ON`.
pub fn char_width(ch: char, family: &str, size: f64, bold: bool, _italic: bool) -> f64 {
    if ch == '\n' || ch == '\r' {
        return 0.0;
    }
    let face = resolve_face(family, bold);
    let upem = face.units_per_em() as f64;
    if let Some(gid) = face.glyph_index(ch) {
        if let Some(adv) = face.glyph_hor_advance(gid) {
            return adv as f64 / upem * size;
        }
    }
    // Fallback: use space advance for unmapped characters
    if let Some(sp_gid) = face.glyph_index(' ') {
        if let Some(sp_adv) = face.glyph_hor_advance(sp_gid) {
            return sp_adv as f64 / upem * size;
        }
    }
    size * 0.6 // last-resort fallback
}

/// Total width of a text string (sum of character advances).
pub fn text_width(text: &str, family: &str, size: f64, bold: bool, italic: bool) -> f64 {
    text.chars()
        .map(|c| char_width(c, family, size, bold, italic))
        .sum()
}

/// Line height = ascent + |descent| (leading is 0 for DejaVu fonts).
///
/// Matches Java's `LineMetrics.getHeight()`.
pub fn line_height(family: &str, size: f64, _bold: bool, _italic: bool) -> f64 {
    let face = resolve_face(family, false); // vertical metrics are style-independent
    let upem = face.units_per_em() as f64;
    let asc = face.ascender() as f64; // positive (hhea.ascender)
    let desc = face.descender().unsigned_abs() as f64; // make positive
    (asc + desc) / upem * size
}

/// Font ascent (baseline to top of tallest glyph).
///
/// Matches Java's `LineMetrics.getAscent()`.
pub fn ascent(family: &str, size: f64, _bold: bool, _italic: bool) -> f64 {
    let face = resolve_face(family, false);
    face.ascender() as f64 / face.units_per_em() as f64 * size
}

/// Font descent (baseline to bottom of lowest glyph).
///
/// Matches Java's `LineMetrics.getDescent()` (positive value).
pub fn descent(family: &str, size: f64, _bold: bool, _italic: bool) -> f64 {
    let face = resolve_face(family, false);
    face.descender().unsigned_abs() as f64 / face.units_per_em() as f64 * size
}

/// OS/2 typographic ascent. Used for DOT cluster label dimensions which match
/// Java's `StringBounder.calculateDimension()` text block height.
pub fn typo_ascent(family: &str, size: f64, _bold: bool, _italic: bool) -> f64 {
    let face = resolve_face(family, false);
    let upem = face.units_per_em() as f64;
    // OS/2 sTypoAscender; fallback to hhea ascender if unavailable
    let typo_asc = face.typographic_ascender().unwrap_or(face.ascender()) as f64;
    typo_asc / upem * size
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Java ground truth (FRACTIONALMETRICS_ON, DejaVu Sans):
    // SansSerif 12 PLAIN: ascent=11.1386718750 descent=2.8300781250 height=13.9687500000
    // SansSerif 13 PLAIN: ascent=12.0668945313 descent=3.0659179688 height=15.1328125000
    // SansSerif 18 PLAIN: ascent=16.7080078125 descent=4.2451171875 height=20.9531250000
    // charW('W') at 12 = 11.8652343750
    // width('foo1') at 12 = 26.5429687500

    #[test]
    fn ascent_matches_java() {
        let a12 = ascent("SansSerif", 12.0, false, false);
        let a13 = ascent("SansSerif", 13.0, false, false);
        let a18 = ascent("SansSerif", 18.0, false, false);
        assert!((a12 - 11.1386718750).abs() < 1e-6, "a12={a12}");
        assert!((a13 - 12.0668945313).abs() < 1e-6, "a13={a13}");
        assert!((a18 - 16.7080078125).abs() < 1e-6, "a18={a18}");
    }

    #[test]
    fn descent_matches_java() {
        let d12 = descent("SansSerif", 12.0, false, false);
        assert!((d12 - 2.8300781250).abs() < 1e-6, "d12={d12}");
    }

    #[test]
    fn line_height_matches_java() {
        let h12 = line_height("SansSerif", 12.0, false, false);
        let h13 = line_height("SansSerif", 13.0, false, false);
        let h18 = line_height("SansSerif", 18.0, false, false);
        assert!((h12 - 13.9687500000).abs() < 1e-6, "h12={h12}");
        assert!((h13 - 15.1328125000).abs() < 1e-6, "h13={h13}");
        assert!((h18 - 20.9531250000).abs() < 1e-6, "h18={h18}");
    }

    #[test]
    fn char_width_w_matches_java() {
        let w = char_width('W', "SansSerif", 12.0, false, false);
        assert!((w - 11.8652343750).abs() < 1e-6, "W width={w}");
    }

    #[test]
    fn text_width_foo1_matches_java() {
        let w = text_width("foo1", "SansSerif", 12.0, false, false);
        assert!((w - 26.5429687500).abs() < 1e-4, "foo1 width={w}");
    }

    #[test]
    fn monospaced_metrics() {
        // All monospaced chars should have equal advance width
        let w_a = char_width('a', "Monospaced", 13.0, false, false);
        let w_w = char_width('W', "Monospaced", 13.0, false, false);
        assert!(
            (w_a - w_w).abs() < 1e-6,
            "mono: a={w_a} W={w_w} should be equal"
        );
    }

    #[test]
    fn bold_width_differs() {
        let w_plain = char_width('W', "SansSerif", 12.0, false, false);
        let w_bold = char_width('W', "SansSerif", 12.0, true, false);
        assert!(w_bold > w_plain, "bold W should be wider");
    }

    #[test]
    fn family_resolution() {
        // "Monospaced" (Java logical name) resolves to mono font
        let w_mono = char_width('a', "Monospaced", 12.0, false, false);
        let w_monospace = char_width('a', "monospace", 12.0, false, false);
        assert!((w_mono - w_monospace).abs() < 1e-10);
        // "Courier" (Java logical font, no "New") maps to Monospaced
        let w_courier = char_width('a', "Courier", 12.0, false, false);
        assert!((w_mono - w_courier).abs() < 1e-10, "Courier maps to mono");
        // "Courier New" is a physical font not installed on reference machine
        // → Java Dialog fallback → sans-serif
        let w_courier_new = char_width('a', "Courier New", 12.0, false, false);
        let w_sans = char_width('a', "SansSerif", 12.0, false, false);
        assert!(
            (w_courier_new - w_sans).abs() < 1e-10,
            "Courier New maps to sans (Dialog fallback)"
        );
        // "SansSerif", "Dialog", "Arial" all resolve to sans font
        let w3 = char_width('a', "SansSerif", 12.0, false, false);
        let w4 = char_width('a', "Dialog", 12.0, false, false);
        assert!((w3 - w4).abs() < 1e-10);
    }

    #[test]
    fn arbitrary_size_works() {
        // Size 15 was not in the old lookup table — runtime computation handles any size
        let h = line_height("SansSerif", 15.0, false, false);
        assert!(h > 0.0);
        assert!((h - (1901.0 + 483.0) / 2048.0 * 15.0).abs() < 1e-6);
    }

    #[test]
    fn text_width_matches_java_reference() {
        // Verify text_width matches Java PlantUML's getStringBounds for various strings
        let cases: &[(&str, f64, bool, f64)] = &[
            ("Alice", 14.0, false, 33.667),
            ("Bob", 14.0, false, 27.0566),
            ("Hello", 13.0, false, 32.9507),
            ("Test", 14.0, false, 29.9482),
            ("Grouping messages", 13.0, true, 144.5869),
            ("Swimlane1", 18.0, false, 98.6484),
            ("Action 1", 12.0, false, 49.2422),
        ];
        for (text, size, bold, java_w) in cases {
            let our_w = text_width(text, "SansSerif", *size, *bold, false);
            assert!(
                (our_w - java_w).abs() < 0.001,
                "text_width(\"{text}\", size={size}, bold={bold}): ours={our_w:.4}, java={java_w:.4}"
            );
        }
    }
}
