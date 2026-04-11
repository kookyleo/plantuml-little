use crate::style::compat::{normalize_color, parse_skinparams, SkinParams, Theme};

// ── Color normalization tests ────────────────────────────────────

#[test]
fn normalize_hex6_passthrough() {
    assert_eq!(normalize_color("#FEFECE"), "#FEFECE");
}

#[test]
fn normalize_hex3_expand() {
    assert_eq!(normalize_color("#F0C"), "#FF00CC");
}

#[test]
fn normalize_hex8_drop_alpha() {
    assert_eq!(normalize_color("#80FF0000"), "#FF0000");
}

#[test]
fn normalize_transparent() {
    assert_eq!(normalize_color("transparent"), "none");
}

#[test]
fn normalize_transparent_case_insensitive() {
    assert_eq!(normalize_color("Transparent"), "none");
    assert_eq!(normalize_color("TRANSPARENT"), "none");
}

#[test]
fn normalize_named_color_to_hex() {
    assert_eq!(normalize_color("red"), "#FF0000");
    assert_eq!(normalize_color("LightBlue"), "#ADD8E6");
    assert_eq!(normalize_color("DarkGreen"), "#006400");
}

#[test]
fn normalize_whitespace_trimmed() {
    assert_eq!(normalize_color("  #FFF  "), "#FFFFFF");
    assert_eq!(normalize_color("  red  "), "#FF0000");
}

// ── Skinparam parsing tests ────────────────────────────────────

#[test]
fn parse_single_line_skinparam() {
    let src = "skinparam BackgroundColor #FEFECE\nclass Foo";
    let params = parse_skinparams(src);
    assert_eq!(params.get("backgroundcolor"), Some("#FEFECE"));
}

#[test]
fn parse_single_line_element_skinparam() {
    let src = "skinparam ClassBackgroundColor #FEFECE";
    let params = parse_skinparams(src);
    assert_eq!(params.get("classbackgroundcolor"), Some("#FEFECE"));
}

#[test]
fn parse_element_block() {
    let src = "skinparam class {\n  BackgroundColor #FEFECE\n  BorderColor #A80036\n}";
    let params = parse_skinparams(src);
    assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
    assert_eq!(params.get("class.bordercolor"), Some("#A80036"));
}

#[test]
fn parse_nested_block() {
    let src = "skinparam {\n  class {\n    BackgroundColor #FEFECE\n  }\n}";
    let params = parse_skinparams(src);
    assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
}

#[test]
fn parse_nested_block_with_global_params() {
    let src =
        "skinparam {\n  BackgroundColor #FFFFFF\n  class {\n    BackgroundColor #FEFECE\n  }\n}";
    let params = parse_skinparams(src);
    assert_eq!(params.get("backgroundcolor"), Some("#FFFFFF"));
    assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
}

#[test]
fn parse_multiple_skinparam_lines() {
    let src =
        "skinparam BackgroundColor #FEFECE\nskinparam ArrowColor #A80036\nskinparam FontColor black";
    let params = parse_skinparams(src);
    assert_eq!(params.get("backgroundcolor"), Some("#FEFECE"));
    assert_eq!(params.get("arrowcolor"), Some("#A80036"));
    assert_eq!(params.get("fontcolor"), Some("#000000"));
}

#[test]
fn parse_skinparam_case_insensitive_lookup() {
    let src = "skinparam ClassBackgroundColor #FEFECE";
    let params = parse_skinparams(src);
    assert_eq!(params.get("ClassBackgroundColor"), Some("#FEFECE"));
    assert_eq!(params.get("classbackgroundcolor"), Some("#FEFECE"));
    assert_eq!(params.get("CLASSBACKGROUNDCOLOR"), Some("#FEFECE"));
}

#[test]
fn parse_skinparam_ignores_non_skinparam_lines() {
    let src = "class Foo\ninterface Bar\nskinparam ArrowColor red\nFoo --> Bar";
    let params = parse_skinparams(src);
    assert_eq!(params.len(), 1);
    assert_eq!(params.get("arrowcolor"), Some("#FF0000"));
}

#[test]
fn parse_skinparam_skips_style_blocks() {
    let src = "<style>\nskinparam Foo bar\n</style>\nskinparam ArrowColor red";
    let params = parse_skinparams(src);
    assert_eq!(params.len(), 1);
    assert_eq!(params.get("arrowcolor"), Some("#FF0000"));
}

