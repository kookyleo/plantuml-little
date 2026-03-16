// Port of net.sourceforge.plantuml.dot.GraphvizVersion
// and net.sourceforge.plantuml.dot.GraphvizVersionFinder
//
// GraphvizVersion in Java is an interface with boolean flags that govern
// layout workarounds for specific Graphviz versions. GraphvizVersionFinder
// detects the installed version by running `dot -V` and parsing output.

use std::fmt;

/// Graphviz version information.
///
/// Stores the parsed major.minor.patch version from `dot -V` output
/// and provides version-dependent layout behavior flags that match
/// Java PlantUML's `GraphvizVersion` interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphvizVersion {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

impl GraphvizVersion {
    /// Default version used when detection fails.
    /// Matches Java `GraphvizVersionFinder.DEFAULT` behavior.
    pub const DEFAULT: GraphvizVersion = GraphvizVersion {
        major: 2,
        minor: 28,
        patch: 0,
    };

    /// Numeric version as `major * 100 + minor` for comparison.
    /// Matches Java's `int v = 100 * major + minor`.
    pub fn numeric(&self) -> i32 {
        100 * self.major + self.minor
    }

    /// Java: `useShieldForQuantifier()` — true when v <= 228.
    /// Older Graphviz versions need quantifier shielding in labels.
    pub fn use_shield_for_quantifier(&self) -> bool {
        self.numeric() <= 228
    }

    /// Java: `useProtectionWhenThereALinkFromOrToGroup()`
    /// Versions 2.39 and 2.40 return false; all others true.
    pub fn use_protection_for_group_links(&self) -> bool {
        let v = self.numeric();
        if v == 239 || v == 240 {
            return false;
        }
        true
    }

    /// Java: `useXLabelInsteadOfLabel()` — always false in standard builds.
    pub fn use_xlabel_instead_of_label(&self) -> bool {
        false
    }

    /// Java: `isVizjs()` — always false for native Graphviz.
    pub fn is_vizjs(&self) -> bool {
        false
    }

    /// Java: `ignoreHorizontalLinks()` — true only for version 2.30.
    pub fn ignore_horizontal_links(&self) -> bool {
        self.numeric() == 230
    }

    /// Parse a version from `dot -V` output string.
    ///
    /// Typical output: `dot - graphviz version 2.43.0 (0)`
    /// Extracts the first `(\d+)\.(\d+)` pattern.
    /// Returns `None` if no version pattern is found.
    pub fn parse_from_dot_output(s: &str) -> Option<GraphvizVersion> {
        // Match pattern: \d+\.\d+ (with optional \.\d+ for patch)
        let re = regex::Regex::new(r"(\d+)\.(\d{1,2})(?:\.(\d+))?").ok()?;
        let caps = re.captures(s)?;
        let major: i32 = caps.get(1)?.as_str().parse().ok()?;
        let minor: i32 = caps.get(2)?.as_str().parse().ok()?;
        let patch: i32 = caps
            .get(3)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        Some(GraphvizVersion {
            major,
            minor,
            patch,
        })
    }

    /// Retrieve numeric version from a `dot -V` output string.
    /// Returns -1 if parsing fails.
    /// Matches Java `GraphvizRuntimeEnvironment.retrieveVersion()`.
    pub fn retrieve_numeric(s: &str) -> i32 {
        match Self::parse_from_dot_output(s) {
            Some(v) => v.numeric(),
            None => -1,
        }
    }
}

impl Default for GraphvizVersion {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for GraphvizVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Minimum Graphviz version limit.
/// Java: `GraphvizUtils.DOT_VERSION_LIMIT = 226`
pub const DOT_VERSION_LIMIT: i32 = 226;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_version() {
        let v = GraphvizVersion::DEFAULT;
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 28);
        assert_eq!(v.numeric(), 228);
    }

    #[test]
    fn parse_standard_output() {
        let out = "dot - graphviz version 2.43.0 (0)";
        let v = GraphvizVersion::parse_from_dot_output(out).unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 43);
        assert_eq!(v.patch, 0);
        assert_eq!(v.numeric(), 243);
    }

    #[test]
    fn parse_short_version() {
        let out = "dot - graphviz version 2.40 (something)";
        let v = GraphvizVersion::parse_from_dot_output(out).unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 40);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn parse_no_version() {
        assert!(GraphvizVersion::parse_from_dot_output("no version here").is_none());
    }

    #[test]
    fn retrieve_numeric_ok() {
        assert_eq!(
            GraphvizVersion::retrieve_numeric("dot - graphviz version 2.43.0 (0)"),
            243
        );
    }

    #[test]
    fn retrieve_numeric_fail() {
        assert_eq!(GraphvizVersion::retrieve_numeric("garbage"), -1);
    }

    #[test]
    fn shield_for_quantifier() {
        let old = GraphvizVersion {
            major: 2,
            minor: 26,
            patch: 0,
        };
        assert!(old.use_shield_for_quantifier());

        let new = GraphvizVersion {
            major: 2,
            minor: 43,
            patch: 0,
        };
        assert!(!new.use_shield_for_quantifier());
    }

    #[test]
    fn protection_for_group_links() {
        let v239 = GraphvizVersion {
            major: 2,
            minor: 39,
            patch: 0,
        };
        assert!(!v239.use_protection_for_group_links());

        let v240 = GraphvizVersion {
            major: 2,
            minor: 40,
            patch: 0,
        };
        assert!(!v240.use_protection_for_group_links());

        let v243 = GraphvizVersion {
            major: 2,
            minor: 43,
            patch: 0,
        };
        assert!(v243.use_protection_for_group_links());
    }

    #[test]
    fn ignore_horizontal_links() {
        let v230 = GraphvizVersion {
            major: 2,
            minor: 30,
            patch: 0,
        };
        assert!(v230.ignore_horizontal_links());

        let v243 = GraphvizVersion {
            major: 2,
            minor: 43,
            patch: 0,
        };
        assert!(!v243.ignore_horizontal_links());
    }

    #[test]
    fn display_format() {
        let v = GraphvizVersion {
            major: 2,
            minor: 43,
            patch: 1,
        };
        assert_eq!(format!("{v}"), "2.43.1");
    }

    #[test]
    fn dot_version_limit() {
        assert_eq!(DOT_VERSION_LIMIT, 226);
    }
}
