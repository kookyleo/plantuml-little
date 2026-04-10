use crate::font_metrics;
use crate::klimt::drawable::{DrawStyle, Drawable, LineShape, RectShape};
use crate::klimt::svg::{LengthAdjust, SvgGraphic};
use crate::layout::chart::ChartLayout;
use crate::model::chart::ChartDiagram;
use crate::render::svg::{ensure_visible_int, write_bg_rect, write_svg_root_bg_opt};
use crate::style::SkinParams;
use crate::Result;
const FS: f64 = 12.0;
const COLORS: &[&str] = &["#4E79A7","#F28E2B","#E15759","#76B7B2","#59A14F","#EDC948","#B07AA1","#FF9DA7","#9C755F","#BAB0AC"];
pub fn render_chart(_d: &ChartDiagram, l: &ChartLayout, skin: &SkinParams) -> Result<String> {
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    let (sw, sh) = (ensure_visible_int(l.width) as f64, ensure_visible_int(l.height) as f64);
    // Java's chart diagram does not emit data-diagram-type
    write_svg_root_bg_opt(&mut buf, sw, sh, None, bg);
    buf.push_str("<defs/><g>"); write_bg_rect(&mut buf, sw, sh, bg);
    let mut sg = SvgGraphic::new(0, 1.0);
    let grid_style = DrawStyle::outline("#E0E0E0", 0.5);
    for i in 1..5 { let y = l.plot_y + l.plot_height * (1.0 - i as f64 / 5.0); LineShape { x1: l.plot_x, y1: y, x2: l.plot_x + l.plot_width, y2: y }.draw(&mut sg, &grid_style); }
    let axis_style = DrawStyle::outline("#333333", 1.0);
    let bot = l.plot_y + l.plot_height;
    LineShape { x1: l.plot_x, y1: bot, x2: l.plot_x + l.plot_width, y2: bot }.draw(&mut sg, &axis_style);
    LineShape { x1: l.plot_x, y1: l.plot_y, x2: l.plot_x, y2: bot }.draw(&mut sg, &axis_style);
    for bar in &l.bars { if bar.height <= 0.0 { continue; } let c = COLORS[bar.series_index % COLORS.len()]; RectShape { x: bar.x, y: bar.y, w: bar.width, h: bar.height, rx: 0.0, ry: 0.0 }.draw(&mut sg, &DrawStyle::filled(c, c, 0.5)); }
    let text_style = DrawStyle::fill_only("#333333");
    for (label, cx) in &l.x_label_positions { let tw = font_metrics::text_width(label, "SansSerif", FS, false, false); sg.set_fill_color("#333333"); sg.svg_text(label, cx-tw/2.0, bot+15.0, Some("sans-serif"), FS, None, None, None, tw, LengthAdjust::Spacing, None, 0, None); }
    for i in 0..=5 { let f = i as f64/5.0; let v = l.y_max*f; let y = l.plot_y+l.plot_height*(1.0-f); let s = if v==v.floor() { format!("{:.0}",v) } else { format!("{:.1}",v) }; let tw = font_metrics::text_width(&s, "SansSerif", FS, false, false); sg.set_fill_color("#333333"); sg.svg_text(&s, l.plot_x-tw-5.0, y+5.5, Some("sans-serif"), FS, None, None, None, tw, LengthAdjust::Spacing, None, 0, None); }
    let ly = l.plot_y+l.plot_height+35.0; let mut lx = l.plot_x;
    for (i, label) in l.series_labels.iter().enumerate() { let c = COLORS[i%COLORS.len()]; RectShape { x: lx, y: ly, w: 12.0, h: 12.0, rx: 0.0, ry: 0.0 }.draw(&mut sg, &DrawStyle { fill: Some(c.to_string()), stroke: None, stroke_width: 0.0, dash_array: None, delta_shadow: 0.0 }); let tw = font_metrics::text_width(label, "SansSerif", FS, false, false); sg.set_fill_color("#333333"); sg.svg_text(label, lx+16.0, ly+11.0, Some("sans-serif"), FS, None, None, None, tw, LengthAdjust::Spacing, None, 0, None); lx += 16.0+tw+20.0; }
    let _ = text_style;
    buf.push_str(sg.body()); buf.push_str("</g></svg>"); Ok(buf)
}
