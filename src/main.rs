use std::io::Write;
use std::process::exit;

use clap::Parser;

use tracing_subscriber::EnvFilter;

use ccsbl::{compile, display_error, execute, load, load_source, report, AppError, Cli, Operation};

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
            let asm = match load(&input_file) {
                Ok(asm) => asm,
                Err(AppError::ExecutionFailure(code)) => {
                    let source = match load_source(&input_file) {
                        Ok(source) => source,
                        Err(err) => {
                            let _ = display_error(err);
                            exit(code);
                        }
                    };
                    match compile(&source) {
                        Ok(asm) => asm,
                        Err(errs) => {
                            let mut code = 0;
                            for err in errs {
                                let AppError::ExecutionFailure(display_err_code) =
                                    display_error(err);
                                code = display_err_code;
                            }
                            exit(code);
                        }
                    }
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
