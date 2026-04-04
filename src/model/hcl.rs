/// A key-value pair in an HCL block.
#[derive(Debug, Clone)]
pub struct HclEntry {
    pub key: String,
    pub value: String,
}

/// HCL (Hashicorp Configuration Language) diagram model.
/// Rendered as a key-value table, similar to JSON tree-table.
#[derive(Debug, Clone)]
pub struct HclDiagram {
    pub entries: Vec<HclEntry>,
}
