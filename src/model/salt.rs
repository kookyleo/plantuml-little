#[derive(Debug, Clone)]
pub struct SaltDiagram {
    pub root: SaltWidget,
    /// True when salt is embedded inside `@startuml` (inline), false for `@startsalt`.
    /// Inline salt omits `data-diagram-type` in the SVG header (Java PSystemSalt behavior).
    pub is_inline: bool,
}

#[derive(Debug, Clone)]
pub enum SaltWidget {
    Group {
        children: Vec<SaltWidget>,
        separator: bool,
    },
    Row(Vec<SaltWidget>),
    Button(String),
    TextInput(String),
    Label(String),
    Checkbox {
        label: String,
        checked: bool,
    },
    Radio {
        label: String,
        selected: bool,
    },
    Dropdown {
        items: Vec<String>,
    },
    TreeNode {
        label: String,
        depth: usize,
    },
    Separator,
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_preserves_widget_variant() {
        let widget = SaltWidget::Checkbox {
            label: "A".to_string(),
            checked: true,
        };
        match widget.clone() {
            SaltWidget::Checkbox { label, checked } => {
                assert_eq!(label, "A");
                assert!(checked);
            }
            other => panic!("unexpected widget: {:?}", other),
        }
    }
}
