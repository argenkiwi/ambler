use crate::Node;
use crate::nodes::{HasMessage, HasPaving};
use crate::scaffold::pave::{Decision, select_task, spec_nodes};
use crate::utils::fs;
use std::io;

/// Where [`ChooseTask`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The next task is recorded for the agent.
    Assigned,
    /// The walk exists and the check passes: the spec is implemented.
    Done,
    /// The check kept failing for as many repairs as allowed.
    Exhausted,
    /// The loop cannot go on; the message says why.
    Stuck,
}

/// Why [`ChooseTask`] could not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read the spec: {0}")]
    Io(#[from] io::Error),
}

/// The side effects [`ChooseTask`] needs: one method each, mocked in the tests.
pub trait Utils {
    fn read(&self, path: &str) -> io::Result<String>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn read(&self, path: &str) -> io::Result<String> {
        fs::read(path)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Works out the next task from the spec and the project, within the loop's limits.
#[derive(Clone, Debug, Default)]
pub struct ChooseTask<U = SystemUtils> {
    utils: U,
}

impl ChooseTask {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> ChooseTask<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for ChooseTask<U>
where
    S: HasPaving + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        let paving = state.paving();
        let settings = &paving.settings;
        let spec = self.utils.read(&settings.spec_path())?;
        let task = select_task(
            settings.target,
            &spec_nodes(&spec),
            &settings.walk_path(),
            paving.passed,
            |path: &str| self.utils.read(path).ok(),
        );
        let walk = settings.walk_path();
        let paving = state.paving_mut();
        Ok(match paving.decide(task) {
            Decision::Assigned => {
                self.utils.print(&format!(
                    "[amble pave] iteration {}: {}",
                    paving.iterations, paving.label
                ));
                Outcome::Assigned
            }
            Decision::Done => {
                self.utils.print(&format!(
                    "[amble pave] done: {walk} exists and amble check passes."
                ));
                Outcome::Done
            }
            Decision::Exhausted => Outcome::Exhausted,
            Decision::Stuck(reason) => {
                *state.message_mut() = Some(format!("amble pave: {reason}"));
                Outcome::Stuck
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::pave::{Paving, Settings, Task};
    use crate::testing::{Fake, State};

    const SPEC: &str = "# Program Specifications\n\n## Nodes\n\n### Ask\n\n### Greet\n\n\
                        ## Shared State\n";

    fn state(passed: bool) -> State {
        State {
            paving: Paving {
                settings: Settings {
                    spec: "hello".to_string(),
                    max_repairs: 1,
                    ..Settings::default()
                },
                passed,
                ..Paving::default()
            },
            ..State::default()
        }
    }

    fn project(files: &[(&'static str, &'static str)]) -> Fake {
        let mut all = vec![("specs/hello.md", SPEC)];
        all.extend_from_slice(files);
        Fake::with_files(&all)
    }

    #[tokio::test]
    async fn assigns_the_next_node_and_announces_the_iteration() {
        let fake = project(&[("src/nodes/ask.rs", "done")]);
        let mut state = state(true);

        let outcome = ChooseTask::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Assigned);
        assert_eq!(state.paving.task, Some(Task::Node("greet".to_string())));
        assert_eq!(state.paving.iterations, 1);
        assert_eq!(fake.printed(), ["[amble pave] iteration 1: node greet"]);
    }

    #[tokio::test]
    async fn is_done_once_the_walk_exists_and_the_check_passes() {
        let fake = project(&[
            ("src/nodes/ask.rs", "done"),
            ("src/nodes/greet.rs", "done"),
            ("src/walks/hello.rs", "done"),
        ]);
        let mut state = state(true);

        let outcome = ChooseTask::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Done);
        assert_eq!(state.paving.task, None);
        assert!(fake.printed()[0].contains("done"));
    }

    #[tokio::test]
    async fn is_exhausted_when_repairs_run_out() {
        let fake = project(&[]);
        let mut state = state(false);
        state.paving.repairs = 1;

        let outcome = ChooseTask::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Exhausted);
        assert_eq!(state.message, None);
    }

    #[tokio::test]
    async fn is_stuck_when_a_guard_trips() {
        let fake = project(&[]);
        let mut state = state(false);
        state.paving.repairs = 1;
        state.paving.rolled_back = true;

        let outcome = ChooseTask::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Stuck);
        assert!(state.message.unwrap().contains("still failing"));
    }

    #[tokio::test]
    async fn fails_if_the_spec_cannot_be_read() {
        let fake = Fake::default();

        let result = ChooseTask::with_utils(&fake).run(&mut state(true)).await;

        assert!(matches!(result, Err(Error::Io(_))));
    }
}
