use crate::Node;
use crate::nodes::HasCount;
use crate::utils::terminal;
use std::io;

/// Where [`Start`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The count was set, from the input or to 0 if there was none.
    Success,
    /// The input was not a number; the count is untouched.
    InvalidInput,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read the starting number")]
    Io(#[from] io::Error),
}

pub trait Utils {
    /// Prints `message` and reads a line, or `None` at end of input.
    fn read_line(&self, message: &str) -> io::Result<Option<String>>;
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn read_line(&self, message: &str) -> io::Result<Option<String>> {
        terminal::prompt(message)
    }

    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Asks for a starting number. Empty input or end of input counts as 0.
#[derive(Clone, Debug, Default)]
pub struct Start<U = SystemUtils> {
    utils: U,
}

impl Start {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Start<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Start<U>
where
    S: HasCount + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Error;

    async fn run(&self, state: &mut S) -> Result<Outcome, Error> {
        let input = self.utils.read_line("Enter a starting number: ")?;
        match input.as_deref() {
            None | Some("") => {
                *state.count_mut() = 0;
                Ok(Outcome::Success)
            }
            Some(input) => match input.parse() {
                Ok(n) => {
                    *state.count_mut() = n;
                    Ok(Outcome::Success)
                }
                Err(_) => {
                    self.utils
                        .print("Error: Invalid input. Please enter a number.");
                    Ok(Outcome::InvalidInput)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockState {
        count: i32,
    }

    impl HasCount for MockState {
        fn count(&self) -> i32 {
            self.count
        }
        fn count_mut(&mut self) -> &mut i32 {
            &mut self.count
        }
    }

    struct MockUtils {
        input: io::Result<Option<&'static str>>,
    }

    impl Utils for MockUtils {
        fn read_line(&self, _message: &str) -> io::Result<Option<String>> {
            match &self.input {
                Ok(input) => Ok(input.map(str::to_string)),
                Err(e) => Err(io::Error::new(e.kind(), "mock")),
            }
        }
        fn print(&self, _message: &str) {}
    }

    async fn run(input: &'static str, count: i32) -> (Outcome, i32) {
        let utils = MockUtils {
            input: Ok(Some(input)),
        };
        let mut state = MockState { count };
        let outcome = Start::with_utils(utils).run(&mut state).await.unwrap();
        (outcome, state.count)
    }

    #[tokio::test]
    async fn succeeds_with_0_if_input_is_empty() {
        assert_eq!(run("", 7).await, (Outcome::Success, 0));
    }

    #[tokio::test]
    async fn succeeds_with_0_at_end_of_input() {
        let utils = MockUtils { input: Ok(None) };
        let mut state = MockState { count: 7 };

        let outcome = Start::with_utils(utils).run(&mut state).await.unwrap();

        assert_eq!((outcome, state.count), (Outcome::Success, 0));
    }

    #[tokio::test]
    async fn succeeds_with_input_number() {
        assert_eq!(run("42", 0).await, (Outcome::Success, 42));
    }

    #[tokio::test]
    async fn reports_invalid_input_and_keeps_count() {
        assert_eq!(run("abc", 123).await, (Outcome::InvalidInput, 123));
    }

    #[tokio::test]
    async fn fails_if_input_cannot_be_read() {
        let utils = MockUtils {
            input: Err(io::ErrorKind::BrokenPipe.into()),
        };
        let mut state = MockState { count: 1 };

        let result = Start::with_utils(utils).run(&mut state).await;

        assert!(matches!(result, Err(Error::Io(_))));
        assert_eq!(state.count, 1);
    }
}
