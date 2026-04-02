/// Activity diagram node kind
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityNodeKind {
    /// Start node
    Start,
    /// Stop node
    Stop,
    /// End node (end)
    End,
    /// Action `:text;`
    Action,
    /// Conditional branch
    If,
    /// Merge (endif)
    Merge,
    /// Fork branch
    Fork,
    /// Fork merge
    ForkEnd,
    /// Detach separator `====`
    Detach,
}

/// Note position in activity diagram
#[derive(Debug, Clone, PartialEq)]
pub enum NotePosition {
    Left,
    Right,
}

/// Activity diagram event
#[derive(Debug, Clone)]
pub enum ActivityEvent {
    /// start
    Start,
    /// stop / end
    Stop,
    /// Action node `:text;`
    Action { text: String },
    /// Conditional branch
    If {
        condition: String,
        then_label: String,
    },
    /// Else-if branch
    ElseIf { condition: String, label: String },
    /// else
    Else { label: String },
    /// endif
    EndIf,
    /// While loop
    While { condition: String, label: String },
    /// endwhile
    EndWhile { label: String },
    /// repeat
    Repeat,
    /// repeat while
    RepeatWhile { condition: String },
    /// fork
    Fork,
    /// fork again
    ForkAgain,
    /// end fork
    EndFork,
    /// Swimlane switch
    Swimlane { name: String },
    /// Note
    Note {
        position: NotePosition,
        text: String,
    },
    /// Floating note
    FloatingNote {
        position: NotePosition,
        text: String,
    },
    /// detach
    Detach,
    /// Synchronization bar (old-style `===NAME===`)
    SyncBar(String),
    /// Incoming convergence to an existing sync bar (old-style target)
    GotoSyncBar(String),
    /// Resume layout from a sync bar (old-style source in arrow)
    ResumeFromSyncBar(String),
}

/// Activity diagram IR
#[derive(Debug, Clone)]
pub struct ActivityDiagram {
    pub events: Vec<ActivityEvent>,
    pub swimlanes: Vec<String>,
    pub direction: super::diagram::Direction,
    /// Maximum width for note text wrapping (from `<style>` MaximumWidth).
    pub note_max_width: Option<f64>,
    /// True when the diagram uses old-style `(*)` / `===` activity syntax.
    pub is_old_style: bool,
}
