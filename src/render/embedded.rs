//! Embedded diagram (`{{ }}`) support.
//!
//! Java PlantUML allows embedding a sub-diagram inside note text using `{{ ... }}`.
//! The inner content (between `{{` and `}}`) is treated as a separate PlantUML
//! diagram source, rendered to SVG, and embedded as a base64-encoded `<image>` element.
//!
//! Flow:
//! 1. The preprocessor expands directives (e.g. `!theme`) inside `{{ }}` — this is correct.
//! 2. At render time, `{{ ... }}` blocks in note text are detected by this module.
//! 3. The inner content is wrapped with `@startuml`/`@enduml`, rendered recursively.
//! 4. The resulting SVG is base64-encoded and emitted as `<image xlink:href="data:...">`.

use log::{debug, warn};

/// Parsed note text with embedded diagram support.
///
/// When note text contains `{{ ... }}` blocks, the text is split into:
/// - `before`: lines before the `{{ }}` block
/// - `embedded_source`: the inner diagram source (to be rendered separately)
/// - `after`: lines after the `}}` block
pub struct EmbeddedBlock {
    /// Lines before the `{{` delimiter.
    pub before: String,
    /// The embedded diagram source (between `{{` and `}}`), ready for rendering.
    /// This gets wrapped with `@startuml`/`@enduml` before recursive rendering.
    pub inner_source: String,
    /// The diagram type extracted from `{{` line (e.g. "uml", "salt", "ditaa").
    pub diagram_type: String,
    /// Lines after the `}}` delimiter.
    pub after: String,
}

/// Detect and extract `{{ ... }}` embedded blocks from text.
///
/// Returns `None` if no embedded block is found.
/// Handles nested `{{ }}` — only the outermost pair is extracted.
pub fn extract_embedded(text: &str) -> Option<EmbeddedBlock> {
    let lines: Vec<&str> = text.lines().collect();

    let mut open_idx = None;
    let mut diagram_type = String::from("uml");

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(dtype) = get_embedded_type(trimmed) {
            open_idx = Some(i);
            diagram_type = dtype.to_string();
            break;
        }
    }

    let open_idx = open_idx?;

    // Find closing `}}`
    let mut nested = 1;
    let mut close_idx = None;
    for (i, line) in lines.iter().enumerate().skip(open_idx + 1) {
        let trimmed = line.trim();
        if get_embedded_type(trimmed).is_some() {
            nested += 1;
        } else if trimmed == "}}" {
            nested -= 1;
            if nested == 0 {
                close_idx = Some(i);
                break;
            }
        }
    }

    let close_idx = close_idx?;

    let before = lines[..open_idx].join("\n");
    let inner_lines: Vec<&str> = lines[open_idx + 1..close_idx].to_vec();
    let inner_source = inner_lines.join("\n");
    let after = if close_idx + 1 < lines.len() {
        lines[close_idx + 1..].join("\n")
    } else {
        String::new()
    };

    debug!(
        "extract_embedded: type={}, before_lines={}, inner_lines={}, after_lines={}",
        diagram_type,
        before.lines().count(),
        inner_lines.len(),
        after.lines().count(),
    );

    Some(EmbeddedBlock {
        before,
        inner_source,
        diagram_type,
        after,
    })
}

/// Check if a trimmed line starts an embedded block (`{{ }}`).
/// Returns the diagram type if it does.
fn get_embedded_type(trimmed: &str) -> Option<&'static str> {
    if !trimmed.starts_with("{{") {
        return None;
    }
    match trimmed {
        "{{" => Some("uml"),
        "{{ditaa" => Some("ditaa"),
        "{{salt" => Some("salt"),
        "{{uml" => Some("uml"),
        "{{wbs" => Some("wbs"),
        "{{mindmap" => Some("mindmap"),
        "{{gantt" => Some("gantt"),
        "{{json" => Some("json"),
        "{{yaml" => Some("yaml"),
        "{{wire" => Some("wire"),
        "{{creole" => Some("creole"),
        "{{board" => Some("board"),
        "{{ebnf" => Some("ebnf"),
        "{{regex" => Some("regex"),
        "{{files" => Some("files"),
        "{{chronology" => Some("chronology"),
        "{{chen" => Some("chen"),
        "{{chart" => Some("chart"),
        "{{nwdiag" => Some("nwdiag"),
        "{{packetdiag" => Some("packetdiag"),
        _ => None,
    }
}

