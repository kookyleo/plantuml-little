use log::{debug, trace, warn};

use crate::model::erd::{
    ErdAttribute, ErdDiagram, ErdDirection, ErdEntity, ErdIsa, ErdLink, ErdNote, ErdRelationship,
    IsaKind,
};
use crate::Result;

/// Parse ERD (Chen notation) diagram source text into an ErdDiagram IR.
pub fn parse_erd_diagram(source: &str) -> Result<ErdDiagram> {
    let block = super::common::extract_block(source).unwrap_or_else(|| source.to_string());

    // Compute the 0-indexed line offset where the block body starts in the full source.
    // extract_block skips the @start... line, so body starts at line 1 (0-indexed).
    let block_line_offset: usize = source
        .lines()
        .position(|l| {
            let t = l.trim();
            t.starts_with("@startchen") || t.starts_with("@startuml")
        })
        .map(|p| p + 1)
        .unwrap_or(0);

    let mut entities: Vec<ErdEntity> = Vec::new();
    let mut relationships: Vec<ErdRelationship> = Vec::new();
    let mut links: Vec<ErdLink> = Vec::new();
    let mut isas: Vec<ErdIsa> = Vec::new();
    let mut notes: Vec<ErdNote> = Vec::new();
    let mut direction = ErdDirection::TopToBottom;

    let mut source_order_counter = 0usize;

    let mut in_note_block = false;
    let mut note_block_position = String::new();
    let mut note_block_target: Option<String> = None;
    let mut note_block_lines: Vec<String> = Vec::new();

    let lines: Vec<&str> = block.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Skip blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        // Handle multi-line note accumulation
        if in_note_block {
            if trimmed == "end note" || trimmed == "endnote" {
                let text = note_block_lines.join("\n");
                debug!("erd parser: end note block, text={text:?}");
                notes.push(ErdNote {
                    text,
                    position: note_block_position.clone(),
                    target: note_block_target.take(),
                });
                note_block_lines.clear();
                in_note_block = false;
            } else {
                note_block_lines.push(trimmed.to_string());
            }
            i += 1;
            continue;
        }

        // Direction directive
        if trimmed == "left to right direction" {
            direction = ErdDirection::LeftToRight;
            debug!("erd parser: direction = LeftToRight");
            i += 1;
            continue;
        }
        if trimmed == "top to bottom direction" {
            direction = ErdDirection::TopToBottom;
            i += 1;
            continue;
        }

        // Entity definition
        if trimmed.starts_with("entity ") {
            let (entity, next_i) = parse_entity(&lines, i)?;
            debug!(
                "erd parser: entity '{}' (id={}, weak={}, attrs={})",
                entity.name,
                entity.id,
                entity.is_weak,
                entity.attributes.len()
            );
            let mut entity = entity;
            entity.source_order = source_order_counter;
            source_order_counter += 1;
            entities.push(entity);
            i = next_i;
            continue;
        }

        // Relationship definition
        if trimmed.starts_with("relationship ") {
            let (rel, next_i) = parse_relationship(&lines, i)?;
            debug!(
                "erd parser: relationship '{}' (id={}, identifying={}, attrs={})",
                rel.name,
                rel.id,
                rel.is_identifying,
                rel.attributes.len()
            );
            let mut rel = rel;
            rel.source_order = source_order_counter;
            source_order_counter += 1;
            relationships.push(rel);
            i = next_i;
            continue;
        }

        // ISA or link line (starts with an identifier)
        if try_parse_isa(trimmed, &mut isas, source_order_counter) {
            trace!("erd parser: parsed ISA line: {trimmed}");
            source_order_counter += 1;
            i += 1;
            continue;
        }

        if try_parse_link(trimmed, &mut links, source_order_counter, i + block_line_offset) {
            trace!("erd parser: parsed link line: {trimmed}");
            source_order_counter += 1;
            i += 1;
            continue;
        }

        // Note parsing
        if let Some(note_result) = try_parse_erd_note(trimmed) {
            match note_result {
                ErdNoteParseResult::SingleLine(note) => {
                    debug!("erd parser: single-line note for {:?}", note.target);
                    notes.push(note);
                }
                ErdNoteParseResult::MultiLineStart { position, target } => {
                    debug!("erd parser: start multi-line note for {target:?}");
                    in_note_block = true;
                    note_block_position = position;
                    note_block_target = target;
                    note_block_lines.clear();
                }
            }
            i += 1;
            continue;
        }

        // Unknown line
        trace!("erd parser: skipping unrecognized line: {trimmed}");
        i += 1;
    }

    debug!(
        "erd parser: done - {} entities, {} relationships, {} links, {} ISAs",
        entities.len(),
        relationships.len(),
        links.len(),
        isas.len()
    );

    Ok(ErdDiagram {
        entities,
        relationships,
        links,
        isas,
        direction,
        notes,
    })
}

