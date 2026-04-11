use log::debug;

use super::SkinParams;

/// Check if a trimmed line opens an embedded `{{ }}` diagram block.
fn is_embedded_open(trimmed: &str) -> bool {
    if !trimmed.starts_with("{{") {
        return false;
    }
    matches!(
        trimmed,
        "{{" | "{{ditaa"
            | "{{salt"
            | "{{uml"
            | "{{wbs"
            | "{{mindmap"
            | "{{gantt"
            | "{{json"
            | "{{yaml"
            | "{{wire"
            | "{{creole"
            | "{{board"
            | "{{ebnf"
            | "{{regex"
            | "{{files"
            | "{{chronology"
            | "{{chen"
            | "{{chart"
            | "{{nwdiag"
            | "{{packetdiag"
    )
}

/// Normalize the content before line-based skinparam parsing.
///
/// 1. Replace the private-use `%newline()` / `%n()` marker (U+E100) with a
///    real newline. The C4 stdlib preprocessor uses this marker to join
///    multi-line helper output onto a single physical line, so the skinparam
///    tokenizer must unwrap them before splitting by whitespace.
/// 2. Insert a newline between any `}` and a subsequent `skinparam` keyword so
///    that chained inline blocks (as emitted by the C4 stdlib preprocessor)
///    are handled by the line-based parser below.
///
/// Example input chunk (C4 stdlib output, single line):
///   `skinparam rectangle<<container>> { ... }skinparam database<<container>> { ... }`
/// After normalization:
///   `skinparam rectangle<<container>> { ... }`
///   `skinparam database<<container>> { ... }`
fn split_chained_skinparams(content: &str) -> String {
    // Fast path: no chained occurrences and no private-use marker.
    let has_chained = content.contains("}skinparam") || content.contains("} skinparam");
    let has_pnl = content.contains(crate::NEWLINE_CHAR);
    if !has_chained && !has_pnl {
        return content.to_string();
    }
    // Replace the private-use newline marker with a real newline first so the
    // chained-skinparam split (and the downstream line-based parser) can
    // consume them uniformly.
    let stage1 = if has_pnl {
        content.replace(crate::NEWLINE_CHAR, "\n")
    } else {
        content.to_string()
    };
    if !stage1.contains("}skinparam") && !stage1.contains("} skinparam") {
        return stage1;
    }
    // Byte-level walk is safe here because we only look at ASCII markers
    // (`}`, space, tab, `skinparam`), and UTF-8 multi-byte sequences never
    // contain ASCII bytes in their continuation bytes. We still copy the
    // full run verbatim via `push_str` to preserve any multi-byte content.
    let mut out = String::with_capacity(stage1.len() + 64);
    let bytes = stage1.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'}' {
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if bytes[j..].starts_with(b"skinparam") {
                // Copy [start, i] (inclusive of the '}') then insert a newline
                // and resume at the 'skinparam' keyword.
                out.push_str(&stage1[start..=i]);
                out.push('\n');
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    if start < bytes.len() {
        out.push_str(&stage1[start..]);
    }
    out
}

/// Parse skinparam declarations from PlantUML source text.
///
/// Supports:
/// - Single line: `skinparam BackgroundColor #FEFECE`
/// - Block: `skinparam component { BackgroundColor #FEFECE }`
/// - Nested: `skinparam { component { BackgroundColor #FEFECE } }`
pub fn parse_skinparams(content: &str) -> SkinParams {
    let mut params = SkinParams::new();
    // Pre-normalize: the C4 stdlib preprocessor emits multiple chained inline
    // `skinparam element<<stereo>> { ... }skinparam ...` blocks on the same
    // line. Split such chained occurrences into individual lines so the
    // line-based parser below can handle them. We only split between `}` and
    // a subsequent `skinparam` keyword, which is safe because:
    //   - A closing `}` terminates a block.
    //   - The `skinparam` keyword always starts a new statement.
    let normalized = split_chained_skinparams(content);
    let mut lines = normalized.lines().peekable();
    let mut in_style_block = false;
    let mut style_content = String::new();
    let mut embedded_depth: usize = 0;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Skip lines inside `{{ }}` embedded diagram blocks.
        // Java: PSystemCommandFactory.addOneSingleLineManageEmbedded2 skips these;
        // the embedded content has its own skinparams that should not affect the parent.
        if is_embedded_open(trimmed) {
            embedded_depth += 1;
            continue;
        }
        if embedded_depth > 0 {
            if trimmed == "}}" {
                embedded_depth -= 1;
            }
            continue;
        }

        // Collect <style> blocks for post-processing
        if trimmed.starts_with("<style>") {
            in_style_block = true;
            continue;
        }
        if in_style_block {
            if trimmed.starts_with("</style>") {
                in_style_block = false;
            } else {
                style_content.push_str(trimmed);
                style_content.push('\n');
            }
            continue;
        }

        let lower = trimmed.to_lowercase();

        // Handle `skin rose` directive: apply Rose theme defaults.
        // Java Rose skin uses legacy ColorParam defaults (MY_RED=#A80036,
        // MY_YELLOW=#FEFECE, COL_FBFB77 for notes). This completely replaces
        // the modern default theme colors.
        if let Some(stripped) = lower.strip_prefix("skin ") {
            let skin_name = stripped.trim();
            if skin_name == "rose" {
                // Border colors (Java: MY_RED = #A80036)
                params.set("sequencelifelinebordercolor", "#A80036");
                params.set("participant.bordercolor", "#A80036");
                params.set("participantbordercolor", "#A80036");
                params.set("sequence.bordercolor", "#A80036");
                params.set("notebordercolor", "#A80036");
                params.set("sequencegroupbordercolor", "#A80036");
                // Background colors (Java: MY_YELLOW = #FEFECE)
                params.set("participantbackgroundcolor", "#FEFECE");
                params.set("participant.backgroundcolor", "#FEFECE");
                // Note background (Java: COL_FBFB77)
                params.set("notebackgroundcolor", "#FBFB77");
                // Line thickness (Java: Rose skin default)
                params.set("root.linethickness", "1");
                // Participant stroke-width 1.5 (Java: UStroke(1.5) for participant)
                params.set("participant.linethickness", "1.5");
                // Flag for rendering: no rounded corners, different box layout
                params.set("_skin_rose", "true");
            }
            continue;
        }

        if !lower.starts_with("skinparam") {
            continue;
        }

        // Remove the "skinparam" prefix
        let after = trimmed[9..].trim();

        if after.is_empty() {
            // Bare "skinparam" on its own line - not valid, skip
            continue;
        }

        if after.starts_with('{') {
            // Nested block: skinparam { ... }
            // Content inside can be either:
            //   - key value pairs (global)
            //   - element { key value } blocks
            parse_nested_block(&mut lines, "", &mut params);
        } else if after.contains('{') {
            // Element block: skinparam element { ... }
            // Extract element name (everything before '{')
            let brace_pos = after.find('{').unwrap();
            let element = after[..brace_pos].trim();
            let after_brace = after[brace_pos + 1..].trim();

            // Check if the closing brace is on the same line
            if let Some(close_pos) = after_brace.find('}') {
                // Inline block: skinparam element { key val }
                let inner = after_brace[..close_pos].trim();
                parse_inline_pairs(inner, element, &mut params);
            } else {
                // Multi-line block
                if !after_brace.is_empty() {
                    // There may be a key-value pair on the same line as the opening brace
                    parse_single_pair(after_brace, element, &mut params);
                }
                parse_element_block(&mut lines, element, &mut params);
            }
        } else {
            // Single line: skinparam key value
            // Could be "skinparam elementKey value" or "skinparam element.Key value"
            parse_single_pair(after, "", &mut params);
        }
    }

    // Extract document-level styles from <style> CSS blocks.
    // Java: `document { BackGroundColor orange }` sets the SVG background.
    if !style_content.is_empty() {
        extract_document_style(&style_content, &mut params);
    }

    debug!("parsed {} skinparams", params.len());
    params
}

/// Extract document-level CSS properties from `<style>` content.
/// Supports `document { BackGroundColor orange }` and nested sub-blocks like
/// `document { title { BackGroundColor yellow } footer { FontColor red } }`.
/// Parse a CSS-style property line into (key, value).
/// Handles both `PropertyName value` and `PropertyName: value;` syntax.
fn parse_css_property(line: &str) -> Option<(String, String)> {
    // Try splitting on colon first (CSS syntax: `key: value;`)
    if let Some(colon_pos) = line.find(':') {
        let key = line[..colon_pos].trim().to_lowercase();
        let value = line[colon_pos + 1..]
            .trim()
            .trim_end_matches(';')
            .trim()
            .to_string();
        if !key.is_empty() && !value.is_empty() {
            return Some((key, value));
        }
    }
    // Fall back to whitespace splitting (PlantUML syntax: `PropertyName value`)
    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    if parts.len() == 2 {
        let key = parts[0].trim().to_lowercase();
        let value = parts[1].trim().trim_end_matches(';').trim().to_string();
        if !key.is_empty() && !value.is_empty() {
            return Some((key, value));
        }
    }
    None
}

fn extract_document_style(css: &str, params: &mut SkinParams) {
    let mut depth = 0;
    let mut in_document = false;
    let mut doc_depth = 0;
    // Track which sub-block we're inside (e.g., "title", "footer", "header", "legend", "caption")
    let mut current_section: Option<String> = None;
    let mut section_depth = 0;
    // Diagram-type wrappers (e.g., `sequenceDiagram { ... }`) are transparent:
    // inner blocks like `participant { ... }` are treated as top-level sections.
    let mut wrapper_depth: usize = 0;

    for line in css.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let opens = trimmed.matches('{').count();
        let closes = trimmed.matches('}').count();

        if !in_document {
            let lower = trimmed.to_lowercase();
            if lower.starts_with("document") && trimmed.contains('{') {
                in_document = true;
                doc_depth = depth;
                depth += opens;
                depth = depth.saturating_sub(closes);
                continue;
            }

            // Diagram-type wrapper blocks — transparent containers for element blocks.
            // `sequenceDiagram { participant { ... } }` => `participant { ... }`
            if current_section.is_none()
                && wrapper_depth == 0
                && depth == 0
                && trimmed.contains('{')
            {
                let brace_pos = trimmed.find('{').unwrap();
                let name = trimmed[..brace_pos].trim().to_lowercase();
                if matches!(
                    name.as_str(),
                    "sequencediagram"
                        | "classdiagram"
                        | "activitydiagram"
                        | "statediagram"
                        | "componentdiagram"
                        | "usecasediagram"
                ) {
                    wrapper_depth = 1;
                    depth += opens;
                    depth = depth.saturating_sub(closes);
                    continue;
                }
            }
            // Track wrapper closing brace
            if wrapper_depth > 0 && current_section.is_none() && trimmed == "}" {
                wrapper_depth = 0;
                depth = depth.saturating_sub(closes);
                continue;
            }

            // Top-level section blocks — not inside document {}
            // Also matches blocks inside a transparent diagram-type wrapper.
            let effective_depth = if wrapper_depth > 0 {
                depth.saturating_sub(wrapper_depth)
            } else {
                depth
            };
            if current_section.is_none() && effective_depth == 0 && trimmed.contains('{') {
                let brace_pos = trimmed.find('{').unwrap();
                let name = trimmed[..brace_pos].trim().to_lowercase();
                if matches!(
                    name.as_str(),
                    "title" | "footer" | "header" | "legend" | "caption"
                ) {
                    current_section = Some(name);
                    section_depth = depth;
                    depth += opens;
                    depth = depth.saturating_sub(closes);
                    continue;
                }
                // Element-level blocks (node, root, etc.) — store under "element.xxx"
                if matches!(
                    name.as_str(),
                    "node"
                        | "root"
                        | "arrow"
                        | "group"
                        | "separator"
                        | "mindmapdiagram"
                        | "wbsdiagram"
                        | "element"
                        | "component"
                        | "participant"
                        | "actor"
                        | "boundary"
                        | "control"
                        | "entity"
                        | "database"
                        | "collections"
                        | "queue"
                        | "note"
                        | "package"
                        | "rectangle"
                        | "card"
                        | "cloud"
                        | "frame"
                        | "folder"
                        | "interface"
                        | "abstract"
                        | "class"
                        | "enum"
                        | "state"
                        | "usecase"
                        | "activity"
                        | "diamond"
                ) {
                    current_section = Some(name);
                    section_depth = depth;
                    depth += opens;
                    depth = depth.saturating_sub(closes);
                    continue;
                }
            }
        }

        // Extract properties from top-level section blocks (not inside document).
        // Only pick up direct properties (depth == section_depth + 1), not nested
        // sub-selectors like `.highlight { BackGroundColor ... }`.
        if !in_document
            && depth == section_depth + 1
            && !trimmed.contains('{')
            && !trimmed.starts_with('}')
        {
            if let (Some(section), Some((key, value))) =
                (current_section.as_ref(), parse_css_property(trimmed))
            {
                // Document sub-sections (title, footer, etc.) use document.{section}.{key}
                // Element-level blocks (node, root, etc.) use {section}.{key}
                let param_key = if matches!(
                    section.as_str(),
                    "title" | "footer" | "header" | "legend" | "caption"
                ) {
                    format!("document.{section}.{key}")
                } else {
                    format!("{section}.{key}")
                };
                params.set(&param_key, &value);
                // Also store concatenated form for legacy lookups
                // (e.g., "participant.fontsize" => also "participantfontsize")
                if !matches!(
                    section.as_str(),
                    "title" | "footer" | "header" | "legend" | "caption"
                ) {
                    let legacy_key = format!("{section}{key}");
                    params.set(&legacy_key, &value);
                }
                log::debug!("extracted style {param_key}: {value}");
            }
        }

        if in_document && depth > doc_depth {
            // Check for sub-block opening (title {, footer {, etc.)
            if current_section.is_none() && depth == doc_depth + 1 && trimmed.contains('{') {
                let brace_pos = trimmed.find('{').unwrap();
                let name = trimmed[..brace_pos].trim().to_lowercase();
                if matches!(
                    name.as_str(),
                    "title" | "footer" | "header" | "legend" | "caption"
                ) {
                    current_section = Some(name);
                    section_depth = depth;
                    depth += opens;
                    depth = depth.saturating_sub(closes);
                    continue;
                }
            }

            // Inside a sub-block: extract properties
            if let Some(ref section) = current_section {
                if depth > section_depth && !trimmed.contains('{') && !trimmed.starts_with('}') {
                    if let Some((key, value)) = parse_css_property(trimmed) {
                        let param_key = format!("document.{section}.{key}");
                        params.set(&param_key, &value);
                        log::debug!("extracted document.{section}.{key}: {value}");
                    }
                }
            }

            // Direct document-level properties (not in sub-block)
            if current_section.is_none()
                && depth == doc_depth + 1
                && !trimmed.contains('{')
                && !trimmed.starts_with('}')
            {
                if let Some((key, value)) = parse_css_property(trimmed) {
                    if key == "backgroundcolor" {
                        // Store under document-specific key so it doesn't override
                        // entity fill colors via the generic fallback chain.
                        params.set("document.backgroundcolor", &value);
                        log::debug!("extracted document BackGroundColor: {value}");
                    }
                }
            }
        }

        depth += opens;
        depth = depth.saturating_sub(closes);

        // Check if we're closing the current section
        if current_section.is_some() && depth <= section_depth + 1 {
            // Count closes that happen on this line after the section depth
            if closes > 0 && depth <= section_depth + 1 {
                current_section = None;
            }
        }

        if in_document && depth <= doc_depth {
            in_document = false;
            current_section = None;
        }
    }
}

