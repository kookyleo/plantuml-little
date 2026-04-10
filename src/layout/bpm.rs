use std::collections::HashMap;

use crate::font_metrics;
use crate::model::bpm::{BpmDiagram, BpmElementType, BpmEvent, Where};
use crate::Result;

/// Java GridArray margin between cells.
const CELL_MARGIN: f64 = 30.0;

/// Connector line length (10px).
#[allow(dead_code)] // reserved for future BPM connector rendering
const CONNECTOR_LEN: f64 = 10.0;

/// Start circle radius (Java FtileCircleStart: circledCharacterRadius from SkinParam = 10).
const START_RADIUS: f64 = 10.0;

/// Diamond half-size (Java FtileDiamond: 12px each side).
const DIAMOND_HALF: f64 = 12.0;

/// Task box corner radius.
#[allow(dead_code)] // reserved for future BPM box rendering
const BOX_CORNER_RADIUS: f64 = 12.5;

/// Task box font size (Java FtileBox uses SansSerif 12pt).
const BOX_FONT_SIZE: f64 = 12.0;

/// Task box padding (Java FtileBox style.getPadding() defaults to 10 on all sides).
const BOX_PADDING_TOP: f64 = 10.0;
const BOX_PADDING_BOTTOM: f64 = 10.0;
const BOX_PADDING_LEFT: f64 = 10.0;
const BOX_PADDING_RIGHT: f64 = 10.0;

/// Laid-out BPM element in the grid.
#[derive(Debug, Clone)]
pub struct BpmCellLayout {
    pub element_type: BpmElementType,
    pub label: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Connector lines to draw.
    pub connectors: Vec<Where>,
    /// Grid row index (for rendering order).
    pub row: usize,
    /// Grid col index (for rendering order).
    pub col: usize,
}

/// A connector puzzle cell (intermediate routing).
#[derive(Debug, Clone)]
pub struct BpmConnectorLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Directions this connector links.
    pub directions: Vec<Where>,
    /// Grid row index (for rendering order).
    pub row: usize,
    /// Grid col index (for rendering order).
    pub col: usize,
}

/// Grid line (horizontal) for internal borders.
#[derive(Debug, Clone)]
pub struct GridLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// BPM layout result.
#[derive(Debug)]
pub struct BpmLayout {
    pub width: f64,
    pub height: f64,
    pub cells: Vec<BpmCellLayout>,
    pub connectors: Vec<BpmConnectorLayout>,
    pub grid_lines: Vec<GridLine>,
}

/// Internal grid cell data.
#[derive(Debug, Clone)]
enum CellData {
    Element(GridElement),
    Connector(Vec<Where>),
}

#[derive(Debug, Clone)]
struct GridElement {
    element_type: BpmElementType,
    label: Option<String>,
    id: Option<String>,
    connectors: Vec<Where>,
    destinations: Vec<usize>, // indices into grid cells by id
}

/// Identity-based linked list item for grid lines/cols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LineId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ColId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord {
    line: LineId,
    col: ColId,
}

/// The grid manages ordered lines and columns, with cells at intersections.
struct Grid {
    lines: Vec<LineId>,
    cols: Vec<ColId>,
    cells: HashMap<Coord, CellData>,
    next_line_id: usize,
    next_col_id: usize,
}

impl Grid {
    fn new() -> Self {
        let root_line = LineId(0);
        let root_col = ColId(0);
        Grid {
            lines: vec![root_line],
            cols: vec![root_col],
            cells: HashMap::new(),
            next_line_id: 1,
            next_col_id: 1,
        }
    }

    fn root(&self) -> Coord {
        Coord {
            line: self.lines[0],
            col: self.cols[0],
        }
    }

    fn new_line(&mut self) -> LineId {
        let id = LineId(self.next_line_id);
        self.next_line_id += 1;
        id
    }

    fn new_col(&mut self) -> ColId {
        let id = ColId(self.next_col_id);
        self.next_col_id += 1;
        id
    }

