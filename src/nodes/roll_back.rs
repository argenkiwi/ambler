use crate::Node;
use crate::nodes::HasPaving;
use crate::utils::git;
use std::io;

/// Where [`RollBack`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The changes since the last checkpoint are stashed, and the retry is used.
    RolledBack,
}

/// Why [`RollBack`] could not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not roll back to the last checkpoint: {0}")]
    Io(#[from] io::Error),
}

/// The side effects [`RollBack`] needs: one method each, mocked in the tests.
pub trait Utils {
    /// Stashes every change, untracked files included, under `message`.
    fn stash(&self, message: &str) -> io::Result<()>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn stash(&self, message: &str) -> io::Result<()> {
        git::stash_all(message)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Stashes the changes since the last checkpoint so the failed task can be retried once.
#[derive(Clone, Debug, Default)]
pub struct RollBack<U = SystemUtils> {
    utils: U,
}

impl RollBack {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> RollBack<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for RollBack<U>
where
    S: HasPaving + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        let paving = state.paving();
        let message = format!(
            "amble-pave({}): failed {} after {} repairs",
            paving.settings.spec, paving.label, paving.repairs
        );
        if paving.settings.dry_run {
            self.utils
                .print(&format!("[amble pave] dry run: would stash: {message}"));
        } else {
            self.utils.stash(&message)?;
            self.utils.print(&format!(
                "[amble pave] rolled back to the last checkpoint (stashed: \"{message}\"); \
                 retrying once"
            ));
        }
        let paving = state.paving_mut();
        paving.rolled_back = true;
        paving.repairs = 0;
        Ok(Outcome::RolledBack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::pave::{Paving, Settings};
    use crate::testing::{Fake, State};

    const STASH: &str = "amble-pave(game): failed node report after 3 repairs";

    fn state(dry_run: bool) -> State {
        State {
            paving: Paving {
                settings: Settings {
                    spec: "game".to_string(),
                    dry_run,
                    ..Settings::default()
                },
                label: "node report".to_string(),
                repairs: 3,
                ..Paving::default()
            },
            ..State::default()
        }
    }

    #[tokio::test]
    async fn stashes_the_changes_and_uses_the_retry() {
        let fake = Fake::default();
        let mut state = state(false);

        let outcome = RollBack::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::RolledBack);
        assert_eq!(fake.git(), [format!("stash {STASH}")]);
        assert!(fake.printed()[0].contains(STASH));
        assert!(state.paving.rolled_back);
        assert_eq!(state.paving.repairs, 0);
    }

    #[tokio::test]
    async fn only_says_what_it_would_stash_in_a_dry_run() {
        let fake = Fake::default();
        let mut state = state(true);

        let outcome = RollBack::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::RolledBack);
        assert!(fake.git().is_empty());
        assert_eq!(
            fake.printed(),
            [format!("[amble pave] dry run: would stash: {STASH}")]
        );
        assert!(state.paving.rolled_back);
    }

    #[tokio::test]
    async fn fails_if_git_fails() {
        let fake = Fake {
            failing: Some("git"),
            ..Fake::default()
        };
        let mut state = state(false);

        let result = RollBack::with_utils(&fake).run(&mut state).await;

        assert!(matches!(result, Err(Error::Io(_))));
        assert!(!state.paving.rolled_back);
    }
}
