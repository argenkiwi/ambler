//! The `amble` command line: runs the walk for the verb in `args[0]`; see `specs/*.md`.

use amble::walks::{self, Error};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let verb = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    exit_code(match verb.as_str() {
        "new" => walks::new::run(rest).await,
        "node" => walks::node::run(rest).await,
        "util" => walks::util::run(rest).await,
        "walk" => walks::walk::run(rest).await,
        "check" => walks::check::run(rest).await,
        "help" => walks::help::run(rest).await,
        "" => walks::help::run(Vec::new()).await,
        _ => walks::help::run(vec![verb]).await,
    })
}

/// Exit 1 on any failure. A [`Error::Failed`] walk has already said why; other errors are
/// printed here.
fn exit_code(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Failed) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("amble: {error}");
            ExitCode::FAILURE
        }
    }
}
