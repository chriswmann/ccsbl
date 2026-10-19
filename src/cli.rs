use std::path;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg()]
    pub input_file: Option<path::PathBuf>,

    #[arg(short, long, requires = "input_file")]
    pub output_file: Option<path::PathBuf>,

    #[arg(short, long, requires = "output_file")]
    pub force: bool,
}

#[derive(Debug)]
pub enum Operation {
    Execute {
        input_file: path::PathBuf,
    },
    Serialise {
        input_file: path::PathBuf,
        output_file: path::PathBuf,
        force: bool,
    },
}

impl From<Cli> for Operation {
    fn from(value: Cli) -> Self {
        let Cli {
            input_file,
            output_file,
            force,
        } = value;
        match (input_file, output_file, force) {
            (None, ..) => {
                eprintln!("The REPL is not implemented yet.");
                std::process::exit(0);
            }
            (Some(input_file), None, _) => Self::Execute { input_file },
            (Some(input_file), Some(output_file), force) => Self::Serialise {
                input_file,
                output_file,
                force,
            },
        }
    }
}
