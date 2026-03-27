// decoration::symbol - UML symbol shapes (component, database, cloud, etc.)
// Port of Java PlantUML's decoration.symbol package (30 files)

use crate::klimt::geom::XDimension2D;
use crate::klimt::shape::{UEllipse, ULine, UPath, URectangle};

// ── SymbolMargin ────────────────────────────────────────────────────

/// Margin specification for a UML symbol.
/// Java: `USymbol.Margin`
#[derive(Debug, Clone, Copy)]
pub struct SymbolMargin {
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

impl SymbolMargin {
    pub fn new(x1: f64, x2: f64, y1: f64, y2: f64) -> Self {
        Self { x1, x2, y1, y2 }
    }
    pub fn width(&self) -> f64 {
        self.x1 + self.x2
    }
    pub fn height(&self) -> f64 {
        self.y1 + self.y2
    }
    pub fn add_dimension(&self, dim: XDimension2D) -> XDimension2D {
        XDimension2D::new(
            dim.width + self.x1 + self.x2,
            dim.height + self.y1 + self.y2,
        )
    }
}

// ── USymbolKind ─────────────────────────────────────────────────────

/// UML symbol type enumeration.
/// Java: `decoration.symbol.USymbols` registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum USymbolKind {
    Action,
    ActorStickman,
    ActorAwesome,
    ActorHollow,
    ActorBusiness,
    Agent,
    Archimate,
    Artifact,
    Boundary,
    Card,
    Cloud,
    Collections,
    Component1,
    Component2,
    ComponentRectangle,
    Control,
    Database,
    EntityDomain,
    File,
    Folder,
    Frame,
    Group,
    Hexagon,
    Interface,
    Label,
    Node,
    Package,
    Person,
    Process,
    Queue,
    Rectangle,
    SimpleAbstract,
    Stack,
    Storage,
    Usecase,
}

impl USymbolKind {
    /// Get the margin for this symbol type.
    /// Java: each USymbol subclass defines its own margin via getMargin()
    pub fn margin(&self) -> SymbolMargin {
        match self {
            // Java: USymbolAction.getMargin() => Margin(10, 20, 10, 10)
            Self::Action => SymbolMargin::new(10.0, 20.0, 10.0, 10.0),
            // Java: USymbolComponent1.getMargin() => Margin(10, 10, 10, 10)
            Self::Component1 => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            // Java: USymbolComponent2.getMargin() => Margin(10+5, 20+5, 15+5, 5+5)
            Self::Component2 => SymbolMargin::new(15.0, 25.0, 20.0, 10.0),
            // Java: USymbolDatabase.getMargin() => Margin(10, 10, 24, 5)
            Self::Database => SymbolMargin::new(10.0, 10.0, 24.0, 5.0),
            // Java: USymbolCloud.getMargin() => Margin(15, 15, 15, 15) (NEW=true)
            Self::Cloud => SymbolMargin::new(15.0, 15.0, 15.0, 15.0),
            // Java: USymbolFolder.getMargin() => Margin(10, 10+10, 10+3, 10)
            Self::Folder => SymbolMargin::new(10.0, 20.0, 13.0, 10.0),
            // Java: USymbolFrame.getMargin() => Margin(10+5, 20+5, 15+5, 5+5)
            Self::Frame | Self::Group => SymbolMargin::new(15.0, 25.0, 20.0, 10.0),
            // Java: USymbolNode.getMargin() => Margin(10+5, 20+5, 15+5, 5+5)
            Self::Node => SymbolMargin::new(15.0, 25.0, 20.0, 10.0),
            // Java: USymbolStorage.getMargin() => Margin(10, 10, 10, 10)
            Self::Storage => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            // Java: USymbolArtifact.getMargin() => Margin(10, 10+10, 10+3, 10)
            Self::Artifact => SymbolMargin::new(10.0, 20.0, 13.0, 10.0),
            // Java: USymbolCard.getMargin() => Margin(10, 10, 3, 3)
            Self::Card => SymbolMargin::new(10.0, 10.0, 3.0, 3.0),
            // Java: USymbolPackage = USymbolFolder(SName.package_, true)
            // same margin as Folder: Margin(10, 10+10, 10+3, 10)
            Self::Package => SymbolMargin::new(10.0, 20.0, 13.0, 10.0),
            // Java: USymbolQueue.getMargin() => Margin(5, 15, 5, 5)
            Self::Queue => SymbolMargin::new(5.0, 15.0, 5.0, 5.0),
            // Java: USymbolStack.getMargin() => Margin(25, 25, 10, 10)
            Self::Stack => SymbolMargin::new(25.0, 25.0, 10.0, 10.0),
            // Java: USymbolHexagon.getMargin() => Margin(10, 10, 10, 10)
            Self::Hexagon => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            // Java: USymbolPerson.getMargin() => Margin(10, 10, 10, 10)
            Self::Person => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            // Java: USymbolFile.getMargin() => Margin(10, 10, 10, 10)
            Self::File => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            // Java: USymbolCollections.getMargin() => Margin(10, 10, 10, 10)
            Self::Collections => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            // Java: USymbolRectangle.getMargin() => Margin(10, 10, 10, 10)
            Self::Rectangle | Self::Agent | Self::Archimate | Self::ComponentRectangle => {
                SymbolMargin::new(10.0, 10.0, 10.0, 10.0)
            }
            // Java: USymbolProcess.getMargin() => Margin(20, 20, 10, 10)
            Self::Process => SymbolMargin::new(20.0, 20.0, 10.0, 10.0),
            // Java: USymbolLabel.getMargin() => Margin(10, 10, 10, 10)
            Self::Label => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
            // Default for remaining symbols
            _ => SymbolMargin::new(10.0, 10.0, 10.0, 10.0),
        }
    }

    /// Extra height needed due to symbol shape protrusions.
    /// Java: `USymbol.suppHeightBecauseOfShape()`
    pub fn supp_height(&self) -> i32 {
        match self {
            // Java: USymbolDatabase.suppHeightBecauseOfShape() => 15
            Self::Database => 15,
            // Java: USymbolNode.suppHeightBecauseOfShape() => 5
            Self::Node => 5,
            _ => 0,
        }
    }

    /// Extra width needed due to symbol shape protrusions.
    /// Java: `USymbol.suppWidthBecauseOfShape()`
    pub fn supp_width(&self) -> i32 {
        match self {
            // Java: USymbolNode.suppWidthBecauseOfShape() => 60
            Self::Node => 60,
            _ => 0,
        }
    }

    /// Resolve a symbol name to a kind.
    /// Java: `USymbols.fromString()`
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "ACTION" => Some(Self::Action),
            "ACTOR" | "ACTOR_STICKMAN" => Some(Self::ActorStickman),
            "ACTOR_AWESOME" => Some(Self::ActorAwesome),
            "ACTOR_HOLLOW" => Some(Self::ActorHollow),
            "ACTOR_STICKMAN_BUSINESS" => Some(Self::ActorBusiness),
            "AGENT" => Some(Self::Agent),
            "ARCHIMATE" => Some(Self::Archimate),
            "ARTIFACT" => Some(Self::Artifact),
            "BOUNDARY" => Some(Self::Boundary),
            "CARD" => Some(Self::Card),
            "CLOUD" => Some(Self::Cloud),
            "COLLECTIONS" => Some(Self::Collections),
            "COMPONENT" | "COMPONENT2" => Some(Self::Component2),
            "COMPONENT1" => Some(Self::Component1),
            "COMPONENT_RECTANGLE" => Some(Self::ComponentRectangle),
            "CONTROL" => Some(Self::Control),
            "DATABASE" => Some(Self::Database),
            "ENTITY" | "ENTITY_DOMAIN" => Some(Self::EntityDomain),
            "FILE" => Some(Self::File),
            "FOLDER" => Some(Self::Folder),
            "FRAME" => Some(Self::Frame),
            "GROUP" => Some(Self::Group),
            "HEXAGON" => Some(Self::Hexagon),
            "INTERFACE" => Some(Self::Interface),
            "LABEL" => Some(Self::Label),
            "NODE" => Some(Self::Node),
            "PACKAGE" => Some(Self::Package),
            "PERSON" => Some(Self::Person),
            "PROCESS" => Some(Self::Process),
            "QUEUE" => Some(Self::Queue),
            "RECTANGLE" | "RECT" => Some(Self::Rectangle),
            "STACK" => Some(Self::Stack),
            "STORAGE" => Some(Self::Storage),
            "USECASE" => Some(Self::Usecase),
            _ => None,
        }
    }
}

