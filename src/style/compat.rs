use log::debug;
use std::collections::HashMap;

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

/// Parsed skinparam settings from PlantUML source.
///
/// Keys are stored in lowercase for case-insensitive lookup.
/// Element-scoped params use dot notation: `component.backgroundcolor`.
///
/// When no explicit param is set, lookup methods fall back to the embedded
/// [`Theme`] (rose by default).
#[derive(Debug, Clone, Default)]
pub struct SkinParams {
    params: HashMap<String, String>,
    pub theme: Theme,
}

impl SkinParams {
    /// Create an empty SkinParams with the default (rose) theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair. The key is normalized to lowercase.
    pub fn set(&mut self, key: &str, value: &str) {
        let normalized_value = normalize_color(value);
        self.params.insert(key.to_lowercase(), normalized_value);
    }

    /// Get a param value by key (case-insensitive).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params
            .get(&key.to_lowercase())
            .map(std::string::String::as_str)
    }

    /// Get a param value or return the provided default.
    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.params
            .get(&key.to_lowercase())
            .map_or(default, |s| s.as_str())
    }

    /// Get background color for an element type (e.g., "class", "component").
    ///
    /// Lookup order:
    /// 1. `{element}BackgroundColor`
    /// 2. `{element}.BackgroundColor`
    /// 3. `BackgroundColor`
    /// 4. Theme default for the element (if known)
    /// 5. Caller-provided default
    pub fn background_color<'a>(&'a self, element: &str, default: &'a str) -> &'a str {
        let key1 = format!("{element}backgroundcolor");
        let key2 = format!("{element}.backgroundcolor");

        if let Some(v) = self.params.get(&key1) {
            return v.as_str();
        }
        if let Some(v) = self.params.get(&key2) {
            return v.as_str();
        }
        if let Some(v) = self.params.get("root.backgroundcolor") {
            return v.as_str();
        }
        // Note: global "backgroundcolor" is NOT checked here.  In Java PlantUML
        // `skinparam backgroundColor` only sets the diagram canvas background,
        // not element fill colors.  Element fills use their own defaults.
        self.theme_bg(element).unwrap_or(default)
    }

    /// Get font color for an element type.
    ///
    /// Lookup order:
    /// 1. `{element}FontColor`
    /// 2. `{element}.FontColor`
    /// 3. `FontColor`
    /// 4. Theme font color
    /// 5. Caller-provided default
    pub fn font_color<'a>(&'a self, element: &str, default: &'a str) -> &'a str {
        let key1 = format!("{element}fontcolor");
        let key2 = format!("{element}.fontcolor");
        let key3 = "fontcolor";

        if let Some(v) = self.params.get(&key1) {
            return v.as_str();
        }
        if let Some(v) = self.params.get(&key2) {
            return v.as_str();
        }
        if let Some(v) = self.params.get(key3) {
            return v.as_str();
        }
        if let Some(v) = self.params.get("root.fontcolor") {
            return v.as_str();
        }
        self.theme_font(element).unwrap_or(default)
    }

    /// Get border color for an element type.
    ///
    /// Lookup order:
    /// 1. `{element}BorderColor`
    /// 2. `{element}.BorderColor`
    /// 3. `BorderColor`
    /// 4. Theme border color for the element (if known)
    /// 5. Caller-provided default
    pub fn border_color<'a>(&'a self, element: &str, default: &'a str) -> &'a str {
        let key1 = format!("{element}bordercolor");
        let key2 = format!("{element}.bordercolor");
        let key3 = "bordercolor";

        if let Some(v) = self.params.get(&key1) {
            return v.as_str();
        }
        if let Some(v) = self.params.get(&key2) {
            return v.as_str();
        }
        if let Some(v) = self.params.get(key3) {
            return v.as_str();
        }
        if let Some(v) = self.params.get("root.bordercolor") {
            return v.as_str();
        }
        if let Some(v) = self.params.get("root.linecolor") {
            return v.as_str();
        }
        self.theme_border(element).unwrap_or(default)
    }

    /// Get arrow color.
    ///
    /// Lookup order:
    /// 1. `ArrowColor`
    /// 2. Theme arrow color
    pub fn arrow_color<'a>(&'a self, default: &'a str) -> &'a str {
        if let Some(v) = self.params.get("arrowcolor") {
            return v.as_str();
        }
        if default == self.theme.arrow_color {
            return &self.theme.arrow_color;
        }
        default
    }

    // ── Theme element lookups ──────────────────────────────────────

    /// Return the theme background color for a known element, or `None`.
    fn theme_bg(&self, element: &str) -> Option<&str> {
        match element {
            "class" | "object" | "annotation" | "abstract" | "interface" | "enum" => {
                Some(&self.theme.class_bg)
            }
            "participant" => Some(&self.theme.participant_bg),
            "activity" | "action" => Some(&self.theme.activity_bg),
            "state" => Some(&self.theme.state_bg),
            "component" => Some(&self.theme.component_bg),
            "entity" => Some(&self.theme.entity_bg),
            "node" => Some(&self.theme.node_bg),
            "database" => Some(&self.theme.database_bg),
            "cloud" => Some(&self.theme.cloud_bg),
            "note" => Some(&self.theme.note_bg),
            _ => None,
        }
    }

    /// Return the theme border color for a known element, or `None`.
    fn theme_border(&self, element: &str) -> Option<&str> {
        match element {
            "class" | "object" | "annotation" | "abstract" | "interface" | "enum" => {
                Some(&self.theme.class_border)
            }
            "participant" => Some(&self.theme.participant_border),
            "activity" | "action" => Some(&self.theme.activity_border),
            "state" => Some(&self.theme.state_border),
            "component" => Some(&self.theme.component_border),
            "entity" => Some(&self.theme.entity_border),
            "node" => Some(&self.theme.node_border),
            "database" => Some(&self.theme.database_border),
            "cloud" => Some(&self.theme.cloud_border),
            "note" => Some(&self.theme.note_border),
            _ => None,
        }
    }

    /// Return the theme font color for a known element, or `None`.
    fn theme_font(&self, element: &str) -> Option<&str> {
        match element {
            "class" | "object" | "annotation" | "abstract" | "interface" | "enum" => {
                Some(&self.theme.class_font)
            }
            _ => Some(&self.theme.font_color),
        }
    }

    /// Get the default font name. Returns `None` if not set.
    pub fn default_font_name(&self) -> Option<&str> {
        self.params
            .get("defaultfontname")
            .map(std::string::String::as_str)
    }

    /// Get the default font size. Returns `None` if not set.
    pub fn default_font_size(&self) -> Option<f64> {
        self.params
            .get("defaultfontsize")
            .and_then(|s| s.parse::<f64>().ok())
    }

    /// Check if monochrome mode is enabled.
    pub fn is_monochrome(&self) -> bool {
        self.params.get("monochrome").is_some_and(|v| v == "true")
    }

    /// Check if handwritten mode is enabled.
    pub fn is_handwritten(&self) -> bool {
        self.params.get("handwritten").is_some_and(|v| v == "true")
    }

    /// Get the round corner radius. Returns `None` if not set.
    pub fn round_corner(&self) -> Option<f64> {
        self.params
            .get("roundcorner")
            .and_then(|s| s.parse::<f64>().ok())
    }

    /// Get font size for an element type.
    ///
    /// Lookup order:
    /// 1. `{element}FontSize`
    /// 2. `{element}.FontSize`
    /// 3. `defaultFontSize`
    /// 4. Caller-provided default
    pub fn font_size(&self, element: &str, default: f64) -> f64 {
        let key1 = format!("{element}fontsize");
        let key2 = format!("{element}.fontsize");
        let key3 = "defaultfontsize";

        if let Some(v) = self.params.get(&key1).and_then(|s| s.parse::<f64>().ok()) {
            return v;
        }
        if let Some(v) = self.params.get(&key2).and_then(|s| s.parse::<f64>().ok()) {
            return v;
        }
        if let Some(v) = self.params.get(key3).and_then(|s| s.parse::<f64>().ok()) {
            return v;
        }
        default
    }

    /// Get the line thickness for a given element type.
    ///
    /// Lookup chain: `{element}.linethickness` -> `root.linethickness` -> default.
    /// In Java PlantUML, `root { LineThickness N }` in `<style>` sets the
    /// base thickness for all elements.
    pub fn line_thickness(&self, element: &str, default: f64) -> f64 {
        let key1 = format!("{element}.linethickness");
        if let Some(v) = self.params.get(&key1) {
            if let Ok(t) = v.parse::<f64>() {
                return t;
            }
        }
        if let Some(v) = self.params.get("root.linethickness") {
            if let Ok(t) = v.parse::<f64>() {
                return t;
            }
        }
        default
    }

    /// Get sequence arrow thickness.
    pub fn sequence_arrow_thickness(&self) -> Option<f64> {
        self.params
            .get("sequencearrowthickness")
            .and_then(|s| s.parse::<f64>().ok())
    }

    /// Get sequence arrow color with fallback.
    pub fn sequence_arrow_color<'a>(&'a self, default: &'a str) -> &'a str {
        if let Some(v) = self.params.get("sequencearrowcolor") {
            return v.as_str();
        }
        if let Some(v) = self.params.get("sequence.arrowcolor") {
            return v.as_str();
        }
        self.arrow_color(default)
    }

    /// Get sequence lifeline border color with fallback.
    pub fn sequence_lifeline_border_color<'a>(&'a self, default: &'a str) -> &'a str {
        self.params
            .get("sequencelifelinebordercolor")
            .map_or(default, |s| s.as_str())
    }

    /// Get the effective font family for SVG output, considering skinparam overrides.
    pub fn effective_font_family<'a>(&'a self, default: &'a str) -> &'a str {
        if let Some(name) = self.default_font_name() {
            return name;
        }
        default
    }

    /// Get the effective font family for handwritten mode.
    pub fn handwritten_font_family(&self) -> Option<&'static str> {
        if self.is_handwritten() {
            Some("Comic Sans MS, Segoe Print, cursive")
        } else {
            None
        }
    }

    /// Check if any params have been set.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Get the number of params.
    pub fn len(&self) -> usize {
        self.params.len()
    }
}

