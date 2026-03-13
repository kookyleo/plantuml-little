use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "plantuml-little")]
#[command(about = "Convert .puml files to SVG")]
#[command(version)]
struct Cli {
    /// Input .puml file
    input: PathBuf,

    /// Output .svg file (default: same name as input with .svg extension)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let output = cli
        .output
        .unwrap_or_else(|| cli.input.with_extension("svg"));

    let source = match std::fs::read_to_string(&cli.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {:?}: {}", cli.input, e);
            std::process::exit(1);
        }
    };

    match plantuml_little::convert_with_input_path(&source, &cli.input) {
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
