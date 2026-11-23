//! `amble util`: adds a utility module to the project. See `specs/util.md`.

use super::State;
use crate::nodes::plan_util::{self, PlanUtil};
use crate::nodes::report::{self, Report};
use crate::nodes::write::{self, Write};
use crate::scaffold::target::Target;
use crate::{Node, Transition, Walk};

/// A rejected invocation, or a file that could not be written.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    PlanUtil,
    Write,
    Report,
}

#[derive(Default)]
struct Util<A = plan_util::SystemUtils, B = write::SystemUtils, C = report::SystemUtils> {
    plan_util: PlanUtil<A>,
    write: Write<B>,
    report: Report<C>,
}

impl Util {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B, C> Walk for Util<A, B, C>
where
    A: plan_util::Utils + Sync,
    B: write::Utils + Sync,
    C: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::PlanUtil => {
                let Ok(outcome) = self.plan_util.run(state).await;
                match outcome {
                    plan_util::Outcome::Planned => Transition::Next(Id::Write),
                    plan_util::Outcome::Rejected => Transition::Next(Id::Report),
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

/// Runs `amble util <name>`.
///
/// # Errors
///
/// Returns [`Error`] if the invocation is rejected (the reason is already on standard error) or
/// a file cannot be written.
pub async fn run(target: Target, args: Vec<String>) -> Result<(), Error> {
    if let Some(ok) = crate::utils::runner::delegate(target, "util", &args)? {
        return if ok { Ok(()) } else { Err(Error::Failed) };
    }
    let mut state = State::new(target, args);
    Util::new().run(Id::PlanUtil, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    async fn util(fake: &Fake, args: &[&str]) -> State {
        let walk = Util {
            plan_util: PlanUtil::with_utils(fake),
            write: Write::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::with_args(args);
        walk.run(Id::PlanUtil, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn writes_the_module_and_registers_it() {
        let fake = Fake::with_files(&[("src/utils/mod.rs", "pub mod terminal;\n")]);

        let state = util(&fake, &["journal"]).await;

        assert_eq!(state.message, None);
        assert!(
            fake.file("src/utils/journal.rs")
                .unwrap()
                .starts_with("//! ")
        );
        assert_eq!(
            fake.file("src/utils/mod.rs").as_deref(),
            Some("pub mod journal;\npub mod terminal;\n")
        );
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn reports_a_rejected_invocation_and_writes_nothing() {
        let fake = Fake::with_files(&[("src/utils/mod.rs", "pub mod terminal;\n")]);

        let state = util(&fake, &["terminal"]).await;

        assert!(state.finish().is_err());
        assert!(fake.eprinted()[0].contains("already registered"));
        assert_eq!(fake.paths(), ["src/utils/mod.rs"]);
    }
}
