use crate::Node;
use crate::nodes::{HasMessage, HasTarget};
use crate::scaffold::lint;
use crate::scaffold::target::Target;
use crate::utils::fs;
use std::collections::BTreeMap;
use std::io;

/// Where [`Inspect`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The project follows every structural convention.
    Clean,
    /// Violations were printed; the message counts them.
    Violations,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read the project: {0}")]
    Io(#[from] io::Error),
}

pub trait Utils {
    /// The project's manifest, sources and specs, by path, as `target` lays them out.
    fn read_project(&self, target: Target) -> io::Result<BTreeMap<String, String>>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn read_project(&self, target: Target) -> io::Result<BTreeMap<String, String>> {
        fs::read_project(target)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Checks the project in the current directory against the structural conventions.
#[derive(Clone, Debug, Default)]
pub struct Inspect<U = SystemUtils> {
    utils: U,
}

impl Inspect {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Inspect<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Inspect<U>
where
    S: HasTarget + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        let target = state.target();
        let violations = lint(target, &self.utils.read_project(target)?);
        for violation in &violations {
            self.utils.print(&violation.to_string());
        }
        Ok(match violations.len() {
            0 => Outcome::Clean,
            count => {
                *state.message_mut() = Some(format!(
                    "{count} convention violation{}; `amble help check` explains each rule",
                    if count == 1 { "" } else { "s" }
                ));
                Outcome::Violations
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Fake, State};

    const PROJECT: &[(&str, &str)] = &[
        ("Cargo.toml", "[package]\nname = \"app\"\n"),
        ("src/nodes/mod.rs", ""),
        ("src/walks/mod.rs", ""),
        ("src/utils/mod.rs", ""),
    ];

    #[tokio::test]
    async fn a_project_that_follows_the_conventions_is_clean() {
        let fake = Fake::with_files(PROJECT);
        let mut state = State::default();

        let outcome = Inspect::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Clean);
        assert_eq!(state.message, None);
        assert!(fake.printed().is_empty());
    }

    #[tokio::test]
    async fn prints_each_violation_and_counts_them() {
        let mut files = PROJECT.to_vec();
        files.push(("src/utils/clock.rs", "//! Time.\n"));
        let fake = Fake::with_files(&files);
        let mut state = State::default();

        let outcome = Inspect::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Violations);
        assert_eq!(
            fake.printed(),
            ["src/utils/clock.rs:1: registered: add `pub mod clock;` to src/utils/mod.rs"]
        );
        assert!(
            state
                .message
                .unwrap()
                .starts_with("1 convention violation;")
        );
    }

    #[tokio::test]
    async fn fails_if_the_project_cannot_be_read() {
        let fake = Fake {
            unreadable: true,
            ..Fake::default()
        };
        let mut state = State::default();

        let result = Inspect::with_utils(&fake).run(&mut state).await;

        assert!(matches!(result, Err(Error::Io(_))));
        assert_eq!(state.message, None);
    }
}
