//! Files diagram SVG rendering.
//!
//! To match Java PlantUML byte-for-byte, we embed the exact Twemoji paths (folder
//! `1f4c2` and file `1f4c4`) as canonical sequences of (command, x, y) vertices at
//! level 0 (y_base = 10). Each entry shifts the canonical coordinates by
//! `dx = level * 21` horizontally and `dy = y_base - 10` vertically.
//!
//! The path vertices were captured from `java -jar plantuml.jar -tsvg -p` on a
//! minimal file diagram; shifting them by the delta reproduces Java's output
//! identically for every entry (verified for y translation; x translation carries
//! a tiny rounding drift in the 4th decimal because Java's internal floats differ
//! by ±1 ULP — noted in the unit test below).

use crate::klimt::svg::{fmt_coord, LengthAdjust, SvgGraphic};
use crate::layout::files_diagram::{FilesEntryLayout, FilesLayout};
use crate::model::files_diagram::{FilesDiagram, FilesEntryKind};
use crate::render::svg::write_svg_root_bg;
use crate::style::SkinParams;
use crate::Result;
use log::trace;
use std::fmt::Write as _;

/// A single drawing command in an emoji path. Coordinates are absolute.
#[derive(Copy, Clone, Debug)]
enum Cmd {
    /// MoveTo.
    M,
    /// LineTo.
    L,
    /// Cubic Bezier to. Uses 3 (x, y) pairs.
    C,
}

/// A point pair (always rendered as `x,y`).
type Pt = (f64, f64);

/// Serialise a path built from a list of (Cmd, points) groups to a single `d` string,
/// applying `(dx, dy)` translation to every point and formatting each coordinate
/// through `fmt_coord` (Java's `%.4f` + trailing-zero stripping).
fn emit_path(segments: &[(Cmd, &[Pt])], dx: f64, dy: f64) -> String {
    let mut out = String::with_capacity(256);
    let mut first = true;
    for (cmd, pts) in segments {
        if first {
            first = false;
        } else {
            out.push(' ');
        }
        let letter = match cmd {
            Cmd::M => 'M',
            Cmd::L => 'L',
            Cmd::C => 'C',
        };
        out.push(letter);
        for (i, (x, y)) in pts.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            let _ = write!(out, "{},{}", fmt_coord(x + dx), fmt_coord(y + dy));
        }
    }
    out
}

// -------------------------------------------------------------------------
// File emoji (1f4c4 — page_facing_up). Three stacked paths.
// Coordinates captured from Java output at level 0, y_base = 10.

/// Fold shadow corner (fill `#E1E8ED`).
const FILE_FOLD: &[(Cmd, &[Pt])] = &[
    (Cmd::M, &[(28.9088, 15.5918)]),
    (Cmd::L, &[(23.6587, 10.3418)]),
    (
        Cmd::C,
        &[(23.4482, 10.1313), (23.1559, 10.0), (22.8333, 10.0)],
    ),
    (
        Cmd::C,
        &[(22.1893, 10.0), (21.6673, 10.5227), (21.6667, 11.1667)],
    ),
    (
        Cmd::C,
        &[(21.6667, 11.4887), (21.7973, 11.7809), (22.0085, 11.9921)],
    ),
    (Cmd::L, &[(19.7574, 14.2432)]),
    (Cmd::L, &[(25.0074, 19.4932)]),
    (Cmd::L, &[(27.2585, 17.2421)]),
    (
        Cmd::C,
        &[(27.4697, 17.4527), (27.7613, 17.5833), (28.0833, 17.5833)],
    ),
    (
        Cmd::C,
        &[(28.7273, 17.5833), (29.2506, 17.0607), (29.25, 16.4167)],
    ),
    (
        Cmd::C,
        &[(29.25, 16.0947), (29.1193, 15.803), (28.9088, 15.5918)],
    ),
];

/// Main page body (fill `#CCD6DD`).
const FILE_BODY: &[(Cmd, &[Pt])] = &[
    (Cmd::M, &[(22.8333, 10.0)]),
    (Cmd::L, &[(14.0833, 10.0)]),
    (
        Cmd::C,
        &[(12.7948, 10.0), (11.75, 11.0448), (11.75, 12.3333)],
    ),
    (Cmd::L, &[(11.75, 28.6667)]),
    (
        Cmd::C,
        &[(11.75, 29.9553), (12.7948, 31.0), (14.0833, 31.0)],
    ),
    (Cmd::L, &[(26.9167, 31.0)]),
    (
        Cmd::C,
        &[(28.2053, 31.0), (29.25, 29.9553), (29.25, 28.6667)],
    ),
    (Cmd::L, &[(29.25, 16.4167)]),
    (Cmd::L, &[(24.0, 16.4167)]),
    (
        Cmd::C,
        &[(23.4167, 16.4167), (22.8333, 15.8333), (22.8333, 15.25)],
    ),
    (Cmd::L, &[(22.8333, 10.0)]),
];

