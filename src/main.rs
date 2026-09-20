use clap::Parser;
use file::load_file;
use std::process;

use tracing::debug;

mod cli;
mod compiler;
mod errors;
mod file;
mod instructions;
mod ops;

fn main() {
    tracing_subscriber::fmt::init();
    let args = cli::Cli::parse();
    debug!("Args: {:#?}", &args);

    let lines = load_file(args.file.clone());
    match &lines {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
        }
    }
    debug!("Code: {:#?}", lines);

    let mut bytecode = vec![];

    if args.compile {
        bytecode = match compiler::compile(&lines.unwrap()) {
            Err(err) => {
                eprintln!("Error compiling '{}': {}", args.file.display(), err);
                process::exit(1);
            }
            Ok(val) => val,
        };
    }
    debug!("Bytecode: {:#?}", bytecode);
}