    fn insert_col_after(&mut self, after: ColId) -> ColId {
        let new_col = self.new_col();
        let pos = self.cols.iter().position(|c| *c == after).unwrap();
        self.cols.insert(pos + 1, new_col);
        new_col
    }

    fn insert_line_after(&mut self, after: LineId) -> LineId {
        let new_line = self.new_line();
        let pos = self.lines.iter().position(|l| *l == after).unwrap();
        self.lines.insert(pos + 1, new_line);
        new_line
    }

    fn set_cell(&mut self, coord: Coord, data: CellData) {
        self.cells.insert(coord, data);
    }

    #[allow(dead_code)] // reserved for future BPM layout queries
    fn get_cell(&self, coord: &Coord) -> Option<&CellData> {
        self.cells.get(coord)
    }

    fn find_by_id(&self, id: &str) -> Option<Coord> {
        for (coord, data) in &self.cells {
            if let CellData::Element(e) = data {
                if e.id.as_deref() == Some(id) {
                    return Some(*coord);
                }
            }
        }
        None
    }

    fn line_index(&self, line: LineId) -> usize {
        self.lines.iter().position(|l| *l == line).unwrap()
    }

    fn col_index(&self, col: ColId) -> usize {
        self.cols.iter().position(|c| *c == col).unwrap()
    }

    fn add_puzzle(&mut self, line: LineId, col: ColId, directions: &[Where]) {
        let coord = Coord { line, col };
        match self.cells.get_mut(&coord) {
            Some(CellData::Connector(ref mut dirs)) => {
                for d in directions {
                    if !dirs.contains(d) {
                        dirs.push(*d);
                    }
                }
            }
            None => {
                self.cells
                    .insert(coord, CellData::Connector(directions.to_vec()));
            }
            _ => {
                // Cell has an element; add connectors to the element instead
                if let Some(CellData::Element(e)) = self.cells.get_mut(&coord) {
                    for d in directions {
                        if !e.connectors.contains(d) {
                            e.connectors.push(*d);
                        }
                    }
                }
            }
        }
    }

    /// Check if a line has any cells with data (elements or connectors).
    fn used_cols_of(&self, line: LineId) -> Vec<ColId> {
        let mut used = Vec::new();
        for (coord, data) in &self.cells {
            if coord.line != line {
                continue;
            }
            match data {
                CellData::Element(_) => used.push(coord.col),
                CellData::Connector(_) => used.push(coord.col),
            }
        }
        used
    }

    /// Remove a line from the grid (must have no data cells).
    fn remove_line(&mut self, line: LineId) {
        self.cells.retain(|coord, _| coord.line != line);
        self.lines.retain(|l| *l != line);
    }

    /// Run the CleanerEmptyLine algorithm: remove lines with no data.
    fn clean_empty_lines(&mut self) -> bool {
        let mut removed = false;
        let lines: Vec<LineId> = self.lines.clone();
        for line in lines {
            if self.used_cols_of(line).is_empty() {
                self.remove_line(line);
                removed = true;
            }
        }
        removed
    }

    /// Check if two cells can be merged (for CleanerInterleavingLines).
    fn cells_mergeable(&self, c1: Option<&CellData>, c2: Option<&CellData>) -> bool {
        match (c1, c2) {
            (None, _) | (_, None) => true,
            (Some(CellData::Connector(_)), Some(CellData::Connector(_))) => {
                // Both connectors: mergeable if both are NS, or NS+NE, or NS+NW
                true // Simplified: allow connector merging
            }
            (Some(CellData::Connector(dirs)), Some(CellData::Element(_))) => {
                // Puzzle + Element: mergeable if puzzle is NS or SW
                dirs.contains(&Where::North) && dirs.contains(&Where::South)
                    || dirs.contains(&Where::South) && dirs.contains(&Where::West)
            }
            (Some(CellData::Element(_)), Some(CellData::Connector(dirs))) => {
                dirs.contains(&Where::North) && dirs.contains(&Where::South)
            }
            (Some(CellData::Element(_)), Some(CellData::Element(_))) => false,
        }
    }

