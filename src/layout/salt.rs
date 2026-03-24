use crate::font_metrics;
use crate::model::salt::{SaltDiagram, SaltWidget};
use crate::Result;

#[derive(Debug, Clone)]
pub struct SaltLayout {
    pub root: SaltWidgetLayout,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub enum SaltWidgetLayout {
    Group {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        separator: bool,
        children: Vec<SaltWidgetLayout>,
    },
    Row {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        children: Vec<SaltWidgetLayout>,
    },
    Button(LayoutBox, String),
    TextInput(LayoutBox, String),
    Label(LayoutBox, String),
    Checkbox(LayoutBox, String, bool),
    Radio(LayoutBox, String, bool),
    Dropdown(LayoutBox, Vec<String>),
    TreeNode(LayoutBox, String, usize),
    Separator(LayoutBox),
    Table {
        rect: LayoutBox,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        col_widths: Vec<f64>,
        row_height: f64,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct LayoutBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

const MARGIN: f64 = 20.0;
const PAD_H: f64 = 10.0;
const PAD_V: f64 = 8.0;
const GAP: f64 = 10.0;
const FONT_SIZE: f64 = 12.0;
const LINE_HEIGHT: f64 = 16.0;
const CONTROL_HEIGHT: f64 = 28.0;

pub fn layout_salt(diagram: &SaltDiagram) -> Result<SaltLayout> {
    let (root, width, height) = layout_widget(&diagram.root, MARGIN, MARGIN);
    Ok(SaltLayout {
        root,
        width: width + MARGIN,
        height: height + MARGIN,
    })
}

fn layout_widget(widget: &SaltWidget, x: f64, y: f64) -> (SaltWidgetLayout, f64, f64) {
    match widget {
        SaltWidget::Group {
            children,
            separator,
        } => layout_group(children, *separator, x, y),
        SaltWidget::Row(children) => layout_row(children, x, y),
        SaltWidget::Button(text) => simple_box(
            x,
            y,
            text_width(text) + 2.0 * PAD_H,
            CONTROL_HEIGHT,
            |rect| SaltWidgetLayout::Button(rect, text.clone()),
        ),
        SaltWidget::TextInput(text) => simple_box(
            x,
            y,
            (text_width(text) + 2.0 * PAD_H + 24.0).max(90.0),
            CONTROL_HEIGHT,
            |rect| SaltWidgetLayout::TextInput(rect, text.clone()),
        ),
        SaltWidget::Label(text) => {
            simple_box(x, y, text_width(text).max(10.0), LINE_HEIGHT, |rect| {
                SaltWidgetLayout::Label(rect, text.clone())
            })
        }
        SaltWidget::Checkbox { label, checked } => {
            simple_box(x, y, text_width(label) + 28.0, CONTROL_HEIGHT, |rect| {
                SaltWidgetLayout::Checkbox(rect, label.clone(), *checked)
            })
        }
        SaltWidget::Radio { label, selected } => {
            simple_box(x, y, text_width(label) + 28.0, CONTROL_HEIGHT, |rect| {
                SaltWidgetLayout::Radio(rect, label.clone(), *selected)
            })
        }
        SaltWidget::Dropdown { items } => {
            let text = items.first().cloned().unwrap_or_default();
            simple_box(
                x,
                y,
                (text_width(&text) + 2.0 * PAD_H + 20.0).max(90.0),
                CONTROL_HEIGHT,
                |rect| SaltWidgetLayout::Dropdown(rect, items.clone()),
            )
        }
        SaltWidget::TreeNode { label, depth } => simple_box(
            x + *depth as f64 * 14.0,
            y,
            text_width(label) + 18.0,
            LINE_HEIGHT,
            |rect| SaltWidgetLayout::TreeNode(rect, label.clone(), *depth),
        ),
        SaltWidget::Separator => simple_box(x, y, 120.0, 8.0, SaltWidgetLayout::Separator),
        SaltWidget::Table { headers, rows } => layout_table(headers, rows, x, y),
    }
}

fn layout_group(
    children: &[SaltWidget],
    separator: bool,
    x: f64,
    y: f64,
) -> (SaltWidgetLayout, f64, f64) {
    let mut child_layouts = Vec::new();
    let mut y_cursor = y + PAD_V;
    let mut max_child_width: f64 = 0.0;

    for (idx, child) in children.iter().enumerate() {
        let (layout, width, height) = layout_widget(child, x + PAD_H, y_cursor);
        child_layouts.push(layout);
        max_child_width = max_child_width.max(width);
        y_cursor += height;
        if idx + 1 < children.len() {
            y_cursor += GAP;
        }
    }

    let width = max_child_width + 2.0 * PAD_H;
    let height = (y_cursor - y).max(CONTROL_HEIGHT) + PAD_V;
    (
        SaltWidgetLayout::Group {
            x,
            y,
            width,
            height,
            separator,
            children: child_layouts,
        },
        width,
        height,
    )
}

fn layout_row(children: &[SaltWidget], x: f64, y: f64) -> (SaltWidgetLayout, f64, f64) {
    let mut child_layouts = Vec::new();
    let mut x_cursor = x;
    let mut max_height: f64 = 0.0;

    for (idx, child) in children.iter().enumerate() {
        let (layout, width, height) = layout_widget(child, x_cursor, y);
        child_layouts.push(layout);
        x_cursor += width;
        if idx + 1 < children.len() {
            x_cursor += GAP;
        }
        max_height = max_height.max(height);
    }

    let width = (x_cursor - x).max(0.0);
    (
        SaltWidgetLayout::Row {
            x,
            y,
            width,
            height: max_height,
            children: child_layouts,
        },
        width,
        max_height,
    )
}

fn layout_table(
    headers: &[String],
    rows: &[Vec<String>],
    x: f64,
    y: f64,
) -> (SaltWidgetLayout, f64, f64) {
    let col_count = headers
        .len()
        .max(rows.iter().map(std::vec::Vec::len).max().unwrap_or(0));
    let mut col_widths: Vec<f64> = vec![60.0; col_count];
    for (idx, header) in headers.iter().enumerate() {
        col_widths[idx] = col_widths[idx].max(text_width(header) + 2.0 * PAD_H);
    }
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            col_widths[idx] = col_widths[idx].max(text_width(cell) + 2.0 * PAD_H);
        }
    }

    let width = col_widths.iter().sum::<f64>();
    let row_height = CONTROL_HEIGHT;
    let height = row_height * (rows.len() + 1).max(1) as f64;

    (
        SaltWidgetLayout::Table {
            rect: LayoutBox {
                x,
                y,
                width,
                height,
            },
            headers: headers.to_vec(),
            rows: rows.to_vec(),
            col_widths,
            row_height,
        },
        width,
        height,
    )
}

fn simple_box<F>(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    builder: F,
) -> (SaltWidgetLayout, f64, f64)
where
    F: FnOnce(LayoutBox) -> SaltWidgetLayout,
{
    let rect = LayoutBox {
        x,
        y,
        width,
        height,
    };
    (builder(rect), width, height)
}

fn text_width(text: &str) -> f64 {
    text.lines()
        .map(|line| font_metrics::text_width(line, "SansSerif", FONT_SIZE, false, false))
        .fold(0.0_f64, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::salt::{SaltDiagram, SaltWidget};

    #[test]
    fn layout_group_stack() {
        let diagram = SaltDiagram {
            root: SaltWidget::Group {
                children: vec![
                    SaltWidget::Button("OK".to_string()),
                    SaltWidget::Button("Cancel".to_string()),
                ],
                separator: false,
            },
            is_inline: false,
        };
        let layout = layout_salt(&diagram).unwrap();
        match layout.root {
            SaltWidgetLayout::Group { children, .. } => assert_eq!(children.len(), 2),
            other => panic!("unexpected root layout: {:?}", other),
        }
    }

    #[test]
    fn layout_table_has_columns() {
        let diagram = SaltDiagram {
            root: SaltWidget::Table {
                headers: vec!["Name".to_string(), "Age".to_string()],
                rows: vec![vec!["Alice".to_string(), "30".to_string()]],
            },
            is_inline: false,
        };
        let layout = layout_salt(&diagram).unwrap();
        match layout.root {
            SaltWidgetLayout::Table { col_widths, .. } => assert_eq!(col_widths.len(), 2),
            other => panic!("unexpected table layout: {:?}", other),
        }
    }
}
