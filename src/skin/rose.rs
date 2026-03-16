// skin::rose - The Rose theme (default PlantUML skin)
// Port of Java PlantUML's skin.rose package (26 files)
//
// Defines rendering constants, size calculations, and drawing instructions
// for all sequence diagram components: arrows, participants, notes,
// dividers, grouping headers, lifelines, activation boxes, etc.

use crate::klimt::color::HColor;
use crate::klimt::geom::{XDimension2D, XPoint2D};
use crate::klimt::shape::UPath;
use crate::klimt::{Fashion, UStroke, UTranslate};
use crate::skin::arrow::{
    ArrowBody, ArrowConfiguration, ArrowDecoration, ArrowDirection, ArrowHead, ArrowPart,
};

// ── Rose constants ──────────────────────────────────────────────────

/// Padding X used by the Rose factory. Java: `Rose.paddingX = 5`
pub const ROSE_PADDING_X: f64 = 5.0;
/// Padding Y used by the Rose factory. Java: `Rose.paddingY = 5`
pub const ROSE_PADDING_Y: f64 = 5.0;

// ── DrawOp: output-independent drawing instruction ──────────────────

/// A single drawing instruction produced by component rendering.
/// This is our lightweight alternative to Java's UGraphic visitor pattern.
/// The SVG/PNG renderer consumes these to produce output.
#[derive(Debug, Clone)]
pub enum DrawOp {
    /// Draw a rectangle at (translate), with given dimensions and rounding.
    Rect {
        translate: UTranslate,
        width: f64,
        height: f64,
        rx: f64,
        stroke: UStroke,
        color: Option<HColor>,
        fill: Option<HColor>,
        shadow: f64,
    },
    /// Draw a line from (translate) with relative (dx, dy).
    Line {
        translate: UTranslate,
        dx: f64,
        dy: f64,
        stroke: UStroke,
        color: Option<HColor>,
    },
    /// Draw a UPath at (translate).
    Path {
        translate: UTranslate,
        path: UPath,
        stroke: UStroke,
        color: Option<HColor>,
        fill: Option<HColor>,
        shadow: f64,
    },
    /// Draw a polygon (filled) at (translate).
    Polygon {
        translate: UTranslate,
        points: Vec<(f64, f64)>,
        stroke: UStroke,
        color: Option<HColor>,
        fill: Option<HColor>,
    },
    /// Draw an ellipse at (translate).
    Ellipse {
        translate: UTranslate,
        width: f64,
        height: f64,
        stroke: UStroke,
        color: Option<HColor>,
        fill: Option<HColor>,
    },
    /// Draw text at (translate).
    Text {
        translate: UTranslate,
        text: String,
        font_family: String,
        font_size: f64,
        bold: bool,
        italic: bool,
        color: Option<HColor>,
    },
}

// ── Area ────────────────────────────────────────────────────────────

/// The available area for a component to draw into.
/// Java: `skin.Area`
#[derive(Debug, Clone)]
pub struct Area {
    pub dimension: XDimension2D,
    pub delta_x1: f64,
    pub text_delta_x: f64,
    pub level: i32,
    pub live_delta_size: f64,
}

impl Area {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            dimension: XDimension2D::new(width, height),
            delta_x1: 0.0,
            text_delta_x: 0.0,
            level: 0,
            live_delta_size: 0.0,
        }
    }

    pub fn from_dim(dim: XDimension2D) -> Self {
        Self::new(dim.width, dim.height)
    }

    pub fn with_delta_x1(mut self, dx: f64) -> Self {
        self.delta_x1 = dx;
        self
    }

    pub fn with_text_delta_x(mut self, dx: f64) -> Self {
        self.text_delta_x = dx;
        self
    }
}

// ── TextMetrics: simulates AbstractTextualComponent text sizing ─────

/// Holds the text-related dimensions computed by a component.
/// Mirrors Java's `AbstractTextualComponent` margin/text dimension logic.
#[derive(Debug, Clone)]
pub struct TextMetrics {
    /// Left margin (Java: marginX1)
    pub margin_x1: f64,
    /// Right margin (Java: marginX2)
    pub margin_x2: f64,
    /// Vertical margin (Java: marginY)
    pub margin_y: f64,
    /// Pure text width (from StringBounder)
    pub pure_text_width: f64,
    /// Text height (from StringBounder)
    pub text_height: f64,
}

impl TextMetrics {
    /// Compute text metrics. Java equivalent of `AbstractTextualComponent` constructor
    /// params: marginX1, marginX2, marginY, plus text measurement.
    pub fn new(
        margin_x1: f64,
        margin_x2: f64,
        margin_y: f64,
        text_width: f64,
        text_height: f64,
    ) -> Self {
        Self {
            margin_x1,
            margin_x2,
            margin_y,
            pure_text_width: text_width,
            text_height,
        }
    }

    /// Total text width including margins. Java: `getTextWidth()`
    pub fn text_width(&self) -> f64 {
        self.pure_text_width + self.margin_x1 + self.margin_x2
    }

    /// Total text height including margins. Java: `getTextHeight()`
    pub fn text_height(&self) -> f64 {
        self.text_height + 2.0 * self.margin_y
    }
}

// ══════════════════════════════════════════════════════════════════════
// Component size calculations
// ══════════════════════════════════════════════════════════════════════

// ── Arrow constants (AbstractComponentRoseArrow) ────────────────────

/// Arrow head delta X. Java: `AbstractComponentRoseArrow.arrowDeltaX = 10`
pub const ARROW_DELTA_X: f64 = 10.0;
/// Arrow head delta Y. Java: `AbstractComponentRoseArrow.arrowDeltaY = 4`
pub const ARROW_DELTA_Y: f64 = 4.0;
/// Arrow component padding Y. Java: `AbstractComponentRoseArrow.getPaddingY() = 4`
pub const ARROW_PADDING_Y: f64 = 4.0;
/// Arrow component padding X. Java: inherited `Rose.paddingX = 5`
pub const ARROW_PADDING_X: f64 = 5.0;

/// Cross X spacing. Java: `ComponentRoseArrow.spaceCrossX = 6`
pub const SPACE_CROSS_X: f64 = 6.0;
/// Circle decoration diameter. Java: `ComponentRoseArrow.diamCircle = 8`
pub const DIAM_CIRCLE: f64 = 8.0;
/// Circle decoration stroke. Java: `ComponentRoseArrow.thinCircle = 1.5`
pub const THIN_CIRCLE: f64 = 1.5;

// ── Self-arrow constants ────────────────────────────────────────────

/// Self-arrow width. Java: `ComponentRoseSelfArrow.arrowWidth = 45`
pub const SELF_ARROW_WIDTH: f64 = 45.0;
/// Self-arrow x-right. Java: `ComponentRoseSelfArrow.xRight = arrowWidth - 3 = 42`
pub const SELF_ARROW_XRIGHT: f64 = 42.0;
/// Self-arrow internal height. Java: `getArrowOnlyHeight() = 13`
pub const SELF_ARROW_ONLY_HEIGHT: f64 = 13.0;

// ── Destroy constants ───────────────────────────────────────────────

/// Destroy cross half-size. Java: `ComponentRoseDestroy.crossSize = 9`
pub const DESTROY_CROSS_SIZE: f64 = 9.0;

// ── Grouping constants ──────────────────────────────────────────────

/// Corner size for grouping header/reference. Java: `cornersize = 10`
pub const CORNER_SIZE: f64 = 10.0;
/// Grouping space default. Java: `ComponentRoseGroupingSpace(7)`
pub const GROUPING_SPACE_HEIGHT: f64 = 7.0;

// ── Reference constants ─────────────────────────────────────────────

/// Reference frame footer height. Java: `heightFooter = 5`
pub const REF_HEIGHT_FOOTER: f64 = 5.0;
/// Reference frame x margin. Java: `xMargin = 2`
pub const REF_X_MARGIN: f64 = 2.0;

// ── Active line width ───────────────────────────────────────────────

/// Active line box width. Java: `ComponentRoseActiveLine.getPreferredWidth() = 10`
pub const ACTIVE_LINE_WIDTH: f64 = 10.0;

// ── Participant ─────────────────────────────────────────────────────

/// Delta for collections offset. Java: `getDeltaCollection() = 4`
pub const COLLECTIONS_DELTA: f64 = 4.0;

// ══════════════════════════════════════════════════════════════════════
// Size calculation functions
// ══════════════════════════════════════════════════════════════════════

/// Preferred size for a normal (non-self) arrow.
/// Java: `ComponentRoseArrow.getPreferredWidth/Height`
pub fn arrow_preferred_size(text: &TextMetrics, inclination1: f64, inclination2: f64) -> XDimension2D {
    let w = text.text_width() + ARROW_DELTA_X;
    let h = text.text_height() + ARROW_DELTA_Y + 2.0 * ARROW_PADDING_Y + inclination1 + inclination2;
    XDimension2D::new(w, h)
}

/// Preferred size for a self-arrow.
/// Java: `ComponentRoseSelfArrow.getPreferredWidth/Height`
pub fn self_arrow_preferred_size(text: &TextMetrics) -> XDimension2D {
    let w = f64::max(text.text_width(), SELF_ARROW_WIDTH + 5.0);
    let h = text.text_height() + ARROW_DELTA_Y + SELF_ARROW_ONLY_HEIGHT + 2.0 * ARROW_PADDING_Y;
    XDimension2D::new(w, h)
}

/// Y point for normal arrow. Java: `ComponentRoseArrow.getYPoint`
pub fn arrow_y_point(text: &TextMetrics, below_for_response: bool) -> f64 {
    if below_for_response {
        ARROW_PADDING_Y
    } else {
        text.text_height() + ARROW_PADDING_Y
    }
}

/// Y point for self-arrow. Java: `ComponentRoseSelfArrow.getYPoint`
pub fn self_arrow_y_point(text: &TextMetrics) -> f64 {
    let text_h = text.text_height();
    let text_and_arrow_h = text_h + SELF_ARROW_ONLY_HEIGHT;
    (text_h + text_and_arrow_h) / 2.0 + ARROW_PADDING_X
}

/// Start/end points for a normal arrow.
/// Java: `ComponentRoseArrow.getStartPoint/getEndPoint`
pub fn arrow_start_point(
    text: &TextMetrics,
    dim: XDimension2D,
    direction: ArrowDirection,
    below_for_response: bool,
    inclination2: f64,
) -> XPoint2D {
    let y = arrow_y_point(text, below_for_response);
    if direction == ArrowDirection::LeftToRight {
        XPoint2D::new(ARROW_PADDING_X, y + inclination2)
    } else {
        XPoint2D::new(dim.width + ARROW_PADDING_X, y + inclination2)
    }
}

