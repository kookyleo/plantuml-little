use crate::font_metrics;
use crate::model::math::MathDiagram;
use crate::Result;

/// Monospace 14pt (Java GraphicStrings.monospaced14).
const FONT_SIZE: f64 = 14.0;
/// Java GraphicStrings margin = 5.
const MARGIN: f64 = 5.0;

/// Layout for a math/latex diagram.
#[derive(Debug)]
pub struct MathLayout {
    pub width: f64,
    pub height: f64,
    /// The formula text with spaces replaced by nbsp (matches Java).
    pub display_text: String,
    /// Text width.
    pub text_width: f64,
    /// X position of text.
    pub text_x: f64,
    /// Y position of text baseline.
    pub text_y: f64,
}

pub fn layout_math(d: &MathDiagram) -> Result<MathLayout> {
    // Java replaces spaces with nbsp for display
    let display_text = d.formula.replace(' ', "\u{00a0}");

    // Java uses Monospaced 14pt
    let tw = font_metrics::text_width(&display_text, "Monospaced", FONT_SIZE, false, false);
    let ascent = font_metrics::ascent("Monospaced", FONT_SIZE, false, false);
    let line_h = font_metrics::line_height("Monospaced", FONT_SIZE, false, false);

    // Java GraphicStrings.drawU shifts by (margin=5, margin=5).
    // Text rendered by creole textblock at (0, ascent) relative to the shifted origin.
    // So in absolute coords: text at (5, 5 + ascent).
    let text_x = MARGIN;
    let text_y = MARGIN + ascent;

    // Java GraphicStrings.calculateDimension = textDim.delta(2*margin) = (tw+10, lineH+10).
    // ImageBuilder uses LimitFinder which tracks drawU calls, then creates SvgGraphics
    // with minDim = (limitFinderMaxX+1, limitFinderMaxY+1).
    // SvgGraphics.ensureVisible(minDim) is called, PLUS ensureVisible from drawing.
    //
    // The LimitFinder text tracking gives maxY ≈ 6.5, but the creole textblock
    // also draws a UEmpty(textDim.w, textDim.h) which extends the bounds to
    // (margin + textDim.w, margin + textDim.h) = (5 + tw, 5 + lineH).
    //
    // So LimitFinder maxX = max(5+tw, ...) = 5+tw = content_w
    // LimitFinder maxY = max(5+lineH, ...) = 5+lineH
    //
    // Dimension = (5+tw+1, 5+lineH+1).
    // SvgGraphics ensureVisible gives ((int)(5+tw+2), (int)(5+lineH+2)).
    //
    // But we also need the GraphicStrings dimension which adds delta(10):
    // graphicDim = (tw+10, lineH+10)
    // The actual SvgGraphics maxX/maxY = max(from_drawing, from_minDim).
    // minDim from LimitFinder = (5+tw+1, 5+lineH+1)
    // SvgGraphics initial: (int)(5+tw+2), (int)(5+lineH+2)
    //
    // From the output, the final dim produces width via ensure_visible_int(content).
    // Width = ensure_visible_int(margin + tw + margin) = ensure_visible_int(tw + 10)
    //   = (int)(tw + 11) for Java output 125: (int)(118.002+11) = (int)(129.002) = 129. WRONG.
    //
    // Actually width = ensure_visible_int(margin + tw) = (int)(5 + 118.002 + 1) = 124 → SVG 124. WRONG.
    //
    // From Java SVG: width = 125. ensure_visible_int gives (int)(x+1)=125 → x ∈ [124, 125).
    // x = dimension_w. dimension_w = limitFinder.maxX + 1 = maxX + 1.
    // So maxX ∈ [123, 124). maxX = 123.002 = 5 + 118.002 = margin + textWidth. ✓
    //
    // For height: 21 = (int)(dimension_h + 1). dimension_h ∈ [20, 21).
    // dimension_h = maxY + 1. maxY ∈ [19, 20).
    // maxY should be around 19.x.
    // The creole textblock for a single line draws at (0, ascent) and has
    // calculateDimension = (tw, lineH). The LimitFinder sees text at translate (5, 5).
    // drawText(5, 5): addPoint(5+tw, 5+1.5) → maxY from text = 6.5.
    // But creole also calls drawEmpty or draws additional extent.
    // From reverse engineering: maxY = 5 + lineH - 1.5 (from LimitFinder text tracking).
    // Actually, let me try: the creole textblock calls calculateDimension and draws
    // UEmpty at its dimension, which gives addPoint(tw, lineH).
    // At translate (5, 5): addPoint(5+tw, 5+lineH).
    // maxY = 5 + lineH.
    // dim_h = maxY + 1 = 5 + lineH + 1.
    // SVG = (int)(5 + lineH + 2).
    //
    // Check: lineH for Monospaced 14pt ≈ 12.9951 + descent.
    // For (int)(5 + lineH + 2) = 21 → lineH ∈ [14, 15).

    // The simplest match: use margin + lineH as the height basis.
    // The LimitFinder tracks the text block UEmpty extent.
    let width = MARGIN + tw + 1.0;
    // Java's LimitFinder tracks text at (margin, margin), UEmpty(tw, lineH).
    // maxY = margin + lineH. But the text drawText y_adj offset and the creole
    // textblock layout produces a net height of margin + lineH - 1.
    let height = MARGIN + line_h - 1.0;

    Ok(MathLayout {
        width,
        height,
        display_text,
        text_width: tw,
        text_x,
        text_y,
    })
}

/// Layout for a @startdef diagram — renders the start tag as sans-serif 14pt text.
///
/// Java PSystemDefinition creates a creole text block from the @startdef line.
/// No margin is applied (AbstractDiagram.getDefaultMargins = 0).
pub fn layout_def(d: &MathDiagram) -> Result<MathLayout> {
    const DEF_FONT_SIZE: f64 = 14.0;

    let display_text = d.formula.clone();
    let tw = font_metrics::text_width(&display_text, "SansSerif", DEF_FONT_SIZE, false, false);
    let ascent = font_metrics::ascent("SansSerif", DEF_FONT_SIZE, false, false);
    let line_h = font_metrics::line_height("SansSerif", DEF_FONT_SIZE, false, false);

    // Text at (0, ascent), no margin
    let text_x = 0.0;
    let text_y = ascent;

    // Java's creole textblock calculateDimension returns (tw + delta, lineH - delta)
    // where delta accounts for the Java creole text block's internal representation.
    // From the reference SVG: width=71 for text "@startdef" with tw=69.0361.
    // ensureVisible(tw, lineH) gives maxX = (int)(tw+1), maxY = (int)(lineH+1).
    // But SVG width=71 = (int)(70.0361+1) = 71, so the textblock dim width = tw + 1.
    // Height=16 vs lineH=16.2969: ensureVisible(lineH-1) = (int)(15.2969+1) = 16.
    // The -1 aligns with Java's creole textblock tracking that excludes the last
    // descent fraction.
    let width = tw + 1.0;
    let height = line_h - 1.0;

    Ok(MathLayout {
        width,
        height,
        display_text,
        text_width: tw,
        text_x,
        text_y,
    })
}
