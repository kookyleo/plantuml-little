use log::debug;

use crate::model::wire::{WireBlock, WireDiagram, WireVLink};
use crate::Result;

fn extract_wire_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endwire") || t.starts_with("@enduml") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startwire") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Parse a wire diagram source into a WireDiagram model.
///
/// Java wire syntax:
///   * ComponentName           — define a block (100x100 default)
///   * ComponentName `[WxH]`   — define a block with explicit size
///   * Name1 --> Name2 : label — vertical link
pub fn parse_wire_diagram(source: &str) -> Result<WireDiagram> {
    let block = extract_wire_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_wire_diagram: {} bytes", block.len());

    let mut blocks = Vec::new();
    let mut vlinks = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') {
            continue;
        }

        // Count leading whitespace/tabs for indentation level
        let indent = line.len() - line.trim_start().len();
        let level = indent / 4; // Java uses 4 spaces = 1 tab = 1 level

        // Component: * Name [WxH] [#color]
        if let Some(rest) = t.strip_prefix('*') {
            let rest = rest.trim();
            let mut name = String::new();
            let mut width = 0.0;
            let mut height = 0.0;
            let mut color = None;

            // Parse name, optional [WxH], optional #color
            let parts: Vec<&str> = rest.splitn(2, '[').collect();
            if parts.len() == 2 {
                name = parts[0].trim().to_string();
                // Parse WxH]
                if let Some(dim_end) = parts[1].find(']') {
                    let dim_str = &parts[1][..dim_end];
                    let dims: Vec<&str> = dim_str.split(['x', '*'].as_slice()).collect();
                    if dims.len() == 2 {
                        width = dims[0].trim().parse().unwrap_or(0.0);
                        height = dims[1].trim().parse().unwrap_or(0.0);
                    }
                    // Check for color after ]
                    let after = parts[1][dim_end + 1..].trim();
                    if after.starts_with('#') {
                        color = Some(after.to_string());
                    }
                }
            } else {
                // No dimension, check for color
                let tokens: Vec<&str> = rest.split_whitespace().collect();
                if !tokens.is_empty() {
                    name = tokens[0].to_string();
                    for tok in &tokens[1..] {
                        if tok.starts_with('#') {
                            color = Some(tok.to_string());
                        }
                    }
                }
            }

            if !name.is_empty() {
                blocks.push(WireBlock {
                    name,
                    width,
                    height,
                    color,
                    level,
                });
            }
            continue;
        }

        // Vertical link: Name1 --> Name2 : label
        if t.contains("-->") {
            let parts: Vec<&str> = t.splitn(2, "-->").collect();
            if parts.len() == 2 {
                let from = parts[0].trim().to_string();
                let to_part = parts[1].trim();
                // Strip optional ": label"
                let to = if let Some(idx) = to_part.find(':') {
                    to_part[..idx].trim().to_string()
                } else {
                    to_part.to_string()
                };
                if !from.is_empty() && !to.is_empty() {
                    vlinks.push(WireVLink { from, to });
                }
            }
        }
    }

    Ok(WireDiagram { blocks, vlinks })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_wire() {
        let src = "@startwire\n* A\n* B\n@endwire";
        let d = parse_wire_diagram(src).unwrap();
        assert_eq!(d.blocks.len(), 2);
        assert_eq!(d.blocks[0].name, "A");
        assert_eq!(d.blocks[1].name, "B");
        assert!(d.vlinks.is_empty());
    }

    #[test]
    fn test_parse_wire_with_link() {
        let src = "@startwire\n* A\n* B\nA --> B : data\n@endwire";
        let d = parse_wire_diagram(src).unwrap();
        assert_eq!(d.blocks.len(), 2);
        assert_eq!(d.vlinks.len(), 1);
        assert_eq!(d.vlinks[0].from, "A");
        assert_eq!(d.vlinks[0].to, "B");
    }

    #[test]
    fn test_parse_wire_with_dimensions() {
        let src = "@startwire\n* Chip [200x150]\n@endwire";
        let d = parse_wire_diagram(src).unwrap();
        assert_eq!(d.blocks.len(), 1);
        assert_eq!(d.blocks[0].name, "Chip");
        assert!((d.blocks[0].width - 200.0).abs() < 0.01);
        assert!((d.blocks[0].height - 150.0).abs() < 0.01);
    }
}
