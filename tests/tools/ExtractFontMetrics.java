import java.awt.*;
import java.awt.font.FontRenderContext;
import java.awt.font.LineMetrics;
import java.awt.image.BufferedImage;
import java.io.PrintStream;

/**
 * Extract precise font metrics from Java AWT, matching PlantUML's
 * StringBounder configuration.
 *
 * Usage:
 *   javac tests/tools/ExtractFontMetrics.java
 *   java -cp tests/tools ExtractFontMetrics > tests/tools/font_metrics.json
 *
 * PlantUML uses:
 *   - Font: "SansSerif" (Java logical font)
 *   - RenderingHints: FRACTIONALMETRICS_ON, TEXT_ANTIALIAS_ON
 *   - FontRenderContext from BufferedImage(100, 100, TYPE_INT_RGB)
 */
public class ExtractFontMetrics {

    // Font sizes used by PlantUML (from FontParam.java)
    static final int[] SIZES = {10, 11, 12, 13, 14, 17, 18};

    // Font families used by PlantUML
    static final String[] FAMILIES = {"SansSerif", "Monospaced"};

    // Font styles
    static final int[] STYLES = {Font.PLAIN, Font.BOLD, Font.ITALIC, Font.BOLD | Font.ITALIC};
    static final String[] STYLE_NAMES = {"plain", "bold", "italic", "bold_italic"};

    public static void main(String[] args) {
        // Create graphics context matching PlantUML's StringBounderRaw
        BufferedImage img = new BufferedImage(100, 100, BufferedImage.TYPE_INT_RGB);
        Graphics2D g2d = img.createGraphics();
        g2d.setRenderingHint(RenderingHints.KEY_FRACTIONALMETRICS,
                             RenderingHints.VALUE_FRACTIONALMETRICS_ON);
        g2d.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING,
                             RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
        FontRenderContext frc = g2d.getFontRenderContext();

        PrintStream out = System.out;
        out.println("{");

        boolean firstFamily = true;
        for (String family : FAMILIES) {
            if (!firstFamily) out.println(",");
            firstFamily = false;
            out.printf("  \"%s\": {%n", family);

            boolean firstSize = true;
            for (int size : SIZES) {
                if (!firstSize) out.println(",");
                firstSize = false;
                out.printf("    \"%d\": {%n", size);

                boolean firstStyle = true;
                for (int si = 0; si < STYLES.length; si++) {
                    if (!firstStyle) out.println(",");
                    firstStyle = false;

                    Font font = new Font(family, STYLES[si], size);
                    FontMetrics fm = g2d.getFontMetrics(font);
                    LineMetrics lm = font.getLineMetrics("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz", frc);

                    out.printf("      \"%s\": {%n", STYLE_NAMES[si]);

                    // Vertical metrics
                    out.printf("        \"ascent\": %.6f,%n", lm.getAscent());
                    out.printf("        \"descent\": %.6f,%n", lm.getDescent());
                    out.printf("        \"leading\": %.6f,%n", lm.getLeading());
                    out.printf("        \"height\": %.6f,%n", lm.getHeight());

                    // Per-character advance widths
                    out.println("        \"advances\": {");
                    boolean firstChar = true;

                    // Printable ASCII: U+0020 to U+007E
                    for (int cp = 0x20; cp <= 0x7E; cp++) {
                        String ch = String.valueOf((char) cp);
                        double width = font.getStringBounds(ch, frc).getWidth();

                        if (!firstChar) out.println(",");
                        firstChar = false;

                        // Escape special JSON characters
                        String escaped = ch;
                        if (cp == '"') escaped = "\\\"";
                        else if (cp == '\\') escaped = "\\\\";

                        out.printf("          \"%s\": %.6f", escaped, width);
                    }

                    // Common non-ASCII characters
                    int[][] extraRanges = {
                        {0x00A0, 0x00FF},  // Latin-1 Supplement (includes NBSP)
                        {0x2000, 0x206F},  // General Punctuation
                        {0x2190, 0x21FF},  // Arrows
                        {0x2500, 0x257F},  // Box Drawing
                        {0x25A0, 0x25FF},  // Geometric Shapes
                        {0x2600, 0x26FF},  // Miscellaneous Symbols
                    };

                    for (int[] range : extraRanges) {
                        for (int cp = range[0]; cp <= range[1]; cp++) {
                            if (!font.canDisplay(cp)) continue;
                            String ch = String.valueOf((char) cp);
                            double width = font.getStringBounds(ch, frc).getWidth();

                            out.println(",");
                            // Use Unicode escape for non-ASCII
                            out.printf("          \"\\u%04X\": %.6f", cp, width);
                        }
                    }

                    out.println();
                    out.println("        }");
                    out.printf("      }");
                }

                out.println();
                out.printf("    }");
            }

            out.println();
            out.printf("  }");
        }

        out.println();
        out.println("}");

        g2d.dispose();
    }
}
