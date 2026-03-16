use std::path::Path;

pub mod error;
pub mod font_metrics;
pub mod abel;
pub mod decoration;
pub mod dot;
pub mod klimt;
pub mod layout;
pub mod skin;
pub mod svek;
pub mod tim;
pub mod model;
pub mod parser;
pub mod preproc;
pub mod render;
pub mod style;

pub use error::{Error, Result};

/// Convert PlantUML text to an SVG string
pub fn convert(puml_source: &str) -> Result<String> {
    let cwd = std::env::current_dir().ok();
    let expanded = if let Some(base_dir) = cwd.as_deref() {
        preproc::preprocess_with_base_dir(puml_source, base_dir)?
    } else {
        preproc::preprocess(puml_source)?
    };
    render_expanded(puml_source, &expanded)
}

/// Convert PlantUML text to SVG using an explicit base directory for relative
/// preprocessor includes.
pub fn convert_with_base_dir(puml_source: &str, base_dir: &Path) -> Result<String> {
    let expanded = preproc::preprocess_with_base_dir(puml_source, base_dir)?;
    render_expanded(puml_source, &expanded)
}

/// Convert PlantUML text to SVG using the original input file path.
/// This preserves filename/dirpath preprocessor context.
pub fn convert_with_input_path(puml_source: &str, input_path: &Path) -> Result<String> {
    let expanded = preproc::preprocess_with_source_path(puml_source, input_path)?;
    render_expanded(puml_source, &expanded)
}

fn render_expanded(original_source: &str, expanded: &str) -> Result<String> {
    // Extract SVG sprite definitions before parsing (sprite lines would confuse parsers)
    let (cleaned, sprites) = parser::common::extract_sprites(expanded);
    render::svg_richtext::set_sprites(sprites);
    // Use a guard to ensure sprites are cleared even if rendering panics
    struct SpriteGuard;
    impl Drop for SpriteGuard {
        fn drop(&mut self) {
            crate::render::svg_richtext::clear_sprites();
        }
    }
    let _guard = SpriteGuard;
    render_cleaned(original_source, &cleaned)
}

fn render_cleaned(original_source: &str, source: &str) -> Result<String> {
    let diagram = parser::parse(source)?;
    let skin = style::parse_skinparams(source);
    let diagram_layout = layout::layout(&diagram, &skin)?;
    let mut meta = parser::common::parse_meta(source);
    enrich_meta_source_lines(&mut meta, source);
    let svg = render::svg::render_with_source(
        &diagram,
        &diagram_layout,
        &skin,
        &meta,
        Some(original_source),
    )?;
    Ok(svg)
}
fn enrich_meta_source_lines(meta: &mut model::DiagramMeta, source: &str) {
    for (i, line) in source.lines().enumerate() {
        let t = line.trim();
        if meta.header.is_some() && meta.header_line.is_none() && (t.starts_with("header ") || t == "header") { meta.header_line = Some(i); }
        if meta.title.is_some() && meta.title_line.is_none() && (t.starts_with("title ") || t == "title") { meta.title_line = Some(i); }
        if meta.footer.is_some() && meta.footer_line.is_none() && (t.starts_with("footer ") || t == "footer") { meta.footer_line = Some(i); }
        if meta.caption.is_some() && meta.caption_line.is_none() && t.starts_with("caption ") { meta.caption_line = Some(i); }
        if meta.legend.is_some() && meta.legend_line.is_none() && t.starts_with("legend") && (t.len() == 6 || t.as_bytes().get(6) == Some(&b' ')) { meta.legend_line = Some(i); }
    }
}
