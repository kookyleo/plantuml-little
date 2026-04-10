//! Salt SVG renderer — takes the flat draw command list produced by
//! `layout::salt` and emits exactly the SVG Java PlantUML emits for salt
//! diagrams (text + line + rect + ellipse + polygon only).

use crate::klimt::drawable::{
    DrawStyle, Drawable, EllipseShape, LineShape, PolygonShape, RectShape, TextShape,
};
use crate::klimt::svg::SvgGraphic;
use crate::layout::salt::{DrawCmd, SaltLayout};
use crate::model::salt::SaltDiagram;
use crate::render::svg::write_svg_root_bg_opt;
use crate::style::SkinParams;
use crate::Result;

pub fn render_salt(
    diagram: &SaltDiagram,
    layout: &SaltLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let svg_w = layout.width;
    let svg_h = layout.height;
    // Java PSystemSalt always emits `data-diagram-type="SALT"` whether the
    // diagram is standalone (`@startsalt`) or inline inside `@startuml`.
    let _ = diagram; // is_inline is parsing metadata, unused at render time
    write_svg_root_bg_opt(&mut buf, svg_w, svg_h, Some("SALT"), bg);
    buf.push_str("<defs/><g>");

    let mut sg = SvgGraphic::new(0, 1.0);
    for cmd in &layout.commands {
        emit_command(&mut sg, cmd);
    }
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn emit_command(sg: &mut SvgGraphic, cmd: &DrawCmd) {
    match cmd {
        DrawCmd::Text {
            x,
            y,
            text,
            text_length,
        } => {
            let style = DrawStyle {
                fill: Some("#000000".into()),
                stroke: None,
                stroke_width: 0.0,
                dash_array: None,
                delta_shadow: 0.0,
            };
            TextShape {
                x: *x,
                y: *y,
                text: text.clone(),
                font_family: "sans-serif".into(),
                font_size: 12.0,
                text_length: *text_length,
                bold: false,
                italic: false,
            }
            .draw(sg, &style);
        }
        DrawCmd::Line { x1, y1, x2, y2 } => {
            LineShape {
                x1: *x1,
                y1: *y1,
                x2: *x2,
                y2: *y2,
            }
            .draw(sg, &DrawStyle::outline("#000000", 1.0));
        }
        DrawCmd::RectOutline {
            x,
            y,
            w,
            h,
            stroke_width,
        } => {
            RectShape {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                rx: 0.0,
                ry: 0.0,
            }
            .draw(sg, &DrawStyle::outline("#000000", *stroke_width));
        }
        DrawCmd::RectFilled {
            x,
            y,
            w,
            h,
            rx,
            fill,
            stroke_width,
        } => {
            RectShape {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                rx: *rx,
                ry: *rx,
            }
            .draw(sg, &DrawStyle::filled(fill, "#000000", *stroke_width));
        }
        DrawCmd::Ellipse {
            cx,
            cy,
            rx,
            ry,
            stroke_width,
        } => {
            EllipseShape {
                cx: *cx,
                cy: *cy,
                rx: *rx,
                ry: *ry,
            }
            .draw(sg, &DrawStyle::outline("#000000", *stroke_width));
        }
        DrawCmd::EllipseFilled {
            cx,
            cy,
            rx,
            ry,
            stroke_width,
        } => {
            EllipseShape {
                cx: *cx,
                cy: *cy,
                rx: *rx,
                ry: *ry,
            }
            .draw(sg, &DrawStyle::filled("#000000", "#000000", *stroke_width));
        }
        DrawCmd::Polygon {
            points,
            stroke_width,
        } => {
            let flat: Vec<f64> = points.iter().flat_map(|(x, y)| [*x, *y]).collect();
            PolygonShape { points: flat }
                .draw(sg, &DrawStyle::filled("#000000", "#000000", *stroke_width));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::salt::{SaltDiagram, SaltElement, SaltPyramid, TableStrategy};
    use crate::parser::salt::parse_salt_diagram;
    use crate::style::SkinParams;

    #[test]
    fn renders_single_button() {
        let src = "@startsalt\n{\n[Cancel]\n}\n@endsalt";
        let diag = parse_salt_diagram(src).unwrap();
        let layout = crate::layout::salt::layout_salt(&diag).unwrap();
        let svg = render_salt(&diag, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("width=\"66px\""));
        assert!(svg.contains("Cancel"));
        // No background rect for default white bg
        assert!(!svg.contains("<rect fill=\"#FFFFFF\""));
    }

    #[test]
    fn renders_single_title() {
        let src = "@startsalt\n{\nTitle\n}\n@endsalt";
        let diag = parse_salt_diagram(src).unwrap();
        let layout = crate::layout::salt::layout_salt(&diag).unwrap();
        let svg = render_salt(&diag, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("width=\"39px\""));
        assert!(svg.contains("Title"));
    }

    #[test]
    fn renders_empty_pyramid_safely() {
        // Defensive check: an empty pyramid should still produce a valid SVG.
        let diag = SaltDiagram {
            root: SaltElement::Pyramid(SaltPyramid {
                cells: vec![],
                rows: 1,
                cols: 1,
                strategy: TableStrategy::DrawNone,
            }),
            is_inline: false,
        };
        let layout = crate::layout::salt::layout_salt(&diag).unwrap();
        let svg = render_salt(&diag, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
    }
}
