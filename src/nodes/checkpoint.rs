use crate::Node;
use crate::nodes::HasPaving;
use crate::utils::git;
use std::io;

/// Where [`Checkpoint`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The changes are committed, or there were none.
    Saved,
}

/// Why [`Checkpoint`] could not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not commit a checkpoint: {0}")]
    Io(#[from] io::Error),
}

/// The side effects [`Checkpoint`] needs: one method each, mocked in the tests.
pub trait Utils {
    /// Whether the working tree has uncommitted changes.
    fn dirty(&self) -> io::Result<bool>;
    /// Commits every change.
    fn commit(&self, subject: &str, body: &str) -> io::Result<()>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn dirty(&self) -> io::Result<bool> {
        Ok(!git::status()?.trim().is_empty())
    }

    fn commit(&self, subject: &str, body: &str) -> io::Result<()> {
        git::commit_all(subject, body)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Commits the changes that brought the check to pass, named after the last task.
#[derive(Clone, Debug, Default)]
pub struct Checkpoint<U = SystemUtils> {
    utils: U,
}

impl Checkpoint {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Checkpoint<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Checkpoint<U>
where
    S: HasPaving + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        let paving = state.paving();
        let label = &paving.label;
        if self.utils.dirty()? {
            if paving.settings.dry_run {
                self.utils.print(&format!(
                    "[ambler pave] dry run: would commit a checkpoint: {label}"
                ));
            } else {
                self.utils.commit(
                    &format!("ambler-pave({}): {label}", paving.settings.spec),
                    "Checkpoint: ambler check passed.",
                )?;
                self.utils
                    .print(&format!("[ambler pave] checkpoint: {label}"));
            }
        }
        state.paving_mut().repairs = 0;
        Ok(Outcome::Saved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::pave::{Paving, Settings};
    use crate::testing::{Fake, State};
    use std::sync::Mutex;

    fn state(dry_run: bool) -> State {
        State {
            paving: Paving {
                settings: Settings {
                    spec: "game".to_string(),
                    dry_run,
                    ..Settings::default()
                },
                label: "node read_move".to_string(),
                repairs: 2,
                ..Paving::default()
            },
            ..State::default()
        }
    }

    fn dirty() -> Fake {
        Fake {
            dirty: Mutex::new(true),
            ..Fake::default()
        }
    }

    #[tokio::test]
    async fn commits_the_changes_named_after_the_last_task() {
        let fake = dirty();
        let mut state = state(false);

        let outcome = Checkpoint::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Saved);
        assert_eq!(fake.git(), ["commit ambler-pave(game): node read_move"]);
        assert_eq!(fake.printed(), ["[ambler pave] checkpoint: node read_move"]);
        assert_eq!(state.paving.repairs, 0);
    }

    #[tokio::test]
    async fn commits_nothing_on_a_clean_tree() {
        let fake = Fake::default();
        let mut state = state(false);

        let outcome = Checkpoint::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Saved);
        assert!(fake.git().is_empty());
        assert_eq!(state.paving.repairs, 0);
    }

    #[tokio::test]
    async fn only_says_what_it_would_commit_in_a_dry_run() {
        let fake = dirty();
        let mut state = state(true);

        let outcome = Checkpoint::with_utils(&fake).run(&mut state).await.unwrap();

        assert_eq!(outcome, Outcome::Saved);
        assert!(fake.git().is_empty());
        assert_eq!(
            fake.printed(),
            ["[ambler pave] dry run: would commit a checkpoint: node read_move"]
        );
    }

    #[tokio::test]
    async fn fails_if_git_fails() {
        let fake = Fake {
            failing: Some("git"),
            ..dirty()
        };

        let result = Checkpoint::with_utils(&fake).run(&mut state(false)).await;

        assert!(matches!(result, Err(Error::Io(_))));
    }
}