    /// Check if two adjacent lines can be merged.
    fn lines_mergeable(&self, line1: LineId, line2: LineId) -> bool {
        for col in &self.cols {
            let c1 = self.cells.get(&Coord {
                line: line1,
                col: *col,
            });
            let c2 = self.cells.get(&Coord {
                line: line2,
                col: *col,
            });
            if !self.cells_mergeable(c1, c2) {
                return false;
            }
        }
        true
    }

    /// Merge two lines: for each col, keep whichever cell has data.
    fn merge_lines(&mut self, line1: LineId, line2: LineId) {
        let cols: Vec<ColId> = self.cols.clone();
        for col in cols {
            let coord1 = Coord { line: line1, col };
            let coord2 = Coord { line: line2, col };
            let cell2 = self.cells.remove(&coord2);
            if let Some(data2) = cell2 {
                let cell1 = self.cells.get(&coord1);
                match (cell1, &data2) {
                    (None, _) => {
                        self.cells.insert(coord1, data2);
                    }
                    (Some(CellData::Element(_)), _) => {
                        // Keep cell1 (element takes priority)
                    }
                    (Some(CellData::Connector(dirs1)), CellData::Element(e2)) => {
                        // Puzzle + Element: check if puzzle is SW, modify element
                        let dirs1 = dirs1.clone();
                        let mut e2 = e2.clone();
                        if dirs1.contains(&Where::South) && dirs1.contains(&Where::West) {
                            e2.connectors.retain(|d| *d != Where::North);
                            if !e2.connectors.contains(&Where::West) {
                                e2.connectors.push(Where::West);
                            }
                        }
                        self.cells.insert(coord1, CellData::Element(e2));
                    }
                    (Some(CellData::Connector(_)), CellData::Connector(_)) => {
                        // Keep cell2 (newer connector)
                        self.cells.insert(coord1, data2);
                    }
                }
            }
        }
        self.remove_line(line2);
    }

    /// CleanerInterleavingLines: merge adjacent lines that are compatible.
    fn clean_interleaving_lines(&mut self) -> bool {
        let lines: Vec<LineId> = self.lines.clone();
        for i in 0..lines.len().saturating_sub(1) {
            if self.lines_mergeable(lines[i], lines[i + 1]) {
                self.merge_lines(lines[i], lines[i + 1]);
                return true;
            }
        }
        false
    }

    /// Run all cleaners until fixpoint (mirrors Java BpmDiagram.cleanGrid).
    fn clean(&mut self) {
        loop {
            let v1 = self.clean_empty_lines();
            let v2 = self.clean_interleaving_lines();
            // CleanerMoveBlock always returns false in Java, skip it.
            if !v1 && !v2 {
                return;
            }
        }
    }
}

/// Compute the dimension of a single BPM element.
fn element_size(etype: &BpmElementType, label: Option<&str>) -> (f64, f64) {
    match etype {
        BpmElementType::Start => {
            // Circle: diameter 20, but the Java block includes stroke margins.
            // Java FtileCircleStart calculateDimension = (20, 20).
            (START_RADIUS * 2.0, START_RADIUS * 2.0)
        }
        BpmElementType::Merge => {
            // Diamond: 24x24
            (DIAMOND_HALF * 2.0, DIAMOND_HALF * 2.0)
        }
        BpmElementType::DockedEvent => {
            // Box: padding + text
            let text = label.unwrap_or("");
            let tw = font_metrics::text_width(text, "SansSerif", BOX_FONT_SIZE, false, false);
            let w = BOX_PADDING_LEFT + tw + BOX_PADDING_RIGHT;
            let line_h = font_metrics::line_height("SansSerif", BOX_FONT_SIZE, false, false);
            let h = BOX_PADDING_TOP + line_h + BOX_PADDING_BOTTOM;
            (w, h)
        }
        BpmElementType::End => (START_RADIUS * 2.0, START_RADIUS * 2.0),
    }
}

