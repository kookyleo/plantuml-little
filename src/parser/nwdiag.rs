use log::{debug, trace, warn};

use crate::model::nwdiag::{Network, NwdiagDiagram, ServerRef};
use crate::Result;

/// Parse nwdiag diagram source text into an NwdiagDiagram IR.
pub fn parse_nwdiag_diagram(source: &str) -> Result<NwdiagDiagram> {
    let block = super::common::extract_block(source).unwrap_or_else(|| source.to_string());

    let mut networks: Vec<Network> = Vec::new();
    let mut title: Option<String> = None;

    let lines: Vec<&str> = block.lines().collect();
    let mut i = 0;

    // Skip optional `nwdiag {` wrapper
    let mut nwdiag_wrapper = false;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("//") {
            i += 1;
            continue;
        }
        if trimmed.starts_with("nwdiag") && trimmed.contains('{') {
            nwdiag_wrapper = true;
            debug!("nwdiag parser: found nwdiag wrapper");
            i += 1;
            break;
        }
        break;
    }

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        // End of nwdiag wrapper
        if nwdiag_wrapper && trimmed == "}" {
            debug!("nwdiag parser: end of nwdiag wrapper");
            i += 1;
            continue;
        }

        // Title
        if let Some(rest) = trimmed.strip_prefix("title ") {
            let rest = rest.trim();
            if !rest.is_empty() {
                title = Some(rest.to_string());
                debug!("nwdiag parser: title = {rest:?}");
            }
            i += 1;
            continue;
        }

        // Network block
        if trimmed.starts_with("network ") {
            let (network, next_i) = parse_network_block(&lines, i)?;
            debug!(
                "nwdiag parser: network '{}' with {} servers",
                network.name,
                network.servers.len()
            );
            networks.push(network);
            i = next_i;
            continue;
        }

        trace!("nwdiag parser: skipping line: {trimmed}");
        i += 1;
    }

    debug!("nwdiag parser: done - {} networks", networks.len());

    Ok(NwdiagDiagram { networks, title })
}

/// Parse a `network name { ... }` block starting at line `start`.
fn parse_network_block(lines: &[&str], start: usize) -> Result<(Network, usize)> {
    let trimmed = lines[start].trim();
    let rest = trimmed.strip_prefix("network ").unwrap();

    // Extract name: could be quoted or unquoted, followed by `{`
    let (name, rest) = parse_network_name(rest);
    let rest = rest.trim();

    let mut address: Option<String> = None;
    let mut color: Option<String> = None;
    let mut servers: Vec<ServerRef> = Vec::new();

    // Check if `{` is present on the same line
    if !rest.starts_with('{') {
        warn!("nwdiag parser: expected '{{' in network declaration");
        return Ok((
            Network {
                name,
                address,
                color,
                servers,
            },
            start + 1,
        ));
    }

    let mut i = start + 1;
    while i < lines.len() {
        let line_trimmed = lines[i].trim();

        // Skip blank lines and comments
        if line_trimmed.is_empty()
            || line_trimmed.starts_with('\'')
            || line_trimmed.starts_with("//")
        {
            i += 1;
            continue;
        }

        // End of network block
        if line_trimmed == "}" || line_trimmed.starts_with('}') {
            return Ok((
                Network {
                    name,
                    address,
                    color,
                    servers,
                },
                i + 1,
            ));
        }

        // Network-level `address = "..."` (without semicolon)
        if let Some(addr) = try_parse_key_value(line_trimmed, "address") {
            address = Some(addr);
            trace!("nwdiag parser: network address = {address:?}");
            i += 1;
            continue;
        }

        // Network-level `color = "#xxx"`
        if let Some(c) = try_parse_key_value(line_trimmed, "color") {
            color = Some(c);
            trace!("nwdiag parser: network color = {color:?}");
            i += 1;
            continue;
        }

        // Server reference: `server_name [address = "...", description = "..."];`
        // or just `server_name;`
        if let Some(server) = try_parse_server_ref(line_trimmed) {
            trace!("nwdiag parser: server '{}' in network", server.name);
            servers.push(server);
            i += 1;
            continue;
        }

        trace!("nwdiag parser: skipping line inside network: {line_trimmed}");
        i += 1;
    }

    warn!("nwdiag parser: reached end of input without closing brace for network '{name}'");
    Ok((
        Network {
            name,
            address,
            color,
            servers,
        },
        i,
    ))
}

