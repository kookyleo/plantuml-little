// klimt::color - Color system
// Port of Java PlantUML's klimt.color package

use super::UChange;

// ── HColor ───────────────────────────────────────────────────────────

/// Hierarchical color. Java: `klimt.color.HColor`
///
/// Represents a color in PlantUML's color system. Can be:
/// - A simple RGB hex color
/// - A named color (resolved via HColorSet)
/// - Transparent/none
/// - Automatic (context-dependent)
#[derive(Debug, Clone, PartialEq)]
pub enum HColor {
    None,
    Simple { r: u8, g: u8, b: u8 },
    WithAlpha { r: u8, g: u8, b: u8, a: u8 },
    Gradient { color1: Box<HColor>, color2: Box<HColor>, angle: char },
}

impl UChange for HColor {}

impl HColor {
    pub fn none() -> Self { Self::None }

    /// Parse a hex color like "#FF0000" or "#F00" or "FF0000".
    pub fn simple(s: &str) -> Self {
        let s = s.strip_prefix('#').unwrap_or(s);
        if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            Self::Simple { r, g, b }
        } else if s.len() == 3 {
            let r = u8::from_str_radix(&s[0..1], 16).unwrap_or(0) * 17;
            let g = u8::from_str_radix(&s[1..2], 16).unwrap_or(0) * 17;
            let b = u8::from_str_radix(&s[2..3], 16).unwrap_or(0) * 17;
            Self::Simple { r, g, b }
        } else {
            Self::None
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self { Self::Simple { r, g, b } }

    pub fn is_none(&self) -> bool { matches!(self, Self::None) }

    /// Convert to SVG color string: "#RRGGBB"
    pub fn to_svg(&self) -> String {
        match self {
            Self::None => "none".to_string(),
            Self::Simple { r, g, b } => format!("#{:02X}{:02X}{:02X}", r, g, b),
            Self::WithAlpha { r, g, b, a } => {
                if *a == 255 {
                    format!("#{:02X}{:02X}{:02X}", r, g, b)
                } else {
                    format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
                }
            }
            Self::Gradient { color1, .. } => color1.to_svg(),
        }
    }

    /// Convert to RGB integer (0xRRGGBB)
    pub fn to_rgb(&self) -> u32 {
        match self {
            Self::Simple { r, g, b } | Self::WithAlpha { r, g, b, .. } => {
                ((*r as u32) << 16) | ((*g as u32) << 8) | (*b as u32)
            }
            _ => 0,
        }
    }

    /// Darken color by a factor (0.0 = black, 1.0 = unchanged)
    pub fn darken(&self, factor: f64) -> Self {
        match self {
            Self::Simple { r, g, b } => Self::Simple {
                r: ((*r as f64) * factor) as u8,
                g: ((*g as f64) * factor) as u8,
                b: ((*b as f64) * factor) as u8,
            },
            other => other.clone(),
        }
    }
}

impl Default for HColor {
    fn default() -> Self { Self::None }
}

// ── HColorSet: Named color registry ─────────────────────────────────

/// Resolves named colors (e.g., "red", "LightBlue", "DarkSalmon")
/// Java: `klimt.color.HColorSet`
pub fn resolve_color(name: &str) -> Option<HColor> {
    // Match Java PlantUML's color names (subset of SVG named colors)
    let rgb = match name.to_lowercase().as_str() {
        "red" => (0xFF, 0x00, 0x00),
        "green" => (0x00, 0x80, 0x00),
        "blue" => (0x00, 0x00, 0xFF),
        "yellow" => (0xFF, 0xFF, 0x00),
        "black" => (0x00, 0x00, 0x00),
        "white" => (0xFF, 0xFF, 0xFF),
        "transparent" | "none" => return Some(HColor::None),
        _ => {
            // Try parsing as hex
            if name.starts_with('#') || name.len() == 6 {
                return Some(HColor::simple(name));
            }
            return None;
        }
    };
    Some(HColor::rgb(rgb.0, rgb.1, rgb.2))
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_6() {
        let c = HColor::simple("#FF8800");
        assert_eq!(c.to_svg(), "#FF8800");
    }

    #[test]
    fn parse_hex_3() {
        let c = HColor::simple("#F80");
        assert_eq!(c.to_svg(), "#FF8800");
    }

    #[test]
    fn parse_without_hash() {
        let c = HColor::simple("00FF00");
        assert_eq!(c.to_svg(), "#00FF00");
    }

    #[test]
    fn to_rgb_int() {
        let c = HColor::rgb(0x12, 0x34, 0x56);
        assert_eq!(c.to_rgb(), 0x123456);
    }

    #[test]
    fn none_renders_none() {
        assert_eq!(HColor::none().to_svg(), "none");
    }

    #[test]
    fn resolve_named() {
        assert_eq!(resolve_color("red").unwrap().to_svg(), "#FF0000");
        assert!(resolve_color("nonexistent_xyz").is_none());
    }
}
