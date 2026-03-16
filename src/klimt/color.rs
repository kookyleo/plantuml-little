// klimt::color - Color system
// Port of Java PlantUML's klimt.color package

use super::UChange;

// ── HColor ───────────────────────────────────────────────────────────

/// Hierarchical color. Java: `klimt.color.HColor`
///
/// Represents a color in PlantUML's color system. Can be:
/// - A simple RGB hex color (optionally with alpha)
/// - A named color (resolved via HColorSet)
/// - Transparent/none
/// - A gradient between two colors
/// - A linear gradient with multiple stops
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HColor {
    /// Transparent / no color (alpha=0).
    #[default]
    None,
    /// Simple RGB color.
    Simple { r: u8, g: u8, b: u8 },
    /// RGB color with explicit alpha channel.
    WithAlpha { r: u8, g: u8, b: u8, a: u8 },
    /// Two-color gradient with a direction policy ('-', '|', '/', '\\').
    Gradient {
        color1: Box<HColor>,
        color2: Box<HColor>,
        policy: char,
    },
    /// SVG-style linear gradient with multiple stops.
    LinearGradient(HColorLinearGradient),
}

impl UChange for HColor {}

impl HColor {
    pub fn none() -> Self {
        Self::None
    }

    /// Parse a hex color like "#FF0000", "#F00", "FF0000", or single-digit "#A".
    /// Also handles 8-digit hex with alpha: "#RRGGBBAA".
    /// Returns `HColor::None` if the input is not valid hex.
    pub fn simple(s: &str) -> Self {
        let s = s.strip_prefix('#').unwrap_or(s);
        try_parse_hex(s).unwrap_or(Self::None)
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Simple { r, g, b }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        if a == 0 {
            Self::None
        } else if a == 255 {
            Self::Simple { r, g, b }
        } else {
            Self::WithAlpha { r, g, b, a }
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            Self::None => true,
            Self::WithAlpha { a, .. } => *a == 0,
            _ => false,
        }
    }

    /// Grayscale value using YIQ formula (Java: ColorUtils.getGrayScale).
    pub fn gray_scale(&self) -> u8 {
        let (r, g, b) = self.rgb_components();
        ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8
    }

    /// Whether this color is considered "dark" (grayscale < 128).
    pub fn is_dark(&self) -> bool {
        match self {
            Self::None => false,
            Self::LinearGradient(lg) => {
                if let Some(stop) = lg.stops.first() {
                    stop.color.is_dark()
                } else {
                    true
                }
            }
            _ => self.gray_scale() < 128,
        }
    }

    /// Extract (r, g, b) components. Returns (0,0,0) for None/Gradient.
    pub fn rgb_components(&self) -> (u8, u8, u8) {
        match self {
            Self::Simple { r, g, b } | Self::WithAlpha { r, g, b, .. } => (*r, *g, *b),
            Self::Gradient { color1, .. } => color1.rgb_components(),
            Self::LinearGradient(lg) => {
                if let Some(stop) = lg.stops.first() {
                    stop.color.rgb_components()
                } else {
                    (0, 0, 0)
                }
            }
            Self::None => (0, 0, 0),
        }
    }

    /// Get alpha component (255 = fully opaque, 0 = fully transparent).
    pub fn alpha(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Simple { .. } => 255,
            Self::WithAlpha { a, .. } => *a,
            Self::Gradient { color1, .. } => color1.alpha(),
            Self::LinearGradient(_) => 255,
        }
    }

    /// Convert to SVG color string: "#RRGGBB" or "#RRGGBBAA" or "none".
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
            Self::LinearGradient(lg) => {
                if let Some(stop) = lg.stops.first() {
                    stop.color.to_svg()
                } else {
                    "none".to_string()
                }
            }
        }
    }

    /// Convert to RGB integer (0xRRGGBB).
    pub fn to_rgb(&self) -> u32 {
        let (r, g, b) = self.rgb_components();
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    /// Java-style asString: "#RRGGBB" or "transparent" or "#aarrggbb".
    pub fn as_string(&self) -> String {
        if self.is_transparent() {
            return "transparent".to_string();
        }
        match self {
            Self::Simple { r, g, b } => format!("#{:02X}{:02X}{:02X}", r, g, b),
            Self::WithAlpha { r, g, b, a } => {
                if *a == 255 {
                    format!("#{:02X}{:02X}{:02X}", r, g, b)
                } else {
                    // Java format: alpha first, then rgb, all lowercase
                    format!("#{:02x}{:02x}{:02x}{:02x}", a, r, g, b)
                }
            }
            _ => self.to_svg(),
        }
    }

    /// Darken color by a factor (0.0 = black, 1.0 = unchanged).
    /// This is a simple linear darken; for HSL-based darken see `darken_hsl`.
    pub fn darken(&self, factor: f64) -> Self {
        match self {
            Self::Simple { r, g, b } => Self::Simple {
                r: ((*r as f64) * factor) as u8,
                g: ((*g as f64) * factor) as u8,
                b: ((*b as f64) * factor) as u8,
            },
            Self::WithAlpha { r, g, b, a } => Self::WithAlpha {
                r: ((*r as f64) * factor) as u8,
                g: ((*g as f64) * factor) as u8,
                b: ((*b as f64) * factor) as u8,
                a: *a,
            },
            other => other.clone(),
        }
    }

    /// Darken using HSL model (Java: HColorSimple.darken).
    /// `ratio` is 0..100; luminance is decreased by that percentage.
    pub fn darken_hsl(&self, ratio: i32) -> Self {
        let (r, g, b) = self.rgb_components();
        let mut hsl = rgb_to_hsl(r, g, b);
        hsl[2] -= hsl[2] * (ratio as f32 / 100.0);
        let (r2, g2, b2) = hsl_to_rgb(hsl[0], hsl[1], hsl[2]);
        Self::Simple {
            r: r2,
            g: g2,
            b: b2,
        }
    }

    /// Lighten using HSL model (Java: HColorSimple.lighten).
    /// `ratio` is 0..100; luminance is increased by that percentage.
    pub fn lighten_hsl(&self, ratio: i32) -> Self {
        let (r, g, b) = self.rgb_components();
        let mut hsl = rgb_to_hsl(r, g, b);
        hsl[2] += hsl[2] * (ratio as f32 / 100.0);
        let (r2, g2, b2) = hsl_to_rgb(hsl[0], hsl[1], hsl[2]);
        Self::Simple {
            r: r2,
            g: g2,
            b: b2,
        }
    }

    /// Opposite color: compute grayscale, invert, then return black or white.
    pub fn opposite(&self) -> Self {
        let gs = self.gray_scale();
        let v = if (255 - gs) > 127 { 255 } else { 0 };
        Self::Simple { r: v, g: v, b: v }
    }

    /// Convert to grayscale color.
    pub fn as_monochrome(&self) -> Self {
        let gs = self.gray_scale();
        Self::Simple {
            r: gs,
            g: gs,
            b: gs,
        }
    }

    /// Check if this is a gray color (r == g == b).
    pub fn is_gray(&self) -> bool {
        match self {
            Self::Simple { r, g, b } | Self::WithAlpha { r, g, b, .. } => r == g && g == b,
            _ => false,
        }
    }

    /// Euclidean distance between two colors using YIQ weighting.
    pub fn distance_to(&self, other: &HColor) -> u32 {
        let (r1, g1, b1) = self.rgb_components();
        let (r2, g2, b2) = other.rgb_components();
        let dr = (r1 as i32 - r2 as i32).unsigned_abs();
        let dg = (g1 as i32 - g2 as i32).unsigned_abs();
        let db = (b1 as i32 - b2 as i32).unsigned_abs();
        dr * 299 + dg * 587 + db * 114
    }

    /// Interpolate between two colors at coefficient `coeff` in [0, 1].
    pub fn interpolate(&self, other: &HColor, coeff: f64) -> Self {
        let (r1, g1, b1) = self.rgb_components();
        let (r2, g2, b2) = other.rgb_components();
        let r = (r1 as f64 + (r2 as f64 - r1 as f64) * coeff) as u8;
        let g = (g1 as f64 + (g2 as f64 - g1 as f64) * coeff) as u8;
        let b = (b1 as f64 + (b2 as f64 - b1 as f64) * coeff) as u8;
        Self::Simple { r, g, b }
    }

    /// Strip gradient: if this is a gradient, return the first color.
    pub fn no_gradient(&self) -> HColor {
        match self {
            Self::Gradient { color1, .. } => color1.as_ref().clone(),
            Self::LinearGradient(lg) => {
                if let Some(stop) = lg.stops.first() {
                    stop.color.clone()
                } else {
                    Self::None
                }
            }
            other => other.clone(),
        }
    }

    /// Return whether this is a gradient (two-color or linear).
    pub fn is_gradient(&self) -> bool {
        matches!(self, Self::Gradient { .. } | Self::LinearGradient(_))
    }
}

