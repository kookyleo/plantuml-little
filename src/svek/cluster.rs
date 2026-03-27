// svek::cluster - Subgraph/package grouping
// Port of Java PlantUML's svek.Cluster, ClusterDotString, ClusterDecoration

/// A cluster (subgraph) grouping nodes in the Graphviz layout.
/// Java: `svek.Cluster`
#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: String,
    pub title: Option<String>,
    pub label_size: Option<(f64, f64)>,
    pub node_uids: Vec<String>,
    pub sub_clusters: Vec<Cluster>,
    /// Position after layout
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Style
    pub style: ClusterStyle,
}

/// Visual style for cluster borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClusterStyle {
    #[default]
    Rectangle,
    RoundedRectangle,
    Package,
    Frame,
    Folder,
    Cloud,
    Node,
}

impl Cluster {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            title: None,
            label_size: None,
            node_uids: Vec::new(),
            sub_clusters: Vec::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            style: ClusterStyle::default(),
        }
    }

    pub fn add_node(&mut self, uid: &str) {
        self.node_uids.push(uid.to_string());
    }
}

// TODO: Full port of ClusterDotString, ClusterDecoration

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_basic() {
        let mut c = Cluster::new("pkg1");
        c.add_node("Foo");
        c.add_node("Bar");
        assert_eq!(c.node_uids.len(), 2);
    }
}
