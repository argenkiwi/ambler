use crate::Node;
use crate::nodes::{HasArgs, HasMessage, HasPlan, HasTarget};
use crate::scaffold::rust::project::is_package_name;
use crate::scaffold::target::Target;
use crate::scaffold::target::package_of;
use crate::scaffold::{Change, Plan, args, names, py, rust, ts};
use crate::utils::fs;
use std::convert::Infallible;
use std::io;

const USAGE: &str = "usage: amble new [<name>] [--example] [--target rust|ts|py]";

/// Every file of a new `target` project named `package` in `folder`.
fn project(target: Target, package: &str, folder: &str, example: bool) -> Vec<Change> {
    match target {
        Target::Rust => rust::project::project(package, folder, example),
        Target::TypeScript => ts::project::project(package, folder, example),
        Target::Python { .. } => {
            py::project::project(package, &package_of(package), folder, example)
        }
    }
}

/// Whether a `target` project can be named `name`, which is already a package name.
fn check_name(target: Target, name: &str) -> Result<(), String> {
    match target {
        Target::Python { .. } if !names::is_snake(&package_of(name)) => Err(format!(
            "`{name}` cannot be a Python package (`{}` is a keyword): choose another name",
            package_of(name)
        )),
        _ => Ok(()),
    }
}

/// What to do once the files are written, `then` being the first step.
fn next(target: Target, then: &str) -> String {
    let install = match target {
        Target::Rust => String::new(),
        Target::TypeScript => "`bun install`, then ".to_string(),
        Target::Python { .. } => "`uv sync`, then ".to_string(),
    };
    format!("Next: {then}{install}`amble help` for how to add nodes and walks.")
}

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
    /// The name of the current folder, if it has one.
    fn current_folder(&self) -> Option<String>;
    /// The names of the entries in the folder at `path`.
    fn entries(&self, path: &str) -> io::Result<Vec<String>>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn exists(&self, path: &str) -> bool {
        fs::exists(path)
    }

    fn current_folder(&self) -> Option<String> {
        fs::current_folder_name()
    }

    fn entries(&self, path: &str) -> io::Result<Vec<String>> {
        fs::entries(path)
    }
}

/// Plans the files of a new project from `<name> [--example]`, in the state's target.
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
    fn plan(&self, target: Target, args: &[String]) -> Result<Plan, String> {
        let parsed = args::parse(args, &[], &["example"]).map_err(|e| format!("{e}\n{USAGE}"))?;
        let example = parsed.has("example");
        let name = match parsed.positionals.as_slice() {
            [] => return self.plan_here(target, example),
            [name] => name,
            _ => return Err(USAGE.to_string()),
        };
        if !is_package_name(name) {
            return Err(format!(
                "`{name}` is not a package name: use lowercase letters, digits, `-` and `_`"
            ));
        }
        check_name(target, name)?;
        if self.utils.exists(name) {
            return Err(format!("`{name}` already exists"));
        }
        Ok(Plan {
            changes: project(target, name, name, example),
            hint: format!(
                "Created {name}/. {}",
                next(target, &format!("`cd {name}`, then "))
            ),
        })
    }

    /// Plans the project in the current folder, named after it. Only `.git` may be there.
    fn plan_here(&self, target: Target, example: bool) -> Result<Plan, String> {
        let Some(name) = self.utils.current_folder() else {
            return Err(format!("the current folder has no name\n{USAGE}"));
        };
        if !is_package_name(&name) {
            return Err(format!(
                "the folder name `{name}` is not a package name: rename the folder, or run \
                 `amble new <name>` to create a subfolder"
            ));
        }
        check_name(target, &name)?;
        let entries = self
            .utils
            .entries(".")
            .map_err(|e| format!("could not read the current folder: {e}"))?;
        if entries.iter().any(|entry| entry != ".git") {
            return Err(
                "the current folder is not empty: run `amble new <name>` to create a subfolder"
                    .to_string(),
            );
        }
        Ok(Plan {
            changes: project(target, &name, "", example),
            hint: format!("Created {name} here. {}", next(target, "")),
        })
    }
}

