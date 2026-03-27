// Port of net.sourceforge.plantuml.dot.DotData
//
// Container for input data fed to the DOT layout engine.
// Java DotData holds entities, links, group hierarchy, etc.
// In Rust we keep a simplified but equivalent structure.

use std::collections::{HashMap, HashSet};

/// Represents a leaf entity (node) in the diagram.
#[derive(Debug, Clone)]
pub struct DotEntity {
    pub uid: String,
    pub label: String,
    pub width: f64,
    pub height: f64,
}

/// Represents a link (edge) between two entities.
#[derive(Debug, Clone)]
pub struct DotLink {
    pub entity1_uid: String,
    pub entity2_uid: String,
    pub label: Option<String>,
    pub minlen: u32,
    pub invisible: bool,
    /// Same-tail grouping key. Java: `Link.getSametail()`
    pub sametail: Option<String>,
    /// Whether the link decoration extends-like (arrow).
    /// Java: `link.getType().getDecor2().isExtendsLike()`
    pub is_extends_like: bool,
}

/// Container for DOT layout input data.
///
/// Port of Java `DotData`: holds the leaf entities, links, group hierarchy
/// reference, and diagram metadata needed to generate DOT source.
///
/// Java fields ported:
/// - `leafs` -> `leafs`
/// - `links` -> `links`
/// - `topParent` -> `top_parent_uid`
/// - `groupHierarchy.isEmpty(g)` -> `empty_groups`
/// - `entityFactory.groups().size()` -> `group_count`
/// - `skinParam.groupInheritance()` -> `group_inheritance`
#[derive(Debug, Clone)]
pub struct DotData {
    pub leafs: Vec<DotEntity>,
    pub links: Vec<DotLink>,
    pub top_parent_uid: String,
    /// UIDs of groups that are empty.
    pub empty_groups: HashSet<String>,
    /// Total number of groups in the diagram.
    pub group_count: usize,
    /// `skinParam.groupInheritance()` threshold for sametail removal.
    pub group_inheritance: usize,
    /// Hide empty description for state diagrams.
    pub hide_empty_description_for_state: bool,
}

impl DotData {
    /// Java: `isDegeneratedWithFewEntities(nb)`
    /// True when there are no groups, no links, and exactly `nb` leafs.
    pub fn is_degenerated_with_few_entities(&self, nb: usize) -> bool {
        self.group_count == 0 && self.links.is_empty() && self.leafs.len() == nb
    }

    /// Java: `isEmpty(g)` — delegates to group hierarchy.
    pub fn is_group_empty(&self, group_uid: &str) -> bool {
        self.empty_groups.contains(group_uid)
    }

    /// Java: `getLeaf(key)` — find a leaf entity by UID.
    pub fn get_leaf(&self, uid: &str) -> Option<&DotEntity> {
        self.leafs.iter().find(|e| e.uid == uid)
    }

    /// Java: `getLinksOfThisLeaf(leaf)` — all links involving a given entity.
    pub fn links_of_leaf(&self, uid: &str) -> Vec<&DotLink> {
        self.links
            .iter()
            .filter(|l| l.entity1_uid == uid || l.entity2_uid == uid)
            .collect()
    }

    /// Java: `removeIrrelevantSametail()`
    ///
    /// Assigns sametail from entity1 UID for extends-like links, then removes
    /// sametail markers that appear fewer than `group_inheritance` times.
    pub fn remove_irrelevant_sametail(&mut self) {
        // Step 1: assign sametail for extends-like links
        for link in &mut self.links {
            if link.is_extends_like {
                link.sametail = Some(link.entity1_uid.clone());
            }
        }

        // Step 2: count sametail occurrences
        let mut sametail_counts: HashMap<String, usize> = HashMap::new();
        for link in &self.links {
            if let Some(ref st) = link.sametail {
                *sametail_counts.entry(st.clone()).or_insert(0) += 1;
            }
        }

        // Step 3: determine which sametails to remove (below threshold)
        let limit = self.group_inheritance;
        let to_remove: HashSet<String> = sametail_counts
            .iter()
            .filter(|(_, &count)| count < limit)
            .map(|(key, _)| key.clone())
            .collect();

        // Step 4: clear sametail on links whose key is below threshold
        for link in &mut self.links {
            if let Some(ref st) = link.sametail {
                if to_remove.contains(st) {
                    link.sametail = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> DotData {
        DotData {
            leafs: vec![
                DotEntity {
                    uid: "A".into(),
                    label: "ClassA".into(),
                    width: 100.0,
                    height: 40.0,
                },
                DotEntity {
                    uid: "B".into(),
                    label: "ClassB".into(),
                    width: 100.0,
                    height: 40.0,
                },
                DotEntity {
                    uid: "C".into(),
                    label: "ClassC".into(),
                    width: 100.0,
                    height: 40.0,
                },
            ],
            links: vec![
                DotLink {
                    entity1_uid: "A".into(),
                    entity2_uid: "B".into(),
                    label: None,
                    minlen: 1,
                    invisible: false,
                    sametail: None,
                    is_extends_like: true,
                },
                DotLink {
                    entity1_uid: "A".into(),
                    entity2_uid: "C".into(),
                    label: None,
                    minlen: 1,
                    invisible: false,
                    sametail: None,
                    is_extends_like: true,
                },
            ],
            top_parent_uid: "root".into(),
            empty_groups: HashSet::new(),
            group_count: 0,
            group_inheritance: 2,
            hide_empty_description_for_state: false,
        }
    }

    #[test]
    fn degenerated_check() {
        let d = sample_data();
        // Has links, so not degenerated
        assert!(!d.is_degenerated_with_few_entities(3));

        let d2 = DotData { links: vec![], ..d };
        assert!(d2.is_degenerated_with_few_entities(3));
        assert!(!d2.is_degenerated_with_few_entities(2));
    }

    #[test]
    fn get_leaf_by_uid() {
        let d = sample_data();
        assert!(d.get_leaf("A").is_some());
        assert_eq!(d.get_leaf("A").unwrap().label, "ClassA");
        assert!(d.get_leaf("Z").is_none());
    }

    #[test]
    fn links_of_leaf() {
        let d = sample_data();
        let links_a = d.links_of_leaf("A");
        assert_eq!(links_a.len(), 2);
        let links_b = d.links_of_leaf("B");
        assert_eq!(links_b.len(), 1);
        let links_z = d.links_of_leaf("Z");
        assert!(links_z.is_empty());
    }

    #[test]
    fn remove_irrelevant_sametail_keeps_above_threshold() {
        let mut d = sample_data();
        d.group_inheritance = 2;
        d.remove_irrelevant_sametail();

        // Both extends-like links get sametail="A", count=2 >= threshold=2 -> kept
        for link in &d.links {
            assert_eq!(link.sametail, Some("A".into()));
        }
    }

    #[test]
    fn remove_irrelevant_sametail_removes_below_threshold() {
        let mut d = sample_data();
        d.group_inheritance = 3; // threshold=3, only 2 links with sametail -> remove
        d.remove_irrelevant_sametail();

        for link in &d.links {
            assert_eq!(link.sametail, None);
        }
    }

    #[test]
    fn empty_group_check() {
        let mut d = sample_data();
        d.empty_groups.insert("g1".into());
        assert!(d.is_group_empty("g1"));
        assert!(!d.is_group_empty("g2"));
    }
}
