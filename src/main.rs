use std::path::PathBuf;

fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let (input, output) = match parse_args(&args) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{msg}");
            std::process::exit(1);
        }
    };

    let source = match std::fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {input:?}: {e}");
            std::process::exit(1);
        }
    };

    match plantuml_little::convert_with_input_path(&source, &input) {
        Ok(svg) => {
            if let Err(e) = std::fs::write(&output, svg) {
                eprintln!("error: cannot write {output:?}: {e}");
                std::process::exit(1);
            }
            log::info!("written: {output:?}");
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> Result<(PathBuf, PathBuf), String> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                println!(
                    "plantuml-little {}\nConvert .puml files to SVG\n\n\
                     Usage: plantuml-little [OPTIONS] <INPUT>\n\n\
                     Arguments:\n  <INPUT>  Input .puml file\n\n\
                     Options:\n  -o, --output <OUTPUT>  Output .svg file\n  \
                     -h, --help             Print help\n  \
                     -V, --version          Print version",
                    env!("CARGO_PKG_VERSION")
                );
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("plantuml-little {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("error: -o requires a value".into());
                }
                output = Some(PathBuf::from(&args[i]));
            }
            arg if arg.starts_with('-') => {
                return Err(format!("error: unknown option: {arg}"));
            }
            _ => {
                if input.is_some() {
                    return Err("error: unexpected extra argument".into());
                }
                input = Some(PathBuf::from(&args[i]));
            }
        }
        i += 1;
    }
    let input = input.ok_or(
        "error: missing required argument <INPUT>\n\nUsage: plantuml-little [OPTIONS] <INPUT>",
    )?;
    let output = output.unwrap_or_else(|| input.with_extension("svg"));
    Ok((input, output))
}