pub fn layout_bpm(d: &BpmDiagram) -> Result<BpmLayout> {
    // Phase 1: Build the grid from events (mirrors Java BpmDiagram.createGrid)
    let mut grid = Grid::new();
    let root = grid.root();
    let mut current = root;
    let mut last_coord = root;

    // Place start element at root
    grid.set_cell(
        root,
        CellData::Element(GridElement {
            element_type: BpmElementType::Start,
            label: None,
            id: None,
            connectors: Vec::new(),
            destinations: Vec::new(),
        }),
    );

    for event in &d.events {
        match event {
            BpmEvent::Add(element) => {
                // Insert new column after current
                let new_col = grid.insert_col_after(current.col);
                current = Coord {
                    line: current.line,
                    col: new_col,
                };
                grid.set_cell(
                    current,
                    CellData::Element(GridElement {
                        element_type: element.element_type.clone(),
                        label: element.label.clone(),
                        id: element.id.clone(),
                        connectors: Vec::new(),
                        destinations: Vec::new(),
                    }),
                );
                // Record connection from last to current
                add_destination(&mut grid, last_coord, current);
                last_coord = current;
            }
            BpmEvent::Resume(id) => {
                if let Some(dest) = grid.find_by_id(id) {
                    current = dest;
                    last_coord = dest;
                    // Insert new line after current line
                    let new_line = grid.insert_line_after(current.line);
                    current = Coord {
                        line: new_line,
                        col: current.col,
                    };
                }
            }
            BpmEvent::Goto(id) => {
                if let Some(dest) = grid.find_by_id(id) {
                    let src = last_coord;
                    last_coord = dest;
                    // Insert new line after dest line
                    let new_line = grid.insert_line_after(dest.line);
                    current = Coord {
                        line: new_line,
                        col: dest.col,
                    };
                    // Record connection from src to dest
                    add_destination(&mut grid, src, dest);
                }
            }
        }
    }

    // Phase 2: Add connections (draw connector lines), then clean
    add_connections(&mut grid);
    grid.clean();

    // Phase 3: Convert grid to array layout
    let num_lines = grid.lines.len();
    let num_cols = grid.cols.len();

    // Compute row heights and column widths
    let mut col_widths = vec![0.0f64; num_cols];
    let mut row_heights = vec![0.0f64; num_lines];

    for (coord, data) in &grid.cells {
        let li = grid.line_index(coord.line);
        let ci = grid.col_index(coord.col);
        let (w, h) = match data {
            CellData::Element(e) => element_size(&e.element_type, e.label.as_deref()),
            CellData::Connector(_) => (0.0, 0.0), // Connectors don't take space
        };
        col_widths[ci] = col_widths[ci].max(w);
        row_heights[li] = row_heights[li].max(h);
    }

    // Total dimensions including margins
    let total_width: f64 = col_widths.iter().map(|w| w + CELL_MARGIN).sum();
    let total_height: f64 = row_heights.iter().map(|h| h + CELL_MARGIN).sum();

    // Compute cumulative positions
    let mut col_starts = vec![0.0f64; num_cols];
    let mut running = 0.0;
    for ci in 0..num_cols {
        col_starts[ci] = running;
        running += col_widths[ci] + CELL_MARGIN;
    }

    let mut row_starts = vec![0.0f64; num_lines];
    running = 0.0;
    for li in 0..num_lines {
        row_starts[li] = running;
        running += row_heights[li] + CELL_MARGIN;
    }

    // Build layout cells and connectors
    let mut cells = Vec::new();
    let mut connectors = Vec::new();

    for (coord, data) in &grid.cells {
        let li = grid.line_index(coord.line);
        let ci = grid.col_index(coord.col);
        let cell_x = col_starts[ci];
        let cell_y = row_starts[li];
        let cw = col_widths[ci];
        let ch = row_heights[li];

        match data {
            CellData::Element(e) => {
                let (ew, eh) = element_size(&e.element_type, e.label.as_deref());
                // Center element in cell (with margin)
                let x = cell_x + (cw + CELL_MARGIN - ew) / 2.0;
                let y = cell_y + (ch + CELL_MARGIN - eh) / 2.0;
                cells.push(BpmCellLayout {
                    element_type: e.element_type.clone(),
                    label: e.label.clone(),
                    x,
                    y,
                    width: ew,
                    height: eh,
                    connectors: e.connectors.clone(),
                    row: li,
                    col: ci,
                });
            }
            CellData::Connector(dirs) => {
                // Connector puzzle dimension is 20x20 in Java (ConnectorPuzzleEmpty).
                let puzzle_dim = 20.0;
                let px = cell_x + (cw + CELL_MARGIN - puzzle_dim) / 2.0;
                let py = cell_y + (ch + CELL_MARGIN - puzzle_dim) / 2.0;
                connectors.push(BpmConnectorLayout {
                    x: px,
                    y: py,
                    width: puzzle_dim,
                    height: puzzle_dim,
                    directions: dirs.clone(),
                    row: li,
                    col: ci,
                });
            }
        }
    }

    // Build grid lines
    let mut grid_lines = Vec::new();
    // Horizontal lines at each row boundary
    for li in 0..num_lines {
        let y = row_starts[li];
        grid_lines.push(GridLine {
            x1: 0.0,
            y1: y,
            x2: total_width,
            y2: y,
        });
    }
    // Vertical lines at each column boundary
    for ci in 0..num_cols {
        let x = col_starts[ci];
        grid_lines.push(GridLine {
            x1: x,
            y1: 0.0,
            x2: x,
            y2: total_height,
        });
    }

    Ok(BpmLayout {
        width: total_width,
        height: total_height,
        cells,
        connectors,
        grid_lines,
    })
}