// ---------------------------------------------------------------------------
// Entity parsing
// ---------------------------------------------------------------------------

/// Parse an entity definition starting at line `start`.
/// Returns the entity and the index of the next line after the entity block.
fn parse_entity(lines: &[&str], start: usize) -> Result<(ErdEntity, usize)> {
    let trimmed = lines[start].trim();
    // Remove "entity " prefix
    let rest = trimmed.strip_prefix("entity ").unwrap();

    // Parse: "DisplayName" as ALIAS <<weak>> #color { ... }
    // or:    NAME <<weak>> #color { ... }
    let (display_name, alias, rest) = parse_name_alias(rest);
    let (is_weak, rest) = parse_stereotype_weak(&rest);
    let (color, rest) = parse_color(&rest);

    let rest_trimmed = rest.trim();

    // Check if block starts on same line
    let has_brace = rest_trimmed.starts_with('{');
    let (attributes, next_i) = if has_brace {
        // Check if closing brace is on same line: `entity Foo {}`
        let after_brace = rest_trimmed.strip_prefix('{').unwrap().trim();
        if after_brace.starts_with('}') {
            (Vec::new(), start + 1)
        } else {
            parse_attributes(lines, start + 1)?
        }
    } else {
        (Vec::new(), start + 1)
    };

    let id = alias.unwrap_or_else(|| display_name.clone());

    Ok((
        ErdEntity {
            id,
            name: display_name,
            attributes,
            is_weak,
            color,
            source_order: 0, // set by caller
        },
        next_i,
    ))
}

// ---------------------------------------------------------------------------
// Relationship parsing
// ---------------------------------------------------------------------------

fn parse_relationship(lines: &[&str], start: usize) -> Result<(ErdRelationship, usize)> {
    let trimmed = lines[start].trim();
    let rest = trimmed.strip_prefix("relationship ").unwrap();

    let (display_name, alias, rest) = parse_name_alias(rest);
    let (is_identifying, rest) = parse_stereotype_identifying(&rest);
    let (color, rest) = parse_color(&rest);

    let rest_trimmed = rest.trim();

    let has_brace = rest_trimmed.starts_with('{');
    let (attributes, next_i) = if has_brace {
        let after_brace = rest_trimmed.strip_prefix('{').unwrap().trim();
        if after_brace.starts_with('}') {
            (Vec::new(), start + 1)
        } else {
            parse_attributes(lines, start + 1)?
        }
    } else {
        (Vec::new(), start + 1)
    };

    let id = alias.unwrap_or_else(|| display_name.clone());

    Ok((
        ErdRelationship {
            id,
            name: display_name,
            attributes,
            is_identifying,
            color,
            source_order: 0, // set by caller
        },
        next_i,
    ))
}

// ---------------------------------------------------------------------------
// Attribute parsing
// ---------------------------------------------------------------------------

/// Parse attribute lines inside braces until `}`.
/// Returns (attributes, next_line_index_after_closing_brace).
fn parse_attributes(lines: &[&str], start: usize) -> Result<(Vec<ErdAttribute>, usize)> {
    let mut attrs = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        if trimmed == "}" || trimmed.starts_with('}') {
            return Ok((attrs, i + 1));
        }

        // Check for nested attribute: `Name { Fname Lname }`
        // or `Name {` on this line, children on next lines
        if let Some(attr) = try_parse_nested_attr(trimmed, lines, &mut i)? {
            attrs.push(attr);
            continue;
        }

        // Simple attribute: `AttrName <<modifier>> #color`
        // or `AttrName : TYPE <<modifier>>`
        if let Some(attr) = parse_simple_attr(trimmed) {
            attrs.push(attr);
        }

        i += 1;
    }

    warn!("erd parser: reached end of input without closing brace");
    Ok((attrs, i))
}

