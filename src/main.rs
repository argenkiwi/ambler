//! The `ambler` command line: runs the walk for the verb in `args[0]`; see `specs/*.md`.
//!
//! The target language is worked out here, once, for every verb: see `scaffold::target`.

use ambler::guides;
use ambler::scaffold::target::{self, Target};
use ambler::walks::{self, Error};
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
        |path| {
            let path = Path::new(path);
            // Only `pyproject.toml` is read; other markers only need to exist.
            path.is_file()
                .then(|| std::fs::read_to_string(path).unwrap_or_default())
        },
    );
    let (target, rest): (Target, Vec<String>) = match resolved {
        Ok(resolved) => resolved,
        Err(message) => {
            eprintln!("ambler: {message}");
            return ExitCode::FAILURE;
        }
    };
    exit_code(match verb.as_str() {
        "new" => walks::new::run(target, rest).await,
        // What each target's native CLI does best is left to it: see `docs/tooling_protocol.md`.
        "node" | "util" | "walk" | "check" => walks::delegate::run(target, &verb, rest).await,
        "pave" => walks::pave::run(target, rest).await,
        // A project's components and conventions are documented by its own CLI.
        "help" if rest.first().is_some_and(|topic| guides::is_native(topic)) => {
            walks::delegate::run(target, &verb, rest).await
        }
        "help" => walks::help::run(target, rest).await,
        "" => walks::help::run(target, Vec::new()).await,
        _ => walks::help::run(target, vec![verb]).await,
    })
}

/// Exit 1 on any failure, or with the native CLI's own code when a verb was handed to it. A
/// [`Error::Failed`] or [`Error::Exit`] walk has already said why; other errors are printed here.
fn exit_code(result: Result<(), Error>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Failed) => ExitCode::FAILURE,
        Err(Error::Exit(code)) => ExitCode::from(code),
        Err(error) => {
            eprintln!("ambler: {error}");
            ExitCode::FAILURE
        }
    }
}