/// Record a destination from src cell to dest cell in the grid.
fn add_destination(grid: &mut Grid, src: Coord, dest: Coord) {
    // Store dest coord index; we'll process connections later
    if let Some(CellData::Element(e)) = grid.cells.get_mut(&src) {
        // Store the dest line/col id packed as a usize
        e.destinations.push(dest.line.0 * 10000 + dest.col.0);
    }
}

/// Process all connections and add connector puzzle cells + element connectors.
/// Mirrors Java Grid.addConnections().
fn add_connections(grid: &mut Grid) {
    // Collect all (src_coord, dest_packed) pairs first to avoid borrow issues
    let mut connections: Vec<(Coord, Vec<usize>, Vec<usize>)> = Vec::new();
    for (coord, data) in &grid.cells {
        if let CellData::Element(e) = data {
            if !e.destinations.is_empty() {
                connections.push((*coord, e.destinations.clone(), Vec::new()));
            }
        }
    }

    for (src, dests, _) in &connections {
        for (i, packed) in dests.iter().enumerate() {
            let dest_line = LineId(packed / 10000);
            let dest_col = ColId(packed % 10000);
            let dest = Coord {
                line: dest_line,
                col: dest_col,
            };

            let start_horizontal = i == 0;
            if start_horizontal {
                draw_start_horizontal(grid, *src, dest);
            } else {
                draw_start_vertical(grid, *src, dest);
            }
        }
    }
}

