//! Mindmap diagram layout engine.
//!
//! Converts a `MindmapDiagram` into a fully positioned `MindmapLayout` ready
//! for SVG rendering. Uses the Java PlantUML Tetris packing algorithm for
//! compact sibling placement.

use std::collections::HashMap;

use crate::font_metrics;
use crate::model::mindmap::{MindmapDiagram, MindmapNode, MindmapNote};
use crate::model::richtext::plain_text;
use crate::parser::creole::parse_creole;
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Draw order item: either a node or an edge, by index.
#[derive(Debug, Clone, Copy)]
pub enum DrawItem {
    Node(usize),
    Edge(usize),
}

/// Fully positioned mindmap layout ready for rendering.
#[derive(Debug)]
pub struct MindmapLayout {
    pub nodes: Vec<MindmapNodeLayout>,
    pub edges: Vec<MindmapEdgeLayout>,
    pub notes: Vec<MindmapNoteLayout>,
    pub width: f64,
    pub height: f64,
    /// Caption text and its position/width.
    pub caption: Option<(String, f64, f64, f64)>, // (text, x, y, text_width)
    /// Raw body dimensions for wrap_with_meta (Java MindMapDiagram TextBlock size).
    pub raw_body_dim: Option<(f64, f64)>,
    /// Interleaved draw order matching Java rendering sequence.
    pub draw_order: Vec<DrawItem>,
}

/// A positioned mindmap node.
#[derive(Debug, Clone)]
pub struct MindmapNodeLayout {
    /// Display text (may contain `\n` for multiline).
    pub text: String,
    /// Top-left x coordinate.
    pub x: f64,
    /// Top-left y coordinate.
    pub y: f64,
    /// Box width.
    pub width: f64,
    /// Box height.
    pub height: f64,
    /// Tree depth level (1 = root).
    pub level: usize,
    /// Text lines (split on `\n`).
    pub lines: Vec<String>,
}

/// A connection line between parent and child nodes.
#[derive(Debug, Clone)]
pub struct MindmapEdgeLayout {
    /// Parent node right-center x.
    pub from_x: f64,
    /// Parent node right-center y.
    pub from_y: f64,
    /// Child node left-center x.
    pub to_x: f64,
    /// Child node left-center y.
    pub to_y: f64,
}

