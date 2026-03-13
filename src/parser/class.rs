use crate::model::{
    ArrowHead, ClassDiagram, ClassNote, Direction, Entity, EntityKind, Group, GroupKind, LineStyle,
    Link, Member, MemberModifiers, Stereotype, Visibility,
};
use crate::Result;
use log::{debug, warn};
use regex::Regex;

/// Parse class diagram source text into ClassDiagram IR
pub fn parse_class_diagram(source: &str) -> Result<ClassDiagram> {
    let block = super::common::extract_block(source);
    let content = block.as_deref().unwrap_or(source);

    // Preprocess: merge continuation lines (line ending with `\` joins next line)
    let merged = merge_continuation_lines(content);

    let mut entities: Vec<Entity> = Vec::new();
    let mut links: Vec<Link> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();
    let mut direction = Direction::TopToBottom;

    let mut notes: Vec<ClassNote> = Vec::new();

    let mut in_body = false;
    let mut current_entity: Option<Entity> = None;
    let mut in_style_block = false;
    let mut in_legend = false;
    let mut in_note_block = false;
    let mut note_block_position = String::new();
    let mut note_block_target: Option<String> = None;
    let mut note_block_lines: Vec<String> = Vec::new();
    let mut group_stack: Vec<Group> = Vec::new();
    let mut brace_depth: usize = 0;

    // Entity: class/interface/abstract class/abstract/enum/annotation/static class Name <<stereo>> #color {
    let re_entity = Regex::new(concat!(
        r#"(?x)"#,
        r#"^-?(class|interface|abstract\s+class|abstract|enum|annotation|static\s+class|object)"#,
        r#"\s+"#,
        r#"("(?:[^"]+)"|[\w.<>,\s]+?)"#,
        r#"\s*"#,
        r#"(?:<<([^>]+)>>(?:\s*<<([^>]+)>>)?(?:\s*<<([^>]+)>>)?)?"#,
        r#"\s*"#,
        r#"(\#\w+)?"#,
        r#"\s*"#,
        r#"(?:(\{)\s*(\})?)?\s*$"#,
    ))
    .unwrap();

    // Group: package/namespace/rectangle "name" <<stereo>> {
    let re_group = Regex::new(concat!(
        r#"(?x)"#,
        r#"^(package|namespace|rectangle)"#,
        r#"\s+"#,
        r#"("(?:[^"]+)"|[^\s{<]+(?:\s+[^\s{<]+)*)"#,
        r#"\s*"#,
        r#"(?:<<[^>]+>>(?:\s*<<[^>]+>>)*)?"#,
        r#"\s*"#,
        r#"\{"#,
    ))
    .unwrap();

    let re_direction_lr = Regex::new(r"^left\s+to\s+right\s+direction$").unwrap();
    let re_direction_tb = Regex::new(r"^top\s+to\s+bottom\s+direction$").unwrap();

    for line in merged.lines() {
        let trimmed = line.trim();

        // Handle style blocks
        if trimmed.starts_with("<style>") {
            in_style_block = true;
            debug!("entering <style> block");
            continue;
        }
        if in_style_block {
            if trimmed.starts_with("</style>") {
                in_style_block = false;
                debug!("leaving <style> block");
            }
            continue;
        }

        // Handle legend blocks (legend may be multi-line with `end legend`)
        if trimmed.starts_with("legend") {
            if trimmed == "legend" {
                in_legend = true;
            }
            continue;
        }
        if in_legend {
            if trimmed == "end legend" || trimmed == "endlegend" {
                in_legend = false;
            }
            continue;
        }

        // Handle multi-line note accumulation
        if in_note_block {
            if trimmed == "end note" || trimmed == "endnote" {
                let text = note_block_lines.join("\n");
                debug!("end note block: text={text:?}");
                notes.push(ClassNote {
                    text,
                    position: note_block_position.clone(),
                    target: note_block_target.take(),
                });
                note_block_lines.clear();
                in_note_block = false;
            } else {
                note_block_lines.push(trimmed.to_string());
            }
            continue;
        }

        // Skip empty and comment lines
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        // Skip known directives
        if should_skip_line(trimmed) {
            continue;
        }

        // Direction
        if re_direction_lr.is_match(trimmed) {
            direction = Direction::LeftToRight;
            debug!("direction set to LeftToRight");
            continue;
        }
        if re_direction_tb.is_match(trimmed) {
            direction = Direction::TopToBottom;
            debug!("direction set to TopToBottom");
            continue;
        }

        // Entity body parsing
        if in_body {
            if trimmed == "}" {
                in_body = false;
                if let Some(ent) = current_entity.take() {
                    debug!("finished entity body: {}", ent.name);
                    let name = ent.name.clone();
                    entities.push(ent);
                    if let Some(g) = group_stack.last_mut() {
                        g.entities.push(name);
                    }
                }
                continue;
            }
            if let Some(ref mut ent) = current_entity {
                if let Some(member) = parse_member(trimmed) {
                    ent.members.push(member);
                } else if !trimmed.starts_with("--")
                    && !trimmed.starts_with("==")
                    && !trimmed.starts_with("..")
                {
                    warn!("unrecognized member line: {trimmed}");
                }
            }
            continue;
        }

        // Group opening
        if let Some(caps) = re_group.captures(trimmed) {
            let kind_str = caps.get(1).unwrap().as_str();
            let name = caps.get(2).unwrap().as_str().trim_matches('"').to_string();
            let kind = match kind_str {
                "package" => GroupKind::Package,
                "namespace" => GroupKind::Namespace,
                "rectangle" => GroupKind::Rectangle,
                _ => GroupKind::Package,
            };
            debug!("opening group: {name} ({kind_str})");
            group_stack.push(Group {
                kind,
                name,
                entities: Vec::new(),
            });
            brace_depth += 1;
            continue;
        }

        // Closing brace for groups
        if trimmed == "}" && !group_stack.is_empty() {
            if let Some(g) = group_stack.pop() {
                debug!("closing group: {}", g.name);
                groups.push(g);
                brace_depth = brace_depth.saturating_sub(1);
            }
            continue;
        }

        // Entity declaration
        if let Some(caps) = re_entity.captures(trimmed) {
            let kind_str = caps.get(1).unwrap().as_str().trim();
            let raw_name = caps.get(2).unwrap().as_str().trim().trim_matches('"');
            let stereo1 = caps.get(3).map(|m| m.as_str().to_string());
            let stereo2 = caps.get(4).map(|m| m.as_str().to_string());
            let stereo3 = caps.get(5).map(|m| m.as_str().to_string());
            let color = caps.get(6).map(|m| m.as_str().to_string());
            let has_open_brace = caps.get(7).is_some();
            let has_close_brace = caps.get(8).is_some();

            let kind = parse_entity_kind(kind_str);

            let (name, generic) = parse_generic(raw_name);

            let mut stereotypes = Vec::new();
            for s in [stereo1, stereo2, stereo3].into_iter().flatten() {
                stereotypes.push(Stereotype(s));
            }

            debug!("entity declaration: {name} ({kind:?})");

            let entity = Entity {
                name: name.clone(),
                kind,
                stereotypes,
                members: Vec::new(),
                color,
                generic,
            };

            if has_open_brace && !has_close_brace {
                // Opening brace only: enter body mode
                in_body = true;
                current_entity = Some(entity);
            } else {
                // No brace or inline `{}`: treat as complete entity
                if let Some(g) = group_stack.last_mut() {
                    g.entities.push(name);
                }
                entities.push(entity);
            }
            continue;
        }

        // Relationship parsing
        if let Some(link) = parse_link(trimmed) {
            debug!("link: {} -> {} ({:?})", link.from, link.to, link.line_style);
            links.push(link);
            continue;
        }

        // Note parsing: single-line or multi-line start
        if let Some(note_result) = try_parse_class_note(trimmed) {
            match note_result {
                ClassNoteParseResult::SingleLine(note) => {
                    debug!("single-line note for {:?}", note.target);
                    notes.push(note);
                }
                ClassNoteParseResult::MultiLineStart { position, target } => {
                    debug!("start multi-line note for {target:?}");
                    in_note_block = true;
                    note_block_position = position;
                    note_block_target = target;
                    note_block_lines.clear();
                }
            }
            continue;
        }

        debug!("skipping unrecognized line: {trimmed}");
    }

    // If we were still parsing a body (missing closing brace), flush it
    if let Some(ent) = current_entity.take() {
        warn!("entity {} body not closed properly", ent.name);
        entities.push(ent);
    }

    // Flush any unclosed groups
    while let Some(g) = group_stack.pop() {
        warn!("group {} not closed properly", g.name);
        groups.push(g);
    }

    // Auto-create entities referenced in links but not declared
    auto_create_entities(&mut entities, &links);

    Ok(ClassDiagram {
        entities,
        links,
        groups,
        direction,
        notes,
    })
}

