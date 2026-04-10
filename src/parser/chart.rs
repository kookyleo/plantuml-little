use crate::model::chart::{ChartDiagram, ChartSeries, ChartSeriesType};
use crate::Result;
use log::{debug, trace};
fn extract_chart_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endchart") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startchart") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}
pub fn parse_chart_diagram(source: &str) -> Result<ChartDiagram> {
    let mut inside = false;
    for (idx, line) in source.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("@startchart") {
            inside = true;
            continue;
        }
        if inside {
            if t.starts_with("@endchart") {
                break;
            }
            if !t.is_empty() && !t.starts_with('\'') {
                return Err(crate::Error::JavaErrorPage {
                    line: idx + 1,
                    message: "Syntax Error? (Assumed diagram type: chart)".into(),
                });
            }
        }
    }

    let block = extract_chart_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_chart_diagram: {} bytes", block.len());
    let (mut xl, mut sr, mut xt, mut yt) = (vec![], vec![], None, None);
    for (n, line) in block.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("'") {
            continue;
        }
        if let Some(r) = t.strip_prefix("h-axis") {
            let r = r.trim();
            if let Some(labels) = parse_quoted_list(r) {
                if labels.len() == 1 && !r.contains(',') {
                    xt = Some(labels[0].clone());
                } else {
                    xl = labels;
                }
            }
        } else if let Some(r) = t.strip_prefix("v-axis") {
            let r = r.trim();
            if let Some(p) = r.find('"') {
                if let Some(e) = r[p + 1..].find('"') {
                    yt = Some(r[p + 1..p + 1 + e].to_string());
                }
            }
        } else if let Some(r) = t.strip_prefix("bar") {
            if let Some((l, v)) = parse_series_line(r.trim()) {
                debug!("line {}: bar '{}' {} vals", n + 1, l, v.len());
                sr.push(ChartSeries {
                    label: l,
                    values: v,
                    series_type: ChartSeriesType::Bar,
                });
            }
        } else {
            trace!("line {}: skip '{}'", n + 1, t);
        }
    }
    Ok(ChartDiagram {
        x_labels: xl,
        series: sr,
        x_title: xt,
        y_title: yt,
    })
}
fn parse_quoted_list(input: &str) -> Option<Vec<String>> {
    let (mut r, mut rest) = (vec![], input);
    loop {
        rest = rest.trim();
        if rest.is_empty() {
            break;
        }
        if rest.starts_with('"') {
            if let Some(e) = rest[1..].find('"') {
                r.push(rest[1..1 + e].to_string());
                rest = rest[2 + e..].trim_start_matches(',').trim_start();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    if r.is_empty() {
        None
    } else {
        Some(r)
    }
}
fn parse_series_line(input: &str) -> Option<(String, Vec<f64>)> {
    let t = input.trim();
    if !t.starts_with('"') {
        return None;
    }
    let eq = t[1..].find('"')?;
    let label = t[1..1 + eq].to_string();
    let rest = t[2 + eq..].trim().strip_prefix(':')?.trim();
    let vals: Vec<f64> = rest
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if vals.is_empty() {
        None
    } else {
        Some((label, vals))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_chart_fixture_syntax_errors_like_java_stable() {
        let err = parse_chart_diagram(
            "@startchart\nh-axis \"Q1\", \"Q2\"\nbar \"S\" : 30, 50\n@endchart",
        )
        .unwrap_err();
        match err {
            crate::Error::JavaErrorPage { line, message } => {
                assert_eq!(line, 2);
                assert_eq!(message, "Syntax Error? (Assumed diagram type: chart)");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