#[test]
fn parse_skinparam_color_normalization() {
    let src = "skinparam BackgroundColor transparent\nskinparam BorderColor #F00";
    let params = parse_skinparams(src);
    assert_eq!(params.get("backgroundcolor"), Some("none"));
    assert_eq!(params.get("bordercolor"), Some("#FF0000"));
}

#[test]
fn parse_skinparam_inline_block() {
    let src = "skinparam class { BackgroundColor #FEFECE BorderColor #A80036 }";
    let params = parse_skinparams(src);
    assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
    assert_eq!(params.get("class.bordercolor"), Some("#A80036"));
}

#[test]
fn parse_stereotype_keyed_block() {
    // C4 stdlib stores container styling as rectangle<<container>> { ... }.
    let src = concat!(
        "skinparam rectangle<<container>> {\n",
        "    FontColor #FFFFFF\n",
        "    BackgroundColor #438DD5\n",
        "    BorderColor #3C7FC0\n",
        "}\n",
    );
    let params = parse_skinparams(src);
    assert_eq!(
        params.get("rectangle<<container>>.fontcolor"),
        Some("#FFFFFF")
    );
    assert_eq!(
        params.get("rectangle<<container>>.backgroundcolor"),
        Some("#438DD5")
    );
    assert_eq!(
        params.background_color_for("rectangle", &["container"], "#000000"),
        "#438DD5"
    );
}

#[test]
fn parse_stereotype_chained_inline_blocks() {
    // C4 stdlib emits chained inline blocks on a single line. The
    // normalization step should split these into individual statements.
    let src = "skinparam rectangle<<container>> {    FontColor #FFFFFF    BackgroundColor #438DD5    BorderColor #3C7FC0}skinparam database<<container>> {    FontColor #FFFFFF    BackgroundColor #438DD5    BorderColor #3C7FC0}";
    let params = parse_skinparams(src);
    assert_eq!(
        params.get("rectangle<<container>>.backgroundcolor"),
        Some("#438DD5")
    );
    assert_eq!(
        params.get("database<<container>>.backgroundcolor"),
        Some("#438DD5")
    );
}

#[test]
fn parse_stereotype_boundary_block_dashed() {
    let src = "skinparam rectangle<<system_boundary>> {    FontColor #444444    BackgroundColor transparent    BorderColor #444444    BorderStyle dashed}";
    let params = parse_skinparams(src);
    assert_eq!(
        params.border_color_for("rectangle", &["system_boundary"], "#181818"),
        "#444444"
    );
    assert_eq!(
        params.border_style_for("rectangle", &["system_boundary"]),
        Some("dashed")
    );
}

#[test]
fn parse_empty_source() {
    let params = parse_skinparams("");
    assert!(params.is_empty());
    assert_eq!(params.len(), 0);
}

#[test]
fn parse_skinparam_with_comments() {
    let src = "skinparam class {\n  ' this is a comment\n  BackgroundColor #FEFECE\n}";
    let params = parse_skinparams(src);
    assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
    assert_eq!(params.len(), 1);
}

// ── Convenience method tests ────────────────────────────────────

#[test]
fn background_color_element_key() {
    let src = "skinparam ClassBackgroundColor #FEFECE";
    let params = parse_skinparams(src);
    assert_eq!(params.background_color("class", "#default"), "#FEFECE");
}

#[test]
fn background_color_dot_key() {
    let src = "skinparam class {\n  BackgroundColor #AABB00\n}";
    let params = parse_skinparams(src);
    assert_eq!(params.background_color("class", "#default"), "#AABB00");
}

#[test]
fn background_color_global_does_not_cascade() {
    // In Java PlantUML, global `skinparam backgroundColor` only affects
    // the diagram canvas, NOT element fills.  Elements use their own
    // defaults (theme or hardcoded).
    let src = "skinparam BackgroundColor #FFFFFF";
    let params = parse_skinparams(src);
    // Should return theme default #F1F1F1 for class, not global #FFFFFF
    assert_eq!(params.background_color("class", "#default"), "#F1F1F1");
}

#[test]
fn background_color_default_fallback() {
    let params = SkinParams::new();
    assert_eq!(params.background_color("class", "#FEFECE"), "#F1F1F1");
}

