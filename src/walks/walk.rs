//! `amble walk`: wires nodes into a new walk. See `specs/walk.md`.

use super::State;
use crate::nodes::plan_walk::{self, PlanWalk};
use crate::nodes::report::{self, Report};
use crate::nodes::write::{self, Write};
use crate::{Node, Transition, Walk as _};

/// A rejected invocation, or a file that could not be written.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    PlanWalk,
    Write,
    Report,
}

#[derive(Default)]
struct Walk<A = plan_walk::SystemUtils, B = write::SystemUtils, C = report::SystemUtils> {
    plan_walk: PlanWalk<A>,
    write: Write<B>,
    report: Report<C>,
}

impl Walk {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B, C> crate::Walk for Walk<A, B, C>
where
    A: plan_walk::Utils + Sync,
    B: write::Utils + Sync,
    C: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::PlanWalk => {
                let Ok(outcome) = self.plan_walk.run(state).await;
                match outcome {
                    plan_walk::Outcome::Planned => Transition::Next(Id::Write),
                    plan_walk::Outcome::Rejected => Transition::Next(Id::Report),
                }
            }
            Id::Write => {
                self.write.run(state).await?;
                Transition::Halt
            }
            Id::Report => {
                let Ok(()) = self.report.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs `amble walk <name> --nodes … [--edge …]`.
///
/// # Errors
///
/// Returns [`Error`] if the invocation is rejected (the reason is already on standard error) or
/// a file cannot be written.
pub async fn run(args: Vec<String>) -> Result<(), Error> {
    let mut state = State::new(args);
    Walk::new().run(Id::PlanWalk, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    const PROJECT: &[(&str, &str)] = &[
        ("Cargo.toml", "[package]\nname = \"app\"\n"),
        ("src/walks/mod.rs", "//! Walks.\n"),
        ("src/nodes/mod.rs", "pub mod stop;\n"),
        (
            "src/nodes/stop.rs",
            include_str!("../../templates/example/stop.rs"),
        ),
    ];

    async fn walk(fake: &Fake, args: &[&str]) -> State {
        let walk = Walk {
            plan_walk: PlanWalk::with_utils(fake),
            write: Write::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::with_args(args);
        walk.run(Id::PlanWalk, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn writes_the_walk_its_spec_and_its_runner() {
        let fake = Fake::with_files(PROJECT);

        let state = walk(&fake, &["finish", "--nodes", "stop"]).await;

        assert_eq!(state.message, None);
        assert!(
            fake.file("src/walks/finish.rs")
                .unwrap()
                .contains("struct Finish<A = stop::SystemUtils>")
        );
        assert_eq!(
            fake.file("src/walks/mod.rs").as_deref(),
            Some("//! Walks.\n\npub mod finish;\n")
        );
        assert!(fake.file("specs/finish.md").unwrap().contains("### Stop"));
        assert!(
            fake.file("src/main.rs")
                .unwrap()
                .contains("app::walks::finish::run()")
        );
        assert_eq!(*fake.formatted.lock().unwrap(), ["src/walks/finish.rs"]);
    }

    #[tokio::test]
    async fn reports_a_rejected_invocation_and_writes_nothing() {
        let fake = Fake::with_files(PROJECT);

        let state = walk(&fake, &["finish", "--nodes", "start,stop"]).await;

        assert!(state.finish().is_err());
        assert!(fake.eprinted()[0].contains("amble node start"));
        assert_eq!(fake.paths().len(), PROJECT.len());
    }
}
