//! The `amble` command line: runs the walk for the verb in `args[0]`; see `specs/*.md`.
//!
//! The target language is worked out here, once, for every verb: see `scaffold::target`.

use amble::scaffold::target::{self, Target};
use amble::walks::{self, Error};
use std::path::Path;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let verb = args.next().unwrap_or_default();
    let resolved = target::resolve(
        &verb,
        args.collect(),
        std::env::var(target::VAR).ok(),
        |path| Path::new(path).exists(),
    );
    let (target, rest): (Target, Vec<String>) = match resolved {
        Ok(resolved) => resolved,
        Err(message) => {
            eprintln!("amble: {message}");
            return ExitCode::FAILURE;
        }
    };
    exit_code(match verb.as_str() {
        "new" => walks::new::run(target, rest).await,
        "node" => walks::node::run(target, rest).await,
        "util" => walks::util::run(target, rest).await,
        "walk" => walks::walk::run(target, rest).await,
        "check" => walks::check::run(target, rest).await,
        "pave" => walks::pave::run(target, rest).await,
        "help" => walks::help::run(target, rest).await,
        "" => walks::help::run(target, Vec::new()).await,
        _ => walks::help::run(target, vec![verb]).await,
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
