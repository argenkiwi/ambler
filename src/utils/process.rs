//! Running other programs: a native CLI, ambler itself and a coding agent.

use std::fs::File;
use std::io;
use std::process::{Command, Stdio};

/// Runs `program` with `args`, sharing this process's terminal, and gives its exit code (1 if it
/// was ended by a signal).
///
/// # Errors
///
/// Returns the I/O error if the program could not be started.
pub fn run(program: &str, args: &[&str]) -> io::Result<i32> {
    Ok(Command::new(program)
        .args(args)
        .status()?
        .code()
        .unwrap_or(1))
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
        .arg("ambler")
        .arg(argument)
        .status()?
        .success())
}