/// Try to parse a nested attribute (one that has `{ ... }` children).
fn try_parse_nested_attr(
    trimmed: &str,
    lines: &[&str],
    i: &mut usize,
) -> Result<Option<ErdAttribute>> {
    // Pattern: `Name { Child1 Child2 }` (all on one line)
    // or: `Name {` (children on subsequent lines)
    let brace_pos = match trimmed.find('{') {
        Some(pos) => pos,
        None => return Ok(None),
    };

    let name_part = trimmed[..brace_pos].trim();
    let attr_name = name_part.to_string();

    let after_brace = trimmed[brace_pos + 1..].trim();

    if let Some(end_pos) = after_brace.find('}') {
        // Everything on one line: `Name { Fname Lname }`
        let children_str = after_brace[..end_pos].trim();
        let children: Vec<ErdAttribute> = children_str
            .split_whitespace()
            .map(|name| ErdAttribute {
                name: name.to_string(),
                display_name: None,
                is_key: false,
                is_derived: false,
                is_multi: false,
                attr_type: None,
                children: Vec::new(),
                color: None,
            })
            .collect();

        *i += 1;
        return Ok(Some(ErdAttribute {
            name: attr_name,
            display_name: None,
            is_key: false,
            is_derived: false,
            is_multi: false,
            attr_type: None,
            children,
            color: None,
        }));
    }

    // Multi-line: `Name {`  then children on following lines
    *i += 1;
    let mut children = Vec::new();
    while *i < lines.len() {
        let child_trimmed = lines[*i].trim();
        if child_trimmed == "}" || child_trimmed.starts_with('}') {
            *i += 1;
            break;
        }
        if !child_trimmed.is_empty() {
            if let Some(attr) = parse_simple_attr(child_trimmed) {
                children.push(attr);
            }
        }
        *i += 1;
    }

    Ok(Some(ErdAttribute {
        name: attr_name,
        display_name: None,
        is_key: false,
        is_derived: false,
        is_multi: false,
        attr_type: None,
        children,
        color: None,
    }))
}

/// Parse a simple attribute line (no nested braces).
fn parse_simple_attr(line: &str) -> Option<ErdAttribute> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check for "alias" as display form: `"No." as Number <<key>>`
    let (display_name, name, rest) = if let Some(after_quote) = trimmed.strip_prefix('"') {
        if let Some(end_quote) = after_quote.find('"') {
            let dn = after_quote[..end_quote].to_string();
            let after = after_quote[end_quote + 1..].trim();
            if let Some(after_as) = after.strip_prefix("as ") {
                let after_as = after_as.trim();
                // Name is the next word
                let (n, rest) = split_first_word(after_as);
                (Some(dn), n, rest)
            } else {
                (Some(dn.clone()), dn, after.to_string())
            }
        } else {
            let (n, rest) = split_first_word(trimmed);
            (None, n, rest)
        }
    } else {
        let (n, rest) = split_first_word(trimmed);
        (None, n, rest)
    };

    // Parse type annotation `: TYPE`
    let (attr_type, rest) = parse_attr_type(&rest);

    // Parse modifiers: <<key>>, <<derived>>, <<multi>>
    let (is_key, is_derived, is_multi, rest) = parse_modifiers(&rest);

    // Parse color
    let (color, _) = parse_color(&rest);

    Some(ErdAttribute {
        name,
        display_name,
        is_key,
        is_derived,
        is_multi,
        attr_type,
        children: Vec::new(),
        color,
    })
}

/// Split first whitespace-delimited word from rest.
fn split_first_word(s: &str) -> (String, String) {
    let s = s.trim();
    if let Some(pos) = s.find(|c: char| c.is_whitespace()) {
        (s[..pos].to_string(), s[pos..].to_string())
    } else {
        (s.to_string(), String::new())
    }
}

