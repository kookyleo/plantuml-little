/// Pseudo-state kind for special state nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum StateKind {
    /// Regular state (default).
    #[default]
    Normal,
    /// Fork bar — synchronization split.
    Fork,
    /// Join bar — synchronization merge.
    Join,
    /// Choice pseudo-state (diamond).
    Choice,
    /// End pseudo-state (<<end>> stereotype).
    End,
    /// Shallow history (H).
    History,
    /// Deep history (H*).
    DeepHistory,
    /// Entry point pseudo-state.
    EntryPoint,
    /// Exit point pseudo-state.
    ExitPoint,
}

/// State in a state diagram
#[derive(Debug, Clone)]
pub struct State {
    /// State name (display name)
    pub name: String,
    /// State ID (used for references)
    pub id: String,
    /// Description lines
    pub description: Vec<String>,
    /// Stereotype (e.g. <<inputPin>>)
    pub stereotype: Option<String>,
    /// Child states (composite state)
    pub children: Vec<State>,
    /// Whether this is a special state [*]
    pub is_special: bool,
    /// Pseudo-state kind (fork, join, choice, history, etc.)
    pub kind: StateKind,
    /// Concurrent regions within a composite state.
    /// Each region is a list of child states.
    /// If non-empty, `children` holds the first region and `regions` holds additional regions.
    pub regions: Vec<Vec<State>>,
}

/// State transition
#[derive(Debug, Clone)]
pub struct Transition {
    /// Source state ID
    pub from: String,
    /// Target state ID
    pub to: String,
    /// Transition label
    pub label: String,
    /// Arrow style: `->` (solid) or `-->` (dashed) -- both rendered as solid in state diagrams
    pub dashed: bool,
}

/// Note
#[derive(Debug, Clone)]
pub struct StateNote {
    /// Note alias
    pub alias: Option<String>,
    /// Note text
    pub text: String,
}

/// State diagram IR
#[derive(Debug, Clone)]
pub struct StateDiagram {
    /// All top-level states
    pub states: Vec<State>,
    /// All transitions
    pub transitions: Vec<Transition>,
    /// Notes
    pub notes: Vec<StateNote>,
    /// Layout direction
    pub direction: super::diagram::Direction,
}
