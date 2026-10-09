use tracing_subscriber::EnvFilter;

mod assembler;
mod bytecode;
mod cli;
mod error;
mod file;
mod program;
mod scan;
mod token;
mod vm;

#[derive(Debug)]
enum RunStatus {
    Success,
    // 1: compilation or execution failure, reported
    // 2: compilation or execution failure, failure while reporting
    Failure(i32),
}

fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();
}
