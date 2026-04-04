/// A task node in a board diagram (Kanban-style).
#[derive(Debug, Clone)]
pub struct BoardTask {
    pub label: String,
    pub level: usize,
    pub children: Vec<BoardTask>,
}

/// Board diagram model.
#[derive(Debug, Clone)]
pub struct BoardDiagram {
    pub tasks: Vec<BoardTask>,
}