/// Parse `: TYPE` annotation
fn parse_attr_type(s: &str) -> (Option<String>, String) {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix(':') {
        let rest = rest.trim();
        let (type_name, remaining) = split_first_word(rest);
        if type_name.is_empty() {
            (None, remaining)
        } else {
            (Some(type_name), remaining)
        }
    } else {
        (None, trimmed.to_string())
    }
}

/// Parse modifiers: <<key>>, <<derived>>, <<multi>>
fn parse_modifiers(s: &str) -> (bool, bool, bool, String) {
    let mut is_key = false;
    let mut is_derived = false;
    let mut is_multi = false;
    let mut remaining = s.to_string();

    loop {
        let trimmed = remaining.trim();
        if let Some(rest) = trimmed.strip_prefix("<<") {
            if let Some(end) = rest.find(">>") {
                let modifier = &rest[..end];
                match modifier {
                    "key" => is_key = true,
                    "derived" => is_derived = true,
                    "multi" => is_multi = true,
                    _ => {} // skip unknown modifiers
                }
                remaining = rest[end + 2..].to_string();
                continue;
            }
        }
        break;
    }

    (is_key, is_derived, is_multi, remaining)
}

// ---------------------------------------------------------------------------
// Name/alias parsing
// ---------------------------------------------------------------------------

/// Parse `"Display Name" as ALIAS rest...` or `NAME rest...`
fn parse_name_alias(s: &str) -> (String, Option<String>, String) {
    let trimmed = s.trim();

    // Quoted name with alias: `"Name" as ALIAS ...`
    if let Some(after_quote) = trimmed.strip_prefix('"') {
        if let Some(end_quote) = after_quote.find('"') {
            let display = after_quote[..end_quote].to_string();
            let after = after_quote[end_quote + 1..].trim();
            if let Some(after_as) = after.strip_prefix("as ") {
                let after_as = after_as.trim();
                let (alias, rest) = split_first_word(after_as);
                return (display, Some(alias), rest);
            }
            return (display, None, after.to_string());
        }
    }

    // Unquoted: `NAME rest...`
    let (name, rest) = split_first_word(trimmed);
    (name, None, rest)
}

/// Check for `<<weak>>` stereotype
fn parse_stereotype_weak(s: &str) -> (bool, String) {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix("<<weak>>") {
        (true, rest.to_string())
    } else {
        (false, trimmed.to_string())
    }
}

/// Check for `<<identifying>>` stereotype
fn parse_stereotype_identifying(s: &str) -> (bool, String) {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix("<<identifying>>") {
        (true, rest.to_string())
    } else {
        (false, trimmed.to_string())
    }
}

/// Parse optional color: `#lightblue;line:blue`
fn parse_color(s: &str) -> (Option<String>, String) {
    let trimmed = s.trim();
    if trimmed.starts_with('#') {
        // Color goes until whitespace or `{`
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '{')
            .unwrap_or(trimmed.len());
        let color = trimmed[..end].to_string();
        let rest = trimmed[end..].to_string();
        (Some(color), rest)
    } else {
        (None, trimmed.to_string())
    }
}

// ---------------------------------------------------------------------------
// Link / ISA parsing
// ---------------------------------------------------------------------------