// ── Shape drawing functions ─────────────────────────────────────────
// Each function returns the drawing primitives for a particular UML symbol.
// The primitives (UPath, URectangle, ULine, etc.) represent the shape outline
// and any internal decoration lines.

/// Drawing result for a symbol shape.
/// Contains all the primitives needed to render the symbol.
#[derive(Debug, Clone)]
pub struct SymbolShape {
    /// Primary outline path(s) - drawn with fill+stroke
    pub outlines: Vec<ShapePrimitive>,
    /// Decoration lines/paths drawn on top with no fill (just stroke)
    pub decorations: Vec<ShapePrimitive>,
}

/// A single drawing primitive with optional translation offset.
#[derive(Debug, Clone)]
pub struct ShapePrimitive {
    pub kind: PrimitiveKind,
    pub dx: f64,
    pub dy: f64,
    /// If true, draw with no background fill (transparent)
    pub no_fill: bool,
}

/// Drawing primitive types supported by symbol shapes.
#[derive(Debug, Clone)]
pub enum PrimitiveKind {
    Rect(URectangle),
    Path(UPath),
    Ellipse(UEllipse),
    Line(ULine),
}

impl ShapePrimitive {
    fn rect(r: URectangle, dx: f64, dy: f64) -> Self {
        Self {
            kind: PrimitiveKind::Rect(r),
            dx,
            dy,
            no_fill: false,
        }
    }
    fn path(p: UPath, dx: f64, dy: f64) -> Self {
        Self {
            kind: PrimitiveKind::Path(p),
            dx,
            dy,
            no_fill: false,
        }
    }
    fn ellipse(e: UEllipse, dx: f64, dy: f64) -> Self {
        Self {
            kind: PrimitiveKind::Ellipse(e),
            dx,
            dy,
            no_fill: false,
        }
    }
    fn line(l: ULine, dx: f64, dy: f64) -> Self {
        Self {
            kind: PrimitiveKind::Line(l),
            dx,
            dy,
            no_fill: false,
        }
    }
    fn no_fill(mut self) -> Self {
        self.no_fill = true;
        self
    }
}

impl SymbolShape {
    fn new() -> Self {
        Self {
            outlines: Vec::new(),
            decorations: Vec::new(),
        }
    }
}

// ── Rectangle ────────────────────────────────────────────────────────
// Java: USymbolRectangle - simple rect for agent, archimate, component_rectangle

/// Draw a rectangle symbol.
/// Java: `USymbolRectangle.drawRect()`
pub fn draw_rectangle(
    width: f64,
    height: f64,
    shadow: f64,
    round_corner: f64,
    diagonal_corner: f64,
) -> SymbolShape {
    let mut shape = SymbolShape::new();
    if diagonal_corner > 0.0 {
        let mut rect = URectangle::build(width, height);
        rect.shadow = shadow;
        let mut path = rect.diagonal_corner(diagonal_corner);
        path.shadow = shadow;
        shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));
    } else {
        let mut rect = URectangle::build(width, height).rounded(round_corner);
        rect.shadow = shadow;
        shape.outlines.push(ShapePrimitive::rect(rect, 0.0, 0.0));
    }
    shape
}

// ── Component2 ───────────────────────────────────────────────────────
// Java: USymbolComponent2 - UML2 component notation

/// Draw a UML2 component symbol: rect + small component icon in upper right.
/// Java: `USymbolComponent2.drawComponent2()`
pub fn draw_component2(width: f64, height: f64, shadow: f64, round_corner: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    // Main rectangle
    let mut form = URectangle::build(width, height).rounded(round_corner);
    form.shadow = shadow;
    shape.outlines.push(ShapePrimitive::rect(form, 0.0, 0.0));

    // Small rectangle (15x10) at top-right
    let small = URectangle::build(15.0, 10.0);
    shape
        .decorations
        .push(ShapePrimitive::rect(small, width - 20.0, 5.0));

    // Two tiny rectangles (4x2) as tabs
    let tiny1 = URectangle::build(4.0, 2.0);
    let tiny2 = URectangle::build(4.0, 2.0);
    shape
        .decorations
        .push(ShapePrimitive::rect(tiny1, width - 22.0, 7.0));
    shape
        .decorations
        .push(ShapePrimitive::rect(tiny2, width - 22.0, 11.0));

    shape
}

// ── Component1 ───────────────────────────────────────────────────────
// Java: USymbolComponent1 - UML1 component notation

/// Draw a UML1 component symbol: rect + two small rectangles on the left edge.
/// Java: `USymbolComponent1.drawComponent1()`
pub fn draw_component1(width: f64, height: f64, shadow: f64, round_corner: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    // Main rectangle
    let mut form = URectangle::build(width, height).rounded(round_corner);
    form.shadow = shadow;
    shape.outlines.push(ShapePrimitive::rect(form, 0.0, 0.0));

    // UML 1 notation: two small rectangles (10x5) on the left edge
    let small1 = URectangle::build(10.0, 5.0);
    let small2 = URectangle::build(10.0, 5.0);
    shape
        .decorations
        .push(ShapePrimitive::rect(small1, -5.0, 5.0));
    shape
        .decorations
        .push(ShapePrimitive::rect(small2, -5.0, height - 10.0));

    shape
}

// ── Database ─────────────────────────────────────────────────────────
// Java: USymbolDatabase - cylinder shape

/// Draw a database (cylinder) symbol.
/// Java: `USymbolDatabase.drawDatabase()` + `getClosingPath()`
pub fn draw_database(width: f64, height: f64, shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    // Main cylinder body path
    let mut body = UPath::new();
    body.shadow = shadow;
    body.move_to(0.0, 10.0);
    body.cubic_to(0.0, 0.0, width / 2.0, 0.0, width / 2.0, 0.0);
    body.cubic_to(width / 2.0, 0.0, width, 0.0, width, 10.0);
    body.line_to(width, height - 10.0);
    body.cubic_to(width, height, width / 2.0, height, width / 2.0, height);
    body.cubic_to(width / 2.0, height, 0.0, height, 0.0, height - 10.0);
    body.line_to(0.0, 10.0);
    shape.outlines.push(ShapePrimitive::path(body, 0.0, 0.0));

    // Closing arc at top (the visible inner ellipse bottom)
    let mut closing = UPath::new();
    closing.move_to(0.0, 10.0);
    closing.cubic_to(0.0, 20.0, width / 2.0, 20.0, width / 2.0, 20.0);
    closing.cubic_to(width / 2.0, 20.0, width, 20.0, width, 10.0);
    shape
        .decorations
        .push(ShapePrimitive::path(closing, 0.0, 0.0).no_fill());

    shape
}

// ── Folder ───────────────────────────────────────────────────────────
// Java: USymbolFolder - folder with tab

/// Folder shape constants matching Java USymbolFolder.
const FOLDER_MARGIN_TITLE_X1: f64 = 3.0;
const FOLDER_MARGIN_TITLE_X2: f64 = 3.0;
const FOLDER_MARGIN_TITLE_X3: f64 = 7.0;
const FOLDER_MARGIN_TITLE_Y1: f64 = 3.0;
const FOLDER_MARGIN_TITLE_Y2: f64 = 3.0;

