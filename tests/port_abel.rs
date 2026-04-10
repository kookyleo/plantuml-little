// Port of Java abel-package skeleton tests to Rust.
// Source: generated-public-api-tests-foundation/.../abel/
//
// Mapping notes:
//   Java EntityPosition           → abel::EntityPosition (re-exported from svek::node)
//   Java GroupType                → abel::GroupType
//   Java LeafType                 → abel::LeafType
//   Java NoteLinkStrategy         → abel::NoteLinkStrategy
//   Java LinkArrow                → abel::LinkArrow
//   Java LinkStrategy             → abel::link::LinkStrategy
//   Java EntityPortion            → abel::EntityPortion
//   Java CucaNote                 → abel::CucaNote  (simplified: no Colors/Display)
//   Java LinkArg                  → abel::LinkArg
//   Java DisplayPositioned        → abel::DisplayPositioned (simplified)
//   Java EntityGender / Utils     → abel::EntityGender enum
//   Java Together                 → abel::Together (in svek::node)

// ═══════════════════════════════════════════════════════════════════════════
// EntityPositionSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod entity_position {
    use plantuml_little::abel::EntityPosition;

    #[test]
    fn values_all_9_variants() {
        // Java: assertEquals(9, values.length)
        let variants = [
            EntityPosition::Normal,
            EntityPosition::EntryPoint,
            EntityPosition::ExitPoint,
            EntityPosition::InputPin,
            EntityPosition::OutputPin,
            EntityPosition::ExpansionInput,
            EntityPosition::ExpansionOutput,
            EntityPosition::PortIn,
            EntityPosition::PortOut,
        ];
        assert_eq!(variants.len(), 9);
        // All distinct
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
    fn value_of_all_names() {
        // Java: assertSame(EntityPosition.NORMAL, EntityPosition.valueOf("NORMAL")) etc.
        // Verify Debug representation matches expected name (equivalent to valueOf by name).
        assert_eq!(format!("{:?}", EntityPosition::Normal), "Normal");
        assert_eq!(format!("{:?}", EntityPosition::EntryPoint), "EntryPoint");
        assert_eq!(format!("{:?}", EntityPosition::ExitPoint), "ExitPoint");
        assert_eq!(format!("{:?}", EntityPosition::InputPin), "InputPin");
        assert_eq!(format!("{:?}", EntityPosition::OutputPin), "OutputPin");
        assert_eq!(
            format!("{:?}", EntityPosition::ExpansionInput),
            "ExpansionInput"
        );
        assert_eq!(
            format!("{:?}", EntityPosition::ExpansionOutput),
            "ExpansionOutput"
        );
        assert_eq!(format!("{:?}", EntityPosition::PortIn), "PortIn");
        assert_eq!(format!("{:?}", EntityPosition::PortOut), "PortOut");
        // Cross-inequality: all variants are distinct
        assert_ne!(EntityPosition::Normal, EntityPosition::EntryPoint);
        assert_ne!(EntityPosition::InputPin, EntityPosition::OutputPin);
        assert_ne!(EntityPosition::PortIn, EntityPosition::PortOut);
    }

    #[test]
    fn get_inputs_set() {
        // Java: getInputs() returns {ENTRY_POINT, INPUT_PIN, EXPANSION_INPUT, PORTIN}
        let all = [
            EntityPosition::Normal,
            EntityPosition::EntryPoint,
            EntityPosition::ExitPoint,
            EntityPosition::InputPin,
            EntityPosition::OutputPin,
            EntityPosition::ExpansionInput,
            EntityPosition::ExpansionOutput,
            EntityPosition::PortIn,
            EntityPosition::PortOut,
        ];
        let inputs: Vec<EntityPosition> = all.iter().filter(|p| p.is_input()).copied().collect();
        assert_eq!(inputs.len(), 4);
        assert!(inputs.contains(&EntityPosition::EntryPoint));
        assert!(inputs.contains(&EntityPosition::InputPin));
        assert!(inputs.contains(&EntityPosition::ExpansionInput));
        assert!(inputs.contains(&EntityPosition::PortIn));
    }

    #[test]
    fn get_outputs_set() {
        // Java: getOutputs() returns {EXIT_POINT, OUTPUT_PIN, EXPANSION_OUTPUT, PORTOUT}
        let all = [
            EntityPosition::Normal,
            EntityPosition::EntryPoint,
            EntityPosition::ExitPoint,
            EntityPosition::InputPin,
            EntityPosition::OutputPin,
            EntityPosition::ExpansionInput,
            EntityPosition::ExpansionOutput,
            EntityPosition::PortIn,
            EntityPosition::PortOut,
        ];
        let outputs: Vec<EntityPosition> = all.iter().filter(|p| p.is_output()).copied().collect();
        assert_eq!(outputs.len(), 4);
        assert!(outputs.contains(&EntityPosition::ExitPoint));
        assert!(outputs.contains(&EntityPosition::OutputPin));
        assert!(outputs.contains(&EntityPosition::ExpansionOutput));
        assert!(outputs.contains(&EntityPosition::PortOut));
    }

    #[test]
    fn get_normals_set() {
        // Java: getNormals() == {NORMAL}
        let all = [
            EntityPosition::Normal,
            EntityPosition::EntryPoint,
            EntityPosition::ExitPoint,
            EntityPosition::InputPin,
            EntityPosition::OutputPin,
            EntityPosition::ExpansionInput,
            EntityPosition::ExpansionOutput,
            EntityPosition::PortIn,
            EntityPosition::PortOut,
        ];
        let normals: Vec<EntityPosition> = all.iter().filter(|p| p.is_normal()).copied().collect();
        assert_eq!(normals.len(), 1);
        assert!(normals.contains(&EntityPosition::Normal));
    }

    #[test]
    fn is_normal() {
        // Java: assertTrue(NORMAL.isNormal()); assertFalse(ENTRY_POINT.isNormal()) etc.
        assert!(EntityPosition::Normal.is_normal());
        assert!(!EntityPosition::EntryPoint.is_normal());
        assert!(!EntityPosition::ExitPoint.is_normal());
        assert!(!EntityPosition::InputPin.is_normal());
        assert!(!EntityPosition::PortIn.is_normal());
    }

    #[test]
    fn is_input() {
        // Java: assertFalse(NORMAL.isInput()); assertTrue(ENTRY_POINT.isInput()) etc.
        assert!(!EntityPosition::Normal.is_input());
        assert!(EntityPosition::EntryPoint.is_input());
        assert!(EntityPosition::InputPin.is_input());
        assert!(EntityPosition::ExpansionInput.is_input());
        assert!(EntityPosition::PortIn.is_input());
        assert!(!EntityPosition::ExitPoint.is_input());
        assert!(!EntityPosition::OutputPin.is_input());
        assert!(!EntityPosition::ExpansionOutput.is_input());
        assert!(!EntityPosition::PortOut.is_input());
    }

    #[test]
    fn is_output() {
        // Java: assertFalse(NORMAL.isOutput()); assertTrue(EXIT_POINT.isOutput()) etc.
        assert!(!EntityPosition::Normal.is_output());
        assert!(EntityPosition::ExitPoint.is_output());
        assert!(EntityPosition::OutputPin.is_output());
        assert!(EntityPosition::ExpansionOutput.is_output());
        assert!(EntityPosition::PortOut.is_output());
        assert!(!EntityPosition::EntryPoint.is_output());
        assert!(!EntityPosition::InputPin.is_output());
        assert!(!EntityPosition::ExpansionInput.is_output());
        assert!(!EntityPosition::PortIn.is_output());
    }

    // Java: @Ignore("drawSymbol requires a live UGraphic renderer — skip")
    // → no Rust test needed

    #[test]
    fn get_dimension_radius_constant() {
        // Java: getDimension(Rankdir) tests with RADIUS constant; full Rankdir enum not ported.
        // We verify RADIUS == 6.0 and is positive.
        let r = EntityPosition::RADIUS;
        assert_eq!(r, 6.0);
        assert!(r > 0.0);
        // Java: non-expansion positions return 2R x 2R
        // Java: EntityPosition.EXPANSION_INPUT.getDimension(TOP_TO_BOTTOM) → 4*2R x 2R
        // → getDimension() not yet ported with Rankdir; covered by ignore below
    }

    #[test]
    #[ignore = "gap: getDimension(Rankdir) not yet ported — Rankdir enum absent"]
    fn get_dimension_expansion_variants() {
        todo!("EntityPosition::get_dimension(Rankdir) not available in Rust")
    }

    #[test]
    #[ignore = "gap: getShapeType() not yet ported to Rust EntityPosition"]
    fn get_shape_type_for_non_normal() {
        todo!("EntityPosition::get_shape_type() not available in Rust")
    }

    #[test]
    #[ignore = "gap: getShapeType() panics for NORMAL not yet ported"]
    fn get_shape_type_throws_for_normal() {
        todo!("EntityPosition::get_shape_type() not available in Rust")
    }

    #[test]
    fn from_stereotype() {
        // Java: fromStereotype() — case-insensitive matching via lowercase
        assert_eq!(
            EntityPosition::from_stereotype("<<entrypoint>>"),
            EntityPosition::EntryPoint
        );
        assert_eq!(
            EntityPosition::from_stereotype("<<ENTRYPOINT>>"),
            EntityPosition::EntryPoint
        );
        assert_eq!(
            EntityPosition::from_stereotype("<<exitpoint>>"),
            EntityPosition::ExitPoint
        );
        assert_eq!(
            EntityPosition::from_stereotype("<<inputpin>>"),
            EntityPosition::InputPin
        );
        assert_eq!(
            EntityPosition::from_stereotype("<<outputpin>>"),
            EntityPosition::OutputPin
        );
        assert_eq!(
            EntityPosition::from_stereotype("<<expansioninput>>"),
            EntityPosition::ExpansionInput
        );
        assert_eq!(
            EntityPosition::from_stereotype("<<expansionoutput>>"),
            EntityPosition::ExpansionOutput
        );
        // Unknown stereotype falls back to NORMAL
        assert_eq!(
            EntityPosition::from_stereotype("<<unknown>>"),
            EntityPosition::Normal
        );
    }

    #[test]
    fn is_port() {
        // Java: assertTrue(PORTIN.isPort()); assertTrue(PORTOUT.isPort()); assertFalse rest
        assert!(EntityPosition::PortIn.is_port());
        assert!(EntityPosition::PortOut.is_port());
        assert!(!EntityPosition::Normal.is_port());
        assert!(!EntityPosition::EntryPoint.is_port());
        assert!(!EntityPosition::ExitPoint.is_port());
        assert!(!EntityPosition::InputPin.is_port());
        assert!(!EntityPosition::OutputPin.is_port());
    }

    #[test]
    fn use_port_p() {
        // Java: assertTrue for PORTIN, PORTOUT, ENTRY_POINT, EXIT_POINT;
        //       assertFalse for NORMAL, INPUT_PIN, OUTPUT_PIN, EXPANSION_INPUT, EXPANSION_OUTPUT
        assert!(EntityPosition::PortIn.use_port_p());
        assert!(EntityPosition::PortOut.use_port_p());
        assert!(EntityPosition::EntryPoint.use_port_p());
        assert!(EntityPosition::ExitPoint.use_port_p());
        assert!(!EntityPosition::Normal.use_port_p());
        assert!(!EntityPosition::InputPin.use_port_p());
        assert!(!EntityPosition::OutputPin.use_port_p());
        assert!(!EntityPosition::ExpansionInput.use_port_p());
        assert!(!EntityPosition::ExpansionOutput.use_port_p());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GroupTypeSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod group_type {
    use plantuml_little::abel::GroupType;

    #[test]
    fn values_8_variants() {
        // Java: assertEquals(8, values.length)
        let variants = [
            GroupType::Root,
            GroupType::Package,
            GroupType::State,
            GroupType::ConcurrentState,
            GroupType::InnerActivity,
            GroupType::ConcurrentActivity,
            GroupType::Domain,
            GroupType::Requirement,
        ];
        assert_eq!(variants.len(), 8);
    }

    #[test]
    fn value_of_all_names() {
        // Java: assertSame(GroupType.ROOT, GroupType.valueOf("ROOT")) etc.
        // Verify Debug representation matches expected name (equivalent to valueOf by name).
        assert_eq!(format!("{:?}", GroupType::Root), "Root");
        assert_eq!(format!("{:?}", GroupType::Package), "Package");
        assert_eq!(format!("{:?}", GroupType::State), "State");
        assert_eq!(
            format!("{:?}", GroupType::ConcurrentState),
            "ConcurrentState"
        );
        assert_eq!(format!("{:?}", GroupType::InnerActivity), "InnerActivity");
        assert_eq!(
            format!("{:?}", GroupType::ConcurrentActivity),
            "ConcurrentActivity"
        );
        assert_eq!(format!("{:?}", GroupType::Domain), "Domain");
        assert_eq!(format!("{:?}", GroupType::Requirement), "Requirement");
        // Cross-inequality: all variants are distinct
        assert_ne!(GroupType::Root, GroupType::Package);
        assert_ne!(GroupType::State, GroupType::ConcurrentState);
        assert_ne!(GroupType::InnerActivity, GroupType::ConcurrentActivity);
        assert_ne!(GroupType::Domain, GroupType::Requirement);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LeafTypeSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod leaf_type {
    use plantuml_little::abel::LeafType;

    #[test]
    fn values_contains_key_variants() {
        // Java: assertTrue(values.length > 0) + spot-check CLASS, INTERFACE, STATE
        let class = LeafType::Class;
        let iface = LeafType::Interface;
        let state = LeafType::State;
        assert_ne!(class, iface);
        assert_ne!(class, state);
        assert_ne!(iface, state);
    }

    #[test]
    fn value_of_exact_names() {
        // Java: assertSame(LeafType.CLASS, LeafType.valueOf("CLASS")) etc.
        // Verify Debug representation matches expected name (equivalent to valueOf by name).
        assert_eq!(format!("{:?}", LeafType::Class), "Class");
        assert_eq!(format!("{:?}", LeafType::Interface), "Interface");
        assert_eq!(format!("{:?}", LeafType::Enum), "Enum");
        assert_eq!(format!("{:?}", LeafType::AbstractClass), "AbstractClass");
        assert_eq!(format!("{:?}", LeafType::State), "State");
        assert_eq!(format!("{:?}", LeafType::Note), "Note");
        assert_eq!(format!("{:?}", LeafType::Usecase), "Usecase");
        assert_eq!(format!("{:?}", LeafType::StillUnknown), "StillUnknown");
        // Cross-inequality: selected variants are distinct
        assert_ne!(LeafType::Class, LeafType::Interface);
        assert_ne!(LeafType::State, LeafType::Note);
        assert_ne!(LeafType::AbstractClass, LeafType::Class);
    }

    #[test]
    fn get_leaf_type_exact_match() {
        // Java: LeafType.getLeafType("CLASS") == CLASS
        assert_eq!(LeafType::from_str_loose("CLASS"), Some(LeafType::Class));
        assert_eq!(
            LeafType::from_str_loose("INTERFACE"),
            Some(LeafType::Interface)
        );
        assert_eq!(LeafType::from_str_loose("ENUM"), Some(LeafType::Enum));
    }

    #[test]
    fn get_leaf_type_abstract_prefix() {
        // Java: "abstract"/"ABSTRACT" → ABSTRACT_CLASS; "ABSTRACT_CLASS" → ABSTRACT_CLASS
        assert_eq!(
            LeafType::from_str_loose("ABSTRACT"),
            Some(LeafType::AbstractClass)
        );
        assert_eq!(
            LeafType::from_str_loose("abstract"),
            Some(LeafType::AbstractClass)
        );
        assert_eq!(
            LeafType::from_str_loose("ABSTRACT_CLASS"),
            Some(LeafType::AbstractClass)
        );
    }

    #[test]
    fn get_leaf_type_diamond_prefix() {
        // Java: "DIAMOND"/"diamond" → STATE_CHOICE
        assert_eq!(
            LeafType::from_str_loose("DIAMOND"),
            Some(LeafType::StateChoice)
        );
        assert_eq!(
            LeafType::from_str_loose("diamond"),
            Some(LeafType::StateChoice)
        );
    }

    #[test]
    fn get_leaf_type_static_prefix() {
        // Java: "static"/"STATIC" → CLASS
        assert_eq!(LeafType::from_str_loose("STATIC"), Some(LeafType::Class));
        assert_eq!(LeafType::from_str_loose("static"), Some(LeafType::Class));
    }

    #[test]
    fn get_leaf_type_case_insensitive() {
        // Java: lowercased input still resolves correctly
        assert_eq!(LeafType::from_str_loose("class"), Some(LeafType::Class));
        assert_eq!(
            LeafType::from_str_loose("interface"),
            Some(LeafType::Interface)
        );
    }

    #[test]
    fn is_like_class_true_cases() {
        // Java: assertTrue for each of these
        assert!(LeafType::Class.is_like_class());
        assert!(LeafType::AbstractClass.is_like_class());
        assert!(LeafType::Interface.is_like_class());
        assert!(LeafType::Enum.is_like_class());
        assert!(LeafType::Annotation.is_like_class());
        assert!(LeafType::Entity.is_like_class());
        assert!(LeafType::Protocol.is_like_class());
        assert!(LeafType::Struct.is_like_class());
        assert!(LeafType::Exception.is_like_class());
        assert!(LeafType::Metaclass.is_like_class());
        assert!(LeafType::Stereotype.is_like_class());
        assert!(LeafType::Dataclass.is_like_class());
        assert!(LeafType::Record.is_like_class());
    }

    #[test]
    fn is_like_class_false_cases() {
        // Java: assertFalse for each of these
        assert!(!LeafType::Note.is_like_class());
        assert!(!LeafType::Usecase.is_like_class());
        assert!(!LeafType::State.is_like_class());
        assert!(!LeafType::Activity.is_like_class());
        assert!(!LeafType::StillUnknown.is_like_class());
    }

    #[test]
    fn to_html() {
        // Java: toHtml() replaces '_' with ' ', lowercases, then capitalises first letter
        assert_eq!(LeafType::Class.to_html(), "Class");
        assert_eq!(LeafType::AbstractClass.to_html(), "Abstract class");
        assert_eq!(LeafType::Interface.to_html(), "Interface");
        assert_eq!(LeafType::Enum.to_html(), "Enum");
        assert_eq!(LeafType::State.to_html(), "State");
        assert_eq!(LeafType::Note.to_html(), "Note");
        assert_eq!(LeafType::StillUnknown.to_html(), "Still unknown");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EntityPortionSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod entity_portion {
    use plantuml_little::abel::EntityPortion;

    #[test]
    fn values_5_variants() {
        // Java: assertEquals(5, values.length)
        let variants = [
            EntityPortion::Field,
            EntityPortion::Method,
            EntityPortion::Member,
            EntityPortion::CircledCharacter,
            EntityPortion::Stereotype,
        ];
        assert_eq!(variants.len(), 5);
        // All distinct
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
    fn value_of_all_names() {
        // Java: assertSame(EntityPortion.FIELD, EntityPortion.valueOf("FIELD")) etc.
        // Verify Debug representation matches expected name (equivalent to valueOf by name).
        assert_eq!(format!("{:?}", EntityPortion::Field), "Field");
        assert_eq!(format!("{:?}", EntityPortion::Method), "Method");
        assert_eq!(format!("{:?}", EntityPortion::Member), "Member");
        assert_eq!(
            format!("{:?}", EntityPortion::CircledCharacter),
            "CircledCharacter"
        );
        assert_eq!(format!("{:?}", EntityPortion::Stereotype), "Stereotype");
        // Cross-inequality: all variants are distinct
        assert_ne!(EntityPortion::Field, EntityPortion::Method);
        assert_ne!(EntityPortion::Member, EntityPortion::Field);
        assert_ne!(EntityPortion::CircledCharacter, EntityPortion::Stereotype);
    }

    #[test]
    fn as_set_field_singleton() {
        // Java: FIELD.asSet() == {FIELD} (size 1)
        let set = EntityPortion::Field.as_set();
        assert_eq!(set.len(), 1);
        assert!(set.contains(&EntityPortion::Field));
    }

    #[test]
    fn as_set_method_singleton() {
        // Java: METHOD.asSet() == {METHOD} (size 1)
        let set = EntityPortion::Method.as_set();
        assert_eq!(set.len(), 1);
        assert!(set.contains(&EntityPortion::Method));
    }

    #[test]
    fn as_set_stereotype_singleton() {
        // Java: STEREOTYPE.asSet() == {STEREOTYPE} (size 1)
        let set = EntityPortion::Stereotype.as_set();
        assert_eq!(set.len(), 1);
        assert!(set.contains(&EntityPortion::Stereotype));
    }

    #[test]
    fn as_set_member_expands_to_field_and_method() {
        // Java: MEMBER.asSet() == {FIELD, METHOD} (size 2)
        let set = EntityPortion::Member.as_set();
        assert_eq!(set.len(), 2);
        assert!(set.contains(&EntityPortion::Field));
        assert!(set.contains(&EntityPortion::Method));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LinkArgSkeletonTest.java
// Note: Java LinkArg.build(Display, int) → LinkArg::new(vec![label], length)
//       Java LinkArg.noDisplay(int)      → LinkArg::no_display(length)
//       Java Display.isNull()            → label.is_empty()
//       Java getInv()                    → inverted()
//       Mutating setters mirror Java's mutable API.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod link_arg {
    use plantuml_little::abel::LinkArg;

    #[test]
    fn build_with_label_and_length() {
        // Java: build(label, 2) → getLength()==2, getLabel() non-null
        let arg = LinkArg::new(vec!["myLabel".to_string()], 2);
        assert_eq!(arg.length(), 2);
        assert!(!arg.label().is_empty());
    }

    #[test]
    fn no_display_has_empty_label() {
        // Java: noDisplay(3) → getLength()==3, Display.isNull(getLabel())==true
        let arg = LinkArg::no_display(3);
        assert_eq!(arg.length(), 3);
        assert!(arg.label().is_empty());
    }

    #[test]
    fn build_overload_with_boolean() {
        // Java: build(label, 1, false) → getLength()==1
        // (boolean param not modelled in Rust; basic length check)
        let arg = LinkArg::new(vec!["hello".to_string()], 1);
        assert_eq!(arg.length(), 1);
    }

    #[test]
    fn with_quantifier() {
        // Java: withQuantifier("1","*") → getQuantifier1()=="1", getQuantifier2()=="*"
        let base = LinkArg::no_display(1);
        let with_q = base.with_quantifier(Some("1".to_string()), Some("*".to_string()));
        assert_eq!(with_q.quantifier1(), Some("1"));
        assert_eq!(with_q.quantifier2(), Some("*"));
    }

    #[test]
    fn with_role() {
        // Java: withRole("owner","member") → getRole1()=="owner", getRole2()=="member"
        let base = LinkArg::no_display(1);
        let with_r = base.with_role(Some("owner".to_string()), Some("member".to_string()));
        assert_eq!(with_r.role1(), Some("owner"));
        assert_eq!(with_r.role2(), Some("member"));
    }

    #[test]
    fn with_kal() {
        // Java: withKal("k1","k2") → getKal1()=="k1", getKal2()=="k2"
        let base = LinkArg::no_display(1);
        let with_k = base.with_kal(Some("k1".to_string()), Some("k2".to_string()));
        assert_eq!(with_k.kal1(), Some("k1"));
        assert_eq!(with_k.kal2(), Some("k2"));
    }

    #[test]
    fn with_distance_angle() {
        // Java: withDistanceAngle("5.0","45") → getLabeldistance()=="5.0", getLabelangle()=="45"
        let base = LinkArg::no_display(1);
        let with_da = base.with_distance_angle(Some("5.0".to_string()), Some("45".to_string()));
        assert_eq!(with_da.label_distance(), Some("5.0"));
        assert_eq!(with_da.label_angle(), Some("45"));
    }

    #[test]
    fn get_inv_swaps_all_pairs() {
        // Java: getInv() swaps quantifier1/2, kal1/2, role1/2
        let base = LinkArg::no_display(2)
            .with_quantifier(Some("1".to_string()), Some("n".to_string()))
            .with_role(Some("src".to_string()), Some("dst".to_string()))
            .with_kal(Some("ka1".to_string()), Some("ka2".to_string()));
        let inv = base.inverted();
        assert_eq!(inv.quantifier1(), Some("n"));
        assert_eq!(inv.quantifier2(), Some("1"));
        assert_eq!(inv.role1(), Some("dst"));
        assert_eq!(inv.role2(), Some("src"));
        assert_eq!(inv.kal1(), Some("ka2"));
        assert_eq!(inv.kal2(), Some("ka1"));
    }

    #[test]
    fn get_label_no_display_is_empty() {
        // Java: Display.isNull(arg.getLabel()) == true
        let arg = LinkArg::no_display(1);
        assert!(arg.label().is_empty());
    }

    #[test]
    fn get_length() {
        // Java: getLength() == 4
        let arg = LinkArg::no_display(4);
        assert_eq!(arg.length(), 4);
    }

    #[test]
    fn get_quantifier1_default_none() {
        // Java: assertNull(getQuantifier1())
        let arg = LinkArg::no_display(1);
        assert!(arg.quantifier1().is_none());
    }

    #[test]
    fn get_quantifier2_default_none() {
        let arg = LinkArg::no_display(1);
        assert!(arg.quantifier2().is_none());
    }

    #[test]
    fn get_label_distance_default_none() {
        // Java: assertNull(getLabeldistance())
        let arg = LinkArg::no_display(1);
        assert!(arg.label_distance().is_none());
    }

    #[test]
    fn get_label_angle_default_none() {
        let arg = LinkArg::no_display(1);
        assert!(arg.label_angle().is_none());
    }

    #[test]
    fn get_visibility_modifier_default_none() {
        // Java: assertNull(getVisibilityModifier())
        let arg = LinkArg::no_display(1);
        assert!(arg.visibility_modifier().is_none());
    }

    #[test]
    fn set_visibility_modifier_null() {
        // Java: setVisibilityModifier(null) should not throw; remains null
        let mut arg = LinkArg::no_display(1);
        arg.set_visibility_modifier(None);
        assert!(arg.visibility_modifier().is_none());
    }

    #[test]
    fn set_length() {
        // Java: setLength(7) → getLength()==7
        let mut arg = LinkArg::no_display(1);
        arg.set_length(7);
        assert_eq!(arg.length(), 7);
    }

    #[test]
    fn get_kal1_default_none() {
        let arg = LinkArg::no_display(1);
        assert!(arg.kal1().is_none());
    }

    #[test]
    fn get_kal2_default_none() {
        let arg = LinkArg::no_display(1);
        assert!(arg.kal2().is_none());
    }

    #[test]
    fn get_role1_default_none() {
        let arg = LinkArg::no_display(1);
        assert!(arg.role1().is_none());
    }

    #[test]
    fn get_role2_default_none() {
        let arg = LinkArg::no_display(1);
        assert!(arg.role2().is_none());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LinkArrowSkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod link_arrow {
    use plantuml_little::abel::LinkArrow;

    #[test]
    fn values_3_variants() {
        // Java: assertEquals(3, values.length)
        let variants = [
            LinkArrow::NoneOrSeveral,
            LinkArrow::DirectNormal,
            LinkArrow::Backward,
        ];
        assert_eq!(variants.len(), 3);
        assert_ne!(variants[0], variants[1]);
        assert_ne!(variants[0], variants[2]);
        assert_ne!(variants[1], variants[2]);
    }

    #[test]
    fn value_of_all_names() {
        // Java: assertSame(LinkArrow.NONE_OR_SEVERAL, LinkArrow.valueOf("NONE_OR_SEVERAL")) etc.
        // Verify Debug representation matches expected name (equivalent to valueOf by name).
        assert_eq!(format!("{:?}", LinkArrow::NoneOrSeveral), "NoneOrSeveral");
        assert_eq!(format!("{:?}", LinkArrow::DirectNormal), "DirectNormal");
        assert_eq!(format!("{:?}", LinkArrow::Backward), "Backward");
        // Cross-inequality: all variants are distinct
        assert_ne!(LinkArrow::NoneOrSeveral, LinkArrow::DirectNormal);
        assert_ne!(LinkArrow::NoneOrSeveral, LinkArrow::Backward);
        assert_ne!(LinkArrow::DirectNormal, LinkArrow::Backward);
    }

    #[test]
    fn reverse_direct_to_backward() {
        // Java: DIRECT_NORMAL.reverse() == BACKWARD
        assert_eq!(LinkArrow::DirectNormal.reverse(), LinkArrow::Backward);
    }

    #[test]
    fn reverse_backward_to_direct() {
        // Java: BACKWARD.reverse() == DIRECT_NORMAL
        assert_eq!(LinkArrow::Backward.reverse(), LinkArrow::DirectNormal);
    }

    #[test]
    fn reverse_none_or_several_identity() {
        // Java: NONE_OR_SEVERAL.reverse() == NONE_OR_SEVERAL
        assert_eq!(LinkArrow::NoneOrSeveral.reverse(), LinkArrow::NoneOrSeveral);
    }

    // Java: @Ignore("mute() requires a concrete GuideLine implementation — skip")
    // → no Rust test needed
}

// ═══════════════════════════════════════════════════════════════════════════
// CucaNoteSkeletonTest.java
// Note: Java CucaNote.build(Display, Position, Colors) →
//       Rust CucaNote::new(Vec<String>, NotePosition)
//       Java Position.RIGHT/LEFT/TOP/BOTTOM → NotePosition::Top/Bottom
//       Colors field not ported; strategy behaviour is identical.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod cuca_note {
    use plantuml_little::abel::entity::NotePosition;
    use plantuml_little::abel::{CucaNote, NoteLinkStrategy};

    fn make_display() -> Vec<String> {
        vec!["test note".to_string()]
    }

    #[test]
    fn build_default_strategy_is_normal() {
        // Java: assertSame(NoteLinkStrategy.NORMAL, note.getStrategy())
        let note = CucaNote::new(make_display(), NotePosition::Top);
        assert_eq!(note.strategy, NoteLinkStrategy::Normal);
    }

    #[test]
    fn with_strategy_half_printed_full() {
        // Java: withStrategy(HALF_PRINTED_FULL) → strategy == HALF_PRINTED_FULL;
        //       original strategy unchanged (NORMAL)
        let note = CucaNote::new(make_display(), NotePosition::Top);
        let half_printed = note.with_strategy(NoteLinkStrategy::HalfPrintedFull);
        assert_eq!(half_printed.strategy, NoteLinkStrategy::HalfPrintedFull);
        // Original unchanged
        assert_eq!(note.strategy, NoteLinkStrategy::Normal);
    }

    #[test]
    fn with_strategy_half_not_printed() {
        let note = CucaNote::new(make_display(), NotePosition::Top);
        let not_printed = note.with_strategy(NoteLinkStrategy::HalfNotPrinted);
        assert_eq!(not_printed.strategy, NoteLinkStrategy::HalfNotPrinted);
    }

    #[test]
    fn get_display() {
        // Java: assertSame(display, note.getDisplay())
        let text = make_display();
        let note = CucaNote::new(text.clone(), NotePosition::Top);
        assert_eq!(note.display, text);
    }

    #[test]
    fn get_strategy_default() {
        // Java: assertSame(NoteLinkStrategy.NORMAL, note.getStrategy())
        let note = CucaNote::new(make_display(), NotePosition::Top);
        assert_eq!(note.strategy, NoteLinkStrategy::Normal);
    }

    // Java: getColors() — Colors not ported; gap documented
    #[test]
    #[ignore = "gap: Colors field not ported to Rust CucaNote"]
    fn get_colors() {
        todo!("Colors not ported to Rust CucaNote")
    }

    #[test]
    fn get_position_top_and_bottom() {
        // Java: iterates Position.values() and checks getPosition()
        // Rust NotePosition only has Top/Bottom (LEFT/RIGHT/BOTTOM → Bottom, TOP → Top)
        for pos in [NotePosition::Top, NotePosition::Bottom] {
            let note = CucaNote::new(make_display(), pos);
            assert_eq!(note.position, pos);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NoteLinkStrategySkeletonTest.java
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod note_link_strategy {
    use plantuml_little::abel::NoteLinkStrategy;

    #[test]
    fn values_3_variants() {
        // Java: assertEquals(3, values.length)
        let variants = [
            NoteLinkStrategy::Normal,
            NoteLinkStrategy::HalfPrintedFull,
            NoteLinkStrategy::HalfNotPrinted,
        ];
        assert_eq!(variants.len(), 3);
        assert_ne!(variants[0], variants[1]);
        assert_ne!(variants[0], variants[2]);
        assert_ne!(variants[1], variants[2]);
    }

    #[test]
    fn value_of_all_names() {
        // Java: assertSame(NORMAL, ...) etc.
        // Verify Debug representation matches expected name (equivalent to valueOf by name).
        assert_eq!(format!("{:?}", NoteLinkStrategy::Normal), "Normal");
        assert_eq!(
            format!("{:?}", NoteLinkStrategy::HalfPrintedFull),
            "HalfPrintedFull"
        );
        assert_eq!(
            format!("{:?}", NoteLinkStrategy::HalfNotPrinted),
            "HalfNotPrinted"
        );
        // Cross-inequality: all variants are distinct
        assert_ne!(NoteLinkStrategy::Normal, NoteLinkStrategy::HalfPrintedFull);
        assert_ne!(NoteLinkStrategy::Normal, NoteLinkStrategy::HalfNotPrinted);
        assert_ne!(
            NoteLinkStrategy::HalfPrintedFull,
            NoteLinkStrategy::HalfNotPrinted
        );
    }

    #[test]
    fn compute_dimension_normal_full_size() {
        // Java: NORMAL returns full width x height
        let (w, h) = NoteLinkStrategy::Normal.compute_dimension(100.0, 50.0);
        assert!((w - 100.0).abs() < 1e-9);
        assert!((h - 50.0).abs() < 1e-9);
    }

    #[test]
    fn compute_dimension_half_printed_full_half_width() {
        // Java: HALF_PRINTED_FULL returns width/2 x height
        let (w, h) = NoteLinkStrategy::HalfPrintedFull.compute_dimension(100.0, 50.0);
        assert!((w - 50.0).abs() < 1e-9);
        assert!((h - 50.0).abs() < 1e-9);
    }

    #[test]
    fn compute_dimension_half_not_printed_zero() {
        // Java: HALF_NOT_PRINTED returns 0 x 0
        let (w, h) = NoteLinkStrategy::HalfNotPrinted.compute_dimension(100.0, 50.0);
        assert!((w - 0.0).abs() < 1e-9);
        assert!((h - 0.0).abs() < 1e-9);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DisplayPositionedSkeletonTest.java
// Note: Java DisplayPositioned has LineLocation + rich withPage/withDisplay/
//       withHorizontalAlignment/withLocation accessors not all ported.
//       Ported: single(), none(), is_null(), alignment fields.
//       Gaps are annotated with #[ignore].
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod display_positioned {
    use plantuml_little::abel::DisplayPositioned;
    use plantuml_little::klimt::geom::{HorizontalAlignment, VerticalAlignment};

    fn make_display() -> Vec<String> {
        vec!["some text".to_string()]
    }

    #[test]
    #[ignore = "gap: withPage() / LineLocation not ported to Rust DisplayPositioned"]
    fn with_page() {
        todo!("withPage(int, int) not available in Rust DisplayPositioned")
    }

    #[test]
    #[ignore = "gap: withDisplay() copy-with-new-display not ported to Rust DisplayPositioned"]
    fn with_display() {
        todo!("withDisplay(Display) not available in Rust DisplayPositioned")
    }

    #[test]
    #[ignore = "gap: withHorizontalAlignment() copy not ported to Rust DisplayPositioned"]
    fn with_horizontal_alignment() {
        todo!("withHorizontalAlignment() not available in Rust DisplayPositioned")
    }

    #[test]
    #[ignore = "gap: withLocation() / LineLocation not ported to Rust DisplayPositioned"]
    fn with_location() {
        todo!("withLocation() not available in Rust DisplayPositioned")
    }

    #[test]
    fn single_factory() {
        // Java: single(display, LEFT, BOTTOM) → not null, display preserved, alignments set
        let dp = DisplayPositioned::single(
            make_display(),
            HorizontalAlignment::Left,
            VerticalAlignment::Bottom,
        );
        assert!(!dp.is_null());
        assert_eq!(dp.display, make_display());
        assert_eq!(dp.horizontal_alignment, HorizontalAlignment::Left);
        assert_eq!(dp.vertical_alignment, VerticalAlignment::Bottom);
    }

    #[test]
    fn single_overload_with_null_location() {
        // Java: single(null, display, RIGHT, TOP) → not null, RIGHT alignment
        // (LineLocation param not modelled; we just verify the basic factory)
        let dp = DisplayPositioned::single(
            make_display(),
            HorizontalAlignment::Right,
            VerticalAlignment::Top,
        );
        assert!(!dp.is_null());
        assert_eq!(dp.horizontal_alignment, HorizontalAlignment::Right);
    }

    #[test]
    fn none_factory_is_null() {
        // Java: none(CENTER, CENTER).isNull() == true
        let dp = DisplayPositioned::none();
        assert!(dp.is_null());
    }

    #[test]
    fn get_display() {
        // Java: assertSame(display, dp.getDisplay())
        let text = make_display();
        let dp = DisplayPositioned::single(
            text.clone(),
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
        );
        assert_eq!(dp.display, text);
    }

    #[test]
    fn get_horizontal_alignment_right() {
        // Java: getHorizontalAlignment() == RIGHT
        let dp = DisplayPositioned::single(
            make_display(),
            HorizontalAlignment::Right,
            VerticalAlignment::Top,
        );
        assert_eq!(dp.horizontal_alignment, HorizontalAlignment::Right);
    }

    #[test]
    fn get_vertical_alignment_bottom() {
        // Java: getVerticalAlignment() == BOTTOM
        let dp = DisplayPositioned::single(
            make_display(),
            HorizontalAlignment::Left,
            VerticalAlignment::Bottom,
        );
        assert_eq!(dp.vertical_alignment, VerticalAlignment::Bottom);
    }

    #[test]
    fn is_null_false_for_content() {
        // Java: single(...).isNull() == false
        let dp = DisplayPositioned::single(
            make_display(),
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
        );
        assert!(!dp.is_null());
    }

    #[test]
    fn is_null_true_for_none() {
        // Java: none(...).isNull() == true
        let dp = DisplayPositioned::none();
        assert!(dp.is_null());
    }

    #[test]
    #[ignore = "gap: hasUrl() not directly ported to Rust DisplayPositioned"]
    fn has_url() {
        todo!("hasUrl() not available in Rust DisplayPositioned")
    }

    #[test]
    #[ignore = "gap: getLineLocation() not ported to Rust DisplayPositioned"]
    fn get_line_location_default_null() {
        todo!("LineLocation not ported")
    }

    // Java: @Ignore("createRibbon requires ISkinSimple/Style rendering infrastructure — skip")
    // → no Rust test needed
}

// ═══════════════════════════════════════════════════════════════════════════
// EntityGenderSkeletonTest.java
// Note: Java uses interface + anonymous inner classes; Rust uses EntityGender enum.
//       Mock-based tests are ported using Entity::new_leaf() directly.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod entity_gender {
    use plantuml_little::abel::{Entity, EntityGender, LeafType};

    #[test]
    fn contains_always_true() {
        // Java: anonymous EntityGender.contains() always returns true → EntityGender::All
        let all = EntityGender::All;
        let e = Entity::new_leaf("anything", LeafType::Note);
        assert!(all.contains(&e));
    }

    #[test]
    fn contains_always_false() {
        // Java: anonymous EntityGender.contains() always returns false
        // → use ByClassName with name that will never match
        let none_gender = EntityGender::ByClassName("__nonexistent_xyz__".to_string());
        let e = Entity::new_leaf("SomeClass", LeafType::Class);
        assert!(!none_gender.contains(&e));
    }

    #[test]
    fn get_gender_returns_label() {
        // Java: getGender() returns the label string
        // EntityGender::All has None gender; ByClassName has Some
        assert_eq!(EntityGender::All.gender(), None);
        let g = EntityGender::ByClassName("foo".to_string());
        assert_eq!(g.gender(), Some("foo"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EntityGenderUtilsSkeletonTest.java
// Note: Java EntityGenderUtils static methods → EntityGender enum constructors.
//       Mockito mocks → Entity::new_leaf() with mutating methods.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod entity_gender_utils {
    use plantuml_little::abel::{Entity, EntityGender, LeafType};

    #[test]
    fn by_entity_type_class_contains_class_entity() {
        // Java: byEntityType(CLASS) → contains(classEntity)==true, contains(ifaceEntity)==false
        //       gender.getGender() == "CLASS" (not directly available via ByEntityType variant)
        let gender = EntityGender::ByEntityType(LeafType::Class);
        let class_entity = Entity::new_leaf("C", LeafType::Class);
        let iface_entity = Entity::new_leaf("I", LeafType::Interface);
        assert!(gender.contains(&class_entity));
        assert!(!gender.contains(&iface_entity));
        // Verify variant matches CLASS
        match &gender {
            EntityGender::ByEntityType(lt) => assert_eq!(*lt, LeafType::Class),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn by_entity_alone_uid_match() {
        // Java: byEntityAlone(target) → contains(same-uid)==true, contains(other)==false
        //       gender.getGender() == uid string
        let target = Entity::new_leaf("T", LeafType::Class);
        let uid = target.uid().to_string();
        let gender = EntityGender::ByEntityAlone(uid.clone());
        assert!(gender.contains(&target));
        assert_eq!(gender.gender(), Some(uid.as_str()));

        let other = Entity::new_leaf("O", LeafType::Class);
        assert!(!gender.contains(&other));
    }

    #[test]
    fn by_stereotype_match() {
        // Java: entity with matching stereotype → contains==true
        //       entity with null stereotype → false
        let mut e = Entity::new_leaf("S", LeafType::Class);
        e.set_stereotype(Some("<<service>>".to_string()));
        let gender = EntityGender::ByStereotype("<<service>>".to_string());
        assert_eq!(gender.gender(), Some("<<service>>"));
        // Note: Rust ByStereotype uses contains() on the stereotype string
        // The Java impl checks exact match on getStereotype().
        // Our Rust impl checks if stereotype contains the pattern string.
        // Since "<<service>>" contains "<<service>>" this is true.
        assert!(gender.contains(&e));

        // Entity with no stereotype → false
        let no_stereo = Entity::new_leaf("NoStereo", LeafType::Class);
        assert!(!gender.contains(&no_stereo));
    }

    #[test]
    fn and_both_all() {
        // Java: and(all, all) → contains any entity; getGender() == null
        let combined = EntityGender::And(Box::new(EntityGender::All), Box::new(EntityGender::All));
        let e = Entity::new_leaf("X", LeafType::Class);
        assert!(combined.contains(&e));
        assert!(combined.gender().is_none());
    }

    #[test]
    fn and_one_false() {
        // Java: and(all, byEntityType(CLASS)) with non-CLASS → false; with CLASS → true
        let class_only = EntityGender::ByEntityType(LeafType::Class);
        let and_gender = EntityGender::And(Box::new(EntityGender::All), Box::new(class_only));
        let non_class = Entity::new_leaf("I", LeafType::Interface);
        assert!(!and_gender.contains(&non_class));

        let class_entity = Entity::new_leaf("C", LeafType::Class);
        assert!(and_gender.contains(&class_entity));
    }

    #[test]
    fn all_contains_any_entity() {
        // Java: all() always contains any entity; getGender() == null
        let all = EntityGender::All;
        let e1 = Entity::new_leaf("E1", LeafType::Class);
        let e2 = Entity::new_leaf("E2", LeafType::Note);
        assert!(all.contains(&e1));
        assert!(all.contains(&e2));
        assert!(all.gender().is_none());
    }

    #[test]
    fn by_class_name_match() {
        // Java: byClassName("MyClass") → contains entity named "MyClass" == true
        //       gender.getGender() == "MyClass"
        let gender = EntityGender::ByClassName("MyClass".to_string());
        let match_e = Entity::new_leaf("MyClass", LeafType::Class);
        let no_match = Entity::new_leaf("OtherClass", LeafType::Class);
        assert!(gender.contains(&match_e));
        assert!(!gender.contains(&no_match));
        assert_eq!(gender.gender(), Some("MyClass"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TogetherSkeletonTest.java
// Note: Java Together(parent) → Rust Together::new(id, parent).
//       Java getParent() → Rust .parent field (Option<Box<Together>>).
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod together {
    use plantuml_little::abel::Together;

    #[test]
    fn root_has_no_parent() {
        // Java: new Together(null) → getParent() == null
        let root = Together::new(0, None);
        assert!(root.parent.is_none());
    }

    #[test]
    fn child_links_to_parent() {
        // Java: new Together(root) → getParent() == root
        let root = Together::new(0, None);
        let child = Together::new(1, Some(root.clone()));
        assert!(child.parent.is_some());
        assert_eq!(child.parent.as_ref().unwrap().id, root.id);
    }

    #[test]
    fn grandchild_parent_chain() {
        // Java: grandchild.getParent() == child, grandchild.getParent().getParent() == root
        let root = Together::new(0, None);
        let child = Together::new(1, Some(root.clone()));
        let grandchild = Together::new(2, Some(child.clone()));

        let grandchild_parent = grandchild.parent.as_ref().unwrap();
        assert_eq!(grandchild_parent.id, child.id);

        let grandchild_grandparent = grandchild_parent.parent.as_ref().unwrap();
        assert_eq!(grandchild_grandparent.id, root.id);
    }
}
