use crate::model::math::MathDiagram;
use crate::Result;

fn extract_math_block(source: &str, start_tag: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@end") {
                break;
            }
            lines.push(line);
        } else if t.starts_with(start_tag) {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Parse a @startmath diagram.
pub fn parse_math_diagram(source: &str) -> Result<MathDiagram> {
    let block = extract_math_block(source, "@startmath").unwrap_or_default();
    // Java PSystemMath takes the last non-command line as the formula.
    // For simplicity, use the full block trimmed.
    let formula = block.trim().to_string();
    Ok(MathDiagram { formula })
}

/// Parse a @startlatex diagram.
pub fn parse_latex_diagram(source: &str) -> Result<MathDiagram> {
    let block = extract_math_block(source, "@startlatex").unwrap_or_default();
    let formula = block.trim().to_string();
    Ok(MathDiagram { formula })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_math() {
        let src = "@startmath\nf(x) = x^2 + 1\n@endmath";
        let d = parse_math_diagram(src).unwrap();
        assert_eq!(d.formula, "f(x) = x^2 + 1");
    }

    #[test]
    fn test_parse_latex() {
        let src = "@startlatex\nE = mc^2\n@endlatex";
        let d = parse_latex_diagram(src).unwrap();
        assert_eq!(d.formula, "E = mc^2");
    }
}
