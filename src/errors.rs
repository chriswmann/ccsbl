#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not read source file: {0}")]
    SourceFileRead(#[from] std::io::Error),

    #[error("Unrecognised token: '{token}' on line {line_number}")]
    Token { line_number: usize, token: String },
}
