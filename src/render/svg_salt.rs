use crate::font_metrics;
use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
use crate::layout::salt::{LayoutBox, SaltLayout, SaltWidgetLayout};
use crate::model::salt::SaltDiagram;
use crate::render::svg::{write_bg_rect, write_svg_root_bg_opt, ensure_visible_int};
use crate::style::SkinParams;
use crate::Result;

use crate::skin::rose::{ACTIVATION_BG, BORDER_COLOR, ENTITY_BG, TEXT_COLOR};
const STROKE_WIDTH: f64 = 0.5;

pub fn render_salt(
    diagram: &SaltDiagram,
    layout: &SaltLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = ensure_visible_int(layout.width) as f64;
    let svg_h = ensure_visible_int(layout.height) as f64;
    // Java PSystemSalt via @startsalt emits data-diagram-type="SALT",
    // but inline salt inside @startuml does not (different code path)
    let dtype = if diagram.is_inline { None } else { Some("SALT") };
    write_svg_root_bg_opt(&mut buf, svg_w, svg_h, dtype, bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, svg_w, svg_h, bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    let border = skin.border_color("salt", BORDER_COLOR);
    let fill = skin.background_color("salt", ENTITY_BG);
    let font = skin.font_color("salt", TEXT_COLOR);

    // Background rect (no stroke)
    sg.set_fill_color(&skin.background_color("saltbg", ACTIVATION_BG));
    sg.set_stroke_width(0.0, None);
    sg.svg_rectangle(0.0, 0.0, layout.width, layout.height, 0.0, 0.0, 0.0);
    sg.push_raw("\n");

    render_widget(&mut sg, &layout.root, fill, border, font);

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_widget(
    sg: &mut SvgGraphic,
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
            sg.set_fill_color("none");
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(STROKE_WIDTH, None);
            sg.svg_rectangle(*x, *y, *width, *height, 6.0, 6.0, 0.0);
            sg.push_raw("\n");
            for child in children {
                render_widget(sg, child, fill, border, font);
            }
            if *separator && children.len() > 1 {
                for child in children.iter().take(children.len() - 1) {
                    let sep_y = child_bounds(child).y + child_bounds(child).height + 5.0;
                    sg.set_stroke_color(Some(border));
                    sg.set_stroke_width(STROKE_WIDTH, None);
                    sg.svg_line(x + 8.0, sep_y, x + width - 8.0, sep_y, 0.0);
                    sg.push_raw("\n");
                }
            }
        }
        SaltWidgetLayout::Row { children, .. } => {
            for child in children {
                render_widget(sg, child, fill, border, font);
            }
        }
        SaltWidgetLayout::Button(rect, text) => {
            render_boxed_text(sg, rect, text, fill, border, font, 6.0);
        }
        SaltWidgetLayout::TextInput(rect, text) => {
            render_boxed_text(sg, rect, text, "#FFFFFF", border, font, 4.0);
        }
        SaltWidgetLayout::Label(rect, text) => {
            render_text(sg, rect.x, rect.y + 12.0, text, font, None);
        }
        SaltWidgetLayout::Checkbox(rect, label, checked) => {
            render_checkbox(sg, rect, label, *checked, border, font);
        }
        SaltWidgetLayout::Radio(rect, label, selected) => {
            render_radio(sg, rect, label, *selected, border, font);
        }
        SaltWidgetLayout::Dropdown(rect, items) => {
            render_dropdown(sg, rect, items, fill, border, font);
        }
        SaltWidgetLayout::TreeNode(rect, label, _) => {
            sg.set_fill_color(border);
            sg.set_stroke_width(0.0, None);
            sg.svg_circle(rect.x + 6.0, rect.y + rect.height / 2.0, 3.0, 0.0);
            sg.push_raw("\n");
            render_text(sg, rect.x + 14.0, rect.y + 12.0, label, font, None);
        }
        SaltWidgetLayout::Separator(rect) => {
            let sep_y = rect.y + rect.height / 2.0;
            sg.set_stroke_color(Some(border));
            sg.set_stroke_width(STROKE_WIDTH, None);
            sg.svg_line(rect.x, sep_y, rect.x + rect.width, sep_y, 0.0);
            sg.push_raw("\n");
        }
        SaltWidgetLayout::Table {
            rect,
            headers,
            rows,
            col_widths,
            row_height,
        } => render_table(
            sg,
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
    sg: &mut SvgGraphic,
    rect: &LayoutBox,
    text: &str,
    fill: &str,
    border: &str,
    font: &str,
    radius: f64,
) {
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(STROKE_WIDTH, None);
    sg.svg_rectangle(rect.x, rect.y, rect.width, rect.height, radius, radius, 0.0);
    sg.push_raw("\n");
    render_text(
        sg,
        rect.x + rect.width / 2.0,
        rect.y + rect.height / 2.0 + 4.0,
        text,
        font,
        Some("middle"),
    );
}

fn render_checkbox(
    sg: &mut SvgGraphic,
    rect: &LayoutBox,
    label: &str,
    checked: bool,
    border: &str,
    font: &str,
) {
    sg.set_fill_color("#FFFFFF");
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(STROKE_WIDTH, None);
    sg.svg_rectangle(rect.x, rect.y + 7.0, 14.0, 14.0, 0.0, 0.0, 0.0);
    sg.push_raw("\n");
    if checked {
        sg.push_raw(&format!(
            r#"<path d="M{},{} L{},{} L{},{} " fill="none" style="stroke:{};stroke-width:0.5;"/>"#,
            fmt_coord(rect.x + 3.0),
            fmt_coord(rect.y + 14.0),
            fmt_coord(rect.x + 6.0),
            fmt_coord(rect.y + 18.0),
            fmt_coord(rect.x + 11.0),
            fmt_coord(rect.y + 9.0),
            border,
        ));
        sg.push_raw("\n");
    }
    render_text(sg, rect.x + 22.0, rect.y + 18.0, label, font, None);
}

fn render_radio(
    sg: &mut SvgGraphic,
    rect: &LayoutBox,
    label: &str,
    selected: bool,
    border: &str,
    font: &str,
) {
    sg.set_fill_color("#FFFFFF");
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(STROKE_WIDTH, None);
    sg.svg_circle(rect.x + 7.0, rect.y + 14.0, 7.0, 0.0);
    sg.push_raw("\n");
    if selected {
        sg.set_fill_color(border);
        sg.set_stroke_width(0.0, None);
        sg.svg_circle(rect.x + 7.0, rect.y + 14.0, 3.0, 0.0);
        sg.push_raw("\n");
    }
    render_text(sg, rect.x + 22.0, rect.y + 18.0, label, font, None);
}

fn render_dropdown(
    sg: &mut SvgGraphic,
    rect: &LayoutBox,
    items: &[String],
    fill: &str,
    border: &str,
    font: &str,
) {
    let text = items.first().cloned().unwrap_or_default();
    render_boxed_text(sg, rect, &text, fill, border, font, 4.0);
    sg.push_raw(&format!(
        r#"<path d="M{},{} L{},{} L{},{} Z " fill="{}"/>"#,
        fmt_coord(rect.x + rect.width - 16.0),
        fmt_coord(rect.y + rect.height / 2.0 - 3.0),
        fmt_coord(rect.x + rect.width - 8.0),
        fmt_coord(rect.y + rect.height / 2.0 - 3.0),
        fmt_coord(rect.x + rect.width - 12.0),
        fmt_coord(rect.y + rect.height / 2.0 + 3.0),
        border,
    ));
    sg.push_raw("\n");
}

#[allow(clippy::too_many_arguments)]
fn render_table(
    sg: &mut SvgGraphic,
    rect: &LayoutBox,
    headers: &[String],
    rows: &[Vec<String>],
    col_widths: &[f64],
    row_height: f64,
    fill: &str,
    border: &str,
    font: &str,
) {
    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(STROKE_WIDTH, None);
    sg.svg_rectangle(rect.x, rect.y, rect.width, rect.height, 0.0, 0.0, 0.0);
    sg.push_raw("\n");

    let mut x_cursor = rect.x;
    for width in col_widths.iter().take(col_widths.len().saturating_sub(1)) {
        x_cursor += *width;
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(STROKE_WIDTH, None);
        sg.svg_line(x_cursor, rect.y, x_cursor, rect.y + rect.height, 0.0);
        sg.push_raw("\n");
    }
    for row_idx in 1..=rows.len() {
        let line_y = rect.y + row_idx as f64 * row_height;
        sg.set_stroke_color(Some(border));
        sg.set_stroke_width(STROKE_WIDTH, None);
        sg.svg_line(rect.x, line_y, rect.x + rect.width, line_y, 0.0);
        sg.push_raw("\n");
    }

    let mut cell_x = rect.x;
    for (idx, header) in headers.iter().enumerate() {
        render_text(
            sg,
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
                sg,
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

fn render_text(sg: &mut SvgGraphic, x: f64, y: f64, text: &str, font: &str, anchor: Option<&str>) {
    let tl = font_metrics::text_width(text, "SansSerif", 12.0, false, false);
    sg.set_fill_color(font);
    sg.set_stroke_width(0.0, None);
    sg.svg_text(
        text,
        x,
        y,
        Some("sans-serif"),
        12.0,
        None,  // font_weight
        None,  // font_style
        None,  // text_decoration
        tl,
        LengthAdjust::Spacing,
        None,  // text_back_color
        0,     // orientation
        anchor,
    );
    sg.push_raw("\n");
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
                is_inline: false,
            },
            &layout,
            &SkinParams::default(),
        )
        .unwrap();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("OK"));
    }
}
