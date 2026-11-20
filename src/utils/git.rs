//! The git repository around the current directory: where it keeps its files, what changed,
//! checkpoints and roll backs.

use std::io;
use std::process::{Command, Output};

fn git(args: &[&str]) -> io::Result<Output> {
    Command::new("git").args(args).output()
}

/// Runs git with `args`, failing with its standard error if it does not succeed.
fn succeed(args: &[&str]) -> io::Result<String> {
    let output = git(args)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(io::Error::other(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// The absolute path of the repository's git folder, or `None` if the current directory is not
/// in a repository or git is not installed.
#[must_use]
pub fn git_dir() -> Option<String> {
    let output = git(&["rev-parse", "--absolute-git-dir"]).ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// The uncommitted changes, one `git status --short` line each.
///
/// # Errors
///
/// Returns the error if git cannot be run or fails.
pub fn status() -> io::Result<String> {
    succeed(&["status", "--short"])
}

/// Commits every change, untracked files included, with `subject` and `body` as its message.
///
/// # Errors
///
/// Returns the error if git cannot be run or fails, for example without an identity.
pub fn commit_all(subject: &str, body: &str) -> io::Result<()> {
    succeed(&["add", "-A"])?;
    succeed(&["commit", "-q", "-m", subject, "-m", body]).map(drop)
}

/// Stashes every change, untracked files included, under `message`.
///
/// # Errors
///
/// Returns the error if git cannot be run or fails.
pub fn stash_all(message: &str) -> io::Result<()> {
    succeed(&["stash", "push", "-u", "-q", "-m", message]).map(drop)
}