/// A positioned note annotation attached to the root node.
#[derive(Debug, Clone)]
pub struct MindmapNoteLayout {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub connector: Option<(f64, f64, f64, f64)>,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Font size for mindmap nodes.
const FONT_SIZE: f64 = 14.0;
/// Line height in pixels.
const LINE_HEIGHT: f64 = 16.0;
/// Horizontal padding inside nodes.
const H_PADDING: f64 = 8.0;
/// Vertical padding inside nodes.
const V_PADDING: f64 = 4.0;
/// Minimum node width.
const MIN_NODE_WIDTH: f64 = 40.0;
/// Minimum node height.
const MIN_NODE_HEIGHT: f64 = 24.0;
/// Canvas margin around the diagram (Java ImageBuilder default: 10px).
const MARGIN: f64 = 10.0;
/// Gap between a node and an attached note.
const NOTE_GAP: f64 = 16.0;
/// Minimum note width.
const MIN_NOTE_WIDTH: f64 = 60.0;
/// Minimum note height.
const MIN_NOTE_HEIGHT: f64 = 28.0;

/// Default node Margin from plantuml.skin (mindmapDiagram.node.Margin).
const DEFAULT_NODE_MARGIN: f64 = 10.0;

/// Java FingerImpl: getX2() = margin.getRight() + 30, getX1() = margin.getLeft().
/// Base gap added to margin.getRight() in getX2().
const X2_BASE_GAP: f64 = 30.0;

// ---------------------------------------------------------------------------
// Text measurement helpers
// ---------------------------------------------------------------------------

/// Split text on `\n` escape sequences and return individual lines.
/// Java preserves whitespace (affects node width calculation and centering).
fn split_text_lines(text: &str) -> Vec<String> {
    text.split("\\n")
        .flat_map(|s| s.split(crate::NEWLINE_CHAR))
        .map(|s| s.to_string())
        .collect()
}

/// Estimate the rendered size of a node based on its text.
fn estimate_node_size(text: &str, is_root: bool) -> (f64, f64, Vec<String>) {
    estimate_node_size_styled(text, is_root, H_PADDING, V_PADDING)
}

/// Estimate node size with custom padding values from style.
fn estimate_node_size_styled(
    text: &str,
    is_root: bool,
    h_pad: f64,
    v_pad: f64,
) -> (f64, f64, Vec<String>) {
    let lines = split_text_lines(text);
    let max_line_width = lines
        .iter()
        .map(|l| font_metrics::text_width(l, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);

    let has_custom_padding = h_pad != H_PADDING || v_pad != V_PADDING;
    let text_h = font_metrics::line_height("SansSerif", FONT_SIZE, false, false);
    let base_width = max_line_width + 2.0 * h_pad;
    let base_height = if has_custom_padding {
        // When padding is customized (e.g. Padding 0), use exact text height
        lines.len() as f64 * text_h + 2.0 * v_pad
    } else {
        lines.len() as f64 * LINE_HEIGHT + 2.0 * v_pad
    };

    let (width, height) = if has_custom_padding {
        // With custom padding, don't apply min sizes or root extras
        (base_width, base_height)
    } else {
        let w = if is_root {
            base_width.max(MIN_NODE_WIDTH) + 16.0
        } else {
            base_width.max(MIN_NODE_WIDTH)
        };
        let h = if is_root {
            base_height.max(MIN_NODE_HEIGHT) + 8.0
        } else {
            base_height.max(MIN_NODE_HEIGHT)
        };
        (w, h)
    };

    (width, height, lines)
}

fn plain_text_lines(text: &str) -> Vec<String> {
    let plain = plain_text(&parse_creole(text));
    let normalized = plain
        .replace("\\n", "\n")
        .replace(crate::NEWLINE_CHAR, "\n");
    let lines: Vec<String> = normalized
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn estimate_note_size(text: &str) -> (f64, f64) {
    let lines = plain_text_lines(text);
    let max_line_width = lines
        .iter()
        .map(|line| font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max);
    let width = (max_line_width + 2.0 * H_PADDING).max(MIN_NOTE_WIDTH);
    let height = (lines.len().max(1) as f64 * LINE_HEIGHT + 2.0 * V_PADDING).max(MIN_NOTE_HEIGHT);
    (width, height)
}

// ===========================================================================
// Tetris layout algorithm (ported from Java PlantUML)
// ===========================================================================

/// A T-shaped element representing a node and its children subtree.
/// In left-to-right mode:
///   thickness = vertical extent (height)
///   elongation = horizontal extent (width)
#[derive(Debug, Clone)]
struct SymetricalTee {
    thickness1: f64,   // phalanx (node) thickness
    elongation1: f64,  // phalanx (node) elongation
    thickness2: f64,   // nail (children) thickness
    elongation2: f64,  // nail (children) elongation
}

impl SymetricalTee {
    fn full_thickness(&self) -> f64 {
        self.thickness1.max(self.thickness2)
    }
    fn full_elongation(&self) -> f64 {
        self.elongation1 + self.elongation2
    }
}

/// A T-shape placed at a specific y position (center of the T-shape).
#[derive(Debug, Clone)]
struct SymetricalTeePositioned {
    tee: SymetricalTee,
    y: f64,
}

impl SymetricalTeePositioned {
    fn new(tee: SymetricalTee) -> Self {
        Self { tee, y: 0.0 }
    }

    /// Top edge of the phalanx (left part of the T).
    fn segment_a1_y(&self) -> f64 {
        self.y - self.tee.thickness1 / 2.0
    }

    /// Bottom edge of the phalanx.
    fn segment_b1_y(&self) -> f64 {
        self.y + self.tee.thickness1 / 2.0
    }

    /// Top edge of the nail (right part of the T).
    fn segment_a2_y(&self) -> f64 {
        self.y - self.tee.thickness2 / 2.0
    }

    /// Bottom edge of the nail.
    fn segment_b2_y(&self) -> f64 {
        self.y + self.tee.thickness2 / 2.0
    }

    fn max_x(&self) -> f64 {
        self.tee.elongation1 + self.tee.elongation2
    }

    fn min_y(&self) -> f64 {
        self.y - self.tee.full_thickness() / 2.0
    }

    fn max_y(&self) -> f64 {
        self.y + self.tee.full_thickness() / 2.0
    }

    fn move_so_that_segment_a1_is_on(&mut self, new_y: f64) {
        let current = self.segment_a1_y();
        self.y += new_y - current;
    }

    fn move_so_that_segment_a2_is_on(&mut self, new_y: f64) {
        let current = self.segment_a2_y();
        self.y += new_y - current;
    }

    fn shift(&mut self, delta: f64) {
        self.y += delta;
    }
}

/// A frontier of horizontal line segments used for Tetris packing.
/// Tracks the lowest y-value that is "occupied" for each x-range.
#[derive(Debug, Clone)]
struct Stripe {
    start: f64,
    end: f64,
    value: f64,
}

impl Stripe {
    fn contains(&self, x: f64) -> bool {
        x >= self.start && x < self.end
    }
}

impl PartialEq for Stripe {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
    }
}
impl Eq for Stripe {}
impl PartialOrd for Stripe {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Stripe {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.partial_cmp(&other.start).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Debug, Clone)]
struct StripeFrontier {
    stripes: Vec<Stripe>,
}

impl StripeFrontier {
    fn new() -> Self {
        Self {
            stripes: vec![Stripe {
                start: f64::NEG_INFINITY,
                end: f64::INFINITY,
                value: f64::NEG_INFINITY,
            }],
        }
    }

    fn is_empty(&self) -> bool {
        self.stripes.len() == 1
    }

    /// Find the maximum y-value (frontier height) in the x-range [x1, x2).
    fn get_contact(&self, x1: f64, x2: f64) -> f64 {
        let collisions = self.collisionning(x1, x2);
        collisions.iter().map(|s| s.value).fold(f64::NEG_INFINITY, f64::max)
    }

    /// Add a horizontal segment to the frontier.
    fn add_segment(&mut self, x1: f64, x2: f64, value: f64) {
        if x2 <= x1 {
            return;
        }
        let collisions = self.collisionning(x1, x2);
        if collisions.len() > 1 {
            // Split into sub-segments at collision boundaries
            let mut x = x1;
            for i in 1..collisions.len() {
                let boundary = collisions[i].start;
                self.add_segment(x, boundary, value);
                x = boundary;
            }
            self.add_segment(x, x2, value);
        } else if let Some(touch) = collisions.into_iter().next() {
            self.add_single_internal(x1, x2, value, touch);
        }
    }

    fn add_single_internal(&mut self, x1: f64, x2: f64, value: f64, touch: Stripe) {
        if value <= touch.value {
            return;
        }
        // Remove the touched stripe
        self.stripes.retain(|s| !(s.start == touch.start && s.end == touch.end));
        // Add prefix (unmodified part before x1)
        if touch.start != x1 {
            self.stripes.push(Stripe {
                start: touch.start,
                end: x1,
                value: touch.value,
            });
        }
        // Add new segment
        self.stripes.push(Stripe {
            start: x1,
            end: x2,
            value,
        });
        // Add suffix (unmodified part after x2)
        if x2 != touch.end {
            self.stripes.push(Stripe {
                start: x2,
                end: touch.end,
                value: touch.value,
            });
        }
        self.stripes.sort();
    }

    /// Find all stripes that overlap with [x1, x2).
    fn collisionning(&self, x1: f64, x2: f64) -> Vec<Stripe> {
        let mut result = Vec::new();
        for stripe in &self.stripes {
            if x1 >= stripe.end {
                continue;
            }
            result.push(stripe.clone());
            if x2 <= stripe.end {
                return result;
            }
        }
        result
    }
}

/// Tetris packing of T-shapes. Stacks children compactly using a frontier.
#[derive(Debug)]
struct Tetris {
    frontier: StripeFrontier,
    elements: Vec<SymetricalTeePositioned>,
    min_y: f64,
    max_y: f64,
}

impl Tetris {
    fn new() -> Self {
        Self {
            frontier: StripeFrontier::new(),
            elements: Vec::new(),
            min_y: f64::MAX,
            max_y: f64::NEG_INFINITY,
        }
    }

    fn add(&mut self, tee: SymetricalTee) {
        if self.frontier.is_empty() {
            let stp = SymetricalTeePositioned::new(tee);
            self.add_internal(stp);
            return;
        }

        let c1 = self.frontier.get_contact(0.0, tee.elongation1);
        let c2 = self
            .frontier
            .get_contact(tee.elongation1, tee.elongation1 + tee.elongation2);

        let mut p1 = SymetricalTeePositioned::new(tee.clone());
        p1.move_so_that_segment_a1_is_on(c1);

        let mut p2 = SymetricalTeePositioned::new(tee);
        p2.move_so_that_segment_a2_is_on(c2);

        // Choose the one that positions lower (further down)
        let result = if p2.y > p1.y { p2 } else { p1 };
        self.add_internal(result);
    }

    fn add_internal(&mut self, stp: SymetricalTeePositioned) {
        // Add phalanx bottom edge to frontier
        let b1_y = stp.segment_b1_y();
        self.frontier
            .add_segment(0.0, stp.tee.elongation1, b1_y);

        // Add nail bottom edge to frontier (if it has width)
        let b2_x1 = stp.tee.elongation1;
        let b2_x2 = stp.tee.elongation1 + stp.tee.elongation2;
        if b2_x1 != b2_x2 {
            let b2_y = stp.segment_b2_y();
            self.frontier.add_segment(b2_x1, b2_x2, b2_y);
        }

        self.elements.push(stp);
    }

    /// Center-balance all elements around y=0.
    fn balance(&mut self) {
        if self.elements.is_empty() {
            return;
        }
        for elem in &self.elements {
            self.min_y = self.min_y.min(elem.min_y());
            self.max_y = self.max_y.max(elem.max_y());
        }
        let mean = (self.min_y + self.max_y) / 2.0;
        for elem in &mut self.elements {
            elem.shift(-mean);
        }
    }

    fn height(&self) -> f64 {
        if self.elements.is_empty() {
            return 0.0;
        }
        self.max_y - self.min_y
    }

    fn width(&self) -> f64 {
        self.elements
            .iter()
            .map(|e| e.max_x())
            .fold(0.0_f64, f64::max)
    }
}

// ---------------------------------------------------------------------------
// Finger: a node + its children subtree
// ---------------------------------------------------------------------------

/// Recursive finger structure mirroring Java FingerImpl.
/// Each finger has a phalanx (the node itself) and a nail (children).
#[derive(Debug)]
struct Finger {
    /// Node box dimensions (width, height) — without margin.
    box_width: f64,
    box_height: f64,
    /// Node text and metadata.
    text: String,
    lines: Vec<String>,
    level: usize,
    /// Node margin (top, right, bottom, left) from style.
    margin_top: f64,
    margin_right: f64,
    margin_bottom: f64,
    margin_left: f64,
    /// Child fingers.
    children: Vec<Finger>,
    /// Tetris layout of children (computed lazily).
    tetris: Option<Tetris>,
}

impl Finger {
    fn build(node: &MindmapNode, h_pad: f64, v_pad: f64, node_margin: f64) -> Self {
        let is_root = node.level == 1;
        let (w, h, lines) = estimate_node_size_styled(&node.text, is_root, h_pad, v_pad);
        let children: Vec<Finger> = node
            .children
            .iter()
            .map(|c| Finger::build(c, h_pad, v_pad, node_margin))
            .collect();
        Finger {
            box_width: w,
            box_height: h,
            text: node.text.clone(),
            lines,
            level: node.level,
            margin_top: node_margin,
            margin_right: node_margin,
            margin_bottom: node_margin,
            margin_left: node_margin,
            children,
            tetris: None,
        }
    }

    /// Phalanx thickness in LR mode = box height + margin.top + margin.bottom.
    /// Java: TextBlockUtils.withMargin(box, 0, 0, margin.getTop(), margin.getBottom()).
    fn phalanx_thickness(&self) -> f64 {
        self.box_height + self.margin_top + self.margin_bottom
    }

    /// Phalanx elongation in LR mode = box width (no left/right margin on phalanx).
    fn phalanx_elongation(&self) -> f64 {
        self.box_width
    }

    /// Java: getX1() = margin.getLeft() (LR mode).
    fn x1(&self) -> f64 {
        self.margin_left
    }

    /// Java: getX2() = margin.getRight() + 30 (LR mode).
    fn x2(&self) -> f64 {
        self.margin_right + X2_BASE_GAP
    }

    /// Java: getX12() = getX1() + getX2().
    fn x12(&self) -> f64 {
        self.x1() + self.x2()
    }

    fn ensure_tetris(&mut self) {
        if self.tetris.is_some() {
            return;
        }
        let mut tetris = Tetris::new();
        for child in &mut self.children {
            tetris.add(child.as_symetrical_tee());
        }
        tetris.balance();
        self.tetris = Some(tetris);
    }

    /// Convert this finger to a SymetricalTee for packing in the parent's Tetris.
    /// Java: FingerImpl.asSymetricalTee().
    fn as_symetrical_tee(&mut self) -> SymetricalTee {
        let thickness1 = self.phalanx_thickness();
        let elongation1 = self.phalanx_elongation();

        if self.children.is_empty() {
            return SymetricalTee {
                thickness1,
                elongation1,
                thickness2: 0.0,
                elongation2: 0.0,
            };
        }

        self.ensure_tetris();
        let tetris = self.tetris.as_ref().unwrap();
        let thickness2 = tetris.height();
        // Java: new SymetricalTee(thickness1, elongation1 + getX1(), thickness2, getX2() + elongation2)
        let elongation2 = self.x2() + tetris.width();

        SymetricalTee {
            thickness1,
            elongation1: elongation1 + self.x1(),
            thickness2,
            elongation2,
        }
    }

    /// Get the full thickness (max of phalanx and nail heights).
    fn full_thickness(&mut self) -> f64 {
        let tee = self.as_symetrical_tee();
        tee.full_thickness()
    }

    /// Get the full elongation (phalanx width + nail width).
    fn full_elongation(&mut self) -> f64 {
        let tee = self.as_symetrical_tee();
        tee.full_elongation()
    }

    /// Recursively collect positioned nodes and edges in Java draw order.
    /// Java order: phalanx (node), then for each child: child.drawU (recurse), then edge.
    /// `origin_x`, `origin_y` is the center point of this finger in absolute coordinates.
    fn collect(
        &mut self,
        origin_x: f64,
        origin_y: f64,
        nodes_out: &mut Vec<MindmapNodeLayout>,
        edges_out: &mut Vec<MindmapEdgeLayout>,
        draw_order: &mut Vec<DrawItem>,
    ) {
        let phalanx_top = origin_y - self.phalanx_thickness() / 2.0;
        let node_x = origin_x;
        let node_y = phalanx_top + self.margin_top;

        // 1. Draw phalanx (node)
        let node_idx = nodes_out.len();
        nodes_out.push(MindmapNodeLayout {
            text: self.text.clone(),
            x: node_x,
            y: node_y,
            width: self.box_width,
            height: self.box_height,
            level: self.level,
            lines: self.lines.clone(),
        });
        draw_order.push(DrawItem::Node(node_idx));

        if self.children.is_empty() {
            return;
        }

        self.ensure_tetris();
        let tetris = self.tetris.as_ref().unwrap();

        let edge_from_x = origin_x + self.box_width;
        let edge_from_y = origin_y;
        let child_base_x = origin_x + self.box_width + self.x12();

        // 2. For each child: recurse (draws child subtree), then draw edge
        for (i, child) in self.children.iter_mut().enumerate() {
            let stp = &tetris.elements[i];
            let child_center_y = origin_y + stp.y;

            // Recurse: draw child subtree first
            child.collect(child_base_x, child_center_y, nodes_out, edges_out, draw_order);

            // Then draw edge from parent to child
            let edge_idx = edges_out.len();
            edges_out.push(MindmapEdgeLayout {
                from_x: edge_from_x,
                from_y: edge_from_y,
                to_x: child_base_x,
                to_y: child_center_y,
            });
            draw_order.push(DrawItem::Edge(edge_idx));
        }
    }
}

// ---------------------------------------------------------------------------
// Note layout
// ---------------------------------------------------------------------------

fn layout_notes(notes: &[MindmapNote], root: &MindmapNodeLayout) -> Vec<MindmapNoteLayout> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut result = Vec::new();
    let root_center_x = root.x + root.width / 2.0;
    let root_center_y = root.y + root.height / 2.0;

    for note in notes {
        let (width, height) = estimate_note_size(&note.text);
        let stack_index = {
            let count = counts.entry(note.position.as_str()).or_insert(0);
            let current = *count as f64;
            *count += 1;
            current
        };

        let (x, y, connector) = match note.position.as_str() {
            "left" => {
                let x = root.x - NOTE_GAP - width;
                let y = root.y + stack_index * (height + NOTE_GAP);
                let connector = Some((root.x, root_center_y, x + width, y + height / 2.0));
                (x, y, connector)
            }
            "top" => {
                let x = root_center_x - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                let y = root.y - NOTE_GAP - height;
                let connector = Some((root_center_x, root.y, x + width / 2.0, y + height));
                (x, y, connector)
            }
            "bottom" => {
                let x = root_center_x - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                let y = root.y + root.height + NOTE_GAP;
                let connector = Some((root_center_x, root.y + root.height, x + width / 2.0, y));
                (x, y, connector)
            }
            _ => {
                let x = root.x + root.width + NOTE_GAP;
                let y = root.y + stack_index * (height + NOTE_GAP);
                let connector = Some((root.x + root.width, root_center_y, x, y + height / 2.0));
                (x, y, connector)
            }
        };

        result.push(MindmapNoteLayout {
            text: note.text.clone(),
            x,
            y,
            width,
            height,
            connector,
        });
    }

    result
}

/// Compute the maximum x extent of all nodes.
fn max_right_extent(nodes: &[MindmapNodeLayout]) -> f64 {
    nodes.iter().map(|n| n.x + n.width).fold(0.0_f64, f64::max)
}

/// Compute the maximum y extent of all nodes.
fn max_bottom_extent(nodes: &[MindmapNodeLayout]) -> f64 {
    nodes.iter().map(|n| n.y + n.height).fold(0.0_f64, f64::max)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Lay out a mindmap diagram into positioned nodes and edges.
pub fn layout_mindmap(
    diagram: &MindmapDiagram,
    skin: &crate::style::SkinParams,
) -> Result<MindmapLayout> {
    // Extract style overrides for node padding
    let h_pad = skin
        .get("node.padding")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(H_PADDING);
    let v_pad = skin
        .get("node.padding")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(V_PADDING);

    // Node margin from style (default: 10 from plantuml.skin)
    let node_margin = skin
        .get("node.margin")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(DEFAULT_NODE_MARGIN);

    let mut root_finger = Finger::build(&diagram.root, h_pad, v_pad, node_margin);

    // Compute the tree dimensions (Java MindMap.calculateDimensionSlow)
    let half_thickness = root_finger.full_thickness() / 2.0;

    // Java MindMap.drawU applies UTranslate(x=reverse.getX12(), y=y)
    // For right-only mindmaps: x=0, y=half_thickness.
    // Then ImageBuilder adds margin(10,10,10,10).
    // So the root phalanx center is at (MARGIN, MARGIN + half_thickness).
    let origin_x = MARGIN;
    let origin_y = MARGIN + half_thickness;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut draw_order = Vec::new();

    root_finger.collect(origin_x, origin_y, &mut nodes, &mut edges, &mut draw_order);

    let root = nodes
        .iter()
        .find(|node| node.level == 1)
        .cloned()
        .unwrap_or_else(|| nodes[0].clone());
    let mut notes = layout_notes(&diagram.notes, &root);

    let mut min_x = nodes.iter().map(|n| n.x).fold(f64::INFINITY, f64::min);
    let mut min_y = nodes.iter().map(|n| n.y).fold(f64::INFINITY, f64::min);
    let mut max_x = max_right_extent(&nodes);
    let mut max_y = max_bottom_extent(&nodes);
    for note in &notes {
        min_x = min_x.min(note.x);
        min_y = min_y.min(note.y);
        max_x = max_x.max(note.x + note.width);
        max_y = max_y.max(note.y + note.height);
    }

    let shift_x = if min_x < MARGIN { MARGIN - min_x } else { 0.0 };
    let shift_y = if min_y < MARGIN { MARGIN - min_y } else { 0.0 };

    if shift_x > 0.0 || shift_y > 0.0 {
        for node in &mut nodes {
            node.x += shift_x;
            node.y += shift_y;
        }
        for edge in &mut edges {
            edge.from_x += shift_x;
            edge.to_x += shift_x;
            edge.from_y += shift_y;
            edge.to_y += shift_y;
        }
        for note in &mut notes {
            note.x += shift_x;
            note.y += shift_y;
            if let Some((x1, y1, x2, y2)) = note.connector.as_mut() {
                *x1 += shift_x;
                *x2 += shift_x;
                *y1 += shift_y;
                *y2 += shift_y;
            }
        }
        max_x += shift_x;
        max_y += shift_y;
    }

    // Java canvas dimensions:
    //   MindMap height = fullThickness (logical height of the tree)
    //   ImageBuilder adds margin 10 on each side
    //   Final height = fullThickness + 20
    //
    // Use fullThickness for height, because the Tetris layout may allocate
    // vertical space beyond the actual node bounding boxes.
    // Java: MindMapDiagram returns (width + 10, fullThickness).
    //       ImageBuilder adds 10px margins → final = (w+30, fullThickness+20).
    let full_thickness = 2.0 * half_thickness;
    let width = max_x + 2.0 * MARGIN;
    let height = full_thickness + 2.0 * MARGIN;

    // Raw body dimensions: Java MindMapDiagram.calculateDimension returns
    // (MindMap.width + 10, MindMap.height) = (fullElongation + x12 + 10, fullThickness).
    // This is the "TextBlock" size before ImageBuilder adds margins.
    // wrap_with_meta will use this instead of extracting from SVG header.
    let raw_body_w = max_x - MARGIN + MARGIN; // max_x already includes left margin
    let raw_body_h = full_thickness;
    let raw_body_dim = Some((raw_body_w, raw_body_h));

    // Caption is handled by wrap_with_meta in the rendering pipeline.
    let caption = None;

    log::debug!(
        "layout_mindmap: {} nodes, {} edges, canvas {}x{}",
        nodes.len(),
        edges.len(),
        width,
        height
    );

    Ok(MindmapLayout {
        nodes,
        edges,
        notes,
        width,
        height,
        caption,
        raw_body_dim,
        draw_order,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mindmap::{MindmapDiagram, MindmapNode, MindmapNote};

    fn simple_diagram() -> MindmapDiagram {
        let mut root = MindmapNode::new("Root", 1);
        root.children.push(MindmapNode::new("Child1", 2));
        root.children.push(MindmapNode::new("Child2", 2));
        MindmapDiagram {
            root,
            notes: vec![],
            caption: None,
        }
    }

    fn deep_diagram() -> MindmapDiagram {
        let mut root = MindmapNode::new("Root", 1);
        let mut child = MindmapNode::new("A", 2);
        let mut grandchild = MindmapNode::new("A1", 3);
        grandchild.children.push(MindmapNode::new("A1a", 4));
        child.children.push(grandchild);
        root.children.push(child);
        root.children.push(MindmapNode::new("B", 2));
        MindmapDiagram {
            root,
            notes: vec![],
            caption: None,
        }
    }

    #[test]
    fn layout_simple_produces_correct_node_count() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.nodes.len(), 3);
    }

    #[test]
    fn layout_simple_produces_correct_edge_count() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.edges.len(), 2);
    }

