//! `amble new`: creates a project. See `specs/new.md`.

use super::State;
use crate::nodes::plan_project::{self, PlanProject};
use crate::nodes::report::{self, Report};
use crate::nodes::write::{self, Write};
use crate::scaffold::target::Target;
use crate::{Node, Transition, Walk};

/// A rejected invocation, or a file that could not be written.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    PlanProject,
    Write,
    Report,
}

#[derive(Default)]
struct NewProject<A = plan_project::SystemUtils, B = write::SystemUtils, C = report::SystemUtils> {
    plan_project: PlanProject<A>,
    write: Write<B>,
    report: Report<C>,
}

impl NewProject {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B, C> Walk for NewProject<A, B, C>
where
    A: plan_project::Utils + Sync,
    B: write::Utils + Sync,
    C: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::PlanProject => {
                let Ok(outcome) = self.plan_project.run(state).await;
                match outcome {
                    plan_project::Outcome::Planned => Transition::Next(Id::Write),
                    plan_project::Outcome::Rejected => Transition::Next(Id::Report),
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

/// Runs `amble new [<name>] [--example]`.
///
/// # Errors
///
/// Returns [`Error`] if the invocation is rejected (the reason is already on standard error) or
/// a file cannot be written.
pub async fn run(target: Target, args: Vec<String>) -> Result<(), Error> {
    let mut state = State::new(target, args);
    NewProject::new().run(Id::PlanProject, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    async fn new(fake: &Fake, args: &[&str]) -> State {
        let walk = NewProject {
            plan_project: PlanProject::with_utils(fake),
            write: Write::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::with_args(args);
        walk.run(Id::PlanProject, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn writes_the_project_and_says_what_to_do_next() {
        let fake = Fake::default();

        let state = new(&fake, &["my-app", "--example"]).await;

        assert_eq!(state.message, None);
        assert!(
            fake.file("my-app/src/engine.rs")
                .unwrap()
                .contains("pub trait Node")
        );
        assert!(fake.file("my-app/src/walks/counter.rs").is_some());
        assert_eq!(
            fake.file("my-app/.claude/skills").as_deref(),
            Some("-> ../.agents/skills")
        );
        assert!(
            fake.printed()
                .last()
                .unwrap()
                .starts_with("Created my-app/.")
        );
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn without_a_name_writes_the_project_in_the_empty_current_folder() {
        let fake = Fake {
            folder: Some("my-app"),
            ..Fake::default()
        };

        let state = new(&fake, &[]).await;

        assert_eq!(state.message, None);
        assert!(
            fake.file("src/engine.rs")
                .unwrap()
                .contains("pub trait Node")
        );
        assert_eq!(
            fake.file(".claude/skills").as_deref(),
            Some("-> ../.agents/skills")
        );
        assert!(
            fake.printed()
                .last()
                .unwrap()
                .starts_with("Created my-app here.")
        );
    }

    #[tokio::test]
    async fn reports_a_rejected_invocation_and_writes_nothing() {
        let fake = Fake::default();

        let state = new(&fake, &["a", "b"]).await;

        assert!(state.finish().is_err());
        assert_eq!(
            fake.eprinted(),
            ["usage: amble new [<name>] [--example] [--target rust|ts]"]
        );
        assert!(fake.paths().is_empty());
        assert!(fake.printed().is_empty());
    }

    /// End to end against the real file system and cargo: the example project `amble new`
    /// writes builds, passes its tests and passes `amble check`.
    #[tokio::test]
    #[ignore = "slow: builds a generated project with cargo"]
    async fn the_generated_example_project_passes_its_own_checks() {
        use std::process::Command;

        let folder = std::env::temp_dir().join(format!("amble-e2e-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let amble = |args: &[&str], within: &std::path::Path| {
            let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
            let status = Command::new("cargo")
                .args(["run", "--quiet", "--manifest-path", manifest, "--"])
                .args(args)
                .current_dir(within)
                .status()
                .unwrap();
            assert!(status.success(), "amble {args:?} failed");
        };

        amble(&["new", "demo", "--example"], &folder);
        amble(&["check"], &folder.join("demo"));

        std::fs::remove_dir_all(&folder).unwrap();
    }
}
