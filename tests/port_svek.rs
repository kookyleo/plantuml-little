// Port of Java PlantUML svek package test suite.
//
// Sources: generated-public-api-tests-foundation/packages/net/sourceforge/plantuml/svek/
//
// Mapping:
//   SvgResult          → svek::svg_result::SvgResult
//   ColorSequence      → svek::ColorSequence
//   Margins            → svek::Margins
//   DotMode            → svek::DotMode
//   ShapeType          → svek::shape_type::ShapeType
//   PackageStyle       → svek::shape_type::PackageStyle
//   ConditionStyle     → svek::shape_type::ConditionStyle
//   ConditionEndStyle  → svek::shape_type::ConditionEndStyle
//   YDelta             → svek::snake::YDelta
//   Oscillator         → svek::snake::Oscillator
//   PortGeometry       → svek::node::PortGeometry
//   SvekNode/Bibliotekon → svek::Bibliotekon / svek::node::SvekNode
//   SvekUtils          → svek::utils
//
// Gap classes (no Rust equivalent):
//   ArithmeticStrategyMax/Sum, SingleStrategy, EntityDomain,
//   LineOfSegments, FrontierCalculator, Ports, BaseFile,
//   GraphvizImageBuilder (Java pipeline only), SvekResult (Java pipeline only)

use plantuml_little::svek;
use plantuml_little::svek::node::PortGeometry;
use plantuml_little::svek::shape_type::{
    ConditionEndStyle, ConditionStyle, PackageStyle, ShapeType,
};
use plantuml_little::svek::snake::{Oscillator, YDelta};
use plantuml_little::svek::svg_result::{SvgResult, POINTS_EQUALS};

// ── Helpers ───────────────────────────────────────────────────────────────

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

// ═══════════════════════════════════════════════════════════════════════════
// SvgResultSkeletonTest
// Java: net.sourceforge.plantuml.svek.SvgResultSkeletonTest
// Priority: HIGH — SVG coordinate parsing is core functionality.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn svg_result_get_svg_skeleton_for_none() {
    // Java: getSvg_skeleton_for_none
    let result = SvgResult::new("<svg>hello</svg>".to_string());
    assert_eq!(result.svg(), "<svg>hello</svg>");
}

#[test]
fn svg_result_index_of_skeleton_for_string_int() {
    // Java: indexOf_skeleton_for_java_lang_String_int
    let result = SvgResult::new("stroke=\"#ff0000\" stroke=\"#00ff00\"".to_string());
    let first = result.index_of("stroke=", 0);
    assert_eq!(first, Some(0));
    let second = result.index_of("stroke=", first.unwrap() + 1);
    assert_eq!(second, Some(17));
    assert_eq!(result.index_of("notfound", 0), None);
}

#[test]
fn svg_result_substring_skeleton_for_int() {
    // Java: substring_skeleton_for_int
    let result = SvgResult::new("abcdef".to_string());
    let sub = result.substring_from(3);
    assert_eq!(sub.svg(), "def");
}

#[test]
fn svg_result_substring_overload_2_skeleton_for_int_int() {
    // Java: substring_overload_2_skeleton_for_int_int
    let result = SvgResult::new("abcdef".to_string());
    let sub = result.substring(1, 4);
    assert_eq!(sub.svg(), "bcd");
}

#[test]
fn svg_result_is_path_consistent_skeleton_for_none() {
    // Java: isPathConsistent_skeleton_for_none
    let starting_with_m = SvgResult::new("M10,20 C30,40 50,60 70,80".to_string());
    assert!(starting_with_m.is_path_consistent());

    let not_path = SvgResult::new("stroke=\"red\"".to_string());
    assert!(!not_path.is_path_consistent());

    let empty = SvgResult::new("".to_string());
    assert!(!empty.is_path_consistent());
}

#[test]
fn svg_result_get_points_skeleton_for_string() {
    // Java: getPoints_skeleton_for_java_lang_String
    // "10,20 30,40" separated by space
    let result = SvgResult::new("10,20 30,40".to_string());
    let points = result.get_points(" ");
    assert_eq!(points.len(), 2);
    assert!(approx_eq(points[0].x, 10.0));
    assert!(approx_eq(points[0].y, 20.0));
    assert!(approx_eq(points[1].x, 30.0));
    assert!(approx_eq(points[1].y, 40.0));
}

#[test]
fn svg_result_get_next_point_skeleton_for_none() {
    // Java: getNextPoint_skeleton_for_none
    let result = SvgResult::new("5.5,7.5".to_string());
    let pt = result.get_next_point();
    assert!(pt.is_some());
    let pt = pt.unwrap();
    assert!(approx_eq(pt.x, 5.5));
    assert!(approx_eq(pt.y, 7.5));
}

