use std::io::Write;
use std::path;
use std::process::exit;

use clap::Parser;
use tracing::instrument;
use tracing_subscriber::EnvFilter;

use crate::assembler::{assemble, resolve};
use crate::bytecode::AsmInstr;
use crate::cli::Operation;
use crate::error::{display_error, report, AppError, CompileError};
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
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    let args = cli::Cli::parse();
    let operation = Operation::from(args);
    match operation {
        Operation::Execute { input_file } => {
            let asm = match load(&input_file) {
                Ok(asm) => asm,
                Err(AppError::ExecutionFailure(code)) => {
                    let source = match load_source(&input_file) {
                        Ok(source) => source,
                        Err(err) => {
                            let _ = display_error(err);
                            exit(code);
                        }
                    };
                    match compile(&source) {
                        Ok(asm) => asm,
                        Err(errs) => {
                            let mut code = 0;
                            for err in errs {
                                let AppError::ExecutionFailure(display_err_code) =
                                    display_error(err);
                                code = display_err_code;
                            }
                            exit(code);
                        }
                    }
                }
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
                exit(2);
            }
        }
        Operation::Serialise {
            input_file: _input_file,
            output_file: _output_file,
            force: _force,
        } => println!("TBD"),
    }
}

#[instrument]
fn load(file_path: &path::Path) -> Result<Vec<AsmInstr>, AppError> {
    let progam = match Program::load(file_path) {
        Ok(program) => program,
        Err(file_err) => {
            return Err(display_error(file_err));
        }
    };
    let asm = match progam.decode() {
        Ok(asm) => asm,
        Err(err) => {
            return Err(display_error(err));
        }
    };
    Ok(asm)
}

#[instrument]
fn compile(source: &str) -> Result<Vec<AsmInstr>, Vec<CompileError<'_>>> {
    let lexemes = scan::scan(source);
    let spans = scan::classify(&lexemes);
    let assembled = assemble(&spans)?;
    let asm = resolve(assembled)?;
    Ok(asm)
}