/// Check if a trimmed line opens an embedded `{{ }}` diagram block.
fn is_embedded_open(trimmed: &str) -> bool {
    if !trimmed.starts_with("{{") {
        return false;
    }
    matches!(
        trimmed,
        "{{"
            | "{{ditaa"
            | "{{salt"
            | "{{uml"
            | "{{wbs"
            | "{{mindmap"
            | "{{gantt"
            | "{{json"
            | "{{yaml"
            | "{{wire"
            | "{{creole"
            | "{{board"
            | "{{ebnf"
            | "{{regex"
            | "{{files"
            | "{{chronology"
            | "{{chen"
            | "{{chart"
            | "{{nwdiag"
            | "{{packetdiag"
    )
}

/// Parse skinparam declarations from PlantUML source text.
///
/// Supports:
/// - Single line: `skinparam BackgroundColor #FEFECE`
/// - Block: `skinparam component { BackgroundColor #FEFECE }`
/// - Nested: `skinparam { component { BackgroundColor #FEFECE } }`
pub fn parse_skinparams(content: &str) -> SkinParams {
    let mut params = SkinParams::new();
    let mut lines = content.lines().peekable();
    let mut in_style_block = false;
    let mut style_content = String::new();
    let mut embedded_depth: usize = 0;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Skip lines inside `{{ }}` embedded diagram blocks.
        // Java: PSystemCommandFactory.addOneSingleLineManageEmbedded2 skips these;
        // the embedded content has its own skinparams that should not affect the parent.
        if is_embedded_open(trimmed) {
            embedded_depth += 1;
            continue;
        }
        if embedded_depth > 0 {
            if trimmed == "}}" {
                embedded_depth -= 1;
            }
            continue;
        }

        // Collect <style> blocks for post-processing
        if trimmed.starts_with("<style>") {
            in_style_block = true;
            continue;
        }
        if in_style_block {
            if trimmed.starts_with("</style>") {
                in_style_block = false;
            } else {
                style_content.push_str(trimmed);
                style_content.push('\n');
            }
            continue;
        }

        let lower = trimmed.to_lowercase();

        // Handle `skin rose` directive: apply Rose theme defaults.
        // Java Rose skin uses legacy ColorParam defaults (MY_RED=#A80036,
        // MY_YELLOW=#FEFECE, COL_FBFB77 for notes). This completely replaces
        // the modern default theme colors.
        if lower.starts_with("skin ") {
            let skin_name = lower["skin ".len()..].trim();
            if skin_name == "rose" {
                // Border colors (Java: MY_RED = #A80036)
                params.set("sequencelifelinebordercolor", "#A80036");
                params.set("participant.bordercolor", "#A80036");
                params.set("participantbordercolor", "#A80036");
                params.set("sequence.bordercolor", "#A80036");
                params.set("notebordercolor", "#A80036");
                params.set("sequencegroupbordercolor", "#A80036");
                // Background colors (Java: MY_YELLOW = #FEFECE)
                params.set("participantbackgroundcolor", "#FEFECE");
                params.set("participant.backgroundcolor", "#FEFECE");
                // Note background (Java: COL_FBFB77)
                params.set("notebackgroundcolor", "#FBFB77");
                // Line thickness (Java: Rose skin default)
                params.set("root.linethickness", "1");
                // Participant stroke-width 1.5 (Java: UStroke(1.5) for participant)
                params.set("participant.linethickness", "1.5");
                // Flag for rendering: no rounded corners, different box layout
                params.set("_skin_rose", "true");
            }
            continue;
        }

        if !lower.starts_with("skinparam") {
            continue;
        }

        // Remove the "skinparam" prefix
        let after = trimmed[9..].trim();

        if after.is_empty() {
            // Bare "skinparam" on its own line - not valid, skip
            continue;
        }

        if after.starts_with('{') {
            // Nested block: skinparam { ... }
            // Content inside can be either:
            //   - key value pairs (global)
            //   - element { key value } blocks
            parse_nested_block(&mut lines, "", &mut params);
        } else if after.contains('{') {
            // Element block: skinparam element { ... }
            // Extract element name (everything before '{')
            let brace_pos = after.find('{').unwrap();
            let element = after[..brace_pos].trim();
            let after_brace = after[brace_pos + 1..].trim();

            // Check if the closing brace is on the same line
            if let Some(close_pos) = after_brace.find('}') {
                // Inline block: skinparam element { key val }
                let inner = after_brace[..close_pos].trim();
                parse_inline_pairs(inner, element, &mut params);
            } else {
                // Multi-line block
                if !after_brace.is_empty() {
                    // There may be a key-value pair on the same line as the opening brace
                    parse_single_pair(after_brace, element, &mut params);
                }
                parse_element_block(&mut lines, element, &mut params);
            }
        } else {
            // Single line: skinparam key value
            // Could be "skinparam elementKey value" or "skinparam element.Key value"
            parse_single_pair(after, "", &mut params);
        }
    }

    // Extract document-level styles from <style> CSS blocks.
    // Java: `document { BackGroundColor orange }` sets the SVG background.
    if !style_content.is_empty() {
        extract_document_style(&style_content, &mut params);
    }

    debug!("parsed {} skinparams", params.len());
    params
}