fn draw_start_horizontal(grid: &mut Grid, src: Coord, dest: Coord) {
    if src == dest {
        return;
    }

    let src_ci = grid.col_index(src.col);
    let dest_ci = grid.col_index(dest.col);
    let src_li = grid.line_index(src.line);
    let dest_li = grid.line_index(dest.line);

    // Add EAST or WEST connector to source
    let col_dir = if src_ci < dest_ci {
        Where::East
    } else {
        Where::West
    };
    append_connector(grid, src, col_dir);

    // Add horizontal puzzle cells between src and dest columns (on src line)
    let (min_ci, max_ci) = if src_ci < dest_ci {
        (src_ci, dest_ci)
    } else {
        (dest_ci, src_ci)
    };
    for ci in (min_ci + 1)..max_ci {
        let col = grid.cols[ci];
        grid.add_puzzle(src.line, col, &[Where::East, Where::West]);
    }

    // Add vertical puzzle cells between src and dest lines (on dest column)
    let (min_li, max_li) = if src_li < dest_li {
        (src_li, dest_li)
    } else {
        (dest_li, src_li)
    };
    for li in (min_li + 1)..max_li {
        let line = grid.lines[li];
        grid.add_puzzle(line, dest.col, &[Where::North, Where::South]);
    }

    // Add connector to destination
    if src.line == dest.line {
        // Same line: dest gets opposite of col_dir
        let dest_dir = if col_dir == Where::East {
            Where::West
        } else {
            Where::East
        };
        append_connector(grid, dest, dest_dir);
    }

    if src.line != dest.line && src.col != dest.col {
        // Corner: add puzzle at (src.line, dest.col)
        let _corner = Coord {
            line: src.line,
            col: dest.col,
        };
        let h_dir = if dest_ci > src_ci {
            Where::West
        } else {
            Where::East
        };
        let v_dir = if dest_li > src_li {
            Where::South
        } else {
            Where::North
        };
        grid.add_puzzle(src.line, dest.col, &[h_dir, v_dir]);

        // Dest gets the opposite vertical direction
        let dest_dir = if src_li > dest_li {
            Where::South
        } else {
            Where::North
        };
        append_connector(grid, dest, dest_dir);
    }
}

fn draw_start_vertical(grid: &mut Grid, src: Coord, dest: Coord) {
    if src == dest {
        return;
    }

    let src_li = grid.line_index(src.line);
    let dest_li = grid.line_index(dest.line);
    let src_ci = grid.col_index(src.col);
    let dest_ci = grid.col_index(dest.col);

    // Add SOUTH or NORTH connector to source
    let line_dir = if src_li < dest_li {
        Where::South
    } else {
        Where::North
    };
    append_connector(grid, src, line_dir);

    // Add vertical puzzle cells
    let (min_li, max_li) = if src_li < dest_li {
        (src_li, dest_li)
    } else {
        (dest_li, src_li)
    };
    for li in (min_li + 1)..max_li {
        let line = grid.lines[li];
        grid.add_puzzle(line, src.col, &[Where::North, Where::South]);
    }

    // Add horizontal puzzle cells
    let (min_ci, max_ci) = if src_ci < dest_ci {
        (src_ci, dest_ci)
    } else {
        (dest_ci, src_ci)
    };
    for ci in (min_ci + 1)..max_ci {
        let col = grid.cols[ci];
        grid.add_puzzle(dest.line, col, &[Where::East, Where::West]);
    }

    // Add connector to destination
    if src.line == dest.line {
        let dest_dir = if line_dir == Where::South {
            Where::North
        } else {
            Where::South
        };
        append_connector(grid, dest, dest_dir);
    }

    if src.line != dest.line && src.col != dest.col {
        // Corner: add puzzle at (dest.line, src.col)
        let v_dir = if dest_li > src_li {
            Where::North
        } else {
            Where::South
        };
        let h_dir = if dest_ci > src_ci {
            Where::East
        } else {
            Where::West
        };
        grid.add_puzzle(dest.line, src.col, &[v_dir, h_dir]);

        let dest_dir = if src_ci > dest_ci {
            Where::East
        } else {
            Where::West
        };
        append_connector(grid, dest, dest_dir);
    }
}

fn append_connector(grid: &mut Grid, coord: Coord, dir: Where) {
    if let Some(CellData::Element(e)) = grid.cells.get_mut(&coord) {
        if !e.connectors.contains(&dir) {
            e.connectors.push(dir);
        }
    }
}
