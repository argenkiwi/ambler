use crate::Node;
use crate::nodes::HasMessage;
use crate::utils::process;
use std::io;

/// What a project has to pass, in the order it is run.
const COMMANDS: [&[&str]; 3] = [
    &["fmt", "--check"],
    &["clippy", "--all-targets", "--", "-D", "warnings"],
    &["test"],
];

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
    #[error("could not run cargo: {0}")]
    Io(#[from] io::Error),
}

pub trait Utils {
    /// Runs `cargo` with `args` and reports whether it succeeded.
    fn cargo(&self, args: &[&str]) -> io::Result<bool>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn cargo(&self, args: &[&str]) -> io::Result<bool> {
        process::run("cargo", args)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Runs `cargo fmt --check`, clippy with warnings denied and the tests, stopping at a failure.
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
    S: HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        for args in COMMANDS {
            let command = format!("cargo {}", args.join(" "));
            self.utils.print(&format!("$ {command}"));
            if !self.utils.cargo(args)? {
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

        assert!(matches!(result, Err(Error::Io(_))));
    }
}
