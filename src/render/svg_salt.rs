use std::fmt::Write;

use crate::layout::salt::{LayoutBox, SaltLayout, SaltWidgetLayout};
use crate::model::salt::SaltDiagram;
use crate::render::svg::xml_escape;
use crate::render::svg::write_svg_root;
use crate::style::SkinParams;
use crate::Result;

const BG: &str = "#FFFFFF";
const BORDER: &str = "#181818";
const FILL: &str = "#F1F1F1";
const TEXT: &str = "#000000";

pub fn render_salt(
    _diagram: &SaltDiagram,
    layout: &SaltLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    write_svg_root(&mut buf, layout.width, layout.height);
    buf.push_str("<defs/><g>");

    let border = skin.border_color("salt", BORDER);
    let fill = skin.background_color("salt", FILL);
    let font = skin.font_color("salt", TEXT);
    write!(
        buf,
        r#"<rect fill="{}" height="{:.0}" width="{:.0}" x="0" y="0"/>"#,
        layout.width,
        layout.height,
        skin.background_color("saltbg", BG)
    )
    .unwrap();
    buf.push('\n');

    render_widget(&mut buf, &layout.root, fill, border, font);

    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_widget(
    buf: &mut String,
    widget: &SaltWidgetLayout,
    fill: &str,
    border: &str,
    font: &str,
) {
    match widget {
        SaltWidgetLayout::Group {
            x,
            y,
            width,
            height,
            separator,
            children,
        } => {
            write!(
                buf,
                r#"<rect fill="none" height="{height:.1}" rx="6" ry="6" style="stroke:{border};stroke-width:1;" width="{width:.1}" x="{x:.1}" y="{y:.1}"/>"#,
            )
            .unwrap();
            buf.push('\n');
            for child in children {
                render_widget(buf, child, fill, border, font);
            }
            if *separator && children.len() > 1 {
                for child in children.iter().take(children.len() - 1) {
                    let y = child_bounds(child).y + child_bounds(child).height + 5.0;
                    write!(
                        buf,
                        r#"<line style="stroke:{border};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y:.1}" y2="{y:.1}"/>"#,
                        x1 = x + 8.0,
                        x2 = x + width - 8.0,
                    )
                    .unwrap();
                    buf.push('\n');
                }
            }
        }
        SaltWidgetLayout::Row { children, .. } => {
            for child in children {
                render_widget(buf, child, fill, border, font);
            }
        }
        SaltWidgetLayout::Button(rect, text) => {
            render_boxed_text(buf, rect, text, fill, border, font, 6.0);
        }
        SaltWidgetLayout::TextInput(rect, text) => {
            render_boxed_text(buf, rect, text, "#FFFFFF", border, font, 4.0);
        }
        SaltWidgetLayout::Label(rect, text) => {
            render_text(buf, rect.x, rect.y + 12.0, text, font, None);
        }
        SaltWidgetLayout::Checkbox(rect, label, checked) => {
            render_checkbox(buf, rect, label, *checked, border, font);
        }
        SaltWidgetLayout::Radio(rect, label, selected) => {
            render_radio(buf, rect, label, *selected, border, font);
        }
        SaltWidgetLayout::Dropdown(rect, items) => {
            render_dropdown(buf, rect, items, fill, border, font);
        }
        SaltWidgetLayout::TreeNode(rect, label, _) => {
            write!(
                buf,
                r#"<circle cx="{:.1}" cy="{:.1}" fill="{}" r="3"/>"#,
                rect.x + 6.0,
                rect.y + rect.height / 2.0,
                border
            )
            .unwrap();
            buf.push('\n');
            render_text(buf, rect.x + 14.0, rect.y + 12.0, label, font, None);
        }
        SaltWidgetLayout::Separator(rect) => {
            write!(
                buf,
                r#"<line style="stroke:{border};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y:.1}" y2="{y:.1}"/>"#,
                x1 = rect.x,
                x2 = rect.x + rect.width,
                y = rect.y + rect.height / 2.0,
            )
            .unwrap();
            buf.push('\n');
        }
        SaltWidgetLayout::Table {
            rect,
            headers,
            rows,
            col_widths,
            row_height,
        } => render_table(
            buf,
            rect,
            headers,
            rows,
            col_widths,
            *row_height,
            fill,
            border,
            font,
        ),
    }
}

fn render_boxed_text(
    buf: &mut String,
    rect: &LayoutBox,
    text: &str,
    fill: &str,
    border: &str,
    font: &str,
    radius: f64,
) {
    write!(
        buf,
        r#"<rect fill="{fill}" height="{h:.1}" rx="{r:.1}" ry="{r:.1}" style="stroke:{border};stroke-width:1;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = rect.x,
        y = rect.y,
        w = rect.width,
        h = rect.height,
        r = radius,
    )
    .unwrap();
    buf.push('\n');
    render_text(
        buf,
        rect.x + rect.width / 2.0,
        rect.y + rect.height / 2.0 + 4.0,
        text,
        font,
        Some("middle"),
    );
}

fn render_checkbox(
    buf: &mut String,
    rect: &LayoutBox,
    label: &str,
    checked: bool,
    border: &str,
    font: &str,
) {
    write!(
        buf,
        r##"<rect fill="#FFFFFF" height="14" style="stroke:{};stroke-width:1;" width="14" x="{:.1}" y="{:.1}"/>"##,
        border,
        rect.x,
        rect.y + 7.0,
    )
    .unwrap();
    buf.push('\n');
    if checked {
        write!(
            buf,
            r#"<path d="M {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1}" fill="none" style="stroke:{};stroke-width:1.5;"/>"#,
            rect.x + 3.0,
            rect.y + 14.0,
            rect.x + 6.0,
            rect.y + 18.0,
            rect.x + 11.0,
            rect.y + 9.0,
            border,
        )
        .unwrap();
        buf.push('\n');
    }
    render_text(buf, rect.x + 22.0, rect.y + 18.0, label, font, None);
}

fn render_radio(
    buf: &mut String,
    rect: &LayoutBox,
    label: &str,
    selected: bool,
    border: &str,
    font: &str,
) {
    write!(
        buf,
        r##"<circle cx="{:.1}" cy="{:.1}" fill="#FFFFFF" r="7" style="stroke:{};stroke-width:1;"/>"##,
        rect.x + 7.0,
        rect.y + 14.0,
        border,
    )
    .unwrap();
    buf.push('\n');
    if selected {
        write!(
            buf,
            r#"<circle cx="{:.1}" cy="{:.1}" fill="{}" r="3"/>"#,
            rect.x + 7.0,
            rect.y + 14.0,
            border,
        )
        .unwrap();
        buf.push('\n');
    }
    render_text(buf, rect.x + 22.0, rect.y + 18.0, label, font, None);
}

fn render_dropdown(
    buf: &mut String,
    rect: &LayoutBox,
    items: &[String],
    fill: &str,
    border: &str,
    font: &str,
) {
    let text = items.first().cloned().unwrap_or_default();
    render_boxed_text(buf, rect, &text, fill, border, font, 4.0);
    write!(
        buf,
        r#"<path d="M {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1} Z" fill="{}"/>"#,
        rect.x + rect.width - 16.0,
        rect.y + rect.height / 2.0 - 3.0,
        rect.x + rect.width - 8.0,
        rect.y + rect.height / 2.0 - 3.0,
        rect.x + rect.width - 12.0,
        rect.y + rect.height / 2.0 + 3.0,
        border,
    )
    .unwrap();
    buf.push('\n');
}

#[allow(clippy::too_many_arguments)]
fn render_table(
    buf: &mut String,
    rect: &LayoutBox,
    headers: &[String],
    rows: &[Vec<String>],
    col_widths: &[f64],
    row_height: f64,
    fill: &str,
    border: &str,
    font: &str,
) {
    write!(
        buf,
        r#"<rect fill="{fill}" height="{h:.1}" style="stroke:{border};stroke-width:1;" width="{w:.1}" x="{x:.1}" y="{y:.1}"/>"#,
        x = rect.x,
        y = rect.y,
        w = rect.width,
        h = rect.height,
    )
    .unwrap();
    buf.push('\n');

    let mut x_cursor = rect.x;
    for width in col_widths.iter().take(col_widths.len().saturating_sub(1)) {
        x_cursor += *width;
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:1;" x1="{x:.1}" x2="{x:.1}" y1="{y1:.1}" y2="{y2:.1}"/>"#,
            x = x_cursor,
            y1 = rect.y,
            y2 = rect.y + rect.height,
        )
        .unwrap();
        buf.push('\n');
    }
    for row in 1..=rows.len() {
        let y = rect.y + row as f64 * row_height;
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:1;" x1="{x1:.1}" x2="{x2:.1}" y1="{y:.1}" y2="{y:.1}"/>"#,
            x1 = rect.x,
            x2 = rect.x + rect.width,
        )
        .unwrap();
        buf.push('\n');
    }

    let mut cell_x = rect.x;
    for (idx, header) in headers.iter().enumerate() {
        render_text(
            buf,
            cell_x + 8.0,
            rect.y + row_height / 2.0 + 4.0,
            header,
            font,
            None,
        );
        cell_x += col_widths[idx];
    }
    for (row_idx, row) in rows.iter().enumerate() {
        let mut cell_x = rect.x;
        for (col_idx, cell) in row.iter().enumerate() {
            render_text(
                buf,
                cell_x + 8.0,
                rect.y + (row_idx as f64 + 1.5) * row_height + 4.0,
                cell,
                font,
                None,
            );
            cell_x += col_widths[col_idx];
        }
    }
}

