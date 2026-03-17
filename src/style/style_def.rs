// Style, StyleBuilder, StyleStorage, StyleLoader
// Placeholder types for forward references from signature.rs

use super::signature::StyleSignatureBasic;
use super::value::MergeStrategy;

/// Placeholder: a resolved style (map of PName -> Value).
/// Will be fully implemented when the style loader is ported.
#[derive(Debug, Clone)]
pub struct Style;

impl Style {
    /// Merge with another Style using the given strategy.
    pub fn merge_with(&self, _other: &Style, _strategy: MergeStrategy) -> Style {
        Style
    }
}

/// Placeholder: storage and lookup for styles.
/// Will be fully implemented when the style loader is ported.
#[derive(Debug, Clone)]
pub struct StyleBuilder;

impl StyleBuilder {
    /// Resolve styles for a given signature.
    pub fn get_merged_style(&self, _sig: &StyleSignatureBasic) -> Style {
        Style
    }
}
