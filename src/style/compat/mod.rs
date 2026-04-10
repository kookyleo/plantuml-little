use std::collections::HashMap;

mod lookup;
mod normalize;
mod parse;
mod stereotype;
mod tests;
mod theme;

pub use normalize::normalize_color;
pub use parse::parse_skinparams;
pub use theme::Theme;

/// Parsed skinparam settings from PlantUML source.
///
/// Keys are stored in lowercase for case-insensitive lookup.
/// Element-scoped params use dot notation: `component.backgroundcolor`.
///
/// When no explicit param is set, lookup methods fall back to the embedded
/// [`Theme`] (rose by default).
#[derive(Debug, Clone, Default)]
pub struct SkinParams {
    pub(super) params: HashMap<String, String>,
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
}