    #[test]
    fn layout_root_is_leftmost() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        let root = &layout.nodes[0];
        for node in &layout.nodes[1..] {
            assert!(
                node.x > root.x,
                "child x ({}) should be > root x ({})",
                node.x,
                root.x
            );
        }
    }

    #[test]
    fn layout_children_at_same_x() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        let child1_x = layout.nodes[1].x;
        let child2_x = layout.nodes[2].x;
        assert!(
            (child1_x - child2_x).abs() < 0.001,
            "siblings should have same x: {} vs {}",
            child1_x,
            child2_x
        );
    }

    #[test]
    fn layout_children_vertically_ordered() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert!(
            layout.nodes[1].y < layout.nodes[2].y,
            "first child should be above second"
        );
    }

    #[test]
    fn layout_canvas_positive_dimensions() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn layout_deep_correct_node_count() {
        let diagram = deep_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.nodes.len(), 5);
    }

    #[test]
    fn layout_deep_correct_edge_count() {
        let diagram = deep_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.edges.len(), 4);
    }

    #[test]
    fn layout_node_levels_correct() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.nodes[0].level, 1);
        assert_eq!(layout.nodes[1].level, 2);
        assert_eq!(layout.nodes[2].level, 2);
    }

    #[test]
    fn layout_single_node() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("Alone", 1),
            notes: vec![],
            caption: None,
        };
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.edges.len(), 0);
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn layout_nodes_have_positive_dimensions() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        for node in &layout.nodes {
            assert!(node.width > 0.0, "node width should be positive");
            assert!(node.height > 0.0, "node height should be positive");
        }
    }

    #[test]
    fn layout_edges_connect_parent_to_child() {
        let diagram = simple_diagram();
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        // Each edge from_x should be root's right side, to_x should be child's left
        let root = &layout.nodes[0];
        for edge in &layout.edges {
            assert!(
                (edge.from_x - (root.x + root.width)).abs() < 0.001,
                "edge from_x should be root right edge"
            );
        }
    }

    #[test]
    fn estimate_node_size_basic() {
        let (w, h, lines) = estimate_node_size("hello", false);
        assert!(w >= MIN_NODE_WIDTH);
        assert!(h >= MIN_NODE_HEIGHT);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn estimate_node_size_multiline() {
        let (_, h, lines) = estimate_node_size("line1\\nline2\\nline3", false);
        assert_eq!(lines.len(), 3);
        assert!(h > MIN_NODE_HEIGHT);
    }

    #[test]
    fn estimate_node_size_root_larger() {
        let (w_root, h_root, _) = estimate_node_size("test", true);
        let (w_child, h_child, _) = estimate_node_size("test", false);
        assert!(w_root > w_child);
        assert!(h_root > h_child);
    }

    #[test]
    fn split_text_lines_single() {
        let lines = split_text_lines("hello");
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn split_text_lines_multi() {
        // Java preserves whitespace for node width calculation
        let lines = split_text_lines("a \\n b \\n c");
        assert_eq!(lines, vec!["a ", " b ", " c"]);
    }

    #[test]
    fn layout_multiline_node() {
        let mut root = MindmapNode::new("Root", 1);
        root.children
            .push(MindmapNode::new("Line1\\nLine2\\nLine3", 2));
        let diagram = MindmapDiagram {
            root,
            notes: vec![],
            caption: None,
        };
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.nodes.len(), 2);
        assert_eq!(layout.nodes[1].lines.len(), 3);
    }

    #[test]
    fn layout_wide_tree() {
        let mut root = MindmapNode::new("Root", 1);
        for i in 0..10 {
            root.children
                .push(MindmapNode::new(&format!("Child{}", i), 2));
        }
        let diagram = MindmapDiagram {
            root,
            notes: vec![],
            caption: None,
        };
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.nodes.len(), 11);
        assert_eq!(layout.edges.len(), 10);
        // Verify all children are vertically ordered
        for i in 1..10 {
            assert!(layout.nodes[i].y < layout.nodes[i + 1].y);
        }
    }

    #[test]
    fn layout_note_attaches_to_root() {
        let diagram = MindmapDiagram {
            root: MindmapNode::new("Root", 1),
            notes: vec![MindmapNote {
                text: "hello".to_string(),
                position: "right".to_string(),
            }],
            caption: None,
        };
        let layout = layout_mindmap(&diagram, &crate::style::SkinParams::default()).unwrap();
        assert_eq!(layout.notes.len(), 1);
        assert!(layout.notes[0].x > layout.nodes[0].x + layout.nodes[0].width);
        assert!(layout.notes[0].connector.is_some());
    }

    #[test]
    fn tetris_basic_packing() {
        // Two elements of same size should pack tightly
        let mut tetris = Tetris::new();
        tetris.add(SymetricalTee {
            thickness1: 10.0,
            elongation1: 20.0,
            thickness2: 0.0,
            elongation2: 0.0,
        });
        tetris.add(SymetricalTee {
            thickness1: 10.0,
            elongation1: 20.0,
            thickness2: 0.0,
            elongation2: 0.0,
        });
        tetris.balance();
        assert_eq!(tetris.elements.len(), 2);
        assert!((tetris.height() - 20.0).abs() < 0.001);
    }

    #[test]
    fn tetris_t_shape_packing() {
        // A T-shape with children should pack compactly
        let mut tetris = Tetris::new();
        tetris.add(SymetricalTee {
            thickness1: 10.0,
            elongation1: 20.0,
            thickness2: 0.0,
            elongation2: 0.0,
        });
        tetris.add(SymetricalTee {
            thickness1: 10.0,
            elongation1: 20.0,
            thickness2: 30.0,
            elongation2: 40.0,
        });
        tetris.balance();
        assert_eq!(tetris.elements.len(), 2);
        // The T-shape nail extends below, so total height should be > 20
        assert!(tetris.height() > 20.0);
    }
}
