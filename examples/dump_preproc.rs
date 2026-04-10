use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let puml_path = &args[1];
    let source = fs::read_to_string(puml_path).expect("cannot read");
    let out = plantuml_little::preproc::preprocess_with_source_path(&source, Path::new(puml_path))
        .expect("preproc failed");
    println!("{}", out);
}
