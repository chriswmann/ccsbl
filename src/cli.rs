use clap::Parser;
use std::path;

#[derive(Debug, Parser)]
pub struct Cli {
    /// The file to compile or execute
    #[arg(short, long)]
    pub file: path::PathBuf,

    #[arg(short, long, default_value_t = false)]
    pub compile: bool,

    #[arg(short, long, default_value_t = false)]
    pub execute: bool,
}
