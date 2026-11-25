use crate::Node;
use crate::nodes::{HasArgs, HasExitCode, HasMessage, HasTarget, HasVerb};
use crate::utils::runner::Route;
use crate::utils::{process, runner};
use std::io;
use std::path::Path;

/// Where [`Delegate`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The native CLI ran, and left the exit code to report.
    Ran,
    /// There was no native CLI to run, or it could not be started; the message says why.
    Unavailable,
}

pub trait Utils {
    fn exists(&self, path: &str) -> bool;
    /// The contents of the file at `path`, or `None` if it cannot be read.
    fn read(&self, path: &str) -> Option<String>;
    /// Whether `program` is on the `PATH`.
    fn installed(&self, program: &str) -> bool;
    /// Runs `program` with `args` and gives its exit code.
    fn run(&self, program: &str, args: &[&str]) -> io::Result<i32>;
    fn eprint(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn read(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn installed(&self, program: &str) -> bool {
        runner::on_path(program)
    }

    fn run(&self, program: &str, args: &[&str]) -> io::Result<i32> {
        process::run(program, args)
    }

    fn eprint(&self, message: &str) {
        eprintln!("{message}");
    }
}

/// Hands the verb to the target's native CLI, unchanged, and keeps its exit code. `check` never
/// downloads the CLI with a package runner: it is run where the project's own tools are.
#[derive(Clone, Debug, Default)]
pub struct Delegate<U = SystemUtils> {
    utils: U,
}

impl Delegate {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Delegate<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Delegate<U>
where
    S: HasTarget + HasVerb + HasArgs + HasExitCode + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = std::convert::Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Self::Error> {
        let target = state.target();
        let verb = state.verb().to_string();
        let routed = runner::route(
            target,
            &verb,
            state.args(),
            |path| self.utils.exists(path),
            |path| self.utils.read(path),
            |program| self.utils.installed(program),
        );
        let words = match routed {
            Route::Local(words) => words,
            Route::Global(words) if verb != "check" => {
                self.utils.eprint(&runner::diagnostic(target));
                self.utils
                    .eprint(&format!("ambler: running '{}' instead.", words[0]));
                words
            }
            Route::Global(_) | Route::Missing => {
                *state.message_mut() = Some(runner::diagnostic(target));
                *state.exit_code_mut() = 2;
                return Ok(Outcome::Unavailable);
            }
        };
        let refs: Vec<&str> = words[1..].iter().map(String::as_str).collect();
        match self.utils.run(&words[0], &refs) {
            Ok(code) => {
                *state.exit_code_mut() = u8::try_from(code).unwrap_or(1);
                Ok(Outcome::Ran)
            }
            Err(error) => {
                *state.message_mut() = Some(if error.kind() == io::ErrorKind::NotFound {
                    format!("ambler: '{}' not found.", words[0])
                } else {
                    format!("ambler: could not run '{}': {error}", words[0])
                });
                *state.exit_code_mut() = 2;
                Ok(Outcome::Unavailable)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::target::Target;
    use crate::testing::{Fake, State};

    async fn run(fake: &Fake, target: Target, verb: &str, args: &[&str]) -> (Outcome, State) {
        let mut state = State::delegating(target, verb, args);
        let Ok(outcome) = Delegate::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    #[tokio::test]
    async fn runs_the_projects_own_cli_with_the_verb_and_arguments_and_keeps_its_exit_code() {
        let fake = Fake::with_programs(&["cargo", "cargo-ambler"], &[]).exiting(1);

        let (outcome, state) = run(&fake, Target::Rust, "node", &["ask", "--terminal"]).await;

        assert_eq!(outcome, Outcome::Ran);
        assert_eq!(state.exit_code, 1);
        assert_eq!(state.message, None);
        assert_eq!(fake.ran(), ["cargo ambler node ask --terminal"]);
    }

    #[tokio::test]
    async fn runs_a_cargo_ambler_alias_declared_in_the_cargo_config() {
        let fake = Fake::with_programs(
            &["cargo"],
            &[(
                ".cargo/config.toml",
                "[alias]\nambler = \"run --bin cargo-ambler --\"\n",
            )],
        );

        let (outcome, state) = run(&fake, Target::Rust, "check", &[]).await;

        assert_eq!(outcome, Outcome::Ran);
        assert_eq!(state.message, None);
        assert_eq!(fake.ran(), ["cargo ambler check"]);
    }

    #[tokio::test]
    async fn fetches_the_cli_for_node_walk_and_util_when_the_project_has_none() {
        let fake = Fake::with_programs(&["uvx"], &[]);

        let (outcome, state) = run(&fake, Target::Python { package: "app" }, "util", &["x"]).await;

        assert_eq!(outcome, Outcome::Ran);
        assert_eq!(state.exit_code, 0);
        assert!(fake.ran()[0].starts_with("uvx --from git+https://github.com/argenkiwi/ambler-py"));
        assert!(fake.eprinted()[1].contains("running 'uvx' instead"));
    }

    #[tokio::test]
    async fn never_fetches_the_cli_for_check() {
        let fake = Fake::with_programs(&["uvx"], &[]);

        let (outcome, state) = run(&fake, Target::Python { package: "app" }, "check", &[]).await;

        assert_eq!(outcome, Outcome::Unavailable);
        assert_eq!(state.exit_code, 2);
        assert!(state.message.unwrap().contains("uv sync"));
        assert!(fake.ran().is_empty());
    }

    #[tokio::test]
    async fn fails_with_how_to_install_the_cli_instead_of_doing_the_work_itself() {
        let fake = Fake::with_programs(&["cargo"], &[]);

        let (outcome, state) = run(&fake, Target::Rust, "node", &["ask"]).await;

        assert_eq!(outcome, Outcome::Unavailable);
        assert_eq!(state.exit_code, 2);
        assert!(state.message.unwrap().contains("cargo install --git"));
        assert!(fake.ran().is_empty());
    }
}
