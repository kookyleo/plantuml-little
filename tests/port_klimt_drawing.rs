// Port of Java PlantUML klimt.drawing package tests to Rust.
//
// The primary Java class under test is `LimitFinder`, which corresponds to
// Rust's `BoundsTracker` in render/svg.rs.  Because BoundsTracker is
// pub(crate), integration tests cannot instantiate it directly.  Those tests
// are written as #[ignore] TDD anchors documenting the exact Java behavior.
//
// Tests for publicly accessible types (UStroke, UTranslate, UClip, SvgGraphic,
// etc.) exercise the API directly.

// ── Imports for public types ────────────────────────────────────────────────

use plantuml_little::klimt::svg::{fmt_coord, xml_escape, LengthAdjust, SvgGraphic};
use plantuml_little::klimt::{UClip, UStroke, UTranslate};

// ═══════════════════════════════════════════════════════════════════════════
// 1. LimitFinder (Java) -> BoundsTracker (Rust)  — pub(crate), all #[ignore]
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod limit_finder_tests {
    //! Ports of LimitFinderTest.java from net.sourceforge.plantuml.klimt.drawing.
    //!
    //! BoundsTracker is pub(crate) in render/svg.rs and cannot be constructed
    //! from an integration test.  Each test documents the exact Java LimitFinder
    //! behavior with expected min/max bounds.

    // ── drawRectangle ──────────────────────────────────────────────

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_rectangle_origin() {
        // Java: LimitFinder.drawRectangle(0, 0, 100, 50)
        //   addPoint(x-1, y-1) = (-1, -1)
        //   addPoint(x+w-1, y+h-1) = (99, 49)
        // Expected: min=(-1,-1), max=(99,49)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_rectangle_with_translate() {
        // Java: translate(10, 20) then drawRectangle(0, 0, 100, 50)
        //   addPoint(10-1, 20-1) = (9, 19)
        //   addPoint(10+100-1, 20+50-1) = (109, 69)
        // Expected: min=(9,19), max=(109,69)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_rectangle_negative_coords() {
        // Java: drawRectangle(-10, -20, 100, 50)
        //   addPoint(-10-1, -20-1) = (-11, -21)
        //   addPoint(-10+100-1, -20+50-1) = (89, 29)
        // Expected: min=(-11,-21), max=(89,29)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_rectangle_fractional() {
        // Java: drawRectangle(0.5, 0.5, 100.0, 50.0)
        //   addPoint(-0.5, -0.5)
        //   addPoint(99.5, 49.5)
        // Expected: min=(-0.5,-0.5), max=(99.5,49.5)
        todo!()
    }

    // ── drawULine ──────────────────────────────────────────────────

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_line_origin() {
        // Java: LimitFinder.drawULine(0, 0, 50, 30)
        //   addPoint(0, 0)
        //   addPoint(50, 30)
        // Expected: min=(0,0), max=(50,30)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_line_with_translate() {
        // Java: translate(5, 10) then drawULine(0, 0, 50, 30)
        //   addPoint(5, 10)
        //   addPoint(55, 40)
        // Expected: min=(5,10), max=(55,40)
        todo!()
    }

    // ── drawEmpty ──────────────────────────────────────────────────

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_empty_origin() {
        // Java: LimitFinder.drawEmpty(0, 0, 40, 20)
        //   addPoint(0, 0)
        //   addPoint(40, 20)
        // Expected: min=(0,0), max=(40,20)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_empty_with_translate() {
        // Java: translate(10, 10) then drawEmpty(0, 0, 40, 20)
        //   addPoint(10, 10)
        //   addPoint(50, 30)
        // Expected: min=(10,10), max=(50,30)
        todo!()
    }

    // ── drawEllipse ────────────────────────────────────────────────

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_ellipse_origin() {
        // Java: LimitFinder.drawEllipse(0, 0, 60, 40)
        //   cx=30, cy=20, rx=30, ry=20
        //   addPoint(0, 0)
        //   addPoint(0+60-1, 0+40-1) = (59, 39)
        // Expected: min=(0,0), max=(59,39)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn draw_ellipse_with_translate() {
        // Java: translate(10, 10) then drawEllipse(0, 0, 60, 40)
        //   addPoint(10, 10)
        //   addPoint(10+60-1, 10+40-1) = (69, 49)
        // Expected: min=(10,10), max=(69,49)
        todo!()
    }

    // ── Multiple shapes accumulate ─────────────────────────────────

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn multiple_shapes_accumulate() {
        // Java: drawULine(0, 0, 50, 30) then translate(100, 100) then drawULine(0, 0, 20, 10)
        //   First line: addPoint(0,0), addPoint(50,30)
        //   Second line (translated): addPoint(100,100), addPoint(120,110)
        // Expected: min=(0,0), max=(120,110)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn rect_and_ellipse_accumulate() {
        // Java: drawRectangle(0, 0, 100, 50) then drawEllipse(50, 25, 60, 40)
        //   Rect: addPoint(-1,-1), addPoint(99,49)
        //   Ellipse: addPoint(50,25), addPoint(50+60-1, 25+40-1) = (109, 64)
        // Expected: min=(-1,-1), max=(109,64)
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. TextLimitFinder  — pub(crate), all #[ignore]
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod text_limit_finder_tests {
    //! Ports of TextLimitFinderTest.java from net.sourceforge.plantuml.klimt.drawing.
    //!
    //! TextLimitFinder corresponds to the track_text() method of BoundsTracker.
    //! BoundsTracker is pub(crate) so these tests are all ignored stubs.

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_origin() {
        // Java: LimitFinder.drawText(0, 14, "Hello", width=40, height=14)
        //   addPoint(0, 14 - 14 + 1.5) = (0, 1.5)
        //   addPoint(0 + 40, 14 + 14) = (40, 28)
        // Expected: min=(0,1.5), max=(40,28)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_with_translate() {
        // Java: translate(10, 20) then drawText(0, 14, "Hello", width=40, height=14)
        //   addPoint(10, 20+14-14+1.5) = (10, 21.5)
        //   addPoint(10+40, 20+14+14) = (50, 48)
        // Expected: min=(10,21.5), max=(50,48)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_wide_string() {
        // Java: drawText(0, 14, "LongStringContent", width=200, height=14)
        //   addPoint(0, 1.5)
        //   addPoint(200, 28)
        // Expected: min=(0,1.5), max=(200,28)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_large_font_size() {
        // Java: drawText(0, 24, "Big", width=60, height=24)
        //   addPoint(0, 24 - 24 + 1.5) = (0, 1.5)
        //   addPoint(60, 48)
        // Expected: min=(0,1.5), max=(60,48)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_negative_coords() {
        // Java: drawText(-10, 14, "Hello", width=40, height=14)
        //   addPoint(-10, 1.5)
        //   addPoint(30, 28)
        // Expected: min=(-10,1.5), max=(30,28)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_multiple_calls_accumulate() {
        // Java: drawText(0, 14, ..., w=40, h=14) then drawText(50, 14, ..., w=60, h=14)
        //   First:  (0, 1.5) to (40, 28)
        //   Second: (50, 1.5) to (110, 28)
        // Expected: min=(0,1.5), max=(110,28)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_zero_width() {
        // Java: drawText(0, 14, "", width=0, height=14)
        //   addPoint(0, 1.5)
        //   addPoint(0, 28)
        // Expected: min=(0,1.5), max=(0,28)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_combined_with_rect() {
        // Java: drawRectangle(0, 0, 100, 50) then drawText(10, 14, ..., w=40, h=14)
        //   Rect: (-1, -1) to (99, 49)
        //   Text: (10, 1.5) to (50, 28)
        // Expected: min=(-1,-1), max=(99,49)
        todo!()
    }

    #[test]
    #[ignore = "gap: BoundsTracker is pub(crate), not accessible from integration tests"]
    fn track_text_text_extends_bounds_beyond_rect() {
        // Java: drawRectangle(0, 0, 30, 20) then drawText(0, 14, ..., w=200, h=14)
        //   Rect: (-1, -1) to (29, 19)
        //   Text: (0, 1.5) to (200, 28)
        // Expected: min=(-1,-1), max=(200,28)
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. UGraphicNull  — pub(crate), #[ignore]
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod ugraphic_null_tests {
    //! Port of UGraphicNullTest.java.
    //!
    //! UGraphicNull is a no-op UGraphic implementation used for bounds
    //! computation.  It is not exposed in the public API.

    #[test]
    #[ignore = "gap: UGraphicNull is an internal type, not accessible from integration tests"]
    fn ugraphic_null_draw_emits_nothing() {
        // Java: UGraphicNull ignores all draw operations and produces no output.
        // Used purely for LimitFinder bounds accumulation.
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. UGraphicFilter  — pub(crate), #[ignore]
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod ugraphic_filter_tests {
    //! Port of UGraphicFilterTest.java.
    //!
    //! UGraphicFilter wraps another UGraphic and selectively filters shapes.

    #[test]
    #[ignore = "gap: UGraphicFilter is an internal type, not accessible from integration tests"]
    fn ugraphic_filter_rejects_hidden_shapes() {
        // Java: UGraphicFilter with a predicate that rejects ULine.
        // drawLine should be a no-op; drawRectangle should pass through.
        todo!()
    }

    #[test]
    #[ignore = "gap: UGraphicFilter is an internal type, not accessible from integration tests"]
    fn ugraphic_filter_passes_accepted_shapes() {
        // Java: UGraphicFilter with accept-all predicate.
        // All shapes should be drawn to the delegate UGraphic.
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. UGraphicDelegator  — pub(crate), #[ignore]
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod ugraphic_delegator_tests {
    //! Port of UGraphicDelegatorTest.java.
    //!
    //! UGraphicDelegator wraps another UGraphic and forwards all calls,
    //! maintaining its own translate/param state.

    #[test]
    #[ignore = "gap: UGraphicDelegator is an internal type, not accessible from integration tests"]
    fn delegator_forwards_draw_rect() {
        // Java: draw a rectangle through the delegator; verify the delegate
        // receives the rectangle with correct translated coordinates.
        todo!()
    }

    #[test]
    #[ignore = "gap: UGraphicDelegator is an internal type, not accessible from integration tests"]
    fn delegator_composes_translate() {
        // Java: apply UTranslate(10,20) then UTranslate(5,5) to delegator;
        // final translate should be (15,25) before forwarding.
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. UGraphicStencil  — pub(crate), #[ignore]
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod ugraphic_stencil_tests {
    //! Port of UGraphicStencilTest.java.
    //!
    //! UGraphicStencil clips drawing to a stencil region.

    #[test]
    #[ignore = "gap: UGraphicStencil is an internal type, not accessible from integration tests"]
    fn stencil_clips_line_to_region() {
        // Java: drawLine from (-10, 0) to (200, 0) with stencil [0, 100].
        // The line should be clipped to x=[0, 100].
        todo!()
    }

    #[test]
    #[ignore = "gap: UGraphicStencil is an internal type, not accessible from integration tests"]
    fn stencil_passes_fully_inside_shape() {
        // Java: drawRectangle(10, 10, 30, 20) with stencil [0, 100].
        // Rectangle fully inside stencil should pass unchanged.
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. UGraphicDispatchDrawable  — pub(crate), #[ignore]
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod ugraphic_dispatch_drawable_tests {
    //! Port of UGraphicDispatchDrawableTest.java.
    //!
    //! UGraphicDispatchDrawable fans out drawing calls to multiple UGraphic
    //! instances (e.g., one per swimlane).

    #[test]
    #[ignore = "gap: UGraphicDispatchDrawable is an internal type, not accessible from integration tests"]
    fn dispatch_fans_out_draw_to_all_delegates() {
        // Java: draw a rectangle through the dispatcher.
        // All delegate UGraphics should receive the rectangle.
        todo!()
    }

    #[test]
    #[ignore = "gap: UGraphicDispatchDrawable is an internal type, not accessible from integration tests"]
    fn dispatch_applies_per_lane_translate() {
        // Java: each lane delegate has a different translate offset.
        // Verify each receives the rectangle at its lane-specific offset.
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Public API tests — accessible types exercised directly
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod public_svg_graphic_tests {
    use super::*;

    /// Verify that SvgGraphic tracks visibility bounds correctly,
    /// which is the public-API equivalent of LimitFinder for rectangles.
    #[test]
    fn svg_graphic_ensure_visible_after_rectangle() {
        let mut sg = SvgGraphic::new(42, 1.0);
        sg.set_fill_color("#FFFFFF");
        sg.set_stroke_color(Some("#000000"));
        sg.set_stroke_width(1.0, None);
        sg.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);
        // ensure_visible is called with (x + width + 2*shadow, y + height + 2*shadow)
        // = (100, 50) -> max_x = (100+1) as i32 = 101, max_y = (50+1) as i32 = 51
        assert!(sg.max_x() >= 101);
        assert!(sg.max_y() >= 51);
    }

    /// Verify that SvgGraphic tracks bounds for ellipses.
    #[test]
    fn svg_graphic_ensure_visible_after_ellipse() {
        let mut sg = SvgGraphic::new(42, 1.0);
        sg.set_fill_color("#FFFFFF");
        sg.set_stroke_color(Some("#000000"));
        sg.set_stroke_width(1.0, None);
        // cx=50, cy=25, rx=30, ry=20 -> ensure_visible(80, 45)
        sg.svg_ellipse(50.0, 25.0, 30.0, 20.0, 0.0);
        assert!(sg.max_x() >= 80);
        assert!(sg.max_y() >= 45);
    }

    /// Verify that SvgGraphic tracks bounds for lines.
    #[test]
    fn svg_graphic_ensure_visible_after_line() {
        let mut sg = SvgGraphic::new(42, 1.0);
        sg.set_stroke_color(Some("#000000"));
        sg.set_stroke_width(1.0, None);
        sg.svg_line(0.0, 0.0, 200.0, 150.0, 0.0);
        assert!(sg.max_x() >= 200);
        assert!(sg.max_y() >= 150);
    }

    /// Verify that multiple shapes accumulate visibility bounds.
    #[test]
    fn svg_graphic_multiple_shapes_accumulate_bounds() {
        let mut sg = SvgGraphic::new(42, 1.0);
        sg.set_fill_color("#FFFFFF");
        sg.set_stroke_color(Some("#000000"));
        sg.set_stroke_width(1.0, None);
        sg.svg_rectangle(0.0, 0.0, 50.0, 30.0, 0.0, 0.0, 0.0);
        sg.svg_line(0.0, 0.0, 200.0, 150.0, 0.0);
        assert!(sg.max_x() >= 200);
        assert!(sg.max_y() >= 150);
    }

    /// Verify shadow increases tracked bounds.
    #[test]
    fn svg_graphic_shadow_increases_bounds() {
        let mut sg_no_shadow = SvgGraphic::new(42, 1.0);
        sg_no_shadow.set_fill_color("#FFFFFF");
        sg_no_shadow.set_stroke_color(Some("#000000"));
        sg_no_shadow.set_stroke_width(1.0, None);
        sg_no_shadow.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 0.0);

        let mut sg_shadow = SvgGraphic::new(43, 1.0);
        sg_shadow.set_fill_color("#FFFFFF");
        sg_shadow.set_stroke_color(Some("#000000"));
        sg_shadow.set_stroke_width(1.0, None);
        sg_shadow.svg_rectangle(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 5.0);

        assert!(sg_shadow.max_x() > sg_no_shadow.max_x());
        assert!(sg_shadow.max_y() > sg_no_shadow.max_y());
    }
}

#[cfg(test)]
mod public_format_tests {
    use super::*;

    /// Java LimitFinder relies on coordinate formatting for SVG output.
    /// Verify fmt_coord matches Java's String.format("%.4f", x) with
    /// trailing-zero stripping.
    #[test]
    fn fmt_coord_integer() {
        assert_eq!(fmt_coord(42.0), "42");
    }

    #[test]
    fn fmt_coord_zero() {
        assert_eq!(fmt_coord(0.0), "0");
    }

    #[test]
    fn fmt_coord_fractional() {
        assert_eq!(fmt_coord(1.5), "1.5");
    }

    #[test]
    fn fmt_coord_four_decimal_places_rounded() {
        // 1.23456 -> 1.2346 (Java half-up rounding)
        assert_eq!(fmt_coord(1.23456), "1.2346");
    }

    #[test]
    fn fmt_coord_trailing_zeros_stripped() {
        assert_eq!(fmt_coord(1.50), "1.5");
        assert_eq!(fmt_coord(10.0), "10");
    }

    #[test]
    fn fmt_coord_negative() {
        assert_eq!(fmt_coord(-5.0), "-5");
    }
}

#[cfg(test)]
mod public_xml_escape_tests {
    use super::*;

    #[test]
    fn xml_escape_ampersand() {
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn xml_escape_angle_brackets() {
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn xml_escape_quotes() {
        assert_eq!(xml_escape("say \"hello\""), "say &quot;hello&quot;");
    }

    #[test]
    fn xml_escape_non_ascii_to_numeric_entity() {
        // Non-ASCII characters are encoded as &#NNN;
        assert_eq!(xml_escape("\u{00E9}"), "&#233;");
    }

    #[test]
    fn xml_escape_ascii_passthrough() {
        assert_eq!(xml_escape("plain text 123"), "plain text 123");
    }
}

#[cfg(test)]
mod public_length_adjust_tests {
    use super::*;

    #[test]
    fn length_adjust_default_is_spacing() {
        let la = LengthAdjust::default();
        assert_eq!(la, LengthAdjust::Spacing);
    }

    #[test]
    fn length_adjust_variants_distinct() {
        assert_ne!(LengthAdjust::Spacing, LengthAdjust::SpacingAndGlyphs);
        assert_ne!(LengthAdjust::Spacing, LengthAdjust::None);
        assert_ne!(LengthAdjust::SpacingAndGlyphs, LengthAdjust::None);
    }
}

#[cfg(test)]
mod public_uclip_drawing_interaction_tests {
    use super::*;

    /// UClip is used by UGraphic implementations to clip drawing regions.
    /// Verify the core clip behavior that LimitFinder/BoundsTracker depend on.
    #[test]
    fn uclip_boundary_points_inside() {
        let clip = UClip::new(0.0, 0.0, 100.0, 50.0);
        // Origin corner
        assert!(clip.is_inside(0.0, 0.0));
        // Far corner (inclusive)
        assert!(clip.is_inside(100.0, 50.0));
        // Center
        assert!(clip.is_inside(50.0, 25.0));
    }

    #[test]
    fn uclip_outside_points() {
        let clip = UClip::new(0.0, 0.0, 100.0, 50.0);
        assert!(clip.is_inside(-0.1, 25.0) == false);
        assert!(clip.is_inside(50.0, 50.1) == false);
    }

    /// Verify clipping coordinates used by drawing code.
    #[test]
    fn uclip_clipped_coords_match_limits() {
        let clip = UClip::new(10.0, 20.0, 100.0, 50.0);
        // x below range -> clamped to x
        assert_eq!(clip.clipped_x(5.0), 10.0);
        // x within range -> unchanged
        assert_eq!(clip.clipped_x(50.0), 50.0);
        // x above range -> clamped to x + width
        assert_eq!(clip.clipped_x(200.0), 110.0);
        // y below range -> clamped to y
        assert_eq!(clip.clipped_y(10.0), 20.0);
        // y within range -> unchanged
        assert_eq!(clip.clipped_y(40.0), 40.0);
        // y above range -> clamped to y + height
        assert_eq!(clip.clipped_y(80.0), 70.0);
    }

    /// translate() adjusts clip region like Java UClip.moved(UTranslate).
    #[test]
    fn uclip_translate_interaction_with_drawing() {
        let clip = UClip::new(0.0, 0.0, 100.0, 50.0);
        let t = UTranslate::new(10.0, 20.0);
        let moved = clip.translate(t.dx, t.dy);
        assert_eq!(moved.x, 10.0);
        assert_eq!(moved.y, 20.0);
        assert_eq!(moved.width, 100.0);
        assert_eq!(moved.height, 50.0);
        // Point that was inside at origin is now outside
        assert!(moved.is_inside(5.0, 5.0) == false);
        // Point at new origin is inside
        assert!(moved.is_inside(10.0, 20.0));
    }
}

#[cfg(test)]
mod public_utranslate_drawing_interaction_tests {
    use super::*;

    /// UTranslate.compose mirrors the way LimitFinder accumulates translations.
    #[test]
    fn utranslate_compose_accumulates_like_limit_finder() {
        let t1 = UTranslate::new(10.0, 20.0);
        let t2 = UTranslate::new(5.0, -3.0);
        let composed = t1.compose(t2);
        assert_eq!(composed.dx, 15.0);
        assert_eq!(composed.dy, 17.0);
    }

    /// UTranslate.reverse undoes the translation.
    #[test]
    fn utranslate_reverse_returns_to_origin() {
        let t = UTranslate::new(100.0, 200.0);
        let round_trip = t.compose(t.reverse());
        assert!((round_trip.dx).abs() < 1e-10);
        assert!((round_trip.dy).abs() < 1e-10);
    }

    /// UTranslate.scaled is used for zoom transformations in drawing.
    #[test]
    fn utranslate_scaled_doubles_offset() {
        let t = UTranslate::new(10.0, 20.0);
        let scaled = t.scaled(2.0);
        assert_eq!(scaled.dx, 20.0);
        assert_eq!(scaled.dy, 40.0);
    }
}

#[cfg(test)]
mod public_ustroke_drawing_interaction_tests {
    use super::*;

    /// UStroke.dasharray_svg is used by the SVG drawing driver to set
    /// stroke-dasharray on SVG elements.
    #[test]
    fn ustroke_solid_produces_no_dasharray() {
        let s = UStroke::with_thickness(2.0);
        assert!(s.dasharray_svg().is_none());
    }

    #[test]
    fn ustroke_dashed_produces_dasharray() {
        let s = UStroke::new(5.0, 3.0, 1.0);
        let (vis, space) = s.dasharray_svg().unwrap();
        assert!((vis - 5.0).abs() < 1e-10);
        assert!((space - 3.0).abs() < 1e-10);
    }

    /// only_thickness() is used to strip dash patterns while preserving line
    /// weight, matching how Java drivers call UStroke.onlyStroke().
    #[test]
    fn ustroke_only_thickness_clears_dash() {
        let dashed = UStroke::new(8.0, 4.0, 2.5);
        let solid = dashed.only_thickness();
        assert!(solid.dasharray_svg().is_none());
        assert!((solid.thickness - 2.5).abs() < 1e-10);
    }
}
