use crate::Node;
use crate::nodes::HasPaving;
use crate::scaffold::pave::{FAILURE_LINES, PROMPT_LINES, tail};
use crate::utils::{fs, process};
use std::io;

/// Where [`CheckProject`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// `ambler check` passed.
    Passed,
    /// `ambler check` failed; the end of its output is recorded.
    Failed,
}

/// Why [`CheckProject`] could not complete.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not run `ambler check`: {0}")]
    Io(#[from] io::Error),
}

/// The side effects [`CheckProject`] needs: one method each, mocked in the tests.
pub trait Utils {
    /// Runs `ambler check` building in `target`, with its output in the file at `log`, and
    /// reports whether it passed.
    fn run_check(&self, log: &str, target: &str) -> io::Result<bool>;
    fn read(&self, path: &str) -> io::Result<String>;
    fn print(&self, message: &str);
    fn eprint(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn run_check(&self, log: &str, target: &str) -> io::Result<bool> {
        process::rerun_logged(&["check"], &[("CARGO_TARGET_DIR", target)], log)
    }

    fn read(&self, path: &str) -> io::Result<String> {
        fs::read(path)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }

    fn eprint(&self, message: &str) {
        eprintln!("{message}");
    }
}

/// Runs `ambler check` on the project and records whether it passed and how its output ends.
#[derive(Clone, Debug, Default)]
pub struct CheckProject<U = SystemUtils> {
    utils: U,
}

impl CheckProject {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> CheckProject<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for CheckProject<U>
where
    S: HasPaving + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        let paving = state.paving();
        self.utils.print("[ambler pave] ambler check …");
        let passed = self
            .utils
            .run_check(&paving.log, &paving.settings.build_dir)?;
        let output = self.utils.read(&paving.log)?;
        if !passed {
            self.utils.eprint("[ambler pave] ambler check failed:");
            self.utils.eprint(&tail(&output, FAILURE_LINES));
        }
        let paving = state.paving_mut();
        paving.passed = passed;
        paving.output = tail(&output, PROMPT_LINES);
        Ok(if passed {
            Outcome::Passed
        } else {
            Outcome::Failed
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::pave::Paving;
    use crate::testing::{Fake, State};
    use std::collections::VecDeque;

    const LOG: &str = ".git/ambler-pave-check.log";

    fn state() -> State {
        State {
            paving: Paving {
                log: LOG.to_string(),
                output: "stale".to_string(),
                ..Paving::default()
            },
            ..State::default()
        }
    }

    #[tokio::test]
    async fn records_a_passing_check() {
        let fake = Fake::default();
        let mut state = state();

        let outcome = CheckProject::with_utils(&fake)
            .run(&mut state)
            .await
            .unwrap();

        assert_eq!(outcome, Outcome::Passed);
        assert!(state.paving.passed);
        assert_eq!(state.paving.output, "");
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn records_and_prints_the_end_of_a_failing_check() {
        let output: String = (1..=100).map(|n| format!("line {n}\n")).collect();
        let output: &'static str = Box::leak(output.into_boxed_str());
        let fake = Fake {
            checks: std::sync::Mutex::new(VecDeque::from([(false, output)])),
            ..Fake::default()
        };
        let mut state = State {
            paving: Paving {
                passed: true,
                ..state().paving
            },
            ..State::default()
        };

        let outcome = CheckProject::with_utils(&fake)
            .run(&mut state)
            .await
            .unwrap();

        assert_eq!(outcome, Outcome::Failed);
        assert!(!state.paving.passed);
        assert!(state.paving.output.starts_with("line 21\n"));
        assert!(state.paving.output.ends_with("line 100"));
        let eprinted = fake.eprinted();
        assert_eq!(eprinted.len(), 2);
        assert!(eprinted[1].starts_with("line 86\n"));
    }

    #[tokio::test]
    async fn fails_if_the_check_cannot_be_run() {
        let fake = Fake {
            failing: Some("check"),
            ..Fake::default()
        };
        let mut state = state();

        let result = CheckProject::with_utils(&fake).run(&mut state).await;

        assert!(matches!(result, Err(Error::Io(_))));
        assert_eq!(state.paving.output, "stale");
    }
}
