//! Files-diagram layout.
//!
//! Closely follows Java `FilesListing`/`FEntry`:
//! - Entries are traversed depth-first and laid out vertically.
//! - Each entry occupies `ENTRY_H = 24.75` units of vertical space.
//! - Level-L entry draws its emoji with an x shift of `L * DEPTH_DX = L * 21`.
//! - Text baseline sits at `10 + 19.4482 + index * ENTRY_H` where `index` is the
//!   pre-order flat position of the entry.
//!
//! The actual emoji paths are hard-coded in `render::svg_files` as byte-exact
//! copies of Java output; this module only computes where to place them and the
//! final SVG dimensions.

use crate::font_metrics;
use crate::model::files_diagram::{FilesDiagram, FilesEntry, FilesEntryKind};
use crate::Result;

/// Vertical space between consecutive entries (Java's `getHeight() + 2`).
pub const ENTRY_H: f64 = 24.75;
/// Horizontal indent per nesting level (Java hard-codes `deltax + 21`).
pub const DEPTH_DX: f64 = 21.0;
/// Top margin before the first entry.
pub const TOP_MARGIN: f64 = 10.0;
/// Extra height added after the last entry baseline. Empirical — matches Java.
pub const BOTTOM_EXTRA: f64 = 17.5;
/// Horizontal padding added to the rightmost text end to get SVG width.
pub const RIGHT_PAD: f64 = 11.0;
/// Base X for the text label at level 0.
pub const TEXT_BASE_X: f64 = 31.0;
/// Text baseline offset within an entry (Java Creole line ascent).
pub const TEXT_BASELINE_DY: f64 = 19.4482;

#[derive(Debug, Clone)]
pub struct FilesEntryLayout {
    pub name: String,
    pub kind: FilesEntryKind,
    /// Nesting level (0-indexed).
    pub level: usize,
    /// Base y for the emoji (level 0 file: emoji top = y_base, folder: y_base + 1.75).
    pub y_base: f64,
    /// Text baseline y coordinate.
    pub text_y: f64,
    /// Text x coordinate (`TEXT_BASE_X + level * DEPTH_DX`).
    pub text_x: f64,
    /// Pre-measured text length (via font metrics, matches Java).
    pub text_length: f64,
}

#[derive(Debug, Clone)]
pub struct FilesLayout {
    pub entries: Vec<FilesEntryLayout>,
    pub width: f64,
    pub height: f64,
}

/// Produce the flat layout of a files diagram.
pub fn layout_files(d: &FilesDiagram) -> Result<FilesLayout> {
    let mut entries = Vec::new();
    for e in &d.entries {
        walk(e, 0, &mut entries);
    }
    // Assign y positions and measure text.
    let mut max_text_right: f64 = 0.0;
    for (i, entry) in entries.iter_mut().enumerate() {
        let y_base = TOP_MARGIN + (i as f64) * ENTRY_H;
        entry.y_base = y_base;
        entry.text_y = y_base + TEXT_BASELINE_DY;
        entry.text_x = TEXT_BASE_X + (entry.level as f64) * DEPTH_DX;
        entry.text_length = font_metrics::text_width(&entry.name, "SansSerif", 14.0, false, false);
        let right = entry.text_x + entry.text_length;
        if right > max_text_right {
            max_text_right = right;
        }
    }
    let n = entries.len() as f64;
    let width = (max_text_right + RIGHT_PAD).ceil();
    let height = (n * ENTRY_H + BOTTOM_EXTRA).ceil();
    Ok(FilesLayout {
        entries,
        width,
        height,
    })
}

fn walk(e: &FilesEntry, level: usize, out: &mut Vec<FilesEntryLayout>) {
    out.push(FilesEntryLayout {
        name: e.name.clone(),
        kind: e.kind.clone(),
        level,
        y_base: 0.0,
        text_y: 0.0,
        text_x: 0.0,
        text_length: 0.0,
    });
    for c in &e.children {
        walk(c, level + 1, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_entry_has_expected_dimensions() {
        // One file at level 0 should produce svg_height = ceil(1 * 24.75 + 17.5) = 43.
        let d = FilesDiagram {
            entries: vec![FilesEntry {
                name: "x".to_string(),
                kind: FilesEntryKind::File,
                children: vec![],
            }],
        };
        let layout = layout_files(&d).unwrap();
        assert_eq!(layout.entries.len(), 1);
        assert_eq!(layout.entries[0].level, 0);
        assert_eq!(layout.entries[0].y_base, 10.0);
        assert!(
            (layout.entries[0].text_y - 29.4482).abs() < 1e-6,
            "text_y was {}",
            layout.entries[0].text_y
        );
        assert_eq!(layout.height, 43.0);
    }

    #[test]
    fn two_entries_have_24_75_spacing() {
        // Two files should produce text y values 29.4482 and 54.1982.
        let d = FilesDiagram {
            entries: vec![
                FilesEntry {
                    name: "a".to_string(),
                    kind: FilesEntryKind::File,
                    children: vec![],
                },
                FilesEntry {
                    name: "b".to_string(),
                    kind: FilesEntryKind::File,
                    children: vec![],
                },
            ],
        };
        let layout = layout_files(&d).unwrap();
        assert_eq!(layout.entries.len(), 2);
        assert!((layout.entries[0].text_y - 29.4482).abs() < 1e-6);
        assert!((layout.entries[1].text_y - 54.1982).abs() < 1e-6);
        assert_eq!(layout.height, 67.0);
    }

    #[test]
    fn nested_entries_are_flattened_in_preorder() {
        let d = FilesDiagram {
            entries: vec![FilesEntry {
                name: "etc".to_string(),
                kind: FilesEntryKind::Folder,
                children: vec![FilesEntry {
                    name: "nginx".to_string(),
                    kind: FilesEntryKind::Folder,
                    children: vec![FilesEntry {
                        name: "conf".to_string(),
                        kind: FilesEntryKind::File,
                        children: vec![],
                    }],
                }],
            }],
        };
        let layout = layout_files(&d).unwrap();
        assert_eq!(layout.entries.len(), 3);
        assert_eq!(layout.entries[0].name, "etc");
        assert_eq!(layout.entries[0].level, 0);
        assert_eq!(layout.entries[1].name, "nginx");
        assert_eq!(layout.entries[1].level, 1);
        assert_eq!(layout.entries[2].name, "conf");
        assert_eq!(layout.entries[2].level, 2);
    }
}
