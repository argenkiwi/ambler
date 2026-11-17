//! Running other programs: cargo and rustfmt.

use std::io;
use std::process::Command;

/// Edition the generated code is written for.
const EDITION: &str = "2024";

/// Runs `program` with `args`, sharing this process's terminal, and reports whether it succeeded.
///
/// # Errors
///
/// Returns the I/O error if the program could not be started.
pub fn run(program: &str, args: &[&str]) -> io::Result<bool> {
    Ok(Command::new(program).args(args).status()?.success())
}

/// Formats the Rust files at `paths` in place.
///
/// Best effort: generated code is valid without it, so a missing rustfmt is not an error.
pub fn rustfmt(paths: &[String]) {
    if !paths.is_empty() {
        let _ = Command::new("rustfmt")
            .args(["--edition", EDITION])
            .args(paths)
            .status();
    }
}