/// Extract document-level CSS properties from `<style>` content.
/// Supports `document { BackGroundColor orange }` and nested sub-blocks like
/// `document { title { BackGroundColor yellow } footer { FontColor red } }`.
fn extract_document_style(css: &str, params: &mut SkinParams) {
    let mut depth = 0;
    let mut in_document = false;
    let mut doc_depth = 0;
    // Track which sub-block we're inside (e.g., "title", "footer", "header", "legend", "caption")
    let mut current_section: Option<String> = None;
    let mut section_depth = 0;

    for line in css.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let opens = trimmed.matches('{').count();
        let closes = trimmed.matches('}').count();

        if !in_document {
            let lower = trimmed.to_lowercase();
            if lower.starts_with("document") && trimmed.contains('{') {
                in_document = true;
                doc_depth = depth;
                depth += opens;
                depth = depth.saturating_sub(closes);
                continue;
            }

            // Top-level section blocks — not inside document {}
            if current_section.is_none() && depth == 0 && trimmed.contains('{') {
                let brace_pos = trimmed.find('{').unwrap();
                let name = trimmed[..brace_pos].trim().to_lowercase();
                if matches!(
                    name.as_str(),
                    "title" | "footer" | "header" | "legend" | "caption"
                ) {
                    current_section = Some(name);
                    section_depth = depth;
                    depth += opens;
                    depth = depth.saturating_sub(closes);
                    continue;
                }
                // Element-level blocks (node, root, etc.) — store under "element.xxx"
                if matches!(
                    name.as_str(),
                    "node"
                        | "root"
                        | "arrow"
                        | "group"
                        | "separator"
                        | "mindmapdiagram"
                        | "wbsdiagram"
                        | "element"
                        | "component"
                        | "participant"
                        | "actor"
                        | "boundary"
                        | "control"
                        | "entity"
                        | "database"
                        | "collections"
                        | "queue"
                        | "note"
                        | "package"
                        | "rectangle"
                        | "card"
                        | "cloud"
                        | "frame"
                        | "folder"
                        | "interface"
                        | "abstract"
                        | "class"
                        | "enum"
                        | "state"
                        | "usecase"
                        | "activity"
                        | "diamond"
                ) {
                    current_section = Some(name);
                    section_depth = depth;
                    depth += opens;
                    depth = depth.saturating_sub(closes);
                    continue;
                }
            }
        }

        // Extract properties from top-level section blocks (not inside document).
        // Only pick up direct properties (depth == section_depth + 1), not nested
        // sub-selectors like `.highlight { BackGroundColor ... }`.
        if !in_document && current_section.is_some() && depth == section_depth + 1 {
            if !trimmed.contains('{') && !trimmed.starts_with('}') {
                let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
                if parts.len() == 2 {
                    let section = current_section.as_ref().unwrap();
                    let key = parts[0].trim().to_lowercase();
                    let value = parts[1].trim();
                    // Document sub-sections (title, footer, etc.) use document.{section}.{key}
                    // Element-level blocks (node, root, etc.) use {section}.{key}
                    let param_key = if matches!(
                        section.as_str(),
                        "title" | "footer" | "header" | "legend" | "caption"
                    ) {
                        format!("document.{section}.{key}")
                    } else {
                        format!("{section}.{key}")
                    };
                    params.set(&param_key, value);
                    log::debug!("extracted style {param_key}: {value}");
                }
            }
        }

        if in_document && depth > doc_depth {
            // Check for sub-block opening (title {, footer {, etc.)
            if current_section.is_none() && depth == doc_depth + 1 && trimmed.contains('{') {
                let brace_pos = trimmed.find('{').unwrap();
                let name = trimmed[..brace_pos].trim().to_lowercase();
                if matches!(
                    name.as_str(),
                    "title" | "footer" | "header" | "legend" | "caption"
                ) {
                    current_section = Some(name);
                    section_depth = depth;
                    depth += opens;
                    depth = depth.saturating_sub(closes);
                    continue;
                }
            }

            // Inside a sub-block: extract properties
            if let Some(ref section) = current_section {
                if depth > section_depth && !trimmed.contains('{') && !trimmed.starts_with('}') {
                    let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim().to_lowercase();
                        let value = parts[1].trim();
                        let param_key = format!("document.{section}.{key}");
                        params.set(&param_key, value);
                        log::debug!("extracted document.{section}.{key}: {value}");
                    }
                }
            }

            // Direct document-level properties (not in sub-block)
            if current_section.is_none()
                && depth == doc_depth + 1
                && !trimmed.contains('{')
                && !trimmed.starts_with('}')
            {
                let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
                if parts.len() == 2 {
                    let key = parts[0].trim().to_lowercase();
                    let value = parts[1].trim();
                    if key == "backgroundcolor" {
                        // Store under document-specific key so it doesn't override
                        // entity fill colors via the generic fallback chain.
                        params.set("document.backgroundcolor", value);
                        log::debug!("extracted document BackGroundColor: {value}");
                    }
                }
            }
        }

        depth += opens;
        depth = depth.saturating_sub(closes);

        // Check if we're closing the current section
        if current_section.is_some() && depth <= section_depth + 1 {
            // Count closes that happen on this line after the section depth
            if closes > 0 && depth <= section_depth + 1 {
                current_section = None;
            }
        }

        if in_document && depth <= doc_depth {
            in_document = false;
            current_section = None;
        }
    }
}

/// Parse a nested skinparam block (after `skinparam {`).
/// Handles both global key-value pairs and element sub-blocks.
fn parse_nested_block<'a, I>(lines: &mut I, _prefix: &str, params: &mut SkinParams)
where
    I: Iterator<Item = &'a str>,
{
    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed == "}" {
            return;
        }

        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        if trimmed.contains('{') {
            let brace_pos = trimmed.find('{').unwrap();
            let element = trimmed[..brace_pos].trim();
            let after_brace = trimmed[brace_pos + 1..].trim();

            if let Some(close_pos) = after_brace.find('}') {
                let inner = after_brace[..close_pos].trim();
                parse_inline_pairs(inner, element, params);
            } else {
                if !after_brace.is_empty() {
                    parse_single_pair(after_brace, element, params);
                }
                parse_element_block(lines, element, params);
            }
        } else {
            parse_single_pair(trimmed, "", params);
        }
    }
}