/// Parse network name: `"Quoted Name" {` or `simple_name {`
fn parse_network_name(s: &str) -> (String, String) {
    let trimmed = s.trim();

    if let Some(after_quote) = trimmed.strip_prefix('"') {
        if let Some(end_quote) = after_quote.find('"') {
            let name = after_quote[..end_quote].to_string();
            let rest = after_quote[end_quote + 1..].to_string();
            return (name, rest);
        }
    }

    // Unquoted: name is everything up to `{` or whitespace
    let end = trimmed
        .find(|c: char| c == '{' || c.is_whitespace())
        .unwrap_or(trimmed.len());
    let name = trimmed[..end].to_string();
    let rest = trimmed[end..].to_string();
    (name, rest)
}

/// Try to parse `key = "value"` or `key = value` without semicolon.
fn try_parse_key_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(';');
    let pattern = format!("{key} =");

    if !trimmed.starts_with(&pattern) {
        // Also try without space before =
        let pattern2 = format!("{key}=");
        if !trimmed.starts_with(&pattern2) {
            return None;
        }
        let rest = trimmed.strip_prefix(&pattern2)?.trim();
        return Some(unquote(rest));
    }

    let rest = trimmed.strip_prefix(&pattern)?.trim();
    Some(unquote(rest))
}

/// Remove surrounding quotes if present.
fn unquote(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Try to parse a server reference line.
///
/// Patterns:
///   `server_name;`
///   `server_name [address = "...", description = "..."];`
///   `server_name [address = "..."];`
fn try_parse_server_ref(line: &str) -> Option<ServerRef> {
    let trimmed = line.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        return None;
    }

    // Check if there's a `[...]` section
    if let Some(bracket_start) = trimmed.find('[') {
        let name = trimmed[..bracket_start].trim().to_string();
        if name.is_empty() || name.contains('=') {
            return None;
        }

        let bracket_end = trimmed.rfind(']')?;
        let attrs_str = &trimmed[bracket_start + 1..bracket_end];

        let mut address: Option<String> = None;
        let mut description: Option<String> = None;

        // Parse key=value pairs separated by commas
        for part in split_attrs(attrs_str) {
            let part = part.trim();
            if let Some(val) = try_parse_key_value(part, "address") {
                address = Some(val);
            } else if let Some(val) = try_parse_key_value(part, "description") {
                description = Some(val);
            }
        }

        return Some(ServerRef {
            name,
            address,
            description,
        });
    }

    // Simple server name (no brackets)
    let name = trimmed.trim().to_string();
    // Make sure it looks like a server name (no special chars suggesting it's something else)
    if name.is_empty() || name.contains('=') || name.contains('{') || name.contains('}') {
        return None;
    }

    Some(ServerRef {
        name,
        address: None,
        description: None,
    })
}

