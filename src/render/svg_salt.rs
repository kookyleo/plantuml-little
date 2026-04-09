//! Salt SVG renderer — takes the flat draw command list produced by
//! `layout::salt` and emits exactly the SVG Java PlantUML emits for salt
//! diagrams (text + line + rect + ellipse + polygon only).

use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
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
            sg.set_fill_color("#000000");
            sg.set_stroke_width(0.0, None);
            sg.svg_text(
                text,
                *x,
                *y,
                Some("sans-serif"),
                12.0,
                None,
                None,
                None,
                *text_length,
                LengthAdjust::Spacing,
                None,
                0,
                None,
            );
        }
        DrawCmd::Line { x1, y1, x2, y2 } => {
            sg.set_stroke_color(Some("#000000"));
            sg.set_stroke_width(1.0, None);
            sg.svg_line(*x1, *y1, *x2, *y2, 0.0);
        }
        DrawCmd::RectOutline {
            x,
            y,
            w,
            h,
            stroke_width,
        } => {
            sg.set_fill_color("none");
            sg.set_stroke_color(Some("#000000"));
            sg.set_stroke_width(*stroke_width, None);
            sg.svg_rectangle(*x, *y, *w, *h, 0.0, 0.0, 0.0);
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
            sg.set_fill_color(fill);
            sg.set_stroke_color(Some("#000000"));
            sg.set_stroke_width(*stroke_width, None);
            sg.svg_rectangle(*x, *y, *w, *h, *rx, *rx, 0.0);
        }
        DrawCmd::Ellipse {
            cx,
            cy,
            rx,
            ry,
            stroke_width,
        } => {
            sg.set_fill_color("none");
            sg.set_stroke_color(Some("#000000"));
            sg.set_stroke_width(*stroke_width, None);
            sg.svg_ellipse(*cx, *cy, *rx, *ry, 0.0);
        }
        DrawCmd::EllipseFilled {
            cx,
            cy,
            rx,
            ry,
            stroke_width,
        } => {
            sg.set_fill_color("#000000");
            sg.set_stroke_color(Some("#000000"));
            sg.set_stroke_width(*stroke_width, None);
            sg.svg_ellipse(*cx, *cy, *rx, *ry, 0.0);
        }
        DrawCmd::Polygon {
            points,
            stroke_width,
        } => {
            sg.set_fill_color("#000000");
            sg.set_stroke_color(Some("#000000"));
            sg.set_stroke_width(*stroke_width, None);
            let flat: Vec<f64> = points.iter().flat_map(|(x, y)| [*x, *y]).collect();
            sg.svg_polygon(0.0, &flat);
        }
    }
    // The Java polygon output uses integer-valued points, so fmt_coord suffices.
    let _ = fmt_coord;
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