/// Darker detail overlay (fill `#99AAB5`). This is a long path with multiple
/// `M` subpaths for the dogear shadow and the five horizontal text lines.
#[allow(clippy::too_many_lines)]
const FILE_DETAIL: &[(Cmd, &[Pt])] = &[
    // dog-ear shadow
    (Cmd::M, &[(22.8333, 10.0)]),
    (Cmd::L, &[(21.6667, 10.0)]),
    (Cmd::L, &[(21.6667, 15.25)]),
    (
        Cmd::C,
        &[(21.6667, 16.5386), (22.7114, 17.5833), (24.0, 17.5833)],
    ),
    (Cmd::L, &[(29.25, 17.5833)]),
    (Cmd::L, &[(29.25, 16.4167)]),
    (Cmd::L, &[(24.0, 16.4167)]),
    (
        Cmd::C,
        &[(23.4167, 16.4167), (22.8333, 15.8333), (22.8333, 15.25)],
    ),
    (Cmd::L, &[(22.8333, 10.0)]),
    // line 1
    (Cmd::M, &[(19.9167, 14.6667)]),
    (
        Cmd::C,
        &[(19.9167, 14.9887), (19.6553, 15.25), (19.3333, 15.25)],
    ),
    (Cmd::L, &[(14.6667, 15.25)]),
    (
        Cmd::C,
        &[(14.3447, 15.25), (14.0833, 14.9887), (14.0833, 14.6667)],
    ),
    (
        Cmd::C,
        &[(14.0833, 14.3447), (14.3447, 14.0833), (14.6667, 14.0833)],
    ),
    (Cmd::L, &[(19.3333, 14.0833)]),
    (
        Cmd::C,
        &[(19.6553, 14.0833), (19.9167, 14.3447), (19.9167, 14.6667)],
    ),
    // line 2
    (Cmd::M, &[(19.9167, 17.0)]),
    (
        Cmd::C,
        &[(19.9167, 17.322), (19.6553, 17.5833), (19.3333, 17.5833)],
    ),
    (Cmd::L, &[(14.6667, 17.5833)]),
    (
        Cmd::C,
        &[(14.3447, 17.5833), (14.0833, 17.322), (14.0833, 17.0)],
    ),
    (
        Cmd::C,
        &[(14.0833, 16.678), (14.3447, 16.4167), (14.6667, 16.4167)],
    ),
    (Cmd::L, &[(19.3333, 16.4167)]),
    (
        Cmd::C,
        &[(19.6553, 16.4167), (19.9167, 16.678), (19.9167, 17.0)],
    ),
    // line 3
    (Cmd::M, &[(26.9167, 19.3333)]),
    (
        Cmd::C,
        &[(26.9167, 19.6553), (26.6559, 19.9167), (26.3333, 19.9167)],
    ),
    (Cmd::L, &[(14.6667, 19.9167)]),
    (
        Cmd::C,
        &[(14.3447, 19.9167), (14.0833, 19.6553), (14.0833, 19.3333)],
    ),
    (
        Cmd::C,
        &[(14.0833, 19.0113), (14.3447, 18.75), (14.6667, 18.75)],
    ),
    (Cmd::L, &[(26.3333, 18.75)]),
    (
        Cmd::C,
        &[(26.6559, 18.75), (26.9167, 19.0113), (26.9167, 19.3333)],
    ),
    // line 4
    (Cmd::M, &[(26.9167, 21.6667)]),
    (
        Cmd::C,
        &[(26.9167, 21.9893), (26.6559, 22.25), (26.3333, 22.25)],
    ),
    (Cmd::L, &[(14.6667, 22.25)]),
    (
        Cmd::C,
        &[(14.3447, 22.25), (14.0833, 21.9893), (14.0833, 21.6667)],
    ),
    (
        Cmd::C,
        &[(14.0833, 21.3441), (14.3447, 21.0833), (14.6667, 21.0833)],
    ),
    (Cmd::L, &[(26.3333, 21.0833)]),
    (
        Cmd::C,
        &[(26.6559, 21.0833), (26.9167, 21.3441), (26.9167, 21.6667)],
    ),
    // line 5
    (Cmd::M, &[(26.9167, 24.0)]),
    (
        Cmd::C,
        &[(26.9167, 24.3226), (26.6559, 24.5833), (26.3333, 24.5833)],
    ),
    (Cmd::L, &[(14.6667, 24.5833)]),
    (
        Cmd::C,
        &[(14.3447, 24.5833), (14.0833, 24.3226), (14.0833, 24.0)],
    ),
    (
        Cmd::C,
        &[(14.0833, 23.6774), (14.3447, 23.4167), (14.6667, 23.4167)],
    ),
    (Cmd::L, &[(26.3333, 23.4167)]),
    (
        Cmd::C,
        &[(26.6559, 23.4167), (26.9167, 23.6774), (26.9167, 24.0)],
    ),
    // line 6
    (Cmd::M, &[(26.9167, 26.3333)]),
    (
        Cmd::C,
        &[(26.9167, 26.6559), (26.6559, 26.9167), (26.3333, 26.9167)],
    ),
    (Cmd::L, &[(14.6667, 26.9167)]),
    (
        Cmd::C,
        &[(14.3447, 26.9167), (14.0833, 26.6559), (14.0833, 26.3333)],
    ),
    (
        Cmd::C,
        &[(14.0833, 26.0108), (14.3447, 25.75), (14.6667, 25.75)],
    ),
    (Cmd::L, &[(26.3333, 25.75)]),
    (
        Cmd::C,
        &[(26.6559, 25.75), (26.9167, 26.0108), (26.9167, 26.3333)],
    ),
];

