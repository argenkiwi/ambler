//! Asks for a number, counts up from it once a second, and stops at random.
//!
//! See `specs/counter.md`.

use crate::nodes::HasCount;
use crate::nodes::count::{self, Count};
use crate::nodes::start::{self, Start};
use crate::nodes::stop::{self, Stop};
use crate::{Node, Transition, Walk};

/// What can stop the walk early. Only the start node can fail.
pub type Error = start::Error;

#[derive(Clone, Debug, Default)]
struct State {
    count: i32,
}

impl HasCount for State {
    fn count(&self) -> i32 {
        self.count
    }
    fn count_mut(&mut self) -> &mut i32 {
        &mut self.count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    Start,
    Count,
    Stop,
}

#[derive(Default)]
struct Counter<A = start::SystemUtils, B = count::SystemUtils, C = stop::SystemUtils> {
    start: Start<A>,
    count: Count<B>,
    stop: Stop<C>,
}

impl Counter {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B, C> Walk for Counter<A, B, C>
where
    A: start::Utils + Sync,
    B: count::Utils + Sync,
    C: stop::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::Start => match self.start.run(state).await? {
                start::Outcome::Success => Transition::Next(Id::Count),
                start::Outcome::InvalidInput => Transition::Next(Id::Start),
            },
            Id::Count => {
                let Ok(outcome) = self.count.run(state).await;
                match outcome {
                    count::Outcome::Again => Transition::Next(Id::Count),
                    count::Outcome::Finished => Transition::Next(Id::Stop),
                }
            }
            Id::Stop => {
                let Ok(()) = self.stop.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs the counter walk starting at 0.
///
/// # Errors
///
/// Returns [`Error`] if reading user input fails.
pub async fn run() -> Result<(), Error> {
    Counter::new().run(Id::Start, &mut State::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::io;
    use std::sync::Mutex;
    use std::time::Duration;

    /// Plays back typed lines and coin flips, and records everything printed.
    struct Script {
        lines: Mutex<VecDeque<&'static str>>,
        flips: Mutex<VecDeque<bool>>,
        printed: Mutex<Vec<String>>,
    }

    impl Script {
        fn new(lines: &[&'static str], flips: &[bool]) -> Self {
            Self {
                lines: Mutex::new(lines.iter().copied().collect()),
                flips: Mutex::new(flips.iter().copied().collect()),
                printed: Mutex::default(),
            }
        }

        fn record(&self, message: &str) {
            self.printed.lock().unwrap().push(message.to_string());
        }
    }

    impl start::Utils for &Script {
        fn read_line(&self, _message: &str) -> io::Result<Option<String>> {
            Ok(self.lines.lock().unwrap().pop_front().map(str::to_string))
        }
        fn print(&self, message: &str) {
            self.record(message);
        }
    }

    impl count::Utils for &Script {
        fn print(&self, message: &str) {
            self.record(message);
        }
        async fn sleep(&self, _duration: Duration) {}
        fn random_bool(&self, _p: f64) -> bool {
            self.flips.lock().unwrap().pop_front().unwrap()
        }
    }

    impl stop::Utils for &Script {
        fn print(&self, message: &str) {
            self.record(message);
        }
    }

    fn counter(script: &Script) -> Counter<&Script, &Script, &Script> {
        Counter {
            start: Start::with_utils(script),
            count: Count::with_utils(script),
            stop: Stop::with_utils(script),
        }
    }

    #[tokio::test]
    async fn retries_invalid_input_then_counts_until_the_coin_says_stop() {
        let script = Script::new(&["abc", "3"], &[true, false]);
        let mut state = State { count: 99 };

        counter(&script).run(Id::Start, &mut state).await.unwrap();

        assert_eq!(state.count, 5);
        assert_eq!(
            *script.printed.lock().unwrap(),
            [
                "Error: Invalid input. Please enter a number.",
                "Current count: 3",
                "Current count: 4",
                "Final count: 5",
            ]
        );
    }
}
