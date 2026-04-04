use log::debug;

use crate::model::hcl::{HclDiagram, HclEntry};
use crate::Result;

fn extract_hcl_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endhcl") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@starthcl") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn parse_hcl_diagram(source: &str) -> Result<HclDiagram> {
    let block = extract_hcl_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_hcl_diagram: {} bytes", block.len());

    let mut entries = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') || t.starts_with('{') || t.starts_with('}') {
            continue;
        }

        // Skip resource/block declaration lines like: resource "type" "name" {
        if t.starts_with("resource ")
            || t.starts_with("variable ")
            || t.starts_with("output ")
            || t.starts_with("data ")
            || t.starts_with("module ")
            || t.starts_with("provider ")
            || t.starts_with("terraform ")
            || t.starts_with("locals ")
        {
            continue;
        }

        // Parse key = "value" or key = value
        if let Some(eq_pos) = t.find('=') {
            let key = t[..eq_pos].trim().to_string();
            let val_raw = t[eq_pos + 1..].trim();
            // Strip surrounding quotes from value
            let value = if val_raw.starts_with('"') && val_raw.ends_with('"') && val_raw.len() >= 2
            {
                val_raw[1..val_raw.len() - 1].to_string()
            } else {
                val_raw.to_string()
            };
            entries.push(HclEntry { key, value });
        }
    }

    Ok(HclDiagram { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_hcl() {
        let src = r#"@starthcl
resource "aws_instance" "web" {
  ami           = "abc-123"
  instance_type = "t2.micro"
}
@endhcl"#;
        let d = parse_hcl_diagram(src).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert_eq!(d.entries[0].key, "ami");
        assert_eq!(d.entries[0].value, "abc-123");
        assert_eq!(d.entries[1].key, "instance_type");
        assert_eq!(d.entries[1].value, "t2.micro");
    }
}