/// Compute the folder tab width given total width and title dimensions.
/// Java: `USymbolFolder.getWTitle()`
pub fn folder_wtitle(width: f64, dim_title: XDimension2D) -> f64 {
    if dim_title.width == 0.0 {
        f64::max(30.0, width / 4.0)
    } else {
        dim_title.width + FOLDER_MARGIN_TITLE_X1 + FOLDER_MARGIN_TITLE_X2
    }
}

/// Compute the folder tab height given title dimensions.
/// Java: `USymbolFolder.getHTitle()`
pub fn folder_htitle(dim_title: XDimension2D) -> f64 {
    if dim_title.width == 0.0 {
        10.0
    } else {
        dim_title.height + FOLDER_MARGIN_TITLE_Y1 + FOLDER_MARGIN_TITLE_Y2
    }
}

/// Draw a folder symbol (polygon or rounded path + tab line).
/// Java: `USymbolFolder.drawFolder()`
pub fn draw_folder(
    width: f64,
    height: f64,
    dim_title: XDimension2D,
    shadow: f64,
    round_corner: f64,
) -> SymbolShape {
    let mut shape = SymbolShape::new();
    let wtitle = folder_wtitle(width, dim_title);
    let htitle = folder_htitle(dim_title);

    if round_corner == 0.0 {
        // Sharp-cornered polygon
        let mut path = UPath::new();
        path.shadow = shadow;
        path.move_to(0.0, 0.0);
        path.line_to(wtitle, 0.0);
        path.line_to(wtitle + FOLDER_MARGIN_TITLE_X3, htitle);
        path.line_to(width, htitle);
        path.line_to(width, height);
        path.line_to(0.0, height);
        path.line_to(0.0, 0.0);
        shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));
    } else {
        // Rounded path
        let r = round_corner / 2.0;
        let mut path = UPath::new();
        path.shadow = shadow;
        path.move_to(r, 0.0);
        path.line_to(wtitle - r, 0.0);
        path.arc_to(r * 1.5, r * 1.5, 0.0, 0.0, 1.0, wtitle, r);
        path.line_to(wtitle + FOLDER_MARGIN_TITLE_X3, htitle);
        path.line_to(width - r, htitle);
        path.arc_to(r, r, 0.0, 0.0, 1.0, width, htitle + r);
        path.line_to(width, height - r);
        path.arc_to(r, r, 0.0, 0.0, 1.0, width - r, height);
        path.line_to(r, height);
        path.arc_to(r, r, 0.0, 0.0, 1.0, 0.0, height - r);
        path.line_to(0.0, r);
        path.arc_to(r, r, 0.0, 0.0, 1.0, r, 0.0);
        path.close();
        shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));
    }

    // Horizontal line separating the tab from the body
    let line = ULine::hline(wtitle + FOLDER_MARGIN_TITLE_X3);
    shape
        .decorations
        .push(ShapePrimitive::line(line, 0.0, htitle));

    shape
}

// ── Cloud ────────────────────────────────────────────────────────────
// Java: USymbolCloud - cloud outline via randomized cubic curves

/// Draw a cloud symbol using randomized bubbles.
/// Java: `USymbolCloud.drawCloud()` / `getSpecificFrontierForCloudNew()`
///
/// Uses a deterministic pseudo-random sequence seeded from width+height
/// to generate the bubbly cloud outline, matching Java's `Random`.
pub fn draw_cloud(width: f64, height: f64, shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    let mut path = cloud_frontier(width, height);
    path.shadow = shadow;
    shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));

    shape
}

/// Generate the cloud frontier path.
/// Java: `getSpecificFrontierForCloudNew()` -- the NEW=true branch.
///
/// This is a simplified but deterministic cloud outline that produces
/// bubbled edges. The exact randomization matches Java's `java.util.Random`
/// with seed `(long)width + 7919L * (long)height`.
fn cloud_frontier(width: f64, height: f64) -> UPath {
    // Use a simple LCG matching Java's Random internals
    let seed = (width as i64) + 7919i64 * (height as i64);
    let mut rng = JavaRandom::new(seed);

    let mut bubble_size: f64 = 11.0;
    if f64::max(width, height) / bubble_size > 16.0 {
        bubble_size = f64::max(width, height) / 16.0;
    }

    let margin1 = 8.0;
    let point_a = (margin1, margin1);
    let point_b = (width - margin1, margin1);
    let point_c = (width - margin1, height - margin1);
    let point_d = (margin1, height - margin1);

    let mut points: Vec<(f64, f64)> = Vec::new();

    if width > 100.0 && height > 100.0 {
        cloud_complex(
            &mut rng,
            &mut points,
            bubble_size,
            point_a,
            point_b,
            point_c,
            point_d,
        );
    } else {
        cloud_simple(
            &mut rng,
            &mut points,
            bubble_size,
            point_a,
            point_b,
            point_c,
            point_d,
        );
    }

    // Close the loop
    if let Some(&first) = points.first() {
        points.push(first);
    }

    let mut path = UPath::new();
    if points.is_empty() {
        return path;
    }
    path.move_to(points[0].0, points[0].1);
    for i in 0..points.len() - 1 {
        cloud_add_curve(&mut rng, &mut path, points[i], points[i + 1]);
    }
    path
}

fn cloud_complex(
    rng: &mut JavaRandom,
    points: &mut Vec<(f64, f64)>,
    bubble_size: f64,
    a: (f64, f64),
    b: (f64, f64),
    c: (f64, f64),
    d: (f64, f64),
) {
    let margin2 = 7.0;
    cloud_special_line(
        bubble_size,
        rng,
        points,
        mv_x(a, margin2),
        mv_x(b, -margin2),
    );
    points.push(mv_y(b, margin2));
    cloud_special_line(
        bubble_size,
        rng,
        points,
        mv_y(b, margin2),
        mv_y(c, -margin2),
    );
    points.push(mv_x(c, -margin2));
    cloud_special_line(
        bubble_size,
        rng,
        points,
        mv_x(c, -margin2),
        mv_x(d, margin2),
    );
    points.push(mv_y(d, -margin2));
    cloud_special_line(
        bubble_size,
        rng,
        points,
        mv_y(d, -margin2),
        mv_y(a, margin2),
    );
    points.push(mv_x(a, margin2));
}

fn cloud_simple(
    rng: &mut JavaRandom,
    points: &mut Vec<(f64, f64)>,
    bubble_size: f64,
    a: (f64, f64),
    b: (f64, f64),
    c: (f64, f64),
    d: (f64, f64),
) {
    cloud_special_line(bubble_size, rng, points, a, b);
    cloud_special_line(bubble_size, rng, points, b, c);
    cloud_special_line(bubble_size, rng, points, c, d);
    cloud_special_line(bubble_size, rng, points, d, a);
}

fn mv_x(pt: (f64, f64), dx: f64) -> (f64, f64) {
    (pt.0 + dx, pt.1)
}
fn mv_y(pt: (f64, f64), dy: f64) -> (f64, f64) {
    (pt.0, pt.1 + dy)
}

fn cloud_special_line(
    bubble_size: f64,
    rng: &mut JavaRandom,
    points: &mut Vec<(f64, f64)>,
    p1: (f64, f64),
    p2: (f64, f64),
) {
    let length = coord_length(p1, p2);
    let (cos, sin) = coord_angle(p1, p2);
    let r = rng_range(rng, 1.0, 1.0 + f64::min(12.0, bubble_size * 0.8));
    let mid = coord_true(p1, cos, sin, length / 2.0, -r);

    cloud_bubble_line(rng, points, p1, mid, bubble_size);
    cloud_bubble_line(rng, points, mid, p2, bubble_size);
}

