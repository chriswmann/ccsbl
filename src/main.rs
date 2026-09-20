use clap::Parser;
use std::process::exit;

use tracing::debug;

mod cli;
mod compiler;
mod errors;
mod file;
mod ops;

fn main() {
    tracing_subscriber::fmt::init();
    let args = cli::Cli::parse();
    debug!("Args: {:#?}", &args);

    let code = match file::load_file(&args.file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };
    dbg!(&code);

    let tokens = match compiler::tokenise(&code) {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };
    dbg!(&tokens);
}
