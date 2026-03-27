use plantuml_little::parser::component::parse_component_diagram;
fn main() {
    let source = "@startuml\nrectangle A [\ntest 1\\ntest 11\ntest 2%chr(10)test 22\ntest 3 %chr(10)test 33\ntest 4 %newline()test44\n]\n@enduml";
    let p = plantuml_little::preproc::preprocess(source).unwrap();
    println!("PREPROC:\n{}\n---", p);
    let d = parse_component_diagram(&p).unwrap();
    println!("{:#?}", d);
}
