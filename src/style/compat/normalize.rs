/// Normalize a color value for SVG output.
///
/// - `#RGB` (3-char hex) -> `#RRGGBB`
/// - `#RRGGBB` -> as-is
/// - `#AARRGGBB` (8-char hex with alpha) -> `#RRGGBB` (alpha dropped for SVG)
/// - `transparent` -> `none`
/// - Named colors (e.g., `red`, `LightBlue`) -> pass through (SVG supports them)
pub fn normalize_color(color: &str) -> String {
    let trimmed = color.trim();

    // Handle "transparent"
    if trimmed.eq_ignore_ascii_case("transparent") {
        return "none".to_string();
    }

    // Handle hex colors — Java normalizes to uppercase #RRGGBB
    if let Some(hex) = trimmed.strip_prefix('#') {
        let hex_clean: String = hex
            .chars()
            .filter(char::is_ascii_hexdigit)
            .map(|c| c.to_ascii_uppercase())
            .collect();

        return match hex_clean.len() {
            3 => {
                // #RGB -> #RRGGBB (uppercase)
                let mut expanded = String::with_capacity(7);
                expanded.push('#');
                for c in hex_clean.chars() {
                    expanded.push(c);
                    expanded.push(c);
                }
                expanded
            }
            6 => {
                format!("#{hex_clean}")
            }
            8 => {
                // #AARRGGBB -> #RRGGBB (drop alpha, uppercase)
                format!("#{}", &hex_clean[2..])
            }
            _ => trimmed.to_string(),
        };
    }

    // Named colors: convert to hex (#RRGGBB) to match Java PlantUML output.
    if let Some(hex) = named_color_to_hex(trimmed) {
        return hex.to_string();
    }

    // Bare hex without '#' prefix (e.g. "22A722" from parser)
    let all_hex = trimmed.len() == 6
        && trimmed.chars().all(|c| c.is_ascii_hexdigit());
    if all_hex {
        return format!("#{}", trimmed.to_ascii_uppercase());
    }

    trimmed.to_string()
}

/// Convert a named CSS/SVG color to its hex equivalent.
/// Java PlantUML always renders colors as hex codes.
fn named_color_to_hex(name: &str) -> Option<&'static str> {
    let lower: String = name.to_lowercase();
    match lower.as_str() {
        "black" => Some("#000000"),
        "white" => Some("#FFFFFF"),
        "red" => Some("#FF0000"),
        "green" => Some("#008000"),
        "blue" => Some("#0000FF"),
        "yellow" => Some("#FFFF00"),
        "cyan" | "aqua" => Some("#00FFFF"),
        "magenta" | "fuchsia" => Some("#FF00FF"),
        "gray" | "grey" => Some("#808080"),
        "darkgray" | "darkgrey" => Some("#A9A9A9"),
        "lightgray" | "lightgrey" => Some("#D3D3D3"),
        "orange" => Some("#FFA500"),
        "pink" => Some("#FFC0CB"),
        "purple" => Some("#800080"),
        "brown" => Some("#A52A2A"),
        "navy" => Some("#000080"),
        "teal" => Some("#008080"),
        "olive" => Some("#808000"),
        "maroon" => Some("#800000"),
        "lime" => Some("#00FF00"),
        "silver" => Some("#C0C0C0"),
        "gold" => Some("#FFD700"),
        "indigo" => Some("#4B0082"),
        "violet" => Some("#EE82EE"),
        "coral" => Some("#FF7F50"),
        "salmon" => Some("#FA8072"),
        "tomato" => Some("#FF6347"),
        "orangered" => Some("#FF4500"),
        "crimson" => Some("#DC143C"),
        "darkblue" => Some("#00008B"),
        "darkgreen" => Some("#006400"),
        "darkred" => Some("#8B0000"),
        "lightblue" => Some("#ADD8E6"),
        "lightgreen" => Some("#90EE90"),
        "lightyellow" => Some("#FFFFE0"),
        "skyblue" => Some("#87CEEB"),
        "steelblue" => Some("#4682B4"),
        "royalblue" => Some("#4169E1"),
        "forestgreen" => Some("#228B22"),
        "seagreen" => Some("#2E8B57"),
        "limegreen" => Some("#32CD32"),
        "chocolate" => Some("#D2691E"),
        "sienna" => Some("#A0522D"),
        "tan" => Some("#D2B48C"),
        "wheat" => Some("#F5DEB3"),
        "khaki" => Some("#F0E68C"),
        "plum" => Some("#DDA0DD"),
        "orchid" => Some("#DA70D6"),
        "turquoise" => Some("#40E0D0"),
        "slategray" | "slategrey" => Some("#708090"),
        "dimgray" | "dimgrey" => Some("#696969"),
        "ivory" => Some("#FFFFF0"),
        "beige" => Some("#F5F5DC"),
        "linen" => Some("#FAF0E6"),
        "honeydew" => Some("#F0FFF0"),
        "mintcream" => Some("#F5FFFA"),
        "lavender" => Some("#E6E6FA"),
        "mistyrose" => Some("#FFE4E1"),
        "cornsilk" => Some("#FFF8DC"),
        _ => None,
    }
}
