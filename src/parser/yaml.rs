use log::{debug, trace, warn};

use crate::model::json_diagram::{JsonDiagram, JsonValue};
use crate::Result;

/// Parse a PlantUML YAML diagram source into a `JsonDiagram`.
///
/// Extracts the content between `@startyaml` / `@endyaml`, then parses
/// the YAML with a hand-written indentation-based parser.
pub fn parse_yaml_diagram(source: &str) -> Result<JsonDiagram> {
    let block = super::common::extract_block(source).unwrap_or_else(|| source.to_string());

    debug!("parse_yaml_diagram: block length = {}", block.len());

    let trimmed = block.trim();
    if trimmed.is_empty() {
        warn!("parse_yaml_diagram: empty YAML block");
        return Err(crate::Error::Parse {
            line: 1,
            column: Some(1),
            message: "empty YAML block".to_string(),
        });
    }

    let lines: Vec<&str> = trimmed.lines().collect();
    let mut parser = YamlParser::new(&lines);
    let root = parser.parse_root()?;

    debug!(
        "parse_yaml_diagram: parsed root type = {}",
        root.type_label()
    );
    Ok(JsonDiagram { root })
}

// ---------------------------------------------------------------------------
// Hand-written YAML parser
// ---------------------------------------------------------------------------

struct YamlParser<'a> {
    lines: &'a [&'a str],
    pos: usize,
}

