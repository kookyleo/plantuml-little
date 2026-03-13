use std::collections::HashMap;

use super::DiagramHint;
use crate::model::DiagramMeta;

/// Detect special @start tags and return the determined diagram type
pub fn detect_start_tag(source: &str) -> Option<DiagramHint> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("@startchen") {
            return Some(DiagramHint::Erd);
        }
        if trimmed.starts_with("@startgantt") {
            return Some(DiagramHint::Gantt);
        }
        if trimmed.starts_with("@startditaa") {
            return Some(DiagramHint::Ditaa);
        }
        if trimmed.starts_with("@startjson") {
            return Some(DiagramHint::Json);
        }
        if trimmed.starts_with("@startmindmap") {
            return Some(DiagramHint::Mindmap);
        }
        if trimmed.starts_with("@startnwdiag") {
            return Some(DiagramHint::Nwdiag);
        }
        if trimmed.starts_with("@startsalt") {
            return Some(DiagramHint::Salt);
        }
        if trimmed.starts_with("@startwbs") {
            return Some(DiagramHint::Wbs);
        }
        if trimmed.starts_with("@startyaml") {
            return Some(DiagramHint::Yaml);
        }
        if trimmed.starts_with("@startdot") {
            return Some(DiagramHint::Dot);
        }
        if trimmed.starts_with("@start") {
            return None;
        }
    }
    None
}

