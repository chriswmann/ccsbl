use std::io::Write;
use std::path;
use std::process::exit;

use clap::Parser;
use tracing::instrument;
use tracing_subscriber::EnvFilter;

use crate::assembler::{assemble, resolve};
use crate::bytecode::AsmInstr;
use crate::cli::Commands;
use crate::error::{display_error, report, report_many, AppError, CompileError};
use crate::file::load_source;
use crate::program::Program;
use crate::vm::execute;

mod assembler;
mod bytecode;
mod cli;
mod error;
mod file;
mod program;
mod scan;
mod token;
mod ui;
mod vm;

fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    let args = cli::Cli::parse();
    let asm = match args.command {
        Commands::Load { file_path } => match load(file_path) {
            Ok(asm) => asm,
            Err(AppError::ExecutionFailure(code)) => exit(code),
        },
    };
    let mut stdout = std::io::stdout().lock();

    let execution_result = execute(&asm, &mut stdout);
    let flush_result = stdout.flush();
    drop(stdout);

    if execution_result.is_err() || flush_result.is_err() {
        let mut stderr = std::io::stderr().lock();

        if let Err(err) = execution_result {
            let _ = report(err, &mut stderr);
        }

        if let Err(err) = flush_result {
            let _ = writeln!(stderr, "Failed to flush program output: {err}");
        }
        exit(1);
    }
}

#[instrument]
fn load(file_path: path::PathBuf) -> Result<Vec<AsmInstr>, AppError> {
    let extension = file_path.extension().expect("should be able to get file extension as cli::validate_file ensures we have a 'ccl' or 'ccbc' file").to_string_lossy();
    match extension.as_ref() {
        "ccbc" => match Program::load(&file_path) {
            Ok(program) => program.decode().map_err(display_error),
            Err(err) => Err(display_error(err)),
        },
        "ccl" => match load_source(file_path) {
            Ok(source) => match compile(&source) {
                Ok(asm) => Ok(asm),
                Err(errs) => {
                    let mut lock = std::io::stderr().lock();
                    let report_results = report_many(&errs, &mut lock);
                    if let Err(errors) = report_results {
                        let mut stderr = std::io::stderr().lock();
                        for error in errors {
                            let _ = writeln!(stderr, "Failed to report compilation error: {error}");
                        }
                    }
                    Err(AppError::ExecutionFailure(1))
                }
            },
            Err(err) => Err(display_error(err)),
        },
        _ => unreachable!(
            "`cli::validate_file` ensures that the file types are either 'ccl' or 'ccbc' only"
        ),
    }
}

fn compile(source: &str) -> Result<Vec<AsmInstr>, Vec<CompileError<'_>>> {
    let lexemes = scan::scan(source);
    let spans = scan::classify(&lexemes);
    let assembled = assemble(&spans)?;
    let asm = resolve(assembled)?;
    Ok(asm)
}
