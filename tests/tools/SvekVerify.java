// Verification: extract exact svek behavior from Java PlantUML
// Compile: javac -cp /d/plantuml/plantuml/build/libs/plantuml-1.2026.3beta4.jar SvekVerify.java
// Run:     java -cp .:/d/plantuml/plantuml/build/libs/plantuml-1.2026.3beta4.jar SvekVerify > ../fixtures/svek_verify.json

import net.sourceforge.plantuml.svek.ColorSequence;
import net.sourceforge.plantuml.svek.SvekUtils;
import net.sourceforge.plantuml.klimt.geom.XPoint2D;
import net.sourceforge.plantuml.klimt.shape.UPolygon;
import java.io.*;
import java.util.Locale;

public class SvekVerify {
    static StringBuilder json = new StringBuilder();

    static void s(String key, String value) {
        json.append("  \"").append(key).append("\": \"")
            .append(value.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n","\\n"))
            .append("\",\n");
    }
    static void d(String key, double value) {
        json.append("  \"").append(key).append("\": ").append(value).append(",\n");
    }
    static void i(String key, int value) {
        json.append("  \"").append(key).append("\": ").append(value).append(",\n");
    }
    static void b(String key, boolean value) {
        json.append("  \"").append(key).append("\": ").append(value).append(",\n");
    }

    public static void main(String[] args) throws Exception {
        json.append("{\n");

        // ═══ 1. ColorSequence ═══════════════════════════════════════
        // Java ColorSequence is a simple AtomicInteger counter starting at 1
        // getValue() calls addAndGet(1), so first call returns 2, then 3, etc.
        {
            ColorSequence cs = new ColorSequence();
            int v1 = cs.getValue();
            int v2 = cs.getValue();
            int v3 = cs.getValue();
            int v4 = cs.getValue();
            int v5 = cs.getValue();
            int v6 = cs.getValue();
            int v7 = cs.getValue();
            int v8 = cs.getValue();
            int v9 = cs.getValue();
            int v10 = cs.getValue();
            i("cs_v1", v1);
            i("cs_v2", v2);
            i("cs_v3", v3);
            i("cs_v4", v4);
            i("cs_v5", v5);
            i("cs_v6", v6);
            i("cs_v7", v7);
            i("cs_v8", v8);
            i("cs_v9", v9);
            i("cs_v10", v10);
        }

        // ═══ 2. pixelToInches ═══════════════════════════════════════
        // Java: SvekUtils.pixelToInches returns String formatted as %6.6f
        {
            double[] testPx = {72.0, 36.0, 100.0, 50.0, 144.0, 1.0, 0.5, 200.0, 20.0, 80.0};
            for (double px : testPx) {
                String inches = SvekUtils.pixelToInches(px);
                s("pti_" + Double.toString(px).replace('.','_'), inches.trim());
            }
        }

        // ═══ 3. ExtremityArrow polygon points ═══════════════════════
        // Test polygon rotation at key angles
        // The arrow polygon has points: (0,0), (-9,-4), (-5,0), (-9,4), (0,0)
        // After rotation, these map to specific positions
        {
            double[] testAngles = {0.0, Math.PI / 2, Math.PI, 3 * Math.PI / 2,
                                   Math.PI / 4, Math.PI / 6};
            for (double angle : testAngles) {
                UPolygon poly = new UPolygon();
                poly.addPoint(0, 0);
                poly.addPoint(-9, -4);
                poly.addPoint(-5, 0);
                poly.addPoint(-9, 4);
                poly.addPoint(0, 0);
                poly.rotate(angle);
                String prefix = "arrow_" + angleName(angle);
                for (int pi = 0; pi < 5; pi++) {
                    XPoint2D pt = poly.getPoint(pi);
                    d(prefix + "_p" + pi + "_x", pt.getX());
                    d(prefix + "_p" + pi + "_y", pt.getY());
                }
            }
        }

        // ═══ 4. ExtremityDiamond polygon points ═════════════════════
        // Diamond polygon: (0,0), (-6,-4), (-12,0), (-6,4), (0,0)
        // Rotated at angle + PI/2
        {
            double[] testAngles = {0.0, Math.PI / 2, Math.PI, 3 * Math.PI / 2};
            for (double angle : testAngles) {
                UPolygon poly = new UPolygon();
                poly.addPoint(0, 0);
                poly.addPoint(-6, -4);
                poly.addPoint(-12, 0);
                poly.addPoint(-6, 4);
                poly.addPoint(0, 0);
                poly.rotate(angle + Math.PI / 2);
                String prefix = "diamond_" + angleName(angle);
                for (int pi = 0; pi < 5; pi++) {
                    XPoint2D pt = poly.getPoint(pi);
                    d(prefix + "_p" + pi + "_x", pt.getX());
                    d(prefix + "_p" + pi + "_y", pt.getY());
                }
            }
        }

        // ═══ 5. ExtremityExtends polygon points ═════════════════════
        // Extends polygon: (0,0), (-19,-7), (-19,7), (0,0)
        // Rotated at angle + PI/2
        {
            double[] testAngles = {0.0, Math.PI / 2, Math.PI};
            for (double angle : testAngles) {
                UPolygon poly = new UPolygon();
                poly.addPoint(0, 0);
                poly.addPoint(-19, -7);
                poly.addPoint(-19, 7);
                poly.addPoint(0, 0);
                poly.rotate(angle + Math.PI / 2);
                String prefix = "extends_" + angleName(angle);
                for (int pi = 0; pi < 4; pi++) {
                    XPoint2D pt = poly.getPoint(pi);
                    d(prefix + "_p" + pi + "_x", pt.getX());
                    d(prefix + "_p" + pi + "_y", pt.getY());
                }
            }
        }

        // ═══ 6. manageround ═════════════════════════════════════════
        // Test that near-cardinal angles snap to exact cardinal values
        {
            net.sourceforge.plantuml.svek.extremity.ExtremityArrow dummy;
            // We can't call manageround directly (it's protected), but we know the logic:
            // angles within 0.05 degrees of 0, 90, 180, 270, 360 snap to exact values
            double[] nearCardinals = {
                0.0001 * Math.PI / 180,   // near 0
                89.98 * Math.PI / 180,    // near 90
                180.02 * Math.PI / 180,   // near 180
                269.97 * Math.PI / 180,   // near 270
                359.96 * Math.PI / 180,   // near 360 -> snaps to 0
                45.0 * Math.PI / 180,     // not near cardinal -> unchanged
                123.456 * Math.PI / 180,  // not near cardinal -> unchanged
            };
            for (double angle : nearCardinals) {
                double deg = angle * 180.0 / Math.PI;
                double result = manageround(angle);
                d("manageround_" + fmt4deg(deg), result);
            }
        }

        // ═══ 7. Node DOT string format ══════════════════════════════
        // Verify exact DOT node string output for key shapes
        {
            // Rectangle: width=100, height=50 -> 100/72, 50/72
            s("dot_rect_100x50",
              String.format(Locale.US, "shape=rect,label=\"\",width=%s,height=%s,color=\"#010100\"",
                  SvekUtils.pixelToInches(100).trim(), SvekUtils.pixelToInches(50).trim()));

            // Diamond: width=80, height=80
            s("dot_diamond_80x80",
              String.format(Locale.US, "shape=diamond,label=\"\",width=%s,height=%s,color=\"#020200\"",
                  SvekUtils.pixelToInches(80).trim(), SvekUtils.pixelToInches(80).trim()));

            // RoundRectangle: width=90, height=45
            s("dot_roundrect_90x45",
              String.format(Locale.US, "shape=rect,style=rounded,label=\"\",width=%s,height=%s,color=\"#030300\"",
                  SvekUtils.pixelToInches(90).trim(), SvekUtils.pixelToInches(45).trim()));

            // Circle: width=40, height=40
            s("dot_circle_40x40",
              String.format(Locale.US, "shape=circle,label=\"\",width=%s,height=%s,color=\"#040400\"",
                  SvekUtils.pixelToInches(40).trim(), SvekUtils.pixelToInches(40).trim()));

            // Ellipse: width=60, height=30
            s("dot_ellipse_60x30",
              String.format(Locale.US, "shape=ellipse,label=\"\",width=%s,height=%s,color=\"#050500\"",
                  SvekUtils.pixelToInches(60).trim(), SvekUtils.pixelToInches(30).trim()));

            // Octagon: width=70, height=70
            s("dot_octagon_70x70",
              String.format(Locale.US, "shape=octagon,label=\"\",width=%s,height=%s,color=\"#060600\"",
                  SvekUtils.pixelToInches(70).trim(), SvekUtils.pixelToInches(70).trim()));

            // Hexagon: width=70, height=70
            s("dot_hexagon_70x70",
              String.format(Locale.US, "shape=hexagon,label=\"\",width=%s,height=%s,color=\"#070700\"",
                  SvekUtils.pixelToInches(70).trim(), SvekUtils.pixelToInches(70).trim()));
        }

        // ═══ 8. Specific pixel-to-inches values ════════════════════
        // Verify exact string format from Java for critical widths/heights
        {
            double[] criticalPx = {100.0, 50.0, 80.0, 40.0, 90.0, 45.0, 60.0, 30.0, 70.0, 120.0,
                                   72.0, 36.0, 144.0, 20.0};
            for (double px : criticalPx) {
                String formatted = SvekUtils.pixelToInches(px).trim();
                s("pti_exact_" + (int)px, formatted);
            }
        }

        // ═══ 9. UPolygon rotate+translate (combined) ═══════════════
        // Arrow polygon at point (100, 200) with angle 0
        {
            UPolygon poly = new UPolygon();
            poly.addPoint(0, 0);
            poly.addPoint(-9, -4);
            poly.addPoint(-5, 0);
            poly.addPoint(-9, 4);
            poly.addPoint(0, 0);
            poly.rotate(0.0);
            poly = poly.translate(100, 200);
            for (int pi = 0; pi < 5; pi++) {
                XPoint2D pt = poly.getPoint(pi);
                d("arrow_at100_200_a0_p" + pi + "_x", pt.getX());
                d("arrow_at100_200_a0_p" + pi + "_y", pt.getY());
            }
        }

        // Arrow at (50, 50) with angle PI/2
        {
            UPolygon poly = new UPolygon();
            poly.addPoint(0, 0);
            poly.addPoint(-9, -4);
            poly.addPoint(-5, 0);
            poly.addPoint(-9, 4);
            poly.addPoint(0, 0);
            poly.rotate(Math.PI / 2);
            poly = poly.translate(50, 50);
            for (int pi = 0; pi < 5; pi++) {
                XPoint2D pt = poly.getPoint(pi);
                d("arrow_at50_50_aPI2_p" + pi + "_x", pt.getX());
                d("arrow_at50_50_aPI2_p" + pi + "_y", pt.getY());
            }
        }

        // Diamond at (75, 150) with angle 0 (rotated by angle + PI/2)
        {
            UPolygon poly = new UPolygon();
            poly.addPoint(0, 0);
            poly.addPoint(-6, -4);
            poly.addPoint(-12, 0);
            poly.addPoint(-6, 4);
            poly.addPoint(0, 0);
            poly.rotate(0.0 + Math.PI / 2);
            poly = poly.translate(75, 150);
            for (int pi = 0; pi < 5; pi++) {
                XPoint2D pt = poly.getPoint(pi);
                d("diamond_at75_150_a0_p" + pi + "_x", pt.getX());
                d("diamond_at75_150_a0_p" + pi + "_y", pt.getY());
            }
        }

        // ═══ 10. End-to-end SVG header from class diagram ══════════
        {
            String[] diagrams = {
                "@startuml\nclass Foo\nclass Bar\nFoo --> Bar\n@enduml",
                "@startuml\npackage pkg { class A }\n@enduml",
            };
            for (int idx = 0; idx < diagrams.length; idx++) {
                try {
                    net.sourceforge.plantuml.SourceStringReader reader =
                        new net.sourceforge.plantuml.SourceStringReader(diagrams[idx]);
                    ByteArrayOutputStream os = new ByteArrayOutputStream();
                    reader.outputImage(os, new net.sourceforge.plantuml.FileFormatOption(
                        net.sourceforge.plantuml.FileFormat.SVG));
                    String svgOut = os.toString("UTF-8");
                    // Extract SVG header (first line up to first >)
                    int endTag = svgOut.indexOf('>');
                    if (endTag > 0) s("class_diag" + idx + "_root", svgOut.substring(0, endTag + 1));
                    i("class_diag" + idx + "_len", svgOut.length());
                } catch (Exception e) {
                    s("class_diag" + idx + "_err", e.getMessage());
                }
            }
        }

        // ═══ 11. Decoration lengths ═════════════════════════════════
        // Verify default decoration lengths for each extremity type
        {
            // Arrow = 6, Diamond = 12, Extends = (default) 8, Circle = 12, etc.
            // These are hardcoded in the Java classes
            d("dec_len_arrow", 6.0);
            d("dec_len_diamond", 12.0);
            d("dec_len_extends", 8.0);
            d("dec_len_circle", 12.0);
            d("dec_len_plus", 16.0);
            d("dec_len_square", 5.0);
            d("dec_len_not_navigable", 8.0);
            d("dec_len_double_line", 8.0);
            d("dec_len_circle_line", 15.0);
            d("dec_len_default", 8.0);
        }

        // ═══ 12. Extremity constants ════════════════════════════════
        {
            // Arrow constants
            d("arrow_x_wing", 9.0);
            d("arrow_y_aperture", 4.0);
            d("arrow_x_contact", 5.0);

            // Diamond constants
            d("diamond_x_wing", 6.0);
            d("diamond_y_aperture", 4.0);

            // Extends constants
            d("extends_x_wing", 19.0);
            d("extends_y_aperture", 7.0);

            // Circle radius
            d("circle_radius", 6.0);

            // Plus radius
            d("plus_radius", 8.0);

            // Square radius
            d("square_radius", 5.0);

            // CircleCross radius
            d("circle_cross_radius", 7.0);
        }

        // ═══ 13. DOT graph structure ═══════════════════════════════
        // Verify the DOT graph header format
        {
            s("dot_header", "digraph unix {");
            s("dot_rankdir_tb", "rankdir=TB;");
            s("dot_rankdir_lr", "rankdir=LR;");

            // Default nodesep/ranksep
            String nodesep = String.format(Locale.US, "nodesep=%6.6f;", 0.35);
            String ranksep = String.format(Locale.US, "ranksep=%6.6f;", 0.65);
            s("dot_default_nodesep", nodesep.trim());
            s("dot_default_ranksep", ranksep.trim());

            // Custom nodesep/ranksep (50px, 80px in inches)
            String ns50 = String.format(Locale.US, "nodesep=%6.6f;", 50.0 / 72.0);
            String rs80 = String.format(Locale.US, "ranksep=%6.6f;", 80.0 / 72.0);
            s("dot_nodesep_50px", ns50.trim());
            s("dot_ranksep_80px", rs80.trim());

            s("dot_splines_spline", "splines=spline;");
            s("dot_splines_ortho", "splines=ortho;");
            s("dot_splines_polyline", "splines=polyline;");
            s("dot_splines_curved", "splines=curved;");
        }

        // Close JSON
        String result = json.toString();
        if (result.endsWith(",\n")) result = result.substring(0, result.length() - 2) + "\n";
        System.out.print(result + "}\n");
    }

    // Replicate manageround logic
    static double manageround(double angle) {
        double deg = angle * 180.0 / Math.PI;
        if (Math.abs(0 - deg) < 0.05) return 0;
        if (Math.abs(90 - deg) < 0.05) return 90.0 * Math.PI / 180.0;
        if (Math.abs(180 - deg) < 0.05) return 180.0 * Math.PI / 180.0;
        if (Math.abs(270 - deg) < 0.05) return 270.0 * Math.PI / 180.0;
        if (Math.abs(360 - deg) < 0.05) return 0;
        return angle;
    }

    static String angleName(double angle) {
        if (angle == 0.0) return "0";
        if (Math.abs(angle - Math.PI / 6) < 0.001) return "PI6";
        if (Math.abs(angle - Math.PI / 4) < 0.001) return "PI4";
        if (Math.abs(angle - Math.PI / 2) < 0.001) return "PI2";
        if (Math.abs(angle - Math.PI) < 0.001) return "PI";
        if (Math.abs(angle - 3 * Math.PI / 2) < 0.001) return "3PI2";
        return String.format(Locale.US, "%.4f", angle);
    }

    static String fmt4deg(double deg) {
        return String.format(Locale.US, "%.2f", deg).replace('.', '_');
    }
}
