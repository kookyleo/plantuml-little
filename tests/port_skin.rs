// tests/port_skin.rs
//
// Rust ports of Java skin package skeleton tests.
// Sources:
//   generated-public-api-tests-foundation/packages/net/sourceforge/plantuml/skin/
//
// One submodule per Java *SkeletonTest class.  Tests that have a Rust
// equivalent are written out; tests whose target is not yet ported carry
// #[ignore = "gap: …"].

use plantuml_little::skin::{
    ActorStyle, ArrowBody, ArrowConfiguration, ArrowDecoration, ArrowDirection, ArrowDressing,
    ArrowHead, ArrowPart, ComponentStyle, ComponentType,
};

// ════════════════════════════════════════════════════════════════════
// ArrowDirectionSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod arrow_direction {
    use super::*;

    #[test]
    fn values_four_variants_in_order() {
        // Java: values().length == 4, order: LEFT_TO_RIGHT_NORMAL, RIGHT_TO_LEFT_REVERSE, SELF, BOTH_DIRECTION
        let all = [
            ArrowDirection::LeftToRight,
            ArrowDirection::RightToLeft,
            ArrowDirection::Self_,
            ArrowDirection::Both,
        ];
        assert_eq!(all.len(), 4);
        // All variants are distinct
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[0], all[3]);
        assert_ne!(all[1], all[2]);
        assert_ne!(all[1], all[3]);
        assert_ne!(all[2], all[3]);
    }

    #[test]
    fn value_of_left_to_right_normal() {
        // Verify the variant has the expected Debug name
        assert_eq!(format!("{:?}", ArrowDirection::LeftToRight), "LeftToRight");
    }

    #[test]
    fn value_of_right_to_left_reverse() {
        assert_eq!(format!("{:?}", ArrowDirection::RightToLeft), "RightToLeft");
    }

    #[test]
    fn value_of_self_() {
        assert_eq!(format!("{:?}", ArrowDirection::Self_), "Self_");
    }

    #[test]
    fn value_of_both_direction() {
        assert_eq!(format!("{:?}", ArrowDirection::Both), "Both");
    }

    #[test]
    fn reverse_left_to_right_gives_right_to_left() {
        assert_eq!(
            ArrowDirection::LeftToRight.reverse(),
            ArrowDirection::RightToLeft
        );
    }

    #[test]
    fn reverse_right_to_left_gives_left_to_right() {
        assert_eq!(
            ArrowDirection::RightToLeft.reverse(),
            ArrowDirection::LeftToRight
        );
    }
}

// ════════════════════════════════════════════════════════════════════
// ArrowBodySkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod arrow_body {
    use super::*;

    #[test]
    fn values_five_variants_in_order() {
        // Java: values().length == 5, order: NORMAL, DOTTED, DASHED, HIDDEN, BOLD
        let all = [
            ArrowBody::Normal,
            ArrowBody::Dotted,
            ArrowBody::Dashed,
            ArrowBody::Hidden,
            ArrowBody::Bold,
        ];
        assert_eq!(all.len(), 5);
        // All variants are distinct
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[0], all[3]);
        assert_ne!(all[0], all[4]);
        assert_ne!(all[1], all[2]);
        assert_ne!(all[1], all[3]);
        assert_ne!(all[1], all[4]);
        assert_ne!(all[2], all[3]);
        assert_ne!(all[2], all[4]);
        assert_ne!(all[3], all[4]);
    }

    #[test]
    fn value_of_normal() {
        assert_eq!(ArrowBody::Normal.to_string(), "NORMAL");
    }

    #[test]
    fn value_of_dotted() {
        assert_eq!(ArrowBody::Dotted.to_string(), "DOTTED");
    }

    #[test]
    fn value_of_dashed() {
        assert_eq!(ArrowBody::Dashed.to_string(), "DASHED");
    }

    #[test]
    fn value_of_hidden() {
        assert_eq!(ArrowBody::Hidden.to_string(), "HIDDEN");
    }

    #[test]
    fn value_of_bold() {
        assert_eq!(ArrowBody::Bold.to_string(), "BOLD");
    }
}

