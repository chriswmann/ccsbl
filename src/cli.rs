use clap::Parser;
use std::path;

#[derive(Debug, Parser)]
pub struct Cli {
    /// The file to compile or execute
    #[arg(short, long, value_parser = validate_extension)]
    pub file: path::PathBuf,

    /// Print additional debuging information, such as longer slices of lexems and tokens
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
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
