use clap::Parser;
use std::cmp;
use std::process::exit;

use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::assembler::{assemble, resolve, Assembled};
use crate::errors::Error;
use crate::scan::{classify, scan};
use crate::vm::execute;

mod assembler;
mod bytecode;
mod cli;
mod errors;
mod file;
mod scan;
mod token;
mod vm;

#[derive(Debug)]
enum RunStatus {
    Success,
    // 1: compilation or execution failure, reported
    // 2: compilation or execution failure, failure while reporting
    Failure(i32),
}

fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();
    match run() {
        RunStatus::Success => {}
        RunStatus::Failure(code) => {
            exit(code);
        }
    }
}

fn run() -> RunStatus {
    let args = cli::Cli::parse();
    debug!("Args: {:?}", &args);

    let code = match file::load_source(args.file) {
        Ok(code) => code,
        Err(err) => {
            let mut lock = std::io::stderr().lock();
            match err.report(&mut lock) {
                Ok(()) => return RunStatus::Failure(1),
                Err(_) => {
                    return RunStatus::Failure(2);
                }
            }
        }
    };
    debug!("source code:\n{}", &code);

    let mut max_end = 4;
    if args.verbose {
        max_end = 12;
    }

    let result = compile(&code, max_end);

    match result {
        Ok(()) => RunStatus::Success,
        // These diagnostics were already printed by `compile()`.
        Err(Error::Compiler) => RunStatus::Failure(1),
        Err(err) => {
            let mut lock = std::io::stderr().lock();
            match err.report(&mut lock) {
                Ok(()) => RunStatus::Failure(1),
                // Try to report errors again
                Err(_) => RunStatus::Failure(2),
            }
        }
    }
}

fn compile(code: &str, max_end: usize) -> Result<(), Error<'_>> {
    let lexemes = scan(code);
    let end = cmp::min(lexemes.len(), max_end);
    debug!("lexemes:\n{:?}", &lexemes[..end]);

    let tokens = classify(&lexemes);
    let end = cmp::min(tokens.len(), max_end);
    debug!("tokens:\n{:?}", &tokens[..end]);

    match assemble(&tokens) {
        Ok(Assembled { instrs, labels }) => match resolve(&instrs, &labels) {
            Ok(asm) => execute(&asm),
            Err(errors) => {
                let mut lock = std::io::stderr().lock();
                Error::report_many(&errors, &mut lock);
                Err(Error::Compiler)
            }
        },
        Err(errors) => {
            let mut lock = std::io::stderr().lock();
            Error::report_many(&errors, &mut lock);
            Err(Error::Compiler)
        }
    }
}