#[test]
fn font_color_lookup_chain() {
    let src = "skinparam class {\n  FontColor #333333\n}";
    let params = parse_skinparams(src);
    assert_eq!(params.font_color("class", "#000000"), "#333333");
}

#[test]
fn border_color_lookup_chain() {
    let src = "skinparam ClassBorderColor #A80036";
    let params = parse_skinparams(src);
    assert_eq!(params.border_color("class", "#000000"), "#A80036");
}

#[test]
fn arrow_color_lookup() {
    let src = "skinparam ArrowColor blue";
    let params = parse_skinparams(src);
    assert_eq!(params.arrow_color("#A80036"), "#0000FF");
}

#[test]
fn arrow_color_default() {
    let params = SkinParams::new();
    assert_eq!(params.arrow_color("#A80036"), "#A80036");
}

#[test]
fn get_or_returns_default_when_missing() {
    let params = SkinParams::new();
    assert_eq!(params.get_or("nonexistent", "fallback"), "fallback");
}

#[test]
fn get_or_returns_value_when_present() {
    let src = "skinparam Foo bar";
    let params = parse_skinparams(src);
    assert_eq!(params.get_or("foo", "fallback"), "bar");
}

// ── Theme tests ───────────────────────────────────────────────────

#[test]
fn theme_rose_global_colors() {
    let t = Theme::rose();
    assert_eq!(t.background_color, "#FFFFFF");
    assert_eq!(t.font_color, "#000000");
    assert_eq!(t.arrow_color, "#181818");
    assert_eq!(t.border_color, "#181818");
}

#[test]
fn theme_rose_class_colors() {
    let t = Theme::rose();
    assert_eq!(t.class_bg, "#F1F1F1");
    assert_eq!(t.class_border, "#181818");
    assert_eq!(t.class_font, "#000000");
}

#[test]
fn theme_rose_sequence_colors() {
    let t = Theme::rose();
    assert_eq!(t.participant_bg, "#E2E2F0");
    assert_eq!(t.participant_border, "#181818");
    assert_eq!(t.lifeline_color, "#181818");
    assert_eq!(t.activation_bg, "#F1F1F1");
    assert_eq!(t.activation_border, "#181818");
    assert_eq!(t.group_bg, "#EEEEEE");
    assert_eq!(t.group_border, "#000000");
}

#[test]
fn theme_rose_note_colors() {
    let t = Theme::rose();
    assert_eq!(t.note_bg, "#FEFFDD");
    assert_eq!(t.note_border, "#181818");
}

#[test]
fn theme_rose_activity_colors() {
    let t = Theme::rose();
    assert_eq!(t.activity_bg, "#F1F1F1");
    assert_eq!(t.activity_border, "#181818");
    assert_eq!(t.diamond_bg, "#F1F1F1");
    assert_eq!(t.diamond_border, "#181818");
    assert_eq!(t.swimlane_border, "#181818");
    assert_eq!(t.swimlane_header_bg, "#F1F1F1");
}

#[test]
fn theme_rose_state_colors() {
    let t = Theme::rose();
    assert_eq!(t.state_bg, "#F1F1F1");
    assert_eq!(t.state_border, "#181818");
    assert_eq!(t.composite_bg, "#F1F1F1");
    assert_eq!(t.composite_border, "#181818");
}

#[test]
fn theme_rose_component_colors() {
    let t = Theme::rose();
    assert_eq!(t.component_bg, "#F1F1F1");
    assert_eq!(t.component_border, "#181818");
    assert_eq!(t.node_bg, "#F1F1F1");
    assert_eq!(t.node_border, "#181818");
    assert_eq!(t.database_bg, "#F1F1F1");
    assert_eq!(t.database_border, "#181818");
    assert_eq!(t.cloud_bg, "#F1F1F1");
    assert_eq!(t.cloud_border, "#181818");
}

#[test]
fn theme_rose_erd_colors() {
    let t = Theme::rose();
    assert_eq!(t.entity_bg, "#F1F1F1");
    assert_eq!(t.entity_border, "#181818");
    assert_eq!(t.relationship_bg, "#F1F1F1");
    assert_eq!(t.relationship_border, "#181818");
}

