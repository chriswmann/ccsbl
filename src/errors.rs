#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid input file path: {0}")]
    FileNotFound(std::path::PathBuf),

    #[error("Error parsing input: {0}")]
    Parse(String),

    #[error("Value out of range: {0}")]
    ValueOutOfRange(#[from] std::num::TryFromIntError),

    #[error("Unexpected value error: {0}")]
    ValueUnexpected(String),
}