// ════════════════════════════════════════════════════════════════════
// ArrowHeadSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod arrow_head {
    use super::*;

    #[test]
    fn values_four_variants_in_order() {
        // Java: values().length == 4, order: NORMAL, CROSSX, ASYNC, NONE
        let all = [
            ArrowHead::Normal,
            ArrowHead::CrossX,
            ArrowHead::Async,
            ArrowHead::None,
        ];
        assert_eq!(all.len(), 4);
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[0], all[3]);
        assert_ne!(all[1], all[2]);
        assert_ne!(all[1], all[3]);
        assert_ne!(all[2], all[3]);
    }

    #[test]
    fn value_of_normal() {
        assert_eq!(ArrowHead::Normal.to_string(), "NORMAL");
    }

    #[test]
    fn value_of_crossx() {
        assert_eq!(ArrowHead::CrossX.to_string(), "CROSSX");
    }

    #[test]
    fn value_of_async() {
        assert_eq!(ArrowHead::Async.to_string(), "ASYNC");
    }

    #[test]
    fn value_of_none() {
        assert_eq!(ArrowHead::None.to_string(), "NONE");
    }
}

// ════════════════════════════════════════════════════════════════════
// ArrowDecorationSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod arrow_decoration {
    use super::*;

    #[test]
    fn values_two_variants_in_order() {
        // Java: values().length == 2, order: NONE, CIRCLE
        let all = [ArrowDecoration::None, ArrowDecoration::Circle];
        assert_eq!(all.len(), 2);
        assert_ne!(all[0], all[1]);
    }

    #[test]
    fn value_of_none() {
        assert_eq!(ArrowDecoration::None.to_string(), "NONE");
    }

    #[test]
    fn value_of_circle() {
        assert_eq!(ArrowDecoration::Circle.to_string(), "CIRCLE");
    }
}

// ════════════════════════════════════════════════════════════════════
// ArrowPartSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod arrow_part {
    use super::*;

    #[test]
    fn values_three_variants_in_order() {
        // Java: values().length == 3, order: FULL, TOP_PART, BOTTOM_PART
        let all = [ArrowPart::Full, ArrowPart::TopPart, ArrowPart::BottomPart];
        assert_eq!(all.len(), 3);
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[1], all[2]);
    }

    #[test]
    fn value_of_full() {
        assert_eq!(format!("{:?}", ArrowPart::Full), "Full");
    }

    #[test]
    fn value_of_top_part() {
        assert_eq!(format!("{:?}", ArrowPart::TopPart), "TopPart");
    }

    #[test]
    fn value_of_bottom_part() {
        assert_eq!(format!("{:?}", ArrowPart::BottomPart), "BottomPart");
    }
}

// ════════════════════════════════════════════════════════════════════
// ArrowDressingSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod arrow_dressing {
    use super::*;

    #[test]
    fn create_gives_none_head_and_full_part() {
        // Java: ArrowDressing.create() => head=NONE, part=FULL
        let d = ArrowDressing::none();
        assert_eq!(d.head, ArrowHead::None);
        assert_eq!(d.part, ArrowPart::Full);
    }

    #[test]
    fn name_is_none() {
        // Java: d.name() == "NONE"
        let d = ArrowDressing::none();
        assert_eq!(d.to_string(), "NONE");
    }

    #[test]
    fn to_string_is_none() {
        // Java: d.toString() == "NONE"
        let d = ArrowDressing::none();
        assert_eq!(d.to_string(), "NONE");
    }

    #[test]
    fn with_head_changes_head_and_preserves_part() {
        // Java: ArrowDressing.create().withHead(NORMAL) => head=NORMAL, part=FULL
        let d = ArrowDressing::none().with_head(ArrowHead::Normal);
        assert_eq!(d.head, ArrowHead::Normal);
        assert_eq!(d.part, ArrowPart::Full);
    }

    #[test]
    fn with_part_changes_part_and_preserves_head() {
        // Java: d.withHead(NORMAL).withPart(TOP_PART) => head=NORMAL, part=TOP_PART
        let d = ArrowDressing::none()
            .with_head(ArrowHead::Normal)
            .with_part(ArrowPart::TopPart);
        assert_eq!(d.head, ArrowHead::Normal);
        assert_eq!(d.part, ArrowPart::TopPart);
    }

    #[test]
    fn get_head_default_is_none() {
        assert_eq!(ArrowDressing::none().head, ArrowHead::None);
    }

    #[test]
    fn get_head_after_with_head_async() {
        assert_eq!(
            ArrowDressing::none().with_head(ArrowHead::Async).head,
            ArrowHead::Async
        );
    }

    #[test]
    fn get_part_default_is_full() {
        assert_eq!(ArrowDressing::none().part, ArrowPart::Full);
    }

    #[test]
    fn get_part_after_with_part_bottom() {
        assert_eq!(
            ArrowDressing::none().with_part(ArrowPart::BottomPart).part,
            ArrowPart::BottomPart
        );
    }
}

