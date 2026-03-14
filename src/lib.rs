use std::path::Path;

pub mod error;
pub mod font_metrics;
pub mod layout;
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
    let result = render_cleaned(original_source, &cleaned);
    render::svg_richtext::clear_sprites();
    result
}

fn render_cleaned(original_source: &str, source: &str) -> Result<String> {
    let diagram = parser::parse(source)?;
    let diagram_layout = layout::layout(&diagram)?;
    let skin = style::parse_skinparams(source);
    let meta = parser::common::parse_meta(source);
    let svg = render::svg::render_with_source(
        &diagram,
        &diagram_layout,
        &skin,
        &meta,
        Some(original_source),
    )?;
    Ok(svg)
}