// Default: HColor derives Default via #[default] on the None variant.

// ── HColorGradient: two-color gradient ───────────────────────────────

impl HColor {
    /// Create a two-color gradient. Policy chars: '-' (horizontal),
    /// '|' (vertical), '/' (diagonal), '\\' (reverse diagonal).
    pub fn gradient(color1: HColor, color2: HColor, policy: char) -> Self {
        // Flatten nested gradients
        let c1 = match color1 {
            HColor::Gradient { color1: inner, .. } => *inner,
            other => other,
        };
        let c2 = match color2 {
            HColor::Gradient { color2: inner, .. } => *inner,
            other => other,
        };
        Self::Gradient {
            color1: Box::new(c1),
            color2: Box::new(c2),
            policy,
        }
    }

    /// Get gradient policy character, if this is a gradient.
    pub fn gradient_policy(&self) -> Option<char> {
        match self {
            Self::Gradient { policy, .. } => Some(*policy),
            _ => None,
        }
    }

    /// Get gradient colors as (color1, color2), if this is a gradient.
    pub fn gradient_colors(&self) -> Option<(&HColor, &HColor)> {
        match self {
            Self::Gradient { color1, color2, .. } => Some((color1, color2)),
            _ => None,
        }
    }

    /// Interpolate gradient at coefficient `coeff` in [0, 1] with alpha.
    pub fn gradient_color_at(&self, coeff: f64, alpha: u8) -> HColor {
        match self {
            Self::Gradient { color1, color2, .. } => {
                let coeff = coeff.clamp(0.0, 1.0);
                let (r1, g1, b1) = color1.rgb_components();
                let (r2, g2, b2) = color2.rgb_components();
                let r = (r1 as f64 + (r2 as f64 - r1 as f64) * coeff) as u8;
                let g = (g1 as f64 + (g2 as f64 - g1 as f64) * coeff) as u8;
                let b = (b1 as f64 + (b2 as f64 - b1 as f64) * coeff) as u8;
                if alpha == 255 {
                    HColor::Simple { r, g, b }
                } else {
                    HColor::WithAlpha { r, g, b, a: alpha }
                }
            }
            other => other.clone(),
        }
    }
}

// ── HColorLinearGradient: SVG multi-stop gradient ────────────────────

/// Spread method for linear gradients (mirrors SVG spreadMethod).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadMethod {
    /// Extend edge colors (default).
    Pad,
    /// Reflect the gradient back and forth.
    Reflect,
    /// Repeat the gradient in the same direction.
    Repeat,
}

impl Default for SpreadMethod {
    fn default() -> Self {
        Self::Pad
    }
}

/// A single gradient stop with offset, color, and opacity.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientStop {
    /// Position along gradient vector (0.0..1.0).
    pub offset: f64,
    /// Color at this stop.
    pub color: HColor,
    /// Opacity multiplier (0.0..1.0).
    pub opacity: f64,
}

/// SVG-style linear gradient with multiple stops.
/// Java: `klimt.color.HColorLinearGradient`
#[derive(Debug, Clone, PartialEq)]
pub struct HColorLinearGradient {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub user_space_on_use: bool,
    pub spread_method: SpreadMethod,
    pub stops: Vec<GradientStop>,
}

impl HColorLinearGradient {
    /// Create a new linear gradient. Stops are normalized and sorted internally.
    pub fn new(
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        user_space_on_use: bool,
        spread_method: SpreadMethod,
        stops: Vec<GradientStop>,
    ) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            user_space_on_use,
            spread_method,
            stops: normalize_stops(stops),
        }
    }

    /// Get the first stop's color (fallback for single-color APIs).
    pub fn color1(&self) -> HColor {
        self.stops
            .first()
            .map(|s| s.color.clone())
            .unwrap_or(HColor::None)
    }
}

/// Normalize stops: sort by offset, clamp to 0..1, nudge duplicates.
fn normalize_stops(mut stops: Vec<GradientStop>) -> Vec<GradientStop> {
    stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(std::cmp::Ordering::Equal));

    let mut result = Vec::with_capacity(stops.len());
    let mut last = -1.0_f64;
    for stop in stops {
        let mut offset = stop.offset.clamp(0.0, 1.0);
        if offset <= last {
            let candidate = last + 0.000001;
            offset = if candidate > 1.0 { last } else { candidate };
        }
        last = offset;
        result.push(GradientStop {
            offset,
            color: stop.color,
            opacity: stop.opacity,
        });
    }
    result
}

// ── HSL conversion (port of Java HSLColor) ───────────────────────────