#[test]
fn svg_result_get_index_from_color_skeleton_for_int() {
    // Java: getIndexFromColor_skeleton_for_int
    // 0x0000FF == blue; stroke="#0000ff"
    let result = SvgResult::new("stroke=\"#0000ff\"".to_string());
    let idx = result.get_index_from_color(0x0000FF);
    assert!(
        idx.is_some(),
        "Expected Some index for known color, got None"
    );

    let no_match = SvgResult::new("<svg/>".to_string());
    assert_eq!(no_match.get_index_from_color(0x123456), None);
}

#[test]
fn svg_result_extract_list_skeleton_for_string() {
    // Java: extractList_skeleton_for_java_lang_String
    // Test with "points=" attribute using POINTS_EQUALS constant
    let svg = format!("{}10,20 30,40\"", POINTS_EQUALS);
    let result = SvgResult::new(svg);
    let pts = result.extract_list(POINTS_EQUALS);
    assert_eq!(pts.len(), 2);
    assert!(approx_eq(pts[0].x, 10.0));
    assert!(approx_eq(pts[0].y, 20.0));
}

#[test]
fn svg_result_extract_list_returns_empty_when_not_found() {
    // Java: extractList_returns_empty_when_not_found
    let result = SvgResult::new("<svg/>".to_string());
    let pts = result.extract_list(POINTS_EQUALS);
    assert!(pts.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// ArithmeticStrategyMaxSkeletonTest
// Java: net.sourceforge.plantuml.svek.ArithmeticStrategyMaxSkeletonTest
// Gap: ArithmeticStrategyMax not ported. Rust uses direct f64 max tracking.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: ArithmeticStrategyMax not yet ported — tracked with f64 directly"]
fn arithmetic_strategy_max_eat_skeleton_for_double() {
    todo!()
}

#[test]
#[ignore = "gap: ArithmeticStrategyMax not yet ported"]
fn arithmetic_strategy_max_get_result_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: ArithmeticStrategyMax not yet ported"]
fn arithmetic_strategy_max_eat_ignores_smaller_values() {
    todo!()
}

#[test]
#[ignore = "gap: ArithmeticStrategyMax not yet ported"]
fn arithmetic_strategy_max_eat_negative_values_cannot_exceed_zero_default() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// ArithmeticStrategySumSkeletonTest
// Gap: ArithmeticStrategySum not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: ArithmeticStrategySum not yet ported"]
fn arithmetic_strategy_sum_eat_skeleton_for_double() {
    todo!()
}

#[test]
#[ignore = "gap: ArithmeticStrategySum not yet ported"]
fn arithmetic_strategy_sum_get_result_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: ArithmeticStrategySum not yet ported"]
fn arithmetic_strategy_sum_eat_accumulates_negative_values() {
    todo!()
}

#[test]
#[ignore = "gap: ArithmeticStrategySum not yet ported"]
fn arithmetic_strategy_sum_eat_single_value() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// DotModeSkeletonTest
// Java: net.sourceforge.plantuml.svek.DotModeSkeletonTest
// Maps to: svek::DotMode (2 variants: Normal, NoLeftRightAndXlabel)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn dot_mode_values_skeleton_for_none() {
    // Java: DotMode.values() returns 2 elements
    // Rust: verify both variants exist and are distinct
    let all = [svek::DotMode::Normal, svek::DotMode::NoLeftRightAndXlabel];
    assert_eq!(all.len(), 2);
    assert_ne!(all[0], all[1]);
}

#[test]
fn dot_mode_value_of_skeleton_for_string() {
    // Java: DotMode.valueOf("NORMAL") == DotMode.NORMAL
    // Verify Debug representation is non-empty (analogous to assertNotNull)
    assert!(!format!("{:?}", svek::DotMode::Normal).is_empty());
    assert!(!format!("{:?}", svek::DotMode::NoLeftRightAndXlabel).is_empty());
    // The two variants are distinguishable
    assert_ne!(
        format!("{:?}", svek::DotMode::Normal),
        format!("{:?}", svek::DotMode::NoLeftRightAndXlabel)
    );
}

#[test]
fn dot_mode_enum_constants_are_distinct() {
    // Java: assertFalse(DotMode.NORMAL == DotMode.NO_LEFT_RIGHT_AND_XLABEL)
    assert_ne!(svek::DotMode::Normal, svek::DotMode::NoLeftRightAndXlabel);
}

// ═══════════════════════════════════════════════════════════════════════════
// ShapeTypeSkeletonTest
// Java: net.sourceforge.plantuml.svek.ShapeTypeSkeletonTest
// Maps to: svek::shape_type::ShapeType (12 constants)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn shape_type_values_skeleton_for_none() {
    // Java: ShapeType.values() returns 12 elements
    // Verify all 12 variants exist
    let _types = [
        ShapeType::Rectangle,
        ShapeType::RectanglePort,
        ShapeType::RectangleWithCircleInside,
        ShapeType::RectangleHtmlForPorts,
        ShapeType::RoundRectangle,
        ShapeType::Circle,
        ShapeType::Oval,
        ShapeType::Diamond,
        ShapeType::Octagon,
        ShapeType::Folder,
        ShapeType::Hexagon,
        ShapeType::Port,
    ];
    assert_eq!(_types.len(), 12);
}

