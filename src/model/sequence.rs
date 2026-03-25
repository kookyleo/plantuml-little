/// Participant kind
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ParticipantKind {
    #[default]
    Default,
    Actor,
    Boundary,
    Control,
    Entity,
    Database,
    Collections,
    Queue,
}

/// Participant
#[derive(Debug, Clone)]
pub struct Participant {
    pub name: String,
    pub display_name: Option<String>,
    pub kind: ParticipantKind,
    pub color: Option<String>,
    /// Source line number (0-based) for data-source-line attribute
    pub source_line: Option<usize>,
    /// URL from `[[url text]]` link markup in the display name.
    /// Used for Java-compatible `<title>` encoding in lifelines.
    pub link_url: Option<String>,
}

/// Message arrow style
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SeqArrowStyle {
    #[default]
    Solid, // ->
    Dashed, // -->
    Dotted, // ..>  (rarely used but reserved)
}

/// Message arrow head
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SeqArrowHead {
    #[default]
    Filled,     // > or / or \ — filled triangle or half-arrow
    Open,       // >> or << — open V-shaped head (2 lines)
    HalfTop,    // // — open half-arrow, upper line only
    HalfBottom, // \\ — open half-arrow, lower line only
}

/// Message direction
#[derive(Debug, Clone, PartialEq)]
pub enum SeqDirection {
    LeftToRight, // ->
    RightToLeft, // <-
}

/// A single message
#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub text: String,
    pub arrow_style: SeqArrowStyle,
    pub arrow_head: SeqArrowHead,
    pub direction: SeqDirection,
    /// Optional arrow color, e.g. `[#blue]->` stores `"blue"`
    pub color: Option<String>,
    /// Source line number (0-based) for data-source-line attribute
    pub source_line: Option<usize>,
    /// Circle decoration on the "from" end of the arrow (o->)
    pub circle_from: bool,
    /// Circle decoration on the "to" end of the arrow (->o)
    pub circle_to: bool,
}

/// Combined fragment kind
#[derive(Debug, Clone, PartialEq)]
pub enum FragmentKind {
    Alt,
    Loop,
    Opt,
    Par,
    Break,
    Critical,
    Group,
}

impl FragmentKind {
    /// Return the display label for this fragment kind
    pub fn label(&self) -> &'static str {
        match self {
            FragmentKind::Alt => "alt",
            FragmentKind::Loop => "loop",
            FragmentKind::Opt => "opt",
            FragmentKind::Par => "par",
            FragmentKind::Break => "break",
            FragmentKind::Critical => "critical",
            FragmentKind::Group => "group",
        }
    }
}

/// Events in a sequence diagram (in chronological order)
#[derive(Debug, Clone)]
pub enum SeqEvent {
    Message(Message),
    Activate(String),
    Deactivate(String),
    Destroy(String),
    NoteRight {
        participant: String,
        text: String,
    },
    NoteLeft {
        participant: String,
        text: String,
    },
    NoteOver {
        participants: Vec<String>,
        text: String,
    },
    /// Legacy group start (kept for backward compatibility, maps to Fragment)
    GroupStart {
        label: Option<String>,
    },
    /// Legacy group end
    GroupEnd,
    /// Combined fragment start (alt, loop, opt, par, break, critical, group)
    FragmentStart {
        kind: FragmentKind,
        label: String,
    },
    /// Fragment separator (else within alt/par)
    FragmentSeparator {
        label: String,
    },
    /// Fragment end
    FragmentEnd,
    /// Reference over participants
    Ref {
        participants: Vec<String>,
        label: String,
    },
    Divider {
        text: Option<String>,
    },
    Delay {
        text: Option<String>,
    },
    /// Explicit spacing: ||| or || N ||
    Spacing {
        pixels: u32,
    },
    /// Auto-numbering control
    AutoNumber {
        start: Option<u32>,
    },
}

/// Sequence diagram IR
#[derive(Debug, Clone)]
pub struct SequenceDiagram {
    pub participants: Vec<Participant>,
    pub events: Vec<SeqEvent>,
    /// Whether `!pragma teoz true` was set (parallel message rendering)
    pub teoz_mode: bool,
    /// Whether `hide footbox` was set (hide tail participant boxes)
    pub hide_footbox: bool,
}
