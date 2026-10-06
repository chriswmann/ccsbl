use std::io::{self, Write};

use crate::token::Token;

#[derive(thiserror::Error, Debug)]
pub enum Error<'src> {
    #[error("Could not read source file {}: {source}", path.display())]
    SourceFile {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

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

    #[error("Truncated byte code at offset {offset}")]
    TruncatedByteCode { offset: usize },

    #[error("Unknown opcode 0x{opcode:02X} at offset {offset}")]
    UnknownOpcode { opcode: u8, offset: usize },

    #[error(
        "Jump target {target} at offset {offset} exceeds the platform's instruction index range"
    )]
    JumpTargetOutOfRange { target: u64, offset: usize },

    #[error("Stack underflow error")]
    StackUnderFlow,

    #[error("Overflow error")]
    Overflow,

    #[error("Divide by zero error")]
    DivideByZero,

    #[error("Compiler error")]
    Compiler,
}

impl Error<'_> {
    pub fn report<W: Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "{self}")?;
        Ok(())
    }

    pub fn report_many<W: Write>(errors: &[Self], w: &mut W) {
        for error in errors {
            let _ = error.report(w);
        }
    }
}
