use super::SkinParams;

/// Skinparam value lookup methods for `SkinParams`.
///
/// These implement the element-keyed fallback chains that mirror Java
/// PlantUML's `ISkinParam` resolution order.
impl SkinParams {
    /// Lookup the global `wrapWidth` skinparam (C4 stdlib sets `skinparam
    /// wrapWidth 200`). Returns `None` when unset.
    pub fn wrap_width(&self) -> Option<f64> {
        self.params
            .get("wrapwidth")
            .and_then(|v| v.parse::<f64>().ok())
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
        // Java's PName only has LineColor (no BorderColor); element <style>
        // blocks set LineColor, which becomes the visible border for
        // bounded shapes. Fall back to it after the legacy bordercolor keys.
        let line_key1 = format!("{element}linecolor");
        let line_key2 = format!("{element}.linecolor");
        if let Some(v) = self.params.get(&line_key1) {
            return v.as_str();
        }
        if let Some(v) = self.params.get(&line_key2) {
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
    /// 1. `ArrowColor` skinparam
    /// 2. `root.linecolor` from style block
    /// 3. Theme arrow color / default
    pub fn arrow_color<'a>(&'a self, default: &'a str) -> &'a str {
        // Java: `skinparam arrowColor` or `skinparam arrow { Color }` both
        // resolve to the same value.  Our parser stores the block form as
        // `arrow.color` (dot notation) and the flat form as `arrowcolor`.
        if let Some(v) = self.params.get("arrowcolor") {
            return v.as_str();
        }
        if let Some(v) = self.params.get("arrow.color") {
            return v.as_str();
        }
        // Java style system: arrows inherit from root { LineColor }
        if let Some(v) = self.params.get("root.linecolor") {
            return v.as_str();
        }
        if default == self.theme.arrow_color {
            return &self.theme.arrow_color;
        }
        default
    }

    // ── Theme element lookups ──────────────────────────────────────

    /// Return the theme background color for a known element, or `None`.
    pub(super) fn theme_bg(&self, element: &str) -> Option<&str> {
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
    pub(super) fn theme_border(&self, element: &str) -> Option<&str> {
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
    pub(super) fn theme_font(&self, element: &str) -> Option<&str> {
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

    /// Get font size for an element, returning None when unset.
    pub fn font_size_opt(&self, element: &str) -> Option<f64> {
        let key1 = format!("{element}fontsize");
        let key2 = format!("{element}.fontsize");
        if let Some(v) = self.params.get(&key1).and_then(|s| s.parse::<f64>().ok()) {
            return Some(v);
        }
        if let Some(v) = self.params.get(&key2).and_then(|s| s.parse::<f64>().ok()) {
            return Some(v);
        }
        None
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