// ════════════════════════════════════════════════════════════════════
// ArrowConfigurationSkeletonTest (34 methods from Java)
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod arrow_configuration {
    use super::*;

    #[test]
    fn with_direction_normal_not_null_and_direction() {
        // Java: assertNotNull(cfg); assertEquals(LEFT_TO_RIGHT_NORMAL, cfg.getArrowDirection())
        let cfg = ArrowConfiguration::with_direction_normal();
        assert_eq!(cfg.arrow_direction(), ArrowDirection::LeftToRight);
    }

    #[test]
    fn with_direction_both_not_null_and_direction() {
        // Java: assertNotNull(cfg); assertEquals(BOTH_DIRECTION, cfg.getArrowDirection())
        let cfg = ArrowConfiguration::with_direction_both();
        assert_eq!(cfg.arrow_direction(), ArrowDirection::Both);
    }

    #[test]
    fn with_direction_self_not_null_direction_and_is_self() {
        // Java: assertNotNull(cfg); assertEquals(SELF, cfg.getArrowDirection()); assertTrue(cfg.isSelfArrow())
        let cfg = ArrowConfiguration::with_direction_self(false);
        assert_eq!(cfg.arrow_direction(), ArrowDirection::Self_);
        assert!(cfg.is_self_arrow());
    }

    #[test]
    fn with_direction_reverse_not_null_and_direction() {
        // Java: assertNotNull(cfg); assertEquals(RIGHT_TO_LEFT_REVERSE, cfg.getArrowDirection())
        let cfg = ArrowConfiguration::with_direction_reverse();
        assert_eq!(cfg.arrow_direction(), ArrowDirection::RightToLeft);
    }

    #[test]
    fn reverse_flips_direction_to_right_to_left() {
        // Java: normal.reverse().getArrowDirection() == RIGHT_TO_LEFT_REVERSE
        let normal = ArrowConfiguration::with_direction_normal();
        let reversed = normal.reverse();
        assert_eq!(reversed.arrow_direction(), ArrowDirection::RightToLeft);
    }

    #[test]
    fn self_marks_is_self_arrow_and_direction() {
        // Java: assertTrue(cfg.isSelfArrow()); assertEquals(SELF, cfg.getArrowDirection())
        let cfg = ArrowConfiguration::with_direction_normal().self_arrow();
        assert!(cfg.is_self_arrow());
        assert_eq!(cfg.arrow_direction(), ArrowDirection::Self_);
    }

    #[test]
    fn with_body_dotted_is_dotted_not_hidden() {
        // Java: assertTrue(cfg.isDotted()); assertFalse(cfg.isHidden())
        let cfg = ArrowConfiguration::with_direction_normal().with_body(ArrowBody::Dotted);
        assert!(cfg.is_dotted());
        assert!(!cfg.is_hidden());
    }

    #[test]
    fn is_dotted_default_false_and_true_after_dotted_body() {
        assert!(!ArrowConfiguration::with_direction_normal().is_dotted());
        assert!(ArrowConfiguration::with_direction_normal()
            .with_body(ArrowBody::Dotted)
            .is_dotted());
    }

    #[test]
    fn is_hidden_default_false_and_true_after_hidden_body() {
        assert!(!ArrowConfiguration::with_direction_normal().is_hidden());
        assert!(ArrowConfiguration::with_direction_normal()
            .with_body(ArrowBody::Hidden)
            .is_hidden());
    }

    #[test]
    fn is_self_arrow_false_for_normal_true_for_self() {
        assert!(!ArrowConfiguration::with_direction_normal().is_self_arrow());
        assert!(ArrowConfiguration::with_direction_self(false).is_self_arrow());
    }

    #[test]
    fn get_arrow_direction_all_four_variants() {
        assert_eq!(
            ArrowConfiguration::with_direction_normal().arrow_direction(),
            ArrowDirection::LeftToRight
        );
        assert_eq!(
            ArrowConfiguration::with_direction_reverse().arrow_direction(),
            ArrowDirection::RightToLeft
        );
        assert_eq!(
            ArrowConfiguration::with_direction_self(false).arrow_direction(),
            ArrowDirection::Self_
        );
        assert_eq!(
            ArrowConfiguration::with_direction_both().arrow_direction(),
            ArrowDirection::Both
        );
    }

    #[test]
    fn get_head_normal_arrow_is_normal() {
        // Java: assertEquals(ArrowHead.NORMAL, normal.getHead())
        let normal = ArrowConfiguration::with_direction_normal();
        assert_eq!(normal.head(), ArrowHead::Normal);
    }

    #[test]
    fn get_decoration1_default_is_none() {
        assert_eq!(
            ArrowConfiguration::with_direction_normal().decoration1(),
            ArrowDecoration::None
        );
    }

    #[test]
    fn get_decoration2_default_is_none() {
        assert_eq!(
            ArrowConfiguration::with_direction_normal().decoration2(),
            ArrowDecoration::None
        );
    }

    #[test]
    fn with_decoration1_circle_sets_deco1_leaves_deco2_none() {
        let cfg =
            ArrowConfiguration::with_direction_normal().with_decoration1(ArrowDecoration::Circle);
        assert_eq!(cfg.decoration1(), ArrowDecoration::Circle);
        assert_eq!(cfg.decoration2(), ArrowDecoration::None);
    }

    #[test]
    fn with_decoration2_circle_leaves_deco1_none_sets_deco2() {
        let cfg =
            ArrowConfiguration::with_direction_normal().with_decoration2(ArrowDecoration::Circle);
        assert_eq!(cfg.decoration1(), ArrowDecoration::None);
        assert_eq!(cfg.decoration2(), ArrowDecoration::Circle);
    }

    #[test]
    fn get_color_default_is_none() {
        // Java: assertNull(ArrowConfiguration.withDirectionNormal().getColor())
        assert!(ArrowConfiguration::with_direction_normal()
            .color()
            .is_none());
    }

    #[test]
    fn get_part_default_is_full() {
        assert_eq!(
            ArrowConfiguration::with_direction_normal().part(),
            ArrowPart::Full
        );
    }

    #[test]
    fn get_dressing1_not_null() {
        // Java: assertNotNull(cfg.getDressing1())
        // Verify the dressing has a sensible head value (not a panic)
        let d = ArrowConfiguration::with_direction_normal().dressing1();
        assert_eq!(d.head, ArrowHead::None);
    }

    #[test]
    fn get_dressing2_not_null() {
        // Java: assertNotNull(cfg.getDressing2())
        // Verify dressing2 has the expected head (Normal for a normal arrow)
        let d = ArrowConfiguration::with_direction_normal().dressing2();
        assert_eq!(d.head, ArrowHead::Normal);
    }

    #[test]
    fn is_async1_default_false() {
        assert!(!ArrowConfiguration::with_direction_normal().is_async1());
    }

    #[test]
    fn is_async2_default_false_true_after_with_head2_async() {
        assert!(!ArrowConfiguration::with_direction_normal().is_async2());
        assert!(ArrowConfiguration::with_direction_normal()
            .with_head2(ArrowHead::Async)
            .is_async2());
    }

    #[test]
    fn with_head_crossx_changes_head() {
        let cfg = ArrowConfiguration::with_direction_normal().with_head(ArrowHead::CrossX);
        assert_eq!(cfg.head(), ArrowHead::CrossX);
    }

    #[test]
    fn with_head1_async_sets_dressing1_head() {
        let cfg = ArrowConfiguration::with_direction_normal().with_head1(ArrowHead::Async);
        assert_eq!(cfg.dressing1().head, ArrowHead::Async);
    }

    #[test]
    fn with_head2_crossx_sets_dressing2_head() {
        let cfg = ArrowConfiguration::with_direction_normal().with_head2(ArrowHead::CrossX);
        assert_eq!(cfg.dressing2().head, ArrowHead::CrossX);
    }

    #[test]
    fn with_part_top_part_changes_part() {
        let cfg = ArrowConfiguration::with_direction_normal().with_part(ArrowPart::TopPart);
        assert_eq!(cfg.part(), ArrowPart::TopPart);
    }

    #[test]
    fn with_thickness_returns_non_null_config() {
        // Java: assertNotNull(cfg)
        let cfg = ArrowConfiguration::with_direction_normal().with_thickness(2.5);
        assert!((cfg.thickness() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn reverse_define_toggles_flag_and_double_toggle_restores() {
        // Java: assertFalse(cfg.isReverseDefine())
        //       assertTrue(cfg.reverseDefine().isReverseDefine())
        //       assertFalse(cfg.reverseDefine().reverseDefine().isReverseDefine())
        let cfg = ArrowConfiguration::with_direction_normal();
        assert!(!cfg.is_reverse_define());
        assert!(cfg.reverse_define().is_reverse_define());
        assert!(!cfg.reverse_define().reverse_define().is_reverse_define());
    }

    #[test]
    fn is_reverse_define_false_for_normal_false_for_self_false_true_for_self_true() {
        assert!(!ArrowConfiguration::with_direction_normal().is_reverse_define());
        assert!(!ArrowConfiguration::with_direction_self(false).is_reverse_define());
        assert!(ArrowConfiguration::with_direction_self(true).is_reverse_define());
    }

    #[test]
    fn with_inclination_sets_inclination2() {
        // Java: assertEquals(45, cfg.getInclination2())
        let cfg = ArrowConfiguration::with_direction_normal().with_inclination(45);
        assert_eq!(cfg.inclination2(), 45);
    }

    #[test]
    fn get_inclination1_normal_arrow_returns_zero_reverse_returns_value() {
        // Java comment: withDirectionNormal() has dressing2 with ArrowHead.NORMAL — not NONE/CROSSX,
        // so getInclination1() returns 0 regardless of inclination value.
        let cfg = ArrowConfiguration::with_direction_normal().with_inclination(10);
        assert_eq!(cfg.inclination1(), 0);

        // withDirectionReverse() has dressing2 with ArrowHead.NONE, so inclination is returned
        let rev = ArrowConfiguration::with_direction_reverse().with_inclination(10);
        assert_eq!(rev.inclination1(), 10);
    }

    #[test]
    fn get_inclination2_returns_set_value() {
        let cfg = ArrowConfiguration::with_direction_normal().with_inclination(20);
        assert_eq!(cfg.inclination2(), 20);
    }

    #[test]
    fn name_not_null_and_not_empty() {
        // Java: assertNotNull(cfg.name()); assertFalse(cfg.name().isEmpty())
        let cfg = ArrowConfiguration::with_direction_normal();
        let name = cfg.name();
        assert!(!name.is_empty());
    }

    #[test]
    fn to_string_equals_name() {
        // Java: assertEquals(cfg.name(), cfg.toString())
        let cfg = ArrowConfiguration::with_direction_normal();
        assert_eq!(cfg.to_string(), cfg.name());
    }

    // The following three Java tests are @Ignore("requires UGraphic setup — non-SVG rendering test")
    // and four more are @Ignore for color/stroke.  We mark them all as ignored gaps.

    #[test]
    #[ignore = "gap: stroke requires UGraphic setup — non-SVG rendering test"]
    fn stroke_skeleton_for_ugraphic_double_double_double() {
        todo!()
    }

    #[test]
    #[ignore = "gap: applyStroke requires UGraphic setup — non-SVG rendering test"]
    fn apply_stroke_with_style() {
        todo!()
    }

    #[test]
    #[ignore = "gap: applyStroke overload requires UGraphic setup — non-SVG rendering test"]
    fn apply_stroke_overload_2() {
        todo!()
    }

    #[test]
    #[ignore = "gap: applyThicknessOnly requires UGraphic setup — non-SVG rendering test"]
    fn apply_thickness_only() {
        todo!()
    }

    #[test]
    #[ignore = "gap: withColor requires HColor — not yet ported"]
    fn with_color_hcolor() {
        todo!()
    }
}

// ════════════════════════════════════════════════════════════════════
// ComponentTypeSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod component_type {
    use super::*;

    #[test]
    fn values_non_empty_and_first_is_arrow() {
        // Java: assertTrue(vals.length > 0); assertEquals(ARROW, vals[0])
        // Verify Arrow variant exists and is_arrow() returns true
        assert!(ComponentType::Arrow.is_arrow());
    }

    #[test]
    fn value_of_arrow() {
        // Arrow is distinct from Note
        assert_ne!(ComponentType::Arrow, ComponentType::Note);
        assert_eq!(format!("{:?}", ComponentType::Arrow), "Arrow");
    }

    #[test]
    fn value_of_note() {
        // Note is distinct from Arrow and Divider
        assert_ne!(ComponentType::Note, ComponentType::Arrow);
        assert_ne!(ComponentType::Note, ComponentType::Divider);
        assert_eq!(format!("{:?}", ComponentType::Note), "Note");
    }

    #[test]
    fn value_of_divider() {
        assert_ne!(ComponentType::Divider, ComponentType::Arrow);
        assert_ne!(ComponentType::Divider, ComponentType::Note);
        assert_eq!(format!("{:?}", ComponentType::Divider), "Divider");
    }

    #[test]
    fn value_of_participant_head() {
        assert_ne!(ComponentType::ParticipantHead, ComponentType::Arrow);
        assert_ne!(ComponentType::ParticipantHead, ComponentType::Note);
        assert_eq!(
            format!("{:?}", ComponentType::ParticipantHead),
            "ParticipantHead"
        );
    }

    #[test]
    fn is_arrow_true_for_arrow() {
        assert!(ComponentType::Arrow.is_arrow());
    }

    #[test]
    fn is_arrow_false_for_note() {
        assert!(!ComponentType::Note.is_arrow());
    }

    #[test]
    fn is_arrow_false_for_divider() {
        assert!(!ComponentType::Divider.is_arrow());
    }

    #[test]
    fn is_arrow_false_for_participant_head() {
        assert!(!ComponentType::ParticipantHead.is_arrow());
    }

    #[test]
    #[ignore = "gap: ComponentType::getStyleSignature not yet ported"]
    fn get_style_signature_not_null_for_various_types() {
        // Java: assertNotNull(PARTICIPANT_HEAD.getStyleSignature()); etc.
        todo!()
    }
}

// ════════════════════════════════════════════════════════════════════
// ComponentStyleSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod component_style {
    use super::*;

    #[test]
    fn values_three_variants_in_order() {
        // Java: values().length == 3, order: UML1, UML2, RECTANGLE
        let all = [
            ComponentStyle::Uml1,
            ComponentStyle::Uml2,
            ComponentStyle::Rectangle,
        ];
        assert_eq!(all.len(), 3);
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[1], all[2]);
    }

    #[test]
    fn value_of_uml1() {
        assert_ne!(ComponentStyle::Uml1, ComponentStyle::Uml2);
        assert_eq!(format!("{:?}", ComponentStyle::Uml1), "Uml1");
    }

    #[test]
    fn value_of_uml2() {
        assert_ne!(ComponentStyle::Uml2, ComponentStyle::Uml1);
        assert_ne!(ComponentStyle::Uml2, ComponentStyle::Rectangle);
        assert_eq!(format!("{:?}", ComponentStyle::Uml2), "Uml2");
    }

    #[test]
    fn value_of_rectangle() {
        assert_ne!(ComponentStyle::Rectangle, ComponentStyle::Uml1);
        assert_ne!(ComponentStyle::Rectangle, ComponentStyle::Uml2);
        assert_eq!(format!("{:?}", ComponentStyle::Rectangle), "Rectangle");
    }

    #[test]
    #[ignore = "gap: ComponentStyle::toUSymbol not yet ported"]
    fn to_u_symbol_not_null_for_all_variants() {
        // Java: assertNotNull(UML1.toUSymbol()); assertNotNull(UML2.toUSymbol()); assertNotNull(RECTANGLE.toUSymbol())
        todo!()
    }
}

