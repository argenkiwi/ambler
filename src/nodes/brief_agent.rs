use crate::Node;
use crate::nodes::HasPaving;
use crate::scaffold::pave::{Brief, Paving, Task, node_prompt, repair_prompt, walk_prompt};
use crate::utils::{fs, git, process};
use std::io;

/// Where [`BriefAgent`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The agent ran with the task's prompt.
    Ran,
    /// A dry run: the prompt was printed instead.
    Previewed,
}

/// Why [`BriefAgent`] could not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not brief the agent: {0}")]
    Io(#[from] io::Error),
    #[error("no agent command to run")]
    NoAgent,
}

/// The side effects [`BriefAgent`] needs: one method each, mocked in the tests.
pub trait Utils {
    fn read(&self, path: &str) -> io::Result<String>;
    /// The uncommitted changes, one line each.
    fn status(&self) -> io::Result<String>;
    /// Runs the agent `command` with `prompt` as its last argument, and reports whether it
    /// exited successfully.
    fn agent(&self, command: &str, prompt: &str) -> io::Result<bool>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn read(&self, path: &str) -> io::Result<String> {
        fs::read(path)
    }

    fn status(&self) -> io::Result<String> {
        git::status()
    }

    fn agent(&self, command: &str, prompt: &str) -> io::Result<bool> {
        process::shell_with(command, prompt)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Gives the agent the current task, with only the context that task needs.
#[derive(Clone, Debug, Default)]
pub struct BriefAgent<U = SystemUtils> {
    utils: U,
}

impl BriefAgent {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> BriefAgent<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<U: Utils> BriefAgent<U> {
    /// The prompt for the current task.
    fn prompt(&self, paving: &Paving) -> io::Result<String> {
        let settings = &paving.settings;
        let spec = self.utils.read(&settings.spec_path())?;
        let target = settings.target;
        let brief = Brief {
            settings,
            spec: &spec,
            passed: paving.passed,
            output: &paving.output,
        };
        let exists = |path: &str| self.utils.read(path).is_ok();
        Ok(match &paving.task {
            Some(Task::Node(name)) => node_prompt(&brief, name, exists(&target.node_path(name))),
            Some(Task::Walk) => walk_prompt(&brief, exists(&settings.walk_path())),
            Some(Task::Repair | Task::Done) | None => repair_prompt(&brief, &self.utils.status()?),
        })
    }
}

impl<S, U> Node<S> for BriefAgent<U>
where
    S: HasPaving + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        let paving = state.paving();
        let prompt = self.prompt(paving)?;
        let label = &paving.label;
        if paving.settings.dry_run {
            self.utils.print(&format!(
                "\n----- prompt ({label}) -----\n{prompt}\n----- end prompt -----"
            ));
            return Ok(Outcome::Previewed);
        }
        let command = paving.settings.agent.as_deref().ok_or(Error::NoAgent)?;
        self.utils.print(&format!("[amble pave] agent: {label}"));
        if !self.utils.agent(command, &prompt)? {
            self.utils.print(
                "[amble pave] the agent exited with an error (continuing; the check decides)",
            );
        }
        let paving = state.paving_mut();
        if !paving.passed {
            paving.repairs += 1;
        }
        Ok(Outcome::Ran)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::pave::Settings;
    use crate::testing::{Fake, State};
    use std::sync::Mutex;

    const SPEC: &str = "# Program Specifications\n\n## Nodes\n\n### Ask\n- Asks.\n\n\
                        ### Greet\n- Greets.\n\n## Shared State\n\n- `name`: who.\n";
    fn project() -> Fake {
        Fake::with_files(&[
            ("specs/hello.md", SPEC),
            ("src/nodes/ask.rs", "pub struct Ask;\n"),
        ])
    }

    fn state(task: Task, passed: bool, dry_run: bool) -> State {
        State {
            paving: Paving {
                settings: Settings {
                    spec: "hello".to_string(),
                    agent: Some("claude -p".to_string()),
                    dry_run,
                    ..Settings::default()
                },
                passed,
                output: "`cargo test` failed".to_string(),
                label: task.label("hello"),
                task: Some(task),
                ..Paving::default()
            },
            ..State::default()
        }
    }

    async fn run(fake: &Fake, state: &mut State) -> Outcome {
        BriefAgent::with_utils(fake).run(state).await.unwrap()
    }

    #[tokio::test]
    async fn runs_the_agent_with_the_nodes_section_only() {
        let fake = project();
        let mut state = state(Task::Node("greet".to_string()), true, false);

        let outcome = run(&fake, &mut state).await;

        assert_eq!(outcome, Outcome::Ran);
        let prompts = fake.prompts();
        let (command, prompt) = &prompts[0];
        assert_eq!(command, "claude -p");
        assert!(prompt.contains("### Greet\n- Greets."));
        assert!(!prompt.contains("### Ask"));
        assert!(prompt.contains("does not exist yet"));
        assert_eq!(fake.printed(), ["[amble pave] agent: node greet"]);
        assert_eq!(state.paving.repairs, 0);
    }

    #[tokio::test]
    async fn briefs_the_walk_without_outcomes_for_the_agent_to_read() {
        let fake = project();
        let mut state = state(Task::Walk, true, false);

        run(&fake, &mut state).await;

        let prompt = &fake.prompts()[0].1;
        assert!(prompt.contains("`amble walk hello --nodes ask,greet`"));
        assert!(!prompt.contains("--edge"));
    }

    #[tokio::test]
    async fn counts_a_repair_and_briefs_it_with_the_changes() {
        let fake = Fake {
            dirty: Mutex::new(true),
            ..project()
        };
        let mut state = state(Task::Repair, false, false);

        run(&fake, &mut state).await;

        let prompt = &fake.prompts()[0].1;
        assert!(prompt.contains("`cargo test` failed"));
        assert!(prompt.contains(" M src/lib.rs"));
        assert_eq!(state.paving.repairs, 1);
    }

    #[tokio::test]
    async fn notes_an_agent_that_exits_with_an_error() {
        let fake = Fake {
            failing: Some("agent-exit"),
            ..project()
        };
        let mut state = state(Task::Repair, false, false);

        let outcome = run(&fake, &mut state).await;

        assert_eq!(outcome, Outcome::Ran);
        assert!(fake.printed()[1].contains("the check decides"));
        assert_eq!(state.paving.repairs, 1);
    }

    #[tokio::test]
    async fn prints_the_prompt_in_a_dry_run() {
        let fake = project();
        let mut state = state(Task::Node("ask".to_string()), true, true);

        let outcome = run(&fake, &mut state).await;

        assert_eq!(outcome, Outcome::Previewed);
        assert!(fake.prompts().is_empty());
        let printed = fake.printed().join("\n");
        assert!(printed.contains("----- prompt (node ask) -----"));
        assert!(printed.contains("already exists"));
    }

    #[tokio::test]
    async fn fails_if_the_agent_cannot_be_started() {
        let fake = Fake {
            failing: Some("agent"),
            ..project()
        };
        let mut state = state(Task::Walk, false, false);

        let result = BriefAgent::with_utils(&fake).run(&mut state).await;

        assert!(matches!(result, Err(Error::Io(_))));
        assert_eq!(state.paving.repairs, 0);
    }

    #[tokio::test]
    async fn fails_without_an_agent_command() {
        let fake = project();
        let mut state = state(Task::Walk, true, false);
        state.paving.settings.agent = None;

        let result = BriefAgent::with_utils(&fake).run(&mut state).await;

        assert!(matches!(result, Err(Error::NoAgent)));
    }

    #[tokio::test]
    async fn fails_if_the_spec_cannot_be_read() {
        let mut state = state(Task::Walk, true, false);

        let result = BriefAgent::with_utils(&Fake::default())
            .run(&mut state)
            .await;

        assert!(matches!(result, Err(Error::Io(_))));
    }
}
