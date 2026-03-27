// Port of Java PlantUML `klimt.drawing.svg` package tests to Rust.
// Java source: SvgGraphicsSkeletonTest, UGraphicSvgSkeletonTest, DriverXxxSvg tests
//
// Tests exercise SvgGraphic (low-level SVG emission), UGraphicSvg (high-level
// UGraphic trait impl), and utility functions (fmt_coord, xml_escape, LengthAdjust).

// ═══════════════════════════════════════════════════════════════════════
// Module 1: svg_graphic_tests — SvgGraphic direct API
// ═══════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod svg_graphic_tests {
    use plantuml_little::klimt::shape::UPath;
    use plantuml_little::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};

    /// Standard seed and scale for all tests in this module.
    fn make() -> SvgGraphic {
        SvgGraphic::new(42, 1.0)
    }

    // ── 1. body() is not empty after drawing ─────────────────────────

    #[test]
    fn body_not_empty_after_rectangle() {
        let mut svg = make();
        svg.set_fill_color("#CCCCCC");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        assert!(
            !svg.body().is_empty(),
            "body should not be empty after drawing a rectangle"
        );
    }

    // ── 2. svgEllipse with fill/stroke produces ellipse element ──────

    #[test]
    fn ellipse_with_fill_and_stroke() {
        let mut svg = make();
        svg.set_fill_color("#FEFECE");
        svg.set_stroke_color(Some("#A80036"));
        svg.set_stroke_width(1.5, None);
        svg.svg_ellipse(60.0, 40.0, 20.0, 15.0, 0.0);
        let body = svg.body();
        assert!(body.contains("<ellipse"), "should contain <ellipse element");
        assert!(
            body.contains("fill=\"#FEFECE\""),
            "fill color should appear"
        );
        assert!(
            body.contains("stroke:#A80036"),
            "stroke color should appear in style"
        );
    }

    // ── 3. svgArcEllipse produces SVG content ────────────────────────

    #[test]
    fn arc_ellipse_produces_path() {
        let mut svg = make();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_arc_ellipse(20.0, 15.0, 10.0, 30.0, 50.0, 30.0);
        let body = svg.body();
        assert!(
            !body.is_empty(),
            "body should have content after arc ellipse"
        );
        assert!(
            body.contains("<path"),
            "arc ellipse produces a <path element"
        );
    }

    // ── 4. createSvgGradient: non-empty, idempotent, policy-sensitive ─

    #[test]
    fn gradient_returns_non_empty_id() {
        let mut svg = make();
        let id = svg.create_svg_gradient("#FF0000", "#0000FF", '/');
        assert!(!id.is_empty(), "gradient id should not be empty");
    }

    #[test]
    fn gradient_same_params_same_id() {
        let mut svg = make();
        let id1 = svg.create_svg_gradient("#FF0000", "#0000FF", '/');
        let id2 = svg.create_svg_gradient("#FF0000", "#0000FF", '/');
        assert_eq!(id1, id2, "same gradient params should return same id");
    }

    #[test]
    fn gradient_different_policy_different_id() {
        let mut svg = make();
        let id_pipe = svg.create_svg_gradient("#FF0000", "#0000FF", '|');
        let id_dash = svg.create_svg_gradient("#FF0000", "#0000FF", '-');
        assert_ne!(
            id_pipe, id_dash,
            "different policy should produce different ids"
        );
    }

    #[test]
    fn gradient_different_colors_different_id() {
        let mut svg = make();
        let id1 = svg.create_svg_gradient("#FF0000", "#0000FF", '/');
        let id2 = svg.create_svg_gradient("#00FF00", "#0000FF", '/');
        assert_ne!(id1, id2, "different colors should produce different ids");
    }

    #[test]
    fn gradient_defs_contain_linear_gradient() {
        let mut svg = make();
        let _id = svg.create_svg_gradient("#112233", "#445566", '|');
        let defs = svg.defs();
        assert!(
            defs.contains("<linearGradient"),
            "defs should contain linearGradient"
        );
        assert!(defs.contains("stop-color=\"#112233\""), "first stop-color");
        assert!(defs.contains("stop-color=\"#445566\""), "second stop-color");
    }

    #[test]
    fn gradient_horizontal_policy_x_attrs() {
        let mut svg = make();
        let _id = svg.create_svg_gradient("#AA", "#BB", '|');
        let defs = svg.defs();
        // Horizontal: x1=0%, y1=50%, x2=100%, y2=50%
        assert!(defs.contains("x1=\"0%\""));
        assert!(defs.contains("x2=\"100%\""));
        assert!(defs.contains("y1=\"50%\""));
        assert!(defs.contains("y2=\"50%\""));
    }

    #[test]
    fn gradient_vertical_policy_y_attrs() {
        let mut svg = make();
        let _id = svg.create_svg_gradient("#AA", "#BB", '-');
        let defs = svg.defs();
        // Vertical: x1=50%, y1=0%, x2=50%, y2=100%
        assert!(defs.contains("x1=\"50%\""));
        assert!(defs.contains("x2=\"50%\""));
        assert!(defs.contains("y1=\"0%\""));
        assert!(defs.contains("y2=\"100%\""));
    }

    // ── 5. setFillColor + svgRectangle → rect in output ──────────────

    #[test]
    fn rectangle_with_fill() {
        let mut svg = make();
        svg.set_fill_color("#E2E2F0");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(10.0, 20.0, 80.0, 40.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(body.contains("<rect"), "should contain <rect");
        assert!(body.contains("fill=\"#E2E2F0\""));
    }

    // ── 6. setStrokeColor + svgLine → line in output ─────────────────

    #[test]
    fn line_with_stroke() {
        let mut svg = make();
        svg.set_stroke_color(Some("#333333"));
        svg.set_stroke_width(2.0, None);
        svg.svg_line(10.0, 10.0, 90.0, 90.0, 0.0);
        let body = svg.body();
        assert!(body.contains("<line"), "should contain <line");
        assert!(body.contains("stroke:#333333"), "stroke color in style");
    }

    // ── 7. setStrokeWidth with dasharray ─────────────────────────────

    #[test]
    fn stroke_with_dasharray() {
        let mut svg = make();
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, Some((5.0, 3.0)));
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(
            body.contains("stroke-dasharray:5,3;"),
            "dasharray should appear in style"
        );
    }

    // ── 8. svgRectangle with rounded corners ─────────────────────────

    #[test]
    fn rectangle_with_rounded_corners() {
        let mut svg = make();
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(5.0, 5.0, 80.0, 40.0, 5.0, 5.0, 0.0);
        let body = svg.body();
        assert!(
            body.contains("rx=\"5\""),
            "rx attribute for rounded corners"
        );
        assert!(
            body.contains("ry=\"5\""),
            "ry attribute for rounded corners"
        );
    }

    #[test]
    fn rectangle_without_rounded_corners_omits_rx_ry() {
        let mut svg = make();
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 80.0, 40.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(!body.contains("rx="), "no rx when corners are zero");
        assert!(!body.contains("ry="), "no ry when corners are zero");
    }

    // ── 9. svgPolygon produces polygon ───────────────────────────────

    #[test]
    fn polygon_basic() {
        let mut svg = make();
        svg.set_fill_color("#181818");
        svg.set_stroke_color(Some("#181818"));
        svg.set_stroke_width(1.0, None);
        let points = [10.0, 20.0, 30.0, 40.0, 50.0, 20.0];
        svg.svg_polygon(0.0, &points);
        let body = svg.body();
        assert!(body.contains("<polygon"), "should contain <polygon");
        assert!(body.contains("points=\""), "should have points attribute");
    }

    // ── 10. text produces text element ───────────────────────────────

    #[test]
    fn text_basic() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "Hello World",
            10.0,
            25.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            60.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("<text"), "should contain <text element");
        assert!(body.contains(">Hello World</text>"), "text content");
    }

    #[test]
    fn text_with_bold_and_italic() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "styled",
            0.0,
            0.0,
            Some("serif"),
            16.0,
            Some("bold"),
            Some("italic"),
            None,
            40.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("font-weight=\"bold\""));
        assert!(body.contains("font-style=\"italic\""));
    }

    #[test]
    fn text_with_underline() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "underlined",
            0.0,
            0.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            Some("underline"),
            60.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(body.contains("text-decoration=\"underline\""));
    }

    #[test]
    fn text_with_text_anchor() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "centered",
            50.0,
            25.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            50.0,
            LengthAdjust::None,
            None,
            0,
            Some("middle"),
        );
        let body = svg.body();
        assert!(body.contains("text-anchor=\"middle\""));
    }

    #[test]
    fn text_with_rotation_90() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "rotated",
            100.0,
            50.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            40.0,
            LengthAdjust::None,
            None,
            90,
            None,
        );
        let body = svg.body();
        assert!(
            body.contains("transform=\"rotate(-90 100 50)\""),
            "90-degree orientation emits rotate(-90)"
        );
    }

    #[test]
    fn text_with_rotation_270() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "rotated",
            100.0,
            50.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            40.0,
            LengthAdjust::None,
            None,
            270,
            None,
        );
        let body = svg.body();
        assert!(
            body.contains("transform=\"rotate(90 100 50)\""),
            "270-degree orientation emits rotate(90)"
        );
    }

    #[test]
    fn text_monospaced_becomes_monospace() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "code",
            0.0,
            0.0,
            Some("monospaced"),
            13.0,
            None,
            None,
            None,
            30.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        let body = svg.body();
        assert!(
            body.contains("font-family=\"monospace\""),
            "monospaced should be normalized to monospace"
        );
    }

    // ── 11. svgPath with UPath ───────────────────────────────────────

    #[test]
    fn path_with_moveto_lineto_close() {
        let mut svg = make();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);

        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(100.0, 0.0);
        path.line_to(100.0, 50.0);
        path.line_to(0.0, 50.0);
        path.close();

        svg.svg_path(0.0, 0.0, &path, 0.0);
        let body = svg.body();
        assert!(body.contains("<path"), "should contain <path element");
        assert!(body.contains("d=\""), "should have d attribute");
        assert!(body.contains("M0,0"), "path starts with MoveTo");
        assert!(body.contains("L100,0"), "path contains LineTo");
    }

    #[test]
    fn path_with_cubic_to() {
        let mut svg = make();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);

        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.cubic_to(10.0, 0.0, 20.0, 10.0, 20.0, 20.0);

        svg.svg_path(5.0, 5.0, &path, 0.0);
        let body = svg.body();
        assert!(body.contains("<path"));
        assert!(body.contains("C"), "path data should contain Cubic command");
    }

    #[test]
    fn path_with_arc_to() {
        let mut svg = make();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);

        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.arc_to(25.0, 25.0, 0.0, 0.0, 1.0, 50.0, 50.0);

        svg.svg_path(0.0, 0.0, &path, 0.0);
        let body = svg.body();
        assert!(
            body.contains("A25,25"),
            "path data should contain Arc command"
        );
    }

    // ── 12-14. Path builder API (newpath/moveto/lineto/closepath/fill, curveto, quadto) ──
    // These Java methods may not be ported to Rust SvgGraphic.

    #[test]
    #[ignore = "gap: Java SvgGraphics.newpath/moveto/lineto/closepath/fill path builder not ported; use svg_path(UPath) instead"]
    fn path_builder_newpath_moveto_lineto_close_fill() {
        // Java: sg.newpath(); sg.moveto(0,0); sg.lineto(100,0); sg.closepath(); sg.fill(2);
        // Produces <path> with M/L/Z path data
        todo!()
    }

    #[test]
    #[ignore = "gap: Java SvgGraphics.curveto path builder not ported; use UPath.cubic_to() + svg_path() instead"]
    fn path_builder_curveto() {
        // Java: sg.newpath(); sg.moveto(0,0); sg.curveto(cp1x,cp1y,cp2x,cp2y,x,y); sg.fill(0);
        todo!()
    }

    #[test]
    #[ignore = "gap: Java SvgGraphics.quadto path builder not ported; no quad-to in Rust UPath either"]
    fn path_builder_quadto() {
        // Java: sg.newpath(); sg.moveto(0,0); sg.quadto(cpx,cpy,x,y); sg.fill(0);
        todo!()
    }

    // ── 15. setHidden suppresses output ──────────────────────────────

    #[test]
    fn hidden_suppresses_rectangle() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        assert!(
            svg.body().is_empty(),
            "hidden rectangle should not appear in body"
        );
    }

    #[test]
    fn hidden_suppresses_ellipse() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_ellipse(50.0, 50.0, 25.0, 25.0, 0.0);
        assert!(
            svg.body().is_empty(),
            "hidden ellipse should not appear in body"
        );
    }

    #[test]
    fn hidden_suppresses_line() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_line(0.0, 0.0, 100.0, 100.0, 0.0);
        assert!(
            svg.body().is_empty(),
            "hidden line should not appear in body"
        );
    }

    #[test]
    fn hidden_suppresses_polygon() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_polygon(0.0, &[10.0, 20.0, 30.0, 40.0, 50.0, 20.0]);
        assert!(
            svg.body().is_empty(),
            "hidden polygon should not appear in body"
        );
    }

    #[test]
    fn hidden_suppresses_path() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(50.0, 50.0);
        svg.svg_path(0.0, 0.0, &path, 0.0);
        assert!(
            svg.body().is_empty(),
            "hidden path should not appear in body"
        );
    }

    #[test]
    fn hidden_suppresses_text() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_fill_color("#000000");
        svg.svg_text(
            "invisible",
            10.0,
            20.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            50.0,
            LengthAdjust::Spacing,
            None,
            0,
            None,
        );
        assert!(
            !svg.body().contains("<text"),
            "hidden text should not produce <text"
        );
    }

    #[test]
    fn hidden_still_tracks_max_extents() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 200.0, 150.0, 0.0, 0.0, 0.0);
        assert!(svg.max_x() >= 200, "hidden rect should still update max_x");
        assert!(svg.max_y() >= 150, "hidden rect should still update max_y");
    }

    #[test]
    fn unhide_resumes_output() {
        let mut svg = make();
        svg.set_hidden(true);
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        assert!(svg.body().is_empty());

        svg.set_hidden(false);
        svg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        assert!(
            svg.body().contains("<rect"),
            "after unhiding, rect should appear"
        );
    }

    // ── 16. addComment produces comment in output ────────────────────

    #[test]
    fn comment_in_body() {
        let mut svg = make();
        svg.add_comment("test comment");
        let body = svg.body();
        assert!(body.contains("<!--"), "should contain XML comment opening");
        assert!(body.contains("test comment"), "comment text should appear");
        assert!(body.contains("-->"), "should contain XML comment closing");
    }

    // ── 17. openLink/closeLink produces <a> element ──────────────────

    #[test]
    fn link_produces_anchor() {
        let mut svg = make();
        svg.open_link("http://example.com", Some("Tooltip"), "_blank");
        svg.set_fill_color("#0000FF");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        svg.close_link();
        let body = svg.body();
        assert!(body.contains("<a"), "should contain <a element");
        assert!(
            body.contains("href=\"http://example.com\""),
            "href attribute"
        );
        assert!(body.contains("target=\"_blank\""), "target attribute");
        assert!(body.contains("title=\"Tooltip\""), "title from tooltip");
        assert!(body.contains("</a>"), "closing </a> tag");
    }

    #[test]
    fn link_without_tooltip_uses_url_as_title() {
        let mut svg = make();
        svg.open_link("http://example.com", None, "_top");
        svg.close_link();
        let body = svg.body();
        assert!(
            body.contains("title=\"http://example.com\""),
            "when tooltip is None, url should be used as title"
        );
    }

    #[test]
    fn link_xlink_attributes() {
        let mut svg = make();
        svg.open_link("http://test.org", Some("tip"), "_blank");
        svg.close_link();
        let body = svg.body();
        assert!(
            body.contains("xlink:href=\"http://test.org\""),
            "xlink:href"
        );
        assert!(body.contains("xlink:title=\"tip\""), "xlink:title");
        assert!(body.contains("xlink:type=\"simple\""), "xlink:type");
    }

    // ── 18. startGroup/closeGroup produces <g> element ───────────────

    #[test]
    fn group_produces_g_element() {
        let mut svg = make();
        svg.start_group(&[("id", "grp1"), ("class", "myclass")]);
        svg.close_group();
        let body = svg.body();
        assert!(body.contains("<g"), "should contain <g element");
        assert!(body.contains("id=\"grp1\""), "id attribute");
        assert!(body.contains("class=\"myclass\""), "class attribute");
        assert!(body.contains("</g>"), "closing </g>");
    }

    #[test]
    fn group_empty_attrs() {
        let mut svg = make();
        svg.start_group(&[]);
        svg.close_group();
        let body = svg.body();
        assert!(body.contains("<g>"), "empty attrs should produce bare <g>");
        assert!(body.contains("</g>"));
    }

    #[test]
    fn nested_groups() {
        let mut svg = make();
        svg.start_group(&[("id", "outer")]);
        svg.start_group(&[("id", "inner")]);
        svg.close_group();
        svg.close_group();
        let body = svg.body();
        // Count <g and </g> occurrences
        let open_count = body.matches("<g").count();
        let close_count = body.matches("</g>").count();
        assert_eq!(open_count, 2, "two nested <g>");
        assert_eq!(close_count, 2, "two closing </g>");
    }

    // ── 19. to_svg produces valid SVG document ───────────────────────

    #[test]
    fn to_svg_produces_svg_document() {
        let mut svg = make();
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(10.0, 10.0, 80.0, 40.0, 0.0, 0.0, 0.0);

        let doc = svg.to_svg(None, "SEQUENCE");
        assert!(doc.starts_with("<svg"), "document should start with <svg");
        assert!(
            doc.contains("xmlns=\"http://www.w3.org/2000/svg\""),
            "SVG namespace"
        );
        assert!(doc.contains("xmlns:xlink="), "xlink namespace");
        assert!(doc.contains("version=\"1.1\""), "SVG version");
        assert!(doc.ends_with("</svg>"), "should end with </svg>");
    }

    #[test]
    fn to_svg_contains_body_content() {
        let mut svg = make();
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        let doc = svg.to_svg(None, "");
        assert!(
            doc.contains("<rect"),
            "full SVG document should contain the rect"
        );
    }

    #[test]
    fn to_svg_with_background_color() {
        let svg = make();
        let doc = svg.to_svg(Some("#FFFF00"), "");
        assert!(
            doc.contains("background:#FFFF00"),
            "background color in style"
        );
    }

    #[test]
    fn to_svg_transparent_bg_omitted() {
        let svg = make();
        let doc = svg.to_svg(Some("#00000000"), "");
        assert!(
            !doc.contains("background:"),
            "transparent bg should not appear"
        );
    }

    #[test]
    fn to_svg_with_diagram_type() {
        let svg = make();
        let doc = svg.to_svg(None, "CLASS");
        assert!(doc.contains("data-diagram-type=\"CLASS\""));
    }

    #[test]
    fn to_svg_empty_diagram_type_omitted() {
        let svg = make();
        let doc = svg.to_svg(None, "");
        assert!(
            !doc.contains("data-diagram-type"),
            "empty type should be omitted"
        );
    }

    #[test]
    fn to_svg_includes_defs_when_present() {
        let mut svg = make();
        let _id = svg.create_svg_gradient("#FF0000", "#0000FF", '/');
        let doc = svg.to_svg(None, "");
        assert!(doc.contains("<defs>"), "should include <defs> block");
        assert!(doc.contains("</defs>"), "should close </defs>");
        assert!(doc.contains("<linearGradient"), "gradient in defs");
    }

    #[test]
    fn to_svg_omits_defs_when_empty() {
        let svg = make();
        let doc = svg.to_svg(None, "");
        assert!(!doc.contains("<defs>"), "no defs block when empty");
    }

    #[test]
    fn to_svg_contains_plantuml_pi() {
        let svg = make();
        let doc = svg.to_svg(None, "");
        assert!(
            doc.contains("<?plantuml"),
            "should contain PlantUML processing instruction"
        );
    }

    // ── Shadow filter in defs ────────────────────────────────────────

    #[test]
    fn shadow_creates_filter_in_defs() {
        let mut svg = make();
        svg.set_fill_color("#FFFFFF");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(10.0, 10.0, 80.0, 40.0, 0.0, 0.0, 3.0);
        let defs = svg.defs();
        assert!(defs.contains("<filter"), "shadow should create filter");
        assert!(
            defs.contains("feGaussianBlur"),
            "Gaussian blur in shadow filter"
        );
        let body = svg.body();
        assert!(
            body.contains("filter=\"url(#"),
            "rect should reference shadow filter"
        );
    }

    // ── Circle ───────────────────────────────────────────────────────

    #[test]
    fn circle_basic() {
        let mut svg = make();
        svg.set_fill_color("#AABBCC");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_circle(50.0, 50.0, 20.0, 0.0);
        let body = svg.body();
        assert!(body.contains("<circle"), "should contain <circle element");
        assert!(body.contains("cx=\"50\""));
        assert!(body.contains("cy=\"50\""));
        assert!(body.contains("r=\"20\""));
    }

    // ── Polyline ─────────────────────────────────────────────────────

    #[test]
    fn polyline_basic() {
        let mut svg = make();
        svg.set_fill_color("none");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_polyline(&[10.0, 20.0, 30.0, 40.0, 50.0, 20.0]);
        let body = svg.body();
        assert!(
            body.contains("<polyline"),
            "should contain <polyline element"
        );
        assert!(body.contains("points=\""), "should have points attribute");
    }

    // ── Accessors ────────────────────────────────────────────────────

    #[test]
    fn scale_accessor() {
        let svg = SvgGraphic::new(0, 2.0);
        assert!(
            (svg.scale() - 2.0).abs() < f64::EPSILON,
            "scale() returns construction scale"
        );
    }

    #[test]
    fn set_max_overrides() {
        let mut svg = make();
        svg.set_max(500, 300);
        assert_eq!(svg.max_x(), 500);
        assert_eq!(svg.max_y(), 300);
    }

    #[test]
    fn initial_max_is_small() {
        let svg = make();
        assert!(svg.max_x() <= 10, "initial max_x should be small");
        assert!(svg.max_y() <= 10, "initial max_y should be small");
    }

    // ── push_raw / push_raw_defs ─────────────────────────────────────

    #[test]
    fn push_raw_appends_to_body() {
        let mut svg = make();
        svg.push_raw("<custom-element/>");
        assert!(svg.body().contains("<custom-element/>"));
    }

    #[test]
    fn push_raw_defs_appends_to_defs() {
        let mut svg = make();
        svg.push_raw_defs("<marker id=\"m1\"/>");
        assert!(svg.defs().contains("<marker id=\"m1\"/>"));
    }

    // ── Fill opacity for #RRGGBBAA format ────────────────────────────

    #[test]
    fn fill_with_alpha_channel() {
        let mut svg = make();
        svg.set_fill_color_with_opacity("#FF000080");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(body.contains("fill=\"#FF0000\""), "color without alpha");
        assert!(body.contains("fill-opacity="), "opacity attribute present");
    }

    // ── Transparent fill via #00000000 ───────────────────────────────

    #[test]
    fn transparent_fill_becomes_none() {
        let mut svg = make();
        svg.set_fill_color("#00000000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(
            body.contains("fill=\"none\""),
            "#00000000 should become fill=none"
        );
    }

    // ── Stroke width zero suppresses style ───────────────────────────

    #[test]
    fn stroke_width_zero_omits_style() {
        let mut svg = make();
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(0.0, None);
        svg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        let body = svg.body();
        assert!(
            !body.contains("style="),
            "stroke-width 0 should not emit style attribute"
        );
    }

    // ── Zero-size rectangle skipped ──────────────────────────────────

    #[test]
    fn zero_height_rectangle_skipped() {
        let mut svg = make();
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0);
        assert!(svg.body().is_empty(), "zero-height rect should be skipped");
    }

    #[test]
    fn zero_width_rectangle_skipped() {
        let mut svg = make();
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(0.0, 0.0, 0.0, 50.0, 0.0, 0.0, 0.0);
        assert!(svg.body().is_empty(), "zero-width rect should be skipped");
    }

    // ── Utility functions ────────────────────────────────────────────

    #[test]
    fn fmt_coord_integer() {
        assert_eq!(fmt_coord(42.0), "42");
    }

    #[test]
    fn fmt_coord_zero() {
        assert_eq!(fmt_coord(0.0), "0");
    }

    #[test]
    fn fmt_coord_decimal() {
        let s = fmt_coord(1.5);
        assert_eq!(s, "1.5");
    }

    #[test]
    fn fmt_coord_trailing_zeros_stripped() {
        let s = fmt_coord(3.10);
        assert_eq!(s, "3.1");
    }

    #[test]
    fn xml_escape_ampersand() {
        assert!(xml_escape("a&b").contains("&amp;"));
    }

    #[test]
    fn xml_escape_lt_gt() {
        let escaped = xml_escape("<tag>");
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
    }

    #[test]
    fn xml_escape_quotes() {
        assert!(xml_escape("say \"hi\"").contains("&quot;"));
    }

    #[test]
    fn xml_escape_non_ascii() {
        let escaped = xml_escape("\u{00E9}"); // e-acute
        assert!(escaped.contains("&#233;"));
    }

    #[test]
    fn xml_escape_plain_text_unchanged() {
        assert_eq!(xml_escape("hello"), "hello");
    }

    // ── LengthAdjust ────────────────────────────────────────────────

    #[test]
    fn length_adjust_default_is_spacing() {
        let la = LengthAdjust::default();
        assert_eq!(la, LengthAdjust::Spacing);
    }

    #[test]
    fn length_adjust_variants_differ() {
        assert_ne!(LengthAdjust::Spacing, LengthAdjust::SpacingAndGlyphs);
        assert_ne!(LengthAdjust::Spacing, LengthAdjust::None);
        assert_ne!(LengthAdjust::SpacingAndGlyphs, LengthAdjust::None);
    }

    // ── Text back color filter ──────────────────────────────────────

    #[test]
    fn text_back_color_creates_filter() {
        let mut svg = make();
        svg.set_fill_color("#000000");
        svg.svg_text(
            "highlighted",
            0.0,
            0.0,
            Some("sans-serif"),
            14.0,
            None,
            None,
            None,
            50.0,
            LengthAdjust::Spacing,
            Some("#FFFF00"),
            0,
            None,
        );
        let body = svg.body();
        assert!(
            body.contains("filter=\"url(#"),
            "text should reference back-color filter"
        );
        let defs = svg.defs();
        assert!(
            defs.contains("flood-color=\"#FFFF00\""),
            "filter defs should have flood-color"
        );
    }

    // ── Ensure visible updates extents after shapes ──────────────────

    #[test]
    fn extents_grow_with_shapes() {
        let mut svg = make();
        let initial_x = svg.max_x();
        let initial_y = svg.max_y();
        svg.set_fill_color("#FF0000");
        svg.set_stroke_color(Some("#000000"));
        svg.set_stroke_width(1.0, None);
        svg.svg_rectangle(200.0, 300.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        assert!(
            svg.max_x() > initial_x,
            "max_x should grow after large rect"
        );
        assert!(
            svg.max_y() > initial_y,
            "max_y should grow after large rect"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Module 2: ugraphic_svg_tests — UGraphicSvg (UGraphic trait impl)
// ═══════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod ugraphic_svg_tests {
    use plantuml_little::klimt::font::DefaultStringBounder;
    use plantuml_little::klimt::svg::{LengthAdjust, SvgGraphic, UGraphicSvg};
    use plantuml_little::klimt::{UGraphic, UGroup, UGroupType};

    fn make_ug() -> UGraphicSvg {
        let svg = SvgGraphic::new(99, 1.0);
        UGraphicSvg::new(svg, Box::new(DefaultStringBounder), LengthAdjust::Spacing)
    }

    #[test]
    fn ugraphic_svg_constructs() {
        let ug = make_ug();
        assert!(!ug.param().hidden, "default param should not be hidden");
    }

    #[test]
    fn ugraphic_draw_rect_produces_rect() {
        let mut ug = make_ug();
        ug.draw_rect(100.0, 50.0, 0.0);
        let body = ug.svg().body();
        assert!(
            body.contains("<rect"),
            "draw_rect should produce <rect in SVG body"
        );
    }

    #[test]
    fn ugraphic_draw_line_produces_line() {
        let mut ug = make_ug();
        ug.draw_line(100.0, 0.0);
        let body = ug.svg().body();
        assert!(
            body.contains("<line"),
            "draw_line should produce <line in SVG body"
        );
    }

    #[test]
    fn ugraphic_draw_ellipse_produces_ellipse() {
        let mut ug = make_ug();
        ug.draw_ellipse(80.0, 60.0);
        let body = ug.svg().body();
        assert!(
            body.contains("<ellipse"),
            "draw_ellipse should produce <ellipse"
        );
    }

    #[test]
    fn ugraphic_draw_text_produces_text() {
        let mut ug = make_ug();
        ug.draw_text("Hello", "SansSerif", 14.0, false, false);
        let body = ug.svg().body();
        assert!(body.contains("<text"), "draw_text should produce <text");
        assert!(body.contains("Hello"), "text content should appear");
    }

    #[test]
    fn ugraphic_draw_polygon_produces_polygon() {
        let mut ug = make_ug();
        ug.draw_polygon(&[(10.0, 20.0), (30.0, 40.0), (50.0, 20.0)]);
        let body = ug.svg().body();
        assert!(
            body.contains("<polygon"),
            "draw_polygon should produce <polygon"
        );
    }

    #[test]
    fn ugraphic_draw_path_produces_path() {
        use plantuml_little::klimt::shape::UPath;
        let mut ug = make_ug();
        let mut path = UPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(50.0, 50.0);
        ug.draw_path(&path);
        let body = ug.svg().body();
        assert!(body.contains("<path"), "draw_path should produce <path");
    }

    #[test]
    fn ugraphic_start_group_close_group() {
        let mut ug = make_ug();
        let group = UGroup::singleton(UGroupType::Id, "test-group");
        ug.start_group(&group);
        ug.close_group();
        let body = ug.svg().body();
        assert!(body.contains("<g"), "start_group should produce <g");
        assert!(body.contains("id=\"test-group\""), "group id attribute");
        assert!(body.contains("</g>"), "close_group should produce </g>");
    }

    #[test]
    fn ugraphic_start_url_close_url() {
        let mut ug = make_ug();
        ug.start_url("http://example.com", "Example");
        ug.close_url();
        let body = ug.svg().body();
        assert!(body.contains("<a"), "start_url should produce <a");
        assert!(body.contains("</a>"), "close_url should produce </a>");
    }

    #[test]
    fn ugraphic_into_svg_returns_graphic() {
        let mut ug = make_ug();
        ug.draw_rect(50.0, 30.0, 0.0);
        let svg = ug.into_svg();
        assert!(
            svg.body().contains("<rect"),
            "into_svg should preserve drawn content"
        );
    }

    #[test]
    #[ignore = "gap: Java UGraphicSvg.matchesProperty(\"SVG\") not yet ported as a method on UGraphicSvg"]
    fn ugraphic_matches_property_svg() {
        // Java: ugSvg.matchesProperty("SVG") returns true
        todo!()
    }

    #[test]
    #[ignore = "gap: Java UGraphicSvg.dpiFactor() not ported; Java returns 1.0"]
    fn ugraphic_dpi_factor() {
        // Java: ugSvg.dpiFactor() == 1.0
        todo!()
    }

    #[test]
    #[ignore = "gap: Java UGraphicSvg.writeToStream() not ported; Java writes SVG bytes to OutputStream"]
    fn ugraphic_write_to_stream() {
        // Java: ugSvg.writeToStream(baos); baos.toString() contains <svg
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Module 3: svg_data_tests — SvgData (if it existed)
// ═══════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod svg_data_tests {
    #[test]
    #[ignore = "gap: Java SvgData class not ported to Rust (encapsulates SVG string + dimensions)"]
    fn svg_data_basic() {
        // Java: SvgData wraps SVG output string, provides getWidth/getHeight
        todo!()
    }

    #[test]
    #[ignore = "gap: Java SvgData.appendSvg not ported"]
    fn svg_data_append() {
        // Java: SvgData.appendSvg(StringBuilder) appends content
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Module 4: driver_tests — Driver*Svg (individual shape drivers)
// ═══════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod driver_tests {
    // In Java PlantUML, each shape type has a dedicated DriverXxxSvg class
    // (DriverRectangleSvg, DriverEllipseSvg, DriverLineSvg, etc.) that takes
    // a UShape + UParam + SvgGraphics and emits SVG.
    //
    // In Rust, these are folded into UGraphicSvg's draw_* methods and
    // SvgGraphic's svg_* methods — no separate Driver types exist.
    // Tests for the actual rendering behavior are covered by modules 1-2.

    #[test]
    #[ignore = "gap: Java DriverRectangleSvg not ported as separate type; covered by SvgGraphic.svg_rectangle"]
    fn driver_rectangle_svg() {
        // Java: new DriverRectangleSvg().draw(shape, x, y, colorMapper, param, svgGraphics)
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverEllipseSvg not ported as separate type; covered by SvgGraphic.svg_ellipse"]
    fn driver_ellipse_svg() {
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverLineSvg not ported as separate type; covered by SvgGraphic.svg_line"]
    fn driver_line_svg() {
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverTextSvg not ported as separate type; covered by SvgGraphic.svg_text"]
    fn driver_text_svg() {
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverPolygonSvg not ported as separate type; covered by SvgGraphic.svg_polygon"]
    fn driver_polygon_svg() {
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverPathSvg not ported as separate type; covered by SvgGraphic.svg_path"]
    fn driver_path_svg() {
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverDotPathSvg not ported as separate type"]
    fn driver_dot_path_svg() {
        // Java: DriverDotPathSvg renders DotPath as cubic curves
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverImageSvg not ported as separate type"]
    fn driver_image_svg() {
        // Java: DriverImageSvg renders UImage as base64 <image> element
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverImageSvgSvg not ported as separate type"]
    fn driver_image_svg_svg() {
        // Java: DriverImageSvgSvg renders UImageSvg as inline SVG
        todo!()
    }

    #[test]
    #[ignore = "gap: Java DriverCenteredCharacterSvg not ported as separate type"]
    fn driver_centered_character_svg() {
        // Java: DriverCenteredCharacterSvg renders UCenteredCharacter as centered <text>
        todo!()
    }
}