impl<'a> YamlParser<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
        Self { lines, pos: 0 }
    }

    /// Returns the current line number (1-based) for error reporting.
    fn current_line_number(&self) -> usize {
        self.pos + 1
    }

    /// Create a parse error at the current position.
    fn err(&self, message: String) -> crate::Error {
        let column = self.lines.get(self.pos).map_or(1, |line| {
            let indent = line.len() - line.trim_start().len();
            indent + 1
        });
        crate::Error::Parse {
            line: self.current_line_number(),
            column: Some(column),
            message,
        }
    }

    /// Skip blank lines and comment lines (starting with #).
    fn skip_blank_and_comments(&mut self) {
        while self.pos < self.lines.len() {
            let trimmed = self.lines[self.pos].trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Compute the indentation level (number of leading spaces) for a line.
    fn indent_of(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    /// Parse the root value of the YAML document.
    fn parse_root(&mut self) -> Result<JsonValue> {
        self.skip_blank_and_comments();

        if self.pos >= self.lines.len() {
            return Err(self.err("unexpected end of YAML input".to_string()));
        }

        // Check for document start marker
        if self.lines[self.pos].trim() == "---" {
            self.pos += 1;
            self.skip_blank_and_comments();
        }

        if self.pos >= self.lines.len() {
            return Err(self.err("unexpected end of YAML input after ---".to_string()));
        }

        let trimmed = self.lines[self.pos].trim();

        // Detect root type
        if trimmed.starts_with("- ") || trimmed == "-" {
            // Root is a list
            let indent = Self::indent_of(self.lines[self.pos]);
            self.parse_list(indent)
        } else if trimmed.starts_with('[') {
            // Inline flow sequence
            self.parse_flow_value(trimmed)
        } else if trimmed.starts_with('{') {
            // Inline flow mapping
            self.parse_flow_value(trimmed)
        } else {
            // Root is a mapping
            let indent = Self::indent_of(self.lines[self.pos]);
            self.parse_mapping(indent)
        }
    }

    /// Parse a YAML mapping at the given indentation level.
    fn parse_mapping(&mut self, base_indent: usize) -> Result<JsonValue> {
        let mut entries: Vec<(String, JsonValue)> = Vec::new();

        loop {
            self.skip_blank_and_comments();

            if self.pos >= self.lines.len() {
                break;
            }

            let line = self.lines[self.pos];
            let indent = Self::indent_of(line);

            // If indentation is less than base, we've left this mapping
            if indent < base_indent {
                break;
            }

            // If indentation is greater, that's an error (unexpected indent)
            if indent > base_indent {
                break;
            }

            let trimmed = line.trim();

            // A list item at this indent means we've left the mapping
            if trimmed.starts_with("- ") || trimmed == "-" {
                break;
            }

            // Parse key: value
            if let Some((key, rest)) = split_key_value(trimmed) {
                trace!(
                    "parse_mapping: key={:?} rest={:?} at line {}",
                    key,
                    rest,
                    self.current_line_number()
                );
                self.pos += 1;

                let value = if rest.is_empty() {
                    // Value is on next line(s), could be a nested mapping or list
                    self.skip_blank_and_comments();
                    if self.pos < self.lines.len() {
                        let next_indent = Self::indent_of(self.lines[self.pos]);
                        let next_trimmed = self.lines[self.pos].trim();

                        if next_indent > base_indent {
                            if next_trimmed.starts_with("- ") || next_trimmed == "-" {
                                self.parse_list(next_indent)?
                            } else {
                                self.parse_mapping(next_indent)?
                            }
                        } else {
                            // Empty value -> null
                            JsonValue::Null
                        }
                    } else {
                        JsonValue::Null
                    }
                } else {
                    self.parse_scalar(rest)?
                };

                entries.push((key, value));
            } else {
                return Err(self.err(format!("expected key-value pair, got: {trimmed}")));
            }
        }

        Ok(JsonValue::Object(entries))
    }

    /// Parse a YAML list at the given indentation level.
    fn parse_list(&mut self, base_indent: usize) -> Result<JsonValue> {
        let mut items: Vec<JsonValue> = Vec::new();

        loop {
            self.skip_blank_and_comments();

            if self.pos >= self.lines.len() {
                break;
            }

            let line = self.lines[self.pos];
            let indent = Self::indent_of(line);

            if indent < base_indent {
                break;
            }

            if indent > base_indent {
                break;
            }

            let trimmed = line.trim();

            if !trimmed.starts_with("- ") && trimmed != "-" {
                break;
            }

            // Remove the "- " prefix
            let rest = if trimmed == "-" { "" } else { &trimmed[2..] };
            let rest = rest.trim();

            trace!(
                "parse_list: item rest={:?} at line {}",
                rest,
                self.current_line_number()
            );
            self.pos += 1;

            if rest.is_empty() {
                // Nested block under this list item
                self.skip_blank_and_comments();
                if self.pos < self.lines.len() {
                    let next_indent = Self::indent_of(self.lines[self.pos]);
                    let next_trimmed = self.lines[self.pos].trim();

                    if next_indent > base_indent {
                        if next_trimmed.starts_with("- ") || next_trimmed == "-" {
                            items.push(self.parse_list(next_indent)?);
                        } else {
                            items.push(self.parse_mapping(next_indent)?);
                        }
                    } else {
                        items.push(JsonValue::Null);
                    }
                } else {
                    items.push(JsonValue::Null);
                }
            } else if rest.contains(": ") || rest.ends_with(':') {
                // List item is an inline mapping key, e.g. "- name: Alice"
                // We need to parse this as a mapping entry, possibly followed by
                // more entries at a deeper indent
                let item_indent = base_indent + 2;
                let mut inline_entries: Vec<(String, JsonValue)> = Vec::new();

                // Parse the inline key-value
                if let Some((key, val_str)) = split_key_value(rest) {
                    let value = if val_str.is_empty() {
                        // Value on next lines
                        self.skip_blank_and_comments();
                        if self.pos < self.lines.len() {
                            let next_indent = Self::indent_of(self.lines[self.pos]);
                            let next_trimmed = self.lines[self.pos].trim();
                            if next_indent > base_indent {
                                if next_trimmed.starts_with("- ") || next_trimmed == "-" {
                                    self.parse_list(next_indent)?
                                } else {
                                    self.parse_mapping(next_indent)?
                                }
                            } else {
                                JsonValue::Null
                            }
                        } else {
                            JsonValue::Null
                        }
                    } else {
                        self.parse_scalar(val_str)?
                    };
                    inline_entries.push((key, value));
                }

                // Check for continuation entries at the item_indent level
                loop {
                    self.skip_blank_and_comments();
                    if self.pos >= self.lines.len() {
                        break;
                    }
                    let next_line = self.lines[self.pos];
                    let next_indent = Self::indent_of(next_line);
                    if next_indent < item_indent {
                        break;
                    }
                    if next_indent > item_indent {
                        break;
                    }
                    let next_trimmed = next_line.trim();
                    if next_trimmed.starts_with("- ") || next_trimmed == "-" {
                        break;
                    }
                    if let Some((key, val_str)) = split_key_value(next_trimmed) {
                        self.pos += 1;
                        let value = if val_str.is_empty() {
                            self.skip_blank_and_comments();
                            if self.pos < self.lines.len() {
                                let ni = Self::indent_of(self.lines[self.pos]);
                                let nt = self.lines[self.pos].trim();
                                if ni > item_indent {
                                    if nt.starts_with("- ") || nt == "-" {
                                        self.parse_list(ni)?
                                    } else {
                                        self.parse_mapping(ni)?
                                    }
                                } else {
                                    JsonValue::Null
                                }
                            } else {
                                JsonValue::Null
                            }
                        } else {
                            self.parse_scalar(val_str)?
                        };
                        inline_entries.push((key, value));
                    } else {
                        break;
                    }
                }

                items.push(JsonValue::Object(inline_entries));
            } else {
                // Simple scalar list item
                items.push(self.parse_scalar(rest)?);
            }
        }

        Ok(JsonValue::Array(items))
    }

    /// Parse a scalar value from a string.
    fn parse_scalar(&self, s: &str) -> Result<JsonValue> {
        let s = s.trim();

        // Flow sequence
        if s.starts_with('[') && s.ends_with(']') {
            return self.parse_flow_value(s);
        }

        // Flow mapping
        if s.starts_with('{') && s.ends_with('}') {
            return self.parse_flow_value(s);
        }

        // Null
        if s == "null" || s == "~" || s.is_empty() {
            return Ok(JsonValue::Null);
        }

        // Boolean
        if s == "true"
            || s == "True"
            || s == "TRUE"
            || s == "yes"
            || s == "Yes"
            || s == "YES"
            || s == "on"
            || s == "On"
            || s == "ON"
        {
            return Ok(JsonValue::Bool(true));
        }
        if s == "false"
            || s == "False"
            || s == "FALSE"
            || s == "no"
            || s == "No"
            || s == "NO"
            || s == "off"
            || s == "Off"
            || s == "OFF"
        {
            return Ok(JsonValue::Bool(false));
        }

        // Number
        if let Ok(n) = s.parse::<f64>() {
            // Make sure it's a valid number representation (not NaN/Inf from YAML words)
            if n.is_finite() {
                // Verify it looks like a number, not a random string that parses as f64
                let first = s.as_bytes()[0];
                if first == b'-' || first == b'+' || first.is_ascii_digit() || first == b'.' {
                    return Ok(JsonValue::Number(n));
                }
            }
        }

        // Quoted string (single or double)
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            let inner = &s[1..s.len() - 1];
            return Ok(JsonValue::Str(unescape_string(inner)));
        }

        // Unquoted string
        Ok(JsonValue::Str(s.to_string()))
    }

    /// Parse a flow-style value (inline JSON-like syntax).
    fn parse_flow_value(&self, s: &str) -> Result<JsonValue> {
        let s = s.trim();

        if s.starts_with('[') && s.ends_with(']') {
            let inner = s[1..s.len() - 1].trim();
            if inner.is_empty() {
                return Ok(JsonValue::Array(vec![]));
            }
            let parts = split_flow_items(inner);
            let mut items = Vec::new();
            for part in parts {
                items.push(self.parse_scalar(part.trim())?);
            }
            return Ok(JsonValue::Array(items));
        }

        if s.starts_with('{') && s.ends_with('}') {
            let inner = s[1..s.len() - 1].trim();
            if inner.is_empty() {
                return Ok(JsonValue::Object(vec![]));
            }
            let parts = split_flow_items(inner);
            let mut entries = Vec::new();
            for part in parts {
                let part = part.trim();
                if let Some((key, val)) = split_key_value(part) {
                    entries.push((key, self.parse_scalar(val)?));
                }
            }
            return Ok(JsonValue::Object(entries));
        }

        self.parse_scalar(s)
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Split a "key: value" line into (key, value_str).
/// Returns None if no colon is found.
fn split_key_value(line: &str) -> Option<(String, &str)> {
    // Handle quoted keys
    if let Some(after_dq) = line.strip_prefix('"') {
        if let Some(end_quote) = after_dq.find('"') {
            let key = &after_dq[..end_quote];
            let rest = &after_dq[end_quote + 1..]; // skip closing quote
            let rest = rest.trim_start();
            if let Some(after_colon) = rest.strip_prefix(':') {
                let val = after_colon.trim();
                return Some((key.to_string(), val));
            }
        }
        return None;
    }
    if let Some(after_sq) = line.strip_prefix('\'') {
        if let Some(end_quote) = after_sq.find('\'') {
            let key = &after_sq[..end_quote];
            let rest = &after_sq[end_quote + 1..];
            let rest = rest.trim_start();
            if let Some(after_colon) = rest.strip_prefix(':') {
                let val = after_colon.trim();
                return Some((key.to_string(), val));
            }
        }
        return None;
    }

    // Unquoted key: find the first ": " or trailing ":"
    if let Some(colon_pos) = line.find(':') {
        let key = line[..colon_pos].trim();
        let val = line[colon_pos + 1..].trim();
        if key.is_empty() {
            return None;
        }
        Some((key.to_string(), val))
    } else {
        None
    }
}

/// Unescape a quoted string (basic escape sequences).
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Split flow-style items by commas, respecting nesting of brackets/braces.
fn split_flow_items(s: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, ch) in s.char_indices() {
        match ch {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                items.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    items.push(&s[start..]);
    items
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::json_diagram::JsonValue;

    // -- Helper --

    fn parse(src: &str) -> JsonDiagram {
        parse_yaml_diagram(src).expect("parse failed")
    }

    fn parse_wrapped(yaml: &str) -> JsonDiagram {
        let src = format!("@startyaml\n{}\n@endyaml", yaml);
        parse_yaml_diagram(&src).expect("parse failed")
    }

    // 1. Simple key-value mapping
    #[test]
    fn test_simple_mapping() {
        let yd = parse("name: Alice\nage: 30");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "name");
                assert_eq!(entries[0].1, JsonValue::Str("Alice".into()));
                assert_eq!(entries[1].0, "age");
                assert_eq!(entries[1].1, JsonValue::Number(30.0));
            }
            _ => panic!("expected object"),
        }
    }

    // 2. Nested mapping
    #[test]
    fn test_nested_mapping() {
        let yd = parse("person:\n  name: Alice\n  age: 30");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "person");
                match &entries[0].1 {
                    JsonValue::Object(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner[0].0, "name");
                        assert_eq!(inner[0].1, JsonValue::Str("Alice".into()));
                    }
                    _ => panic!("expected inner object"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    // 3. Simple list
    #[test]
    fn test_simple_list() {
        let yd = parse("- apple\n- banana\n- cherry");
        match &yd.root {
            JsonValue::Array(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], JsonValue::Str("apple".into()));
                assert_eq!(items[1], JsonValue::Str("banana".into()));
                assert_eq!(items[2], JsonValue::Str("cherry".into()));
            }
            _ => panic!("expected array"),
        }
    }

    // 4. List with numbers
    #[test]
    fn test_list_numbers() {
        let yd = parse("- 1\n- 2.5\n- -3");
        match &yd.root {
            JsonValue::Array(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], JsonValue::Number(1.0));
                assert_eq!(items[1], JsonValue::Number(2.5));
                assert_eq!(items[2], JsonValue::Number(-3.0));
            }
            _ => panic!("expected array"),
        }
    }

    // 5. Boolean values
    #[test]
    fn test_booleans() {
        let yd = parse("a: true\nb: false\nc: yes\nd: no");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Bool(true));
                assert_eq!(entries[1].1, JsonValue::Bool(false));
                assert_eq!(entries[2].1, JsonValue::Bool(true));
                assert_eq!(entries[3].1, JsonValue::Bool(false));
            }
            _ => panic!("expected object"),
        }
    }

    // 6. Null values
    #[test]
    fn test_null_values() {
        let yd = parse("a: null\nb: ~\nc:");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Null);
                assert_eq!(entries[1].1, JsonValue::Null);
                assert_eq!(entries[2].1, JsonValue::Null);
            }
            _ => panic!("expected object"),
        }
    }

    // 7. Quoted strings
    #[test]
    fn test_quoted_strings() {
        let yd = parse("a: \"hello world\"\nb: 'single quoted'");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Str("hello world".into()));
                assert_eq!(entries[1].1, JsonValue::Str("single quoted".into()));
            }
            _ => panic!("expected object"),
        }
    }

    // 8. PlantUML wrapped YAML
    #[test]
    fn test_plantuml_wrapped() {
        let yd = parse_wrapped("name: Alice\nage: 30");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "name");
            }
            _ => panic!("expected object"),
        }
    }

    // 9. Mapping with list value
    #[test]
    fn test_mapping_with_list_value() {
        let yd = parse("fruits:\n  - apple\n  - banana\ncount: 2");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "fruits");
                match &entries[0].1 {
                    JsonValue::Array(items) => {
                        assert_eq!(items.len(), 2);
                        assert_eq!(items[0], JsonValue::Str("apple".into()));
                        assert_eq!(items[1], JsonValue::Str("banana".into()));
                    }
                    _ => panic!("expected array for fruits"),
                }
                assert_eq!(entries[1].0, "count");
                assert_eq!(entries[1].1, JsonValue::Number(2.0));
            }
            _ => panic!("expected object"),
        }
    }

    // 10. Deeply nested structure
    #[test]
    fn test_deeply_nested() {
        let yd = parse("a:\n  b:\n    c:\n      d: 42");
        match &yd.root {
            JsonValue::Object(l1) => match &l1[0].1 {
                JsonValue::Object(l2) => match &l2[0].1 {
                    JsonValue::Object(l3) => match &l3[0].1 {
                        JsonValue::Object(l4) => {
                            assert_eq!(l4[0].1, JsonValue::Number(42.0));
                        }
                        _ => panic!("expected l4 object"),
                    },
                    _ => panic!("expected l3 object"),
                },
                _ => panic!("expected l2 object"),
            },
            _ => panic!("expected root object"),
        }
    }

    // 11. Empty input error
    #[test]
    fn test_empty_input_error() {
        let result = parse_yaml_diagram("@startyaml\n@endyaml");
        assert!(result.is_err());
    }

    // 12. Comments are ignored
    #[test]
    fn test_comments_ignored() {
        let yd = parse("# This is a comment\nname: Alice\n# Another comment\nage: 30");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "name");
                assert_eq!(entries[1].0, "age");
            }
            _ => panic!("expected object"),
        }
    }

    // 13. Document start marker
    #[test]
    fn test_document_start_marker() {
        let yd = parse("---\nname: Alice\nage: 30");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "name");
            }
            _ => panic!("expected object"),
        }
    }

    // 14. Flow sequence (inline list)
    #[test]
    fn test_flow_sequence() {
        let yd = parse("items: [1, 2, 3]");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].0, "items");
                match &entries[0].1 {
                    JsonValue::Array(items) => {
                        assert_eq!(items.len(), 3);
                        assert_eq!(items[0], JsonValue::Number(1.0));
                        assert_eq!(items[1], JsonValue::Number(2.0));
                        assert_eq!(items[2], JsonValue::Number(3.0));
                    }
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    // 15. Flow mapping (inline object)
    #[test]
    fn test_flow_mapping() {
        let yd = parse("point: {x: 1, y: 2}");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].0, "point");
                match &entries[0].1 {
                    JsonValue::Object(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner[0].0, "x");
                        assert_eq!(inner[0].1, JsonValue::Number(1.0));
                        assert_eq!(inner[1].0, "y");
                        assert_eq!(inner[1].1, JsonValue::Number(2.0));
                    }
                    _ => panic!("expected inner object"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    // 16. List of mappings
    #[test]
    fn test_list_of_mappings() {
        let yd = parse("- name: Alice\n  age: 30\n- name: Bob\n  age: 25");
        match &yd.root {
            JsonValue::Array(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    JsonValue::Object(e) => {
                        assert_eq!(e[0].0, "name");
                        assert_eq!(e[0].1, JsonValue::Str("Alice".into()));
                        assert_eq!(e[1].0, "age");
                        assert_eq!(e[1].1, JsonValue::Number(30.0));
                    }
                    _ => panic!("expected object in list"),
                }
                match &items[1] {
                    JsonValue::Object(e) => {
                        assert_eq!(e[0].0, "name");
                        assert_eq!(e[0].1, JsonValue::Str("Bob".into()));
                    }
                    _ => panic!("expected object in list"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    // 17. Mixed types in mapping
    #[test]
    fn test_mixed_types() {
        let yd = parse("str: hello\nnum: 42\nbool: true\nnull_val: null");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Str("hello".into()));
                assert_eq!(entries[1].1, JsonValue::Number(42.0));
                assert_eq!(entries[2].1, JsonValue::Bool(true));
                assert_eq!(entries[3].1, JsonValue::Null);
            }
            _ => panic!("expected object"),
        }
    }

    // 18. String with special YAML characters
    #[test]
    fn test_string_with_colon() {
        let yd = parse("message: \"hello: world\"");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Str("hello: world".into()));
            }
            _ => panic!("expected object"),
        }
    }

    // 19. Boolean edge cases (yes/no/on/off)
    #[test]
    fn test_boolean_variants() {
        let yd = parse("a: Yes\nb: No\nc: on\nd: off\ne: TRUE\nf: FALSE");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Bool(true));
                assert_eq!(entries[1].1, JsonValue::Bool(false));
                assert_eq!(entries[2].1, JsonValue::Bool(true));
                assert_eq!(entries[3].1, JsonValue::Bool(false));
                assert_eq!(entries[4].1, JsonValue::Bool(true));
                assert_eq!(entries[5].1, JsonValue::Bool(false));
            }
            _ => panic!("expected object"),
        }
    }

    // 20. Tilde as null
    #[test]
    fn test_tilde_null() {
        let yd = parse("val: ~");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Null);
            }
            _ => panic!("expected object"),
        }
    }

    // 21. Empty flow collections
    #[test]
    fn test_empty_flow_collections() {
        let yd = parse("list: []\nmap: {}");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Array(vec![]));
                assert_eq!(entries[1].1, JsonValue::Object(vec![]));
            }
            _ => panic!("expected object"),
        }
    }

    // 22. Float number
    #[test]
    fn test_float_number() {
        let yd = parse("pi: 3.15159");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Number(3.15159));
            }
            _ => panic!("expected object"),
        }
    }

    // 23. Complex nested structure with lists and mappings
    #[test]
    fn test_complex_structure() {
        let src = "database:\n  host: localhost\n  port: 5432\n  tables:\n    - users\n    - orders\ndebug: false";
        let yd = parse(src);
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "database");
                match &entries[0].1 {
                    JsonValue::Object(db) => {
                        assert_eq!(db.len(), 3);
                        assert_eq!(db[0].0, "host");
                        assert_eq!(db[0].1, JsonValue::Str("localhost".into()));
                        assert_eq!(db[1].0, "port");
                        assert_eq!(db[1].1, JsonValue::Number(5432.0));
                        assert_eq!(db[2].0, "tables");
                        match &db[2].1 {
                            JsonValue::Array(tables) => {
                                assert_eq!(tables.len(), 2);
                            }
                            _ => panic!("expected array for tables"),
                        }
                    }
                    _ => panic!("expected object for database"),
                }
                assert_eq!(entries[1].0, "debug");
                assert_eq!(entries[1].1, JsonValue::Bool(false));
            }
            _ => panic!("expected root object"),
        }
    }

    // 24. Escaped string content
    #[test]
    fn test_escaped_string() {
        let yd = parse("msg: \"hello\\nworld\"");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Str("hello\nworld".into()));
            }
            _ => panic!("expected object"),
        }
    }

    // 25. Negative float
    #[test]
    fn test_negative_float() {
        let yd = parse("temp: -12.5");
        match &yd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Number(-12.5));
            }
            _ => panic!("expected object"),
        }
    }
}