/// Merge continuation lines: a line ending with `\` (backslash at end) joins with the next line.
fn merge_continuation_lines(content: &str) -> String {
    let mut result = Vec::new();
    let mut carry = String::new();

    for line in content.lines() {
        if let Some(stripped) = line.strip_suffix('\\') {
            carry.push_str(stripped);
        } else if !carry.is_empty() {
            carry.push_str(line);
            result.push(carry.clone());
            carry.clear();
        } else {
            result.push(line.to_string());
        }
    }
    if !carry.is_empty() {
        result.push(carry);
    }

    result.join("\n")
}

fn should_skip_line(trimmed: &str) -> bool {
    let skip_prefixes = [
        "skinparam",
        "hide ",
        "show ",
        "title ",
        "title\t",
        "footer ",
        "footer\t",
        "header ",
        "header\t",
        "caption ",
        "caption\t",
        "remove ",
        "set ",
        "scale ",
    ];
    for prefix in &skip_prefixes {
        if trimmed.starts_with(prefix) {
            return true;
        }
    }
    if trimmed == "hide"
        || trimmed == "show"
        || trimmed == "title"
        || trimmed == "footer"
        || trimmed == "header"
        || trimmed == "caption"
    {
        return true;
    }
    false
}

fn parse_entity_kind(s: &str) -> EntityKind {
    match s {
        "class" | "static class" => EntityKind::Class,
        "interface" => EntityKind::Interface,
        "enum" => EntityKind::Enum,
        "abstract" | "abstract class" => EntityKind::Abstract,
        "annotation" => EntityKind::Annotation,
        "object" => EntityKind::Object,
        _ => EntityKind::Class,
    }
}

