// Port of Java decoration-package skeleton tests to Rust.
// Source: generated-tests/.../decoration/
//
// Mapping notes:
//   Java LinkDecor               -> decoration::LinkDecor
//   Java LinkMiddleDecor         -> decoration::LinkMiddleDecor
//   Java LinkStyle               -> decoration::LinkStyle
//   Java LinkType                -> decoration::LinkType
//   Java HtmlColorAndStyle       -> not yet ported
//   Java Rainbow                 -> not yet ported
//   Java WithLinkType            -> not yet ported (abstract base class)
//   Java USymbol / USymbolKind   -> decoration::symbol::USymbolKind

// ═══════════════════════════════════════════════════════════════════════════
// LinkDecorSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod link_decor_tests {
    use plantuml_little::decoration::LinkDecor;
    use plantuml_little::decoration::link_decor::ExtremityKind;

    // ── Variant count ──────────────────────────────────────────────────────

    #[test]
    fn all_25_variants_distinct() {
        // Java: assertEquals(25, LinkDecor.values().length)
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
        assert_eq!(variants.len(), 25);
        // All variants are pair-wise distinct
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn default_is_none() {
        assert_eq!(LinkDecor::default(), LinkDecor::None);
    }

    // ── Variant debug names ────────────────────────────────────────────────

    #[test]
    fn variant_names_match_enum() {
        assert_eq!(format!("{:?}", LinkDecor::None), "None");
        assert_eq!(format!("{:?}", LinkDecor::Extends), "Extends");
        assert_eq!(format!("{:?}", LinkDecor::Composition), "Composition");
        assert_eq!(format!("{:?}", LinkDecor::Arrow), "Arrow");
        assert_eq!(format!("{:?}", LinkDecor::Plus), "Plus");
        assert_eq!(format!("{:?}", LinkDecor::HalfArrowUp), "HalfArrowUp");
        assert_eq!(format!("{:?}", LinkDecor::HalfArrowDown), "HalfArrowDown");
        assert_eq!(format!("{:?}", LinkDecor::SquareToBeRemoved), "SquareToBeRemoved");
    }

    // ── Margin metadata ────────────────────────────────────────────────────

    #[test]
    fn none_margin_is_2() {
        assert_eq!(LinkDecor::None.margin(), 2);
    }

    #[test]
    fn extends_margin_is_30() {
        assert_eq!(LinkDecor::Extends.margin(), 30);
    }

    #[test]
    fn composition_margin_is_15() {
        assert_eq!(LinkDecor::Composition.margin(), 15);
    }

    #[test]
    fn agregation_margin_is_15() {
        assert_eq!(LinkDecor::Agregation.margin(), 15);
    }

    #[test]
    fn not_navigable_margin_is_1() {
        assert_eq!(LinkDecor::NotNavigable.margin(), 1);
    }

    #[test]
    fn redefines_margin_is_30() {
        assert_eq!(LinkDecor::Redefines.margin(), 30);
    }

    #[test]
    fn defined_by_margin_is_30() {
        assert_eq!(LinkDecor::DefinedBy.margin(), 30);
    }

    #[test]
    fn crowfoot_margin_is_10() {
        assert_eq!(LinkDecor::Crowfoot.margin(), 10);
    }

    #[test]
    fn circle_crowfoot_margin_is_14() {
        assert_eq!(LinkDecor::CircleCrowfoot.margin(), 14);
    }

    #[test]
    fn circle_line_margin_is_10() {
        assert_eq!(LinkDecor::CircleLine.margin(), 10);
    }

    #[test]
    fn double_line_margin_is_7() {
        assert_eq!(LinkDecor::DoubleLine.margin(), 7);
    }

    #[test]
    fn line_crowfoot_margin_is_10() {
        assert_eq!(LinkDecor::LineCrowfoot.margin(), 10);
    }

    #[test]
    fn arrow_margin_is_10() {
        assert_eq!(LinkDecor::Arrow.margin(), 10);
    }

    #[test]
    fn arrow_triangle_margin_is_10() {
        assert_eq!(LinkDecor::ArrowTriangle.margin(), 10);
    }

    #[test]
    fn arrow_and_circle_margin_is_10() {
        assert_eq!(LinkDecor::ArrowAndCircle.margin(), 10);
    }

    #[test]
    fn circle_margin_is_0() {
        assert_eq!(LinkDecor::Circle.margin(), 0);
    }

    #[test]
    fn circle_fill_margin_is_0() {
        assert_eq!(LinkDecor::CircleFill.margin(), 0);
    }

    #[test]
    fn circle_connect_margin_is_0() {
        assert_eq!(LinkDecor::CircleConnect.margin(), 0);
    }

    #[test]
    fn parenthesis_margin_is_0() {
        assert_eq!(LinkDecor::Parenthesis.margin(), 0);
    }

    #[test]
    fn square_margin_is_0() {
        assert_eq!(LinkDecor::Square.margin(), 0);
    }

    #[test]
    fn circle_cross_margin_is_0() {
        assert_eq!(LinkDecor::CircleCross.margin(), 0);
    }

    #[test]
    fn plus_margin_is_0() {
        assert_eq!(LinkDecor::Plus.margin(), 0);
    }

    #[test]
    fn half_arrow_up_margin_is_0() {
        assert_eq!(LinkDecor::HalfArrowUp.margin(), 0);
    }

    #[test]
    fn half_arrow_down_margin_is_0() {
        assert_eq!(LinkDecor::HalfArrowDown.margin(), 0);
    }

    #[test]
    fn square_to_be_removed_margin_is_30() {
        assert_eq!(LinkDecor::SquareToBeRemoved.margin(), 30);
    }

    // ── Fill metadata ──────────────────────────────────────────────────────

    #[test]
    fn none_is_not_fill() {
        assert_eq!(LinkDecor::None.is_fill(), false);
    }

    #[test]
    fn extends_is_not_fill() {
        assert_eq!(LinkDecor::Extends.is_fill(), false);
    }

    #[test]
    fn composition_is_fill() {
        assert_eq!(LinkDecor::Composition.is_fill(), true);
    }

    #[test]
    fn agregation_is_not_fill() {
        assert_eq!(LinkDecor::Agregation.is_fill(), false);
    }

    #[test]
    fn crowfoot_is_fill() {
        assert_eq!(LinkDecor::Crowfoot.is_fill(), true);
    }

    #[test]
    fn arrow_is_fill() {
        assert_eq!(LinkDecor::Arrow.is_fill(), true);
    }

    #[test]
    fn arrow_triangle_is_fill() {
        assert_eq!(LinkDecor::ArrowTriangle.is_fill(), true);
    }

    #[test]
    fn circle_crowfoot_is_not_fill() {
        assert_eq!(LinkDecor::CircleCrowfoot.is_fill(), false);
    }

    #[test]
    fn plus_is_not_fill() {
        assert_eq!(LinkDecor::Plus.is_fill(), false);
    }

    // ── Arrow-size metadata ────────────────────────────────────────────────

    #[test]
    fn none_arrow_size_is_zero() {
        assert_eq!(LinkDecor::None.arrow_size(), 0.0);
    }

    #[test]
    fn extends_arrow_size_is_2() {
        assert_eq!(LinkDecor::Extends.arrow_size(), 2.0);
    }

    #[test]
    fn composition_arrow_size_is_1_3() {
        assert_eq!(LinkDecor::Composition.arrow_size(), 1.3);
    }

    #[test]
    fn arrow_arrow_size_is_0_5() {
        assert_eq!(LinkDecor::Arrow.arrow_size(), 0.5);
    }

    #[test]
    fn crowfoot_arrow_size_is_0_8() {
        assert_eq!(LinkDecor::Crowfoot.arrow_size(), 0.8);
    }

    #[test]
    fn plus_arrow_size_is_1_5() {
        assert_eq!(LinkDecor::Plus.arrow_size(), 1.5);
    }

    #[test]
    fn half_arrow_up_arrow_size_is_1_5() {
        assert_eq!(LinkDecor::HalfArrowUp.arrow_size(), 1.5);
    }

    #[test]
    fn half_arrow_down_arrow_size_is_1_5() {
        assert_eq!(LinkDecor::HalfArrowDown.arrow_size(), 1.5);
    }

    #[test]
    fn double_line_arrow_size_is_0_7() {
        assert_eq!(LinkDecor::DoubleLine.arrow_size(), 0.7);
    }

    #[test]
    fn square_to_be_removed_arrow_size_is_zero() {
        assert_eq!(LinkDecor::SquareToBeRemoved.arrow_size(), 0.0);
    }

    // ── is_extends_like ────────────────────────────────────────────────────

    #[test]
    fn extends_is_extends_like() {
        assert!(LinkDecor::Extends.is_extends_like());
    }

    #[test]
    fn redefines_is_extends_like() {
        assert!(LinkDecor::Redefines.is_extends_like());
    }

    #[test]
    fn defined_by_is_extends_like() {
        assert!(LinkDecor::DefinedBy.is_extends_like());
    }

    #[test]
    fn arrow_is_not_extends_like() {
        assert_eq!(LinkDecor::Arrow.is_extends_like(), false);
    }

    #[test]
    fn none_is_not_extends_like() {
        assert_eq!(LinkDecor::None.is_extends_like(), false);
    }

    #[test]
    fn composition_is_not_extends_like() {
        assert_eq!(LinkDecor::Composition.is_extends_like(), false);
    }

    #[test]
    fn crowfoot_is_not_extends_like() {
        assert_eq!(LinkDecor::Crowfoot.is_extends_like(), false);
    }

    // ── lookup_decors1 ─────────────────────────────────────────────────────

    #[test]
    fn lookup1_extends_from_caret_and_pipe_angle() {
        assert_eq!(LinkDecor::lookup_decors1("<|"), LinkDecor::Extends);
        assert_eq!(LinkDecor::lookup_decors1("^"), LinkDecor::Extends);
    }

    #[test]
    fn lookup1_composition_from_star() {
        assert_eq!(LinkDecor::lookup_decors1("*"), LinkDecor::Composition);
    }

    #[test]
    fn lookup1_agregation_from_o() {
        assert_eq!(LinkDecor::lookup_decors1("o"), LinkDecor::Agregation);
    }

    #[test]
    fn lookup1_not_navigable_from_x() {
        assert_eq!(LinkDecor::lookup_decors1("x"), LinkDecor::NotNavigable);
    }

    #[test]
    fn lookup1_redefines_from_double_pipe_angle() {
        assert_eq!(LinkDecor::lookup_decors1("<||"), LinkDecor::Redefines);
    }

    #[test]
    fn lookup1_defined_by_from_angle_pipe_colon() {
        assert_eq!(LinkDecor::lookup_decors1("<|:"), LinkDecor::DefinedBy);
    }

    #[test]
    fn lookup1_crowfoot_from_brace() {
        assert_eq!(LinkDecor::lookup_decors1("}"), LinkDecor::Crowfoot);
    }

    #[test]
    fn lookup1_circle_crowfoot() {
        assert_eq!(LinkDecor::lookup_decors1("}o"), LinkDecor::CircleCrowfoot);
    }

    #[test]
    fn lookup1_circle_line() {
        assert_eq!(LinkDecor::lookup_decors1("|o"), LinkDecor::CircleLine);
    }

    #[test]
    fn lookup1_double_line() {
        assert_eq!(LinkDecor::lookup_decors1("||"), LinkDecor::DoubleLine);
    }

    #[test]
    fn lookup1_line_crowfoot() {
        assert_eq!(LinkDecor::lookup_decors1("}|"), LinkDecor::LineCrowfoot);
    }

    #[test]
    fn lookup1_arrow_from_angle() {
        assert_eq!(LinkDecor::lookup_decors1("<"), LinkDecor::Arrow);
        assert_eq!(LinkDecor::lookup_decors1("<_"), LinkDecor::Arrow);
    }

    #[test]
    fn lookup1_arrow_triangle_from_double_angle() {
        assert_eq!(LinkDecor::lookup_decors1("<<"), LinkDecor::ArrowTriangle);
    }

    #[test]
    fn lookup1_circle_from_zero() {
        assert_eq!(LinkDecor::lookup_decors1("0"), LinkDecor::Circle);
    }

    #[test]
    fn lookup1_circle_fill_from_at() {
        assert_eq!(LinkDecor::lookup_decors1("@"), LinkDecor::CircleFill);
    }

    #[test]
    fn lookup1_circle_connect_from_zero_paren() {
        assert_eq!(LinkDecor::lookup_decors1("0)"), LinkDecor::CircleConnect);
    }

    #[test]
    fn lookup1_parenthesis_from_close_paren() {
        assert_eq!(LinkDecor::lookup_decors1(")"), LinkDecor::Parenthesis);
    }

    #[test]
    fn lookup1_square_from_hash() {
        assert_eq!(LinkDecor::lookup_decors1("#"), LinkDecor::Square);
    }

    #[test]
    fn lookup1_plus_from_plus() {
        assert_eq!(LinkDecor::lookup_decors1("+"), LinkDecor::Plus);
    }

    #[test]
    fn lookup1_unknown_returns_none() {
        assert_eq!(LinkDecor::lookup_decors1("???"), LinkDecor::None);
        assert_eq!(LinkDecor::lookup_decors1(""), LinkDecor::None);
        assert_eq!(LinkDecor::lookup_decors1("xyz"), LinkDecor::None);
    }

    #[test]
    fn lookup1_trims_surrounding_whitespace() {
        assert_eq!(LinkDecor::lookup_decors1(" < "), LinkDecor::Arrow);
        assert_eq!(LinkDecor::lookup_decors1(" * "), LinkDecor::Composition);
    }

    // ── lookup_decors2 ─────────────────────────────────────────────────────

    #[test]
    fn lookup2_extends_from_pipe_angle_and_caret() {
        assert_eq!(LinkDecor::lookup_decors2("|>"), LinkDecor::Extends);
        assert_eq!(LinkDecor::lookup_decors2("^"), LinkDecor::Extends);
    }

    #[test]
    fn lookup2_composition_from_star() {
        assert_eq!(LinkDecor::lookup_decors2("*"), LinkDecor::Composition);
    }

    #[test]
    fn lookup2_agregation_from_o() {
        assert_eq!(LinkDecor::lookup_decors2("o"), LinkDecor::Agregation);
    }

    #[test]
    fn lookup2_not_navigable_from_x() {
        assert_eq!(LinkDecor::lookup_decors2("x"), LinkDecor::NotNavigable);
    }

    #[test]
    fn lookup2_redefines() {
        assert_eq!(LinkDecor::lookup_decors2("||>"), LinkDecor::Redefines);
    }

    #[test]
    fn lookup2_defined_by() {
        assert_eq!(LinkDecor::lookup_decors2(":|>"), LinkDecor::DefinedBy);
    }

    #[test]
    fn lookup2_crowfoot_variants() {
        assert_eq!(LinkDecor::lookup_decors2("{"), LinkDecor::Crowfoot);
        assert_eq!(LinkDecor::lookup_decors2("o{"), LinkDecor::CircleCrowfoot);
        assert_eq!(LinkDecor::lookup_decors2("o|"), LinkDecor::CircleLine);
        assert_eq!(LinkDecor::lookup_decors2("||"), LinkDecor::DoubleLine);
        assert_eq!(LinkDecor::lookup_decors2("|{"), LinkDecor::LineCrowfoot);
    }

    #[test]
    fn lookup2_arrow_from_angle() {
        assert_eq!(LinkDecor::lookup_decors2(">"), LinkDecor::Arrow);
        assert_eq!(LinkDecor::lookup_decors2("_>"), LinkDecor::Arrow);
    }

    #[test]
    fn lookup2_arrow_triangle_from_double_angle() {
        assert_eq!(LinkDecor::lookup_decors2(">>"), LinkDecor::ArrowTriangle);
    }

    #[test]
    fn lookup2_circle_connect_from_paren_zero() {
        assert_eq!(LinkDecor::lookup_decors2("(0"), LinkDecor::CircleConnect);
    }

    #[test]
    fn lookup2_parenthesis_from_open_paren() {
        assert_eq!(LinkDecor::lookup_decors2("("), LinkDecor::Parenthesis);
    }

    #[test]
    fn lookup2_square_and_plus() {
        assert_eq!(LinkDecor::lookup_decors2("#"), LinkDecor::Square);
        assert_eq!(LinkDecor::lookup_decors2("+"), LinkDecor::Plus);
    }

    #[test]
    fn lookup2_half_arrows() {
        assert_eq!(LinkDecor::lookup_decors2("\\\\"), LinkDecor::HalfArrowUp);
        assert_eq!(LinkDecor::lookup_decors2("//"), LinkDecor::HalfArrowDown);
    }

    #[test]
    fn lookup2_unknown_returns_none() {
        assert_eq!(LinkDecor::lookup_decors2("???"), LinkDecor::None);
        assert_eq!(LinkDecor::lookup_decors2(""), LinkDecor::None);
    }

    #[test]
    fn lookup2_trims_surrounding_whitespace() {
        assert_eq!(LinkDecor::lookup_decors2(" > "), LinkDecor::Arrow);
    }

    // ── regex_decors ───────────────────────────────────────────────────────

    #[test]
    fn regex_decors1_is_valid_regex_wrapped_in_optional_group() {
        let pat = LinkDecor::regex_decors1();
        assert!(pat.starts_with('('), "should start with '('");
        assert!(pat.ends_with(")?"), "should end with ')?'");
        regex::Regex::new(&pat).expect("regex_decors1 must produce a valid regex");
    }

    #[test]
    fn regex_decors2_is_valid_regex_wrapped_in_optional_group() {
        let pat = LinkDecor::regex_decors2();
        assert!(pat.starts_with('('), "should start with '('");
        assert!(pat.ends_with(")?"), "should end with ')?'");
        regex::Regex::new(&pat).expect("regex_decors2 must produce a valid regex");
    }

    #[test]
    fn regex_decors1_matches_known_symbols() {
        let pat = LinkDecor::regex_decors1();
        let re = regex::Regex::new(&pat).unwrap();
        // Arrow "<" should match when it appears at the start of a string
        assert!(re.is_match("<-"), "should match arrow '<'");
        assert!(re.is_match("<<"), "should match double-angle '<<'");
    }

    #[test]
    fn regex_decors2_matches_known_symbols() {
        let pat = LinkDecor::regex_decors2();
        let re = regex::Regex::new(&pat).unwrap();
        assert!(re.is_match(">"), "should match arrow '>'");
    }

    // ── extremity_kind ─────────────────────────────────────────────────────

    #[test]
    fn extremity_kind_none_returns_none() {
        assert!(LinkDecor::None.extremity_kind().is_none());
    }

    #[test]
    fn extremity_kind_square_to_be_removed_returns_none() {
        assert!(LinkDecor::SquareToBeRemoved.extremity_kind().is_none());
    }

    #[test]
    fn extremity_kind_extends_is_large_triangle() {
        match LinkDecor::Extends.extremity_kind() {
            Some(ExtremityKind::Triangle { w, h, len }) => {
                assert_eq!(w, 18.0);
                assert_eq!(h, 6.0);
                assert_eq!(len, 18.0);
            }
            other => panic!("expected Triangle(18,6,18), got {:?}", other),
        }
    }

    #[test]
    fn extremity_kind_arrow_triangle_is_smaller_triangle() {
        match LinkDecor::ArrowTriangle.extremity_kind() {
            Some(ExtremityKind::Triangle { w, h, len }) => {
                assert_eq!(w, 8.0);
                assert_eq!(h, 3.0);
                assert_eq!(len, 8.0);
            }
            other => panic!("expected Triangle(8,3,8), got {:?}", other),
        }
    }

    #[test]
    fn extremity_kind_composition_is_filled_diamond() {
        assert_eq!(
            LinkDecor::Composition.extremity_kind(),
            Some(ExtremityKind::Diamond { filled: true })
        );
    }

    #[test]
    fn extremity_kind_agregation_is_hollow_diamond() {
        assert_eq!(
            LinkDecor::Agregation.extremity_kind(),
            Some(ExtremityKind::Diamond { filled: false })
        );
    }

    #[test]
    fn extremity_kind_circle_is_hollow() {
        assert_eq!(
            LinkDecor::Circle.extremity_kind(),
            Some(ExtremityKind::Circle { filled: false })
        );
    }

    #[test]
    fn extremity_kind_circle_fill_is_filled() {
        assert_eq!(
            LinkDecor::CircleFill.extremity_kind(),
            Some(ExtremityKind::Circle { filled: true })
        );
    }

    #[test]
    fn extremity_kind_redefines_has_no_dot() {
        assert_eq!(
            LinkDecor::Redefines.extremity_kind(),
            Some(ExtremityKind::ExtendsLike { has_dot: false })
        );
    }

    #[test]
    fn extremity_kind_defined_by_has_dot() {
        assert_eq!(
            LinkDecor::DefinedBy.extremity_kind(),
            Some(ExtremityKind::ExtendsLike { has_dot: true })
        );
    }

    #[test]
    fn extremity_kind_half_arrow_up_direction_1() {
        assert_eq!(
            LinkDecor::HalfArrowUp.extremity_kind(),
            Some(ExtremityKind::HalfArrow { direction: 1 })
        );
    }

    #[test]
    fn extremity_kind_half_arrow_down_direction_neg1() {
        assert_eq!(
            LinkDecor::HalfArrowDown.extremity_kind(),
            Some(ExtremityKind::HalfArrow { direction: -1 })
        );
    }

    #[test]
    fn extremity_kind_arrow_is_arrow() {
        assert_eq!(LinkDecor::Arrow.extremity_kind(), Some(ExtremityKind::Arrow));
    }

    #[test]
    fn extremity_kind_crowfoot_variants() {
        assert_eq!(LinkDecor::Crowfoot.extremity_kind(), Some(ExtremityKind::Crowfoot));
        assert_eq!(
            LinkDecor::CircleCrowfoot.extremity_kind(),
            Some(ExtremityKind::CircleCrowfoot)
        );
        assert_eq!(
            LinkDecor::LineCrowfoot.extremity_kind(),
            Some(ExtremityKind::LineCrowfoot)
        );
    }

    #[test]
    fn extremity_kind_remaining_single_variants() {
        assert_eq!(LinkDecor::CircleLine.extremity_kind(), Some(ExtremityKind::CircleLine));
        assert_eq!(LinkDecor::DoubleLine.extremity_kind(), Some(ExtremityKind::DoubleLine));
        assert_eq!(LinkDecor::CircleCross.extremity_kind(), Some(ExtremityKind::CircleCross));
        assert_eq!(
            LinkDecor::ArrowAndCircle.extremity_kind(),
            Some(ExtremityKind::ArrowAndCircle)
        );
        assert_eq!(
            LinkDecor::NotNavigable.extremity_kind(),
            Some(ExtremityKind::NotNavigable)
        );
        assert_eq!(LinkDecor::Plus.extremity_kind(), Some(ExtremityKind::Plus));
        assert_eq!(LinkDecor::Square.extremity_kind(), Some(ExtremityKind::Square));
        assert_eq!(
            LinkDecor::Parenthesis.extremity_kind(),
            Some(ExtremityKind::Parenthesis)
        );
        assert_eq!(
            LinkDecor::CircleConnect.extremity_kind(),
            Some(ExtremityKind::CircleConnect)
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LinkMiddleDecorSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod link_middle_decor_tests {
    use plantuml_little::decoration::LinkMiddleDecor;

    // ── Variant count ──────────────────────────────────────────────────────

    #[test]
    fn all_7_variants_distinct() {
        // Java: assertEquals(7, LinkMiddleDecor.values().length)
        let variants = [
            LinkMiddleDecor::None,
            LinkMiddleDecor::Circle,
            LinkMiddleDecor::CircleCircled,
            LinkMiddleDecor::CircleCircled1,
            LinkMiddleDecor::CircleCircled2,
            LinkMiddleDecor::Subset,
            LinkMiddleDecor::Superset,
        ];
        assert_eq!(variants.len(), 7);
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn default_is_none() {
        assert_eq!(LinkMiddleDecor::default(), LinkMiddleDecor::None);
    }

    // ── inversed ──────────────────────────────────────────────────────────

    #[test]
    fn inversed_circled1_gives_circled2() {
        // Java: assertEquals(CIRCLE_CIRCLED2, CIRCLE_CIRCLED1.getInversed())
        assert_eq!(
            LinkMiddleDecor::CircleCircled1.inversed(),
            LinkMiddleDecor::CircleCircled2
        );
    }

    #[test]
    fn inversed_circled2_gives_circled1() {
        // Java: assertEquals(CIRCLE_CIRCLED1, CIRCLE_CIRCLED2.getInversed())
        assert_eq!(
            LinkMiddleDecor::CircleCircled2.inversed(),
            LinkMiddleDecor::CircleCircled1
        );
    }

    #[test]
    fn inversed_none_is_identity() {
        assert_eq!(LinkMiddleDecor::None.inversed(), LinkMiddleDecor::None);
    }

    #[test]
    fn inversed_circle_is_identity() {
        assert_eq!(LinkMiddleDecor::Circle.inversed(), LinkMiddleDecor::Circle);
    }

    #[test]
    fn inversed_circle_circled_is_identity() {
        assert_eq!(
            LinkMiddleDecor::CircleCircled.inversed(),
            LinkMiddleDecor::CircleCircled
        );
    }

    #[test]
    fn inversed_subset_is_identity() {
        assert_eq!(LinkMiddleDecor::Subset.inversed(), LinkMiddleDecor::Subset);
    }

    #[test]
    fn inversed_superset_is_identity() {
        assert_eq!(LinkMiddleDecor::Superset.inversed(), LinkMiddleDecor::Superset);
    }

    #[test]
    fn inversed_is_self_inverse_for_circled_pair() {
        // Applying inversed twice returns the original
        let start = LinkMiddleDecor::CircleCircled1;
        assert_eq!(start.inversed().inversed(), start);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LinkStyleSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod link_style_tests {
    use plantuml_little::decoration::{LinkStyle, LinkStyleKind};
    use plantuml_little::klimt::UStroke;

    // ── Factory constructors ───────────────────────────────────────────────

    #[test]
    fn normal_kind_is_normal() {
        assert_eq!(LinkStyle::normal().kind(), LinkStyleKind::Normal);
    }

    #[test]
    fn dashed_kind_is_dashed() {
        assert_eq!(LinkStyle::dashed().kind(), LinkStyleKind::Dashed);
    }

    #[test]
    fn dotted_kind_is_dotted() {
        assert_eq!(LinkStyle::dotted().kind(), LinkStyleKind::Dotted);
    }

    #[test]
    fn bold_kind_is_bold() {
        assert_eq!(LinkStyle::bold().kind(), LinkStyleKind::Bold);
    }

    #[test]
    fn invisible_kind_is_invisible() {
        assert_eq!(LinkStyle::invisible().kind(), LinkStyleKind::Invisible);
    }

    // ── Boolean queries ────────────────────────────────────────────────────

    #[test]
    fn normal_is_normal_and_not_invisible() {
        let s = LinkStyle::normal();
        assert!(s.is_normal());
        assert_eq!(s.is_invisible(), false);
    }

    #[test]
    fn invisible_is_invisible_and_not_normal() {
        let s = LinkStyle::invisible();
        assert!(s.is_invisible());
        assert_eq!(s.is_normal(), false);
    }

    #[test]
    fn dashed_is_not_normal_and_not_invisible() {
        let s = LinkStyle::dashed();
        assert_eq!(s.is_normal(), false);
        assert_eq!(s.is_invisible(), false);
    }

    #[test]
    fn thickness_not_overridden_by_default() {
        assert_eq!(LinkStyle::normal().is_thickness_overridden(), false);
        assert_eq!(LinkStyle::dashed().is_thickness_overridden(), false);
        assert_eq!(LinkStyle::bold().is_thickness_overridden(), false);
    }

    #[test]
    fn go_thickness_marks_as_overridden() {
        let s = LinkStyle::normal().go_thickness(3.0);
        assert!(s.is_thickness_overridden());
    }

    // ── get_stroke3 behavior ───────────────────────────────────────────────

    #[test]
    fn normal_stroke_has_thickness_1_no_dasharray() {
        let s = LinkStyle::normal().get_stroke3();
        assert_eq!(s.thickness, 1.0);
        assert!(s.dasharray_svg().is_none());
    }

    #[test]
    fn dashed_stroke_has_7_7_dasharray() {
        let s = LinkStyle::dashed().get_stroke3();
        assert_eq!(s.dasharray_svg(), Some((7.0, 7.0)));
        assert_eq!(s.thickness, 1.0);
    }

    #[test]
    fn dotted_stroke_has_1_3_dasharray() {
        let s = LinkStyle::dotted().get_stroke3();
        assert_eq!(s.dasharray_svg(), Some((1.0, 3.0)));
        assert_eq!(s.thickness, 1.0);
    }

    #[test]
    fn bold_stroke_has_thickness_2_no_dasharray() {
        let s = LinkStyle::bold().get_stroke3();
        assert!(s.dasharray_svg().is_none());
        assert_eq!(s.thickness, 2.0);
    }

    #[test]
    fn go_thickness_overrides_stroke_thickness() {
        let s = LinkStyle::normal().go_thickness(3.5).get_stroke3();
        assert_eq!(s.thickness, 3.5);
    }

    #[test]
    fn dashed_with_thickness_keeps_dasharray_and_new_thickness() {
        let s = LinkStyle::dashed().go_thickness(2.0).get_stroke3();
        assert_eq!(s.dasharray_svg(), Some((7.0, 7.0)));
        assert_eq!(s.thickness, 2.0);
    }

    #[test]
    fn dotted_with_thickness_keeps_dasharray_and_new_thickness() {
        let s = LinkStyle::dotted().go_thickness(1.5).get_stroke3();
        assert_eq!(s.dasharray_svg(), Some((1.0, 3.0)));
        assert_eq!(s.thickness, 1.5);
    }

    // ── mute_stroke ────────────────────────────────────────────────────────

    #[test]
    fn normal_mute_stroke_passes_through_unchanged() {
        let original = UStroke::with_thickness(5.0);
        let result = LinkStyle::normal().mute_stroke(original.clone());
        assert_eq!(result, original);
    }

    #[test]
    fn invisible_mute_stroke_passes_through_unchanged() {
        let original = UStroke::with_thickness(3.0);
        let result = LinkStyle::invisible().mute_stroke(original.clone());
        assert_eq!(result, original);
    }

    #[test]
    fn dashed_mute_stroke_overrides_to_dashed() {
        let original = UStroke::with_thickness(5.0);
        let result = LinkStyle::dashed().mute_stroke(original);
        assert_eq!(result.dasharray_svg(), Some((7.0, 7.0)));
    }

    #[test]
    fn dotted_mute_stroke_overrides_to_dotted() {
        let original = UStroke::with_thickness(5.0);
        let result = LinkStyle::dotted().mute_stroke(original);
        assert_eq!(result.dasharray_svg(), Some((1.0, 3.0)));
    }

    #[test]
    fn bold_mute_stroke_overrides_to_bold() {
        let original = UStroke::with_thickness(1.0);
        let result = LinkStyle::bold().mute_stroke(original);
        assert_eq!(result.thickness, 2.0);
        assert!(result.dasharray_svg().is_none());
    }

    // ── parsing ────────────────────────────────────────────────────────────

    #[test]
    fn from_string1_dashed_case_insensitive() {
        assert_eq!(LinkStyle::from_string1("dashed").kind(), LinkStyleKind::Dashed);
        assert_eq!(LinkStyle::from_string1("DASHED").kind(), LinkStyleKind::Dashed);
        assert_eq!(LinkStyle::from_string1("Dashed").kind(), LinkStyleKind::Dashed);
    }

    #[test]
    fn from_string1_dotted_case_insensitive() {
        assert_eq!(LinkStyle::from_string1("dotted").kind(), LinkStyleKind::Dotted);
        assert_eq!(LinkStyle::from_string1("DOTTED").kind(), LinkStyleKind::Dotted);
    }

    #[test]
    fn from_string1_bold_case_insensitive() {
        assert_eq!(LinkStyle::from_string1("bold").kind(), LinkStyleKind::Bold);
        assert_eq!(LinkStyle::from_string1("Bold").kind(), LinkStyleKind::Bold);
    }

    #[test]
    fn from_string1_hidden_maps_to_invisible() {
        assert!(LinkStyle::from_string1("hidden").is_invisible());
    }

    #[test]
    fn from_string1_unknown_falls_back_to_normal() {
        assert!(LinkStyle::from_string1("unknown").is_normal());
        assert!(LinkStyle::from_string1("").is_normal());
        assert!(LinkStyle::from_string1("plain").is_normal());
    }

    #[test]
    fn from_string2_recognises_known_values() {
        assert!(LinkStyle::from_string2("dashed").is_some());
        assert!(LinkStyle::from_string2("dotted").is_some());
        assert!(LinkStyle::from_string2("bold").is_some());
        assert!(LinkStyle::from_string2("hidden").is_some());
    }

    #[test]
    fn from_string2_returns_none_for_unknown() {
        assert!(LinkStyle::from_string2("plain").is_none());
        assert!(LinkStyle::from_string2("").is_none());
        assert!(LinkStyle::from_string2("normal").is_none());
    }

    // ── Default ────────────────────────────────────────────────────────────

    #[test]
    fn default_is_normal() {
        assert!(LinkStyle::default().is_normal());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LinkTypeSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod link_type_tests {
    use plantuml_little::decoration::{LinkDecor, LinkMiddleDecor, LinkStyleKind, LinkType};
    use plantuml_little::decoration::link_type::LinkStrategy;
    use plantuml_little::klimt::UStroke;

    // ── Construction ───────────────────────────────────────────────────────

    #[test]
    fn new_stores_both_decors() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        assert_eq!(lt.decor1(), LinkDecor::Arrow);
        assert_eq!(lt.decor2(), LinkDecor::Extends);
    }

    #[test]
    fn new_defaults_to_no_middle_decor() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::None);
    }

    #[test]
    fn new_defaults_to_normal_style() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        assert!(lt.style().is_normal());
    }

    // ── is_double_decorated ────────────────────────────────────────────────

    #[test]
    fn both_none_is_not_double_decorated() {
        assert_eq!(
            LinkType::new(LinkDecor::None, LinkDecor::None).is_double_decorated(),
            false
        );
    }

    #[test]
    fn only_decor1_is_not_double_decorated() {
        assert_eq!(
            LinkType::new(LinkDecor::Arrow, LinkDecor::None).is_double_decorated(),
            false
        );
    }

    #[test]
    fn only_decor2_is_not_double_decorated() {
        assert_eq!(
            LinkType::new(LinkDecor::None, LinkDecor::Arrow).is_double_decorated(),
            false
        );
    }

    #[test]
    fn both_present_is_double_decorated() {
        assert!(LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).is_double_decorated());
    }

    // ── looks_like_reverted_for_svg ────────────────────────────────────────

    #[test]
    fn only_decor2_looks_reverted() {
        assert!(LinkType::new(LinkDecor::None, LinkDecor::Arrow).looks_like_reverted_for_svg());
    }

    #[test]
    fn only_decor1_does_not_look_reverted() {
        assert_eq!(
            LinkType::new(LinkDecor::Arrow, LinkDecor::None).looks_like_reverted_for_svg(),
            false
        );
    }

    #[test]
    fn both_present_does_not_look_reverted() {
        assert_eq!(
            LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).looks_like_reverted_for_svg(),
            false
        );
    }

    #[test]
    fn both_none_does_not_look_reverted() {
        assert_eq!(
            LinkType::new(LinkDecor::None, LinkDecor::None).looks_like_reverted_for_svg(),
            false
        );
    }

    // ── looks_like_no_decor_at_all_svg ────────────────────────────────────

    #[test]
    fn both_none_looks_like_no_decor() {
        assert!(
            LinkType::new(LinkDecor::None, LinkDecor::None).looks_like_no_decor_at_all_svg()
        );
    }

    #[test]
    fn both_present_looks_like_no_decor() {
        assert!(
            LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).looks_like_no_decor_at_all_svg()
        );
    }

    #[test]
    fn only_one_side_does_not_look_like_no_decor() {
        assert_eq!(
            LinkType::new(LinkDecor::Arrow, LinkDecor::None).looks_like_no_decor_at_all_svg(),
            false
        );
        assert_eq!(
            LinkType::new(LinkDecor::None, LinkDecor::Arrow).looks_like_no_decor_at_all_svg(),
            false
        );
    }

    // ── is_invisible / is_extends ─────────────────────────────────────────

    #[test]
    fn normal_style_is_not_invisible() {
        assert_eq!(LinkType::new(LinkDecor::Arrow, LinkDecor::None).is_invisible(), false);
    }

    #[test]
    fn invisible_style_is_invisible() {
        assert!(LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_invisible().is_invisible());
    }

    #[test]
    fn extends_on_decor1_is_extends() {
        assert!(LinkType::new(LinkDecor::Extends, LinkDecor::None).is_extends());
    }

    #[test]
    fn extends_on_decor2_is_extends() {
        assert!(LinkType::new(LinkDecor::None, LinkDecor::Extends).is_extends());
    }

    #[test]
    fn arrow_on_both_is_not_extends() {
        assert_eq!(LinkType::new(LinkDecor::Arrow, LinkDecor::Arrow).is_extends(), false);
    }

    // ── Style derivation ───────────────────────────────────────────────────

    #[test]
    fn go_dashed_produces_dashed_style() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_dashed();
        assert_eq!(lt.style().kind(), LinkStyleKind::Dashed);
    }

    #[test]
    fn go_dotted_produces_dotted_style() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_dotted();
        assert_eq!(lt.style().kind(), LinkStyleKind::Dotted);
    }

    #[test]
    fn go_bold_produces_bold_style() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_bold();
        assert_eq!(lt.style().kind(), LinkStyleKind::Bold);
    }

    #[test]
    fn go_invisible_produces_invisible_style() {
        assert!(LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_invisible().is_invisible());
    }

    #[test]
    fn go_thickness_marks_style_as_overridden() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_thickness(3.0);
        assert!(lt.style().is_thickness_overridden());
        assert_eq!(lt.style().get_stroke3().thickness, 3.0);
    }

    #[test]
    fn style_derivation_preserves_decors() {
        let lt = LinkType::new(LinkDecor::Extends, LinkDecor::Composition).go_dashed();
        assert_eq!(lt.decor1(), LinkDecor::Extends);
        assert_eq!(lt.decor2(), LinkDecor::Composition);
    }

    // ── inversed ──────────────────────────────────────────────────────────

    #[test]
    fn inversed_swaps_decors() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).inversed();
        assert_eq!(lt.decor1(), LinkDecor::Extends);
        assert_eq!(lt.decor2(), LinkDecor::Arrow);
    }

    #[test]
    fn inversed_preserves_style() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None)
            .go_dashed()
            .inversed();
        assert_eq!(lt.style().kind(), LinkStyleKind::Dashed);
    }

    #[test]
    fn inversed_inverts_middle_circled1_to_circled2() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None)
            .with_middle_circle_circled1()
            .inversed();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::CircleCircled2);
    }

    #[test]
    fn inversed_inverts_middle_circled2_to_circled1() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None)
            .with_middle_circle_circled2()
            .inversed();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::CircleCircled1);
    }

    #[test]
    fn inversed_preserves_neutral_middle_decors() {
        let lt_circle = LinkType::new(LinkDecor::Arrow, LinkDecor::None)
            .with_middle_circle()
            .inversed();
        assert_eq!(lt_circle.middle_decor(), LinkMiddleDecor::Circle);

        let lt_subset = LinkType::new(LinkDecor::Arrow, LinkDecor::None)
            .with_middle_subset()
            .inversed();
        assert_eq!(lt_subset.middle_decor(), LinkMiddleDecor::Subset);
    }

    // ── without_decors / parts ────────────────────────────────────────────

    #[test]
    fn without_decors1_sets_decor1_to_none() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).without_decors1();
        assert_eq!(lt.decor1(), LinkDecor::None);
        assert_eq!(lt.decor2(), LinkDecor::Extends);
    }

    #[test]
    fn without_decors2_sets_decor2_to_none() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).without_decors2();
        assert_eq!(lt.decor1(), LinkDecor::Arrow);
        assert_eq!(lt.decor2(), LinkDecor::None);
    }

    #[test]
    fn part1_keeps_decor1_clears_decor2() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).part1();
        assert_eq!(lt.decor1(), LinkDecor::Arrow);
        assert_eq!(lt.decor2(), LinkDecor::None);
    }

    #[test]
    fn part2_clears_decor1_keeps_decor2() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).part2();
        assert_eq!(lt.decor1(), LinkDecor::None);
        assert_eq!(lt.decor2(), LinkDecor::Extends);
    }

    // ── middle_decor derivation ────────────────────────────────────────────

    #[test]
    fn with_middle_circle_sets_circle() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).with_middle_circle();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::Circle);
    }

    #[test]
    fn with_middle_circle_circled_sets_circled() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).with_middle_circle_circled();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::CircleCircled);
    }

    #[test]
    fn with_middle_circle_circled1_sets_circled1() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).with_middle_circle_circled1();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::CircleCircled1);
    }

    #[test]
    fn with_middle_circle_circled2_sets_circled2() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).with_middle_circle_circled2();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::CircleCircled2);
    }

    #[test]
    fn with_middle_subset_sets_subset() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).with_middle_subset();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::Subset);
    }

    #[test]
    fn with_middle_superset_sets_superset() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).with_middle_superset();
        assert_eq!(lt.middle_decor(), LinkMiddleDecor::Superset);
    }

    // ── lollipop helpers ──────────────────────────────────────────────────

    #[test]
    fn lollipop_eye1_keeps_decor1_clears_decor2() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).with_lollipop_interface_eye1();
        assert_eq!(lt.decor1(), LinkDecor::Arrow);
        assert_eq!(lt.decor2(), LinkDecor::None);
    }

    #[test]
    fn lollipop_eye2_clears_decor1_keeps_decor2() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends).with_lollipop_interface_eye2();
        assert_eq!(lt.decor1(), LinkDecor::None);
        assert_eq!(lt.decor2(), LinkDecor::Extends);
    }

    // ── get_stroke3 ───────────────────────────────────────────────────────

    #[test]
    fn get_stroke3_no_default_returns_style_stroke() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        let s = lt.get_stroke3(None);
        assert_eq!(s.thickness, 1.0);
        assert!(s.dasharray_svg().is_none());
    }

    #[test]
    fn get_stroke3_with_thickness_default_applies_default_thickness() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        let default_t = UStroke::with_thickness(2.5);
        let s = lt.get_stroke3(Some(&default_t));
        assert_eq!(s.thickness, 2.5);
    }

    #[test]
    fn get_stroke3_dash_default_passes_through_unchanged() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None);
        let default_t = UStroke::new(5.0, 5.0, 2.0);
        let s = lt.get_stroke3(Some(&default_t));
        assert_eq!(s.dasharray_svg(), Some((5.0, 5.0)));
        assert_eq!(s.thickness, 2.0);
    }

    #[test]
    fn get_stroke3_explicit_override_ignores_default() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::None).go_thickness(4.0);
        let default_t = UStroke::with_thickness(2.0);
        let s = lt.get_stroke3(Some(&default_t));
        assert_eq!(s.thickness, 4.0);
    }

    // ── specific_decoration_svek ───────────────────────────────────────────

    #[test]
    fn svek_simplier_strategy_suppresses_all_arrows() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let s = lt.specific_decoration_svek(LinkStrategy::Simplier);
        assert_eq!(s, "arrowtail=none,arrowhead=none");
    }

    #[test]
    fn svek_both_none_produces_no_arrows() {
        let lt = LinkType::new(LinkDecor::None, LinkDecor::None);
        let s = lt.specific_decoration_svek(LinkStrategy::Normal);
        assert!(s.contains("arrowtail=none"), "got: {}", s);
        assert!(s.contains("arrowhead=none"), "got: {}", s);
    }

    #[test]
    fn svek_both_present_produces_dir_both() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let s = lt.specific_decoration_svek(LinkStrategy::Normal);
        assert!(s.contains("dir=both"), "got: {}", s);
        assert!(s.contains("arrowtail=empty"), "got: {}", s);
        assert!(s.contains("arrowhead=empty"), "got: {}", s);
    }

    #[test]
    fn svek_only_decor2_produces_dir_back() {
        let lt = LinkType::new(LinkDecor::None, LinkDecor::Arrow);
        let s = lt.specific_decoration_svek(LinkStrategy::Normal);
        assert!(s.contains("arrowtail=empty"), "got: {}", s);
        assert!(s.contains("arrowhead=none"), "got: {}", s);
        assert!(s.contains("dir=back"), "got: {}", s);
    }

    #[test]
    fn svek_arrow_size_included_when_nonzero() {
        let lt = LinkType::new(LinkDecor::Arrow, LinkDecor::Extends);
        let s = lt.specific_decoration_svek(LinkStrategy::Normal);
        assert!(s.contains("arrowsize="), "got: {}", s);
    }

    // ── link_type_name ─────────────────────────────────────────────────────

    #[test]
    fn link_type_name_composition() {
        assert_eq!(
            LinkType::new(LinkDecor::Composition, LinkDecor::None).link_type_name(),
            Some("composition")
        );
    }

    #[test]
    fn link_type_name_aggregation() {
        assert_eq!(
            LinkType::new(LinkDecor::None, LinkDecor::Agregation).link_type_name(),
            Some("aggregation")
        );
    }

    #[test]
    fn link_type_name_extension() {
        assert_eq!(
            LinkType::new(LinkDecor::Extends, LinkDecor::None).link_type_name(),
            Some("extension")
        );
    }

    #[test]
    fn link_type_name_redefines() {
        assert_eq!(
            LinkType::new(LinkDecor::Redefines, LinkDecor::None).link_type_name(),
            Some("redefines")
        );
    }

    #[test]
    fn link_type_name_defined_by() {
        assert_eq!(
            LinkType::new(LinkDecor::DefinedBy, LinkDecor::None).link_type_name(),
            Some("definedby")
        );
    }

    #[test]
    fn link_type_name_dependency_from_arrow() {
        assert_eq!(
            LinkType::new(LinkDecor::Arrow, LinkDecor::None).link_type_name(),
            Some("dependency")
        );
    }

    #[test]
    fn link_type_name_dependency_from_arrow_triangle() {
        assert_eq!(
            LinkType::new(LinkDecor::None, LinkDecor::ArrowTriangle).link_type_name(),
            Some("dependency")
        );
    }

    #[test]
    fn link_type_name_not_navigable() {
        assert_eq!(
            LinkType::new(LinkDecor::NotNavigable, LinkDecor::None).link_type_name(),
            Some("not_navigable")
        );
    }

    #[test]
    fn link_type_name_crowfoot_variants() {
        assert_eq!(
            LinkType::new(LinkDecor::Crowfoot, LinkDecor::None).link_type_name(),
            Some("crowfoot")
        );
        assert_eq!(
            LinkType::new(LinkDecor::CircleCrowfoot, LinkDecor::None).link_type_name(),
            Some("crowfoot")
        );
        assert_eq!(
            LinkType::new(LinkDecor::LineCrowfoot, LinkDecor::None).link_type_name(),
            Some("crowfoot")
        );
    }

    #[test]
    fn link_type_name_association_from_both_none() {
        assert_eq!(
            LinkType::new(LinkDecor::None, LinkDecor::None).link_type_name(),
            Some("association")
        );
    }

    #[test]
    fn link_type_name_association_from_circle_line() {
        assert_eq!(
            LinkType::new(LinkDecor::CircleLine, LinkDecor::None).link_type_name(),
            Some("association")
        );
    }

    #[test]
    fn link_type_name_association_from_double_line() {
        assert_eq!(
            LinkType::new(LinkDecor::DoubleLine, LinkDecor::None).link_type_name(),
            Some("association")
        );
    }

    #[test]
    fn link_type_name_nested_from_plus() {
        assert_eq!(
            LinkType::new(LinkDecor::Plus, LinkDecor::None).link_type_name(),
            Some("nested")
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// USymbolSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod usymbol_tests {
    use plantuml_little::decoration::symbol::{SymbolMargin, USymbolKind};
    use plantuml_little::klimt::geom::XDimension2D;

    // ── Variant count ──────────────────────────────────────────────────────

    #[test]
    fn all_35_variants_distinct() {
        // Java: assertEquals(35, USymbols.all().size())
        let variants = [
            USymbolKind::Action,
            USymbolKind::ActorStickman,
            USymbolKind::ActorAwesome,
            USymbolKind::ActorHollow,
            USymbolKind::ActorBusiness,
            USymbolKind::Agent,
            USymbolKind::Archimate,
            USymbolKind::Artifact,
            USymbolKind::Boundary,
            USymbolKind::Card,
            USymbolKind::Cloud,
            USymbolKind::Collections,
            USymbolKind::Component1,
            USymbolKind::Component2,
            USymbolKind::ComponentRectangle,
            USymbolKind::Control,
            USymbolKind::Database,
            USymbolKind::EntityDomain,
            USymbolKind::File,
            USymbolKind::Folder,
            USymbolKind::Frame,
            USymbolKind::Group,
            USymbolKind::Hexagon,
            USymbolKind::Interface,
            USymbolKind::Label,
            USymbolKind::Node,
            USymbolKind::Package,
            USymbolKind::Person,
            USymbolKind::Process,
            USymbolKind::Queue,
            USymbolKind::Rectangle,
            USymbolKind::SimpleAbstract,
            USymbolKind::Stack,
            USymbolKind::Storage,
            USymbolKind::Usecase,
        ];
        assert_eq!(variants.len(), 35);
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    // ── from_name ──────────────────────────────────────────────────────────

    #[test]
    fn from_name_database_case_insensitive() {
        assert_eq!(USymbolKind::from_name("database"), Some(USymbolKind::Database));
        assert_eq!(USymbolKind::from_name("DATABASE"), Some(USymbolKind::Database));
        assert_eq!(USymbolKind::from_name("Database"), Some(USymbolKind::Database));
    }

    #[test]
    fn from_name_cloud() {
        assert_eq!(USymbolKind::from_name("cloud"), Some(USymbolKind::Cloud));
    }

    #[test]
    fn from_name_component_alias_maps_to_component2() {
        assert_eq!(USymbolKind::from_name("component"), Some(USymbolKind::Component2));
        assert_eq!(USymbolKind::from_name("component2"), Some(USymbolKind::Component2));
    }

    #[test]
    fn from_name_component1() {
        assert_eq!(USymbolKind::from_name("component1"), Some(USymbolKind::Component1));
    }

    #[test]
    fn from_name_component_rectangle() {
        assert_eq!(
            USymbolKind::from_name("component_rectangle"),
            Some(USymbolKind::ComponentRectangle)
        );
    }

    #[test]
    fn from_name_actor_alias_maps_to_stickman() {
        assert_eq!(USymbolKind::from_name("actor"), Some(USymbolKind::ActorStickman));
        assert_eq!(
            USymbolKind::from_name("actor_stickman"),
            Some(USymbolKind::ActorStickman)
        );
    }

    #[test]
    fn from_name_actor_awesome() {
        assert_eq!(USymbolKind::from_name("actor_awesome"), Some(USymbolKind::ActorAwesome));
    }

    #[test]
    fn from_name_actor_hollow() {
        assert_eq!(USymbolKind::from_name("actor_hollow"), Some(USymbolKind::ActorHollow));
    }

    #[test]
    fn from_name_actor_business() {
        assert_eq!(
            USymbolKind::from_name("actor_stickman_business"),
            Some(USymbolKind::ActorBusiness)
        );
    }

    #[test]
    fn from_name_entity_aliases() {
        assert_eq!(USymbolKind::from_name("entity"), Some(USymbolKind::EntityDomain));
        assert_eq!(USymbolKind::from_name("entity_domain"), Some(USymbolKind::EntityDomain));
    }

    #[test]
    fn from_name_rectangle_and_rect_alias() {
        assert_eq!(USymbolKind::from_name("rectangle"), Some(USymbolKind::Rectangle));
        assert_eq!(USymbolKind::from_name("rect"), Some(USymbolKind::Rectangle));
    }

    #[test]
    fn from_name_remaining_symbols() {
        let cases: &[(&str, USymbolKind)] = &[
            ("action", USymbolKind::Action),
            ("agent", USymbolKind::Agent),
            ("archimate", USymbolKind::Archimate),
            ("artifact", USymbolKind::Artifact),
            ("boundary", USymbolKind::Boundary),
            ("card", USymbolKind::Card),
            ("collections", USymbolKind::Collections),
            ("control", USymbolKind::Control),
            ("file", USymbolKind::File),
            ("folder", USymbolKind::Folder),
            ("frame", USymbolKind::Frame),
            ("group", USymbolKind::Group),
            ("hexagon", USymbolKind::Hexagon),
            ("interface", USymbolKind::Interface),
            ("label", USymbolKind::Label),
            ("node", USymbolKind::Node),
            ("package", USymbolKind::Package),
            ("person", USymbolKind::Person),
            ("process", USymbolKind::Process),
            ("queue", USymbolKind::Queue),
            ("stack", USymbolKind::Stack),
            ("storage", USymbolKind::Storage),
            ("usecase", USymbolKind::Usecase),
        ];
        for (name, expected) in cases {
            assert_eq!(
                USymbolKind::from_name(name),
                Some(*expected),
                "from_name('{}') should be {:?}",
                name,
                expected
            );
        }
    }

    #[test]
    fn from_name_unknown_returns_none() {
        assert!(USymbolKind::from_name("nonexistent").is_none());
        assert!(USymbolKind::from_name("").is_none());
        assert!(USymbolKind::from_name("foobar").is_none());
    }

    // ── margin values ──────────────────────────────────────────────────────

    #[test]
    fn margin_action_is_10_20_10_10() {
        let m = USymbolKind::Action.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 20.0, 10.0, 10.0));
    }

    #[test]
    fn margin_database_is_10_10_24_5() {
        let m = USymbolKind::Database.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 10.0, 24.0, 5.0));
    }

    #[test]
    fn margin_cloud_is_15_15_15_15() {
        let m = USymbolKind::Cloud.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (15.0, 15.0, 15.0, 15.0));
    }

    #[test]
    fn margin_component1_is_10_10_10_10() {
        let m = USymbolKind::Component1.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 10.0, 10.0, 10.0));
    }

    #[test]
    fn margin_component2_is_15_25_20_10() {
        // Java: Margin(10+5, 20+5, 15+5, 5+5)
        let m = USymbolKind::Component2.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (15.0, 25.0, 20.0, 10.0));
    }

    #[test]
    fn margin_component_rectangle_is_10_10_10_10() {
        let m = USymbolKind::ComponentRectangle.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 10.0, 10.0, 10.0));
    }

    #[test]
    fn margin_folder_is_10_20_13_10() {
        let m = USymbolKind::Folder.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 20.0, 13.0, 10.0));
    }

    #[test]
    fn margin_frame_is_15_25_20_10() {
        let m = USymbolKind::Frame.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (15.0, 25.0, 20.0, 10.0));
    }

    #[test]
    fn margin_group_same_as_frame() {
        assert_eq!(USymbolKind::Group.margin().x1, USymbolKind::Frame.margin().x1);
        assert_eq!(USymbolKind::Group.margin().y1, USymbolKind::Frame.margin().y1);
    }

    #[test]
    fn margin_node_is_15_25_20_10() {
        let m = USymbolKind::Node.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (15.0, 25.0, 20.0, 10.0));
    }

    #[test]
    fn margin_card_is_10_10_3_3() {
        let m = USymbolKind::Card.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 10.0, 3.0, 3.0));
    }

    #[test]
    fn margin_queue_is_5_15_5_5() {
        let m = USymbolKind::Queue.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (5.0, 15.0, 5.0, 5.0));
    }

    #[test]
    fn margin_stack_is_25_25_10_10() {
        let m = USymbolKind::Stack.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (25.0, 25.0, 10.0, 10.0));
    }

    #[test]
    fn margin_storage_is_10_10_10_10() {
        let m = USymbolKind::Storage.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 10.0, 10.0, 10.0));
    }

    #[test]
    fn margin_artifact_is_10_20_13_10() {
        // Java: Margin(10, 10+10, 10+3, 10)
        let m = USymbolKind::Artifact.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 20.0, 13.0, 10.0));
    }

    #[test]
    fn margin_package_same_as_folder() {
        let pf = USymbolKind::Package.margin();
        let ff = USymbolKind::Folder.margin();
        assert_eq!((pf.x1, pf.x2, pf.y1, pf.y2), (ff.x1, ff.x2, ff.y1, ff.y2));
    }

    #[test]
    fn margin_rectangle_is_10_10_10_10() {
        let m = USymbolKind::Rectangle.margin();
        assert_eq!((m.x1, m.x2, m.y1, m.y2), (10.0, 10.0, 10.0, 10.0));
    }

    #[test]
    fn margin_agent_archimate_same_as_rectangle() {
        for kind in [USymbolKind::Agent, USymbolKind::Archimate] {
            let m = kind.margin();
            assert_eq!(m.x1, 10.0, "{:?}.margin().x1", kind);
            assert_eq!(m.x2, 10.0, "{:?}.margin().x2", kind);
            assert_eq!(m.y1, 10.0, "{:?}.margin().y1", kind);
            assert_eq!(m.y2, 10.0, "{:?}.margin().y2", kind);
        }
    }

    #[test]
    fn margin_process_is_20_20_10_10() {
        let m = USymbolKind::Process.margin();
        assert_eq!(m.x1, 20.0);
        assert_eq!(m.x2, 20.0);
        assert_eq!(m.y1, 10.0);
        assert_eq!(m.y2, 10.0);
    }

    // ── supp_height / supp_width ───────────────────────────────────────────

    #[test]
    fn supp_height_database_is_15() {
        assert_eq!(USymbolKind::Database.supp_height(), 15);
    }

    #[test]
    fn supp_height_node_is_5() {
        assert_eq!(USymbolKind::Node.supp_height(), 5);
    }

    #[test]
    fn supp_height_default_is_zero() {
        for kind in [
            USymbolKind::Rectangle,
            USymbolKind::Cloud,
            USymbolKind::Frame,
            USymbolKind::Folder,
            USymbolKind::Storage,
        ] {
            assert_eq!(kind.supp_height(), 0, "{:?}.supp_height() should be 0", kind);
        }
    }

    #[test]
    fn supp_width_node_is_60() {
        assert_eq!(USymbolKind::Node.supp_width(), 60);
    }

    #[test]
    fn supp_width_database_is_zero() {
        assert_eq!(USymbolKind::Database.supp_width(), 0);
    }

    #[test]
    fn supp_width_default_is_zero() {
        for kind in [USymbolKind::Rectangle, USymbolKind::Cloud, USymbolKind::Folder] {
            assert_eq!(kind.supp_width(), 0, "{:?}.supp_width() should be 0", kind);
        }
    }

    // ── SymbolMargin arithmetic ────────────────────────────────────────────

    #[test]
    fn margin_width_is_x1_plus_x2() {
        let m = SymbolMargin::new(10.0, 20.0, 5.0, 15.0);
        assert_eq!(m.width(), 30.0);
    }

    #[test]
    fn margin_height_is_y1_plus_y2() {
        let m = SymbolMargin::new(10.0, 20.0, 5.0, 15.0);
        assert_eq!(m.height(), 20.0);
    }

    #[test]
    fn margin_add_dimension_expands_by_total_margin() {
        let m = SymbolMargin::new(10.0, 20.0, 5.0, 15.0);
        let dim = XDimension2D::new(100.0, 50.0);
        let expanded = m.add_dimension(dim);
        // width += x1 + x2 = 10 + 20 = 30 => 130
        assert_eq!(expanded.width, 130.0);
        // height += y1 + y2 = 5 + 15 = 20 => 70
        assert_eq!(expanded.height, 70.0);
    }

    #[test]
    fn margin_add_dimension_zero_margin_is_identity() {
        let m = SymbolMargin::new(0.0, 0.0, 0.0, 0.0);
        let dim = XDimension2D::new(100.0, 50.0);
        let expanded = m.add_dimension(dim);
        assert_eq!(expanded.width, 100.0);
        assert_eq!(expanded.height, 50.0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HtmlColorAndStyleSkeletonTest.java  (not yet ported)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod html_color_and_style_tests {
    // Java: HtmlColorAndStyle bundles an HColor with a LinkStyle.
    // It provides: getColor(), getStyle(), and factory methods that
    // accept color strings and style modifiers.
    // Ported tests would verify: round-trip color+style storage,
    // default style is normal, null-color handling.

    #[test]
    #[ignore = "gap: HtmlColorAndStyle not yet ported — Java: net.sourceforge.plantuml.decoration.HtmlColorAndStyle"]
    fn color_and_style_round_trip() {
        todo!("HtmlColorAndStyle not yet ported to Rust")
    }

    #[test]
    #[ignore = "gap: HtmlColorAndStyle not yet ported — Java: net.sourceforge.plantuml.decoration.HtmlColorAndStyle"]
    fn default_style_is_normal() {
        todo!("HtmlColorAndStyle not yet ported to Rust")
    }

    #[test]
    #[ignore = "gap: HtmlColorAndStyle not yet ported — Java: net.sourceforge.plantuml.decoration.HtmlColorAndStyle"]
    fn null_color_handling() {
        todo!("HtmlColorAndStyle not yet ported to Rust")
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RainbowSkeletonTest.java  (not yet ported)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod rainbow_tests {
    // Java: Rainbow holds a list of HtmlColorAndStyle entries, enabling
    // multi-color gradient arrows. Methods: getFirst(), size(), is_singleton(),
    // build(HtmlColorAndStyle), build(List<HtmlColorAndStyle>).
    // Ported tests would verify: singleton construction, list construction,
    // first-element access, size count.

    #[test]
    #[ignore = "gap: Rainbow not yet ported — Java: net.sourceforge.plantuml.decoration.Rainbow"]
    fn singleton_has_size_1() {
        todo!("Rainbow not yet ported to Rust")
    }

    #[test]
    #[ignore = "gap: Rainbow not yet ported — Java: net.sourceforge.plantuml.decoration.Rainbow"]
    fn singleton_get_first_returns_the_entry() {
        todo!("Rainbow not yet ported to Rust")
    }

    #[test]
    #[ignore = "gap: Rainbow not yet ported — Java: net.sourceforge.plantuml.decoration.Rainbow"]
    fn multi_entry_size_matches_input_list() {
        todo!("Rainbow not yet ported to Rust")
    }

    #[test]
    #[ignore = "gap: Rainbow not yet ported — Java: net.sourceforge.plantuml.decoration.Rainbow"]
    fn is_singleton_true_for_one_entry() {
        todo!("Rainbow not yet ported to Rust")
    }

    #[test]
    #[ignore = "gap: Rainbow not yet ported — Java: net.sourceforge.plantuml.decoration.Rainbow"]
    fn is_singleton_false_for_multiple_entries() {
        todo!("Rainbow not yet ported to Rust")
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WithLinkTypeSkeletonTest.java  (not yet ported — abstract base)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod with_link_type_tests {
    // Java: WithLinkType is an abstract base class exposing getLinkType(),
    // getType(), getArrow(), and style-delegation helpers. In Rust this
    // would become a trait. Tests would verify that concrete implementors
    // correctly delegate to the underlying LinkType.

    #[test]
    #[ignore = "gap: WithLinkType not yet ported — Java: net.sourceforge.plantuml.decoration.WithLinkType (abstract)"]
    fn get_link_type_returns_stored_link_type() {
        todo!("WithLinkType not yet ported to Rust — needs trait + concrete impl")
    }

    #[test]
    #[ignore = "gap: WithLinkType not yet ported — Java: net.sourceforge.plantuml.decoration.WithLinkType (abstract)"]
    fn get_type_delegates_to_link_type() {
        todo!("WithLinkType not yet ported to Rust — needs trait + concrete impl")
    }

    #[test]
    #[ignore = "gap: WithLinkType not yet ported — Java: net.sourceforge.plantuml.decoration.WithLinkType (abstract)"]
    fn get_arrow_delegates_to_link_type() {
        todo!("WithLinkType not yet ported to Rust — needs trait + concrete impl")
    }

    #[test]
    #[ignore = "gap: WithLinkType not yet ported — Java: net.sourceforge.plantuml.decoration.WithLinkType (abstract)"]
    fn style_helpers_delegate_correctly() {
        todo!("WithLinkType not yet ported to Rust — needs trait + concrete impl")
    }
}
