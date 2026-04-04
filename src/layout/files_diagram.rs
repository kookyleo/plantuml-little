use crate::font_metrics;
use crate::model::files_diagram::{FilesDiagram, FilesEntry, FilesEntryKind};
use crate::Result;
#[derive(Debug, Clone)]
pub struct FilesEntryLayout { pub name: String, pub kind: FilesEntryKind, pub x: f64, pub y: f64, pub text_width: f64 }
#[derive(Debug, Clone)]
pub struct FilesLayout { pub entries: Vec<FilesEntryLayout>, pub width: f64, pub height: f64 }
pub fn layout_files(d: &FilesDiagram) -> Result<FilesLayout> {
    let (mut entries, mut y, mut mw) = (Vec::new(), 10.0, 0.0_f64);
    for e in &d.entries { lay(e, 0, &mut y, &mut entries, &mut mw); }
    Ok(FilesLayout { entries, width: mw + 20.0, height: y + 10.0 })
}
fn lay(e: &FilesEntry, depth: usize, y: &mut f64, out: &mut Vec<FilesEntryLayout>, mw: &mut f64) {
    let x = 10.0 + depth as f64 * 21.0;
    let tw = font_metrics::text_width(&e.name, "SansSerif", 14.0, false, false);
    *mw = mw.max(x + 20.0 + tw);
    out.push(FilesEntryLayout { name: e.name.clone(), kind: e.kind.clone(), x, y: *y, text_width: tw });
    *y += 20.0;
    for c in &e.children { lay(c, depth+1, y, out, mw); }
}
