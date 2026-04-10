// Port of Java PlantUML net.atmp package tests to Rust.
//
// Source Java classes:
//   net.sourceforge.plantuml.atmp.SvgOption
//   net.sourceforge.plantuml.atmp.PixelImage
//   net.sourceforge.plantuml.atmp.SpecialText
//   net.sourceforge.plantuml.atmp.CucaDiagram (tested via end-to-end pipeline)
//
// Mapping notes:
//   Java SvgOption          -> gap: Rust uses render function params + klimt::svg::SvgGraphic
//   Java PixelImage         -> gap: Rust is SVG-only, no raster image wrapper
//   Java SpecialText        -> gap: not ported
//   Java CucaDiagram        -> plantuml_little::convert() end-to-end pipeline
//                              plantuml_little::parser::parse() for model inspection

// ═══════════════════════════════════════════════════════════════════════════
// SvgOption — Java: net.sourceforge.plantuml.atmp.SvgOption
//
// Java SvgOption is a builder holding SVG generation options (scale,
// preserveAspectRatio, lengthAdjust, backcolor, minDim, interactive, etc.).
// In Rust these options are scattered across render function parameters and
// klimt::svg::SvgGraphic / LengthAdjust.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod svg_option_tests {

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic() returns default with preserveAspectRatio=\"none\""]
    fn basic_preserve_aspect_ratio_default_is_none() {
        // Java: SvgOption basic = SvgOption.basic();
        //       assertEquals("none", basic.getPreserveAspectRatio());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getScale() == 1.0"]
    fn basic_scale_default_is_one() {
        // Java: assertEquals(1.0, SvgOption.basic().getScale(), 0.0001);
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getSvgDimensionStyle() == true"]
    fn basic_svg_dimension_style_default_is_true() {
        // Java: assertTrue(SvgOption.basic().getSvgDimensionStyle());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().isInteractive() == false"]
    fn basic_is_interactive_default_is_false() {
        // Java: assertFalse(SvgOption.basic().isInteractive());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getInteractiveBaseFilename() == null"]
    fn basic_interactive_base_filename_default_is_null() {
        // Java: assertNull(SvgOption.basic().getInteractiveBaseFilename());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getLengthAdjust() == LengthAdjust.defaultValue()"]
    fn basic_length_adjust_default() {
        // Java: assertEquals(LengthAdjust.defaultValue(), SvgOption.basic().getLengthAdjust());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getMinDim() width=0.0, height=0.0"]
    fn basic_min_dim_default_is_zero() {
        // Java: assertEquals(0.0, SvgOption.basic().getMinDim().getWidth(), 0.0001);
        //       assertEquals(0.0, SvgOption.basic().getMinDim().getHeight(), 0.0001);
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getBackcolor() == null"]
    fn basic_backcolor_default_is_null() {
        // Java: assertNull(SvgOption.basic().getBackcolor());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getColorMapper() == ColorMapper.IDENTITY"]
    fn basic_color_mapper_default_is_identity() {
        // Java: assertSame(ColorMapper.IDENTITY, SvgOption.basic().getColorMapper());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getHover() == null"]
    fn basic_hover_default_is_null() {
        // Java: assertNull(SvgOption.basic().getHover());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getLinkTarget() == null"]
    fn basic_link_target_default_is_null() {
        // Java: assertNull(SvgOption.basic().getLinkTarget());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getFont() == null"]
    fn basic_font_default_is_null() {
        // Java: assertNull(SvgOption.basic().getFont());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getTitle() == null"]
    fn basic_title_default_is_null() {
        // Java: assertNull(SvgOption.basic().getTitle());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getDesc() == null"]
    fn basic_desc_default_is_null() {
        // Java: assertNull(SvgOption.basic().getDesc());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: basic().getRootAttributes() is empty"]
    fn basic_root_attributes_default_is_empty() {
        // Java: assertTrue(SvgOption.basic().getRootAttributes().isEmpty());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withPreserveAspectRatio returns same instance (fluent)"]
    fn with_preserve_aspect_ratio_is_fluent() {
        // Java: SvgOption opt = SvgOption.basic();
        //       SvgOption ret = opt.withPreserveAspectRatio("xMidYMid meet");
        //       assertSame(opt, ret);
        //       assertEquals("xMidYMid meet", opt.getPreserveAspectRatio());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withScale(2.0) sets scale to 2.0"]
    fn with_scale_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withScale(2.0);
        //       assertEquals(2.0, opt.getScale(), 0.0001);
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withSvgDimensionStyle(false) changes the flag"]
    fn with_svg_dimension_style_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withSvgDimensionStyle(false);
        //       assertFalse(opt.getSvgDimensionStyle());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withInteractive(\"base\", true) sets interactive mode"]
    fn with_interactive_sets_mode() {
        // Java: SvgOption opt = SvgOption.basic().withInteractive("base", true);
        //       assertTrue(opt.isInteractive());
        //       assertEquals("base", opt.getInteractiveBaseFilename());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withLengthAdjust sets the LengthAdjust value"]
    fn with_length_adjust_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withLengthAdjust(LengthAdjust.SPACING_AND_GLYPHS);
        //       assertEquals(LengthAdjust.SPACING_AND_GLYPHS, opt.getLengthAdjust());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withMinDim sets minimum dimensions"]
    fn with_min_dim_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withMinDim(new XDimension2D(100, 50));
        //       assertEquals(100.0, opt.getMinDim().getWidth(), 0.0001);
        //       assertEquals(50.0, opt.getMinDim().getHeight(), 0.0001);
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withBackcolor sets backcolor"]
    fn with_backcolor_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withBackcolor(HColors.RED);
        //       assertNotNull(opt.getBackcolor());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withHover sets hover text"]
    fn with_hover_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withHover("tooltip");
        //       assertEquals("tooltip", opt.getHover());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withLinkTarget sets link target"]
    fn with_link_target_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withLinkTarget("_blank");
        //       assertEquals("_blank", opt.getLinkTarget());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withFont sets the default font"]
    fn with_font_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withFont("Courier");
        //       assertEquals("Courier", opt.getFont());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withTitle sets SVG <title> element text"]
    fn with_title_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withTitle("My Diagram");
        //       assertEquals("My Diagram", opt.getTitle());
        todo!()
    }

    #[test]
    #[ignore = "gap: SvgOption not directly ported — Java: withDesc sets SVG <desc> element text"]
    fn with_desc_changes_value() {
        // Java: SvgOption opt = SvgOption.basic().withDesc("description");
        //       assertEquals("description", opt.getDesc());
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PixelImage — Java: net.sourceforge.plantuml.atmp.PixelImage
//
// Java PixelImage wraps a BufferedImage with scale and color manipulation.
// Rust is SVG-only; there is no raster image wrapper.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod pixel_image_tests {

    #[test]
    #[ignore = "gap: PixelImage not ported — Java: getScale() default is 1.0"]
    fn get_scale_default_is_one() {
        // Java: PixelImage img = new PixelImage(buffered, AffineTransformType.TYPE_BILINEAR);
        //       assertEquals(1.0, img.getScale(), 0.0001);
        todo!()
    }

    #[test]
    #[ignore = "gap: PixelImage not ported — Java: withScale(2.0) produces new object with scale=2.0"]
    fn with_scale_returns_new_instance() {
        // Java: PixelImage img2 = img.withScale(2.0);
        //       assertEquals(2.0, img2.getScale(), 0.0001);
        //       assertNotSame(img, img2);
        todo!()
    }

    #[test]
    #[ignore = "gap: PixelImage not ported — Java: withScale cumulative: withScale(3.0).withScale(2.0) -> 6.0"]
    fn with_scale_is_cumulative() {
        // Java: PixelImage img = new PixelImage(buffered, type)
        //           .withScale(3.0).withScale(2.0);
        //       assertEquals(6.0, img.getScale(), 0.0001);
        todo!()
    }

    #[test]
    #[ignore = "gap: PixelImage not ported — Java: monochrome() returns non-null new instance"]
    fn monochrome_returns_new_instance() {
        // Java: PixelImage mono = img.monochrome();
        //       assertNotNull(mono);
        //       assertNotSame(img, mono);
        todo!()
    }

    #[test]
    #[ignore = "gap: PixelImage not ported — Java: muteColor(null) returns same instance"]
    fn mute_color_null_returns_same() {
        // Java: PixelImage result = img.muteColor(null);
        //       assertSame(img, result);
        todo!()
    }

    #[test]
    #[ignore = "gap: PixelImage not ported — Java: getImage() returns underlying BufferedImage"]
    fn get_image_returns_underlying() {
        // Java: assertNotNull(img.getImage());
        //       assertSame(buffered, img.getImage());
        todo!()
    }

    #[test]
    #[ignore = "gap: PixelImage not ported — Java: getWidth()/getHeight() match underlying image dimensions"]
    fn dimensions_match_underlying_image() {
        // Java: assertEquals(buffered.getWidth(), img.getWidth());
        //       assertEquals(buffered.getHeight(), img.getHeight());
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SpecialText — Java: net.sourceforge.plantuml.atmp.SpecialText
//
// Java SpecialText wraps a TextBlock with title retrieval and compression
// flags. Not ported to Rust.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod special_text_tests {

    #[test]
    #[ignore = "gap: SpecialText not ported — Java: getTitle() returns the stored TextBlock"]
    fn get_title_returns_stored_text_block() {
        // Java: SpecialText st = new SpecialText(title);
        //       assertSame(title, st.getTitle());
        todo!()
    }

    #[test]
    #[ignore = "gap: SpecialText not ported — Java: isIgnoreForCompressionOn(X) returns true for all CompressionMode values"]
    fn is_ignore_for_compression_on_all_modes_true() {
        // Java: for (CompressionMode mode : CompressionMode.values()) {
        //           assertTrue(st.isIgnoreForCompressionOn(mode));
        //       }
        todo!()
    }

    #[test]
    #[ignore = "gap: SpecialText not ported — Java: constructor stores non-null TextBlock"]
    fn constructor_stores_non_null_text_block() {
        // Java: TextBlock tb = new TextBlockUtils.emptyTextBlock();
        //       SpecialText st = new SpecialText(tb);
        //       assertNotNull(st.getTitle());
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CucaDiagram — Java: net.sourceforge.plantuml.atmp.CucaDiagram
//
// Java CucaDiagram is the base class for class/component/state/use-case
// diagrams. We test the equivalent pipeline via plantuml_little::convert()
// and plantuml_little::parser::parse() for model-level inspection.
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod cuca_diagram_tests {
    use plantuml_little::convert;
    use plantuml_little::model::Diagram;
    use plantuml_little::parser;

    // ── Helper ──────────────────────────────────────────────────────

    fn class_diagram_source() -> &'static str {
        "@startuml\nclass A {\n}\nclass B {\n}\nA --> B\n@enduml"
    }

    fn parse_class(src: &str) -> plantuml_little::model::ClassDiagram {
        match parser::parse(src).expect("parse should succeed") {
            Diagram::Class(cd) => cd,
            other => panic!("expected Class diagram, got {:?}", other),
        }
    }

    // ── Entity count (Java: leafs().size()) ─────────────────────────

    #[test]
    fn parse_class_diagram_has_two_entities() {
        // Java: assertEquals(2, diagram.leafs().size());
        let cd = parse_class(class_diagram_source());
        assert_eq!(cd.entities.len(), 2, "A and B should yield 2 entities");
    }

    #[test]
    fn parse_class_diagram_entity_names() {
        // Java: entity A and entity B found by name
        let cd = parse_class(class_diagram_source());
        let names: Vec<&str> = cd.entities.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.contains(&"A"),
            "should contain entity A, got {:?}",
            names
        );
        assert!(
            names.contains(&"B"),
            "should contain entity B, got {:?}",
            names
        );
    }

    // ── Groups (Java: groups() empty, groupsAndRoot() = 1) ─────────

    #[test]
    fn parse_class_diagram_no_groups() {
        // Java: assertTrue(diagram.groups().isEmpty());
        let cd = parse_class(class_diagram_source());
        assert!(
            cd.groups.is_empty(),
            "plain class diagram should have no groups"
        );
    }

    #[test]
    fn parse_class_diagram_with_package_has_one_group() {
        // Java: groupsAndRoot().size() == 1 for packaged classes
        let src = "@startuml\npackage foo {\n  class A\n}\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.groups.len(), 1, "one package should yield one group");
        assert_eq!(cd.groups[0].name, "foo");
    }

    // ── Links (Java: getLinks().size()) ─────────────────────────────

    #[test]
    fn parse_class_diagram_has_one_link() {
        // Java: assertEquals(1, diagram.getLinks().size());
        let cd = parse_class(class_diagram_source());
        assert_eq!(cd.links.len(), 1, "A --> B should yield exactly 1 link");
    }

    #[test]
    fn parse_class_diagram_link_endpoints() {
        // Java: link connects A to B
        let cd = parse_class(class_diagram_source());
        let link = &cd.links[0];
        assert_eq!(link.from, "A");
        assert_eq!(link.to, "B");
    }

    #[test]
    fn parse_class_diagram_link_style_is_solid() {
        // Java: default link style for --> is solid with arrow head
        let cd = parse_class(class_diagram_source());
        let link = &cd.links[0];
        assert_eq!(link.line_style, plantuml_little::model::LineStyle::Solid);
    }

    #[test]
    fn parse_class_diagram_link_has_arrow_head() {
        // Java: --> produces Arrow head on right side
        let cd = parse_class(class_diagram_source());
        let link = &cd.links[0];
        assert_eq!(link.right_head, plantuml_little::model::ArrowHead::Arrow);
    }

    // ── Empty diagram (Java: leafs() empty for no-body diagram) ────

    #[test]
    fn parse_minimal_diagram_single_entity() {
        // Java: minimal diagram with a single class has exactly one leaf
        let src = "@startuml\nclass Alone\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.entities.len(), 1, "single class should yield one entity");
        assert!(
            cd.links.is_empty(),
            "single class diagram should have no links"
        );
    }

    // ── Entity kind (Java: leafs match expected LeafType) ───────────

    #[test]
    fn parse_interface_entity_kind() {
        // Java: LeafType for 'interface' keyword is INTERFACE
        let src = "@startuml\ninterface Foo\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(
            cd.entities[0].kind,
            plantuml_little::model::EntityKind::Interface
        );
    }

    #[test]
    fn parse_enum_entity_kind() {
        // Java: LeafType for 'enum' keyword is ENUM
        let src = "@startuml\nenum Color\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(
            cd.entities[0].kind,
            plantuml_little::model::EntityKind::Enum
        );
    }

    #[test]
    fn parse_abstract_class_entity_kind() {
        // Java: LeafType for 'abstract class' keyword is ABSTRACT
        let src = "@startuml\nabstract class Base\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(
            cd.entities[0].kind,
            plantuml_little::model::EntityKind::Abstract
        );
    }

    // ── Convert to SVG (Java: SourceStringReader → SVG output) ──────

    #[test]
    fn convert_class_diagram_produces_valid_svg() {
        // Java: SourceStringReader generates SVG that starts with <svg
        let svg = convert(class_diagram_source()).expect("convert should succeed");
        assert!(svg.contains("<svg"), "output must contain <svg tag");
    }

    #[test]
    fn convert_class_diagram_svg_contains_entity_names() {
        // Java: SVG output embeds entity name text
        let svg = convert(class_diagram_source()).expect("convert should succeed");
        assert!(svg.contains("<svg"), "output must contain <svg tag");
        assert!(
            svg.contains(">A<") || svg.contains(">A</"),
            "SVG should contain entity name A"
        );
        assert!(
            svg.contains(">B<") || svg.contains(">B</"),
            "SVG should contain entity name B"
        );
    }

    #[test]
    fn convert_class_diagram_svg_is_well_formed() {
        // Java: SVG output has matching closing tag
        let svg = convert(class_diagram_source()).expect("convert should succeed");
        assert!(svg.contains("<svg"), "must start with svg element");
        assert!(svg.contains("</svg>"), "must have closing </svg> tag");
    }

    #[test]
    fn convert_class_diagram_svg_has_xmlns() {
        // Java: SVG includes xmlns attribute for valid XML
        let svg = convert(class_diagram_source()).expect("convert should succeed");
        assert!(svg.contains("xmlns"), "SVG must declare xmlns namespace");
    }

    #[test]
    fn convert_minimal_diagram_produces_svg() {
        // Java: even minimal diagrams produce valid SVG output
        let src = "@startuml\nclass Alone\n@enduml";
        let svg = convert(src).expect("convert should succeed");
        assert!(svg.contains("<svg"), "minimal diagram must produce SVG");
    }

    #[test]
    fn convert_with_colors_produces_valid_svg() {
        // Java: colored entities generate valid SVG with fill attributes
        let src = "@startuml\nclass A #red\nclass B #blue\nA --> B\n@enduml";
        let svg = convert(src).expect("convert should succeed");
        assert!(svg.contains("<svg"), "colored diagram must produce SVG");
    }

    #[test]
    fn convert_class_with_members_produces_svg_containing_members() {
        // Java: class members appear in SVG output
        let src = "@startuml\nclass Foo {\n  +doSomething()\n  -name: String\n}\n@enduml";
        let svg = convert(src).expect("convert should succeed");
        assert!(svg.contains("<svg"), "output must contain <svg tag");
        assert!(
            svg.contains("doSomething"),
            "SVG should contain method name"
        );
    }

    // ── Multiple links ──────────────────────────────────────────────

    #[test]
    fn parse_diagram_with_multiple_links() {
        // Java: getLinks().size() matches number of relationship lines
        let src = "@startuml\nclass A\nclass B\nclass C\nA --> B\nB --> C\nA ..> C\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.entities.len(), 3);
        assert_eq!(
            cd.links.len(),
            3,
            "three relationship lines should yield 3 links"
        );
    }

    // ── Direction ────────────────────────────────────────────────────

    #[test]
    fn parse_default_direction_is_top_to_bottom() {
        // Java: default layout direction is top-to-bottom
        let cd = parse_class(class_diagram_source());
        assert_eq!(cd.direction, plantuml_little::model::Direction::TopToBottom);
    }

    #[test]
    fn parse_left_to_right_direction() {
        // Java: 'left to right direction' sets horizontal layout
        let src = "@startuml\nleft to right direction\nclass A\nclass B\nA --> B\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.direction, plantuml_little::model::Direction::LeftToRight);
    }

    // ── Count by name (Java: countByName / firstWithName) ───────────

    #[test]
    fn count_entities_by_name() {
        // Java: countByName("A") == 1
        let cd = parse_class(class_diagram_source());
        let count_a = cd.entities.iter().filter(|e| e.name == "A").count();
        let count_b = cd.entities.iter().filter(|e| e.name == "B").count();
        let count_x = cd.entities.iter().filter(|e| e.name == "X").count();
        assert_eq!(count_a, 1, "exactly one entity named A");
        assert_eq!(count_b, 1, "exactly one entity named B");
        assert_eq!(count_x, 0, "no entity named X");
    }

    #[test]
    fn first_with_name_finds_entity() {
        // Java: firstWithName("A") returns non-null
        let cd = parse_class(class_diagram_source());
        let found = cd.entities.iter().find(|e| e.name == "A");
        assert!(found.is_some(), "should find entity A by name");
        assert_eq!(found.unwrap().name, "A");
    }

    // ── has_url equivalent (Java: hasUrl()) ─────────────────────────

    #[test]
    #[ignore = "gap: URL/hyperlink on class entities not yet ported — Java: hasUrl() returns false for plain diagram"]
    fn has_url_returns_false_for_plain_diagram() {
        // Java: assertFalse(diagram.hasUrl());
        todo!()
    }

    // ── Stereotype on entity ────────────────────────────────────────

    #[test]
    fn parse_entity_with_stereotype() {
        // Java: entity stereotype is parsed and retrievable
        let src = "@startuml\nclass Foo <<entity>>\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.entities.len(), 1);
        assert!(
            cd.entities[0]
                .stereotypes
                .iter()
                .any(|s| s.0.contains("entity")),
            "entity should have <<entity>> stereotype, got {:?}",
            cd.entities[0].stereotypes
        );
    }

    // ── Notes on diagram ────────────────────────────────────────────

    #[test]
    fn parse_diagram_with_note() {
        // Java: notes collection is populated when note syntax is used
        let src = "@startuml\nclass A\nnote right of A : a note\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.entities.len(), 1);
        assert!(
            !cd.notes.is_empty(),
            "diagram should have at least one note"
        );
    }

    // ── Dashed link style ───────────────────────────────────────────

    #[test]
    fn parse_dashed_link_style() {
        // Java: A ..> B produces dashed link
        let src = "@startuml\nclass A\nclass B\nA ..> B\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.links.len(), 1);
        assert_eq!(
            cd.links[0].line_style,
            plantuml_little::model::LineStyle::Dashed
        );
    }

    // ── Inheritance link ────────────────────────────────────────────

    #[test]
    fn parse_inheritance_link_has_triangle_head() {
        // Java: A --|> B produces Triangle (inheritance) arrow head
        let src = "@startuml\nclass A\nclass B\nA --|> B\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.links.len(), 1);
        assert_eq!(
            cd.links[0].right_head,
            plantuml_little::model::ArrowHead::Triangle
        );
    }

    // ── Link with label ─────────────────────────────────────────────

    #[test]
    fn parse_link_with_label() {
        // Java: labeled links store the label text
        let src = "@startuml\nclass A\nclass B\nA --> B : uses\n@enduml";
        let cd = parse_class(src);
        assert_eq!(cd.links.len(), 1);
        assert_eq!(
            cd.links[0].label.as_deref(),
            Some("uses"),
            "link label should be 'uses'"
        );
    }

    // ── Convert SVG dimensions ──────────────────────────────────────

    #[test]
    fn convert_svg_has_width_and_height() {
        // Java: SVG output contains viewBox or width/height dimensions
        let svg = convert(class_diagram_source()).expect("convert should succeed");
        assert!(svg.contains("<svg"), "output must contain <svg tag");
        // SVG should have dimensional attributes
        let has_dimensions = svg.contains("width=") || svg.contains("viewBox");
        assert!(
            has_dimensions,
            "SVG should have width/viewBox dimension attributes"
        );
    }

    // ── Hide/show rules ─────────────────────────────────────────────

    #[test]
    #[ignore = "gap: hide empty members not yet ported — Java: isHideEmptyDescriptionForState() returns false"]
    fn hide_empty_description_for_state_default_false() {
        // Java: assertFalse(diagram.isHideEmptyDescriptionForState());
        todo!()
    }

    // ── Unique sequence value ───────────────────────────────────────

    #[test]
    #[ignore = "gap: unique sequence counter not ported — Java: getUniqueSequenceValue() returns specific number"]
    fn get_unique_sequence_value() {
        // Java: long val = diagram.getUniqueSequenceValue();
        //       assertTrue(val > 0);
        todo!()
    }

    // ── cleanId ─────────────────────────────────────────────────────

    #[test]
    #[ignore = "gap: cleanId not ported — Java: cleanId(\"A_B\") returns alphanumeric-only"]
    fn clean_id_strips_non_alphanumeric() {
        // Java: String cleaned = CucaDiagram.cleanId("A_B");
        //       assertTrue(cleaned.matches("[a-zA-Z0-9]+"));
        todo!()
    }

    // ── Label distance / angle defaults ─────────────────────────────

    #[test]
    #[ignore = "gap: label distance not ported — Java: getLabeldistance() == \"1.7\""]
    fn label_distance_default() {
        // Java: assertEquals("1.7", diagram.getLabeldistance());
        todo!()
    }

    #[test]
    #[ignore = "gap: label angle not ported — Java: getLabelangle() == \"25\""]
    fn label_angle_default() {
        // Java: assertEquals("25", diagram.getLabelangle());
        todo!()
    }
}
