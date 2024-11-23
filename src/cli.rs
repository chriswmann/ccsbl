use clap::Parser;
use std::path;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub input_file_path: path::PathBuf,
}
