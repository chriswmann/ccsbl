use std::io::Write;
use std::process::exit;

use clap::Parser;

use tracing_subscriber::EnvFilter;

use ccsbl::{compile_file, display_error, execute, report, AppError, Cli, Operation};

fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    let args = Cli::parse();
    let operation = Operation::from(args);
    match operation {
        Operation::Execute { input_file } => {
            let asm = match compile_file(&input_file) {
                Ok(asm) => asm,
                Err(app_err) => {
                    let AppError::ExecutionFailure(code) = display_error(app_err);
                    exit(code);
                }
            };
            let mut stdout = std::io::stdout().lock();

            let execution_result = execute(&asm, &mut stdout);
            let flush_result = stdout.flush();
            drop(stdout);

            if execution_result.is_err() || flush_result.is_err() {
                let mut stderr = std::io::stderr().lock();

                if let Err(err) = execution_result {
                    let _ = report(err, &mut stderr);
                }

                if let Err(err) = flush_result {
                    let _ = writeln!(stderr, "Failed to flush program output: {err}");
                }
                exit(2);
            }
        }
        Operation::Serialise {
            input_file: _input_file,
            output_file: _output_file,
            force: _force,
        } => println!("TBD"),
    }
}
