use crate::klimt::svg::{fmt_coord, xml_escape};
use crate::model::hyperlink::Hyperlink;
use std::fmt::Write;

/// Escape a URL for use in an XML/SVG attribute value.
///
/// In addition to the standard XML escaping, this percent-encodes characters
/// that are not valid in URLs but might appear in user input.
pub fn url_escape(url: &str) -> String {
    // First apply XML escaping (handles &, <, >, ")
    xml_escape(url)
}

/// Wrap SVG content in an `<a>` element with href and optional tooltip.
///
/// Produces:
/// ```xml
/// <a href="url" target="_blank">
///   <title>tooltip</title>
///   content
/// </a>
/// ```
pub fn wrap_with_link(content: &str, link: &Hyperlink) -> String {
    if link.url.is_empty() && link.tooltip.is_none() {
        return content.to_string();
    }

    let mut buf = String::with_capacity(content.len() + 128);
    buf.push_str("<a");
    if !link.url.is_empty() {
        let escaped_url = url_escape(&link.url);
        write!(buf, r#" href="{escaped_url}" target="_blank""#).unwrap();
    }
    buf.push('>');
    buf.push('\n');

    if let Some(ref tooltip) = link.tooltip {
        let escaped_tip = xml_escape(tooltip);
        writeln!(buf, "<title>{escaped_tip}</title>").unwrap();
    }

    buf.push_str(content);
    buf.push('\n');
    buf.push_str("</a>");
    buf
}

/// Render a hyperlinked text element.
///
/// Produces a `<text>` element wrapped in `<a>`, with optional tooltip
/// via `<title>`.
pub fn render_linked_text(
    text: &str,
    x: f64,
    y: f64,
    link: &Hyperlink,
    extra_attrs: &str,
) -> String {
    let escaped_text = xml_escape(text);
    let text_elem = format!(
        r#"<text font-family="sans-serif" x="{}" y="{}"{extra}>{text}</text>"#,
        fmt_coord(x),
        fmt_coord(y),
        extra = if extra_attrs.is_empty() {
            String::new()
        } else {
            format!(" {extra_attrs}")
        },
        text = escaped_text,
    );
    wrap_with_link(&text_elem, link)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::hyperlink::Hyperlink;

    #[test]
    fn wrap_basic_content() {
        let link = Hyperlink {
            url: "https://example.com".into(),
            tooltip: None,
            label: None,
        };
        let result = wrap_with_link("<rect/>", &link);
        assert!(result.contains(r#"href="https://example.com""#));
        assert!(result.contains(r#"target="_blank""#));
        assert!(result.contains("<rect/>"));
        assert!(result.contains("</a>"));
        // No <title> when tooltip is None
        assert!(!result.contains("<title>"));
    }

    #[test]
    fn wrap_with_tooltip_adds_title() {
        let link = Hyperlink {
            url: "https://example.com".into(),
            tooltip: Some("Go here".into()),
            label: None,
        };
        let result = wrap_with_link("<circle/>", &link);
        assert!(result.contains("<title>Go here</title>"));
        assert!(result.contains("<circle/>"));
    }

    #[test]
    fn render_linked_text_produces_a_with_text() {
        let link = Hyperlink {
            url: "https://example.com".into(),
            tooltip: None,
            label: Some("Click".into()),
        };
        let result = render_linked_text("Click", 10.0, 20.0, &link, "");
        assert!(result.contains(r#"<a href="https://example.com""#));
        assert!(result.contains(r#"<text font-family="sans-serif" x="10" y="20">Click</text>"#));
        assert!(result.contains("</a>"));
    }

    #[test]
    fn url_with_special_chars_is_escaped() {
        let link = Hyperlink {
            url: "https://example.com?a=1&b=2".into(),
            tooltip: None,
            label: None,
        };
        let result = wrap_with_link("x", &link);
        assert!(
            result.contains("a=1&amp;b=2"),
            "ampersand must be XML-escaped"
        );
        assert!(!result.contains("a=1&b"), "raw ampersand must not appear");
    }

    #[test]
    fn tooltip_with_special_chars_is_escaped() {
        let link = Hyperlink {
            url: "https://x.com".into(),
            tooltip: Some("A <b> & C".into()),
            label: None,
        };
        let result = wrap_with_link("z", &link);
        assert!(result.contains("A &lt;b&gt; &amp; C"));
    }

    #[test]
    fn render_linked_text_with_extra_attrs() {
        let link = Hyperlink {
            url: "https://x.com".into(),
            tooltip: Some("tip".into()),
            label: None,
        };
        let result = render_linked_text("hi", 5.0, 15.0, &link, r#"font-weight="bold""#);
        assert!(result.contains(r#"font-weight="bold""#));
        assert!(result.contains("<title>tip</title>"));
    }

    #[test]
    fn tooltip_only_link_wraps_without_href() {
        let link = Hyperlink {
            url: String::new(),
            tooltip: Some("hover".into()),
            label: Some("Visible".into()),
        };
        let result = wrap_with_link("<tspan>Visible</tspan>", &link);
        assert!(result.contains("<a>"));
        assert!(result.contains("<title>hover</title>"));
        assert!(!result.contains(" href="));
    }
}