// -------------------------------------------------------------------------
// Folder emoji (1f4c2 — open_file_folder). Two stacked paths.
// Coordinates captured from Java output at level 0, y_base = 10 (emoji top shifted
// 1.75 units down from the entry y_base).

/// Folder back panel (fill `#226699`).
const FOLDER_BACK: &[(Cmd, &[Pt])] = &[
    (Cmd::M, &[(10.0, 26.9167)]),
    (
        Cmd::C,
        &[(10.0, 28.2053), (11.0448, 29.25), (12.3333, 29.25)],
    ),
    (Cmd::L, &[(26.3333, 29.25)]),
    (
        Cmd::C,
        &[(27.6219, 29.25), (28.6667, 28.2053), (28.6667, 26.9167)],
    ),
    (Cmd::L, &[(28.6667, 17.0)]),
    (
        Cmd::C,
        &[(28.6667, 15.7114), (27.6219, 14.6667), (26.3333, 14.6667)],
    ),
    (Cmd::L, &[(21.0833, 14.6667)]),
    (
        Cmd::C,
        &[(19.0055, 14.6667), (19.3333, 11.75), (16.1612, 11.75)],
    ),
    (Cmd::L, &[(12.3333, 11.75)]),
    (
        Cmd::C,
        &[(11.0448, 11.75), (10.0, 12.7948), (10.0, 14.0833)],
    ),
    (Cmd::L, &[(10.0, 26.9167)]),
];

/// Folder front flap (fill `#55ACEE`).
const FOLDER_FRONT: &[(Cmd, &[Pt])] = &[
    (Cmd::M, &[(28.8627, 17.0)]),
    (Cmd::L, &[(25.0348, 17.0)]),
    (
        Cmd::C,
        &[(21.8627, 17.0), (21.8948, 19.9167), (19.8169, 19.9167)],
    ),
    (Cmd::L, &[(14.5669, 19.9167)]),
    (
        Cmd::C,
        &[(13.2783, 19.9167), (12.1274, 20.9614), (11.9967, 22.25)],
    ),
    (Cmd::L, &[(11.7092, 24.5098)]),
    (Cmd::L, &[(11.4035, 26.9167)]),
    (Cmd::L, &[(11.3918, 26.9155)]),
    (
        Cmd::C,
        &[(11.3242, 27.2696), (10.9998, 27.4983), (10.6335, 27.4983)],
    ),
    (
        Cmd::C,
        &[(10.2584, 27.4983), (9.9883, 27.2025), (10.007, 26.8309)],
    ),
    (
        Cmd::C,
        &[(10.0058, 26.8601), (10.0, 26.8875), (10.0, 26.9167)],
    ),
    (
        Cmd::C,
        &[(10.0, 28.0857), (10.8622, 29.0447), (11.9833, 29.2144)],
    ),
    (
        Cmd::C,
        &[(12.0866, 29.2366), (12.1998, 29.25), (12.3333, 29.25)],
    ),
    (Cmd::L, &[(27.5, 29.25)]),
    (
        Cmd::C,
        &[(28.7886, 29.25), (29.9395, 28.2053), (30.0702, 26.9167)],
    ),
    (Cmd::L, &[(30.9586, 19.3333)]),
    (
        Cmd::C,
        &[(31.0898, 18.0448), (30.1513, 17.0), (28.8627, 17.0)],
    ),
];

