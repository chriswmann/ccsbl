use crate::token::Token;
#[derive(thiserror::Error, Debug)]
pub enum Error<'src> {
    #[error("Could not read source file: {0}")]
    SourceFileRead(#[from] std::io::Error),

    #[error("Unrecognised token: '{token}' on line {line}")]
    Token { token: Token<'src>, line: usize },

    #[error("Assembler error: {msg} on line {line}")]
    Assembler { msg: String, line: usize },

    #[error("Truncated byte code at offset {offset}")]
    TruncatedByteCode { offset: usize },

    #[error("Unknown opcode 0x{opcode:02X} at offset {offset}")]
    UnknownOpcode { opcode: u8, offset: usize },
}