impl<S, U> Node<S> for PlanProject<U>
where
    S: HasTarget + HasArgs + HasPlan + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Infallible> {
        Ok(match self.plan(state.target(), state.args()) {
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

    fn in_folder(name: &'static str, files: &[(&str, &str)]) -> Fake {
        Fake {
            folder: Some(name),
            ..Fake::with_files(files)
        }
    }

    #[tokio::test]
    async fn without_a_name_plans_the_project_in_the_empty_current_folder() {
        let (outcome, state) = run(&in_folder("my-app", &[]), &["--example"]).await;

        assert_eq!(outcome, Outcome::Planned);
        let paths: Vec<&str> = state.plan.changes.iter().map(|c| c.path()).collect();
        assert!(paths.contains(&"src/engine.rs"));
        assert!(paths.contains(&"src/walks/counter.rs"));
        let cargo = state.plan.changes.iter().find_map(|change| match change {
            crate::scaffold::Change::Write { path, contents, .. } if path == "Cargo.toml" => {
                Some(contents.clone())
            }
            _ => None,
        });
        assert!(cargo.unwrap().contains("name = \"my-app\""));
        assert!(state.plan.hint.starts_with("Created my-app here."));
        assert!(!state.plan.hint.contains("cd "));
    }

    #[tokio::test]
    async fn without_a_name_ignores_a_git_folder() {
        let fake = in_folder("my-app", &[(".git/HEAD", "ref: refs/heads/main")]);

        let (outcome, _) = run(&fake, &[]).await;

        assert_eq!(outcome, Outcome::Planned);
    }

    #[tokio::test]
    async fn without_a_name_rejects_a_current_folder_that_is_not_empty() {
        let fake = in_folder("my-app", &[(".git/HEAD", ""), ("notes.txt", "")]);

        let (outcome, state) = run(&fake, &[]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert!(
            state
                .message
                .unwrap()
                .starts_with("the current folder is not empty")
        );
        assert!(state.plan.changes.is_empty());
    }

    #[tokio::test]
    async fn without_a_name_rejects_a_current_folder_named_unlike_a_package() {
        let (outcome, state) = run(&in_folder("My App", &[]), &[]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert!(
            state
                .message
                .unwrap()
                .starts_with("the folder name `My App` is not a package name")
        );
    }

    #[tokio::test]
    async fn rejects_a_missing_or_invalid_name() {
        let (outcome, state) = run(&Fake::default(), &["a", "b"]).await;
        assert_eq!(outcome, Outcome::Rejected);
        assert_eq!(state.message.as_deref(), Some(USAGE));

        let (outcome, state) = run(&Fake::default(), &["MyApp"]).await;
        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains("not a package name"));
    }

    #[tokio::test]
    async fn plans_a_typescript_project_for_that_target() {
        let mut state = State::with_target(Target::TypeScript, &["my-app", "--example"]);
        let Ok(outcome) = PlanProject::with_utils(&Fake::default())
            .run(&mut state)
            .await;

        assert_eq!(outcome, Outcome::Planned);
        let paths: Vec<&str> = state.plan.changes.iter().map(|c| c.path()).collect();
        assert!(paths.contains(&"my-app/ambler.ts"));
        assert!(paths.contains(&"my-app/walks/counter.ts"));
        assert!(!paths.contains(&"my-app/src/engine.rs"));
        assert!(
            state
                .plan
                .hint
                .contains("`cd my-app`, then `bun install`, then")
        );
    }

    #[tokio::test]
    async fn plans_a_python_project_in_a_package_named_after_it() {
        let target = Target::Python { package: "" };
        let mut state = State::with_target(target, &["my-app", "--example"]);
        let Ok(outcome) = PlanProject::with_utils(&Fake::default())
            .run(&mut state)
            .await;

        assert_eq!(outcome, Outcome::Planned);
        let paths: Vec<&str> = state.plan.changes.iter().map(|c| c.path()).collect();
        assert!(paths.contains(&"my-app/src/my_app/engine.py"));
        assert!(paths.contains(&"my-app/src/my_app/walks/counter.py"));
        assert!(paths.contains(&"my-app/tests/walks/test_counter.py"));
        assert!(
            state
                .plan
                .hint
                .contains("`cd my-app`, then `uv sync`, then")
        );

        let mut state = State::with_target(target, &["import"]);
        let Ok(outcome) = PlanProject::with_utils(&Fake::default())
            .run(&mut state)
            .await;
        assert_eq!(outcome, Outcome::Rejected);
        assert!(
            state
                .message
                .unwrap()
                .contains("cannot be a Python package")
        );
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
