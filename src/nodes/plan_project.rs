use crate::Node;
use crate::nodes::{HasArgs, HasMessage, HasPlan};
use crate::scaffold::project::{is_package_name, project};
use crate::scaffold::{Plan, args};
use crate::utils::fs;
use std::convert::Infallible;

const USAGE: &str = "usage: amble new <name> [--example]";

/// Where [`PlanProject`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The plan holds every file of the new project.
    Planned,
    /// The arguments were wrong or the folder is taken; the message says which.
    Rejected,
}

pub trait Utils {
    fn exists(&self, path: &str) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn exists(&self, path: &str) -> bool {
        fs::exists(path)
    }
}

/// Plans the files of a new project from `<name> [--example]`.
#[derive(Clone, Debug, Default)]
pub struct PlanProject<U = SystemUtils> {
    utils: U,
}

impl PlanProject {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> PlanProject<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<U: Utils> PlanProject<U> {
    fn plan(&self, args: &[String]) -> Result<Plan, String> {
        let parsed = args::parse(args, &[], &["example"]).map_err(|e| format!("{e}\n{USAGE}"))?;
        let [name] = parsed.positionals.as_slice() else {
            return Err(USAGE.to_string());
        };
        if !is_package_name(name) {
            return Err(format!(
                "`{name}` is not a package name: use lowercase letters, digits, `-` and `_`"
            ));
        }
        if self.utils.exists(name) {
            return Err(format!("`{name}` already exists"));
        }
        Ok(Plan {
            changes: project(name, parsed.has("example")),
            hint: format!(
                "Created {name}/. Next: `cd {name}`, then `amble help` for how to add nodes \
                 and walks."
            ),
        })
    }
}

impl<S, U> Node<S> for PlanProject<U>
where
    S: HasArgs + HasPlan + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Infallible> {
        Ok(match self.plan(state.args()) {
            Ok(plan) => {
                *state.plan_mut() = plan;
                Outcome::Planned
            }
            Err(message) => {
                *state.message_mut() = Some(message);
                Outcome::Rejected
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Fake, State};

    async fn run(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_args(args);
        let Ok(outcome) = PlanProject::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    #[tokio::test]
    async fn plans_an_empty_project_in_a_folder_named_after_it() {
        let (outcome, state) = run(&Fake::default(), &["my-app"]).await;

        assert_eq!(outcome, Outcome::Planned);
        let paths: Vec<&str> = state.plan.changes.iter().map(|c| c.path()).collect();
        assert!(paths.contains(&"my-app/src/engine.rs"));
        assert!(!paths.contains(&"my-app/src/walks/counter.rs"));
        assert!(state.plan.hint.contains("cd my-app"));
    }

    #[tokio::test]
    async fn plans_the_counter_walk_when_asked_for_the_example() {
        let (outcome, state) = run(&Fake::default(), &["my-app", "--example"]).await;

        assert_eq!(outcome, Outcome::Planned);
        assert!(
            state
                .plan
                .changes
                .iter()
                .any(|change| change.path() == "my-app/src/walks/counter.rs")
        );
    }

    #[tokio::test]
    async fn rejects_a_missing_or_invalid_name() {
        let (outcome, state) = run(&Fake::default(), &[]).await;
        assert_eq!(outcome, Outcome::Rejected);
        assert_eq!(state.message.as_deref(), Some(USAGE));

        let (outcome, state) = run(&Fake::default(), &["MyApp"]).await;
        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains("not a package name"));
    }

    #[tokio::test]
    async fn rejects_a_folder_that_exists() {
        let fake = Fake::with_files(&[("my-app", "")]);

        let (outcome, state) = run(&fake, &["my-app"]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert_eq!(state.message.as_deref(), Some("`my-app` already exists"));
        assert!(state.plan.changes.is_empty());
    }
}