// -------------------------------------------------------------------------

pub fn render_files(d: &FilesDiagram, layout: &FilesLayout, skin: &SkinParams) -> Result<String> {
    trace!(
        "render_files: {} entries, {}×{}",
        layout.entries.len(),
        layout.width,
        layout.height
    );
    let _ = d;
    let mut buf = String::with_capacity(4096);
    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    // Java's FilesListing dimensions already include all needed margins; pass them
    // through as-is (rounded up to whole pixels via `ceil`, not via +1).
    let sw = layout.width.ceil().max(10.0);
    let sh = layout.height.ceil().max(10.0);
    write_svg_root_bg(&mut buf, sw, sh, "FILES", bg);
    buf.push_str("<defs/><g>");
    let fc = skin.font_color("files", "#000000");
    for e in &layout.entries {
        render_entry(&mut buf, e);
        render_text(&mut buf, e, fc);
    }
    buf.push_str("</g></svg>");
    Ok(buf)
}

/// Emit the three/two emoji paths for an entry with the correct translation.
fn render_entry(buf: &mut String, e: &FilesEntryLayout) {
    let dx = (e.level as f64) * 21.0;
    match e.kind {
        FilesEntryKind::File => {
            // File emoji is anchored so that the canvas origin (`y_base = 10`) sits
            // at the top of the page; for entries at y_base Y we add dy = Y - 10.
            let dy = e.y_base - 10.0;
            write_path(buf, FILE_FOLD, dx, dy, "#E1E8ED");
            write_path(buf, FILE_BODY, dx, dy, "#CCD6DD");
            write_path(buf, FILE_DETAIL, dx, dy, "#99AAB5");
        }
        FilesEntryKind::Folder => {
            // Folder emoji's canonical origin is at y_base + 1.75 — Java's FEntry lays
            // the folder with a small top gap so the open flap aligns with text.
            let dy = e.y_base - 10.0 + 1.75;
            write_path(buf, FOLDER_BACK, dx, dy, "#226699");
            write_path(buf, FOLDER_FRONT, dx, dy, "#55ACEE");
        }
    }
}

fn write_path(buf: &mut String, segments: &[(Cmd, &[Pt])], dx: f64, dy: f64, fill: &str) {
    let d = emit_path(segments, dx, dy);
    let _ = write!(buf, r#"<path d="{}" fill="{}"/>"#, d, fill);
}

fn render_text(buf: &mut String, e: &FilesEntryLayout, fc: &str) {
    // Use SvgGraphic's text helper to match the attribute order used elsewhere.
    let mut sg = SvgGraphic::new(0, 1.0);
    sg.set_fill_color(fc);
    sg.svg_text(
        &e.name,
        e.text_x,
        e.text_y,
        Some("sans-serif"),
        14.0,
        None,
        None,
        None,
        e.text_length,
        LengthAdjust::Spacing,
        None,
        0,
        None,
    );
    // SvgGraphic buffers the element; copy into the output buffer.
    buf.push_str(sg.body());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_path_formats_with_java_style_numbers() {
        // A single M should emit with fmt_coord (no trailing zeros).
        let seg = &[(Cmd::M, &[(10.5, 11.75)] as &[Pt])];
        assert_eq!(emit_path(seg, 0.0, 0.0), "M10.5,11.75");
    }

    #[test]
    fn emit_path_applies_translation() {
        // Shifting by (1, 2) should add to every coordinate.
        let seg = &[(Cmd::L, &[(10.0, 20.0), (30.0, 40.0)] as &[Pt])];
        assert_eq!(emit_path(seg, 1.0, 2.0), "L11,22 31,42");
    }

    #[test]
    fn file_fold_first_point_matches_java_reference_at_level_0() {
        // The first MoveTo of the fold triangle must equal the byte-exact Java output.
        let s = emit_path(&FILE_FOLD[..1], 0.0, 0.0);
        assert_eq!(s, "M28.9088,15.5918");
    }
}
