use crate::model::salt::{SaltDiagram, SaltWidget};
use crate::Result;

/// Extract salt block content and whether it's inline (`@startuml`+`salt`)
/// Returns (block_text, is_inline).
fn extract_salt_block(source: &str) -> Option<(String, bool)> {
    let mut inside = false;
    let mut lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !inside {
            if trimmed.starts_with("@startsalt") {
                inside = true;
            }
            continue;
        }
        if trimmed.starts_with("@endsalt") || trimmed.starts_with("@end") {
            break;
        }
        lines.push(line);
    }

    if lines.is_empty() {
        extract_inline_salt_block(source).map(|s| (s, true))
    } else {
        Some((lines.join("\n"), false))
    }
}

fn extract_inline_salt_block(source: &str) -> Option<String> {
    let mut saw_salt = false;
    let mut lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !saw_salt {
            if trimmed == "salt" {
                saw_salt = true;
            }
            continue;
        }
        if trimmed.starts_with("@end") {
            break;
        }
        lines.push(line);
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn parse_salt_diagram(source: &str) -> Result<SaltDiagram> {
    let (block, is_inline) = extract_salt_block(source)
        .unwrap_or_else(|| (source.to_string(), false));
    let lines: Vec<&str> = block.lines().collect();
    let mut pos = 0;

    while pos < lines.len() && lines[pos].trim().is_empty() {
        pos += 1;
    }

    let root = if pos < lines.len() && lines[pos].trim().starts_with('{') {
        parse_group(&lines, &mut pos)?
    } else {
        let mut children = Vec::new();
        while pos < lines.len() {
            let trimmed = lines[pos].trim();
            if trimmed.is_empty() || trimmed.starts_with('\'') {
                pos += 1;
                continue;
            }
            children.push(parse_line_widget(trimmed));
            pos += 1;
        }
        SaltWidget::Group {
            children,
            separator: false,
        }
    };

    Ok(SaltDiagram { root, is_inline })
}

fn parse_group(lines: &[&str], pos: &mut usize) -> Result<SaltWidget> {
    if *pos >= lines.len() {
        return Ok(SaltWidget::Group {
            children: vec![],
            separator: false,
        });
    }

    let start = lines[*pos].trim();
    let separator = start.starts_with("{-");
    let is_table = start.starts_with("{#");
    let is_tree = start.starts_with("{*");
    *pos += 1;

    if is_table {
        return parse_table(lines, pos);
    }
    if is_tree {
        return parse_tree(lines, pos);
    }

    let mut children = Vec::new();
    while *pos < lines.len() {
        let trimmed = lines[*pos].trim();
        if trimmed == "}" {
            *pos += 1;
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            *pos += 1;
            continue;
        }
        if trimmed.starts_with('{') {
            children.push(parse_group(lines, pos)?);
            continue;
        }
        if matches!(trimmed, "--" | ".." | "==" | "~~") {
            children.push(SaltWidget::Separator);
            *pos += 1;
            continue;
        }
        children.push(parse_line_widget(trimmed));
        *pos += 1;
    }

    Ok(SaltWidget::Group {
        children,
        separator,
    })
}

fn parse_table(lines: &[&str], pos: &mut usize) -> Result<SaltWidget> {
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let mut first_row = true;

    while *pos < lines.len() {
        let trimmed = lines[*pos].trim();
        if trimmed == "}" {
            *pos += 1;
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            *pos += 1;
            continue;
        }
        let cells: Vec<String> = trimmed
            .split('|')
            .map(|cell| cell.trim().to_string())
            .filter(|cell| !cell.is_empty())
            .collect();
        if first_row {
            headers = cells;
            first_row = false;
        } else {
            rows.push(cells);
        }
        *pos += 1;
    }

    Ok(SaltWidget::Table { headers, rows })
}

fn parse_tree(lines: &[&str], pos: &mut usize) -> Result<SaltWidget> {
    let mut children = Vec::new();
    while *pos < lines.len() {
        let trimmed = lines[*pos].trim();
        if trimmed == "}" {
            *pos += 1;
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            *pos += 1;
            continue;
        }
        let depth = trimmed
            .chars()
            .take_while(|ch| *ch == '+' || *ch == '-')
            .count();
        let label = trimmed
            .trim_start_matches('+')
            .trim_start_matches('-')
            .trim()
            .to_string();
        children.push(SaltWidget::TreeNode { label, depth });
        *pos += 1;
    }
    Ok(SaltWidget::Group {
        children,
        separator: false,
    })
}

fn parse_line_widget(line: &str) -> SaltWidget {
    if line.contains('|') {
        let items: Vec<SaltWidget> = line
            .split('|')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(parse_widget_text)
            .collect();
        return if items.len() == 1 {
            items.into_iter().next().unwrap()
        } else {
            SaltWidget::Row(items)
        };
    }
    parse_widget_text(line)
}

fn parse_widget_text(text: &str) -> SaltWidget {
    let text = text.trim();
    if matches!(text, "--" | ".." | "==" | "~~") {
        return SaltWidget::Separator;
    }
    if is_checkbox(text) {
        return SaltWidget::Checkbox {
            label: text[3..].trim().to_string(),
            checked: matches!(&text[1..2], "X" | "x"),
        };
    }
    if is_radio(text) {
        return SaltWidget::Radio {
            label: text[3..].trim().to_string(),
            selected: matches!(&text[1..2], "X" | "x"),
        };
    }
    if text.starts_with('[') && text.ends_with(']') {
        return SaltWidget::Button(text[1..text.len() - 1].trim().to_string());
    }
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return SaltWidget::TextInput(text[1..text.len() - 1].to_string());
    }
    if text.starts_with('^') {
        let items = text
            .trim_matches('^')
            .split('^')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect();
        return SaltWidget::Dropdown { items };
    }
    SaltWidget::Label(text.to_string())
}

fn is_checkbox(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 3
        && bytes[0] == b'['
        && matches!(bytes[1], b'X' | b'x' | b' ')
        && bytes[2] == b']'
}

fn is_radio(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 3
        && bytes[0] == b'('
        && matches!(bytes[1], b'X' | b'x' | b' ')
        && bytes[2] == b')'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_button_row() {
        let src = "@startsalt\n{\n[OK] | [Cancel]\n}\n@endsalt";
        let diagram = parse_salt_diagram(src).unwrap();
        match diagram.root {
            SaltWidget::Group { children, .. } => match &children[0] {
                SaltWidget::Row(items) => assert_eq!(items.len(), 2),
                other => panic!("unexpected row widget: {:?}", other),
            },
            other => panic!("unexpected root: {:?}", other),
        }
    }

    #[test]
    fn parse_table_group() {
        let src = "@startsalt\n{#\n| Name | Age |\n| Alice | 30 |\n}\n@endsalt";
        let diagram = parse_salt_diagram(src).unwrap();
        match diagram.root {
            SaltWidget::Table { headers, rows } => {
                assert_eq!(headers, vec!["Name", "Age"]);
                assert_eq!(rows.len(), 1);
            }
            other => panic!("unexpected table widget: {:?}", other),
        }
    }

    #[test]
    fn parse_inline_salt_block_inside_uml() {
        let src = "@startuml\nsalt\n{#\n| Name | Age |\n| Alice | 30 |\n}\n@enduml";
        let diagram = parse_salt_diagram(src).unwrap();
        match diagram.root {
            SaltWidget::Table { headers, rows } => {
                assert_eq!(headers, vec!["Name", "Age"]);
                assert_eq!(rows.len(), 1);
            }
            other => panic!("unexpected inline salt widget: {:?}", other),
        }
    }
}
