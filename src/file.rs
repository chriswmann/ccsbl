use std::fs;
use std::path;

use crate::errors::Error;

pub fn load_source<'src>(file_path: impl AsRef<path::Path>) -> Result<String, Error<'src>> {
    let contents = fs::read_to_string(file_path.as_ref()).map_err(|e| Error::SourceFile {
        path: file_path.as_ref().to_path_buf(),
        source: e,
    })?;
    Ok(contents)
}

#[cfg(test)]
mod tests {

    use std::io;
    use tempfile::{self, tempdir};

    use super::*;

    #[test]
    fn returns_expected_error_when_no_file_at_given_path() {
        let dir = tempdir().expect("should be able to create test directory");
        let missing_path = dir.path().join("missing.ccsbl");
        let result = load_source(&missing_path);
        assert!(matches!(
        result,
        Err(
            Error::SourceFile {
                path, source }
        ) if path == missing_path && source.kind() == io::ErrorKind::NotFound,
        ));
    }

    #[test]
    fn returns_expected_error_when_passed_directory_path() {
        let dir = tempdir().expect("should be able to create test directory");
        let result = load_source(&dir);
        assert!(matches!(
        result,
        Err(
            Error::SourceFile { 
                path, source }
        ) if path == dir.path() && source.kind() == io::ErrorKind::IsADirectory));
    }
}
