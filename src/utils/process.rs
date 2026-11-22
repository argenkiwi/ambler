//! Running other programs: the checks, the formatters, amble itself and a coding agent.

use crate::scaffold::target::Target;
use std::fs::File;
use std::io;
use std::process::{Command, Stdio};

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

/// Runs this program again with `args` and the environment variables in `env`, writing its
/// standard output and error, in the order they come, to the file at `log`. Reports whether
/// it succeeded.
///
/// # Errors
///
/// Returns the I/O error if the log cannot be created or the program cannot be started.
pub fn rerun_logged(args: &[&str], env: &[(&str, &str)], log: &str) -> io::Result<bool> {
    let out = File::create(log)?;
    let err = out.try_clone()?;
    Ok(Command::new(std::env::current_exe()?)
        .args(args)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .status()?
        .success())
}

/// Runs the shell `command` with `argument` added as its last argument, sharing this process's
/// terminal, and reports whether it succeeded.
///
/// The command may carry its own flags (`claude -p`); the argument is passed as one word
/// whatever it contains.
///
/// # Errors
///
/// Returns the I/O error if the shell could not be started.
pub fn shell_with(command: &str, argument: &str) -> io::Result<bool> {
    Ok(Command::new("sh")
        .arg("-c")
        .arg(format!("{command} \"$1\""))
        .arg("amble")
        .arg(argument)
        .status()?
        .success())
}

/// Formats `target`'s source files at `paths` in place: rustfmt for Rust, Biome for TypeScript.
///
/// Best effort: generated code is valid without it, so a missing formatter is not an error.
/// Biome is only run once `bun install` has put it in `node_modules`.
pub fn format(target: Target, paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    let _ = match target {
        Target::Rust => Command::new("rustfmt")
            .args(["--edition", EDITION])
            .args(paths)
            .status(),
        Target::TypeScript => {
            let biome = "node_modules/.bin/biome";
            if !std::path::Path::new(biome).exists() {
                return;
            }
            Command::new(biome)
                .args(["check", "--write", "--no-errors-on-unmatched"])
                .args(paths)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        }
    };
}