/// Try to parse an ISA line: `PARENT =>= d { CHILD1, CHILD2 }`
/// or `PARENT ->- U { CHILD1, CHILD2 } #color`
fn try_parse_isa(line: &str, isas: &mut Vec<ErdIsa>, source_order: usize) -> bool {
    let trimmed = line.trim();

    // Pattern: `PARENT =>= d { C1, C2, ... }` or `PARENT ->- U { C1, C2, ... }`
    // The connector is `=>=`, `->-`
    let (parent, rest, is_double) = if let Some((p, r)) = try_split_isa_connector(trimmed, "=>=") {
        (p, r, true)
    } else if let Some((p, r)) = try_split_isa_connector(trimmed, "->-") {
        (p, r, false)
    } else {
        return false;
    };

    let rest = rest.trim();

    // Next token: 'd' or 'U'
    let (kind_str, rest) = split_first_word(rest);
    let kind = match kind_str.as_str() {
        "d" => IsaKind::Disjoint,
        "U" => IsaKind::Union,
        _ => return false,
    };

    // Expect `{ C1, C2, ... }` possibly followed by #color
    let rest = rest.trim();
    if !rest.starts_with('{') {
        return false;
    }
    let rest = rest.strip_prefix('{').unwrap().trim();

    let brace_end = match rest.find('}') {
        Some(pos) => pos,
        None => return false,
    };

    let children_str = &rest[..brace_end];
    let children: Vec<String> = children_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if children.is_empty() {
        return false;
    }

    let after_brace = rest[brace_end + 1..].trim();
    let (color, _) = parse_color(after_brace);

    isas.push(ErdIsa {
        parent: parent.to_string(),
        kind,
        children,
        is_double,
        color,
        source_order,
    });

    true
}

fn try_split_isa_connector<'a>(s: &'a str, connector: &str) -> Option<(&'a str, &'a str)> {
    if let Some(pos) = s.find(connector) {
        let before = s[..pos].trim();
        let after = s[pos + connector.len()..].trim();
        if !before.is_empty() && !after.is_empty() {
            return Some((before, after));
        }
    }
    None
}

/// Try to parse a link line.
///
/// Patterns:
///   `FROM -N- TO #color`
///   `FROM -1- TO`
///   `FROM -(N,M)- TO`
///   `FROM =N= TO`   (double-line)
///   `FROM ->- TO`    (ISA arrow - already handled above, but ->- without d/U is a link)
///   `FROM -<- TO`    (reverse ISA arrow)
fn try_parse_link(line: &str, links: &mut Vec<ErdLink>, source_order: usize, source_line: usize) -> bool {
    let trimmed = line.trim();

    // Match link connectors: single `-X-` or double `=X=`
    // Also handle `-(N,M)-` (parenthesized cardinality)

    // Try to find a link pattern. We look for:
    //   - `=...=` (double line)
    //   - `-...-` (single line)
    // The content between the delimiters is the cardinality.

    // Strategy: find all `-...-` or `=...=` patterns.

    // Find double-line links first: `=X=`
    if let Some(mut link) = try_parse_link_pattern(trimmed, true) {
        link.source_order = source_order;
        link.source_line = source_line;
        links.push(link);
        return true;
    }

    // Single-line links: `-X-`
    if let Some(mut link) = try_parse_link_pattern(trimmed, false) {
        link.source_order = source_order;
        link.source_line = source_line;
        links.push(link);
        return true;
    }

    false
}

/// Try to parse a link with a specific pattern (single or double line).
fn try_parse_link_pattern(line: &str, is_double: bool) -> Option<ErdLink> {
    let delim = if is_double { '=' } else { '-' };

    // Find the first delimiter that's part of a connector
    let bytes = line.as_bytes();
    let len = bytes.len();

    // Scan for first delimiter
    let mut start_pos = None;
    for (i, &b) in bytes.iter().enumerate() {
        if b == delim as u8 {
            // Check this is preceded by whitespace or start-of-line
            if i == 0 || bytes[i - 1] == b' ' {
                start_pos = Some(i);
                break;
            }
        }
    }

    let start = start_pos?;

    // Now find the end of the connector: scan for the closing delimiter
    // Connector content can be: N, 1, (N,M), >-, -<-, etc.
    let inner_start = start + 1;
    let mut end_pos = None;

    for i in inner_start..len {
        if bytes[i] == delim as u8 {
            // Check this is followed by whitespace or end-of-line
            if i + 1 >= len || bytes[i + 1] == b' ' || bytes[i + 1] == b'#' {
                end_pos = Some(i);
                break;
            }
        }
    }

    let end = end_pos?;

    let from = line[..start].trim();
    let cardinality = line[inner_start..end].trim();
    let after = line[end + 1..].trim();

    if from.is_empty() || after.is_empty() {
        return None;
    }

    // `after` is `TO #color` or just `TO`
    let (to, rest) = split_first_word(after);

    if to.is_empty() {
        return None;
    }

    let (color, _) = parse_color(&rest);

    // Detect ISA simple subclass arrows: `->-` gives cardinality ">" and `-<-` gives "<"
    let isa_arrow = match cardinality {
        ">" => Some(true),  // superset (->-)
        "<" => Some(false), // subset (-<-)
        _ => None,
    };

    Some(ErdLink {
        from: from.to_string(),
        to: to.to_string(),
        cardinality: cardinality.to_string(),
        is_double,
        color,
        isa_arrow,
        source_order: 0, // set by caller
        source_line: 0,  // set by caller
    })
}

