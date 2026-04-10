use crate::model::packet::PacketDiagram;
use crate::Result;

/// Layout for a single cell in the packet grid.
#[derive(Debug, Clone)]
pub struct PacketCellLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub label: String,
    /// Bit range text (e.g., "0-15").
    pub range_text: String,
}

/// Layout for the full packet diagram.
#[derive(Debug, Clone)]
pub struct PacketLayout {
    pub cells: Vec<PacketCellLayout>,
    pub width: f64,
    pub height: f64,
    pub bits_per_row: u32,
    /// Bit number labels at the top.
    pub bit_labels: Vec<(f64, String)>,
}

/// Width of one bit column.
const BIT_WIDTH: f64 = 20.0;
/// Row height for each data row.
const ROW_HEIGHT: f64 = 40.0;
/// Height of the header row (bit numbers).
const HEADER_HEIGHT: f64 = 20.0;
/// Left/top margin.
const MARGIN: f64 = 10.0;
/// Font size for labels.
#[allow(dead_code)] // Java-ported layout constant
const FONT_SIZE: f64 = 12.0;

pub fn layout_packet(d: &PacketDiagram) -> Result<PacketLayout> {
    let bpr = d.bits_per_row;
    let total_bit_width = bpr as f64 * BIT_WIDTH;

    // Compute bit labels at top
    let mut bit_labels = Vec::new();
    for i in 0..bpr {
        let x = MARGIN + i as f64 * BIT_WIDTH + BIT_WIDTH / 2.0;
        bit_labels.push((x, format!("{}", i)));
    }

    // Lay out each field into cells, potentially spanning multiple rows
    let mut cells = Vec::new();
    let mut max_row: u32 = 0;

    for field in &d.fields {
        let start = field.start;
        let end = field.end;
        let span = end - start + 1;

        // A field may span multiple rows
        let mut bit = start;
        while bit <= end {
            let row = bit / bpr;
            let col = bit % bpr;
            // How many bits remain in this row
            let bits_in_row = (bpr - col).min(end - bit + 1);

            let x = MARGIN + col as f64 * BIT_WIDTH;
            let y = MARGIN + HEADER_HEIGHT + row as f64 * ROW_HEIGHT;
            let w = bits_in_row as f64 * BIT_WIDTH;

            // Only show the label in the first (or only) segment
            let label = if bit == start {
                field.label.clone()
            } else {
                String::new()
            };

            let range_text = if span == 1 {
                format!("{}", start)
            } else {
                format!("{}-{}", start, end)
            };

            cells.push(PacketCellLayout {
                x,
                y,
                width: w,
                height: ROW_HEIGHT,
                label,
                range_text,
            });

            if row > max_row {
                max_row = row;
            }
            bit += bits_in_row;
        }
    }

    let width = MARGIN * 2.0 + total_bit_width;
    let height = MARGIN * 2.0 + HEADER_HEIGHT + (max_row + 1) as f64 * ROW_HEIGHT;

    Ok(PacketLayout {
        cells,
        width,
        height,
        bits_per_row: bpr,
        bit_labels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::packet::{PacketDiagram, PacketField};

    #[test]
    fn test_layout_basic() {
        let d = PacketDiagram {
            fields: vec![
                PacketField {
                    start: 0,
                    end: 15,
                    label: "Source Port".into(),
                },
                PacketField {
                    start: 16,
                    end: 31,
                    label: "Dest Port".into(),
                },
                PacketField {
                    start: 32,
                    end: 63,
                    label: "Seq Number".into(),
                },
            ],
            bits_per_row: 32,
        };
        let l = layout_packet(&d).unwrap();
        // 3 fields, 2nd one spans full row → 3 cells total
        // Actually field 0-15 = 1 cell, 16-31 = 1 cell, 32-63 = 1 cell (full row)
        assert_eq!(l.cells.len(), 3);
        assert!(l.width > 0.0);
        assert!(l.height > 0.0);
        assert_eq!(l.bit_labels.len(), 32);
    }
}
