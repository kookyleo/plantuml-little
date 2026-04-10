use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let puml_path = &args[1];
    let source = fs::read_to_string(puml_path).expect("cannot read");
    let preproc_out =
        plantuml_little::preproc::preprocess_with_source_path(&source, Path::new(puml_path))
            .expect("preproc failed");
    let diag = plantuml_little::parser::component::parse_component_diagram(&preproc_out)
        .expect("parse failed");
    println!("entities: {}", diag.entities.len());
    for e in &diag.entities {
        println!(
            "  - {} (id={}, kind={:?}, stereo={:?})",
            e.name, e.id, e.kind, e.stereotype
        );
    }
    println!("groups: {}", diag.groups.len());
    for g in &diag.groups {
        println!(
            "  - name={} id={} children={:?} stereo={:?}",
            g.name, g.id, g.children, g.stereotype
        );
    }
    println!("links: {}", diag.links.len());
}