/// Extract the content within @startuml/@enduml block from PlantUML text
pub fn extract_block(source: &str) -> Option<String> {
    let mut inside = false;
    let mut lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if inside {
            if trimmed.starts_with("@end") {
                break;
            }
            lines.push(line);
        } else {
            if trimmed.starts_with("@startuml")
                || trimmed.starts_with("@startchen")
                || trimmed.starts_with("@startgantt")
                || trimmed.starts_with("@startditaa")
                || trimmed.starts_with("@startjson")
                || trimmed.starts_with("@startmindmap")
                || trimmed.starts_with("@startnwdiag")
                || trimmed.starts_with("@startsalt")
                || trimmed.starts_with("@startwbs")
                || trimmed.starts_with("@startyaml")
                || trimmed.starts_with("@startdot")
            {
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

/// Detect diagram type (heuristic detection for @startuml)
pub fn detect_diagram_type(content: &str) -> DiagramHint {
    let class_keywords = [
        "class ",
        "interface ",
        "abstract ",
        "enum ",
        "extends ",
        "implements ",
        "object ",
    ];

    // Keywords that unambiguously identify a sequence diagram.
    // Note: "database ", "queue " are also valid deployment keywords;
    // "actor " is also a valid use-case keyword.  These are handled as
    // ambiguous keywords below, not as definitive sequence markers.
    let sequence_keywords_definitive = ["participant ", "boundary ", "control ", "collections "];
    // Ambiguous: sequence diagrams use these, but so do other diagram types.
    let sequence_keywords_ambiguous = ["database ", "queue "];
    // "actor " is shared between sequence and use-case diagrams
    let mut has_seq_actor = false;

    // Sequence fragment keywords — unambiguously identify sequence diagrams
    let seq_fragment_keywords = [
        "alt ",
        "else ",
        "loop ",
        "opt ",
        "par ",
        "break",
        "critical",
        "ref over ",
    ];

    let mut has_activity_action = false;
    let mut has_activity_start_stop = false; // "start" or "stop" (not bare "end")
    let mut has_activity_swimlane = false;
    let mut has_state_keyword = false;
    let mut has_component_keyword = false;
    let mut has_usecase_keyword = false;
    let mut has_salt_keyword = false;
    let mut has_timing_keyword = false;
    let mut has_arrow = false;
    let mut has_seq_arrow = false; // "A -> B" or "A -> B : label" pattern
    let mut has_seq_fragment = false;
    let mut has_class_kw = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('\'') {
            continue;
        }

        if trimmed.starts_with(':') && trimmed.ends_with(';') {
            has_activity_action = true;
        }
        if matches!(trimmed, "start" | "stop") {
            has_activity_start_stop = true;
        }
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2 {
            has_activity_swimlane = true;
        }

        // Detect sequence fragment keywords
        for kw in &seq_fragment_keywords {
            if trimmed.starts_with(kw) || trimmed == kw.trim() {
                has_seq_fragment = true;
            }
        }

        if trimmed.starts_with("state ") {
            has_state_keyword = true;
        }
        if trimmed.contains("[*]") {
            has_state_keyword = true;
        }

        if trimmed.starts_with("component ")
            || trimmed.starts_with("node ")
            || trimmed.starts_with("cloud ")
            || trimmed.starts_with("rectangle ")
            || trimmed.starts_with("database ")
            || trimmed.starts_with("package ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("card ")
            || trimmed.starts_with("file ")
            || trimmed.starts_with("artifact ")
            || trimmed.starts_with("storage ")
            || trimmed.starts_with("folder ")
            || trimmed.starts_with("frame ")
            || trimmed.starts_with("agent ")
            || trimmed.starts_with("stack ")
            || trimmed.starts_with("queue ")
            || trimmed.starts_with('[')
        {
            has_component_keyword = true;
        }

        if trimmed == "salt" {
            has_salt_keyword = true;
        }

        if trimmed.starts_with("usecase ") || trimmed.starts_with("usecase\"") {
            has_usecase_keyword = true;
        }
        // (Name) round-bracket syntax for use cases
        if trimmed.starts_with('(') && trimmed.contains(')') && !trimmed.starts_with("()") {
            has_usecase_keyword = true;
        }

        if trimmed.starts_with("robust ") || trimmed.starts_with("concise ") {
            has_timing_keyword = true;
        }

        for kw in &class_keywords {
            let check = trimmed.strip_prefix('-').unwrap_or(trimmed);
            if check.starts_with(kw) || trimmed.contains(&format!(" {}", kw.trim())) {
                has_class_kw = true;
            }
        }
        for kw in &sequence_keywords_definitive {
            if trimmed.starts_with(kw) {
                return DiagramHint::Sequence;
            }
        }
        if trimmed.starts_with("actor ") {
            has_seq_actor = true;
        }
        // Ambiguous keywords: only classify as Sequence if no component keyword seen yet.
        for kw in &sequence_keywords_ambiguous {
            if trimmed.starts_with(kw) && !has_component_keyword {
                return DiagramHint::Sequence;
            }
        }
        if !has_arrow && (trimmed.contains("->") || trimmed.contains("<-")) {
            has_arrow = true;
            // Sequence-style arrow: "Word -> Word" or "Word -> Word : label"
            // Activity arrows look like: "-> label;" or bare "->"
            if let Some(pos) = trimmed.find("->").or_else(|| trimmed.find("<-")) {
                let before = trimmed[..pos].trim();
                let after_arrow = &trimmed[pos + 2..];
                let after = after_arrow.trim_start_matches(['>', '-']);
                let after = after.trim();
                // If there's a non-empty word before AND after the arrow, it's sequence-style
                if !before.is_empty()
                    && !before.starts_with(':')
                    && !after.is_empty()
                    && !after.starts_with(';')
                {
                    has_seq_arrow = true;
                }
            }
        }
    }

    if has_timing_keyword {
        return DiagramHint::Timing;
    }
    if has_salt_keyword {
        return DiagramHint::Salt;
    }
    if has_state_keyword {
        return DiagramHint::State;
    }
    if has_usecase_keyword {
        return DiagramHint::UseCase;
    }
    // Sequence fragments unambiguously identify sequence diagrams
    if has_seq_fragment && !has_state_keyword && !has_component_keyword {
        return DiagramHint::Sequence;
    }
    if has_component_keyword && !has_activity_action {
        return DiagramHint::Component;
    }
    if has_activity_action || has_activity_start_stop || has_activity_swimlane {
        return DiagramHint::Activity;
    }
    if has_component_keyword {
        return DiagramHint::Component;
    }
    if has_class_kw {
        return DiagramHint::Class;
    }
    // "actor" without use-case keywords implies sequence diagram
    if has_seq_actor {
        return DiagramHint::Sequence;
    }
    // Sequence-style arrows (A -> B : label) strongly suggest sequence diagram
    if has_seq_arrow {
        return DiagramHint::Sequence;
    }
    if has_arrow {
        return DiagramHint::Sequence;
    }

    DiagramHint::Unknown("unknown".into())
}

/// Return `true` when the `@startuml` body contains actual diagram content,
/// excluding metadata and cosmetic directives.
pub fn has_meaningful_uml_content(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut in_style_block = false;
    let mut skinparam_depth = 0usize;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if in_style_block {
            if trimmed == "</style>" {
                in_style_block = false;
            }
            i += 1;
            continue;
        }

        if skinparam_depth > 0 {
            skinparam_depth = skinparam_depth
                .saturating_add(trimmed.matches('{').count())
                .saturating_sub(trimmed.matches('}').count());
            i += 1;
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('\'') {
            i += 1;
            continue;
        }

        match trimmed {
            "title" => {
                if let Some((_, end)) = collect_block(&lines, i + 1, "end title", "endtitle") {
                    i = end + 1;
                } else {
                    i += 1;
                }
                continue;
            }
            "header" => {
                if let Some((_, end)) = collect_block(&lines, i + 1, "end header", "endheader") {
                    i = end + 1;
                } else {
                    i += 1;
                }
                continue;
            }
            "footer" => {
                if let Some((_, end)) = collect_block(&lines, i + 1, "end footer", "endfooter") {
                    i = end + 1;
                } else {
                    i += 1;
                }
                continue;
            }
            "legend" => {
                if let Some((_, end)) = collect_block(&lines, i + 1, "end legend", "endlegend") {
                    i = end + 1;
                } else {
                    i += 1;
                }
                continue;
            }
            "left to right direction" | "top to bottom direction" => {
                i += 1;
                continue;
            }
            "<style>" => {
                in_style_block = true;
                i += 1;
                continue;
            }
            _ => {}
        }

        if trimmed.starts_with("title ")
            || trimmed.starts_with("header ")
            || trimmed.starts_with("footer ")
            || trimmed.starts_with("caption ")
            || trimmed.starts_with("legend ")
            || trimmed.starts_with("hide ")
            || trimmed.starts_with("show ")
            || trimmed.starts_with("scale ")
        {
            i += 1;
            continue;
        }

        if trimmed.starts_with("<style>") {
            if !trimmed.contains("</style>") {
                in_style_block = true;
            }
            i += 1;
            continue;
        }

        if trimmed.starts_with("skinparam ") {
            skinparam_depth = trimmed
                .matches('{')
                .count()
                .saturating_sub(trimmed.matches('}').count());
            i += 1;
            continue;
        }

        return true;
    }

    false
}

/// Extract meta information (title / header / footer / legend / caption) from PlantUML source.
///
/// Supports both single-line and multi-line syntax:
/// - Single-line: `title My Title`
/// - Multi-line: `title\n...\nend title`
pub fn parse_meta(source: &str) -> DiagramMeta {
    let mut meta = DiagramMeta::default();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // title
        if trimmed == "title" {
            if let Some((block, end)) = collect_block(&lines, i + 1, "end title", "endtitle") {
                meta.title = Some(block);
                i = end + 1;
                continue;
            }
        } else if let Some(rest) = trimmed.strip_prefix("title ") {
            let rest = rest.trim();
            if !rest.is_empty() {
                meta.title = Some(rest.to_string());
            }
        }

        // header
        if trimmed == "header" {
            if let Some((block, end)) = collect_block(&lines, i + 1, "end header", "endheader") {
                meta.header = Some(block);
                i = end + 1;
                continue;
            }
        } else if let Some(rest) = trimmed.strip_prefix("header ") {
            let rest = rest.trim();
            if !rest.is_empty() {
                meta.header = Some(rest.to_string());
            }
        }

        // footer
        if trimmed == "footer" {
            if let Some((block, end)) = collect_block(&lines, i + 1, "end footer", "endfooter") {
                meta.footer = Some(block);
                i = end + 1;
                continue;
            }
        } else if let Some(rest) = trimmed.strip_prefix("footer ") {
            let rest = rest.trim();
            if !rest.is_empty() {
                meta.footer = Some(rest.to_string());
            }
        }

        // legend
        if trimmed == "legend" || trimmed.starts_with("legend ") {
            if let Some((block, end)) = collect_block(&lines, i + 1, "end legend", "endlegend") {
                meta.legend = Some(block);
                i = end + 1;
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("legend ") {
                let rest = rest.trim();
                if !rest.is_empty() {
                    meta.legend = Some(rest.to_string());
                }
            }
        }

        // caption
        if let Some(rest) = trimmed.strip_prefix("caption ") {
            let rest = rest.trim();
            if !rest.is_empty() {
                meta.caption = Some(rest.to_string());
            }
        }

        i += 1;
    }

    meta
}

/// Collect a multi-line block from lines[start_idx..] until end_marker or end_marker_alt is found.
fn collect_block(
    lines: &[&str],
    start_idx: usize,
    end_marker: &str,
    end_marker_alt: &str,
) -> Option<(String, usize)> {
    let mut collected = Vec::new();
    for (offset, line) in lines[start_idx..].iter().enumerate() {
        let t = line.trim();
        if t.eq_ignore_ascii_case(end_marker) || t.eq_ignore_ascii_case(end_marker_alt) {
            return Some((collected.join("\n"), start_idx + offset));
        }
        collected.push(t);
    }
    None
}

/// Extract SVG sprite definitions from source text.
///
/// Parses `sprite NAME <svg ...>...</svg>` blocks (single- or multi-line)
/// and returns a map of sprite name → SVG content, plus the cleaned source
/// with sprite definitions removed.
pub fn extract_sprites(source: &str) -> (String, HashMap<String, String>) {
    let mut sprites = HashMap::new();
    let mut cleaned = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Match: sprite [optional $]NAME <svg ...
        if let Some(rest) = trimmed.strip_prefix("sprite ") {
            let rest = rest.trim();
            // Strip optional leading $
            let rest = rest.strip_prefix('$').unwrap_or(rest);
            // Find the name (everything before the first space or <)
            if let Some(svg_start) = rest.find("<svg") {
                let name = rest[..svg_start].trim().to_string();
                if !name.is_empty() {
                    // Collect SVG content (may span multiple lines)
                    let mut svg_buf = rest[svg_start..].to_string();
                    if svg_buf.contains("</svg>") {
                        // Single-line sprite
                        sprites.insert(name, svg_buf);
                        i += 1;
                        continue;
                    }
                    // Multi-line: accumulate until </svg>
                    i += 1;
                    while i < lines.len() {
                        svg_buf.push('\n');
                        svg_buf.push_str(lines[i]);
                        if lines[i].contains("</svg>") {
                            break;
                        }
                        i += 1;
                    }
                    sprites.insert(name, svg_buf);
                    i += 1;
                    continue;
                }
            }
        }

        cleaned.push(lines[i]);
        i += 1;
    }

    (cleaned.join("\n"), sprites)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_block_basic() {
        let src = "@startuml\nclass Foo {}\n@enduml\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "class Foo {}");
    }

    #[test]
    fn extract_block_with_name() {
        let src = "@startuml myDiagram\nclass Foo {}\n@enduml\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "class Foo {}");
    }

    #[test]
    fn extract_block_none_when_empty() {
        let src = "no startuml here";
        assert!(extract_block(src).is_none());
    }

    #[test]
    fn extract_block_chen() {
        let src = "@startchen movies\nentity Foo {}\n@endchen\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "entity Foo {}");
    }

    #[test]
    fn extract_block_gantt() {
        let src = "@startgantt\n[Task] lasts 5 days\n@endgantt\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "[Task] lasts 5 days");
    }

    #[test]
    fn extract_block_json() {
        let src = "@startjson\n{\"a\": 1}\n@endjson\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "{\"a\": 1}");
    }

    #[test]
    fn extract_block_mindmap() {
        let src = "@startmindmap\n* root\n@endmindmap\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "* root");
    }

    #[test]
    fn extract_block_wbs() {
        let src = "@startwbs\n* root\n@endwbs\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "* root");
    }

    #[test]
    fn extract_block_yaml() {
        let src = "@startyaml\nkey: value\n@endyaml\n";
        let block = extract_block(src).unwrap();
        assert_eq!(block, "key: value");
    }

    #[test]
    fn detect_class_diagram() {
        let content = "class Foo {}\n";
        assert!(matches!(detect_diagram_type(content), DiagramHint::Class));
    }

    #[test]
    fn detect_unknown_diagram() {
        let content = "something else\n";
        assert!(matches!(
            detect_diagram_type(content),
            DiagramHint::Unknown(_)
        ));
    }

    #[test]
    fn detect_sequence_by_participant() {
        let content = "participant Alice\nAlice -> Bob : Hello\n";
        assert!(matches!(
            detect_diagram_type(content),
            DiagramHint::Sequence
        ));
    }

    #[test]
    fn detect_sequence_by_arrow() {
        let content = "Alice -> Bob : Hello\n";
        assert!(matches!(
            detect_diagram_type(content),
            DiagramHint::Sequence
        ));
    }

    #[test]
    fn detect_activity_by_action() {
        let content = ":foo;\nstop\n";
        assert!(matches!(
            detect_diagram_type(content),
            DiagramHint::Activity
        ));
    }

    #[test]
    fn detect_activity_by_swimlane() {
        let content = "|Actor 1|\nstart\n:foo;\n";
        assert!(matches!(
            detect_diagram_type(content),
            DiagramHint::Activity
        ));
    }

    #[test]
    fn detect_state_by_keyword() {
        let content = "state s1\n[*] --> s1\n";
        assert!(matches!(detect_diagram_type(content), DiagramHint::State));
    }

    #[test]
    fn detect_component_by_keyword() {
        let content = "component A\ncomponent B\nA -> B\n";
        assert!(matches!(
            detect_diagram_type(content),
            DiagramHint::Component
        ));
    }

    #[test]
    fn detect_component_by_file_keyword() {
        let content = "file Report\n";
        assert!(matches!(
            detect_diagram_type(content),
            DiagramHint::Component
        ));
    }

    #[test]
    fn detect_timing_by_robust() {
        let content = "robust \"DNS\" as DNS\nconcise \"Web\" as WB\n";
        assert!(matches!(detect_diagram_type(content), DiagramHint::Timing));
    }

    #[test]
    fn detect_start_tag_chen() {
        assert!(matches!(
            detect_start_tag("@startchen movies\nentity X {}\n@endchen"),
            Some(DiagramHint::Erd)
        ));
    }

    #[test]
    fn detect_start_tag_gantt() {
        assert!(matches!(
            detect_start_tag("@startgantt\n[T] lasts 5 days\n@endgantt"),
            Some(DiagramHint::Gantt)
        ));
    }

    #[test]
    fn detect_start_tag_json() {
        assert!(matches!(
            detect_start_tag("@startjson\n{}\n@endjson"),
            Some(DiagramHint::Json)
        ));
    }

    #[test]
    fn detect_start_tag_mindmap() {
        assert!(matches!(
            detect_start_tag("@startmindmap\n* root\n@endmindmap"),
            Some(DiagramHint::Mindmap)
        ));
    }

    #[test]
    fn detect_start_tag_wbs() {
        assert!(matches!(
            detect_start_tag("@startwbs\n* root\n@endwbs"),
            Some(DiagramHint::Wbs)
        ));
    }

    #[test]
    fn detect_start_tag_yaml() {
        assert!(matches!(
            detect_start_tag("@startyaml\nkey: val\n@endyaml"),
            Some(DiagramHint::Yaml)
        ));
    }

    #[test]
    fn detect_start_tag_uml_returns_none() {
        assert!(detect_start_tag("@startuml\nclass Foo\n@enduml").is_none());
    }

    // ── parse_meta tests ────────────────────────────────────────────

    #[test]
    fn parse_meta_empty_source() {
        let meta = parse_meta("");
        assert!(meta.is_empty());
    }

    #[test]
    fn parse_meta_single_line_title() {
        let src = "@startuml\ntitle My Title\nclass Foo\n@enduml";
        let meta = parse_meta(src);
        assert_eq!(meta.title.as_deref(), Some("My Title"));
    }

    #[test]
    fn parse_meta_multi_line_title() {
        let src = "@startuml\ntitle\nLine 1\nLine 2\nend title\nclass Foo\n@enduml";
        let meta = parse_meta(src);
        assert_eq!(meta.title.as_deref(), Some("Line 1\nLine 2"));
    }

    #[test]
    fn parse_meta_single_line_header() {
        let src = "header My Header\nclass Foo";
        let meta = parse_meta(src);
        assert_eq!(meta.header.as_deref(), Some("My Header"));
    }

    #[test]
    fn parse_meta_multi_line_header() {
        let src = "header\nH1\nH2\nend header\nclass Foo";
        let meta = parse_meta(src);
        assert_eq!(meta.header.as_deref(), Some("H1\nH2"));
    }

    #[test]
    fn parse_meta_single_line_footer() {
        let src = "footer Page 1\nclass Foo";
        let meta = parse_meta(src);
        assert_eq!(meta.footer.as_deref(), Some("Page 1"));
    }

    #[test]
    fn parse_meta_multi_line_footer() {
        let src = "footer\nF1\nF2\nend footer\nclass Foo";
        let meta = parse_meta(src);
        assert_eq!(meta.footer.as_deref(), Some("F1\nF2"));
    }

    #[test]
    fn parse_meta_caption() {
        let src = "caption Figure 1. Overview\nclass Foo";
        let meta = parse_meta(src);
        assert_eq!(meta.caption.as_deref(), Some("Figure 1. Overview"));
    }

    #[test]
    fn parse_meta_legend_multiline() {
        let src = "legend\nLegend line 1\nLegend line 2\nend legend\nclass Foo";
        let meta = parse_meta(src);
        assert_eq!(meta.legend.as_deref(), Some("Legend line 1\nLegend line 2"));
    }

    #[test]
    fn parse_meta_legend_with_position() {
        let src = "legend right\nSome legend\nend legend";
        let meta = parse_meta(src);
        assert_eq!(meta.legend.as_deref(), Some("Some legend"));
    }

    #[test]
    fn parse_meta_all_fields() {
        let src =
            "header Top\ntitle Big Title\ncaption Fig 1\nfooter Bottom\nlegend\nL1\nend legend";
        let meta = parse_meta(src);
        assert_eq!(meta.title.as_deref(), Some("Big Title"));
        assert_eq!(meta.header.as_deref(), Some("Top"));
        assert_eq!(meta.footer.as_deref(), Some("Bottom"));
        assert_eq!(meta.caption.as_deref(), Some("Fig 1"));
        assert_eq!(meta.legend.as_deref(), Some("L1"));
        assert!(!meta.is_empty());
    }

    #[test]
    fn parse_meta_no_directives() {
        let src = "@startuml\nclass Foo {}\nFoo --> Bar\n@enduml";
        let meta = parse_meta(src);
        assert!(meta.is_empty());
    }

    #[test]
    fn parse_meta_endtitle_alt_form() {
        let src = "title\nAlt form\nendtitle";
        let meta = parse_meta(src);
        assert_eq!(meta.title.as_deref(), Some("Alt form"));
    }

    #[test]
    fn parse_meta_endheader_alt_form() {
        let src = "header\nAlt header\nendheader";
        let meta = parse_meta(src);
        assert_eq!(meta.header.as_deref(), Some("Alt header"));
    }

    #[test]
    fn parse_meta_endfooter_alt_form() {
        let src = "footer\nAlt footer\nendfooter";
        let meta = parse_meta(src);
        assert_eq!(meta.footer.as_deref(), Some("Alt footer"));
    }

    #[test]
    fn parse_meta_endlegend_alt_form() {
        let src = "legend\nAlt legend\nendlegend";
        let meta = parse_meta(src);
        assert_eq!(meta.legend.as_deref(), Some("Alt legend"));
    }

    #[test]
    fn parse_meta_is_empty_default() {
        let meta = DiagramMeta::default();
        assert!(meta.is_empty());
    }

    #[test]
    fn meta_only_content_is_not_meaningful() {
        let content = "title\nHello\nend title\nheader Top\n";
        assert!(!has_meaningful_uml_content(content));
    }

    #[test]
    fn file_content_is_meaningful() {
        let content = "title Example\nfile report [\nhello\n]\n";
        assert!(has_meaningful_uml_content(content));
    }

    #[test]
    fn extract_sprites_single_line() {
        let src = "Alice -> Bob : hi\nsprite redrect <svg viewBox=\"0 0 100 50\"><rect/></svg>\nBob -> Alice : ok\n";
        let (cleaned, sprites) = extract_sprites(src);
        assert_eq!(sprites.len(), 1);
        assert!(sprites.contains_key("redrect"));
        assert!(sprites["redrect"].contains("<rect/>"));
        assert!(!cleaned.contains("sprite"));
        assert!(cleaned.contains("Alice -> Bob"));
    }

    #[test]
    fn extract_sprites_multiline() {
        let src = "sprite myicon <svg viewBox=\"0 0 50 50\">\n  <circle cx=\"25\" cy=\"25\" r=\"20\"/>\n</svg>\nAlice -> Bob\n";
        let (cleaned, sprites) = extract_sprites(src);
        assert_eq!(sprites.len(), 1);
        assert!(sprites["myicon"].contains("<circle"));
        assert!(cleaned.contains("Alice -> Bob"));
        assert!(!cleaned.contains("sprite"));
    }

    #[test]
    fn extract_sprites_dollar_prefix() {
        let src = "sprite $icon <svg viewBox=\"0 0 10 10\"><rect/></svg>\n";
        let (_, sprites) = extract_sprites(src);
        assert_eq!(sprites.len(), 1);
        assert!(sprites.contains_key("icon"));
    }

    #[test]
    fn extract_sprites_none() {
        let src = "Alice -> Bob : hello\n";
        let (cleaned, sprites) = extract_sprites(src);
        assert!(sprites.is_empty());
        assert_eq!(cleaned, "Alice -> Bob : hello");
    }
}