#[test]
fn theme_rose_mindmap_wbs_colors() {
    let t = Theme::rose();
    assert_eq!(t.mindmap_node_bg, "#F1F1F1");
    assert_eq!(t.mindmap_node_border, "#181818");
    assert_eq!(t.wbs_root_bg, "#FFD700");
}

#[test]
fn theme_rose_legend_colors() {
    let t = Theme::rose();
    assert_eq!(t.legend_bg, "#FEFFDD");
    assert_eq!(t.legend_border, "#000000");
}

#[test]
fn theme_default_is_rose() {
    let def = Theme::default();
    let rose = Theme::rose();
    assert_eq!(def.background_color, rose.background_color);
    assert_eq!(def.class_bg, rose.class_bg);
    assert_eq!(def.arrow_color, rose.arrow_color);
    assert_eq!(def.note_bg, rose.note_bg);
    assert_eq!(def.entity_bg, rose.entity_bg);
    assert_eq!(def.wbs_root_bg, rose.wbs_root_bg);
}

// ── SkinParams + Theme integration tests ──────────────────────────

#[test]
fn skinparams_default_has_rose_theme() {
    let sp = SkinParams::default();
    assert_eq!(sp.theme.class_bg, "#F1F1F1");
    assert_eq!(sp.theme.arrow_color, "#181818");
}

#[test]
fn skinparams_theme_fallback_bg() {
    let sp = SkinParams::new();
    // No explicit skinparam set: should fall back to theme for known elements
    assert_eq!(sp.background_color("class", "#IGNORED"), "#F1F1F1");
    assert_eq!(sp.background_color("component", "#IGNORED"), "#F1F1F1");
    assert_eq!(sp.background_color("entity", "#IGNORED"), "#F1F1F1");
    assert_eq!(sp.background_color("note", "#IGNORED"), "#FEFFDD");
    assert_eq!(sp.background_color("cloud", "#IGNORED"), "#F1F1F1");
}

#[test]
fn skinparams_theme_fallback_border() {
    let sp = SkinParams::new();
    assert_eq!(sp.border_color("class", "#IGNORED"), "#181818");
    assert_eq!(sp.border_color("state", "#IGNORED"), "#181818");
    assert_eq!(sp.border_color("note", "#IGNORED"), "#181818");
}

#[test]
fn skinparams_theme_fallback_font() {
    let sp = SkinParams::new();
    assert_eq!(sp.font_color("class", "#IGNORED"), "#000000");
    assert_eq!(sp.font_color("participant", "#IGNORED"), "#000000");
}

#[test]
fn skinparams_explicit_overrides_theme() {
    let src = "skinparam ClassBackgroundColor #112233";
    let sp = parse_skinparams(src);
    // Explicit skinparam should win over theme
    assert_eq!(sp.background_color("class", "#IGNORED"), "#112233");
}

#[test]
fn skinparams_global_does_not_override_theme() {
    let src = "skinparam BackgroundColor #AABBCC";
    let sp = parse_skinparams(src);
    // Global backgroundColor does not cascade to element fills
    assert_eq!(sp.background_color("class", "#IGNORED"), "#F1F1F1");
}

#[test]
fn skinparams_root_style_cascades_to_element_colors() {
    let src = "<style>\nroot {\n  BackgroundColor #ABCDEF\n  FontColor #654321\n  LineColor #123456\n}\n</style>";
    let sp = parse_skinparams(src);
    assert_eq!(sp.background_color("participant", "#IGNORED"), "#ABCDEF");
    assert_eq!(sp.border_color("participant", "#IGNORED"), "#123456");
    assert_eq!(sp.font_color("participant", "#IGNORED"), "#654321");
}

#[test]
fn theme_plain_cascades_root_background_to_sequence_participants() {
    let src = "@startuml\n!theme plain\nactor Alice\nparticipant Bob\nAlice -> Bob\n@enduml";
    let preprocessed = crate::preproc::preprocess(src).expect("theme preprocess");
    let sp = parse_skinparams(&preprocessed);
    assert_eq!(sp.background_color("participant", "#IGNORED"), "#FFFFFF");
    assert_eq!(sp.border_color("participant", "#IGNORED"), "#000000");
}

#[test]
fn skinparams_unknown_element_uses_caller_default() {
    let sp = SkinParams::new();
    // Unknown element has no theme mapping, so caller default is returned
    assert_eq!(sp.background_color("unknownelement", "#CALLER"), "#CALLER");
}

