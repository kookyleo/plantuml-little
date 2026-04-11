use log::debug;

use crate::model::pie::{PieDiagram, PieSlice};
use crate::Result;

fn extract_pie_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endpie") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startpie") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn parse_pie_diagram(source: &str) -> Result<PieDiagram> {
    let block = extract_pie_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_pie_diagram: {} bytes", block.len());

    let mut title = None;
    let mut slices = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') {
            continue;
        }

        if let Some(rest) = t.strip_prefix("title ") {
            title = Some(rest.trim().to_string());
            continue;
        }

        // Parse "Label" : value
        if let Some(after_quote) = t.strip_prefix('"') {
            if let Some(end_quote) = after_quote.find('"') {
                let label = after_quote[..end_quote].to_string();
                let rest = t[2 + end_quote..].trim();
                if let Some(val_str) = rest.strip_prefix(':') {
                    if let Ok(value) = val_str.trim().parse::<f64>() {
                        slices.push(PieSlice { label, value });
                    }
                }
            }
        }
    }

    Ok(PieDiagram { title, slices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_pie() {
        let src = r#"@startpie
title Browser Share
"Chrome" : 65
"Firefox" : 15
@endpie"#;
        let d = parse_pie_diagram(src).unwrap();
        assert_eq!(d.title.as_deref(), Some("Browser Share"));
        assert_eq!(d.slices.len(), 2);
        assert_eq!(d.slices[0].label, "Chrome");
        assert_eq!(d.slices[0].value, 65.0);
    }
}
