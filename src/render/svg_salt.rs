use std::fmt::Write;

use crate::layout::salt::{LayoutBox, SaltLayout, SaltWidgetLayout};
use crate::model::salt::SaltDiagram;
use crate::render::svg::fmt_coord;
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
    write_svg_root(&mut buf, layout.width, layout.height, "SALT");
    buf.push_str("<defs/><g>");

    let border = skin.border_color("salt", BORDER);
    let fill = skin.background_color("salt", FILL);
    let font = skin.font_color("salt", TEXT);
    write!(
        buf,
        r#"<rect fill="{}" height="{}" width="{}" x="0" y="0"/>"#,
        skin.background_color("saltbg", BG),
        fmt_coord(layout.height),
        fmt_coord(layout.width),
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
                r#"<rect fill="none" height="{}" rx="6" ry="6" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
                fmt_coord(*height), fmt_coord(*width), fmt_coord(*x), fmt_coord(*y),
            )
            .unwrap();
            buf.push('\n');
            for child in children {
                render_widget(buf, child, fill, border, font);
            }
            if *separator && children.len() > 1 {
                for child in children.iter().take(children.len() - 1) {
                    let sep_y = child_bounds(child).y + child_bounds(child).height + 5.0;
                    write!(
                        buf,
                        r#"<line style="stroke:{border};stroke-width:0.5;" x1="{}" x2="{}" y1="{sy}" y2="{sy}"/>"#,
                        fmt_coord(x + 8.0), fmt_coord(x + width - 8.0),
                        sy = fmt_coord(sep_y),
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
                r#"<circle cx="{}" cy="{}" fill="{}" r="3"/>"#,
                fmt_coord(rect.x + 6.0),
                fmt_coord(rect.y + rect.height / 2.0),
                border
            )
            .unwrap();
            buf.push('\n');
            render_text(buf, rect.x + 14.0, rect.y + 12.0, label, font, None);
        }
        SaltWidgetLayout::Separator(rect) => {
            let sep_y = rect.y + rect.height / 2.0;
            write!(
                buf,
                r#"<line style="stroke:{border};stroke-width:0.5;" x1="{}" x2="{}" y1="{sy}" y2="{sy}"/>"#,
                fmt_coord(rect.x), fmt_coord(rect.x + rect.width),
                sy = fmt_coord(sep_y),
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
        r#"<rect fill="{fill}" height="{}" rx="{}" ry="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(rect.height), fmt_coord(radius), fmt_coord(radius),
        fmt_coord(rect.width), fmt_coord(rect.x), fmt_coord(rect.y),
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
        r##"<rect fill="#FFFFFF" height="14" style="stroke:{};stroke-width:0.5;" width="14" x="{}" y="{}"/>"##,
        border,
        fmt_coord(rect.x),
        fmt_coord(rect.y + 7.0),
    )
    .unwrap();
    buf.push('\n');
    if checked {
        write!(
            buf,
            r#"<path d="M {} {} L {} {} L {} {}" fill="none" style="stroke:{};stroke-width:0.5;"/>"#,
            fmt_coord(rect.x + 3.0),
            fmt_coord(rect.y + 14.0),
            fmt_coord(rect.x + 6.0),
            fmt_coord(rect.y + 18.0),
            fmt_coord(rect.x + 11.0),
            fmt_coord(rect.y + 9.0),
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
        r##"<circle cx="{}" cy="{}" fill="#FFFFFF" r="7" style="stroke:{};stroke-width:0.5;"/>"##,
        fmt_coord(rect.x + 7.0),
        fmt_coord(rect.y + 14.0),
        border,
    )
    .unwrap();
    buf.push('\n');
    if selected {
        write!(
            buf,
            r#"<circle cx="{}" cy="{}" fill="{}" r="3"/>"#,
            fmt_coord(rect.x + 7.0),
            fmt_coord(rect.y + 14.0),
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
        r#"<path d="M {} {} L {} {} L {} {} Z" fill="{}"/>"#,
        fmt_coord(rect.x + rect.width - 16.0),
        fmt_coord(rect.y + rect.height / 2.0 - 3.0),
        fmt_coord(rect.x + rect.width - 8.0),
        fmt_coord(rect.y + rect.height / 2.0 - 3.0),
        fmt_coord(rect.x + rect.width - 12.0),
        fmt_coord(rect.y + rect.height / 2.0 + 3.0),
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
        r#"<rect fill="{fill}" height="{}" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(rect.height), fmt_coord(rect.width), fmt_coord(rect.x), fmt_coord(rect.y),
    )
    .unwrap();
    buf.push('\n');

    let mut x_cursor = rect.x;
    for width in col_widths.iter().take(col_widths.len().saturating_sub(1)) {
        x_cursor += *width;
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:0.5;" x1="{xc}" x2="{xc}" y1="{}" y2="{}"/>"#,
            fmt_coord(rect.y), fmt_coord(rect.y + rect.height),
            xc = fmt_coord(x_cursor),
        )
        .unwrap();
        buf.push('\n');
    }
    for row_idx in 1..=rows.len() {
        let line_y = rect.y + row_idx as f64 * row_height;
        write!(
            buf,
            r#"<line style="stroke:{border};stroke-width:0.5;" x1="{}" x2="{}" y1="{ly}" y2="{ly}"/>"#,
            fmt_coord(rect.x), fmt_coord(rect.x + rect.width),
            ly = fmt_coord(line_y),
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
    write!(buf, r#"<text fill="{}" font-family="sans-serif" font-size="12""#, font).unwrap();
    if let Some(anchor) = anchor {
        write!(buf, r#" text-anchor="{}""#, xml_escape(anchor)).unwrap();
    }
    write!(buf, r#" x="{}" y="{}">{}"#, fmt_coord(x), fmt_coord(y), xml_escape(text)).unwrap();
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