pub fn arrow_end_point(
    text: &TextMetrics,
    dim: XDimension2D,
    direction: ArrowDirection,
    below_for_response: bool,
) -> XPoint2D {
    let y = arrow_y_point(text, below_for_response);
    if direction == ArrowDirection::LeftToRight {
        XPoint2D::new(dim.width + ARROW_PADDING_X, y)
    } else {
        XPoint2D::new(ARROW_PADDING_X, y)
    }
}

/// Start/end points for a self-arrow.
/// Java: `ComponentRoseSelfArrow.getStartPoint/getEndPoint`
pub fn self_arrow_start_point(text: &TextMetrics) -> XPoint2D {
    let text_h = text.text_height();
    XPoint2D::new(ARROW_PADDING_X, text_h + ARROW_PADDING_Y)
}

pub fn self_arrow_end_point(text: &TextMetrics) -> XPoint2D {
    let text_h = text.text_height();
    let text_and_arrow_h = text_h + SELF_ARROW_ONLY_HEIGHT;
    XPoint2D::new(ARROW_PADDING_X, text_and_arrow_h + ARROW_PADDING_Y)
}

/// Preferred size for a participant box.
/// Java: `ComponentRoseParticipant.getPreferredWidth/Height`
pub fn participant_preferred_size(
    text: &TextMetrics,
    delta_shadow: f64,
    collections: bool,
    padding: f64,
    min_width: f64,
) -> XDimension2D {
    let delta_coll = if collections { COLLECTIONS_DELTA } else { 0.0 };
    let pure = f64::max(text.pure_text_width, min_width);
    let tw = pure + text.margin_x1 + text.margin_x2;
    let w = tw + delta_shadow + delta_coll + 2.0 * padding;
    let h = text.text_height() + delta_shadow + 1.0 + delta_coll;
    XDimension2D::new(w, h)
}

/// Preferred size for a note.
/// Java: `ComponentRoseNote.getPreferredWidth/Height`
pub fn note_preferred_size(
    text: &TextMetrics,
    padding_x: f64,
    padding_y: f64,
    delta_shadow: f64,
) -> XDimension2D {
    let w = text.text_width() + 2.0 * padding_x + delta_shadow;
    let h = text.text_height() + 2.0 * padding_y + delta_shadow;
    XDimension2D::new(w, h)
}

/// Preferred size for a note box.
/// Java: `ComponentRoseNoteBox.getPreferredWidth/Height`
pub fn note_box_preferred_size(text: &TextMetrics) -> XDimension2D {
    let px = 5.0;
    let py = 5.0;
    let w = text.text_width() + 2.0 * px;
    let h = text.text_height() + 2.0 * py;
    XDimension2D::new(w, h)
}

/// Preferred size for a hexagonal note.
/// Java: `ComponentRoseNoteHexagonal.getPreferredWidth/Height`
pub fn note_hexagonal_preferred_size(text: &TextMetrics) -> XDimension2D {
    let px = 5.0;
    let py = 5.0;
    let w = text.text_width() + 2.0 * px;
    let h = text.text_height() + 2.0 * py;
    XDimension2D::new(w, h)
}

/// Preferred size for a divider.
/// Java: `ComponentRoseDivider.getPreferredWidth/Height`
pub fn divider_preferred_size(text: &TextMetrics) -> XDimension2D {
    let w = text.text_width() + 30.0;
    let h = text.text_height() + 20.0;
    XDimension2D::new(w, h)
}

/// Preferred size for a grouping header.
/// Java: `ComponentRoseGroupingHeader.getPreferredWidth/Height`
pub fn grouping_header_preferred_size(
    text: &TextMetrics,
    comment_width: f64,
    comment_height: f64,
    padding_y: f64,
) -> XDimension2D {
    let supp_h = if comment_height > 15.0 {
        comment_height - 15.0
    } else {
        0.0
    };
    let sup = if comment_width > 0.0 {
        text.margin_x1 + comment_width
    } else {
        0.0
    };
    let w = text.text_width() + sup;
    let h = text.text_height() + 2.0 * padding_y + supp_h;
    XDimension2D::new(w, h)
}

/// Preferred size for a grouping else.
/// Java: `ComponentRoseGroupingElse.getPreferredWidth/Height`
pub fn grouping_else_preferred_size(text: &TextMetrics, teoz: bool) -> XDimension2D {
    let w = text.text_width();
    let h = if teoz {
        text.text_height() + 16.0
    } else {
        text.text_height()
    };
    XDimension2D::new(w, h)
}

/// Preferred size for grouping space.
/// Java: `ComponentRoseGroupingSpace.getPreferredWidth/Height`
pub fn grouping_space_preferred_size() -> XDimension2D {
    XDimension2D::new(0.0, GROUPING_SPACE_HEIGHT)
}

/// Preferred size for a reference.
/// Java: `ComponentRoseReference.getPreferredWidth/Height`
pub fn reference_preferred_size(
    text: &TextMetrics,
    header_width: f64,
    header_height: f64,
    delta_shadow: f64,
) -> XDimension2D {
    let w = f64::max(text.text_width(), header_width) + REF_X_MARGIN * 2.0 + delta_shadow;
    let h = text.text_height() + header_height + REF_HEIGHT_FOOTER;
    XDimension2D::new(w, h)
}

/// Header width for reference. Java: `getHeaderWidth = headerDim.width + 30 + 15`
pub fn reference_header_width(header_text_width: f64) -> f64 {
    header_text_width + 30.0 + 15.0
}

/// Header height for reference. Java: `getHeaderHeight = headerDim.height + 2`
pub fn reference_header_height(header_text_height: f64) -> f64 {
    header_text_height + 2.0
}

/// Preferred size for a lifeline.
/// Java: `ComponentRoseLine.getPreferredWidth/Height`
pub fn line_preferred_size() -> XDimension2D {
    XDimension2D::new(1.0, 20.0)
}

/// Preferred size for an activation box.
/// Java: `ComponentRoseActiveLine.getPreferredWidth/Height`
pub fn active_line_preferred_size() -> XDimension2D {
    XDimension2D::new(ACTIVE_LINE_WIDTH, 0.0)
}

/// Preferred size for a destroy cross.
/// Java: `ComponentRoseDestroy.getPreferredWidth/Height`
pub fn destroy_preferred_size() -> XDimension2D {
    let s = DESTROY_CROSS_SIZE * 2.0;
    XDimension2D::new(s, s)
}

/// Preferred size for a delay line.
/// Java: `ComponentRoseDelayLine.getPreferredWidth/Height`
pub fn delay_line_preferred_size() -> XDimension2D {
    XDimension2D::new(1.0, 20.0)
}

/// Preferred size for delay text.
/// Java: `ComponentRoseDelayText.getPreferredWidth/Height`
pub fn delay_text_preferred_size(text: &TextMetrics) -> XDimension2D {
    let w = text.pure_text_width;
    let h = text.text_height() + 20.0;
    XDimension2D::new(w, h)
}

/// Preferred size for a newpage line.
/// Java: `ComponentRoseNewpage.getPreferredWidth/Height`
pub fn newpage_preferred_size() -> XDimension2D {
    XDimension2D::new(0.0, 1.0)
}

/// Preferred size for an englober (box around participants).
/// Java: `ComponentRoseEnglober.getPreferredWidth/Height`
pub fn englober_preferred_size(text: &TextMetrics) -> XDimension2D {
    let w = text.text_width();
    let h = text.text_height() + 3.0;
    XDimension2D::new(w, h)
}

// ══════════════════════════════════════════════════════════════════════
// Drawing functions - produce Vec<DrawOp>
// ══════════════════════════════════════════════════════════════════════

/// Build the polygon for a normal arrow head (pointing right).
/// Java: `ComponentRoseArrow.getPolygonNormal`
pub fn polygon_normal(part: ArrowPart, nice_arrow: bool) -> Vec<(f64, f64)> {
    match part {
        ArrowPart::TopPart => {
            vec![
                (-ARROW_DELTA_X, -ARROW_DELTA_Y),
                (0.0, 0.0),
                (-ARROW_DELTA_X, 0.0),
            ]
        }
        ArrowPart::BottomPart => {
            vec![
                (-ARROW_DELTA_X, 0.0),
                (0.0, 0.0),
                (-ARROW_DELTA_X, ARROW_DELTA_Y),
            ]
        }
        ArrowPart::Full => {
            let mut pts = vec![
                (-ARROW_DELTA_X, -ARROW_DELTA_Y),
                (0.0, 0.0),
                (-ARROW_DELTA_X, ARROW_DELTA_Y),
            ];
            if nice_arrow {
                pts.push((-ARROW_DELTA_X + 4.0, 0.0));
            }
            pts
        }
    }
}

/// Build the polygon for a reverse arrow head (pointing left).
/// Java: `ComponentRoseArrow.getPolygonReverse`
pub fn polygon_reverse(part: ArrowPart, nice_arrow: bool) -> Vec<(f64, f64)> {
    match part {
        ArrowPart::TopPart => {
            vec![
                (ARROW_DELTA_X, -ARROW_DELTA_Y),
                (0.0, 0.0),
                (ARROW_DELTA_X, 0.0),
            ]
        }
        ArrowPart::BottomPart => {
            vec![
                (ARROW_DELTA_X, 0.0),
                (0.0, 0.0),
                (ARROW_DELTA_X, ARROW_DELTA_Y),
            ]
        }
        ArrowPart::Full => {
            let mut pts = vec![
                (ARROW_DELTA_X, -ARROW_DELTA_Y),
                (0.0, 0.0),
                (ARROW_DELTA_X, ARROW_DELTA_Y),
            ];
            if nice_arrow {
                pts.push((ARROW_DELTA_X - 4.0, 0.0));
            }
            pts
        }
    }
}

/// Build the polygon for a self-arrow head.
/// Java: `ComponentRoseSelfArrow.getPolygon`
pub fn polygon_self(config: &ArrowConfiguration, nice_arrow: bool) -> Vec<(f64, f64)> {
    let direction: f64 = if config.is_reverse_define() { -1.0 } else { 1.0 };
    let x = direction * ARROW_DELTA_X;
    match config.part() {
        ArrowPart::TopPart => {
            vec![(x - 1.0, -ARROW_DELTA_Y), (-1.0, 0.0), (x - 1.0, 0.0)]
        }
        ArrowPart::BottomPart => {
            vec![(x - 1.0, 0.0), (-1.0, 0.0), (x - 1.0, ARROW_DELTA_Y)]
        }
        ArrowPart::Full => {
            let mut pts = vec![(x, -ARROW_DELTA_Y), (0.0, 0.0), (x, ARROW_DELTA_Y)];
            if nice_arrow {
                pts.push((x - direction * 4.0, 0.0));
            }
            pts
        }
    }
}