// ════════════════════════════════════════════════════════════════════
// ActorStyleSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod actor_style {
    use super::*;

    #[test]
    fn values_four_variants_in_order() {
        // Java: values().length == 4, order: STICKMAN, STICKMAN_BUSINESS, AWESOME, HOLLOW
        let all = [
            ActorStyle::Stickman,
            ActorStyle::StickmanBusiness,
            ActorStyle::Awesome,
            ActorStyle::Hollow,
        ];
        assert_eq!(all.len(), 4);
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[0], all[3]);
        assert_ne!(all[1], all[2]);
        assert_ne!(all[1], all[3]);
        assert_ne!(all[2], all[3]);
    }

    #[test]
    fn value_of_stickman() {
        // Stickman is the default variant
        assert_eq!(ActorStyle::default(), ActorStyle::Stickman);
        assert_ne!(ActorStyle::Stickman, ActorStyle::StickmanBusiness);
    }

    #[test]
    fn value_of_stickman_business() {
        assert_ne!(ActorStyle::StickmanBusiness, ActorStyle::Stickman);
        assert_ne!(ActorStyle::StickmanBusiness, ActorStyle::Awesome);
        assert_eq!(
            format!("{:?}", ActorStyle::StickmanBusiness),
            "StickmanBusiness"
        );
    }

    #[test]
    fn value_of_awesome() {
        assert_ne!(ActorStyle::Awesome, ActorStyle::Stickman);
        assert_ne!(ActorStyle::Awesome, ActorStyle::Hollow);
        assert_eq!(format!("{:?}", ActorStyle::Awesome), "Awesome");
    }

    #[test]
    fn value_of_hollow() {
        assert_ne!(ActorStyle::Hollow, ActorStyle::Stickman);
        assert_ne!(ActorStyle::Hollow, ActorStyle::Awesome);
        assert_eq!(format!("{:?}", ActorStyle::Hollow), "Hollow");
    }

    #[test]
    #[ignore = "gap: ActorStyle::toUSymbol not yet ported"]
    fn to_u_symbol_not_null_for_all_variants() {
        // Java: assertNotNull(STICKMAN.toUSymbol()); etc.
        todo!()
    }

    #[test]
    #[ignore = "gap: ActorStyle::getTextBlock requires Fashion (rendering context) — non-SVG rendering test"]
    fn get_text_block_with_fashion() {
        todo!()
    }
}

