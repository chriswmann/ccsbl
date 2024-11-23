#[derive(thiserror::Error, Debug)]
pub enum CcsblError<'a> {
    #[error("Invalid input file path: {0}")]
    FileNotFound(&'a std::path::Path),
}
