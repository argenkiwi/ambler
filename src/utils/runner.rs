//! Handing a verb to the language-native Ambler runner (see `specs/tooling_protocol.md`).
//!
//! The runner is looked for in this order: the project's own (`.venv/bin/ambler-cli`, `cli.ts`),
//! then a package runner (`uvx`, `bunx`) that fetches it, and otherwise there is none and the
//! caller falls back to amble's own behaviour, after saying how to get the project's runner.

use crate::scaffold::target::Target;
use crate::utils::process;
use std::io;
use std::path::Path;

/// Where a verb is run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// The project's own runner.
    Local(Vec<String>),
    /// A package runner that fetches the tool.
    Global(Vec<String>),
    /// No runner: the project has not been set up, and no package runner is installed.
    Missing,
}

fn command(prefix: (&str, &[&str]), verb: &str, args: &[String]) -> Vec<String> {
    let (program, rest) = prefix;
    std::iter::once(program)
        .chain(rest.iter().copied())
        .chain(std::iter::once(verb))
        .map(ToString::to_string)
        .chain(args.iter().cloned())
        .collect()
}

/// Picks where to run `verb`: `exists` tells which files the project has and `installed` which
/// programs are on the `PATH`.
#[must_use]
pub fn route(
    target: Target,
    verb: &str,
    args: &[String],
    exists: impl Fn(&str) -> bool,
    installed: impl Fn(&str) -> bool,
) -> Route {
    if let Some(local) = target.native_runner() {
        if target.has_native_runner(&exists) && installed(local.0) {
            return Route::Local(command(local, verb, args));
        }
    }
    match target.global_runner() {
        Some(global) if installed(global.0) => Route::Global(command(global, verb, args)),
        _ => Route::Missing,
    }
}

/// The message for a project whose own runner is not installed, if its target has one.
#[must_use]
pub fn diagnostic(target: Target) -> Option<String> {
    match target {
        Target::Rust => None,
        Target::TypeScript => Some(
            "amble: 'ambler' script (cli.ts) not found in this project.\n\
             Tip: Did you run 'bun install' in this project? Run 'bun install' to install \
             project dependencies."
                .to_string(),
        ),
        Target::Python { .. } => Some(
            "amble: 'ambler-cli' not found in virtual environment.\n\
             Tip: Did you run 'uv sync' in this project? Run 'uv sync' to install project \
             dependencies."
                .to_string(),
        ),
    }
}

/// Whether `program` can be found on the `PATH`.
#[must_use]
pub fn on_path(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            let file = dir.join(program);
            file.is_file() || file.with_extension("exe").is_file()
        })
    })
}

/// Runs `verb` with `args` through the target's native runner.
///
/// Returns `Ok(None)` if there is none to run, after printing how to install the project's own;
/// the caller then does the work itself. Otherwise reports whether the runner succeeded.
///
/// # Errors
///
/// Returns the I/O error if the runner was found but could not be started for another reason
/// than being missing.
pub fn delegate(target: Target, verb: &str, args: &[String]) -> io::Result<Option<bool>> {
    dispatch(target, verb, args, true)
}

/// Like [`delegate`], but never fetches the tool with a package runner: for verbs, like
/// `check`, that are better done by amble itself than by a download.
///
/// # Errors
///
/// As [`delegate`].
pub fn delegate_local(target: Target, verb: &str, args: &[String]) -> io::Result<Option<bool>> {
    dispatch(target, verb, args, false)
}

fn dispatch(target: Target, verb: &str, args: &[String], fetch: bool) -> io::Result<Option<bool>> {
    if target.native_runner().is_none() {
        return Ok(None);
    }
    let routed = route(target, verb, args, |path| Path::new(path).exists(), on_path);
    let words = match &routed {
        Route::Local(words) => words,
        Route::Global(words) if fetch => {
            if let Some(message) = diagnostic(target) {
                eprintln!("{message}");
            }
            eprintln!("amble: running '{}' instead.", words[0]);
            words
        }
        Route::Global(_) | Route::Missing => {
            if let Some(message) = diagnostic(target) {
                eprintln!("{message}");
            }
            return Ok(None);
        }
    };
    let refs: Vec<&str> = words[1..].iter().map(String::as_str).collect();
    match process::run(&words[0], &refs) {
        Ok(ok) => Ok(Some(ok)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            eprintln!("amble: '{}' not found.", words[0]);
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PY: Target = Target::Python { package: "app" };

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    fn words(words: &[&str]) -> Vec<String> {
        args(words)
    }

    #[test]
    fn a_synced_python_project_runs_its_own_cli() {
        let routed = route(
            PY,
            "check",
            &args(&["--lint-only"]),
            |p| p == ".venv/bin/ambler-cli",
            |_| true,
        );
        assert_eq!(
            routed,
            Route::Local(words(&["uv", "run", "ambler-cli", "check", "--lint-only"]))
        );
    }

    #[test]
    fn an_unsynced_python_project_falls_back_to_uvx() {
        let routed = route(PY, "node", &args(&["ask"]), |_| false, |_| true);
        assert_eq!(
            routed,
            Route::Global(words(&[
                "uvx",
                "--from",
                "git+https://github.com/argenkiwi/ambler-py",
                "ambler-cli",
                "node",
                "ask"
            ]))
        );
    }

    #[test]
    fn typescript_uses_cli_ts_then_bunx() {
        let local = route(Target::TypeScript, "walk", &[], |p| p == "cli.ts", |_| true);
        assert_eq!(
            local,
            Route::Local(words(&["bun", "run", "ambler", "walk"]))
        );
        let global = route(Target::TypeScript, "walk", &[], |_| false, |_| true);
        assert_eq!(
            global,
            Route::Global(words(&["bunx", "github:argenkiwi/ambler-ts", "walk"]))
        );
    }

    #[test]
    fn with_no_runner_installed_there_is_none_to_route_to() {
        assert_eq!(route(PY, "check", &[], |_| true, |_| false), Route::Missing);
        assert_eq!(
            route(Target::Rust, "check", &[], |_| true, |_| true),
            Route::Missing
        );
    }

    #[test]
    fn the_python_diagnostic_says_to_run_uv_sync() {
        assert_eq!(
            diagnostic(PY).unwrap(),
            "amble: 'ambler-cli' not found in virtual environment.\nTip: Did you run 'uv sync' \
             in this project? Run 'uv sync' to install project dependencies."
        );
        assert_eq!(diagnostic(Target::Rust), None);
    }
}
