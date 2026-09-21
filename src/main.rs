use clap::Parser;
use std::process::exit;

use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::errors::Error;

mod cli;
mod compiler;
mod errors;
mod file;
mod ops;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();
    match run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    }
}

fn run() -> Result<(), Error> {
    let args = cli::Cli::parse();
    debug!("Args: {:#?}", &args);

    let code = file::load_file(&args.file)?;
    debug!("{}", &code);

    let tokens = compiler::tokenise(&code)?;
    debug!("{:?}", &tokens);
    Ok(())
}
