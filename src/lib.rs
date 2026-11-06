use std::path;

use tracing::instrument;

use crate::assembler::{assemble, resolve};
pub use crate::bytecode::AsmInstr;
pub use crate::cli::{Cli, Operation};
pub use crate::error::{Error, display_error, display_multiple_errors, report};
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

/// # Errors
/// Returns an error if the code can't be tokenised and resolved.
#[instrument]
pub fn compile(source: &str) -> Result<Vec<AsmInstr>, Vec<Error>> {
    let assembled = assemble(classify(scan(source)))?;
    let asm = resolve(assembled)?;
    Ok(asm)
}

/// # Errors
/// Returns an error if the source file can't be loaded, or if the
/// code can't be tokenised and resolved.
pub fn compile_file(source_file: &path::Path) -> Result<Vec<AsmInstr>, Vec<Error>> {
    let mut errors = Vec::new();
    match load_source(source_file) {
        Ok(source) => compile(&source),
        Err(err) => {
            errors.push(err);
            Err(errors)
        }
    }
}