/// Split attribute string by commas, respecting quoted strings.
fn split_attrs(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in s.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            ',' if !in_quotes => {
                parts.push(current.clone());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Empty nwdiag
    #[test]
    fn test_parse_empty() {
        let src = "@startnwdiag\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert!(d.networks.is_empty());
        assert!(d.title.is_none());
    }

    // 2. Single network with one server
    #[test]
    fn test_parse_single_network() {
        let src = "@startnwdiag\nnwdiag {\n  network dmz {\n    web01;\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks.len(), 1);
        assert_eq!(d.networks[0].name, "dmz");
        assert_eq!(d.networks[0].servers.len(), 1);
        assert_eq!(d.networks[0].servers[0].name, "web01");
    }

    // 3. Network with address
    #[test]
    fn test_parse_network_address() {
        let src = "@startnwdiag\nnwdiag {\n  network dmz {\n    address = \"210.x.x.x/24\"\n    web01;\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks[0].address.as_deref(), Some("210.x.x.x/24"));
    }

    // 4. Server with address
    #[test]
    fn test_parse_server_with_address() {
        let src = "@startnwdiag\nnwdiag {\n  network dmz {\n    web01 [address = \"210.x.x.1\"];\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks[0].servers[0].name, "web01");
        assert_eq!(
            d.networks[0].servers[0].address.as_deref(),
            Some("210.x.x.1")
        );
    }

    // 5. Server with address and description
    #[test]
    fn test_parse_server_with_description() {
        let src = "@startnwdiag\nnwdiag {\n  network lan {\n    db01 [address = \"172.x.x.10\", description = \"database\"];\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        let s = &d.networks[0].servers[0];
        assert_eq!(s.name, "db01");
        assert_eq!(s.address.as_deref(), Some("172.x.x.10"));
        assert_eq!(s.description.as_deref(), Some("database"));
    }

    // 6. Multiple networks
    #[test]
    fn test_parse_multiple_networks() {
        let src = "@startnwdiag\nnwdiag {\n  network dmz {\n    web01;\n  }\n  network internal {\n    db01;\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks.len(), 2);
        assert_eq!(d.networks[0].name, "dmz");
        assert_eq!(d.networks[1].name, "internal");
    }

    // 7. Server appearing in multiple networks
    #[test]
    fn test_parse_server_in_multiple_networks() {
        let src = "@startnwdiag\nnwdiag {\n  network dmz {\n    web01 [address = \"210.x.x.1\"];\n  }\n  network internal {\n    web01 [address = \"172.x.x.1\"];\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks.len(), 2);
        assert_eq!(d.networks[0].servers[0].name, "web01");
        assert_eq!(d.networks[1].servers[0].name, "web01");
        assert_eq!(
            d.networks[0].servers[0].address.as_deref(),
            Some("210.x.x.1")
        );
        assert_eq!(
            d.networks[1].servers[0].address.as_deref(),
            Some("172.x.x.1")
        );
    }

    // 8. Network with color
    #[test]
    fn test_parse_network_color() {
        let src = "@startnwdiag\nnwdiag {\n  network dmz {\n    color = \"#AABBCC\"\n    web01;\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks[0].color.as_deref(), Some("#AABBCC"));
    }

    // 9. Title
    #[test]
    fn test_parse_title() {
        let src = "@startnwdiag\nnwdiag {\n  title Network Overview\n  network dmz {\n    web01;\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.title.as_deref(), Some("Network Overview"));
    }

    // 10. Comments are skipped
    #[test]
    fn test_parse_comments() {
        let src = "@startnwdiag\nnwdiag {\n  ' This is a comment\n  // Another comment\n  network dmz {\n    web01;\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks.len(), 1);
    }

    // 11. Without nwdiag wrapper
    #[test]
    fn test_parse_without_wrapper() {
        let src = "@startnwdiag\n  network dmz {\n    web01;\n  }\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks.len(), 1);
        assert_eq!(d.networks[0].name, "dmz");
    }

    // 12. Quoted network name
    #[test]
    fn test_parse_quoted_network_name() {
        let src =
            "@startnwdiag\nnwdiag {\n  network \"DMZ Network\" {\n    web01;\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks[0].name, "DMZ Network");
    }

    // 13. Full example from spec
    #[test]
    fn test_parse_full_example() {
        let src = r#"@startnwdiag
nwdiag {
  network dmz {
    address = "210.x.x.x/24"
    web01 [address = "210.x.x.1"];
    web02 [address = "210.x.x.2"];
  }
  network internal {
    address = "172.x.x.x/24"
    web01 [address = "172.x.x.1"];
    db01;
  }
}
@endnwdiag"#;
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks.len(), 2);
        assert_eq!(d.networks[0].name, "dmz");
        assert_eq!(d.networks[0].address.as_deref(), Some("210.x.x.x/24"));
        assert_eq!(d.networks[0].servers.len(), 2);
        assert_eq!(d.networks[1].name, "internal");
        assert_eq!(d.networks[1].address.as_deref(), Some("172.x.x.x/24"));
        assert_eq!(d.networks[1].servers.len(), 2);
        assert_eq!(d.networks[1].servers[1].name, "db01");
        assert!(d.networks[1].servers[1].address.is_none());
    }

    // 14. Server with no attributes
    #[test]
    fn test_parse_bare_server() {
        let src = "@startnwdiag\nnetwork net {\n  myserver;\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks[0].servers[0].name, "myserver");
        assert!(d.networks[0].servers[0].address.is_none());
        assert!(d.networks[0].servers[0].description.is_none());
    }

    // 15. Multiple servers
    #[test]
    fn test_parse_multiple_servers() {
        let src = "@startnwdiag\nnwdiag {\n  network lan {\n    srv1 [address = \"10.0.0.1\"];\n    srv2 [address = \"10.0.0.2\"];\n    srv3 [address = \"10.0.0.3\"];\n  }\n}\n@endnwdiag";
        let d = parse_nwdiag_diagram(src).unwrap();
        assert_eq!(d.networks[0].servers.len(), 3);
        assert_eq!(d.networks[0].servers[2].name, "srv3");
        assert_eq!(
            d.networks[0].servers[2].address.as_deref(),
            Some("10.0.0.3")
        );
    }

    // 16. unquote helper
    #[test]
    fn test_unquote() {
        assert_eq!(unquote("\"hello\""), "hello");
        assert_eq!(unquote("plain"), "plain");
        assert_eq!(unquote("\"\""), "");
    }

    // 17. split_attrs helper
    #[test]
    fn test_split_attrs() {
        let parts = split_attrs(r#"address = "10.0.0.1", description = "web server""#);
        assert_eq!(parts.len(), 2);
        assert!(parts[0].contains("address"));
        assert!(parts[1].contains("description"));
    }
}
