use crate::font_metrics;
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::chart::{ChartLayout};
use crate::model::chart::ChartDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg};
use crate::style::SkinParams;
use crate::Result;
const FS: f64 = 12.0;
const COLORS: &[&str] = &["#4E79A7","#F28E2B","#E15759","#76B7B2","#59A14F","#EDC948","#B07AA1","#FF9DA7","#9C755F","#BAB0AC"];
pub fn render_chart(_d: &ChartDiagram, l: &ChartLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let (sw, sh) = (ensure_visible_int(l.width) as f64, ensure_visible_int(l.height) as f64);
    write_svg_root_bg(&mut buf, sw, sh, "CHART", bg);
    buf.push_str("<defs/><g>"); write_bg_rect(&mut buf, sw, sh, bg);
    let mut sg = SvgGraphic::new(0, 1.0);
    sg.set_stroke_color(Some("#E0E0E0")); sg.set_stroke_width(0.5, None);
    for i in 1..5 { let y = l.plot_y + l.plot_height * (1.0 - i as f64 / 5.0); sg.svg_line(l.plot_x, y, l.plot_x + l.plot_width, y, 0.0); }
    sg.set_stroke_color(Some("#333333")); sg.set_stroke_width(1.0, None);
    let bot = l.plot_y + l.plot_height;
    sg.svg_line(l.plot_x, bot, l.plot_x + l.plot_width, bot, 0.0);
    sg.svg_line(l.plot_x, l.plot_y, l.plot_x, bot, 0.0);
    for bar in &l.bars { if bar.height <= 0.0 { continue; } let c = COLORS[bar.series_index % COLORS.len()]; sg.set_fill_color(c); sg.set_stroke_color(Some(c)); sg.set_stroke_width(0.5, None); sg.svg_rectangle(bar.x, bar.y, bar.width, bar.height, 0.0, 0.0, 0.0); }
    for (label, cx) in &l.x_label_positions { let tw = font_metrics::text_width(label, "SansSerif", FS, false, false); sg.set_fill_color("#333333"); sg.svg_text(label, cx-tw/2.0, bot+15.0, Some("sans-serif"), FS, None, None, None, tw, LengthAdjust::Spacing, None, 0, None); }
    for i in 0..=5 { let f = i as f64/5.0; let v = l.y_max*f; let y = l.plot_y+l.plot_height*(1.0-f); let s = if v==v.floor() { format!("{:.0}",v) } else { format!("{:.1}",v) }; let tw = font_metrics::text_width(&s, "SansSerif", FS, false, false); sg.set_fill_color("#333333"); sg.svg_text(&s, l.plot_x-tw-5.0, y+5.5, Some("sans-serif"), FS, None, None, None, tw, LengthAdjust::Spacing, None, 0, None); }
    let ly = l.plot_y+l.plot_height+35.0; let mut lx = l.plot_x;
    for (i, label) in l.series_labels.iter().enumerate() { let c = COLORS[i%COLORS.len()]; sg.set_fill_color(c); sg.set_stroke_color(None); sg.set_stroke_width(0.0, None); sg.svg_rectangle(lx, ly, 12.0, 12.0, 0.0, 0.0, 0.0); let tw = font_metrics::text_width(label, "SansSerif", FS, false, false); sg.set_fill_color("#333333"); sg.svg_text(label, lx+16.0, ly+11.0, Some("sans-serif"), FS, None, None, None, tw, LengthAdjust::Spacing, None, 0, None); lx += 16.0+tw+20.0; }
    buf.push_str(sg.body()); buf.push_str("</g></svg>"); Ok(buf)
}