/// Generate DrawOps for a normal (non-self) arrow.
/// Java: `ComponentRoseArrow.drawInternalU`
pub fn draw_arrow(
    config: &ArrowConfiguration,
    text: &TextMetrics,
    area: &Area,
    fg_color: &HColor,
    bg_color: &HColor,
    stroke: &UStroke,
    nice_arrow: bool,
    below_for_response: bool,
    inclination1: f64,
    inclination2: f64,
) -> Vec<DrawOp> {
    if config.is_hidden() {
        return vec![];
    }
    let mut ops = Vec::new();
    let dim = area.dimension;

    let dressing1 = config.dressing1();
    let dressing2 = config.dressing2();

    let mut start = 0.0;
    let mut len = dim.width - 1.0;
    let _len_full = dim.width;

    let pos1 = start + 1.0;
    let pos2 = len - 1.0;

    // Decoration adjustments
    if config.decoration2() == ArrowDecoration::Circle {
        if dressing2.head == ArrowHead::None {
            len -= DIAM_CIRCLE / 2.0;
        }
        if dressing2.head != ArrowHead::None {
            len -= DIAM_CIRCLE / 2.0 + THIN_CIRCLE;
        }
    }
    if config.decoration1() == ArrowDecoration::Circle {
        if dressing1.head == ArrowHead::None {
            start += DIAM_CIRCLE / 2.0;
            len -= DIAM_CIRCLE / 2.0;
        }
        if dressing1.head == ArrowHead::Async {
            start += DIAM_CIRCLE / 2.0 + THIN_CIRCLE;
            len -= DIAM_CIRCLE / 2.0 + THIN_CIRCLE;
        }
        if dressing1.head == ArrowHead::Normal {
            start += DIAM_CIRCLE / 2.0 + THIN_CIRCLE;
            len -= DIAM_CIRCLE / 2.0 + THIN_CIRCLE;
        }
    }

    if dressing2.head == ArrowHead::Normal {
        len -= ARROW_DELTA_X / 2.0;
    }
    if dressing1.head == ArrowHead::Normal {
        start += ARROW_DELTA_X / 2.0;
        len -= ARROW_DELTA_X / 2.0;
    }
    if dressing2.head == ArrowHead::CrossX {
        len -= 2.0 * SPACE_CROSS_X;
    }
    if dressing1.head == ArrowHead::CrossX {
        start += 2.0 * SPACE_CROSS_X;
        len -= 2.0 * SPACE_CROSS_X;
    }

    let is_below = below_for_response && config.is_reverse_define();
    let pos_arrow = if is_below {
        0.0
    } else {
        text.text_height()
    };

    // Main line
    let line_stroke = if config.is_dotted() {
        UStroke::new(5.0, 5.0, stroke.thickness)
    } else {
        stroke.clone()
    };

    if inclination1 == 0.0 && inclination2 == 0.0 {
        ops.push(DrawOp::Line {
            translate: UTranslate::new(start, pos_arrow),
            dx: len,
            dy: 0.0,
            stroke: line_stroke,
            color: Some(fg_color.clone()),
        });
    }

    // Dressing2 (right end) - normal arrow head
    if dressing2.head == ArrowHead::Normal {
        let poly = polygon_normal(ArrowPart::Full, nice_arrow);
        ops.push(DrawOp::Polygon {
            translate: UTranslate::new(pos2, pos_arrow + inclination2),
            points: poly,
            stroke: stroke.clone(),
            color: Some(fg_color.clone()),
            fill: Some(fg_color.clone()),
        });
    } else if dressing2.head == ArrowHead::Async {
        if config.part() != ArrowPart::BottomPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(pos2, pos_arrow + inclination2),
                dx: -ARROW_DELTA_X,
                dy: -ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
        if config.part() != ArrowPart::TopPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(pos2, pos_arrow + inclination2),
                dx: -ARROW_DELTA_X,
                dy: ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
    } else if dressing2.head == ArrowHead::CrossX {
        let x_stroke = UStroke::with_thickness(2.0);
        ops.push(DrawOp::Line {
            translate: UTranslate::new(
                pos2 - SPACE_CROSS_X - ARROW_DELTA_X,
                pos_arrow + inclination2 - ARROW_DELTA_X / 2.0,
            ),
            dx: ARROW_DELTA_X,
            dy: ARROW_DELTA_X,
            stroke: x_stroke.clone(),
            color: Some(fg_color.clone()),
        });
        ops.push(DrawOp::Line {
            translate: UTranslate::new(
                pos2 - SPACE_CROSS_X - ARROW_DELTA_X,
                pos_arrow + inclination2 + ARROW_DELTA_X / 2.0,
            ),
            dx: ARROW_DELTA_X,
            dy: -ARROW_DELTA_X,
            stroke: x_stroke,
            color: Some(fg_color.clone()),
        });
    }

    // Dressing1 (left end) - reverse arrow head
    if dressing1.head == ArrowHead::Normal {
        let poly = polygon_reverse(ArrowPart::Full, nice_arrow);
        ops.push(DrawOp::Polygon {
            translate: UTranslate::new(pos1, pos_arrow + inclination1),
            points: poly,
            stroke: stroke.clone(),
            color: Some(fg_color.clone()),
            fill: Some(fg_color.clone()),
        });
    } else if dressing1.head == ArrowHead::Async {
        if config.part() != ArrowPart::BottomPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(pos1, pos_arrow + inclination1),
                dx: ARROW_DELTA_X,
                dy: -ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
        if config.part() != ArrowPart::TopPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(pos1, pos_arrow + inclination1),
                dx: ARROW_DELTA_X,
                dy: ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
    } else if dressing1.head == ArrowHead::CrossX {
        let x_stroke = UStroke::with_thickness(2.0);
        ops.push(DrawOp::Line {
            translate: UTranslate::new(
                pos1 + SPACE_CROSS_X,
                pos_arrow + inclination1 - ARROW_DELTA_X / 2.0,
            ),
            dx: ARROW_DELTA_X,
            dy: ARROW_DELTA_X,
            stroke: x_stroke.clone(),
            color: Some(fg_color.clone()),
        });
        ops.push(DrawOp::Line {
            translate: UTranslate::new(
                pos1 + SPACE_CROSS_X,
                pos_arrow + inclination1 + ARROW_DELTA_X / 2.0,
            ),
            dx: ARROW_DELTA_X,
            dy: -ARROW_DELTA_X,
            stroke: x_stroke,
            color: Some(fg_color.clone()),
        });
    }

    // Decorations (circles)
    if config.decoration1() == ArrowDecoration::Circle {
        ops.push(DrawOp::Ellipse {
            translate: UTranslate::new(
                pos1 - DIAM_CIRCLE / 2.0 - THIN_CIRCLE,
                pos_arrow + inclination1 - DIAM_CIRCLE / 2.0 - THIN_CIRCLE / 2.0,
            ),
            width: DIAM_CIRCLE,
            height: DIAM_CIRCLE,
            stroke: UStroke::with_thickness(THIN_CIRCLE),
            color: Some(fg_color.clone()),
            fill: Some(bg_color.clone()),
        });
    }
    if config.decoration2() == ArrowDecoration::Circle {
        ops.push(DrawOp::Ellipse {
            translate: UTranslate::new(
                pos2 - DIAM_CIRCLE / 2.0 + THIN_CIRCLE,
                pos_arrow + inclination2 - DIAM_CIRCLE / 2.0 - THIN_CIRCLE / 2.0,
            ),
            width: DIAM_CIRCLE,
            height: DIAM_CIRCLE,
            stroke: UStroke::with_thickness(THIN_CIRCLE),
            color: Some(fg_color.clone()),
            fill: Some(bg_color.clone()),
        });
    }

    ops
}

