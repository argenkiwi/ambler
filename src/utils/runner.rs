//! Where a verb runs: the language-native Ambler CLI of the project's target (see
//! `docs/tooling_protocol.md`).
//!
//! The CLI is looked for in this order: the project's own (`cargo ambler`, an alias in
//! `.cargo/config.toml`; `bun node_modules/.bin/ambler`; `uv run ambler`: a dev dependency that
//! `ambler new` adds, run from the project, so nothing has to be on the `PATH`), then, for
//! TypeScript and Python, a package runner (`bunx`, `uvx`) that fetches it. There is no fallback: with neither, the verb fails and says how to get the
//! CLI.

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

/// Picks where to run `verb`: `exists` tells which files the project has, `read` gives a file's
/// contents (or `None` if it is not there) and `installed` tells which programs are on the
/// `PATH`.
#[must_use]
pub fn route(
    target: Target,
    verb: &str,
    args: &[String],
    exists: impl Fn(&str) -> bool,
    read: impl Fn(&str) -> Option<String>,
    installed: impl Fn(&str) -> bool,
) -> Route {
    if target.has_native_runner(&exists, &read, &installed) {
        let runner = match target {
            // Projects made before the CLI became `ambler` only have `ambler-cli`.
            Target::Python { .. } if !exists(".venv/bin/ambler") => {
                ("uv", &["run", "ambler-cli"][..])
            }
            // Projects made before the CLI became a dependency carry their own `cli.ts`.
            Target::TypeScript if !exists("node_modules/.bin/ambler") => ("bun", &["cli.ts"][..]),
            _ => target.native_runner(),
        };
        return Route::Local(command(runner, verb, args));
    }
    match target.global_runner() {
        Some(global) if installed(global.0) => Route::Global(command(global, verb, args)),
        _ => Route::Missing,
    }
}

/// What to do to get the target's own CLI: install what the project already depends on, add the
/// dependency if it does not have it yet, or, for Rust, install the CLI globally.
#[must_use]
pub fn diagnostic(target: Target) -> String {
    match target {
        Target::Rust => "ambler: the Rust Ambler CLI ('cargo ambler') was not found.\n\
             Tip: Run 'cargo build' if this project has an 'xtask' crate. Otherwise add one that \
             depends on https://github.com/argenkiwi/ambler-rs (see its README), define an \
             'ambler' alias under [alias] in .cargo/config.toml that runs the CLI, or install the \
             CLI with 'cargo install --git https://github.com/argenkiwi/ambler-rs'."
            .to_string(),
        Target::TypeScript => "ambler: the TypeScript Ambler CLI ('ambler') was not found in this \
             project.\n\
             Tip: Run 'bun install' to install the project's dependencies. If it does not depend \
             on the CLI yet, run 'bun add -d github:argenkiwi/ambler-ts'."
            .to_string(),
        Target::Python { .. } => "ambler: the Python Ambler CLI ('ambler') was not found in the \
             virtual environment.\n\
             Tip: Run 'uv sync' to install the project's dependencies. If it does not depend on \
             the CLI yet, run 'uv add --dev git+https://github.com/argenkiwi/ambler-py'."
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
            |p| p == ".venv/bin/ambler",
            |_| None,
            |_| true,
        );
        assert_eq!(
            routed,
            Route::Local(args(&["uv", "run", "ambler", "check", "--lint-only"]))
        );
    }

    #[test]
    fn an_unsynced_python_project_falls_back_to_uvx() {
        let routed = route(PY, "node", &args(&["ask"]), |_| false, |_| None, |_| true);
        assert_eq!(
            routed,
            Route::Global(args(&[
                "uvx",
                "--from",
                "git+https://github.com/argenkiwi/ambler-py",
                "ambler",
                "node",
                "ask"
            ]))
        );
    }

    #[test]
    fn typescript_uses_cli_ts_then_bunx() {
        let local = route(
            Target::TypeScript,
            "walk",
            &[],
            |p| p == "cli.ts",
            |_| None,
            |_| true,
        );
        assert_eq!(local, Route::Local(args(&["bun", "cli.ts", "walk"])));
        let global = route(
            Target::TypeScript,
            "walk",
            &[],
            |_| false,
            |_| None,
            |_| true,
        );
        assert_eq!(
            global,
            Route::Global(args(&["bunx", "github:argenkiwi/ambler-ts", "walk"]))
        );
    }

    #[test]
    fn rust_uses_cargo_ambler_when_installed() {
        let routed = route(
            Target::Rust,
            "node",
            &args(&["ask"]),
            |_| true,
            |_| None,
            |_| true,
        );
        assert_eq!(
            routed,
            Route::Local(args(&["cargo", "ambler", "node", "ask"]))
        );
    }

    #[test]
    fn rust_has_nowhere_else_to_run() {
        let no_ambler = |program: &str| program == "cargo";
        assert_eq!(
            route(Target::Rust, "check", &[], |_| false, |_| None, no_ambler),
            Route::Missing
        );
    }

    #[test]
    fn with_no_cli_installed_there_is_none_to_route_to() {
        assert_eq!(
            route(PY, "check", &[], |_| true, |_| None, |_| false),
            Route::Missing
        );
    }

    #[test]
    fn a_python_project_made_before_the_cli_was_named_ambler_runs_ambler_cli() {
        let routed = route(
            PY,
            "check",
            &[],
            |p| p == ".venv/bin/ambler-cli",
            |_| None,
            |_| true,
        );
        assert_eq!(
            routed,
            Route::Local(args(&["uv", "run", "ambler-cli", "check"]))
        );
    }

    #[test]
    fn rust_runs_its_xtask_alias_without_a_global_install() {
        let only_cargo = |program: &str| program == "cargo";
        let routed = route(
            Target::Rust,
            "node",
            &[],
            |p| p == "xtask/Cargo.toml",
            |_| None,
            only_cargo,
        );
        assert_eq!(routed, Route::Local(args(&["cargo", "ambler", "node"])));
    }

    #[test]
    fn rust_runs_its_cargo_alias_without_an_xtask_or_a_global_install() {
        let only_cargo = |program: &str| program == "cargo";
        let config = |path: &str| {
            (path == ".cargo/config.toml")
                .then(|| "[alias]\nambler = \"run --quiet --bin cargo-ambler --\"\n".to_string())
        };
        let routed = route(Target::Rust, "check", &[], |_| false, config, only_cargo);
        assert_eq!(routed, Route::Local(args(&["cargo", "ambler", "check"])));
    }

    #[test]
    fn typescript_runs_the_installed_dependency() {
        let routed = route(
            Target::TypeScript,
            "util",
            &[],
            |p| p == "node_modules/.bin/ambler",
            |_| None,
            |_| true,
        );
        assert_eq!(
            routed,
            Route::Local(args(&["bun", "node_modules/.bin/ambler", "util"]))
        );
    }

    #[test]
    fn each_diagnostic_says_how_to_install_the_dependency_or_the_cli() {
        let rust = diagnostic(Target::Rust);
        assert!(rust.contains("cargo build"));
        assert!(rust.contains(".cargo/config.toml"));
        assert!(rust.contains("cargo install --git https://github.com/argenkiwi/ambler-rs"));
        let ts = diagnostic(Target::TypeScript);
        assert!(ts.contains("bun install"));
        assert!(ts.contains("bun add -d github:argenkiwi/ambler-ts"));
        let py = diagnostic(PY);
        assert!(py.contains("uv sync"));
        assert!(py.contains("uv add --dev git+https://github.com/argenkiwi/ambler-py"));
    }
}
