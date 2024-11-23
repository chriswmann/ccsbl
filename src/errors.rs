#[derive(thiserror::Error, Debug)]
pub enum CcsblError<'a> {
    #[error("Invalid input: {0}")]
    FileNotFound(&'a std::path::Path),
}
