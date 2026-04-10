use super::SkinParams;

/// Stereotype-keyed lookup helpers for `SkinParams`.
///
/// Stereotype-keyed skinparams are written as
/// `skinparam element<<stereo>> { Key value }` in PlantUML source.
/// Our parser stores these under `element<<stereo>>.key`.
impl SkinParams {
    /// Look up a stereotype-keyed param for a specific element.
    ///
    /// This helper accepts a list of stereotype labels (from the entity's
    /// `stereotypes` field) and returns the first matching value, so the
    /// first stereotype in the list takes precedence — matching how Java
    /// PlantUML resolves tag-based styling.
    pub(super) fn stereotype_param(
        &self,
        element: &str,
        stereotypes: &[&str],
        key: &str,
    ) -> Option<&str> {
        for stereo in stereotypes {
            let full = format!("{element}<<{stereo}>>.{key}").to_lowercase();
            if let Some(v) = self.params.get(&full) {
                return Some(v.as_str());
            }
        }
        None
    }

    /// Stereotype-aware background color lookup. Falls back to the regular
    /// element lookup chain when no stereotype override matches.
    pub fn background_color_for<'a>(
        &'a self,
        element: &str,
        stereotypes: &[&str],
        default: &'a str,
    ) -> &'a str {
        if let Some(v) = self.stereotype_param(element, stereotypes, "backgroundcolor") {
            return v;
        }
        self.background_color(element, default)
    }

    /// Stereotype-aware border color lookup.
    pub fn border_color_for<'a>(
        &'a self,
        element: &str,
        stereotypes: &[&str],
        default: &'a str,
    ) -> &'a str {
        if let Some(v) = self.stereotype_param(element, stereotypes, "bordercolor") {
            return v;
        }
        self.border_color(element, default)
    }

    /// Stereotype-aware font color lookup.
    pub fn font_color_for<'a>(
        &'a self,
        element: &str,
        stereotypes: &[&str],
        default: &'a str,
    ) -> &'a str {
        if let Some(v) = self.stereotype_param(element, stereotypes, "fontcolor") {
            return v;
        }
        self.font_color(element, default)
    }

    /// Stereotype-aware border style lookup. Returns the raw style string
    /// (`dashed`, `dotted`, `solid`, etc.) as provided in the skinparam.
    pub fn border_style_for(&self, element: &str, stereotypes: &[&str]) -> Option<&str> {
        self.stereotype_param(element, stereotypes, "borderstyle")
    }

    /// Stereotype-aware stereotype-font-color lookup (used when rendering
    /// the `«stereo»` label above an element title).
    pub fn stereotype_font_color_for<'a>(
        &'a self,
        element: &str,
        stereotypes: &[&str],
    ) -> Option<&'a str> {
        self.stereotype_param(element, stereotypes, "stereotypefontcolor")
    }

    /// Stereotype-aware stereotype-font-size lookup.
    /// Returns the font size used for rendering stereotype labels above
    /// the element title. Used by ClusterHeader to include the stereo block
    /// height in the cluster label dimension.
    pub fn stereotype_font_size_for(&self, element: &str, stereotypes: &[&str]) -> Option<f64> {
        if let Some(v) = self.stereotype_param(element, stereotypes, "stereotypefontsize") {
            return v.parse::<f64>().ok();
        }
        // Fall back to the undecorated element param
        let key = format!("{element}.stereotypefontsize");
        if let Some(v) = self.params.get(&key) {
            return v.parse::<f64>().ok();
        }
        None
    }

    /// Stereotype-aware `RoundCorner` lookup.
    /// Returns `Some(0.0)` when `RoundCorner 0` is set (sharp corners),
    /// `Some(N)` for an explicit radius, or `None` when no value is
    /// specified (caller decides the default).
    pub fn round_corner_for(&self, element: &str, stereotypes: &[&str]) -> Option<f64> {
        if let Some(v) = self.stereotype_param(element, stereotypes, "roundcorner") {
            return v.parse::<f64>().ok();
        }
        None
    }
}
