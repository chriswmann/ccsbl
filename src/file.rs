use std::fs;
use std::path;

use crate::errors::Error;

pub fn load_file<'src>(file_path: impl AsRef<path::Path>) -> Result<String, Error<'src>> {
    let contents = fs::read_to_string(&file_path)?;
    Ok(contents)
}
