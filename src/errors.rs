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

    #[error("Truncated byte code at offset {offset}")]
    TruncatedByteCode { offset: usize },

    #[error("Unknown opcode 0x{opcode:02X} at offset {offset}")]
    UnknownOpcode { opcode: u8, offset: usize },
}

impl Error<'_> {
    pub fn report<W: Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "{self}")?;
        Ok(())
    }
}