fn cloud_bubble_line(
    rng: &mut JavaRandom,
    points: &mut Vec<(f64, f64)>,
    p1: (f64, f64),
    p2: (f64, f64),
    mut bubble_size: f64,
) {
    let length = coord_length(p1, p2);
    let (cos, sin) = coord_angle(p1, p2);
    let mut nb = (length / bubble_size) as i32;
    if nb == 0 {
        bubble_size = length / 2.0;
        nb = (length / bubble_size) as i32;
    }
    for i in 0..nb {
        let pt = coord_true(p1, cos, sin, i as f64 * length / nb as f64, 0.0);
        let rx = pt.0 + bubble_size * 0.2 * rng.next_double();
        let ry = pt.1 + bubble_size * 0.2 * rng.next_double();
        points.push((rx, ry));
    }
}

fn cloud_add_curve(rng: &mut JavaRandom, path: &mut UPath, p1: (f64, f64), p2: (f64, f64)) {
    let length = coord_length(p1, p2);
    let (cos, sin) = coord_angle(p1, p2);
    let coef = rng_range(rng, 0.25, 0.35);
    let mid = coord_true(
        p1,
        cos,
        sin,
        length * coef,
        -length * rng_range(rng, 0.4, 0.55),
    );
    let mid2 = coord_true(
        p1,
        cos,
        sin,
        length * (1.0 - coef),
        -length * rng_range(rng, 0.4, 0.55),
    );
    path.cubic_to(mid.0, mid.1, mid2.0, mid2.1, p2.0, p2.1);
}

// Coordinate change helpers (matching Java's CoordinateChange)

fn coord_length(p1: (f64, f64), p2: (f64, f64)) -> f64 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    (dx * dx + dy * dy).sqrt()
}

fn coord_angle(p1: (f64, f64), p2: (f64, f64)) -> (f64, f64) {
    let length = coord_length(p1, p2);
    if length == 0.0 {
        return (1.0, 0.0);
    }
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    (dx / length, dy / length)
}

fn coord_true(origin: (f64, f64), cos: f64, sin: f64, along: f64, perp: f64) -> (f64, f64) {
    let x = origin.0 + along * cos - perp * sin;
    let y = origin.1 + along * sin + perp * cos;
    (x, y)
}

fn rng_range(rng: &mut JavaRandom, a: f64, b: f64) -> f64 {
    rng.next_double() * (b - a) + a
}

// ── Node ─────────────────────────────────────────────────────────────
// Java: USymbolNode - 3D box

/// Draw a node (3D box) symbol.
/// Java: `USymbolNode.drawNode()`
pub fn draw_node(width: f64, height: f64, shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    // Main polygon (front face + top ridge)
    let mut body = UPath::new();
    body.shadow = shadow;
    body.move_to(0.0, 10.0);
    body.line_to(10.0, 0.0);
    body.line_to(width, 0.0);
    body.line_to(width, height - 10.0);
    body.line_to(width - 10.0, height);
    body.line_to(0.0, height);
    body.line_to(0.0, 10.0);
    body.close();
    shape.outlines.push(ShapePrimitive::path(body, 0.0, 0.0));

    // Diagonal line: upper-right corner fold
    shape.decorations.push(ShapePrimitive::line(
        ULine::new(10.0, -10.0),
        width - 10.0,
        10.0,
    ));
    // Horizontal line: bottom of top face
    shape
        .decorations
        .push(ShapePrimitive::line(ULine::hline(width - 10.0), 0.0, 10.0));
    // Vertical line: right side inner edge
    shape.decorations.push(ShapePrimitive::line(
        ULine::vline(height - 10.0),
        width - 10.0,
        10.0,
    ));

    shape
}

// ── Frame ────────────────────────────────────────────────────────────
// Java: USymbolFrame - rect with title tab in upper-left corner

/// Compute the Y position of the frame title separator.
/// Java: `USymbolFrame.getYpos()`
pub fn frame_ypos(dim_title: XDimension2D) -> f64 {
    if dim_title.width == 0.0 {
        12.0
    } else {
        dim_title.height + 3.0
    }
}

/// Draw a frame symbol (rectangle + title-tab corner path).
/// Java: `USymbolFrame.drawFrame()`
pub fn draw_frame(
    width: f64,
    height: f64,
    dim_title: XDimension2D,
    shadow: f64,
    round_corner: f64,
) -> SymbolShape {
    let mut shape = SymbolShape::new();

    // Main rectangle
    let mut rect = URectangle::build(width, height).rounded(round_corner);
    rect.shadow = shadow;
    shape.outlines.push(ShapePrimitive::rect(rect, 0.0, 0.0));

    // Title tab corner
    let text_width;
    let cornersize;
    if dim_title.width == 0.0 {
        text_width = width / 3.0;
        cornersize = 7.0;
    } else {
        text_width = dim_title.width + 10.0;
        cornersize = 10.0;
    }
    let text_height = frame_ypos(dim_title);

    let mut tab = UPath::new();
    tab.move_to(text_width, 0.0);
    tab.line_to(text_width, text_height - cornersize);
    tab.line_to(text_width - cornersize, text_height);
    tab.line_to(0.0, text_height);
    shape
        .decorations
        .push(ShapePrimitive::path(tab, 0.0, 0.0).no_fill());

    shape
}

// ── Usecase ──────────────────────────────────────────────────────────
// Java: USymbolUsecase - ellipse

/// Draw a usecase (ellipse) symbol.
/// Java: `USymbolUsecase` renders via `TextBlockInEllipse`
pub fn draw_usecase(width: f64, height: f64, shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    let mut ell = UEllipse::build(width, height);
    ell.shadow = shadow;
    shape.outlines.push(ShapePrimitive::ellipse(ell, 0.0, 0.0));

    shape
}

// ── Card ─────────────────────────────────────────────────────────────
// Java: USymbolCard - rounded rectangle with optional divider line

/// Draw a card symbol.
/// Java: `USymbolCard.drawCard()`
pub fn draw_card(width: f64, height: f64, shadow: f64, top: f64, round_corner: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    let mut rect = URectangle::build(width, height).rounded(round_corner);
    rect.shadow = shadow;
    shape.outlines.push(ShapePrimitive::rect(rect, 0.0, 0.0));

    if top != 0.0 {
        shape
            .decorations
            .push(ShapePrimitive::line(ULine::hline(width), 0.0, top));
    }

    shape
}

// ── Storage ──────────────────────────────────────────────────────────
// Java: USymbolStorage - heavily rounded rectangle

/// Draw a storage symbol.
/// Java: `USymbolStorage.drawStorage()` -- URectangle with rounded(70)
pub fn draw_storage(width: f64, height: f64, shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    let mut rect = URectangle::build(width, height).rounded(70.0);
    rect.shadow = shadow;
    shape.outlines.push(ShapePrimitive::rect(rect, 0.0, 0.0));

    shape
}

// ── Queue ────────────────────────────────────────────────────────────
// Java: USymbolQueue - pill shape with visible right-side closing arc

/// Draw a queue symbol.
/// Java: `USymbolQueue.drawQueue()`
pub fn draw_queue(width: f64, height: f64, shadow: f64) -> SymbolShape {
    let dx: f64 = 5.0;
    let mut shape = SymbolShape::new();

    // Main body path
    let mut body = UPath::new();
    body.shadow = shadow;
    body.move_to(dx, 0.0);
    body.line_to(width - dx, 0.0);
    body.cubic_to(width, 0.0, width, height / 2.0, width, height / 2.0);
    body.cubic_to(width, height / 2.0, width, height, width - dx, height);
    body.line_to(dx, height);
    body.cubic_to(0.0, height, 0.0, height / 2.0, 0.0, height / 2.0);
    body.cubic_to(0.0, height / 2.0, 0.0, 0.0, dx, 0.0);
    shape.outlines.push(ShapePrimitive::path(body, 0.0, 0.0));

    // Closing arc on the right side
    let mut closing = UPath::new();
    closing.move_to(width - dx, 0.0);
    closing.cubic_to(
        width - dx * 2.0,
        0.0,
        width - dx * 2.0,
        height / 2.0,
        width - dx * 2.0,
        height / 2.0,
    );
    closing.cubic_to(
        width - dx * 2.0,
        height,
        width - dx,
        height,
        width - dx,
        height,
    );
    shape
        .decorations
        .push(ShapePrimitive::path(closing, 0.0, 0.0).no_fill());

    shape
}

