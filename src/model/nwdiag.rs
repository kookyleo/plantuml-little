/// Network diagram (nwdiag) IR.

#[derive(Debug, Clone)]
pub struct NwdiagDiagram {
    pub networks: Vec<Network>,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub name: String,
    pub address: Option<String>,
    pub color: Option<String>,
    pub servers: Vec<ServerRef>,
}

#[derive(Debug, Clone)]
pub struct ServerRef {
    pub name: String,
    pub address: Option<String>,
    pub description: Option<String>,
}