/// Parse a multi-line element block (lines after `skinparam element {`).
fn parse_element_block<'a, I>(lines: &mut I, element: &str, params: &mut SkinParams)
where
    I: Iterator<Item = &'a str>,
{
    for line in lines.by_ref() {
        let trimmed = line.trim();

        if trimmed == "}" || trimmed.starts_with('}') {
            return;
        }

        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        parse_single_pair(trimmed, element, params);
    }
}

/// Parse space-separated key-value pairs from an inline block.
fn parse_inline_pairs(content: &str, element: &str, params: &mut SkinParams) {
    // Simple approach: split by whitespace, take pairs
    let tokens: Vec<&str> = content.split_whitespace().collect();
    let mut i = 0;
    while i + 1 < tokens.len() {
        let key = tokens[i];
        let value = tokens[i + 1];
        let full_key = if element.is_empty() {
            key.to_string()
        } else {
            format!("{element}.{key}")
        };
        debug!("skinparam: {full_key} = {value}");
        params.set(&full_key, value);
        i += 2;
    }
}

/// Parse a single "key value" pair line with an optional element prefix.
fn parse_single_pair(content: &str, element: &str, params: &mut SkinParams) {
    let parts: Vec<&str> = content.splitn(2, char::is_whitespace).collect();
    if parts.len() == 2 {
        let key = parts[0].trim();
        let value = parts[1].trim();
        let full_key = if element.is_empty() {
            key.to_string()
        } else {
            format!("{element}.{key}")
        };
        debug!("skinparam: {full_key} = {value}");
        params.set(&full_key, value);
    }
}

/// Normalize a color value for SVG output.
///
/// - `#RGB` (3-char hex) -> `#RRGGBB`
/// - `#RRGGBB` -> as-is
/// - `#AARRGGBB` (8-char hex with alpha) -> `#RRGGBB` (alpha dropped for SVG)
/// - `transparent` -> `none`
/// - Named colors (e.g., `red`, `LightBlue`) -> pass through (SVG supports them)
pub fn normalize_color(color: &str) -> String {
    let trimmed = color.trim();

    // Handle "transparent"
    if trimmed.eq_ignore_ascii_case("transparent") {
        return "none".to_string();
    }

    // Handle hex colors — Java normalizes to uppercase #RRGGBB
    if let Some(hex) = trimmed.strip_prefix('#') {
        let hex_clean: String = hex
            .chars()
            .filter(char::is_ascii_hexdigit)
            .map(|c| c.to_ascii_uppercase())
            .collect();

        return match hex_clean.len() {
            3 => {
                // #RGB -> #RRGGBB (uppercase)
                let mut expanded = String::with_capacity(7);
                expanded.push('#');
                for c in hex_clean.chars() {
                    expanded.push(c);
                    expanded.push(c);
                }
                expanded
            }
            6 => {
                format!("#{hex_clean}")
            }
            8 => {
                // #AARRGGBB -> #RRGGBB (drop alpha, uppercase)
                format!("#{}", &hex_clean[2..])
            }
            _ => trimmed.to_string(),
        };
    }

    // Named colors: convert to hex (#RRGGBB) to match Java PlantUML output.
    if let Some(hex) = named_color_to_hex(trimmed) {
        return hex.to_string();
    }

    // Bare hex without '#' prefix (e.g. "22A722" from parser)
    let all_hex = trimmed.len() == 6
        && trimmed.chars().all(|c| c.is_ascii_hexdigit());
    if all_hex {
        return format!("#{}", trimmed.to_ascii_uppercase());
    }

    trimmed.to_string()
}

