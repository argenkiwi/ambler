//! `amble pave`: implements a spec by running a coding agent in a loop, one narrow task per
//! run, with a git checkpoint after every passing check. See `specs/pave.md`.

use super::State;
use crate::nodes::brief_agent::{self, BriefAgent};
use crate::nodes::check_project::{self, CheckProject};
use crate::nodes::checkpoint::{self, Checkpoint};
use crate::nodes::choose_task::{self, ChooseTask};
use crate::nodes::plan_pave::{self, PlanPave};
use crate::nodes::report::{self, Report};
use crate::nodes::roll_back::{self, RollBack};
use crate::scaffold::target::Target;
use crate::{Node, Transition, Walk};

/// A rejected invocation or a loop that gave up, or a check, git or agent that could not be run.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    PlanPave,
    CheckProject,
    Checkpoint,
    ChooseTask,
    RollBack,
    BriefAgent,
    Report,
}

#[derive(Default)]
struct Pave<
    A = plan_pave::SystemUtils,
    B = check_project::SystemUtils,
    C = checkpoint::SystemUtils,
    D = choose_task::SystemUtils,
    E = roll_back::SystemUtils,
    F = brief_agent::SystemUtils,
    G = report::SystemUtils,
> {
    plan_pave: PlanPave<A>,
    check_project: CheckProject<B>,
    checkpoint: Checkpoint<C>,
    choose_task: ChooseTask<D>,
    roll_back: RollBack<E>,
    brief_agent: BriefAgent<F>,
    report: Report<G>,
}

