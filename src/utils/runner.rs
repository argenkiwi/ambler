//! Where a verb runs: the language-native Ambler CLI of the project's target (see
//! `docs/tooling_protocol.md`).
//!
//! The CLI is looked for in this order: the project's own (`cargo ambler`, `bun run ambler`,
//! `uv run ambler-cli`), then, for TypeScript and Python, a package runner (`bunx`, `uvx`) that
//! fetches it. There is no fallback: with neither, the verb fails and says how to get the CLI.

use crate::scaffold::target::Target;

/// Where a verb is run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// The project's own CLI.
    Local(Vec<String>),
    /// A package runner that fetches the CLI.
    Global(Vec<String>),
    /// No CLI: the project has not been set up, and no package runner is installed.
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
    if target.has_native_runner(&exists, &installed) {
        return Route::Local(command(target.native_runner(), verb, args));
    }
    match target.global_runner() {
        Some(global) if installed(global.0) => Route::Global(command(global, verb, args)),
        _ => Route::Missing,
    }
}

/// What to do to get the target's own CLI.
#[must_use]
pub fn diagnostic(target: Target) -> String {
    match target {
        Target::Rust => "ambler: 'cargo-ambler' not found.\n\
             Tip: Run 'cargo install --git https://github.com/argenkiwi/ambler-rs cargo-ambler' \
             to install it."
            .to_string(),
        Target::TypeScript => "ambler: 'ambler' script (cli.ts) not found in this project.\n\
             Tip: Did you run 'bun install' in this project? Run 'bun install' to install \
             project dependencies."
            .to_string(),
        Target::Python { .. } => "ambler: 'ambler-cli' not found in virtual environment.\n\
             Tip: Did you run 'uv sync' in this project? Run 'uv sync' to install project \
             dependencies."
            .to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    const PY: Target = Target::Python { package: "app" };

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
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
            Route::Local(args(&["uv", "run", "ambler-cli", "check", "--lint-only"]))
        );
    }

    #[test]
    fn an_unsynced_python_project_falls_back_to_uvx() {
        let routed = route(PY, "node", &args(&["ask"]), |_| false, |_| true);
        assert_eq!(
            routed,
            Route::Global(args(&[
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
        assert_eq!(local, Route::Local(args(&["bun", "run", "ambler", "walk"])));
        let global = route(Target::TypeScript, "walk", &[], |_| false, |_| true);
        assert_eq!(
            global,
            Route::Global(args(&["bunx", "github:argenkiwi/ambler-ts", "walk"]))
        );
    }

    #[test]
    fn rust_uses_cargo_ambler_when_installed() {
        let routed = route(Target::Rust, "node", &args(&["ask"]), |_| true, |_| true);
        assert_eq!(
            routed,
            Route::Local(args(&["cargo", "ambler", "node", "ask"]))
        );
    }

    #[test]
    fn rust_has_nowhere_else_to_run() {
        let no_ambler = |program: &str| program == "cargo";
        assert_eq!(
            route(Target::Rust, "check", &[], |_| true, no_ambler),
            Route::Missing
        );
    }

    #[test]
    fn with_no_cli_installed_there_is_none_to_route_to() {
        assert_eq!(route(PY, "check", &[], |_| true, |_| false), Route::Missing);
    }

    #[test]
    fn each_diagnostic_says_how_to_get_the_cli() {
        assert!(diagnostic(Target::Rust).contains("cargo install --git"));
        assert!(diagnostic(Target::TypeScript).contains("bun install"));
        assert_eq!(
            diagnostic(PY),
            "ambler: 'ambler-cli' not found in virtual environment.\nTip: Did you run 'uv sync' \
             in this project? Run 'uv sync' to install project dependencies."
        );
    }
}
