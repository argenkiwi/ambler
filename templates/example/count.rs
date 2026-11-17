use crate::Node;
use crate::nodes::HasCount;
use std::convert::Infallible;
use std::future::Future;
use std::time::Duration;

/// Where [`Count`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Keep counting.
    Again,
    /// Stop counting.
    Finished,
}

pub trait Utils {
    fn print(&self, message: &str);
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;
    /// Returns `true` with probability `p`.
    fn random_bool(&self, p: f64) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn print(&self, message: &str) {
        println!("{message}");
    }

    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        tokio::time::sleep(duration)
    }

    fn random_bool(&self, p: f64) -> bool {
        rand::random_bool(p)
    }
}

/// Prints the count, waits a second, increments it, then flips a coin between
/// [`Again`](Outcome::Again) and [`Finished`](Outcome::Finished).
#[derive(Clone, Debug, Default)]
pub struct Count<U = SystemUtils> {
    utils: U,
}

impl Count {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Count<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Count<U>
where
    S: HasCount + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Infallible> {
        self.utils
            .print(&format!("Current count: {}", state.count()));
        self.utils.sleep(Duration::from_secs(1)).await;
        *state.count_mut() += 1;

        Ok(if self.utils.random_bool(0.5) {
            Outcome::Again
        } else {
            Outcome::Finished
        })
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
        heads: bool,
    }

    impl Utils for MockUtils {
        fn print(&self, _message: &str) {}
        async fn sleep(&self, _duration: Duration) {}
        fn random_bool(&self, _p: f64) -> bool {
            self.heads
        }
    }

    async fn run(heads: bool, count: i32) -> (Outcome, i32) {
        let mut state = MockState { count };
        let Ok(outcome) = Count::with_utils(MockUtils { heads }).run(&mut state).await;
        (outcome, state.count)
    }

    #[tokio::test]
    async fn increments_count_and_goes_again_if_coin_lands_heads() {
        assert_eq!(run(true, 5).await, (Outcome::Again, 6));
    }

    #[tokio::test]
    async fn increments_count_and_finishes_if_coin_lands_tails() {
        assert_eq!(run(false, 10).await, (Outcome::Finished, 11));
    }
}