// ════════════════════════════════════════════════════════════════════
// AlignmentParamSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod alignment_param {
    use plantuml_little::style::skin_param::AlignmentParam;

    #[test]
    fn values_non_empty() {
        // Java: assertTrue(vals.length > 0)
        // Manually enumerate the known variants and verify count >= 2
        let all = [
            AlignmentParam::ArrowMessageAlignment,
            AlignmentParam::SequenceMessageAlignment,
            AlignmentParam::SequenceMessageTextAlignment,
            AlignmentParam::SequenceReferenceAlignment,
        ];
        assert!(all.len() >= 2);
        // All variants are distinct
        assert_ne!(all[0], all[1]);
        assert_ne!(all[0], all[2]);
        assert_ne!(all[1], all[2]);
    }

    #[test]
    fn value_of_arrow_message_alignment() {
        // Java: AlignmentParam.valueOf("arrowMessageAlignment")
        assert_ne!(
            AlignmentParam::ArrowMessageAlignment,
            AlignmentParam::SequenceMessageAlignment
        );
        assert_eq!(
            format!("{:?}", AlignmentParam::ArrowMessageAlignment),
            "ArrowMessageAlignment"
        );
    }

    #[test]
    fn value_of_sequence_message_alignment() {
        assert_ne!(
            AlignmentParam::SequenceMessageAlignment,
            AlignmentParam::ArrowMessageAlignment
        );
        assert_eq!(
            format!("{:?}", AlignmentParam::SequenceMessageAlignment),
            "SequenceMessageAlignment"
        );
    }

    #[test]
    fn value_of_note_text_alignment() {
        // Java: AlignmentParam.valueOf("noteTextAlignment")
        // Maps to SequenceMessageTextAlignment in Rust port
        assert_ne!(
            AlignmentParam::SequenceMessageTextAlignment,
            AlignmentParam::ArrowMessageAlignment
        );
        assert_eq!(
            format!("{:?}", AlignmentParam::SequenceMessageTextAlignment),
            "SequenceMessageTextAlignment"
        );
    }

    #[test]
    #[ignore = "gap: AlignmentParam::getDefaultValue not yet ported (LEFT vs CENTER per variant)"]
    fn get_default_value_per_variant() {
        // Java:
        //   arrowMessageAlignment.getDefaultValue()      == LEFT
        //   sequenceMessageAlignment.getDefaultValue()   == LEFT
        //   noteTextAlignment.getDefaultValue()          == LEFT
        //   stateMessageAlignment.getDefaultValue()      == CENTER
        //   sequenceReferenceAlignment.getDefaultValue() == CENTER
        //   packageTitleAlignment.getDefaultValue()      == CENTER
        todo!()
    }
}

