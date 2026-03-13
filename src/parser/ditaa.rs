use crate::model::ditaa::{DitaaDiagram, DitaaOptions};
use crate::Result;

pub fn parse_ditaa(source: &str) -> Result<DitaaDiagram> {
    let lines: Vec<&str> = source.lines().collect();

    let start_idx = lines
        .iter()
        .position(|line| line.trim().starts_with("@startditaa"))
        .ok_or_else(|| crate::Error::Parse {
            line: 1,
            column: Some(1),
            message: "missing @startditaa".to_string(),
        })?;
    let end_idx = lines
        .iter()
        .position(|line| line.trim().starts_with("@endditaa"))
        .ok_or_else(|| crate::Error::Parse {
            line: lines.len().max(1),
            column: Some(1),
            message: "missing @endditaa".to_string(),
        })?;

    if end_idx <= start_idx {
        return Err(crate::Error::Parse {
            line: end_idx + 1,
            column: Some(1),
            message: "@endditaa before @startditaa".to_string(),
        });
    }

    let options = parse_options(lines[start_idx].trim());
    let body = lines[start_idx + 1..end_idx].join("\n");
    Ok(DitaaDiagram {
        source: body,
        options,
    })
}

fn parse_options(line: &str) -> DitaaOptions {
    let mut options = DitaaOptions::default();
    let rest = line.strip_prefix("@startditaa").unwrap_or("").trim();
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut idx = 0;
    while idx < tokens.len() {
        match tokens[idx] {
            "-E" => options.no_separation = true,
            "-r" => options.round_corners = true,
            "-S" => options.no_shadows = true,
            "-T" => {
                if idx + 1 < tokens.len() {
                    idx += 1;
                }
            }
            "--scale" | "scale" => {
                if idx + 1 < tokens.len() {
                    options.scale = tokens[idx + 1].parse::<f64>().ok();
                    idx += 1;
                }
            }
            value if value.starts_with("scale=") => {
                options.scale = value[6..].parse::<f64>().ok();
            }
            _ => {}
        }
        idx += 1;
    }
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_ditaa() {
        let src = "@startditaa\n+--+\n|A |\n+--+\n@endditaa";
        let diagram = parse_ditaa(src).unwrap();
        assert!(diagram.source.contains("+--+"));
    }

    #[test]
    fn parse_options() {
        let src = "@startditaa -r -S -E --scale 2.5\n+--+\n@endditaa";
        let diagram = parse_ditaa(src).unwrap();
        assert!(diagram.options.round_corners);
        assert!(diagram.options.no_shadows);
        assert!(diagram.options.no_separation);
        assert_eq!(diagram.options.scale, Some(2.5));
    }
}