// ── New skinparam methods tests ────────────────────────────────────

#[test]
fn default_font_name_none_when_unset() {
    let sp = SkinParams::new();
    assert_eq!(sp.default_font_name(), None);
}

#[test]
fn default_font_name_returns_value() {
    let src = "skinparam defaultFontName Arial";
    let sp = parse_skinparams(src);
    assert_eq!(sp.default_font_name(), Some("Arial"));
}

#[test]
fn default_font_size_none_when_unset() {
    let sp = SkinParams::new();
    assert_eq!(sp.default_font_size(), None);
}

#[test]
fn default_font_size_returns_value() {
    let src = "skinparam defaultFontSize 14";
    let sp = parse_skinparams(src);
    assert_eq!(sp.default_font_size(), Some(14.0));
}

#[test]
fn monochrome_false_by_default() {
    let sp = SkinParams::new();
    assert!(!sp.is_monochrome());
}

#[test]
fn monochrome_true_when_set() {
    let src = "skinparam monochrome true";
    let sp = parse_skinparams(src);
    assert!(sp.is_monochrome());
}

#[test]
fn monochrome_false_when_explicit() {
    let src = "skinparam monochrome false";
    let sp = parse_skinparams(src);
    assert!(!sp.is_monochrome());
}

#[test]
fn handwritten_false_by_default() {
    let sp = SkinParams::new();
    assert!(!sp.is_handwritten());
}

#[test]
fn handwritten_true_when_set() {
    let src = "skinparam handwritten true";
    let sp = parse_skinparams(src);
    assert!(sp.is_handwritten());
}

#[test]
fn round_corner_none_when_unset() {
    let sp = SkinParams::new();
    assert_eq!(sp.round_corner(), None);
}

#[test]
fn round_corner_returns_value() {
    let src = "skinparam roundcorner 15";
    let sp = parse_skinparams(src);
    assert_eq!(sp.round_corner(), Some(15.0));
}

#[test]
fn font_size_element_key() {
    let src = "skinparam classFontSize 16";
    let sp = parse_skinparams(src);
    assert_eq!(sp.font_size("class", 12.0), 16.0);
}

#[test]
fn font_size_default_fallback() {
    let src = "skinparam defaultFontSize 14";
    let sp = parse_skinparams(src);
    assert_eq!(sp.font_size("class", 12.0), 14.0);
}

#[test]
fn font_size_caller_default() {
    let sp = SkinParams::new();
    assert_eq!(sp.font_size("class", 12.0), 12.0);
}

#[test]
fn sequence_arrow_thickness_none_when_unset() {
    let sp = SkinParams::new();
    assert_eq!(sp.sequence_arrow_thickness(), None);
}

#[test]
fn sequence_arrow_thickness_returns_value() {
    let src = "skinparam sequenceArrowThickness 2";
    let sp = parse_skinparams(src);
    assert_eq!(sp.sequence_arrow_thickness(), Some(2.0));
}

#[test]
fn sequence_arrow_color_returns_value() {
    let src = "skinparam sequenceArrowColor DarkBlue";
    let sp = parse_skinparams(src);
    assert_eq!(sp.sequence_arrow_color("#A80036"), "#00008B");
}

#[test]
fn sequence_arrow_color_fallback() {
    let sp = SkinParams::new();
    assert_eq!(sp.sequence_arrow_color("#A80036"), "#A80036");
}

#[test]
fn sequence_lifeline_border_color_returns_value() {
    let src = "skinparam sequenceLifeLineBorderColor blue";
    let sp = parse_skinparams(src);
    assert_eq!(sp.sequence_lifeline_border_color("#A80036"), "#0000FF");
}

#[test]
fn effective_font_family_default() {
    let sp = SkinParams::new();
    assert_eq!(sp.effective_font_family("monospace"), "monospace");
}

#[test]
fn effective_font_family_override() {
    let src = "skinparam defaultFontName Arial";
    let sp = parse_skinparams(src);
    assert_eq!(sp.effective_font_family("monospace"), "Arial");
}

#[test]
fn handwritten_font_family_none_when_disabled() {
    let sp = SkinParams::new();
    assert_eq!(sp.handwritten_font_family(), None);
}

