/// Creole markup parser for PlantUML rich text.
///
/// Supports: bold, italic, underline, strikethrough, monospace, color, size,
/// HTML-style tags, line breaks, horizontal rules, bullet/numbered lists, and tables.
use crate::model::hyperlink::parse_hyperlink;
use crate::model::richtext::{RichText, TextSpan};

/// Parse a Creole-formatted string into a structured `RichText` model.
pub fn parse_creole(input: &str) -> RichText {
    let lines = split_lines(input);

    if lines.is_empty() {
        return RichText::Line(vec![]);
    }

    let mut blocks: Vec<RichText> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];

        // Horizontal rule: a line that is exactly "----" (with possible surrounding dashes)
        if is_horizontal_rule(line) {
            blocks.push(RichText::HorizontalRule);
            i += 1;
            continue;
        }

        // Table: lines starting with '|'
        if line.starts_with('|') {
            let (table, consumed) = parse_table(&lines[i..]);
            blocks.push(table);
            i += consumed;
            continue;
        }

        // Bullet list: lines starting with '*' (and space)
        if is_bullet_line(line) {
            let (list, consumed) = parse_list(&lines[i..], '*');
            blocks.push(list);
            i += consumed;
            continue;
        }

        // Numbered list: lines starting with '#' (and space)
        if is_numbered_line(line) {
            let (list, consumed) = parse_list(&lines[i..], '#');
            blocks.push(list);
            i += consumed;
            continue;
        }

        // Regular line
        let spans = parse_inline(line);
        blocks.push(RichText::Line(spans));
        i += 1;
    }

    if blocks.len() == 1 {
        blocks.into_iter().next().unwrap()
    } else {
        RichText::Block(blocks)
    }
}

// ---------------------------------------------------------------------------
// Line splitting
// ---------------------------------------------------------------------------

/// Split input on real newlines and on `\n` / `\\n` escape sequences.
fn split_lines(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut in_link = false;
    let mut tooltip_depth = 0usize;
    let chars: Vec<char> = input.chars().collect();
    let mut buf = String::new();
    let mut i = 0;

    while i < chars.len() {
        if !in_link && i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            in_link = true;
            buf.push('[');
            buf.push('[');
            i += 2;
            continue;
        }

        if in_link && i + 1 < chars.len() && chars[i] == ']' && chars[i + 1] == ']' {
            in_link = false;
            tooltip_depth = 0;
            buf.push(']');
            buf.push(']');
            i += 2;
            continue;
        }

        if in_link {
            match chars[i] {
                '{' => tooltip_depth += 1,
                '}' => tooltip_depth = tooltip_depth.saturating_sub(1),
                _ => {}
            }
        }

        if chars[i] == '\n' {
            if in_link || tooltip_depth > 0 {
                buf.push('\n');
            } else {
                parts.push(std::mem::take(&mut buf));
            }
            i += 1;
            continue;
        }

        if chars[i] == '\\' {
            if i + 2 < chars.len() && chars[i + 1] == '\\' && chars[i + 2] == 'n' {
                if in_link || tooltip_depth > 0 {
                    buf.push('\n');
                } else {
                    parts.push(std::mem::take(&mut buf));
                }
                i += 3;
                continue;
            }
            if i + 1 < chars.len() && chars[i + 1] == 'n' {
                if in_link || tooltip_depth > 0 {
                    buf.push('\n');
                } else {
                    parts.push(std::mem::take(&mut buf));
                }
                i += 2;
                continue;
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    parts.push(buf);
    parts
}

// ---------------------------------------------------------------------------
// Block-level detection helpers
// ---------------------------------------------------------------------------

fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 4 && trimmed.chars().all(|c| c == '-')
}

fn is_bullet_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("* ")
        || (trimmed.len() >= 2 && trimmed.starts_with('*') && trimmed.as_bytes()[1] == b' ')
}

fn is_numbered_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("# ")
        || (trimmed.len() >= 2 && trimmed.starts_with('#') && trimmed.as_bytes()[1] == b' ')
}

/// Parse consecutive list lines (bullet or numbered).
fn parse_list(lines: &[String], marker: char) -> (RichText, usize) {
    let mut items = Vec::new();
    let mut consumed = 0;
    let check = if marker == '*' {
        is_bullet_line as fn(&str) -> bool
    } else {
        is_numbered_line
    };

    for line in lines {
        if !check(line) {
            break;
        }
        let trimmed = line.trim_start();
        // Skip the marker and the space after it
        let content = &trimmed[2..];
        items.push(RichText::Line(parse_inline(content)));
        consumed += 1;
    }

    let list = if marker == '*' {
        RichText::BulletList(items)
    } else {
        RichText::NumberedList(items)
    };
    (list, consumed)
}