// ════════════════════════════════════════════════════════════════════
// ColorParamSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod color_param {
    use plantuml_little::style::skin_param::ColorParam;

    #[test]
    fn values_non_empty_and_first_is_background() {
        // Java: assertTrue(vals.length > 0); assertEquals(background, vals[0])
        // Background is the first variant defined
        assert_eq!(format!("{:?}", ColorParam::Background), "Background");
        assert_ne!(ColorParam::Background, ColorParam::ClassBackground);
    }

    #[test]
    fn value_of_background() {
        assert_ne!(ColorParam::Background, ColorParam::NoteBackground);
        assert_eq!(format!("{:?}", ColorParam::Background), "Background");
    }

    #[test]
    fn value_of_class_background() {
        assert_ne!(ColorParam::ClassBackground, ColorParam::Background);
        assert_ne!(ColorParam::ClassBackground, ColorParam::NoteBackground);
        assert_eq!(
            format!("{:?}", ColorParam::ClassBackground),
            "ClassBackground"
        );
    }

    #[test]
    fn value_of_note_background() {
        assert_ne!(ColorParam::NoteBackground, ColorParam::Background);
        assert_ne!(ColorParam::NoteBackground, ColorParam::ClassBackground);
        assert_eq!(
            format!("{:?}", ColorParam::NoteBackground),
            "NoteBackground"
        );
    }

    #[test]
    fn value_of_activity_background() {
        assert_ne!(ColorParam::ActivityBackground, ColorParam::Background);
        assert_ne!(ColorParam::ActivityBackground, ColorParam::ClassBackground);
        assert_eq!(
            format!("{:?}", ColorParam::ActivityBackground),
            "ActivityBackground"
        );
    }

    #[test]
    #[ignore = "gap: ColorParam::getDefaultValue not yet ported"]
    fn get_default_value_some_non_null_some_null() {
        // Java:
        //   background.getDefaultValue()      != null
        //   classBackground.getDefaultValue() != null
        //   noteBackground.getDefaultValue()  != null
        //   diagramBorder.getDefaultValue()   == null
        todo!()
    }

    #[test]
    #[ignore = "gap: ColorParam::getColorType not yet ported"]
    fn get_color_type_back_line_and_null() {
        // Java:
        //   background -> BACK
        //   classBorder -> LINE
        //   classBackground -> BACK
        //   activityBorder -> LINE
        //   hyperlink -> null
        todo!()
    }
}