#[test]
fn handwritten_font_family_returns_cursive() {
    let src = "skinparam handwritten true";
    let sp = parse_skinparams(src);
    assert!(sp.handwritten_font_family().is_some());
    assert!(sp.handwritten_font_family().unwrap().contains("cursive"));
}

// ══════════════════════════════════════════════════════════════════
// Tests ported from upstream PlantUML Java project
// ══════════════════════════════════════════════════════════════════

// ── Ported from upstream: StringTrieTest ─────────────────────────

// Ported from upstream: StringTrieTest.testPutAndGetSimple
#[test]
fn upstream_trie_put_and_get_simple() {
    let mut sp = SkinParams::new();
    sp.set("foo", "123");
    assert_eq!(sp.get("foo"), Some("123"));
    assert_eq!(sp.get("bar"), None);
}

// Ported from upstream: StringTrieTest.testCaseInsensitivity
#[test]
fn upstream_trie_case_insensitivity() {
    let mut sp = SkinParams::new();
    sp.set("Hello", "world");
    assert_eq!(sp.get("hello"), Some("world"));
    assert_eq!(sp.get("HELLO"), Some("world"));
    assert_eq!(sp.get("HeLlO"), Some("world"));
}

// Ported from upstream: StringTrieTest.testOverwriteValue
#[test]
fn upstream_trie_overwrite_value() {
    let mut sp = SkinParams::new();
    sp.set("key", "1");
    sp.set("key", "2");
    assert_eq!(sp.get("KEY"), Some("2"));
}

// Ported from upstream: StringTrieTest.testPrefixCollision
#[test]
fn upstream_trie_prefix_collision() {
    let mut sp = SkinParams::new();
    sp.set("abc", "10");
    sp.set("abcd", "20");
    assert_eq!(sp.get("ABC"), Some("10"));
    assert_eq!(sp.get("ABCD"), Some("20"));
    assert_eq!(sp.get("ab"), None);
}

// Ported from upstream: StringTrieTest.testEmptyStringKey
#[test]
fn upstream_trie_empty_string_key() {
    let mut sp = SkinParams::new();
    sp.set("", "empty");
    assert_eq!(sp.get(""), Some("empty"));
}

// ── Ported from upstream: ColorTrieNodeTest ──────────────────────

// Ported from upstream: ColorTrieNodeTest.testInvalidCharacterIgnoredOnPut
#[test]
fn upstream_color_normalize_named_darkblue() {
    assert_eq!(normalize_color("darkblue"), "#00008B");
}

// ── Ported from upstream: ColorHSBTest — hex color normalization ─

// Ported from upstream: ColorHSBTest.test_toString — ARGB alpha stripping
#[test]
fn upstream_color_normalize_hex_8digit_alpha_red() {
    assert_eq!(normalize_color("#AAFF0000"), "#FF0000");
}

#[test]
fn upstream_color_normalize_hex_8digit_alpha_green() {
    assert_eq!(normalize_color("#AA00FF00"), "#00FF00");
}

#[test]
fn upstream_color_normalize_hex_8digit_alpha_blue() {
    assert_eq!(normalize_color("#AA0000FF"), "#0000FF");
}

#[test]
fn upstream_color_normalize_hex_8digit_half_saturated() {
    assert_eq!(normalize_color("#FFFF8080"), "#FF8080");
}

#[test]
fn upstream_color_normalize_hex_8digit_half_brightness() {
    assert_eq!(normalize_color("#FF7F0000"), "#7F0000");
}

// ── Ported from upstream: StyleFontWeightTest (skinparam storage) ─

// Ported from upstream: StyleFontWeightTest — block with multiple properties
#[test]
fn upstream_skinparam_block_multiple_properties() {
    let src = "\
skinparam participant {
  FontName Roboto
  FontColor green
  FontSize 26
  LineColor #EE0000
}";
    let sp = parse_skinparams(src);
    assert_eq!(sp.get("participant.fontname"), Some("Roboto"));
    assert_eq!(sp.get("participant.fontcolor"), Some("#008000"));
    assert_eq!(sp.get("participant.fontsize"), Some("26"));
    assert_eq!(sp.get("participant.linecolor"), Some("#EE0000"));
}

// ── Ported from upstream: ValueImplFontFaceTest (property storage) ─

// Ported from upstream: ValueImplFontFaceTest.numericWeight100
#[test]
fn upstream_font_weight_numeric_stored_raw() {
    let src = "skinparam participant {\n  FontWeight 100\n}";
    let sp = parse_skinparams(src);
    assert_eq!(sp.get("participant.fontweight"), Some("100"));
}

