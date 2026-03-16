// abel::group_type - Group entity type enumeration
// Port of Java PlantUML's abel.GroupType

/// Type of a group (container) entity.
/// Java: `abel.GroupType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupType {
    Root,
    Package,
    State,
    ConcurrentState,
    InnerActivity,
    ConcurrentActivity,
    Domain,
    Requirement,
}

impl GroupType {
    /// Whether this group type is autarkic (self-contained for layout).
    /// Used by `Entity::is_autarkic()` to decide if inner links stay internal.
    pub fn is_autarkic(&self) -> bool {
        matches!(
            self,
            Self::InnerActivity | Self::ConcurrentActivity | Self::ConcurrentState
        )
    }
}

impl std::fmt::Display for GroupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autarkic_types() {
        assert!(!GroupType::Root.is_autarkic());
        assert!(!GroupType::Package.is_autarkic());
        assert!(GroupType::State.is_autarkic() == false);
        assert!(GroupType::ConcurrentState.is_autarkic());
        assert!(GroupType::InnerActivity.is_autarkic());
        assert!(GroupType::ConcurrentActivity.is_autarkic());
        assert!(!GroupType::Domain.is_autarkic());
        assert!(!GroupType::Requirement.is_autarkic());
    }

    #[test]
    fn display_format() {
        assert_eq!(format!("{}", GroupType::Package), "Package");
        assert_eq!(format!("{}", GroupType::ConcurrentState), "ConcurrentState");
    }

    #[test]
    fn equality() {
        assert_eq!(GroupType::Package, GroupType::Package);
        assert_ne!(GroupType::Package, GroupType::State);
    }

    #[test]
    fn clone_and_copy() {
        let g = GroupType::State;
        let g2 = g;
        assert_eq!(g, g2);
    }

    #[test]
    fn hash_support() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(GroupType::Root);
        set.insert(GroupType::Package);
        set.insert(GroupType::Root); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn all_variants() {
        let variants = vec![
            GroupType::Root,
            GroupType::Package,
            GroupType::State,
            GroupType::ConcurrentState,
            GroupType::InnerActivity,
            GroupType::ConcurrentActivity,
            GroupType::Domain,
            GroupType::Requirement,
        ];
        assert_eq!(variants.len(), 8);
    }
}
