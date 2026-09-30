use clap::Parser;
use std::cmp;
use std::process::exit;

use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::assembler::assemble;
use crate::bytecode::AsmInstr;
use crate::errors::Error;
use crate::scan::{classify, scan};

mod assembler;
mod bytecode;
mod cli;
mod errors;
mod file;
mod scan;
mod token;

#[derive(Debug)]
enum RunStatus {
    Success,
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

    let instr = compile(&code, max_end);

    match instr {
        Ok(instr) => {
            // encode instructions here
            let end = cmp::min(instr.len(), max_end);
            debug!("instr:\n{:?}", &instr[..end]);
            RunStatus::Success
        }
        Err(err) => {
            let mut lock = std::io::stderr().lock();
            match err.report(&mut lock) {
                Ok(()) => RunStatus::Failure(1),
                Err(_) => RunStatus::Failure(2),
            }
        }
    }
}

fn compile(code: &str, max_end: usize) -> Result<Vec<AsmInstr<'_>>, Error<'_>> {
    let lexemes = scan(code);
    let end = cmp::min(lexemes.len(), max_end);
    debug!("lexemes:\n{:?}", &lexemes[..end]);

    let tokens = classify(&lexemes);
    let end = cmp::min(tokens.len(), max_end);
    debug!("tokens:\n{:?}", &tokens[..end]);

    assemble(&tokens)
}
