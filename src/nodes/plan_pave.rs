use crate::Node;
use crate::nodes::{HasArgs, HasMessage, HasPaving, HasTarget};
use crate::scaffold::pave::{self, AGENT_VAR, BUILD_VAR, LOG_NAME, Paving};
use crate::scaffold::target::Target;
use crate::utils::{fs, git};
use std::convert::Infallible;

/// Where [`PlanPave`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The settings are recorded and the project can be paved.
    Ready,
    /// The arguments or the project do not allow it; the message says why.
    Rejected,
}

/// The side effects [`PlanPave`] needs: one method each, mocked in the tests.
pub trait Utils {
    fn exists(&self, path: &str) -> bool;
    /// The value of the environment variable `name`, if it is set.
    fn env(&self, name: &str) -> Option<String>;
    /// The repository's git folder, or `None` outside a repository.
    fn git_dir(&self) -> Option<String>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn exists(&self, path: &str) -> bool {
        fs::exists(path)
    }

    fn env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn git_dir(&self) -> Option<String> {
        git::git_dir()
    }
}

/// Reads what `ambler pave` was asked to do and checks the project can be paved.
#[derive(Clone, Debug, Default)]
pub struct PlanPave<U = SystemUtils> {
    utils: U,
}

impl PlanPave {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> PlanPave<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<U: Utils> PlanPave<U> {
    fn plan(&self, target: Target, args: &[String]) -> Result<Paving, String> {
        let settings = pave::settings(
            target,
            args,
            self.utils.env(AGENT_VAR),
            self.utils.env(BUILD_VAR),
        )?;
        if !target.is_project(|path| self.utils.exists(path)) {
            return Err("run this from the root of an Ambler project".to_string());
        }
        let spec = settings.spec_path();
        if !self.utils.exists(&spec) {
            return Err(format!(
                "no spec at {spec}: write it first (`ambler help spec`)"
            ));
        }
        let git_dir = self.utils.git_dir().ok_or(
            "not a git repository: pave commits a checkpoint after every passing check, so \
             run `git init` first",
        )?;
        Ok(Paving {
            settings,
            log: format!("{git_dir}/{LOG_NAME}"),
            ..Paving::default()
        })
    }
}

impl<S, U> Node<S> for PlanPave<U>
where
    S: HasTarget + HasArgs + HasPaving + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Infallible> {
        Ok(match self.plan(state.target(), state.args()) {
            Ok(paving) => {
                *state.paving_mut() = paving;
                Outcome::Ready
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

    const PROJECT: &[(&str, &str)] = &[
        ("Cargo.toml", ""),
        ("src/engine.rs", ""),
        ("src/nodes/mod.rs", ""),
        ("specs/game.md", ""),
    ];

    fn project() -> Fake {
        Fake {
            git_dir: Some("/repo/.git"),
            ..Fake::with_files(PROJECT)
        }
    }

    async fn run(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_args(args);
        let Ok(outcome) = PlanPave::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    #[tokio::test]
    async fn records_the_settings_and_the_log_in_the_git_folder() {
        let fake = Fake {
            env: vec![(AGENT_VAR, "pi -p"), (BUILD_VAR, "/workspace/target")],
            ..project()
        };

        let (outcome, state) = run(&fake, &["game", "--max-repairs", "1"]).await;

        assert_eq!(outcome, Outcome::Ready);
        assert_eq!(state.message, None);
        let paving = state.paving;
        assert_eq!(paving.settings.spec, "game");
        assert_eq!(paving.settings.agent.as_deref(), Some("pi -p"));
        assert_eq!(paving.settings.max_repairs, 1);
        assert_eq!(paving.settings.build_dir, "/workspace/target");
        assert_eq!(paving.settings.target, Target::Rust);
        assert_eq!(paving.log, format!("/repo/.git/{LOG_NAME}"));
    }

    #[tokio::test]
    async fn rejects_invalid_arguments_and_a_missing_agent() {
        let (outcome, state) = run(&project(), &["game"]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains(AGENT_VAR));
        assert_eq!(state.paving, Paving::default());
    }

    #[tokio::test]
    async fn rejects_a_spec_that_does_not_exist() {
        let (outcome, state) = run(&project(), &["chess", "--dry-run"]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains("no spec at specs/chess.md"));
    }

    #[tokio::test]
    async fn rejects_a_folder_that_is_not_an_ambler_project() {
        let fake = Fake {
            git_dir: Some("/repo/.git"),
            ..Fake::with_files(&[("specs/game.md", "")])
        };

        let (outcome, state) = run(&fake, &["game", "--dry-run"]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains("root of an Ambler project"));
    }

    #[tokio::test]
    async fn rejects_a_project_outside_a_git_repository() {
        let fake = Fake::with_files(PROJECT);

        let (outcome, state) = run(&fake, &["game", "--dry-run"]).await;

        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains("git init"));
    }
}
