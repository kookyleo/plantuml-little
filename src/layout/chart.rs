use crate::model::chart::ChartDiagram;
use crate::Result;
#[derive(Debug, Clone)]
pub struct BarLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub series_index: usize,
    pub category_index: usize,
}
#[derive(Debug, Clone)]
pub struct ChartLayout {
    pub bars: Vec<BarLayout>,
    pub width: f64,
    pub height: f64,
    pub plot_x: f64,
    pub plot_y: f64,
    pub plot_width: f64,
    pub plot_height: f64,
    pub y_max: f64,
    pub x_label_positions: Vec<(String, f64)>,
    pub series_labels: Vec<String>,
}
pub fn layout_chart(d: &ChartDiagram) -> Result<ChartLayout> {
    let nc = d
        .x_labels
        .len()
        .max(d.series.first().map_or(0, |s| s.values.len()));
    let ns = d.series.len().max(1);
    let ym = round_up(
        d.series
            .iter()
            .flat_map(|s| s.values.iter())
            .copied()
            .fold(0.0_f64, f64::max)
            .max(1.0),
    );
    let bw = 40.0_f64;
    let gw = bw * ns as f64 + 10.0;
    let pw = (gw * nc as f64).max(100.0);
    let ph = 200.0;
    let px = 90.0;
    let py = 40.0;
    let mut bars = Vec::new();
    for ci in 0..nc {
        let gx = px + ci as f64 * gw + 5.0;
        for (si, s) in d.series.iter().enumerate() {
            let v = s.values.get(ci).copied().unwrap_or(0.0);
            let bh = (v / ym) * ph;
            bars.push(BarLayout {
                x: gx + si as f64 * bw,
                y: py + ph - bh,
                width: bw,
                height: bh,
                series_index: si,
                category_index: ci,
            });
        }
    }
    let xlp: Vec<(String, f64)> = (0..nc)
        .map(|ci| {
            (
                d.x_labels
                    .get(ci)
                    .cloned()
                    .unwrap_or_else(|| format!("{}", ci + 1)),
                px + ci as f64 * gw + gw / 2.0,
            )
        })
        .collect();
    Ok(ChartLayout {
        bars,
        width: px + pw + 40.0,
        height: py + ph + 90.0,
        plot_x: px,
        plot_y: py,
        plot_width: pw,
        plot_height: ph,
        y_max: ym,
        x_label_positions: xlp,
        series_labels: d.series.iter().map(|s| s.label.clone()).collect(),
    })
}
fn round_up(v: f64) -> f64 {
    if v <= 0.0 {
        return 1.0;
    }
    let m = 10.0_f64.powf(v.log10().floor());
    let r = v / m;
    let n = if r <= 1.0 {
        1.0
    } else if r <= 2.0 {
        2.0
    } else if r <= 5.0 {
        5.0
    } else {
        10.0
    };
    n * m
}
