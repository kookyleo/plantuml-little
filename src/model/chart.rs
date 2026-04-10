#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub label: String,
    pub values: Vec<f64>,
    pub series_type: ChartSeriesType,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ChartSeriesType {
    Bar,
}
#[derive(Debug, Clone)]
pub struct ChartDiagram {
    pub x_labels: Vec<String>,
    pub series: Vec<ChartSeries>,
    pub x_title: Option<String>,
    pub y_title: Option<String>,
}
