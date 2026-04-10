/// Default "rose" theme color palette for PlantUML diagrams.
///
/// This centralizes the default colors used across all diagram types when no
/// explicit skinparam overrides are specified. The name "rose" comes from
/// PlantUML's built-in default theme.
#[derive(Debug, Clone)]
pub struct Theme {
    // ── Global ──────────────────────────────────────────────────────
    pub background_color: String,
    pub font_color: String,
    pub arrow_color: String,
    pub border_color: String,

    // ── Class / Object ──────────────────────────────────────────────
    pub class_bg: String,
    pub class_border: String,
    pub class_font: String,

    // ── Sequence ────────────────────────────────────────────────────
    pub participant_bg: String,
    pub participant_border: String,
    pub lifeline_color: String,
    pub activation_bg: String,
    pub activation_border: String,
    pub group_bg: String,
    pub group_border: String,

    // ── Note (shared across diagrams) ───────────────────────────────
    pub note_bg: String,
    pub note_border: String,

    // ── Activity ────────────────────────────────────────────────────
    pub activity_bg: String,
    pub activity_border: String,
    pub diamond_bg: String,
    pub diamond_border: String,
    pub swimlane_border: String,
    pub swimlane_header_bg: String,

    // ── State ───────────────────────────────────────────────────────
    pub state_bg: String,
    pub state_border: String,
    pub composite_bg: String,
    pub composite_border: String,

    // ── Component ───────────────────────────────────────────────────
    pub component_bg: String,
    pub component_border: String,
    pub node_bg: String,
    pub node_border: String,
    pub database_bg: String,
    pub database_border: String,
    pub cloud_bg: String,
    pub cloud_border: String,

    // ── ERD ─────────────────────────────────────────────────────────
    pub entity_bg: String,
    pub entity_border: String,
    pub relationship_bg: String,
    pub relationship_border: String,

    // ── Mindmap / WBS ───────────────────────────────────────────────
    pub mindmap_node_bg: String,
    pub mindmap_node_border: String,
    pub wbs_root_bg: String,

    // ── Legend ──────────────────────────────────────────────────────
    pub legend_bg: String,
    pub legend_border: String,
}

impl Theme {
    /// Construct the default theme, matching Java PlantUML's current defaults.
    pub fn rose() -> Self {
        Self {
            // Global
            background_color: "#FFFFFF".into(),
            font_color: "#000000".into(),
            arrow_color: "#181818".into(),
            border_color: "#181818".into(),

            // Class / Object
            class_bg: "#F1F1F1".into(),
            class_border: "#181818".into(),
            class_font: "#000000".into(),

            // Sequence
            participant_bg: "#E2E2F0".into(),
            participant_border: "#181818".into(),
            lifeline_color: "#181818".into(),
            activation_bg: "#F1F1F1".into(),
            activation_border: "#181818".into(),
            group_bg: "#EEEEEE".into(),
            group_border: "#000000".into(),

            // Note
            note_bg: "#FEFFDD".into(),
            note_border: "#181818".into(),

            // Activity
            activity_bg: "#F1F1F1".into(),
            activity_border: "#181818".into(),
            diamond_bg: "#F1F1F1".into(),
            diamond_border: "#181818".into(),
            swimlane_border: "#181818".into(),
            swimlane_header_bg: "#F1F1F1".into(),

            // State
            state_bg: "#F1F1F1".into(),
            state_border: "#181818".into(),
            composite_bg: "#F1F1F1".into(),
            composite_border: "#181818".into(),

            // Component
            component_bg: "#F1F1F1".into(),
            component_border: "#181818".into(),
            node_bg: "#F1F1F1".into(),
            node_border: "#181818".into(),
            database_bg: "#F1F1F1".into(),
            database_border: "#181818".into(),
            cloud_bg: "#F1F1F1".into(),
            cloud_border: "#181818".into(),

            // ERD
            entity_bg: "#F1F1F1".into(),
            entity_border: "#181818".into(),
            relationship_bg: "#F1F1F1".into(),
            relationship_border: "#181818".into(),

            // Mindmap / WBS
            mindmap_node_bg: "#F1F1F1".into(),
            mindmap_node_border: "#181818".into(),
            wbs_root_bg: "#FFD700".into(),

            // Legend
            legend_bg: "#FEFFDD".into(),
            legend_border: "#000000".into(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::rose()
    }
}