fn render_text(buf: &mut String, x: f64, y: f64, text: &str, font: &str, anchor: Option<&str>) {
    write!(buf, r#"<text x="{x:.1}" y="{y:.1}""#).unwrap();
    if let Some(anchor) = anchor {
        write!(buf, r#" text-anchor="{}""#, xml_escape(anchor)).unwrap();
    }
    write!(buf, r#" fill="{}">{}"#, font, xml_escape(text)).unwrap();
    buf.push_str("</text>\n");
}

fn child_bounds(widget: &SaltWidgetLayout) -> LayoutBox {
    match widget {
        SaltWidgetLayout::Group {
            x,
            y,
            width,
            height,
            ..
        }
        | SaltWidgetLayout::Row {
            x,
            y,
            width,
            height,
            ..
        } => LayoutBox {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
        },
        SaltWidgetLayout::Button(rect, _)
        | SaltWidgetLayout::TextInput(rect, _)
        | SaltWidgetLayout::Label(rect, _)
        | SaltWidgetLayout::Checkbox(rect, ..)
        | SaltWidgetLayout::Radio(rect, ..)
        | SaltWidgetLayout::Dropdown(rect, _)
        | SaltWidgetLayout::TreeNode(rect, ..)
        | SaltWidgetLayout::Separator(rect) => *rect,
        SaltWidgetLayout::Table { rect, .. } => *rect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::salt::{LayoutBox, SaltLayout, SaltWidgetLayout};
    use crate::style::SkinParams;

    #[test]
    fn render_simple_button() {
        let layout = SaltLayout {
            root: SaltWidgetLayout::Button(
                LayoutBox {
                    x: 20.0,
                    y: 20.0,
                    width: 80.0,
                    height: 28.0,
                },
                "OK".to_string(),
            ),
            width: 140.0,
            height: 80.0,
        };
        let svg = render_salt(
            &SaltDiagram {
                root: crate::model::salt::SaltWidget::Button("OK".to_string()),
            },
            &layout,
            &SkinParams::default(),
        )
        .unwrap();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("OK"));
    }
}
