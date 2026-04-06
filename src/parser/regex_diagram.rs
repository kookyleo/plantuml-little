use crate::model::regex_diagram::{RegexDiagram, RegexNode};
use crate::Result;

pub fn parse_regex_diagram(source: &str) -> Result<RegexDiagram> {
    let lines: Vec<&str> = source.lines().collect();
    let start_idx = lines.iter().position(|line| line.trim().starts_with("@startregex"))
        .ok_or_else(|| crate::Error::Parse { line: 1, column: Some(1), message: "missing @startregex".into() })?;
    let end_idx = lines.iter().position(|line| line.trim().starts_with("@endregex"))
        .ok_or_else(|| crate::Error::Parse { line: lines.len().max(1), column: Some(1), message: "missing @endregex".into() })?;
    let body: String = lines[start_idx + 1..end_idx].iter().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<&str>>().join("");
    if body.is_empty() { return Ok(RegexDiagram { node: RegexNode::Literal(String::new()) }); }
    let chars: Vec<char> = body.chars().collect();
    let (node, _) = parse_alternation(&chars, 0)?;
    Ok(RegexDiagram { node })
}

fn parse_alternation(chars: &[char], start: usize) -> Result<(RegexNode, usize)> {
    let mut branches = Vec::new();
    let (first, mut pos) = parse_concat(chars, start)?;
    branches.push(first);
    while pos < chars.len() && chars[pos] == '|' {
        pos += 1;
        let (branch, next) = parse_concat(chars, pos)?;
        branches.push(branch); pos = next;
    }
    if branches.len() == 1 { Ok((branches.remove(0), pos)) } else { Ok((RegexNode::Alternate(branches), pos)) }
}

fn parse_concat(chars: &[char], start: usize) -> Result<(RegexNode, usize)> {
    let mut items = Vec::new();
    let mut pos = start;
    while pos < chars.len() {
        match chars[pos] {
            '|' | ')' => break,
            '(' => { let (g, n) = parse_group(chars, pos)?; let (q, n) = apply_quantifier(chars, n, g)?; items.push(q); pos = n; }
            '[' => { let (c, n) = parse_char_class(chars, pos)?; let (q, n) = apply_quantifier(chars, n, c)?; items.push(q); pos = n; }
            '.' => { let nd = RegexNode::Literal(".".into()); pos += 1; let (q, n) = apply_quantifier(chars, pos, nd)?; items.push(q); pos = n; }
            '\\' if pos + 1 < chars.len() => { let nd = RegexNode::Literal(format!("\\{}", chars[pos + 1])); pos += 2; let (q, n) = apply_quantifier(chars, pos, nd)?; items.push(q); pos = n; }
            c => { let nd = RegexNode::Literal(c.to_string()); pos += 1; let (q, n) = apply_quantifier(chars, pos, nd)?; items.push(q); pos = n; }
        }
    }
    // Merge consecutive Literal nodes into single strings (Java renders "cat"
    // as one box, not three separate character boxes).
    let mut merged = Vec::new();
    let mut lit_buf = String::new();
    for item in items {
        if let RegexNode::Literal(ref s) = item {
            lit_buf.push_str(s);
        } else {
            if !lit_buf.is_empty() {
                merged.push(RegexNode::Literal(std::mem::take(&mut lit_buf)));
            }
            merged.push(item);
        }
    }
    if !lit_buf.is_empty() {
        merged.push(RegexNode::Literal(lit_buf));
    }
    match merged.len() {
        0 => Ok((RegexNode::Literal(String::new()), pos)),
        1 => Ok((merged.remove(0), pos)),
        _ => Ok((RegexNode::Concat(merged), pos)),
    }
}

fn parse_group(chars: &[char], start: usize) -> Result<(RegexNode, usize)> {
    let (inner, pos) = parse_alternation(chars, start + 1)?;
    let pos = if pos < chars.len() && chars[pos] == ')' { pos + 1 } else { pos };
    Ok((RegexNode::Group(Box::new(inner)), pos))
}

fn parse_char_class(chars: &[char], start: usize) -> Result<(RegexNode, usize)> {
    let mut pos = start + 1;
    let mut items = Vec::new();
    while pos < chars.len() && chars[pos] != ']' {
        if chars[pos] == '\\' && pos + 1 < chars.len() { items.push(format!("\\{}", chars[pos + 1])); pos += 2; }
        else { items.push(chars[pos].to_string()); pos += 1; }
    }
    if pos < chars.len() { pos += 1; }
    Ok((RegexNode::CharClass(items), pos))
}

fn apply_quantifier(chars: &[char], start: usize, inner: RegexNode) -> Result<(RegexNode, usize)> {
    if start >= chars.len() { return Ok((inner, start)); }
    match chars[start] {
        '?' => Ok((RegexNode::Optional(Box::new(inner)), start + 1)),
        '*' => Ok((RegexNode::Quantifier { inner: Box::new(inner), min: 0, max: None, label: "*".into() }, start + 1)),
        '+' => Ok((RegexNode::Quantifier { inner: Box::new(inner), min: 1, max: None, label: "+".into() }, start + 1)),
        '{' => {
            let mut pos = start + 1;
            let mut ns = String::new();
            while pos < chars.len() && chars[pos].is_ascii_digit() { ns.push(chars[pos]); pos += 1; }
            let min: u32 = ns.parse().unwrap_or(0);
            let mut max = Some(min); let mut has_comma = false;
            if pos < chars.len() && chars[pos] == ',' {
                has_comma = true; pos += 1;
                let mut ms = String::new();
                while pos < chars.len() && chars[pos].is_ascii_digit() { ms.push(chars[pos]); pos += 1; }
                max = if ms.is_empty() { None } else { Some(ms.parse().unwrap_or(0)) };
            }
            if pos < chars.len() && chars[pos] == '}' { pos += 1; }
            let label = if has_comma { if let Some(m) = max { format!("{{{},{}}}", min, m) } else { format!("{{{},}}", min) } } else { format!("{{{}}}", min) };
            Ok((RegexNode::Quantifier { inner: Box::new(inner), min, max, label }, pos))
        }
        _ => Ok((inner, start)),
    }
}