/// Generate DrawOps for a self-arrow (right side).
/// Java: `ComponentRoseSelfArrow.drawRightSide`
pub fn draw_self_arrow(
    config: &ArrowConfiguration,
    text: &TextMetrics,
    _area: &Area,
    fg_color: &HColor,
    _bg_color: &HColor,
    stroke: &UStroke,
    nice_arrow: bool,
) -> Vec<DrawOp> {
    if config.is_hidden() {
        return vec![];
    }
    let mut ops = Vec::new();
    let text_height = text.text_height();
    let arrow_height = SELF_ARROW_ONLY_HEIGHT;

    let line_stroke = if config.is_dotted() {
        UStroke::new(5.0, 5.0, stroke.thickness)
    } else {
        stroke.clone()
    };

    let mut x1: f64 = 0.0;
    let mut x2: f64 = 1.0;

    if config.decoration1() == ArrowDecoration::Circle {
        x1 += DIAM_CIRCLE / 2.0 + THIN_CIRCLE + 1.0;
    }
    if config.decoration2() == ArrowDecoration::Circle {
        x2 += DIAM_CIRCLE / 2.0 + THIN_CIRCLE;
    }

    let has_starting_cross = config.dressing1().head == ArrowHead::CrossX;
    if has_starting_cross {
        x1 += 2.0 * SPACE_CROSS_X;
    }

    let has_final_cross = config.dressing2().head == ArrowHead::CrossX;
    if has_final_cross {
        x2 += 2.0 * SPACE_CROSS_X;
    }

    // Three lines forming the self-arrow bracket
    // Top horizontal
    ops.push(DrawOp::Line {
        translate: UTranslate::new(x1, text_height),
        dx: SELF_ARROW_XRIGHT - x1,
        dy: 0.0,
        stroke: line_stroke.clone(),
        color: Some(fg_color.clone()),
    });
    // Vertical
    ops.push(DrawOp::Line {
        translate: UTranslate::new(SELF_ARROW_XRIGHT, text_height),
        dx: 0.0,
        dy: arrow_height,
        stroke: line_stroke.clone(),
        color: Some(fg_color.clone()),
    });
    // Bottom horizontal
    ops.push(DrawOp::Line {
        translate: UTranslate::new(x2, text_height + arrow_height),
        dx: SELF_ARROW_XRIGHT - x2,
        dy: 0.0,
        stroke: line_stroke,
        color: Some(fg_color.clone()),
    });

    // Arrow head at bottom-left
    if has_final_cross {
        let x_stroke = UStroke::with_thickness(2.0);
        ops.push(DrawOp::Line {
            translate: UTranslate::new(
                SPACE_CROSS_X,
                text_height - ARROW_DELTA_X / 2.0 + arrow_height,
            ),
            dx: ARROW_DELTA_X,
            dy: ARROW_DELTA_X,
            stroke: x_stroke.clone(),
            color: Some(fg_color.clone()),
        });
        ops.push(DrawOp::Line {
            translate: UTranslate::new(
                SPACE_CROSS_X,
                text_height + ARROW_DELTA_X / 2.0 + arrow_height,
            ),
            dx: ARROW_DELTA_X,
            dy: -ARROW_DELTA_X,
            stroke: x_stroke,
            color: Some(fg_color.clone()),
        });
    } else if config.dressing2().head == ArrowHead::Async {
        if config.part() != ArrowPart::BottomPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(x2, text_height + arrow_height),
                dx: ARROW_DELTA_X,
                dy: -ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
        if config.part() != ArrowPart::TopPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(x2, text_height + arrow_height),
                dx: ARROW_DELTA_X,
                dy: ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
    } else if config.dressing2().head == ArrowHead::Normal {
        let poly = polygon_self(config, nice_arrow);
        ops.push(DrawOp::Polygon {
            translate: UTranslate::new(x2, text_height + arrow_height),
            points: poly,
            stroke: stroke.clone(),
            color: Some(fg_color.clone()),
            fill: Some(fg_color.clone()),
        });
    }

    // Starting dressing (top-left)
    if has_starting_cross {
        let x_stroke = UStroke::with_thickness(2.0);
        ops.push(DrawOp::Line {
            translate: UTranslate::new(SPACE_CROSS_X, text_height - ARROW_DELTA_X / 2.0),
            dx: ARROW_DELTA_X,
            dy: ARROW_DELTA_X,
            stroke: x_stroke.clone(),
            color: Some(fg_color.clone()),
        });
        ops.push(DrawOp::Line {
            translate: UTranslate::new(SPACE_CROSS_X, text_height + ARROW_DELTA_X / 2.0),
            dx: ARROW_DELTA_X,
            dy: -ARROW_DELTA_X,
            stroke: x_stroke,
            color: Some(fg_color.clone()),
        });
    } else if config.dressing1().head == ArrowHead::Async {
        if config.part() != ArrowPart::BottomPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(x1, text_height),
                dx: ARROW_DELTA_X,
                dy: ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
        if config.part() != ArrowPart::TopPart {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(x1, text_height),
                dx: ARROW_DELTA_X,
                dy: -ARROW_DELTA_Y,
                stroke: stroke.clone(),
                color: Some(fg_color.clone()),
            });
        }
    } else if config.dressing1().head == ArrowHead::Normal {
        let poly = polygon_self(config, nice_arrow);
        ops.push(DrawOp::Polygon {
            translate: UTranslate::new(x1, text_height),
            points: poly,
            stroke: stroke.clone(),
            color: Some(fg_color.clone()),
            fill: Some(fg_color.clone()),
        });
    }

    ops
}

/// Generate DrawOps for a participant box.
/// Java: `ComponentRoseParticipant.drawInternalU`
pub fn draw_participant(
    text: &TextMetrics,
    _area: &Area,
    fg_color: &HColor,
    bg_color: &HColor,
    stroke: &UStroke,
    round_corner: f64,
    _diagonal_corner: f64,
    delta_shadow: f64,
    collections: bool,
    padding: f64,
    min_width: f64,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();

    let pure = f64::max(text.pure_text_width, min_width);
    let tw = pure + text.margin_x1 + text.margin_x2;
    let th = text.text_height();
    let delta_coll = if collections { COLLECTIONS_DELTA } else { 0.0 };

    let supp_width = pure - text.pure_text_width;

    if collections {
        ops.push(DrawOp::Rect {
            translate: UTranslate::new(padding + delta_coll, 0.0),
            width: tw,
            height: th,
            rx: round_corner,
            stroke: stroke.clone(),
            color: Some(fg_color.clone()),
            fill: Some(bg_color.clone()),
            shadow: delta_shadow,
        });
    }

    ops.push(DrawOp::Rect {
        translate: UTranslate::new(padding, delta_coll),
        width: tw,
        height: th,
        rx: round_corner,
        stroke: stroke.clone(),
        color: Some(fg_color.clone()),
        fill: Some(bg_color.clone()),
        shadow: delta_shadow,
    });

    // text position
    ops.push(DrawOp::Text {
        translate: UTranslate::new(
            padding + text.margin_x1 + supp_width / 2.0,
            delta_coll + text.margin_y,
        ),
        text: String::new(), // placeholder: actual text drawn by caller
        font_family: String::new(),
        font_size: 0.0,
        bold: false,
        italic: false,
        color: None,
    });

    ops
}

/// Generate DrawOps for a note.
/// Java: `ComponentRoseNote.drawInternalU`
pub fn draw_note(
    text: &TextMetrics,
    area: &Area,
    fashion: &Fashion,
    padding_x: f64,
    padding_y: f64,
    round_corner: f64,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();

    let text_height = text.text_height() as i32;
    let mut x2 = text.text_width() as i32;

    let _diff_x = area.dimension.width - note_preferred_size(text, padding_x, padding_y, fashion.delta_shadow).width;

    if area.dimension.width > note_preferred_size(text, padding_x, padding_y, fashion.delta_shadow).width {
        x2 = (area.dimension.width - 2.0 * padding_x) as i32;
    }

    // Note polygon (rectangle with folded corner)
    let mut path = UPath::new();
    if round_corner == 0.0 {
        path.move_to(0.0, 0.0);
        path.line_to(x2 as f64 - CORNER_SIZE, 0.0);
        path.line_to(x2 as f64, CORNER_SIZE);
        path.line_to(x2 as f64, text_height as f64);
        path.line_to(0.0, text_height as f64);
        path.close();
    } else {
        let r = round_corner;
        path.move_to(r, 0.0);
        path.line_to(x2 as f64 - CORNER_SIZE, 0.0);
        path.line_to(x2 as f64, CORNER_SIZE);
        path.line_to(x2 as f64, text_height as f64 - r);
        path.arc_to(r, r, 0.0, 0.0, 1.0, x2 as f64 - r, text_height as f64);
        path.line_to(r, text_height as f64);
        path.arc_to(r, r, 0.0, 0.0, 1.0, 0.0, text_height as f64 - r);
        path.line_to(0.0, r);
        path.arc_to(r, r, 0.0, 0.0, 1.0, r, 0.0);
    }

    ops.push(DrawOp::Path {
        translate: UTranslate::none(),
        path,
        stroke: fashion.stroke.clone(),
        color: fashion.fore_color.clone(),
        fill: fashion.back_color.clone(),
        shadow: fashion.delta_shadow,
    });

    // Corner fold
    let mut corner = UPath::new();
    corner.move_to(x2 as f64 - CORNER_SIZE, 0.0);
    corner.line_to(x2 as f64 - CORNER_SIZE, CORNER_SIZE);
    corner.line_to(x2 as f64, CORNER_SIZE);
    ops.push(DrawOp::Path {
        translate: UTranslate::none(),
        path: corner,
        stroke: fashion.stroke.clone(),
        color: fashion.fore_color.clone(),
        fill: None,
        shadow: 0.0,
    });

    ops
}

/// Generate DrawOps for a divider.
/// Java: `ComponentRoseDivider.drawInternalU`
pub fn draw_divider(
    text: &TextMetrics,
    area: &Area,
    border_color: &HColor,
    bg_color: &HColor,
    stroke: &UStroke,
    round_corner: f64,
    shadow: f64,
    empty: bool,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let dim = area.dimension;

    if empty {
        // Just draw separator lines
        draw_divider_sep(&mut ops, dim.width, dim.height / 2.0, bg_color, border_color, stroke, round_corner, shadow);
    } else {
        let text_width = text.text_width();
        let text_height = text.text_height();
        let delta_x = 6.0;
        let xpos = (dim.width - text_width - delta_x) / 2.0;
        let ypos = (dim.height - text_height) / 2.0;

        draw_divider_sep(&mut ops, dim.width, dim.height / 2.0, bg_color, border_color, stroke, round_corner, shadow);

        // Text background rect
        ops.push(DrawOp::Rect {
            translate: UTranslate::new(xpos, ypos),
            width: text_width + delta_x,
            height: text_height,
            rx: round_corner,
            stroke: stroke.clone(),
            color: Some(border_color.clone()),
            fill: Some(bg_color.clone()),
            shadow,
        });
    }

    ops
}

fn draw_divider_sep(
    ops: &mut Vec<DrawOp>,
    width: f64,
    y: f64,
    bg_color: &HColor,
    border_color: &HColor,
    stroke: &UStroke,
    round_corner: f64,
    shadow: f64,
) {
    // Background rect (3px tall)
    ops.push(DrawOp::Rect {
        translate: UTranslate::new(0.0, y - 1.0),
        width,
        height: 3.0,
        rx: round_corner,
        stroke: UStroke::simple(),
        color: Some(bg_color.clone()),
        fill: Some(bg_color.clone()),
        shadow,
    });

    // Double lines
    let half_thick = stroke.thickness / 2.0;
    let line_stroke = UStroke::with_thickness(half_thick);
    ops.push(DrawOp::Line {
        translate: UTranslate::new(0.0, y - 1.0),
        dx: width,
        dy: 0.0,
        stroke: line_stroke.clone(),
        color: Some(border_color.clone()),
    });
    ops.push(DrawOp::Line {
        translate: UTranslate::new(0.0, y + 2.0),
        dx: width,
        dy: 0.0,
        stroke: line_stroke,
        color: Some(border_color.clone()),
    });
}