/// Parse generic from entity name, e.g. "HashMap<K,V>" -> ("HashMap", Some("K,V"))
fn parse_generic(name: &str) -> (String, Option<String>) {
    if let Some(idx) = name.find('<') {
        if name.ends_with('>') {
            let base = name[..idx].trim().to_string();
            let generic = name[idx + 1..name.len() - 1].to_string();
            return (base, Some(generic));
        }
    }
    (name.trim().to_string(), None)
}

/// Parse a member line inside an entity body
fn parse_member(line: &str) -> Option<Member> {
    let mut s = line.trim().to_string();

    if s.is_empty() {
        return None;
    }

    // Parse modifiers: {method}, {static}, {abstract}, {field}
    let mut modifiers = MemberModifiers::default();
    let mut force_method = false;
    let mut force_field = false;

    loop {
        let trimmed = s.trim_start();
        if let Some(rest) = trimmed.strip_prefix("{method}") {
            force_method = true;
            s = rest.to_string();
        } else if let Some(rest) = trimmed.strip_prefix("{static}") {
            modifiers.is_static = true;
            s = rest.to_string();
        } else if let Some(rest) = trimmed.strip_prefix("{abstract}") {
            modifiers.is_abstract = true;
            s = rest.to_string();
        } else if let Some(rest) = trimmed.strip_prefix("{field}") {
            force_field = true;
            s = rest.to_string();
        } else {
            break;
        }
    }

    let s = s.trim();

    // Parse visibility
    let (visibility, rest) = if let Some(first) = s.chars().next() {
        match first {
            '+' => (Some(Visibility::Public), s[1..].trim()),
            '-' => (Some(Visibility::Private), s[1..].trim()),
            '#' => (Some(Visibility::Protected), s[1..].trim()),
            '~' => (Some(Visibility::Package), s[1..].trim()),
            _ => (None, s),
        }
    } else {
        return None;
    };

    if rest.is_empty() {
        return None;
    }

    // Detect method: contains `(` or has {method} modifier
    let is_method = force_method || (!force_field && rest.contains('('));

    // Parse name and return_type
    let (name, return_type) = if is_method {
        if let Some(paren_close) = rest.rfind(')') {
            let method_part = &rest[..=paren_close];
            let after = rest[paren_close + 1..].trim();
            if let Some(stripped) = after.strip_prefix(':') {
                (
                    method_part.trim().to_string(),
                    Some(stripped.trim().to_string()),
                )
            } else {
                (method_part.trim().to_string(), None)
            }
        } else {
            // {method} modifier but no parens
            if let Some((name_part, type_part)) = rest.split_once(':') {
                (
                    name_part.trim().to_string(),
                    Some(type_part.trim().to_string()),
                )
            } else {
                (rest.to_string(), None)
            }
        }
    } else {
        // Field
        if let Some((name_part, type_part)) = rest.split_once(':') {
            (
                name_part.trim().to_string(),
                Some(type_part.trim().to_string()),
            )
        } else {
            (rest.to_string(), None)
        }
    };

    Some(Member {
        visibility,
        name,
        return_type,
        is_method,
        modifiers,
    })
}

