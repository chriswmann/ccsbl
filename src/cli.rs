use clap::Parser;
use std::path;

#[derive(Debug, Parser)]
pub struct Cli {
    /// The file to compile or execute
    #[arg(short, long, value_parser = validate_extension)]
    pub file: path::PathBuf,

    #[arg(short, long, default_value_t = false)]
    pub compile: bool,

    #[arg(short, long, default_value_t = false)]
    pub execute: bool,
}

fn validate_extension(path: &str) -> Result<path::PathBuf, String> {
    let path = path::PathBuf::from(path);
    if path.extension().is_some_and(|ext| ext == "ccl") {
        Ok(path)
    } else {
        Err(format!(
            "File must have a .ccl extension: {}",
            path.display()
        ))
    }
}