// ── Stack ────────────────────────────────────────────────────────────
// Java: USymbolStack - rect with bracket decorations on left/right

/// Draw a stack symbol.
/// Java: `USymbolStack.drawQueue()` (yes, method is named drawQueue in Java)
pub fn draw_stack(width: f64, height: f64, shadow: f64, round_corner: f64) -> SymbolShape {
    let border = 15.0;
    let mut shape = SymbolShape::new();

    // Inner rectangle (drawn with no-fill color in Java)
    let mut rect = URectangle::build(width - 2.0 * border, height).rounded(round_corner);
    rect.shadow = 0.0; // inner rect has no shadow
    shape.outlines.push(ShapePrimitive::rect(rect, border, 0.0));

    // Bracket path (the L/R decorative brackets)
    let mut bracket = UPath::new();
    bracket.shadow = shadow;
    if round_corner == 0.0 {
        bracket.move_to(0.0, 0.0);
        bracket.line_to(border, 0.0);
        bracket.line_to(border, height);
        bracket.line_to(width - border, height);
        bracket.line_to(width - border, 0.0);
        bracket.line_to(width, 0.0);
    } else {
        let r = round_corner / 2.0;
        bracket.move_to(0.0, 0.0);
        bracket.line_to(border - r, 0.0);
        bracket.arc_to(r, r, 0.0, 0.0, 1.0, border, r);
        bracket.line_to(border, height - r);
        bracket.arc_to(r, r, 0.0, 0.0, 0.0, border + r, height);
        bracket.line_to(width - border - r, height);
        bracket.arc_to(r, r, 0.0, 0.0, 0.0, width - border, height - r);
        bracket.line_to(width - border, r);
        bracket.arc_to(r, r, 0.0, 0.0, 1.0, width - border + r, 0.0);
        bracket.line_to(width, 0.0);
    }
    shape
        .decorations
        .push(ShapePrimitive::path(bracket, 0.0, 0.0).no_fill());

    shape
}

// ── Hexagon ──────────────────────────────────────────────────────────
// Java: USymbolHexagon

/// Draw a hexagon symbol.
/// Java: `USymbolHexagon.drawRect()`
pub fn draw_hexagon(width: f64, height: f64, shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();
    let dx = width / 8.0;

    let mut path = UPath::new();
    path.shadow = shadow;
    path.move_to(0.0, height / 2.0);
    path.line_to(dx, 0.0);
    path.line_to(width - dx, 0.0);
    path.line_to(width, height / 2.0);
    path.line_to(width - dx, height);
    path.line_to(dx, height);
    path.line_to(0.0, height / 2.0);
    path.close();
    shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));

    shape
}

// ── Person ───────────────────────────────────────────────────────────
// Java: USymbolPerson - head (ellipse) + body (rounded rect)

/// Compute the head size for a person symbol given the body dimensions.
/// Java: `USymbolPerson.headSize()`
pub fn person_head_size(body_dim: XDimension2D) -> f64 {
    let surface = body_dim.width * body_dim.height;
    surface.sqrt() * 0.42
}

/// Draw a person symbol (head circle + rounded rect body).
/// Java: `USymbolPerson.drawHeadAndBody()`
pub fn draw_person(body_dim: XDimension2D, shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();
    let head_size = person_head_size(body_dim);

    // Head (circle)
    let mut head = UEllipse::build(head_size, head_size);
    head.shadow = shadow;
    let posx = (body_dim.width - head_size) / 2.0;
    shape
        .outlines
        .push(ShapePrimitive::ellipse(head, posx, 0.0));

    // Body (rounded rectangle)
    let mut body = URectangle::build(body_dim.width, body_dim.height).rounded(head_size);
    body.shadow = shadow;
    shape
        .outlines
        .push(ShapePrimitive::rect(body, 0.0, head_size));

    shape
}

// ── File ─────────────────────────────────────────────────────────────
// Java: USymbolFile - page with dog-ear corner

/// Draw a file symbol (page with folded corner).
/// Java: `USymbolFile.drawFile()`
pub fn draw_file(width: f64, height: f64, shadow: f64, round_corner: f64) -> SymbolShape {
    let cornersize = 10.0;
    let mut shape = SymbolShape::new();

    if round_corner == 0.0 {
        // Sharp polygon
        let mut path = UPath::new();
        path.shadow = shadow;
        path.move_to(0.0, 0.0);
        path.line_to(0.0, height);
        path.line_to(width, height);
        path.line_to(width, cornersize);
        path.line_to(width - cornersize, 0.0);
        path.line_to(0.0, 0.0);
        shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));
    } else {
        let r = round_corner / 2.0;
        let mut path = UPath::new();
        path.shadow = shadow;
        path.move_to(0.0, r);
        path.line_to(0.0, height - r);
        path.arc_to(r, r, 0.0, 0.0, 0.0, r, height);
        path.line_to(width - r, height);
        path.arc_to(r, r, 0.0, 0.0, 0.0, width, height - r);
        path.line_to(width, cornersize);
        path.line_to(width - cornersize, 0.0);
        path.line_to(r, 0.0);
        path.arc_to(r, r, 0.0, 0.0, 0.0, 0.0, r);
        shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));
    }

    // Dog-ear fold line
    let mut fold = UPath::new();
    fold.move_to(width - cornersize, 0.0);
    if round_corner == 0.0 {
        fold.line_to(width - cornersize, cornersize);
    } else {
        let r = round_corner / 2.0;
        fold.line_to(width - cornersize, cornersize - r);
        fold.arc_to(r, r, 0.0, 0.0, 0.0, width - cornersize + r, cornersize);
    }
    fold.line_to(width, cornersize);
    shape.decorations.push(ShapePrimitive::path(fold, 0.0, 0.0));

    shape
}

// ── Artifact ─────────────────────────────────────────────────────────
// Java: USymbolArtifact - rect with small file icon in upper-right

/// Draw an artifact symbol (rectangle + small page icon).
/// Java: `USymbolArtifact.drawArtifact()`
pub fn draw_artifact(width: f64, height: f64, shadow: f64, round_corner: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    // Main rectangle
    let mut form = URectangle::build(width, height).rounded(round_corner);
    form.shadow = shadow;
    shape.outlines.push(ShapePrimitive::rect(form, 0.0, 0.0));

    // Small file icon (polygon)
    let height_sym = 14.0;
    let width_sym = 12.0;
    let corner = 6.0;
    let x_sym = width - width_sym - 5.0;
    let y_sym = 5.0;

    let mut icon = UPath::new();
    icon.move_to(0.0, 0.0);
    icon.line_to(0.0, height_sym);
    icon.line_to(width_sym, height_sym);
    icon.line_to(width_sym, corner);
    icon.line_to(width_sym - corner, 0.0);
    icon.line_to(0.0, 0.0);
    shape
        .decorations
        .push(ShapePrimitive::path(icon, x_sym, y_sym));

    // Fold lines
    shape.decorations.push(ShapePrimitive::line(
        ULine::vline(corner),
        x_sym + width_sym - corner,
        y_sym,
    ));
    shape.decorations.push(ShapePrimitive::line(
        ULine::hline(-corner),
        x_sym + width_sym,
        y_sym + corner,
    ));

    shape
}

// ── Collections ──────────────────────────────────────────────────────
// Java: USymbolCollections - two stacked rectangles