/// Convert RGB (0-255) to HSL: [hue(0-360), saturation(0-100), luminance(0-100)].
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> [f32; 3] {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let min = rf.min(gf).min(bf);
    let max = rf.max(gf).max(bf);

    // Hue
    let h = if (max - min).abs() < f32::EPSILON {
        0.0
    } else if (max - rf).abs() < f32::EPSILON {
        ((60.0 * (gf - bf) / (max - min)) + 360.0) % 360.0
    } else if (max - gf).abs() < f32::EPSILON {
        (60.0 * (bf - rf) / (max - min)) + 120.0
    } else {
        (60.0 * (rf - gf) / (max - min)) + 240.0
    };

    // Luminance
    let l = (max + min) / 2.0;

    // Saturation
    let s = if (max - min).abs() < f32::EPSILON {
        0.0
    } else if l <= 0.5 {
        (max - min) / (max + min)
    } else {
        (max - min) / (2.0 - max - min)
    };

    [h, s * 100.0, l * 100.0]
}

/// Convert HSL to RGB (0-255). h in 0-360, s in 0-100, l in 0-100.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s.clamp(0.0, 100.0);
    let l = l.clamp(0.0, 100.0);

    let h = (h % 360.0) / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        (l + s) - (s * l)
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0).clamp(0.0, 1.0);
    let g = hue_to_rgb(p, q, h).clamp(0.0, 1.0);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0).clamp(0.0, 1.0);

    (
        (r * 255.0 + 0.5) as u8,
        (g * 255.0 + 0.5) as u8,
        (b * 255.0 + 0.5) as u8,
    )
}

fn hue_to_rgb(p: f32, q: f32, mut h: f32) -> f32 {
    if h < 0.0 {
        h += 1.0;
    }
    if h > 1.0 {
        h -= 1.0;
    }
    if 6.0 * h < 1.0 {
        return p + ((q - p) * 6.0 * h);
    }
    if 2.0 * h < 1.0 {
        return q;
    }
    if 3.0 * h < 2.0 {
        return p + ((q - p) * 6.0 * (2.0 / 3.0 - h));
    }
    p
}

// ── HColors: well-known color constants ──────────────────────────────

/// Well-known PlantUML color constants (Java: HColors).
pub mod colors {
    use super::HColor;

    pub const BLACK: HColor = HColor::Simple {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    };
    pub const WHITE: HColor = HColor::Simple {
        r: 0xFF,
        g: 0xFF,
        b: 0xFF,
    };
    pub const RED: HColor = HColor::Simple {
        r: 0xFF,
        g: 0x00,
        b: 0x00,
    };
    pub const GREEN: HColor = HColor::Simple {
        r: 0x00,
        g: 0xFF,
        b: 0x00,
    };
    pub const BLUE: HColor = HColor::Simple {
        r: 0x00,
        g: 0x00,
        b: 0xFF,
    };
    pub const GRAY: HColor = HColor::Simple {
        r: 0x80,
        g: 0x80,
        b: 0x80,
    };
    pub const LIGHT_GRAY: HColor = HColor::Simple {
        r: 0xC0,
        g: 0xC0,
        b: 0xC0,
    };
    pub const RED_LIGHT: HColor = HColor::Simple {
        r: 0xFE,
        g: 0xF6,
        b: 0xF3,
    };
    pub const RED_DARK: HColor = HColor::Simple {
        r: 0xCD,
        g: 0x0A,
        b: 0x0A,
    };
    pub const MY_YELLOW: HColor = HColor::Simple {
        r: 0xFE,
        g: 0xFE,
        b: 0xCE,
    };
    pub const MY_RED: HColor = HColor::Simple {
        r: 0xA8,
        g: 0x00,
        b: 0x36,
    };
    pub const MY_GREEN: HColor = HColor::Simple {
        r: 0x33,
        g: 0xFF,
        b: 0x02,
    };
}

// ── HColorSet: Named color registry ─────────────────────────────────

