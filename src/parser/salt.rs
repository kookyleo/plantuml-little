//! Salt diagram parser — mirrors Java's `DataSourceImpl` + `Positionner2` +
//! `ElementFactory*` pipeline so that the resulting grid layout matches Java
//! byte-for-byte.
//!
//! The Java flow is:
//! 1. `DataSourceImpl` tokenises each line on `|` and `}`. Non-`|`/`}` tokens
//!    become "terminated" items; the terminator is `NEWCOL` when a `|` follows
//!    on the same line, `NEWLINE` otherwise.
//! 2. `ElementFactoryPyramid` consumes tokens until the matching `}`. A nested
//!    `{` opens a recursive pyramid.
//! 3. For each token, simple factories try to produce an element (Button,
//!    TextField, Checkbox, …). The `Positionner2` places the element at the
//!    current (row, col) and then advances: `col++` on `NEWCOL`, `row++;col=0`
//!    on `NEWLINE`.
//!
//! Because all terminators come from `|}` the positioner never advances rows
//! for lines ending with `|` — that's why `{#| a | b |\n| c | d |}` collapses
//! into four cells on a single row (matching Java observed behaviour).

use crate::model::salt::{SaltCell, SaltDiagram, SaltElement, SaltPyramid, TableStrategy};
use crate::Result;

/// Terminator between tokens. Java: `Terminator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    NewCol,
    NewLine,
}

#[derive(Debug, Clone)]
struct Token {
    text: String,
    terminator: Terminator,
}

/// Tokenise lines the way Java's `DataSourceImpl` does. For each line, split on
/// the delimiters `|` and `}` while preserving those delimiters; drop `|`
/// tokens (they act as separators) and attach a `NEWCOL` terminator when the
/// next token on the same line is `|`, otherwise `NEWLINE`.
fn tokenise(lines: &[&str]) -> Vec<Token> {
    let mut result: Vec<Token> = Vec::new();
    for line in lines {
        // StringTokenizer("|}", true) returns both tokens and delimiters.
        let mut raw: Vec<String> = Vec::new();
        let mut current = String::new();
        for ch in line.chars() {
            if ch == '|' || ch == '}' {
                if !current.is_empty() {
                    raw.push(std::mem::take(&mut current));
                }
                raw.push(ch.to_string());
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            raw.push(current);
        }

        // Emit tokens skipping bare `|` separators; track whether more tokens
        // remain on this line to determine the terminator.
        let mut idx = 0;
        while idx < raw.len() {
            let token = raw[idx].trim();
            idx += 1;
            if token == "|" {
                continue;
            }
            // hasMoreTokens flag = more non-nothing tokens exist after this one
            let has_more = idx < raw.len();
            let terminator = if has_more {
                Terminator::NewCol
            } else {
                Terminator::NewLine
            };
            // Handle nested `{...}`/`{#...}` embedded in the middle of a token
            // (Java uses `STRUCTURED_BLOCK_START_PATTERN` for this). For salt
            // content we support standalone block headers on their own line;
            // tokens that look like `{#`, `{`, `}` are kept as-is.
            if token.is_empty() {
                continue;
            }
            result.push(Token {
                text: token.to_string(),
                terminator,
            });
        }
    }
    result
}

/// Streaming token cursor used by the factory.
struct Cursor {
    tokens: Vec<Token>,
    pos: usize,
}

impl Cursor {
    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned()?;
        self.pos += 1;
        Some(tok)
    }
}

/// Positioner mirroring Java `Positionner2`: places cells, advances (row, col).
struct Positioner {
    row: usize,
    col: usize,
    cells: Vec<SaltCell>,
    max_row: usize,
    max_col: usize,
}

impl Positioner {
    fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            cells: Vec::new(),
            max_row: 0,
            max_col: 0,
        }
    }

    fn add(&mut self, element: SaltElement, terminator: Terminator) {
        let cell = SaltCell::new(self.row, self.col, element);
        self.cells.push(cell);
        self.update_max();
        match terminator {
            Terminator::NewCol => self.col += 1,
            Terminator::NewLine => {
                self.row += 1;
                self.col = 0;
            }
        }
    }

    fn update_max(&mut self) {
        if self.row > self.max_row {
            self.max_row = self.row;
        }
        if self.col > self.max_col {
            self.max_col = self.col;
        }
    }

    fn nb_rows(&self) -> usize {
        self.max_row + 1
    }

    fn nb_cols(&self) -> usize {
        self.max_col + 1
    }
}

/// Extract the salt source block. Supports `@startsalt/@endsalt` (standalone
/// salt diagram) and inline `salt` inside `@startuml/@enduml`.
fn extract_salt_block(source: &str) -> (String, bool) {
    let mut inside = false;
    let mut lines: Vec<&str> = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if !inside {
            if trimmed.starts_with("@startsalt") {
                inside = true;
            }
            continue;
        }
        if trimmed.starts_with("@endsalt") || trimmed.starts_with("@end") {
            break;
        }
        lines.push(line);
    }
    if !lines.is_empty() {
        return (lines.join("\n"), false);
    }
    // Inline salt (salt keyword inside @startuml).
    let mut saw_salt = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if !saw_salt {
            if trimmed == "salt" {
                saw_salt = true;
            }
            continue;
        }
        if trimmed.starts_with("@end") {
            break;
        }
        lines.push(line);
    }
    (lines.join("\n"), true)
}

