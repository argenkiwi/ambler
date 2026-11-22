use crate::Node;
use crate::nodes::{HasArgs, HasMessage, HasPlan, HasTarget};
use crate::scaffold::rust::register::register;
use crate::scaffold::target::Target;
use crate::scaffold::{Change, Plan, args, names};
use crate::utils::fs;
use std::convert::Infallible;
use std::io;

const USAGE: &str = "usage: amble util <name>";
const MODULES: &str = "src/utils/mod.rs";

/// Where [`PlanUtil`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The plan holds the utility module and its registration.
    Planned,
    /// The arguments or the project do not allow it; the message says why.
    Rejected,
}

pub trait Utils {
    fn exists(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> io::Result<String>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn exists(&self, path: &str) -> bool {
        fs::exists(path)
    }

    fn read(&self, path: &str) -> io::Result<String> {
        fs::read(path)
    }
}

/// Plans a new utility module: in Rust, in `src/utils/` and registered; in TypeScript, in `utils/`.
#[derive(Clone, Debug, Default)]
pub struct PlanUtil<U = SystemUtils> {
    utils: U,
}

impl PlanUtil {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> PlanUtil<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<U: Utils> PlanUtil<U> {
    fn plan(&self, target: Target, args: &[String]) -> Result<Plan, String> {
        let parsed = args::parse(args, &[], &[]).map_err(|e| format!("{e}\n{USAGE}"))?;
        let [name] = parsed.positionals.as_slice() else {
            return Err(USAGE.to_string());
        };
        if target == Target::TypeScript {
            return self.plan_ts(&name.replace('-', "_"));
        }
        if !names::is_snake(name) {
            return Err(format!("`{name}` is not a module name: use snake_case"));
        }
        let path = format!("src/utils/{name}.rs");
        if self.utils.exists(&path) {
            return Err(format!("{path} already exists"));
        }
        let modules = self.utils.read(MODULES).map_err(|_| {
            format!("{MODULES} not found: run this from the root of an Ambler project")
        })?;
        let modules = register(&modules, name)
            .ok_or_else(|| format!("`{name}` is already registered in {MODULES}"))?;
        Ok(Plan {
            changes: vec![
                Change::write(path, "//! TODO: say what side effects this module wraps.\n"),
                Change::write(MODULES, modules),
            ],
            hint: format!(
                "Next: add documented functions to src/utils/{name}.rs and call them from a \
                 node's `SystemUtils`. See `amble help util`."
            ),
        })
    }

    fn plan_ts(&self, name: &str) -> Result<Plan, String> {
        if !names::is_snake(name) {
            return Err(format!(
                "`{name}` is not a module name: use snake_case or kebab-case"
            ));
        }
        if !self.utils.exists("ambler.ts") {
            return Err(
                "ambler.ts not found: run this from the root of an Ambler project".to_string(),
            );
        }
        let path = Target::TypeScript.util_path(name);
        if self.utils.exists(&path) {
            return Err(format!("{path} already exists"));
        }
        Ok(Plan {
            changes: vec![Change::write(
                &path,
                "// TODO: say what side effects this module wraps.\n",
            )],
            hint: format!(
                "Next: export documented functions from {path} and use them in a node's \
                 `defaultUtils`. See `amble help util`."
            ),
        })
    }
}

impl<S, U> Node<S> for PlanUtil<U>
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
        let Ok(outcome) = PlanUtil::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    #[tokio::test]
    async fn plans_a_typescript_module_without_registering_it() {
        let fake = Fake::with_files(&[("ambler.ts", "")]);
        let mut state = State::with_target(Target::TypeScript, &["llm-chat"]);

        let Ok(outcome) = PlanUtil::with_utils(&fake).run(&mut state).await;

        assert_eq!(outcome, Outcome::Planned);
        assert_eq!(
            state.plan.changes,
            [Change::write(
                "utils/llm-chat.ts",
                "// TODO: say what side effects this module wraps.\n"
            )]
        );
        assert!(state.plan.hint.contains("`defaultUtils`"));
    }

    #[tokio::test]
    async fn plans_the_module_and_registers_it() {
        let fake = Fake::with_files(&[(MODULES, "//! Utilities.\n\npub mod terminal;\n")]);

        let (outcome, state) = run(&fake, &["journal"]).await;

        assert_eq!(outcome, Outcome::Planned);
        assert_eq!(state.plan.changes[0].path(), "src/utils/journal.rs");
        assert_eq!(
            state.plan.changes[1],
            Change::write(
                MODULES,
                "//! Utilities.\n\npub mod journal;\npub mod terminal;\n"
            )
        );
    }

    #[tokio::test]
    async fn rejects_bad_names_and_modules_that_exist() {
        let fake =
            Fake::with_files(&[(MODULES, "pub mod terminal;\n"), ("src/utils/clock.rs", "")]);

        for (args, reason) in [
            (&[][..], USAGE),
            (&["Journal"][..], "snake_case"),
            (&["clock"][..], "src/utils/clock.rs already exists"),
            (&["terminal"][..], "already registered"),
        ] {
            let (outcome, state) = run(&fake, args).await;
            assert_eq!(outcome, Outcome::Rejected);
            assert!(state.message.unwrap().contains(reason), "{args:?}");
        }
    }

    #[tokio::test]
    async fn rejects_a_folder_that_is_not_a_project() {
        let (outcome, state) = run(&Fake::default(), &["journal"]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains("root of an Ambler project"));
    }
}
