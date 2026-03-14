import java.awt.*;
import java.awt.font.*;
import java.awt.geom.*;
import java.awt.image.BufferedImage;

/**
 * Extract SVG path data for circled characters (C, I, E, A) at size 17,
 * relative to the circle center (0, 0).
 *
 * Usage: javac tests/tools/ExtractGlyphPaths.java -d tests/tools/
 *        java -cp tests/tools ExtractGlyphPaths
 */
public class ExtractGlyphPaths {
    public static void main(String[] args) {
        BufferedImage img = new BufferedImage(100, 100, BufferedImage.TYPE_INT_RGB);
        Graphics2D g2d = img.createGraphics();
        g2d.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

        // PlantUML uses Monospaced font at size 17 for circled characters
        Font font = new Font("Monospaced", Font.BOLD, 17);
        FontRenderContext frc = g2d.getFontRenderContext();

        String[] chars = {"C", "I", "E", "A"};

        for (String ch : chars) {
            GlyphVector gv = font.createGlyphVector(frc, ch);
            Shape shape = gv.getOutline();
            PathIterator pi = shape.getPathIterator(null);

            // Get glyph bounds to compute center offset
            Rectangle2D bounds = gv.getLogicalBounds();
            double glyphWidth = bounds.getWidth();
            // Center the glyph horizontally: offset = -glyphWidth/2
            // Vertically: Java renders with baseline at y=0

            System.out.println("// Glyph '" + ch + "' relative to draw position (0, 0)");
            System.out.println("// Bounds: " + bounds);
            System.out.print("const GLYPH_" + ch + "_REL: &[(u8, &[(f64, f64)])] = &[");

            double[] coords = new double[6];
            boolean first = true;
            while (!pi.isDone()) {
                int type = pi.currentSegment(coords);
                if (!first) System.out.print(", ");
                first = false;

                switch (type) {
                    case PathIterator.SEG_MOVETO:
                        System.out.printf("(b'M', &[(%.4f, %.4f)])", coords[0], coords[1]);
                        break;
                    case PathIterator.SEG_LINETO:
                        System.out.printf("(b'L', &[(%.4f, %.4f)])", coords[0], coords[1]);
                        break;
                    case PathIterator.SEG_QUADTO:
                        System.out.printf("(b'Q', &[(%.4f, %.4f), (%.4f, %.4f)])",
                            coords[0], coords[1], coords[2], coords[3]);
                        break;
                    case PathIterator.SEG_CUBICTO:
                        System.out.printf("(b'C', &[(%.4f, %.4f), (%.4f, %.4f), (%.4f, %.4f)])",
                            coords[0], coords[1], coords[2], coords[3], coords[4], coords[5]);
                        break;
                    case PathIterator.SEG_CLOSE:
                        System.out.print("(b'Z', &[])");
                        break;
                }
                pi.next();
            }
            System.out.println("];");
            System.out.println();
        }

        g2d.dispose();
    }
}
