use std::fs;
use std::path;

pub fn load_file(file_path: &path::Path) -> Result<Vec<String>, crate::errors::CcsblError> {
    match fs::read_to_string(file_path) {
        Ok(content) => Ok(content.lines().map(|l| l.to_owned()).collect()),
        Err(_) => Err(crate::errors::CcsblError::FileNotFound(file_path)),
    }
}
