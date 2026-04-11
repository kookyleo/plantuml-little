use std::collections::HashMap;

use crate::model::{
    ArrowHead, ClassDiagram, ClassHideShowRule, ClassNote, ClassPortion, ClassRuleTarget,
    Direction, Entity, EntityKind, Group, GroupKind, LineStyle, Link, Member, MemberModifiers,
    RectSymbol, Stereotype, Visibility,
};
use crate::Result;
use log::{debug, warn};
use regex::Regex;

/// Parse class diagram source text into ClassDiagram IR
pub fn parse_class_diagram(source: &str) -> Result<ClassDiagram> {
    parse_class_diagram_with_original(source, None)
}

pub fn parse_class_diagram_with_original(
    source: &str,
    original_source: Option<&str>,
) -> Result<ClassDiagram> {
    let block = super::common::extract_block(source);
    let content = block.as_deref().unwrap_or(source);
    let line_mapping = build_line_mapping(source, original_source, content);

    // Preprocess: merge continuation lines (line ending with `\` joins next line)
    let merged = merge_continuation_lines_with_mapping(content, &line_mapping);

    let mut entities: Vec<Entity> = Vec::new();
    let mut links: Vec<Link> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();
    let mut direction = Direction::TopToBottom;
    let mut direction_explicit = false;
    let mut hide_show_rules = Vec::new();
    let mut stereotype_backgrounds = HashMap::new();
    let mut entity_order: Vec<String> = Vec::new();
    let mut next_uid_value: u32 = 1;
    let mut entity_uids: HashMap<String, String> = HashMap::new();
    let mut entity_first_source_lines: HashMap<String, usize> = HashMap::new();
    let mut known_implicit_groups: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    let mut notes: Vec<ClassNote> = Vec::new();

    let mut in_body = false;
    let mut current_entity: Option<Entity> = None;
    let mut in_style_block = false;
    let mut in_legend = false;
    let mut in_note_block = false;
    let mut in_description_block = false;
    let mut description_block_lines: Vec<String> = Vec::new();
    let mut note_block_position = String::new();
    let mut note_block_target: Option<String> = None;
    let mut note_block_lines: Vec<String> = Vec::new();
    let mut group_stack: Vec<Group> = Vec::new();
    let mut brace_depth: usize = 0;
    let mut active_style_stereotype: Option<String> = None;
    let mut last_entity_name: Option<String> = None;

    // Entity: [visibility]class/interface/abstract class/abstract/enum/annotation/static class Name [as Alias] <<stereo>> #color {
    // Visibility prefix: +/-/#/~ before the keyword (e.g. -class foo)
    let re_entity = Regex::new(concat!(
        r#"(?x)"#,
        r#"^([+\-\#~])?(class|interface|abstract\s+class|abstract|enum|annotation|static\s+class|object|map|rectangle|component|file|folder|frame|card|agent|storage|artifact|node|cloud|stack|queue)"#,
        r#"\s+"#,
        r#"("(?:[^"]+)"|[\w.<>,\s]+?)"#,
        r#"(?:\s+as\s+([\w]+))?"#,
        r#"\s*"#,
        r#"(?:<<([^>]+)>>(?:\s*<<([^>]+)>>)?(?:\s*<<([^>]+)>>)?)?"#,
        r#"\s*"#,
        r#"(\#\w+)?"#,
        r#"\s*"#,
        r#"(?:(\{)\s*(\})?|(\[))?\s*$"#,
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

    for (line, source_line) in &merged {
        let source_line = *source_line;
        let trimmed = line.trim();

        // Handle multi-line note accumulation FIRST: lines inside a note body
        // (e.g. embedded `{{ ... }}` sub-diagrams) may legitimately contain
        // tokens like `<style>` or `legend` that must be preserved verbatim,
        // not interpreted as outer-diagram constructs.
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

        // Handle style blocks
        if trimmed.starts_with("<style>") {
            in_style_block = true;
            debug!("entering <style> block");
            continue;
        }
        if in_style_block {
            if let Some(stereo) = trimmed
                .strip_prefix('.')
                .and_then(|s| s.strip_suffix('{'))
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                active_style_stereotype = Some(stereo.to_string());
            } else if let Some(stereo) = active_style_stereotype.as_deref() {
                if let Some(color) = parse_style_background_color(trimmed) {
                    stereotype_backgrounds.insert(stereo.to_string(), color.to_string());
                }
                if trimmed == "}" {
                    active_style_stereotype = None;
                }
            }
            if trimmed.starts_with("</style>") {
                in_style_block = false;
                active_style_stereotype = None;
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

        // Handle multi-line description block (rectangle A [...])
        // NOTE: blank lines inside `[...]` must be preserved (Java keeps them
        // as empty Display entries rendered as &#160;), so this check sits
        // BEFORE the generic empty/comment skip below.
        if in_description_block {
            if trimmed == "]" {
                if let Some(ref mut ent) = current_entity {
                    // Java `CommandCreateElementMultilines.executeNow`:
                    //   lines = lines.trimSmart(1).expandsNewline(false);
                    //   Display display = lines.toDisplay();
                    // so each physical line becomes one Display entry with
                    // U+E100 preserved in-place.  Downstream, `StripeTable`
                    // splits individual table cells on U+E100, while
                    // non-table lines handle newline expansion themselves.
                    // We therefore keep U+E100 (and `%chr(10)`) intact in the
                    // description lines and let the Creole renderer split as
                    // needed per-line.
                    ent.description = description_block_lines
                        .iter()
                        .map(|l| l.replace("%chr(10)", "\n").trim_end().to_string())
                        .collect();
                    let name = ent.name.clone();
                    entities.push(current_entity.take().unwrap());
                    if let Some(g) = group_stack.last_mut() {
                        g.entities.push(name);
                    }
                    debug!("finished entity description block");
                }
                description_block_lines.clear();
                in_description_block = false;
            } else {
                description_block_lines.push(trimmed.to_string());
            }
            continue;
        }

        // Skip empty and comment lines (AFTER in_description_block check so
        // blank lines inside bracket-body survive as nbsp rows).
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        if let Some(rule) = parse_hide_show_rule(trimmed) {
            for r in &rule {
                if let ClassRuleTarget::Entity(name) = &r.target {
                    reserve_entity_order(&mut entity_order, name);
                }
            }
            hide_show_rules.extend(rule);
            continue;
        }

        // Skip known directives
        if should_skip_line(trimmed) {
            continue;
        }

        // Direction
        if re_direction_lr.is_match(trimmed) {
            direction = Direction::LeftToRight;
            direction_explicit = true;
            debug!("direction set to LeftToRight (explicit)");
            continue;
        }
        if re_direction_tb.is_match(trimmed) {
            direction = Direction::TopToBottom;
            direction_explicit = true;
            debug!("direction set to TopToBottom (explicit)");
            continue;
        }

        // Entity body parsing
        if in_body {
            if trimmed == "}" {
                in_body = false;
                if let Some(ent) = current_entity.take() {
                    debug!("finished entity body: {}", ent.name);
                    let name = ent.name.clone();
                    last_entity_name = Some(name.clone());
                    entities.push(ent);
                    if let Some(g) = group_stack.last_mut() {
                        g.entities.push(name);
                    }
                }
                continue;
            }
            if let Some(ref mut ent) = current_entity {
                if ent.kind == EntityKind::Map {
                    if let Some(ap) = trimmed.find("=>") {
                        let k = trimmed[..ap].trim().to_string();
                        let v = trimmed[ap + 2..].trim().to_string();
                        ent.map_entries.push((k, v));
                    } else if !trimmed.starts_with("--")
                        && !trimmed.starts_with("==")
                        && !trimmed.starts_with("..")
                    {
                        debug!("map: ignoring non-entry line: {trimmed}");
                    }
                } else if let Some(member) = parse_member(trimmed) {
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
            let rest = trimmed[caps.get(2).unwrap().end()..]
                .trim_end_matches('{')
                .trim();
            let kind = match kind_str {
                "package" => GroupKind::Package,
                "namespace" => GroupKind::Namespace,
                "rectangle" => GroupKind::Rectangle,
                _ => GroupKind::Package,
            };
            let stereotypes = parse_stereotypes_from_tail(rest);
            let color = parse_color_from_tail(rest);
            debug!("opening group: {name} ({kind_str})");
            known_implicit_groups.insert(name.clone());
            group_stack.push(Group {
                uid: Some(next_entity_uid(&mut next_uid_value)),
                kind,
                name,
                entities: Vec::new(),
                stereotypes,
                color,
                source_line: Some(source_line),
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
            let entity_visibility = caps.get(1).and_then(|m| match m.as_str() {
                "+" => Some(Visibility::Public),
                "-" => Some(Visibility::Private),
                "#" => Some(Visibility::Protected),
                "~" => Some(Visibility::Package),
                _ => None,
            });
            let kind_str = caps.get(2).unwrap().as_str().trim();
            let raw_name = caps.get(3).unwrap().as_str().trim().trim_matches('"');
            let alias = caps.get(4).map(|m| m.as_str().to_string());
            let stereo1 = caps.get(5).map(|m| m.as_str().to_string());
            let stereo2 = caps.get(6).map(|m| m.as_str().to_string());
            let stereo3 = caps.get(7).map(|m| m.as_str().to_string());
            let color = caps.get(8).map(|m| m.as_str().to_string());
            let has_open_brace = caps.get(9).is_some();
            let has_close_brace = caps.get(10).is_some();
            let has_open_bracket = caps.get(11).is_some();

            let kind = parse_entity_kind(kind_str);
            let rect_symbol = parse_rect_symbol(kind_str);

            // When `as Alias` is used, alias becomes the code name and raw_name is display
            let (name, display_name) = if let Some(ref al) = alias {
                (al.clone(), Some(raw_name.to_string()))
            } else {
                let (n, _gen) = parse_generic(raw_name);
                (n, None)
            };
            let generic = if alias.is_none() {
                let (_n, g) = parse_generic(raw_name);
                g
            } else {
                None
            };

            let mut stereotypes = Vec::new();
            for s in [stereo1, stereo2, stereo3].into_iter().flatten() {
                stereotypes.push(Stereotype(s));
            }

            debug!("entity declaration: {name} ({kind:?})");
            reserve_entity_order(&mut entity_order, &name);
            let uid = ensure_entity_uid(
                &mut entity_uids,
                &mut entity_first_source_lines,
                &mut next_uid_value,
                &name,
                source_line,
            );
            // Java creates implicit package groups immediately when an entity
            // with dots is registered, so group UIDs come before link UIDs.
            synthesize_implicit_groups_for_entity(
                &name,
                Some(source_line),
                &mut groups,
                &mut known_implicit_groups,
                &mut next_uid_value,
            );

            let entity = Entity {
                uid: Some(uid),
                name: name.clone(),
                kind,
                stereotypes,
                members: Vec::new(),
                description: Vec::new(),
                color,
                generic,
                source_line: entity_first_source_lines.get(&name).copied(),
                visibility: entity_visibility,
                display_name,
                map_entries: vec![],
                rect_symbol,
            };

            if has_open_brace && !has_close_brace {
                // Opening brace only: enter body mode
                in_body = true;
                current_entity = Some(entity);
            } else if has_open_bracket {
                // Opening bracket: enter description mode (rectangle [...])
                in_description_block = true;
                current_entity = Some(entity);
                description_block_lines.clear();
            } else {
                // No brace or inline `{}`: treat as complete entity
                last_entity_name = Some(name.clone());
                if let Some(g) = group_stack.last_mut() {
                    g.entities.push(name);
                }
                entities.push(entity);
            }
            continue;
        }

        // Relationship parsing
        if let Some((link, arrow_len, reversed_by_direction)) = parse_link(trimmed, source_line) {
            debug!(
                "link: {} -> {} ({:?}, len={})",
                link.from, link.to, link.line_style, arrow_len
            );
            reserve_entity_order(&mut entity_order, &link.from);
            reserve_entity_order(&mut entity_order, &link.to);
            ensure_entity_uid(
                &mut entity_uids,
                &mut entity_first_source_lines,
                &mut next_uid_value,
                &link.from,
                source_line,
            );
            synthesize_implicit_groups_for_entity(
                &link.from,
                Some(source_line),
                &mut groups,
                &mut known_implicit_groups,
                &mut next_uid_value,
            );
            ensure_entity_uid(
                &mut entity_uids,
                &mut entity_first_source_lines,
                &mut next_uid_value,
                &link.to,
                source_line,
            );
            synthesize_implicit_groups_for_entity(
                &link.to,
                Some(source_line),
                &mut groups,
                &mut known_implicit_groups,
                &mut next_uid_value,
            );
            let mut link = link;
            // Java CommandLinkClass creates the link before applying `getInv()`
            // for `u`/`l` hints, so those reversed links burn one sequence slot.
            if reversed_by_direction {
                let _ = next_link_uid(&mut next_uid_value);
            }
            link.uid = Some(next_link_uid(&mut next_uid_value));
            // Java: first link's arrow length determines rankdir.
            // Single dash/dot (len=1) = horizontal (LR).
            // Double+ dash/dot (len>=2) = vertical (TB).
            if links.is_empty() && direction == Direction::TopToBottom && arrow_len == 1 {
                direction = Direction::LeftToRight;
                debug!("direction inferred from arrow: LeftToRight");
            }
            links.push(link);
            continue;
        }

        // Note parsing: single-line or multi-line start
        if let Some(note_result) = try_parse_class_note(trimmed) {
            match note_result {
                ClassNoteParseResult::SingleLine(mut note) => {
                    // Java: note without "of" target auto-attaches to last entity
                    if note.target.is_none() {
                        note.target = last_entity_name.clone();
                    }
                    debug!("single-line note for {:?}", note.target);
                    notes.push(note);
                }
                ClassNoteParseResult::MultiLineStart { position, target } => {
                    // Java: note without "of" target auto-attaches to last entity
                    let effective_target = target.or_else(|| last_entity_name.clone());
                    debug!("start multi-line note for {effective_target:?}");
                    in_note_block = true;
                    note_block_position = position;
                    note_block_target = effective_target;
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
    auto_create_entities(
        &mut entities,
        &links,
        &mut entity_order,
        &entity_uids,
        &entity_first_source_lines,
    );
    // Synthesize implicit packages for any auto-created entities that weren't
    // handled inline during parsing (e.g. entities only referenced in links).
    for entity in &entities {
        synthesize_implicit_groups_for_entity(
            &entity.name,
            entity.source_line,
            &mut groups,
            &mut known_implicit_groups,
            &mut next_uid_value,
        );
    }
    sort_entities_by_order(&mut entities, &entity_order);

    Ok(ClassDiagram {
        entities,
        links,
        groups,
        direction,
        direction_explicit,
        notes,
        hide_show_rules,
        stereotype_backgrounds,
    })
}

fn reserve_entity_order(entity_order: &mut Vec<String>, name: &str) {
    if !entity_order.iter().any(|existing| existing == name) {
        entity_order.push(name.to_string());
    }
}

fn next_entity_uid(next_uid_value: &mut u32) -> String {
    *next_uid_value += 1;
    format!("ent{:04}", *next_uid_value)
}

fn next_link_uid(next_uid_value: &mut u32) -> String {
    *next_uid_value += 1;
    format!("lnk{}", *next_uid_value)
}

fn ensure_entity_uid(
    entity_uids: &mut HashMap<String, String>,
    entity_first_source_lines: &mut HashMap<String, usize>,
    next_uid_value: &mut u32,
    name: &str,
    source_line: usize,
) -> String {
    entity_first_source_lines
        .entry(name.to_string())
        .or_insert(source_line);
    entity_uids
        .entry(name.to_string())
        .or_insert_with(|| next_entity_uid(next_uid_value))
        .clone()
}

fn sort_entities_by_order(entities: &mut [Entity], entity_order: &[String]) {
    let order_index: HashMap<&str, usize> = entity_order
        .iter()
        .enumerate()
        .map(|(idx, name)| (name.as_str(), idx))
        .collect();
    entities.sort_by_key(|entity| {
        order_index
            .get(entity.name.as_str())
            .copied()
            .unwrap_or(usize::MAX)
    });
}

fn normalize_for_mapping(s: &str) -> String {
    s.replace("%newline()", "\u{E100}")
        .replace("%n()", "\u{E100}")
}

/// Check if two lines match for mapping purposes, tolerating variable
/// substitution differences. Compares exact first, then via normalize,
/// then via prefix before the first `\n` escape.
fn lines_match_for_mapping(block_line: &str, orig_line: &str) -> bool {
    if block_line == orig_line {
        return true;
    }
    if normalize_for_mapping(block_line) == normalize_for_mapping(orig_line) {
        return true;
    }
    let bl_prefix = block_line.split("\\n").next().unwrap_or(block_line);
    let ol_prefix = orig_line.split("\\n").next().unwrap_or(orig_line);
    if !bl_prefix.is_empty() && bl_prefix.len() > 5 && bl_prefix == ol_prefix {
        return true;
    }
    false
}

fn build_line_mapping(
    cleaned_source: &str,
    original_source: Option<&str>,
    block: &str,
) -> Vec<usize> {
    let orig = original_source.unwrap_or(cleaned_source);
    let orig_lines: Vec<&str> = orig.lines().collect();

    let start_pos = orig_lines
        .iter()
        .position(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("@startuml") || trimmed.starts_with("@start")
        })
        .unwrap_or(0);

    let mut mapping = Vec::with_capacity(block.lines().count());
    let mut search_from = start_pos + 1;

    for block_line in block.lines() {
        let trimmed = block_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("skinparam ") {
            mapping.push(start_pos + 1 + mapping.len());
            continue;
        }
        let found_idx = orig_lines[search_from..]
            .iter()
            .position(|ol| lines_match_for_mapping(trimmed, ol.trim()))
            .map(|i| i + search_from);
        if let Some(orig_idx) = found_idx {
            mapping.push(orig_idx);
            search_from = orig_idx + 1;
        } else {
            mapping.push(start_pos + 1 + mapping.len());
        }
    }

    mapping
}

/// Merge continuation lines: a line ending with `\` (backslash at end) joins
/// with the next line, keeping the first original source line number for the
/// combined logical line.
fn merge_continuation_lines_with_mapping(
    content: &str,
    line_mapping: &[usize],
) -> Vec<(String, usize)> {
    let mut result = Vec::new();
    let mut carry = String::new();
    let mut carry_source_line: Option<usize> = None;

    for (idx, line) in content.lines().enumerate() {
        let mapped_source_line = line_mapping.get(idx).copied().unwrap_or(idx + 1);
        if let Some(stripped) = line.strip_suffix('\\') {
            if carry.is_empty() {
                carry_source_line = Some(mapped_source_line);
            }
            carry.push_str(stripped);
            continue;
        }
        if !carry.is_empty() {
            carry.push_str(line);
            result.push((
                std::mem::take(&mut carry),
                carry_source_line.take().unwrap_or(mapped_source_line),
            ));
            continue;
        }
        result.push((line.to_string(), mapped_source_line));
    }

    if !carry.is_empty() {
        result.push((
            carry,
            carry_source_line.unwrap_or(line_mapping.len().saturating_add(1)),
        ));
    }

    result
}

fn should_skip_line(trimmed: &str) -> bool {
    let skip_prefixes = [
        "skinparam",
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

fn parse_hide_show_rule(trimmed: &str) -> Option<Vec<ClassHideShowRule>> {
    let mut parts = trimmed.split_whitespace();
    let command = parts.next()?;
    let show = match command {
        "hide" => false,
        "show" => true,
        _ => return None,
    };

    let rest: Vec<&str> = parts.collect();
    if rest.is_empty() {
        return None;
    }

    if rest.len() == 1 && rest[0] == "stereotype" {
        return Some(vec![ClassHideShowRule {
            target: ClassRuleTarget::Any,
            portion: ClassPortion::Stereotype,
            show,
            empty_only: false,
        }]);
    }

    if rest.len() == 2
        && rest[0].starts_with("<<")
        && rest[0].ends_with(">>")
        && rest[1] == "stereotype"
    {
        return Some(vec![ClassHideShowRule {
            target: ClassRuleTarget::Stereotype(rest[0][2..rest[0].len() - 2].trim().to_string()),
            portion: ClassPortion::Stereotype,
            show,
            empty_only: false,
        }]);
    }

    // "hide empty members" / "hide empty fields" / "hide empty methods"
    if rest.len() == 2 && rest[0] == "empty" {
        match rest[1] {
            "members" => {
                return Some(vec![
                    ClassHideShowRule {
                        target: ClassRuleTarget::Any,
                        portion: ClassPortion::Field,
                        show,
                        empty_only: true,
                    },
                    ClassHideShowRule {
                        target: ClassRuleTarget::Any,
                        portion: ClassPortion::Method,
                        show,
                        empty_only: true,
                    },
                ]);
            }
            "fields" => {
                return Some(vec![ClassHideShowRule {
                    target: ClassRuleTarget::Any,
                    portion: ClassPortion::Field,
                    show,
                    empty_only: true,
                }]);
            }
            "methods" => {
                return Some(vec![ClassHideShowRule {
                    target: ClassRuleTarget::Any,
                    portion: ClassPortion::Method,
                    show,
                    empty_only: true,
                }]);
            }
            _ => {}
        }
    }

    if rest.len() == 1 && rest[0] == "members" {
        return Some(vec![
            ClassHideShowRule {
                target: ClassRuleTarget::Any,
                portion: ClassPortion::Field,
                show,
                empty_only: false,
            },
            ClassHideShowRule {
                target: ClassRuleTarget::Any,
                portion: ClassPortion::Method,
                show,
                empty_only: false,
            },
        ]);
    }

    if rest.len() == 2 {
        let target = ClassRuleTarget::Entity(rest[0].to_string());
        let portion = match rest[1] {
            "fields" => Some(ClassPortion::Field),
            "methods" => Some(ClassPortion::Method),
            "stereotype" => Some(ClassPortion::Stereotype),
            _ => None,
        }?;
        return Some(vec![ClassHideShowRule {
            target,
            portion,
            show,
            empty_only: false,
        }]);
    }

    None
}

fn parse_stereotypes_from_tail(s: &str) -> Vec<Stereotype> {
    let mut result = Vec::new();
    let mut rest = s.trim();
    while let Some(start) = rest.find("<<") {
        let after = &rest[start + 2..];
        let Some(end) = after.find(">>") else {
            break;
        };
        let stereo = after[..end].trim();
        if !stereo.is_empty() {
            result.push(Stereotype(stereo.to_string()));
        }
        rest = &after[end + 2..];
    }
    result
}

fn parse_color_from_tail(s: &str) -> Option<String> {
    s.split_whitespace()
        .find(|token| token.starts_with('#'))
        .map(ToString::to_string)
}

fn parse_style_background_color(trimmed: &str) -> Option<&str> {
    trimmed
        .split_once(' ')
        .filter(|(key, _)| key.eq_ignore_ascii_case("BackGroundColor"))
        .map(|(_, value)| value.trim())
        .filter(|value| !value.is_empty())
}

fn parse_entity_kind(s: &str) -> EntityKind {
    match s {
        "class" | "static class" => EntityKind::Class,
        "interface" => EntityKind::Interface,
        "enum" => EntityKind::Enum,
        "abstract" | "abstract class" => EntityKind::Abstract,
        "annotation" => EntityKind::Annotation,
        "object" => EntityKind::Object,
        "map" => EntityKind::Map,
        "rectangle" | "file" | "folder" | "frame" | "card" | "agent" | "storage" | "artifact"
        | "node" | "cloud" | "stack" | "queue" => EntityKind::Rectangle,
        "component" => EntityKind::Component,
        _ => EntityKind::Class,
    }
}

/// Map a parser keyword to the concrete rectangle symbol variant.
/// Only meaningful when `parse_entity_kind` returned `EntityKind::Rectangle`.
fn parse_rect_symbol(s: &str) -> RectSymbol {
    match s {
        "file" => RectSymbol::File,
        "folder" => RectSymbol::Folder,
        "frame" => RectSymbol::Frame,
        "card" => RectSymbol::Card,
        "agent" => RectSymbol::Agent,
        "storage" => RectSymbol::Storage,
        "artifact" => RectSymbol::Artifact,
        "node" => RectSymbol::Node,
        "cloud" => RectSymbol::Cloud,
        "stack" => RectSymbol::Stack,
        "queue" => RectSymbol::Queue,
        _ => RectSymbol::Rectangle,
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
            } else if after.is_empty() {
                (method_part.trim().to_string(), None)
            } else {
                // Text after closing paren that's not a return type (e.g. ";")
                // is kept as part of the member name for display purposes.
                let full = format!("{}{}", method_part, &rest[paren_close + 1..]);
                (full.trim().to_string(), None)
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
        display: Some(rest.to_string()),
    })
}

/// Parse a relationship/link line.
///
/// Arrow patterns:
///   left_head + line + right_head
///   left heads: `<|`, `<`, `*`, `o`, `+`, or none
///   line: `--` (solid) or `..` (dashed), with optional direction hint letters
///   right heads: `|>`, `>`, `*`, `o`, `+`, or none
/// Returns (Link, arrow_length, reversed_by_direction) where arrow_length is
/// the number of dashes/dots. `reversed_by_direction` matches Java's `getInv()`
/// path, which consumes an additional link UID.
/// length=1 means horizontal (LR), length>=2 means vertical (TB).
fn parse_link(line: &str, source_line: usize) -> Option<(Link, usize, bool)> {
    // Left heads: <|, <, *, o, +, or nothing
    // Line: --..variations with optional direction letters
    // Right heads: |>, >, *, o, +, or nothing
    let re = Regex::new(concat!(
        r"(?x)",
        r#"^((?:"[^"]+"|[\w.]+))"#, // from entity (quoted or simple)
        r"\s*",
        r"(?:\[([^\]]*)\])?", // optional from qualifier [...]
        r"\s*",
        r#"(?:"([^"]*)"\s*)?"#, // optional from-label "..."
        r"(",                   // arrow group start
        r"(?:<\||\*|o|\+|<)?",  // optional left head
        r"(?:-+[udlr]*-*|\.+[udlr]*\.*)",
        r"(?:\|>|>|\*|o|\+)?", // optional right head
        r")",                  // arrow group end
        r"\s*",
        r#"(?:"([^"]*)"\s*)?"#, // optional to-label "..."
        r"(?:\[([^\]]*)\])?",   // optional to qualifier [...]
        r"\s*",
        r#"((?:"[^"]+"|[\w.]+))"#, // to entity (quoted or simple)
        r"\s*",
        r"(?:\s*:\s*(.*))?", // optional label
        r"$",
    ))
    .unwrap();

    let trimmed = line.trim();
    let caps = re.captures(trimmed)?;

    let mut from = caps.get(1).unwrap().as_str().trim_matches('"').to_string();
    let mut from_qualifier = caps
        .get(2)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty());
    let mut from_label = caps
        .get(3)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty());
    let arrow = caps.get(4).unwrap().as_str();
    let mut to_label = caps
        .get(5)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty());
    let mut to_qualifier = caps
        .get(6)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty());
    let mut to = caps.get(7).unwrap().as_str().trim_matches('"').to_string();
    let label = caps
        .get(8)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty());

    let (mut left_head, line_style, mut right_head) = parse_arrow(arrow);
    // Arrow length: count dashes/dots. 1=horizontal(LR), 2+=vertical(TB).
    let arrow_len = compute_arrow_len(arrow);

    let reversed_by_direction = arrow_direction_requires_reverse(arrow);
    if reversed_by_direction {
        std::mem::swap(&mut from, &mut to);
        std::mem::swap(&mut from_qualifier, &mut to_qualifier);
        std::mem::swap(&mut from_label, &mut to_label);
        std::mem::swap(&mut left_head, &mut right_head);
    }

    Some((
        Link {
            uid: None,
            from,
            to,
            left_head,
            right_head,
            line_style,
            label,
            from_label,
            to_label,
            from_qualifier,
            to_qualifier,
            source_line: Some(source_line),
            arrow_len,
        },
        arrow_len,
        reversed_by_direction,
    ))
}

fn arrow_direction_requires_reverse(arrow: &str) -> bool {
    arrow.contains('u') || arrow.contains('l')
}

fn compute_arrow_len(arrow: &str) -> usize {
    if arrow.contains('l') || arrow.contains('r') {
        1
    } else {
        arrow.chars().filter(|c| *c == '-' || *c == '.').count()
    }
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
                let text = after_of[colon_pos + 1..]
                    .trim()
                    .replace("\\n", "\n")
                    .replace(crate::NEWLINE_CHAR, "\n");
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
            let text = after_colon
                .trim()
                .replace("\\n", "\n")
                .replace(crate::NEWLINE_CHAR, "\n");
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
fn auto_create_entities(
    entities: &mut Vec<Entity>,
    links: &[Link],
    entity_order: &mut Vec<String>,
    entity_uids: &HashMap<String, String>,
    entity_first_source_lines: &HashMap<String, usize>,
) {
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
        reserve_entity_order(entity_order, &name);
        entities.push(Entity {
            uid: entity_uids.get(&name).cloned(),
            name: name.clone(),
            kind: EntityKind::Class,
            stereotypes: Vec::new(),
            members: Vec::new(),
            description: Vec::new(),
            color: None,
            generic: None,
            source_line: entity_first_source_lines.get(&name).copied(),
            visibility: None,
            display_name: None,
            map_entries: vec![],
            rect_symbol: RectSymbol::Rectangle,
        });
    }
}

fn implicit_name_head(name: &str) -> &str {
    let mut end = name.len();
    for needle in ["\\r", "\\n"] {
        if let Some(pos) = name.find(needle) {
            end = end.min(pos);
        }
    }
    if let Some(pos) = name.find(crate::NEWLINE_CHAR) {
        end = end.min(pos);
    }
    if let Some(pos) = name.find('\r') {
        end = end.min(pos);
    }
    if let Some(pos) = name.find('\n') {
        end = end.min(pos);
    }
    &name[..end]
}

fn implicit_package_prefixes(name: &str) -> Vec<String> {
    let head = implicit_name_head(name);
    let Some(last_dot) = head.rfind('.') else {
        return Vec::new();
    };
    let package_path = &head[..last_dot];
    if package_path.is_empty() {
        return Vec::new();
    }
    let mut prefixes = Vec::new();
    let mut current = String::new();
    for segment in package_path.split('.') {
        if segment.is_empty() {
            break;
        }
        if !current.is_empty() {
            current.push('.');
        }
        current.push_str(segment);
        prefixes.push(current.clone());
    }
    prefixes
}

/// Synthesize implicit package groups for a single entity name.
/// Java creates package hierarchy immediately when an entity with dots is registered,
/// so group UIDs are allocated from the shared counter before any subsequent link UIDs.
fn synthesize_implicit_groups_for_entity(
    entity_name: &str,
    source_line: Option<usize>,
    groups: &mut Vec<Group>,
    known_implicit_groups: &mut std::collections::HashSet<String>,
    next_uid_value: &mut u32,
) {
    let prefixes = implicit_package_prefixes(entity_name);
    if prefixes.is_empty() {
        return;
    }
    for prefix in &prefixes {
        if known_implicit_groups.insert(prefix.clone()) {
            groups.push(Group {
                uid: Some(next_entity_uid(next_uid_value)),
                kind: GroupKind::Package,
                name: prefix.clone(),
                entities: Vec::new(),
                stereotypes: Vec::new(),
                color: None,
                source_line,
            });
        }
    }
    if let Some(deepest) = prefixes.last() {
        if let Some(group) = groups.iter_mut().find(|g| g.name == *deepest) {
            if !group.entities.iter().any(|name| name == entity_name) {
                group.entities.push(entity_name.to_string());
            }
            if group.source_line.is_none() {
                group.source_line = source_line;
            }
        }
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

    #[test]
    fn specific_show_rule_reserves_entity_order() {
        let cd = parse(
            "hide members\nshow B methods\nclass A\nclass B {\n  ~run(): boolean\n}\nclass C\nclass D",
        );
        let names: Vec<&str> = cd
            .entities
            .iter()
            .map(|entity| entity.name.as_str())
            .collect();
        assert_eq!(names, vec!["B", "A", "C", "D"]);
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

    #[test]
    fn parse_association_with_side_labels() {
        let cd = parse(r#"A "many" --> "one" B"#);
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.from_label.as_deref(), Some("many"));
        assert_eq!(link.to_label.as_deref(), Some("one"));
    }

    #[test]
    fn parse_association_with_qualifiers() {
        let cd = parse(r#"Shop [customerId: long] ---> "customer\n1" Customer"#);
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.from_qualifier.as_deref(), Some("customerId: long"));
        assert_eq!(link.to_label.as_deref(), Some(r#"customer\n1"#));
        assert_eq!(link.to_qualifier, None);
    }

    #[test]
    fn parse_association_with_two_qualifiers() {
        let cd = parse(r#"B [key2a] o-- [key2b] C : holds >"#);
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.from_qualifier.as_deref(), Some("key2a"));
        assert_eq!(link.to_qualifier.as_deref(), Some("key2b"));
        assert_eq!(link.label.as_deref(), Some("holds >"));
    }

    #[test]
    fn parse_single_dash_association_with_label() {
        let cd = parse(r#"A [key1] *- "1" B : holds >"#);
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.arrow_len, 1);
        assert_eq!(link.from_qualifier.as_deref(), Some("key1"));
        assert_eq!(link.to_label.as_deref(), Some("1"));
        assert_eq!(link.label.as_deref(), Some("holds >"));
    }

    #[test]
    fn parse_direction_hint_lr_forces_horizontal_queue() {
        let cd = parse(r#"HashMap [b2] *.r.> [f] V2"#);
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.arrow_len, 1);
        assert_eq!(link.from_qualifier.as_deref(), Some("b2"));
        assert_eq!(link.to_qualifier.as_deref(), Some("f"));
    }

    #[test]
    fn parse_direction_hint_up_reverses_endpoints() {
        let cd = parse(r#"HashMap [a1] <|-u-> [e] V1"#);
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.from, "V1");
        assert_eq!(link.to, "HashMap");
        assert_eq!(link.from_qualifier.as_deref(), Some("e"));
        assert_eq!(link.to_qualifier.as_deref(), Some("a1"));
        assert_eq!(link.left_head, ArrowHead::Arrow);
        assert_eq!(link.right_head, ArrowHead::Triangle);
    }

    #[test]
    fn parse_direction_hint_left_reverses_endpoints() {
        let cd = parse(r#"HashMap [d4] +-l-> [h] V4"#);
        assert_eq!(cd.links.len(), 1);
        let link = &cd.links[0];
        assert_eq!(link.from, "V4");
        assert_eq!(link.to, "HashMap");
        assert_eq!(link.from_qualifier.as_deref(), Some("h"));
        assert_eq!(link.to_qualifier.as_deref(), Some("d4"));
        assert_eq!(link.left_head, ArrowHead::Arrow);
        assert_eq!(link.right_head, ArrowHead::Plus);
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

    #[test]
    fn implicit_entities_follow_java_shared_uid_sequence() {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/class/qualifiedassoc002.puml"
        ))
        .unwrap();
        let cd = parse_class_diagram(&src).unwrap();

        let entity_uids: HashMap<_, _> = cd
            .entities
            .iter()
            .map(|entity| (entity.name.as_str(), entity.uid.as_deref()))
            .collect();
        assert_eq!(entity_uids.get("Map"), Some(&Some("ent0002")));
        assert_eq!(entity_uids.get("HashMap"), Some(&Some("ent0003")));
        assert_eq!(entity_uids.get("Shop"), Some(&Some("ent0005")));
        assert_eq!(entity_uids.get("Customer"), Some(&Some("ent0006")));

        let shop = cd
            .entities
            .iter()
            .find(|entity| entity.name == "Shop")
            .unwrap();
        let customer = cd
            .entities
            .iter()
            .find(|entity| entity.name == "Customer")
            .unwrap();
        assert_eq!(shop.source_line, Some(5));
        assert_eq!(customer.source_line, Some(5));

        let link_uids: Vec<_> = cd.links.iter().map(|link| link.uid.as_deref()).collect();
        assert_eq!(link_uids, vec![Some("lnk4"), Some("lnk7"), Some("lnk8")]);
    }

    #[test]
    fn explicit_entity_after_link_keeps_implicit_uid_and_source_line() {
        let cd = parse("A --> B\nclass A");
        let a = cd
            .entities
            .iter()
            .find(|entity| entity.name == "A")
            .unwrap();
        let b = cd
            .entities
            .iter()
            .find(|entity| entity.name == "B")
            .unwrap();
        assert_eq!(a.uid.as_deref(), Some("ent0002"));
        assert_eq!(b.uid.as_deref(), Some("ent0003"));
        assert_eq!(a.source_line, Some(1));
        assert_eq!(b.source_line, Some(1));
        assert_eq!(cd.links[0].uid.as_deref(), Some("lnk4"));
    }

    #[test]
    fn reversed_direction_link_burns_intermediate_uid_like_java_get_inv() {
        let cd = parse("class HashMap\nHashMap [a1] <|-u-> [e] V1\nHashMap [b2] *.r.> [f] V2");
        let v1 = cd
            .entities
            .iter()
            .find(|entity| entity.name == "V1")
            .unwrap();
        let v2 = cd
            .entities
            .iter()
            .find(|entity| entity.name == "V2")
            .unwrap();
        assert_eq!(v1.uid.as_deref(), Some("ent0003"));
        assert_eq!(cd.links[0].uid.as_deref(), Some("lnk5"));
        assert_eq!(v2.uid.as_deref(), Some("ent0006"));
        assert_eq!(cd.links[1].uid.as_deref(), Some("lnk7"));
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

    #[test]
    fn parse_fixture_jaws3_uses_original_source_line_for_rectangle() {
        let original = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/dev/jaws/jaws3.puml"
        ))
        .unwrap();
        let expanded = crate::preproc::preprocess(&original).unwrap();
        let cd = parse_class_diagram_with_original(&expanded, Some(&original)).unwrap();
        let rectangle = cd
            .entities
            .iter()
            .find(|entity| entity.name == "r")
            .unwrap();
        assert_eq!(rectangle.source_line, Some(9));
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

    #[test]
    fn parse_link_with_quoted_entity_names() {
        let src = concat!(
            "@startuml\n",
            "class \"A name with spaces\"\n",
            "\"A name with spaces\" --> \"A name with spaces\" : Hello\n",
            "@enduml\n",
        );
        let cd = parse_class_diagram(src).unwrap();
        assert_eq!(cd.entities.len(), 1);
        assert_eq!(cd.links.len(), 1);
        assert_eq!(cd.links[0].from, "A name with spaces");
        assert_eq!(cd.links[0].to, "A name with spaces");
        assert_eq!(cd.links[0].label.as_deref(), Some("Hello"));
    }

    #[test]
    fn parse_implicit_package_groups_from_qualified_name() {
        let src = concat!(
            "@startuml\n",
            "class \"pkg1.pkg2.Class 1\\r\\n\\tBody\"\n",
            "@enduml\n",
        );
        let cd = parse_class_diagram(src).unwrap();
        assert_eq!(cd.entities.len(), 1);
        assert!(cd.groups.iter().any(|g| g.name == "pkg1"));
        assert!(cd.groups.iter().any(|g| g.name == "pkg1.pkg2"));
        let inner = cd.groups.iter().find(|g| g.name == "pkg1.pkg2").unwrap();
        assert_eq!(
            inner.entities,
            vec!["pkg1.pkg2.Class 1\\r\\n\\tBody".to_string()]
        );
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