/// Parse a relationship/link line.
///
/// Arrow patterns:
///   left_head + line + right_head
///   left heads: `<|`, `<`, `*`, `o`, `+`, or none
///   line: `--` (solid) or `..` (dashed), with optional direction hint letters
///   right heads: `|>`, `>`, `*`, `o`, `+`, or none
fn parse_link(line: &str) -> Option<Link> {
    // Build pattern for the arrow itself
    // Left heads: <|, <, *, o, +, or nothing
    // Line: --..variations with optional direction letters
    // Right heads: |>, >, *, o, +, or nothing
    let re = Regex::new(concat!(
        r"(?x)",
        r"^([\w.]+)", // from entity
        r"\s*",
        r"(?:\[[^\]]*\])?", // optional qualifier [...]
        r"\s+",
        r"(",                  // arrow group start
        r"(?:<\||\*|o|\+|<)?", // optional left head
        r"(?:",
        r"-+[udlr]*-+", // solid: --  -u-  ---
        r"|",
        r"\.+[udlr]*\.+", // dashed: ..  .r.  ...
        r"|",
        r"-+[udlr]*-*>", // solid ending with >: --> -u->
        r"|",
        r"\.+[udlr]*\.*>", // dashed ending with >: ..> .r.>
        r"|",
        r"-+[udlr]*-*\|>", // solid ending with |>: --|>
        r"|",
        r"\.+[udlr]*\.*\|>", // dashed ending with |>: ..|>
        r"|",
        r"-+[udlr]*-*\*", // solid ending with *: --*
        r"|",
        r"\.+[udlr]*\.*\*", // dashed ending with *: ..*
        r"|",
        r"-+[udlr]*-*o", // solid ending with o: --o
        r"|",
        r"\.+[udlr]*\.*o", // dashed ending with o: ..o
        r"|",
        r"-+[udlr]*-*\+", // solid ending with +: --+
        r"|",
        r"\.+[udlr]*\.*\+", // dashed ending with +: ..+
        r")",
        r")", // arrow group end
        r"\s*",
        r#"(?:"([^"]*)"\s+)?"#, // optional to-label "..."
        r"([\w.]+)",            // to entity
        r"\s*",
        r"(?:\s*:\s*(.*))?", // optional label
        r"$",
    ))
    .unwrap();

    let trimmed = line.trim();

    if let Some(caps) = re.captures(trimmed) {
        let from = caps.get(1).unwrap().as_str().to_string();
        let arrow = caps.get(2).unwrap().as_str();
        let to_label = caps.get(3).map(|m| m.as_str().trim().to_string());
        let to = caps.get(4).unwrap().as_str().to_string();
        let label = caps.get(5).map(|m| m.as_str().trim().to_string());

        let (left_head, line_style, right_head) = parse_arrow(arrow);

        return Some(Link {
            from,
            to,
            left_head,
            right_head,
            line_style,
            label,
            from_label: None,
            to_label,
        });
    }

    // Also try with qualifier brackets between arrow and entity
    let re2 = Regex::new(concat!(
        r"(?x)",
        r"^([\w.]+)", // from entity
        r"\s*",
        r"(?:\[[^\]]*\])?", // optional qualifier [...]
        r"\s+",
        r"(", // arrow group start
        r"(?:<\||\*|o|\+|<)?",
        r"(?:",
        r"-+[udlr]*-+",
        r"|\.+[udlr]*\.+",
        r"|-+[udlr]*-*>",
        r"|\.+[udlr]*\.*>",
        r"|-+[udlr]*-*\|>",
        r"|\.+[udlr]*\.*\|>",
        r"|-+[udlr]*-*\*",
        r"|\.+[udlr]*\.*\*",
        r"|-+[udlr]*-*o",
        r"|\.+[udlr]*\.*o",
        r"|-+[udlr]*-*\+",
        r"|\.+[udlr]*\.*\+",
        r")",
        r")", // arrow group end
        r"\s*",
        r"(?:\[[^\]]*\])?", // optional qualifier [...]
        r"\s*",
        r#"(?:"([^"]*)"\s+)?"#, // optional to-label "..."
        r"([\w.]+)",            // to entity
        r"\s*",
        r"(?:\s*:\s*(.*))?", // optional label
        r"$",
    ))
    .unwrap();

    if let Some(caps) = re2.captures(trimmed) {
        let from = caps.get(1).unwrap().as_str().to_string();
        let arrow = caps.get(2).unwrap().as_str();
        let to_label = caps.get(3).map(|m| m.as_str().trim().to_string());
        let to = caps.get(4).unwrap().as_str().to_string();
        let label = caps.get(5).map(|m| m.as_str().trim().to_string());

        let (left_head, line_style, right_head) = parse_arrow(arrow);

        return Some(Link {
            from,
            to,
            left_head,
            right_head,
            line_style,
            label,
            from_label: None,
            to_label,
        });
    }

    None
}