/// Parse consecutive table lines.
fn parse_table(lines: &[String]) -> (RichText, usize) {
    let mut headers: Vec<Vec<TextSpan>> = Vec::new();
    let mut rows: Vec<Vec<Vec<TextSpan>>> = Vec::new();
    let mut consumed = 0;

    for line in lines {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            break;
        }
        consumed += 1;

        // Check if this is a header row (contains `|=`)
        let is_header = trimmed.contains("|=");

        let cells = parse_table_cells(trimmed, is_header);
        if is_header {
            headers = cells;
        } else {
            rows.push(cells);
        }
    }

    (RichText::Table { headers, rows }, consumed)
}

/// Parse cells from a table line like `| cell1 | cell2 |` or `|= hdr1 |= hdr2 |`.
fn parse_table_cells(line: &str, is_header: bool) -> Vec<Vec<TextSpan>> {
    let mut cells = Vec::new();
    let trimmed = line.trim();

    // Remove leading '|'
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    // Remove trailing '|'
    let inner = inner.strip_suffix('|').unwrap_or(inner);

    for part in inner.split('|') {
        let cell_text = part.trim();
        // Strip header marker `=` at the start for header cells
        let cell_text = if is_header {
            cell_text.strip_prefix('=').unwrap_or(cell_text).trim()
        } else {
            cell_text
        };
        cells.push(parse_inline(cell_text));
    }

    cells
}

// ---------------------------------------------------------------------------
// Inline parsing
// ---------------------------------------------------------------------------

/// Parse inline Creole markup into a flat list of `TextSpan` values.
pub fn parse_inline(input: &str) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    parse_inline_chars(&chars, 0, chars.len(), &mut spans, None);
    merge_plains(spans)
}

/// Recursive inline parser operating on a char slice `[start..end)`.
/// `stop` is an optional delimiter that terminates this scope.
fn parse_inline_chars(
    chars: &[char],
    start: usize,
    end: usize,
    out: &mut Vec<TextSpan>,
    stop: Option<&str>,
) -> usize {
    let mut i = start;
    let mut plain_buf = String::new();

    while i < end {
        // Check for stop delimiter first
        if let Some(delim) = stop {
            if matches_at(chars, i, delim) {
                flush_plain(&mut plain_buf, out);
                return i + delim.len();
            }
        }

        // Monospace `""..""` — no nesting inside
        if matches_at(chars, i, "\"\"") {
            flush_plain(&mut plain_buf, out);
            let content_start = i + 2;
            if let Some(close) = find_delimiter(chars, content_start, end, "\"\"") {
                let text: String = chars[content_start..close].iter().collect();
                out.push(TextSpan::Monospace(text));
                i = close + 2;
            } else {
                // No closing delimiter — treat as plain text
                plain_buf.push_str("\"\"");
                i += 2;
            }
            continue;
        }

        // Bold `**`
        if matches_at(chars, i, "**") {
            flush_plain(&mut plain_buf, out);
            let content_start = i + 2;
            let mut inner = Vec::new();
            let after = parse_inline_chars(chars, content_start, end, &mut inner, Some("**"));
            if after > content_start && after != content_start {
                out.push(TextSpan::Bold(merge_plains(inner)));
                i = after;
            } else {
                plain_buf.push_str("**");
                i += 2;
            }
            continue;
        }

        // Italic `//`
        if matches_at(chars, i, "//") {
            flush_plain(&mut plain_buf, out);
            let content_start = i + 2;
            let mut inner = Vec::new();
            let after = parse_inline_chars(chars, content_start, end, &mut inner, Some("//"));
            if after > content_start {
                out.push(TextSpan::Italic(merge_plains(inner)));
                i = after;
            } else {
                plain_buf.push_str("//");
                i += 2;
            }
            continue;
        }

        // Underline `__`
        if matches_at(chars, i, "__") {
            flush_plain(&mut plain_buf, out);
            let content_start = i + 2;
            let mut inner = Vec::new();
            let after = parse_inline_chars(chars, content_start, end, &mut inner, Some("__"));
            if after > content_start {
                out.push(TextSpan::Underline(merge_plains(inner)));
                i = after;
            } else {
                plain_buf.push_str("__");
                i += 2;
            }
            continue;
        }

        // Strikethrough `~~`
        if matches_at(chars, i, "~~") {
            flush_plain(&mut plain_buf, out);
            let content_start = i + 2;
            let mut inner = Vec::new();
            let after = parse_inline_chars(chars, content_start, end, &mut inner, Some("~~"));
            if after > content_start {
                out.push(TextSpan::Strikethrough(merge_plains(inner)));
                i = after;
            } else {
                plain_buf.push_str("~~");
                i += 2;
            }
            continue;
        }

        // Escape: `~` escapes the next character (checked after double-char delimiters)
        if chars[i] == '~' && i + 1 < end {
            plain_buf.push(chars[i + 1]);
            i += 2;
            continue;
        }

        // Inline SVG sprite reference: <$name>
        if chars[i] == '<' && i + 2 < end && chars[i + 1] == '$' {
            if let Some((span, consumed)) = try_parse_sprite_ref(chars, i, end) {
                flush_plain(&mut plain_buf, out);
                out.push(span);
                i += consumed;
                continue;
            }
        }

        // HTML-style tags: <b>, <i>, <u>, <s>, <color:...>, <size:...>
        if chars[i] == '<' {
            if let Some((span, consumed)) = try_parse_html_tag(chars, i, end) {
                flush_plain(&mut plain_buf, out);
                out.push(span);
                i += consumed;
                continue;
            }
        }

        // Link `[[url]]` or `[[url label]]`
        if matches_at(chars, i, "[[") {
            if let Some((span, consumed)) = try_parse_link(chars, i, end) {
                flush_plain(&mut plain_buf, out);
                out.push(span);
                i += consumed;
                continue;
            }
        }

        // Default: plain character
        plain_buf.push(chars[i]);
        i += 1;
    }

    flush_plain(&mut plain_buf, out);
    i
}

