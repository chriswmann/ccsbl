use clap::Parser;
use std::path;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(value_parser = validate_file)]
    pub input_file: Option<path::PathBuf>,

    #[arg(value_parser = validate_file)]
    pub output_file: Option<path::PathBuf>,
}

fn validate_file(path: &str) -> Result<path::PathBuf, String> {
    let path = path::PathBuf::from(path);
    if path
        .extension()
        .is_some_and(|ext| ext == "ccl" || ext == "ccbc")
    {
        Ok(path)
    } else {
        Err(format!(
            "File must have a .ccl extension for source code or .ccbc for bytecocde: {}",
            path.display()
        ))
    }
}