/// Parse an arrow string into (left_head, line_style, right_head)
fn parse_arrow(arrow: &str) -> (ArrowHead, LineStyle, ArrowHead) {
    // Parse left head
    let (left_head, rest) = if let Some(r) = arrow.strip_prefix("<|") {
        (ArrowHead::Triangle, r)
    } else if let Some(r) = arrow.strip_prefix('<') {
        (ArrowHead::Arrow, r)
    } else if let Some(r) = arrow.strip_prefix('*') {
        (ArrowHead::Diamond, r)
    } else if let Some(r) = arrow.strip_prefix('o') {
        (ArrowHead::DiamondHollow, r)
    } else if let Some(r) = arrow.strip_prefix('+') {
        (ArrowHead::Plus, r)
    } else {
        (ArrowHead::None, arrow)
    };

    // Parse right head (from end)
    let (right_head, middle) = if let Some(m) = rest.strip_suffix("|>") {
        (ArrowHead::Triangle, m)
    } else if let Some(m) = rest.strip_suffix('>') {
        (ArrowHead::Arrow, m)
    } else if let Some(m) = rest.strip_suffix('*') {
        (ArrowHead::Diamond, m)
    } else if let Some(m) = rest.strip_suffix('o') {
        (ArrowHead::DiamondHollow, m)
    } else if let Some(m) = rest.strip_suffix('+') {
        (ArrowHead::Plus, m)
    } else {
        (ArrowHead::None, rest)
    };

    // Determine line style from the middle part (the line chars)
    let line_style = if middle.contains('.') {
        LineStyle::Dashed
    } else {
        LineStyle::Solid
    };

    (left_head, line_style, right_head)
}

// ---------------------------------------------------------------------------
// Note parsing
// ---------------------------------------------------------------------------

enum ClassNoteParseResult {
    SingleLine(ClassNote),
    MultiLineStart {
        position: String,
        target: Option<String>,
    },
}

/// Try to parse a note line.
///
/// Supported forms:
///   `note left of EntityName : text`   (single-line)
///   `note right of EntityName`          (multi-line start)
///   `note left : text`                  (floating single-line)
///   `note right`                        (floating multi-line)
fn try_parse_class_note(line: &str) -> Option<ClassNoteParseResult> {
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

        // `note <pos> of Target : text` or `note <pos> of Target`
        if let Some(after_of) = after_pos.strip_prefix("of ") {
            let after_of = after_of.trim();

            if let Some(colon_pos) = after_of.find(':') {
                let target = after_of[..colon_pos].trim().to_string();
                let text = after_of[colon_pos + 1..].trim().replace("\\n", "\n");
                return Some(ClassNoteParseResult::SingleLine(ClassNote {
                    text,
                    position: pos.to_string(),
                    target: Some(target),
                }));
            }

            let target = after_of.trim().to_string();
            return Some(ClassNoteParseResult::MultiLineStart {
                position: pos.to_string(),
                target: if target.is_empty() {
                    None
                } else {
                    Some(target)
                },
            });
        }

        // `note <pos> : text` or `note <pos>` (no target)
        if let Some(after_colon) = after_pos.strip_prefix(':') {
            let text = after_colon.trim().replace("\\n", "\n");
            return Some(ClassNoteParseResult::SingleLine(ClassNote {
                text,
                position: pos.to_string(),
                target: None,
            }));
        }

        if after_pos.is_empty() {
            return Some(ClassNoteParseResult::MultiLineStart {
                position: pos.to_string(),
                target: None,
            });
        }
    }

    None
}