// Ported from upstream: ValueImplFontFaceTest.numericWeight900
#[test]
fn upstream_font_weight_900_stored() {
    let src = "skinparam participant {\n  FontWeight 900\n}";
    let sp = parse_skinparams(src);
    assert_eq!(sp.get("participant.fontweight"), Some("900"));
}

// Ported from upstream: ValueImplFontFaceTest.boldKeyword
#[test]
fn upstream_font_style_bold_stored() {
    let src = "skinparam participant {\n  FontStyle bold\n}";
    let sp = parse_skinparams(src);
    assert_eq!(sp.get("participant.fontstyle"), Some("bold"));
}

// Ported from upstream: ValueImplFontFaceTest.italicKeyword
#[test]
fn upstream_font_style_italic_stored() {
    let src = "skinparam participant {\n  FontStyle italic\n}";
    let sp = parse_skinparams(src);
    assert_eq!(sp.get("participant.fontstyle"), Some("italic"));
}

// ── Ported from upstream: StyleFontWeightTest — independent axes ─

// Ported from upstream: StyleFontWeightTest.fontWeight900AndItalicAreBothPreserved
#[test]
fn upstream_font_weight_and_style_independent() {
    let src = "skinparam participant {\n  FontWeight 900\n  FontStyle italic\n  FontSize 26\n}";
    let sp = parse_skinparams(src);
    assert_eq!(sp.get("participant.fontweight"), Some("900"));
    assert_eq!(sp.get("participant.fontstyle"), Some("italic"));
    assert_eq!(sp.font_size("participant", 12.0), 26.0);
}

// ── Ported from upstream: resolution chain tests ─────────────────

// Ported from upstream: style resolution chain for background color
#[test]
fn upstream_background_color_resolution_chain() {
    // Level 1: element-specific key wins
    let src1 = "skinparam ComponentBackgroundColor #111111\nskinparam BackgroundColor #222222";
    let sp1 = parse_skinparams(src1);
    assert_eq!(sp1.background_color("component", "#default"), "#111111");

    // Level 2: global backgroundColor does NOT cascade to elements
    let src2 = "skinparam BackgroundColor #222222";
    let sp2 = parse_skinparams(src2);
    assert_eq!(sp2.background_color("component", "#default"), "#F1F1F1");

    // Level 3: theme fallback when nothing is set
    let sp3 = SkinParams::new();
    assert_eq!(sp3.background_color("component", "#default"), "#F1F1F1");
}

// Ported from upstream: font color resolution chain
#[test]
fn upstream_font_color_resolution_chain() {
    let src = "skinparam ClassFontColor #AA0000\nskinparam FontColor #BB0000";
    let sp = parse_skinparams(src);
    assert_eq!(sp.font_color("class", "#000000"), "#AA0000");

    let src2 = "skinparam FontColor #BB0000";
    let sp2 = parse_skinparams(src2);
    assert_eq!(sp2.font_color("class", "#000000"), "#BB0000");
}

// Ported from upstream: border color resolution chain
#[test]
fn upstream_border_color_resolution_chain() {
    let src = "skinparam StateBorderColor #CC0000\nskinparam BorderColor #DD0000";
    let sp = parse_skinparams(src);
    assert_eq!(sp.border_color("state", "#000000"), "#CC0000");

    let src2 = "skinparam BorderColor #DD0000";
    let sp2 = parse_skinparams(src2);
    assert_eq!(sp2.border_color("state", "#000000"), "#DD0000");
}

#[test]
fn parse_skinparam_nested_block_maxmessagesize() {
    let src = "@startuml\nskinparam {\n   Maxmessagesize 200\n}\ngroup Grouping messages\n    Test <- Test : text\nend\n@enduml";
    let params = parse_skinparams(src);
    assert_eq!(params.get("maxmessagesize"), Some("200"));
}

#[test]
fn parse_root_style_linethickness() {
    let src = "<style>\nroot {\n  LineThickness 1\n  FontName Verdana\n}\n</style>";
    let params = parse_skinparams(src);
    assert_eq!(params.get("root.linethickness"), Some("1"));
    assert_eq!(params.get("root.fontname"), Some("Verdana"));
}