/// Complete named color table from Java PlantUML's ColorTrieNode.
/// Case-insensitive lookup of SVG/X11 named colors plus Archimate extras.
///
/// Returns the (r, g, b) tuple for a known color name, or None.
fn named_color_rgb(name: &str) -> Option<(u8, u8, u8)> {
    // Build lowercase for case-insensitive matching.
    // Java uses a trie on lowercased chars; we use a match for zero dependencies.
    let lower = name.to_ascii_lowercase();
    let rgb: u32 = match lower.as_str() {
        "aliceblue" => 0xF0F8FF,
        "antiquewhite" => 0xFAEBD7,
        "aqua" => 0x00FFFF,
        "aquamarine" => 0x7FFFD4,
        "azure" => 0xF0FFFF,
        "beige" => 0xF5F5DC,
        "bisque" => 0xFFE4C4,
        "black" => 0x000000,
        "blanchedalmond" => 0xFFEBCD,
        "blue" => 0x0000FF,
        "blueviolet" => 0x8A2BE2,
        "brown" => 0xA52A2A,
        "burlywood" => 0xDEB887,
        "cadetblue" => 0x5F9EA0,
        "chartreuse" => 0x7FFF00,
        "chocolate" => 0xD2691E,
        "coral" => 0xFF7F50,
        "cornflowerblue" => 0x6495ED,
        "cornsilk" => 0xFFF8DC,
        "crimson" => 0xDC143C,
        "cyan" => 0x00FFFF,
        "darkblue" => 0x00008B,
        "darkcyan" => 0x008B8B,
        "darkgoldenrod" => 0xB8860B,
        "darkgray" | "darkgrey" => 0xA9A9A9,
        "darkgreen" => 0x006400,
        "darkkhaki" => 0xBDB76B,
        "darkmagenta" => 0x8B008B,
        "darkolivegreen" => 0x556B2F,
        "darkorange" => 0xFF8C00,
        "darkorchid" => 0x9932CC,
        "darkred" => 0x8B0000,
        "darksalmon" => 0xE9967A,
        "darkseagreen" => 0x8FBC8F,
        "darkslateblue" => 0x483D8B,
        "darkslategray" | "darkslategrey" => 0x2F4F4F,
        "darkturquoise" => 0x00CED1,
        "darkviolet" => 0x9400D3,
        "deeppink" => 0xFF1493,
        "deepskyblue" => 0x00BFFF,
        "dimgray" | "dimgrey" => 0x696969,
        "dodgerblue" => 0x1E90FF,
        "firebrick" => 0xB22222,
        "floralwhite" => 0xFFFAF0,
        "forestgreen" => 0x228B22,
        "fuchsia" => 0xFF00FF,
        "gainsboro" => 0xDCDCDC,
        "ghostwhite" => 0xF8F8FF,
        "gold" => 0xFFD700,
        "goldenrod" => 0xDAA520,
        "gray" | "grey" => 0x808080,
        "green" => 0x008000,
        "greenyellow" => 0xADFF2F,
        "honeydew" => 0xF0FFF0,
        "hotpink" => 0xFF69B4,
        "indianred" => 0xCD5C5C,
        "indigo" => 0x4B0082,
        "ivory" => 0xFFFFF0,
        "khaki" => 0xF0E68C,
        "lavender" => 0xE6E6FA,
        "lavenderblush" => 0xFFF0F5,
        "lawngreen" => 0x7CFC00,
        "lemonchiffon" => 0xFFFACD,
        "lightblue" => 0xADD8E6,
        "lightcoral" => 0xF08080,
        "lightcyan" => 0xE0FFFF,
        "lightgoldenrodyellow" => 0xFAFAD2,
        "lightgray" | "lightgrey" => 0xD3D3D3,
        "lightgreen" => 0x90EE90,
        "lightpink" => 0xFFB6C1,
        "lightsalmon" => 0xFFA07A,
        "lightseagreen" => 0x20B2AA,
        "lightskyblue" => 0x87CEFA,
        "lightslategray" | "lightslategrey" => 0x778899,
        "lightsteelblue" => 0xB0C4DE,
        "lightyellow" => 0xFFFFE0,
        "lime" => 0x00FF00,
        "limegreen" => 0x32CD32,
        "linen" => 0xFAF0E6,
        "magenta" => 0xFF00FF,
        "maroon" => 0x800000,
        "mediumaquamarine" => 0x66CDAA,
        "mediumblue" => 0x0000CD,
        "mediumorchid" => 0xBA55D3,
        "mediumpurple" => 0x9370D8,
        "mediumseagreen" => 0x3CB371,
        "mediumslateblue" => 0x7B68EE,
        "mediumspringgreen" => 0x00FA9A,
        "mediumturquoise" => 0x48D1CC,
        "mediumvioletred" => 0xC71585,
        "midnightblue" => 0x191970,
        "mintcream" => 0xF5FFFA,
        "mistyrose" => 0xFFE4E1,
        "moccasin" => 0xFFE4B5,
        "navajowhite" => 0xFFDEAD,
        "navy" => 0x000080,
        "oldlace" => 0xFDF5E6,
        "olive" => 0x808000,
        "olivedrab" => 0x6B8E23,
        "orange" => 0xFFA500,
        "orangered" => 0xFF4500,
        "orchid" => 0xDA70D6,
        "palegoldenrod" => 0xEEE8AA,
        "palegreen" => 0x98FB98,
        "paleturquoise" => 0xAFEEEE,
        "palevioletred" => 0xD87093,
        "papayawhip" => 0xFFEFD5,
        "peachpuff" => 0xFFDAB9,
        "peru" => 0xCD853F,
        "pink" => 0xFFC0CB,
        "plum" => 0xDDA0DD,
        "powderblue" => 0xB0E0E6,
        "purple" => 0x800080,
        "red" => 0xFF0000,
        "rosybrown" => 0xBC8F8F,
        "royalblue" => 0x4169E1,
        "saddlebrown" => 0x8B4513,
        "salmon" => 0xFA8072,
        "sandybrown" => 0xF4A460,
        "seagreen" => 0x2E8B57,
        "seashell" => 0xFFF5EE,
        "sienna" => 0xA0522D,
        "silver" => 0xC0C0C0,
        "skyblue" => 0x87CEEB,
        "slateblue" => 0x6A5ACD,
        "slategray" | "slategrey" => 0x708090,
        "snow" => 0xFFFAFA,
        "springgreen" => 0x00FF7F,
        "steelblue" => 0x4682B4,
        "tan" => 0xD2B48C,
        "teal" => 0x008080,
        "thistle" => 0xD8BFD8,
        "tomato" => 0xFF6347,
        "turquoise" => 0x40E0D0,
        "violet" => 0xEE82EE,
        "wheat" => 0xF5DEB3,
        "white" => 0xFFFFFF,
        "whitesmoke" => 0xF5F5F5,
        "yellow" => 0xFFFF00,
        "yellowgreen" => 0x9ACD32,
        // Archimate colors
        "business" => 0xFFFFCC,
        "application" => 0xC2F0FF,
        "motivation" => 0xCCCCFF,
        "strategy" => 0xF8E7C0,
        "technology" => 0xC9FFC9,
        "physical" => 0x97FF97,
        "implementation" => 0xFFE0E0,
        _ => return None,
    };
    Some((
        ((rgb >> 16) & 0xFF) as u8,
        ((rgb >> 8) & 0xFF) as u8,
        (rgb & 0xFF) as u8,
    ))
}

/// Try to parse `s` as a pure hex color (1, 3, 6, or 8 hex digits).
/// Returns None if the string contains any non-hex character or has
/// an unsupported length.
fn try_parse_hex(s: &str) -> Option<HColor> {
    // All characters must be hex digits
    if !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    match s.len() {
        1 => {
            let d = u8::from_str_radix(s, 16).ok()?;
            let v = (d << 4) | d;
            Some(HColor::Simple { r: v, g: v, b: v })
        }
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()?;
            let g = u8::from_str_radix(&s[1..2], 16).ok()?;
            let b = u8::from_str_radix(&s[2..3], 16).ok()?;
            Some(HColor::Simple {
                r: (r << 4) | r,
                g: (g << 4) | g,
                b: (b << 4) | b,
            })
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(HColor::Simple { r, g, b })
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some(HColor::rgba(r, g, b, a))
        }
        _ => None,
    }
}

/// Parse a simple color string: hex (#RGB, #RRGGBB, #RRGGBBAA, #R) or named color.
/// Matches Java's HColorSet.parseSimpleColor: tries hex first, then named lookup.
fn parse_simple_color(s: &str) -> Option<HColor> {
    let s = s.strip_prefix('#').unwrap_or(s);

    // Try hex first
    if let Some(color) = try_parse_hex(s) {
        return Some(color);
    }

    // Then try named color
    let (r, g, b) = named_color_rgb(s)?;
    Some(HColor::Simple { r, g, b })
}

/// Resolve a color specification string. Handles:
/// - Hex colors: "#FF0000", "#F00", "#A", "#RRGGBBAA"
/// - Named colors: "red", "LightBlue", "DarkSalmon", etc.
/// - Transparent / background: "transparent", "background"
/// - Gradient: "red-blue", "red|blue", "red/blue", "red\\blue"
///
/// Java: `klimt.color.HColorSet.parseColor`
pub fn resolve_color(name: &str) -> Option<HColor> {
    let s = name.strip_prefix('#').unwrap_or(name);

    // Transparent / background
    if s.eq_ignore_ascii_case("transparent") || s.eq_ignore_ascii_case("background") {
        return Some(HColor::None);
    }

    // Try as simple hex or named color
    if let Some(color) = parse_simple_color(s) {
        return Some(color);
    }

    // Try gradient: look for separator '-', '|', '/', '\'
    for (i, c) in s.char_indices() {
        if c == '-' || c == '|' || c == '/' || c == '\\' {
            let left = &s[..i];
            let right = &s[i + c.len_utf8()..];
            if let (Some(c1), Some(c2)) = (parse_simple_color(left), parse_simple_color(right)) {
                return Some(HColor::gradient(c1, c2, c));
            }
        }
    }

    None
}