/// Draw a collections symbol (two offset rectangles).
/// Java: `USymbolCollections.drawCollections()`
pub fn draw_collections(width: f64, height: f64, shadow: f64, round_corner: f64) -> SymbolShape {
    let delta = 4.0;
    let mut shape = SymbolShape::new();

    // Back rectangle (offset by delta)
    let mut back = URectangle::build(width - delta, height - delta).rounded(round_corner);
    back.shadow = shadow;
    shape
        .outlines
        .push(ShapePrimitive::rect(back, delta, delta));

    // Front rectangle (no shadow)
    let front = URectangle::build(width - delta, height - delta).rounded(round_corner);
    shape.outlines.push(ShapePrimitive::rect(front, 0.0, 0.0));

    shape
}

// ── Action ───────────────────────────────────────────────────────────
// Java: USymbolAction - pentagon (arrow-like shape)

/// Draw an action symbol (pentagon pointing right).
/// Java: `USymbolAction.drawAction()`
pub fn draw_action(width: f64, height: f64, _shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    let mut path = UPath::new();
    path.move_to(0.0, 0.0);
    path.line_to(width - 10.0, 0.0);
    path.line_to(width, height / 2.0);
    path.line_to(width - 10.0, height);
    path.line_to(0.0, height);
    path.close();
    shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));

    shape
}

// ── Process ──────────────────────────────────────────────────────────
// Java: USymbolProcess - hexagon with left and right arrow points

/// Draw a process symbol (double-arrow hexagon).
/// Java: `USymbolProcess.drawProcess()`
pub fn draw_process(width: f64, height: f64, _shadow: f64) -> SymbolShape {
    let mut shape = SymbolShape::new();

    let mut path = UPath::new();
    path.move_to(0.0, 0.0);
    path.line_to(width - 10.0, 0.0);
    path.line_to(width, height / 2.0);
    path.line_to(width - 10.0, height);
    path.line_to(0.0, height);
    path.line_to(10.0, height / 2.0);
    path.close();
    shape.outlines.push(ShapePrimitive::path(path, 0.0, 0.0));

    shape
}

// ── Label ────────────────────────────────────────────────────────────
// Java: USymbolLabel - no outline (just text with margin)

/// Draw a label symbol (no outline shape).
/// Java: `USymbolLabel` does not draw any border
pub fn draw_label() -> SymbolShape {
    SymbolShape::new()
}

// ── JavaRandom ───────────────────────────────────────────────────────
// Minimal port of Java's `java.util.Random` LCG for deterministic cloud shapes.

struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    fn new(seed: i64) -> Self {
        let s = (seed ^ 0x5DEECE66Di64) & ((1i64 << 48) - 1);
        Self { seed: s }
    }

    fn next(&mut self, bits: i32) -> i32 {
        self.seed =
            (self.seed.wrapping_mul(0x5DEECE66Di64).wrapping_add(0xBi64)) & ((1i64 << 48) - 1);
        (self.seed >> (48 - bits)) as i32
    }

    fn next_double(&mut self) -> f64 {
        let hi = self.next(26) as i64;
        let lo = self.next(27) as i64;
        (hi * (1i64 << 27) + lo) as f64 / ((1i64 << 53) as f64)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── from_name ──

    #[test]
    fn symbol_from_name() {
        assert_eq!(
            USymbolKind::from_name("database"),
            Some(USymbolKind::Database)
        );
        assert_eq!(USymbolKind::from_name("CLOUD"), Some(USymbolKind::Cloud));
        assert_eq!(
            USymbolKind::from_name("component"),
            Some(USymbolKind::Component2)
        );
        assert_eq!(
            USymbolKind::from_name("ACTOR"),
            Some(USymbolKind::ActorStickman)
        );
        assert!(USymbolKind::from_name("nonexistent").is_none());
    }

    #[test]
    fn symbol_from_name_aliases() {
        assert_eq!(USymbolKind::from_name("RECT"), Some(USymbolKind::Rectangle));
        assert_eq!(
            USymbolKind::from_name("entity"),
            Some(USymbolKind::EntityDomain)
        );
        assert_eq!(
            USymbolKind::from_name("entity_domain"),
            Some(USymbolKind::EntityDomain)
        );
        assert_eq!(
            USymbolKind::from_name("component2"),
            Some(USymbolKind::Component2)
        );
        assert_eq!(
            USymbolKind::from_name("component1"),
            Some(USymbolKind::Component1)
        );
    }

    // ── margins (verified against Java source) ──

    #[test]
    fn margin_database() {
        let m = USymbolKind::Database.margin();
        assert_eq!(m.x1, 10.0);
        assert_eq!(m.x2, 10.0);
        assert_eq!(m.y1, 24.0);
        assert_eq!(m.y2, 5.0);
    }

    #[test]
    fn margin_component2() {
        let m = USymbolKind::Component2.margin();
        assert_eq!(m.x1, 15.0); // 10+5
        assert_eq!(m.x2, 25.0); // 20+5
        assert_eq!(m.y1, 20.0); // 15+5
        assert_eq!(m.y2, 10.0); // 5+5
    }

    #[test]
    fn margin_cloud() {
        let m = USymbolKind::Cloud.margin();
        assert_eq!(m.x1, 15.0);
        assert_eq!(m.x2, 15.0);
        assert_eq!(m.y1, 15.0);
        assert_eq!(m.y2, 15.0);
    }

    #[test]
    fn margin_rectangle() {
        let m = USymbolKind::Rectangle.margin();
        assert_eq!(m.x1, 10.0);
        assert_eq!(m.x2, 10.0);
        assert_eq!(m.y1, 10.0);
        assert_eq!(m.y2, 10.0);
    }

    #[test]
    fn margin_node() {
        let m = USymbolKind::Node.margin();
        assert_eq!(m.x1, 15.0); // 10+5
        assert_eq!(m.x2, 25.0); // 20+5
        assert_eq!(m.y1, 20.0); // 15+5
        assert_eq!(m.y2, 10.0); // 5+5
    }

    #[test]
    fn margin_frame() {
        let m = USymbolKind::Frame.margin();
        assert_eq!(m.x1, 15.0);
        assert_eq!(m.x2, 25.0);
        assert_eq!(m.y1, 20.0);
        assert_eq!(m.y2, 10.0);
    }

    #[test]
    fn margin_folder() {
        let m = USymbolKind::Folder.margin();
        assert_eq!(m.x1, 10.0);
        assert_eq!(m.x2, 20.0);
        assert_eq!(m.y1, 13.0);
        assert_eq!(m.y2, 10.0);
    }

    #[test]
    fn margin_card() {
        let m = USymbolKind::Card.margin();
        assert_eq!(m.x1, 10.0);
        assert_eq!(m.x2, 10.0);
        assert_eq!(m.y1, 3.0);
        assert_eq!(m.y2, 3.0);
    }

    #[test]
    fn margin_queue() {
        let m = USymbolKind::Queue.margin();
        assert_eq!(m.x1, 5.0);
        assert_eq!(m.x2, 15.0);
        assert_eq!(m.y1, 5.0);
        assert_eq!(m.y2, 5.0);
    }

    #[test]
    fn margin_stack() {
        let m = USymbolKind::Stack.margin();
        assert_eq!(m.x1, 25.0);
        assert_eq!(m.x2, 25.0);
        assert_eq!(m.y1, 10.0);
        assert_eq!(m.y2, 10.0);
    }

    #[test]
    fn margin_action() {
        let m = USymbolKind::Action.margin();
        assert_eq!(m.x1, 10.0);
        assert_eq!(m.x2, 20.0);
        assert_eq!(m.y1, 10.0);
        assert_eq!(m.y2, 10.0);
    }

    #[test]
    fn margin_process() {
        let m = USymbolKind::Process.margin();
        assert_eq!(m.x1, 20.0);
        assert_eq!(m.x2, 20.0);
    }

    #[test]
    fn margin_artifact() {
        let m = USymbolKind::Artifact.margin();
        assert_eq!(m.x1, 10.0);
        assert_eq!(m.x2, 20.0); // 10+10
        assert_eq!(m.y1, 13.0); // 10+3
        assert_eq!(m.y2, 10.0);
    }

    #[test]
    fn margin_agent_archimate() {
        // Agent and Archimate use USymbolRectangle => Margin(10,10,10,10)
        assert_eq!(USymbolKind::Agent.margin().x1, 10.0);
        assert_eq!(USymbolKind::Archimate.margin().x1, 10.0);
        assert_eq!(USymbolKind::ComponentRectangle.margin().x1, 10.0);
    }

    // ── supp_height / supp_width ──

    #[test]
    fn supp_database() {
        assert_eq!(USymbolKind::Database.supp_height(), 15);
        assert_eq!(USymbolKind::Database.supp_width(), 0);
    }

    #[test]
    fn supp_node() {
        assert_eq!(USymbolKind::Node.supp_height(), 5);
        assert_eq!(USymbolKind::Node.supp_width(), 60);
    }

    #[test]
    fn supp_default() {
        assert_eq!(USymbolKind::Rectangle.supp_height(), 0);
        assert_eq!(USymbolKind::Rectangle.supp_width(), 0);
    }

    // ── margin arithmetic ──

    #[test]
    fn margin_add_dimension() {
        let m = SymbolMargin::new(10.0, 20.0, 5.0, 15.0);
        let dim = m.add_dimension(XDimension2D::new(100.0, 50.0));
        assert_eq!(dim.width, 130.0);
        assert_eq!(dim.height, 70.0);
    }

    #[test]
    fn margin_width_height() {
        let m = SymbolMargin::new(10.0, 20.0, 5.0, 15.0);
        assert_eq!(m.width(), 30.0);
        assert_eq!(m.height(), 20.0);
    }

    // ── draw_rectangle ──

    #[test]
    fn rectangle_shape() {
        let s = draw_rectangle(100.0, 60.0, 0.0, 0.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert!(s.decorations.is_empty());
        match &s.outlines[0].kind {
            PrimitiveKind::Rect(r) => {
                assert_eq!(r.width, 100.0);
                assert_eq!(r.height, 60.0);
            }
            _ => panic!("expected Rect"),
        }
    }

    #[test]
    fn rectangle_with_diagonal() {
        let s = draw_rectangle(100.0, 60.0, 0.0, 0.0, 10.0);
        assert_eq!(s.outlines.len(), 1);
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => {
                assert!(!p.segments.is_empty());
            }
            _ => panic!("expected Path for diagonal corner"),
        }
    }

    #[test]
    fn rectangle_rounded() {
        let s = draw_rectangle(100.0, 60.0, 3.0, 8.0, 0.0);
        match &s.outlines[0].kind {
            PrimitiveKind::Rect(r) => {
                assert_eq!(r.rx, 8.0);
                assert_eq!(r.ry, 8.0);
                assert_eq!(r.shadow, 3.0);
            }
            _ => panic!("expected Rect"),
        }
    }

    // ── draw_component2 ──

    #[test]
    fn component2_shape() {
        let s = draw_component2(120.0, 80.0, 0.0, 0.0);
        assert_eq!(s.outlines.len(), 1); // main rect
        assert_eq!(s.decorations.len(), 3); // small + 2 tiny
        match &s.decorations[0].kind {
            PrimitiveKind::Rect(r) => {
                assert_eq!(r.width, 15.0);
                assert_eq!(r.height, 10.0);
            }
            _ => panic!("expected Rect for small icon"),
        }
        // Check positions of tiny rects
        assert_eq!(s.decorations[1].dx, 120.0 - 22.0);
        assert_eq!(s.decorations[1].dy, 7.0);
        assert_eq!(s.decorations[2].dy, 11.0);
    }

    // ── draw_component1 ──

    #[test]
    fn component1_shape() {
        let s = draw_component1(100.0, 80.0, 0.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 2); // two small rects
        assert_eq!(s.decorations[0].dx, -5.0);
        assert_eq!(s.decorations[0].dy, 5.0);
        assert_eq!(s.decorations[1].dx, -5.0);
        assert_eq!(s.decorations[1].dy, 70.0); // height - 10
    }

    // ── draw_database ──

    #[test]
    fn database_shape() {
        let s = draw_database(100.0, 80.0, 3.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 1);
        // Verify shadow on body
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => assert_eq!(p.shadow, 3.0),
            _ => panic!("expected Path"),
        }
        // Closing arc should have no fill
        assert!(s.decorations[0].no_fill);
    }

    #[test]
    fn database_path_starts_at_top() {
        let s = draw_database(100.0, 80.0, 0.0);
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => {
                // First segment is moveTo(0, 10)
                assert_eq!(p.segments[0].coords[0], 0.0);
                assert_eq!(p.segments[0].coords[1], 10.0);
            }
            _ => panic!("expected Path"),
        }
    }

    // ── draw_folder ──

    #[test]
    fn folder_shape_no_title() {
        let s = draw_folder(200.0, 100.0, XDimension2D::new(0.0, 0.0), 0.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 1); // tab line
    }

    #[test]
    fn folder_wtitle_no_title() {
        let w = folder_wtitle(200.0, XDimension2D::new(0.0, 0.0));
        assert_eq!(w, 50.0); // max(30, 200/4)
    }

    #[test]
    fn folder_wtitle_with_title() {
        let w = folder_wtitle(200.0, XDimension2D::new(60.0, 15.0));
        assert_eq!(w, 66.0); // 60 + 3 + 3
    }

    #[test]
    fn folder_htitle_no_title() {
        assert_eq!(folder_htitle(XDimension2D::new(0.0, 0.0)), 10.0);
    }

    #[test]
    fn folder_htitle_with_title() {
        assert_eq!(folder_htitle(XDimension2D::new(50.0, 15.0)), 21.0); // 15+3+3
    }

    #[test]
    fn folder_rounded() {
        let s = draw_folder(200.0, 100.0, XDimension2D::new(50.0, 15.0), 0.0, 10.0);
        assert_eq!(s.outlines.len(), 1);
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => {
                // Rounded path has arc segments
                let has_arc = p
                    .segments
                    .iter()
                    .any(|seg| matches!(seg.kind, crate::klimt::geom::USegmentType::ArcTo));
                assert!(has_arc);
            }
            _ => panic!("expected Path for rounded folder"),
        }
    }

    // ── draw_cloud ──

    #[test]
    fn cloud_shape() {
        let s = draw_cloud(150.0, 100.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert!(s.decorations.is_empty());
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => {
                // Should have a reasonable number of segments
                assert!(p.segments.len() > 5);
            }
            _ => panic!("expected Path"),
        }
    }

    #[test]
    fn cloud_deterministic() {
        // Same dimensions should produce identical paths
        let s1 = draw_cloud(200.0, 120.0, 0.0);
        let s2 = draw_cloud(200.0, 120.0, 0.0);
        match (&s1.outlines[0].kind, &s2.outlines[0].kind) {
            (PrimitiveKind::Path(p1), PrimitiveKind::Path(p2)) => {
                assert_eq!(p1.segments.len(), p2.segments.len());
                assert_eq!(p1.to_svg_path_d(), p2.to_svg_path_d());
            }
            _ => panic!("expected Path"),
        }
    }

    #[test]
    fn cloud_small_vs_large() {
        // Small cloud (simple path) vs large cloud (complex path)
        let small = draw_cloud(80.0, 60.0, 0.0);
        let large = draw_cloud(200.0, 150.0, 0.0);
        match (&small.outlines[0].kind, &large.outlines[0].kind) {
            (PrimitiveKind::Path(ps), PrimitiveKind::Path(pl)) => {
                // Both should have segments; large might have more due to complex mode
                assert!(ps.segments.len() > 3);
                assert!(pl.segments.len() > 3);
            }
            _ => panic!("expected Path"),
        }
    }

    // ── draw_node ──

    #[test]
    fn node_shape() {
        let s = draw_node(100.0, 80.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 3); // diagonal, hline, vline
    }

    #[test]
    fn node_decoration_positions() {
        let s = draw_node(100.0, 80.0, 0.0);
        // Diagonal line at (width-10, 10)
        assert_eq!(s.decorations[0].dx, 90.0);
        assert_eq!(s.decorations[0].dy, 10.0);
        // Hline at y=10
        assert_eq!(s.decorations[1].dy, 10.0);
        // Vline at x=width-10, y=10
        assert_eq!(s.decorations[2].dx, 90.0);
        assert_eq!(s.decorations[2].dy, 10.0);
    }

    // ── draw_frame ──

    #[test]
    fn frame_shape_no_title() {
        let s = draw_frame(200.0, 100.0, XDimension2D::new(0.0, 0.0), 0.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 1); // tab corner path
        assert!(s.decorations[0].no_fill);
    }

    #[test]
    fn frame_ypos_no_title() {
        assert_eq!(frame_ypos(XDimension2D::new(0.0, 0.0)), 12.0);
    }

    #[test]
    fn frame_ypos_with_title() {
        assert_eq!(frame_ypos(XDimension2D::new(50.0, 14.0)), 17.0); // 14+3
    }

    // ── draw_usecase ──

    #[test]
    fn usecase_shape() {
        let s = draw_usecase(100.0, 60.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert!(s.decorations.is_empty());
        match &s.outlines[0].kind {
            PrimitiveKind::Ellipse(e) => {
                assert_eq!(e.width, 100.0);
                assert_eq!(e.height, 60.0);
            }
            _ => panic!("expected Ellipse"),
        }
    }

    // ── draw_card ──

    #[test]
    fn card_shape_no_divider() {
        let s = draw_card(100.0, 60.0, 0.0, 0.0, 5.0);
        assert_eq!(s.outlines.len(), 1);
        assert!(s.decorations.is_empty());
    }

    #[test]
    fn card_shape_with_divider() {
        let s = draw_card(100.0, 60.0, 0.0, 20.0, 5.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 1); // horizontal line
        assert_eq!(s.decorations[0].dy, 20.0);
    }

    // ── draw_storage ──

    #[test]
    fn storage_shape() {
        let s = draw_storage(100.0, 60.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        match &s.outlines[0].kind {
            PrimitiveKind::Rect(r) => {
                assert_eq!(r.rx, 70.0);
                assert_eq!(r.ry, 70.0);
            }
            _ => panic!("expected Rect"),
        }
    }

    // ── draw_queue ──

    #[test]
    fn queue_shape() {
        let s = draw_queue(100.0, 60.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 1);
        assert!(s.decorations[0].no_fill);
    }

    // ── draw_stack ──

    #[test]
    fn stack_shape() {
        let s = draw_stack(100.0, 60.0, 3.0, 0.0);
        assert_eq!(s.outlines.len(), 1); // inner rect
        assert_eq!(s.decorations.len(), 1); // bracket path
                                            // Inner rect is offset by border=15
        assert_eq!(s.outlines[0].dx, 15.0);
        match &s.outlines[0].kind {
            PrimitiveKind::Rect(r) => {
                assert_eq!(r.width, 70.0); // 100 - 2*15
            }
            _ => panic!("expected Rect"),
        }
    }

    // ── draw_hexagon ──

    #[test]
    fn hexagon_shape() {
        let s = draw_hexagon(100.0, 60.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => {
                // move + 6 lines + close
                assert_eq!(p.segments.len(), 8);
            }
            _ => panic!("expected Path"),
        }
    }

    // ── draw_person ──

    #[test]
    fn person_head_size_calculation() {
        let dim = XDimension2D::new(100.0, 100.0);
        let hs = person_head_size(dim);
        // sqrt(10000) * 0.42 = 100 * 0.42 = 42
        assert!((hs - 42.0).abs() < 0.01);
    }

    #[test]
    fn person_shape() {
        let body = XDimension2D::new(80.0, 60.0);
        let s = draw_person(body, 0.0);
        assert_eq!(s.outlines.len(), 2); // head + body
                                         // Head should be an ellipse
        match &s.outlines[0].kind {
            PrimitiveKind::Ellipse(e) => {
                assert!(e.width > 0.0);
                assert_eq!(e.width, e.height); // circle
            }
            _ => panic!("expected Ellipse for head"),
        }
        // Body should be a rounded rect
        match &s.outlines[1].kind {
            PrimitiveKind::Rect(r) => {
                assert_eq!(r.width, 80.0);
                assert_eq!(r.height, 60.0);
            }
            _ => panic!("expected Rect for body"),
        }
    }

    // ── draw_file ──

    #[test]
    fn file_shape_sharp() {
        let s = draw_file(100.0, 80.0, 0.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 1); // dog-ear fold
    }

    #[test]
    fn file_shape_rounded() {
        let s = draw_file(100.0, 80.0, 0.0, 10.0);
        assert_eq!(s.outlines.len(), 1);
        assert_eq!(s.decorations.len(), 1);
        // Rounded version uses arcs in the fold
        match &s.decorations[0].kind {
            PrimitiveKind::Path(p) => {
                let has_arc = p
                    .segments
                    .iter()
                    .any(|seg| matches!(seg.kind, crate::klimt::geom::USegmentType::ArcTo));
                assert!(has_arc);
            }
            _ => panic!("expected Path for fold"),
        }
    }

    // ── draw_artifact ──

    #[test]
    fn artifact_shape() {
        let s = draw_artifact(120.0, 80.0, 0.0, 0.0);
        assert_eq!(s.outlines.len(), 1); // main rect
        assert_eq!(s.decorations.len(), 3); // icon polygon + vline + hline
    }

    // ── draw_collections ──

    #[test]
    fn collections_shape() {
        let s = draw_collections(100.0, 60.0, 0.0, 0.0);
        assert_eq!(s.outlines.len(), 2); // back + front rect
                                         // Back rect offset by delta=4
        assert_eq!(s.outlines[0].dx, 4.0);
        assert_eq!(s.outlines[0].dy, 4.0);
        match &s.outlines[0].kind {
            PrimitiveKind::Rect(r) => {
                assert_eq!(r.width, 96.0);
                assert_eq!(r.height, 56.0);
            }
            _ => panic!("expected Rect"),
        }
    }

    // ── draw_action ──

    #[test]
    fn action_shape() {
        let s = draw_action(100.0, 60.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => {
                // move + 4 lines + close = 6 segments
                assert_eq!(p.segments.len(), 6);
            }
            _ => panic!("expected Path"),
        }
    }

    // ── draw_process ──

    #[test]
    fn process_shape() {
        let s = draw_process(100.0, 60.0, 0.0);
        assert_eq!(s.outlines.len(), 1);
        match &s.outlines[0].kind {
            PrimitiveKind::Path(p) => {
                // move + 5 lines + close = 7 segments
                assert_eq!(p.segments.len(), 7);
            }
            _ => panic!("expected Path"),
        }
    }

    // ── draw_label ──

    #[test]
    fn label_shape() {
        let s = draw_label();
        assert!(s.outlines.is_empty());
        assert!(s.decorations.is_empty());
    }

    // ── JavaRandom ──

    #[test]
    fn java_random_deterministic() {
        let mut r1 = JavaRandom::new(42);
        let mut r2 = JavaRandom::new(42);
        for _ in 0..100 {
            assert_eq!(r1.next_double(), r2.next_double());
        }
    }

    #[test]
    fn java_random_range() {
        let mut r = JavaRandom::new(123);
        for _ in 0..1000 {
            let v = r.next_double();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn java_random_first_values() {
        // Java: new Random(0).nextDouble() => 0.730967787376657
        let mut r = JavaRandom::new(0);
        let v = r.next_double();
        assert!((v - 0.730967787376657).abs() < 1e-12);
    }
}
