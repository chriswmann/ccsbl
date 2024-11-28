use clap::Parser;
use file::load_file;
use std::path;
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

    let file_path = &args.file;
    let lines = load_file(file_path.as_path());
    match &lines {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    debug!("Code: {:#?}", lines);

    let mut bytecode = vec![];

    if args.compile {
        if path::Path::new(&args.file).extension().is_some()
            && path::Path::new(&args.file).extension().unwrap() == "mclb"
        {
            eprintln!("Error: mclb file supplied, refusing to compile bytecode. Did you mean to execute it?");
        }
        bytecode = match compiler::compile(lines.unwrap()) {
            Err(err) => {
                eprintln!("Error compiling '{:?}': {}", args.file, err);
                process::exit(1);
            }
            Ok(val) => val,
        };
    }
    debug!("Bytecode: {:#?}", bytecode);
}