/// Auto-create entities that appear in links but were not declared
fn auto_create_entities(entities: &mut Vec<Entity>, links: &[Link]) {
    let known: std::collections::HashSet<String> =
        entities.iter().map(|e| e.name.clone()).collect();

    let mut to_add = Vec::new();
    for link in links {
        for name in [&link.from, &link.to] {
            if !known.contains(name.as_str()) && !to_add.contains(name) {
                debug!("auto-creating entity: {name}");
                to_add.push(name.clone());
            }
        }
    }

    for name in to_add {
        entities.push(Entity {
            name,
            kind: EntityKind::Class,
            stereotypes: Vec::new(),
            members: Vec::new(),
            color: None,
            generic: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn parse(body: &str) -> ClassDiagram {
        let src = format!("@startuml\n{}\n@enduml", body);
        parse_class_diagram(&src).expect("parse should succeed")
    }

    // 1. Parse empty class
    #[test]
    fn parse_empty_class() {
        let cd = parse("class Foo {}");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].name, "Foo");
        assert_eq!(cd.entities[0].kind, EntityKind::Class);
        assert!(cd.entities[0].members.is_empty());
    }

    // 2. Parse class with members (fields + methods with visibility)
    #[test]
    fn parse_class_with_members() {
        let cd = parse(
            "class A {\n  - name: String\n  + id: long\n  # doSomething(): void\n  ~run(): boolean\n}",
        );
        assert_eq!(cd.entities.len(), 1);
        let ent = &cd.entities[0];
        assert_eq!(ent.members.len(), 4);

        let m0 = &ent.members[0];
        assert_eq!(m0.visibility, Some(Visibility::Private));
        assert_eq!(m0.name, "name");
        assert_eq!(m0.return_type.as_deref(), Some("String"));
        assert!(!m0.is_method);

        let m1 = &ent.members[1];
        assert_eq!(m1.visibility, Some(Visibility::Public));
        assert!(!m1.is_method);

        let m2 = &ent.members[2];
        assert_eq!(m2.visibility, Some(Visibility::Protected));
        assert!(m2.is_method);
        assert_eq!(m2.return_type.as_deref(), Some("void"));

        let m3 = &ent.members[3];
        assert_eq!(m3.visibility, Some(Visibility::Package));
        assert!(m3.is_method);
    }

    // 3. Parse abstract class
    #[test]
    fn parse_abstract_class() {
        let cd = parse("abstract class B {}");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].kind, EntityKind::Abstract);
        assert_eq!(cd.entities[0].name, "B");
    }

    // 4. Parse interface
    #[test]
    fn parse_interface() {
        let cd = parse("interface Runnable {}");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].kind, EntityKind::Interface);
    }

    // 5. Parse enum
    #[test]
    fn parse_enum() {
        let cd = parse("enum Color {}");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].kind, EntityKind::Enum);
    }

    // 6. Parse extension arrow: A --|> B
    #[test]
    fn parse_extension_arrow() {
        let cd = parse("A --|> B");
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.from, "A");
        assert_eq!(link.to, "B");
        assert_eq!(link.left_head, ArrowHead::None);
        assert_eq!(link.right_head, ArrowHead::Triangle);
        assert_eq!(link.line_style, LineStyle::Solid);
    }

    // 7. Parse implementation arrow: A ..|> B
    #[test]
    fn parse_implementation_arrow() {
        let cd = parse("A ..|> B");
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.left_head, ArrowHead::None);
        assert_eq!(link.right_head, ArrowHead::Triangle);
        assert_eq!(link.line_style, LineStyle::Dashed);
    }

    // 8. Parse composition: A *-- B
    #[test]
    fn parse_composition() {
        let cd = parse("A *-- B");
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.left_head, ArrowHead::Diamond);
        assert_eq!(link.line_style, LineStyle::Solid);
    }

    // 9. Parse aggregation: A o-- B
    #[test]
    fn parse_aggregation() {
        let cd = parse("A o-- B");
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.left_head, ArrowHead::DiamondHollow);
        assert_eq!(link.line_style, LineStyle::Solid);
    }

    // 10. Parse dependency: A ..> B
    #[test]
    fn parse_dependency() {
        let cd = parse("A ..> B");
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.right_head, ArrowHead::Arrow);
        assert_eq!(link.line_style, LineStyle::Dashed);
    }

    // 11. Parse association with label: A --> B : uses
    #[test]
    fn parse_association_with_label() {
        let cd = parse("A --> B : uses");
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.right_head, ArrowHead::Arrow);
        assert_eq!(link.line_style, LineStyle::Solid);
        assert_eq!(link.label.as_deref(), Some("uses"));
    }

    // 12. Parse class with stereotype
    #[test]
    fn parse_class_with_stereotype() {
        let cd = parse("class Access <<Entity>>");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].stereotypes.len(), 1);
        assert_eq!(cd.entities[0].stereotypes[0].0, "Entity");
    }

    // 13. Parse class with generic
    #[test]
    fn parse_class_with_generic() {
        let cd = parse("class HashMap<K,V>");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].name, "HashMap");
        assert_eq!(cd.entities[0].generic.as_deref(), Some("K,V"));
    }

    // 14. Parse package group
    #[test]
    fn parse_package_group() {
        let cd = parse("package mypackage {\n  class A\n  class B\n}");
        assert_eq!(cd.groups.len(), 1);
        assert_eq!(cd.groups[0].kind, GroupKind::Package);
        assert_eq!(cd.groups[0].name, "mypackage");
        assert_eq!(cd.groups[0].entities.len(), 2);
        assert!(cd.groups[0].entities.contains(&"A".to_string()));
        assert!(cd.groups[0].entities.contains(&"B".to_string()));
    }

    // 15. Parse direction directive
    #[test]
    fn parse_direction_left_to_right() {
        let cd = parse("left to right direction\nclass Foo");
        assert_eq!(cd.direction, Direction::LeftToRight);
    }

    // 16. Auto-create entity from relationship
    #[test]
    fn auto_create_from_relationship() {
        let cd = parse("A --> B");
        assert_eq!(cd.entities.len(), 2);
        assert!(cd.entities.iter().any(|e| e.name == "A"));
        assert!(cd.entities.iter().any(|e| e.name == "B"));
    }

    // 17. Skip style block / skinparam / comments
    #[test]
    fn skip_style_and_comments() {
        let cd = parse(
            "<style>\n  body { color: red; }\n</style>\nskinparam classBackgroundColor White\n' comment\nclass Foo",
        );
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].name, "Foo");
    }

    // 18. Parse member with modifiers
    #[test]
    fn parse_member_modifiers() {
        let cd = parse("class A {\n  {method}{abstract}{static} + method\n}");
        assert_eq!(cd.entities[0].members.len(), 1);
        let m = &cd.entities[0].members[0];
        assert!(m.is_method);
        assert!(m.modifiers.is_static);
        assert!(m.modifiers.is_abstract);
        assert_eq!(m.visibility, Some(Visibility::Public));
    }

    // 19. Parse fixture xmi0002
    #[test]
    fn parse_fixture_xmi0002() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/class/xmi0002.puml"
        ))
        .unwrap();
        let cd = parse_class_diagram(&src).unwrap();
        assert_eq!(cd.entities.len(), 2);
        assert_eq!(cd.links.len(), 1);
        assert_eq!(cd.links[0].from, "A");
        assert_eq!(cd.links[0].to, "B");
    }

    // 20. Parse fixture xmi0004 - dependency (.>)
    #[test]
    fn parse_fixture_xmi0004() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/class/xmi0004.puml"
        ))
        .unwrap();
        let cd = parse_class_diagram(&src).unwrap();
        assert_eq!(cd.links.len(), 1);
        assert_eq!(cd.links[0].line_style, LineStyle::Dashed);
        assert_eq!(cd.links[0].right_head, ArrowHead::Arrow);
    }

    // 21. Parse fixture hideshow002 - rectangle, package groups
    #[test]
    fn parse_fixture_hideshow002() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/class/hideshow002.puml"
        ))
        .unwrap();
        let cd = parse_class_diagram(&src).unwrap();
        assert!(cd.groups.len() >= 2);
        assert!(cd.entities.len() >= 4);
    }

    // 22. Parse fixture a0005 - style blocks, title, legend, footer, header
    #[test]
    fn parse_fixture_a0005() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/class/a0005.puml"
        ))
        .unwrap();
        let cd = parse_class_diagram(&src).unwrap();
        assert!(cd.entities.iter().any(|e| e.name == "Bob"));
        assert!(cd.entities.iter().any(|e| e.name == "Sally"));
        assert_eq!(cd.links.len(), 1);
    }

    // 23. Parse multiline labels
    #[test]
    fn parse_multiline_labels() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/class/class_funcparam_arrow_01.puml"
        ))
        .unwrap();
        let cd = parse_class_diagram(&src).unwrap();
        assert_eq!(cd.entities.len(), 4);
        assert_eq!(cd.links.len(), 3);
        assert!(cd.links[0].label.is_some());
    }

    // ── Object diagram tests ──

    #[test]
    fn parse_object_simple() {
        let cd = parse("object London");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].name, "London");
        assert_eq!(cd.entities[0].kind, EntityKind::Object);
    }

    #[test]
    fn parse_multiple_objects() {
        let cd = parse("object London\nobject Washington\nobject Berlin");
        assert_eq!(cd.entities.len(), 3);
        assert!(cd.entities.iter().all(|e| e.kind == EntityKind::Object));
    }

    #[test]
    fn parse_object_empty_body() {
        let cd = parse("object Foo {}");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].kind, EntityKind::Object);
        assert!(cd.entities[0].members.is_empty());
    }

    #[test]
    fn parse_object_with_fields() {
        let cd = parse("object User {\n  name: String\n  age: int\n}");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].kind, EntityKind::Object);
        assert_eq!(cd.entities[0].members.len(), 2);
        assert!(!cd.entities[0].members[0].is_method);
        assert!(!cd.entities[0].members[1].is_method);
    }

    #[test]
    fn parse_object_with_relationships() {
        let cd = parse("object A\nobject B\nA --> B : link");
        assert_eq!(cd.entities.len(), 2);
        assert_eq!(cd.links.len(), 1);
        assert_eq!(cd.links[0].from, "A");
        assert_eq!(cd.links[0].to, "B");
        assert_eq!(cd.links[0].label.as_deref(), Some("link"));
    }

    #[test]
    fn parse_object_with_stereotype() {
        let cd = parse("object Server <<Singleton>>");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].kind, EntityKind::Object);
        assert_eq!(cd.entities[0].stereotypes.len(), 1);
        assert_eq!(cd.entities[0].stereotypes[0].0, "Singleton");
    }

    #[test]
    fn parse_object_with_color() {
        let cd = parse("object Server #red");
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].kind, EntityKind::Object);
        assert_eq!(cd.entities[0].color.as_deref(), Some("#red"));
    }

    #[test]
    fn parse_mixed_class_and_object() {
        let cd = parse("class Car\nobject myCar\nmyCar --> Car");
        assert_eq!(cd.entities.len(), 2);
        assert!(cd
            .entities
            .iter()
            .any(|e| e.name == "Car" && e.kind == EntityKind::Class));
        assert!(cd
            .entities
            .iter()
            .any(|e| e.name == "myCar" && e.kind == EntityKind::Object));
        assert_eq!(cd.links.len(), 1);
    }

    #[test]
    fn parse_object_quoted_name() {
        let cd = parse(r#"object "My Server" "#);
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.entities[0].name, "My Server");
        assert_eq!(cd.entities[0].kind, EntityKind::Object);
    }

    #[test]
    fn parse_object_visibility_fields() {
        let cd = parse("object Config {\n  + host: String\n  - port: int\n}");
        assert_eq!(cd.entities[0].members.len(), 2);
        assert_eq!(
            cd.entities[0].members[0].visibility,
            Some(Visibility::Public)
        );
        assert_eq!(
            cd.entities[0].members[1].visibility,
            Some(Visibility::Private)
        );
    }

    #[test]
    fn parse_fixture_object_basic() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/object/basic.puml"
        ))
        .unwrap();
        let cd = parse_class_diagram(&src).unwrap();
        assert_eq!(cd.entities.len(), 3);
        assert!(cd.entities.iter().all(|e| e.kind == EntityKind::Object));
        assert_eq!(cd.links.len(), 2);
        assert!(cd.entities.iter().any(|e| e.name == "London"));
        assert!(cd.entities.iter().any(|e| e.name == "Washington"));
        assert!(cd.entities.iter().any(|e| e.name == "Berlin"));
    }

    // ── Note parsing tests ──

    #[test]
    fn parse_single_line_note() {
        let cd = parse("class Foo\nnote left of Foo : this is a note");
        assert_eq!(cd.notes.len(), 1);
        assert_eq!(cd.notes[0].position, "left");
        assert_eq!(cd.notes[0].target.as_deref(), Some("Foo"));
        assert_eq!(cd.notes[0].text, "this is a note");
    }

    #[test]
    fn parse_multi_line_note() {
        let cd = parse("class Bar\nnote right of Bar\nline one\nline two\nend note");
        assert_eq!(cd.notes.len(), 1);
        assert_eq!(cd.notes[0].position, "right");
        assert_eq!(cd.notes[0].target.as_deref(), Some("Bar"));
        assert_eq!(cd.notes[0].text, "line one\nline two");
    }
}