/// Convert a named CSS/SVG color to its hex equivalent.
/// Java PlantUML always renders colors as hex codes.
fn named_color_to_hex(name: &str) -> Option<&'static str> {
    let lower: String = name.to_lowercase();
    match lower.as_str() {
        "black" => Some("#000000"),
        "white" => Some("#FFFFFF"),
        "red" => Some("#FF0000"),
        "green" => Some("#008000"),
        "blue" => Some("#0000FF"),
        "yellow" => Some("#FFFF00"),
        "cyan" | "aqua" => Some("#00FFFF"),
        "magenta" | "fuchsia" => Some("#FF00FF"),
        "gray" | "grey" => Some("#808080"),
        "darkgray" | "darkgrey" => Some("#A9A9A9"),
        "lightgray" | "lightgrey" => Some("#D3D3D3"),
        "orange" => Some("#FFA500"),
        "pink" => Some("#FFC0CB"),
        "purple" => Some("#800080"),
        "brown" => Some("#A52A2A"),
        "navy" => Some("#000080"),
        "teal" => Some("#008080"),
        "olive" => Some("#808000"),
        "maroon" => Some("#800000"),
        "lime" => Some("#00FF00"),
        "silver" => Some("#C0C0C0"),
        "gold" => Some("#FFD700"),
        "indigo" => Some("#4B0082"),
        "violet" => Some("#EE82EE"),
        "coral" => Some("#FF7F50"),
        "salmon" => Some("#FA8072"),
        "tomato" => Some("#FF6347"),
        "orangered" => Some("#FF4500"),
        "crimson" => Some("#DC143C"),
        "darkblue" => Some("#00008B"),
        "darkgreen" => Some("#006400"),
        "darkred" => Some("#8B0000"),
        "lightblue" => Some("#ADD8E6"),
        "lightgreen" => Some("#90EE90"),
        "lightyellow" => Some("#FFFFE0"),
        "skyblue" => Some("#87CEEB"),
        "steelblue" => Some("#4682B4"),
        "royalblue" => Some("#4169E1"),
        "forestgreen" => Some("#228B22"),
        "seagreen" => Some("#2E8B57"),
        "limegreen" => Some("#32CD32"),
        "chocolate" => Some("#D2691E"),
        "sienna" => Some("#A0522D"),
        "tan" => Some("#D2B48C"),
        "wheat" => Some("#F5DEB3"),
        "khaki" => Some("#F0E68C"),
        "plum" => Some("#DDA0DD"),
        "orchid" => Some("#DA70D6"),
        "turquoise" => Some("#40E0D0"),
        "slategray" | "slategrey" => Some("#708090"),
        "dimgray" | "dimgrey" => Some("#696969"),
        "ivory" => Some("#FFFFF0"),
        "beige" => Some("#F5F5DC"),
        "linen" => Some("#FAF0E6"),
        "honeydew" => Some("#F0FFF0"),
        "mintcream" => Some("#F5FFFA"),
        "lavender" => Some("#E6E6FA"),
        "mistyrose" => Some("#FFE4E1"),
        "cornsilk" => Some("#FFF8DC"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Color normalization tests ────────────────────────────────────

    #[test]
    fn normalize_hex6_passthrough() {
        assert_eq!(normalize_color("#FEFECE"), "#FEFECE");
    }

    #[test]
    fn normalize_hex3_expand() {
        assert_eq!(normalize_color("#F0C"), "#FF00CC");
    }

    #[test]
    fn normalize_hex8_drop_alpha() {
        assert_eq!(normalize_color("#80FF0000"), "#FF0000");
    }

    #[test]
    fn normalize_transparent() {
        assert_eq!(normalize_color("transparent"), "none");
    }

    #[test]
    fn normalize_transparent_case_insensitive() {
        assert_eq!(normalize_color("Transparent"), "none");
        assert_eq!(normalize_color("TRANSPARENT"), "none");
    }

    #[test]
    fn normalize_named_color_to_hex() {
        assert_eq!(normalize_color("red"), "#FF0000");
        assert_eq!(normalize_color("LightBlue"), "#ADD8E6");
        assert_eq!(normalize_color("DarkGreen"), "#006400");
    }

    #[test]
    fn normalize_whitespace_trimmed() {
        assert_eq!(normalize_color("  #FFF  "), "#FFFFFF");
        assert_eq!(normalize_color("  red  "), "#FF0000");
    }

    // ── Skinparam parsing tests ────────────────────────────────────

    #[test]
    fn parse_single_line_skinparam() {
        let src = "skinparam BackgroundColor #FEFECE\nclass Foo";
        let params = parse_skinparams(src);
        assert_eq!(params.get("backgroundcolor"), Some("#FEFECE"));
    }

    #[test]
    fn parse_single_line_element_skinparam() {
        let src = "skinparam ClassBackgroundColor #FEFECE";
        let params = parse_skinparams(src);
        assert_eq!(params.get("classbackgroundcolor"), Some("#FEFECE"));
    }

    #[test]
    fn parse_element_block() {
        let src = "skinparam class {\n  BackgroundColor #FEFECE\n  BorderColor #A80036\n}";
        let params = parse_skinparams(src);
        assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
        assert_eq!(params.get("class.bordercolor"), Some("#A80036"));
    }

    #[test]
    fn parse_nested_block() {
        let src = "skinparam {\n  class {\n    BackgroundColor #FEFECE\n  }\n}";
        let params = parse_skinparams(src);
        assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
    }

    #[test]
    fn parse_nested_block_with_global_params() {
        let src = "skinparam {\n  BackgroundColor #FFFFFF\n  class {\n    BackgroundColor #FEFECE\n  }\n}";
        let params = parse_skinparams(src);
        assert_eq!(params.get("backgroundcolor"), Some("#FFFFFF"));
        assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
    }

    #[test]
    fn parse_multiple_skinparam_lines() {
        let src =
            "skinparam BackgroundColor #FEFECE\nskinparam ArrowColor #A80036\nskinparam FontColor black";
        let params = parse_skinparams(src);
        assert_eq!(params.get("backgroundcolor"), Some("#FEFECE"));
        assert_eq!(params.get("arrowcolor"), Some("#A80036"));
        assert_eq!(params.get("fontcolor"), Some("#000000"));
    }

    #[test]
    fn parse_skinparam_case_insensitive_lookup() {
        let src = "skinparam ClassBackgroundColor #FEFECE";
        let params = parse_skinparams(src);
        assert_eq!(params.get("ClassBackgroundColor"), Some("#FEFECE"));
        assert_eq!(params.get("classbackgroundcolor"), Some("#FEFECE"));
        assert_eq!(params.get("CLASSBACKGROUNDCOLOR"), Some("#FEFECE"));
    }

    #[test]
    fn parse_skinparam_ignores_non_skinparam_lines() {
        let src = "class Foo\ninterface Bar\nskinparam ArrowColor red\nFoo --> Bar";
        let params = parse_skinparams(src);
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("arrowcolor"), Some("#FF0000"));
    }

    #[test]
    fn parse_skinparam_skips_style_blocks() {
        let src = "<style>\nskinparam Foo bar\n</style>\nskinparam ArrowColor red";
        let params = parse_skinparams(src);
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("arrowcolor"), Some("#FF0000"));
    }

    #[test]
    fn parse_skinparam_color_normalization() {
        let src = "skinparam BackgroundColor transparent\nskinparam BorderColor #F00";
        let params = parse_skinparams(src);
        assert_eq!(params.get("backgroundcolor"), Some("none"));
        assert_eq!(params.get("bordercolor"), Some("#FF0000"));
    }

    #[test]
    fn parse_skinparam_inline_block() {
        let src = "skinparam class { BackgroundColor #FEFECE BorderColor #A80036 }";
        let params = parse_skinparams(src);
        assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
        assert_eq!(params.get("class.bordercolor"), Some("#A80036"));
    }

    #[test]
    fn parse_empty_source() {
        let params = parse_skinparams("");
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn parse_skinparam_with_comments() {
        let src = "skinparam class {\n  ' this is a comment\n  BackgroundColor #FEFECE\n}";
        let params = parse_skinparams(src);
        assert_eq!(params.get("class.backgroundcolor"), Some("#FEFECE"));
        assert_eq!(params.len(), 1);
    }

    // ── Convenience method tests ────────────────────────────────────

    #[test]
    fn background_color_element_key() {
        let src = "skinparam ClassBackgroundColor #FEFECE";
        let params = parse_skinparams(src);
        assert_eq!(params.background_color("class", "#default"), "#FEFECE");
    }

    #[test]
    fn background_color_dot_key() {
        let src = "skinparam class {\n  BackgroundColor #AABB00\n}";
        let params = parse_skinparams(src);
        assert_eq!(params.background_color("class", "#default"), "#AABB00");
    }

    #[test]
    fn background_color_global_does_not_cascade() {
        // In Java PlantUML, global `skinparam backgroundColor` only affects
        // the diagram canvas, NOT element fills.  Elements use their own
        // defaults (theme or hardcoded).
        let src = "skinparam BackgroundColor #FFFFFF";
        let params = parse_skinparams(src);
        // Should return theme default #F1F1F1 for class, not global #FFFFFF
        assert_eq!(params.background_color("class", "#default"), "#F1F1F1");
    }

    #[test]
    fn background_color_default_fallback() {
        let params = SkinParams::new();
        assert_eq!(params.background_color("class", "#FEFECE"), "#F1F1F1");
    }

    #[test]
    fn font_color_lookup_chain() {
        let src = "skinparam class {\n  FontColor #333333\n}";
        let params = parse_skinparams(src);
        assert_eq!(params.font_color("class", "#000000"), "#333333");
    }

    #[test]
    fn border_color_lookup_chain() {
        let src = "skinparam ClassBorderColor #A80036";
        let params = parse_skinparams(src);
        assert_eq!(params.border_color("class", "#000000"), "#A80036");
    }

    #[test]
    fn arrow_color_lookup() {
        let src = "skinparam ArrowColor blue";
        let params = parse_skinparams(src);
        assert_eq!(params.arrow_color("#A80036"), "#0000FF");
    }

    #[test]
    fn arrow_color_default() {
        let params = SkinParams::new();
        assert_eq!(params.arrow_color("#A80036"), "#A80036");
    }

    #[test]
    fn get_or_returns_default_when_missing() {
        let params = SkinParams::new();
        assert_eq!(params.get_or("nonexistent", "fallback"), "fallback");
    }

    #[test]
    fn get_or_returns_value_when_present() {
        let src = "skinparam Foo bar";
        let params = parse_skinparams(src);
        assert_eq!(params.get_or("foo", "fallback"), "bar");
    }

    // ── Theme tests ───────────────────────────────────────────────────

    #[test]
    fn theme_rose_global_colors() {
        let t = Theme::rose();
        assert_eq!(t.background_color, "#FFFFFF");
        assert_eq!(t.font_color, "#000000");
        assert_eq!(t.arrow_color, "#181818");
        assert_eq!(t.border_color, "#181818");
    }

    #[test]
    fn theme_rose_class_colors() {
        let t = Theme::rose();
        assert_eq!(t.class_bg, "#F1F1F1");
        assert_eq!(t.class_border, "#181818");
        assert_eq!(t.class_font, "#000000");
    }

    #[test]
    fn theme_rose_sequence_colors() {
        let t = Theme::rose();
        assert_eq!(t.participant_bg, "#E2E2F0");
        assert_eq!(t.participant_border, "#181818");
        assert_eq!(t.lifeline_color, "#181818");
        assert_eq!(t.activation_bg, "#F1F1F1");
        assert_eq!(t.activation_border, "#181818");
        assert_eq!(t.group_bg, "#EEEEEE");
        assert_eq!(t.group_border, "#000000");
    }

    #[test]
    fn theme_rose_note_colors() {
        let t = Theme::rose();
        assert_eq!(t.note_bg, "#FEFFDD");
        assert_eq!(t.note_border, "#181818");
    }

    #[test]
    fn theme_rose_activity_colors() {
        let t = Theme::rose();
        assert_eq!(t.activity_bg, "#F1F1F1");
        assert_eq!(t.activity_border, "#181818");
        assert_eq!(t.diamond_bg, "#F1F1F1");
        assert_eq!(t.diamond_border, "#181818");
        assert_eq!(t.swimlane_border, "#181818");
        assert_eq!(t.swimlane_header_bg, "#F1F1F1");
    }

    #[test]
    fn theme_rose_state_colors() {
        let t = Theme::rose();
        assert_eq!(t.state_bg, "#F1F1F1");
        assert_eq!(t.state_border, "#181818");
        assert_eq!(t.composite_bg, "#F1F1F1");
        assert_eq!(t.composite_border, "#181818");
    }

    #[test]
    fn theme_rose_component_colors() {
        let t = Theme::rose();
        assert_eq!(t.component_bg, "#F1F1F1");
        assert_eq!(t.component_border, "#181818");
        assert_eq!(t.node_bg, "#F1F1F1");
        assert_eq!(t.node_border, "#181818");
        assert_eq!(t.database_bg, "#F1F1F1");
        assert_eq!(t.database_border, "#181818");
        assert_eq!(t.cloud_bg, "#F1F1F1");
        assert_eq!(t.cloud_border, "#181818");
    }

    #[test]
    fn theme_rose_erd_colors() {
        let t = Theme::rose();
        assert_eq!(t.entity_bg, "#F1F1F1");
        assert_eq!(t.entity_border, "#181818");
        assert_eq!(t.relationship_bg, "#F1F1F1");
        assert_eq!(t.relationship_border, "#181818");
    }

    #[test]
    fn theme_rose_mindmap_wbs_colors() {
        let t = Theme::rose();
        assert_eq!(t.mindmap_node_bg, "#F1F1F1");
        assert_eq!(t.mindmap_node_border, "#181818");
        assert_eq!(t.wbs_root_bg, "#FFD700");
    }

    #[test]
    fn theme_rose_legend_colors() {
        let t = Theme::rose();
        assert_eq!(t.legend_bg, "#FEFFDD");
        assert_eq!(t.legend_border, "#000000");
    }

    #[test]
    fn theme_default_is_rose() {
        let def = Theme::default();
        let rose = Theme::rose();
        assert_eq!(def.background_color, rose.background_color);
        assert_eq!(def.class_bg, rose.class_bg);
        assert_eq!(def.arrow_color, rose.arrow_color);
        assert_eq!(def.note_bg, rose.note_bg);
        assert_eq!(def.entity_bg, rose.entity_bg);
        assert_eq!(def.wbs_root_bg, rose.wbs_root_bg);
    }

    // ── SkinParams + Theme integration tests ──────────────────────────

    #[test]
    fn skinparams_default_has_rose_theme() {
        let sp = SkinParams::default();
        assert_eq!(sp.theme.class_bg, "#F1F1F1");
        assert_eq!(sp.theme.arrow_color, "#181818");
    }

    #[test]
    fn skinparams_theme_fallback_bg() {
        let sp = SkinParams::new();
        // No explicit skinparam set: should fall back to theme for known elements
        assert_eq!(sp.background_color("class", "#IGNORED"), "#F1F1F1");
        assert_eq!(sp.background_color("component", "#IGNORED"), "#F1F1F1");
        assert_eq!(sp.background_color("entity", "#IGNORED"), "#F1F1F1");
        assert_eq!(sp.background_color("note", "#IGNORED"), "#FEFFDD");
        assert_eq!(sp.background_color("cloud", "#IGNORED"), "#F1F1F1");
    }

    #[test]
    fn skinparams_theme_fallback_border() {
        let sp = SkinParams::new();
        assert_eq!(sp.border_color("class", "#IGNORED"), "#181818");
        assert_eq!(sp.border_color("state", "#IGNORED"), "#181818");
        assert_eq!(sp.border_color("note", "#IGNORED"), "#181818");
    }

    #[test]
    fn skinparams_theme_fallback_font() {
        let sp = SkinParams::new();
        assert_eq!(sp.font_color("class", "#IGNORED"), "#000000");
        assert_eq!(sp.font_color("participant", "#IGNORED"), "#000000");
    }

    #[test]
    fn skinparams_explicit_overrides_theme() {
        let src = "skinparam ClassBackgroundColor #112233";
        let sp = parse_skinparams(src);
        // Explicit skinparam should win over theme
        assert_eq!(sp.background_color("class", "#IGNORED"), "#112233");
    }

    #[test]
    fn skinparams_global_does_not_override_theme() {
        let src = "skinparam BackgroundColor #AABBCC";
        let sp = parse_skinparams(src);
        // Global backgroundColor does not cascade to element fills
        assert_eq!(sp.background_color("class", "#IGNORED"), "#F1F1F1");
    }

    #[test]
    fn skinparams_root_style_cascades_to_element_colors() {
        let src = "<style>\nroot {\n  BackgroundColor #ABCDEF\n  FontColor #654321\n  LineColor #123456\n}\n</style>";
        let sp = parse_skinparams(src);
        assert_eq!(sp.background_color("participant", "#IGNORED"), "#ABCDEF");
        assert_eq!(sp.border_color("participant", "#IGNORED"), "#123456");
        assert_eq!(sp.font_color("participant", "#IGNORED"), "#654321");
    }

    #[test]
    fn theme_plain_cascades_root_background_to_sequence_participants() {
        let src = "@startuml\n!theme plain\nactor Alice\nparticipant Bob\nAlice -> Bob\n@enduml";
        let preprocessed = crate::preproc::preprocess(src).expect("theme preprocess");
        let sp = parse_skinparams(&preprocessed);
        assert_eq!(sp.background_color("participant", "#IGNORED"), "#FFFFFF");
        assert_eq!(sp.border_color("participant", "#IGNORED"), "#000000");
    }

    #[test]
    fn skinparams_unknown_element_uses_caller_default() {
        let sp = SkinParams::new();
        // Unknown element has no theme mapping, so caller default is returned
        assert_eq!(sp.background_color("unknownelement", "#CALLER"), "#CALLER");
    }

    // ── New skinparam methods tests ────────────────────────────────────

    #[test]
    fn default_font_name_none_when_unset() {
        let sp = SkinParams::new();
        assert_eq!(sp.default_font_name(), None);
    }

    #[test]
    fn default_font_name_returns_value() {
        let src = "skinparam defaultFontName Arial";
        let sp = parse_skinparams(src);
        assert_eq!(sp.default_font_name(), Some("Arial"));
    }

    #[test]
    fn default_font_size_none_when_unset() {
        let sp = SkinParams::new();
        assert_eq!(sp.default_font_size(), None);
    }

    #[test]
    fn default_font_size_returns_value() {
        let src = "skinparam defaultFontSize 14";
        let sp = parse_skinparams(src);
        assert_eq!(sp.default_font_size(), Some(14.0));
    }

    #[test]
    fn monochrome_false_by_default() {
        let sp = SkinParams::new();
        assert!(!sp.is_monochrome());
    }

    #[test]
    fn monochrome_true_when_set() {
        let src = "skinparam monochrome true";
        let sp = parse_skinparams(src);
        assert!(sp.is_monochrome());
    }

    #[test]
    fn monochrome_false_when_explicit() {
        let src = "skinparam monochrome false";
        let sp = parse_skinparams(src);
        assert!(!sp.is_monochrome());
    }

    #[test]
    fn handwritten_false_by_default() {
        let sp = SkinParams::new();
        assert!(!sp.is_handwritten());
    }

    #[test]
    fn handwritten_true_when_set() {
        let src = "skinparam handwritten true";
        let sp = parse_skinparams(src);
        assert!(sp.is_handwritten());
    }

    #[test]
    fn round_corner_none_when_unset() {
        let sp = SkinParams::new();
        assert_eq!(sp.round_corner(), None);
    }

    #[test]
    fn round_corner_returns_value() {
        let src = "skinparam roundcorner 15";
        let sp = parse_skinparams(src);
        assert_eq!(sp.round_corner(), Some(15.0));
    }

    #[test]
    fn font_size_element_key() {
        let src = "skinparam classFontSize 16";
        let sp = parse_skinparams(src);
        assert_eq!(sp.font_size("class", 12.0), 16.0);
    }

    #[test]
    fn font_size_default_fallback() {
        let src = "skinparam defaultFontSize 14";
        let sp = parse_skinparams(src);
        assert_eq!(sp.font_size("class", 12.0), 14.0);
    }

    #[test]
    fn font_size_caller_default() {
        let sp = SkinParams::new();
        assert_eq!(sp.font_size("class", 12.0), 12.0);
    }

    #[test]
    fn sequence_arrow_thickness_none_when_unset() {
        let sp = SkinParams::new();
        assert_eq!(sp.sequence_arrow_thickness(), None);
    }

    #[test]
    fn sequence_arrow_thickness_returns_value() {
        let src = "skinparam sequenceArrowThickness 2";
        let sp = parse_skinparams(src);
        assert_eq!(sp.sequence_arrow_thickness(), Some(2.0));
    }

    #[test]
    fn sequence_arrow_color_returns_value() {
        let src = "skinparam sequenceArrowColor DarkBlue";
        let sp = parse_skinparams(src);
        assert_eq!(sp.sequence_arrow_color("#A80036"), "#00008B");
    }

    #[test]
    fn sequence_arrow_color_fallback() {
        let sp = SkinParams::new();
        assert_eq!(sp.sequence_arrow_color("#A80036"), "#A80036");
    }

    #[test]
    fn sequence_lifeline_border_color_returns_value() {
        let src = "skinparam sequenceLifeLineBorderColor blue";
        let sp = parse_skinparams(src);
        assert_eq!(sp.sequence_lifeline_border_color("#A80036"), "#0000FF");
    }

    #[test]
    fn effective_font_family_default() {
        let sp = SkinParams::new();
        assert_eq!(sp.effective_font_family("monospace"), "monospace");
    }

    #[test]
    fn effective_font_family_override() {
        let src = "skinparam defaultFontName Arial";
        let sp = parse_skinparams(src);
        assert_eq!(sp.effective_font_family("monospace"), "Arial");
    }

    #[test]
    fn handwritten_font_family_none_when_disabled() {
        let sp = SkinParams::new();
        assert_eq!(sp.handwritten_font_family(), None);
    }

    #[test]
    fn handwritten_font_family_returns_cursive() {
        let src = "skinparam handwritten true";
        let sp = parse_skinparams(src);
        assert!(sp.handwritten_font_family().is_some());
        assert!(sp.handwritten_font_family().unwrap().contains("cursive"));
    }

    // ══════════════════════════════════════════════════════════════════
    // Tests ported from upstream PlantUML Java project
    // ══════════════════════════════════════════════════════════════════

    // ── Ported from upstream: StringTrieTest ─────────────────────────

    // Ported from upstream: StringTrieTest.testPutAndGetSimple
    #[test]
    fn upstream_trie_put_and_get_simple() {
        let mut sp = SkinParams::new();
        sp.set("foo", "123");
        assert_eq!(sp.get("foo"), Some("123"));
        assert_eq!(sp.get("bar"), None);
    }

    // Ported from upstream: StringTrieTest.testCaseInsensitivity
    #[test]
    fn upstream_trie_case_insensitivity() {
        let mut sp = SkinParams::new();
        sp.set("Hello", "world");
        assert_eq!(sp.get("hello"), Some("world"));
        assert_eq!(sp.get("HELLO"), Some("world"));
        assert_eq!(sp.get("HeLlO"), Some("world"));
    }

    // Ported from upstream: StringTrieTest.testOverwriteValue
    #[test]
    fn upstream_trie_overwrite_value() {
        let mut sp = SkinParams::new();
        sp.set("key", "1");
        sp.set("key", "2");
        assert_eq!(sp.get("KEY"), Some("2"));
    }

    // Ported from upstream: StringTrieTest.testPrefixCollision
    #[test]
    fn upstream_trie_prefix_collision() {
        let mut sp = SkinParams::new();
        sp.set("abc", "10");
        sp.set("abcd", "20");
        assert_eq!(sp.get("ABC"), Some("10"));
        assert_eq!(sp.get("ABCD"), Some("20"));
        assert_eq!(sp.get("ab"), None);
    }

    // Ported from upstream: StringTrieTest.testEmptyStringKey
    #[test]
    fn upstream_trie_empty_string_key() {
        let mut sp = SkinParams::new();
        sp.set("", "empty");
        assert_eq!(sp.get(""), Some("empty"));
    }

    // ── Ported from upstream: ColorTrieNodeTest ──────────────────────

    // Ported from upstream: ColorTrieNodeTest.testInvalidCharacterIgnoredOnPut
    #[test]
    fn upstream_color_normalize_named_darkblue() {
        assert_eq!(normalize_color("darkblue"), "#00008B");
    }

    // ── Ported from upstream: ColorHSBTest — hex color normalization ─

    // Ported from upstream: ColorHSBTest.test_toString — ARGB alpha stripping
    #[test]
    fn upstream_color_normalize_hex_8digit_alpha_red() {
        assert_eq!(normalize_color("#AAFF0000"), "#FF0000");
    }

    #[test]
    fn upstream_color_normalize_hex_8digit_alpha_green() {
        assert_eq!(normalize_color("#AA00FF00"), "#00FF00");
    }

    #[test]
    fn upstream_color_normalize_hex_8digit_alpha_blue() {
        assert_eq!(normalize_color("#AA0000FF"), "#0000FF");
    }

    #[test]
    fn upstream_color_normalize_hex_8digit_half_saturated() {
        assert_eq!(normalize_color("#FFFF8080"), "#FF8080");
    }

    #[test]
    fn upstream_color_normalize_hex_8digit_half_brightness() {
        assert_eq!(normalize_color("#FF7F0000"), "#7F0000");
    }

    // ── Ported from upstream: StyleFontWeightTest (skinparam storage) ─

    // Ported from upstream: StyleFontWeightTest — block with multiple properties
    #[test]
    fn upstream_skinparam_block_multiple_properties() {
        let src = "\
skinparam participant {
  FontName Roboto
  FontColor green
  FontSize 26
  LineColor #EE0000
}";
        let sp = parse_skinparams(src);
        assert_eq!(sp.get("participant.fontname"), Some("Roboto"));
        assert_eq!(sp.get("participant.fontcolor"), Some("#008000"));
        assert_eq!(sp.get("participant.fontsize"), Some("26"));
        assert_eq!(sp.get("participant.linecolor"), Some("#EE0000"));
    }

    // ── Ported from upstream: ValueImplFontFaceTest (property storage) ─

    // Ported from upstream: ValueImplFontFaceTest.numericWeight100
    #[test]
    fn upstream_font_weight_numeric_stored_raw() {
        let src = "skinparam participant {\n  FontWeight 100\n}";
        let sp = parse_skinparams(src);
        assert_eq!(sp.get("participant.fontweight"), Some("100"));
    }

    // Ported from upstream: ValueImplFontFaceTest.numericWeight900
    #[test]
    fn upstream_font_weight_900_stored() {
        let src = "skinparam participant {\n  FontWeight 900\n}";
        let sp = parse_skinparams(src);
        assert_eq!(sp.get("participant.fontweight"), Some("900"));
    }

    // Ported from upstream: ValueImplFontFaceTest.boldKeyword
    #[test]
    fn upstream_font_style_bold_stored() {
        let src = "skinparam participant {\n  FontStyle bold\n}";
        let sp = parse_skinparams(src);
        assert_eq!(sp.get("participant.fontstyle"), Some("bold"));
    }

    // Ported from upstream: ValueImplFontFaceTest.italicKeyword
    #[test]
    fn upstream_font_style_italic_stored() {
        let src = "skinparam participant {\n  FontStyle italic\n}";
        let sp = parse_skinparams(src);
        assert_eq!(sp.get("participant.fontstyle"), Some("italic"));
    }

    // ── Ported from upstream: StyleFontWeightTest — independent axes ─

    // Ported from upstream: StyleFontWeightTest.fontWeight900AndItalicAreBothPreserved
    #[test]
    fn upstream_font_weight_and_style_independent() {
        let src = "skinparam participant {\n  FontWeight 900\n  FontStyle italic\n  FontSize 26\n}";
        let sp = parse_skinparams(src);
        assert_eq!(sp.get("participant.fontweight"), Some("900"));
        assert_eq!(sp.get("participant.fontstyle"), Some("italic"));
        assert_eq!(sp.font_size("participant", 12.0), 26.0);
    }

    // ── Ported from upstream: resolution chain tests ─────────────────

    // Ported from upstream: style resolution chain for background color
    #[test]
    fn upstream_background_color_resolution_chain() {
        // Level 1: element-specific key wins
        let src1 = "skinparam ComponentBackgroundColor #111111\nskinparam BackgroundColor #222222";
        let sp1 = parse_skinparams(src1);
        assert_eq!(sp1.background_color("component", "#default"), "#111111");

        // Level 2: global backgroundColor does NOT cascade to elements
        let src2 = "skinparam BackgroundColor #222222";
        let sp2 = parse_skinparams(src2);
        assert_eq!(sp2.background_color("component", "#default"), "#F1F1F1");

        // Level 3: theme fallback when nothing is set
        let sp3 = SkinParams::new();
        assert_eq!(sp3.background_color("component", "#default"), "#F1F1F1");
    }

    // Ported from upstream: font color resolution chain
    #[test]
    fn upstream_font_color_resolution_chain() {
        let src = "skinparam ClassFontColor #AA0000\nskinparam FontColor #BB0000";
        let sp = parse_skinparams(src);
        assert_eq!(sp.font_color("class", "#000000"), "#AA0000");

        let src2 = "skinparam FontColor #BB0000";
        let sp2 = parse_skinparams(src2);
        assert_eq!(sp2.font_color("class", "#000000"), "#BB0000");
    }

    // Ported from upstream: border color resolution chain
    #[test]
    fn upstream_border_color_resolution_chain() {
        let src = "skinparam StateBorderColor #CC0000\nskinparam BorderColor #DD0000";
        let sp = parse_skinparams(src);
        assert_eq!(sp.border_color("state", "#000000"), "#CC0000");

        let src2 = "skinparam BorderColor #DD0000";
        let sp2 = parse_skinparams(src2);
        assert_eq!(sp2.border_color("state", "#000000"), "#DD0000");
    }

    #[test]
    fn parse_skinparam_nested_block_maxmessagesize() {
        let src = "@startuml\nskinparam {\n   Maxmessagesize 200\n}\ngroup Grouping messages\n    Test <- Test : text\nend\n@enduml";
        let params = parse_skinparams(src);
        assert_eq!(params.get("maxmessagesize"), Some("200"));
    }

    #[test]
    fn parse_root_style_linethickness() {
        let src = "<style>\nroot {\n  LineThickness 1\n  FontName Verdana\n}\n</style>";
        let params = parse_skinparams(src);
        assert_eq!(params.get("root.linethickness"), Some("1"));
        assert_eq!(params.get("root.fontname"), Some("Verdana"));
    }
}
