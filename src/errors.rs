#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not read source file: {0}")]
    SourceFileRead(#[from] std::io::Error),

    #[error("Unrecognised token: '{0}'")]
    SyntaxError(String),
}
