//! `amble node`, `walk`, `util` and `check`: hand the verb to the target's native CLI. See
//! `specs/delegate.md`.

use super::State;
use crate::nodes::delegate::{self, Delegate};
use crate::nodes::report::{self, Report};
use crate::scaffold::target::Target;
use crate::{Node, Transition, Walk};

/// The native CLI's non-zero exit code, or why there was none to run.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    Delegate,
    Report,
}

#[derive(Default)]
struct Hand<A = delegate::SystemUtils, B = report::SystemUtils> {
    delegate: Delegate<A>,
    report: Report<B>,
}

impl Hand {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B> Walk for Hand<A, B>
where
    A: delegate::Utils + Sync,
    B: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::Delegate => {
                let Ok(outcome) = self.delegate.run(state).await;
                match outcome {
                    delegate::Outcome::Ran => Transition::Halt,
                    delegate::Outcome::Unavailable => Transition::Next(Id::Report),
                }
            }
            Id::Report => {
                let Ok(()) = self.report.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs `amble <verb> …` through the target's native CLI.
///
/// # Errors
///
/// Returns [`Error::Exit`] with the CLI's exit code if it did not succeed, or if there was no
/// CLI to run (the reason is already on standard error, and the exit code is 2).
pub async fn run(target: Target, verb: &str, args: Vec<String>) -> Result<(), Error> {
    let mut state = State::new(target, args);
    state.verb = verb.to_string();
    Hand::new().run(Id::Delegate, &mut state).await?;
    match state.exit_code {
        0 => Ok(()),
        code => Err(Error::Exit(code)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    async fn hand(fake: &Fake, target: Target, verb: &str, args: &[&str]) -> State {
        let walk = Hand {
            delegate: Delegate::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::delegating(target, verb, args);
        walk.run(Id::Delegate, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn the_cli_reports_for_itself() {
        let fake = Fake::with_programs(&["cargo", "cargo-ambler"], &[]).exiting(1);

        let state = hand(&fake, Target::Rust, "check", &["--lint-only"]).await;

        assert_eq!(state.exit_code, 1);
        assert_eq!(fake.ran(), ["cargo ambler check --lint-only"]);
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn without_a_cli_the_reason_is_reported_and_nothing_is_written() {
        let fake = Fake::default();

        let state = hand(&fake, Target::Rust, "node", &["ask"]).await;

        assert_eq!(state.exit_code, 2);
        assert!(fake.eprinted()[0].contains("cargo install --git"));
        assert!(fake.paths().is_empty());
    }
}