/// Generate DrawOps for a grouping header.
/// Java: `ComponentRoseGroupingHeader.drawInternalU` + `drawBackgroundInternalU`
pub fn draw_grouping_header(
    text: &TextMetrics,
    area: &Area,
    fashion: &Fashion,
    corner_fashion: &Fashion,
    background: &HColor,
    round_corner: f64,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let dim = area.dimension;
    let text_width = text.text_width();
    let text_height = text.text_height();

    // Background rect
    ops.push(DrawOp::Rect {
        translate: UTranslate::none(),
        width: dim.width,
        height: dim.height,
        rx: round_corner,
        stroke: fashion.stroke.clone(),
        color: fashion.fore_color.clone(),
        fill: Some(background.clone()),
        shadow: fashion.delta_shadow,
    });

    // Corner tab
    let corner_path = grouping_corner_path(text_width, text_height, round_corner);
    ops.push(DrawOp::Path {
        translate: UTranslate::none(),
        path: corner_path,
        stroke: corner_fashion.stroke.clone(),
        color: corner_fashion.fore_color.clone(),
        fill: corner_fashion.back_color.clone(),
        shadow: 0.0,
    });

    // Outline rect (no fill)
    ops.push(DrawOp::Rect {
        translate: UTranslate::none(),
        width: dim.width,
        height: dim.height,
        rx: round_corner,
        stroke: fashion.stroke.clone(),
        color: fashion.fore_color.clone(),
        fill: None,
        shadow: 0.0,
    });

    ops
}

/// Build the corner tab path for a grouping header.
/// Java: `ComponentRoseGroupingHeader.getCorner`
pub fn grouping_corner_path(width: f64, height: f64, round_corner: f64) -> UPath {
    let mut path = UPath::new();
    if round_corner == 0.0 {
        path.move_to(0.0, 0.0);
        path.line_to(width, 0.0);
        path.line_to(width, height - CORNER_SIZE);
        path.line_to(width - CORNER_SIZE, height);
        path.line_to(0.0, height);
        path.line_to(0.0, 0.0);
    } else {
        let r = round_corner / 2.0;
        path.move_to(r, 0.0);
        path.line_to(width, 0.0);
        path.line_to(width, height - CORNER_SIZE);
        path.line_to(width - CORNER_SIZE, height);
        path.line_to(0.0, height);
        path.line_to(0.0, r);
        path.arc_to(r, r, 0.0, 0.0, 1.0, r, 0.0);
    }
    path
}

/// Generate DrawOps for a grouping else separator.
/// Java: `ComponentRoseGroupingElse.drawInternalU`
pub fn draw_grouping_else(
    _text: &TextMetrics,
    area: &Area,
    border_color: &HColor,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let dim = area.dimension;

    // Dashed line
    ops.push(DrawOp::Line {
        translate: UTranslate::new(0.0, 1.0),
        dx: dim.width,
        dy: 0.0,
        stroke: UStroke::new(2.0, 2.0, 1.0),
        color: Some(border_color.clone()),
    });

    ops
}

/// Generate DrawOps for a lifeline.
/// Java: `ComponentRoseLine.drawInternalU`
pub fn draw_line(area: &Area, color: &HColor, stroke: &UStroke) -> Vec<DrawOp> {
    let dim = area.dimension;
    let x = (dim.width / 2.0) as i32;

    let mut ops = Vec::new();

    // Hover target rect (transparent)
    if dim.height > 0.0 {
        let hover_w = 8.0;
        ops.push(DrawOp::Rect {
            translate: UTranslate::new((dim.width - hover_w) / 2.0, 0.0),
            width: hover_w,
            height: dim.height,
            rx: 0.0,
            stroke: UStroke::with_thickness(0.0),
            color: Some(HColor::None),
            fill: Some(HColor::None),
            shadow: 0.0,
        });
    }

    ops.push(DrawOp::Line {
        translate: UTranslate::new(x as f64, 0.0),
        dx: 0.0,
        dy: dim.height,
        stroke: stroke.clone(),
        color: Some(color.clone()),
    });

    ops
}

/// Generate DrawOps for an activation box.
/// Java: `ComponentRoseActiveLine.drawInternalU`
pub fn draw_active_line(
    area: &Area,
    fashion: &Fashion,
    close_up: bool,
    close_down: bool,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let dim = area.dimension;
    let x = ((dim.width - ACTIVE_LINE_WIDTH) / 2.0) as i32;

    if dim.height == 0.0 {
        return ops;
    }

    let shadow = if fashion.is_shadowing() { 1.0 } else { 0.0 };

    if close_up && close_down {
        ops.push(DrawOp::Rect {
            translate: UTranslate::new(x as f64, 0.0),
            width: ACTIVE_LINE_WIDTH,
            height: dim.height,
            rx: 0.0,
            stroke: UStroke::simple(),
            color: fashion.fore_color.clone(),
            fill: fashion.back_color.clone(),
            shadow,
        });
    } else {
        // Background rect (no border)
        ops.push(DrawOp::Rect {
            translate: UTranslate::new(x as f64, 0.0),
            width: ACTIVE_LINE_WIDTH,
            height: dim.height,
            rx: 0.0,
            stroke: UStroke::simple(),
            color: fashion.back_color.clone(),
            fill: fashion.back_color.clone(),
            shadow: 0.0,
        });

        // Left & right vertical lines
        ops.push(DrawOp::Line {
            translate: UTranslate::new(x as f64, 0.0),
            dx: 0.0,
            dy: dim.height,
            stroke: UStroke::simple(),
            color: fashion.fore_color.clone(),
        });
        ops.push(DrawOp::Line {
            translate: UTranslate::new(x as f64 + ACTIVE_LINE_WIDTH, 0.0),
            dx: 0.0,
            dy: dim.height,
            stroke: UStroke::simple(),
            color: fashion.fore_color.clone(),
        });

        // Top/bottom lines if closed
        if close_up {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(x as f64, 0.0),
                dx: ACTIVE_LINE_WIDTH,
                dy: 0.0,
                stroke: UStroke::simple(),
                color: fashion.fore_color.clone(),
            });
        }
        if close_down {
            ops.push(DrawOp::Line {
                translate: UTranslate::new(x as f64, dim.height),
                dx: ACTIVE_LINE_WIDTH,
                dy: 0.0,
                stroke: UStroke::simple(),
                color: fashion.fore_color.clone(),
            });
        }
    }

    ops
}

/// Generate DrawOps for a destroy cross.
/// Java: `ComponentRoseDestroy.drawInternalU`
pub fn draw_destroy(color: &HColor, stroke: &UStroke) -> Vec<DrawOp> {
    let s = DESTROY_CROSS_SIZE;
    vec![
        DrawOp::Line {
            translate: UTranslate::none(),
            dx: 2.0 * s,
            dy: 2.0 * s,
            stroke: stroke.clone(),
            color: Some(color.clone()),
        },
        DrawOp::Line {
            translate: UTranslate::new(0.0, 2.0 * s),
            dx: 2.0 * s,
            dy: -2.0 * s,
            stroke: stroke.clone(),
            color: Some(color.clone()),
        },
    ]
}

/// Generate DrawOps for a delay line.
/// Java: `ComponentRoseDelayLine.drawInternalU`
pub fn draw_delay_line(area: &Area, color: &HColor, stroke: &UStroke) -> Vec<DrawOp> {
    let dim = area.dimension;
    let x = (dim.width / 2.0) as i32;
    vec![DrawOp::Line {
        translate: UTranslate::new(x as f64, 0.0),
        dx: 0.0,
        dy: dim.height,
        stroke: stroke.clone(),
        color: Some(color.clone()),
    }]
}

/// Generate DrawOps for a newpage line.
/// Java: `ComponentRoseNewpage.drawInternalU`
pub fn draw_newpage(area: &Area, color: &HColor, stroke: &UStroke) -> Vec<DrawOp> {
    let dim = area.dimension;
    vec![DrawOp::Line {
        translate: UTranslate::none(),
        dx: dim.width,
        dy: 0.0,
        stroke: stroke.clone(),
        color: Some(color.clone()),
    }]
}

/// Generate DrawOps for a reference frame.
/// Java: `ComponentRoseReference.drawInternalU`
pub fn draw_reference(
    text: &TextMetrics,
    area: &Area,
    header_fashion: &Fashion,
    body_fashion: &Fashion,
    header_text_width: f64,
    header_text_height: f64,
    round_corner: f64,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let dim = area.dimension;

    let text_header_width = reference_header_width(header_text_width) as i32;
    let text_header_height = reference_header_height(header_text_height) as i32;

    // Body rect
    let body_width = dim.width - REF_X_MARGIN * 2.0 - body_fashion.delta_shadow;
    let body_height = dim.height - REF_HEIGHT_FOOTER;
    ops.push(DrawOp::Rect {
        translate: UTranslate::new(REF_X_MARGIN, 0.0),
        width: body_width,
        height: body_height,
        rx: round_corner,
        stroke: body_fashion.stroke.clone(),
        color: body_fashion.fore_color.clone(),
        fill: body_fashion.back_color.clone(),
        shadow: body_fashion.delta_shadow,
    });

    // Header corner tab
    let header_corner = reference_corner_path(
        text_header_width as f64,
        text_header_height as f64,
        round_corner,
    );
    ops.push(DrawOp::Path {
        translate: UTranslate::new(REF_X_MARGIN, 0.0),
        path: header_corner,
        stroke: header_fashion.stroke.clone(),
        color: header_fashion.fore_color.clone(),
        fill: header_fashion.back_color.clone(),
        shadow: 0.0,
    });

    ops
}

/// Build the corner tab path for a reference header.
/// Java: `ComponentRoseReference` corner path
pub fn reference_corner_path(width: f64, height: f64, round_corner: f64) -> UPath {
    let mut path = UPath::new();
    if round_corner == 0.0 {
        path.move_to(0.0, 0.0);
        path.line_to(width, 0.0);
        path.line_to(width, height - CORNER_SIZE);
        path.line_to(width - CORNER_SIZE, height);
        path.line_to(0.0, height);
        path.line_to(0.0, 0.0);
    } else {
        let r = round_corner / 2.0;
        path.move_to(r, 0.0);
        path.line_to(width, 0.0);
        path.line_to(width, height - CORNER_SIZE);
        path.line_to(width - CORNER_SIZE, height);
        path.line_to(0.0, height);
        path.line_to(0.0, r);
        path.arc_to(r, r, 0.0, 0.0, 1.0, r, 0.0);
    }
    path
}

/// Generate DrawOps for a note box.
/// Java: `ComponentRoseNoteBox.drawInternalU`
pub fn draw_note_box(
    text: &TextMetrics,
    area: &Area,
    fashion: &Fashion,
    round_corner: f64,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let px = 5.0;

    let text_height = text.text_height() as i32;
    let mut x2 = text.text_width() as i32;

    if area.dimension.width > note_box_preferred_size(text).width {
        x2 = (area.dimension.width - 2.0 * px) as i32;
    }

    ops.push(DrawOp::Rect {
        translate: UTranslate::none(),
        width: x2 as f64,
        height: text_height as f64,
        rx: round_corner,
        stroke: fashion.stroke.clone(),
        color: fashion.fore_color.clone(),
        fill: fashion.back_color.clone(),
        shadow: fashion.delta_shadow,
    });

    ops
}

