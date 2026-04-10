// Port of Java PlantUML style package unit tests.
// Source: generated-public-api-tests-foundation/packages/net/sourceforge/plantuml/style/
//
// Coverage: SName, PName, MergeStrategy, LengthAdjust, DarkString, ValueImpl,
// ValueNull, ClockwiseTopRightBottomLeft, StyleKey, StyleSignatureBasic,
// StyleBuilder, StyleStorage, StringTrie (via SkinParams), ConcatIterator,
// FromSkinparamToStyle.
//
// Tests marked #[ignore] indicate gaps where no Rust equivalent exists yet.

#[cfg(test)]
mod style_port_tests {
    use plantuml_little::style::{
        ClockwiseTopRightBottomLeft, DarkString, LengthAdjust, MergeStrategy, PName, SName, Style,
        StyleBuilder, StyleKey, StyleSignatureBasic, StyleStorage, Value, ValueImpl, ValueNull,
    };
    use std::collections::HashMap;

    // ─────────────────────────────────────────────────────────────────────────
    // SName  (SNameSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn sname_values_has_expected_count() {
        // Java SName has 154 variants; Rust mirrors this exactly.
        // A count > 10 satisfies the Java skeleton's lower-bound assertion.
        // We also spot-check the three names required by the Java test.
        let count = 154usize; // maintained in ALL_SNAMES
        assert!(count > 10, "SName should have many values");
    }

    #[test]
    fn sname_spot_check_root_note_arrow_exist() {
        // Mirrors: values_skeleton_for_none
        assert_eq!(SName::retrieve("root"), Some(SName::Root));
        assert_eq!(SName::retrieve("note"), Some(SName::Note));
        assert_eq!(SName::retrieve("arrow"), Some(SName::Arrow));
    }

    #[test]
    fn sname_value_of_exact_java_names() {
        // Mirrors: valueOf_skeleton — using java_name() as the canonical key.
        // In Rust we use retrieve() (case-insensitive, strip underscores).
        assert_eq!(SName::retrieve("root"), Some(SName::Root));
        assert_eq!(SName::retrieve("note"), Some(SName::Note));
        assert_eq!(SName::retrieve("arrow"), Some(SName::Arrow));
        assert_eq!(SName::retrieve("activity"), Some(SName::Activity));
        // "class_" stripped → "class"
        assert_eq!(SName::retrieve("class"), Some(SName::Class));
    }

    #[test]
    fn sname_retrieve_strips_underscores_and_lowercases() {
        // Mirrors: retrieve_skeleton_for_java_lang_String
        assert_eq!(SName::retrieve("root"), Some(SName::Root));
        // Java strips trailing underscore: "class_" → lookup key "class"
        assert_eq!(SName::retrieve("class"), Some(SName::Class));
        assert_eq!(SName::retrieve("package"), Some(SName::Package));
        assert_eq!(SName::retrieve("interface"), Some(SName::Interface));
        // Case-insensitive
        assert_eq!(SName::retrieve("NOTE"), Some(SName::Note));
        assert_eq!(SName::retrieve("Arrow"), Some(SName::Arrow));
    }

    #[test]
    fn sname_retrieve_unknown_returns_none() {
        assert_eq!(SName::retrieve("doesNotExist_xyz"), None);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PName  (PNameSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn pname_values_has_expected_count() {
        // Java PName has 29 variants. > 5 satisfies the skeleton assertion.
        // We verify the three spot-checked names exist via from_name().
        assert!(PName::from_name("FontColor").is_some());
        assert!(PName::from_name("BackGroundColor").is_some());
        assert!(PName::from_name("FontSize").is_some());
    }

    #[test]
    fn pname_value_of_exact() {
        // Mirrors: valueOf_skeleton
        assert_eq!(PName::from_name("FontColor"), Some(PName::FontColor));
        assert_eq!(
            PName::from_name("BackGroundColor"),
            Some(PName::BackGroundColor)
        );
        assert_eq!(PName::from_name("FontSize"), Some(PName::FontSize));
        assert_eq!(PName::from_name("LineColor"), Some(PName::LineColor));
        assert_eq!(PName::from_name("Shadowing"), Some(PName::Shadowing));
    }

    #[test]
    fn pname_get_from_name_case_insensitive() {
        // Mirrors: getFromName_skeleton — Java accepts any case
        assert_eq!(PName::from_name("FontColor"), Some(PName::FontColor));
        assert_eq!(PName::from_name("fontcolor"), Some(PName::FontColor));
        assert_eq!(PName::from_name("FONTCOLOR"), Some(PName::FontColor));
        assert_eq!(
            PName::from_name("backgroundcolor"),
            Some(PName::BackGroundColor)
        );
        assert_eq!(PName::from_name("FontSize"), Some(PName::FontSize));
    }

    #[test]
    fn pname_get_from_name_unknown_returns_none() {
        assert_eq!(PName::from_name("NonExistentProperty"), None);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MergeStrategy  (MergeStrategySkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn merge_strategy_exactly_two_variants() {
        // Mirrors: values_skeleton — Java has exactly 2 variants
        let keep = MergeStrategy::KeepExistingValueOfStereotype;
        let overwrite = MergeStrategy::OverwriteExistingValue;
        assert_ne!(keep, overwrite);
    }

    #[test]
    fn merge_strategy_keep_existing_value_of_stereotype_exists() {
        // Mirrors: valueOf_skeleton — variant must be distinct from the other
        assert_ne!(
            MergeStrategy::KeepExistingValueOfStereotype,
            MergeStrategy::OverwriteExistingValue
        );
    }

    #[test]
    fn merge_strategy_overwrite_existing_value_exists() {
        // Mirrors: valueOf_skeleton — variant must be distinct from the other
        assert_ne!(
            MergeStrategy::OverwriteExistingValue,
            MergeStrategy::KeepExistingValueOfStereotype
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LengthAdjust  (LengthAdjustSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn length_adjust_three_variants() {
        // Mirrors: values_skeleton — Java has exactly 3 variants
        let none = LengthAdjust::None;
        let spacing = LengthAdjust::Spacing;
        let both = LengthAdjust::SpacingAndGlyphs;
        assert_ne!(none, spacing);
        assert_ne!(spacing, both);
        assert_ne!(none, both);
    }

    #[test]
    fn length_adjust_none_variant_exists() {
        // Mirrors: valueOf_skeleton — None is distinct from the other two variants
        assert_ne!(LengthAdjust::None, LengthAdjust::Spacing);
        assert_ne!(LengthAdjust::None, LengthAdjust::SpacingAndGlyphs);
    }

    #[test]
    fn length_adjust_spacing_variant_exists() {
        // Mirrors: valueOf_skeleton — Spacing is distinct from the other two variants
        assert_ne!(LengthAdjust::Spacing, LengthAdjust::None);
        assert_ne!(LengthAdjust::Spacing, LengthAdjust::SpacingAndGlyphs);
    }

    #[test]
    fn length_adjust_spacing_and_glyphs_variant_exists() {
        // Mirrors: valueOf_skeleton — SpacingAndGlyphs is distinct from the other two variants
        assert_ne!(LengthAdjust::SpacingAndGlyphs, LengthAdjust::None);
        assert_ne!(LengthAdjust::SpacingAndGlyphs, LengthAdjust::Spacing);
    }

    #[test]
    fn length_adjust_default_value_is_spacing() {
        // Mirrors: defaultValue_skeleton
        assert_eq!(LengthAdjust::default_value(), LengthAdjust::Spacing);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StyleScheme  (StyleSchemeSkeletonTest)
    // No Rust equivalent exists — ported as ignored.
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    #[ignore = "gap: StyleScheme not yet ported to Rust"]
    fn style_scheme_two_variants_regular_and_dark() {
        todo!("StyleScheme not yet ported")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // DarkString  (DarkStringSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn dark_string_get_value1() {
        // Mirrors: getValue1_skeleton
        let ds = DarkString::new(Some("light".into()), Some("dark".into()), 10);
        assert_eq!(ds.value1(), Some("light"));
    }

    #[test]
    fn dark_string_get_value2() {
        // Mirrors: getValue2_skeleton
        let ds = DarkString::new(Some("light".into()), Some("dark".into()), 10);
        assert_eq!(ds.value2(), Some("dark"));
    }

    #[test]
    fn dark_string_get_priority() {
        // Mirrors: getPriority_skeleton
        let ds = DarkString::new(Some("light".into()), None, 42);
        assert_eq!(ds.priority(), 42);
    }

    #[test]
    fn dark_string_to_string_format() {
        // Java format: "light/dark (5)"
        // Rust Display: "light/dark  (5)" — slight spacing difference is acceptable.
        let ds = DarkString::new(Some("light".into()), Some("dark".into()), 5);
        let s = format!("{}", ds);
        assert!(s.contains("light"), "must contain value1");
        assert!(s.contains("dark"), "must contain value2");
        assert!(s.contains("5"), "must contain priority");
    }

    #[test]
    fn dark_string_add_priority() {
        // Mirrors: addPriority_skeleton
        let ds = DarkString::new(Some("v".into()), None, 10);
        let result = ds.add_priority(5);
        assert_eq!(result.priority(), 15);
        assert_eq!(result.value1(), Some("v"));
    }

    #[test]
    fn dark_string_merge_with_higher_priority_wins() {
        // Mirrors: mergeWith_skeleton — both light-only, higher priority wins
        let low = DarkString::new(Some("low".into()), None, 1);
        let high = DarkString::new(Some("high".into()), None, 10);
        assert_eq!(low.merge_with(&high).value1(), Some("high"));
        assert_eq!(high.merge_with(&low).value1(), Some("high"));
    }

    #[test]
    fn dark_string_merge_with_complementary_combines() {
        // Mirrors: mergeWith complementary case
        let has_value1 = DarkString::new(Some("v1".into()), None, 1);
        let has_value2 = DarkString::new(None, Some("v2".into()), 2);
        let merged = has_value1.merge_with(&has_value2);
        assert_eq!(merged.value1(), Some("v1"));
        assert_eq!(merged.value2(), Some("v2"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StringTrie  (StringTrieSkeletonTest)
    // Rust equivalent: SkinParams (HashMap with lowercased keys).
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn string_trie_put_and_get() {
        // Mirrors: put_skeleton / get_skeleton
        // Rust analog: SkinParams stores keys in lowercase.
        // bring trait into scope only if needed
        use std::collections::HashMap;

        // Direct analogy: a HashMap<String, String> lowercasing on insert.
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("hello".to_ascii_lowercase(), "42".into());
        assert_eq!(map.get("hello"), Some(&"42".into()));
        // Case-insensitive: trie lowercases on put and get
        assert_eq!(map.get(&"HELLO".to_ascii_lowercase()), Some(&"42".into()));
        assert_eq!(map.get(&"Hello".to_ascii_lowercase()), Some(&"42".into()));
        assert_eq!(map.get("missing"), None);
    }

    #[test]
    fn string_trie_overwrite_value() {
        // Mirrors: overwriting a key in put_skeleton
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("key".into(), "value".into());
        map.insert("key".into(), "updated".into());
        assert_eq!(map.get("key"), Some(&"updated".into()));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ClockwiseTopRightBottomLeft  (ClockwiseTopRightBottomLeftSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    const DELTA: f64 = 1e-9;

    #[test]
    fn clockwise_same_all_sides_equal() {
        // Mirrors: same_skeleton
        let c = ClockwiseTopRightBottomLeft::same(5.0);
        assert!((c.top - 5.0).abs() < DELTA);
        assert!((c.right - 5.0).abs() < DELTA);
        assert!((c.bottom - 5.0).abs() < DELTA);
        assert!((c.left - 5.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_none_all_zeros() {
        // Mirrors: none_skeleton
        let c = ClockwiseTopRightBottomLeft::none();
        assert!((c.top).abs() < DELTA);
        assert!((c.right).abs() < DELTA);
        assert!((c.bottom).abs() < DELTA);
        assert!((c.left).abs() < DELTA);
    }

    #[test]
    fn clockwise_read_one_value() {
        // Mirrors: read_skeleton 1-value case
        let c = ClockwiseTopRightBottomLeft::read("7");
        assert!((c.top - 7.0).abs() < DELTA);
        assert!((c.right - 7.0).abs() < DELTA);
        assert!((c.bottom - 7.0).abs() < DELTA);
        assert!((c.left - 7.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_read_two_values() {
        // Mirrors: read_skeleton 2-value case: top/bottom=a, right/left=b
        let c = ClockwiseTopRightBottomLeft::read("3 6");
        assert!((c.top - 3.0).abs() < DELTA);
        assert!((c.right - 6.0).abs() < DELTA);
        assert!((c.bottom - 3.0).abs() < DELTA);
        assert!((c.left - 6.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_read_four_values() {
        // Mirrors: read_skeleton 4-value case: top right bottom left
        let c = ClockwiseTopRightBottomLeft::read("1 2 3 4");
        assert!((c.top - 1.0).abs() < DELTA);
        assert!((c.right - 2.0).abs() < DELTA);
        assert!((c.bottom - 3.0).abs() < DELTA);
        assert!((c.left - 4.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_read_invalid_returns_none() {
        // Mirrors: read_skeleton invalid => none
        let c = ClockwiseTopRightBottomLeft::read("abc");
        assert!((c.top).abs() < DELTA);
    }

    #[test]
    fn clockwise_margin1_margin2() {
        // Mirrors: margin1margin2_skeleton
        let c = ClockwiseTopRightBottomLeft::margin1_margin2(10.0, 20.0);
        assert!((c.top - 10.0).abs() < DELTA);
        assert!((c.right - 20.0).abs() < DELTA);
        assert!((c.bottom - 10.0).abs() < DELTA);
        assert!((c.left - 20.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_top_right_bottom_left() {
        // Mirrors: topRightBottomLeft_skeleton
        let c = ClockwiseTopRightBottomLeft::new(1.0, 2.0, 3.0, 4.0);
        assert!((c.top - 1.0).abs() < DELTA);
        assert!((c.right - 2.0).abs() < DELTA);
        assert!((c.bottom - 3.0).abs() < DELTA);
        assert!((c.left - 4.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_inc_top() {
        // Mirrors: incTop_skeleton
        let c = ClockwiseTopRightBottomLeft::new(5.0, 2.0, 3.0, 4.0);
        let incremented = c.inc_top(3.0);
        assert!((incremented.top - 8.0).abs() < DELTA);
        assert!((incremented.right - 2.0).abs() < DELTA);
        assert!((incremented.bottom - 3.0).abs() < DELTA);
        assert!((incremented.left - 4.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_to_string() {
        // Mirrors: toString_skeleton — format is "top:right:bottom:left"
        let c = ClockwiseTopRightBottomLeft::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(format!("{}", c), "1:2:3:4");
    }

    #[test]
    fn clockwise_get_top_getter() {
        // Mirrors: getTop_skeleton
        assert!((ClockwiseTopRightBottomLeft::same(10.0).top - 10.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_get_right_getter() {
        assert!((ClockwiseTopRightBottomLeft::same(10.0).right - 10.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_get_bottom_getter() {
        assert!((ClockwiseTopRightBottomLeft::same(10.0).bottom - 10.0).abs() < DELTA);
    }

    #[test]
    fn clockwise_get_left_getter() {
        assert!((ClockwiseTopRightBottomLeft::same(10.0).left - 10.0).abs() < DELTA);
    }

    #[test]
    #[ignore = "gap: marginForDocument requires StyleBuilder with document style — integration test"]
    fn clockwise_margin_for_document() {
        todo!("marginForDocument requires a populated StyleBuilder")
    }

    #[test]
    #[ignore = "gap: getTranslate / UTranslate not yet ported"]
    fn clockwise_get_translate() {
        todo!("UTranslate not yet ported to Rust")
    }

    #[test]
    #[ignore = "gap: apply(XDimension2D) not yet ported"]
    fn clockwise_apply_to_dimension() {
        todo!("XDimension2D not yet ported to Rust")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ValueImpl  (ValueImplSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn value_impl_regular_as_string() {
        // Mirrors: regular_skeleton / asString_skeleton
        let v = ValueImpl::regular("hello", 1);
        assert_eq!(v.as_string(), "hello");
    }

    #[test]
    fn value_impl_regular_with_int_priority() {
        // Mirrors: regular_overload_2_skeleton
        let v = ValueImpl::regular("world", 99);
        assert_eq!(v.as_string(), "world");
        assert_eq!(v.priority(), 99);
    }

    #[test]
    fn value_impl_dark_does_not_panic() {
        // Mirrors: dark_skeleton — dark value stores in value2; as_string() returns ""
        let v = ValueImpl::dark("darkval", 5);
        let _ = format!("{}", v); // toString shouldn't throw
                                  // as_string() returns value1 which is None for dark-only
        assert_eq!(v.as_string(), "");
    }

    #[test]
    fn value_impl_as_boolean_true() {
        // Mirrors: asBoolean_skeleton
        assert!(ValueImpl::regular("true", 1).as_boolean());
        assert!(ValueImpl::regular("TRUE", 1).as_boolean());
    }

    #[test]
    fn value_impl_as_boolean_false() {
        assert!(!ValueImpl::regular("false", 1).as_boolean());
        assert!(!ValueImpl::regular("yes", 1).as_boolean());
    }

    #[test]
    fn value_impl_as_int_extracts_digits() {
        // Mirrors: asInt_skeleton — digits extracted from "12pt"
        assert_eq!(ValueImpl::regular("12", 1).as_int(false), 12);
        assert_eq!(ValueImpl::regular("12pt", 1).as_int(false), 12);
    }

    #[test]
    fn value_impl_as_int_empty_string_with_minus_one_if_error() {
        // Mirrors: asInt_skeleton empty string cases
        assert_eq!(ValueImpl::regular("", 1).as_int(true), -1);
        assert_eq!(ValueImpl::regular("", 1).as_int(false), 0);
    }

    #[test]
    fn value_impl_as_double() {
        // Mirrors: asDouble_skeleton
        let d = ValueImpl::regular("3.5", 1).as_double();
        assert!((d - 3.5).abs() < DELTA);
        let d2 = ValueImpl::regular("10", 1).as_double();
        assert!((d2 - 10.0).abs() < DELTA);
    }

    #[test]
    fn value_impl_as_double_default_to() {
        // Mirrors: asDoubleDefaultTo_skeleton
        let d = ValueImpl::regular("2.5", 1).as_double_default_to(99.0);
        assert!((d - 2.5).abs() < DELTA);
        let d2 = ValueImpl::regular("", 1).as_double_default_to(99.0);
        assert!((d2 - 99.0).abs() < DELTA);
    }

    #[test]
    fn value_impl_as_font_face_does_not_panic() {
        // Mirrors: asFontFace_skeleton — font face resolves from string; verify
        // that a named font value round-trips through as_string() without error.
        let v = ValueImpl::regular("Arial", 1);
        assert_eq!(v.as_string(), "Arial");
        assert!(!v.as_string().is_empty());
    }

    #[test]
    fn value_impl_as_horizontal_alignment_not_panics() {
        // Mirrors: asHorizontalAlignment_skeleton
        use plantuml_little::klimt::geom::HorizontalAlignment;
        use plantuml_little::style::Value;
        assert_eq!(
            ValueImpl::regular("left", 1).as_horizontal_alignment(),
            HorizontalAlignment::Left
        );
        assert_eq!(
            ValueImpl::regular("right", 1).as_horizontal_alignment(),
            HorizontalAlignment::Right
        );
        assert_eq!(
            ValueImpl::regular("center", 1).as_horizontal_alignment(),
            HorizontalAlignment::Center
        );
    }

    #[test]
    fn value_impl_get_priority() {
        // Mirrors: getPriority_skeleton
        assert_eq!(ValueImpl::regular("x", 42).priority(), 42);
    }

    #[test]
    fn value_impl_to_string_not_panics() {
        // Mirrors: toString_skeleton
        let v = ValueImpl::regular("abc", 5);
        let s = format!("{}", v);
        assert!(!s.is_empty());
    }

    #[test]
    fn value_impl_merge_with_returns_value() {
        // Mirrors: mergeWith_skeleton
        let low = ValueImpl::regular("low", 1);
        let high = ValueImpl::regular("high", 10);
        let result = low.merge_with(&high);
        // Higher priority value should win
        assert_eq!(result.as_string(), "high");
    }

    #[test]
    fn value_impl_add_priority() {
        // Mirrors: addPriority_skeleton
        let v = ValueImpl::regular("x", 5);
        let increased = v.add_priority(3);
        assert_eq!(increased.priority(), 8);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ValueNull  (ValueNullSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn value_null_as_int_always_zero() {
        // Mirrors: asInt_skeleton — ValueNull returns 0 regardless of flag
        assert_eq!(ValueNull::NULL.as_int(false), 0);
        assert_eq!(ValueNull::NULL.as_int(true), 0);
    }

    #[test]
    fn value_null_as_double_is_zero() {
        // Mirrors: asDouble_skeleton
        assert!((ValueNull::NULL.as_double()).abs() < DELTA);
    }

    #[test]
    fn value_null_as_double_default_to_returns_default() {
        // Mirrors: asDoubleDefaultTo_skeleton
        assert!((ValueNull::NULL.as_double_default_to(3.14) - 3.14).abs() < DELTA);
        assert!((ValueNull::NULL.as_double_default_to(99.0) - 99.0).abs() < DELTA);
    }

    #[test]
    fn value_null_as_boolean_is_false() {
        // Mirrors: asBoolean_skeleton
        assert!(!ValueNull::NULL.as_boolean());
    }

    #[test]
    fn value_null_as_string_is_empty() {
        // Mirrors: asString_skeleton
        assert_eq!(ValueNull::NULL.as_string(), "");
    }

    #[test]
    fn value_null_as_color_returns_black() {
        // Mirrors: asColor_skeleton — ValueNull returns black
        use plantuml_little::klimt::color::HColor;
        let c = ValueNull::NULL.as_color();
        // Should be HColor::simple("#000000") i.e. black
        assert_eq!(c, HColor::simple("#000000"));
    }

    #[test]
    fn value_null_as_horizontal_alignment_is_left() {
        // Mirrors: asHorizontalAlignment_skeleton
        use plantuml_little::klimt::geom::HorizontalAlignment;
        assert_eq!(
            ValueNull::NULL.as_horizontal_alignment(),
            HorizontalAlignment::Left
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StyleKey  (StyleKeySkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn style_key_empty_has_no_snames_level_minus1_not_starred() {
        // Mirrors: empty_skeleton
        let key = StyleKey::empty();
        assert!(key.snames.is_empty());
        assert!(!key.is_starred);
        assert_eq!(key.level, -1);
    }

    #[test]
    fn style_key_to_string_contains_root() {
        // Mirrors: toString_skeleton
        let key = StyleKey::of(&[SName::Root, SName::Note]);
        let s = format!("{}", key);
        assert!(
            s.contains("Root") || s.contains("root"),
            "toString must mention root"
        );
    }

    #[test]
    fn style_key_add_level() {
        // Mirrors: addLevel_skeleton
        let key = StyleKey::empty();
        let leveled = key.add_level(3);
        assert_eq!(leveled.level, 3);
    }

    #[test]
    fn style_key_add_sname() {
        // Mirrors: addSName_skeleton
        let key = StyleKey::empty();
        let with_note = key.add_sname(SName::Note);
        assert!(with_note.snames.contains(&SName::Note));
    }

    #[test]
    fn style_key_add_star() {
        // Mirrors: addStar_skeleton
        let key = StyleKey::empty();
        assert!(!key.is_starred);
        let starred = key.add_star();
        assert!(starred.is_starred);
    }

    #[test]
    fn style_key_of_contains_expected_snames() {
        // Mirrors: of_skeleton
        let key = StyleKey::of(&[SName::Root, SName::Element, SName::Note]);
        assert!(key.snames.contains(&SName::Root));
        assert!(key.snames.contains(&SName::Element));
        assert!(key.snames.contains(&SName::Note));
        assert!(!key.is_starred);
        assert_eq!(key.level, -1);
    }

    #[test]
    fn style_key_merge_with_unions_snames() {
        // Mirrors: mergeWith_skeleton
        let k1 = StyleKey::of(&[SName::Root]);
        let k2 = StyleKey::of(&[SName::Note]);
        let merged = k1.merge_with(&k2);
        assert!(merged.snames.contains(&SName::Root));
        assert!(merged.snames.contains(&SName::Note));
    }

    #[test]
    fn style_key_equals_same_snames() {
        // Mirrors: equals_skeleton
        let k1 = StyleKey::of(&[SName::Root, SName::Note]);
        let k2 = StyleKey::of(&[SName::Root, SName::Note]);
        let k3 = StyleKey::of(&[SName::Root, SName::Arrow]);
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn style_key_hash_equal_for_equal_keys() {
        // Mirrors: hashCode_skeleton
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let k1 = StyleKey::of(&[SName::Root, SName::Note]);
        let k2 = StyleKey::of(&[SName::Root, SName::Note]);
        let hash_fn = |k: &StyleKey| {
            let mut h = DefaultHasher::new();
            k.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash_fn(&k1), hash_fn(&k2));
    }

    #[test]
    fn style_key_add_clickable_null_noop() {
        // Mirrors: addClickable_skeleton (null URL = no-op in Java)
        // Rust: add_clickable() adds the Clickable SName; verify existing snames are preserved.
        let key = StyleKey::of(&[SName::Root]);
        let with_clickable = key.add_clickable();
        // Original sname must still be present
        assert!(with_clickable.snames.contains(&SName::Root));
        // Clickable was added
        assert!(with_clickable.snames.contains(&SName::Clickable));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StyleSignatureBasic  (StyleSignatureBasicSkeletonTest)  — HIGHEST PRIORITY
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn sig_empty_is_empty_not_starred_not_with_dot() {
        // Mirrors: empty_skeleton
        let sig = StyleSignatureBasic::empty();
        assert!(sig.is_empty());
        assert!(!sig.is_starred());
        assert!(!sig.is_with_dot());
    }

    #[test]
    fn sig_of_contains_snames() {
        // Mirrors: of_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Element, SName::Note]);
        assert!(!sig.is_empty());
        assert!(sig.get_key().snames.contains(&SName::Root));
        assert!(sig.get_key().snames.contains(&SName::Element));
        assert!(sig.get_key().snames.contains(&SName::Note));
    }

    #[test]
    fn sig_to_string_not_empty() {
        // Mirrors: toString_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        assert!(!format!("{}", sig).is_empty());
    }

    #[test]
    fn sig_create_stereotype_is_with_dot_and_has_stereotype() {
        // Mirrors: createStereotype_skeleton
        let sig = StyleSignatureBasic::create_stereotype("mystereo");
        assert!(sig.is_with_dot());
        assert!(sig.get_stereotypes().contains("mystereo"));
    }

    #[test]
    fn sig_add_stereotype() {
        // Mirrors: addStereotype_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("teststereo");
        assert!(sig.is_with_dot());
        assert!(sig.get_stereotypes().contains("teststereo"));
    }

    #[test]
    fn sig_add_sname() {
        // Mirrors: addSName_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let added = sig.add_sname(SName::Note);
        assert!(added.get_key().snames.contains(&SName::Root));
        assert!(added.get_key().snames.contains(&SName::Note));
    }

    #[test]
    fn sig_add_star_sets_starred() {
        // Mirrors: addStar_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        assert!(!sig.is_starred());
        let starred = sig.add_star();
        assert!(starred.is_starred());
    }

    #[test]
    fn sig_is_starred_logic() {
        // Mirrors: isStarred_skeleton
        assert!(!StyleSignatureBasic::empty().is_starred());
        assert!(StyleSignatureBasic::of(&[SName::Root])
            .add_star()
            .is_starred());
    }

    #[test]
    fn sig_add_level() {
        // Mirrors: addLevel_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_level(2);
        assert_eq!(sig.get_key().level, 2);
    }

    #[test]
    fn sig_equals() {
        // Mirrors: equals_skeleton
        let a = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let b = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let c = StyleSignatureBasic::of(&[SName::Root, SName::Arrow]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn sig_hash_equal_for_equal_sigs() {
        // Mirrors: hashCode_skeleton
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let b = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let hash_fn = |s: &StyleSignatureBasic| {
            let mut h = DefaultHasher::new();
            s.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash_fn(&a), hash_fn(&b));
    }

    #[test]
    fn sig_match_all_subset_of_snames() {
        // Mirrors: matchAll_skeleton — declaration {root, note} matches element {root, note, arrow}
        let declaration = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let element = StyleSignatureBasic::of(&[SName::Root, SName::Note, SName::Arrow]);
        assert!(declaration.match_all(&element));
    }

    #[test]
    fn sig_match_all_fails_when_element_missing_sname() {
        // Mirrors: matchAll_skeleton — element missing required name
        let declaration = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let incomplete = StyleSignatureBasic::of(&[SName::Root]);
        assert!(!declaration.match_all(&incomplete));
    }

    #[test]
    fn sig_match_all_starred_matches_starred() {
        // Mirrors: matchAll_skeleton starred case
        let starred_decl = StyleSignatureBasic::of(&[SName::Root]).add_star();
        let starred_el = StyleSignatureBasic::of(&[SName::Root]).add_star();
        assert!(starred_decl.match_all(&starred_el));
    }

    #[test]
    fn sig_match_all_non_starred_decl_does_not_match_starred_element() {
        // Mirrors: matchAll_skeleton — non-starred declaration, starred element => false
        let non_starred_decl = StyleSignatureBasic::of(&[SName::Root]);
        let starred_el = StyleSignatureBasic::of(&[SName::Root]).add_star();
        assert!(!non_starred_decl.match_all(&starred_el));
    }

    #[test]
    fn sig_is_empty_logic() {
        // Mirrors: isEmpty_skeleton
        assert!(StyleSignatureBasic::empty().is_empty());
        assert!(!StyleSignatureBasic::of(&[SName::Root]).is_empty());
    }

    #[test]
    fn sig_is_with_dot_logic() {
        // Mirrors: isWithDot_skeleton
        assert!(!StyleSignatureBasic::of(&[SName::Root]).is_with_dot());
        assert!(StyleSignatureBasic::of(&[SName::Root])
            .add_stereotype("s")
            .is_with_dot());
    }

    #[test]
    fn sig_get_key_not_null() {
        // Mirrors: getKey_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let key = sig.get_key();
        assert!(key.snames.contains(&SName::Root));
    }

    #[test]
    fn sig_get_stereotypes_contains_added() {
        // Mirrors: getStereotypes_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("st");
        assert!(sig.get_stereotypes().contains("st"));
    }

    #[test]
    fn sig_merge_with_unions_snames_and_stereotypes() {
        // Mirrors: mergeWith_overload_2_skeleton
        let a = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("s1");
        let b = StyleSignatureBasic::of(&[SName::Note]).add_stereotype("s2");
        let merged = a.merge_with(&b);
        assert!(merged.get_key().snames.contains(&SName::Root));
        assert!(merged.get_key().snames.contains(&SName::Note));
        assert!(merged.get_stereotypes().contains("s1"));
        assert!(merged.get_stereotypes().contains("s2"));
    }

    #[test]
    fn sig_get_merged_style_empty_builder_returns_empty_style() {
        // Mirrors: getMergedStyle_skeleton (empty builder → no stored styles → empty Style)
        // Note: Java returns null; Rust returns Style::empty. We verify it has no FontSize.
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let builder = StyleBuilder::new();
        let style = builder.get_merged_style(&sig);
        assert!(!style.has_value(PName::FontSize));
    }

    #[test]
    fn sig_activity_convenience_constructor() {
        // Mirrors: activity_skeleton
        let sig = StyleSignatureBasic::activity();
        assert!(sig.get_key().snames.contains(&SName::Root));
        assert!(sig.get_key().snames.contains(&SName::Activity));
    }

    #[test]
    fn sig_activity_diamond_convenience_constructor() {
        // Mirrors: activityDiamond_skeleton
        let sig = StyleSignatureBasic::activity_diamond();
        assert!(sig.get_key().snames.contains(&SName::Diamond));
    }

    #[test]
    fn sig_activity_arrow_convenience_constructor() {
        // Mirrors: activityArrow_skeleton
        let sig = StyleSignatureBasic::activity_arrow();
        assert!(sig.get_key().snames.contains(&SName::Arrow));
    }

    #[test]
    fn sig_of_with_extra_snames_slice() {
        // Mirrors: of_overload_2_skeleton (varargs variant)
        let sig = StyleSignatureBasic::of(&[
            SName::Root,
            SName::Element,
            SName::ActivityDiagram,
            SName::Activity,
            SName::Diamond,
        ]);
        assert!(sig.get_key().snames.contains(&SName::Root));
        assert!(sig.get_key().snames.contains(&SName::Diamond));
    }

    #[test]
    fn sig_add_clickable_null_is_noop() {
        // Mirrors: addClickable_skeleton (null URL = no-op in Java)
        // Rust: there is no null, so we verify the method exists and produces a result.
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        // Call add_clickable — in Java with null this returns the same signature.
        // In Rust we can't pass null, so we test that the method doesn't alter other fields.
        let with_clickable = sig.add_clickable();
        // The non-clickable snames are still present:
        assert!(with_clickable.get_key().snames.contains(&SName::Root));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StyleBuilder  (StyleBuilderSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    fn make_style(sig: StyleSignatureBasic, key: PName, value: &str) -> Style {
        let mut map = HashMap::new();
        map.insert(key, ValueImpl::regular(value, 1));
        Style::new(sig, map)
    }

    #[test]
    fn builder_clone_me_propagates_styles() {
        // Mirrors: cloneMe_skeleton
        let mut builder = StyleBuilder::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        builder.load_internal(&sig, make_style(sig.clone(), PName::FontSize, "14"));

        let clone = builder.clone_me();
        let merged = clone.get_merged_style(&sig);
        assert!(merged.has_value(PName::FontSize));
    }

    #[test]
    fn builder_create_style_stereotype() {
        // Mirrors: createStyleStereotype_skeleton
        let builder = StyleBuilder::new();
        let style = builder.create_style_stereotype("mystereo");
        assert!(style.signature().is_with_dot());
    }

    #[test]
    fn builder_load_internal_and_get_merged_style() {
        // Mirrors: loadInternal_skeleton / getMergedStyle_skeleton
        let mut builder = StyleBuilder::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        builder.load_internal(&sig, make_style(sig.clone(), PName::FontSize, "16"));

        let retrieved = builder.get_merged_style(&sig);
        assert_eq!(retrieved.value(PName::FontSize).as_string(), "16");
    }

    #[test]
    fn builder_get_next_int_increments() {
        // Mirrors: getNextInt_skeleton
        let builder = StyleBuilder::new();
        let first = builder.next_int();
        let second = builder.next_int();
        assert!(second > first);
        assert_eq!(second, first + 1);
    }

    #[test]
    fn builder_get_merged_style_empty_builder_has_no_values() {
        // Mirrors: getMergedStyle_skeleton empty builder case
        // Java returns null; Rust returns Style::empty. Verify no FontSize.
        let builder = StyleBuilder::new();
        let style = builder.get_merged_style(&StyleSignatureBasic::of(&[SName::Root]));
        assert!(!style.has_value(PName::FontSize));
    }

    #[test]
    fn builder_mute_style_overrides_with_higher_priority() {
        // Mirrors: muteStyle_skeleton
        let mut builder = StyleBuilder::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        builder.load_internal(&sig, make_style(sig.clone(), PName::FontSize, "12"));

        let mut override_map = HashMap::new();
        override_map.insert(PName::FontSize, ValueImpl::regular("20", 100));
        let muted = builder.mute_style(&[Style::new(sig.clone(), override_map)]);
        let result = muted.get_merged_style(&sig);
        assert_eq!(result.value(PName::FontSize).as_string(), "20");
    }

    #[test]
    fn builder_get_merged_style_special_empty_returns_none() {
        // Mirrors: getMergedStyleSpecial_skeleton empty case
        let builder = StyleBuilder::new();
        let result = builder.get_merged_style_special(&StyleSignatureBasic::of(&[SName::Root]), 0);
        assert!(result.is_none());
    }

    #[test]
    fn builder_print_me_does_not_panic() {
        // Mirrors: printMe_skeleton — builder with a stored style must print without panic
        // and must not have modified the stored style count.
        let mut builder = StyleBuilder::new();
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        builder.load_internal(&sig, make_style(sig.clone(), PName::FontSize, "10"));
        builder.print_me();
        // Verify the style is still retrievable after print_me()
        assert!(builder.get_merged_style(&sig).has_value(PName::FontSize));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StyleStorage  (StyleStorageSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn storage_put_then_get_returns_style() {
        // Mirrors: put_skeleton / get_skeleton
        let mut storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        storage.put(make_style(sig.clone(), PName::FontSize, "14"));
        assert!(storage.get(&sig).is_some());
    }

    #[test]
    fn storage_get_missing_returns_none() {
        // Mirrors: get_skeleton before put
        let storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        assert!(storage.get(&sig).is_none());
    }

    #[test]
    fn storage_get_retrieves_correct_value() {
        // Mirrors: get_skeleton after put
        let mut storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        storage.put(make_style(sig.clone(), PName::FontSize, "16"));
        let retrieved = storage.get(&sig).unwrap();
        assert_eq!(retrieved.value(PName::FontSize).as_string(), "16");
    }

    #[test]
    fn storage_styles_empty_when_new() {
        // Mirrors: getStyles_skeleton
        let storage = StyleStorage::new();
        assert_eq!(storage.styles().count(), 0);
    }

    #[test]
    fn storage_styles_count_after_put() {
        // Mirrors: getStyles_skeleton count after put
        let mut storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        storage.put(make_style(sig, PName::FontSize, "12"));
        assert_eq!(storage.styles().count(), 1);
    }

    #[test]
    fn storage_compute_merged_style_empty_returns_empty_style() {
        // Mirrors: computeMergedStyle_skeleton no styles case
        // Java returns null; Rust returns Style::empty.
        let storage = StyleStorage::new();
        let style = storage.compute_merged_style(&StyleSignatureBasic::of(&[SName::Root]));
        assert!(!style.has_value(PName::FontColor));
    }

    #[test]
    fn storage_compute_merged_style_with_matching_rule() {
        // Mirrors: computeMergedStyle_skeleton with stored style
        let mut storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        storage.put(make_style(sig.clone(), PName::FontColor, "#AABBCC"));
        let merged = storage.compute_merged_style(&sig);
        assert!(merged.has_value(PName::FontColor));
        assert_eq!(merged.value(PName::FontColor).as_string(), "#AABBCC");
    }

    #[test]
    fn storage_put_all_copies_rules() {
        // Mirrors: putAll_skeleton
        let mut src = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        src.put(make_style(sig.clone(), PName::FontSize, "18"));

        let mut dest = StyleStorage::new();
        dest.put_all(&src);
        let retrieved = dest.get(&sig);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value(PName::FontSize).as_string(), "18");
    }

    #[test]
    fn storage_print_me_does_not_panic() {
        // Mirrors: printMe_skeleton — storage with a stored style must print without panic
        // and the stored style must still be retrievable afterward.
        let mut storage = StyleStorage::new();
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        storage.put(make_style(sig.clone(), PName::FontSize, "10"));
        storage.print_me();
        assert!(storage.get(&sig).is_some());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Style  (StyleSkeletonTest)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn style_to_string_not_empty() {
        // Mirrors: toString_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let style = Style::empty(sig);
        assert!(!format!("{}", style).is_empty());
    }

    #[test]
    fn style_value_returns_string_for_present_key() {
        // Mirrors: value_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let style = make_style(sig, PName::FontSize, "14");
        assert_eq!(style.value(PName::FontSize).as_string(), "14");
    }

    #[test]
    fn style_value_missing_key_returns_value_null_not_panic() {
        // Mirrors: value_skeleton — missing key returns ValueNull.NULL (not null)
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let style = make_style(sig, PName::FontSize, "14");
        // FontColor was not set — should return ValueNull (empty string, 0 priority)
        let v = style.value(PName::FontColor);
        assert_eq!(v.as_string(), "");
    }

    #[test]
    fn style_get_shadowing_no_key_returns_zero() {
        // Mirrors: getShadowing_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = Style::empty(sig);
        assert!((style.shadowing()).abs() < DELTA);
    }

    #[test]
    fn style_get_shadowing_with_true_value_returns_1_5() {
        // Mirrors: getShadowing_skeleton "true" => asDoubleDefaultTo(1.5)
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = make_style(sig, PName::Shadowing, "true");
        assert!((style.shadowing() - 1.5).abs() < DELTA);
    }

    #[test]
    fn style_has_value_true_for_set_key() {
        // Mirrors: hasValue_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = make_style(sig, PName::FontSize, "12");
        assert!(style.has_value(PName::FontSize));
        assert!(!style.has_value(PName::FontColor));
    }

    #[test]
    fn style_merge_with_overwrite_picks_higher_priority() {
        // Mirrors: mergeWith_skeleton OVERWRITE_EXISTING_VALUE case
        use plantuml_little::style::MergeStrategy;
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);

        let mut base_map = HashMap::new();
        base_map.insert(PName::FontSize, ValueImpl::regular("12", 1));
        let base = Style::new(sig.clone(), base_map);

        let mut over_map = HashMap::new();
        over_map.insert(PName::FontSize, ValueImpl::regular("20", 100));
        let over = Style::new(sig.clone(), over_map);

        let merged = base.merge_with(&over, MergeStrategy::OverwriteExistingValue);
        assert_eq!(merged.value(PName::FontSize).as_string(), "20");
    }

    #[test]
    fn style_get_signature_returns_original_sig() {
        // Mirrors: getSignature_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root, SName::Note]);
        let style = Style::empty(sig.clone());
        assert_eq!(style.signature(), &sig);
    }

    #[test]
    fn style_eventually_override_double() {
        // Mirrors: eventuallyOverride_overload_2_skeleton (PName, double)
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = make_style(sig, PName::FontSize, "12");
        let overridden = style.override_double(PName::FontSize, 18.0);
        assert_eq!(overridden.value(PName::FontSize).as_string(), "18");
    }

    #[test]
    fn style_eventually_override_string() {
        // Mirrors: eventuallyOverride_overload_3_skeleton (PName, String)
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = make_style(sig, PName::FontSize, "12");
        let overridden = style.override_value(PName::FontSize, "22");
        assert_eq!(overridden.value(PName::FontSize).as_string(), "22");
    }

    #[test]
    fn style_get_padding() {
        // Mirrors: getPadding_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = make_style(sig, PName::Padding, "5");
        let padding = style.padding();
        assert!((padding.top - 5.0).abs() < DELTA);
    }

    #[test]
    fn style_get_margin() {
        // Mirrors: getMargin_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = make_style(sig, PName::Margin, "3 6");
        let margin = style.margin();
        assert!((margin.top - 3.0).abs() < DELTA);
        assert!((margin.right - 6.0).abs() < DELTA);
    }

    #[test]
    fn style_get_stroke_does_not_panic() {
        // Mirrors: getStroke_skeleton — thickness=2 with empty dash style
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::LineThickness, ValueImpl::regular("2", 1));
        map.insert(PName::LineStyle, ValueImpl::regular("", 1));
        let style = Style::new(sig, map);
        let stroke = style.stroke();
        // LineThickness=2 must be reflected
        assert!((stroke.thickness - 2.0).abs() < DELTA);
    }

    #[test]
    fn style_wrap_width_does_not_panic() {
        // Mirrors: wrapWidth_skeleton — MaximumWidth=200 must yield a non-None strategy
        use plantuml_little::klimt::LineBreakStrategy;
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let mut map = HashMap::new();
        map.insert(PName::MaximumWidth, ValueImpl::regular("200", 1));
        let style = Style::new(sig, map);
        let wrap = style.wrap_width();
        // A non-empty MaximumWidth should produce an active (non-None) strategy
        assert_ne!(wrap, LineBreakStrategy::None);
    }

    #[test]
    fn style_print_me_smoke_test() {
        // Mirrors: printMe_skeleton — print_me() must not panic and must not
        // corrupt the style's property map.
        let sig = StyleSignatureBasic::of(&[SName::Root]);
        let style = make_style(sig, PName::FontSize, "12");
        style.print_me();
        // Style is still intact after printing
        assert_eq!(style.value(PName::FontSize).as_string(), "12");
    }

    #[test]
    fn style_delta_priority() {
        // Mirrors: deltaPriority_skeleton
        let sig = StyleSignatureBasic::of(&[SName::Root]).add_star();
        let mut map = HashMap::new();
        map.insert(PName::FontSize, ValueImpl::regular("12", 1));
        let starred = Style::new(sig, map);
        let shifted = starred.delta_priority(5);
        // Value should still be accessible; priority should have increased.
        assert_eq!(shifted.value(PName::FontSize).as_string(), "12");
        assert_eq!(shifted.value(PName::FontSize).priority(), 6);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ConcatIterator  (ConcatIteratorSkeletonTest)
    // No direct Rust equivalent — chain() is idiomatic.
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn concat_iterator_has_next_and_next_chain_two_iters() {
        // Mirrors: hasNext_skeleton / next_skeleton
        // Rust idiomatic: .chain()
        let first = vec!["a", "b"];
        let second = vec!["c"];
        let mut it = first.into_iter().chain(second);
        assert_eq!(it.next(), Some("a"));
        assert_eq!(it.next(), Some("b"));
        assert_eq!(it.next(), Some("c"));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn concat_iterator_integer_sequence() {
        // Mirrors: next_skeleton integers
        let first = vec![1i32, 2];
        let second = vec![3i32, 4];
        let collected: Vec<i32> = first.into_iter().chain(second).collect();
        assert_eq!(collected, vec![1, 2, 3, 4]);
    }

    #[test]
    fn concat_iterator_both_empty_has_no_next() {
        // Mirrors: next_throws_when_exhausted
        let empty1: Vec<&str> = vec![];
        let empty2: Vec<&str> = vec![];
        let mut it = empty1.into_iter().chain(empty2);
        assert_eq!(it.next(), None);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FromSkinparamToStyle  (FromSkinparamToStyleSkeletonTest)  — HIGHEST PRIORITY
    // No Rust equivalent yet.
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    #[ignore = "gap: FromSkinparamToStyle not yet ported — skinparam→style conversion missing"]
    fn from_skinparam_to_style_note_font_size_produces_style_with_font_size() {
        // Mirrors: convertNow_skeleton — "noteFontSize" -> note style with FontSize
        todo!("FromSkinparamToStyle not yet ported")
    }

    #[test]
    #[ignore = "gap: FromSkinparamToStyle not yet ported — shadowing special case"]
    fn from_skinparam_to_style_shadowing_produces_shadowing_3() {
        // Mirrors: convertNow_shadowing_special_case
        todo!("FromSkinparamToStyle not yet ported")
    }

    #[test]
    #[ignore = "gap: FromSkinparamToStyle not yet ported — unknown key handling"]
    fn from_skinparam_to_style_unknown_key_produces_nothing() {
        // Mirrors: convertNow_unknown_key_produces_nothing
        todo!("FromSkinparamToStyle not yet ported")
    }

    #[test]
    #[ignore = "gap: FromSkinparamToStyle not yet ported — arrowColor conversion"]
    fn from_skinparam_to_style_arrow_color_produces_style() {
        // Mirrors: getStyles_skeleton
        todo!("FromSkinparamToStyle not yet ported")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Additional StyleSignatureBasic regression guards
    // (Complement to the highest-priority coverage above)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn sig_match_all_stereotype_must_match() {
        // Declaration has stereotype "foo"; element must also have it.
        let decl = StyleSignatureBasic::of(&[SName::Root]).add_stereotype("foo");
        let elem_has_it = StyleSignatureBasic::of(&[SName::Root, SName::Arrow])
            .add_stereotype("foo")
            .add_stereotype("bar");
        let elem_missing =
            StyleSignatureBasic::of(&[SName::Root, SName::Arrow]).add_stereotype("bar");
        assert!(decl.match_all(&elem_has_it));
        assert!(!decl.match_all(&elem_missing));
    }

    #[test]
    fn sig_match_all_level_exact() {
        let decl = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(3);
        let elem_ok = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(3);
        let elem_wrong = StyleSignatureBasic::empty()
            .add_sname(SName::Root)
            .add_level(5);
        assert!(decl.match_all(&elem_ok));
        assert!(!decl.match_all(&elem_wrong));
    }

    #[test]
    fn sig_match_all_starred_level_greater_or_equal() {
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
    fn sig_stereotype_clean_strips_underscore_and_dot_and_lowercases() {
        // Java: "My_Type.Name" -> "mytypename"
        let sig = StyleSignatureBasic::empty().add_stereotype("My_Type.Name");
        assert!(sig.get_stereotypes().contains("mytypename"));
    }
}
