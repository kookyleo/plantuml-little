// Verification: extract exact klimt behavior from Java PlantUML
// Compile: javac -cp /d/plantuml/plantuml/build/libs/plantuml-1.2026.3beta4.jar KlimtVerify.java
// Run:     java -cp .:/d/plantuml/plantuml/build/libs/plantuml-1.2026.3beta4.jar KlimtVerify > klimt_verify.json

import net.sourceforge.plantuml.klimt.*;
import net.sourceforge.plantuml.klimt.color.*;
import net.sourceforge.plantuml.klimt.geom.*;
import net.sourceforge.plantuml.klimt.shape.*;
import java.io.*;
import java.util.Locale;

public class KlimtVerify {
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

        // ═══ 1. Color resolution (HColorSet) ═══════════════════════
        HColorSet cs = HColorSet.instance();
        String[] names = {"red","blue","green","yellow","black","white",
            "LightBlue","DarkSalmon","Gold","Navy","Crimson","Lavender",
            "Aquamarine","Chocolate","Indigo","Lime","Olive","Teal",
            "transparent","APPLICATION","BUSINESS","TECHNOLOGY"};
        for (String n : names) {
            try {
                HColor c = cs.getColor(n);
                s("color_" + n, c.toSvg(ColorMapper.IDENTITY));
            } catch (Exception e) {
                s("color_" + n, "NONE");
            }
        }

        // ═══ 2. HColor methods ═════════════════════════════════════
        HColor red = cs.getColor("red");
        HColor white = cs.getColor("white");
        HColor black = cs.getColor("black");
        // midgray skipped - no public constructor

        // isDark
        b("black_is_dark", black.isDark());
        b("white_is_dark", white.isDark());
        b("red_is_dark", red.isDark());

        // ═══ 3. UStroke ════════════════════════════════════════════
        UStroke dashed = new UStroke(5.0, 5.0, 1.0);
        d("stroke_dash_vis", dashed.getDashVisible());
        d("stroke_dash_spc", dashed.getDashSpace());
        d("stroke_dash_thick", dashed.getThickness());
        double[] da = dashed.getDasharraySvg();
        d("stroke_dasharray_0", da[0]);
        d("stroke_dasharray_1", da[1]);

        UStroke solid = UStroke.withThickness(1.5);
        b("stroke_solid_no_dash", solid.getDasharraySvg() == null);
        d("stroke_solid_thick", solid.getThickness());

        UStroke simple = UStroke.simple();
        d("stroke_simple_thick", simple.getThickness());

        // ═══ 4. UTranslate ════════════════════════════════════════
        UTranslate t1 = new UTranslate(10, 20);
        UTranslate t2 = new UTranslate(5, -3);
        UTranslate t3 = t1.compose(t2);
        d("tr_compose_dx", t3.getDx());
        d("tr_compose_dy", t3.getDy());
        UTranslate t4 = t1.reverse();
        d("tr_reverse_dx", t4.getDx());
        d("tr_reverse_dy", t4.getDy());
        UTranslate t5 = t1.scaled(2.0);
        d("tr_scaled_dx", t5.getDx());
        d("tr_scaled_dy", t5.getDy());

        // ═══ 5. XPoint2D ══════════════════════════════════════════
        d("pt_dist_3_4", XPoint2D.distance(0, 0, 3, 4));
        XPoint2D p1 = new XPoint2D(10, 20);
        XPoint2D p2 = new XPoint2D(13, 24);
        d("pt_dist_p1_p2", p1.distance(p2));

        // ═══ 6. XDimension2D ══════════════════════════════════════
        XDimension2D da1 = new XDimension2D(100, 50);
        XDimension2D da2 = new XDimension2D(80, 30);
        d("dim_tb_w", da1.mergeTB(da2).getWidth());
        d("dim_tb_h", da1.mergeTB(da2).getHeight());
        d("dim_lr_w", da1.mergeLR(da2).getWidth());
        d("dim_lr_h", da1.mergeLR(da2).getHeight());
        d("dim_max_w", XDimension2D.max(da1, da2).getWidth());
        d("dim_max_h", XDimension2D.max(da1, da2).getHeight());
        d("dim_delta_w", da1.delta(10, -5).getWidth());
        d("dim_delta_h", da1.delta(10, -5).getHeight());

        // ═══ 7. XLine2D ═══════════════════════════════════════════
        XLine2D line = XLine2D.line(new XPoint2D(0, 0), new XPoint2D(10, 0));
        XPoint2D mid = line.getMiddle();
        d("line_mid_x", mid.getX());
        d("line_mid_y", mid.getY());
        d("line_angle_horiz", line.getAngle());

        XLine2D line2 = XLine2D.line(new XPoint2D(0, 0), new XPoint2D(0, 10));
        d("line_angle_vert", line2.getAngle());

        // ═══ 8. URectangle ═════════════════════════════════════════
        URectangle rect = URectangle.build(100, 50);
        d("rect_w", rect.getWidth());
        d("rect_h", rect.getHeight());
        URectangle rr = rect.rounded(10);
        d("rect_rx", rr.getRx());
        d("rect_ry", rr.getRy());

        // ═══ 9. UEllipse ═══════════════════════════════════════════
        UEllipse ell = UEllipse.build(80, 60);
        d("ell_w", ell.getWidth());
        d("ell_h", ell.getHeight());
        XPoint2D ep = ell.getPointAtAngle(0);
        d("ell_pt0_x", ep.getX());
        d("ell_pt0_y", ep.getY());
        XPoint2D ep2 = ell.getPointAtAngle(Math.PI);
        d("ell_ptPI_x", ep2.getX());
        d("ell_ptPI_y", ep2.getY());

        // ═══ 10. ULine ════════════════════════════════════════════
        ULine ul = ULine.hline(100);
        d("uline_h_dx", ul.getDX());
        d("uline_h_dy", ul.getDY());
        ULine uv = ULine.vline(50);
        d("uline_v_dx", uv.getDX());
        d("uline_v_dy", uv.getDY());

        // ═══ 11. Number formatting (critical for SVG matching) ════
        // Replicate SvgGraphics format behavior
        double[] fmtTests = {0, 42, 1.5, 1.23456, -0.00004,
            30.2969, 47.667, 0.5, 24.9951, 33.667,
            114.5625, 36.2969, 167.6152, 5.0, 2.5,
            0.00000, 13.9688, 7.0, 1.0};
        for (double v : fmtTests) {
            s("fmt_" + Double.toString(v).replace('.','_').replace('-','m'), fmt4(v));
        }

        // ═══ 12. End-to-end small diagrams ════════════════════════
        String[] diagrams = {
            "@startuml\nAlice -> Bob: hello\n@enduml",
            "@startuml\nclass Foo\n@enduml",
            "@startuml\n[*] --> Active\n@enduml",
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
                if (endTag > 0) s("diag" + idx + "_root", svgOut.substring(0, endTag + 1));
                i("diag" + idx + "_len", svgOut.length());
            } catch (Exception e) {
                s("diag" + idx + "_err", e.getMessage());
            }
        }

        // Close JSON
        String result = json.toString();
        if (result.endsWith(",\n")) result = result.substring(0, result.length() - 2) + "\n";
        System.out.print(result + "}\n");
    }

    static String fmt4(double v) {
        if (v == 0) return "0";
        String s = String.format(Locale.US, "%.4f", v);
        s = s.replaceAll("0+$", "");
        s = s.replaceAll("\\.$", "");
        return s;
    }
}
