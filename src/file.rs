use std::fs;
use std::path;

use crate::errors::Error;

pub fn load_file(file_path: impl AsRef<path::Path>) -> Result<String, Error> {
    let contents = fs::read_to_string(&file_path)?;
    Ok(contents)
}