/// Generate DrawOps for a hexagonal note.
/// Java: `ComponentRoseNoteHexagonal.drawInternalU`
pub fn draw_note_hexagonal(
    text: &TextMetrics,
    area: &Area,
    fashion: &Fashion,
) -> Vec<DrawOp> {
    let mut ops = Vec::new();
    let px = 5.0;

    let text_height = text.text_height() as i32;
    let mut x2 = text.text_width() as i32;

    if area.dimension.width > note_hexagonal_preferred_size(text).width {
        x2 = (area.dimension.width - 2.0 * px) as i32;
    }

    let cs = CORNER_SIZE;
    let th2 = text_height as f64 / 2.0;
    let points = vec![
        (cs, 0.0),
        (x2 as f64 - cs, 0.0),
        (x2 as f64, th2),
        (x2 as f64 - cs, text_height as f64),
        (cs, text_height as f64),
        (0.0, th2),
        (cs, 0.0),
    ];

    ops.push(DrawOp::Polygon {
        translate: UTranslate::none(),
        points,
        stroke: fashion.stroke.clone(),
        color: fashion.fore_color.clone(),
        fill: fashion.back_color.clone(),
    });

    ops
}

/// Generate DrawOps for an englober (box around participants).
/// Java: `ComponentRoseEnglober.drawBackgroundInternalU`
pub fn draw_englober(
    _text: &TextMetrics,
    area: &Area,
    fashion: &Fashion,
    round_corner: f64,
) -> Vec<DrawOp> {
    let dim = area.dimension;
    vec![DrawOp::Rect {
        translate: UTranslate::none(),
        width: dim.width,
        height: dim.height,
        rx: round_corner,
        stroke: fashion.stroke.clone(),
        color: fashion.fore_color.clone(),
        fill: fashion.back_color.clone(),
        shadow: fashion.delta_shadow,
    }]
}

