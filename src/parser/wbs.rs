use log::{debug, trace, warn};

use crate::model::wbs::{WbsDiagram, WbsDirection, WbsLink, WbsNode, WbsNote};
use crate::Result;

/// Extract content between @startwbs and @endwbs.
fn extract_wbs_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if inside {
            if trimmed.starts_with("@endwbs") {
                break;
            }
            lines.push(line);
        } else {
            if trimmed.starts_with("@startwbs") {
                inside = true;
                continue;
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Count the number of leading `*` characters in a trimmed line.
/// Returns (level, rest_after_stars) where rest_after_stars includes any
/// direction suffix and the remaining text.
fn count_stars(line: &str) -> (usize, &str) {
    // WBS supports *, +, - as node level markers
    let count = line
        .chars()
        .take_while(|&c| c == '*' || c == '+' || c == '-')
        .count();
    (count, &line[count..])
}

/// Parse direction suffix from the text right after the stars.
/// Returns (direction, remaining_text).
fn parse_direction(after_stars: &str) -> (WbsDirection, &str) {
    if let Some(rest) = after_stars.strip_prefix('>') {
        (WbsDirection::Right, rest)
    } else if let Some(rest) = after_stars.strip_prefix('<') {
        (WbsDirection::Left, rest)
    } else {
        (WbsDirection::Default, after_stars)
    }
}

/// Parse alias and text from the remainder after stars+direction.
/// Supports:
///   `(ALIAS) text` - parenthesized alias before text
///   `"text" as ALIAS` - quoted text with alias after
///   `text` - plain text, no alias
/// Replace `\n` with newline, but only OUTSIDE `[[...]]` link markers.
/// Inside link markers, `\n` stays as literal characters (the renderer
/// handles newline interpretation in tooltips).
fn replace_newlines_outside_links(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;
    while !rest.is_empty() {
        if let Some(start) = rest.find("[[") {
            // Replace \n in the part before [[
            let before = &rest[..start];
            result.push_str(
                &before
                    .replace("\\n", "\n")
                    .replace(crate::NEWLINE_CHAR, "\n"),
            );
            // Find matching ]]
            if let Some(end) = rest[start + 2..].find("]]") {
                let link_end = start + 2 + end + 2;
                result.push_str(&rest[start..link_end]);
                rest = &rest[link_end..];
            } else {
                // No matching ]] — treat rest as plain text
                result.push_str(
                    &rest[start..]
                        .replace("\\n", "\n")
                        .replace(crate::NEWLINE_CHAR, "\n"),
                );
                break;
            }
        } else {
            result.push_str(&rest.replace("\\n", "\n").replace(crate::NEWLINE_CHAR, "\n"));
            break;
        }
    }
    result
}

fn parse_alias_and_text(input: &str) -> (Option<String>, String) {
    let trimmed = input.trim();

    // Check for `(ALIAS) text` pattern
    if trimmed.starts_with('(') {
        if let Some(close_paren) = trimmed.find(')') {
            let alias = trimmed[1..close_paren].trim().to_string();
            let text = trimmed[close_paren + 1..].trim().to_string();
            let text = replace_newlines_outside_links(&text);
            return (Some(alias), text);
        }
    }

    // Check for `"text" as ALIAS` pattern
    if let Some(after_open) = trimmed.strip_prefix('"') {
        if let Some(end_quote) = after_open.find('"') {
            let quoted_text = after_open[..end_quote].to_string();
            let after_quote = after_open[end_quote + 1..].trim();
            let after_quote_clean = after_quote.trim();

            if let Some(rest) = after_quote_clean.strip_prefix("as ") {
                let alias = rest.trim().to_string();
                let text = replace_newlines_outside_links(&quoted_text);
                return (Some(alias), text);
            } else {
                // Quoted text without alias
                let text = replace_newlines_outside_links(&quoted_text);
                return (None, text);
            }
        }
    }

    // Plain text, preserving inline link markup for the renderer.
    let text = replace_newlines_outside_links(trimmed);
    (None, text)
}

/// Parse a link line like `ALIAS1 -> ALIAS2`.
fn parse_link_line(line: &str) -> Option<WbsLink> {
    let parts: Vec<&str> = line.split("->").collect();
    if parts.len() == 2 {
        let from = parts[0].trim().to_string();
        let to = parts[1].trim().to_string();
        if !from.is_empty() && !to.is_empty() {
            return Some(WbsLink { from, to });
        }
    }
    None
}

/// Parse a WBS diagram from source text.
pub fn parse_wbs_diagram(source: &str) -> Result<WbsDiagram> {
    let block = extract_wbs_block(source).unwrap_or_else(|| source.to_string());

    debug!("parse_wbs_diagram: extracted block ({} bytes)", block.len());

    // Flat list of (level, direction, alias, text) parsed from `*`-prefixed lines
    let mut flat_nodes: Vec<(usize, WbsDirection, Option<String>, String)> = Vec::new();
    let mut links: Vec<WbsLink> = Vec::new();
    let mut notes: Vec<WbsNote> = Vec::new();
    let mut in_note_block = false;
    let mut note_block_position = String::new();
    let mut note_block_lines: Vec<String> = Vec::new();

    // Pre-process line continuations (trailing `\`)
    let mut merged_lines: Vec<String> = Vec::new();
    let mut pending = String::new();
    for line in block.lines() {
        let trimmed = line.trim_end();
        if let Some(stripped) = trimmed.strip_suffix('\\') {
            pending.push_str(stripped);
        } else if !pending.is_empty() {
            pending.push_str(trimmed);
            merged_lines.push(std::mem::take(&mut pending));
        } else {
            merged_lines.push(trimmed.to_string());
        }
    }
    if !pending.is_empty() {
        merged_lines.push(pending);
    }

    for (line_num, line) in merged_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("'") {
            trace!("line {}: skip empty/comment", line_num + 1);
            continue;
        }

        // Handle multi-line note accumulation
        if in_note_block {
            if trimmed == "end note" || trimmed == "endnote" {
                let text = note_block_lines.join("\n");
                debug!("line {}: end note block, text={:?}", line_num + 1, text);
                notes.push(WbsNote {
                    text,
                    position: note_block_position.clone(),
                });
                note_block_lines.clear();
                in_note_block = false;
            } else {
                note_block_lines.push(trimmed.to_string());
            }
            continue;
        }

        // Note parsing: `note left : text` or `note right`
        if let Some(note_result) = try_parse_wbs_note(trimmed) {
            match note_result {
                WbsNoteParseResult::SingleLine(note) => {
                    debug!("line {}: single-line note", line_num + 1);
                    notes.push(note);
                }
                WbsNoteParseResult::MultiLineStart { position } => {
                    debug!("line {}: start multi-line note", line_num + 1);
                    in_note_block = true;
                    note_block_position = position;
                    note_block_lines.clear();
                }
            }
            continue;
        }

        // Check for link lines: `ALIAS -> ALIAS`
        if trimmed.contains("->")
            && !trimmed.starts_with('*')
            && !trimmed.starts_with('+')
            && !trimmed.starts_with('-')
        {
            if let Some(link) = parse_link_line(trimmed) {
                debug!(
                    "line {}: link '{}' -> '{}'",
                    line_num + 1,
                    link.from,
                    link.to
                );
                links.push(link);
                continue;
            }
        }

        // Skip lines that don't start with `*`
        if !trimmed.starts_with('*') && !trimmed.starts_with('+') && !trimmed.starts_with('-') {
            trace!("line {}: skip non-star line: '{}'", line_num + 1, trimmed);
            continue;
        }

        let (level, after_stars) = count_stars(trimmed);
        let (direction, rest) = parse_direction(after_stars);
        let (alias, text) = parse_alias_and_text(rest);

        debug!(
            "line {}: level={}, dir={:?}, alias={:?}, text='{}'",
            line_num + 1,
            level,
            direction,
            alias,
            text
        );

        flat_nodes.push((level, direction, alias, text));
    }

    if flat_nodes.is_empty() {
        return Err(crate::Error::Parse {
            line: 1,
            column: Some(1),
            message: "WBS diagram has no nodes".to_string(),
        });
    }

    // Build tree from flat list using a stack-based approach
    let root = build_tree(&flat_nodes)?;

    debug!(
        "parse_wbs_diagram done: root='{}', {} links",
        root.text,
        links.len()
    );

    Ok(WbsDiagram { root, links, notes })
}

/// Build a tree from a flat list of (level, direction, alias, text).
/// The first entry must be level 1 (root).
fn build_tree(flat: &[(usize, WbsDirection, Option<String>, String)]) -> Result<WbsNode> {
    if flat.is_empty() {
        return Err(crate::Error::Parse {
            line: 1,
            column: Some(1),
            message: "empty node list".to_string(),
        });
    }

    let (root_level, ref root_dir, ref root_alias, ref root_text) = flat[0];
    if root_level != 1 {
        warn!("WBS root node should be level 1, got level {root_level}");
    }

    let mut root = WbsNode {
        text: root_text.clone(),
        children: Vec::new(),
        direction: root_dir.clone(),
        alias: root_alias.clone(),
        level: root_level,
    };

    // Build children recursively using index-based slicing
    build_children(&mut root, &flat[1..]);

    Ok(root)
}

/// Recursively attach children to a parent node.
/// `remaining` is the slice of flat nodes after the parent.
/// This function consumes nodes that belong as descendants of `parent`.
fn build_children(
    parent: &mut WbsNode,
    remaining: &[(usize, WbsDirection, Option<String>, String)],
) {
    let parent_level = parent.level;
    let mut i = 0;

    while i < remaining.len() {
        let (level, ref dir, ref alias, ref text) = remaining[i];

        if level <= parent_level {
            // This node belongs to an ancestor, stop
            break;
        }

        if level == parent_level + 1 {
            // Direct child
            let mut child = WbsNode {
                text: text.clone(),
                children: Vec::new(),
                direction: dir.clone(),
                alias: alias.clone(),
                level,
            };

            // Find the range of descendants for this child
            let child_start = i + 1;
            let mut child_end = child_start;
            while child_end < remaining.len() {
                let (next_level, _, _, _) = remaining[child_end];
                if next_level <= level {
                    break;
                }
                child_end += 1;
            }

            // Recursively build the child's descendants
            if child_start < child_end {
                build_children(&mut child, &remaining[child_start..child_end]);
            }

            parent.children.push(child);
            i = child_end;
        } else {
            // Skip nodes deeper than direct child (they'll be handled recursively)
            warn!(
                "unexpected level {} (expected {} or less), skipping",
                level,
                parent_level + 1
            );
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Note parsing
// ---------------------------------------------------------------------------

enum WbsNoteParseResult {
    SingleLine(WbsNote),
    MultiLineStart { position: String },
}

/// Parse a WBS note line.
///
/// Supported forms:
///   `note left : text`    (single-line)
///   `note right`          (multi-line start)
fn try_parse_wbs_note(line: &str) -> Option<WbsNoteParseResult> {
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

        if let Some(after_colon) = after_pos.strip_prefix(':') {
            let text = after_colon
                .trim()
                .replace("\\n", "\n")
                .replace(crate::NEWLINE_CHAR, "\n");
            return Some(WbsNoteParseResult::SingleLine(WbsNote {
                text,
                position: pos.to_string(),
            }));
        }

        if after_pos.is_empty() {
            return Some(WbsNoteParseResult::MultiLineStart {
                position: pos.to_string(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 1. Basic single-root parse ──────────────────────────────────

    #[test]
    fn test_single_root() {
        let src = "@startwbs\n* Root\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.text, "Root");
        assert_eq!(d.root.level, 1);
        assert!(d.root.children.is_empty());
        assert!(d.links.is_empty());
    }

    // ── 2. Two levels ───────────────────────────────────────────────

    #[test]
    fn test_two_levels() {
        let src = "@startwbs\n* Root\n** Child1\n** Child2\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.children.len(), 2);
        assert_eq!(d.root.children[0].text, "Child1");
        assert_eq!(d.root.children[1].text, "Child2");
        assert_eq!(d.root.children[0].level, 2);
    }

    // ── 3. Three levels deep ────────────────────────────────────────

    #[test]
    fn test_three_levels() {
        let src = "@startwbs\n* Root\n** A\n*** A1\n*** A2\n** B\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.children.len(), 2);
        assert_eq!(d.root.children[0].children.len(), 2);
        assert_eq!(d.root.children[0].children[0].text, "A1");
        assert_eq!(d.root.children[0].children[0].level, 3);
        assert_eq!(d.root.children[1].text, "B");
        assert!(d.root.children[1].children.is_empty());
    }

    // ── 4. Direction suffixes ───────────────────────────────────────

    #[test]
    fn test_direction_suffixes() {
        let src = "@startwbs\n* Root\n**> Right\n**< Left\n** Default\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.children[0].direction, WbsDirection::Right);
        assert_eq!(d.root.children[1].direction, WbsDirection::Left);
        assert_eq!(d.root.children[2].direction, WbsDirection::Default);
    }

    // ── 5. Alias with parentheses ───────────────────────────────────

    #[test]
    fn test_alias_parentheses() {
        let src = "@startwbs\n* Root\n**(TLB) Team B\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.children[0].alias.as_deref(), Some("TLB"));
        assert_eq!(d.root.children[0].text, "Team B");
    }

    // ── 6. Alias with "text" as ALIAS ───────────────────────────────

    #[test]
    fn test_alias_quoted_as() {
        let src = r#"@startwbs
* Root
** "Team A" as TLA
@endwbs"#;
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.children[0].alias.as_deref(), Some("TLA"));
        assert_eq!(d.root.children[0].text, "Team A");
    }

    // ── 7. Link parsing ────────────────────────────────────────────

    #[test]
    fn test_link_parsing() {
        let src = r#"@startwbs
* r
** "Teamlead\nTeam A" as TLA
**(TLB) Teamlead\nTeam B
TLB -> TLA
@endwbs"#;
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.links.len(), 1);
        assert_eq!(d.links[0].from, "TLB");
        assert_eq!(d.links[0].to, "TLA");
    }

    // ── 8. Newline escape ───────────────────────────────────────────

    #[test]
    fn test_newline_escape() {
        let src = "@startwbs\n* Root\n** Line 1\\nLine 2\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.children[0].text, "Line 1\nLine 2");
    }

    // ── 9. URL link annotations preserved for rendering ────────────

    #[test]
    fn test_url_preserved() {
        let src = r#"@startwbs
* Root
** d1 [[http://plantuml.com{tooltip}]]
** d2 [[http://plantuml.com]]
@endwbs"#;
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(
            d.root.children[0].text,
            "d1 [[http://plantuml.com{tooltip}]]"
        );
        assert_eq!(d.root.children[1].text, "d2 [[http://plantuml.com]]");
    }

    // ── 10. URL on root node ────────────────────────────────────────

    #[test]
    fn test_url_on_root() {
        let src = r#"@startwbs
* [[http://plantuml.com]] r
** d1
@endwbs"#;
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.text, "[[http://plantuml.com]] r");
    }

    // ── 11. Empty diagram error ─────────────────────────────────────

    #[test]
    fn test_empty_diagram_error() {
        let src = "@startwbs\n@endwbs";
        let result = parse_wbs_diagram(src);
        assert!(result.is_err());
    }

    // ── 12. Comments and blank lines skipped ────────────────────────

    #[test]
    fn test_comments_skipped() {
        let src = "@startwbs\n' this is a comment\n\n* Root\n\n** A\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.text, "Root");
        assert_eq!(d.root.children.len(), 1);
    }

    // ── 13. Direction with deeper nesting ───────────────────────────

    #[test]
    fn test_direction_deep_nesting() {
        let src = r#"@startwbs
* Root
**> Line 1A\nLine 1B
***> Right R2A\nRight R2B
****> Right R3A\nRight R3B
***< Left L2A\nLeft L2B
****< Left L3A\nLeft L3B
@endwbs"#;
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.children.len(), 1);
        let c1 = &d.root.children[0];
        assert_eq!(c1.direction, WbsDirection::Right);
        assert_eq!(c1.text, "Line 1A\nLine 1B");
        assert_eq!(c1.children.len(), 2);
        assert_eq!(c1.children[0].direction, WbsDirection::Right);
        assert_eq!(c1.children[1].direction, WbsDirection::Left);
        assert_eq!(c1.children[0].children.len(), 1);
        assert_eq!(c1.children[0].children[0].direction, WbsDirection::Right);
    }

    // ── 14. Full wbs_arrow fixture ──────────────────────────────────

    #[test]
    fn test_wbs_arrow_fixture() {
        let src = r#"@startwbs
* r
** "Teamlead\nTeam A" as TLA
**(TLB) Teamlead\nTeam B
TLB -> TLA
@endwbs"#;
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.root.text, "r");
        assert_eq!(d.root.children.len(), 2);
        assert_eq!(d.root.children[0].alias.as_deref(), Some("TLA"));
        assert_eq!(d.root.children[0].text, "Teamlead\nTeam A");
        assert_eq!(d.root.children[1].alias.as_deref(), Some("TLB"));
        assert_eq!(d.root.children[1].text, "Teamlead\nTeam B");
        assert_eq!(d.links.len(), 1);
    }

    // ── 15. extract_wbs_block ───────────────────────────────────────

    #[test]
    fn test_extract_block() {
        let src = "@startwbs\n* Root\n** A\n@endwbs";
        let block = extract_wbs_block(src).unwrap();
        assert!(block.contains("* Root"));
        assert!(block.contains("** A"));
        assert!(!block.contains("@startwbs"));
        assert!(!block.contains("@endwbs"));
    }

    #[test]
    fn test_extract_block_missing_tags() {
        let src = "* Root\n** A";
        let block = extract_wbs_block(src);
        assert!(block.is_none());
    }

    // ── 16. count_stars ─────────────────────────────────────────────

    #[test]
    fn test_count_stars() {
        assert_eq!(count_stars("*"), (1, ""));
        assert_eq!(count_stars("** text"), (2, " text"));
        assert_eq!(count_stars("***> foo"), (3, "> foo"));
        assert_eq!(count_stars("****< bar"), (4, "< bar"));
    }

    // ── 17. parse_direction ─────────────────────────────────────────

    #[test]
    fn test_parse_direction_fn() {
        let (d, r) = parse_direction("> hello");
        assert_eq!(d, WbsDirection::Right);
        assert_eq!(r, " hello");

        let (d, r) = parse_direction("< world");
        assert_eq!(d, WbsDirection::Left);
        assert_eq!(r, " world");

        let (d, r) = parse_direction(" plain");
        assert_eq!(d, WbsDirection::Default);
        assert_eq!(r, " plain");
    }

    // ── 18. parse_alias_and_text with links preserved ───────────────

    #[test]
    fn test_parse_alias_and_text_preserves_links() {
        let (alias, text) = parse_alias_and_text("d1 [[http://plantuml.com{tooltip}]]");
        assert!(alias.is_none());
        assert_eq!(text, "d1 [[http://plantuml.com{tooltip}]]");
    }

    // ── 19. parse_alias_and_text ────────────────────────────────────

    #[test]
    fn test_parse_alias_and_text() {
        let (alias, text) = parse_alias_and_text("(TLB) Team B");
        assert_eq!(alias.as_deref(), Some("TLB"));
        assert_eq!(text, "Team B");

        let (alias, text) = parse_alias_and_text(r#""Team A" as TLA"#);
        assert_eq!(alias.as_deref(), Some("TLA"));
        assert_eq!(text, "Team A");

        let (alias, text) = parse_alias_and_text(" plain text");
        assert!(alias.is_none());
        assert_eq!(text, "plain text");
    }

    // ── 20. parse_link_line ─────────────────────────────────────────

    #[test]
    fn test_parse_link_line() {
        let link = parse_link_line("TLB -> TLA").unwrap();
        assert_eq!(link.from, "TLB");
        assert_eq!(link.to, "TLA");

        assert!(parse_link_line("no arrow here").is_none());
        assert!(parse_link_line("-> missing_from").is_none());
    }

    // ── 21. Note parsing ─────────────────────────────────────────────

    #[test]
    fn test_note_right() {
        let src = "@startwbs\n* Root\n** Child\nnote right : a wbs note\n@endwbs";
        let d = parse_wbs_diagram(src).unwrap();
        assert_eq!(d.notes.len(), 1);
        assert_eq!(d.notes[0].position, "right");
        assert_eq!(d.notes[0].text, "a wbs note");
    }
}