// ---------------------------------------------------------------------------
// Note parsing
// ---------------------------------------------------------------------------

enum ErdNoteParseResult {
    SingleLine(ErdNote),
    MultiLineStart {
        position: String,
        target: Option<String>,
    },
}

fn try_parse_erd_note(line: &str) -> Option<ErdNoteParseResult> {
    let trimmed = line.trim();
    if !trimmed.starts_with("note ") {
        return None;
    }

    let rest = trimmed[5..].trim();

    for pos in &["left", "right", "top", "bottom"] {
        if !rest.starts_with(pos) {
            continue;
        }
        let after_pos = rest[pos.len()..].trim();

        if let Some(after_of_raw) = after_pos.strip_prefix("of ") {
            let after_of = after_of_raw.trim();

            if let Some(colon_pos) = after_of.find(':') {
                let target = after_of[..colon_pos].trim().to_string();
                let text = after_of[colon_pos + 1..]
                    .trim()
                    .replace("\\n", "\n")
                    .replace(crate::NEWLINE_CHAR, "\n");
                return Some(ErdNoteParseResult::SingleLine(ErdNote {
                    text,
                    position: pos.to_string(),
                    target: Some(target),
                }));
            }

            let target = after_of.trim().to_string();
            return Some(ErdNoteParseResult::MultiLineStart {
                position: pos.to_string(),
                target: if target.is_empty() {
                    None
                } else {
                    Some(target)
                },
            });
        }

        if let Some(after_colon) = after_pos.strip_prefix(':') {
            let text = after_colon
                .trim()
                .replace("\\n", "\n")
                .replace(crate::NEWLINE_CHAR, "\n");
            return Some(ErdNoteParseResult::SingleLine(ErdNote {
                text,
                position: pos.to_string(),
                target: None,
            }));
        }

        if after_pos.is_empty() {
            return Some(ErdNoteParseResult::MultiLineStart {
                position: pos.to_string(),
                target: None,
            });
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Basic entity parsing
    #[test]
    fn test_parse_simple_entity() {
        let src = "@startchen\nentity MOVIE {\n  Code\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.entities.len(), 1);
        assert_eq!(d.entities[0].id, "MOVIE");
        assert_eq!(d.entities[0].name, "MOVIE");
        assert_eq!(d.entities[0].attributes.len(), 1);
        assert_eq!(d.entities[0].attributes[0].name, "Code");
        assert!(!d.entities[0].is_weak);
    }

    // 2. Entity with attributes and modifiers
    #[test]
    fn test_parse_entity_with_modifiers() {
        let src = "@startchen\nentity CUSTOMER {\n  Number <<key>>\n  Bonus <<derived>>\n  Name <<multi>>\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.entities.len(), 1);
        let e = &d.entities[0];
        assert_eq!(e.attributes.len(), 3);
        assert!(e.attributes[0].is_key);
        assert!(e.attributes[1].is_derived);
        assert!(e.attributes[2].is_multi);
    }

    // 3. Weak entity
    #[test]
    fn test_parse_weak_entity() {
        let src = "@startchen\nentity CHILD <<weak>> {\n  Name <<key>>\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.entities.len(), 1);
        assert!(d.entities[0].is_weak);
    }

    // 4. Relationship parsing
    #[test]
    fn test_parse_relationship() {
        let src = "@startchen\nrelationship RENTED_TO {\n  Date\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.relationships.len(), 1);
        assert_eq!(d.relationships[0].id, "RENTED_TO");
        assert_eq!(d.relationships[0].attributes.len(), 1);
        assert!(!d.relationships[0].is_identifying);
    }

    // 5. Identifying relationship
    #[test]
    fn test_parse_identifying_relationship() {
        let src = "@startchen\nrelationship PARENT_OF <<identifying>> {\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.relationships.len(), 1);
        assert!(d.relationships[0].is_identifying);
    }

    // 6. Link parsing
    #[test]
    fn test_parse_links() {
        let src = "@startchen\nRENTED_TO -1- CUSTOMER\nRENTED_TO -N- MOVIE\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.links.len(), 2);
        assert_eq!(d.links[0].from, "RENTED_TO");
        assert_eq!(d.links[0].to, "CUSTOMER");
        assert_eq!(d.links[0].cardinality, "1");
        assert!(!d.links[0].is_double);
        assert_eq!(d.links[1].cardinality, "N");
    }

    // 7. Double-line link
    #[test]
    fn test_parse_double_link() {
        let src = "@startchen\nPARENT_OF =N= CHILD\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.links.len(), 1);
        assert!(d.links[0].is_double);
        assert_eq!(d.links[0].cardinality, "N");
    }

    // 8. Parenthesized cardinality
    #[test]
    fn test_parse_parenthesized_cardinality() {
        let src = "@startchen\nRENTED_TO -(N,M)- DIRECTOR\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.links.len(), 1);
        assert_eq!(d.links[0].cardinality, "(N,M)");
    }

    // 9. ISA parsing (disjoint)
    #[test]
    fn test_parse_isa_disjoint() {
        let src = "@startchen\nCHILD =>= d { TODDLER, PRIMARY_AGE, TEEN }\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.isas.len(), 1);
        assert_eq!(d.isas[0].parent, "CHILD");
        assert_eq!(d.isas[0].kind, IsaKind::Disjoint);
        assert_eq!(d.isas[0].children.len(), 3);
        assert!(d.isas[0].is_double);
    }

    // 10. ISA parsing (union)
    #[test]
    fn test_parse_isa_union() {
        let src = "@startchen\nPERSON ->- U { CUSTOMER, DIRECTOR }\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.isas.len(), 1);
        assert_eq!(d.isas[0].parent, "PERSON");
        assert_eq!(d.isas[0].kind, IsaKind::Union);
        assert_eq!(d.isas[0].children.len(), 2);
        assert!(!d.isas[0].is_double);
    }

    // 11. Alias parsing (entity)
    #[test]
    fn test_parse_entity_alias() {
        let src = "@startchen\nentity \"Director\" as DIRECTOR {\n  Age\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.entities.len(), 1);
        assert_eq!(d.entities[0].id, "DIRECTOR");
        assert_eq!(d.entities[0].name, "Director");
    }

    // 12. Alias parsing (relationship)
    #[test]
    fn test_parse_relationship_alias() {
        let src = "@startchen\nrelationship \"was-rented-to\" as RENTED_TO {\n  Date\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.relationships.len(), 1);
        assert_eq!(d.relationships[0].id, "RENTED_TO");
        assert_eq!(d.relationships[0].name, "was-rented-to");
    }

    // 13. Direction
    #[test]
    fn test_parse_direction() {
        let src = "@startchen\nleft to right direction\nentity Person {\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.direction, ErdDirection::LeftToRight);
    }

    // 14. Nested attributes
    #[test]
    fn test_parse_nested_attributes() {
        let src = "@startchen\nentity DIRECTOR {\n  Name {\n    Fname\n    Lname\n  }\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.entities[0].attributes.len(), 1);
        assert_eq!(d.entities[0].attributes[0].name, "Name");
        assert_eq!(d.entities[0].attributes[0].children.len(), 2);
        assert_eq!(d.entities[0].attributes[0].children[0].name, "Fname");
        assert_eq!(d.entities[0].attributes[0].children[1].name, "Lname");
    }

    // 15. Attribute type
    #[test]
    fn test_parse_attribute_type() {
        let src = "@startchen\nentity X {\n  Born : DATE\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(
            d.entities[0].attributes[0].attr_type.as_deref(),
            Some("DATE")
        );
    }

    // 16. Color parsing
    #[test]
    fn test_parse_color() {
        let src =
            "@startchen\nentity CUSTOMER #lightblue;line:blue {\n  Number <<key>> #lime;line:orange\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.entities[0].color.as_deref(), Some("#lightblue;line:blue"));
        assert_eq!(
            d.entities[0].attributes[0].color.as_deref(),
            Some("#lime;line:orange")
        );
    }

    // 17. Link with color
    #[test]
    fn test_parse_link_color() {
        let src = "@startchen\nRENTED_TO -1- CUSTOMER #lime\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.links[0].color.as_deref(), Some("#lime"));
    }

    // 18. ISA with color
    #[test]
    fn test_parse_isa_color() {
        let src = "@startchen\nPERSON ->- U { CUSTOMER, DIRECTOR } #lime;line:green\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.isas[0].color.as_deref(), Some("#lime;line:green"));
    }

    // 19. Empty entity
    #[test]
    fn test_parse_empty_entity() {
        let src = "@startchen\nentity Person {}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.entities.len(), 1);
        assert_eq!(d.entities[0].attributes.len(), 0);
    }

    // 20. Full chenmovie.puml fixture
    #[test]
    fn test_parse_chenmovie_fixture() {
        let src = std::fs::read_to_string("tests/fixtures/erd/chenmovie.puml").unwrap();
        let d = parse_erd_diagram(&src).unwrap();
        assert_eq!(d.entities.len(), 3); // DIRECTOR, CUSTOMER, MOVIE
        assert_eq!(d.relationships.len(), 1); // RENTED_TO
        assert_eq!(d.links.len(), 3);
        assert_eq!(d.direction, ErdDirection::TopToBottom);
    }

    // 21. Full chenrankdir.puml fixture
    #[test]
    fn test_parse_chenrankdir_fixture() {
        let src = std::fs::read_to_string("tests/fixtures/erd/chenrankdir.puml").unwrap();
        let d = parse_erd_diagram(&src).unwrap();
        assert_eq!(d.entities.len(), 2); // Person, Location
        assert_eq!(d.relationships.len(), 1); // Birthplace
        assert_eq!(d.links.len(), 2);
        assert_eq!(d.direction, ErdDirection::LeftToRight);
    }

    // 22. Arrow link patterns
    #[test]
    fn test_parse_arrow_links() {
        let src = "@startchen\nCUSTOMER ->- PARENT\nMEMBER -<- CUSTOMER\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        // These are ISA-like but without d/U, so they should be parsed as links
        // Connector `-X-` where X is the inner content: `>` and `<`
        assert_eq!(d.links.len(), 2);
        assert_eq!(d.links[0].cardinality, ">");
        assert_eq!(d.links[1].cardinality, "<");
    }

    // 23. Attribute alias
    #[test]
    fn test_parse_attribute_alias() {
        let src = "@startchen\nentity X {\n  \"No.\" as Number <<key>>\n}\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        let attr = &d.entities[0].attributes[0];
        assert_eq!(attr.name, "Number");
        assert_eq!(attr.display_name.as_deref(), Some("No."));
        assert!(attr.is_key);
    }

    // 24. chenmoviealias.puml fixture
    #[test]
    fn test_parse_chenmoviealias_fixture() {
        let src = std::fs::read_to_string("tests/fixtures/erd/chenmoviealias.puml").unwrap();
        let d = parse_erd_diagram(&src).unwrap();
        // Should have Director, Customer, Movie, Parent, Member, Child, Toddler, Primary-Aged Kid, Teenager, Human
        assert!(d.entities.len() >= 8);
        assert!(d.relationships.len() >= 2);
        assert!(!d.isas.is_empty());
    }

    // 25. Note parsing
    #[test]
    fn test_parse_note() {
        let src =
            "@startchen\nentity MOVIE {\n  Code\n}\nnote right of MOVIE : primary entity\n@endchen";
        let d = parse_erd_diagram(src).unwrap();
        assert_eq!(d.notes.len(), 1);
        assert_eq!(d.notes[0].position, "right");
        assert_eq!(d.notes[0].target.as_deref(), Some("MOVIE"));
        assert_eq!(d.notes[0].text, "primary entity");
    }
}
