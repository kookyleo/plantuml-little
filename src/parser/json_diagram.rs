use log::{debug, trace, warn};

use crate::model::json_diagram::{JsonDiagram, JsonValue};
use crate::Result;

/// Parse a PlantUML JSON diagram source into a `JsonDiagram`.
///
/// Extracts the content between `@startjson` / `@endjson`, then parses
/// the JSON with a hand-written recursive descent parser.
pub fn parse_json_diagram(source: &str) -> Result<JsonDiagram> {
    let block = super::common::extract_block(source).unwrap_or_else(|| source.to_string());

    debug!("parse_json_diagram: block length = {}", block.len());

    let trimmed = block.trim();
    if trimmed.is_empty() {
        warn!("parse_json_diagram: empty JSON block");
        return Err(crate::Error::Parse {
            line: 1,
            column: Some(1),
            message: "empty JSON block".to_string(),
        });
    }

    let mut parser = JsonParser::new(trimmed);
    let root = parser.parse_value()?;

    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        warn!(
            "parse_json_diagram: trailing content at position {}",
            parser.pos
        );
    }

    debug!(
        "parse_json_diagram: parsed root type = {}",
        root.type_label()
    );
    Ok(JsonDiagram { root })
}

// ---------------------------------------------------------------------------
// Recursive descent JSON parser
// ---------------------------------------------------------------------------