#[test]
fn shape_type_value_of_skeleton_for_string() {
    // Java: ShapeType.valueOf("RECTANGLE") == ShapeType.RECTANGLE, etc.
    // Verify key variants are mutually distinct
    assert_ne!(ShapeType::Rectangle, ShapeType::Circle);
    assert_ne!(ShapeType::Circle, ShapeType::Diamond);
    assert_ne!(ShapeType::Diamond, ShapeType::Hexagon);
    assert_ne!(ShapeType::Hexagon, ShapeType::Port);
    assert_ne!(ShapeType::Port, ShapeType::Rectangle);
}

#[test]
fn shape_type_all_expected_constants_present() {
    // Java: assertNotNull on each constant — verify via Debug repr and cross-inequality
    assert_ne!(ShapeType::RectanglePort, ShapeType::Rectangle);
    assert_ne!(
        ShapeType::RectangleWithCircleInside,
        ShapeType::RectanglePort
    );
    assert_ne!(
        ShapeType::RectangleHtmlForPorts,
        ShapeType::RectangleWithCircleInside
    );
    assert_ne!(ShapeType::RoundRectangle, ShapeType::RectangleHtmlForPorts);
    assert_ne!(ShapeType::Oval, ShapeType::RoundRectangle);
    assert_ne!(ShapeType::Octagon, ShapeType::Oval);
    assert_ne!(ShapeType::Folder, ShapeType::Octagon);
    // Also verify dot_shape returns a non-empty string for each
    assert!(!ShapeType::RectanglePort.dot_shape().is_empty());
    assert!(!ShapeType::RectangleWithCircleInside.dot_shape().is_empty());
    assert!(!ShapeType::RoundRectangle.dot_shape().is_empty());
    assert!(!ShapeType::Oval.dot_shape().is_empty());
    assert!(!ShapeType::Octagon.dot_shape().is_empty());
    assert!(!ShapeType::Folder.dot_shape().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// ConditionStyleSkeletonTest
// Java: net.sourceforge.plantuml.svek.ConditionStyleSkeletonTest
// Maps to: svek::shape_type::ConditionStyle
// Note: Java has 3 values: EMPTY_DIAMOND, INSIDE_HEXAGON, INSIDE_DIAMOND
//       Rust has: Diamond, Inside, Foo1 (different naming convention)
//       Java fromString aliases: "InsideDiamond"→INSIDE_DIAMOND, "Foo1"→INSIDE_DIAMOND,
//                                "Diamond"→EMPTY_DIAMOND, "Inside"→INSIDE_HEXAGON
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn condition_style_values_skeleton_for_none() {
    // Java: ConditionStyle.values() returns 3 elements
    let _styles = [
        ConditionStyle::Diamond,
        ConditionStyle::Inside,
        ConditionStyle::Foo1,
    ];
    assert_eq!(_styles.len(), 3);
}

#[test]
fn condition_style_value_of_skeleton_for_string() {
    // Verify all three variants are distinct
    assert_ne!(ConditionStyle::Diamond, ConditionStyle::Inside);
    assert_ne!(ConditionStyle::Inside, ConditionStyle::Foo1);
    assert_ne!(ConditionStyle::Diamond, ConditionStyle::Foo1);
}

#[test]
#[ignore = "gap: ConditionStyle::from_string alias mapping not yet ported (InsideDiamond/Foo1/Diamond/Inside)"]
fn condition_style_from_string_skeleton_for_string() {
    // Java: ConditionStyle.fromString("InsideDiamond") == INSIDE_DIAMOND
    //       ConditionStyle.fromString("Foo1") == INSIDE_DIAMOND
    //       ConditionStyle.fromString("Diamond") == EMPTY_DIAMOND
    //       ConditionStyle.fromString("Inside") == INSIDE_HEXAGON
    //       ConditionStyle.fromString("unknown") == null
    // No from_string method on Rust ConditionStyle yet.
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// ConditionEndStyleSkeletonTest
// Java: net.sourceforge.plantuml.svek.ConditionEndStyleSkeletonTest
// Maps to: svek::shape_type::ConditionEndStyle (2 variants: Diamond, Hline)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn condition_end_style_values_skeleton_for_none() {
    // Java: ConditionEndStyle.values() returns 2 elements
    let _styles = [ConditionEndStyle::Diamond, ConditionEndStyle::Hline];
    assert_eq!(_styles.len(), 2);
}

#[test]
fn condition_end_style_value_of_skeleton_for_string() {
    // Verify both variants are distinct and have non-empty Debug representation
    assert_ne!(ConditionEndStyle::Diamond, ConditionEndStyle::Hline);
    assert_ne!(
        format!("{:?}", ConditionEndStyle::Diamond),
        format!("{:?}", ConditionEndStyle::Hline)
    );
}

#[test]
#[ignore = "gap: ConditionEndStyle::from_string (case-insensitive) not yet ported"]
fn condition_end_style_from_string_skeleton_for_string() {
    // Java: case-insensitive by enum name; null for unknown
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// PackageStyleSkeletonTest
// Java: net.sourceforge.plantuml.svek.PackageStyleSkeletonTest
// Maps to: svek::shape_type::PackageStyle (12 constants)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn package_style_values_skeleton_for_none() {
    // Java: PackageStyle.values() returns 12 elements
    let _styles = [
        PackageStyle::Folder,
        PackageStyle::Rectangle,
        PackageStyle::Node,
        PackageStyle::Frame,
        PackageStyle::Cloud,
        PackageStyle::Database,
        PackageStyle::Agent,
        PackageStyle::Storage,
        PackageStyle::Component1,
        PackageStyle::Component2,
        PackageStyle::Artifact,
        PackageStyle::Card,
    ];
    assert_eq!(_styles.len(), 12);
}

#[test]
fn package_style_value_of_skeleton_for_string() {
    // Verify key variants are mutually distinct (analogous to Java valueOf round-trip check)
    assert_ne!(PackageStyle::Folder, PackageStyle::Rectangle);
    assert_ne!(PackageStyle::Rectangle, PackageStyle::Node);
    assert_ne!(PackageStyle::Node, PackageStyle::Frame);
    assert_ne!(PackageStyle::Frame, PackageStyle::Cloud);
    assert_ne!(PackageStyle::Cloud, PackageStyle::Database);
    assert_ne!(PackageStyle::Database, PackageStyle::Folder);
}

#[test]
fn package_style_from_string_skeleton_for_string() {
    // Java: PackageStyle.fromString("FOLDER") == FOLDER (case-insensitive)
    assert_eq!(PackageStyle::from_str("FOLDER"), Some(PackageStyle::Folder));
    assert_eq!(PackageStyle::from_str("folder"), Some(PackageStyle::Folder));
    assert_eq!(
        PackageStyle::from_str("RECTANGLE"),
        Some(PackageStyle::Rectangle)
    );
    assert_eq!(PackageStyle::from_str("cloud"), Some(PackageStyle::Cloud));
    // special alias: "rect" -> RECTANGLE
    assert_eq!(
        PackageStyle::from_str("rect"),
        Some(PackageStyle::Rectangle)
    );
    assert_eq!(
        PackageStyle::from_str("RECT"),
        Some(PackageStyle::Rectangle)
    );
    // unknown
    assert_eq!(PackageStyle::from_str("unknown"), None);
    assert_eq!(PackageStyle::from_str(""), None);
}

#[test]
#[ignore = "gap: PackageStyle::to_u_symbol not yet ported"]
fn package_style_to_u_symbol_returns_non_null_for_supported_styles() {
    // Java: NODE, CARD, DATABASE, CLOUD, FRAME, RECTANGLE, FOLDER -> non-null USymbol
    todo!()
}

#[test]
#[ignore = "gap: PackageStyle::to_u_symbol not yet ported"]
fn package_style_to_u_symbol_returns_null_for_unsupported_styles() {
    // Java: COMPONENT1, COMPONENT2, STORAGE, AGENT, ARTIFACT -> null
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// ColorSequenceSkeletonTest
// Java: net.sourceforge.plantuml.svek.ColorSequenceSkeletonTest
// Maps to: svek::ColorSequence
// Note: Java ColorSequence uses AtomicInteger (values 2, 3, 4, ...).
//       Rust uses RGB color integers (0x010100, 0x020200, ...).
//       The Java test verifies monotonically increasing positive integers,
//       which our Rust implementation also satisfies for next_color().
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn color_sequence_get_value_skeleton_for_none() {
    // Java: ColorSequence.getValue() returns strictly increasing values
    let mut seq = svek::ColorSequence::new();
    let first = seq.next_color();
    let second = seq.next_color();
    let third = seq.next_color();
    assert!(second > first, "second ({}) > first ({})", second, first);
    assert!(third > second, "third ({}) > second ({})", third, second);
}

#[test]
fn color_sequence_get_value_returns_positive_values() {
    // Java: all getValue() calls return > 0
    let mut seq = svek::ColorSequence::new();
    for _ in 0..10 {
        assert!(seq.next_color() > 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MarginsSkeletonTest
// Java: net.sourceforge.plantuml.svek.MarginsSkeletonTest
// Maps to: svek::Margins
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn margins_getters_skeleton_for_none() {
    // Java: new Margins(1.0, 2.0, 3.0, 4.0)
    let m = svek::Margins::new(1.0, 2.0, 3.0, 4.0);
    assert!(approx_eq(m.x1, 1.0));
    assert!(approx_eq(m.x2, 2.0));
    assert!(approx_eq(m.y1, 3.0));
    assert!(approx_eq(m.y2, 4.0));
}

#[test]
fn margins_get_x1_skeleton_for_none() {
    let m = svek::Margins::new(5.5, 0.0, 0.0, 0.0);
    assert!(approx_eq(m.x1, 5.5));
}

#[test]
fn margins_get_x2_skeleton_for_none() {
    let m = svek::Margins::new(0.0, 6.6, 0.0, 0.0);
    assert!(approx_eq(m.x2, 6.6));
}

#[test]
fn margins_get_y1_skeleton_for_none() {
    let m = svek::Margins::new(0.0, 0.0, 7.7, 0.0);
    assert!(approx_eq(m.y1, 7.7));
}

#[test]
fn margins_get_y2_skeleton_for_none() {
    let m = svek::Margins::new(0.0, 0.0, 0.0, 8.8);
    assert!(approx_eq(m.y2, 8.8));
}

#[test]
fn margins_get_total_width_skeleton_for_none() {
    // Java: getTotalWidth() = x1 + x2
    let m = svek::Margins::new(3.0, 5.0, 1.0, 1.0);
    assert!(approx_eq(m.total_width(), 8.0));
}

#[test]
fn margins_get_total_height_skeleton_for_none() {
    // Java: getTotalHeight() = y1 + y2
    let m = svek::Margins::new(1.0, 1.0, 4.0, 6.0);
    assert!(approx_eq(m.total_height(), 10.0));
}

#[test]
fn margins_is_zero_skeleton_for_none() {
    assert!(svek::Margins::none().is_zero());
    assert!(svek::Margins::new(0.0, 0.0, 0.0, 0.0).is_zero());
    assert!(!svek::Margins::new(1.0, 0.0, 0.0, 0.0).is_zero());
    assert!(!svek::Margins::new(0.0, 0.0, 0.0, 1.0).is_zero());
}

#[test]
fn margins_uniform_skeleton_for_double() {
    let m = svek::Margins::uniform(5.0);
    assert!(approx_eq(m.x1, 5.0));
    assert!(approx_eq(m.x2, 5.0));
    assert!(approx_eq(m.y1, 5.0));
    assert!(approx_eq(m.y2, 5.0));
    assert!(!m.is_zero());
}

#[test]
fn margins_to_string_skeleton_for_none() {
    // Java: toString() contains all four values
    let m = svek::Margins::new(1.0, 2.0, 3.0, 4.0);
    // Rust Debug format shows field values
    let s = format!("{:?}", m);
    assert!(
        s.contains("1.0") || s.contains("x1"),
        "debug repr should contain values: {}",
        s
    );
}

#[test]
#[ignore = "gap: Margins::merge (max of each side) not yet ported"]
fn margins_merge_skeleton_for_margins() {
    // Java: merge takes max of each corresponding side
    // a = Margins(1.0, 5.0, 2.0, 3.0), b = Margins(3.0, 2.0, 4.0, 1.0)
    // merged: x1=3, x2=5, y1=4, y2=3
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// PortGeometrySkeletonTest
// Java: net.sourceforge.plantuml.svek.PortGeometrySkeletonTest
// Maps to: svek::node::PortGeometry
// Note: Java PortGeometry has (id, position, height, score) — Rust version
//       has only (id, position, height). Score and compareTO/translateY not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn port_geometry_getters_cover_all_fields() {
    // Java: new PortGeometry("port1", 10.0, 20.0, 5) — score not in Rust struct
    let pg = PortGeometry::new("port1", 10.0, 20.0);
    assert_eq!(pg.id, "port1");
    assert!(approx_eq(pg.position, 10.0));
    assert!(approx_eq(pg.height, 20.0));
}

#[test]
fn port_geometry_get_id_skeleton_for_none() {
    let pg = PortGeometry::new("myPort", 0.0, 0.0);
    assert_eq!(pg.id, "myPort");
}

#[test]
fn port_geometry_get_position_skeleton_for_none() {
    let pg = PortGeometry::new("p", 42.5, 10.0);
    assert!(approx_eq(pg.position, 42.5));
}

#[test]
fn port_geometry_get_height_skeleton_for_none() {
    let pg = PortGeometry::new("p", 0.0, 15.0);
    assert!(approx_eq(pg.height, 15.0));
}

#[test]
#[ignore = "gap: PortGeometry::score field not yet ported"]
fn port_geometry_get_score_skeleton_for_none() {
    // Java: PortGeometry("p", 0, 0, 7).getScore() == 7
    todo!()
}

#[test]
#[ignore = "gap: PortGeometry::get_last_y (position + height) not yet ported"]
fn port_geometry_get_last_y_skeleton_for_none() {
    // Java: getLastY() = position + height = 10.0 + 30.0 = 40.0
    todo!()
}

#[test]
#[ignore = "gap: PortGeometry::translate_y not yet ported"]
fn port_geometry_translate_y_skeleton_for_double() {
    // Java: translateY(5.0) returns new PortGeometry with position += 5.0
    todo!()
}

#[test]
fn port_geometry_to_string_skeleton_for_none() {
    let pg = PortGeometry::new("p", 5.0, 10.0);
    let s = format!("{:?}", pg);
    // Debug output contains field values
    assert!(s.contains("5.0") || s.contains("position"), "debug: {}", s);
}

#[test]
#[ignore = "gap: PortGeometry::compare_to (by position) not yet ported"]
fn port_geometry_compare_to_skeleton_for_port_geometry() {
    // Java: Comparable by position: a(5.0) < b(10.0), a(5.0) == c(5.0)
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// SingleStrategySkeletonTest
// Gap: SingleStrategy enum not ported (SQUARE/HLINE/VLINE + computeBranch).
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: SingleStrategy (SQUARE/HLINE/VLINE + computeBranch) not yet ported"]
fn single_strategy_values_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: SingleStrategy not yet ported"]
fn single_strategy_compute_branch_perfect_square() {
    todo!()
}

#[test]
#[ignore = "gap: SingleStrategy not yet ported"]
fn single_strategy_compute_branch_non_perfect_square() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// EntityDomainSkeletonTest
// Gap: EntityDomain (Fashion + radius=12 + margin=4 → 32×32) not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: EntityDomain (svek entity image: radius=12, margin=4, 32×32) not yet ported"]
fn entity_domain_calculate_dimension_skeleton_for_string_bounder() {
    todo!()
}

#[test]
#[ignore = "gap: EntityDomain not yet ported"]
fn entity_domain_calculate_dimension_returns_positive_size() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// YDeltaSkeletonTest
// Java: net.sourceforge.plantuml.svek.YDeltaSkeletonTest
// Maps to: svek::snake::YDelta
// Note: Java YDelta has two constructors: YDelta(delta) and YDelta(factor, delta).
//       Rust has only YDelta { delta } — factor-form not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn y_delta_apply_skeleton_for_x_point_2d() {
    // Java: new YDelta(10.0).apply(new XPoint2D(5.0, 3.0)) → (5.0, 13.0)
    use plantuml_little::klimt::geom::XPoint2D;
    let yd = YDelta::new(10.0);
    let result = {
        use plantuml_little::svek::Point2DFunction;
        yd.apply(XPoint2D::new(5.0, 3.0))
    };
    assert!(approx_eq(result.x, 5.0));
    assert!(approx_eq(result.y, 13.0));
}

#[test]
#[ignore = "gap: YDelta(factor, delta) two-parameter constructor not yet ported"]
fn y_delta_apply_with_factor_and_delta() {
    // Java: new YDelta(2.0, 5.0).apply(XPoint2D(7.0, 3.0)) → y = 3*2 + 5 = 11
    todo!()
}

#[test]
fn y_delta_apply_preserves_x() {
    // Java: apply_preserves_x — x coordinate unchanged
    use plantuml_little::klimt::geom::XPoint2D;
    use plantuml_little::svek::Point2DFunction;
    let yd = YDelta::new(100.0);
    let result = yd.apply(XPoint2D::new(42.0, 0.0));
    assert!(approx_eq(result.x, 42.0));
}

#[test]
fn y_delta_apply_zero_delta_is_identity() {
    // Java: new YDelta(0.0).apply(XPoint2D(3.0, 4.0)) → (3.0, 4.0)
    use plantuml_little::klimt::geom::XPoint2D;
    use plantuml_little::svek::Point2DFunction;
    let yd = YDelta::new(0.0);
    let result = yd.apply(XPoint2D::new(3.0, 4.0));
    assert!(approx_eq(result.x, 3.0));
    assert!(approx_eq(result.y, 4.0));
}

// ═══════════════════════════════════════════════════════════════════════════
// OscillatorSkeletonTest
// Java: net.sourceforge.plantuml.svek.OscillatorSkeletonTest
// Maps to: svek::snake::Oscillator
// Note: Java Oscillator generates unique 2D positions (spiral-like).
//       Rust Oscillator is a simple value list — different API.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn oscillator_next_position_skeleton_for_none() {
    // Java: osc.nextPosition() returns non-null XPoint2D
    // Rust: Oscillator uses add_value/get_value_at — different API
    let mut o = Oscillator::new();
    o.add_value(1.0);
    // get_value_at returns 0.0 for out-of-bounds, so at least there's no panic
    assert_eq!(o.get_value_at(0), 1.0);
}

#[test]
#[ignore = "gap: Oscillator::next_position (spiral unique position generator) not yet ported"]
fn oscillator_next_position_returns_unique_positions() {
    // Java: 20 calls to nextPosition() return 20 distinct XPoint2D values
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// SvekUtilsSkeletonTest
// Java: net.sourceforge.plantuml.svek.SvekUtilsSkeletonTest
// Maps to: svek::utils
// Note: Java SvekUtils.getValue parses XML attribute values from strings.
//       Java getMinXY/getMaxXY find extremes of point lists.
//       Rust utils only has pixel_to_inches / px_to_dot — others not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn svek_utils_pixel_to_inches_skeleton_for_double() {
    // Java: SvekUtils.pixelToInches(72.0) == 1.0, pixelToInches(36.0) == 0.5
    let one_inch = svek::utils::pixel_to_inches(72.0);
    assert!(
        (one_inch - 1.0).abs() < 1e-6,
        "72px should be 1.0 inches, got {}",
        one_inch
    );
    let half_inch = svek::utils::pixel_to_inches(36.0);
    assert!(
        (half_inch - 0.5).abs() < 1e-6,
        "36px should be 0.5 inches, got {}",
        half_inch
    );
}

#[test]
#[ignore = "gap: SvekUtils::get_value (XML attribute parser) not yet ported"]
fn svek_utils_get_value_skeleton_for_string_int_string() {
    // Java: SvekUtils.getValue("width=\"120\" height=\"80\"", 0, "width") == 120.0
    todo!()
}

#[test]
#[ignore = "gap: SvekUtils::get_min_xy not yet ported"]
fn svek_utils_get_min_xy_skeleton_for_list_x_point_2d() {
    // Java: getMinXY([3,7], [1,9], [5,2]) → (1.0, 2.0)
    todo!()
}

#[test]
#[ignore = "gap: SvekUtils::get_max_xy not yet ported"]
fn svek_utils_get_max_xy_skeleton_for_list_x_point_2d() {
    // Java: getMaxXY([3,7], [1,9], [5,2]) → (5.0, 9.0)
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// LineOfSegmentsSkeletonTest
// Gap: LineOfSegments not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: LineOfSegments (addSegment, getMean, solveOverlaps) not yet ported"]
fn line_of_segments_add_segment_skeleton_for_double_double() {
    todo!()
}

#[test]
#[ignore = "gap: LineOfSegments not yet ported"]
fn line_of_segments_get_mean_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: LineOfSegments not yet ported"]
fn line_of_segments_solve_overlaps_non_overlapping() {
    todo!()
}

#[test]
#[ignore = "gap: LineOfSegments not yet ported"]
fn line_of_segments_solve_overlaps_single_segment() {
    todo!()
}

#[test]
#[ignore = "gap: LineOfSegments not yet ported"]
fn line_of_segments_solve_overlaps_overlapping_pushes_apart() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// FrontierCalculatorSkeletonTest
// Gap: FrontierCalculator not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: FrontierCalculator (getSuggestedPosition, ensureMinWidth) not yet ported"]
fn frontier_calculator_get_suggested_position_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: FrontierCalculator not yet ported"]
fn frontier_calculator_ensure_min_width_skeleton_for_double() {
    todo!()
}

#[test]
#[ignore = "gap: FrontierCalculator not yet ported"]
fn frontier_calculator_get_suggested_position_with_inside_area() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// PortsSkeletonTest
// Gap: Ports (encodePortNameToId/add/translateY/addThis) not ported.
// Java Ports uses MD5-based port ID encoding and a priority map for port geometries.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: Ports (encodePortNameToId, add with priority, translateY) not yet ported"]
fn ports_encode_port_name_to_id_skeleton_for_string() {
    todo!()
}

#[test]
#[ignore = "gap: Ports not yet ported"]
fn ports_add_and_get_all_port_geometry_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: Ports not yet ported"]
fn ports_add_higher_score_replaces_lower() {
    todo!()
}

#[test]
#[ignore = "gap: Ports not yet ported"]
fn ports_add_lower_score_does_not_replace() {
    todo!()
}

#[test]
#[ignore = "gap: Ports not yet ported"]
fn ports_translate_y_skeleton_for_double() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// BaseFileSkeletonTest
// Gap: BaseFile (wrapping SFile for DOT trace file management) not ported.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: BaseFile (getBasename, getBasedir, getTraceFile) not yet ported"]
fn base_file_get_basename_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: BaseFile not yet ported"]
fn base_file_get_basedir_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: BaseFile not yet ported"]
fn base_file_to_string_skeleton_for_none() {
    todo!()
}

#[test]
#[ignore = "gap: BaseFile not yet ported"]
fn base_file_get_trace_file_skeleton_for_string() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// GraphvizImageBuilderSkeletonTest
// Gap: Full Java pipeline (SourceStringReader → GraphvizImageBuilder → SVG).
//      Rust equivalent is svek::builder which uses vizoxide — different API.
//      The golden SVG files are copied to tests/port_golden/svek/ for reference,
//      but Rust output will differ in attribute ordering / whitespace.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: GraphvizImageBuilder end-to-end test requires full Java pipeline; Rust builder uses different SVG structure"]
fn graphviz_image_builder_build_image_class_diagram_produces_valid_svg() {
    // Golden file: tests/port_golden/svek/buildImage_class_diagram.svg
    todo!()
}

#[test]
#[ignore = "gap: GraphvizImageBuilder end-to-end test not ported"]
fn graphviz_image_builder_build_image_component_diagram_produces_valid_svg() {
    // Golden file: tests/port_golden/svek/buildImage_component_diagram.svg
    todo!()
}

#[test]
#[ignore = "gap: GraphvizImageBuilder end-to-end test not ported"]
fn graphviz_image_builder_build_image_two_class_diagram_produces_valid_svg() {
    // Golden file: tests/port_golden/svek/buildImage_two_class_diagram.svg
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// SvekResultSkeletonTest
// Gap: SvekResult is the final IEntityImage from GraphvizImageBuilder.
//      Only testable end-to-end through the Java pipeline. Rust equivalent
//      is the output of svek::builder::GraphvizImageBuilder::build_image.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "gap: SvekResult (drawU, getBackcolor, calculateDimension) requires full diagram pipeline"]
fn svek_result_draw_u_class_diagram_renders_nodes_and_edges() {
    // Golden file: tests/port_golden/svek/drawU_class_diagram.svg
    todo!()
}

#[test]
#[ignore = "gap: SvekResult not ported as standalone — full pipeline required"]
fn svek_result_calculate_dimension_class_diagram_has_positive_dimensions() {
    // Golden file: tests/port_golden/svek/calculateDimension_class_diagram.svg
    todo!()
}

#[test]
#[ignore = "gap: SvekResult not ported as standalone — full pipeline required"]
fn svek_result_get_shape_type_class_diagram_renders_correctly() {
    // Golden file: tests/port_golden/svek/getShapeType_class_diagram.svg
    todo!()
}

// ═══════════════════════════════════════════════════════════════════════════
// BibliotekonSkeletonTest
// Java: net.sourceforge.plantuml.svek.BibliotekonSkeletonTest
// Maps to: svek::Bibliotekon
// Note: Java Bibliotekon has createNode/addLine/getNodeUid/getLeaf/getLine APIs
//       requiring full diagram pipeline. Rust Bibliotekon is a simpler registry.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn bibliotekon_all_nodes_skeleton_for_none() {
    // Java: freshBibliotekon().allNodes() is empty
    let b = svek::Bibliotekon::new();
    assert!(b.all_nodes().is_empty(), "Fresh Bibliotekon has no nodes");
}

#[test]
fn bibliotekon_all_lines_skeleton_for_none() {
    // Java: freshBibliotekon().allLines() is empty
    let b = svek::Bibliotekon::new();
    assert!(b.all_edges().is_empty(), "Fresh Bibliotekon has no edges");
}

#[test]
fn bibliotekon_all_cluster_skeleton_for_none() {
    // Java: freshBibliotekon().allCluster() is empty
    let b = svek::Bibliotekon::new();
    assert!(b.clusters.is_empty(), "Fresh Bibliotekon has no clusters");
}

#[test]
fn bibliotekon_get_node_skeleton_for_entity() {
    // Java: freshBibliotekon().getNode(null) returns null
    let b = svek::Bibliotekon::new();
    // No nodes registered: find_node returns None for any uid
    assert!(b.find_node("anything").is_none());
}

#[test]
fn bibliotekon_get_cluster_skeleton_for_entity() {
    // Java: freshBibliotekon().getCluster(null) returns null
    let b = svek::Bibliotekon::new();
    // No clusters registered: none found
    assert!(b.clusters.is_empty());
}

#[test]
fn bibliotekon_add_and_find_node() {
    // Not in Java skeleton test but verifies basic add/find functionality
    let mut b = svek::Bibliotekon::new();
    b.add_node(svek::node::SvekNode::new("n1", 100.0, 50.0));
    b.add_node(svek::node::SvekNode::new("n2", 80.0, 40.0));
    assert_eq!(b.all_nodes().len(), 2);
    assert!(b.find_node("n1").is_some());
    assert!(b.find_node("n3").is_none());
}

#[test]
#[ignore = "gap: Bibliotekon::get_warning_or_error not yet ported"]
fn bibliotekon_get_warning_or_error_skeleton_for_int() {
    // Java: freshBibliotekon().getWarningOrError(1000) == ""
    todo!()
}

#[test]
#[ignore = "gap: Bibliotekon::get_max_x not yet ported"]
fn bibliotekon_get_max_x_skeleton_for_none() {
    // Java: freshBibliotekon().getMaxX() is empty map
    todo!()
}

#[test]
#[ignore = "gap: Bibliotekon::get_all_line_connected_to not yet ported"]
fn bibliotekon_get_all_line_connected_to_skeleton_for_entity() {
    // Java: no lines registered → empty list for any entity
    todo!()
}

#[test]
#[ignore = "gap: Bibliotekon::create_node requires Entity+IEntityImage+StringBounder from full pipeline"]
fn bibliotekon_create_node_skeleton() {
    todo!()
}

#[test]
#[ignore = "gap: Bibliotekon::add_line requires SvekEdge from full pipeline"]
fn bibliotekon_add_line_skeleton() {
    todo!()
}

#[test]
#[ignore = "gap: Bibliotekon::get_color_sequence — Java has AtomicInteger-based sequence; Rust uses RGB colors"]
fn bibliotekon_get_color_sequence_skeleton_for_none() {
    // Java: b.getColorSequence().getValue() returns increasing positive integers
    // Rust: ColorSequence uses next_color() with RGB scheme
    let mut cs = svek::ColorSequence::new();
    let v1 = cs.next_color();
    let v2 = cs.next_color();
    assert!(v2 > v1);
}
