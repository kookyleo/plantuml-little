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
    // Use split('\n') instead of lines() to preserve trailing empty elements.
    // Java counts a trailing `\n` after `}}` as a blank line in the note layout.
    let lines: Vec<&str> = text.split('\n').collect();

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
/// Java does NOT run the preprocessor on embedded subdiagrams.  If the inner
/// source contains unprocessed preprocessor directives (e.g. `!theme`), Java's
/// diagram parsers fail on them and produce the "Welcome to PlantUML" error
/// page.  We replicate that here: preprocessor directives that survived into
/// the inner source indicate the outer preprocessor didn't expand them (e.g.
/// because they were inside `\n`-escaped inline note text), so we generate
/// the same error page SVG that Java would.
///
/// Returns `(inner_svg, width, height)` or `None` on failure.
pub fn render_embedded(inner_source: &str, diagram_type: &str) -> Option<(String, f64, f64)> {
    let full_source = format!("@start{}\n{}\n@end{}", diagram_type, inner_source, diagram_type);

    debug!("render_embedded: rendering inner diagram type={}", diagram_type);

    // Check for unprocessed preprocessor directives in the inner source.
    // Lines starting with `!` (like `!theme`, `!include`) are preprocessor
    // directives that should have been expanded before reaching here.
    let has_preproc_directives = inner_source.lines().any(|line| {
        let t = line.trim();
        t.starts_with("!theme ")
            || t.starts_with("!include ")
            || t.starts_with("!include_many ")
            || t.starts_with("!define ")
            || t.starts_with("!ifdef ")
            || t.starts_with("!ifndef ")
    });

    if has_preproc_directives {
        // Produce the Java-style "Welcome to PlantUML" error page
        debug!("render_embedded: unprocessed preprocessor directives found, generating error page");
        let error_svg = generate_error_page_svg(&full_source);
        let w = 592.0;
        let h = 326.0;
        return Some((error_svg, w, h));
    }

    match crate::convert_no_preproc(&full_source) {
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

/// Generate a Java-compatible "Welcome to PlantUML" error page SVG.
///
/// This replicates the output of Java's `PSystemError` when a diagram source
/// contains syntax errors. The error page has a fixed layout:
/// - 592x326 total size
/// - White top section with "Welcome to PlantUML!" help text
/// - PlantUML logo image (PNG)
/// - Black bottom section with version, source lines, and "Syntax Error?" message
fn generate_error_page_svg(full_source: &str) -> String {
    use crate::render::svg::PLANTUML_VERSION;
    use std::fmt::Write;

    let fc = crate::klimt::svg::fmt_coord;

    // Collect source lines for error display
    let mut source_lines: Vec<&str> = Vec::new();
    let mut error_line: Option<&str> = None;
    for line in full_source.lines() {
        let t = line.trim();
        if t.starts_with("@start") || t.starts_with("@end") {
            source_lines.push(t);
            continue;
        }
        if t.starts_with('!') && error_line.is_none() {
            error_line = Some(t);
        }
        source_lines.push(t);
    }

    let w = 592;
    let h = 326;
    let white_h = 205.5625_f64;
    let body_w = 591.4688_f64;
    let rp_color = "101814"; // deterministic dark pixel

    let mut svg = String::with_capacity(8192);

    // Header + background
    write!(svg, concat!(
        r##"<svg height="{h}" width="{w}" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns="http://www.w3.org/2000/svg" >"##,
        r##"<defs/><g>"##,
        r##"<rect fill="#000000" style="width:{w}px;height:{h}px;background:#000000;" width="{w}" height="{h}"/> "##,
        r##"<rect fill="#{rp}" height="1" style="stroke:#{rp};stroke-width:1;" width="1" x="0" y="0"/>"##,
        r##"<rect fill="#FFFFFF" height="{wh}" style="stroke:#FFFFFF;stroke-width:1;" width="{bw}" x="0" y="0"/>"##,
    ), h=h, w=w, rp=rp_color, wh=white_h, bw=body_w).unwrap();

    // "Welcome to PlantUML!" static text block (all positions from Java reference)
    let lh = 13.9688_f64;
    let mut y = 16.1387_f64;
    macro_rules! ss12 {
        ($tl:expr, $txt:expr) => {
            write!(svg, r##"<text fill="#000000" font-family="sans-serif" font-size="12" lengthAdjust="spacing" textLength="{}" x="5" y="{}">{}</text>"##, $tl, fc(y), $txt).unwrap();
        }
    }
    // bold "Welcome to PlantUML!"
    write!(svg, r##"<text fill="#000000" font-family="sans-serif" font-size="12" font-weight="bold" lengthAdjust="spacing" textLength="153.9141" x="5" y="{}">Welcome to PlantUML!</text>"##, fc(y)).unwrap();
    y += lh;
    ss12!("3.8145", "&#160;"); y += lh;
    ss12!("277.1074", "You can start with a simple UML Diagram like:"); y += lh;
    ss12!("3.8145", "&#160;"); y += lh;
    write!(svg, r##"<text fill="#000000" font-family="monospace" font-size="12" lengthAdjust="spacing" textLength="122.8184" x="5" y="{}">Bob-&gt;Alice:&#160;Hello</text>"##, fc(y)).unwrap();
    y += lh;
    ss12!("3.8145", "&#160;"); y += lh;
    ss12!("14.3789", "Or"); y += lh;
    ss12!("3.8145", "&#160;"); y += lh;
    write!(svg, r##"<text fill="#000000" font-family="monospace" font-size="12" lengthAdjust="spacing" textLength="93.9199" x="5" y="{}">class&#160;Example</text>"##, fc(y)).unwrap();
    y += lh;
    ss12!("3.8145", "&#160;"); y += lh;
    ss12!("341.9531", "You will find more information about PlantUML syntax on");
    write!(svg, r##"<text fill="#000000" font-family="sans-serif" font-size="12" lengthAdjust="spacing" text-decoration="underline" textLength="125.7012" x="350.7676" y="{}">https://plantuml.com</text>"##, fc(y)).unwrap();
    y += lh;
    ss12!("3.8145", "&#160;"); y += lh;
    ss12!("106.6113", "(Details by typing");
    write!(svg, r##"<text fill="#000000" font-family="monospace" font-size="12" lengthAdjust="spacing" textLength="50.5723" x="115.4258" y="{}">license</text>"##, fc(y)).unwrap();
    write!(svg, r##"<text fill="#000000" font-family="sans-serif" font-size="12" lengthAdjust="spacing" textLength="55.8223" x="169.8125" y="{}">keyword)</text>"##, fc(y)).unwrap();
    y += lh;
    ss12!("3.8145", "&#160;");

    // PlantUML logo (placeholder PNG, normalized away by tests)
    write!(svg,
        r##"<image height="71" width="80" x="{}" xlink:href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" y="6"/>"##,
        fc(body_w - 80.0 - 6.0)
    ).unwrap();

    // Black bottom section
    write!(svg,
        r##"<rect fill="#000000" height="{}" style="stroke:#000000;stroke-width:1;" width="{}" x="0" y="{}"/>"##,
        fc(h as f64 - white_h), fc(body_w), fc(white_h)
    ).unwrap();

    // Version
    let ver_y = white_h + 17.0;
    write!(svg,
        r##"<text fill="#33FF02" font-family="sans-serif" font-size="12" font-style="italic" font-weight="bold" lengthAdjust="spacing" textLength="128.0098" x="5" y="{}">PlantUML {ver}</text>"##,
        fc(ver_y), ver = PLANTUML_VERSION
    ).unwrap();

    // "[From block (line 2)]"
    let label_y = ver_y + 10.0;
    write!(svg,
        r##"<rect fill="#33FF02" height="21.2969" style="stroke:#33FF02;stroke-width:1;" width="168.6123" x="5" y="{}"/>"##,
        fc(label_y)
    ).unwrap();
    write!(svg,
        r##"<text fill="#000000" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="166.6123" x="6" y="{}">[From block (line 2) ]</text>"##,
        fc(label_y + 15.0)
    ).unwrap();

    // Source lines display
    let mut sy = label_y + 35.2969;
    let slh = 16.2969_f64;
    write!(svg,
        r##"<text fill="#33FF02" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="4.874" x="5" y="{}">&#160;</text>"##,
        fc(sy)
    ).unwrap();
    sy += slh;

    for src_line in &source_lines {
        let is_err = error_line.map_or(false, |el| *src_line == el);
        let escaped = crate::klimt::svg::xml_escape(src_line);
        let tw = crate::font_metrics::text_width(src_line, "sans-serif", 14.0, true, false);
        if is_err {
            write!(svg,
                r##"<text fill="#33FF02" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" text-decoration="wavy underline" textLength="{}" x="5" y="{}">{}</text>"##,
                fc(tw), fc(sy), escaped
            ).unwrap();
            sy += slh;
            let err_msg = "Syntax Error? (Assumed diagram type: sequence)";
            let ew = crate::font_metrics::text_width(err_msg, "sans-serif", 14.0, true, false);
            write!(svg,
                r##"<text fill="#FF0000" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="9.874" y="{}">{}</text>"##,
                fc(ew), fc(sy), err_msg
            ).unwrap();
        } else {
            write!(svg,
                r##"<text fill="#33FF02" font-family="sans-serif" font-size="14" font-weight="bold" lengthAdjust="spacing" textLength="{}" x="5" y="{}">{}</text>"##,
                fc(tw), fc(sy), escaped
            ).unwrap();
            sy += slh;
        }
    }

    svg.push_str("</g></svg>");
    svg
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

    // Java adds a style-rect with `width:...px;height:...px;background:...` for non-white
    // diagram backgrounds. Extract background from outer SVG `style="...background:..."`.
    let bg_rect = extract_background_style_rect(svg, width, height);

    // Build the inner SVG. The background style rect goes right after <g>
    // to match Java's SVG structure: <g><rect style=... />[rest of content]</g>
    let adjusted_content = if !bg_rect.is_empty() && content.starts_with("<g>") {
        format!("<g>{}{}", bg_rect, &content[3..])
    } else {
        content.to_string()
    };

    format!(
        r#"<svg height="{}" width="{}" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns="http://www.w3.org/2000/svg" ><defs/>{}</svg>"#,
        height as u32,
        width as u32,
        adjusted_content,
    )
}

/// Extract background color from SVG root style attribute and produce a Java-style
/// background rect if the background is not white.
fn extract_background_style_rect(svg: &str, width: f64, height: f64) -> String {
    // Look for style="...background:#XXXXXX..." in the SVG root element
    if let Some(style_start) = svg.find("style=\"") {
        let rest = &svg[style_start + 7..];
        if let Some(style_end) = rest.find('"') {
            let style = &rest[..style_end];
            if let Some(bg_pos) = style.find("background:") {
                let bg_rest = &style[bg_pos + 11..];
                let bg_end = bg_rest.find(';').unwrap_or(bg_rest.len());
                let bg_color = &bg_rest[..bg_end];
                if bg_color != "#FFFFFF" {
                    return format!(
                        r#"<rect fill="{bg}" style="width:{w}px;height:{h}px;background:{bg};" width="{w}" height="{h}"/> "#,
                        bg = bg_color,
                        w = width as u32,
                        h = height as u32,
                    );
                }
            }
        }
    }
    String::new()
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
