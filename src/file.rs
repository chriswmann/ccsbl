use std::fs;
use std::path;

pub fn load_file(file_path: &path::Path) -> Vec<String> {
    fs::read_to_string(file_path)
        .expect("Couldn't open path: {file_path}")
        .lines()
        .map(|l| l.to_owned())
        .collect()
}
