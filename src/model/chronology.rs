/// A single event on a chronology timeline.
#[derive(Debug, Clone)]
pub struct ChronologyEvent {
    pub date: String,
    pub label: String,
}

/// Chronology diagram model.
#[derive(Debug, Clone)]
pub struct ChronologyDiagram {
    pub events: Vec<ChronologyEvent>,
}