// ══════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_text(pure_w: f64, h: f64) -> TextMetrics {
        TextMetrics::new(7.0, 7.0, 1.0, pure_w, h)
    }

    // ── TextMetrics ─────────────────────────────────────────────────

    #[test]
    fn text_metrics_width() {
        let tm = make_text(50.0, 14.0);
        assert_eq!(tm.text_width(), 50.0 + 7.0 + 7.0);
    }

    #[test]
    fn text_metrics_height() {
        let tm = make_text(50.0, 14.0);
        assert_eq!(tm.text_height(), 14.0 + 2.0);
    }

    // ── Arrow size ──────────────────────────────────────────────────

    #[test]
    fn arrow_preferred_size_basic() {
        let tm = make_text(80.0, 14.0);
        let dim = arrow_preferred_size(&tm, 0.0, 0.0);
        assert_eq!(dim.width, tm.text_width() + ARROW_DELTA_X);
        assert_eq!(
            dim.height,
            tm.text_height() + ARROW_DELTA_Y + 2.0 * ARROW_PADDING_Y
        );
    }

    #[test]
    fn arrow_preferred_size_with_inclination() {
        let tm = make_text(80.0, 14.0);
        let dim = arrow_preferred_size(&tm, 5.0, 3.0);
        let base = arrow_preferred_size(&tm, 0.0, 0.0);
        assert_eq!(dim.height, base.height + 8.0);
    }

    // ── Self-arrow size ─────────────────────────────────────────────

    #[test]
    fn self_arrow_preferred_size_basic() {
        let tm = make_text(30.0, 14.0);
        let dim = self_arrow_preferred_size(&tm);
        assert_eq!(dim.width, SELF_ARROW_WIDTH + 5.0);
        assert_eq!(
            dim.height,
            tm.text_height() + ARROW_DELTA_Y + SELF_ARROW_ONLY_HEIGHT + 2.0 * ARROW_PADDING_Y
        );
    }

    #[test]
    fn self_arrow_preferred_size_wide_text() {
        let tm = make_text(100.0, 14.0);
        let dim = self_arrow_preferred_size(&tm);
        assert_eq!(dim.width, tm.text_width());
    }

    // ── Participant size ────────────────────────────────────────────

    #[test]
    fn participant_preferred_size_basic() {
        let tm = make_text(60.0, 14.0);
        let dim = participant_preferred_size(&tm, 0.0, false, 0.0, 0.0);
        assert_eq!(dim.width, tm.text_width());
        assert_eq!(dim.height, tm.text_height() + 1.0);
    }

    #[test]
    fn participant_preferred_size_with_min_width() {
        let tm = make_text(20.0, 14.0);
        let dim = participant_preferred_size(&tm, 0.0, false, 0.0, 100.0);
        assert!(dim.width >= 100.0 + tm.margin_x1 + tm.margin_x2);
    }

    #[test]
    fn participant_preferred_size_with_collections() {
        let tm = make_text(60.0, 14.0);
        let dim_normal = participant_preferred_size(&tm, 0.0, false, 0.0, 0.0);
        let dim_coll = participant_preferred_size(&tm, 0.0, true, 0.0, 0.0);
        assert_eq!(dim_coll.width, dim_normal.width + COLLECTIONS_DELTA);
        assert_eq!(dim_coll.height, dim_normal.height + COLLECTIONS_DELTA);
    }

    #[test]
    fn participant_preferred_size_with_padding() {
        let tm = make_text(60.0, 14.0);
        let dim = participant_preferred_size(&tm, 0.0, false, 10.0, 0.0);
        let dim_no_pad = participant_preferred_size(&tm, 0.0, false, 0.0, 0.0);
        assert_eq!(dim.width, dim_no_pad.width + 20.0);
    }

    // ── Note size ───────────────────────────────────────────────────

    #[test]
    fn note_preferred_size_basic() {
        let tm = make_text(80.0, 14.0);
        let dim = note_preferred_size(&tm, 5.0, 5.0, 0.0);
        assert_eq!(dim.width, tm.text_width() + 10.0);
        assert_eq!(dim.height, tm.text_height() + 10.0);
    }

    #[test]
    fn note_preferred_size_with_shadow() {
        let tm = make_text(80.0, 14.0);
        let dim = note_preferred_size(&tm, 5.0, 5.0, 3.0);
        let dim_noshadow = note_preferred_size(&tm, 5.0, 5.0, 0.0);
        assert_eq!(dim.width, dim_noshadow.width + 3.0);
        assert_eq!(dim.height, dim_noshadow.height + 3.0);
    }

    // ── Divider size ────────────────────────────────────────────────

    #[test]
    fn divider_preferred_size_basic() {
        let tm = make_text(40.0, 14.0);
        let dim = divider_preferred_size(&tm);
        assert_eq!(dim.width, tm.text_width() + 30.0);
        assert_eq!(dim.height, tm.text_height() + 20.0);
    }

    // ── Grouping header size ────────────────────────────────────────

    #[test]
    fn grouping_header_preferred_size_no_comment() {
        let tm = TextMetrics::new(15.0, 30.0, 1.0, 40.0, 14.0);
        let dim = grouping_header_preferred_size(&tm, 0.0, 0.0, 5.0);
        assert_eq!(dim.width, tm.text_width());
        assert_eq!(dim.height, tm.text_height() + 10.0);
    }

    #[test]
    fn grouping_header_preferred_size_with_comment() {
        let tm = TextMetrics::new(15.0, 30.0, 1.0, 40.0, 14.0);
        let dim = grouping_header_preferred_size(&tm, 50.0, 20.0, 5.0);
        assert!(dim.width > tm.text_width());
        assert_eq!(dim.height, tm.text_height() + 10.0 + 5.0);
    }

    // ── Grouping else size ──────────────────────────────────────────

    #[test]
    fn grouping_else_preferred_size_legacy() {
        let tm = make_text(30.0, 14.0);
        let dim = grouping_else_preferred_size(&tm, false);
        assert_eq!(dim.height, tm.text_height());
    }

    #[test]
    fn grouping_else_preferred_size_teoz() {
        let tm = make_text(30.0, 14.0);
        let dim = grouping_else_preferred_size(&tm, true);
        assert_eq!(dim.height, tm.text_height() + 16.0);
    }

    // ── Grouping space size ─────────────────────────────────────────

    #[test]
    fn grouping_space_size() {
        let dim = grouping_space_preferred_size();
        assert_eq!(dim.width, 0.0);
        assert_eq!(dim.height, 7.0);
    }

    // ── Reference size ──────────────────────────────────────────────

    #[test]
    fn reference_preferred_size_basic() {
        let tm = make_text(80.0, 14.0);
        let hw = reference_header_width(30.0);
        let hh = reference_header_height(14.0);
        let dim = reference_preferred_size(&tm, hw, hh, 0.0);
        assert_eq!(dim.height, tm.text_height() + hh + REF_HEIGHT_FOOTER);
    }

    #[test]
    fn reference_header_dimensions() {
        assert_eq!(reference_header_width(30.0), 30.0 + 45.0);
        assert_eq!(reference_header_height(14.0), 16.0);
    }

    // ── Line size ───────────────────────────────────────────────────

    #[test]
    fn line_preferred() {
        let dim = line_preferred_size();
        assert_eq!(dim.width, 1.0);
        assert_eq!(dim.height, 20.0);
    }

    // ── Active line size ────────────────────────────────────────────

    #[test]
    fn active_line_preferred() {
        let dim = active_line_preferred_size();
        assert_eq!(dim.width, 10.0);
        assert_eq!(dim.height, 0.0);
    }

    // ── Destroy size ────────────────────────────────────────────────

    #[test]
    fn destroy_preferred() {
        let dim = destroy_preferred_size();
        assert_eq!(dim.width, 18.0);
        assert_eq!(dim.height, 18.0);
    }

    // ── Delay sizes ─────────────────────────────────────────────────

    #[test]
    fn delay_line_preferred() {
        let dim = delay_line_preferred_size();
        assert_eq!(dim.width, 1.0);
        assert_eq!(dim.height, 20.0);
    }

    #[test]
    fn delay_text_preferred() {
        let tm = TextMetrics::new(0.0, 0.0, 4.0, 50.0, 14.0);
        let dim = delay_text_preferred_size(&tm);
        assert_eq!(dim.width, 50.0);
        assert_eq!(dim.height, tm.text_height() + 20.0);
    }

    // ── Newpage size ────────────────────────────────────────────────

    #[test]
    fn newpage_preferred() {
        let dim = newpage_preferred_size();
        assert_eq!(dim.width, 0.0);
        assert_eq!(dim.height, 1.0);
    }

    // ── Englober size ───────────────────────────────────────────────

    #[test]
    fn englober_preferred() {
        let tm = TextMetrics::new(3.0, 3.0, 1.0, 60.0, 14.0);
        let dim = englober_preferred_size(&tm);
        assert_eq!(dim.width, tm.text_width());
        assert_eq!(dim.height, tm.text_height() + 3.0);
    }

    // ── NoteBox / NoteHexagonal sizes ───────────────────────────────

    #[test]
    fn note_box_preferred() {
        let tm = make_text(50.0, 14.0);
        let dim = note_box_preferred_size(&tm);
        assert_eq!(dim.width, tm.text_width() + 10.0);
        assert_eq!(dim.height, tm.text_height() + 10.0);
    }

    #[test]
    fn note_hexagonal_preferred() {
        let tm = make_text(50.0, 14.0);
        let dim = note_hexagonal_preferred_size(&tm);
        assert_eq!(dim.width, tm.text_width() + 10.0);
        assert_eq!(dim.height, tm.text_height() + 10.0);
    }

    // ── Constants match Java ────────────────────────────────────────

    #[test]
    fn constants_match_java() {
        assert_eq!(ARROW_DELTA_X, 10.0);
        assert_eq!(ARROW_DELTA_Y, 4.0);
        assert_eq!(ARROW_PADDING_Y, 4.0);
        assert_eq!(SPACE_CROSS_X, 6.0);
        assert_eq!(DIAM_CIRCLE, 8.0);
        assert_eq!(THIN_CIRCLE, 1.5);
        assert_eq!(SELF_ARROW_WIDTH, 45.0);
        assert_eq!(SELF_ARROW_XRIGHT, 42.0);
        assert_eq!(SELF_ARROW_ONLY_HEIGHT, 13.0);
        assert_eq!(DESTROY_CROSS_SIZE, 9.0);
        assert_eq!(CORNER_SIZE, 10.0);
        assert_eq!(GROUPING_SPACE_HEIGHT, 7.0);
        assert_eq!(REF_HEIGHT_FOOTER, 5.0);
        assert_eq!(REF_X_MARGIN, 2.0);
        assert_eq!(ACTIVE_LINE_WIDTH, 10.0);
        assert_eq!(COLLECTIONS_DELTA, 4.0);
    }

    // ── Polygon generation ──────────────────────────────────────────

    #[test]
    fn polygon_normal_full() {
        let pts = polygon_normal(ArrowPart::Full, false);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (-ARROW_DELTA_X, -ARROW_DELTA_Y));
        assert_eq!(pts[1], (0.0, 0.0));
        assert_eq!(pts[2], (-ARROW_DELTA_X, ARROW_DELTA_Y));
    }

    #[test]
    fn polygon_normal_nice() {
        let pts = polygon_normal(ArrowPart::Full, true);
        assert_eq!(pts.len(), 4);
        assert_eq!(pts[3], (-ARROW_DELTA_X + 4.0, 0.0));
    }

    #[test]
    fn polygon_normal_top_part() {
        let pts = polygon_normal(ArrowPart::TopPart, false);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[2], (-ARROW_DELTA_X, 0.0));
    }

    #[test]
    fn polygon_normal_bottom_part() {
        let pts = polygon_normal(ArrowPart::BottomPart, false);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (-ARROW_DELTA_X, 0.0));
    }

    #[test]
    fn polygon_reverse_full() {
        let pts = polygon_reverse(ArrowPart::Full, false);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (ARROW_DELTA_X, -ARROW_DELTA_Y));
        assert_eq!(pts[1], (0.0, 0.0));
        assert_eq!(pts[2], (ARROW_DELTA_X, ARROW_DELTA_Y));
    }

    #[test]
    fn polygon_reverse_nice() {
        let pts = polygon_reverse(ArrowPart::Full, true);
        assert_eq!(pts.len(), 4);
        assert_eq!(pts[3], (ARROW_DELTA_X - 4.0, 0.0));
    }

    #[test]
    fn polygon_self_forward() {
        let config = ArrowConfiguration::with_direction_self(false);
        let pts = polygon_self(&config, false);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (ARROW_DELTA_X, -ARROW_DELTA_Y));
    }

    #[test]
    fn polygon_self_reversed() {
        let config = ArrowConfiguration::with_direction_self(true);
        let pts = polygon_self(&config, false);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0], (-ARROW_DELTA_X, -ARROW_DELTA_Y));
    }

    #[test]
    fn polygon_self_nice() {
        let config = ArrowConfiguration::with_direction_self(false);
        let pts = polygon_self(&config, true);
        assert_eq!(pts.len(), 4);
    }

    // ── Arrow y-point ───────────────────────────────────────────────

    #[test]
    fn arrow_y_point_normal() {
        let tm = make_text(80.0, 14.0);
        let y = arrow_y_point(&tm, false);
        assert_eq!(y, tm.text_height() + ARROW_PADDING_Y);
    }

    #[test]
    fn arrow_y_point_below() {
        let tm = make_text(80.0, 14.0);
        let y = arrow_y_point(&tm, true);
        assert_eq!(y, ARROW_PADDING_Y);
    }

    // ── Self-arrow y-point ──────────────────────────────────────────

    #[test]
    fn self_arrow_y_point_calc() {
        let tm = make_text(30.0, 14.0);
        let y = self_arrow_y_point(&tm);
        let th = tm.text_height();
        let expected = (th + th + SELF_ARROW_ONLY_HEIGHT) / 2.0 + ARROW_PADDING_X;
        assert_eq!(y, expected);
    }

    // ── Start/end points ────────────────────────────────────────────

    #[test]
    fn arrow_start_point_ltr() {
        let tm = make_text(80.0, 14.0);
        let dim = XDimension2D::new(200.0, 30.0);
        let pt = arrow_start_point(&tm, dim, ArrowDirection::LeftToRight, false, 0.0);
        assert_eq!(pt.x, ARROW_PADDING_X);
    }

    #[test]
    fn arrow_start_point_rtl() {
        let tm = make_text(80.0, 14.0);
        let dim = XDimension2D::new(200.0, 30.0);
        let pt = arrow_start_point(&tm, dim, ArrowDirection::RightToLeft, false, 0.0);
        assert_eq!(pt.x, dim.width + ARROW_PADDING_X);
    }

    #[test]
    fn arrow_end_point_ltr() {
        let tm = make_text(80.0, 14.0);
        let dim = XDimension2D::new(200.0, 30.0);
        let pt = arrow_end_point(&tm, dim, ArrowDirection::LeftToRight, false);
        assert_eq!(pt.x, dim.width + ARROW_PADDING_X);
    }

    #[test]
    fn self_arrow_start_end() {
        let tm = make_text(30.0, 14.0);
        let start = self_arrow_start_point(&tm);
        let end = self_arrow_end_point(&tm);
        assert!(end.y > start.y);
        assert_eq!(start.x, end.x);
    }

    // ── Drawing functions produce ops ───────────────────────────────

    #[test]
    fn draw_arrow_hidden_produces_empty() {
        let hidden_config = ArrowConfiguration::with_direction_normal()
            .with_body(ArrowBody::Hidden);
        let tm = make_text(80.0, 14.0);
        let area = Area::new(200.0, 30.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 255);
        let stroke = UStroke::simple();
        let ops = draw_arrow(&hidden_config, &tm, &area, &fg, &bg, &stroke, true, false, 0.0, 0.0);
        assert!(ops.is_empty());
    }

    #[test]
    fn draw_arrow_normal_produces_ops() {
        let config = ArrowConfiguration::with_direction_normal();
        let tm = make_text(80.0, 14.0);
        let area = Area::new(200.0, 30.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 255);
        let stroke = UStroke::simple();
        let ops = draw_arrow(&config, &tm, &area, &fg, &bg, &stroke, true, false, 0.0, 0.0);
        assert!(!ops.is_empty());
        // Should have at least a line and a polygon
        let has_line = ops.iter().any(|op| matches!(op, DrawOp::Line { .. }));
        let has_poly = ops.iter().any(|op| matches!(op, DrawOp::Polygon { .. }));
        assert!(has_line);
        assert!(has_poly);
    }

    #[test]
    fn draw_arrow_async_produces_lines() {
        let config = ArrowConfiguration::with_direction_normal()
            .with_head2(ArrowHead::Async);
        let tm = make_text(80.0, 14.0);
        let area = Area::new(200.0, 30.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 255);
        let stroke = UStroke::simple();
        let ops = draw_arrow(&config, &tm, &area, &fg, &bg, &stroke, true, false, 0.0, 0.0);
        assert!(!ops.is_empty());
        // Async arrow: two angled lines instead of polygon
        let line_count = ops.iter().filter(|op| matches!(op, DrawOp::Line { .. })).count();
        assert!(line_count >= 3); // main line + 2 async head lines
    }

    #[test]
    fn draw_arrow_crossx_produces_lines() {
        let config = ArrowConfiguration::with_direction_normal()
            .with_head2(ArrowHead::CrossX);
        let tm = make_text(80.0, 14.0);
        let area = Area::new(200.0, 30.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 255);
        let stroke = UStroke::simple();
        let ops = draw_arrow(&config, &tm, &area, &fg, &bg, &stroke, true, false, 0.0, 0.0);
        let line_count = ops.iter().filter(|op| matches!(op, DrawOp::Line { .. })).count();
        assert!(line_count >= 3); // main line + 2 cross lines
    }

    #[test]
    fn draw_arrow_with_circle_decoration() {
        let config = ArrowConfiguration::with_direction_normal()
            .with_decoration1(ArrowDecoration::Circle)
            .with_decoration2(ArrowDecoration::Circle);
        let tm = make_text(80.0, 14.0);
        let area = Area::new(200.0, 30.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 255);
        let stroke = UStroke::simple();
        let ops = draw_arrow(&config, &tm, &area, &fg, &bg, &stroke, true, false, 0.0, 0.0);
        let ellipse_count = ops.iter().filter(|op| matches!(op, DrawOp::Ellipse { .. })).count();
        assert_eq!(ellipse_count, 2);
    }

    #[test]
    fn draw_self_arrow_hidden() {
        let config = ArrowConfiguration::with_direction_self(false)
            .with_body(ArrowBody::Hidden);
        let tm = make_text(30.0, 14.0);
        let area = Area::new(100.0, 40.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 255);
        let stroke = UStroke::simple();
        let ops = draw_self_arrow(&config, &tm, &area, &fg, &bg, &stroke, true);
        assert!(ops.is_empty());
    }

    #[test]
    fn draw_self_arrow_normal() {
        let config = ArrowConfiguration::with_direction_self(false);
        let tm = make_text(30.0, 14.0);
        let area = Area::new(100.0, 40.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 255);
        let stroke = UStroke::simple();
        let ops = draw_self_arrow(&config, &tm, &area, &fg, &bg, &stroke, true);
        assert!(!ops.is_empty());
        // Should have 3 bracket lines + 1 polygon
        let line_count = ops.iter().filter(|op| matches!(op, DrawOp::Line { .. })).count();
        assert!(line_count >= 3);
    }

    #[test]
    fn draw_participant_basic() {
        let tm = make_text(60.0, 14.0);
        let area = Area::new(80.0, 20.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 200);
        let stroke = UStroke::simple();
        let ops = draw_participant(&tm, &area, &fg, &bg, &stroke, 5.0, 0.0, 0.0, false, 0.0, 0.0);
        let rect_count = ops.iter().filter(|op| matches!(op, DrawOp::Rect { .. })).count();
        assert_eq!(rect_count, 1);
    }

    #[test]
    fn draw_participant_collections() {
        let tm = make_text(60.0, 14.0);
        let area = Area::new(80.0, 20.0);
        let fg = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(255, 255, 200);
        let stroke = UStroke::simple();
        let ops = draw_participant(&tm, &area, &fg, &bg, &stroke, 5.0, 0.0, 0.0, true, 0.0, 0.0);
        let rect_count = ops.iter().filter(|op| matches!(op, DrawOp::Rect { .. })).count();
        assert_eq!(rect_count, 2); // two rects for collections
    }

    #[test]
    fn draw_note_produces_path() {
        let tm = make_text(60.0, 14.0);
        let area = Area::new(100.0, 30.0);
        let fashion = Fashion::new(Some(HColor::rgb(255, 255, 200)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_note(&tm, &area, &fashion, 5.0, 5.0, 0.0);
        let path_count = ops.iter().filter(|op| matches!(op, DrawOp::Path { .. })).count();
        assert_eq!(path_count, 2); // main note shape + corner fold
    }

    #[test]
    fn draw_divider_empty() {
        let tm = make_text(0.0, 0.0);
        let area = Area::new(200.0, 20.0);
        let border = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(200, 200, 200);
        let stroke = UStroke::simple();
        let ops = draw_divider(&tm, &area, &border, &bg, &stroke, 0.0, 0.0, true);
        assert!(!ops.is_empty());
    }

    #[test]
    fn draw_divider_with_text() {
        let tm = make_text(40.0, 14.0);
        let area = Area::new(200.0, 40.0);
        let border = HColor::rgb(0, 0, 0);
        let bg = HColor::rgb(200, 200, 200);
        let stroke = UStroke::simple();
        let ops = draw_divider(&tm, &area, &border, &bg, &stroke, 5.0, 0.0, false);
        let rect_count = ops.iter().filter(|op| matches!(op, DrawOp::Rect { .. })).count();
        assert!(rect_count >= 2); // sep rect + text rect
    }

    #[test]
    fn draw_grouping_header_produces_ops() {
        let tm = TextMetrics::new(15.0, 30.0, 1.0, 40.0, 14.0);
        let area = Area::new(200.0, 30.0);
        let fashion = Fashion::new(Some(HColor::rgb(200, 200, 200)), Some(HColor::rgb(0, 0, 0)));
        let corner_fashion = Fashion::new(Some(HColor::rgb(180, 180, 180)), Some(HColor::rgb(0, 0, 0)));
        let bg = HColor::rgb(240, 240, 240);
        let ops = draw_grouping_header(&tm, &area, &fashion, &corner_fashion, &bg, 0.0);
        assert!(!ops.is_empty());
        let rect_count = ops.iter().filter(|op| matches!(op, DrawOp::Rect { .. })).count();
        assert!(rect_count >= 2); // background + outline
    }

    #[test]
    fn draw_grouping_else_produces_line() {
        let tm = make_text(30.0, 14.0);
        let area = Area::new(200.0, 20.0);
        let color = HColor::rgb(0, 0, 0);
        let ops = draw_grouping_else(&tm, &area, &color);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], DrawOp::Line { .. }));
    }

    #[test]
    fn draw_line_produces_ops() {
        let area = Area::new(10.0, 100.0);
        let color = HColor::rgb(0, 0, 0);
        let stroke = UStroke::new(5.0, 5.0, 1.0);
        let ops = draw_line(&area, &color, &stroke);
        assert!(ops.len() >= 2); // hover rect + line
    }

    #[test]
    fn draw_active_line_close_both() {
        let area = Area::new(20.0, 50.0);
        let fashion = Fashion::new(Some(HColor::rgb(255, 255, 200)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_active_line(&area, &fashion, true, true);
        let rect_count = ops.iter().filter(|op| matches!(op, DrawOp::Rect { .. })).count();
        assert_eq!(rect_count, 1);
    }

    #[test]
    fn draw_active_line_open_both() {
        let area = Area::new(20.0, 50.0);
        let fashion = Fashion::new(Some(HColor::rgb(255, 255, 200)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_active_line(&area, &fashion, false, false);
        // background rect + 2 vertical lines, no horizontal
        let line_count = ops.iter().filter(|op| matches!(op, DrawOp::Line { .. })).count();
        assert_eq!(line_count, 2);
    }

    #[test]
    fn draw_active_line_zero_height() {
        let area = Area::new(20.0, 0.0);
        let fashion = Fashion::new(Some(HColor::rgb(255, 255, 200)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_active_line(&area, &fashion, true, true);
        assert!(ops.is_empty());
    }

    #[test]
    fn draw_destroy_produces_two_lines() {
        let color = HColor::rgb(0, 0, 0);
        let stroke = UStroke::with_thickness(2.0);
        let ops = draw_destroy(&color, &stroke);
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn draw_delay_line_produces_one_line() {
        let area = Area::new(10.0, 50.0);
        let color = HColor::rgb(0, 0, 0);
        let stroke = UStroke::simple();
        let ops = draw_delay_line(&area, &color, &stroke);
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn draw_newpage_produces_one_line() {
        let area = Area::new(200.0, 1.0);
        let color = HColor::rgb(0, 0, 0);
        let stroke = UStroke::simple();
        let ops = draw_newpage(&area, &color, &stroke);
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn draw_reference_produces_rect_and_path() {
        let tm = make_text(80.0, 14.0);
        let area = Area::new(200.0, 60.0);
        let header_fashion = Fashion::new(Some(HColor::rgb(200, 200, 200)), Some(HColor::rgb(0, 0, 0)));
        let body_fashion = Fashion::new(Some(HColor::rgb(255, 255, 255)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_reference(&tm, &area, &header_fashion, &body_fashion, 30.0, 14.0, 0.0);
        let rect_count = ops.iter().filter(|op| matches!(op, DrawOp::Rect { .. })).count();
        let path_count = ops.iter().filter(|op| matches!(op, DrawOp::Path { .. })).count();
        assert_eq!(rect_count, 1);
        assert_eq!(path_count, 1);
    }

    #[test]
    fn draw_note_box_produces_rect() {
        let tm = make_text(50.0, 14.0);
        let area = Area::new(80.0, 30.0);
        let fashion = Fashion::new(Some(HColor::rgb(255, 255, 200)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_note_box(&tm, &area, &fashion, 5.0);
        let rect_count = ops.iter().filter(|op| matches!(op, DrawOp::Rect { .. })).count();
        assert_eq!(rect_count, 1);
    }

    #[test]
    fn draw_note_hexagonal_produces_polygon() {
        let tm = make_text(50.0, 14.0);
        let area = Area::new(80.0, 30.0);
        let fashion = Fashion::new(Some(HColor::rgb(255, 255, 200)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_note_hexagonal(&tm, &area, &fashion);
        let poly_count = ops.iter().filter(|op| matches!(op, DrawOp::Polygon { .. })).count();
        assert_eq!(poly_count, 1);
    }

    #[test]
    fn draw_englober_produces_rect() {
        let tm = TextMetrics::new(3.0, 3.0, 1.0, 60.0, 14.0);
        let area = Area::new(100.0, 20.0);
        let fashion = Fashion::new(Some(HColor::rgb(240, 240, 240)), Some(HColor::rgb(0, 0, 0)));
        let ops = draw_englober(&tm, &area, &fashion, 5.0);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], DrawOp::Rect { .. }));
    }

    // ── Corner paths ────────────────────────────────────────────────

    #[test]
    fn grouping_corner_path_no_round() {
        let path = grouping_corner_path(50.0, 20.0, 0.0);
        assert_eq!(path.segments.len(), 6); // move + 5 lines
    }

    #[test]
    fn grouping_corner_path_with_round() {
        let path = grouping_corner_path(50.0, 20.0, 10.0);
        // move + lines + arc
        assert!(path.segments.len() >= 7);
    }

    #[test]
    fn reference_corner_path_no_round() {
        let path = reference_corner_path(50.0, 20.0, 0.0);
        assert_eq!(path.segments.len(), 6);
    }

    #[test]
    fn reference_corner_path_with_round() {
        let path = reference_corner_path(50.0, 20.0, 10.0);
        assert!(path.segments.len() >= 7);
    }

    // ── Area ────────────────────────────────────────────────────────

    #[test]
    fn area_new() {
        let a = Area::new(100.0, 50.0);
        assert_eq!(a.dimension.width, 100.0);
        assert_eq!(a.dimension.height, 50.0);
        assert_eq!(a.delta_x1, 0.0);
    }

    #[test]
    fn area_from_dim() {
        let dim = XDimension2D::new(200.0, 100.0);
        let a = Area::from_dim(dim);
        assert_eq!(a.dimension.width, 200.0);
    }

    #[test]
    fn area_with_delta() {
        let a = Area::new(100.0, 50.0).with_delta_x1(10.0).with_text_delta_x(5.0);
        assert_eq!(a.delta_x1, 10.0);
        assert_eq!(a.text_delta_x, 5.0);
    }

    // ── DrawOp variants ─────────────────────────────────────────────

    #[test]
    fn draw_op_debug() {
        let op = DrawOp::Line {
            translate: UTranslate::none(),
            dx: 100.0,
            dy: 0.0,
            stroke: UStroke::simple(),
            color: Some(HColor::rgb(0, 0, 0)),
        };
        let s = format!("{:?}", op);
        assert!(s.contains("Line"));
    }
}
