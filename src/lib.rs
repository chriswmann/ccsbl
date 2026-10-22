use std::path;

use tracing::instrument;

use crate::assembler::{assemble, resolve};
pub use crate::bytecode::AsmInstr;
pub use crate::cli::{Cli, Operation};
pub use crate::error::{display_error, report, AppError, CompileError};
pub use crate::file::load_source;
pub use crate::program::Program;
use crate::scan::{classify, scan};
pub use crate::vm::execute;

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

#[instrument]
pub fn compile(source: &str) -> Result<Vec<AsmInstr>, Vec<CompileError<'_>>> {
    let lexemes = scan(source);
    let spans = classify(&lexemes);
    let assembled = assemble(&spans)?;
    let asm = resolve(assembled)?;
    Ok(asm)
}

#[instrument]
pub fn load(file_path: &path::Path) -> Result<Vec<AsmInstr>, AppError> {
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
