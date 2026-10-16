use std::{error::Error, fmt};
use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::{bytecode::InstructionIndex, token::Token};

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

    #[error("One or more jump targets are for beyond the program length: {0}")]
    JumpTargetOutOfProgram(BadJumpTargets),
}

#[derive(Debug, PartialEq)]
pub struct BadJumpTargets {
    targets: Vec<BadJumpTarget>,
}

impl BadJumpTargets {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    pub fn push(&mut self, bad_target: BadJumpTarget) {
        self.targets.push(bad_target);
    }

    #[cfg(test)]
    pub fn targets(&self) -> Vec<BadJumpTarget> {
        self.targets.to_vec()
    }
}

impl fmt::Display for BadJumpTargets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for target in &self.targets {
            writeln!(f, "{target}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BadJumpTarget {
    target: InstructionIndex,
    idx: InstructionIndex,
}

impl BadJumpTarget {
    pub fn from(target: InstructionIndex, idx: InstructionIndex) -> Self {
        Self { target, idx }
    }
}

impl fmt::Display for BadJumpTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Target: {:16} index: {:16}", self.target, self.idx)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("Stack underflow error")]
    StackUnderFlow,

    #[error("Overflow error")]
    Overflow,

    #[error("Divide by zero error")]
    DivideByZero,

    #[error("Error printing {message} to console ({source})")]
    Report {
        message: String,
        #[source]
        source: std::io::Error,
    },
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

    #[error("Execution failed and cannot report why")]
    ExecutionAndReportingFailure(i32),
}

pub fn report<W: Write>(err: impl std::error::Error, w: &mut W) -> io::Result<()> {
    writeln!(w, "{err}")?;
    Ok(())
}

pub fn report_many<W: Write>(
    errors: &[impl std::error::Error],
    w: &mut W,
) -> Result<(), Vec<io::Error>> {
    let mut write_errors = Vec::new();
    for error in errors {
        if let Err(err) = writeln!(w, "{error}") {
            write_errors.push(err);
        }
    }
    if write_errors.is_empty() {
        Ok(())
    } else {
        Err(write_errors)
    }
}