pub fn parse_salt_diagram(source: &str) -> Result<SaltDiagram> {
    let (block, is_inline) = extract_salt_block(source);
    // %newline() places U+E100 in preprocessor output; salt tokenises on real
    // newlines, so expand placeholders before splitting.
    let block = block.replace(crate::NEWLINE_CHAR, "\n");
    let raw_lines: Vec<&str> = block.lines().collect();
    // Drop blank/comment lines before tokenising to match Java behaviour.
    let filtered: Vec<&str> = raw_lines
        .into_iter()
        .filter(|line| {
            let t = line.trim();
            !t.is_empty() && !t.starts_with('\'')
        })
        .collect();

    let tokens = tokenise(&filtered);
    let mut cursor = Cursor { tokens, pos: 0 };

    // Salt diagrams must start with a block header.
    let root = parse_pyramid(&mut cursor)?;
    Ok(SaltDiagram {
        root: SaltElement::Pyramid(root),
        is_inline,
    })
}

/// Parse a pyramid (`{`, `{#`, … until matching `}`).
fn parse_pyramid(cursor: &mut Cursor) -> Result<SaltPyramid> {
    // Expect a block header token.
    let header = cursor.next().map(|t| t.text).unwrap_or_default();
    let strategy = match header.as_str() {
        "{#" => TableStrategy::DrawAll,
        _ => TableStrategy::DrawNone,
    };

    let mut positioner = Positioner::new();

    while let Some(tok) = cursor.peek(0) {
        if tok.text == "}" {
            cursor.next();
            break;
        }
        if is_block_start(&tok.text) {
            // Nested pyramid — consume recursively.
            let saved_terminator = tok.terminator;
            let nested = parse_pyramid(cursor)?;
            positioner.add(SaltElement::Pyramid(nested), saved_terminator);
            continue;
        }
        let tok = cursor.next().unwrap();
        let element = build_element(&tok.text);
        positioner.add(element, tok.terminator);
    }

    let rows = positioner.nb_rows();
    let cols = positioner.nb_cols();
    Ok(SaltPyramid {
        cells: positioner.cells,
        rows,
        cols,
        strategy,
    })
}

fn is_block_start(text: &str) -> bool {
    matches!(text, "{" | "{#" | "{+" | "{^" | "{!" | "{-")
}

/// Build a concrete `SaltElement` from a token string. Mirrors Java's simple
/// factory dispatch order (TextField → Checkbox → Button → Radio → Text).
fn build_element(text: &str) -> SaltElement {
    // TextField: quoted string.
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return SaltElement::TextField(text[1..text.len() - 1].to_string());
    }
    // Checkbox: [X] / [x] / [ ] followed by label.
    if let Some(rest) = text.strip_prefix("[X]") {
        return SaltElement::Checkbox {
            label: rest.trim().to_string(),
            checked: true,
        };
    }
    if let Some(rest) = text.strip_prefix("[x]") {
        return SaltElement::Checkbox {
            label: rest.trim().to_string(),
            checked: true,
        };
    }
    if let Some(rest) = text.strip_prefix("[ ]") {
        return SaltElement::Checkbox {
            label: rest.trim().to_string(),
            checked: false,
        };
    }
    // Button: [label]
    if text.starts_with('[') && text.ends_with(']') && text.len() >= 2 {
        return SaltElement::Button(text[1..text.len() - 1].trim().to_string());
    }
    // Radio: (X) / (x) / ( ) followed by label.
    if let Some(rest) = text.strip_prefix("(X)") {
        return SaltElement::Radio {
            label: rest.trim().to_string(),
            selected: true,
        };
    }
    if let Some(rest) = text.strip_prefix("(x)") {
        return SaltElement::Radio {
            label: rest.trim().to_string(),
            selected: true,
        };
    }
    if let Some(rest) = text.strip_prefix("( )") {
        return SaltElement::Radio {
            label: rest.trim().to_string(),
            selected: false,
        };
    }
    // Fallback: plain text label.
    SaltElement::Text(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenise_splits_pipe_separated_row() {
        // Single line `| a | b |` → tokens [a(NewCol), b(NewCol)].
        let toks = tokenise(&["| a | b |"]);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].text, "a");
        assert_eq!(toks[0].terminator, Terminator::NewCol);
        assert_eq!(toks[1].text, "b");
        assert_eq!(toks[1].terminator, Terminator::NewCol);
    }

    #[test]
    fn tokenise_last_token_without_pipe_is_newline() {
        // Line with no trailing `|` → last token has NewLine.
        let toks = tokenise(&["a | b"]);
        assert_eq!(toks[0].terminator, Terminator::NewCol);
        assert_eq!(toks[1].terminator, Terminator::NewLine);
    }

    #[test]
    fn parse_nested_table_flattens_into_single_row() {
        // This mirrors Java's observed output: `{#| a | b | \n | c | d |}`
        // produces a 1x4 grid because all cells get NEWCOL terminators.
        let src = "@startsalt\n{#\n| a | b |\n| c | d |\n}\n@endsalt";
        let diag = parse_salt_diagram(src).unwrap();
        match &diag.root {
            SaltElement::Pyramid(p) => {
                assert_eq!(p.cells.len(), 4);
                assert_eq!(p.rows, 1);
                assert_eq!(p.cols, 4);
            }
            _ => panic!("expected pyramid root"),
        }
    }

    #[test]
    fn parse_button_label_strips_brackets() {
        let el = build_element("[OK]");
        match el {
            SaltElement::Button(text) => assert_eq!(text, "OK"),
            _ => panic!("expected button"),
        }
    }

    #[test]
    fn parse_checkbox_with_label() {
        match build_element("[X] Remember me") {
            SaltElement::Checkbox { label, checked } => {
                assert_eq!(label, "Remember me");
                assert!(checked);
            }
            _ => panic!("expected checkbox"),
        }
    }
}
