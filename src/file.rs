use std::fs;
use std::path;

use crate::errors::Error;

pub fn load_file(file_path: path::PathBuf) -> Result<Vec<String>, Error> {
    match fs::read_to_string(&file_path) {
        Ok(content) => Ok(content.lines().map(&str::to_owned).collect()),
        Err(_) => Err(crate::errors::Error::FileNotFound(file_path)),
    }
}