/// Resolve a color, returning white if the name is unknown.
/// Java: `HColorSet.getColorOrWhite`
pub fn resolve_color_or_white(name: &str) -> HColor {
    resolve_color(name).unwrap_or(colors::WHITE)
}

/// List all known named color names (sorted).
pub fn named_color_names() -> Vec<&'static str> {
    let mut names = vec![
        "AliceBlue",
        "AntiqueWhite",
        "Aqua",
        "Aquamarine",
        "Azure",
        "Beige",
        "Bisque",
        "Black",
        "BlanchedAlmond",
        "Blue",
        "BlueViolet",
        "Brown",
        "BurlyWood",
        "CadetBlue",
        "Chartreuse",
        "Chocolate",
        "Coral",
        "CornflowerBlue",
        "Cornsilk",
        "Crimson",
        "Cyan",
        "DarkBlue",
        "DarkCyan",
        "DarkGoldenRod",
        "DarkGray",
        "DarkGreen",
        "DarkGrey",
        "DarkKhaki",
        "DarkMagenta",
        "DarkOliveGreen",
        "DarkOrchid",
        "DarkRed",
        "DarkSalmon",
        "DarkSeaGreen",
        "DarkSlateBlue",
        "DarkSlateGray",
        "DarkSlateGrey",
        "DarkTurquoise",
        "DarkViolet",
        "Darkorange",
        "DeepPink",
        "DeepSkyBlue",
        "DimGray",
        "DimGrey",
        "DodgerBlue",
        "FireBrick",
        "FloralWhite",
        "ForestGreen",
        "Fuchsia",
        "Gainsboro",
        "GhostWhite",
        "Gold",
        "GoldenRod",
        "Gray",
        "Green",
        "GreenYellow",
        "Grey",
        "HoneyDew",
        "HotPink",
        "IndianRed",
        "Indigo",
        "Ivory",
        "Khaki",
        "Lavender",
        "LavenderBlush",
        "LawnGreen",
        "LemonChiffon",
        "LightBlue",
        "LightCoral",
        "LightCyan",
        "LightGoldenRodYellow",
        "LightGray",
        "LightGreen",
        "LightGrey",
        "LightPink",
        "LightSalmon",
        "LightSeaGreen",
        "LightSkyBlue",
        "LightSlateGray",
        "LightSlateGrey",
        "LightSteelBlue",
        "LightYellow",
        "Lime",
        "LimeGreen",
        "Linen",
        "Magenta",
        "Maroon",
        "MediumAquaMarine",
        "MediumBlue",
        "MediumOrchid",
        "MediumPurple",
        "MediumSeaGreen",
        "MediumSlateBlue",
        "MediumSpringGreen",
        "MediumTurquoise",
        "MediumVioletRed",
        "MidnightBlue",
        "MintCream",
        "MistyRose",
        "Moccasin",
        "NavajoWhite",
        "Navy",
        "OldLace",
        "Olive",
        "OliveDrab",
        "Orange",
        "OrangeRed",
        "Orchid",
        "PaleGoldenRod",
        "PaleGreen",
        "PaleTurquoise",
        "PaleVioletRed",
        "PapayaWhip",
        "PeachPuff",
        "Peru",
        "Pink",
        "Plum",
        "PowderBlue",
        "Purple",
        "Red",
        "RosyBrown",
        "RoyalBlue",
        "SaddleBrown",
        "Salmon",
        "SandyBrown",
        "SeaGreen",
        "SeaShell",
        "Sienna",
        "Silver",
        "SkyBlue",
        "SlateBlue",
        "SlateGray",
        "SlateGrey",
        "Snow",
        "SpringGreen",
        "SteelBlue",
        "Tan",
        "Teal",
        "Thistle",
        "Tomato",
        "Turquoise",
        "Violet",
        "Wheat",
        "White",
        "WhiteSmoke",
        "Yellow",
        "YellowGreen",
        // Archimate
        "APPLICATION",
        "BUSINESS",
        "IMPLEMENTATION",
        "MOTIVATION",
        "PHYSICAL",
        "STRATEGY",
        "TECHNOLOGY",
    ];
    names.sort_unstable();
    names
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── HColor::simple parsing ───────────────────────────────────────

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
    fn parse_hex_1() {
        let c = HColor::simple("#A");
        assert_eq!(c.to_svg(), "#AAAAAA");
    }

    #[test]
    fn parse_hex_8_with_alpha() {
        let c = HColor::simple("#FF880080");
        assert_eq!(c.to_svg(), "#FF880080");
        assert_eq!(c.alpha(), 0x80);
    }

    #[test]
    fn parse_hex_8_fully_opaque() {
        // Alpha 0xFF should collapse to Simple
        let c = HColor::simple("#FF8800FF");
        assert_eq!(c.to_svg(), "#FF8800");
        assert!(matches!(c, HColor::Simple { .. }));
    }

    #[test]
    fn parse_without_hash() {
        let c = HColor::simple("00FF00");
        assert_eq!(c.to_svg(), "#00FF00");
    }

    #[test]
    fn parse_invalid() {
        assert!(HColor::simple("xyz").is_none());
        assert!(HColor::simple("").is_none());
    }

    // ── to_rgb / rgb_components ──────────────────────────────────────

    #[test]
    fn to_rgb_int() {
        let c = HColor::rgb(0x12, 0x34, 0x56);
        assert_eq!(c.to_rgb(), 0x123456);
    }

    #[test]
    fn rgb_components_simple() {
        let c = HColor::rgb(10, 20, 30);
        assert_eq!(c.rgb_components(), (10, 20, 30));
    }

    #[test]
    fn rgb_components_with_alpha() {
        let c = HColor::rgba(10, 20, 30, 128);
        assert_eq!(c.rgb_components(), (10, 20, 30));
        assert_eq!(c.alpha(), 128);
    }

    // ── none / transparent ───────────────────────────────────────────

    #[test]
    fn none_renders_none() {
        assert_eq!(HColor::none().to_svg(), "none");
    }

    #[test]
    fn none_is_transparent() {
        assert!(HColor::none().is_transparent());
    }

    #[test]
    fn rgba_zero_alpha_is_transparent() {
        let c = HColor::rgba(255, 0, 0, 0);
        assert!(c.is_transparent());
        assert!(c.is_none());
    }

    // ── resolve_color: named colors ──────────────────────────────────

    #[test]
    fn resolve_named_red() {
        assert_eq!(resolve_color("red").unwrap().to_svg(), "#FF0000");
    }

    #[test]
    fn resolve_named_case_insensitive() {
        assert_eq!(resolve_color("Red").unwrap().to_svg(), "#FF0000");
        assert_eq!(resolve_color("RED").unwrap().to_svg(), "#FF0000");
        assert_eq!(resolve_color("LightBlue").unwrap().to_svg(), "#ADD8E6");
        assert_eq!(resolve_color("lightblue").unwrap().to_svg(), "#ADD8E6");
    }

    #[test]
    fn resolve_named_green_is_008000() {
        // SVG "green" is #008000, not #00FF00
        assert_eq!(resolve_color("green").unwrap().to_svg(), "#008000");
    }

    #[test]
    fn resolve_named_lime_is_00ff00() {
        assert_eq!(resolve_color("lime").unwrap().to_svg(), "#00FF00");
    }

    #[test]
    fn resolve_unknown_returns_none() {
        assert!(resolve_color("nonexistent_xyz").is_none());
    }

    #[test]
    fn resolve_transparent() {
        let c = resolve_color("transparent").unwrap();
        assert!(c.is_none());
    }

    #[test]
    fn resolve_background() {
        let c = resolve_color("background").unwrap();
        assert!(c.is_none());
    }

    #[test]
    fn resolve_hex_color() {
        assert_eq!(resolve_color("#FF8800").unwrap().to_svg(), "#FF8800");
        assert_eq!(resolve_color("FF8800").unwrap().to_svg(), "#FF8800");
    }

    // ── resolve_color: complete named color table ────────────────────

    #[test]
    fn resolve_all_svgx11_colors() {
        let cases: &[(&str, u32)] = &[
            ("AliceBlue", 0xF0F8FF),
            ("AntiqueWhite", 0xFAEBD7),
            ("Aqua", 0x00FFFF),
            ("Aquamarine", 0x7FFFD4),
            ("Azure", 0xF0FFFF),
            ("Beige", 0xF5F5DC),
            ("Bisque", 0xFFE4C4),
            ("Black", 0x000000),
            ("BlanchedAlmond", 0xFFEBCD),
            ("Blue", 0x0000FF),
            ("BlueViolet", 0x8A2BE2),
            ("Brown", 0xA52A2A),
            ("BurlyWood", 0xDEB887),
            ("CadetBlue", 0x5F9EA0),
            ("Chartreuse", 0x7FFF00),
            ("Chocolate", 0xD2691E),
            ("Coral", 0xFF7F50),
            ("CornflowerBlue", 0x6495ED),
            ("Cornsilk", 0xFFF8DC),
            ("Crimson", 0xDC143C),
            ("Cyan", 0x00FFFF),
            ("DarkBlue", 0x00008B),
            ("DarkCyan", 0x008B8B),
            ("DarkGoldenRod", 0xB8860B),
            ("DarkGray", 0xA9A9A9),
            ("DarkGrey", 0xA9A9A9),
            ("DarkGreen", 0x006400),
            ("DarkKhaki", 0xBDB76B),
            ("DarkMagenta", 0x8B008B),
            ("DarkOliveGreen", 0x556B2F),
            ("Darkorange", 0xFF8C00),
            ("DarkOrchid", 0x9932CC),
            ("DarkRed", 0x8B0000),
            ("DarkSalmon", 0xE9967A),
            ("DarkSeaGreen", 0x8FBC8F),
            ("DarkSlateBlue", 0x483D8B),
            ("DarkSlateGray", 0x2F4F4F),
            ("DarkSlateGrey", 0x2F4F4F),
            ("DarkTurquoise", 0x00CED1),
            ("DarkViolet", 0x9400D3),
            ("DeepPink", 0xFF1493),
            ("DeepSkyBlue", 0x00BFFF),
            ("DimGray", 0x696969),
            ("DimGrey", 0x696969),
            ("DodgerBlue", 0x1E90FF),
            ("FireBrick", 0xB22222),
            ("FloralWhite", 0xFFFAF0),
            ("ForestGreen", 0x228B22),
            ("Fuchsia", 0xFF00FF),
            ("Gainsboro", 0xDCDCDC),
            ("GhostWhite", 0xF8F8FF),
            ("Gold", 0xFFD700),
            ("GoldenRod", 0xDAA520),
            ("Gray", 0x808080),
            ("Grey", 0x808080),
            ("Green", 0x008000),
            ("GreenYellow", 0xADFF2F),
            ("HoneyDew", 0xF0FFF0),
            ("HotPink", 0xFF69B4),
            ("IndianRed", 0xCD5C5C),
            ("Indigo", 0x4B0082),
            ("Ivory", 0xFFFFF0),
            ("Khaki", 0xF0E68C),
            ("Lavender", 0xE6E6FA),
            ("LavenderBlush", 0xFFF0F5),
            ("LawnGreen", 0x7CFC00),
            ("LemonChiffon", 0xFFFACD),
            ("LightBlue", 0xADD8E6),
            ("LightCoral", 0xF08080),
            ("LightCyan", 0xE0FFFF),
            ("LightGoldenRodYellow", 0xFAFAD2),
            ("LightGray", 0xD3D3D3),
            ("LightGrey", 0xD3D3D3),
            ("LightGreen", 0x90EE90),
            ("LightPink", 0xFFB6C1),
            ("LightSalmon", 0xFFA07A),
            ("LightSeaGreen", 0x20B2AA),
            ("LightSkyBlue", 0x87CEFA),
            ("LightSlateGray", 0x778899),
            ("LightSlateGrey", 0x778899),
            ("LightSteelBlue", 0xB0C4DE),
            ("LightYellow", 0xFFFFE0),
            ("Lime", 0x00FF00),
            ("LimeGreen", 0x32CD32),
            ("Linen", 0xFAF0E6),
            ("Magenta", 0xFF00FF),
            ("Maroon", 0x800000),
            ("MediumAquaMarine", 0x66CDAA),
            ("MediumBlue", 0x0000CD),
            ("MediumOrchid", 0xBA55D3),
            ("MediumPurple", 0x9370D8),
            ("MediumSeaGreen", 0x3CB371),
            ("MediumSlateBlue", 0x7B68EE),
            ("MediumSpringGreen", 0x00FA9A),
            ("MediumTurquoise", 0x48D1CC),
            ("MediumVioletRed", 0xC71585),
            ("MidnightBlue", 0x191970),
            ("MintCream", 0xF5FFFA),
            ("MistyRose", 0xFFE4E1),
            ("Moccasin", 0xFFE4B5),
            ("NavajoWhite", 0xFFDEAD),
            ("Navy", 0x000080),
            ("OldLace", 0xFDF5E6),
            ("Olive", 0x808000),
            ("OliveDrab", 0x6B8E23),
            ("Orange", 0xFFA500),
            ("OrangeRed", 0xFF4500),
            ("Orchid", 0xDA70D6),
            ("PaleGoldenRod", 0xEEE8AA),
            ("PaleGreen", 0x98FB98),
            ("PaleTurquoise", 0xAFEEEE),
            ("PaleVioletRed", 0xD87093),
            ("PapayaWhip", 0xFFEFD5),
            ("PeachPuff", 0xFFDAB9),
            ("Peru", 0xCD853F),
            ("Pink", 0xFFC0CB),
            ("Plum", 0xDDA0DD),
            ("PowderBlue", 0xB0E0E6),
            ("Purple", 0x800080),
            ("Red", 0xFF0000),
            ("RosyBrown", 0xBC8F8F),
            ("RoyalBlue", 0x4169E1),
            ("SaddleBrown", 0x8B4513),
            ("Salmon", 0xFA8072),
            ("SandyBrown", 0xF4A460),
            ("SeaGreen", 0x2E8B57),
            ("SeaShell", 0xFFF5EE),
            ("Sienna", 0xA0522D),
            ("Silver", 0xC0C0C0),
            ("SkyBlue", 0x87CEEB),
            ("SlateBlue", 0x6A5ACD),
            ("SlateGray", 0x708090),
            ("SlateGrey", 0x708090),
            ("Snow", 0xFFFAFA),
            ("SpringGreen", 0x00FF7F),
            ("SteelBlue", 0x4682B4),
            ("Tan", 0xD2B48C),
            ("Teal", 0x008080),
            ("Thistle", 0xD8BFD8),
            ("Tomato", 0xFF6347),
            ("Turquoise", 0x40E0D0),
            ("Violet", 0xEE82EE),
            ("Wheat", 0xF5DEB3),
            ("White", 0xFFFFFF),
            ("WhiteSmoke", 0xF5F5F5),
            ("Yellow", 0xFFFF00),
            ("YellowGreen", 0x9ACD32),
        ];
        for &(name, expected_rgb) in cases {
            let color = resolve_color(name).unwrap_or_else(|| panic!("color '{}' not found", name));
            assert_eq!(
                color.to_rgb(),
                expected_rgb,
                "color '{}': expected #{:06X}, got #{:06X}",
                name,
                expected_rgb,
                color.to_rgb()
            );
        }
    }

    #[test]
    fn resolve_archimate_colors() {
        assert_eq!(resolve_color("BUSINESS").unwrap().to_rgb(), 0xFFFFCC);
        assert_eq!(resolve_color("APPLICATION").unwrap().to_rgb(), 0xC2F0FF);
        assert_eq!(resolve_color("MOTIVATION").unwrap().to_rgb(), 0xCCCCFF);
        assert_eq!(resolve_color("STRATEGY").unwrap().to_rgb(), 0xF8E7C0);
        assert_eq!(resolve_color("TECHNOLOGY").unwrap().to_rgb(), 0xC9FFC9);
        assert_eq!(resolve_color("PHYSICAL").unwrap().to_rgb(), 0x97FF97);
        assert_eq!(resolve_color("IMPLEMENTATION").unwrap().to_rgb(), 0xFFE0E0);
    }

    // ── Gradient parsing ─────────────────────────────────────────────

    #[test]
    fn resolve_gradient_dash() {
        let c = resolve_color("red-blue").unwrap();
        assert!(c.is_gradient());
        if let HColor::Gradient {
            color1,
            color2,
            policy,
        } = &c
        {
            assert_eq!(color1.to_svg(), "#FF0000");
            assert_eq!(color2.to_svg(), "#0000FF");
            assert_eq!(*policy, '-');
        } else {
            panic!("expected Gradient");
        }
    }

    #[test]
    fn resolve_gradient_pipe() {
        let c = resolve_color("red|blue").unwrap();
        assert_eq!(c.gradient_policy(), Some('|'));
    }

    #[test]
    fn resolve_gradient_slash() {
        let c = resolve_color("red/blue").unwrap();
        assert_eq!(c.gradient_policy(), Some('/'));
    }

    #[test]
    fn resolve_gradient_backslash() {
        let c = resolve_color("red\\blue").unwrap();
        assert_eq!(c.gradient_policy(), Some('\\'));
    }

    #[test]
    fn resolve_gradient_hex_colors() {
        let c = resolve_color("FF0000-0000FF").unwrap();
        assert!(c.is_gradient());
    }

    #[test]
    fn gradient_no_gradient_fallback() {
        let c = resolve_color("red-blue").unwrap();
        let flat = c.no_gradient();
        assert_eq!(flat.to_svg(), "#FF0000");
    }

    #[test]
    fn gradient_color_at() {
        let c = HColor::gradient(HColor::rgb(0, 0, 0), HColor::rgb(255, 255, 255), '-');
        let mid = c.gradient_color_at(0.5, 255);
        assert_eq!(mid.rgb_components(), (127, 127, 127));
    }

    // ── HColor methods ──────────────────────────────────────────────

    #[test]
    fn gray_scale_white() {
        assert_eq!(HColor::rgb(255, 255, 255).gray_scale(), 255);
    }

    #[test]
    fn gray_scale_black() {
        assert_eq!(HColor::rgb(0, 0, 0).gray_scale(), 0);
    }

    #[test]
    fn is_dark_black() {
        assert!(HColor::rgb(0, 0, 0).is_dark());
    }

    #[test]
    fn is_dark_white() {
        assert!(!HColor::rgb(255, 255, 255).is_dark());
    }

    #[test]
    fn opposite_of_black_is_white() {
        let opp = HColor::rgb(0, 0, 0).opposite();
        assert_eq!(opp, HColor::rgb(255, 255, 255));
    }

    #[test]
    fn opposite_of_white_is_black() {
        let opp = HColor::rgb(255, 255, 255).opposite();
        assert_eq!(opp, HColor::rgb(0, 0, 0));
    }

    #[test]
    fn monochrome() {
        let c = HColor::rgb(255, 0, 0);
        let mono = c.as_monochrome();
        // YIQ: (255*299 + 0*587 + 0*114) / 1000 = 76
        assert_eq!(mono.rgb_components(), (76, 76, 76));
    }

    #[test]
    fn is_gray_true() {
        assert!(HColor::rgb(128, 128, 128).is_gray());
    }

    #[test]
    fn is_gray_false() {
        assert!(!HColor::rgb(128, 127, 128).is_gray());
    }

    #[test]
    fn distance_same_color() {
        let c = HColor::rgb(100, 100, 100);
        assert_eq!(c.distance_to(&c), 0);
    }

    #[test]
    fn distance_black_white() {
        let d = HColor::rgb(0, 0, 0).distance_to(&HColor::rgb(255, 255, 255));
        // 255*299 + 255*587 + 255*114 = 255*1000 = 255000
        assert_eq!(d, 255000);
    }

    #[test]
    fn darken_simple() {
        let c = HColor::rgb(200, 100, 50);
        let d = c.darken(0.5);
        assert_eq!(d.rgb_components(), (100, 50, 25));
    }

    #[test]
    fn interpolate_midpoint() {
        let c1 = HColor::rgb(0, 0, 0);
        let c2 = HColor::rgb(200, 100, 50);
        let mid = c1.interpolate(&c2, 0.5);
        assert_eq!(mid.rgb_components(), (100, 50, 25));
    }

    // ── HSL round-trip ──────────────────────────────────────────────

    #[test]
    fn hsl_round_trip_red() {
        let hsl = rgb_to_hsl(255, 0, 0);
        assert!((hsl[0] - 0.0).abs() < 1.0); // hue ~ 0
        let (r, g, b) = hsl_to_rgb(hsl[0], hsl[1], hsl[2]);
        assert_eq!((r, g, b), (255, 0, 0));
    }

    #[test]
    fn hsl_round_trip_green() {
        let hsl = rgb_to_hsl(0, 128, 0);
        let (r, g, b) = hsl_to_rgb(hsl[0], hsl[1], hsl[2]);
        // Allow +/- 1 due to floating point
        assert!((r as i32).abs() <= 1);
        assert!((g as i32 - 128).abs() <= 1);
        assert!((b as i32).abs() <= 1);
    }

    #[test]
    fn hsl_round_trip_gray() {
        let hsl = rgb_to_hsl(128, 128, 128);
        assert!(hsl[1].abs() < 0.01); // saturation ~ 0
        let (r, g, b) = hsl_to_rgb(hsl[0], hsl[1], hsl[2]);
        assert!((r as i32 - 128).abs() <= 1);
        assert!((g as i32 - 128).abs() <= 1);
        assert!((b as i32 - 128).abs() <= 1);
    }

    #[test]
    fn darken_hsl() {
        let c = HColor::rgb(200, 100, 50);
        let d = c.darken_hsl(50);
        // After 50% darken, luminance is halved
        let (r, g, b) = d.rgb_components();
        // The darkened color should be noticeably darker
        assert!(r < 200);
        assert!(g < 100);
        assert!(b < 50);
    }

    #[test]
    fn lighten_hsl() {
        let c = HColor::rgb(100, 50, 25);
        let l = c.lighten_hsl(50);
        let (r, g, b) = l.rgb_components();
        // The lightened color should be noticeably lighter
        assert!(r > 100);
        assert!(g > 50);
    }

    // ── LinearGradient ──────────────────────────────────────────────

    #[test]
    fn linear_gradient_basic() {
        let lg = HColorLinearGradient::new(
            0.0,
            0.0,
            1.0,
            0.0,
            false,
            SpreadMethod::Pad,
            vec![
                GradientStop {
                    offset: 0.0,
                    color: HColor::rgb(255, 0, 0),
                    opacity: 1.0,
                },
                GradientStop {
                    offset: 1.0,
                    color: HColor::rgb(0, 0, 255),
                    opacity: 1.0,
                },
            ],
        );
        assert_eq!(lg.color1().to_svg(), "#FF0000");
        assert_eq!(lg.stops.len(), 2);
        assert!(!lg.user_space_on_use);
    }

    #[test]
    fn linear_gradient_stop_normalization() {
        let lg = HColorLinearGradient::new(
            0.0,
            0.0,
            1.0,
            0.0,
            false,
            SpreadMethod::Pad,
            vec![
                GradientStop {
                    offset: 0.5,
                    color: HColor::rgb(0, 255, 0),
                    opacity: 1.0,
                },
                GradientStop {
                    offset: 0.0,
                    color: HColor::rgb(255, 0, 0),
                    opacity: 1.0,
                },
                GradientStop {
                    offset: 1.5, // clamped to 1.0
                    color: HColor::rgb(0, 0, 255),
                    opacity: 1.0,
                },
            ],
        );
        // Should be sorted and clamped
        assert_eq!(lg.stops[0].offset, 0.0);
        assert_eq!(lg.stops[1].offset, 0.5);
        assert!((lg.stops[2].offset - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn linear_gradient_duplicate_offsets_nudged() {
        let lg = HColorLinearGradient::new(
            0.0,
            0.0,
            1.0,
            0.0,
            false,
            SpreadMethod::Pad,
            vec![
                GradientStop {
                    offset: 0.5,
                    color: HColor::rgb(255, 0, 0),
                    opacity: 1.0,
                },
                GradientStop {
                    offset: 0.5,
                    color: HColor::rgb(0, 255, 0),
                    opacity: 1.0,
                },
            ],
        );
        assert!(lg.stops[1].offset > lg.stops[0].offset);
    }

    #[test]
    fn linear_gradient_as_hcolor() {
        let lg = HColorLinearGradient::new(
            0.0,
            0.0,
            1.0,
            0.0,
            true,
            SpreadMethod::Reflect,
            vec![GradientStop {
                offset: 0.0,
                color: HColor::rgb(255, 0, 0),
                opacity: 0.5,
            }],
        );
        let hc = HColor::LinearGradient(lg);
        assert!(hc.is_gradient());
        assert_eq!(hc.to_svg(), "#FF0000");
        assert_eq!(hc.no_gradient().to_svg(), "#FF0000");
    }

    // ── as_string ───────────────────────────────────────────────────

    #[test]
    fn as_string_transparent() {
        assert_eq!(HColor::None.as_string(), "transparent");
    }

    #[test]
    fn as_string_simple() {
        assert_eq!(HColor::rgb(255, 0, 128).as_string(), "#FF0080");
    }

    #[test]
    fn as_string_with_alpha() {
        let c = HColor::WithAlpha {
            r: 0xFF,
            g: 0x00,
            b: 0x80,
            a: 0x40,
        };
        // Java format: #aarrggbb lowercase
        assert_eq!(c.as_string(), "#40ff0080");
    }

    // ── Color constants ─────────────────────────────────────────────

    #[test]
    fn color_constants() {
        assert_eq!(colors::BLACK.to_svg(), "#000000");
        assert_eq!(colors::WHITE.to_svg(), "#FFFFFF");
        assert_eq!(colors::RED.to_svg(), "#FF0000");
        assert_eq!(colors::GREEN.to_svg(), "#00FF00");
        assert_eq!(colors::BLUE.to_svg(), "#0000FF");
        assert_eq!(colors::GRAY.to_svg(), "#808080");
        assert_eq!(colors::MY_YELLOW.to_svg(), "#FEFECE");
        assert_eq!(colors::MY_RED.to_svg(), "#A80036");
    }

    // ── named_color_names ───────────────────────────────────────────

    #[test]
    fn named_colors_count() {
        let names = named_color_names();
        // 140 SVG/X11 colors + 7 Archimate = 147 unique entries in the list
        // (gray/grey duplicates are listed separately)
        assert!(names.len() >= 147, "got {} names", names.len());
    }

    #[test]
    fn named_colors_sorted() {
        let names = named_color_names();
        for w in names.windows(2) {
            assert!(
                w[0] <= w[1],
                "names not sorted: '{}' > '{}'",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn all_named_colors_resolvable() {
        for name in named_color_names() {
            assert!(
                resolve_color(name).is_some(),
                "named color '{}' not resolvable",
                name
            );
        }
    }

    // ── resolve_color_or_white ───────────────────────────────────────

    #[test]
    fn resolve_or_white_known() {
        assert_eq!(resolve_color_or_white("red").to_svg(), "#FF0000");
    }

    #[test]
    fn resolve_or_white_unknown() {
        assert_eq!(resolve_color_or_white("xyzzy").to_svg(), "#FFFFFF");
    }
}
