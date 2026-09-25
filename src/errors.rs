use crate::bytecode::Instr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not read source file: {0}")]
    SourceFileRead(#[from] std::io::Error),

    #[error("Unrecognised token: '{token}' on line {line_number}")]
    Token { token: String, line_number: usize },

    #[error("Jump label found instead of instruction: '{label}' on line {line_number}")]
    Encode { label: String, line_number: usize },

    #[error("Truncated byte code at offset {offset}")]
    TruncatedByteCode { offset: usize },

    #[error("Unknown opcode 0x{opcode:02X} at offset {offset}")]
    UnknownOpcode { opcode: u8, offset: usize },
}
