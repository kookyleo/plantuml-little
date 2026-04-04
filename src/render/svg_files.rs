use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::files_diagram::{FilesEntryLayout, FilesLayout};
use crate::model::files_diagram::{FilesDiagram, FilesEntryKind};
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;

const FILE_ICON_COLOR: &str = "#909090";

pub fn render_files(_d: &FilesDiagram, layout: &FilesLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let sw = ensure_visible_int(layout.width) as f64;
    let sh = ensure_visible_int(layout.height) as f64;
    write_svg_root_bg(&mut buf, sw, sh, "FILES", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, sw, sh, bg);
    let fc = skin.font_color("files", "#333333");
    let mut sg = SvgGraphic::new(0, 1.0);
    for e in &layout.entries {
        render_entry(&mut sg, e, fc);
    }
    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_entry(sg: &mut SvgGraphic, e: &FilesEntryLayout, fc: &str) {
    match e.kind {
        FilesEntryKind::Folder => {
            sg.set_fill_color("#F0C040");
            sg.set_stroke_color(Some("#C09030"));
            sg.set_stroke_width(0.5, None);
            sg.svg_rectangle(e.x, e.y + 3.0, 14.0, 10.0, 1.0, 1.0, 0.0);
            sg.svg_rectangle(e.x, e.y + 1.0, 6.0, 3.0, 0.5, 0.5, 0.0);
        }
        FilesEntryKind::File => {
            sg.set_fill_color("#FFFFFF");
            sg.set_stroke_color(Some(FILE_ICON_COLOR));
            sg.set_stroke_width(0.5, None);
            sg.svg_rectangle(e.x + 1.0, e.y + 1.0, 10.0, 14.0, 0.0, 0.0, 0.0);
            // Corner fold triangle
            let fx = e.x + 8.0;
            let fy1 = e.y + 1.0;
            let fy2 = e.y + 4.0;
            let fx2 = e.x + 11.0;
            sg.push_raw(&format!(
                "<path d=\"M{},{} L{},{} L{},{} Z\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.5\"/>",
                fx, fy1, fx, fy2, fx2, fy2, FILE_ICON_COLOR, FILE_ICON_COLOR
            ));
        }
    }
    let tl = font_metrics::text_width(&e.name, "SansSerif", 14.0, false, false);
    sg.set_fill_color(fc);
    sg.svg_text(
        &e.name, e.x + 20.0, e.y + 12.9, Some("sans-serif"), 14.0,
        None, None, None, tl, LengthAdjust::Spacing, None, 0, None,
    );
}