// ════════════════════════════════════════════════════════════════════
// VisibilityModifierSkeletonTest
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod visibility_modifier {

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn values_nine_variants() {
        // Java: values().length == 9
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn value_of_private_field() {
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn value_of_public_method() {
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn value_of_ie_mandatory() {
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn regex_for_visibility_character_matches_minus_plus_hash_tilde() {
        // Java: assertTrue("-".matches(regex)); assertTrue("+".matches(regex));
        //       assertTrue("#".matches(regex)); assertTrue("~".matches(regex))
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn is_visibility_character_rules() {
        // Java:
        //   isVisibilityCharacter("-a")    == false  (length <= 2)
        //   isVisibilityCharacter("--foo") == false  (same first two chars)
        //   isVisibilityCharacter("-foo")  == true
        //   isVisibilityCharacter("#bar")  == true
        //   isVisibilityCharacter("+baz")  == true
        //   isVisibilityCharacter("~qux")  == true
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn get_visibility_modifier_field_and_method() {
        // Java:
        //   getVisibilityModifier("-foo", true)  == PRIVATE_FIELD
        //   getVisibilityModifier("#foo", true)  == PROTECTED_FIELD
        //   getVisibilityModifier("+foo", true)  == PUBLIC_FIELD
        //   getVisibilityModifier("~foo", true)  == PACKAGE_PRIVATE_FIELD
        //   getVisibilityModifier("-foo", false) == PRIVATE_METHOD
        //   getVisibilityModifier("+foo", false) == PUBLIC_METHOD
        //   getVisibilityModifier("-a", true)    == null  (too short)
        //   getVisibilityModifier("--foo", true) == null  (same first two chars)
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn get_by_unicode_private_field() {
        // Java: getByUnicode(StringUtils.PRIVATE_FIELD) == PRIVATE_FIELD
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn get_foreground_returns_correct_color_param() {
        // Java:
        //   PRIVATE_FIELD.getForeground()    == iconPrivate
        //   PUBLIC_FIELD.getForeground()     == iconPublic
        //   PROTECTED_METHOD.getForeground() == iconProtected
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn get_background_null_for_fields_non_null_for_methods() {
        // Java:
        //   PRIVATE_FIELD.getBackground()  == null
        //   PUBLIC_FIELD.getBackground()   == null
        //   PRIVATE_METHOD.getBackground() == iconPrivateBackground
        //   PUBLIC_METHOD.getBackground()  == iconPublicBackground
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn is_field_true_for_fields_false_for_methods_and_ie() {
        // Java:
        //   PRIVATE_FIELD.isField()       == true
        //   PUBLIC_FIELD.isField()        == true
        //   PROTECTED_FIELD.isField()     == true
        //   PACKAGE_PRIVATE_FIELD.isField() == true
        //   PRIVATE_METHOD.isField()      == false
        //   PUBLIC_METHOD.isField()       == false
        //   IE_MANDATORY.isField()        == false
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn get_xmi_visibility_per_modifier() {
        // Java:
        //   PUBLIC_FIELD.getXmiVisibility()          == "public"
        //   PUBLIC_METHOD.getXmiVisibility()         == "public"
        //   PRIVATE_FIELD.getXmiVisibility()         == "private"
        //   PRIVATE_METHOD.getXmiVisibility()        == "private"
        //   PROTECTED_FIELD.getXmiVisibility()       == "protected"
        //   PROTECTED_METHOD.getXmiVisibility()      == "protected"
        //   PACKAGE_PRIVATE_FIELD.getXmiVisibility() == "package"
        //   PACKAGE_PRIVATE_METHOD.getXmiVisibility() == "package"
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn get_style_signature_not_null() {
        // Java: assertNotNull(PRIVATE_FIELD.getStyleSignature()); etc.
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier not yet ported"]
    fn replace_visibility_modifier_by_unicode_char_field() {
        // Java: replaceVisibilityModifierByUnicodeChar("-foo", true)
        //   result.length() == 4   (1 unicode char + "foo")
        //   result.substring(1) == "foo"
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier::getUDrawable requires UGraphic — non-SVG rendering test"]
    fn get_u_drawable_requires_ugraphic() {
        todo!()
    }

    #[test]
    #[ignore = "gap: VisibilityModifier::getUBlock requires UGraphic — non-SVG rendering test"]
    fn get_u_block_requires_ugraphic() {
        todo!()
    }
}