impl Pave {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B, C, D, E, F, G> Walk for Pave<A, B, C, D, E, F, G>
where
    A: plan_pave::Utils + Sync,
    B: check_project::Utils + Sync,
    C: checkpoint::Utils + Sync,
    D: choose_task::Utils + Sync,
    E: roll_back::Utils + Sync,
    F: brief_agent::Utils + Sync,
    G: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::PlanPave => {
                let Ok(outcome) = self.plan_pave.run(state).await;
                match outcome {
                    plan_pave::Outcome::Ready => Transition::Next(Id::CheckProject),
                    plan_pave::Outcome::Rejected => Transition::Next(Id::Report),
                }
            }
            Id::CheckProject => match self.check_project.run(state).await? {
                check_project::Outcome::Passed => Transition::Next(Id::Checkpoint),
                check_project::Outcome::Failed => Transition::Next(Id::ChooseTask),
            },
            Id::Checkpoint => match self.checkpoint.run(state).await? {
                checkpoint::Outcome::Saved => Transition::Next(Id::ChooseTask),
            },
            Id::ChooseTask => match self.choose_task.run(state).await? {
                choose_task::Outcome::Assigned => Transition::Next(Id::BriefAgent),
                choose_task::Outcome::Done => Transition::Halt,
                choose_task::Outcome::Exhausted => Transition::Next(Id::RollBack),
                choose_task::Outcome::Stuck => Transition::Next(Id::Report),
            },
            Id::RollBack => match self.roll_back.run(state).await? {
                roll_back::Outcome::RolledBack => Transition::Next(Id::CheckProject),
            },
            Id::BriefAgent => match self.brief_agent.run(state).await? {
                brief_agent::Outcome::Ran => Transition::Next(Id::CheckProject),
                brief_agent::Outcome::Previewed => Transition::Halt,
            },
            Id::Report => {
                let Ok(()) = self.report.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs `amble pave <spec> …`.
///
/// # Errors
///
/// Returns [`Error`] if the invocation is rejected or the loop gives up (the reason is already
/// on standard error), or if the check, git or the agent cannot be run.
pub async fn run(target: Target, args: Vec<String>) -> Result<(), Error> {
    let mut state = State::new(target, args);
    Pave::new().run(Id::PlanPave, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::pave::UNFINISHED;
    use crate::testing::Fake;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    const SPEC: &str = "# Program Specifications\n\n## Nodes\n\n### Ask\n- Asks.\n\n\
                        ### Greet\n- Greets.\n\n## Shared State\n\n- `name`: who.\n";

    /// A project with a spec and no nodes yet, in a repository.
    fn project(checks: &[(bool, &'static str)]) -> Fake {
        Fake {
            git_dir: Some("/repo/.git"),
            checks: Mutex::new(checks.iter().copied().collect()),
            ..Fake::with_files(&[
                ("Cargo.toml", ""),
                ("src/nodes/mod.rs", ""),
                ("specs/hello.md", SPEC),
            ])
        }
    }

    /// What each agent run writes, in order.
    fn agent_writes(fake: &Fake, runs: Vec<Vec<(&'static str, String)>>) {
        *fake.agent_writes.lock().unwrap() = VecDeque::from(runs);
    }

    async fn pave(fake: &Fake, args: &[&str]) -> State {
        let walk = Pave {
            plan_pave: PlanPave::with_utils(fake),
            check_project: CheckProject::with_utils(fake),
            checkpoint: Checkpoint::with_utils(fake),
            choose_task: ChooseTask::with_utils(fake),
            roll_back: RollBack::with_utils(fake),
            brief_agent: BriefAgent::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::with_args(args);
        walk.run(Id::PlanPave, &mut state).await.unwrap();
        state
    }

    fn labels(fake: &Fake) -> Vec<String> {
        fake.printed()
            .into_iter()
            .filter_map(|line| Some(line.strip_prefix("[amble pave] agent: ")?.to_string()))
            .collect()
    }

    #[tokio::test]
    async fn paves_each_node_then_the_walk_with_a_checkpoint_after_each() {
        // The first run of the greet node leaves it unfinished and the check failing; the next
        // run of the same node finishes it.
        let fake = project(&[
            (true, ""),
            (true, ""),
            (false, "todo"),
            (true, ""),
            (true, ""),
        ]);
        agent_writes(
            &fake,
            vec![
                vec![("src/nodes/ask.rs", "done".to_string())],
                vec![("src/nodes/greet.rs", format!("{UNFINISHED})"))],
                vec![("src/nodes/greet.rs", "done".to_string())],
                vec![("src/walks/hello.rs", "done".to_string())],
            ],
        );

        let state = pave(&fake, &["hello", "--agent", "claude -p"]).await;

        assert_eq!(state.message, None);
        assert_eq!(
            labels(&fake),
            ["node ask", "node greet", "node greet", "walk hello"]
        );
        assert_eq!(
            fake.git(),
            [
                "commit amble-pave(hello): node ask",
                "commit amble-pave(hello): node greet",
                "commit amble-pave(hello): walk hello",
            ]
        );
        assert!(
            fake.prompts()[2]
                .1
                .contains("## Last `amble check` output\n```\ntodo")
        );
        assert!(fake.printed().last().unwrap().contains("done"));
    }

    #[tokio::test]
    async fn rolls_back_once_the_repairs_run_out_then_reports_it() {
        // Passes, then fails after the node, after its repair, after the roll back and after the
        // one retry.
        let fake = project(&[
            (true, ""),
            (false, "broken"),
            (false, "still broken"),
            (false, "broken before the stash"),
            (false, "broken after the retry"),
        ]);
        agent_writes(
            &fake,
            vec![vec![("src/nodes/ask.rs", "broken".to_string())]],
        );

        let state = pave(&fake, &["hello", "--agent", "pi -p", "--max-repairs", "1"]).await;

        assert_eq!(labels(&fake), ["node ask", "repair", "repair"]);
        assert_eq!(
            fake.git(),
            ["stash amble-pave(hello): failed repair after 1 repairs"]
        );
        assert!(state.message.is_some());
        assert!(fake.eprinted().last().unwrap().contains("still failing"));
    }

    #[tokio::test]
    async fn stops_when_the_agent_makes_no_progress() {
        let fake = project(&[]);

        let state = pave(&fake, &["hello", "--agent", "true"]).await;

        assert_eq!(labels(&fake), ["node ask", "node ask", "node ask"]);
        assert!(state.message.unwrap().contains("no progress on `node ask`"));
    }

    #[tokio::test]
    async fn a_dry_run_prints_the_first_prompt_and_changes_nothing() {
        let fake = Fake {
            dirty: Mutex::new(true),
            ..project(&[])
        };

        let state = pave(&fake, &["hello", "--dry-run"]).await;

        assert_eq!(state.message, None);
        assert!(fake.git().is_empty());
        assert!(fake.prompts().is_empty());
        let printed = fake.printed().join("\n");
        assert!(printed.contains("would commit a checkpoint: startup"));
        assert!(printed.contains("----- prompt (node ask) -----"));
    }

    #[tokio::test]
    async fn reports_a_rejected_invocation_without_running_anything() {
        let fake = project(&[]);

        let state = pave(&fake, &["hello"]).await;

        assert!(state.finish().is_err());
        assert!(fake.eprinted()[0].contains("AMBLE_AGENT"));
        assert!(fake.printed().is_empty());
    }
}
