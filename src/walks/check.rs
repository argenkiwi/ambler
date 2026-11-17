//! `amble check`: verifies the conventions, then formatting, lints and tests. See
//! `specs/check.md`.

use super::State;
use crate::nodes::inspect::{self, Inspect};
use crate::nodes::report::{self, Report};
use crate::nodes::verify::{self, Verify};
use crate::{Node, Transition, Walk};

const USAGE: &str = "usage: amble check [--no-cargo]";

/// A failed check, or a project or cargo that could not be reached.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    Inspect,
    Verify,
    Report,
}

#[derive(Default)]
struct Check<A = inspect::SystemUtils, B = verify::SystemUtils, C = report::SystemUtils> {
    inspect: Inspect<A>,
    verify: Verify<B>,
    report: Report<C>,
    /// Whether a clean inspection goes on to cargo. `--no-cargo` ends the walk there instead.
    cargo: bool,
}

impl Check {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new(cargo: bool) -> Self {
        Self {
            cargo,
            ..Self::default()
        }
    }
}

impl<A, B, C> Walk for Check<A, B, C>
where
    A: inspect::Utils + Sync,
    B: verify::Utils + Sync,
    C: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::Inspect => match self.inspect.run(state).await? {
                inspect::Outcome::Clean if self.cargo => Transition::Next(Id::Verify),
                inspect::Outcome::Clean => Transition::Halt,
                inspect::Outcome::Violations => Transition::Next(Id::Report),
            },
            Id::Verify => match self.verify.run(state).await? {
                verify::Outcome::Passed => Transition::Halt,
                verify::Outcome::Failed => Transition::Next(Id::Report),
            },
            Id::Report => {
                let Ok(()) = self.report.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs `amble check [--no-cargo]`.
///
/// # Errors
///
/// Returns [`Error`] if a convention is broken or cargo fails (the reason is already printed),
/// or if the project or cargo cannot be reached.
pub async fn run(args: Vec<String>) -> Result<(), Error> {
    let cargo = !args.iter().any(|arg| arg == "--no-cargo");
    let known = args.iter().all(|arg| arg == "--no-cargo");
    let mut state = State::new(args);
    let start = if known {
        Id::Inspect
    } else {
        state.message = Some(USAGE.to_string());
        Id::Report
    };
    Check::new(cargo).run(start, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    const PROJECT: &[(&str, &str)] = &[
        ("Cargo.toml", "[package]\nname = \"app\"\n"),
        ("src/nodes/mod.rs", ""),
        ("src/walks/mod.rs", ""),
        ("src/utils/mod.rs", ""),
    ];

    async fn check(fake: &Fake, cargo: bool) -> State {
        let walk = Check {
            inspect: Inspect::with_utils(fake),
            verify: Verify::with_utils(fake),
            report: Report::with_utils(fake),
            cargo,
        };
        let mut state = State::default();
        walk.run(Id::Inspect, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn a_clean_project_goes_on_to_cargo_and_passes() {
        let fake = Fake::with_files(PROJECT);

        let state = check(&fake, true).await;

        assert_eq!(state.message, None);
        assert_eq!(fake.printed().last().unwrap(), "$ cargo test");
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn cargo_is_skipped_when_asked() {
        let fake = Fake::with_files(PROJECT);

        let state = check(&fake, false).await;

        assert_eq!(state.message, None);
        assert!(fake.printed().is_empty());
    }

    #[tokio::test]
    async fn violations_are_reported_without_running_cargo() {
        let mut files = PROJECT.to_vec();
        files.push(("src/nodes/ask.rs", "pub struct Ask;\n"));
        let fake = Fake::with_files(&files);

        let state = check(&fake, true).await;

        assert!(state.finish().is_err());
        assert!(
            fake.printed()
                .iter()
                .all(|line| line.starts_with("src/nodes/ask.rs:1: "))
        );
        assert!(fake.eprinted()[0].contains("convention violation"));
    }

    #[tokio::test]
    async fn a_failing_cargo_command_is_reported() {
        let fake = Fake {
            failing: Some("test"),
            ..Fake::with_files(PROJECT)
        };

        let state = check(&fake, true).await;

        assert!(state.finish().is_err());
        assert_eq!(fake.eprinted(), ["`cargo test` failed"]);
    }
}
