// use std::path;

use tracing::instrument;

// use crate::assembler::{assemble, resolve};
pub use crate::bytecode::AsmInstr;
pub use crate::cli::{Cli, Operation};
pub use crate::error::{Error, display_error, report};
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
pub fn compile(source: &str) -> Result<Vec<AsmInstr>, Vec<Error>> {
    let lexemes = scan(source);
    // let spans = classify(&lexemes);
    // let assembled = assemble(&spans)?;
    // let asm = resolve(assembled)?;
    // Ok(asm)
    Ok(Vec::new())
}

// #[instrument]
// pub fn load(file_path: &path::Path) -> Result<Vec<AsmInstr>, Error> {
//     let progam = match Program::load(file_path) {
//         Ok(program) => program,
//         Err(file_err) => {
//             return Err(display_error(file_err));
//         }
//     };
//     let asm = match progam.decode() {
//         Ok(asm) => asm,
//         Err(err) => {
//             return Err(display_error(err));
//         }
//     };
//     Ok(asm)
// }
//
// #[instrument]
// pub fn compile_file(input_file: &path::Path) -> Result<Vec<AsmInstr>, Error> {
//     let asm = match load(&input_file) {
//         Ok(asm) => asm,
//         Err(Error::ExecutionFailure(code)) => {
//             let source = load_source(&input_file)?;
//             match compile(&source) {
//                 Ok(asm) => asm,
//                 Err(errs) => {
//                     for err in errs {
//                         display_error(err);
//                     }
//                     Err(Error::ExecutionFailure(code))?
//                 }
//             }
//         }
//     };
//     Ok(asm)
// }
//
// #[instrument]
// pub fn serialise(
//     input_file: &path::Path,
//     output_file: &mut path::Path,
//     force: bool,
// ) -> Result<(), Error> {
//     let asm = compile_file(input_file)?;
//     let program = Program::encode(&asm);
//     program.save(output_file)?;
//     Ok(())
// }
