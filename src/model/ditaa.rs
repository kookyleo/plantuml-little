#[derive(Debug, Clone)]
pub struct DitaaDiagram {
    pub source: String,
    pub options: DitaaOptions,
}

#[derive(Debug, Clone, Default)]
pub struct DitaaOptions {
    pub no_separation: bool,
    pub round_corners: bool,
    pub no_shadows: bool,
    pub scale: Option<f64>,
}
