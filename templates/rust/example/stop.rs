use crate::Node;
use crate::nodes::HasCount;
use std::convert::Infallible;

pub trait Utils {
    fn print(&self, message: &str);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Prints the final count. Terminal: it has no outgoing edges.
#[derive(Clone, Debug, Default)]
pub struct Stop<U = SystemUtils> {
    utils: U,
}

impl Stop {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> Stop<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<S, U> Node<S> for Stop<U>
where
    S: HasCount + Send,
    U: Utils + Sync,
{
    type Outcome = ();
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<(), Infallible> {
        self.utils.print(&format!("Final count: {}", state.count()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

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

    #[derive(Default)]
    struct MockUtils {
        message: Mutex<Option<String>>,
    }

    impl Utils for &MockUtils {
        fn print(&self, message: &str) {
            *self.message.lock().unwrap() = Some(message.to_string());
        }
    }

    #[tokio::test]
    async fn prints_final_count_and_leaves_state_untouched() {
        let utils = MockUtils::default();
        let mut state = MockState { count: 15 };

        let Ok(()) = Stop::with_utils(&utils).run(&mut state).await;

        assert_eq!(state.count, 15);
        assert_eq!(
            utils.message.lock().unwrap().as_deref(),
            Some("Final count: 15")
        );
    }
}
