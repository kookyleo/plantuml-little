use log::debug;

use crate::model::packet::{PacketDiagram, PacketField};
use crate::Result;

fn extract_packet_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if inside {
            if t.starts_with("@endpacket") {
                break;
            }
            lines.push(line);
        } else if t.starts_with("@startpacket") {
            inside = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn parse_packet_diagram(source: &str) -> Result<PacketDiagram> {
    let block = extract_packet_block(source).unwrap_or_else(|| source.to_string());
    debug!("parse_packet_diagram: {} bytes", block.len());

    let mut fields = Vec::new();

    for line in block.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('\'') {
            continue;
        }

        // Parse lines like "0-15: Source Port" or "32-63: Sequence Number"
        if let Some(colon_pos) = t.find(':') {
            let range_part = t[..colon_pos].trim();
            let label = t[colon_pos + 1..].trim().to_string();

            if let Some((start, end)) = parse_bit_range(range_part) {
                debug!("packet field: {}-{}: {}", start, end, label);
                fields.push(PacketField { start, end, label });
            }
        }
    }

    Ok(PacketDiagram {
        fields,
        bits_per_row: 32,
    })
}

fn parse_bit_range(s: &str) -> Option<(u32, u32)> {
    if let Some(dash_pos) = s.find('-') {
        let start = s[..dash_pos].trim().parse::<u32>().ok()?;
        let end = s[dash_pos + 1..].trim().parse::<u32>().ok()?;
        Some((start, end))
    } else {
        // Single bit: "16: Flags"
        let bit = s.trim().parse::<u32>().ok()?;
        Some((bit, bit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let src = "@startpacket\n0-15: Source Port\n16-31: Destination Port\n32-63: Sequence Number\n@endpacket";
        let d = parse_packet_diagram(src).unwrap();
        assert_eq!(d.fields.len(), 3);
        assert_eq!(d.fields[0].start, 0);
        assert_eq!(d.fields[0].end, 15);
        assert_eq!(d.fields[0].label, "Source Port");
        assert_eq!(d.fields[1].start, 16);
        assert_eq!(d.fields[1].end, 31);
        assert_eq!(d.fields[2].start, 32);
        assert_eq!(d.fields[2].end, 63);
        assert_eq!(d.bits_per_row, 32);
    }

    #[test]
    fn test_parse_single_bit() {
        let src = "@startpacket\n0: Flag\n@endpacket";
        let d = parse_packet_diagram(src).unwrap();
        assert_eq!(d.fields.len(), 1);
        assert_eq!(d.fields[0].start, 0);
        assert_eq!(d.fields[0].end, 0);
    }
}
