use crate::model::creole_diagram::{CreoleDiagram, CreoleElement};
use crate::Result;

fn extract_creole_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endcreole") || t.starts_with("@enduml") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startcreole") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Parse a @startcreole diagram.
pub fn parse_creole_diagram(source: &str) -> Result<CreoleDiagram> {
    let block = extract_creole_block(source).unwrap_or_default();
    let mut elements = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }

        // Heading: = Title, == Subtitle, etc.
        let heading_level = t.chars().take_while(|&c| c == '=').count();
        if heading_level > 0 && heading_level <= 5 {
            let text = t[heading_level..].trim().to_string();
            if !text.is_empty() {
                elements.push(CreoleElement::Heading {
                    text,
                    level: heading_level,
                });
                continue;
            }
        }

        // Bullet: * item, ** sub-item, etc.
        let bullet_level = t.chars().take_while(|&c| c == '*').count();
        if bullet_level > 0 {
            let text = t[bullet_level..].trim().to_string();
            elements.push(CreoleElement::Bullet {
                text,
                level: bullet_level,
            });
            continue;
        }

        // Plain text
        elements.push(CreoleElement::Text(t.to_string()));
    }

    Ok(CreoleDiagram { elements })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_creole() {
        let src = "@startcreole\n= Title\n* bullet 1\n* bullet 2\n@endcreole";
        let d = parse_creole_diagram(src).unwrap();
        assert_eq!(d.elements.len(), 3);
        assert!(
            matches!(&d.elements[0], CreoleElement::Heading { text, level: 1 } if text == "Title")
        );
        assert!(
            matches!(&d.elements[1], CreoleElement::Bullet { text, level: 1 } if text == "bullet 1")
        );
        assert!(
            matches!(&d.elements[2], CreoleElement::Bullet { text, level: 1 } if text == "bullet 2")
        );
    }
}
