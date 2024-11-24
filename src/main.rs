use clap::Parser;
use file::load_file;

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
    debug!("Args: {:#?}", args);

    let file_path = args.input_file_path;
    let code = load_file(file_path.as_path());
    debug!("Code: {:#?}", code);
}
