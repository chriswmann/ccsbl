use std::path;

use clap::Parser;

use crate::error::OperationError;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg()]
    pub input_file: Option<path::PathBuf>,

    #[arg()]
    pub output_file: Option<path::PathBuf>,
}

#[derive(Debug)]
pub enum Operation {
    Execute {
        input_file: path::PathBuf,
    },
    Serialise {
        input_file: path::PathBuf,
        output_file: path::PathBuf,
    },
}

impl TryFrom<Cli> for Operation {
    type Error = OperationError;
    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        let Cli {
            input_file,
            output_file,
        } = value;
        match (input_file.clone(), output_file.clone()) {
            (None, Some(_) | None) => Err(OperationError::UnknownOperation {
                input_file,
                output_file,
            }),
            (Some(input_file), None) => Ok(Self::Execute { input_file }),
            (Some(input_file), Some(output_file)) => Ok(Self::Serialise {
                input_file,
                output_file,
            }),
        }
    }
}
