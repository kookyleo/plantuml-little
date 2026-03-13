/// Gantt chart diagram IR

#[derive(Debug, Clone)]
pub struct GanttTask {
    pub name: String,
    pub alias: Option<String>,
    pub duration_days: u32,
    pub color: Option<String>,
    pub start_date: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GanttDependency {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct GanttColoredRange {
    pub start: String,
    pub end: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct GanttNote {
    pub text: String,
    pub position: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GanttDiagram {
    pub tasks: Vec<GanttTask>,
    pub dependencies: Vec<GanttDependency>,
    pub project_start: Option<String>,
    pub closed_days: Vec<String>,
    pub colored_ranges: Vec<GanttColoredRange>,
    pub scale: Option<u32>,
    pub print_scale: Option<String>,
    pub notes: Vec<GanttNote>,
}