struct JsonParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    /// Peek at the current byte without consuming it.
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    /// Advance the cursor by one byte and return it.
    fn advance(&mut self) -> Option<u8> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    /// Skip whitespace characters (space, tab, CR, LF).
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                _ => break,
            }
        }
    }

    /// Consume the expected byte or return an error.
    fn expect(&mut self, expected: u8) -> Result<()> {
        self.skip_whitespace();
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(self.err(format!(
                "expected '{}', found '{}'",
                expected as char, ch as char
            ))),
            None => Err(self.err(format!(
                "expected '{}', found end of input",
                expected as char
            ))),
        }
    }

    /// Create a parse error at the current position.
    fn err(&self, message: String) -> crate::Error {
        // Compute a rough line number from current position
        let line = self.input[..self.pos.min(self.input.len())]
            .iter()
            .filter(|&&b| b == b'\n')
            .count()
            + 1;
        let column = {
            let line_start = self.input[..self.pos.min(self.input.len())]
                .iter()
                .rposition(|&b| b == b'\n')
                .map_or(0, |p| p + 1);
            self.pos.saturating_sub(line_start) + 1
        };
        crate::Error::Parse {
            line,
            column: Some(column),
            message,
        }
    }

    // -----------------------------------------------------------------------
    // Value parsing
    // -----------------------------------------------------------------------

    fn parse_value(&mut self) -> Result<JsonValue> {
        self.skip_whitespace();

        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(JsonValue::Str),
            Some(b't' | b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(ch) if ch == b'-' || ch.is_ascii_digit() => self.parse_number(),
            Some(ch) => Err(self.err(format!("unexpected character '{}'", ch as char))),
            None => Err(self.err("unexpected end of input".to_string())),
        }
    }

    // -----------------------------------------------------------------------
    // Object: { "key": value, ... }
    // -----------------------------------------------------------------------

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.expect(b'{')?;
        trace!("parse_object: start at pos {}", self.pos);

        let mut entries: Vec<(String, JsonValue)> = Vec::new();

        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.advance();
            return Ok(JsonValue::Object(entries));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.expect(b':')?;
            let value = self.parse_value()?;

            trace!("parse_object: key = {key:?}");
            entries.push((key, value));

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b'}') => {
                    self.advance();
                    break;
                }
                _ => return Err(self.err("expected ',' or '}' in object".to_string())),
            }
        }

        Ok(JsonValue::Object(entries))
    }

    // -----------------------------------------------------------------------
    // Array: [ value, ... ]
    // -----------------------------------------------------------------------

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.expect(b'[')?;
        trace!("parse_array: start at pos {}", self.pos);

        let mut items: Vec<JsonValue> = Vec::new();

        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.advance();
            return Ok(JsonValue::Array(items));
        }

        loop {
            let value = self.parse_value()?;
            items.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b']') => {
                    self.advance();
                    break;
                }
                _ => return Err(self.err("expected ',' or ']' in array".to_string())),
            }
        }

        Ok(JsonValue::Array(items))
    }

    // -----------------------------------------------------------------------
    // String: "..."  with escape handling
    // -----------------------------------------------------------------------

    fn parse_string(&mut self) -> Result<String> {
        self.skip_whitespace();
        self.expect(b'"')?;

        let mut result = String::new();

        loop {
            match self.advance() {
                Some(b'\\') => {
                    match self.advance() {
                        Some(b'"') => result.push('"'),
                        Some(b'\\') => result.push('\\'),
                        Some(b'/') => result.push('/'),
                        Some(b'b') => result.push('\u{0008}'),
                        Some(b'f') => result.push('\u{000C}'),
                        Some(b'n') => result.push('\n'),
                        Some(b'r') => result.push('\r'),
                        Some(b't') => result.push('\t'),
                        Some(b'u') => {
                            let hex = self.read_hex4()?;
                            if let Some(ch) = char::from_u32(hex) {
                                result.push(ch);
                            } else {
                                result.push('\u{FFFD}');
                            }
                        }
                        Some(ch) => {
                            // Lenient: keep the backslash and the character
                            result.push('\\');
                            result.push(ch as char);
                        }
                        None => return Err(self.err("unterminated escape sequence".to_string())),
                    }
                }
                Some(b'"') => break,
                Some(ch) => result.push(ch as char),
                None => return Err(self.err("unterminated string".to_string())),
            }
        }

        Ok(result)
    }

    /// Read exactly 4 hex digits for a \uXXXX escape.
    fn read_hex4(&mut self) -> Result<u32> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            match self.advance() {
                Some(ch) if ch.is_ascii_hexdigit() => {
                    value = value * 16
                        + match ch {
                            b'0'..=b'9' => (ch - b'0') as u32,
                            b'a'..=b'f' => (ch - b'a' + 10) as u32,
                            b'A'..=b'F' => (ch - b'A' + 10) as u32,
                            _ => unreachable!(),
                        };
                }
                _ => return Err(self.err("invalid \\uXXXX escape".to_string())),
            }
        }
        Ok(value)
    }

    // -----------------------------------------------------------------------
    // Number: integer or floating-point
    // -----------------------------------------------------------------------

    fn parse_number(&mut self) -> Result<JsonValue> {
        let start = self.pos;

        // Optional leading minus
        if self.peek() == Some(b'-') {
            self.advance();
        }

        // Integer part
        if self.peek() == Some(b'0') {
            self.advance();
        } else {
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                return Err(self.err("expected digit in number".to_string()));
            }
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.advance();
            }
        }

        // Fractional part
        if self.peek() == Some(b'.') {
            self.advance();
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                return Err(self.err("expected digit after decimal point".to_string()));
            }
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.advance();
            }
        }

        // Exponent part
        if self.peek().is_some_and(|b| b == b'e' || b == b'E') {
            self.advance();
            if self.peek().is_some_and(|b| b == b'+' || b == b'-') {
                self.advance();
            }
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                return Err(self.err("expected digit in exponent".to_string()));
            }
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.advance();
            }
        }

        let num_str = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| self.err("invalid UTF-8 in number".to_string()))?;

        let n: f64 = num_str
            .parse()
            .map_err(|_| self.err(format!("invalid number: {num_str}")))?;

        trace!("parse_number: {num_str} -> {n}");
        Ok(JsonValue::Number(n))
    }

    // -----------------------------------------------------------------------
    // Bool: true / false
    // -----------------------------------------------------------------------

    fn parse_bool(&mut self) -> Result<JsonValue> {
        if self.starts_with(b"true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.starts_with(b"false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.err("expected 'true' or 'false'".to_string()))
        }
    }

    // -----------------------------------------------------------------------
    // Null
    // -----------------------------------------------------------------------

    fn parse_null(&mut self) -> Result<JsonValue> {
        if self.starts_with(b"null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(self.err("expected 'null'".to_string()))
        }
    }

    /// Check if the input at the current position starts with the given bytes.
    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.input[self.pos..].starts_with(prefix)
    }
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
        parse_json_diagram(src).expect("parse failed")
    }

    // 1. Simple object with string value
    #[test]
    fn test_simple_object() {
        let jd = parse(r#"{"name": "Alice"}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "name");
                assert_eq!(entries[0].1, JsonValue::Str("Alice".into()));
            }
            _ => panic!("expected object"),
        }
    }

    // 2. Empty object
    #[test]
    fn test_empty_object() {
        let jd = parse("{}");
        assert_eq!(jd.root, JsonValue::Object(vec![]));
    }

    // 3. Empty array
    #[test]
    fn test_empty_array() {
        let jd = parse("[]");
        assert_eq!(jd.root, JsonValue::Array(vec![]));
    }

    // 4. Boolean values
    #[test]
    fn test_booleans() {
        let jd = parse(r#"{"a": true, "b": false}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Bool(true));
                assert_eq!(entries[1].1, JsonValue::Bool(false));
            }
            _ => panic!("expected object"),
        }
    }

    // 5. Null value
    #[test]
    fn test_null() {
        let jd = parse(r#"{"x": null}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Null);
            }
            _ => panic!("expected object"),
        }
    }

    // 6. Numbers (integer and float)
    #[test]
    fn test_numbers() {
        let jd = parse(r#"{"int": 42, "neg": -7, "float": 3.15, "exp": 1e10}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Number(42.0));
                assert_eq!(entries[1].1, JsonValue::Number(-7.0));
                assert_eq!(entries[2].1, JsonValue::Number(3.15));
                assert_eq!(entries[3].1, JsonValue::Number(1e10));
            }
            _ => panic!("expected object"),
        }
    }

    // 7. Nested objects
    #[test]
    fn test_nested_object() {
        let jd = parse(r#"{"outer": {"inner": "value"}}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 1);
                match &entries[0].1 {
                    JsonValue::Object(inner) => {
                        assert_eq!(inner[0].0, "inner");
                        assert_eq!(inner[0].1, JsonValue::Str("value".into()));
                    }
                    _ => panic!("expected inner object"),
                }
            }
            _ => panic!("expected object"),
        }
    }

    // 8. Array with mixed types
    #[test]
    fn test_array_mixed() {
        let jd = parse(r#"[1, "two", true, null, [3]]"#);
        match &jd.root {
            JsonValue::Array(items) => {
                assert_eq!(items.len(), 5);
                assert_eq!(items[0], JsonValue::Number(1.0));
                assert_eq!(items[1], JsonValue::Str("two".into()));
                assert_eq!(items[2], JsonValue::Bool(true));
                assert_eq!(items[3], JsonValue::Null);
                assert!(matches!(items[4], JsonValue::Array(_)));
            }
            _ => panic!("expected array"),
        }
    }

    // 9. String escapes
    #[test]
    fn test_string_escapes() {
        let jd = parse(r#"{"s": "a\nb\tc\\d\"e\/f"}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Str("a\nb\tc\\d\"e/f".into()));
            }
            _ => panic!("expected object"),
        }
    }

    // 10. Unicode escape
    #[test]
    fn test_unicode_escape() {
        let jd = parse(r#"{"emoji": "\u0041\u0042"}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Str("AB".into()));
            }
            _ => panic!("expected object"),
        }
    }

    // 11. PlantUML wrapped JSON
    #[test]
    fn test_plantuml_wrapped() {
        let src = "@startjson\n{\"key\": \"value\"}\n@endjson\n";
        let jd = parse_json_diagram(src).unwrap();
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "key");
            }
            _ => panic!("expected object"),
        }
    }

    // 12. Complex fixture (json_escaped.puml pattern)
    #[test]
    fn test_complex_fixture() {
        let src = r#"@startjson
{
    "a": true,
    "desc": "a\\nb\\nc",
    "required": [
        "r1",
        "r2"
    ],
    "addP": false,
    "properties": {
        "P": "{ ... }"
    }
}
@endjson"#;

        let jd = parse_json_diagram(src).unwrap();
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries.len(), 5);
                assert_eq!(entries[0].0, "a");
                assert_eq!(entries[0].1, JsonValue::Bool(true));
                assert_eq!(entries[1].0, "desc");
                // "a\\nb\\nc" in JSON source => a\nb\nc as string
                assert_eq!(entries[1].1, JsonValue::Str("a\\nb\\nc".into()));
                assert_eq!(entries[2].0, "required");
                match &entries[2].1 {
                    JsonValue::Array(items) => assert_eq!(items.len(), 2),
                    _ => panic!("expected array"),
                }
                assert_eq!(entries[3].0, "addP");
                assert_eq!(entries[3].1, JsonValue::Bool(false));
            }
            _ => panic!("expected object"),
        }
    }

    // 13. Error on empty input
    #[test]
    fn test_empty_input_error() {
        let result = parse_json_diagram("@startjson\n@endjson");
        assert!(result.is_err());
    }

    // 14. Error on invalid JSON
    #[test]
    fn test_invalid_json() {
        let result = parse_json_diagram("{invalid}");
        assert!(result.is_err());
    }

    // 15. Deeply nested structure
    #[test]
    fn test_deeply_nested() {
        let jd = parse(r#"{"a": {"b": {"c": {"d": 42}}}}"#);
        match &jd.root {
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

    // 16. Number zero
    #[test]
    fn test_number_zero() {
        let jd = parse(r#"{"z": 0}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Number(0.0));
            }
            _ => panic!("expected object"),
        }
    }

    // 17. Top-level array
    #[test]
    fn test_top_level_array() {
        let jd = parse(r#"[1, 2, 3]"#);
        match &jd.root {
            JsonValue::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("expected array"),
        }
    }

    // 18. String with only escapes
    #[test]
    fn test_escaped_backslashes() {
        // JSON: "a\\b" means the string a\b
        let jd = parse(r#"{"v": "a\\b"}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                assert_eq!(entries[0].1, JsonValue::Str("a\\b".into()));
            }
            _ => panic!("expected object"),
        }
    }

    // 19. Escaped newlines (\\n in JSON → literal \n in value)
    #[test]
    fn test_escaped_newline_in_string() {
        // JSON: "a\\nb\\nc" → parsed string: a\nb\nc (literal backslash-n)
        let jd = parse(r#"{"desc": "a\\nb\\nc\\nd\\ne\\nf"}"#);
        match &jd.root {
            JsonValue::Object(entries) => {
                let val = match &entries[0].1 {
                    JsonValue::Str(s) => s.clone(),
                    _ => panic!("expected string"),
                };
                // Should be literal \n (two chars: backslash + n)
                assert_eq!(val, "a\\nb\\nc\\nd\\ne\\nf");
                assert_eq!(
                    val.split("\\n").count(),
                    6,
                    "Should split into 6 parts on literal \\n"
                );
            }
            _ => panic!("expected object"),
        }
    }
}