/// Render an embedded diagram to SVG and return the inner SVG string.
///
/// The `inner_source` is the content between `{{` and `}}`.
/// It is wrapped with `@startuml`/`@enduml` and rendered using the main convert function.
///
/// Returns `(inner_svg, width, height)` or `None` on failure.
pub fn render_embedded(inner_source: &str, diagram_type: &str) -> Option<(String, f64, f64)> {
    let full_source = format!("@start{}\n{}\n@end{}", diagram_type, inner_source, diagram_type);

    debug!("render_embedded: rendering inner diagram type={}", diagram_type);

    match crate::convert(&full_source) {
        Ok(svg) => {
            // Extract width/height from the SVG root element
            let (w, h) = extract_svg_dimensions(&svg)?;
            // Strip the outer SVG wrapper to get just the inner content for embedding
            let inner_svg = strip_to_inner_svg(&svg, w, h);
            Some((inner_svg, w, h))
        }
        Err(e) => {
            warn!("render_embedded: failed to render inner diagram: {}", e);
            None
        }
    }
}

/// Extract width and height from an SVG root element.
fn extract_svg_dimensions(svg: &str) -> Option<(f64, f64)> {
    // Look for viewBox="x y w h" or width="Npx" height="Npx"
    let w = extract_attr_px(svg, "width")?;
    let h = extract_attr_px(svg, "height")?;
    Some((w, h))
}

/// Extract a pixel dimension attribute from SVG markup.
fn extract_attr_px(svg: &str, attr: &str) -> Option<f64> {
    // Match attr="123px" or attr="123"
    let pattern = format!("{}=\"", attr);
    let start = svg.find(&pattern)?;
    let val_start = start + pattern.len();
    let rest = &svg[val_start..];
    let end = rest.find('"')?;
    let val = &rest[..end];
    val.strip_suffix("px").unwrap_or(val).parse().ok()
}

/// Strip the outer `<svg>` wrapper and produce a standalone inner SVG for embedding.
///
/// Java embeds the sub-diagram as a complete `<svg>` element (with its own
/// width/height and xmlns) that is then base64-encoded into an `<image>` element.
fn strip_to_inner_svg(svg: &str, width: f64, height: f64) -> String {
    // Find the content after <defs/> and before the closing </svg>
    // The structure is: <svg ...><defs/>...<g>...</g></svg>
    // We want to produce: <svg height="H" width="W" xmlns:xlink="..." xmlns="..."><defs/><g>...</g></svg>

    // Find <defs/> position
    let defs_end = svg.find("<defs/>").map(|p| p + "<defs/>".len());
    let content_start = defs_end.unwrap_or(0);

    // Find the end: remove trailing </svg> and the <?plantuml-src ...?> processing instruction
    let mut content_end = svg.len();
    if let Some(pos) = svg.rfind("</svg>") {
        content_end = pos;
    }
    // Also strip trailing PI like <?plantuml-src ...?>
    let content = &svg[content_start..content_end];

    // Build the inner SVG
    format!(
        r#"<svg height="{}" width="{}" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns="http://www.w3.org/2000/svg" ><defs/>{}</svg>"#,
        height as u32,
        width as u32,
        content,
    )
}

/// Encode the inner SVG as a base64 data URI for use in `<image xlink:href="...">`.
pub fn svg_to_data_uri(inner_svg: &str) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(inner_svg.as_bytes());
    format!("data:image/svg+xml;base64,{}", encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_embedded_simple() {
        let text = "heading\n{{\nUser->System: test\n}}\nfooter";
        let block = extract_embedded(text).unwrap();
        assert_eq!(block.before, "heading");
        assert_eq!(block.inner_source, "User->System: test");
        assert_eq!(block.diagram_type, "uml");
        assert_eq!(block.after, "footer");
    }

    #[test]
    fn test_extract_embedded_no_block() {
        let text = "just plain text\nno embedded block";
        assert!(extract_embedded(text).is_none());
    }

    #[test]
    fn test_extract_embedded_with_type() {
        let text = "{{salt\nbutton\n}}";
        let block = extract_embedded(text).unwrap();
        assert_eq!(block.before, "");
        assert_eq!(block.inner_source, "button");
        assert_eq!(block.diagram_type, "salt");
        assert_eq!(block.after, "");
    }

    #[test]
    fn test_get_embedded_type() {
        assert_eq!(get_embedded_type("{{"), Some("uml"));
        assert_eq!(get_embedded_type("{{salt"), Some("salt"));
        assert_eq!(get_embedded_type("{{ditaa"), Some("ditaa"));
        assert_eq!(get_embedded_type("nope"), None);
        assert_eq!(get_embedded_type("{not double"), None);
    }

    #[test]
    fn test_extract_svg_dimensions() {
        let svg = r#"<svg width="183px" height="122px" viewBox="0 0 183 122">"#;
        let (w, h) = extract_svg_dimensions(svg).unwrap();
        assert_eq!(w, 183.0);
        assert_eq!(h, 122.0);
    }
}
