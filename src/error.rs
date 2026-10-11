use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::token::Token;

#[derive(thiserror::Error, Debug)]
pub enum CompileError<'src> {
    #[error("Unrecognised token: '{token}' on line {}", line + 1)]
    Token { token: Token<'src>, line: usize },

    #[error("Expected jump label on line {}, found {token} instead", line + 1)]
    NotAJumpLabel { token: Token<'src>, line: usize },

    #[error("Expected jump label on line {}", line + 1)]
    MissingJumpLabel { line: usize },

    #[error("Could not resolve jump target {target} on line {}", line + 1)]
    ResolveJumpTarget { target: &'src str, line: usize },

    #[error("Label {label} on line {} is not unique", line + 1)]
    NotUniqueLabel { label: &'src str, line: usize },

    #[error("Compiler error")]
    Compiler,
}

#[derive(thiserror::Error, Debug)]
pub enum BytecodeError {
    #[error("Truncated byte code at offset {offset}")]
    TruncatedByteCode { offset: usize },

    #[error("Unknown opcode 0x{opcode:02X} at offset {offset}")]
    UnknownOpcode { opcode: u8, offset: usize },

    #[error(
        "Jump target {target} at offset {offset} exceeds the platform's instruction index range"
    )]
    JumpTargetOutOfRange { target: u64, offset: usize },
}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("Stack underflow error")]
    StackUnderFlow,

    #[error("Overflow error")]
    Overflow,

    #[error("Divide by zero error")]
    DivideByZero,
}

#[derive(thiserror::Error, Debug)]
pub enum FileError {
    #[error("Could not access file {}: {source}", path.display())]
    File {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Unknown save location: {path}")]
    UnknownSaveLocation { path: PathBuf },

    #[error("Unknown file format: {path}")]
    UnknownFileFormat { path: PathBuf },

    #[error("IO error: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Execution failed")]
    ExecutionFailure(i32),

    #[error("Error reporting execution failure(s)")]
    ExecutionAndReportingFailure(i32),
}

pub fn report<W: Write>(err: impl std::error::Error, w: &mut W) -> io::Result<()> {
    writeln!(w, "{err}")?;
    Ok(())
}

pub fn report_many<W: Write>(errors: &[impl std::error::Error], w: &mut W) {
    for error in errors {
        let _ = writeln!(w, "{error}");
    }
}
