use clap::{Parser, Subcommand};
use std::path;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Load {
        #[arg(value_parser = validate_file)]
        file_path: path::PathBuf,
    },
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
