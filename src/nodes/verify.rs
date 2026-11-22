use crate::Node;
use crate::nodes::{HasMessage, HasTarget};
use crate::utils::process;
use std::io;

/// Where [`Verify`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Formatting, lints and tests all pass.
    Passed,
    /// A command failed and the rest were skipped; the message names it.
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not run `{program}`: {source}")]
    Io { program: String, source: io::Error },
}

pub trait Utils {
    /// Runs `program` with `args` and reports whether it succeeded.
    fn run(&self, program: &str, args: &[&str]) -> io::Result<bool>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn run(&self, program: &str, args: &[&str]) -> io::Result<bool> {
        process::run(program, args)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Runs the target's formatting check, lints and tests (`cargo fmt --check`, clippy and
/// `cargo test`, or `bun run check`, `lint` and `test`), stopping at a failure.
#[derive(Clone, Debug, Default)]
pub struct Verify<U = SystemUtils> {
    utils: U,
}

impl Verify {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Verify<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Verify<U>
where
    S: HasTarget + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        for (program, args) in state.target().checks() {
            let command = format!("{program} {}", args.join(" "));
            self.utils.print(&format!("$ {command}"));
            let passed = self.utils.run(program, args).map_err(|source| Error::Io {
                program: (*program).to_string(),
                source,
            })?;
            if !passed {
                *state.message_mut() = Some(format!("`{command}` failed"));
                return Ok(Outcome::Failed);
            }
        }
        Ok(Outcome::Passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Fake, State};

    #[tokio::test]
    async fn passes_when_formatting_lints_and_tests_do() {
        let fake = Fake::default();
        let mut state = State::default();

        let outcome = Verify::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Passed);
        assert_eq!(
            fake.printed(),
            [
                "$ cargo fmt --check",
                "$ cargo clippy --all-targets -- -D warnings",
                "$ cargo test",
            ]
        );
        assert_eq!(state.message, None);
    }

    #[tokio::test]
    async fn stops_at_the_first_command_that_fails() {
        let fake = Fake {
            failing: Some("clippy"),
            ..Fake::default()
        };
        let mut state = State::default();

        let outcome = Verify::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Failed);
        assert_eq!(fake.printed().len(), 2);
        assert_eq!(
            state.message.as_deref(),
            Some("`cargo clippy --all-targets -- -D warnings` failed")
        );
    }

    #[tokio::test]
    async fn fails_if_cargo_cannot_be_run() {
        let fake = Fake {
            failing: Some("missing"),
            ..Fake::default()
        };

        let result = Verify::with_utils(&fake).run(&mut State::default()).await;

        assert!(matches!(result, Err(Error::Io { program, .. }) if program == "cargo"));
    }

    #[tokio::test]
    async fn runs_bun_for_a_typescript_project() {
        let fake = Fake {
            failing: Some("lint"),
            ..Fake::default()
        };
        let mut state = State::with_target(crate::scaffold::target::Target::TypeScript, &[]);

        let outcome = Verify::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Failed);
        assert_eq!(fake.printed(), ["$ bun run check", "$ bun run lint"]);
        assert_eq!(state.message.as_deref(), Some("`bun run lint` failed"));
    }
}