// ---------------------------------------------------------------------------
// HTML tag parsing
// ---------------------------------------------------------------------------

fn try_parse_html_tag(chars: &[char], start: usize, end: usize) -> Option<(TextSpan, usize)> {
    // Try simple tags: <b>...</b>, <i>...</i>, <u>...</u>, <s>...</s>
    for (open, close, make) in &[
        ("<b>", "</b>", make_bold as fn(Vec<TextSpan>) -> TextSpan),
        ("<i>", "</i>", make_italic as fn(Vec<TextSpan>) -> TextSpan),
        (
            "<u>",
            "</u>",
            make_underline as fn(Vec<TextSpan>) -> TextSpan,
        ),
        ("<s>", "</s>", make_strike as fn(Vec<TextSpan>) -> TextSpan),
    ] {
        if matches_at_ci(chars, start, open) {
            let content_start = start + open.len();
            if let Some(close_pos) = find_tag_close_ci(chars, content_start, end, close) {
                let mut inner = Vec::new();
                parse_inline_chars(chars, content_start, close_pos, &mut inner, None);
                let span = make(merge_plains(inner));
                return Some((span, close_pos + close.len() - start));
            }
        }
    }

    // <sub>...</sub>
    if matches_at_ci(chars, start, "<sub>") {
        let content_start = start + 5; // length of "<sub>"
        if let Some(close_pos) = find_tag_close_ci(chars, content_start, end, "</sub>") {
            let mut inner = Vec::new();
            parse_inline_chars(chars, content_start, close_pos, &mut inner, None);
            let span = TextSpan::Subscript(merge_plains(inner));
            return Some((span, close_pos + 6 - start)); // 6 = "</sub>".len()
        }
    }

    // <sup>...</sup>
    if matches_at_ci(chars, start, "<sup>") {
        let content_start = start + 5; // length of "<sup>"
        if let Some(close_pos) = find_tag_close_ci(chars, content_start, end, "</sup>") {
            let mut inner = Vec::new();
            parse_inline_chars(chars, content_start, close_pos, &mut inner, None);
            let span = TextSpan::Superscript(merge_plains(inner));
            return Some((span, close_pos + 6 - start)); // 6 = "</sup>".len()
        }
    }

    // <back:COLOR>...</back>  (unclosed → rest of scope becomes content)
    if matches_at_ci(chars, start, "<back:") {
        let attr_start = start + 6; // length of "<back:"
        if let Some(gt_pos) = find_char(chars, attr_start, end, '>') {
            let color: String = chars[attr_start..gt_pos].iter().collect();
            let content_start = gt_pos + 1;
            let close_pos = find_tag_close_ci(chars, content_start, end, "</back>")
                .unwrap_or(end);
            let consumed_end = if close_pos == end {
                end
            } else {
                close_pos + 7 // 7 = "</back>".len()
            };
            let mut inner = Vec::new();
            parse_inline_chars(chars, content_start, close_pos, &mut inner, None);
            let span = TextSpan::BackHighlight {
                color,
                content: merge_plains(inner),
            };
            return Some((span, consumed_end - start));
        }
    }

    // <font:NAME>...</font>  (unclosed → rest of scope becomes content)
    if matches_at_ci(chars, start, "<font:") {
        let attr_start = start + 6; // length of "<font:"
        if let Some(gt_pos) = find_char(chars, attr_start, end, '>') {
            let family: String = chars[attr_start..gt_pos].iter().collect();
            let content_start = gt_pos + 1;
            let close_pos = find_tag_close_ci(chars, content_start, end, "</font>")
                .unwrap_or(end);
            let consumed_end = if close_pos == end {
                end
            } else {
                close_pos + 7 // 7 = "</font>".len()
            };
            let mut inner = Vec::new();
            parse_inline_chars(chars, content_start, close_pos, &mut inner, None);
            let span = TextSpan::FontFamily {
                family,
                content: merge_plains(inner),
            };
            return Some((span, consumed_end - start));
        }
    }

    // <color:NAME>...</color>  (unclosed → rest of scope becomes content)
    if matches_at_ci(chars, start, "<color:") {
        let attr_start = start + 7; // length of "<color:"
        if let Some(gt_pos) = find_char(chars, attr_start, end, '>') {
            let color: String = chars[attr_start..gt_pos].iter().collect();
            let content_start = gt_pos + 1;
            let close_pos = find_tag_close_ci(chars, content_start, end, "</color>")
                .unwrap_or(end);
            let consumed_end = if close_pos == end {
                end
            } else {
                close_pos + 8 // 8 = "</color>".len()
            };
            let mut inner = Vec::new();
            parse_inline_chars(chars, content_start, close_pos, &mut inner, None);
            let span = TextSpan::Colored {
                color,
                content: merge_plains(inner),
            };
            return Some((span, consumed_end - start));
        }
    }

    // <size:N>...</size>  (unclosed → rest of scope becomes content)
    if matches_at_ci(chars, start, "<size:") {
        let attr_start = start + 6; // length of "<size:"
        if let Some(gt_pos) = find_char(chars, attr_start, end, '>') {
            let size_str: String = chars[attr_start..gt_pos].iter().collect();
            if let Ok(size) = size_str.trim().parse::<f64>() {
                let content_start = gt_pos + 1;
                let close_pos = find_tag_close_ci(chars, content_start, end, "</size>")
                    .unwrap_or(end);
                let consumed_end = if close_pos == end {
                    end
                } else {
                    close_pos + 7 // 7 = "</size>".len()
                };
                let mut inner = Vec::new();
                parse_inline_chars(chars, content_start, close_pos, &mut inner, None);
                let span = TextSpan::Sized {
                    size,
                    content: merge_plains(inner),
                };
                return Some((span, consumed_end - start));
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Sprite reference parsing: <$name>
// ---------------------------------------------------------------------------

fn try_parse_sprite_ref(chars: &[char], start: usize, end: usize) -> Option<(TextSpan, usize)> {
    // Expected pattern: <$name> where name is [-\w/]+
    if !matches_at(chars, start, "<$") {
        return None;
    }
    let name_start = start + 2;
    let mut i = name_start;
    while i < end && chars[i] != '>' && chars[i] != ',' && chars[i] != ' ' {
        i += 1;
    }
    if i >= end || i == name_start {
        return None;
    }
    // Skip optional scale/color parameters: <$name,scale=2> or <#color $name>
    while i < end && chars[i] != '>' {
        i += 1;
    }
    if i >= end {
        return None;
    }
    // chars[i] == '>'
    let name: String = chars[name_start..i]
        .iter()
        .collect::<String>()
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if name.is_empty() {
        return None;
    }
    Some((TextSpan::InlineSvg { name }, i + 1 - start))
}

// ---------------------------------------------------------------------------
// Link parsing
// ---------------------------------------------------------------------------

fn try_parse_link(chars: &[char], start: usize, end: usize) -> Option<(TextSpan, usize)> {
    if !matches_at(chars, start, "[[") {
        return None;
    }
    let content_start = start + 2;
    let close_pos = find_delimiter(chars, content_start, end, "]]")?;
    let consumed = close_pos + 2 - start;
    let raw: String = chars[start..close_pos + 2].iter().collect();
    let (link, _) = parse_hyperlink(&raw)?;
    Some((
        TextSpan::Link {
            url: link.url,
            tooltip: link.tooltip,
            label: link.label,
        },
        consumed,
    ))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_bold(v: Vec<TextSpan>) -> TextSpan {
    TextSpan::Bold(v)
}
fn make_italic(v: Vec<TextSpan>) -> TextSpan {
    TextSpan::Italic(v)
}
fn make_underline(v: Vec<TextSpan>) -> TextSpan {
    TextSpan::Underline(v)
}
fn make_strike(v: Vec<TextSpan>) -> TextSpan {
    TextSpan::Strikethrough(v)
}

/// Check if `pattern` matches at position `pos` in `chars`.
fn matches_at(chars: &[char], pos: usize, pattern: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    if pos + pat.len() > chars.len() {
        return false;
    }
    for (j, pc) in pat.iter().enumerate() {
        if chars[pos + j] != *pc {
            return false;
        }
    }
    true
}

/// Case-insensitive variant of `matches_at`.
fn matches_at_ci(chars: &[char], pos: usize, pattern: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    if pos + pat.len() > chars.len() {
        return false;
    }
    for (j, pc) in pat.iter().enumerate() {
        if !chars[pos + j].eq_ignore_ascii_case(pc) {
            return false;
        }
    }
    true
}

/// Find the position of a multi-char delimiter.
fn find_delimiter(chars: &[char], start: usize, end: usize, delim: &str) -> Option<usize> {
    let d: Vec<char> = delim.chars().collect();
    let dlen = d.len();
    if dlen == 0 {
        return Some(start);
    }
    let mut i = start;
    while i + dlen <= end {
        if (0..dlen).all(|j| chars[i + j] == d[j]) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Case-insensitive search for a closing tag.
fn find_tag_close_ci(chars: &[char], start: usize, end: usize, tag: &str) -> Option<usize> {
    let t: Vec<char> = tag.chars().collect();
    let tlen = t.len();
    let mut i = start;
    while i + tlen <= end {
        if (0..tlen).all(|j| chars[i + j].eq_ignore_ascii_case(&t[j])) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_char(chars: &[char], start: usize, end: usize, target: char) -> Option<usize> {
    chars[start..end]
        .iter()
        .position(|&c| c == target)
        .map(|pos| pos + start)
}

fn flush_plain(buf: &mut String, out: &mut Vec<TextSpan>) {
    if !buf.is_empty() {
        out.push(TextSpan::Plain(std::mem::take(buf)));
    }
}

/// Merge adjacent `Plain` spans into single spans.
fn merge_plains(spans: Vec<TextSpan>) -> Vec<TextSpan> {
    let mut result: Vec<TextSpan> = Vec::new();
    for span in spans {
        if let TextSpan::Plain(ref s) = span {
            if let Some(TextSpan::Plain(ref mut prev)) = result.last_mut() {
                prev.push_str(s);
                continue;
            }
        }
        result.push(span);
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::richtext::plain_text;

    // -- Plain text passthrough --

    #[test]
    fn test_plain_text_passthrough() {
        let rt = parse_creole("hello world");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Plain("hello world".into())])
        );
    }

    #[test]
    fn test_empty_input() {
        let rt = parse_creole("");
        assert_eq!(rt, RichText::Line(vec![]));
    }

    // -- Single formatting --

    #[test]
    fn test_bold() {
        let rt = parse_creole("**bold**");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Bold(vec![TextSpan::Plain("bold".into())])])
        );
    }

    #[test]
    fn test_italic() {
        let rt = parse_creole("//italic//");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Italic(vec![TextSpan::Plain(
                "italic".into()
            )])])
        );
    }

    #[test]
    fn test_underline() {
        let rt = parse_creole("__underline__");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Underline(vec![TextSpan::Plain(
                "underline".into()
            )])])
        );
    }

    #[test]
    fn test_strikethrough() {
        let rt = parse_creole("~~strike~~");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Strikethrough(vec![TextSpan::Plain(
                "strike".into()
            )])])
        );
    }

    #[test]
    fn test_monospace() {
        let rt = parse_creole("\"\"mono\"\"");
        assert_eq!(rt, RichText::Line(vec![TextSpan::Monospace("mono".into())]));
    }

    // -- Nested formatting --

    #[test]
    fn test_bold_italic_nested() {
        let rt = parse_creole("**//bold italic//**");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Bold(vec![TextSpan::Italic(vec![
                TextSpan::Plain("bold italic".into())
            ])])])
        );
    }

    #[test]
    fn test_italic_bold_nested() {
        let rt = parse_creole("//**italic bold**//");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Italic(vec![TextSpan::Bold(vec![
                TextSpan::Plain("italic bold".into())
            ])])])
        );
    }

    // -- HTML-style tags --

    #[test]
    fn test_html_bold() {
        let rt = parse_creole("<b>bold</b>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Bold(vec![TextSpan::Plain("bold".into())])])
        );
    }

    #[test]
    fn test_html_italic() {
        let rt = parse_creole("<i>italic</i>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Italic(vec![TextSpan::Plain(
                "italic".into()
            )])])
        );
    }

    #[test]
    fn test_html_underline() {
        let rt = parse_creole("<u>underline</u>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Underline(vec![TextSpan::Plain(
                "underline".into()
            )])])
        );
    }

    #[test]
    fn test_html_strike() {
        let rt = parse_creole("<s>strike</s>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Strikethrough(vec![TextSpan::Plain(
                "strike".into()
            )])])
        );
    }

    // -- Color and size --

    #[test]
    fn test_color_tag() {
        let rt = parse_creole("<color:red>red text</color>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Colored {
                color: "red".into(),
                content: vec![TextSpan::Plain("red text".into())]
            }])
        );
    }

    #[test]
    fn test_size_tag() {
        let rt = parse_creole("<size:18>big</size>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Sized {
                size: 18.0,
                content: vec![TextSpan::Plain("big".into())]
            }])
        );
    }

    #[test]
    fn test_color_with_hex() {
        let rt = parse_creole("<color:#FF0000>red</color>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Colored {
                color: "#FF0000".into(),
                content: vec![TextSpan::Plain("red".into())]
            }])
        );
    }

    // -- Line breaks --

    #[test]
    fn test_line_break_backslash_n() {
        let rt = parse_creole("line1\\nline2");
        assert_eq!(
            rt,
            RichText::Block(vec![
                RichText::Line(vec![TextSpan::Plain("line1".into())]),
                RichText::Line(vec![TextSpan::Plain("line2".into())]),
            ])
        );
    }

    #[test]
    fn test_line_break_double_backslash_n() {
        let rt = parse_creole("a\\\\nb");
        assert_eq!(
            rt,
            RichText::Block(vec![
                RichText::Line(vec![TextSpan::Plain("a".into())]),
                RichText::Line(vec![TextSpan::Plain("b".into())]),
            ])
        );
    }

    #[test]
    fn test_real_newline() {
        let rt = parse_creole("first\nsecond");
        assert_eq!(
            rt,
            RichText::Block(vec![
                RichText::Line(vec![TextSpan::Plain("first".into())]),
                RichText::Line(vec![TextSpan::Plain("second".into())]),
            ])
        );
    }

    #[test]
    fn test_link_tooltip_preserves_escaped_newline() {
        let rt = parse_creole("[[https://example.com{line1\\nline2} Label]]");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Link {
                url: "https://example.com".into(),
                tooltip: Some("line1\nline2".into()),
                label: Some("Label".into()),
            }])
        );
    }

    #[test]
    fn test_link_tooltip_preserves_real_newline() {
        let rt = parse_creole("[[{line1\nline2} Label]]");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Link {
                url: String::new(),
                tooltip: Some("line1\nline2".into()),
                label: Some("Label".into()),
            }])
        );
    }

    // -- Bullet list --

    #[test]
    fn test_bullet_list() {
        let rt = parse_creole("* apple\n* banana\n* cherry");
        assert_eq!(
            rt,
            RichText::BulletList(vec![
                RichText::Line(vec![TextSpan::Plain("apple".into())]),
                RichText::Line(vec![TextSpan::Plain("banana".into())]),
                RichText::Line(vec![TextSpan::Plain("cherry".into())]),
            ])
        );
    }

    // -- Numbered list --

    #[test]
    fn test_numbered_list() {
        let rt = parse_creole("# one\n# two\n# three");
        assert_eq!(
            rt,
            RichText::NumberedList(vec![
                RichText::Line(vec![TextSpan::Plain("one".into())]),
                RichText::Line(vec![TextSpan::Plain("two".into())]),
                RichText::Line(vec![TextSpan::Plain("three".into())]),
            ])
        );
    }

    // -- Tables --

    #[test]
    fn test_table_basic() {
        let rt = parse_creole("|= Name |= Age |\n| Alice | 30 |");
        assert_eq!(
            rt,
            RichText::Table {
                headers: vec![
                    vec![TextSpan::Plain("Name".into())],
                    vec![TextSpan::Plain("Age".into())],
                ],
                rows: vec![vec![
                    vec![TextSpan::Plain("Alice".into())],
                    vec![TextSpan::Plain("30".into())],
                ]],
            }
        );
    }

    #[test]
    fn test_table_no_header() {
        let rt = parse_creole("| a | b |\n| c | d |");
        assert_eq!(
            rt,
            RichText::Table {
                headers: vec![],
                rows: vec![
                    vec![
                        vec![TextSpan::Plain("a".into())],
                        vec![TextSpan::Plain("b".into())],
                    ],
                    vec![
                        vec![TextSpan::Plain("c".into())],
                        vec![TextSpan::Plain("d".into())],
                    ],
                ],
            }
        );
    }

    // -- Horizontal rule --

    #[test]
    fn test_horizontal_rule() {
        let rt = parse_creole("above\n----\nbelow");
        assert_eq!(
            rt,
            RichText::Block(vec![
                RichText::Line(vec![TextSpan::Plain("above".into())]),
                RichText::HorizontalRule,
                RichText::Line(vec![TextSpan::Plain("below".into())]),
            ])
        );
    }

    // -- Escape sequences --

    #[test]
    fn test_escape_bold() {
        let rt = parse_creole("~*~*not bold~*~*");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Plain("**not bold**".into())])
        );
    }

    #[test]
    fn test_escape_tilde() {
        let rt = parse_creole("~~ is tilde");
        // `~~` with no closing `~~` before end — treated as unmatched
        // Actually `~~` starts strikethrough, finds nothing, so just plain
        // Let's verify
        let text = plain_text(&rt);
        assert!(text.contains("is tilde"));
    }

    // -- Mixed complex --

    #[test]
    fn test_mixed_inline() {
        let rt = parse_creole("Hello **world** and //earth//");
        assert_eq!(
            rt,
            RichText::Line(vec![
                TextSpan::Plain("Hello ".into()),
                TextSpan::Bold(vec![TextSpan::Plain("world".into())]),
                TextSpan::Plain(" and ".into()),
                TextSpan::Italic(vec![TextSpan::Plain("earth".into())]),
            ])
        );
    }

    #[test]
    fn test_color_nested_bold() {
        let rt = parse_creole("<color:blue>**important**</color>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Colored {
                color: "blue".into(),
                content: vec![TextSpan::Bold(vec![TextSpan::Plain("important".into())])],
            }])
        );
    }

    // -- plain_text() extraction --

    #[test]
    fn test_plain_text_simple() {
        let rt = parse_creole("hello");
        assert_eq!(plain_text(&rt), "hello");
    }

    #[test]
    fn test_plain_text_bold() {
        let rt = parse_creole("**bold**");
        assert_eq!(plain_text(&rt), "bold");
    }

    #[test]
    fn test_plain_text_complex() {
        let rt = parse_creole("Hello **world** and //earth//");
        assert_eq!(plain_text(&rt), "Hello world and earth");
    }

    #[test]
    fn test_plain_text_multiline() {
        let rt = parse_creole("line1\nline2");
        assert_eq!(plain_text(&rt), "line1\nline2");
    }

    #[test]
    fn test_plain_text_list() {
        let rt = parse_creole("* a\n* b");
        assert_eq!(plain_text(&rt), "a\nb");
    }

    #[test]
    fn test_plain_text_table() {
        let rt = parse_creole("|= H1 |= H2 |\n| v1 | v2 |");
        let text = plain_text(&rt);
        assert!(text.contains("H1"));
        assert!(text.contains("v2"));
    }

    #[test]
    fn test_plain_text_horizontal_rule() {
        let rt = parse_creole("----");
        assert_eq!(plain_text(&rt), "----");
    }

    // -- Links --

    #[test]
    fn test_link_simple() {
        let rt = parse_creole("[[http://example.com]]");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Link {
                url: "http://example.com".into(),
                tooltip: None,
                label: None,
            }])
        );
    }

    #[test]
    fn test_link_with_label() {
        let rt = parse_creole("[[http://example.com Example Site]]");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Link {
                url: "http://example.com".into(),
                tooltip: None,
                label: Some("Example Site".into()),
            }])
        );
    }

    #[test]
    fn test_link_with_tooltip() {
        let rt = parse_creole("[[http://example.com{hover} Example Site]]");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::Link {
                url: "http://example.com".into(),
                tooltip: Some("hover".into()),
                label: Some("Example Site".into()),
            }])
        );
    }

    #[test]
    fn test_plain_text_link_with_label() {
        let rt = parse_creole("[[http://example.com Click Here]]");
        assert_eq!(plain_text(&rt), "Click Here");
    }

    #[test]
    fn test_plain_text_link_no_label() {
        let rt = parse_creole("[[http://example.com]]");
        assert_eq!(plain_text(&rt), "http://example.com");
    }

    // -- Subscript and superscript --

    #[test]
    fn test_subscript() {
        let rt = parse_creole("H<sub>2</sub>O");
        assert_eq!(
            rt,
            RichText::Line(vec![
                TextSpan::Plain("H".into()),
                TextSpan::Subscript(vec![TextSpan::Plain("2".into())]),
                TextSpan::Plain("O".into()),
            ])
        );
    }

    #[test]
    fn test_superscript() {
        let rt = parse_creole("E = mc<sup>2</sup>");
        assert_eq!(
            rt,
            RichText::Line(vec![
                TextSpan::Plain("E = mc".into()),
                TextSpan::Superscript(vec![TextSpan::Plain("2".into())]),
            ])
        );
    }

    #[test]
    fn test_subscript_plain_text() {
        let rt = parse_creole("H<sub>2</sub>O");
        assert_eq!(plain_text(&rt), "H2O");
    }

    #[test]
    fn test_superscript_plain_text() {
        let rt = parse_creole("x<sup>n</sup>");
        assert_eq!(plain_text(&rt), "xn");
    }

    // -- Back highlight --

    #[test]
    fn test_back_highlight() {
        let rt = parse_creole("<back:yellow>important</back>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::BackHighlight {
                color: "yellow".into(),
                content: vec![TextSpan::Plain("important".into())],
            }])
        );
    }

    #[test]
    fn test_back_highlight_plain_text() {
        let rt = parse_creole("<back:red>alert</back> text");
        assert_eq!(plain_text(&rt), "alert text");
    }

    // -- Font family --

    #[test]
    fn test_font_family() {
        let rt = parse_creole("<font:courier>code</font>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::FontFamily {
                family: "courier".into(),
                content: vec![TextSpan::Plain("code".into())],
            }])
        );
    }

    #[test]
    fn test_font_family_plain_text() {
        let rt = parse_creole("<font:Arial>sans</font> text");
        assert_eq!(plain_text(&rt), "sans text");
    }

    #[test]
    fn test_mixed_new_tags() {
        let rt = parse_creole("H<sub>2</sub>O is <back:yellow>water</back>");
        assert_eq!(
            rt,
            RichText::Line(vec![
                TextSpan::Plain("H".into()),
                TextSpan::Subscript(vec![TextSpan::Plain("2".into())]),
                TextSpan::Plain("O is ".into()),
                TextSpan::BackHighlight {
                    color: "yellow".into(),
                    content: vec![TextSpan::Plain("water".into())],
                },
            ])
        );
    }

    // -- Sprite references --

    #[test]
    fn test_sprite_ref_basic() {
        let rt = parse_creole("hello <$redrect> there");
        assert_eq!(
            rt,
            RichText::Line(vec![
                TextSpan::Plain("hello ".into()),
                TextSpan::InlineSvg {
                    name: "redrect".into(),
                },
                TextSpan::Plain(" there".into()),
            ])
        );
    }

    #[test]
    fn test_sprite_ref_with_scale() {
        let rt = parse_creole("<$icon,scale=2>");
        assert_eq!(
            rt,
            RichText::Line(vec![TextSpan::InlineSvg {
                name: "icon".into(),
            }])
        );
    }

    #[test]
    fn test_sprite_ref_plain_text() {
        let rt = parse_creole("text <$mySprite> more");
        assert_eq!(plain_text(&rt), "text  more");
    }
}