/// Parse a nested skinparam block (after `skinparam {`).
/// Handles both global key-value pairs and element sub-blocks.
fn parse_nested_block<'a, I>(lines: &mut I, _prefix: &str, params: &mut SkinParams)
where
    I: Iterator<Item = &'a str>,
{
    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed == "}" {
            return;
        }

        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        if trimmed.contains('{') {
            let brace_pos = trimmed.find('{').unwrap();
            let element = trimmed[..brace_pos].trim();
            let after_brace = trimmed[brace_pos + 1..].trim();

            if let Some(close_pos) = after_brace.find('}') {
                let inner = after_brace[..close_pos].trim();
                parse_inline_pairs(inner, element, params);
            } else {
                if !after_brace.is_empty() {
                    parse_single_pair(after_brace, element, params);
                }
                parse_element_block(lines, element, params);
            }
        } else {
            parse_single_pair(trimmed, "", params);
        }
    }
}

/// Parse a multi-line element block (lines after `skinparam element {`).
fn parse_element_block<'a, I>(lines: &mut I, element: &str, params: &mut SkinParams)
where
    I: Iterator<Item = &'a str>,
{
    for line in lines.by_ref() {
        let trimmed = line.trim();

        if trimmed == "}" || trimmed.starts_with('}') {
            return;
        }

        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        parse_single_pair(trimmed, element, params);
    }
}

/// Parse space-separated key-value pairs from an inline block.
fn parse_inline_pairs(content: &str, element: &str, params: &mut SkinParams) {
    // Simple approach: split by whitespace, take pairs
    let tokens: Vec<&str> = content.split_whitespace().collect();
    let mut i = 0;
    while i + 1 < tokens.len() {
        let key = tokens[i];
        let value = tokens[i + 1];
        let full_key = if element.is_empty() {
            key.to_string()
        } else {
            format!("{element}.{key}")
        };
        debug!("skinparam: {full_key} = {value}");
        params.set(&full_key, value);
        i += 2;
    }
}

/// Parse a single "key value" pair line with an optional element prefix.
fn parse_single_pair(content: &str, element: &str, params: &mut SkinParams) {
    let parts: Vec<&str> = content.splitn(2, char::is_whitespace).collect();
    if parts.len() == 2 {
        let key = parts[0].trim();
        let value = parts[1].trim();
        let full_key = if element.is_empty() {
            key.to_string()
        } else {
            format!("{element}.{key}")
        };
        debug!("skinparam: {full_key} = {value}");
        params.set(&full_key, value);
    }
}
